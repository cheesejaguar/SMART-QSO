# Radio Frequency (RF) Subsystem Design Document

**Document Number**: SMART-QSO-RF-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary Design

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- ITU Radio Regulations - Amateur Satellite Service
- FCC Part 97 - Amateur Radio Service
- IARU Satellite Frequency Coordination

**Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This document defines the detailed design of the Radio Frequency (RF) subsystem for the SMART-QSO 1U CubeSat. The mission employs a beacon-only architecture with AI-powered autonomous operations.

### 1.2 Scope

The RF subsystem encompasses:
- Beacon transmitter (UHF/VHF)
- Antenna system (deployable monopole)
- RF power amplifier
- Modulation and protocol (AX.25 AFSK)
- Ground station interface

### 1.3 Design Philosophy

SMART-QSO implements a **beacon-only RF architecture**:
- No uplink command capability
- AI-driven autonomous decision-making
- Continuous telemetry broadcast via intelligent beacons
- Simplified hardware, reduced complexity

---

## 2. RF Requirements

### 2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| RF-F001 | Transmit beacon at 1200 bps AFSK | Test |
| RF-F002 | Support AX.25 UI frame format | Test |
| RF-F003 | Transmit on amateur VHF band (145-146 MHz) | Test |
| RF-F004 | Provide software-controllable TX power | Test |
| RF-F005 | Deploy antenna after separation from deployer | Test |
| RF-F006 | Include valid amateur callsign in beacon | Inspection |
| RF-F007 | Support beacon interval of 30-180 seconds | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| RF-P001 | TX power | 0.5-1.0W | 0.5-2.0W selectable |
| RF-P002 | Link margin (500km, 10° elev) | >6 dB | >10 dB |
| RF-P003 | Modulation accuracy | <5% deviation | <3% deviation |
| RF-P004 | Frequency stability | ±5 kHz | ±2 kHz |
| RF-P005 | Spurious emissions | <-40 dBc | <-50 dBc |
| RF-P006 | Antenna deployment reliability | >95% | >99% |

### 2.3 Regulatory Requirements

| ID | Requirement | Authority |
|----|-------------|-----------|
| RF-R001 | IARU frequency coordination | IARU |
| RF-R002 | FCC Part 97 compliance | FCC |
| RF-R003 | ITU Region 2 compliance | ITU |
| RF-R004 | Amateur license requirement | National authority |
| RF-R005 | No encryption (amateur service) | FCC/ITU |

### 2.4 Environmental Requirements

| Parameter | Operating | Non-Operating |
|-----------|-----------|---------------|
| Temperature | -40°C to +70°C | -55°C to +85°C |
| Vibration | 14.1 Grms (random) | N/A |
| Vacuum | 10⁻⁶ Torr | N/A |
| Radiation | 10 krad TID | N/A |

---

## 3. RF Architecture

### 3.1 Block Diagram

```
+------------------------------------------------------------------+
|                        SMART-QSO RF SUBSYSTEM                      |
|                                                                    |
|  +------------+     +------------+     +------------+     +-----+  |
|  |   OBC      |     |  BEACON    |     |    PA      |     | LPF |  |
|  | (Telemetry)|---->|  MODULATOR |---->| (0.5-2W)   |---->|     |--|
|  +------------+     |  (AFSK)    |     +-----+------+     +--+--+  |
|                     +------------+           |               |     |
|                                              |               |     |
|                     +------------+           |               |     |
|                     |   TCXO     |-----------+               |     |
|                     | (145 MHz)  |                           |     |
|                     +------------+                           |     |
|                                                              |     |
+--------------------------------------------------------------+-----+
                                                               |
                                                               v
                                                    +------------------+
                                                    |    DEPLOYABLE    |
                                                    |     ANTENNA      |
                                                    |  (1/4λ monopole) |
                                                    +------------------+
```

### 3.2 Transmitter

#### 3.2.1 Beacon Modulator

**Selected Approach**: Software-defined modulation via OBC

| Parameter | Specification |
|-----------|---------------|
| Modulation | AFSK (Bell 202) |
| Mark Frequency | 1200 Hz |
| Space Frequency | 2200 Hz |
| Bit Rate | 1200 bps |
| FM Deviation | ±3 kHz |
| Protocol | AX.25 UI frames |

**Implementation**:
```c
// AFSK modulation constants
#define AFSK_MARK_FREQ    1200  // Hz
#define AFSK_SPACE_FREQ   2200  // Hz
#define AFSK_BAUD_RATE    1200  // bps
#define AFSK_SAMPLE_RATE  9600  // Hz

// Bit period = 8 samples at 9600 Hz
// Mark = 1 cycle per bit (1200 Hz)
// Space = 1.833 cycles per bit (2200 Hz)
```

