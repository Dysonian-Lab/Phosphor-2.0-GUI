use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::WizardMachine;

// We'll return a String as the output
/// Run `hw tune` and return the output.
#[tauri::command]
pub async fn hw_tune(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    // Use the shared get_port helper so the Advanced tab works after a scan
    // without forcing the user to click Back first.
    let port = get_port(&machine)?;

    // Execute the PM3 command.
    let raw = connection::run_command(&app, &port, "hw tune").await?;

    Ok(raw)
}

/// Run `lf tune` and return the output.
#[tauri::command]
pub async fn lf_tune(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    // Use the shared get_port helper so the Advanced tab works after a scan
    // without forcing the user to click Back first.
    let port = get_port(&machine)?;

    // Execute the PM3 command - lf tune takes no arguments
    let raw = connection::run_command(&app, &port, "lf tune").await?;

    Ok(raw)
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