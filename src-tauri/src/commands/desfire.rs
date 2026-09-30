use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::pm3::connection;
use crate::pm3::mfdes;
use crate::state::{WizardMachine, WizardState};

/// Flag names usable as subcommand-specific extras. Only names in this list can
/// ever reach the command string, and each is validated by [`extras::validate`],
/// so a caller cannot inject an arbitrary option.
mod extras {
    pub const ISOID: &str = "--isoid";
    pub const DFNAME: &str = "--dfname";
    pub const FID: &str = "--fid";
    pub const ISOFID: &str = "--isofid";
    pub const AMODE: &str = "--amode";
    pub const RAWDATA: &str = "--rawdata";
    pub const RAWRIGHTS: &str = "--rawrights";
    pub const RRIGHTS: &str = "--rrights";
    pub const WRIGHTS: &str = "--wrights";
    pub const RWIGHTS: &str = "--rwrights";
    pub const CHRIGHTS: &str = "--chrights";
    pub const DAMSLOT: &str = "--damslot";
    pub const START: &str = "--start";
    pub const END: &str = "--end";
    pub const STEP: &str = "--step";
    pub const DATA: &str = "-d";
    pub const SAVE: &str = "--save";
    pub const CYCLIC: &str = "--cyclic";
    pub const BACKUP: &str = "--backup";
    pub const KA: &str = "--ka";
    pub const KB: &str = "--kb";
    pub const ACCESS_CONDITIONS: &str = "--access-conditions";
    pub const RESTORE_TRANSFER: &str = "--restore-transfer";

    /// Validates `value` according to what `flag` is documented to accept.
    pub fn validate(flag: &str, value: &str) -> Result<String, String> {
        use super::mfdes;
        match flag {
            ISOID => mfdes::isoid(value),
            DFNAME => mfdes::dfname(value),
            FID => mfdes::fid(value),
            ISOFID => mfdes::isofid(value),
            AMODE => mfdes::cmode(value),
            RAWDATA | RAWRIGHTS => mfdes::data(value),
            RRIGHTS | WRIGHTS | RWIGHTS | CHRIGHTS => {
                mfdes::rights(value, &format!("{} value", flag))
            }
            DAMSLOT | START | END | DATA => mfdes::data(value),
            STEP => mfdes::keyno(value),
            _ => Err(format!("Unsupported DESFire option '{}'", flag)),
        }
    }
}

/// Every flag each subcommand accepts, read off the complete v4.23346 usage
/// blocks rather than the first line.
///
/// This matters: the client rejects a command outright with `invalid option` if
/// it is handed a flag the subcommand does not declare, so the shared set is
/// *not* uniform. `lsapp` has no `--aid`, `default` has neither `--aid` nor
/// `--no-auth`, and `createdelegateapp` has no `-n`. `-a` and `-v` appear
/// everywhere as the `[-hav]` cluster.
const FULL_SHARED: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
];

/// Shared flags minus `--aid`, used by `lsapp`, `freemem`, `getaids`,
/// `getappnames`, `getdelegateappinfo` and `brutedamslot`.
const NO_AID: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--no-auth",
];

/// Shared flags with `--aid` but without `--no-auth`.
const AID_ONLY: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid",
];

/// `default`: no `--aid` and no `--no-auth`.
const NEITHER: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann",
];

/// `createdelegateapp`: no `-n`.
const NO_KEYNO: &[&str] = &[
    "-a", "-v", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
];

/// `selectisofid` only declares `[-hav]`, `--aid`, `--dfname` and `--isofid`.
const SELECT_ISOFID: &[&str] = &["-a", "-v", "--aid"];

/// Which extras each subcommand accepts alongside its shared flags.
const SUB_EXTRAS: &[(&str, &[&str])] = &[
    ("auth", &[extras::ISOID, extras::DFNAME, extras::SAVE]),
    (
        "chfilesettings",
        &[
            extras::ISOID,
            extras::FID,
            extras::RAWDATA,
            extras::AMODE,
            extras::RAWRIGHTS,
            extras::RRIGHTS,
            extras::WRIGHTS,
            extras::RWIGHTS,
            extras::CHRIGHTS,
        ],
    ),
    ("chkeysettings", &[extras::DATA]),
    ("clearrecfile", &[extras::ISOID, extras::FID]),
    ("getdelegateappinfo", &[extras::DAMSLOT]),
    ("getfileisoids", &[extras::ISOID, extras::DFNAME]),
    (
        "getfilesettings",
        &[extras::ISOID, extras::DFNAME, extras::FID],
    ),
    (
        "brutedamslot",
        &[extras::START, extras::END, extras::STEP],
    ),
    (
        "createmacfile",
        &[
            extras::ISOID,
            extras::FID,
            extras::AMODE,
            extras::RAWRIGHTS,
            extras::RRIGHTS,
            extras::WRIGHTS,
            extras::RWIGHTS,
            extras::CHRIGHTS,
        ],
    ),
    (
        "createvaluefile",
        &[
            extras::FID,
            extras::AMODE,
            extras::RAWRIGHTS,
            extras::RRIGHTS,
            extras::WRIGHTS,
            extras::RWIGHTS,
            extras::CHRIGHTS,
        ],
    ),
    (
        "createrecordfile",
        &[
            extras::FID,
            extras::ISOFID,
            extras::AMODE,
            extras::RAWRIGHTS,
            extras::RRIGHTS,
            extras::WRIGHTS,
            extras::RWIGHTS,
            extras::CHRIGHTS,
        ],
    ),
    (
        "createmfcmapping",
        &[
            extras::ISOID,
            extras::FID,
            extras::KA,
            extras::KB,
            extras::ACCESS_CONDITIONS,
            extras::RESTORE_TRANSFER,
        ],
    ),
    // `selectisofid` spells it `--isofid`, unlike every other subcommand.
    ("selectisofid", &[extras::ISOFID, extras::DFNAME]),
];

