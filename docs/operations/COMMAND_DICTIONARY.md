# SMART-QSO Command Dictionary

**Document ID**: SQSO-CMD-001
**Version**: 1.4
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft (operations reference). The flight software is implemented and tested in the host simulation; nothing has been commanded on hardware. No independent CDR has been held (BASELINE §1)

## 1. Purpose

This document defines all commands for the SMART-QSO mission. It serves as the authoritative reference for command generation and verification.

**Governing source.** The flight command set is generated from `software/flight/command_dictionary.json` and
documented, with framing and authentication, in SQSO-ICD-013 (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md` §3 to §9,
dictionary in §7). Where this document and SQSO-ICD-013 differ, SQSO-ICD-013 governs. §1.1 lists the commands the
flight software accepts; §2 to §7 summarise the frame, keys, execution and constraints with pointers to it.

### 1.1 Flight Command Set (from the dictionary, 2026-10-27)

| ID | Name | Args (bytes) | Key | Use |
|----|------|--------------|-----|-----|
| 0x00 | NOP | 0 | operational | Contact without effect (resets the loss-of-contact count) |
| 0x01 | RESET | 0 | operational | Logged only |
| 0x02 | SET_MODE | 1 | operational | 0 SAFE, 1 IDLE, 2 ACTIVE, through the transition table |
| 0x03 | GET_STATUS | 0 | operational | Status (not downlinked; use GET_HOUSEKEEPING) |
| 0x05 | CLEAR_FAULTS | 0 | operational | Clear the fault log |
| 0x06 | KEY_ROTATE | 1 | recovery | Activate a spare operational key |
| 0x07 | SET_CONTACT_TIMEOUT | 2 | operational | Loss-of-contact timeout, days (30 at launch, 14 after commissioning) |
| 0x08 | COMMISSIONING_RELEASE | 0 | operational | Release the commissioning hold |
| 0x09 | GET_HOUSEKEEPING | 0 | operational | Downlink an HK1 frame |
| 0x0A | DUMP_TLM | 7 | operational | Downlink stored housekeeping: store, first record, count ≤ 600 (0 stops) |
| 0x0B | FRAM_BYPASS | 4 | operational | `FRM!`: run without the FRAM, only once it is declared failed (HK1 `FR=1`); until the FRAM reads back or the next reset (SRS-F160) |
| 0x10 | EPS_SET_MODE | 1 | operational | EPS power mode |
| 0x13 | EPS_GET_TELEMETRY | 0 | operational | EPS telemetry (not downlinked) |
| 0x14 | EPS_CLEAR_LOCKOUT | 4 | operational | `CHG!`: clear the EPS recharge lockout (after a cell below 2.0 V) |
| 0x15 | EPS_DISABLE_TRACKERS | 1 | operational | Tracker mask (bits 0–3 = T1–T4 off, 0 all on) through the EPS controller's TRK_DISABLE (SRS-F162) |
| 0x21 | ADCS_DETUMBLE | 0 | operational | Start detumble |
| 0x23 | ADCS_GET_ATTITUDE | 0 | operational | Attitude (not downlinked) |
| 0x24 | ADCS_SET_GEOMETRY | 1 | operational | Spin-axis and inertia preset: 0 both wings, 1 W1 only, 2 W2 only, 3 none |
| 0x25 | ADCS_SET_GYRO_ALIGN | 36 | operational | Gyro alignment matrix, nine signed 32-bit elements × 10⁻⁶ (persisted) |
| 0x26 | ADCS_SET_MAG_BIAS | 13 | operational | Magnetometer bias of one rod polarity state (persisted) |
| 0x30 | SET_BEACON | 2 | operational | Beacon interval, 10 to 120 s |
| 0x31 | TX_ENABLE | 0 | operational | Clear the TX-OFF latch |
| 0x32 | TX_DISABLE | 0 | any (none if no key is usable) | Latch TX-OFF (97.207(b)) |
| 0x33 | SET_POWER | 1 | operational | 0 = 0.5 W, 1 = 1 W |
| 0x34 | XPDR_ENABLE | 0 | operational | Allow the FM transponder |
| 0x35 | XPDR_DISABLE | 0 | operational | Stop the FM transponder |
| 0x40 | PLD_ENABLE | 0 | operational | Enable the payload |
| 0x41 | PLD_DISABLE | 0 | operational | Disable the payload |
| 0x50 | DEPLOY_RETRY | 1–2 | operational | Retry a deployment element: 0 antenna, 1 wings; an optional second byte 1 forces the antenna past its temperature gate (deploys at once whatever the AntS temperature, decision C-18; refused with the wings) |
| 0x60 | GNSS_REQUEST | 0 | operational | Start a GNSS session |
| 0x70 | PASSIVATE_ARM | 4 | operational | `EOL!`: arm end-of-life passivation for 600 s |
| 0x71 | PASSIVATE_EXECUTE | 0 | recovery | Within 600 s of the arm: passivate (irreversible) |

Each command counted as contact is acknowledged on the downlink (`ack KK6GDB-11: C= R= N=`, SQSO-ICD-005 §5.5).

## 2. Command Frame Structure

Governing: SQSO-ICD-013 §3–§4 (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`). In summary:

