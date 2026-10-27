# SMART-QSO Battery Safety Assessment

**Document ID**: SQSO-BSA-001
**Version**: A
**Date**: 2026-10-27
**Status**: CDR assessment. No cell has been bought or tested; every control below is a design requirement until its
verification item (§9) is closed.

The pack is the one stored-energy hazard on SMART-QSO (no propulsion, pyrotechnics or pressure vessel, SYS-082). This
document identifies its hazards, the controls against each, the fault tolerance each control set gives, and what
the SpaceX rideshare path (the baseline, decision D7) and the ISS/NRCSD backup (JSC 20793 Rev D) require. It
governs the battery-safety content of `hardware/eps/BATTERY_DESIGN.md` and `docs/hardware/EPS_DESIGN.md`.

## 1. The battery

| Parameter | Value | Source |
|-----------|-------|--------|
| Cells | 4 × Samsung SDI INR18650-35E in series (4S1P), one lot, one date code | BASELINE §5; BATTERY_DESIGN §2 |
| Energy | 49.7 Wh typical (48.2 Wh at minimum capacity) | BATTERY_DESIGN §3.1 |
| Voltage | 16.4 V at the 4.10 V/cell charge set point (D8); 16.8 V at the cell's 4.20 V maximum; 12.0 V operational floor | BASELINE §5 |
| Charge | ≤ 0.40 A CC (array-limited, ≈ C/9), CV 4.10 V/cell, 0 … +45 °C only | BASELINE §5; SYS-602 |
| Discharge | ≤ 2.0 A peak (1.6 A burn plus housekeeping), ≈ 0.1 A in eclipse; cell continuous limit 8 A | POWER_BUDGET_ANALYSIS §5 |
| Thresholds | 80 Wh (JSC 20793 §5.1.5.1a propagation test), 100 Wh (CDS 2.1.5, FAA carry-on) | — |

The pack is below both energy thresholds: no cell-to-cell propagation test is required by JSC 20793 Rev D §5.1.5.1a.

## 2. Governing requirements

| Source | Requirement used here |
|--------|-----------------------|
| CDS Rev 14.1 §2.3.1, §2.3.1.2, §2.3.6 | Power-off from delivery to deployment; powered-on battery protection permitted; protection for charging and discharging against unbalanced cells, documentation or test for customised cells |
| SpaceX Rideshare Payload User's Guide v11 (Aug 2026) §6.7.7, §7.6, §11.1–11.2 | Inhibits verified during random vibration with the batteries in their flight charge state; all payload systems including batteries powered off during launch except inhibit-sensing circuits; SSCMAN 91-710 compliance through SpaceX's safety process (batteries listed as potentially hazardous systems). The guide defines no battery failure-tolerance level of its own; SSCMAN 91-710 Volume 3 and the integrator's ICD do (paragraph not verified here; the SSCMAN text could not be reached) |
| JSC 20793 Rev D (March 2017) §4.1.1 | Failure tolerance as the preferred control for every catastrophic hazard cause; level set by the integrated safety analysis, at least one. The ISS payload safety process (SSP 51700, as invoked by the NRCSD provider) makes catastrophic hazards two-fault tolerant |
| JSC 20793 Rev D §4.2.3, §6.4.3.3 | 100 % cell screening (visual with wrap removed, mass, OCV retention, AC and DC resistance); battery acceptance (functional baseline, vibration, vacuum or thermal vacuum, recheck); ≥ 10 cycles; ±3σ screening with lot uniformity OCV < 1 %, mass < 2 %, capacity < 5 %, resistance < 15 %, lot rejected if > 15 % fail |
| JSC 20793 Rev D §4.3.2, §4.4.3–§4.4.5, §5.1.2.1, §6.4.3.1 | Repeat lot qualification for any new lot or date code; cell-voltage, current and temperature monitoring; cell matching; software controls enveloped by hardware controls; charger with at least two levels of overcharge control, dissimilar |
| UN Manual of Tests and Criteria Part III §38.3 | Cell and battery type tests T1–T8 before transport (49 CFR 173.185) |
| 47 CFR 97.207(g)(1)(iii); NASA-STD-8719.14 | Batteries left in a permanent discharge state at end of life |

## 3. Hazards and controls

