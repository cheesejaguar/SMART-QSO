# Mission Requirements Document
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REQ-001
**Revision:** D
**Date:** 2026-10-27
**Reference:** NPR 7123.1D (tailored, `docs/project/CDR_CRITERIA_MATRIX.md` §2); `docs/BASELINE_PARAMETERS.md` Rev D

---

## 1. Purpose

This document defines the Level 1 (Mission) requirements for the SMART-QSO CubeSat mission. These requirements are derived from the mission objectives and Concept of Operations (ConOps) and flow down to system-level requirements.

## 2. Scope

This document covers:
- Mission objectives and success criteria
- Top-level mission requirements
- Verification approach for each requirement
- Traceability to stakeholder needs

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-BL-001 Rev D | Baseline Parameters (governs every value; where this document disagrees, the baseline wins) |
| NPR 7123.1D | NASA Systems Engineering Processes and Requirements (reference framework, tailored; the project is not NASA-funded) |
| NPR 7120.8A | NASA Research and Technology Program and Project Management (reference for a NASA partnership; `docs/project/CDR_CRITERIA_MATRIX.md` §1) |
| SQSO-CONOPS | SMART-QSO Concept of Operations |
| Cal Poly CDS Rev 14.1 | CubeSat Design Specification (February 2022) |
| 47 CFR Part 97 | FCC Amateur Radio Service, in particular 97.113, 97.207, 97.211 and 97.307 |
| NASA-STD-8719.14C | Process for Limiting Orbital Debris (re-entry, casualty risk, collision probability) |
| SpaceX Rideshare Payload User's Guide v11 (August 2026) | Interim launch-provider requirements until the integrator ICD is signed (decision D7) |

## 4. Mission Overview

### 4.1 Mission Statement

SMART-QSO is a 1U CubeSat mission demonstrating onboard agentic AI to manage an amateur radio transponder, using TinyML/TinyLM models to learn power availability, predict regional demand, prioritize QSOs fairly, and compose dynamic beacons.

### 4.2 Mission Objectives

| ID | Objective | Priority |
|----|-----------|----------|
| MO-1 | Demonstrate AI-managed amateur radio operations from LEO | Primary |
| MO-2 | Provide amateur radio operators with transponder access | Primary |
| MO-3 | Demonstrate graceful degradation to conventional operation | Primary |
| MO-4 | Advance TinyML/TinyLM technology for space applications | Secondary |
| MO-5 | Engage amateur radio community in space communications | Secondary |

### 4.3 Success Criteria

| Level | Criteria |
|-------|----------|
| Minimum | Beacon transmission received on ground |
| Partial | Conventional transponder operation achieved |
| Full | AI-managed transponder operation demonstrated |
| Extended | *Under review; the PI keeps or drops this criterion:* results from ground-side community-model ("federated learning") research using downlinked data. Federated learning is not in the flight baseline (`docs/BASELINE_PARAMETERS.md` §2) |

---

## 5. Mission Requirements

### 5.1 Mission Duration Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-001 | The mission shall have a design life of 2 years and a minimum mission duration of 12 months (`docs/BASELINE_PARAMETERS.md` §2) *(design life 12 months until Rev C; aligned with the baseline at the CDR review, 2026-10-27)* | A | Critical |
| MRD-002 | The spacecraft shall be capable of autonomous operations for up to 7 days without ground contact | D | Critical |
| MRD-003 | The spacecraft shall re-enter the atmosphere no later than 5 years after completion of the mission with disposal success probability ≥ 0.9 (47 CFR 97.207(g)(1)(vii)(D)(1)), and within 25 years after mission completion per NASA-STD-8719.14C. *(Changed from the 25-year rule by the funding audit, 2026-10-14; see `docs/regulatory/filings/ODAR_SMART-QSO.md`.)* | A | Critical |

