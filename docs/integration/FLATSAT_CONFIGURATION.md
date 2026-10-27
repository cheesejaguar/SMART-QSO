# Flatsat Configuration
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-INT-004
**Revision:** B
**Date:** 2026-10-27
**Reference:** NASA Ames SmallSat Development Heritage; `hardware/electrical/obc/OBC_BOARD_DESIGN.md` (SQSO-OBC-BRD-001); `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001)

> **Configuration planned, not built.** No flatsat exists yet (`docs/BASELINE_PARAMETERS.md` §1). The development-board
> stage below is replaced by the EM boards as they are built (EM integration and flatsat 2027-06-01 to 2027-07-27,
> `docs/verification/VI_REGISTER.md`). Interfaces and parts follow the OBC board design and the interconnect, which govern.

---

## 1. Purpose

This document defines the Flatsat (engineering development unit) configuration for SMART-QSO, enabling software development, integration testing, and anomaly investigation without risk to flight hardware.

## 2. Flatsat Philosophy

### 2.1 NASA Ames Heritage

Following NASA Ames Small Satellite Division heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat):

| Principle | Implementation |
|-----------|----------------|
| Build the flatsat first | SW development before flight HW |
| Flight-like interfaces | Same connectors, protocols |
| Continuous operation | 24/7 availability for testing |
| Anomaly investigation | Safe environment for debugging |
| Operator training | Ground team familiarity |

### 2.2 Flatsat Uses

| Use Case | Description |
|----------|-------------|
| Software development | Code before flight hardware |
| Integration testing | Subsystem integration |
| Procedure development | Ops procedures validation |
| Anomaly investigation | Debug without flight risk |
| Training | Operator familiarization |
| Regression testing | Verify SW updates |

## 3. Configuration Overview

### 3.1 Block Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     FLATSAT CONFIGURATION                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   EPS Dev    │    │   OBC Dev    │    │   RF Dev     │      │
│  │    Board     │◄──►│    Board     │◄──►│    Board     │      │
│  │              │    │              │    │              │      │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘      │
│         │                   │                   │               │
│         │    ┌──────────────┴──────────────┐    │               │
│         │    │                             │    │               │
│         ▼    ▼                             ▼    ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ Solar Sim    │    │  ADCS Dev    │    │ Jetson Dev   │      │
│  │   (PSU)      │    │   Board      │    │    Kit       │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐     │
│  │                    GSE / Test Equipment                 │     │
│  │  Power Supply │ Logic Analyzer │ RF Receiver │ PC      │     │
│  └────────────────────────────────────────────────────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Hardware Comparison

| Subsystem | Flight | Flatsat | Notes |
|-----------|--------|---------|-------|
| OBC | Flight MCU | Dev board | Same MCU family |
| EPS | Flight PCBA | Dev board | Equivalent functionality |
| ADCS | Flight sensors | Breakout boards | Same sensor ICs |
| RF | Flight radio | SDR + PA | TX attenuated |
| Payload | Jetson Orin | Jetson Dev Kit | Identical SW |
| Structure | 1U frame | Breadboard mount | Open access |

## 4. Subsystem Details

### 4.1 OBC Development Board

| Parameter | Specification |
|-----------|---------------|
| Board | STM32L4 Nucleo-144 |
| MCU | STM32L4R5ZI |
| Clock | Same as flight (SQSO-OBC-BRD-001) |
| Memory | Same as flight |
| Debug | ST-Link integrated |
| Interfaces | All flight interfaces broken out |

**Connections:**
- USART3: console and key loading (access port)
- USART1: Jetson carrier (115200 baud, OBC transmits 8N2)
- USART2: RF board (TNC), with its readbacks
- LPUART1: GNSS receiver
- I2C1: RTC RV-3028 (0x52); I2C2: EPS controller (0x48); I2C3: AntS (0x31, 0x32); I2C4: ADS7138 and the RM3100 (0x20)
- SPI1: FRAM FM25V20A ×2 (no external flash); SPI2: BMI088; SPI3: M-G370PDT0
- GPIO: DEPLOY_SENSE, UMBILICAL_SENSE, rail enables, burn lines

### 4.2 EPS Development Board

| Parameter | Specification |
|-----------|---------------|
| Board | Custom EPS dev board |
| Battery | Flight-equivalent cells |
| MPPT | Flight-equivalent IC |
| Power rails | 3V3_AO, 5V_PA, VBAT_JET, V_BURN, VBAT (11.2–16.4 V, 4S) (`docs/hardware/EPS_DESIGN.md` §4) |
| Telemetry | All EPS channels |
| Load switches | All flight switches |

**Solar Simulation:**
- Keithley 2400 or equivalent
- Programmable I-V curve
- Orbital profile simulation

### 4.3 ADCS Development

| Sensor | Board | Interface |
|--------|-------|-----------|
| Magnetometer | PNI RM3100 (board A2, or its breakout) | I2C4 |
| Sun sensors | Six face photodiodes through the ADS7138 | I2C4 |
| Gyroscope | Epson M-G370PDT0 (BMI088 backup) | SPI3 (SPI2) |

**Magnetorquer Simulation:**
- H-bridge driver boards
- Current sense resistors
- Helmholtz cage interface (optional)

### 4.4 RF Development

| Parameter | Specification |
|-----------|---------------|
| SDR | RTL-SDR or USRP |
| Frequency | The IARU-coordinated downlink in 145.800–146.000 MHz, not 145.825 MHz (SYS-200; BASELINE §7) |
| TX simulation | Low-power, attenuated |
| RX | Direct SDR input |
| Antenna | Dummy load + coupler |

**Safety:**
- TX power limited to < 10 mW
- RF shielded enclosure
- "TX ACTIVE" indicator

### 4.5 Payload Development

| Parameter | Specification |
|-----------|---------------|
| Board | Jetson Orin Nano Dev Kit in the 7 W power mode, then the EM carrier with the 8 GB module (BASELINE §6) |
| Interface | USB-UART adapter, then USART1 through the carrier's level shifters |
| Power | Dev kit supply, then VBAT_JET from the EPS (the carrier makes the module's 5 V) |
| Storage | Flight image (NVMe on the carrier, V-JET-3) |
| Network | Ethernet for dev |

## 5. Test Equipment

### 5.1 Required GSE

| Equipment | Model | Purpose |
|-----------|-------|---------|
| Power supply | Keithley 2230G | Solar simulation, charging |
| DMM | Keysight 34465A | Voltage/current measurement |
| Logic analyzer | Saleae Logic Pro | Bus debugging |
| Oscilloscope | Keysight DSOX2024A | Signal analysis |
| RF receiver | SDR + GNURadio | Beacon reception |
| Spectrum analyzer | TinySA Ultra | RF verification |

### 5.2 Software Tools

| Tool | Purpose |
|------|---------|
| Rust toolchain (`software/rust/rust-toolchain.toml`), probe-rs or ST-Link | OBC firmware build and load (`software/rust/firmware`) |
| Serial terminal | UART monitoring |
| GNURadio | RF signal processing |
| Python scripts | Automated testing |
| Git | Version control |
| Jenkins/CI | Automated builds |

## 6. Interface Definitions

### 6.1 Debug Console (USART3)

| Parameter | Value |
|-----------|-------|
| Baud rate | 115200 |
| Data bits | 8 |
| Parity | None |
| Stop bits | 1 (per SQSO-ICX-001) |
| Flow control | None |

### 6.2 Jetson Interface (USART1)

| Parameter | Value |
|-----------|-------|
| Baud rate | 115200 |
| Protocol | Custom framed |
| Connector | USB-UART adapter |

### 6.3 EPS Interface (I2C2)

| Parameter | Value |
|-----------|-------|
| Bus | I2C2, EPS controller at 0x48, 100 kHz |
| Protocol | Register commands (`docs/interfaces/ICD_OBC_EPS.md`, SMART-QSO-ICD-001), OBC heartbeat at least every 60 s |

## 7. Test Procedures

### 7.1 Daily Health Check

| Step | Action | Expected |
|------|--------|----------|
| 1 | Power on flatsat | Boot < 30s |
| 2 | Check debug console | Banner displayed |
| 3 | Verify telemetry | All channels valid |
| 4 | Receive beacon | Decoded correctly |
| 5 | Send test command | Acknowledged |

### 7.2 Software Update Procedure

| Step | Action |
|------|--------|
| 1 | Connect debugger |
| 2 | Halt processor |
| 3 | Flash new image |
| 4 | Reset and verify |
| 5 | Run regression tests |
| 6 | Document version |

### 7.3 Anomaly Investigation

| Step | Action |
|------|--------|
| 1 | Reproduce on flatsat |
| 2 | Enable verbose logging |
| 3 | Capture all interfaces |
| 4 | Analyze root cause |
| 5 | Develop fix |
| 6 | Verify on flatsat |
| 7 | Update flight SW |

## 8. Configuration Management

### 8.1 Flatsat vs Flight Differences

| Item | Flatsat | Flight | Rationale |
|------|---------|--------|-----------|
| Debug UART | Enabled | Disabled | Development access |
| Watchdog | Configurable | Enabled | Debug flexibility |
| TX power | Limited | Full | Safety |
| Logging | Verbose | Reduced | Storage |

### 8.2 Software Branches

| Branch | Target |
|--------|--------|
| develop | Flatsat testing |
| release/* | Flight candidate |
| main | Flight release |

## 9. Safety

### 9.1 RF Safety

| Precaution | Implementation |
|------------|----------------|
| Power limit | < 10 mW output |
| Shielding | RF enclosure |
| Indicator | "TX ACTIVE" LED |
| Interlock | Emergency stop |

### 9.2 Electrical Safety

| Precaution | Implementation |
|------------|----------------|
| Current limit | PSU protection |
| Fusing | All power lines fused |
| ESD | Standard ESD precautions |
| Grounding | Common ground point |

## 10. Maintenance

### 10.1 Weekly Maintenance

| Task | Action |
|------|--------|
| Backup | Save all logs and data |
| Verify | Run health check |
| Clean | Dust, organize cables |
| Document | Log any issues |

### 10.2 As-Needed Maintenance

| Trigger | Action |
|---------|--------|
| Component failure | Replace, document |
| SW update | Flash, verify, document |
| Configuration change | Document, notify team |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | Documentation audit (post-CDR 2026): planned-configuration banner; OBC interfaces from SQSO-OBC-BRD-001 (EPS on I2C2 at 0x48, was UART3 at 9600; Jetson USART1; console USART3; FRAM, no flash); sensors RM3100, M-G370PDT0, ADS7138 (were LIS3MDL and LSM6DSO breakouts); EPS rails from EPS_DESIGN; downlink the coordinated frequency (was 145.825 MHz, which BASELINE excludes); Rust toolchain (was STM32CubeIDE); Jetson 7 W mode and VBAT_JET | Documentation audit |
