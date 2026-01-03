# Test Procedure: Final Acceptance Functional Test
## TP-FAT-001

**Document Number:** SQSO-TP-FAT-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure defines the final acceptance functional test performed on the flight unit after all environmental testing to verify flight readiness.

---

## 2. SCOPE

- Complete functional test of all subsystems
- RF characterization (final)
- Battery capacity verification
- Mass properties final measurement
- Configuration verification

---

## 3. APPLICABLE DOCUMENTS

1. TP-SYS-001_Comprehensive_Functional.md
2. TP-ENV-003_Thermal_Vacuum.md (prerequisite)
3. TP-ENV-002_Vibration.md (prerequisite)
4. TP-ENV-005_Deployment_Post_Env.md (prerequisite)
5. System Requirements Specification

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Power supply | 0-12V, 5A |
| Power analyzer | DC, 0-30V, 0-5A |
| Spectrum analyzer | 100 kHz - 3 GHz |
| Signal generator | 100 kHz - 3 GHz |
| RF power meter | 0.1 W - 10 W |
| Battery cycler | 0-5V, 0-5A per channel |
| Mass scale | 0-5 kg, 0.1 g resolution |
| CG measurement fixture | 3-axis |
| Development PC | Ground station software |

---

## 5. PRECONDITIONS

- [ ] All environmental testing complete
- [ ] Deployment test complete
- [ ] Flight configuration verified
- [ ] Battery installed and charged
- [ ] Software at flight version
- [ ] All prior test anomalies resolved

---

## 6. PROCEDURE

### 6.1 Pre-Test Setup

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Visual inspection | No damage | |
| 2 | Verify flight configuration | Per as-built list | |
| 3 | Record ambient conditions | T, RH, P logged | |
| 4 | Connect GSE power | 4.0V battery sim | |
| 5 | Connect ground station | Serial link active | |
| 6 | Connect RF to spectrum analyzer | Via 30dB attenuator | |

### 6.2 Power-On and Boot Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Apply power | Boot sequence starts | |
| 2 | Record boot time | < 30 seconds | |
| 3 | Verify boot telemetry | All init complete | |
| 4 | Check fault flags | No faults | |
| 5 | Record idle power | Per spec | |

### 6.3 Complete Functional Test

#### 6.3.1 EPS Subsystem

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify all power rails | 3.3V, 5V within ±5% | |
| 2 | Verify battery telemetry | V, I, T correct | |
| 3 | Command load switch toggle | Correct response | |
| 4 | Verify solar array input | MPPT tracking | |
| 5 | Verify power mode transitions | All modes functional | |

#### 6.3.2 OBC Subsystem

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify CPU status | Running normally | |
| 2 | Verify memory status | No errors | |
| 3 | Verify software version | Flight version | |
| 4 | Verify configuration CRC | Valid | |
| 5 | Verify watchdog function | Kick regular | |
| 6 | Test all mode transitions | Correct behavior | |

#### 6.3.3 RF Subsystem

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify RF power off | No RF output | |
| 2 | Command RF on | TX active | |
| 3 | Measure TX frequency | 145.825 MHz ±2 kHz | |
| 4 | Measure TX power | 0.5W ±0.1W (low mode) | |
| 5 | Measure TX power | 2.0W ±0.2W (high mode) | |
| 6 | Verify modulation | AFSK 1200 baud | |
| 7 | Verify beacon content | Valid AX.25 frame | |
| 8 | Command RF off | TX disabled | |

#### 6.3.4 ADCS Subsystem

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify magnetometer | Valid field reading | |
| 2 | Verify sun sensors | Valid readings | |
| 3 | Verify gyroscope | Valid rate reading | |
| 4 | Command magnetorquer X | Correct polarity/current | |
| 5 | Command magnetorquer Y | Correct polarity/current | |
| 6 | Command magnetorquer Z | Correct polarity/current | |
| 7 | Verify ADCS mode transitions | Correct behavior | |

#### 6.3.5 Payload Subsystem

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify Jetson power off | < 1 mA standby | |
| 2 | Command Jetson power on | Boot sequence | |
| 3 | Verify Jetson boot complete | Status received | |
| 4 | Request health status | Valid JSON | |
| 5 | Request inference | Text generated | |
| 6 | Verify beacon integration | AI text in beacon | |
| 7 | Verify thermal status | < 70°C | |
| 8 | Command Jetson shutdown | Clean shutdown | |
| 9 | Command power off | < 1 mA standby | |

### 6.4 RF Characterization (Final)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure TX power at 0.5W | 0.5W ±0.1W | |
| 2 | Measure TX power at 2.0W | 2.0W ±0.2W | |
| 3 | Measure harmonics | 2nd < -30 dBc | |
| 4 | Measure harmonics | 3rd < -40 dBc | |
| 5 | Measure spurious | < -50 dBc | |
| 6 | Measure antenna VSWR | < 2:1 | |
| 7 | Compare to pre-env baseline | Within 10% | |

### 6.5 Battery Capacity Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Charge battery to 100% | 4.2V | |
| 2 | Discharge at C/5 | Monitor capacity | |
| 3 | Measure capacity | > 3.0 Ah (or spec) | |
| 4 | Compare to BOL | > 95% of original | |
| 5 | Recharge for flight | 100% SOC | |

### 6.6 Mass Properties Measurement

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure total mass | Record value | |
| 2 | Verify mass budget | < 1.33 kg | |
| 3 | Measure CG (X axis) | Record value | |
| 4 | Measure CG (Y axis) | Record value | |
| 5 | Measure CG (Z axis) | Record value | |
| 6 | Verify CG within 2 cm | Per CDS Rev 14.1 | |
| 7 | Compare to pre-env | Within tolerance | |

### 6.7 Configuration Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify software version | Flight version | |
| 2 | Verify configuration parameters | Per flight config | |
| 3 | Verify callsign | N0CALL or assigned | |
| 4 | Verify beacon text | Correct format | |
| 5 | Verify ADCS parameters | Per mission config | |
| 6 | Verify power thresholds | Per mission config | |
| 7 | Verify deployment timer | T+30 min | |
| 8 | Generate config hash | Record for reference | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Boot | < 30 seconds, no faults |
| Power rails | ±5% of nominal |
| TX frequency | 145.825 MHz ±2 kHz |
| TX power | Per spec ±10% |
| Harmonics | Per spec |
| All sensors | Valid readings |
| All actuators | Correct response |
| Jetson inference | Valid text output |
| Battery capacity | > 95% of BOL |
| Mass | < 1.33 kg |
| CG offset | < 2 cm from center |
| Configuration | Matches flight baseline |

---

## 8. DATA RECORDING

- [ ] All telemetry logs
- [ ] RF measurement data
- [ ] Battery capacity data
- [ ] Mass properties data
- [ ] Configuration verification record
- [ ] Test photos
- [ ] Anomaly notes (if any)

---

## 9. SIGNATURES

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Test Engineer | | | |
| Quality Assurance | | | |
| Systems Engineer | | | |
| Project Manager | | | |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
