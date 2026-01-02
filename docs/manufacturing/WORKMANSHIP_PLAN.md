# Workmanship Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-WP-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA-STD-8739 Family

---

## 1. Purpose

This Workmanship Plan defines the quality standards and inspection criteria for SMART-QSO CubeSat manufacturing and assembly.

## 2. Applicable Standards

| Standard | Title | Application |
|----------|-------|-------------|
| NASA-STD-8739.1 | Workmanship for Staking and Conformal Coating | PCB protection |
| NASA-STD-8739.3 | Soldered Electrical Connections | Solder joints |
| NASA-STD-8739.4 | Crimping, Interconnecting Cables | Harness assembly |
| IPC J-STD-001 Space | Soldering Requirements | Alternate soldering std |
| IPC-A-610 Class 3 | Acceptability of Electronic Assemblies | Inspection criteria |
| ANSI/ESD S20.20 | ESD Control Program | ESD protection |

## 3. Soldering Requirements

### 3.1 General Requirements

| Parameter | Requirement |
|-----------|-------------|
| Standard | NASA-STD-8739.3 or IPC J-STD-001 Space |
| Workmanship Class | Class 3 (High Reliability) |
| Personnel | Certified to applicable standard |
| Equipment | Calibrated, ESD-safe |

### 3.2 Solder Materials

| Material | Specification |
|----------|---------------|
| Solder alloy | Sn63Pb37 or SAC305 (lead-free) |
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
| Standard | NASA-STD-8739.1 |
| Material | Per materials list (Dow 1-2577 or equiv) |
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

| Component | Method |
|-----------|--------|
| Heavy components (>3g) | Adhesive staking |
| Tall components (>6mm) | Adhesive staking |
| Connectors | Adhesive or mechanical |
| Wire bundles | Adhesive at stress points |
| Crystals | Adhesive staking |

### 5.2 Staking Materials

| Material | Application |
|----------|-------------|
| Dow Corning 93-500 | General staking |
| Hysol EA 9394 | High-strength staking |

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
| Tie-down | Nylon ties or lacing cord |

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
| Thread locker | Loctite 242 (unless thermal cycling) |
| Verification | Calibrated torque driver |
| Documentation | Torque values recorded |

### 7.2 Torque Values

| Fastener | Torque (in-lb) |
|----------|----------------|
| M2 | 1.5-2.0 |
| M2.5 | 2.5-3.5 |
| M3 | 4.0-5.5 |
| #2-56 | 2.0-2.5 |
| #4-40 | 4.5-5.5 |

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
| Soldering operators | IPC J-STD-001 or NASA-STD-8739.3 |
| Inspectors | IPC-A-610 CIS |
| Wire harness | NASA-STD-8739.4 |
| ESD handlers | ESD awareness |

## 13. Records

| Record | Retention |
|--------|-----------|
| Traveler | Permanent |
| Inspection records | Permanent |
| Calibration records | Per cal program |
| Training records | Duration of employment |
| NCRs | Permanent |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
