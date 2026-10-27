# SMART-QSO Interface Verification Plan

**Document ID**: SQSO-IVP-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Plan. No interface has been built or verified; no independent CDR has been held (BASELINE Rev E §1)

## 1. Purpose

This document defines the Interface Verification Plan for SMART-QSO, establishing the approach for verifying all internal and external interfaces per NPR 7123.1.

## 2. Interface Architecture

### 2.1 Interface Categories

| Category | Description | Count |
|----------|-------------|-------|
| Internal electrical | Between subsystems | 6 |
| Internal mechanical | Physical mounting | 8 |
| Internal software | Data protocols | 4 |
| External electrical | Ground/deployer | 2 |
| External RF | Ground station | 2 |
| External mechanical | Deployer | 1 |

### 2.2 Interface Hierarchy

Values below are summaries; the ICDs and the master interconnect (`hardware/electrical/INTERCONNECT.md`, SQSO-ICX-001 D)
govern. Nothing has been built: every verification below is planned on the engineering model (EM) and the flight
model.

```
INTERFACE HIERARCHY

External Interfaces
├── Dispenser: Exolaunch EXOpod Nova 1U slot (mechanical, separation switches)
├── Ground stations (RF, SQSO-ICD-005; telecommand SQSO-ICD-013)
└── GSE access port (OBC J9 through board A3, Micro-D 21)

Internal Interfaces
├── OBC ←→ EPS (I2C2 + discretes + analog, W3; SMART-QSO-ICD-001)
├── OBC ←→ RF (USART2 KISS TNC + discretes + monitors, W4A; SQSO-ICD-003)
├── OBC ←→ Jetson carrier (USART1, power-good, spreader NTC, W5B; SMART-QSO-ICD-002)
├── OBC ←→ ADCS devices (on the OBC board; rods, photodiodes, magnetometer board on W6; SQSO-ICD-004)
├── OBC ←→ GNSS receiver (LPUART1, 1PPS, W9)
├── EPS ←→ Solar array, battery, deployment switches (W1, W2, W12)
├── RF ←→ ISIS AntS (coax W10; AntS I2C3 through the EPS, W7)
└── All ←→ Structure (mechanical)
```

## 3. Internal Interfaces

### 3.1 OBC-EPS Interface (IF-001)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Protocol | I2C2, 100 kHz (designed for 400 kHz), 7-bit | Test |
| Voltage | 3.3 V logic | Analysis |
| Addresses | EPS controller 0x48; INA228 pack gauge 0x44, array monitor 0x40 | Test |
| Discretes | 15 load enables, 4 burn lines, EOL_ARM/EOL_FIRE, LOCK_CLR, DEPLOY_SENSE, SOLAR_SENSE, EPS_ALERT_N; 7 analog lines | Test |
| Connector | W3: OBC J1 to EPS J3, 40-way Samtec TFM-120 with an SFSDT 30 AWG assembly | Inspection |

Pin list: SMART-QSO-ICD-001 §2 and SQSO-ICX-001 §3.1.

**Verification Tests**:
- [ ] I2C communication with each device at 100 kHz
- [ ] All registers of SMART-QSO-ICD-001 §4 readable
- [ ] Each load enable switches its rail; each latch-up trip at its limit
- [ ] Alert and sense lines read with the polarity of SQSO-ICX-001 §3.7

### 3.2 OBC-RF Interface (IF-002)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Data link | USART2 (PD5/PD6), 9600 8N1, KISS to the TNC on the RF board | Test |
| Voltage | 3.3 V out; 3.1 V CMOS in | Analysis |
| Discretes | TX_PWR_SEL, XPDR_EN heartbeat; readbacks TONE_OK, XPDR_KEY, PTT_EXCITER, RX_CD, RX_FAULT; RADIO_EN and RF_TX_EN switch the RF rails on the EPS | Test |
| Monitors | TX_TEMP, TX_FWD, TX_REV (analog) | Test |
| Connector | W4A: OBC J2 to RF J1, 16-way Samtec TFM-108; power on W4B | Inspection |

Pin list: SQSO-ICD-003 §4 and SQSO-ICX-001 §3.2.

**Verification Tests**:
- [ ] KISS frames of 1–328 bytes byte-exact through the EM TNC
- [ ] RF_TX_EN low, high-Z and OBC reset leave the PA off (VI-RF-12)
- [ ] Transponder unkeys within 100 ms of a stopped heartbeat (VI-RF-11)
- [ ] Monitors calibrated

### 3.3 OBC-Jetson Interface (IF-003)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Protocol | USART1 (PA9/PA10), 115200 baud; the OBC transmits 8N2 | Test |
| Voltage | 3.3 V at the OBC; translated to the module's 1.8 V on the carrier; isolated with the carrier off | Analysis and test (V-OBC-9) |
| Messages | ASCII lines with CRC-16 (SMART-QSO-ICD-002 §4) | Test |
| Power | VBAT through SW1 (PAYLOAD_EN) and the eFuse (JETSON_PWR) on the EPS; power-good JET_PG | Test |
| Connector | W5B: OBC J3 to carrier J2, Pico-Clasp 6 (with the spreader thermistor); power on W5A | Inspection |

