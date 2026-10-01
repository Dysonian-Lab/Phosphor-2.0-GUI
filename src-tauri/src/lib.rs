mod cards;
mod commands;
mod db;
mod error;
mod logging;
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
            // Open the session diagnostic log and write the environment header.
            //
            // This replaces a startup marker that wrote to a HARDCODED developer
            // path (`D:\kilocode\...\logs`). On any other machine that path was
            // not creatable, so the marker was skipped silently and testers
            // produced NO log at all -- the worst possible failure, since it is
            // indistinguishable from "nothing went wrong". See `logging`.
            if let Some(path) = logging::init() {
                logging::line(&format!("Session log opened: {}", path.display()));
                // Tell the terminal panel where the log is, so the path can be
                // copied straight out of the UI.
                logging::line(
                    "When reporting a problem, send this whole file. It records \
                     every command, its raw output, exit code and timing.",
                );
            }
            Ok(())
        })
.invoke_handler(tauri::generate_handler![
              logging::get_session_log_path,
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
              commands::dump::list_dump_files,
               commands::raw::run_raw_command,
              commands::hf_clone::hf_autopwn,
              commands::hf_clone::hf_write_clone,
              commands::hf_clone::hf_dump,
              commands::hf_clone::hf_verify_clone,
              commands::hf_clone::cancel_hf_operation,
              commands::iso14b::iso14b_info,
              commands::iso14b::iso14b_rdbl,
              commands::iso14b::iso14b_ctrdbl,
              commands::iso14b::iso14b_view_selftest,
              commands::iso14b::iso14b_view,
              commands::iso14b::hf_14a_antifuzz,
              commands::iso14b::hf_14a_antifuzz_coll,
commands::iso14b::hf_mf_autopwn,
                commands::iso14b::hf_mf_view,
                commands::iso14b::hf_mf_view_selftest,
                commands::iso14b::hf_mf_dump,
                commands::iso14b::hf_mf_cview,
                commands::iso14b::mf_rdbl,
                commands::iso14b::mf_rdsc,
                commands::iso14b::mf_wrbl,
                commands::iso14b::mf_nested,
                commands::iso14b::mf_esave,
                commands::iso14b::mf_sim,
                commands::iso14b::mf_eclr,
                commands::iso14b::mf_eload,
                commands::iso14b::mf_egetblk,
                commands::iso14b::mf_esetblk,
commands::iso14b::lf_t55xx_set_config,
               commands::iso14b::lf_t55xx_chk_pwds,
               commands::iso14b::lf_t55xx_dangerraw,
               commands::iso14b::lf_t55xx_wakeup,
               commands::iso15::iso15_info,
              commands::felica::felica_info,
              commands::iclass_se::iclass_se_info,
              commands::legic::legic_info,
              commands::emrtd::hf_emrtd_info,
              commands::emrtd::hf_emrtd_dump,
              commands::emrtd::hf_emrtd_list,
              commands::emrtd::hf_emrtd_test,
              commands::smartcard::smart_pps,
              commands::smartcard::smart_pps_t0,
              commands::smartcard::smart_pps_t1,
              commands::smartcard::smart_pps_ta1,
              commands::trace::trace_clear,
               commands::calypso::calypso_info,
               commands::calypso::calypso_dump,
                commands::calypso::calypso_list,
                commands::desfire::desfire_auth,
                commands::desfire::desfire_bare,
                commands::desfire::desfire_detect,
                commands::desfire::desfire_chk,
                commands::desfire::desfire_dump,
                commands::desfire::desfire_read,
                commands::desfire::desfire_write,
                commands::desfire::desfire_value,
                commands::desfire::desfire_file,
                commands::desfire::desfire_etest,
                commands::desfire::desfire_list,
                commands::desfire::desfire_pc,
                commands::desfire::desfire_createapp,
                commands::desfire::desfire_createfile,
                commands::desfire::desfire_changekey,
                commands::desfire::desfire_bruteaid,
                commands::desfire::desfire_bruteisofid,
                commands::desfire::desfire_cert,
                commands::desfire::desfire_verifycert,
                commands::desfire::desfire_makelicense,
                commands::thinfilm::thinfilm_sniff,
               commands::mad::mad_read,
               commands::mad::mad_write,
               commands::mad::mad_verify,
               commands::mad::mad_decode,
               commands::mad::mad_encode,
               commands::nfc::nfc_encode,
               commands::mfu::mfu_chk,
               commands::mfu::mfu_ndefwrite,
               commands::mfu::mfu_ndefformat,
               commands::mf::mf_chk,
               commands::felica_sim::felica_sim,
               commands::iclass_legbrute::iclass_legbrute,
               commands::lf_trovan::lf_trovan,
              commands::script::run_script,
              commands::script::list_scripts,
              commands::script::read_script,
              commands::script::write_script,
              commands::tuning::hw_tune,
              commands::tuning::lf_tune,
              commands::antenna::hw_decay,
                commands::antenna::hf_tune,
          ])
        .run(tauri::generate_context!())
        .expect("error running Phosphor 2.2 GUI");
}
