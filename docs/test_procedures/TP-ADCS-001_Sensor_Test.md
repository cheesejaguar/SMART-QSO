# Test Procedure: ADCS Sensor Test
## TP-ADCS-001

**Document Number:** TP-ADCS-001
**Revision:** B
**Date:** 2026-01-02

**Requirements verified:** SYS-323, SYS-400, SYS-401, SYS-402, SYS-403

---

## 1. Purpose

Verify calibration and accuracy of ADCS sensors: the PNI RM3100 magnetometer on board A2 and the six face photodiodes
read through the ADS7138 (`docs/BASELINE_PARAMETERS.md` §8; `hardware/electrical/obc/OBC_BOARD_DESIGN.md`).

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Helmholtz cage | 3-axis, ±100 µT |
| Gauss meter | Reference magnetometer |
| Sun simulator | Collimated light source |
| Rotary table | 0.1° resolution |
| Data acquisition | 1 kHz sample rate |

## 3. Procedure

### 3.1 Magnetometer Calibration [SYS-400, SYS-401]

| Step | Field (µT) | Axis | Expected | Measured | Error | P/F |
|------|------------|------|----------|----------|-------|-----|
| 1 | 0 | All | 0 ± 1 | ______ | <1 µT | |
| 2 | +50 | X | +50 ± 1 | ______ | <2% | |
| 3 | -50 | X | -50 ± 1 | ______ | <2% | |
| 4 | +50 | Y | +50 ± 1 | ______ | <2% | |
| 5 | -50 | Y | -50 ± 1 | ______ | <2% | |
| 6 | +50 | Z | +50 ± 1 | ______ | <2% | |
| 7 | -50 | Z | -50 ± 1 | ______ | <2% | |

### 3.2 Sun Sensor Test [SYS-402, SYS-403]

| Step | Face | Illumination | Expected | Measured | P/F |
|------|------|--------------|----------|----------|-----|
| 1 | +X | Normal | Max output | ______ | |
| 2 | +X | 45° | 70% output | ______ | |
| 3 | -X | Normal | Max output | ______ | |
| 4 | +Y | Normal | Max output | ______ | |
| 5 | -Y | Normal | Max output | ______ | |
| 6 | +Z | Normal | Max output | ______ | |
| 7 | -Z | Normal | Max output | ______ | |

## 4. Pass/Fail Criteria

- Magnetometer, after the per-polarity bias correction and sampled only in the coils-off window: residual bias
  ≤ 0.3 µT per axis and noise ≤ 0.2 µT 1σ per sample (SYS-401; AV-V2, AV-V4)
- Sun sensors: each photodiode within 2 % of cos θ up to 60° incidence and at least 0.5 × cos θ at 85°, gain spread
  ≤ 3 % 1σ (SYS-403); all six faces functional (SYS-402)
- Cross-axis coupling: < 2%

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) | SMART-QSO Team |
