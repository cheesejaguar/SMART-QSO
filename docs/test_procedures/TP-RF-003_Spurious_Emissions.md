# Test Procedure: Spurious Emissions
## TP-RF-003

**Document Number:** SQSO-TP-RF-003
**Version:** 1.2
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-200, SYS-201, SYS-203

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
| 3 | Measure 2nd harmonic (2 × f_c, 291.6–292.0 MHz) | < -40 dBc (SYS-203) and ≤ 25 µW (47 CFR 97.307(e): < -46 dBc at 1 W, `docs/BASELINE_PARAMETERS.md` §7) | |
| 4 | Measure 3rd harmonic (3 × f_c, 437.4–438.0 MHz, in the uplink band) | As step 3; target < -60 dBc (`docs/BASELINE_PARAMETERS.md` §7) | |
| 5 | Sweep 9 kHz - 1 GHz | All spurious < -50 dBc | |
| 6 | Record all emissions > -60 dBc | Document | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| 2nd harmonic | < -40 dBc (SYS-203) and ≤ 25 µW (47 CFR 97.307(e); < -46 dBc at 1 W) |
| 3rd harmonic | As 2nd harmonic; target < -60 dBc (BASELINE §7) |
| All spurious | < -50 dBc |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | 2nd-harmonic limit < -40 dBc (SYS-203; was -30 dBc), with the 25 µW of 47 CFR 97.307(e) and the -60 dBc 3rd-harmonic target of BASELINE §7; carrier at the IARU-coordinated frequency (SYS-200, TBD-SYS-001; was 145.825 MHz), harmonics as multiples of it; maximum power ≤ 1 W (SYS-201) (finding VV-06) |
