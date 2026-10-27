# System Requirements Specification
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REQ-002
**Revision:** G
**Date:** 2026-10-27
**Reference:** NPR 7123.1D (tailored, `docs/project/CDR_CRITERIA_MATRIX.md` §2); `docs/BASELINE_PARAMETERS.md` Rev E (amended)

> **Change status (Rev G).** The Rev E, F and G changes are Class I and are carried by CR-2026-009 and CR-2026-010
> (`docs/project/CHANGE_REQUEST_LOG.md`), proposed by the CDR 2026 design review. **PI decision required by 2027-01-12**
> (`mission/SCHEDULE.md` DA-01); until then the requirement text below is the proposed text.

---

## 1. Purpose

This document defines the Level 2 (System) requirements for the SMART-QSO CubeSat. These requirements are derived from the Mission Requirements Document (MRD) and flow down to subsystem and software requirements.

## 2. Scope

This document covers:
- Functional requirements
- Performance requirements
- Interface requirements
- Environmental requirements
- Verification approach

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-REQ-001 | Mission Requirements Document |
| SQSO-BL-001 Rev E (amended) | Baseline Parameters (governs every value) |
| NPR 7123.1D | NASA Systems Engineering Processes and Requirements (reference framework, tailored) |
| Cal Poly CDS Rev 14.1 | CubeSat Design Specification (February 2022) |
| GSFC-STD-7000B | General Environmental Verification Standard, including Appendix B (CubeSats) |
| SpaceX Rideshare Payload User's Guide v11 | Interim launch-provider requirements (August 2026) until the integrator ICD is signed |
| LSP-REQ-317.01 Rev B | NASA LSP dispenser and CubeSat requirements (applies only on the CSLI path) |
| 47 CFR Part 97 | FCC Amateur Radio Service |

---

## 4. System Overview

### 4.1 System Description

SMART-QSO is a 1U CubeSat consisting of:
- Electrical Power Subsystem (EPS)
- Command & Data Handling (C&DH/OBC)
- Communication Subsystem (RF)
- Attitude Determination & Control (ADCS)
- AI Payload (Jetson Orin Nano)
- Structure and Thermal

### 4.2 Operational Modes

| Mode | Description | Power |
|------|-------------|-------|
| SAFE | Minimum operations, beacon only | <1W |
| IDLE | Normal standby, monitoring | <2W |
| ACTIVE | Full operations, no AI | <4W |
| BURST | AI inference active | <10W |

---

## 5. General Requirements

### 5.1 Physical Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-001 | The spacecraft shall conform to 1U CubeSat form factor per Cal Poly CDS Rev 14.1 | MRD-080 | I,T |
| SYS-002 | The spacecraft mass shall not exceed 2.0 kg (CDS Rev 14.1 Table 1), or the integrator's dispenser limit if lower *(dispenser clause added at the CDR review, 2026-10-27; BASELINE §4)* | MRD-080 | T |
| SYS-003 | The spacecraft center of gravity shall be within 2 cm of the geometric center along each of the X, Y and Z axes (CDS Rev 14.1 §2.2.11, Table 2) *(per-axis wording added at the CDR review, 2026-10-27)* | MRD-080 | T |
| SYS-004 | The spacecraft shall have deployment switches per CDS requirements | MRD-080 | I,T |
| SYS-005 | The spacecraft shall have a Remove Before Flight (RBF) pin | MRD-080 | I |

### 5.1a CubeSat Design Specification Requirements (CDS Rev 14.1)

Added at the CDR review (2026-10-27) to flow down the CDS clauses that had no system requirement (finding VV-14).

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-080 | The spacecraft dimensions, rails and standoffs shall conform to the 1U drawing of CDS Rev 14.1 Appendix B: rails at least 8.5 mm wide (§2.2.5), rail ends at least 6.5 mm × 6.5 mm (§2.2.8), at least 75 % rail contact (§2.2.9), no protrusion beyond the allowed envelope on the shaded faces (§2.2.3); rail surface roughness below 1.6 µm and rail edges rounded to at least 1 mm (§2.2.6, §2.2.7, recommended) | MRD-080 | I |
| SYS-081 | Deployables shall be constrained by the spacecraft, not by the dispenser (CDS §2.2.4), and all parts shall remain attached during launch, ejection and operation (CDS §2.1.1) | MRD-080 | I,T |
| SYS-082 | The spacecraft shall carry no pyrotechnics, propulsion or pressure vessel; any hazardous material shall conform to AFSPCMAN 91-710 Volume 3 (CDS §2.1.2, §2.1.3, §2.1.6) | MRD-080 | I |
| SYS-083 | The magnetic field of any passive magnet shall be no more than 0.5 gauss above the Earth's field outside the static envelope (CDS §2.1.8) | MRD-080 | T |
| SYS-084 | The spacecraft shall be designed for ascent venting (CDS §2.1.9); vent area at least 0.05 in² per ft³ of enclosed volume (GSFC-STD-7000B Appendix B §2.4.6) | MRD-080 | A |
| SYS-085 | The power system shall be in the power-off state from delivery to the launch vehicle until on-orbit deployment (CDS §2.3.1); at least one deployment switch shall disconnect the power system from all powered functions while actuated (CDS §2.3.2 to §2.3.2.3) | MRD-080 | I,T |
| SYS-086 | Any real-time clock shall be isolated from the main power system, run below 320 kHz and be current-limited below 10 mA (CDS §2.3.3) | MRD-080 | I,T |
| SYS-087 | The RBF pin and every umbilical connector shall be within the dispenser's access-port locations if it has them, and the RBF pin shall cut all power and protrude no more than 6.5 mm from the rail surface when inserted (CDS §2.3.4, §2.3.5) | MRD-080 | I |
| SYS-088 | The battery shall have charge and discharge protection against unbalanced cell conditions, with manufacturer documentation or test data for the cells (CDS §2.3.6) | MRD-080 | I,T |
| SYS-089 | The flight unit shall not be disassembled or modified after protoflight testing without the integrator's written approval; a change after testing shall be re-verified as the integrator requires (CDS §3.5.2, §3.5.2.1) | MRD-098 | I |

