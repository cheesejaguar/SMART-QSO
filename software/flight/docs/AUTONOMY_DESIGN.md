# Autonomy Design
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-AUTO-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the autonomous operation capabilities of the SMART-QSO CubeSat, enabling extended periods of operation without ground contact while maintaining mission safety.

## 2. Autonomy Philosophy

### 2.1 Design Principles

| Principle | Implementation |
|-----------|----------------|
| Fail-safe defaults | Conservative modes on anomaly |
| Graceful degradation | Reduce capability, maintain safety |
| Predictable behavior | Deterministic state machines |
| Ground override | All autonomous actions overridable |
| Transparency | All decisions logged for analysis |

### 2.2 Autonomy Levels

| Level | Name | Description |
|-------|------|-------------|
| 0 | Manual | All actions require ground command |
| 1 | Sequenced | Execute pre-loaded command sequences |
| 2 | Rule-based | Autonomous response to defined conditions |
| 3 | Adaptive | AI-assisted decision making |

SMART-QSO operates primarily at Level 2 with Level 3 for beacon generation.

## 3. Autonomous Functions

### 3.1 Power Management Autonomy

```c
/**
 * Autonomous power mode transitions
 * Based on battery state and solar input
 */
typedef struct {
    PowerMode_t current_mode;
    double battery_soc;
    double solar_input_w;
    double load_w;
    uint32_t time_in_mode_s;
} PowerState_t;

/**
 * Power mode transition rules (autonomous)
 */
static const PowerRule_t POWER_RULES[] = {
    // SOC < 20%: Enter SAFE mode
    {.condition = "soc < 20", .action = POWER_MODE_SAFE},
    // SOC < 30% and in ACTIVE: Drop to IDLE
    {.condition = "soc < 30 && mode == ACTIVE", .action = POWER_MODE_IDLE},
    // SOC > 80% and in IDLE for 5 min: Allow ACTIVE
    {.condition = "soc > 80 && mode == IDLE && time > 300", .action = POWER_MODE_ACTIVE},
    // Eclipse and SOC < 50%: Enter IDLE
    {.condition = "eclipse && soc < 50", .action = POWER_MODE_IDLE},
};
```

### 3.2 ADCS Autonomy

| Function | Trigger | Action |
|----------|---------|--------|
| Detumble | Rate > 5°/s | Activate B-dot control |
| Sun-pointing | Detumble complete | Point +Z to sun |
| Eclipse handling | Sun sensors dark | Maintain last attitude |
| Rate limit | Rate > 10°/s | Emergency detumble |

### 3.3 Communication Autonomy

```c
/**
 * Autonomous beacon scheduling
 */
typedef struct {
    uint32_t nominal_interval_s;    // 60 seconds typical
    uint32_t low_power_interval_s;  // 120 seconds in safe mode
    uint32_t pass_interval_s;       // 30 seconds during ground pass
    bool ai_generation_enabled;     // Use AI for content
} BeaconConfig_t;

/**
 * Beacon interval adjustment rules
 */
void update_beacon_interval(void) {
    if (power_mode == POWER_MODE_SAFE) {
        beacon_interval = LOW_POWER_INTERVAL;
    } else if (is_ground_pass_predicted()) {
        beacon_interval = PASS_INTERVAL;
    } else {
        beacon_interval = NOMINAL_INTERVAL;
    }
}
```

### 3.4 Fault Response Autonomy

| Fault | Autonomous Response | Escalation |
|-------|---------------------|------------|
| Sensor timeout | Use backup/stale data | Log, continue |
| EPS overcurrent | Disable affected load | Enter SAFE if repeated |
| Temperature limit | Reduce duty cycle | Safe mode if critical |
| Comm loss > 24h | Beacon-only mode | Reset after 72h |
| Watchdog reset | Log and continue | Safe mode after 3x |
| Memory corruption | Use backup data | Safe mode if critical |

## 4. State Machine Design

### 4.1 Mission State Machine

