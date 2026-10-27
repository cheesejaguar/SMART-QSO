# SMART-QSO Mass Budget

**Document ID**: SQSO-MB-001
**Version**: 3.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR issue. Bottom-up master equipment list (MEL) of the build-versus-buy configuration
(`docs/hardware/BUILD_VS_BUY.md` §1–§2) and [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) §4–§8, with mass growth,
centre of gravity and moments of inertia. **Analysis and estimates; nothing has been weighed** (no hardware exists).
Scripts: `software/simulation/cdr/structures/mass_properties.py` (MEL and reference-layout model) and, from Version 3.1,
`hardware/mechanical/model.py` (the CAD model of layout B, SQSO-MECH-CAD-001, which reads its masses from the MEL of this
document and computes the CG and the inertia tensor from the solids; §6.6).

## 1. Purpose and Summary

This document is the controlled mass budget for the SMART-QSO 1U CubeSat: every item with its current best estimate (CBE),
its basis, a mass growth allowance (MGA) set by its maturity, the maximum expected value (MEV = CBE × (1 + MGA)), the
margin against each mass limit, the centre of gravity (CG) and the inertia tensor stowed and deployed.

| Quantity | CR0002 rods (BUILD_VS_BUY §1) | Air-core coils (BASELINE §8) |
|----------|-------------------------------|------------------------------|
| CBE | **1,494.3 g** | **1,575.7 g** |
| MGA (mass-weighted) | 15.6 % | 15.9 % |
| MEV | **1,727.7 g** | **1,826.5 g** |
| Margin to 2.00 kg (SYS-002; CDS Rev 14.1 Table 1) | **+15.8 %** (lower-growth case +21.8 %) | **+9.5 %** (lower-growth case +15.1 %) |
| Margin to the EXOpod Nova 1U limit, 2.5 kg | +44.7 % | +36.9 % |
| Margin to the NRCSD backup limit, 2.40 kg | +38.9 % | +31.4 % |
| CG, stowed (recommended layout, §5) | (−0.8, −0.7, −4.7) mm | (−1.2, −1.0, −4.5) mm |
| CG, stowed, **from the CAD** (layout B as modelled, CBE / MEV; §6.6) | (+0.4, −0.1, −1.8) / (+0.5, −0.2, −0.7) mm | not modelled |
| Deployed inertia **from the CAD** (AntS option A, CBE / MEV; §6.6) | Iyy/max(Ixx, Izz) 1.056 / 1.047; major axis 0.6° / 1.1° from +Y | not modelled |
| CG limit | ±20 mm per axis (CDS Rev 14.1 §2.2.11, Table 2) | same |

