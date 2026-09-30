use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::state::{WizardAction, WizardMachine, WizardState};

#[tauri::command]
pub async fn detect_device(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Transition to DetectingDevice
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::StartDetection)?;
    }

    match connection::detect_device(&app).await {
        Ok((port, model, firmware)) => {
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::DeviceFound {
                port,
                model,
                firmware,
            })?;
            Ok(m.current.clone())
        }
        Err(e) => {
            let err_msg = e.to_string();
            // Match on the error variant, not on substrings of its text.
            //
            // `DeviceNotFound` renders as "PM3 not found on any port", which
            // contains the substring "not found" -- so the old substring check
            // classified *no device answered the probe* as *the client binary
            // is missing* and told the user to install proxmark3 in their PATH.
            // The client is bundled beside phosphor.exe, so that message is
            // almost always wrong. It is especially confusing on an iCopy-X,
            // where the real cause is that the device was not in PC mode.
            let user_message = match &e {
                AppError::CommandFailed(msg) if msg.contains("Failed to spawn proxmark3") => {
                    "Proxmark3 client could not be started. Reinstall Phosphor, or make \
                     sure proxmark3.exe is in the same folder as phosphor.exe."
                        .to_string()
                }
                AppError::DeviceNotFound => {
                    "No Proxmark3 responded. If your device has a PC mode (iCopy-X and \
                     similar), switch it to PC mode first, then reconnect and retry."
                        .to_string()
                }
                _ => "No Proxmark3 device found. Check your USB connection.".to_string(),
            };
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: err_msg,
                user_message,
                recoverable: true,
                recovery_action: Some(crate::cards::types::RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
    }
}
