# AI Payload (Jetson Orin Nano) Design Document

**Document Number**: SMART-QSO-PAYLOAD-001
**Version**: 1.3
**Date**: 2026-10-27
**Status**: Preliminary Design

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- NPR 7150.2D - NASA Software Engineering Requirements (Class C, BASELINE §1)
- NVIDIA Jetson Orin Nano Developer Kit Documentation

**Reference missions**: NASA Ames Research Center small-satellite AI/ML technology demonstrations (design patterns consulted, not project heritage)

> **Documentation audit notice (2026-10-27, version 1.3).** Sections 1–10 and the appendices were written before the
> baseline. `docs/BASELINE_PARAMETERS.md` §6 (decision D5), §11 (power gating, trips, thermal, fallback) and §12
> (carrier design SQSO-JET-BRD-001) govern wherever they differ. In particular:
> - **Role.** The Jetson runs only a **gated experiment**: generative beacon text, enabled only if the measured energy
>   per accepted beacon line at the carrier input is ≤ 0.1 Wh (360 J) over ≥ 30 EM bursts (VI-02; decision C-20). The
>   flight baseline is a calibrated template selector on the OBC (`docs/payload/OBC_SELECTOR_DESIGN.md`); the Jetson
>   does not manage power or the mission, and nothing on board learns or adapts (community model updates are not in
>   the flight baseline). §1.3 items 3–4 and "first orbital deployment" are withdrawn.
> - **Model.** SmolLM2-360M-Instruct or Qwen3-0.6B (non-thinking), INT4, Apache 2.0, in batches of ≥ 10 lines
>   (`docs/payload/AI_MODEL_SELECTION.md`); the ~1M-parameter TinyLM of §4.1 and Appendix B is withdrawn.
> - **Content.** **No numbers in generated text**: the OBC writes every value from measurements, and the same
>   deterministic filter runs on the payload and the OBC (`docs/BEACON_SPEC.md` §6; 97.113). The example outputs of
>   §4.1–§4.2 that contain numbers would be rejected.
> - **Bursts.** Each power-on ≤ 10 min including boot and ≤ 3.6 kJ, then ≥ 100 min off; sunlit, SOC ≥ 55 %, Jetson
>   ≥ −15 °C; duty ≤ 5 % tumbling, ≤ 10 % sun-pointed (BASELINE §5–§6). The 30 s per-beacon cycle of §3.4 and §6.3, the
>   20–30 % duty of PAY-P005 and the 15 W peak are withdrawn.
> - **Module.** Orin Nano 8 GB in the original flash configuration, 7 W mode (nvpmodel ID 1), ≤ 8 W input including the
>   carrier, no fan; 5 V input made on the carrier (§12).

---

## 1. Introduction

### 1.1 Purpose

This document defines the detailed design of the AI Payload subsystem for the SMART-QSO 1U CubeSat. The payload uses an NVIDIA Jetson Orin Nano 8 GB module to generate beacon text as a gated experiment (BASELINE §6).

### 1.2 Scope

The AI Payload encompasses:
- Jetson Orin Nano 8 GB compute module (original flash configuration, 7 W mode)
- Power management and duty cycling
- Thermal management
- OBC interface
- AI model execution (TinyML/TinyLM)
- Beacon text generation

### 1.3 Mission Objectives

1. **Technology Demonstration**: First orbital deployment of Jetson Orin for spacecraft autonomy
2. **AI Beacon Generation**: Natural language beacon text generation
3. **Autonomous Operations**: AI-driven power and mission management
4. **Learning Systems**: On-orbit adaptation and optimization

---

## 2. Payload Requirements

### 2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| PAY-F001 | Generate AI-based beacon text | Test |
| PAY-F002 | Execute TinyLM model for text generation | Test |
| PAY-F003 | Interface with OBC via UART | Test |
| PAY-F004 | Respond to telemetry requests | Test |
| PAY-F005 | Support power gating by OBC | Test |
| PAY-F006 | Boot to ready state within 60 seconds | Test |
| PAY-F007 | Generate beacon text within 10 seconds | Test |
| PAY-F008 | Provide fallback to template text on fault | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| PAY-P001 | Inference latency | <10 s | <5 s |
| PAY-P002 | Boot time | <60 s | <30 s |
| PAY-P003 | Active power | <15 W | <10 W |
| PAY-P004 | Idle power | <3 W | <2 W |
| PAY-P005 | Duty cycle (ACTIVE mode) | 20% | 30% |
| PAY-P006 | Model accuracy | >80% | >90% |

