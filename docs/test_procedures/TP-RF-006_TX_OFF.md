# Test Procedure: Telecommand TX-OFF at the Antenna Port
## TP-RF-006

**Document Number:** TP-RF-006
**Revision:** Draft
**Date:** 2026-10-15
**Classification:** Development/Qualification Test (hardware-in-the-loop)
**Status:** Written; not yet run. Needs EM or FM hardware, the target receiver driver and the FRAM driver (SQSO-ICD-013 §10).

---

## 1. Purpose

Verify on hardware that an authenticated telecommand stops every emission at the antenna port, that the stop survives reset and safe mode, and that only authenticated commands can restart transmission. This verifies SYS-204, SYS-205, SYS-224 and SYS-325 and supports the cessation-of-emissions statements in the FCC pre-space notification (item 6 of its checklist) and in IARU form item 7.8.

## 2. Scope

The test is conducted only: the RF output goes into an attenuator and a load, and the uplink is injected by cable. Nothing is radiated. The test covers:

- Command reception from the 435 MHz receiver through the AX.25 receive path
- TX-OFF latch, PA power removal through `RF_TX_EN`, persistence across a power cycle
- Safe mode and EPS beacon enables cannot override TX-OFF
- Replayed, forged and unauthenticated commands rejected
- TX-OFF without any key material (key-loss rule)
- Loss-of-contact inhibit (SYS-205) and its release by an authenticated command
- Post-deployment 45-minute RF silence (SYS-224), including a mid-countdown watchdog reset and a power toggle
- Key rotation with the recovery key

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-ICD-013 | Telecommand Authentication and TX-OFF (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`) |
| SQSO-ICD-005 | OBC to RF Interface (`docs/interfaces/ICD_OBC_RF.md`) |
| SYS-204, SYS-325 | System requirements (`docs/requirements/SYSTEM_REQUIREMENTS.md`) |
| 47 CFR 97.207(b), ITU RR 22.1 | Cessation of emissions by telecommand |

## 4. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | OBC + RF board (transmitter, 435 MHz receiver, PA load switch) |
| Serial Number | _____________ |
| Flight software | Version/commit: _____________ (must include `uplink.c`, `rf_control.c`, `cmd_auth.c`, `gse_provision.c`) |
| Configuration | GSE umbilical connected (sense line high); flight harness otherwise |

## 5. Test Equipment

| Equipment | Specification | Cal Due |
|-----------|---------------|---------|
| Spectrum analyzer | 9 kHz–3 GHz; RBW 1 kHz; noise floor ≤ −110 dBm | _______ |
| Attenuator (TX path) | 30 dB, ≥ 2 W | _______ |
| 50 Ω load | ≥ 5 W | _______ |
| Uplink source | 435 MHz FM transmitter or signal generator with 1200 bps AFSK, through ≥ 60 dB attenuation into the receiver port | _______ |
| KISS TNC | 1200 bps AFSK | _______ |
| Ground computer | `software/ground/tools/telecommand_builder.py`, `key_provision.py` | n/a |
| Oscilloscope | Probe on `RF_TX_EN` and PA supply | _______ |
| GSE umbilical | UART + sense line | n/a |

## 6. Preconditions

- [ ] TP-RF-001 passed on this unit (transmitter works)
- [ ] TEST key set generated for this test only: `key_provision.py generate --dir tp-rf-006-keys` (never flight keys)
- [ ] TEST keys loaded through the umbilical (`frames`, check every reply is `SQGR 00`)
- [ ] Separate counter files for the TEST operational and recovery keys, starting empty
- [ ] Beacon interval set to 10 s (SET_BEACON) so emissions are frequent

## 7. Safety

- **RF:** never transmit without the attenuator and load; up to 1 W output
- **Uplink isolation:** no radiated uplink; check the cable path's attenuation before keying the source

## 8. Procedure

Each command below is built with `telecommand_builder.py <COMMAND> --key-file <key> --counter-file <counter> --ax25 <SAT-CALL> <GROUND-CALL> --kiss`. The result is sent through the TNC and uplink source. "Silent" means the following, measured on the spectrum analyzer through the 30 dB attenuator, with RBW 1 kHz and max hold over the stated period:

- no signal above the analyzer noise floor at the downlink frequency;
- no signal above the noise floor at its 2nd and 3rd harmonics.

### 8.1 Baseline

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 1 | Power on; wait for two beacons | Beacons at the downlink frequency every 10 s; `RF_TX_EN` high | |
| 2 | Send TX_ENABLE (operational key); **save the transmitted frame** for step 6 | Accepted (no change: already enabled) | |

### 8.2 TX-OFF

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 3 | Send TX_DISABLE (operational key); start a timer at the end of the uplink frame | `RF_TX_EN` low and PA supply off within **1 s (TBR)**; silent for 120 s | |
| 4 | Force SAFE (SET_MODE 0) | Accepted; still silent for 120 s | |
| 5 | Remove and restore OBC power | After boot, `RF_TX_EN` stays low from power-on (oscilloscope); silent for 120 s | |
| 6 | Re-send the TX_ENABLE frame saved at step 2, unchanged (replay) | Rejected; silent for 60 s | |
| 7 | Send TX_ENABLE with a wrong key | Rejected; silent for 60 s | |
| 8 | Send TX_ENABLE (operational key, fresh counter) | Beacons resume within one interval | |

### 8.3 TX-OFF with the recovery key

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 9 | Send TX_DISABLE authenticated by the **recovery** key | Silent for 60 s | |
| 10 | Send TX_ENABLE authenticated by the recovery key | Rejected (operational key only); still silent | |
| 11 | Send TX_ENABLE (operational key) | Beacons resume | |

### 8.4 Key rotation

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 12 | Send KEY_ROTATE 1 authenticated by the operational key | Rejected | |
| 13 | Send KEY_ROTATE 1 authenticated by the recovery key | Accepted | |
| 14 | Send GET_STATUS with the old operational key | Rejected | |
| 15 | Send TX_DISABLE with the slot-1 key (new counter file) | Silent for 60 s | |
| 16 | Send TX_ENABLE with the slot-1 key | Beacons resume | |

### 8.5 TX-OFF without key material

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 17 | Send ZEROIZE through the umbilical (`key_provision.py zeroize`) | Reply `SQGR 00` | |
| 18 | Send TX_ENABLE built with any key | Rejected | |
| 19 | Send TX_DISABLE built with any key | Silent for 120 s (accepted without authentication) | |
| 20 | Power cycle | Still silent | |
| 21 | Re-provision the TEST key set through the umbilical; send TX_ENABLE | Beacons resume | |

### 8.6 Umbilical gate

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 22 | Disconnect the umbilical sense line; send a PROVISION frame on the UART | No reply; key table unchanged (step 23 still works with the old key) | |
| 23 | Send GET_STATUS with the current key | Accepted | |

### 8.7 Loss-of-contact inhibit

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 23a | Send SET_CONTACT_TIMEOUT 1 day (operational key); send no further commands | Accepted; beacons continue | |
| 23b | Wait 24 h (monitor continuously) | Silent from the 24 h mark, within one beacon interval; `RF_TX_EN` low | |
| 23c | Power cycle while silent | Still silent from power-on | |
| 23d | Replay any earlier frame | Rejected; still silent | |
| 23e | Send NOP (operational key) | Beacons resume within one interval | |
| 23f | Send SET_CONTACT_TIMEOUT back to the configured value | Accepted | |

### 8.8 Post-deployment RF silence (SYS-224)

Monitor the downlink frequency and its harmonics, and the receiver's local-oscillator frequency, continuously (max hold).

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 23g | Send `arm-flight` through the umbilical; power off; disconnect the umbilical | Reply `SQGR 00` before power-off | |
| 23h | Apply power with the deployment switches released (simulated deployment); start a timer | Boot log `state=COUNTING elapsed=0s`; radio and transmitter unpowered; no signal | |
| 23i | At 30 min, force a watchdog reset (debug command or by withholding the watchdog kick) | Silent; boot log shows the count resumed (about 1,800 s) | |
| 23j | At 40 min, remove and restore power (switch toggle) | Silent; count restarts at 0 | |
| 23k | Send TX_ENABLE (operational key) during the silence | No effect while the radio is off; still silent | |
| 23l | Continue to 45 min after step 23j | First beacon no earlier than 45:00 after 23j | |
| 23m | Re-arm (step 23g) before any further integration activity | `SQGR 00` | |

### 8.9 Completion

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 24 | Reconnect the umbilical; send ZEROIZE | Reply `SQGR 00`; TEST keys removed from the unit | |
| 25 | Delete the TEST key set and frame files from the ground computer | Done | |

## 9. Pass/Fail Criteria

**Pass:** every step meets its expected result. In particular:

- Every TX-OFF step is silent within 1 s (TBR) and for the whole observation period.
- No unauthenticated, replayed or wrong-key command restarts transmission.
- TX-OFF survives a power cycle.

**Fail:** any emission while TX-OFF is latched, or any command accepted contrary to the table.

## 10. Data Recording

- Spectrum analyzer max-hold captures for each silent period
- Oscilloscope captures of `RF_TX_EN` and PA supply at steps 3 and 5
- Ground log of every frame sent (hex) and its counter
- Uplink and command statistics before and after the test

## 11. Post-Test

- [ ] TEST keys zeroized on the unit and deleted on the ground
- [ ] Data archived; results entered against SYS-204 and SYS-325 in the verification matrix
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
| Draft | 2026-10-15 | Initial procedure for SYS-204 / SYS-325 hardware verification | Funding audit |
