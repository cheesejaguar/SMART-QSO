# Concept of Operations (ConOps)

This ConOps follows the lightweight style used by NASA Ames smallsat missions (e.g., TechEdSat, GeneSat, O/OREOS, PharmaSat), tailored to a 1U CubeSat with a transmit‑only 2 m beacon. The payload demonstrates power‑aware autonomy and human‑readable beacons generated onboard. All transmissions are in the clear per Amateur‑Satellite Service.

## 1. Mission Overview
- Spacecraft: 1U CubeSat in ~500 km SSO
- Primary objective: Operate a smart, power‑aware beacon that reports status and context in human‑readable form, with a deterministic machine‑parsable tail (see `docs/BEACON_SPEC.md`).
- RF: Transmit‑only Stensat beacon on 2 m, AX.25 UI at 1200 bps AFSK (no uplink)
- Autonomy: Low‑power OBC (FreeRTOS) + declocked Jetson Orin Nano Super for burst text generation when power/thermal allow; fail‑graceful templates otherwise
- Compliance: Amateur‑Satellite Service; callsign and plain‑text beacon each frame; no encryption

## 2. Operational Phases
1) Launch & Early Orbit (LEOP) — Day 0 to ~Day 1
   - Detumble using magnetorquers; establish coarse sun‑pointing
   - Enable basic beacon at low cadence (e.g., 120 s) with template text only
   - Verify reception with partner stations; confirm power‑positive state

2) Commissioning — ~Week 1–2
   - Characterize power/thermal; calibrate solar and battery telemetry
   - Exercise DVFS/declocking profiles on Jetson; validate thermal margins
   - Beacon conformance check: deviation, bandwidth, cadence, callsign
   - Set initial adaptive cadence schedule (60–120 s) and SOC thresholds

3) Nominal Operations — Months 1–12
   - Smart Beacon Mode (adaptive):
     - Sunlit + SOC>threshold: allow Jetson bursts to generate personified text; otherwise use templates
     - Eclipse or low‑SOC: reduce cadence and suspend Jetson; maintain minimal health beacons
   - Health & Performance Monitoring: watchdogs; duty‑cycle and thermal gates
   - Public Engagement: publish decoded beacons and status timeline

4) Extended Operations (as margins allow)
   - Continue nominal operations at reduced cadence as battery ages

5) Deorbit & Passivation — End of Life
   - Cease nonessential transmissions; final “goodbye” beacon
   - Safe the battery per provider ICD; passive deorbit by drag device

## 3. Modes of Operation
- Launch‑Safe: Inhibits armed; beacon off; awaiting deployer release
- Detumble: Attitude control to reduce rates; minimal housekeeping
- Commissioning: Functional/telemetry checks; basic beacon at low cadence
- Nominal‑Idle: Beacon active; Jetson suspended; cadence adaptive by power forecast
- Nominal‑Active: Beacon active; Jetson permitted for short bursts under DVFS limits
- Safe‑Mode: Lowest power; template beacon only; extended cadence (e.g., ≥180 s)

Mode transitions are governed by SOC thresholds, sun/eclipse state, thermal sensors, and watchdogs. Any fault triggers Safe‑Mode until cleared by autonomous checks.

## 4. Communications Concept
- Downlink: 145.8–146.0 MHz (2 m), AX.25 UI frames, 1200 bps AFSK; Stensat beacon module
- Identification: Callsign in AX.25 addresses and again in the text body
- Beacon cadence: adaptive 60–120 s (longer in Safe‑Mode)
- Antenna: deployable quarter‑wave tape monopole with chassis counterpoise (near‑omni)
- Uplink: none (receive‑only ground segment)

## 5. Autonomy, Power, and Thermal Management
- OBC always‑on; Jetson power‑gated and declocked; DVFS profiles selected by power/thermal state
- Energy forecast influences cadence and Jetson enablement
- Thermal limits enforce Jetson suspension and cadence increases when hot
- Fail‑graceful: if Jetson unavailable or text exceeds limits, beacon falls back to concise template

## 6. Ground Segment & Operations
- Partner and community ground stations receive and decode beacons; logs published openly
- Operations verify beacon content/format, cadence, and power trends; anomalies flagged
- No telecommand; corrective actions are autonomous only

## 7. Contingencies
- Loss of attitude control: remain in Safe‑Mode; minimal cadence
- Power shortfall: duty‑cycle increases; Jetson disabled; recovery upon SOC/hysteresis
- Thermal over‑limit: Jetson off; cadence increased; resume when within limits
- RF anomaly: verify deviation/occupancy; if out‑of‑family, reduce cadence or suspend beacon until safe

## 8. End‑of‑Life
- Finalize beacon log publication; send concluding identification beacon
- Execute passivation steps per launch provider ICD; remain silent thereafter

