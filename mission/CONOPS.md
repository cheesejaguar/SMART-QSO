# Concept of Operations (ConOps)

This ConOps follows the lightweight style used by NASA Ames smallsat missions (e.g., TechEdSat, GeneSat, O/OREOS, PharmaSat), tailored to a 1U CubeSat that carries a duty‑gated U/V FM transponder and a 1200 bps telemetry beacon, commanded from designated amateur stations. `docs/BASELINE_PARAMETERS.md` governs every value here; where this document disagrees with it, the baseline is right. All transmissions are in the clear per the Amateur‑Satellite Service.

## 1. Mission Overview
- Spacecraft: 1U CubeSat in circular SSO, 475 km target, 485 km cap (BASELINE §3), launched on a SpaceX Transporter rideshare from Vandenberg in an Exolaunch EXOpod Nova (planning date about 2028-11)
- Primary objective: a single‑channel U/V FM transponder for worldwide amateur contacts, duty‑gated by available power (BASELINE §2; decision D4 keeps it)
- Secondary: AX.25 telemetry beacon, 1200 bps AFSK, plain text with a machine‑parsable telemetry tail (`docs/BEACON_SPEC.md`)
- Experiment: onboard AI that schedules the transponder within the power budget and selects beacon text (§5)
- RF: uplink 435–438 MHz (70 cm), downlink 145.8–146.0 MHz (2 m), one IARU‑coordinated frequency each; 0.5 W nominal, 1 W maximum (BASELINE §7, `docs/RF_PLAN.md`)
- Command: authenticated telecommand from designated stations, with a hardware TX‑OFF path (§4)
- Autonomy: low‑power OBC (STM32L4R5ZI; Rust flight software on the Embassy executor, no RTOS) runs the spacecraft. The Jetson Orin Nano is power‑gated and used only in short bursts. If the Jetson is off or faulty, the spacecraft is a conventional transponder with template beacons
- Compliance: FCC Part 97 amateur space station, licensee KK6GDB, station KK6GDB‑11 (§8)

## 2. Operational Phases
1) Launch & Early Orbit (LEOP) — Day 0 to ~Day 1 (minute by minute: `docs/operations/MISSION_TIMELINE.md`)
   - Before launch: final charge to 60 % ± 5 % SOC at the integrator about 47 days before launch; no access afterwards. Unpowered in the dispenser, all inhibits open (≤ 20 µA drain); separation SOC ≥ 40 %
   - Deployment from the dispenser; the OBC boots on switch release. No magnetorquer actuation for 120 s (SpaceX UG v11 §3.3.4). No RF for 45 minutes: the whole radio stays unpowered, receiver included (CDS 2.4.5); once complete, the silence is latched
   - Solar wings deploy no earlier than 30 minutes after switch release (CDS 2.4.4), then the antennas (ISIS AntS, I²C-commanded), with retries spread over sunlight and a ground-commanded retry
   - Detumble with B-dot magnetorquer control (from 10°/s within 3 orbits; about 1 h in the mission simulation); spin-stabilized coarse sun-pointing of +Y
   - First beacons in IDLE, template text, every 120 s (SAFE, every 180 s, if the separation SOC is below 25 %); receiver on from the end of the silence. The spacecraft holds IDLE, with the Jetson and the transponder off, until an authenticated command releases the commissioning hold
   - First pass over the Bay Area 0.7–12.2 h after separation (median 5.2 h); first commanding pass median 6.3 h, earlier with the second station. First operator ephemeris within 24 h of first contact (SpaceX UG v11 §9.5.7.2), from the onboard GNSS fixes
   - Partner and community stations confirm reception and a power-positive state

