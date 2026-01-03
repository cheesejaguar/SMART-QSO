# SMART-QSO Ground Station Operations Manual

**Document ID**: SQSO-GSOM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This manual provides step-by-step procedures for operating the SMART-QSO ground station. It covers routine pass operations, equipment setup, and troubleshooting.

## 2. Ground Station Overview

### 2.1 System Block Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    GROUND STATION SYSTEM                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │ Antenna │───▶│   LNA   │───▶│  Radio  │───▶│   TNC   │      │
│  │  (Yagi) │    │ (+20dB) │    │(IC-9700)│    │(Direwolf│      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│       │                             │              │            │
│       │                             │              │            │
│  ┌─────────┐                   ┌─────────┐    ┌─────────┐      │
│  │ Rotator │◀──────────────────│ Gpredict│───▶│ Decoder │      │
│  │Controller                   │         │    │ Software│      │
│  └─────────┘                   └─────────┘    └─────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Equipment List

| Equipment | Model | Purpose |
|-----------|-------|---------|
| VHF Antenna | M2 2MCP22 | 145 MHz receive |
| UHF Antenna | M2 436CP30 | 435 MHz transmit |
| Rotator | Yaesu G-5500 | Antenna pointing |
| LNA | SSB SP-2000 | Low noise amplifier |
| Radio | Icom IC-9700 | Transceiver |
| TNC | Direwolf (software) | Packet modem |
| Computer | Linux workstation | Control & decode |

### 2.3 Software Components

| Software | Version | Purpose |
|----------|---------|---------|
| Gpredict | 2.2.1+ | Tracking & Doppler |
| Direwolf | 1.7+ | TNC/KISS interface |
| smart-qso-ground | 1.0.0 | Beacon decoder |
| Hamlib | 4.5+ | Radio control |

## 3. Pre-Pass Checklist

### 3.1 T-60 Minutes: System Power-Up

```
□ 1. Power on ground station equipment
     □ Computer
     □ Radio (IC-9700)
     □ Rotator controller
     □ LNA power supply

□ 2. Verify equipment status
     □ Radio displays nominal frequency
     □ Rotator responds to commands
     □ Computer network connected
```

### 3.2 T-30 Minutes: Software Initialization

```
□ 3. Start tracking software
     $ gpredict &
     □ Verify TLE is current (< 7 days old)
     □ Verify SMART-QSO visible in satellite list
     □ Confirm pass prediction matches schedule

□ 4. Start TNC software
     $ direwolf -c direwolf.conf &
     □ Verify audio device connected
     □ Verify KISS TCP port open (8001)

□ 5. Start decoder software
     $ cd smart-qso-ground
     $ python beacon_decoder.py --kiss localhost:8001 &
     □ Verify decoder running
     □ Verify log file created
```

### 3.3 T-15 Minutes: Antenna Positioning

```
□ 6. Command antenna to AOS position
     □ Set azimuth to AOS azimuth
     □ Set elevation to 0° (horizon)
     □ Verify rotator movement

□ 7. Verify antenna tracking enabled
     □ Gpredict → Rotator → Engage
     □ Confirm tracking mode active
```

### 3.4 T-5 Minutes: Radio Configuration

```
□ 8. Configure receive frequency
     □ Set to 145.XXX MHz (downlink)
     □ Enable Doppler correction in Gpredict
     □ Verify radio follows Doppler commands

□ 9. Configure transmit (if commanding)
     □ Set to 435.XXX MHz (uplink)
     □ Verify TX power setting (50W)
     □ Verify PTT interface connected

□ 10. Final verification
      □ Audio levels nominal
      □ Decoder showing "Waiting for data"
      □ Log file open
```

## 4. Pass Operations

### 4.1 Acquisition of Signal (AOS)

```
At AOS (elevation > 0°):

□ 1. Monitor for beacon
     □ First beacon expected within 60 seconds
     □ Watch decoder output for frame detection
     □ Note signal strength (S-meter)

□ 2. Confirm spacecraft identity
     □ Verify callsign "SQSO-1" in beacon
     □ Verify telemetry values reasonable
     □ Log AOS time and signal strength

□ 3. Begin telemetry recording
     □ Confirm decoder logging active
     □ Note any anomalies
```

### 4.2 During Pass

```
□ 4. Monitor telemetry
     □ Watch for beacon every 60 seconds
     □ Verify telemetry values in limits
     □ Flag any out-of-limit values

□ 5. If commanding (pre-authorized only):
     □ Verify two operators present
     □ Execute commands per procedure
     □ Verify command acknowledgment
     □ Log all commands sent

□ 6. Track signal quality
     □ Note signal strength variations
     □ Record maximum elevation signal
     □ Note any dropouts
```

### 4.3 Loss of Signal (LOS)

```
At LOS (elevation < 0°):

□ 7. Record LOS
     □ Log LOS time
     □ Record last telemetry values
     □ Note total beacons received

□ 8. End active tracking
     □ Disable Doppler correction
     □ Park antenna (optional)

□ 9. Secure commanding equipment
     □ Disable transmit
     □ Return radio to receive-only
```

## 5. Post-Pass Procedures

### 5.1 Data Processing

```
□ 1. Stop decoder
     □ Ctrl-C to stop beacon_decoder.py
     □ Verify log file saved

□ 2. Archive data
     $ cp beacon_log_YYYYMMDD_HHMMSS.json \
         ~/smart-qso/passes/

□ 3. Process telemetry
     $ python telemetry_display.py \
         --input beacon_log_YYYYMMDD_HHMMSS.json \
         --output pass_report.html

□ 4. Review telemetry
     □ Check all values in limits
     □ Note any anomalies
     □ Compare to previous pass
```

### 5.2 Pass Report

```
□ 5. Complete pass log entry

     PASS LOG ENTRY
     ─────────────────────────────────
     Date: ____________
     Pass #: ___
     Operator: ____________

     AOS: _______ UTC  Az: ___°
     LOS: _______ UTC  Az: ___°
     Max El: ___°
     Duration: ___ min

     Beacons received: ___
     Commands sent: ___
     Anomalies: ____________

     Notes:
     ________________________
     ________________________

□ 6. Submit pass report
     □ Email to mission ops team
     □ Upload to mission database
```

## 6. Telemetry Monitoring

### 6.1 Real-Time Display

The telemetry display shows current spacecraft health:

```
┌─────────────────────────────────────────────────────────────┐
│              SMART-QSO TELEMETRY DISPLAY                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  SPACECRAFT STATE: ACTIVE          TIME: 2026-01-02 12:34Z  │
│                                                              │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐  │
│  │   BATTERY   │   THERMAL   │     RF      │    ADCS     │  │
│  ├─────────────┼─────────────┼─────────────┼─────────────┤  │
│  │ V: 3.85V    │ OBC: 25°C   │ TX: ON      │ SUN: YES    │  │
│  │ I: +0.3A    │ BAT: 22°C   │ PWR: 1.0W   │ TUMBLE: NO  │  │
│  │ SOC: 78%    │ JET: 45°C   │ RSSI: -85   │             │  │
│  └─────────────┴─────────────┴─────────────┴─────────────┘  │
│                                                              │
│  LAST BEACON: 12:34:05Z  NEXT: ~12:35:05Z                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Limit Checking

| Parameter | Low Limit | High Limit | Action if Exceeded |
|-----------|-----------|------------|-------------------|
| BATT_V | 3.0V | 4.2V | Flag, notify |
| BATT_SOC | 20% | 100% | Flag, notify if <30% |
| OBC_TEMP | -40°C | +85°C | Flag, notify |
| BAT_TEMP | -10°C | +45°C | Flag, notify |
| JET_TEMP | -20°C | +70°C | Flag, notify |

### 6.3 Anomaly Indicators

| Indicator | Meaning | Action |
|-----------|---------|--------|
| 🔴 RED | Critical - Out of limits | Immediate report |
| 🟡 YELLOW | Warning - Near limits | Monitor closely |
| 🟢 GREEN | Nominal | Continue monitoring |
| ⚪ GRAY | Stale - No recent data | Check link |

## 7. Command Operations

### 7.1 Command Authorization

**ALL COMMANDS REQUIRE**:
- [ ] Written authorization from Mission Director
- [ ] Two certified operators on console
- [ ] Command procedure reviewed
- [ ] Backup plan documented

### 7.2 Command Procedure

```
COMMAND EXECUTION PROCEDURE

1. VERIFY authorization
   □ Command in authorized list
   □ Two operators present
   □ Pass duration sufficient

2. PREPARE command
   □ Open command interface
   □ Enter command
   □ Operator 1: Read back command
   □ Operator 2: Confirm command

3. TRANSMIT
   □ Wait for clear beacon gap
   □ Operator 2: "Ready to transmit"
   □ Operator 1: "Transmitting [command]"
   □ Press PTT and send
   □ Release PTT

4. VERIFY
   □ Wait for next beacon (≤60s)
   □ Verify command effect in telemetry
   □ Log result

5. DOCUMENT
   □ Record command in log
   □ Record spacecraft response
   □ Note any anomalies
```

### 7.3 Command List Quick Reference

| Command | Effect | Verification |
|---------|--------|--------------|
| SAFE_MODE | Enter safe mode | STATE=SAFE in TLM |
| RELEASE_SAFE | Exit safe mode | STATE=IDLE/ACTIVE |
| BEACON_RATE n | Set beacon rate | Beacon interval changes |
| TX_POWER n | Set TX power | TX_PWR in TLM |

## 8. Troubleshooting

### 8.1 No Signal at AOS

```
SYMPTOM: Expected beacon not received at AOS

CHECK:
1. □ TLE current? May need update
2. □ Antenna pointed correctly?
3. □ Radio on correct frequency?
4. □ LNA powered and connected?
5. □ Audio path to TNC working?

IF ALL OK:
- Wait 2 minutes (beacon every 60s)
- Try wider frequency search (±10 kHz)
- Check for spacecraft safe mode (lower power)
```

### 8.2 Weak Signal

```
SYMPTOM: Signal received but weak/noisy

CHECK:
1. □ LNA gain setting
2. □ Antenna connections tight
3. □ Coax cable condition
4. □ Local interference (check waterfall)

ADJUST:
- Increase audio gain
- Enable noise reduction
- Check antenna polarization
```

### 8.3 Decoder Not Decoding

```
SYMPTOM: Signal visible but no decoded frames

CHECK:
1. □ Direwolf running?
2. □ Audio levels correct?
3. □ KISS port connected?
4. □ Decoder receiving data?

FIX:
- Restart Direwolf
- Adjust audio levels (peak at 50%)
- Check decoder logs for errors
```

### 8.4 Command Not Acknowledged

```
SYMPTOM: Command sent but no response

CHECK:
1. □ TX power sufficient?
2. □ Antenna switched to TX?
3. □ Frequency correct (with Doppler)?
4. □ Spacecraft in commandable mode?

RETRY:
- Wait for higher elevation
- Increase TX power if available
- Verify spacecraft not in safe mode
```

## 9. Emergency Procedures

### 9.1 Loss of Communication

See CP-001 in Contingency Procedures document.

### 9.2 Spacecraft Emergency

If telemetry indicates emergency:
1. DO NOT COMMAND without authorization
2. Document all telemetry
3. Contact Mission Director immediately
4. Prepare for contingency commanding

### 9.3 Ground Station Emergency

| Emergency | Action |
|-----------|--------|
| Power failure | Switch to UPS, safe shutdown |
| Equipment failure | Use backup, document |
| Weather (lightning) | Disconnect antennas, standby |

## 10. Quick Reference Cards

### 10.1 Pass Checklist Card

```
┌─────────────────────────────────────────┐
│         SMART-QSO PASS CHECKLIST        │
├─────────────────────────────────────────┤
│ T-60  □ Power on equipment              │
│ T-30  □ Start Gpredict, Direwolf        │
│ T-15  □ Point antenna to AOS            │
│ T-5   □ Configure radio, start decoder  │
│ AOS   □ Monitor for beacon              │
│ PASS  □ Log telemetry, monitor health   │
│ LOS   □ Log end time, beacons received  │
│ POST  □ Archive data, submit report     │
└─────────────────────────────────────────┘
```

### 10.2 Frequency Card

```
┌─────────────────────────────────────────┐
│         SMART-QSO FREQUENCIES           │
├─────────────────────────────────────────┤
│ DOWNLINK (VHF):  145.XXX MHz            │
│ UPLINK (UHF):    435.XXX MHz            │
│                                         │
│ Mode: FM / 1200 baud AFSK              │
│ Beacon: Every 60 seconds               │
│ Callsign: SQSO-1                       │
└─────────────────────────────────────────┘
```

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
