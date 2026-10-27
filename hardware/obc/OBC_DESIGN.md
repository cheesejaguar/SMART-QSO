# On-Board Computer (OBC) Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-CDH-OBC-001
**Version:** 1.7
**Date:** 2026-10-27
**Status:** Draft
**Supersedes:** `docs/hardware/OBC_DESIGN.md` (SMART-QSO-OBC-001), merged as Appendix A (finding PI-17)

---

## 1. Overview

This document defines the On-Board Computer (OBC) design for the SMART-QSO 1U CubeSat, following NASA Ames SmallSat heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat) and NPR 7150.2 software engineering requirements.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Processing capability | 32-bit ARM Cortex-M4F | Mission |
| Clock speed | ≥ 80 MHz | Performance |
| RAM | ≥ 256 KB | Software |
| Flash | ≥ 1 MB | Software |
| Power consumption | < 100 mW active | EPS Budget |
| Operating temperature | -40°C to +85°C | Environment |

---

## 2. Microcontroller Selection

### 2.1 Selected MCU

| Parameter | Value |
|-----------|-------|
| Manufacturer | STMicroelectronics |
| Part Number | STM32L4R5ZI (`docs/BASELINE_PARAMETERS.md` §6); STM32L4R5ZIT6 in `docs/manufacturing/EEE_PARTS_LIST.md` §4.1 |
| Core | ARM Cortex-M4F |
| Clock Speed | Up to 120 MHz in voltage Range 1 boost mode (ST STM32L4R5xx datasheet). The flight image runs at 80 MHz, the top of Range 1 normal mode, from the internal MSI through the PLL (`clocks()` in `software/rust/firmware/src/main.rs`) |
| Flash | 2 MB |
| SRAM | 640 KB: SRAM1 192 KB at 0x20000000, SRAM2 64 KB at 0x20030000, SRAM3 384 KB at 0x20040000, contiguous. The firmware links them as one 640 KB region (the `memory.x` of embassy-stm32 with cortex-m-rt's `link.x`; `software/rust/firmware/.cargo/config.toml`) |
| Package | LQFP144 (STM32L4R5ZIT6, `docs/manufacturing/EEE_PARTS_LIST.md` §4.1) |

The baseline OBC is the **STM32L4R5ZI** (`docs/BASELINE_PARAMETERS.md` §6). Version 1.1 of this table gave the
superseded STM32L476RG or Apollo4 Blue (1 MB flash; 128 KB or 2 MB SRAM; 80 or 96 MHz). The memory (§3) and clock (§4)
data now follow the STM32L4R5ZI. The currents in §2.3 and §6.3 are still the earlier figures and have not been checked
against the STM32L4R5 datasheet.

### 2.2 Selection Justification

- Ultra-low power operation (< 50 µA/MHz active)
- Extensive peripheral set (UART, SPI, I2C, ADC)
- Hardware FPU for attitude calculations
- Flight heritage on similar CubeSat missions
- Industrial temperature grade, -40°C to +85°C (STM32L4R5ZIT6, `docs/manufacturing/EEE_PARTS_LIST.md` §4.1)
- Radiation tolerance adequate for LEO (TID > 10 krad with latchup immunity)

### 2.3 Operating Modes

| Mode | Clock | Current | Use Case |
|------|-------|---------|----------|
| Run | 80 MHz | 15 mA | Active processing |
| Low Power Run | 2 MHz | 1 mA | Idle operations |
| Sleep | - | 200 µA | Waiting for event |
| Stop | - | 5 µA | Deep sleep |

---

## 3. Memory Architecture

### 3.1 Memory Map

| Region | Start | Size | Purpose |
|--------|-------|------|---------|
| Flash (Code) | 0x08000000 | 512 KB | Flight software |
| Flash (Golden) | 0x08080000 | 256 KB | Backup image |
| Flash (Data) | 0x080C0000 | 256 KB | Configuration, logs |
| Flash (not allocated) | 0x08100000 | 1 MB | Not used by the plan above |
| SRAM (SRAM1, SRAM2, SRAM3) | 0x20000000 | 640 KB | Data and bss from the bottom, stack down from the top; no heap |

The code / golden / data flash partition is the plan for a bootloader, which does not exist yet. The flight image is
linked for the start of flash and may use all 2 MB, and its RAM is one 640 KB region (the `memory.x` of embassy-stm32;
the C linker script cited here until 2026-10-24 was removed with the C target build).

### 3.2 Non-Volatile Storage

| Type | Size | Interface | Purpose |
|------|------|-----------|---------|
| Internal Flash | 2 MB | Direct | Code, config |
| FRAM | 256 KB | SPI | Critical state |
| MRAM (optional) | 1 MB | SPI | Telemetry storage |

### 3.3 FRAM Usage

| Address | Size | Content |
|---------|------|---------|
| 0x0000 | 256 B | Mission state |
| 0x0100 | 256 B | EPS state |
| 0x0200 | 256 B | Fault counters |
| 0x0300 | 1 KB | Configuration |
| 0x0700 | 32 KB | Telemetry buffer |
| 0x8700 | 24 KB | Event log |

---

## 4. Clock System

### 4.1 Clock Sources

| Source | Frequency | Accuracy | Purpose |
|--------|-----------|----------|---------|
| HSE (Crystal) | 8 MHz | ±20 ppm | Planned main clock source; a board decision, not used by the flight image (`clocks()` in `software/rust/firmware/src/main.rs`) |
| LSE (Crystal) | 32.768 kHz | ±20 ppm | RTC, watchdog |
| HSI | 16 MHz | ±1% | Backup oscillator |
| LSI | 32 kHz | ±5% | Backup RTC |

### 4.2 Clock Distribution

- System clock: 80 MHz from the PLL, sourced from the internal MSI at 4 MHz (M = 1, N = 40, R = 2; `clocks()` in `software/rust/firmware/src/main.rs`). The
  start-up waits for the PLL to lock; the independent watchdog, started first at its longest timeout (about 32.8 s),
  resets the OBC if it never does. Moving to the HSE is a board decision; above 80 MHz (up to 120 MHz)
  would need Range 1 boost mode
- Peripheral clocks: Divided from system clock
- RTC: LSE direct (32.768 kHz)
- Watchdog: LSI independent (32 kHz)

### 4.3 Clock Monitoring

- CSS (Clock Security System) enabled
- Automatic failover to HSI on HSE failure
- Clock glitch detection
- Monitored in telemetry

---

## 5. Reset and Watchdog

### 5.1 Reset Sources

| Source | Recovery Action |
|--------|-----------------|
| Power-on | Full initialization |
| External pin | Full initialization |
| Watchdog | Log, resume operation |
| Software | Controlled restart |
| Low-power | Resume from stop mode |
| Brown-out | Full initialization |

### 5.2 Watchdog Configuration

| Parameter | Value |
|-----------|-------|
| Type | Independent Watchdog (IWDG) |
| Clock | LSI (32 kHz) |
| Timeout | 30 seconds (`WDT_HW_TIMEOUT_MS`; refreshed only while every task checks in) |
| Prescaler | 256 |
| Reload | 500 |

### 5.3 Multi-Task Watchdog

Per FDIR design, each task checks in:

| Task | Period | Timeout |
|------|--------|---------|
| Main loop | 100 ms | 500 ms |
| Telemetry | 1 s | 5 s |
| ADCS | 100 ms | 500 ms |
| Beacon | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (`docs/BEACON_SPEC.md` §7) | — |

---

## 6. Power Supply

### 6.1 Voltage Rails

| Rail | Source | Current | Load |
|------|--------|---------|------|
| VDD (3.3V) | PDU | 50 mA nom | MCU core |
| VDDA (3.3V) | Filtered | 5 mA | ADC, DAC |
| VBAT (3.3V) | Battery | 5 µA | RTC backup |

### 6.2 Power Filtering

- 100 nF bypass on each VDD pin
- 10 µF bulk capacitor
- Ferrite bead on VDDA
- 1 µF on VBAT

### 6.3 Power Consumption Budget

| Mode | Current | Power |
|------|---------|-------|
| Active (80 MHz) | 15 mA | 50 mW |
| Active (2 MHz) | 1 mA | 3.3 mW |
| Sleep | 200 µA | 0.66 mW |
| Stop | 5 µA | 16 µW |

---

## 7. Debug and Programming

### 7.1 Debug Interface

| Interface | Pins | Purpose |
|-----------|------|---------|
| SWD | SWDIO, SWCLK | Debug/program |
| UART | TX, RX | Console output |
| ITM | SWO | Trace output |

### 7.2 Programming Methods

| Method | Use Case |
|--------|----------|
| SWD (J-Link) | Development, initial load |
| UART Bootloader | Field update |
| In-Application | On-orbit update (future) |

### 7.3 Debug Connector

- 10-pin ARM Cortex Debug Connector
- Accessible from exterior for development
- Unpopulated for flight (optional)

---

## 8. Peripheral Interfaces

**Superseded (1.7):** the board design SQSO-OBC-BRD-001 (`hardware/electrical/obc/OBC_BOARD_DESIGN.md`) and the master
pin allocation SQSO-ICX-001 (`hardware/electrical/INTERCONNECT.md` §4) replace §8.1–§8.5 and Appendix A.5. The tables
below are kept as the record of the earlier plan; where they disagree with SQSO-ICX-001, it governs.

### 8.1 UART Interfaces

| UART | Baud Rate | Connection | Purpose |
|------|-----------|------------|---------|
| USART1 | 115200 | Jetson | Payload data |
| USART2 | 9600 | RF KISS TNC | Uplink frames in, downlink frames out (CR-2026-005) |
| USART3 | 115200 | Debug | Console |

### 8.2 SPI Interfaces

| SPI | Clock | Connection | Purpose |
|-----|-------|------------|---------|
| SPI1 | 10 MHz | FRAM | State storage |
| SPI2 | 1 MHz | Sensors | Sensor expansion |

### 8.3 I2C Interfaces

| I2C | Speed | Connection | Purpose |
|-----|-------|------------|---------|
| I2C1 | 400 kHz | EPS | Power telemetry |
| I2C2 | 400 kHz | ADCS | Sensors, actuators |

### 8.4 GPIO Allocation

| GPIO | Function | Direction |
|------|----------|-----------|
| PA0 | Deploy sense | Input |
| PA1 | Deploy enable | Output |
| PA2 | Jetson power gate | Output |
| PA3 | RF TX enable | Output |
| PA4-PA7 | Status LEDs | Output |
| PB0-PB3 | ADC channels | Analog |
| PC0 | Heater control | Output |
| PC1 | Watchdog kick | Output |

The firmware's provisional pin map (`software/rust/firmware/src/gpio.rs`, `adc.rs`) differs from this table until the
schematic baselines one. For the downlink (CR-2026-005) it adds TX_PWR_SEL on PE3 (output, low = 0.5 W) and the
transmitter monitors TX_TEMP on PA0, TX_FWD on PC0 and TX_REV on PC1 (ADC1 IN5, IN1, IN2); to free those ADC pins
the deployment sense moves to PE2 and the FRAM WP#/HOLD# to PE0/PE1.

### 8.5 ADC Channels

| Channel | Input | Range | Purpose |
|---------|-------|-------|---------|
| ADC1_IN0 | VBAT sense | 0-10V | Battery monitor |
| ADC1_IN1 | Solar sense | 0-10V | Solar monitor |
| ADC1_IN2 | Temp 1 | 0-3.3V | Board temp |
| ADC1_IN3 | Temp 2 | 0-3.3V | Battery temp |

---

## 9. Electrical Design

### 9.1 Schematic Blocks

1. MCU and power supply
2. Clock generation
3. Memory (FRAM)
4. Debug interface
5. UART interfaces
6. I2C interfaces
7. SPI interfaces
8. GPIO/ADC buffers
9. Connectors

### 9.2 PCB Specifications

| Parameter | Value |
|-----------|-------|
| Layers | 4 |
| Size | 90mm × 90mm (PC104 form) |
| Thickness | 1.6 mm |
| Copper | 1 oz outer, 0.5 oz inner |
| Finish | ENIG |
| Impedance | 50Ω controlled (high-speed) |

### 9.3 EMC Design

- Ground plane on layer 2
- Power plane on layer 3
- Decoupling at each IC
- Ferrite beads on power inputs
- ESD protection on external interfaces

---

## 10. Software Architecture Interface

### 10.1 HAL Requirements

The OBC hardware supports the HAL interfaces defined in:
- `hal_gpio.h` - GPIO control
- `hal_uart.h` - UART communication
- `hal_spi.h` - SPI communication
- `hal_i2c.h` - I2C communication
- `hal_timer.h` - Timing services
- `hal_watchdog.h` - Watchdog control
- `hal_flash.h` - Flash/FRAM access
- `hal_adc.h` - ADC reading

Update 2026-10-24: the headers are in `software/flight/include/hal`. The watchdog header is `hal_wdt.h` (there is no
`hal_watchdog.h`), and there are also `hal_radio.h` (telecommand receiver) and `hal_reset.h` (reset cause). The flight
software is Rust (`software/rust/README.md`). On this board the HAL is implemented in Rust on embassy-stm32 in
`software/rust/firmware/src`, with the provisional pin assignment of `software/rust/firmware/src/gpio.rs`. No hardware
has run it.

### 10.2 Boot Sequence

1. Hardware reset
2. Clock initialization
3. Memory test (optional)
4. Peripheral initialization
5. Watchdog start
6. Load configuration from FRAM
7. Check boot reason
8. Start scheduler

As built (2026-10-24, `software/rust/firmware/src/main.rs`; nothing run on hardware): (1) the IWDG starts at its longest
timeout, about 32 s; (2) clocks: MSI 4 MHz through the PLL to 80 MHz; (3) GPIO with the enable and burn lines low, then
FRAM, SPI, I2C, the three UARTs and the ADC; (4) the flight software starts (`Executive::start()`): time base, GPIO and
flash, deployment, fault management, RF safety and telecommand, mission data, EPS, sensors, the mode manager, then the
watchdog manager, which sets the 30 s operating timeout; (5) the main loop, one pass every 20 ms on the Embassy
executor. There is no RTOS scheduler and no memory test (`software/flight/docs/RTOS_ARCHITECTURE.md`).

---

## 11. Radiation Considerations

### 11.1 TID Tolerance

| Component | Tolerance | Margin |
|-----------|-----------|--------|
| STM32L4 | > 30 krad | 3x |
| FRAM | > 100 krad | 10x |
| Passives | > 100 krad | 10x |

### 11.2 SEE Mitigation

| Effect | Mitigation |
|--------|------------|
| SEU (bit flip) | FRAM with ECC, triple voting |
| SET (transient) | Watchdog, power filtering |
| SEL (latchup) | Current limiting, power cycling |

---

## 12. Testing Requirements

### 12.1 Unit-Level Tests

| Test | Requirement |
|------|-------------|
| Power consumption | Measure all modes |
| Clock accuracy | ±50 ppm over temp |
| Interface timing | Meet specifications |
| Watchdog timeout | 4s ±10% |

### 12.2 Integration Tests

| Test | Requirement |
|------|-------------|
| Boot to operational | < 10 seconds |
| All interfaces functional | Per ICD |
| Memory integrity | No errors |
| Extended operation | 72 hours |

---

## 13. CDR Interface Completeness (2026-10-27)

**Schematic state.** No OBC schematic exists in the repository (no capture, netlist or layout files; §9.1 lists
blocks). The only pin assignment that is exercised is the firmware's provisional map (`software/rust/firmware/src/`),
which CI builds and analyses but which has not run on hardware. §8 above disagrees with it in several places
(I2C1/I2C2 against the firmware's I2C2/I2C4; SPI2 "sensors" against the RF transceiver; PC1 "watchdog kick" against
the downlink's TX_REV monitor; PA4–PA7 LEDs against the FRAM SPI1). The firmware map governs until the schematic
baselines one (RID AV-01).

**Update (1.7, design completion):** the schematic now exists: SQSO-OBC-BRD-001 (`hardware/electrical/obc/design.py`,
netlist, BOM, ERC) with the master pin allocation SQSO-ICX-001. §13.5 gives the resolution of every gap below; the
firmware changes that follow from it are listed in SQSO-ICX-001 §6.

### 13.1 Every OBC Interface

| Interface | Peripheral / pins (provisional) | Firmware driver | Flight software | Gap |
|-----------|---------------------------------|-----------------|-----------------|-----|
| Jetson link | USART1 PA9/PA10, 115200 | `uart.rs` | `jetson_link`, executive | back-powering of the unpowered module (AV-09) |
| KISS TNC (uplink, downlink) | USART2 PD5/PD6, 9600 | `radio.rs` | `uplink`, `downlink` | TNC not selected (comms reviewer) |
| Debug console | USART3 PD8/PD9, 115200 | `uart.rs` | console | — |
| GNSS receiver | LPUART1 PG7/PG8, 9600; `GNSS_EN` PE5 | `uart.rs`, `gpio.rs` | `gnss`, `gnss_schedule` (LV-04) | receiver not selected; latch-up trip (AV-04) |
| FRAM FM25V20A | SPI1 PA5/PA6/PA7, CS PA4, WP PE0, HOLD PE1 | `fram.rs` (12 regions) | `nvm`, every record | — |
| RF transceiver | SPI2 PB13/PB14/PB15, CS PD0 | `spi.rs` (generic) | none | not used by the KISS architecture (CR-2026-005); free for the gyro |
| Gyro M-G370PDT0 (BMI088 backup) | SPI3 PC10/PC11/PC12, CS PD1 (proposed); DRDY, RESET open | `spi.rs` (generic) | rate as `Option` | **no driver, no DRDY/RESET pins, no load switch** (AV-03) |
| EPS controller, power monitors, AntS | I2C2 PF1/PF0 (bus 0) | `i2c.rs` | `eps_control`, `ants` | AntS codes to confirm (AV-V7) |
| Magnetometer | I2C4 PD12/PD13 (bus 1) | `i2c.rs` (generic) | field input | **part and driver open** (AV-03) |
| Coarse sun sensors (6) | ADC1, no channels assigned | `adc.rs` has 9 external channels, none for them | sun-vector input | **channels open** (AV-03) |
| Magnetorquer PWM and direction (3 axes) | timer outputs, not assigned | none | `MtqCommand`, burst | **driver and pins open**; the timer must end each burst itself (AV-03) |
| EPS and transmitter monitors | ADC1: VBATT PC2, IBATT PC3, VSOLAR PC4, ISOLAR PC5, VBUS PB0, TEMP_BOARD PB1, TX_TEMP PA0, TX_FWD PC0, TX_REV PC1; MCU temperature internal | `adc.rs` | `sensors`, `downlink` | front-end scale factors open until the EPS sense circuits are drawn |
| Load switches | PAYLOAD_EN PB4, RADIO_EN PB5, ADCS_EN PB6, HEATER_EN PD2, JETSON_PWR PA2 | `gpio.rs` | `eps_control` | switch readback (AV-06) |
| RF control | RF_TX_EN PA3, TX_PWR_SEL PE3 | `gpio.rs` | `rf_control`, `downlink` | — |
| Transponder enable | XPDR_EN PE4, heartbeat from TIM7 (100 Hz, 60 ms lease) | `heartbeat.rs` | `transponder` (SRS-F081) | the radio board's monostable and its readbacks `TONE_OK`, `XPDR_KEY` (AV-05) |
| Deployment | DEPLOY_EN PA1, SOLAR_BURN PB10 (wing driver A), SOLAR_BURN_B PE6 (driver B), SOLAR_SENSE PB12; antenna via AntS I2C (ANTENNA_BURN PB9 unused, held low; ANTENNA_SENSE PB11 configured as an input, no longer read) | `gpio.rs` | `deployment`, `ants` (SRS-F089) | two resistors per wing on separate drivers (STR-15) |
| Separation and GSE | DEPLOY_SENSE PE2, UMBILICAL_SENSE PB8 | `gpio.rs` | `separation_switch`, `gse_provision` | — |
| EPS alert | ALERT PB7 (input) | `gpio.rs` | not read | unused input; either wire it to an EXTI for a fast load-shed or remove it |
| Status LED | PD3 | `gpio.rs` | — | off in flight |
| Watchdog | IWDG on LSI, 30 s | `wdt.rs` | `watchdog_mgr` (per-task check-in) | no external supervisor (AV-02) |
| Reset cause | RCC CSR | `wdt.rs` (`hal_reset_cause`) | boot record, housekeeping `RST` | — |
| Time base | Embassy time driver, 32.768 kHz, 64-bit ms since boot | `timer.rs` | every timer | no wall clock (AV-10) |

### 13.2 Watchdog and Reset

The IWDG (LSI, 30 s) is refreshed only while every registered task has checked in (`watchdog_mgr`); its clock is
independent of the main oscillator, and it cannot be stopped once started. There is no external watchdog or voltage
supervisor: an LSI failure or a latch-up that holds the MCU in a state the IWDG cannot reset (only a power cycle
clears a latch-up) is covered only by the EPS removing the OBC rail, and no OBC-rail trip is specified (AV-08). The
"watchdog kick" GPIO of §8.4 has no driver and no external watchdog to kick (AV-02: add a windowed external
supervisor such as a TPS3851-class part on its own GPIO, or delete the line).

### 13.3 Non-volatile Memory

Twelve FRAM regions in the 256 KB part (`fram.rs`, STORAGE_DESIGN §3.3); every flight record is two slots with a
sequence number and CRC-32 (`sqso_fsw::nvm`), so a write interrupted by a reset leaves the older slot valid. The new
operations record (commissioning hold, transponder enable, ADCS geometry preset; `ops_state.h`) is region 11, 128 bytes
at 0x2_0600. FRAM is not susceptible to upsets of the stored data at these doses (RADIATION_ANALYSIS Rev C §5).

### 13.4 Timekeeping (AV-10)

Time is milliseconds since boot; it restarts at every reset. The fault log and housekeeping carry it with the boot count,
which is enough to order events but not to date them. The GNSS fix carries GPS week and time of week (`gnss.rs`) and the housekeeping
frame its age; the flight software does not use it to set a mission clock. Recommended (fabrication phase): keep a
persisted mission-elapsed-time offset in FRAM, updated at each GNSS fix, so that logs survive resets with absolute time.

### 13.5 Resolution in the Board Design (1.7)

| Gap (§13.1–§13.4) | Resolution (SQSO-OBC-BRD-001 / SQSO-ICX-001) |
|-------------------|------------------------------------------------|
| AV-01 schematic and pin conflicts | schematic as code with selected parts; one pin table builds the MCU (ICX §4); 25 firmware changes listed (ICX §6), among them PAYLOAD_EN PB4 → PB2 (PB4 resets as NJTRST with a pull-up), DEPLOY_EN PA1 → PF3, FRAM PA4–PA7 → PG2–PG5 |
| AV-02 external supervisor | TPS3850H33-Q1 window watchdog and 3.3 V supervisor on NRST; window 5.6–33.3 s, kicked on PG11 every 10 s; the IWDG (28–32.5 s) acts first (BRD §6) |
| AV-03 gyro, backup, magnetometer, sun sensors, coils, sensor rail | on the OBC board: M-G370PDT0 (SPI3, DRDY PD10, /RST PD14), BMI088 (SPI2), RM3100 (I2C4 0x20), six transimpedance amplifiers into an ADS7138-Q1 (0x10), three DRV8876-Q1 from VBAT with a hardware current limit and a hardware off-timer (0.92–0.98 s); three latch-up switches (BRD §9, §10). Drivers remain flight-software work |
| AV-05 / RF-21 readbacks | TONE_OK PC9, XPDR_KEY PG12, PTT_EXCITER PC7 (TIM8 capture) (BRD §13) |
| AV-06 switch independence | PAYLOAD_EN PB2 and JETSON_PWR PA2 on different ports; readbacks of SW1's and the eFuse's outputs (ADC) and the carrier's 3.3 V (BRD §11.2) |
| AV-09 back-power | SN74LVC1T45-Q1 translators powered from the carrier's own 3.3 V: no firmware workaround (BRD §11.1) |
| AV-10 mission clock | RV-3028-C7 on the always-on 3V3_AO (EPS J3-38), time-stamping every NRST edge (BRD §8) |
| EPS alert PB7 | kept as an input (EXTI7), 10 kΩ series |
| GNSS receiver latch-up and back-power | EPS SW7 trip; translators referenced to the receiver's rail; 1PPS on PC6 (BRD §12) |
| HSE "a board decision" | 8 MHz XO fitted (HSE bypass); the firmware may keep MSI (BRD §5) |

## Appendix A: Content Merged from docs/hardware/OBC_DESIGN.md

`docs/hardware/OBC_DESIGN.md` (SMART-QSO-OBC-001) was a second OBC design document for the same board (finding PI-17). Its
sections that are not in the body of this document are carried here unchanged, except for heading numbers and the
test-procedure IDs in A.11.2. They have **not** been reconciled with §1–§12. Where the two documents disagree, A.1 lists
both values; where `docs/BASELINE_PARAMETERS.md` speaks, it governs. Not carried: the introduction, the STM32L476RG and
Apollo4 Blue Plus processor tables and the NUCLEO-L476RG development board, because both parts are superseded by the
baseline OBC.

### A.1 Disagreements Between the Two Documents

| Item | This document (§) | Former SMART-QSO-OBC-001 (Appendix A §) | Baseline |
|------|-------------------|------------------------------------------|----------|
| MCU | STM32L4R5ZI from Version 1.2 (§2.1); was STM32L476RG or Apollo4 Blue | STM32L476RG, Apollo4 Blue Plus alternate (former §3.2) | **STM32L4R5ZI**; STM32L476 and Apollo4 superseded (BASELINE §6) |
| Apollo4 clock | 96 MHz (§2.1 until Version 1.1) | 192 MHz (former §3.2) | Superseded |
| Flash layout | 512 KB code, 256 KB golden, 256 KB data, a bootloader plan; 1 MB of the 2 MB not allocated (§3.1) | 512 KB main, 256 KB backup, 128 KB bootloader (A.4.3) | Not set |
| FRAM | 256 KB (§3.2) | FM25V10, 1 Mbit (A.4.2) | Not set |
| Bulk storage | MRAM 1 MB, optional (§3.2) | W25Q128 flash, 16 MB (A.4.2) | ≥ 7 days onboard storage (BASELINE §8) |
| IWDG reload at prescaler 256 | 500 (§5.2) | 3750 (A.7.1) | Timeout 30 s in both (`WDT_HW_TIMEOUT_MS`). At LSI 32 kHz / 256, 500 counts is about 4 s and 3750 counts is 30 s |
| Watchdog timeout test | 4 s ±10 % (§12.1) | — | 30 s (FINDINGS_REGISTER line 155 records the 4 s value) |
| HSE accuracy | ±20 ppm (§4.1) | ±10 ppm (A.6.2) | Not set |
| UART allocation | USART1 Jetson, USART2 RF TX 9600, USART3 debug 115200 (§8.1) | UART1 Jetson, UART2 debug 9600, UART3 RF spare (A.5.2) | RF: USART2 at 9600 baud to the KISS TNC, both directions (CR-2026-005); the rest not set |
| RF transceiver link | UART (§8.1) | SPI1, 1 MHz (A.5.3) | UART to a KISS TNC (§8.1; CR-2026-005); A.5.3 superseded |
| I2C1 speed | 400 kHz (§8.3) | 100 kHz (A.5.1) | Not set |
| GPIO allocation | §8.4 (PA0 deploy sense … PC1 watchdog kick) | A.5.4, including the 2026-10-15 rows for RF_TX_EN, UMBILICAL_SENSE, DEPLOY_EN and the deployment sense lines (finding SW-10) | Not set; pins to be assigned in the OBC schematic |
| ADC channels | 4: battery, solar, two temperatures (§8.5) | 7: six sun sensors and internal temperature (A.5.5) | Not set |
| Active power | < 100 mW (§1.1); 50 mW budget (§6.3) | < 200 mW threshold (A.2.2, OBC-P006) | Housekeeping allocation 0.70 W orbit average (BASELINE §5) |
| Boot time | < 10 s (§12.2) | < 30 s requirement (A.2.2); < 500 ms budget (A.8.3) | Not set |
| RTOS | — | FreeRTOS 10.4.3 configuration (A.9.1) | No RTOS: the Rust flight software runs on the Embassy executor (BASELINE §6, Revision C, 2026-10-25; CR-2026-002) |

### A.2 OBC Requirements (former §2)

#### A.2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| OBC-F001 | Execute flight software on the Embassy executor (Rust; no RTOS) | Test |
| OBC-F002 | Interface with EPS via I2C | Test |
| OBC-F003 | Interface with Jetson via UART | Test |
| OBC-F004 | Control subsystem power via GPIO | Test |
| OBC-F005 | Store and retrieve persistent data | Test |
| OBC-F006 | Implement hardware watchdog | Test |
| OBC-F007 | Boot to safe mode on watchdog reset | Test |
| OBC-F008 | Provide telemetry collection and formatting | Test |
| OBC-F009 | Generate beacon at configurable intervals | Test |
| OBC-F010 | Detect and log faults | Test |

Note (2026-10-25): OBC-F001 required FreeRTOS until the PI rebaselined the OBC to the Embassy executor (BASELINE §6,
Revision C; change request CR-2026-002, `docs/project/CHANGE_REQUEST_LOG.md`). The flight software runs that way
(`software/flight/docs/RTOS_ARCHITECTURE.md`); verification on hardware is pending (A.11.1).

#### A.2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| OBC-P001 | Main loop execution | <20 ms | <10 ms |
| OBC-P002 | Boot time | <30 s | <15 s |
| OBC-P003 | Flash endurance | >10,000 cycles | >100,000 cycles |
| OBC-P004 | RAM availability | >32 KB | >64 KB |
| OBC-P005 | Flash availability | >128 KB | >256 KB |
| OBC-P006 | Power consumption (active) | <200 mW | <150 mW |
| OBC-P007 | Power consumption (sleep) | <10 mW | <5 mW |

#### A.2.3 Interface Requirements

| ID | Interface | Specification |
|----|-----------|---------------|
| OBC-I001 | I2C Bus 1 | 100 kHz, EPS + sensors |
| OBC-I002 | I2C Bus 2 | 400 kHz, ADCS |
| OBC-I003 | UART 1 | 115200 bps, Jetson |
| OBC-I004 | UART 2 | 9600 bps, Debug |
| OBC-I005 | SPI | 1 MHz, RF transceiver |
| OBC-I006 | GPIO | 16 lines, load control |
| OBC-I007 | ADC | 6 channels, 12-bit |

#### A.2.4 Environmental Requirements

| Parameter | Operating | Non-Operating |
|-----------|-----------|---------------|
| Temperature | -40°C to +85°C | -55°C to +125°C |
| Vibration | 14.1 Grms (random) *(superseded 2026-10-16: ≥ 5.57 Grms MPE, interim SpaceX UG v11 level, `docs/test_procedures/TP-ENV-002_Vibration.md` §4.1)* | N/A |
| Radiation | 10 krad TID | N/A |
| SEU Rate | <10⁻⁹/bit/day | N/A |

### A.3 Block Diagram (former §3.1)

*Superseded part (STM32L476RG, 1 MB flash, 128 KB RAM); the baseline STM32L4R5ZI is in §2.1 and §3.1.*

```
+------------------------------------------------------------------+
|                        OBC SUBSYSTEM                               |
|                                                                    |
|  +--------------------+         +--------------------+             |
|  |    STM32L476RG     |         |   External Flash   |             |
|  |    (Primary MCU)   |         |   (Mission Data)   |             |
|  |                    |         |                    |             |
|  | +----------------+ |   SPI   |   W25Q128JVSIQ     |             |
|  | |  ARM Cortex-M4 | |=========|   (16 MB)          |             |
|  | |  80 MHz        | |         +--------------------+             |
|  | +----------------+ |                                            |
|  |                    |         +--------------------+             |
|  | +----------------+ |         |   External FRAM    |             |
|  | |  Flash: 1 MB   | |   SPI   |   (Critical Data)  |             |
|  | |  RAM: 128 KB   | |=========|   FM25V10          |             |
|  | +----------------+ |         |   (1 Mbit)         |             |
|  |                    |         +--------------------+             |
|  | +----------------+ |                                            |
|  | |  Peripherals   | |                                            |
|  | | - I2C x2       | |                                            |
|  | | - UART x3      |-+----> To EPS, ADCS, Jetson, Debug           |
|  | | - SPI x2       | |                                            |
|  | | - GPIO x16     | |                                            |
|  | | - ADC x6       | |                                            |
|  | | - IWDG         | |                                            |
|  | +----------------+ |                                            |
|  +--------------------+                                            |
|                                                                    |
+------------------------------------------------------------------+
           |              |              |              |
           v              v              v              v
      +--------+     +--------+     +--------+     +--------+
      |  EPS   |     |  ADCS  |     | Jetson |     |   RF   |
      | (I2C)  |     | (I2C)  |     | (UART) |     | (SPI)  |
      +--------+     +--------+     +--------+     +--------+
```

### A.4 Memory Architecture (former §3.3)

#### A.4.1 Memory Map (STM32L476)

*Superseded part; the STM32L4R5ZI memory map is in §3.1.*

| Region | Address | Size | Usage |
|--------|---------|------|-------|
| Flash | 0x0800_0000 | 1 MB | Flight software |
| SRAM1 | 0x2000_0000 | 96 KB | Runtime data |
| SRAM2 | 0x1000_0000 | 32 KB | Stack, critical data |
| EEPROM | 0x0808_0000 | 16 KB | Configuration |
| Option | 0x1FFF_7800 | 1 KB | Boot config |

#### A.4.2 External Memory

| Device | Interface | Size | Usage |
|--------|-----------|------|-------|
| W25Q128 | SPI | 16 MB | Mission data, logs |
| FM25V10 | SPI | 128 KB | Critical state, fault log |

#### A.4.3 Memory Allocation

```
+------------------+  0x0810_0000
|  Reserved        |
+------------------+  0x080E_0000
|  Bootloader      |  128 KB
+------------------+  0x080C_0000
|  Flight SW       |  512 KB
|  (Main Image)    |
+------------------+  0x0804_0000
|  Flight SW       |  256 KB
|  (Backup Image)  |
+------------------+  0x0800_0000

+------------------+  0x2001_8000
|  Heap            |  32 KB
+------------------+  0x2001_0000
|  Stack           |  8 KB
+------------------+  0x2000_E000
|  FreeRTOS        |  24 KB
|  Kernel          |
+------------------+  0x2000_8000
|  Application     |  32 KB
|  Data            |
+------------------+  0x2000_0000
```

*(Superseded 2026-10-25: there is no FreeRTOS kernel and no heap. The RAM holds the static data from 0x20000000 and one
8 KB stack below 0x200A0000 with an MPU guard; `software/flight/docs/MEMORY_POLICY.md` §3.1 and §6.)*

### A.5 Peripheral Interfaces (former §4)

#### A.5.1 I2C Bus Configuration

##### A.5.1.1 I2C Bus 1 (Low Speed)

| Device | Address | Speed | Function |
|--------|---------|-------|----------|
| EPS | 0x48 | 100 kHz | Power telemetry |
| Battery | 0x55 | 100 kHz | SoC, voltage |
| Solar | 0x40 | 100 kHz | Current, MPPT |
| Temperature | 0x48 | 100 kHz | TMP117 sensors |

##### A.5.1.2 I2C Bus 2 (ADCS)

| Device | Address | Speed | Function |
|--------|---------|-------|----------|
| Magnetometer | 0x1E | 400 kHz | HMC5883L |
| IMU (spare) | 0x68 | 400 kHz | MPU6050 |

#### A.5.2 UART Configuration

| Port | Baud | Connection | Protocol |
|------|------|------------|----------|
| UART1 | 115200 | Jetson | Custom (ICD-002) |
| UART2 | 9600 | Debug | ASCII |
| UART3 | 9600 | RF (spare) | AX.25 |

#### A.5.3 SPI Configuration

| Port | Speed | Connection | Mode |
|------|-------|------------|------|
| SPI1 | 1 MHz | RF Transceiver | Mode 0 |
| SPI2 | 10 MHz | External Flash | Mode 0 |
| SPI3 | 10 MHz | External FRAM | Mode 0 |

#### A.5.4 GPIO Allocation

| GPIO | Function | Direction | Default |
|------|----------|-----------|---------|
| PA0 | PAYLOAD_EN | Output | LOW |
| PA1 | RADIO_EN | Output | LOW |
| PA2 | ADCS_EN | Output | LOW |
| PA3 | HEATER_EN | Output | LOW |
| PA4 | SENSOR_EN | Output | HIGH |
| PA5 | SPARE_EN | Output | LOW |
| PB0 | DEPLOY_1 (`GPIO_PIN_ANTENNA_BURN`, TBR) | Output, pull-down | LOW |
| PB1 | DEPLOY_2 (`GPIO_PIN_SOLAR_BURN`, TBR) | Output, pull-down | LOW |
| PB2 | DEPLOY_SENSE | Input | - |
| PC0 | JETSON_RDY | Input | - |
| PC1 | EPS_INT | Input | - |
| PC2 | WDT_KICK | Output | - |
| PC3 | LED_STATUS | Output | LOW |
| TBD | RF_TX_EN | Output, pull-down | LOW (PA off until `rf_control_init()` restores the TX-OFF latch) |
| TBD | UMBILICAL_SENSE | Input, pull-down | GSE connector drives high; enables key provisioning (SQSO-ICD-013 §9.2) |
| TBD | DEPLOY_EN (master arm) | Output, pull-down | LOW; a burn needs this **and** DEPLOY_1/2 high |
| TBD | ANTENNA_SENSE | Input, pull-down | High = antenna deployed (TBR) |
| TBD | SOLAR_SENSE | Input, pull-down | High = solar array deployed (TBR) |

*RF_TX_EN, UMBILICAL_SENSE, DEPLOY_EN and the two deployment sense lines added 2026-10-15 to match `hal_gpio.h`; pins to be assigned in the OBC schematic. Before this, the software fired both burn wires from one line and read both deployables from the separation-switch line (finding SW-10).*

#### A.5.5 ADC Channels

| Channel | Signal | Range | Resolution |
|---------|--------|-------|------------|
| ADC1_IN0 | SUN_PX | 0-3.3V | 12-bit |
| ADC1_IN1 | SUN_MX | 0-3.3V | 12-bit |
| ADC1_IN2 | SUN_PY | 0-3.3V | 12-bit |
| ADC1_IN3 | SUN_MY | 0-3.3V | 12-bit |
| ADC1_IN4 | SUN_PZ | 0-3.3V | 12-bit |
| ADC1_IN5 | SUN_MZ | 0-3.3V | 12-bit |
| ADC1_IN6 | TEMP_INT | 0-3.3V | 12-bit |

### A.6 Clock Configuration (former §5)

#### A.6.1 Clock Tree

```
                   +--------+
    HSE (8 MHz) -->|  PLL   |--> SYSCLK (80 MHz)
                   +--------+
                       |
         +-------------+-------------+
         |             |             |
         v             v             v
    +--------+    +--------+    +--------+
    | APB1   |    | APB2   |    | AHB    |
    | 80 MHz |    | 80 MHz |    | 80 MHz |
    +--------+    +--------+    +--------+
         |             |             |
         v             v             v
    UART, I2C    SPI, ADC      DMA, Flash
```

#### A.6.2 Clock Sources

| Source | Frequency | Usage | Accuracy |
|--------|-----------|-------|----------|
| HSE | 8 MHz | PLL input | ±10 ppm |
| LSE | 32.768 kHz | RTC | ±20 ppm |
| HSI | 16 MHz | Backup | ±1% |
| LSI | 32 kHz | IWDG | ±5% |

#### A.6.3 Real-Time Clock

| Parameter | Specification |
|-----------|---------------|
| Source | LSE (32.768 kHz) |
| Battery | CR1220 backup |
| Accuracy | ±2 s/day |
| Epoch | Unix time (seconds since 1970) |
| Calendar | BCD format |

### A.7 Watchdog Design (former §6)

#### A.7.1 Independent Watchdog (IWDG)

| Parameter | Value |
|-----------|-------|
| Clock | LSI (32 kHz) |
| Prescaler | 256 |
| Reload | 3750 |
| Timeout | 30 seconds |
| Reset Type | System reset |

#### A.7.2 Watchdog Kick Pattern

```c
// Main loop watchdog kick
void main_loop(void) {
    while (1) {
        // Execute tasks
        eps_task();
        beacon_task();
        telemetry_task();

        // Kick watchdog only if all critical tasks complete
        if (eps_healthy() && comms_healthy()) {
            hal_wdt_kick();
        }

        // Sleep until next tick
        vTaskDelay(pdMS_TO_TICKS(MAIN_LOOP_MS));
    }
}
```

#### A.7.3 Watchdog Reset Recovery

```
                   +-------------+
                   | Watchdog    |
                   | Reset       |
                   +------+------+
                          |
                          v
                   +-------------+
                   | Bootloader  |
                   | Check flags |
                   +------+------+
                          |
          +---------------+---------------+
          |                               |
          v                               v
   +-------------+                 +-------------+
   | Normal Boot |                 | Safe Mode   |
   | (reset_cnt  |                 | (reset_cnt  |
   |    < 3)     |                 |    >= 3)    |
   +-------------+                 +-------------+
```

### A.8 Boot Sequence (former §7)

#### A.8.1 Normal Boot

```
1. Power On
   |
   v
2. Hardware Init (HAL)
   - Clock configuration
   - GPIO defaults
   - Watchdog start
   |
   v
3. Memory Init
   - Stack check
   - Heap init
   - FRAM read
   |
   v
4. FreeRTOS Start
   - Scheduler init
   - Task creation
   |
   v
5. Subsystem Init
   - EPS interface
   - Sensor init
   - Comms init
   |
   v
6. Main Application
   - Enter SAFE mode
   - Begin telemetry
```

#### A.8.2 Safe Mode Boot

| Action | Description |
|--------|-------------|
| Disable Jetson | Keep payload off |
| Disable RF | TX only essential beacons |
| Minimum telemetry | Basic health only |
| Extended beacon | 180s interval |
| Wait for recovery | Power or thermal improvement |

#### A.8.3 Boot Time Budget

| Phase | Time | Cumulative |
|-------|------|------------|
| Reset vector | 1 ms | 1 ms |
| Clock init | 10 ms | 11 ms |
| HAL init | 50 ms | 61 ms |
| FreeRTOS init | 20 ms | 81 ms |
| Subsystem init | 200 ms | 281 ms |
| FRAM load | 50 ms | 331 ms |
| First beacon | 100 ms | 431 ms |
| **Total** | | **<500 ms** |

### A.9 Software Architecture (former §8)

Note (2026-10-24): A.8 and A.9 are the superseded design merged from SMART-QSO-OBC-001. FreeRTOS was never integrated.
The software as built is in §10.2 and `software/flight/docs/RTOS_ARCHITECTURE.md`. *(2026-10-25: the baseline is the
Embassy executor with no RTOS; BASELINE §6, CR-2026-002.)*

#### A.9.1 FreeRTOS Configuration

| Parameter | Value |
|-----------|-------|
| Version | 10.4.3 |
| Tick Rate | 1000 Hz |
| Max Priorities | 8 |
| Stack Size (default) | 256 words |
| Heap Size | 32 KB |
| Heap Algorithm | heap_4 (coalesce) |

#### A.9.2 Task Configuration

| Task | Priority | Stack | Period |
|------|----------|-------|--------|
| Main | 4 | 512 | 100 ms |
| EPS | 5 | 256 | 1000 ms |
| Beacon | 3 | 512 | 30-180 s |
| Telemetry | 3 | 256 | 1000 ms |
| ADCS | 4 | 256 | 1000 ms |
| Jetson Comm | 2 | 512 | Event |
| Fault Monitor | 6 | 256 | 100 ms |
| Idle | 0 | 128 | N/A |

#### A.9.3 Inter-Task Communication

```
+------------+     +------------+
|  EPS Task  |     | Main Task  |
+-----+------+     +-----+------+
      |                  |
      | PowerMode_t      | Commands
      v                  v
+----------+        +----------+
| EPS      |        | Command  |
| Queue    |        | Queue    |
+----------+        +----------+
      |                  |
      +--------+---------+
               |
               v
         +----------+
         | Beacon   |
         | Task     |
         +----------+
```

### A.10 Telemetry Collection (former §9)

#### A.10.1 Telemetry Structure

```c
typedef struct {
    // Header
    uint32_t timestamp;          // Unix time
    uint16_t sequence;           // Packet sequence
    uint8_t  power_mode;         // SAFE/IDLE/ACTIVE

    // EPS Data
    uint16_t battery_voltage_mv;  // mV
    int16_t  battery_current_ma;  // mA
    uint8_t  state_of_charge;     // %
    uint16_t solar_current_ma;    // mA

    // Thermal
    int8_t   temp_obc;           // °C
    int8_t   temp_battery;       // °C
    int8_t   temp_payload;       // °C

    // ADCS
    int16_t  mag_x, mag_y, mag_z; // μT * 10
    uint16_t sun_sensors[6];      // ADC counts

    // Status
    uint8_t  fault_flags;        // Bitmask
    uint16_t reset_count;        // Total resets
    uint32_t uptime_s;           // Seconds since boot

    // CRC
    uint32_t crc32;              // Data integrity
} Telemetry_t;
```

#### A.10.2 Telemetry Rates

| Data Type | Collection Rate | Downlink Rate |
|-----------|-----------------|---------------|
| Housekeeping | 1 Hz | Every beacon |
| EPS | 1 Hz | Every beacon |
| ADCS | 10 Hz | Averaged per beacon |
| Thermal | 0.1 Hz | Every beacon |
| Fault Log | Event | On change |

### A.11 Verification (former §10)

#### A.11.1 Verification Matrix

| Requirement | Method | Status |
|-------------|--------|--------|
| OBC-F001 | Demonstration | Pending |
| OBC-F002 | Test | Pending |
| OBC-F003 | Test | Pending |
| OBC-F004 | Test | Pending |
| OBC-F005 | Test | Pending |
| OBC-F006 | Test | Pending |
| OBC-F007 | Test | Pending |
| OBC-F008 | Test | Pending |
| OBC-F009 | Test | Pending |
| OBC-F010 | Test | Pending |

#### A.11.2 Test Procedures

| Test | Description | Equipment |
|------|-------------|-----------|
| TP-OBC-001 | Boot sequence timing | Logic analyzer |
| TP-OBC-004 | Watchdog timeout | Oscilloscope |
| TP-OBC-001 | I2C communication | Bus analyzer |
| TP-OBC-001 | UART communication | Serial monitor |
| TP-OBC-005 | Flash endurance | Cycle tester |
| TP-OBC-006 | Power consumption | Power analyzer |
| TP-OBC-007 | Thermal cycling | Thermal chamber |

IDs are those of the procedure files in `docs/test_procedures/` that cover each test (aligned 2026-10-16, finding VV-18;
the former document numbered TP-OBC-002 to -004 differently). TP-OBC-005 to -007 have no procedure yet.

### A.12 Risk Assessment (former §11)

#### A.12.1 OBC-Specific Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| OBC-R001 | Flash corruption | L | H | CRC, backup image |
| OBC-R002 | SEU in RAM | M | M | Error detection, restart |
| OBC-R003 | I2C bus lockup | L | M | Timeout, bus reset |
| OBC-R004 | Clock failure | L | H | Fallback to HSI |
| OBC-R005 | Watchdog failure | L | H | External WDT backup |

### A.13 Programming Interface (former Appendix B)

```
+----------+     SWD      +----------+
|  Host    |==============|  STM32   |
|  (USB)   |  SWCLK/SWDIO |  L476    |
+----------+              +----------+
     |
     v
OpenOCD / STM32CubeProgrammer
```

**SWD Connections**:
| Signal | Pin |
|--------|-----|
| SWCLK | PA14 |
| SWDIO | PA13 |
| NRST | NRST |
| GND | GND |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-16 | Merged `docs/hardware/OBC_DESIGN.md` (SMART-QSO-OBC-001), a second OBC design for the same board, as Appendix A without reconciling it; A.1 lists where the two disagree and what the baseline says; §2.1 notes the baseline MCU; test IDs in A.11.2 aligned with the procedure files; status changed from "CDR Baseline" to Draft, as no CDR has been held (findings PI-17, VV-18) |
| 1.2 | 2026-10-16 | §2.1, §2.2, §3.1, §3.2, §4.1 and §4.2: part data for the baseline STM32L4R5ZI (2 MB flash, 640 KB SRAM, up to 120 MHz; flight image at 80 MHz from MSI), replacing the superseded STM32L476RG / Apollo4 values; flash partition kept and marked as a bootloader plan; A.1 rows updated; A.2.4 vibration and A.3 / A.4.1 carry dated notes (BASELINE §6; findings PI-14, LA-05) |
| 1.3 | 2026-10-24 | Software sections for the Rust flight software: the removed C linker script replaced by the firmware's link configuration (§2.1, §3.1); §10.1 HAL headers as they are and their Rust implementation; §10.2 boot sequence as built; A.1 RTOS row; note that OBC-F001 (FreeRTOS) is not met as written; A.9 marked superseded |
| 1.4 | 2026-10-25 | OBC-F001 rebaselined by the PI to the Embassy executor (was FreeRTOS; BASELINE §6 Revision C, CR-2026-002); A.1 RTOS row; superseded note on the A.4 RAM map |
| 1.5 | 2026-10-25 | CR-2026-005: USART2 is the KISS TNC link for the uplink and the downlink (§8.1; conflicts table: UART allocation, RF transceiver link); §8.4 note on the firmware's downlink pins |
| 1.6 | 2026-10-27 | §13 new (CDR avionics review): schematic state, every OBC interface with its pins, driver and gap (RIDs AV-01 to AV-10), watchdog and reset, NVM regions including the operations record, timekeeping |
| 1.7 | 2026-10-27 | §8 marked superseded by SQSO-ICX-001 (master pin allocation); §13 note and §13.5: resolution of every open avionics item (AV-01, AV-02, AV-03, AV-05, AV-06, AV-09, AV-10, RF-21) in the board design SQSO-OBC-BRD-001 |