2) Commissioning — ~2 weeks (`docs/operations/COMMISSIONING_PLAN.md`)
   - Telecommand checkout from each designated station: authenticated `GET_STATUS`, a `TX_DISABLE`/`TX_ENABLE` cycle to prove TX‑OFF, beacon interval and power level commands, a housekeeping dump
   - TX-OFF proven from each station before the transponder is opened to users; operator ephemeris after every pass and at least every 24 h until cataloged; object claimed with the 18th SDS within 30 days
   - Characterize power and thermal; calibrate solar and battery telemetry
   - Beacon conformance: deviation, bandwidth, cadence, call sign
   - Commissioning hold released by an authenticated command once telecommand, TX‑OFF, power and thermal are proven. Transponder enable, then: first windows during passes of the telecommand stations, observed from the ground, then autonomous windows within the gates of §5
   - Jetson power‑on and burst tests within the OBC limits; fallback test (Jetson off, templates continue)
   - At the end: `SET_CONTACT_TIMEOUT` from 30 to 14 days (decision D3)

3) Nominal Operations — Months 1–24 (minimum 12 months)
   - Transponder windows when sunlit and SOC ≥ 55 %, duty ≤ 25 %, CTCSS‑gated access
   - QSO handling: users work each other through the transponder with FM handhelds or mobile rigs; the scheduler places windows by predicted regional demand and fair access, within the power gates
   - Beacon every 60 s (ACTIVE), 120 s (IDLE), 180 s (SAFE), time‑shared with the transponder downlink
   - Regular authenticated contacts from the telecommand stations, at least once per loss‑of‑contact period
   - Public engagement: decoded beacons and status published openly

4) Extended Operations (as margins allow)
   - Continue nominal operations; transponder duty reduced as the battery ages

5) Deorbit & Passivation — End of Life
   - Final identification beacon
   - Permanent passivation by a two-step authenticated command (`PASSIVATE_ARM` with the operational key, then `PASSIVATE_EXECUTE` with the recovery key within 600 s; SQSO-ICD-013 §6.4): TX-OFF, then the battery charge path disconnected (non‑resettable), battery left discharging (47 CFR 97.207(g)(1)(iii))
   - Passive re‑entry with the solar wings deployed, no later than 5 years after mission end. There is no separate drag device

## 3. Modes of Operation
- Launch‑Safe: inhibits armed; radio unpowered; awaiting deployer release
- Commissioning hold: from separation until released by an authenticated command; IDLE at most, no transponder, no Jetson
- Boot/Detumble: attitude control to reduce rates; minimal housekeeping; beacon only after the RF silence
- SAFE: lowest power; template beacon every 180 s; no transponder, no Jetson
- IDLE: beacon every 120 s; Jetson suspended; transponder only if the power gates allow
- ACTIVE: beacon every 60 s; transponder windows and Jetson bursts permitted within the gates of §5
- EMERGENCY: below 15 % SOC; beacon shed; receiver stays on

Mode transitions follow the SOC thresholds (ACTIVE at ≥ 55 % sunlit; IDLE below 40 %; SAFE below 25 %; EMERGENCY below 15 %), sun/eclipse state, thermal limits and the watchdog. Faults drop to SAFE. TX‑OFF and the loss‑of‑contact inhibit override every mode.