/// Shared-flag set for each subcommand, matching the usage blocks exactly.
const SUB_SHARED: &[(&str, &[&str])] = &[
    ("auth", AID_ONLY),
    ("chfilesettings", FULL_SHARED),
    ("chkeysettings", AID_ONLY),
    ("clearrecfile", FULL_SHARED),
    ("createdelegateapp", NO_KEYNO),
    ("default", NEITHER),
    ("deleteapp", AID_ONLY),
    ("deletefile", FULL_SHARED),
    ("formatpicc", AID_ONLY),
    ("freemem", NO_AID),
    ("getaids", NO_AID),
    ("getappnames", NO_AID),
    ("getdelegateappinfo", NO_AID),
    ("getfileids", FULL_SHARED),
    ("getfileisoids", FULL_SHARED),
    ("getfilesettings", FULL_SHARED),
    ("getkeysettings", AID_ONLY),
    ("getkeyversions", FULL_SHARED),
    ("getuid", AID_ONLY),
    ("lsapp", NO_AID),
    ("lsfiles", FULL_SHARED),
    ("mad", AID_ONLY),
    ("selectapp", AID_ONLY),
    ("selectisofid", SELECT_ISOFID),
    ("setconfig", AID_ONLY),
    ("brutedamslot", NO_AID),
    ("createmacfile", FULL_SHARED),
    ("createvaluefile", FULL_SHARED),
    ("createrecordfile", FULL_SHARED),
    ("createmfcmapping", AID_ONLY),
];

/// Subcommands that accept no options at all.
const BARE: &[&str] = &["getversion", "info", "sim", "eview", "test", "help"];

/// Allowlists for the subcommands that have dedicated handlers.
const DETECT: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--isoid",
    "--dfname", "-f", "--save",
];
const DUMP: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--isoid", "--dfname", "-l", "-f", "--keys", "--ns",
];
const READ: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--fid", "--isoid", "--dfname", "--fileisoid", "--type", "-o", "--isochain",
];
const WRITE: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--fid", "--isoid", "--fileisoid", "--type", "-o", "-d", "--debit", "--commit",
    "--updaterec", "--readerid", "--trkey",
];
const VALUE: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--fid", "--isoid", "-o", "-d",
];
const CREATEAPP: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--rawdata", "--fid", "--dfname", "--dfhex", "--ks1", "--ks2", "--dstalgo", "--numkeys",
];
const CREATEFILE: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--fid", "--isofid", "--rawtype", "--rawdata", "--amode", "--rawrights", "--rrights",
    "--wrights", "--rwrights", "--chrights", "--size", "--backup",
];
const CHANGEKEY: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--isoid",
    "--oldalgo", "--oldkey", "--newkeyno", "--newalgo", "--newkey", "--newver",
];
const VERIFYCERT: &[&str] = &[
    "-a", "-v", "-n", "-t", "-k", "--kdf", "-i", "-m", "-c", "--schann", "--aid", "--no-auth",
    "--isoid", "--dfname", "--fid", "--ca", "--keyaid", "--keyisoid", "--keydfname", "--keyidx",
    "--validatemethod", "--readmethod",
];

fn get_port(machine: &State<'_, Mutex<WizardMachine>>) -> Result<String, AppError> {
    let m = machine.lock().map_err(|e| {
        AppError::CommandFailed(format!("State lock poisoned: {}", e))
    })?;

    if let WizardState::DeviceConnected { port, .. } = &m.current {
        return Ok(port.clone());
    }

    if let Some(port) = &m.port {
        return Ok(port.clone());
    }

    Err(AppError::InvalidTransition("No device connected".to_string()))
}

/// Every optional extra value, in the canonical order used for emission.
#[derive(Default, Clone)]
struct ExtraInputs {
    isoid: Option<String>,
    dfname: Option<String>,
    fid: Option<String>,
    isofid: Option<String>,
    amode: Option<String>,
    rawdata: Option<String>,
    rawrights: Option<String>,
    rrights: Option<String>,
    wrights: Option<String>,
    rwrights: Option<String>,
    chrights: Option<String>,
    damslot: Option<String>,
    start: Option<String>,
    end: Option<String>,
    step: Option<String>,
    data: Option<String>,
}

