# Flatsat Configuration
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-INT-004
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA Ames SmallSat Development Heritage

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
| Clock | Same as flight (80 MHz) |
| Memory | Same as flight |
| Debug | ST-Link integrated |
| Interfaces | All flight interfaces broken out |

**Connections:**
- UART1: Debug console (USB)
- UART2: Jetson interface
- UART3: EPS interface
- I2C1: Sensors (mag, temp)
- SPI1: Flash memory
- GPIO: Deployment switches, LEDs

### 4.2 EPS Development Board

| Parameter | Specification |
|-----------|---------------|
| Board | Custom EPS dev board |
| Battery | Flight-equivalent cells |
| MPPT | Flight-equivalent IC |
| Power rails | 3.3V, 5V, Battery |
| Telemetry | All EPS channels |
| Load switches | All flight switches |

**Solar Simulation:**
- Keithley 2400 or equivalent
- Programmable I-V curve
- Orbital profile simulation

### 4.3 ADCS Development

| Sensor | Board | Interface |
|--------|-------|-----------|
| Magnetometer | LIS3MDL breakout | I2C |
| Sun sensors | Photodiode array | ADC |
| Gyroscope | LSM6DSO breakout | I2C |

**Magnetorquer Simulation:**
- H-bridge driver boards
- Current sense resistors
- Helmholtz cage interface (optional)

### 4.4 RF Development

| Parameter | Specification |
|-----------|---------------|
| SDR | RTL-SDR or USRP |
| Frequency | 145.825 MHz |
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
| Board | Jetson Orin Nano Dev Kit |
| Interface | USB-UART adapter |
| Power | Separate 5V supply |
| Storage | Same SD card image |
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
| STM32CubeIDE | OBC development |
| Serial terminal | UART monitoring |
| GNURadio | RF signal processing |
| Python scripts | Automated testing |
| Git | Version control |
| Jenkins/CI | Automated builds |

## 6. Interface Definitions

### 6.1 Debug Console (UART1)

| Parameter | Value |
|-----------|-------|
| Baud rate | 115200 |
| Data bits | 8 |
| Parity | None |
| Stop bits | 1 |
| Flow control | None |

### 6.2 Jetson Interface (UART2)

| Parameter | Value |
|-----------|-------|
| Baud rate | 115200 |
| Protocol | Custom framed |
| Connector | USB-UART adapter |

### 6.3 EPS Interface (UART3)

| Parameter | Value |
|-----------|-------|
| Baud rate | 9600 |
| Protocol | Command/response |

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
