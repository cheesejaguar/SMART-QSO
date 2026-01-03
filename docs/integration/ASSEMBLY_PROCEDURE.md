# SMART-QSO Assembly Procedure

**Document ID**: SQSO-ASM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the assembly procedure for SMART-QSO flight unit integration per NASA-STD-8739 workmanship standards and NASA Ames SmallSat heritage practices.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| NASA-STD-8739.1 | Workmanship Standard for Polymeric Application |
| NASA-STD-8739.3 | Soldering |
| NASA-STD-8739.4 | Crimping and Wire Harness |
| IPC-A-610 | Acceptability of Electronic Assemblies |
| SQSO-HDP-001 | Hardware Development Plan |

## 3. Prerequisites

### 3.1 Personnel Requirements

| Role | Certification | Count |
|------|---------------|-------|
| Assembly Technician | IPC-A-610 | 1 |
| QA Inspector | IPC-A-610 | 1 |
| Integration Lead | - | 1 |

### 3.2 Environment Requirements

| Parameter | Requirement | Verification |
|-----------|-------------|--------------|
| Cleanliness | Class 100,000 | [ ] Verified |
| Temperature | 20-25°C | [ ] Verified |
| Humidity | 30-60% RH | [ ] Verified |
| ESD Control | <100V | [ ] Verified |
| Lighting | Adequate | [ ] Verified |

### 3.3 Equipment and Tools

| Equipment | Cal Due | Status |
|-----------|---------|--------|
| Torque driver set | | [ ] Ready |
| Dental picks | N/A | [ ] Ready |
| Tweezers (ESD) | N/A | [ ] Ready |
| Isopropyl alcohol | N/A | [ ] Ready |
| Lint-free wipes | N/A | [ ] Ready |
| Multimeter | | [ ] Ready |
| Torque wrench | | [ ] Ready |
| Microscope/loupe | N/A | [ ] Ready |

### 3.4 Parts and Materials

| Item | P/N | Qty | Received |
|------|-----|-----|----------|
| Structure assembly | STR-001 | 1 | [ ] |
| EPS board | EPS-001 | 1 | [ ] |
| OBC board | OBC-001 | 1 | [ ] |
| RF board | RF-001 | 1 | [ ] |
| ADCS board | ADCS-001 | 1 | [ ] |
| Jetson assembly | PL-001 | 1 | [ ] |
| Harness set | HAR-001 | 1 | [ ] |
| Antenna assembly | ANT-001 | 1 | [ ] |
| Solar panels | SOL-001 | 6 | [ ] |
| Fastener kit | FAS-001 | 1 | [ ] |
| Thermal interface | TIM-001 | 1 | [ ] |

## 4. Assembly Sequence

### 4.1 Assembly Flow

```
ASSEMBLY FLOW

┌─────────────┐
│  Structure  │
│  Preparation│
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│  Harness    │────►│    EPS      │
│  Install    │     │  Install    │
└─────────────┘     └──────┬──────┘
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    OBC      │     │    RF       │     │   ADCS      │
│  Install    │     │  Install    │     │  Install    │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Jetson    │
                    │  Install    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  Antenna    │
                    │  Install    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Solar     │
                    │  Install    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  Close-out  │
                    │  & Final    │
                    └─────────────┘
```

## 5. Detailed Procedures

### 5.1 Step 1: Structure Preparation

**Duration**: 30 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 1.1 | Inspect structure for damage | No dents, scratches >0.1mm | [ ] |
| 1.2 | Verify rail dimensions | 100×100×113.5mm ±0.1mm | [ ] |
| 1.3 | Clean structure with IPA | No residue visible | [ ] |
| 1.4 | Verify threaded inserts | All inserts present, no cross-threads | [ ] |
| 1.5 | Install standoffs per drawing | Correct locations, torqued | [ ] |
| 1.6 | Document structure S/N | S/N: _______ | [ ] |

**QA Hold Point**: Inspector verifies structure preparation complete

---

### 5.2 Step 2: Harness Installation