### 5.2 Lifetime Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-010 | The spacecraft shall be designed for a 2-year operational life, with a minimum mission of 12 months (BASELINE §2) *(was a 12-month life until Rev D; CDR review 2026-10-27)* | MRD-001 | A |
| SYS-011 | All components shall be derated per EEE-INST-002 guidelines | MRD-001 | A |
| SYS-012 | The spacecraft shall survive 11,200 orbital thermal cycles (2 years at about 15.3 orbits a day at 465-485 km) *(was 10,000, below the 2-year count; CDR review 2026-10-27)* | MRD-001 | A |
| SYS-013 | The spacecraft shall survive the radiation environment for mission duration | MRD-001 | A |

### 5.3 Reliability Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-020 | Single-point failures shall be identified and mitigated where practical | MRD-001 | A |
| SYS-021 | The spacecraft shall implement watchdog protection on all processors | MRD-002 | D |
| SYS-022 | The spacecraft shall be capable of autonomous recovery from single-event upsets | MRD-002 | D |
| SYS-023 | Critical data shall be protected by CRC or equivalent integrity mechanism | MRD-002 | D |

**Proposed (PI to ratify), SYS-020, TBR-SYS-002:** "Each single-point failure found by the hardware and software FMEA shall be listed with either a mitigation or a retention rationale accepted by the PI. No single failure shall prevent TX-OFF from removing power from the transmitter's power amplifier." Verification: Inspection and Analysis. Inspection that every FMEA line is dispositioned in the single-point-failure list; analysis of the TX-OFF path. "Where practical" gives no pass criterion. A Class D mission (`docs/BASELINE_PARAMETERS.md` §1) may keep single-point failures it knows and accepts; the one it cannot keep is losing the ability to cease emissions (47 CFR 97.207(b); baseline §7).

---

## 6. Electrical Power Subsystem Requirements

