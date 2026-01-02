# Interface Control Document: OBC-EPS

**Document Number**: SMART-QSO-ICD-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

---

## 1. Introduction

### 1.1 Purpose

This Interface Control Document (ICD) defines the hardware and software interface between the On-Board Computer (OBC) and the Electrical Power Subsystem (EPS).

### 1.2 Scope

This document covers:
- Physical/electrical interface specifications
- I2C communication protocol
- Register map and commands
- Power control interface
- Telemetry data formats

### 1.3 Interface Diagram

```
+----------------+                    +------------------+
|                |      I2C Bus       |                  |
|      OBC       |<------------------>|       EPS        |
| (STM32L4/      |     SCL/SDA        |   Board          |
|  Apollo4)      |                    |                  |
|                |     Load Switch    |   +----------+   |
|                |------------------->|   | Battery  |   |
|                |     Control GPIO   |   +----------+   |
|                |                    |   +----------+   |
|                |     Alert GPIO     |   | Solar    |   |
|                |<-------------------|   | Panels   |   |
|                |                    |   +----------+   |
+----------------+                    +------------------+
```

---

## 2. Physical Interface

### 2.1 Connector Specification

| Parameter | OBC Side | EPS Side |
|-----------|----------|----------|
| Connector | PC-104 Stack | PC-104 Stack |
| I2C Pins | SCL (Pin H2-5), SDA (Pin H2-6) | SCL, SDA |
| Power | 3.3V (Pin H1-1,2) | 3.3V regulated |
| GND | GND (Pin H1-49,50) | GND |
| Load Ctrl | GPIO (Pin H2-10) | Load Switch Enable |
| Alert | GPIO (Pin H2-11) | Battery Alert |

### 2.2 Electrical Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| I2C SCL frequency | - | 100 | 400 | kHz |
| I2C Logic High | 2.1 | 3.3 | 3.6 | V |
| I2C Logic Low | 0 | 0 | 0.6 | V |
| I2C Pull-up | 2.2 | 4.7 | 10 | kΩ |
| Load Control High | 2.4 | 3.3 | 3.6 | V |
| Load Control Low | 0 | 0 | 0.4 | V |

### 2.3 I2C Configuration

| Parameter | Value |
|-----------|-------|
| Mode | Standard/Fast |
| Clock Speed | 100 kHz (default) |
| Address Mode | 7-bit |
| EPS Address | 0x48 |
| Battery Monitor Address | 0x55 |
| Solar Monitor Address | 0x40 |

---

## 3. Power Interface

### 3.1 Power Rails

| Rail | Voltage | Current Max | Purpose |
|------|---------|-------------|---------|
| 3.3V_OBC | 3.3V ±5% | 200 mA | OBC logic |
| 5V_BUS | 5.0V ±5% | 3.0 A | Jetson payload |
| 3.3V_RF | 3.3V ±5% | 500 mA | RF subsystem |
| VBATT | 6.0-8.4V | 2.0 A | Raw battery |

### 3.2 Load Switch Control

| Load | GPIO Pin | Active | Max Current |
|------|----------|--------|-------------|
| Payload (5V) | LOAD_EN_1 | High | 3.0 A |
| RF System | LOAD_EN_2 | High | 500 mA |
| Heater | LOAD_EN_3 | High | 200 mA |
| ADCS | LOAD_EN_4 | High | 100 mA |

### 3.3 Power Sequencing

#### Boot Sequence
```
T=0:      EPS enables 3.3V_OBC
T+10ms:   OBC starts boot
T+100ms:  OBC initializes I2C
T+200ms:  OBC reads EPS status
T+500ms:  OBC enables loads per mode
```

#### Safe Mode Entry
```
T=0:      Alert pin asserted OR SOC < 25%
T+10ms:   OBC disables non-essential loads
T+50ms:   OBC enters SAFE mode
T+100ms:  Only beacon transmissions active
```

---

## 4. Communication Protocol

### 4.1 I2C Transaction Format

#### Write Transaction
```
START | ADDR+W | REG | DATA[0] | ... | DATA[n] | STOP
```

