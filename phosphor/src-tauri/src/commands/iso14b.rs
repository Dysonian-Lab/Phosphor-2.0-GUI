use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardAction, WizardMachine, WizardState};

// Use the Iso14bInfo defined in lib.rs
use crate::Iso14bInfo;

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

/// Run `lf t55xx set config` — configure T55xx tag parameters.
#[tauri::command]
pub async fn lf_t55xx_set_config(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx set config").await
}

/// Run `lf t55xx chk pwds` — check T55xx passwords.
#[tauri::command]
pub async fn lf_t55xx_chk_pwds(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx chk pwds").await
}

/// Run `lf t55xx dangerraw` — raw T55xx danger mode.
#[tauri::command]
pub async fn lf_t55xx_dangerraw(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, "lf t55xx dangerraw").await
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

fn get_port(machine: &State<'_, Mutex<WizardMachine>>) -> Result<String, AppError> {
    let m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    match &m.current {
        WizardState::DeviceConnected { port, .. } => Ok(port.clone()),
        _ => Err(AppError::InvalidTransition(
            "No device connected".to_string(),
        )),
    }
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

/// Run `hf 14b info` and return a tidy structure.
#[tauri::command]
pub async fn iso14b_info(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<Iso14bInfo, AppError> {
    // Grab the currently‑connected port from the wizard state
    let port = {
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        let port = match &m.current {
            WizardState::DeviceConnected { port, .. } => port.clone(),
            _ => {
                return Err(AppError::InvalidTransition(
                    "No device connected".to_string(),
                ));
            }
        };
        port
    };

    // Execute the PM3 command.
    let raw = connection::run_command(&app, &port, "hf 14b info").await?;

    // Parse the output – you can reuse helpers from `output_parser.rs` or
    // write a small bespoke parser here.
    let info = parse_iso14b_output(&raw)?;

    Ok(info)
}

// ---------------------------------------------------------------------------
// Helper: simple parser for hf 14b info (adjust to match actual PM3 output)
// ---------------------------------------------------------------------------
fn parse_iso14b_output(output: &str) -> Result<Iso14bInfo, AppError> {
    let cleaned = crate::pm3::output_parser::strip_ansi(output);
    let mut atqa = None;
    let mut uid = None;
    // Example lines from PM3:
    //   [=] UID: 04 12 34 56 78
    //   [=] ATQA: 00 02
    for line in cleaned.lines() {
        let line = line.trim();
        if line.starts_with("[=] UID:") {
            uid = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        } else if line.starts_with("[=] ATQA:") {
            atqa = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        }
    }
    Ok(Iso14bInfo {
        uid: uid.unwrap_or_default(),
        atqa: atqa.unwrap_or_default(),
        // add more fields as you discover them (SAK, etc.)
    })
}