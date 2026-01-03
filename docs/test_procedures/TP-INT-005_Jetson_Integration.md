# Test Procedure: Jetson Payload Integration
## TP-INT-005

**Document Number:** SQSO-TP-INT-005
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

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

1. hardware/payload-jetson/INTEGRATION_DESIGN.md
2. ICD_OBC_JETSON.md
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
- [ ] SD card with flight software

---

## 6. PROCEDURE

### 6.1 Jetson Installation

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Install thermal interface | Even coverage | |
| 2 | Mount Jetson module | Secure, aligned | |
| 3 | Install heatspreader | Good contact | |
| 4 | Connect power harness | 5V supply | |
| 5 | Connect UART cable | To OBC USART1 | |

### 6.2 Power Gating Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | OBC: JETSON_PWR_EN = 0 | Jetson off | |
| 2 | Measure standby current | < 1 mA | |
| 3 | OBC: JETSON_PWR_EN = 1 | Jetson boots | |
| 4 | Measure boot current | < 2A peak | |
| 5 | Measure idle current | < 1A | |

### 6.3 UART Interface Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Wait for Jetson ready | UART active | |
| 2 | OBC sends health data | JSON received | |
| 3 | Jetson sends beacon text | Text received | |
| 4 | Verify protocol | Per ICD | |

### 6.4 Inference Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Send inference request | Processing starts | |
| 2 | Measure inference power | < 8W | |
| 3 | Receive beacon text | Valid output | |
| 4 | Verify content | Appropriate text | |
| 5 | Measure inference time | < 10 s | |

### 6.5 Thermal Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Record ambient temp | Baseline | |
| 2 | Run inference (5 min) | Stable thermal | |
| 3 | Measure Jetson temp | < 70°C | |
| 4 | Measure spreader temp | < 60°C | |

### 6.6 Shutdown Test

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
| Idle power | < 5W |
| Inference power | < 8W |
| UART communication | Reliable |
| Inference output | Valid beacon text |
| Temperature | < 70°C Jetson, < 60°C spreader |
| Boot time | < 60 s |
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
