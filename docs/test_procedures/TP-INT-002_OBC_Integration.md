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

1. `docs/interfaces/ICD_OBC_EPS.md` (SMART-QSO-ICD-001)
2. `hardware/electrical/obc/OBC_BOARD_DESIGN.md` (SQSO-OBC-BRD-001) and `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001)
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
| ST-Link or J-Link debugger | SWD, through the access port J9 |

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
| 1 | Enable 3V3_AO (OBC fed through SW10 as 3V3_OBC) | 3.3 V ± 2 % (`docs/hardware/EPS_DESIGN.md` §4) | |
| 2 | Measure OBC current | Record; 0.04 A nominal, below the SW10 0.15 A trip (EPS_DESIGN §5) | |
| 3 | Verify the watchdog is kicked | IWDG refreshed; no supervisor reset | |

### 6.2 Boot Test [SYS-300, SRS-F005]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Apply power | Flight software running within 30 s (SYS-300) | |
| 2 | Check console output | Boot messages | |
| 3 | Verify software version | Correct version | |
| 4 | Check boot reason | Power-on reset | |

### 6.3 Interface Verification [SYS-302, SYS-320, SYS-321, SYS-323]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | I2C2 (EPS controller, 0x48) and I2C1 (RTC RV-3028, 0x52) | Devices answer (OBC_BOARD_DESIGN) | |
| 2 | I2C4 (ADS7138, RM3100 0x20) and SPI3/SPI2 (M-G370, BMI088) | Devices answer; IDs read | |
| 3 | USART1 to the Jetson carrier (115200 baud, OBC transmits 8N2; `docs/interfaces/ICD_OBC_JETSON.md`) | Data echoed | |
| 4 | GPIO test | All pins toggle | |
| 5 | ADC test | Valid readings | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Boot time | < 30 seconds (SYS-300) |
| Power consumption | Recorded; within the SW10 trip (0.15 A) |
| All interfaces | Functional |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
