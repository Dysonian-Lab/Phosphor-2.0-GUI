/// PM3 CLI command strings.
/// All commands assume the Iceman fork with `-f` flag for subprocess piping.

use crate::cards::types::{BlankType, CardType};
use regex::Regex;
use std::sync::LazyLock;

/// Validates that a string contains only hex characters.
static HEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Fa-f]+$").unwrap());
/// Validates that a string contains only alphanumeric characters and optional colons.
/// HID UIDs use format "FC65:CN29334" which contains non-hex letters.
static HEX_COLON_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9A-Za-z:]+$").unwrap());
/// Validates a T5577 password: exactly 8 hex characters.
static PASSWORD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9A-Fa-f]{8}$").unwrap());

fn validate_password(password: &str) -> Result<(), String> {
    if !PASSWORD_RE.is_match(password) {
        return Err(format!(
            "Invalid password: must be exactly 8 hex characters, got '{}'",
            password
        ));
    }
    Ok(())
}

fn validate_hex(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() || !HEX_RE.is_match(value) {
        return Err(format!(
            "Invalid {}: must be non-empty hex string, got '{}'",
            field_name, value
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_hex_or_colon(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() || !HEX_COLON_RE.is_match(value) {
        return Err(format!(
            "Invalid {}: must be hex with optional colons, got '{}'",
            field_name, value
        ));
    }
    Ok(())
}

/// Allowed HID Wiegand format strings.
const VALID_HID_FORMATS: &[&str] = &["H10301", "H10302", "H10304", "Corp1000"];

fn validate_hid_format(format: &str) -> bool {
    VALID_HID_FORMATS.contains(&format)
}

// ---------------------------------------------------------------------------
// Device / search commands
// ---------------------------------------------------------------------------

pub fn build_lf_search() -> &'static str {
    "lf search"
}

// ---------------------------------------------------------------------------
// T5577 blank management
// ---------------------------------------------------------------------------

pub fn build_t5577_detect() -> &'static str {
    "lf t55xx detect"
}

pub fn build_t5577_chk() -> &'static str {
    "lf t55xx chk"
}

pub fn build_t5577_wipe() -> &'static str {
    "lf t55xx wipe"
}

/// Wipe a T5577 that has a known password.
pub fn build_t5577_wipe_with_password(password: &str) -> Result<String, String> {
    validate_password(password)?;
    Ok(format!("lf t55xx wipe -p {}", password))
}

// ---------------------------------------------------------------------------
// EM4305 blank management
// ---------------------------------------------------------------------------

pub fn build_em4305_wipe() -> &'static str {
    "lf em 4x05 wipe"
}

/// Query EM4305 chip info to verify it's present before wiping.
/// Returns output that can be parsed by `output_parser::parse_em4305_info()`.
pub fn build_em4305_info() -> &'static str {
    "lf em 4x05 info"
}

/// Read a specific word from an EM4305 chip.
/// Used after wipe to verify word 0 is zeroed (wipe verification).
pub fn build_em4305_read_word(word: u8) -> String {
    format!("lf em 4x05 read -a {}", word)
}

/// Append `--em` flag to a base clone command for EM4305 blanks.
pub fn build_clone_for_em4305(base_cmd: &str) -> String {
    format!("{} --em", base_cmd)
}

/// Append `-p {password}` to a base clone command for password-protected T5577.
pub fn build_clone_with_password(base_cmd: &str, password: &str) -> Result<String, String> {
    validate_password(password)?;
    Ok(format!("{} -p {}", base_cmd, password))
}

// ---------------------------------------------------------------------------
// LF clone commands — original 11 types (improved)
// ---------------------------------------------------------------------------

pub fn build_em4100_clone(id: &str) -> String {
    format!("lf em 410x clone --id {}", id)
}

/// HID clone using detected Wiegand format (defaults to H10301 / 26-bit).
pub fn build_hid_clone(fc: u32, cn: u32, format: Option<&str>) -> String {
    let wiegand = format.unwrap_or("H10301");
    format!("lf hid clone -w {} --fc {} --cn {}", wiegand, fc, cn)
}

pub fn build_hid_clone_raw(raw: &str) -> String {
    format!("lf hid clone -r {}", raw)
}

pub fn build_indala_clone(raw: &str) -> String {
    format!("lf indala clone --raw {}", raw)
}

/// IO Prox clone with version number support.
pub fn build_ioprox_clone(fc: u32, cn: u32, vn: u32) -> String {
    format!("lf io clone --vn {} --fc {} --cn {}", vn, fc, cn)
}

pub fn build_ioprox_clone_raw(raw: &str) -> String {
    format!("lf io clone --raw {}", raw)
}

/// AWID clone with format support (26/34/37/50 bit).
pub fn build_awid_clone(fc: u32, cn: u32, fmt: Option<u32>) -> String {
    match fmt {
        Some(f) => format!("lf awid clone --fmt {} --fc {} --cn {}", f, fc, cn),
        None => format!("lf awid clone --fc {} --cn {}", fc, cn),
    }
}

/// FDX-B clone with country code + national ID.
pub fn build_fdxb_clone(country: u32, national_id: u64) -> String {
    format!(
        "lf fdxb clone --country {} --national {}",
        country, national_id
    )
}

pub fn build_fdxb_clone_raw(raw: &str) -> String {
    format!("lf fdxb clone --raw {}", raw)
}

/// Paradox clone with FC/CN (preferred over raw).
pub fn build_paradox_clone(fc: u32, cn: u32) -> String {
    format!("lf paradox clone --fc {} --cn {}", fc, cn)
}

pub fn build_paradox_clone_raw(raw: &str) -> String {
    format!("lf paradox clone --raw {}", raw)
}

pub fn build_viking_clone(cn: &str) -> String {
    format!("lf viking clone --cn {}", cn)
}

pub fn build_pyramid_clone(fc: u32, cn: u32) -> String {
    format!("lf pyramid clone --fc {} --cn {}", fc, cn)
}

