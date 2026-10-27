# SMART-QSO Ground Station Operations Manual

**Document ID**: SQSO-GSOM-001
**Version**: 1.5
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR draft for the designated telecommand stations (none agreed yet; `docs/ground/GROUND_STATION_DESIGN.md` Rev F). Frequencies are planning values until the IARU letter. Command procedures follow SQSO-ICD-013, which governs.

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
| UHF Antenna | Variant A: M2 436CP30 class, ≥ 14 dBic; variant B (US270 sites): stacked pair, ≈ 18 dBic, same circular sense on both | 435 MHz transmit; **≥ 64 dBm EIRP toward the spacecraft** (decision C-14) |
| UHF amplifier and sequencer | Variant A: class AB solid-state, ≥ 150 W FM, 100 W PEP at the feed; variant B: none, the transceiver at **≤ 50 W PEP** (US270); both: hardware PTT gate (no keying below 10°) | SQSO-GND-001 Rev F §3.1, §3.1a |
| Rotator accuracy | ±5° (variant A); **±2°** for the variant B array (calibrate against the Sun or a beacon before each campaign) | 0.3 dB pointing allowance (SQSO-LBA-001 v3.4 §6.1) |
| Rotator | Yaesu G-5500 | Antenna pointing |
| LNA | SSB SP-2000 | Low noise amplifier |
| Radio | Icom IC-9700 class, satellite (full-duplex) mode | Transceiver |
| Offline key machine | Laptop without network, or hardware token | Holds the operational key; builds command frames (SQSO-ICD-013 §9) |
| Time reference | GNSS or NTP checked against GNSS | ±1 s (UTC-derived counters, SQSO-ICD-013 §5.2.1) |
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
     □ Set to the coordinated downlink (planning value 145.900 MHz, FM wide, 25 kHz filter)
     □ Enable Doppler correction in Gpredict (±3.5 kHz over the pass)
     □ Verify radio follows Doppler commands

□ 9. Configure transmit (if commanding)
     □ Set to the coordinated uplink (planning value 435.500 MHz), Doppler pre-correction ON (±10.4 kHz over the
       pass; the uplink must be within ±1 kHz at the spacecraft or nothing decodes, SQSO-LBA-001 §4.4)
     □ Verify the power at the antenna feed on the wattmeter: variant A 100 W ± 0.5 dB; variant B (US270 site) ≤ 50 W PEP,
       never more; deviation ±3.0 kHz (VI-GND-01)
     □ Verify the sequencer's PTT gate refuses to key below 10° (rotator parked at 0°: no keying)
     □ Verify PTT interface connected
     □ Verify station clock against GNSS (±1 s)
     □ Command frames for this pass built on the offline key machine, counters reserved (SQSO-ICD-013 §5.2)

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
     □ Verify callsign "KK6GDB-11" in beacon
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
- [ ] The command on the pass plan approved by the licensee (or a standing plan, for routine `NOP`/`GET_STATUS`)
- [ ] A licensed control operator at the station (97.211(a))
- [ ] Frames built from the command dictionary (SQSO-ICD-013 §7) with `telecommand_builder.py`, each with a new counter
- [ ] For `KEY_ROTATE` or passivation: the recovery-key custodian present (two-person rule)

### 7.2 Command Procedure

```
COMMAND EXECUTION PROCEDURE

1. PREPARE (offline machine, before the pass)
   □ Read the counter file for the active key (or compute UTC-derived counters, ICD-013 §5.2.1)
   □ Build each frame: telecommand_builder.py <CMD> --key-file <op.key> --counter-file <file> --kiss
   □ The builder reserves (writes and fsyncs) the counter BEFORE producing the frame
   □ Copy the frames (not the key) to the station computer

2. WAIT FOR CONFIRMATION THE SPACECRAFT IS HEARD
   □ At least one beacon decoded this pass (it confirms the spacecraft is not silent and the Doppler is right)
   □ **Elevation above 10°** (decision C-14: commands are sent only above 10°, with 100 W at the feed; +8.3 dB
     worst-case margin at 10°, SQSO-LBA-001 v3.3 §6.1). **Degraded mode** (amplifier out of service, or 50 W PEP with
     the 14 dBic antenna): above 16° only. Never key below 10°, even for TX_DISABLE: wait for the elevation

3. TRANSMIT
   □ Send one frame through the KISS port (Dire Wolf keys the radio)
   □ Note the counter used

4. VERIFY
   □ Wait for the next beacon (<= 60 s in ACTIVE): check the field the command changes (M=, RF=)
   □ There is no command acknowledgement frame (SQSO-ICD-005 v2.0 §5.5). If no change is seen and
     the command is safe to repeat, rebuild it with a NEW counter and send again (up to 4 attempts)
   □ NEVER re-send a frame that has already been sent (its counter is spent or will be rejected)

5. DOCUMENT
   □ Log: UTC, command, counter, key ID, attempts, effect seen
   □ Upload the log and the frames heard to the mission database
```

