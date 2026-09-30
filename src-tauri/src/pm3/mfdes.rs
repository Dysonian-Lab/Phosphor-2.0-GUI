//! `hf mfdes` command construction for PM3 v4.23346 ("Frosty Lemon").
//!
//! Every argument used here was taken from the live client's own `--help`
//! output, not from the command name. The DESFire commands share a large
//! common authentication flag set, so it is composed once here rather than
//! restated per command:
//!
//! ```text
//! -t  <DES|2TDEA|3TDEA|AES>   crypto algorithm
//! -k  <8|16|24 hex bytes>     key to authenticate with
//! --kdf <none|AN10922|gallagher>
//! -i  <hex>                  KDF input (1-31 hex bytes)
//! -m  <plain|mac|encrypt>    communication mode
//! -c  <native|niso|iso>      communication command set
//! --schann <d40|ev1|ev2|lrp>  secure channel
//! --aid <3 hex bytes>         application ID
//! -n  <dec>                  key number
//! --no-auth                   skip authentication
//! ```
//!
//! Flags are emitted in a fixed order and only when set, so a given set of
//! inputs always produces the same command string.

use regex::Regex;
use std::sync::LazyLock;

static HEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Fa-f]*$").unwrap());
static DIGITS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]*$").unwrap());

/// Algorithms accepted by `-t`.
const ALGOS: &[&str] = &["DES", "2TDEA", "3TDEA", "AES"];
/// Communication modes accepted by `-m`.
const CMODES: &[&str] = &["plain", "mac", "encrypt"];
/// Command sets accepted by `-c`.
const CCSETS: &[&str] = &["native", "niso", "iso"];
/// Secure channels accepted by `--schann`.
const SCHANNS: &[&str] = &["d40", "ev1", "ev2", "lrp"];
/// Key derivation functions accepted by `--kdf`.
const KDFS: &[&str] = &["none", "AN10922", "gallagher"];
/// File types accepted by `--type`.
const FILE_TYPES: &[&str] = &["auto", "data", "value", "record", "mac"];
/// Value operations accepted by `-o` on `hf mfdes value`.
const VALUE_OPS: &[&str] = &["get", "credit", "limcredit", "debit", "clear"];

fn hex(value: &str, field: &str) -> Result<String, String> {
    let v = value.trim();
    if v.is_empty() || !HEX_RE.is_match(v) {
        return Err(format!("{} must be hex characters, got '{}'", field, value));
    }
    Ok(v.to_string())
}

fn one_of(value: &str, field: &str, allowed: &[&str]) -> Result<String, String> {
    let v = value.trim();
    if allowed.iter().any(|a| a.eq_ignore_ascii_case(v)) {
        Ok(v.to_string())
    } else {
        Err(format!(
            "{} must be one of {}, got '{}'",
            field,
            allowed.join("/"),
            value
        ))
    }
}

/// Crypto algorithm (`-t`).
pub fn algo(v: &str) -> Result<String, String> {
    one_of(v, "Algorithm (-t)", ALGOS)
}

/// Key (`-k`).
///
/// The client's help says "8|16|24 hex bytes", but `desfire_get_key_length`
/// in `desfirecrypto.c` returns the length in *bytes*: DES = 8, 2TDEA = 16,
/// 3TDEA = 24 and AES = 16. So the accepted hex string lengths are 16, 32 and
/// 48 characters, and `cmdhfmfdes.c` compares the parsed byte count against
/// the selected algorithm.
pub fn key(v: &str) -> Result<String, String> {
    let v = hex(v, "Key (-k)")?;
    match v.len() {
        16 | 32 | 48 => Ok(v),
        n => Err(format!(
            "Key (-k) must be 16, 32 or 48 hex characters \
             (8/16/24 bytes: DES, 2TDEA or 3TDEA, and AES is 16 bytes), got {}",
            n
        )),
    }
}

/// Communication mode (`-m`).
pub fn cmode(v: &str) -> Result<String, String> {
    one_of(v, "Communication mode (-m)", CMODES)
}

