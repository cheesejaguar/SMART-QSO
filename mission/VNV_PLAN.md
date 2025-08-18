# Verification & Validation Plan (Lightweight, NASA‑inspired)

This plan tailors NASA smallsat practice (e.g., TechEdSat, EcAMSat, PharmaSat, GeneSat, O/OREOS) to a minimal, launch‑worthy set of activities. It follows the spirit of NASA systems and environmental guidance while applying risk‑based tailoring appropriate to a 1U CubeSat.

## References (spirit only)
- Systems Engineering: NPR 7123.1 (tailored), NPR 7120.5 (governance, tailored)
- Environmental: GSFC‑STD‑7000 (GEVS) — protoflight approach
- Safety & Mission Assurance: NPR 8715 series (tailored), workmanship NASA‑STD‑8739.x family (wiring/solder/ESD)
- Launch/ICD: Launch provider rideshare user guide and CubeSat ICD (per assigned provider/CSLI)

## Methods and Articles
- Methods: Analysis (A), Test (T), Inspection (I), Demonstration (D)
- Test Articles:
  - Flatsat (FS): bench integration for functional/software/ML regression
  - Engineering Model (EM): structural/thermal fit checks if available
  - Protoflight Unit (PFU): flight unit tested to protoflight (PF) levels

## Verification Matrix (top‑level)
- Structure & Mechanisms: A/T/I — mass properties, fasteners, deployment checks
- Thermal: A/T — TVAC cycling; thermal sensors correlation
- EMI/EMC: A/T — pre‑scan, emissions/susceptibility per launch provider limits
- Vibration/Shock: T — random vibe (PF), sine burst as required; separation shock if applicable
- Power/EPS: A/T/D — battery safety, charge control, load shedding, MPPT behavior
- RF/Beacon: T/D — AX.25/AFSK beacons per `docs/BEACON_SPEC.md`, conducted/OTA in shielded setup
- C&DH/Software: T/D — watchdog, safe‑mode, non‑encrypted ops; fail‑graceful with AI payload off
- Materials/Processes: A/I — outgassing screening, prohibited materials, workmanship

## Environmental Test Plan (Protoflight per GEVS)
Tailor final levels/durations to the assigned launch vehicle/deployer. Typical protoflight practices:

1. Random Vibration (PFU) — T
   - Levels/profiles: per rideshare user guide (GEVS‑compatible)
   - Duration: protoflight durations per axis (e.g., ~1–2 min/axis)
   - Success: no structural damage or performance degradation; post‑test functional pass

2. Sine Burst/Low‑Freq Survey — T (as required by provider)
   - Verify low‑frequency stiffness and fastener integrity

3. Separation/Pyro Shock — T/A
   - If provider supplies shock environment, test or analysis of margin vs. specification

4. Thermal Vacuum (PFU) — T
   - Cycles: protoflight cycles through hot/cold operational and survival plateaus
   - Dwell: sufficient thermal soak for steady‑state at each extreme
   - Success: functional checks at temperature plateaus; leak‑free operation, no anomalies

5. EMI/EMC Pre‑scan (FS/PFU) — T
   - Radiated/conducted emissions below provider limits; susceptibility checks for key bands

6. Deployment/Release Tests — D/I
   - Antenna release and fail‑safe inhibits per ICD (off‑gassed, no binding)

## Functional & RF Verification
1. End‑to‑End Beacon Test — T/D
   - Conducted or OTA in shielded setup; AX.25 UI frames at 1200 bps AFSK; verify info‑field format and callsign in each frame
2. Safe‑Mode & Watchdog — T/D
   - Induce faults; verify revert to basic beacon; AI payload optional
3. Power Budget & Duty‑Cycle — A/T
   - Sun‑sim tests validating scheduler limits and SOC thresholds
4. Data Integrity — T
   - Verify no encryption/obscuration; frames match `docs/BEACON_SPEC.md`

## Safety, Materials, and Workmanship
- Battery safety: charge inhibit for launch, venting path, protective circuitry validated (T/A)
- Materials: outgassing screening (ASTM E595 or provider‑accepted database), restricted materials compliance (A)
- Workmanship: wiring/solder per NASA‑STD‑8739.x intent; ESD controls documented (I)

## Artifacts and Deliverables
- Verification Compliance Matrix (VCM): requirements vs. A/T/I/D evidence
- Environmental test procedures, as‑run logs, data, and post‑test functional results
- Mass properties report; center‑of‑gravity within ICD limits
- EMI/EMC pre‑scan results; RF channelization and deviation measurements
- Hazard/safety report per provider template; battery test summary
- IARU coordination letter; regulatory summary per `docs/REGULATORY.md`

## Reviews and Entry/Exit Criteria (Lightweight)
- PDR: initial requirements, RF plan, early risk assessment
- CDR: finalized ICD interface, environmental plan, test procedures, VCM baseline
- PSR/FRR: completed environmental testing, functional verification, configuration freeze, mass properties, regulatory clearances

## Tailoring & Risk
- AI payload (Jetson) is opportunistic; fail‑graceful operations verified with AI disabled
- Environmental margins aligned to rideshare guidance; additional testing only where risk justifies