#### 3.2.2 Power Amplifier

**Selected Component**: Custom Class-E PA or COTS Module

| Parameter | Specification |
|-----------|---------------|
| Frequency | 145-146 MHz |
| Output Power | 0.5-2.0W (adjustable) |
| Efficiency | >60% |
| Input Power | +0 dBm nominal |
| Supply Voltage | 12V |
| Current | 170mA @ 1W, 300mA @ 2W |
| Harmonics | <-30 dBc |

**Power Control**:
| Setting | Output Power | DC Current |
|---------|--------------|------------|
| LOW | 0.5W (+27 dBm) | 100 mA |
| MEDIUM | 1.0W (+30 dBm) | 170 mA |
| HIGH | 2.0W (+33 dBm) | 300 mA |

#### 3.2.3 Frequency Reference

**Selected Component**: TCXO 145.xxx MHz

| Parameter | Specification |
|-----------|---------------|
| Nominal Frequency | 145.xxx MHz (TBD after IARU) |
| Stability | ±2.5 ppm (-40 to +85°C) |
| Aging | <1 ppm/year |
| Phase Noise | <-110 dBc/Hz @ 10 kHz |
| Power | 3.3V, 5 mA |

### 3.3 Antenna System

#### 3.3.1 Antenna Design

**Type**: Quarter-wave monopole with ground plane

| Parameter | Specification |
|-----------|---------------|
| Configuration | 1/4λ monopole |
| Length | ~50 cm (145 MHz) |
| Material | Beryllium-copper tape |
| Width | 10 mm |
| Thickness | 0.1 mm |
| Stowed Length | Coiled, <10mm diameter |
| Gain | ~2 dBi (typical) |
| Polarization | Linear (vertical) |
| Pattern | Omnidirectional (azimuth) |

#### 3.3.2 Deployment Mechanism

**Method**: Burn-wire release with spring deployment

```
        STOWED                          DEPLOYED
    +----------+                    +----------+
    |  Coiled  |                    |          |
    | Antenna  |                    |          |
    |  Spring  |   Burn Wire        |          |
    |    ↓     | =============>     |          |
    |  [~~~~]  |   Activation       |          |
    |  Latch   |                    |    |     |
    +----------+                    +----+-----+
                                         |
                                         v
                                    1/4λ Monopole
                                    (~50 cm)
```

**Deployment Sequence**:
1. Separation detection (30-min timer or accelerometer)
2. Burn-wire activation (2A, 3s pulse)
3. Spring-loaded tape uncoils
4. Deployment confirmation via status GPIO

**Redundancy**:
- Dual burn wires (independent circuits)
- Backup timer (45 min from power-on)
- Manual ground command (future uplink if added)

### 3.4 Output Filter

**Type**: 7-pole Chebyshev Low-Pass Filter

| Parameter | Specification |
|-----------|---------------|
| Passband | DC - 150 MHz |
| Cutoff (-3dB) | 155 MHz |
| Insertion Loss | <0.5 dB |
| 2nd Harmonic (290 MHz) | >40 dB rejection |
| 3rd Harmonic (435 MHz) | >50 dB rejection |

---

## 4. Link Budget

### 4.1 Downlink Budget (145 MHz)

| Parameter | Value | Units | Notes |
|-----------|-------|-------|-------|
| **Transmitter** | | | |
| TX Power | +30.0 | dBm | 1W nominal |
| TX Line Loss | -0.5 | dB | Coax + connectors |
| TX Antenna Gain | +2.0 | dBi | 1/4λ monopole |
| EIRP | +31.5 | dBm | |
| | | | |
| **Path** | | | |
| Slant Range | 1500 | km | 10° elevation |
| Frequency | 145.0 | MHz | |
| Free Space Loss | -143.7 | dB | |
| Atmospheric Loss | -0.5 | dB | |
| Ionospheric Loss | -1.0 | dB | |
| Polarization Loss | -3.0 | dB | Linear mismatch |
| Total Path Loss | -148.2 | dB | |
| | | | |
| **Receiver** | | | |
| RX Antenna Gain | +10.0 | dBi | Yagi (typical) |
| RX Line Loss | -1.0 | dB | |
| Received Power | -107.7 | dBm | |
| | | | |
| **Noise** | | | |
| System Temp | 500 | K | Ground station |
| Bandwidth | 15.0 | kHz | AFSK receiver |
| Noise Power | -131.1 | dBm | N = kTB |
| | | | |
| **Performance** | | | |
| C/N | +23.4 | dB | |
| Required C/N | +10.0 | dB | For BER < 10⁻⁵ |
| **Link Margin** | **+13.4** | **dB** | Exceeds 6 dB req |

