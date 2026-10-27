# Environmental Test Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ETP-001
**Revision:** C
**Date:** 2026-10-27
**Reference:** GSFC-STD-7000B (GEVS, 2021-04-28) including Appendix B (CubeSats); SpaceX Rideshare Payload User's Guide
v11 (August 2026, "UG v11"); Cal Poly CubeSat Design Specification Rev 14.1 §3; LSP-REQ-317.01 Rev B (CSLI path only)
**Status:** Plan. No test article exists and no test has been run. Levels are interim until the integrator ICD is
signed (decision D7, by 2027-06-30).

---

## 1. Purpose and Scope

This plan defines the verification test program of the SMART-QSO 1U CubeSat, a Class D payload (NPR 8705.4B as a
reference framework) built on a **protoflight** approach: one engineering model (EM) for development tests and one
flight model (FM) tested to protoflight levels and flown. It sets the test levels and durations with their sources,
the test articles, the test flow, the facilities and their cost, and the verification items that close review item
discrepancies needing hardware. Each level is computed by `software/simulation/cdr/pm/test_levels.py`, and the cost by
`software/simulation/cdr/pm/test_cost.py`.

## 2. Test Philosophy

### 2.1 Definitions (corrected from Revision B)

Revision B defined protoflight as "qualification duration at acceptance levels". That is backwards. GSFC-STD-7000B
Table 2.2-2 and CDS Rev 14.1 §3.5.2 use:

| Approach | Levels | Durations | Article |
|----------|--------|-----------|---------|
| Qualification (prototype) | Limit (MPE) + 3 dB random; 1.25 × limit sine; MPE ± 10 °C thermal | 2 min/axis random | A dedicated unit, not flown |
| **Protoflight (this mission)** | **Qualification levels** | **Acceptance durations** (1 min/axis random) | **The flight unit** |
| Acceptance | Limit level (MPE), at least the workmanship level | 1 min/axis random | Flight units of a qualified design |

GEVS-B Table 2.2-2 note 2: "As a minimum, the test level shall be equal to or greater than the workmanship level."
GEVS-B Appendix B §2.4.2.3: "The cubesat protoflight vibration environment should therefore be derived as the envelope
of limit level + 3dB and minimum workmanship."

### 2.2 Source precedence