#### Read Transaction
```
START | ADDR+W | REG | RESTART | ADDR+R | DATA[0] | ... | DATA[n] | STOP
```

### 4.2 Register Map - EPS Controller (0x48)

| Register | Address | R/W | Size | Description |
|----------|---------|-----|------|-------------|
| STATUS | 0x00 | R | 1 | System status |
| MODE | 0x01 | R/W | 1 | Power mode |
| LOAD_EN | 0x02 | R/W | 1 | Load enables |
| VBATT | 0x10 | R | 2 | Battery voltage (mV) |
| IBATT | 0x12 | R | 2 | Battery current (mA, signed) |
| TBATT | 0x14 | R | 2 | Battery temp (0.1°C) |
| SOC | 0x16 | R | 1 | State of charge (%) |
| VSOLAR | 0x20 | R | 2 | Solar voltage (mV) |
| ISOLAR | 0x22 | R | 2 | Solar current (mA) |
| PSOLAR | 0x24 | R | 2 | Solar power (mW) |
| VBUS | 0x30 | R | 2 | 5V bus voltage (mV) |
| IBUS | 0x32 | R | 2 | 5V bus current (mA) |
| ENERGY | 0x40 | R | 4 | Energy consumed (mWh) |
| BOOT_CNT | 0x50 | R | 2 | Boot counter |
| ALERT_CFG | 0x60 | R/W | 4 | Alert configuration |
| VERSION | 0xF0 | R | 2 | Firmware version |

### 4.3 Status Register (0x00)

| Bit | Name | Description |
|-----|------|-------------|
| 7 | CHARGING | Battery charging active |
| 6 | SUNLIT | Solar power available |
| 5 | LOW_BATT | Low battery warning |
| 4 | OVER_TEMP | Over temperature |
| 3 | UNDER_TEMP | Under temperature |
| 2 | FAULT | Fault condition active |
| 1:0 | MODE | Current power mode |

### 4.4 Mode Register (0x01)

| Value | Mode | Description |
|-------|------|-------------|
| 0x00 | SAFE | Minimum power, beacon only |
| 0x01 | IDLE | Normal operations, no payload |
| 0x02 | ACTIVE | Full operations, payload enabled |

### 4.5 Load Enable Register (0x02)

| Bit | Name | Default | Description |
|-----|------|---------|-------------|
| 7:4 | Reserved | 0 | Reserved |
| 3 | ADCS_EN | 0 | ADCS power enable |
| 2 | HEATER_EN | 0 | Heater enable |
| 1 | RF_EN | 1 | RF system enable |
| 0 | PAYLOAD_EN | 0 | Jetson payload enable |

---

## 5. Data Formats

### 5.1 Voltage Format

16-bit unsigned, LSB first, units of millivolts (mV)

```
Example: VBATT = 0x1F40 = 8000 mV = 8.0V
```

### 5.2 Current Format

