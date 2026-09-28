use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::WizardMachine;

/// Run `hw decay` — measure HF antenna decay after field-off.
/// `stabilize_ms`: field stabilization time in ms (default 50).
/// `measure_us`: measurement window in us (default 2000).
#[tauri::command]
pub async fn hw_decay(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    stabilize_ms: Option<u16>,
    measure_us: Option<u16>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let cmd = match (stabilize_ms, measure_us) {
        (Some(ms), Some(us)) => format!("hw decay --ms {} --us {}", ms, us),
        (Some(ms), None) => format!("hw decay --ms {}", ms),
        (None, Some(us)) => format!("hw decay --us {}", us),
        (None, None) => "hw decay".to_string(),
    };
    connection::run_command(&app, &port, &cmd).await
}

/// Run `hf tune` — continuously measure HF antenna tuning.
/// `iter`: number of iterations (0 = infinite, default).
/// `style`: "bar", "mix", "value", or none (default session style).
#[tauri::command]
pub async fn hf_tune(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    iter: Option<u64>,
    style: Option<String>,
) -> Result<String, AppError> {
    let port = get_port(&machine)?;
    let mut cmd = String::from("hf tune");
    if let Some(n) = iter {
        cmd.push_str(&format!(" --iter {}", n));
    }
    if let Some(s) = &style {
        let flag = match s.to_lowercase().as_str() {
            "bar" => "--bar",
            "mix" => "--mix",
            "value" => "--value",
            _ => "",
        };
        if !flag.is_empty() {
            cmd.push(' ');
            cmd.push_str(flag);
        }
    }
    connection::run_command(&app, &port, &cmd).await
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