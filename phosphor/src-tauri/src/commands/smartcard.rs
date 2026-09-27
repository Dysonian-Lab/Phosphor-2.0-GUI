use tauri::{AppHandle, State};
use std::sync::Mutex;

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardMachine, WizardState};
use crate::pm3::command_builder;

/// Run `smart pps` — ISO 7816-3 PPS exchange (RDV4 smartcard module required).
#[tauri::command]
pub async fn smart_pps(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_smart_pps()).await
}

/// Run `smart pps --t0` — select T=0 protocol.
#[tauri::command]
pub async fn smart_pps_t0(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_smart_pps_t0()).await
}

/// Run `smart pps --t1` — select T=1 protocol.
#[tauri::command]
pub async fn smart_pps_t1(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, command_builder::build_smart_pps_t1()).await
}

/// Run `smart pps --ta1 <hex>` — negotiate TA1 byte.
#[tauri::command]
pub async fn smart_pps_ta1(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    ta1: String,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = command_builder::build_smart_pps_ta1(&ta1);
    connection::run_command(&app, &port, &cmd).await
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