Margin as defined by AIAA S-120A: (limit − MEV)/MEV. Both options are green at CDR by the guidance quoted in §2.2
(> 5 %). The coil option is **below the 10 % trigger of decision D8** (BASELINE §9: "revisit with 2S1P if the MEL re-run
after the COTS buys still has < 10 % mass margin"); the rod option is above it.

The mass is no longer the problem it was in Version 2.0 (0–4 % margin). The new problem is the **inertia**: with the
wings and the AntS antennas deployed, +Y (the spin axis of the sun-pointing law) is only 0.5–4 % larger than +Z, and the
ADCS Monte Carlo inputs do not hold. §6 gives the numbers, the corrected values and the layout that recovers them. A
2 m antenna element along ±Y, as the antenna design's recommended options B and C have, makes +Y the intermediate axis
and the sun-pointing fails (52–60 % of sunlit time within 15°): §6.4, item 6.

### 1.1 Changes from Version 2.0

Version 2.0 (2026-10-16) was the first bottom-up MEL, for an all-custom spacecraft: CBE 1,690.7 g, MEV 1,918–2,001 g. The
build-versus-buy decisions of 2026-10-26 replaced the frame, antennas, panels, magnetometer, gyro and (optionally) the
magnetorquers with catalogue parts. Net change −198.0 g CBE with rods, −116.6 g with coils:

| Item | v2.0 CBE (g) | v3.0 CBE (g) | Δ (g) | Reason |
|------|--------------|--------------|-------|--------|
| Frame, rails, end plates, switches, brackets | 300.0 | 158.0 (rows 1–5) | −142.0 | Bought 1U frame 108–120 g (BUILD_VS_BUY §2.2) instead of a 280 g custom estimate; switches, separation springs, RBF listed separately |
| Solar array: cells, panels, hinges | 237.8 | 274.2 (rows 6–11) | +36.4 | Vendor body panels (30–50 g each with cells) instead of a bare-FR-4 estimate; wing hardware rebuilt bottom-up |
| Battery and isolated mount | 249.0 | 257.0 (rows 13–16) | +8.0 | Isolator bushings, Ti screws, disc springs, two heater elements and cutout (BUILD_VS_BUY §4.3) |
| Jetson module, carrier, interface, spreader | 245.0 | 157.6 (rows 20–23) | −87.4 | Module 31 g (distributor weight) instead of the unsourced 120 g |
| Antennas | 35.0 | 85.0 (row 19) | +50.0 | ISIS AntS 77–90 g |
| Magnetorquers | 138.9 | 57.5 rods / 138.9 coils | −81.4 / 0 | CubeSpace CR0002 16.5 g each plus bracket |
| Fasteners, spacers, staking | 25.0 | 45.0 (rows 34–35) | +20.0 | Stack spacers and adhesives counted |
| All other rows (EPS, OBC, RF, ADCS sensors, GNSS, harness, thermal) | 460.0 | 460.0 | 0 | Unchanged; still estimates |
| **Total** | **1,690.7** | **1,494.3 / 1,575.7** | **−196.4 / −115.0** | |

BUILD_VS_BUY §1 predicted −200 to −250 g; the result is −198 g with rods because the antenna (+50 g) and the vendor panels
(+36 g) offset part of the frame saving.

### 1.2 Changes from Version 3.0 (CAD reconciliation)

The CAD model of layout B (`hardware/mechanical/model.py`, SQSO-MECH-CAD-001) carries every MEL row of §3.1 (rods
option) on one or more solids; the CAD total equals the MEL total exactly (1,494.3 g CBE, 1,727.7 g MEV, with row 23 at 66.6 g: see the end of this
section), so **the MEL masses and margins are those of §3.1**. What changes is the geometry behind §5 and §6: the CAD places each part where it
fits (no overlaps, checked by boolean intersection) instead of the reference layout's free boxes, uses the bought parts'
published envelopes (CR0002 rods 10.5 × 10.5 × 47 mm, not 70 mm; AntS 98 × 98 × 7 mm on a −Z face recessed 1.5 mm), the
antenna configuration the chair adopted (option A: one 2 m element along +X, one 70 cm element along −X; BASELINE §7) and
a placement chosen to cancel the product of inertia Iyz (§6.6). Results: the stowed CG moves from −4.7 to −1.8 mm in Z
(CBE); the deployed inertia is 3 % lower on every axis; the deployed major axis is within 0.6° (CBE) and 1.1° (MEV) of +Y,
where the reference layout gave 13–15°, which did not meet the BASELINE §4 limit of 5°. **Row 5 (internal brackets,
30 g CBE) is re-split, not grown (finding MCAD-02 of SQSO-MECH-CAD-001):** the torquer deck that the CAD adds (X rod, GNSS
receiver) is a 1.0 mm Al skeleton of 7.8 g (a solid plate would be 23 g) and takes 9 g with its clips; the Jetson
heat-path flanges, which row 5 had at 18 g, are 5.5 g of aluminium in the CAD and take 15 g with the thermal gasket,
fasteners and inserts; hinge doublers 4 g; GNSS patch bracket 2 g: total 30 g. Had row 5 instead grown by the 3 g the
deck needs over its first allocation, the MEV would have been 1,729.7 g and the margin +15.6 % (against +15.9 % before the pedestal). The wing outline
is 81.6 × 92 mm instead of 83 × 98 mm (FR-4 1.8 g lighter per wing, not taken). **Centre supports (chair decision,
2026-10-27):** two per board on the EPS, OBC and carrier boards at (0, ±21) mm and one at (0, 0) on the RF board, carried
by row 34 (2 g of spacers). **Gyro (OBC item C3):** the M-G370PDT0 in its socket stands 14.0 mm above the OBC, more
than the 11.4 mm to the carrier; it stays on the OBC at (−20, +30.5) mm and stands through a 27 × 27 mm window in the
Jetson carrier, framed by a 2.2 g Al stiffener (SQSO-MECH-CAD-001 §4.3, D-11). No MEL change: row 25 is unchanged and
row 21 (carrier, 40 g) carries the frame in place of the 2.2 g of FR-4 the window removes; the mass properties below
include it. **Jetson thermal interface (SQSO-JET-BRD-001 §14, item M1):** the spreader carries a
22 × 22 × 0.75 mm pedestal over the SoC lid and a 0.5 mm second face over the DRAM and regulators: 1.8 g of Al on top of
the 64.8 g plate: **row 23 raised from 65.0 to 66.6 g CBE** (chair, 2026-10-27; `mass_properties.py` updated): CBE
1,494.3 g, MEV 1,727.7 g, margin +15.8 % (from +15.9 %) with rods; 1,575.7 / 1,826.5 g, +9.5 % with coils. The
reference-layout rows of §5 and §6.1 (layouts A and B) were computed with row 23 at 65.0 g; re-run with 66.6 g they move
by ≤ 0.1 mm in CG and ≤ 0.004 × 10⁻³ kg·m² in inertia, and no ratio by more than 0.001, so they are left as written.
The gap pads (about 5 g) sit in row 22's 10 g interface allocation and the two steel module standoffs (0.8 g) in row
21.

## 2. Mass Requirements

### 2.1 Mass Limits

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Maximum mass | **2.00 kg** | SYS-002 (≤ 2.0 kg); BASELINE §4. CDS Rev 14.1 §2.2.10 gives 2.00 kg as the *typical maximum* (a note, not a "shall"); masses above it are evaluated mission by mission |
| Integrator limit | 2.5 kg for a 1U | Exolaunch EXOpod Nova User Manual Rev 1.2 (June 2024) Table 1. The classic EXOpod (Rev 6.0, 2022) carries 1U only in a shared 3U slot with an adapter; its 1U figure was not extracted |
| Backup (ISS) limit | 2.40 kg | BASELINE §4 (NRCSD) |
| CG | Within ±2 cm of the geometric centre on each axis | CDS Rev 14.1 §2.2.11 and Table 2; SYS-003; EXOpod Nova Table 2 (±20 mm, a guideline) |
| Mass properties reported to the integrator | ±15 % at L-8 months, ±5 % at L-3.5 months, as measured at L-2 months | SpaceX Rideshare Payload User's Guide v11 (August 2026) Table G-3, as carried in SQSO-LVR-001 LVR-005 |
| Deployed inertia | **Proposed (STR-01):** with the wings and antennas deployed, Iyy ≥ 1.03 × max(Ixx, Izz) at CBE and at MEV (design target 1.05), and the major principal axis within 5° of +Y | §6.4; ADCS Monte Carlo (this document §6.5) |

The governing limit is 2.00 kg (SYS-002): it is lower than the integrator's.

### 2.2 Growth Allowance and Margin Method

Method: ANSI/AIAA S-120A-2015, *Mass Properties Control for Space Systems* (definitions and maturity-based growth). The
standard's own MGA table could not be obtained for this issue (it is sold by AIAA; the NASA briefing cited in Version 2.0,
NTRS 20130014265, does not reproduce it). The percentages below are **this project's choices**, the same scheme as
Version 2.0 with one category added for catalogue parts whose vendor or configuration is still open:

| Code | Meaning | MGA | Lower-growth value (sensitivity) |
|------|---------|-----|----------------------------------|
| SPEC | Published not-to-exceed (maximum) mass | 0 % | 0 % |
| C1 | Vendor-published mass of a selected part, configuration fixed | 3 % | 2 % |
| C2 | Vendor-published range, vendor or configuration not yet selected: CBE at the middle of the range, MGA to the top of the range (or 10 % when only one figure exists) | per item, 4–25 % | per item |
| E2 | Layout: calculated from assumed dimensions and materials | 15 % | 12 % |
| E1 | Estimate: no drawing; an earlier estimate or a similar item | 25 % | 18 % |

Definitions (S-120A): MGA % = (MEV − CBE)/CBE × 100; mass margin % = (allowable − MEV)/MEV × 100. The S-120 guidance as
quoted in Version 2.0 grades a margin above 5 % green at CDR.

## 3. Master Equipment List

### 3.1 MEL (both magnetorquer options)