- **Link layer:** one AX.25 v2.2 UI frame (control `0x03`, PID `0xF0`) per command on the 435 MHz uplink, addressed to
  `KK6GDB-11`, from the sending telecommand station's own call sign; FCS CRC-16/X.25 (SQSO-ICD-013 §3.1).
- **Command frame:** `0xAA 0x55 | cmd_id (1) | len (1) | payload (len) | sum16 (2, big-endian)`; 4 to 256 bytes; `sum16`
  is an additive error check, not security (§3.2).
- **Authenticated payload:** `counter (4, big-endian) | args (N ≤ 64) | tag (16)`, the tag the first 16 bytes of
  HMAC-SHA-256(key, `SQSO-TC-v1` | cmd_id | N | counter | args); one monotonic counter per key (§4, §5).
- **Ground counters:** from UTC when two or more stations command (§5.2.1, adopted), from a counter file for one station
  alone (§5.2).

The frame is authenticated, not encrypted (47 CFR 97.211(b)); the downlink stays in the clear.

## 3. Keys and Authority

The key a command needs is the `key` field of `software/flight/command_dictionary.json` (the "Key" column of §1.1),
with the key table and its custody in SQSO-ICD-013 §9:

| Key | Commands | Custody |
|-----|----------|---------|
| Operational (one active of four slots) | Every command except those below | Each designated telecommand station (BASELINE §1, §7) |
| Recovery (offline) | `KEY_ROTATE` (0x06), `PASSIVATE_EXECUTE` (0x71); also accepted for `TX_DISABLE` | A different custodian from the operational key |
| None | `TX_DISABLE` (0x32), only when no key at all is usable (97.207(b); SQSO-ICD-013 §6) | — |

Keys are loaded only through the umbilical at integration and are never in the repository (BASELINE §7; SQSO-ICD-013 §9.2).

## 4. Command Definitions

The flight command set is the 32 commands of §1.1, generated from `software/flight/command_dictionary.json` by
`scripts/gen_command_dictionary.py` (Rust `sqso-protocol/src/dictionary.rs`, C `software/flight/include/cmd_dictionary.h`,
ground `software/ground/tools/command_dictionary.py`). Argument encodings, ranges and the commands defined in
`cmd_handler.h` but refused (`SET_TIME`, the heater, ADCS-mode and payload-data IDs) are in SQSO-ICD-013 §7. The argument
length must match the dictionary exactly; otherwise the command is refused with `INVALID_PARAM` before its counter is
consumed.

*Removed in Version 1.4:* the Version 1.0 definitions (a 32-byte frame with a CRC-16 and 16-bit opcodes CMD-0100 to
CMD-FF01, software upload and bank-switch commands). None of them was implemented; Version 1.3 and earlier keep them in
the repository history.

## 5. Command Execution

