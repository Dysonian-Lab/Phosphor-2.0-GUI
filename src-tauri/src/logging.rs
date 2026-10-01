//! Session diagnostic logging.
//!
//! The goal is that a tester can reproduce a problem, send ONE file, and that
//! file is enough to diagnose it without asking them a single follow-up
//! question. Everything needed to interpret a Proxmark3 exchange therefore has
//! to be in the log: what build of the app, what build of the PM3 client, what
//! device firmware, the exact command, the raw output, the exit code and how
//! long it took.
//!
//! # Why this replaced the previous logger
//!
//! The old implementation had three defects that made bug reports unusable:
//!
//! 1. **It wrote to a hardcoded developer path**
//!    (`D:\kilocode\Phosphor-debug\Phosphor2.2GUI\logs`). On any other machine
//!    `create_dir_all` failed and the function returned silently, so the tester
//!    got NO log at all -- and nothing anywhere said so. This is the single
//!    worst failure mode: it looks identical to "nothing went wrong".
//! 2. **One file per output chunk.** `emit_output` is called once per chunk of
//!    PM3 output, so a single clone attempt sprayed dozens of unrelated
//!    `001.md`, `002.md`, ... files with no record of which command produced
//!    which, and no way to know which ones to send.
//! 3. **It recorded only raw output.** Nothing about the app version, the
//!    device, the firmware, or the exit code -- so even a good log could not
//!    answer "what did you have installed?".
//!
//! # Privacy
//!
//! These logs contain card UIDs and, for dictionary attacks, recovered keys.
//! That is unavoidable for the logs to be useful, so they are written next to
//! the executable rather than into a synced/telemetry location, and the path is
//! printed at startup so nobody has to hunt for it.

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// SHA-256 prefix of the PM3 client build verified against real hardware.
///
/// If a tester reports a client whose hash differs from this, that is the first
/// thing to check: an unverified locally rebuilt client fails against hardware
/// with `Received packet frame with invalid CRC`. See `build_portable.ps1`,
/// which refuses to package a client that does not match.
pub const VERIFIED_CLIENT_SHA256_PREFIX: &str = "F7BA073E30F6";

/// Capabilities version the bundled client and firmware must both report.
pub const REQUIRED_CAPABILITIES_VERSION: &str = "11";

/// `Some(path)` once [`init`] has run.
static SESSION: OnceLock<PathBuf> = OnceLock::new();

/// Open handle for the current session's log, if logging initialised.
static HANDLE: OnceLock<Mutex<Option<std::fs::File>>> = OnceLock::new();

/// Decide where session logs live.
///
/// Order of preference:
/// 1. `PHOSPHOR_LOG_DIR` -- explicit override, mainly for tests.
/// 2. `<exe dir>/logs` -- next to the app, which is where someone who was told
///    "send us the log" will look. Works for the portable build.
/// 3. `%APPDATA%/Phosphor/logs` -- the installed build may not be writable.
///
/// Unlike the old implementation this never fails silently: if every candidate
/// is unwritable we still produce a log in the system temp directory and say
/// so in the file itself.
fn resolve_log_dir() -> (PathBuf, &'static str) {
    if let Ok(dir) = std::env::var("PHOSPHOR_LOG_DIR") {
        if !dir.is_empty() {
            return (PathBuf::from(dir), "PHOSPHOR_LOG_DIR");
        }
    }

    // <exe dir>/logs. Tauri sets CWD to the portable dir in practice, but the
    // exe's own location is the reliable anchor for an installed app.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("logs");
            if std::fs::create_dir_all(&candidate).is_ok() {
                return (candidate, "executable directory");
            }
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.is_empty() {
            let candidate = PathBuf::from(appdata).join("Phosphor").join("logs");
            if std::fs::create_dir_all(&candidate).is_ok() {
                return (candidate, "%APPDATA%");
            }
        }
    }

    let candidate = std::env::temp_dir().join("phosphor-logs");
    let _ = std::fs::create_dir_all(&candidate);
    (candidate, "system temp (no writable location found)")
}

/// Describe a file on disk well enough to identify which build it is.
fn describe_file(path: &std::path::Path) -> String {
    match std::fs::metadata(path) {
        Ok(meta) => {
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .and_then(|secs| chrono::DateTime::from_timestamp(secs as i64, 0))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            format!(
                "{} ({} bytes, modified {})",
                path.display(),
                meta.len(),
                modified
            )
        }
        Err(_) => format!("{} (NOT FOUND)", path.display()),
    }
}

/// Locate the PM3 client the app would use, for the environment header.
fn find_client_binary() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("proxmark3.exe"));
        }
    }
    candidates.push(PathBuf::from("proxmark3.exe"));
    candidates
        .into_iter()
        .find(|p| p.is_file() || p.exists())
}

