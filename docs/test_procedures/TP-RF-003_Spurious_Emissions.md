# Test Procedure: Spurious Emissions
## TP-RF-003

**Document Number:** SQSO-TP-RF-003
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-203

**To be corrected:** step 3 and the 2nd-harmonic pass/fail limit (< -30 dBc) are weaker than the 40 dB of SYS-203, and step 2 names a 145.825 MHz carrier, which SYS-200 excludes.

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

## 3. PROCEDURE [SYS-203]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect TX to analyzer via attenuator | | |
| 2 | Enable TX at high power | Carrier at 145.825 MHz | |
| 3 | Measure 2nd harmonic (291.65 MHz) | < -30 dBc | |
| 4 | Measure 3rd harmonic (437.475 MHz) | < -40 dBc | |
| 5 | Sweep 9 kHz - 1 GHz | All spurious < -50 dBc | |
| 6 | Record all emissions > -60 dBc | Document | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| 2nd harmonic | < -30 dBc |
| 3rd harmonic | < -40 dBc |
| All spurious | < -50 dBc |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
