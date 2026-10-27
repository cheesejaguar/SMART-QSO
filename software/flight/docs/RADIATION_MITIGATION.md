# Radiation Mitigation Design
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-RAD-001
**Revision:** D
**Date:** 2026-10-27
**Reference:** NASA-HDBK-4002A

---

> **Revision B status note.** Sections 2 to 10 are the January 2026 design intent for a C flight software and the ISS
> orbit. They are **not** the as-built design. The environment, the rates and the as-built mitigation status are in
> `docs/environment/RADIATION_ANALYSIS.md` (Revision F; §4 to §7; the baseline orbit is a 475 km SSO for 2 years, BASELINE §3, §6) and summarised in §11 below. Where the two disagree,
> §11 and the radiation analysis govern.

## 1. Purpose

This document defines the radiation mitigation strategies implemented in SMART-QSO flight software to ensure reliable operation in the Low Earth Orbit (LEO) radiation environment.

## 2. Radiation Environment

### 2.1 Mission Environment (500 km, 51.6° ISS orbit)

| Source | Annual Dose | Mitigation |
|--------|-------------|------------|
| Trapped protons | ~5 krad (Si) | Shielding, SEU mitigation |
| Trapped electrons | ~1 krad (Si) | Shielding |
| Solar protons | Event-dependent | Safe mode during SPE |
| Galactic cosmic rays | ~0.1 krad (Si) | SEU mitigation |
| South Atlantic Anomaly | Higher flux region | Enhanced monitoring |

### 2.2 Single Event Effects

| Effect | Description | Mitigation Strategy |
|--------|-------------|---------------------|
| SEU (Single Event Upset) | Bit flip in memory | ECC, TMR, checksums |
| SET (Single Event Transient) | Transient on signal | Filtering, voting |
| SEL (Single Event Latchup) | Parasitic SCR trigger | Current limiting, power cycling |
| SEFI (Single Event Functional Interrupt) | Processor hang | Watchdog reset |

## 3. Memory Protection

### 3.1 RAM Protection

```c
/**
 * Triple Modular Redundancy for critical variables
 * Per NASA-HDBK-4002A Section 6.3
 */
typedef struct {
    uint32_t value_a;
    uint32_t value_b;
    uint32_t value_c;
} TmrValue_t;

/**
 * @brief Read TMR-protected value with voting
 * @param tmr Pointer to TMR structure
 * @return Voted value (majority wins)
 */
static inline uint32_t tmr_read(const TmrValue_t *tmr) {
    if (tmr->value_a == tmr->value_b) {
        return tmr->value_a;
    } else if (tmr->value_a == tmr->value_c) {
        return tmr->value_a;
    } else {
        return tmr->value_b;
    }
}

/**
 * @brief Write TMR-protected value
 * @param tmr Pointer to TMR structure
 * @param value Value to write
 */
static inline void tmr_write(TmrValue_t *tmr, uint32_t value) {
    tmr->value_a = value;
    tmr->value_b = value;
    tmr->value_c = value;
}
```

### 3.2 Critical Variable Protection

| Variable | Protection Method | Scrub Interval |
|----------|-------------------|----------------|
| Power mode | TMR + CRC | 1 second |
| Battery SOC | TMR | 1 second |
| Fault counters | TMR | 10 seconds |
| ADCS state | TMR + bounds check | 100 ms |
| Mission time | TMR + monotonic check | 1 second |
| Config parameters | CRC32 | On access |

### 3.3 Memory Scrubbing

```c
/**
 * @brief Periodic memory scrub task
 * Runs every 10 seconds to detect and correct SEUs
 */
void memory_scrub_task(void) {
    static uint32_t scrub_address = 0;

    for (uint32_t i = 0; i < SCRUB_BLOCK_SIZE; i++) {
        // Read and rewrite to trigger ECC correction
        volatile uint32_t *addr = (volatile uint32_t *)(RAM_START + scrub_address);
        uint32_t value = *addr;
        *addr = value;

        scrub_address += sizeof(uint32_t);
        if (scrub_address >= RAM_SIZE) {
            scrub_address = 0;
        }
    }
}
```

## 4. Code Protection

### 4.1 Flash Memory Protection

| Protection | Implementation |
|------------|----------------|
| CRC32 on code sections | Verified at boot and periodically |
| Redundant code storage | Primary and backup partitions |
| Boot ROM integrity | Hardware-protected boot vector |

### 4.2 Code Integrity Verification

```c
/**
 * @brief Verify flash code integrity
 * @return SMART_QSO_OK if integrity verified
 */
SmartQsoResult_t verify_code_integrity(void) {
    uint32_t stored_crc = *(uint32_t *)CODE_CRC_ADDRESS;
    uint32_t calculated_crc = crc32_calculate(
        (uint8_t *)CODE_START,
        CODE_SIZE
    );

    if (stored_crc != calculated_crc) {
        fault_log(FAULT_TYPE_MEMORY, "Code CRC mismatch");
        return SMART_QSO_ERROR;
    }

    return SMART_QSO_OK;
}
```

### 4.3 Periodic Integrity Checks

