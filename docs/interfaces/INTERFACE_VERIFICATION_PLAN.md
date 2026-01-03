# SMART-QSO Interface Verification Plan

**Document ID**: SQSO-IVP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

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

```
INTERFACE HIERARCHY

External Interfaces
├── Deployer (mechanical)
├── Ground Station (RF)
└── EGSE (electrical)

Internal Interfaces
├── OBC ←→ EPS (I2C)
├── OBC ←→ RF (SPI)
├── OBC ←→ Jetson (UART)
├── OBC ←→ Sensors (I2C/SPI)
├── EPS ←→ Solar (analog)
├── EPS ←→ Battery (analog)
├── RF ←→ Antenna (coax)
└── All ←→ Structure (mechanical)
```

## 3. Internal Interfaces

### 3.1 OBC-EPS Interface (IF-001)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Protocol | I2C, 400 kHz | Test |
| Voltage | 3.3V logic | Analysis |
| Address | 0x50 | Test |
| Data rate | 10 Hz polling | Test |
| Connector | DF13-6P | Inspection |

**Signals**:
| Pin | Signal | Direction |
|-----|--------|-----------|
| 1 | VCC (3.3V) | EPS→OBC |
| 2 | GND | Common |
| 3 | SDA | Bidirectional |
| 4 | SCL | OBC→EPS |
| 5 | INT | EPS→OBC |
| 6 | Reserved | - |

**Verification Tests**:
- [ ] I2C communication at 400 kHz
- [ ] All telemetry points readable
- [ ] Command response verified
- [ ] Interrupt function verified

### 3.2 OBC-RF Interface (IF-002)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Protocol | SPI, 10 MHz | Test |
| Voltage | 3.3V logic | Analysis |
| Mode | Mode 0 (CPOL=0, CPHA=0) | Test |
| Connector | 10-pin header | Inspection |

**Signals**:
| Pin | Signal | Direction |
|-----|--------|-----------|
| 1 | VCC | OBC→RF |
| 2 | GND | Common |
| 3 | MOSI | OBC→RF |
| 4 | MISO | RF→OBC |
| 5 | SCK | OBC→RF |
| 6 | CS | OBC→RF |
| 7 | IRQ | RF→OBC |
| 8 | SDN | OBC→RF |
| 9 | GPIO1 | RF→OBC |
| 10 | GPIO2 | OBC→RF |

**Verification Tests**:
- [ ] SPI communication at 10 MHz
- [ ] Register read/write
- [ ] TX/RX functionality
- [ ] Interrupt handling

### 3.3 OBC-Jetson Interface (IF-003)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Protocol | UART, 115200 baud | Test |
| Voltage | 3.3V (isolated) | Analysis |
| Format | 8N1 | Test |
| Flow control | None | Test |
| Connector | DF12-10P | Inspection |

**Signals**:
| Pin | Signal | Direction |
|-----|--------|-----------|
| 1 | VCC_ISO | Isolated supply |
| 2 | GND_ISO | Isolated ground |
| 3 | TX | OBC→Jetson |
| 4 | RX | Jetson→OBC |
| 5 | PWR_EN | OBC→Jetson |
| 6 | STATUS | Jetson→OBC |
| 7-10 | Reserved | - |

**Verification Tests**:
- [ ] UART communication at 115200
- [ ] Power enable/disable
- [ ] AI request/response
- [ ] Timeout handling

### 3.4 OBC-Sensors Interface (IF-004)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Protocol | I2C (400 kHz), SPI | Test |
| Voltage | 3.3V logic | Analysis |
| Addresses | Per sensor spec | Test |

**Sensors**:
| Sensor | Interface | Address |
|--------|-----------|---------|
| Magnetometer | I2C | 0x1E |
| Gyroscope | I2C | 0x68 |
| Accelerometer | SPI | CS0 |
| Temperature | I2C | 0x48-0x4B |
| Sun sensors | Analog | ADC |

**Verification Tests**:
- [ ] Each sensor responds
- [ ] Data format correct
- [ ] Sample rate achieved
- [ ] Calibration verified

### 3.5 EPS Power Interfaces (IF-005, IF-006)

| Interface | Type | Verification |
|-----------|------|--------------|
| Solar input | Analog (0-18V) | Test |
| Battery | Power (3.0-4.2V) | Test |
| 3.3V rail | Regulated | Test |
| 5V rail | Regulated | Test |

**Verification Tests**:
- [ ] Solar MPPT operation
- [ ] Battery charge/discharge
- [ ] Rail regulation accuracy
- [ ] Current limit function

### 3.6 RF-Antenna Interface (IF-007)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Connector | SMA | Inspection |
| Impedance | 50Ω | Test |
| VSWR | <2.0 | Test |
| Frequency | 145/435 MHz | Test |

**Verification Tests**:
- [ ] VSWR measurement
- [ ] Antenna pattern
- [ ] Gain verification
- [ ] Deployment function

## 4. External Interfaces

### 4.1 Deployer Interface (IF-EXT-001)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Envelope | 100×100×113.5mm | Inspection |
| Rail dimensions | Per CDS Rev 14.1 | Inspection |
| Mass | ≤2.0 kg | Test |
| CG location | Within 2cm of center | Test |
| Deploy switches | 2× redundant | Test |

**Verification Tests**:
- [ ] Fit check in deployer
- [ ] Mass properties
- [ ] Deploy switch function
- [ ] Separation velocity

### 4.2 Ground Station RF Interface (IF-EXT-002)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Downlink freq | 145.XXX MHz | Test |
| Uplink freq | 435.XXX MHz | Test |
| Modulation | AFSK/FSK | Test |
| Protocol | AX.25 | Test |
| Power | 0.5-2W | Test |

**Verification Tests**:
- [ ] Beacon transmission
- [ ] Command reception
- [ ] Link budget verification
- [ ] Protocol compliance

### 4.3 EGSE Interface (IF-EXT-003)

| Parameter | Specification | Verification |
|-----------|---------------|--------------|
| Power | External supply | Test |
| Debug | J-Link | Test |
| Serial | USB-UART | Test |
| RF | Attenuated | Test |

**Verification Tests**:
- [ ] Power supply function
- [ ] Debug interface
- [ ] Test commanding
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
| Breadboard | All internal | PDR-CDR |
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
