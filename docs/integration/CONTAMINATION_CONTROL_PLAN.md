# SMART-QSO Contamination Control Plan

**Document ID**: SQSO-CCP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the Contamination Control Plan for SMART-QSO per NASA-STD-8739.6 (Procedures for the Control of Molecular and Particulate Contamination) and NASA Ames SmallSat heritage practices.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| NASA-STD-8739.6 | Contamination Control |
| NASA-HDBK-6017 | Contamination Control Handbook |
| IEST-STD-CC1246 | Product Cleanliness Levels |
| Federal Standard 209E | Clean Room Classes (historical) |
| ISO 14644 | Clean Room Standards |
| SQSO-ASM-001 | Assembly Procedure |

## 3. Contamination Requirements

### 3.1 Cleanliness Levels

| Hardware | Level | Rationale |
|----------|-------|-----------|
| Optical surfaces | Level 100A | Sun sensor accuracy |
| RF connectors | Level 300A | Contact reliability |
| Solar cells | Level 200A | Power output |
| General surfaces | Level 500 | Workmanship |
| Internal cavities | Level 750 | FOD prevention |

### 3.2 Particulate Limits

| Level | Particles/0.1m² |
|-------|-----------------|
| Level 100 | ≤1 particle >100µm |
| Level 200 | ≤7 particles >100µm |
| Level 300 | ≤43 particles >100µm |
| Level 500 | ≤1069 particles >100µm |
| Level 750 | Max >750µm |

### 3.3 Molecular Limits

| Parameter | Requirement |
|-----------|-------------|
| NVR (Non-Volatile Residue) | ≤A/2 (1mg/0.1m²) |
| Outgassing (TML) | <1.0% |
| Outgassing (CVCM) | <0.1% |

## 4. Contamination Sources

### 4.1 Particulate Sources

| Source | Control Method |
|--------|----------------|
| Personnel | Cleanroom garments, access limits |
| Materials | Pre-cleaned, bagged |
| Tools | Cleaned and verified |
| Environment | Cleanroom, HEPA filtered |
| Handling | Gloved, lint-free |

### 4.2 Molecular Sources

| Source | Control Method |
|--------|----------------|
| Fingerprints | Gloves mandatory |
| Oils/lubricants | Space-qualified only |
| Outgassing | Material screening |
| Adhesives | Controlled application |
| Cleaning solvents | IPA only, residue-free |

## 5. Cleanroom Requirements

### 5.1 Classification

| Area | Class | ISO | Activity |
|------|-------|-----|----------|
| Integration | 100,000 | ISO 8 | General assembly |
| Critical ops | 10,000 | ISO 7 | Optical, RF work |
| Storage | 100,000 | ISO 8 | Bagged hardware |

### 5.2 Environmental Controls

| Parameter | Requirement | Verification |
|-----------|-------------|--------------|
| Particle count | Per class | Daily monitor |
| Temperature | 20-25°C | Continuous |
| Humidity | 30-60% RH | Continuous |
| Positive pressure | ≥0.02" H2O | Continuous |
| Air changes | ≥20/hour | Certified |

### 5.3 Cleanroom Layout

```
CLEANROOM LAYOUT

┌──────────────────────────────────────────────┐
│  GOWNING ROOM (ISO 8)                        │
│  - Smocks, booties, caps                     │
│  - Glove dispensers                          │
│  - Tacky mats                                │
└───────────────────┬──────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────┐
│  INTEGRATION AREA (ISO 8)                    │
│                                              │
│  ┌─────────────┐   ┌─────────────┐          │
│  │  Work       │   │  Clean      │          │
│  │  Station 1  │   │  Storage    │          │
│  └─────────────┘   └─────────────┘          │
│                                              │
│  ┌─────────────┐   ┌─────────────┐          │
│  │  Work       │   │  Critical   │ (ISO 7)  │
│  │  Station 2  │   │  Ops Area   │          │
│  └─────────────┘   └─────────────┘          │
│                                              │
│  ┌─────────────────────────────────┐        │
│  │  Assembly Table (Laminar Flow)  │        │
│  └─────────────────────────────────┘        │
│                                              │
└──────────────────────────────────────────────┘
```

