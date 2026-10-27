# Technology Readiness Assessment and Technology Development Plan
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-TRA-001
**Revision:** B
**Date:** 2026-10-27
**Status:** Assessment by the CDR 2026 design review (round 3, RID SE-09), for the independent PDR (2027-01-12). No
hardware exists: every TRL below rests on analysis, simulation or a vendor's record, as its evidence column says. To be
re-assessed at the PDR and with the engineering-model (EM) results at the CDR.
**Reference:** NPR 7123.1D Appendix E (Technology Readiness Levels, hardware and software descriptions; exit criteria);
NASA TRL scale as applied in NASA/SP-2016-6105 Rev 2 Appendix G; `docs/project/CDR_CRITERIA_MATRIX.md` §2.3 rows 14–15
(project tailoring: the TRA and the technology development plan are required at the PDR for the Jetson and the
transponder, and for every critical element at the CDR); `docs/project/HERITAGE_ASSESSMENT.md` (precedents)

---

## 1. Purpose and Method

The review found no technology readiness assessment anywhere in the project (RID SE-09), although the project's own
tailoring requires one at the PDR. This document:

1. identifies the **critical elements**: every element that is new, or a known element in a new application or
   environment (1U volume, LEO thermal-vacuum, launch loads, radiation), whose failure would cost a mission success
   level or the safety of the launch;
2. assigns each a **TRL today** with the NPR 7123.1D Appendix E definition it meets and the evidence for it, and states
   the next level's exit criterion;
3. sets the **target TRL 6 at the CDR** (2027-09-28, `mission/SCHEDULE.md` Rev E): "system/subsystem model or
   prototype demonstration in a relevant environment"; for this protoflight project the model is the EM and the
   relevant environment is the EM test campaign (thermal vacuum, random vibration, EMC, power and thermal duty), with
   radiation by analysis plus the latch-up protection test where no beam test is planned;
4. gives the **technology development plan**: the steps, verification items and dates, tied to the EM schedule, that
   take each element to TRL 6, and the fallback if one does not get there.

### 1.1 TRL definitions used (NPR 7123.1D Appendix E, abridged)

| TRL | Hardware | Software | Exit criterion (abridged) |
|-----|----------|----------|---------------------------|
| 3 | Analytical and experimental proof of concept of critical functions | Algorithms run on a surrogate processor in a laboratory | Documented analytical or experimental results validating the predictions |
| 4 | Component or breadboard validated in a laboratory environment | Key, functionally critical components integrated and functionally validated; relevant environments defined | Documented test performance against analytical predictions; relevant environment defined |
| 5 | Component or breadboard validated in a relevant environment | End-to-end software elements implemented and interfaced with existing systems or simulations conforming to the target environment | Documented test performance against predictions; scaling requirements defined |
| 6 | System or subsystem model or prototype demonstrated in a relevant environment | Prototype implementations on full-scale realistic problems, partially integrated with existing hardware and software | Documented test performance against predictions |
| 7 | System prototype demonstrated in an operational (space) environment | Prototype software on the target in a high-fidelity operational environment | — (flight) |
| 9 | Actual system flight-proven through successful mission operations | Same | — |

A bought unit takes the TRL of its flight record for the configuration and environment it flew in. Where this mission
uses it outside that record (another orbit, a 1U, another mode), the TRL is assessed for the new use.

## 2. Critical Elements and TRL Today

