# Phosphor 2.2 GUI

Desktop GUI for Proxmark3. Scan, clone and manage RFID/NFC cards without touching the command line.

![Windows](https://img.shields.io/badge/Windows-10%2B-blue) ![License](https://img.shields.io/badge/license-GPL--3.0-green) ![Version](https://img.shields.io/badge/version-2.2.1-brightgreen)

> ### ⚠️ Requirements — read before using this build
>
> **v2.2.1 only works with one of these two device firmwares:**
>
> 1. **iCopy-X running the FULL FLASH `icopy-x v1.1.6`** (lab-401
>    `icopy-x-flash.ipk`), or
> 2. **Any Proxmark3 running the FULL FLASH `Frosty Lemon` / Iceman
>    `v4.23346`**, `CAPABILITIES_VERSION 11`.
>
> Client and firmware must be flashed as a **matched pair**. A stock Proxmark
> firmware, a partially flashed device, or an iCopy-X still on its factory
> Proxmark build will misbehave — most visibly as blank/failed detection that
> is not a real detection fault.
>
> Verify with `hw version`: you must see `v4.23346` and
> `CAPABILITIES_VERSION: 11`.
>
> **Phosphor does not flash or update your device.** See
> [iCopy-X firmware](#icopy-x) below.

## What it does

Phosphor wraps the Proxmark3 client into a visual wizard. You plug in your Proxmark, place a card on the reader, and Phosphor handles the rest: identifying the card type, reading its data, detecting the right blank, and writing the clone. The whole process is point-and-click.

**LF (125 kHz)** cards are cloned in seconds. **HF (13.56 MHz)** cards like MIFARE Classic go through automatic key recovery (autopwn) with real-time progress, then write to a magic card.

## What's new in 2.2.1

- **Retry a failed write in place** — RETRY now returns to the blank step and keeps your source card and device connection. It used to drop you back at Scan and discard the source card, so a failed write could never be retried without walking the whole wizard again.
- **Failed verification is retryable** — previously a failed verify only offered RESET (discard everything) or DISCONNECT (unplug).
- **New BACK button after a successful clone** — returns to the blank step while staying connected and keeping the source card loaded.
- **Fixed: T5577 blank detection missed `T55x7`** — Iceman prints the chip with a lowercase `x`, which the detector never matched, so it fell back to matching surrounding label text. Any firmware formatting that label differently reported a blank as absent ("Place the correct blank" on a PM3 Easy).
- **The version shown in the app is now read from `package.json`**, so the splash screen and top bar can no longer drift from the real build version.

### What's new in 2.2.0

- **Iceman fork v4.23346** — upgraded PM3 firmware base to the latest upstream
- **CAPABILITIES_VERSION 11** — client and firmware must be flashed as a matched pair
- **21 Advanced tab panels** — up from 14. New tabs: DESFire, MF View (RKF/VIGIK/HID PACS), eMRTD (PACE-CAM passport), Smart Card (ISO 7816-3 PPS), Antifuzz, T55xx, Trace
- **Full DESFire support** — a dedicated panel exposing all 57 `hf mfdes` subcommands: card info, application/file/key management, `read`/`write`/`value`, `dump`, `detect`, key `chk`, `bruteaid`, the self-contained EV1 emulator (`eload`/`esave`/`eview`/`view`/`sim`/`etest`), card configuration, and DUOX (`verifycert`, `intauth`, `vdesign`)
- **New commands**: `hf mf view`, `hf 14b view -f` (MyKey/COGES), `smart pps`, `hf emrtd info/dump/list/test`, `trace clear`, `hf 14a antifuzz --coll`, `lf t55xx set config/chk pwds/dangerraw/wakeup`, `hf thinfilm sim`, `mad read/write/verify/decode/encode`, `nfc encode`, `hf mfu ndefwrite/ndefformat/chk`, `hf iclass legbrute`, `hf 14b rdbl/ctrdbl/view --selftest`, `hf calypso info/dump/list`, `hf felica sim`, `lf trovan`
- **Command alignment** — `hf mf cchk` + `hf mf aeschk` merged into `hf mf chk`; `hf 14b valid` removed
- **19 unreleased master features evaluated** — 13 fully covered, 2 partial, 4 N/A (BWM BLE/WiFi/power, hw powersave, Flipper Zero link, ePassport)
- **Fixed: T55xx dictionary checks no longer time out** — `lf t55xx chk` took 26.8s against a 30s cap; on a slower run the Proxmark3 process was killed mid-attack while holding the serial port, breaking the *next* command. Brute-force commands now get a 15-minute budget
- **Readable PM3 errors** — documented exit codes are translated into plain language instead of raw integers like `Exit code -20`

## Supported cards

### LF (125 kHz) - 22 types

HID ProxII, EM4100, AWID, IOProx, Indala, FDX-B, HID Corporate 1000, Paradox, Keri, Viking, Visa2000, Noralsy, Presco, Jablotron, NexWatch, PAC/Stanley, SecuraKey, Gallagher, GProxII, Pyramid, NEDAP, T55x7

### HF (13.56 MHz) - 6 types

MIFARE Classic 1K/4K (with autopwn key recovery), MIFARE Ultralight, NTAG, iCLASS/PicoPass, DESFire, DESFire Light (DUOX)

DESFire cards are detected and can be fully managed from the **Advanced → DESFire**
panel (dump, read/write, key recovery, emulator). They are not "cloned" onto a
magic card the way MIFARE Classic is — DESFire has no universal magic card.

### Supported magic blanks

T5577 (LF), Gen1a, Gen2/CUID, Gen3, Gen4 GTU, Gen4 GDM/USCUID (HF)

## Requirements

- **Proxmark3** device (Easy, RDV4, RDV4+BT, Generic, iCopy-X, or compatible clone)
- **Windows 10** or later (x64)
- USB cable (data cable, not charge-only)

Proxmark3 firmware v4.23346+ recommended (tested with Iceman fork v4.23346). Phosphor bundles its own PM3 client binary, so you don't need a separate Proxmark3 installation.

### iCopy-X

Phosphor is **verified working with iCopy-X hardware**, tested on a physical
iCopy-X on COM19 over USB CDC running the lab-401 **icopy-x v1.1.6**
open-source firmware (the `icopy-x-flash.ipk` variant).

That variant ships the Proxmark module as **Iceman v4.23346**
(`CAPABILITIES_VERSION 11`), which is exactly what Phosphor bundles and what it
is verified against. Client and firmware feature versions must match or the
Proxmark refuses to talk.

> ⚠️ **Phosphor does not update or flash an iCopy-X.**
>
> Updating an iCopy-X is an **IPK update performed on the device**, not a
> Proxmark firmware flash from the desktop. This is a manual, optional step —
> if your device already reports Iceman v4.23346, you do not need to do
> anything.
>
> Note that three different version numbers are in play, and they are not
> interchangeable:
>
> | Version | What it is |
> |---|---|
> | `1.0.90` | The device's own base/bootloader firmware. A **precondition**, not an upgrade target. |
> | `v1.1.5` / `v1.1.6` | The **IPK package** (device shell + menus). |
> | `v4.23346` | The **Iceman Proxmark client/firmware bundled inside** that IPK — what Phosphor talks to. |
>
> v1.1.5 and v1.1.6 both ship Iceman v4.23346, so flashing v1.1.6 over
> v1.1.5 does not change the Proxmark client Phosphor depends on. Per
> [lab-401/icopy-x](https://github.com/lab-401/icopy-x/releases/tag/v1.1.6):
>
> 1. Ensure the device's base firmware is 1.0.90
> 2. Put the iCopy-X into **PC-Mode**
> 3. Delete **all other** IPK files from the device
> 4. Transfer the IPK, then close PC-Mode
> 5. On the device: **About → Update**, press OK
> 6. Using the `flash` variant, the device offers to flash the Proxmark
>    firmware — accept it
>
> Use **`icopy-x-flash.ipk`** (Iceman v4.23346, matching Phosphor), not
> `icopy-x-noflash.ipk` (which leaves the factory Proxmark in place and will not
> match Phosphor's client).

## How to run on a clean Windows machine

1. Download `Phosphor_2.2_GUI_v2.2.1_Windows_Portable.zip` from [Releases](../../releases)
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
Phosphor_2.2_GUI_v2.2.1_Windows_Portable/
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

1. Download `Phosphor_2.2.1_x64-setup.exe` from [Releases](../../releases)
2. Run the installer
3. Plug in your Proxmark3
4. Launch Phosphor

## Features

- **One-click cloning** for LF and HF cards
- **Auto-detection** of card type and frequency with port scoring
- **MIFARE Classic autopwn** with live progress (dictionary, nested, darkside, hardnested attacks)
- **Magic card detection** identifies Gen1a through Gen4 GDM
- **Blank card data check** warns if the blank already has data written to it
- **Firmware flash** with variant picker (RDV4, RDV4+BT, Generic) — **not for iCopy-X**, see the warning above
- **T5577 chip detection** and password-protected chip handling
- **Advanced Tools tab** — 21 panels covering ISO 14443-B/15693, Felica, iCLASS, LEGIC, **DESFire**, MIFARE View, Calypso, ThinFilm, MAD, NFC, Ultralight, eMRTD, Smart Card, Antifuzz, T55xx, Trace, Lua scripting, firmware flashing, tuning and antenna tests
- **Sound effects** and terminal-style UI

## Troubleshooting

- **"Proxmark3 binary not found"**: ensure `phosphor.exe` and `proxmark3.exe` are in the same folder. Do not move DLLs out of that folder.
- **"No tag found" / non-zero exit**: some PM3 commands intentionally return non-zero when no tag is present; this is expected and handled by the UI. Documented PM3 error codes are translated into readable messages.
- **"APDU exchange failed"**: the card on the reader is not the type the command expects (e.g. a DESFire command run against a MIFARE Classic). Put the correct card on the reader.
- **Commands that seem to hang**: dictionary and brute-force attacks legitimately run for minutes. They use a 15-minute timeout; ordinary commands time out in 30s.
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
