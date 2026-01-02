# Fault Detection, Isolation, and Recovery Design
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-FDIR-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA-HDBK-1002, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the Fault Detection, Isolation, and Recovery (FDIR) architecture for the SMART-QSO CubeSat flight software, ensuring mission continuity and spacecraft safety.

## 2. FDIR Philosophy

### 2.1 Design Principles

| Principle | Implementation |
|-----------|----------------|
| Fail-safe | Default to safe state on unknown fault |
| Fail-operational | Maintain basic function after fault |
| Defense in depth | Multiple detection layers |
| Minimal ground dependence | Autonomous fault handling |
| Transparency | All faults logged and reported |

### 2.2 FDIR Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                     FDIR HIERARCHY                           │
├─────────────────────────────────────────────────────────────┤
│ Level 4: Mission Control                                     │
│   • Ground-commanded recovery                                │
│   • Configuration updates                                    │
│   • Manual override                                          │
├─────────────────────────────────────────────────────────────┤
│ Level 3: System Manager                                      │
│   • Cross-subsystem fault handling                           │
│   • Mode transitions (safe mode entry)                       │
│   • Power mode management                                    │
├─────────────────────────────────────────────────────────────┤
│ Level 2: Subsystem Manager                                   │
│   • Subsystem-specific fault handling                        │
│   • Component reconfiguration                                │
│   • Redundancy switching                                     │
├─────────────────────────────────────────────────────────────┤
│ Level 1: Hardware/Local                                      │
│   • Hardware protection (overcurrent, etc.)                  │
│   • Watchdog timer                                           │
│   • Memory ECC                                               │
└─────────────────────────────────────────────────────────────┘
```

## 3. Fault Detection

### 3.1 Detection Methods

| Method | Description | Response Time |
|--------|-------------|---------------|
| Limit checking | Compare values against thresholds | < 100 ms |
| Rate monitoring | Detect abnormal rate of change | < 1 s |
| Reasonableness | Cross-check correlated values | < 1 s |
| Watchdog | Detect software hang | 30 s |
| Heartbeat | Monitor task execution | 5 s |
| CRC checking | Detect data corruption | On access |

### 3.2 Monitored Parameters

#### 3.2.1 EPS Monitoring

| Parameter | Low Limit | High Limit | Action |
|-----------|-----------|------------|--------|
| Battery voltage | 6.0 V | 8.5 V | Safe mode |
| Battery SOC | 10% | - | Emergency shutdown |
| Battery temp | -10°C | 45°C | Heater/load shed |
| Solar current | - | 1.5 A | Limit MPPT |
| Bus current | - | 2.0 A | Disable loads |

#### 3.2.2 Thermal Monitoring

| Parameter | Low Limit | High Limit | Action |
|-----------|-----------|------------|--------|
| OBC temp | -30°C | 70°C | Duty cycle reduction |
| Jetson temp | -20°C | 70°C | Power off payload |
| Battery temp | 0°C | 45°C | Charge inhibit |
| RF PA temp | - | 65°C | TX power reduction |

#### 3.2.3 ADCS Monitoring

| Parameter | Limit | Action |
|-----------|-------|--------|
| Angular rate | > 10°/s | Force detumble |
| Pointing error | > 30° | Reacquire sun |
| Magnetometer | Out of range | Use backup/stale |
| Sun sensors | All dark > 2 hours | Eclipse mode |

### 3.3 Fault Detection Implementation

```c
/**
 * Fault detection structure
 */
typedef struct {
    FaultId_t id;
    const char *name;
    FaultSeverity_t severity;
    double low_limit;
    double high_limit;
    uint32_t persist_time_ms;
    FaultAction_t action;
} FaultDetector_t;

/**
 * @brief Check parameter against fault detector
 * @param detector Fault detector definition
 * @param value Current parameter value
 * @return Fault status
 */
FaultStatus_t check_fault(
    const FaultDetector_t *detector,
    double value
);
```

## 4. Fault Isolation

### 4.1 Isolation Strategy

| Fault Type | Isolation Method |
|------------|------------------|
| Power fault | Disable affected rail |
| Sensor fault | Mark sensor invalid |
| Actuator fault | Disable actuator |
| Communication fault | Switch to backup path |
| Software fault | Restart task/reset |

### 4.2 Fault Isolation Matrix

| Symptom | Possible Causes | Isolation Tests |
|---------|-----------------|-----------------|
| High current | Short circuit, latchup | Rail isolation |
| No telemetry | Sensor fail, bus error | Individual sensor read |
| Attitude drift | Sensor bias, actuator fail | Sensor comparison |
| Comm loss | TX fail, antenna | TX test, RSSI check |
| Watchdog reset | SW hang, memory error | Boot log analysis |

### 4.3 Isolation Implementation

```c
/**
 * @brief Isolate faulty component
 * @param fault_id Detected fault ID
 * @return Isolation result
 */