### 2.3 Interface Requirements

| ID | Interface | Specification |
|----|-----------|---------------|
| PAY-I001 | Power input | 5V, 3A max |
| PAY-I002 | UART to OBC | 115200 bps, 3.3V logic |
| PAY-I003 | Enable signal | 3.3V logic, active high |
| PAY-I004 | Ready signal | 3.3V logic, active high |
| PAY-I005 | Temperature sensor | I2C, OBC-readable |

### 2.4 Environmental Requirements

| Parameter | Operating | Non-Operating |
|-----------|-----------|---------------|
| Temperature | -10°C to +55°C | -20°C to +70°C |
| Power cycling | >10,000 cycles | N/A |
| Radiation | 1 krad TID | N/A |
| Vacuum | 10⁻⁶ Torr | N/A |

**Note**: Jetson Orin is COTS, not rad-hard. Limited radiation tolerance mitigated by power gating and watchdog.

---

## 3. Payload Architecture

### 3.1 Block Diagram

```
+------------------------------------------------------------------+
|                      AI PAYLOAD SUBSYSTEM                          |
|                                                                    |
|  +------------------------+         +------------------------+     |
|  |  POWER MANAGEMENT      |         |  JETSON ORIN NANO      |     |
|  |                        |         |  SUPER                 |     |
|  |  +------------------+  |   5V    |                        |     |
|  |  | Load Switch      |--|-------->| +------------------+   |     |
|  |  | (TPS22810)       |  |   3A    | | ARM Cortex-A78AE |   |     |
|  |  +------------------+  |         | | 6 cores @ 1.5GHz |   |     |
|  |         ^              |         | +------------------+   |     |
|  |         |              |         |                        |     |
|  |    EN_PAYLOAD          |         | +------------------+   |     |
|  |    (from OBC)          |         | | Ampere GPU       |   |     |
|  |                        |         | | 1024 CUDA cores  |   |     |
|  +------------------------+         | +------------------+   |     |
|                                     |                        |     |
|  +------------------------+         | +------------------+   |     |
|  |  INTERFACE LOGIC       |         | | 8GB LPDDR5       |   |     |
|  |                        |         | | 128-bit @ 102GB/s|   |     |
|  |  +------------------+  |  UART   | +------------------+   |     |
|  |  | Level Shifter    |--|-------->|                        |     |
|  |  | (TXB0104)        |  | 115200  | +------------------+   |     |
|  |  +------------------+  |         | | NVMe SSD         |   |     |
|  |         ^              |         | | 64GB (optional)  |   |     |
|  |         |              |         | +------------------+   |     |
|  |    RDY_PAYLOAD         |         +------------------------+     |
|  |    (to OBC)            |                    |                   |
|  +------------------------+                    |                   |
|                                                v                   |
|                              +------------------------+            |
|                              |  THERMAL MANAGEMENT    |            |
|                              |  - Heatsink            |            |
|                              |  - Thermal strap       |            |
|                              |  - Temperature sensor  |            |
|                              +------------------------+            |
|                                                                    |
+------------------------------------------------------------------+
```

### 3.2 Jetson Orin Nano Specifications (pre-baseline; §12 governs)

| Parameter | Specification |
|-----------|---------------|
| GPU | 1024 CUDA cores, Ampere architecture |
| CPU | 6-core ARM Cortex-A78AE @ 1.5 GHz |
| Memory | 8 GB LPDDR5, 128-bit @ 102 GB/s |
| Storage | 16 GB eMMC (on-module) |
| AI Performance | 67 TOPS (INT8) |
| Power Range | 7W - 25W (configurable) |
| Thermal | Requires active cooling at full power |
| Interface | PCIe, USB, UART, I2C, SPI, GPIO |
| OS | JetPack 6.x (Linux) |

### 3.3 Power Profile

**Declocked Configuration for CubeSat**:

