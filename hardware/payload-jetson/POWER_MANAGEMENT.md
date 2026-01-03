# Jetson Payload Power Management
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-JET-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the power management strategy for the Jetson Orin Nano Super AI payload, including DVFS configuration, power limiting, thermal management, and integration with the OBC power control system.

---

## 2. Power Requirements

### 2.1 Jetson Orin Nano Super Specifications

| Parameter | Typical | Maximum | Notes |
|-----------|---------|---------|-------|
| Input voltage | 5.0V | 5.25V | Regulated supply |
| Idle power | 5W | 7W | No inference |
| Inference power | 10W | 15W | Peak during AI burst |
| Burst power | 15W | 25W | GPU max load |
| Standby (off) | 0 | 1 mA | Power gated |

### 2.2 Mission Power Budget Allocation

| Mode | Jetson Allocation | Notes |
|------|-------------------|-------|
| Safe Mode | 0W | Power gated |
| Idle Mode | 0W | Power gated |
| Active Mode | 5W | Idle, ready for inference |
| AI Burst Mode | 8W max | Limited for thermal |

### 2.3 Duty Cycle Requirements

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Maximum ON time | 10 minutes | Thermal limit |
| Minimum OFF time | 5 minutes | Cool-down |
| Maximum duty cycle | 50% | Power budget |
| Typical duty cycle | 10-20% | Normal operations |

---

## 3. DVFS Configuration

### 3.1 DVFS (Dynamic Voltage and Frequency Scaling) Overview

The Jetson Orin Nano supports multiple power modes through DVFS, controlling:
- CPU frequency and voltage
- GPU frequency and voltage
- Memory frequency
- Overall power cap

### 3.2 Power Mode Selection

| Mode | Power Cap | CPU Max | GPU Max | Use Case |
|------|-----------|---------|---------|----------|
| 7W Mode | 7W | 1.2 GHz | 306 MHz | Standby/idle |
| 10W Mode | 10W | 1.5 GHz | 510 MHz | **Selected for flight** |
| 15W Mode | 15W | 1.7 GHz | 625 MHz | Development only |
| MAXN | 25W | 2.0 GHz | 1.0 GHz | Disabled for flight |

**Flight Selection: 10W Mode**

### 3.3 DVFS Configuration Commands

```bash
# Set power mode to 10W (mode 1)
sudo nvpmodel -m 1

# Verify power mode
nvpmodel -q

# Set maximum frequencies within 10W mode
sudo jetson_clocks --show
```

### 3.4 Power Limit Configuration

```bash
# Set GPU power cap (if applicable)
echo 8000 | sudo tee /sys/kernel/debug/tegra_dc/powergate

# Monitor current power
cat /sys/bus/i2c/drivers/ina3221x/*/iio:device*/in_power*_input
```

---

## 4. Software Power Control

### 4.1 OBC Interface

| Signal | Type | Function |
|--------|------|----------|
| JETSON_PWR_EN | GPIO Out | Power enable (active high) |
| JETSON_STATUS | GPIO In | Jetson running status |
| UART_TX | UART | OBC to Jetson |
| UART_RX | UART | Jetson to OBC |

### 4.2 Power Control State Machine

```
┌──────────────┐
│  POWER_OFF   │◄──────────────────────────────┐
└──────┬───────┘                               │
       │ OBC: PWR_EN=1                         │
       ▼                                       │
┌──────────────┐                               │
│   BOOTING    │ Timeout (60s) ───────────────►│
└──────┬───────┘                               │
       │ STATUS=1                              │
       ▼                                       │
┌──────────────┐                               │
│    IDLE      │◄──────────────────────────────┤
└──────┬───────┘                               │
       │ Inference request                     │
       ▼                                       │
┌──────────────┐                               │
│  INFERENCE   │ Complete or timeout ─────────►│
└──────┬───────┘                               │
       │ Inference complete                    │
       │                                       │
       ▼                                       │
┌──────────────┐                               │
│  SHUTDOWN    │────────────────────────────────┘
└──────────────┘
       │ Graceful shutdown (5s)
       ▼
       OBC: PWR_EN=0
```

### 4.3 Power Gating Procedure

**Power ON Sequence:**
1. OBC checks power budget (SOC > 50%)
2. OBC checks thermal budget (< 60°C ambient)
3. OBC sets JETSON_PWR_EN = 1
4. Wait for JETSON_STATUS = 1 (timeout: 60s)
5. Verify UART communication
6. Jetson ready for inference

**Power OFF Sequence:**
1. OBC sends shutdown command via UART
2. Wait for graceful shutdown (timeout: 10s)
3. Verify JETSON_STATUS = 0
4. OBC sets JETSON_PWR_EN = 0
5. Verify current < 1 mA

### 4.4 Emergency Power Off

| Condition | Action |
|-----------|--------|
| Overtemperature (>85°C) | Immediate PWR_EN=0 |
| Battery critical (<10% SOC) | Immediate PWR_EN=0 |
| Communication timeout (>30s) | Power cycle |
| Excessive current (>3A) | Immediate PWR_EN=0 |

---

## 5. Thermal Management

### 5.1 Thermal Limits

| Point | Warning | Shutdown |
|-------|---------|----------|
| Jetson junction | 85°C | 95°C |
| Heatspreader | 60°C | 70°C |
| Ambient (structure) | 50°C | 60°C |

### 5.2 Passive Cooling Design

