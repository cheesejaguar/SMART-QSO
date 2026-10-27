# Test Procedure: Antenna Deployment
## TP-RF-005

**Document Number:** SQSO-TP-RF-005
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-220, SYS-221, SYS-223

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Verify antenna deployment mechanism and post-deployment RF performance. The antennas are the bought ISIS AntS, deployed
by its own controller on command over I2C3, after both solar wings (`docs/BASELINE_PARAMETERS.md` §4, §7;
`docs/interfaces/ICD_OBC_ADCS.md` §12.3; SRS-F089). Wing deployment is TP-MECH-002.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Current monitor | 0-5A |
| Timer | 0.1s resolution |
| Video camera | HD, 60 fps |
| VNA | 50 MHz - 500 MHz |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify antenna stowed | Secure, constrained | |
| 2 | Command deployment (DEPLOY_RETRY, or the flight sequence after the wings): temperature read (0xC0), ARM 0xAD, automatic deployment 0xA5, STATUS 0xC3 polled | AntS temperature ≥ −17 °C before the attempt (decision C-18, SRS-F165); each element burns in turn (10 s per element, ICD_OBC_ADCS §12.3) | |
| 3 | Measure the AntS supply current (3V3_ANTS, EPS SW8) | ≤ 0.2 A at the pack (`docs/hardware/EPS_DESIGN.md` §5.1); record | |
| 4 | Record each element's time to release from the status word | Record | |
| 5 | Verify full deployment | All four elements extended; all four "not deployed" status bits clear; housekeeping `DEP` mask agrees | |
| 6 | Measure VSWR | < 2:1 (SYS-223) at the IARU-coordinated downlink frequency (TBD until coordination, TBD-SYS-001; SYS-200) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Deployment success | 100% |
| Status | Status word and housekeeping agree with the observed state (SYS-709) |
| VSWR | < 2:1 |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Step 6 VSWR at the IARU-coordinated downlink frequency (SYS-200, TBD-SYS-001), not 145.825 MHz; SYS-223 cited (finding VV-06) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): antenna deployment rewritten for the bought ISIS AntS over I2C after the wings, with the C-18 temperature gate (was a 0.5–1 A burn wire, < 5 s); status and banner |
