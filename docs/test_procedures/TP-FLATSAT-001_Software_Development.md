# Test Procedure: Flatsat Software Development
## TP-FLATSAT-001

**Document Number:** SQSO-TP-FLATSAT-001
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SRS-F001, SRS-F011, SRS-F012, SRS-F013, SRS-F020, SRS-F023, SRS-F030, SRS-F031, SRS-F040, SRS-F042, SRS-F052, SRS-F062, SRS-I010, SRS-I021, SRS-Q011, SYS-304, SYS-323, SYS-324, SYS-523

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

1. `docs/integration/FLATSAT_CONFIGURATION.md`
2. `software/rust/README.md` (the flight software) and `CLAUDE.md` (build commands)
3. `software/flight/docs/FDIR_DESIGN.md`
4. `docs/interfaces/ICD_OBC_EPS.md`, `docs/interfaces/ICD_OBC_JETSON.md`

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Flatsat boards | EM boards when built (EPS, OBC with the STM32L4R5ZI, RF, magnetometer board A2, Jetson carrier); development boards until then (`docs/integration/FLATSAT_CONFIGURATION.md`) |
| Power supply | 0-12V, 5A (battery simulation) |
| Solar simulator | 0-6V, 3A programmable |
| Serial interface | USB-UART adapters (multiple) |
| Logic analyzer | 8+ channels |
| Development PC | Rust toolchain (`software/rust/rust-toolchain.toml`), probe-rs or ST-Link, serial terminal |

---

## 5. PRECONDITIONS

- [ ] Flatsat assembled per FLATSAT_CONFIGURATION.md
- [ ] All subsystem boards functional (standalone tests complete)
- [ ] Flight software compiled for flatsat target
- [ ] Debug UART enabled in software configuration
- [ ] Regression test suite ready

---

## 6. PROCEDURE

### 6.1 OBC Software Execution [SRS-F001, SRS-F020, SRS-I021]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on flatsat | All boards powered | |
| 2 | Connect debug UART to PC | Serial terminal ready | |
| 3 | Load flight software to OBC | Flash successful | |
| 4 | Reset OBC | Boot banner appears | |
| 5 | Verify boot sequence | All init messages OK | |
| 6 | Verify nominal telemetry | Valid values | |
| 7 | Verify beacon generation | AX.25 frame transmitted | |

### 6.2 Subsystem Interface Verification [SRS-F011, SRS-F023, SRS-F030, SRS-F031, SRS-I010, SYS-323, SYS-324]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Command EPS telemetry | Valid EPS data | |
| 2 | Command load switch toggle | Correct switching | |
| 3 | Verify ADCS sensor data | Valid sensor readings | |
| 4 | Command magnetorquer | Correct actuation | |
| 5 | Enable Jetson | Boot complete | |
| 6 | Request Jetson status | Status received | |
| 7 | Trigger a Jetson burst (gated experiment, D5) | Text generated; filtered on the OBC (`docs/BEACON_SPEC.md` §6) | |
| 8 | Verify RF beacon content | Template text from the OBC selector (flight baseline, D5); accepted Jetson lines only when the experiment is enabled | |

### 6.3 Mode Transition Test [SRS-F012, SRS-F013, SYS-304]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Start in BOOT | Mode = BOOT | |
| 2 | Complete boot | Transition to DETUMBLE (`software/rust/sqso-fsw/src/mode.rs`) | |
| 3 | Command transition to IDLE | Mode = IDLE | |
| 4 | Command transition to ACTIVE (SOC ≥ 55 %, sunlit) | Mode = ACTIVE | |
| 5 | In ACTIVE, start a Jetson burst | Burst runs within ACTIVE (there is no burst mode); ends within 10 min (BASELINE §6) | |
| 6 | Simulate falling SOC | IDLE below 40 %, SAFE below 25 %, EMERGENCY below 15 % (BASELINE §5) | |
| 7 | Recover power | SAFE at 25 %, IDLE above 40 % (BASELINE §5) | |

### 6.4 Fault Injection Testing [SRS-F040, SRS-F042, SRS-F052, SRS-F062, SYS-523]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Disconnect EPS I2C | Fault detected, logged | |
| 2 | Reconnect EPS I2C | Fault cleared | |
| 3 | Simulate sensor failure | Fallback to backup | |
| 4 | Simulate OBC reset | Clean recovery | |
| 5 | Corrupt configuration CRC | Fallback to defaults | |
| 6 | Simulate watchdog timeout | OBC resets safely | |
| 7 | Power cycle during write | Data integrity maintained | |
| 8 | Simulate Jetson hang | Jetson powered off within its 10 min burst limit or at the per-rail current limit; OBC, beacon and telecommand unaffected (SYS-522, SYS-523) | |

### 6.5 Regression Testing [SRS-Q011]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Run the Rust tests and the C test suites (`cargo test --workspace` in `software/rust`; `ctest --test-dir build`) | All tests pass | |
| 2 | Run the mission simulation (`docs/test/SIL_MISSION_SIMULATION.md`) | All checks pass | |
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
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
