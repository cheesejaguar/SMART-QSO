# On-Board Computer (OBC) Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-CDH-OBC-001
**Version:** 1.2
**Date:** 2026-10-16
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
| SRAM | 640 KB: SRAM1 192 KB at 0x20000000, SRAM2 64 KB at 0x20030000, SRAM3 384 KB at 0x20040000, contiguous (`software/flight/cmake/stm32l4r5zi.ld`) |
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
linked for the start of flash and may use all 2 MB, and its RAM is one 640 KB region (`software/flight/cmake/stm32l4r5zi.ld`).

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

### 8.1 UART Interfaces

| UART | Baud Rate | Connection | Purpose |
|------|-----------|------------|---------|
| USART1 | 115200 | Jetson | Payload data |
| USART2 | 9600 | RF TX | Beacon data |
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

### 10.2 Boot Sequence

1. Hardware reset
2. Clock initialization
3. Memory test (optional)
4. Peripheral initialization
5. Watchdog start
6. Load configuration from FRAM
7. Check boot reason
8. Start scheduler

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
| UART allocation | USART1 Jetson, USART2 RF TX 9600, USART3 debug 115200 (§8.1) | UART1 Jetson, UART2 debug 9600, UART3 RF spare (A.5.2) | Not set |
| RF transceiver link | UART (§8.1) | SPI1, 1 MHz (A.5.3) | Not set |
| I2C1 speed | 400 kHz (§8.3) | 100 kHz (A.5.1) | Not set |
| GPIO allocation | §8.4 (PA0 deploy sense … PC1 watchdog kick) | A.5.4, including the 2026-10-15 rows for RF_TX_EN, UMBILICAL_SENSE, DEPLOY_EN and the deployment sense lines (finding SW-10) | Not set; pins to be assigned in the OBC schematic |
| ADC channels | 4: battery, solar, two temperatures (§8.5) | 7: six sun sensors and internal temperature (A.5.5) | Not set |
| Active power | < 100 mW (§1.1); 50 mW budget (§6.3) | < 200 mW threshold (A.2.2, OBC-P006) | Housekeeping allocation 0.70 W orbit average (BASELINE §5) |
| Boot time | < 10 s (§12.2) | < 30 s requirement (A.2.2); < 500 ms budget (A.8.3) | Not set |
| RTOS | — | FreeRTOS 10.4.3 configuration (A.9.1) | FreeRTOS (BASELINE §6). Not yet integrated: the flight software runs as a host simulation |

### A.2 OBC Requirements (former §2)

#### A.2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| OBC-F001 | Execute flight software in FreeRTOS | Test |
| OBC-F002 | Interface with EPS via I2C | Test |
| OBC-F003 | Interface with Jetson via UART | Test |
| OBC-F004 | Control subsystem power via GPIO | Test |
| OBC-F005 | Store and retrieve persistent data | Test |
| OBC-F006 | Implement hardware watchdog | Test |
| OBC-F007 | Boot to safe mode on watchdog reset | Test |
| OBC-F008 | Provide telemetry collection and formatting | Test |
| OBC-F009 | Generate beacon at configurable intervals | Test |
| OBC-F010 | Detect and log faults | Test |

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
