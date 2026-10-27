# Jetson Payload Power Management
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-JET-002
**Version:** 1.2
**Date:** 2026-10-27
**Status:** CDR Baseline

---

> **CDR note (2026-10-27, version 1.1).** BASELINE §6 governs: the flight mode is **7 W**, not 10 W (§3.2 is
> superseded; the Orin Nano 8 GB has no 10 W mode). On the original flash configuration the 7 W mode is nvpmodel ID 1
> (4 CPU cores ≤ 960 MHz, 2 GPU TPC ≤ 408 MHz, EMC 2133 MHz; Jetson Linux r36.4.4); ID 0 is 15 W and is the factory
> default, so `/etc/nvpmodel.conf` sets `PM_CONFIG DEFAULT=1`. The Super configuration has no 7 W mode (TB-13). The
> carrier design (SQSO-JET-BRD-001 §13) adds a 6.5 W cap on the module input (`curr1_max` = 1300 mA on INA3221
> channel 1, VDD_IN) so that the carrier input stays ≤ 8 W with margin. The input is VBAT through the carrier's own 5 V
> converter, not a 5 V rail (§2.1), and the power-off is `SHUTDOWN` → `poweroff` (§4.2 timeouts follow ICD-002 1.3 §10).
>
> **Documentation audit (2026-10-27, version 1.2).** §2.1, §2.3, §3.2, §4.1, §4.3, §4.4 and §5.1 corrected to
> BASELINE §5–§6, `docs/hardware/PAYLOAD_DESIGN.md` §11 and SQSO-JET-BRD-001. The OBC-side trips and gates of
> PAYLOAD_DESIGN §11 govern this document's §4–§5.

## 1. Overview

This document defines the power management strategy for the Jetson Orin Nano 8 GB AI payload (original flash configuration, gated beacon-text experiment), including DVFS configuration, power limiting, thermal management, and integration with the OBC power control system.

---

## 2. Power Requirements

### 2.1 Jetson Orin Nano 8 GB in the Flight Configuration

| Parameter | Value | Notes |
|-----------|-------|-------|
| Module input (VDD_IN) | 4.75–5.25 V (5.5 V absolute) | Made on the carrier from VBAT 11.2–16.4 V (SQSO-JET-BRD-001 §0) |
| Power mode | 7 W (nvpmodel ID 1) | Original configuration; the only flight mode |
| Module cap | 6.5 W at VDD_IN | INA3221 `curr1_max` = 1300 mA |
| Burst input | 7.19–7.48 W modelled with the cap, ≤ 8 W at the carrier input | BASELINE §6; SQSO-JET-BRD-001 §13 |
| Off | Both series switches open | PAYLOAD_EN and JETSON_PWR low |

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
| Maximum ON time | 10 minutes per power-on, boot included, ≤ 3.6 kJ at the carrier input | BASELINE §6; decision C-20 |
| Minimum OFF time | 100 minutes | BASELINE §6 (`sqso_fsw::payload_schedule`) |
| Maximum duty cycle | ≤ 5 % tumbling, ≤ 10 % sun-pointed | Power budget (BASELINE §5) |
| Gates | Sunlit, SOC ≥ 55 %, Jetson temperature ≥ −15 °C at power-on | BASELINE §6 |

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
| 7W Mode (ID 1) | 7W | 960 MHz, 4 cores | 408 MHz, 2 TPC | **Selected for flight** (BASELINE §6) |
| 15W Mode (ID 0) | 15W | — | — | Factory default; not used in flight |

The Orin Nano 8 GB has no 10 W mode; the Super configuration's modes (15 W, 25 W, MAXN_SUPER) do not exist on the
flight flash.

**Flight Selection: 7W Mode (nvpmodel ID 1), VDD_IN capped at 6.5 W**

### 3.3 DVFS Configuration Commands

```bash
# Set power mode to 7W (ID 1 on the original configuration)
sudo nvpmodel -m 1

# Verify power mode
nvpmodel -q

# Show the frequencies of the 7W mode
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
Pins per `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001) and PAYLOAD_DESIGN §11.1.

| Signal | Type | Function |
|--------|------|----------|
| PAYLOAD_EN (PB2) | GPIO Out | EPS high-side switch SW1, VBAT to the carrier |
| JETSON_PWR (PA8) | GPIO Out | EPS eFuse U19 on the carrier's input |
| JET_3V3 / power-good | Input (W5B pin 4) | Carrier I/O rail, high only after the module's power sequence |
| USART1 TX / RX | UART (8N2 from the OBC) | OBC to and from the Jetson, translated from 1.8 V on the carrier |

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
1. OBC checks the gates of BASELINE §6: sunlit, SOC ≥ 55 %, spreader below its limit, Jetson temperature ≥ −15 °C
2. OBC checks the burst schedule (≥ 100 min since the last power-off)
3. OBC sets PAYLOAD_EN = 1, then JETSON_PWR = 1
4. Wait for the carrier power-good (timeout: 60s)
5. Verify UART communication
6. Jetson ready for inference

**Power OFF Sequence:**
1. OBC sends shutdown command via UART
2. Wait for graceful shutdown (timeout: 10s)
3. Verify the power-good falls (≤ 5 s, then cut anyway; SQSO-JET-BRD-001 §19 F2)
4. OBC sets JETSON_PWR = 0, then PAYLOAD_EN = 0 10 ms later
5. Verify current < 1 mA

### 4.4 Emergency Power Off

| Condition | Action |
|-----------|--------|
Governed by PAYLOAD_DESIGN §11.2–§11.3; summary:

| Condition | Action |
|-----------|--------|
| Spreader ≥ 70 °C or reported junction ≥ 95 °C | Both lines low; restart only 10 °C below the limit and after 10 min |
| One power sample ≥ 12 W (latch-up) or > 8 W on two samples | Both lines low, 10 min off |
| No power or spreader sample for 5 s | Both lines low |
| Bus below 13.6 V | Hardware load shed of the Jetson rail (EPS) |
| Current ≥ 1.2 A at the EPS eFuse | Hardware trip, latched until JETSON_PWR is cycled |

---

## 5. Thermal Management

### 5.1 Thermal Limits

| Point | Warning | Shutdown |
|-------|---------|----------|
| Jetson junction | — | 95°C (SYS-603 Rev G) |
| Heatspreader | +40 °C in a 600 s batch (design limit) | 70 °C (stuck-on limit; OBC trip) |
| Power-on gate | — | Jetson ≥ −15 °C to start |

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
| 1.1 | 2026-10-27 | CDR note: 7 W mode (ID 1) instead of 10 W, 6.5 W input cap, VBAT input through the carrier, commanded power-off (SQSO-JET-BRD-001 A) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): §2.1 flight-configuration figures (Super figures removed), §2.3 burst rules of BASELINE §6 (≥ 100 min off, ≤ 5/10 % duty, was 5 min and 50 %), §3.2 7 W selected, §4.1 signals PAYLOAD_EN/JETSON_PWR per SQSO-ICX-001, §4.3 gates (SOC ≥ 55 %), §4.4 and §5.1 trips and limits per PAYLOAD_DESIGN §11 and SYS-603 Rev G |
