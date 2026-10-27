# Thermal Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-THM-001
**Version:** 1.5
**Date:** 2026-10-27
**Status:** CDR Baseline. The analysis is `docs/environment/THERMAL_ANALYSIS.md` (SQSO-THA-001 v2.0); this document holds
the design (finishes, mount, heater, sensors) and the allowable temperatures

---

## 1. Overview

This document defines the thermal design for the SMART-QSO 1U CubeSat for LEO mission operations.

### 1.1 Thermal Design Requirements

Allowable flight temperatures (AFT) per unit, derived from the parts' data-sheet ranges (SQSO-THA-001 2.2 §6.6, RID
THM-04; proposed SYS-600/601 text in CR-2026-017, PI decision by 2027-01-12). They replace the single SYS-600/601 range
(−20 … +50 °C operating, −40 … +60 °C survival for all components), which the boards cannot meet with the 10 °C model
margin. Rule: the protoflight test runs at the AFT ± 10 °C (ETP §3.4) and no plateau takes a part below its data-sheet
minimum, so units whose parts stop at −40 °C have a −30 °C cold AFT, operating and non-operating. The first orbit after
separation (the LEOP cold case L4 after the dispenser cold soak) extends the cold AFT to −40 °C, the parts' rating,
demonstrated by a cold start at −40 °C without the 10 °C test margin (ETP §9 deviation 8).

| Unit | Operating (°C) | Non-operating (°C) | First orbit after separation (°C) | Basis |
|------|----------------|--------------------|-----------------------------------|-------|
| OBC/EPS stack, with the ADCS and GNSS units and the magnetometer board | −30 … +50 | −30 … +60 | −40 (operating) | Parts −40 … +85 °C or wider; EPS fuses ITV4030 rated from −10 °C, qualified to −40 °C by V-EPS-8 |
| RF board | −30 … +50 | −30 … +60 | −40 (operating) | Parts −40 … +85 °C or wider (crystal Y451 in its −40 … +85 °C grade); PA gate 70 °C in software |
| Jetson carrier | −15 (power-on gate, SRS-F131) … +50 at the spreader | −30 … +60 | −40 (non-operating) | Parts −40 … +85 °C or wider |
| Jetson module | −15 (power-on gate) … +50 at the spreader; junction per SYS-603 (≤ 95 guard, 105 data sheet) | −30 … +60 | −40 (non-operating) | Data-sheet junction minimum −25 °C; below it qualified by the EM soak V-THM-8 |
| Battery | 0 … +45 charging (SYS-602); −10 … +60 discharging | −20 … +60 | as non-operating; heater-controlled | Cell specification (INR18650-35E) |
| Solar panels and wings | Vendor's qualified range, at least −56 … +93 | as operating | as operating | V-THM-4 |
| Structure | −40 … +80 | −60 … +100 | — | Unchanged |

## 2. Orbital Environment

### 2.1 Orbital Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 475 km target, 465–485 km, not above 485 km (BASELINE Rev E §3); the thermal cases were run at 500 km, where the eclipse is ≈ 0.3 % of the orbit shorter |
| Orbit | Sun-synchronous, 97.4°, LTAN 22:30 ± 1 h |
| Beta angle range | 2.4° … 40.9° over a year across the LTAN dispersion (never eclipse-free) |
| Eclipse duration | 32–36 minutes |
| Orbit period | 94.6 minutes |
| Mission duration | 2 years |

### 2.2 Environmental Heat Loads

| Source | Value | Notes |
|--------|-------|-------|
| Solar flux | 1322–1414 W/m² | Aphelion to perihelion |
| Earth albedo | 0.30 (avg); 0.25 cold case, 0.35 hot case | SQSO-THA-001 §2.2 |
| Earth IR | 237 W/m² (avg); 218 cold, 258 hot | SQSO-THA-001 §2.2 |

---

## 3. Surface Properties

### 3.1 External Surfaces