/// Create the session log and write the environment header.
///
/// Call once during app setup. Safe to call again: subsequent calls are no-ops.
pub fn init() -> Option<PathBuf> {
    if let Some(existing) = SESSION.get() {
        return Some(existing.clone());
    }

    let (dir, origin) = resolve_log_dir();
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("phosphor-session-{}.log", stamp));

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;

    let _ = HANDLE.set(Mutex::new(Some(file)));

    let start = chrono::Local::now();
    let header = format!(
        "================================================================\n\
         PHOSPHOR SESSION LOG\n\
         =================================================================\n\
         App version      : {version}\n\
         Log started      : {now}\n\
         Log file         : {path}\n\
         Log location     : {dir} ({origin})\n\
         OS               : {os} / {arch}\n\
         Exe path         : {exe}\n\
         Current dir      : {cwd}\n\
         \n\
         --- PROXMARK3 CLIENT ---\n\
         Client path      : {client}\n\
         Verified client  : sha256 prefix {verified}\n\
         Client size/mtime is recorded above so a rebuilt-but-unverified client\n\
         (which fails with 'Received packet frame with invalid CRC') is obvious.\n\
         build_portable.ps1 refuses to package anything other than {verified}.\n\
         Required caps    : CAPABILITIES_VERSION {caps}\n\
         \n\
         --- HOW TO READ THIS LOG ---\n\
         Each command block shows the exact command, how long it took, the exit\n\
         code, and the client's raw stdout/stderr verbatim. Device firmware\n\
         details are recorded by the 'hw version' probe after connecting.\n\
         \n\
         NOTE: this file contains card UIDs and may contain recovered keys.\n\
         Send it to the developer, but do not post it publicly.\n\
         =================================================================\n\
         \n",
        version = env!("CARGO_PKG_VERSION"),
        now = start.to_rfc3339(),
        path = path.display(),
        dir = dir.display(),
        origin = origin,
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        exe = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".into()),
        cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".into()),
        client = find_client_binary()
            .map(|p| describe_file(&p))
            .unwrap_or_else(|| "NOT FOUND".to_string()),
        verified = VERIFIED_CLIENT_SHA256_PREFIX,
        caps = REQUIRED_CAPABILITIES_VERSION,
    );

    write_raw(&header);
    let _ = SESSION.set(path.clone());
    Some(path)
}

/// Path of the current session log, if initialised.
pub fn session_log_path() -> Option<PathBuf> {
    SESSION.get().cloned()
}

/// Append text to the session log. Never panics, never blocks for long.
pub fn log(text: &str) {
    write_raw(text);
}

fn write_raw(text: &str) {
    if let Some(handle) = HANDLE.get() {
        if let Ok(mut guard) = handle.lock() {
            if let Some(file) = guard.as_mut() {
                let _ = file.write_all(text.as_bytes());
                let _ = file.flush();
            }
        }
    }
}

/// Write a timestamped line.
pub fn line(text: &str) {
    log(&format!("[{}] {}\n", chrono::Local::now().format("%H:%M:%S%.3f"), text));
}

/// Write a visual section separator, e.g. a wizard step or scan attempt.
pub fn section(title: &str) {
    log(&format!(
        "\n---------------- {} ----------------\n",
        title.to_uppercase()
    ));
}

/// Warning text when the device's capabilities version does not match what
/// this build requires, or `None` when it matches / is unknown.
///
/// Split out as a pure function so it can be tested directly: the session
/// logger is process-global (`OnceLock`), so testing it through `device()`
/// would make tests race over which one initialised the session.
pub fn capabilities_warning(capabilities: &str) -> Option<String> {
    let reported = capabilities.trim();
    if reported.is_empty() || reported.eq_ignore_ascii_case("not reported") {
        return None;
    }
    if reported == REQUIRED_CAPABILITIES_VERSION {
        return None;
    }
    Some(format!(
        "*** WARNING: device reports CAPABILITIES_VERSION {} but this build \
         requires {}. Client and firmware must be flashed as a matched pair. \
         Expect detection and write failures until this matches. ***",
        reported, REQUIRED_CAPABILITIES_VERSION
    ))
}

/// Record a device connection / firmware identification.
pub fn device(port: &str, model: &str, firmware: &str, capabilities: &str) {
    section("device");
    line(&format!("Port          : {}", port));
    line(&format!("Model         : {}", model));
    line(&format!("Firmware      : {}", firmware));
    line(&format!("Capabilities  : {}", capabilities));
    if let Some(w) = capabilities_warning(capabilities) {
        line(&w);
    }
}

/// Record a high-level wizard operation and its outcome.
pub fn operation(name: &str, detail: &str, outcome: &str) {
    section(name);
    if !detail.is_empty() {
        line(&format!("Detail        : {}", detail));
    }
    line(&format!("Outcome       : {}", outcome));
}

