# Interface Control Document: Telecommand Authentication and TX-OFF

**Document ID**: SQSO-ICD-013
**Version**: 0.7 (Draft)
**Date**: 2026-10-27
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
| 47 CFR 97.211(a) | Only designated telecommand stations may send telecommand | Only holders of a key can produce an accepted command. The stations themselves are still to be designated (finding RG-03): at least two, by 2027-01-05 (`docs/ground/GROUND_STATION_DESIGN.md` §3; REGULATORY_ROADMAP F13) |
| 47 CFR 97.313(f); 2.106 US270 | 70 cm power near listed military radars | The Bay Area site is within 240 km of Beale AFB: 50 W PEP unless the antenna's lower half-power point stays above 10° elevation (up to 611 W ERP). Ground-station matter (SQSO-GND-001 §3) |

## 3. Uplink Framing

### 3.1 Link layer

Each telecommand is one AX.25 v2.2 UI frame (control `0x03`, PID `0xF0`) on the 435 MHz uplink. The destination address must be this station's call sign and SSID (`software/flight/include/station_id.h`). Frames addressed to any other station are dropped.

- **Call sign:** `KK6GDB-11`, the licensee's call sign (decision D1, 2026-10-15) with SSID 11.
- **Source address:** must be the sending telecommand station's own call sign.
- **Digipeater paths:** parsed, but not needed.

```
+------------------+----------------+------+------+------------------------+-----------+
| dest (7)         | src (7)        | 0x03 | 0xF0 | command frame (3.2)    | FCS (2,LE)|
+------------------+----------------+------+------+------------------------+-----------+
```

- **Receiver HAL contract (`hal_radio.h`):** each frame is delivered from the first address byte through the 2-byte FCS, with HDLC flags and bit stuffing already removed.
- **Defence in depth:** `software/rust/sqso-ffi/src/uplink.rs` checks the FCS (CRC-16/X.25) again even if the transceiver already did.
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
5. **Doppler.** Telecommand stations pre-correct the uplink to within ±1 kHz of the spacecraft receiver frequency.
   Uncorrected, the 435 MHz Doppler (up to ±10.4 kHz) takes the signal outside the receiver's ±7.5 kHz filter and
   nothing decodes (SQSO-LBA-001 v3.0 §3.2, §4.4).

#### 5.2.1 Two or more stations: counters from UTC (proposed)

Rule 3 above makes every station depend on one shared counter file, which is a single point of failure for the
loss-of-contact inhibit: the contact analysis (SQSO-GND-001 §5) shows that common-mode ground outages, not the link,
dominate the risk of tripping the 14-day inhibit. The onboard rule needs only that each accepted counter exceed the
last, so the stations can derive counters from time instead of sharing a file:

```
counter = 4 × (UTC seconds since 2028-01-01T00:00:00Z) + station_index      (station_index 0–3)
```

- Strictly increasing across stations as long as their clocks agree to better than the time between two stations'
  commands (passes from different sites are hours apart; GNSS or NTP time to ±1 s is ample). Two stations in the same
  second differ in the low bits.
- 2³² / 4 s is 34 years, so the counter cannot run out in the mission; the exhaustion rule (§5.1 rule 2) is untouched.
- A station whose clock runs fast burns counter space ahead of the others; its commands still pass, and the others'
  are rejected (`AUTH_FAIL`, counter not consumed) until real time catches up, which shows at once in the result.
  The station checks its clock against GNSS before each pass.
- No flight-software change. Ground-tool change: `telecommand_builder.py --counter-from-utc --station-index N`, still
  reserving (writing and fsyncing) the value before building the frame (rule 2). For the chair: `software/ground` is
  outside this document.
- After `KEY_ROTATE` the new key starts at 0, so UTC-derived counters are immediately valid.

## 6. TX-OFF

