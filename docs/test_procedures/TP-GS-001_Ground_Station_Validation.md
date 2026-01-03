# TP-GS-001: Ground Station Validation Test Procedure

**Document ID**: SQSO-TP-GS-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This procedure validates the complete ground station system for SMART-QSO mission operations, verifying end-to-end functionality from RF reception through telemetry processing.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-GSD-001 | Ground Station Design |
| SQSO-MOP-001 | Mission Operations Plan |
| SQSO-ICD-006 | Space-Ground Interface |

## 3. Test Configuration

### 3.1 Equipment Required

| Equipment | Model/Spec | Cal Due |
|-----------|------------|---------|
| Transceiver | Dual-band VHF/UHF | |
| TNC | KISS mode capable | |
| Antenna system | VHF/UHF Yagi | |
| Rotator controller | Az/El | |
| RF signal generator | 145/435 MHz | |
| RF power meter | VHF/UHF | |
| Spectrum analyzer | Optional | |
| Reference GPS | Time sync | |

### 3.2 Software Required

| Software | Version | Purpose |
|----------|---------|---------|
| Mission Control SW | 1.0.0 | Operations |
| Packet decoder | Direwolf 1.6+ | AX.25 decode |
| SDR software | GQRX/SDR# | RF monitoring |
| Rotator control | GPredict/PstRotator | Antenna tracking |
| Orbit prediction | GPredict/Orbitron | Pass planning |

### 3.3 Test Configuration Diagram

```
TEST CONFIGURATION

                    ┌─────────────────┐
                    │ RF Signal       │
                    │ Generator       │
                    └────────┬────────┘
                             │ RF (attenuated)
                             ▼
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   Antenna   │◄───►│   Transceiver   │◄───►│    TNC      │
│   (or load) │     │                 │     │             │
└─────────────┘     └─────────────────┘     └──────┬──────┘
                             │                      │
                             │ Audio                │ Serial
                             ▼                      ▼
                    ┌─────────────────┐     ┌─────────────┐
                    │  SDR Monitor    │     │    MCS      │
                    │   (optional)    │     │  Computer   │
                    └─────────────────┘     └─────────────┘
```

## 4. Pre-Test

### 4.1 Pre-Test Checklist

```
PRE-TEST CHECKLIST

Date: ___________
Operator: ___________
Witness: ___________

EQUIPMENT STATUS:
[ ] Transceiver powered and configured
[ ] TNC connected and KISS mode enabled
[ ] Antenna connected or dummy load attached
[ ] Rotator controller connected
[ ] Signal generator available
[ ] All software launched

DOCUMENTATION:
[ ] Test procedure printed/available
[ ] As-run log ready
[ ] Equipment log available

SAFETY:
[ ] RF safety zone established
[ ] TX inhibit verified if using antenna

Ready for test: [ ] YES  [ ] NO

Signature: ___________ Time: ___________
```

## 5. Test Procedures

### 5.1 Test 1: Receiver Verification

**Test ID**: GS-001-RX
**Duration**: 30 minutes
**Purpose**: Verify VHF receiver performance

```
RECEIVER VERIFICATION

Step 1: Configure signal generator
  Frequency: 145.XXX MHz (mission downlink)
  Modulation: FM, 3.5 kHz deviation
  Level: -100 dBm

  Action: Connect to antenna port via attenuator
  Expected: Signal visible on S-meter
  Actual: ___________________________
  Pass: [ ]

Step 2: Verify frequency display
  Action: Note receiver frequency display
  Expected: 145.XXX MHz ± 100 Hz
  Actual: ___________________________
  Pass: [ ]

Step 3: Audio output verification
  Action: Listen to receiver audio
  Expected: Clear FM audio (or tone if modulated)
  Actual: ___________________________
  Pass: [ ]

Step 4: Sensitivity check
  Action: Reduce signal to -115 dBm
  Expected: Still audible, marginal
  Actual: ___________________________
  Pass: [ ]

Step 5: Adjacent channel rejection
  Action: Set generator to 145.XXX ± 25 kHz
  Expected: No interference to main channel
  Actual: ___________________________
  Pass: [ ]

RESULT: [ ] PASS  [ ] FAIL
```

### 5.2 Test 2: Packet Decoder Verification

**Test ID**: GS-001-PKT
**Duration**: 30 minutes
**Purpose**: Verify AX.25 packet decoding

```
PACKET DECODER VERIFICATION

Step 1: Configure signal generator
  Frequency: 145.XXX MHz
  Modulation: AFSK 1200 baud
  Data: Pre-recorded AX.25 frame
  Level: -90 dBm

  Action: Transmit test frame
  Expected: Frame decoded by TNC
  Actual: ___________________________
  Pass: [ ]

Step 2: Callsign verification
  Action: Check decoded callsign
  Expected: Matches transmitted callsign
  Actual: ___________________________
  Pass: [ ]

Step 3: Data integrity
  Action: Compare decoded data to source
  Expected: Byte-for-byte match
  Actual: ___________________________
  Pass: [ ]

Step 4: Weak signal decode
  Action: Reduce signal to -110 dBm
  Expected: Still decoding (may have errors)
  Actual: ___________________________
  Pass: [ ]

Step 5: Multiple frame test
  Action: Transmit 10 consecutive frames
  Expected: ≥90% decode success
  Decoded: ___ / 10
  Pass: [ ]

RESULT: [ ] PASS  [ ] FAIL
```

### 5.3 Test 3: Telemetry Processing

**Test ID**: GS-001-TLM
**Duration**: 30 minutes
**Purpose**: Verify telemetry extraction and display

