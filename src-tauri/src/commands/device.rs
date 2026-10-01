use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::logging;
use crate::pm3::connection;
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Pull `CAPABILITIES_VERSION` out of the `hw version` banner.
///
/// PM3 prints it as `Capabilities version: 11`, so the label is searched
/// case-insensitively and any number after it is taken. Returns an empty string
/// when absent, which the logger reports rather than guessing.
fn extract_capabilities(firmware: &str) -> String {
    static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)capabilities\s*version\s*[:.]+\s*(\d+)").expect("bad caps regex")
    });
    RE.captures(firmware)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "not reported".to_string())
}

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
            // Record the device identity once per connection. This is the single
            // most valuable line in the whole log: "which build, which device,
            // which firmware" is otherwise unanswerable from a bug report, and
            // the same PM3 command behaving differently between testers is almost
            // always a firmware or client-build difference.
            logging::device(&port, &model, &firmware, &extract_capabilities(&firmware));

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
            logging::section("DEVICE DETECTION");
            logging::line(&format!("FAILED: {}", e));
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
