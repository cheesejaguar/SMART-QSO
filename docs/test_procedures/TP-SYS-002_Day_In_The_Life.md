# Test Procedure: Day-In-The-Life Test
## TP-SYS-002

**Document Number:** TP-SYS-002
**Revision:** A
**Date:** 2026-01-02

---

## 1. Purpose

Verify SMART-QSO CubeSat performance over a complete orbital day simulation (24 hours) including all operational modes, power cycles, thermal transitions, and automated operations.

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Solar simulator | 1 sun AM0 equivalent |
| Thermal shroud | Eclipse simulation |
| RF receiver | 145.825 MHz |
| Spectrum analyzer | 100-500 MHz |
| Power analyzer | Continuous logging |
| Time sync | GPS reference |
| Ground station | Full command capability |

## 3. Test Configuration

### 3.1 Orbit Parameters (Simulated)

| Parameter | Value |
|-----------|-------|
| Altitude | 500 km |
| Inclination | 51.6° (ISS) |
| Orbital period | 94.5 minutes |
| Eclipse duration | ~35 minutes |
| Sunlit duration | ~59 minutes |
| Orbits per day | 15.2 |

### 3.2 Mission Phases to Exercise

| Phase | Duration | Activities |
|-------|----------|------------|
| LEOP simulation | 2 orbits | Boot, detumble, first beacon |
| Normal ops | 10 orbits | Beacons, ADCS, AI payload |
| Ground pass | 4 passes | Commands, telemetry download |
| Anomaly response | 1 orbit | Fault injection, recovery |
| Low power mode | 1 orbit | Eclipse operations |

## 4. Procedure

### 4.1 Test Initialization

| Step | Action | Expected Result | Time | P/F |
|------|--------|-----------------|------|-----|
| 1 | Power on spacecraft | Boot sequence | T+0:00 | |
| 2 | Verify boot complete | < 30 seconds | T+0:00:30 | |
| 3 | Verify beacon transmission | First beacon | T+0:01:00 | |
| 4 | Start data logging | All channels | T+0:01:00 | |
| 5 | Sync test timeline | GPS time | T+0:02:00 | |

### 4.2 LEOP Simulation (Orbits 1-2)

#### Orbit 1: Deployment and Detumble

| Event | Time | Action | Expected | Actual | P/F |
|-------|------|--------|----------|--------|-----|
| Deploy | T+0:00 | Separation | 30-min inhibit start | ______ | |
| Antenna | T+0:30 | Deploy antenna | Successful deploy | ______ | |
| Detumble | T+0:31 | Start B-dot | < 0.5°/s in 45 min | ______ | |
| First beacon | T+0:32 | Transmit | 145.825 MHz received | ______ | |
| Eclipse entry | T+0:59 | Sun lost | Safe mode | ______ | |
| Eclipse exit | T+1:34 | Sun acquired | Resume ops | ______ | |

#### Orbit 2: Initial Checkout

| Event | Time | Action | Expected | Actual | P/F |
|-------|------|--------|----------|--------|-----|
| Sun pointing | T+1:35 | ADCS mode | Pointing < 10° | ______ | |
| Telemetry | T+1:40 | Full TLM | All sensors nominal | ______ | |
| Power check | T+1:45 | SOC verify | Battery > 50% | ______ | |
| Health check | T+1:50 | System status | No faults | ______ | |

### 4.3 Normal Operations (Orbits 3-12)

#### Beacon Generation Log

| Orbit | Beacons | AI Generated | Template | Errors |
|-------|---------|--------------|----------|--------|
| 3 | ______ | ______ | ______ | ______ |
| 4 | ______ | ______ | ______ | ______ |
| 5 | ______ | ______ | ______ | ______ |
| 6 | ______ | ______ | ______ | ______ |
| 7 | ______ | ______ | ______ | ______ |
| 8 | ______ | ______ | ______ | ______ |
| 9 | ______ | ______ | ______ | ______ |
| 10 | ______ | ______ | ______ | ______ |
| 11 | ______ | ______ | ______ | ______ |
| 12 | ______ | ______ | ______ | ______ |

#### Power Profile

| Orbit | Peak Power (W) | Avg Power (W) | SOC Start (%) | SOC End (%) |
|-------|---------------|---------------|---------------|-------------|
| 3 | ______ | ______ | ______ | ______ |
| 4 | ______ | ______ | ______ | ______ |
| 5 | ______ | ______ | ______ | ______ |
| 6 | ______ | ______ | ______ | ______ |
| 7 | ______ | ______ | ______ | ______ |
| 8 | ______ | ______ | ______ | ______ |
| 9 | ______ | ______ | ______ | ______ |
| 10 | ______ | ______ | ______ | ______ |
| 11 | ______ | ______ | ______ | ______ |
| 12 | ______ | ______ | ______ | ______ |

