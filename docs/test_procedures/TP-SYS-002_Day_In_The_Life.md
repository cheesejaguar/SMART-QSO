# Test Procedure: Day-In-The-Life Test
## TP-SYS-002

**Document Number:** TP-SYS-002
**Revision:** D
**Date:** 2026-01-02

**Requirements verified:** MRD-002, SYS-111, SYS-200, SYS-215, SYS-220, SYS-221, SYS-224, SYS-300, SYS-303, SYS-325, SYS-421, SYS-422, SYS-512, SRS-F013, SRS-F052, SRS-Q002

---

## 1. Purpose

Verify SMART-QSO CubeSat performance over a complete orbital day simulation (24 hours) including all operational modes, power cycles, thermal transitions, and automated operations.

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Solar simulator | 1 sun AM0 equivalent |
| Thermal shroud | Eclipse simulation |
| RF receiver | The IARU-coordinated downlink frequency (TBD until coordination, TBD-SYS-001; SYS-200) |
| Spectrum analyzer | 100-500 MHz |
| Power analyzer | Continuous logging |
| Time sync | GPS reference |
| Ground station | Full command capability |

## 3. Test Configuration

### 3.1 Orbit Parameters (Simulated)

| Parameter | Value |
|-----------|-------|
| Altitude | 475 km (target; 465–485 km, 485 km cap, `docs/BASELINE_PARAMETERS.md` §3) |
| Inclination | ≈ 97.31° at 475 km (circular SSO, `docs/BASELINE_PARAMETERS.md` §3) |
| Orbital period | 94.1 minutes at 475 km |
| Eclipse duration | ~36 minutes (β = 0, the cold case of `docs/BASELINE_PARAMETERS.md` §8) |
| Sunlit duration | ~59 minutes |
| Orbits per day | 15.3 (SYS-012) |

### 3.2 Mission Phases to Exercise

| Phase | Duration | Activities |
|-------|----------|------------|
| LEOP simulation | 2 orbits | Boot, detumble, first beacon |
| Normal ops | 10 orbits | Beacons, ADCS, AI payload |
| Ground pass | 4 passes | Commands, telemetry download |
| Anomaly response | 1 orbit | Fault injection, recovery |
| Low power mode | 1 orbit | Eclipse operations |

## 4. Procedure

### 4.1 Test Initialization [SYS-224, SYS-300]

| Step | Action | Expected Result | Time | P/F |
|------|--------|-----------------|------|-----|
| 1 | Power on spacecraft | Boot sequence | T+0:00 | |
| 2 | Verify boot complete | < 30 seconds | T+0:00:30 | |
| 3 | Verify beacon transmission | First beacon, only if the RF silence was already completed on the bench (`software/flight/include/rf_silence.h`); otherwise none before T+0:45 (SYS-224) | T+0:01:00 | |
| 4 | Start data logging | All channels | T+0:01:00 | |
| 5 | Sync test timeline | GPS time | T+0:02:00 | |

### 4.2 LEOP Simulation (Orbits 1-2)

#### Orbit 1: Deployment and Detumble [SYS-200, SYS-220, SYS-221, SYS-224, SYS-421]

| Event | Time | Action | Expected | Actual | P/F |
|-------|------|--------|----------|--------|-----|
| Deploy | T+0:00 | Separation | 30-min inhibit start | ______ | |
| RF silence | T+0:00 to T+0:45 | Monitor spectrum | No RF emission (SYS-224) | ______ | |
| Detumble | T+0:02 | Start B-dot, no actuation before 120 s (SRS-F086) | Detumbled from 10°/s within 3 orbits (SYS-420) | ______ | |
| Wings | T+0:30 | Deploy both wings, one resistor at a time (10 s, C-17) | Successful deploy | ______ | |
| Antenna | After the wings | AntS deployment, temperature gate ≥ −17 °C (C-18) | Successful deploy; transmitters silent until then (C-19) | ______ | |
| First beacon | ≥ T+0:45 | Transmit | Received at the IARU-coordinated frequency (TBD-SYS-001; SYS-200), not before T+0:45 (SYS-224) | ______ | |
| Eclipse entry | T+0:59 | Sun lost | Mode by SOC (BASELINE §5); transponder and Jetson off (sunlit only) | ______ | |
| Eclipse exit | T+1:34 | Sun acquired | Resume ops | ______ | |

#### Orbit 2: Initial Checkout [SYS-303, SYS-422]

| Event | Time | Action | Expected | Actual | P/F |
|-------|------|--------|----------|--------|-----|
| Sun pointing | T+1:35 | ADCS mode | Spin-stabilized sun-pointing of +Y at 8°/s (SYS-422) | ______ | |
| Telemetry | T+1:40 | Full TLM | All sensors nominal | ______ | |
| Power check | T+1:45 | SOC verify | Battery > 50% | ______ | |
| Health check | T+1:50 | System status | No faults | ______ | |

### 4.3 Normal Operations (Orbits 3-12)

#### Beacon Generation Log [SYS-215]

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

### 4.4 Ground Pass Simulation [SYS-325]

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

### 4.5 Anomaly Response (Orbit 13) [MRD-002, SRS-F013, SRS-F052, SRS-Q002]

| Test | Injection | Expected Response | Actual | Recovery | P/F |
|------|-----------|-------------------|--------|----------|-----|
| Sensor fail | Disable magnetometer | Use backup/degrade | ______ | ______ | |
| Low battery | Force SOC < 25% | Enter safe mode | ______ | ______ | |
| Comm timeout | Block uplink | Continue autonomous | ______ | ______ | |
| Watchdog | Delay kick | System reset | ______ | ______ | |
| Memory error | Inject bit flip | CRC detect, recover | ______ | ______ | |

### 4.6 Low Power Operations (Orbit 14) [SYS-111, SRS-F013]

| Event | SOC | Mode | Load (mW) | Result | P/F |
|-------|-----|------|-----------|--------|-----|
| Enter eclipse | ______ | ______ | ______ | ______ | |
| Mid-eclipse | ______ | ______ | ______ | ______ | |
| SOC < 25% (SRS-F013) | ______ | Safe mode | ______ | ______ | |
| Exit eclipse | ______ | ______ | ______ | ______ | |
| Recovery | ______ | Normal | ______ | ______ | |

### 4.7 AI Payload Operations [SYS-512]

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
| ADCS pointing | Spin about +Y at 8°/s held; pointing performance judged by the Monte Carlo (decision C-23, ADCS-P002) | ______ | |
| Battery SOC | Never < 15 % (EMERGENCY threshold, BASELINE §5); depth of discharge ≤ 10 % per eclipse (SYS-112) | ______ | |
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
| B | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) | SMART-QSO Team |
| C | 2026-10-16 | Conflicting steps corrected and cited: first beacon not before T+0:45, with a no-emission check from separation (SYS-224; was T+0:32), received at the IARU-coordinated frequency (SYS-200, TBD-SYS-001; was 145.825 MHz); §4.1 step 3 conditional on a completed bench RF silence; SAFE at SOC < 25 % (SRS-F013; was 30 %); simulated orbit SSO ≈ 97.4° (BASELINE §3; was 51.6°, ISS) (finding VV-06) | SMART-QSO Team |
| D | 2026-10-27 | CDR 2026 consistency pass: simulated orbit at the 475 km baseline target, period 94.1 min (BASELINE Rev E §3) | CDR documentation review |