| # | Item | Qty | CBE (g) | Basis | Code | MGA | MEV (g) |
|---|------|-----|---------|-------|------|-----|---------|
| 1 | 1U frame: rails, side frames, end frames (bought) | 1 | 115.0 | Spacemanic 115 g; ISIS 108 g with secondary structure; EnduroSat 120 g (delisted); Pumpkin not stated (BUILD_VS_BUY §2.2) | C2 | 4 % | 120.1 |
| 2 | Deployment switches: two in −Z rail standoffs, one in a +Z standoff | 3 | 6.0 | 3 × 2 g; three switches for the three physical inhibits (BASELINE §4) | E1 | 25 % | 7.5 |
| 3 | Separation spring plungers, −Z standoffs | 2 | 3.0 | CDS §2.2.14 (shared dispenser): < 6.7 N, stroke > 2.5 mm | E1 | 25 % | 3.8 |
| 4 | RBF pin and housing, −Y face | 1 | 4.0 | Estimate | E1 | 25 % | 5.0 |
| 5 | Internal brackets (Jetson heat path, GNSS patch, rods, hinge doublers) | 1 set | 30.0 | Custom brackets on a bought frame; Version 2.0 had 20 g | E1 | 25 % | 37.5 |
| 6 | Body solar panels, vendor, 2 cells, photodiode and thermistor each | 4 | 160.0 | Spacemanic Ra ≈ 30 g, DHV < 50 g, ISIS 50 g per panel with cells (BUILD_VS_BUY §2.3): 40 g each, MGA to 50 g | C2 | 25 % | 200.0 |
| 7 | Wing W1 panel: FR-4 1.6 mm, photodiode, thermistor, connector, back-wiring | 1 | 28.7 | 83 × 98 × 1.6 mm at 1.9 g/cm³ = 24.7 g + 4 g | E2 | 15 % | 33.0 |
| 8 | Wing W2 panel (as row 7) | 1 | 28.7 | As row 7 | E2 | 15 % | 33.0 |
| 9 | Solar cells on the wings, AZUR SPACE 3G30A | 8 | 28.8 | ≤ 3.6 g each, datasheet DB 00010891-01 | SPEC | 0 % | 28.8 |
| 10 | Wing W1 hinges (2), torsion springs (2), hold-down, stop, deployed-state sensor, hinge flex jumper | 1 | 14.0 | Hinges 6, springs 1, hold-down 3, bumpers 1, sensor 1, flex 2 g | E1 | 25 % | 17.5 |
| 11 | Wing W2 hinge set (as row 10) | 1 | 14.0 | As row 10 | E1 | 25 % | 17.5 |
| 12 | EPS board (MPPT, BMS, PDU, burn drivers), in-house | 1 | 90.0 | Version 2.0 row 6 | E1 | 25 % | 112.5 |
| 13 | Battery cells, Samsung INR18650-35E (4S1P) | 4 | 200.0 | ≤ 50.0 g each, Samsung SDI specification | SPEC | 0 % | 200.0 |
| 14 | Battery interconnect and sense board | 1 | 15.0 | Version 2.0 row 8 | E1 | 25 % | 18.8 |
| 15 | Isolated battery mount: Al 6061-T6 clamp bracket | 1 | 34.0 | Version 2.0 §4.3; BUILD_VS_BUY §4.3 | E2 | 15 % | 39.1 |
| 16 | Isolated-mount hardware: 4 G-10CR bushings, 4 Ti-6Al-4V M2.5 screws, 4 disc springs, heater (2 elements), cutout, VDA tape, pad | 1 set | 8.0 | Bushings 1.2, screws 1.6, disc springs 0.8, heaters 2, cutout 1.5, tape and pad 1 g | E2 | 15 % | 9.2 |
| 17 | OBC (STM32L4R5ZI, BMI088 backup gyro, coil drivers), in-house | 1 | 45.0 | Version 2.0 row 10 | E1 | 25 % | 56.2 |
| 18 | RF: transponder receiver, 2 m transmitter, PA, filters, shield, in-house | 1 | 120.0 | Version 2.0 row 11 | E1 | 25 % | 150.0 |
| 19 | Antenna system, ISIS AntS 1U–3U, monopole/hybrid, 4 tape elements | 1 | 85.0 | 77–85 g by configuration (ISIS brochure, search listing), 89 g (SatNow listing), < 90 g (BUILD_VS_BUY §2.6): 85 g, MGA to 90 g | C2 | 6 % | 90.1 |
| 20 | Jetson Orin Nano 8 GB module (900-13767-0030-000) | 1 | 31.0 | 30.8 g distributor shipping weight (Farnell 4200232); NVIDIA publishes no module mass | C2 | 10 % | 34.1 |
| 21 | Jetson carrier board, in-house | 1 | 40.0 | Version 2.0 §4.4 | E1 | 25 % | 50.0 |
| 22 | Jetson thermal interface and module clamp | 1 | 20.0 | Version 2.0 §4.4 (interface 10, mounting 10) | E1 | 25 % | 25.0 |
| 23 | Jetson heat spreader to the frame, Al 6061 80 × 60 × 5 mm, with the SoC pedestal | 1 | 66.6 | Plate 64.8 g Al + pedestal and second face 1.8 g (SQSO-MECH-CAD-001 §4.5; Version 2.0 §4.4: ≥ 1 W/K, BASELINE §6) | E2 | 15 % | 76.6 |
| 24 | Magnetometer (M315, MM200 or Lodestone) | 1 | 10.0 | 8–12 g (BUILD_VS_BUY §2.7) | C2 | 20 % | 12.0 |
| 25 | Gyro, Epson M-G370PDT0 | 1 | 10.0 | 10 g (BUILD_VS_BUY §5.2) | C1 | 3 % | 10.3 |
| 26 | ADCS sensor brackets and cables (sun sensors are the panel photodiodes) | 1 set | 5.0 | Estimate | E1 | 25 % | 6.2 |
| 27r | *Rods option:* magnetorquer rods, CubeSpace CR0002 | 3 | 49.5 | 16.5 g each (BUILD_VS_BUY §2.7) | C1 | 3 % | 51.0 |
| 28r | *Rods option:* rod bracket | 1 | 8.0 | Estimate | E1 | 25 % | 10.0 |
| 27c | *Coils option:* air-core magnetorquers 80 × 80 mm, 250 turns | 3 | 138.9 | Version 2.0 §4.5 (copper 36.3 g + bobbin 10 g each) | E2 | 15 % | 159.7 |
| 29 | GNSS receiver, SkyFox piNAV-NG/FM | 1 | 24.0 | Datasheet (D6 research) | C1 | 3 % | 24.7 |
| 30 | GNSS patch antenna, Spacemanic ZENITH V3a, +Z face | 1 | 20.0 | About 20 g (D6 research) | C2 | 10 % | 22.0 |
| 31 | GNSS cable, adapter and bracket | 1 | 26.0 | Balance of the ≈ 70 g GNSS allocation | E1 | 25 % | 32.5 |
| 32 | Harness, including the 100 mm battery flying leads and the wing flex jumpers | 1 | 80.0 | Version 2.0 row 20 (`hardware/harness/HARNESS_DESIGN.md` §9 lists 55 g without the GNSS and the wing jumpers) | E1 | 25 % | 100.0 |
| 33 | Thermal control (tapes, interface materials, MLI patches) | 1 | 30.0 | Version 2.0 row 21 | E1 | 25 % | 37.5 |
| 34 | Fasteners, PCB-stack spacers (4 columns plus centre supports), inserts | many | 35.0 | Version 2.0 row 22 (25 g) + spacers 10 g | E1 | 25 % | 43.8 |
| 35 | Staking, conformal coat, adhesives | 1 | 10.0 | Estimate | E1 | 25 % | 12.5 |
| | **Total, rods option (rows 1–26, 27r, 28r, 29–35)** | | **1,494.3** | | | **15.6 %** | **1,727.7** |
| | **Total, coils option (rows 1–26, 27c, 29–35)** | | **1,575.7** | | | **15.9 %** | **1,826.5** |

