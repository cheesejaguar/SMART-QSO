# Interface Control Document: Telecommand Authentication and TX-OFF

**Document ID**: SQSO-ICD-013
**Version**: 0.2 (Draft)
**Date**: 2026-10-15
**Status**: Draft. Implemented and tested in the host simulation. Not yet run on target hardware (Section 10).

## 1. Purpose

This document covers four things:

- how the ground segment builds uplink telecommands and how the flight software receives and authenticates them;
- the latched transmitter-off (TX-OFF) command;
- the telecommand key table and how keys are rotated;
- how keys are provisioned through the GSE umbilical.

Where this document conflicts with SQSO-ICD-005 (`ICD_SPACE_GROUND.md`) or `docs/operations/CMD_TLM_DATABASE.md` on telecommand, this document governs.

## 2. Regulatory Basis

| Source | Requirement | How it is met |
|--------|-------------|---------------|
| 47 CFR 97.207(b) | A space station must have a means to cease transmissions by telecommand whenever the FCC orders it | `TX_DISABLE` (0x32) latches the transmitter off in every operating state. It is still accepted when no key is usable (Section 6) |
| ITU RR 22.1 | Space stations must be able to cease emissions immediately by telecommand | Same as above |
| 47 CFR 97.211(b) | Telecommand to a space station may use codes that obscure meaning | The uplink is authenticated with a message authentication code. Nothing is encrypted, no key material is ever sent over RF, and the downlink stays in the clear (97.113(a)(4)) |
| 47 CFR 97.211(a) | Only designated telecommand stations may send telecommand | Only holders of a key can produce an accepted command. The stations themselves are still to be designated (finding RG-03) |

## 3. Uplink Framing

### 3.1 Link layer

Each telecommand is one AX.25 v2.2 UI frame (control `0x03`, PID `0xF0`) on the 435 MHz uplink. The destination address must be this station's call sign and SSID (`software/flight/include/station_id.h`). Frames addressed to any other station are dropped.

- **Call sign:** the value in the repository, `SQSO`-1, is a placeholder until the licensee's call sign is issued (decision D1).
- **Source address:** must be the sending telecommand station's own call sign.
- **Digipeater paths:** parsed, but not needed.

```
+------------------+----------------+------+------+------------------------+-----------+
| dest (7)         | src (7)        | 0x03 | 0xF0 | command frame (3.2)    | FCS (2,LE)|
+------------------+----------------+------+------+------------------------+-----------+
```

- **Receiver HAL contract (`hal_radio.h`):** each frame is delivered from the first address byte through the 2-byte FCS, with HDLC flags and bit stuffing already removed.
- **Defence in depth:** `uplink.c` checks the FCS (CRC-16/X.25) again even if the transceiver already did.
- **KISS TNCs:** a TNC appends the FCS itself, so the ground tool's `--kiss` output leaves it out (Section 11).

### 3.2 Command frame

```
+------+------+--------+-----+-----------------+-------------+
| 0xAA | 0x55 | cmd_id | len | payload (len B) | sum16 (2 B) |
+------+------+--------+-----+-----------------+-------------+
```

- `cmd_id`: the command ID (Section 7).
- `len`: the payload length in bytes, always including the 20 bytes of authentication overhead.
- `sum16`: the 16-bit additive sum of every preceding byte, sent big-endian. It detects errors only and provides no security.

`input_validate_command()` rejects any of the following:

- frames shorter than 4 bytes or longer than 256 bytes;
- frames without the sync bytes;
- frames whose length disagrees with `len`;
- frames that fail `sum16`.

## 4. Authenticated Payload

```
+-------------------+----------------+-------------+
| counter (4 B, BE) | args (N bytes) | tag (16 B)  |
+-------------------+----------------+-------------+

tag       = first 16 bytes of HMAC-SHA-256(key, MAC input)
MAC input = "SQSO-TC-v1" | cmd_id (1 B) | N (1 B) | counter (4 B, BE) | args (N B)
```

| Parameter | Value |
|-----------|-------|
| MAC | HMAC-SHA-256 (FIPS 198-1, RFC 2104), SHA-256 per FIPS 180-4 |
| Key | 32 bytes. Section 9 says which key authenticates which command |
| Tag | 16 bytes (128 bits), the leftmost bytes of the HMAC output |
| Domain separation | ASCII `SQSO-TC-v1`, 10 bytes, no terminator |
| Maximum args (N) | 64 bytes |
| Counter | Unsigned 32-bit, big-endian; one counter per key |

