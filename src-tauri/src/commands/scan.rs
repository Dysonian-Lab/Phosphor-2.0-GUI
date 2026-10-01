use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::cards::types::{CardType, RecoveryAction};
use crate::error::AppError;
use crate::pm3::{command_builder, connection, output_parser};
use crate::state::{WizardAction, WizardMachine, WizardState};

#[tauri::command]
pub async fn scan_card(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
) -> Result<WizardState, AppError> {
    // Get the port from current state, then transition to ScanningCard
    let port = {
        let mut m = machine.lock().map_err(|e| {
            AppError::CommandFailed(format!("State lock poisoned: {}", e))
        })?;
        let port = match &m.current {
            WizardState::DeviceConnected { port, .. } => port.clone(),
            _ => {
                return Err(AppError::InvalidTransition(
                    "Must be in DeviceConnected to scan".to_string(),
                ));
            }
        };
        m.transition(WizardAction::StartScan)?;
        port
    };

    // 1. Try LF search first (fast path for 125 kHz cards)
    // `run_search_command` keeps the output even when PM3 exits non-zero, which
    // matters here: `lf search` exits -10 on a T5577 *after* printing
    // "[+] Chipset... T55xx". The strict path discarded that and reported a
    // connection failure for a card that was detected.
    let lf_result =
        connection::run_search_command(&app, &port, command_builder::build_lf_search()).await;

    if let Ok(ref output) = lf_result {
        if let Some((card_type, mut card_data)) = output_parser::parse_lf_search(output) {
            // T55xx carries its clonable payload in config blocks that `lf search`
            // does not print, so they have to be read explicitly.
            if card_type == CardType::T55xx {
                enrich_t55xx_data(&app, &port, &mut card_data).await;
            }
            return finish_scan(&machine, card_type, card_data);
        }
    }

    // 2. LF found nothing → try HF search (13.56 MHz)
    let hf_result =
        connection::run_search_command(&app, &port, command_builder::build_hf_search()).await;

    match hf_result {
        Ok(output) => {
            if let Some((card_type, mut card_data)) = output_parser::parse_hf_search(&output)
            {
                // Enrich HF data with protocol-specific info commands
                enrich_hf_data(&app, &port, &card_type, &mut card_data).await;
                return finish_scan(&machine, card_type, card_data);
            }

            // Neither LF nor HF found a card
            let mut m = machine.lock().map_err(|e| {
                AppError::CommandFailed(format!("State lock poisoned: {}", e))
            })?;
            m.transition(WizardAction::ReportError {
                message: "No card detected".to_string(),
                user_message: "No card found. Place the card on the reader and try again."
                    .to_string(),
                recoverable: true,
                recovery_action: Some(RecoveryAction::Retry),
            })?;
            Ok(m.current.clone())
        }
        Err(_) => {
            // HF search also failed — check if LF had a connection error
            if let Err(e) = lf_result {
                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
            m.transition(WizardAction::ReportError {
                message: e.to_string(),
                // Only spawn/timeout failures land here now — the search commands
                // return their output regardless of exit code, so a card that was
                // detected but not classified no longer ends up on this path.
                user_message: "Could not run the scan. Check the device connection and that
                    the Proxmark3 is switched on."
                    .to_string(),
                recoverable: true,
                recovery_action: Some(RecoveryAction::Retry),
            })?;
                Ok(m.current.clone())
            } else {
                let mut m = machine.lock().map_err(|e| {
                    AppError::CommandFailed(format!("State lock poisoned: {}", e))
                })?;
                m.transition(WizardAction::ReportError {
                    message: "No card detected".to_string(),
                    user_message:
                        "No card found. Place the card on the reader and try again."
                            .to_string(),
                    recoverable: true,
                    recovery_action: Some(RecoveryAction::Retry),
                })?;
                Ok(m.current.clone())
            }
        }
    }
}

