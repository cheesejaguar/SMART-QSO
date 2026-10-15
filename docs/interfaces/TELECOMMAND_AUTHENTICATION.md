# Interface Control Document: Telecommand Authentication and TX-OFF

**Document ID**: SQSO-ICD-013
**Version**: 0.1 (Draft)
**Date**: 2026-10-15
**Status**: Draft. Implemented and tested in simulation; hardware-in-the-loop verification pending.

## 1. Purpose

This document defines how the ground segment builds uplink telecommands and how the flight software authenticates them. It also defines the latched transmitter-off (TX-OFF) command. Where this document conflicts with SQSO-ICD-005 (`ICD_SPACE_GROUND.md`) on command framing, this document governs.

## 2. Regulatory Basis

| Source | Requirement | How it is met |
|--------|-------------|---------------|
| 47 CFR 97.207(b) | A space station must have a means to cease transmissions by telecommand whenever the FCC orders it | `TX_DISABLE` (0x32) latches the transmitter off in every operating state (Section 6) |
| ITU RR 22.1 | Space stations must be able to cease emissions immediately by telecommand | Same as above |
| 47 CFR 97.211(b) | Telecommand to a space station may use codes that obscure meaning | The uplink is authenticated with a message authentication code. Nothing is encrypted, and the downlink stays in the clear (97.113(a)(4)) |
| 47 CFR 97.211(a) | Only designated telecommand stations may send telecommand | Only holders of the key can produce an accepted command. The stations themselves are still to be designated (finding RG-03) |

## 3. Uplink Frame

```
+------+------+--------+-----+-----------------+-------------+
| 0xAA | 0x55 | cmd_id | len | payload (len B) | sum16 (2 B) |
+------+------+--------+-----+-----------------+-------------+
```

- `cmd_id`: the command ID (Section 7).
- `len`: the payload length in bytes. It always includes the 20 bytes of authentication overhead.
- `sum16`: the 16-bit additive sum of every preceding byte, sync bytes included, sent big-endian. It catches transmission errors only and provides no security.

The flight software's framing check (`input_validate_command()`) rejects frames shorter than 4 bytes, longer than 256 bytes, without the sync bytes, whose length disagrees with `len`, or that fail `sum16`.

Link-layer encapsulation, such as AX.25 or a FEC wrapper, is defined by the RF interface (SQSO-ICD-005). The flight software has no receive driver yet (Section 10).

## 4. Authenticated Payload

```
+-------------------+---------------+-------------+
| counter (4 B, BE) | args (N bytes) | tag (16 B)  |
+-------------------+---------------+-------------+
```

```
tag = first 16 bytes of HMAC-SHA-256(key, MAC input)

MAC input = "SQSO-TC-v1" | cmd_id (1 B) | N (1 B) | counter (4 B, BE) | args (N B)
```

| Parameter | Value |
|-----------|-------|
| MAC | HMAC-SHA-256 (FIPS 198-1, RFC 2104), SHA-256 per FIPS 180-4 |
| Key | 32 bytes, unique to this spacecraft |
| Tag | 16 bytes (128 bits), the leftmost bytes of the HMAC output |
| Domain separation | ASCII `SQSO-TC-v1`, 10 bytes, no terminator |
| Maximum args (N) | 64 bytes |
| Counter | Unsigned 32-bit, big-endian |

Design notes:

- The MAC covers `cmd_id` and `N`. A captured frame therefore cannot be relabelled as a different command, and its arguments cannot be extended or truncated.
- The domain string separates these tags from any other use of the key, and the version suffix allows the format to change later.
- The flight software compares tags in constant time, so the comparison takes the same time whichever byte differs.

## 5. Counter Rules

### 5.1 Onboard

1. A command is accepted only if its counter is strictly greater than the last accepted counter.
2. Counter 0xFFFFFFFF is never accepted. Once the last accepted counter reaches 0xFFFFFFFE, every command is rejected until a new key is provisioned.
3. The counter is checked only after the tag verifies. A forged frame therefore reveals nothing about the counter.
4. The accepted counter is persisted before the command executes. A replay is rejected even if the spacecraft resets immediately after executing.
5. If the counter cannot be persisted, every command except `TX_DISABLE` is refused with `EXEC_FAIL`. The in-memory counter still advances, so the frame cannot be replayed before the next reset. `TX_DISABLE` executes anyway because reducing emissions is always safe, and replaying a TX-OFF can only silence the station.

### 5.2 Ground

