use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;

// Use the IclassSeInfo defined in lib.rs
use crate::IclassSeInfo;

/// Run `hf iclass info` and return a tidy structure.
/// The command will timeout via the underlying PM3_COMMAND_TIMEOUT if no card is present.
#[tauri::command]
pub async fn iclass_se_info(
    app: AppHandle,
    machine: State<'_, Mutex<crate::state::WizardMachine>>,
) -> Result<IclassSeInfo, AppError> {
    // Grab the currently‑connected port from the wizard state
    let port = {
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        let port = match &m.current {
            crate::state::WizardState::DeviceConnected { port, .. } => port.clone(),
            _ => {
                return Err(AppError::InvalidTransition(
                    "No device connected".to_string(),
                ));
            }
        };
        port
    };

    // Execute the PM3 command - relies on built-in timeout to prevent hanging
    let raw = connection::run_command(&app, &port, "hf iclass info").await?;

    // Parse the output – adjust to match actual PM3 output.
    let info = parse_iclass_se_output(&raw)?;

    Ok(info)
}

// ---------------------------------------------------------------------------
// Helper: simple parser for hf iclass info (adjust to match actual PM3 output)
// ---------------------------------------------------------------------------
fn parse_iclass_se_output(output: &str) -> Result<IclassSeInfo, AppError> {
    let cleaned = crate::pm3::output_parser::strip_ansi(output);
    let mut uid = None;
    let mut atqa = None;
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
    Ok(IclassSeInfo {
        uid: uid.unwrap_or_default(),
        atqa: atqa.unwrap_or_default(),
        // add more fields as you discover them (e.g., SAK, etc.)
    })
}