### 7.3 Command List Quick Reference

From SQSO-ICD-013 §7 (the dictionary `software/flight/command_dictionary.json` governs):

| Command | ID | Args | Key | Effect seen in the beacon |
|---------|----|------|-----|---------------------------|
| NOP | 0x00 | — | operational | None (counts as contact for the loss-of-contact timer) |
| SET_MODE | 0x02 | 1 byte: 0 SAFE, 1 IDLE, 2 ACTIVE | operational | `M=`; SAFE → ACTIVE needs IDLE first |
| GET_STATUS | 0x03 | — | operational | None on the downlink today (response discarded) |
| SET_CONTACT_TIMEOUT | 0x07 | 2 bytes, days 1–90 | operational | None; set 14 days at the end of commissioning |
| SET_BEACON | 0x30 | 2 bytes, seconds 10–120 | operational | Beacon interval |
| TX_ENABLE | 0x31 | — | operational | Beacons resume (`RF=1`) |
| **TX_DISABLE** | **0x32** | — | **any key; none if no key is usable** | **All emissions stop** |
| SET_POWER | 0x33 | 1 byte: 0 = 0.5 W, 1 = 1 W | operational | Signal level |
| KEY_ROTATE | 0x06 | 1 byte, spare slot 0–3 | **recovery** | None (next command must use the new key, counter from 0) |
| PLD_ENABLE / PLD_DISABLE | 0x40 / 0x41 | — | operational | AI text in the beacon |

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
- Wait 3 minutes (beacon every 60 s in ACTIVE, 120 s IDLE, 180 s SAFE)
- Check Doppler: the downlink is within ±3.5 kHz of nominal; search ±5 kHz
- Check the loss-of-contact state: if no authenticated contact for 14 days the spacecraft is silent but still
  listening: send an authenticated NOP
- The spin (8°/s, BASELINE §8) puts the antenna null on the station for about 5 s (95th percentile 6 s) every 22.5 s: expect gaps
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
- Check the uplink Doppler correction (must be within ±1 kHz at the spacecraft)
- Wait for higher elevation (≥ 10°; ≥ 16° in the degraded mode); variant A 100 W at the feed, variant B ≤ 50 W PEP (US270)
- Rebuild with a new counter; never re-send a spent frame
- If AUTH_FAIL is suspected (wrong key slot after a rotation, counter behind): check the counter file and key ID
```

## 9. Emergency Procedures

### 9.1 Loss of Communication

See CP-001 in Contingency Procedures document.

### 9.1a Order to Cease Transmissions (FCC, 97.207(b))

1. The licensee informs both telecommand stations at once (24-hour contact).
2. On the next pass of either station: build `TX_DISABLE` with the next counter; if no key is usable, send the keyless
   `TX_DISABLE` (SQSO-ICD-013 §6). Repeat each pass (new counters) until no beacon is heard on two passes.
3. Longest wait for a commanding pass: 13.0 h with one station, 9.9 h with two (GROUND_STATION_DESIGN Rev D §5).
4. The licensee files the post-space notification within 24 hours of the termination (97.207(g)(3)).

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
│ Mode: FM / 1200 baud AFSK, +/-3 kHz     │
│ Beacon: 60 s ACTIVE, 120 IDLE, 180 SAFE │
│ Transponder: CTCSS 67.0 Hz              │
│ Doppler: 2 m +/-3.5 kHz, 70 cm +/-10.4  │
│ Callsign: KK6GDB-11                    │
└─────────────────────────────────────────┘
```

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | CDR review (communications panel) | Equipment (offline key machine, time reference, 50 W PEP limit); radio set-up with Doppler rules; command procedure rewritten for authenticated commands and counters; quick reference from the command dictionary; troubleshooting for Doppler, spin fades and the loss-of-contact inhibit; FCC cease-transmission procedure; frequency card |
| 1.2 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: spin 8°/s and its fade period (BASELINE Rev E §8) |
| 1.3 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Fade length from SQSO-LBA-001 v3.2 §5.2 (5 s every 22.5 s at 8°/s); TX-OFF wait 13.0 h from SQSO-GND-001 Rev D §5 (475 km) |
| 1.4 | 2026-10-27 | CDR S&MA analysis (decision C-14) | Commanding only above 10° elevation (16° in the degraded mode at 50 W); 100 W PEP at the antenna feed with the amplifier and sequencer, PTT gate check; US270 sites (§2.2, §5, troubleshooting) |
| 1.5 | 2026-10-27 | CDR S&MA analysis (decision C-14 restated) | Station variants A (100 W into ≥ 14 dBic) and B (≤ 50 W PEP into a stacked pair at US270 sites), rotator accuracy ±2° for B, power checks |
