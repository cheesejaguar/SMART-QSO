# Test Procedure: Vibration Test
## TP-ENV-002

**Document Number:** TP-ENV-002
**Revision:** F (interim levels)
**Date:** 2026-10-27
**Reference:** SpaceX Rideshare Payload User's Guide, Version 11, August 2026 ("UG v11",
<https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf>): §4.1.6, Table 4-6, §6.1.2, §6.6,
Table 6-2, §6.7.1, Table 6-3, §6.7.6, §6.7.7; GSFC-STD-7000B (GEVS) Table 2.2-2, Table 2.4-4 and Appendix B §2.4.2.3,
which set the protoflight level (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §3.1).

> **Interim levels, pending the launch contract and PI confirmation.** The baseline launch path is a Transporter-class
> rideshare through a 1U-capable integrator (`docs/BASELINE_PARAMETERS.md` §3, decision D7), so SpaceX UG v11 is used
> as the interim provider. The integrator's user guide and ICD will supersede these levels. Revision A used a 14.1 Grms
> profile tied to no provider (finding LA-05). No test has been run.

**Requirements verified:** SYS-700, SYS-703, SYS-704, SYS-705

---

## 1. Purpose

Verify the structural integrity and functional performance of the SMART-QSO CubeSat after random vibration at the
interim provider's levels, and verify that the power inhibits work **during** random vibration (UG v11 §6.1.2,
§6.7.7; inhibit requirement SYS-225; vibration requirement SYS-700, amended on 2026-10-16 from "GEVS protoflight" to the launch provider's
levels, with this guide as the interim provider, for the PI to ratify).

## 2. Test Approach

SMART-QSO flies in a dispenser as a fully containerized CubeSat, so it may be tested on its own at the unit levels of
UG v11 Table 6-2 (§6.6). Only random vibration and the power-inhibit check are required there; quasi-static load, sine
vibration and acoustic tests are not required for a containerized CubeSat, and shock is advised only.

| Approach (UG v11 Table 6-2) | Unit | Random vibration level | Duration |
|------------------------------|------|------------------------|----------|
| Protoflight (SpaceX preferred; SpaceX minimum) | Flight unit | MPE | 1 minute in each of 3 axes |
| **Protoflight, this procedure (GEVS-B App. B)** | **Flight unit** | **Envelope of MPE + 3 dB and GEVS workmanship, 8.18 Grms** | **1 minute in each of 3 axes (EM pathfinder: 2 minutes)** |
| Qualification | Dedicated unit, not flown | MPE + 3 dB | 2 minutes in each of 3 axes |
| Acceptance (with a qualification unit) | Flight unit | MPE | 1 minute in each of 3 axes |

> **Under-test warning (UG v11 Table 6-2, note 1).** SpaceX requires CubeSat random vibration at no less than the MPE of
> §4.1.6, but strongly advises using levels derived from testing the fully integrated payload (the loaded dispenser).
> Testing a bare CubeSat at the §4.1.6 MPE is treated as an under-test and must be acknowledged in the environmental
> summary report. **Replace the levels below with integrator-derived levels at contract.**

## 3. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Vibration system | Electrodynamic shaker with random-vibration control, 20–2000 Hz |
| Accelerometers | Triaxial control and response accelerometers, calibrated |
| Control system | Random vibration capable; notching capability (§5.3) |
| Data acquisition | 16+ channels, 20 kHz sample rate |
| Fixture | Test pod or dispenser-representative fixture specified by the integrator |
| Inhibit monitoring | Oscilloscope or data logger on the battery-to-load power path test points; RF sniffer antenna and spectrum analyser (instrumentation TBD with the integrator) |
| GSE | Functional test equipment; battery charger (used before the test only) |

## 4. Test Levels

### 4.1 Random Vibration [SYS-700, SYS-703]

