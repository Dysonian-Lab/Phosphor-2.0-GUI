use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;

// Use the FelicaInfo defined in lib.rs
use crate::FelicaInfo;

/// Run `hf felica info` and return a tidy structure.
#[tauri::command]
pub async fn felica_info(
    app: AppHandle,
    machine: State<'_, Mutex<crate::state::WizardMachine>>,
) -> Result<FelicaInfo, AppError> {
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
    let raw = connection::run_command(&app, &port, "hf felica info").await?;

    // Parse the output – adjust to match actual PM3 output.
    let info = parse_felica_output(&raw)?;

    Ok(info)
}

// ---------------------------------------------------------------------------
// Helper: simple parser for hf felica info (adjust to match actual PM3 output)
// ---------------------------------------------------------------------------
fn parse_felica_output(output: &str) -> Result<FelicaInfo, AppError> {
    let cleaned = crate::pm3::output_parser::strip_ansi(output);
    let mut idm = None;
    let mut pmm = None;
    // Example lines from PM3 (need to verify actual output):
    //   [=] IDm: 01 23 45 67 89 AB CD EF
    //   [=] PMm: 01 23 45 67 89 AB CD EF
    for line in cleaned.lines() {
        let line = line.trim();
        if line.starts_with("[=] IDm:") {
            idm = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        } else if line.starts_with("[=] PMm:") {
            pmm = Some(line.split_whitespace().nth(1).unwrap_or("").to_uppercase());
        }
    }
    Ok(FelicaInfo {
        idm: idm.unwrap_or_default(),
        pmm: pmm.unwrap_or_default(),
        // add more fields as you discover them
    })
}