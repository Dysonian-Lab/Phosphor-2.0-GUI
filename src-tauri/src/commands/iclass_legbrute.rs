use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::command_builder;
use crate::pm3::connection;
use crate::state::{WizardMachine, WizardState};

/// Run `hf iclass legbrute` — bruteforce 40 bits of a partial diversified key (v4.23346).
#[tauri::command]
pub async fn iclass_legbrute(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_hf_iclass_legbrute()).await
}

fn get_port(machine: &State<'_, Mutex<WizardMachine>>) -> Result<String, AppError> {
    let m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;

    if let WizardState::DeviceConnected { port, .. } = &m.current {
        return Ok(port.clone());
    }

    if let Some(port) = &m.port {
        return Ok(port.clone());
    }

    Err(AppError::InvalidTransition("No device connected".to_string()))
}
