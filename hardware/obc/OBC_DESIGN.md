# On-Board Computer (OBC) Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-CDH-OBC-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

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
| Manufacturer | STMicroelectronics / Ambiq |
| Part Number | STM32L476RG or Apollo4 Blue |
| Core | ARM Cortex-M4F |
| Clock Speed | 80 MHz (STM32L4) / 96 MHz (Apollo4) |
| Flash | 1 MB |
| SRAM | 128 KB (STM32L4) / 2 MB (Apollo4) |
| Package | LQFP-64 or BGA |

### 2.2 Selection Justification

- Ultra-low power operation (< 50 µA/MHz active)
- Extensive peripheral set (UART, SPI, I2C, ADC)
- Hardware FPU for attitude calculations
- Flight heritage on similar CubeSat missions
- Wide temperature range (-40°C to +105°C)
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
| SRAM | 0x20000000 | 128 KB | Runtime data |
| SRAM2 | 0x10000000 | 32 KB | Stack, critical data |

### 3.2 Non-Volatile Storage

| Type | Size | Interface | Purpose |
|------|------|-----------|---------|
| Internal Flash | 1 MB | Direct | Code, config |
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
| HSE (Crystal) | 8 MHz | ±20 ppm | Main clock source |
| LSE (Crystal) | 32.768 kHz | ±20 ppm | RTC, watchdog |
| HSI | 16 MHz | ±1% | Backup oscillator |
| LSI | 32 kHz | ±5% | Backup RTC |

### 4.2 Clock Distribution

- System clock: PLL from HSE → 80 MHz
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
| Timeout | 4 seconds |
| Prescaler | 256 |
| Reload | 500 |

### 5.3 Multi-Task Watchdog

Per FDIR design, each task checks in:

| Task | Period | Timeout |
|------|--------|---------|
| Main loop | 100 ms | 500 ms |
| Telemetry | 1 s | 5 s |
| ADCS | 100 ms | 500 ms |
| Beacon | 30 s | 60 s |

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

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
