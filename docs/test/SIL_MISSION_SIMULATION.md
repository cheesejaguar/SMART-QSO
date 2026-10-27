# Mission Simulation (Software in the Loop)

**Document ID**: SQSO-SIL-001
**Version**: G
**Date**: 2026-10-27
**Status**: Draft

The mission simulation runs the SMART-QSO flight software, unchanged, against models of the spacecraft and its
environment for the whole mission: integration and key loading on the bench, storage in the dispenser, separation,
launch and early operations (LEOP), commissioning, the two-year design life (BASELINE §2), end-of-life passivation and
the orbit's decay to re-entry. It measures everything the requirements and the baseline put a number on, and judges a
run against them.

Code: `software/rust/sqso-sil` (the crate), `software/simulation/sil/report.py` (the report and plots). The models are
analysis-grade. They use the baseline's and the design documents' values and have not been correlated with hardware:
read a result as "the design closes with these assumptions", not as a prediction of flight.

## 1. Running it

```bash
cd software/rust
cargo build --profile sil -p sqso-sil
./target/sil/sqso_sil --scenario full                 # integration to re-entry, about 15 minutes
./target/sil/sqso_sil --scenario commissioning        # the first 14 days, about 15 s
./target/sil/sqso_sil --scenario smoke --set seed=7   # the first day; any setting with --set key=value
python3 ../simulation/sil/report.py sil-out/full      # report.md and plots/ beside the results
```

