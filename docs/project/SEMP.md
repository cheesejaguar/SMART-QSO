# Systems Engineering Management Plan (SEMP)

**Document Number**: SMART-QSO-SEMP-001
**Version**: 2.2
**Date**: 2026-10-27
**Status**: Draft

**Reference framework**: NPR 7123.1D, NASA Systems Engineering Processes and Requirements (effective 2023-07-05),
tailored. It is not mandatory for this privately funded amateur mission (`docs/project/CDR_CRITERIA_MATRIX.md` §1).
Version 1.x cited "NPR 7123.1 Rev E", which does not exist (finding VV-11). `docs/BASELINE_PARAMETERS.md` Rev D
governs every value.

---

## 1. Introduction

### 1.1 Purpose

This plan defines how systems engineering is done on SMART-QSO: the technical processes and how they are tailored,
the reviews, the technical performance measures, integration and verification, and how requirements, interfaces and
configuration are controlled.

### 1.2 Scope and status

From Phase B (now, pre-PDR) to the end of Phase E. No hardware exists; the flight software is Rust on the Embassy
executor, tested in CI on the host and built for the STM32L4R5ZI (`software/rust/README.md`).

### 1.3 Design precedents

Version 1.0 said this plan "follows the NASA Ames Research Center Small Satellite Division approach" of GeneSat-1,
PharmaSat, O/OREOS and EcAMSat. The project has no affiliation with NASA Ames (finding PI-01); those missions are
design precedents only, surveyed with sources in `docs/project/HERITAGE_ASSESSMENT.md`.

## 2. Technical Approach

### 2.1 System architecture (BASELINE Rev D)

| Element | Baseline |
|---------|----------|
| Form factor | 1U, CDS Rev 14.1; COTS frame (buy, `docs/hardware/BUILD_VS_BUY.md`) |
| OBC | STM32L4R5ZI, Rust flight software on the Embassy executor, no RTOS (BASELINE §6; CR-2026-002) |
| AI payload | Jetson Orin Nano 8 GB module on a custom carrier, 7 W mode, ≤ 8 W input, bursts ≤ 120 s; flight baseline AI is the OBC-resident template selector (D5) |
| RF | Single-channel FM transponder, uplink 435-438 MHz (70 cm), downlink 145.800-146.000 MHz (2 m); 1200 bps AFSK AX.25 beacon; 0.5 W nominal, 1 W maximum; HMAC-authenticated telecommand with hardware TX-OFF (BASELINE §7). Version 1.0 said "UHF downlink, VHF optional": wrong |
| ADCS | Active: 3-axis magnetorquers, magnetometer, sun sensors, MEMS gyro (Epson M-G370PDT0); B-dot detumble; spin-stabilised coarse sun-pointing of +Y at 8°/s (BASELINE §8, D9). Version 1.0 said "passive magnetic": wrong |
| EPS | 16 triple-junction cells (body plus two deployable wings), 4S1P INR18650-35E ≈ 50 Wh charged to 4.10 V/cell (D8), custom EPS (ISIS ICEPS2 alternate) |
| GNSS | SkyFox piNAV-NG/FM, duty-cycled (D6) |
| Disposal | Passive drag, wings deployed; 465–485 km deployment, not above 485 km (BASELINE §3); FCC 5-year rule |

### 2.2 Key technical decisions

Decisions D1 to D9 (BASELINE §9) with their research in `docs/funding_audit/research/`, and the build-versus-buy
decisions in `docs/hardware/BUILD_VS_BUY.md`. A decision changes the baseline only through a change request
(`docs/project/CHANGE_REQUEST_LOG.md`).

## 3. Systems Engineering Processes

### 3.1 NPR 7123.1D common technical processes, tailored for a Class D 1U CubeSat