- **Command ID and length are in the MAC.** A captured frame cannot be relabelled as another command, and its arguments cannot be extended or truncated.
- **Comparison:** tags are compared in constant time.
- **Domain string:** separates these tags from any other use of the key. Its version suffix leaves room to change the format later.

## 5. Counter Rules

### 5.1 Onboard

1. **Separate counters.** The active operational key and the recovery key each have their own counter. A command is accepted only if its counter is strictly greater than the last one accepted for the key that authenticated it.
2. **Exhaustion.** Counter 0xFFFFFFFF is never accepted. Once a key's last counter reaches 0xFFFFFFFE, that key accepts nothing more; rotate to a spare (Section 9.3).
3. **Order of checks.** The counter is checked only after the tag verifies.
4. **Persist before executing.** The accepted counter is saved before the command runs, so a replay is rejected even across a reset.
5. **Persistence failure.**
   - Every command except `TX_DISABLE` is refused with `EXEC_FAIL`.
   - The in-memory counter still advances, so the frame cannot be replayed before the next reset.
   - `TX_DISABLE` executes anyway: reducing emissions is always safe.
6. **Rotation.** `KEY_ROTATE` starts the new operational key at counter 0.

### 5.2 Ground

1. **Counter files.** Every frame uses a new counter, and counters never go backwards. Keep one counter file per key: one for the active operational key and one for the recovery key. Start a new operational counter file after each rotation.
2. **Reserve first.** `telecommand_builder.py --counter-file` writes and fsyncs the counter before producing the frame. A crash can skip a counter but never reuse one.
3. **All telecommand stations must draw counters from one shared sequence per key.** The onboard window is strictly monotonic, so splitting the counter range between stations does not work.
4. **Lost frames.** If a frame is lost, send the next command with a new counter. Never re-send a frame with the same counter.

## 6. TX-OFF

| Property | Behavior |
|----------|----------|
| Commands | `TX_DISABLE` (0x32) latches TX-OFF. `TX_ENABLE` (0x31) clears it. Neither takes arguments |
| Authentication | `TX_DISABLE`: operational **or** recovery key. `TX_ENABLE`: operational key only |
| **No usable key** | `TX_DISABLE` is accepted **without authentication** (the payload must still be well formed). Every other command, `TX_ENABLE` included, is rejected. Decision recorded 2026-10-15: TX-OFF can only reduce emissions, so the 97.207(b) obligation never depends on key material. Once TX-OFF is latched in this state, the station stays silent until keys are re-provisioned |
| Allowed states | Every operating state: BOOT, SAFE, DETUMBLE, IDLE, ACTIVE, EMERGENCY |
| Order of operations | `TX_DISABLE` drives `GPIO_PIN_RF_TX_EN` low first, then saves the latch. The transmitter is off even if the latch cannot be saved; the command still reports `SUCCESS`, and the fault log records the failure |
| Persistence | Two-slot record with CRC-32 (`nvm_record`) in `FLASH_REGION_RF_CONTROL`, base 0 |
| After reset | The latch is restored at boot, before anything can transmit (`rf_control_init()` in `main.c`) |
| Blank record (first boot) | Transmit permitted |
| Corrupt or unreadable record | Transmit **inhibited** (fail safe) |
| SAFE mode, EPS `beacon_enabled` | Cannot override the latch. `beacon_should_transmit()` and `beacon_transmit()` refuse while TX-OFF is latched |
| Command receiver | Not switched by TX-OFF or by EPS `radio_enabled`. It stays powered so the station can still be commanded |
| Hardware path | `RF_TX_EN` drives the PA load-switch enable and is pulled down, so a reset or unpowered OBC leaves the PA off (SQSO-ICD-005 §4.2, OBC_DESIGN §4.4). Verified by TP-RF-006 (pending hardware) |

A replayed `TX_ENABLE` cannot undo a later `TX_DISABLE`. `test_replayed_tx_enable_cannot_undo_tx_off` covers this.

## 7. Command Dictionary

The argument length must match the dictionary exactly. If it doesn't, the command is rejected with `INVALID_PARAM` and its counter is not consumed.

**Single source:** `software/flight/command_dictionary.json`. `scripts/gen_command_dictionary.py` generates from it:

- the flight table, `src/cmd_dictionary.c`, used by `cmd_args_spec()` and by `cmd_auth` to choose the key;
- the ground module, `software/ground/tools/command_dictionary.py`.

CI runs the generator with `--check` and fails if either output is stale. `test_every_dictionary_command` and `TestDictionary` check both sides independently.

