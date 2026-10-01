# Phosphor 2.2 GUI - Release Notes

## v2.2.0 — PM3 v4.23346 Alignment (September 2026)

> **This build supersedes three earlier v2.2.0 uploads.** If you downloaded
> any earlier v2.2.0 build, **please re-download** — the 28 Sep one could not
> complete a scan, the 29 Sep one had no DESFire tools and a timeout bug on
> T55xx dictionary checks, and the 30 Sep (earlier) one could not run any
> Advanced Mifare panel after a scan.

### Fixed: every Advanced Mifare panel failed with "device not connected"

After a successful scan, **every command in the MF View panel returned
`Error: device not connected`**. The Mifare tools only worked if you connected
to the reader and went straight to Advanced without scanning.

Cause: the panel resolved the serial port out of the wizard state enum, and
only the `DeviceConnected` state carries a port. Scanning moves the state to
`CardIdentified`, which has no port, so every handler failed. It now falls back
to the persistent connection record, matching the other 20 command modules
that already worked.

### Fixed: a missing card was reported as a blank success

> **Note:** the first attempt at this fix, included in the 30 Sep (earlier)
> build, **did not work**. It is described here because the root cause is
> non-obvious and worth recording.

Proxmark3 signals "nothing found" inconsistently. `hf search` exits non-zero,
but `hf 14a info`, `hf mf info` and `hf mfu info` **exit 0 having printed
nothing but the connection banner**. Those reached the interface as an empty,
success-looking result — indistinguishable from the app being broken, which
made a card sitting outside the reader's coil look like a dead reader.

Those commands now report:

```
No tag detected running: hf mf info. Place the card on the reader and try again.
```

Device-level commands such as `hw version` are deliberately excluded, because
some of them legitimately print nothing.

The earlier build failed for two independent reasons:

1. **It was never called in a released app.** The check lived in one code path,
   but Phosphor resolves the bundled `proxmark3.exe` from its own folder first,
   so nearly every command arrived via a *different* path that skipped the check
   entirely. All spawn paths now share one result interpreter, so this class of
   bug cannot recur.
2. **The banner test could not match real output.** Phosphor always passes `-f`
   to PM3, which makes the client print an extra
   `[=] Output will be flushed after every print.` line on every invocation.
   The banner matcher did not know about that line, so it always returned false.
   Its unit test had passed because the test used a hand-written banner that
   the client never actually emits — the fixture has been replaced with a real
   captured one.

Verified on hardware: **Advanced → Read ISO14443-B Tag** with no card now
reports *"No tag detected running: hf 14b info. Place the card on the reader and
try again."*

### Fixed: "no device" was reported as "proxmark3 is not installed"

When no Proxmark3 answered the probe, the app said *"Proxmark3 binary not
found. Ensure proxmark3 is installed and in your PATH."* — advice that is
wrong when the client is bundled next to `phosphor.exe`. It matched on
substrings of the error text, and the "no device answered" message contains the
words "not found". It now distinguishes the two cases and, for devices with a
PC mode such as the iCopy-X, tells you to switch to PC mode first.

### Fixed: concurrent commands could collide on the serial port

Only one `proxmark3.exe` can hold a COM port at a time. Phosphor's state lock
guarded the port *string*, not the subprocess, so two overlapping commands could
launch two clients against one port and produce *"invalid serial port"*. Spawns
are now serialised process-wide.

### Fixed: a timeout silently started a second client on the same port

If the bundled client timed out, the app treated that as "client not found" and
fell through to launch the same command again against the next candidate
binary — on the same port, while the first client still held it. Timeouts are
now reported instead of retried.

### Fixed: MIFARE autopwn was killed after 30 seconds from the panel

The dictionary-attack timeout list did not include `hf mf autopwn`, so the MF
View panel gave it the default 30-second budget while the clone panel gave the
same command an hour. The panel version was guaranteed to be killed
mid-attack. `hf mf autopwn` and `lf iclass brute` are now routed to the long
timeout.

### Fixed: Proxmark3 error messages were sometimes misleading

Exit codes were mapped to messages that contradicted PM3's own definitions — a
real timeout was reported as *"Invalid argument"*, which blamed you for a bad
command. All codes are now transcribed from PM3's `pm3_cmd.h`. In particular,
`-5` is PM3's generic *"found nothing"* result, not "aborted by the device", so
Read ISO14443-B Tag now correctly says **no tag**.

Errors written to stderr are also surfaced. PM3 sends every `[!!]` line to
stderr, so a command that failed with no stdout at all used to look like a
blank success.

### Verified on hardware: iCopy-X

Tested against a physical **iCopy-X** running the lab-401 **icopy-x v1.1.6**
firmware, on COM19 over USB CDC.

