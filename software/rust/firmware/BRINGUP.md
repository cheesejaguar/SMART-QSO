# Firmware Bring-Up Checklist (STM32L4R5ZI)

First hardware run of the Embassy firmware in this directory. No board has run the image yet: CI builds it and checks
the linked image, and everything below was prepared without hardware. Where this checklist and
`software/flight/tests/hil/HIL_TEST_PLAN.md` differ, the HIL plan's pass criteria govern; this list gives the order and
the commands for the first day.

Status of the information here:

- **Checked against the code** in this repository (`src/*.rs`, `sqso-ffi`): pins, baud rates, symbols, boot order,
  console lines.
- **Checked against the probe-rs 0.32.0 source** (crates `probe-rs` and `probe-rs-tools`): command options, output
  format, what a session changes when it ends.
- **Checked against ST's STM32L4R5 SVD** (and stm32-metapac 21), not RM0432 itself: RCC_CSR, DBGMCU_APB1FZR1.
- **Checked against the Zephyr board file for the NUCLEO-L4R5ZI**, not UM2179 itself: the board's VCP, LED and button
  pins. Items marked *(UM2179)* below are still to be checked against the board's user manual and schematic (MB1312).

## 1. Equipment

| Item | Notes |
|------|-------|
| Board | NUCLEO-L4R5ZI (expected dev board). A NUCLEO-L4R5ZI-P carries the STM32L4R5ZITxP (SMPS variant): pass `--chip STM32L4R5ZITxP` to probe-rs and the scripts. Or the OBC board, when one exists |
| Probe | The NUCLEO's on-board ST-LINK (SWD). For the OBC board, any SWD probe probe-rs supports (ST-LINK, J-Link, CMSIS-DAP) on SWDIO, SWCLK, NRST, GND and VTref |
| USB-serial adapters | Two, 3.3 V logic (not RS-232, not 5 V): one for the console (USART3), one moved between the Jetson link (USART1) and the TNC port (USART2) |
| Bench supply | Current-limited, for the OBC board (§3). The NUCLEO is powered from its ST-LINK USB port |
| Ammeter | In the NUCLEO's IDD jumper *(UM2179)*, or in series with the OBC board's 3.3 V |
| Optional | FM25V20A FRAM breakout (§5, step 6); logic analyzer |

Host software:

| Tool | Version | Use |
|------|---------|-----|
| Rust | pinned by `software/rust/rust-toolchain.toml` (rustup installs it) | `cargo build --release` |
| probe-rs-tools | 0.32.0 (`cargo install probe-rs-tools --version 0.32.0 --locked`; on Linux, the probe-rs udev rules) | flashing, `probe-rs read`/`write` |
| GNU Arm binutils | any (`arm-none-eabi-nm`, `-objcopy`, `-size`) | symbol addresses, `scripts/release_package.sh` |
| Python | 3.8 or later, standard library only (pytest for the scripts' tests) | `tools/*.py` |
| Serial terminal | e.g. `picocom -b 115200 --logfile console.log /dev/ttyUSB0` | console capture |

## 2. NUCLEO-L4R5ZI and the flight pin map

The pin map in `src/gpio.rs`, `src/main.rs` and the driver files is provisional (TBR until the OBC schematic baselines a
pinout) and was written for the OBC, not for the NUCLEO. On the NUCLEO:

| Function (flight map) | Pins | On the NUCLEO-L4R5ZI | Consequence |
|-----------------------|------|----------------------|-------------|
| Console, USART3 | PD8 TX, PD9 RX, 115200 8N1 | The ST-LINK virtual COM port is on LPUART1 (PG7 TX, PG8 RX), which the firmware does not use | The console is **not** on the ST-LINK USB port. Wire a 3.3 V adapter: PD8 to the adapter's RX, PD9 to its TX, GND to GND (header pins *(UM2179)*) |
| Jetson link, USART1 | PA9 TX, PA10 RX, 115200 | PA9/PA10 may be wired to the USB OTG connector (VBUS sense, ID) *(UM2179)* | Check before wiring the adapter; leave the OTG connector unplugged |
| TNC, USART2 | PD5 TX, PD6 RX, 9600 | free *(UM2179)* | Uplink frames in and downlink frames out (KISS). With no TNC, a logic analyzer on PD5 shows each beacon: the five TNC parameter commands, then the frame (HIL-030) |
| ALERT input (pull-up) | PB7 | LD2 (blue LED) | The LED loads the input: LD2 may glow and the ALERT level read is not meaningful |
| SPI2 MISO (RF) | PB14 | LD3 (red LED) | Nothing is on SPI2 on the NUCLEO; LD3 shows SPI2 activity, if any |
| Status LED | PD3 | not an LED | No visible status LED; add an LED and resistor on PD3, or watch it on the logic analyzer |
| — | PC7 (LD1, green), PC13 (B1, user button) | not used by the firmware | LD1 stays off; B1 does nothing. B2 (reset) gives a pin reset |
| FRAM, SPI1 | PA5 SCK, PA6 MISO, PA7 MOSI, PA4 CS#, PE0 WP#, PE1 HOLD# | no FRAM fitted | See §5 step 6 |
| Enable and burn lines | PB4, PB5, PB6, PD2, PA1, PA2, PA3, PB9, PB10, and the downlink power select PE3 (outputs, low at boot) | header pins | Nothing to drive on the NUCLEO. On an OBC board these switch loads and burn wires: check each against the schematic before the first power-on |
| ADC inputs | PC2, PC3, PC4, PC5, PB0, PB1, and the downlink monitors PA0 (TX_TEMP), PC0 (TX_FWD), PC1 (TX_REV) | header pins, floating | Readings are meaningless until the EPS sense circuits and the transmitter exist (front-end scaling TBR, `src/adc.rs`). Floating monitors can trip the downlink's protections: tie PA0, PC0 and PC1 to GND, so the downlink sees a cool transmitter and no output (it reports "keyed with no forward power" once per beacon) |

The firmware runs from the internal MSI through the PLL (80 MHz, `clocks()` in `src/main.rs`); the board's HSE (ST-LINK
MCO) and 32.768 kHz crystal are not used.

## 3. Power

- **NUCLEO.** Power it from the ST-LINK USB port with the default power jumper setting *(UM2179)*. For a current
  reading, put the ammeter in the IDD jumper *(UM2179)*. `hardware/obc/OBC_DESIGN.md` §2.3 gives 15 mA in run mode at
  80 MHz (not yet checked against the STM32L4R5 datasheet); the core sleeps in the executor between 20 ms passes, so
  the average should be lower. Record what you measure.
- **OBC board (bench supply).** 3.3 V, current limit 50 mA for the first power-on, with the probe connected but not
  powering the target (VTref sense only). Turn on with the probe attached, read the current, and turn off if it exceeds
  30 mA before anything has been flashed (an erased part draws little). Raise the limit to 100 mA once the image runs
  and the draw is understood. These limits are a bring-up choice, not a design value; revisit them when the OBC
  schematic and its loads are known.

## 4. Build, package and flash

Commands here and in §5 run in `software/rust/firmware`.

```bash
cargo build --release
# The package records what is flashed: the ELF, build_info.txt, stack_usage.txt, SHA256SUMS
PKG=../../../build/bringup-$(date +%Y%m%d)
../../../scripts/release_package.sh --dev "$PKG"
ELF=$(ls "$PKG"/sqso-firmware-*.elf)
# First flash: probe-rs run --chip STM32L4R5ZITx (the runner in .cargo/config.toml)
cargo run --release
```

Before the first flash of a board, set SRAM2 parity on (option bit `SRAM2_PE` = 0; the flight software never writes
option bytes): `STM32_Programmer_CLI -c port=SWD mode=UR -ob SRAM2_PE=0`, then `-ob displ` must list
`SRAM2_PE : 0x0` (`docs/development/RELEASE_PROCEDURE.md` §5.3.1). The console then shows `[SYSTEM] SRAM2 parity on`
at every boot; "SRAM2 parity OFF (option bit SRAM2_PE)" means the step was missed.

