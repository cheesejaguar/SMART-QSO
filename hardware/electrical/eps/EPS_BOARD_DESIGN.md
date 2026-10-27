# SMART-QSO EPS Board: Build-to Design

**Document ID**: SQSO-EPS-BRD-001
**Version**: G
**Date**: 2026-10-27
**Status**: CDR build-to electrical design (schematic captured as code, parts selected, ERC clean). No PCB layout, no
engineering model; every number below is analysis until its verification item (§19) closes.

**Design files** (this directory): [`design.py`](design.py) (SKiDL 3.0 schematic, the source of truth for nets and
parts), [`eps_board.net`](eps_board.net) (KiCad 8 netlist, generated), [`eps_board_nets.csv`](eps_board_nets.csv) (net to
pins, generated), [`eps_board_bom.csv`](eps_board_bom.csv) (BOM, generated), [`eps_board_erc.log`](eps_board_erc.log) (ERC,
generated), [`eps_calcs.py`](eps_calcs.py) (every computed value in this document). Regenerate with the CAD venv:
`python hardware/electrical/eps/design.py` and `python3 hardware/electrical/eps/eps_calcs.py`.

**Governing documents**: `docs/BASELINE_PARAMETERS.md` Rev E (governs every value); `docs/hardware/EPS_DESIGN.md` 2.2
(architecture); `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001 B, board-to-board pins and harness bundles); `hardware/eps/PDU_DESIGN.md` 1.3, `hardware/eps/EPS_ELECTRICAL_DESIGN.md` 1.3,
`hardware/eps/BATTERY_DESIGN.md` 2.3, `hardware/eps/SOLAR_ARRAY_DESIGN.md` 2.1; `docs/environment/BATTERY_SAFETY_ASSESSMENT.md`
(SQSO-BSA-001), `docs/environment/EMC_ANALYSIS.md` (SQSO-EMC-001), `docs/environment/POWER_BUDGET_ANALYSIS.md` (SQSO-PBA-001
v2.0), `docs/environment/THERMAL_ANALYSIS.md` (SQSO-THA-001 v2.0), `docs/environment/RADIATION_ANALYSIS.md` Rev D §5.3;
`docs/interfaces/ICD_OBC_EPS.md` (SMART-QSO-ICD-001 v1.2); `docs/manufacturing/EEE_PARTS_LIST.md` Rev C §2A; CDS Rev 14.1
§2.3; SpaceX Rideshare Payload User's Guide v11 §6.7.7.

---

## 1. Summary

| Item | Value |
|------|-------|
| Board | PC/104 (90.17 × 95.89 mm), **2.0 mm** (D-14), 6 layers, in the OBC/EPS stack; battery on flying leads (no stack connector) |
| Placements / BOM lines / nets | 742 / 157 / 435 (generated counts) |
| ICs | 80 plus the oscillator (TI unless noted): BQ76952 AFE, BQ771806 secondary protector, 4 × TPS55340-Q1 trackers, 8 × LM74700-Q1 ideal diodes, 3 × LMR43620-Q1 bucks, TPS16630 eFuse, 2 × TPS1H100B-Q1, 2 × TPS1H000-Q1, 7 × TPS2553-Q1, 2 × INA228-Q1, 4 × INA3221-Q1, 3 × INA186-Q1, ADS7138-Q1, MSP430FR2355 EPS controller, ISO1540-Q1, TCA9517-Q1, 2 × REF3425-Q1, TPS70933-Q1, 6 × TPL5111, TLV9002/OPA2991/TLV7042 analog, LVC logic, Epson SG-8018CA oscillator |
| Worst-case board dissipation | **1.33 W** sunlit ACTIVE at maximum array power with Jetson, PA and magnetorquers on; 0.37 W in eclipse; **3.77 W for ≤ 12 s** during a burn; orbit average ≈ 0.49 W (§16) |
| Hottest parts | burn-driver FET Q24–Q27 (3.0 W for 12 s, Tj 92 °C); burn buck U17 (0.95 W, 12 s, Tj 93 °C); tracker U4 (0.21 W, Tj 66 °C); D9–D12 (0.12 W); SW1 U20 (0.17 W); 5V_PA buck U16 (0.15 W); all with the board at +60 °C |
| Worst stresses | DS2 FET Vgs 73 % (7.3 V / 10 V); BQ771806 VDD 70 % (17.4 V / 25 V, on the battery sense board from Version F); PTC at 73 % of its 60 °C hold; INA3221 heater channel 65 % (16.8 V / 26 V); all within the §2.3 limits (§15) |
| Storage drain, switches actuated | **7.8 µA typical, 13.8 µA worst** (limit 20 µA; §6.6; incl. the access-port board's UMB_VMON TVS) |
| Peak pack current, one burn | **0.75 A** with the magnetorquers at 1.16 W (0.67 A without; 0.90 A with the coil bridges at their hardware limit; chair limit 1.0 A); four drivers at once (software fault) held to 2.0 A by the burn buck's limit |
| Tracker efficiency (analysis) | 90.9–92.3 % (EPS-F002 asks ≥ 92 %: **marginal**, §4.3) |

What is new against EPS_DESIGN 2.0 (all within its architecture; the departures are listed in §2.2): the parts; a
7.0 V burn buck so the linear burn drivers dissipate 3 W instead of 10 W and draw a third of the pack current; a
back-to-back DS2 so the pack return blocks charge as well as discharge; a power domain rule that keeps DS2 from being
bypassed by a sneak path; four burn drivers; an OBC supply switch with latch-up power cycling (AV-08); and the
EPS controller that the ICD's register block at 0x48 already assumed. Version B applies the OBC designer's interconnect
(SQSO-ICX-001 §7, E1–E8): Jetson readbacks VJET_RB and VSW1_RB, a magnetorquer supply switch SW11 (VBAT_MTQ), the RTC
supply 3V3_RTC, the W6B/W9B routing of J20/J21 and the allocated OBC pins (§13.2).

## 2. Requirements, standards and departures

### 2.1 Requirements implemented (EPS_DESIGN 2.2 §2)

| ID | Requirement | Implemented by | Section |
|----|-------------|----------------|---------|
| EPS-F001 | ≥ 2.16 W EOL tumbling, ≥ 3.0 W sun-pointed delivered | 8 strings, ideal diodes, 4 trackers; delivered fraction 0.90–0.92 (SIL assumes 0.892) | §4 |
| EPS-F002 | MPPT ≥ 92 % over 3.5–6.4 V | TPS55340-Q1 boost, 1 MHz, analog MPPT with MCU trim; 90.9–92.3 % by analysis | §4.3 |
| EPS-F003 | CC ≤ 0.40 A, CV 4.10 V/cell, 0–45 °C in hardware | CV 16.40 V ± 0.44 %, CC 0.394 A ± 2.3 %, NTC1 window +1.1…+44.0 °C | §5 |
| EPS-F004 | Loads from the array while charging is inhibited | Inhibit sets the charge-current set point to 0 A: trackers feed the bus, the pack neither charges nor is needed | §5.2 |
| EPS-F005 | Bus 11.2–16.4 V (16.8 V max), converters ≥ 25 V input | All VBAT parts ≥ 40 V (LMR43620 42 V abs, TPS1663 60 V, TPS1H100 40 V) | §8 |
| EPS-F006 | 3V3_AO always on | U15 LMR43620MC3 | §8.1 |
| EPS-F007 | Switched rails with limits, off at reset | 100 kΩ pull-down on every enable; limits in §8.3 | §8.3 |
| EPS-F008 | Hardware TX-OFF | RF_TX_EN AND SHED_N → SW3 U23 (PA supply) | §8.3 |
| EPS-F009 | ≥ 3 independent physical inhibits, RF and each deployable | Pack: DS2 → Q6/Q7, DS1 → Q4, DS3 → Q5; array: DS1 → Q4, DS3 → Q5, DS2 pole 2 → Q40 clamp on TRK_EN (active high; DS3's NO unused since Version E) | §6 |
| EPS-F010 | Non-resettable EOL charge disconnect, two independent signals | F1 three-terminal fuse, Q20 (EOL_FIRE) in series with Q21 (EOL_ARM latch) | §7 |
| EPS-F011 | Battery protection per SQSO-BSA-001 §3 | BQ76952 (OTP), BQ771806 + F3, F2, RT1, NTC comparators, lockout K1 | §5, §6 |
| EPS-F012 | Hardware low-voltage disconnect of payload and PA | U48 + U51/U52: < 13.6 V for 5 s sheds SW1 and SW3, release > 14.4 V | §8.4 |
| EPS-F013 | Telemetry | 7 analog lines to the OBC (incl. the Jetson readbacks VJET_RB, VSW1_RB), INA228 × 2 on I2C2, AFE / INA3221 / ADS7138 via the EPS controller | §11 |

### 2.2 Departures from the architecture documents (for the chair)

| Item | EPS_DESIGN 2.0 / analysis | This design | Why |
|------|---------------------------|-------------|-----|
| Burn supply | Drivers on VBAT through SW6 | SW6 → **7.0 V burn buck U17** → drivers | A linear 0.8 A limiter on 16.4 V dissipates up to 10 W per driver; from 7.0 V ≤ 3.0 W, and the pack sees 0.51 A per burn instead of 0.88 A. The buck's 2.8 A limit caps a four-driver software fault at 2.0 A at the pack |
| Burn drivers | One per wing (§5.1 text) / chair: one per resistor | **One per resistor (four)** | Chair decision; REL-03 |
| DS2 | One low-side FET | **Two FETs back to back** (Q6, Q7) | One FET's body diode passes charge current into the pack with DS2 open |
| DS1/DS3 gate return | not specified | DS1 returns to PACK_N, DS3 to GND, each through its own contact | Independence: no FET's drive depends on another switch's wiring (§6.3) |
| Shunts for gauge and charge loop | low side | **high side** (RS6 between RT1 and the AFE FETs) | Any ground-referenced circuit on the PACK_N node would bypass DS2 (§6.4) |
| NTC1 | common return in the pack | **own return wire** (W1 J1-6) | Keeps the hardware comparators in the ground domain without a path to the pack negative |
| Charge inhibit | "charger disabled and CHG FET open" | Charge-current set point to 0 A (trackers keep the bus up) **and** AFE CHG FET open (CFETOFF) | A boost-tracker power path cannot be disabled without losing the bus; the set point is the charge path's own control |
| Converter UVLO (EMC rule E8) | 10.8 V falling | 7.5 V (3V3_AO) and 8.7 V (others) falling; 10.5 V and 12.1 V rising | The LMR43620's fixed EN hysteresis makes a 10.8 V fall a 15.1 V start; E8's intent (BMS decides, below 11.2 V) is met (§8.1) |
| Cell-tap filters (EMC rule E6) | 1 kΩ + 0.1 µF at the AFE | 204 Ω (3 × 68.1 Ω, current limiting) at the battery sense board + 10 Ω + 0.1 µF at the AFE (Version D; internal balancing ≈ 9 mA; Version A–C: 100 Ω at the AFE, balancing at ≈ 18 mA); 1 kΩ + 0.1 µF at the secondary protector | 1 kΩ would cut balancing to ≈ 3 mA; 100 Ω/0.1 µF is a 16 kHz pole, ample against the 145.9 MHz field. **For the EMC owner:** amend E6 |
| Jetson rail trip | 1.2 A (EPS_DESIGN), 0.75 A proposed (RADIATION_ANALYSIS §5.3) | 1.20 A (1.10–1.30 A) eFuse, OBC guard 0.93 A first | A latch-up (amperes) trips either; 0.75 A would sit 1.34× the 8 W load at the low bus |
| Tracker efficiency | ≥ 92 % | 90.9–92.3 % (analysis) | See §4.3; the power margins (+31 % tumbling) absorb it; CR-2026-014 awaits the PI |
| Hardware load shed (EPS-F012) | payload and PA | **Jetson only** (Version E) | A spurious permanent shed would hold the PA off with no override (SMA-B05); a stuck-keyed PA is energy-positive; CR to EPS-F012 for the chair |
| VBAT_MTQ telemetry (Version B) | voltage and current on every rail | SW11 fault readback (FLT_SW11) on the EPS; the current is the sum of the OBC's three coil-current channels (MTQ_X/Y/Z_I, SQSO-ICX-001 §6 row 14); the voltage is VBAT (INA3221) less the switch drop (≤ 0.46 V) | The 12 INA3221 channels, the 8 ADS7138 inputs and the controller's 10 ADC inputs are all in use; a fifth INA3221 has no free address. The OBC measures the load itself |

### 2.3 Derating standard and part rules

Derating to **EEE-INST-002** (NASA GSFC 2003, as tailored by SQSO-HAR-001 §5.2) with the stricter of it and the common
50 % rule: resistors ≤ 50 % power, ≤ 80 % voltage; ceramic capacitors ≤ 50 % voltage; MOSFETs ≤ 75 % Vds and Vgs,
≤ 50 % power; diodes ≤ 70 % PIV, ≤ 50 % current; microcircuits supply ≤ 80 % of the absolute maximum; junctions
≤ 110 °C (microcircuits) and ≤ 125 °C (discretes); fuses and connectors ≤ 50 % current after temperature rerating;
relay contacts ≤ 50 %. Parts: AEC-Q100/Q101/Q200 where offered; otherwise industrial −40 °C or wider, active. Terminal
finishes recorded per part (BOM column); matte-tin parts (LMR43620, INA228, passives, connectors, discretes) take the
EEE_PARTS_LIST §2A.3 mitigation: hot-solder-dip re-tin with Sn63Pb37 where possible, conformal coat (Arathane 5750 per
NASA-STD-8739.1) over all terminations, and Sn63Pb37 assembly per J-STD-001 with the space addendum.

## 3. Architecture

```
 8 strings ─TVS─LM74700─┬─ T1 (W1F,W2F) ┐                                              ┌─ U15 3V3_AO ─ SW2 RADIO, SW4 ADCS,
                        ├─ T2 (PY)      ├ TPS55340 ×4 ─RS5─ VTRK ─F1(EOL)─┐            │  SW7 GNSS, SW8 AntS, SW9 EPS ctrl,
                        ├─ T3 (W1B,W2B) │  CV/CC/MPPT loops (VCTL)        │            │  SW10 OBC
                        └─ T4 (PX,MX,MY)┘                                 │            ├─ U16 5V_PA ── SW3 PA (TX-OFF, shed)
                                                                          ▼            ├─ SW1+U19 Jetson (VBAT, shed)
 BAT_TOP ─F2─RT1─────────RS6─[Q1 CHG|Q2 DSG]─ PACK_P ─Q4(DS1)─ VB1 ─Q5(DS3)─ VB3 ─RS8─ VBAT ─┼─ SW5 heater (F4, 40 °C cutout)
                                                                                            ├─ SW11 VBAT_MTQ (ADCS_EN) → OBC coil bridges
   4S cells (U1 BQ76952; BQ771806 + F3 on the sense board)                                                     └─ SW6 → U17 V_BURN → 4 burn drivers
 BAT_BOT ─RS7─ PACK_N ─[Q6|Q7 back to back](DS2)─ GND        umbilical J13: PACK_P / PACK_N (bypasses the inhibits for charging)
