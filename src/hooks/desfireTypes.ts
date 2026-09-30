// Argument types for the DESFire (`hf mfdes`) commands of PM3 v4.23346.
//
// Every option name here mirrors the client's own `--help` output. Optional
// values are sent as `null` when blank so the backend omits the flag entirely
// rather than passing an empty argument.

/** Fields shared by nearly every DESFire subcommand. */
export type DesfireAuthBase = {
  /** `-t` crypto algorithm. */
  algo?: string | null;
  /** `-k` key, 16/32/48 hex characters (8/16/24 bytes). */
  key?: string | null;
  /** `--kdf` for most commands: none, AN10922 or gallagher. */
  kdf?: string | null;
  /** `-i` KDF input, 1-31 hex bytes. */
  kdfi?: string | null;
  /** `-m` communication mode. */
  cmode?: string | null;
  /** `-c` command set. */
  ccset?: string | null;
  /** `--schann` secure channel. */
  schann?: string | null;
  /** `--aid` application ID, 3 hex bytes. */
  aid?: string | null;
  /** `-n` key number. */
  keyno?: string | null;
  noAuth?: boolean;
  apdu?: boolean;
  verbose?: boolean;
};

/** For `desfire_auth`, which dispatches over the auth + extras commands. */
export type DesfireAuthArgs = DesfireAuthBase & {
  sub: string;
  isoid?: string | null;
  dfname?: string | null;
  fid?: string | null;
  isofid?: string | null;
  amode?: string | null;
  rawdata?: string | null;
  rawrights?: string | null;
  rrights?: string | null;
  wrights?: string | null;
  rwrights?: string | null;
  chrights?: string | null;
  damslot?: string | null;
  start?: string | null;
  end?: string | null;
  step?: string | null;
  data?: string | null;
  save?: boolean;
  cyclic?: boolean;
  backup?: boolean;
  ka?: boolean;
  kb?: boolean;
  accessConditions?: boolean;
  restoreTransfer?: boolean;
};

export type DesfireDetectArgs = DesfireAuthBase & {
  isoid?: string | null;
  dfname?: string | null;
  file?: string | null;
  save?: boolean;
};

/** `chk` uses a numeric KDF and has no `-t`/`-m`/`-c`. */
export type DesfireChkArgs = {
  aid?: string | null;
  key?: string | null;
  file?: string | null;
  /** `--kdf` 0 (none), 1 (AN10922) or 2 (gallagher). */
  kdf?: string | null;
  kdfi?: string | null;
  schann?: string | null;
  pattern1b?: boolean;
  pattern2b?: boolean;
  startp2b?: string | null;
  json?: string | null;
  apdu?: boolean;
  verbose?: boolean;
};

export type DesfireDumpArgs = DesfireAuthBase & {
  isoid?: string | null;
  dfname?: string | null;
  /** `-l` maximum read length, 3 hex bytes. */
  length?: string | null;
  file?: string | null;
  keys?: string | null;
  /** `--ns`, do not save the dump to a file. */
  noSave?: boolean;
};

export type DesfireReadArgs = DesfireAuthBase & {
  fid?: string | null;
  isoid?: string | null;
  dfname?: string | null;
  fileisoid?: string | null;
  /** `--type` auto, data, value, record or mac. */
  ftype?: string | null;
  /** `-o` offset, 3 hex bytes. */
  offset?: string | null;
};

export type DesfireWriteArgs = DesfireReadArgs & {
  /** `-d` data to write. */
  data?: string | null;
  readerid?: string | null;
  trkey?: string | null;
  updaterec?: string | null;
  debit?: boolean;
  commit?: boolean;
};

export type DesfireValueArgs = DesfireAuthBase & {
  fid?: string | null;
  isoid?: string | null;
  /** `-o` get, credit, limcredit, debit or clear. */
  operation?: string | null;
  data?: string | null;
};

/** `hf mfdes list` is an alias of `trace list -t des -c`. */
export type DesfireListArgs = {
  buffer?: boolean;
  frame?: boolean;
  crc?: boolean;
  relative?: boolean;
  microseconds?: boolean;
  pcap?: boolean;
  file?: string | null;
};

export type DesfirePcArgs = {
  key: string;
  retry?: string | null;
  ccset?: string | null;
  aid?: string | null;
  apdu?: boolean;
  verbose?: boolean;
};

export type DesfireCreateappArgs = DesfireAuthBase & {
  rawdata?: string | null;
  fid?: string | null;
  /** Plain DF name string here, unlike the hex DF name used elsewhere. */
  dfname?: string | null;
  dfhex?: string | null;
  ks1?: string | null;
  ks2?: string | null;
  dstalgo?: string | null;
  numkeys?: string | null;
};

export type DesfireCreatefileArgs = DesfireAuthBase & {
  fid?: string | null;
  isofid?: string | null;
  rawtype?: string | null;
  rawdata?: string | null;
  amode?: string | null;
  rawrights?: string | null;
  rrights?: string | null;
  wrights?: string | null;
  rwrights?: string | null;
  chrights?: string | null;
  size?: string | null;
  backup?: boolean;
};

export type DesfireChangekeyArgs = DesfireAuthBase & {
  isoid?: string | null;
  oldalgo?: string | null;
  oldkey?: string | null;
  newkeyno?: string | null;
  newalgo?: string | null;
  newkey?: string | null;
  newver?: string | null;
};

export type DesfireBruteaidArgs = {
  start?: string | null;
  end?: string | null;
  info?: string | null;
  preset?: string | null;
};

export type DesfireBruteisofidArgs = {
  aid?: string | null;
  isoid?: string | null;
  dfname?: string | null;
  start?: string | null;
  end?: string | null;
  step?: string | null;
  apdu?: boolean;
  verbose?: boolean;
};

/** `vdesign` and `intauth` share a shape. */
export type DesfireCertArgs = {
  data?: string | null;
  /** `-p` hex, pem, der or path. */
  source?: string | null;
  aid?: string | null;
  isoid?: string | null;
  dfname?: string | null;
  keyno?: string | null;
  apdu?: boolean;
  verbose?: boolean;
};

export type DesfireVerifycertArgs = DesfireAuthBase & {
  profile?: string | null;
  isoid?: string | null;
  dfname?: string | null;
  fid?: string | null;
  ca?: string | null;
  keyaid?: string | null;
  keyisoid?: string | null;
  keydfname?: string | null;
  keyidx?: string | null;
  validatemethod?: string | null;
  readmethod?: string | null;
};

export type DesfireMakelicenseArgs = {
  key: string;
  sectors?: string | null;
  ka?: boolean;
  kb?: boolean;
  restrict?: boolean;
  map?: boolean;
  accessConditions?: boolean;
  readerid?: string | null;
  mfcKeys?: string | null;
  save?: boolean;
  verbose?: boolean;
};

