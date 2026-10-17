# Test Procedure: RF Conducted Power
## TP-RF-002

**Document Number:** SQSO-TP-RF-002
**Version:** 1.2
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-201

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
| 3 | Measure power (low) | 0.5W ±0.1W | |
| 4 | Enable TX at the maximum power setting | Carrier detected | |
| 5 | Measure power (maximum) | ≤ 1.0 W (30.0 dBm); record value (SYS-201) | |
| 6 | Verify power stability | ±0.1 dB over 60s | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Low power | 0.5W ±0.1W |
| Maximum power | ≤ 1.0 W (SYS-201: 0.5 W nominal, never above 1 W) |
| Stability | ±0.1 dB |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | The 2.0 W high-power steps and pass/fail row replaced by a maximum-setting check, ≤ 1.0 W (SYS-201); every step now cites SYS-201 (finding VV-06) |
