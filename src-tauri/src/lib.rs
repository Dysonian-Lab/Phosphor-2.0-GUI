mod cards;
mod commands;
mod db;
mod error;
mod pm3;
mod state;

use std::sync::Mutex;

use commands::firmware::FlashState;
use pm3::connection::HfOperationState;
use state::WizardMachine;
use tauri::Manager;

// ISO 14443-B Info structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct Iso14bInfo {
    pub uid: String,
    pub atqa: String,
}

// ISO 15693 Info structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct Iso15Info {
    pub uid: String,
    pub dsfid: String,
}

// Felica Info structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct FelicaInfo {
    pub idm: String,
    pub pmm: String,
}

// iCLASS SE/SEOS Info structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct IclassSeInfo {
    pub uid: String,
    pub atqa: String,
}

// LEGIC Info structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct LegicInfo {
    pub uid: String,
    pub atqa: String,
}

// Script result structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct ScriptResult {
    pub output: String,
}

// Antenna test result structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct AntennaTestResult {
    pub output: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let database =
                db::Database::open(data_dir).expect("failed to open database");
            app.manage(database);
            app.manage(Mutex::new(WizardMachine::new()));
            app.manage(FlashState::new());
            app.manage(HfOperationState::new());
            Ok(())
        })
         .invoke_handler(tauri::generate_handler![
             commands::wizard::get_wizard_state,
             commands::wizard::wizard_action,
             commands::device::detect_device,
             commands::blank::detect_blank,
             commands::scan::scan_card,
             commands::write::write_clone,
             commands::write::write_clone_with_data,
             commands::write::verify_clone,
             commands::history::get_history,
             commands::history::save_clone_record,
             commands::firmware::check_firmware_version,
             commands::firmware::flash_firmware,
             commands::firmware::cancel_flash,
             commands::erase::detect_chip,
             commands::erase::wipe_chip,
             commands::saved::save_card,
             commands::saved::get_saved_cards,
             commands::saved::delete_saved_card,
             commands::raw::run_raw_command,
             commands::hf_clone::hf_autopwn,
             commands::hf_clone::hf_write_clone,
             commands::hf_clone::hf_dump,
             commands::hf_clone::hf_verify_clone,
             commands::hf_clone::cancel_hf_operation,
             commands::iso14b::iso14b_info,
             commands::iso15::iso15_info,
             commands::felica::felica_info,
             commands::iclass_se::iclass_se_info,
             commands::legic::legic_info,
             commands::script::run_script,
             commands::script::list_scripts,
             commands::script::read_script,
             commands::script::write_script,
             commands::tuning::hw_tune,
             commands::tuning::lf_tune,
             commands::antenna::hw_measure,
         ])
        .run(tauri::generate_context!())
        .expect("error running Phosphor 2.0 GUI");
}