### 6.1 Power Generation

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-100 | The EPS shall deliver at least 2.23 W orbit-average power to the bus at end of life while tumbling: the 1.86 W tumbling load of BASELINE §5 plus the 20 % margin requirement (1.86 × 1.2) *(was 2 W; 2.16 W at the 1.80 W load in Rev E; 2.21 W in Rev G; 2.23 W after the RF board's synthesizer regulator raised the transponder allocation to 1.85 W DC, CR-2026-009: met by the simulated 2.48 W, +12 %)* | MRD-050 | A,T |
| SYS-101 | The EPS shall incorporate body-mounted and/or deployable solar arrays | MRD-050 | I |
| SYS-102 | The EPS shall track maximum power point | MRD-050 | T |
| SYS-103 | Solar array output shall support battery charging plus loads in sunlight | MRD-050 | A,T |

### 6.2 Energy Storage

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-110 | The EPS shall include rechargeable battery storage | MRD-051 | I |
| SYS-111 | Battery capacity shall support a 36-minute eclipse in IDLE mode at no more than 10 % depth of discharge (BASELINE §5, §8) *(was 35 minutes with no depth; CDR review 2026-10-27)* | MRD-051 | A,T |
| SYS-112 | Battery depth of discharge shall not exceed 30% during normal operations, including contingency, and shall not exceed 10 % per eclipse in nominal operations (BASELINE §5) *(10 % nominal added at the CDR review, 2026-10-27)* | MRD-051 | A |
| SYS-113 | The EPS shall provide battery charge and discharge protection | MRD-051 | D |
| SYS-114 | The EPS shall report battery state of charge | MRD-051 | D |

### 6.3 Power Distribution

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-120 | The EPS shall provide regulated 3.3V rail | MRD-050 | T |
| SYS-121 | The EPS shall provide regulated 5V rail for AI payload | MRD-050 | T |
| SYS-122 | The EPS shall provide switchable power to subsystems | MRD-052 | D |
| SYS-123 | The EPS shall provide overcurrent protection on all rails | MRD-050 | T |
| SYS-124 | The EPS shall report voltage, current, and temperature telemetry | MRD-021 | D |

---

## 7. Communication Subsystem Requirements

### 7.1 Transmitter Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-200 | The transmitter shall operate on one IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz (BASELINE §7) | MRD-023 | T |
| SYS-201 | The transmitter shall support 0.5 W nominal output power and shall not exceed 1 W (BASELINE §7) | MRD-022 | T |
| SYS-202 | The transmitter shall support AFSK 1200 bps modulation | MRD-022 | T |
| SYS-203 | Spurious emissions shall be at least 40 dB below carrier and no more than 25 µW, which at 1 W is at least 46 dB (47 CFR 97.307(e)); the third harmonic shall be at least 70 dB below the carrier, measured at the antenna port with the antennas deployed (BASELINE §7; VI-RF-04) *(46 dB and 60 dB added at the CDR review, 2026-10-27, finding VV-09; 70 dB in Rev G, CR-2026-009, RID RF-07)* | MRD-024 | T |
| SYS-205 | If no authenticated telecommand is received for N days, the spacecraft shall cease all transmissions, keep the telecommand receiver on, and resume only after an authenticated telecommand; N shall be configurable by telecommand within 1–90 days (default and range TBR per decision D3), and the elapsed time shall persist across resets *(added 2026-10-15; SQSO-ICD-013 §6.2; IARU 7.9)* | MRD-024 | T |
| SYS-204 | The spacecraft shall cease all RF emissions on an authenticated telecommand in every operating mode, and also on an unauthenticated TX-OFF telecommand whenever no telecommand key is usable; shall keep emissions ceased across resets until an authenticated telecommand re-enables them; and shall keep the telecommand receiver powered while ceased (47 CFR 97.207(b); ITU RR 22.1) *(added 2026-10-15; SQSO-ICD-013)* | MRD-024 | T |
| SYS-206 | Frequency accuracy shall be within 5 kHz *(renumbered 2026-10-16 from a second SYS-204; SYS-204 is TX-OFF)* | MRD-024 | T |

### 7.2 Beacon Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-210 | The spacecraft shall transmit AX.25 formatted beacons | MRD-022 | T |
| SYS-211 | Beacon interval shall be 60 s in ACTIVE, 120 s in IDLE and 180 s in SAFE, unless a transmit inhibit is active (`docs/BASELINE_PARAMETERS.md` §8) *[PI to ratify. Amended 2026-10-16 from "configurable from 10 to 120 seconds" to the baseline. The SET_BEACON telecommand still accepts only 10 to 120 s (SQSO-ICD-013 §7), so 180 s cannot be commanded]* | MRD-020 | D |
| SYS-212 | Beacons shall include spacecraft callsign | MRD-024 | I,T |
| SYS-213 | Beacons shall include health telemetry in machine-readable format | MRD-021 | D |
| SYS-214 | Beacons shall include human-readable information field | MRD-044 | D |
| SYS-215 | Beacon content shall be generated by AI when available, template fallback otherwise | MRD-034 | D |

### 7.3 Antenna Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-220 | The antenna shall deploy after separation from deployer | MRD-080 | T |
| SYS-221 | Antenna and solar-array deployment shall be inhibited until 30 minutes after deployment switch activation (CDS Rev 14.1 §2.4.4). A power-on, brownout or unknown reset or a stowed switch shall restart the wait (CDS §2.3.2.4); a watchdog or software reset shall not shorten it; deployment attempts shall persist across resets and be bounded, and an element sensed as deployed shall not be fired *(reset rules added 2026-10-15; `deployment.h`)* | MRD-080 | T |
| SYS-222 | With the antennas deployed, the gain of each antenna (2 m and 70 cm) toward the ground over the pass-weighted directions of SQSO-LBA-001 §5.2 shall have a median of at least −3 dBi and a 10th percentile of at least −10 dBi (RF-P013, `docs/hardware/RF_DESIGN.md`) *(was "The antenna shall provide omnidirectional coverage", which a monopole can neither meet nor verify; Rev G, CR-2026-009, RID RF-05; PI ratification TBR-SYS-003)* | MRD-022 | A,T |
| SYS-223 | Antenna VSWR shall be less than 2:1 at operating frequency | MRD-022 | T |
| SYS-224 | The spacecraft shall not generate or transmit any RF signal, from any transmitter or any other RF emitter including the GNSS receiver, earlier than 45 minutes after on-orbit deployment (CDS Rev 14.1 §2.4.5). While the count runs, a deployment-switch toggle (CDS §2.3.2.4) or a power-on, brownout or unknown reset shall restart it; a watchdog or software reset shall not shorten it; no telecommand shall shorten or clear it. Once the 45 minutes have completed, the completion shall be latched so that no later reset or stowed-switch reading restarts the silence (BASELINE §4 timers; NRCSD IDD §4.1.4(5)) *(added by the funding audit, 2026-10-14; reset rules 2026-10-15; SQSO-ICD-013 §6.3; every emitter including the GNSS receiver, and the latch, in Rev G, CR-2026-009, RIDs P3A-01, LV-15)* | MRD-080 | T |
| SYS-225 | The spacecraft shall have at least three independent physical inhibits between the battery and each RF transmitter, and between the battery and each deployable actuator; RBF pins and timers do not count (CDS Rev 14.1 §2.3.7, §2.3.8) *(added by the funding audit, 2026-10-14)* | MRD-080 | I,T |

**SYS-222 in Rev G (PI to ratify, TBR-SYS-003; CR-2026-009).** The Rev B proposal ("at least −6 dBi over at least 90 % of the sphere") is withdrawn: one deployed monopole reaches −6 dBi over only 81 % of the sphere (`docs/hardware/RF_DESIGN.md` §5). SYS-222 now carries the RF design's RF-P013 statement over the directions the link uses (SQSO-LBA-001 §5.2). Verification: Analysis and Test. An electromagnetic model of each deployed monopole on the 1U body, and a pattern measurement of the engineering model in at least two orthogonal planes that agrees with the model within 2 dB (VI-RF-01, ETP VI-05). A quarter-wave monopole (BASELINE §7) has a null along its axis, so coverage of the whole sphere ("omnidirectional") can be neither met nor verified.

---

## 8. Command & Data Handling Requirements

### 8.1 OBC Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-300 | The OBC shall execute flight software within 30 seconds of power-on | MRD-002 | T |
| SYS-301 | The OBC shall maintain mission elapsed time | MRD-002 | D |
| SYS-302 | The OBC shall interface with all subsystems | MRD-001 | D |
| SYS-303 | The OBC shall collect and store telemetry | MRD-021 | D |
| SYS-304 | The OBC shall manage operational mode transitions | MRD-052 | D |
| SYS-305 | The OBC shall implement fault detection and recovery | MRD-002 | D |

### 8.2 Data Storage Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-310 | The OBC shall store at least 1 MB of mission data | MRD-001 | I |
| SYS-311 | Stored data shall be protected against corruption | MRD-002 | D |
| SYS-312 | Critical configuration shall persist across power cycles | MRD-002 | D |
| SYS-313 | Fault log shall persist across resets | MRD-002 | D |

### 8.3 Interface Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-320 | The OBC shall communicate with EPS via UART or I2C | MRD-001 | D |
| SYS-321 | The OBC shall communicate with AI payload via UART | MRD-040 | D |
| SYS-322 | The OBC shall control RF transmitter enable | MRD-020 | D |
| SYS-323 | The OBC shall read ADCS sensors | MRD-060 | D |
| SYS-324 | The OBC shall control ADCS actuators | MRD-062 | D |
| SYS-325 | The OBC shall execute only telecommands that carry a valid HMAC-SHA-256 tag under the key the command dictionary assigns and a counter greater than the last accepted counter for that key; accepted counters shall persist across resets (47 CFR 97.211(b)). The sole exception is SYS-204's TX-OFF when no key is usable *(added 2026-10-15; SQSO-ICD-013)* | MRD-024 | T |
| SYS-326 | Telecommand keys shall be loadable only through the GSE umbilical, never over RF; a recovery key shall be able to replace a compromised operational key on orbit with a pre-loaded spare, without sending key material *(added 2026-10-15; SQSO-ICD-013 §9)* | MRD-024 | T |

---

## 9. Attitude Determination & Control Requirements

### 9.1 Sensor Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-400 | The ADCS shall include a 3-axis magnetometer | MRD-061 | I |
| SYS-401 | Magnetometer accuracy shall be better than 5 degrees | MRD-061 | T |
| SYS-402 | The ADCS shall include sun sensors on at least 4 faces | MRD-061 | I |
| SYS-403 | Sun sensor accuracy shall be better than 10 degrees | MRD-061 | T |

### 9.2 Actuator Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-410 | The ADCS shall include 3-axis magnetorquers | MRD-060 | I |
| SYS-411 | Magnetorquer dipole moment shall be at least 0.1 Am² per axis | MRD-060 | T |
| SYS-412 | Magnetorquers shall be independently controllable | MRD-062 | D |

### 9.3 Control Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-420 | The ADCS shall detumble the spacecraft from body rates of up to 10 deg/s within 3 orbits (BASELINE §8, decision D9) *(was below 5 deg/s within 7 days; CDR review 2026-10-27; exit condition TBR-MRD-005)* | MRD-060 | A,T |
| SYS-421 | The ADCS shall implement B-dot detumble control | MRD-060 | D |
| SYS-422 | The ADCS shall achieve spin-stabilized coarse sun-pointing of +Y at 8 deg/s, with +Y within 15° of the Sun for at least 90 % of sunlit time (BASELINE §8; Monte Carlo of the flight code with the layout-B inertia: 99.6 %; 6 deg/s until BASELINE Rev E) *(was "gravity gradient or sun-pointing"; CDR review 2026-10-27)* | MRD-062 | A,T |

---

## 10. AI Payload Requirements

### 10.1 Hardware Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-500 | The AI payload shall use NVIDIA Jetson Orin Nano or equivalent | MRD-040 | I |
| SYS-501 | The AI payload peak power shall not exceed 8 W at the carrier input, in the 7 W module power mode (BASELINE §6) *(was 10 W; CDR review 2026-10-27)* | MRD-053 | T |
| SYS-502 | The AI payload shall be unpowered between bursts, and each burst shall last no more than 120 s including boot, in sunlight at SOC ≥ 55 % (BASELINE §6); the AI payload idle power shall not exceed 5 W *(burst limits added at the CDR review, 2026-10-27)* | MRD-053 | T |
| SYS-503 | The AI payload shall be power-gated by OBC command | MRD-052 | D |
| SYS-504 | The AI payload shall operate within thermal limits passively | MRD-001 | A,T |

### 10.2 Software Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-510 | The flight-baseline beacon text shall come from the OBC-resident selector choosing among pre-approved templates; the AI payload shall run a TinyLM model for text generation (SmolLM2-360M-Instruct or Qwen3-0.6B, INT4) only as the gated experiment of BASELINE §6, enabled when the measured p95 energy per accepted beacon is ≤ 0.1 Wh (decision D5) *(CDR review 2026-10-27)* | MRD-044 | D |
| SYS-511 | The AI payload shall receive health data from OBC | MRD-041 | D |
| SYS-512 | The AI payload shall generate beacon text in batches of at least 10 lines within one burst of no more than 120 s, at no more than 0.1 Wh (360 J) p95 per accepted beacon at the carrier input (BASELINE §6, D5) *(was "within 10 seconds"; CDR review 2026-10-27)* | MRD-044 | T |
| SYS-513 | Generated text shall be ASCII-only and within length limits, and shall pass the deterministic content filter of `docs/BEACON_SPEC.md` §6 on both the payload and the OBC (no URLs, commercial words, profanity, encoded-looking words, other call signs or numbers) *(filter added at the CDR review, 2026-10-27; 97.113(a)(3)-(4))* | MRD-097 | T |
| SYS-514 | The AI payload shall report health status to OBC | MRD-045 | D |

### 10.3 Fallback Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-520 | The OBC shall detect AI payload failure or timeout | MRD-034 | D |
| SYS-521 | The OBC shall use template beacons if AI is unavailable | MRD-034 | D |
| SYS-522 | AI failure shall not affect other spacecraft functions | MRD-034 | D |
| SYS-523 | The OBC shall automatically power-cycle AI payload on repeated failures | MRD-034 | D |

**Proposed (PI to ratify), SYS-522, TBR-SYS-004:** "If the Jetson hangs, latches up, fails to boot, sends malformed data or none, or loses power during a burst, the OBC shall power it off no later than 120 s after power-on, or at the per-rail current limit if that comes first. The OBC, EPS, telecommand receiver and telemetry beacon shall keep operating, and the transponder shall continue on the OBC's fixed schedule: no OBC reset, no missed beacon, and telecommands accepted throughout." Verification: Test (in place of Demonstration). Inject each failure at least three times on the flatsat or engineering model. The 120 s burst limit and the OBC-enforced per-rail current and temperature limits are in `docs/BASELINE_PARAMETERS.md` §6.

---

## 11. Thermal Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-600 | All components shall operate within -20C to +50C | MRD-001 | A,T |
| SYS-601 | All components shall survive -40C to +60C | MRD-001 | A,T |
| SYS-602 | Battery temperature shall be maintained above 0C for charging | MRD-051 | A |
| SYS-603 | The AI payload shall be passively cooled so that, with the module at 8 W input in the hot case: the Jetson spreader stays at or below +40 °C at the BASELINE §6 burst duty, including a 600 s batch (the +50 °C operating limit of SYS-600 less the 10 °C margin before correlation); the module junction stays at or below 95 °C (the payload guard's limit); and with the module stuck on, the spreader stays at or below +70 °C (the guard's spreader limit). Verified by analysis with the correlated thermal model using the heat-path conductance and the junction-to-spreader resistance measured on the engineering model (ETP VI-01; V-THM-3) *(was "a conductance of at least 1 W/K to the structure" (CDR review, 2026-10-27), which the burst temperatures do not need: 39.0 °C for a 600 s batch even at 0.44 W/K, `docs/environment/THERMAL_ANALYSIS.md` §6.3. Design: 1.10 W/K spreader to frame, 1.09–1.16 K/W junction to spreader, junction ≈ 48 °C at a 39 °C spreader. Rev G, CR-2026-010, RID SE-07)* | MRD-001 | A,T |

---

## 12. Environmental Requirements

### 12.1 Launch Environment

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-700 | The spacecraft shall survive launch vibration at the launch provider's levels (interim: SpaceX Rideshare User's Guide v11, pending the launch contract) *[PI to ratify. Amended 2026-10-16 from "per GEVS protoflight levels" to the launch environment of `docs/BASELINE_PARAMETERS.md` §8; interim levels in `docs/test_procedures/TP-ENV-002_Vibration.md` §4.1; provider levels TBD-SYS-002]* | MRD-080 | T |
| SYS-701 | The spacecraft shall survive acoustic environment per launch provider spec; for a fully containerized CubeSat no acoustic test is required (SpaceX UG v11 §6.6, Table 6-2), so it is verified by inspection that the dispenser is fully containerized *(CDR review 2026-10-27)* | MRD-080 | A,I |
| SYS-702 | The spacecraft shall survive depressurization at the launch-vehicle rate, verified by the venting analysis of SYS-084 and by pump-down during thermal vacuum *(CDR review 2026-10-27)* | MRD-080 | A,T |

