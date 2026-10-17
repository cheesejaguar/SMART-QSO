# SMART-QSO Mass Budget

**Document ID**: SQSO-MB-001
**Version**: 2.0
**Date**: 2026-10-16
**Classification**: Unclassified
**Status**: Bottom-up master equipment list (MEL) for the baseline in [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) §4–§8. **Analysis and estimates; nothing has been weighed** (no hardware exists).

## 1. Purpose

This document is the controlled mass budget for the SMART-QSO 1U CubeSat. It lists every item of the baseline
configuration with its current best estimate (CBE), the basis of that estimate, a mass growth allowance (MGA) set by the
item's design maturity, and the resulting predicted mass, and compares the total with one mass limit (finding TB-06).

### 1.1 Correction to Version 1.0

Version 1.0 totalled 1,200 g CBE and omitted the magnetorquers, the solar wings and their hinges, the body solar-panel
substrates and the Jetson heat spreader. It applied an 11 % average contingency, carried "CG Verified" and moments of
inertia with no model behind them, and reported a growth history that no record supports. The repository also quoted
eight different spacecraft masses (0.77 to 1.56 kg, including a "measured" 0.77 kg with no hardware) and two different
"CDS 14.1" limits; 1.33 kg is the CDS Rev 13 1U limit, not Rev 14.1. Version 2.0 replaces all of these.

## 2. Mass Requirements

### 2.1 Mass Limit

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Maximum mass | **2.00 kg**, or the integrator's dispenser limit if lower | CDS Rev 14.1 Table 1; BASELINE §4; SYS-002 (≤ 2.0 kg) |
| Maximum mass, ISS/NRCSD backup | 2.40 kg | BASELINE §4 |
| CG location | Within 2 cm of the geometric center | SYS-003; CDS Rev 14.1 |
| Mass properties | Measured on the flight unit and documented | §8 |

### 2.2 Growth Allowance and Margin Method

Method: ANSI/AIAA S-120A-2015, *Mass Properties Control for Space Systems* (maturity categories and the definitions
below). The growth percentages are **this document's choices**. They are set at the upper end of the cumulative growth
bands for each S-120 maturity category as presented in NASA MSFC, *Mass Properties for Space Systems Standards
Development*, SAWE 2013 (NTRS 20130014265), slide 4; the standard's own table could not be obtained for this issue.

| Maturity code | Meaning | MGA used | Lower-end value (sensitivity) |
|---------------|---------|----------|-------------------------------|
| E-1 | Estimated: no drawing; value from a sketch, an earlier estimate or a similar item | 25 % | 18 % |
| E-2 | Layout: calculated from assumed dimensions and materials | 15 % | 12 % |
| A-5 | Existing hardware: off-the-shelf item with a published typical mass | 3 % | 2 % |
| Spec max | Published not-to-exceed (maximum) mass; the maximum already covers the variation | 0 % | 0 % |

Definitions (S-120): MGA % = (predicted − CBE)/CBE × 100; mass margin % = (allowable − predicted)/predicted × 100. The
S-120 guidance reproduced on slide 5 of the same briefing grades a mass margin above 9 % green at PDR (5–9 % yellow, ≤ 5 %
red) and above 5 % green at CDR. The project is in Phase B (pre-PDR; BASELINE §1).

## 3. Mass Budget Summary

### 3.1 Master Equipment List

