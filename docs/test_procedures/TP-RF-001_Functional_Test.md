# Test Procedure: RF Functional Test
## TP-RF-001

**Document Number:** TP-RF-001
**Revision:** A
**Date:** 2026-01-02
**Classification:** Development/Qualification Test

---

## 1. Purpose

Verify the functional operation of the RF communication subsystem including transmitter power, frequency, modulation, and AX.25 frame generation.

## 2. Scope

This procedure tests the RF subsystem in standalone and integrated configurations:
- Transmitter power output
- Frequency accuracy and stability
- AFSK modulation quality
- AX.25 frame generation
- Beacon timing
- Antenna interface

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-RF-001 | RF Design Document |
| SQSO-ICD-003 | Beacon Format ICD |
| SQSO-REG-001 | Regulatory Compliance Document |

## 4. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | RF Board + OBC |
| Serial Number | _____________ |
| Firmware Version | _____________ |
| Configuration | Integrated with OBC |

## 5. Test Equipment

| Equipment | Specification | Cal Due |
|-----------|---------------|---------|
| Spectrum Analyzer | 9kHz-3GHz | _______ |
| Power Meter | 10MHz-18GHz | _______ |
| Signal Generator | 9kHz-3GHz | _______ |
| Frequency Counter | 12 digit | _______ |
| Oscilloscope | 100MHz, 4 channel | _______ |
| 50Ω Dummy Load | 5W rating | _______ |
| Attenuator | 30dB, 2W | _______ |
| TNC/Decoder | 1200 bps AFSK | _______ |
| SDR Receiver | RTL-SDR or better | _______ |

## 6. Preconditions

- [ ] RF board assembled and inspected
- [ ] OBC programmed with test firmware
- [ ] All test equipment calibrated
- [ ] Test cables characterized (loss known)
- [ ] Ambient temperature: 20-25°C
- [ ] IARU-coordinated frequency programmed

## 7. Safety

- **RF Hazard:** Output up to 2W, use dummy load
- **Electrical Hazard:** 12V supply to PA

**Required PPE:** Safety glasses, ESD wrist strap

**Emergency Procedures:**
- Never transmit without load connected
- Never exceed 2W output power

## 8. Procedure

### 8.1 Test Setup

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 1 | Connect RF output to attenuator | Connection secure | |
| 2 | Connect attenuator to power meter | Connection secure | |
| 3 | Connect spectrum analyzer via coupler | Monitoring active | |
| 4 | Connect TNC to monitoring receiver | Decoder ready | |
| 5 | Power on OBC and RF subsystem | Subsystem powered | |
| 6 | Verify RF subsystem in standby | No RF output | |

### 8.2 Transmitter Power Output

| Step | Power Setting | Expected | Measured | Tolerance | P/F |
|------|---------------|----------|----------|-----------|-----|
| 7 | Low (0.5W) | 27 dBm | _______ | ±1 dB | |
| 8 | Medium (1W) | 30 dBm | _______ | ±1 dB | |
| 9 | High (2W) | 33 dBm | _______ | ±1 dB | |

### 8.3 Frequency Accuracy

| Step | Parameter | Expected | Measured | Tolerance | P/F |
|------|-----------|----------|----------|-----------|-----|
| 10 | Carrier frequency | 145.825 MHz | _______ | ±1 kHz | |
| 11 | Frequency stability (1 min) | < 100 Hz drift | _______ | | |
| 12 | Frequency vs temperature | TBD | _______ | ±1 ppm/°C | |

### 8.4 Modulation Quality

| Step | Parameter | Expected | Measured | Tolerance | P/F |
|------|-----------|----------|----------|-----------|-----|
| 13 | FM deviation (1200 Hz tone) | ±3.5 kHz | _______ | ±0.5 kHz | |
| 14 | FM deviation (2200 Hz tone) | ±3.5 kHz | _______ | ±0.5 kHz | |
| 15 | Modulation index | 3.5-4.0 | _______ | | |
| 16 | Audio bandpass (300-3000 Hz) | Pass | _______ | | |