**Duration**: 60 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 2.1 | Lay out harness set | All harnesses accounted | [ ] |
| 2.2 | Inspect for damage | No nicks, cuts, or exposed wire | [ ] |
| 2.3 | Route main harness | Per routing diagram | [ ] |
| 2.4 | Secure with tie points | Every 50mm, no chafing | [ ] |
| 2.5 | Coil excess length | Secured, no sharp bends | [ ] |
| 2.6 | Verify connector labels | All labeled correctly | [ ] |

**QA Hold Point**: Inspector verifies harness routing

---

### 5.3 Step 3: EPS Installation

**Duration**: 45 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 3.1 | Verify EPS board tested | Test report available | [ ] |
| 3.2 | Install thermal pad | Full coverage, no bubbles | [ ] |
| 3.3 | Position EPS board | Aligned with standoffs | [ ] |
| 3.4 | Install fasteners | M2×6, torque 0.1 Nm | [ ] |
| 3.5 | Connect power harness | Secure, correct polarity | [ ] |
| 3.6 | Connect telemetry harness | Secure, correct pinout | [ ] |
| 3.7 | Verify no interference | Clearance >1mm | [ ] |
| 3.8 | Document EPS S/N | S/N: _______ | [ ] |

**QA Hold Point**: Inspector verifies EPS installation

---

### 5.4 Step 4: OBC Installation

**Duration**: 45 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 4.1 | Verify OBC board tested | Test report available | [ ] |
| 4.2 | Verify software loaded | Version: _______ | [ ] |
| 4.3 | Install thermal pad | Full coverage | [ ] |
| 4.4 | Position OBC board | Aligned with standoffs | [ ] |
| 4.5 | Install fasteners | M2×6, torque 0.1 Nm | [ ] |
| 4.6 | Connect EPS harness | Power verified | [ ] |
| 4.7 | Connect debug harness | Accessible | [ ] |
| 4.8 | Document OBC S/N | S/N: _______ | [ ] |

**QA Hold Point**: Inspector verifies OBC installation

---

### 5.5 Step 5: RF Installation

**Duration**: 45 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 5.1 | Verify RF board tested | Test report available | [ ] |
| 5.2 | Install thermal pad | Under PA | [ ] |
| 5.3 | Position RF board | Aligned with standoffs | [ ] |
| 5.4 | Install fasteners | M2×6, torque 0.1 Nm | [ ] |
| 5.5 | Connect power harness | Correct polarity | [ ] |
| 5.6 | Connect SPI harness | Secure connection | [ ] |
| 5.7 | Route antenna cables | No sharp bends | [ ] |
| 5.8 | Document RF S/N | S/N: _______ | [ ] |

**Caution**: Handle RF connectors with care - do not over-torque

---

### 5.6 Step 6: ADCS Installation

**Duration**: 30 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 6.1 | Verify ADCS board tested | Test report available | [ ] |
| 6.2 | Note magnetometer orientation | Per drawing | [ ] |
| 6.3 | Position ADCS board | Aligned with standoffs | [ ] |
| 6.4 | Install fasteners | M2×6, torque 0.1 Nm | [ ] |
| 6.5 | Connect I2C harness | Secure connection | [ ] |
| 6.6 | Verify magnetorquer polarity | Per schematic | [ ] |
| 6.7 | Document ADCS S/N | S/N: _______ | [ ] |

---

### 5.7 Step 7: Jetson Installation

**Duration**: 60 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 7.1 | Verify Jetson assembly tested | Test report available | [ ] |
| 7.2 | Verify software loaded | Version: _______ | [ ] |
| 7.3 | Apply thermal interface | Per specification | [ ] |
| 7.4 | Position Jetson assembly | Thermal path verified | [ ] |
| 7.5 | Install fasteners | M3×8, torque 0.15 Nm | [ ] |
| 7.6 | Connect power harness | Enable line verified | [ ] |
| 7.7 | Connect UART harness | Secure connection | [ ] |
| 7.8 | Verify thermal contact | Gap check | [ ] |
| 7.9 | Document Jetson S/N | S/N: _______ | [ ] |

**QA Hold Point**: Inspector verifies thermal interface

---

### 5.8 Step 8: Antenna Installation

