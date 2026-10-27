# Workmanship Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-WP-001
**Revision:** D
**Date:** 2026-10-27
**Reference:** NASA-STD-8739.6B and the standards it invokes (§2)

---

## 1. Purpose

This Workmanship Plan defines the quality standards and inspection criteria for SMART-QSO CubeSat manufacturing and assembly.

## 2. Applicable Standards

| Standard | Title | Application |
|----------|-------|-------------|
| NASA-STD-8739.6B (effective 2021-02-04) | Implementation Requirements for NASA Workmanship Standards | Invokes the IPC standards below with their space addenda, training and ESD control |
| IPC J-STD-001 (current revision) with the Space and Military Applications Electronic Hardware Addendum | Requirements for Soldered Electrical and Electronic Assemblies | All flight soldering (replaces NASA-STD-8739.3, cancelled) |
| IPC-A-610, Class 3 | Acceptability of Electronic Assemblies | Inspection criteria |
| NASA-STD-8739.4A (2016), with IPC/WHMA-A-620 and its space addendum | Crimping, Interconnecting Cables, Harnesses, and Wiring | Harness assembly |
| NASA-STD-8739.1B | Workmanship Standard for Polymeric Application on Electronic Assemblies | Conformal coating, staking, bonding |
| ANSI/ESD S20.20 (current revision), with the NASA-STD-8739.6B additions | ESD Control Program | ESD protection |
| NASA-STD-5020B practice | Threaded fastening systems | Preload and locking features |

## 3. Soldering Requirements

### 3.1 General Requirements

| Parameter | Requirement |
|-----------|-------------|
| Standard | IPC J-STD-001 with the space addendum (NASA-STD-8739.6B) |
| Workmanship Class | Class 3 (High Reliability) |
| Personnel | Certified to applicable standard |
| Equipment | Calibrated, ESD-safe |

### 3.2 Solder Materials

| Material | Specification |
|----------|---------------|
| Solder alloy | Sn63Pb37 only on flight boards (no SAC305: tin-whisker control) |
| Flux | RMA or ROL0 per J-STD-004 |
| Solder paste | Type 3 or 4 per J-STD-005 |
| Cleaning | Per IPC-A-610 |

### 3.3 Solder Joint Criteria

| Joint Type | Acceptance Criteria |
|------------|---------------------|
| Through-hole | Wetting 270°, fillet formation |
| SMT chip | Wetting to termination, no tombstoning |
| SMT gull wing | Heel fillet, toe fillet visible |
| SMT J-lead | Side fillet visible |
| BGA | X-ray inspection, no voids > 25% |

### 3.4 Defects

| Defect | Classification |
|--------|---------------|
| Cold solder joint | Reject |
| Insufficient wetting | Reject |
| Solder bridge | Reject |
| Solder ball > 0.13mm | Reject |
| Disturbed joint | Reject |
| Lifted pad | Reject |

## 4. Conformal Coating

### 4.1 Requirements

| Parameter | Requirement |
|-----------|-------------|
| Standard | NASA-STD-8739.1B |
| Material | Arathane 5750 A/B (LV) polyurethane, preferred; HumiSeal 1B73 and Dow 1-2577 alternates, E595 data open (`docs/compliance/MATERIALS_LIST.md` §5) |
| Thickness | 25-75 µm (1-3 mils) |
| Coverage | All exposed conductors |

### 4.2 Keep-Out Areas

| Area | Reason |
|------|--------|
| Connectors | Mating surfaces |
| Test points | Accessibility |
| Adjustment pots | Adjustability |
| Thermal interfaces | Heat transfer |
| Mounting holes | Hardware installation |

### 4.3 Inspection

| Criterion | Method |
|-----------|--------|
| Coverage | Visual, UV inspection |
| Thickness | Eddy current or calibrated gauge |
| Adhesion | Tape test (per ASTM D3359) |
| Bubbles/voids | Visual, < 3mm diameter |

## 5. Staking Requirements

### 5.1 Components Requiring Staking

The same thresholds as WORKMANSHIP_STANDARDS (SQSO-WKS-001) §4.4; the records of §13 match its §10.2.

| Component | Method |
|-----------|--------|
| Heavy components (> 2 g) | Adhesive staking |
| Tall components (> 6 mm) | Adhesive staking |
| Connectors | Adhesive or mechanical |
| Wire bundles | Adhesive at stress points |
| Crystals | Adhesive staking |

### 5.2 Staking Materials

| Material | Application |
|----------|-------------|
| 3M Scotch-Weld 2216 B/A Gray | Component and wire staking, harness tie-down dots; staking only, never structural (`docs/compliance/MATERIALS_LIST.md` §5) |
| Arathane 5750 A/B | Staking of mated wire-to-board connectors (removable bead; `hardware/harness/HARNESS_DESIGN.md`) |

### 5.3 Staking Criteria

- Fillet height: 1/3 to 2/3 component height
- Coverage: 2 or more sides for rectangular components
- No bridging to adjacent components
- Fully cured before handling

## 6. Wire and Cable Assembly

### 6.1 Requirements

| Parameter | Requirement |
|-----------|-------------|
| Standard | NASA-STD-8739.4 |
| Wire type | Per materials list (Teflon insulated) |
| Routing | No sharp bends, service loops |
| Tie-down | MIL-T-43435 lacing tape (no nylon ties: `docs/compliance/MATERIALS_LIST.md` §7) |

### 6.2 Crimping

