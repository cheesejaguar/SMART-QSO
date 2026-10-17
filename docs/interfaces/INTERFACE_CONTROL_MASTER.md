# Interface Control Document - Master
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ICD-000
**Revision:** C
**Date:** 2026-01-02
**Reference:** NPR 7123.1, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document provides the master index and overview of all interface control documents (ICDs) for the SMART-QSO CubeSat mission, establishing interface ownership, configuration control, and verification approach.

## 2. Scope

This document covers all internal and external interfaces for the SMART-QSO spacecraft, including:
- Internal hardware interfaces
- Internal software interfaces
- External mechanical interfaces
- External RF interfaces
- Ground segment interfaces

---

## 3. Interface Control Documents Index

### 3.1 Internal Hardware ICDs

Numbers are the ones each ICD carries in its own header; `docs/DOCUMENT_REGISTER.md` is the controlled list. Revision A of this
index numbered the ICDs differently (SQSO-ICD-001 OBC–Jetson, -002 OBC–EPS, -003 beacon format, -005 OBC–RF, -006 EPS
power bus), so the same number meant two documents. Citations of the old numbers were corrected on 2026-10-16 (finding
PI-17). Some ICDs carry the prefix SMART-QSO- and others SQSO-; both are listed as written.

| ICD Number | Title | Status | Owner |
|------------|-------|--------|-------|
| SMART-QSO-ICD-001 | OBC to EPS Interface | Draft | C&DH |
| SMART-QSO-ICD-002 | OBC to Jetson Interface | Draft | C&DH |
| SQSO-ICD-003 | OBC to RF Interface | Draft | RF |
| SQSO-ICD-004 | OBC to ADCS Interface | Draft | ADCS |
| SMART-QSO-ICD-006 | Beacon Format Interface | Draft | RF |
| — | EPS Power Bus Interface | Not written; no number assigned | EPS |

### 3.2 External ICDs

| ICD Number | Title | Status | Owner |
|------------|-------|--------|-------|
| SQSO-ICD-005 | Space-to-Ground Interface | Draft | Ground |
| SQSO-ICD-010 | Launch Vehicle Interface (Launch Provider ICD) | Draft; no provider selected | Systems |
| SQSO-ICD-011 | Deployer Interface | Not written | Mechanical |
| SQSO-ICD-012 | Ground Station Interface | Not written (the space-to-ground link is SQSO-ICD-005) | Ground |
| SQSO-ICD-013 | Telecommand Authentication and TX-OFF | Draft | C&DH |

### 3.3 Document Locations

| Document | Location |
|----------|----------|
| ICD OBC-EPS (SMART-QSO-ICD-001) | docs/interfaces/ICD_OBC_EPS.md |
| ICD OBC-Jetson (SMART-QSO-ICD-002) | docs/interfaces/ICD_OBC_JETSON.md |
| ICD OBC-RF (SQSO-ICD-003) | docs/interfaces/ICD_OBC_RF.md |
| ICD OBC-ADCS (SQSO-ICD-004) | docs/interfaces/ICD_OBC_ADCS.md |
| ICD Space-to-Ground (SQSO-ICD-005) | docs/interfaces/ICD_SPACE_GROUND.md |
| ICD Beacon Format (SMART-QSO-ICD-006) | docs/interfaces/ICD_BEACON_FORMAT.md |
| Launch Provider ICD (SQSO-ICD-010) | docs/compliance/LAUNCH_PROVIDER_ICD.md |
| ICD Telecommand Authentication (SQSO-ICD-013) | docs/interfaces/TELECOMMAND_AUTHENTICATION.md |

---

## 4. Interface Ownership Matrix

### 4.1 Hardware Interfaces

| Interface | Provider | User | Owner | CCB |
|-----------|----------|------|-------|-----|
| OBC ↔ Jetson | C&DH | Payload | C&DH | SW CCB |
| OBC ↔ EPS | EPS | C&DH | EPS | HW CCB |
| OBC ↔ ADCS | ADCS | C&DH | ADCS | HW CCB |
| OBC ↔ RF TX | RF | C&DH | RF | HW CCB |
| EPS → All | EPS | All | EPS | HW CCB |
| Structure → All | Mechanical | All | Mech | HW CCB |

### 4.2 Software Interfaces

| Interface | Provider | User | Owner | CCB |
|-----------|----------|------|-------|-----|
| FSW API | OBC SW | Subsystems | SW Lead | SW CCB |
| Telemetry Format | OBC SW | Ground | SW Lead | SW CCB |
| Beacon Protocol | OBC SW | RF | SW Lead | SW CCB |
| Jetson Protocol | Payload SW | OBC SW | SW Lead | SW CCB |

---

## 5. Interface Block Diagram

