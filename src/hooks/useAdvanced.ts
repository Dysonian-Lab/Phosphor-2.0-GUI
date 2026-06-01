import { invoke } from '@tauri-apps/api/core';

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

export interface AntennaTestResult {
  output: string;
}

export const useAdvanced = () => ({
  // ISO 14443‑B
  iso14bInfo: async () => invoke<Iso14bInfo>('iso14b_info'),

  // ISO 15693
  iso15Info: async () => invoke<Iso15Info>('iso15_info'),

  // Felica
  felicaInfo: async () => invoke<FelicaInfo>('felica_info'),

  // iCLASS SE/SEOS
  iclassSeInfo: async () => invoke<IclassSeInfo>('iclass_se_info'),

  // LEGIC
  legicInfo: async () => invoke<LegicInfo>('legic_info'),

  // Scripting
  runScript: async (script: string) => invoke<ScriptResult>('run_script', { script }),
  listScripts: async () => invoke<string[]>('list_scripts'),
  readScript: async (filename: string) => invoke<string>('read_script', { filename }),
  writeScript: async (filename: string, content: string) => invoke<void>('write_script', { filename, content }),

  // Firmware flashing (reuse existing)
  flashFirmware: async (port: string, variant: string) =>
    invoke<void>('flash_firmware', { port, variant }),

  // Tuning
  hwTune: async () => invoke<string>('hw_tune'),
  lfTune: async () => invoke<string>('lf_tune'),

  // Antenna / measure
  hwMeasure: async () => invoke<AntennaTestResult>('hw_measure'),
});