```
                        ┌─────────────────┐
                        │    LAUNCH       │
                        │   (Inhibited)   │
                        └────────┬────────┘
                                 │ T+30min
                                 ▼
                        ┌─────────────────┐
                        │    LEOP         │
                        │  (Deployment)   │
                        └────────┬────────┘
                                 │ Deployed & Detumbled
                                 ▼
        ┌───────────────────────────────────────────┐
        │                                           │
        ▼                                           ▼
┌───────────────┐                         ┌─────────────────┐
│    NOMINAL    │◄────────────────────────│   COMMISSIONING │
│  (Operations) │     Pass Complete       │                 │
└───────┬───────┘                         └─────────────────┘
        │
        │ Fault/Low Power
        ▼
┌───────────────┐
│     SAFE      │
│    (Minimal)  │
└───────┬───────┘
        │ Recovery
        ▼
┌───────────────┐
│   RECOVERY    │
│  (Diagnostic) │
└───────────────┘
```

### 4.2 State Definitions

| State | Power Mode | ADCS Mode | RF Mode | AI Payload |
|-------|------------|-----------|---------|------------|
| LAUNCH | OFF | OFF | OFF | OFF |
| LEOP | SAFE | Detumble | Beacon only | OFF |
| COMMISSIONING | IDLE | Sun-point | Full | Test only |
| NOMINAL | ACTIVE | Sun-point | Full | Enabled |
| SAFE | SAFE | Detumble | Low power beacon | OFF |
| RECOVERY | IDLE | Sun-point | Full | Disabled |

### 4.3 State Transition Rules

```c
/**
 * State transition validation
 * All transitions must pass safety checks
 */
typedef struct {
    MissionState_t from_state;
    MissionState_t to_state;
    bool (*guard_function)(void);
    void (*action_function)(void);
} StateTransition_t;

static const StateTransition_t TRANSITIONS[] = {
    {MISSION_LEOP, MISSION_COMMISSIONING,
     guard_deployment_complete, action_start_commissioning},

    {MISSION_COMMISSIONING, MISSION_NOMINAL,
     guard_commissioning_complete, action_start_nominal},

    {MISSION_NOMINAL, MISSION_SAFE,
     guard_any_critical_fault, action_enter_safe_mode},

    {MISSION_SAFE, MISSION_RECOVERY,
     guard_safe_stable, action_start_recovery},

    {MISSION_RECOVERY, MISSION_NOMINAL,
     guard_recovery_complete, action_resume_nominal},
};
```

## 5. AI Payload Autonomy

### 5.1 Jetson Power Management

```c
/**
 * Autonomous Jetson operation based on power budget
 */
typedef struct {
    bool power_available;      // Sufficient SOC and solar
    bool thermal_ok;           // Temperature within limits
    bool cooldown_complete;    // Minimum off-time elapsed
    uint32_t last_inference_s; // Time since last use
} JetsonState_t;

/**
 * @brief Determine if Jetson can be activated
 */
bool can_activate_jetson(void) {
    JetsonState_t state = get_jetson_state();

    return state.power_available &&
           state.thermal_ok &&
           state.cooldown_complete &&
           (get_power_mode() == POWER_MODE_ACTIVE);
}
```

### 5.2 AI Fallback Logic

| Condition | Action |
|-----------|--------|
| Jetson available | Generate AI beacon content |
| Jetson unavailable | Use template library |
| Jetson timeout | Kill process, use template |
| Jetson error | Log, disable, use template |
| AI output invalid | Filter, use template if needed |

### 5.3 Content Filtering

```c
/**
 * Autonomous content validation for AI-generated beacons
 */
typedef struct {
    bool length_valid;      // Within AX.25 limits
    bool encoding_valid;    // ASCII-only
    bool content_clean;     // No prohibited content
    bool callsign_present;  // Regulatory requirement
} ContentValidation_t;

/**
 * @brief Validate AI-generated beacon content
 * @param content Generated text
 * @param len Length of content
 * @return Validation result
 */
ContentValidation_t validate_ai_content(
    const char *content,
    size_t len
);
```

## 6. Scheduling Autonomy

### 6.1 Activity Scheduling