`cargo run --release` flashes, resets and runs the image, then keeps the probe attached until Ctrl-C. The image has no
RTT or defmt channel, so the runner prints nothing from the firmware: the console is the USART3 adapter (§2). In
probe-rs 0.32.0 the runner also arms vector catches (HardFault, core reset) and does not clear them when it exits, and
on the STM32L4 the ending session leaves DEMCR as it is (from the probe-rs source). A later HardFault then halts the
core instead of running the firmware's handler, and a reset halts it at the reset vector. So:

- after the runner, **power-cycle the board** (or unplug the ST-LINK USB) before the watchdog and guard tests, or flash
  with `probe-rs download --chip STM32L4R5ZITx <elf>` and `probe-rs reset --chip STM32L4R5ZITx` instead;
- `tools/guard_test.py` and the HIL-003 commands below write DEMCR themselves (TRCENA only, no vector catch).

Flash the packaged ELF (`probe-rs download --chip STM32L4R5ZITx "$ELF"`, then `probe-rs reset --chip STM32L4R5ZITx`)
for the recorded runs, so the record matches the image. `cargo run --release` flashes the same build if nothing changed
in between.

## 5. First checks, in order

The scripts are in `tools/` (§6). Each prints PASS or FAIL lines and exits 0 (pass), 1 (fail) or 2 (error: no probe,
unreadable input). `$ELF` and `$PKG` are those of §4.

| # | Check | How | Pass | HIL |
|---|-------|-----|------|-----|
| 1 | Probe and part | `probe-rs list`; `probe-rs info --chip STM32L4R5ZITx` | The probe is listed; the part answers on SWD (ARMv7-M core) | — |
| 2 | Boot to the console banner | Start the console capture, then power-cycle the board (or press B2) and time it | `[SYSTEM] SMART-QSO Flight Software Started`, then `[SYSTEM] Version: 0.2.0-dev` and the mission phase, power mode and SOC line, within 30 s of power-on (SRS-F001) | HIL-001 |
| 3 | Watchdog running and kicked | `arm-none-eabi-nm -S -C "$ELF" \| grep sqso_firmware::wdt::`; then `probe-rs read --chip STM32L4R5ZITx b32 <TIMEOUT_MS> 1` and `probe-rs read --chip STM32L4R5ZITx b8 <RUNNING> 1`; `probe-rs read --chip STM32L4R5ZITx b32 <LAST_KICK_MS> 2` five times, about 0.5 s apart | TIMEOUT_MS = 0x7530 (30,000 ms, the operating timeout); RUNNING = 01; LAST_KICK_MS (milliseconds since boot, low word first) changes between every two reads | HIL-043 |
| 4 | Loop timing | `tools/loop_timing.py --elf "$ELF" --samples 5 --interval 10` (after at least a minute of running) | Longest pass ≤ 20 ms, no overruns, `passes` grows between samples | HIL-040 |
| 5 | Stack high-water mark | `tools/stack_high_water.py --stack-usage "$PKG"/stack_usage.txt --elf "$ELF" --save stack_dump.txt` | Used ≤ the static worst case of the image (5,004 B on 2026-10-25) and < 8,192 B | HIL-044 |
| 6 | FRAM read and write | Wire an FM25V20A at 3.3 V to SPI1 (§2), then reset three times, at least 5 s after each banner (the mission data is saved every 100 passes, 2 s) | The first boot on a blank part shows `[MISSION] Initialized new mission data`; each later boot shows `[MISSION] Loaded mission data (reset #n)` with n one higher than the boot before (the reset count is read from and written to the FRAM) | HIL-002 (part) |
| 7 | Jetson link, USART1 | Adapter on PA9/PA10 at 115200 8N1 (§2) | The OBC sends `TELEMETRY` and `BEACON_REQ` lines in the `docs/interfaces/ICD_OBC_JETSON.md` §4 format with valid CRCs; a `BEACON_RSP` with the request's sequence number and a valid CRC, typed back, gives `[UART] Beacon text from Jetson (N chars)` on the console | HIL-032 (part) |
| 8 | KISS frames, USART2 | Adapter on PD5/PD6 at 9600 8N1; build a frame with `software/ground/tools/telecommand_builder.py <command> --key-file <throwaway key> --counter 1 --ax25 <spacecraft call> <station call> --kiss --out frame.bin`, then `stty -F /dev/ttyUSB1 9600 raw && cat frame.bin > /dev/ttyUSB1` | With no key provisioned the command is refused: a `[FAULT]` line with the rejection reason appears on the console (the first of a run of rejections is logged). Executing a command (HIL-031 proper) needs a provisioned key over the GSE port | HIL-031 (part) |
| 9 | Loop timing and stack again | Steps 4 and 5 again after steps 7 and 8, before any reset (a reset repaints the budget and zeroes the timing). Step 6 reset the board last, so this covers the boot and steps 7-8 | As steps 4 and 5 | HIL-040, HIL-044 |
| 10 | Watchdog timeout | The HIL-003 commands below | The processor resets within 30 s of the halt; the next boot's reset cause is WATCHDOG (2) | HIL-003 |
| 11 | Stack guard | Power-cycle first if the runner was used (§4); `tools/guard_test.py --elf "$ELF"` after a minute of running | MemManage in the guard (CFSR.MSTKERR, or DACCVIOL with MMFAR in 0x2009DFE0-0x2009DFFF), then a reset, then reset cause WATCHDOG (2) | HIL-004 |

