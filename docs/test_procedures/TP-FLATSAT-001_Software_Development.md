# Test Procedure: Flatsat Software Development
## TP-FLATSAT-001

**Document Number:** SQSO-TP-FLATSAT-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure defines the software development and verification testing on the flatsat configuration prior to flight hardware integration.

---

## 2. SCOPE

- OBC software execution on flatsat
- Subsystem interface verification
- Fault injection testing
- Regression testing
- Extended duration testing (Day-in-the-Life)

---

## 3. APPLICABLE DOCUMENTS

1. docs/integration/FLATSAT_CONFIGURATION.md
2. software/flight/README.md
3. docs/design/FDIR_DESIGN.md
4. ICD_OBC_EPS.md, ICD_OBC_JETSON.md

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Flatsat boards | EPS dev, OBC dev, RF dev, ADCS breakout, Jetson dev kit |
| Power supply | 0-12V, 5A (battery simulation) |
| Solar simulator | 0-6V, 3A programmable |
| Serial interface | USB-UART adapters (multiple) |
| Logic analyzer | 8+ channels |
| Development PC | STM32CubeIDE, serial terminal |

---

## 5. PRECONDITIONS

- [ ] Flatsat assembled per FLATSAT_CONFIGURATION.md
- [ ] All subsystem boards functional (standalone tests complete)
- [ ] Flight software compiled for flatsat target
- [ ] Debug UART enabled in software configuration
- [ ] Regression test suite ready

---

## 6. PROCEDURE

### 6.1 OBC Software Execution

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on flatsat | All boards powered | |
| 2 | Connect debug UART to PC | Serial terminal ready | |
| 3 | Load flight software to OBC | Flash successful | |
| 4 | Reset OBC | Boot banner appears | |
| 5 | Verify boot sequence | All init messages OK | |
| 6 | Verify nominal telemetry | Valid values | |
| 7 | Verify beacon generation | AX.25 frame transmitted | |

### 6.2 Subsystem Interface Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Command EPS telemetry | Valid EPS data | |
| 2 | Command load switch toggle | Correct switching | |
| 3 | Verify ADCS sensor data | Valid sensor readings | |
| 4 | Command magnetorquer | Correct actuation | |
| 5 | Enable Jetson | Boot complete | |
| 6 | Request Jetson status | Status received | |
| 7 | Trigger Jetson inference | Text generated | |
| 8 | Verify RF beacon content | Contains Jetson text | |

### 6.3 Mode Transition Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Start in INIT mode | Mode = INIT | |
| 2 | Complete init sequence | Transition to SAFE | |
| 3 | Command transition to IDLE | Mode = IDLE | |
| 4 | Command transition to ACTIVE | Mode = ACTIVE | |
| 5 | Command transition to AI_BURST | Mode = AI_BURST | |
| 6 | Simulate low power | Auto-transition to SAFE | |
| 7 | Recover power | Return to previous mode | |

### 6.4 Fault Injection Testing

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Disconnect EPS I2C | Fault detected, logged | |
| 2 | Reconnect EPS I2C | Fault cleared | |
| 3 | Simulate sensor failure | Fallback to backup | |
| 4 | Simulate OBC reset | Clean recovery | |
| 5 | Corrupt configuration CRC | Fallback to defaults | |
| 6 | Simulate watchdog timeout | OBC resets safely | |
| 7 | Power cycle during write | Data integrity maintained | |
| 8 | Simulate Jetson hang | Jetson power-cycled | |

### 6.5 Regression Testing

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Run unit test suite | All tests pass | |
| 2 | Run integration test suite | All tests pass | |
| 3 | Verify code coverage | >= 80% achieved | |
| 4 | Run static analysis | No new issues | |
| 5 | Compare to baseline | No regressions | |

### 6.6 Extended Duration Test (72+ hours)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Start continuous operation | System operational | |
| 2 | Enable orbital simulation | Light/dark cycling | |
| 3 | Enable power profile | Varying power budget | |
| 4 | Log all telemetry | Data recorded | |
| 5 | Monitor for anomalies | No unexpected resets | |
| 6 | Monitor memory usage | No leaks | |
| 7 | After 72 hours: full functional | All functions nominal | |
| 8 | Analyze trend data | Parameters stable | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Boot success | 100% reliable boot |
| Interface tests | All interfaces functional |
| Mode transitions | All transitions correct |
| Fault detection | All faults detected |
| Fault recovery | All recoveries successful |
| Regression tests | Zero regressions |
| Extended duration | No anomalies in 72+ hours |
| Memory | No memory leaks |

---

## 8. DATA RECORDING

- [ ] Serial console logs
- [ ] Telemetry recordings
- [ ] Power measurements
- [ ] Fault injection results
- [ ] Regression test reports
- [ ] Extended duration trend analysis

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
