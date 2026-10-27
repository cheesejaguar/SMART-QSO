# SMART-QSO Executive Summary

**Document ID**: SQSO-ES-001
**Version**: 2.3
**Date**: 2026-10-27
**Classification**: Unclassified — open-source project
**Status**: Phase B (pre-PDR) (BASELINE §1). Current review record: the [CDR 2026 design review](reviews/CDR/CDR_2026_DESIGN_REVIEW.md), an engineering assessment by AI reviewers, not an independent board.

## Mission Overview

**SMART-QSO** (Self-Managing Amateur Radio Transponder — QSO) is a proposed 1U CubeSat in the amateur-satellite
service. It carries a single-channel U/V FM transponder for worldwide amateur contacts and a plain-text AX.25 telemetry
beacon. It also carries an amateur technical investigation of onboard AI: a small selector on the flight computer schedules the
transponder within the available power and chooses plain-language beacon text (flight baseline), and a power-gated
NVIDIA Jetson composes beacon text as a gated experiment (decision D5). If the AI is unavailable, the spacecraft falls back to
a conventional transponder and template beacon.

## Mission Objectives

1. **Provide amateur radio service**: an FM transponder and decodable telemetry for licensed amateurs worldwide.
2. **Demonstrate power-aware onboard AI**: AI-scheduled transponder access and AI-composed plain-language beacons, within a strict energy budget.
3. **Fail gracefully**: conventional transponder and template beacon when the AI is off or faulted.
4. **Comply with regulations**: FCC Part 97 notifications, IARU coordination, the FCC 5-year disposal rule, and all transmissions in the clear.

## Baseline (proposed, from [BASELINE_PARAMETERS.md](BASELINE_PARAMETERS.md))

| Feature | Baseline |
|---------|----------|
| Form factor | 1U CubeSat, 100 × 100 × 113.5 mm (CDS Rev 14.1) |
| Mass | Design limit 2.00 kg (dispenser limit EXOpod Nova 1U 2.5 kg). CBE 1.52 kg, MEV 1.755 kg, margin +14.0 % (BASELINE §4; [mass budget](environment/MASS_BUDGET.md) v3.3) |
| Power | 16 triple-junction cells (body + two deployable wings): 2.48 W EOL orbit average delivered tumbling, 3.84 W sun-pointed (mission simulation, BASELINE §5; [power budget](environment/POWER_BUDGET_ANALYSIS.md) v2.5); 2.14 W isotropic-tumble bound ([array analysis](../hardware/eps/SOLAR_ARRAY_DESIGN.md) §9) |
| Battery | 4S1P Li-ion 18650, ≈ 50 Wh, charged to 4.10 V/cell (D8); depth of discharge as flown in the mission simulation ≤ 2.3 %, deepest contingency 2.8 %, against a 10 % limit (BASELINE §5) |
| Flight computer | STM32L4R5ZI; Rust flight software on the Embassy executor, no RTOS (BASELINE §6, rebaselined 2026-10-25). It runs as a host simulation, and the image has not run on hardware |
| AI payload | Jetson Orin Nano 8 GB module, 7 W mode, power-on ≤ 10 min per burst then ≥ 100 min off, duty ≤ 5–10 %; flight baseline is a template selector on the OBC, Jetson text a gated experiment (BASELINE §6, D5) |
| Radio | Downlink 145.800–146.000 MHz and uplink requested in 435.222–435.778 MHz, else 436.222–436.500 MHz (IARU to coordinate; BASELINE §7); ≤ 1 W; FM transponder plus 1200 bps AX.25 beacon; authenticated telecommand with a hardware TX-OFF path |
| Link | 1200 bps beacon at 0.5 W closes from 5° elevation with +7.9 dB for a 12 dBic station at the median spacecraft gain (+0.9 dB at the 10th percentile); telecommand ≥ 64 dBm EIRP from each station (C-14); analysis, [link budget](environment/LINK_BUDGET_ANALYSIS.md) v3.5 (BASELINE §7) |
| Orbit | Sun-synchronous, 475 km target, 465–485 km, not above 485 km; Falcon 9 Transporter-class rideshare through Exolaunch, planning launch about 2028-11 (ISS deployment as backup) (BASELINE §3) |
| Design life | 2 years (12 months minimum); passive re-entry within the FCC 5-year rule |

## Program Status (as of 2026-10-27)

