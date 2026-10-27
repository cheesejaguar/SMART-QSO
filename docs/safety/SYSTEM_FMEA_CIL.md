# SMART-QSO System (Hardware) FMEA and Critical Items List

**Document ID**: SQSO-FMEA-002
**Version**: D
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR draft, functional level, at BASELINE Rev E. Version C rested on BASELINE Rev D, `docs/hardware/BUILD_VS_BUY.md`
and the block schematics S1–S8; the board-level FMEA of the round-2 build-to designs is the companion SQSO-FMEA-003
(`docs/safety/BOARD_FMEA.md`, closure-panel finding P3P-04), whose critical items and single points this version
carries (§3, §4). Piece-part FMEA follows the engineering model.

## 1. Purpose and Method

This FMEA extends the software FMEA (SQSO-FMEA-001, `docs/safety/SOFTWARE_FMEA.md`) to the hardware of the whole
spacecraft. It feeds the system hazard analysis (SQSO-SDP-002), the reliability block diagram and success
probabilities (`docs/environment/RELIABILITY_ANALYSIS.md` v2.0) and the risk register.

- **Method:** functional and hardware FMEA in the form of MIL-STD-1629A tasks 101 (FMEA) and 102 (criticality, by
  category, without a numerical criticality number), tailored for Class D under NPR 8705.4B. Each item lists the
  failure mode, cause, effect at system and mission level, detection, compensating provisions and a criticality.
- **Criticality categories:**

| Category | Meaning |
|----------|---------|
| **1** | Can cause a catastrophic or critical hazard (SQSO-SDP-002) |
| **2** | Single failure that ends the mission (no further success level can be met) |
| **2R** | As 2, but redundant or with a working alternate path |
| **3** | Loses a success criterion or degrades service; the mission continues |
| **4** | Negligible |

- **Critical items:** every Category 1 item and every Category 2 single point (§3).
- Success levels L1, L2, L3 and their criteria are those of `docs/mission/MISSION_SUCCESS_CRITERIA.md` (v2.1).
- **Board level:** SQSO-FMEA-003 analyses the EPS (Rev D), battery sense board, OBC, magnetometer board A2, access-port
  board A3, RF board, Jetson carrier, interconnect and harness: 176 failure modes, 59 whole-mission single points (36 not
  seen at this level), findings SMA-B01 to SMA-B19. It uses this document's categories, with 1R for a Category 1 mode that
  keeps two further controls. Where a row below and a board-level row disagree, the board-level row is the later analysis.

## 2. FMEA

