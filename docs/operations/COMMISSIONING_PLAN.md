# SMART-QSO Commissioning Plan

**Document ID**: SQSO-COMM-001
**Version**: 2.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft for CDR. Commands are the flight dictionary as of flight software round 3
(`software/flight/command_dictionary.json`, 30 commands; framing, keys and results in SQSO-ICD-013
`docs/interfaces/TELECOMMAND_AUTHENTICATION.md` §7–§8; summary in `docs/operations/COMMAND_DICTIONARY.md` §1.1).
Telemetry is the beacon tail, the housekeeping frames HK1 and HK2, the command acknowledgement and the stored records
(`docs/operations/TELEMETRY_DICTIONARY.md` §4.3–§4.6). Steps that still depend on something the design does not provide
are marked **blocked** with the RID that would unblock them.

## 1. Purpose and scope

Commissioning takes the spacecraft from separation to primary operations in about 14 days. Each step below has entry
criteria, the commands, the telemetry that proves it and an exit criterion. The timeline, the pass budget and the
contingencies are in `docs/operations/MISSION_TIMELINE.md` (SQSO-MTL-001) §5–§8; the ConOps is `mission/CONOPS.md` §2.

Rules for every step:

1. **Command only in a pass in which the station has decoded a beacon** (the simulation's finding SIL-10) and only
   **above 10° elevation** (chair decision C-14: the station radiates ≥ 64 dBm EIRP toward the spacecraft; the PTT gate
   refuses to key below 10°). In the **degraded mode** (variant A with its amplifier out of service) command only above
   **12°** (SQSO-GND-001 §3.1, §3.1a; SQSO-LBA-001 §6.1). Bay Area budget at 10°: about 3 commanding passes a day
   (2.97 at Santa Clara; 16.6 min a day above 10°; SQSO-GND-001 §4, SQSO-MTL-001 §10).
2. **Uplink frequency.** Pre-correct the uplink Doppler to ±1 kHz, and before the first command of each pass apply the
   receiver's frequency offset estimated from the measured downlink: the downlink and the receiver share one TCXO, so
   the downlink's offset from its nominal (after removing the predicted Doppler), scaled by f_up / f_down, is the
   receiver's offset at the uplink (RID RFB-05: up to 2.1 ppm after 2 years, 0.9 kHz at 435.5 MHz; SQSO-GND-001
   GND-R5).
3. Every command is authenticated with the active operational key (HMAC-SHA-256, persisted counter; SQSO-ICD-013)
   except `TX_DISABLE`, which is accepted without a key when no key is usable. The recovery key is never used in
   commissioning unless a key has to be rotated. `PASSIVATE_ARM` (0x70) and `PASSIVATE_EXECUTE` (0x71) are never sent
   in commissioning (`docs/operations/END_OF_LIFE_PLAN.md`).
4. `TX_DISABLE` is ready to send on every pass. It stops all transmission (beacon and transponder) and is latched in
   non-volatile memory; only an authenticated `TX_ENABLE` clears it. It is the one command with no acknowledgement:
   silence is its response.
5. Each command counted as contact is acknowledged on the downlink (`ack KK6GDB-11: C=<counter> R=<result> N=<days>`;
   `R=0` executed, `N=0` counted as contact). A command is closed by its acknowledgement and the HK1 field it changes,
   not by the uplink log.
6. Each step runs first from one designated station and is repeated from the second before the step is closed
   (CONOPS §2: telecommand checkout from each designated station).
7. Results are compared with the simulation's prediction for the same day (`sqso_sil --scenario commissioning`);
   the simulation is also the rehearsal environment for C-0.

### 1.1 Telemetry available

