use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::error::AppError;

/// List dump files available in the app directory.
///
/// PM3 writes dumps to the current working directory at runtime, which in the
/// portable build is the executable directory (e.g. `hf-mf-01020304-dump.bin`,
/// `hf-mf-01020304-dump.eml`, `hf-iclass-...-dump.bin`, `hf-mfu-...-dump.bin`).
///
/// This command scans the executable directory and its `.proxmark3/` subdirectory
/// for files matching PM3 dump naming patterns, and returns them sorted by
/// modification time (newest first) so the user can pick the right one.
#[tauri::command]
pub async fn list_dump_files(app: AppHandle) -> Result<Vec<DumpFileEntry>, AppError> {
    let dirs = dump_search_dirs(&app);
    let mut seen: Vec<DumpFileEntry> = Vec::new();

    for dir in &dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(n) => n,
                    None => continue,
                };
                if !is_dump_file(name) {
                    continue;
                }
                let meta = match fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let modified = meta.modified().ok().and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_millis() as u64)
                }).unwrap_or(0);
                let size_bytes = meta.len();
                seen.push(DumpFileEntry {
                    name: name.to_string(),
                    path: path.to_string_lossy().into_owned(),
                    size_bytes: size_bytes,
                    modified_ms: modified,
                });
            }
        }
    }

    // Newest first
    seen.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    Ok(seen)
}

/// A single dump file entry returned to the frontend.
#[derive(serde::Serialize)]
pub struct DumpFileEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub modified_ms: u64,
}

/// Build the list of directories to search for dump files.
/// Order: executable dir first (where PM3 writes dumps in portable builds),
/// then `.proxmark3/` (PM3's own config/data dir).
fn dump_search_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            dirs.push(parent.to_path_buf());
            let pm3_dir = parent.join(".proxmark3");
            dirs.push(pm3_dir);
        }
    }

    // Also check the app data dir (Tauri) as a fallback for installed builds
    if let Ok(data_dir) = app.path().app_data_dir() {
        dirs.push(data_dir.clone());
        dirs.push(data_dir.join(".proxmark3"));
    }

    // Deduplicate while preserving order
    let mut unique: Vec<PathBuf> = Vec::new();
    for d in dirs {
        if !unique.iter().any(|u| u == &d) {
            unique.push(d);
        }
    }
    unique
}

/// Decide whether a filename looks like a PM3-generated dump file.
///
/// We are intentionally STRICT here. The point of this list is to let the user
/// pick a dump to *view/decode*, so we only return files that PM3 actually wrote
/// during a scan/dump/autopwn operation. Bundled resource files (sim011.bin,
/// iclass_dump.bin, key_retail.bin, hardnested_bf_bench_data.bin, etc.) are
/// excluded even though they share extensions — they are sample/template data,
/// not user dumps, and running `hf mf view` on them produces a confusing error.
///
/// PM3 dump naming patterns we accept:
///   - `hf-mf-<uid>-dump.bin` / `.eml`          (MIFARE Classic autopwn)
///   - `hf-iclass-<uid>-dump.bin`              (iCLASS)
///   - `hf-mfu-<uid>-dump.bin` / `.eml`         (Ultralight/NTAG)
///   - `hf-14b-<uid>-dump.json` / `.bin`        (SRIX4K MyKey)
///   - `hf-felica-<uid>-dump.bin` / `.dump`     (Felica)
///   - `hf-mfdes-<uid>.json`                    (DESFire)
///   - `<hex-uid>.bin` / `.dump` / `.eml` / `.dfu`  (generic dumps, 8+ hex chars)
fn is_dump_file(name: &str) -> bool {
    let lower = name.to_lowercase();

    // Explicit PM3 dump prefixes — these are always real dumps
    if lower.starts_with("hf-mf-")
        || lower.starts_with("hf-iclass-")
        || lower.starts_with("hf-mfu-")
        || lower.starts_with("hf-14b-")
        || lower.starts_with("hf-felica-")
        || lower.starts_with("hf-mfdes-")
        || lower.starts_with("hf-14a-")
    {
        return has_dump_extension(&lower);
    }

    // Generic hex-UID dumps like 01020304.bin, 01020304.dump, 01020304.eml
    if has_dump_extension(&lower) && looks_like_hex_dump(&lower) {
        return true;
    }

    false
}

fn has_dump_extension(lower: &str) -> bool {
    lower.ends_with(".bin")
        || lower.ends_with(".dump")
        || lower.ends_with(".eml")
        || lower.ends_with(".dfu")
        || lower.ends_with(".json")
}

/// True if the filename stem is 8+ hex digits (a UID), e.g. `01020304.bin`.
/// This excludes bundled resources like `sim011.bin` (stem "sim011" is not hex)
/// and `iclass_dump.bin` (stem "iclass_dump" is not hex).
fn looks_like_hex_dump(lower: &str) -> bool {
    let stem = lower.rsplitn(2, '.').nth(1).unwrap_or(lower);
    stem.len() >= 8 && stem.chars().all(|c| c.is_ascii_hexdigit())
}