Severity: **catastrophic** for fire, explosion or venting while integrated with the launch vehicle, co-passengers or
the ISS (loss of the vehicle or crew); **critical** on orbit after deployment (loss of the mission, debris).
"Controls" counts independent controls in series; fault tolerance is controls − 1.

| # | Hazard cause | Phase | Controls (independent) | Count | Fault tolerance | Required |
|---|--------------|-------|------------------------|-------|-----------------|----------|
| H1 | Overcharge (charger fault, cell imbalance) | On orbit; ground charging | (1) BCR constant-voltage regulation at 4.10 V/cell; (2) BMS AFE cell over-voltage at 4.25 V opens the CHG FET (hardware comparator, thresholds written at power-up); (3) **independent secondary over-voltage protector** at 4.35 V/cell with a fixed factory threshold, blowing a three-terminal fuse in the charge path (to be added: EPS-11) | 3 | **2** (with (3)); 1 today | ISS: 2; rideshare: 1 while charging is inhibited at launch |
| H1a | Overcharge at launch | Integrated, powered off | Charging cannot occur: the array is dark in a closed dispenser, DS2 opens the pack return; (1)–(3) above | — | — | Inhibits (§5) |
| H2 | Over-discharge, then recharge of a damaged cell | On orbit | (1) Flight-software mode thresholds (SAFE 25 %, EMERGENCY 15 %); (2) BMS AFE under-voltage 2.8 V/cell opens the DSG FET; (3) BCR charge lockout below 2.0 V/cell and pre-charge at 0.1 A below 3.0 V (to be specified: EPS-12); (4) independent hardware low-voltage disconnect of the payload, PA and heater rails at 3.4 V/cell (to be added: EPS-09) | 4 | 2 with (3) and (4) | 1 (critical) |
| H3 | External short (harness, connector, board) | All | (1) Non-resettable pack fuse F2, 5 A, breaking capacity ≥ 100 A at 32 VDC (cell short-circuit current ≈ 80–100 A); (2) BMS AFE short-circuit discharge trip (≤ 200 µs); (3) resettable PTC RT1; plus insulation (DFMR: pack-to-frame isolation ≥ 1 MΩ, both poles) | 3 | 2 | ISS: 2 |
| H3a | Short inside the pack upstream of F2 (bussing, insulation, cell-to-holder) | All | DFMR: insulated bussing, cell wraps intact, insulating sleeves at the clamp, inspection, isolation test, vibration screening | — | DFMR | DFMR accepted by JSC 20793 §4.1.2 |
| H4 | Internal cell short (manufacturing defect, plating) | All | DFMR: one screened lot (§4), no charging below 0 °C (H5), C/9 charge rate, vibration screening, mechanical retention | — | DFMR | DFMR |
| H5 | Charging below 0 °C (lithium plating) | On orbit | (1) BCR temperature input (TS pin, hardware) from NTC1 inhibits charge outside 0 … +45 °C; (2) flight software inhibits charge on NTC2; (3) heater holds ≥ +4 °C (THERMAL_ANALYSIS §6.1, with the recommended +5/+9 °C set points) | 3 | 2 | 1 (critical) |
| H6 | Over-temperature (stuck heater, stuck load, hot dispenser) | All | (1) Heater thermostat in software; (2) series hardware cutout at +40 °C in the heater circuit; (3) heater fuse or current limit; (4) pack over-temperature at +60 °C opening both FETs through a hardware comparator on an NTC (the BQ76920-class AFE has no autonomous over-temperature trip); charge inhibited above +45 °C (H5). No low-temperature discharge cut-off (EPS-08) | 4 | 2 for heater faults; dispenser soak is an environment (THERMAL_ANALYSIS §6.7) | 1 |
| H7 | Electrolyte leakage or venting | All | DFMR: hermetic 18650 cans with vent and CID; mechanical retention; vacuum leak check (mass change) | — | DFMR | DFMR |
| H8 | Mechanical damage (vibration, handling) | Ground, launch | Clamp design and vibration test at protoflight levels with functional checks; handling procedure | — | DFMR | DFMR |
| H9 | Stored energy after end of life (debris) | EOL | (1) Non-resettable charge-path disconnect F1, armed and fired by two independent signals; (2) loads drain the pack to the BMS under-voltage cut | 2 | 1 | 97.207(g)(1)(iii) |