### 12.2 Space Environment

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-710 | The spacecraft shall survive radiation environment for mission duration | MRD-001 | A |
| SYS-711 | The spacecraft shall tolerate single-event upsets | MRD-002 | D |
| SYS-712 | All materials shall meet outgassing requirements per ASTM E595: TML ≤ 1.0 % and CVCM ≤ 0.1 % (CDS §2.1.7) | MRD-080 | I |

### 12.3 Launch Environment Verification (added at the CDR review, 2026-10-27)

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-703 | The flight unit shall pass protoflight random vibration in each of three axes at the envelope of the provider's MPE + 3 dB and the GSFC-STD-7000B minimum workmanship spectrum, 1 minute per axis (GSFC-STD-7000B Table 2.2-2, Table 2.4-4, Appendix B §2.4.2.3; at least the SpaceX UG v11 Table 6-2 MPE level): 8.18 Grms on the interim levels (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §3.1) | MRD-080 | T |
| SYS-704 | The power inhibits shall be shown to function during random vibration: no power to any powered function and no RF emission while the deployment switches are actuated (SpaceX UG v11 Table 6-2 note 2, §6.7.7; CDS §2.3.1) | MRD-080 | T |
| SYS-705 | All elastic natural frequencies of the spacecraft shall be above 40 Hz, or the integrator's limit if higher, with a first-mode frequency shift below 10 % and a first-mode amplitude shift below 30 % across the vibration test (SpaceX UG v11 §6.7.1, Table 6-3) | MRD-080 | T |
| SYS-706 | The flight unit shall be baked out in thermal vacuum at no more than 1 × 10⁻⁴ Torr, at +60 °C for at least 6 hours after thermal stabilisation (CDS §3.2; LSP-REQ-317.01 Rev B Table 1 note 5, the 70 °C alternative exceeding the battery limit), or to the integrator's specification if more stringent | MRD-080 | T |
| SYS-707 | The flight unit shall pass protoflight thermal-vacuum cycling: at least 8 cycles at 10 °C beyond the allowable flight temperatures, capped at the battery's +60 °C non-operating limit, with a comprehensive performance test at the first and last hot and cold plateaus (GSFC-STD-7000B §2.6.3.2, Table 2.6-1; `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §3.4). Where the integrator requires more cycles (SpaceX UG v11 Table 6-2 advises 20 in all with at least 4 in vacuum), the 4 board-level cycles and further ambient cycles of the flight unit make up the count (ETP VI-08); the SpaceX advised acceptance range ± 5 °C is enveloped by the 10 °C margin except at the battery cap (ETP §9 deviation 4) *(cycle count and the SpaceX range reconciled in Rev G, CR-2026-009, RID P3S-10)* | SYS-600 | T |
| SYS-708 | The spacecraft shall survive the dispenser-ejection and launch-vehicle separation shock specified by the integrator; verified by analysis against the random-vibration test spectrum, or by test if the integrator requires it (SpaceX UG v11 Table 6-2, shock advised; CDS §3.3) | MRD-080 | A |
| SYS-709 | Each deployable (both antennas and both solar wings) shall deploy at the hot and cold operational test temperatures from the flight burn-wire circuit at the minimum battery voltage of the end-of-life state of charge, and the deployment-sensed telemetry shall agree with the observed state (GSFC-STD-7000B Appendix B §2.4.5) | MRD-080 | T |

