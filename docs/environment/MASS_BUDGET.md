# SMART-QSO Mass Budget

**Document ID**: SQSO-MB-001
**Version**: 3.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR issue. Bottom-up master equipment list (MEL) of the build-versus-buy configuration
(`docs/hardware/BUILD_VS_BUY.md` §1–§2) and [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) §4–§8, with mass growth,
centre of gravity and moments of inertia. **Analysis and estimates; nothing has been weighed** (no hardware exists).
Script: `software/simulation/cdr/structures/mass_properties.py`.

## 1. Purpose and Summary

This document is the controlled mass budget for the SMART-QSO 1U CubeSat: every item with its current best estimate (CBE),
its basis, a mass growth allowance (MGA) set by its maturity, the maximum expected value (MEV = CBE × (1 + MGA)), the
margin against each mass limit, the centre of gravity (CG) and the inertia tensor stowed and deployed.

| Quantity | CR0002 rods (BUILD_VS_BUY §1) | Air-core coils (BASELINE §8) |
|----------|-------------------------------|------------------------------|
| CBE | **1,492.7 g** | **1,574.1 g** |
| MGA (mass-weighted) | 15.6 % | 15.9 % |
| MEV | **1,725.9 g** | **1,824.6 g** |
| Margin to 2.00 kg (SYS-002; CDS Rev 14.1 Table 1) | **+15.9 %** (lower-growth case +22.0 %) | **+9.6 %** (lower-growth case +15.2 %) |
| Margin to the EXOpod Nova 1U limit, 2.5 kg | +44.9 % | +37.0 % |
| Margin to the NRCSD backup limit, 2.40 kg | +39.1 % | +31.5 % |
| CG, stowed (recommended layout, §5) | (−0.8, −0.7, −4.7) mm | (−1.2, −1.0, −4.5) mm |
| CG limit | ±20 mm per axis (CDS Rev 14.1 §2.2.11, Table 2) | same |

