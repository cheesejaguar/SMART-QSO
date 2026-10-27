# SMART-QSO System (Hardware) FMEA and Critical Items List

**Document ID**: SQSO-FMEA-002
**Version**: C
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR draft, functional and hardware level, from the CDR design (BASELINE Rev D, `docs/hardware/BUILD_VS_BUY.md`,
schematics S1–S8 in `hardware/diagrams/schematics/`). Piece-part FMEA follows the board designs.

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
- Success levels L1, L2, L3 and their criteria are those of `docs/mission/MISSION_SUCCESS_CRITERIA.md` v2.0.

## 2. FMEA

| ID | Item / function | Failure mode | Cause | System effect | Mission effect | Detection | Compensating provisions | Cat |
|----|-----------------|--------------|-------|---------------|----------------|-----------|------------------------|-----|
| HF-01 | Deployment switches DS1/DS2 | Fail open after separation | Switch damage, contamination, misalignment | Bus never powers | Loss of mission (DOA) | None (silent) | Two rail switches in series give no redundancy for this mode; pre-ship and post-vibe switch test (VI-LV-05) | 2 |
| HF-02 | DS1/DS2 (and DS3 proposed) | Fail closed / bounce in dispenser | Welded contact, vibration | Bus powered inside dispenser; RF or burn possible if other inhibits fail | Safety | Inhibit monitoring in vibration (VI-LV-04) | Three independent inhibits (needs DS3); timers | 1 |
| HF-03 | Battery cell (×4 series) | Open circuit | Cell CID trip, weld failure | No battery; bus only in sunlight if the EPS can start from the array (not designed) | Loss of mission | VBAT telemetry | Cell screening; no 2P redundancy (BASELINE §5) | 2 |
| HF-04 | Battery cell | Internal short / runaway | Latent defect, plating, over-discharge and recharge | Venting, fire | Safety (ground, ISS); loss of mission | Cell voltage, temperature | Screening (VI-SAF-03), temperature charge inhibit, recharge lockout (missing, SMA-06) | 1 |
| HF-05 | Charger (MPPT, CV) | Regulation fails high | IC failure, feedback open | Overcharge | Safety | Cell voltage telemetry | BMS OV and independent comparator (2-FT) | 1 |
| HF-06 | Charger | No charge | IC failure, input open | Pack drains | Loss of mission within days | SOC trend | None (single string) | 2 |
| HF-07 | BMS (BQ76920 class) | Spurious trip / FET stuck open | IC latch-up, SEU in configuration | Bus loss until reset | Loss of mission if not recoverable | Bus resets | BMS power-on defaults safe and auto-recovering; SEL current limit | 2 |
| HF-08 | Pack fuse F2 | Opens | Over-current (e.g. the wing burn: SMA-05) | Bus loss | Loss of mission | None | Burn current limit (missing) | 2 |
| HF-09 | EPS rail regulators / PDU load switches | Output short or latch-up | SEL, part failure | Rail lost or bus collapse | Loss of mission if OBC or radio rail | Rail currents | Latch-up current limiters; OBC-commanded power cycling | 2 |
| HF-10 | EOL passivation fuse F1 | Opens early | Spurious command, part failure | Battery can no longer charge | Loss of mission | Charge current | Two-key command sequence PASSIVATE_ARM (operational key) then PASSIVATE_EXECUTE (recovery key) within 600 s, refused in year 1 (TELECOMMAND_AUTHENTICATION v0.7 §6.4, designed, not implemented); two independent fire signals (SQSO-BSA-001 H9) | 2 |
| HF-11 | EOL passivation | Fails to open | Not implemented yet; command not received (receiver, keys) | Battery stays charged and cycling | Debris requirement 4.4-2 met by option 2 (controlled energy, ODAR §4); no autonomous backup recommended | Zero charge current in sunlight after the command | Battery protections remain | 3 |
| HF-12 | Solar string (6–8 strings) | Open or short | Cell crack, interconnect, blocking diode | −1/8 to −1/6 of generation | Degraded | String telemetry | Many strings | 4 |
| HF-13 | Wing burn driver (one for both wings, S5) | Fails open | Driver, SW6, timer or software | Neither wing deploys | No transponder service (L2, L3 lost); disposal in wings-failed case (ODAR §6) | Deploy-sense switches | Retry; none in hardware (one driver) | 3 (single point for L2) |
| HF-14 | Wing hinge or restraint (each) | Does not release | Line not cut, hinge jam, cold | One wing stowed | L2 possible with sun-pointing; tumbling margin lost | Deploy sense, generation | ADCS sun-pointing | 3 |
| HF-15 | Wing restraint | Releases in dispenser | Line creep or cut in vibration | Appendage in dispenser | Safety (HR-03) | — | DFMR line design, VI-SAF-05 | 1 |
| HF-16 | Burn circuit | Stuck on | Driver short, software | ≈ 83 W draw; fuse opens or fire risk at the restraint | Loss of mission; safety | Bus current | Hardware on-time limit (missing, SMA-05) | 1 |
| HF-17 | AntS VHF element | Fails to deploy | Release fault | TX into a stowed tape, link loss ≫ margin | Loss of mission (downlink) | No ground reception | AntS redundant release, retry | 2R |
| HF-18 | AntS UHF element | Fails to deploy | Release fault | Uplink and transponder input degraded | L2 at risk; telecommand degraded | Command success rate | AntS redundant release | 2R |
| HF-19 | OBC (STM32L4R5) | Latch-up, hang, corrupt NVM | SEL, SEU, flash wear, software | Reset loop or silence | Loss of mission if unrecoverable | Beacon absence | Watchdog, latch-up current limit, FRAM state, safe boot | 2 |
| HF-20 | OBC XPDR_EN | Stuck high | GPIO or software | Transponder enabled outside schedule | Power drain; duty gate bypassed | XPDR_KEY readback | Heartbeat gate (monostable) drops the enable on a hang (BUILD_VS_BUY §3.2) | 4 |
| HF-21 | 435 MHz receiver | No output | Part failure, LNA burn-out from own TX | No telecommand, no transponder input | L2 lost; **after N = 30/14 days without an authenticated command the transmitter is inhibited** (D3): beacon lost too | Command acknowledgements | None (single string) | 2 |
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
| HF-32 | Jetson rail | Stuck on (both series switches fail) | PDU switch and carrier enable both fail | Load > generation; pack flattened | Loss of mission | Rail current | Two independent switches on different lines, latch-up trip on the PDU switch (BUILD_VS_BUY §7.3) | 2R |
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
| CIL-01 | Inhibits DS1/DS2/DS3 (HF-02) | 1 | Required by CDS | DS3 added (SMA-01); VI-LV-04, VI-LV-06 |
| CIL-02 | Battery cells and protections (HF-04, HF-05) | 1 | Required energy store | VI-SAF-01–03; recharge lockout (SMA-06) |
| CIL-03 | Burn circuit (HF-16) and restraint (HF-15) | 1 | Required to deploy | Current and on-time limit (SMA-05); VI-SAF-04, -05 |
| CIL-04 | Structure and fasteners (HF-37) | 1 | Bought frame; locking features | VI-SAF-06, VI-LV-04 |
| CIL-05 | Deployment switches fail open (HF-01) | 2 | CDS requires series switches; redundancy against "fail open" would defeat the inhibit | Switch selection from flown parts; test before and after vibration |
| CIL-06 | Battery string open (HF-03) | 2 | 4S1P kept by D8 (mass); 2P would add ≈ 200 g | Cell screening; matched cells |
| CIL-07 | Charger, BMS, fuse, PDU (HF-06–HF-09) | 2 | Custom EPS (BUILD_VS_BUY §2.4); single string normal for 1U | EM and flatsat burn-in ≥ 500 h; latch-up limiters; protoflight TVAC with functional tests |
| CIL-08 | OBC (HF-19) | 2 | Single OBC normal for 1U | Watchdog, SEL limiter, flatsat hours; radiation data for the STM32L4R5 (MAP §7) |
| CIL-09 | 435 MHz receiver (HF-21) | 2 | Single receiver; coupling with the loss-of-contact inhibit makes it a whole-mission single point | Consider letting the beacon continue at reduced cadence when the receiver is known failed (proposal to the D3 consultation) |
| CIL-10 | 2 m transmitter / PA and TX_EN (HF-22, HF-24) | 2 | Single downlink | PA derating; thermal limit; flatsat burn-in at transponder duty |
| CIL-11 | Harness (HF-36) | 2 | — | NASA-STD-8739.4A workmanship; inspection |
| CIL-12 | Wing burn driver (HF-13) | 3 (L2 single point; disposal case) | One driver for both wings (S5) | Separate drivers per wing recommended (cost: one NMOS and one GPIO); VI-LV-17 |
| CIL-13 | Repeat path (HF-26) | 3 (L2 single point) | Transponder is the primary objective but not needed for the beacon | Flatsat end-to-end repeat test; burn-in |

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
   service; it is a regulatory question for the FCC consultation (D3), raised to the chair.

## 5. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR S&MA review | Initial issue: 38-item hardware FMEA, critical items list, single-point acceptance |
| B | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: HF-13 removed by the EPS board design (one driver per resistor, SQSO-EPS-BRD-001 §9) |
| C | 2026-10-27 | CDR S&MA analysis (RID SE-16) | §4 item 1: wings-failed probability re-run with the four-line design, 0.29 % (the consistency pass's ≈ 0.1 % left out the shared SW6/U17 burn supply) |
