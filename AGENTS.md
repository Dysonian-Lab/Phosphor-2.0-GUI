# Agent Instructions for Phosphor Release

## Release Commands

To execute the release workflow:

### 1. Clean old build artifacts
```powershell
powershell -ExecutionPolicy Bypass -File "D:\kilocode\Phosphor-debug\clean_artifacts.ps1"
```

### 2. Build portable ZIP from v2.2.0 installer
```powershell
powershell -ExecutionPolicy Bypass -File "D:\kilocode\Phosphor-debug\build_portable.ps1"
```

### 3. Create/update GitHub release v2.2.0
```powershell
powershell -ExecutionPolicy Bypass -File "D:\kilocode\Phosphor-debug\create_release.ps1"
```

## Build Commands (PM3 v4.23346)

### Build PM3 client
```bash
"D:\proxspace\ProxSpace-master\msys2\usr\bin\bash.exe" -lc 'export PATH="/d/proxspace/ProxSpace-master/gcc-arm-none-eabi-10.3-2021.10/bin:/d/proxspace/ProxSpace-master/msys2/mingw64/bin:/d/proxspace/ProxSpace-master/msys2/usr/bin:/mingw64/bin:$PATH"; cd /d/kilocode/Phosphor-debug/proxmark3-4.23346 && make client IS_MINGW=1 LUAPLATFORM=mingw'
```

### Build firmware (all 3 variants)
```bash
"D:\proxspace\ProxSpace-master\msys2\usr\bin\bash.exe" -lc "bash /d/kilocode/Phosphor-debug/bac.sh"
```
This builds rdv4, rdv4-bt (BTADDON FPC USART_DEV), and generic (SMARTCARD) firmware and copies them into the app.

### Toolchain
- ARM cross-compiler: `D:\proxspace\ProxSpace-master\gcc-arm-none-eabi-10.3-2021.10\bin\arm-none-eabi-gcc.exe` (10.3.1)
- MSYS2: `D:\proxspace\ProxSpace-master\msys2`
- Note: Use the ProxSpace toolchain, NOT the system MSYS2 ucrt64 toolchain (binutils 2.47 has a linker regression with duplicate ldscript INCLUDE).

## GitHub Token
- Token: (see memory-bank/activeContext.md for current token)
- Repository: Dysonian-Lab/Phosphor-2.0-GUI
- **WARNING**: This token has ZERO OAuth scopes and cannot write. A permissioned token is needed for the release.

## Emergency Fix: Version Number Issue
To fix version number issues in the release:

### Run the emergency fix script
```powershell
powershell -ExecutionPolicy Bypass -File "D:\kilocode\Phosphor-debug\fix_release.ps1"
```

**Note:** For a simpler one-command fix, run `fix_release.ps1` which handles all necessary corrections automatically.