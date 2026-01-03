# Timing Analysis
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-007
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NASA-HDBK-2203

---

## 1. Purpose

This document provides timing analysis for SMART-QSO flight software, including worst-case execution time (WCET) estimates, task scheduling, and deadline verification.

## 2. Scope

This analysis covers:
- Task execution times
- Interrupt latencies
- Scheduling analysis
- Deadline verification
- Timing margins

---

## 3. System Timing Parameters

### 3.1 Processor Characteristics

| Parameter | Value |
|-----------|-------|
| Processor | STM32L4R5 (Cortex-M4) |
| Clock frequency | 80 MHz |
| Flash wait states | 4 (at 80 MHz) |
| Instruction cache | Enabled |
| Data cache | Enabled |

### 3.2 Timing Constraints

| Constraint | Value | Source |
|------------|-------|--------|
| Watchdog timeout | 30 s | System requirement |
| Beacon interval (min) | 10 s | Mission requirement |
| Sensor sample rate | 10 Hz | ADCS requirement |
| UART timeout | 1 s | Protocol spec |
| Control loop period | 1 s | ADCS requirement |

---

## 4. Task Timing Analysis

### 4.1 Task Summary

| Task | Period | Deadline | Priority | WCET |
|------|--------|----------|----------|------|
| Watchdog kick | 10 s | 10 s | Highest | 10 us |
| Sensor sampling | 100 ms | 100 ms | High | 2 ms |
| ADCS control | 1000 ms | 1000 ms | High | 15 ms |
| Beacon generation | Variable | Period | Normal | 50 ms |
| Telemetry collection | 5000 ms | 5000 ms | Low | 10 ms |
| Housekeeping | 10000 ms | 10000 ms | Low | 5 ms |
| Main loop | 100 ms | 100 ms | Normal | 20 ms |

### 4.2 WCET Breakdown

#### 4.2.1 Sensor Sampling Task (WCET: 2 ms)

| Function | WCET | Notes |
|----------|------|-------|
| I2C read magnetometer | 500 us | 400 kHz I2C |
| I2C read temperature | 200 us | Same bus |
| ADC read sun sensors | 600 us | 6 channels |
| Data filtering | 200 us | IIR filter |
| Data validation | 100 us | Range checks |
| Buffer update | 100 us | Copy to shared |
| Overhead | 300 us | Task switch, etc. |
| **Total** | **2000 us** | |

#### 4.2.2 ADCS Control Task (WCET: 15 ms)

| Function | WCET | Notes |
|----------|------|-------|
| Get sensor data | 100 us | From buffer |
| Attitude determination | 2 ms | TRIAD algorithm |
| B-dot calculation | 500 us | Magnetic derivative |
| Control law | 1 ms | PD controller |
| Magnetorquer command | 500 us | PWM update |
| Telemetry update | 500 us | Status logging |
| State machine | 200 us | Mode logic |
| Overhead | 200 us | |
| **Margin (3x)** | **10 ms** | Safety factor |
| **Total** | **15000 us** | |

#### 4.2.3 Beacon Generation Task (WCET: 50 ms)

| Function | WCET | Notes |
|----------|------|-------|
| Collect telemetry | 2 ms | All subsystems |
| Format JSON (if Jetson) | 5 ms | Health data |
| Wait for Jetson response | 30 ms | Timeout |
| Fallback text generation | 1 ms | Template |
| AX.25 frame construction | 5 ms | With CRC |
| UART transmit to RF | 5 ms | 1200 bps start |
| Overhead | 2 ms | |
| **Total** | **50000 us** | |

#### 4.2.4 Main Loop (WCET: 20 ms)

| Function | WCET | Notes |
|----------|------|-------|
| Check power state | 500 us | ADC reads |
| Mode management | 1 ms | State machine |
| Fault checking | 2 ms | All monitors |
| Command processing | 5 ms | If pending |
| Jetson interface | 5 ms | Health exchange |
| Logging | 2 ms | Event log |
| Idle processing | 4.5 ms | Scrubbing, etc. |
| **Total** | **20000 us** | |

---

## 5. Interrupt Timing

### 5.1 Interrupt Priorities

| Interrupt | Priority | Max Latency | WCET |
|-----------|----------|-------------|------|
| Watchdog warning | 0 (highest) | 1 us | 5 us |
| SysTick | 1 | 5 us | 10 us |
| UART RX | 2 | 10 us | 20 us |
| I2C event | 3 | 20 us | 30 us |
| ADC complete | 4 | 50 us | 20 us |
| Timer | 5 | 100 us | 15 us |