**Duration**: 30 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 8.1 | Verify antenna tested | Pattern verified | [ ] |
| 8.2 | Stow antenna elements | Per stowage procedure | [ ] |
| 8.3 | Install deployment mechanism | Aligned with mount | [ ] |
| 8.4 | Connect RF cables | Torque SMA 0.5 Nm | [ ] |
| 8.5 | Connect deployment harness | Burn wire verified | [ ] |
| 8.6 | Verify stowage secure | Constraint verified | [ ] |
| 8.7 | Install RBF pin | Deployment inhibited | [ ] |

**Warning**: Antenna may deploy unexpectedly - wear safety glasses

---

### 5.9 Step 9: Solar Panel Installation

**Duration**: 60 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 9.1 | Verify solar cells tested | I-V data available | [ ] |
| 9.2 | Clean mounting surfaces | IPA, lint-free | [ ] |
| 9.3 | Install +X panel | Aligned, fastened | [ ] |
| 9.4 | Install -X panel | Aligned, fastened | [ ] |
| 9.5 | Install +Y panel | Aligned, fastened | [ ] |
| 9.6 | Install -Y panel | Aligned, fastened | [ ] |
| 9.7 | Install +Z panel | Aligned, fastened | [ ] |
| 9.8 | Install -Z panel (if applicable) | Aligned, fastened | [ ] |
| 9.9 | Connect solar harness | Verify polarity | [ ] |
| 9.10 | Verify no shadows | Clearance check | [ ] |

**Caution**: Solar cells are fragile - handle by edges only

---

### 5.10 Step 10: Close-out and Final Assembly

**Duration**: 60 minutes

| Step | Action | Criteria | Inspector |
|------|--------|----------|-----------|
| 10.1 | Final harness routing check | No pinch points | [ ] |
| 10.2 | Final torque verification | All fasteners | [ ] |
| 10.3 | Foreign Object Debris (FOD) check | None present | [ ] |
| 10.4 | Install access panels | Aligned, fastened | [ ] |
| 10.5 | Install RBF pin | In place | [ ] |
| 10.6 | Install deployment switches | Actuated | [ ] |
| 10.7 | Verify rail cleanliness | Per CDS | [ ] |
| 10.8 | Final visual inspection | Workmanship check | [ ] |
| 10.9 | Document as-built config | Complete | [ ] |

**QA Hold Point**: Final assembly inspection

---

## 6. Post-Assembly

### 6.1 Mass Properties Measurement

| Property | Requirement | Measured | Pass |
|----------|-------------|----------|------|
| Total mass | ≤2.0 kg | | [ ] |
| CG-X | ±20 mm | | [ ] |
| CG-Y | ±20 mm | | [ ] |
| CG-Z | ±20 mm | | [ ] |

### 6.2 Aliveness Test

| Check | Expected | Result | Pass |
|-------|----------|--------|------|
| Power on | Boot to SAFE | | [ ] |
| Console access | Messages visible | | [ ] |
| EPS telemetry | Valid values | | [ ] |
| RF test (dummy load) | Output verified | | [ ] |
| Deployment inhibit | TX disabled | | [ ] |

### 6.3 Documentation

| Document | Completed | Location |
|----------|-----------|----------|
| As-built list | [ ] | |
| Assembly photos | [ ] | |
| Traveler signed | [ ] | |
| NCRs (if any) | [ ] | |

## 7. Assembly Traveler

```
SMART-QSO ASSEMBLY TRAVELER

Unit S/N: _______________
Assembly Start Date: _______________
Assembly End Date: _______________

STEP COMPLETION:

Step | Description        | Tech  | Date  | Insp | Date
─────┼────────────────────┼───────┼───────┼──────┼──────
1    | Structure Prep     |       |       |      |
2    | Harness Install    |       |       |      |
3    | EPS Install        |       |       |      |
4    | OBC Install        |       |       |      |
5    | RF Install         |       |       |      |
6    | ADCS Install       |       |       |      |
7    | Jetson Install     |       |       |      |
8    | Antenna Install    |       |       |      |
9    | Solar Install      |       |       |      |
10   | Close-out          |       |       |      |

FINAL APPROVAL:

Integration Lead: _____________ Date: _______
QA Inspector: _____________ Date: _______
Project Manager: _____________ Date: _______
```

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
