# SMART-QSO Mission Timeline and Event Sequence

**Document ID**: SQSO-MTL-001
**Version**: 2.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft for CDR. Times after separation are from the mission simulation and the pass analysis named in §1;
launch-vehicle times are typical, not from a mission timeline.

## 1. Purpose and evidence

This is the concept of operations as a timeline: from delivery of the flight model to the integrator, through
integration into the dispenser, launch, separation, the first passes and commissioning, to day 14. For each step it
says what is powered, what draws energy, the battery state and what the ground does, and it records what the
design does not yet provide (§9).

| Evidence | What it gives | How to reproduce |
|----------|---------------|------------------|
| Mission simulation, commissioning scenario (seed 20280601; integration 2028-04-15 at 60 % SOC, separation 2028-06-01 06:00 UTC, 500 km SSO, LTAN 22:30) | Every flight-software event and the power, thermal and attitude state at 10 s resolution; 0 anomalies in 35 checks | `cd software/rust && cargo build --profile sil -p sqso-sil && ./target/sil/sqso_sil --scenario commissioning --out <dir>`; `events.jsonl`, `timeseries.csv`, `console.log` |
| Simulation variants | Low separation SOC (40, 25, 15, 10 %), wings failed, wings and ADCS failed, 3-day ground outage, 0 °C storage | Same, with `--set integration_soc=0.25` and so on (§8) |
| `software/simulation/cdr/launch/leop_passes.py` | Passes over the candidate telecommand stations for 576 separation cases (UTC hour × argument of latitude), eclipse at separation, β over 2028 | `python3 leop_passes.py --out <dir>` |
| `software/simulation/cdr/launch/leop_feasibility.py` | Storage drain, eclipse energy, burn energy, ejection velocity, turn-on delay, hard-body radius, ephemeris deadline | `python3 leop_feasibility.py --out <dir> --passes <dir>` |

The planning launch date is L = 2028-06-01 (the simulation's). The simulation separates at 06:00 UTC; the real time
depends on the flight and its deploy order, so pass times are given as statistics over all separation cases.

## 2. Mission phases and durations

| Phase | From | To | Duration |
|-------|------|----|----------|
| Delivery and integration | Flight model at the integrator (≈ L-50 d) | Door closed in the EXOpod Nova (≈ L-47 d) | ≈ 3 days |
| Storage and launch campaign | Integration | Liftoff | ≈ 47 days planned; the design tolerates 9.4 months to the 40 % floor at a 50 µA drain (§3) |
| Launch and ascent | Liftoff | Separation from the dispenser | ≈ 1–3 h (deploy order) |
| LEOP | Separation (S) | First authenticated command executed and TX-OFF proven | S to ≈ S+1 d |
| Commissioning | LEOP end | Commissioning review, `SET_CONTACT_TIMEOUT` 14 days | Days 1–14 |
| Primary operations | Day 14 | 2 years (design life); minimum 12 months | |
| End of life | Day 730 | Passivation (TX-OFF, charge path cut); re-entry within 5 years of mission end (SIL: 3.0 years after) | |

## 3. Delivery, integration and storage

| When | Event | Spacecraft state | Energy | Ground / team action |
|------|-------|------------------|--------|----------------------|
| ≈ L-50 d | Flight model received at the integrator's site; receiving inspection, rail measurement against the Exolaunch guide | Off; RBF in | Pack as shipped (≤ 30 % if flown as air cargo under the carrier's lithium rules; otherwise 60 %) | Exolaunch and the team; record SOC by OCV |
| L-48 d | Functional test on the umbilical (OBC up from the umbilical, no RF: conducted checks into a load only, UG §9.5.2) | OBC on the umbilical | GSE power | Record boot log, EPS telemetry |
| L-47 d, step 1 | **Final charge to 60 % ± 5 %** through the umbilical, RBF in (the charge path must not energize the loads, VI-LV-15) | Charger only | 3.4 Ah pack to ≈ 2.0 Ah | Charge log; stop by OCV/coulomb count |
| L-47 d, step 2 | Key load and GSE provisioning (7 operations), `ARM_FLIGHT` as the **last** umbilical action: deployment sequence and RF silence set to ARMED | OBC on the umbilical | GSE | The SIL does this: "GSE: 7 of 7 operations done; OBC off, umbilical removed" |
| L-47 d, step 3 | Umbilical removed; RBF removed; CubeSat slid into the 3U slot with its adapter; door closed (switches actuated); set screws | **Off**: separation switches open the battery path; only the battery-side BMS is powered (CDS §2.3.1.2) | Self-discharge plus BMS drain | Close-out photos; the integrator's encapsulation readiness letter (UG Appendix D, scenario A) |
| L-47 d to L | In the dispenser: integrator, PPF (≤ 45 d, 15–25 °C, 30–65 % RH), hangar (≤ 40 d), pad (≤ 5 d, 15–30 °C) | Off | SIL: 59.9 % → **58.3 %** after 47 days at 20 °C (self-discharge only). Model with a 50 µA BMS drain: 56.7 % after 47 days; 47.3 % after 6 months; 40 % after 9.4 months | None possible: no umbilical, no charging (NOVA §5.4; UG §7.6) |

