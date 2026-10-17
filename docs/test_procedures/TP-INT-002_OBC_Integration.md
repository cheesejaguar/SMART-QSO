# Test Procedure: OBC Integration
## TP-INT-002

**Document Number:** SQSO-TP-INT-002
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-021, SYS-300, SYS-302, SYS-320, SYS-321, SYS-323, SRS-F005

---

## 1. PURPOSE

This procedure defines the integration test for the On-Board Computer (OBC) with the EPS and other subsystems.

---

## 2. SCOPE

- OBC to EPS integration
- OBC power verification
- Boot test
- Interface verification to all subsystems

---

## 3. APPLICABLE DOCUMENTS

1. ICD_OBC_EPS.md
2. hardware/obc/OBC_DESIGN.md
3. TP-EPS-001 (prerequisite)
4. TP-INT-001 (prerequisite)

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| DMM | 6.5 digit |
| Oscilloscope | 200 MHz |
| Logic analyzer | 8 channel |
| UART adapter | USB-UART |
| J-Link debugger | SWD compatible |

---

## 5. PRECONDITIONS

- [ ] EPS integration complete (TP-INT-001)
- [ ] OBC functional test passed
- [ ] Flight software loaded
- [ ] Debug console connected

---

## 6. PROCEDURE

### 6.1 Power Integration [SYS-021]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Enable 3.3V rail | 3.3V ±0.1V | |
| 2 | Measure OBC current | < 50 mA idle | |
| 3 | Verify watchdog LED | Blinking | |

### 6.2 Boot Test [SYS-300, SRS-F005]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Apply power | Boot within 5s | |
| 2 | Check console output | Boot messages | |
| 3 | Verify software version | Correct version | |
| 4 | Check boot reason | Power-on reset | |

### 6.3 Interface Verification [SYS-302, SYS-320, SYS-321, SYS-323]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | I2C scan (EPS bus) | Devices found | |
| 2 | I2C scan (ADCS bus) | Devices found | |
| 3 | UART loopback (Jetson) | Data echoed | |
| 4 | GPIO test | All pins toggle | |
| 5 | ADC test | Valid readings | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Boot time | < 10 seconds |
| Power consumption | < 100 mW |
| All interfaces | Functional |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