| ID | Item / function | Failure mode | Cause | System effect | Mission effect | Detection | Compensating provisions | Cat |
|----|-----------------|--------------|-------|---------------|----------------|-----------|------------------------|-----|
| HF-01 | Deployment switches DS1/DS2 | Fail open after separation | Switch damage, contamination, misalignment | Bus never powers | Loss of mission (DOA) | None (silent) | Two rail switches in series give no redundancy for this mode; pre-ship and post-vibe switch test (VI-LV-05) | 2 |
| HF-02 | DS1/DS2 (and DS3 proposed) | Fail closed / bounce in dispenser | Welded contact, vibration | Bus powered inside dispenser; RF or burn possible if other inhibits fail | Safety | Inhibit monitoring in vibration (VI-LV-04) | Three independent inhibits on each source: DS1, DS2, DS3 on the pack; DS1, DS3 and DS2's second pole (TRK_EN, EPS Rev D) on the array (SQSO-FMEA-003 BF-EPS-01 to -09); timers | 1 |
| HF-03 | Battery cell (×4 series) | Open circuit | Cell CID trip, weld failure | No battery; bus only in sunlight if the EPS can start from the array (not designed) | Loss of mission | VBAT telemetry | Cell screening; no 2P redundancy (BASELINE §5). The array-only path of the EPS (§5.2) does not survive a pack open: DS2's gate drive comes only from the pack (SQSO-FMEA-003 BF-EPS-10, SMA-B01) | 2 |
| HF-04 | Battery cell | Internal short / runaway | Latent defect, plating, over-discharge and recharge | Venting, fire | Safety (ground, ISS); loss of mission | Cell voltage, temperature | Screening (VI-SAF-03), temperature charge inhibit, latched recharge lockout K1 (designed, SQSO-EPS-BRD-001 D §5.4; BF-EPS-21 to -24) | 1 |
| HF-05 | Charger (MPPT, CV) | Regulation fails high | IC failure, feedback open | Overcharge | Safety | Cell voltage telemetry | BMS OV and independent comparator (2-FT) | 1 |
| HF-06 | Charger | No charge | IC failure, input open | Pack drains | Loss of mission within days | SOC trend | None (single string) | 2 |
| HF-07 | BMS (BQ76920 class) | Spurious trip / FET stuck open | IC latch-up, SEU in configuration | Bus loss until reset | Loss of mission if not recoverable | Bus resets | BMS power-on defaults safe and auto-recovering; SEL current limit | 2 |
| HF-08 | Pack fuse F2 | Opens | Over-current (e.g. the wing burn: SMA-05) | Bus loss | Loss of mission | None | Burn current 0.8 A per driver, 2.0 A fault cap (designed, EPS §9); fuse rated 1000 A interrupt (FM12); F0A, F0B, F0N on the sense board add series pack-open elements (BF-EPS-10) | 2 |
| HF-09 | EPS rail regulators / PDU load switches | Output short or latch-up | SEL, part failure | Rail lost or bus collapse | Loss of mission if OBC or radio rail | Rail currents | Latch-up current limiters; OBC-commanded power cycling | 2 |
| HF-10 | EOL passivation fuse F1 | Opens early | Spurious command, part failure | Battery can no longer charge | Loss of mission | Charge current | Two-key command sequence PASSIVATE_ARM (operational key) then PASSIVATE_EXECUTE (recovery key) within 600 s, refused in year 1 (TELECOMMAND_AUTHENTICATION v0.7 §6.4, designed, not implemented); two independent fire signals (SQSO-BSA-001 H9) | 2 |
| HF-11 | EOL passivation | Fails to open | Heater open, a fire FET open, arm latch or (Rev D) the EPS controller's EOL_KEY unavailable; command not received (receiver, keys) (BF-EPS-48, -65) | Battery stays charged and cycling | Debris requirement 4.4-2 met by option 2 (controlled energy, ODAR §4); no autonomous backup recommended | Zero charge current in sunlight after the command | Battery protections remain | 3 |
| HF-12 | Solar string (6–8 strings) | Open or short | Cell crack, interconnect, blocking diode | −1/8 to −1/6 of generation | Degraded | String telemetry | Many strings | 4 |
| HF-13 | Wing burn driver (one for both wings, S5) | Fails open | Driver, SW6, timer or software | Neither wing deploys | No transponder service (L2, L3 lost); disposal in wings-failed case (ODAR §6) | Deploy-sense switches | Retry; none in hardware (one driver) | 3 (single point for L2) |
| HF-14 | Wing hinge or restraint (each) | Does not release | Line not cut, hinge jam, cold | One wing stowed | L2 possible with sun-pointing; tumbling margin lost | Deploy sense, generation | ADCS sun-pointing | 3 |
| HF-15 | Wing restraint | Releases in dispenser | Line creep or cut in vibration | Appendage in dispenser | Safety (HR-03) | — | DFMR line design, VI-SAF-05 | 1 |
| HF-16 | Burn circuit | Stuck on | Driver short, software | ≈ 83 W draw; fuse opens or fire risk at the restraint | Loss of mission; safety | Bus current | 12 s one-shot per line and 0.8 A per driver (designed, EPS §9); a shorted driver FET burns while DEPLOY_EN is high (BF-EPS-58, SMA-B06) | 1 |
| HF-17 | AntS VHF element | Fails to deploy | Release fault | TX into a stowed tape, link loss ≫ margin | Loss of mission (downlink) | No ground reception | AntS redundant release, retry | 2R |
| HF-18 | AntS UHF element | Fails to deploy | Release fault | Uplink and transponder input degraded | L2 at risk; telecommand degraded | Command success rate | AntS redundant release | 2R |
| HF-19 | OBC (STM32L4R5) | Latch-up, hang, corrupt NVM | SEL, SEU, flash wear, software | Reset loop or silence | Loss of mission if unrecoverable | Beacon absence | Watchdog, latch-up current limit, FRAM state, safe boot | 2 |
| HF-20 | OBC XPDR_EN | Stuck high | GPIO or software | Transponder enabled outside schedule | Power drain; duty gate bypassed | XPDR_KEY readback | Heartbeat gate (monostable) drops the enable on a hang (BUILD_VS_BUY §3.2) | 4 |
| HF-21 | 435 MHz receiver | No output | Part failure, LNA burn-out from own TX | No telecommand, no transponder input | L2 lost; **after N = 30/14 days without an authenticated command the transmitter is inhibited** (D3): beacon lost too | Command acknowledgements, RX_FAULT, RX_CD | None (single string); REL-04 keeps the beacon when the receiver is known failed (SRS-F115 to F117), with evidence gaps (SQSO-FMEA-003 SMA-B11) | 2 |
| HF-22 | 2 m transmitter / PA | No output | PA failure, thermal stress at transponder duty | No beacon, no transponder | Loss of mission | Ground reception | Derating; PA temperature limit | 2 |
| HF-23 | 2 m transmitter | Continuous carrier | Exciter or PTT fault | Interference; battery drain | Regulatory (97.207(b)); power | Ground | TX-OFF removes PA supply in hardware (TX_EN); 120 s key-down timer | 3 |
| HF-24 | TX_EN hardware switch | Stuck open | Part failure | Cannot transmit | Loss of mission | Ground | None | 2 |
| HF-25 | TX_EN | Stuck closed | Part failure | TX-OFF ineffective | Regulatory (cessation of emissions, 97.207(b)) | Ground test of TX-OFF | Second series element would make it 1-FT (not designed) | 3 |
| HF-26 | Transponder repeat path (CTCSS decoder, monostable, AND, timer) | No repeat | Part failure | No transponder | L2/L3 lost; beacon continues | No relays heard | None | 3 (single point for L2) |
| HF-27 | Magnetometer | Fails | Part, SEL | Rate estimate lost for B-dot and spin law | Tumbling: generation 2.14 W (−11 %) | Telemetry | Gyro-based control; tumbling still powers housekeeping | 3 |
| HF-28 | Gyro M-G370PDT0 | Fails | Part, SEL | Switch to BMI088 (calibrated) | None if BMI088 works | Telemetry | BMI088 backup; then magnetometer rate (mean Sun angle 23°) | 4 |
| HF-29 | Magnetorquer driver | Stuck on | Driver short | Residual dipole, torque | Pointing degraded | Coil current | Driver power switch | 3 |
| HF-30 | Battery heater | Fails off | Element open, thermostat | Pack may fall below 0 °C (charge inhibited) | Degraded in cold cases | Pack temperature | 0.05 W/K mount keeps the pack above +1.6 °C in the SIL year run | 3 |
| HF-31 | Battery heater | Fails on | Switch short | Pack heats ≈ 33 K above frame at 1 W, 0.03 W/K | Accelerated ageing | Pack temperature | Hardware cut-out at +40 °C, fuse | 4 |
| HF-32 | Jetson rail | Stuck on (both series switches fail) | PDU switch and carrier enable both fail | Load > generation; pack flattened | Loss of mission | Rail current | Two independent switches on different lines, latch-up trip on the PDU switch (BUILD_VS_BUY §7.3); the hardware shed gates both since EPS Rev D (BF-EPS-41, -42) | 2R |
| HF-33 | Jetson module | Short or latch-up | SEL, part | Rail over-current | None if tripped | Rail current | OBC per-rail trip independent of the Jetson | 4 |
| HF-34 | Jetson carrier UART | Back-powers OBC I/O | Unpowered carrier on a driven line | OBC pin damage | Degraded | — | Ioff translators (S7) | 4 |
| HF-35 | GNSS receiver | Fails or shorts | Part, SEL | No ephemeris from GNSS | Third-party ephemeris (contingency, LVR-089) | Telemetry | Load switch with latch-up trip | 4 |
| HF-36 | Harness / connector | Open or intermittent | Workmanship, vibration | Any of the above | Up to loss of mission | Varies | Workmanship standards (MAP §8), inspection, vibration and TVAC with functional tests | 2 |
| HF-37 | Structure, fasteners | Loosening, part release | Preload loss (Ti screws in polymer bushings), staking crack | Loose part in dispenser | Safety (HR-05) | Post-vibe inspection | Locking features, staking, VI-SAF-06 | 1 |
| HF-38 | Telecommand keys / counters | Lock-out | Counter desynchronised, key corrupted | No command accepted | After N days: transmit inhibit (D3) | Rejected-command counter | Spare keys and offline recovery key (SQSO-ICD-013) | 2R |

