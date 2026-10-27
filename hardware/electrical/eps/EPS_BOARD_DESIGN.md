# SMART-QSO EPS Board: Build-to Design

**Document ID**: SQSO-EPS-BRD-001
**Version**: A
**Date**: 2026-10-27
**Status**: CDR build-to electrical design (schematic captured as code, parts selected, ERC clean). No PCB layout, no
engineering model; every number below is analysis until its verification item (§19) closes.

**Design files** (this directory): [`design.py`](design.py) (SKiDL 3.0 schematic, the source of truth for nets and
parts), [`eps_board.net`](eps_board.net) (KiCad 8 netlist, generated), [`eps_board_nets.csv`](eps_board_nets.csv) (net to
pins, generated), [`eps_board_bom.csv`](eps_board_bom.csv) (BOM, generated), [`eps_board_erc.log`](eps_board_erc.log) (ERC,
generated), [`eps_calcs.py`](eps_calcs.py) (every computed value in this document). Regenerate with the CAD venv:
`python hardware/electrical/eps/design.py` and `python3 hardware/electrical/eps/eps_calcs.py`.

**Governing documents**: `docs/BASELINE_PARAMETERS.md` Rev D (governs every value); `docs/hardware/EPS_DESIGN.md` 2.1
(architecture); `hardware/eps/PDU_DESIGN.md` 1.2, `hardware/eps/EPS_ELECTRICAL_DESIGN.md` 1.2,
`hardware/eps/BATTERY_DESIGN.md` 2.3, `hardware/eps/SOLAR_ARRAY_DESIGN.md` 2.1; `docs/environment/BATTERY_SAFETY_ASSESSMENT.md`
(SQSO-BSA-001), `docs/environment/EMC_ANALYSIS.md` (SQSO-EMC-001), `docs/environment/POWER_BUDGET_ANALYSIS.md` (SQSO-PBA-001
v2.0), `docs/environment/THERMAL_ANALYSIS.md` (SQSO-THA-001 v2.0), `docs/environment/RADIATION_ANALYSIS.md` Rev C §5.3;
`docs/interfaces/ICD_OBC_EPS.md` (SMART-QSO-ICD-001 v1.1); `docs/manufacturing/EEE_PARTS_LIST.md` Rev C §2A; CDS Rev 14.1
§2.3; SpaceX Rideshare Payload User's Guide v11 §6.7.7.

---

## 1. Summary

| Item | Value |
|------|-------|
| Board | PC/104 (90.17 × 95.89 mm), 6 layers, in the OBC/EPS stack; battery on flying leads (no stack connector) |
| Placements / BOM lines / nets | 686 / 149 / 402 (generated counts) |
| ICs | 72 plus the oscillator (TI unless noted): BQ76952 AFE, BQ771806 secondary protector, 4 × TPS55340-Q1 trackers, 8 × LM74700-Q1 ideal diodes, 3 × LMR43620-Q1 bucks, TPS16630 eFuse, 2 × TPS1H100B-Q1, TPS1H000-Q1, 7 × TPS2553-Q1, 2 × INA228-Q1, 4 × INA3221-Q1, 3 × INA186-Q1, ADS7138-Q1, MSP430FR2355 EPS controller, ISO1540-Q1, TCA9517-Q1, 2 × REF3425-Q1, TPS70933-Q1, 6 × TPL5111, TLV9002/OPA2991/TLV7042 analog, LVC logic, Epson SG-8018CA oscillator |
| Worst-case board dissipation | **1.29 W** sunlit ACTIVE at maximum array power with Jetson and PA on; 0.35 W in eclipse; **3.75 W for ≤ 12 s** during a burn; orbit average ≈ 0.48 W (§16) |
| Hottest parts | burn-driver FET Q24–Q27 (3.0 W for 12 s, Tj 92 °C); burn buck U17 (0.95 W, 12 s, Tj 79 °C); tracker U4 (0.21 W, Tj 66 °C); D9–D12 (0.12 W); SW1 U20 (0.17 W); 5V_PA buck U16 (0.15 W); all with the board at +60 °C |
| Worst stresses | DS2 FET Vgs 73 % (7.3 V / 10 V); BQ771806 VDD 70 % (17.4 V / 25 V); PTC at 68 % of its 60 °C hold; INA3221 heater channel 65 % (16.8 V / 26 V); all within the §2.3 limits (§15) |
| Storage drain, switches actuated | **2.2 µA typical, 6.7 µA worst** (limit 20 µA; §6.6) |
| Peak pack current, one burn | **0.67 A** (chair limit 1.0 A); four drivers at once (software fault) held to 2.0 A by the burn buck's limit |
| Tracker efficiency (analysis) | 90.9–92.3 % (EPS-F002 asks ≥ 92 %: **marginal**, §4.3) |

What is new against EPS_DESIGN 2.0 (all within its architecture; the departures are listed in §2.2): the parts; a
7.0 V burn buck so the linear burn drivers dissipate 3 W instead of 10 W and draw a third of the pack current; a
back-to-back DS2 so the pack return blocks charge as well as discharge; a power domain rule that keeps DS2 from being
bypassed by a sneak path; four burn drivers; an OBC supply switch with latch-up power cycling (AV-08); and the
EPS controller that the ICD's register block at 0x48 already assumed.

## 2. Requirements, standards and departures

### 2.1 Requirements implemented (EPS_DESIGN 2.1 §2)

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
| EPS-F009 | ≥ 3 independent physical inhibits, RF and each deployable | DS1 → Q4, DS2 → Q6/Q7, DS3 → Q5 and TRK_EN | §6 |
| EPS-F010 | Non-resettable EOL charge disconnect, two independent signals | F1 three-terminal fuse, Q20 (EOL_FIRE) in series with Q21 (EOL_ARM latch) | §7 |
| EPS-F011 | Battery protection per SQSO-BSA-001 §3 | BQ76952 (OTP), BQ771806 + F3, F2, RT1, NTC comparators, lockout K1 | §5, §6 |
| EPS-F012 | Hardware low-voltage disconnect of payload and PA | U48 + U51/U52: < 13.6 V for 5 s sheds SW1 and SW3, release > 14.4 V | §8.4 |
| EPS-F013 | Telemetry | 5 analog lines to the OBC, INA228 × 2 on I2C2, AFE / INA3221 / ADS7138 via the EPS controller | §11 |

