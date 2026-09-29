import { invoke } from '@tauri-apps/api/core';
import { listDumpFiles } from '../lib/api';

// Define the response types matching the Rust structs
export interface Iso14bInfo {
  uid: string;
  atqa: string;
}

export interface Iso15Info {
  uid: string;
  dsfid: string;
}

export interface FelicaInfo {
  idm: string;
  pmm: string;
}

export interface IclassSeInfo {
  uid: string;
  atqa: string;
}

export interface LegicInfo {
  uid: string;
  atqa: string;
}

export interface ScriptResult {
  output: string;
}

export interface ScriptRunParams {
  script: string;
  args?: string;
}

export interface AntennaTestResult {
  output: string;
}

export const useAdvanced = () => ({
  // ISO 14443‑B
  iso14bInfo: async () => invoke<Iso14bInfo>('iso14b_info'),

  // ISO 14443-B v4.23346: rdbl / ctrdbl / view --selftest / view -f
  iso14bRdbl: async (block: number) => invoke<string>('iso14b_rdbl', { block }),
  iso14bCtrdbl: async (block: number) => invoke<string>('iso14b_ctrdbl', { block }),
  iso14bViewSelftest: async () => invoke<string>('iso14b_view_selftest'),
  iso14bView: async (file: string) => invoke<string>('iso14b_view', { file: file }),

  // ISO 15693
  iso15Info: async () => invoke<Iso15Info>('iso15_info'),

  // Felica
  felicaInfo: async () => invoke<FelicaInfo>('felica_info'),
  // Felica v4.23346: sim from dump file
  felicaSim: async (dumpPath: string) => invoke<string>('felica_sim', { dumpPath: dumpPath }),

  // iCLASS SE/SEOS
  iclassSeInfo: async () => invoke<IclassSeInfo>('iclass_se_info'),
  // iCLASS v4.23346: legbrute key recovery
  iclassLegbrute: async () => invoke<string>('iclass_legbrute'),

  // LEGIC
  legicInfo: async () => invoke<LegicInfo>('legic_info'),

  // MFU v4.23346: chk/ndefwrite/ndefformat
  mfuChk: async () => invoke<string>('mfu_chk'),
  mfuNdefwrite: async (data: string) => invoke<string>('mfu_ndefwrite', { data: data }),
  mfuNdefformat: async () => invoke<string>('mfu_ndefformat'),

  // MF v4.23346: chk (cchk + aeschk merged)
  mfChk: async () => invoke<string>('mf_chk'),

  // MF view v4.23346: RKF / VIGIK / HID PACS decode
  hfMfView: async (file: string) => invoke<string>('hf_mf_view', { file: file }),
  hfMfViewSelftest: async () => invoke<string>('hf_mf_view_selftest'),

  // MF autopwn v4.23346: automatic key recovery + dump (creates hf-mf-<uid>-dump.bin)
  hfMfAutopwn: async () => invoke<string>('hf_mf_autopwn'),
  // MF dump v4.23346: dump a MIFARE Classic card to hf-mf-<uid>-dump.bin
  hfMfDump: async () => invoke<string>('hf_mf_dump'),
  // MF cview v4.23346: backdoor block read (magic cards, no keys needed)
  hfMfCview: async () => invoke<string>('hf_mf_cview'),

  // MF read / write / break / emulator (v4.23346)
  mfRdbl: async (block: number, key: string) => invoke<string>('mf_rdbl', { block, key }),
  mfRdsc: async (sector: number, key: string) => invoke<string>('mf_rdsc', { sector, key }),
  mfWrbl: async (block: number, key: string, data: string) => invoke<string>('mf_wrbl', { block, key, data }),
  mfNested: async (cardType: string, block: number, keyType: string, key: string) =>
    invoke<string>('mf_nested', { cardType: cardType, block: block, keyType: keyType, key: key }),
  mfEsave: async (filename?: string, size?: string) => invoke<string>('mf_esave', { filename: filename ?? null, size: size ?? null }),
  mfSim: async (size?: string, uid?: string) => invoke<string>('mf_sim', { size: size ?? null, uid: uid ?? null }),
  mfEclr: async () => invoke<string>('mf_eclr'),
  mfEload: async (filename: string, size?: string) => invoke<string>('mf_eload', { filename, size: size ?? null }),
  mfEgetblk: async (block: number) => invoke<string>('mf_egetblk', { block }),
  mfEsetblk: async (block: number, data: string) => invoke<string>('mf_esetblk', { block, data }),

  // Calypso v4.23346: info/dump/list
  calypsoInfo: async () => invoke<string>('calypso_info'),
  calypsoDump: async () => invoke<string>('calypso_dump'),
  calypsoList: async () => invoke<string>('calypso_list'),

  // Thinfilm v4.23346: sniff (sim needs a dump file — not exposed)
  thinfilmSniff: async () => invoke<string>('thinfilm_sniff'),

  // MAD v4.23346: read/write/verify/decode/encode
  madRead: async () => invoke<string>('mad_read'),
  madWrite: async (data: string) => invoke<string>('mad_write', { data: data }),
  madVerify: async () => invoke<string>('mad_verify'),
  madDecode: async () => invoke<string>('mad_decode'),
  madEncode: async (data: string) => invoke<string>('mad_encode', { data: data }),

  // NFC v4.23346: encode (with record-type flags)
  nfcEncode: async (data: string) => invoke<string>('nfc_encode', { data: data }),

  // LF T55xx v4.23346: additional commands
  lfT55xxSetConfig: async () => invoke<string>('lf_t55xx_set_config'),
  lfT55xxChkPwds: async () => invoke<string>('lf_t55xx_chk_pwds'),
  lfT55xxDangerraw: async () => invoke<string>('lf_t55xx_dangerraw'),
  lfT55xxWakeup: async () => invoke<string>('lf_t55xx_wakeup'),

  // HF 14a antifuzz v4.23346
  hf14aAntifuzz: async () => invoke<string>('hf_14a_antifuzz'),
  hf14aAntifuzzColl: async () => invoke<string>('hf_14a_antifuzz_coll'),

  // Smart card v4.23346: PPS
  smartPps: async () => invoke<string>('smart_pps'),
  smartPpsT0: async () => invoke<string>('smart_pps_t0'),
  smartPpsT1: async () => invoke<string>('smart_pps_t1'),
  smartPpsTa1: async (ta1: string) => invoke<string>('smart_pps_ta1', { ta1: ta1 }),

  // eMRTD v4.23346: PACE-CAM passport reading
  emrtdInfo: async () => invoke<string>('hf_emrtd_info'),
  emrtdDump: async () => invoke<string>('hf_emrtd_dump'),
  emrtdList: async () => invoke<string>('hf_emrtd_list'),
  emrtdTest: async () => invoke<string>('hf_emrtd_test'),

  // Trace v4.23346
  traceClear: async () => invoke<string>('trace_clear'),

  // LF Trovan v4.23346
  lfTrovan: async () => invoke<string>('lf_trovan'),

  // Scripting
  runScript: async (script: string, args?: string) =>
    invoke<ScriptResult>('run_script', args ? { script, args } : { script }),
  listScripts: async () => invoke<string[]>('list_scripts'),
  readScript: async (filename: string) => invoke<string>('read_script', { filename }),
  writeScript: async (filename: string, content: string) => invoke<void>('write_script', { filename, content }),

  // Firmware flashing (reuse existing)
  flashFirmware: async (port: string, variant: string) =>
    invoke<void>('flash_firmware', { port, hardwareVariant: variant }),

  // Dump file discovery — scans the app dir for PM3 dump files
  listDumpFiles: async () => listDumpFiles(),

  // Tuning
  hwTune: async () => invoke<string>('hw_tune'),
  hwDecay: async (stabilizeMs?: number, measureUs?: number) =>
    invoke<string>('hw_decay', { stabilizeMs: stabilizeMs ?? null, measureUs: measureUs ?? null }),
  hfTune: async (iter?: number, style?: string) =>
    invoke<string>('hf_tune', { iter: iter ?? null, style: style ?? null }),
  lfTune: async () => invoke<string>('lf_tune'),

  // Antenna / measure (legacy alias — hw tune is the real command)
  hwMeasure: async () => invoke<string>('hw_tune'),
});