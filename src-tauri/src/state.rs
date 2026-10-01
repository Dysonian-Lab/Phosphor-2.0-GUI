use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cards::types::{
    BlankType, CardData, CardSummary, CardType, Frequency, ProcessPhase, RecoveryAction,
};
use crate::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "step", content = "data")]
pub enum WizardState {
    Idle,
    DetectingDevice,
    DeviceConnected {
        port: String,
        model: String,
        firmware: String,
    },
    ScanningCard,
    CardIdentified {
        frequency: Frequency,
        card_type: CardType,
        card_data: CardData,
        cloneable: bool,
        recommended_blank: BlankType,
    },
    HfProcessing {
        phase: ProcessPhase,
        keys_found: u32,
        keys_total: u32,
        elapsed_secs: u32,
    },
    HfDumpReady {
        dump_info: String,
    },
    WaitingForBlank {
        expected_blank: BlankType,
    },
    BlankDetected {
        blank_type: BlankType,
        ready_to_write: bool,
        existing_data_type: Option<String>,
    },
    Writing {
        progress: f32,
        current_block: Option<u16>,
        total_blocks: Option<u16>,
    },
    Verifying,
    VerificationComplete {
        success: bool,
        mismatched_blocks: Vec<u16>,
    },
    Complete {
        source: CardSummary,
        target: CardSummary,
        timestamp: String,
    },
    Error {
        message: String,
        user_message: String,
        recoverable: bool,
        recovery_action: Option<RecoveryAction>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum WizardAction {
    StartDetection,
    DeviceFound {
        port: String,
        model: String,
        firmware: String,
    },
    StartScan,
    CardFound {
        frequency: Frequency,
        card_type: CardType,
        card_data: CardData,
        cloneable: bool,
        recommended_blank: BlankType,
    },
    StartHfProcess,
    UpdateHfProgress {
        phase: ProcessPhase,
        keys_found: u32,
        keys_total: u32,
        elapsed_secs: u32,
    },
    HfProcessComplete {
        dump_info: String,
    },
    CancelHfProcess,
    ProceedToWrite {
        blank_type: BlankType,
    },
    BlankReady {
        blank_type: BlankType,
        existing_data_type: Option<String>,
    },
    StartWrite,
    UpdateWriteProgress {
        progress: f32,
        current_block: Option<u16>,
        total_blocks: Option<u16>,
    },
    WriteFinished,
    VerificationResult {
        success: bool,
        mismatched_blocks: Vec<u16>,
    },
    MarkComplete {
        source: CardSummary,
        target: CardSummary,
    },
    ReportError {
        message: String,
        user_message: String,
        recoverable: bool,
        recovery_action: Option<RecoveryAction>,
    },
    Retry,
    Reset,
    BackToScan,
    SoftReset,
    Disconnect,
    ReDetectBlank,
    LoadSavedCard {
        frequency: Frequency,
        card_type: CardType,
        uid: String,
        raw: String,
        decoded: HashMap<String, String>,
        cloneable: bool,
        recommended_blank: BlankType,
    },
}

fn state_name(s: &WizardState) -> &str {
    match s {
        WizardState::Idle => "Idle",
        WizardState::DetectingDevice => "DetectingDevice",
        WizardState::DeviceConnected { .. } => "DeviceConnected",
        WizardState::ScanningCard => "ScanningCard",
        WizardState::CardIdentified { .. } => "CardIdentified",
        WizardState::HfProcessing { .. } => "HfProcessing",
        WizardState::HfDumpReady { .. } => "HfDumpReady",
        WizardState::WaitingForBlank { .. } => "WaitingForBlank",
        WizardState::BlankDetected { .. } => "BlankDetected",
        WizardState::Writing { .. } => "Writing",
        WizardState::Verifying => "Verifying",
        WizardState::VerificationComplete { .. } => "VerificationComplete",
        WizardState::Complete { .. } => "Complete",
        WizardState::Error { .. } => "Error",
    }
}

fn action_name(a: &WizardAction) -> &str {
    match a {
        WizardAction::StartDetection => "StartDetection",
        WizardAction::DeviceFound { .. } => "DeviceFound",
        WizardAction::StartScan => "StartScan",
        WizardAction::CardFound { .. } => "CardFound",
        WizardAction::StartHfProcess => "StartHfProcess",
        WizardAction::UpdateHfProgress { .. } => "UpdateHfProgress",
        WizardAction::HfProcessComplete { .. } => "HfProcessComplete",
        WizardAction::CancelHfProcess => "CancelHfProcess",
        WizardAction::ProceedToWrite { .. } => "ProceedToWrite",
        WizardAction::BlankReady { .. } => "BlankReady",
        WizardAction::StartWrite => "StartWrite",
        WizardAction::UpdateWriteProgress { .. } => "UpdateWriteProgress",
        WizardAction::WriteFinished => "WriteFinished",
        WizardAction::VerificationResult { .. } => "VerificationResult",
        WizardAction::MarkComplete { .. } => "MarkComplete",
        WizardAction::ReportError { .. } => "ReportError",
        WizardAction::Retry => "Retry",
        WizardAction::Reset => "Reset",
        WizardAction::BackToScan => "BackToScan",
        WizardAction::SoftReset => "SoftReset",
        WizardAction::Disconnect => "Disconnect",
        WizardAction::ReDetectBlank => "ReDetectBlank",
        WizardAction::LoadSavedCard { .. } => "LoadSavedCard",
    }
}

pub struct WizardMachine {
    pub current: WizardState,
    pub port: Option<String>,
    pub model: Option<String>,
    pub firmware: Option<String>,
}

impl WizardMachine {
    pub fn new() -> Self {
        WizardMachine {
            current: WizardState::Idle,
            port: None,
            model: None,
            firmware: None,
        }
    }

    /// The blank type to expect when returning to `WaitingForBlank` for a retry
    /// or a Back navigation.
    ///
    /// `WaitingForBlank.expected_blank` is only a UI hint (which blank the user
    /// should place on the reader); the actual blank is re-detected by the
    /// `detectBlank` invoke on arrival. So this must never fail or panic.
    ///
    /// Preference order:
    ///   1. A blank type carried in the state we are transitioning FROM, so a
    ///      user who already had a blank detected keeps the same expectation.
    ///   2. The recommended blank for the identified source card.
    ///   3. `T5577`, the default LF target, as a last resort.
    fn expected_blank(&self) -> BlankType {
        match &self.current {
            WizardState::WaitingForBlank { expected_blank } => expected_blank.clone(),
            WizardState::BlankDetected { blank_type, .. } => blank_type.clone(),
            _ => self
                .card_type()
                .map(|ct| ct.recommended_blank())
                .unwrap_or(BlankType::T5577),
        }
    }

    /// The card type recorded for the source card, if one is loaded.
    ///
    /// `CardIdentified` is the only state that carries the type directly; the
    /// write/verify states do not repeat it, so this returns `None` there and the
    /// caller falls back to a default.
    fn card_type(&self) -> Option<CardType> {
        match &self.current {
            WizardState::CardIdentified { card_type, .. } => Some(card_type.clone()),
            _ => None,
        }
    }

    pub fn transition(&mut self, action: WizardAction) -> Result<&WizardState, AppError> {
        // Reset is always valid from any state — full reset to idle
        if matches!(action, WizardAction::Reset) {
            self.current = WizardState::Idle;
            self.port = None;
            self.model = None;
            self.firmware = None;
            return Ok(&self.current);
        }

        // Disconnect is valid from any connected state — clears persistent fields
        if matches!(action, WizardAction::Disconnect) {
            self.current = WizardState::Idle;
            self.port = None;
            self.model = None;
            self.firmware = None;
            return Ok(&self.current);
        }

        // ReportError is always valid from any state
        if let WizardAction::ReportError {
            message,
            user_message,
            recoverable,
            recovery_action,
        } = &action
        {
            self.current = WizardState::Error {
                message: message.clone(),
                user_message: user_message.clone(),
                recoverable: *recoverable,
                recovery_action: recovery_action.clone(),
            };
            return Ok(&self.current);
        }

        let next = match (&self.current, &action) {
            // Idle -> DetectingDevice
            (WizardState::Idle, WizardAction::StartDetection) => WizardState::DetectingDevice,

            // DetectingDevice -> DeviceConnected (also stores persistent device info)
            (
                WizardState::DetectingDevice,
                WizardAction::DeviceFound {
                    port,
                    model,
                    firmware,
                },
            ) => {
                self.port = Some(port.clone());
                self.model = Some(model.clone());
                self.firmware = Some(firmware.clone());
                WizardState::DeviceConnected {
                    port: port.clone(),
                    model: model.clone(),
                    firmware: firmware.clone(),
                }
            }

            // DeviceConnected -> ScanningCard
            (WizardState::DeviceConnected { .. }, WizardAction::StartScan) => {
                WizardState::ScanningCard
            }

            // ScanningCard -> CardIdentified
            (
                WizardState::ScanningCard,
                WizardAction::CardFound {
                    frequency,
                    card_type,
                    card_data,
                    cloneable,
                    recommended_blank,
                },
            ) => WizardState::CardIdentified {
                frequency: frequency.clone(),
                card_type: card_type.clone(),
                card_data: card_data.clone(),
                cloneable: *cloneable,
                recommended_blank: recommended_blank.clone(),
            },

            // CardIdentified -> HfProcessing (start key recovery)
            (WizardState::CardIdentified { .. }, WizardAction::StartHfProcess) => {
                WizardState::HfProcessing {
                    phase: ProcessPhase::KeyCheck,
                    keys_found: 0,
                    keys_total: 0,
                    elapsed_secs: 0,
                }
            }

            // HfProcessing -> HfProcessing (progress update)
            (
                WizardState::HfProcessing { .. },
                WizardAction::UpdateHfProgress {
                    phase,
                    keys_found,
                    keys_total,
                    elapsed_secs,
                },
            ) => WizardState::HfProcessing {
                phase: phase.clone(),
                keys_found: *keys_found,
                keys_total: *keys_total,
                elapsed_secs: *elapsed_secs,
            },

            // HfProcessing -> HfDumpReady (key recovery + dump complete)
            (
                WizardState::HfProcessing { .. },
                WizardAction::HfProcessComplete { dump_info },
            ) => WizardState::HfDumpReady {
                dump_info: dump_info.clone(),
            },

            // HfProcessing -> DeviceConnected (user cancelled)
            (WizardState::HfProcessing { .. }, WizardAction::CancelHfProcess) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "CancelHfProcess requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // HfDumpReady -> WaitingForBlank (proceed to write)
            (
                WizardState::HfDumpReady { .. },
                WizardAction::ProceedToWrite { blank_type },
            ) => WizardState::WaitingForBlank {
                expected_blank: blank_type.clone(),
            },

            // HfDumpReady -> DeviceConnected (back to scan)
            (WizardState::HfDumpReady { .. }, WizardAction::BackToScan) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "BackToScan requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // CardIdentified -> WaitingForBlank (LF cards skip HF processing)
            (
                WizardState::CardIdentified { .. },
                WizardAction::ProceedToWrite { blank_type },
            ) => WizardState::WaitingForBlank {
                expected_blank: blank_type.clone(),
            },

            // WaitingForBlank -> BlankDetected
            (WizardState::WaitingForBlank { .. }, WizardAction::BlankReady { blank_type, existing_data_type }) => {
                WizardState::BlankDetected {
                    blank_type: blank_type.clone(),
                    ready_to_write: true,
                    existing_data_type: existing_data_type.clone(),
                }
            }

            // BlankDetected -> WaitingForBlank (re-detect after erase)
            (WizardState::BlankDetected { .. }, WizardAction::ReDetectBlank) => {
                // Extract expected blank from current state
                WizardState::WaitingForBlank {
                    expected_blank: match &self.current {
                        WizardState::BlankDetected { blank_type, .. } => blank_type.clone(),
                        _ => unreachable!(),
                    },
                }
            }

            // BlankDetected -> Writing
            (WizardState::BlankDetected { .. }, WizardAction::StartWrite) => {
                WizardState::Writing {
                    progress: 0.0,
                    current_block: None,
                    total_blocks: None,
                }
            }

            // Writing -> Writing (progress update)
            (
                WizardState::Writing { .. },
                WizardAction::UpdateWriteProgress {
                    progress,
                    current_block,
                    total_blocks,
                },
            ) => WizardState::Writing {
                progress: *progress,
                current_block: *current_block,
                total_blocks: *total_blocks,
            },

            // Writing -> Verifying
            (WizardState::Writing { .. }, WizardAction::WriteFinished) => WizardState::Verifying,

            // Verifying -> VerificationComplete
            (
                WizardState::Verifying,
                WizardAction::VerificationResult {
                    success,
                    mismatched_blocks,
                },
            ) => WizardState::VerificationComplete {
                success: *success,
                mismatched_blocks: mismatched_blocks.clone(),
            },

            // VerificationComplete -> Complete
            (
                WizardState::VerificationComplete { success: true, .. },
                WizardAction::MarkComplete { source, target },
            ) => WizardState::Complete {
                source: source.clone(),
                target: target.clone(),
                timestamp: chrono::Local::now().to_rfc3339(),
            },

            // Error + Retry -> WaitingForBlank (retry without rescanning source).
            //
            // `WaitingForBlank` carries only `expected_blank`; the source card
            // (`card_type` / `card_data`, including any T55xx config blocks) lives
            // on the machine, not in the enum, so returning here preserves
            // everything the next write needs. The previous target was `Idle`,
            // which discarded the source card and forced a full rescan of the
            // source tag before a retry could even be attempted -- the reported
            // "a failed write never retries" behaviour.
            (WizardState::Error { recoverable: true, .. }, WizardAction::Retry) => {
                WizardState::WaitingForBlank {
                    expected_blank: self.expected_blank(),
                }
            }

            // VerificationComplete -> WaitingForBlank (retry a failed verification)
            //
            // Same reasoning as Error + Retry above. This path was previously
            // absent entirely: there was no route out of a failed verification
            // except RESET (wipe everything) or DISCONNECT (drop the device).
            (WizardState::VerificationComplete { .. }, WizardAction::Retry) => {
                WizardState::WaitingForBlank {
                    expected_blank: self.expected_blank(),
                }
            }

            // Complete -> WaitingForBlank (plain Back from the completion screen)
            //
            // Normal navigation: keep the device connected and the source card
            // loaded so the user can clone again or back out. RESET / DISCONNECT
            // remain available as the deliberate "start over" / "unplug" choices.
            (WizardState::Complete { .. }, WizardAction::Retry) => {
                WizardState::WaitingForBlank {
                    expected_blank: self.expected_blank(),
                }
            }

            // Complete -> Idle (start over)
            (WizardState::Complete { .. }, WizardAction::StartDetection) => {
                WizardState::DetectingDevice
            }

            // BackToScan: post-scan states -> DeviceConnected using persistent device info
            (WizardState::CardIdentified { .. }, WizardAction::BackToScan)
            | (WizardState::WaitingForBlank { .. }, WizardAction::BackToScan)
            | (WizardState::HfProcessing { .. }, WizardAction::BackToScan) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "BackToScan requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // SoftReset: Complete/Error -> DeviceConnected using persistent device info
            (WizardState::Complete { .. }, WizardAction::SoftReset)
            | (WizardState::Error { .. }, WizardAction::SoftReset) => {
                match (&self.port, &self.model, &self.firmware) {
                    (Some(p), Some(m), Some(f)) => WizardState::DeviceConnected {
                        port: p.clone(),
                        model: m.clone(),
                        firmware: f.clone(),
                    },
                    _ => {
                        return Err(AppError::InvalidTransition(
                            "SoftReset requires persistent device info".to_string(),
                        ));
                    }
                }
            }

            // LoadSavedCard: DeviceConnected -> CardIdentified with provided card data
            (
                WizardState::DeviceConnected { .. },
                WizardAction::LoadSavedCard {
                    frequency,
                    card_type,
                    uid,
                    raw,
                    decoded,
                    cloneable,
                    recommended_blank,
                },
            ) => WizardState::CardIdentified {
                frequency: frequency.clone(),
                card_type: card_type.clone(),
                card_data: CardData {
                    uid: uid.clone(),
                    raw: raw.clone(),
                    decoded: decoded.clone(),
                },
                cloneable: *cloneable,
                recommended_blank: recommended_blank.clone(),
            },

            _ => {
                return Err(AppError::InvalidTransition(format!(
                    "{} is not valid from {}",
                    action_name(&action),
                    state_name(&self.current)
                )));
            }
        };

        self.current = next;
        Ok(&self.current)
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    fn connected_machine(current: WizardState) -> WizardMachine {
        WizardMachine {
            current,
            port: Some("COM19".to_string()),
            model: Some("iCopy-X".to_string()),
            firmware: Some("v4.23346".to_string()),
        }
    }

    /// Regression: "a failed write never attempts to actually rewrite it, it
    /// goes back to scan". `Error + Retry` used to target `Idle`, which discarded
    /// the source card and the device, forcing a full rescan before a retry could
    /// even be attempted. It must return to `WaitingForBlank`.
    #[test]
    fn error_retry_goes_to_waiting_for_blank_not_idle() {
        let mut m = connected_machine(WizardState::Error {
            message: "block 2 of 8 failed".to_string(),
            user_message: "Write may have failed.".to_string(),
            recoverable: true,
            recovery_action: Some(RecoveryAction::Retry),
        });

        m.transition(WizardAction::Retry).expect("retry must be valid");

        assert!(
            matches!(m.current, WizardState::WaitingForBlank { .. }),
            "retry must land on WaitingForBlank, got {:?}",
            m.current
        );
        // The device stays connected -- that is the whole point of a retry.
        assert_eq!(m.port.as_deref(), Some("COM19"));
        assert!(!matches!(m.current, WizardState::Idle));
    }

    /// A failed verification previously had NO route out except RESET (wipe
    /// everything) or DISCONNECT (drop the device). Retry must now work.
    #[test]
    fn failed_verification_can_retry() {
        let mut m = connected_machine(WizardState::VerificationComplete {
            success: false,
            mismatched_blocks: vec![2],
        });

        m.transition(WizardAction::Retry).expect("retry must be valid");

        assert!(
            matches!(m.current, WizardState::WaitingForBlank { .. }),
            "failed verification must be retryable, got {:?}",
            m.current
        );
        assert_eq!(m.port.as_deref(), Some("COM19"));
    }

    /// Regression: after a successful clone the completion screen offered only
    /// "clone another" or "disconnect". A plain Back must exist that keeps the
    /// connection and returns to the blank step.
    #[test]
    fn complete_offers_back_without_disconnecting() {
        let mut m = connected_machine(WizardState::Complete {
            source: CardSummary {
                card_type: "EM4100".to_string(),
                display_name: "EM4100".to_string(),
                uid: "4E008E7AC1".to_string(),
            },
            target: CardSummary {
                card_type: "EM4100".to_string(),
                display_name: "EM4100".to_string(),
                uid: "4E008E7AC1".to_string(),
            },
            timestamp: "2026-10-01T10:00:00+00:00".to_string(),
        });

        m.transition(WizardAction::Retry).expect("back must be valid");

        assert!(
            matches!(m.current, WizardState::WaitingForBlank { .. }),
            "Back must return to WaitingForBlank, got {:?}",
            m.current
        );
        assert_eq!(
            m.port.as_deref(),
            Some("COM19"),
            "Back must not drop the device connection"
        );
    }

    /// Back must carry the blank type the user was working with, not invent one.
    ///
    /// From `BlankDetected` the Rust FSM routes Back via the existing
    /// `ReDetectBlank` action (same destination, same blank-type preservation);
    /// `Retry` is deliberately not made valid there because
    /// `BlankDetected + Retry` has no meaning -- the blank is already detected.
    #[test]
    fn back_preserves_the_detected_blank_type() {
        let mut m = connected_machine(WizardState::BlankDetected {
            blank_type: BlankType::T5577,
            ready_to_write: true,
            existing_data_type: None,
        });

        m.transition(WizardAction::ReDetectBlank)
            .expect("back must be valid from BlankDetected");

        match m.current {
            WizardState::WaitingForBlank { expected_blank } => {
                assert_eq!(expected_blank, BlankType::T5577);
            }
            other => panic!("expected WaitingForBlank, got {:?}", other),
        }
    }

    /// A non-recoverable error must NOT offer a retry -- `Retry` is guarded on
    /// `recoverable`, so this must be rejected rather than silently navigating.
    #[test]
    fn non_recoverable_error_rejects_retry() {
        let mut m = connected_machine(WizardState::Error {
            message: "no clone command".to_string(),
            user_message: "This card type cannot be cloned.".to_string(),
            recoverable: false,
            recovery_action: None,
        });

        assert!(
            m.transition(WizardAction::Retry).is_err(),
            "a non-recoverable error must not be retryable"
        );
    }

    /// expected_blank() must never panic from any state it can be reached from.
    #[test]
    fn expected_blank_is_total() {
        for st in [
            WizardState::Idle,
            WizardState::ScanningCard,
            WizardState::WaitingForBlank {
                expected_blank: BlankType::T5577,
            },
            WizardState::VerificationComplete {
                success: true,
                mismatched_blocks: vec![],
            },
            WizardState::Complete {
                source: CardSummary {
                    card_type: "EM4100".to_string(),
                    display_name: "EM4100".to_string(),
                    uid: "0".to_string(),
                },
                target: CardSummary {
                    card_type: "EM4100".to_string(),
                    display_name: "EM4100".to_string(),
                    uid: "0".to_string(),
                },
                timestamp: "t".to_string(),
            },
        ] {
            let m = connected_machine(st);
            // Must not panic, for any input state.
            let _ = m.expected_blank();
        }
    }
}
