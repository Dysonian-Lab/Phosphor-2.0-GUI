# Phosphor 2.0 GUI

Desktop GUI for Proxmark3. Scan, clone and manage RFID/NFC cards without touching the command line.

![Windows](https://img.shields.io/badge/Windows-10%2B-blue) ![License](https://img.shields.io/badge/license-GPL--3.0-green) ![Version](https://img.shields.io/badge/version-2.0.0-brightgreen)

## What it does

Phosphor wraps the Proxmark3 client into a visual wizard. You plug in your Proxmark, place a card on the reader, and Phosphor handles the rest: identifying the card type, reading its data, detecting the right blank, and writing the clone. The whole process is point-and-click.

**LF (125 kHz)** cards are cloned in seconds. **HF (13.56 MHz)** cards like MIFARE Classic go through automatic key recovery (autopwn) with real-time progress, then write to a magic card.

## What's new in 2.0.0

- **Iceman fork v4.21611** — upgraded PM3 firmware base for better compatibility and reliability
- **iCopy-X support** — native compatibility with the upgraded iCopy-X hardware
- **Intelligent port scoring** — faster Proxmark3 detection across COM, USB FTDI/SiLabs/Atmel adapters
- **Advanced Tools tab** — direct access to professional PM3 commands:
  - ISO 14443-B reader
  - ISO 15693 reader
  - Felica reader
  - iCLASS SE/SEOS reader
  - LEGIC reader
  - Lua Script Editor (open, edit, save, run scripts on-device)
  - Firmware flashing (RDV4, RDV4+BT, Generic)
  - Hardware tuning (`hw tune`, `lf tune`)
  - Antenna measurement (`hw measure`)
- **Improved error display** — Tauri errors surface as readable PM3 output instead of `[object Object]`

## Supported cards

### LF (125 kHz) - 22 types

HID ProxII, EM4100, AWID, IOProx, Indala, FDX-B, HID Corporate 1000, Paradox, Keri, Viking, Visa2000, Noralsy, Presco, Jablotron, NexWatch, PAC/Stanley, SecuraKey, Gallagher, GProxII, Pyramid, NEDAP, T55x7

### HF (13.56 MHz) - 6 types

MIFARE Classic 1K/4K (with autopwn key recovery), MIFARE Ultralight, NTAG, iCLASS/PicoPass, DESFire (detection only, non-cloneable)

### Supported magic blanks

T5577 (LF), Gen1a, Gen2/CUID, Gen3, Gen4 GTU, Gen4 GDM/USCUID (HF)

## Requirements

- **Proxmark3** device (Easy, RDV4, RDV4+BT, Generic, or compatible clone)
- **Windows 10** or later (x64)
- USB cable (data cable, not charge-only)
- Compatible with the upgraded icopy-x hardware

Proxmark3 firmware v4.21611+ recommended (tested with Iceman fork v4.21611). Phosphor bundles its own PM3 client binary, so you don't need a separate Proxmark3 installation.

## How to run on a clean Windows machine

1. Download `Phosphor_2.0_GUI_v2.0.0_Windows_Portable.zip` from [Releases](../../releases)
2. Extract the `.zip` to any folder (e.g., `C:\Tools\Phosphor`)
3. Plug in your Proxmark3 via USB
4. Double-click `phosphor.exe` to launch
5. On first run the app creates a `.proxmark3` folder in your home directory for logs, scripts, and saved cards

### Driver notes

- RDV4 / Easy / iCopy-X: use the Proxmark3 USB CDC driver from the PM3 installer or Zadig if Windows does not auto-enumerate the COM port.
- If the port does not appear, check Device Manager → Ports (COM & LPT).

### WebView2 runtime

Phosphor uses Tauri with the OS WebView. Windows 10 May 2020 Update (1903+) ships the WebView2 runtime by default. If you see a WebView error, install the [Evergreen Standalone Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

### Portable layout

```
Phosphor_2.0_GUI_v2.0.0_Windows_Portable/
├── phosphor.exe
├── proxmark3.exe
├── *.dll
├── resources/
├── firmware/
│   ├── rdv4/
│   ├── rdv4-bt/
│   └── generic/
├── pm3-libs/
└── platforms/
```

## Installation (setup-based)

1. Download `Phosphor_2.0_GUI_2.0.0_x64-setup.exe` from [Releases](../../releases)
2. Run the installer
3. Plug in your Proxmark3
4. Launch Phosphor

## Features

- **One-click cloning** for LF and HF cards
- **Auto-detection** of card type and frequency with port scoring
- **MIFARE Classic autopwn** with live progress (dictionary, nested, darkside, hardnested attacks)
- **Magic card detection** identifies Gen1a through Gen4 GDM
- **Blank card data check** warns if the blank already has data written to it
- **Firmware flash** with variant picker (RDV4, RDV4+BT, Generic)
- **T5577 chip detection** and password-protected chip handling
- **Advanced Tools tab** for ISO 14443-B/15693, Felica, iCLASS, LEGIC, Lua scripting, tuning, and antenna tests
- **Sound effects** and terminal-style UI

## Troubleshooting

- **"Proxmark3 binary not found"**: ensure `phosphor.exe` and `proxmark3.exe` are in the same folder. Do not move DLLs out of that folder.
- **"No tag found" / non-zero exit**: some PM3 commands intentionally return non-zero when no tag is present; this is expected and handled by the UI.
- **lf tune / hw tune**: `lf tune` takes no positional argument. `hw tune` is informational only; it does not actively tune the antenna.
- **iCLASS hangs**: some iCLASS commands hang without a built-in timeout. If a command appears frozen, close the connection and retry with PM3 default behavior.
- **Logs**: command output is captured and shown in the terminal panel. `.proxmark3` config and script data live under `%USERPROFILE%\.proxmark3\`.

## Building from source

```powershell
# Prerequisites: Node.js 18+, Rust 1.70+, C++ build tools

git clone https://github.com/Dysonian-Lab/phosphor-PM3-GUI.git
cd phosphor-PM3-GUI\phosphor
npm install
npm run tauri build
```

The PM3 client binary and its DLLs go in `src-tauri/binaries/` and `src-tauri/pm3-libs/`. See `tauri.conf.json` for the resource mapping.

## Tech stack

Tauri v2, React 19, TypeScript, XState v5, Rust. Dual state machine architecture: Rust backend (WizardMachine) and frontend (XState) stay in sync through Tauri commands.

## Author

Created by **Dysonian Lab** (fork of [nik shuv/phosphor](https://github.com/nikitaart2000/phosphor))

## License

[GPL-3.0](LICENSE) — Copyright 2025-2026 Dysonian Lab