### 12.4 Electromagnetic Compatibility (added at the CDR review, 2026-10-27)

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-713 | Emissions of the integrated spacecraft, in every operating mode including a Jetson burst and magnetorquer actuation, measured at the 435 MHz receiver input shall stay at least 6 dB below the receiver's minimum usable signal, and the spacecraft shall operate normally while each transmitter radiates full power (GSFC-STD-7000B Appendix B §2.5; SpaceX UG v11 Table 6-2, 6 dB EMISM by test) | MRD-022 | T |

---

## 13. End-of-Life Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-800 | At end of mission the spacecraft shall permanently disconnect the battery charge path (non-resettable) and leave the battery in a permanent discharge state (47 CFR 97.207(g)(1)(iii)) | MRD-082 | D |
| SYS-801 | The spacecraft shall disable RF transmitter at end of life | MRD-082 | D |
| SYS-802 | *Deleted (funding audit, 2026-10-14):* no separate drag device; disposal is by natural decay with the solar wings deployed (`docs/BASELINE_PARAMETERS.md` §3) | — | — |
| SYS-803 | The spacecraft shall re-enter no later than 5 years after completion of the mission, and within 25 years per NASA-STD-8719.14C (parent corrected from MRD-083) | MRD-003 | A |

### 13a. Launch Provider, Integrator and Regulatory Requirements (added at the CDR review, 2026-10-27)