#### ADCS Performance

| Orbit | Detumble Rate (°/s) | Pointing Error (°) | Mode |
|-------|---------------------|-------------------|------|
| 3 | ______ | ______ | ______ |
| 6 | ______ | ______ | ______ |
| 9 | ______ | ______ | ______ |
| 12 | ______ | ______ | ______ |

### 4.4 Ground Pass Simulation

#### Pass 1 (Orbit 4)

| Event | Duration | Activity | Result | P/F |
|-------|----------|----------|--------|-----|
| AOS | T+5:00 | Lock signal | ______ | |
| Command | T+5:02 | Send health query | ______ | |
| Response | T+5:03 | Receive telemetry | ______ | |
| LOS | T+5:10 | Pass complete | ______ | |

#### Pass 2 (Orbit 8)

| Event | Duration | Activity | Result | P/F |
|-------|----------|----------|--------|-----|
| AOS | T+12:00 | Lock signal | ______ | |
| Command | T+12:02 | Change beacon rate | ______ | |
| Verify | T+12:04 | Confirm change | ______ | |
| LOS | T+12:10 | Pass complete | ______ | |

#### Pass 3 (Orbit 11)

| Event | Duration | Activity | Result | P/F |
|-------|----------|----------|--------|-----|
| AOS | T+17:00 | Lock signal | ______ | |
| Download | T+17:02 | Get stored data | ______ | |
| Verify | T+17:08 | Data integrity | ______ | |
| LOS | T+17:10 | Pass complete | ______ | |

### 4.5 Anomaly Response (Orbit 13)

| Test | Injection | Expected Response | Actual | Recovery | P/F |
|------|-----------|-------------------|--------|----------|-----|
| Sensor fail | Disable magnetometer | Use backup/degrade | ______ | ______ | |
| Low battery | Force SOC < 20% | Enter safe mode | ______ | ______ | |
| Comm timeout | Block uplink | Continue autonomous | ______ | ______ | |
| Watchdog | Delay kick | System reset | ______ | ______ | |
| Memory error | Inject bit flip | CRC detect, recover | ______ | ______ | |

### 4.6 Low Power Operations (Orbit 14)

| Event | SOC | Mode | Load (mW) | Result | P/F |
|-------|-----|------|-----------|--------|-----|
| Enter eclipse | ______ | ______ | ______ | ______ | |
| Mid-eclipse | ______ | ______ | ______ | ______ | |
| SOC < 30% | ______ | Safe mode | ______ | ______ | |
| Exit eclipse | ______ | ______ | ______ | ______ | |
| Recovery | ______ | Normal | ______ | ______ | |

### 4.7 AI Payload Operations

| Orbit | Jetson On | Generation Time (s) | Power (W) | Temperature (°C) | Output |
|-------|-----------|--------------------:|-----------|------------------|--------|
| 5 | ______ | ______ | ______ | ______ | ______ |
| 7 | ______ | ______ | ______ | ______ | ______ |
| 9 | ______ | ______ | ______ | ______ | ______ |
| 11 | ______ | ______ | ______ | ______ | ______ |

## 5. Pass/Fail Criteria

| Criterion | Requirement | Result | P/F |
|-----------|-------------|--------|-----|
| Test duration | 24 hours minimum | ______ | |
| Beacon success rate | > 95% | ______ | |
| ADCS pointing | < 10° error | ______ | |
| Battery SOC | Never < 10% | ______ | |
| Command success | > 99% | ______ | |
| Autonomous recovery | All faults recovered | ______ | |
| AI payload ops | > 80% availability | ______ | |
| No unrecoverable faults | Zero | ______ | |

## 6. Data Products

| Product | Description | Location |
|---------|-------------|----------|
| Power log | Continuous power data | ______ |
| Telemetry log | All telemetry frames | ______ |
| Beacon recordings | All transmitted beacons | ______ |
| Temperature log | All thermal channels | ______ |
| Event log | All mode changes/faults | ______ |
| Command log | All commands sent | ______ |

## 7. Test Summary

| Metric | Value |
|--------|-------|
| Test start time | ______ |
| Test end time | ______ |
| Total duration | ______ hrs |
| Orbits completed | ______ |
| Total beacons | ______ |
| Commands sent | ______ |
| Faults detected | ______ |
| Faults recovered | ______ |
| Overall result | ______ |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
