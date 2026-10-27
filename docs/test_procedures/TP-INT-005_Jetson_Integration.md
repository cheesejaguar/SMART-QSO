# Test Procedure: Jetson Payload Integration
## TP-INT-005

**Document Number:** SQSO-TP-INT-005
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-321, SYS-501, SYS-502, SYS-503, SYS-504, SYS-511, SYS-512, SYS-513, SRS-F030, SRS-F031

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

This procedure defines the integration test for the Jetson Orin Nano AI payload module.

---

## 2. SCOPE

- Jetson module integration
- Thermal interface verification
- Power gating verification
- UART interface test
- Inference test

---

## 3. APPLICABLE DOCUMENTS

1. `hardware/electrical/jetson/CARRIER_BOARD_DESIGN.md` (the custom carrier); `hardware/payload-jetson/INTEGRATION_DESIGN.md` is earlier background
2. `docs/interfaces/ICD_OBC_JETSON.md` (SMART-QSO-ICD-002)
3. TP-JETSON-001 (prerequisite)
4. TP-INT-002 (prerequisite)

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Power analyzer | DC, 0-30V, 0-5A |
| Thermal camera | ±2°C accuracy |
| Thermocouple logger | 4+ channels |
| UART terminal | USB-UART adapter |

---

## 5. PRECONDITIONS

- [ ] OBC integration complete (TP-INT-002)
- [ ] Jetson module tested standalone (TP-JETSON-001)
- [ ] Thermal interface materials installed
- [ ] Flight image on the carrier's NVMe (custom partition layout, V-JET-3)

---

## 6. PROCEDURE

### 6.1 Jetson Installation

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Install thermal interface | Even coverage | |
| 2 | Mount Jetson module | Secure, aligned | |
| 3 | Install heatspreader | Good contact | |
| 4 | Connect power harness | VBAT_JET from the EPS (SW1 and eFuse U19; the carrier converts to the module's 5 V; `docs/hardware/EPS_DESIGN.md` §4) | |
| 5 | Connect UART cable | To OBC USART1 | |

### 6.2 Power Gating Verification [SYS-502, SYS-503]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | OBC: PAYLOAD_EN and JETSON_PWR low | Jetson off | |
| 2 | Measure standby current | < 1 mA | |
| 3 | OBC: PAYLOAD_EN and JETSON_PWR high (sunlit, SOC ≥ 55 %, Jetson ≥ −15 °C simulated; BASELINE §6) | Jetson boots; first STATUS line within 45 s (V-JET-3) | |
| 4 | Measure boot current at the carrier input | Record; below the eFuse's 1.20 A latch-off | |
| 5 | Measure input power during the burst | ≤ 8.0 W, 2 s average (V-JET-4; SYS-501) | |

### 6.3 UART Interface Test [SYS-321, SYS-511, SRS-F030, SRS-F031]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Wait for Jetson ready | UART active | |
| 2 | OBC sends health data | JSON received | |
| 3 | Jetson sends beacon text | Text received | |
| 4 | Verify protocol | Per ICD | |

### 6.4 Inference Test [SYS-501, SYS-512, SYS-513]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Send inference request | Processing starts | |
| 2 | Measure inference power at the carrier input | ≤ 8 W (SYS-501) | |
| 3 | Receive beacon text | Valid output | |
| 4 | Verify content | Passes the `docs/BEACON_SPEC.md` §6 filter on the payload and the OBC (SYS-513) | |
| 5 | Measure the burst | One power-on ≤ 10 min including boot and ≤ 3.6 kJ (SYS-512); then ≥ 100 min off (BASELINE §6) | |

### 6.5 Thermal Verification [SYS-504]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Record ambient temp | Baseline | |
| 2 | Run one flight burst | Stable thermal | |
| 3 | Measure Jetson junction temp | ≤ 95 °C (SYS-603) | |
| 4 | Measure spreader temp | Record for the SYS-603 model (≤ +40 °C in a 600 s batch in the hot case; VI-01) | |

### 6.6 Shutdown Test [SYS-503]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | OBC sends shutdown cmd | Graceful shutdown | |
| 2 | Wait for shutdown | < 10 s | |
| 3 | OBC: JETSON_PWR_EN = 0 | Power removed | |
| 4 | Verify power off | < 1 mA | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Power gating | Clean on/off |
| Idle power | < 5W (SYS-502) |
| Inference power | ≤ 8 W at the carrier input (SYS-501) |
| UART communication | Reliable |
| Inference output | Valid beacon text |
| Temperature | Junction ≤ 95 °C; spreader recorded for the SYS-603 model |
| Boot time | First STATUS line ≤ 45 s (V-JET-3) |
| Shutdown time | < 10 s |

---

## 8. DATA RECORDING

- [ ] Power measurements (idle, inference, peak)
- [ ] Thermal images
- [ ] Temperature logs
- [ ] UART traffic capture
- [ ] Inference output samples

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): VBAT_JET supply (was 5 V), NVMe image, V-JET-3/-4, SYS-512/513/603 criteria (were < 70/60 °C, < 10 s inference); status and banner |