| Component | Specification |
|-----------|---------------|
| Heatspreader | Aluminum, 3mm thick |
| TIM (Jetson to spreader) | Fujipoly 3 W/m·K |
| TIM (Spreader to structure) | Gap pad 2 W/m·K |
| Heat path | Jetson → Spreader → Structure → Radiator |

### 5.3 Fan Configuration

**Fan disabled for flight** (no active cooling)

```bash
# Disable fan control
sudo systemctl disable nvfancontrol

# Verify fan is off
cat /sys/devices/pwm-fan/target_pwm
# Should return 0
```

### 5.4 Thermal Throttling

| Temperature | Action |
|-------------|--------|
| 75°C | Reduce CPU frequency |
| 80°C | Reduce GPU frequency |
| 85°C | Send warning to OBC |
| 90°C | Begin graceful shutdown |
| 95°C | Hardware thermal shutdown |

---

## 6. Power Monitoring

### 6.1 Internal Monitoring (Jetson)

The Jetson monitors power via INA3221 on-board sensors:

| Rail | Measurement | Units |
|------|-------------|-------|
| VDD_IN | Total input power | mW |
| VDD_CPU | CPU power | mW |
| VDD_GPU | GPU power | mW |
| VDD_SOC | SoC power | mW |

```python
# Python code to read power
def read_jetson_power():
    import subprocess
    result = subprocess.run(['tegrastats', '--interval', '1000'],
                          capture_output=True, timeout=2)
    # Parse output for power values
    return power_mw
```

### 6.2 External Monitoring (OBC)

The OBC monitors Jetson power via:

| Parameter | Method |
|-----------|--------|
| Input current | EPS current sense on 5V rail |
| Power enable status | GPIO readback |
| Communication status | UART heartbeat |
| Temperature | Jetson reports via UART |

### 6.3 Telemetry Points

| Parameter | ID | Format | Rate |
|-----------|-----|--------|------|
| Jetson power state | TM_JET_STATE | Enum | 1 Hz |
| Jetson power (mW) | TM_JET_PWR | U16 | 1 Hz |
| Jetson temp (°C) | TM_JET_TEMP | S8 | 1 Hz |
| Inference count | TM_JET_INF | U32 | 0.1 Hz |

---

## 7. Inference Power Profile

### 7.1 Typical Inference Power Profile

```
Power (W)
   │
12 ┤                    ╭──────╮
   │                   ╱        ╲
10 ┤                  ╱          ╲
   │                 ╱            ╲
 8 ┤                ╱              ╲
   │               ╱                ╲
 6 ┤   ╭─────────╮╱                  ╲
   │  ╱           Load Model          ╲
 5 ┤─╯  Idle                    Idle   ╲──
   │
   └────┬───────┬────────┬────────┬────────
       0s      2s       5s       8s      10s
               ◄─ Inference Time ─►
```

### 7.2 Power Budget Enforcement

```python
class JetsonPowerManager:
    MAX_POWER_MW = 8000  # 8W limit
    MAX_TEMP_C = 85

    def can_start_inference(self):
        if self.get_power_mw() > self.MAX_POWER_MW:
            return False
        if self.get_temperature() > self.MAX_TEMP_C:
            return False
        if not self.obc_power_authorized():
            return False
        return True

    def monitor_during_inference(self):
        while self.inference_running:
            if self.get_power_mw() > self.MAX_POWER_MW * 1.1:
                self.throttle_gpu()
            if self.get_temperature() > self.MAX_TEMP_C:
                self.abort_inference()
            time.sleep(0.1)
```

---

## 8. Fault Protection

### 8.1 Power Faults

| Fault | Detection | Response |
|-------|-----------|----------|
| Overcurrent | > 3A for 100ms | Power off, log |
| Overvoltage | > 5.5V | Power off, log |
| Undervoltage | < 4.5V | Power off, log |
| Overtemperature | > 85°C | Power off, log |

### 8.2 Communication Faults

| Fault | Detection | Response |
|-------|-----------|----------|
| Boot timeout | No STATUS after 60s | Power cycle |
| UART timeout | No response 30s | Send reset command |
| Persistent timeout | 3 consecutive | Power off, fault flag |

### 8.3 Recovery Procedures

| Condition | Recovery |
|-----------|----------|
| Single power fault | Wait 5 min, retry once |
| Repeated power fault | Disable Jetson for orbit |
| Thermal fault | Wait until < 60°C |
| Communication fault | Power cycle, retry 3x |

---

## 9. Integration Testing

### 9.1 Power Tests

| Test | Procedure | Pass Criteria |
|------|-----------|---------------|
| Idle power | Boot to idle, measure | < 7W |
| Inference power | Run inference, measure | < 10W |
| Power gating | PWR_EN low, measure | < 1 mA |
| Boot current | Monitor during boot | < 3A peak |
| Shutdown | Command shutdown | Clean, < 10s |

### 9.2 Thermal Tests

| Test | Procedure | Pass Criteria |
|------|-----------|---------------|
| Idle thermal | Run idle 30 min | < 60°C |
| Burst thermal | Run inference 10 min | < 75°C |
| Recovery | After burst, monitor | < 60°C in 5 min |

### 9.3 TVAC Power Profile

| Condition | Expected Power |
|-----------|----------------|
| Hot case (+50°C) | +10% above nominal |
| Cold case (-20°C) | -5% below nominal |
| Vacuum | Same as ambient |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
