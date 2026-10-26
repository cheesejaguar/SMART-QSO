# Mission Simulation (Software in the Loop)

**Document ID**: SQSO-SIL-001
**Version**: A
**Date**: 2026-10-26
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
| `full` | Integration (2028-04-15) to deployment (2028-06-01 06:00 UTC), 730 days of operations, end-of-life TX-OFF and passivation, then the decay to re-entry in 1-day steps |
| `year` | The first 365 days, no passivation |
| `commissioning` | The first 14 days |
| `smoke` | The first day (CI) |

The exit status is 0 when no check finds an anomaly, 1 otherwise. CI (`.github/workflows/ci.yml`, job *Mission
Simulation*) builds the crate, runs `smoke` and `commissioning`, and keeps their results and report as an artifact.
Settings (`--set`): `seed`, `altitude_km`, `ltan_h`, `integration_soc`, `storage_temp_c`, `mission_days`,
`passivate_at_eol`, `to_reentry`, `step_s`, `fine_step_s`, `record_s`, `detail_record_s`, `detail_days`,
`burn_limit_a`, `contacts_per_day`, `contact_min_elev_deg`, `commissioning_days`, `gauge_offset_a`, `radiation`,
`contact_outage_days`, `contact_outage_start_day` (`sqso-sil/src/config.rs`).

## 2. How the flight software is in the loop

The simulation links `sqso-ffi`, the flight software behind the C headers, and runs its executive
(`Executive::start`, `Executive::step`) on the simulated HAL (`hal_sim.rs`), the same build the C suites test. Nothing
in the flight logic knows it is simulated. The simulation reaches it only where the hardware would:

| Interface | How |
|-----------|-----|
| Time | A virtual clock (`sqso_ffi::sil`): it starts at 150 ms at each boot, as the target's does, and advances by the simulation step. Steps are 1 s, or 0.1 s while the downlink is busy or a burn wire is on |
| EPS controller | Its telemetry registers on I²C bus 0, address 0x48 (`sqso_fsw::eps_telemetry`, SMART-QSO-ICD-001 §4.2): battery voltage, current, temperature, SOC, array voltage and current, bus, status bits |
| ADC | Battery, array and bus monitors, board temperatures, the transmitter's TX_TEMP, TX_FWD and TX_REV |
| GPIO | The flight software's outputs (load switches, PA rail, power select, burn lines, heater, transponder enable) are read back each step; the sense lines (separation, umbilical, antenna and wing switches) are driven |
| ADCS sensors | Magnetometer, coarse sun sensors and gyro, in the body frame (`adcs_set_sim_sensors`) |
| Radio | Downlink frames are taken from the TNC path and put on the air by the radio model; uplink frames (authenticated telecommands, AX.25 with FCS) are injected into the receiver |
| Jetson | A pseudo-terminal: the flight software opens it as its Jetson port through its own serial driver and the payload model answers on the other end |
| GSE | Key provisioning, activation and the flight arming over the umbilical before launch |
| Non-volatile memory | The fault log, EPS configuration, mission data and state snapshot are files in the run's directory, kept across the simulated resets |

## 3. Models