The launch-vehicle, integrator and dispenser requirements are flowed down in `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`
(SQSO-LVR-001, LVR-001 to LVR-114, from SpaceX Rideshare Payload User's Guide v11, CDS Rev 14.1, the EXOpod Nova user
manual and, for the ISS backup, the NRCSD IDD). The bracketed provider IDs below, and the trace table that follows,
connect them to the system requirements. The signed integrator ICD (decision D7) supersedes them where it differs.

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-900 | The spacecraft shall fit and deploy from the selected integrator's dispenser, demonstrated by a fit check in the dispenser or its test pod (CDS §2.4.7) [Provider: LVR-001, LVR-020, LVR-021, LVR-023, LVR-024, LVR-028, LVR-029, LVR-031] | MRD-098 | I,T |
| SYS-901 | The spacecraft shall meet the integrator's mass, center-of-gravity and dimensional limits, reported in a mass-properties report from measurement of the flight unit [Provider: LVR-005, LVR-025, LVR-026, LVR-027] | MRD-098 | T |
| SYS-902 | Random vibration, bake-out and any shock test shall be performed at the levels and durations of the signed integrator ICD where they exceed SYS-703, SYS-706 and SYS-708 (CDS §3.1 to §3.3) [Provider: LVR-007, LVR-040, LVR-043, LVR-044, LVR-046, LVR-047, LVR-050, LVR-084] | MRD-098 | T |
| SYS-903 | The battery shall be delivered at the integrator's specified state of charge and with its safety data (cell lot, screening, protection test results) [Provider: LVR-013, LVR-070, LVR-072] | MRD-098 | I |
| SYS-904 | The spacecraft shall provide a forward-predicted, GNSS-based ephemeris with covariance within 24 hours of first contact, and then at least every 24 hours until it is catalogued (SpaceX UG v11 §9.5.7.2; BASELINE §6, decision D6) [Provider: LVR-089, LVR-091] | MRD-098 | D |
| SYS-905 | The spacecraft shall be compatible with the launch environment's EMI: it is unpowered in the dispenser, so compatibility is shown by the inhibits (SYS-085, SYS-704) and by the integrator's EMISM requirement [Provider: LVR-053, LVR-054, LVR-061, LVR-062] | MRD-098 | A |
| SYS-906 | The developer shall deliver the integrator's required documents: ICD verification matrix, environmental test reports, materials list with ASTM E595 data, hazard declaration, battery data, mass-properties report, export statement and proof of the FCC filing (`docs/regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md`) [Provider: LVR-002 to LVR-012, LVR-014, LVR-015, LVR-082] | MRD-099 | I |
| SYS-907 | The deployment altitude in the contract shall be at most 475 km target and 485 km hard cap, with no-penalty rebooking if the assigned flight offers nothing at or below the cap (BASELINE §3, D7) | MRD-013 | I |
| SYS-908 | No hazardous material, pressure system or energy source other than the battery and the burn-wire release shall be carried, and the hazard declaration shall be accepted by the integrator (range safety via AFSPCMAN/SSCMAN 91-710) [Provider: LVR-008, LVR-072, LVR-083] | MRD-080 | I |
| SYS-909 | After delivery, the spacecraft shall need no access, charging or software load until launch, and shall tolerate storage in the dispenser at the integrator's specified state of charge for the stated duration [Provider: LVR-051, LVR-052, LVR-071, LVR-074; at least 45 days in the dispenser without charging] | MRD-098 | A |
| SYS-910 | If the NRCSD (ISS) backup is used, the battery shall meet JSC 20793 Rev D and the NRCSD IDD Rev A, and the spacecraft shall meet the NRCSD 2.40 kg limit (BASELINE §3) [conditional: ISS path only; Provider: LVR-100 to LVR-114] | MRD-080 | I,T |
| SYS-911 | If the NASA CSLI path is used, the spacecraft shall meet LSP-REQ-317.01 Rev B, including protoflight random vibration at MPE + 3 dB for 2 minutes per axis and bake-out per its Table 1 [conditional: CSLI path only] | MRD-080 | I,T |
| SYS-912 | Separation tip-off: the ADCS shall detumble from the integrator's maximum tip-off rate, not less than the 10 deg/s of SYS-420 [Provider: LVR-030] | MRD-060 | A |
| SYS-913 | The spacecraft design shall give a re-entry casualty risk below 1:10,000 and component re-entry energy below 15 J (NASA DAS 3.2.6 analysis in the ODAR) | MRD-084 | A |
| SYS-914 | The FCC pre-space notification, with the ITU Appendix 4 data and the orbital-debris disclosures, shall be on file at least 90 days before integration into the launch vehicle (47 CFR 97.207(g)(1)) [Provider: LVR-010, LVR-075] | MRD-096 | I |
| SYS-915 | No cadmium or cadmium plating, zinc plating, mercury, or pure tin or tin electroplate (unless alloyed with lead, antimony or bismuth) shall be used, including in COTS parts finishes, unless the integrator approves a waiver [Provider: LVR-081] | MRD-080 | I |
| SYS-916 | The structure shall show positive margins at the provider's quasi-static loads with the provider's factors of safety (yield 1.10, ultimate 1.25 flight, 1.40 ground) and, for random vibration, at 3-sigma peak responses to the protoflight level with the GSFC-STD-7000B Table 2.2-3 factors [Provider: LVR-041, LVR-042] | MRD-080 | A |
| SYS-917 | Sealed containers that may pressurize on orbit, including the battery cells, shall meet the provider's design and test factors for sealed containers or pass a vacuum exposure test [Provider: LVR-073] | MRD-080 | A,T |
| SYS-918 | The spacecraft shall make no attitude actuation and no appendage deployment earlier than 120 s after separation, and shall report a bounding hard-body radius for any appendage deployed in the first 24 hours [Provider: LVR-065] | MRD-080 | T,A |
| SYS-919 | The licensee shall register the spacecraft with the 18th Space Defense Squadron no later than 2 months before launch, with a published operator contact, and claim the object within 30 days of deployment [Provider: LVR-088, LVR-090] | MRD-085 | I |