IsolationResult_t isolate_fault(FaultId_t fault_id) {
    IsolationResult_t result = {0};

    switch (fault_id) {
        case FAULT_EPS_OVERCURRENT:
            result = isolate_power_rail(get_fault_rail());
            break;

        case FAULT_SENSOR_TIMEOUT:
            result = isolate_sensor(get_fault_sensor());
            break;

        case FAULT_ADCS_ACTUATOR:
            result = isolate_actuator(get_fault_actuator());
            break;

        default:
            result.status = ISOLATION_NOT_POSSIBLE;
            break;
    }

    log_isolation(fault_id, result);
    return result;
}
```

## 5. Fault Recovery

### 5.1 Recovery Actions

| Action | Description | Triggered By |
|--------|-------------|--------------|
| NONE | Log only | Minor faults |
| RETRY | Retry operation | Transient faults |
| RECONFIGURE | Switch to backup | Component failure |
| RESTART | Restart subsystem | Software fault |
| SAFE_MODE | Enter safe mode | Critical fault |
| RESET | System reset | Unrecoverable |

### 5.2 Recovery Procedures

#### 5.2.1 EPS Recovery

```c
/**
 * EPS fault recovery procedures
 */
const RecoveryProcedure_t EPS_RECOVERY[] = {
    // Overcurrent recovery
    {FAULT_EPS_OVERCURRENT, {
        {ACTION_DISABLE_RAIL, 0, 100},      // Disable immediately
        {ACTION_WAIT, 0, 1000},             // Wait 1 second
        {ACTION_ENABLE_RAIL, 0, 100},       // Try re-enable
        {ACTION_CHECK_CURRENT, 0, 500},     // Verify current OK
        {ACTION_SAFE_MODE_IF_FAIL, 0, 0}    // Safe mode if still bad
    }},

    // Low battery recovery
    {FAULT_EPS_LOW_BATTERY, {
        {ACTION_SHED_LOADS, LOAD_PAYLOAD, 100},
        {ACTION_REDUCE_BEACON, 120, 100},
        {ACTION_SAFE_MODE, 0, 0}
    }},
};
```

#### 5.2.2 ADCS Recovery

```c
/**
 * ADCS fault recovery procedures
 */
const RecoveryProcedure_t ADCS_RECOVERY[] = {
    // High rate recovery
    {FAULT_ADCS_HIGH_RATE, {
        {ACTION_SET_MODE, ADCS_DETUMBLE, 100},
        {ACTION_WAIT, 0, 300000},           // Wait 5 minutes
        {ACTION_CHECK_RATE, 5, 1000},       // Check rate < 5°/s
        {ACTION_RESUME_NORMAL, 0, 0}
    }},

    // Sensor failure recovery
    {FAULT_ADCS_SENSOR, {
        {ACTION_MARK_INVALID, 0, 100},
        {ACTION_USE_BACKUP, 0, 100},
        {ACTION_LOG_FAULT, 0, 0}
    }},
};
```

### 5.3 Safe Mode Definition

```c
/**
 * Safe mode configuration
 * Minimal power, no payload, beacon only
 */
typedef struct {
    PowerMode_t power_mode;      // POWER_MODE_SAFE
    AdcsMode_t adcs_mode;        // ADCS_DETUMBLE
    bool payload_enabled;        // false
    bool tx_enabled;             // true (reduced power)
    uint16_t beacon_interval_s;  // 120 seconds
    bool autonomy_enabled;       // Limited
} SafeModeConfig_t;

static const SafeModeConfig_t SAFE_MODE = {
    .power_mode = POWER_MODE_SAFE,
    .adcs_mode = ADCS_DETUMBLE,
    .payload_enabled = false,
    .tx_enabled = true,
    .beacon_interval_s = 120,
    .autonomy_enabled = true  // For basic survival
};
```

## 6. Fault Counters and Thresholds

### 6.1 Fault Persistence

| Fault Type | Persistence | Threshold | Action |
|------------|-------------|-----------|--------|
| Watchdog reset | Cumulative | 3 | Safe mode |
| Sensor timeout | Window (1 hr) | 10 | Mark invalid |
| Overcurrent | Window (1 min) | 3 | Disable rail |
| Comm timeout | Cumulative | 48 hours | Beacon only |
| Memory error | Window (1 hr) | 5 | Reset |

### 6.2 Counter Management

```c
/**
 * Fault counter with window and threshold
 */