### 3.2 Margin

```
MASS BUDGET SUMMARY (analysis, not measured)          rods        coils
Limit (SYS-002, CDS Rev 14.1 Table 1):              2,000 g     2,000 g
CBE                                                 1,494.3 g   1,575.7 g
Growth (maturity-based, section 2.2)                  233.4 g     250.8 g
MEV                                                 1,727.7 g   1,826.5 g
Margin to 2.00 kg                                     +15.8 %      +9.5 %
```

| Growth case | Rods MEV | Rods margin to 2.00 kg | Coils MEV | Coils margin to 2.00 kg |
|-------------|----------|------------------------|-----------|-------------------------|
| §2.2 values | 1,727.7 g | +15.8 % | 1,826.5 g | +9.5 % |
| §2.2 lower-growth values | 1,641.5 g | +21.8 % | 1,737.2 g | +15.1 % |
| Uniform 15 % (BASELINE §4 method) | 1,718.4 g | +16.4 % | 1,812.1 g | +10.4 % |
| Uniform 25 % (every item treated as an estimate) | 1,867.9 g | +7.1 % | 1,969.6 g | +1.5 % |

The largest single uncertainties are the in-house boards (EPS, OBC, RF, carrier: 295 g CBE, all E1) and the vendor body
panels (160–200 g). Weighing the first engineering-model boards retires most of the E1 growth.

### 3.3 Consequences for Other Documents (for the chair)

1. BASELINE §4 mass row: replace with the text proposed in §9.
2. Decision D8 (battery): with coils the margin is 9.5 %, below the 10 % trigger. Either choose the CR0002 rods (margin
   15.8 %; the ADCS Monte Carlo must then be re-run with rod remanence, BUILD_VS_BUY §6.5) or revisit 2S1P (−125 g, margin
   rises to ≈ 17 %).
3. Orbital lifetime (BASELINE §3, ODAR): the analyses use 1.6 kg with a 1.8 kg bound. The MEV is 1.73 kg (rods) to
   1.82 kg (coils); the bound should be 1.85 kg. A heavier spacecraft decays more slowly, so the wings-failed case at
   500–525 km should be re-run in NASA DAS at 1.85 kg.
4. SQSO-LVR-001 LVR-005 and the launch ICD §4.1: the mass to report at L-8 months is 1.73 ± 0.26 kg (rods) or
   1.82 ± 0.27 kg (coils), not 1.85 kg ± 15 % from Version 2.0.

## 4. Reference Layout

Version 3.0 had no CAD model; the CAD of Version 3.1 is in §6.6. The analytic CG and inertia of §5–§6.4 use the
reference layout of `docs/environment/STRUCTURAL_ANALYSIS.md` §3,
items modelled as boxes, bars and square loops in the CDS body frame (origin at the geometric centre; +Z along the rails,
−Z inserted first; faces at ±50 mm; rails to ±56.75 mm; wings hinged on the ±X/+Y edges, stowed over ±X, deployed into
a plane parallel to the +Y face: `hardware/eps/SOLAR_ARRAY_DESIGN.md` §3.1).

Two stack orders are evaluated:

| Layout | Stack from −Z to +Z (centre z, mm) |
|--------|------------------------------------|
| A, "ref" (battery mid-stack) | AntS outside −Z (−53); magnetorquers (−44 to −46); EPS (−38); battery (−20); OBC (−2); RF (+10); carrier and module (+22, +26); heat spreader (+33.5); GNSS receiver (+40); GNSS patch outside +Z (+52.5) |
| **B, "endmass" (recommended, STR-01)** | AntS outside −Z (−53); **battery on its isolators at the −Z end (−35)**; magnetorquer X/Y rods or Z coil (−19.5); EPS (−12); OBC (−3); carrier and module (+6, +10); heat spreader (+19); **RF at the +Z end (+36)**; GNSS receiver (+45); patch (+52.5) |

The AntS VHF elements are taken along ±X (parallel to the wings) unless stated; §6.3 shows why.

## 5. Centre of Gravity

| Case | Layout A, rods | Layout B, rods | Layout B, coils | **Layout B, rods, CAD (v3.1)** | Limit |
|------|----------------|----------------|-----------------|--------------------------------|-------|
| Stowed, CBE (x, y, z) | (−0.8, −0.7, −4.8) mm | (−0.8, −0.7, −4.7) mm | (−1.2, −1.0, −4.5) mm | **(+0.4, −0.1, −1.8) mm** | ±20 mm |
| Stowed, MEV | (−0.7, −0.6, −4.3) mm | (−0.7, −0.6, −3.7) mm | (−1.1, −1.0, −3.6) mm | **(+0.5, −0.2, −0.7) mm** | ±20 mm |
| Deployed, CBE | (−0.8, +2.2, −4.8) mm | (−0.8, +2.2, −4.7) mm | (−1.2, +1.7, −4.5) mm | (+0.8, +2.1, −1.8) mm | — |
| One wing deployed (W1) | (+0.6, +0.8, −4.8) mm | (+0.6, +0.8, −4.7) mm | (+0.2, +0.3, −4.5) mm | (+2.0, +0.9, −1.8) mm | — |

Placement uncertainty: moving every item independently by ±5 mm (1σ) per axis moves the CG by 1.2 mm (1σ). The stowed CG
is within 5 mm of the geometric centre, with ≥ 15 mm margin to the CDS limit on every axis. The −Z offset comes from the
AntS on the −Z face. Requirement and verification: SQSO-LVR-001 LVR-026, VI-LV-02; this document §8.

## 6. Moments of Inertia

### 6.1 Results

About the CG, kg·m² (×10⁻³), body axes. "VHF along X": the two AntS VHF tape elements deploy along ±X.