**Trace of the provider requirements (SQSO-LVR-001) to the system requirements.** Every LVR ID appears once or more;
LVR-092 (no propulsion) is met by SYS-082.

| System requirement | Provider requirements |
|--------------------|-----------------------|
| SYS-004, SYS-085 | LVR-060, LVR-067 |
| SYS-081 | LVR-022, LVR-086 |
| SYS-082 | LVR-092 |
| SYS-083 | LVR-085 |
| SYS-084 | LVR-048 |
| SYS-087 | LVR-068 |
| SYS-221 | LVR-064, LVR-066 |
| SYS-224 | LVR-063, LVR-066 |
| SYS-225 | LVR-069 |
| SYS-600 | LVR-049 |
| SYS-704 | LVR-045 |
| SYS-712 | LVR-080 |
| SYS-803, SYS-913 | LVR-087 |
| SYS-900 | LVR-001, LVR-020, LVR-021, LVR-023, LVR-024, LVR-028, LVR-029, LVR-031 |
| SYS-901 | LVR-005, LVR-025, LVR-026, LVR-027 |
| SYS-902 | LVR-007, LVR-040, LVR-043, LVR-044, LVR-046, LVR-047, LVR-050, LVR-084 |
| SYS-903 | LVR-013, LVR-070, LVR-072 |
| SYS-904 | LVR-089, LVR-091 |
| SYS-905 | LVR-053, LVR-054, LVR-061, LVR-062 |
| SYS-906 | LVR-002, LVR-003, LVR-004, LVR-006, LVR-007, LVR-008, LVR-009, LVR-010, LVR-011, LVR-012, LVR-014, LVR-015, LVR-082 |
| SYS-908 | LVR-008, LVR-072, LVR-083 |
| SYS-909 | LVR-051, LVR-052, LVR-071, LVR-074 |
| SYS-910 | LVR-100, LVR-101, LVR-102, LVR-103, LVR-104, LVR-105, LVR-106, LVR-107, LVR-108, LVR-109, LVR-110, LVR-111, LVR-112, LVR-113, LVR-114 |
| SYS-912 | LVR-030 |
| SYS-914 | LVR-010, LVR-075 |
| SYS-915 | LVR-081 |
| SYS-916 | LVR-041, LVR-042 |
| SYS-917 | LVR-073 |
| SYS-918 | LVR-065 |
| SYS-919 | LVR-088, LVR-090 |

---

## 14. Verification Matrix Summary

