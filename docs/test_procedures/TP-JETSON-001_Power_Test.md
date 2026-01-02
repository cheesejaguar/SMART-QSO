# Test Procedure: Jetson Payload Power Test
## SMART-QSO CubeSat Mission

**Document Number:** TP-JETSON-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NVIDIA Jetson Orin Nano Datasheet, NASA Ames SmallSat Heritage

---

## 1. Purpose

This procedure verifies the power characteristics of the Jetson Orin Nano payload, including idle power, inference power, peak power, power gating, and thermal behavior.

## 2. Scope

- Idle power consumption measurement
- Inference power consumption during AI operations
- Peak power draw characterization
- Power gating (enable/disable) function verification
- Power sequencing verification
- DVFS (Dynamic Voltage Frequency Scaling) profile verification

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-HW-004 | Payload Design Document |
| SQSO-ICD-001 | ICD OBC-Jetson |
| NVIDIA | Jetson Orin Nano Datasheet |
| SQSO-REQ-001 | System Requirements |

## 4. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | Jetson Orin Nano Module |
| Carrier Board | SMART-QSO Carrier or Dev Kit |
| Configuration | Flight software loaded |
| Firmware Version | |
| JetPack Version | |

## 5. Test Equipment

| Equipment | Model | Cal Due |
|-----------|-------|---------|
| DC Power Supply | 5V/5A capable | |
| Current Shunt | 100mOhm, 0.1% | |
| DAQ/Oscilloscope | High-speed sampling | |
| Temperature Probe | Thermocouple or IR | |
| Stopwatch | | |
| USB-Serial Adapter | For OBC interface | |

## 6. Preconditions

- [ ] Jetson module installed on carrier
- [ ] Flight software image installed
- [ ] OBC interface connected and verified
- [ ] Power supply configured for 5V output
- [ ] Temperature monitoring in place
- [ ] Test scripts ready

## 7. Safety

- Jetson may become hot during sustained operation
- Monitor temperature continuously
- Maximum junction temperature: 105C
- Maximum ambient: 65C (with cooling)
- Abort if thermal limits approached
- Allow cool-down between tests

---

## 8. Procedure

### 8.1 Test Setup

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.1.1 | Configure power supply for 5.0V | Stable output | |
| 8.1.2 | Insert current shunt in power path | Shunt resistance verified | |
| 8.1.3 | Connect temperature monitoring | Reading ambient temp | |
| 8.1.4 | Connect OBC interface | Serial link established | |
| 8.1.5 | Verify data logging | Recording active | |
| 8.1.6 | Record ambient temperature | T_amb = ____C | |

### 8.2 Power-Up Sequence

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.2.1 | Enable Jetson power (OBC command) | Power applied | |
| 8.2.2 | Monitor inrush current | Peak < 3A | |
| 8.2.3 | Record boot current profile | Capture waveform | |
| 8.2.4 | Measure time to boot complete | < 30 seconds | |
| 8.2.5 | Verify UART response | "READY" message | |
| 8.2.6 | Record steady-state boot current | I_boot = ____mA | |

**Power-Up Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| Inrush peak | < 3A | A | |
| Boot time | < 30s | s | |
| Boot current average | < 1.5A | A | |

### 8.3 Idle Power Measurement

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.3.1 | Wait for thermal stabilization | Temperature stable | |
| 8.3.2 | Verify Jetson in idle state | No active processes | |
| 8.3.3 | Measure power for 60 seconds | Stable reading | |
| 8.3.4 | Record average idle power | P_idle = ____W | |
| 8.3.5 | Record idle current | I_idle = ____mA | |
| 8.3.6 | Record Jetson temperature | T_idle = ____C | |

**Idle Power Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| Idle power | < 5W | W | |
| Idle current | < 1A at 5V | A | |
| Junction temp | < 50C (idle) | C | |

### 8.4 DVFS Profile Verification

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.4.1 | Query current power mode | Mode reported | |
| 8.4.2 | Command low-power mode (7W) | Mode change confirmed | |
| 8.4.3 | Verify CPU frequency reduced | < 1.0 GHz | |
| 8.4.4 | Verify GPU frequency reduced | < 400 MHz | |
| 8.4.5 | Measure power in low-power mode | < 7W | |
| 8.4.6 | Command nominal mode (15W) | Mode change confirmed | |
| 8.4.7 | Measure power in nominal mode | 8-15W | |