| ID | Name | Args (bytes) | Key | Notes |
|----|------|--------------|-----|-------|
| 0x00 | NOP | 0 | operational | |
| 0x01 | RESET | 0 | operational | Logged only: no build calls a hardware reset yet |
| 0x02 | SET_MODE | 1 | operational | 0 = SAFE, 1 = IDLE, 2 = ACTIVE. Goes through the state-machine transition table; SAFE → ACTIVE needs IDLE first (`NOT_ALLOWED` otherwise) |
| 0x03 | GET_STATUS | 0 | operational | Returns 4 bytes |
| 0x05 | CLEAR_FAULTS | 0 | operational | |
| 0x06 | KEY_ROTATE | 1 | **recovery** | Activates spare operational slot 0–3 and revokes the current key (Section 9.3). Allowed in every state |
| 0x10 | EPS_SET_MODE | 1 | operational | 0 = SAFE, 1 = IDLE, 2 = ACTIVE |
| 0x13 | EPS_GET_TELEMETRY | 0 | operational | |
| 0x21 | ADCS_DETUMBLE | 0 | operational | |
| 0x23 | ADCS_GET_ATTITUDE | 0 | operational | |
| 0x30 | SET_BEACON | 2 | operational | Interval in seconds, big-endian; 10 to 120 s (SYS-211) |
| 0x31 | TX_ENABLE | 0 | operational | Clears the TX-OFF latch |
| 0x32 | TX_DISABLE | 0 | **any**; none if no key is usable | Latches TX-OFF; allowed in every state |
| 0x33 | SET_POWER | 1 | operational | 0 = 0.5 W, 1 = 1 W (BASELINE §7). Persisted and reported; applying it to the PA is the transmitter driver's job (interface TBD) |
| 0x40 | PLD_ENABLE | 0 | operational | |
| 0x41 | PLD_DISABLE | 0 | operational | |

**Not in the dictionary.** These IDs are defined in `cmd_handler.h` but have no handler, so they are refused with `INVALID_CMD` before a counter is consumed:

- 0x04 SET_TIME
- 0x11 EPS_ENABLE_HEATER, 0x12 EPS_DISABLE_HEATER
- 0x20 ADCS_SET_MODE, 0x22 ADCS_POINT_SUN
- 0x42 PLD_SEND_DATA, 0x43 PLD_REQUEST_DATA

Any other ID is also rejected with `INVALID_CMD`, unless a handler was registered for it at run time with `cmd_register_handler()`. Such commands use the operational key and accept 0 to 64 argument bytes.

## 8. Onboard Processing Order and Results

`uplink_poll()` takes a frame from the receiver. It drops the frame if the FCS fails, if it is not UI/0xF0, or if it is not addressed to this station. Otherwise it passes the information field to `cmd_process()`, which then works through these steps:

| Step | Check | Result on failure | Counter consumed? |
|------|-------|-------------------|-------------------|
| 1 | Framing (Section 3.2) | `INVALID_CMD` | No |
| 2 | A key of the command's class is usable (or the command is `without_key` and no key at all is usable) | `AUTH_FAIL` | No |
| 3 | Payload carries at least 20 bytes and at most 64 argument bytes | `INVALID_PARAM` | No |
| 4 | Tag valid under the candidate key(s), then that key's counter not exhausted and fresh | `AUTH_FAIL` | No |
| 5 | Argument length matches the dictionary | `INVALID_CMD` (unknown ID) or `INVALID_PARAM` | No |
| 6 | Counter persisted for the authenticating key (`TX_DISABLE` continues regardless) | `EXEC_FAIL` | In memory only |
| 7 | Command allowed in the current state | `NOT_ALLOWED` | Yes |
| 8 | Execute | Handler result | Yes |

Steps 2 and 4 increment `auth_failures`.

Result codes (`CmdResult_t`): 0 `SUCCESS`, 1 `INVALID_CMD`, 2 `INVALID_PARAM`, 3 `AUTH_FAIL`, 4 `NOT_ALLOWED`, 5 `EXEC_FAIL`, 6 `BUSY`.

Every rejection is counted in `CmdStats_t`, and receive statistics are kept in `UplinkStats_t`. Only the first rejection after an accepted command is written to the fault log, because each fault-log entry is a non-volatile write. `cmd_execute()` does no authentication; it is only for commands generated onboard or already through `cmd_process()`.

## 9. Key Management

### 9.1 Key table