| Surface | Finish | α (solar) | ε (IR) | α/ε |
|---------|--------|-----------|--------|-----|
| ±X, ±Y long faces | Two cells (53 % of the face, α 0.92, ε 0.85) and black anodize between | 0.90 | 0.85 | 1.06 |
| ±Z end faces | Black anodize | 0.88 | 0.85 | 1.04 |
| Wings, both sides | Cells (74 %) and substrate | 0.91 | 0.85 | 1.07 |

Version 1.2 had cells on ±X only; the array has two cells on each long face and four on each wing
(SOLAR_ARRAY_DESIGN §3.1), as the simulation's network uses.

### 3.2 Internal Surfaces

| Surface | Finish | ε (IR) |
|---------|--------|--------|
| PCB | FR4 green | 0.90 |
| Components | Various | 0.85 |
| Structure | Bare Al | 0.05 |

---

## 4. Thermal Model

### 4.1 Node Definition

The model is the mission simulation's 12-node network (`software/rust/sqso-sil/src/env/thermal.rs`), with heat
capacities from the mass budget (version 1.2's table, with 15 J/K for the OBC and 120 J/K for the battery, did not
match the masses):

| Node | Component | Capacitance (J/K) |
|------|-----------|-------------------|
| 1–4 | ±X, ±Y body panels with their share of the frame | 76 each |
| 5–6 | ±Z end panels | 30 each |
| 7–8 | Wings W1, W2 | 42 each |
| 9 | Battery pack | 250 |
| 10 | OBC/EPS/ADCS/GNSS stack | 200 |
| 11 | RF board with PA | 130 |
| 12 | Jetson module with spreader | 220 |

### 4.2 Conductance Network

| From | To | Conductance (W/K) | Path |
|------|-----|-------------------|------|
| OBC | Structure | 0.5 | Standoffs |
| Battery | Structure | 0.05 (model, conservative); 0.03 design | Isolating bushings (§7.2); plus radiation ≈ 0.008 W/K with the ε ≤ 0.10 exterior (0.07 W/K if black); was 1.0 W/K through a thermal pad |
| RF | Structure | 0.3 | Standoffs |
| Jetson | Structure | ≥ 1.0 required (BASELINE §6, SYS-603); 0.44 audit estimate of the documented stack, used by the model | Heatspreader |
| ADCS | Structure | 0.3 | Standoffs |

---

## 5. Thermal Analysis Cases

### 5.1 Hot Case

BASELINE §8: β at maximum with no eclipse, maximum power (ACTIVE, PA at 1 W RF keyed 25 % of sunlight, Jetson 8 W
bursts), heater off, 1414 W/m², albedo 0.35, IR 258 W/m². The baseline orbit never loses its eclipse (β ≤ 41°), so
this is a bound; the hottest real orbit (LTAN 21:30, β 40°, January) is analysed as well.

### 5.2 Cold Case

BASELINE §8: β = 0 (36 min eclipse), SAFE mode (ADCS off), 1322 W/m², albedo 0.25, IR 218 W/m², heater thermostatic at
the minimum bus voltage. Also: EMERGENCY, wings stowed, heater failed, a 0.10 W/K mount, a black pack, the dispenser
cold and hot soaks and the deployment that follows.

### 5.3 Analysis Results

From SQSO-THA-001 v2.0 §4–§5 (12-node model, uncorrelated; 10 °C uncertainty margin applies):

| Component | Hot bounding (°C) | Cold (°C) | Margin after 10 °C (worst) |
|-----------|-------------------|-----------|----------------------------|
| OBC/EPS stack | +31.9 | −11.6 (−17.4 after a 2 h cold soak) | +8.1 hot; **−1.6 / −7.4** cold against SYS-600 |
| Battery | +28.9 | +1.3 at 0.72 W, 43 % duty; +4.0 with set points +5/+9 °C | +6.1 hot; heater duty below 70 % |
| RF | +32.2 | −11.8 (−18.6 wings stowed) | +7.8 hot; −1.8 / −8.6 cold |
| Jetson (burst, 600 s batch) | +39.0 | −12.0 (unpowered) | +1.0 against SYS-600, +21 against the 70 °C guard |
| Structure, body panels | +39.0 (+51.5 with a stuck-on Jetson) | −22.7 | +7.3 cold |
| Wings | +75.2 (+82.9 at LTAN 21:30) | −45.4 (LEOP after a cold soak) | −15.4 / −7.9 against −40/+85 (qualification needed) |

