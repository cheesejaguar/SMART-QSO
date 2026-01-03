# Test Procedure: Vibration Test
## TP-ENV-002

**Document Number:** TP-ENV-002
**Revision:** A
**Date:** 2026-01-02
**Reference:** GSFC-STD-7000A (GEVS)

---

## 1. Purpose

Verify structural integrity and functional performance of the SMART-QSO CubeSat when subjected to launch vibration environments per GEVS protoflight levels.

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Vibration system | Electrodynamic shaker, 50 kN force |
| Accelerometers | Triaxial, 500g range |
| Control system | Random vibration capable |
| Data acquisition | 16+ channels, 20 kHz sample rate |
| Fixture | CubeSat deployer mockup |
| GSE | Functional test equipment |

## 3. Test Levels

### 3.1 Random Vibration (Protoflight)

| Frequency (Hz) | ASD (g²/Hz) |
|----------------|-------------|
| 20 | 0.026 |
| 20-50 | +6 dB/oct |
| 50-800 | 0.16 |
| 800-2000 | -6 dB/oct |
| 2000 | 0.026 |
| **Overall** | **14.1 Grms** |

### 3.2 Duration

| Axis | Duration | Tolerance |
|------|----------|-----------|
| X | 1 minute | ±5% |
| Y | 1 minute | ±5% |
| Z | 1 minute | ±5% |

## 4. Procedure

### 4.1 Pre-Test Baseline

| Step | Action | Expected Result | Result | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Visual inspection | No damage | ______ | |
| 2 | Mass measurement | 2.0 kg ± 100g | ______ kg | |
| 3 | CG measurement | Within 2 cm of center | ______ cm | |
| 4 | Functional test | TP-SYS-001 pass | ______ | |
| 5 | Photograph all faces | Documentation | ______ | |

### 4.2 Test Setup

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Mount in test fixture | Torque verified |
| 2 | Install accelerometers | Calibration verified |
| 3 | Connect GSE umbilical | Continuity verified |
| 4 | Verify control system | System check pass |
| 5 | Set test levels | Profile loaded |

### 4.3 Sine Signature (Pre-Random)

| Axis | Frequency Range | Level | Sweep Rate |
|------|-----------------|-------|------------|
| All | 5-2000 Hz | 0.25g | 2 oct/min |

| Axis | Resonance Frequencies | Notes |
|------|----------------------|-------|
| X | ______ Hz | |
| Y | ______ Hz | |
| Z | ______ Hz | |

### 4.4 Random Vibration

#### X-Axis Test

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Verify orientation | X-axis vertical | |
| 2 | Run -12dB (checkout) | Control within ±3dB | |
| 3 | Run -6dB | Control within ±3dB | |
| 4 | Run full level | Control within ±3dB | |
| 5 | Monitor responses | No excessive response | |
| 6 | Record overall Grms | ______ Grms | |

#### Y-Axis Test

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Rotate to Y-axis | Verified | |
| 2 | Run -12dB (checkout) | Control within ±3dB | |
| 3 | Run -6dB | Control within ±3dB | |
| 4 | Run full level | Control within ±3dB | |
| 5 | Monitor responses | No excessive response | |
| 6 | Record overall Grms | ______ Grms | |

#### Z-Axis Test

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Rotate to Z-axis | Verified | |
| 2 | Run -12dB (checkout) | Control within ±3dB | |
| 3 | Run -6dB | Control within ±3dB | |
| 4 | Run full level | Control within ±3dB | |
| 5 | Monitor responses | No excessive response | |
| 6 | Record overall Grms | ______ Grms | |

### 4.5 Sine Signature (Post-Random)

| Axis | Pre-Test (Hz) | Post-Test (Hz) | Shift | P/F |
|------|---------------|----------------|-------|-----|
| X | ______ | ______ | <5% | |
| Y | ______ | ______ | <5% | |
| Z | ______ | ______ | <5% | |

### 4.6 Post-Test Verification

| Step | Action | Expected Result | Result | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Visual inspection | No visible damage | ______ | |
| 2 | Mass measurement | Same as pre-test | ______ kg | |
| 3 | Torque check | All fasteners tight | ______ | |
| 4 | Functional test | TP-SYS-001 pass | ______ | |
| 5 | Deployment test | Mechanisms operate | ______ | |

## 5. Pass/Fail Criteria

- No structural damage
- Mass change < 1%
- Resonance frequency shift < 5%
- Post-test functional test pass
- No loose hardware
- Deployment mechanisms operational

## 6. Abort Criteria

| Condition | Action |
|-----------|--------|
| Control response > +6dB of input | Abort test |
| Visible damage observed | Abort test |
| Accelerometer overrange | Reduce level |
| Equipment malfunction | Pause test |

## 7. Test Data Summary

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| X-axis Grms | 14.1 ± 10% | ______ | |
| Y-axis Grms | 14.1 ± 10% | ______ | |
| Z-axis Grms | 14.1 ± 10% | ______ | |
| Duration per axis | 60s ± 5% | ______ | |
| Resonance shift | < 5% | ______ | |
| Post-test functional | Pass | ______ | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