| Case | Ixx | Iyy | Izz | Ixy | Ixz | Iyz | Principal | Major axis from +Y | Iyy / max(Ixx, Izz) |
|------|-----|-----|-----|-----|-----|-----|-----------|--------------------|---------------------|
| A rods, stowed, CBE | 2.664 | 2.917 | 2.756 | −0.025 | −0.020 | −0.015 | 2.657, 2.760, 2.920 | 7.0° | 1.058 |
| A rods, deployed, CBE | 2.780 | 4.188 | 4.141 | −0.029 | −0.020 | −0.036 | 2.779, 4.122, 4.208 | 28° | **1.011** |
| A rods, deployed, MEV | 3.203 | 4.757 | 4.733 | −0.031 | −0.021 | −0.036 | 3.202, 4.708, 4.783 | 36° | **1.005** |
| A coils, deployed, MEV | 3.498 | 5.047 | 5.049 | −0.001 | +0.021 | +0.006 | 3.498, 5.042, 5.054 | 51° (Z) | **1.000** |
| A rods, deployed, CBE, VHF along Y | 3.291 | 3.677 | 4.141 | −0.029 | −0.020 | −0.036 | 3.288, 3.677, 4.144 | 86° (Z) | **0.888** |
| B rods, stowed, CBE | 2.788 | 3.040 | 2.756 | −0.025 | −0.028 | −0.024 | 2.736, 2.804, 3.044 | 6.6° | 1.091 |
| B rods, stowed, MEV | 3.209 | 3.487 | 3.208 | −0.027 | −0.034 | −0.028 | 3.170, 3.243, 3.492 | 7.0° | 1.086 |
| **B rods, deployed, CBE** | **2.904** | **4.312** | **4.141** | −0.029 | −0.028 | −0.044 | 2.903, 4.131, 4.322 | 13° | **1.041** |
| **B rods, deployed, MEV** | **3.335** | **4.889** | **4.733** | −0.031 | −0.034 | −0.046 | 3.334, 4.721, 4.902 | 15° | **1.033** |
| B coils, deployed, CBE | 3.132 | 4.536 | 4.405 | −0.000 | −0.009 | −0.024 | 3.132, 4.401, 4.541 | 10° | 1.030 |
| B coils, deployed, MEV | 3.606 | 5.154 | 5.049 | −0.001 | −0.014 | −0.025 | 3.606, 5.043, 5.160 | 13° | 1.021 |
| B rods, one wing (W1) deployed, CBE | 2.867 | 3.950 | 3.742 | — | — | — | 2.819, 3.742, 3.998 | 12.5° | 1.056 |
| B rods, no wing deployed, antennas out, CBE | 2.824 | 3.587 | 3.337 | — | — | — | 2.822, 3.336, 3.590 | 5.4° | 1.075 |
| **CAD (v3.1), B rods, AntS option A, stowed, CBE** | 2.716 | 3.079 | 2.805 | +0.003 | +0.001 | +0.027 | 2.716, 2.802, 3.082 | 5.5° | 1.098 |
| **CAD, stowed, MEV** | 3.133 | 3.557 | 3.283 | +0.003 | +0.000 | +0.031 | 3.133, 3.279, 3.560 | 6.5° | 1.083 |
| **CAD, deployed, CBE** | **2.780** | **3.963** | **3.753** | +0.006 | +0.032 | −0.002 | 2.779, 3.754, 3.963 | **0.6°** | **1.056** |
| **CAD, deployed, MEV** | **3.205** | **4.524** | **4.321** | +0.006 | +0.034 | +0.004 | 3.204, 4.322, 4.524 | **1.1°** | **1.047** |
| CAD, deployed, CBE, MEL hybrid antennas (2 m ±X, 70 cm ±Y; compare row 8) | 2.819 | 4.216 | 4.044 | +0.005 | +0.001 | −0.002 | 2.819, 4.044, 4.216 | 0.8° | 1.043 |
| CAD, deployed, MEV, MEL hybrid antennas (compare row 9) | 3.245 | 4.792 | 4.630 | +0.004 | +0.000 | +0.004 | 3.245, 4.630, 4.792 | 1.3° | 1.035 |
| CAD, W1 deployed only, CBE | 2.746 | 3.663 | 3.419 | −0.159 | +0.022 | +0.008 | — | 9.6° | 1.071 |
| CAD, antennas out, wings stowed, CBE | 2.708 | 3.373 | 3.090 | +0.003 | +0.032 | +0.018 | — | 3.7° | 1.091 |

The CAD rows use the tensor convention of the rows above (off-diagonal terms are −Σ m·x·y and so on); the reference-layout
rows were written with the same convention.

### 6.2 Comparison with the Flight Software and the Mission Simulation

| Source | Ixx | Iyy | Izz | Iyy / Izz |
|--------|-----|-----|-----|-----------|
| `INERTIA` in `software/rust/sqso-fsw/src/adcs.rs` (deployed) | 2.9 | 4.1 | 3.7 | 1.108 |
| Mission simulation, deployed (`sqso-sil/src/env/attitude.rs`: uniform 1.6 kg box, 55 g wings centred at (±100, +50, 0) mm, no antennas) | 3.19 | 4.09 | 3.90 | 1.049 |
| Mission simulation, stowed | 3.05 | 3.05 | 2.67 | 1.000 |
| This analysis, layout B rods, deployed CBE / MEV | 2.90 / 3.34 | 4.31 / 4.89 | 4.14 / 4.73 | 1.041 / 1.033 |
| This analysis, layout A rods, deployed CBE / MEV | 2.78 / 3.20 | 4.19 / 4.76 | 4.14 / 4.73 | 1.011 / 1.005 |

The flight constant is within 0–13 % of the computed values on each axis. The axis that matters, the margin of Iyy over
Izz, is not: the flight constant and the simulation assume 5–11 %; the computed value is 0.5–4 %. The simulation's model
differs in three ways that all reduce Izz: no AntS tape elements (they add ≈ 0.6 × 10⁻³ kg·m² about Z), uniform body
density (real mass is concentrated near the centre), and wings centred 50 mm off the Y axis instead of 39.5 mm (the hinge
must stay 8.5 mm clear of the rail, CDS §2.2.5).

### 6.3 Why +Y Is Nearly Degenerate with +Z

Iyy − Izz = Σ m (z² − y²) over all parts. The body is 113.5 mm long in Z and 100 mm in Y, which favours Y by
≈ m (113.5² − 100²)/12. Everything else works against it:

- The deployed wings lie in a plane 39.5 mm on the +Y side and span only ±49 mm in Z: net −0.08 × 10⁻³ kg·m².
- The AntS tape elements lie in the plane of the −Z face. An element along Y adds ≈ 0.33 × 10⁻³ kg·m² to Izz and nothing
  to Iyy; along X it adds the same to both. **Clocking the VHF elements along ±Y makes +Y the intermediate axis**
  (Iyy/Izz = 0.89, row 5 of §6.1): the spin about +Y is then unstable, rigid or not.
- Concentrated mass near z = 0 (the battery mid-stack in layout A) adds to Ixx and Iyy little.

Putting the heavy items at the ±Z ends (layout B) and the VHF elements along ±X gives 1.02–1.05. Moving the heat spreader
to the +Z end frame and using one VHF element instead of two (the baseline needs one 2 m monopole) would add ≈ 0.01.

### 6.4 Does the ADCS Monte Carlo Hold? (STR-01)