### 2.2 Departures from the architecture documents (for the chair)

| Item | EPS_DESIGN 2.0 / analysis | This design | Why |
|------|---------------------------|-------------|-----|
| Burn supply | Drivers on VBAT through SW6 | SW6 → **7.0 V burn buck U17** → drivers | A linear 0.8 A limiter on 16.4 V dissipates up to 10 W per driver; from 7.0 V ≤ 3.0 W, and the pack sees 0.51 A per burn instead of 0.88 A. The buck's 2.8 A limit caps a four-driver software fault at 2.0 A at the pack |
| Burn drivers | One per wing (§5.1 text) / chair: one per resistor | **One per resistor (four)** | Chair decision; REL-03 |
| DS2 | One low-side FET | **Two FETs back to back** (Q6, Q7) | One FET's body diode passes charge current into the pack with DS2 open |
| DS1/DS3 gate return | not specified | DS1 returns to PACK_N, DS3 to GND, each through its own contact | Independence: no FET's drive depends on another switch's wiring (§6.3) |
| Shunts for gauge and charge loop | low side | **high side** (RS6 between F3 and the AFE FETs) | Any ground-referenced circuit on the PACK_N node would bypass DS2 (§6.4) |
| NTC1 | common return in the pack | **own return wire** (W1 J1-6) | Keeps the hardware comparators in the ground domain without a path to the pack negative |
| Charge inhibit | "charger disabled and CHG FET open" | Charge-current set point to 0 A (trackers keep the bus up) **and** AFE CHG FET open (CFETOFF) | A boost-tracker power path cannot be disabled without losing the bus; the set point is the charge path's own control |
| Converter UVLO (EMC rule E8) | 10.8 V falling | 7.5 V (3V3_AO) and 8.7 V (others) falling; 10.5 V and 12.1 V rising | The LMR43620's fixed EN hysteresis makes a 10.8 V fall a 15.1 V start; E8's intent (BMS decides, below 11.2 V) is met (§8.1) |
| Cell-tap filters (EMC rule E6) | 1 kΩ + 0.1 µF at the AFE | 100 Ω + 0.1 µF at the AFE (internal balancing at ≈ 18 mA); 1 kΩ + 0.1 µF at the secondary protector | 1 kΩ would cut balancing to ≈ 3 mA; 100 Ω/0.1 µF is a 16 kHz pole, ample against the 145.9 MHz field. **For the EMC owner:** amend E6 |
| Jetson rail trip | 1.2 A (EPS_DESIGN), 0.75 A proposed (RADIATION_ANALYSIS §5.3) | 1.20 A (1.10–1.30 A) eFuse, OBC guard 0.93 A first | A latch-up (amperes) trips either; 0.75 A would sit 1.34× the 8 W load at the low bus |
| Tracker efficiency | ≥ 92 % | 90.9–92.3 % (analysis) | See §4.3; the power margins (+31 % tumbling) absorb it |

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
 BAT_TOP ─F2─RT1─F3(OVP)─RS6─[Q1 CHG|Q2 DSG]─ PACK_P ─Q4(DS1)─ VB1 ─Q5(DS3)─ VB3 ─RS8─ VBAT ─┼─ SW5 heater (F4, 40 °C cutout)
   4S cells (U1 BQ76952, U2 BQ771806)                                                       └─ SW6 → U17 V_BURN → 4 burn drivers
 BAT_BOT ─RS7─ PACK_N ─[Q6|Q7 back to back](DS2)─ GND        umbilical J13: PACK_P / PACK_N (bypasses the inhibits for charging)