1. The **signed integrator ICD** and its referenced launch-vehicle guide, when they exist (CDS Rev 14.1 §3: "The Launch
   Provider testing requirements will supersede testing environments from any other source").
2. Until then, **SpaceX UG v11** as the interim provider (Transporter-class baseline, BASELINE §3). Its Table 6-2 sets
   the minimum: random vibration at MPE for 1 min/axis on the flight unit, power-inhibit verification during random
   vibration, and advises shock, EMC and thermal cycling.
3. **GSFC-STD-7000B** where it is more stringent or where the provider is silent. This plan adopts the GEVS-B
   protoflight level (MPE + 3 dB, enveloped with workmanship) rather than the SpaceX minimum (MPE alone), because MPE
   alone is 1.7 dB below the GEVS workmanship screen (5.57 Grms against 6.78 Grms) and SpaceX itself calls testing a
   bare CubeSat at the §4.1.6 MPE an under-test (UG v11 Table 6-2 note 1).
4. **Backup and contingency paths**: on the NASA CSLI path, LSP-REQ-317.01 Rev B Table 1 governs (protoflight random
   MPE + 3 dB for **2 minutes** per axis; bake-out ≥ 70 °C for 3 h); on the ISS path, the Voyager NRCSD IDD Rev A and
   JSC 20793 Rev D (battery). Requirements SYS-910 and SYS-911 carry these conditionally.

### 2.3 Test articles

| Article | Build | Purpose | Flies |
|---------|-------|---------|-------|
| Flatsat | Development boards and EM boards on a bench | Flight software on the STM32L4R5 target, interface and HIL tests (`software/flight/tests/hil/HIL_TEST_PLAN.md`) | No |
| **EM** | Flight-design boards (EM lot), COTS frame and AntS, EM Jetson carrier, battery of a non-flight lot, EM solar panels | Development tests that retire design risk before the CDR (§6); pathfinder for every protoflight procedure | No |
| **FM (protoflight unit)** | Flight boards, flight-lot screened cells, flight panels | Protoflight test campaign (§4), then delivery | Yes |

No qualification unit is planned (GEVS-B Appendix B §2.4: "a structural qualification unit is not likely ... Most
cubesat projects that are launching a single cubesat will want to follow a protoflight model").

## 3. Test Levels

### 3.1 Random vibration [SYS-700, SYS-703, SYS-704]

Protoflight level = envelope of the UG v11 Table 4-6 MPE + 3 dB and the GEVS-B Table 2.4-4 minimum workmanship
spectrum, 1 minute in each of three axes, in the integrator's test pod with the deployment switches actuated and the
RBF pin removed (flight-like inhibit configuration, UG v11 §6.7.7).

| Frequency (Hz) | ASD (g²/Hz) | Slope to next point |
|----------------|-------------|---------------------|
| 20 | 0.0200 | +1.33 dB/oct |
| 50 | 0.0299 | flat |
| 59.9 | 0.0299 | +3.01 dB/oct |
| 80 | 0.0400 | flat |
| 500 | 0.0400 | −3.01 dB/oct |
| 668.2 | 0.0299 | flat |
| 700 | 0.0299 | +15.6 dB/oct |
| 800 | 0.0599 | flat |
| 925 | 0.0599 | −6.01 dB/oct |
| 2000 | 0.0128 | — |
| **Overall** | **8.18 Grms** | |

| Spectrum (20-2000 Hz) | Overall | Use |
|-----------------------|---------|-----|
| UG v11 MPE (Table 4-6) | 5.57 Grms | SpaceX minimum for the flight unit |
| GEVS-B minimum workmanship (Table 2.4-4) | 6.78 Grms (GEVS rounds to 6.8) | Floor for any level |
| MPE + 3 dB | 7.86 Grms | GEVS-B protoflight level |
| **Envelope of MPE + 3 dB and workmanship** | **8.18 Grms** | **This plan, FM 1 min/axis; EM 2 min/axis** |
| LSP-REQ-317.01 Rev B protoflight | 7.86 Grms, 2 min/axis | CSLI path only |
| GEVS-B Table 2.4-3 component qualification | 14.14 Grms | Not used (component level, ELV generic) |

The envelope is +3.3 dB over the MPE. For fatigue with an S-N exponent of 4 to 8, one minute at the envelope does 4.7 to
22 times the damage of one minute at the MPE, which is why the EM runs the envelope for 2 minutes per axis first (§6)
and the structural analysis must show positive margins at the envelope with the GEVS-B Table 2.2-3 random factors
(1.6 yield, 1.8 ultimate on 3σ peaks). Tolerances: ASD ± 3 dB, overall ± 10 % (GEVS-B §2.4.2.5 a). Notching only to
the quasi-static load, primary mode only, at most 10 dB, reported to the integrator (UG v11 §6.7.6). **When the
integrator supplies dispenser-derived levels, the protoflight level becomes the envelope of those levels + 3 dB and the
workmanship spectrum** (SYS-902).

### 3.2 Low-level sine signature and natural frequency [SYS-705]

0.1 g minimum (0.25 g if the fixture allows), 5-2000 Hz, 3 oct/min or slower, each axis before and after the random
run (UG v11 §6.7.1, Table 6-3). Pass: all modes above 40 Hz (or the integrator's limit), first-mode frequency shift
< 10 % and amplitude shift < 30 %; investigate any shift above 5 %. **No sine-vibration test**: it is not required for
a containerized CubeSat (UG v11 Table 6-2) and GEVS-B Appendix B §2.4.3 requires it only if the first mode is below 1.5
times the upper sine frequency of the launch vehicle. The structural analysis must confirm the first mode; the
integrator decides for a rail-style dispenser (rattle).

### 3.3 Shock [SYS-708]

Shock is "advised" by UG v11 Table 6-2 and "typically not required" by CDS Rev 14.1 §3.3.1.1. Verification is by
analysis: the 3σ random-vibe-equivalent shock response spectrum (Q = 10) of the §3.1 envelope is 24 g at 100 Hz, 53 g
at 500 Hz, 85 g at 1000 Hz and 60 g at 2000 Hz (`test_levels.py`). By the LSP-REQ-317.01 Rev B Table 1 note 3
criterion, a shock test may be waived only where this exceeds the qualification shock spectrum below 2000 Hz.
Dispenser-ejection shocks are typically higher than this at 1-2 kHz, so the waiver is **not assumed**: verification item
VI-07 (§8) obtains the integrator's shock MPE and decides between analysis and a test (2 actuations per axis at MPE +
3 dB, UG v11 Table 6-2). Self-induced shock from the burn-wire releases and the wing hard stops is covered by the
deployment tests (§3.8).

### 3.4 Thermal vacuum and thermal cycling [SYS-600, SYS-601, SYS-602, SYS-702, SYS-707]

GEVS-B §2.6.3.2.1.1 sets the protoflight test margin at no less than 10 °C beyond the allowable flight temperatures
(AFT), and at the system level at least 10 °C beyond the margined predictions. The correlated thermal model does not
exist yet (finding TB-07; `docs/environment/THERMAL_ANALYSIS.md` is the unverified January 2026 issue), so the interim
levels are set from the requirement and part limits and capped by the battery:

| Plateau | GEVS-B rule | Uncapped value | **Interim test value** | Limiting item |
|---------|-------------|----------------|------------------------|---------------|
| Hot operational | SYS-600 max + 10 °C | +60 °C | **+60 °C** | Battery non-operating maximum +60 °C; charging inhibited above +45 °C (SYS-602), which the test exercises |
| Cold operational | SYS-600 min − 10 °C | −30 °C | **−30 °C** | — |
| Hot non-operating (survival) | SYS-601 max + 10 °C | +70 °C | **+60 °C** (cap) | Battery; **deviation from GEVS of 10 °C**, recorded in §9 |
| Cold non-operating (survival) | SYS-601 min − 10 °C | −50 °C | **−40 °C at the structure, battery heater enabled, battery held above −20 °C** | Battery; open under TBR-TPENV003-001 |
| Hot start / cold start | GEVS-B §2.6.3.3.1.5 | — | Start at +60 °C after bake-out; start at −30 °C after the cold non-operating plateau | — |

The SpaceX advised protoflight range (UG v11 Table 6-2: envelope of the maximum predicted temperatures and −24 to
+61 °C, ± 5 °C, so at least −29 to +66 °C) exceeds the battery limit by 6 °C at the hot end; it is advised, not
required, and the shortfall is recorded with the GEVS deviation. **When the correlated model gives the margined
predictions, each plateau becomes max(prediction + 10 °C, AFT ± 10 °C) and the battery cap is re-checked: the hot
plateau is compliant only if the hot-case prediction plus 10 °C is at or below +60 °C.**

| Parameter | Value | Source |
|-----------|-------|--------|
| TV cycles, FM | **8** | GEVS-B §2.6.3.2.2.1 "minimum of the 8 thermal-vacuum cycles required for qualification"; Table 2.6-1 |
| Board-level thermal cycles before integration, FM boards | 4, ambient pressure, −30/+60 °C | Brings the total to the GEVS 12-cycle workmanship goal (Table 2.6-1 note 1) |
| Total thermal cycles | 12 (8 in vacuum) | SpaceX advises 20 with ≥ 4 in vacuum: 8 further air cycles of the FM in a thermal chamber are added if the integrator asks (VI-08) |
| Operational plateau | Stabilised (98 % of sensors within 2 °C, or < 0.1 °C/h) + the performance test; ≥ 3 h | GEVS-B §2.6.3.2.3; Appendix B §2.6 ("not necessary for a cubesat to dwell for 24 hours") |
| Non-operating plateau | ≥ 2 h after stabilisation | GEVS-B §2.6.3.2.3 (unit level) |
| Comprehensive performance test (TP-SYS-001) | At the first and last hot and cold operational plateaus; limited functional at the others | GEVS-B Appendix B §2.3.2 |
| Jetson burst | One 120 s burst at each hot operational plateau, with the per-rail current and temperature limits armed | BASELINE §6; SYS-504 |
| Transition rate | ≤ 1 °C/min | GEVS-B Table 2.6-1 |
| Pressure | ≤ 1 × 10⁻⁵ Torr (1.33 × 10⁻³ Pa) | GEVS-B §2.6.3.2.4 |
| Thermal balance | Two steady-state points (hot operational with burst, cold safe mode) for model correlation | GEVS-B §2.6.4, Appendix B §2.6.3 |

Failure-free operating time: GEVS-B Appendix B §2.6.2 notes 100 h hot and 100 h cold reduce risk. The plan gives
about 24 h hot and 24 h cold in vacuum on the FM; the shortfall is accepted for Class D and offset by the 7-day
day-in-the-life (§3.10) and a 168-hour powered burn-in of the EM at +50 °C ambient (§6, VI-09).

Chamber time: 8 cycles of 90 °C span at 0.5 °C/min with 3 h plateaus, plus bake-out and set-up, is 5.9 chamber days
for the FM and 4.5 for the EM (`test_cost.py`).

### 3.5 Thermal-vacuum bake-out [SYS-706, SYS-712]

CDS Rev 14.1 §3.2.1 requires a bake-out and leaves the specification to the launch provider. Interim specification:
**+60 °C (+0/−2 °C) for at least 6 hours after thermal stabilisation at ≤ 1 × 10⁻⁴ Torr**, the LSP-REQ-317.01 Rev B
Table 1 note 5 alternative to 70 °C for 3 h, because 70 °C exceeds the battery's +60 °C limit (note 7: bake-out not
above the qualification temperatures). It runs as the outgassing phase at the start of the FM TVAC (GEVS-B
§2.6.3.3.1.1). A TQCM is not required if the time-at-temperature requirement is met (GEVS-B Appendix B §2.6.2). The
materials list with ASTM E595 data (TML ≤ 1.0 %, CVCM ≤ 0.1 %) is a pre-requisite. The integrator's specification
supersedes this.

### 3.6 Electromagnetic compatibility [SYS-203, SYS-713]

The spacecraft is unpowered in the dispenser, so launch-vehicle EMI is covered by the inhibits (SYS-085, SYS-704,
SYS-905); MIL-STD-461G RE102/CE102 at the spacecraft level are not required (GEVS-B Appendix B §2.5). Tests:

| Test | Article | Pass criterion | Source |
|------|---------|----------------|--------|
| Bus ripple at each power card and at the distribution point, all loads on | EM, FM | ≤ 200 mV peak-to-peak on the 14.4 V bus, 1 MHz bandwidth (GEVS recommendation for a 12 V bus, used unchanged) | GEVS-B Appendix B §2.5 |
| Emissions at the 435 MHz receiver input, every mode including a Jetson burst and magnetorquer PWM | EM, FM | ≥ 6 dB below the receiver's minimum usable signal from the link analysis | GEVS-B Appendix B §2.5; UG v11 Table 6-2 (6 dB EMISM) |
| RF self-compatibility, each transmitter at full power | EM, FM | No reset, no false telecommand, no sensor fault | GEVS-B Appendix B §2.5 |
| Spurious and harmonics, conducted (TP-RF-003) | EM, FM | ≥ 46 dBc and ≤ 25 µW at 1 W; third harmonic ≥ 60 dBc | 47 CFR 97.307(e); BASELINE §7 |
| GNSS L1 noise with Jetson, PA and magnetorquers on | EM | Receiver keeps a fix with C/N0 loss ≤ 3 dB | BASELINE §6 open item (D6) |
| Magnetometer with Jetson and magnetorquer currents | EM | Bias shift within the ADCS calibration budget | ADCS design |

### 3.7 Mass properties [SYS-002, SYS-003, SYS-901]

Mass to ± 1 g; centre of gravity in X, Y and Z to ± 1 mm by three-point load-cell or knife-edge measurement, against
CDS Rev 14.1 Table 2 (± 2 cm each axis) and the integrator's limit; moments of inertia by bifilar or trifilar pendulum
to ± 10 %, for the ADCS Monte Carlo re-run that decision D9 requires. Measured on the EM (pathfinder) and the FM before
and after the environmental campaign.

### 3.8 Deployment tests [SYS-220, SYS-221, SYS-709]

Both antennas and both solar wings, from the flight burn-wire circuit, at the minimum battery voltage of the end-of-life
state of charge, at ambient and at the hot (+60 °C) and cold (−30 °C) operational test temperatures, with the 30-minute
deployment timer (SYS-221) started from deployment-switch release. Pass: full deployment within the timer window,
deployment telemetry agrees with the observed state, burn time within the design value with at least 2× margin at
cold. The EM does at least five deployments per mechanism (life and re-stow practice).

**Order on the FM.** CDS Rev 14.1 §3.5.2 forbids disassembly or modification after protoflight testing, and §3.5.2.1
notes that some providers treat a deployment as a configuration change that invalidates the vibration test. The FM
deployment tests therefore run **before** the protoflight vibration (§4), followed by a re-stow per the vendor
procedure and an inspection. After vibration and TVAC, only burn-wire continuity and stowed-state telemetry are
checked, unless the integrator approves a post-environment deployment in writing (TP-ENV-005).

### 3.9 Magnetic properties [SYS-083]

Residual dipole of the FM, wings deployed and illuminated, in a Helmholtz cage: ≤ 3 mA·m² design, ≤ 10 mA·m² limit
(BASELINE §8); field ≤ 0.5 gauss above the Earth's field outside the envelope from any permanent magnet or magnetised
rod (CDS §2.1.8). If CubeSpace CR0002 rods are bought, their remanence is measured on the EM first.

### 3.10 End-to-end, day-in-the-life and mission simulation [MRD-002, MRD-022, MRD-030 to MRD-034, SYS-204, SYS-325]

| Test | Article | Content |
|------|---------|---------|
| End-to-end RF (TP-GS-001, `docs/test/END_TO_END_TEST_PLAN.md`) | FM | Over the air through attenuators or an outdoor range: beacon decode by a standard amateur station, authenticated telecommand, TX-OFF and recovery, transponder repeat path, loss-of-contact inhibit (time-accelerated) |
| Day in the life (TP-SYS-002, TP-DITL-001) | FM | 7 days without ground contact (MRD-002), solar-array simulator driven by the SIL orbit profile, eclipse and SOC mode transitions, Jetson bursts, fault injection with the AI off |
| Mission simulation, hardware in the loop | Flatsat then EM | The SIL commissioning scenario (`docs/test/SIL_MISSION_SIMULATION.md`) driving the flight image on the STM32L4R5 through simulated sensors (`software/flight/tests/hil/HIL_TEST_PLAN.md`); detumble and spin sun-pointing closed loop (SYS-420, SYS-422) |

## 4. Protoflight Test Flow (FM)

| Step | Activity | Procedure | Duration |
|------|----------|-----------|----------|
| 1 | Comprehensive performance test (baseline), RF characterisation | TP-SYS-001, TP-RF-001 to -004, TP-ENV-001 | 3 days |
| 2 | Mass properties; residual dipole | TP-ENV-001; §3.9 | 2 days |
| 3 | Deployment tests ambient, hot and cold (thermal chamber); re-stow; inspection | TP-ENV-005, TP-MECH-002, TP-MECH-003 | 3 days |
| 4 | Fit check in the integrator's test pod | TP-MECH-001 | 0.5 day |
| 5 | Random vibration, three axes, with sine signatures and inhibit monitoring | TP-ENV-002 | 2 days |
| 6 | Post-vibration limited functional, inspection | TP-SYS-001 (limited) | 1 day |
| 7 | TVAC: pump-down, bake-out, hot start, 8 cycles, thermal balance, cold start | TP-ENV-003 | 6 days |
| 8 | EMC and RF self-compatibility, spurious | TP-ENV-004, TP-RF-003 | 2 days |
| 9 | Final comprehensive performance test, burn-wire continuity, mass, photographs | TP-SYS-001, TP-FAT-001, TP-FAT-002 | 3 days |
| 10 | Configuration freeze (SYS-089); data package | TP-FAT-002 | — |
| | **Total with transport (planned 7 weeks in `mission/SCHEDULE.md`)** | | **22.5 working days** |

The day-in-the-life and end-to-end tests (§3.10) run before step 1 (schedule event DITL).

## 5. Test Matrix: Engineering Model versus Flight Model

| Test | EM | FM | Requirement |
|------|----|----|-------------|
| Random vibration | Envelope, **2 min/axis** (pathfinder; fixture and response survey) | Envelope, 1 min/axis | SYS-703 |
| Inhibits during vibration | Yes (rehearsal) | Yes | SYS-704 |
| Sine signature | Yes | Yes | SYS-705 |
| Shock | Only if VI-07 requires it (EM, then accept FM by similarity) | No | SYS-708 |
| TVAC cycles | 4, plus thermal balance | 8 | SYS-707 |
| Bake-out | No | Yes | SYS-706 |
| Jetson heat path and burst energy | **Yes** (VI-01, VI-02) | Burst at hot plateau | SYS-603, SYS-512 |
| Deployment, ambient, hot, cold | ≥ 5 per mechanism | Once each, before vibe | SYS-709 |
| EMC, receiver-input emissions, GNSS noise | Yes | Receiver-input and spurious | SYS-713, SYS-203 |
| Antenna pattern | Yes (VI-05) | No | SYS-222 |
| Mass properties, inertia | Yes | Yes | SYS-002, SYS-003 |
| Residual dipole | Rods or coils | Yes | SYS-083 |
| Burn-in 168 h at +50 °C | Yes (VI-09) | No | SYS-010 |
| Day in the life, end-to-end | Rehearsal | Yes | MRD-002 |

## 6. Engineering-Model Development Tests (before the CDR)

These retire the design risks that the CDR needs answered and are scheduled as event EMT (2027-06 to 2027-08):
Jetson heat path and burst energy (VI-01, VI-02), EM TVAC with thermal balance (VI-03), pathfinder vibration (VI-04),
antenna pattern (VI-05), EMC pre-scan and GNSS noise (VI-06), burn-in (VI-09). Their pass criteria are in §8.

## 7. Facilities and Cost

No facility has been contacted. The options, with what the project can realistically use:

| Option | What it offers | Notes |
|--------|----------------|-------|
| Integrator-arranged testing (Exolaunch, ISISPACE) | Vibration in their own test pod at a partner lab; often bundled with the launch service | Ask in the RFQs due 2026-12-31 (D7); test-pod rental is in the ROM; foreign lab adds export handling |
| Commercial labs in California: Experior Laboratories (Oxnard), Kelly Space & Technology (San Bernardino), NTS (several sites), Element | Shakers, TVAC, EMC; accredited | Rates not published; request quotes for a 1U with the §4 flow |
| NASA Ames Research Center (prospective partner, BASELINE §1) | Environmental test facilities, under a reimbursable or non-reimbursable Space Act Agreement | Nothing agreed; a partnership also changes the governing framework (`docs/project/CDR_CRITERIA_MATRIX.md` §1) |
| Universities: Cal Poly CubeSat Laboratory, Santa Clara University (prospective partner), Stanford, UC Berkeley SSL | Shakers, small TVAC chambers, Helmholtz cages used for student CubeSats | Access by agreement; good for EM tests and the dipole measurement |
| Own bench vacuum chamber with heated and cooled platen | Bake-out practice, EM cycling, 12-cycle goal at low marginal cost | $8k-20k [U]; the integrator may still require an accredited lab for the FM bake-out |
| Published benchmark | Hawaii Space Flight Laboratory: TVAC and vibration at $3,500 a day each | Public rate card, used to size the ROM |

**Cost (bottom-up ROM, `test_cost.py`)**: EM tests $14.5k-38k; FM and common $21k-63k; total **$35k-101k**, against
$23k-78k in the lifecycle ROM WBS 10 (SQSO-AUD-005 Rev B). The ROM line covers the FM campaign only; it has no EM
environmental tests, Helmholtz-cage test or re-test allowance. The difference ($12k-23k) is reported to the chair; the
ROM is not edited here.

## 8. Verification Items (hardware closure of review items)

| VI | Test | Pass criterion | Verifies | Article, event |
|----|------|----------------|----------|----------------|
| VI-01 | Jetson heat-path conductance: steady 7 W into the module, measure module and structure temperatures in vacuum | ≥ 1 W/K module to structure | SYS-603, BASELINE §6 | EM, EMT (2027-06) |
| VI-02 | Energy per accepted beacon at the carrier input, boot to power-off, ≥ 30 bursts | p95 ≤ 0.1 Wh (360 J); burst ≤ 120 s | SYS-512, D5 gate | EM, EMT |
| VI-03 | EM TVAC, 4 cycles −30/+60 °C, two thermal-balance points | Model correlated within ± 5 °C at the balance points; all functions at each plateau | SYS-600, SYS-601 (closes TBR-TPENV003-001), SYS-504 | EM, EMT |
| VI-04 | EM random vibration at the §3.1 envelope, 2 min/axis, in the test pod | Sine-signature shifts within §3.2; no structural or workmanship failure; inhibits hold | SYS-703, SYS-704, SYS-705 | EM, EMT |
| VI-05 | Antenna pattern of the deployed EM, two orthogonal planes, both bands | Agrees with the model within 2 dB; gain ≥ −6 dBi over ≥ 90 % of the sphere (proposed SYS-222 wording) | SYS-222, SYS-223 | EM, EMT |
| VI-06 | EMC pre-scan and GNSS noise (§3.6) | §3.6 criteria | SYS-713, SYS-203 | EM, EMT |
| VI-07 | Obtain the integrator's shock MPE; compare with the §3.3 equivalent SRS | Analysis shows the vibe-equivalent SRS envelops the shock MPE below 2000 Hz, or an EM shock test at MPE + 3 dB, 2 actuations per axis, passes | SYS-708 | Integrator ICD (CON), EM if needed |
| VI-08 | Integrator's thermal-cycle count | If 20 total is required: 8 further FM air cycles in a thermal chamber, −29/+60 °C | SYS-707, SYS-902 | FM, PFE |
| VI-09 | EM burn-in, 168 h powered at +50 °C ambient, cycling all modes | No failure; fault log clean | SYS-010, SYS-022 | EM, EMT |
| VI-10 | FM protoflight campaign (§4) | Each procedure's criteria | SYS-700 to SYS-713 | FM, PFE (2028-01 to 2028-02) |

## 9. Deviations and Tailoring Record

| # | Standard and clause | This plan | Rationale | Approval |
|---|---------------------|-----------|-----------|----------|
| 1 | GSFC-STD-7000B §2.6.3.2.1.1: 10 °C beyond AFT | Hot non-operating plateau +60 °C instead of +70 °C | Battery limit; the hot plateau is compliant once the margined prediction is ≤ +50 °C | PI (to record as a CR) |
| 2 | GSFC-STD-7000B Table 2.6-1: 12 cycles before flight | 8 TV + 4 board-level air cycles | Same total; no unit-level TV for a 1U | PI |
| 3 | GSFC-STD-7000B Appendix B §2.6.2: 100 h hot and cold failure-free | About 24 h each in vacuum | Class D; offset by DITL and EM burn-in | PI |
| 4 | SpaceX UG v11 Table 6-2 (advised): protoflight TV at −29/+66 °C, 20 cycles | −30/+60 °C, 12 cycles (VI-08 adds cycles if required) | Battery limit; advised only | Integrator |
| 5 | SpaceX UG v11 Table 6-2: random vibe at MPE | Envelope at 8.18 Grms, more severe | GEVS-B Appendix B §2.4.2.3; SpaceX note 1 under-test | None needed (exceeds the minimum) |
| 6 | LSP-REQ-317.01 Rev B Table 1: bake-out 70 °C for 3 h | 60 °C for 6 h | Note 5 alternative; battery | Integrator (CSLI: LSP) |

## 10. Data and Success Criteria

Each test records the input and response data, the as-run procedure, photographs and the before and after
performance results, archived under `docs/integration/TEST_DATA_MANAGEMENT.md`. Success: every procedure's pass criteria
met, every anomaly dispositioned through `docs/integration/ANOMALY_RESOLUTION.md`, and the VCRM rows of the verified
requirements updated with the test report. A failure on the FM triggers root cause, repair and the re-test that the
integrator and GEVS-B §2.4.2.8 require; the schedule holds 2.7 weeks of float for it on a 2028-06-01 launch
(`mission/SCHEDULE.md`), which is less than the GOLD-rule margin.

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §3.1 vibration levels point to the interim SpaceX UG v11 levels of TP-ENV-002 (5.57 Grms MPE), replacing the unsourced 6.8 / 5.1 Grms profile (finding LA-05); §3.2 cold survival marked open against SYS-601 (TBR-TPENV003-001) | SMART-QSO Team |
| C | 2026-10-27 | CDR review (PM/SE/V&V). Rewritten: protoflight defined correctly (GEVS-B Table 2.2-2); source precedence; EM and FM articles; random vibration at the GEVS-B Appendix B envelope of MPE + 3 dB and workmanship (8.18 Grms); sine signature; shock by analysis with VI-07; TVAC levels with GEVS margins and the battery cap, 8 cycles plus 4 board cycles; bake-out +60 °C 6 h; EMC tailored per GEVS-B Appendix B; mass properties with inertia; deployment tests hot and cold before vibration (CDS §3.5.2); magnetic; end-to-end, DITL and HIL; FM flow; EM/FM matrix; facilities and cost; verification items VI-01 to VI-10; deviations. Scripts `software/simulation/cdr/pm/test_levels.py`, `test_cost.py` | CDR review, PM/SE/V&V |