| Mode | CPU Freq | GPU Freq | Power | Usage |
|------|----------|----------|-------|-------|
| Off | - | - | 0 W | Power gated |
| Sleep | 400 MHz | Off | 2 W | Standby |
| Inference | 1.0 GHz | 500 MHz | 8 W | AI execution |
| Peak | 1.5 GHz | 1.0 GHz | 15 W | (Limited) |

**DVFS Configuration** (Dynamic Voltage and Frequency Scaling):
```bash
# Set power mode to 7W mode (ID 1 on the original configuration; ID 0 is 15 W, §12)
sudo nvpmodel -m 1

# Limit GPU clock
sudo jetson_clocks --store
echo 500000000 > /sys/devices/gpu.0/devfreq/*/max_freq

# Limit CPU clock
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/; do
    echo 1000000 > $cpu/scaling_max_freq
done
```

### 3.4 Duty Cycling Strategy

```
Time (30-second beacon cycle)
|<----------- ACTIVE Mode ----------->|
|                                     |
|  OBC Request    Inference   Idle   |
|     |              |         |      |
|     v              v         v      |
+-----+----+----+----+----+----+-----+
|Power|Boot|Wait|Inf |Resp|Idle|Power|
| On  | Up |RDY |    |    |    | Off |
+-----+----+----+----+----+----+-----+
  2s   20s  1s   5s   1s   1s
      <--------  30s  -------->

Power Profile:
- Power On: 8W (boot)
- Inference: 10W (peak)
- Average (30s): 6W
- Duty cycle: 30s active / 30s beacon = 100% in ACTIVE
- But ACTIVE mode only ~50% of orbit, so overall ~50% duty
```

---

## 4. AI Models

### 4.1 TinyLM Beacon Generator

**Model Specifications**:

| Parameter | Value |
|-----------|-------|
| Architecture | Transformer (decoder-only) |
| Parameters | ~1M |
| Model Size | 4 MB (INT8 quantized) |
| Context Length | 64 tokens |
| Vocabulary | 8,000 tokens |
| Inference Time | <5 seconds |
| Output | 20-100 characters |

**Model Input**:
```json
{
  "state_of_charge": 78,
  "power_mode": "ACTIVE",
  "sunlit": true,
  "temperature": 23,
  "qso_count": 42,
  "orbit_number": 1234
}
```

**Model Output**:
```
"I'm sunlit and feeling energetic—batteries topped off at 78%! Ready for some ragchewing! 73!"
```

### 4.2 Context-Aware Generation

| Context | Condition | Example Output |
|---------|-----------|----------------|
| High SOC, Sunlit | SOC>70, SUN=1 | "Batteries full, soaking up the rays!" |
| Low SOC | SOC<30 | "Running low on juice, conserving power" |
| Eclipse Entry | SUN 1→0 | "Heading into shadow, going quiet" |
| Eclipse Exit | SUN 0→1 | "Back in the sun, charging up!" |
| Cold | TEMP<-10 | "Brrr! Getting chilly up here" |
| Hot | TEMP>40 | "Warming up, watching my thermals" |
| Many QSOs | QSO>50 | "What a day! 50+ contacts!" |
| First Light | ORBIT=1 | "First orbit complete, all systems go!" |

### 4.3 Model Safety

**Constraints**:
- Output length: 20-200 characters
- ASCII only: No Unicode, no special characters
- Content filter: No inappropriate content
- Fallback: Template text if generation fails

**Template Fallback Messages**:
```c
const char *fallback_messages[] = {
    "SMART-QSO beacon - AI offline, standby mode",
    "Hello from space! Systems nominal",
    "CubeSat beacon - autonomous AI transponder",
    "SMART-QSO 1U - amateur radio in orbit"
};
```

---

## 5. OBC-Jetson Interface

### 5.1 UART Protocol

**Physical Layer**:
| Parameter | Value |
|-----------|-------|
| Baud Rate | 115200 |
| Data Bits | 8 |
| Parity | None |
| Stop Bits | 1 |
| Flow Control | None |
| Logic Level | 3.3V (via level shifter) |

### 5.2 Message Format