typedef struct {
    FaultId_t fault_id;
    uint16_t count;
    uint16_t threshold;
    uint32_t window_ms;
    uint32_t first_occurrence;
    uint32_t last_occurrence;
} FaultCounter_t;

/**
 * @brief Increment fault counter and check threshold
 * @param counter Fault counter
 * @return true if threshold exceeded
 */
bool increment_fault_counter(FaultCounter_t *counter);

/**
 * @brief Decay fault counters over time
 * Called periodically to reduce counts
 */
void decay_fault_counters(void);
```

## 7. Fault Logging

### 7.1 Fault Log Structure

```c
/**
 * Fault log entry
 */
typedef struct __attribute__((packed)) {
    uint32_t timestamp;          // System time
    uint8_t fault_id;            // Fault identifier
    uint8_t severity;            // Fault severity
    uint8_t isolation_result;    // Isolation status
    uint8_t recovery_result;     // Recovery status
    uint32_t value;              // Fault parameter value
    uint32_t context[2];         // Additional context
} FaultLogEntry_t;

#define FAULT_LOG_SIZE 256  // Number of entries
```

### 7.2 Fault Telemetry

| Item | Description | Downlink |
|------|-------------|----------|
| Active faults | Current fault bitmap | Every beacon |
| Fault count | Total faults since boot | Housekeeping |
| Last fault | Most recent fault | Housekeeping |
| Fault log | Complete history | On request |

## 8. Watchdog System

### 8.1 Watchdog Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   WATCHDOG HIERARCHY                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────┐    ┌────────────┐    ┌────────────┐        │
│  │  Task 1    │    │  Task 2    │    │  Task N    │        │
│  │ Heartbeat  │    │ Heartbeat  │    │ Heartbeat  │        │
│  └─────┬──────┘    └─────┬──────┘    └─────┬──────┘        │
│        │                 │                 │                │
│        └────────────┬────┴─────────────────┘                │
│                     ▼                                        │
│              ┌─────────────┐                                │
│              │  Software   │  Monitors all task heartbeats  │
│              │  Watchdog   │  Timeout: 10 seconds           │
│              └──────┬──────┘                                │
│                     │                                        │
│                     ▼                                        │
│              ┌─────────────┐                                │
│              │  Hardware   │  Final safety net              │
│              │  Watchdog   │  Timeout: 30 seconds           │
│              └─────────────┘                                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 8.2 Watchdog Implementation

```c
/**
 * Task heartbeat registration
 */
typedef struct {
    TaskId_t task_id;
    uint32_t last_heartbeat;
    uint32_t timeout_ms;
    bool active;
} TaskHeartbeat_t;

/**
 * @brief Register task heartbeat
 */
void heartbeat_beat(TaskId_t task);

/**
 * @brief Check all task heartbeats
 * @return Bitmap of failed tasks
 */
uint32_t check_heartbeats(void);

/**
 * @brief Kick hardware watchdog
 * Only if all software checks pass
 */
void watchdog_kick(void);
```

## 9. Ground Override

### 9.1 Override Commands

| Command | Effect |
|---------|--------|
| DISABLE_FDIR | Disable all autonomous FDIR |
| ENABLE_FDIR | Re-enable FDIR |
| CLEAR_FAULT | Clear specific fault |
| FORCE_SAFE | Enter safe mode |
| FORCE_NOMINAL | Exit safe mode |
| SET_THRESHOLD | Modify fault threshold |

### 9.2 Override Safety

```c
/**
 * Ground override requires confirmation
 */
typedef struct {
    uint32_t confirmation_code;   // Magic number
    uint32_t timeout_s;           // Override duration
    uint8_t override_mask;        // What to override
} OverrideRequest_t;

// Override auto-expires after timeout
```

## 10. Testing

### 10.1 FDIR Test Matrix

| Test | Injection | Expected Response |
|------|-----------|-------------------|
| Sensor timeout | Disable sensor | Mark invalid, use backup |
| Overcurrent | Force high current | Disable rail, safe mode |
| Watchdog | Block task | System reset |
| Low battery | Force low SOC | Load shed, safe mode |
| High temp | Force high reading | Duty cycle reduction |
| Memory error | Inject bit flip | Detection, correction |

### 10.2 Coverage Requirements

| FDIR Aspect | Coverage Target |
|-------------|-----------------|
| Fault detection | 100% of monitored parameters |
| Isolation | 100% of isolatable components |
| Recovery | 100% of recovery procedures |
| Safe mode entry | All trigger conditions |
| Watchdog | All monitored tasks |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
