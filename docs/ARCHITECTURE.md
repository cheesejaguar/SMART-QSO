# SMART-QSO System Architecture

**Version**: 2.0
**Date**: 2026-10-27
**Status**: Design-phase overview (Phase B, pre-PDR). Nothing has been built or measured; every value below is
designed or predicted. `docs/BASELINE_PARAMETERS.md` (SQSO-BL-001 Rev E, amended) governs every value; where this page
disagrees with it, the baseline is right. Current review record: `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`
(an engineering assessment by AI reviewers, not an independent board).

Version 1.x of this page (2026-01) described an architecture that no longer exists: a "Jetson Orin Nano Super", a
GomSpace/ISIS module EPS on 3.3/5/12 V rails, a secondary OBC, CAN bus, microSD and NAND storage, a 145.825 MHz beacon,
an onboard QSO-priority CNN and over-the-air model updates, NRCSD as the deployer and a $142.5k budget. Those are
withdrawn; the previous text is in the Git history of this file.

---

## 1. Mission Architecture

| Element | Architecture | Source |
|---------|--------------|--------|
| Primary mission | Single-channel U/V FM transponder for worldwide amateur contacts, duty-gated by available power | BASELINE §2 (D4) |
| Secondary | AX.25 telemetry beacon, 1200 bps AFSK, plain text and published formats | BASELINE §2; `docs/BEACON_SPEC.md` |
| Experiment | Onboard AI that schedules the transponder within the power budget and selects plain-language beacon text. Flight baseline: a calibrated selector on the OBC choosing among pre-approved templates; gated experiment: generative text on the power-gated Jetson | BASELINE §2, §6 (D5) |
| Fail-graceful | AI off or faulted: conventional transponder on fixed rules and template beacons. Hard limits are enforced by the OBC and by hardware, never by a model | BASELINE §2, §6 |
| Community model updates | Not in the flight baseline; no onboard implementation | BASELINE §2 |
| Life | 2-year design life, 12 months minimum; passive re-entry with the wings deployed within 5 years of mission end | BASELINE §2, §3 |

## 2. Launch and Orbit

Falcon 9 Transporter-class rideshare from Vandenberg, in a 1U slot of an Exolaunch EXOpod Nova; planning date about
2028-11-01; circular SSO, 475 km target, 485 km cap, LTAN 22:30. Backup: ISS deployment through the Voyager NRCSD.
Launch requirements on the spacecraft: `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001); launch chain and
decision points: `docs/ORBITAL_ACCESS.md` (BASELINE §3).

## 3. System Block Diagram

```
             16 triple-junction cells (4 body faces + 2 deployable wings)
                                 |
                    +------------v-------------+
                    | EPS board (built)        |  4 MPPT trackers, 4S1P 18650 pack (~50 Wh, 4.10 V/cell),
                    | MSP430 EPS controller    |  load switches, inhibits DS1-DS3, burn drivers,
                    +--+------+------+------+--+  EOL charge disconnect, hardware load shed (Jetson rail)
                       |      |      |      |
        I2C telemetry  |      |      |      +---------------------------+
                       v      |      v                                  v
   +-------------------------+|  +-----------------------------+   +-----------------------------+
   | OBC: STM32L4R5ZI        ||  | RF board                    |   | Jetson Orin Nano 8 GB        |
   | Rust on Embassy, no RTOS|+->| 435 MHz receiver (CC1200)   |   | on a custom carrier, 7 W mode|
   | FRAM (mirrored), RTC,   |   | KISS TNC (STM32L431)        |   | power-gated, burst-only      |
   | watchdog supervisor     |-->| 2 m exciter + PA (<= 1 W)   |   | UART line protocol to OBC    |
   | template selector (D5)  |   | CTCSS repeat path, TX-OFF   |   +-----------------------------+
   +---+------------------+--+   +--------------+--------------+
       |                  |                     |
       v                  v                     v
   ADCS sensors and     GNSS receiver        ISIS AntS: 2 m elements along +-X,
   CR0002 rods          (duty-cycled)        70 cm elements along +-Y
```

Board-to-board pins: `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001). Build-versus-buy decisions:
`docs/hardware/BUILD_VS_BUY.md`.

## 4. Hardware Architecture

### 4.1 Structure and mass properties

- 1U, 100 × 100 × 113.5 mm (CDS Rev 14.1); mass CBE 1.52 kg, MEV 1.755 kg against the 2.00 kg design limit
  (+14.0 %); EXOpod Nova 1U limit 2.5 kg (BASELINE §4; `docs/environment/MASS_BUDGET.md`).
