# Phosphor 2.2 GUI - Release Notes

## v2.2.0 — PM3 v4.23346 Alignment (September 2026)

> **This build supersedes four earlier v2.2.0 uploads.** If you downloaded
> any earlier v2.2.0 build, **please re-download** — the 28 Sep one could not
> complete a scan, the 29 Sep one had no DESFire tools and a timeout bug on
> T55xx dictionary checks, the 30 Sep (earlier) one could not run any
> Advanced Mifare panel after a scan, and the build published earlier today
> **showed a blank UID for EM410x cards that also carry a T55xx block**, which
> broke scan-then-write.

### Fixed: EM410x cards that also carry a T55xx block showed a BLANK UID

**The most visible defect in this build.** Scanning an ordinary EM410x badge that
happens to *also* answer as a T55xx config block displayed an **empty UID**, and
because verification matches on UID, scan-then-write could never succeed.

Cause: `lf search` prints both lines for these cards —

```
[+] EM 410x ID 4E008E7AC1
[+] Valid EM410x ID found!
[+] Chipset... T55xx          <-- also present, in the "special cases" section
```

…and the parser tested for the T55xx chipset line **first** and returned
`uid: String::new()` unconditionally. The EM410x branch, which held the real
UID, was never reached. A T55xx config block is not a card type — plenty of
ordinary badges report one.

Standard card types are now evaluated **first**; the T55xx chipset is a
**fallback**, used only when nothing else matched. That fallback is retained so
a genuine bare T5577 is still detected rather than reported as "no card". When
the two coexist, the block is recorded so its config blocks are still read for
the clone payload.

Verified against a live capture from a real card (COM19, iCopy-X):

```
card_type = EM4100
uid       = 4E008E7AC1
clone cmd = lf em 410x clone --id 4E008E7AC1
```

### Fixed: T55xx config block was written FIRST instead of LAST

Block 0 is the **modulation / bit-rate config word**. Writing it before the data
blocks makes the chip re-modulate while those blocks are still incomplete, so
the remaining writes land wrong.

Order is now **data blocks 1..N ascending, config block 0 LAST**, matching the
iCopy-X open-source middleware (`lfwrite.write_raw`): *"Data blocks 1..N are
written FIRST, then config block 0 LAST. Block 0 sets modulation/bit-rate —
writing it last avoids the tag re-modulating mid-sequence while data blocks are
incomplete."*

The previous regression test asserted the inverted order, so it passed while the
defect was live; it has been replaced with three tests that check the real
ordering.

### Fixed: every Advanced Mifare panel failed with "device not connected"

After a successful scan, **every command in the MF View panel returned
`Error: device not connected`**. The Mifare tools only worked if you connected
to the reader and went straight to Advanced without scanning.

Cause: the panel resolved the serial port out of the wizard state enum, and
only the `DeviceConnected` state carries a port. Scanning moves the state to
`CardIdentified`, which has no port, so every handler failed. It now falls back
to the persistent connection record, matching the other 20 command modules
that already worked.

### Fixed: every T55xx card reported itself as blank — and could not be written

The most serious defect in this release. T5577 detection worked, but **every read
of a T55xx returned nothing**, so a card that was present and readable was
reported as *"blank"*, and writes to it silently did nothing.

Root cause: `lf t55xx read` and `lf t55xx dump` **only return data when
`lf t55xx detect` has already run in the same client process.** Detect is what
auto-detects the modulation, bit rate, offset and sequence-terminator flag, and
that state exists only inside the running client. Phosphor starts a fresh
`proxmark3.exe` for each command, so the state was destroyed between calls.

Measured on an iCopy-X with the same card, seconds apart:

| Command | Result |
|---|---|
| `lf t55xx read -b 0` | exit **-16**, empty table |
| `lf t55xx detect; lf t55xx read -b 0` | exit 0, `00 \| 000880E0` |
| `lf t55xx dump` | exit **0**, completely empty tables |
| `lf t55xx detect; lf t55xx dump` | exit 0, all 8 blocks read |

Three things made this hard to see:

- `lf t55xx dump` **exits 0 with empty tables**, which looked like a successful
  read of an empty card rather than a failure.
- Setting the modulation by hand (`lf t55xx config --ASK --rate 32 -o 33 --st`)
  did **not** help — the read still returned -16. Only the same-process chain works.
- The blank/unconfigured decision was being made from `lf search`, which never
  prints block contents, so it could not distinguish a blank card from a
  configured one.

All T55xx `read` / `write` / `dump` operations now run behind the required
`lf t55xx detect` prelude, and blank-vs-configured is decided by reading block 0
(all-zero means a factory blank).

T5577 detection reported the card correctly even before this fix — the card was
never the problem.

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

> ⚠️ **Phosphor does not update or flash an iCopy-X.** Updating an iCopy-X is
> an **IPK update on the device itself** (PC-Mode → transfer IPK → About →
> Update), not a Proxmark firmware flash from the desktop. Use
> `icopy-x-flash.ipk`, which ships Iceman v4.23346 (`CAPABILITIES_VERSION 11`)
> to match the client Phosphor bundles; `icopy-x-noflash.ipk` leaves the factory
> Proxmark in place and will not match. Full steps:
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
- `cargo test --lib` — **354 passed, 0 failed** (342 prior + 12 new)
- `npx tsc --noEmit` — clean
- `npm run tauri build` — SUCCESS (NSIS installer + portable ZIP)
- Device verified on v4.23346, **iCopy-X** (`hw version` reports
  `Iceman/master/v4.23346`, FPGA `fpga_icopyx_hf.ncd`)
- Live scan of an EM410x+T55xx test card (COM19): UID `4E008E7AC1` preserved
  through the parser, clone command `lf em 410x clone --id 4E008E7AC1`
- All 8 config blocks read live: `0=00148040 1=FFA7A004 2=7AFA6078 3-7=0`,
  and the generated write order confirmed block 0 last
- `hf mf autopwn` verified live against a MIFARE Classic 1K: 31 keys recovered,
  exit 0, 7 seconds, dump + key files written

### Release artifacts (current)

| Asset | Size | SHA-256 |
|-------|------|---------|
| `Phosphor_2.2.0_x64-setup.exe` | 65,750,788 | `9fc5ecab2d4fdab40a4aacb32d773aac4b6331bc0f48414d9ca8f5171282e242` |
| `Phosphor_2.2_GUI_v2.2.0_Windows_Portable.zip` | 111,446,002 | `e543644224fbf20dba46fd869654b683ea34688c68247c17fa37247eee08f563` |

- `phosphor.exe` inside both artifacts: 20,054,528 bytes,
  SHA-256 `b519e33be79499328e5364c554afed15a94f42936382150ddf52fc470edaf7d9`
- Bundled PM3 client: SHA-256 prefix `F7BA073E30F6` — the hardware-verified
  build, `CAPABILITIES_VERSION 11`. A locally rebuilt client (prefix
  `B444910108AD`) fails against real hardware with
  `Received packet frame with invalid CRC` and is never packaged.
  `build_portable.ps1` aborts rather than package an unverified client.
  Note: the client must be launched from its own directory, or it fails to
  start with `0xC0000139` (missing DLL entry point). Phosphor does this
  correctly; it only matters when running `proxmark3.exe` by hand.
- Source: commit `12db82f`, tagged `v2.2.0`.

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
- All 338 unit tests pass.
- 21 Advanced tab panels functional.
- Verified live on an iCopy-X: MIFARE Classic 1K (SAK 08) detected, `hf search`,
  `hf mf autopwn` and the MF View panel all working.