## 4. Communications Concept
- Downlink: 145.8–146.0 MHz (2 m). FM transponder output (16K0F3E) and AX.25 UI beacon frames, 1200 bps AFSK (10K4F2D)
- Uplink: 435–438 MHz (70 cm). Transponder input (CTCSS‑gated) and telecommand (1200 bps AFSK)
- Telecommand: HMAC‑SHA‑256 with a persisted monotonic counter per key, from designated stations only (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`). Commands are sent in the clear and authenticated, not encrypted (97.211(b)). Keys are loaded only through the umbilical
- TX‑OFF: `TX_DISABLE` removes PA power through a hardware switch and is latched in non‑volatile memory. It is accepted without authentication if no key is usable. The receiver stays on (97.207(b))
- Loss of contact: transmit inhibit after N days with no authenticated command, receiver on; N = 30 days at launch, 14 days after commissioning (D3). The next authenticated contact lifts it
- Identification: call sign in the AX.25 source address and again at the start of the beacon text (97.119)
- Antennas: separate deployable quarter‑wave monopoles for 2 m (≈ 0.5 m) and 70 cm (≈ 0.17 m), with a receive filter

## 5. Autonomy, Power, and Thermal Management
- OBC always on. It enforces every hard limit itself; no AI output can override them
- Beacon text: an OBC selector picks among pre‑approved templates (flight baseline, D5). Jetson‑generated text is a gated experiment, enabled only if the measured energy per accepted beacon is ≤ 0.1 Wh. Both sides apply the same content filter, and generated text carries no numbers; the OBC writes all values from measurements (`docs/BEACON_SPEC.md` §6)
- Transponder: the selector proposes the window schedule and QSO priority; the OBC applies the power gates (sunlit, SOC ≥ 55 %, duty ≤ 25 %)
- Jetson bursts: ≤ 120 s including boot, sunlit, SOC ≥ 55 %; the OBC cuts the rail on its own current and temperature measurements
- Fail‑graceful: if the Jetson is off, faulty or its text fails the filter, the beacon uses a template and the transponder runs on fixed rules
- As built (2026‑10‑25): the OBC sends its fixed templates in turn; the selector and the transponder scheduler are not implemented, and the flight software has not run on hardware

## 6. Ground Segment & Operations
- Telecommand stations: at least two designated amateur stations at least 1,000 km apart, each able to command alone (Santa Clara and NASA Ames, 13 km apart, count as one site), with cross‑Yagis, rotators, 50 W PEP on 70 cm and Doppler pre-correction; agreements by 2027-01-05, not yet named (D1). Only they hold keys
- Users: any licensed amateur with an FM handheld or mobile rig and a small Yagi
- Community and SatNOGS stations receive and decode beacons; logs published openly
- Operations: verify beacon content, cadence and power trends; command mode, beacon interval, power level and payload as needed; renew contact within the loss‑of‑contact period (`docs/operations/MISSION_OPERATIONS_PLAN.md`)

## 7. Contingencies
- Loss of attitude control: SAFE mode, detumble; transponder off
- Power shortfall: modes step down by SOC; transponder and Jetson off; recovery with hysteresis
- Thermal over‑limit: Jetson off; transponder duty reduced; resume within limits
- RF anomaly or interference: a telecommand station sends TX‑OFF; transmissions resume only on authenticated `TX_ENABLE`
- No telecommand for N days: the transmitter inhibits itself; the receiver stays on for the next contact
- Jetson fault or latch‑up: the OBC powers it off; template beacons continue

## 8. Compliance
- FCC Part 97 amateur space station; no compensation for operations; telecommand from designated stations (97.211)
- Telecommand authentication only; all other content in the clear, formats published
- Cessation of emissions on command at any time (97.207(b), ITU RR 22.1)
- IARU frequency coordination and FCC pre‑space notification before launch (`docs/REGULATORY.md`)
- No RF for 45 minutes after deployment (CDS 2.4.5)
- End‑of‑life passivation and re‑entry within 5 years of mission end (47 CFR 97.207(g))

---

*Revision note (2026‑10‑27): the design review against the CDR criteria (`docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`) changed the launch chain (EXOpod Nova on a Transporter, planning date about 2028-11), the altitude (475 km target, 485 km cap), the LEOP sequence (120 s actuator hold, latched RF silence, wings before antennas, commissioning hold, GNSS ephemeris), the telecommand stations (two sites ≥ 1,000 km apart) and passivation (two-step command).*

*Revision note (2026‑10‑25): rewritten for the baseline mission. The previous text described a transmit‑only 2 m beacon with no uplink and no telecommand, and a separate drag device. It now follows `docs/BASELINE_PARAMETERS.md` §2, §5, §7 and §9 (D3, D4, D5): the 70 cm uplink, the duty‑gated transponder, authenticated telecommand with TX‑OFF, the loss‑of‑contact inhibit, the OBC beacon selector, the Embassy flight software and passivation without a drag device.*