| # | Item | Qty | CBE (g) | Basis of CBE | Maturity | MGA | Predicted (g) |
|---|------|-----|---------|--------------|----------|-----|---------------|
| 1 | Primary structure: frame, rails, end plates, deployment switches | 1 | 280.0 | Version 1.0 §4.1 (frame 180, rails 60, end panels 20, switches 20); ODAR §2 | E-1 | 25 % | 350.0 |
| 2 | Internal brackets | 1 set | 20.0 | `hardware/structure/STRUCTURAL_DESIGN.md` §2.1 (not in Version 1.0) | E-1 | 25 % | 25.0 |
| 3 | Solar cells, AZUR SPACE 3G30A (cell-interconnect-coverglass) | 16 | 57.6 | ≤ 3.6 g each, AZUR SPACE datasheet DB 00010891-01 (2025) | Spec max | 0 % | 57.6 |
| 4 | Solar-panel substrates (4 body, 2 wing) | 6 | 148.2 | Calculated, §4.2 | E-2 | 15 % | 170.4 |
| 5 | Wing hinges, springs, hold-down and burn-wire release | 2 | 32.0 | ODAR §2 wing allowance (110 g) minus two substrates (49.4 g) and eight cells (28.8 g) | E-1 | 25 % | 40.0 |
| 6 | EPS electronics (MPPT, battery management, distribution) | 1 | 90.0 | Version 1.0 §4.2 (board 60, connectors 20, protection 10) | E-1 | 25 % | 112.5 |
| 7 | Battery cells, Samsung INR18650-35E | 4 | 200.0 | ≤ 50.0 g each, Samsung SDI specification | Spec max | 0 % | 200.0 |
| 8 | Battery cell-interconnect board | 1 | 15.0 | Version 1.0 §4.3 | E-1 | 25 % | 18.8 |
| 9 | Battery holder (two aluminum clamp plates) | 1 | 34.0 | Calculated, §4.3 | E-2 | 15 % | 39.1 |
| 10 | On-board computer (STM32L4R5ZI) | 1 | 45.0 | Version 1.0 §4.4 | E-1 | 25 % | 56.2 |
| 11 | RF: transponder receiver and transmitter, PA, LNA, filters, switches | 1 | 120.0 | Version 1.0 §4.5 | E-1 | 25 % | 150.0 |
| 12 | Antennas (2 m and 70 cm monopoles) and deployment | 2 | 35.0 | Version 1.0 §4.6 | E-1 | 25 % | 43.8 |
| 13 | Jetson Orin Nano module, carrier, thermal interface, mounting | 1 | 180.0 | Version 1.0 §4.7 (module 120, carrier 40, interface 10, mounting 10); no NVIDIA module mass was found | E-1 | 25 % | 225.0 |
| 14 | Jetson heat spreader to structure | 1 | 65.0 | Calculated, §4.4 | E-2 | 15 % | 74.8 |
| 15 | ADCS sensors (magnetometer, 6 sun sensors, gyro, brackets) | 1 set | 25.0 | Version 1.0 §4.8 | E-1 | 25 % | 31.2 |
| 16 | Magnetorquers (air-core coils) | 3 | 138.9 | Calculated, §4.5 | E-2 | 15 % | 159.7 |
| 17 | GNSS receiver, SkyFox piNAV-NG/FM | 1 | 24.0 | Datasheet Rev A/2025, per [D6 research](../funding_audit/research/D6_GNSS_RECEIVER.md) | A-5 | 3 % | 24.7 |
| 18 | GNSS patch antenna, Spacemanic ZENITH V3a | 1 | 20.0 | About 20 g, D6 research | A-5 | 3 % | 20.6 |
| 19 | GNSS cable, adapter and bracket | 1 | 26.0 | Balance of the ≈ 70 g GNSS allocation in BASELINE §4 | E-1 | 25 % | 32.5 |
| 20 | Harness | 1 | 80.0 | Version 1.0 §4.9 | E-1 | 25 % | 100.0 |
| 21 | Thermal control (battery blanket, heater, straps, interface material) | 1 | 30.0 | Version 1.0 §4.10 | E-1 | 25 % | 37.5 |
| 22 | Fasteners | many | 25.0 | Version 1.0 §4.11 | E-1 | 25 % | 31.2 |
| | **Total** | | **1,690.7** | | | **18.3 %** | **2,000.7** |

**CBE** = current best estimate. **Predicted** = CBE × (1 + MGA). "Version 1.0" means the item tables of this document's
Version 1.0 (2026-01-02), whose values are estimates without a drawing or datasheet behind them; they are kept only where
nothing better exists, at E-1 maturity.

### 3.2 Mass Margin

```
MASS BUDGET SUMMARY (analysis, not measured)

Limit (CDS Rev 14.1, BASELINE §4):       2,000 g
──────────────────────────────────────────────────
CBE total:                               1,691 g
Growth allowance (§2.2, upper values):     310 g   (18.3 %)
Predicted mass:                          2,001 g
──────────────────────────────────────────────────
Margin to 2.00 kg:                          −1 g   (≈ 0 %)   RED by the S-120 PDR guidance (> 9 % green)
```

| Growth case | Predicted mass | Margin to 2.00 kg | Margin to 2.40 kg (NRCSD backup) |
|-------------|----------------|-------------------|----------------------------------|
| §2.2 upper values (MGA 18.3 %) | 2,000.7 g | −0.0 % | +20.0 % |
| §2.2 lower values (MGA 13.5 %) | 1,918.5 g | +4.2 % | +25.1 % |
| BASELINE §4 uniform 15 % | 1,944.3 g | +2.9 % | +23.4 % |

**Result:** the CBE is **1.69 kg** and the predicted mass is **1.92–2.00 kg** against the 2.00 kg limit, a margin of
0–4 %. That is red at PDR whichever growth scheme is used; the result is driven by the CBE, not by the allowance. The
NRCSD backup limit (2.40 kg) would leave 20–25 %.

### 3.3 Differences from the Provisional MEL and from BASELINE §4