**Findings.** Two hazard causes do not have the tolerance the backup path needs: overcharge (H1) has two
controls today, and both depend on settings written by firmware (the BCR set point and the AFE threshold); external
short (H3) has three. For the SpaceX path at launch the pack cannot be charged and is behind the inhibits, so H1 is
controlled by the inhibits and H3 by the fuse and insulation; the integrator's safety review decides whether one fault
tolerance is enough there. For the ISS backup, add the dissimilar secondary over-voltage protector with fuse
(H1 control 3). It is cheap (one IC and one three-terminal fuse) and should go in the baseline whichever path flies
(RID EPS-11).

## 4. Cell screening and lot acceptance

Per JSC 20793 Rev D §4.2.3 and §6.4.3.3, and the BATTERY_DESIGN §9 tests, for both paths:

| Step | Test | Criterion |
|------|------|-----------|
| 1 | Procure ≥ 40 cells of one lot and one date code, with the manufacturer's UN 38.3 test summary and certificate of conformance | One lot, traceable |
| 2 | 100 % visual with the wrap removed and re-sleeved, mass, dimensions, OCV, AC impedance (1 kHz), DC resistance | No defect; ±3σ outliers removed |
| 3 | Capacity: CC-CV charge to 4.20 V at 0.5 A, discharge at 0.68 A (0.2C) to 2.65 V, three cycles | ≥ 3,350 mAh; ±3σ |
| 4 | OCV retention: 30 days at 25 °C after a charge to 50 % SOC | Voltage drop within ±3σ of the lot; reject outliers |
| 5 | Lot uniformity after outlier removal | ±3σ / mean: OCV < 1 %, mass < 2 %, capacity < 5 %, resistance < 15 %; lot rejected if > 15 % of cells fail |
| 6 | Matching for each pack (flight and spare): capacity within 1 %, DC resistance within 5 %, OCV retention within the lot's 1σ | JSC 20793 §4.4.4 |
| 7 | Lot sample destructive tests (ISS path, ≥ 5 cells): external short at the pack's worst case, overcharge, forced discharge, CID and vent operation | Behaviour as the qualification lot (JSC 20793 §4.3.2) |
| 8 | Pack acceptance: functional baseline (OCV, mass, capacity at 4.10 V, resistance, visual), vibration at protoflight level, functional recheck, thermal vacuum ≥ 4 cycles below 1 × 10⁻⁴ Torr, recheck, ≥ 10 cycles in total | OCV and mass change < 0.5 %, capacity change < 3 % (JSC 20793 §6.4.3.2 b) |
| 9 | Protection functional test on the flight pack and EPS: each threshold of §3 tripped, including the 0 °C and 45 °C charge inhibits and the secondary protector (on the EM only, as it blows the fuse) | Trip within ±25 mV and ±2 °C of the setting |

## 5. Inhibits and the battery

The pack sits upstream of the deployment switches. The only circuits powered at launch are the BMS AFE (CDS
2.3.1.2), which must be in its lowest-power state (BQ76920 class: ship mode), and the secondary protector. The
inhibit chains are in `docs/hardware/EPS_DESIGN.md` §8. Two battery-specific points:

1. The pack's negative is switched by DS2. A short from the pack's negative to the frame would bypass DS2. The
   isolated mount (BUILD_VS_BUY §4) helps; specify ≥ 1 MΩ from each pole to the frame at 50 VDC and test it after
   vibration (V-BSA-4).
2. SpaceX requires the inhibit test during random vibration with the batteries in their flight charge state (UG v11
   §6.7.7). The flight charge state is §6 below.

## 6. State of charge for storage, launch and transport