| Item | Rule |
|------|------|
| Operational keys | Four slots (`CMD_AUTH_OP_SLOTS`). One is **active**; the others are **spares**. Revoked keys are zeroized and marked revoked, never reused |
| Recovery key | One key, kept **offline** on the ground. It authenticates only `KEY_ROTATE` and `TX_DISABLE` |
| Onboard storage | `FLASH_REGION_AUTH`, one two-slot CRC-32 record each (768 B total; layout in `cmd_auth.h`): state record (active slot, operational counter) at 0; recovery record (key, ID, counter) at 128; operational key records (key, ID, status) at 256 + 128 × slot |
| Generation | 32 bytes each from a cryptographically secure generator (`key_provision.py generate`), one set per spacecraft |
| Ground storage | Key files of 64 hex characters, mode 600, held only by designated telecommand stations, never committed (`scripts/check_secrets.py` fails CI on key material) |
| Test keys | Patterns such as `00 01 … 1F` are used **only** by unit tests and the vectors below. They must never be loaded into flight hardware |

### 9.2 Provisioning (integration only)

Keys are loaded through the GSE umbilical UART by `gse_provision.c`.

- **Gate:** frames are honoured only while `GPIO_PIN_UMBILICAL_SENSE` is high. The GSE connector drives it high and the flight harness pulls it low. Bytes received without the umbilical are discarded.
- **Not reachable over RF.** None of this path can be reached from the radio.

```
"SQGP" | op (1) | slot (1) | key_id (1) | key (32) | CRC-32 (4, BE; IEEE 802.3, as zlib)
reply: "SQGR" | status (1)    0 OK, 1 locked (no umbilical), 2 bad frame, 3 bad op, 4 rejected
```

| Op | Meaning |
|----|---------|
| 0x01 PROVISION | Write a key: slots 0–3 as spares, slot 4 as the recovery key (recovery counter reset to 0) |
| 0x02 ACTIVATE | Make a spare the active operational key, revoking any other active key (same code path as `KEY_ROTATE`) |
| 0x03 ZEROIZE | Erase the key table |

Procedure:

1. Run `key_provision.py generate --dir <set>`.
2. Run `key_provision.py frames --dir <set> --out frames.bin`. This produces four operational provisions, the recovery provision, and activation of slot 0.
3. Send `frames.bin` over the umbilical and check that each reply is `SQGR 00`.
4. Delete `frames.bin`.
5. Move `recovery.key` to offline storage.

### 9.3 Rotation on orbit

`KEY_ROTATE` (0x06) carries the target slot. It must be authenticated by the recovery key, and the target must be a spare.

**Steps on the spacecraft:**

1. Mark the spare active.
2. Write the state record pointing at it, starting at counter 0. This write is the commit point.
3. Revoke the previous key.

**Interrupted rotation.** If power is lost before step 2, the old key stays in force, and the next boot revokes the half-activated spare.

**No key material crosses RF.** Rotation only selects among keys loaded at integration, which keeps the uplink free of key material, consistent with 97.211(b) and 97.113(a)(4).

**Spare limit.** With four operational slots, three rotations are possible over the mission.

### 9.4 Faults

| Condition | Effect | Recovery |
|-----------|--------|----------|
| Active operational key or state record corrupt | Operational commands rejected (fail closed); fault logged | `KEY_ROTATE` with the recovery key to a spare. It revokes every key still marked active |
| Recovery record corrupt | Recovery commands rejected; operational commands unaffected | Operational key continues. Rotation is impossible until re-provisioned at integration |
| No usable key at all | Only `TX_DISABLE` accepted (Section 6) | Re-provision through the umbilical (ground only) |
| Operational key compromised | | `KEY_ROTATE` to a spare with the recovery key |
| Recovery key compromised | An attacker could rotate keys, but cannot command anything except TX-OFF | No on-orbit remedy; keep the recovery key offline |

## 10. Implementation Status

