# Solar Array Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-SA-001
**Version:** 2.3
**Date:** 2026-10-27
**Status:** Controlled array layout and power-versus-attitude table for the baseline in `docs/BASELINE_PARAMETERS.md` §5 (finding TB-14). **Analysis, not measured.** Other documents that state array power should cite §3 and §9 of this document.

---

## 1. Overview

This document defines the solar array layout of the SMART-QSO 1U CubeSat and the power it delivers in each attitude.

Version 1.0 summed the peak power of all 16 cells (19.4 W) as if every face could face the sun at once. Other documents
claimed 4.8 W from two 100 × 50 mm panels with "2S4P per panel" (8 cells on 50 cm², where one 40 × 80 mm cell alone
is 30 cm²; 50 cm² of 30 % cells can deliver at most 1361 W/m² × 0.0050 m² × 0.30 = 2.0 W) and a 0.06 m² array on a
1U. Those figures are withdrawn; §3 and §9 replace them.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| EOL orbit-average generation | 2.39 W tumbling, 3.58 W sun-pointed (audit estimate) | BASELINE §5; reproduced and qualified in §9 |
| Array | 16 triple-junction cells: 2 on each of the 4 long body faces, 2 per side on 2 deployable 1U-face wings | BASELINE §5 |
| Battery bus | 4S Li-ion, 14.4 V nominal, 16.4 V at the 4.10 V/cell charge set point (D8), 16.8 V absolute maximum | BASELINE §5; `hardware/eps/BATTERY_DESIGN.md` |
| Operating temperature | -40°C to +85°C | Cell qualification range to be confirmed with the supplier |
| Deployment | Body-mounted plus two deployable wings, ≥ 30 min after deployment-switch release | BASELINE §4 |
| Design life | 2 years on orbit | BASELINE §2 |

---

## 2. Solar Cell Selection

### 2.1 Cell Specification

AZUR SPACE 3G30-Advanced, 4 × 8 cm, as a cell-interconnect-coverglass assembly (3G30A, part 81442), from datasheet
DB 00010891-01 (issued 2025-04-11). Electrical values at AM0, 1367 W/m², 28 °C; radiation columns are 1-MeV electron
fluence in e/cm².

| Parameter | BOL | 5E13 | 1E15 |
|-----------|-----|------|------|
| Voc | 2,700 mV | 2,633 mV | 2,519 mV |
| Isc | 515 mA | 513 mA | 496 mA |
| Vmp | 2,411 mV | 2,336 mV | 2,230 mV |
| Imp | 499 mA | 496 mA | 480 mA |
| Pmp (Vmp × Imp) | 1.203 W | 1.159 W | 1.070 W |
| Efficiency | 29.2 % | 28.1 % | 25.9 % |

| Parameter | Value |
|-----------|-------|
| Cell area | 30.18 cm² (40.0 × 80.0 mm); glassed area 31.13 cm² |
| Mass | ≤ 3.6 g per assembly |
| Coverglass | 100 µm, AR coated |
| Bypass diode | Corner-mounted silicon diode on each assembly |
| Temperature coefficients (BOL) | ΔVmp/ΔT = −6.7 mV/°C; ΔImp/ΔT = +0.24 mA/°C |
| Temperature coefficients (1E15) | ΔVmp/ΔT = −7.2 mV/°C; ΔImp/ΔT = +0.28 mA/°C |

A Spectrolab cell of the same size may be substituted after this analysis is repeated with its data.

### 2.2 Procurement Specification

- Supplier qualification: the datasheet states ECSS-E-ST-20-08C (Rev. 1) qualification of the 3G30-Advanced cells and assemblies; obtain the qualification report before procurement
- Lot acceptance testing required
- ESD-sensitive handling per ANSI/ESD S20.20
- Inspection per IPC-A-610 Class 3 Space Addendum

---

## 3. Array Layout and String Configuration

### 3.1 Axes and Layout

Body axes: **+Z** along the 113.5 mm rail direction; ±X and ±Y are the four long faces (100 × 113.5 mm). No cells are on
the ±Z end faces, which carry the deployment switches, separation springs and access ports.

