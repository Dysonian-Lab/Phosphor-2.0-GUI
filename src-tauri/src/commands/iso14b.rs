use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardAction, WizardMachine, WizardState};

// Use the Iso14bInfo defined in lib.rs
use crate::Iso14bInfo;

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