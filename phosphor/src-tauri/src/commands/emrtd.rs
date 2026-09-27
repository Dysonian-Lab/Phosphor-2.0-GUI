use tauri::{AppHandle, State};
use std::sync::Mutex;

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardMachine, WizardState};
use crate::pm3::command_builder;

/// Run `hf emrtd info` — tag information (offline with --dir).
#[tauri::command]
pub async fn hf_emrtd_info(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_hf_emrtd_info()).await
}

/// Run `hf emrtd dump` — dump eMRTD files to binary files.
#[tauri::command]
pub async fn hf_emrtd_dump(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_hf_emrtd_dump()).await
}

/// Run `hf emrtd list` — list ISO 14443A/7816 history (offline, no device needed).
#[tauri::command]
pub async fn hf_emrtd_list(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_hf_emrtd_list()).await
}

/// Run `hf emrtd test` — offline regression tests (no device needed).
#[tauri::command]
pub async fn hf_emrtd_test(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_hf_emrtd_test()).await
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