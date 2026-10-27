# SMART-QSO Battery Safety Assessment

**Document ID**: SQSO-BSA-001
**Version**: D
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
| Discharge | ≤ 1.7 A peak (Jetson at the 12 W guard, PA at 1 W, housekeeping, heater, magnetorquers, with the magnetorquer switch in its limit); 0.75 A during a burn; ≈ 0.1 A in eclipse; cell continuous limit 8 A | SQSO-EPS-BRD-001 §6.1; POWER_BUDGET_ANALYSIS §5 |
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
| H1 | Overcharge (charger fault, cell imbalance) | On orbit; ground charging | (1) BCR constant-voltage regulation at 4.10 V/cell; (2) BMS AFE cell over-voltage at 4.25 V opens the CHG FET (hardware comparator, thresholds written at power-up); (3) **independent secondary over-voltage protector** at 4.35 V/cell with a fixed factory threshold, blowing a three-terminal fuse in the pack line (BQ771806 + F3, SQSO-EPS-BRD-001 §6.1; RID EPS-11) | 3 | **2** | ISS: 2; rideshare: 1 while charging is inhibited at launch |
| H1a | Overcharge at launch | Integrated, powered off | Charging cannot occur: the array is dark in a closed dispenser, DS2 opens the pack return; (1)–(3) above | — | — | Inhibits (§5) |
| H2 | Over-discharge, then recharge of a damaged cell | On orbit | (1) Flight-software mode thresholds (SAFE 25 %, EMERGENCY 15 %); (2) BMS AFE under-voltage 2.8 V/cell opens the DSG FET; (3) **latched recharge lockout**: any cell below 2.0 V/cell sets a non-volatile flag that holds the charger off and the CHG FET open until an authenticated ground command clears it (the bus runs on the array through the power path meanwhile); pre-charge at 0.1 A below 3.0 V (EPS_DESIGN v2.0 §3.3; RID SMA-06 / EPS-12); (4) independent hardware low-voltage disconnect of the payload and PA rails at 3.4 V/cell (13.6 V for 5 s, SQSO-EPS-BRD-001 §8.4; RID EPS-09) | 4 | 2 with (3) and (4) | 1 (critical) |
| H3 | External short downstream of the AFE FETs (EPS board after the FETs, loads, harnesses from the EPS) | All | (1) Positive-leg fuses F0A, F0B (battery sense board) and F2 (EPS), MIL-PRF-23419/12 FM12A72V5.0A, interrupt 1000 A against a prospective ≤ 209 A (SQSO-EPS-BRD-001 D §6.1); (2) negative-leg fuse **F0N** (same part, sense board, cell-1 negative: JSC 20793 Rev D §5.1.3.1a); (3) BMS AFE over-current 4 A for 0.5 s and short-circuit trip 10 A in 15 µs, below the fuses so that the resettable trip acts first; RT1 (40 A maximum) is not counted as an interrupter; plus insulation (pack-to-frame ≥ 1 MΩ, both poles) not counted | 3 (positive-leg fuses counted as one) | **2** | ISS: 2 |
| H3b | Short upstream of the AFE FETs: battery leads W1, EPS J1 and the F2–RT1–F3 copper (lead to lead; BAT_TOP to frame, the frame reaching the pack negative through DS2 once released), and the cell-tap leads | All | Pack leads: (1) F0A, (2) F0B (BAT_TOP, two lots), (3) F0N (BAT_BOT): three interrupting elements in every loop; DS2 open at launch also breaks the lead-to-frame loop. Taps: (1) the 3 × 68.1 Ω current-limiting chain at the sense-board end of each tap (≤ 82 mA tap to frame, ≤ 41 mA tap to tap; one chain alone holds the current below the wire's rating), (2) the second tap's chain, (3) the FM12A72V3/4A tap fuses. Construction (22 AWG with PTFE over-sleeve, 0.12 m, hipot) kept, not counted: JSC 20793 Rev D §4.1.2 does not accept DFMR for wiring | 3 | **2** (chair decision, CDR round 3: F0A and F0B are from different lots, each rated for the full fault and each able to clear it alone, so they are two independent controls, F0N the third); taps **2**. **Decision point:** the NRCSD safety engineer's concurrence for the ISS backup path (owner: safety lead, at the NRCSD contract) | ISS: 2 |
| H3a | Short inside the pack upstream of the sense-board fuses (cells to F0A or F0N: bussing, insulation, cell-to-holder) | All | DFMR: insulated bussing, cell wraps intact, insulating sleeves at the clamp, inspection, isolation test, vibration screening | — | DFMR | DFMR accepted by JSC 20793 §4.1.2 |
| H4 | Internal cell short (manufacturing defect, plating) | All | DFMR: one screened lot (§4), no charging below 0 °C (H5), C/9 charge rate, vibration screening, mechanical retention | — | DFMR | DFMR |
| H5 | Charging below 0 °C (lithium plating) | On orbit | (1) BCR temperature input (TS pin, hardware) from NTC1 inhibits charge outside 0 … +45 °C; (2) flight software inhibits charge on NTC2; (3) heater holds ≥ +4 °C (THERMAL_ANALYSIS §6.1, with the recommended +5/+9 °C set points) | 3 | 2 | 1 (critical) |
| H6 | Over-temperature (stuck heater, stuck load, hot dispenser) | All | (1) Heater thermostat in software; (2) series hardware cutout at +40 °C in the heater circuit; (3) heater fuse or current limit; (4) pack over-temperature at +60 °C opening both FETs through a hardware comparator on NTC1, with the BQ76952 AFE's own 60 °C discharge trip on NTC2 behind it (SQSO-EPS-BRD-001 §5.3); charge inhibited above +45 °C (H5). No low-temperature discharge cut-off (EPS-08) | 4 | 2 for heater faults; dispenser soak is an environment (THERMAL_ANALYSIS §6.7) | 1 |
| H7 | Electrolyte leakage or venting | All | DFMR: hermetic 18650 cans with vent and CID; mechanical retention; vacuum leak check (mass change) | — | DFMR | DFMR |
| H8 | Mechanical damage (vibration, handling) | Ground, launch | Clamp design and vibration test at protoflight levels with functional checks; handling procedure | — | DFMR | DFMR |
| H8a | Deployment burn over-current (direct drive ≈ 5.8 A on one line would open F2: mission loss, not a safety hazard) | LEOP | Four independent burn lines, one per resistor (BURN_W1A PB10, BURN_W1B PE6, BURN_W2A PB9, BURN_W2B PE7; SQSO-ICX-001 §3.1), fed from a 7.0 V burn buck: (1) current-limited driver per resistor (0.8 A at the resistor, 0.51 A at the pack); (2) one resistor at a time in software, so the pack carries **0.67 A** nominal with housekeeping and heater, **0.75 A** worst case with the magnetorquers at 1.16 W, against a **1.0 A** limit; (3) 12 s hardware one-shot per driver; (4) burn-buck current limit, which holds even four drivers on at once (a software fault) to 2.0 A at the pack; (5) BMS 4 A trip below F2 (SQSO-EPS-BRD-001 §9, §6.1) | 5 | 2 for the fuse | Mission (RID SMA-05) |
| H9 | Stored energy after end of life (debris) | EOL | (1) Non-resettable charge-path disconnect F1: arm = the OBC's EOL_ARM AND the EPS controller's keyed EOL_KEY, fire = the OBC's EOL_FIRE (no single hardware or OBC pin fault fires or blocks both steps; a flight-software fault executing the whole keyed sequence is not excluded, SQSO-EPS-BRD-001 D §7); (2) loads drain the pack to the BMS under-voltage cut | 2 | 1 | 97.207(g)(1)(iii) |

**Findings.** At the first issue two hazard causes lacked the tolerance the backup path needs: overcharge (H1) had two
controls, both depending on settings written by firmware (the BCR set point and the AFE threshold); external short
(H3) has three. The EPS board (SQSO-EPS-BRD-001) now adds the fixed-threshold secondary protector, so H1 has three
dissimilar controls. Version D adds H3b: the battery leads and taps were upstream of every fuse until the sense-board fuses F0A, F0B, F0N and the tap chains. For the SpaceX path at launch the pack cannot be charged and is behind the inhibits, so H1 is
controlled by the inhibits and H3 by the fuse and insulation; the integrator's safety review decides whether one fault
tolerance is enough there. For the ISS backup, the dissimilar secondary over-voltage protector with fuse
(H1 control 3) was needed; it is cheap (one IC and one three-terminal fuse) and is in the board design for both paths
(RID EPS-11).

### 3.1 Hazard class and the custom-battery abuse tests (JSC 20793 Rev D; RID P3E-03)

**Class.** Applying the JSC 20793 Rev D §4.1.3 classes, as the round-3 review panel reads them: a custom-built 4S
lithium-ion pack (49.7 Wh, not a certified commercial battery) in a payload is a **catastrophic** hazard class: it
needs two-fault tolerance for each catastrophic cause (§4.1.1, through SSP 51700 for the ISS path) and the abuse
tests of a custom battery. The §4.1.3 text itself is not reproduced here (the document is not in the repository):
the class is to be confirmed against it with the NRCSD provider's safety engineer. For the Falcon 9 path the
same class is the conservative reading of the SSCMAN 91-710 "potentially hazardous system" listing (§2).

**Abuse-test programme** (cells from the flight lot; packs built to the flight drawing from the same lot):

| Test (JSC 20793 Rev D) | Level | Article | Falcon 9 (rideshare) | ISS / NRCSD backup |
|------------------------|-------|---------|----------------------|--------------------|
| Overcharge (§6.4.3.1) | Cell and pack, protection disabled and enabled | 3 cells, EM pack | Pack with protections enabled (also UN 38.3 T7) | All |
| Over-discharge / forced discharge (§6.4.3.1) | Cell and pack | 3 cells, EM pack | Pack (protections enabled) | All |
| External short, low impedance (§6.4.3.1) | Cell (≤ 5 mΩ) | 3 cells | — (UN 38.3 T5 at pack level) | Yes |
| External short, "smart" impedance (just below the protection's trip, §6.4.3.1) | Cell and pack | 3 cells, pack | — | Yes |
| **Pack hard short and soft short with the current limiters bypassed** (§5.1.3.1c) | Pack, fuses and AFE bypassed in turn, at full charge | Sacrificial pack 1 | **Yes** (shows the fuses and the AFE are the controls: H3, H3b) | Yes |
| Internal short / crush (§6.4.3.1) | Cell | 3 cells | — | Yes |
| Heat to vent (§6.4.3.1) | Cell | 3 cells | — | Yes |
| Vent/burst ratio and venting path (§5.1.1.1e–g) | Pack enclosure: vent area vs. cell vent flow, burst margin | Analysis + sacrificial pack 2 | Analysis | Analysis and test |
| Vibration (§6.4.3.1) | Pack, qualification level, with function check | EM pack | Yes (protoflight, V-BSA-6) | Yes (qualification level) |
| Drop (§6.4.3.1) | Pack | Sacrificial pack 2 | — | Yes |
| Propagation (§5.1.5.1a) | — | — | Not required (< 80 Wh, §1) | Not required |

**Cost (ROM, no quote):** Falcon 9 path, over the baseline UN 38.3 budget (§8): one extra flight-lot pack (≈ $2–3k
in cells, board and labour), the pack hard/soft short with limiters bypassed and the protection-enabled overcharge
and over-discharge at an accredited lab (≈ $8–12k): **≈ $10–15k**, 3–4 weeks in parallel with UN 38.3. ISS backup,
the full programme: 24–30 flight-lot cells (≈ $0.2k), two sacrificial packs (≈ $5–6k), cell-level abuse at a
battery test lab (≈ $25–40k), pack-level abuse including vent/burst, drop and the bypassed shorts (≈ $20–30k),
reports for the NRCSD safety data package: **≈ $50–75k**, 8–12 weeks. Both are added to V-BSA-9.

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

The pack sits upstream of the three deployment switches DS1, DS2 and DS3 (DS3 added at CDR, EPS_DESIGN v2.0 §8). The
only circuits powered at launch are the BMS AFE (CDS 2.3.1.2), which must be in its lowest-power state (BQ76952
SHUTDOWN), the secondary protector and the access-port board's UMB_VMON TVS: ≤ 20 µA in all (7.7 µA worst designed, SQSO-EPS-BRD-001 §6.6) (LVR-051/071, `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`),
1.3 % of the pack in 90 days. The
inhibit chains are in `docs/hardware/EPS_DESIGN.md` §8. Two battery-specific points:

1. The pack's negative is switched by DS2 (DS1 and DS3 are in VBAT+). A short from the pack's negative to the frame would bypass DS2. The
   isolated mount (BUILD_VS_BUY §4) helps; specify ≥ 1 MΩ from each pole to the frame at 50 VDC and test it after
   vibration (V-BSA-4).
2. SpaceX requires the inhibit test during random vibration with the batteries in their flight charge state (UG v11
   §6.7.7). The flight charge state is §6 below.

## 6. State of charge for storage, launch and transport

| Phase | State of charge | Reason |
|-------|-----------------|--------|
| Cell and pack storage before integration | 40–60 % (OCV 3.65–3.75 V/cell), 10–25 °C, re-charged to that band every 3 months | Calendar ageing grows with SOC and temperature; keeps margin over self-discharge |
| Integration and the inhibit test in vibration | **60 ± 5 %** (≈ 3.80 V/cell, 15.2 V), as LVR-051/071 of `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` | Flight state for UG v11 §6.7.7; the value the mission simulation uses (`integration_soc` 0.60) |
| Delivery to the integrator (up to ≈ 90 days in processing, UG v11 Table 4-12) | 60 ± 5 % at delivery; ≥ 40 % at deployment (self-discharge ≈ 4.5 % and the ≤ 20 µA drain 1.3 % in 90 days leave ≈ 49–60 %) | Energy for LEOP: the simulation leaves the dispenser at 58 % from 60 % and needs ≈ 6 Wh (12 %) for a cold first two orbits without charge (THERMAL_ANALYSIS §4) |
| Air transport of the spacecraft (UN3481, battery contained in equipment, PI 967; 49.7 Wh < 100 Wh) | **≤ 30 %**, then the final charge to 60 ± 5 % at the integrator (L-47 d); road transport may ship at the 60 % integration state | IATA DGR 67th edition (2026): ≤ 30 % SoC is **mandatory** for cells and batteries shipped alone (PI 965) and packed with equipment (PI 966, > 2.7 Wh) and **strongly recommended, not mandatory**, for batteries contained in equipment (PI 967) (IATA Battery Guidance Document, revised for the 2026 regulations, PI 967 Section I and II); operator variations can make it mandatory, so the project ships at ≤ 30 % by air (MISSION_TIMELINE §3, LAUNCH_PROVIDER_DELIVERABLES aligned) |
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
| V-BSA-2 | Protection trip tests on the EM EPS and pack (§4 step 9), including the secondary protector and its fuse | Trips within tolerance; the secondary protector blows the fuse at 4.35 ± 0.025 V | H1, H2, H3, H3b, H5, H6 | EM |
| V-BSA-3 | Pack acceptance on the flight pack (§4 step 8) | §4 criteria | JSC 20793 §6.4.3.3 | FM |
| V-BSA-4 | Pack-to-frame isolation, both poles, after vibration | ≥ 1 MΩ at 50 VDC | H3a, H3b, §5 item 1 | EM, FM |
| V-BSA-5 | UN 38.3 battery tests (§8) | All tests passed, test summary issued | 49 CFR 173.185 | Before first shipment of a pack |
| V-BSA-6 | Inhibit test during random vibration at the flight SOC (60 ± 5 %, §6), each of DS1, DS2, DS3 checked | No power-up, no RF (UG v11 §6.7.7) | SYS-704 | FM protoflight vibration |
| V-BSA-7 | EOL disconnect: arm and fire on the EM; no single fault fires it | F1 opens only with both signals; non-resettable | H9, 97.207(g) | EM |
| V-BSA-8 | LEO-profile cycling of flight-lot cells at 4.10 V (BATTERY_DESIGN §9.3) | Fade per cycle within the §3.3 assumption | D8 | EM phase, ≥ 3 months |
| V-BSA-9 | Custom-battery abuse tests of §3.1 (the Falcon 9 set; the full set if the ISS backup is used) | No fire, explosion or rupture; venting only through the vent path; the fuses or the AFE end each short; protections end overcharge and over-discharge | H1, H2, H3, H3b, H6, H7 | EM phase |

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR review, EPS/thermal | Initial issue: hazards, controls and fault tolerance, screening, inhibits, state of charge, 4.10 V set point, UN 38.3; after the safety review: latched recharge lockout, BMS 4 A below the fuse, deployment burn current (H8a), DS3, integration SOC 60 ± 5 % as LVR-051/071 |
| B | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: H8a from the EPS board: four burn lines PB10, PE6, PB9, PE7, one resistor at a time, 0.67 A nominal at the pack (0.75 A worst case, 1.0 A limit), burn-buck fault cap; H1, H2, H6 controls and §5 drain as designed in SQSO-EPS-BRD-001; §1 discharge peak |
| C | 2026-10-27 | CDR design (EPS board) | H3: pack fuses F0 (battery sense board) and F2 in series, both MIL-PRF-23419/12 FM12A72V5.0A with 1000 A interrupt against a ≤ 314 A prospective current (replacing the Nano2 rated 50 A at 125 V / 300 A at 32 VDC); new H3b for shorts in the battery leads W1 and the cell taps, controlled by F0 and the tap fuses FT0–FT4 plus double insulation (they were upstream of every fuse); H3a now ends at F0; §5 storage drain 7.7 µA worst with the access-port board's TVS |
| D | 2026-10-27 | CDR design (EPS board) | Round-3 panel: H3 and H3b restated with the negative-leg fuse F0N, the second positive-leg fuse and the tap current-limiting chains (P3E-02; RT1 not counted, DFMR for wiring not counted); hazard class catastrophic and the custom-battery abuse-test programme with Falcon 9 / ISS applicability and cost (§3.1, V-BSA-9; P3E-03); H9 with the keyed EOL arm (P3E-11); air transport ≤ 30 % SoC per the IATA 2026 guidance for PI 967 (§6; P3E-12); amended: H3b 2-fault tolerant on the chair's decision (F0A and F0B from two lots as independent controls, F0N the third), NRCSD safety engineer's concurrence recorded as a decision point |