/// Command set (`-c`).
pub fn ccset(v: &str) -> Result<String, String> {
    one_of(v, "Command set (-c)", CCSETS)
}

/// Secure channel (`--schann`).
pub fn schann(v: &str) -> Result<String, String> {
    one_of(v, "Secure channel (--schann)", SCHANNS)
}

/// Key derivation function (`--kdf`). Note `chk` uses numeric 0/1/2 for the
/// same three choices while `detect`/`dump` use these names, so both spellings
/// are handled separately by their callers.
pub fn kdf(v: &str) -> Result<String, String> {
    one_of(v, "KDF (--kdf)", KDFS)
}

/// Key number (`-n`), decimal digits only per the client's help text.
pub fn keyno(v: &str) -> Result<String, String> {
    let v = v.trim();
    if !DIGITS_RE.is_match(v) {
        return Err(format!("Key number (-n) must be decimal digits, got '{}'", v));
    }
    Ok(v.to_string())
}

/// Application ID (`--aid`), 3 hex bytes = 6 hex characters.
pub fn aid(v: &str) -> Result<String, String> {
    let v = hex(v, "AID (--aid)")?;
    if v.len() != 6 {
        return Err(format!(
            "AID (--aid) must be 3 hex bytes (6 characters), got '{}'",
            v
        ));
    }
    Ok(v)
}

/// File ID (`--fid`), 1 hex byte = 2 hex characters.
pub fn fid(v: &str) -> Result<String, String> {
    let v = hex(v, "File ID (--fid)")?;
    if v.len() != 2 {
        return Err(format!(
            "File ID (--fid) must be 1 hex byte (2 characters), got '{}'",
            v
        ));
    }
    Ok(v)
}

/// Application ISO file ID (`--isoid`), 2 hex bytes = 4 hex characters.
pub fn isoid(v: &str) -> Result<String, String> {
    let v = hex(v, "ISO ID (--isoid)")?;
    if v.len() != 4 {
        return Err(format!(
            "ISO ID (--isoid) must be 2 hex bytes (4 characters), got '{}'",
            v
        ));
    }
    Ok(v)
}

/// File ISO ID (`--fileisoid`), 2 hex bytes = 4 hex characters.
pub fn fileisoid(v: &str) -> Result<String, String> {
    let v = hex(v, "File ISO ID (--fileisoid)")?;
    if v.len() != 4 {
        return Err(format!(
            "File ISO ID (--fileisoid) must be 2 hex bytes (4 characters), got '{}'",
            v
        ));
    }
    Ok(v)
}

/// 3-hex-byte length/offset value (`-l`, `-o` on read, `--offset` on write).
pub fn hex3(v: &str, field: &str) -> Result<String, String> {
    let v = hex(v, field)?;
    if v.len() != 6 {
        return Err(format!(
            "{} must be 3 hex bytes (6 characters), got '{}'",
            field, v
        ));
    }
    Ok(v)
}

/// Free-form hex payload (`-d`).
pub fn data(v: &str) -> Result<String, String> {
    if v.trim().is_empty() {
        return Err("Data (-d) must not be empty".to_string());
    }
    hex(v, "Data (-d)")
}

/// File type (`--type`).
pub fn filetype(v: &str) -> Result<String, String> {
    one_of(v, "File type (--type)", FILE_TYPES)
}

/// Value-file operation (`-o` on `hf mfdes value`).
pub fn valueop(v: &str) -> Result<String, String> {
    one_of(v, "Value operation (-o)", VALUE_OPS)
}

/// KDF input (`-i`), 1-31 hex bytes.
pub fn kdfi(v: &str) -> Result<String, String> {
    let v = hex(v, "KDF input (-i)")?;
    if v.len() < 2 || v.len() > 62 {
        return Err(format!(
            "KDF input (-i) must be 1-31 hex bytes (2-62 characters), got {}",
            v.len()
        ));
    }
    Ok(v)
}