pub fn build_pyramid_clone_raw(raw: &str) -> String {
    format!("lf pyramid clone --raw {}", raw)
}

/// Keri clone with type support: i = Internal, m = MS format.
/// PM3 expects `-t <i|m> --cn <decimal_id>`.
/// For MS format: also needs `--fc <decimal_fc>`.
pub fn build_keri_clone(cn: &str, fc: Option<&str>, keri_type: Option<&str>) -> String {
    match (keri_type, fc) {
        // MS format with FC
        (Some("m"), Some(fc)) => format!("lf keri clone -t m --fc {} --cn {}", fc, cn),
        // Internal or unspecified type
        (Some(t), _) => format!("lf keri clone -t {} --cn {}", t, cn),
        (None, _) => format!("lf keri clone --cn {}", cn),
    }
}

pub fn build_nexwatch_clone(raw: &str) -> String {
    format!("lf nexwatch clone --raw {}", raw)
}

// ---------------------------------------------------------------------------
// LF clone commands — 11 new types
// ---------------------------------------------------------------------------

/// Presco clone with hex data.
pub fn build_presco_clone_hex(hex: &str) -> String {
    format!("lf presco clone -d {}", hex)
}

/// Presco clone with site code + user code.
pub fn build_presco_clone(site_code: u32, user_code: u32) -> String {
    format!(
        "lf presco clone --sitecode {} --usercode {}",
        site_code, user_code
    )
}

/// Nedap clone with subtype + customer code + ID.
/// PM3 `lf nedap clone` uses --st (subtype), --cc (customer code), --id (ID).
pub fn build_nedap_clone(subtype: u32, customer_code: u32, id: u32) -> String {
    format!(
        "lf nedap clone --st {} --cc {} --id {}",
        subtype, customer_code, id
    )
}

/// GProxII clone with xor + format + FC + CN.
/// PM3 `lf gproxii clone` uses --xor, --fmt, --fc, --cn.
pub fn build_gproxii_clone(xor: u32, fmt: u32, fc: u32, cn: u32) -> String {
    format!(
        "lf gproxii clone --xor {} --fmt {} --fc {} --cn {}",
        xor, fmt, fc, cn
    )
}

/// Gallagher clone with region, facility, card number, issue level.
pub fn build_gallagher_clone(rc: u32, fc: u32, cn: u32, il: u32) -> String {
    format!(
        "lf gallagher clone --rc {} --fc {} --cn {} --il {}",
        rc, fc, cn, il
    )
}

/// PAC/Stanley clone with card number.
pub fn build_pac_clone(cn: &str) -> String {
    format!("lf pac clone --cn {}", cn)
}

pub fn build_pac_clone_raw(raw: &str) -> String {
    format!("lf pac clone --raw {}", raw)
}

/// Noralsy clone with card number and optional year.
/// PM3 `lf noralsy clone` uses --cn (card ID, decimal) and -y (year, optional).
pub fn build_noralsy_clone(cn: &str, year: Option<&str>) -> String {
    match year {
        Some(y) => format!("lf noralsy clone --cn {} -y {}", cn, y),
        None => format!("lf noralsy clone --cn {}", cn),
    }
}

/// Jablotron clone with hex card number.
pub fn build_jablotron_clone(cn: &str) -> String {
    format!("lf jablotron clone --cn {}", cn)
}

/// SecuraKey clone (raw only).
pub fn build_securakey_clone(raw: &str) -> String {
    format!("lf securakey clone --raw {}", raw)
}

/// Visa2000 clone with card number.
pub fn build_visa2000_clone(cn: u32) -> String {
    format!("lf visa2000 clone --cn {}", cn)
}

/// Motorola clone (raw only).
pub fn build_motorola_clone(raw: &str) -> String {
    format!("lf motorola clone --raw {}", raw)
}

/// IDTECK clone (raw only).
pub fn build_idteck_clone(raw: &str) -> String {
    format!("lf idteck clone --raw {}", raw)
}

// ---------------------------------------------------------------------------
// Build clone command dispatcher
// ---------------------------------------------------------------------------