```
+-------+-------+-------+-------+--------+-------+-------+
| START | LEN   | TYPE  | SEQ   | PAYLOAD| CRC16 | END   |
| 0xAA  | 1 byte| 1 byte| 1 byte| N bytes| 2 byte| 0x55  |
+-------+-------+-------+-------+--------+-------+-------+
```

### 5.3 Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| 0x01 | OBC→Jetson | Telemetry data |
| 0x02 | OBC→Jetson | Beacon request |
| 0x03 | OBC→Jetson | Status query |
| 0x81 | Jetson→OBC | Beacon response |
| 0x82 | Jetson→OBC | Status response |
| 0x83 | Jetson→OBC | Error response |

### 5.4 Telemetry Message (0x01)

```c
typedef struct {
    uint32_t timestamp;     // Unix time
    uint8_t  power_mode;    // 0=SAFE, 1=IDLE, 2=ACTIVE
    uint8_t  soc;           // 0-100%
    uint16_t battery_mv;    // mV
    int8_t   temp_obc;      // °C
    int8_t   temp_payload;  // °C
    uint8_t  sunlit;        // 0/1
    uint16_t qso_count;     // Total QSOs
    uint32_t orbit_number;  // Current orbit
} TelemetryPayload_t;
```

### 5.5 Beacon Response (0x81)

```c
typedef struct {
    uint8_t  status;        // 0=OK, 1=Error
    uint8_t  text_len;      // Length of beacon text
    char     text[256];     // Beacon text (null-terminated)
} BeaconResponse_t;
```

### 5.6 Communication Sequence

```
OBC                                         Jetson
 |                                            |
 |------ Power On (EN_PAYLOAD) ------------->|
 |                                            |
 |                                     [Boot] |
 |                                            |
 |<----- Ready Signal (RDY_PAYLOAD) ---------|
 |                                            |
 |------ Telemetry (0x01) ------------------>|
 |                                            |
 |------ Beacon Request (0x02) ------------->|
 |                                            |
 |                                  [Inference]
 |                                            |
 |<----- Beacon Response (0x81) -------------|
 |                                            |
 |------ Power Off (EN_PAYLOAD low) -------->|
 |                                            |
```

### 5.7 Timeout Handling

| Event | Timeout | Action |
|-------|---------|--------|
| Boot (RDY signal) | 60 s | Power cycle |
| Beacon response | 15 s | Use fallback text |
| Status response | 5 s | Log fault |
| No activity | 30 s | Power down |

---

## 6. Power Management

### 6.1 Power Budget

| State | Voltage | Current | Power | Duration |
|-------|---------|---------|-------|----------|
| Off | 0 V | 0 A | 0 W | Variable |
| Boot | 5 V | 2.0 A | 10 W | 20 s |
| Idle | 5 V | 0.5 A | 2.5 W | Variable |
| Inference | 5 V | 2.0 A | 10 W | 5 s |

### 6.2 Power Sequencing

**Power On**:
```
1. Check EPS SOC > 55%
2. Check thermal < 50°C
3. Assert EN_PAYLOAD high
4. Start 60s boot timer
5. Wait for RDY_PAYLOAD high
6. Begin communication
```

**Power Off**:
```
1. Send shutdown command (optional)
2. Wait 1s for graceful shutdown
3. Assert EN_PAYLOAD low
4. Confirm power down via current sense
```

### 6.3 Energy Budget Analysis

```
Beacon cycle: 30 seconds
Jetson active: 25 seconds (boot + inference)
Jetson power: 10W average during active

Energy per beacon: 10W × 25s = 250 J = 0.069 Wh

Beacons per hour: 120 (30s interval)
Hourly energy: 0.069 × 120 = 8.3 Wh (Jetson only)

With 50Wh battery and 50% average SOC:
Available energy: 25 Wh
Runtime at full duty: 25 / 8.3 = 3 hours

Sustainability requires:
- Solar generation > 8.3 W average, or
- Reduced duty cycle (every other beacon)
```

---

## 7. Thermal Management

### 7.1 Thermal Budget

| Component | Dissipation | Allowable Rise |
|-----------|-------------|----------------|
| Jetson SoC | 10 W (peak) | 30°C |
| VRM | 1 W | 20°C |
| RAM | 0.5 W | 10°C |
| Total | 11.5 W | - |

