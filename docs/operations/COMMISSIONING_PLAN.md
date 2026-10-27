# SMART-QSO Commissioning Plan

**Document ID**: SQSO-COMM-001
**Version**: 2.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft for CDR. Commands are the flight dictionary (`software/rust/sqso-protocol/src/dictionary.rs`,
17 commands); responses are as `software/rust/sqso-ffi/src/cmd_handler.rs` builds them. Steps that need telemetry
or commands the flight software does not have are marked **blocked** with the RID that would unblock them.

## 1. Purpose and scope

Commissioning takes the spacecraft from separation to primary operations in about 14 days. Each step below has entry
criteria, the commands, the telemetry that proves it and an exit criterion. The timeline, the pass budget and the
contingencies are in `docs/operations/MISSION_TIMELINE.md` (SQSO-MTL-001) §5–§8; the ConOps is `mission/CONOPS.md` §2.

Rules for every step:

1. **Command only in a pass in which the station has decoded a beacon** (the simulation's finding SIL-10) and above
   15° elevation. Bay Area budget: 2.5 such passes a day, about 21 min of commanding a day (MISSION_TIMELINE §6, §10).
2. Every command is authenticated with the active operational key (HMAC-SHA-256, persisted counter; SQSO-ICD-013)
   except `TX_DISABLE`, which is accepted without a key when no key is usable. The recovery key is never used in
   commissioning unless a key has to be rotated.
3. `TX_DISABLE` is ready to send on every pass. It stops all transmission (beacon and transponder) and is latched in
   non-volatile memory; only an authenticated `TX_ENABLE` clears it.
4. Each step runs first from one designated station and is repeated from the second before the step is closed
   (CONOPS §2: telecommand checkout from each designated station).
5. Results are compared with the simulation's prediction for the same day (`sqso_sil --scenario commissioning`);
   the simulation is also the rehearsal environment for C-0.

### 1.1 Telemetry available

| Source | Content | When |
|--------|---------|------|
| Beacon (AX.25 UI, 1200 bps) | Call sign; text (template or generated); tail `T=<s since boot> M=<SAFE/IDLE/ACTIVE> SOC=<%> BV=<V> SUN=<0/1> RF=<0/1> QSO=<count>` (`docs/BEACON_SPEC.md` §3) | Every 60 s (ACTIVE), 120 s (IDLE), 180 s (SAFE) |
| `GET_STATUS` (0x03) | 4 bytes: operational state, power mode, payload enabled, thermal fault | On command |
| `EPS_GET_TELEMETRY` (0x13) | 3 bytes: battery voltage (0.1 V), SOC (1 %), power mode | On command |
| `ADCS_GET_ATTITUDE` (0x23) | 2 bytes: detumbled flag, sun acquired flag | On command |

Not on the downlink in any form: temperatures, battery current, array power, the deployment state and sense lines,
the RF-silence state, the boot count and reset cause, the fault log, the body rates, the loss-of-contact timeout, the
GNSS fix. The binary housekeeping, EPS and ADCS frames of `sqso-fsw/src/telemetry.rs` exist but the executive sends
them only to the Jetson over the UART. Several exit criteria below therefore rest on proxies (RID LV-27).

## 2. Commissioning sequence

| Step | Name | Nominal day | Depends on |
|------|------|-------------|------------|
| C-0 | Ground readiness | L-2 M to S | — |
| C-1 | Acquisition and identification | S to S+1 d | C-0 |
| C-2 | First authenticated command | First commanding pass | C-1 |
| C-3 | Cessation of emissions (TX-OFF) proof | Same or next pass | C-2 |
| C-4 | Health and power characterization | Days 1–3 | C-2 |
| C-5 | Deployables and attitude | Days 1–4 | C-2 |
| C-6 | Beacon conformance | Days 3–5 | C-3 |
| C-7 | Transponder | Days 5–8 | C-3, C-4 |
| C-8 | AI payload (Jetson) and fallback | Days 8–11 | C-4 |
| C-9 | GNSS and operator ephemeris | From first contact | C-1; **blocked** (LV-04) |
| C-10 | Operations validation and SAFE drill | Days 11–13 | C-3 to C-8 |
| C-11 | Commissioning review and close-out | Day 14 | All |

## 3. Steps

### C-0 Ground readiness

| | |
|---|---|
| Entry | Launch contract and integration date known; telecommand stations designated (D1); keys loaded in the flight unit and held by the stations (SQSO-ICD-013) |
| Actions | Space-Track organization account, satellite-operator status, operator contact public (UG v11 §9.5.7.1); SpaceX Space Traffic Coordination onboarding (space-safety-onboarding@spacex.com) and a test upload; TraCSS registration; pre-launch OPM loaded in the tracking software; beacon decoder and frame logger at each station and on SatNOGS; command tool with counters synchronized to the flight unit's at `ARM_FLIGHT`; orbit-determination pipeline (or the third-party service) tested; rehearsal of C-1 to C-3 against `sqso_sil` |
| Proof | Rehearsal log; Space-Track and SpaceX test acknowledgements |
| Exit | Ground readiness review passed at L-2 M; the integrator's ephemeris requirement accepted in writing (VI-LV-13) |

### C-1 Acquisition and identification

| | |
|---|---|
| Entry | Separation report and SpaceX state vector received; S+45 min passed |
| Commands | None |
| Proof | Beacon decoded with source KK6GDB-11 and the call sign at the start of the text; tail values: `M=` IDLE or ACTIVE (SAFE or a missing beacon means a low separation SOC, MISSION_TIMELINE §8.4), `SOC=` ≥ 40, `BV=` 14.4–16.4 V, `SUN=` alternating with the orbit; `T=` consistent with the separation time (the beacon's `T` counts seconds since boot, so T ≈ time since separation unless the OBC reset); Doppler curve matching the SpaceX state vector within the dispersion |
| Expected | First beacon over the Bay Area after S+0.7 to S+12.2 h (median S+5.2 h); anywhere in the network after about S+1.8 h (simulation) |
| Exit | Beacons decoded on ≥ 2 passes; candidate object identified among the 18th SDS elements by Doppler; association reported to the integrator (SQSO-REG-015 §2) |
| If not | MISSION_TIMELINE §8.6 |

### C-2 First authenticated command

| | |
|---|---|
| Entry | C-1 on this pass; elevation ≥ 15°; ≥ 3 min of pass left |
| Commands | `GET_STATUS` (0x03); then `NOP` (0x00) |
| Proof | Response from `GET_STATUS`: operational state and power mode match the beacon's `M=`; payload flag; thermal fault 0. Counter accepted (no authentication failure) |
| Exit | Both commands executed from station 1, then from station 2 on a later pass |
| If not | Resend once per pass; check the counter window and the key slot; after 3 passes, follow the key-recovery path (SQSO-ICD-013) |

### C-3 Cessation of emissions (TX-OFF) proof

Required before the transponder is opened to users (CONOPS §2) and the basis of the 97.207(b) certification.

| | |
|---|---|
| Entry | C-2 executed on an earlier pass; a pass with ≥ 6 min above 15° and both stations able to hear |
| Commands | `SET_BEACON` (0x30) 30 s, so that silence is visible within the pass; `TX_DISABLE` (0x32); wait ≥ 2 beacon intervals; `TX_ENABLE` (0x31); `SET_BEACON` back to the mode default |
| Proof | No beacon and no transponder output for ≥ 60 s after the `TX_DISABLE` acknowledgement (both stations listening); beacons resume after `TX_ENABLE`; `RF=1` in the tail afterwards |
| Exit | TX-OFF and TX-ON demonstrated once from each station |
| If not | If `TX_ENABLE` is lost the spacecraft stays silent: send it on the next pass (the receiver stays on, and the latch persists through resets) |

### C-4 Health and power characterization

| | |
|---|---|
| Entry | C-2 |
| Commands | `EPS_GET_TELEMETRY` (0x13) and `GET_STATUS` on every commanding pass |
| Proof | SOC and battery voltage on each pass and in every decoded beacon, plotted against the sunlit/eclipse phase from the orbit; compared with the simulation (SOC 92 % at end of charge, depth of discharge ≤ 2.1 % per eclipse, about 2.6 Wh drawn per orbit against 2.25 Wh used) |
| Exit | 3 days with SOC ≥ 80 % at each pass, no SAFE or EMERGENCY entry (`M=` never SAFE), `BV` at the 4.10 V/cell end of charge (16.4 V) in sunlit passes; thermal fault 0 |
| Gap | Temperatures, battery current and array power are not on the downlink (LV-27): the battery charge window (0–45 °C) and the heater cannot be verified from orbit |

### C-5 Deployables and attitude

| | |
|---|---|
| Entry | C-2 |
| Commands | `ADCS_GET_ATTITUDE` (0x23); `ADCS_DETUMBLE` (0x21) only if the detumbled flag stays 0 beyond S+5 h |
| Proof | Detumbled = 1 and sun acquired = 1; `SUN=1` through each sunlit pass; beacon signal strength against range consistent with a deployed 2 m monopole (≈ +6 dB margin at 5°, BASELINE §7); SOC recovering at about 8 % per sunlit orbit after a discharge, which needs the wings |
| Exit | Both flags 1 on 3 consecutive passes; link and charge proxies consistent with antennas and wings deployed |
| Gap | The deployment state is not on the downlink, and a failed element cannot be retried by command (LV-07, LV-27). If the proxies show a failed wing, continue: the simulation's wings-failed case is power-positive (MISSION_TIMELINE §8.2) |

### C-6 Beacon conformance

| | |
|---|---|
| Entry | C-3 |
| Commands | `SET_POWER` (0x33) 0 (0.5 W) and 1 (1 W) on separate passes; `SET_BEACON` 10, 60 and 120 s |
| Proof | Recorded spectrum at two stations: occupied bandwidth within 10K4F2D, deviation, carrier within the coordinated channel plus Doppler; cadence matches the command; call sign in every frame |
| Exit | Both power levels and the cadence within the licence and the IARU coordination; return to `SET_POWER` 0 |

### C-7 Transponder

| | |
|---|---|
| Entry | C-3 and C-4 passed; ACTIVE (`SET_MODE` (0x02) 2 if not already); sunlit pass with SOC ≥ 55 % |
| Commands | `SET_MODE` 2; team uplink with the correct CTCSS tone, then without it, during a Bay Area pass |
| Proof | Repeated audio on the downlink only with a valid CTCSS carrier; `QSO=` count in the tail increments; the transponder stops in eclipse and below 55 % SOC (observe on a pass that crosses the terminator); keyed time ≤ 25 % of an orbit (simulation: ≤ 14.4 %, about 80 min a day) |
| Exit | Gates observed working on 2 passes from each station; public announcement of the transponder |
| As built | The flight software enables the transponder by itself at S+53 min, before C-2 (LV-01). There is no command to switch the transponder alone: `SET_MODE` 1 (IDLE) or `TX_DISABLE` are the only ways to stop it (LV-14) |

### C-8 AI payload (Jetson) and fallback

| | |
|---|---|
| Entry | C-4; ACTIVE; sunlit; SOC ≥ 55 % |
| Commands | `PLD_DISABLE` (0x41); observe; `PLD_ENABLE` (0x40) |
| Proof | `GET_STATUS` payload flag follows the command; with the payload disabled the beacon text is a fixed template and the telemetry tail is unchanged; with it enabled, generated lines appear in batches, each passing the ground copy of the content filter (`docs/BEACON_SPEC.md` §6) |
| Exit | Fallback demonstrated; 100 generated beacons decoded with no filter failure |
| Gap | The energy per accepted beacon (the D5 gate, ≤ 0.1 Wh) cannot be measured from orbit: no Jetson energy telemetry (LV-27). As built, batches already run from S+53 min |

### C-9 GNSS and operator ephemeris

| | |
|---|---|
| Entry | First contact (C-1) |
| Commands | **None exist** for the receiver (LV-04, LV-14) |
| Proof | Operator ephemeris (CCSDS OEM with covariance) accepted by Space-Track and SpaceX Space Traffic Coordination within 24 h of first contact, after every pass and at least every 24 h until the 18th SDS catalogs the object (UG v11 §9.5.7.2) |
| Exit | NORAD ID assigned, positional data public, operator contact public; object claimed within 30 days |
| Status | **Blocked** for onboard GNSS. Until LV-04 closes, the third-party service contracted before integration delivers the ephemeris |

### C-10 Operations validation and SAFE drill

| | |
|---|---|
| Entry | C-3 to C-8 |
| Commands | `SET_MODE` 0 (SAFE); observe 2 beacons at 180 s; `SET_MODE` 1 (IDLE); `SET_MODE` 2 (ACTIVE) on the next pass; `CLEAR_FAULTS` (0x05) only after the fault log is understood (it is not on the downlink: LV-27) |
| Proof | `M=` follows each command; SAFE: template text, no transponder; the transition table refuses SAFE → ACTIVE directly (`NOT_ALLOWED` response) |
| Exit | 24 h of autonomous operation with no SAFE or EMERGENCY entry; shift handover rehearsed |

### C-11 Commissioning review and close-out

| | |
|---|---|
| Entry | C-1 to C-8 and C-10 closed; C-9 closed or covered by the third party |
| Commands | `SET_CONTACT_TIMEOUT` (0x07), 2-byte big-endian days = 14 (D3); the command is itself a contact, so the count restarts |
| Proof | Command acknowledgement. The timeout value is not readable back (LV-27) |
| Exit | Review signed by the operations lead, the systems engineer and the licensee; primary operations start |

## 4. Anomaly handling

| Anomaly | Likely cause | Response |
|---------|--------------|----------|
| No beacon at the first Bay Area pass | Pass geometry (median first pass S+5.2 h); low SOC; antenna stowed | MISSION_TIMELINE §8.6 |
| `M=SAFE` at first contact | Separation SOC below 25 % | Wait: the simulation reaches IDLE within S+4–8 h |
| Weak signal at high elevation | Antenna not deployed | No commanded retry exists (LV-07); characterize the link |
| Command not executed | Counter window, key slot, link | C-2 "If not" |
| Unexpected SAFE | Fault, low SOC, thermal | `GET_STATUS`, `EPS_GET_TELEMETRY`; return through IDLE after the cause is understood |
| Interference or a stuck transmitter | — | `TX_DISABLE` immediately; investigate before `TX_ENABLE` |

## 5. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR launch and ConOps review | Rewritten step by step (C-0 to C-11) with entry criteria, the commands of the flight dictionary, the telemetry that proves each step and its exit criterion; TX-OFF proof added before the transponder; ephemeris step added; the RF silence is 45 min; the separation SOC and pass statistics from the simulation and the pass analysis. Version 1.0 used commands that do not exist (`JETSON_POWER`, `AI_ENABLE`, `FORCE_FALLBACK`, `ECHO_TEST`, `RELEASE_SAFE`, `SET_TX_POWER`) and assumed a 30-minute RF timer, a > 95 % SOC at deployment and telemetry the downlink does not carry |