A launch slip with the CubeSat integrated is tolerated without recharge for as long as the separation SOC stays
above 40 % (§8.4): 18.9 months with no drain, 15.7 months at 10 µA, 9.4 months at 50 µA, 6.3 months at 100 µA (20 °C;
`leop_feasibility.py` §1). The drain is not yet designed or measured; LVR-071 sets it at ≤ 20 µA (VI-LV-14).

## 4. Launch and ascent

Falcon 9 Transporter events are typical values, not from a mission timeline (the SpaceX flight timeline comes with
the trajectory analysis, UG §9.6.9.1).

| Time | Event | Spacecraft | Ground |
|------|-------|------------|--------|
| ≈ L-5 d | Encapsulated assembly mated to the launch vehicle | Off | — |
| T-1 h to T-0 | Countdown; fairing on GN₂ purge | Off; switches held by the Nova door and wagon | Team on station at the operations centre; tracking software loaded with the pre-launch orbit (UG Appendix E, pre-launch OPM) |
| T-0 | Liftoff from SLC-4E | Random vibration 5.57 Grms MPE; acoustic 139.3 dB; quasi-static up to 10 g axial / 17 g lateral at the dispenser | — |
| ≈ T+2.5 min | Stage separation | Shock at the interface (MPE 1000 g above 1 kHz) | — |
| ≈ T+3 min | Fairing separation | Pressure decay ≤ 2.8 kPa/s before it; free-molecular heating < 3,500 W/m² | — |
| ≈ T+8 min | Second-stage cutoff; coast; later burns to the drop-off orbit | Conductive boundary −20 to +69 °C (UG Table 4-11) | — |
| ≈ T+1 h to T+3 h | Deployment sequence; the integrator's Nova door opens on its signal | — | Integrator reports the door telemetry |
| **S** | **SMART-QSO ejected** (1.1–1.5 m/s for 1.5–2.0 kg; tip-off < 10°/s) | Switches release | SpaceX quick-look orbit injection report and separation state vector (UG §10.6, Appendix E) to the integrator, then to the team |

## 5. Separation to first contact (LEOP)

Times are from separation S. "Load" is the bus load measured in the simulation; the battery is 58.3 % at S in the
reference case.