| Parameter | Requirement |
|-----------|-------------|
| Tool | Calibrated crimp tool for terminal |
| Inspection | Visual + pull test (sample) |
| Wire strip length | Per terminal specification |
| Insulation gap | 0.5-1.0 mm |

### 6.3 Connector Assembly

| Parameter | Requirement |
|-----------|-------------|
| Pin insertion | Full seat, locking feature engaged |
| Strain relief | Required on all connectors |
| Labeling | All connectors labeled |
| Keying | Verified correct mating |

## 7. Mechanical Assembly

### 7.1 Fastener Installation

| Parameter | Requirement |
|-----------|-------------|
| Torque | Per fastener specification |
| Thread locker | None without E595 data; locking inserts and staking (WORKMANSHIP_STANDARDS §6.3) |
| Verification | Calibrated torque driver |
| Documentation | Torque values recorded |

### 7.2 Torque Values

The installation torque of every flight joint is set by the structural design, joint by joint, and governs: M2 A286
0.15 N·m, M2.5 A286 0.35 N·m, M3 A286 stack studs 0.60 N·m, the M2.5 Ti-6Al-4V battery isolators 0.20 N·m ± 10 %, the
M2 Ti-6Al-4V cell-retention screws 0.10 N·m (`hardware/structure/STRUCTURAL_DESIGN.md` §8.1). The Revision A generic table (in-lb per thread size) is withdrawn.

### 7.3 Inspection

| Criterion | Method |
|-----------|--------|
| Torque | Verify with calibrated tool |
| Thread engagement | Visual (minimum 2 threads) |
| Witness mark | Applied after torque |
| Lock wire | If required |

## 8. ESD Control

### 8.1 Requirements

| Parameter | Requirement |
|-----------|-------------|
| Standard | ANSI/ESD S20.20 |
| Work surface | < 1 × 10⁹ Ω |
| Personnel | Wrist strap or heel straps |
| Packaging | ESD-safe bags and containers |

### 8.2 Handling

| Practice | Requirement |
|----------|-------------|
| Grounding | Always before handling ESD-sensitive items |
| Packaging | Use ESD bags for transport/storage |
| Work area | Designated ESD-protected area |
| Training | All personnel ESD trained |

## 9. Cleanliness

### 9.1 Requirements

| Parameter | Requirement |
|-----------|-------------|
| PCB cleanliness | Per IPC-A-610 |
| Ionic contamination | < 1.56 µg/cm² NaCl equiv |
| Particle contamination | Visibly clean |
| Final clean | Prior to conformal coating |

### 9.2 Cleaning Methods

| Contaminant | Method |
|-------------|--------|
| Flux residue | Saponified water or IPA |
| Fingerprints | IPA wipe |
| Particles | Compressed air (filtered) |
| Grease/oil | IPA or approved solvent |

## 10. Inspection Requirements

### 10.1 In-Process Inspection

| Point | Inspection |
|-------|------------|
| SMT placement | Visual (sample) |
| After reflow | Visual (100%) |
| After hand solder | Visual (100%) |
| Before coating | Cleanliness (sample) |
| After coating | Visual (100%) |
| After assembly | Visual + dimensional |

### 10.2 Inspection Documentation

| Document | Purpose |
|----------|---------|
| Traveler | Track assembly progress |
| Inspection record | Document results |
| NCR | Non-conformance report |
| MRB | Material review board disposition |

## 11. Non-Conformance Handling

### 11.1 Classification

| Class | Definition | Disposition |
|-------|------------|-------------|
| Critical | Safety/mission impact | MRB required |
| Major | Performance impact | MRB or engineer disposition |
| Minor | Cosmetic | Inspector disposition |

### 11.2 Disposition Options

| Option | Description |
|--------|-------------|
| Use as-is | Accept with documentation |
| Rework | Repair to specification |
| Repair | Alternative fix (MRB approval) |
| Scrap | Non-repairable |

## 12. Training Requirements

| Role | Certification |
|------|---------------|
| Soldering operators | IPC J-STD-001 space addendum certified (NASA-STD-8739.6B) |
| Inspectors | IPC-A-610 CIS |
| Wire harness | NASA-STD-8739.4 |
| ESD handlers | ESD awareness |

## 13. Records

| Record | Retention |
|--------|-----------|
| Traveler | Permanent |
| Inspection records | Permanent |
| Photographs | Permanent |
| Calibration records | Per the calibration programme, at least 3 years |
| Training records | Duration of employment, at least 3 years |
| NCRs | Permanent |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | §2 standards corrected (8739.6B, J-STD-001 space addendum, 8739.4A, 8739.1B; 8739.3 cancelled); Sn63Pb37 only; lacing instead of nylon ties; Loctite 242 removed | CDR S&MA review |
| C | 2026-10-27 | Documentation audit (post-CDR 2026): §4.1 conformal coat Arathane 5750 (preferred) with the open alternates, §5.2 staking materials Scotch-Weld 2216 and Arathane 5750 (Dow 93-500 is the panel encapsulant, Hysol EA 9394 is not in the materials list), §6.1 MIL-T-43435 lacing tape, all per MATERIALS_LIST §5 and §7; §7.2 torques from STRUCTURAL_DESIGN §8.1 in place of the generic table | Documentation audit |
| D | 2026-10-27 | Documentation audit (post-CDR 2026), follow-up: §5.1 staking thresholds and §13 record retention made identical to SQSO-WKS-001 1.3 (the stricter value of each: heavy above 2 g, was 3 g; photographs permanent; calibration and training records at least 3 years) | Documentation audit |