## 6. Thermal Control Hardware

### 6.1 Passive Thermal Control

| Element | Location | Purpose |
|---------|----------|---------|
| Black anodize | External | Radiator |
| Thermal pads | Battery, Jetson | Conduction |
| Heatspreader | Jetson | Heat spreading |
| MLI | Not used | N/A for 1U |

### 6.2 Active Thermal Control

| Element | Location | Power | Activation |
|---------|----------|-------|------------|
| Battery heater | Battery pack | ≈ 1 W at 14.4 V (≈ 200 Ω, two elements): 0.72 W at 12.0 V, 0.95 W at 14.0 V, 1.30 W at 16.4 V | Flight software: on below +2 °C, off above +6 °C (`sqso_fsw::heater`). **Recommended +5/+9 °C** (SQSO-THA-001 §6.1, RID THM-02); hardware over-temperature cutout at +40 °C; fuse 0.25 A |
| (Jetson heater) | Heatspreader | (0.5W) | (Optional) |

### 6.3 Heater Sizing

```
Battery heater (1.2; docs/hardware/BUILD_VS_BUY.md §4):
  G_total ≈ 0.03 W/K to the frame (bushings and Ti screws 0.015,
           flying leads 0.008, low-emissivity exterior 0.005-0.01)
  Q = G × ΔT: 0.81 W at a -25 °C frame, 0.51 W at -15 °C
  Heater power: about 1 W at 14.4 V (margin for an as-built G of
  0.10 W/K, which the mission simulation shows 0.5 W cannot hold)
  Thermostat setpoint: +2°C (on), +6°C (off): the design's 0/+5 °C
  with 2 °C of margin over the 0 °C charge limit (SYS-602) for the
  sensor and the gradient across the pack
```

The sizing above assumes the pack is isolated from the frame (about 0.03 W/K). §4.2 and §7.2 had put it on a
1.0 W/K thermal pad instead; with the pad the mission simulation (`docs/test/SIL_MISSION_SIMULATION.md`) kept the
pack below 0 °C for a fifth of every sun-pointed orbit with the heater on, so charging was refused each time the Sun
returned. The pack is now on isolating standoffs, 0.05 W/K to the frame and 0.02 W/K to the OBC stack: its own I²R
heat (≤ 0.1 W) raises it about 2 K above the frame in the hot case, and the heater holds it above +2 °C in eclipse.

---

## 7. Thermal Interfaces

### 7.1 Jetson Thermal Stack

| Layer | Material | Thickness | Conductivity |
|-------|----------|-----------|--------------|
| Jetson die | Silicon | - | - |
| TIM 1 | Fujipoly | 0.5 mm | 3 W/m·K |
| Heatspreader | Al 6061 | 3 mm | 167 W/m·K |
| TIM 2 | Gap pad | 1 mm | 2 W/m·K |
| Structure | Al 7075 | 3 mm | 130 W/m·K |

### 7.2 Battery Thermal Interface

| Layer | Material | Thickness | Conductivity |
|-------|----------|-----------|--------------|
| Cell case | Aluminum | 0.3 mm | 200 W/m·K |
| Thermal pad | Bergquist | 1 mm | 3 W/m·K |
| Bracket | Al 6061 | 2 mm | 167 W/m·K |
| Standoffs to the frame | G-10CR or PEEK 450G shouldered bushings, Ti-6Al-4V screws (BUILD_VS_BUY §4.3; confirmed by the EM thermal balance, V-THM-2) | 4 × M2.5, 8 mm | G-10CR 0.56–0.82 W/m·K, PEEK 0.29 W/m·K; the bushings with their screws ≈ 0.015 W/K, and the whole mount ≈ 0.03 W/K with the harness and radiation (BUILD_VS_BUY §4.2). The screws, the harness and the surface finish set the value, not the polymer |