**Verification Tests**:
- [ ] Golden lines of SMART-QSO-ICD-002 §4.3 exchanged
- [ ] Switch-on, power-good and commanded switch-off (SMART-QSO-ICD-002 §10.4)
- [ ] Beacon request and response; template fallback on timeout
- [ ] TX idle with the carrier off: carrier 3.3 V rail below 0.3 V

### 3.4 OBC-ADCS Devices Interface (IF-004)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Gyros | Epson M-G370PDT0 on SPI3 (CS PD1); Bosch BMI088 backup on SPI2 | Test |
| Magnetometer | PNI RM3100, I2C4 0x20, on board A2 via W6E | Test |
| Sun sensors | Six panel photodiodes into the ADS7138-Q1 (I2C4 0x10) | Test |
| Magnetorquers | Three CubeSpace CR0002 rods on DRV8876-Q1 bridges (TIM1 PWM, TIM15 900 ms burst) | Test |

Pin list: SQSO-ICD-004 §4 and SQSO-ICX-001 §3.4, §4.

**Verification Tests**:
- [ ] Each device answers with its ID (AV-V1, AV-V2, AV-V5)
- [ ] Coil burst ends by itself (AV-V3)
- [ ] Magnetometer samples only in the coils-off window
- [ ] Calibration telecommands accepted and persisted

### 3.5 EPS Power Interfaces (IF-005, IF-006)

| Interface | Type | Verification |
|-----------|------|--------------|
| Solar input | 16 cells in strings to four trackers | Test |
| Battery | 4S1P Li-ion, 14.4 V nominal, charged to 4.10 V/cell (16.4 V) (BASELINE §5) | Test |
| VBAT bus | 11.2–16.4 V | Test |
| 3.3 V rails | 3V3_AO and the switched rails | Test |
| 5V_PA | 5.0 V, PA only | Test |

**Verification Tests**:
- [ ] Tracker operation
- [ ] Battery charge/discharge inside 0 to +45 °C
- [ ] Rail regulation accuracy
- [ ] Current limit function

### 3.6 RF-Antenna Interface (IF-007)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Antenna | ISIS AntS 1U–3U: 2 m and 70 cm quarter-wave tape monopoles (BASELINE §7) | Inspection |
| Connector | MMCX on the RF board, coax W10 (RF board C §4.3) | Inspection |
| Impedance | 50 Ω | Test |
| Frequency | 145.8–146.0 MHz; 435–438 MHz | Test |

**Verification Tests**:
- [ ] VSWR measurement, antennas deployed
- [ ] Third harmonic at the antenna port ≥ 70 dB below the carrier (BASELINE §7)
- [ ] Deployment by the AntS I2C sequence (AV-V7)

## 4. External Interfaces

### 4.1 Dispenser Interface (IF-EXT-001)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Envelope | 1U, 100 × 100 × 113.5 mm (CDS Rev 14.1) | Inspection |
| Rails | CDS Rev 14.1 and the EXOpod Nova user manual (SQSO-LVR-001) | Inspection |
| Mass | CBE 1.52 kg, MEV 1.75 kg; limit 2.00 kg (SYS-002); Nova 1U 2.5 kg (BASELINE §4) | Test |
| CG location | Within ±20 mm of the geometric centre (CAD within 5 mm) | Test |
| Deployment switches | Three, on three rails (DS1–DS3; BASELINE §4) | Test |

**Verification Tests**:
- [ ] Fit check in the dispenser or its test pod
- [ ] Mass properties (VI-MCAD-03)
- [ ] Deployment switch and inhibit function (VI-LV-06)
- [ ] Separation velocity (integrator data)

### 4.2 Ground Station RF Interface (IF-EXT-002)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Downlink | One IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz | Test |
| Uplink | One IARU-coordinated frequency, requested in 435.222–435.778 MHz, else 436.222–436.500 MHz | Test |
| Modulation | 1200 bps AFSK (10K4F2D); FM transponder (16K0F3E) | Test |
| Protocol | AX.25 UI; telecommand HMAC-SHA-256 (SQSO-ICD-013) | Test |
| Power | 0.5 W nominal, 1 W maximum | Test |

**Verification Tests**:
- [ ] Beacon transmission
- [ ] Command reception at the budget's levels (SQSO-ICD-005 §10)
- [ ] TX-OFF end to end
- [ ] Protocol compliance