### 7.2 Thermal Design

**Passive Cooling**:
- Aluminum heatsink bonded to module
- Thermal strap to spacecraft structure
- Radiator area on -Z face

**Active Management**:
- Temperature-based duty cycling
- Thermal throttling via DVFS
- Emergency shutdown at 70°C

### 7.3 Thermal Limits

| Condition | Junction Temp | Action |
|-----------|---------------|--------|
| Normal | <65°C | Full operation |
| Warm | 65-75°C | Reduce duty cycle |
| Hot | 75-85°C | Throttle frequency |
| Critical | >85°C | Emergency shutdown |

### 7.4 Thermal Strap Design

```
    +-------------+
    |   Jetson    |
    |   Module    |
    +------+------+
           |
    +------+------+
    |  Heatsink   |
    |  (Al 6061)  |
    +------+------+
           |
    Thermal Strap (Copper braid)
           |
    +------+------+
    |  Structure  |
    |  (-Z Face)  |
    +-------------+
           |
           v
       RADIATOR
```

**Thermal Strap Specifications**:
| Parameter | Value |
|-----------|-------|
| Material | Copper braid |
| Length | 50 mm |
| Cross-section | 10 mm² |
| Conductance | >5 W/K |

---

## 8. Software Stack

### 8.1 Operating System

| Component | Version |
|-----------|---------|
| JetPack | 6.0 |
| Linux Kernel | 5.15 |
| CUDA | 12.2 |
| TensorRT | 8.6 |
| cuDNN | 8.9 |

### 8.2 Payload Software Architecture

```
+----------------------------------------------------------+
|                    APPLICATION LAYER                       |
|  +------------------+  +------------------+               |
|  | Beacon Generator |  | Health Monitor   |               |
|  +------------------+  +------------------+               |
+----------------------------------------------------------+
|                    AI FRAMEWORK LAYER                      |
|  +------------------+  +------------------+               |
|  | TensorRT         |  | PyTorch (dev)    |               |
|  | Runtime          |  | ONNX Export      |               |
|  +------------------+  +------------------+               |
+----------------------------------------------------------+
|                    COMMUNICATION LAYER                     |
|  +------------------+  +------------------+               |
|  | UART Driver      |  | Protocol Handler |               |
|  +------------------+  +------------------+               |
+----------------------------------------------------------+
|                    OPERATING SYSTEM                        |
|  +--------------------------------------------------+    |
|  | JetPack 6.0 (Ubuntu 22.04 + NVIDIA extensions)   |    |
|  +--------------------------------------------------+    |
+----------------------------------------------------------+
```

### 8.3 Boot Optimization

**Standard Boot**: ~45 seconds
**Optimized Boot**: ~20 seconds

Optimizations:
1. Disable unused services (GUI, network)
2. Preload AI model to memory
3. Use initramfs for faster filesystem
4. Defer non-essential initialization

```bash
# Disable unnecessary services
systemctl disable gdm
systemctl disable NetworkManager
systemctl disable bluetooth

# Enable fast boot
systemctl enable smart-qso-payload
```

### 8.4 Payload Main Loop

```python
#!/usr/bin/env python3
import serial
import tensorrt as trt
from payload_protocol import *

class SmartQsoPayload:
    def __init__(self):
        self.uart = serial.Serial('/dev/ttyTHS0', 115200)
        self.model = load_tinyml_model('/opt/smart-qso/beacon_model.engine')
        self.ready = False

    def run(self):
        # Signal ready
        set_ready_gpio(True)
        self.ready = True

        while True:
            msg = self.receive_message()

            if msg.type == MSG_TELEMETRY:
                self.update_state(msg.payload)

            elif msg.type == MSG_BEACON_REQUEST:
                text = self.generate_beacon()
                self.send_beacon_response(text)

            elif msg.type == MSG_STATUS_QUERY:
                self.send_status()

    def generate_beacon(self):
        try:
            input_tensor = self.prepare_input()
            output = self.model.infer(input_tensor)
            text = self.decode_output(output)
            return self.validate_text(text)
        except Exception as e:
            log_error(e)
            return FALLBACK_MESSAGE

if __name__ == '__main__':
    payload = SmartQsoPayload()
    payload.run()
```

