use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::command_builder;
use crate::pm3::connection;
use crate::state::{WizardMachine, WizardState};

/// Run `mad read` — read data from MAD AID sectors (v4.23346).
#[tauri::command]
pub async fn mad_read(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_mad_read()).await
}

/// Run `mad write -d <data>` — write data to MAD AID sectors (v4.23346).
#[tauri::command]
pub async fn mad_write(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    data: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &command_builder::build_mad_write(&data)).await
}

/// Run `mad verify` — verify data in MAD AID sectors (v4.23346).
#[tauri::command]
pub async fn mad_verify(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_mad_verify()).await
}

/// Run `mad decode` — decode MAD byte array (v4.23346).
#[tauri::command]
pub async fn mad_decode(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_mad_decode()).await
}

/// Run `mad encode -d <data>` — encode MAD byte array from AID mappings (v4.23346).
#[tauri::command]
pub async fn mad_encode(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    data: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &command_builder::build_mad_encode(&data)).await
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
