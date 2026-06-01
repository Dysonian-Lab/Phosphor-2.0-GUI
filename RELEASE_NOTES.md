# Phosphor 2.0 GUI - Release Notes

## What's new

### Foundation: Iceman fork v4.21611
- Upgraded the bundled Proxmark3 client to the Iceman fork v4.21611.
- Firmware compatibility, command behavior, and feature coverage are now aligned with the current upstream PM3 baseline.

### Hardware: iCopy-X compatibility
- Added native support for the iCopy-X device.
- USB driver / enumeration is handled automatically via the improved port scoring.

### Detection: Intelligent port scoring
- Dynamic COM port enumeration with heuristic scoring.
- Windows COM ports: +50 base score, +30 extra for COM1-10.
- USB vendor matching (FTDI/SiLabs/Atmel): +30 bonus points.
- Result: faster, more reliable Proxmark3 discovery.

### Advanced Tools tab
Direct, button-driven access to professional PM3 commands:
- ISO 14443-B reader
- ISO 15693 reader
- Felica reader
- iCLASS SE/OS reader
- LEGIC reader
- Lua Script Editor (CodeMirror-based with syntax highlighting, line numbers, session-style workflow)
- Firmware flashing (RDV4, RDV4+BT, Generic)
- Hardware tuning (`hw tune`, `lf tune`)
- Antenna measurement (`hw measure`)

### Error handling
- Tauri error objects now surface as readable PM3 command output.
- No more `[object Object]` in the UI.

## Command behavior notes

- **No tag found**: Several `hf` / `lf` information commands return non-zero exit codes when no tag is present. This is expected PM3 behavior and is now handled gracefully.
- **`lf tune` syntax**: the command takes no positional argument. Do not append a value.
- **`hw tune`**: informational only; it does not actively tune the antenna.
- **iCLASS**: some commands hang without a built-in timeout. If a command appears frozen, cancel the connection and retry.

## Portable structure

```
portable/
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

## Verified

- Windows 10 x64 with Proxmark3 USB + bundled client.
- Missing binary / console popup issues resolved.
- Serial port detection confirmed with heuristic scoring.