/// Build the appropriate clone command for a given card type + data.
/// Returns None if clone is not supported for this type or if input validation fails.
pub fn build_clone_command(
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
) -> Option<String> {
    // Validate uid: must be hex with optional colons (no spaces, semicolons, or other injection vectors)
    if !HEX_COLON_RE.is_match(uid) {
        return None;
    }

    match card_type {
        CardType::EM4100 => Some(build_em4100_clone(uid)),

        CardType::HIDProx => {
            // Prefer raw clone — exact bit copy, no re-encoding.
            // Structured clone (Wiegand format) can fail on PM3 Easy (weaker antenna).
            if let Some(raw) = decoded
                .get("raw")
                .filter(|raw| validate_hex(raw, "raw").is_ok())
            {
                return Some(build_hid_clone_raw(raw));
            }
            // Fallback to structured clone when raw not available
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    let fmt = decoded
                        .get("format")
                        .map(|s| s.as_str())
                        .filter(|f| validate_hid_format(f));
                    return Some(build_hid_clone(fc_n, cn_n, fmt));
                }
            }
            None
        }

        CardType::Indala => {
            // Prefer raw hex from parser (avoids using decimal UID as --raw)
            let raw = decoded.get("raw").map(|s| s.as_str()).unwrap_or(uid);
            Some(build_indala_clone(raw))
        }

        CardType::IOProx => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                let vn = decoded
                    .get("version")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_ioprox_clone(fc_n, cn_n, vn));
                }
            }
            // uid already validated at top
            Some(build_ioprox_clone_raw(uid))
        }

        CardType::AWID => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    let fmt = decoded.get("format").and_then(|f| f.parse::<u32>().ok());
                    return Some(build_awid_clone(fc_n, cn_n, fmt));
                }
            }
            // No raw fallback — awid clone requires --fc and --cn flags
            None
        }

        CardType::FDX_B => {
            if let (Some(country), Some(national)) =
                (decoded.get("country"), decoded.get("national_id"))
            {
                if let (Ok(cc), Ok(nid)) = (country.parse::<u32>(), national.parse::<u64>()) {
                    return Some(build_fdxb_clone(cc, nid));
                }
            }
            // Fallback to raw (validated) then uid (already validated at top)
            if let Some(raw) = decoded
                .get("raw")
                .filter(|r| validate_hex(r, "raw").is_ok())
            {
                Some(build_fdxb_clone_raw(raw))
            } else {
                Some(build_fdxb_clone_raw(uid))
            }
        }

        CardType::Paradox => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_paradox_clone(fc_n, cn_n));
                }
            }
            // uid already validated at top
            Some(build_paradox_clone_raw(uid))
        }

        CardType::Viking => Some(build_viking_clone(uid)),

        CardType::Pyramid => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_pyramid_clone(fc_n, cn_n));
                }
            }
            // Raw fallback — parser stores raw hex in decoded["raw"]
            decoded
                .get("raw")
                .filter(|r| validate_hex(r, "raw").is_ok())
                .map(|raw| build_pyramid_clone_raw(raw))
        }

        CardType::Keri => {
            // Use card_number (decimal ID from parser), fallback to uid
            let cn = decoded
                .get("card_number")
                .map(|s| s.as_str())
                .unwrap_or(uid);
            let fc = decoded.get("facility_code").map(|s| s.as_str());
            let keri_type = decoded
                .get("keri_type")
                .map(|s| s.as_str())
                .filter(|t| *t == "i" || *t == "m");
            Some(build_keri_clone(cn, fc, keri_type))
        }

        CardType::NexWatch => Some(build_nexwatch_clone(uid)),

        // --- New 11 types ---

        CardType::Presco => {
            if let (Some(sc), Some(uc)) =
                (decoded.get("site_code"), decoded.get("user_code"))
            {
                if let (Ok(sc_n), Ok(uc_n)) = (sc.parse::<u32>(), uc.parse::<u32>()) {
                    return Some(build_presco_clone(sc_n, uc_n));
                }
            }
            // uid already validated at top (hex with optional colons)
            Some(build_presco_clone_hex(uid))
        }

        CardType::Nedap => {
            if let (Some(st), Some(cc), Some(id)) = (
                decoded.get("subtype"),
                decoded.get("customer_code"),
                decoded.get("card_number"),
            ) {
                if let (Ok(st_n), Ok(cc_n), Ok(id_n)) =
                    (st.parse::<u32>(), cc.parse::<u32>(), id.parse::<u32>())
                {
                    return Some(build_nedap_clone(st_n, cc_n, id_n));
                }
            }
            // No raw fallback — nedap clone requires --st, --cc and --id flags
            None
        }

        CardType::GProxII => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                let xor = decoded
                    .get("xor")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                let fmt = decoded
                    .get("format")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(26);
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_gproxii_clone(xor, fmt, fc_n, cn_n));
                }
            }
            // No raw fallback — gproxii clone requires --xor, --fmt, --fc and --cn flags
            None
        }

        CardType::Gallagher => {
            if let (Some(rc), Some(fc), Some(cn), Some(il)) = (
                decoded.get("region_code"),
                decoded.get("facility_code"),
                decoded.get("card_number"),
                decoded.get("issue_level"),
            ) {
                if let (Ok(rc_n), Ok(fc_n), Ok(cn_n), Ok(il_n)) = (
                    rc.parse::<u32>(),
                    fc.parse::<u32>(),
                    cn.parse::<u32>(),
                    il.parse::<u32>(),
                ) {
                    return Some(build_gallagher_clone(rc_n, fc_n, cn_n, il_n));
                }
            }
            // No raw fallback — gallagher clone requires --rc, --fc, --cn, --il flags
            None
        }

        CardType::PAC => {
            // Validate raw hex before using
            if let Some(raw) = decoded
                .get("raw")
                .filter(|r| validate_hex(r, "raw").is_ok())
            {
                return Some(build_pac_clone_raw(raw));
            }
            if let Some(cn) = decoded.get("card_number") {
                return Some(build_pac_clone(cn));
            }
            Some(build_pac_clone(uid))
        }

        CardType::Noralsy => {
            let cn = decoded
                .get("card_number")
                .map(|s| s.as_str())
                .unwrap_or(uid);
            let year = decoded.get("year").map(|s| s.as_str());
            Some(build_noralsy_clone(cn, year))
        }

        CardType::Jablotron => {
            if let Some(cn) = decoded.get("card_number") {
                // card_number for Jablotron is hex
                if validate_hex(cn, "card_number").is_ok() {
                    return Some(build_jablotron_clone(cn));
                }
            }
            Some(build_jablotron_clone(uid))
        }

        CardType::SecuraKey => {
            if let Some(raw) = decoded
                .get("raw")
                .filter(|r| validate_hex(r, "raw").is_ok())
            {
                Some(build_securakey_clone(raw))
            } else {
                Some(build_securakey_clone(uid))
            }
        }

        CardType::Visa2000 => {
            if let Some(cn) = decoded.get("card_number") {
                if let Ok(cn_n) = cn.parse::<u32>() {
                    return Some(build_visa2000_clone(cn_n));
                }
            }
            // No raw fallback — visa2000 clone requires numeric --cn flag
            None
        }

        CardType::Motorola => {
            if let Some(raw) = decoded
                .get("raw")
                .filter(|r| validate_hex(r, "raw").is_ok())
            {
                Some(build_motorola_clone(raw))
            } else {
                Some(build_motorola_clone(uid))
            }
        }

        CardType::IDTECK => {
            if let Some(raw) = decoded
                .get("raw")
                .filter(|r| validate_hex(r, "raw").is_ok())
            {
                Some(build_idteck_clone(raw))
            } else {
                Some(build_idteck_clone(uid))
            }
        }

        // Non-cloneable LF types
        CardType::COTAG | CardType::EM4x50 | CardType::Hitag => None,

        // HF cloning not yet implemented in this module
        CardType::MifareClassic1K
        | CardType::MifareClassic4K
        | CardType::MifareUltralight
        | CardType::NTAG
        | CardType::DESFire
        | CardType::IClass => None,
    }
}

