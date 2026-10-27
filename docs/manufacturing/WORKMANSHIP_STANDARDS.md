# SMART-QSO Workmanship Standards

**Document ID**: SQSO-WKS-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft (no hardware built). The workmanship plan, SQSO-WP-001 (`docs/manufacturing/WORKMANSHIP_PLAN.md`), carries
the same standards with the inspection, non-conformance and records process; where the two differ on a value, the
design documents cited here govern

## 1. Purpose

This document establishes the workmanship standards for SMART-QSO per NASA-STD-8739.6B and the standards it invokes
(§2). Version 1.0 cited NASA-STD-8739.1 as "Workmanship for Ground Support Equipment" (it is the polymeric-application
standard) and NASA-STD-8739.3 for soldering (cancelled); both are corrected.

## 2. Applicable Standards

### 2.1 Primary Standards

| Standard | Title | Application |
|----------|-------|-------------|
| NASA-STD-8739.6B (effective 2021-02-04) | Implementation Requirements for NASA Workmanship Standards | Invokes the IPC standards below with their space addenda, training and ESD control |
| IPC J-STD-001 (current revision) with the Space and Military Applications Electronic Hardware Addendum | Requirements for Soldered Electrical and Electronic Assemblies | All flight soldering (replaces NASA-STD-8739.3, cancelled) |
| IPC-A-610, Class 3 | Acceptability of Electronic Assemblies | Inspection criteria |
| NASA-STD-8739.4A (2016), with IPC/WHMA-A-620 and its space addendum | Crimping, Interconnecting Cables, Harnesses, and Wiring | Harness assembly |
| NASA-STD-8739.1B | Workmanship Standard for Polymeric Application on Electronic Assemblies | Conformal coating, staking, bonding |
| ANSI/ESD S20.20 (current revision), with the NASA-STD-8739.6B additions | ESD Control Program | ESD protection |
| NASA-STD-5020B practice | Threaded fastening systems | Preload and locking features |

### 2.2 Classification Level

| Item | Class | Rationale |
|------|-------|-----------|
| Flight PCBs | Class 3 | Space hardware |
| Harness | Class 3 | Space hardware |
| Structure | Class 2 | Mechanical assembly |
| GSE | Class 2 | Ground equipment |

## 3. Soldering Requirements

### 3.1 General Soldering

| Parameter | Requirement |
|-----------|-------------|
| Solder alloy | Sn63/Pb37 only on flight boards (tin-whisker control; SAC305 not used: `docs/compliance/MATERIALS_LIST.md` Rev B §2.2) |
| Flux | Type RMA or ROL0 |
| Tip temperature | 315-370°C (600-700°F) |
| Contact time | 2-5 seconds typical |
| Iron power | 25-60W, temperature controlled |

### 3.2 Surface Mount Soldering

| Criterion | Requirement |
|-----------|-------------|
| Solder fillet | Wetting to pad and lead |
| Heel fillet | Visible, wetted |
| Side fillet | >25% of lead height |
| Solder amount | No bridging, no insufficient |
| Alignment | Within pad boundaries |

### 3.3 Through-Hole Soldering

| Criterion | Requirement |
|-----------|-------------|
| Hole fill | >75% vertical |
| Fillet | 360° wetting |
| Lead protrusion | 0.5-2.0mm |
| Solder surface | Smooth, shiny (Sn/Pb) |

### 3.4 Solder Joint Defects

```
SOLDER DEFECT CLASSIFICATION

DEFECT           CLASS 3 ACCEPT    REWORK REQUIRED
───────────────────────────────────────────────────
Cold joint          No             Yes
Disturbed joint     No             Yes
Insufficient        No             Yes
Excessive           No             Yes
Bridging            No             Yes
Solder balls        No             Yes
Dewetting           No             Yes
Nonwetting          No             Yes
Voids (>25%)        No             Yes
Cracks              No             Yes
```

## 4. PCB Assembly

### 4.1 Component Placement

| Parameter | Requirement |
|-----------|-------------|
| Orientation | Per assembly drawing |
| Polarity | Verified before solder |
| Alignment | Within pad, centered |
| Standoff | Per component spec |

### 4.2 Cleaning

| Stage | Method |
|-------|--------|
| Pre-assembly | IPA wipe |
| Post-solder | Aqueous wash or IPA |
| Final clean | Verified by inspection |
| Cleanliness | Per J-STD-001 |

### 4.3 Conformal Coating

| Parameter | Requirement |
|-----------|-------------|
| Material | Arathane 5750 A/B (LV) polyurethane, preferred; HumiSeal 1B73 (acrylic) and Dow 1-2577 alternates, E595 data open (`docs/compliance/MATERIALS_LIST.md` §5) |
| Thickness | 25-75 μm |
| Coverage | All exposed surfaces |
| Masking | Connectors, test points |
| Cure | 24 hours at 25°C |

