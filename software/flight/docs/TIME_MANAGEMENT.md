# Time Management Design
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-TIME-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, CCSDS 301.0-B-4

---

## 1. Purpose

This document defines the time management architecture for the SMART-QSO CubeSat, including timekeeping, synchronization, and time-tagged operations.

## 2. Time System Overview

### 2.1 Time References

| Time Type | Description | Use |
|-----------|-------------|-----|
| Mission Elapsed Time (MET) | Seconds since boot | Internal scheduling |
| Spacecraft Time (SCT) | Unix epoch (UTC) | Telemetry timestamps |
| Ground Time | UTC from ground station | Time correlation |

### 2.2 Time Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   TIME SYSTEM ARCHITECTURE                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐                                           │
│  │   RTC (32kHz)│────┐                                      │
│  │   Hardware   │    │                                      │
│  └──────────────┘    │                                      │
│                      ▼                                      │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │  System Tick │  │  Epoch Time  │                        │
│  │   (1 kHz)    │  │   Counter    │                        │
│  └──────┬───────┘  └──────┬───────┘                        │
│         │                 │                                 │
│         ▼                 ▼                                 │
│  ┌────────────────────────────────────┐                    │
│  │         Time Manager               │                    │
│  │  • MET calculation                 │                    │
│  │  • UTC conversion                  │                    │
│  │  • Drift compensation              │                    │
│  └────────────────────────────────────┘                    │
│         │                                                   │
│         ▼                                                   │
│  ┌────────────────────────────────────┐                    │
│  │      Time Distribution             │                    │
│  │  • Telemetry timestamps            │                    │
│  │  • Event logging                   │                    │
│  │  • Scheduled operations            │                    │
│  └────────────────────────────────────┘                    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. Hardware Time Sources

### 3.1 Real-Time Clock (RTC)

| Parameter | Specification |
|-----------|---------------|
| Crystal frequency | 32.768 kHz |
| Accuracy | ±20 ppm (nominal) |
| Temperature stability | ±50 ppm (-20°C to +50°C) |
| Battery backup | Supercapacitor |
| Backup duration | > 1 hour |

### 3.2 System Timer

| Parameter | Specification |
|-----------|---------------|
| Clock source | MCU oscillator |
| Tick rate | 1 kHz (1 ms resolution) |
| Counter width | 32-bit |
| Overflow | ~49.7 days |

## 4. Time Data Structures

### 4.1 Time Types

```c
/**
 * Mission Elapsed Time (seconds since boot)
 */
typedef uint32_t met_t;

/**
 * Spacecraft Time (Unix epoch seconds)
 */
typedef uint32_t sct_t;

/**
 * High-resolution time (milliseconds)
 */
typedef uint64_t time_ms_t;

/**
 * Complete time structure
 */
typedef struct {
    sct_t epoch_sec;        // Seconds since Unix epoch
    uint16_t msec;          // Milliseconds (0-999)
    uint8_t valid;          // Time validity flag
    uint8_t source;         // Time source ID
} SpacecraftTime_t;
```

### 4.2 Time Validity

```c
/**
 * Time validity states
 */
typedef enum {
    TIME_INVALID = 0,       // No valid time
    TIME_ESTIMATED = 1,     // Estimated from MET
    TIME_COARSE = 2,        // Set by ground (±1s)
    TIME_FINE = 3           // Synchronized (±100ms)
} TimeValidity_t;
```

## 5. Time Functions

### 5.1 Core Time API

```c
/**
 * @brief Get current Mission Elapsed Time
 * @return Seconds since boot
 */
met_t time_get_met(void);

/**
 * @brief Get current Mission Elapsed Time in milliseconds
 * @return Milliseconds since boot
 */
time_ms_t time_get_met_ms(void);

/**
 * @brief Get current Spacecraft Time
 * @return Unix epoch seconds
 */
sct_t time_get_sct(void);

/**
 * @brief Get complete time structure
 * @param time Output time structure
 */
void time_get_current(SpacecraftTime_t *time);

/**
 * @brief Set Spacecraft Time from ground
 * @param epoch_sec Unix epoch seconds
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t time_set_sct(sct_t epoch_sec);

/**
 * @brief Get time validity
 * @return Current time validity state
 */
TimeValidity_t time_get_validity(void);
```

### 5.2 Time Conversion

```c
/**
 * @brief Convert MET to SCT
 * @param met Mission Elapsed Time
 * @return Spacecraft Time (0 if epoch not set)
 */
sct_t time_met_to_sct(met_t met);

/**
 * @brief Convert SCT to calendar time
 * @param sct Spacecraft Time
 * @param cal Output calendar structure
 */
void time_sct_to_calendar(sct_t sct, CalendarTime_t *cal);

/**
 * @brief Format time as ISO 8601 string
 * @param sct Spacecraft Time
 * @param buf Output buffer (min 20 chars)
 */
void time_format_iso8601(sct_t sct, char *buf);
```

## 6. Time Synchronization

### 6.1 Ground Time Update

```
Ground Station                    Spacecraft
      │                               │
      │ Send time at predicted       │
      │ one-way light time           │
      │                               │
      ├────── TIME_SET (T1) ────────►│
      │                               │ Receive at T2
      │                               │ SCT = T1 + propagation
      │                               │
      │◄───── TIME_ACK (T2) ─────────┤
      │                               │
      │ Calculate round-trip         │
      │ Verify sync accuracy         │
      │                               │
```

### 6.2 Synchronization Algorithm