### 4.2 Link Budget Variations

| Condition | Margin Change | Total Margin |
|-----------|---------------|--------------|
| Nominal (1W, 10° elev) | 0 dB | +13.4 dB |
| Low power (0.5W) | -3 dB | +10.4 dB |
| High elevation (45°) | +5 dB | +18.4 dB |
| Horizon (5°) | -4 dB | +9.4 dB |
| Rain fade (heavy) | -2 dB | +11.4 dB |
| Antenna pointing error | -2 dB | +11.4 dB |

---

## 5. Beacon Protocol

### 5.1 AX.25 Frame Format

```
+-------+-------+-------+-------+-------+-------+---------+-------+
| FLAG  | DEST  | SRC   | CTRL  |  PID  | INFO  | FCS     | FLAG  |
| 0x7E  | ADDR  | ADDR  | 0x03  | 0xF0  | DATA  | CRC-16  | 0x7E  |
+-------+-------+-------+-------+-------+-------+---------+-------+
  1 byte 7 bytes 7 bytes 1 byte  1 byte  N bytes  2 bytes   1 byte
```

**Field Details**:

| Field | Size | Content |
|-------|------|---------|
| FLAG | 1 | 0x7E (frame delimiter) |
| DEST ADDR | 7 | "CQ    " + SSID (broadcast) |
| SRC ADDR | 7 | Callsign + SSID |
| CTRL | 1 | 0x03 (UI frame) |
| PID | 1 | 0xF0 (no L3 protocol) |
| INFO | ≤256 | Telemetry + AI text |
| FCS | 2 | CRC-16-CCITT |
| FLAG | 1 | 0x7E (frame delimiter) |

### 5.2 Beacon Content

#### 5.2.1 Telemetry Format

```
de [CALL]-1: [AI_TEXT] | T=[TIMESTAMP] M=[MODE] SOC=[SOC] BV=[BATT_V]
SUN=[SUN] RF=[RF_PWR] QSO=[QSO_CNT] TMP=[TEMP]
```

**Example**:
```
de K6ARC-1: I'm sunlit and feeling energetic—ready for some ragchewing! 73! |
T=2026-01-02T12:34:56Z M=ACTIVE SOC=78 BV=7.8 SUN=1 RF=1 QSO=42 TMP=23
```

#### 5.2.2 Telemetry Fields

| Field | Format | Description | Range |
|-------|--------|-------------|-------|
| T | ISO8601 | UTC timestamp | - |
| M | String | Power mode | SAFE/IDLE/ACTIVE |
| SOC | Integer | State of charge | 0-100 % |
| BV | Float | Battery voltage | 6.0-8.4 V |
| SUN | Boolean | Sunlit flag | 0/1 |
| RF | Integer | TX power level | 0-2 |
| QSO | Integer | QSO counter | 0-65535 |
| TMP | Integer | OBC temperature | -40 to +85 °C |

#### 5.2.3 AI-Generated Content

The Jetson payload generates context-aware text:

| Context | Example Message |
|---------|-----------------|
| High SOC, Sunlit | "Batteries topped off and ready for action!" |
| Eclipse Entry | "Heading into shadow—going quiet to save power" |
| Low Power | "Running low on juice, conserving energy" |
| High QSO Count | "What a day! 50 QSOs and counting!" |
| Cold Temperature | "Brrr! Getting chilly up here at -20°C" |

### 5.3 Beacon Timing

| Mode | Interval | Rationale |
|------|----------|-----------|
| ACTIVE | 30 s | Full capability, frequent updates |
| IDLE | 60 s | Moderate power conservation |
| SAFE | 180 s | Maximum power conservation |
| Transition | Immediate | Mode change notification |

---

## 6. Frequency Coordination

### 6.1 IARU Coordination

**Process**:
1. Submit coordination request to IARU
2. Provide technical parameters
3. Receive frequency assignment
4. Update documentation

**Requested Parameters**:
| Parameter | Request |
|-----------|---------|
| Service | Amateur Satellite |
| Band | 2m (144-146 MHz) |
| Mode | FM, 1200 bps AFSK |
| Bandwidth | 15 kHz |
| Power | 1W (nominal) |
| Antenna | Omnidirectional |

### 6.2 Regulatory Compliance

| Requirement | Implementation |
|-------------|----------------|
| Callsign ID | Every transmission |
| No encryption | All data in clear |
| Power limit | <2W (amateur) |
| Frequency tolerance | <5 kHz |
| Spurious emissions | ITU compliant |

---

## 7. Hardware Implementation

### 7.1 RF PCB Layout

**Critical Considerations**:
- 50Ω microstrip traces for RF paths
- Ground plane under all RF components
- Separate analog and digital grounds
- Shielding for PA section
- Thermal relief for PA transistor

