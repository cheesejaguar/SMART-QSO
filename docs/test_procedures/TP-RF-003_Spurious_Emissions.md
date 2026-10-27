# Test Procedure: Spurious Emissions
## TP-RF-003

**Document Number:** SQSO-TP-RF-003
**Version:** 1.3
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-200, SYS-201, SYS-203
**Verification item:** VI-RF-04 (`docs/verification/VI_REGISTER.md`): spectrum at the antenna port, and radiated with the antennas deployed, at 1 W

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Verify RF transmitter spurious and harmonic emissions comply with specifications.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Spectrum analyzer | 9 kHz - 3 GHz |
| Attenuator | 30-40 dB |
| 50Ω load | 5W minimum |

---

## 3. PROCEDURE [SYS-200, SYS-201, SYS-203]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect TX to analyzer via attenuator | | |
| 2 | Enable TX at the maximum power setting (≤ 1 W, SYS-201) | Carrier at the IARU-coordinated downlink frequency f_c (TBD until coordination, TBD-SYS-001; SYS-200) | |
| 3 | Measure 2nd harmonic (2 × f_c, 291.6–292.0 MHz) and 4th harmonic (4 × f_c, 583.2–584.0 MHz) | ≤ −46 dBc and ≤ 25 µW at 1 W (SYS-203; 47 CFR 97.307(e); VI-RF-04) | |
| 4 | Measure 3rd harmonic (3 × f_c, 437.4–438.0 MHz, next to the uplink band) at the antenna port | ≤ −70 dBc (SYS-203 Rev G; `docs/BASELINE_PARAMETERS.md` §7; VI-RF-04) | |
| 5 | Sweep 9 kHz - 1 GHz | All spurious ≤ −46 dBc and ≤ 25 µW (SYS-203) | |
| 6 | Record all emissions > -60 dBc | Document | |
| 7 | Engineering model only, antennas deployed: repeat steps 3 and 4 radiated, with a calibrated receive antenna (VI-RF-04) | As steps 3 and 4 | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| 2nd and 4th harmonics | ≤ −46 dBc and ≤ 25 µW at 1 W (SYS-203; 47 CFR 97.307(e)) |
| 3rd harmonic | ≤ −70 dBc at the antenna port (SYS-203 Rev G; BASELINE §7) |
| All spurious | ≤ −46 dBc and ≤ 25 µW (SYS-203) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | 2nd-harmonic limit < -40 dBc (SYS-203; was -30 dBc), with the 25 µW of 47 CFR 97.307(e) and the -60 dBc 3rd-harmonic target of BASELINE §7; carrier at the IARU-coordinated frequency (SYS-200, TBD-SYS-001; was 145.825 MHz), harmonics as multiples of it; maximum power ≤ 1 W (SYS-201) (finding VV-06) |
| 1.3 | 2026-10-27 | Documentation audit (post-CDR 2026): criteria from SYS-203 Rev G and VI-RF-04: 2nd/4th ≤ −46 dBc and ≤ 25 µW, 3rd ≤ −70 dBc (was −40 dBc and a −60 dBc target), radiated EM step; status and banner |