| Process | Application | Tailoring and products |
|---------|-------------|------------------------|
| Stakeholder expectations | Full | Amateur community, licensee, FCC/IARU, integrator; `mission/CONOPS.md`, mission success criteria |
| Technical requirements definition | Full | MRD (SQSO-REQ-001), SRS system (SQSO-REQ-002), software SRS; flow-down from CDS Rev 14.1, Part 97, NASA-STD-8719.14C and the provider (MRD Rev D, SYS Rev E) |
| Logical decomposition | Tailored | Functional allocation in `docs/ARCHITECTURE.md`; no separate functional-analysis document |
| Design solution definition | Full | Subsystem designs under `hardware/` and `docs/hardware/`; trades in BUILD_VS_BUY and the D-series research |
| Product implementation | Full | Buy (frame, panels, AntS, sensors) and build (OBC, EPS, transponder, carrier, battery mount) |
| Product integration | Full | Flatsat → EM → FM (`docs/integration/INTEGRATION_PLAN.md` Rev B) |
| Product verification | Full | Protoflight (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C); VCRM generated for every requirement |
| Product validation | Full | Day-in-the-life, end-to-end, SIL mission simulation, commissioning |
| Product transition | Tailored | Delivery to the integrator; data package (SYS-906) |
| Technical planning | Full | This plan; `mission/SCHEDULE.md` |
| Requirements management | Full | Specifications in Git; RTM generated (`scripts/gen_rtm.py`); traceability checks in CI |
| Interface management | Full | ICDs (§9); integrator ICD at contract |
| Technical risk management | Full | `mission/RISK_REGISTER.md`, 5 × 5, monthly |
| Configuration management | Full | `docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` |
| Technical data management | Tailored | Git; `docs/integration/TEST_DATA_MANAGEMENT.md` |
| Technical assessment | Full | TPMs (§3.3); independent PDR and CDR |
| Decision analysis | Tailored | Decision records in BASELINE §9 with research |

### 3.2 Technical reviews

Tailored life cycle and products per review: `docs/project/CDR_CRITERIA_MATRIX.md` §2. Summary: independent PDR
(NPR 7123.1D Table G-6) on 2027-01-12; independent CDR (Table G-7) on 2027-09-28 after the EM tests; test readiness
before the protoflight campaign (2028-02); pre-ship review (2028-05-19 earliest); post-launch assessment. A combined PDR/CDR is
not used (§2.2 of that document).

### 3.3 Technical performance measures

Updated 2026-10-27. Each value cites its controlled analysis; "not measured" until hardware exists.

| TPM | Requirement | Current value | Margin | Status |
|-----|-------------|---------------|--------|--------|
| Total mass (predicted) | ≤ 2.00 kg (SYS-002) | CBE 1.51 kg, MEV 1.75 kg with the CAD (`docs/environment/MASS_BUDGET.md` v3.1; BASELINE Rev E (amended) §4) | +14.4 % | Met by analysis (≥ 9 % wanted at PDR) |
| Orbit-average generation, EOL, tumbling | ≥ 2.21 W (SYS-100 Rev G: 1.84 W × 1.2, CR-2026-009) | 2.48 W delivered (mission simulation, `docs/environment/POWER_BUDGET_ANALYSIS.md` v2.2 §4); 2.14 W isotropic bound | +12 % over the requirement (+35 % over the load) | Met by analysis; V-PWR-1 |
| Orbit-average generation, sun-pointed | — | 3.92 W (same source) | +75 % on the 2.24 W load (BASELINE §5) | Met by analysis |
| Beacon link margin at 5° elevation, 0.5 W | > 6 dB | +7.7 dB with a 12 dBic station (`docs/environment/LINK_BUDGET_ANALYSIS.md` v2.0) | +1.7 dB over the requirement | Met by analysis |
| Beacon interval, ACTIVE | ≤ 60 s (MRD-020) | 60 s (`software/flight/include/beacon.h`, CI parameter check) | 0 | Met by design |
| Energy per accepted AI beacon (gated experiment) | ≤ 0.1 Wh p95 (D5) | Not measured | — | Verification item VI-02 (EM) |
| Jetson spreader temperature, 600 s batch, hot case | ≤ +40 °C (SYS-603 Rev G, CR-2026-010) | 39.0 °C at the audit's 0.44 W/K; 35.3 °C at 1.0 W/K; design path 1.10 W/K (`docs/environment/THERMAL_ANALYSIS.md` §6.3) | 1.0 °C at 0.44 W/K; about 5 °C at the design | Predicted to comply; VI-01, V-THM-3 |
| Sun-pointing (+Y within 15° of the Sun) | ≥ 90 % of sunlit time (SYS-422) | 100 % in the Monte Carlo of the flight code (BASELINE §8) | +10 points | Met by analysis |
| Detumble from 10°/s | ≤ 3 orbits (SYS-420) | Monte Carlo of the flight code (BUILD_VS_BUY §6) | — | Met by analysis; HIL pending |
| Schedule float to the integrator's L − 4 month milestone | ≥ GOLD Rule 1.06 margin (9.6 weeks for November 2028) | −11.4 weeks for June 2028; +10.1 weeks for November 2028 (`mission/SCHEDULE.md` Rev D §3) | +0.5 weeks for November 2028 | Met for the planning launch with two layout designers (DA-04) |