| Phase | State of charge | Reason |
|-------|-----------------|--------|
| Cell and pack storage before integration | 40–60 % (OCV 3.65–3.75 V/cell), 10–25 °C, re-charged to that band every 3 months | Calendar ageing grows with SOC and temperature; keeps margin over self-discharge |
| Integration and the inhibit test in vibration | **50 ± 10 %** (≈ 3.70 V/cell, 14.8 V) | Flight state for UG v11 §6.7.7 |
| Delivery to the integrator (up to ≈ 90 days in processing, UG v11 Table 4-12) | 50 ± 10 % at delivery; ≥ 40 % at deployment | Energy for LEOP: the simulation leaves the dispenser at 58 % from 60 % and needs ≈ 6 Wh (12 %) for a cold first two orbits without charge (THERMAL_ANALYSIS §4) |
| Air transport of the spacecraft (UN3481, battery contained in equipment) | ≤ 50 %; the IATA 30 % limit applies only to batteries shipped alone (UN3480) | Lower severity; confirm with the carrier and the IATA Dangerous Goods Regulations edition in force |
| On orbit | Charged to 4.10 V/cell (≈ 91 % of the rated capacity) | D8 |

Never charge above 4.10 V/cell on the ground except in the cell capacity screen (step 3 of §4, cells not yet in a
pack). The ground charger must use the flight set point and its own over-voltage cut.

## 7. Charge set point 4.10 V/cell (decision D8)

| Check | Value | Verdict |
|-------|-------|---------|
| Margin to the BMS over-voltage threshold | 4.25 − 4.10 = 0.15 V/cell (was 0.05 V at 4.20 V) | Improved |
| Worst cell at end of charge: set point accuracy ±0.5 % (±20 mV/cell) plus 50 mV balance window | 4.17 V/cell | Below the cell's 4.20 V maximum and the 4.25 V threshold |
| Capacity given up | ≈ 9 % (≈ 91 % of rated at 4.10 V; the simulation's gauge resets to the OCV-implied charge) | Accepted (largest simulated drawdown 3.4 Wh, < 10 % of the pack at end of life) |
| Secondary protector threshold | 4.35 V/cell, ≥ 0.2 V above the set point | No nuisance trip from ripple or calibration |
| Effect on safety | Lower stored energy at end of charge, slower calendar ageing (Keil et al., *J. Electrochem. Soc.* 163(9), 2016) | Positive |

## 8. UN 38.3

The INR18650-35E has a manufacturer UN 38.3 test summary; obtain it with the lot (step 1). The assembled 4S pack is a
new battery design: test a pack built from the flight lot to T1 (altitude), T2 (thermal), T3 (vibration), T4 (shock),
T5 (external short) and T7 (overcharge), with T6 and T8 covered by the cell summary, and issue the test summary
required by 49 CFR 173.185(a)(3). Use the engineering-model pack (or a third pack of flight-lot cells) and an
accredited laboratory. Budget ≈ $8–15k and 6–8 weeks (no quote yet).

## 9. Verification items

| ID | Item | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-BSA-1 | Cell lot screening and acceptance (§4 steps 1–6) | §4 criteria | H4, SYS-088 | Before the EM pack |
| V-BSA-2 | Protection trip tests on the EM EPS and pack (§4 step 9), including the secondary protector and its fuse | Trips within tolerance; the secondary protector blows the fuse at 4.35 ± 0.025 V | H1, H2, H3, H5, H6 | EM |
| V-BSA-3 | Pack acceptance on the flight pack (§4 step 8) | §4 criteria | JSC 20793 §6.4.3.3 | FM |
| V-BSA-4 | Pack-to-frame isolation, both poles, after vibration | ≥ 1 MΩ at 50 VDC | H3a, §5 item 1 | EM, FM |
| V-BSA-5 | UN 38.3 battery tests (§8) | All tests passed, test summary issued | 49 CFR 173.185 | Before first shipment of a pack |
| V-BSA-6 | Inhibit test during random vibration at the flight SOC (§6) | No power-up, no RF (UG v11 §6.7.7) | SYS-704 | FM protoflight vibration |
| V-BSA-7 | EOL disconnect: arm and fire on the EM; no single fault fires it | F1 opens only with both signals; non-resettable | H9, 97.207(g) | EM |
| V-BSA-8 | LEO-profile cycling of flight-lot cells at 4.10 V (BATTERY_DESIGN §9.3) | Fade per cycle within the §3.3 assumption | D8 | EM phase, ≥ 3 months |

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR review, EPS/thermal | Initial issue: hazards, controls and fault tolerance, screening, inhibits, state of charge, 4.10 V set point, UN 38.3 |
