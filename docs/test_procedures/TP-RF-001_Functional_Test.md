# Test Procedure: RF Functional Test
## TP-RF-001

**Document Number:** TP-RF-001
**Revision:** D
**Date:** 2026-01-02
**Classification:** Development/Qualification Test

**Requirements verified:** SYS-200, SYS-201, SYS-202, SYS-203, SYS-206, SYS-210, SYS-211, SYS-212

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
| SMART-QSO-RF-001 | RF Design Document (`docs/hardware/RF_DESIGN.md`); SQSO-RF-BRD-001, RF board build-to design |
| SMART-QSO-ICD-006 | Beacon Format ICD |
| SQSO-REG-001 | IARU Satellite Frequency Coordination — Status (`docs/regulatory/IARU_COORDINATION.md`) |

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

- **RF Hazard:** Output up to 1 W (SYS-201), use dummy load
- **Electrical Hazard:** 5 V PA supply (5V_PA) from the 14.4 V bus

**Required PPE:** Safety glasses, ESD wrist strap

**Emergency Procedures:**
- Never transmit without load connected
- Never exceed 1 W output power (SYS-201)

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

### 8.2 Transmitter Power Output [SYS-201]

| Step | Power Setting | Expected | Measured | Tolerance | P/F |
|------|---------------|----------|----------|-----------|-----|
| 7 | Low (0.5W) | 27 dBm | _______ | ±1 dB | |
| 8 | Maximum (1W) | 30 dBm | _______ | -1 / +0 dB (never above 1 W) | |
| 9 | Highest setting the radio accepts | ≤ 30 dBm | _______ | Must not exceed 1 W | |

### 8.3 Frequency Accuracy [SYS-200, SYS-206]

| Step | Parameter | Expected | Measured | Tolerance | P/F |
|------|-----------|----------|----------|-----------|-----|
| 10 | Carrier frequency | The IARU-coordinated downlink frequency (TBD until coordination, TBD-SYS-001; SYS-200) | _______ | ±1 kHz | |
| 11 | Frequency stability (1 min) | < 100 Hz drift | _______ | | |
| 12 | Frequency vs temperature | TBD | _______ | ±1 ppm/°C | |

### 8.4 Modulation Quality [SYS-202]

| Step | Parameter | Expected | Measured | Tolerance | P/F |
|------|-----------|----------|----------|-----------|-----|
| 13 | FM deviation (1200 Hz tone) | ±3.0 kHz (10K4F2D, BASELINE §7; TP-RF-004) | _______ | ±0.5 kHz | |
| 14 | FM deviation (2200 Hz tone) | ±3.0 kHz | _______ | ±0.5 kHz | |
| 15 | Occupied bandwidth | ≤ 10.4 kHz (10K4F2D) | _______ | | |
| 16 | Audio bandpass (300-3000 Hz) | Pass | _______ | | |

### 8.5 AX.25 Frame Generation [SYS-210, SYS-212]

| Step | Test | Expected | Result | P/F |
|------|------|----------|--------|-----|
| 17 | Transmit test beacon | Frame generated | _______ | |
| 18 | Verify destination callsign | "CQ" | _______ | |
| 19 | Verify source callsign | KK6GDB-11 | _______ | |
| 20 | Verify SSID | 11 (KK6GDB-11; `software/flight/include/station_id.h`) | _______ | |
| 21 | Verify control byte | 0x03 (UI) | _______ | |
| 22 | Verify PID byte | 0xF0 (no L3) | _______ | |
| 23 | Verify FCS (CRC-16) | Correct | _______ | |
| 24 | Decode with TNC | Clean decode | _______ | |

### 8.6 Beacon Content Verification

| Step | Field | Expected | Observed | P/F |
|------|-------|----------|----------|-----|
| 25 | Header | "de KK6GDB-11:" | _______ | |
| 26 | Text content | ASCII printable | _______ | |
| 27 | Telemetry block | Present | _______ | |
| 28 | Timestamp | Valid | _______ | |
| 29 | Power mode | SAFE/IDLE/ACTIVE | _______ | |
| 30 | Total length | < 256 bytes | _______ | |

### 8.7 Beacon Timing [SYS-211]

| Step | Mode | Expected Interval | Measured | Tolerance | P/F |
|------|------|-------------------|----------|-----------|-----|
| 31 | SAFE | 180 seconds | _______ | ±1 sec | |
| 32 | IDLE | 120 seconds | _______ | ±1 sec | |
| 33 | ACTIVE | 60 seconds | _______ | ±1 sec | |

### 8.8 Spurious Emissions [SYS-203]

| Step | Frequency | Limit | Measured | P/F |
|------|-----------|-------|----------|-----|
| 34 | 2nd harmonic (2 × f_c, 291.6–292.0 MHz) | ≤ −46 dBc and ≤ 25 µW at 1 W (SYS-203; 47 CFR 97.307(e)) | _______ | |
| 35 | 3rd harmonic (3 × f_c, 437.4–438.0 MHz) | ≤ −70 dBc at the antenna port (SYS-203 Rev G; `docs/BASELINE_PARAMETERS.md` §7) | _______ | |
| 36 | Out-of-band (DC-30 MHz) | ≤ −46 dBc and ≤ 25 µW | _______ | |
| 37 | Out-of-band (300-1000 MHz) | ≤ −46 dBc and ≤ 25 µW | _______ | |

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
- FM deviation within ±0.5 kHz of 3.0 kHz
- All AX.25 frames decode correctly
- Spurious emissions at least 46 dB below carrier and ≤ 25 µW at 1 W (SYS-203; 47 CFR 97.307(e)); third harmonic at least 70 dB below carrier (SYS-203 Rev G)
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
| B | 2026-10-16 | Beacon Format ICD cited by its own number, SMART-QSO-ICD-006 (was SQSO-ICD-003, the OBC-to-RF ICD) (finding PI-17) | SMART-QSO Team |
| C | 2026-10-16 | §8.3 cites SYS-206, frequency accuracy (formerly a duplicate ID); its 145.825 MHz carrier is flagged as conflicting with the frequency requirement (finding VV-06) | SMART-QSO Team |
| D | 2026-10-16 | Steps corrected to the requirements and cited: carrier the IARU-coordinated frequency (SYS-200, TBD-SYS-001; was 145.825 MHz); power 0.5 W and 1 W maximum, the 2 W step and the 2 W safety limit removed (SYS-201); SSID 11 (was 1); beacon intervals 180/120/60 s in SAFE/IDLE/ACTIVE (SYS-211; were 120/60/30 s); harmonics as multiples of the carrier, < -40 dBc (SYS-203) and ≤ 25 µW (47 CFR 97.307(e)) (finding VV-06) | SMART-QSO Team |