| Model | Basis | Code |
|-------|-------|------|
| Orbit | Mean elements, J2 secular drift, drag decay of the semi-major axis; 500 km sun-synchronous, LTAN 22:30 | `env/orbit.rs` |
| Atmosphere | Diffusive-equilibrium density with the Bates profile, driven by the solar cycle | `env/atmosphere.rs` |
| Sun, eclipse | Low-precision solar coordinates (Astronomical Almanac); conical umbra and penumbra | `env/sun.rs`, `env/eclipse.rs` |
| Magnetic field | IGRF-14 tilted dipole with secular variation | `env/magfield.rs` |
| Attitude | Rigid body (Euler's equations, RK4) with the stowed and deployed inertias; magnetorquer torque from the flight software's dipole; residual dipole, gravity-gradient and aerodynamic torques; 10°/s tip-off | `env/attitude.rs` |
| Solar array | 16 cells in 8 strings on the body faces and the two wings, with incidence, shadowing by the body, albedo, temperature and radiation degradation | `env/solar.rs` |
| Battery | 4S1P INR18650-35E equivalent circuit with ageing (cycle and calendar fade) | `env/battery.rs` |
| EPS | MPPT, charge regulator (0.40 A CC, 4.20 V a cell CV, 0–45 °C window), BMS (2.8 V UV, 3.0 V reconnect, 5 A OC), regulators, load switches, fuel gauge, end-of-life passivation latch | `hw/eps.rs` |
| Thermal | 12-node lumped network: six panels, two wings, battery, OBC stack, RF board, Jetson; solar, albedo and Earth IR loads; heaters and dissipation | `env/thermal.rs` |
| Radio | KISS TNC and VHF PA (keying, airtime at 1200 bps, DC draw, PA heat, detectors); the FM transponder keying the PA on a CTCSS-valid carrier while `XPDR_EN` is high | `hw/radio.rs` |
| Link | 2 m downlink and 70 cm uplink budgets after SQSO-LBA-001, frame success against margin | `env/link.rs` |
| Ground | Nine stations (one with telecommand, KK6GDB-1); passes, beacon reception, the commanding plan | `env/ground.rs`, `ops.rs` |
| Transponder users | Regional demand (North America, Europe, Japan and others), local-time activity, weekends; carrier present for a share of the time, 20 s overs | `qso.rs` |
| Deployables | Separation switches, two antennas and two wings on two burn lines, current-limited resistors, release energy and cold factor | `hw/deploy.rs` |
| Jetson | Boot, idle and inference power; STATUS and BEACON_RSP answers on the line, 3 % unsafe texts for the filter | `jetson.rs` |
| Radiation | Single-event functional interrupts of the OBC (about one in 90 days) and Jetson latch-ups (about one in 60 powered days), weighted by the South Atlantic Anomaly and the polar horns; total dose | `env/radiation.rs` |

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
| `summary.json` | The checks with their values and verdicts, the totals, the fault-log entries by text |

## 5. Checks

`sqso-sil/src/checks.rs` judges a run. An *anomaly* fails it; a *warning* is reported.

| Area | Checks |
|------|--------|
| LEOP | First burn ≥ 30 min after separation (CDS 2.4.4); no RF before 45 min (CDS 2.4.5); antennas and wings deployed and sensed within 2 h; detumbled below 0.5°/s within 3 orbits (BASELINE §8) |
| Power | Depth of discharge per eclipse ≤ 10 % (BASELINE §5); lowest SOC in operations ≥ 40 %; generation over consumption; no under-voltage or over-current cut-off before passivation; no SAFE or EMERGENCY entry after commissioning; mode changes per orbit ≤ 4; capacity fade ≤ 20 %; gauge error ≤ 5 % |
| Thermal | Battery −10 to +45 °C; charging refused by the 0–45 °C window < 1 % of the time (SYS-602); PA ≤ 70 °C; Jetson ≤ 70 °C; OBC −20 to +60 °C |
| Downlink | Longest key-down ≤ 5 s and duty over 60 s ≤ 25 % (SRS-P002, the TNC's frames); no frame while transmission is not permitted; beacon gaps ≤ 600 s; ≥ 80 % of passes with a beacon decoded |
| Transponder | Never radiating while not permitted; radiating in eclipse or below 55 % SOC ≤ 0.5 % of its keyed time; keyed ≤ 25 % of an orbit; ≥ 70 % of the uplink demand repeated while its gates hold |
| Operations | No loss-of-contact inhibit; ≥ 90 % of telecommands executed, no authentication failure; no watchdog reset or brownout; every Jetson latch-up cleared; Jetson energy per accepted beacon ≤ 0.1 Wh (BASELINE §6); OBC availability ≥ 99 % |
| End of life | No charge into the pack after passivation; re-entry within 5 years of the end of the mission (the FCC's five-year rule; CLAUDE.md, mission phase 3) |

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

Without the EPS controller answering (the stand-in power model of the development simulation, SW-01) the executive
keeps its previous behaviour: detumble completes in the first pass and the payload follows the power mode. The C
suites run that way, unchanged except where SIL-06 changed a specified value.

## 7. Results

The reference run: `--scenario full`, seed 20280601, deployment 2028-06-01 06:00 UTC, 500 km sun-synchronous at
LTAN 22:30, 730 days of operations. It found **0 anomalies and 0 warnings in 37 checks** (about 15 minutes on one
core). The main values:

| Area | Result |
|------|--------|
| LEOP | First burn 30.0 min and first RF 45.0 min after separation; antennas and wings sensed deployed at 30.1 min; detumbled (below 0.5°/s) after 3.7 h, from a 10°/s tip-off |
| Attitude | Spin about +Y at about 2°/s; +Y within about 2–10° of the Sun in sunlight, drifting to about 40° by the end of eclipse |
| Power | Depth of discharge at most 2.9 % per eclipse (after LEOP, about 2 %); lowest SOC in operations 98.3 %; 24,970 Wh drawn from the array and 22,182 Wh used over the mission, with the regulator curtailing the rest at full charge; capacity fade 1.4 %; gauge error at most 0.7 % |
| Thermal | Battery +1.4 to +20.4 °C, never outside the charge window; OBC −10.6 to +26.3 °C; PA at most 25.8 °C; Jetson at most 26.4 °C |
| Downlink | 1.40 s longest key-down, 2.2 % worst 60 s duty; longest beacon gap 166 s; 98.8 % of 29,844 passes with a beacon decoded; no frame while not permitted |
| Transponder | Enabled about 900 min and keyed about 57 min a day; at most 14.1 % of an orbit keyed; 97.9 % of the uplink demand repeated while its gates held; never radiating while not permitted; 95 s in all (0.00 % of its keyed time) within one pass of an eclipse entry |
| Jetson | 174,364 beacons with Jetson text; 0.036 Wh per accepted beacon (gate 0.1 Wh) |
| Operations | 1,440 of 1,453 telecommands executed (the rest lost on the link), no authentication failure, no loss-of-contact inhibit; 5 single-event OBC resets, each recovered through DETUMBLE; 1 Jetson latch-up, the rail cut by the guard after 1.0 s; no watchdog reset or brownout; OBC availability 100.000 % |
| End of life | TX-OFF and passivation at day 730; no charge into the pack afterwards; re-entry 3.03 years after the design life (5.03 years after deployment) |

The battery sits near full charge for most of the mission. The model's calendar fade is small (1.4 %), but holding
Li-ion cells at 4.20 V for two years is known to age them faster than a lower float voltage. Lowering the
charge-voltage limit would buy life at the cost of margin the run shows is there; that is a design choice for the
EPS, not an anomaly.

## 8. Limits and open items

- The models are not correlated with hardware. The battery mount (SIL-03), the transponder's repeat-path hardware
  behind `XPDR_EN` (SIL-08) and the gyro (SIL-05) are design assumptions to confirm (TB-07; TRANSMITTER_DESIGN §10.1).
- The ADCS law's gains and the 2°/s spin are set by simulation only; the attitude model has no flexible modes,
  magnetometer noise or sensor misalignment, and the gyro has a fixed 0.02°/s bias.
- The thermal network is lumped (12 nodes) and its internal conductances are assumptions.
- The Jetson's inference and the onboard AI selector are represented by their power and their answers on the line,
  not by models.
- After a reset the spacecraft detumbles again (the spin is above the 0.5°/s criterion), so each single-event reset
  costs up to a few hours of IDLE. Skipping the detumble when the rate is a spin about +Y would remove that; it is
  not in the baseline.