### 5.2 Orbit Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-010 | The mission shall operate in Low Earth Orbit (LEO) | A | Critical |
| MRD-011 | The nominal orbit altitude at deployment shall be 450-500 km, circular and sun-synchronous (inclination about 97.4°), so that the nominal orbital lifetime with the wings deployed (2.3 years at 450 km, `docs/BASELINE_PARAMETERS.md` §3) exceeds the 2-year design life *(was 400-500 km until Rev C: at 400 km the lifetime is shorter than the design life, finding TB-03)* | A | Critical |
| MRD-012 | The orbit inclination shall support global amateur radio coverage | A | High |
| MRD-013 | The deployment altitude shall not exceed 500 km unless NASA DAS shows MRD-003 is met with the solar wings undeployed, and shall not exceed 525 km in any case. *(Added by the funding audit, 2026-10-14; see `docs/BASELINE_PARAMETERS.md` §3.)* | A | Critical |

**Proposed (PI to ratify), MRD-012, TBR-MRD-002:** "The orbit shall give every point on the Earth's surface at least two passes a day with a maximum elevation of at least 5°." Verification: Analysis. Propagate the baseline orbit (circular, sun-synchronous, ≈ 97.4°; `docs/BASELINE_PARAMETERS.md` §3) at 450 km and at the 525 km cap for 30 days and count, on a 5° latitude-longitude grid, the passes that reach 5° elevation, where the baseline beacon link closes (baseline §7). The ISS backup orbit (51.6°) reaches only about ±67° latitude and would not meet this wording.

### 5.3 Communication Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-020 | The spacecraft shall transmit a beacon at least every 60 seconds in ACTIVE mode, and at least every 180 seconds in IDLE and SAFE to save power (`docs/BEACON_SPEC.md` §7), unless a transmit inhibit is active | T | Critical |
| MRD-021 | The beacon shall include spacecraft health telemetry | T | Critical |
| MRD-022 | The spacecraft shall be detectable by amateur ground stations | T | Critical |
| MRD-023 | The downlink frequency shall be coordinated with IARU | I | Critical |
| MRD-024 | The downlink shall comply with all applicable amateur radio regulations | I | Critical |
| MRD-025 | All transmissions shall be in the clear (no encryption) per Part 97 | I | Critical |

### 5.4 Transponder Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-030 | The spacecraft shall provide amateur radio transponder capability | D | Primary |
| MRD-031 | The transponder shall support single-channel FM (16K0F3E), U/V: uplink 435-438 MHz, downlink 145.800-146.000 MHz (`docs/BASELINE_PARAMETERS.md` §2, §7) *(was "linear and/or FM" until Rev C)* | T | Primary |
| MRD-032 | The transponder shall be accessible to licensed amateur operators | D | Primary |
| MRD-033 | Transponder operation shall be managed by onboard AI when available | D | Primary |
| MRD-034 | The spacecraft shall revert to conventional transponder mode if AI is unavailable | D | Critical |

### 5.5 AI/ML Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-040 | The spacecraft shall incorporate onboard AI for transponder management | D | Primary |
| MRD-041 | The AI shall learn power availability patterns | D | Primary |
| MRD-042 | The AI shall predict regional amateur radio demand | D | Secondary |
| MRD-043 | The AI shall prioritize QSOs fairly | D | Primary |
| MRD-044 | The AI shall compose dynamic beacon messages | D | Primary |
| MRD-045 | AI operations shall not compromise spacecraft safety | A | Critical |

**Proposed (PI to ratify), MRD-043, TBR-MRD-003:** "Over any 60 days of primary operations, the Gini coefficient across regions of the minutes the transponder is available while each region is in view shall be no more than 0.35, and no higher than that of a fixed transponder schedule using the same energy." Regions are those of the demand model of MRD-042. Verification: Analysis (in place of Demonstration). Before launch, a 60-day simulation of the scheduler with the baseline orbit and power model; after commissioning, the same calculation from the transponder on/off telemetry and the orbit. The 0.35 limit is mission objective 3 of `docs/pdr/SMART-QSO_PDR.md` §2.1. The transponder is a single-channel FM transponder (`docs/BASELINE_PARAMETERS.md` §2), so the spacecraft cannot choose between individual QSOs; what it controls is when the transponder is on.

