# AI Payload (Jetson Orin Nano) Design Document

**Document Number**: SMART-QSO-PAYLOAD-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary Design

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- NPR 7150.2 Rev B - NASA Software Engineering Requirements
- NVIDIA Jetson Orin Nano Developer Kit Documentation

**Heritage**: NASA Ames Research Center Small Satellite Division - AI/ML technology demonstration

---

## 1. Introduction

### 1.1 Purpose

This document defines the detailed design of the AI Payload subsystem for the SMART-QSO 1U CubeSat. The payload uses an NVIDIA Jetson Orin Nano Super to perform onboard AI inference for autonomous spacecraft operations.

### 1.2 Scope

The AI Payload encompasses:
- Jetson Orin Nano Super compute module
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

### 3.2 Jetson Orin Nano Super Specifications

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
# Set power mode to 7W mode
sudo nvpmodel -m 0

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
