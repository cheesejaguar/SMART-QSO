# Test Procedure: RF Conducted Power
## TP-RF-002

**Document Number:** SQSO-TP-RF-002
**Version:** 1.4
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-201
**Verification item:** VI-RF-02 (`docs/verification/VI_REGISTER.md`; defined in `docs/environment/LINK_BUDGET_ANALYSIS.md`): conducted power at the antenna port at 0.5 and 1 W, −20 to +60 °C

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Measure RF transmitter conducted power output at antenna connector.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| RF power meter | 0.1 - 10W, 100-500 MHz |
| Attenuator | 30 dB, 10W |
| 50Ω load | 5W minimum |

---

## 3. PROCEDURE [SYS-201]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect power meter via attenuator | Setup verified | |
| 2 | Enable TX @ low power | Carrier detected | |
| 3 | Measure power (nominal, 0.5 W setting) at the antenna port, after the filter | 27.0 ± 0.5 dBm (VI-RF-02) | |
| 4 | Enable TX at the maximum power setting | Carrier detected | |
| 5 | Measure power (maximum) | 30.0 dBm − 0.5 dB (VI-RF-02), never above 30.0 dBm (1 W, SYS-201); record value | |
| 6 | Verify power stability | ±0.1 dB over 60s | |
| 7 | Repeat steps 3 and 5 at −20 °C and +60 °C in a thermal chamber (VI-RF-02) | As steps 3 and 5 | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Nominal power | 27.0 ± 0.5 dBm at −20, +25 and +60 °C (VI-RF-02) |
| Maximum power | 29.5–30.0 dBm (VI-RF-02: 30.0 dBm −0.5 dB, never above 1 W, SYS-201: 0.5 W nominal) |
| Stability | ±0.1 dB |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | The 2.0 W high-power steps and pass/fail row replaced by a maximum-setting check, ≤ 1.0 W (SYS-201); every step now cites SYS-201 (finding VV-06) |
| 1.3 | 2026-10-27 | Documentation audit (post-CDR 2026): criteria from VI-RF-02 (27.0 ± 0.5 dBm, 30.0 dBm − 0.5 dB capped at 1 W, −20 to +60 °C); status and banner |
| 1.4 | 2026-10-27 | Documentation audit (post-CDR 2026), follow-up: maximum-power criterion written as VI-RF-02 now states it (30.0 dBm −0.5 dB, never above 1 W) |