- Layout B: battery at the −Z end of the stack, RF at +Z; stowed CG within 5 mm of the geometric centre; deployed
  inertia Iyy ≥ 1.03 × max(Ixx, Izz) with 2 × 10 g ballast on the +Z end frame (decision C-13)
  (`hardware/mechanical/MECHANICAL_DESIGN.md` §5; `docs/environment/STRUCTURAL_ANALYSIS.md`).
- Deployables: two solar wings and the ISIS AntS. Wings first, at least 30 minutes after separation, two burn resistors
  per wing on independent current-limited drivers, 10 s per resistor (C-17); then the AntS, gated on its own
  temperature (C-18) (BASELINE §4; `docs/environment/MECHANISMS_ANALYSIS.md`).
- Inhibits: three deployment switches on three rails (DS1–DS3), giving ≥ 3 independent physical inhibits for RF and for
  each deployable (CDS 2.3.7, 2.3.8) (BASELINE §4).

### 4.2 Electrical power

- Array: 16 triple-junction cells, 2 on each of the 4 long body faces and 2 per side on each wing; EOL orbit-average
  generation 2.48 W tumbling, 3.84 W sun-pointed (mission simulation) (BASELINE §5;
  `hardware/eps/SOLAR_ARRAY_DESIGN.md`).
- Battery: 4S1P Li-ion 18650, ≈ 3.5 Ah, 14.4 V nominal, ≈ 50 Wh, charged to 4.10 V/cell; charging only between 0 and
  +45 °C; battery heater ≈ 207 Ω, on below +5 °C (BASELINE §5, §8; `hardware/eps/BATTERY_DESIGN.md`).
- The EPS is built, not bought: four MPPT trackers, the unregulated battery bus and regulated rails through load
  switches, the inhibits, the hardware TX-OFF path, the end-of-life charge disconnect and a hardware load shed of the
  Jetson rail below 13.6 V; its controller is a Class C firmware item (`docs/hardware/EPS_DESIGN.md`;
  `hardware/electrical/eps/EPS_BOARD_DESIGN.md`; `docs/software/EPS_CONTROLLER_FIRMWARE_REQUIREMENTS.md`).
- Power closure: margin ≥ 20 % in every mode at end of life (BASELINE §5; `docs/environment/POWER_BUDGET_ANALYSIS.md`).

### 4.3 Onboard computer

- STM32L4R5ZI (Cortex-M4F, 640 KB RAM, 2 MB flash); flight software in Rust on the Embassy executor, no RTOS: one task
  running the 20 ms main loop (BASELINE §6; `software/flight/docs/RTOS_ARCHITECTURE.md`).
- Non-volatile state in FRAM (FM25V20A with a mirror) for configuration, counters and the housekeeping store
  (`hardware/obc/STORAGE_DESIGN.md` §3); mission clock RV-3028-C7; independent watchdog backed by a TPS3850 supervisor
  (`hardware/obc/OBC_DESIGN.md`).
- No secondary OBC; the Apollo4 and STM32L476 options of earlier documents are superseded (BASELINE §6).

### 4.4 RF and antennas

- 435 MHz receiver, always on after the 45-minute RF silence, carrying the transponder uplink and authenticated
  telecommand; one 2 m FM transmitter time-shared by the AX.25 beacon (through a KISS TNC) and the CTCSS-gated repeat
  path; 0.5 W nominal, 1 W maximum (`docs/hardware/RF_DESIGN.md`; `hardware/electrical/rf/RF_BOARD_DESIGN.md`).
- Downlink one IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz; uplink requested in
  435.222–435.778 MHz, else 436.222–436.500 MHz (BASELINE §7; `docs/RF_PLAN.md`).
- TX-OFF removes PA power through a hardware switch, latched in NVM, independent of the AI payload (BASELINE §7).
- Antennas: ISIS AntS 1U–3U, separate quarter-wave tape elements for 2 m and 70 cm, no diplexer
  (`hardware/rf/ANTENNA_DESIGN.md`).

### 4.5 Attitude determination and control

Three-axis magnetorquers (CubeSpace CR0002 rods), magnetometer, coarse sun sensors from the panel photodiodes, MEMS
gyro (Epson M-G370PDT0, BMI088 backup). B-dot detumble from 10°/s within 3 orbits, then spin-stabilized coarse
sun-pointing of +Y at 8°/s; no onboard three-axis estimator (C-22). Pointing requirement restated for power (C-23):
≤ 5 % of sunlit generation lost and the Sun within 30° of +Y for ≥ 95 % of sunlit time (BASELINE §8;
`docs/hardware/ADCS_DESIGN.md`).

### 4.6 AI payload

NVIDIA Jetson Orin Nano 8 GB module (P/N 900-13767-0030-000) on a custom carrier, 7 W mode, ≤ 8 W input, no fan.
Each power-on ≤ 10 minutes including boot, then ≥ 100 minutes off; sunlit, SOC ≥ 55 %; OBC-enforced per-rail current
and temperature limits independent of the Jetson. Treated as non-critical and expendable (BASELINE §6;
`docs/hardware/PAYLOAD_DESIGN.md`; `software/payload/README.md`).