### 5.2 Interrupt Load

| Source | Frequency | WCET | Load |
|--------|-----------|------|------|
| SysTick | 1000 Hz | 10 us | 1.0% |
| UART RX | ~100 Hz | 20 us | 0.2% |
| I2C | ~50 Hz | 30 us | 0.15% |
| ADC | 10 Hz | 20 us | 0.02% |
| **Total** | | | **~1.4%** |

---

## 6. Scheduling Analysis

### 6.1 CPU Utilization

| Task | Period | WCET | Utilization |
|------|--------|------|-------------|
| Sensor sampling | 100 ms | 2 ms | 2.0% |
| ADCS control | 1000 ms | 15 ms | 1.5% |
| Main loop | 100 ms | 20 ms | 20.0% |
| Beacon (30s avg) | 30000 ms | 50 ms | 0.17% |
| Telemetry | 5000 ms | 10 ms | 0.2% |
| Housekeeping | 10000 ms | 5 ms | 0.05% |
| Interrupt load | - | - | 1.4% |
| **Total** | | | **~25.3%** |

### 6.2 Utilization Margin

| Metric | Value |
|--------|-------|
| Total utilization | 25.3% |
| Target maximum | 70% |
| **Margin** | **44.7%** |

### 6.3 Rate Monotonic Analysis

For periodic tasks with deadlines equal to periods:

| Task | Priority | Period | WCET | U |
|------|----------|--------|------|---|
| Sensor | 1 | 100 ms | 2 ms | 0.020 |
| Main | 2 | 100 ms | 20 ms | 0.200 |
| ADCS | 3 | 1000 ms | 15 ms | 0.015 |

Total U = 0.235 < U_lub(3) = 0.779

**All deadlines met under Rate Monotonic scheduling.**

---

## 7. Critical Path Analysis

### 7.1 Beacon Generation Path

```
Trigger (timer)
    │
    ▼ t=0
Collect telemetry ──────────── 2 ms
    │
    ▼ t=2 ms
Request Jetson text ─────────── 1 ms
    │
    ▼ t=3 ms
Wait for response ───────────── 30 ms (timeout)
    │
    ▼ t=33 ms
Format AX.25 frame ──────────── 5 ms
    │
    ▼ t=38 ms
Transmit to RF ──────────────── 5 ms
    │
    ▼ t=43 ms
Complete
```

**Critical path: 43 ms < 10 s deadline (OK)**

### 7.2 Fault Response Path

```
Fault detected
    │
    ▼ t=0
Log fault ───────────────────── 1 ms
    │
    ▼ t=1 ms
Evaluate severity ───────────── 100 us
    │
    ▼ t=1.1 ms
Execute response ────────────── 5 ms (safe mode)
    │
    ▼ t=6.1 ms
Confirm safe state ──────────── 2 ms
    │
    ▼ t=8.1 ms
Complete
```

**Fault response: 8.1 ms (well within 30s watchdog)**

---

## 8. Timing Margins

### 8.1 Margin Summary

| Path | WCET | Deadline | Margin |
|------|------|----------|--------|
| Sensor loop | 2 ms | 100 ms | 98% |
| ADCS control | 15 ms | 1000 ms | 98.5% |
| Beacon generation | 50 ms | 10000 ms | 99.5% |
| Fault response | 8 ms | 30000 ms | 99.97% |
| Main loop | 20 ms | 100 ms | 80% |

### 8.2 Margin Policy

| Condition | Required Margin |
|-----------|-----------------|
| Safety-critical path | >50% |
| Normal operation | >25% |
| Low priority | >10% |

**All paths exceed margin requirements.**

---

## 9. Timing Verification

### 9.1 Measurement Approach

| Method | Application |
|--------|-------------|
| GPIO toggle + scope | ISR timing |
| Cycle counter (DWT) | Function timing |
| Timestamp logging | Task timing |
| Profiling (gprof) | Development |

### 9.2 Test Cases

| Test | Purpose |
|------|---------|
| TT-001 | Sensor task timing |
| TT-002 | ADCS control timing |
| TT-003 | Beacon generation timing |
| TT-004 | Interrupt latency |
| TT-005 | Full load timing |

---

## 10. Recommendations

1. **Maintain 3x margin** on WCET estimates for flight
2. **Profile on target hardware** before flight release
3. **Add timing assertions** for critical paths
4. **Monitor CPU utilization** via telemetry
5. **Test under thermal extremes** for timing variation

---

## 11. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