// ---------------------------------------------------------------------------
// HF scan / info commands
// ---------------------------------------------------------------------------

pub fn build_hf_search() -> &'static str {
    "hf search"
}

pub fn build_hf_14a_info() -> &'static str {
    "hf 14a info"
}

pub fn build_hf_mf_info() -> &'static str {
    "hf mf info"
}

pub fn build_hf_mfu_info() -> &'static str {
    "hf mfu info"
}

pub fn build_hf_iclass_info() -> &'static str {
    "hf iclass info"
}

pub fn build_hf_mfdes_info() -> &'static str {
    "hf mfdes info"
}

// ---------------------------------------------------------------------------
// HF MFDES v4.23346 new commands (eload/esave/eview/dump/view/etest/sim/chk/detect)
// ---------------------------------------------------------------------------

/// `hf mfdes eload` — load dump file into MFDES emulator.
pub fn build_mfdes_eload(dump_path: &str) -> String {
    format!("hf mfdes eload -f {}", dump_path)
}

/// `hf mfdes esave` — save emulator state to dump file.
pub fn build_mfdes_esave(dump_path: &str) -> String {
    format!("hf mfdes esave -f {}", dump_path)
}

/// `hf mfdes eview` — view emulator state.
pub fn build_mfdes_eview() -> &'static str {
    "hf mfdes eview"
}

/// `hf mfdes dump` — dump MFDES tag.
pub fn build_mfdes_dump() -> &'static str {
    "hf mfdes dump"
}

/// `hf mfdes view` — view MFDES tag data.
pub fn build_mfdes_view() -> &'static str {
    "hf mfdes view"
}

/// `hf mfdes etest` — test MFDES emulator.
pub fn build_mfdes_etest() -> &'static str {
    "hf mfdes etest"
}

/// `hf mfdes sim` — simulate MFDES tag from dump.
pub fn build_mfdes_sim(dump_path: &str) -> String {
    format!("hf mfdes sim -f {}", dump_path)
}

/// `hf mfdes chk` — check MFDES keys (v4.23346: no-arg).
pub fn build_mfdes_chk() -> &'static str {
    "hf mfdes chk"
}

/// `hf mfdes detect` — detect MFDES tag (v4.23346: no-arg).
pub fn build_mfdes_detect() -> &'static str {
    "hf mfdes detect"
}

// ---------------------------------------------------------------------------
// HF MF view v4.23346 (RKF / VIGIK / HID PACS decode)
// ---------------------------------------------------------------------------

/// `hf mf view -f <file>` — decode a MIFARE Classic dump file (RKF/VIGIK/HID PACS).
pub fn build_hf_mf_view(file: &str) -> String {
    format!("hf mf view -f {}", file)
}

/// `hf mf view --selftest` — run the dump parser self tests.
pub fn build_hf_mf_view_selftest() -> &'static str {
    "hf mf view --selftest"
}

// ---------------------------------------------------------------------------
// HF 14b view v4.23346 (MyKey / COGES decode on SRIX4K)
// ---------------------------------------------------------------------------

/// `hf 14b view -f <file>` — decode a SRIX4K dump (MyKey/COGES).
pub fn build_hf_14b_view(file: &str) -> String {
    format!("hf 14b view -f {}", file)
}

// ---------------------------------------------------------------------------
// Smart card v4.23346 (PPS — ISO 7816-3 protocol parameter selection)
// ---------------------------------------------------------------------------

/// `smart pps` — negotiate PPS with the card (RDV4 smartcard module required).
pub fn build_smart_pps() -> &'static str {
    "smart pps"
}

/// `smart pps --t0` — select T=0 protocol.
pub fn build_smart_pps_t0() -> &'static str {
    "smart pps --t0"
}

/// `smart pps --t1` — select T=1 protocol.
pub fn build_smart_pps_t1() -> &'static str {
    "smart pps --t1"
}

/// `smart pps --ta1 <hex>` — negotiate TA1 byte.
pub fn build_smart_pps_ta1(ta1: &str) -> String {
    format!("smart pps --ta1 {}", ta1)
}

// ---------------------------------------------------------------------------
// HF eMRTD v4.23346 (PACE-CAM passport reading)
// ---------------------------------------------------------------------------

/// `hf emrtd info` — tag information (offline with --dir).
pub fn build_hf_emrtd_info() -> &'static str {
    "hf emrtd info"
}

/// `hf emrtd dump` — dump eMRTD files.
pub fn build_hf_emrtd_dump() -> &'static str {
    "hf emrtd dump"
}

/// `hf emrtd list` — list ISO 14443A/7816 history.
pub fn build_hf_emrtd_list() -> &'static str {
    "hf emrtd list"
}

/// `hf emrtd test` — offline regression tests for PACE / secure messaging.
pub fn build_hf_emrtd_test() -> &'static str {
    "hf emrtd test"
}

// ---------------------------------------------------------------------------
// Trace v4.23346
// ---------------------------------------------------------------------------

/// `trace clear` — clear the client-side trace buffer.
pub fn build_trace_clear() -> &'static str {
    "trace clear"
}

// ---------------------------------------------------------------------------
// HF 14a antifuzz v4.23346
// ---------------------------------------------------------------------------

/// `hf 14a antifuzz` — fuzz the ISO14443a anticollision phase.
pub fn build_hf_14a_antifuzz() -> &'static str {
    "hf 14a antifuzz"
}

/// `hf 14a antifuzz --coll` — collision storm mode.
pub fn build_hf_14a_antifuzz_coll() -> &'static str {
    "hf 14a antifuzz --coll"
}

// ---------------------------------------------------------------------------
// LF T55xx v4.23346 additional commands
// ---------------------------------------------------------------------------

/// `lf t55xx set config` — configure T55xx tag parameters.
pub fn build_lf_t55xx_set_config() -> &'static str {
    "lf t55xx set config"
}