```
DEPLOYED, viewed along −Z (looking down the long axis)

        W2 front (2 cells, +Y)   +Y body (2 cells)   W1 front (2 cells, +Y)
   ─────────────────────────┬────────────────────┬─────────────────────────   plane of the +Y face
        W2 back  (2 cells, −Y)│                    │ W1 back  (2 cells, −Y)
                    −X body │                    │ +X body
                   (2 cells)│        BODY        │(2 cells)
                            │                    │
                            └────────────────────┘
                                −Y body (2 cells)

STOWED: W1 lies over the +X face and W2 over the −X face, front side outward.
```

| Panel | Cells | Faces (deployed) | Faces (stowed) |
|-------|-------|------------------|----------------|
| +Y body | 2 | +Y | +Y |
| −Y body | 2 | −Y | −Y |
| +X body | 2 | +X | Covered by W1 |
| −X body | 2 | −X | Covered by W2 |
| Wing W1 (hinged on the +X/+Y edge, parallel to Z) | 2 front + 2 back | Front +Y, back −Y | Front +X, back against the body |
| Wing W2 (hinged on the −X/+Y edge, parallel to Z) | 2 front + 2 back | Front +Y, back −Y | Front −X, back against the body |

Deployed, the two wings swing 90° outward into the plane of the +Y face. The +Y side then presents **6 cells** in one
plane (the sun-pointing face), the −Y side another 6, and ±X two each.

This layout is this document's choice: it is the one arrangement of the BASELINE §5 cell count (2 per long face, 2 per
wing side) that gives the 6 coplanar cells the BASELINE sun-pointed generation assumes. Each panel is taken as
83 × 98 mm, to fit between the rails and, stowed, within the CDS deployable envelope **[assumption; to be confirmed by
the mechanical design]**. A wing of one 1U face is also what the BASELINE §3 drag area (0.028 m², wings deployed)
assumes. Antenna locations must be checked against the wings in the mechanical layout.

### 3.2 Strings

| Parameter | Value |
|-----------|-------|
| String | 2 cells in series, one string per panel side (8 strings) |
| String Vmp | 4.82 V at BOL and 28 °C; 4.24 V at 5E13 and 60 °C (2 × 2,121.6 mV, §9.1) |
| String Imp | 0.50 A |
| String protection | Blocking diode (or ideal-diode switch) per string; bypass diode per cell (on the assembly) |
| Conversion | The string voltage (3.9 V hot EOL to 6.3 V open-circuit at a −45 °C sunrise) is below the 11.2–16.4 V bus: boost trackers, four of them (wing fronts; +Y body; wing backs; ±X and −Y bodies), because a hot wing string and a cooler body string on one tracker lose 2.7–3.9 % (`docs/hardware/EPS_DESIGN.md` v2.0 §3) |

### 3.3 Total Array Summary

| Quantity | Value | Basis |
|----------|-------|-------|
| Cells | 16 | §3.1 |
| Pmp of one cell at normal incidence, BOL, 28 °C, 1367 W/m² | 1.203 W | §2.1 |
| Sum of all 16 cells | 19.2 W | **Not achievable:** at most 6 cells face the sun at once |
| Maximum instantaneous, BOL, 28 °C (6 cells normal to the sun) | 7.2 W | 6 × 1.203 W |
| Maximum instantaneous, EOL, 60 °C, delivered (6 cells) | 5.76 W | 6 × 0.961 W (§9.1) |
| Orbit average | §9.3 | Depends on attitude |

---

## 4. Interconnect Design

Sections 4–8 and 10 carry Version 1.0 design values that have no source yet, except where a basis is given.

### 4.1 Cell Interconnects

| Parameter | Specification |
|-----------|---------------|
| Material | Silver-coated Kovar (supplied on the 3G30A assembly) |
| Method | Welded (preferred) or high-rel solder |
| Redundancy | Dual-tab interconnect |
| Workmanship | Per the project's applicable-documents list (NASA-STD-8739.3 is cancelled; finding VV-11) |

### 4.2 String Wiring

- **28 AWG M22759/33** (ETFE, 200 °C) wire, **two wires per pole** on the EPS's Pico-Clasp headers (the Pico-Clasp terminal takes 28–32 AWG only): a string's Isc reaches 0.55 A (1414 W/m², hot), 0.28 A per wire, within the EEE-INST-002 bundle-derated 0.90 A of SQSO-HAR-001 2.0 §5.2 (its §9.1 gives 37 % at the 0.33 A worst wire); harnesses W2PX–W2MY and W2W1, W2W2 (SQSO-HAR-001 §4), gold contacts
- Strain relief at all connection points
- Harness routing per ICD

