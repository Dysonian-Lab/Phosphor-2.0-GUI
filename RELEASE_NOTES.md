# Phosphor v1.1.0 - Fixes

## Bug Fixes

### 1. Proxmark3 Binary Not Found Error
**Problem**: Application could not find the bundled proxmark3 binary, showing "[!!] ERROR - Proxmark3 binary not found."

**Root Cause**: 
- Tauri's `externalBin` configuration places `proxmark3.exe` in the root install directory
- The code was only looking in `binaries/` subdirectory for the binary
- Runtime DLLs were in the root but the binary lookup failed

**Fix**: Updated `try_sidecar_silent()` in `connection.rs` to check BOTH:
- Root directory (where DLLs are located) - checked FIRST
- `binaries/` subdirectory (alternative configuration)

### 2. Console Popup Windows
**Problem**: Each proxmark3 operation spawned a console window popup.

**Fix**: Added Windows `CREATE_NO_WINDOW` flag (0x08000000) to suppress console windows when spawning proxmark3 processes.

### 3. Serial Port Detection
**Problem**: Port detection used a static list without prioritization.

**Fix**: Implemented dynamic port enumeration using the `serialport` crate with heuristic scoring:
- Windows COM ports: +50 base score, +30 extra for COM1-10 (lower ports = higher priority)
- USB vendor matching (FTDI/SiLabs/Atmel): +30 bonus points
- Linux: ttyACM gets +50, ttyUSB gets +20
- macOS: Known PM3 suffixes get +50

## Files Changed
- `src-tauri/src/pm3/connection.rs` - Binary lookup, port scoring, console suppression
- `src-tauri/Cargo.toml` - Added `serialport = "4.7"` dependency

## Portable Distribution Structure
```
portable/
├── proxmark3.exe          # Binary in root (accesses DLLs)
├── phosphor.exe
├── *.dll                  # 129 MinGW runtime DLLs
├── resources/             # Frontend assets (index.html + assets/)
├── binaries/              # Also contains proxmark3.exe for compatibility
├── firmware/
├── platforms/
└── pm3-libs/
```

## Testing
Tested on Windows 10 with Proxmark3 connected via USB. The application:
- Launches successfully with the bundled frontend
- Finds and executes proxmark3 binary without errors
- Detects connected Proxmark3 device via serial port scoring
- Runs commands without console popup windows