The onboard processing order, the result codes (0 `SUCCESS`, 1 `INVALID_CMD`, 2 `INVALID_PARAM`, 3 `AUTH_FAIL`,
4 `NOT_ALLOWED`, 5 `EXEC_FAIL`, 6 `BUSY`) and when the counter is consumed are in SQSO-ICD-013 §8. Each command counted as
contact is acknowledged on the downlink as `ack KK6GDB-11: C=<counter> R=<result> N=<days since contact>`
(SQSO-ICD-005 §5.5); a telecommand station sends each planned command with a new counter and checks its acknowledgement
(SQSO-ICD-005 §7).

## 6. Command Constraints

| Constraint | Commands | Source |
|------------|----------|--------|
| Elevation: commands only above 10° (12° in the degraded mode) | All | Decision C-14; SQSO-ICD-005 §7 |
| Mode transitions through the state-machine table (SAFE → ACTIVE needs IDLE first, else `NOT_ALLOWED`) | `SET_MODE` (0x02) | SQSO-ICD-013 §7 |
| Allowed in every state | `TX_DISABLE`, `KEY_ROTATE`, `FRAM_BYPASS` | SQSO-ICD-013 §6, §7 |
| Fixed four-character argument against a mistaken command | `FRAM_BYPASS` `FRM!`, `EPS_CLEAR_LOCKOUT` `CHG!`, `PASSIVATE_ARM` `EOL!` | SQSO-ICD-013 §7 |
| Accepted only in a declared condition | `FRAM_BYPASS` once the FRAM is declared failed (HK1 `FR=1`) | SRS-F160 |
| Two keys, two commands, 600 s window; irreversible | `PASSIVATE_ARM` (operational) then `PASSIVATE_EXECUTE` (recovery) | SQSO-ICD-013 §6.4 |
| TX-OFF cannot be undone by a replay | `TX_ENABLE` after a later `TX_DISABLE` | SQSO-ICD-013 §6 |
| Contact timeout | `SET_CONTACT_TIMEOUT` 1–90 days (30 at launch, 14 after commissioning) | BASELINE §7 (D3); SQSO-ICD-013 §6.2 |

## 7. Quick Reference

Frames are built by `software/ground/tools/telecommand_builder.py` (SMART-QSO-GND-005), with the same Rust protocol code
the spacecraft uses (`software/rust/sqso-py`; install it first, `software/ground/tools/README.md`):

```bash
pip install ./software/rust/sqso-py
# list the dictionary
python3 software/ground/tools/telecommand_builder.py --list
# a housekeeping request from one of two or more stations (counter from UTC), KISS-encoded for a TNC
python3 software/ground/tools/telecommand_builder.py GET_HOUSEKEEPING --key-file <key file> \
    --counter-from-utc --station-config <station.json> --ax25 KK6GDB-11 <station call> --kiss --out frame.bin
```

The key file is 64 hex characters, mode 600, held by the station and never committed (SQSO-ICD-013 §9.1).

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 2) | §1: SQSO-ICD-013 and the generated dictionary govern; §1.1 the flight command set, with DUMP_TLM (0x0A), EPS_CLEAR_LOCKOUT (0x14), PASSIVATE_ARM (0x70) and PASSIVATE_EXECUTE (0x71) added; §6.3 the two-key passivation. The Version 1.0 sections are kept for history |
| 1.2 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3) | §1.1 FRAM_BYPASS (0x0B), EPS_DISABLE_TRACKERS (0x15), ADCS_SET_GYRO_ALIGN (0x25) and ADCS_SET_MAG_BIAS (0x26) added |
| 1.3 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, third part) | §1.1 DEPLOY_RETRY (0x50): optional force flag for the antenna past the AntS temperature gate (decision C-18, SRS-F165) |
| 1.4 | 2026-10-27 | Documentation audit (post-CDR 2026) | Status "CDR Baseline" replaced by Draft; §1 governing source restated; §2–§7 rewritten to the flight command set from `command_dictionary.json`, SQSO-ICD-013 §3–§9 and SQSO-ICD-005 §5.5, §7 (frame, keys and authority, definitions, execution and acknowledgement, constraints, ground tool); the Version 1.0 frame and CMD-0100 to CMD-FF01 definitions removed |
