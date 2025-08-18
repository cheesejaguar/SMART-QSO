# Risk Register (Top‑10)

Style inspired by NASA Ames CubeSat missions (TechEdSat, GeneSat, O/OREOS). Tracked as issues with labels `risk/*` and fields (Likelihood, Consequence, Mitigation, Owner).

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

Notes:
- Likelihood/Consequence levels: L (Low), M (Medium), H (High). Owners are accountable leads.
- See PDR §9 for additional detail and rationale; this register is the active, working list.