### 4.4 Staking

The thresholds are the same as WORKMANSHIP_PLAN §5.1 (the stricter of the values the two documents carried before
version 1.3); §10.2 likewise matches the plan's §13.

| Component | Staking |
|-----------|---------|
| Heavy components (> 2 g) | Required |
| Tall components (> 6 mm) | Required |
| Connectors | Required |
| Crystals | Required |

## 5. Harness Assembly

### 5.1 Wire Preparation

| Step | Requirement |
|------|-------------|
| Wire cutting | Clean cut, no fraying |
| Strip length | Per terminal spec |
| Strip quality | No nicks, no strands cut |
| Twisting | Tight twist, no loose strands |
| Tinning | Full coverage, no excess |

### 5.2 Crimping

| Parameter | Requirement |
|-----------|-------------|
| Crimp tool | Calibrated, proper die |
| Wire gauge | Match terminal size |
| Insulation grip | Visible, not in crimp |
| Pull test | Per wire gauge table |
| Inspection | 100% visual |

### 5.3 Connector Assembly

| Step | Requirement |
|------|-------------|
| Pin insertion | Full seat, click felt |
| Retention | Pull test passed |
| Orientation | Per drawing |
| Backshell | Torqued, strain relief |

### 5.4 Harness Routing

| Parameter | Requirement |
|-----------|-------------|
| Bend radius | >3× wire diameter |
| Strain relief | At all connectors |
| Tie-down | Every 50mm min |
| Chafe protection | At all edges |
| Separation | Power/signal separated |

## 6. Mechanical Assembly

### 6.1 Fastener Installation

| Parameter | Requirement |
|-----------|-------------|
| Hole alignment | No forced fits |
| Thread engagement | >2× diameter |
| Torque | Per fastener table |
| Locking | Per §6.3: locking insert or prevailing-torque locknut plus staking; no thread-locker alone |
| Verification | Torque stripe |

### 6.2 Torque Values

Flight torques are set joint by joint in `hardware/structure/STRUCTURAL_DESIGN.md` §8.1, which governs:

| Fastener | Torque (N·m) | Joint |
|----------|--------------|-------|
| M2 A286 | 0.15 | PCB mounting and centre supports, hinges, access panels |
| M2 Ti-6Al-4V | 0.10–0.15 | Cell-retention locator plates 0.10; Z-rod clips and inertia ballast 0.15 |
| M2.5 A286 | 0.35 | Structure assembly, heat spreader |
| M2.5 Ti-6Al-4V | 0.20 ± 10 % | Battery isolators (polymer-bushed) |
| M3 A286 | 0.60 | PCB stack studs |

The Version 1.1 generic values (M2 0.17, M2.5 0.28, M3 0.56, M4 1.35 N·m) are withdrawn.

### 6.3 Thread Locking

| Joint Type | Method |
|------------|--------|
| Structural | Locking helicoil or prevailing-torque insert plus staking with Scotch-Weld 2216 (E595-compliant) |
| Adjustable | Prevailing-torque (all-metal) locknut |
| Polymer-bushed joints (battery mount, Ti-6Al-4V screws) | Low preload, staking, locking insert; re-torque check after thermal cycling; **no thread-locker alone** |
| Thread-lockers | Only grades with ASTM E595 data in the NASA database or a vendor certificate. Loctite 222, 242 and 262 are not on Henkel's NASA outgassing-approved list and are not used until such data exist |

### 6.4 Torque Sequence

```
TORQUE SEQUENCE (Multi-bolt patterns)

1. Hand-start all fasteners
2. Snug in star pattern to 50%
3. Torque in star pattern to 75%
4. Final torque in star pattern to 100%
5. Apply torque stripe
6. Record torque values

   ┌─────────────────┐
   │  2       4      │
   │     (star)      │
   │  1       3      │
   └─────────────────┘
```

## 7. Cleanliness

### 7.1 Particulate Cleanliness

| Level | Maximum Particle | Application |
|-------|------------------|-------------|
| 300 | 300 μm | Optical |
| 500 | 500 μm | General flight |
| 750 | 750 μm | Structure |

### 7.2 Molecular Cleanliness

| Level | Maximum NVR | Application |
|-------|-------------|-------------|
| A | 1 mg/0.1m² | Optical |
| B | 2 mg/0.1m² | General flight |
| C | 3 mg/0.1m² | Structure |

### 7.3 Cleaning Methods

| Contamination | Method |
|---------------|--------|
| Particulate | N2 blow, vacuum |
| Fingerprints | IPA wipe |
| Flux residue | Aqueous wash |
| Oils/grease | IPA or approved solvent |

