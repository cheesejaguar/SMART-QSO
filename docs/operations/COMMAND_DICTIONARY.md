# SMART-QSO Command Dictionary

**Document ID**: SQSO-CMD-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines all commands for the SMART-QSO mission. It serves as the authoritative reference for command generation and verification.

**Governing source (Version 1.1).** The flight command set is generated from `software/flight/command_dictionary.json`
and documented, with framing and authentication, in SQSO-ICD-013 (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md` §3
to §8, dictionary in §7). Where §2 to §7 below (the Version 1.0 design: a 32-byte frame with a CRC-16, 16-bit opcodes)
disagree with SQSO-ICD-013, SQSO-ICD-013 governs; those sections are kept for history until this document is rewritten
to the flight command set. The commands the flight software accepts are summarised in §1.1.

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

### 2.1 Frame Format

```
┌──────────────────────────────────────────────────────────────┐
│                    COMMAND FRAME (32 bytes)                  │
├──────┬──────┬──────┬──────┬───────────────────┬─────┬───────┤
│ SYNC │CALLSN│ SEQ  │ CMD  │    PARAMETERS     │SPARE│  CRC  │
│2 byte│6 byte│2 byte│2 byte│     16 bytes      │2byte│2 bytes│
└──────┴──────┴──────┴──────┴───────────────────┴─────┴───────┘
```

### 2.2 Header Fields

| Field | Offset | Size | Format | Description |
|-------|--------|------|--------|-------------|
| SYNC | 0 | 2 | 0x55AA | Command sync word |
| CALLSN | 2 | 6 | ASCII | Source callsign (padded) |
| SEQ | 8 | 2 | uint16 | Sequence number |
| CMD | 10 | 2 | uint16 | Command opcode |

### 2.3 Validation

All commands are validated before execution:
1. SYNC word matches 0x55AA
2. Callsign in authorized list
3. Sequence number > last received (replay protection)
4. CRC-16 valid
5. Command opcode valid
6. Parameters in range

## 3. Command Categories

### 3.1 Category Overview

| Category | Opcode Range | Description | Authority |
|----------|--------------|-------------|-----------|
| Mode Control | 0x0100-0x01FF | Spacecraft mode commands | MD |
| Configuration | 0x0200-0x02FF | Parameter configuration | FC |
| Payload | 0x0300-0x03FF | AI payload control | FC |
| Diagnostic | 0x0400-0x04FF | Diagnostic and test | MD |
| Software | 0x0500-0x05FF | Software management | MD + SW |
| Emergency | 0xFF00-0xFFFF | Emergency commands | Any |

### 3.2 Authority Levels

| Level | Abbreviation | Description |
|-------|--------------|-------------|
| FC | Flight Controller | Certified operator |
| MD | Mission Director | Mission Director approval |
| SW | Software Lead | Software team approval |
| Any | Any Operator | Emergency use only |

## 4. Command Definitions

### 4.1 Mode Control Commands

#### CMD-0100: SAFE_MODE
Enter safe mode immediately.

| Field | Value |
|-------|-------|
| Opcode | 0x0100 |
| Parameters | None |
| Authority | FC |
| Hazardous | No |

**Verification**: STATE = SAFE in next telemetry

**Effects**:
- Immediately transition to SAFE mode
- Disable non-essential systems
- Reduce beacon power
- Disable AI payload

---

#### CMD-0101: RELEASE_SAFE
Release safe mode (if conditions met).

| Field | Value |
|-------|-------|
| Opcode | 0x0101 |
| Parameters | None |
| Authority | MD |
| Hazardous | Yes |

**Prerequisites**:
- SOC > 30%
- No active critical faults
- Mission Director approval

**Verification**: STATE = IDLE in next telemetry

---

#### CMD-0102: SET_MODE
Set operational mode directly.

| Field | Value |
|-------|-------|
| Opcode | 0x0102 |
| Parameters | P0: target mode (1=SAFE, 2=IDLE, 3=ACTIVE) |
| Authority | MD |
| Hazardous | Yes (if ACTIVE) |

**Verification**: STATE = requested mode in telemetry

---

### 4.2 Configuration Commands

#### CMD-0200: SET_BEACON_RATE
Set beacon transmission interval.

| Field | Value |
|-------|-------|
| Opcode | 0x0200 |
| Parameters | P0-P1: interval in seconds (uint16, 30-300) |
| Authority | FC |
| Hazardous | No |

**Verification**: Beacon interval changes

---

#### CMD-0201: SET_TX_POWER
Set RF transmit power level.

| Field | Value |
|-------|-------|
| Opcode | 0x0201 |
| Parameters | P0: power level (0=0.5W, 1=1W, 2=2W) |
| Authority | FC |
| Hazardous | No |

**Verification**: TX_POWER in telemetry

---

#### CMD-0202: SET_TLM_RATE
Set telemetry collection rate.

| Field | Value |
|-------|-------|
| Opcode | 0x0202 |
| Parameters | P0: rate in Hz (1-10) |
| Authority | FC |
| Hazardous | No |

---

### 4.3 Payload Commands

#### CMD-0300: JETSON_POWER
Control Jetson power state.

| Field | Value |
|-------|-------|
| Opcode | 0x0300 |
| Parameters | P0: state (0=OFF, 1=ON) |
| Authority | FC |
| Hazardous | No |

**Verification**: JETSON_STATE in FLAGS

---

#### CMD-0301: AI_ENABLE
Enable or disable AI beacon generation.

| Field | Value |
|-------|-------|
| Opcode | 0x0301 |
| Parameters | P0: state (0=disable, 1=enable) |
| Authority | FC |
| Hazardous | No |

**Verification**: AI_AVAILABLE flag

---

#### CMD-0302: FORCE_FALLBACK
Force fallback beacon mode (disable AI).

| Field | Value |
|-------|-------|
| Opcode | 0x0302 |
| Parameters | None |
| Authority | FC |
| Hazardous | No |

**Verification**: FALLBACK_MODE = 1

---

### 4.4 Diagnostic Commands

#### CMD-0400: REQUEST_TLM
Request immediate telemetry transmission.

| Field | Value |
|-------|-------|
| Opcode | 0x0400 |
| Parameters | None |
| Authority | FC |
| Hazardous | No |

**Verification**: Beacon transmitted immediately

---

#### CMD-0401: CLEAR_FAULTS
Clear latched fault flags.

| Field | Value |
|-------|-------|
| Opcode | 0x0401 |
| Parameters | None |
| Authority | MD |
| Hazardous | No |

**Verification**: FAULT_FLAGS = 0x00

---

#### CMD-0402: RESET_COUNTERS
Reset telemetry counters.

| Field | Value |
|-------|-------|
| Opcode | 0x0402 |
| Parameters | P0: counter mask |
| Authority | MD |
| Hazardous | No |

Counter mask bits:
- Bit 0: TX_COUNT
- Bit 1: RX_COUNT
- Bit 2: WDT_RESETS

---

#### CMD-0403: ECHO_TEST
Echo command for link testing.

| Field | Value |
|-------|-------|
| Opcode | 0x0403 |
| Parameters | P0-P15: echo data |
| Authority | FC |
| Hazardous | No |

**Verification**: Echo in beacon info field

---

### 4.5 Software Commands

#### CMD-0500: SW_RESET
Software reset (warm reboot).

| Field | Value |
|-------|-------|
| Opcode | 0x0500 |
| Parameters | None |
| Authority | MD |
| Hazardous | Yes |

**Verification**: BOOT_REASON = SW_RESET, UPTIME reset

---

#### CMD-0501: BANK_SWITCH
Switch to alternate software bank.

| Field | Value |
|-------|-------|
| Opcode | 0x0501 |
| Parameters | None |
| Authority | MD + SW |
| Hazardous | Yes |

**Prerequisites**:
- Backup bank verified valid
- Full pass remaining
- Two operators on console

**Verification**: Version changes in beacon

---

#### CMD-0502: UPLOAD_START
Begin software upload.

| Field | Value |
|-------|-------|
| Opcode | 0x0502 |
| Parameters | P0: bank (0=A, 1=B), P1-P4: size |
| Authority | MD + SW |
| Hazardous | Yes |

---

#### CMD-0503: UPLOAD_DATA
Send software upload block.

| Field | Value |
|-------|-------|
| Opcode | 0x0503 |
| Parameters | P0-P1: sequence, P2-P15: data (14 bytes) |
| Authority | MD + SW |
| Hazardous | No |

---

#### CMD-0504: UPLOAD_END
Finalize software upload.

| Field | Value |
|-------|-------|
| Opcode | 0x0504 |
| Parameters | P0-P3: CRC-32 |
| Authority | MD + SW |
| Hazardous | No |

**Verification**: CRC match reported in beacon

---

### 4.6 Emergency Commands

#### CMD-FF00: EMERGENCY_SAFE
Emergency safe mode (no prerequisites).

| Field | Value |
|-------|-------|
| Opcode | 0xFF00 |
| Parameters | None |
| Authority | Any |
| Hazardous | No |

**Use**: When normal SAFE_MODE fails

---

#### CMD-FF01: EMERGENCY_RESET
Emergency hardware reset.

| Field | Value |
|-------|-------|
| Opcode | 0xFF01 |
| Parameters | None |
| Authority | Any |
| Hazardous | Yes |

**Use**: Last resort recovery

---

## 5. Command Execution

### 5.1 Execution Flow

```
COMMAND EXECUTION SEQUENCE

1. Receive frame
2. Validate CRC
3. Validate callsign
4. Validate sequence (anti-replay)
5. Validate opcode
6. Validate parameters
7. Check prerequisites
8. Execute command
9. Update telemetry
10. Log execution
```

### 5.2 Response Codes

| Code | Name | Description |
|------|------|-------------|
| 0x00 | ACK | Command accepted |
| 0x01 | NAK_CRC | CRC error |
| 0x02 | NAK_AUTH | Authentication failed |
| 0x03 | NAK_SEQ | Sequence error |
| 0x04 | NAK_CMD | Invalid command |
| 0x05 | NAK_PARAM | Invalid parameter |
| 0x06 | NAK_STATE | Invalid state for command |
| 0x07 | NAK_BUSY | System busy |

## 6. Command Constraints

### 6.1 Rate Limits

| Command Type | Rate Limit |
|--------------|------------|
| All commands | 1 per second max |
| Mode changes | 1 per minute |
| SW upload | 10 blocks per second |

### 6.2 State Constraints

| Command | Allowed States |
|---------|----------------|
| SAFE_MODE | Any |
| RELEASE_SAFE | SAFE |
| SET_MODE | IDLE, ACTIVE |
| BANK_SWITCH | SAFE only |
| UPLOAD_* | SAFE only |

### 6.3 Hazardous Command Confirmation

**Flight implementation (Version 1.1).** The one irreversible command is end-of-life passivation, which takes two
commands under two keys: `PASSIVATE_ARM` (0x70, operational key, argument `EOL!`), then `PASSIVATE_EXECUTE` (0x71,
recovery key) within 600 s; the arm lapses otherwise (SQSO-ICD-013 §6.4). `EPS_CLEAR_LOCKOUT` (0x14) needs its
argument `CHG!` and acts only with the EPS controller's unlock. The Version 1.0 scheme below is not implemented.

Version 1.0 scheme (superseded):

Hazardous commands require two-step execution:
1. Send command with CONFIRM=0
2. Receive confirmation prompt in beacon
3. Send command with CONFIRM=1 within 60 seconds

## 7. Quick Reference

### 7.1 Common Commands

| Action | Command | Opcode |
|--------|---------|--------|
| Enter safe mode | SAFE_MODE | 0x0100 |
| Exit safe mode | RELEASE_SAFE | 0x0101 |
| Set TX power | SET_TX_POWER | 0x0201 |
| Power cycle Jetson | JETSON_POWER | 0x0300 |
| Request telemetry | REQUEST_TLM | 0x0400 |
| Software reset | SW_RESET | 0x0500 |

### 7.2 Command Generation Example

```python
import struct

def create_command(callsign, seq, opcode, params=None):
    """Generate command frame."""
    frame = bytearray(32)

    # Sync
    frame[0:2] = struct.pack('<H', 0x55AA)

    # Callsign (6 bytes, padded)
    cs = callsign.encode()[:6].ljust(6)
    frame[2:8] = cs

    # Sequence
    frame[8:10] = struct.pack('<H', seq)

    # Opcode
    frame[10:12] = struct.pack('<H', opcode)

    # Parameters (16 bytes)
    if params:
        frame[12:12+len(params)] = params

    # CRC (bytes 30-31)
    crc = crc16_ccitt(frame[0:30])
    frame[30:32] = struct.pack('<H', crc)

    return bytes(frame)

# Example: SAFE_MODE command
cmd = create_command("N0CALL", 1, 0x0100)
```

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 2) | §1: SQSO-ICD-013 and the generated dictionary govern; §1.1 the flight command set, with DUMP_TLM (0x0A), EPS_CLEAR_LOCKOUT (0x14), PASSIVATE_ARM (0x70) and PASSIVATE_EXECUTE (0x71) added; §6.3 the two-key passivation. The Version 1.0 sections are kept for history |
| 1.2 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3) | §1.1 FRAM_BYPASS (0x0B), EPS_DISABLE_TRACKERS (0x15), ADCS_SET_GYRO_ALIGN (0x25) and ADCS_SET_MAG_BIAS (0x26) added |
| 1.3 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, third part) | §1.1 DEPLOY_RETRY (0x50): optional force flag for the antenna past the AntS temperature gate (decision C-18, SRS-F165) |
