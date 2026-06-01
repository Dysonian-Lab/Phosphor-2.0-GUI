use std::{
    env,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;

// We'll return a String as the script output
/// Run a Lua script on the Proxmark3 and return the output.
#[tauri::command]
pub async fn run_script(
    app: AppHandle,
    machine: State<'_, Mutex<crate::state::WizardMachine>>,
    script: String,
    args: Option<String>,
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

    // Write the script to a temp file in the executable directory, then run it by filename.
    let script_dir = get_exe_dir(&app)?;
    if !script_dir.exists() {
        fs::create_dir_all(&script_dir)
            .map_err(|e| AppError::CommandFailed(format!("Failed to create executable directory: {}", e)))?;
    }
    let temp_name = format!("phosphor_inline_{}.lua", std::process::id());
    let temp_path = script_dir.join(&temp_name);
    fs::write(&temp_path, script)
        .map_err(|e| AppError::CommandFailed(format!("Failed to write temp script: {}", e)))?;

    let cmd = match args {
        Some(a) if !a.trim().is_empty() => format!("script run {} {}", temp_name, a.trim()),
        _ => format!("script run {}", temp_name),
    };

    let raw = connection::run_command(&app, &port, &cmd).await?;

    let _ = fs::remove_file(&temp_path);
    Ok(raw)
}

/// List all Lua script files in the scripts directory.
#[tauri::command]
pub async fn list_scripts(
    app: AppHandle,
) -> Result<Vec<String>, AppError> {
    let script_dir = get_script_dir(&app)?;

    // Read the directory
    let entries = fs::read_dir(&script_dir)
        .map_err(|e| AppError::CommandFailed(format!("Failed to read script directory: {}", e)))?;

    let mut scripts = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|e| AppError::CommandFailed(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();
        if path.is_file() {
            // We'll only show the filename (without extension? but we keep extension for now)
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                scripts.push(file_name.to_string());
            }
        }
    }

    Ok(scripts)
}

/// Read the content of a script file.
#[tauri::command]
pub async fn read_script(
    app: AppHandle,
    filename: String,
) -> Result<String, AppError> {
    let script_dir = get_script_dir(&app)?;
    let file_path = script_dir.join(&filename);

    // Read the file
    let content = fs::read_to_string(&file_path)
        .map_err(|e| AppError::CommandFailed(format!("Failed to read script file '{}': {}", filename, e)))?;

    Ok(content)
}

/// Write content to a script file (create or overwrite).
#[tauri::command]
pub async fn write_script(
    app: AppHandle,
    filename: String,
    content: String,
) -> Result<(), AppError> {
    let script_dir = get_script_dir(&app)?;
    let file_path = script_dir.join(&filename);

    // Ensure the script directory exists
    if !script_dir.exists() {
        fs::create_dir_all(&script_dir)
            .map_err(|e| AppError::CommandFailed(format!("Failed to create script directory: {}", e)))?;
    }

    // Write the file
    fs::write(&file_path, content)
        .map_err(|e| AppError::CommandFailed(format!("Failed to write script file '{}': {}", filename, e)))?;

    Ok(())
}

/// Helper function to get the script directory path (.proxmark3/scripts).
fn get_script_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let exe_path = env::current_exe()
        .map_err(|e| AppError::CommandFailed(format!("Failed to get current executable path: {}", e)))?;
    let mut script_dir = exe_path.parent()
        .ok_or_else(|| AppError::CommandFailed("Failed to get parent directory of executable".to_string()))?
        .to_path_buf();
    script_dir.push(".proxmark3");
    script_dir.push("scripts");
    Ok(script_dir)
}

/// Helper function to get the executable directory (for temp script runs).
fn get_exe_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let exe_path = env::current_exe()
        .map_err(|e| AppError::CommandFailed(format!("Failed to get current executable path: {}", e)))?;
    Ok(exe_path.parent()
        .ok_or_else(|| AppError::CommandFailed("Failed to get parent directory of executable".to_string()))?
        .to_path_buf())
}