### 7.2 Component List

| Component | Part Number | Qty | Function |
|-----------|-------------|-----|----------|
| TCXO | TBD | 1 | Reference oscillator |
| PA Module | TBD | 1 | Power amplifier |
| LPF | Custom | 1 | Harmonic filter |
| RF Switch | PE42421 | 1 | Antenna switch (if needed) |
| SMA Connector | SMA-KHD | 2 | RF interface |
| Antenna | Custom | 1 | Deployable monopole |

### 7.3 Power Consumption

| State | Current (12V) | Power |
|-------|---------------|-------|
| Idle (TX off) | 10 mA | 0.12 W |
| TX Low (0.5W) | 100 mA | 1.2 W |
| TX Med (1.0W) | 170 mA | 2.0 W |
| TX High (2.0W) | 300 mA | 3.6 W |

---

## 8. Ground Segment

### 8.1 Ground Station Requirements

| Parameter | Minimum | Recommended |
|-----------|---------|-------------|
| Antenna | Dipole | Yagi (>10 dBi) |
| Receiver | RTL-SDR | SDRPlay RSP1A |
| Preamp | Optional | Mast-mounted LNA |
| Software | Direwolf | GNU Radio + custom |
| Tracking | Manual | Auto (rotator) |

### 8.2 Decoding Software

**Recommended**: Direwolf + Custom Parser

```bash
# Direwolf configuration for SMART-QSO
ADEVICE plughw:1,0
CHANNEL 0
MYCALL N0CALL
MODEM 1200
PTT NONE
```

### 8.3 SatNOGS Integration

SMART-QSO beacons will be uploaded to SatNOGS network:
- Telemetry forwarding enabled
- Kaitai struct decoder provided
- Dashboard integration planned

---

## 9. Verification

### 9.1 Verification Matrix

| Requirement | Method | Status |
|-------------|--------|--------|
| RF-F001 | Test | Pending |
| RF-F002 | Test | Pending |
| RF-F003 | Test | Pending |
| RF-F004 | Test | Pending |
| RF-F005 | Test | Pending |
| RF-F006 | Inspection | Pending |
| RF-F007 | Test | Pending |

### 9.2 Test Procedures

| Test | Description | Equipment |
|------|-------------|-----------|
| TP-RF-001 | TX power measurement | Power meter, attenuator |
| TP-RF-002 | Frequency accuracy | Spectrum analyzer |
| TP-RF-003 | Modulation verification | Demodulator, BER tester |
| TP-RF-004 | Spurious emissions | Spectrum analyzer |
| TP-RF-005 | Antenna deployment | Visual, continuity test |
| TP-RF-006 | Link test | RF test range |
| TP-RF-007 | TVAC operation | TVAC chamber |

### 9.3 EMC Testing

| Test | Standard | Limit |
|------|----------|-------|
| Conducted Emissions | MIL-STD-461G CE102 | Per standard |
| Radiated Emissions | MIL-STD-461G RE102 | Per standard |
| Spurious Response | FCC Part 97 | <-40 dBc |

---

## 10. Risk Assessment

### 10.1 RF-Specific Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| RF-R001 | Antenna deployment failure | L | H | Dual burn wires, testing |
| RF-R002 | TCXO drift beyond spec | L | M | Temperature compensation |
| RF-R003 | PA failure | L | H | Conservative derating |
| RF-R004 | Frequency coordination denial | L | M | Early coordination request |
| RF-R005 | Interference to/from other satellites | L | L | IARU coordination |

---

## Appendix A: Antenna Radiation Pattern

```
         +90° (Zenith)
              |
         \    |    /
          \   |   /
           \  |  /
            \ | /
    ---------[*]---------  Horizon (0°)
            / | \
           /  |  \
          /   |   \
         /    |    \
              |
         -90° (Nadir)

Legend:
[*] = Spacecraft
Pattern: Omnidirectional in azimuth
         Null at +Z (zenith)
         Peak near horizon
```

---

## Appendix B: AX.25 Bit Stuffing

```
Original: 01111110 (6 consecutive 1s)
Stuffed:  011111010 (0 inserted after 5 1s)

Purpose: Ensure FLAG (0x7E = 01111110) is unique
```

---

## Appendix C: CRC-16-CCITT Calculation

```c
uint16_t crc16_ccitt(const uint8_t *data, size_t len) {
    uint16_t crc = 0xFFFF;
    for (size_t i = 0; i < len; i++) {
        crc ^= data[i];
        for (int j = 0; j < 8; j++) {
            if (crc & 1)
                crc = (crc >> 1) ^ 0x8408;
            else
                crc >>= 1;
        }
    }
    return crc ^ 0xFFFF;
}
```

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
