# Phosphor 2.2 GUI - Release Notes

## v2.2.0 — PM3 v4.23346 Alignment (September 2026)

### Foundation: Iceman fork v4.23346 ("Frosty Lemon")
- Upgraded the bundled Proxmark3 client to the Iceman fork v4.23346.
- Rebuilt all 3 firmware images (rdv4, rdv4-bt, generic) from v4.23346 source.
- **CAPABILITIES_VERSION is now 11** — client and firmware must be flashed as a matched pair or they refuse to talk.

### Command alignment (v4.23346)
- `hf mf cchk` + `hf mf aeschk` → merged into `hf mf chk`.
- `hf mfdes chk` now runs with no args.
- `hf 14b valid` removed.

### Advanced tab — 20 panels (up from 14)
All v4.23346 commands exposed as buttons:

| Tab | Commands |
|-----|----------|
| 14B | info, rdbl, ctrdbl, view --selftest, view -f (MyKey/COGES) |
| 15693 | info |
| Felica | info, sim from dump |
| iCLASS | info, legbrute |
| LEGIC | info |
| DESFire | eload/esave/eview/dump/view/etest/sim/chk/detect |
| **MF View** | view -f (RKF/VIGIK/HID PACS decode), view --selftest |
| Calypso | info/dump/list |
| ThinFilm | sniff, sim |
| MAD | read/write/verify/decode/encode |
| NFC | encode |
| MFU | chk/ndefwrite/ndefformat |
| **eMRTD** | info/dump/list/test (PACE-CAM passport) |
| **Smart Card** | pps/t0/t1/ta1 (ISO 7816-3 PPS) |
| **Antifuzz** | antifuzz/antifuzz --coll |
| **T55xx** | set config/chk pwds/dangerraw/wakeup |
| **Trace** | clear |
| Script | 72 Lua + 21 lualibs + 3 cmdscripts + 33 pyscripts |
| Firmware | flash (rdv4/rdv4-bt/generic) |
| Tuning | hw tune, lf tune |
| Antenna | hw measure |

### Evaluated master features (19 items)
- **12 COMPLETE** — all commands from the unreleased CHANGELOG that exist in v4.23346 are now exposed in the GUI
- **3 PARTIAL** — `hf mfdes chk` (flags only), `nfc encode` (record types only), `hf mfu ndefwrite` (record types only)
- **4 N/A** — BWM BLE/WiFi/power (not in v4.23346), `hw powersave` (not in v4.23346), Flipper Zero link (firmware test scaffolding only, no CLI), ePassport (Kivy 60-100 MB, deferred)

### Build environment
- ARM cross-compiler: `arm-none-eabi-gcc` 10.3.1 (ProxSpace toolchain).
- PM3 client built with `make client IS_MINGW=1 LUAPLATFORM=mingw`.
- Firmware built with `make fullimage PLATFORM=PM3RDV4 [PLATFORM_EXTRAS=...]`.

### Verification
- `cargo build` — SUCCESS
- `cargo test --lib` — 272 passed, 0 failed
- `npm run build` — SUCCESS (3.33s)
- `npm run tauri build` — SUCCESS (NSIS installer + portable ZIP)
- Device verified on v4.23346 (`hw version` reports `Iceman/master/v4.23346`)

## v2.1.0 — iCopy-X ICS Decoder Support (August 2026)
- PM3 client rebuilt with iCopy-X patches.
- Version bumped to 2.1.0.

## v2.0.0 (May 2026)
- Foundation: Iceman fork v4.21611.
- Hardware: iCopy-X compatibility.
- Detection: Intelligent port scoring.
- Advanced Tools tab: ISO 14443-B, ISO 15693, Felica, iCLASS SE/OS, LEGIC, Lua Script Editor, Firmware flashing, Hardware tuning, Antenna measurement.

## Important notice (July 5, 2026)
The May 31, 2026 release incorrectly showed **v1.1.0** in file properties. This was caused by stale build artifacts being packaged during the release process. The issue has been fixed by cleaning the build cache and rebuilding.

**If you downloaded Phosphor 2.0.0 on or before July 5, 2026**, please re-download to obtain the correct **v2.0.0** build.

## Command behavior notes
- **No tag found**: Several `hf` / `lf` information commands return non-zero exit codes when no tag is present. This is expected PM3 behavior and is now handled gracefully.
- **`lf tune` syntax**: the command takes no positional argument. Do not append a value.
- **`hw tune`**: informational only; it does not actively tune the antenna.
- **iCLASS**: some commands hang without a built-in timeout. If a command appears frozen, cancel the connection and retry.
- **`smart pps`**: requires RDV4 with smartcard module. Not available on PM5.
- **`hf emrtd`**: online modes need an ISO 14443 tag; `hf emrtd test` and `hf emrtd list` work offline.
- **`hf 14a antifuzz`**: fuzzes reader anticollision. Use `--coll` for collision storm mode.

## Portable structure
```
portable/
|-- phosphor.exe
|-- proxmark3.exe
|-- *.dll
|-- resources/
|-- firmware/
|   |-- rdv4/
|   |-- rdv4-bt/
|   `-- generic/
|-- pm3-libs/
`-- platforms/
```

## Verified
- Windows 10 x64 with Proxmark3 USB + bundled client.
- Missing binary / console popup issues resolved.
- Serial port detection confirmed with heuristic scoring.
- All 272 unit tests pass.
- 20 Advanced tab panels functional.