/// `lf t55xx chk pwds` — check T55xx passwords.
pub fn build_lf_t55xx_chk_pwds() -> &'static str {
    "lf t55xx chk pwds"
}

/// `lf t55xx dangerraw` — raw T55xx danger mode.
pub fn build_lf_t55xx_dangerraw() -> &'static str {
    "lf t55xx dangerraw"
}

/// `lf t55xx wakeup` — wake up T55xx tag.
pub fn build_lf_t55xx_wakeup() -> &'static str {
    "lf t55xx wakeup"
}

// ---------------------------------------------------------------------------
// HF MFU v4.23346 new commands (ndefwrite/ndefformat/chk)
// ---------------------------------------------------------------------------

/// `hf mfu chk` — check MIFARE Ultralight keys (v4.23346).
pub fn build_mfu_chk() -> &'static str {
    "hf mfu chk"
}

/// `hf mfu ndefwrite` — write NDEF records to card.
pub fn build_mfu_ndefwrite(ndef_data: &str) -> String {
    format!("hf mfu ndefwrite -d {}", ndef_data)
}

/// `hf mfu ndefformat` — format card as NFC tag.
pub fn build_mfuf_ndefformat() -> &'static str {
    "hf mfu ndefformat"
}

// ---------------------------------------------------------------------------
// HF MF chk (v4.23346: cchk + aeschk merged into chk)
// ---------------------------------------------------------------------------

/// `hf mf chk` — check MIFARE Classic keys (v4.23346: replaces cchk + aeschk).
pub fn build_hf_mf_chk() -> &'static str {
    "hf mf chk"
}

// ---------------------------------------------------------------------------
// HF 14b v4.23346 new commands (ctrdbl/rdbl/view --selftest)
// ---------------------------------------------------------------------------

/// `hf 14b rdbl` — read SRI512/SRIX4 block.
pub fn build_hf_14b_rdbl(block: u16) -> String {
    format!("hf 14b rdbl -b {}", block)
}

/// `hf 14b ctrdbl` — read ASK CTS/C-ticket block.
pub fn build_hf_14b_ctrdbl(block: u16) -> String {
    format!("hf 14b ctrdbl -b {}", block)
}

/// `hf 14b view --selftest` — self-test mode.
pub fn build_hf_14b_view_selftest() -> &'static str {
    "hf 14b view --selftest"
}

// ---------------------------------------------------------------------------
// HF Calypso v4.23346 new commands (info/dump/list)
// ---------------------------------------------------------------------------

/// `hf calypso info` — Calypso tag info.
pub fn build_hf_calypso_info() -> &'static str {
    "hf calypso info"
}

/// `hf calypso dump` — dump Calypso tag.
pub fn build_hf_calypso_dump() -> &'static str {
    "hf calypso dump"
}

/// `hf calypso list` — list Calypso applications.
pub fn build_hf_calypso_list() -> &'static str {
    "hf calypso list"
}

// ---------------------------------------------------------------------------
// HF Felica v4.23346 new command (sim)
// ---------------------------------------------------------------------------

/// `hf felica sim` — emulate FeliCa from dump file.
pub fn build_hf_felica_sim(dump_path: &str) -> String {
    format!("hf felica sim -f {}", dump_path)
}

// ---------------------------------------------------------------------------
// HF iCLASS v4.23346 new command (legbrute)
// ---------------------------------------------------------------------------

/// `hf iclass legbrute` — LEGIC brute-force key recovery (NEON/AVX2/AVX-512).
pub fn build_hf_iclass_legbrute() -> &'static str {
    "hf iclass legbrute"
}

// ---------------------------------------------------------------------------
// HF Thinfilm v4.23346 new command (sniff)
// ---------------------------------------------------------------------------

/// `hf thinfilm sniff` — sniff thin-film tags.
pub fn build_hf_thinfilm_sniff() -> &'static str {
    "hf thinfilm sniff"
}

// ---------------------------------------------------------------------------
// MAD v4.23346 new commands (read/write/verify/decode/encode)
// ---------------------------------------------------------------------------

/// `mad read` — read MAD data.
pub fn build_mad_read() -> &'static str {
    "mad read"
}

/// `mad write` — write MAD data.
pub fn build_mad_write(data: &str) -> String {
    format!("mad write -d {}", data)
}

/// `mad verify` — verify MAD data.
pub fn build_mad_verify() -> &'static str {
    "mad verify"
}

/// `mad decode` — decode MAD data.
pub fn build_mad_decode() -> &'static str {
    "mad decode"
}

/// `mad encode` — encode MAD data.
pub fn build_mad_encode(data: &str) -> String {
    format!("mad encode -d {}", data)
}

// ---------------------------------------------------------------------------
// NFC v4.23346 new command (encode)
// ---------------------------------------------------------------------------

/// `nfc encode` — encode NFC data.
pub fn build_nfc_encode(data: &str) -> String {
    format!("nfc encode -d {}", data)
}

// ---------------------------------------------------------------------------
// LF Trovan v4.23346 new command
// ---------------------------------------------------------------------------

/// `lf trovan` — Trovan tag operations.
pub fn build_lf_trovan() -> &'static str {
    "lf trovan"
}

// ---------------------------------------------------------------------------
// HF autopwn (MIFARE Classic key recovery + dump)
// ---------------------------------------------------------------------------

/// Build `hf mf autopwn` command. Uses `--4k` flag for 4K cards.
pub fn build_hf_autopwn(card_type: &CardType) -> String {
    match card_type {
        CardType::MifareClassic4K => "hf mf autopwn --4k".to_string(),
        _ => "hf mf autopwn".to_string(),
    }
}

// ---------------------------------------------------------------------------
// HF clone write commands
// ---------------------------------------------------------------------------

/// Gen1a: load full dump via magic wakeup (40/43) backdoor.
pub fn build_mf_cload(dump_path: &str) -> String {
    format!("hf mf cload -f {}", dump_path)
}

/// Gen2/CUID: force 14a config to allow block 0 write.
/// Must call `build_mf_gen2_config_reset()` after writing.
pub fn build_mf_gen2_config_force() -> &'static str {
    "hf 14a config --atqa force --bcc ignore --cl2 skip --rats skip"
}