Version 1.x listed a "current" beacon interval of 30 s and an AI latency TPM of 10 s; both were superseded by the
baseline.

## 4. Integration Strategy

Flatsat first, then the EM (flight-design boards in a COTS frame), then the FM; functional test after every
integration step; safe-to-mate checks before every first connection (GEVS-B Appendix B §2.3.1). Sequence and dates:
`docs/integration/INTEGRATION_PLAN.md` Rev B. Integration order of the FM: structure and rails; EPS and battery
(isolated mount); OBC; ADCS sensors and magnetorquers; GNSS; RF and transponder; Jetson carrier; solar panels and
wings; antennas stowed; final assembly; mass properties.

## 5. Verification and Validation

### 5.1 Verification approach

Every requirement has a method (A, D, I, T) and a planned event in the VCRM (`docs/verification/VERIFICATION_MATRIX.md`
Rev C, generated). Verification status changes only with committed evidence.

### 5.2 Protoflight testing

Protoflight means qualification levels for acceptance durations (GSFC-STD-7000B Table 2.2-2). Version 1.0 said
"Protoflight (3 dB above qual)" for random vibration: wrong (protoflight is the qualification level, MPE + 3 dB).

| Test | Level | Duration |
|------|-------|----------|
| Random vibration (FM) | Envelope of MPE + 3 dB and GEVS workmanship: 8.18 Grms on the interim SpaceX MPE | 1 min/axis |
| Random vibration (EM pathfinder) | Same | 2 min/axis |
| Thermal vacuum (FM) | AFT ± 10 °C, capped by the battery at +60 °C; bake-out +60 °C for 6 h | 8 cycles (+ 4 board cycles) |
| Functional | Comprehensive performance test at the first and last hot and cold plateaus | Per procedure |