> ⚠️ Flashing an iCopy-X is not done from Phosphor — the bundled images do not
> include PM3ICOPYX firmware and Phosphor does not detect iCopy-X. Use the
> official method:
> [lab-401/icopy-x releases — v1.1.6](https://github.com/lab-401/icopy-x/releases/tag/v1.1.6)

### New: DESFire Advanced panel (full `hf mfdes`)

A dedicated DESFire tab covering all **57** `hf mfdes` subcommands from
v4.23346 — not just the handful that were previously reachable.

- **Card info** — `info`, `getversion`, `getuid`, `freemem`, `mad`
- **Applications** — `lsapp`, `getaids`, `getappnames`, `selectapp`,
  `createapp`, `deleteapp`, `createdelegateapp`, `getdelegateappinfo`,
  `selectisofid`
- **Files** — `lsfiles`, `getfileids`, `getfileisoids`, `getfilesettings`,
  `chfilesettings`, `createfile`, `createvaluefile`, `createrecordfile`,
  `createmacfile`, `clearrecfile`, `deletefile`
- **Data** — `read`, `write`, `value`, `changekey`, `auth`, `chkeysettings`,
  `getkeysettings`, `getkeyversions`
- **Dumping and key recovery** — `dump`, `detect`, `chk`, `default`
- **Emulator (self-contained DESFire EV1 simulation)** — `eload`, `esave`,
  `eview`, `view`, `sim`, `etest` (host-driven, no RF), `list`
- **Card config** — `setconfig`, `pc`, `formatpicc`
- **Brute force** — `bruteaid`, `bruteisofid`, `brutedamslot`
- **DUOX / DESFire Light** — `verifycert`, `intauth`, `vdesign`
- **MFC transfer** — `makemfclicense`, `createmfcmapping`

**Correctness note.** Every option name was taken from the client's own
`--help` output, and 36 generated command shapes were executed against real
v4.23346 hardware/software to confirm they parse. Two traps were found and
handled:

- The shared authentication flags are **not** uniform. The client rejects a
  command outright with `invalid option` for any flag its subcommand does not
  declare — `lsapp` has no `--aid`, `default` has neither `--aid` nor
  `--no-auth`, `createdelegateapp` has no `-n`, `selectisofid` spells it
  `--isofid` rather than `--isoid`, and `chkeysettings` has no `--isoid`.
  Each subcommand has its own verified allowlist.
- **Key length is in bytes, not hex characters.** The help says
  "8|16|24 hex bytes", but `desfire_get_key_length()` returns bytes
  (DES = 8, 2TDEA = 16, 3TDEA = 24, AES = 16), so a valid key is 16, 32 or
  48 hex characters.

### Fixed: T55xx dictionary check timed out and broke the serial port

`lf t55xx chk` measured **26.8 s** against the 30 s default command timeout —
a 3-second margin. On a slower run it hit the cap, the Proxmark3 child was
killed mid-attack **while still holding COM24**, and the next command then
failed with `invalid serial port`. The app reported that as a device error
rather than a result.

Brute-force and dictionary-attack commands (`lf t55xx chk`/`bruteforce`,
`hf 15 bruteforce`, `hf mf bruteforce`, `hf mfu desbrute`, `hf mfdes chk`,
`hf mfdes bruteaid`, `hf mfdes bruteisofid`, `hf mfdes brutedamslot`) now get
a **15-minute** budget. Ordinary probes keep the short timeout.

### Improved: readable PM3 errors instead of exit-code integers

Commands previously surfaced raw numbers such as
`PM3 command failed: Exit code -20: hf mfdes getversion`. The documented
`#define PM3_E*` codes from `pm3_cmd.h` are now mapped to plain language, so
-20 reads `APDU exchange failed … Is a card of the expected type on the
reader?`, -23 `Card left the field (tear-off)`, -28 `No key available`, and
so on.

### ⚠ Earlier critical scan and write fixes (29 Sep 2026)
- **Scanning was broken for every card.** `lf search` and `hf search` are full
  sweeps that take ~10-11 seconds, but the app was timing them out at 8 seconds.
  The interrupted Proxmark3 process was not terminated and kept holding the
  serial port, so the next command failed with `invalid serial port`. The app
  then reported this as "No card found" even though the card was on the reader.
  The search timeout is now 30 seconds and the child process is killed on
  timeout, so the port is always released.
- **A dropped connection is now reported as a connection error**, not as
  "No card found". You will get a clear "could not open the serial port" message
  telling you to reconnect, instead of being told to place a card that is
  already there.
- **T55xx cards are writable again.** T55xx was incorrectly flagged as
  non-cloneable, so a detected T55xx showed no WRITE button. Scanning a T55xx
  now reads its configuration blocks and WRITE copies them onto a T5577 blank,
  block by block, with verification after each write.

### Foundation: Iceman fork v4.23346 ("Frosty Lemon")
- Upgraded the bundled Proxmark3 client to the Iceman fork v4.23346.
- Rebuilt all 3 firmware images (rdv4, rdv4-bt, generic) from v4.23346 source.
- **CAPABILITIES_VERSION is now 11** — client and firmware must be flashed as a
  matched pair or they refuse to talk.

### Command alignment (v4.23346)
- `hf mf cchk` + `hf mf aeschk` → merged into `hf mf chk`.
- `hf mfdes chk` runs with no arguments by default; all of its flags are now
  exposed in the DESFire panel.
- `hf 14b valid` removed.

### Advanced tab — 21 panels (up from 14)

| Tab | Commands |
|-----|----------|
| 14B | info, rdbl, ctrdbl, view --selftest, view -f (MyKey/COGES) |
| 15693 | info |
| Felica | info, sim from dump |
| iCLASS | info, legbrute |
| LEGIC | info |
| **DESFire** | **all 57 `hf mfdes` subcommands — see above** |
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
- **13 COMPLETE** — all commands from the unreleased CHANGELOG that exist in
  v4.23346 are now exposed in the GUI. `hf mfdes chk` moved from PARTIAL to
  COMPLETE with the DESFire panel.
- **2 PARTIAL** — `nfc encode` (record types only), `hf mfu ndefwrite`
  (record types only)
- **4 N/A** — BWM BLE/WiFi/power (not in v4.23346), `hw powersave` (not in
  v4.23346), Flipper Zero link (firmware test scaffolding only, no CLI),
  ePassport (Kivy 60-100 MB, deferred)

### Build environment
- ARM cross-compiler: `arm-none-eabi-gcc` 10.3.1 (ProxSpace toolchain).
- PM3 client built with `make client IS_MINGW=1 LUAPLATFORM=mingw`.
- Firmware built with `make fullimage PLATFORM=PM3RDV4 [PLATFORM_EXTRAS=...]`.

### Verification
- `cargo test --lib` — **328 passed, 0 failed** (324 prior + 4 new)
- `npx tsc --noEmit` — clean
- `npm run tauri build` — SUCCESS (NSIS installer + portable ZIP)
- Device verified on v4.23346, **iCopy-X** (`hw version` reports
  `Iceman/master/v4.23346`, FPGA `fpga_icopyx_hf.ncd`)
- `hf mf autopwn` verified live against a MIFARE Classic 1K: 31 keys recovered,
  exit 0, 7 seconds, dump + key files written

### Release artifacts (current)

| Asset | Size | SHA-256 |
|-------|------|---------|
| `Phosphor_2.2.0_x64-setup.exe` | 65,696,804 | `b5f18f045d32e8eb40e50ae3d7341386ec7ee93995793aff1a8b5fd4b1075ef7` |
| `Phosphor_2.2_GUI_v2.2.0_Windows_Portable.zip` | 111,430,479 | `5e64ca0f45058eda7bc17cfd3c818f5278fe55e7b4af51a824b6a96a261b673d` |

- `phosphor.exe` inside both artifacts: 19,999,744 bytes,
  SHA-256 `6bc1a026abeeb76c6a7a6cf1e8d5fa425d7a0ccb5ed1e0ecbe7791896352591b`
- Bundled PM3 client: SHA-256 prefix `F7BA073E30F6` — the hardware-verified
  build, `CAPABILITIES_VERSION 11`. A locally rebuilt client (prefix
  `B444910108AD`) fails against real hardware with
  `Received packet frame with invalid CRC` and is never packaged.
  `build_portable.ps1` aborts rather than package an unverified client.
- Source: commit `6fa272a`, tagged `v2.2.0`.

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
- **No tag found**: Proxmark3 reports absence inconsistently. `hf search` exits
  non-zero, while `hf 14a info` / `hf mf info` / `hf mfu info` exit 0 having
  printed only the connection banner. Phosphor now detects the banner-only case
  and reports *"No tag detected"* instead of showing a blank result.
- **Card placement matters**: on a small HF coil (notably the iCopy-X) a card
  can read and then stop reading as it is nudged. If a card is detected by
  `hf search` but a follow-up command reports no tag, re-seat the card. This is
  coupling, not a fault.
- **Wrong card on the reader**: a DESFire command against a MIFARE Classic (or
  vice versa) fails with a clear card-exchange or APDU error. That is the
  reader reporting a mismatch, not a bug.
- **`lf tune` syntax**: the command takes no positional argument. Do not append a value.
- **`hw tune`**: informational only; it does not actively tune the antenna.
- **iCLASS**: some commands hang without a built-in timeout. If a command appears frozen, cancel the connection and retry.
- **`smart pps`**: requires RDV4 with smartcard module. Not available on PM5.
- **`hf emrtd`**: online modes need an ISO 14443 tag; `hf emrtd test` and `hf emrtd list` work offline.
- **`hf 14a antifuzz`**: fuzzes reader anticollision. Use `--coll` for collision storm mode.
- **`hf mfdes sim` / `etest`**: need `hf mfdes eload` first. `etest` drives the
  emulator from the host over USB with no RF involved.

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
- All 328 unit tests pass.
- 21 Advanced tab panels functional.
- Verified live on an iCopy-X: MIFARE Classic 1K (SAK 08) detected, `hf search`,
  `hf mf autopwn` and the MF View panel all working.
