# Test Procedure: OBC Functional Test
## TP-OBC-001

**Document Number:** TP-OBC-001
**Revision:** B
**Date:** 2026-01-02

**Requirements verified:** SYS-021, SYS-300, SYS-302, SYS-312, SRS-F001, SRS-F040, SRS-F043, SRS-F050, SRS-F052, SRS-F060, SRS-F061, SRS-F062, SRS-I001, SRS-S003, SRS-S004

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

### 4.1 Power-On Sequence [SYS-300, SRS-F001]

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
| 1 | Verify main loop pass period | 20 ms wait plus the pass time; each pass within 20 ms (SRS-P001; `software/flight/docs/RTOS_ARCHITECTURE.md` §6.1) | ______ ms | |
| 2 | Check the stages of the main loop | One Embassy task, every stage run each pass in the order of RTOS_ARCHITECTURE §4.2 (no RTOS) | ______ | |
| 3 | Monitor CPU usage | < 80% utilization | ______ % | |
| 4 | Check memory usage | < 90% RAM utilized | ______ % | |
| 5 | Verify no stack overflow | Stack markers intact | ______ | |

### 4.3 Watchdog Operation [SYS-021, SRS-F050, SRS-F052, SRS-S003]

| Step | Action | Expected Result | Measured | P/F |
|------|--------|-----------------|----------|-----|
| 1 | Verify watchdog active | WDT timer running | ______ | |
| 2 | Disable watchdog kick | System reset < 30 sec | ______ sec | |
| 3 | Verify reset detection | Reset cause logged | ______ | |
| 4 | Re-enable watchdog | Normal operation resumes | ______ | |

### 4.4 Interface Tests [SYS-302, SRS-I001]

#### 4.4.1 UART Interfaces

| Interface | Baud Rate | TX Test | RX Test | P/F |
|-----------|-----------|---------|---------|-----|
| Console and key loading (USART3, access port) | Per SQSO-ICX-001 | ______ | ______ | |
| Jetson (USART1; OBC transmits 8N2) | 115200 | ______ | ______ | |
| RF board (USART2) | Per SQSO-ICX-001 | ______ | ______ | |
| GNSS receiver (LPUART1) | Per SQSO-ICX-001 | ______ | ______ | |

#### 4.4.2 I2C Interfaces

| Device | Address | Read | Write | P/F |
|--------|---------|------|-------|-----|
| EPS controller (I2C2) | 0x48 | ______ | ______ | |
| RTC RV-3028 (I2C1) | 0x52 | ______ | ______ | |
| Magnetometer RM3100 (I2C4, board A2) | 0x20 | ______ | ______ | |
| Sun-sensor ADC ADS7138 (I2C4) | Per SQSO-ICX-001 | ______ | ______ | |
| AntS controllers (I2C3) | 0x31, 0x32 | ______ | ______ | |

#### 4.4.3 SPI Interfaces

| Device | Speed | Read | Write | P/F |
|--------|-------|------|-------|-----|
| FRAM U3 and mirror U36 (FM25V20A, SPI1) | 20 MHz | ______ | ______ | |
| Gyro M-G370PDT0 (SPI3) | Per SQSO-ICX-001 | ______ | ______ | |
| Backup gyro BMI088 (SPI2) | Per SQSO-ICX-001 | ______ | ______ | |

#### 4.4.4 GPIO Interfaces

| Signal | Direction | Test | Result | P/F |
|--------|-----------|------|--------|-----|
| PAYLOAD_EN, JETSON_PWR | Output | Toggle | ______ | |
| RADIO_EN, RF_TX_EN | Output | Toggle | ______ | |
| ADCS_EN, GNSS_EN, HEATER_EN | Output | Toggle | ______ | |
| DEPLOY_EN and the four burn lines (into dummy loads) | Output | Toggle | ______ | |
| DEPLOY_SENSE, UMBILICAL_SENSE | Input | Read | ______ | |

### 4.5 Data Persistence [SYS-312, SRS-F060, SRS-F061]

| Step | Action | Expected Result | Measured | P/F |
|------|--------|-----------------|----------|-----|
| 1 | Write test data to FRAM (both copies) | Write succeeds | ______ | |
| 2 | Read back test data | Data matches | ______ | |
| 3 | Verify CRC protection | CRC valid | ______ | |
| 4 | Power cycle OBC | Data retained | ______ | |
| 5 | Verify data after reset | Data matches | ______ | |

### 4.6 Fault Detection [SRS-F040, SRS-F043, SRS-F062, SRS-S004]

| Step | Fault Injection | Expected Response | Result | P/F |
|------|-----------------|-------------------|--------|-----|
| 1 | Sensor timeout | Fault logged, safe mode | ______ | |
| 2 | Invalid command | Command rejected | ______ | |
| 3 | Memory corruption | CRC error detected | ______ | |
| 4 | Multiple resets | Recovery mode entered | ______ | |

## 5. Pass/Fail Criteria

- Boot time: < 30 seconds
- Main loop: each pass within 20 ms (SRS-P001)
- Watchdog timeout: 24.1–27.8 s (IWDG; TP-OBC-004)
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
| B | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) | SMART-QSO Team |