---

## 9. Verification

### 9.1 Verification Matrix

| Requirement | Method | Status |
|-------------|--------|--------|
| PAY-F001 | Test | Pending |
| PAY-F002 | Test | Pending |
| PAY-F003 | Test | Pending |
| PAY-F004 | Test | Pending |
| PAY-F005 | Test | Pending |
| PAY-F006 | Test | Pending |
| PAY-F007 | Test | Pending |
| PAY-F008 | Test | Pending |

### 9.2 Test Procedures

| Test | Description | Equipment |
|------|-------------|-----------|
| TP-PAY-001 | Boot time measurement | Timer, logic analyzer |
| TP-PAY-002 | Inference latency | Profiling, timestamps |
| TP-PAY-003 | Power measurement | Power analyzer |
| TP-PAY-004 | Thermal cycling | Thermal chamber |
| TP-PAY-005 | UART communication | Serial analyzer |
| TP-PAY-006 | Model accuracy | Test dataset |
| TP-PAY-007 | Stress test | Extended operation |

### 9.3 Integration Testing

**OBC-Payload Integration**:
1. Power sequencing verification
2. Communication protocol validation
3. End-to-end beacon generation
4. Fault injection and recovery
5. Duty cycle validation

---

## 10. Risk Assessment

### 10.1 Payload-Specific Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| PAY-R001 | Radiation-induced failure | M | H | Power gating, watchdog |
| PAY-R002 | Thermal runaway | L | H | DVFS limiting, shutdown |
| PAY-R003 | AI model failure | L | M | Fallback templates |
| PAY-R004 | Boot loop | L | M | Boot counter, safe mode |
| PAY-R005 | Memory corruption | M | M | CRC checks, restart |
| PAY-R006 | Excessive power draw | L | M | Current limiting |

### 10.2 Radiation Mitigation

**Strategies**:
1. **Power Gating**: Minimize exposure time
2. **Watchdog**: Detect and recover from hangs
3. **Scrubbing**: Periodic memory validation
4. **Triple Modular Redundancy**: For critical decisions

**Expected SEU Rate**:
- LEO environment: ~10⁻¹⁰ upsets/bit/day
- 8GB RAM: ~8 upsets/day
- Power gating reduces exposure ~80%
- Net: ~1-2 upsets/day (manageable with watchdog)

---

## 11. CDR Update (2026-10-27): Power Gating, Latch-up, Thermal and Fallback

Sections 6, 7 and 10 above are the 2026-01 preliminary design. Where they disagree with this section, this section and
`docs/BASELINE_PARAMETERS.md` §6 govern. The flight AI is the OBC-resident selector (decision D5,
`docs/payload/OBC_SELECTOR_DESIGN.md`); the Jetson is a gated experiment.

### 11.1 Two Switches in Series

