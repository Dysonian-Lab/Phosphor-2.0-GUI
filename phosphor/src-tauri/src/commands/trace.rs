use tauri::{AppHandle, State};
use std::sync::Mutex;

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardMachine, WizardState};
use crate::pm3::command_builder;

/// Run `trace clear` — clear the client-side trace buffer.
#[tauri::command]
pub async fn trace_clear(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_trace_clear()).await
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