## 3. Critical Items List (CIL)

Items whose failure modes are Category 1 or Category 2 single points. Class D payloads (NPR 8705.4B risk
classification) tolerate a medium to significant risk of not achieving mission success and permit single-string
design, so a Category 2 single point is retained when its rationale holds; Category 1 items are never accepted on
class grounds and must meet the safety package's fault tolerance.

| CIL | Item (FMEA) | Cat | Retention rationale | Controls and verification |
|-----|-------------|-----|---------------------|---------------------------|
| CIL-01 | Inhibits DS1/DS2/DS3 (HF-02); board level: BF-EPS-01, -03, -05, -07, -09, -16, BF-A3-07 | 1 | Required by CDS | DS3 designed (SMA-01); DS2 second pole on the array (EPS Rev D); domain crossings ≥ 100 kΩ and the isolator (V-EPS-4); A3 group separation (V-OBC-18); VI-LV-04, VI-LV-06, V-EPS-3 with two of three failed closed |
| CIL-02 | Battery cells and protections (HF-04, HF-05); board level: BF-EPS-12, -14, -15, -17, -23 to -26, -28, -39, BF-HAR-02, -03 | 1 | Required energy store | VI-SAF-01–03; lockout K1 designed (V-EPS-9); F0A, F0B, F0N, F2 at 1000 A interrupt; W1 short 1-FT by leg plus DS2 at launch (SQSO-FMEA-003 §4 (b)) |
| CIL-03 | Burn circuit (HF-16) and restraint (HF-15) | 1 | Required to deploy | Current and on-time limit designed (0.8 A, 12 s one-shot, U17 2.0 A cap; V-EPS-6); VI-SAF-04, -05 |
| CIL-04 | Structure and fasteners (HF-37); board mounts BF-A2-06, BF-A3-08, BF-JET-12 | 1 | Bought frame; locking features | VI-SAF-06, VI-LV-04; prevailing-torque threads and staking on A2, the A3 jackposts, the M.2 card; V-JET-8 |
| CIL-05 | Deployment switches fail open (HF-01) | 2 | CDS requires series switches; redundancy against "fail open" would defeat the inhibit | Switch selection from flown parts; test before and after vibration |
| CIL-06 | Battery string open (HF-03) | 2 | 4S1P kept by D8 (mass); 2P would add ≈ 200 g | Cell screening; matched cells |
| CIL-07 | Charger, BMS, fuse, PDU (HF-06–HF-09); board level BF-EPS-13, -35, -37, -44, -46 | 2 | Custom EPS (BUILD_VS_BUY §2.4); single string normal for 1U | EM and flatsat burn-in ≥ 500 h; latch-up limiters; protoflight TVAC with functional tests |
| CIL-08 | OBC (HF-19) | 2 | Single OBC normal for 1U | Watchdog, SEL limiter, flatsat hours; radiation data for the STM32L4R5 (MAP §7) |
| CIL-09 | 435 MHz receiver (HF-21; BF-RF-04) | 2 | Single receiver | REL-04 implemented (SRS-F115 to F117: the beacon continues when the receiver is known failed, to be declared to IARU and the D3 consultation); its evidence gaps are CIL-24 |
| CIL-10 | 2 m transmitter / PA and TX_EN (HF-22, HF-24) | 2 | Single downlink | PA derating; thermal limit; flatsat burn-in at transponder duty |
| CIL-11 | Harness (HF-36); BF-HAR-06, -12, -19 | 2 | — | NASA-STD-8739.4A workmanship; inspection; recommended: double the single conductors whose open ends the mission (W3 RADIO_EN, RF_TX_EN, DEPLOY_EN on spare GND contacts; W4B RADIO_3V3 on a second contact) |
| CIL-12 | Wing burn supply shared by the four drivers (HF-13; BF-EPS-55) | 3 (L2 single point; disposal case) | Four drivers on four lines since EPS B; SW6, U17, V_BURN and DEPLOY_EN stay common (0.2 % of the 0.29 % wings-failed) | V-EPS-6; VI-LV-17; DEPLOY_EN also arms the AntS (CIL-19) |
| CIL-13 | Repeat path (HF-26; BF-OBC-15, BF-RF-23, BF-HAR-10) | 3 (L2 single point) | Transponder is the primary objective but not needed for the beacon | Flatsat end-to-end repeat test; burn-in |
| CIL-14 | Pack path series elements and the drives sourced from BAT_F (BF-EPS-10, -11) | 2 | 4S1P kept by D8; the fuses are required by H3 | FM12 lot acceptance (X-ray, resistance); **SMA-B01** (diode-OR into DS2's drive: sunlit operation after a pack open); V-BFM-01 |
| CIL-15 | Cell taps shared by the AFE and the secondary protector (BF-HAR-01) | 2 | **Not accepted as designed** | **SMA-B03** (open-tap-safe input ladder, open-wire check); V-BFM-03 |
| CIL-16 | Charge loops, VCTL, hardware charge window, tracker output diodes (BF-EPS-18, -19, -20, -30) | 2 | Loops single string (custom EPS); NTC1 and the diode case **not accepted** | **SMA-B04** (NTC1 plausibility), **SMA-B17** (per-tracker disable); V-EPS-12, V-BFM-04, V-BFM-08 |
| CIL-17 | 3V3_AO converter and its unswitched branches; capacitors on unfused battery nets (BF-EPS-32, -33, -34) | 2 | Single always-on rail, normal for 1U | **SMA-B15** (fail-safe capacitors), **SMA-B16** (exciter supply off 3V3_AO); flex-crack layout keep-outs |
| CIL-18 | Hardware load shed, spurious (BF-EPS-40) | 2 | **Not accepted** | **SMA-B05** (PA out of the shed); V-EPS-7, V-BFM-06 |
| CIL-19 | Antenna deployment chain: DEPLOY_EN, SW8, U80, I2C2 and its pull-ups, W7C (BF-EPS-50 to -53, BF-OBC-20, BF-HAR-16) | 2 | **Not accepted** for the bus and DEPLOY_EN; SW8 retained until the AntS ICD shows whether the AntS deploys by itself | **SMA-B06**, **SMA-B07**; flatsat test with each I2C2 device stuck |
| CIL-20 | OBC supervisor and reset network, NRST at the access port (BF-OBC-04, -05, BF-A3-01) | 2 | Hard supervisor output retained (single part, AEC-Q100 grade 1); CWD network and access-port NRST **not accepted** | **SMA-B08**, **SMA-B14**; V-OBC-1, V-BFM-02 |
| CIL-21 | OBC rail fault below the SW10 trip (BF-OBC-02) | 2 | **Not accepted** | **SMA-B09** (EPS-controller heartbeat watch of the OBC) |
| CIL-22 | FRAM (BF-OBC-07) | 2 | **Not accepted**: the fail-safe rules make a hard failure permanent silence | **SMA-B10** (second FRAM, or a commanded degraded mode) |
| CIL-23 | RF board chain at board level: SW2, RADIO_3V3, TCXO and +3V0_RX, TNC MCU and +3V1_DIG, exciter, modulation path, select and PTT gate, APC and detectors, LPF, PA temperature; RADIO_EN, RF_TX_EN, W4A TNC_RXD, W4B RADIO_3V3 (BF-EPS-44, BF-OBC-11, -13, BF-RF-02, -05, -07, -09 to -11, -14, -15, -17, -19, -26, -27, BF-HAR-09, -12) | 2 | Single-string amateur station, Class D; the PA-temperature case (BF-RF-17) **not accepted** | RF EM tests VI-RFB-01 to -13; burn-in at transponder duty; **SMA-B19** |
| CIL-24 | Evidence for REL-04: deaf front end, TNC_TXD, RX_CD, RX_FAULT (BF-RF-03, -29, -31, BF-HAR-08, -11) | 2 | **Not accepted**: the beacon is silenced after N days for receiver-path failures REL-04 means to cover | **SMA-B11**; V-BFM-05 |
| CIL-25 | Recharge-lockout relay drives; controller LOCK_SET (BF-EPS-21, -22, -49) | 2 | Drive shorts **not accepted**; the controller case retained (MSP430 watchdog, Class C firmware) | **SMA-B12**; V-EPS-9 |
| CIL-26 | Inhibit wiring that fails held: TRK_EN clamps, DS2 drive pair (BF-EPS-06, -08) | 2 | TRK_EN clamps **not accepted**; DS2 drive pair retained (inherent to the low-side FET) with construction controls | **SMA-B13**; V-HAR-1 extended to W12B |
| CIL-27 | DEPLOY_SENSE interpretation (BF-OBC-17, -18) | 2 | **Design error, to be corrected before flight** | **SMA-B02**; V-BFM-07 |
| CIL-28 | EOL fuse F1 opening early or its heater return bridged (HF-10; BF-EPS-61, -62) | 2 | Required by 97.207(g)(1)(iii) | V-EPS-8 (ITV4030 at −30 °C); no copper under F1, ≥ 0.3 mm clearance, coating, inspection |

## 4. Single Points of Failure and Class D Acceptance

The single points are the bus chain (HF-01, -03, -06 to -09, -19, -21, -22, -24, -36) and, for L2, the wing driver and
the repeat path. The reliability model (RELIABILITY_ANALYSIS v2.0 §4) gives their share of the L2 loss probability; the
EPS, OBC, flight software and the two radios account for about two thirds of it. Accepting them is consistent with Class
D only together with the test program that removes early failures (RELIABILITY_ANALYSIS v2.0 §6), because early
failures, not wear-out, dominate.

Two single points are **not** accepted, because they are cheap to remove:

1. **HF-13**, one burn driver for both wings: a second driver removes the common cause that dominates the wings-failed
   probability (1.6 % with it). *Removed by design 2026-10-27: one driver per burn resistor on four
   independent lines (`hardware/electrical/eps/EPS_BOARD_DESIGN.md` §9, RID REL-03). Re-run (RELIABILITY_ANALYSIS v2.1):
   both wings lost 0.29 %, of which 0.2 % is the burn supply the four drivers still share (SW6, U17, DEPLOY_EN) and
   0.09 % both wings failing independently.*
2. **HF-21 with D3**: a failed receiver silences the beacon after N days. A rule that keeps the beacon when the
   receiver is known failed (no carrier detected for N days, not merely no valid command) would keep L1/L2 beacon
   service; it is a regulatory question for the FCC consultation (D3), raised to the chair. *Implemented in flight
   software round 2 (REL-04, SRS-F115 to F117, with the RF board's RX_CD and RX_FAULT lines); its evidence does not cover
   a deaf front end or the TNC-to-OBC link (CIL-24, SMA-B11).*

**Board level (SQSO-FMEA-003, Version D).** The build-to boards have 59 whole-mission single points: 23 restate the
chains above at board level, 36 are new (SQSO-FMEA-003 §3). They are carried as CIL-14 to CIL-28. Retained for Class D,
with their controls: the pack path (CIL-14), the charge loops (part of CIL-16), the 3V3_AO rail and the battery-net
capacitors (CIL-17), SW8 pending the AntS ICD (part of CIL-19), the supervisor's own output (part of CIL-20), the
single-string radio (CIL-23 except the PA-temperature case), the DS2 drive pair (part of CIL-26) and F1 (CIL-28). **Not
accepted**, because a cheap change removes them or because they are design errors, with the owner board:

| Finding | Owner | Single point removed or reduced |
|---------|-------|---------------------------------|
| SMA-B02 | Flight software, ICX | DEPLOY_SENSE read with the opposite polarity: no deployment, no transmission (design error, CIL-27) |
| SMA-B03 | EPS, battery sense board | One open cell tap fires F3 or latches the lockout (CIL-15) |
| SMA-B04 | EPS | NTC1 or its wires end charging or isolate the pack (CIL-16) |
| SMA-B05 | EPS | A spurious hardware shed silences the PA for good (CIL-18) |
| SMA-B06, SMA-B07 | EPS, OBC, ICX | DEPLOY_EN, I2C2 and its pull-ups keep the antennas stowed (CIL-19) |
| SMA-B08, SMA-B14 | OBC, A3 | Supervisor timing network; NRST at the access port (CIL-20) |
| SMA-B09 | EPS, OBC | OBC rail fault below the SW10 trip (CIL-21) |
| SMA-B10 | OBC or flight software | FRAM failure becomes permanent TX-OFF (CIL-22) |
| SMA-B11 | RF (TNC firmware), flight software | REL-04 evidence gaps (CIL-24) |
| SMA-B12 | EPS | Lockout coil drives (CIL-25) |
| SMA-B13 | EPS, harness | TRK_EN clamps fail held on a frame short (CIL-26) |
| SMA-B17 | EPS | Shorted tracker output diode cannot be isolated (CIL-16) |
| SMA-B19 | Flight software, OBC | A failed PA-temperature reading silences the downlink (CIL-23) |

Recommended, retained if not taken: SMA-B01 (sunlit operation after a pack open), SMA-B15, SMA-B16 (lower the rate of
CIL-17), SMA-B18 (Category 3).

## 5. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR S&MA review | Initial issue: 38-item hardware FMEA, critical items list, single-point acceptance |
| B | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: HF-13 removed by the EPS board design (one driver per resistor, SQSO-EPS-BRD-001 §9) |
| C | 2026-10-27 | CDR S&MA analysis (RID SE-16) | §4 item 1: wings-failed probability re-run with the four-line design, 0.29 % (the consistency pass's ≈ 0.1 % left out the shared SW6/U17 burn supply) |
| D | 2026-10-27 | CDR S&MA, round 3 (P3P-04) | Basis BASELINE Rev E; companion board-level FMEA SQSO-FMEA-003 (176 modes); HF-02, -03, -04, -08, -11, -16, -21, -32 provisions brought to the build-to designs (lockout, burn limits, DS2 pole 2, shed of both Jetson switches, REL-04); CIL-01 to -04, -07, -09, -11 to -13 extended; CIL-14 to CIL-28 added; §4 board-level single points and the findings not accepted (SMA-B02 to B14, B17, B19) |