```
TELEMETRY PROCESSING VERIFICATION

Step 1: Inject test telemetry
  Action: Send beacon with known telemetry values
  Expected: MCS receives and parses frame
  Actual: ___________________________
  Pass: [ ]

Step 2: Verify parameter extraction
  Test values:
  - MODE = 1 (IDLE)
  - BATT_V = 7.850 V
  - SOC = 75%
  - TEMP = 25.0°C

  MCS displays:
  - MODE = ___
  - BATT_V = ___ V
  - SOC = ___%
  - TEMP = ___°C
  Pass: [ ]

Step 3: Limit checking
  Action: Send out-of-limit value (SOC = 15%)
  Expected: Yellow/Red alarm displayed
  Actual: ___________________________
  Pass: [ ]

Step 4: Database storage
  Action: Query database for received frame
  Expected: Frame stored with timestamp
  Actual: ___________________________
  Pass: [ ]

Step 5: Display update
  Action: Send updated telemetry
  Expected: Display updates in <2 seconds
  Actual: ___________________________
  Pass: [ ]

RESULT: [ ] PASS  [ ] FAIL
```

### 5.4 Test 4: Transmitter Verification

**Test ID**: GS-001-TX
**Duration**: 30 minutes
**Purpose**: Verify UHF transmitter performance

**WARNING: Use dummy load or authorized transmissions only**

```
TRANSMITTER VERIFICATION

Step 1: Power output measurement
  Action: Key transmitter into power meter (dummy load)
  Expected: Rated power ± 10%
  Actual: ___ W
  Pass: [ ]

Step 2: Frequency accuracy
  Action: Measure carrier frequency
  Expected: 435.XXX MHz ± 500 Hz
  Actual: ___________________________
  Pass: [ ]

Step 3: Modulation test
  Action: Transmit AFSK test pattern
  Expected: Clean modulation on analyzer
  Actual: ___________________________
  Pass: [ ]

Step 4: PTT response
  Action: Time PTT to RF output
  Expected: <100 ms
  Actual: ___ ms
  Pass: [ ]

RESULT: [ ] PASS  [ ] FAIL
```

### 5.5 Test 5: Antenna System Verification

**Test ID**: GS-001-ANT
**Duration**: 45 minutes
**Purpose**: Verify antenna pointing and tracking

```
ANTENNA SYSTEM VERIFICATION

Step 1: Manual positioning
  Action: Command antenna to Az=0°, El=0°
  Expected: Antenna moves to position
  Actual Az: ___° El: ___°
  Pass: [ ]

Step 2: Full range test
  Positions to test:
  [ ] Az=0°, El=0° (stow)
  [ ] Az=90°, El=45°
  [ ] Az=180°, El=90° (zenith)
  [ ] Az=270°, El=45°
  [ ] Az=359°, El=0°
  Pass: [ ]

Step 3: Tracking test
  Action: Load test satellite TLE
  Expected: Antenna tracks predicted path
  Actual: ___________________________
  Pass: [ ]

Step 4: Position feedback
  Action: Compare commanded vs reported
  Expected: Within ±2°
  Actual: ___________________________
  Pass: [ ]

Step 5: Speed test
  Action: Command 90° azimuth slew
  Expected: Completes in <60 seconds
  Actual: ___ seconds
  Pass: [ ]

RESULT: [ ] PASS  [ ] FAIL
```

### 5.6 Test 6: End-to-End Validation

**Test ID**: GS-001-E2E
**Duration**: 60 minutes
**Purpose**: Complete end-to-end ground station test

```
END-TO-END VALIDATION

Step 1: Simulate pass
  Action: Load SMART-QSO TLE (simulated)
  Expected: Pass prediction displayed
  AOS: ___________
  TCA: ___________
  LOS: ___________
  Pass: [ ]

Step 2: Pre-pass preparation
  Action: Execute pre-pass checklist
  Expected: All items complete
  Actual: ___________________________
  Pass: [ ]

Step 3: AOS activities
  Action: Inject beacon at predicted AOS
  Expected: Beacon decoded, TLM extracted
  Actual: ___________________________
  Pass: [ ]

Step 4: Tracking during pass
  Action: Antenna tracks as pass progresses
  Expected: Continuous signal reception
  Actual: ___________________________
  Pass: [ ]

Step 5: Command uplink (simulated)
  Action: Generate and transmit test command
  Expected: Command formatted correctly
  Actual: ___________________________
  Pass: [ ]

Step 6: LOS activities
  Action: Complete pass, stow antenna
  Expected: Data logged, antenna stowed
  Actual: ___________________________
  Pass: [ ]

Step 7: Post-pass data review
  Action: Review logged telemetry
  Expected: All frames logged with times
  Frames received: ___
  Pass: [ ]

RESULT: [ ] PASS  [ ] FAIL
```

## 6. Test Results Summary

```
TEST RESULTS SUMMARY

Test ID       Description              Result    Notes
─────────────────────────────────────────────────────────
GS-001-RX     Receiver Verification    [ ]
GS-001-PKT    Packet Decoder           [ ]
GS-001-TLM    Telemetry Processing     [ ]
GS-001-TX     Transmitter              [ ]
GS-001-ANT    Antenna System           [ ]
GS-001-E2E    End-to-End               [ ]

OVERALL RESULT: [ ] PASS  [ ] FAIL  [ ] PARTIAL

Anomalies/Issues:
_________________________________________________
_________________________________________________

Recommendations:
_________________________________________________
_________________________________________________

Test Conductor: _____________ Date: _______
Witness: _____________ Date: _______
QA: _____________ Date: _______
```

## 7. Troubleshooting

| Symptom | Possible Cause | Resolution |
|---------|----------------|------------|
| No RX signal | Antenna disconnect | Check cables |
| Decode failures | Audio level | Adjust levels |
| TLM parse errors | Software config | Check format |
| Antenna won't move | Rotator power | Check supply |
| Database errors | Connection | Restart DB |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