---

## 5. Coverglass Design

### 5.1 Coverglass Specification

| Parameter | Value |
|-----------|-------|
| Material | Ceria-doped borosilicate (CMG) |
| Thickness | 100 µm (3G30A assembly) |
| AR Coating | MgF₂ dual-layer |
| UV Cut-off | 350nm |
| Bonding | DC93-500 or equivalent |

### 5.2 Protection Provided

- Proton/electron shielding
- UV degradation protection
- Micrometeoroid protection (limited)
- Electrostatic discharge path

---

## 6. Bypass Diodes

### 6.1 Configuration

- One silicon bypass diode per cell, corner-mounted on the 3G30A assembly
- Forward voltage ≤ 0.8 V at 620 mA (datasheet)

### 6.2 Placement

- On the cell assembly, under the coverglass edge
- Thermal path to substrate

---

## 7. Thermal Design

### 7.1 Operating Range

| Condition | Temperature | Duration |
|-----------|-------------|----------|
| Sunlit peak | +85°C max | Continuous |
| Predicted, wings (SQSO-THA-001 v2.0 §4) | −45 … +83 °C (cold-soak LEOP to the hottest real orbit, LTAN 21:30) | The panel vendor's qualification must cover −56 … +93 °C (10 °C model margin; RID THM-05) |
| Predicted, body panels | −23 … +52 °C | |
| Used in §9 | +60 °C | Conservative against the predictions: the mission simulation, with the thermal model's temperatures, gives about 10 % more (§9.5) |
| Eclipse | -40°C min | Up to 36.1 min per orbit (β = 2.4°, LTAN 23:30) |
| Thermal cycling | ≈ 5,590 cycles/yr (one per orbit at 475 km) | 2 years |

### 7.2 Thermal Expansion

- CTE matching between cell, coverglass, and substrate
- Stress-relief interconnects accommodate expansion
- Substrate: FR4 or aluminum-core PCB

---

## 8. Deployment Mechanism Interface

### 8.1 Hinge Design

| Parameter | Specification |
|-----------|---------------|
| Type | Spring-loaded hinge, axis parallel to Z on the ±X/+Y edges |
| Material | Stainless steel / aluminum |
| Deployment Angle | 90° ± 2° (into the plane of the +Y face) |
| Deployment Time | < 5 seconds |
| Actuation | Burn wire / thermal knife, not before 30 min after deployment-switch release (BASELINE §4) |

### 8.2 Stowed Configuration

- Panels secured with nylon restraint
- Burn wire redundancy (2 circuits)
- Deployment switches for status telemetry
- Three independent physical inhibits per deployable (BASELINE §4)

---

## 9. Power Versus Attitude

All values are analyses, not measurements.

### 9.1 Power of One Cell (EOL, Delivered)

| Step | Value | Basis |
|------|-------|-------|
| 3G30A at 5E13 e/cm², 28 °C, 1367 W/m² | 2,336 mV × 496 mA = 1.159 W | §2.1. The 2-year 1-MeV equivalent fluence at ≤ 525 km SSO behind 100 µm coverglass is taken as 5E13 **[assumption; replace with a mission radiation analysis]** |
| At 60 °C cell temperature | (2,336 − 6.7 × 32) mV × (496 + 0.24 × 32) mA = 2,121.6 mV × 503.7 mA = 1.069 W | §2.1 coefficients; §7.1 temperature assumption |
| At 1361 W/m² (mean solar irradiance at 1 AU) | × 1361/1367 → 1.064 W | ±3.3 % over the year |
| Harness, string diode and mismatch | × 0.97 | **[assumption]** |
| MPPT conversion | × 0.95 | **[assumption]** |
| UV darkening and contamination | × 0.98 | **[assumption]** |
| **Delivered power per cell at normal incidence, P₁** | **0.961 W** | 1.064 × 0.903 |

Sensitivity: at the datasheet's 1E15 column and 60 °C, the cell gives (2,230 − 7.2 × 32) mV × (480 + 0.28 × 32) mA =
0.978 W before the factors above, 8.5 % less than at 5E13.