| Property | Behavior |
|----------|----------|
| Commands | `TX_DISABLE` (0x32) latches TX-OFF. `TX_ENABLE` (0x31) clears it. Neither takes arguments |
| Authentication | `TX_DISABLE`: operational **or** recovery key. `TX_ENABLE`: operational key only |
| **No usable key** | `TX_DISABLE` is accepted **without authentication** (the payload must still be well formed). Every other command, `TX_ENABLE` included, is rejected. Decision recorded 2026-10-15: TX-OFF can only reduce emissions, so the 97.207(b) obligation never depends on key material. Once TX-OFF is latched in this state, the station stays silent until keys are re-provisioned |
| Allowed states | Every operating state: BOOT, SAFE, DETUMBLE, IDLE, ACTIVE, EMERGENCY |
| Order of operations | `TX_DISABLE` drives `GPIO_PIN_RF_TX_EN` low first, then saves the latch. The transmitter is off even if the latch cannot be saved; the command still reports `SUCCESS`, and the fault log records the failure |
| Persistence | Two-slot record with CRC-32 (`nvm_record`) in `FLASH_REGION_RF_CONTROL`, base 0 |
| After reset | The latch is restored at boot, before anything can transmit (`rf_control_init()` in the executive, `software/rust/sqso-ffi/src/executive.rs`) |
| Blank record (first boot) | Transmit permitted |
| Corrupt or unreadable record | Transmit **inhibited** (fail safe) |
| SAFE mode, EPS `beacon_enabled` | Cannot override the latch. `beacon_should_transmit()` and `beacon_transmit()` refuse while TX-OFF is latched |
| Command receiver | Not switched by TX-OFF or by EPS `radio_enabled`. It stays powered so the station can still be commanded |
| Hardware path | `RF_TX_EN` drives the PA load-switch enable and is pulled down, so a reset or unpowered OBC leaves the PA off (SQSO-ICD-003 §4.2; `hardware/obc/OBC_DESIGN.md` Appendix A.5.4). Verified by TP-RF-006 (pending hardware) |

A replayed `TX_ENABLE` cannot undo a later `TX_DISABLE`. `test_replayed_tx_enable_cannot_undo_tx_off` covers this.

### 6.2 Loss-of-Contact Inhibit

If no authenticated telecommand is received for **N days**, every transmitter is inhibited and the receiver stays on (`software/rust/sqso-fsw/src/contact_monitor.rs`, SYS-205; IARU form item 7.9).

| Property | Behavior |
|----------|----------|
| N | Persisted. Set by `SET_CONTACT_TIMEOUT` (0x07) to 1–90 days. **30 days at launch**, so a late first contact does not find the spacecraft silent; operations set **14 days** once commissioning is complete (decision D3; values go to the FCC consultation) |
| Contact | An authenticated frame (operational or recovery key) whose counter was committed, whatever the command's result, including `NOT_ALLOWED` |
| Not contact | Forged or replayed frames; the keyless `TX_DISABLE` (Section 6); authentic frames rejected before their counter is spent (for example a wrong argument length). These are replayable, so counting them would let a replay hold off the inhibit |
| Lifting | The next contact lifts the inhibit. It never clears the TX-OFF latch, and `TX_ENABLE` does not lift the inhibit except by being contact itself. Transmission needs both inhibits clear |
| Hardware path | The same `RF_TX_EN` line as TX-OFF: `rf_control` drives it high only when neither inhibit is set |
| Time base | No reset-proof clock exists, so `contact_monitor_tick()` accumulates seconds without contact from the main loop. The counter is persisted every 60 s (1,440 writes a day to FRAM, which is rated for 10^14 writes) |
| Boot | The inhibit is recomputed from the counter before `rf_control_init()` drives the line, so a silent spacecraft stays silent through a reset |
| Errors make it come earlier | Each boot that finds a counter adds 60 s, so a reset loop still reaches N. A corrupt counter counts as timed out. Blank (first boot) starts at 0 |
| Commissioning | The count starts at first boot, about 45 minutes before the RF silence ends; that difference is negligible against N. If first contact takes longer than N, the spacecraft falls silent until commanded. That is intended, and operations must plan for it |
| Passivation (M) | Not implemented. Proposed: after a further 180 days inhibited, never in the first year after launch, permanent passivation (BASELINE §7). For the FCC consultation (D3) |
| Not implemented | The M-day autonomous end-of-mission passivation in IARU 7.9 and ODAR 4.4-2. It is irreversible, needs the passivation hardware path (SYS-800), and is a larger decision than D3. The telecommanded passivation is §6.4 |

### 6.3 Post-Deployment RF Silence

No RF signal may be generated or transmitted until 45 minutes after deployment, and a deployment-switch toggle restarts the timer (CDS Rev 14.1 §2.4.5, §2.3.2.4; SYS-224; `software/rust/sqso-fsw/src/rf_silence.rs`).