| Req Range | Analysis | Demonstration | Inspection | Test |
|-----------|:--------:|:-------------:|:----------:|:----:|
| SYS-001-005 | 1 | 0 | 4 | 3 |
| SYS-010-023 | 5 | 4 | 0 | 0 |
| SYS-100-124 | 3 | 3 | 1 | 5 |
| SYS-200-225 | 1 | 3 | 3 | 8 |
| SYS-300-324 | 0 | 12 | 1 | 1 |
| SYS-400-422 | 0 | 3 | 2 | 4 |
| SYS-500-523 | 2 | 9 | 1 | 3 |
| SYS-600-603 | 4 | 0 | 0 | 2 |
| SYS-700-712 | 3 | 1 | 1 | 2 |
| SYS-800-803 | 1 | 2 | 0 | 0 |
| SYS-080-089, SYS-703-713, SYS-900-919 | see the VCRM | | | |

The counts above date from Rev D and are not maintained by hand. The verification method and the planned
verification event of every requirement are in the VCRM, `docs/verification/VERIFICATION_MATRIX.md` §3, generated
from this document by `software/simulation/cdr/pm/gen_vcrm.py`.

---

## 15. TBDs and TBRs

| ID | Item | Closure Plan | Target |
|----|------|--------------|--------|
| TBD-SYS-001 | Final IARU coordinated frequency | IARU coordination: request 2027-01-26, letter by the 90th-percentile date (`mission/SCHEDULE.md` DA-09) | 2027-05-18 |
| TBD-SYS-002 | Launch provider vibration spec | Manifest assignment: the integrator ICD at contract signature (decision D7); SYS-703 levels hold until then | 2027-06-30 |
| TBR-SYS-001 | AI model final specification | Closed (2026-10-16) by PI decision D5: the AI model, AI burst and AI text-control rows of `docs/BASELINE_PARAMETERS.md` §6 specify it | — |
| TBR-SYS-002 | SYS-020 ("mitigated where practical") has no pass criterion | PI ratifies, amends or rejects the proposed wording beneath §5.3 (finding VV-16) | 2027-01-12 (PDR) |
| TBR-SYS-003 | SYS-222 ("omnidirectional coverage") could be neither met by a monopole nor verified | PI ratifies, amends or rejects the Rev G wording (RF-P013; CR-2026-009, DA-01) | 2027-01-12 (PDR) |
| TBR-SYS-004 | SYS-522 ("shall not affect other spacecraft functions") lists no failures and no pass criterion | PI ratifies, amends or rejects the proposed wording beneath §10.3 (finding VV-16) | 2027-01-12 (PDR) |

---

## 16. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §15 TBD/TBR IDs made unique to this document: TBD-001, TBD-002 and TBR-001 became TBD-SYS-001, TBD-SYS-002 and TBR-SYS-001. TBR-SYS-001 closed by decision D5 (`docs/BASELINE_PARAMETERS.md` §6) (finding VV-17). Proposed testable wording, for the PI to ratify, beneath SYS-020, SYS-222 and SYS-522; requirement text unchanged; tracked as TBR-SYS-002 to TBR-SYS-004 (finding VV-16) | SMART-QSO Team |
| C | 2026-10-16 | Frequency accuracy renumbered SYS-206 (SYS-204 was defined twice; SYS-204 stays TX-OFF). SYS-211 amended to the BASELINE §8 beacon intervals, PI to ratify (finding VV-06) | SMART-QSO Team |
| D | 2026-10-16 | SYS-700 amended from GEVS protoflight levels to the launch provider's levels, interim SpaceX Rideshare User's Guide v11 as used by TP-ENV-002, PI to ratify; ID kept (findings LA-05, VV-06) | SMART-QSO Team |
| E | 2026-10-27 | CDR review (PM/SE/V&V). Aligned with `docs/BASELINE_PARAMETERS.md` Rev D: SYS-002 (dispenser limit), SYS-003 (per axis), SYS-010, SYS-012, SYS-100, SYS-111, SYS-203 (46 dB and 60 dB), SYS-420, SYS-422, SYS-112 (10 % nominal), SYS-501, SYS-502, SYS-510, SYS-512, SYS-513, SYS-603, SYS-701, SYS-702, SYS-712. Added CDS Rev 14.1 flow-down SYS-080 to SYS-089 (§5.1a), launch-environment verification SYS-703 to SYS-709 (§12.3), EMC SYS-713 (§12.4), and launch-provider, integrator and regulatory requirements SYS-900 to SYS-919 (§13a), traced to the launch reviewer's LVR-001 to LVR-114 (SQSO-LVR-001). TBD/TBR target dates set. Change request to be logged by the chair (CR-2026-007) | CDR review, PM/SE/V&V |
| F | 2026-10-27 | CDR 2026 consistency pass: SYS-012 (465-485 km), SYS-422 (8 deg/s spin), SYS-907 (475 km target, 485 km cap) from `docs/BASELINE_PARAMETERS.md` Rev E (CR-2026-007) | CDR documentation review |
| G | 2026-10-27 | CDR round 3 (proposed; PI decision by 2027-01-12, CR-2026-009 and CR-2026-010): SYS-100 2.23 W (was proposed 2.21 W; transponder allocation 1.85 W DC, RF board E) at the BASELINE §5 1.84 W load (SE-03, P3E-08); SYS-203 third harmonic ≥ 70 dB at the antenna port (RF-07); SYS-222 the RF-P013 pattern statement (RF-05); SYS-224 every RF emitter including the GNSS receiver, and the completion latch (P3A-01, LV-15); SYS-603 a temperature requirement instead of the 1 W/K conductance (SE-07); SYS-707 the integrator's cycle count and the SpaceX range reconciled (P3S-10); applicable baseline Rev E (amended); Rev E and F, which CR-2026-007 omitted, recorded in CR-2026-009 | CDR round 3, systems and programmatics |