The Monte Carlo of BUILD_VS_BUY §6 (the basis of BASELINE §8 "mean 2.5°, 100 % of sunlit time within 15°") ran the
simulation's diagonal inertia (Iyy/Izz = 1.049). It was re-run here with the computed inertias, in a scratch copy of
`sqso_adcs_mc` whose plant inertia is read from the environment (no repository code changed; the flight constant
`INERTIA` unchanged unless stated): 40 trials of 2 days, 6°/s spin, the flight gains, M-G370PDT0 gyro, residual dipole
3 mA·m² unless stated. Products of inertia are not modelled by the simulation (diagonal only), so the results are
optimistic for the tilted principal axes of §6.1.

| Plant inertia, deployed (×10⁻³ kg·m²) | Iyy/Izz | Mean Sun angle | Worst trial p95 | Sunlit time within 15° | Worst at eclipse exit | Worst-trial generation |
|---------------------------------------|---------|----------------|-----------------|------------------------|-----------------------|------------------------|
| Simulation default (3.19, 4.09, 3.90) | 1.049 | 2.5° | 9.8° | 100 % | 12° | 4.50 W |
| Layout A rods CBE (2.78, 4.19, 4.14) | 1.011 | 3.2° | 11.5° | 100 % | 28° | 4.49 W |
| Layout A rods MEV (3.20, 4.76, 4.73) | 1.005 | 8.1° | 145° | 93.4 % | 170° | 3.30 W |
| Layout A coils MEV (3.50, 5.04, 5.05) | 0.999 | 20.2° | 154° | 76.9 % | 179° | 3.19 W |
| Layout A rods CBE, VHF along Y (3.29, 3.68, 4.14) | 0.888 | 11.1° | 142° | 88.8 % | 178° | 3.71 W |
| Layout A rods CBE, dipole 10 mA·m² | 1.011 | 24.0° | 160° | 69.3 % | 179° | 3.23 W |
| Ratio 1.02 (2.95, 4.22, 4.14) | 1.020 | 2.9° | 10.8° | 100 % | 16° | 4.49 W |
| Ratio 1.03 | 1.030 | 2.7° | 10.5° | 100 % | 13° | 4.50 W |
| Ratio 1.03, dipole 10 mA·m² | 1.030 | 7.3° | 30° | 91.7 % | 40° | 4.40 W |
| **Layout B rods CBE (2.90, 4.31, 4.14)** | 1.041 | 2.6° | 9.9° | 100 % | 12° | 4.50 W |
| **Layout B rods MEV (3.34, 4.89, 4.73)** | 1.033 | 2.6° | 9.4° | 100 % | 11° | 4.50 W |
| Layout B coils CBE (3.13, 4.54, 4.41) | 1.030 | 2.7° | 10.1° | 100 % | 12° | 4.50 W |
| Layout B coils MEV (3.61, 5.15, 5.05) | 1.021 | 2.7° | 9.7° | 100 % | 16° | 4.50 W |
| Layout B rods MEV, dipole 10 mA·m² | 1.033 | 6.6° | 27° | 93.9 % | 36° | 4.43 W |
| Layout B rods MEV, tip-off σ 10°/s to 30°/s | 1.033 | 2.6° | 9.4° | 100 % | 11° | 4.50 W |
| Layout B rods (end-mass variant with rods at −44 mm: 2.95, 4.35, 4.14), flight `INERTIA` changed to (3.2, 4.6, 4.4) | 1.052 | 2.5° | 9.3° | 100 % | 12° | 4.50 W |
| Layout B rods, AntS option A of SQSO-RF-ANT-001 v2.0 §3 (one 2 m element along +X), CBE (2.87, 4.06, 3.85) | 1.054 | 2.6° | 9.8° | 100 % | 12° | 4.50 W |
| Same, MEV (3.30, 4.62, 4.43) | 1.044 | 2.5° | 9.2° | 100 % | 11° | 4.50 W |
| Layout B rods, AntS **option B** (one 2 m element along +Y), CBE (3.16, 3.76, 3.85) | 0.978 | **36.1°** | 158° | **58.5 %** | 179° | 3.19 W |
| Same, MEV (3.61, 4.31, 4.42) | 0.974 | **43.1°** | 154° | **51.6 %** | 179° | 3.22 W |
| Layout B rods, AntS **option C** (2 m elements along +X and +Y, quadrature), CBE (3.16, 4.06, 4.14) | 0.980 | **38.5°** | 156° | **54.9 %** | 179° | 3.22 W |
| Same, MEV (3.60, 4.62, 4.73) | 0.976 | **34.0°** | 154° | **59.6 %** | 179° | 3.36 W |
| Option C CBE with 8°/s spin and nutation gain 3·10⁻⁴ | 0.980 | 26.2° | 163° | 75.3 % | 179° | 3.62 W |

Findings:

1. **The Monte Carlo inputs do not hold for layout A.** At MEV the +Y spin degrades to 93 % (rods) or 77 % (coils) of
   sunlit time within 15°, with trials that lose the Sun. Generation in the worst trial falls from 4.5 W to 3.2–3.3 W.
2. **Iyy/Izz ≥ 1.03 recovers the published result**, with the nominal 3 mA·m² dipole. At 10 mA·m² every case degrades;
   the ratio makes it worse.
3. **Detumble is unaffected** (all cases captured within 2.3 h).
4. The flight `INERTIA` constant is not the problem: changing it to the computed values gives the same result. It should
   still be corrected (§9).
5. A gain change also tolerates the degenerate case: at Iyy/Izz = 1.005, 8°/s spin with nutation gain 3·10⁻⁴ N·m·s gave
   2.0° mean and 100 % within 15° (worst eclipse exit 31°). That is for the ADCS discipline to evaluate (8°/s is where
   BUILD_VS_BUY §6.3 expected the 1 Hz sun-sensor sampling to start to matter).
6. **Any 2 m element along ±Y defeats the sun-pointing.** The antenna design (`hardware/rf/ANTENNA_DESIGN.md` v2.0 §3)
   recommends option C (2 m elements along ±X and ±Y in quadrature) or option B (one along +Y) for the link fading.
   Both make +Y the intermediate axis (Iyy/Izz 0.97–0.98): mean Sun angle 34–43°, 52–60 % of sunlit time within 15°,
   worst-trial generation 3.2–3.4 W instead of 4.5 W; the gain change of item 5 does not recover it (75 %). Option A
   (one 2 m element along ±X) gives 1.03–1.05 and the published result. This is a conflict between the link and the
   attitude design for the chair (STR-01): with the AntS (elements only in the plane of the ±Z face), either the 2 m
   element lies along ±X and the fading of option A is accepted, or a Y element is compensated by ≈ 0.5 × 10⁻³ kg·m² of
   Iyy (≈ 200 g of ballast at the ±Z ends, which the 15.9 % rods margin would almost exhaust), or the ADCS law is
   re-designed for spin about an intermediate axis and shown by Monte Carlo.

