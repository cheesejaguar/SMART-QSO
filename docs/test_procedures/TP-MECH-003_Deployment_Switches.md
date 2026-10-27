# Test Procedure: Deployment Switch Test
## TP-MECH-003

**Document Number:** SQSO-TP-MECH-003
**Version:** 1.1
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-221, SYS-224, SYS-225

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Verify deployment switch (kill switch) operation and redundancy. The baseline has **three deployment switches on three
rails**: DS2 drives a low-side FET in the pack return, DS1 a high-side FET in VBAT+, DS3 a second high-side FET in VBAT+
that also holds the trackers' enables low, each through a ≥ 10 ms RC filter (`docs/BASELINE_PARAMETERS.md` §4;
`docs/hardware/EPS_DESIGN.md` §8). The inhibit combinations at board level are V-EPS-3.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Multimeter | Continuity mode |
| Test-POD or fixture | Switch actuator |
| Force gauge | 0-5 N |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure DS1, DS2 and DS3 contacts (actuated, as in the dispenser) | Each in its inhibiting state; record resistance | |
| 2 | Release DS1 only, then DS2 only, then DS3 only | Bus stays off with any one switch still actuated: no converter running, no RF (SYS-225) | |
| 3 | Release all three | OBC boots; deployment timer starts (DEPLOY_SENSE) | |
| 4 | Re-actuate one switch during the 30 min count | Timer restarts (CDS §2.3.2.4; SYS-221) | |
| 5 | After the 45 min RF silence has completed, re-actuate and release one switch | The completed silence is latched and is not restarted (SYS-224; BASELINE §4) | |
| 6 | Measure actuation force of each switch | 0.5-2.0 N | |
| 7 | Chatter: 5 ms bursts on one switch | Filtered by the ≥ 10 ms RC; bus does not rise (V-EPS-3) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Contact resistance | < 1Ω closed |
| Open resistance | > 1MΩ |
| Actuation force | 0.5-2.0 N |
| OBC detection | Correct logic |
| Redundancy | Each of the three switches alone holds the inhibit (SYS-225) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-27 | Documentation audit (post-CDR 2026): rewritten for the three deployment switches DS1–DS3 (was two, in series), with the timer restart and RF-silence latch; status and banner |
