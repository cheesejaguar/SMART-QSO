# Test Procedure: ADCS Algorithm Test
## TP-ADCS-003

**Document Number:** SQSO-TP-ADCS-003
**Version:** 1.4
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** MRD-061, SYS-421, SYS-422

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Verify ADCS attitude determination and control algorithms.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Helmholtz cage | 3-axis, ±1 Gauss |
| Rate table | 0.1-10°/s |
| Sun simulator | Collimated light source |

---

## 3. PROCEDURE

### 3.1 Sun-Vector and Rate Knowledge [MRD-061]

The flight software has no three-axis attitude estimate (decision C-22; `docs/hardware/ADCS_DESIGN.md` §4.3), and no
accuracy is required of the sun vector (decision C-23: the pointing is judged by its power, ADCS-P002, in the
Monte Carlo). This section measures what the sun-pointing law uses against what the Monte Carlo's photodiode model
assumes (ADCS_DESIGN §11.6). A result worse than the model takes the recorded option, an albedo-immune sun sensor on
+Y (C-23). The EM has its wings deployed.

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Sun simulator on each face photodiode in turn at 0°, 30°, 60°, 75°, 85° and 88° incidence; record the ADS7138 code | Response within 2 % of cos θ up to 60°; at 85° and 88° at least 0.5 × cos θ (the Monte Carlo's grazing model); the six gains recorded | |
| 2 | Sun simulator at 20 directions within 30° of +Y, the spacecraft turned on the rate table; record the flight software's sun vector | Angle from the true direction within the model without albedo: ≤ 3.2° in 95 % of directions (ADCS_DESIGN §11.6) | |
| 3 | The same with the ±X photodiodes behind the deployed wings (directions 10–60° from +Y toward ±X) | The ±X channels read the shadow (code within the noise of zero) | |
| 4 | Spin at 8°/s about +Y on the rate table, 10 min | Gyro rate within 0.2°/s of the table on each axis (ADCS-P007) | |

### 3.2 B-dot Detumble [SYS-421]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Simulate rotation | Rate detected | |
| 2 | Enable B-dot controller | Actuators respond | |
| 3 | Verify opposing torque | Correct direction | |

**Proposed (PI to ratify), TBR-TPADCS003-001:** Step 1: rotate the unit on the rate table at 0.5, 5 and 10°/s about each axis in turn; pass if the reported rate is within ±0.1°/s at 0.5°/s and within ±5 % at 5 and 10°/s. Steps 2 and 3: in the Helmholtz cage, apply a field that rotates at the same rates; pass if each magnetorquer's commanded dipole has the sign that opposes the change in the measured field and reaches more than 0.2 A·m² at saturation with the CR0002 rods (at least 0.15 A·m² with the air-core alternate; `docs/BASELINE_PARAMETERS.md` §8). §4 "B-dot convergence": replace "Damping observed" by the mission detumble criterion once the PI ratifies it (proposed: below 0.5°/s within 3 orbits from 10°/s, shown by simulation and a hardware-in-the-loop run). The rate table covers 0.1–10°/s (§2) and 10°/s is the baseline's worst tip-off rate. ±0.1°/s is a fifth of the 0.5°/s detumble exit threshold, so measurement error cannot decide the exit.

### 3.3 Sun Pointing [SYS-422]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Set sun vector | Known direction | |
| 2 | Enable sun pointing (spin about +Y at 8°/s, BASELINE §8) | Commanded dipole drives +Y toward the Sun with the sign the law gives | |
| 3 | Pointing performance | Not measurable on the bench: verified by the Monte Carlo of the flight code with the face-photodiode and albedo model (ADCS_DESIGN §11.6), with the photodiode data of §3.1 entered: sunlit generation lost to pointing ≤ 5 % and the Sun within 30° of +Y for ≥ 95 % of sunlit time in every trial (decision C-23; ADCS-P002) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Photodiode response and sun vector against the Monte Carlo model (§3.1 steps 1–3) | Within the model; otherwise the +Y sensor option (C-23) |
| Rate knowledge (§3.1 step 4) | ≤ 0.2°/s per axis |
| B-dot convergence | Damping observed |
| Sun pointing | Law sign correct on the bench; ADCS-P002 (≤ 5 % generation loss, ≥ 95 % of sunlit time within 30°) by the Monte Carlo with the measured photodiode data (C-23) |

---

## 5. TBD/TBR ITEMS

| ID | Item | Closure Plan | Target |
|----|------|--------------|--------|
| TBR-TPADCS003-001 | §3.2 and §4 give no pass values for detumble ("Rate detected", "Damping observed") | PI ratifies, amends or rejects the values proposed beneath §3.2, with the mission detumble requirement (finding VV-16) | TBD |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Proposed pass values for §3.2 and §4, for the PI to ratify, beneath §3.2; tracked as TBR-TPADCS003-001 in §5 (finding VV-16) |
| 1.2 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.3 | 2026-10-27 | §3.1 and §4: the TRIAD attitude test replaced by sun-vector and rate knowledge (MRD-061 Rev G, decisions C-22 and C-23: the pass values are the Monte Carlo model's, not a knowledge requirement; ADCS_DESIGN 1.9 §4.3, §11.6), with the photodiode response, the wings' shadow and the gyro at the spin rate (finding P3F-G-07) |
| 1.4 | 2026-10-27 | Documentation audit (post-CDR 2026): §3.3 and §4 sun pointing judged by ADCS-P002 in the Monte Carlo (C-23; was < 10°); rod dipole > 0.2 A·m²; status and banner |
