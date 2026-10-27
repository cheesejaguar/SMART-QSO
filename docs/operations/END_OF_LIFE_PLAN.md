# End-of-Life Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-EOL-001
**Revision:** C
**Date:** 2026-10-27
**Reference:** NASA-STD-8719.14C; 47 CFR 97.207(b), 97.207(g)(1)(iii), (g)(1)(vii)(D)(1), (g)(3); 97.211(b)
**Governing sources:** disposal and re-entry, `docs/regulatory/filings/ODAR_SMART-QSO.md` (SQSO-REG-018) Draft D §4–§6;
passivation, `docs/BASELINE_PARAMETERS.md` Rev E §5 and §7, the telecommands of `docs/interfaces/TELECOMMAND_AUTHENTICATION.md`
(SQSO-ICD-013) v0.8 §6.4 and the charge disconnect of `hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001) §7;
the concept of operations, `mission/CONOPS.md` phase 5. Where this plan and those sources differ, they govern.

---

## 1. Purpose and Summary

This plan says when the SMART-QSO mission ends, how the spacecraft is passivated, how the ground confirms it, what
happens when passivation cannot be commanded, and how the spacecraft leaves orbit. Revision B and earlier described a
deployable drag device, a 400–500 km orbit, the 25-year rule and 47 CFR Part 25; all four are withdrawn.

| Item | Baseline |
|------|----------|
| End of mission | Planned at the end of the 2-year design life (about 2030-11-01 for the 2028-11-01 planning launch), or earlier on the criteria of §2 |
| Passivation | Two-step authenticated telecommand: `PASSIVATE_ARM` (0x70) with the operational key, then `PASSIVATE_EXECUTE` (0x71) with the offline recovery key within 600 s. The spacecraft latches TX-OFF first, then fires the one-shot charge disconnect (fuse F1). Irreversible |
| After passivation | Transmitters permanently off; the solar array disconnected from the pack and the bus; the loads drain the pack to the battery-management under-voltage cut-off, after which the spacecraft is dark (97.207(g)(1)(iii): battery in a permanent discharge state) |
| Disposal | Uncontrolled re-entry by natural decay with the solar wings deployed (they are the drag area). **No drag device.** No propulsion |
| Disposal rule | FCC: re-entry no later than 5 years after completion of the mission, probability ≥ 0.9 (97.207(g)(1)(vii)(D)(1)). NASA-STD-8719.14C 4.6-1a (25 years) is met with a wide margin |
| Expected re-entry (ODAR Draft D) | From the 475 km target: 4.8–5.1 years after deployment with the nominal solar projection, 5.3–5.7 years with the conservative one (1.6–1.85 kg, wings deployed), i.e. 2.8–3.7 years after a 2-year mission. P(re-entry ≤ 5 years after the planned mission end) = 0.997 at 475–485 km |
| Autonomous passivation | **None.** The loss-of-contact inhibit only stops transmission (BASELINE §7); the proposed M-day autonomous passivation is not implemented and the ODAR recommends against it (§5) |

## 2. Ending the Mission: Decision Criteria

The PI decides to end the mission, with the licensee (KK6GDB), on a recorded review (§2.3). The decision is the start
of the procedure of §3; nothing on board ends the mission by itself.

### 2.1 Planned end

The mission ends at the 2-year design life (BASELINE §2; mission success Level 3) unless an extension is approved under
§2.2. The planned termination date for the regulatory dates is 2030-11-01 (post-space notification by 2031-02-01,
SQSO-REG-014).

### 2.2 Extension

Extending the mission shortens the time from mission end to re-entry, so the five-year rule does not limit an
extension. An extension of up to 12 months at a time may be approved while **all** of these hold, checked from the
telemetry of the last 30 days:

| # | Condition | Evidence |
|---|-----------|----------|
| E1 | Both keys usable: the operational key authenticated a command in the last 14 days, and the recovery key custodian holds the key and its counter (a `KEY_ROTATE` or `TX_DISABLE` under it is not needed to prove it) | Ground key log; HK1 `CS`, `CN` |
| E2 | Battery healthy: lowest SOC in the last 30 days ≥ 40 %; depth of discharge ≤ 10 % per eclipse; charge reaches end of charge (4.10 V/cell) each sunlit period; no BMS fault or cell-imbalance fault in the fault log | HK1 `SOC`, `VB`, `IB`, `TB`, `FL`; stored housekeeping (`DUMP_TLM`) |
| E3 | The passivation path is intact: no fault logged against `EOL_ARM`/`EOL_FIRE`, `PS=0` | HK1 `PS`, `FL` |
| E4 | The transmitter meets its emission limits (no spurious reports, deviation in specification) | Ground spectrum checks (COMMISSIONING_PLAN C-6 method) |
| E5 | At least one telecommand station is available for the next 12 months, so the procedure of §3 can be run on demand | Station agreements (BASELINE §1) |
| E6 | The licensee's authorization and coordination remain valid | Licensee |

### 2.3 Early end

The PI ends the mission before 2 years, and passivates on the next suitable pass (§3.2), on any of:

| Trigger | Condition | Why passivate now |
|---------|-----------|-------------------|
| Loss of the mission | Transponder service lost with no recovery path (Level 2 or 3 cannot be met) and the beacon has no further value to the PI | No reason to keep stored energy on board |
| Threat to the passivation itself | Any of E1–E3 fails or is degrading: one key left, receiver recoveries increasing (HK1 `RX`), capacity fade > 20 %, a BMS or charger fault, a cell more than 50 mV from the others | Passivate while it can still be commanded and fired |
| Battery hazard | Pack temperature above +45 °C while charging, an over-voltage event, or any sign of a venting cell | Removes the charge source that drives the hazard |
| Regulatory | Licence revoked or not renewed; an interference complaint that cannot be resolved; an FCC order to cease transmissions (97.207(b)) | An FCC order is first answered with `TX_DISABLE` (§5.6); passivation follows on the PI's decision |
| Orbit | Predicted re-entry within 90 days (the spacecraft will be lost to drag and heating soon after) | Passivation needs sunlight and a working bus |

### 2.4 Decision record

The end-of-mission review records: the trigger, the telemetry evidence, the orbit (latest Space-Track elements and the
operator ephemeris), the re-run lifetime prediction (NASA DAS or the project estimator, `software/simulation/cdr/sma/debris_lifetime.py`,
with the tracked orbit and the current NOAA solar prediction), the passivation date, the two key custodians and the
stations for the passivation pass and the confirmation passes.

## 3. Passivation Procedure

### 3.1 What the spacecraft does (SQSO-ICD-013 v0.8 §6.4; SQSO-EPS-BRD-001 §7)

1. `PASSIVATE_ARM` (0x70, operational key, 4-byte argument: the ASCII code `EOL!`) arms for 600 s and drives nothing.
   A wrong code is rejected (`INVALID_PARAM`); an arm not executed within 600 s lapses silently (logged).
2. `PASSIVATE_EXECUTE` (0x71, **recovery key**, no argument) within those 600 s executes. Without a live arm it is
   `NOT_ALLOWED` and does nothing. The two keys are held by two custodians: a two-person rule for an irreversible act.
   There is no first-year lockout.
3. On execute, in this order: (a) the TX-OFF latch is set together with `RF_INHIBIT_PASSIVATED`, which nothing clears,
   so nothing transmits from this moment; (b) the passivation record is persisted (`FLASH_REGION_EOL`); (c) in sunlight,
   `EOL_ARM` (OBC PF4) is raised, its rising edge setting the EPS arm latch, and 200 ms later `EOL_FIRE` (OBC PG1) is
   held for 60–120 s, dropped early once the EPS reports the charge path open. An eclipse before 60 s drops both lines
   and the fire is repeated at the next sunlight, at most 3 attempts. After a reset the record restores the passivated
   state before anything can transmit, and an incomplete fire resumes.
4. The EPS fires the three-terminal fuse F1 (ITV4030L1412NR) only when both the arm-latch FET Q21 and the fire FET Q20
   conduct; no single failure fires it. F1 sits between the trackers' output and the pack: once open, the solar array
   feeds neither the pack nor the bus. The fire needs 9.8–24 W for under a minute, which is why it is done in sunlight.
5. The loads stay on (receiver, OBC, EPS, sensors, the battery heater when the pack is below +5 °C) and drain the pack
   to the BMS under-voltage cut-off (2.8 V/cell). From about 90 % SOC (≈ 45 Wh) at 0.6–1.9 W that takes about 1–3
   days; then the spacecraft is permanently dark.

### 3.2 Choice of the passivation pass

| Criterion | Value |
|-----------|-------|
| Pass | Over the primary telecommand station, maximum elevation ≥ 30° (≥ 6 minutes above 10°, room for two attempts of each command), with a second station's pass within the following 2 hours for confirmation |
| Spacecraft | In sunlight for the whole pass and for at least 10 minutes after it (the fire needs sunlight; it waits for it otherwise); MODE IDLE or ACTIVE |
| People | Both key custodians present at the station (or the recovery-key custodian at the station with the operational-key frame prepared); a licensed control operator; the licensee or the PI on the line |
| Avoid | Passes in which a conjunction warning is open; the 24 h after a reset (`UP` < 86,400 s in HK1) |

### 3.3 Before the passivation pass

| When | Step | Verification |
|------|------|--------------|
| T − 90 d | End-of-mission review and decision record (§2.4); lifetime re-run with the tracked orbit; confirm predicted re-entry ≤ 5 years after T | Signed decision record |
| T − 30 d | Announce the end of the mission: AMSAT-BB and AMSAT News Service, the IARU satellite frequency coordinator (the frequencies will be released), SatNOGS (request observations of the passes of T − 1 d to T + 7 d), the telecommand station partners | Messages archived |
| T − 30 d to T − 7 d | Final science and engineering data: daily `DUMP_TLM` (0x0A) of the stored housekeeping; archive every beacon and HK1 frame (§7) | Archive index complete; stored records up to T − 7 d received |
| T − 14 d | Optional end-of-mission beacon from the pre-approved template set (`SET_BEACON`; the content filter of `docs/BEACON_SPEC.md` §6 applies, no generated text) | Beacon text decoded |
| T − 7 d | Ground rehearsal of the two-command sequence on the flatsat or with the mission simulation's end-of-life plan (`software/rust/sqso-sil`), using test keys | Both frames accepted in the rehearsal; timing within 600 s |
| T − 1 d | Build both frames with the ground tool and the next counter of each key; check the counters against the last acknowledgements; check both keys against their custody records | Frames reviewed by both custodians |
| T − 1 d | Final orbit: latest GNSS-based ephemeris (or third-party orbit) sent to Space-Track; record apogee and perigee for the post-space notification | Ephemeris accepted |

### 3.4 The passivation pass

Telemetry fields are those of the HK1 frame (`docs/operations/TELEMETRY_DICTIONARY.md` §4.4) and the command
acknowledgement (§4.5). `GET_HOUSEKEEPING` (0x09) returns an HK1 frame on demand.

| Step | Action | Go / verification from telemetry |
|------|--------|----------------------------------|
| 1 | Acquisition: decode the beacon and send `GET_HOUSEKEEPING` | Call sign KK6GDB-11; `MODE` IDLE or ACTIVE; `SOC` ≥ 550 ‰; `VB` ≥ 14,000 mV; `PA` > 0 (sunlit); `TB` 0–45 °C (0.1 °C units: 0–450); `TX=1`; `PS=0`; no new `FL` entry since the review. **No go** if any fails: end the pass, re-plan |
| 2 | Send `PASSIVATE_ARM` (operational key, argument `EOL!`) | Acknowledgement `ack KK6GDB-11: C=<counter> R=0 N=0`; the next HK1 (send `GET_HOUSEKEEPING`) shows `PS=1`. If `R` is not 0: stop, diagnose (wrong code, counter); a second ARM with a new counter is allowed |
| 3 | Within 600 s of the ARM acknowledgement, and in the same pass: send `PASSIVATE_EXECUTE` (recovery key) | **No acknowledgement and no further beacon**: TX-OFF is latched before anything else, so silence is the expected response. If a beacon or an acknowledgement is still heard 60 s after the command: the execute was not received or was refused (`NOT_ALLOWED` if the arm lapsed); repeat steps 1–3 with new counters while the pass lasts |
| 4 | Listen to the end of the pass on the downlink frequency with a spectrum display | No carrier on the downlink channel (the beacon would be due every 60–180 s) |
| 5 | Log the UTC time of the last frame heard and of the execute command | Station log, both custodians sign |

### 3.5 Confirmation after the pass

| When | Step | Verification |
|------|------|--------------|
| T + 0 to T + 2 h | Second station listens on its pass | No carrier, no frame |
| T to T + 7 d | Every pass over both stations, and the SatNOGS observations requested at T − 30 d | No frame from KK6GDB-11 on any pass (≥ 50 station-passes); SatNOGS observation IDs recorded for the post-space notification |
| T + 7 d | Declare **termination of transmissions** at the UTC time of step 3; file the post-space notification (§6) | Licensee's declaration |
| T onward | Do **not** send further commands. `TX_ENABLE` cannot undo the passivation (`RF_INHIBIT_PASSIVATED` is never cleared), but nothing is to be gained, and the receiver goes dark within days anyway |

**What telemetry cannot show.** Nothing is transmitted after `PASSIVATE_EXECUTE` (the `PS` values 2, firing, and 3,
passivated, are never downlinked), so the charge disconnect itself cannot be confirmed from the ground; the beacon's
end confirms only the TX-OFF. The fire is verified before flight: the C suite `test_passivation`, the Rust unit tests of
`software/rust/sqso-fsw/src/passivation.rs`, the mission simulation's end-of-life plan, and V-EPS-8 on the EM with a
sacrificial fuse (SQSO-EPS-BRD-001 §19). The HK1 frame does not carry the EPS `EOL_STATE` register or the F1-intact
sense (F1_SNS), so the ground cannot check before the pass that the fuse is intact either (finding EOL-1, §9).

## 4. Stored Energy after Passivation

| Source | State after passivation | Basis |
|--------|-------------------------|-------|
| Li-ion battery (4S1P, ≈ 50 Wh) | Disconnected from the array by F1 (non-resettable); drained by the loads to the BMS cut-off (2.8 V/cell), then self-discharging with no charge source | BASELINE §5; SQSO-EPS-BRD-001 §7 |
| Solar array | Generates, but connected to nothing downstream of F1 | same |
| RF transmitters | TX-OFF latched and `RF_INHIBIT_PASSIVATED`; then unpowered when the pack reaches the cut-off | SQSO-ICD-013 §6, §6.4 |
| Mechanisms | Wings and antennas deployed at T + 30 min after separation; no stored spring energy left | ODAR §2 |
| Momentum | Spin decays once the OBC is dark (no actuators) | — |
| Propellant, pressure vessels, pyrotechnics | None | ODAR §2 |

This is NASA-STD-8719.14C Req. 4.4-2 option 1. If passivation is not achieved, option 2 holds: the explosion
probability of ODAR §4 (1.4 × 10⁻⁴, requirement < 10⁻³) already assumes no passivation in half of the missions.

## 5. If Passivation Cannot Be Commanded

There is no autonomous passivation. BASELINE §7 proposes one (M = 180 days after the loss-of-contact inhibit, never in
the first year), but it is not implemented, and ODAR Draft D §4 recommends not implementing it: it would act only when
the bus works but cannot be commanded, a case in which option 2 already holds, and being irreversible and triggered by
an absence of commands it adds a premature-passivation failure mode (decision D3, FCC consultation). The cases:

| # | Situation | What the spacecraft does | Ground action | End state |
|---|-----------|--------------------------|---------------|-----------|
| 5.1 | **Spacecraft lost** (no beacon, no acknowledgement) before or at the end of the mission | Nothing can be commanded | Listen on scheduled passes and SatNOGS for 30 days; then declare termination; post-space notification within 3 months, method "loss of power", passivation status "not achieved; energy controlled by the battery protections" | Option 2 (ODAR §4); disposal unchanged (passive) |
| 5.2 | **Beacon heard, commands not accepted** (receiver failed, keys or counters out of step) | The **loss-of-contact inhibit** turns every transmitter off N days after the last authenticated command, receiver still on: **N = 14 days** after commissioning (30 days before), SQSO-ICD-013 §6.2, BASELINE §7. Any later authenticated command lifts it | Try the counter rules of SQSO-ICD-013 §5.2 and the recovery key's `KEY_ROTATE` to a spare key; if a key is usable again, run §3. If none works within the N days, the spacecraft falls silent by itself; continue to listen | Transmissions end without a command; charge path connected; option 2. The FCC consultation (D3) asks whether this reversible inhibit counts as termination for the post-space notification |
| 5.3 | **Recovery key unusable** (lost, custodian unavailable for good), operational key good | `PASSIVATE_EXECUTE` cannot be authenticated; an ARM lapses after 600 s | Send `TX_DISABLE` (operational key): TX-OFF latched, receiver on. Declare termination; passivation status "transmitters disabled; charge path not disconnected" | Option 2. Made unlikely by two sealed copies of the recovery key at two sites (EOL-5) |
| 5.4 | **Execute accepted, fire not completed** (eclipse on 3 attempts, F1 does not open, an EOL line fault) | TX-OFF and `RF_INHIBIT_PASSIVATED` hold whatever happens to the fire, so emissions have ended; the fire resumes after a reset if it did not complete | Not observable from the ground (§3.5); report "charge disconnect commanded at UTC …; completion verified by design and test, not by telemetry" | Emissions ended; option 2 if the fuse did not open |
| 5.5 | **Bus in SAFE or EMERGENCY at the end** (low SOC) | Commands are still accepted; the fire needs sunlight and 9.8–24 W | Wait for SOC ≥ 55 % in sunlight if the battery recovers; otherwise execute anyway in sunlight (the array also feeds the F1 heater) | Passivated, or 5.4 |
| 5.6 | **FCC order to cease transmissions** (97.207(b)) at any time | — | `TX_DISABLE` on the next pass of either station (operational key; keyless `TX_DISABLE` if no key is usable on board, 97.211(b)); repeat with new counters until no beacon is heard on two passes. Longest wait for a commanding pass: 13.0 h with one station, 9.9 h with two (SQSO-GND-001 Rev D §5). Post-space notification within 24 hours | Transmissions ended; passivation by §3 on the PI's decision |

## 6. Regulatory and Coordination Notifications

| Recipient | When | Content |
|-----------|------|---------|
| **FCC Space Bureau: post-space notification** (97.207(g)(3)) | **No later than 3 months after termination of transmissions**; **within 24 hours** if the termination was ordered by the FCC. Planned: by 2031-02-01 for termination on 2030-11-01 | Template SQSO-REG-014 §B (`docs/regulatory/filings/FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md`): licensee; UTC of termination and the method (telecommand TX-OFF and passivation, loss of power, or re-entry; never "autonomous passivation", which is not implemented); confirmation (station logs, SatNOGS observation IDs); reason; passivation status against the pre-space disclosure of 97.207(g)(1)(iii); orbit at termination; predicted re-entry and the five-year compliance date |
| IARU satellite frequency coordinator | T − 30 d and after termination | End of transmissions; frequencies released |
| 18th Space Defense Squadron, through Space-Track | T − 1 d (final ephemeris) and after termination | Spacecraft passivated, non-manoeuvring; no further operator ephemeris |
| AMSAT, SatNOGS, the amateur community | T − 30 d and after termination | End-of-mission announcement; thanks; request to report any signal heard (there should be none) |
| Launch integrator (Exolaunch), if the launch agreement asks | After termination | End-of-mission report |

## 7. Disposal by Natural Decay

Uncontrolled re-entry with the wings deployed; no drag device, no manoeuvre. Lifetimes are from the project estimator
(NRLMSIS 2.1, Cd 2.2, the NASA-STD-8719.14C average cross-sections: 0.0247 m² wings deployed, 0.0164 m² wings failed;
deployment 2028-11-01; `software/simulation/cdr/sma/debris_lifetime.py`), as tabulated in ODAR Draft D §6.2, and must
be reproduced in NASA DAS before filing (ODAR open item 1).

### 7.1 Orbital lifetime

Total life in years from deployment (life after a 2-year mission in brackets). Solar projections: nominal (NOAA SWPC
prediction and a cycle 26 of maximum 140), conservative (NOAA low band and a weak cycle 26 of 110), bound (F10.7 = 70
held throughout). **Bold**: more than 5 years after the mission.

| Deployment | Mass | Wings deployed: nominal / conservative / bound | Wings failed: nominal / conservative / bound |
|------------|------|-----------------------------------------------|---------------------------------------------|
| 465 km | 1.85 kg | 4.7 (2.7) / 5.2 (3.2) / 5.9 (3.9) | 5.6 (3.6) / 6.4 (4.4) / **8.9 (6.9)** |
| **475 km (target)** | 1.6 kg | 4.8 (2.8) / 5.3 (3.3) / 6.3 (4.3) | 5.7 (3.7) / 6.6 (4.6) / **9.5 (7.5)** |
| **475 km (target)** | 1.85 kg | 5.1 (3.1) / 5.7 (3.7) / **7.3 (5.3)** | 6.1 (4.1) / **7.1 (5.1)** / **10.9 (8.9)** |
| 485 km (cap) | 1.6 kg | 5.3 (3.3) / 5.9 (3.9) / **7.7 (5.7)** | 6.2 (4.2) / **7.2 (5.2)** / **11.7 (9.7)** |
| 485 km (cap) | 1.85 kg | 5.6 (3.6) / 6.3 (4.3) / **9.0 (7.0)** | 6.5 (4.5) / **7.9 (5.9)** / **13.5 (11.5)** |

Masses: 1.6 kg nominal and 1.85 kg bound (BASELINE §4; the MEV is 1.73 kg). From the 475 km target the spacecraft is
at about 451–460 km at the end of the 2-year mission and crosses the ISS altitude (≈ 400 km) 4.2–5.1 years after
deployment (nominal to conservative projection; ODAR §5 covers conjunctions during the decay). Expected re-entry:
2033-08 to 2034-07 from 475 km; 2034-02 to 2035-02 from the 485 km cap. The FCC limit for a mission ending on
2030-11-01 is 2035-11-01.

### 7.2 Compliance

| Requirement | Result (ODAR Draft D) |
|-------------|------------------------|
| FCC 97.207(g)(1)(vii)(D)(1): re-entry ≤ 5 years after the mission, P ≥ 0.9 | Mission counted to its planned 2 years (reading B): **P = 0.997** at 475 and 485 km with the conservative projection (the only failing trials are the wings-failed ones, probability 0.29 % with four independent burn lines). If the FCC counts an early failure as the end of the mission (reading A), P is 0.67–0.75 at 475 km and the band moves to 460 km (P ≥ 0.99); question for the FCC consultation (D3) |
| NASA-STD-8719.14C 4.6-1a: ≤ 25 years after the mission | Met at ≤ 500 km in every case; ≤ 12.6 years at the 485 km cap |
| 4.6-4: disposal reliability ≥ 0.9 | Disposal is passive and needs only the wings, deployed at T + 30 min; met |
| 4.5-1: collision with objects ≥ 10 cm < 0.001 | ≤ 1.2 × 10⁻⁴ (worst case 525 km, 2.0 kg, wings failed, bound projection); ≤ 5.4 × 10⁻⁵ at 475 km |
| 4.7-1: casualty risk < 1:10,000 | No fragment > 15 J with nominal heating; worst sensitivity 1:47,000 |

### 7.3 Re-entry tracking

| Phase | Activity |
|-------|----------|
| After passivation | Follow the Space-Track elements monthly; update the lifetime prediction yearly with the NOAA solar forecast |
| Below 300 km | Weekly; announce the predicted re-entry window to the community |
| Re-entry | Space-Track decay message (TIP); record the date; add it to the archive and, if the post-space notification is already filed, send the FCC the re-entry date |

## 8. Data Archive and Closeout

| Product | Retention | Due |
|---------|-----------|-----|
| Every beacon, HK1 frame, acknowledgement and dump frame received (station and SatNOGS), as received with time tags | Permanent, public | T + 30 d |
| Station logs of the passivation and confirmation passes; key custody records (not the keys) | 10 years | T + 30 d |
| Final mission report (including the ODAR as flown and the lifetime prediction at termination) | Permanent, public | T + 60 d |
| Regulatory correspondence (pre-space, in-space, post-space notifications; IARU) | Licence record | T + 90 d |
| Key destruction: the operational keys and the recovery key are destroyed once termination is declared, with a witnessed record | — | T + 30 d |

## 9. Findings and Open Items

| ID | Item | Owner |
|----|------|-------|
| EOL-1 | The HK1 frame carries `PS` but neither the EPS `EOL_STATE` register nor the F1-intact sense (F1_SNS), so the ground cannot see before the passivation pass that the fuse is intact. Recommended: add an `F1=` field (F1_SNS, EOL_STATE) to HK1 (`docs/operations/TELEMETRY_DICTIONARY.md` §4.4; SRS-F126) | Flight software (for the chair) |
| EOL-2 | Run V-EPS-8 (passivation fire on the EM with a sacrificial fuse, both signals, in the sunlit and eclipse cases) before the flight build | EPS |
| EOL-3 | Reproduce the ODAR lifetimes in NASA DAS with the controlled mass properties (ODAR open item 1); repeat with the tracked orbit at the end-of-mission review (§2.4) | S&MA |
| EOL-4 | The D3 consultation: whether the reversible loss-of-contact inhibit counts as termination for 97.207(g)(3), and which reading of "completion of the mission" applies to a failed spacecraft | Licensee |
| EOL-5 | Recovery key: two sealed copies at two sites, with a named second custodian (§5.3) | Licensee |

## 10. Verification before Launch

| What | How | Where |
|------|-----|-------|
| Two-key sequence, lapse, eclipse, resume after reset | C suite `test_passivation`; Rust unit tests of `passivation.rs` | CI (done) |
| End-of-life plan end to end, decay to re-entry | Mission simulation `full` scenario (`docs/test/SIL_MISSION_SIMULATION.md`) | SIL |
| F1 fire through both signals only; no single failure fires it | V-EPS-8 | EM |
| Procedure of §3 with the ground tool and real keys on the flight unit (TX into a load, F1 replaced by a dummy) | Day-in-the-life test | FM |

---

## Appendix A: Passivation Checklist

- [ ] End-of-mission review signed (§2.4); lifetime re-run: re-entry ≤ 5 years after T
- [ ] T − 30 d announcements: AMSAT, IARU, SatNOGS observations requested, station partners
- [ ] Final `DUMP_TLM` dumps received and archived
- [ ] Rehearsal of the two-command sequence with test keys
- [ ] Both frames built and reviewed by both custodians; counters checked
- [ ] Final ephemeris to Space-Track; apogee and perigee recorded
- [ ] Pass: go criteria of §3.4 step 1 met (`SOC` ≥ 550, `VB` ≥ 14,000, `PA` > 0, `TB` 0–450, `TX=1`, `PS=0`)
- [ ] `PASSIVATE_ARM`: `ack … R=0 N=0`, `PS=1`
- [ ] `PASSIVATE_EXECUTE` within 600 s: no acknowledgement, no further beacon
- [ ] No frame on any pass for 7 days (both stations, SatNOGS IDs recorded)
- [ ] Termination declared; post-space notification filed (≤ 3 months; ≤ 24 h if FCC-ordered)
- [ ] IARU and Space-Track informed
- [ ] Keys destroyed with a witnessed record
- [ ] Archive and final mission report

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | CDR 2026 consistency pass: notice that §3–§10 are stale (drag device, 400–500 km, 25-year rule) with the governing sources (ODAR, BASELINE Rev E §3, §5, the EOL disconnect of SQSO-EPS-BRD-001 §7) | CDR documentation review |
| C | 2026-10-27 | Rewritten to the Rev E baseline (RID SE-15): end-of-mission decision criteria and extension conditions; the two-step passivation of SQSO-ICD-013 v0.8 §6.4 (PASSIVATE_ARM 0x70, operational key; PASSIVATE_EXECUTE 0x71, recovery key, within 600 s; TX-OFF, then the one-shot charge disconnect F1 of SQSO-EPS-BRD-001 §7) step by step with telemetry verification; cases in which passivation cannot be commanded (loss-of-contact inhibit, BASELINE §7; no autonomous passivation); FCC post-space notification; re-entry by natural decay with the wings deployed within 5 years of mission end, with the lifetimes and probability of ODAR Draft D. Drag device, 25-year rule, Part 25 and the autonomous passivation sequence withdrawn | CDR S&MA analysis |