| Property | Behavior |
|----------|----------|
| During the silence | The whole radio is unpowered (`GPIO_PIN_RADIO_EN` low) as well as the transmitter (`RF_TX_EN` low). **This is the one exception to "the receiver is always on"**: no telecommand can be received in this window. That is acceptable because nothing can transmit either. The reading of "generate any signal" as including the receiver is conservative and TBR with the launch provider |
| No command shortens it | No telecommand clears or shortens the silence. An authenticated `TX_ENABLE` clears the TX-OFF latch, but the radio stays off (`test_tx_enable_cannot_shorten_silence`) |
| Power-on, brownout, pin or unknown reset | Restart the count from 0. The deployment switches are power inhibits, so deployment and any switch toggle arrive as a power-on reset |
| Watchdog or software reset | Resume from the persisted count. The count is saved every 60 s and lags real time, so the silence only gets longer |
| Switch reads stowed while powered | Restart from 0, even after completion (re-integration while powered) |
| Blank or corrupt record | Count from 0 |
| Completion | Persists across later resets of any kind |
| `hal_reset_cause()` | Read once after boot; the MCU's sticky reset flags are cleared and the result cached (`hal_reset.h`) |
| Bench testing | Completes the silence, and completion persists. So **GSE `ARM_FLIGHT` (0x04, umbilical only; `key_provision.py arm-flight`) must be the last powered action before integration**. It re-arms the silence and the deployment sequence together (TP-FAT-002 §6.7, PRE_SHIP_CHECKLIST). ARMED does not count while powered; the count starts at the next boot. Any power-up after ARM that lasts 25 minutes or more requires re-arming (the deployment wait is 30 minutes). The state is printed at every boot |
| Switch polarity | Defined once in `software/rust/sqso-ffi/src/separation_switch.rs` (pull-up, high = stowed), shared with the deployment sequence, and **TBR** against the separation-switch schematic. Inverted polarity on orbit would restart the count forever: fail safe, but mission-ending. The pre-ship check confirms that released switches read as released |
| Target | Same time-base dependency as Section 6.2 (SW-01) |
| Deployables | The 30-minute deployable timer (`software/rust/sqso-fsw/src/deployment.rs`, SYS-221) follows the same reset rules and is persisted; burns are bounded across resets (finding SW-10) |

`rf_control` combines three independent inhibits: the TX-OFF latch, loss of contact, and RF silence. The transmitter is enabled only when all three are clear. Only the silence removes radio power.
| Target | `contact_monitor_tick()` needs a monotonic time source. `smart_qso_now_ms()` is not implemented on target yet (SW-01) |

### 6.4 End-of-Life Passivation by Telecommand (proposed, not implemented)

BASELINE §5 and the pre-space notification (97.207(g)(1)(iii)) say the battery charge path is permanently
disconnected at end of life and the transmitters disabled. **No telecommand does this today:** the command dictionary
(Section 7) has no passivation command; the mission simulation fires a modelled passivation latch when its end-of-life
`TX_DISABLE` is delivered (`software/rust/sqso-sil/src/mission.rs`), which the flight software does not have. Proposed
design, for a change request (flight software and EPS are outside this document):

| Item | Proposal |
|------|----------|
| Commands | `PASSIVATE_ARM` (0x34, operational key, 4-byte argument: the ASCII arming code `EOL!`) then `PASSIVATE_EXECUTE` (0x35, **recovery key**, no argument) within 600 s of the arm, in the same contact. Two keys from two custodians: the two-person rule for an irreversible act |
| Order of execution | `TX_DISABLE` latch first (Section 6), then the EPS non-resettable charge-disconnect (SYS-800, EPS design), then the loads are left on to discharge the pack. The receiver goes dark when the pack is exhausted |
| Allowed states | Every state except BOOT; refused in the first 365 days after deployment unless the arm argument is `EOL!` and a GSE-set flag allows early passivation (none by default) |
| Telemetry | The beacon carries `M=PASSIVE` between the arm and the execute; arming times out silently |
| Verification | Test on the EM with a sacrificial charge-disconnect device; the latch is proven non-resettable by power cycling and by every telecommand |

Until this is implemented, the pre-space notification must not claim passivation by telecommand
(`docs/regulatory/filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md` Rev A.1, (iii)).

## 7. Command Dictionary

The argument length must match the dictionary exactly. If it doesn't, the command is rejected with `INVALID_PARAM` and its counter is not consumed.

**Single source:** `software/flight/command_dictionary.json`. `scripts/gen_command_dictionary.py` generates from it:

- the Rust `Command` enum, `software/rust/sqso-protocol/src/dictionary.rs`. The flight argument check, the key choice and
  the command dispatcher all use it, and the dispatcher is an exhaustive `match`: a command added to the JSON does not
  build until it has a handler. The C table `g_cmd_dictionary` is built from it at compile time;