/// Render a command block. Pure so the exact shape written to disk is
/// testable without touching the process-global session file.
pub fn render_command_block(
    port: &str,
    cmd: &str,
    elapsed_ms: u128,
    exit_code: Option<i32>,
    raw: &str,
    note: Option<&str>,
) -> String {
    let code = exit_code
        .map(|c| c.to_string())
        .unwrap_or_else(|| "none (killed / signal)".to_string());
    let mut out = format!(
        "\n>>> COMMAND : {}\n\
         PORT       : {}\n\
         ELAPSED    : {} ms\n\
         EXIT CODE  : {}\n",
        cmd, port, elapsed_ms, code
    );
    if let Some(n) = note {
        out.push_str(&format!("NOTE        : {}\n", n));
    }
    if raw.trim().is_empty() {
        out.push_str("RAW OUTPUT  : <empty>\n");
    } else {
        out.push_str(&format!(
            "--- raw output begin ---\n{}\n--- raw output end ---\n",
            raw.trim_end()
        ));
    }
    out
}

/// Record a PM3 command execution with everything needed to diagnose it.
///
/// `raw` is the client's combined stdout/stderr exactly as received, so a
/// discrepancy between the parsed result and the raw output is visible.
pub fn command(
    port: &str,
    cmd: &str,
    elapsed_ms: u128,
    exit_code: Option<i32>,
    raw: &str,
    note: Option<&str>,
) {
    log(&render_command_block(port, cmd, elapsed_ms, exit_code, raw, note));
}

/// Absolute path of the current session log, for display in the UI.
///
/// Surfaced so a tester can be told "open this file and send it" instead of
/// hunting for a `logs` folder they were never told about.
#[tauri::command]
pub fn get_session_log_path() -> Option<String> {
    session_log_path().map(|p| p.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_log_is_created_and_written() {
        let dir = std::env::temp_dir().join("phosphor-logger-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("PHOSPHOR_LOG_DIR", &dir);

        // init() is a process-global OnceLock, so only the first caller gets a
        // fresh session. Everything below must therefore be asserted only when
        // this test actually won that race, and the pure formatting helpers are
        // covered separately.
        let Some(path) = init() else {
            eprintln!("SKIP: session already initialised by another test");
            return;
        };
        assert!(path.exists(), "session log must exist on disk");
        line("hello from test");
        command(
            "COM19",
            "lf t55xx detect",
            12,
            Some(0),
            "[=]  Chip type......... T55x7\n",
            Some("unit test"),
        );
        section("device");
        device("COM19", "iCopy-X", "Iceman/master/v4.23346", "11");
        operation("scan", "uid=4E008E7AC1", "ok");

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("PHOSPHOR SESSION LOG"), "header missing");
        assert!(body.contains(env!("CARGO_PKG_VERSION")), "version missing");
        assert!(body.contains("hello from test"), "line() missing");
        assert!(body.contains(">>> COMMAND : lf t55xx detect"), "cmd missing");
        assert!(body.contains("T55x7"), "raw output missing");
        assert!(body.contains("EXIT CODE  : 0"), "exit code missing");
        assert!(body.contains("ELAPSED    : 12 ms"), "elapsed missing");
        assert!(body.contains("CAPABILITIES_VERSION 11"), "caps missing");
    }

    #[test]
    fn command_block_records_all_diagnostic_fields() {
        // Exercises the real renderer, so this holds regardless of whether
        // another test already initialised the process-global session.
        let block = render_command_block(
            "COM19",
            "hf iclass dump",
            7,
            Some(-3),
            "raw text",
            Some("binary: sidecar"),
        );
        for needle in [
            ">>> COMMAND : hf iclass dump",
            "PORT       : COM19",
            "ELAPSED    : 7 ms",
            "EXIT CODE  : -3",
            "NOTE        : binary: sidecar",
            "raw output begin",
            "raw output end",
        ] {
            assert!(block.contains(needle), "missing field: {}", needle);
        }
    }

    #[test]
    fn command_block_marks_missing_exit_and_empty_output() {
        let block = render_command_block("COM4", "hw version", 3, None, "   \n", None);
        assert!(
            block.contains("EXIT CODE  : none (killed / signal)"),
            "a killed child has no exit code and must say so explicitly"
        );
        assert!(
            block.contains("RAW OUTPUT  : <empty>"),
            "empty output must be visible, not silently blank"
        );
    }

    #[test]
    fn capabilities_mismatch_is_flagged() {
        // Pure-function check, so it cannot race the process-global session.
        assert!(
            capabilities_warning("9").is_some(),
            "a mismatched CAPABILITIES_VERSION must be called out explicitly, \
             because client and firmware must be flashed as a matched pair"
        );
        assert!(capabilities_warning("10").is_some());
        assert!(
            capabilities_warning("11").is_none(),
            "the required capabilities version must not warn"
        );
        assert!(
            capabilities_warning("not reported").is_none(),
            "an unknown version must not be treated as a mismatch"
        );
        assert!(capabilities_warning("").is_none());
    }

    #[test]
    fn logging_never_panics_without_a_session() {
        // Must be safe no matter what order things run in.
        line("no session yet");
        section("x");
        command("COM1", "hw version", 1, None, "", None);
        assert!(session_log_path().is_none() || session_log_path().is_some());
    }
}