| Scenario | Covers |
|----------|--------|
| `full` | Integration (2028-09-15, the integrator's final charge to 60 % SOC, storage L-47 d) to deployment (2028-11-01 06:00 UTC, 475 km, LTAN 22:30), 730 days of operations, end-of-life passivation by telecommand (PASSIVATE_ARM then PASSIVATE_EXECUTE), then the decay to re-entry in 1-day steps |
| `year` | The first 365 days, no passivation |
| `commissioning` | The first 14 days |
| `ants_cold` | The first 14 days after an eclipse separation that follows the analysis's 2 h dispenser cold soak (Version F: `separate_at_eclipse`, `separation_soak_c` −20 °C; decisions C-18 and C-19) |
| `ants_cap` | Version G: `ants_cold` for 3 days with the AntS reading held at −30 °C (`ants_temp_fixed_c`): the 30-deferral cap and its release in sunlight (RID P3F-G-02) |
| `ants_notemp` | Version G: `ants_cold` for 3 days with no valid AntS temperature (`ants_temp_failed`): the schedule fallback after three unknown readings (RID P3F-G-02) |
| `smoke` | The first day (CI) |

The exit status is 0 when no check finds an anomaly, 1 otherwise. CI (`.github/workflows/ci.yml`, job *Mission
Simulation*) builds the crate, runs `smoke` and `commissioning`, and keeps their results and report as an artifact.
Settings (`--set`): `seed`, `altitude_km`, `ltan_h`, `integration_soc`, `storage_temp_c`, `mission_days`,
`passivate_at_eol`, `to_reentry`, `step_s`, `fine_step_s`, `record_s`, `detail_record_s`, `detail_days`,
`burn_limit_a`, `contacts_per_day`, `contact_min_elev_deg`, `commissioning_days`, `gauge_offset_a`, `radiation`,
`contact_outage_days`, `contact_outage_start_day`, and for the design trades `battery_scale`, `charge_v_cell`,
`array_years`, `gyro`, `adcs_failed`, `wings_failed`, `payload_stuck_on`, and since Version C `rx_failed_day` (the
command receiver fails for good on that day; its health line goes high), `rx_sefi` (receiver hangs from radiation, on
by default) and `dump_every_days` (the ground's DUMP_TLM plan), and since Version E `wing_a_failed` (the A resistors
heat but never part their lines), `eol_heater_ohms` (the end-of-life fuse's heater, 11.2–20.0 Ω), `separate_at_eclipse`
(separation at the first umbra entry after the deployment time) and `separation_soak_c` (the dispenser cold soak; since
Version F the whole network soaked 2 h against the analysis's cold wall profile ending at that temperature,
`Thermal::dispenser_soak`; in Version E only the structure and the AntS were set to it), and since Version G
`ants_temp_failed` (every AntS temperature reading invalid), `ants_temp_offset_c` (the reading's error),
`ants_temp_fixed_c` (the reading held at a value) and `ai_reject_fraction` (the share of the Jetson's lines meant to
fail the content filter, 3 % by default) (`sqso-sil/src/config.rs`). The ADCS Monte Carlo is a
separate binary, `sqso_adcs_mc` (`docs/hardware/BUILD_VS_BUY.md` §6).

The failed-receiver contingency of RID REL-04 (Version C):

```bash
./target/sil/sqso_sil --scenario full --set mission_days=60 --set rx_failed_day=20 \
    --set passivate_at_eol=false --set to_reentry=false
```

Its `rx-failed-beacon` check must pass; its `contact` anomaly (the inhibit's flag does set) and the commands warning
are the expected consequences of a receiver that hears nothing.

The AntS temperature gate and the transmitter hold (decisions C-18 and C-19, Versions E and F): an eclipse separation
after the analysis's cold dispenser soak.

```bash
./target/sil/sqso_sil --scenario ants_cold                              # the analysis's soak (wall −20 °C)
./target/sil/sqso_sil --scenario ants_cold --set separation_soak_c=-65  # the AntS at −44 °C at separation + 31 min
```

Its `ants-gate` check must pass (the AntS deferred while below −17 °C, deployed once warm) and its `ants-hold` check
(no transmission until the AntS reads deployed or its sequence ends, no VSWR stop into the stowed elements; SIL-41).
The soak is calibrated to THERMAL_ANALYSIS 2.4 §4.4: with the analysis's soak the AntS is at −17.5 °C at separation
(analysis −18.0 °C) and −35.9 °C at its coldest in the first eclipse (the analysis's deployment-orbit case −35.6 °C);
the −44 °C of the analysis's bounding case (β = 0, cold environment, tumbling) at separation + 31 min needs a −65 °C
wall in this model, which also takes the battery to −38.8 °C (its `tbatt` anomaly is the expected consequence of a soak
colder than any dispenser, §7).

## 2. How the flight software is in the loop

The simulation links `sqso-ffi`, the flight software behind the C headers, and runs its executive
(`Executive::start`, `Executive::step`) on the simulated HAL (`hal_sim.rs`), the same build the C suites test. Nothing
in the flight logic knows it is simulated. The simulation reaches it only where the hardware would:

| Interface | How |
|-----------|-----|
| Time | A virtual clock (`sqso_ffi::sil`): it starts at 150 ms at each boot, as the target's does, and advances by the simulation step. Steps are 1 s, or 0.1 s while the downlink is busy or a burn wire is on |
| EPS controller | Its telemetry registers on I²C bus 0, address 0x48 (`sqso_fsw::eps_telemetry`, SMART-QSO-ICD-001 §4.2): battery voltage, current, temperature, SOC, array voltage and current (the current measured at the bus, so array power is battery voltage × array current), bus, status bits, IMON_JET; the EOL_STATE register 0x7C (arm latch, fired, F1 intact, key) with the key at 0x7E (the simulated controller, `hal_sim`); the two INA228s (pack and array), their dies at the OBC/EPS stack node's temperature (the passivation fire's board condition) |
| ADC | Battery, array and bus monitors, board temperatures, the transmitter's TX_TEMP, TX_FWD and TX_REV |
| GPIO | The flight software's outputs (load switches, PA rail, power select, the four wing burn lines, heater, transponder enable, the end-of-life lines EOL_ARM and EOL_FIRE, the lockout-clear LOCK_CLR) are read back each step; the sense lines (separation DEPLOY_SENSE high = released, umbilical, the wings' series switches SOLAR_SENSE low = both deployed, the receiver's carrier detect RX_CD and health line RX_FAULT) are driven, with no internal pulls (SQSO-ICX-001 §3.7) |
| ADCS sensors | Magnetometer, coarse sun sensors and gyro, in the body frame (`adcs_set_sim_sensors`) |
| AntS | Its status word (0xC3) and, since Version E, its controller's temperature (0xC0, the AntS thermal node) on I²C3, both sides (0x31, 0x32) |
| Radio | Downlink frames are taken from the TNC path and put on the air by the radio model; the TNC answers the STATUS loop query (KISS SetHardware 0x06/0x01, answer 0x81: version, criteria, noise floor, carrier and frame counts, uptime); uplink frames (authenticated telecommands under the operational key, and PASSIVATE_EXECUTE under the recovery key, each key with its own counter; AX.25 with FCS) are injected into the receiver, when it is powered, not hung and not failed |
| Jetson | A pseudo-terminal: the flight software opens it as its Jetson port through its own serial driver and the payload model answers on the other end |
| GSE | Key provisioning, activation and the flight arming over the umbilical before launch |
| Non-volatile memory | The fault log, EPS configuration, mission data and state snapshot are files in the run's directory, kept across the simulated resets |

## 3. Models

| Model | Basis | Code |
|-------|-------|------|
| Orbit | Mean elements, J2 secular drift, drag decay of the semi-major axis (wings deployed, 0.0247 m², NASA-STD-8719.14C average area); 475 km sun-synchronous, LTAN 22:30 (Version B: 500 km) | `env/orbit.rs` |
| Atmosphere | Diffusive-equilibrium density with the Bates profile, driven by the solar activity of the filed lifetime (Version E): the NOAA SWPC predicted monthly F10.7, 2026-04 to 2030-12, as the ODAR's analysis embeds it (`software/simulation/cdr/sma/debris_lifetime.py`), then half-cosine cycles (minima 68 sfu, cycle 26 maximum 140 sfu at 2035.5, cycle 27 repeating it) | `env/atmosphere.rs` |
| Sun, eclipse | Low-precision solar coordinates (Astronomical Almanac); conical umbra and penumbra | `env/sun.rs`, `env/eclipse.rs` |
| Magnetic field | IGRF-14 tilted dipole with secular variation | `env/magfield.rs` |
| Attitude | Rigid body (Euler's equations, RK4) with the final CAD's inertia (MASS_BUDGET §6.1 and §6.5, CDR round 3; principal axes, products left out): stowed (2.960, 3.292, 2.970) × 10⁻³ kg·m²; deployed the mean of CBE (2.853, 4.202, 3.979) and MEV (3.276, 4.760, 4.546), (3.064, 4.481, 4.262) × 10⁻³, the flight software's `INERTIA` (Version F: MASS_BUDGET 3.2 §6.5; Version E (3.0645, 4.479, 4.2615)); one wing out (W1 or W2, STR-27) (3.027, 4.167, 3.911) × 10⁻³ (their products tilt the major axis 8.4° and 9.1°, left out here; the Monte Carlo of ADCS_DESIGN §11.2 has them). The angular momentum is kept as each wing swings out; magnetorquer torque from the flight software's dipole; residual dipole, gravity-gradient and aerodynamic torques; 10°/s tip-off | `env/attitude.rs`, `adcs_mc.rs` |
| ADCS sensors | Magnetometer (50 nT noise, 300 nT bias), six coarse sun sensors (5°), gyro (Epson M-G370PDT0 model: turn-on bias, bias instability, noise, scale factor, quantisation; `--set gyro=`) | `adcs_mc.rs` |
| Solar array | 16 cells in 8 strings on the body faces and the two wings, with incidence, shadowing by the body, albedo, temperature and radiation degradation | `env/solar.rs` |
| Battery | 4S1P INR18650-35E equivalent circuit with ageing (cycle and calendar fade) | `env/battery.rs` |
| EPS | MPPT, charge regulator (0.40 A CC, 4.10 V a cell CV, 0–45 °C window), BMS (2.8 V UV, 3.0 V reconnect, 4 A OC), regulators, load switches, fuel gauge; the Jetson at its carrier's input from VBAT through SW1 and the eFuse U19 (IMON_JET), the magnetorquers from VBAT (SW11, their I²R through the bridges for the burst, Version E); the hardware load shed (SQSO-EPS-BRD-001 E §8.4): SHED_N low after VBAT below 13.6 V for 5 s, high again above 14.4 V, gating SW1 and U19 only (not the PA, RID SMA-B05); the end-of-life charge disconnect (SQSO-EPS-BRD-001 D §7): the controller takes the key at EOL_KEY and sets the arm latch on an EOL_ARM rising edge while its key is up; with the latch set and EOL_FIRE high the fuse F1's heater (ITV4030, 11.2–20.0 Ω, 15.6 Ω nominal, `eol_heater_ohms`) warms, and inside its 6–35 W firing window the charge path opens after 30 s, reported in EOL_STATE. Nothing else passivates | `hw/eps.rs`, `mission.rs` |
| Thermal | 13-node lumped network (Version E): six panels, two wings, battery, OBC stack, RF board, Jetson and the ISIS AntS on the −Z face (THERMAL_ANALYSIS 2.4 §4.4: 77 J/K, black anodize α 0.88 ε 0.85; its 98 × 98 mm outer face and four 7 mm edges replace the −Z face's surface, its back radiates to the −Z frame, σ · 0.74 · 0.0096 m², and its mount conducts 0.02 W/K); solar, albedo and Earth IR loads; heaters and dissipation. The battery heater is a resistance of 207 Ω on the pack (V²/R: 0.70 W at 12.0 V, 1.3 W at 16.4 V), switched by the flight software at +5 / +9 °C. With `separation_soak_c` the whole network first soaks 2 h in the dispenser (Version F: radiation to the wall at ε_eff 0.70, 0.5 W/K to the rails, the analysis's cold wall profile −5 to −20 °C shifted to end at the setting; THERMAL_ANALYSIS 2.4 §4.3 case D3). The thermal analyst's round-3 cases (wing sub-nodes, the LEOP case L4) are folded in as `tests/thermal_round3.rs` (`cargo test -p sqso-sil --release --test thermal_round3 -- --ignored`) | `env/thermal.rs`, `mission.rs`, `tests/thermal_round3.rs` |
| Radio | KISS TNC and VHF PA (keying, airtime at 1200 bps, DC draw, PA heat, detectors); the FM transponder keying the PA on a CTCSS-valid carrier while `XPDR_EN` is high | `hw/radio.rs` |
| Link | 2 m downlink and 70 cm uplink after SQSO-LBA-001 v3.4 (Version E; v3.0 before): the AntS's 2 m tapes along ±X and 70 cm tapes along ±Y (the spin axis) with their patterns (floor −20 dBi), the polarization loss of a linear antenna against each station's (circular or linear) with Faraday rotation through the pass, the demodulators' required C/N₀ (1200 bps AFSK with and without FEC, telecommand) and their BER curves; the spacecraft receiver as designed, 948 K (598 K at the antenna terminals plus the 350 K in-band interference allowance) with 2 dB of implementation loss; frame success from the BER over the frame's bits. Telecommand only above 10° elevation (decision C-14) | `env/link.rs` |
| Ground | Nine stations (one with telecommand, KK6GDB-1, at decision C-14's minimum (SQSO-GND-001 Rev F §3.1 variant B): 50 W PEP through a 0.5 dB feed into 17.8 dBic, 64.0 dBm EIRP toward the spacecraft after 0.3 dB of pointing, 12 dBic receive, 500 K, Doppler-corrected; the others linear 5 dBi, 1000 K); passes, beacon reception, the commanding plan with the daily DUMP_TLM of the newest 48 records (from the HK1 `ST=` field) and the end-of-life plan (PASSIVATE_ARM, then PASSIVATE_EXECUTE 10 s later, within the 600 s window) | `env/ground.rs`, `env/link.rs`, `ops.rs` |
| Transponder users | Regional demand (North America, Europe, Japan and others), local-time activity, weekends; carrier present for a share of the time, 20 s overs | `qso.rs` |
| Deployables | Separation switches, the AntS antennas (released at the AntS node's temperature), and two wings, each with its own restraint, on four independent burn lines (one resistor each, SQSO-EPS-BRD-001), fed through the 7.0 V burn buck (86 %, 2.8 A limit; about 0.51 A at the pack per resistor at 12.8 V, 0.67 A at the board design's worst case), the EPS's 12 s one-shot per line, 10 s burns (decision C-17), release energy and cold factor; the wings' hinge switches in series (SOLAR_SENSE reads deployed only with both out) | `hw/deploy.rs` |
| Jetson | At the carrier's input (SQSO-JET-BRD-001 §13, the module capped at 6.5 W; Version E): 4.5 W booting, the capped burst 7.19–7.48 W (drawn once per power-on in that range) for the whole burst, 3.5 W idle as a latch-up's baseline; JET_PG 0.1 s after the rail; `SHUTDOWN` runs `poweroff` for 3 s, then the carrier latches off; the input current's field at the magnetometer (JETB-09); STATUS and BEACON_RSP answers on the line, 3 % unsafe texts for the filter | `jetson.rs` |
| GNSS receiver | piNAV-NG (decision D6): after an 80 s cold start a GGA and a `$PSLSP` state record every 10 s; the positions from its own arc, propagated with J2 from the orbit model's state at power-on (Version E: the mean-element orbit has no short-period J2, so the flight software's J2-consistent velocity would disagree with it by up to 0.5 m/s) | `mission.rs` |
| Radiation | Single-event functional interrupts of the OBC (about one in 90 days), Jetson latch-ups (about one in 60 powered days) and command-receiver hangs (about one in 120 days, cleared only by a power cycle), weighted by the South Atlantic Anomaly and the polar horns; total dose after RADIATION_ANALYSIS Rev C §4 (trapped 1,830 rad a year, region-weighted, plus 797 rad a year solar and galactic; depth scaling exponent 1.35) | `env/radiation.rs` |

## 4. What is measured

Each run writes to its output directory:

| File | Content |
|------|---------|
| `timeseries.csv` | The whole spacecraft each 10 s for the first days, then each 10 minutes: orbit, sun angle to +Y, rate, array and load power, battery current, voltage, true, gauge and flight-software SOC, capacity, temperatures, operational state and power mode, payload, radio and PA rails, radiating, heater, ADCS and dipole, deployables, best ground elevation, RF silence, contact inhibit, TX permission, transponder enable and keying, stations calling, uplink carrier |
| `orbits.csv` | Per orbit: eclipse, beta, energy generated and used, SOC range and depth of discharge, battery, PA and Jetson temperatures, radiating time, payload, heater, charge-blocked time, mode changes, OBC off time, mean rate, transponder enabled and keyed time |
| `passes.csv` | Each ground pass: station, times, elevation, frames sent and heard, commands |
| `events.jsonl` | Phases, deployment, resets, single-event effects, mode changes into or out of DETUMBLE, SAFE and EMERGENCY, commands, anomalies, passivation, re-entry |
| `console.log` | The flight software's console |
| `decay.csv` | Altitude by day after the design life to re-entry |
| `summary.json` | The checks with their values and verdicts, the totals (since Version C also the acknowledgements and dump frames radiated, the receiver hangs and recoveries, the time the receiver was known failed and the beacons then, passivation by telecommand, the most wing burn lines on at once), the fault-log entries by text |

## 5. Checks

`sqso-sil/src/checks.rs` judges a run. An *anomaly* fails it; a *warning* is reported.

| Area | Checks |
|------|--------|
| LEOP | First burn ≥ 30 min after separation (CDS 2.4.4); no RF before 45 min (CDS 2.4.5); antennas and wings deployed and sensed within 2 h; detumbled (rate across +Y below 0.5°/s) within 3 orbits (BASELINE §8) |
| Power | Depth of discharge per eclipse ≤ 10 % (BASELINE §5); lowest SOC in operations ≥ 40 %; generation over consumption; no under-voltage or over-current cut-off before passivation; no SAFE or EMERGENCY entry after commissioning; mode changes per orbit ≤ 4; capacity fade ≤ 20 %; gauge error ≤ 5 % |
| Thermal | Battery −10 to +45 °C; charging refused by the 0–45 °C window < 1 % of the time (SYS-602); PA ≤ 70 °C; Jetson ≤ 70 °C; OBC −20 to +60 °C |
| Downlink | Longest key-down ≤ 5 s and duty over 60 s ≤ 25 % (SRS-P002, the TNC's frames, the dump frames included); no frame while transmission is not permitted; beacon gaps ≤ 600 s (since Version C counted between beacons only: housekeeping, acknowledgement and dump frames are not beacons); ≥ 80 % of passes with a beacon decoded |
| Transponder | Never radiating while not permitted; radiating in eclipse or below 55 % SOC ≤ 0.5 % of its keyed time; keyed ≤ 25 % of an orbit; ≥ 70 % of the uplink demand repeated while its gates hold |
| Operations | No loss-of-contact inhibit; ≥ 90 % of telecommands executed, no authentication failure; no watchdog reset or brownout; every Jetson latch-up cleared; Jetson energy per accepted beacon ≤ 0.1 Wh (BASELINE §6; a warning, not a pass, when no AI beacon was sent); OBC availability ≥ 99 % |
| CDR round 2 (Version C) | `burn-lines`: at most one wing burn resistor on at once (EPS board); `rx-recovery`: every receiver hang cleared by the flight software's power cycle within 26 h (FSW-09: 24 h of silence, then the cycle; one still open at the end of a run is allowed); `rx-failed-beacon`: with `rx_failed_day` set, the receiver known failed, beacons radiated, gaps between them ≤ 600 s while it is, and the transponder never keyed (REL-04); without it, the receiver never declared failed; `acks`: acknowledgements radiated for ≥ 90 % of the commands executed (a warning below: TX_DISABLE gets none); `dumps`: dump frames radiated after commissioning |
| CDR round 3 (Version G) | `ants-commands` (RID P3F-A-01): 1 to 3 AntS deployment commands and none once its elements read deployed (the AntS answers only with SW8 on, so a deployed AntS read unpowered would be fired again); `ants-gate` extended: at the 30-deferral cap or on the schedule fallback the release must follow at least 19 min of continuous sun (the flight software's 20 min) and, on the schedule, come no earlier than separation + 69 min (RID P3F-G-02); `ai-bursts` (decision C-20): every Jetson power-on at most 3.75 kJ at the model's input (3.6 kJ at the carrier input, metered 20 ms late), the per-burst p50 and p95 and the Wh per line meant to pass the filter reported |
| CDR round 3 (Version F) | `ants-hold` (decision C-19): the first transmission at or after the end of the flight software's AntS hold (the AntS read deployed or its sequence ended), and no VSWR stop of the downlink with the AntS stowed unless its sequence ended without it (reported: the hold's end, the first RF, the VSWR stops); `ptt-watch` (SW-HAZ-014 control C-014-9, SRS-F168): no TX-OFF from the exciter watch; `ants-gate` also reports the AntS node's coldest in the first 3 h and when it first stood at or above −17 °C after the first attempt |
| CDR round 3 (Version E) | `load-shed`: hardware load sheds of SW1 and U19 before passivation (a warning above 0: the stuck-on payload contingency shows it acting); `gnss-frame`: a GNSS state frame (SRS-F158) per fix session, its position within 1 m of the receiver's record and its derived ECEF velocity within 0.5 m/s of the receiver's arc; `ants-gate`: the AntS told to deploy only at or above −17 °C, or after 30 deferrals (decision C-18; the AntS node's temperature at separation + 31 min, an ungated release, is reported with it); `antennas` allows 2 h plus 50 min per AntS deferral. `payload-limits`: no onset of the flight software's payload hard-limit condition (MSC L2-05; a stuck-on payload contingency shows it); `duty` counts exactly the last 60 s (Version D's window held 61 steps) and reports when its worst window ended |
| End of life | Passivated by telecommand (Version C: the two-key sequence through the flight software, not a modelled latch); no charge into the pack after passivation; re-entry within 5 years of the end of the mission (the FCC's five-year rule; CLAUDE.md, mission phase 3) |

## 6. Findings and changes

The first runs of the full mission had the true SOC at 1 % after 30 days and empty after 60. The flight software, the
design and the simulation's own bookkeeping were changed until every check passed. Each row says what the simulation
showed, the cause, and what changed.

| # | Showed | Cause | Change |
|---|--------|-------|--------|
| SIL-01 | Jetson on for all of ACTIVE: 37 of 57 Wh used a day; SOC collapses | The executive left the payload on whenever the mode manager was in ACTIVE | `sqso_fsw::payload_schedule`: batches of 10 beacon lines (BASELINE §6), 10 min at most, then 100 min of rest while the cached lines are spent one per beacon. The mode manager may switch the payload off but not on (`eps_hold_payload`). About 0.04 Wh per accepted beacon |
| SIL-02 | Battery below 0 °C for 18–20 % of every orbit: charging refused when the Sun came back | No heater control in the flight software | `sqso_fsw::heater`: on below +2 °C, off above +6 °C on the battery temperature, driving `HEATER_EN` |
| SIL-03 | With the heater on, the pack still followed the frame to −8 °C | THERMAL_DESIGN put the pack on a 1.0 W/K thermal pad (§4.2, §7.2) but sized the 0.5 W heater for about 0.03 W/K (§6.3) | Isolated battery mount: 0.05 W/K to the frame, 0.02 W/K to the OBC stack (THERMAL_DESIGN 1.1; to be confirmed with the thermal model and EM test of finding TB-07) |
| SIL-04 | Never detumbled; ADCS not running | The executive did not run the ADCS and declared the detumble complete at boot | The executive runs the ADCS at 1 Hz once separated; the state machine leaves DETUMBLE when the ADCS does; an idle ADCS in the mission is told to detumble again |
| SIL-05 | "Detumbled" after 5 min at 2.7°/s | The magnetometer rate estimate cannot see the rate about the field; B-dot leaves it | The gyro's rate (BASELINE §8 lists a MEMS gyro) is the rate estimate when it reads |
| SIL-06 | Detumble timeouts logged on every nominal tip-off | 90-minute timeout (ADCS_DESIGN) against about 2.5 orbits for B-dot to remove the rate about the field | Timeout 3 orbits, 285 min (BASELINE §8 "within 3 orbits"; ADCS_DESIGN 1.3) |
| SIL-07 | Tumbling: generation about the tumbling allocation, margin below the 20 % requirement | SUNPOINT pointed +X, a face with 2 of the 16 cells, while the deployed wings put 6 in the +Y plane; and its proportional law, with no damping, cannot converge on magnetorquers | Spin-stabilized coarse sun pointing of +Y, the deployed array's normal (BASELINE §8; `spin_sunpoint`, ADCS_DESIGN §4.2.1): +Y within about 2–10° of the Sun in sunlight |
| SIL-08 | The primary payload never ran | The radio interface had no transponder enable; nothing applied the power closure's gates | `XPDR_EN` (logical GPIO 16, PE4 provisional; TRANSMITTER_DESIGN 1.2) and `sqso_fsw::transponder`: permitted, radio on, ACTIVE, sunlit, SOC ≥ 55 %, PA < 70 °C (resume at 60 °C), keyed < 25 % of 100 min (resume at 20 %) |
| SIL-09 | High-discharge alarm during every deployment burn | 1.5 A threshold below two current-limited burn resistors plus housekeeping (about 1.9 A) | 2.5 A |
| SIL-10 | First telecommand lost | The plan commanded before the antennas deployed | Commands only in a pass in which the station has heard the spacecraft |
| SIL-11 | False anomalies: SAFE entries after resets, an under-voltage trip, 23 % of commands executed | The simulation's bookkeeping: SAFE and DETUMBLE swapped in its state names; the post-passivation drain counted; the flight software's counters, which restart at each boot, read as a maximum | Fixed in `sqso-sil` and `report.py` |
| SIL-12 | A Jetson latch-up lasted until the 10-minute batch timeout | The model drew only the 6 W latch current on a latched module, below the guard's 8 W trip | A latched module hangs drawing its idle current as well (about 9.5 W in all); the guard cuts the rail within a second |
| SIL-13 | ADCS Monte Carlo (`sqso_adcs_mc`, `docs/hardware/BUILD_VS_BUY.md` §6): 15 of 40 trials still tumbling after 2 days | B-dot gain 5·10⁶, 600 times the optimum: the momentum follows the turning field | Gain 10⁴: every trial detumbled, the slowest in 1.4 h (ADCS_DESIGN 1.4 §4.1) |
| SIL-14 | 23 of 40 trials at rest with −Y on the Sun, the wings facing away | The momentum law also accepts h = −h_s(−ŝ), with the spin reversed | Spin-hold term on the momentum about +Y (`SPIN_HOLD_GAIN`) |
| SIL-15 | +Y within 15° of the Sun for only 75 % of sunlit time; 163° at the worst eclipse exit | At 2°/s the spin is too soft against the disturbance torques | 6°/s, nutation gain 10⁻⁴: mean 2.5°, 100 % within 15°, 12° at the worst eclipse exit |
| SIL-16 | Each single-event reset cost a despin and a new spin-up (§8 of revision A) | The detumble's criterion, \|ω\| < 0.5°/s, treated the wanted spin as a tumble | With a gyro, the criterion is the rate across +Y below 0.5°/s and the spin below twice its set point |
| SIL-17 | Gyro bias ≥ 0.3°/s: Sun capture lost in some trials; uncalibrated BMI088 (±1°/s): in most | The bias across +Y looks like nutation, and the damping drives a real one | Gyro requirement ≤ 0.1°/s: Epson M-G370PDT0, BMI088 backup with ground calibration (BASELINE §8, D9) |
| SIL-18 | The pack at the end of charge for most of the mission | Generation exceeds the load in every orbit; the pack would sit full whatever its size | Charge set point 4.10 V a cell (BASELINE §5, D8; BUILD_VS_BUY §7). The model's fuel gauge then resets at the end of charge to the charge the set point implies (about 90 %), not to 100 %, as an OCV-correcting gauge would; without that the gauge read 8 % high |

Version C (2026-10-27, flight software round 2):

| # | Showed | Cause | Change |
|---|--------|-------|--------|
| SIL-19 | Smoke run: `ai-energy` anomaly, Jetson energy per accepted beacon far above 0.1 Wh | The simulation counted every HK1 frame as an AI beacon (its decoder read any frame from KK6GDB-11 as a beacon) | Frames classified as beacon, housekeeping, acknowledgement or dump (`ops.rs`); only beacons can be AI beacons, and the check is a warning when none was sent rather than a pass on no data |
| SIL-20 | Commissioning run: worst 60 s duty 25.7 %, over the 25 % limit | DUMP_TLM frames (181 bytes of information field, about 1.6 s on the air with TXDELAY) every 8 s on top of the beacon and HK1 | Dump pacing 12 s a frame (`hk_store::DUMP_FRAME_INTERVAL_MS`); duty then 18.7 % |
| SIL-21 | Failed-receiver contingency: the beacon silenced for 5.2 days after the loss-of-contact inhibit, before the receiver counted as known failed | The receiver's silence was counted from boot, and a single-event OBC reset restarted it, so the evidence waited a whole contact timeout again | The known-failed test uses the contact monitor's persisted expiry instead (SRS-F116); after a reset it waits for one new recovery cycle (24 h). Rerun: known failed 1 s after the inhibit, 37,812 beacons in 26.5 days, longest gap 66 s, transponder 0 s |
| SIL-22 | Version B's passivation could not fail | The simulation fired a modelled latch on the end-of-life TX_DISABLE; the flight software had no passivation | PASSIVATE_ARM and PASSIVATE_EXECUTE in the plan; the latch fires only through EOL_ARM and EOL_FIRE; the `passivation` check requires it by telecommand |
| SIL-23 | The burn current at the pack was the resistor current | Version B fed the resistors from the pack directly | The burn buck model: the pack current is the resistor power over the buck efficiency and the pack voltage; one resistor at a time |

Version E (2026-10-27, flight software round 3):

| # | Showed | Cause | Change |
|---|--------|-------|--------|
| SIL-24 | Both wings released by any one burn line; the wings sensed deployed with one out | The model had one restraint for both wings and one sense switch | Each wing its own restraint (W1A/W1B, W2A/W2B), the hinge switches in series (SOLAR_SENSE low only with both out), each release an event; the angular momentum kept as each wing swings out |
| SIL-25 | Magnetorquer power independent of the coils | The load was the commanded dipole's fraction of a fixed power | Each rod's I²R through its bridge for the 900 ms burst, from VBAT (SW11), with the bridges' awake current (SQSO-OBC-BRD-001 B §10) |
| SIL-26 | Jetson energy per beacon optimistic | The module drew its inference power only while a line was generated | The carrier document's figures at the carrier's input: 4.5 W booting, the capped 7.19–7.48 W for the whole burst (RID JETB-11), JET_PG and the `poweroff` sequence |
| SIL-27 | The decay did not use the solar activity of the filed lifetime | The atmosphere followed `orbit_lifetime.py`'s cycle points (cycle 25 maximum 185 sfu) | The ODAR's NOAA SWPC predicted F10.7 table, then the assumed cycles (atmosphere row of §3) |
| SIL-28 | Attitude with a uniform-box inertia | Version D derived the tensors from a box and two plates | The final CAD's tensors (stowed, deployed, one wing out; attitude row of §3), the deployed diagonal shared with the flight software |
| SIL-29 | The passivation could be fired by the OBC alone in the model | The latch set on any EOL_ARM edge | The two-processor arm: the latch sets only with the controller's key up (EOL_KEY 0x3C, 0xC3) and an EOL_ARM rising edge (RID P3E-11, SRS-F157) |
| SIL-30 | F1 with a 35 Ω heater would not fire at the pack's voltage | The data sheet's heater is 11.2–20.0 Ω with a 6–35 W firing window | 15.6 Ω nominal, both ends checked inside the window across 13.6–16.4 V (`eol_heater_ohms`) |
| SIL-31 | The final passivation report counted as frames while not permitted | The check forbade every frame once the charge path opened, but the report follows the opening by design (RID REG-12) | Forbidden only after the report window (180 s) plus 60 s |
| SIL-32 | The passivation fire never ended in the model's run: the charge path open, the fire still waiting for sunlight | The opened path removes the array, and with it the sunlight the flight software sees | Flight software: the EPS reporting the path open ends the fire whatever the sunlight (SRS-F111) |
| SIL-33 | `ai-energy` understated about 2.4 times (commissioning: 0.026 Wh over 6,390 beacons) | HK2 frames, new in round 3, were counted as AI beacons | HK2 classified as housekeeping, the GNSS state frame as its own kind: 0.064 Wh over 2,633 beacons in that run |
| SIL-34 | A GNSS state frame per session needed checking | New frame (RID LV-04, SRS-F158) | `gnss-frame` check against the receiver's records; the receiver's arc propagated with J2 (§3), after which the derived velocity agrees within 0.001 m/s (it was up to 0.54 m/s against the mean-element orbit, the model's error) |
| SIL-35 | The link margin at the receiver as designed | LBA v3.0 assumed a better receiver and other antenna axes | LBA v3.4 values (link row of §3): 8.3 dB of telecommand margin at 10°, 6.5 dB at 5°, 64.0 dBm EIRP |
| SIL-36 | The separation and wing sense lines read with the wrong polarity | The model drove them as the flight software expected before RID SMA-B02 | DEPLOY_SENSE high = released, SOLAR_SENSE low = both deployed, as the EPS drives them, and no internal pulls in the flight software |
| SIL-37 | Eclipse separation after a cold soak (`separate_at_eclipse`, `separation_soak_c` −30 °C): the AntS at −18.2 °C at separation + 31 min; the flight software deferred it one slot and deployed it at +1.4 °C, 80.7 min after separation. The beacon gap was then 1,312 s | The gate works (decision C-18). The gap: from the end of the RF silence (45 min) until the AntS was out the downlink keyed into the stowed antennas, its VSWR protection tripped (above 3:1) and stopped the downlink for 10 minutes, twice | Reported for a decision (RF lead): accept the VSWR stops while the AntS is deferred (up to 25 h at the cap), or hold the downlink until the AntS reads deployed. Decided by the chair (C-19) and implemented in Version F (SIL-41) |
| SIL-38 | The full run's worst 60 s downlink duty 25.3 %, over the 25 % check | The check's window held 61 one-second steps (61 s of keying counted against 60 s); the flight software's limit is a credit of one 60 s window at 25 % (SRS-I025) | The window counts exactly 60 s and reports when it ended: 23.7 % in the reference run (§7) |
| SIL-39 | The flight software's own payload hard-limit condition (MSC L2-05: the Jetson on only in ACTIVE, sunlit, SOC ≥ 55 %, within its burst limit) logged 4,497 onsets in the full run (78 in commissioning), each cleared some 530 s later | The batch schedule started and kept a batch on the mode alone (ACTIVE): a batch begun before an eclipse ran on into it, and one could start in eclipse | Flight software: a batch starts only, and ends at once, within the hard limits (`run_payload_schedule`, SRS-F166). A rerun still showed 2 onsets: a switch-on in the last second of sunlight started its batch on the next pass, in eclipse; the schedule now starts a batch only while it is allowed (`payload_schedule`). New check `payload-limits` (0 onsets) |

Version F (2026-10-27, flight software round 3, fourth part):

| # | Showed | Cause | Change |
|---|--------|-------|--------|
| SIL-40 | Version E's full run radiated 5,364 dump frames with 20,256 records against Version D's 8,664 and 34,656. From day 140.5 every daily DUMP_TLM gave one frame with no record | HK1 had grown past its 250-byte frame (every field at its widest is 341 bytes; in the run its counters crossed the limit at day 140.5, and on some frames before). `housekeeping::format` then returned nothing and the executive sent an **empty** HK1 frame; the ground's `ST=` froze at 40,480, so each dump asked for records the 7.1-day store had overwritten. Not a deliberate change (the payload-batch gating, the acknowledgement rule, the duty limit and the pass plan were not involved); the worst-case tests had passed on the empty text. HK2 (474 bytes at its widest) had the same latent defect | Flight software: the housekeeping text is formatted into 512 bytes and cut into frames of whole fields (`housekeeping::Parts`), each frame after the first repeating the `HKn N=<n> ` head, 5 s apart, HK1's then HK2's; an empty frame is never sent; the worst-case tests now check the whole text and its frames (SRS-F085). Rerun (§7): 8,664 dump frames, 34,656 records, as Version D; Version E's `ai-energy` had counted the empty frames as AI beacons |
| SIL-41 | With an eclipse separation the downlink keyed into the stowed AntS (SIL-37) | Nothing held the transmitters for the AntS | Decision C-19, SRS-F167: the transmitters stay silent (the receiver on) while the AntS deployment is pending, until the AntS reads deployed or its sequence ends (attempts spent, or the attempt at the 30-deferral cap made), latched; `ants-hold` check. `ants_cold`: first RF 81.2 min, when the AntS was out; no VSWR stop; longest beacon gap 126 s |
| SIL-42 | The cold soak left the AntS at −18.2 °C at separation + 31 min from −30 °C (Version E) | Only the structure and the AntS were set cold; the internal nodes at the storage temperature warmed the frame | The whole network soaks 2 h against the analysis's wall profile (`Thermal::dispenser_soak`): with its −20 °C the AntS is at −17.5 °C at separation (analysis −18.0) and −35.9 °C at its coldest (analysis −35.6 for this orbit); scenario `ants_cold` |

Version G (2026-10-27, flight software round 3, fifth part: the closure panel's findings):

| # | Showed | Cause | Change |
|---|--------|-------|--------|
| SIL-43 | The AntS answered on I2C whatever its supply, so the flight software's reads with the AntS unpowered (after the burn's STOP had lowered SW8) found it deployed in the model and not on hardware (RID P3F-A-01) | The simulated HAL wrote and read the AntS registers every step regardless of SW8 | `hal_sim` power-gates the AntS: it answers only with SW8 on (DEPLOY_EN or the EPS controller's ANTS_PWR) and 0.5 s after it turned on, and forgets its arm and deployment commands when SW8 drops; the model writes its status and temperature through a back door. Flight software: every AntS read with SW8 raised for 1 s first (SRS-F169). New check `ants-commands`: one AntS command in every scenario, none with the elements deployed |
| SIL-44 | The repeat path's key-down timer was not modelled: an uplink carrier held past 120 s kept XPDR_KEY high | The radio model keyed the repeat path for the whole carrier | The 120 s timer (reset when the tone drops) in `mission.rs`; the flight software's exciter watch now bounds an over at 130 s (SRS-F168, RID P3F-A-02), so without the timer the model would have tripped it |
| SIL-45 | The AntS gate's two ends were exercised only in unit tests (RID P3F-G-02) | No setting forced them | Settings `ants_temp_fixed_c`, `ants_temp_offset_c`, `ants_temp_failed`; scenarios `ants_cap` (reading −30 °C: 30 deferrals, released at S + 1,548.7 min after 19.9 min of sun, the AntS at −13.1 °C, first RF 1,549.5 min) and `ants_notemp` (no valid reading: two deferrals, then the schedule, released at S + 135.3 min after 19.9 min of sun at −13.8 °C, first RF 136.1 min). Both 0 anomalies |
| SIL-46 | The energy per accepted beacon was an aggregate only (RID P3F-G-03) | No per-burst record | The Jetson model logs each power-on's input energy and its lines meant to pass the filter; check `ai-bursts` and summary values `jetson_burst_j_p50`, `_p95`, `jetson_wh_per_good_line`; setting `ai_reject_fraction` for the sensitivity of decision C-20 (§7) |

Without the EPS controller answering (the stand-in power model of the development simulation, SW-01) the executive
keeps its previous behaviour: detumble completes in the first pass and the payload follows the power mode. The C
suites run that way, unchanged except where SIL-06 changed a specified value.

## 7. Results

The reference run: `--scenario full`, seed 20280601, integration 2028-09-15, deployment 2028-11-01 06:00 UTC, 475 km
sun-synchronous at LTAN 22:30, 730 days of operations, passivation by telecommand, then the decay to re-entry. Version G
(2026-10-27, flight software round 3, fifth part: the AntS read only while powered and the power-gated AntS model, the
persisted separation, the bounded repeat path and the key-down timer, the IWDG paired with the WDI edge, the AntS gate's
persisted slot and its sunlit releases, the energy gate on accepted lines, the HK2 selector, preset and sun fields)
found **0 anomalies and 0 warnings in 50 checks** (32 minutes, sharing the machine with other runs). Version G changed
the values below (Version F in brackets): first RF and the AntS unchanged (45.0 min; the AntS told to deploy once, at
+2.9 °C after 9.3 min of sun, deployed at 30.6 min); depth of discharge 3.5 % (3.4 %); 29,523 Wh drawn and 26,283 Wh used
(29,566 and 26,322); longest beacon gap 245 s (191 s); worst 60 s duty 24.0 % (23.8 %); 1,682,651 frames (1,664,437);
95.9 % of 28,567 passes heard (95.7 % of 28,563); transponder at most 12.7 % of an orbit keyed (14.9 %), 95.5 % of the
demand repeated (95.8 %); 9,227 Jetson power-ons (9,216), 8,077 Wh (8,145), 125,127 AI beacons (126,292), **0.065 Wh per
accepted beacon** (0.064); every burst at most 3,633 J at the model's input (p50 3,626 J, p95 3,630 J: the
batches end at the 3.6 kJ cap, decision C-20), 0.0652 Wh per line meant to pass the filter; 0 Jetson latch-ups in this
run's radiation draw (1); GNSS position within 0.81 m (0.84 m). The dumps are unchanged (8,664 frames, 34,656 records).
The Version F values follow:

| Area | Result |
|------|--------|
| LEOP | First burn 30.0 min and first RF 45.0 min after separation; wings sensed deployed at 30.2 min (W1A then W2A), antennas at 30.6 min; at most one burn resistor on at once; the AntS told to deploy at +2.8 °C, no deferral, so the C-19 hold ended at 31.2 min, before the silence; no VSWR stop in the whole run; detumbled after 0.2 h |
| Attitude | Spin about +Y at 8.05°/s (median in sunlight); from day 3, in sunlight +Y within 1.9° of the Sun on average (95th percentile 3.9°, 99.99 % of the 10-minute records within 15°) |
| Power | Depth of discharge at most 3.4 % per eclipse (3.5 %); lowest SOC in operations 90.8 %; 29,566 Wh drawn from the array and 26,322 Wh used (29,398 and 26,173); capacity fade 1.5 %; gauge error at most 0.4 %; no hardware load shed |
| Thermal | Battery +4.9 to +20.0 °C, never outside the charge window; OBC −7.5 to +24.6 °C; PA at most 23.8 °C; Jetson at most 28.9 °C |
| Downlink | 2.20 s longest key-down, 23.8 % worst 60 s duty (day 86); longest beacon gap 191 s (249 s); 95.7 % of 28,563 passes with a beacon decoded; 1,664,437 frames (1,467,673), none while not permitted, none empty; keyed 2,344,368 s (1,913,101 s): 208,263 HK1 texts, 193,563 of them in a second frame, and 208,262 HK2 texts, all in one frame |
| Transponder | Enabled about 890 min and keyed about 53 min a day; at most 14.9 % of an orbit keyed; 95.8 % of the uplink demand repeated while its gates held; never radiating while not permitted; 157 s in all (0.01 % of its keyed time) within one pass of an eclipse entry |
| Jetson | 126,292 beacons with Jetson text from 9,216 power-ons; 8,145 Wh; **0.064 Wh per accepted beacon** (gate 0.1 Wh; Version E's 0.025 Wh over 319,795 counted the empty HK1 frames of SIL-40 as AI beacons, since an empty information field is neither housekeeping nor a template; the commissioning run's 0.064 Wh of SIL-33 is the same figure); no onset of the payload hard-limit condition; one latch-up, cut by the 8 W guard within 1.0 s |
| GNSS | 2,918 state frames for 2,922 sessions with a fix, every one with a velocity; position within 0.84 m of the receiver's record, velocity within 0.001 m/s of its arc |
| Operations | 1,449 of 1,463 telecommands executed (the rest lost on the link), no authentication failure, no loss-of-contact inhibit; 1,448 acknowledgements radiated (99.9 %); **8,664 dump frames with 34,656 stored records** (Version E 5,364 and 20,256; Version D 8,664 and 34,656): 722 daily dumps of 48 records, every frame with four (SIL-40); 9 single-event OBC resets, each recovered through DETUMBLE; 7 command-receiver hangs, each cleared by the flight software's power cycle (longest 24.9 h); no watchdog reset or brownout; no exciter-watch trip; OBC availability 100.000 %; total dose 4.9 krad |
| End of life | PASSIVATE_ARM and PASSIVATE_EXECUTE at day 730.1, the charge path opened through the EPS's key, EOL_ARM and EOL_FIRE; no charge into the pack afterwards; re-entry 2.55 years after the design life (4.55 years after deployment) |

The mission success criteria's telemetry (SQSO-MSC-001 §7) comes back whole: every HK1 and HK2 text of the run went out
complete (no field lost, no empty frame), and the stored records the ground asked for were all returned, against
none after day 140.5 in Version E. The cost is the HK1 continuation frame, about 0.5 s on the air every 5 minutes
(the keyed time rises 23 %, mostly from HK1's now non-empty first frame), inside the duty limit (worst 23.8 %).

The fault log records conditions at their onset and their clearing (SRS-F153): the low-SOC and low-voltage entries
are one each. "Payload enabled" and "Payload disabled" (9,216 and 9,217) are the batches; "Entered SAFE mode" (12) is
the EPS power mode at each of the 11 boots and after passivation, not the operating state.

Version G, the other scenarios: `commissioning` 0 anomalies and 0 warnings (48 checks; first RF 45.0 min, one AntS
command, the wings at their first attempt); `smoke` 0 anomalies (the one-day warnings); `ants_cold` 0 anomalies (first
RF 81.2 min, one deferral, one AntS command, no VSWR stop, unchanged); `ants_cap` and `ants_notemp` 0 anomalies (SIL-45;
their warnings the cold soak's battery and OBC and the short runs' pass and acknowledgement counts). Decision C-20's
sensitivity, `commissioning` with `ai_reject_fraction` 0.03 / 0.10 / 0.30: 165 / 166 / 168 bursts, p50 3,625 / 3,625 /
3,624 J, p95 3,629 / 3,630 / 3,629 J; energy per line meant to pass the filter 0.065 / 0.070 / 0.090 Wh, per AI beacon
sent 0.065 / 0.068 / 0.080 Wh: the 0.1 Wh gate holds to 30 % rejection with 10 % margin, under the model's
assumptions (20–30 s boot, no model-load phase, lines at the OBC's request cadence).

Version F, the other scenarios with the same code: `commissioning` 0 anomalies and 0 warnings; `smoke` 0 anomalies (its warnings,
no transponder service, no AI beacon and 2 of 3 acknowledgements, are those of a one-day run); `ants_cold` (the
analysis's soak) 0 anomalies, its warnings the cold battery (−7.2 °C at separation) and OBC (−21.9 °C) of the soak:
the AntS at −17.5 °C at separation, −18.7 °C at separation + 31 min, −35.9 °C at its coldest; the first attempt
(S+30.4 min) deferred, the gate open (the AntS at or above −17 °C) from 32.6 min, the AntS deployed at the next slot,
80.7 min, at −3.7 °C; the transmitters silent until then (first RF 81.2 min, decision C-19), no VSWR stop, longest
beacon gap 126 s. With `separation_soak_c=-65` (the AntS at −44.6 °C at separation + 31 min, the analysis's bounding
release temperature): deferred once, the gate open from 49.7 min (the analysis's β = 0 cases 69–177 min), deployed at
80.7 min at −15.8 °C, first RF 81.2 min; its one anomaly is the battery at −38.8 °C, the soak's (expected). With the
wings failed as well (`ants_cold`, `wings_failed=true`): the AntS deployed after the wings' attempts at 181.3 min,
the first RF then (181.9 min), no VSWR stop; its anomaly is the wings' own.

## 8. Limits and open items

- The models are not correlated with hardware. The battery mount (SIL-03), the transponder's repeat-path hardware
  behind `XPDR_EN` (SIL-08) and the gyro (SIL-05) are design assumptions to confirm (TB-07; TRANSMITTER_DESIGN §10.1).
- The receiver supervision's inputs (carrier detect RX_CD, health line RX_FAULT) are two lines and the TNC's STATUS
  answer has fixed fields (version, criteria, noise floor at its calibrated value, counts, uptime); the answer's byte
  layout is defined on the OBC side and is to be confirmed with the TNC firmware. The contingency shows the logic, not
  the hardware.
- The reference run separates warm (every node at the storage temperature; the AntS at +4.2 °C at separation + 31 min).
  The `ants_cold` contingency soaks the whole network with the analysis's dispenser profile (Version F); it agrees with
  THERMAL_ANALYSIS 2.4 §4.4 at separation (−17.5 against −18.0 °C) and on the first eclipse's minimum in the deployment
  orbit (−35.9 against −35.6 °C), but its AntS warms faster after that minimum (−18.7 °C at separation + 31 min, the
  analysis −35.1 °C): after its detumble at 0.2 h the model's spin keeps the −Z face's view of the Earth larger than the
  analysis's tumbling case. The analysis's bounding −44 °C at separation + 31 min (β = 0, cold environment) is reached
  only with a −65 °C wall, colder than any dispenser. The AntS temperature reading's command and conversion are a
  documented assumption until the ISIS ICD (SQSO-MFG-003 §12 V-1).
- The attitude model uses principal axes: the products of inertia (the one-wing tensors tilt the major axis 8.3° and
  9.1°) are in the ADCS Monte Carlo of ADCS_DESIGN §11.2, not here.
- The thermal network's AntS node changes the −Z face's balance; the folded round-3 cases (`tests/thermal_round3.rs`)
  were the analyst's 12-node results and now run on 13 nodes.
- The end-of-life fuse and its heater (35 Ω, 30 s) and the battery heater (207 Ω) are design values; the burn buck's
  efficiency (86 %) is a data-sheet value at the burn load.
- The ADCS gains and the spin (8°/s in the flight software since the CDR; 6°/s in Version B) are set by simulation only (SIL-13 to SIL-17). The attitude model has no flexible
  modes, no sensor misalignment and no magnetorquer remanence. The tip-off distribution and the inertia are
  assumptions.
- The thermal network is lumped (13 nodes) and its internal conductances are assumptions.
- The Jetson's inference and the onboard AI selector are represented by their power and their answers on the line,
  not by models.
- The battery's calendar fade depends on the state of charge only linearly in the model. Published data show a
  larger benefit from a lower end-of-charge voltage than the model gives (SIL-18), so the model's fade at 4.10 V
  is pessimistic.

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| B | 2026-10-26 | The 1.4 ADCS gains, the 6°/s spin, the M-G370PDT0 gyro model and the 4.10 V charge set point; §7 reference run at 500 km, 2028-06-01 |
| C | 2026-10-27 | Flight software round 2 (§1 to §6, §8): 475 km, 2028-11-01, integration at 60 % SOC and L-47 d; LBA v3.0 link, CAD inertia, four burn lines and the burn buck, 207 Ω heater, Rev C dose, receiver hangs, the EOL latch through EOL_ARM and EOL_FIRE; new settings, checks and findings SIL-19 to SIL-23 |
| D | 2026-10-27 | §7 with the round-2 reference run (42 checks, 0 anomalies, 0 warnings): the attitude row from the time series, the fault-log entries explained (low SOC and voltage after passivation; SAFE power mode at each boot) |
| E | 2026-10-27 | Flight software round 3 (§1 to §8): settings `wing_a_failed`, `eol_heater_ohms`, `separate_at_eclipse`, `separation_soak_c` and the AntS contingency; §2 the EPS key, the INA228 dies, the AntS temperature, the TNC STATUS answer; §3 the ODAR's solar activity, the final CAD inertia with one wing out, the load shed (SW1, U19), F1's heater, the 13-node thermal network with the AntS, LBA v3.4, the Jetson at its carrier's input, the GNSS receiver's J2 arc; §5 checks `load-shed`, `gnss-frame`, `ants-gate`, `payload-limits`, the 60 s duty window; §6 findings SIL-24 to SIL-39; §7 the round-3 reference run (46 checks, 0 anomalies, 0 warnings); §8 the warm separation, the AntS reading's assumption, principal axes |
| F | 2026-10-27 | Flight software round 3, fourth part: scenario `ants_cold`; the whole-network dispenser soak (`separation_soak_c`); MASS_BUDGET 3.2 inertia; §5 checks `ants-hold`, `ptt-watch` and the gate's opening time; §6 findings SIL-40 (the empty HK1 frame and the dumps), SIL-41 (decision C-19), SIL-42 (the soak); §7 the round-3 reference run again; §8 the cold separation |
| G | 2026-10-27 | Flight software round 3, fifth part: scenarios `ants_cap`, `ants_notemp`; settings `ants_temp_failed`, `ants_temp_offset_c`, `ants_temp_fixed_c`, `ai_reject_fraction`; the AntS power-gated, the repeat path's key-down timer; §5 checks `ants-commands`, `ai-bursts` and the sunlit releases in `ants-gate`; §6 SIL-43 to SIL-46; §7 the Version G reference run (50 checks, 0 anomalies, 0 warnings) and the reject-fraction sensitivity |