| Source | Content | When |
|--------|---------|------|
| Beacon (AX.25 UI, 1200 bps) | Call sign; text (template or generated); tail `T=<s since boot> M=<SAFE/IDLE/ACTIVE> SOC=<%> BV=<V> SUN=<0/1> RF=<0/1> QSO=<count>` (`docs/BEACON_SPEC.md` §3) | Every 60 s (ACTIVE), 120 s (IDLE), 180 s (SAFE), jittered ±5 s |
| HK1 housekeeping frame | `N UP BOOT RST MODE HOLD XP TX SOC VB IB PA TB TO TP TJ DEP RFS CT CS FC FL W AM MR GF GS PS RX CN ST` (TELEMETRY_DICTIONARY §4.4): commissioning hold, transponder, temperatures, battery current, array power, deployment state, RF-silence state, contact timeout and count, fault log, body rates, GNSS fix, passivation and EOL_STATE, receiver recoveries, stored-record numbers | After every fifth beacon, and on `GET_HOUSEKEEPING` (0x09) |
| HK2 housekeeping frame | Command counters, watchdog resets, transponder keyed time, Jetson bursts with their energy and lines, mission clock, gyro in use, rail trips (TELEMETRY_DICTIONARY §4.4.1) | 5 s after each HK1 |
| Command acknowledgement | `ack KK6GDB-11: C= R= N=` (§4.5) | After each command counted as contact |
| Stored housekeeping | `HKD` frames: 5-minute records for 7.1 days, 60 s records for 6.8 h and around the last three events (§4.6) | On `DUMP_TLM` (0x0A) |

`GET_STATUS` (0x03), `EPS_GET_TELEMETRY` (0x13) and `ADCS_GET_ATTITUDE` (0x23) are executed but their replies are not
downlinked; `GET_HOUSEKEEPING` gives the same values in an HK1 frame (SQSO-ICD-005 §5.5). Version 2.0's list of values
absent from the downlink (RID LV-27) is closed by HK1 and HK2.

## 2. Commissioning sequence

| Step | Name | Nominal day | Depends on |
|------|------|-------------|------------|
| C-0 | Ground readiness | L-2 M to S | — |
| C-1 | Acquisition and identification | S to S+1 d | C-0 |
| C-2 | First authenticated command | First commanding pass | C-1 |
| C-3 | Cessation of emissions (TX-OFF) proof | Same or next pass | C-2 |
| C-4 | Health and power characterization | Days 1–3 | C-2 |
| C-5 | Deployables and attitude | Days 1–4 | C-2 |
| C-6 | Beacon conformance | Days 2–5 | C-3 |
| C-7 | Commissioning release and transponder | Days 3–8 | C-3, C-4 |
| C-8 | AI payload (Jetson) and fallback | Days 8–11 | C-7 |
| C-9 | GNSS and operator ephemeris | From first contact | C-1; ground pipeline **blocked** (LV-04) |
| C-10 | Operations validation and SAFE drill | Days 11–13 | C-3 to C-8 |
| C-11 | Commissioning review and close-out | Day 14 | All |

Until `COMMISSIONING_RELEASE` (C-7) the commissioning hold set by `ARM_FLIGHT` keeps the spacecraft in IDLE at most,
with the Jetson and the transponder off, across resets (SRS-F082); a corrupt operations record holds as well. The
transponder can therefore run only after the release, and the plan sends `XPDR_DISABLE` and `PLD_DISABLE` before it so
that each is started on its own step.

## 3. Steps

### C-0 Ground readiness

| | |
|---|---|
| Entry | Launch contract and integration date known; telecommand stations designated (D1); keys loaded in the flight unit and held by the stations (SQSO-ICD-013) |
| Actions | Space-Track organization account, satellite-operator status, operator contact public (UG v11 §9.5.7.1); SpaceX Space Traffic Coordination onboarding (space-safety-onboarding@spacex.com) and a test upload; TraCSS registration; pre-launch OPM loaded in the tracking software; beacon, HK1/HK2 and acknowledgement decoder and frame logger at each station and on SatNOGS; command tool with counters synchronized to the flight unit's at `ARM_FLIGHT` (UTC-derived counters if two stations command, SQSO-ICD-013 §5.2.1); the downlink-offset estimate for the uplink (rule 2) in the station software; the gyro alignment matrix measured on the flight unit (VI-MCAD-05) prepared as an `ADCS_SET_GYRO_ALIGN` frame; orbit-determination pipeline (or the third-party service) tested; rehearsal of C-1 to C-3 against `sqso_sil` |
| Proof | Rehearsal log; Space-Track and SpaceX test acknowledgements |
| Exit | Ground readiness review passed at L-2 M; the integrator's ephemeris requirement accepted in writing (VI-LV-13) |

