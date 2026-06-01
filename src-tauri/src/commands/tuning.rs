use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;

// We'll return a String as the output
/// Run `hw tune` and return the output.
#[tauri::command]
pub async fn hw_tune(
    app: AppHandle,
    machine: State<'_, Mutex<crate::state::WizardMachine>>,
) -> Result<String, AppError> {
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
    let raw = connection::run_command(&app, &port, "hw tune").await?;

    Ok(raw)
}

/// Run `lf tune` and return the output.
#[tauri::command]
pub async fn lf_tune(
    app: AppHandle,
    machine: State<'_, Mutex<crate::state::WizardMachine>>,
) -> Result<String, AppError> {
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

    // Execute the PM3 command - lf tune takes no arguments
    let raw = connection::run_command(&app, &port, "lf tune").await?;

    Ok(raw)
}