## 6. Personnel Requirements

### 6.1 Training

| Topic | Frequency | Required For |
|-------|-----------|--------------|
| Contamination awareness | Initial | All personnel |
| Cleanroom protocol | Initial + annual | Access personnel |
| Specific procedures | Per task | Task performers |

### 6.2 Cleanroom Attire

| Item | When Required |
|------|---------------|
| Smock/bunny suit | Always in cleanroom |
| Booties | Always in cleanroom |
| Cap/hood | Always in cleanroom |
| Gloves (nitrile) | Handling hardware |
| Face mask | Close-proximity work |
| Safety glasses | As required |

### 6.3 Gowning Procedure

```
GOWNING SEQUENCE

1. Remove outerwear, jewelry
2. Enter gowning room
3. Step on tacky mat
4. Don booties (one at a time, step over bench)
5. Don smock/bunny suit
6. Don hood/cap
7. Don gloves
8. Enter cleanroom
9. Sign cleanroom log

DEGOWNING (reverse order)
```

### 6.4 Prohibited Items

| Item | Reason |
|------|--------|
| Food/drink | Contamination source |
| Cosmetics | Particulate, molecular |
| Pencils | Graphite particles |
| Paper (untreated) | Fiber shedding |
| Personal electronics | Not cleaned |
| Unapproved materials | Unknown cleanliness |

## 7. Materials and Tools

### 7.1 Approved Materials

| Category | Approved Items |
|----------|----------------|
| Cleaning | IPA (99.9%), lint-free wipes |
| Writing | Cleanroom pens, cleanroom paper |
| Adhesives | 3M Y966, Kapton tape |
| Lubricants | Braycote 601EF (space-qualified) |
| Conformal coating | Per approved list |

### 7.2 Tool Control

| Requirement | Implementation |
|-------------|----------------|
| Cleaning | IPA wipe before entry |
| Inventory | Tool check-in/out log |
| Storage | Cleaned tool cabinet |
| Dedicated | No cross-use with non-flight |

### 7.3 Material Bagging

| Level | Bag Type | Application |
|-------|----------|-------------|
| Single bag | Antistatic poly | General storage |
| Double bag | Cleanroom poly | Critical hardware |
| Nitrogen purge | With desiccant | Long-term storage |

## 8. Hardware Handling

### 8.1 General Handling Rules

| Rule | Rationale |
|------|-----------|
| Gloves always | Prevent skin oils |
| Two-person for flight HW | Prevent drops |
| Lint-free gloves only | Fiber control |
| No direct breathing on HW | Moisture, particles |
| Immediate bagging | Exposure control |

### 8.2 Sensitive Items

| Item | Special Handling |
|------|------------------|
| Solar cells | Handle by edges only |
| Optical sensors | Cover when not in use |
| RF connectors | Protective caps |
| Antenna elements | Gloved handling |
| Jetson module | ESD + contamination |

### 8.3 Work Surface Preparation

| Step | Action |
|------|--------|
| 1 | Clear surface of non-essential items |
| 2 | Wipe with IPA and lint-free wipe |
| 3 | Lay ESD mat if required |
| 4 | Cover with cleanroom paper |
| 5 | Verify lighting adequate |
| 6 | Position tools needed |

## 9. Cleaning Procedures

### 9.1 Standard Cleaning

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Blow with filtered GN2 | Particles removed |
| 2 | Wipe with IPA dampened wipe | One direction |
| 3 | Wipe with dry lint-free wipe | Remove residue |
| 4 | Blow with filtered GN2 | Dry |
| 5 | Visual inspection | No particles visible |

### 9.2 Precision Cleaning

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Ultrasonic clean (IPA) | 5 minutes |
| 2 | Rinse (clean IPA) | 2 minutes |
| 3 | Blow dry (filtered GN2) | No droplets |
| 4 | Bake (if required) | Per procedure |
| 5 | Verification wipe | NVR analysis |