| Switch | Line (provisional pin) | Part (`docs/hardware/EPS_DESIGN.md` v2.0) | Acts on | Opened by |
|--------|------------------------|---------------------------------------------|---------|-----------|
| SW1, PDU high-side switch | `PAYLOAD_EN` (PB2; was PB4, SQSO-ICX-001) | TPS1H100 class, ≥ 25 V | VBAT to the carrier | flight software; hardware low-voltage disconnect (EPS-F012) |
| U3, carrier eFuse (the EPS board's U19, §12) | `JETSON_PWR` (PA8; was PA2, SQSO-ICX-001) | TPS25947 class, 1.2 A, latch-off, ≤ 1 ms | carrier converter input | flight software; its own current limit (latch-up) |

*Independence.* The two are different parts on different lines, so one stuck-on switch leaves the other able to cut
the rail; this is what keeps the bus alive with a stuck Jetson (BUILD_VS_BUY §7.3: with both switches failed the pack
is flat in every simulated case). **Finding (RID AV-06):** the flight software drives both lines from one state bit
(`eps_control.rs`, `drive_load_pins`), in the same call. A software fault that leaves the bit set holds both on; the
independence against software faults then rests on the hardware disconnect (SW1) and the eFuse (U3), which is
acceptable only if both are verified. Recommended at the EM: drive `JETSON_PWR` low first and `PAYLOAD_EN` 10 ms later
on switch-off (the converter discharges into a closed path), read each switch's state back (power-good of U3, a sense
of SW1's output) and log a disagreement as a fault; test each switch alone opening the rail (RAD-V5).

### 11.2 Latch-up and Over-power Trips

| Layer | Threshold | Time | Then |
|-------|-----------|------|------|
| U3 eFuse (hardware) | 1.2 A ≈ 17 W at 14.4 V | ≤ 1 ms | latched off until `JETSON_PWR` is cycled |
| `payload_guard` latch-up (software) | one sample ≥ 12 W | next guard sample (≤ 1 s) | both lines low, 10 min off, then until the spreader is 10 °C below its limit |
| `payload_guard` over-power (software) | > 8 W on 2 samples in a row | ≤ 2 s | as above |
| `payload_guard` telemetry loss | no power or spreader sample for 5 s | 5 s | as above |

The Orin Nano's operating maximum at the declocked point is 8 W (BASELINE §6), 0.56 A; the eFuse trips at 2.1× that,
the software latch-up at 1.5×. A latch-up adds amperes, so the eFuse sees it. The SIL mission run injects latch-ups and
requires every one cleared by the rail cut within 10 s (check `seu-recovery`). The Jetson's radiation rates are in
`docs/environment/RADIATION_ANALYSIS.md` Rev C §5: DRAM upsets 0.001–0.1 a day at a 10 % duty, not the "~8 a day" of
§10.2, and the "scrubbing" and "TMR" of §10.2 are not implemented on the Jetson; none is needed for a gated experiment
whose every output passes the OBC's filter.

### 11.3 Thermal

The OBC decides from its own spreader thermistor, not from the Jetson's report: 70 °C spreader and 95 °C junction
(reported) trip the rail; restart only 10 °C below the spreader limit and after 10 min. A burst starts only with the
spreader below its limit and the SOC and power gates of BASELINE §5 holding. The 14-day SIL commissioning run reaches
25 °C on the spreader (165 power-ons, 0.016 Wh per accepted beacon). §7.3's 65–85 °C junction ladder is the Jetson's
own DVFS behaviour, not a flight-software function.

### 11.4 Fallback

The beacon never depends on the Jetson. The OBC sends a pre-approved template unless a Jetson text arrives in time and
passes the content filter (`docs/BEACON_SPEC.md` §6); the transponder, telecommand and TX-OFF do not involve the
Jetson. The payload rail is off: in SAFE and EMERGENCY, below the SOC gate, during a GNSS session, and from ARM_FLIGHT
until the authenticated commissioning release (SRS-F082). `PLD_ENABLE` is refused while the hold is set.

### 11.5 Open Items

- Readback of both switches and the switch-off order (RID AV-06).
- Back-powering of the unpowered module through the idle USART1 TX line (RID AV-09,
  `docs/interfaces/ICD_OBC_JETSON.md` §9).
- Thermal: the spreader-to-structure path is analysed by the thermal reviewer; the guard limits above do not change.

## 12. Carrier Board (2026-10-27, SQSO-JET-BRD-001 A)

The carrier is designed to part level in `hardware/electrical/jetson/CARRIER_BOARD_DESIGN.md` (schematic as code, netlist,
BOM, ERC, calculations). It corrects this document where they disagree:

| Item | This document before | Carrier design (governs) |
|------|----------------------|--------------------------|
| Module and power input | "Orin Nano Super", 5 V 3 A (§2.3), "7 W–25 W" | Orin Nano 8 GB (900-13767-0030-000, P3767-0003); **VDD_IN 4.75–5.25 V only** (DS-11105-001 v1.5 Table 5-1: the 5–20 V range is the Orin NX's). The carrier converts VBAT to 5.02 V (LM61440-Q1) behind an eFuse with 5.38 V overvoltage lock-out and a 2.51 A latching circuit breaker (TPS259474L) |
| Second switch | "U3 carrier eFuse, TPS25947 class, 1.2 A" (§11.1) | the 1.2 A eFuse on JETSON_PWR is the EPS board's U19 (TPS16630); the carrier adds its own VDD_IN breaker (§11.2's layers gain one: 2.51 A at 5 V, ≤ 3.2 ms) |
| Storage | 16 GB eMMC on the module (§3.2); microSD (INTEGRATION_DESIGN §5) | the production module has only QSPI-NOR boot flash; rootfs, models and data on a Swissbit N2600 40 GB pSLC M.2 2230 NVMe (PCIe C4, Gen2 x1) |
| UART | 3.3 V, 8N1 | module UART1 is 1.8 V CMOS, translated on the carrier; the OBC transmits 8N2 (DS §2.10.4) |
| Power mode (TB-13) | "nvpmodel -m 0" as 7 W (§3.3) | 7 W is mode ID 1 of the **original** flash configuration (ID 0 is 15 W); the Super configuration has no 7 W mode. Default mode set to ID 1, plus a 6.5 W VDD_IN average cap, because at the full 7 W budget the carrier input is 7.7–8.0 W: the 8 W limit is met only with the cap (7.2–7.5 W) |
| Ready signal | RDY_PAYLOAD GPIO (§2.3, Appendix A) | none: the carrier's JET_3V3 on W5B pin 4 is the power-good; the OBC waits for STATUS |
| Power-off | enable low after an optional shutdown | `SHUTDOWN` → module `poweroff` → carrier latch → JET_3V3 low → OBC opens the switches (ICD-002 1.3 §10.4) |
| Thermal (§7.4) | copper braid to −Z | spreader pedestal on the SoC lid through a 1.0 mm gap pad, ≤ 38 N clamp (module limit 80 N); junction-to-spreader 1.1–1.2 K/W estimated against the 5.7 K/W allowable (THM-10) |
| Spreader temperature | "OBC's own thermistor" (§11.3), no hardware path | 10 kΩ NTC on the spreader, passed through the carrier on W5B pins 5–6 to an OBC ADC channel (interface change E10, for the chair) |

Appendix A's pinout (40-pin developer-kit header) does not apply to the flight carrier; the module pins used are listed in
the carrier design §11.

---

## Appendix A: Jetson Orin Nano Pinout

| Pin | Function | Usage |
|-----|----------|-------|
| Pin 6 | GND | Ground |
| Pin 1 | 3.3V | Level shifter power |
| Pin 8 | UART1_TX | To OBC RX |
| Pin 10 | UART1_RX | From OBC TX |
| Pin 12 | GPIO_17 | RDY signal out |
| Pin 2,4 | 5V | Not used (external supply) |

---

## Appendix B: Model Training

**Training Data**:
- 10,000 synthetic beacon examples
- Context: SOC, temperature, mode, etc.
- Output: Human-friendly status text

**Training Pipeline**:
```
Raw Text Data → Tokenization → Model Training → Quantization → TensorRT Export
     ↓              ↓              ↓              ↓              ↓
  10K examples   8K vocab     1M params      INT8           4MB .engine
```

**Validation**:
- BLEU score: >0.7
- Human evaluation: 90% acceptable
- Safety review: 100% compliant

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | SMART-QSO Team (CDR avionics review) | §11 new: two series switches and their independence (RID AV-06), latch-up and over-power trips, thermal limits, fallback; §6, §7 and §10 marked as preliminary where §11 governs |
| 1.2 | 2026-10-27 | Payload carrier design (CDR design completion) | §12 new: carrier board SQSO-JET-BRD-001 summary and the corrections it makes (5 V-only module input, storage, 1.8 V UART and 8N2, 7 W mode and the 6.5 W cap for TB-13, power-good, commanded switch-off, thermal interface, spreader thermistor) (amended: §11.1 pins PB2 and PA8 per SQSO-ICX-001, panel item P3E-05) |
| 1.3 | 2026-10-27 | SMART-QSO Team (documentation audit) | Documentation audit (post-CDR 2026): notice at the top listing where §1–§10 and the appendices conflict with BASELINE §5–§6 (gated experiment only, SmolLM2/Qwen3 INT4 model, no numbers in generated text, burst rules, module); §1.1, §1.2, §3.2 "Super" removed; §3.3 nvpmodel ID 1 (was 0); NPR 7150.2D; heritage line reworded |