| # | Critical element | Why critical | TRL today | Evidence | Gap to TRL 6 |
|---|------------------|--------------|-----------|----------|--------------|
| CE-1 | **NVIDIA Jetson Orin Nano 8 GB in LEO on a 1U**, power-gated bursts at 7 W mode, ≤ 8 W input, no fan (BASELINE §6) | The gated AI experiment (D5); the largest load and heat source; a latch-up or a stuck-on rail threatens the bus | **4** (the module works in the laboratory as a commercial product; LEO thermal, vacuum, vibration and radiation environments defined in `docs/environment/THERMAL_ANALYSIS.md`, `docs/environment/RADIATION_ANALYSIS.md`, ETP §3) | Commercial module data sheet; one Orin-class carrier flown on a small CubeSat with no published results (HERITAGE M7); Orin AGX SEE/TID test listing only, none for the Nano (RADIATION_ANALYSIS §4); the 7 W mode is listed by NVIDIA only for the original flash configuration (TB-13) | Burst energy and thermal on the EM carrier in vacuum (VI-01, VI-02, V-JET-4, V-JET-7); vibration of the populated carrier (V-JET-8); power-cut and fault injection (V-JET-5, V-JET-6, RAD-V5). **Radiation stays an analysis** (no beam test planned); accepted because the module is non-critical and expendable with hardware latch-up trips |
| CE-2 | **Jetson carrier board and heat path** (SQSO-JET-BRD-001: 6-layer 2.0 mm, PCIe and USB striplines, SO-DIMM, spreader pedestal and gasket to the shear panels) | Custom; carries the module's 8 W into a 1U structure; SYS-603 (Rev G) temperature limits | **3** | Schematic captured as code with ERC and BOM; `jetson_calcs.py` (junction-to-spreader 1.09–1.16 K/W against 5.7 K/W allowed); spreader-to-frame 1.10 W/K estimated (MECHANICAL_DESIGN §6.10); thermal analysis: 39.0 °C for a 600 s batch even at 0.44 W/K | Carrier EM built and powered (V-JET-1 to V-JET-3, TRL 4); heat path measured in vacuum (VI-01, V-THM-3, TRL 5); EM thermal vacuum and vibration (VI-03, V-JET-8, TRL 6) |
| CE-3 | **Custom U/V FM transponder and telemetry radio** (SQSO-RF-BRD-001: CC1200 analog-FM receive with BFP640F LNA and SAW, LMX2571 exciter, PA, TNC on an STM32L431) | Primary mission (MO-1 to MO-3); 47 CFR 97.307(e) spurious limits and the TX-OFF path | **3** | Schematic captured as code (316 parts, ERC 0) with `rf_calcs.py`; demodulator simulation (`software/simulation/cdr/rf/fm_demod_sim.py`); desense and EMC analyses (SQSO-EMC-001); single-channel FM transponders flew on 1U before (AO-85, HERITAGE M1) but this design has not been built | Breadboard-level tests on the EM board: sensitivity, filter rejection, spurious ≥ 70 dBc third harmonic, TX-OFF (VI-RFB-01 to -11, VI-RF-11, -12; TRL 4); temperature (VI-RFB-07) and EMC/self-compatibility in the EM (VI-06, TRL 5); EM thermal vacuum and vibration with the RF board powered at the plateaus (VI-03, VI-04, TRL 6) |
| CE-4 | **Custom EPS board** (SQSO-EPS-BRD-001: four TPS55340-Q1 MPPT trackers, BMS and AFE with secondary protector, three deployment-switch inhibits, per-rail latch-up trips, permanent passivation) | Every function depends on it; carries the safety inhibits and the end-of-life passivation | **3** | Schematic as code, 700 placements, ERC clean, `eps_calcs.py` (tracker efficiency 90.9–92.3 %, inhibit independence, fuse coordination); EMC analysis of the synchronised converters | EPS EM tests (V-EPS-1 to -13, V-EMC-3, VI-SAF-01 to -05, RAD-V5; TRL 4–5); EM thermal vacuum, vibration and the bus load-step test (VI-03, VI-04, V-EMC-6; TRL 6) |
| CE-5 | **OBC board and flight software** (SQSO-OBC-BRD-001 with the STM32L4R5ZI and the TPS3850 supervisor; Rust flight software on the Embassy executor) | Executive, FDIR, telecommand authentication, inhibits and timers in software | Board **3**; software **4** | Board: schematic as code (351 placements), ERC clean, `obc_calcs.py`. Software: 100 % line and condition coverage on the safety-critical files in CI; the mission simulation `sqso-sil` runs the flight code end to end from integration to re-entry; the firmware image builds for the target in CI but has run on no hardware (CLAUDE.md; BASELINE §6) | Development board HIL (`software/flight/tests/hil/HIL_TEST_PLAN.md`; software TRL 5, development board by 2026-11-30, HRT §8); OBC EM bring-up (V-OBC-1 to -18; board TRL 4); flatsat with the flight software on the target, ≥ 500 h (VI-11; TRL 5); EM thermal vacuum and vibration (VI-03, VI-04, V-OBC-13, V-OBC-16; TRL 6) |
| CE-6 | **OBC-resident AI selector** (gradient-boosted trees or a tiny INT8 MLP choosing pre-approved templates and the transponder schedule, with abstention to fixed rules; `docs/payload/OBC_SELECTOR_DESIGN.md`) | The flight-baseline AI (D5); hard limits stay outside it, so a failure falls back to rules | **3** (software) | Designed, not implemented (BASELINE §6) | Implementation and validation in the mission simulation against the rule-based baseline (TRL 4, by the CDR); on the target in the flatsat (TRL 5); on the EM in the day-in-the-life rehearsal (TRL 6 by the FM day-in-the-life, 2028-02). **It does not reach TRL 6 by the CDR**; acceptable because the rules fallback is the baseline behaviour |
| CE-7 | **Battery pack and isolated mount** (4S1P INR18650-35E at 4.10 V/cell, G-10CR or PEEK isolators, heater, flying leads; `hardware/eps/BATTERY_DESIGN.md`) | Catastrophic-hazard item for the launch (fire, venting); cold-case survival | Cells **9** as a CubeSat cell type (HERITAGE R1); pack and mount **3** | Cell type widely flown (R1); thermal analysis of the mount (0.03 W/K, heater 0.72–1.30 W); BSA hazard controls | Cell-lot screening (V-BSA-1); EM pack protection trips (V-BSA-2; TRL 4); abuse tests on sacrificial packs (V-BSA-9, task BATT_ABUSE; TRL 5); EM thermal balance and vibration (V-THM-2, VI-03, VI-04; TRL 6) |
| CE-8 | **Deployable solar wings and hinge** (two coaxial 17-7PH torsion springs, clock-spring flex, damped stop, two burn resistors per wing on four independent drivers; `docs/environment/MECHANISMS_ANALYSIS.md`) | Power (wings stowed: SAFE and IDLE only); disposal (both wings lost raises the lifetime) | **3** | Mechanisms analysis (end-of-travel torque margin +0.14 cold), FE model, CAD sweep clearance 0.35 mm; burn-wire release is a common CubeSat technique but this hinge has not been built | Spring-lot tests (VI-MECH-01; TRL 4); EM hinge torque test in vacuum at −40/+20/+60 °C (VI-STR-04; TRL 5); EM deployments at the cold predicted temperature and minimum voltage, before and after EM vibration (VI-LV-17 on the EM, VI-STR-06; TRL 6) |
| CE-9 | **ISIS AntS 1U–3U** in the monopole configuration, I²C arm/deploy, deployed after the wings | Every RF link; disposal cross-section | **9** for the product as the vendor flies it (vendor flight record, to be cited from the ISIS ICD); **6** assessed for this integration (new mounting, wing-first sequence, our I²C driver) | Vendor product; `sqso_fsw::ants` against the published command set (AV-V7 open); CAD clearance to the stowed wings 4–6 mm | ISIS ICD inspection (AV-V7, with the AntS RFQ by 2026-11-06); EM deployment, cold case and VSWR (VI-RF-16, VI-MCAD-09); the flexible-tape case in the ADCS Monte Carlo (DA-17) |
| CE-10 | **ADCS sensors and actuators in this use**: Epson M-G370PDT0 industrial gyro (not space-qualified), PNI RM3100 magnetometer on board A2, CubeSpace CR0002 rods; spin-stabilised sun-pointing at 8 °/s | Sun-pointing gives the +75 % sun-pointed power margin; detumble | Rods and magnetometer **5–9** by vendor and CubeSat use (to be confirmed from vendor data); gyro **4** for this use; control law **4** (software) | Monte Carlo of the flight code with the CAD tensor (ADCS_DESIGN §11.4); gyro vibration and shock assessment (OBC_BOARD_DESIGN §9.2: random at the FE-predicted module levels needs a test) | Gyro vibration and shock on the OBC EM (V-OBC-16; TRL 5–6); coil-to-magnetometer coupling and rod remanence (AV-V4, V-OBC-11); HIL closed loop on the flatsat (AV-V6; TRL 5); EM-level vibration (VI-04; TRL 6) |
| CE-11 | **GNSS receiver** (SkyFox piNAV-NG/FM, duty-cycled) for the SpaceX ephemeris rule | Launch-provider requirement (UG v11 §9.5.7.2) | **7–9** by the vendor's record (first flight 2017, > 50 units, vendor claims; HERITAGE R2) | Vendor data; duty-cycled use and the 45-min silence in SYS-224 (Rev G) | L1 noise with the Jetson, PA and magnetorquers (V-EMC-2); fix and power gating (AV-V8); end-to-end ephemeris demonstration (VI-LV-13) |
| CE-12 | **Authenticated amateur-band telecommand with hardware TX-OFF** (HMAC-SHA-256, monotonic counters, key table; SQSO-ICD-013) | 47 CFR 97.211 and 97.207(b); a failure leaves a transmitter that cannot be turned off | Software **4**; the TX-OFF hardware path **3** | CI tests of the authentication and counters; design study only as a precedent (HERITAGE R3) | TX-OFF at the antenna port on the EM (VI-RF-12, TP-RF-006); end-to-end through a station simulator on the flatsat (TRL 5) and on the EM (TRL 6) |