### 9.3 Cleaning Verification

| Method | Application |
|--------|-------------|
| Visual inspection | All surfaces |
| UV light inspection | Molecular films |
| Tape lift test | Particulate count |
| Witness plate | Critical operations |
| NVR analysis | Molecular limits |

## 10. Verification and Inspection

### 10.1 Inspection Points

| Point | Inspection | Record |
|-------|------------|--------|
| Incoming | Visual + bag integrity | Receiving log |
| Pre-integration | Visual + NVR if required | Assembly traveler |
| Post-assembly | Visual + tape lift | Assembly traveler |
| Pre-ship | Full verification | Ship log |

### 10.2 Witness Plates

| When | Location | Analysis |
|------|----------|----------|
| Integration | Near hardware | Tape lift, NVR |
| Environmental test | Inside chamber | Tape lift, NVR |
| Storage (>30 days) | Same bag | Tape lift |

### 10.3 Acceptance Criteria

| Check | Criterion |
|-------|-----------|
| Visual | No visible contamination |
| Tape lift | Per cleanliness level |
| NVR | ≤A/2 (1mg/0.1m²) |
| Particle count | Per IEST-STD-CC1246 |

## 11. Contamination Events

### 11.1 Event Categories

| Category | Definition | Response |
|----------|------------|----------|
| Minor | <Level 500 violation | Clean, log |
| Major | >Level 500 or molecular | Stop work, assess |
| Critical | Flight hardware contact | NCR required |

### 11.2 Response Procedure

```
CONTAMINATION EVENT RESPONSE

1. STOP work immediately
2. CONTAIN the contamination source
3. ASSESS extent of contamination
4. NOTIFY Integration Lead
5. DOCUMENT with photos
6. CLEAN per approved procedure
7. VERIFY cleanliness restored
8. RECORD in event log

If Major/Critical:
9. Initiate NCR
10. Hold hardware pending disposition
```

### 11.3 Common Contamination Events

| Event | Cause | Prevention |
|-------|-------|------------|
| Particle fallout | Poor gowning | Training |
| Tool contamination | Cross-use | Tool control |
| Hardware drop | Single person handling | Two-person rule |
| Bag breach | Sharp edges | Proper bagging |
| IPA residue | Excess solvent | Proper technique |

## 12. Storage Requirements

### 12.1 Hardware Storage

| Duration | Method | Verification |
|----------|--------|--------------|
| <24 hours | Single bag + cap | Visual |
| 1-7 days | Double bag | Bag integrity |
| 7-30 days | Double bag + desiccant | Humidity indicator |
| >30 days | Nitrogen purge + desiccant | Witness plate |

### 12.2 Storage Environment

| Parameter | Requirement |
|-----------|-------------|
| Temperature | 15-30°C |
| Humidity | 20-60% RH |
| Particulate | Class 100,000 or bagged |
| Access | Limited, logged |

## 13. Records and Documentation

### 13.1 Required Records

| Record | Location | Retention |
|--------|----------|-----------|
| Cleanroom log | Integration area | 10 years |
| Training records | QA files | 10 years |
| Inspection results | Assembly traveler | Mission life |
| Contamination events | NCR system | Mission life |
| Witness plate data | Test files | 10 years |

### 13.2 Cleanroom Log Format

```
CLEANROOM ACCESS LOG

Date: ___________
Room Class: ISO ___

Entry:
Time In | Name | Purpose | Time Out
--------|------|---------|----------
        |      |         |
        |      |         |
        |      |         |

Environmental Readings:
Time: _____ Temp: ___°C Humidity: ___%
Particle Count: _____ ≥0.5µm/ft³

Anomalies: ___________________________
```

## 14. Roles and Responsibilities

| Role | Responsibility |
|------|----------------|
| Integration Lead | Overall contamination control |
| Technicians | Follow procedures, report events |
| QA | Verify compliance, audit |
| Facility | Cleanroom maintenance |

## 15. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