| Item | Status |
|------|--------|
| SHA-256, HMAC-SHA-256 (`sha256.c`) | Implemented; FIPS 180-4 and RFC 4231 vectors (9 tests) |
| Two-slot records with base offsets (`nvm_record.c`) | Implemented; power-fail and corruption cases (12 tests) |
| Authentication, per-key counters, key table, rotation (`cmd_auth.c`) | Implemented (25 tests, including interrupted rotation and corrupt state) |
| GSE provisioning (`gse_provision.c`) | Implemented (6 tests; the umbilical gate was mutation-checked) |
| Latched TX-OFF, power level (`rf_control.c`), beacon gating | Implemented (11 tests) |
| Command path (`cmd_handler.c`) | Implemented (19 tests, including every dictionary command, KEY_ROTATE and SET_MODE guards) |
| AX.25 and receive path (`ax25.c`, `uplink.c`) | Implemented; end to end from receiver frame to TX-OFF (7 tests; the callsign filter was mutation-checked) |
| Ground tools (`telecommand_builder.py`, `key_provision.py`) | Implemented; byte-identical to the flight vectors |
| **Target receiver driver behind `hal_radio.h`** | **Not implemented.** Waits on transceiver selection (SQSO-ICD-005's SPI command set is a placeholder) |
| **`RF_TX_EN` removes PA power at the antenna port** | **Not verified.** TP-RF-006 is written; it needs EM hardware |
| **SET_POWER applied to the PA** | **Not implemented.** The level is persisted and reported; the actuator interface is TBD in the RF design |
| FRAM driver for `FLASH_REGION_AUTH` and `FLASH_REGION_RF_CONTROL` | Allocated in SQSO-OBC-002 §3.3; simulation HAL only |
| Station call sign | Placeholder `SQSO`-1 (decision D1) |

## 11. Test Vectors

All vectors use the TEST-ONLY key `000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F`. The flight tests (`test_cmd_auth.c`, `test_cmd_handler.c`, `test_ax25.c`, `test_uplink.c`, `test_gse_provision.c`) and the ground tests (`test_telecommand_builder.py`, `test_key_provision.py`) check the same bytes.

| Command | Counter | Args | Payload (hex) |
|---------|---------|------|---------------|
| TX_DISABLE (0x32) | 1 | none | `00000001 9A83F5C7B21F7511E82F5E94B9DE5851` |
| SET_BEACON (0x30) | 2 | `003C` | `00000002 003C B49634055CFFC160654E1B7E96DA3C13` |
| TX_ENABLE (0x31) | 2 | none | `00000002 3B24D1FDF658A147FAA5F97B4BCF779C` |
| TX_DISABLE (0x32) | 3 | none | `00000003 566CA34E13C96BA0DC25E0AA93DDD23B` |
| GET_STATUS (0x03) | 1 | none | `00000001 81A953BF2B4C0361FB2CC17DA4789B12` |

Complete command frames:

```
TX_DISABLE, counter 1:
AA55 32 14 00000001 9A83F5C7B21F7511E82F5E94B9DE5851 09BF

TX_ENABLE, counter 3:
AA55 31 14 00000003 64B167923681E291200754A1BB670845 080A
```

AX.25 UI frame `SQSO-1` ← `N0CALL` carrying TX_DISABLE counter 1 (FCS last):

```
A6A2A69E404062 9C608682989861 03 F0 AA553214000000019A83F5C7B21F7511E82F5E94B9DE585109BF 6559
```

GSE frame PROVISION slot 0, key ID 1:

```
53514750 01 00 01 000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F E93364B5
```

Reproduce with the ground tools, using a scratch copy of the test key (mode 600):

```bash
python3 software/ground/tools/telecommand_builder.py TX_DISABLE --key-file test.key --counter 1 --ax25 SQSO-1 N0CALL
```

## 12. Verification

| Requirement | Method | Evidence |
|-------------|--------|----------|
| Forged, relabelled, truncated or extended frames rejected | Test | `test_cmd_auth.c`, `test_cmd_handler.c` |
| Replays rejected, including after reset, per key | Test | `test_replay_rejected_after_reset`, `test_recovery_counter_is_separate` |
| Only the recovery key rotates keys; rotation survives reset and power loss | Test (simulation) | `test_recovery_key_scope`, `test_rotate_replaces_key`, `test_interrupted_rotation_is_safe`, `test_key_rotate_end_to_end` |
| TX-OFF accepted in every state, without keys, and survives a power cycle | Test (simulation) | `test_tx_disable_accepted_in_every_state`, `test_no_key_material_tx_disable_still_silences`, `test_rf_control.c` |
| Receiver frame to TX-OFF end to end | Test (simulation) | `test_received_tx_disable_silences_station` |
| Keys load only through the umbilical | Test (simulation) | `test_locked_without_umbilical`, `test_poll_reads_umbilical_uart` |
| TX-OFF removes RF output at the antenna port | Test (HIL) | TP-RF-006, **pending hardware** |
| Ground and flight agree byte for byte | Test | `test_frame_matches_ground_tool`, `TestAx25`, `TestFrame` |

## 13. Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1 | 2026-10-15 | Initial draft with the flight implementation (findings SW-02, SW-03, SW-04) |
| 0.2 | 2026-10-15 | AX.25 receive path; key table, recovery key, KEY_ROTATE, GSE provisioning; TX-OFF without keys; generated dictionary; SET_MODE guards; SET_POWER |