| Item | Status |
|------|--------|
| Hardware | **None built or procured** |
| Design reviews (SRR/PDR/CDR) | **No independent review held.** The records of 2026-01-02 are internal self-assessments. The CDR 2026 design review (2026-10-27) is an engineering assessment by AI reviewers against the NPR 7123.1D CDR criteria, not an independent board: no review action open; verdict "a strong PDR with CDR-depth analysis, not past a CDR" (review record §7). Independent PDR planned 2027-01-12, independent CDR 2027-09-28 ([schedule](../mission/SCHEDULE.md)) |
| Flight software | Rust (`software/rust/README.md`): a host simulation, a mission simulation from integration to re-entry (SQSO-SIL-001), and an STM32L4R5ZI image on the Embassy executor that CI builds and checks but no hardware has run. At the CDR 2026 review 622 Rust tests and 56 C suites passed, with the safety-critical coverage gate (32 files, 100 % of lines and conditions) (review record §7.1); coverage gates and values in `software/flight/tests/README.md` |
| Payload / ground software | Python; payload and ground pytest suites run in CI (`.github/workflows/program-checks.yml`) |
| Regulatory filings | **None submitted.** Drafts exist for the IARU request, FCC Part 97 pre-space notification, ITU Appendix 4 data and ODAR |
| Launch | **Not contracted.** Planned: Falcon 9 Transporter rideshare through Exolaunch in an EXOpod Nova, about 2028-11-01; integrator RFQs by 2026-12-31, contract by 2027-06-30 (BASELINE §3, D7) |
| Licensee / team | Licensee **KK6GDB** (individual, D1); telecommand stations and team **not yet named or agreed** |
| Budget | ROM only: $0.31M–$0.96M cash for a volunteer build, $0.61M–$1.56M for a university build ([cost ROM](funding_audit/LIFECYCLE_COST_ROM.md) Rev F §5) |

## Key Design Features

### Graceful degradation
```
AI-scheduled transponder + AI beacon → Template beacon + scheduled transponder → Beacon only → Safe mode
```

### Power-aware operations
- AI bursts only when sunlit, SOC ≥ 55 % and within temperature limits; duty ≤ 5 % tumbling, ≤ 10 % sun-pointed
- Transponder duty ≤ 25 %, power-gated
- Power allocation closes with +33 % (tumbling) and +70 % (sun-pointed) margin against the simulated end-of-life generation; the bottom-up ACTIVE loads with the designed transponder give +27 % and +45 % (BASELINE §5; requirement ≥ 20 %). Predicted, not measured

## Top Risks

See [mission/RISK_REGISTER.md](../mission/RISK_REGISTER.md). The highest-rated risks (score 20) are funding timing versus launch
capacity (R11), key-person dependency (R14), the third RF and deployable inhibit (R26, designed, open until tested) and
the wing burn line (R27, designed, open until tested); next (16) are Jetson feasibility in 1U (R16), schedule (R21), no
PCB layout or engineering model (R24), the deployment altitude (R25) and reliability (R29).

## Success Criteria

| Level | Criterion | Duration |
|-------|-----------|----------|
| **Level 1** | Beacon operations and two-way command | 30 days |
| **Level 2** | AI-assisted operations and amateur service | 1 year |
| **Level 3** | Continuous amateur service | 2+ years |

## Applicable Standards

Standards are applied tailored for a proposed Class D (NPR 8705.4B) CubeSat; status per
[NASA_STANDARDS_COMPLIANCE_MATRIX.md](funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md). Software classification per
NPR 7150.2D: Class C for the flight software and the ground software used in operations, Class D for research and
outreach tools ([SDP](software/SOFTWARE_DEVELOPMENT_PLAN.md) §3.1). No standard is yet demonstrated compliant.

## Forward Plan

Dated actions from the CDR 2026 review (review record §7.3; [schedule](../mission/SCHEDULE.md) §5):
1. Request the NASA DAS agreement (2026-11-06) and the FCC Space Bureau consultation (2026-11-15)
2. Name the independent PDR board (2026-12-01); name the contracting legal entity and send the integrator RFQs (2026-12-31)
3. Telecommand station agreements (2027-01-05)
4. Ratify or reject the review's chair decisions C-1 to C-24; independent PDR (2027-01-12); engineering-model order (2027-01-15)
5. Early FCC pre-space notification (2027-01-19) and IARU request (2027-01-26)
6. Launch contract with the altitude clause by 2027-06-30; independent CDR with engineering-model data (2027-09-28)

---

**Document Control**

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | "CDR release" (status and budget figures withdrawn by the funding audit) |
| 2.0 | 2026-10-14 | Funding audit | Rewritten from BASELINE_PARAMETERS with evidenced status |
| 2.1 | 2026-10-16 | Funding audit | Link, mass, power and battery rows cite the re-issued budgets (TB-05, TB-06, TB-08, TB-14, PI-14); power margins updated to BASELINE Rev B |
| 2.1, note | 2026-10-24 | SMART-QSO Team | Flight computer and flight software rows: the flight software is Rust on the Embassy executor (BASELINE §6 note); test counts and coverage of 2026-10-24 |
| 2.1, note | 2026-10-25 | SMART-QSO Team | Flight computer row: Embassy executor, no RTOS, is the baseline (CR-2026-002); software classification NPR 7150.2D Class C (SDP §3.1) |
| 2.2 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: mass, power, AI burst, uplink and orbit rows from BASELINE Rev E (§3–§7) |
| 2.3 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): status Phase B and the CDR 2026 review; mass, power, battery and link rows from BASELINE Rev E (amended); AI selector as flight baseline; program status, budget and forward plan from the review record §7 and the cost ROM Rev F; top risks from SQSO-RISK-001 |
