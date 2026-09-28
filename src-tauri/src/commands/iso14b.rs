use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::command_builder;
use crate::pm3::connection::{self, HfOperationState};
use crate::state::{WizardMachine, WizardState};

/// Run `hf 14b view -f <file>` — decode a SRIX4K dump (MyKey/COGES).
#[tauri::command]
pub async fn iso14b_view(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    file: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = format!("hf 14b view -f {}", file);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf 14b info` and return a tidy structure.
#[tauri::command]
pub async fn iso14b_info(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<crate::Iso14bInfo, AppError> {
    let port = get_port(&machine)?;
    let raw = connection::run_command_quick(&app, &port, "hf 14b info").await?;
    let info = parse_iso14b_output(&raw)?;
    Ok(info)
}

/// Run `hf 14b rdbl -b <block>` — read SRI512/SRIX4 block.
#[tauri::command]
pub async fn iso14b_rdbl(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    block: u16,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = format!("hf 14b rdbl -b {}", block);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf 14b ctrdbl -b <block>` — read ASK CTS/C-ticket block.
#[tauri::command]
pub async fn iso14b_ctrdbl(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    block: u16,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = format!("hf 14b ctrdbl -b {}", block);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf 14b view --selftest` — run MyKey parser self tests (offline).
#[tauri::command]
pub async fn iso14b_view_selftest(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf 14b view --selftest").await
}

/// Run `hf 14a antifuzz` — fuzz ISO14443a anticollision.
#[tauri::command]
pub async fn hf_14a_antifuzz(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf 14a antifuzz").await
}

/// Run `hf 14a antifuzz --coll` — collision storm mode.
#[tauri::command]
pub async fn hf_14a_antifuzz_coll(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf 14a antifuzz --coll").await
}

/// Run `hf 14a reader -n 1` — single poll for ISO14443a tag.
/// Returns UID, ATQA, SAK if a card is found, or empty if no card detected.
#[tauri::command]
pub async fn hf_14a_reader_single(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf 14a reader -n 1").await
}

/// Run `hf 14a reader -n <count>` — poll for ISO14443a tag multiple times.
/// Returns success rate and card details. Use for measuring read reliability at different positions.
#[tauri::command]
pub async fn hf_14a_reader_multiple(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    count: u32,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = format!("hf 14a reader -n {}", count);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf 14a info` — detailed ISO14443a tag information.
#[tauri::command]
pub async fn hf_14a_info(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf 14a info").await
}

/// Run `hf mf view -f <file>` — decode MIFARE Classic dump (RKF/VIGIK/HID PACS).
#[tauri::command]
pub async fn hf_mf_view(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    file: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = format!("hf mf view -f {}", file);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf mf view --selftest` — run dump parser self tests (offline).
#[tauri::command]
pub async fn hf_mf_view_selftest(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf mf view --selftest").await
}

/// Run `hf mf dump` — dump a MIFARE Classic card to a binary file.
///
/// Requires known keys (run `hf mf autopwn` first, or use `hf mf cview` for
/// magic cards with the backdoor enabled). Writes `hf-mf-<uid>-dump.bin` to
/// the current working directory. The dump file path is also stored in
/// `HfOperationState` so the write/verify phases can find it.
#[tauri::command]
pub async fn hf_mf_dump(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    hf_state: State<'_, HfOperationState>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let output = connection::run_command(&app, &port, "hf mf dump").await?;

    // Store the dump path so subsequent operations (view, write, verify) can find it
    if let Some(path) = crate::pm3::output_parser::extract_dump_file_path(&output) {
        if let Ok(mut lock) = hf_state.dump_path.lock() {
            *lock = Some(path);
        }
    }

    Ok(output)
}

/// Run `hf mf cview` — backdoor read of all MIFARE Classic blocks.
///
/// Works on magic cards (Gen1a/Gen2/Gen3/Gen4) without needing keys, because
/// the backdoor lets us bypass authentication. Does NOT write a dump file —
/// it just prints the blocks to stdout. Useful for quick inspection.
#[tauri::command]
pub async fn hf_mf_cview(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf mf cview").await
}

/// Run `hf mf autopwn` — automatic key recovery for MIFARE Classic.
///
/// Recovers keys for all sectors and writes `hf-mf-<uid>-key.bin` and
/// `hf-mf-<uid>-dump.bin` to the current working directory. This is the
/// recommended way to create a dump file — it's what the wizard scan flow
/// runs internally.
#[tauri::command]
pub async fn hf_mf_autopwn(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf mf autopwn").await
}

/// Nested attack. `hf mf nested <cardtype> <blk> <a|b> <key> [t]`.
#[tauri::command]
pub async fn mf_nested(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    card_type: String,
    block: u16,
    key_type: String,
    key: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_nested(&card_type, block, &key_type, &key);
    connection::run_command(&app, &port, &cmd).await
}

/// Emulator: fill memory from keys. `hf mf ecfill` (v4.23346).
#[tauri::command]
pub async fn mf_ecfill(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf mf ecfill").await
}

/// Emulator: save memory to a dump file. `hf mf esave [--4k] -f <filename>`.
#[tauri::command]
pub async fn mf_esave(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    filename: Option<String>,
    size: Option<String>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_esave(filename.as_deref(), size.as_deref());
    connection::run_command(&app, &port, &cmd).await
}

/// Emulator: simulate the loaded card. `hf mf sim [--1k] [-u <uid>]`.
#[tauri::command]
pub async fn mf_sim(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    size: Option<String>,
    uid: Option<String>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_sim(size.as_deref(), uid.as_deref());
    connection::run_command(&app, &port, &cmd).await
}

/// Emulator: clear memory. `hf mf eclr`.
#[tauri::command]
pub async fn mf_eclr(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "hf mf eclr").await
}

/// Emulator: load a dump file. `hf mf eload [--4k] -f <filename>`.
#[tauri::command]
pub async fn mf_eload(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    filename: String,
    size: Option<String>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_eload(&filename, size.as_deref());
    connection::run_command(&app, &port, &cmd).await
}

/// Emulator: get a single block. `hf mf ejectblk --blk <n>`.
#[tauri::command]
pub async fn mf_egetblk(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    block: u16,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_egetblk(block);
    connection::run_command(&app, &port, &cmd).await
}

/// Emulator: set a single block. `hf mf esetblk --blk <n> -d <data>`.
#[tauri::command]
pub async fn mf_esetblk(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    block: u16,
    data: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_esetblk(block, &data);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `lf t55xx set config` — configure T55xx tag parameters.
#[tauri::command]
pub async fn lf_t55xx_set_config(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx set config").await
}

/// Run `lf t55xx chk pwd` — check T55xx passwords.
#[tauri::command]
pub async fn lf_t55xx_chk_pwds(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx chk pwd").await
}

/// Run `lf t55xx danger raw` — write raw data to T55xx.
#[tauri::command]
pub async fn lf_t55xx_dangerraw(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx danger raw").await
}

/// Run `lf t55xx wakeup` — wake up T55xx tag.
#[tauri::command]
pub async fn lf_t55xx_wakeup(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx wakeup").await
}

/// Run `hf mf rdbl --blk <n> -k <key>` — read single block with key (v4.23346).
#[tauri::command]
pub async fn mf_rdbl(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    block: u16,
    key: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_rdbl(block, &key);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf mf rdsc -s <n> -k <key>` — read full sector with key (v4.23346).
#[tauri::command]
pub async fn mf_rdsc(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    sector: u16,
    key: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_rdsc(sector, &key);
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf mf wrbl --blk <n> -k <key> -d <data>` — write single block with key (v4.23346).
#[tauri::command]
pub async fn mf_wrbl(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    block: u16,
    key: String,
    data: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_mf_wrbl(block, &key, &data);
    connection::run_command(&app, &port, &cmd).await
}

// ---------------------------------------------------------------------------
// Parser for hf 14b info
// ---------------------------------------------------------------------------
fn parse_iso14b_output(output: &str) -> Result<crate::Iso14bInfo, AppError> {
    let cleaned = crate::pm3::output_parser::strip_ansi(output);
    let mut uid = None;
    let mut atqa = None;
    for line in cleaned.lines() {
        let line = line.trim();
        if line.starts_with("[=] UID:") {
            uid = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        } else if line.starts_with("[=] ATQA:") {
            atqa = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        }
    }
    Ok(crate::Iso14bInfo {
        uid: uid.unwrap_or_default(),
        atqa: atqa.unwrap_or_default(),
    })
}

/// Helper to extract port from wizard machine state.
fn get_port(machine: &State<'_, Mutex<WizardMachine>>) -> Result<String, AppError> {
    let guard = machine.lock().map_err(|_| AppError::InvalidTransition("mutex poisoned".into()))?;
    match &guard.current {
        WizardState::DeviceConnected { port, .. } => Ok(port.clone()),
        _ => Err(AppError::InvalidTransition("device not connected".into())),
    }
}