- the C command-ID enum in `software/flight/include/cmd_dictionary.h`;
- the ground module, `software/ground/tools/command_dictionary.py`.

CI runs the generator with `--check` and fails if any output is stale. `test_every_dictionary_command` and `TestDictionary` check both sides independently.

| ID | Name | Args (bytes) | Key | Notes |
|----|------|--------------|-----|-------|
| 0x00 | NOP | 0 | operational | |
| 0x01 | RESET | 0 | operational | Logged only: no build calls a hardware reset yet |
| 0x02 | SET_MODE | 1 | operational | 0 = SAFE, 1 = IDLE, 2 = ACTIVE. Goes through the state-machine transition table; SAFE → ACTIVE needs IDLE first (`NOT_ALLOWED` otherwise) |
| 0x03 | GET_STATUS | 0 | operational | Returns 4 bytes |
| 0x05 | CLEAR_FAULTS | 0 | operational | |
| 0x06 | KEY_ROTATE | 1 | **recovery** | Activates spare operational slot 0–3 and revokes the current key (Section 9.3). Allowed in every state |
| 0x07 | SET_CONTACT_TIMEOUT | 2 | operational | Loss-of-contact timeout in days, big-endian, 1–90 (Section 6.2; default and range are placeholders, D3) |
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
| 6 | Counter persisted for the authenticating key (`TX_DISABLE` continues regardless). A keyed frame here is **contact** (Section 6.2) | `EXEC_FAIL` | In memory only |
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