Elements with flight heritage in the same use and no new environment (the 1U frame, the AZUR 3G30A cells, the
deployment switches) are not critical technology and are verified by inspection and the protoflight campaign.

## 3. Summary

| TRL today | Critical elements |
|-----------|-------------------|
| 3 | CE-2 carrier and heat path, CE-3 transponder, CE-4 EPS, CE-5 OBC board, CE-6 AI selector, CE-7 pack and mount, CE-8 wings and hinge, CE-12 TX-OFF hardware |
| 4 | CE-1 Jetson in LEO, CE-5 flight software, CE-10 gyro and control law, CE-12 authentication software |
| 6–9 | CE-9 AntS (6 for this integration), CE-10 rods and magnetometer (vendor data to be cited), CE-11 GNSS receiver |

**Finding.** Eight critical elements are at TRL 3 two and a half months before the PDR: their designs are complete to
schematic and analysis level (round 2), and none has been built. The project's protoflight plan takes them to TRL 6 on
the EM before the CDR; that is a three-level step in one EM cycle, which is the norm for a Class D CubeSat but leaves
no room for a second EM. The plan in §4 is therefore on the schedule's critical path, and its risks are in the risk
register (R1, R16 for the Jetson; R23 and R24 for the hardware build).

## 4. Technology Development Plan