**DVFS Record:**

| Mode | Expected | Measured | P/F |
|------|----------|----------|-----|
| 7W mode (idle) | < 5W | W | |
| 7W mode (load) | < 7W | W | |
| 15W mode (idle) | < 8W | W | |
| 15W mode (load) | < 15W | W | |

### 8.5 Inference Power Measurement

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.5.1 | Configure for flight DVFS mode | 7W mode | |
| 8.5.2 | Start inference benchmark | Inference running | |
| 8.5.3 | Monitor power during inference | Capture waveform | |
| 8.5.4 | Measure average inference power | P_infer = ____W | |
| 8.5.5 | Measure peak inference power | P_peak = ____W | |
| 8.5.6 | Record inference rate | ____ tokens/sec | |
| 8.5.7 | Record temperature during inference | T_infer = ____C | |
| 8.5.8 | Stop inference | Return to idle | |

**Inference Power Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| Average inference power | < 8W | W | |
| Peak power | < 10W | W | |
| Temperature during inference | < 85C | C | |

### 8.6 Sustained Load Test

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.6.1 | Start continuous inference | Running | |
| 8.6.2 | Run for 30 minutes | No throttling | |
| 8.6.3 | Record temperature every 5 min | Document trend | |
| 8.6.4 | Record power every 5 min | Document trend | |
| 8.6.5 | Verify no thermal throttling | Frequency stable | |
| 8.6.6 | Record final steady-state temp | T_ss = ____C | |

**Sustained Load Record:**

| Time (min) | Power (W) | Temp (C) | Throttled | Notes |
|------------|-----------|----------|-----------|-------|
| 0 | | | No | |
| 5 | | | | |
| 10 | | | | |
| 15 | | | | |
| 20 | | | | |
| 25 | | | | |
| 30 | | | | |

### 8.7 Power Gating Test

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.7.1 | Command graceful shutdown | Shutdown complete | |
| 8.7.2 | Command power disable (OBC) | 5V rail off | |
| 8.7.3 | Measure residual current | < 1mA | |
| 8.7.4 | Wait 60 seconds | Power off confirmed | |
| 8.7.5 | Command power enable | Power applied | |
| 8.7.6 | Verify boot sequence | READY message | |
| 8.7.7 | Repeat power cycle 3x | All successful | |

**Power Gating Record:**

| Cycle | Shutdown OK | Power Off (mA) | Boot OK | Notes |
|-------|-------------|----------------|---------|-------|
| 1 | | | | |
| 2 | | | | |
| 3 | | | | |

### 8.8 OBC Interface Power Control

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.8.1 | OBC commands Jetson enable | Jetson boots | |
| 8.8.2 | OBC requests health status | Status received | |
| 8.8.3 | OBC commands inference | Beacon text returned | |
| 8.8.4 | OBC commands shutdown | Graceful shutdown | |
| 8.8.5 | OBC commands disable | Power gated off | |
| 8.8.6 | Verify EPS current reduction | System power reduced | |

---

## 9. Pass/Fail Criteria

| Criterion | Requirement | Result |
|-----------|-------------|--------|
| Inrush current | < 3A peak | |
| Boot time | < 30 seconds | |
| Idle power | < 5W | |
| Inference power | < 8W average | |
| Peak power | < 10W | |
| Thermal margin | T_junction < 85C | |
| Power gating | Residual < 1mA | |
| Power cycling | 3/3 successful | |

## 10. Data Recording

### 10.1 Required Data

- [ ] Power waveforms (boot, inference, shutdown)
- [ ] Temperature profiles
- [ ] DVFS mode transitions
- [ ] OBC command/response log
- [ ] Thermal images (optional)

### 10.2 Data Files

| File | Description |
|------|-------------|
| JETSON-001_power.csv | Power time series |
| JETSON-001_temp.csv | Temperature time series |
| JETSON-001_boot.png | Boot power waveform |
| JETSON-001_infer.png | Inference power waveform |

---

## 11. Post-Test

- [ ] Power down Jetson
- [ ] Allow thermal cool-down
- [ ] Review all data
- [ ] Complete test report
- [ ] Document any anomalies

---

## 12. Test Log

| Date | Time | Operator | Notes |
|------|------|----------|-------|
| | | | |
| | | | |
| | | | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