```

**Domains.** The PACK domain is referenced to BAT_BOT (cell 1 negative): cells, F2/RT1/F3, the AFE U1, its FETs, the
secondary protector U2 and the DS2 gate drive. Everything else is the GND domain. DS2 sits between them. Every signal
that crosses goes through the I2C isolator U50 or a ≥ 100 kΩ series resistor, and no ground-referenced part touches
PACK_N: otherwise a component's input clamp would carry bus return current around an open DS2 (§6.4).

**Rails.** VBAT (bus); 3V3_AO; 3V3_OBC and 3V3_MCU (switched, with latch-up retry); 5V_PA; V_BURN (7.0 V, only with
DEPLOY_EN); VCTL (3.3 V LDO U40 fed by the strings: the charge-control loops run whenever the array is lit, even with
a dead bus); VHW (diode-OR of VCTL and 3V3_AO: the NTC1 comparators); REG1 (AFE LDO: AFE I2C side of U50); REF_C and
REF_A (2.500 V references in the VCTL and 3V3_AO domains).

## 4. Solar inputs and trackers

### 4.1 Strings

Eight strings on J4–J9 (W2), two contacts and two 26 AWG wires per pole (0.28 A each against 0.5 A derated: RID EPS-13).
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
loop always governs. Enables: TRK_EN, pulled up from VCTL and held low by DS3's NO contact and the RBF's pole B.

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

### 5.4 Latched recharge lockout (RID EPS-12 / SMA-06)

| Element | Implementation |
|---------|----------------|
| Detection | BQ76952 Safety Cell Undervoltage permanent fail (SUV), threshold 2.00 V, delay 5 s, written to OTP at acceptance; the AFE runs in NORMAL/SLEEP on orbit (protections on) so it sees a cell pass 2.0 V while the pack drains |
| Memory | **K1 Panasonic TQ2SA-L2-3V-Z**, 2-coil latching relay: non-volatile without power, independent of any RAM or OTP |
| Set | AFE FUSE output (PF) → 100 k/100 k → U56 OR (with the EPS controller's LOCK_SET, which it raises if it reads the PF or a cell < 2.0 V) → U57 TPL5111 one-shot (powered by the request, so one 50 ms pulse per request and on every power-up while it persists) → Q28 → SET coil |
| Effect | Pole 1 (NC) opens PERMIT → CHG_INHIBIT: CC set point 0 and AFE CHG FET off. Pole 2 (NO) grounds the CC set point directly. Two independent paths; the AFE PF also holds its FETs off until reset. The bus keeps running from the array (§5.2) so the OBC and receiver can take the clear command |
| Clear | **Only** LOCK_CLR (OBC GPIO, after the authenticated telecommand) **AND** UNLOCK (EPS controller GPIO, written by the OBC over I2C as a second step) → U58 → U59 one-shot → Q29 → RESET coil. No single line, glitch or controller fault clears it |
| Precharge | Below 3.0 V/cell the AFE charges through Q3 and 200 Ω (≤ 45 mA); at an AFE wake with a cell below 2.0 V, at most 45 mA for the 5 s SUV delay (0.06 mAh) can flow before the PF |
| Residual risk | Relay change of state by shock (TQ2 functional shock 750 m/s²): locked-out after a shock is safe, unlocked by a shock during a lockout needs a second event; included in V-EPS-9 |

## 6. Battery protection and inhibits

### 6.1 Protection chain and coordination

| Item | Value | Margin |
|---|---|---|
| Largest legitimate pack current (Jetson at the 12 W guard + PA 1 W + housekeeping + heater) | 1.31 A | |
| Single burn, normal sequence (one resistor, 0.88 A at +10 %) + housekeeping + heater | 0.67 A | chair limit 1.0 A |
| All four burn drivers on (software fault): burn buck in its 2.8 A current limit | 2.00 A | hardware cap |
| AFE OCD1 (40 mV on RS7 10 mΩ, 500 ms) | 4.0 A (3.6–4.4) | 2.7× the largest load; below F2 |
| AFE SCD (100 mV, 15 µs) | 10 A | cell short-circuit current 80–100 A |
| F2 0451005.MRL 5 A at 60 °C (× 0.90 rerating) | 4.5 A | largest load 29 % |
| RT1 2920L260/33DR hold at 60 °C | 1.92 A | 68 % of hold |
| F3 ITV4030L1412NR 12 A (secondary-protector fuse) at 60 °C (10 A) | 10 A | 13 % |

F2 (Littelfuse Nano2 451, 5 A, 125 V; 50 A at 125 V, 300 A at 32 VDC per the 451/453 datasheet as quoted by
distributors, to be confirmed on the current datasheet) is the only element rated to break the cell's 80–100 A short
current; RT1 (40 A maximum) and F3 (50 A) are not, so they sit after F2. The PTC's trip rises to ≈ 6.5 A at −20 °C;
it is a backup.

**AFE BQ76952 OTP settings** (written and verified at EPS acceptance; V-EPS-2): COV 4.25 V (1 s), CUV 2.80 V,
recovery 3.00 V, OCD1 40 mV 500 ms, SCD 100 mV 15 µs, OCC (charge) 10 mV (1 A) 1 s, UTC 0 °C / OTC 45 °C / OTD 60 °C on
TS1 (NTC2), FET temperature on TS3 (RT2), SUV PF 2.00 V 5 s (FUSE drive on PF, no PF write to OTP), precharge below
3.0 V, body-diode protection on, internal balancing above 3.9 V at > 50 mV spread (≈ 18 mA through the 100 Ω taps, one
cell at a time), CFETOFF and DFETOFF (BOTHOFF) active high with internal pull-downs off, auto-SHUTDOWN on low voltage
**disabled** (it would blind the SUV detection), wake from SHUTDOWN on TS2 and LD. Cell inputs: VC0–VC2, VC15, VC16 to
the four cells; VC3–VC14 shorted to VC2 (datasheet §16.6).

**Secondary over-voltage protector** U2 **BQ771806DPJR** (4.350 V ± 10 mV per cell, 3 s delay, CMOS active-high
output): own taps through 1 kΩ + 0.1 µF from the same connector J2, own supply ahead of F3, own ground; output through
47 k/100 k and an 8.2 V zener to Q16, which pulls F3's heater (11.2–20 Ω) to the cell-1 negative: 15–27 W, F3 opens in
< 1 min (datasheet: 6–35 W window). F3 is in the pack line, so a trip isolates the pack for good (charge and
discharge); the spacecraft then runs on the array in sunlight. This gives overcharge three dissimilar controls: CV loop,
AFE COV (OTP), fixed-threshold U2 + F3 (SQSO-BSA-001 H1: two-fault tolerant; RID EPS-11).

**Fuel gauge**: U28 INA228-Q1 on RS6 (high side), 20-bit, charge and energy accumulators, OBC I2C2 at 0x44, powered
from 3V3_AO; plus the AFE's coulomb counter (RS7). Two NTCs at the cells: NTC1 (hardware comparators, ADS7138 telemetry)
and NTC2 (AFE TS1). **Pack-to-frame isolation** ≥ 1 MΩ at 50 VDC is a property of the pack and its mount and is tested
with J1/J2 disconnected (V-BSA-4): on the board, PACK_N joins GND only through DS2 (§6.4).

### 6.2 Inhibit chains

| Inhibit | Switch (connector) | Element | Gate drive | Breaks |
|---------|--------------------|---------|-----------|--------|
| **1 DS2** | J16: C via 10 k from BAT_F, NC → S2N; RBF pole A (J14 1-2) in series → S2R | Q6 + Q7 BUK9Y4R4-40E back to back, PACK_N–CS2–GND | S2R → 1.3 M → G2, 1.0 M + 47 nF to CS2; Vgs 4.9–7.3 V | pack (charge and discharge) |
| **2 DS1** | J15: C via 10 k to PACK_N, NC → S1N | Q4 IPD50P04P4L-11 in PACK_P→VB1 | 1.0 M gate–source, 47 nF, 1.0 M to S1N; |Vgs| 5.6–8.4 V | pack and array |
| **3 DS3** | J17: C → GND, NC → S3N, NO → TRK_EN | Q5 IPD50P04P4L-11 in VB1→VB3; NO contact holds TRK_EN low when stowed | as DS1, returned to GND | pack and array (and the trackers) |
| (4) | — | SW3 (RF), SW6 + driver FET (wings), SW8 + AntS ARM (antennas) | OBC lines, low at reset | margin only |

| Gate network | Network | |Vgs| (rating) | Filter time constant | Current |
|---|---|---|---|---|
| DS1 / DS3 high-side P-FET gate | 1.0 M / 1.0 M, 47 nF | 5.6–8.4 V (52 % of 16 V) | 23.5 ms on / 47.0 ms off | 8.4 µA released, 0 stowed |
| DS2 back-to-back N-FET gate | 1.3 M / 1.0 M, 47 nF | 4.9–7.3 V (73 % of 10 V) | 26.6 ms on / 47.0 ms off | 7.3 µA released, 0 stowed |

A contact closure must last 11.7 ms before a DS1/DS3 FET begins to conduct at the 11.2 V minimum, so contact chatter in
vibration cannot power the bus; the switches carry µA only (RID EPS-05). Wiring: each switch on its own rail and its own
three-wire harness; the 10 kΩ at the board end of DS1's and DS2's common limits a harness short to the frame to ≤ 1.7 mA.
The RBF (J14) is two-pole: pole A in series with DS2's drive, pole B clamping TRK_EN (not counted). DS2's release also
wakes the AFE from SHUTDOWN (Q23 pulls TS2 to VSS); the array's first light wakes it through LD as a second path.

### 6.3 Independence

No single failure removes more than one of the three: each has its own switch, rail, harness, gate network and power
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
| Q16 secondary-fuse FET leakage (PMV40UN2, 25 °C / 60 °C) | 0.05 | 1.0 |
| Q23 wake FET + DS2 gate network (contacts open) | 0.0 | 0.1 |
| Pack TVS D14, CV and VBATT dividers: isolated by the open AFE FETs | 0.0 | 0.0 |
| Cell-tap filter capacitors (ceramic leakage, 9 × 0.1 µF) | 0.01 | 0.1 |
| **Total** | **2.2** | **6.7** |

90 days at the worst drain: 14.5 mAh (0.42 % of 3.45 Ah). The AFE enters SHUTDOWN on the GSE command after the
final charge (UMB_SHUT) and stays there until DS2's release or the array's light wakes it; in SHUTDOWN its FETs are open,
which also isolates PACK_P (not counted as an inhibit).

## 7. End-of-life charge disconnect (97.207(g)(1)(iii); RID EPS-06)

F1 **ITV4030L1412NR** (three-terminal fuse, 12 A, heater 11.2–20 Ω, triggerable 10.5–19.6 V) sits between the trackers'
output (VTRK) and PACK_P: opening it disconnects the array from the pack and the bus, so the loads drain the pack to the
BMS cut and the spacecraft stays dark. Firing needs **both**: Q21 (gate from the arm latch U61, a D flip-flop clocked by
an EOL_ARM rising edge through a 1 ms RC and a Schmitt buffer, cleared at every power-up) and Q20 (gate from EOL_FIRE,
1 ms RC, 100 k pull-down), in series in the heater return. Heater power 9.8–24 W at 14.0–16.4 V (the datasheet's 6–35 W
window, cut < 1 min); fire in sunlight so the array also feeds the heater. No single failure fires it: a stuck line,
a latch upset or one shorted FET each leaves the other element open. F1_SNS (1 M / 220 k from the heater pin) tells the
controller whether F1 is intact. Pins and commands: EOL_ARM and EOL_FIRE on J3-32/33 (§13; the flight software's
PASSIVATE_ARM / PASSIVATE_EXECUTE of REG-01 map onto them).

## 8. Distribution

### 8.1 Converters

| Converter | Output | Inductor ripple | Loss at full load | Efficiency |
|---|---|---|---|---|
| U15 LMR43620MC3 3V3_AO | 3.30 V / 0.30 A | 0.39 A p-p | 67 mW | 93.7 % (85–90 % at 0.1 A, forced PWM) |
| U16 LMR43620MC5 5V_PA | 5.00 V / 0.54 A | 0.52 A p-p | 143 mW | 95.0 % |
| U17 LMR43620MC5 (adj.) V_BURN | 7.04 V / 1.76 A | 0.60 A p-p | 948 mW | 92.9 % |

All 2 A parts, 42 V abs max input (VBAT 40 %), no spread spectrum (MC variants), synchronised to SYNC_A. UVLO (EN
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
| SW10 OBC 3.3 V | TPS2553-Q1 U38 | 178 k | 0.152 (0.130–0.176) A | 0.040 A | 0.28 s, auto | always (no command) |
| SW9 EPS controller | TPS2553-Q1 U27 | ILIM to IN | 0.075 (0.050–0.100) A | 0.012 A | 0.28 s, auto | always |
| SW2 RADIO | TPS2553-Q1 U22 | 88.7 k | 0.299 (0.265–0.339) A | 0.100 A | 1.3 s, auto | RADIO_EN PB5 |
| SW3 PA (TX-OFF) | TPS2553-Q1 U23 | 26.1 k | 0.989 (0.918–1.071) A | 0.540 A | 0.28 s, auto | RF_TX_EN PA3 AND SHED_N |
| SW4 ADCS (sensors, coils) | TPS2553-Q1 U24 | 88.7 k | 0.299 (0.265–0.339) A | 0.130 A | 0.28 s, auto | ADCS_EN PB6 |
| SW7 GNSS | TPS2553-Q1 U25 | 133 k | 0.202 (0.175–0.232) A | 0.100 A | 1.3 s, auto | GNSS_EN PE5 |
| SW8 AntS | TPS2553-Q1 U26 | 18.7 k | 1.370 (1.287–1.465) A | 0.610 A | 0.28 s, auto | DEPLOY_EN PA1 |
| SW1 PAYLOAD | TPS1H100B-Q1 U20 | 1.24 k | 1.99 (1.71–2.27) A | 0.93 A | thermal cycling | PAYLOAD_EN PB4 AND SHED_N |
| U19 Jetson eFuse | TPS16630 | 15.0 k | 1.20 (1.10–1.30) A | 0.93 A (12 W guard) | latch-off, OBC re-enables | JETSON_PWR PA2 |
| SW6 DEPLOY | TPS1H100B-Q1 U18 | 1.24 k | 1.99 (1.71–2.27) A | 1.25 A | thermal cycling | DEPLOY_EN PA1 |
| SW5 HEATER | TPS1H000-Q1 U21 | 3.24 k | 0.148 (0.126–0.170) A | 0.079 A | auto-retry | HEATER_EN PD2 |

TPS2553 rails: the current is limited within 2 µs; FAULT asserts after its 8 ms deglitch and pulls EN low (FAULT tied
to EN, R_FAULT 100 kΩ, C_RETRY 22 µF or 4.7 µF), which removes the rail; EN recharges to its threshold in ≈ 0.6 × R × C
(1.3 s or 0.28 s), and a persisting latch-up repeats the cycle. A BAT54 across R_FAULT makes a commanded turn-off fast
(< 1 ms). Trip levels against RADIATION_ANALYSIS §5.3: OBC 0.15 A (normal ≈ 30–40 mA, proposed 150 mA), radio 0.30 A
(0.1 A, proposed 0.3 A), GNSS 0.20 A (0.1 A, proposed 0.2 A), all ≥ 100 ms off (≥ 1 s for radio and GNSS). **The OBC rail
SW10 is the power-cycling supervisor the OBC cannot be for itself**: it needs no command, and while it is off every OBC
output is low or high-impedance, so the EPS pull-downs turn every switched load off (the TX-OFF latch lives in the OBC's
FRAM and survives). The sensor rail SW4 trips at 0.30 A only: a magnetometer or gyro latch-up (tens of mA) is masked by
the coil current, as RID AV-03 records; separating the sensors onto their own switch needs a rail and a pin on the ADCS
board (for the chair). Every switch's fault state is read by the EPS controller (FLT_* lines) and reported in the
LOAD_SHED/fault register.

### 8.4 Hardware load shed (RID EPS-09)

| Item | Value | Requirement |
|---|---|---|
| Shed (falling) | 13.62 V | VBAT < 13.6 V |
| Release (rising) | 14.42 V | VBAT > 14.4 V |
| Delay before shedding | 4.9 s (4.75 M × 1.0 µF from 3.1 V to the 1.100 V threshold; 4.2–5.7 s) | 5 s |
| Release delay | < 1 ms | |

U48A compares VBAT (1.00 M / 221 k, 0.1 %; 4.12 M hysteresis) with REF_A; U48B times 5 s; SHED_N gates SW1 (U51) and
SW3 (U52) in hardware, independent of the OBC and the EPS controller. The heater is not shed (EPS_DESIGN §6).

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

Energy into a 5 Ω resistor in the 8 s burn: 25.6 J. Peak pack current: one resistor 0.51 A for the burn, **0.67 A**
with housekeeping and heater; two at once (an A/B-pair line arrangement, §13) 1.25 A; four at once 2.0 A (U17's limit).
Hinge sense: both wings' switches in series (J10-5/6, J11-5/6), 10 k pull-up, 1 k + 100 nF to SOLAR_SENSE (PB12): low =
both deployed. **AntS** (RID EPS-07): 3V3_ANTS from SW8 (DEPLOY_EN), downstream of DS1–DS3; I2C2 through the TCA9517-Q1
U80, whose ports are high-impedance while the AntS is unpowered; the AntS's ARM command and thermal knives complete its
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
| Pack gauge | U28 INA228-Q1 on RS6 (V, I, charge, energy, die temperature) | I2C2 0x44 | OBC (direct, independent of the controller) |
| Array output | U29 INA228-Q1 on RS5 | I2C2 0x40 | OBC (direct) |
| EPS register block | U37 MSP430FR2355 | I2C2 0x48 (ICD §4.2, §4.6) | OBC |
| Cell voltages, AFE status, PF, CC | U1 BQ76952 via U50 | local bus 0x08 | EPS controller |
| Tracker inputs ×4, bus load, 3V3_AO, 5V_PA, radio, ADCS, GNSS, AntS, heater | U32–U35 INA3221-Q1 (10–500 mΩ shunts) | local bus 0x40–0x43 | EPS controller |
| Panel NTCs ×6, NTC1, VHW | U36 ADS7138-Q1 | local bus 0x10 | EPS controller |
| Burn currents ×4, Jetson IMON, F1 intact, V_BURN, VTRK | MSP430 ADC (A0, A4, A6, A7, A8–A11) | — | EPS controller |
| PERMIT, WIN_OK, SHED_N, EOL armed, all switch faults, AFE ALERT | MSP430 GPIO (100 k series) | — | EPS controller |
| EPS_ALERT_N (PB7) | open drain, INA228s and controller | 4.7 k pull-up | OBC |

I2C2 (OBC bus) carries 0x31 (AntS, behind U80), 0x40 and 0x44 (INA228), 0x48 (EPS controller) and the OBC's own devices
(TMP117 moved to 0x49, ICD §2.3). The AFE (0x08), INA3221s and ADS7138 are on the controller's local bus. The ICD's
"battery monitor 0x55" and "solar monitor 0x40" become the INA228s at 0x44 and 0x40 (ICD change, §13).

## 12. EPS controller (U37)

MSP430FR2355TPTR (FRAM, −40…105 °C) on its own latch-up-tripping supply SW9. Firmware (a new configuration item, for the
chair: not written; Class C, NPR 7150.2D) implements the ICD register block (0x00–0x33 read by `sqso_fsw::eps_telemetry`
and the §4.6 extended registers), the MPPT perturb-and-observe on the four SAC DACs, SOC from the INA228, the lockout
SET backup and the UNLOCK step, and the fault and status flags. **No protection depends on it**: if it stops, the DACs
go high-impedance (MPPT falls back to 4.40 V per string), its lines fall to their pull-downs (no lockout set or
clear), and the OBC still has the analog lines and the two INA228s. Programming through J22 (Spy-Bi-Wire), ground only.

## 13. Interfaces

### 13.1 Connectors

| Ref | Part | Function | Harness |
|-----|------|----------|---------|
| J1 | Micro-Fit 3.0 2×4, 0430450812 | BAT_TOP ×2, BAT_BOT ×2, NTC1, NTC1 return, heater ± | W1 |
| J2 | Pico-Clasp 6, 5013310607 | TAP0–TAP4, NTC2 | W1 |
| J3 | Samtec FTSH-120-01-L-DV-K (40) | OBC (W3's 30 pins kept on 1–28, extended) | W3 (change, §13.2) |
| J4–J7 | Pico-Clasp 5, 5013310507 | body strings PX, MX, PY, MY (2 + 2 contacts, TEMP) | W2 |
| J8, J9 | Pico-Clasp 8, 5013310807 | wings W1, W2 (front ×2, back ×2, return ×3, TEMP) | W2 |
| J10, J11 | Micro-Fit 3.0 2×3, 0430450612 | burn resistors A and B, hinge switch | W7 |
| J12 | Pico-Clasp 4, 5013310407 | AntS 3.3 V, GND, SDA, SCL | W7 |
| J13 | Micro-Fit 3.0 2×3 | umbilical: charge ×2, return ×2, pack monitor, AFE shutdown | W11 (new, for the chair) |
| J14 | Pico-Clasp 4 | RBF (two poles) | access port |
| J15–J17 | Pico-Clasp 3 | DS1, DS2, DS3 (C, NC, NO) | one harness per switch, on its own rail |
| J18 | Micro-Fit 3.0 2×2, 0430450412 | VBAT_JET ×2, GND ×2 | W5 (change: VBAT, not 5 V) |
| J19 | Micro-Fit 3.0 2×3 | PA 5 V switched ×2, GND ×3, radio 3.3 V switched | W4 (change) |
| J20, J21 | Pico-Clasp 2 | ADCS 3.3 V, GNSS 3.3 V | W6, W9 |
| J22 | FTSH 2×2 | EPS controller programming | GSE |

### 13.2 Control and status lines from the OBC (for the chair and the OBC designer)

All 3.3 V CMOS. Outputs from the OBC are active high and must be low at OBC reset; the EPS holds each with 100 kΩ to
GND. Pins as in `software/rust/firmware/src/gpio.rs` and `adc.rs` where they exist.

| J3 pin | Signal | Direction (OBC) | Level / default | OBC pin | Function |
|--------|--------|-----------------|-----------------|---------|----------|
| 1, 2 | 3V3_OBC | power in | 3.3 V, 0.15 A trip | — | OBC supply through SW10 (latch-up retry) |
| 5, 6 | I2C2_SCL, I2C2_SDA | bidir | open drain, pull-ups on the OBC | PF1, PF0 | EPS controller 0x48, INA228 0x40/0x44, AntS 0x31 |
| 8 | EPS_ALERT_N | in | open drain, active low, 4.7 k on the EPS | PB7 | INA228 alerts, controller interrupt |
| 9 | PAYLOAD_EN | out | high = on, low at reset | PB4 | SW1 (Jetson), shed-gated |
| 10 | RADIO_EN | out | high = on, low at reset | PB5 | SW2 receiver/TNC/exciter (the flight software raises it at boot, 97.207(b)) |
| 11 | ADCS_EN | out | high = on | PB6 | SW4 |
| 12 | HEATER_EN | out | high = on | PD2 | SW5 |
| 13 | DEPLOY_EN | out | high = on | PA1 | SW6 (burn buck) and SW8 (AntS) |
| 14 | RF_TX_EN | out | high = PA powered; low = TX-OFF | PA3 | SW3, shed-gated |
| 15 | GNSS_EN | out | high = on | PE5 | SW7 |
| 16 | DEPLOY_SENSE | in | high = DS2 released (1 MΩ source, clamped) | PE2 | separation state |
| 17 | BURN_W2A | out | high = burn, rising edge starts 12 s | PB9 (was ANTENNA_BURN, freed by the AntS) | wing 2 resistor A |
| 18 | BURN_W1A | out | as above | PB10 (SOLAR_BURN, driver A) | wing 1 resistor A |
| 20 | SOLAR_SENSE | in | low = both wings deployed | PB12 | hinge switches |
| 22–26 | VBATT_SNS, IBATT_SNS, VSOLAR_SNS, ISOLAR_SNS, VBUS_SNS | analog in | 0–3.3 V, 100 nF / 10 nF filtered | PC2, PC3, PC4, PC5, PB0 | §11 |
| 29 | JETSON_PWR | out | high = on | PA2 | U19 eFuse (SHDN) |
| 30 | BURN_W1B | out | high = burn | PE6 (SOLAR_BURN_B, driver B) | wing 1 resistor B |
| 31 | BURN_W2B | out | high = burn | **to allocate** | wing 2 resistor B |
| 32 | EOL_ARM | out | rising edge sets the arm latch, low at reset | **to allocate** | passivation arm (with PASSIVATE_ARM) |
| 33 | EOL_FIRE | out | high = fire (hold ≥ 60 s), low at reset | **to allocate** | passivation fire (PASSIVATE_EXECUTE) |
| 34 | LOCK_CLR | out | high pulse ≥ 100 ms with the controller's UNLOCK | **to allocate** | recharge-lockout clear |
| 3, 4, 7, 21, 27, 28, 35, 39, 40 | GND / AGND | — | | | 19 (ANTENNA_SENSE, PB11), 36–38 spare |

**Burn-line mapping (for the chair).** The avionics reviewer assigned driver A to PB10 and driver B to PE6, with driver
A burning the first resistor of both wings together (`sqso-sil/src/hw/deploy.rs`). This board has four independent
inputs, so it supports both: (a) **recommended**, four lines, one resistor at a time (chair decision, 0.67 A peak):
PB10 → W1A, PE6 → W1B, PB9 → W2A, a new pin → W2B, and the flight software drives each separately; (b) the A/B pairing,
by joining J3-17 to J3-18 and J3-30 to J3-31 on the OBC side: two resistors burn at once, peak pack current 1.25 A (above
the chair's 1.0 A, below the 4 A BMS trip), and one driver line again serves both wings (the REL-03 common cause returns
for the line, not for the drivers).

## 14. Parts

Every IC's orderable part number and status were read from the manufacturer's datasheet package addendum (TI parts,
2026-10-27, all "Active"); non-TI parts from Digi-Key listings (status "Active" unless noted). The BOM column "MPN check"
says which. Passive MPNs are built from the manufacturer's series code (Panasonic ERJ-3EKF/ERA-3AEB, Vishay CRCW/WSLP/WSL,
Murata GCM): the series were checked, the individual values not. Not checked at all: 2N7002BK, BAT54/BAT54C/BAT54S,
BZX84-C8V2, 1N4148W-Q, 0466001.NR, NCP15XH103F03RC, FTSH-120/102, Keystone 5015, and the Pico-Clasp 8- and Micro-Fit 2×2/2×3
headers (part numbers from the series pattern). Radiation: no published TID data was found for any of these parts; all are
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
| Fuses | 0451005.MRL (F2), ITV4030L1412NR (F1, F3), 0466.250NR (F4) | — | **ITV4030 operating temperature −10…+65 °C**: below the board's −30 °C AFT; qualification item V-EPS-8 |
| PTC | 2920L260/33DR | −40…85 °C | |
| Relay | TQ2SA-L2-3V-Z | −40…85 °C | gold-clad contacts; pin numbering to be checked against Panasonic's 2-coil drawing when the symbol is entered in KiCad |
| Power FETs | BUK7Y3R5-40HX, BUK9Y4R4-40E, IPD50P04P4L11ATMA2, BUK9M14-40EX | AEC-Q101 | |
| Oscillator | SG-8018CA 4.0000M-TJHPA0 | −40…105 °C | |

## 15. Stress summary (derated)

| Part | Stress | Limit | Status |
|------|--------|-------|--------|
| U2 BQ771806 VDD | 17.4 V / 25 V = 70 % | 80 % | pass |
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
| F2 / RT1 / F3 / F4 / F5 | 29 % / 68 % (of 60 °C hold) / 13 % / 32 % / 50 % | 50 % (fuses) | pass; RT1 is a PTC (hold-current rule, not fuse rule) |
| R73 precharge 200 Ω 1 W | 0.38 W at the worst precharge (16.7 V bus, 8.0 V pack) | 50 % | pass |
| RS20–RS23 burn shunts | 0.19 W / 0.5 W = 39 % | 50 % | pass |
| Connectors | ≤ 55 % of contact rating (strings, two contacts per pole) | 50 % | J4–J9 at 55 % of the 26 AWG wire rating, 28 % of the contact rating |

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
| U17 burn buck, 2 drivers on (fault) | 0 | 0.95 W for 12 s | 79 |
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
| Q16 F3 heater FET (one event) | 0 | 0.16 W for 60 s | 71 |
| Q20+Q21 EOL fire FETs (one event) | 0 | 0.14 W for 60 s | 70 |
| U37 controller, monitors, logic | 40 | | 62 |
| Y1 oscillator and divider | 12 | | 61 |
| Ideal diodes at Isc (8) | 88 | | 65 |

**Board totals for SQSO-THA-001:** 1.29 W sunlit ACTIVE at maximum array power with Jetson and PA on (worst steady);
0.35 W in eclipse; 3.75 W for ≤ 12 s in a burn (one per wing line, ≤ 8 in LEOP); orbit average ≈ 0.48 W in ACTIVE
sun-pointed. Hot spots to place on the board's conductive path to the frame: the four trackers with their diodes and
inductors (≈ 0.4 W each at the worst), the burn FETs (transient), U16 and U20. All junctions stay below the 110 °C / 125 °C
derated limits with ≥ 18 K margin at +60 °C board.

## 17. Layout guidance (layout is not in scope; these are the constraints to start from)

| Item | Requirement |
|------|-------------|
| Outline and mounting | PC/104 90.17 × 95.89 mm, four Ø3.18 mm holes at the PC/104 positions, keep-out Ø6.4 mm copper-free ring except the GND-bonded mounting hole H1 (single frame bond, star with AGND at U37); the frame vendor's stack drawing governs (BUILD_VS_BUY §2.2) |
| Stack-up | 6 layers, 1.6 mm, FR-4 Tg ≥ 170 °C (polyimide preferred for TVAC): L1 parts/power (2 oz), L2 GND, L3 power (PACK_P, VBAT, VTRK islands), L4 signals, L5 GND, L6 parts/power (2 oz); IPC-6012 Class 3/A |
| Domains | PACK domain (U1, U2, Q1–Q3, Q6/Q7, F2/RT1/F3, RS6/RS7, J1/J2 area) in one corner on its own BAT_BOT copper, joined to GND only through Q6/Q7; U50 straddles the boundary; ≥ 1.0 mm gap between BAT_BOT and GND copper |
| Power paths | pack line (J1 → F2 → RT1 → F3 → RS6 → Q1/Q2 → PACK_P → Q4 → Q5 → RS8): ≥ 3 mm/A on 2 oz outer layers, ≥ 2 A design current (5 A fuse); Kelvin sense to RS5–RS8 and RS20–RS23 |
| Converters | each tracker and buck as a tight loop (input cap, switch, diode, output cap) on L1 over solid L2; inductors and switch nodes on the side away from the antennas (rule E2); SYNC traces short, series 100 Ω at the source; no converter within 10 mm of U37's ADC pins or U53/U54 |
| Thermal | thermal vias (≥ 9 × 0.3 mm) under U4–U7, U15–U17, U19, U20, Q24–Q27 into L2/L5; the burn FETs and U17 near a mounting hole; ≥ 2 cm² copper per burn FET |
| Creepage / clearance | ≤ 30 V everywhere: IPC-2221B B4 (coated) ≥ 0.13 mm; use ≥ 0.3 mm between PACK domain and GND copper and around F1/F3 heater pins (fuse opening heat) |
| Keep-outs | 5 mm around Y1 and the clock dividers from switch nodes; no copper under F1/F3 (heat to the fuse element per Littelfuse land pattern); relay K1 away from inductors (stray field) |
| Test points | TP1 TRK_EN, TP2 VBAT, TP3 PACK_P, TP4 GND, TP5 3V3_AO, TP6 5V_PA, TP7 SHED_N; add at layout: VCTL, VHW, REF_C, REF_A, SYNC_A/B, V_BURN, each tracker COMP, CTRL_DN, PERMIT, CHG_INHIBIT, EOL_ARMED, BXP_W1A–W2B |
| Coating | conformal coat (tin-whisker mitigation, EEE_PARTS_LIST §2A.3), keep J1–J22 mating faces, K1 and the test points clear |
| RF | no RF on this board; the PA supply leaves on J19 through RS13 with a 1 µF at the switch and the PA board's own filter |

## 18. Fault tolerance summary (inputs to SQSO-FMEA-002 and SQSO-BSA-001)

| Hazard (BSA) | Controls on this board | Count |
|--------------|-----------------------|-------|
| H1 overcharge | CV loop U45A; AFE COV 4.25 V (OTP); BQ771806 + F3 (fixed threshold) | 3 (2-FT) |
| H2 over-discharge then recharge | FSW mode thresholds; AFE CUV 2.80 V; latched lockout K1 (two actuators, two-signal clear); hardware load shed U48 | 4 |
| H3 external short | F2; AFE SCD/OCD; RT1 | 3 |
| H5 cold charging | NTC1 comparators → CC set point 0 and CFETOFF; AFE UTC on NTC2; FSW; heater | 3+ |
| H6 over-temperature | FSW heater control; 40 °C cutout Q22; SW5 limit + F4; AFE OTD and the 60 °C comparator BOTHOFF | 4 |
| H8a burn over-current | 0.8 A per driver; one line at a time; 12 s one-shot; U17 limit 2.0 A at the pack; BMS 4 A | 5 |
| H9 stored energy after EOL | F1 (two signals) | 1 (+ the loads drain the pack) |
| RF / deployables at launch | DS1, DS2, DS3 (+ SW3/SW6/SW8 and the OBC timers) | 3 physical |

## 19. Verification items (this board)

| ID | Item | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-EPS-1 | Tracker efficiency 3.5–6.3 V, 0.05–1.1 A, −20/+25/+60 °C, synchronised | ≥ 92 % above 2 W, ≥ 90 % above 1 W | EPS-F002, §4.3 | EPS EM |
| V-EPS-2 | AFE OTP programming and readback; trip of every threshold (COV, CUV, OCD, SCD, UTC/OTC/OTD, SUV PF) on a pack simulator | within ± 25 mV, ± 10 % current, ± 2 K | EPS-F011, BSA §4 step 9 | EPS EM |
| V-EPS-3 | Inhibits: each of DS1, DS2, DS3 and the RBF opened alone with the other two released; bus, PA supply, V_BURN, 3V3_ANTS and TRK_EN measured | no bus (< 1 V), no tracker switching; chatter of 5 ms bursts does not raise the bus | EPS-F009, CDS 2.3.7/2.3.8 | EPS EM, FM (with V-BSA-6) |
| V-EPS-4 | GND–PACK_N resistance with DS2 open, board powered from the array simulator | ≥ 100 kΩ | §6.4 | EPS EM |
| V-EPS-5 | Storage drain, switches actuated, AFE in SHUTDOWN, 25 and 60 °C | ≤ 20 µA (design 6.7 µA) | LVR-071 | EPS EM, FM |
| V-EPS-6 | Burn drivers: 4, 5, 6, 7 Ω loads; stuck-high line; four lines at once | 0.80 A ± 10 %; stops at 12.0 s ± 1 s; pack ≤ 2.2 A | SMA-05 | EPS EM |
| V-EPS-7 | Load shed: VBAT ramp and step | shed 13.6 ± 0.2 V after 4–6 s; release 14.4 ± 0.2 V | EPS-F012 | EPS EM |
| V-EPS-8 | ITV4030 qualification at −30 °C: carry 2 A for 1 h, heater fires at 10.5 V; EOL fire on the EM (destructive) | no opening when carrying; opens < 1 min when fired; F1 opens only with both EOL lines | EPS-F010, V-BSA-7 | EM |
| V-EPS-9 | Lockout: set by an emulated SUV PF and by LOCK_SET; clear needs both lines; shock 750 m/s² in both states | state kept through power loss and shock | SMA-06 | EM |
| V-EPS-10 | Latch-up trips: 1.5 × trip step on each switched rail incl. SW10 | rail off ≤ 10 ms, back after 0.2–0.4 s or 1–2 s | AV-08, RAD-V5 | EM |
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

## 21. Not verified and open

1. No part has been bought, built or tested; every number is analysis (`eps_calcs.py`).
2. Pin numbers of the TQ2 relay symbol and the AntS connector pinout are from the part family, not the drawings.
3. Littelfuse 451 breaking capacity at 32 VDC (300 A) is from a datasheet excerpt; the distributor lists 50 A at 125 V.
4. The ITV4030's −10 °C operating limit is below the board's AFT (V-EPS-8).
5. BQ76952 behaviour assumed from its datasheet (FUSE drive on PF, SUV in SLEEP); the TRM settings must be checked when
   the OTP image is written (V-EPS-2).
6. Tracker efficiency is marginal against 92 % (§4.3).
7. No radiation data for any part; latch-up protection by the trips.
8. The schemdraw sheets S1, S2, S5 and the WireViz files W1, W2, W7 were edited but not rendered: schemdraw and WireViz are
   not installed here (`hardware/diagrams/build.sh` not run; `dot` is present).

## 22. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR design (EPS board) | Initial build-to design: schematic as code, BOM, ERC, calculations, interfaces, layout constraints, verification items |