/// The authentication and verbosity flags shared by most `hf mfdes`
/// subcommands. Every field is optional; only populated fields are emitted.
#[derive(Default, Clone, Debug)]
pub struct Auth {
    pub algo: Option<String>,
    pub key: Option<String>,
    pub kdf: Option<String>,
    pub kdfi: Option<String>,
    pub cmode: Option<String>,
    pub ccset: Option<String>,
    pub schann: Option<String>,
    pub aid: Option<String>,
    pub keyno: Option<String>,
    pub no_auth: bool,
    pub apdu: bool,
    pub verbose: bool,
    /// Subcommand-specific `--flag value` pairs, appended after the shared
    /// flags in the order the caller added them. Flag names are always
    /// literals in Rust; only values ever come from user input, so this
    /// cannot be used to smuggle an arbitrary option into the command.
    pub extras: Vec<(String, String)>,
    /// Subcommand-specific bare switches such as `--save` or `--ns`.
    pub switches: Vec<String>,
}

impl Auth {
    /// Adds a `--flag value` pair. Blank values are skipped so a partially
    /// filled form cannot emit a flag with an empty argument.
    pub fn flag(&mut self, name: &str, value: &str) -> &mut Self {
        if let Some(v) = opt(value) {
            self.extras.push((name.to_string(), v));
        }
        self
    }

    /// Adds a bare `--switch`.
    pub fn switch(&mut self, name: &str) -> &mut Self {
        self.switches.push(name.to_string());
        self
    }

    /// Renders the flags that appear in `allowed`, then the subcommand-specific
    /// extras whose name also appears in `allowed`, in insertion order.
    ///
    /// Not every subcommand accepts the whole shared set: `lsapp` has no
    /// `--aid`, `default` has neither `--aid` nor `--no-auth`, and
    /// `createdelegateapp` has no `-n`. Filtering here keeps the client from
    /// rejecting the command with "invalid option".
    ///
    /// The result starts with a space when non-empty, so callers can
    /// concatenate unconditionally.
    pub fn flags(&self, allowed: &[&str]) -> String {
        fn push(s: &mut String, flag: &str, val: &Option<String>) {
            if let Some(v) = val {
                s.push(' ');
                s.push_str(flag);
                s.push(' ');
                s.push_str(v);
            }
        }
        let mut s = String::new();
        if self.apdu && allowed.contains(&"-a") {
            s.push_str(" -a");
        }
        if self.verbose && allowed.contains(&"-v") {
            s.push_str(" -v");
        }
        if allowed.contains(&"-n") {
            push(&mut s, "-n", &self.keyno);
        }
        if allowed.contains(&"-t") {
            push(&mut s, "-t", &self.algo);
        }
        if allowed.contains(&"-k") {
            push(&mut s, "-k", &self.key);
        }
        if allowed.contains(&"--kdf") {
            push(&mut s, "--kdf", &self.kdf);
        }
        if allowed.contains(&"-i") {
            push(&mut s, "-i", &self.kdfi);
        }
        if allowed.contains(&"-m") {
            push(&mut s, "-m", &self.cmode);
        }
        if allowed.contains(&"-c") {
            push(&mut s, "-c", &self.ccset);
        }
        if allowed.contains(&"--schann") {
            push(&mut s, "--schann", &self.schann);
        }
        if allowed.contains(&"--aid") {
            push(&mut s, "--aid", &self.aid);
        }
        if self.no_auth && allowed.contains(&"--no-auth") {
            s.push_str(" --no-auth");
        }
        for (name, value) in &self.extras {
            if !allowed.contains(&name.as_str()) {
                continue;
            }
            s.push(' ');
            s.push_str(name);
            s.push(' ');
            s.push_str(value);
        }
        for name in &self.switches {
            if allowed.contains(&name.as_str()) {
                s.push(' ');
                s.push_str(name);
            }
        }
        s
    }
}