```
                         SMART-QSO Interface Architecture
┌─────────────────────────────────────────────────────────────────────────┐
│                              SPACECRAFT                                  │
│                                                                          │
│  ┌─────────────┐                                    ┌─────────────┐     │
│  │   SOLAR     │────────────────────────────────────│   BATTERY   │     │
│  │   ARRAY     │                                    │             │     │
│  └──────┬──────┘                                    └──────┬──────┘     │
│         │                                                  │            │
│         └─────────────────┬────────────────────────────────┘            │
│                           │                                              │
│                    ┌──────▼──────┐                                      │
│                    │     EPS     │                                      │
│                    │ (SMART-QSO  │                                      │
│                    │  -ICD-001)  │                                      │
│                    └──────┬──────┘                                      │
│                           │ Power Bus                                   │
│         ┌─────────────────┼─────────────────┬───────────────┐          │
│         │                 │                 │               │          │
│  ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐ ┌──────▼──────┐   │
│  │    OBC      │   │    ADCS     │   │   RF TX     │ │   JETSON    │   │
│  │             │◄──┤  (SQSO-ICD  │   │  (SQSO-ICD  │ │ (SMART-QSO  │   │
│  │             │   │    -004)    │   │    -003)    │ │  -ICD-002)  │   │
│  └──────┬──────┘   └─────────────┘   └──────┬──────┘ └──────┬──────┘   │
│         │                                   │               │          │
│         │           SQSO-ICD-003            │               │          │
│         └───────────────────────────────────┘               │          │
│                           │                                 │          │
│                           │ UART (SMART-QSO-ICD-002)        │          │
│                           └─────────────────────────────────┘          │
│                                                                         │
│  ┌─────────────┐                                                       │
│  │  ANTENNA    │◄──── RF                                               │
│  └──────┬──────┘                                                       │
│         │                                                               │
└─────────┼───────────────────────────────────────────────────────────────┘
          │ 145.825 MHz
          │
          ▼
    ┌─────────────┐
    │   GROUND    │
    │   STATION   │
    │ (SQSO-ICD   │
    │   -012)     │
    └─────────────┘
```

---

## 6. Interface Summary Tables

### 6.1 Electrical Interfaces

| Interface | Type | Voltage | Protocol | Connector |
|-----------|------|---------|----------|-----------|
| OBC-Jetson | UART | 3.3V | Custom | 4-pin header |
| OBC-EPS | UART | 3.3V | Command/Response | 4-pin header |
| OBC-ADCS | I2C | 3.3V | Standard I2C | 6-pin header |
| OBC-RF | GPIO+SPI | 3.3V | Control + Data | 10-pin header |
| Power Bus | DC | 3.3V/5V/VBAT | N/A | Harness |

### 6.2 Data Interfaces

| Interface | Data Rate | Format | Direction |
|-----------|-----------|--------|-----------|
| OBC-Jetson | 115200 bps | JSON/Text | Bidirectional |
| OBC-EPS | 9600 bps | Binary | Bidirectional |
| OBC-ADCS | 400 kHz | I2C Registers | Bidirectional |
| Beacon | 1200 bps | AX.25/AFSK | TX only |
| Telemetry | 1200 bps | AX.25/AFSK | TX only |

### 6.3 Mechanical Interfaces

| Interface | Standard | Tolerance |
|-----------|----------|-----------|
| Deployer | Cal Poly CDS Rev 14.1 | Per CDS |
| Rail | PC-104 footprint | ±0.1 mm |
| Connectors | Samtec, Hirose | Per datasheet |

---

## 7. Interface Change Control

### 7.1 Change Process

1. Change request submitted to interface owner
2. Impact assessment by both parties
3. Review by appropriate CCB (HW or SW)
4. Update ICD with new revision
5. Notify all affected parties
6. Update verification plan if needed

### 7.2 Change Categories

| Category | Definition | Approval |
|----------|------------|----------|
| Class I | Form, fit, or function change | CCB + PM |
| Class II | Documentation correction | CCB |
| Class III | Editorial only | Owner |

### 7.3 Interface Baseline

| Phase | Baseline Status |
|-------|-----------------|
| PDR | Preliminary (draft ICDs) |
| CDR | Baselined (frozen) |
| Post-CDR | Change control required |

---

## 8. Interface Verification

### 8.1 Verification Approach

| Interface | Method | When |
|-----------|--------|------|
| OBC-Jetson | Test | Integration |
| OBC-EPS | Test | Integration |
| OBC-ADCS | Test | Integration |
| OBC-RF | Test | Integration |
| Deployer | Test + Inspection | Pre-ship |
| Ground Station | Test | DITL |

### 8.2 Interface Verification Matrix

| ICD | Requirement | Test Procedure | Status |
|-----|-------------|----------------|--------|
| SMART-QSO-ICD-002 | UART timing | TP-INT-005 | Planned |
| SMART-QSO-ICD-001 | Power telemetry | TP-INT-001 | Planned |
| SMART-QSO-ICD-006 | AX.25 format | TP-RF-001 | Planned |
| SQSO-ICD-010 | Mechanical fit | TP-MECH-001 | Planned |

---

## 9. Interface Risk Summary

| Interface | Risk | Mitigation |
|-----------|------|------------|
| OBC-Jetson | Protocol timing | Early integration testing |
| EPS Power | Inrush current | Current limiting, soft start |
| RF-Antenna | Impedance mismatch | Matching network design |
| Deployer | Dimensional tolerance | Pre-ship fit check |

---

## 10. TBDs

| ID | Item | Owner | Target |
|----|------|-------|--------|
| TBD-ICM-001 | Launch provider ICD | Systems | Manifest |
| TBD-ICM-002 | Deployer interface | Mechanical | CDR+30d |
| TBD-ICM-003 | Ground station protocol | Ground | CDR |

---

## 11. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §3 index renumbered to the IDs the ICDs carry; Space-to-Ground and Launch Provider ICDs added; statuses set to Draft or Not written; diagram and §8.2 numbers corrected (finding PI-17) | SMART-QSO Team |
| C | 2026-10-16 | §10 TBD IDs made unique to this document: TBD-001 to TBD-003 became TBD-ICM-001 to TBD-ICM-003 (finding VV-17) | SMART-QSO Team |