```c
/**
 * Autonomous activity scheduling
 * Based on orbital position and power state
 */
typedef struct {
    ActivityType_t type;
    uint32_t start_time;
    uint32_t duration;
    uint32_t priority;
    PowerMode_t required_mode;
} ScheduledActivity_t;

/**
 * Activity priority (lower = higher priority)
 */
typedef enum {
    PRIORITY_SAFETY = 0,        // Fault response
    PRIORITY_POWER = 1,         // Power management
    PRIORITY_ATTITUDE = 2,      // ADCS control
    PRIORITY_BEACON = 3,        // Beacon transmission
    PRIORITY_AI = 4,            // AI payload
    PRIORITY_HOUSEKEEPING = 5,  // Data management
} ActivityPriority_t;
```

### 6.2 Conflict Resolution

| Conflict | Resolution Rule |
|----------|-----------------|
| Power insufficient | Defer lower priority |
| Time overlap | Run higher priority first |
| Resource conflict | Queue by priority |
| Safety conflict | Safety always wins |

## 7. Ground Override

### 7.1 Override Commands

| Command | Effect | Persistence |
|---------|--------|-------------|
| DISABLE_AUTONOMY | Stop autonomous transitions | Until enabled |
| FORCE_MODE | Override current mode | Until cleared |
| SET_PARAMETER | Modify autonomous thresholds | Persistent |
| RESET_STATE | Return to known state | Immediate |

### 7.2 Override Priority

```c
/**
 * Command source priority for conflict resolution
 */
typedef enum {
    CMD_SOURCE_GROUND = 0,      // Highest priority
    CMD_SOURCE_STORED = 1,      // Time-tagged commands
    CMD_SOURCE_AUTONOMOUS = 2,  // Autonomous decisions
    CMD_SOURCE_DEFAULT = 3,     // Default behavior
} CommandSource_t;
```

## 8. Logging and Transparency

### 8.1 Decision Logging

```c
/**
 * All autonomous decisions are logged
 */
typedef struct {
    uint32_t timestamp;
    uint8_t decision_type;
    uint8_t trigger_id;
    uint8_t old_state;
    uint8_t new_state;
    uint32_t supporting_data[4];
} DecisionLog_t;

/**
 * @brief Log autonomous decision
 * @param type Type of decision
 * @param trigger What triggered the decision
 * @param old_state Previous state
 * @param new_state New state
 */
void log_autonomous_decision(
    DecisionType_t type,
    TriggerId_t trigger,
    uint8_t old_state,
    uint8_t new_state
);
```

### 8.2 Telemetry Transparency

| Item | Update Rate | Purpose |
|------|-------------|---------|
| Current state | 1 Hz | Ground awareness |
| Last decision | On change | Debugging |
| Pending actions | 10 s | Predictability |
| Override status | On change | Command confirmation |

## 9. Safety Constraints

### 9.1 Hard Limits (Not Overridable)

| Parameter | Limit | Action |
|-----------|-------|--------|
| Battery SOC | < 10% | Emergency shutdown |
| Temperature | > 80°C | Immediate power reduction |
| Reset count | > 10/hour | Enter recovery mode |
| TX duty cycle | > 50% | Enforce cooldown |

### 9.2 Soft Limits (Adjustable)

| Parameter | Default | Range | Command |
|-----------|---------|-------|---------|
| Safe mode SOC | 20% | 15-30% | SET_SAFE_SOC |
| Beacon interval | 60s | 30-120s | SET_BEACON_INT |
| AI duty cycle | 20% | 0-50% | SET_AI_DUTY |
| Recovery timeout | 24h | 12-72h | SET_RECOVERY_TIME |

## 10. Verification

### 10.1 Autonomy Testing

| Test | Method | Pass Criteria |
|------|--------|---------------|
| State transitions | Simulation | All transitions correct |
| Fault response | Injection | Safe mode entered |
| Power autonomy | Day-in-life | SOC maintained |
| Override | Command test | Immediate response |
| Logging | Review | All decisions logged |

### 10.2 Coverage Requirements

| Aspect | Coverage Target |
|--------|-----------------|
| State transitions | 100% |
| Fault responses | 100% |
| Decision rules | 100% |
| Override commands | 100% |
| Edge cases | 90% |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