| Check | Interval | Action on Failure |
|-------|----------|-------------------|
| Code CRC | 1 hour | Log fault, attempt recovery |
| Stack canary | 1 second | Reset if corrupted |
| Heap integrity | 10 seconds | Log fault |
| Config CRC | On access | Use backup |

## 5. Watchdog Strategy

### 5.1 Multi-Level Watchdog

```c
/**
 * Watchdog hierarchy for comprehensive fault detection
 */
typedef enum {
    WDT_LEVEL_HARDWARE,    // MCU hardware watchdog (30s)
    WDT_LEVEL_SOFTWARE,    // Software watchdog task (10s)
    WDT_LEVEL_TASK         // Per-task heartbeat (5s)
} WatchdogLevel_t;
```

### 5.2 Watchdog Configuration

| Level | Timeout | Reset Action |
|-------|---------|--------------|
| Hardware WDT | 30 seconds | Full system reset |
| Software WDT | 10 seconds | Task restart attempt |
| Task heartbeat | 5 seconds | Task-specific recovery |

### 5.3 Watchdog Kick Requirements

```c
/**
 * @brief Main loop with watchdog servicing
 * All critical tasks must complete within watchdog window
 */
void main_loop(void) {
    while (1) {
        // Service hardware watchdog
        watchdog_kick();

        // Execute tasks with timing monitoring
        uint32_t start_time = get_system_time_ms();

        execute_scheduled_tasks();

        uint32_t elapsed = get_system_time_ms() - start_time;
        if (elapsed > LOOP_TIME_LIMIT_MS) {
            fault_log(FAULT_TYPE_TIMING, "Loop overrun");
        }

        // Sleep until next cycle
        sleep_until_next_cycle();
    }
}
```

## 6. Safe Mode Design

### 6.1 Safe Mode Triggers

| Trigger | Threshold | Response |
|---------|-----------|----------|
| Watchdog reset | 3 consecutive | Enter safe mode |
| Memory corruption | CRC failure | Enter safe mode |
| Over-temperature | > 70°C | Reduce power, safe mode |
| Under-voltage | < 6.0V | Safe mode |
| Communication loss | > 48 hours | Beacon-only mode |

### 6.2 Safe Mode Configuration

```c
/**
 * Safe mode minimizes power and complexity
 */
typedef struct {
    bool eps_safe_limits;     // Conservative power limits
    bool adcs_detumble_only;  // No pointing control
    bool rf_low_power;        // Reduced TX power
    bool jetson_disabled;     // AI payload off
    bool beacons_only;        // No command processing
} SafeModeConfig_t;

static const SafeModeConfig_t SAFE_MODE_CONFIG = {
    .eps_safe_limits = true,
    .adcs_detumble_only = true,
    .rf_low_power = true,
    .jetson_disabled = true,
    .beacons_only = true
};
```

## 7. South Atlantic Anomaly Handling

### 7.1 SAA Detection

```c
/**
 * @brief Check if spacecraft is in SAA region
 * Based on simplified geographic bounds
 */
bool is_in_saa_region(double lat, double lon) {
    // SAA approximate bounds
    const double SAA_LAT_MIN = -50.0;
    const double SAA_LAT_MAX = 0.0;
    const double SAA_LON_MIN = -90.0;
    const double SAA_LON_MAX = 40.0;

    return (lat >= SAA_LAT_MIN && lat <= SAA_LAT_MAX &&
            lon >= SAA_LON_MIN && lon <= SAA_LON_MAX);
}
```

### 7.2 SAA Mitigation Actions

| Action | Purpose |
|--------|---------|
| Increase scrub rate | Faster SEU detection |
| Defer non-critical writes | Reduce corruption risk |
| Log radiation events | Trend analysis |
| Suspend Jetson operations | Protect AI payload |

## 8. Latchup Protection

### 8.1 Current Monitoring

```c
/**
 * @brief Monitor for latchup conditions
 * @param rail Power rail to monitor
 * @return true if latchup detected
 */
bool check_latchup(PowerRail_t rail) {
    double current = eps_read_current(rail);

    if (current > LATCHUP_THRESHOLD_MA) {
        fault_log(FAULT_TYPE_LATCHUP, "Latchup on rail %d", rail);
        return true;
    }
    return false;
}
```

### 8.2 Latchup Recovery

| Step | Action | Timeout |
|------|--------|---------|
| 1 | Detect overcurrent | Immediate |
| 2 | Power off affected rail | 100 ms |
| 3 | Wait for discharge | 1 second |
| 4 | Power on rail | 100 ms |
| 5 | Verify normal current | 500 ms |
| 6 | Log event and continue | Immediate |

## 9. Data Integrity

### 9.1 Telemetry Protection

```c
/**
 * All telemetry includes CRC for ground verification
 */
typedef struct __attribute__((packed)) {
    uint32_t timestamp;
    uint8_t data[TLM_DATA_SIZE];
    uint32_t crc32;
} TelemetryFrame_t;
```

### 9.2 Command Validation