/// The shared authentication flags, in the order `Auth::flags` emits them.
/// `-a` and `-v` are accepted by every subcommand that has any options at all;
/// the rest vary per command and are gated by the per-subcommand allowlist.
pub const SHARED: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
];

/// Builds `hf mfdes <sub> <flags>`, emitting only the flags in `allowed`.
pub fn build(sub: &str, auth: &Auth, allowed: &[&str]) -> String {
    format!("hf mfdes {}{}", sub, auth.flags(allowed))
}

/// Builds a subcommand that takes no authentication flags at all, such as
/// `getversion`, `info`, `sim`, `pc` and `formatpicc`-style one-shots.
pub fn build_bare(sub: &str) -> String {
    format!("hf mfdes {}", sub)
}

/// Optional-string helper: blank and whitespace-only input becomes `None` so
/// the UI can submit partially filled forms without emitting empty flags.
pub fn opt(v: &str) -> Option<String> {
    let t = v.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Validates the populated fields of an `Auth` and returns it, so that a
/// malformed key or AID is reported before the device is touched.
pub fn validate_auth(a: &Auth) -> Result<(), String> {
    if let Some(v) = &a.algo {
        algo(v)?;
    }
    if let Some(v) = &a.key {
        key(v)?;
    }
    if let Some(v) = &a.kdf {
        kdf(v)?;
    }
    if let Some(v) = &a.kdfi {
        kdfi(v)?;
    }
    if let Some(v) = &a.cmode {
        cmode(v)?;
    }
    if let Some(v) = &a.ccset {
        ccset(v)?;
    }
    if let Some(v) = &a.schann {
        schann(v)?;
    }
    if let Some(v) = &a.aid {
        aid(v)?;
    }
    if let Some(v) = &a.keyno {
        keyno(v)?;
    }
    Ok(())
}

/// `hf mfdes chk` spells its KDF as `0|1|2` rather than by name, so it needs
/// its own validator. Documented mapping: 0=None, 1=AN10922, 2=Gallagher.
pub fn kdf_num(v: &str) -> Result<String, String> {
    let t = v.trim();
    match t {
        "0" | "1" | "2" => Ok(t.to_string()),
        _ => Err(format!("KDF (--kdf) for chk must be 0, 1 or 2, got '{}'", v)),
    }
}

/// Application ISO DF Name (`--dfname`): 5-16 hex bytes, so 10-32 characters.
pub fn dfname(v: &str) -> Result<String, String> {
    let v = hex(v, "DF name (--dfname)")?;
    if v.len() < 10 || v.len() > 32 || v.len() % 2 != 0 {
        return Err(format!(
            "DF name (--dfname) must be 5-16 hex bytes (10-32 characters), got '{}'",
            v
        ));
    }
    Ok(v)
}

/// Starting key for the 2-byte search pattern (`--startp2b`): 2 hex bytes.
pub fn startp2b(v: &str) -> Result<String, String> {
    let v = hex(v, "Start pattern (--startp2b)")?;
    if v.len() != 4 {
        return Err(format!(
            "Start pattern (--startp2b) must be 2 hex bytes (4 characters), got '{}'",
            v
        ));
    }
    Ok(v)
}

/// `--isofid` is a file ISO ID. Unlike `--isoid` the client documents no fixed
/// length, so accept any even-length hex value up to 4 bytes.
pub fn isofid(v: &str) -> Result<String, String> {
    let v = hex(v, "ISO file ID (--isofid)")?;
    if v.is_empty() || v.len() > 8 || v.len() % 2 != 0 {
        return Err(format!(
            "ISO file ID (--isofid) must be 1-4 hex bytes, got '{}'",
            v
        ));
    }
    Ok(v)
}

/// Access-right values for `--rrights`, `--wrights`, `--rwrights` and
/// `--chrights`: a key number `key0`..`key13`, or `free`/`deny`.
pub fn rights(v: &str, field: &str) -> Result<String, String> {
    let t = v.trim();
    let lowered = t.to_ascii_lowercase();
    if lowered == "free" || lowered == "deny" {
        return Ok(lowered);
    }
    if let Some(n) = lowered.strip_prefix("key") {
        if let Ok(idx) = n.parse::<u32>() {
            if idx <= 13 {
                return Ok(format!("key{}", idx));
            }
        }
    }
    Err(format!(
        "{} must be key0..key13, free or deny, got '{}'",
        field, v
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth() -> Auth {
        Auth::default()
    }

    #[test]
    fn empty_auth_produces_no_flags() {
        assert_eq!(build("getuid", &auth(), SHARED), "hf mfdes getuid");
    }

    #[test]
    fn shared_flags_render_in_fixed_order() {
        let a = Auth {
            keyno: Some("0".into()),
            algo: Some("2TDEA".into()),
            key: Some("000102030405060708090a0b0c0d0e0f".into()),
            kdf: Some("AN10922".into()),
            kdfi: Some("abcd".into()),
            cmode: Some("encrypt".into()),
            ccset: Some("native".into()),
            schann: Some("ev2".into()),
            aid: Some("123456".into()),
            no_auth: true,
            apdu: true,
            verbose: true,
            ..Auth::default()
        };
        assert_eq!(
            build("lsapp", &a, SHARED),
            "hf mfdes lsapp -a -v -n 0 -t 2TDEA -k 000102030405060708090a0b0c0d0e0f \
             --kdf AN10922 -i abcd -m encrypt -c native --schann ev2 --aid 123456 --no-auth"
        );
    }

    #[test]
    fn disallowed_shared_flags_are_omitted() {
        // `lsapp` has no --aid, and the client rejects the command outright
        // with "invalid option" if one is sent.
        let a = Auth {
            aid: Some("123456".into()),
            no_auth: true,
            ..Auth::default()
        };
        let lsapp: &[&str] = &[
            "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--no-auth",
        ];
        assert_eq!(build("lsapp", &a, lsapp), "hf mfdes lsapp --no-auth");
    }

    #[test]
    fn apdu_and_verbose_are_always_kept() {
        let a = Auth {
            apdu: true,
            verbose: true,
            ..Auth::default()
        };
        assert_eq!(
            build("lsapp", &a, &["-a", "-v"]),
            "hf mfdes lsapp -a -v"
        );
    }

    #[test]
    fn extras_render_after_shared_flags_then_switches() {
        let mut a = Auth {
            aid: Some("aabbcc".into()),
            ..Auth::default()
        };
        a.flag("--fid", "01");
        a.switch("--cyclic");
        a.flag("--isofid", "df01");
        assert_eq!(
            build(
                "createrecordfile",
                &a,
                &["-a", "-v", "--aid", "--fid", "--isofid", "--cyclic"]
            ),
            "hf mfdes createrecordfile --aid aabbcc --fid 01 --isofid df01 --cyclic"
        );
    }

    #[test]
    fn extras_outside_the_allowlist_are_dropped() {
        let mut a = Auth::default();
        a.flag("--fid", "01");
        a.flag("--damslot", "0a");
        assert_eq!(build("deletefile", &a, &["-a", "-v", "--aid", "--no-auth"]), "hf mfdes deletefile");
    }

    #[test]
    fn blank_extras_are_omitted() {
        let mut a = Auth::default();
        a.flag("--fid", "");
        assert_eq!(
            build("deletefile", &a, &["-a", "-v", "--aid", "--no-auth"]),
            "hf mfdes deletefile"
        );
    }

    #[test]
    fn key_length_is_validated() {
        // The client documents "8|16|24 hex bytes", but desfire_get_key_length
        // works in bytes: DES = 8, 2TDEA = 16, 3TDEA = 24, AES = 16. So the
        // accepted hex string lengths are 16, 32 and 48.
        assert!(key("0001020304050607").is_ok());
        assert!(key("000102030405060708090a0b0c0d0e0f").is_ok());
        assert!(key("000102030405060708090a0b0c0d0e0f1011121314151617").is_ok());
        assert!(key("000102030405060708090a0b").is_err());
        assert!(key("0001").is_err());
        assert!(key("zzzz").is_err());
    }

    #[test]
    fn aid_must_be_three_bytes() {
        assert_eq!(aid("123456").unwrap(), "123456");
        assert!(aid("12345").is_err());
        assert!(aid("1234567").is_err());
    }

    #[test]
    fn fid_must_be_one_byte_and_isoid_two() {
        assert_eq!(fid("01").unwrap(), "01");
        assert!(fid("0102").is_err());
        assert_eq!(isoid("df01").unwrap(), "df01");
        assert!(isoid("01").is_err());
    }

    #[test]
    fn isofid_allows_one_to_four_bytes() {
        assert_eq!(isofid("01").unwrap(), "01");
        assert_eq!(isofid("01020304").unwrap(), "01020304");
        assert!(isofid("0102030405").is_err());
        assert!(isofid("0102030").is_err());
    }

    #[test]
    fn enum_values_are_validated() {
        assert!(algo("AES").is_ok());
        assert!(algo("aes").is_ok());
        assert!(algo("3DES").is_err());
        assert!(cmode("plain").is_ok());
        assert!(cmode("nope").is_err());
        assert!(ccset("iso").is_ok());
        assert!(schann("lrp").is_ok());
        assert!(kdf("gallagher").is_ok());
        assert!(filetype("record").is_ok());
        assert!(valueop("limcredit").is_ok());
    }

    #[test]
    fn chk_kdf_uses_numeric_form() {
        assert_eq!(kdf_num("0").unwrap(), "0");
        assert_eq!(kdf_num("2").unwrap(), "2");
        assert!(kdf_num("none").is_err());
        assert!(kdf_num("3").is_err());
    }

    #[test]
    fn dfname_must_be_five_to_sixteen_bytes() {
        assert!(dfname("0011223344").is_ok());
        assert!(dfname("0011").is_err());
        assert!(dfname("00112233445566778899aabbccddeeff").is_ok());
        assert!(dfname("00112233445566778899aabbccddeeff00").is_err());
    }

    #[test]
    fn startp2b_must_be_two_bytes() {
        assert_eq!(startp2b("FA00").unwrap(), "FA00");
        assert!(startp2b("FA").is_err());
        assert!(startp2b("FA0000").is_err());
    }

    #[test]
    fn rights_accepts_keys_free_and_deny() {
        assert_eq!(rights("key0", "--rrights").unwrap(), "key0");
        assert_eq!(rights("KEY13", "--rrights").unwrap(), "key13");
        assert_eq!(rights("free", "--rrights").unwrap(), "free");
        assert_eq!(rights("deny", "--rrights").unwrap(), "deny");
        assert!(rights("key14", "--rrights").is_err());
        assert!(rights("keyx", "--rrights").is_err());
    }

    #[test]
    fn kdfi_length_is_bounded() {
        assert!(kdfi("ab").is_ok());
        assert!(kdfi("").is_err());
        assert!(kdfi(&"ab".repeat(31)).is_ok());
        assert!(kdfi(&"ab".repeat(32)).is_err());
    }

    #[test]
    fn validate_auth_rejects_bad_values() {
        let a = Auth {
            key: Some("nope".into()),
            ..Auth::default()
        };
        assert!(validate_auth(&a).is_err());
        let a = Auth {
            aid: Some("12".into()),
            ..Auth::default()
        };
        assert!(validate_auth(&a).is_err());
        let a = Auth {
            algo: Some("AES".into()),
            aid: Some("123456".into()),
            ..Auth::default()
        };
        assert!(validate_auth(&a).is_ok());
    }

    #[test]
    fn opt_treats_blank_as_absent() {
        assert_eq!(opt(""), None);
        assert_eq!(opt("   "), None);
        assert_eq!(opt(" 01 "), Some("01".to_string()));
    }

    #[test]
    fn build_bare_has_no_flags() {
        assert_eq!(build_bare("info"), "hf mfdes info");
    }
}