impl ExtraInputs {
    /// Pairs this subcommand's accepted extras with the supplied values, in a
    /// fixed order so identical input always yields an identical command.
    fn pairs(&self, allowed: &[&str]) -> Vec<(&'static str, String)> {
        let table: [(&'static str, &Option<String>); 15] = [
            (extras::ISOID, &self.isoid),
            (extras::DFNAME, &self.dfname),
            (extras::FID, &self.fid),
            (extras::ISOFID, &self.isofid),
            (extras::AMODE, &self.amode),
            (extras::RAWDATA, &self.rawdata),
            (extras::RAWRIGHTS, &self.rawrights),
            (extras::RRIGHTS, &self.rrights),
            (extras::WRIGHTS, &self.wrights),
            (extras::RWIGHTS, &self.rwrights),
            (extras::CHRIGHTS, &self.chrights),
            (extras::DAMSLOT, &self.damslot),
            (extras::START, &self.start),
            (extras::END, &self.end),
            (extras::STEP, &self.step),
        ];
        table
            .iter()
            .filter(|(name, _)| allowed.contains(name))
            .filter_map(|(name, val)| {
                val.as_deref()
                    .and_then(mfdes::opt)
                    .map(|v| (*name, v))
            })
            .collect()
    }
}

/// Builds an `Auth` from optional string inputs, dropping blanks so a
/// partially filled form does not emit empty flags.
fn auth_from(
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> mfdes::Auth {
    mfdes::Auth {
        algo: algo.as_deref().and_then(mfdes::opt),
        key: key.as_deref().and_then(mfdes::opt),
        kdf: kdf.as_deref().and_then(mfdes::opt),
        kdfi: kdfi.as_deref().and_then(mfdes::opt),
        cmode: cmode.as_deref().and_then(mfdes::opt),
        ccset: ccset.as_deref().and_then(mfdes::opt),
        schann: schann.as_deref().and_then(mfdes::opt),
        aid: aid.as_deref().and_then(mfdes::opt),
        keyno: keyno.as_deref().and_then(mfdes::opt),
        no_auth,
        apdu,
        verbose,
        extras: Vec::new(),
        switches: Vec::new(),
    }
}

/// Resolves the complete allowlist for `sub`: its shared flags plus its extras.
fn allowed_for(sub: &str) -> Result<Vec<&'static str>, AppError> {
    let shared = SUB_SHARED
        .iter()
        .find(|(name, _)| *name == sub)
        .map(|(_, list)| *list);
    let extra = SUB_EXTRAS
        .iter()
        .find(|(name, _)| *name == sub)
        .map(|(_, list)| *list);
    match shared {
        Some(shared) => {
            let mut all = shared.to_vec();
            if let Some(extra) = extra {
                all.extend_from_slice(extra);
            }
            Ok(all)
        }
        None => Err(AppError::CommandFailed(format!(
            "Unsupported DESFire subcommand '{}'. Use desfire_bare, desfire_chk, \
             desfire_dump, desfire_read, desfire_write, desfire_value, \
             desfire_file, desfire_etest, desfire_list, desfire_pc, \
             desfire_createapp, desfire_createfile, desfire_changekey, \
             desfire_bruteaid, desfire_bruteisofid, desfire_cert, \
             desfire_verifycert or desfire_makelicense instead.",
            sub
        ))),
    }
}

fn unknown_sub(sub: &str, list: &[&str]) -> AppError {
    AppError::CommandFailed(format!(
        "Unsupported DESFire subcommand '{}'. Expected one of: {}",
        sub,
        list.join(", ")
    ))
}

/// Applies the accepted extras and switches to `a`, validating each value.
fn apply_extras(
    a: &mut mfdes::Auth,
    allowed: &[&str],
    inputs: &ExtraInputs,
    switches: &[(&str, bool)],
) -> Result<(), AppError> {
    for (name, value) in inputs.pairs(allowed) {
        let checked = extras::validate(name, &value).map_err(AppError::CommandFailed)?;
        a.flag(name, &checked);
    }
    for (name, on) in switches {
        if *on && allowed.contains(name) {
            a.switch(name);
        }
    }
    Ok(())
}

/// Run an `hf mfdes` subcommand that uses the shared authentication flags plus
/// a verified set of subcommand-specific extras.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_auth(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    sub: String,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    fid: Option<String>,
    isofid: Option<String>,
    amode: Option<String>,
    rawdata: Option<String>,
    rawrights: Option<String>,
    rrights: Option<String>,
    wrights: Option<String>,
    rwrights: Option<String>,
    chrights: Option<String>,
    damslot: Option<String>,
    start: Option<String>,
    end: Option<String>,
    step: Option<String>,
    data: Option<String>,
    save: bool,
    cyclic: bool,
    backup: bool,
    ka: bool,
    kb: bool,
    access_conditions: bool,
    restore_transfer: bool,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let sub = sub.trim().to_string();
    let allowed = allowed_for(&sub)?;
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    let inputs = ExtraInputs {
        isoid,
        dfname,
        fid,
        isofid,
        amode,
        rawdata,
        rawrights,
        rrights,
        wrights,
        rwrights,
        chrights,
        damslot,
        start,
        end,
        step,
        data,
    };
    apply_extras(
        &mut a,
        &allowed,
        &inputs,
        &[
            (extras::SAVE, save),
            (extras::CYCLIC, cyclic),
            (extras::BACKUP, backup),
            (extras::KA, ka),
            (extras::KB, kb),
            (extras::ACCESS_CONDITIONS, access_conditions),
            (extras::RESTORE_TRANSFER, restore_transfer),
        ],
    )?;
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build(&sub, &a, &allowed);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// Run an `hf mfdes` subcommand that takes no options.
#[tauri::command]
pub async fn desfire_bare(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    sub: String,
) -> Result<String, AppError> {
    let sub = sub.trim().to_string();
    if !BARE.contains(&sub.as_str()) {
        return Err(unknown_sub(&sub, BARE));
    }
    let cmd = mfdes::build_bare(&sub);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes detect` â€” detect the PICC master key, key type, communication
/// mode, command set and channel mode. `--save` records details for `default`.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_detect(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    file: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
    save: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        a.flag("--dfname", &mfdes::dfname(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = file.as_deref().and_then(mfdes::opt) {
        a.flag("-f", &v);
    }
    if save {
        a.switch("--save");
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("detect", &a, DETECT);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes chk` â€” check keys against the card. This subcommand spells its KDF
/// as `0|1|2` rather than by name and has no `-t`/`-m`/`-c`, so it gets its own
/// builder.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_chk(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    aid: Option<String>,
    key: Option<String>,
    file: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    schann: Option<String>,
    pattern1b: bool,
    pattern2b: bool,
    startp2b: Option<String>,
    json: Option<String>,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut cmd = String::from("hf mfdes chk");
    if apdu {
        cmd.push_str(" -a");
    }
    if verbose {
        cmd.push_str(" -v");
    }
    if let Some(v) = aid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --aid {}",
            mfdes::aid(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = key.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(" -k {}", mfdes::key(&v).map_err(AppError::CommandFailed)?));
    }
    if let Some(v) = file.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(" -f {}", v));
    }
    if let Some(v) = kdf.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --kdf {}",
            mfdes::kdf_num(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = kdfi.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " -i {}",
            mfdes::kdfi(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = schann.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --schann {}",
            mfdes::schann(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if pattern1b {
        cmd.push_str(" --pattern1b");
    }
    if pattern2b {
        cmd.push_str(" --pattern2b");
    }
    if let Some(v) = startp2b.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --startp2b {}",
            mfdes::startp2b(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = json.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(" -j {}", v));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes dump` â€” dump a card to a JSON file, walking every application and
/// file unless one is named.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_dump(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    length: Option<String>,
    file: Option<String>,
    keys: Option<String>,
    no_auth: bool,
    no_save: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        a.flag("--dfname", &mfdes::dfname(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = length.as_deref().and_then(mfdes::opt) {
        a.flag(
            "-l",
            &mfdes::hex3(&v, "Length (-l)").map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = file.as_deref().and_then(mfdes::opt) {
        a.flag("-f", &v);
    }
    if let Some(v) = keys.as_deref().and_then(mfdes::opt) {
        a.flag("--keys", &v);
    }
    if no_save {
        a.switch("--ns");
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("dump", &a, DUMP);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes read` â€” read data from a file on the card.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_read(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    fid: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    fileisoid: Option<String>,
    ftype: Option<String>,
    offset: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = fid.as_deref().and_then(mfdes::opt) {
        a.flag("--fid", &mfdes::fid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        a.flag("--dfname", &mfdes::dfname(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = fileisoid.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--fileisoid",
            &mfdes::fileisoid(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = ftype.as_deref().and_then(mfdes::opt) {
        a.flag("--type", &mfdes::filetype(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = offset.as_deref().and_then(mfdes::opt) {
        a.flag("-o", &mfdes::hex3(&v, "Offset (-o)").map_err(AppError::CommandFailed)?);
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("read", &a, READ);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes write` â€” write data to a file on the card.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_write(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    fid: Option<String>,
    isoid: Option<String>,
    fileisoid: Option<String>,
    ftype: Option<String>,
    offset: Option<String>,
    data: Option<String>,
    readerid: Option<String>,
    trkey: Option<String>,
    updaterec: Option<String>,
    debit: bool,
    commit: bool,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = fid.as_deref().and_then(mfdes::opt) {
        a.flag("--fid", &mfdes::fid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = fileisoid.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--fileisoid",
            &mfdes::fileisoid(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = ftype.as_deref().and_then(mfdes::opt) {
        a.flag("--type", &mfdes::filetype(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = offset.as_deref().and_then(mfdes::opt) {
        a.flag("-o", &mfdes::hex3(&v, "Offset (-o)").map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = data.as_deref().and_then(mfdes::opt) {
        a.flag("-d", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = readerid.as_deref().and_then(mfdes::opt) {
        a.flag("--readerid", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = trkey.as_deref().and_then(mfdes::opt) {
        a.flag("--trkey", &mfdes::key(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = updaterec.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--updaterec",
            &mfdes::keyno(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if debit {
        a.switch("--debit");
    }
    if commit {
        a.switch("--commit");
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("write", &a, WRITE);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes value` â€” get, credit, limit, debit or clear a value file.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_value(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    fid: Option<String>,
    isoid: Option<String>,
    operation: Option<String>,
    data: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = fid.as_deref().and_then(mfdes::opt) {
        a.flag("--fid", &mfdes::fid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = operation.as_deref().and_then(mfdes::opt) {
        a.flag("-o", &mfdes::valueop(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = data.as_deref().and_then(mfdes::opt) {
        a.flag("-d", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("value", &a, VALUE);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes eload` / `esave` / `view` â€” emulator image load, save and print.
#[tauri::command]
pub async fn desfire_file(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    sub: String,
    file: Option<String>,
    keep: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let sub = sub.trim().to_string();
    if !["eload", "esave", "view"].contains(&sub.as_str()) {
        return Err(unknown_sub(&sub, &["eload", "esave", "view"]));
    }
    // `view` requires a filename; `eload` requires one too. `esave` defaults
    // to the loaded UID, so the filename is optional there.
    let require_file = sub != "esave";
    let file = file.as_deref().and_then(mfdes::opt);
    if require_file && file.is_none() {
        return Err(AppError::CommandFailed(format!(
            "hf mfdes {} requires a dump filename",
            sub
        )));
    }
    let mut cmd = format!("hf mfdes {}", sub);
    if verbose {
        cmd.push_str(" -v");
    }
    if let Some(v) = file {
        cmd.push_str(&format!(" -f {}", v));
    }
    if keep && sub == "esave" {
        cmd.push_str(" --keep");
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes etest` â€” drive the DESFire simulation from the host over USB.
/// Exactly one action is allowed per call, matching the client's own help.
#[tauri::command]
pub async fn desfire_etest(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    action: String,
    apdu: Option<String>,
    random: Option<String>,
    json: bool,
) -> Result<String, AppError> {
    let action = action.trim().to_ascii_lowercase();
    let mut cmd = String::from("hf mfdes etest");
    match action.as_str() {
        "begin" | "end" | "scan" | "fieldoff" | "state" => {
            cmd.push_str(&format!(" --{}", action));
        }
        "apdu" => {
            let v = apdu
                .as_deref()
                .and_then(mfdes::opt)
                .ok_or_else(|| AppError::CommandFailed("etest --apdu needs a value".into()))?;
            cmd.push_str(&format!(
                " --apdu {}",
                mfdes::data(&v).map_err(AppError::CommandFailed)?
            ));
        }
        "random" => {
            let v = random
                .as_deref()
                .and_then(mfdes::opt)
                .ok_or_else(|| AppError::CommandFailed("etest --random needs a value".into()))?;
            cmd.push_str(&format!(
                " --random {}",
                mfdes::data(&v).map_err(AppError::CommandFailed)?
            ));
        }
        _ => {
            return Err(unknown_sub(
                &action,
                &["begin", "end", "scan", "fieldoff", "state", "apdu", "random"],
            ))
        }
    }
    if json {
        cmd.push_str(" -j");
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes list` â€” annotated trace listing for DESFire traffic. This is an
/// alias of `trace list -t des -c`, so its flags are the trace ones.
#[tauri::command]
pub async fn desfire_list(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    buffer: bool,
    frame: bool,
    crc: bool,
    relative: bool,
    microseconds: bool,
    pcap: bool,
    file: Option<String>,
) -> Result<String, AppError> {
    let mut cmd = String::from("hf mfdes list");
    if buffer {
        cmd.push_str(" -1");
    }
    if frame {
        cmd.push_str(" --frame");
    }
    if crc {
        cmd.push_str(" -c");
    }
    if relative {
        cmd.push_str(" -r");
    }
    if microseconds {
        cmd.push_str(" -u");
    }
    if pcap {
        cmd.push_str(" -x");
    }
    if let Some(v) = file.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(" -f {}", v));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes pc` â€” PICC card configuration. `-k` is required here.
#[tauri::command]
pub async fn desfire_pc(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    key: String,
    retry: Option<String>,
    ccset: Option<String>,
    aid: Option<String>,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut cmd = String::from("hf mfdes pc");
    if apdu {
        cmd.push_str(" -a");
    }
    if verbose {
        cmd.push_str(" -v");
    }
    cmd.push_str(&format!(
        " -k {}",
        mfdes::key(&key).map_err(AppError::CommandFailed)?
    ));
    if let Some(v) = retry.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " -r {}",
            mfdes::keyno(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = ccset.as_deref().and_then(mfdes::opt) {
        if v != "native" && v != "niso" {
            return Err(AppError::CommandFailed(format!(
                "pc -c must be native or niso, got '{}'",
                v
            )));
        }
        cmd.push_str(&format!(" -c {}", v));
    }
    if let Some(v) = aid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --aid {}",
            mfdes::aid(&v).map_err(AppError::CommandFailed)?
        ));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes createapp` â€” create an application. Note its `--dfname` is a
/// plain string here, unlike the hex DF name used by `dump`/`read`/`detect`.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_createapp(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    rawdata: Option<String>,
    fid: Option<String>,
    dfname: Option<String>,
    dfhex: Option<String>,
    ks1: Option<String>,
    ks2: Option<String>,
    dstalgo: Option<String>,
    numkeys: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = rawdata.as_deref().and_then(mfdes::opt) {
        a.flag("--rawdata", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = fid.as_deref().and_then(mfdes::opt) {
        a.flag("--fid", &mfdes::fid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        a.flag("--dfname", &name_str(&v)?);
    }
    if let Some(v) = dfhex.as_deref().and_then(mfdes::opt) {
        a.flag("--dfhex", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = ks1.as_deref().and_then(mfdes::opt) {
        a.flag("--ks1", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = ks2.as_deref().and_then(mfdes::opt) {
        a.flag("--ks2", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = dstalgo.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--dstalgo",
            &mfdes::algo(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = numkeys.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--numkeys",
            &mfdes::keyno(&v).map_err(AppError::CommandFailed)?,
        );
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("createapp", &a, CREATEAPP);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes createfile` â€” create a standard or backup file in an application.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_createfile(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    fid: Option<String>,
    isofid: Option<String>,
    rawtype: Option<String>,
    rawdata: Option<String>,
    amode: Option<String>,
    rawrights: Option<String>,
    rrights: Option<String>,
    wrights: Option<String>,
    rwrights: Option<String>,
    chrights: Option<String>,
    size: Option<String>,
    backup: bool,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = fid.as_deref().and_then(mfdes::opt) {
        a.flag("--fid", &mfdes::fid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = isofid.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--isofid",
            &mfdes::isofid(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = rawtype.as_deref().and_then(mfdes::opt) {
        a.flag("--rawtype", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = rawdata.as_deref().and_then(mfdes::opt) {
        a.flag("--rawdata", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = amode.as_deref().and_then(mfdes::opt) {
        a.flag("--amode", &mfdes::cmode(&v).map_err(AppError::CommandFailed)?);
    }
    for (name, val) in [
        ("--rawrights", &rawrights),
        ("--rrights", &rrights),
        ("--wrights", &wrights),
        ("--rwrights", &rwrights),
        ("--chrights", &chrights),
    ] {
        if let Some(v) = val.as_deref().and_then(mfdes::opt) {
            a.flag(
                name,
                &mfdes::rights(&v, name).map_err(AppError::CommandFailed)?,
            );
        }
    }
    if let Some(v) = size.as_deref().and_then(mfdes::opt) {
        a.flag("--size", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    if backup {
        a.switch("--backup");
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("createfile", &a, CREATEFILE);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes changekey` â€” change a PICC or application key.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_changekey(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    isoid: Option<String>,
    oldalgo: Option<String>,
    oldkey: Option<String>,
    newkeyno: Option<String>,
    newalgo: Option<String>,
    newkey: Option<String>,
    newver: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = oldalgo.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--oldalgo",
            &mfdes::algo(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = oldkey.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--oldkey",
            &mfdes::key(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = newkeyno.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--newkeyno",
            &mfdes::keyno(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = newalgo.as_deref().and_then(mfdes::opt) {
        a.flag(
            "--newalgo",
            &mfdes::algo(&v).map_err(AppError::CommandFailed)?,
        );
    }
    if let Some(v) = newkey.as_deref().and_then(mfdes::opt) {
        a.flag("--newkey", &mfdes::key(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = newver.as_deref().and_then(mfdes::opt) {
        a.flag("--newver", &mfdes::data(&v).map_err(AppError::CommandFailed)?);
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let cmd = mfdes::build("changekey", &a, CHANGEKEY);
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes bruteaid` â€” search the AID space. Presets replace `--start`/
/// `--end` when one is chosen.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_bruteaid(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    start: Option<String>,
    end: Option<String>,
    info: Option<String>,
    preset: Option<String>,
) -> Result<String, AppError> {
    const PRESETS: &[&str] = &["full", "ascii", "numbers", "letters", "dictionary", "mad"];
    let mut cmd = String::from("hf mfdes bruteaid");
    if let Some(v) = start.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --start {}",
            mfdes::hex3(&v, "Start (--start)").map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = end.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --end {}",
            mfdes::hex3(&v, "End (--end)").map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = info.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " -i {}",
            mfdes::keyno(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = preset.as_deref().and_then(mfdes::opt) {
        let matched = PRESETS
            .iter()
            .find(|p| p.eq_ignore_ascii_case(&v))
            .ok_or_else(|| {
                AppError::CommandFailed(format!(
                    "Preset must be one of {}, got '{}'",
                    PRESETS.join("/"),
                    v
                ))
            })?;
        cmd.push_str(&format!(" --preset {}", matched));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes bruteisofid` â€” search the ISO file ID space of an application.
#[tauri::command]
pub async fn desfire_bruteisofid(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    aid: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    start: Option<String>,
    end: Option<String>,
    step: Option<String>,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut cmd = String::from("hf mfdes bruteisofid");
    if apdu {
        cmd.push_str(" -a");
    }
    if verbose {
        cmd.push_str(" -v");
    }
    if let Some(v) = aid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --aid {}",
            mfdes::aid(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --isoid {}",
            mfdes::isoid(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --dfname {}",
            mfdes::dfname(&v).map_err(AppError::CommandFailed)?
        ));
    }
    for (flag, val) in [("--start", &start), ("--end", &end)] {
        if let Some(v) = val.as_deref().and_then(mfdes::opt) {
            cmd.push_str(&format!(
                " {} {}",
                flag,
                mfdes::isofid(&v).map_err(AppError::CommandFailed)?
            ));
        }
    }
    if let Some(v) = step.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --step {}",
            mfdes::keyno(&v).map_err(AppError::CommandFailed)?
        ));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes vdesign` / `intauth` â€” DESFire Light virtual card design and
/// internal authentication. Both take a certificate blob and a key source.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_cert(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    sub: String,
    data: Option<String>,
    source: Option<String>,
    aid: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    keyno: Option<String>,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let sub = sub.trim().to_string();
    if !["vdesign", "intauth"].contains(&sub.as_str()) {
        return Err(unknown_sub(&sub, &["vdesign", "intauth"]));
    }
    const SOURCES: &[&str] = &["hex", "pem", "der", "path"];
    let mut cmd = format!("hf mfdes {}", sub);
    if apdu {
        cmd.push_str(" -a");
    }
    if verbose {
        cmd.push_str(" -v");
    }
    if let Some(v) = data.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(" -d {}", mfdes::data(&v).map_err(AppError::CommandFailed)?));
    }
    if let Some(v) = source.as_deref().and_then(mfdes::opt) {
        let matched = SOURCES
            .iter()
            .find(|s| s.eq_ignore_ascii_case(&v))
            .ok_or_else(|| {
                AppError::CommandFailed(format!(
                    "Key source (-p) must be one of {}, got '{}'",
                    SOURCES.join("/"),
                    v
                ))
            })?;
        cmd.push_str(&format!(" -p {}", matched));
    }
    if let Some(v) = aid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --aid {}",
            mfdes::aid(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --isoid {}",
            mfdes::isoid(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --dfname {}",
            mfdes::dfname(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if sub == "intauth" {
        if let Some(v) = keyno.as_deref().and_then(mfdes::opt) {
            cmd.push_str(&format!(
                " -n {}",
                mfdes::keyno(&v).map_err(AppError::CommandFailed)?
            ));
        }
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes verifycert` â€” verify a certificate chain. Note this subcommand
/// only accepts `--schann ev2`.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn desfire_verifycert(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    profile: Option<String>,
    algo: Option<String>,
    key: Option<String>,
    kdf: Option<String>,
    kdfi: Option<String>,
    cmode: Option<String>,
    ccset: Option<String>,
    schann: Option<String>,
    aid: Option<String>,
    keyno: Option<String>,
    isoid: Option<String>,
    dfname: Option<String>,
    fid: Option<String>,
    ca: Option<String>,
    keyaid: Option<String>,
    keyisoid: Option<String>,
    keydfname: Option<String>,
    keyidx: Option<String>,
    validatemethod: Option<String>,
    readmethod: Option<String>,
    no_auth: bool,
    apdu: bool,
    verbose: bool,
) -> Result<String, AppError> {
    const VALIDATE: &[&str] = &["auto", "intauth", "vde", "skip"];
    const READ: &[&str] = &["auto", "desfire"];
    let mut a = auth_from(
        algo, key, kdf, kdfi, cmode, ccset, schann, aid, keyno, no_auth, apdu, verbose,
    );
    if a.schann.is_some() && a.schann.as_deref() != Some("ev2") {
        return Err(AppError::CommandFailed(
            "verifycert only supports --schann ev2".to_string(),
        ));
    }
    if let Some(v) = isoid.as_deref().and_then(mfdes::opt) {
        a.flag("--isoid", &mfdes::isoid(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = dfname.as_deref().and_then(mfdes::opt) {
        a.flag("--dfname", &mfdes::dfname(&v).map_err(AppError::CommandFailed)?);
    }
    if let Some(v) = fid.as_deref().and_then(mfdes::opt) {
        a.flag("--fid", &mfdes::fid(&v).map_err(AppError::CommandFailed)?);
    }
    for (flag, val, allowed, label) in [
        ("--ca", &ca, &[] as &[&str], "CA"),
        ("--keyaid", &keyaid, &[] as &[&str], "key AID"),
        ("--keyisoid", &keyisoid, &[] as &[&str], "key ISO ID"),
        ("--keydfname", &keydfname, &[] as &[&str], "key DF name"),
        ("--keyidx", &keyidx, &[] as &[&str], "key index"),
        ("--validatemethod", &validatemethod, VALIDATE, "validation method"),
        ("--readmethod", &readmethod, READ, "read method"),
    ] {
        if let Some(v) = val.as_deref().and_then(mfdes::opt) {
            let checked = if allowed.is_empty() {
                match flag {
                    "--ca" => name_or_path(&v)?,
                    "--keyaid" => mfdes::aid(&v).map_err(AppError::CommandFailed)?,
                    "--keyisoid" | "--keydfname" => {
                        mfdes::isoid(&v).map_err(AppError::CommandFailed)?
                    }
                    _ => mfdes::keyno(&v).map_err(AppError::CommandFailed)?,
                }
            } else {
                let matched = allowed
                    .iter()
                    .find(|o| o.eq_ignore_ascii_case(&v))
                    .ok_or_else(|| {
                        AppError::CommandFailed(format!(
                            "{} must be one of {}, got '{}'",
                            label,
                            allowed.join("/"),
                            v
                        ))
                    })?;
                matched.to_string()
            };
            a.flag(flag, &checked);
        }
    }
    mfdes::validate_auth(&a).map_err(AppError::CommandFailed)?;
    let mut cmd = mfdes::build("verifycert", &a, VERIFYCERT);
    if let Some(v) = profile.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(" {}", name_or_path(&v)?));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// `hf mfdes makemfclicense` â€” build an MFC transfer licence descriptor.
#[tauri::command]
pub async fn desfire_makelicense(
    app: AppHandle,
    machine: State<'_, Mutex<WizardMachine>>,
    key: String,
    sectors: Option<String>,
    ka: bool,
    kb: bool,
    restrict: bool,
    map: bool,
    access_conditions: bool,
    readerid: Option<String>,
    mfc_keys: Option<String>,
    save: bool,
    verbose: bool,
) -> Result<String, AppError> {
    let mut cmd = String::from("hf mfdes makemfclicense");
    if verbose {
        cmd.push_str(" -v");
    }
    if save {
        cmd.push_str(" -s");
    }
    cmd.push_str(&format!(
        " -k {}",
        mfdes::key(&key).map_err(AppError::CommandFailed)?
    ));
    if let Some(v) = sectors.as_deref().and_then(mfdes::opt) {
        for part in v.split(',') {
            if !mfdes::keyno(part).is_ok() {
                return Err(AppError::CommandFailed(format!(
                    "Sectors (-b) must be a comma-separated list of numbers, got '{}'",
                    v
                )));
            }
        }
        cmd.push_str(&format!(" -b {}", v.replace(' ', "")));
    }
    for (name, on) in [
        ("--ka", ka),
        ("--kb", kb),
        ("--restrict", restrict),
        ("--map", map),
        ("--access-conditions", access_conditions),
    ] {
        if on {
            cmd.push_str(&format!(" {}", name));
        }
    }
    if let Some(v) = readerid.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " -r {}",
            mfdes::data(&v).map_err(AppError::CommandFailed)?
        ));
    }
    if let Some(v) = mfc_keys.as_deref().and_then(mfdes::opt) {
        cmd.push_str(&format!(
            " --mfc-keys {}",
            mfdes::data(&v).map_err(AppError::CommandFailed)?
        ));
    }
    let port = get_port(&machine)?;
    connection::run_command(&app, &port, &cmd).await
}

/// Accepts a bare name or a filesystem path, rejecting characters that would
/// change the meaning of the generated command.
fn name_or_path(v: &str) -> Result<String, AppError> {
    let t = v.trim();
    if t.is_empty() {
        return Err(AppError::CommandFailed("Value must not be empty".to_string()));
    }
    if t.chars().any(|c| {
        c.is_whitespace() || c == '"' || c == '\'' || c == ';' || c == '&' || c == '|'
    }) {
        return Err(AppError::CommandFailed(format!(
            "Name or path must not contain spaces or quote/semicolon characters, got '{}'",
            v
        )));
    }
    Ok(t.to_string())
}

/// `createapp --dfname` takes an ISO DF name as a plain string.
fn name_str(v: &str) -> Result<String, AppError> {
    let t = v.trim();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::CommandFailed(format!(
            "DF name (--dfname) must be a non-empty alphanumeric name, got '{}'",
            v
        )));
    }
    Ok(t.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> mfdes::Auth {
        mfdes::Auth::default()
    }

    fn key32() -> String {
        "000102030405060708090a0b0c0d0e0f".to_string()
    }

    #[test]
    fn unknown_subcommands_are_rejected() {
        assert!(allowed_for("notacommand").is_err());
        // Commands with dedicated handlers are not reachable via desfire_auth.
        assert!(allowed_for("dump").is_err());
        assert!(allowed_for("chk").is_err());
        assert!(allowed_for("etest").is_err());
    }

    #[test]
    fn lsapp_does_not_accept_aid() {
        // Verified against the client: `hf mfdes lsapp --aid 123456` fails with
        // `invalid option "--aid"`.
        let a = mfdes::Auth {
            key: Some(key32()),
            aid: Some("123456".into()),
            ..base()
        };
        let allowed = allowed_for("lsapp").unwrap();
        let cmd = mfdes::build("lsapp", &a, &allowed);
        assert_eq!(cmd, "hf mfdes lsapp -k 000102030405060708090a0b0c0d0e0f");
        assert!(!cmd.contains("--aid"));
    }

    #[test]
    fn default_accepts_neither_aid_nor_no_auth() {
        let a = mfdes::Auth {
            key: Some(key32()),
            aid: Some("123456".into()),
            no_auth: true,
            ..base()
        };
        let cmd = mfdes::build("default", &a, &allowed_for("default").unwrap());
        assert_eq!(cmd, "hf mfdes default -k 000102030405060708090a0b0c0d0e0f");
    }

    #[test]
    fn createdelegateapp_does_not_accept_keyno() {
        let a = mfdes::Auth {
            keyno: Some("1".into()),
            ..base()
        };
        let cmd = mfdes::build(
            "createdelegateapp",
            &a,
            &allowed_for("createdelegateapp").unwrap(),
        );
        assert_eq!(cmd, "hf mfdes createdelegateapp");
    }

    #[test]
    fn selectisofid_uses_isofid_not_isoid() {
        // Verified against the client: `--isoid` is rejected here.
        let a = mfdes::Auth {
            aid: Some("123456".into()),
            ..base()
        };
        let allowed = allowed_for("selectisofid").unwrap();
        assert!(!allowed.contains(&extras::ISOID));
        assert!(allowed.contains(&extras::ISOFID));
        let mut a = a;
        a.flag("--isofid", "df01");
        a.flag("--isoid", "df02");
        let cmd = mfdes::build("selectisofid", &a, &allowed);
        assert_eq!(cmd, "hf mfdes selectisofid --aid 123456 --isofid df01");
    }

    #[test]
    fn chkeysettings_has_no_isoid() {
        let allowed = allowed_for("chkeysettings").unwrap();
        assert!(!allowed.contains(&extras::ISOID));
        assert!(allowed.contains(&extras::DATA));
    }

    #[test]
    fn detect_includes_its_extra_flags() {
        let a = mfdes::Auth {
            key: Some(key32()),
            aid: Some("123456".into()),
            ..base()
        };
        let mut a = a;
        a.flag("--isoid", "df01");
        a.flag("--dfname", "0011223344");
        a.flag("-f", "mfdes_default_keys");
        a.switch("--save");
        assert_eq!(
            mfdes::build("detect", &a, DETECT),
            "hf mfdes detect -k 000102030405060708090a0b0c0d0e0f --aid 123456 \
             --isoid df01 --dfname 0011223344 -f mfdes_default_keys --save"
        );
    }

    #[test]
    fn read_and_write_keep_their_own_flag_sets() {
        let a = mfdes::Auth {
            key: Some(key32()),
            aid: Some("123456".into()),
            no_auth: true,
            ..base()
        };
        let mut r = a.clone();
        r.flag("--fid", "01");
        r.flag("--type", "data");
        r.flag("-o", "000000");
        assert_eq!(
            mfdes::build("read", &r, READ),
            "hf mfdes read -k 000102030405060708090a0b0c0d0e0f --aid 123456 \
             --no-auth --fid 01 --type data -o 000000"
        );
        let mut w = a;
        w.flag("--fid", "01");
        w.flag("-d", "01020304");
        w.switch("--debit");
        assert_eq!(
            mfdes::build("write", &w, WRITE),
            "hf mfdes write -k 000102030405060708090a0b0c0d0e0f --aid 123456 \
             --no-auth --fid 01 -d 01020304 --debit"
        );
    }

    #[test]
    fn every_dedicated_allowlist_matches_a_real_subcommand() {
        // Guards against a typo in the const tables producing a command the
        // client would reject with `invalid option`.
        for (sub, allowed) in SUB_SHARED {
            assert!(
                allowed_for(sub).is_ok(),
                "{} missing from SUB_EXTRAS or SUB_SHARED",
                sub
            );
            for flag in *allowed {
                assert!(
                    flag.starts_with('-'),
                    "{} allowlist entry is not a flag: {}",
                    sub,
                    flag
                );
            }
        }
        for (sub, _) in SUB_EXTRAS {
            assert!(allowed_for(sub).is_ok(), "{} not in SUB_SHARED", sub);
        }
    }
}
