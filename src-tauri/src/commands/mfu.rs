use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::command_builder;
use crate::pm3::connection;
use crate::state::{WizardMachine, WizardState};

/// Run `hf mfu chk` — Ultralight C/AES authentication dictionary check (v4.23346).
#[tauri::command]
pub async fn mfu_chk(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_mfu_chk()).await
}

/// Run `hf mfu ndefwrite -d <data>` — write NDEF records to card (v4.23346).
#[tauri::command]
pub async fn mfu_ndefwrite(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    data: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &command_builder::build_mfu_ndefwrite(&data)).await
}

/// Run `hf mfu ndefformat` — format tag as NDEF, writes the Capability Container (v4.23346).
#[tauri::command]
pub async fn mfu_ndefformat(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_mfuf_ndefformat()).await
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