**Proposed (PI to ratify), MRD-045, TBR-MRD-004:** "No AI output shall be able to exceed a hard limit. The OBC shall enforce the hard limits independently of the AI, including while the AI is hung, faulted or off: Jetson bursts only in sunlight, at SOC ≥ 55 % and for no more than 120 s including boot; the per-rail current and temperature limits; the SOC mode thresholds; and TX-OFF (`docs/BASELINE_PARAMETERS.md` §5–§7)." Verification: Test (in place of Analysis). Fault injection on the flatsat: for each hard limit, AI requests that would exceed it; malformed and out-of-range AI outputs; and a hung or unpowered AI. Pass if no limit is exceeded and every beacon is sent on its interval.

### 5.6 Power Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-050 | The spacecraft shall generate sufficient power for all mission modes | A,T | Critical |
| MRD-051 | The spacecraft shall survive eclipse periods of up to 36 minutes (the β = 0 cold case, `docs/BASELINE_PARAMETERS.md` §8) *(was 35 minutes until Rev C)* | A,T | Critical |
| MRD-052 | Power-intensive operations shall be duty-cycled based on power availability | D | High |
| MRD-053 | AI payload power consumption shall be managed to maintain positive power budget | D | Critical |

### 5.7 Attitude Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-060 | The spacecraft shall detumble from body rates of up to 10°/s within 3 orbits (`docs/BASELINE_PARAMETERS.md` §8, decision D9) *(was "within 7 days of deployment" until Rev C; the exit condition and the method are still TBR-MRD-005)* | A,T | Critical |
| MRD-061 | The spacecraft shall maintain attitude knowledge sufficient for antenna pointing | A | High |
| MRD-062 | The spacecraft shall stabilize attitude for nominal operations | T | High |

**Proposed (PI to ratify), MRD-060, TBR-MRD-005:** "From body rates of up to 10°/s about each axis at separation, the spacecraft shall reduce its body rate to below 0.5°/s, held for 5 minutes, within 3 orbits." Verification: Analysis and Test (in place of Test alone: a 1U with only magnetorquers cannot be detumbled on the ground without an air bearing). A Monte Carlo simulation of at least 100 runs of the flight B-dot code with a geomagnetic field model, the baseline orbit and 0.15 A·m² per axis, every run passing; a closed-loop hardware-in-the-loop run with simulated dynamics; and the sensor and magnetorquer checks of TP-ADCS-003 §3.2. Confirmed in flight during commissioning. The 10°/s and 3 orbits are the baseline (`docs/BASELINE_PARAMETERS.md` §8). 0.5°/s for 5 minutes is the detumble exit condition in `docs/hardware/ADCS_DESIGN.md` (mode table) and the 0.5°/s threshold in `software/flight/include/adcs_control.h`. The documents disagree today: SYS-420 says below 5°/s within 7 days, and others give 1 orbit, 2 orbits, 1°/s or 2°/s.

### 5.8 Ground Segment Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-070 | The mission shall support ground station reception using amateur equipment | D | Critical |
| MRD-071 | Beacon telemetry shall be decodable by amateur operators | D | High |
| MRD-072 | The mission shall coordinate with amateur satellite tracking networks | I | High |

### 5.9 Safety Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-080 | The spacecraft shall comply with all launch provider safety requirements | I | Critical |
| MRD-081 | The spacecraft shall not create debris during nominal operations | I | Critical |
| MRD-082 | The spacecraft shall passivate at end of life | D | Critical |
| MRD-083 | The spacecraft shall be trackable by ground-based surveillance | I | Critical |
| MRD-084 | The risk of human casualty from any part of the spacecraft surviving re-entry shall be less than 1:10,000 (NASA-STD-8719.14C Requirement 4.7-1; disclosed under 47 CFR 97.207(g)(1)(vii)), and no component shall re-enter with more than 15 J of kinetic energy (CDS Rev 14.1 §2.4.3.1) *(added at the CDR review, 2026-10-27)* | A | Critical |
| MRD-085 | The probability of collision with large objects (≥ 10 cm) during the orbital lifetime shall be no more than 0.001 (NASA-STD-8719.14C Requirement 4.5-1), and the licensee shall be able to respond to conjunction warnings (47 CFR 97.207(g)(1)(iv)-(v); `docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md`) *(added at the CDR review, 2026-10-27)* | A | Critical |