### C-1 Acquisition and identification

| | |
|---|---|
| Entry | Separation report and SpaceX state vector received; S+45 min passed (the radio, receiver included, is unpowered until then). The transmitters stay silent while the AntS deployment is pending (decision C-19, SRS-F167), so after an eclipse separation from a cold dispenser the first beacon can come one or more 50-minute slots later (S+81 min in the simulation), at worst about S+27 h; the receiver is on meanwhile |
| Commands | None |
| Proof | Beacon decoded with source KK6GDB-11 and the call sign at the start of the text; tail values: `M=` IDLE (SAFE or a missing beacon means a low separation SOC, MISSION_TIMELINE §8.4; ACTIVE is impossible under the hold), `SOC=` ≥ 40, `BV=` 14.4–16.4 V, `SUN=` alternating with the orbit; `T=` consistent with the separation time (T counts seconds since boot, so T ≈ time since separation unless the OBC reset; HK1 `BOOT` and `RST` say whether it did); the first HK1 (after the fifth beacon): `HOLD=1`, `XP=0`, `TX=1`, `RFS` complete, `DEP` deployment state; Doppler curve matching the SpaceX state vector within the dispersion |
| Expected | First beacon pass over the Bay Area (≥ 5°) S+0.7 to S+12.1 h after separation (median S+5.3 h); anywhere in the network after about S+1.8 h (simulation; SQSO-MTL-001 §5, §10) |
| Exit | Beacons decoded on ≥ 2 passes; candidate object identified among the 18th SDS elements by Doppler; association reported to the integrator (SQSO-REG-015 §2) |
| If not | MISSION_TIMELINE §8.6 |

### C-2 First authenticated command

