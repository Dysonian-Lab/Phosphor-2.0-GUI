use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::cards::types::{BlankType, CardType, RecoveryAction};
use crate::error::AppError;
use crate::logging;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

/// Total progress steps for the T5577 write flow:
/// detect -> check password -> wipe -> verify wipe -> clone -> done
const T5577_TOTAL_STEPS: u16 = 6;

/// Total progress steps for the EM4305 write flow:
/// detect -> wipe -> verify wipe -> clone -> done
const EM4305_TOTAL_STEPS: u16 = 5;

/// Stub that returns an error directing callers to write_clone_with_data.
/// Kept registered so the frontend gets a clear message if it calls without params.
#[tauri::command]
pub async fn write_clone(
    _app: AppHandle,
    _machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    Err(AppError::CommandFailed(
        "write_clone is deprecated: use write_clone_with_data instead".into(),
    ))
}

/// Write clone with explicit parameters from the frontend.
/// This is the preferred entry point. Handles T5577 password safety and EM4305 blanks.
#[tauri::command]
pub async fn write_clone_with_data(
    app: AppHandle,
    port: String,
    card_type: CardType,
    uid: String,
    decoded: std::collections::HashMap<String, String>,
    blank_type: Option<BlankType>,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    log::debug!("write_clone_with_data: port={}, card_type={:?}, uid={}, blank_type={:?}", port, card_type, uid, blank_type);

    // Guard: reject absurdly large decoded maps (prevents DoS via oversized IPC payload)
    if decoded.len() > 50 {
        return Err(AppError::CommandFailed(
            "Too many decoded fields".into(),
        ));
    }

    // Validate uid: must be non-empty alphanumeric with optional colons.
    // HID UIDs use format "FC65:CN29334" (contains non-hex letters like N).
    // Blocks semicolons, spaces, newlines — prevents command injection.
    if uid.is_empty() || !uid.chars().all(|c| c.is_ascii_alphanumeric() || c == ':') {
        return Err(AppError::CommandFailed(
            "Invalid UID: must contain only alphanumeric characters and colons".into(),
        ));
    }
    if uid.len() > 200 {
        return Err(AppError::CommandFailed("UID too long".into()));
    }

    // Validate port format
    if port.is_empty()
        || port.len() > 50
        || port.contains(';')
        || port.contains('\n')
        || port.contains('\r')
    {
        return Err(AppError::CommandFailed("Invalid port".into()));
    }

    let blank = blank_type.unwrap_or_else(|| card_type.recommended_blank());

    // Record exactly what was asked to be written, where, and onto what. A
    // report of "the write failed" is useless without the source type, target
    // blank and UID that were in play.
    logging::operation(
        "WRITE CLONE",
        &format!(
            "port={} card_type={:?} uid={} target_blank={:?} decoded_fields={}",
            port,
            card_type,
            uid,
            blank,
            decoded.len()
        ),
        "starting",
    );

    // Guard: reject EM4305 blank for card types that don't support the --em flag.
    // Only the original 11 LF types support EM4305. The newer types (Presco, Nedap,
    // GProxII, Gallagher, PAC, Noralsy, Jablotron, SecuraKey, Visa2000, Motorola,
    // IDTECK) will fail silently or error when --em is passed.
    if blank == BlankType::EM4305 && !card_type.supports_em4305() {
        return Err(AppError::CommandFailed(format!(
            "{} does not support EM4305 blanks. Please use a T5577 blank instead.",
            card_type.display_name()
        )));
    }

    // Transition: BlankDetected -> Writing
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::StartWrite)?;
    }

    // Branch based on blank type.
    // Errors from the write flow are caught and reported as FSM Error state
    // to keep the backend FSM in sync with the frontend XState machine.
    match blank {
        BlankType::T5577 => {
            match write_t5577_flow(&app, &port, &card_type, &uid, &decoded, &machine).await {
                Ok(state) => Ok(state),
                Err(e) => {
                    let err_detail = e.to_string();
                    log::warn!("T5577 flow error: {}", err_detail);
                    // Show the actual PM3 error to the user for debugging
                    let user_msg = format!(
                        "Write failed: {}",
                        err_detail.lines().last().unwrap_or("unknown error")
                    );
                    let _ = report_error(
                        &machine,
                        &err_detail,
                        &user_msg,
                        true,
                        Some(RecoveryAction::Retry),
                    );
                    let m = machine.lock().map_err(|e| {
                        AppError::CommandFailed(format!("State lock poisoned: {}", e))
                    })?;
                    Ok(m.current.clone())
                }
            }
        }
        BlankType::EM4305 => {
            match write_em4305_flow(&app, &port, &card_type, &uid, &decoded, &machine).await {
                Ok(state) => Ok(state),
                Err(e) => {
                    let err_detail = e.to_string();
                    let user_msg = format!(
                        "Write failed: {}",
                        err_detail.lines().last().unwrap_or("unknown error")
                    );
                    let _ = report_error(
                        &machine,
                        &err_detail,
                        &user_msg,
                        true,
                        Some(RecoveryAction::Retry),
                    );
                    let m = machine.lock().map_err(|e| {
                        AppError::CommandFailed(format!("State lock poisoned: {}", e))
                    })?;
                    Ok(m.current.clone())
                }
            }
        }
        _ => {
            // Other blank types not yet supported for LF
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: format!("Unsupported blank type {:?} for LF cloning", blank),
                user_message: "This blank type is not supported for LF card cloning.".to_string(),
                recoverable: false,
                recovery_action: None,
            })?;
            Ok(m.current.clone())
        }
    }
}