Margin as defined by AIAA S-120A: (limit − MEV)/MEV. Both options are green at CDR by the guidance quoted in §2.2
(> 5 %). The coil option is **below the 10 % trigger of decision D8** (BASELINE §9: "revisit with 2S1P if the MEL re-run
after the COTS buys still has < 10 % mass margin"); the rod option is above it.

The mass is no longer the problem it was in Version 2.0 (0–4 % margin). The new problem is the **inertia**: with the
wings and the AntS antennas deployed, +Y (the spin axis of the sun-pointing law) is only 0.5–4 % larger than +Z, and the
ADCS Monte Carlo inputs do not hold. §6 gives the numbers, the corrected values and the layout that recovers them.

### 1.1 Changes from Version 2.0

Version 2.0 (2026-10-16) was the first bottom-up MEL, for an all-custom spacecraft: CBE 1,690.7 g, MEV 1,918–2,001 g. The
build-versus-buy decisions of 2026-10-26 replaced the frame, antennas, panels, magnetometer, gyro and (optionally) the
magnetorquers with catalogue parts. Net change −198.0 g CBE with rods, −116.6 g with coils:

| Item | v2.0 CBE (g) | v3.0 CBE (g) | Δ (g) | Reason |
|------|--------------|--------------|-------|--------|
| Frame, rails, end plates, switches, brackets | 300.0 | 158.0 (rows 1–5) | −142.0 | Bought 1U frame 108–120 g (BUILD_VS_BUY §2.2) instead of a 280 g custom estimate; switches, separation springs, RBF listed separately |
| Solar array: cells, panels, hinges | 237.8 | 274.2 (rows 6–11) | +36.4 | Vendor body panels (30–50 g each with cells) instead of a bare-FR-4 estimate; wing hardware rebuilt bottom-up |
| Battery and isolated mount | 249.0 | 257.0 (rows 13–16) | +8.0 | Isolator bushings, Ti screws, disc springs, two heater elements and cutout (BUILD_VS_BUY §4.3) |
| Jetson module, carrier, interface, spreader | 245.0 | 156.0 (rows 20–23) | −89.0 | Module 31 g (distributor weight) instead of the unsourced 120 g |
| Antennas | 35.0 | 85.0 (row 19) | +50.0 | ISIS AntS 77–90 g |
| Magnetorquers | 138.9 | 57.5 rods / 138.9 coils | −81.4 / 0 | CubeSpace CR0002 16.5 g each plus bracket |
| Fasteners, spacers, staking | 25.0 | 45.0 (rows 34–35) | +20.0 | Stack spacers and adhesives counted |
| All other rows (EPS, OBC, RF, ADCS sensors, GNSS, harness, thermal) | 460.0 | 460.0 | 0 | Unchanged; still estimates |
| **Total** | **1,690.7** | **1,492.7 / 1,574.1** | **−198.0 / −116.6** | |

BUILD_VS_BUY §1 predicted −200 to −250 g; the result is −198 g with rods because the antenna (+50 g) and the vendor panels
(+36 g) offset part of the frame saving.

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
| 23 | Jetson heat spreader to the frame, Al 6061 80 × 60 × 5 mm | 1 | 65.0 | Version 2.0 §4.4 (≥ 1 W/K, BASELINE §6) | E2 | 15 % | 74.8 |
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
| | **Total, rods option (rows 1–26, 27r, 28r, 29–35)** | | **1,492.7** | | | **15.6 %** | **1,725.9** |
| | **Total, coils option (rows 1–26, 27c, 29–35)** | | **1,574.1** | | | **15.9 %** | **1,824.6** |

### 3.2 Margin

```
MASS BUDGET SUMMARY (analysis, not measured)          rods        coils
Limit (SYS-002, CDS Rev 14.1 Table 1):              2,000 g     2,000 g
CBE                                                 1,492.7 g   1,574.1 g
Growth (maturity-based, section 2.2)                  233.2 g     250.5 g
MEV                                                 1,725.9 g   1,824.6 g
Margin to 2.00 kg                                     +15.9 %      +9.6 %
```

| Growth case | Rods MEV | Rods margin to 2.00 kg | Coils MEV | Coils margin to 2.00 kg |
|-------------|----------|------------------------|-----------|-------------------------|
| §2.2 values | 1,725.9 g | +15.9 % | 1,824.6 g | +9.6 % |
| §2.2 lower-growth values | 1,639.7 g | +22.0 % | 1,735.4 g | +15.2 % |
| Uniform 15 % (BASELINE §4 method) | 1,716.6 g | +16.5 % | 1,810.2 g | +10.5 % |
| Uniform 25 % (every item treated as an estimate) | 1,865.9 g | +7.2 % | 1,967.6 g | +1.6 % |

The largest single uncertainties are the in-house boards (EPS, OBC, RF, carrier: 295 g CBE, all E1) and the vendor body
panels (160–200 g). Weighing the first engineering-model boards retires most of the E1 growth.

### 3.3 Consequences for Other Documents (for the chair)

1. BASELINE §4 mass row: replace with the text proposed in §9.
2. Decision D8 (battery): with coils the margin is 9.6 %, below the 10 % trigger. Either choose the CR0002 rods (margin
   15.9 %; the ADCS Monte Carlo must then be re-run with rod remanence, BUILD_VS_BUY §6.5) or revisit 2S1P (−125 g, margin
   rises to ≈ 17 %).
3. Orbital lifetime (BASELINE §3, ODAR): the analyses use 1.6 kg with a 1.8 kg bound. The MEV is 1.73 kg (rods) to
   1.82 kg (coils); the bound should be 1.85 kg. A heavier spacecraft decays more slowly, so the wings-failed case at
   500–525 km should be re-run in NASA DAS at 1.85 kg.
4. SQSO-LVR-001 LVR-005 and the launch ICD §4.1: the mass to report at L-8 months is 1.73 ± 0.26 kg (rods) or
   1.82 ± 0.27 kg (coils), not 1.85 kg ± 15 % from Version 2.0.

## 4. Reference Layout

No CAD model exists. The CG and the inertia use the reference layout of `docs/environment/STRUCTURAL_ANALYSIS.md` §3,
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

| Case | Layout A, rods | Layout B, rods | Layout B, coils | Limit |
|------|----------------|----------------|-----------------|-------|
| Stowed, CBE (x, y, z) | (−0.8, −0.7, −4.8) mm | (−0.8, −0.7, −4.7) mm | (−1.2, −1.0, −4.5) mm | ±20 mm |
| Stowed, MEV | (−0.7, −0.6, −4.3) mm | (−0.7, −0.6, −3.7) mm | (−1.1, −1.0, −3.6) mm | ±20 mm |
| Deployed, CBE | (−0.8, +2.2, −4.8) mm | (−0.8, +2.2, −4.7) mm | (−1.2, +1.7, −4.5) mm | — |
| One wing deployed (W1) | (+0.6, +0.8, −4.8) mm | (+0.6, +0.8, −4.7) mm | (+0.2, +0.3, −4.5) mm | — |

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
| Layout B rods CBE (2.95, 4.35, 4.14) | 1.052 | 2.5° | 9.4° | 100 % | 12° | 4.50 W |
| Layout B rods MEV (3.39, 4.94, 4.73) | 1.043 | 2.4° | 9.2° | 100 % | 11° | 4.50 W |
| Layout B coils MEV (3.68, 5.23, 5.05) | 1.035 | 2.4° | 8.9° | 100 % | 10° | 4.50 W |
| Layout B rods CBE, flight `INERTIA` changed to (3.2, 4.6, 4.4) | 1.052 | 2.5° | 9.3° | 100 % | 12° | 4.50 W |

(Layout B rows are from an earlier iteration of layout B with the rods at the −Z end; the final layout B of §6.1 moves the
rods to z = −19.5 mm and gives 1.041/1.033, between the "ratio 1.03" and "layout B" rows.)

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

Results: `/tmp/.../scratchpad/cdr/str/mc/mc_results*.csv` in the review workspace (not part of the repository); the
commands are in `software/simulation/cdr/structures/mass_properties.py` (header) and STRUCTURAL_ANALYSIS §8.

### 6.5 Corrected Values for the Flight Software and the Simulation (for the chair)

- `sqso_fsw::adcs::INERTIA`: **(0.0032, 0.0046, 0.0044) kg·m²** (layout B, rods, mean of CBE and MEV), to be replaced by
  the CAD value at drawing release and by the measured stowed inertia plus the weighed wings before flight. For coils:
  (0.0034, 0.0049, 0.0047).
- Mission simulation (`sqso-sil/src/env/attitude.rs`): use the full inertia tensor of §6.1 (products included), the
  AntS elements, the wing centre at (±93.5, +39.5, 0) mm and the MEL mass (1.73 kg MEV), and re-run the Monte Carlo
  with the products of inertia.

## 7. Mass Growth Tracking

| Date | Source | CBE | Note |
|------|--------|-----|------|
| 2026-01-02 | Version 1.0 | 1,200 g | Incomplete (Version 2.0 §1.1) |
| 2026-10-14 | ODAR §2 provisional MEL | 1,443 g | Added magnetorquers, wings, spreader |
| 2026-10-15 | BASELINE §4 Rev B | ≈ 1,510 g | Added the GNSS receiver (D6) |
| 2026-10-16 | Version 2.0 | 1,691 g | Bottom-up MEL, all custom |
| 2026-10-27 | **Version 3.0** | **1,493 g (rods) / 1,574 g (coils)** | Build-versus-buy configuration (§1.1) |

## 8. Mass Properties Verification

| Item | Method | Accuracy | Pass criterion | Requirement | When |
|------|--------|----------|----------------|-------------|------|
| Component masses | Weigh every item on receipt; replace CBE by the measured mass (MGA → 0) | ±0.1 g | — | MEL | Each delivery |
| Flight-unit mass | Scale | ±1 g | ≤ 2.00 kg and within the reported bounds | SYS-002, LVR-005 | Before environmental test and at L-2 months |
| CG, stowed | CG fixture, three orientations | ±1 mm | Within ±20 mm per axis | SYS-003, LVR-026 | Same |
| MOI, stowed (**VI-STR-01**) | Torsion (trifilar) pendulum about X, Y and Z, calibrated with a reference mass | **±2 % per axis** (the ±5 % of VI-LV-02 cannot verify a 3–5 % difference between Iyy and Izz) | Measured stowed values within 5 % of the CAD values | LVR-027; proposed inertia requirement (§2.1) | Same |
| Deployed inertia | Analysis: CAD with the weighed components, checked against the measured stowed inertia | — | Iyy ≥ 1.03 × max(Ixx, Izz) and major axis within 5° of +Y, at the as-built mass | Proposed (§2.1) | Drawing release and as-built |

## 9. Proposed BASELINE §4 Text (for the chair)

> | Mass | CBE **1.49 kg** with CR0002 rods (1.57 kg with air-core coils); MEV **1.73 kg** (1.82 kg), maturity-based growth
> 15.6 % (15.9 %) by the AIAA S-120A-2015 method (`docs/environment/MASS_BUDGET.md` v3.0). Limit **2.00 kg** (SYS-002;
> CDS Rev 14.1 Table 1); margin +15.9 % (+9.6 %). Integrator EXOpod Nova 1U limit 2.5 kg; NRCSD backup 2.40 kg.
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