### 4.3 GSE Interface (IF-EXT-003)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Access | OBC J9 through the access-port board A3, Micro-D 21 (SQSO-ICX-001 §3.6) | Inspection |
| Debug | SWD, NRST | Test |
| Serial | Console UART; key provisioning and `ARM_FLIGHT` (SQSO-ICD-013 §9.2) | Test |
| Charging | Charge input through W11 | Test |
| RF | Conducted, into an attenuator | Test |

**Verification Tests**:
- [ ] Charge path does not energize the loads (VI-LV-15)
- [ ] Debug interface
- [ ] Key provisioning only with the umbilical sense high
- [ ] Telemetry capture

## 5. Verification Matrix

### 5.1 Interface Verification Summary

| IF ID | Interface | Method | Procedure | Status |
|-------|-----------|--------|-----------|--------|
| IF-001 | OBC-EPS | T | TP-INT-001 | [ ] |
| IF-002 | OBC-RF | T | TP-INT-003 | [ ] |
| IF-003 | OBC-Jetson | T | TP-INT-005 | [ ] |
| IF-004 | OBC-Sensors | T | TP-INT-004 | [ ] |
| IF-005 | EPS-Solar | T | TP-EPS-004 | [ ] |
| IF-006 | EPS-Battery | T | TP-EPS-003 | [ ] |
| IF-007 | RF-Antenna | T | TP-RF-002 | [ ] |
| IF-EXT-001 | Deployer | T,I | TP-MECH-001 | [ ] |
| IF-EXT-002 | Ground RF | T | TP-E2E-001 | [ ] |
| IF-EXT-003 | EGSE | T | TP-GSE-001 | [ ] |

### 5.2 Verification Methods

| Method | Application |
|--------|-------------|
| Test (T) | Functional verification |
| Inspection (I) | Physical verification |
| Analysis (A) | Design verification |
| Demonstration (D) | Operational verification |

## 6. Verification Process

### 6.1 Interface Test Flow

```
INTERFACE VERIFICATION FLOW

1. ICD Review
   └─ Verify interface definition

2. Test Preparation
   └─ Configure test environment
   └─ Prepare test equipment

3. Electrical Verification
   └─ Signal levels
   └─ Timing
   └─ Protocol compliance

4. Functional Verification
   └─ Data transfer
   └─ Command response
   └─ Error handling

5. Integration Verification
   └─ System-level operation
   └─ End-to-end function

6. Documentation
   └─ Test results
   └─ Issue resolution
   └─ Closure
```

### 6.2 Interface Test Procedure Template

```
INTERFACE TEST PROCEDURE

Interface: IF-XXX
Date: ___________
Tester: ___________

PRE-TEST:
[ ] ICD reviewed
[ ] Equipment ready
[ ] Software loaded
[ ] Connections verified

TEST STEPS:
Step | Action           | Expected      | Actual | Pass
─────┼──────────────────┼───────────────┼────────┼──────
  1  |                  |               |        | [ ]
  2  |                  |               |        | [ ]
  3  |                  |               |        | [ ]

POST-TEST:
[ ] Data recorded
[ ] Issues documented
[ ] Report completed

Result: [ ] PASS  [ ] FAIL

Signature: _________ Date: _____
```

## 7. Issue Resolution

### 7.1 Interface Issues

| Issue Type | Resolution |
|------------|------------|
| Signal level | Adjust/add buffer |
| Timing | Adjust clock/delays |
| Protocol | Software fix |
| Connector | Replace/rework |
| Impedance | Match network |

### 7.2 Issue Tracking

| Field | Content |
|-------|---------|
| IF ID | Interface identifier |
| Issue | Description |
| Root cause | Analysis |
| Resolution | Fix applied |
| Verification | Retest |
| Status | Open/Closed |

## 8. Schedule

### 8.1 Interface Verification Timeline

| Phase | Interfaces | Timing |
|-------|------------|--------|
| Engineering model and flatsat | All internal | PDR-CDR (`mission/SCHEDULE.md`) |
| Flatsat | All internal | CDR-TRR |
| Integration | All | TRR-PER |
| System | External | PER-FRR |

### 8.2 Verification Milestones

| Milestone | Criteria |
|-----------|----------|
| PDR | Interface definitions complete |
| CDR | ICDs baselined |
| TRR | Internal interfaces verified |
| FRR | All interfaces verified |

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): status a plan, nothing verified; §2.2 to §4.3 rewritten to the interfaces as designed (I2C2 at 0x48/0x40/0x44 and W3 to the EPS; USART2 KISS to the RF board, not SPI; USART1 8N2 to the Jetson carrier; M-G370, BMI088, RM3100, ADS7138 and CR0002 rods; 4S battery; ISIS AntS; EXOpod Nova, three deployment switches; the BASELINE §7 frequencies and power), replacing the 2026-01 DF13/DF12 headers, the 0x50 EPS address, the 2S cell range and 0.5–2 W; §8.1 EM and flatsat in place of a breadboard |
