use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;

// Use the Iso15Info defined in lib.rs
use crate::Iso15Info;

/// Run `hf 15 info` and return a tidy structure.
#[tauri::command]
pub async fn iso15_info(
    app: AppHandle,
    machine: State<'_, Mutex<crate::state::WizardMachine>>,
) -> Result<Iso15Info, AppError> {
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

    // Execute the PM3 command.
    let raw = connection::run_command(&app, &port, "hf 15 info").await?;

    // Parse the output – adjust to match actual PM3 output.
    let info = parse_iso15_output(&raw)?;

    Ok(info)
}

// ---------------------------------------------------------------------------
// Helper: simple parser for hf 15 info (adjust to match actual PM3 output)
// ---------------------------------------------------------------------------
fn parse_iso15_output(output: &str) -> Result<Iso15Info, AppError> {
    let cleaned = crate::pm3::output_parser::strip_ansi(output);
    let mut uid = None;
    let mut dsfid = None;
    // Example lines from PM3 (need to verify actual output):
    //   [=] UID: 01 02 03 04 05 06 07 08
    //   [=] DSFID: 01
    for line in cleaned.lines() {
        let line = line.trim();
        if line.starts_with("[=] UID:") {
            uid = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        } else if line.starts_with("[=] DSFID:") {
            dsfid = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        }
    }
    Ok(Iso15Info {
        uid: uid.unwrap_or_default(),
        dsfid: dsfid.unwrap_or_default(),
        // add more fields as you discover them (AFI, etc.)
    })
}