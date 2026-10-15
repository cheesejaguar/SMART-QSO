# Risk Register

This register is the single controlled risk list. Other risk tables in the PDR/CDR packages and the Executive Summary are
superseded (finding PI-08 in `docs/funding_audit/FINDINGS_REGISTER.md`). It is **not** currently tracked as GitHub
issues: no `risk/*` label exists. Owners are TBD until the team is named (decision D1).

| ID | Risk | Likelihood | Consequence | Mitigation | Owner |
|---|---|:--:|:--:|---|---|
| R1 | Jetson Orin Nano Super power spikes / thermal over‑limit | M | H | Declock + DVFS limiting; hard power‑gating; thermistor‑based throttling; radiator strap; duty‑cycle caps; watchdog on payload process | Payload/Thermal |
| R2 | Jetson not space‑qualified (TID/SEU upset) | M | H | Treat as opportunistic payload; ECC where possible; periodic checkpoint; auto‑reboot isolation; mission operates in Safe‑Mode without Jetson | Payload/Systems |
| R3 | EPS power shortfall vs. orbit average | M | M | MPPT tuning; adaptive beacon cadence; disable Jetson under low SOC; SOC hysteresis; sun‑sim validation of margins | EPS/Systems |
| R4 | IARU coordination delays/conditions | M | M | Start early; complete dossier (RF plan, emission designator, EIRP); accept alternate channel/cadence; configurable beacon params | RF/Ops |
| R5 | Antenna deployment failure/partial deploy | L | H | Redundant burn‑wires; TVAC/vibe deployment tests; mechanical inspection and workmanship controls | Mech/Systems |
| R6 | EMI/EMC coupling (Jetson/PA/RF/OBC) | M | M | EMI pre‑scan; grounding/segregation; filters; cable routing; spread‑spectrum clocking as needed | RF/Electrical |
| R7 | Thermal capacity shortfall (radiator sizing/straps) | M | H | Thermal analysis + TVAC correlation; improve conduction path; tighten Jetson duty cycle; increase beacon interval under hot conditions | Thermal/Systems |
| R8 | Battery safety/low‑temp performance | M | M | BMS protections; charge inhibits; venting path; heater control with SOC gates; cold‑soak testing | EPS/Thermal |
| R9 | Schedule compression / supply chain lead times | M | M | Parallel flatsat and PFU; COTS preference; second‑source parts; rolling FRR entry criteria; buffer for re‑test | PM/Systems |
| R10 | Environmental test failure (vibe/TVAC/shock) | L | H | EM fit checks; protoflight test readiness reviews; workmanship audits; rework plan and re‑test time in schedule | Systems/Test |

### Programmatic Risks (added by the funding audit, 2026-10-14)

| ID | Risk | Likelihood | Consequence | Mitigation | Owner |
|---|---|:--:|:--:|---|---|
| R11 | **Funding** not secured in time to reserve a 2028 launch (SpaceX posted $7,500/kg for 2027; post-2028 booking limits reported) | H | H | Funding decision by Q2 2027; integrator quotes by 2026-12-31; phased funding tied to dated milestones (`docs/funding_audit/FUNDING_AUDIT_REPORT.md` §7) | PI |
| R12 | **Part 97 validity** lost through funding terms (compensated operations, sponsor data rights, federal procurement contract), forcing a Part 5 pivot | M | H | Grant/donation instruments only; pathway conditions C1–C8 reviewed before accepting funds; FCC pre-filing consultation (D3) | PI |
| R13 | **Launch capacity**: no 1U-capable integrator slot at ≤ 500 km before 2029 | M | H | Quotes from ≥ 2 integrators; ISS/NRCSD backup; CSLI 2028 solicitation as long-term contingency | PI |
| R14 | **Key-person dependency**: one human contributor; no named PI, licensee or team | H | H | Name the team and licensee (D1); document onboarding; at least two trained control operators | PI |
| R15 | **Export / ITAR**: foreign integrator requires a TAA (3–6 months) | M | M | Export counsel by 2027-01-31; prefer a U.S. integrator path if available | PI |
| R16 | **Jetson feasibility in 1U** (power, thermal, radiation) fails at EM test | M | H | EM thermal and power test early; per-rail latch-up protection; descope trigger D4 (beacon-only) | Payload/Systems |
| R17 | **ISS backup battery certification** (JSC-20793, NR-SRD-139) adds 3–6 months | M (if ISS) | M | Freeze cell part number early; budget battery test campaign | EPS |
| R18 | **Flight software maturity**: host-simulation only; no RTOS/target HAL; telecommand, TX-OFF and RF-silence functions missing | H | H | FSW integration plan to target hardware; prioritize the regulatory functions (SW-02, SW-03, SW-05) | FSW |
| R19 | **Regulatory schedule**: IARU coordination (months) and FCC pre-space notification (≥ 90 days before integration) on the critical path | M | H | Submit IARU request at least 12 months before integration; file the FCC notification within 30 days of LV determination | Licensee |
| R20 | **Credibility**: unsubstantiated status claims in legacy documents reach a funder or regulator | M | H | Audit notices; findings register; no document cited externally unless its status is evidenced | PI |

Notes:
- Likelihood/Consequence levels: L (Low), M (Medium), H (High). Owners are accountable leads.
- See PDR §9 for additional detail and rationale; this register is the active, working list.