16-bit signed (two's complement), LSB first, units of milliamps (mA)

```
Example (positive): IBATT = 0x03E8 = +1000 mA (charging)
Example (negative): IBATT = 0xFC18 = -1000 mA (discharging)
```

### 5.3 Temperature Format

16-bit signed, LSB first, units of 0.1°C

```
Example: TBATT = 0x00FA = 250 = 25.0°C
Example: TBATT = 0xFF38 = -200 = -20.0°C
```

### 5.4 State of Charge Format

8-bit unsigned, units of percent (%)

```
Example: SOC = 0x4B = 75%
```

### 5.5 Energy Format

32-bit unsigned, LSB first, units of milliwatt-hours (mWh)

```
Example: ENERGY = 0x00001388 = 5000 mWh = 5.0 Wh
```

---

## 6. Command Interface

### 6.1 Set Power Mode

**Write to MODE register (0x01)**

| Byte | Value | Description |
|------|-------|-------------|
| 0 | 0x00-0x02 | Mode value |

**Response**: ACK if valid, NACK if invalid mode

### 6.2 Set Load Enables

**Write to LOAD_EN register (0x02)**

| Byte | Value | Description |
|------|-------|-------------|
| 0 | 0x00-0x0F | Enable bits |

**Constraints**:
- PAYLOAD_EN only settable in ACTIVE mode
- RF_EN always enabled in all modes

### 6.3 Configure Alerts

**Write to ALERT_CFG register (0x60)**

| Byte | Description |
|------|-------------|
| 0-1 | Low voltage threshold (mV) |
| 2-3 | High temperature threshold (0.1°C) |

---

## 7. Telemetry Collection

### 7.1 Polling Sequence

```
1. Read STATUS (0x00) - 1 byte
2. Read VBATT (0x10) - 2 bytes
3. Read IBATT (0x12) - 2 bytes
4. Read TBATT (0x14) - 2 bytes
5. Read SOC (0x16) - 1 byte
6. Read VSOLAR (0x20) - 2 bytes
7. Read ISOLAR (0x22) - 2 bytes
8. Read VBUS (0x30) - 2 bytes
```

### 7.2 Polling Rate

| Parameter | Rate | Requirement |
|-----------|------|-------------|
| Battery voltage | 1 Hz | SRS-F010 |
| Battery current | 1 Hz | SRS-I011 |
| Battery temp | 0.2 Hz | SRS-I011 |
| SOC | 1 Hz | SRS-F010 |
| Solar | 1 Hz | SRS-I012 |
| Bus | 1 Hz | SRS-I011 |

### 7.3 Telemetry Format for Beacon

```
BV=<vbatt>V,BI=<ibatt>mA,BT=<tbatt>C,SOC=<soc>%,SV=<vsolar>V,SI=<isolar>mA
```

Example:
```
BV=7.8V,BI=-500mA,BT=22.5C,SOC=68%,SV=9.2V,SI=450mA
```

---

## 8. Error Handling

### 8.1 I2C Error Handling

| Error | Detection | Recovery |
|-------|-----------|----------|
| NACK | No ACK after address | Retry (3x), log fault |
| Timeout | No response within 100ms | Reset I2C, retry |
| Arbitration | Multi-master conflict | Retry after delay |
| Bus stuck | SDA or SCL stuck low | Clock recovery sequence |

### 8.2 I2C Recovery Sequence

```
1. Generate 9 clock pulses on SCL
2. Generate STOP condition
3. Re-initialize I2C peripheral
4. Retry transaction
```

### 8.3 Alert Handling

When Alert GPIO is asserted:
1. Read STATUS register immediately
2. Identify alert source
3. Take corrective action per fault type
4. Log fault event

---

## 9. Timing Requirements

### 9.1 I2C Timing

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Transaction time | - | 2 | 10 | ms |
| Bus turnaround | 10 | - | - | μs |
| Retry delay | 10 | - | - | ms |

### 9.2 Alert Response

| Parameter | Max | Unit |
|-----------|-----|------|
| Alert to read | 50 | ms |
| Alert to load disable | 100 | ms |
| Alert to mode change | 500 | ms |

---

## 10. Test Requirements

### 10.1 Interface Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| IFT-EPS-001 | I2C connectivity | ACK received |
| IFT-EPS-002 | Register read | All registers readable |
| IFT-EPS-003 | Mode setting | Mode changes correctly |
| IFT-EPS-004 | Load control | Loads respond to enables |
| IFT-EPS-005 | Alert detection | Alert GPIO functional |
| IFT-EPS-006 | Error recovery | Recovery from bus errors |

### 10.2 Integration Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| INT-EPS-001 | Power mode sequence | All transitions verified |
| INT-EPS-002 | Telemetry accuracy | ±2% of reference |
| INT-EPS-003 | Load switch timing | Within spec |
| INT-EPS-004 | Fault response | Correct mode on fault |

---

## Appendix A: Reference Documents

1. STM32L4 Reference Manual (RM0351)
2. I2C-bus specification (UM10204)
3. SMART-QSO Software Requirements Specification
4. SMART-QSO EPS Design Document

---

## Appendix B: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