| Time after S | Event (flight software) | Powered | Load | Battery | Ground |
|--------------|-------------------------|---------|------|---------|--------|
| S+0.15 s | Switches release: power-on reset; the OBC boots; RF-silence and deployment counts start at 0 (both were ARMED) | OBC, EPS, sensors | 0.42 W | 58.3 %, 15.1 V, 20 °C | — |
| S+1 s | BOOT → DETUMBLE; **ADCS starts B-dot on the magnetorquers** (as built; UG v11 §3.3.4 asks for no attitude maneuver before 120 s: RID LV-02, proposed start at S+120 s) | + magnetorquers (≤ 0.3 W avg) | 0.42–0.44 W | 58.3 % | — |
| S+0 to S+34.5 min worst | Darkness if separated in eclipse: 41.7 % of the separation cases; 2.8–34.5 min (median 18.7) until first sunlight. Reference case: first sun at S+11 min | — | 0.43 W | Worst case −0.25 Wh (−0.5 % SOC) before first sun | — |
| S+11 min onward | Body cells charging (≈ 1.9–2.7 W from the body before the wings) | | | rising | — |
| S+30:00 | Deployment wait served (CDS §2.4.4); 33 % of the cases are in eclipse at this moment (§8.1) | | | 59.2 % | — |
| S+30:01 | Antenna burn line on: two current-limited resistors, 1.6 A for ≈ 2.4 s (3 s maximum); both antennas sensed deployed at S+30:03 | + burn line | 24 W for ≤ 3 s | Peak 1.63 A (0.48 C); 2.5 A alarm, 5 A BMS limit | — |
| S+30:06 | Wing burn line on: 1.6 A for ≈ 2.5 s (5 s maximum); both wings sensed deployed at S+30:09 | + burn line | 24 W for ≤ 5 s | Nominal burns 0.054 Wh; with all 3 retries per element 0.16 Wh (0.33 % SOC) | — |
| S+30:12 | Deployment COMPLETE (persisted); the hard-body radius is now ≈ 0.55 m (reported to SpaceX as 0.6 m, LVR-065) | | | | — |
| S+45:00 | RF silence complete (CDS §2.4.5): radio powered; **first beacon** in IDLE, every 120 s, template text with the telemetry tail, call sign KK6GDB-11 | + receiver (always on), TNC, PA keyed ≈ 1.4 s per frame | 0.64 W | 60.9 % | Community and SatNOGS stations may hear it; Bay Area stations only if a pass is in progress |
| S+49 min | Detumbled: rate across +Y below 0.5°/s (SIL 0.8 h; Monte Carlo slowest 1.4 h) | | | | — |
| S+52.8 min | DETUMBLE → IDLE → **ACTIVE**; **Jetson batch powered; FM transponder enabled** (as built, with no ground contact yet: RID LV-01; the ConOps wants the transponder only after telecommand and TX-OFF are proven) | + Jetson (≈ 5 W during a ≤ 10 min batch), transponder enable | 6.3 W during the batch | 61.6 % | — |
| S+1 orbit (95 min) | Spin-stabilized sun pointing of +Y at 6°/s; generation 7.0 Wh in orbit 1 against 1.85 Wh used | | 2.2 W orbit average | 61.8 % → rising ≈ 8 % per orbit at the 0.40 A charge current | — |
| S+1.8 h | First beacon decoded anywhere in the simulation's network (Fairbanks) | | | | Amateur reports start; compare Doppler against the SpaceX state vector |
| S+0.7 to S+12.2 h (median S+5.2 h) | **First pass over Santa Clara above 5°** after the silence (the first beacon the licensee's station can hear). Reference case: S+11.7 h (7.6°, 4 frames) | | | ≈ 92 % by S+12 h (end of charge at 4.10 V/cell) | Decode, check call sign, mode, SOC, BV, SUN; start the 24 h ephemeris clock (UG §9.5.7.2 counts from first contact) |
| S+0.7 to S+13.7 h (median S+6.3 h) | **First commanding pass** (≥ 15°) over Santa Clara or Ames. Reference case: S+13.2 h (55°): authenticated `GET_STATUS` executed | | | ≈ 92 % | First command; then `TX_DISABLE` / `TX_ENABLE` to prove TX-OFF (COMMISSIONING_PLAN C-3) |
| By first contact + 24 h (S+24.7 h to S+36.2 h) | **First operator ephemeris** with covariance to Space-Track and SpaceX Space Traffic Coordination | | | | Needs GNSS fixes downlinked and the ground orbit determination: **not implemented** (RID LV-04); until then a third party must deliver it |
| S+24 h | End of LEOP if the first command is executed and TX-OFF proven | Battery heater cycling (battery +1 to +5 °C at night) | 1.1 W average | 92 % | Daily: ephemeris, health report, AMSAT-BB note |

With a geographically separated second telecommand station the first commanding pass comes sooner: median
S+3.7 h (with Morehead, KY) or S+3.4 h (with Fairbanks, AK) instead of S+6.3 h (`leop_passes.py`, illustrative sites).

## 6. Commissioning, days 1–14

Pass budget from the Bay Area (Santa Clara or Ames; the two see the same passes): 3.9 passes a day above 5°, of
which 2.5 a day reach 15° (median pass 8.0 min above 5°, 21 min of commanding time a day); the passes come in two
clusters, about 10:00–11:45 and 21:15–23:00 local mean solar time (11:00–12:45 and 22:15–24:00 PDT); the longest gap
between commanding passes is 13.1 h every day. With 2 commanding passes a day (the simulation's plan) the 14 days give
28 commands; the simulation executed 28 of 28.

| Day | Steps (`docs/operations/COMMISSIONING_PLAN.md`) | Passes used | Energy and state | Ground |
|-----|---------------------------------------------------|-------------|------------------|--------|
| 0–1 | C-1 acquisition, C-2 first command, C-3 TX-OFF proof | First 2–4 commanding passes | SOC ≈ 92 % from the end of day 0 | Ephemeris every pass and ≥ every 24 h until cataloged; identification by Doppler (SQSO-REG-015 §2) |
| 1–3 | C-4 health and power characterization (orbit energy balance from the telemetry tail and `EPS_GET_TELEMETRY`) | 2 a day | Generation ≈ 2.6 Wh/orbit drawn against 2.25 Wh used (the regulator curtails the rest at full charge); depth of discharge ≤ 2.1 % per eclipse | Compare with the simulation's orbits.csv |
| 2–4 | C-5 attitude: `ADCS_GET_ATTITUDE`, spin 6°/s, +Y within ≈ 2–4° of the Sun | 2 a day | | |
| 3–5 | C-6 beacon conformance (deviation, bandwidth, cadence, `SET_BEACON`, `SET_POWER`) | 2 a day | | Spectrum recorded at two stations |
| 5–8 | C-7 transponder: first windows in commanded passes, then autonomous | 2 a day | Transponder keyed ≈ 80 min a day in the simulation; ≤ 14.4 % of an orbit | Public announcement only after C-7 passes |
| 8–11 | C-8 payload (Jetson) batches and fallback | 2 a day | ≈ 0.036 Wh per accepted beacon | Filter sweep on the ground |
| 11–13 | C-9 operations validation, contingency drill (`SAFE` and recovery) | 2 a day | | |
| 14 | C-10 review; `SET_CONTACT_TIMEOUT` 14 days | 1 | SOC ≈ 92 % | Commissioning report |

The order above is the plan. **As built, the flight software does not wait for it**: ACTIVE, the Jetson and the
transponder start at S+53 min (§5). Until RID LV-01 is closed, C-7 and C-8 are verifications of behaviour that is
already running, not enabling steps.

## 7. Primary operations, end of life

Unchanged in substance from `mission/CONOPS.md` §2–§3 and `docs/operations/END_OF_LIFE_PLAN.md`: transponder windows
when sunlit and SOC ≥ 55 %; beacon every 60 s (ACTIVE), 120 s (IDLE), 180 s (SAFE); authenticated contact at least
once in each 14-day loss-of-contact period; ephemeris after every pass until cataloged, then by the plan of
SQSO-REG-015. End of life at day 730 (minimum 12 months): final beacon, TX-OFF, permanent charge-path disconnect;
re-entry no later than 5 years after the end of the mission (FCC; the 25-year figure of version 1.0 is withdrawn).
The simulation's full mission re-enters 3.03 years after the design life.

## 8. Contingencies, checked

### 8.1 Deployed in eclipse

41.7 % of the separation cases start in eclipse (≤ 34.5 min of darkness, β = 17° on 2028-06-01; β stays 17–24°
through 2028, so an eclipse of about 35 min exists in every orbit of the year). The load before the burns is 0.43 W:
0.25 Wh, 0.5 % of the pack. 33 % of the cases burn at S+30 min in eclipse, with the hold-downs cold. Energy is not the
issue (0.16 Wh with all retries); release at the cold temperature is, and is not verified (VI-LV-17). The three
attempts per element are 60 s apart (`deployment.rs` `RETRY_DELAY_MS`), so all of them happen in the same eclipse.

### 8.2 A deployable fails

The sequence marks the element FAILED after 3 attempts and is then terminal: only GSE `ARM_FLIGHT` resets it.
`deployment_force_retry` exists behind the C header (`sqso-ffi/src/deployment.rs`) but **no telecommand reaches it**
(the dictionary `sqso-protocol/src/dictionary.rs` has 17 commands, none for deployment): RID LV-07. Consequences,
simulated over 14 days:

| Case | Result |
|------|--------|
| Wings not deployed | Commissioning completes; lowest SOC 69.1 %; depth of discharge 2.8 % per eclipse; generation 537 Wh against 459 Wh used; lifetime on the wings-failed row of BASELINE §3 (500 km: 6.2 yr nominal, inside the FCC rule) |
| Wings not deployed and ADCS failed (tumbling), separation SOC 38 % | Power-positive (534 Wh against 446 Wh); lowest SOC 54.8 %; depth of discharge 3.4 %; never detumbles (expected) |
| Antennas not deployed | Not simulated. The 2 m monopole stowed against the body radiates poorly; the link margin of ≈ +6 dB at 5° (BASELINE §7) would be lost on most passes, and the 70 cm uplink with it. A ground-commanded retry in sunlight is the only recovery |

### 8.3 The first passes are missed

A 3-day ground outage from separation (`contact_outage_days=3`): no anomaly; 22 of 22 later commands executed; the
loss-of-contact inhibit is 30 days at launch (D3), so the spacecraft keeps transmitting. The cost is regulatory and
contractual: the ephemeris clock (24 h from first contact) and the transponder running unattended (RID LV-01).

### 8.4 Low state of charge at separation (long storage, high drain, or a missed recharge)

| Separation SOC | Flight-software response (simulation) | First RF | Detumbled |
|----------------|----------------------------------------|----------|-----------|
| 58 % (reference) | DETUMBLE → IDLE at S+53 min | S+45 min | S+0.8 h |
| 38 % | Same path | S+45 min | S+0.8 h |
| 23 % | Boots into SAFE; IDLE at S+4.2 h | S+45 min | S+4.3 h |
| 13 % | SAFE → EMERGENCY at S+1 s (beacon shed); SAFE at S+2.6 h; IDLE at S+7.0 h | **S+2.6 h** | **S+7.5 h** (beyond 3 orbits) |
| 8 % | EMERGENCY until S+4.0 h | **S+4.0 h** | **S+9.1 h** |

The design recovers from all of them in the first day, but below 15 % the beacon and detumble requirements of LEOP
are missed. Floor set: **≥ 40 % at separation** (LVR-071).

### 8.5 Switch bounce during ejection

The switches are power inhibits, so a bounce that cuts the power is a power-on reset: both counts restart from 0
(`rf_silence.rs`, `deployment.rs`). A bounce that does not cut the power but is seen on the sense line also restarts
both counts. Either way the effective start is the last release, so the 30- and 45-minute rules hold (CDS §2.3.2.4).
A 1U leaves a 3U slot in about 0.1–0.2 s and nothing touches the switches afterwards. One failure mode remains: after
completion the RF silence still restarts whenever the sense line reads stowed, so a sense line failed "stowed" in
orbit keeps the radio silent indefinitely while the spacecraft is otherwise healthy (RID LV-15).

### 8.6 No beacon heard

| By | Action |
|----|--------|
| S+2 h | Nothing expected yet from the Bay Area in most cases; check SatNOGS and the AMSAT network |
| First Bay Area pass ≥ 15° | Search ± 10 kHz around the downlink (Doppler ± 3.3 kHz at 145.9 MHz); update the elements from the SpaceX state vector and the 18th SDS candidates |
| S+24 h | Assume low SOC (§8.4) or a stowed antenna; keep listening every pass; send `GET_STATUS` blind on each pass (the receiver is on in every mode) |
| S+30 d | The loss-of-contact inhibit would stop the transmitter if no command was authenticated; it has no effect on a spacecraft that was never heard |

### 8.7 Unpowered clock

The OBC has no real-time clock through storage (none is fitted; CDS §2.3.3 would allow one), so at separation the
spacecraft knows only the time since boot (the beacon's `T=` field). UTC comes from the GNSS receiver once it runs, or
from ground correlation of the first frames. Version 1.0's "time within 1 s of UTC at delivery" cannot be met and is
withdrawn.

## 9. What the design does not yet provide

| Step | Missing | RID |
|------|---------|-----|
| S+1 s | A 120 s hold on the magnetorquers after separation (UG §3.3.4) | LV-02 |
| S+53 min | A commissioning hold: ACTIVE, the Jetson and the transponder start before any ground contact | LV-01 |
| S+30 min | A ground-commanded deployment retry; scheduling retries for sunlight; the ISIS AntS (bought) is an I²C device with its own controller, while the flight software drives a GPIO burn line | LV-07 |
| First contact + 24 h | The GNSS chain: receiver power schedule, fix in telemetry, ground orbit determination, CCSDS OEM upload | LV-04 |
| Commissioning | Telecommands to switch the transponder alone, to run a GNSS session and to retry a deployment | LV-14 |
| LEOP passes | A second, geographically separated telecommand station (Santa Clara and Ames are 12 km apart) | LV-08 |
| Storage | A measured battery-side drain and a defined integration SOC | LV-06 |
| In orbit | Ignoring a stowed sense line once the silence is complete | LV-15 |

## 10. Contact schedule

| Station | Role | Passes/day ≥ 5° | Passes/day ≥ 15° | Commanding min/day | First beacon pass after S (median, max) | First commanding pass after S (median, max) |
|---------|------|-----------------|------------------|--------------------|-------------------------------------------|----------------------------------------------|
| Santa Clara University, CA | Prospective telecommand station (D1) | 3.88 | 2.51 | 21.4 | 5.2 h, 12.2 h | 6.3 h, 13.7 h |
| NASA Ames, CA | Prospective telecommand station (D1) | 3.88 | 2.51 | 21.4 | 5.2 h, 12.2 h | 6.3 h, 13.7 h |
| Fairbanks, AK | Illustrative separated site | 9.98 | 5.18 | 44.6 | 1.9 h, 12.0 h | 4.2 h, 13.5 h |
| Morehead, KY | Illustrative separated site | 3.93 | 2.55 | 21.8 | 5.2 h, 12.2 h | 6.2 h, 13.7 h |
| SatNOGS and community | Receive only | — | — | — | — | — |

500 km SSO at 97.40°, period 94.6 min; 576 separation cases, 14 days each, 10 s steps; minimum elevation for a usable
command pass 15°, for beacon reception 5° (the simulation's values).

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR launch and ConOps review | Rewritten from the mission simulation and the pass analysis: delivery to the integrator, integration and storage with the battery drain, launch, a minute-by-minute LEOP with power, energy, battery and ground actions, the commissioning pass budget, contingencies checked by simulation (eclipse, failed deployables, missed passes, low SOC, switch bounce), the design gaps, and the contact schedule for the baseline orbit. Version 1.0 described P-POD deployment, the RF released at 30 min, a 400 km 51.6° orbit, a 25-year disposal rule and commands that do not exist |