Reproduce with `software/simulation/cdr/structures/adcs_mc_inertia.sh` (it copies the Rust workspace, patches the copy
and runs every case above; it needs a tree that compiles). The runs above were made on the tree of 2026-10-27 04:45 UTC.

### 6.5 Corrected Values for the Flight Software and the Simulation (for the chair)

- `sqso_fsw::adcs::INERTIA`: Version 3.0 proposed **(0.0032, 0.0046, 0.0044) kg·m²** (reference layout B, hybrid
  antennas, mean of CBE and MEV). **Version 3.1, from the CAD (§6.6), antenna option A, mean of CBE and MEV:
  (0.0030, 0.0042, 0.0040) kg·m²** (diagonal; the products are below 3.3 × 10⁻⁵ kg·m², Ixz the largest). To be replaced
  by the measured stowed inertia plus the weighed wings before flight. For coils (reference layout only):
  (0.0034, 0.0049, 0.0047).
- Mission simulation (`sqso-sil/src/env/attitude.rs`): use the full inertia tensor of §6.1 (products included), the
  AntS elements, the wing centre at (±93.5, +39.5, 0) mm and the MEL mass (1.73 kg MEV), and re-run the Monte Carlo
  with the products of inertia.

### 6.6 CAD Reconciliation (Version 3.1)

Source: `hardware/mechanical/model.py` and `hardware/mechanical/results.json` (SQSO-MECH-CAD-001 §5). Every MEL row of
§3.1 (rods option) is carried by named solids; rows that cover several parts (row 1 frame, row 5 brackets, rows 10–11
hinge sets, row 32 harness, rows 33–35 distributed items) are split as listed there. Each solid has uniform density, so
its own inertia is included (not only its CG). Differences from the reference layout of §4 and their effect:

| Item | Reference layout (§4) | CAD (v3.1) | Effect |
|------|------------------------|-----------|--------|
| AntS face | Outside −Z, centre −53 mm | Housing z −55.5 to −48.5 (−Z face recessed 1.5 mm, STR-11) | +1 mm |
| Antenna elements, deployed | Hybrid: 2 m ±X, 70 cm ±Y | Option A (chair): one 2 m tape along +X, one 70 cm along −X | Iyy, Izz −0.25 × 10⁻³ |
| Battery | Centre −35 mm, 74 × 65 × 18.5 box | Four cells along X, centre −32.9 mm, clamp −44.9 to −20.9, on isolators at (±38.5, ±31) | Ixx, Iyy −0.02 × 10⁻³ |
| Magnetorquer rods | 70 × 10 mm bars | CR0002 10.5 × 10.5 × 47 mm (published): X on the deck at (0, +36, −10), Y beside the battery at (+42, 0, −33), Z at (−41.5, 0) from −44 to +3 through notches in the deck and the EPS board | small |
| Boards | Boxes 90 × 96 × 6–12 mm at −12 … +36 | PC/104 substrates and component envelopes: EPS −3.0, OBC +5.0, carrier +18.0, module +23.6, spreader +27.8, RF +34.8 (substrate bottom z) | CG +1 to +3 mm in Z |
| Jetson module and spreader | Centred | Module centred at y −6, spreader at y −3, SO-DIMM on the −Y edge, centre supports at (0, ±21): chosen to cancel Iyz | Iyz → 0 |
| RBF, access port | (20, −47, −20) | −Y face, −Z end zone of the panel (no cells there): (±20, −50, −43) | Iyz |
| Gyro M-G370PDT0 (row 25) | Inside the OBC box | On the OBC in its socket at (−20, +30.5), 14.0 mm high, through a window in the carrier with a 2.2 g Al stiffening frame (OBC item C3) | with the carrier items below: axis 0.6° / 1.1° (from 0.2° / 1.6°) |
| Magnetometer (rows 24, 26; OBC item C13) | Module box at (+30, +30, −46) | Board A2 (RM3100) in the +Z slot at (+24, +27), board on the end frame and parts below it; Ti screws (row 24's 10 g and row 26's 2 g kept: no MEL change) | small (CG and inertia as for the slot allocation) |
| Jetson thermal interface (rows 20–23; SQSO-JET-BRD-001 §14) | Spreader plate on a TIM layer | Spreader pedestal 22 × 22 × 0.75 mm over the SoC lid and a 0.5 mm second face (1.8 g Al, row 23), gap pads 1.0 / 2.0 mm (row 22), steel module standoffs MP2/MP3 (0.8 g, row 21) | small |
| Harness | One 90 mm box | Four bundles in the ±X wall gaps at y = ±(24 … 34) mm, full height | Ixy, Iyz → 0 |
| Wings | 83 × 98 × 2.2 mm, centre (±54, 0, 0); deployed centre (±93.5, +39.5) | 81.6 × 92 mm, z −43 to +49 (moved +3 mm for the AntS tape clearance, STR-16); hinge axis (±54.1, +38.75), hinge hardware on the axis from z −49 to −37 and 43.6 to 49.6 (SQSO-MECH-001 B §3A); deployed in y = 37.1–39.4 | Iyz +0.01 × 10⁻³ |
| Distributed items (rows 33–35) | 90 mm cubes at the centre | Smeared over the stack volume (fasteners, staking) or the stack above the deck (thermal) | small |

**Principal axis.** The deployed major axis of the reference layout B is 13° (CBE) to 15° (MEV) from +Y (§6.1): because
Iyy − Izz is only 0.17 × 10⁻³ kg·m², a product Iyz of 4 × 10⁻⁵ tilts it by 13°. The CG offset in Y that the deployed
wings cause (+2.0 mm) multiplies every item that sits far from the CG in Z. The CAD cancels Iyz by placement only (no
ballast): moving the Jetson module, its clamp and the spreader to y = −6 mm, the RBF and the access port to the −Y face
and the harness into four symmetric bundles brings Iyz to −2.2 × 10⁻⁶ (CBE) and +3.8 × 10⁻⁶ kg·m² (MEV), and the axis to
0.6° and 1.1°. **The result is sensitive to placement**: moving every internal part independently by a Gaussian error
of σ per axis (2,000 trials; frame, panels, wings and AntS held) gives

| σ per axis | Mean angle CBE / MEV | 95th percentile CBE / MEV | Within 5°, CBE / MEV | Iyy/max(Ixx, Izz), 5th percentile CBE / MEV |
|------------|----------------------|---------------------------|----------------------|---------------------------------------------|
| 1 mm | 1.5° / 1.8° | 3.6° / 4.4° | 99.6 % / 97.6 % | 1.050 / 1.041 |
| 2 mm | 2.9° / 3.3° | 7.2° / 7.9° | 84.2 % / 78.6 % | 1.045 / 1.037 |
| 5 mm | 7.1° / 8.2° | 18.0° / 20.2° | 43.1 % / 38.1 % | 1.028 / 1.023 |