**Test level (Rev E).** The protoflight level is the envelope of the MPE below + 3 dB and the GSFC-STD-7000B Table
2.4-4 minimum workmanship spectrum (GEVS-B Appendix B §2.4.2.3: "the envelope of limit level + 3dB and minimum
workmanship"); the MPE alone is 1.7 dB below the workmanship screen. Breakpoints from
`software/simulation/cdr/pm/test_levels.py`:

| Frequency (Hz) | ASD (g²/Hz) |
|----------------|-------------|
| 20 | 0.0200 |
| 50 | 0.0299 |
| 59.9 | 0.0299 |
| 80 | 0.0400 |
| 500 | 0.0400 |
| 668.2 | 0.0299 |
| 700 | 0.0299 |
| 800 | 0.0599 |
| 925 | 0.0599 |
| 2000 | 0.0128 |
| **Overall** | **8.18 Grms** |

The MPE it is built from (UG v11 Table 4-6, P95/50, all axes):

| Frequency (Hz) | ASD (g²/Hz) |
|----------------|-------------|
| 20 | 0.01 |
| 50 | 0.015 |
| 700 | 0.015 |
| 800 | 0.03 |
| 925 | 0.03 |
| 2000 | 0.00644 |
| **Overall** | **5.57 Grms** |

Check of the overall level: integrating the table with straight lines between breakpoints on log-log axes gives segment
areas of 0.381, 9.750, 2.181, 3.750 and 14.939 g² (20–50, 50–700, 700–800, 800–925 and 925–2000 Hz), a total of
31.00 g², and √31.00 = **5.57 Grms**, matching the guide. For a qualification unit, MPE + 3 dB multiplies every ASD by
10^0.3 ≈ 2.0, giving **7.87 Grms**.

### 4.2 Duration

Flight unit: 1 minute per axis. Engineering model (pathfinder, `ENVIRONMENTAL_TEST_PLAN.md` §5): 2 minutes per axis.

| Axis | Duration (protoflight) | Tolerance |
|------|------------------------|-----------|
| X | 1 minute | −0 / +5 % (procedure value) |
| Y | 1 minute | −0 / +5 % (procedure value) |
| Z | 1 minute | −0 / +5 % (procedure value) |

### 4.3 Low-Level Sine Sweep (UG v11 §6.7.1, Table 6-3)

| Parameter | Value |
|-----------|-------|
| Level | 0.1 g minimum |
| Sweep rate | 3 oct/min or slower |
| Axes | X, Y, Z |
| Frequency range | 5–2000 Hz (procedure value) |
| Success criteria | All modes above 40 Hz; first-mode frequency shift < 10 %; first-mode response amplitude shift < 30 %. Investigate any first-mode shift above 5 % and find the root cause |

## 5. Procedure

### 5.1 Pre-Test Baseline

| Step | Action | Expected Result | Result | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Visual inspection | No damage | ______ | |
| 2 | Mass measurement | Within the current mass list (BASELINE §4: CBE 1.52 kg, MEV 1.75 kg, limit 2.00 kg, or the dispenser limit: EXOpod Nova 1U 2.5 kg) | ______ kg | |
| 3 | CG measurement | Within 2 cm of the geometric centre (CDS Rev 14.1 Table 2) | ______ cm | |
| 4 | Functional test | TP-SYS-001 pass | ______ | |
| 5 | Read and record the OBC boot counter and fault log | Recorded | ______ | |
| 6 | Charge the battery to the flight-like state of charge for launch | Recorded SOC | ______ % | |
| 7 | Photograph all faces | Documentation | ______ | |

### 5.2 Test Setup (flight-like inhibit configuration, UG v11 §6.7.7)

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Mount in the test pod or fixture with the deployment switches held actuated, as in the dispenser | Torque verified |
| 2 | Install accelerometers | Calibration verified |
| 3 | **Remove the RBF pin and any GSE inhibits**; disconnect charging GSE | Flight-like configuration recorded |
| 4 | Connect inhibit monitoring (power-path test points, RF sniffer) | Channels live |
| 5 | Verify control system | System check pass |
| 6 | Load the random profile (§4.1) and any approved notches (§5.3) | Profile loaded |

### 5.3 Notching

Notching is allowed only to keep the response within the quasi-static load levels, and must be reported to the
integrator and SpaceX. Manual notching applies to the primary mode only, at most 10 dB deep, within the bandwidths of
UG v11 Table 6-6; force limiting is preferred (UG v11 §6.7.6). Record every notch.

### 5.4 Sine Signature (Pre-Random)

| Axis | Resonance Frequencies | Notes |
|------|----------------------|-------|
| X | ______ Hz | |
| Y | ______ Hz | |
| Z | ______ Hz | |

### 5.5 Random Vibration

#### X-Axis Test

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Verify orientation | X-axis on the shaker axis | |
| 2 | Run −12 dB (checkout) | Control within ± 3 dB | |
| 3 | Run −6 dB | Control within ± 3 dB | |
| 4 | Run full level | Control within ± 3 dB | |
| 5 | Monitor responses | No excessive response | |
| 6 | Monitor inhibit channels throughout the full-level run | No power-up, no RF | |
| 7 | Record overall Grms | ______ Grms | |

#### Y-Axis Test

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Rotate to Y-axis | Verified | |
| 2 | Run −12 dB (checkout) | Control within ± 3 dB | |
| 3 | Run −6 dB | Control within ± 3 dB | |
| 4 | Run full level | Control within ± 3 dB | |
| 5 | Monitor responses | No excessive response | |
| 6 | Monitor inhibit channels throughout the full-level run | No power-up, no RF | |
| 7 | Record overall Grms | ______ Grms | |

#### Z-Axis Test

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Rotate to Z-axis | Verified | |
| 2 | Run −12 dB (checkout) | Control within ± 3 dB | |
| 3 | Run −6 dB | Control within ± 3 dB | |
| 4 | Run full level | Control within ± 3 dB | |
| 5 | Monitor responses | No excessive response | |
| 6 | Monitor inhibit channels throughout the full-level run | No power-up, no RF | |
| 7 | Record overall Grms | ______ Grms | |

### 5.6 Sine Signature (Post-Random)

| Axis | Pre-Test (Hz) | Post-Test (Hz) | First-mode shift | P/F |
|------|---------------|----------------|------------------|-----|
| X | ______ | ______ | < 10 % (investigate > 5 %) | |
| Y | ______ | ______ | < 10 % (investigate > 5 %) | |
| Z | ______ | ______ | < 10 % (investigate > 5 %) | |

### 5.7 Post-Test Verification

| Step | Action | Expected Result | Result | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Visual inspection | No visible damage | ______ | |
| 2 | Mass measurement | Same as pre-test | ______ kg | |
| 3 | Torque check | All fasteners tight | ______ | |
| 4 | Read the OBC boot counter and fault log | Unchanged from §5.1 step 5 (no boot during vibration) | ______ | |
| 5 | Functional test | TP-SYS-001 pass | ______ | |
| 6 | Burn-wire continuity and stowed-state telemetry; a deployment only if the integrator agrees in writing (deployment tests run before this procedure: TP-ENV-005 version 1.1 or later) | Continuity within the pre-test value ± 10 %; stowed | ______ | |

Actuating deployables after a protoflight test changes the flight configuration (CDS Rev 14.1 §3.5.2.1). For a fully
containerized CubeSat, UG v11 §6.3 item 7 allows burn wires to be replaced and reset identically, with evidence that the
original burn wire did not fail in the test. Any other rework needs the integrator's agreement or a re-test.

## 6. Pass/Fail Criteria

- No structural damage and no loose hardware
- Mass change < 1 %
- All modes above 40 Hz; first-mode frequency shift < 10 % and amplitude shift < 30 % (UG v11 Table 6-3)
- **Inhibits:** no power-up of loads and no RF emission during any full-level run, shown by the monitoring channels and
  the unchanged boot counter; power held off by the deployment switches, not by a software delay (UG v11 §6.7.7)
- Post-test functional test pass
- Deployment mechanisms operational (if tested)

## 7. Abort Criteria

| Condition | Action |
|-----------|--------|
| Control response > +6 dB of input | Abort test |
| Visible damage observed | Abort test |
| Inhibit channel shows power-up or RF | Abort test; safe the battery |
| Accelerometer overrange | Reduce level |
| Equipment malfunction | Pause test |

## 8. Test Data Summary

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| X-axis Grms | 8.18 ± 10 % (never below 5.57, the MPE) | ______ | |
| Y-axis Grms | 8.18 ± 10 % (never below 5.57, the MPE) | ______ | |
| Z-axis Grms | 8.18 ± 10 % (never below 5.57, the MPE) | ______ | |
| Duration per axis | 60 s, −0 / +5 % | ______ | |
| First-mode shift | < 10 % | ______ | |
| Inhibit evidence | No power-up, no RF | ______ | |
| Post-test functional | Pass | ______ | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B (interim) | 2026-10-16 | Funding audit LA-05: levels re-baselined to SpaceX Rideshare Payload User's Guide v11 as the interim provider (MPE 5.57 Grms, Table 4-6; protoflight 1 min/axis, Table 6-2); power-inhibit verification during vibration added (§6.7.7); low-level sine criteria per Table 6-3; notching rules. Pending launch contract and PI confirmation | SMART-QSO Team |
| C | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; it had cited only standards (finding VV-06) | SMART-QSO Team |
| D (interim) | 2026-10-16 | §1: SYS-700 now names the launch provider's levels (SQSO-REQ-002 Rev D, PI to ratify), so the note that its GEVS wording needed updating is replaced (finding LA-05) | SMART-QSO Team |
| E (interim) | 2026-10-27 | CDR review (PM/SE/V&V): protoflight level raised from the MPE (5.57 Grms) to the GEVS-B Appendix B envelope of MPE + 3 dB and workmanship (8.18 Grms), 1 min/axis on the flight unit and 2 min/axis on the EM (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §3.1); SYS-703 to SYS-705 added; post-test deployment replaced by continuity checks | CDR review, PM/SE/V&V |
| F (interim) | 2026-10-27 | CDR 2026 consistency pass: mass check values from BASELINE Rev E §4 (CBE 1.49 kg, MEV 1.73 kg) | CDR documentation review |