## 8. ESD Control

### 8.1 ESD Sensitive Devices

| Sensitivity | Voltage | Examples |
|-------------|---------|----------|
| Class 0 | <250V | MOSFET, some ICs |
| Class 1 | 250-500V | Most ICs |
| Class 2 | 500-2000V | Discretes |
| Class 3 | >2000V | Passives |

### 8.2 ESD Precautions

| Measure | Requirement |
|---------|-------------|
| Wrist strap | <10 MΩ to ground |
| Work surface | ESD dissipative |
| Flooring | ESD or heel straps |
| Ionizers | For non-conductive |
| Packaging | Pink poly or shielding |
| Humidity | >30% RH preferred |

### 8.3 ESD Handling

```
ESD HANDLING REQUIREMENTS

Before touching ESDS:
[ ] Wrist strap connected
[ ] Work surface verified
[ ] Parts in ESD packaging until needed

During handling:
[ ] Handle by edges
[ ] Avoid touching pins/leads
[ ] Ground before first touch
[ ] Keep in ESD container when not working

Storage:
[ ] ESD bags/containers
[ ] Labeled "ESD Sensitive"
[ ] Humidity controlled
```

## 9. Inspection Requirements

### 9.1 Inspection Levels

| Level | Description | Application |
|-------|-------------|-------------|
| 100% visual | Every feature, every unit | Flight |
| Sample | Representative sample | Non-critical |
| In-process | During assembly | All |
| Final | Complete assembly | Flight |

### 9.2 Inspection Equipment

| Equipment | Use | Calibration |
|-----------|-----|-------------|
| Stereo microscope | Solder, components | Annual |
| Magnifying lamp | General visual | N/A |
| Multimeter | Electrical | Annual |
| Calipers | Dimensional | Annual |
| Torque wrench | Fasteners | Annual |

### 9.3 Accept/Reject Criteria

| Criterion | Accept | Reject |
|-----------|--------|--------|
| Solder joints | Per IPC J-STD-001 space addendum and IPC-A-610 Class 3 | Defects per std |
| Components | Correct, undamaged | Wrong/damaged |
| Markings | Legible, correct | Missing/wrong |
| Cleanliness | Per level spec | Contamination |
| Mechanical | Per drawing | Out of tolerance |

## 10. Documentation

### 10.1 Traveler Requirements

| Element | Required |
|---------|----------|
| Part number | Yes |
| Serial number | Yes |
| Operator signature | Yes |
| Inspector signature | Yes |
| Date/time | Yes |
| Step sequence | Yes |
| Deviations noted | Yes |

### 10.2 Inspection Records

| Record | Retention |
|--------|-----------|
| Travelers | Permanent |
| Inspection reports | Permanent |
| Photographs | Permanent |
| Nonconformance | Permanent |
| Calibration | Per the calibration programme, at least 3 years |
| Training | Duration of employment, at least 3 years |

### 10.3 Nonconformance

```
NONCONFORMANCE PROCESS

1. Identify discrepancy
2. Tag item "HOLD"
3. Document on NCR form
4. Engineering disposition:
   - Use as-is
   - Rework
   - Repair
   - Scrap
5. Implement disposition
6. Verify correction
7. Close NCR
8. Archive documentation
```

## 11. Operator Qualification

### 11.1 Required Certifications

| Task | Certification |
|------|---------------|
| Soldering | J-STD-001 or equiv |
| Crimping | NASA-STD-8739.4 |
| Conformal coating | IPC training |
| Inspection | IPC-A-610 |

### 11.2 Recertification

| Certification | Period |
|---------------|--------|
| Soldering | 2 years |
| Crimping | 2 years |
| Inspection | 2 years |
| ESD | Annual |

## 12. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-27 | CDR S&MA review | §2 standards corrected (NASA-STD-8739.6B, J-STD-001 space addendum, 8739.4A, 8739.1B; 8739.3 cancelled; 8739.1 title); Sn63Pb37 only; thread-locking rules without non-compliant Loctite grades |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026) | Status "CDR Baseline" replaced by Draft, with the relation to the workmanship plan; §4.3 conformal coat Arathane 5750 per MATERIALS_LIST §5 (HumiSeal 1B73 an alternate); §6.1 locking aligned with §6.3; §6.2 torques from STRUCTURAL_DESIGN §8.1 |
| 1.3 | 2026-10-27 | Documentation audit (post-CDR 2026), follow-up | §4.4 staking thresholds and §10.2 record retention made identical to SQSO-WP-001 D §5.1 and §13 (the stricter value of each: staking above 2 g or 6 mm; inspection, photograph, nonconformance and traveler records permanent) |
