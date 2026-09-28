use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::WizardMachine;

// Use the LegicInfo defined in lib.rs
use crate::LegicInfo;

/// Run `hf legic info` and return a tidy structure.
#[tauri::command]
pub async fn legic_info(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<LegicInfo, AppError> {
    // Use the shared get_port helper so the Advanced tab works after a scan
    // without forcing the user to click Back first.
    let port = get_port(&machine)?;

    // Execute the PM3 command.
    let raw = connection::run_command_quick(&app, &port, "hf legic info").await?;

    // Parse the output – adjust to match actual PM3 output.
    let info = parse_legic_output(&raw)?;

    Ok(info)
}

// ---------------------------------------------------------------------------
// Helper: simple parser for hf legic info (adjust to match actual PM3 output)
// ---------------------------------------------------------------------------
fn parse_legic_output(output: &str) -> Result<LegicInfo, AppError> {
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
    Ok(LegicInfo {
        uid: uid.unwrap_or_default(),
        atqa: atqa.unwrap_or_default(),
        // add more fields as you discover them
    })
}

fn get_port(machine: &State<'_, Mutex<WizardMachine>>) -> Result<String, AppError> {
    let m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;

    if let crate::state::WizardState::DeviceConnected { port, .. } = &m.current {
        return Ok(port.clone());
    }

    if let Some(port) = &m.port {
        return Ok(port.clone());
    }

    Err(AppError::InvalidTransition("No device connected".to_string()))
}