# SMART-QSO Mission Timeline and Event Sequence

**Document ID**: SQSO-MTL-001
**Version**: 2.4
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
| Mission simulation, commissioning scenario at the Rev E defaults (seed 20280601; integration 2028-09-15 at 60 % SOC, separation 2028-11-01 06:00 UTC, 475 km SSO, LTAN 22:30; four wing burn lines, one resistor at a time; flight software round 2) | Every flight-software event and the power, thermal and attitude state at 10 s resolution; 0 anomalies in 42 checks | `cd software/rust && cargo build --profile sil -p sqso-sil && ./target/sil/sqso_sil --scenario commissioning --out <dir>`; `events.jsonl`, `timeseries.csv`, `console.log` |
| Simulation variants | Low separation SOC (40, 25, 15, 10 %), wings failed, wings and ADCS failed, 3-day ground outage, 0 °C storage | Same, with `--set integration_soc=0.25` and so on (§8) |
| `software/simulation/cdr/launch/leop_passes.py` | Passes over the candidate telecommand stations for 576 separation cases (UTC hour × argument of latitude) at 475 km from 2028-11-01, eclipse at separation, β over the first year | `python3 leop_passes.py --out <dir>` |
| `software/simulation/cdr/launch/leop_feasibility.py` | Storage drain, eclipse energy, burn energy, ejection velocity, turn-on delay, hard-body radius, ephemeris deadline | `python3 leop_feasibility.py --out <dir> --passes <dir>` |