BASELINE §4 gives CBE ≈ 1.51 kg (the ODAR §2 provisional list, 1,443 g, plus ≈ 70 g GNSS) and MEV ≤ 1.75 kg. This MEL is
178 g heavier. Each difference is an estimate the PI can accept or reject:

| Item | ODAR §2 + GNSS (g) | This MEL (g) | Δ (g) | Reason |
|------|--------------------|--------------|-------|--------|
| EPS electronics, body panels, wings | 290 (180 + 110) | 327.8 (rows 3–6) | +37.8 | Panel substrates for the four body faces were not counted; cells now at the datasheet mass |
| Battery | 200 | 249.0 (rows 7–9) | +49.0 | Four cells alone reach 200 g at the datasheet maximum; holder and board added |
| Jetson and heat spreader | 198 | 245.0 (rows 13–14) | +47.0 | Spreader sized for the ≥ 1 W/K path in BASELINE §6 (§4.4) |
| Magnetorquers | 115 | 138.9 (row 16) | +23.9 | Copper mass of the specified coil (§4.5) |
| Internal brackets | — | 20.0 (row 2) | +20.0 | Listed in STRUCTURAL_DESIGN §2.1, absent from Version 1.0 |
| All other rows | 710 | 710.0 | 0 | Unchanged |
| **Total** | **1,513** | **1,690.7** | **+177.7** | |

Consequences, reported to the PI rather than changed here: BASELINE §4 (CBE ≈ 1.51 kg, MEV ≤ 1.75 kg) is not supported by
this MEL; the orbital-lifetime analyses (BASELINE §3, ODAR) use 1.6 kg with a 1.8 kg bound, which the predicted mass
exceeds, so the NASA DAS inputs must be updated (a heavier spacecraft decays more slowly).

## 4. Detailed Mass Breakdown

### 4.1 Structure (300 g CBE, rows 1–2)

| Item | Qty | Unit (g) | Total (g) | Basis |
|------|-----|----------|-----------|-------|
| Primary frame (aluminum) | 1 | 180 | 180 | Version 1.0 |
| Rails | 4 | 15 | 60 | Version 1.0 |
| Deployment switches | 4 | 5 | 20 | Version 1.0 |
| End panels | 2 | 10 | 20 | Version 1.0 |
| Internal brackets | 1 set | 20 | 20 | STRUCTURAL_DESIGN §2.1 |
| **Subtotal** | | | **300** | |

STRUCTURAL_DESIGN §2.1 gives 240 g for frame, rails, end plates and brackets together; the larger Version 1.0 values are
kept until a CAD model exists.

### 4.2 Solar Array (237.8 g CBE, rows 3–5)

Layout from `hardware/eps/SOLAR_ARRAY_DESIGN.md` (16 cells: 2 on each of the 4 long body faces, 2 on each side of 2
deployable wings).

| Item | Qty | Unit (g) | Total (g) | Basis |
|------|-----|----------|-----------|-------|
| 3G30A cell assemblies (40 × 80 mm, 100 µm coverglass) | 16 | 3.6 | 57.6 | Datasheet maximum |
| Panel substrate, FR-4, 83 × 98 × 1.6 mm | 6 | 24.7 | 148.2 | 8.3 × 9.8 × 0.16 cm = 13.0 cm³ × 1.9 g/cm³ **[dimensions to fit between the rails, and density, assumed]** |
| Wing hinge, spring, hold-down and release | 2 | 16.0 | 32.0 | ODAR wing allowance balance (row 5) |
| **Subtotal** | | | **237.8** | |

### 4.3 Battery (249 g CBE, rows 7–9)

| Item | Qty | Unit (g) | Total (g) | Basis |
|------|-----|----------|-----------|-------|
| Samsung INR18650-35E cell | 4 | 50.0 | 200.0 | Specification maximum (`hardware/eps/BATTERY_DESIGN.md` §2) |
| Cell-interconnect board | 1 | 15.0 | 15.0 | Version 1.0 |
| Holder: two Al 6061 plates, 80 × 40 × 3 mm, four Ø18.5 mm cell holes | 2 | 17.2 | 34.4 → 34 | (9.6 − 3.2) cm³ × 2.70 g/cm³ per plate **[geometry assumed]** |
| **Subtotal** | | | **249** | |

Version 1.0 listed a 5 g holder, which cannot be an aluminum holder for four 50 g cells.

### 4.4 AI Payload (245 g CBE, rows 13–14)