```

**Domains.** The PACK domain is referenced to BAT_BOT (cell 1 negative): cells and the battery sense board (F0A/F0B/F0N, the BQ771806 and F3), F2/RT1, the AFE U1, its FETs, the
secondary protector U2 and the DS2 gate drive. Everything else is the GND domain. DS2 sits between them. Every signal
that crosses goes through the I2C isolator U50 or a ≥ 100 kΩ series resistor, and no ground-referenced part touches
PACK_N: otherwise a component's input clamp would carry bus return current around an open DS2 (§6.4).

**Rails.** VBAT (bus); 3V3_AO; 3V3_OBC and 3V3_MCU (switched, with latch-up retry); 5V_PA; V_BURN (7.0 V, only with
DEPLOY_EN); VCTL (3.3 V LDO U40 fed by the strings: the charge-control loops run whenever the array is lit, even with
a dead bus); VHW (diode-OR of VCTL and 3V3_AO: the NTC1 comparators); REG1 (AFE LDO: AFE I2C side of U50); REF_C and
REF_A (2.500 V references in the VCTL and 3V3_AO domains).

## 4. Solar inputs and trackers

### 4.1 Strings

Eight strings on J4–J9 (W2), two contacts and two 28 AWG M22759/33 wires per pole (0.28 A each against the 0.90 A bundle-derated rating of SQSO-HAR-001 2.0 §5.2: RID EPS-13).
Each string has a **SMBJ7.0A** TVS (D1–D8; 7.0 V standoff above the 6.31 V cold-sunrise Voc; 12.0 V clamp at 50 A) and
an **LM74700-Q1** ideal-diode controller (U90–U97) with a BUK7M12-40E FET (Q31–Q38): 20 mV forward regulation, 11 mW per
string at Isc (0.4 %; the simulation allows 3 %). Below 3.1 V (POR) the FET's body diode conducts (graceful). Trackers:
T1 W1F+W2F, T2 PY, T3 W1B+W2B, T4 PX+MX+MY (EPS_DESIGN §3.2).

### 4.2 Trackers

**TPS55340-Q1** non-synchronous boost (U4–U7; 40 V / 5.25 A switch, 2.9–38 V input), 6.8 µH XGL4030 (Isat 3.65 A),
PMEG40T30ER-Q diode, 2 × 10 µF input, 2 × 4.7 µF 50 V output, synchronised to SYNC_A/SYNC_B (1.000 MHz, 180° apart).
Non-synchronous on purpose: a synchronous boost in forced PWM can run current backwards and pump its input up, which
with blocking diodes on the strings would push the TVS into conduction. Each tracker's COMP pin is pulled down through
BAT54 diodes by three loops (wired minimum): its own **MPPT loop** (TLV9002 integrator holding the string at a set point;
default 4.40 V with the EPS controller's DAC high-impedance, 3.3–5.4 V under the controller's perturb-and-observe through
75 kΩ from its SAC DAC), and the shared **CV** and **CC** loops (§5). The internal FB divider (129 k / 10.0 k) is only an
output clamp at 17.08 V nominal (16.70–17.46 V with the ±2 % internal reference), set above the CV loop so the precision
loop always governs. Enables: TRK_EN, pulled up from VCTL and held low by the active-high clamp FETs Q40 (driven from DS2's second pole) and Q41 (driven from the RBF's pole B), so a switch wire chafed to the frame removes a clamp rather than disabling the trackers (SMA-B13); TRK_EN never leaves the board.

### 4.3 Efficiency

| Case | Input | Iin avg / peak | Duty | Loss | Efficiency |
|---|---|---|---|---|---|
| T1 wing fronts, cold sunrise BOL (worst power) | 5.07 W at 5.0 V | 1.01 A / 1.27 A | 0.70 | 390 mW | 92.3 % |
| T1 at the hot EOL Vmp (worst efficiency) | 1.93 W at 3.9 V | 0.49 A / 0.71 A | 0.77 | 169 mW | 91.2 % |
| T2 body +Y, nominal | 1.16 W at 4.7 V | 0.25 A / 0.50 A | 0.72 | 105 mW | 90.9 % |
| T4 two body strings, tumbling | 2.30 W at 4.6 V | 0.50 A / 0.75 A | 0.73 | 180 mW | 92.2 % |

Losses: switch conduction (110 mΩ hot), switching (10 ns edges, 300 pF at 16.4 V), diode 0.40 V, inductor DCR, IC
quiescent (`eps_calcs.py` §2). **Not met with margin:** EPS-F002's 92 % holds only at the higher powers; at 1 W the
conversion is ≈ 91 %. With 98–99 % tracking the chain is 0.89–0.91 against the simulation's 0.892 (diode × MPPT), so
the power closure of SQSO-PBA-001 (+31 % tumbling, +54 % sun-pointed) is unchanged within 2 %. Measured efficiency
closes it (V-PWR-3); the fallback is a 500 kHz assembly option (R_FREQ 93.1 k and the divider tap), worth ≈ 1 %, at the
cost of 100 kHz-wide uplink windows (§8.2).

## 5. Charge control and power path

### 5.1 Loops (VCTL domain, hardware)

| Loop | Implementation | Value | Accuracy / note |
|---|---|---|---|
| CV set point | U45A integrator; PACK_P × 10.0/65.6 (0.1 %) vs REF_C REF3425 2.500 V | 16.40 V | ±0.44 % (reference 0.09, divider 0.20, offset 0.06, path drop 0.10) |
| Worst cell at end of charge | set-point error + 50 mV balance window | 4.168 V | below the 4.20 V cell maximum and the 4.25 V AFE threshold |
| CC limit | U44 INA186A3 (×100) on RS6 10 mΩ = 1.0 V/A vs 21.5 k / 4.02 k | 0.394 A | ±2.3 % |
| Charge inhibited (cold, hot, lockout) | CC set point forced to 0 V by Q18 and, when locked, by K1 pole 2 | ≤ 5 mA residual | INA186 offset 50 µV; the AFE CHG FET is opened as well |
| Trackers' FB clamp (VCTL loops dead) | internal reference | 17.08 V (16.70–17.46 V) | AFE COV 4.25 V acts above 17.0 V; not counted as an overcharge control |

The CV divider draws 0.25 mA only while the AFE FETs are on (none in storage).

### 5.2 Power path

The trackers feed PACK_P, the node between the pack (behind the AFE FETs) and the inhibit chain. Charging is the
pack current into the cells, which the CC loop regulates; when charging is not allowed the set point is 0 A, so the
trackers supply exactly the bus load in sunlight and the pack floats. With the array short of the load the bus falls to
the pack voltage and the pack discharges through the DSG FET (and the CHG FET's body-diode protection in the AFE). The
spacecraft therefore runs from the array with charging refused in the cold (EPS-F004), with the pack locked out (§5.4)
and with a dead pack: VCTL comes from the strings, so the loops work before 3V3_AO exists.

### 5.3 Charge-temperature window (NTC1, hardware)

NTC1 (10 kΩ, B 3435 K, on a cell, own return) is biased by 10.0 kΩ from VHW; a 0.1 % ladder (267 k, 365 k, 31.6 k, 104 k,
229 k) gives ratiometric thresholds, so the supply value does not matter:

| Comparator (TLV7042-Q1, open drain) | Threshold | Action |
|---|---|---|
| U53A cold | +1.1 °C | below: WIN_OK low → PERMIT low → CHG_INHIBIT (CC set point 0, AFE CFETOFF) |
| U53B hot | +44.0 °C | above: same |
| U54A heater cutout | +40.0 °C | above: Q22 (in series with the heater, low side) opens |
| U54B pack over-temperature | +60.0 °C | above: OT60 high → AFE DFETOFF (BOTHOFF): both FETs open |

Each threshold within ±1 K (ladder and bias 0.1 %, NTC 1 %, comparator offset and internal 6.5 mV hysteresis); the
window sits 1 K inside 0–45 °C, so SYS-602 holds at the tolerance edge. Second, independent layer: the AFE's own OTP
UTC 0 °C / OTC 45 °C / OTD 60 °C on NTC2 (TS1), which opens its CHG (and DSG) FET without the comparators; third: the
flight software on the NTC2 telemetry (SQSO-BSA-001 H5).

**NTC1 plausibility (Version E, RID SMA-B04).** U66 (TLV7042-Q1) flags NTC1 open (ratio > 0.985 of VHW, below
−55 °C; 15.0 k / 1.00 M) or shorted (ratio < 0.030, above +150 °C; 1.00 M / 30.9 k), outside the −40…+100 °C
plausible range (0.961…0.090). NTC1_BAD (U67) then hands over: the charge window to the AFE's UTC/OTC on NTC2 (U68:
WIN_EFF = WIN_OK OR NTC1_BAD feeds K1), the 60 °C BOTHOFF to the AFE's OTD on NTC2 (Q43 holds OT60 low), and the heater
cutout to the flight software on NTC2, SW5's limit and F4 (U69: Q22 held on). A failed NTC1 or its wire therefore
neither stops charging for good nor isolates the pack; the controller reads NTC1_BAD (P4.3) and reports it
(CHG_INHIBIT bit 6).

### 5.4 Latched recharge lockout (RID EPS-12 / SMA-06)

| Element | Implementation |
|---------|----------------|
| Detection | BQ76952 Safety Cell Undervoltage permanent fail (SUV), threshold 2.00 V, delay 5 s, written to OTP at acceptance; the AFE runs in NORMAL/SLEEP on orbit (protections on) so it sees a cell pass 2.0 V while the pack drains |
| Memory | **K1 Panasonic TQ2SA-L2-3V-Z**, 2-coil latching relay: non-volatile without power, independent of any RAM or OTP |
| Set | AFE FUSE output (PF) → 100 k/100 k → U56 OR (with the EPS controller's LOCK_SET, which it raises if it reads the PF or a cell < 2.0 V) → U57 TPL5111 one-shot (powered by the request, so one 50 ms pulse per request and on every power-up while it persists) → Q28 and Q30 in series → SET coil (Version E, RID SMA-B12: a single shorted driver FET leaves the coil off) |
| Effect | Pole 1 (NC, common WIN_EFF) opens PERMIT → CHG_INHIBIT: CC set point 0 and AFE CHG FET off. Pole 2 (NO) grounds the CC set point directly. Two independent paths; the AFE PF also holds its FETs off until reset. The bus keeps running from the array (§5.2) so the OBC and receiver can take the clear command |
| Clear | **Only** LOCK_CLR (OBC GPIO, after the authenticated telecommand) **AND** UNLOCK (EPS controller GPIO, written by the OBC over I2C as a second step) → U58 → U59 one-shot → Q29 and Q39 in series → RESET coil. No single line, glitch or controller fault clears it |
| Precharge | Below 3.0 V/cell the AFE charges through Q3 and 200 Ω (≤ 45 mA); at an AFE wake with a cell below 2.0 V, at most 45 mA for the 5 s SUV delay (0.06 mAh) can flow before the PF |
| Residual risk | Relay change of state by shock (TQ2 functional shock 750 m/s²): locked-out after a shock is safe, unlocked by a shock during a lockout needs a second event; included in V-EPS-9 |

## 6. Battery protection and inhibits

### 6.1 Protection chain and coordination

| Item | Value | Margin |
|---|---|---|
| Largest legitimate pack current (Jetson at the 12 W guard + transponder at 1 W RF, 2.42 W DC (RF board E) + housekeeping + heater + magnetorquers 1.16 W, three rods saturated) | 1.36 A | |
| Same, coil bridges at their hardware limit (0.23 A: an OBC control fault) | 1.51 A | 78 % of the RT1 hold at 60 °C: no trip |
| Same, SW11 in its current limit (0.40 A maximum, VBAT_MTQ shorted; auto-retry 4 % duty) | 1.68 A | 87 % of the RT1 hold: no trip |
| Single burn, normal sequence (one resistor, 0.88 A at +10 %) + housekeeping + heater + magnetorquers 1.16 W | 0.75 A | chair limit 1.0 A |
| Same, coil bridges at their hardware limit (0.23 A) | 0.90 A | chair limit 1.0 A |
| Burn + transponder keyed at 1 W RF (not concurrent by design: RF silence to 45 min, burn at 30 min) | 0.94 A | chair limit 1.0 A; 1.09 A with the coil bridges also at their fault limit (BMS 4 A) |
| All four burn drivers on (software fault): burn buck in its 2.8 A current limit | 2.00 A | hardware cap |
| AFE OCD1 (40 mV on RS7 10 mΩ, 500 ms) | 4.0 A (3.6–4.4) | 2.6× the largest load; below F2 |
| AFE SCD (100 mV, 15 µs) | 10 A | prospective short-circuit current 80–100 A typical, ≤ 314 A bound (below) |
| F0A, F0N (battery sense board) and F2 (EPS): **FM12A72V5.0A** (AEM, MIL-PRF-23419/12); F0B (sense board, BAT_TOP): **FM08A125V5A** (Littelfuse, /8); 5 A at 60 °C (× 0.80 rerating, assumed) | 4.0 A | largest load 34 %; four-driver fault 2.0 A 50 % |
| Prospective short-circuit current, short in W1: 16.8 V / (4 × 10 mΩ cells, a lower bound about a third of the 35E's typical DC resistance, + F0A, F0N at 13.5 mΩ and F0B at 12.6 mΩ minimum) | 211 A | FM12 (F0A, F0N) 1000 A at 72 VDC: 4.7×; FM08 (F0B) 300 A at 125 VDC: 1.42× |
| Tap short, tap to frame (one sense-board tap chain 3 × 68.1 Ω 2512 + FM12A72V3/4A tap fuse) | 82 mA | 0.46 W per 1 W resistor; tap to tap 41 mA; the 28 AWG tap wire (0.90 A derated) cannot heat |
| RT1 2920L260/33DR hold at 60 °C | 1.92 A | 71 % of hold |
| F3 ITV4030L1412NR 12 A (secondary-protector fuse, on the battery sense board from Version F) at 60 °C (10 A) | 10 A | 14 % |

**W1 is upstream of F2** (SQSO-HAR-001 2.0 §9.1): a BAT_TOP lead shorted to BAT_BOT or to the frame (GND, which
reaches the pack negative through DS2 once released) would see the cells' short-circuit current with no EPS fuse in
the loop, and the AFE's SCD opens FETs downstream of such a fault. The battery's sense board (BATTERY_DESIGN 2.7 §7.2)
therefore carries **F0A** (FM12A72V5.0A, AEM, MIL-PRF-23419/12) and **F0B** (**FM08A125V5A**, Littelfuse,
MIL-PRF-23419/8: a different part type, slash sheet and manufacturer, Version E) in series in BAT_TOP within 10 mm of
the cell-4 tab, and **F0N** (FM12A72V5.0A) in BAT_BOT at the cell-1 negative (the interrupter in the grounded
negative leg, JSC 20793 Rev D §5.1.3.1a). Every short loop of the pack (lead to lead, lead to frame, on the EPS board, in a load) passes all
three; the EPS's F2 adds a fourth for faults downstream of J1. The cell taps carry, at the sense board, a 0.75 A
FM12A72V3/4A fuse and a **3 × 68.1 Ω 2512 current-limiting chain** each (the AFE's tap filter resistors moved there
from this board, which keeps 10 Ω + 0.1 µF at the AFE): a tap short in W1 is limited to ≤ 82 mA. The fuses are in
series and any one opening isolates the pack for good, so no selectivity is needed among them; the AFE acts first on
every fault downstream of its FETs. The W1 leads also keep their construction controls (22 AWG, PTFE over-sleeve,
0.12 m, hipot V-HAR-1), which are not counted.

**External-short fault tolerance (RID P3E-02).** Downstream of the AFE FETs: (1) the positive-leg fuses F0A/F0B/F2,
(2) the negative-leg fuse F0N, (3) the AFE SCD (10 A, 15 µs) and OCD: three controls, 2-FT, counting the positive-leg
fuses as one (one part type in one leg). Upstream of the AFE FETs (W1, J1, F2–RT1 copper, and F3 on the sense board): (1) F0A, (2) F0B, (3)
F0N: three interrupting elements in every loop, **2-FT**. From Version E the two positive-leg fuses are diverse
(FM12 by AEM, FM08 by Littelfuse), so they are independent controls by design and the count no longer rests on the
CDR round-3 lot decision (C-16). F0B's interrupt rating is 300 A at 125 VDC (the FM08's commercial equivalent, the
Littelfuse 265 series); F0A and F0N stay in every loop while intact, so the current F0B must interrupt is ≤ 211 A
(cells at 10 mΩ, F0A and F0N at 13.5 mΩ, F0B at 12.6 mΩ minimum): 1.42×. It is below the 314 A one-fuse bound, which
F0B never sees. Its interrupting rating at 16.8 V, its vacuum use (MIL-PRF-23419 §6.1.3 for FM08) and its 60 °C
rerating are open items for procurement (§21). Tap shorts: two current-limiting chains (one per tap end) and the tap
fuses: 2-FT. RT1 (40 A) is not counted as an interrupter. The SQSO-BSA-001 D table carries these counts.

**Breaking capacity (Version C).** The Littelfuse Nano2 0451005.MRL of Version A–B is rated 50 A at 125 V AC/DC
(Digi-Key) and 300 A at 32 VDC (the 451/453 datasheet table as indexed; some revisions 400 A): below the 314 A bound of
the prospective current, and the bound rests on the cells' resistance, which nobody has measured. F0A, F0B, F0N and F2 are
therefore the space-heritage **FM12A72V5.0A** (MIL-PRF-23419/12 solid-body fuse, 72 VDC, short-circuit interrupt
1000 A at the maximum DC voltage per the slash sheet; cold resistance 13.5–22.5 mΩ, NASA NEPP parts list), so the
rating holds for any cell and interconnect resistance ≥ 3.3 mΩ in total (16.8 V / 1000 A less the fuse's 13.5 mΩ:
0.8 mΩ per cell, far below any 18650), without relying on a measured cell resistance. F0A, F0B, F0N and F2 are the only elements rated to break
the pack's short-circuit current; RT1 (40 A maximum) is not, so it sits after F2. **F3 in the short loops (RID SMA-B22).**
F3 (ITV4030, 12 A, interrupting rating 50 A) is in every pack-short loop on the sense board, in series with F0A, F0B
and F0N (up to 211 A). It never has to interrupt: the series FM12/FM08 fuses are each rated to interrupt the
prospective current (1000 A / 300 A), so the circuit is opened by the fastest of them in any case, and F3 sees at
most their let-through. F3's element withstands 60 A for 5 ms 10 000 times without melting (5 × I_rated, its
data sheet's interrupting-current test), i.e. a melting I²t well above 18 A²s; a 5 A fast fuse at 40 × its rating
clears in about a millisecond with a let-through of the order of 10 A²s (to be confirmed: AEM and Littelfuse I²t data
are a data item). If F3's element melted first, its arc would be in series with fuses that are still conducting and
that then clear: arc energy ≲ 20 V × 200 A × 1 ms ≈ 4 J in a ceramic body, no interruption duty. F3 is included in the
pack hard-short abuse test (SQSO-BSA-001 §3.1, V-BSA-9): pass if F3 is intact, or open without rupture, smoke or
damage to the sense board. The PTC's trip rises to ≈ 6.5 A at −20 °C;
it is a backup.

**AFE BQ76952 OTP settings** (written and verified at EPS acceptance; V-EPS-2): COV 4.25 V (1 s), CUV 2.80 V,
recovery 3.00 V, OCD1 40 mV 500 ms, SCD 100 mV 15 µs, OCC (charge) 10 mV (1 A) 1 s, UTC 0 °C / OTC 45 °C / OTD 60 °C on
TS1 (NTC2), FET temperature on TS3 (RT2), SUV PF 2.00 V 5 s (FUSE drive on PF, no PF write to OTP), precharge below
3.0 V, body-diode protection on, internal balancing above 3.9 V at > 50 mV spread (≈ 9 mA through the 2 × 214 Ω tap loop; ≈ 18 mA before Version D, one
cell at a time), CFETOFF and DFETOFF (BOTHOFF) active high with internal pull-downs off, auto-SHUTDOWN on low voltage
**disabled** (it would blind the SUV detection), wake from SHUTDOWN on TS2 and LD. Cell inputs: VC0–VC2, VC15, VC16 to
the four cells; VC3–VC14 shorted to VC2 (datasheet §16.6).

**Secondary over-voltage protector** **BQ771806DPJR** (4.350 V ± 10 mV per cell, 3 s delay, CMOS active-high
output), **on the battery sense board from Version F** (RID MFG-003 item 1, SMA-B03): its own taps at the cells
through 1 kΩ + 0.1 µF (no wire shared with the AFE's taps), own supply after F0A/F0B and ahead of F3, own ground at
the cell-1 negative; output through 47 k/100 k and an 8.2 V zener to Q16, which pulls F3's heater (11.2–20 Ω) to the
cell-1 negative: 15–27 W, F3 opens in < 1 min (datasheet: 6–35 W window). F3 is in the pack line (sense board,
BAT_TOP after F0B), so a trip isolates the pack for good (charge and discharge); the spacecraft then runs on the
array in sunlight. **Temperature (ITV4030, −10 … +65 °C):** F3 now sits on the heated pack. It must blow only during an
overcharge, which needs charge current, which the hardware window refuses below +1.1 °C at the pack (and the AFE
below 0 °C): F3's blow function is only ever needed inside its rating. It carries current colder than −10 °C only in
the first orbit after a cold soak (pack −10 … −16 °C, THERMAL_ANALYSIS 2.2 §4.3; above +3.6 °C in every orbit case),
and the hottest it sees is the pack's +60 °C trip (the +64 °C dispenser soak is unpowered, no current). The carry
case below −10 °C and the qualification margins (−40 °C, +70 °C) are V-EPS-8. The EPS controller detects an open
F3 (or an open sense-board fuse) as cell taps present with BAT_F absent (CHG_INHIBIT bit 4).
**Heat at the cell when F3 is blown (RID SMA-B21).** The heater dissipates 15–27 W until the element opens (the
data sheet bounds the cut time to < 1 min; ≈ 5–15 s at these powers on its cut-time curve): ≤ 1.6 kJ in the bound.
Lumped, if all of it went into one 50 g cell (c ≈ 1.0 J/g·K) the can would rise 32 K, from the +45 °C charge limit
to 77 °C, above the limit used here: **cell can ≤ +60 °C** (the cell's discharge maximum; the 35E's self-heating
and vent region starts well above 100 °C). So F3 sits on a **thermal island** of the sense board, ≥ 25 mm from every
cell tab and can, on the board's edge away from the cells, joined to the BAT_TOP chain by a 25 mm × 2 mm 2-oz trace
and to Q16 by a narrow trace, with no copper pour to the tabs: the conduction path is ≈ 390 K/W (trace 460 K/W in
parallel with 2 600 K/W of FR-4), radiation from 1 cm² at ≤ 200 °C to the cells ≤ 0.15 W; with the island at the
element's melting temperature the heat reaching the cells is ≤ 0.6 W, ≤ 35 J in 60 s: **≤ 1 K** at the nearest can.
The heater energy stays in the island and the fuse body (V-BSA-2 measures the island, the cell-4 tab and the nearest
can during the F3 trip on the EM pack). This gives overcharge three dissimilar controls: CV loop,
AFE COV (OTP), fixed-threshold BQ771806 + F3 on the sense board (SQSO-BSA-001 H1: two-fault tolerant; RID EPS-11).

**Fuel gauge**: U28 INA228-Q1 on RS6 (high side), 20-bit, charge and energy accumulators, OBC I2C2 at 0x44, powered
from 3V3_AO; plus the AFE's coulomb counter (RS7). Two NTCs at the cells: NTC1 (hardware comparators, ADS7138 telemetry)
and NTC2 (AFE TS1). **Pack-to-frame isolation** ≥ 1 MΩ at 50 VDC is a property of the pack and its mount and is tested
with J1/J2 disconnected (V-BSA-4): on the board, PACK_N joins GND only through DS2 (§6.4).

### 6.2 Inhibit chains

| Inhibit | Switch (connector) | Element | Gate drive | Breaks |
|---------|--------------------|---------|-----------|--------|
| **1 DS2** | J16 pole 1: C via 10 k from BAT_F **OR PACK_P** (BAT54C, Version E, RID SMA-B01: with an open pack path the array still releases DS2's FETs, so the bus comes up in sunlight), NC → S2N; RBF pole A (J14 1-2) in series → S2R | Q6 + Q7 BUK9Y4R4-40E back to back, PACK_N–CS2–GND | S2R → 1.3 M → G2, 1.0 M + 47 nF to CS2; Vgs 4.9–7.3 V | pack (charge and discharge) |
| **2 DS1** | J15: C via 10 k to PACK_N, NC → S1N | Q4 IPD50P04P4L-11 in PACK_P→VB1 | 1.0 M gate–source, 47 nF, 1.0 M to S1N; |Vgs| 5.6–8.4 V | pack and array |
| **3 DS3** | J17: C → GND, NC → S3N (NO not used from Version E) | Q5 IPD50P04P4L-11 in VB1→VB3 | as DS1, returned to GND | pack and array |
| **DS2 pole 2** (array) | J16-4 C2 (VCTL via 10 k), J16-5 NO2 → 10 k → gate of Q40 (1 M to GND) (DS2 is a two-pole switch) | Q40 2N7002BK clamps TRK_EN (the four trackers' EN) low | **active high** (Version E, RID SMA-B13): stowed, the contact carries VCTL to Q40's gate; a wire chafed to the frame turns the clamp off and cannot disable the trackers; TRK_EN itself never leaves the board | the array's boost: the string pass-through (≤ 6.2 V) is below every converter start |
| (4) | — | SW3 (RF), SW6 + driver FET (wings), SW8 + AntS ARM (antennas) | OBC lines, low at reset | margin only |

| Gate network | Network | |Vgs| (rating) | Filter time constant | Current |
|---|---|---|---|---|
| DS1 / DS3 high-side P-FET gate | 1.0 M / 1.0 M, 68 nF | 5.6–8.4 V (52 % of 16 V) | 34.0 ms on / 68.0 ms off | 8.4 µA released, 0 stowed |
| DS2 back-to-back N-FET gate | 1.3 M / 1.0 M, 68 nF | 4.9–7.3 V (73 % of 10 V) | 38.4 ms on / 68.0 ms off | 7.3 µA released, 0 stowed |

**Three inhibits on each source (RID P3E-01).** Pack: DS2 (Q6/Q7, the return), DS1 (Q4), DS3 (Q5). Array: DS1 (Q4)
and DS3 (Q5) in VTRK → F1 → PACK_P → VBAT, and DS2's second pole holding TRK_EN low through the clamp Q40. With the trackers disabled the boost passes the string through its inductor and
diode: ≤ 6.16 V (Voc 6.31 V at −45 °C BOL less 0.15 V), below the lowest converter start (U15 9.9 V at the EN
threshold minimum; U16/U17 11.4 V; the eFuse U19 12 V), and every load switch needs an enable from 3V3_AO or the OBC,
both unpowered: no rail, no RF, no deployment with any two of DS1–DS3 failed closed (`eps_calcs.py` §5a; V-EPS-3).
DS2 becomes a two-pole (DPDT) switch: W12B carries five wires (for the harness and the frame RFQ). **Fallback if no
DPDT fits the ≤ Ø4.5 mm body:** a second SPDT switch DS2B in a +Z standoff on the same rail as DS2, wired to J16-4
(C, VCTL feed) and J16-5 (NO, clamp gate), its NC unused; J16 and the board are unchanged, and the array then has
three separate switches (DS1, DS3, DS2B) on its path (the switch RFQ asks for both); W12C carries two
(DS3's NO wire is dropped in Version E).

A contact closure must last 17.0 ms (12.8 ms with the X7R gate capacitor at −10 % and −15 % at −40 °C; 68 nF from Version F, 47 nF gave 8.8 ms at that corner) before a DS1/DS3 FET begins to conduct at the 11.2 V minimum, so contact chatter in
vibration cannot power the bus; the switches carry µA only (RID EPS-05). Wiring: each switch on its own rail and its own
three-wire harness; the 10 kΩ at the board end of DS1's and DS2's common limits a harness short to the frame to ≤ 1.7 mA.
The RBF (J14) is two-pole: pole A in series with DS2's drive, pole B (closed with the pin in) carrying VCTL to the
gate of a second clamp Q41 on TRK_EN, active high like Q40 (not counted; it keeps a bench-lit array from powering
the bus with the pin in). DS2's release also
wakes the AFE from SHUTDOWN (Q23 pulls TS2 to VSS); the array's first light wakes it through LD as a second path.

### 6.3 Independence

No single failure removes more than one of the three (for the array: DS1's FET, DS3's FET, DS2's second pole; the
TRK_EN node stays on the board and is clamped by Q40 (DS2 pole 2) and Q41 (RBF pole B), separate FETs with separate
gate networks; a grounded switch wire removes a clamp, never adds one): each has its own switch, rail, harness, gate network and power
FET(s); the DS1 and DS3 FETs are separate parts with separate drives returned to different nodes; a DS1/DS3 drive can
only turn its FET on if DS2 is also closed (their drive current returns through the pack), which makes them harder,
never easier, to defeat. A failed-short FET removes its own inhibit only. Each switch drives a gate, never the bus
(UG v11 §6.7.7 "from a mechanical separation signal").

### 6.4 Sneak paths across DS2 (design rule)

With DS2 open, any path from GND to PACK_N would carry the bus return around it: an amplifier input clamp on a low-side
shunt, an I2C pin of the AFE, a logic input fed from the PACK domain. Rule applied in the netlist: (1) no GND-referenced
part connects to PACK_N or BAT_BOT (the gauge, IBATT and CC amplifiers sit on the high-side shunt RS6, whose inputs are
high-impedance up to 85 V/40 V); (2) the AFE talks through the ISO1540 isolator U50; (3) the remaining crossings
(CHG_INHIBIT → CFETOFF, OT60 → DFETOFF, AFE_FUSE → FUSE_LVL, AFE_ALERT, DEPLOY_SENSE) each have ≥ 100 kΩ in series,
limiting any sneak current to ≤ 0.17 mA (≤ 17 µA for DEPLOY_SENSE), far below the 3V3_AO buck's start-up demand;
(4) NTC1 has its own return. Verification: V-EPS-4 measures GND–PACK_N leakage with DS2 open (≥ 100 kΩ).

### 6.5 Umbilical access

J13 (through the access port; new harness W11 for the chair): UMB_CHG → F5 1 A → D16 → PACK_P, return to PACK_N, so
the ground charger reaches the pack with all three switches actuated and the bus off; the charger voltage on LD wakes
the AFE, which then protects the charge (COV, OTC/UTC, F3). UMB_VMON (100 kΩ from BAT_F) for the GSE voltmeter;
UMB_SHUT → AFE RST_SHUT (held high > 1 s by the GSE puts the AFE in SHUTDOWN for storage, rule E9).

### 6.6 Storage drain with the switches actuated (LVR-071 ≤ 20 µA)

| Element | Typical (µA) | Worst (µA) |
|---|---|---|
| BQ76952 SHUTDOWN (incl. TS2 wake source) | 1.0 | 3.1 |
| BQ771806 secondary protector | 1.0 | 2.0 |
| BQ771806 cell inputs (4 × < 0.1 µA) | 0.1 | 0.4 |
| Open-tap ladder, 4 × 1.00 MΩ across the cells (Version E, RID SMA-B03) | 3.8 | 4.2 |
| Open-tap ladder, 4 × 2.2 MΩ on the BQ771806's own taps (battery sense board, Version G, RID SMA-B24) | 1.7 | 1.9 |
| Q16 secondary-fuse FET leakage (PMV40UN2, 25 °C / 60 °C) | 0.05 | 1.0 |
| Q23 wake FET + DS2 gate network (contacts open) | 0.0 | 0.1 |
| Pack TVS D14, CV and VBATT dividers: isolated by the open AFE FETs | 0.0 | 0.0 |
| Cell-tap filter capacitors (ceramic leakage, 9 × 0.1 µF) | 0.01 | 0.1 |
| SMF24A TVS on UMB_VMON on the access-port board A3 (OBC item C18), fed from BAT_F through the EPS's 100 kΩ (W11 stays attached in flight; worst case its return is PACK_N, inside the pack loop) | 0.1 | 1.0 |
| **Total** | **7.8** | **13.8** |

Worst 13.8 µA = 69 % of the 20 µA limit (margin 6.2 µA). From the final charge at L-47 d (60 ± 5 % SOC,
BASELINE §4) to separation: 15.6 mAh, 0.45 % SOC, so ≥ 54.5 % before the cells' own self-discharge (a cell property
outside this budget), against ≥ 40 % required. 90 days at the worst drain: 29.8 mAh (0.86 % of 3.45 Ah). The 1 µA
through 100 kΩ moves UMB_VMON by at most 0.1 V. No fix is needed. The AFE enters SHUTDOWN on the GSE command after the
final charge (UMB_SHUT) and stays there until DS2's release or the array's light wakes it; in SHUTDOWN its FETs are open,
which also isolates PACK_P (not counted as an inhibit).

## 7. End-of-life charge disconnect (97.207(g)(1)(iii); RID EPS-06)

F1 **ITV4030L1412NR** (three-terminal fuse, 12 A, heater 11.2–20 Ω, triggerable 10.5–19.6 V; rated −10 … +65 °C)
sits between the trackers' output (VTRK) and PACK_P (it cannot move to the pack: it is in the array path). **Temperature
(RID MFG-003 item 1):** the EPS board reaches −11.5 °C in the cold orbit (C12) and −27.6 °C in the first orbit (L4),
+48.4 °C hot (H7), so F1 carries current outside its −10 °C rating in cold orbits. Its blow is needed only once, at
end of life, and the passivation sequence fires it only with the EPS board at ≥ 0 °C (INA228 die temperatures, in
sunlight; ICD-001 §4.6 sequence step), so the blow happens inside the rating; carrying below −10 °C and the
qualification margins are V-EPS-8 (flight-lot qualification at −40 °C and +70 °C). No three-terminal fuse rated
−40 … +70 °C exists in this footprint (SQSO-MFG-003); a disconnect by latching relay or FETs would need power to hold
or a second non-volatile device and is not adopted. F1: opening it disconnects the array from the pack and the bus, so the loads drain the pack to the
BMS cut and the spacecraft stays dark. Firing needs **both**: Q21 (gate from the arm latch U61, a D flip-flop
clocked by U64 = the OBC's EOL_ARM (1 ms RC, Schmitt buffer U60) **AND** the EPS controller's EOL_KEY, cleared at every
power-up) and Q20 (gate from the OBC's EOL_FIRE, 1 ms RC, 100 k pull-down), in series in the heater return. The EPS
controller raises EOL_KEY (P4.2, 100 k pull-down) only after a keyed two-write I2C sequence and holds it ≤ 10 s
(Version D, RID P3E-11), so arming needs a second, independent source besides the OBC's GPIOs. Heater power 9.8–24 W at 14.0–16.4 V (the datasheet's 6–35 W
window, cut < 1 min); fire in sunlight so the array also feeds the heater. No single failure fires it: a stuck line, a latch upset, one shorted FET, or a single OBC
pin-map or GPIO fault (EOL_ARM and EOL_FIRE both driven) each leaves another element open: the arm also needs the EPS
controller's EOL_KEY. A **software** fault is not excluded in principle: the OBC flight software that sends the keyed
I2C sequence also drives EOL_ARM and EOL_FIRE, so a flight-software fault that executes the whole sequence (two
authenticated telecommands, PASSIVATE_ARM and PASSIVATE_EXECUTE, and the key) would fire it; the controls against that
are the two-command protocol, the key, and the EPS controller's own checks (it refuses the key unless the OBC first
reads back EOL_ARMED low and F1 intact, and only within 10 s of the PASSIVATE_ARM write). F1_SNS (1 M / 220 k from the heater pin) tells the
controller whether F1 is intact. Pins and commands: EOL_ARM and EOL_FIRE on J3-32/33 (§13; the flight software's
PASSIVATE_ARM / PASSIVATE_EXECUTE of REG-01 map onto them).

## 8. Distribution

### 8.1 Converters

| Converter | Output | Inductor ripple | Loss at full load | Efficiency |
|---|---|---|---|---|
| U15 LMR43620MC3 3V3_AO | 3.30 V / 0.30 A | 0.39 A p-p | 67 mW | 93.7 % (85–90 % at 0.1 A, forced PWM) |
| U16 at the transponder load | 5.00 V / 0.35 A (1.76 W DC at 0.5 W RF) and ≈ 0.55 A at 1 W RF | | 60–145 mW | ≥ 94 % |
| U16 LMR43620MC5 5V_PA | 5.00 V / 0.54 A | 0.52 A p-p | 143 mW | 95.0 % |
| U17 LMR43620MC5 (adj.) V_BURN | 7.04 V / 1.76 A | 0.60 A p-p | 948 mW | 92.9 % |

All 2 A parts, 42 V abs max input (VBAT 40 %), no spread spectrum (MC variants), synchronised to SYNC_A.
**3V3_AO accuracy (for the RF board's LMX2571, 3.15 V minimum).** The LMR43620MC3's fixed output is 3.27–3.33 V
(±0.9 %, datasheet, VIN 3.6–36 V, forced PWM); the feedback senses after the RS9 shunt; with three 22 µF the 50 mA load
steps of the radio move it < 20 mV. 3V3_AO is therefore held to **±2 %** (3.23–3.37 V) including transients. At J19-5
(3V3_RADIO) the SW2 switch (135 mΩ maximum) and RS12 (100 mΩ) take 9 mV at 38 mA: ≥ 3.22 V at the RF board, 70 mV
above the synthesizer's minimum before the harness (≤ 3 mV at 38 mA for the 0.145 m of 28 AWG in W4B).

**PA rail (5 V or 6 V).** U16 (2 A part) carries the transponder's 1.76 W DC at 0.5 W RF (0.35 A) and ≈ 2.7 W at 1 W RF
(0.55 A) with 3.6× margin; SW3 trips at 0.92 A minimum. A 6 V rail is a feedback change on U16 (the MC5 part is also
adjustable: 50.0 k / 10.0 k), but SW3's TPS2553 (6.5 V operating maximum) would sit at 92 % of its rating, above the
80 % rule: SW3 would become a 40 V high-side switch (TPS1H000-Q1 class, 1 Ω, limit ≥ 0.6 A: adequate for 0.55 A only
with a 0.5 W loss, or a TPS1H100B-Q1 at 0.1 Ω). Recommendation: keep 5 V unless the PA design shows ≥ 5 % better
efficiency at 6 V for the same RF power; the change on this board is then U16's divider and SW3 → TPS1H100B-Q1.
 UVLO (EN
dividers): 3V3_AO starts at 10.5 V and stops at 7.5 V; 5V_PA and V_BURN start at 12.1 V and stop at 8.7 V (§2.2).
The PA has its own buck and the Jetson carrier takes VBAT through its eFuse (EMC rule E3, RID EMC-02).

### 8.2 Frequency plan (EMC rule E1)

Y1 Epson SG-8018CA 4.000 MHz (±50 ppm, −40…105 °C) divided by 4 (U62, U63) gives 1.000 MHz on SYNC_A (trackers T1/T3 and
the bucks) and SYNC_B (T2/T4, 180°, halving the array-bus ripple). Harmonics sit at n × 1.000 MHz ± 22 kHz. **Uplink
channels that keep ≥ 200 kHz from every harmonic: 435.222–435.778, 436.222–436.778, 437.222–437.300 MHz** (for the
chair and the RF lead: put this in the IARU request; SQSO-EMC-001 V-EMC-3 checks it). If the clock fails the trackers
free-run at ≈ 1 MHz (R_FREQ 46.4 k) and the bucks drop to auto mode: no spread spectrum in either case.

### 8.3 Load switches and latch-up trips (RID AV-08, RADIATION_ANALYSIS §5.3)

| Switch | Part | R_ILIM / R_CL | Limit nominal (min–max) | Peak load | Off after a trip | Enable (OBC pin) |
|---|---|---|---|---|---|---|
| SW10 OBC 3.3 V | TPS2553-Q1 U38 | 178 k | 0.152 (0.130–0.176) A | 0.040 A | 0.28 s, auto | always (no command); power-cycled by the EPS controller (Q42) after a 120 s OBC heartbeat timeout |
| SW9 EPS controller | TPS2553-Q1 U27 | ILIM to IN | 0.075 (0.050–0.100) A | 0.012 A | 0.28 s, auto | always |
| SW2 RADIO | TPS2553-Q1 U22 | 226 k | 0.120 (0.102–0.141) A | 0.038 A (RF board, SQSO-RF-BRD-001) | 2.0 s (≥ 1.26 s worst), auto | RADIO_EN PB5 |
| SW3 PA (TX-OFF) | TPS2553-Q1 U23 | 26.1 k | 0.989 (0.918–1.071) A | 0.485 A at 1 W RF (0.368 A at 0.5 W; RF board E) | 0.28 s, auto | RF_TX_EN PA3 (not shed from Version E) |
| SW4 ADCS sensors (OBC J6) | TPS2553-Q1 U24 | 88.7 k | 0.299 (0.265–0.339) A | 0.049 A (SQSO-ICX-001 §3.4) | 0.28 s, auto | ADCS_EN PB6 |
| SW7 GNSS | TPS2553-Q1 U25 | 133 k | 0.202 (0.175–0.232) A | 0.100 A | 2.0 s (≥ 1.26 s worst), auto | GNSS_EN PE5 |
| SW8 AntS | TPS2553-Q1 U26 | 18.7 k | 1.370 (1.287–1.465) A | 0.610 A | 0.28 s, auto | DEPLOY_EN PF3 OR the EPS controller's ANTS_EN (U81; ANTS_PWR register) |
| SW1 PAYLOAD | TPS1H100B-Q1 U20 | 1.24 k | 1.99 (1.71–2.27) A | 0.93 A | thermal cycling | PAYLOAD_EN PB2 AND SHED_N |
| U19 Jetson eFuse | TPS16630 | 15.0 k | 1.20 (1.10–1.30) A | 0.93 A (12 W guard) | latch-off, OBC re-enables | JETSON_PWR PA8 AND SHED_N |
| SW6 DEPLOY | TPS1H100B-Q1 U18 | 1.24 k | 1.99 (1.71–2.27) A | 1.25 A | thermal cycling | DEPLOY_EN PF3 |
| SW5 HEATER | TPS1H000-Q1 U21 | 3.24 k | 0.148 (0.126–0.170) A | 0.079 A | auto-retry | HEATER_EN PD2 |
| SW11 MTQ bridges (VBAT_MTQ, J3-36/39) | TPS1H000-Q1 U39 | 1.33 k | 0.361 (0.322–0.401) A | 0.23 A at the bridges' limit (0.104 A at 1.16 W, 11.2 V) | auto-retry 40 ms on / 1 s off | ADCS_EN PB6 |

TPS2553 rails: the current is limited within 2 µs; FAULT asserts after its 8 ms deglitch and pulls EN low (FAULT tied
to EN, R_FAULT 150 kΩ with C_RETRY 22 µF, or 100 kΩ with 4.7 µF), which removes the rail; EN recharges to its threshold in ≈ 0.6 × R × C
(2.0 s with R_FAULT 150 kΩ and 22 µF, ≥ 1.26 s with the X7R at −10 %, −15 % at −40 °C and DC bias; or 0.28 s, ≥ 0.18 s), and a persisting latch-up repeats the cycle. A BAT54 across R_FAULT makes a commanded turn-off fast
(< 1 ms). Trip levels against RADIATION_ANALYSIS §5.3: OBC 0.15 A (normal ≈ 30–40 mA, proposed 150 mA), radio 0.12 A
(38 mA measured-design load of the RF board; the 0.3 A first proposed could not see a latch-up of a 38 mA load; the
EPS controller also reports the rail's INA3221 current so the OBC can trip it in software), GNSS 0.20 A (0.1 A, proposed 0.2 A), all ≥ 100 ms off (≥ 1 s for radio and GNSS). **The OBC rail
SW10 is the power-cycling supervisor the OBC cannot be for itself**: it needs no command, and while it is off every OBC
output is low or high-impedance, so the EPS pull-downs turn every switched load off (the TX-OFF latch lives in the OBC's
FRAM and survives). Since Version B the coils no longer load SW4: SW11 (U39, TPS1H000-Q1 on VBAT, 10 k
series input, DIAG_EN tied to its own input so its 3 mA diagnostic standby current flows only while enabled) feeds the
OBC's three DRV8876 bridges on J3-36/39. Its limit, 0.32–0.40 A, meets SQSO-ICX-001 E3 (≥ 0.25 A minimum, the bridges'
0.23 A hardware limit at 11.2 V; ≤ 0.5 A maximum, the W3 28 AWG wires carry ≤ 0.20 A each); DELAY pulled up through
1 k selects auto-retry, so a short at the OBC draws 15 mA on average. SW4 now carries only the sensors (0.049 A), and the
OBC's own sensor-rail switches (50–100 mA trips, SQSO-ICX-001 §3.4) see a sensor latch-up (RID AV-03); SW4's 0.30 A
stays as the backstop. Every switch's fault state is read by the EPS controller (FLT_* lines) and reported in the
LOAD_SHED/fault register.

### 8.4 Hardware load shed (RID EPS-09)

| Item | Value | Requirement |
|---|---|---|
| Shed (falling) | 13.62 V | VBAT < 13.6 V |
| Release (rising) | 14.42 V | VBAT > 14.4 V |
| Delay before shedding | 4.9 s (4.75 M × 1.0 µF from 3.1 V to the 1.100 V threshold; 3.7–5.5 s with the X7R at ±10 % and −15 % at −40 °C) | 5 s |
| Release delay | < 1 ms | |

U48A compares VBAT (1.00 M / 221 k, 0.1 %; 4.12 M hysteresis) with REF_A; U48B times 5 s; SHED_N gates SW1 (U51) and the Jetson eFuse U19's
enable (U65, Version D) in hardware, independent of the OBC and the EPS controller: the shed holds with either of the
Jetson's two series switches failed closed (RID P3E-06). **The PA is not shed from Version E** (RID SMA-B05; U52
removed): a spurious permanent shed (comparator, reference or timer fault) would otherwise hold the transmitter off
with no override and end the mission, while a stuck-keyed PA is energy-positive (+0.20 W tumbling, SQSO-PBA-001 §8.4)
and has its own controls (the 120 s key-down timer, TX-OFF, the mode thresholds). EPS-F012 becomes "payload (Jetson)
only" (for the chair: a CR to EPS_DESIGN EPS-F012 and BASELINE). A spurious shed of the Jetson costs only the
experiment. The heater is not shed (EPS_DESIGN §6).

## 9. Deployment drivers (RIDs SMA-05, REL-03, EPS-18)

Four identical drivers, one per burn resistor (W1A, W1B on J10; W2A, W2B on J11), each:

1. **Burn line** BURN_x (OBC, 100 k pull-down) powers a **TPL5111 one-shot** (U70–U73) through 100 Ω: at the rising
   edge it starts and holds DRVn high for **12.0 s ± 0.6 s** (R_EXT 11.8 k + 182 Ω, 0.1 %), then low while the line
   stays high. Only a falling edge (which removes its supply) re-arms it: a stuck-high line burns once, for 12 s.
2. An AND gate (U74–U77, supplied from REF_A 2.500 V) gives 2.500 V only while the line is high and the one-shot runs;
   11.5 k / 1.00 k gives the 0.200 V set point.
3. OPA2991-Q1 (U78, U79, supplied from V_BURN) drives a BUK9M14-40E FET (Q24–Q27) with a 0.25 Ω sense resistor
   (RS20–RS23): **0.800 A ± 2.2 %** worst case (requirement ± 10 %), regulating for burn resistors up to 6.9 Ω.
4. Supply: VBAT (behind DS1–DS3) → **SW6** (DEPLOY_EN) → **U17 burn buck 7.04 V** → resistor → driver FET.

| Burn resistor (Ω) | V_BURN (V) | FET Vds (V) | FET loss (W) |
|---|---|---|---|
| 4.0 | 6.83 / 7.25 | 3.25 / 3.67 | 2.66 / 3.00 |
| 5.0 | 6.83 / 7.25 | 2.43 / 2.86 | 1.99 / 2.34 |
| 6.0 | 6.83 / 7.25 | 1.62 / 2.04 | 1.32 / 1.67 |
| 7.0 | 6.83 / 7.25 | 0.80 / 1.22 | 0.65 / 1.00 |

Energy into a 5 Ω resistor in the **10 s** burn (chair decision C-17, was 8 s): 32 J; 94 J (26 mWh) at the pack per resistor. The 12.0 s ± 0.6 s hardware one-shot leaves ≥ 1.4 s over the software's 10 s, and the thermal case of §16 already uses 12 s (U17 Tj 93 °C, FETs 92 °C); the pack fuses see 0.75 A for 10 s, far below their 5 A rating. Peak pack current: one resistor 0.51 A for the burn, **0.67 A**
with housekeeping and heater, 0.75 A with the magnetorquers at 1.16 W as well (0.90 A with the coil bridges at their
0.23 A hardware limit); two at once (an A/B-pair line arrangement, §13) 1.25 A; four at once 2.0 A (U17's limit).
Hinge sense: both wings' switches in series (J10-5/6, J11-5/6), 10 k pull-up, 1 k + 100 nF to SOLAR_SENSE (PB12): low =
both deployed. **AntS** (RID EPS-07): 3V3_ANTS from SW8 (DEPLOY_EN OR the EPS controller's ANTS_EN, Version E: a stuck-low DEPLOY_EN pin cannot prevent the antenna deployment; RID SMA-B06), downstream of DS1–DS3; the OBC's own I2C3 (J3-28 SCL, J3-40 SDA; Version E, SQSO-ICX-001 E14, no EPS pull-ups on that
side) through the TCA9517-Q1 U80 to both AntS controllers (side A 0x31; side B 0x32 on J12-7/8 through two 0 Ω jumpers
if the variant brings it out separately, E15: the ISIS ICD to confirm; otherwise the jumpers stay open), whose ports are high-impedance while the AntS is unpowered; the AntS's ARM command and thermal knives complete its
chain (DS2, DS1, DS3, SW8, AntS ARM). Which AntS internal switches are physical devices between its supply and its knives
is still to be read from the AntS ICD delivered with the unit (open, RFQ item).

## 10. Battery heater

SW5 TPS1H000-Q1 (HEATER_EN, 0.148 A limit) → F4 0.25 A (32 % at 16.4 V) → RS11 (INA3221) → J1-7 → 207 Ω film → J1-8 →
Q22 (opened by U54A above +40.0 °C on NTC1) → GND. Heater 58–79 mA, 0.70–1.30 W at 12.0–16.4 V. Three independent
limits on a fault: the switch's current limit, the fuse (switch failed short), the hardware 40 °C cutout in series
(software stuck on); and the AFE's 60 °C trip behind them (SQSO-BSA-001 H6).

## 11. Telemetry

| Signal | Source | Scale / address | Consumer |
|---|---|---|---|
| VBATT_SNS (PC2) | PACK_P, 1.00 M / 200 k (0.1 %), 0.1 µF | × 6.0 (2.80 V at 16.8 V) | OBC ADC; adc.rs `Linear(6.0, 0)` (finding F1) |
| IBATT_SNS (PC3) | U30 INA186A2 × 50 on RS6, REF 1.65 V (U49) | 0.5 V/A, 1.65 V at 0 A, ± 3.3 A | OBC ADC; adc.rs unchanged |
| VSOLAR_SNS (PC4) | highest string (VCTL_IN), 300 k / 100 k | × 4.0 | OBC ADC; unchanged |
| ISOLAR_SNS (PC5) | U31 INA186A2 × 50 on RS5 (array output) | 1.0 V/A | OBC ADC; current at the bus voltage (power = VBATT × ISOLAR) |
| VBUS_SNS (PB0) | 5V_PA, 100 k / 100 k | × 2.0 | OBC ADC; unchanged |
| VJET_RB (PA7, J3-19) | VBAT_JET after the eFuse U19, 1.00 M / 200 k (0.1 %), 0.1 µF | × 6.0 (1.87–2.80 V on, < 0.1 V off) | OBC ADC, new channel (RID AV-06) |
| VSW1_RB (PA1, J3-37) | VBAT_SW1 after SW1, 1.00 M / 200 k (0.1 %), 0.1 µF | × 6.0 | OBC ADC, new channel (RID AV-06) |
| Pack gauge | U28 INA228-Q1 on RS6 (V, I, charge, energy, die temperature) | I2C2 0x44 | OBC (direct, independent of the controller) |
| Array output | U29 INA228-Q1 on RS5 | I2C2 0x40 | OBC (direct) |
| EPS register block | U37 MSP430FR2355 | I2C2 0x48 (ICD §4.2, §4.6) | OBC |
| Cell voltages, AFE status, PF, CC | U1 BQ76952 via U50 | local bus 0x08 | EPS controller |
| Tracker inputs ×4, bus load, 3V3_AO, 5V_PA, radio, ADCS, GNSS, AntS, heater | U32–U35 INA3221-Q1 (10–500 mΩ shunts) | local bus 0x40–0x43 | EPS controller |
| Panel NTCs ×6, NTC1, VHW | U36 ADS7138-Q1 | local bus 0x10 | EPS controller |
| Burn currents ×4, Jetson IMON, F1 intact, V_BURN, VTRK | MSP430 ADC (A0, A4, A6, A7, A8–A11) | — | EPS controller |
| PERMIT, WIN_OK, SHED_N, EOL armed, NTC1_BAD (P4.3), all switch faults (incl. FLT_SW11 on P4.5), AFE ALERT | MSP430 GPIO (100 k series) | — | EPS controller |
| EPS_ALERT_N (PB7) | open drain, INA228s and controller | 4.7 k pull-up | OBC |

I2C2 (OBC bus) carries 0x40 and 0x44 (INA228) and 0x48 (EPS controller, incl. the OBC heartbeat register); the AntS is on the OBC's I2C3 through U80 (Version E); the OBC has no device of its own on this bus (its board temperature is an analog TMP235-Q1,
ICD-001 §2.3). The AFE (0x08), INA3221s and ADS7138 are on the controller's local bus. The ICD's
"battery monitor 0x55" and "solar monitor 0x40" become the INA228s at 0x44 and 0x40 (ICD change, §13).

## 12. EPS controller (U37)

MSP430FR2355TPTR (FRAM, −40…105 °C) on its own latch-up-tripping supply SW9. Firmware (a new configuration item, for the
chair: not written; Class C, NPR 7150.2D) implements the ICD register block (0x00–0x33 read by `sqso_fsw::eps_telemetry`
and the §4.6 extended registers), the MPPT perturb-and-observe on the four SAC DACs, SOC from the INA228, the lockout
SET backup and the UNLOCK step, the **EOL key** (Version D: on the two-write sequence to the EOL_KEY register, first
0x3C then 0xC3 within 1 s, accepted only while EOL_STATE shows not armed and F1 intact, it raises EOL_KEY (P4.2) for
10 s and reports it in EOL_STATE bit 3; any other write or a timeout drops it), **OBC heartbeat supervision**
(Version E, RID SMA-B09, SQSO-ICX-001 E14: the OBC writes 0x5A to OBC_HEARTBEAT 0x8C at least every 60 s; after
600 s without one, and at least 600 s after the controller's own start or the last cycle, the controller raises
OBC_CYCLE, whose TPL5111 one-shot U82 (powered by the line, tIP 2.0 s) pulls SW10's EN low through Q42 for 1.95 s
once per rising edge, so a controller stuck with the line high cannot hold the OBC off; at most 3 cycles in 24 h, then
it stops and reports it, and 10 consecutive heartbeats reset the count), the **AntS power line** ANTS_EN (P6.0, ANTS_PWR 0x8D), the **per-tracker disables** TDIS_T1–T4
(P6.2–P6.5, TRK_DISABLE 0x8E, RID SMA-B17: a tracker with a shorted output diode is switched off; its string's
ideal diode blocks the back-feed), the supervisor status (EPS_SUPV 0x8F) and the fault and status flags.
**Passivation depends on it** (Version D on, SQSO-FMEA-003): arming F1 needs its EOL_KEY, so a failed controller
blocks the end-of-life disconnect (it cannot fire it); the controller firmware is Category 3 software for that
function (SQSO-FMEA-003). **No battery protection depends on it**: if it stops, the DACs
go high-impedance (MPPT falls back to 4.40 V per string), its lines fall to their pull-downs (no lockout set or
clear, no EOL arm, trackers enabled, no OBC power cycle, AntS on DEPLOY_EN alone), and the OBC still has the analog lines and the two INA228s. Programming through J22 (Spy-Bi-Wire), ground only.

## 13. Interfaces

### 13.1 Connectors

Every harness connector is on the **bottom side** of the board, in the zones of SQSO-HAR-001 2.0 §10.1 (bottom-side
envelope 9.0 mm, z −3.4 … −12.4 with the 2.0 mm board; the top side has 5 mm under the OBC); all headers are gold (EEE_PARTS_LIST §2A.3 item 5:
no tin contacts). Pico-Clasp headers are Molex **203556** (0.38 µm Au mating, mates the 501330 housings with gold
501334 terminals, 28 AWG); Micro-Fit headers are **43045 right-angle** (0.76 µm Au mating) so the mated plug and its
wire bend stay within 9.0 mm.

| Ref | Part | Function | Harness | Zone (SQSO-HAR-001 §10.1) |
|-----|------|----------|---------|---------------------------|
| J1 | Micro-Fit 3.0 2×4 R/A, 0430450802 | BAT_TOP ×2, BAT_BOT ×2, NTC1, NTC1 return, heater ± | W1 | Z1 (PACK corner, −X +Y) |
| J2 | Pico-Clasp 6, 2035560607 | TAP0–TAP4, NTC2 | W1 | Z1 |
| J3 | Samtec TFM-120-02-S-D-A (40, vertical SMT) | OBC, per SQSO-ICX-001 §3.1 | W3 (Samtec SFSDT 30 AWG; §13.2) | +X edge, \|y\| < 15.5 mm |
| J4–J7 | Pico-Clasp 5, 2035560507 | body strings PX, MX, PY, MY (2 + 2 contacts, TEMP) | W2PX, W2MX, W2PY, W2MY (28 AWG, two wires per pole) | J4 Z4, J5 Z2, J6 Z1, J7 Z2 |
| J8, J9 | Pico-Clasp 8, 2035560807 | wings W1, W2 (front ×2, back ×2, return ×3, TEMP) | W2W1, W2W2 | J8 Z3, J9 Z5 |
| J10, J11 | Micro-Fit 3.0 2×3 R/A, 0430450602 | burn resistors A and B, hinge switch | W7A, W7B | J10 Z3 (plug toward +X), J11 Z2 |
| J12 | Pico-Clasp 8, 2035560807 (Version E) | 1, 2 3V3_ANTS; 3, 4 GND; 5 SDA (side A); 6 SCL (side A); 7 SDA_B, 8 SCL_B (AntS side B 0x32, through 0 Ω jumpers to the same buffered bus, fitted only if the variant brings side B out separately) | W7C (+2 wires if side B is separate: for the harness designer) | Z5 |
| J13 | Micro-Fit 3.0 2×3 R/A, 0430450602 | umbilical: charge ×2, return ×2, pack monitor, AFE shutdown | W11 (to the access-port board A3) | Z5 |
| J14 | Pico-Clasp 4, 2035560407 | RBF (two poles) | W12D | Z4, −Y edge |
| J15, J17 | Pico-Clasp 3, 2035560307 | DS1, DS3 (C, NC, NO; J17-3 unused since SMA-B13) | W12A, W12C (one per switch, each on its own rail) | J15 Z3, J17 Z4 |
| J16 | Pico-Clasp 5, 2035560507 | DS2 two-pole: pole 1 C1, NC1, NO1 (spare); pole 2 C2 (VCTL through 10 k) and NO2 (TCL_A_W to the Q40 clamp gate, active high) | W12B | Z2 |
| J18 | Micro-Fit 3.0 2×2 R/A, 0430450402 | VBAT_JET ×2, GND ×2 | W5A (24 AWG) | Z2 |
| J19 | Pico-Clasp 6, 2035560607 | pinout of RF J2: 1 3V3_RADIO, 2 GND, 3, 4 5V_PA_SW, 5, 6 GND | W4B (28 AWG pairs) | Z2 |
| J20, J21 | Pico-Clasp 2, 2035560207 | ADCS sensor 3.3 V, GNSS 3.3 V | W6B to OBC J6; W9B to OBC J4 (SQSO-ICX-001 B §2.1); the OBC's GNSS translators take their receiver side from J21's rail | J20 Z1 (−X edge, +Y side); J21 Z3 (**+X edge, +Y side**) |
| J22 | FTSH 2×2 | EPS controller programming | GSE (ground use only) | either side, clear of the zones |

### 13.2 Control and status lines from the OBC (for the chair and the OBC designer)

All 3.3 V CMOS. Outputs from the OBC are active high and must be low at OBC reset; the EPS holds each with 100 kΩ to
GND. OBC pins as allocated in SQSO-ICX-001 §3.1 and §4 (Version B; where they differ from today's
`software/rust/firmware/src/gpio.rs`, SQSO-ICX-001 §6 lists the change for the flight software).

| J3 pin | Signal | Direction (OBC) | Level / default | OBC pin | Function |
|--------|--------|-----------------|-----------------|---------|----------|
| 1, 2 | 3V3_OBC | power in | 3.3 V, 0.15 A trip | — | OBC supply through SW10 (latch-up retry) |
| 5, 6 | I2C2_SCL, I2C2_SDA | bidir | open drain, pull-ups on the OBC | PF1, PF0 | EPS controller 0x48 (incl. OBC_HEARTBEAT 0x8C, written every ≤ 60 s), INA228 0x40/0x44 |
| 28, 40 | I2C3_SCL, I2C3_SDA | bidir | open drain, two 4.7 kΩ in parallel on the OBC, none on the EPS; 100 kHz | PG7, PB4 | U80 A side → AntS sides A 0x31 and B 0x32 (Version E, SQSO-ICX-001 E14) |
| 8 | EPS_ALERT_N | in | open drain, active low, 4.7 k on the EPS | PB7 | INA228 alerts, controller interrupt |
| 9 | PAYLOAD_EN | out | high = on, low at reset | **PB2** (PB4 has a JTAG pull-up at reset) | SW1 (Jetson), shed-gated |
| 10 | RADIO_EN | out | high = on, low at reset | PB5 | SW2 receiver/TNC/exciter (the flight software raises it at boot, 97.207(b)) |
| 11 | ADCS_EN | out | high = on | PB6 | SW4 (sensors) and SW11 (VBAT_MTQ) |
| 12 | HEATER_EN | out | high = on | PD2 | SW5 |
| 13 | DEPLOY_EN | out | high = on | **PF3** | SW6 (burn buck) and SW8 (AntS) |
| 14 | RF_TX_EN | out | high = PA powered; low = TX-OFF | PA3 | SW3, shed-gated |
| 15 | GNSS_EN | out | high = on | PE5 | SW7 |
| 16 | DEPLOY_SENSE | in | high = DS2 released (1 MΩ source, clamped) | PE2 | separation state |
| 17 | BURN_W2A | out | high = burn, rising edge starts 12 s | PB9 (was ANTENNA_BURN, freed by the AntS) | wing 2 resistor A |
| 18 | BURN_W1A | out | as above | PB10 (SOLAR_BURN, driver A) | wing 1 resistor A |
| 19 | VJET_RB | analog in | VBAT_JET / 6 | **PA7** (ADC1_IN12) | Jetson eFuse output readback (RID AV-06) |
| 20 | SOLAR_SENSE | in | low = both wings deployed | PB12 | hinge switches |
| 22–26 | VBATT_SNS, IBATT_SNS, VSOLAR_SNS, ISOLAR_SNS, VBUS_SNS | analog in | 0–3.3 V, 100 nF / 10 nF filtered | PC2, PC3, PC4, PC5, PB0 | §11 |
| 29 | JETSON_PWR | out | high = on | **PA8** (SQSO-ICX-001 B; PA2 is the carrier's spreader thermistor) | U19 eFuse (SHDN), shed-gated (U65) |
| 30 | BURN_W1B | out | high = burn | PE6 (SOLAR_BURN_B, driver B) | wing 1 resistor B |
| 31 | BURN_W2B | out | high = burn | **PE7** | wing 2 resistor B |
| 32 | EOL_ARM | out | rising edge sets the arm latch while the EPS controller's EOL_KEY is high, low at reset | **PF4** | passivation arm (with PASSIVATE_ARM and the keyed I2C sequence) |
| 33 | EOL_FIRE | out | high = fire (hold ≥ 60 s), low at reset | **PG1** | passivation fire (PASSIVATE_EXECUTE) |
| 34 | LOCK_CLR | out | high pulse ≥ 100 ms with the controller's UNLOCK | **PF5** | recharge-lockout clear |
| 36, 39 | VBAT_MTQ | power out (EPS) | VBAT 11.2–16.4 V, ≤ 0.23 A, limit 0.32–0.40 A | OBC DRV8876 VM | SW11 on ADCS_EN |
| 37 | VSW1_RB | analog in | VBAT_SW1 / 6 | **PA1** (ADC1_IN6) | SW1 output readback (RID AV-06) |
| 38 | 3V3_RTC | power out (EPS) | 3V3_AO through 1.0 kΩ, ≤ 2 µA, always on | OBC RV-3028-C7 VDD | mission clock supply (RID AV-10) |
| 3, 4, 7, 35; 21, 27 | GND; AGND (analog return of pins 19, 22–26, 37) | — | | | AGND joins GND at the board's analog star point (§17); PB11 (ANTENNA_SENSE) is no longer used |

**Burn-line mapping (for the chair).** The avionics reviewer assigned driver A to PB10 and driver B to PE6, with driver
A burning the first resistor of both wings together (`sqso-sil/src/hw/deploy.rs`). This board has four independent
inputs, so it supports both: (a) **recommended and adopted in SQSO-ICX-001**, four lines, one resistor at a time
(chair decision, 0.67 A peak): PB10 → W1A, PE6 → W1B, PB9 → W2A, PE7 → W2B, and the flight software drives each
separately; (b) the A/B pairing,
by joining J3-17 to J3-18 and J3-30 to J3-31 on the OBC side: two resistors burn at once, peak pack current 1.25 A (above
the chair's 1.0 A, below the 4 A BMS trip), and one driver line again serves both wings (the REL-03 common cause returns
for the line, not for the drivers).

## 14. Parts

Every IC's orderable part number and status were read from the manufacturer's datasheet package addendum (TI parts,
2026-10-27, all "Active"); non-TI parts from Digi-Key listings (status "Active" unless noted). The BOM column "MPN check"
says which. Passive MPNs are built from the manufacturer's series code (Panasonic ERJ-3EKF/ERA-3AEB, Vishay CRCW/WSLP/WSL,
Murata GCM): the series were checked, the individual values not. Corrected in Version F after SQSO-MFG-003: 1N4148WQ-7-F, ERJ-3EKF10R0V (10 Ω), ERA-6AEB105V and ERA-6AEB3653V (0805: ERA-3AEB covers 47 Ω – 330 kΩ), ERA codes of E24 values in three digits (ERA-3AEB103V, 102V, 153V, 204V, 6AEB105V), WSL2010R5000FEA for RS11, TFM-120-02-L-D-A (plating -L, 10 µin gold). Not checked at all: 2N7002BK, BAT54/BAT54C/BAT54S,
BZX84-C8V2, 1N4148W-Q, 0466001.NR, NCP15XH103F03RC, FTSH-102, Keystone 5015, the Pico-Clasp 203556 headers other than
the 6- and 8-way (part numbers from the series pattern), the Micro-Fit 0430450402 finish, and the Samtec TFM-120-02-S-D-A
code (lead style, plating and latch option from the series page, to confirm with Samtec; V-HAR-3). Checked on Digi-Key:
0430450802 and 0430450602 (right-angle, 0.76 µm Au, Active), 2035560607 (0.38 µm Au, Active). Radiation: no published TID data was found for any of these parts; all are
covered by RAD-V3 lot-sample testing and by the latch-up trips of §8.3 (EEE_PARTS_LIST §2A.4).

| Function | Selected | Grade | Notes |
|----------|----------|-------|-------|
| AFE | BQ76952PFBR | industrial −40…85 °C | the candidate BQ76920 drives low-side FETs only and has no OTP permanent-fail detection |
| Secondary OVP | BQ771806DPJR | −40…85 °C (addendum) | 4.350 V option exists only as a 3 s delay |
| Trackers | TPS55340QRTERQ1 | AEC-Q100 G1 | non-synchronous by choice (§4.2) |
| Ideal diodes | LM74700QDDFRQ1 + BUK7M12-40EX | AEC-Q100/Q101 | |
| Bucks | LMR43620MC3RPERQ1, LMR43620MC5RPERQ1 | AEC-Q100 G1 | matte tin: coat |
| High-side switches | TPS1H100BQPWPRQ1, TPS1H000AQDGNRQ1 | AEC-Q100 G1 | |
| eFuse | TPS16630RGER | industrial | 60 V; latch-off |
| 3.3/5 V switches | TPS2553QDBVRQ1 | AEC-Q100 G1 | |
| Monitors | INA228AQDGSRQ1, INA3221AQRGVRQ1, INA186A2/A3QDCKRQ1, ADS7138QRTERQ1 | AEC-Q100 | INA228 matte tin |
| Controller | MSP430FR2355TPTR | −40…105 °C | not AEC-Q100 |
| Connectors | Pico-Clasp 203556 (gold), Micro-Fit 43045 right-angle (gold), Samtec TFM-120 vertical (gold) | −40…105 °C | all harness connectors on the bottom side (§13.1); tin-plated contacts not allowed (EEE_PARTS_LIST §2A.3 item 5) |
| Fuses | FM12A72V5.0A (F2; also F0A and F0N on the battery sense board; MIL-PRF-23419/12), FM08A125V5A (F0B, sense board), ITV4030L1412NR (F1 here; F3 on the sense board from Version F), 0466.250NR (F4) | −55…+125 °C (MIL-PRF-23419) | **ITV4030 operating temperature −10…+65 °C**: below the board's −30 °C AFT; qualification item V-EPS-8 |
| PTC | 2920L260/33DR | −40…85 °C | |
| Relay | TQ2SA-L2-3V-Z | −40…85 °C | gold-clad contacts; pin numbering to be checked against Panasonic's 2-coil drawing when the symbol is entered in KiCad |
| Power FETs | BUK7Y3R5-40HX, BUK9Y4R4-40E, IPD50P04P4L11ATMA2, BUK9M14-40EX | AEC-Q101 | |
| Oscillator | SG-8018CA 4.0000M-TJHPA0 | −40…105 °C | |

## 15. Stress summary (derated)

| Part | Stress | Limit | Status |
|------|--------|-------|--------|
| BQ771806 VDD (sense board) | 17.4 V / 25 V = 70 % | 80 % | pass |
| Q6/Q7 Vgs | 7.3 V / 10 V = 73 % (1.3 M / 1.0 M at 16.8 V) | 75 % | pass; 4.9 V at 11.2 V, where the logic-level FET is specified |
| Q16 Vgs (zener-clamped) | 8.2 V / 12 V = 68 % | 75 % | pass |
| Q1/Q2 Vds, Vgs | 17.4/40 = 44 %; 11/20 = 55 % | 75 % | pass |
| Q4/Q5 Vds, Vgs | 17.4/40 = 44 %; 8.4/16 = 53 % | 75 % | pass |
| Tracker SW node | 17.6 V / 40 V = 44 % | 80 % | pass |
| D9–D12 reverse | 17.5 V / 40 V = 44 %; 0.35 A / 3 A | 70 %, 50 % | pass |
| Bucks VIN | 16.8 / 42 = 40 % | 80 % | pass |
| INA3221 heater channel CM | 16.8 / 26 = 65 % | 80 % | pass |
| VBAT ceramics 50 V | 16.8 / 50 = 34 % | 50 % | pass |
| 22 µF 16 V on V_BURN | 7.25 / 16 = 45 % | 50 % | pass |
| F2 / RT1 / F3 / F4 / F5 | 31 % / 73 % (of 60 °C hold) / 14 % / 32 % / 50 % | 50 % (fuses) | pass; RT1 is a PTC (hold-current rule, not fuse rule) |
| R83 precharge 200 Ω 1 W (reference regenerated in Version F) | 0.38 W at the worst precharge (16.7 V bus, 8.0 V pack) | 50 % | pass |
| RS20–RS23 burn shunts | 0.19 W / 0.5 W = 39 % | 50 % | pass |
| Connectors | ≤ 55 % of contact rating (strings, two contacts per pole) | 50 % | J4–J9 at 37 % of the 28 AWG bundle-derated wire rating (SQSO-HAR-001 §9.1), 28 % of the contact rating |

## 16. Thermal (for the thermal model)

Board at +60 °C (upper operating AFT, THERMAL_DESIGN 1.3 §1.1); vacuum, so heat leaves each part only into the board
(Ψ_JB or θ_JB plus a 12–20 K/W local spreading term for a 6-layer board with 2 oz outer copper and thermal vias);
transients use 0.6 of the spreading term at 8–60 s.

| Part | Steady worst (mW) | Transient | Tj worst (°C) |
|---|---|---|---|
| U4 tracker T1 (TPS55340-Q1) | 209 | | 66 |
| D9 T1 output Schottky (PMEG40T30ER-Q) | 121 | | 66 |
| L1 T1 inductor | 60 | | 62 |
| U15 3V3_AO buck | 60 | | 62 |
| U16 5V_PA buck at 1 W RF | 150 | | 66 |
| U17 burn buck, 2 drivers on (fault) | 0 | 0.95 W for 12 s | 93 |
| Q24–Q27 burn driver FET, 4.0 Ω resistor | 0 | 3.00 W for 12 s | 92 |
| RS20–RS23 burn shunt | 0 | 0.19 W for 12 s | 68 |
| U20 SW1 Jetson switch at 0.93 A | 173 | | 67 |
| U19 Jetson eFuse at 0.93 A | 43 | | 61 |
| Q1+Q2 AFE FETs at 1.3 A | 20 | | 60 |
| Q4+Q5 DS1/DS3 FETs at 1.3 A | 54 | | 61 |
| Q6+Q7 DS2 FETs at 1.3 A | 20 | | 60 |
| Shunts at 1.3 A | 39 | | 62 |
| U1 BQ76952 balancing one cell | 42 | | 62 |
| U21 heater switch | 10 | | 60 |
| U39 SW11 magnetorquer switch at the bridges' 0.23 A limit | 85 | | 64 |
| Q16 F3 heater FET (one event) | 0 | 0.16 W for 60 s | 71 |
| Q20+Q21 EOL fire FETs (one event) | 0 | 0.14 W for 60 s | 70 |
| U37 controller, monitors, logic | 40 | | 62 |
| Y1 oscillator and divider | 12 | | 61 |
| Ideal diodes at Isc (8) | 88 | | 65 |

**Board totals for SQSO-THA-001:** 1.33 W sunlit ACTIVE at maximum array power with Jetson, PA and magnetorquers
(1.16 W) on (worst steady); 0.37 W in eclipse; 3.77 W for ≤ 12 s in a burn (one per wing line, ≤ 8 in LEOP); orbit
average ≈ 0.49 W in ACTIVE sun-pointed (Version A: 1.29 / 0.35 / 3.75 / 0.48 W; SW11 and the extra bus current add
0.04 W). Hot spots to place on the board's conductive path to the frame: the four trackers with their diodes and
inductors (≈ 0.4 W each at the worst), the burn FETs (transient), U16 and U20. All junctions stay below the 110 °C / 125 °C
derated limits with ≥ 17 K margin at +60 °C board (U17 93 °C against 110 °C).

## 17. Layout guidance (layout is not in scope; these are the constraints to start from)

| Item | Requirement |
|------|-------------|
| Outline and mounting | PC/104 90.17 × 95.89 mm, four Ø3.18 mm holes at (±40, ±43) with Ø6.4 mm copper-free rings, and the **two centre supports at (0, ±21) mm** (chair decision C-12, MECHANICAL_DESIGN D-7 and §11 item 5) with Ø7 mm copper-free rings and Ø7 mm kept free of parts on both sides; all isolated from GND except the GND-bonded mounting hole H1 (the single frame bond of the stack, star with AGND at U37); the **Z-rod notch** at x < −35.5 mm, \|y\| < 6.5 mm (the Z rod at (−41.5, 0) passes the board; MECHANICAL_DESIGN §3), cut out of the outline with a 1 mm copper and part keep-out around its edge; the frame vendor's stack drawing governs (BUILD_VS_BUY §2.2) |
| Bottom side over the X rod | bottom-side parts **≤ 0.6 mm** high over the X rod's footprint (x −24.5 … +24.5, y +29.75 … +42.25; 1.0 mm clearance, D-14): 0402/0603 passives only, no ICs or connectors |
| Connector zones (SQSO-HAR-001 2.0 §10.1, moved 0.4 mm down with the 2.0 mm board) | all harness connectors on the **bottom side** in zones Z1–Z5 and the +X edge per §13.1; bottom-side envelope 9.0 mm (z −3.4 … −12.4 with the 2.0 mm board) in the zones; footprints within the harness design's allocations (≤ 80 % of each zone's area: Z1 69 %, Z2 73 %, Z3 67 %, Z4 27 %, Z5 58 %); J3 at the +X edge within \|y\| < 15.5 mm; J21 on the +X edge, +Y side (W9B, (+X, +Y) bundle); J20 on the −X edge, +Y side; J10's plug toward +X |
| Stack-up | 6 layers, **2.0 mm** (D-14, MECHANICAL_DESIGN: Steinberg margin of the coupled stack; bottom face at z −3.4, top unchanged), no controlled impedance (no RF on the board; SYNC, I2C and the SBW lines are short and not impedance-controlled), copper 2 / 1 / 1 / 1 / 1 / 2 oz with ≈ 0.33 mm dielectrics, FR-4 Tg ≥ 170 °C (polyimide preferred for TVAC): L1 parts/power (2 oz), L2 GND, L3 power (PACK_P, VBAT, VTRK islands), L4 signals, L5 GND, L6 parts/power (2 oz); IPC-6012 Class 3/A |
| Domains | PACK domain (U1, Q1–Q3, Q6/Q7, F2/RT1, RS6/RS7, J1/J2 area; the BQ771806 and F3 are on the battery sense board from Version F) in the **(−X, +Y) corner, zone Z1** (the W1 route), on its own BAT_BOT copper, joined to GND only through Q6/Q7; U50 straddles the boundary; ≥ 1.0 mm gap between BAT_BOT and GND copper |
| Power paths | pack line (J1 → F2 → RT1 → RS6 → Q1/Q2 → PACK_P → Q4 → Q5 → RS8): ≥ 3 mm/A on 2 oz outer layers, ≥ 2 A design current (5 A fuse); Kelvin sense to RS5–RS8 and RS20–RS23 |
| Converters | each tracker and buck as a tight loop (input cap, switch, diode, output cap) on L1 over solid L2; inductors and switch nodes on the side away from the antennas (rule E2); SYNC traces short, series 100 Ω at the source; no converter within 10 mm of U37's ADC pins or U53/U54; where the layout allows, keep the tracker inductors and switch nodes out of the board area over the bottom-side zones Z1–Z3, which carry the unshielded W6D sun-sensor branches (SQSO-HAR-001 E-9) |
| Thermal | thermal vias (≥ 9 × 0.3 mm) under U4–U7, U15–U17, U19, U20, Q24–Q27 into L2/L5; the burn FETs and U17 near a mounting hole; ≥ 2 cm² copper per burn FET |
| Creepage / clearance | ≤ 30 V everywhere: IPC-2221B B4 (coated) ≥ 0.13 mm; use ≥ 0.3 mm between PACK domain and GND copper and around F1's heater pins (fuse opening heat; F3 is on the sense board) |
| Keep-outs | 5 mm around Y1 and the clock dividers from switch nodes; no copper under F1 (heat to the fuse element per Littelfuse land pattern; F3's thermal island on the sense board, BATTERY_DESIGN §7.2); relay K1 away from inductors (stray field) |
| Test points | TP1 TRK_EN, TP2 VBAT, TP3 PACK_P, TP4 GND, TP5 3V3_AO, TP6 5V_PA, TP7 SHED_N; add at layout: VCTL, VHW, REF_C, REF_A, SYNC_A/B, V_BURN, each tracker COMP, CTRL_DN, PERMIT, CHG_INHIBIT, EOL_ARMED, BXP_W1A–W2B |
| Coating | conformal coat (tin-whisker mitigation, EEE_PARTS_LIST §2A.3), keep J1–J22 mating faces (gold), K1 and the test points clear |
| RF | no RF on this board; the PA supply leaves on J19 through RS13 with a 1 µF at the switch and the PA board's own filter |

## 18. Fault tolerance summary (inputs to SQSO-FMEA-002 and SQSO-BSA-001)

| Hazard (BSA) | Controls on this board | Count |
|--------------|-----------------------|-------|
| H1 overcharge | CV loop U45A; AFE COV 4.25 V (OTP); BQ771806 + F3 (fixed threshold) | 3 (2-FT) |
| H2 over-discharge then recharge | FSW mode thresholds; AFE CUV 2.80 V; latched lockout K1 (two actuators, two-signal clear); hardware load shed U48 | 4 |
| H3 external short | positive-leg fuses F0A/F0B/F2; negative-leg fuse F0N; AFE SCD/OCD (RT1 not counted) | 3 (2-FT) downstream of the AFE FETs |
| H3b short in W1 / taps | F0A, F0B, F0N (209 A ≤ 1000 A); taps: two 204 Ω chains + tap fuses | 3 (2-FT: chair decision, F0A/F0B from two lots counted as independent; NRCSD concurrence for the ISS path, SQSO-BSA-001 D) |
| H5 cold charging | NTC1 comparators → CC set point 0 and CFETOFF (with the NTC1 plausibility hand-over to NTC2); AFE UTC on NTC2; FSW; heater | 3+ |
| H6 over-temperature | FSW heater control; 40 °C cutout Q22; SW5 limit + F4; AFE OTD and the 60 °C comparator BOTHOFF | 4 |
| Jetson stuck on (EPS-F012, FMEA note) | load shed on both series switches' enables (U51 → SW1, U65 → U19); the PA is not shed (SMA-B05) | covers either switch failed closed, a stuck enable, software never turning it off; **residual**: both power stages failed closed (two part failures), not shed |
| H8a burn over-current | 0.8 A per driver; one line at a time; 12 s one-shot; U17 limit 2.0 A at the pack; BMS 4 A | 5 |
| H9 stored energy after EOL | F1 (arm: OBC EOL_ARM AND EPS-controller key; fire: OBC EOL_FIRE) | 1 (+ the loads drain the pack) |
| RF / deployables at launch | DS1, DS2, DS3 on the pack; DS1, DS3 and DS2 pole 2 (Q40 on TRK_EN, active high) on the array (+ SW3/SW6/SW8 and the OBC timers) | 3 physical on each source |

## 19. Verification items (this board)

| ID | Item | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-EPS-1 | Tracker efficiency 3.5–6.3 V, 0.05–1.1 A, −20/+25/+60 °C, synchronised | ≥ 92 % above 2 W, ≥ 90 % above 1 W | EPS-F002, §4.3 | EPS EM |
| V-EPS-2 | AFE OTP programming and readback; trip of every threshold (COV, CUV, OCD, SCD, UTC/OTC/OTD, SUV PF) on a pack simulator | within ± 25 mV, ± 10 % current, ± 2 K | EPS-F011, BSA §4 step 9 | EPS EM |
| V-EPS-3 | Inhibits: with the array simulator lit and the pack connected, each combination of two of DS1, DS2, DS3 jumpered closed (failed) with the third actuated, and the RBF removed; bus, PA supply, V_BURN, 3V3_ANTS, TRK_EN and every rail measured | bus ≤ 6.5 V and no converter running, no tracker switching, no RF, no burn current in all three cases; chatter of 5 ms bursts does not raise the bus | EPS-F009, CDS 2.3.7/2.3.8, P3E-01 | EPS EM, FM (with V-BSA-6) |
| V-EPS-4 | GND–PACK_N resistance with DS2 open, board powered from the array simulator | ≥ 100 kΩ | §6.4 | EPS EM |
| V-EPS-5 | Storage drain, switches actuated, AFE in SHUTDOWN, 25 and 60 °C | ≤ 20 µA (design 13.8 µA, with A3 fitted and both tap ladders) | LVR-071 | EPS EM, FM |
| V-EPS-6 | Burn drivers: 4, 5, 6, 7 Ω loads; stuck-high line; four lines at once | 0.80 A ± 10 %; stops at 12.0 s ± 1 s; pack ≤ 2.2 A | SMA-05 | EPS EM |
| V-EPS-7 | Load shed: VBAT ramp and step | shed 13.6 ± 0.2 V after 4–6 s at +25 °C (3.7–5.5 s over −40 … +70 °C); release 14.4 ± 0.2 V | EPS-F012 | EPS EM |
| V-EPS-8 | ITV4030 flight-lot qualification (F1 and F3; RID MFG-003 item 1), 6 samples per test: (a) carry 2 A for 1 h at −40 °C and at +70 °C, then 5 thermal cycles −40 / +70 °C carrying 1.5 A; (b) heater trigger at 10.5 V and 16.8 V at 0 °C, +25 °C and +65 °C (the temperatures at which F1 and F3 are fired); (c) heater trigger at −40 °C at 16.8 V as a characterisation; (d) EOL fire on the EM (destructive) | (a) no opening, fuse-element resistance change ≤ 10 %, no visible damage; (b) opens < 1 min; (c) recorded (not a pass criterion: the design never fires below 0 °C); F1 opens only with both EOL lines | EPS-F010, V-BSA-7, SQSO-MFG-003 | EM (before the FM build) |
| V-EPS-9 | Lockout: set by an emulated SUV PF and by LOCK_SET; clear needs both lines; shock 750 m/s² in both states | state kept through power loss and shock | SMA-06 | EM |
| V-EPS-10 | Latch-up trips: 1.5 × trip step on each switched rail incl. SW10 | rail off ≤ 10 ms, back after 0.2–0.4 s or 1–2 s | AV-08, RAD-V5 | EM |
| V-EPS-13 | SW11 and readbacks: VBAT_MTQ short and 0.3–0.5 A steps; VJET_RB and VSW1_RB with SW1 and U19 switched in turn | limit 0.32–0.40 A, retry 40 ms / 1 s; readback ratio 6.00 ± 0.5 %, < 0.1 V off | E1–E3, AV-06 | EM (with V-ICX-1) |
| V-EPS-14 | NTC1 plausibility: NTC1 opened, shorted and its wire grounded, at 10 °C and 40 °C pack (decade box) | NTC1_BAD set; charging continues inside the AFE's UTC/OTC on NTC2; no BOTHOFF; heater runs under SW5's limit | SMA-B04 | EPS EM |
| V-EPS-15 | Array start with an open pack path (F0A, F3 or a pack lead opened; array simulator lit) and the switches released | bus up from the array, OBC boots, DS2's FETs on through the PACK_P diode | SMA-B01 | EPS EM |
| V-EPS-16 | OBC heartbeat supervision: heartbeats stopped; restored; stopped four times in 24 h | SW10 off 1 s at 120 ± 5 s after the last heartbeat; no cycle within 300 s of a start; at most 3 cycles, then stopped and reported | SMA-B09 | EPS EM |
| V-EPS-17 | Per-tracker disable and the active-high tracker clamps: each TDIS line; DS2 pole 2 and RBF pole B wires grounded at the far end | the selected tracker stops; a grounded clamp wire leaves the trackers running; stowed DS2 or RBF in holds them off | SMA-B13, SMA-B17 | EPS EM |
| V-EPS-18 | Lockout coil drivers: each of Q28, Q30, Q29, Q39 shorted in turn | K1 state unchanged; the other coil still operates | SMA-B12 | EPS EM |
| V-EPS-11 | Frequency plan: spectrum at the 70 cm port with all converters running | no comb line within 200 kHz of the coordinated uplink | E1, V-EMC-3 | EM |
| V-EPS-12 | Charge window: NTC1 swept with a decade box | inhibit at +1 ± 1 °C and +44 ± 1 °C; heater open at 40 ± 1 °C; BOTHOFF at 60 ± 1 °C | EPS-F003, H5, H6 | EPS EM |

## 20. RID closure

| RID | Disposition | Evidence |
|-----|-------------|----------|
| SE-01 (EPS part) | **Closed for the EPS board** (schematic with parts, BOM, ERC, calculations); layout and EM open | this document; `design.py`, netlist, BOM, ERC log |
| EPS-04 | **Closed** for S1 and W1/W7 (DS3 drawn and in the netlist); B2 is not this designer's file: for the chair | S1, `design.py` `inhibits()` |
| EPS-06 | **Hardware closed** (F1, Q20/Q21, latch U61, EOL_ARM/EOL_FIRE on J3); pins and commands for the chair and the flight software | §7, §13.2 |
| EPS-07 | **Closed** for S5/W7 (AntS supply downstream of DS1–DS3 via SW8, I2C via U80); AntS internal switches open (AntS ICD) | §9, S5, W7 |
| EPS-09 | **Closed** (hardware load shed) | §8.4; V-EPS-7 |
| EPS-11 | **Closed** (BQ771806 + F3) | §6.1; V-BSA-2 |
| EPS-12 | **Closed in hardware** (K1 latch, two-signal clear); clear command for the flight software | §5.4; V-EPS-9 |
| EPS-15 | **Closed** (S2 redrawn with selected parts, separate PA buck) | S2 |
| EPS-18 | **Closed in hardware** (four driver inputs); pin for W2B for the chair | §9, §13.2 |
| SMA-01 | **Closed** (DS3 designed) | §6.2 |
| SMA-05 | **Closed** (0.8 A per resistor, 12 s one-shot, peak 0.67 A, fault cap 2.0 A) | §9; V-EPS-6 |
| SMA-06 | **Closed in hardware** (= EPS-12) | §5.4 |
| REL-03 | **Closed** (one driver per resistor, so per wing) | §9 |
| LV-05 | **Closed** (inhibit schematic, three switches) | §6, S1 |
| RF-14 | **Closed** for the EPS: three physical RF inhibits DS1–DS3 between both sources and the PA supply, plus SW3 | §6.2 |
| AV-08 | **Closed** for the EPS (OBC, radio, sensor, GNSS, controller rails tripped and retried) | §8.3; V-EPS-10 |
| AV-03 | **Closed for the EPS** (Version B): coils on their own switch SW11, so the sensor rail SW4 carries the sensors only; the sensor latch-up trips are on the OBC | §8.3; V-EPS-13 |
| AV-06 | **Closed for the EPS** (Version B): VSW1_RB and VJET_RB readbacks on J3-37 and J3-19 | §11; V-EPS-13 |
| AV-10 | **Closed for the EPS** (Version B): 3V3_RTC from the always-on rail on J3-38 | §13.2 |

## 21. Not verified and open

1. No part has been bought, built or tested; every number is analysis (`eps_calcs.py`).
2. Pin numbers of the TQ2 relay symbol and the AntS connector pinout are from the part family, not the drawings.
3. F0/F2 are now FM12A72V5.0A (Version C): the 1000 A interrupt rating and the 60 °C rerating (0.80 assumed) are to be
   read from the MIL-PRF-23419/12 slash sheet and the vendor's (AEM) data sheet at procurement; the FM12 footprint
   replaces the Nano2 land at layout.
4. The ITV4030's −10 … +65 °C rating (F1 on this board, F3 on the sense board): blows only inside the rating by design (§6.1, §7); carry below −10 °C and the ±70 °C margins rest on the flight-lot qualification V-EPS-8.
5. BQ76952 behaviour assumed from its datasheet (FUSE drive on PF, SUV in SLEEP); the TRM settings must be checked when
   the OTP image is written (V-EPS-2).
6. Tracker efficiency is marginal against 92 % (§4.3).
7. No radiation data for any part; latch-up protection by the trips.
8. VBAT_MTQ has no current monitor on this board (§2.2): its current telemetry depends on the OBC's coil-current
   channels.
9. The schemdraw sheets S1, S2, S5 and the WireViz files W1, W2, W7 were rendered by `hardware/diagrams/build.sh` in the
   CDR consistency pass (2026-10-27).
10. Connector codes (Version C): the Samtec TFM-120-02-S-D-A code (SMT lead style, 30 µin gold, latch option) and
    the Pico-Clasp 203556 sizes other than 6 and 8 ways are from the series pattern; confirm on the Molex and Samtec
    drawings together with the zone footprints (SQSO-HAR-001 V-HAR-3). The sense-board fuses F0A, F0B, F0N, FT0–FT4
    and the tap chains are on the battery's interconnect board, not in this netlist (BATTERY_DESIGN 2.7 §7.2).
11. DS2 becomes a two-pole (DPDT) deployment switch with a five-wire harness W12B (Version D): the frame RFQ and the
    harness design (SQSO-HAR-001) are to take it.
12. EPS-F002 (proposal for the CR): "≥ 90 % conversion efficiency at every operating point ≥ 1 W per tracker over
    3.5–6.4 V input, ≥ 91 % power-weighted over the orbit-average profile", with V-EPS-1 and V-PWR-3 set to the same
    values and the power analysis's MPPT factor 0.92 → 0.91 (−1.1 % delivered power; the SAFE/ACTIVE margins of
    SQSO-PBA-001 2.2, +54 % / +34 %, absorb it). The design gives 90.9–92.3 % (§4.3).
13. F1 heater for models (the SIL uses 35 Ω): the ITV4030L1412NR heater is **11.2–20.0 Ω** (Littelfuse
    ITV4030 data sheet, electrical characteristics), so 15.6 Ω nominal with both ends checked: 9.8–17.5 W at 14 V,
    inside the 6–35 W firing window (cut < 1 min). 35 Ω would give 5.6 W at 14 V, below the window. The ITV4030's
    interrupting current (Ibreak) is 50 A, which is why F3 and F1 sit behind the FM12 fuses.
14. F0B (FM08A125V5A, Littelfuse, MIL-PRF-23419/8; axial cartridge on the sense board, staked): interrupt rating
    at 16.8 V DC (300 A is stated at 125 VDC for the commercial 265 series), the FM08 vacuum note of MIL-PRF-23419
    §6.1.3 and the 60 °C rerating to be read from the slash sheet and confirmed with Littelfuse at procurement; the
    current it must interrupt is ≤ 211 A (§6.1). If not confirmed, the alternative diverse part is a second space-grade
    fuse with ≥ 314 A at 16.8 V DC from a third manufacturer.
15. The harness design (SQSO-HAR-001) is to take, for Version E: W12B pin 4 is DS2's C2 fed from VCTL through 10 kΩ
    and pin 5 the Q40 clamp gate (was GND and TRK_EN); W12C drops DS3's NO wire (two wires); W12D pin 3 is the Q41
    clamp gate and pin 4 RBF pole B's VCTL feed (was TRK_EN and GND). No wire count changes except W12C (−1).

## 22. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR design (EPS board) | Initial build-to design: schematic as code, BOM, ERC, calculations, interfaces, layout constraints, verification items |
| B | 2026-10-27 | CDR design (EPS board) | Interconnect changes from SQSO-ICX-001 §7 (E1–E8): VJET_RB (J3-19) and VSW1_RB (J3-37) readbacks; SW11 U39 TPS1H000-Q1 VBAT_MTQ (J3-36/39, ADCS_EN, 0.32–0.40 A); 3V3_RTC (J3-38); J20 → OBC J6, J21 → OBC J4; OBC pins PB2, PF3, PE7, PF4, PG1, PF5; pack-current, stress and thermal tables recomputed; U17 transient Tj corrected to 93 °C (the calculation output; Version A quoted 79 °C) |
| B (amended) | 2026-10-27 | CDR design (EPS board) | J21 placed on the +X edge, +Y side for W9B in the (+X, +Y) bundle (SQSO-ICX-001 B §2.1); connector-edge row added to §17; no netlist change |
| C | 2026-10-27 | CDR design (EPS board) | Harness design SQSO-HAR-001 2.0 §15 (E-1 to E-9): every harness connector on the bottom side in zones Z1–Z5 and the +X edge; Micro-Fit 43045 right-angle and Pico-Clasp 203556 headers, all gold; J19 and J12 to Pico-Clasp 6 with the harness pinouts; J3 to Samtec TFM-120 vertical (SFSDT); PACK corner at (−X, +Y) in Z1; harness names W12A–W12D; layout rule for Z1–Z3 (§13.1, §14, §17). W1 upstream of F2: sense-board fuse F0 and tap fuses (B-1, BATTERY_DESIGN 2.5) recorded in §6.1. Netlist: J12 and J19 pinouts changed; ERC clean |
| C (amended) | 2026-10-27 | CDR design (EPS board) | Pack fuse breaking capacity: F2 (and F0 on the battery sense board) changed from the Nano2 0451005.MRL (50 A at 125 V, 300 A at 32 VDC) to the MIL-PRF-23419/12 FM12A72V5.0A (1000 A interrupt) against a 314 A prospective-current bound; tap fuses FM12A72V3/4A; storage drain with the access-port board's UMB_VMON TVS 7.7 µA worst (§6.1, §6.6, §14, §21) |
| D | 2026-10-27 | CDR design (EPS board) | Round-3 panel: array path given its third inhibit (DS2 two-pole, pole 2 holds TRK_EN; pass-through ≤ 6.16 V below every start; V-EPS-3 with two of three switches failed closed; P3E-01); sense-board fuses F0A, F0B (BAT_TOP) and F0N (BAT_BOT), tap resistor chains moved to the sense board (10 Ω left at the AFE), H3/H3b counts restated (P3E-02); EOL arm needs the EPS controller's keyed EOL_KEY AND the OBC's EOL_ARM (U64), software-fault claim corrected (P3E-11); load shed also gates the Jetson eFuse (U65; P3E-06); JETSON_PWR on PA8 (P3E-05); string wire 28 AWG (§4.1, §15); centre supports and Z-rod notch in §17 (P3S-03). Netlist: 706 placements, 153 BOM lines, 413 nets; ERC clean |
| D (amended) | 2026-10-27 | CDR design (EPS board) | §6.1, §18: H3b 2-fault tolerant on the chair's decision (F0A, F0B from two lots counted as independent), NRCSD concurrence a decision point; EOL_KEY register 0x7E in ICD-001 1.5 §4.6 |
| E | 2026-10-27 | CDR design (EPS board) | Board FMEA SQSO-FMEA-003 A findings: DS2 drive from BAT_F OR PACK_P (SMA-B01); 1 MΩ open-tap ladder (SMA-B03); NTC1 plausibility with hand-over to NTC2 (SMA-B04); PA out of the load shed (SMA-B05); AntS supply on DEPLOY_EN OR the EPS controller (SMA-B06); OBC heartbeat supervision and SW10 power cycle (SMA-B09); series-redundant lockout coil drivers (SMA-B12); active-high tracker clamps Q40/Q41, DS3 NO dropped (SMA-B13); series-pair capacitors on the pack lines before the AFE FETs, 50 V (SMA-B15; the AFE BAT capacitor was 25 V at 67 %); per-tracker disables (SMA-B17); F0B diverse (FM08A125V5A, Littelfuse); transponder 2.42 W DC at 1 W RF; storage drain 11.9 µA worst. Netlist 753 / 155 / 438, ERC clean |
| E (amended) | 2026-10-27 | CDR design (EPS board) | SQSO-ICX-001 E14/E15: AntS on the OBC's I2C3 through U80 (J3-28/40, no EPS pull-ups), J12 Pico-Clasp 8 with AntS side B jumpers; OBC heartbeat every ≤ 60 s, 600 s timeout, hardware one-shot U82 for the SW10 cycle; 2.0 mm board and stack-up, X-rod height rule (D-14); burn 10 s (C-17); DS2 fallback (second SPDT, DS2B). Netlist 759 / 157 / 445, ERC clean |
| F | 2026-10-27 | CDR design (EPS board) | Parts temperature audit SQSO-MFG-003: secondary protector BQ771806 and F3 moved to the battery sense board (F3 blows only inside its rating; own taps at the cells), F1 fired only with the board ≥ 0 °C, V-EPS-8 extended to flight-lot qualification at −40 / +70 °C; part codes corrected (1N4148WQ-7-F, ERJ 10R0, ERA-6AEB above 330 k, ERA E24 three-digit codes, WSL2010 0.5 Ω, TFM plating -L); X7R −10 % and −15 % at −40 °C in the timing worst cases: DS1–DS3 gate filters 47 → 68 nF (≥ 12.8 ms), C_RETRY R_FAULT 150 k for the 1 s retries, load-shed delay 3.7–5.5 s. Netlist 742 / 157 / 435, ERC clean |
| F (amended) | 2026-10-27 | CDR design (EPS board) | Stale lines removed: §2.1 EPS-F009 (the array's third inhibit is DS2 pole 2 through the Q40 clamp; DS3's NO is unused); §4.2 and §13.1 checked against Version E (TRK_EN clamps Q40/Q41; J16 a Pico-Clasp 5) |
| G | 2026-10-27 | CDR design (EPS board) | S&MA RIDs on the protector relocation: F3's heat at the cell (thermal island ≥ 25 mm from the tabs, ≤ 1 K at the nearest can, limit +60 °C; SMA-B21); F3 in the short loops never interrupts (series FM12/FM08 fuses; I²t data item; included in the hard-short abuse test; SMA-B22); §3, §6.1 and §17 no longer place F3 or the BQ771806 on this board (SMA-B23); 4 × 2.2 MΩ ladder on the BQ771806's taps, storage drain 13.8 µA worst (SMA-B24). No netlist change |
| B, note | 2026-10-27 | CDR consistency | Citations to BASELINE Rev E, RADIATION Rev D, ICD-001 v1.2; no OBC device on I2C2 (TMP235 analog); §21 item 9 rendered |