| | |
|---|---|
| Entry | C-1 on this pass; elevation ≥ 10° (12° degraded); ≥ 3 min of pass left. First commanding pass: median S+6.4 h, at the latest S+13.7 h (with the simulation's 15° mask; sooner at the 10° rule; SQSO-MTL-001 §10) |
| Commands | `GET_HOUSEKEEPING` (0x09); then `NOP` (0x00) |
| Proof | Acknowledgement `R=0 N=0` for each; an HK1 frame within the pass: `MODE` matches the beacon's `M=`, `HOLD=1`, `CN` incremented, `CS` reset, no authentication failure counted in HK2 `CA` |
| Exit | Both commands executed from station 1, then from station 2 on a later pass |
| If not | Resend once per pass with a new counter; check the downlink-offset correction (rule 2), the counter window and the key slot; after 3 passes, follow the key-recovery path (SQSO-ICD-013 §9) |

### C-3 Cessation of emissions (TX-OFF) proof

Required before the hold is released and the transponder opened to users (CONOPS §2), and the basis of the 97.207(b)
certification.

| | |
|---|---|
| Entry | C-2 executed on an earlier pass; a pass with ≥ 6 min above 10° and both stations able to hear |
| Commands | `SET_BEACON` (0x30) 30 s, so that silence is visible within the pass; `TX_DISABLE` (0x32); wait ≥ 2 beacon intervals; `TX_ENABLE` (0x31); `SET_BEACON` back to the mode default |
| Proof | No beacon, housekeeping frame or transponder output for ≥ 60 s after `TX_DISABLE` (both stations listening; it has no acknowledgement); beacons and an acknowledgement of `TX_ENABLE` resume; `RF=1` in the tail and `TX=1` in HK1 afterwards |
| Exit | TX-OFF and TX-ON demonstrated once from each station (simulation reference: S+23.7 h) |
| If not | If `TX_ENABLE` is lost the spacecraft stays silent: send it on the next pass (the receiver stays on, and the latch persists through resets) |

### C-4 Health and power characterization

| | |
|---|---|
| Entry | C-2 |
| Commands | `GET_HOUSEKEEPING` on every commanding pass; `DUMP_TLM` (0x0A) daily (store 0, from the last record received, count ≤ 600); `EPS_CLEAR_LOCKOUT` (0x14, argument `CHG!`) only if the EPS has latched its recharge lockout after a cell below 2.0 V, and only after the ground has reviewed the cell history |
| Proof | SOC, battery voltage and current (`SOC`, `VB`, `IB`), array power (`PA`), temperatures (`TB` battery, `TO` OBC, `TP` PA) on each pass and in the 5-minute records, against the sunlit/eclipse phase from the orbit; compared with the simulation (SOC ≈ 92 % at end of charge, depth of discharge ≤ 2.1 % per eclipse, about 2.5 Wh generated per orbit against 2.25 Wh used); battery charging only between 0 and +45 °C, the heater on below +5 °C and off above +9 °C (`TB`); HK2 `TR` no rail trips |
| Exit | 3 days with SOC ≥ 80 % at each pass, no SAFE or EMERGENCY entry (`M=` never SAFE), `VB` at the 4.10 V/cell end of charge (16.4 V) in sunlit passes, `TB` inside the charge window; no new fault (`FC`, `FL`) without an explanation |

### C-5 Deployables and attitude

| | |
|---|---|
| Entry | C-2 |
| Commands | `GET_HOUSEKEEPING`; `ADCS_SET_GYRO_ALIGN` (0x25) with the matrix measured on the flight unit (VI-MCAD-05; persisted, refused unless every element is within 1.1 and the determinant within 0.05 of 1); `ADCS_SET_MAG_BIAS` (0x26) per rod polarity state once the on-orbit bias estimate is available (AV-V4; persisted); `ADCS_DETUMBLE` (0x21) only if `W` stays above 0.5°/s across +Y beyond S+5 h; `DEPLOY_RETRY` (0x50: 0 antenna, 1 solar wings) in a sunlit pass if `DEP` reports an element FAILED (refused during the 30-minute wait and during a burn); `ADCS_SET_GEOMETRY` (0x24: 1 wing W1 only, 2 wing W2 only, 3 no wing) only as C-5.1 below decides (decision C-21: with one wing out the spacecraft stays at preset 0 until the identification names the wing) |
| Proof | `DEP`: deployment COMPLETE, AntS elements 15 (all four), both wing senses deployed; `W` body rates: spin about +Y at about 8°/s, transverse rates below 0.5°/s; `AM` sun-pointing; `SUN=1` through each sunlit pass; `PA` and the SOC recovering at about 8 % per sunlit orbit after a discharge, which needs the wings; beacon signal strength against range consistent with a deployed 2 m monopole (≈ +8 dB margin at 5°, BASELINE §7); HK2 `GY` the gyro in use |
| Exit | `DEP` complete and the spin stable on 3 consecutive passes; or, with an element failed after its retries, the geometry preset set and the power case of MISSION_TIMELINE §8.2 confirmed |

#### C-5.1 One wing deployed: which one (decision C-21)

SOLAR_SENSE is both hinge switches in series, so with one wing out HK1 `DEP` shows only "not both". The flight software
then keeps geometry preset 0 (both wings; HK2 `GE=0`, SRS-F171) and never picks a one-wing preset by itself: the wrong
one-wing preset is the case not analysed, preset 0 costs 2–3 % of the power. Presets 1 and 2 are sent only after this
procedure names the wing. Numbers from the GN&C engineer (ADCS_DESIGN 1.9 §11.7, Monte Carlo with photodiodes and
albedo).

| Step | Action | Pass criterion |
|------|--------|----------------|
| 1. Entry | `DEP` wing sense not deployed after the wing retries (or `DEPLOY_RETRY` 1 refused to help); `AM` sun-pointing at preset 0 (`GE=0`); spin captured (`W` about 8°/s about +Y) | — |
| 2. Method B (primary) | Over at least one full sunlit orbit under preset 0, collect HK1 `W` (body rates x/y/z, 0.01 °/s) from every pass and the stored five-minute records (DUMP_TLM); compute the tilt = atan2(mean W_x, mean W_y) | Tilt ≤ −1.5° names **W1** only (expected −3.5 … −3.8°, Monte Carlo trials −4.9 … −3.0°); ≥ +1.5° names **W2** only (expected +3.4°, trials +2.7 … +4.1°); between −1.5° and +1.5° (both wings or neither: expected +0.1°, −0.6 … +0.7°): no one-wing preset, stay at 0. The nearest trial is 1.2° from its threshold, gyro bias included |
| 3. Method A (secondary, confirmation) | Only during a capture (after a commanded `ADCS_DETUMBLE`, while the Sun is 30–70° off +Y, which lasts a few minutes): a 1 Hz record of `PA` (array power), the sun vector and `W` over 3 turns; fit c + a·cos ωt + b·sin ωt (ω from the gyro's y rate) to the +Z photodiode and to `PA` | Sun 30–70° from +Y; `PA` harmonic amplitude ≥ 0.2 W (≥ 0.41 W expected with one wing, ≤ 0.02 W with both or neither); its lag behind the +Z harmonic within ± 45° of +90° (**W1**) or −90° (**W2**); otherwise repeat. Needs a 1 Hz record the flight software does not yet downlink (HK2 `SN` and `IA` are snapshots per `GET_HOUSEKEEPING`); until that capture exists, method A is a ground analysis of repeated `GET_HOUSEKEEPING` snapshots and does not override method B |
| 4. Command | `ADCS_SET_GEOMETRY` 1 (W1 named) or 2 (W2 named); HK2 `GE` confirms | `GE` reads the preset sent |
| 5. Confirm | One day under the new preset: tilt from `W` as in step 2 | Tilt moves to −8.0° (preset 1) or +8.9° (preset 2) within 2°; a wrong preset gives +2.8° (preset 1 sent with W2 out) or −1.9° (preset 2 with W1 out). More than 2° from the preset's axis after a day: send `ADCS_SET_GEOMETRY` 0 and repeat from step 2 |

### C-6 Beacon conformance

| | |
|---|---|
| Entry | C-3 |
| Commands | `SET_POWER` (0x33) 0 (0.5 W) and 1 (1 W) on separate passes; `SET_BEACON` 10, 60 and 120 s |
| Proof | Recorded spectrum at two stations: occupied bandwidth within 10K4F2D, deviation, carrier within the coordinated channel plus Doppler; cadence matches the command; call sign in every frame; the measured downlink offset logged for rule 2 |
| Exit | Both power levels and the cadence within the licence and the IARU coordination; return to `SET_POWER` 0 |

### C-7 Commissioning release and transponder

| | |
|---|---|
| Entry | C-3 and C-4 passed; a sunlit pass with SOC ≥ 55 % |
| Commands | `XPDR_DISABLE` (0x35) and `PLD_DISABLE` (0x41), so that the release starts neither; `COMMISSIONING_RELEASE` (0x08); `SET_MODE` (0x02) 2 if the mode manager has not entered ACTIVE; in a later commanded pass `XPDR_ENABLE` (0x34); team uplink with the correct CTCSS tone, then without it, during a Bay Area pass; `XPDR_DISABLE` to show the transponder stops alone |
| Proof | HK1 `HOLD=0` after the release; `MODE` ACTIVE when sunlit with SOC ≥ 55 %; after `XPDR_ENABLE`, `XP=1` while the gates hold; repeated audio on the downlink only with a valid CTCSS carrier; `QSO=` in the tail and HK2 `XE` increment; the transponder stops in eclipse and below 55 % SOC (observe on a pass that crosses the terminator) and on `XPDR_DISABLE` with the beacon continuing; keyed time ≤ 25 % of 100 minutes (simulation: about 38 min a day, duty 9.9 %) |
| Exit | Gates observed working on 2 passes from each station; public announcement of the transponder |

### C-8 AI payload (Jetson) and fallback

| | |
|---|---|
| Entry | C-7 (hold released); ACTIVE; sunlit; SOC ≥ 55 %; `TJ` ≥ −15 °C |
| Commands | `PLD_ENABLE` (0x40); observe; `PLD_DISABLE` (0x41); `PLD_ENABLE` |
| Proof | With the payload disabled the beacon text is a template and the telemetry tail is unchanged; with it enabled, generated lines appear in batches of ≥ 10, each passing the ground copy of the content filter (`docs/BEACON_SPEC.md` §6); HK2 `JB` (bursts, the last burst's energy, its lines, bursts ended by the energy gate), `JP` (clean switch-offs) and `HL` = 0 (no hard-limit violation). The D5 gate is measured from orbit: the burst energy at the carrier input divided by the accepted lines, against ≤ 0.1 Wh (360 J) per accepted beacon (simulation: about 0.036 Wh) |
| Exit | Fallback demonstrated; 100 generated beacons decoded with no filter failure; energy per accepted beacon within the D5 gate, or the experiment left disabled |

### C-9 GNSS and operator ephemeris

| | |
|---|---|
| Entry | First contact (C-1) |
| Commands | `GNSS_REQUEST` (0x60) to start a session before a pass if no recent fix (its power gates still apply: Jetson off, SOC ≥ 40 %; the transponder is held off during a session); sessions also run by themselves every orbit in launch and early operations, then four a day (SRS-F084) |
| Proof | HK1 `GF` (age of the latest fix, satellites used) and `GS` (sessions run / with a fix); operator ephemeris (CCSDS OEM with covariance) accepted by Space-Track and SpaceX Space Traffic Coordination within 24 h of first contact, after every pass and at least every 24 h until the 18th SDS catalogs the object (UG v11 §9.5.7.2) |
| Exit | NORAD ID assigned, positional data public, operator contact public; object claimed within 30 days |
| Status | The onboard chain is implemented; the ground orbit determination from the fixes and the OEM upload are **blocked** (LV-04). Until they exist, the third-party service contracted before integration delivers the ephemeris |

### C-10 Operations validation and SAFE drill

| | |
|---|---|
| Entry | C-3 to C-8 |
| Commands | `SET_MODE` 0 (SAFE); observe 2 beacons at 180 s; `SET_MODE` 1 (IDLE); `SET_MODE` 2 (ACTIVE) on the next pass; `DUMP_TLM` of the 60 s records around the mode changes; `CLEAR_FAULTS` (0x05) only after the fault log (`FC`, `FL` and the dumped records) is understood |
| Proof | `M=` and `MODE` follow each command; SAFE: template text, no transponder (`XP=0`); the transition table refuses SAFE → ACTIVE directly (acknowledgement `R` = `NOT_ALLOWED`) |
| Exit | 24 h of autonomous operation with no SAFE or EMERGENCY entry; shift handover rehearsed |

### C-11 Commissioning review and close-out

| | |
|---|---|
| Entry | C-1 to C-8 and C-10 closed; C-9 closed or covered by the third party |
| Commands | `SET_CONTACT_TIMEOUT` (0x07), 2-byte big-endian days = 14 (D3); the command is itself a contact, so the count restarts |
| Proof | Acknowledgement; HK1 `CT=14` and `CS` reset; the passivation path intact for the end of life (`PS=0`, EOL_STATE fuse-intact bit set) |
| Exit | Review signed by the operations lead, the systems engineer and the licensee; primary operations start |

## 4. Anomaly handling

| Anomaly | Likely cause | Response |
|---------|--------------|----------|
| No beacon at the first Bay Area pass | Pass geometry (median first pass S+5.3 h); low SOC; antenna stowed; the AntS still deferred by its temperature gate, the transmitters held (decision C-19; `DEPLOY_RETRY` 0 with the force flag, sent blind, deploys it at once) | MISSION_TIMELINE §8.2, §8.6 |
| `M=SAFE` at first contact | Separation SOC below 25 % | Wait: the simulation reaches IDLE within S+4–8 h |
| Weak signal at high elevation; `DEP` shows an element FAILED | Antenna not deployed | `DEPLOY_RETRY` 0 in a sunlit pass; characterize the link |
| Command not executed (no acknowledgement) | Uplink frequency offset, counter window, key slot, link | C-2 "If not" |
| Unexpected SAFE | Fault, low SOC, thermal | `GET_HOUSEKEEPING`, `DUMP_TLM` of the 60 s records; return through IDLE after the cause is understood |
| Recharge lockout latched | A cell below 2.0 V | Review the cell history from the dumped records; `EPS_CLEAR_LOCKOUT` only on the review's decision |
| Interference or a stuck transmitter | — | `TX_DISABLE` immediately; investigate before `TX_ENABLE`; `XPDR_DISABLE` if only the transponder is at fault |

## 5. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR launch and ConOps review | Rewritten step by step (C-0 to C-11) with entry criteria, the commands of the flight dictionary, the telemetry that proves each step and its exit criterion; TX-OFF proof added before the transponder; ephemeris step added; the RF silence is 45 min; the separation SOC and pass statistics from the simulation and the pass analysis. Version 1.0 used commands that do not exist (`JETSON_POWER`, `AI_ENABLE`, `FORCE_FALLBACK`, `ECHO_TEST`, `RELEASE_SAFE`, `SET_TX_POWER`) and assumed a 30-minute RF timer, a > 95 % SOC at deployment and telemetry the downlink does not carry |
| 2.1 | 2026-10-27 | CDR round 3, launch and operations (RID LV-13) | Brought to the command dictionary of flight software round 3 (30 commands): HK1 and HK2 as the proof of each step (`GET_HOUSEKEEPING` 0x09, `DUMP_TLM` 0x0A), the acknowledgement rule; C-7 is the commissioning release (`COMMISSIONING_RELEASE` 0x08, with `XPDR_DISABLE`/`XPDR_ENABLE` 0x35/0x34 and `PLD_DISABLE` first), so the transponder runs only after the release; deployment retry (`DEPLOY_RETRY` 0x50), `ADCS_SET_GEOMETRY` 0x24, `ADCS_SET_GYRO_ALIGN` 0x25 and `ADCS_SET_MAG_BIAS` 0x26 in C-5; `GNSS_REQUEST` 0x60 in C-9; `EPS_CLEAR_LOCKOUT` 0x14 in C-4; `PASSIVATE_ARM`/`PASSIVATE_EXECUTE` 0x70/0x71 excluded; commanding above 10° (12° in the degraded mode, C-14) and the uplink offset from the measured downlink (RFB-05); pass statistics from SQSO-MTL-001 2.3 (first beacon pass median S+5.3 h, at most S+12.1 h; first commanding pass median S+6.4 h, at most S+13.7 h); the D5 energy gate measured from HK2. Version 2.0 said the transponder started by itself at S+53 min, that there was no housekeeping downlink, deployment retry or GNSS command, that commands went only above 15° and that the dictionary had 17 commands |
| 2.2 | 2026-10-27 | CDR flight software, round 3, fourth part (decision C-19) | C-1 entry and §8: the first beacon waits for the AntS sequence's end (SRS-F167) |
| 2.3 | 2026-10-27 | CDR flight software, round 3, fifth part (decision C-21) | C-5.1 added: the one-wing identification (method B from the body-rate tilt, method A from the array-power harmonic), pass criteria from ADCS_DESIGN 1.9 §11.7; preset 0 until the wing is named (SRS-F171, HK2 `GE`) |