/// Read a detected T55xx's functional config blocks (0-7) so the card can
/// actually be cloned onto a blank.
///
/// `lf search` only reports the chipset. The clonable payload lives in the
/// config blocks, and each needs its own `lf t55xx read -b <n>`, so this is
/// 8 round trips. That is the cost of making the card writable, and it is only
/// paid for T55xx -- no other scan path is slowed.
///
/// A blank T5577 has no data in these blocks; PM3 returns only a table header
/// and exits -7. That is recorded explicitly so the write step can explain
/// itself instead of failing with a confusing "no clone command".
async fn enrich_t55xx_data(
    app: &AppHandle,
    port: &str,
    card_data: &mut crate::cards::types::CardData,
) {
    use crate::pm3::command_builder::T55XX_BLOCK_KEY_PREFIX;

    const BLOCKS: std::ops::Range<u8> = 0..8;
    let mut captured = 0u32;

    for block in BLOCKS {
        let cmd = match command_builder::build_t55xx_read(block) {
            Ok(c) => c,
            Err(_) => break,
        };

        // `run_t55xx_memory_command` prepends the `lf t55xx detect` that the
        // client requires: run bare, `lf t55xx read -b N` returns an empty
        // table even when the card reads fine, which is what made every T5577
        // report "blocks: none (blank or unprogrammed chip)".
        let out = match connection::run_t55xx_memory_command(app, port, &cmd).await {
            Ok(o) => o,
            Err(_) => continue,
        };

        if let Some(hex) = output_parser::parse_t55xx_read_block(&out) {
            card_data
                .decoded
                .insert(format!("{}{}", T55XX_BLOCK_KEY_PREFIX, block), hex);
            captured += 1;
        }
    }

    if captured == 0 {
        card_data.decoded.insert(
            "blocks".to_string(),
            "none (blank or unprogrammed chip)".to_string(),
        );
    } else {
        card_data
            .decoded
            .insert("blocks".to_string(), captured.to_string());
    }
}

/// Enrich HF card data with protocol-specific info commands.
/// For MIFARE Classic: `hf 14a info` (PRNG) + `hf mf info` (magic detection).
/// For UL/NTAG: `hf mfu info` for subtype detection.
async fn enrich_hf_data(
    app: &AppHandle,
    port: &str,
    card_type: &CardType,
    card_data: &mut crate::cards::types::CardData,
) {
    match card_type {
        CardType::MifareClassic1K | CardType::MifareClassic4K => {
            // Get PRNG info if not already present
            if !card_data.decoded.contains_key("prng") {
                if let Ok(info_output) =
                    connection::run_command(app, port, command_builder::build_hf_14a_info())
                        .await
                {
                    let clean = output_parser::strip_ansi(&info_output);
                    if let Some(caps) =
                        regex::Regex::new(r"(?i)Prng\s+detection[\s.:]+(WEAK|HARD|STATIC)")
                            .ok()
                            .and_then(|re| re.captures(&clean))
                    {
                        card_data
                            .decoded
                            .insert("prng".to_string(), caps[1].to_uppercase());
                    }
                }
            }
            // Get magic card info
            if !card_data.decoded.contains_key("magic") {
                if let Ok(mf_output) =
                    connection::run_command(app, port, command_builder::build_hf_mf_info())
                        .await
                {
                    let clean = output_parser::strip_ansi(&mf_output);
                    if let Some(caps) = regex::Regex::new(r"(?i)(?:Magic|Gen(?:eration)?)\s*(?:capabilities)?[\s.:]*(?::[\s.]*)?(Gen\s*1[ab]?|CUID|USCUID|Gen\s*2|Gen\s*3|APDU|UFUID|GDM|Gen\s*4\s*(?:GTU|GDM)?|[Uu]ltimate)")
                        .ok()
                        .and_then(|re| re.captures(&clean))
                    {
                        card_data
                            .decoded
                            .insert("magic".to_string(), caps[1].to_string());
                    }
                }
            }
        }
        CardType::MifareUltralight | CardType::NTAG => {
            // Get UL/NTAG subtype info
            if let Ok(mfu_output) =
                connection::run_command(app, port, command_builder::build_hf_mfu_info()).await
            {
                let clean = output_parser::strip_ansi(&mfu_output);
                // Check for NTAG type
                if let Some(caps) = regex::Regex::new(r"(?i)NTAG\s*(\d{3})")
                    .ok()
                    .and_then(|re| re.captures(&clean))
                {
                    card_data
                        .decoded
                        .insert("ntag_type".to_string(), format!("NTAG{}", &caps[1]));
                }
                // Check for UL type
                if let Some(caps) =
                    regex::Regex::new(r"(?i)(?:MIFARE\s+)?Ultralight(?:\s+(EV1|C|Nano|AES))?")
                        .ok()
                        .and_then(|re| re.captures(&clean))
                {
                    if let Some(ul_variant) = caps.get(1) {
                        card_data.decoded.insert(
                            "ul_type".to_string(),
                            format!("Ultralight {}", ul_variant.as_str()),
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// Common finish: transition FSM to CardFound with detected card info.
fn finish_scan(
    machine: &Mutex<WizardMachine>,
    card_type: CardType,
    card_data: crate::cards::types::CardData,
) -> Result<WizardState, AppError> {
    let frequency = card_type.frequency();
    let cloneable = card_type.is_cloneable();
    let recommended_blank = card_type.recommended_blank();

    let mut m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;
    m.transition(WizardAction::CardFound {
        frequency,
        card_type,
        card_data,
        cloneable,
        recommended_blank,
    })?;
    Ok(m.current.clone())
}