The ratio requirement (≥ 1.03) holds for placement errors up to about 3 mm; the 5° requirement needs the as-built
products of inertia within about ±1.5 × 10⁻⁵ kg·m², which only a drawing-controlled layout and a measured, trimmed flight
unit can give. **Proposed design provision (SQSO-MECH-CAD-001 §5.4): two 5 g trim-mass positions** at the ±Y edges of
the +Z end frame (each 5 g at y = ±45 mm, z = +49 mm moves Iyz by ∓1.1 × 10⁻⁵ kg·m², about 3° of axis tilt), set after
the mass-properties measurement; 10 g against the 272 g margin to 2.00 kg.

## 7. Mass Growth Tracking

| Date | Source | CBE | Note |
|------|--------|-----|------|
| 2026-01-02 | Version 1.0 | 1,200 g | Incomplete (Version 2.0 §1.1) |
| 2026-10-14 | ODAR §2 provisional MEL | 1,443 g | Added magnetorquers, wings, spreader |
| 2026-10-15 | BASELINE §4 Rev B | ≈ 1,510 g | Added the GNSS receiver (D6) |
| 2026-10-16 | Version 2.0 | 1,691 g | Bottom-up MEL, all custom |
| 2026-10-27 | **Version 3.0** | **1,493 g (rods) / 1,574 g (coils)** | Build-versus-buy configuration (§1.1) |
| 2026-10-27 | **Version 3.1** | **1,494 g (rods) / 1,576 g (coils)** | Row 23 +1.6 g for the spreader's SoC pedestal (§1.2) |

## 8. Mass Properties Verification

| Item | Method | Accuracy | Pass criterion | Requirement | When |
|------|--------|----------|----------------|-------------|------|
| Component masses | Weigh every item on receipt; replace CBE by the measured mass (MGA → 0) | ±0.1 g | — | MEL | Each delivery |
| Flight-unit mass | Scale | ±1 g | ≤ 2.00 kg and within the reported bounds | SYS-002, LVR-005 | Before environmental test and at L-2 months |
| CG, stowed | CG fixture, three orientations | ±1 mm | Within ±20 mm per axis | SYS-003, LVR-026 | Same |
| MOI, stowed (**VI-STR-01**) | Torsion (trifilar) pendulum about X, Y and Z, calibrated with a reference mass | **±2 % per axis** (the ±5 % of VI-LV-02 cannot verify a 3–5 % difference between Iyy and Izz) | Measured stowed values within 5 % of the CAD values | LVR-027; proposed inertia requirement (§2.1) | Same |
| Deployed inertia | Analysis: CAD with the weighed components, checked against the measured stowed inertia | — | Iyy ≥ 1.03 × max(Ixx, Izz) and major axis within 5° of +Y, at the as-built mass | Proposed (§2.1) | Drawing release and as-built |
| Products of inertia, stowed (VI-MCAD-03) | Six-orientation torsion-pendulum measurement (three axes plus three diagonal axes) or a spin balance, calibrated with a reference mass; trim masses (§6.6) set from the result | ±1 × 10⁻⁵ kg·m² | Deployed major axis (CAD with the measured stowed tensor and the weighed wings) within 5° of +Y | BASELINE §4 | Flight unit before environmental test |

## 9. Proposed BASELINE §4 Text (for the chair)

> | Mass | CBE **1.49 kg** with CR0002 rods (1.58 kg with air-core coils); MEV **1.73 kg** (1.83 kg), maturity-based growth
> 15.6 % (15.9 %) by the AIAA S-120A-2015 method (`docs/environment/MASS_BUDGET.md` v3.1). Limit **2.00 kg** (SYS-002;
> CDS Rev 14.1 Table 1); margin +15.8 % (+9.5 %). Integrator EXOpod Nova 1U limit 2.5 kg; NRCSD backup 2.40 kg.
> Lifetime analyses use 1.6 kg, with **1.85 kg** as the bound |
> | Mass properties | Stowed CG within 5 mm of the geometric centre (limit ±20 mm). Deployed inertia (wings and antennas out):
> **Iyy ≥ 1.03 × max(Ixx, Izz)**, design target 1.05, major principal axis within 5° of +Y; AntS VHF elements along ±X;
> battery at the −Z end of the stack (MASS_BUDGET §6) |

and delete the "Mass, under review" row.

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR release |
| 2.0 | 2026-10-16 | Funding audit (TB-06) | Re-issued as a bottom-up MEL with maturity-based growth allowances and one limit (2.00 kg, CDS Rev 14.1); CBE 1.69 kg, predicted 1.92–2.00 kg |
| 3.0 | 2026-10-27 | CDR structures review (STR-01, STR-02, STR-03) | MEL rebuilt for the build-versus-buy configuration (two magnetorquer options): CBE 1.49/1.57 kg, MEV 1.73/1.82 kg, margin +15.9/+9.6 %; catalogue maturity code C1/C2; CG and full inertia tensor from a reference layout; ADCS Monte Carlo re-run with the computed inertia (the published result needs Iyy/Izz ≥ 1.03); recommended stack order and AntS clocking; corrected flight inertia constant and proposed BASELINE §4 rows for the chair |
| 3.1 | 2026-10-27 | Mechanical design (SQSO-MECH-CAD-001) | CAD reconciliation: MEL masses and margins unchanged (the CAD carries every row exactly); CG and full inertia tensor from the CAD solids of layout B with AntS option A (§1, §5, §6.1 rows, §6.6): stowed CG (+0.4, −0.2, −1.8) mm CBE, deployed Iyy/max 1.056 CBE / 1.047 MEV, major axis 0.6° / 1.1° from +Y after placement balancing; placement Monte Carlo and trim-mass provision; corrected flight inertia constant (§6.5); products-of-inertia measurement added (§8); row 5 re-split for the skeleton torquer deck (MCAD-02, no MEL change, §1.2); two centre supports per board > 50 g (chair decision) noted against row 34; wing hinge hardware of SQSO-MECH-001 B; gyro in its socket through a window in the Jetson carrier (OBC item C3), Jetson carrier item M1 (row 23 raised to 66.6 g for the spreader's SoC pedestal: CBE 1,494.3 g, MEV 1,727.7 g, margin +15.8 % with rods, +9.5 % with coils; gap pads, module standoffs, flush fastener at (0, −21)); magnetometer board A2 in the +Z slot (OBC item C13; MEL row 24 unchanged); mass properties recomputed (CBE CG z −1.8 mm, axis 0.6° / 1.1°) |