Full plan: `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C.

### 5.3 Validation approach

1. End-to-end over the air with an amateur ground station (TP-GS-001).
2. Seven-day day-in-the-life without contact (MRD-002).
3. Mission simulation in the loop: the SIL scenarios on the host, then driving the flight image on the target (HIL).
4. AI selector: beacon text and transponder schedule judged against the proposed MRD-043 fairness measure in the SIL.

## 6. Risk Management

`mission/RISK_REGISTER.md` is the only risk list. Technical risks to watch at this phase: Jetson power and heat in 1U
(R1, R7, R16), EPS margin (R3), deployment (R5), EMC between the Jetson, PA, GNSS and magnetometer (R6); programmatic:
funding timing, key person, launch capacity, regulatory timing (R11-R19).

## 7. Configuration Management

`docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` v1.1.

| Baseline | Established at | Contents |
|----------|----------------|----------|
| Functional | PDR (2027-01-12) | BASELINE parameters, MRD, system requirements, ConOps |
| Allocated | CDR (2027-09-28) | Software requirements, design, ICDs, build-to package |
| Product | Pre-ship review (2028-05-19 earliest) | As-built configuration, flight image, test reports |

(Version 1.0 placed the functional baseline at the PDR and the allocated at the CDR in one table and at the SRR/PDR in
another; this table and the CM plan now agree.)

## 8. Technical Planning

WBS: the NASA standard project WBS of the cost ROM (`docs/project/PROJECT_PLAN.md` §5). Schedule:
`mission/SCHEDULE.md`.

## 9. Interface Management

| Interface | Document |
|-----------|----------|
| Master list | SQSO-ICD-000 (`docs/interfaces/INTERFACE_CONTROL_MASTER.md`) |
| OBC-EPS | SMART-QSO-ICD-001 (`docs/interfaces/ICD_OBC_EPS.md`) |
| OBC-Jetson | SMART-QSO-ICD-002 (`docs/interfaces/ICD_OBC_JETSON.md`) |
| OBC-RF | SQSO-ICD-003 (`docs/interfaces/ICD_OBC_RF.md`) |
| OBC-ADCS | SQSO-ICD-004 (`docs/interfaces/ICD_OBC_ADCS.md`) |
| Space-ground | SQSO-ICD-005 (`docs/interfaces/ICD_SPACE_GROUND.md`) |
| Beacon format | SMART-QSO-ICD-006 (`docs/interfaces/ICD_BEACON_FORMAT.md`) |
| Telecommand authentication | SQSO-ICD-013 (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`) |
| Spacecraft-dispenser | The integrator's ICD at contract (D7); `docs/compliance/LAUNCH_PROVIDER_ICD.md` is design intent only |

Interface changes go through the change request log; interface verification is planned in
`docs/interfaces/INTERFACE_VERIFICATION_PLAN.md`.

## 10. SE Metrics

| Metric | Target | Frequency | Today |
|--------|--------|-----------|-------|
| Requirements with a planned verification event | 100 % | Each change (generated) | 100 % |
| Requirements volatility | < 10 % per quarter after the PDR | Monthly | Rebaseline in this review (CR-2026-007) |
| TBD/TBR closure | Each item with a dated closure; none open at the CDR that blocks fabrication | Monthly | Counted by `scripts/tbd_register.py` |
| RID closure | All severity 1 and 2 before the next gate | Per review | See `docs/project/CDR_CRITERIA_MATRIX.md` §6 |
| TPM status | All meet requirement at the CDR | Monthly | Three not met (§3.3) |
| Schedule float | ≥ GOLD Rule 1.06 margin | Monthly | −11.4 weeks (June 2028); +10.1 against 9.6 weeks (November 2028; SQSO-SCH-001 D) |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | Funding audit | TPM table cites the controlled analyses (link margin: TB-05; mass: TB-06; power generation: TB-14) |
| 2.0 | 2026-10-27 | CDR review, PM/SE/V&V | Reference framework NPR 7123.1D (not "Rev E"), not mandatory; NASA Ames "heritage approach" claim removed; architecture to BASELINE Rev D (STM32L4R5ZI and Embassy, not Apollo4; active ADCS, not passive; VHF downlink and UHF uplink, not "UHF downlink"); NPR 7123.1D process tailoring; reviews per the CDR criteria matrix; TPMs re-stated against requirements with margins (three not met); protoflight definition corrected; baselines aligned with the CM plan; interface list with document IDs; dated metrics |
| 2.1 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: ADCS spin 8°/s (BASELINE Rev E §8); disposal altitude 465–485 km (BASELINE Rev E §3) |
| 2.2 | 2026-10-27 | CDR round 3, systems and programmatics | §3.3 TPMs at the round-2 values: mass +14.4 %, SYS-100 2.21 W against 2.48 W (+12 %), sun-pointed +75 %, the Jetson heat path as the SYS-603 Rev G spreader temperature (RIDs SE-03, SE-07, P3P-06) |