Notes on the steps:

- **Step 2.** The console port is ready from bring-up (`src/uart.rs`, `install`), so the lines printed before
  `gse_init()` (the `[RF_SILENCE] state=…` line, any `GPIO/flash init failed`, fault-management or deployment line)
  are output too. On the NUCLEO, with no FRAM, the flash initialization fails and the boot continues. Per HIL-001 the
  mode manager goes from BOOT through DETUMBLE to IDLE at once (no ADCS yet), or to SAFE on low SOC; record the mode
  the console reports.
- **Step 4.** `SQSO_LOOP_TIMING` (`src/timing.rs`) holds the last and longest `Executive::step()` pass in core cycles
  (80 MHz), the passes over 20 ms and the passes recorded, counted from boot (step 9 reads it again after the
  interface checks). The cycle counter stops while the core is halted.
- **Step 5.** The script reads 0x2009E000-0x2009FFFF without halting the core and computes used = 8192 − 4 × the
  words still painted with 0x57ACC0DE from the bottom (`src/stack.rs`). A result above the static worst case means the
  static analysis (`scripts/stack_usage.py`) missed a path: stop and report it.
- **Step 7.** The OBC sends to the Jetson only while the EPS logic has the payload enabled and the telemetry interval
  has passed. On a bench without the EPS on I2C2 the payload may stay off: record what is sent. The line format and CRC
  can be checked with `software/payload/obc_link.py`, which needs the `sqso_protocol` module (`software/rust/sqso-py`,
  maturin), as does the telecommand builder in step 8.
- **Step 10 (HIL-003).** Halt the core with the IWDG running, then let it expire. Do not run `probe-rs read` while the
  core is halted: in probe-rs 0.32.0 a read resumes a halted core when it ends.

  ```bash
  C="--chip STM32L4R5ZITx"
  probe-rs write $C b32 0xE000EDFC 0x01000000   # DEMCR: TRCENA only, no vector catch
  probe-rs write $C b32 0xE0042008 0x00000000   # DBGMCU_APB1FZR1: the IWDG runs while the core is halted
  probe-rs write $C b32 0xE000EDF0 0xA05F0003   # DHCSR: halt
  # within 30 s the IWDG resets the processor (time it)
  probe-rs write $C b32 0xE000EDF0 0xA05F0001   # DHCSR: run, in case the core is still halted after the reset
  probe-rs read  $C b32 <RESET_CAUSE> 1         # 00000002 = WATCHDOG (hal_reset.h); address from nm -C
  ```

  Whether the core comes out of the reset halted (C_HALT kept over a system reset) was not checked; the run write
  covers both cases. The console then shows the banner again, and telemetry lines to the Jetson carry
  `WATCHDOG=TRIGGERED`.
