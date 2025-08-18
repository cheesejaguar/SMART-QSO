# Schedule (Accelerated 9‑Month to Flight Delivery)

Assumes heritage deployer, standard rideshare reviews, and tailored protoflight test campaign. Parallel workstreams minimize critical path.

## Month ‑9: Kickoff & Long‑Leads
- Freeze v0.3 architecture and ICD skeleton; start parts list and procurement (solar panels, EPS, battery, Stensat beacon, Jetson Orin Nano Super, OBC, structure).
- Begin IARU pre‑coordination (draft RF plan, emission, duty cycle) per `docs/REGULATORY.md`.
- Stand up Flatsat bench; bring‑up OBC RTOS skeleton; AX.25 tooling for beacon.

## Month ‑8: PDR & Flatsat Integration
- Hold PDR (budgets, risks, RF plan, regulatory path).
- Integrate Stensat beacon on bench; validate 1200 bps AFSK AX.25 frames end‑to‑end.
- Power budget analysis with Sun‑sim; draft thermal concept (radiator strap, DVFS/declocking plan).

## Month ‑7: CDR & EM Orders
- Hold CDR; baseline interfaces and environmental plan (GEVS protoflight).
- Release orders for flight/EM PCBs, structure, harness; receive beacon/Jetson/OBC.
- EMI/EMC pre‑scan of Flatsat; RF deviation/occupancy checks in shielded setup.

## Month ‑6: EM Build & Software FC (Feature Complete)
- Assemble Engineering Model (structure fit, harness test). Dry deploy antenna mechanisms.
- Software FC for beacon/safe‑mode; ML model export pipeline standing up.
- Draft IARU coordination package; finalize emission designator and EIRP.

## Month ‑5: Protoflight Unit (PFU) Assembly Start
- PFU board‑level bring‑up (EPS, OBC, RF, payload); harness and workmanship inspections.
- Jetson integration with declocking/DVFS profiles; duty‑cycle limits implemented.
- Battery safety testing; charge inhibit and venting verification.

## Month ‑4: Functional Verification & TVAC Booking
- PFU functional test matrix execution; beacon conformance per `docs/BEACON_SPEC.md`.
- Book TVAC/vibe lab; prepare procedures; finalize IARU submission; respond to panel queries.
- Update Verification Compliance Matrix (VCM) baseline.

## Month ‑3: Environmental (Protoflight per GEVS)
- Random vibration (per axis, PF durations); sine survey/burst if required; post‑vibe functional.
- TVAC cycling through hot/cold operational and survival; functional at plateaus.
- Shock environment by analysis or test per provider; EMI/EMC pre‑scan confirmation.

## Month ‑2: FRR Prep & Documentation Freeze
- RF conformance: deviation, bandwidth, occupancy; beacon cadence and callsign verification.
- Close VCM; compile as‑run environmental data; mass properties/CG report.
- Draft FRR package; ground segment rehearsal; ops procedures and anomaly responses.

## Month ‑1: FRR, Pack‑out, Delivery
- Flight Readiness Review (FRR) — pass criteria: environmental complete, functional conformance, regulatory/coordination in hand, configuration frozen.
- Pack‑out with desiccant and battery safing per provider ICD; deliver to integrator.
- Provide updated TLE/launch info to IARU panel if applicable; publish public beacon details.

Milestone sequence: PDR (‑8) → CDR (‑7) → PFU Assembly (‑5) → Env. Test (‑3) → FRR (‑1) → Delivery (0).