/// Gen2/CUID: reset 14a config to standard after block 0 write.
pub fn build_mf_gen2_config_reset() -> &'static str {
    "hf 14a config --std"
}

/// Gen2/CUID: force-write block 0 with given key and data.
/// `key`: 12 hex chars (e.g., "FFFFFFFFFFFF"), `data`: 32 hex chars.
pub fn build_mf_wrbl0(key: &str, data: &str) -> String {
    format!("hf mf wrbl --blk 0 -k {} -d {} --force", key, data)
}

/// Gen2/Gen3: restore all blocks from a binary dump file.
pub fn build_mf_restore(dump_path: &str) -> String {
    format!("hf mf restore -f {}", dump_path)
}

/// Gen3: set UID via APDU command. `uid`: 8 or 14 hex chars (no spaces).
pub fn build_mf_gen3uid(uid: &str) -> String {
    format!("hf mf gen3uid --uid {}", uid)
}

/// Gen3: write block 0 via APDU command. `block0`: 32 hex chars.
pub fn build_mf_gen3blk(block0: &str) -> String {
    format!("hf mf gen3blk {}", block0)
}

/// Gen4 GTU/UMC: load full dump via gload.
pub fn build_mf_gload(dump_path: &str) -> String {
    format!("hf mf gload -f {}", dump_path)
}

/// Gen4 GDM: write a single block. `blk`: 0-255, `data`: 32 hex chars.
pub fn build_mf_gdm_setblk(blk: u16, data: &str) -> String {
    format!("hf mf gdmsetblk --blk {} -d {}", blk, data)
}

/// UL/NTAG: restore dump from file. `-s` = special pages, `-e` = engineering mode.
pub fn build_mfu_restore(dump_path: &str) -> String {
    format!("hf mfu restore -f {} -s -e", dump_path)
}

/// iCLASS: restore dump from file using default key (key index 0).
/// Writes blocks 6-18 (application data, skips header and config blocks).
pub fn build_iclass_restore(dump_path: &str) -> String {
    format!("hf iclass restore -f {} --first 6 --last 18 --ki 0", dump_path)
}

// ---------------------------------------------------------------------------
// HF dump commands (no key recovery needed)
// ---------------------------------------------------------------------------

/// UL/NTAG: dump card memory to binary file.
pub fn build_mfu_dump() -> &'static str {
    "hf mfu dump"
}

/// iCLASS: dump card memory using leaked master key (key index 0).
pub fn build_iclass_dump() -> &'static str {
    "hf iclass dump --ki 0"
}

// ---------------------------------------------------------------------------
// HF data check commands (blank detection — existing data check)
// ---------------------------------------------------------------------------

/// Gen1a: read single block via magic wakeup backdoor. No keys needed.
/// `blk`: block number (0-63 for 1K, 0-255 for 4K).
pub fn build_mf_cgetblk(blk: u16) -> String {
    format!("hf mf cgetblk --blk {}", blk)
}

/// Read single block with specified key. Returns hex data if key is valid.
/// `blk`: block number, `key`: 12 hex chars (e.g., "FFFFFFFFFFFF").
pub fn build_mf_rdbl(blk: u16, key: &str) -> String {
    format!("hf mf rdbl --blk {} -k {}", blk, key)
}

// ---------------------------------------------------------------------------
// HF verification readback commands
// ---------------------------------------------------------------------------

/// Gen1a: read all blocks via magic wakeup (40/43) backdoor. No keys needed.
pub fn build_mf_cview() -> &'static str {
    "hf mf cview"
}

/// MIFARE Classic: dump all blocks using recovered keys.
/// Auto-discovers `hf-mf-<UID>-key.bin` in working directory.
pub fn build_mf_dump() -> &'static str {
    "hf mf dump"
}

// ---------------------------------------------------------------------------
// Wipe commands
// ---------------------------------------------------------------------------