The planning launch date is L = 2028-11-01 (BASELINE Rev E §3; the simulation's). The simulation separates at 06:00
UTC; the real time depends on the flight and its deploy order, so pass times are given as statistics over all
separation cases.

## 2. Mission phases and durations

| Phase | From | To | Duration |
|-------|------|----|----------|
| Delivery and integration | Flight model at the integrator (≈ L-50 d) | Door closed in the EXOpod Nova (≈ L-47 d) | ≈ 3 days |
| Storage and launch campaign | Integration | Liftoff | ≈ 47 days planned; the design tolerates 9.4 months to the 40 % floor at a 50 µA drain (§3) |
| Launch and ascent | Liftoff | Separation from the dispenser | ≈ 1–3 h (deploy order) |
| LEOP | Separation (S) | First authenticated command executed and TX-OFF proven | S to ≈ S+1 d |
| Commissioning | LEOP end | Commissioning review, `SET_CONTACT_TIMEOUT` 14 days | Days 1–14 |
| Primary operations | Day 14 | 2 years (design life); minimum 12 months | |
| End of life | Day 730 | Passivation (TX-OFF, charge path cut; `docs/operations/END_OF_LIFE_PLAN.md` §3); re-entry within 5 years of mission end (ODAR Draft D: 2.8–3.7 years after it from 475 km; the simulation's own decay model: 2.0 years) | |

## 3. Delivery, integration and storage

| When | Event | Spacecraft state | Energy | Ground / team action |
|------|-------|------------------|--------|----------------------|
| ≈ L-50 d | Flight model received at the integrator's site; receiving inspection, rail measurement against the Exolaunch guide | Off; RBF in | Pack as shipped: ≤ 30 % SoC by air (UN3481 contained in equipment, PI 967: recommended by the IATA DGR 2026 guidance, mandatory under many operator variations), 60 % by road (SQSO-BSA-001 §6); final charge to 60 ± 5 % at the integrator | Exolaunch and the team; record SOC by OCV |
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

Times are from separation S. "Load" is the bus load in the simulation; the battery is 58.3 % at S in the
reference case. Mission simulation, commissioning scenario at the Rev E baseline (475 km, 2028-11-01, the four burn
lines, flight software round 2); pass statistics from `leop_passes.py` at the same orbit.

| Time after S | Event (flight software) | Powered | Load | Battery | Ground |
|--------------|-------------------------|---------|------|---------|--------|
| S+0.15 s | Switches release: power-on reset; the OBC boots; RF-silence and deployment counts start at 0 (both were ARMED); BOOT → DETUMBLE; the commissioning hold is set | OBC, EPS, sensors | 0.42 W | 58.3 %, 15.1 V, 20 °C | — |
| S+121 s | **B-dot detumble starts** (no magnetorquer output before 120 s, SpaceX UG v11 §3.3.4; RID LV-02 closed) | + magnetorquers | 0.42–0.46 W in the simulation (its torquer model); the board design draws 1.16–1.3 W while detumbling (POWER_BUDGET_ANALYSIS §8.6) | 58.3 % | — |
| S+0 to S+28.8 min worst | Darkness if separated in eclipse: 33.3 % of the separation cases; 5.2–28.8 min (median 16.9) until first sunlight. Reference case: first sun at S+21 min | — | 0.43 W | Reference 0.15 Wh before first sun; worst 0.20 Wh (0.4 %) at the simulated load, 1.46 Wh (2.9 points) with the board's torquer power and a cold pack (POWER_BUDGET_ANALYSIS §8.6) | — |
| S+14.8 min | Detumbled: rate across +Y below 0.5°/s (simulation; Monte Carlo slowest 1.2 h, 2.1 h from a 50°/s tip-off, BASELINE §8) | | | | — |
| S+19.2 min | DETUMBLE → **IDLE**; the commissioning hold keeps it there: no ACTIVE, no Jetson, no transponder until `COMMISSIONING_RELEASE` (RID LV-01 closed) | | 0.42 W | 58.0 % | — |
| S+21 min onward | Body cells charging (≈ 2.5–2.8 W from the body before the wings) | | | rising | — |
| S+30:00 | Deployment wait served (CDS §2.4.4); 33.3 % of the cases are in eclipse at this moment (§8.1) | | | 58.5 % | — |
| S+30:01 | **Wing burn**: line W1A only (one resistor, 0.8 A driver from the 7.0 V burn buck); both wings sensed deployed at S+30:12, so W1B, W2A and W2B are not used (at most one line on at any time: check `burn-lines` = 1) | + burn buck, one driver | 7.0 W bus load for 11 s; net pack current −0.15 A (the body array carries most of it in sunlight); in eclipse 0.67 A, 0.75 A with the magnetorquers (SQSO-EPS-BRD-001 §9) | 58.5 % | — |
| S+30:18 | **AntS** armed and deployed over I²C (3V3_ANTS); every element sensed deployed at S+30:33; deployment COMPLETE (persisted) at S+31:06. The hard-body radius is now ≈ 0.55 m (reported to SpaceX as 0.6 m, LVR-065) | + 3V3_ANTS | 1.9 W for 45 s | 58.5 % | — |
| S+45:00 | RF silence complete (CDS §2.4.5): radio powered; **first beacon** in IDLE, every 120 s ± 5 s, template text with the telemetry tail, call sign KK6GDB-11 | + receiver (always on), TNC, PA keyed ≈ 1.4 s per frame | 0.65 W | 61.3 % | Community and SatNOGS stations may hear it; Bay Area stations only if a pass is in progress |
| S+1 orbit (94.1 min) | Spin-stabilized sun pointing of +Y at 8.1°/s; orbit 1: 6.45 Wh generated, 0.89 Wh used (0.57 W average) | | 0.64 W in IDLE | 67.5 %, rising ≈ 10 points per orbit at the 0.40 A charge current | — |
| S+1.8 h | First beacon decoded anywhere in the simulation's network (Fairbanks, 6 of 6 frames) | | | 77.8 % at S+3.1 h | Amateur reports start; compare Doppler against the SpaceX state vector |
| S+0.7 to S+12.1 h (median S+5.3 h) | **First pass over Santa Clara above 5°** after the silence (the first beacon the licensee's station can hear). Reference case: S+11.6 h (5.9°, 2 frames) | | | ≈ 92 % by S+12 h (end of charge at 4.10 V/cell) | Decode, check call sign, mode, SOC, BV, SUN; start the 24 h ephemeris clock (UG §9.5.7.2 counts from first contact) |
| S+0.7 to S+13.7 h (median S+6.4 h) | **First commanding pass** (≥ 15°) over Santa Clara or Ames. Reference case: S+13.1 h (61.9°): authenticated `GET_HOUSEKEEPING` executed and acknowledged | | | 92.5 % | First command; then `TX_DISABLE` / `TX_ENABLE` to prove TX-OFF (COMMISSIONING_PLAN C-3) |
| S+23.7 h | TX-OFF test: `TX_DISABLE`, then `TX_ENABLE` 150 s later (reference case) | | | 92 % | TX-OFF proven |
| By first contact + 24 h (S+24.7 h to S+36.1 h, median S+29.3 h) | **First operator ephemeris** with covariance to Space-Track and SpaceX Space Traffic Coordination | | | | GNSS fixes downlinked and the ground orbit determination (RID LV-04); a third party is the contingency |
| S+24 h | End of LEOP if the first command is executed and TX-OFF proven | Battery heater cycling (battery +4.9 to +20 °C on day 1) | 1.06 W average on day 1 | 91.9 % | Daily: ephemeris, health report, AMSAT-BB note |
| S+36.7 h | `COMMISSIONING_RELEASE` by telecommand: IDLE → ACTIVE; the Jetson batches and the transponder start | + Jetson batches, transponder | | 92 % | Public announcement only after commissioning step C-7 |
| S+48.8 h | First daily `DUMP_TLM` (the newest 48 five-minute records) | | | 92 % | Stored housekeeping received |

The simulation deploys both wings on the first resistor of wing 1 (both hinge switches, wired in series, read
deployed after 11 s). On the hardware each wing has its own restraint, so a nominal sequence burns W1A and then W2A
(SQSO-EPS-BRD-001 §9); the energy difference is one 8 s burn (0.017 Wh). Raised to the chair as a simulation-model
observation.

With a geographically separated second telecommand station the first commanding pass comes sooner: median
S+3.8 h (with Morehead, KY) or S+3.5 h (with Fairbanks, AK) instead of S+6.4 h (`leop_passes.py`, illustrative sites).

## 6. Commissioning, days 1–14

Pass budget from the Bay Area (Santa Clara or Ames; the two see the same passes): 3.8 passes a day above 5°, of
which 2.4 a day reach 15° (median pass 7.7 min above 5°, 20 min of commanding time a day); the passes come in two
clusters, about 9:50–11:30 and 21:00–22:40 local mean solar time; the longest gap between commanding passes is
13.0 h every day. With 2 commanding passes a day (the simulation's plan) the 14 days give 28 commands; the simulation
executed 28 of 28 (27 acknowledgements heard), with 144 dump frames (576 records) and 0 anomalies in 42 checks.

| Day | Steps (`docs/operations/COMMISSIONING_PLAN.md`) | Passes used | Energy and state | Ground |
|-----|---------------------------------------------------|-------------|------------------|--------|
| 0–1 | C-1 acquisition, C-2 first command, C-3 TX-OFF proof | First 2–4 commanding passes | SOC ≈ 92 % from the end of day 0 | Ephemeris every pass and ≥ every 24 h until cataloged; identification by Doppler (SQSO-REG-015 §2) |
| 1–3 | C-4 health and power characterization (orbit energy balance from the telemetry tail and `EPS_GET_TELEMETRY`) | 2 a day | Generation ≈ 2.5 Wh/orbit drawn against 2.25 Wh used (the regulator curtails the rest at full charge); depth of discharge ≤ 2.1 % per eclipse (35.4 min eclipse, β 19.6°) | Compare with the simulation's orbits.csv |
| 2–4 | C-5 attitude: `ADCS_GET_ATTITUDE`, spin 8°/s, +Y within ≈ 4° of the Sun (mean 4.0°, BASELINE §8) | 2 a day | | |
| 3–5 | C-6 beacon conformance (deviation, bandwidth, cadence, `SET_BEACON`, `SET_POWER`) | 2 a day | | Spectrum recorded at two stations |
| 5–8 | C-7 transponder: first windows in commanded passes, then autonomous | 2 a day | Transponder keyed ≈ 38 min a day in the simulation (on 799 min a day; 96.9 % of demand served); duty 9.9 % | Public announcement only after C-7 passes |
| 8–11 | C-8 payload (Jetson) batches and fallback | 2 a day | ≈ 0.036 Wh per accepted beacon | Filter sweep on the ground |
| 11–13 | C-9 operations validation, contingency drill (`SAFE` and recovery) | 2 a day | | |
| 14 | C-10 review; `SET_CONTACT_TIMEOUT` 14 days | 1 | SOC ≈ 92 % | Commissioning report |

The order above is the plan, and the flight software now follows it: the commissioning hold keeps the spacecraft in
IDLE, with the Jetson and the transponder off, until `COMMISSIONING_RELEASE` (S+36.7 h in the simulation; RID LV-01).

## 7. Primary operations, end of life

Unchanged in substance from `mission/CONOPS.md` §2–§3 and `docs/operations/END_OF_LIFE_PLAN.md`: transponder windows
when sunlit and SOC ≥ 55 %; beacon every 60 s (ACTIVE), 120 s (IDLE), 180 s (SAFE); authenticated contact at least
once in each 14-day loss-of-contact period; ephemeris after every pass until cataloged, then by the plan of
SQSO-REG-015. End of life at day 730 (minimum 12 months): final beacon, TX-OFF, permanent charge-path disconnect;
re-entry no later than 5 years after the end of the mission (FCC; the 25-year figure of version 1.0 is withdrawn).
The simulation's full two-year mission (Rev E defaults) passivates at day 730 at 444.9 km and re-enters 2.0 years later, 4.0 years after deployment; its decay model is faster than the ODAR estimator's (2.8–3.7 years after the mission from 475 km, ODAR Draft D §6.2), which governs.

## 8. Contingencies, checked

### 8.1 Deployed in eclipse

33.3 % of the separation cases start in eclipse (≤ 28.8 min of darkness, β = 23.6° in November 2028; β stays
17–24° through the first year, so an eclipse of about 35 min exists in every orbit of the year). The load before the
burns is 0.43 W in the simulation: 0.20 Wh, 0.4 % of the pack (1.46 Wh with the board's detumble power and a cold pack,
POWER_BUDGET_ANALYSIS §8.6). 33 % of the cases burn at S+30 min in eclipse, with the hold-downs cold. Energy is not the
issue: one resistor takes 0.017 Wh, all four at the 12 s limit 0.10 Wh (`leop_feasibility.py` §2); release at the cold
temperature is, and is not verified (VI-LV-17). The retries per element are 60 s apart (`deployment.rs`
`RETRY_DELAY_MS`), so all of them happen in the same eclipse.

### 8.2 A deployable fails

Each element gets its attempts (for the wings: W1A, then W1B; W2A, then W2B, one line at a time), then the sequence
marks it FAILED and moves on; the ground can retry by `DEPLOY_RETRY` (0x50: 0 antenna, 1 solar wings; SQSO-ICD-013
§7), in sunlight (RID LV-07 closed). Consequences, simulated over 14 days at the Rev E baseline:

| Case | Result |
|------|--------|
| Wings not deployed | Commissioning completes; the AntS deploys after the wing attempts (S+3.0 h); lowest SOC 83.4 %; depth of discharge 2.8 % per eclipse; generation 445 Wh against 378 Wh used; no transponder service and one 729 s beacon gap (the simulation's checks: 2 anomalies, both expected for this case); lifetime on the wings-failed row of ODAR Draft D §6.2 (475 km: 5.7–6.1 yr nominal, inside the FCC rule; probability of this case 0.29 %) |
| Wings not deployed and ADCS failed (tumbling), separation SOC 38 % | Power-positive (406 Wh against 333 Wh); lowest SOC 62.3 %; depth of discharge 3.0 %; never detumbles (expected) |
| Antennas not deployed | Not simulated. The 2 m monopole stowed against the body radiates poorly; the link margin of ≈ +8 dB at 5° (SQSO-LBA-001 v3.2) would be lost on most passes, and the 70 cm uplink with it. `DEPLOY_RETRY` in sunlight is the recovery, if a command gets through |

### 8.3 The first passes are missed

A 3-day ground outage from separation (`contact_outage_days=3`): no anomaly; 22 of 23 later commands executed; the
loss-of-contact inhibit is 30 days at launch (D3), so the spacecraft keeps transmitting. The cost is regulatory and
contractual: the ephemeris clock (24 h from first contact); the commissioning hold keeps the transponder off until
`COMMISSIONING_RELEASE`, so it does not run unattended (RID LV-01 closed).

### 8.4 Low state of charge at separation (long storage, high drain, or a missed recharge)

| Separation SOC | Flight-software response (simulation) | First RF | Detumbled |
|----------------|----------------------------------------|----------|-----------|
| 58 % (reference) | DETUMBLE → IDLE at S+19 min | S+45 min | S+0.25 h |
| 38 % | Same path | S+45 min | S+0.25 h |
| 23 % | Boots into SAFE; IDLE at S+4.1 h | S+45 min | S+4.3 h |
| 13 % | SAFE → EMERGENCY at S+1 s (beacon shed); SAFE at S+2.6 h; IDLE at S+7.0 h | **S+2.6 h** | **S+7.4 h** (beyond 3 orbits) |
| 8 % | EMERGENCY until S+4.0 h; IDLE at S+8.4 h | **S+4.0 h** | **S+9.0 h** |

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
| First Bay Area pass ≥ 15° | Search ± 10 kHz around the downlink (Doppler ± 3.5 kHz at 145.9 MHz); update the elements from the SpaceX state vector and the 18th SDS candidates |
| S+24 h | Assume low SOC (§8.4) or a stowed antenna; keep listening every pass; send `GET_STATUS` blind on each pass (the receiver is on in every mode) |
| S+30 d | The loss-of-contact inhibit would stop the transmitter if no command was authenticated; it has no effect on a spacecraft that was never heard |

### 8.7 Unpowered clock

The OBC has no real-time clock through storage (none is fitted; CDS §2.3.3 would allow one), so at separation the
spacecraft knows only the time since boot (the beacon's `T=` field). UTC comes from the GNSS receiver once it runs, or
from ground correlation of the first frames. Version 1.0's "time within 1 s of UTC at delivery" cannot be met and is
withdrawn.

## 9. What the design does not yet provide

Provided since Version 2.1 (flight software round 2, BASELINE Rev E §4, §7): the 120 s magnetorquer hold (LV-02), the
commissioning hold (LV-01), the I²C AntS sequence and `DEPLOY_RETRY` (LV-07), the transponder and GNSS telecommands
(LV-14) and the latched RF silence (LV-15).

| Step | Missing | RID |
|------|---------|-----|
| First contact + 24 h | The ground orbit determination from the GNSS fixes and the CCSDS OEM upload (the receiver session is commanded by `GNSS_REQUEST`, 0x60) | LV-04 |
| LEOP passes | A second, geographically separated telecommand station (Santa Clara and Ames are 12 km apart) | LV-08 |
| Storage | A measured battery-side drain and a defined integration SOC | LV-06 |

## 10. Contact schedule

| Station | Role | Passes/day ≥ 5° | Passes/day ≥ 15° | Commanding min/day | First beacon pass after S (median, max) | First commanding pass after S (median, max) |
|---------|------|-----------------|------------------|--------------------|-------------------------------------------|----------------------------------------------|
| Santa Clara University, CA | Prospective telecommand station (D1) | 3.79 | 2.43 | 20.1 | 5.3 h, 12.1 h | 6.4 h, 13.7 h |
| NASA Ames, CA | Prospective telecommand station (D1) | 3.79 | 2.43 | 20.1 | 5.3 h, 12.1 h | 6.4 h, 13.7 h |
| Fairbanks, AK | Illustrative separated site | 8.98 | 4.99 | 41.6 | 2.1 h, 11.9 h | 4.4 h, 13.5 h |
| Morehead, KY | Illustrative separated site | 3.83 | 2.46 | 20.3 | 5.3 h, 12.1 h | 6.4 h, 13.7 h |
| SatNOGS and community | Receive only | — | — | — | — | — |

475 km SSO at 97.31°, period 94.1 min, from 2028-11-01; 576 separation cases, 14 days each, 10 s steps; minimum elevation for a usable
command pass 15°, for beacon reception 5° (the simulation's values). Decision C-14 allows commanding from 10° (2.97
commanding passes a day at Santa Clara instead of 2.43; SQSO-GND-001 Rev F §5), so these first-command statistics and
the simulation's 15° contact mask are conservative.

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR launch and ConOps review | Rewritten from the mission simulation and the pass analysis: delivery to the integrator, integration and storage with the battery drain, launch, a minute-by-minute LEOP with power, energy, battery and ground actions, the commissioning pass budget, contingencies checked by simulation (eclipse, failed deployables, missed passes, low SOC, switch bounce), the design gaps, and the contact schedule for the baseline orbit. Version 1.0 described P-POD deployment, the RF released at 30 min, a 400 km 51.6° orbit, a 25-year disposal rule and commands that do not exist |
| 2.1 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: §5 note on the changes since the simulation run (wings first, one resistor at a time at 0.67 A, magnetorquers 1.16–1.3 W from S+120 s, 8°/s, 2028-11, 475 km) and the orbit-1 energy figures that disagree; commissioning C-5 spin 8°/s |
| 2.2 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Re-run at the Rev E baseline with flight software round 2 (475 km, 2028-11-01, four burn lines): §5 LEOP table from the new commissioning run (magnetorquers from S+121 s, IDLE under the commissioning hold until `COMMISSIONING_RELEASE` at S+36.7 h, one wing resistor, then the AntS, orbit 1: 6.45 Wh generated against 0.89 Wh used; the §5 note removed) and from `leop_passes.py` at 475 km (eclipse at separation 33.3 %, ≤ 28.8 min); §6 pass budget and commissioning energy; §7 the full run's re-entry; §8 variants re-run (wings failed, low SOC, outage, eclipse); §9 items provided; §10 contact schedule |
| 2.3 | 2026-10-27 | CDR S&MA analysis (decision C-14) | §10: the 15° commanding mask of the pass statistics and the simulation noted as conservative against the 10° commanding rule |
| 2.4 | 2026-10-27 | CDR design (EPS board) | §3 shipped SoC aligned with SQSO-BSA-001 D §6 (≤ 30 % by air per the IATA 2026 guidance for PI 967; 60 % by road) |
