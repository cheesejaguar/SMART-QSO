# SMART-QSO Executive Summary

**Document ID**: SQSO-ES-001
**Version**: 2.0
**Date**: 2026-10-14
**Classification**: Unclassified — open-source project
**Status**: Design-phase baseline (pre-PDR in substance). See the [funding audit](funding_audit/FUNDING_AUDIT_REPORT.md).

## Mission Overview

**SMART-QSO** (Self-Managing Amateur Radio Transponder — QSO) is a proposed 1U CubeSat in the amateur-satellite
service. It carries a single-channel U/V FM transponder for worldwide amateur contacts and a plain-text AX.25 telemetry
beacon. It also carries an amateur technical investigation of onboard AI: a power-gated NVIDIA Jetson schedules the
transponder within the available power and composes beacon text. If the AI is unavailable, the spacecraft falls back to
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
| Mass | Limit 2.00 kg or the dispenser limit. BASELINE §4: CBE ≈ 1.51 kg, MEV ≤ 1.75 kg. The bottom-up [mass budget](environment/MASS_BUDGET.md) v2.0 gives CBE 1.69 kg and a predicted 1.92–2.00 kg, leaving 0–4 % margin; PI decision pending |
| Power | 16 triple-junction cells (body + two deployable wings): ≈ 2.4 W EOL orbit average tumbling, ≈ 3.6 W sun-pointed (audit estimate, BASELINE §5); 2.14 W tumbling once self-shadowing is included ([array analysis](../hardware/eps/SOLAR_ARRAY_DESIGN.md) §9) |
| Battery | 4S1P Li-ion 18650, ≈ 50 Wh; eclipse depth of discharge 1.3–2.0 % against a 10 % limit ([battery design](../hardware/eps/BATTERY_DESIGN.md) §3) |
| Flight computer | STM32L4R5ZI, FreeRTOS (flight software currently runs only as a host simulation) |
| AI payload | Jetson Orin Nano 8 GB module, 7 W mode, bursts ≤ 120 s, duty ≤ 5–10 %; model to be selected by measured energy |
| Radio | Downlink 145.800–146.000 MHz and uplink 435–438 MHz (IARU to coordinate); ≤ 1 W; FM transponder plus 1200 bps AX.25 beacon; authenticated telecommand with a hardware TX-OFF path |
| Link | 1200 bps beacon at 0.5 W closes from 5° elevation with +7.7 dB for a 12 dBic station (+4.7 dB in the conservative antenna-pattern case); analysis, [link budget](environment/LINK_BUDGET_ANALYSIS.md) v2.0 |
| Orbit | Sun-synchronous, ≤ 500 km deployment (525 km maximum); commercial rideshare in 2028 (ISS deployment as backup) |
| Design life | 2 years (12 months minimum); passive re-entry within the FCC 5-year rule |

## Program Status (as of 2026-10-14)

| Item | Status |
|------|--------|
| Hardware | **None built or procured** |
| Design reviews (SRR/PDR/CDR) | **No independent review held.** Existing records are internal self-assessments generated on 2026-01-02 |
| Flight software | C modules plus a host simulation, and a compile-only STM32L4 port (no RTOS, startup code or linker script); 40 unit-test suites (596 cases) pass; 82.7 % line coverage, counting every compiled flight source (CI, 2026-10-16) |
| Payload / ground software | Python; 143 payload and 98 ground unit tests pass in CI |
| Regulatory filings | **None submitted.** Drafts exist for the IARU request, FCC Part 97 pre-space notification, ITU Appendix 4 data and ODAR |
| Launch | **Not contracted.** Trade study recommends a commercial SSO rideshare in 2028 |
| Licensee / team | **Not yet named** |
| Budget | ROM only: $0.28M–$0.72M cash for a volunteer build, $0.58M–$1.32M for a university build ([cost ROM](funding_audit/LIFECYCLE_COST_ROM.md)) |

## Key Design Features

### Graceful degradation
```
AI-scheduled transponder + AI beacon → Template beacon + scheduled transponder → Beacon only → Safe mode
```

### Power-aware operations
- AI bursts only when sunlit, SOC ≥ 55 % and within temperature limits; duty ≤ 5 % tumbling, ≤ 10 % sun-pointed
- Transponder duty ≤ 25 %, power-gated
- Power allocation closes with +33 % (tumbling) and +63 % (sun-pointed) margin (BASELINE §5). This is an allocation, not yet a simulation; with array self-shadowing the tumbling margin is +19 %, below the 20 % requirement

## Top Risks

See [mission/RISK_REGISTER.md](../mission/RISK_REGISTER.md). The highest-rated risks are funding timing versus launch
capacity (R11), key-person dependency (R14), flight-software maturity (R18), Jetson feasibility in 1U (R16) and
regulatory schedule (R19).

## Success Criteria

| Level | Criterion | Duration |
|-------|-----------|----------|
| **Level 1** | Beacon operations and two-way command | 30 days |
| **Level 2** | AI-assisted operations and amateur service | 1 year |
| **Level 3** | Continuous amateur service | 2+ years |

## Applicable Standards

Standards are applied tailored for a proposed Class D (NPR 8705.4B) CubeSat; status per
[NASA_STANDARDS_COMPLIANCE_MATRIX.md](funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md). Software classification per
NPR 7150.2D is pending. No standard is yet demonstrated compliant.

## Forward Plan

1. Name the licensee and team (D1); confirm funding structure (D2); consult the FCC (D3)
2. Obtain integrator quotes; reserve a 2028 slot by mid-2027
3. Rebuild power, thermal and link analyses at the baseline; run NASA DAS
4. Implement the regulatory flight-software functions (TX-OFF, authenticated telecommand, 45-minute RF silence) on target hardware
5. Hold an independent PDR, then build a flatsat/EM
6. Submit the IARU request; file the FCC pre-space notification within 30 days of launch-vehicle determination

---

**Document Control**

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | "CDR release" (status and budget figures withdrawn by the funding audit) |
| 2.0 | 2026-10-14 | Funding audit | Rewritten from BASELINE_PARAMETERS with evidenced status |
| 2.1 | 2026-10-16 | Funding audit | Link, mass, power and battery rows cite the re-issued budgets (TB-05, TB-06, TB-08, TB-14, PI-14); power margins updated to BASELINE Rev B |
