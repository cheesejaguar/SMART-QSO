# Test Procedure: OBC Functional Test
## TP-OBC-001

**Document Number:** TP-OBC-001
**Revision:** A
**Date:** 2026-01-02

---

## 1. Purpose

Verify complete functionality of the On-Board Computer (OBC) subsystem including boot sequence, software execution, watchdog operation, and subsystem interfaces.

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Debug probe | JTAG/SWD compatible |
| Logic analyzer | 8+ channels, 100 MHz |
| Power supply | 3.3V ± 0.1V, 5A |
| Current meter | 0-2A, 1mA resolution |
| Terminal emulator | 115200 baud capable |
| Test harness | All OBC interfaces |

## 3. Prerequisites

- [ ] OBC hardware integration complete
- [ ] Flight software loaded
- [ ] Test harness connected
- [ ] Debug console available

## 4. Procedure

### 4.1 Power-On Sequence

| Step | Action | Expected Result | Measured | P/F |
|------|--------|-----------------|----------|-----|
| 1 | Apply 3.3V to OBC | Current < 100mA initial | ______ mA | |
| 2 | Monitor boot LED | LED blinks during boot | ______ | |
| 3 | Measure boot time | Boot complete < 30 sec | ______ sec | |
| 4 | Check console output | Banner message displayed | ______ | |
| 5 | Measure steady-state current | < 50mA average | ______ mA | |

### 4.2 Software Execution

| Step | Action | Expected Result | Measured | P/F |
|------|--------|-----------------|----------|-----|
| 1 | Verify main loop rate | 1 Hz ± 10% | ______ Hz | |
| 2 | Check task scheduling | All tasks executing | ______ | |
| 3 | Monitor CPU usage | < 80% utilization | ______ % | |
| 4 | Check memory usage | < 90% RAM utilized | ______ % | |
| 5 | Verify no stack overflow | Stack markers intact | ______ | |

### 4.3 Watchdog Operation

| Step | Action | Expected Result | Measured | P/F |
|------|--------|-----------------|----------|-----|
| 1 | Verify watchdog active | WDT timer running | ______ | |
| 2 | Disable watchdog kick | System reset < 30 sec | ______ sec | |
| 3 | Verify reset detection | Reset cause logged | ______ | |
| 4 | Re-enable watchdog | Normal operation resumes | ______ | |

### 4.4 Interface Tests

#### 4.4.1 UART Interfaces

| Interface | Baud Rate | TX Test | RX Test | P/F |
|-----------|-----------|---------|---------|-----|
| Debug | 115200 | ______ | ______ | |
| Jetson | 115200 | ______ | ______ | |
| EPS | 9600 | ______ | ______ | |

#### 4.4.2 I2C Interfaces

| Device | Address | Read | Write | P/F |
|--------|---------|------|-------|-----|
| Magnetometer | 0x1E | ______ | ______ | |
| Temperature | 0x48 | ______ | ______ | |
| EEPROM | 0x50 | ______ | ______ | |

#### 4.4.3 SPI Interfaces

| Device | Speed | Read | Write | P/F |
|--------|-------|------|-------|-----|
| Flash | 10 MHz | ______ | ______ | |
| ADC | 1 MHz | ______ | ______ | |

#### 4.4.4 GPIO Interfaces

| Signal | Direction | Test | Result | P/F |
|--------|-----------|------|--------|-----|
| EPS_EN | Output | Toggle | ______ | |
| RF_EN | Output | Toggle | ______ | |
| JETSON_EN | Output | Toggle | ______ | |
| DEPLOY_SW | Input | Read | ______ | |
| RBF_SW | Input | Read | ______ | |

### 4.5 Data Persistence

| Step | Action | Expected Result | Measured | P/F |
|------|--------|-----------------|----------|-----|
| 1 | Write test data to flash | Write succeeds | ______ | |
| 2 | Read back test data | Data matches | ______ | |
| 3 | Verify CRC protection | CRC valid | ______ | |
| 4 | Power cycle OBC | Data retained | ______ | |
| 5 | Verify data after reset | Data matches | ______ | |

### 4.6 Fault Detection

| Step | Fault Injection | Expected Response | Result | P/F |
|------|-----------------|-------------------|--------|-----|
| 1 | Sensor timeout | Fault logged, safe mode | ______ | |
| 2 | Invalid command | Command rejected | ______ | |
| 3 | Memory corruption | CRC error detected | ______ | |
| 4 | Multiple resets | Recovery mode entered | ______ | |

## 5. Pass/Fail Criteria

- Boot time: < 30 seconds
- Main loop rate: 1 Hz ± 10%
- Watchdog timeout: < 30 seconds
- All interfaces functional
- Data persistence verified
- Fault detection operational

## 6. Test Data

### 6.1 Boot Timing Log

| Event | Time (ms) |
|-------|-----------|
| Power applied | 0 |
| First instruction | ______ |
| Peripheral init complete | ______ |
| Main loop start | ______ |
| Boot complete | ______ |

### 6.2 Current Profile

| State | Current (mA) |
|-------|--------------|
| Boot peak | ______ |
| Idle | ______ |
| Processing | ______ |
| All interfaces active | ______ |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