The pad and bracket spread heat across the cells and from the heater; the standoffs isolate the bracket from the
frame (§6.3).

---

## 8. Thermal Straps (If Required)

### 8.1 Strap Specification

| Parameter | Value |
|-----------|-------|
| Material | Copper braid |
| Cross-section | 5 mm² |
| Conductance | 0.5 W/K per 100mm |
| Application | Jetson to radiator (optional) |

---

## 9. Thermal Testing Requirements

### 9.1 Thermal Balance Test

| Parameter | Requirement |
|-----------|-------------|
| Environment | Thermal vacuum chamber |
| Temperature range | -30°C to +60°C (shroud) |
| Vacuum level | < 10⁻⁵ Torr |
| Duration | Until steady state (2+ hours) |
| Correlation | ±3 °C on the battery, OBC, RF and Jetson nodes (SQSO-THA-001 V-THM-1); after correlation the margin is 5 °C (GSFC-STD-7000B §2.6.4.1) |
| Cases | Hot and cold of §5, heater power stepped, wings deployed and stowed |

### 9.2 Thermal Cycling Test

SpaceX UG v11 Table 6-2 (advised): protoflight = the acceptance range (envelope of the maximum and minimum predicted
temperatures and −24 … +61 °C) ± 5 °C, 20 cycles, at least 4 in vacuum. The battery's −20 … +60 °C limits are
inside that range: the test is tailored at the battery with the integrator's agreement (SQSO-THA-001 V-THM-6).

| Parameter | Requirement |
|-----------|-------------|
| Hot plateau | +66 °C (tailored to +55 °C at the battery) |
| Cold plateau | −29 °C (tailored to −20 °C at the battery, no charging below 0 °C) |
| Dwell time | 2 hours minimum |
| Cycles | 20 (≥ 4 in vacuum) |
| Functional test | Each plateau |

---

## 10. Thermal Telemetry

### 10.1 Temperature Sensors

| Location | Sensor | Range | Accuracy |
|----------|--------|-------|----------|
| OBC board | NTC thermistor | -40 to +85°C | ±1°C |
| Battery 1 | NTC thermistor | -40 to +85°C | ±1°C |
| Battery 2 | NTC thermistor | -40 to +85°C | ±1°C |
| Jetson | Internal sensor | 0 to +100°C | ±2°C |
| Structure | NTC thermistor | -40 to +85°C | ±1°C |

### 10.2 Heater Telemetry

| Point | Type |
|-------|------|
| Heater status | On/Off |
| Heater current | mA |
| Thermostat state | Open/Closed |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-26 | Battery mount isolated (§4.2, §6.3, §7.2) and heater set points +2/+6 °C (§6.2, §6.3), from the mission simulation |
| 1.2 | 2026-10-26 | Battery heater 1 W and the mount conductance network (§6.2, §6.3, §7.2; BUILD_VS_BUY §4) |
| 1.3 | 2026-10-27 | CDR review (thermal): allowable temperatures with proposed per-unit values (§1.1); baseline orbit and environments (§2); conductances (§4.2); cases and results from SQSO-THA-001 v2.0 (§5); heater set points recommended (§6.2); correlation and SpaceX thermal-cycling levels (§9) |
| 1.4 | 2026-10-27 | CDR 2026 consistency pass: altitude row from BASELINE Rev E §3 (§2.1) |
| 1.5 | 2026-10-27 | CDR round 3, thermal (RID THM-04, chair disposition): §1.1 per-unit allowable flight temperatures derived from the parts (SQSO-THA-001 2.2 §6.6): −30 °C cold for every board, operating and non-operating, −40 °C for the first orbit after separation; battery and Jetson per the proposed SYS-600/601 (CR-2026-017) |