### 8.5 AX.25 Frame Generation

| Step | Test | Expected | Result | P/F |
|------|------|----------|--------|-----|
| 17 | Transmit test beacon | Frame generated | _______ | |
| 18 | Verify destination callsign | "CQ" | _______ | |
| 19 | Verify source callsign | [CALLSIGN] | _______ | |
| 20 | Verify SSID | 1 | _______ | |
| 21 | Verify control byte | 0x03 (UI) | _______ | |
| 22 | Verify PID byte | 0xF0 (no L3) | _______ | |
| 23 | Verify FCS (CRC-16) | Correct | _______ | |
| 24 | Decode with TNC | Clean decode | _______ | |

### 8.6 Beacon Content Verification

| Step | Field | Expected | Observed | P/F |
|------|-------|----------|----------|-----|
| 25 | Header | "de [CALLSIGN]:" | _______ | |
| 26 | Text content | ASCII printable | _______ | |
| 27 | Telemetry block | Present | _______ | |
| 28 | Timestamp | Valid | _______ | |
| 29 | Power mode | SAFE/IDLE/ACTIVE | _______ | |
| 30 | Total length | < 256 bytes | _______ | |

### 8.7 Beacon Timing

| Step | Mode | Expected Interval | Measured | Tolerance | P/F |
|------|------|-------------------|----------|-----------|-----|
| 31 | SAFE | 120 seconds | _______ | ±1 sec | |
| 32 | IDLE | 60 seconds | _______ | ±1 sec | |
| 33 | ACTIVE | 30 seconds | _______ | ±1 sec | |

### 8.8 Spurious Emissions

| Step | Frequency | Limit | Measured | P/F |
|------|-----------|-------|----------|-----|
| 34 | 2nd harmonic (291.65 MHz) | < -43 dBc | _______ | |
| 35 | 3rd harmonic (437.475 MHz) | < -43 dBc | _______ | |
| 36 | Out-of-band (DC-30 MHz) | < -40 dBc | _______ | |
| 37 | Out-of-band (300-1000 MHz) | < -40 dBc | _______ | |

### 8.9 TX/RX Isolation (if applicable)

| Step | Condition | Expected | Measured | P/F |
|------|-----------|----------|----------|-----|
| 38 | TX off, RX sensitivity | -120 dBm | _______ | |
| 39 | TX on, RX blocking | No damage | _______ | |

### 8.10 Antenna Interface

| Step | Test | Expected | Measured | P/F |
|------|------|----------|----------|-----|
| 40 | VSWR at operating freq | < 2.0:1 | _______ | |
| 41 | Return loss | > 10 dB | _______ | |
| 42 | Connector mating | Secure | _______ | |

### 8.11 Test Completion

| Step | Action | Result |
|------|--------|--------|
| 43 | Stop all transmissions | Complete |
| 44 | Download test data | Complete |
| 45 | Power off RF subsystem | Complete |
| 46 | Disconnect test equipment | Complete |

## 9. Pass/Fail Criteria

**Pass:**
- Power output within ±1 dB of specification
- Frequency within ±1 kHz
- FM deviation within ±0.5 kHz of 3.5 kHz
- All AX.25 frames decode correctly
- Spurious emissions below IARU limits
- Beacon timing within ±1 second

**Fail:** Any parameter outside specification.

## 10. Data Recording

Record all measurements in RF Test Data Log:
- Spectrum analyzer screenshots
- Power meter readings
- Decoded beacon captures
- Timing measurements
- Any anomalies

## 11. Post-Test

- [ ] All data recorded and archived
- [ ] Test equipment calibration verified
- [ ] Test article inspected
- [ ] Test report completed
- [ ] Anomalies documented

---

## Test Execution Record

| Field | Entry |
|-------|-------|
| Date | _____________ |
| Operator | _____________ |
| Witness | _____________ |
| Overall Result | PASS / FAIL |
| Anomalies | _____________ |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