```c
/**
 * @brief Process time update from ground
 * @param ground_time Time sent by ground
 * @param propagation_ms Estimated propagation delay
 */
void time_process_sync(sct_t ground_time, uint16_t propagation_ms) {
    // Apply propagation correction
    sct_t corrected_time = ground_time + (propagation_ms / 1000);

    // Get current time and calculate offset
    sct_t current_sct = time_get_sct();
    int32_t offset = (int32_t)(corrected_time - current_sct);

    // Log time correction
    log_event(EVENT_TIME_SYNC, offset);

    // Apply correction
    if (abs(offset) > TIME_STEP_THRESHOLD_SEC) {
        // Large correction: step time
        time_set_sct(corrected_time);
    } else {
        // Small correction: slew time
        time_start_slew(offset);
    }

    // Update validity
    time_set_validity(TIME_COARSE);
}
```

### 6.3 Time Slewing

```c
/**
 * Time slewing for small corrections
 * Adjusts tick rate to smoothly correct time
 */
#define SLEW_RATE_PPM 100  // Max 100 ppm adjustment
#define SLEW_THRESHOLD_MS 1000  // Slew if offset < 1s

/**
 * @brief Start time slew
 * @param offset_ms Offset to correct in milliseconds
 */
void time_start_slew(int32_t offset_ms);

/**
 * @brief Check if slew is in progress
 * @return true if currently slewing
 */
bool time_is_slewing(void);
```

## 7. Time-Tagged Operations

### 7.1 Scheduled Commands

```c
/**
 * Time-tagged command structure
 */
typedef struct {
    sct_t execute_time;     // When to execute
    uint8_t command_id;     // Command to execute
    uint8_t params[16];     // Command parameters
    uint8_t status;         // Pending/executed/failed
} ScheduledCommand_t;

#define MAX_SCHEDULED_COMMANDS 32
```

### 7.2 Scheduler Implementation

```c
/**
 * @brief Schedule command for future execution
 * @param exec_time Execution time (SCT)
 * @param cmd Command structure
 * @return Schedule slot or -1 on error
 */
int schedule_command(sct_t exec_time, const Command_t *cmd);

/**
 * @brief Process scheduled commands (called each second)
 */
void scheduler_tick(void) {
    sct_t now = time_get_sct();

    for (int i = 0; i < MAX_SCHEDULED_COMMANDS; i++) {
        ScheduledCommand_t *cmd = &scheduled_commands[i];

        if (cmd->status == SCHED_PENDING && cmd->execute_time <= now) {
            // Execute command
            execute_command(cmd);
            cmd->status = SCHED_EXECUTED;

            // Log execution
            log_event(EVENT_SCHED_EXEC, cmd->command_id);
        }
    }
}
```

## 8. Time in Telemetry

### 8.1 Timestamp Format

| Format | Size | Resolution | Range |
|--------|------|------------|-------|
| Unix epoch | 4 bytes | 1 second | 1970-2106 |
| Milliseconds | 2 bytes | 1 ms | 0-999 |
| MET | 4 bytes | 1 second | 136 years |

### 8.2 Beacon Timestamp

```c
/**
 * Beacon timestamp format (ISO 8601 subset)
 * Format: YYYY-MM-DDTHH:MM:SSZ
 * Example: 2026-01-15T14:32:00Z
 */
void format_beacon_timestamp(char *buf, size_t len) {
    SpacecraftTime_t now;
    time_get_current(&now);

    if (now.valid >= TIME_COARSE) {
        CalendarTime_t cal;
        time_sct_to_calendar(now.epoch_sec, &cal);
        snprintf(buf, len, "%04d-%02d-%02dT%02d:%02d:%02dZ",
                 cal.year, cal.month, cal.day,
                 cal.hour, cal.minute, cal.second);
    } else {
        snprintf(buf, len, "MET:%08lX", time_get_met());
    }
}
```

## 9. Time Fault Handling

### 9.1 RTC Failure

| Symptom | Detection | Response |
|---------|-----------|----------|
| RTC stopped | Tick comparison | Use system timer |
| RTC jumped | Rate check | Ignore, use previous |
| RTC drift | Ground comparison | Apply correction |

### 9.2 Time Validity Degradation

```c
/**
 * Time validity timeout
 * If no ground sync for extended period, degrade validity
 */
#define TIME_VALIDITY_TIMEOUT_S (7 * 24 * 3600)  // 1 week

void time_check_validity(void) {
    uint32_t last_sync = time_get_last_sync();
    uint32_t now = time_get_met();

    if ((now - last_sync) > TIME_VALIDITY_TIMEOUT_S) {
        // Degrade time validity
        if (time_get_validity() > TIME_ESTIMATED) {
            time_set_validity(TIME_ESTIMATED);
            log_event(EVENT_TIME_DEGRADED, now - last_sync);
        }
    }
}
```

## 10. Timing Requirements

### 10.1 Accuracy Requirements

| Application | Requirement | Achieved |
|-------------|-------------|----------|
| Telemetry timestamp | ±1 second | ±100 ms |
| Event correlation | ±100 ms | ±10 ms |
| Beacon timing | ±1 second | ±100 ms |
| Command scheduling | ±1 second | ±100 ms |

### 10.2 Drift Budget

| Source | Contribution |
|--------|--------------|
| RTC crystal | ±20 ppm |
| Temperature | ±30 ppm |
| Aging | ±5 ppm/year |
| **Total** | **±55 ppm** |

Drift rate: ~4.75 seconds/day maximum

## 11. Testing

### 11.1 Time Tests

| Test | Method | Pass Criteria |
|------|--------|---------------|
| RTC accuracy | Compare to reference | ±50 ppm |
| Time set | Ground command | ±100 ms |
| Time slew | Small correction | Smooth transition |
| Scheduler | Time-tagged cmd | ±1 second |
| Rollover | 32-bit overflow | Correct handling |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