- **Step 11 (HIL-004).** `tools/guard_test.py` halts the core, sets SP to 0x2009E010 (16 bytes above the guard) and
  resumes it, with the IWDG frozen while the core is halted (DBGMCU_APB1FZR1.DBG_IWDG_STOP, restored at the end), so
  the halt itself cannot expire it. It then polls the fault registers until the processor resets and reads the reset
  cause the firmware cached at boot (`RESET_CAUSE` in `src/wdt.rs`). It does not read RCC_CSR.IWDGRSTF for the verdict:
  `hal_reset_cause()` clears the RCC_CSR reset flags at boot (RMVF), so by the time the probe can read them they are
  gone; RCC_CSR is printed for the record. `tools/guard_test.py --dry-run` prints the probe-rs commands against a
  simulated board.

## 6. The probe scripts

| Script | HIL | Reads | Fails when |
|--------|-----|-------|------------|
| `tools/stack_high_water.py` | HIL-044 | The 8 KB budget (2,048 words); the static worst case from `stack_usage.txt` | Used > static worst case, or no painted word left |
| `tools/loop_timing.py` | HIL-040 | `SQSO_LOOP_TIMING` (address from the ELF with nm) | Longest pass > 20 ms, overruns, no passes, `passes` not growing or falling |
| `tools/guard_test.py` | HIL-004 | `SQSO_LOOP_TIMING`, `RESET_CAUSE`, CFSR/HFSR/MMFAR, DHCSR; writes DHCSR, DCRDR, DCRSR, DEMCR, DBGMCU_APB1FZR1 | No guard fault, no reset, or a reset cause other than WATCHDOG |

They call the `probe-rs` command line (`read` and `write` only) and `nm`; shared code is in `tools/sqso_probe.py`,
which states the probe-rs assumptions. Common options: `--chip`, `--probe VID:PID[:SERIAL]`, `--protocol`, `--speed`,
`--probe-rs PATH`, `--nm PATH`. Without a board, `stack_high_water.py --dump` and `loop_timing.py --dump` evaluate saved
`probe-rs read` output, and `guard_test.py --dry-run` runs against a simulated board. Unit tests:

```bash
python3 -m pytest -q software/rust/firmware/tools/tests
```

## 7. What to record

For every session, in the test record (`HIL_TEST_PLAN.md` §4.3):

- the package directory from `scripts/release_package.sh --dev`: `build_info.txt` (commit, toolchain, lock-file hashes,
  sizes, stack worst case), `stack_usage.txt`, `SHA256SUMS` and the ELF that was flashed;
- the board (type and revision, serial number, the chip's marking), the probe (type, serial, firmware), the host OS,
  and `probe-rs --version`;
- the wiring: adapters and the pins they are on, the FRAM breakout, and every departure from §2;
- supply voltage and current (idle, during the main loop), and the current limit used;
- the whole console capture of the session;
- the output of each script, and the `stack_dump.txt` of steps 5 and 9 (give them different names);
- per check: pass or fail, the measured value (boot time, longest pass and overruns, stack used and margin, reset
  delay), and anything unexpected.

## 8. Provisional

- The pin map (`src/gpio.rs` and the drivers), until the OBC schematic baselines a pinout;
  `hardware/obc/OBC_DESIGN.md` and `hardware/obc/STORAGE_DESIGN.md` disagree on PA4-PA7 and PC0-PC1.
- The NUCLEO details marked *(UM2179)*.
- The ADC front-end scaling (`src/adc.rs`).
- The current limits in §3.