### 5.10 Regulatory Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-090 | The mission shall obtain IARU frequency coordination | I | Critical |
| MRD-091 | The mission shall obtain required national spectrum authorization | I | Critical |
| MRD-092 | The mission shall comply with ITU Radio Regulations | I | Critical |
| MRD-093 | The mission shall comply with orbital debris mitigation requirements | I | Critical |
| MRD-094 | The licensee shall be able to cease the spacecraft's emissions by telecommand whenever the FCC orders it, in every operating mode and independently of the AI payload (47 CFR 97.207(b); ITU RR 22.1) *(added at the CDR review, 2026-10-27; system requirement SYS-204)* | T | Critical |
| MRD-095 | The spacecraft shall be commanded only by telecommand stations designated by the licensee, with authenticated commands (47 CFR 97.211(a)-(b)); at least two geographically separated stations shall be designated before the IARU request (decision D1) *(added at the CDR review, 2026-10-27)* | I,T | Critical |
| MRD-096 | The licensee shall file the FCC pre-space notification within 30 days of launch-vehicle determination and no later than 90 days before integration into the launch vehicle, the in-space notification within 7 days of the first transmission, and the post-space notification within 3 months of the end of transmissions (47 CFR 97.207(g)(1)-(3); `docs/regulatory/REGULATORY_ROADMAP.md`) *(added at the CDR review, 2026-10-27)* | I | Critical |
| MRD-097 | No transmission shall carry messages encoded to obscure their meaning, music, obscene or indecent content, or content in which the licensee or a control operator has a pecuniary interest (47 CFR 97.113(a)); AI-generated text shall pass the content filter of `docs/BEACON_SPEC.md` §6 *(added at the CDR review, 2026-10-27)* | T,I | Critical |

### 5.11 Launch Provider and Integrator Requirements

The launch reviewer is writing `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`; its requirement IDs flow into SYS-900 to
SYS-912 (`docs/requirements/SYSTEM_REQUIREMENTS.md` §13a), where the chair fills the cross-references. Until the
integrator ICD is signed (decision D7, by 2027-06-30), SpaceX Rideshare Payload User's Guide v11 is the interim provider
document.

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-098 | The spacecraft shall meet the requirements of the integrator's dispenser ICD and the launch provider's user guide, verified by the integrator's acceptance checklist before delivery (CDS Rev 14.1 §1.2, §3) *(added at the CDR review, 2026-10-27)* | I,T | Critical |
| MRD-099 | The spacecraft and its data package shall be delivered to the integrator by the date in the launch contract, with the regulatory filings (MRD-090, MRD-096) on file *(added at the CDR review, 2026-10-27; schedule in `mission/SCHEDULE.md`)* | I | Critical |

---

## 6. Verification Methods

| Code | Method | Description |
|------|--------|-------------|
| A | Analysis | Mathematical analysis, simulation, or modeling |
| D | Demonstration | Functional demonstration of capability |
| I | Inspection | Review of documentation, certificates, or design |
| T | Test | Physical test of hardware/software |

---

## 7. Requirements Traceability

### 7.1 Traceability to Objectives

| Requirement | MO-1 | MO-2 | MO-3 | MO-4 | MO-5 |
|-------------|:----:|:----:|:----:|:----:|:----:|
| MRD-001 | X | X | X | X | X |
| MRD-020-025 | X | X | X | | X |
| MRD-030-034 | X | X | X | | X |
| MRD-040-045 | X | | X | X | |
| MRD-050-053 | X | X | X | X | |
| MRD-080-085, MRD-090-099 | | X | | | X |

### 7.2 Flow-down to System Requirements

