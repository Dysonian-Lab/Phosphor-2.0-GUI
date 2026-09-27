use std::{
    env,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

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

/// List all Lua script files: bundled PM3 scripts merged with user scripts.
/// Bundled scripts live in the resource dir at
/// `share/proxmark3/luascripts/` (and `lualibs/`, `cmdscripts/`, `pyscripts/`).
/// User scripts live in `.proxmark3/scripts/`. Bundled scripts are returned
/// first and prefixed so users can tell them apart from their own.
#[tauri::command]
pub async fn list_scripts(
    app: AppHandle,
) -> Result<Vec<String>, AppError> {
    let mut scripts = Vec::new();

    // 1. Bundled scripts from the resource dir
    if let Ok(resource_dir) = app.path().resource_dir() {
        for sub in &["luascripts", "lualibs", "cmdscripts", "pyscripts"] {
            let dir = resource_dir.join("share").join("proxmark3").join(sub);
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                                // Prefix bundled scripts so users can distinguish
                                scripts.push(format!("bundled/{}/{}", sub, name));
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. User scripts from .proxmark3/scripts
    let user_dir = get_script_dir(&app)?;
    if user_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        scripts.push(format!("user/{}", name));
                    }
                }
            }
        }
    }

    Ok(scripts)
}

/// Read the content of a script file.
/// `filename` may be `bundled/<sub>/<name>` or `user/<name>`.
#[tauri::command]
pub async fn read_script(
    app: AppHandle,
    filename: String,
) -> Result<String, AppError> {
    let path = resolve_script_path(&app, &filename)?;
    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::CommandFailed(format!("Failed to read script file '{}': {}", filename, e)))?;
    Ok(content)
}

/// Write content to a script file (create or overwrite). User scripts only —
/// bundled scripts are read-only.
#[tauri::command]
pub async fn write_script(
    app: AppHandle,
    filename: String,
    content: String,
) -> Result<(), AppError> {
    let file_path = resolve_user_script_path(&app, &filename)?;
    // Ensure the script directory exists
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::CommandFailed(format!("Failed to create script directory: {}", e)))?;
        }
    }
    fs::write(&file_path, content)
        .map_err(|e| AppError::CommandFailed(format!("Failed to write script file '{}': {}", filename, e)))?;
    Ok(())
}

/// Resolve a script path from its list entry name.
/// `bundled/<sub>/<name>` -> resource_dir/share/proxmark3/<sub>/<name>
/// `user/<name>`            -> .proxmark3/scripts/<name>
fn resolve_script_path(app: &AppHandle, name: &str) -> Result<PathBuf, AppError> {
    if let Some(rest) = name.strip_prefix("bundled/") {
        let mut parts = rest.splitn(2, '/');
        let sub = parts.next().ok_or_else(|| AppError::CommandFailed("bad bundled path".into()))?;
        let file = parts.next().ok_or_else(|| AppError::CommandFailed("bad bundled path".into()))?;
        let resource_dir = app.path().resource_dir().map_err(|e| {
            AppError::CommandFailed(format!("Failed to resolve resource dir: {}", e))
        })?;
        Ok(resource_dir.join("share").join("proxmark3").join(sub).join(file))
    } else if let Some(rest) = name.strip_prefix("user/") {
        Ok(get_script_dir(app)?.join(rest))
    } else {
        // Legacy bare filename — treat as user script
        Ok(get_script_dir(app)?.join(name))
    }
}

fn resolve_user_script_path(app: &AppHandle, name: &str) -> Result<PathBuf, AppError> {
    let rest = name.strip_prefix("user/").unwrap_or(name);
    Ok(get_script_dir(app)?.join(rest))
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