/// T5577 write flow:
/// - No password: detect -> clone (clone overwrites config + data blocks directly)
/// - Password: detect -> find password -> wipe -> verify wipe -> clone
async fn write_t5577_flow(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Step 1: Detect T5577
    log::debug!("T5577 flow: Step 1 detect");
    update_progress(app, machine, 0.1, Some(0), Some(T5577_TOTAL_STEPS))?;

    let detect_out =
        connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
    let t5577_status = output_parser::parse_t5577_detect(&detect_out);
    log::debug!("T5577 detect: detected={}, pw={}", t5577_status.detected, t5577_status.password_set);

    if !t5577_status.detected {
        return report_error(
            machine,
            "T5577 not detected on writer",
            "No T5577 blank found. Place blank card on the reader.",
            true,
            Some(RecoveryAction::Retry),
        );
    }

    // Step 2: Check for password protection
    update_progress(app, machine, 0.2, Some(1), Some(T5577_TOTAL_STEPS))?;

    let password: Option<String> = if t5577_status.password_set {
        // Password detected -- run chk to find it
        let chk_out = connection::run_command(app, port, command_builder::build_t5577_chk()).await;
        match chk_out {
            Ok(output) => {
                let found = output_parser::parse_t5577_chk(&output);
                if found.is_none() {
                    // Password set but could not be recovered
                    return report_error(
                        machine,
                        "Card is password-locked, cannot recover password",
                        "This T5577 is password-protected and the password could not be found. \
                         Use a different blank card.",
                        true,
                        Some(RecoveryAction::Retry),
                    );
                }
                found
            }
            Err(_) => {
                return report_error(
                    machine,
                    "Password check failed",
                    "Could not check T5577 password. Try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
    } else {
        None
    };

    // Step 3-4: Wipe + verify (ONLY when password-protected).
    // For clean T5577s the clone command overwrites config + data blocks directly.
    // Skipping wipe avoids an extra write cycle that can fail on weaker LF antennas
    // (PM3 Easy) and eliminates two subprocess spawns (fewer serial port open/close).
    if password.is_some() {
        update_progress(app, machine, 0.35, Some(2), Some(T5577_TOTAL_STEPS))?;

        let wipe_cmd =
            command_builder::build_wipe_command(&BlankType::T5577, password.as_deref())
                .ok_or_else(|| {
                    AppError::CommandFailed("No wipe command for this blank type".into())
                })?;
        connection::run_command(app, port, &wipe_cmd).await?;

        // Verify wipe — ensure T5577 is detected and no longer password-protected.
        // PM3 can return exit code 0 even when a password-protected wipe fails silently.
        update_progress(app, machine, 0.5, Some(3), Some(T5577_TOTAL_STEPS))?;

        let verify_wipe_out =
            connection::run_command(app, port, command_builder::build_t5577_detect()).await?;
        let verify_status = output_parser::parse_t5577_detect(&verify_wipe_out);

        if !verify_status.detected || verify_status.password_set {
            return report_error(
                machine,
                "T5577 wipe verification failed — card may still be password-protected",
                "Wipe verification failed. The card may still be password-protected. \
                 Do not remove the card — try again or use a different blank.",
                true,
                Some(RecoveryAction::Retry),
            );
        }
    }

    // SAFETY: If password was set, it was cleared by wipe above.
    // Shadow the variable to prevent accidental re-lock on clone command.
    let password: Option<String> = None;

    // Step 5: Clone
    update_progress(app, machine, 0.7, Some(4), Some(T5577_TOTAL_STEPS))?;

    log::debug!("Clone: uid={}, type={:?}, decoded={:?}", uid, card_type, decoded);

    let base_clone_cmd = command_builder::build_clone_command(card_type, uid, decoded);

    log::debug!("clone_cmd={:?}", base_clone_cmd);

    match base_clone_cmd {
        Some(cmd) => {
            let final_cmd = match &password {
                Some(pw) => command_builder::build_clone_with_password(&cmd, pw)
                    .map_err(|e| AppError::CommandFailed(format!("Password validation failed: {}", e)))?,
                None => cmd,
            };
            log::debug!("sending={}", final_cmd);

            // Cloning ONTO a T5577 blank is the one operation that must run in
            // the same client process as `lf t55xx detect`, exactly like the
            // block reads. `lf t55xx detect` is what loads the chip's modulation
            // config into the running client; Phosphor spawns a fresh
            // proxmark3.exe per command, so that state is gone otherwise and the
            // clone has nothing valid to write against.
            //
            // iCopy-X does the same thing, in this order, in lfwrite.write():
            //   check_detect()  ->  wipe p 20206666; lf t55xx detect
            //   PAR_CLONE_MAP   ->  lf em 410x clone --id <hex>
            // so detect is established first, then the single clone command.
            //
            // Only the `lf t55xx write` family needs the chaining wrapper; the
            // per-type clone commands take no arguments from the detect state and
            // are sent unchanged.
            let clone_output = if final_cmd.starts_with("lf t55xx ")
                && !final_cmd.starts_with("lf t55xx chk")
                && !final_cmd.starts_with("lf t55xx config")
            {
                connection::run_t55xx_memory_command(app, port, &final_cmd).await
            } else {
                connection::run_command(app, port, &final_cmd).await
            };
            log::debug!("clone_result={:?}", clone_output.as_ref().map(|s| s.chars().take(500).collect::<String>()).map_err(|e| e.to_string()));
            let clone_output = clone_output?;
            // Check for failure indicators in PM3 output
            if clone_output.contains("[!!]")
                || clone_output.to_lowercase().contains("fail")
            {
                return report_error(
                    machine,
                    &format!("Clone command may have failed: {}", clone_output.chars().take(200).collect::<String>()),
                    "Write may have failed. Do not remove the card — try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
        // T55xx config blocks are cloned block-by-block from the source card.
        // There is no single-shot clone command for this chipset.
        //
        // Routing is on the PRESENCE OF BLOCKS, not on the identified card type.
        // A T5577 is a universal tag: block 0 holds the config that decides what
        // the chip emulates, so copying the blocks onto a blank reproduces the
        // tag regardless of what the source card was currently identified as.
        // Plenty of source cards report as EM410x while ALSO carrying a T55xx
        // config block (very common on test cards), and gating on
        // `card_type == CardType::T55xx` sent those down the "cannot be cloned"
        // path even though their blocks had been read successfully.
        //
        // `--w` / `--d` writes per block are the verified-correct route; `wipe`
        // is deliberately not used, since it often fails and is unnecessary when
        // the blocks are simply overwritten.
        None if has_t55xx_blocks(decoded) => {
            // SAFETY: `t55xx_chipset` was written by parse_lf_search and is
            // always "T55XX"-shaped; blocks are only present with it.
            log::debug!(
                "T55xx block clone path for {:?} (blocks present)",
                card_type
            );
            if let Err(e) = write_t55xx_blocks(app, port, decoded, machine).await {
                return report_error(
                    machine,
                    &e,
                    "Could not write the T55xx blocks. Do not remove the card — try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
        None => {
            return report_error(
                machine,
                &format!("No clone command for {:?}", card_type),
                "This card type cannot be cloned with the current method.",
                false,
                None,
            );
        }
    }

    // Step 6: Done writing -> Verifying transition
    update_progress(app, machine, 1.0, Some(5), Some(T5577_TOTAL_STEPS))?;
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::WriteFinished)?;
    }

    Ok({
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.current.clone()
    })
}

/// True when the scan captured T55xx config blocks on the source card.
///
/// This is the routing signal for the block-copy path, deliberately NOT the
/// identified card type: a T5577 source is frequently identified as EM410x (or
/// other) while still carrying the config blocks that make it a universal tag.
fn has_t55xx_blocks(decoded: &std::collections::HashMap<String, String>) -> bool {
    use crate::pm3::command_builder::T55XX_BLOCK_KEY_PREFIX;
    decoded.keys().any(|k| k.starts_with(T55XX_BLOCK_KEY_PREFIX))
}

/// Write the source T55xx's config blocks onto the blank on the reader.
///
/// Each block is written and verified individually. Progress is reported per
/// block so the UI shows real movement rather than sitting at one value, and the
/// first block that fails stops the run -- continuing would leave the target in a
/// half-configured state that is harder to recover than a clean failure.
async fn write_t55xx_blocks(
    app: &AppHandle,
    port: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<(), String> {
    let (commands, total) = command_builder::build_t55xx_block_clone(decoded)?;

    log::debug!("T55xx block clone: {} block(s)", total);

    for (i, cmd) in commands.iter().enumerate() {
        // Scale the clone step across the last portion of the progress bar.
        let progress = 0.7 + (0.3 * (i as f32 / total as f32));
        update_progress(app, machine, progress, Some(i as u16), Some(total as u16))
            .map_err(|e| e.to_string())?;

        log::debug!("T55xx block {}/{}: {}", i + 1, total, cmd);

        // A T55xx write can legitimately exit non-zero while still succeeding,
        // so tolerate the exit code and judge from the output instead.
        let out = connection::run_search_command(app, port, cmd)
            .await
            .map_err(|e| format!("block {} of {} failed: {}", i + 1, total, e))?;

        output_parser::parse_t55xx_write_result(&out)
            .map_err(|e| format!("block {} of {} failed: {}", i + 1, total, e))?;
    }

    update_progress(app, machine, 1.0, Some(total as u16), Some(total as u16))
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// EM4305 write flow with detect + wipe-verify safety checks:
/// 1. detect EM4305 -> 2. wipe -> 3. verify wipe -> 4. clone with --em -> 5. done
async fn write_em4305_flow(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
    machine: &State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Step 1: Detect EM4305 — verify the blank chip is present before wiping.
    // Mirrors the T5577 detect step to prevent wiping air / wrong chip.
    update_progress(app, machine, 0.1, Some(0), Some(EM4305_TOTAL_STEPS))?;

    let info_out =
        connection::run_command(app, port, command_builder::build_em4305_info()).await?;

    if !output_parser::parse_em4305_info(&info_out) {
        return report_error(
            machine,
            "EM4305 not detected on writer",
            "No EM4305 blank found. Place blank card on the reader.",
            true,
            Some(RecoveryAction::Retry),
        );
    }

    // Step 2: Wipe EM4305
    update_progress(app, machine, 0.3, Some(1), Some(EM4305_TOTAL_STEPS))?;

    connection::run_command(app, port, command_builder::build_em4305_wipe()).await?;

    // Step 3: Verify wipe — read word 0 and check it's zeroed.
    // PM3 can return exit code 0 even when wipe fails silently.
    // Proceeding to clone without this check risks corrupted data on the card.
    update_progress(app, machine, 0.5, Some(2), Some(EM4305_TOTAL_STEPS))?;

    let verify_out =
        connection::run_command(app, port, &command_builder::build_em4305_read_word(0)).await?;
    if let Some(word0) = output_parser::parse_em4305_word0(&verify_out) {
        if word0 != "00000000" {
            return report_error(
                machine,
                &format!(
                    "EM4305 wipe verification failed — word 0 is {} (expected 00000000)",
                    word0
                ),
                "Wipe verification failed. The card may not have been wiped correctly. \
                 Do not remove the card — try again or use a different blank.",
                true,
                Some(RecoveryAction::Retry),
            );
        }
    }
    // If parse_em4305_word0 returns None, we can't verify — proceed with caution.
    // This is acceptable: the clone step will fail if the card is in a bad state.

    // Step 4: Clone with --em flag
    update_progress(app, machine, 0.7, Some(3), Some(EM4305_TOTAL_STEPS))?;

    let base_clone_cmd = command_builder::build_clone_command(card_type, uid, decoded);
    match base_clone_cmd {
        Some(cmd) => {
            let em_cmd = command_builder::build_clone_for_em4305(&cmd);
            let clone_output = connection::run_command(app, port, &em_cmd).await?;
            // Check for failure indicators in PM3 output
            if clone_output.contains("[!!]")
                || clone_output.to_lowercase().contains("fail")
            {
                return report_error(
                    machine,
                    &format!("EM4305 clone may have failed: {}", clone_output.chars().take(200).collect::<String>()),
                    "Write may have failed. Do not remove the card — try again.",
                    true,
                    Some(RecoveryAction::Retry),
                );
            }
        }
        None => {
            return report_error(
                machine,
                &format!("No clone command for {:?}", card_type),
                "This card type cannot be cloned with the current method.",
                false,
                None,
            );
        }
    }

    // Step 5: Done -> Verifying
    update_progress(app, machine, 1.0, Some(4), Some(EM4305_TOTAL_STEPS))?;
    {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.transition(WizardAction::WriteFinished)?;
    }

    Ok({
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        m.current.clone()
    })
}

/// Verify the clone by reading the written card and comparing fields.
/// Uses type-specific reader commands for more accurate verification.
///
/// `blank_type` is reserved for Phase 3 HF card verification where the blank
/// type determines the verification command. Currently unused for LF cards.
#[tauri::command]
pub async fn verify_clone(
    app: AppHandle,
    port: String,
    source_uid: String,
    source_card_type: CardType,
    source_decoded: Option<std::collections::HashMap<String, String>>,
    _blank_type: Option<BlankType>,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Guard: must be in Verifying state before running any hardware commands.
    // Without this check, a call from the wrong state would waste a PM3
    // command before failing on the FSM transition.
    {
        let m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        match &m.current {
            WizardState::Verifying => {}
            other => {
                return Err(AppError::InvalidTransition(format!(
                    "Must be in Verifying state to verify clone, currently in {:?}",
                    std::mem::discriminant(other)
                )));
            }
        }
    }

    // Use generic `lf search` for verification — parse_lf_search is designed to parse
    // its output format. Type-specific readers (lf hid reader, etc.) produce different
    // output that parse_lf_search can't handle, causing false verification failures.
    //
    // `run_search_command`, not `run_command`: `lf search` on a T55xx ALWAYS exits -10,
    // after printing `[-] No known 125/134 kHz tags found!` and `[+] Chipset... T55xx`.
    // The strict path throws the stdout away on any non-zero exit, so verification of a
    // perfectly good T5577 died with "No tag found running: lf search".
    let verify_output = connection::run_search_command(&app, &port, "lf search").await?;

    // Use detailed verification if decoded fields are available
    let (success, mismatched) = if let Some(ref decoded) = source_decoded {
        output_parser::verify_match_detailed(&source_card_type, decoded, &verify_output)
    } else {
        output_parser::verify_match(&source_uid, &verify_output)
    };

    // Record the verdict AND what was compared against what. "Verification
    // failed" with no record of the expected vs actual values is the hardest
    // kind of report to act on.
    logging::operation(
        "VERIFY CLONE",
        &format!(
            "port={} card_type={:?} expected_uid={} method={}",
            port,
            source_card_type,
            source_uid,
            if source_decoded.is_some() {
                "detailed field compare"
            } else {
                "uid compare"
            }
        ),
        &format!(
            "success={} mismatched={:?}",
            success, mismatched
        ),
    );
    if !success {
        logging::line("Verification FAILED. Raw verification output follows:");
        logging::log(&format!(
            "--- verify raw output begin ---\n{}\n--- verify raw output end ---",
            verify_output.trim_end()
        ));
        if let Some(ref decoded) = source_decoded {
            logging::line("Expected (source) fields:");
            let mut keys: Vec<&String> = decoded.keys().collect();
            keys.sort();
            for k in keys {
                logging::line(&format!("  {} = {}", k, decoded[k]));
            }
        }
    }

    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::VerificationResult {
        success,
        mismatched_blocks: mismatched.clone(),
    })?;

    // Note: VerificationComplete stores success/failure. The FINISH/MarkComplete
    // transition is guarded in both state.rs (line 272: `success: true` pattern match)
    // and wizardMachine.ts (guard: context.verifySuccess === true) to prevent
    // completing with failed verification. No additional guard needed here.
    if !success {
        log::warn!(
            "Verification failed: {} mismatched blocks",
            mismatched.len()
        );
    }

    Ok(m.current.clone())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn update_progress(
    app: &AppHandle,
    machine: &State<'_, Mutex<WizardMachine>>,
    progress: f32,
    current_block: Option<u16>,
    total_blocks: Option<u16>,
) -> Result<(), AppError> {
    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::UpdateWriteProgress {
        progress,
        current_block,
        total_blocks,
    })?;
    // Emit event to frontend for real-time progress updates
    if let Err(e) = app.emit(
        "write-progress",
        serde_json::json!({
            "progress": progress,
            "current_block": current_block,
            "total_blocks": total_blocks,
        }),
    ) {
        log::warn!("Failed to emit write-progress event: {}", e);
    }
    Ok(())
}

fn report_error(
    machine: &State<'_, Mutex<WizardMachine>>,
    message: &str,
    user_message: &str,
    recoverable: bool,
    recovery_action: Option<RecoveryAction>,
) -> Result<WizardState, AppError> {
    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::ReportError {
        message: message.to_string(),
        user_message: user_message.to_string(),
        recoverable,
        recovery_action,
    })?;
    Ok(m.current.clone())
}

#[cfg(test)]
mod tests {
    use super::has_t55xx_blocks;

    /// A source card identified as EM410x but carrying T55xx config blocks must
    /// still take the block-copy path. This is the user's card (logs 1344-1360):
    /// `lf search` reported `EM 410x ID 4E008E7AC1` alongside `[+] Chipset... T55xx`,
    /// and blocks 0-2 were read successfully. Routing on card type sent it to
    /// "this card type cannot be cloned" instead of copying the blocks.
    #[test]
    fn em410x_with_blocks_routes_to_block_copy() {
        let decoded: std::collections::HashMap<String, String> = [
            ("type", "EM4100"),
            ("id", "4E008E7AC1"),
            ("t55xx_chipset", "T55XX"),
            ("t55xx_blk_0", "00148040"),
            ("t55xx_blk_1", "FFA7A004"),
            ("t55xx_blk_2", "7AFA6078"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        assert!(
            has_t55xx_blocks(&decoded),
            "EM410x card with T55xx blocks must take the block-copy path"
        );
    }

    /// A single block is enough -- blocks 3-7 read as zeros on a partially
    /// written card, but block 0 alone still reproduces the tag.
    #[test]
    fn single_block_is_enough_to_route() {
        let decoded: std::collections::HashMap<String, String> =
            [("t55xx_blk_0", "00148040")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
        assert!(has_t55xx_blocks(&decoded));
    }

    /// An EM410x card with NO config blocks must not be routed to the T55xx
    /// writer; it has nothing to copy block-wise.
    #[test]
    fn em410x_without_blocks_does_not_route() {
        let decoded: std::collections::HashMap<String, String> = [
            ("type", "EM4100"),
            ("id", "0F00112233"),
            ("t55xx_chipset", "T55XX"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        assert!(!has_t55xx_blocks(&decoded));
    }

    #[test]
    fn empty_decoded_does_not_route() {
        let decoded = std::collections::HashMap::new();
        assert!(!has_t55xx_blocks(&decoded));
    }

    /// The key prefix must not be matched loosely -- an unrelated decoded field
    /// must never be mistaken for block data.
    #[test]
    fn unrelated_keys_do_not_route() {
        let decoded: std::collections::HashMap<String, String> = [
            ("type", "EM4100"),
            ("t55xx_chipset", "T55XX"),
            ("blkid", "0"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        assert!(!has_t55xx_blocks(&decoded));
    }
}