| Mission Req | System Requirements |
|-------------|---------------------|
| MRD-001 | SYS-001, SYS-010-020 |
| MRD-020-025 | SYS-100-120 |
| MRD-030-034 | SYS-130-150 |
| MRD-040-045 | SYS-200-230 |
| MRD-050-053 | SYS-300-330 |
| MRD-060-062 | SYS-400-420 |
| MRD-080, MRD-098 | SYS-001-009, SYS-014-019, SYS-221-225, SYS-700-712, SYS-900-912 |
| MRD-084, MRD-085 | SYS-803, SYS-913 |
| MRD-094-097 | SYS-204, SYS-205, SYS-325, SYS-326, SYS-513, SYS-914 |
| MRD-099 | `mission/SCHEDULE.md` (programmatic) |

---

## 8. TBDs and TBRs

| ID | Item | Closure Plan | Target Date |
|----|------|--------------|-------------|
| TBD-MRD-001 | Exact orbit parameters | Launch manifest assignment: the integrator contract (decision D7) names the flight and drop-off altitude | 2027-06-30 |
| TBD-MRD-002 | IARU coordination confirmation | IARU process completion: request after the PDR, letter about 6 months later (`mission/SCHEDULE.md`) | 2027-08-31 |
| TBR-MRD-001 | AI model final selection | Closed (2026-10-16) by PI decision D5: the flight baseline is a calibrated selector on the OBC that chooses among pre-approved templates; generated text on the Jetson (SmolLM2-360M-Instruct or Qwen3-0.6B) is a gated experiment (`docs/BASELINE_PARAMETERS.md` §6, §9) | — |
| TBR-MRD-002 | MRD-012 has no measure of "global coverage" | PI ratifies, amends or rejects the proposed wording beneath §5.2 (finding VV-16) | 2027-01-12 (PDR) |
| TBR-MRD-003 | MRD-043 has no measure of "fairly" | PI ratifies, amends or rejects the proposed wording beneath §5.5 (finding VV-16) | 2027-01-12 (PDR) |
| TBR-MRD-004 | MRD-045 ("shall not compromise spacecraft safety") has no pass criterion | PI ratifies, amends or rejects the proposed wording beneath §5.5 (finding VV-16) | 2027-01-12 (PDR) |
| TBR-MRD-005 | MRD-060 does not say what "detumble" means, and Test alone is not feasible | PI ratifies, amends or rejects the proposed wording beneath §5.7 (finding VV-16) | 2027-01-12 (PDR) |

---

## 9. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §8 TBD/TBR IDs made unique to this document: TBD-001, TBD-002 and TBR-001 became TBD-MRD-001, TBD-MRD-002 and TBR-MRD-001. TBR-MRD-001 closed by decision D5 (`docs/BASELINE_PARAMETERS.md` §9) (finding VV-17). Proposed testable wording, for the PI to ratify, beneath MRD-012, MRD-043, MRD-045 and MRD-060; requirement text unchanged; tracked as TBR-MRD-002 to TBR-MRD-005 (finding VV-16) | SMART-QSO Team |
| C | 2026-10-16 | §4.3 Extended success criterion reworded: federated learning is ground-side research only, not in the flight baseline (BASELINE §2); keeping or dropping the criterion is the PI's decision (finding PI-13) | SMART-QSO Team |
| D | 2026-10-27 | CDR review (PM/SE/V&V). Values aligned with `docs/BASELINE_PARAMETERS.md` Rev D: MRD-001 (2-year design life, 12-month minimum), MRD-011 (450-500 km SSO), MRD-031 (single-channel FM), MRD-051 (36-minute eclipse), MRD-060 (10°/s within 3 orbits; method A,T). Added MRD-084 to MRD-099 from NASA-STD-8719.14C, CDS Rev 14.1, 47 CFR 97.113/97.207/97.211 and the launch provider (§5.11). Applicable documents updated (NPR 7123.1D; NPR 7120.5 replaced by 7120.8A as reference). TBD/TBR target dates set. Change request to be logged by the chair (CR-2026-007) | CDR review, PM/SE/V&V |