| Check | Implementation |
|-------|----------------|
| CRC verification | CRC32 on command packet |
| Sequence number | Reject out-of-order |
| Range checking | All parameters bounds-checked |
| Command echo | Confirm receipt |

## 10. Testing and Verification

### 10.1 Fault Injection Testing

| Test | Method | Expected Result |
|------|--------|-----------------|
| Bit flip in RAM | Debug interface injection | Detection and correction |
| Watchdog timeout | Disable kick | System reset |
| CRC corruption | Modify flash byte | Detection and recovery |
| Stack overflow | Recursive call | Canary detection |

### 10.2 Radiation Test Correlation

| Ground Test | On-Orbit Correlation |
|-------------|---------------------|
| Heavy ion SEU rate | TID-adjusted prediction |
| Proton SEU rate | SAA passage events |
| Latchup threshold | Current trip events |

## 11. As-built Status (Revision C, 2026-10-27)

| Rev A mitigation | Section | As built in the Rust flight software | Status |
|------------------|---------|--------------------------------------|--------|
| SRAM EDAC / parity on critical data | 3.1 | Every static (the live state) linked in SRAM2, the parity RAM; SRAM2 erased before use; a parity error raises the NMI, which records the cause and resets at once; the cause is logged at the next boot (`software/rust/firmware/src/parity.rs`, SRS-F132; MEMORY_POLICY §10.3). Parity itself is enabled by the option bit `SRAM2_PE`, set when the board is programmed (logged at every boot if off). The stack is in SRAM2 too since flight software round 3 (initial SP 0x20040000, RID AV-13; MEMORY_POLICY §10.5) | Implemented (AV-07); not run on hardware (RAD-V6) |
| TMR of critical variables | 3.2 | Not implemented. The critical states (TX-OFF, RF silence, deployment, commissioning hold, passivation) are persisted in FRAM records with CRC and two slots, re-read at every boot, and the TX-OFF latch, the operations record and the passivation record are checked against their RAM copy every 60 s, settling toward the safe side (SRS-F133, MEMORY_POLICY §10.4) | Tailored: persistence, parity and the periodic check instead of TMR (AV-07) |
| Memory scrubbing | 3.3 | Not implemented: parity detects an upset when the byte is read (MEMORY_POLICY §10.2, §10.5) | Not needed for FRAM; for SRAM, tailored (AV-07) |
| Flash ECC, image CRC | 4.1–4.3 | Flash ECC corrects single-bit errors in hardware; a double-bit error raises the NMI, which records the address and resets at once (as for parity); no periodic image CRC check | Implemented for ECC (AV-07); image CRC open |
| Multi-level watchdog | 5 | IWDG 30 s, refreshed only when every stage of the pass has checked in (`software/rust/sqso-ffi/src/watchdog_mgr.rs`); no window watchdog, no external supervisor | Implemented; external supervisor open (AV-02) |
| Safe mode | 6 | Mode manager with SAFE and EMERGENCY (`software/rust/sqso-ffi/src/state_machine.rs`) | Implemented |
| SAA handling | 7 | Not implemented | Not needed: the upset rates of RADIATION_ANALYSIS §5 do not justify it |
| Latch-up current monitoring | 8 | Software guard on the Jetson rail only (`payload_guard`, 12 W single sample); hardware eFuse on the Jetson carrier (EPS design); the GNSS receiver is powered only for sessions of ≤ 5 min; no trip specified for the OBC, radio and sensor rails | Partial (AV-03, AV-04, AV-08) |
| Command validation | 9.2 | HMAC-SHA-256 authentication with a persisted replay counter (`cmd_auth`), length and range checks per command, a checksum on the frame | Implemented (stronger than Rev A's CRC and sequence number) |
| Receiver SEFI recovery | — | After 24 h without a valid frame or a carrier, the radio is power-cycled (RADIO_EN off 2 s), never in a pass, logged (`software/rust/sqso-fsw/src/receiver.rs`, SRS-F115); the mission simulation models receiver SEFIs (one per 120 days) and checks each is cleared within 26 h | Implemented (FSW-09); the carrier-detect and health inputs need OBC pins and RF-board lines |
| Fault-injection tests | 10.1 | Host tests of every persisted record's corruption (C suites); no RAM bit-flip injection on target | Open (RADIATION_ANALYSIS RAD-V6) |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | Status note: §2–§10 are Rev A design intent, not as built; §11 as-built status of each mitigation against the Rust flight software and `docs/environment/RADIATION_ANALYSIS.md` Rev C (RIDs AV-02, AV-03, AV-04, AV-07, AV-08, FSW-09) | SMART-QSO Team (CDR avionics review) |
| C | 2026-10-27 | §11: SRAM2 parity and the NMI handler, flash ECC handling, the periodic check of the persisted state (AV-07) and the receiver recovery (FSW-09) as implemented in the flight software round 2 | SMART-QSO Team (CDR flight software, round 2) |
| D | 2026-10-27 | Documentation audit (post-CDR 2026): status note cites RADIATION_ANALYSIS Rev F and the baseline orbit; §11 the stack in SRAM2 (was SRAM3) | SMART-QSO Team |