| Item | Qty | Unit (g) | Total (g) | Basis |
|------|-----|----------|-----------|-------|
| Jetson Orin Nano 8 GB module | 1 | 120 | 120 | Version 1.0 estimate; no published module mass found |
| Carrier board | 1 | 40 | 40 | Version 1.0 |
| Thermal interface | 1 | 10 | 10 | Version 1.0 |
| Mounting hardware | 1 | 10 | 10 | Version 1.0 |
| Heat spreader, Al 6061, 80 × 60 × 5 mm | 1 | 65 | 65 | 24 cm³ × 2.70 g/cm³ = 64.8 g. Conductance k·A/L = 167 W/m·K × (60 × 5 mm²) / 50 mm = 1.0 W/K, the BASELINE §6 minimum, before interface resistances **[path length and geometry assumed]** |
| **Subtotal** | | | **245** | |

### 4.5 Magnetorquers (138.9 g CBE, row 16)

| Item | Qty | Unit (g) | Total (g) | Basis |
|------|-----|----------|-----------|-------|
| Copper winding: 250 turns, 80 m of 30 AWG (Ø 0.254 mm) | 3 | 36.3 | 108.9 | `hardware/adcs/ACTUATOR_DESIGN.md` §2.4 wire length and gauge: 8,000 cm × π/4 × (0.0254 cm)² = 4.05 cm³ × 8.96 g/cm³ |
| Bobbin and potting | 3 | 10.0 | 30.0 | **[assumption]** |
| **Subtotal** | | | **138.9** | |

ACTUATOR_DESIGN §4.2 states "~20 g each", which is less than the copper in its own §2.4 coil (36.3 g). The coil drivers
are counted with the OBC.

### 4.6 Other Items

Rows 6, 8, 10–12, 15 and 20–22 carry the Version 1.0 item breakdowns (EPS board, OBC, RF, antennas, ADCS sensors,
harness, thermal control, fasteners) unchanged. They remain estimates (E-1) until each subsystem has a drawing or a
selected part with a datasheet. Rows 17–19 are the GNSS receiver added by decision D6.

## 5. Center of Gravity

Not yet computed: it needs a CAD layout with the item locations above. Requirement: within 2 cm of the geometric center
(SYS-003, CDS Rev 14.1). The battery (249 g) and the Jetson with its spreader (245 g) are the largest single masses and
should be placed with this requirement in mind. Version 1.0's "X = Y = 50 mm, Z = 55 mm, verified" had no model behind it.

## 6. Moments of Inertia

Not yet computed from a layout. For scale only, a uniform-density 100 × 100 × 113.5 mm box at the CBE (1.69 kg, wings
stowed) has Ixx = Iyy = m(b² + c²)/12 = 1.69 × (0.100² + 0.1135²)/12 = 0.0032 kg·m² and
Izz = 1.69 × (0.100² + 0.100²)/12 = 0.0028 kg·m². Deployed wings increase these. Version 1.0's 0.0020/0.0020/0.0018 kg·m²
were unsourced. The ADCS analysis (TB-09) needs the CAD values.

## 7. Mass Growth Tracking

### 7.1 Mass History

| Date | Source | CBE | Note |
|------|--------|-----|------|
| 2026-01-02 | Version 1.0 of this document | 1,200 g | Incomplete (§1.1) |
| 2026-10-14 | ODAR §2 provisional MEL | 1,443 g | Added magnetorquers, wings and spreader growth |
| 2026-10-15 | BASELINE §4 Rev B | ≈ 1,510 g | Added the GNSS receiver (D6) |
| 2026-10-16 | This document, Version 2.0 | 1,691 g | Bottom-up MEL (§3.3) |

### 7.2 Actions to Restore Margin

The predicted mass leaves no margin to 2.00 kg. Before PDR the project should either reduce mass or confirm a limit
above 2.00 kg. Candidates, none yet assessed: a measured Jetson module and carrier mass (row 13 is an unverified 180 g);
thinner panel substrates (1.0 mm instead of 1.6 mm would remove about 55 g across six panels); a lighter heat-spreader
material for the same conductance; a structure designed and weighed rather than estimated. The PI decides (see the
report accompanying this issue).

## 8. Mass Verification

### 8.1 Verification Method

| Phase | Method |
|-------|--------|
| Design | This MEL, updated at each design change; maturity codes advance as drawings and parts arrive |
| Build | Weigh each item on receipt; replace CBE with actual mass (MGA → 0 %) |
| Integration | Measurement after each step |
| Final | Precision measurement plus CG measurement (TP-INT-006 §8.6) |

### 8.2 Required Measurements

| Measurement | Equipment | Accuracy |
|-------------|-----------|----------|
| Total mass | Digital scale | ±1 g |
| CG location | CG fixture | ±2 mm |
| MOI | Bifilar pendulum | ±5 % |

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR release |
| 2.0 | 2026-10-16 | Funding audit (TB-06) | Re-issued as a bottom-up MEL with maturity-based growth allowances and one limit (2.00 kg, CDS Rev 14.1); CBE 1.69 kg, predicted 1.92–2.00 kg |