Dates are the early dates of `mission/SCHEDULE.md` Rev E (unchanged from Rev D; EM boards assembled 2027-06-01; EM integration 2027-06-01 to
2027-07-27; EM tests 2027-07-27 to 2027-09-21; CDR 2027-09-28). Each step's verification item, pass criterion and owner
are in `docs/verification/VI_REGISTER.md`.

| Element | Step to TRL 4 (laboratory) | Step to TRL 5 (relevant environment, component) | Step to TRL 6 (EM, relevant environment) | Fallback if TRL 6 is not shown at the CDR |
|---------|---------------------------|-----------------------------------------------|------------------------------------------|-------------------------------------------|
| CE-1 Jetson | Module on the NVIDIA developer kit: 7 W mode on the flight image, burst energy, boot time (by 2027-03-31, before the carrier EM; selection §5) | EM carrier: V-JET-3 to V-JET-6, RAD-V5 latch-up trip (2027-06 to 2027-07) | VI-01 heat path and VI-02 energy per accepted beacon in vacuum; V-JET-8 vibration; EM TVAC with a burst at each hot plateau (2027-07-27 to 2027-09-21) | Fly the carrier powered off or descope the generative experiment; the OBC selector is the flight baseline (D5) |
| CE-2 Carrier and heat path | V-JET-1, V-JET-2 (2027-02-01, before the carrier layout), V-JET-3 | V-JET-7 and V-THM-3 in vacuum | VI-01, VI-03, V-JET-8 | Thicker gasket or a third flange bolt; lower the burst duty to meet SYS-603 |
| CE-3 Transponder | VI-RFB-01 to -06 on the EM board (2027-06) | VI-RFB-07 over temperature; VI-06 EMC pre-scan and receiver-input emissions | VI-03, VI-04 with the RF board powered; VI-RF-04 spurious at the antenna port | Beacon-only fallback (D4) with the receiver kept for telecommand |
| CE-4 EPS | V-EPS-1 to -7 on the EPS EM (2027-06) | V-EPS-8 to -13, V-EMC-3, VI-SAF-01 to -05 | VI-03, VI-04, V-EMC-6 | ISIS ICEPS2 Type B as the alternate EPS (HRT §3) with its own inhibit design |
| CE-5 OBC and flight software | Software: HIL on the development board (by 2027-01-12 for the PDR); board: V-OBC-1 to -12 | Flatsat with the flight image, ≥ 500 h (VI-11, by 2027-09-28) | VI-03, VI-04, V-OBC-13, V-OBC-16 | — (no flight without it) |
| CE-6 AI selector | Implemented and validated in the SIL against the rules baseline (by 2027-06-30) | On the target in the flatsat (2027-07) | EM day-in-the-life rehearsal; FM day-in-the-life (2028-02) | Fly the rules baseline only |
| CE-7 Pack and mount | V-BSA-1 lot screening (before the EM pack) | V-BSA-2 trips; V-BSA-9 abuse tests (2027-06, task BATT_ABUSE) | V-THM-2 thermal balance, VI-03, VI-04 | Bought pack with flight heritage (mass and volume trade) |
| CE-8 Wings and hinge | VI-MECH-01 spring lot (before EM assembly) | VI-STR-04 hinge torque in vacuum, cold | EM deployments at the cold predicted temperature and minimum voltage before and after EM vibration (VI-LV-17, VI-STR-06) | Body-mounted array only: SAFE/IDLE power (−42 % ACTIVE margin with wings stowed); a PI decision on the mission (D4) |
| CE-9 AntS | AV-V7 ICD inspection (with the AntS RFQ, 2026-11-06) | EM deployment cold (VI-RF-16) | VI-MCAD-09, VI-RF-01 pattern | — (bought unit) |
| CE-10 ADCS | Gyro and magnetometer on the OBC EM (AV-V1, AV-V2) | V-OBC-16 gyro vibration and shock; AV-V4 coupling; flexible-tape Monte Carlo (DA-17, 2027-02-26) | AV-V6 HIL closed loop; VI-04 | BMI088 cold-spare gyro; magnetometer-only control (1·10⁻⁴ damping) |
| CE-11 GNSS | AV-V8 on the flatsat | V-EMC-2 L1 noise | VI-LV-13 end-to-end ephemeris | Third-party ephemeris (D6 contingency) |
| CE-12 Telecommand and TX-OFF | Authentication on the development board (HIL) | Flatsat with a station simulator | VI-RF-12 TX-OFF at the antenna port on the EM; TP-RF-006 | — (no flight without it) |

**Re-assessment points.** At the PDR (2027-01-12) the board reviews CE-1 and CE-3 against this plan (the tailoring's
PDR requirement) and the development-board HIL result for CE-5 and CE-12. At the CDR (2027-09-28) every element is
assessed against the TRL 6 exit criterion with the EM data; an element short of TRL 6 goes to the PI with its fallback
before the FM build (gate G3, `docs/project/PROJECT_PLAN.md` §6.1).

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-27 | Initial assessment (RID SE-09): twelve critical elements with TRL, evidence and gap per NPR 7123.1D Appendix E; TRL 6 targets at the CDR and the development plan tied to the EM schedule of SQSO-SCH-001 D | CDR round 3, systems and programmatics |
| B | 2026-10-27 | Documentation audit (post-CDR 2026): schedule cited at SQSO-SCH-001 E (dates unchanged) | Documentation audit |