/// Determine the wipe command based on blank type.
/// Returns `None` for unsupported blank types or invalid passwords.
pub fn build_wipe_command(blank_type: &BlankType, password: Option<&str>) -> Option<String> {
    match blank_type {
        BlankType::EM4305 => Some(build_em4305_wipe().to_string()),
        BlankType::T5577 => match password {
            Some(pw) => Some(build_t5577_wipe_with_password(pw).ok()?),
            None => Some(build_t5577_wipe().to_string()),
        },
        // Other blank types don't have a wipe command in this module
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- HF info commands (static strings) --

    #[test]
    fn hf_search_cmd() {
        assert_eq!(build_hf_search(), "hf search");
    }

    #[test]
    fn hf_14a_info_cmd() {
        assert_eq!(build_hf_14a_info(), "hf 14a info");
    }

    #[test]
    fn hf_mf_info_cmd() {
        assert_eq!(build_hf_mf_info(), "hf mf info");
    }

    #[test]
    fn hf_mfu_info_cmd() {
        assert_eq!(build_hf_mfu_info(), "hf mfu info");
    }

    #[test]
    fn hf_iclass_info_cmd() {
        assert_eq!(build_hf_iclass_info(), "hf iclass info");
    }

    #[test]
    fn hf_mfdes_info_cmd() {
        assert_eq!(build_hf_mfdes_info(), "hf mfdes info");
    }

    // -- HF autopwn --

    #[test]
    fn hf_autopwn_classic_1k() {
        let cmd = build_hf_autopwn(&CardType::MifareClassic1K);
        assert_eq!(cmd, "hf mf autopwn");
    }

    #[test]
    fn hf_autopwn_classic_4k() {
        let cmd = build_hf_autopwn(&CardType::MifareClassic4K);
        assert_eq!(cmd, "hf mf autopwn --4k");
    }

    #[test]
    fn hf_autopwn_other_type_defaults_1k() {
        // Non-Classic types still get basic autopwn (no --4k)
        let cmd = build_hf_autopwn(&CardType::MifareUltralight);
        assert_eq!(cmd, "hf mf autopwn");
    }

    // -- Gen1a clone --

    #[test]
    fn mf_cload_cmd() {
        let cmd = build_mf_cload("hf-mf-01020304-dump.bin");
        assert_eq!(cmd, "hf mf cload -f hf-mf-01020304-dump.bin");
    }

    // -- Gen2/CUID clone --

    #[test]
    fn mf_gen2_config_force_cmd() {
        assert_eq!(
            build_mf_gen2_config_force(),
            "hf 14a config --atqa force --bcc ignore --cl2 skip --rats skip"
        );
    }

    #[test]
    fn mf_gen2_config_reset_cmd() {
        assert_eq!(build_mf_gen2_config_reset(), "hf 14a config --std");
    }

    #[test]
    fn mf_wrbl0_cmd() {
        let cmd = build_mf_wrbl0("FFFFFFFFFFFF", "0102030404080400000000000000BEEF");
        assert_eq!(
            cmd,
            "hf mf wrbl --blk 0 -k FFFFFFFFFFFF -d 0102030404080400000000000000BEEF --force"
        );
    }

    #[test]
    fn mf_restore_cmd() {
        let cmd = build_mf_restore("hf-mf-AABBCCDD-dump.bin");
        assert_eq!(cmd, "hf mf restore -f hf-mf-AABBCCDD-dump.bin");
    }

    // -- Gen3 clone --

    #[test]
    fn mf_gen3uid_4byte() {
        let cmd = build_mf_gen3uid("01020304");
        assert_eq!(cmd, "hf mf gen3uid --uid 01020304");
    }

    #[test]
    fn mf_gen3uid_7byte() {
        let cmd = build_mf_gen3uid("01020304050607");
        assert_eq!(cmd, "hf mf gen3uid --uid 01020304050607");
    }

    #[test]
    fn mf_gen3blk_cmd() {
        let cmd = build_mf_gen3blk("0102030404080400000000000000BEEF");
        assert_eq!(cmd, "hf mf gen3blk 0102030404080400000000000000BEEF");
    }

    // -- Gen4 GTU clone --

    #[test]
    fn mf_gload_cmd() {
        let cmd = build_mf_gload("hf-mf-01020304-dump.bin");
        assert_eq!(cmd, "hf mf gload -f hf-mf-01020304-dump.bin");
    }

    // -- Gen4 GDM clone --

    #[test]
    fn mf_gdm_setblk_block0() {
        let cmd = build_mf_gdm_setblk(0, "0102030404080400000000000000BEEF");
        assert_eq!(
            cmd,
            "hf mf gdmsetblk --blk 0 -d 0102030404080400000000000000BEEF"
        );
    }

    #[test]
    fn mf_gdm_setblk_block63() {
        let cmd = build_mf_gdm_setblk(63, "FFFFFFFFFFFF08778F00FFFFFFFFFFFF");
        assert_eq!(
            cmd,
            "hf mf gdmsetblk --blk 63 -d FFFFFFFFFFFF08778F00FFFFFFFFFFFF"
        );
    }

    #[test]
    fn mf_gdm_setblk_4k_block255() {
        let cmd = build_mf_gdm_setblk(255, "DEADBEEF" .repeat(4).as_str());
        assert!(cmd.starts_with("hf mf gdmsetblk --blk 255 -d "));
    }

    // -- UL/NTAG clone --

    #[test]
    fn mfu_restore_cmd() {
        let cmd = build_mfu_restore("hf-mfu-04112233445566-dump.bin");
        assert_eq!(
            cmd,
            "hf mfu restore -f hf-mfu-04112233445566-dump.bin -s -e"
        );
    }

    // -- iCLASS clone --

    #[test]
    fn iclass_restore_cmd() {
        let cmd = build_iclass_restore("hf-iclass-dump.json");
        assert_eq!(
            cmd,
            "hf iclass restore -f hf-iclass-dump.json --first 6 --last 18 --ki 0"
        );
    }

    // -- Dump commands --

    #[test]
    fn mfu_dump_cmd() {
        assert_eq!(build_mfu_dump(), "hf mfu dump");
    }

    #[test]
    fn iclass_dump_cmd() {
        assert_eq!(build_iclass_dump(), "hf iclass dump --ki 0");
    }

    // -- Verification commands --

    #[test]
    fn mf_cview_cmd() {
        assert_eq!(build_mf_cview(), "hf mf cview");
    }

    #[test]
    fn mf_dump_cmd() {
        assert_eq!(build_mf_dump(), "hf mf dump");
    }

    // -- v4.23346 new commands --

    #[test]
    fn mfdes_eload_cmd() {
        assert_eq!(
            build_mfdes_eload("mfdes-dump.bin"),
            "hf mfdes eload -f mfdes-dump.bin"
        );
    }

    #[test]
    fn mfdes_esave_cmd() {
        assert_eq!(
            build_mfdes_esave("mfdes-dump.bin"),
            "hf mfdes esave -f mfdes-dump.bin"
        );
    }

    #[test]
    fn mfdes_eview_cmd() {
        assert_eq!(build_mfdes_eview(), "hf mfdes eview");
    }

    #[test]
    fn mfdes_dump_cmd() {
        assert_eq!(build_mfdes_dump(), "hf mfdes dump");
    }

    #[test]
    fn mfdes_view_cmd() {
        assert_eq!(build_mfdes_view(), "hf mfdes view");
    }

    #[test]
    fn mfdes_etest_cmd() {
        assert_eq!(build_mfdes_etest(), "hf mfdes etest");
    }

    #[test]
    fn mfdes_sim_cmd() {
        assert_eq!(
            build_mfdes_sim("mfdes-dump.bin"),
            "hf mfdes sim -f mfdes-dump.bin"
        );
    }

    #[test]
    fn mfdes_chk_cmd() {
        assert_eq!(build_mfdes_chk(), "hf mfdes chk");
    }

    #[test]
    fn mfdes_detect_cmd() {
        assert_eq!(build_mfdes_detect(), "hf mfdes detect");
    }

    #[test]
    fn mfu_chk_cmd() {
        assert_eq!(build_mfu_chk(), "hf mfu chk");
    }

    #[test]
    fn mfu_ndefwrite_cmd() {
        assert_eq!(
            build_mfu_ndefwrite("01020304"),
            "hf mfu ndefwrite -d 01020304"
        );
    }

    #[test]
    fn mfu_ndefformat_cmd() {
        assert_eq!(build_mfuf_ndefformat(), "hf mfu ndefformat");
    }

    #[test]
    fn hf_mf_chk_cmd() {
        assert_eq!(build_hf_mf_chk(), "hf mf chk");
    }

    #[test]
    fn hf_14b_rdbl_cmd() {
        assert_eq!(build_hf_14b_rdbl(6), "hf 14b rdbl -b 6");
    }

    #[test]
    fn hf_14b_ctrdbl_cmd() {
        assert_eq!(build_hf_14b_ctrdbl(15), "hf 14b ctrdbl -b 15");
    }

    #[test]
    fn hf_14b_view_selftest_cmd() {
        assert_eq!(build_hf_14b_view_selftest(), "hf 14b view --selftest");
    }

    #[test]
    fn hf_calypso_info_cmd() {
        assert_eq!(build_hf_calypso_info(), "hf calypso info");
    }

    #[test]
    fn hf_calypso_dump_cmd() {
        assert_eq!(build_hf_calypso_dump(), "hf calypso dump");
    }

    #[test]
    fn hf_calypso_list_cmd() {
        assert_eq!(build_hf_calypso_list(), "hf calypso list");
    }

    #[test]
    fn hf_felica_sim_cmd() {
        assert_eq!(
            build_hf_felica_sim("felica-dump.bin"),
            "hf felica sim -f felica-dump.bin"
        );
    }

    #[test]
    fn hf_iclass_legbrute_cmd() {
        assert_eq!(build_hf_iclass_legbrute(), "hf iclass legbrute");
    }

    #[test]
    fn hf_thinfilm_sniff_cmd() {
        assert_eq!(build_hf_thinfilm_sniff(), "hf thinfilm sniff");
    }

    #[test]
    fn mad_read_cmd() {
        assert_eq!(build_mad_read(), "mad read");
    }

    #[test]
    fn mad_write_cmd() {
        assert_eq!(build_mad_write("DEADBEEF"), "mad write -d DEADBEEF");
    }

    #[test]
    fn mad_verify_cmd() {
        assert_eq!(build_mad_verify(), "mad verify");
    }

    #[test]
    fn mad_decode_cmd() {
        assert_eq!(build_mad_decode(), "mad decode");
    }

    #[test]
    fn mad_encode_cmd() {
        assert_eq!(build_mad_encode("DEADBEEF"), "mad encode -d DEADBEEF");
    }

    #[test]
    fn nfc_encode_cmd() {
        assert_eq!(build_nfc_encode("01020304"), "nfc encode -d 01020304");
    }

    #[test]
    fn lf_trovan_cmd() {
        assert_eq!(build_lf_trovan(), "lf trovan");
    }

    #[test]
    fn hf_mf_view_cmd() {
        assert_eq!(
            build_hf_mf_view("hf-mf-01020304-dump.bin"),
            "hf mf view -f hf-mf-01020304-dump.bin"
        );
    }

    #[test]
    fn hf_mf_view_selftest_cmd() {
        assert_eq!(build_hf_mf_view_selftest(), "hf mf view --selftest");
    }

    #[test]
    fn hf_14b_view_cmd() {
        assert_eq!(
            build_hf_14b_view("hf-14b-D0021F673CB26556-dump.json"),
            "hf 14b view -f hf-14b-D0021F673CB26556-dump.json"
        );
    }

    #[test]
    fn smart_pps_cmd() {
        assert_eq!(build_smart_pps(), "smart pps");
    }

    #[test]
    fn smart_pps_t0_cmd() {
        assert_eq!(build_smart_pps_t0(), "smart pps --t0");
    }

    #[test]
    fn smart_pps_t1_cmd() {
        assert_eq!(build_smart_pps_t1(), "smart pps --t1");
    }

    #[test]
    fn smart_pps_ta1_cmd() {
        assert_eq!(build_smart_pps_ta1("93"), "smart pps --ta1 93");
    }

    #[test]
    fn hf_emrtd_info_cmd() {
        assert_eq!(build_hf_emrtd_info(), "hf emrtd info");
    }

    #[test]
    fn hf_emrtd_dump_cmd() {
        assert_eq!(build_hf_emrtd_dump(), "hf emrtd dump");
    }

    #[test]
    fn hf_emrtd_list_cmd() {
        assert_eq!(build_hf_emrtd_list(), "hf emrtd list");
    }

    #[test]
    fn hf_emrtd_test_cmd() {
        assert_eq!(build_hf_emrtd_test(), "hf emrtd test");
    }

    #[test]
    fn trace_clear_cmd() {
        assert_eq!(build_trace_clear(), "trace clear");
    }

    #[test]
    fn hf_14a_antifuzz_cmd() {
        assert_eq!(build_hf_14a_antifuzz(), "hf 14a antifuzz");
    }

    #[test]
    fn hf_14a_antifuzz_coll_cmd() {
        assert_eq!(build_hf_14a_antifuzz_coll(), "hf 14a antifuzz --coll");
    }

    #[test]
    fn lf_t55xx_set_config_cmd() {
        assert_eq!(build_lf_t55xx_set_config(), "lf t55xx set config");
    }

    #[test]
    fn lf_t55xx_chk_pwds_cmd() {
        assert_eq!(build_lf_t55xx_chk_pwds(), "lf t55xx chk pwds");
    }

    #[test]
    fn lf_t55xx_dangerraw_cmd() {
        assert_eq!(build_lf_t55xx_dangerraw(), "lf t55xx dangerraw");
    }

    #[test]
    fn lf_t55xx_wakeup_cmd() {
        assert_eq!(build_lf_t55xx_wakeup(), "lf t55xx wakeup");
    }
}
