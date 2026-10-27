# Jetson Orin Nano Integration Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-PAY-JET-001
**Version:** 1.2
**Date:** 2026-10-27
**Status:** CDR Baseline

---

> **CDR note (2026-10-27, version 1.1).** The carrier is now designed: `hardware/electrical/jetson/CARRIER_BOARD_DESIGN.md`
> (SQSO-JET-BRD-001 A). It supersedes §2 (carrier features), §3 (power: the carrier takes VBAT 11.2–16.4 V from the EPS
> and makes the module's 5 V VDD_IN itself), §4 (the module UART is 1.8 V and is translated on the carrier; the OBC
> transmits 8N2), §5 (no microSD on the production module: storage is an M.2 2230 NVMe), §6 (spreader pedestal and gap
> pad; mechanical design §6.10), §9 (PC/104 carrier in the stack under the spreader, MECHANICAL_DESIGN §4.1) and §10.3
> (power-off by `SHUTDOWN` and the carrier's shutdown latch). The "Super" name is wrong for the flight configuration: the
> module is flashed with the original configuration so that the 7 W mode exists (TB-13).
>
> **Documentation audit (2026-10-27, version 1.2).** §1.1 and §7 corrected to the carrier design (7 W mode, VDD_IN
> capped at 6.5 W, ≤ 8 W including the carrier). The thermal limits of §6.1, §6.5 and §11.2 are superseded by SYS-603
> Rev G as carried in BASELINE §6: spreader ≤ +40 °C in a 600 s batch, junction ≤ 95 °C, spreader ≤ +70 °C if stuck on
> (CR-2026-010; thermal interface in SQSO-JET-BRD-001 §14 and `hardware/structure/THERMAL_DESIGN.md`).

## 1. Overview

This document defines the integration design for the NVIDIA Jetson Orin Nano 8 GB AI compute module within the SMART-QSO 1U CubeSat. The module runs only the gated beacon-text experiment (BASELINE §6, decision D5).

### 1.1 Module Specification

| Parameter | Value |
|-----------|-------|
| Module | Jetson Orin Nano 8 GB (P/N 900-13767-0030-000), original (non-Super) flash configuration (BASELINE §6) |
| GPU | Ampere with 1024 CUDA cores |
| CPU | 6-core Arm Cortex-A78AE (4 cores ≤ 960 MHz in the 7 W mode) |
| RAM | 8 GB LPDDR5 |
| Power mode | 7 W (nvpmodel ID 1), VDD_IN capped at 6.5 W; ≤ 8 W input including the carrier (SQSO-JET-BRD-001 §0, §13) |
| Module input | 5 V only (VDD_IN 4.75–5.25 V), made on the carrier from VBAT |
| Interface | 260-pin SO-DIMM |

### 1.2 Integration Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Operating power | < 8W burst | EPS Budget |
| Idle power | < 3W | EPS Budget |
| Interface | UART to OBC | ICD |
| Thermal | Passive cooling | No fans |

---

## 2. Carrier Board Design

### 2.1 Carrier Board Requirements

| Feature | Implementation |
|---------|----------------|
| SO-DIMM connector | 260-pin DDR4 type |
| Power input | 5V @ 3A |
| UART interface | 3.3V LVTTL |
| Storage | MicroSD or NVMe |
| Minimal I/O | UART, power, enable |

### 2.2 Custom vs. COTS

| Approach | Pros | Cons | Selected |
|----------|------|------|----------|
| Custom carrier | Minimal size/power | Design effort | **Yes** |
| COTS carrier | Fast development | Size, extra I/O | No |
| Carrier-less | Smallest | Complex integration | No |

### 2.3 Carrier Board Features

| Feature | Status | Notes |
|---------|--------|-------|
| UART (1) | Required | OBC interface |
| USB | Not populated | Not needed |
| HDMI | Not populated | Not needed |
| Ethernet | Not populated | Not needed |
| CSI Camera | Not populated | Not needed |
| GPIO | Minimal | Power/status only |
| NVMe | Optional | Model storage |
| MicroSD | Required | OS and models |

---

## 3. Power Supply Design

### 3.1 Power Requirements

| Mode | Voltage | Current | Power |
|------|---------|---------|-------|
| Off | - | 0 | 0 |
| Standby | 5V | 0.5A | 2.5W |
| Idle | 5V | 0.8A | 4W |
| Inference | 5V | 1.5A | 7.5W |
| Peak | 5V | 3.0A | 15W |

### 3.2 Power Source

| Parameter | Value |
|-----------|-------|
| Source | PDU 5V rail |
| Input voltage | 5.0V ±5% |
| Inrush limit | < 3A |
| Connector | Molex Micro-Fit 4-pin |

### 3.3 Power Gating

| Parameter | Value |
|-----------|-------|
| Control | OBC GPIO (JETSON_PWR_EN) |
| Switch type | High-side P-FET |
| On resistance | < 20 mΩ |
| Soft-start | 5 ms ramp |
| Current limit | 4A |

### 3.4 Power Sequencing

| Step | Action | Delay |
|------|--------|-------|
| 1 | OBC asserts PWR_EN | 0 ms |
| 2 | 5V applied to Jetson | 5 ms |
| 3 | Jetson auto-boots | 100 ms |
| 4 | UART becomes active | 5 s |
| 5 | Application ready | 30 s |

---

## 4. UART Interface

### 4.1 Electrical Specification

| Parameter | Value |
|-----------|-------|
| Jetson side | 3.3V LVTTL |
| OBC side | 3.3V LVTTL |
| Level shift | Not required (same voltage) |
| ESD protection | TVS diode array |

### 4.2 UART Configuration

| Parameter | Value |
|-----------|-------|
| Baud rate | 115200 |
| Data bits | 8 |
| Stop bits | 1 |
| Parity | None |
| Flow control | None |
| Protocol | JSON messages |

### 4.3 Physical Interface

| Signal | Jetson Pin | OBC Pin | Notes |
|--------|------------|---------|-------|
| TX | UART1_TX | USART1_RX | Jetson → OBC |
| RX | UART1_RX | USART1_TX | OBC → Jetson |
| GND | GND | GND | Common ground |

---

## 5. Storage Interface

### 5.1 MicroSD Card

| Parameter | Value |
|-----------|-------|
| Type | MicroSD UHS-I |
| Capacity | 32-64 GB |
| Speed class | Class 10 / U3 |
| Contents | OS, models, data |

### 5.2 Storage Layout

| Partition | Size | Content |
|-----------|------|---------|
| Boot | 128 MB | Bootloader |
| Root | 8 GB | Linux OS |
| Models | 16 GB | AI models |
| Data | Remaining | Logs, output |

---

## 6. Thermal Design

### 6.1 Thermal Requirements

| Parameter | Value |
|-----------|-------|
| Tjunction max | 105°C |
| Tcase target | < 70°C |
| Ambient (hot) | 50°C |
| Delta T allowed | 20°C |

### 6.2 Thermal Interface

| Component | Material | Thickness |
|-----------|----------|-----------|
| TIM 1 | Fujipoly XR-m | 0.5 mm |
| Heat spreader | Aluminum 6061 | 3 mm |
| TIM 2 | Thermal pad | 1 mm |
| Cold plate | Spacecraft structure | - |

### 6.3 Thermal Path

```
Jetson die → TIM → Heatspreader → TIM → Structure → Radiator
```

### 6.4 Heat Spreader Design

| Parameter | Value |
|-----------|-------|
| Material | Aluminum 6061-T6 |
| Size | 70mm × 50mm × 3mm |
| Mass | ~28g |
| Conductivity | 167 W/m·K |
| Surface finish | Black anodize (ε = 0.85) |

### 6.5 Thermal Analysis

| Condition | Tjunction | Status |
|-----------|-----------|--------|
| Idle, cold orbit | 25°C | OK |
| Inference, hot orbit | 75°C | OK |
| Peak, hot orbit | 90°C | Marginal |
| Thermal throttle | 95°C | Throttle starts |

---

## 7. Power Management

### 7.1 DVFS Configuration

Flight mode only (SQSO-JET-BRD-001 §0): 7 W, nvpmodel ID 1 of the original configuration, 4 CPU cores ≤ 960 MHz,
2 TPC ≤ 408 MHz, EMC 2133 MHz. The 15 W and higher modes are not used; the Super configuration's modes are not
available on the flight flash.

### 7.2 Power Limiting

| Parameter | Setting |
|-----------|---------|
| Power model | 7 W mode (ID 1) |
| Power limit | VDD_IN 6.5 W (INA3221 channel 1 `curr1_max` = 1300 mA); ≤ 8 W at the carrier input |
| Thermal limit | SYS-603 Rev G (BASELINE §6) |
| Fan mode | No fan (passive) |

### 7.3 Runtime Power Control

```bash
# Set power mode (in Jetson Linux)
sudo nvpmodel -m 1  # 7W mode

# Set clock limits
sudo jetson_clocks --store
sudo jetson_clocks --restore
# Each burst ends with SHUTDOWN commanded by the OBC (SQSO-JET-BRD-001 §7.4)
```

---

## 8. EMC Shielding

### 8.1 EMI Considerations

| Source | Mitigation |
|--------|------------|
| GPU switching noise | Shield enclosure |
| Memory interface | Ground plane |
| Power supply | Input filtering |

### 8.2 Shielding Design

| Parameter | Value |
|-----------|-------|
| Enclosure | Aluminum, 1mm thick |
| Apertures | Minimal, honeycomb |
| Grounding | Multiple points to structure |
| Cable filtering | Ferrite beads |

---

## 9. Mechanical Integration

### 9.1 Module Mounting

| Parameter | Value |
|-----------|-------|
| Carrier board size | 80mm × 50mm |
| Mounting | 4× M2 standoffs |
| Location | +Z side of structure |
| Orientation | Heatspreader facing -Z |

### 9.2 Connector Access

| Connector | Access |
|-----------|--------|
| MicroSD | Side access |
| UART header | Top access |
| Power | Cable to PDU |

---

## 10. Software Configuration

### 10.1 Operating System

| Parameter | Value |
|-----------|-------|
| OS | JetPack / L4T |
| Version | Latest stable |
| Boot | Auto-boot from SD |

### 10.2 Startup Services

| Service | Purpose |
|---------|---------|
| uart_interface | OBC communication |
| llm_runner | Beacon text generation |
| health_monitor | System telemetry |
| watchdog | Software watchdog |

### 10.3 Shutdown Handling

- OBC sends shutdown command via UART
- Jetson performs graceful shutdown
- Sync filesystems
- Power removed after 10s timeout

---

## 11. Testing Requirements

### 11.1 Power Tests

| Test | Requirement |
|------|-------------|
| Idle power | < 4W |
| Inference power | < 8W |
| Peak power | < 15W |
| Power gating | < 1 mA when off |

### 11.2 Thermal Tests

| Test | Requirement |
|------|-------------|
| Steady-state idle | Tcase < 50°C |
| Steady-state inference | Tcase < 70°C |
| Thermal throttle point | > 85°C |

### 11.3 Functional Tests

| Test | Requirement |
|------|-------------|
| Boot time | < 60 s |
| UART communication | Reliable |
| LLM inference | Correct output |
| Shutdown | Clean shutdown |

---

## 12. Failure Modes

### 12.1 FMEA Summary

| Failure | Effect | Mitigation |
|---------|--------|------------|
| Jetson no boot | No AI beacon | Template fallback |
| Thermal throttle | Slow inference | Power limiting |
| UART failure | No communication | Watchdog timeout |
| SD card failure | No boot | Backup partition |

### 12.2 Graceful Degradation

If Jetson fails, OBC reverts to template beacon text:
- Pre-stored beacon messages
- Basic telemetry formatting
- Mission continues in degraded mode

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-27 | CDR note: carrier designed (SQSO-JET-BRD-001 A); §2–§6, §9 and §10.3 superseded as listed |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): §1.1 module (8 GB, original configuration, 7 W, 5 V input), §7 power modes and limits to SQSO-JET-BRD-001; thermal limits to SYS-603 Rev G (note) |