### 9.2 Method

Orbit-average power = N × P₁ × f_sun, where:

- **N** (cell-equivalents) is the sum over all cells of cos⁺(incidence angle) × the unshadowed fraction of the cell,
  averaged over the attitude motion.
- **f_sun** = 0.622, the sunlit fraction of a 500 km circular orbit at β = 0: eclipse half-angle
  acos(√(h² + 2Rₑh)/(Rₑ + h)) = 68.0°, so the eclipse is 68.0/180 = 37.8 % of the 94.6 min orbit (35.8 min). β = 0 gives
  the longest eclipse; higher β gives more sun. The orbit's local time is not yet known (launch contract).

For a uniformly random (tumbling) attitude, each one-sided flat cell averages cos⁺ = 1/4, so N = 16/4 = **4.0** without
shadowing. For a spin about Z with the sun perpendicular to Z, each cell on a face parallel to Z averages 1/π, so
N = 16/π = **5.09** without shadowing.

**Self-shadowing.** In the deployed layout only two groups of cells can be shadowed: the ±X body cells (4 cells,
shadowed by the wings only when the sun is on the +Y side) and the wing back sides (4 cells, shadowed by the body only
when the sun is on the body's side of the wing). Each such cell keeps at least half of its tumbling average, so the
tumbling N lies between 8/4 + 8/8 = **3.0** and **4.0**. A numerical ray-cast of the §3.1 geometry (3,000 sun directions
evenly spread over the sphere, 32 sample points per cell, rays tested against the body and both wings) gives
**N = 3.57** for tumbling (a factor of 0.89) and 4.36 for the spin about Z **[audit numerical estimate; to be replaced by
the controlled power and attitude simulation, TB-02 and TB-09]**.

### 9.3 Power-Versus-Attitude Table

EOL, 500 km, β = 0, P₁ = 0.961 W, f_sun = 0.622 (P₁ × f_sun = 0.598 W per cell-equivalent):

| Attitude | N, no shadowing | N, with shadowing | Orbit average, no shadowing | **Orbit average, with shadowing** | Sunlit instantaneous (with shadowing) |
|----------|-----------------|-------------------|-----------------------------|-----------------------------------|----------------------------------------|
| Tumbling (uniform random) | 4.00 | 3.57 | 2.39 W | **2.14 W** | 3.43 W |
| Spin about Z, sun perpendicular to Z | 5.09 | 4.36 | 3.04 W | **2.61 W** | 4.19 W |
| Sun-pointed ±Y face, no error | 6.00 | 6.00 | 3.59 W | **3.59 W** | 5.76 W |
| Sun-pointed ±Y face, 10° coning error | — | 5.92 | — | **3.54 W** | 5.69 W |
| Sun-pointed ±Y face, 20° coning error | — | 5.66 | — | **3.38 W** | 5.44 W |
| Sun-pointed ±Y face, 30° coning error | — | 5.26 | — | **3.14 W** | 5.05 W |
| **Worst case:** sun held along ±X | 2.00 | 2.00 | 1.20 W | **1.20 W** | 1.92 W |
| **Worst case:** sun held along ±Z | 0 | 0 | 0 W | **0 W** | 0 W |
| **Worst case:** both wings stowed, tumbling (8 exposed cells, no shadowing) | 2.00 | 2.00 | 1.20 W | **1.20 W** | 1.92 W |

The "no shadowing" column reproduces BASELINE §5 (2.39 W tumbling, 3.58 W sun-pointed) with the §9.1 factors.

### 9.4 Comparison with the BASELINE §5 Loads

| Case | Generation | Load (BASELINE §5) | Margin | Requirement ≥ 20 % |
|------|------------|--------------------|--------|--------------------|
| Tumbling, no shadowing (BASELINE method) | 2.39 W | 1.80 W | +33 % | Met |
| **Tumbling, with shadowing** | **2.14 W** | 1.80 W | **+19 %** | **Not met** |
| Sun-pointed, no error | 3.59 W | 2.20 W | +63 % | Met |
| Sun-pointed, 20° error | 3.38 W | 2.20 W | +54 % | Met |
| Both wings stowed, tumbling | 1.20 W | 1.80 W | −34 % | Not met |
| Both wings stowed, transponder and Jetson off | 1.20 W | 1.00 W (housekeeping 0.70, GNSS 0.05, receiver 0.15, beacon 0.10) | +20 % | Met (at the limit) |

### 9.5 Simulated Generation (CDR)

The mission simulation (`software/rust/sqso-sil`, same cell data, shadowing and degradation, with the thermal model's
cell temperatures and albedo) gives, at end of life delivered to the bus, at 475 km from 2028-11-01, worst 30 days of
the second year, LTAN 22:30 ± 1 h: sun-pointed **3.84 W** (3.79 W in the worst ADCS Monte Carlo trial with the face
photodiodes and albedo, decision C-23); free tumble (ADCS failed) **2.48 W**; wings stowed 1.47 W (BASELINE §5;
`docs/environment/POWER_BUDGET_ANALYSIS.md` v2.5 §4; v2.0 gave 3.93 / 2.43 W). The §9.3
values are bounds: their 60 °C cell temperature is about 10 % pessimistic for the body cells and they omit albedo
(≈ 5 %). With the simulated values the tumbling margin is +33 % against the 1.86 W allocation (BASELINE §5), and
finding 1 below is closed (POWER_BUDGET_ANALYSIS §9).

Findings for the power budget (TB-02), reported rather than changed here:

1. With self-shadowing the tumbling margin falls below the 20 % requirement. Either the tumbling Jetson and transponder
   duty cycles come down, or the spacecraft must spend enough time sun-pointed.
2. A spin axis along Z pointed at the sun gives no power. The sun-pointing mode must point a ±Y face (spin about Y), and
   the ADCS must detect and leave a Z-to-sun attitude. With the battery at 50 Wh and a 1.0 W housekeeping load, the time
   from full charge to the 25 % SAFE threshold is (50 × 0.75)/1.0 ≈ 37 h.
3. If both wings fail to deploy, only beacon, receiver and housekeeping are sustainable.

---

## 10. Testing Requirements

### 10.1 Cell-Level Testing

- Flash test (I-V curve)
- Visual inspection
- ESD sensitivity verification

### 10.2 Panel-Level Testing

| Test | Requirement |
|------|-------------|
| I-V Characterization | Per AIAA S-111 |
| Thermal Cycling | -40°C to +85°C, 10 cycles |
| Deployment Test | 10 cycles minimum |
| Illumination Test | AM0 equivalent |
| Shadowing check | Illuminated EM at the §9.3 attitudes, to confirm the ray-cast factor |

---

## 11. Traceability

| Requirement | Design Element | Verification |
|-------------|----------------|--------------|
| BASELINE §5 array definition | 16 cells, §3.1 layout | Inspection |
| BASELINE §5 generation (2.39 / 3.58 W) | §9.3 table | Analysis now; simulation (TB-02) and EM illumination test later |
| BASELINE §4 deployables and inhibits | §8 | Inspection, test |
| ENV-THERMAL-001 | Supplier qualification report (to be obtained) | Inspection, test |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 2.0 | 2026-10-16 | Funding audit (TB-14): one array layout (§3.1), cell data from the AZUR SPACE datasheet, power-versus-attitude table with the calculation shown (§9); the 19.4 W all-cells total withdrawn |
| 2.1 | 2026-10-27 | CDR review (EPS): bus at the 4.10 V/cell set point (§1.1); four trackers (§3.2); 24 AWG string wire (§4.2); predicted panel temperatures (§7.1); simulated generation and the tumbling margin closed (§9.5) |
| 2.2 | 2026-10-27 | CDR design (EPS board): §4.2 string wire per SQSO-HAR-001 2.0 (S-1): 28 AWG M22759/33, two wires per pole (was 24 AWG) |
| 2.3 | 2026-10-27 | Documentation audit (post-CDR 2026): §9.5 simulated generation to BASELINE §5 (2.48 W tumbling, 3.84 W sun-pointed at 475 km, PBA v2.5; was 2.43 / 3.93 W from PBA v2.0) and the tumbling margin +33 % against 1.86 W; §5 thermal cycles at 475 km. The §9.3 bound at 500 km is kept as the bound BASELINE §5 cites |