### 4.7 Navigation

Duty-cycled GNSS receiver (SkyFox piNAV-NG/FM) on an OBC load switch, for the operator ephemeris that the SpaceX
rideshare rules require within 24 h of first contact (BASELINE §6, D6).

## 5. Software Architecture

- **Flight software** (`software/rust`): `sqso-protocol` (wire formats) and `sqso-fsw` (flight logic) are `no_std`
  with `unsafe` forbidden; `sqso-ffi` implements the C headers of `software/flight/include`, which the CMocka suites
  specify; `firmware` is the STM32L4R5 image; `sqso-sim` the host simulation; `sqso-sil` the mission simulation from
  integration to re-entry (`software/rust/README.md`; `docs/test/SIL_MISSION_SIMULATION.md`). Class C under NPR 7150.2D
  (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1). Design: `docs/software/SOFTWARE_DESIGN.md`.
- **Modes**: BOOT, DETUMBLE, SAFE, IDLE, ACTIVE, EMERGENCY, with SOC thresholds ACTIVE at ≥ 55 % sunlit, IDLE below
  40 %, SAFE below 25 %, EMERGENCY below 15 % (BASELINE §5). TX-OFF and the loss-of-contact inhibit override every mode.
- **Telecommand**: HMAC-SHA-256 with a persisted monotonic counter per key, from designated stations only; TX-OFF
  accepted without authentication when no key is usable (BASELINE §7; `docs/interfaces/TELECOMMAND_AUTHENTICATION.md`).
- **AI**: the OBC selector (gradient-boosted trees or a tiny INT8 MLP, abstaining to fixed rules) proposes the power
  mode and transponder schedule and picks templates; it sets no QSO priority, because the FM repeater has no
  per-station control. Designed, not yet implemented (`docs/payload/OBC_SELECTOR_DESIGN.md`). The Jetson agent
  (llama.cpp; SmolLM2-360M-Instruct or Qwen3-0.6B) returns text only; the OBC adds the call sign and every value
  and applies the shared content filter (`docs/BEACON_SPEC.md` §6).
- **Payload and ground software**: Python, using the flight wire formats through `sqso-py` (`software/payload`,
  `software/ground`).

## 6. Interfaces

| Interface | Document |
|-----------|----------|
| OBC–EPS | `docs/interfaces/ICD_OBC_EPS.md` |
| OBC–RF | `docs/interfaces/ICD_OBC_RF.md` |
| OBC–ADCS | `docs/interfaces/ICD_OBC_ADCS.md` |
| OBC–Jetson | `docs/interfaces/ICD_OBC_JETSON.md` |
| Space–ground and beacon format | `docs/interfaces/ICD_SPACE_GROUND.md`; `docs/interfaces/ICD_BEACON_FORMAT.md` |
| Telecommand authentication | `docs/interfaces/TELECOMMAND_AUTHENTICATION.md` |
| Master list | `docs/interfaces/INTERFACE_CONTROL_MASTER.md` |

## 7. Ground Segment and Operations

At least two designated telecommand stations at least 1,000 km apart, each able to command alone, radiating ≥ 64 dBm
EIRP and commanding above 10° (C-14); none agreed yet. Users: any licensed amateur with an FM handheld or mobile rig.
Concept of operations: `mission/CONOPS.md`; minute-by-minute timeline: `docs/operations/MISSION_TIMELINE.md`; ground
stations: `docs/ground/GROUND_STATION_DESIGN.md`.

## 8. Reliability, Verification, Cost and Schedule

- Reliability estimates L1/L2/L3 0.75 / 0.61 / 0.51 (0.86 / 0.71 / 0.59 with the burn-in and flatsat programme), for
  the PI to accept or reject (DA-02; `mission/RISK_REGISTER.md` R29).
- Verification status: `docs/verification/VERIFICATION_MATRIX.md` (the only status record); test levels:
  `docs/environment/ENVIRONMENTAL_TEST_PLAN.md`.
- Cost: `docs/funding_audit/LIFECYCLE_COST_ROM.md`; schedule: `mission/SCHEDULE.md`; risks: `mission/RISK_REGISTER.md`.

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 1.0 | 2026-01 | Initial PDR-level architecture (superseded) |
| 1.1 | 2026-10-24 to 2026-10-27 | Notes: OBC software Rust on the Embassy executor (CR-2026-002); mass from BASELINE Rev E |
| 2.0 | 2026-10-27 | Documentation audit (post-CDR 2026): rewritten as a current overview from BASELINE Rev E (amended) and the design documents; the obsolete hardware, software, cost and schedule content of 1.x withdrawn |