1. Every frame uses a new counter, and the counter never goes backwards.
2. The ground tool reserves the counter on disk (written and fsync'd) before it produces the frame. A crash can skip a counter but never reuse one.
3. **All telecommand stations must draw counters from one shared sequence.** The onboard window is strictly monotonic. Splitting the counter range between stations does not work: after one station's high counter is accepted, the other station's lower counters are all rejected.
4. If a frame is lost, send the next command with a new counter. A skipped counter is harmless, and a lost frame never needs to be re-sent with the same counter.

## 6. TX-OFF

| Property | Behavior |
|----------|----------|
| Commands | `TX_DISABLE` (0x32) latches TX-OFF. `TX_ENABLE` (0x31) clears it. Neither takes arguments. |
| Allowed states | Every operating state: BOOT, SAFE, DETUMBLE, IDLE, ACTIVE, EMERGENCY |
| Order of operations | `TX_DISABLE` drives `GPIO_PIN_RF_TX_EN` low first, then persists the latch. The transmitter is off even if the latch cannot be saved; in that case the command still reports `SUCCESS`, and the fault log records the persistence failure. |
| Persistence | Two-slot record with sequence number and CRC-32 (`nvm_record`), region `FLASH_REGION_RF_CONTROL` |
| After reset | The latch is restored at boot before anything can transmit (`rf_control_init()` in `main.c`) |
| Blank record (first boot) | Transmit permitted |
| Corrupt or unreadable record | Transmit **inhibited** (fail safe) |
| SAFE mode, EPS `beacon_enabled` | Cannot override the latch. `beacon_should_transmit()` and `beacon_transmit()` refuse while TX-OFF is latched. |
| Command receiver | Not switched by TX-OFF or by EPS `radio_enabled`. It must stay powered so the station can still be commanded back on. |

Because only authenticated commands are accepted, a replayed `TX_ENABLE` cannot undo a later `TX_DISABLE`. This is tested in `test_replayed_tx_enable_cannot_undo_tx_off`.

## 7. Command Dictionary

The argument length must match the dictionary exactly, or the command is rejected with `INVALID_PARAM` and its counter is not consumed. The single source of the onboard dictionary is `cmd_args_spec()` in `software/flight/src/cmd_handler.c`. `COMMANDS` in `software/ground/tools/telecommand_builder.py` mirrors it, and `test_every_dictionary_command` and `TestDictionary` check both sides.

| ID | Name | Args (bytes) | Notes |
|----|------|--------------|-------|
| 0x00 | NOP | 0 | |
| 0x01 | RESET | 0 | Logged only: no build calls a hardware reset yet |
| 0x02 | SET_MODE | 1 | 0 = SAFE, 1 = IDLE, 2 = ACTIVE |
| 0x03 | GET_STATUS | 0 | Returns 4 bytes |
| 0x05 | CLEAR_FAULTS | 0 | |
| 0x10 | EPS_SET_MODE | 1 | 0 = SAFE, 1 = IDLE, 2 = ACTIVE |
| 0x13 | EPS_GET_TELEMETRY | 0 | |
| 0x21 | ADCS_DETUMBLE | 0 | |
| 0x23 | ADCS_GET_ATTITUDE | 0 | |
| 0x30 | SET_BEACON | 2 | Interval in seconds, big-endian; 10 to 120 s (SYS-211) |
| 0x31 | TX_ENABLE | 0 | Clears the TX-OFF latch |
| 0x32 | TX_DISABLE | 0 | Latches TX-OFF; allowed in every state |
| 0x40 | PLD_ENABLE | 0 | |
| 0x41 | PLD_DISABLE | 0 | |

The following IDs are defined in `cmd_handler.h` but have no handler. They are **not** in the dictionary and are refused with `INVALID_CMD` before a counter is consumed: 0x04 SET_TIME, 0x11 EPS_ENABLE_HEATER, 0x12 EPS_DISABLE_HEATER, 0x20 ADCS_SET_MODE, 0x22 ADCS_POINT_SUN, 0x33 SET_POWER (needed for SYS-201), 0x42 PLD_SEND_DATA and 0x43 PLD_REQUEST_DATA.

Any other ID is also rejected with `INVALID_CMD`, unless a handler was registered for it at run time with `cmd_register_handler()`; such commands accept 0 to 64 argument bytes.

## 8. Onboard Processing Order and Results

`cmd_process()` handles each frame in this order:

| Step | Check | Result on failure | Counter consumed? |
|------|-------|-------------------|-------------------|
| 1 | Framing (Section 3) | `INVALID_CMD` | No |
| 2 | Key provisioned | `AUTH_FAIL` (`auth_failures` incremented) | No |
| 3 | Payload carries at least 20 bytes and at most 64 argument bytes | `INVALID_PARAM` | No |
| 4 | Tag valid, then counter not exhausted and fresh | `AUTH_FAIL` (`auth_failures` incremented) | No |
| 5 | Argument length matches the dictionary | `INVALID_CMD` (unknown ID) or `INVALID_PARAM` | No |
| 6 | Counter persisted (`TX_DISABLE` continues regardless) | `EXEC_FAIL` | In memory only |
| 7 | Command allowed in the current state | `NOT_ALLOWED` | Yes |
| 8 | Execute | Handler result | Yes |

Result codes (`CmdResult_t`): 0 `SUCCESS`, 1 `INVALID_CMD`, 2 `INVALID_PARAM`, 3 `AUTH_FAIL`, 4 `NOT_ALLOWED`, 5 `EXEC_FAIL`, 6 `BUSY`.

Every rejection is counted in `CmdStats_t`. Only the first rejection after an accepted command is written to the fault log, because each fault-log entry is a non-volatile write. This stops a stream of forged frames from wearing out storage or flushing real faults out of the log.

`cmd_execute()` does no authentication. It is for commands generated onboard, or ones that have already passed `cmd_process()`.

## 9. Key Management

| Item | Rule |
|------|------|
| Generation | 32 bytes from a cryptographically secure generator, one key per spacecraft |
| Storage on ground | Key file of 64 hex characters, mode 600, held only by designated telecommand stations. Never committed to any repository. |
| Provisioning | At integration, through `cmd_auth_provision_key()` (resets the counter to 0). There is no default key. Until a key is provisioned every command is rejected, including `TX_DISABLE`. |
| Onboard storage | Two-slot record with CRC-32 in `FLASH_REGION_AUTH`: key (32), key ID (1), reserved (3), last counter (4, BE) |
| Corrupt record | Commands locked (fail closed); fault logged |
| Test key | `00 01 02 ... 1F` is used **only** by unit tests and the test vectors below. It must never be loaded into flight hardware. |

**Not yet implemented (open):**

- A ground-support-equipment command that calls `cmd_auth_provision_key()` over the umbilical.
- An authenticated on-orbit re-key. A compromised key currently cannot be replaced after launch.

The consequence of the "commands locked" rule must be accepted explicitly. If the key record is corrupted on orbit, no command, `TX_DISABLE` included, is accepted until the record is restored, so any TX-OFF obligation would then rest on the hardware inhibits. The trade between failing closed and failing open with a fallback key is recorded under finding SW-03.

## 10. Implementation Status

| Item | Status |
|------|--------|
| SHA-256 and HMAC-SHA-256 (`sha256.c`) | Implemented; FIPS 180-4 and RFC 4231 vectors pass |
| Two-slot non-volatile record (`nvm_record.c`) | Implemented; power-fail and corruption cases tested in simulation |
| Authentication and replay counter (`cmd_auth.c`) | Implemented; 16 unit tests |
| Latched TX-OFF (`rf_control.c`), beacon gating (`beacon.c`) | Implemented; 10 unit tests |
| Command path (`cmd_handler.c`) | Implemented; 16 end-to-end frame tests, including every dictionary command |
| Ground builder (`telecommand_builder.py`) | Implemented; byte-identical to the flight test vectors |
| Receive driver feeding `cmd_process()` from the 70 cm receiver | **Not implemented** |
| `GPIO_PIN_RF_TX_EN` wired to the PA power switch | **Not verified**; hardware-in-the-loop test pending |
| Hardware mapping of `FLASH_REGION_AUTH` and `FLASH_REGION_RF_CONTROL` to FRAM | Allocated in SQSO-OBC-002 §3.3; driver not implemented (simulation HAL only) |
| `SET_MODE` | Authenticated now, but still uses `sm_force_state()` and bypasses transition guards (open under SW-03) |

## 11. Test Vectors

All vectors use the TEST-ONLY key `000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F`. The flight tests (`test_cmd_auth.c`, `test_cmd_handler.c`) and the ground tests (`test_telecommand_builder.py`) check the same bytes.

| Command | Counter | Args | Payload (hex) |
|---------|---------|------|---------------|
| TX_DISABLE (0x32) | 1 | none | `00000001 9A83F5C7B21F7511E82F5E94B9DE5851` |
| SET_BEACON (0x30) | 2 | `003C` | `00000002 003C B49634055CFFC160654E1B7E96DA3C13` |
| TX_ENABLE (0x31) | 2 | none | `00000002 3B24D1FDF658A147FAA5F97B4BCF779C` |
| TX_DISABLE (0x32) | 3 | none | `00000003 566CA34E13C96BA0DC25E0AA93DDD23B` |
| GET_STATUS (0x03) | 1 | none | `00000001 81A953BF2B4C0361FB2CC17DA4789B12` |

Complete frames:

```
TX_DISABLE, counter 1:
AA55 32 14 00000001 9A83F5C7B21F7511E82F5E94B9DE5851 09BF

TX_ENABLE, counter 3:
AA55 31 14 00000003 64B167923681E291200754A1BB670845 080A
```

To reproduce a vector with the ground tool, using a scratch copy of the test key:

```bash
python3 software/ground/tools/telecommand_builder.py TX_DISABLE --key-file test.key --counter 1
```

## 12. Verification

| Requirement | Method | Evidence |
|-------------|--------|----------|
| Forged, relabelled, truncated or extended frames rejected | Test | `test_cmd_auth.c`, `test_cmd_handler.c` |
| Replays rejected, including after reset | Test | `test_replay_rejected_after_reset`, `test_cmd_auth.c` |
| TX-OFF accepted in every state and survives a power cycle | Test (simulation) | `test_tx_disable_accepted_in_every_state`, `test_rf_control.c` |
| TX-OFF executes when persistence fails | Test (simulation) | `test_tx_disable_executes_despite_persist_failure` |
| TX-OFF removes RF output at the antenna port | Test (HIL) | **Pending** |
| Ground and flight agree byte for byte | Test | `test_frame_matches_ground_tool`, `test_telecommand_builder.py` |

## 13. Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1 | 2026-10-15 | Initial draft with the flight implementation (findings SW-02, SW-03, SW-04) |