Keys are loaded through the GSE umbilical UART by `software/rust/sqso-ffi/src/gse_provision.rs`.

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
| 0x04 ARM_FLIGHT | Re-arm the 45-minute post-deployment silence (Section 6.3) and the deployment sequence (`deployment.h`); last powered action before integration |

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
| SHA-256, HMAC-SHA-256 (`software/rust/sqso-ffi/src/sha256.rs`) | Implemented; FIPS 180-4 and RFC 4231 vectors (9 tests) |
| Two-slot records with base offsets (`software/rust/sqso-fsw/src/nvm.rs`) | Implemented; power-fail and corruption cases (12 tests) |
| Authentication, per-key counters, key table, rotation (`software/rust/sqso-fsw/src/keys.rs`) | Implemented (25 tests, including interrupted rotation and corrupt state) |
| GSE provisioning (`software/rust/sqso-ffi/src/gse_provision.rs`) | Implemented (6 tests; the umbilical gate was mutation-checked, on the C and again on the Rust) |
| Latched TX-OFF, power level (`software/rust/sqso-fsw/src/rf_control.rs`), beacon gating | Implemented (11 tests) |
| The inhibits again at the downlink, when a frame is handed to the TNC (`software/rust/sqso-fsw/src/downlink.rs`) | Implemented (2026-10-25, CR-2026-005; `test_downlink.c`, and `Downlink_TNC_silent`: nothing reaches the TNC during the RF silence) |
| Loss-of-contact inhibit (`software/rust/sqso-fsw/src/contact_monitor.rs`) | Implemented (10 tests, plus 4 handler tests; the contact hook's gate and placement were mutation-checked, on the C and again on the Rust) |
| Post-deployment RF silence (`software/rust/sqso-fsw/src/rf_silence.rs`), GSE ARM | Implemented (10 tests plus 1 GSE test; the reset-cause rule, completion persistence and switch reset were mutation-checked, on the C and again on the Rust) |
| Command path (`software/rust/sqso-ffi/src/cmd_handler.rs`) | Implemented (22 tests, including every dictionary command, KEY_ROTATE, SET_MODE guards and contact detection) |
| AX.25 and receive path (`software/rust/sqso-protocol/src/ax25.rs`, `software/rust/sqso-ffi/src/uplink.rs`) | Implemented; end to end from receiver frame to TX-OFF (7 tests; the callsign filter was mutation-checked, on the C and again on the Rust) |
| Ground tools (`telecommand_builder.py`, `key_provision.py`) | Implemented; byte-identical to the flight vectors |
| Rust implementation | Since 2026-10-24 every item above is Rust (`software/rust/README.md`). The C test suites counted here run unchanged against them, the parsers are fuzzed, and a differential fuzz against the former C agreed on 8.3 million inputs. The mutation checks in this table were first run on the C; on 2026-10-24 they were repeated on the Rust: every mutant is killed by a named test (`software/rust/mutants/RESULTS-targeted.md`; rerun with `software/rust/mutants/targeted.py`). Whole-crate mutation testing: `software/rust/mutants/` |
| **Target receiver driver behind `hal_radio.h`** | **Not implemented.** Waits on transceiver selection (SQSO-ICD-003's SPI command set is a placeholder) |
| **`RF_TX_EN` removes PA power at the antenna port** | **Not verified.** TP-RF-006 is written; it needs EM hardware |
| **SET_POWER applied to the PA** | **Not implemented.** The level is persisted and reported; the actuator interface is TBD in the RF design |
| FRAM driver for `FLASH_REGION_AUTH` and `FLASH_REGION_RF_CONTROL` | Allocated in SQSO-OBC-002 §3.3; simulation HAL only |
| Station call sign | `KK6GDB-11` (licensee KK6GDB, decision D1) |
| Designated telecommand stations | **None agreed** (D1); at least two by 2027-01-05 |
| Passivation telecommand (§6.4) | **Not implemented**; proposed design only |
| Counters from UTC for several stations (§5.2.1) | Proposed; ground tool only |
| Loss-of-contact N and M | N: mechanism implemented; value a placeholder (D3). M (autonomous passivation): **not implemented** |
| Monotonic time source on target | **Not implemented** (SW-01); the loss-of-contact and RF-silence timers depend on it |
| Reset-cause register read on target (`hal_reset.h`) | **Not implemented**; simulation only |
| Deployment-switch polarity | TBR against the schematic (SW-10) |

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

AX.25 UI frame `KK6GDB-11` ← `N0CALL` carrying TX_DISABLE counter 1 (FCS last):

```
96966C8E888476 9C608682989861 03 F0 AA553214000000019A83F5C7B21F7511E82F5E94B9DE585109BF C8B6
```

GSE frame PROVISION slot 0, key ID 1:

```
53514750 01 00 01 000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F E93364B5
```

Reproduce with the ground tools, using a scratch copy of the test key (mode 600):

```bash
python3 software/ground/tools/telecommand_builder.py TX_DISABLE --key-file test.key --counter 1 --ax25 KK6GDB-11 N0CALL
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
| Loss-of-contact inhibit: timeout, contact rules, reset loop, corrupt counter | Test (simulation) | `test_contact_monitor.c`, `test_authentic_command_restores_contact`, `test_unauthenticated_frames_are_not_contact` |
| RF silence: 45 min, reset rules, switch toggle, ARM, no command shortens it | Test (simulation) | `test_rf_silence.c`, `test_arm_flight` |
| No RF for 45 min across a mid-countdown reset, at the antenna port | Test (HIL) | TP-RF-006 §8.8, **pending hardware** |
| TX-OFF removes RF output at the antenna port | Test (HIL) | TP-RF-006, **pending hardware** |
| Ground and flight agree byte for byte | Test | `test_frame_matches_ground_tool`, `TestAx25`, `TestFrame` |

## 13. Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1 | 2026-10-15 | Initial draft with the flight implementation (findings SW-02, SW-03, SW-04) |
| 0.2 | 2026-10-15 | AX.25 receive path; key table, recovery key, KEY_ROTATE, GSE provisioning; TX-OFF without keys; generated dictionary; SET_MODE guards; SET_POWER |
| 0.3 | 2026-10-15 | Loss-of-contact inhibit (Section 6.2) and SET_CONTACT_TIMEOUT |
| 0.4 | 2026-10-15 | Post-deployment RF silence (Section 6.3) and GSE ARM_RF_SILENCE |
| 0.5 | 2026-10-15 | GSE 0x04 renamed ARM_FLIGHT: arms the RF silence and the deployment sequence together |
| 0.6 | 2026-10-16 | OBC-to-RF ICD cited by its own number, SQSO-ICD-003 (was SQSO-ICD-005, the Space-to-Ground ICD), and the OBC GPIO table by its new location after the OBC design merge (finding PI-17). No technical change |
| 0.6, note | 2026-10-24 | The C files named in §6, §7 and §10 were ported to Rust; references now name the Rust files (`software/rust/README.md`). The test suites named are unchanged and run against the Rust. No content change |
| 0.7 | 2026-10-27 | CDR review (communications panel): telecommand-station status and the 70 cm power limit near Beale AFB (§2); Doppler pre-correction rule (§5.2 rule 5); UTC-derived counters for several stations (§5.2.1, proposed); end-of-life passivation by telecommand (§6.4, proposed: no such command exists, which the pre-space notification must reflect); status rows (§10) |
