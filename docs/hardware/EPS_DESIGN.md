# Electrical Power Subsystem (EPS) Design Document

**Document Number**: SMART-QSO-EPS-001
**Version**: 2.3
**Date**: 2026-10-27
**Status**: CDR design description; the build-to board design with selected parts, netlist, BOM and ERC is
`hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001 C), which governs part-level detail; board-to-board pins: `hardware/electrical/INTERCONNECT.md`
(SQSO-ICX-001)

**Governing documents**: `docs/BASELINE_PARAMETERS.md` Rev D (§4 inhibits and timers, §5 power, §7 TX-OFF);
`docs/hardware/BUILD_VS_BUY.md` §2.4 (build the EPS; ISIS ICEPS2 Type B alternate), §3.2, §4; CDS Rev 14.1 §2.3;
SpaceX Rideshare Payload User's Guide v11 §6.7.7 and §7.6; JSC 20793 Rev D for the ISS backup; EEE-INST-002 and
SQSO-HAR-001 §5.2 for derating.

**Analyses**: power closure SQSO-PBA-001 v2.0; thermal SQSO-THA-001 v2.0; battery safety SQSO-BSA-001; EMC SQSO-EMC-001;
drawings SQSO-HW-DIAG-001 (B2, S1, S2, S5, S7, W1, W2, W7).

Version 1.x described an EPS assembled from GomSpace and ISIS modules (NanoPower MPPT, ISIS iBMS, NanoPower PDU) that
are no longer sold for 1U or never existed in that form (BUILD_VS_BUY §2.1, §2.8), on a 3.3/5/12 V rail set fed through
a single MPPT. This version describes the built 4S EPS that the baseline and the drawings assume, and lists what is
still open.

---

## 1. Scope

The EPS takes power from the 16-cell array, charges and protects the 4S1P pack, distributes the unregulated battery
bus and two regulated rails through load switches, provides the inhibits for launch, the hardware TX-OFF path and
the end-of-life charge disconnect, and reports its telemetry to the OBC (SMART-QSO-ICD-001 v1.1).

## 2. Requirements

| ID | Requirement | Verification | Status at CDR |
|----|-------------|--------------|---------------|
| EPS-F001 | Deliver to the bus ≥ 2.16 W EOL orbit average tumbling (SYS-100) and ≥ 3.0 W sun-pointed | A, T | Analysis: 2.43 W and 3.93 W (SQSO-PBA-001 §4) |
| EPS-F002 | Track the maximum power point of each string group of §3.2 with ≥ 92 % conversion efficiency over 3.5–6.4 V input | T | TPS55340-Q1 ×4: 90.9–92.3 % by analysis, marginal (SQSO-EPS-BRD-001 §4.3; V-EPS-1) |
| EPS-F003 | Store ≈ 50 Wh (4S1P INR18650-35E); charge CC ≤ 0.40 A, CV 4.10 V/cell (D8), only between 0 and +45 °C on the pack temperature, in hardware | T | Designed: CV 16.40 V ± 0.44 %, CC 0.394 A, window +1.1…+44.0 °C (BRD-001 §5) |
| EPS-F004 | Supply the loads from the array while charging is inhibited (power-path regulation of the battery bus) | T | Designed: charge-current set point to 0 A (BRD-001 §5.2) |
| EPS-F005 | Battery bus 11.2–16.4 V (16.8 V absolute maximum) to all VBAT loads; every converter rated ≥ 25 V input | T | §4 |
| EPS-F006 | 3V3_AO rail, always on (OBC, FRAM, receiver path, ADCS sensors) | T | §4 |
| EPS-F007 | Switched rails with the current limits of §5, every switch off at reset (pull-down) | T | §5 |
| EPS-F008 | Hardware TX-OFF: the PA supply switch (SW3) removed by RF_TX_EN low, independent of the AI payload (BASELINE §7) | T | §5, §8.1 |
| EPS-F009 | At least three independent physical inhibits each for RF and for every deployable (CDS 2.3.7, 2.3.8); timers and the RBF do not count | I, T | §8: DS1, DS2 and DS3 (DS3 added at CDR) |
| EPS-F010 | Non-resettable end-of-life charge-path disconnect (47 CFR 97.207(g)(1)(iii)), fired only by two independent signals | T | §7 |
| EPS-F011 | Battery protection with the fault tolerance of SQSO-BSA-001 §3 | A, T | §6 |
| EPS-F012 | Independent hardware low-voltage disconnect of the payload and PA rails | T | Designed (§6; BRD-001 §8.4) |
| EPS-F013 | Telemetry: pack voltage and current, each cell voltage, two pack temperatures, array voltage and current per tracker, rail currents, charge-inhibit cause, switch states and fault flags, passivation state | D | §9 |

## 3. Solar array input

### 3.1 Strings

Eight strings of two AZUR SPACE 3G30A cells (SOLAR_ARRAY_DESIGN §3), one per panel side, each with its own blocking
diode (or ideal-diode switch) on the EPS:

| String | Face (deployed) | Temperature range (THERMAL_ANALYSIS §4) | Voc, string | Vmp, string | Isc |
|--------|-----------------|------------------------------------------|-------------|-------------|-----|
| PX, MX | ±X body (2 cells each) | −23 … +52 °C | 6.31 V at −45 °C BOL (cold sunrise, the maximum) | 3.9 V at +83 °C EOL (the minimum, wings) … 5.7 V at −40 °C | ≤ 0.55 A (1414 W/m², hot) |
| PY, MY | ±Y body | −23 … +52 °C | | | |
| W1F, W2F | Wing fronts (+Y) | −45 … +83 °C | | | |
| W1B, W2B | Wing backs (−Y) | −45 … +83 °C | | | |

MPPT input range: **3.5–6.4 V, 0–0.55 A per string**; maximum array power ≈ 7.6 W (six +Y cells at a cold sunrise,
BOL; `mppt_mismatch.py`).

### 3.2 Trackers

Strings at different temperatures on one tracker lose 2.7–3.9 % of the sun-pointed face's power (wing strings run
40–50 K hotter than the body string beside them; `software/simulation/cdr/power_thermal/mppt_mismatch.py`); separate
trackers lose nothing. Opposite faces never see the Sun together and may share one. **Baseline: four trackers**:

| Tracker | Strings | Why |
|---------|---------|-----|
| T1 | W1F, W2F | Coplanar, same temperature: the sun-pointed face's main input |
| T2 | PY | Body string, cooler than the wings |
| T3 | W1B, W2B | Wing backs |
| T4 | PX, MX, MY (blocking diodes) | Body strings; ±X never lit together |

The ICEPS2 Type B alternate has three MPPT inputs: with it, put T2 and T4 together (bodies) and accept 0 % loss on the
sun face, but its other gaps (BUILD_VS_BUY §2.4) remain.

### 3.3 Charge regulator

Each tracker is a boost from 3.5–6.4 V to the battery bus (11.2–16.4 V), with input TVS above the string's 6.31 V
cold open-circuit voltage (standoff ≥ 7 V; the SMBJ6.0A of EPS_ELECTRICAL_DESIGN 1.0 would conduct at a cold sunrise,
RID EPS-13). The regulator must:

1. limit charge to 0.40 A (CC) and hold 16.40 V ± 0.5 % (CV, 4.10 V/cell);
2. inhibit charging outside 0 … +45 °C from NTC1 on the pack, in hardware (TS input), while still supplying the loads
   from the array (power path, EPS-F004): with charging refused at sunrise in the cold (THERMAL_ANALYSIS §4) the
   spacecraft must not run on the pack in sunlight;
3. pre-charge at ≤ 0.1 A below 3.0 V/cell; and if any cell has been below 2.0 V, **latch the charge path off**
   (charger disabled and CHG FET open) in a non-volatile flag that only an authenticated ground command clears, after
   the ground has reviewed the cell history (RID SMA-06 / EPS-12; SQSO-BSA-001 H2). The power path (item 2) keeps the
   bus up from the array in sunlight, so the OBC and receiver can take that command with the pack locked out;
4. report input voltage and current per tracker.

**Selected (SQSO-EPS-BRD-001 §4–§5):** four TPS55340-Q1 non-synchronous boost trackers on a common output, each
with an analog MPPT loop trimmed by the EPS controller, and shared hardware CV (16.40 V) and CC (0.394 A) loops that pull
every tracker's COMP pin down; charging is inhibited by setting the charge-current set point to 0 A (the trackers then
supply the bus only) and opening the AFE's CHG FET. The BQ25798 class was not used: it cannot be synchronised to the
frequency plan (rule E1) and four NVDC chargers cannot share one pack. The lockout memory is a 2-coil latching relay K1
set by the BQ76952's SUV permanent-fail output and cleared only by two independent lines (BRD-001 §5.4).

## 4. Battery bus and rails

| Rail | Source | Voltage | Loads | Max steady / peak | Converter rating (≥ 2× load, ≥ 25 V in) |
|------|--------|---------|-------|-------------------|------------------------------------------|
| VBAT (unregulated) | Pack through F2, RT1, RBF, DS1, inhibit chain | 11.2 (BMS cut) … 16.4 V; 16.8 V abs max; 17.0 V at the 4.25 V OV threshold | Heater, burn wires, Jetson carrier (recommended), bucks | 1.0 A / 1.8 A (burn + housekeeping + heater) | — |
| 3V3_AO | Buck U15 LMR43620MC3 from VBAT, always on | 3.3 V ± 2 % (RF board LMX2571 ≥ 3.15 V) | OBC, FRAM, receiver/TNC/exciter (through SW2 and its own LDO, EMC rule E4), ADCS (SW4), GNSS (SW7) | 0.26 A / 0.30 A | ≥ 0.6 A, 25 V |
| 5V_PA | Buck U16 LMR43620MC5 from VBAT | 5.0 V ± 2 % | PA through SW3 | 0.35 A at 0.5 W RF (1.76 W DC), 0.55 A at 1 W RF | ≥ 1.2 A, 25 V |
| VBAT_JET | eFuse U19 TPS16630 (60 V) behind SW1, carrier converter to the module's 5 V | VBAT | Jetson carrier | 0.62 A at 8 W; 0.93 A at the 12 W guard trip | 1.20 A latch-off |
| V_BURN | Buck U17 from VBAT_DEP (SW6, DEPLOY_EN) | 7.04 V | Four burn drivers | 0.88 A per resistor, one at a time | 2 A part; its 2.8 A limit caps a four-driver fault at 2.0 A at the pack |
| 3V3_OBC, 3V3_MCU | SW10, SW9 (TPS2553) from 3V3_AO | 3.3 V | OBC; EPS controller | 0.04 A; 0.012 A | 0.15 A; 75 mA, latch-up retry (RID AV-08) |

**Change from the drawings (RID EMC-02):** S2 puts the PA and the Jetson on one 5 V buck (≥ 4 A). The Jetson's load
steps would then modulate the PA's supply. Feed the Jetson carrier from VBAT through the eFuse (its own converter on
the carrier) and give the PA its own 5 V buck; U2 then needs 1.2 A, not 4 A. If the carrier must take 5 V, use a
second 5 V buck for it.

Bus voltage under load steps (EMC_ANALYSIS §4): the burn at a cold pack (−20 °C, 55 % SOC) leaves 12.2 V; every
converter's under-voltage lock-out is set at 10.8 V falling (rule E8) so the BMS, not a converter, decides.

## 5. Load switches, current limits and protection

| Switch | Rail in | Enable (firmware pin) | Load | Nominal | Peak | Limit / trip | Part (SQSO-EPS-BRD-001) | Notes |
|--------|---------|-----------------------|------|---------|------|--------------|-------------------------|-------|
| SW1 + U19 | VBAT | PAYLOAD_EN (PB2) on SW1 (AND load shed); JETSON_PWR (PA2) on U19 | Jetson carrier | 0.62 A | 0.93 A (guard) | SW1 2.0 A; U19 eFuse 1.20 A (1.10–1.30) latch-off; flight-software guard 8 W for 2 s and 12 W (`payload_guard.rs`) first | TPS1H100B-Q1 (U20); TPS16630 (U19) | Two different parts on two lines; EPS-F012 shed acts on SW1; readbacks VSW1_RB, VJET_RB to the OBC ADC (RID AV-06) |
| SW2 | 3V3_AO | RADIO_EN (PB5) | Receiver, synthesizer, TNC, exciter (SQSO-RF-BRD-001) | 0.038 A | 0.05 A | 0.12 A (0.10–0.14), retry 1.3 s; software trip on the INA3221 reading | TPS2553-Q1 (U22) | Off only in the RF-silence window (97.207(b)) |
| SW3 | 5V_PA | RF_TX_EN (PA3) AND load shed | PA | 0.30 A | 0.54 A | 0.99 A, retry 0.28 s | TPS2553-Q1 (U23) | Hardware TX-OFF; pull-down mandatory |
| SW4 | 3V3_AO | ADCS_EN (PB6) | ADCS sensors on the OBC (W6B to OBC J6) | 0.04 A | 0.049 A | 0.30 A, retry 0.28 s | TPS2553-Q1 (U24) | Backstop; the OBC's own sensor switches (50–100 mA) see a sensor latch-up (AV-03) |
| SW5 | VBAT | HEATER_EN (PD2) | Battery heater 207 Ω | 0.07 A | 0.08 A (16.4 V) | 0.148 A, F4 0.25 A, 40 °C cutout Q22 in series | TPS1H000-Q1 (U21) | 0.70–1.30 W across the bus |
| SW6 | VBAT | DEPLOY_EN (PF3) | Burn buck U17 → four drivers | 0 | 0.5 A at the pack per burn | 2.0 A; 0.8 A per resistor; 12 s one-shot per driver (§5.1) | TPS1H100B-Q1 (U18) | Deployable control (not counted as an inhibit) |
| SW7 | 3V3_AO | GNSS_EN (PE5) | GNSS receiver | 0.04 A | 0.1 A | 0.20 A, retry 1.3 s | TPS2553-Q1 (U25) | RID EPS-14 closed by the firmware's PE5 |
| SW8 | 3V3_AO | DEPLOY_EN (PF3) | ISIS AntS | 0 | 0.61 A | 1.37 A, retry 0.28 s | TPS2553-Q1 (U26) | Downstream of DS1–DS3 (§8.3) |
| SW9 | 3V3_AO | always | EPS controller | 0.012 A | — | 75 mA, retry 0.28 s | TPS2553-Q1 (U27) | |
| SW10 | 3V3_AO | always | OBC | 0.04 A | — | 0.15 A, retry 0.28 s | TPS2553-Q1 (U38) | The OBC cannot switch its own rail (RID AV-08); its RTC takes 3V3_RTC (3V3_AO through 1 kΩ, RID AV-10) |
| SW11 | VBAT | ADCS_EN (PB6) | OBC magnetorquer H-bridges (VBAT_MTQ, W3 J3-36/39) | 0.083 A | 0.104 A (1.16 W, three rods saturated); 0.23 A at the bridges' limit | 0.36 A (0.32–0.40), auto-retry 40 ms / 1 s | TPS1H000-Q1 (U39) | SQSO-ICX-001 E3 (≥ 0.25 A, ≤ 0.5 A); current telemetry from the OBC's coil-current channels |

TPS22918-class switches are rated 5.5 V: they may switch only the 3.3 V and 5 V rails. Every VBAT switch needs a part
rated ≥ 25 V (derating 16.8 V to ≤ 70 % of the rating, EEE-INST-002).

### 5.1 Deployment drivers and peak battery current

The burn resistors are ≈ 5 Ω. Driven straight from VBAT, each draws 2.9–3.3 A, and the two in parallel on one line
(S5) about 5.8 A, above the 5 A pack fuse: the first wing deployment would open F2 and end the mission (RID SMA-05).
The CDR design:

- **One current-limited driver per burn resistor, four in all** (two resistors per wing; chair decision, RID REL-03):
  OPA2991-Q1 + BUK9M14-40E + 0.25 Ω regulating 0.800 A ± 2.2 % (requirement ± 10 %) for resistors of 4–6.9 Ω
  (SQSO-EPS-BRD-001 §9). The mission simulation's values agree (`sqso-sil/src/hw/deploy.rs`: `BURN_OHMS` 5.0,
  `BURN_LIMIT_A` 0.8).
- **Supply from a 7.0 V burn buck** (U17, behind SW6): the linear drivers dissipate ≤ 3.0 W instead of ≈ 10 W from
  VBAT, the pack sees 0.51 A per burn instead of 0.88 A, and the buck's own limit caps a four-driver software fault
  at 2.0 A at the pack.
- **Hardware on-time limit**: a TPL5111 one-shot per driver, powered by the burn line itself, enables the driver for
  12.0 s ± 0.6 s after the line's rising edge; a stuck-high line burns once, and only a falling edge re-arms it.
- Four burn lines from the OBC: BURN_W1A (PB10), BURN_W1B (PE6), BURN_W2A (PB9, freed by the AntS), BURN_W2B (PE7,
  SQSO-ICX-001). The flight software burns one resistor at a time for 8 s, the second of a wing only if it has not released.

| Deployment case | Burn current | Plus housekeeping and heater | Peak at the pack | Against BMS 4 A / fuse 5 A |
|-----------------|--------------|------------------------------|------------------|----------------------------|
| One resistor, normal sequence (0.88 A at +10 %, from the 7.0 V buck) | 0.51 A at the pack | 0.16 A | **0.67 A** | 6.0× / 7.5× |
| Same, with the magnetorquers at 1.16 W (three rods saturated) | 0.51 A | 0.24 A | 0.75 A (0.90 A with the coil bridges at their 0.23 A limit) | 4.8× / 6.0× |
| AntS antennas (its own controller, 3.3 V on SW8) | ≤ 0.2 A at the pack | 0.16 A | ≤ 0.4 A | 10× |
| Two resistors at once (an A/B line pairing or a software fault) | 1.05 A | 0.2 A | 1.25 A | 3.2× / 4.0× |
| All four drivers on (software fault) | burn buck in current limit | — | **2.0 A** | 2.0× / 2.5× |
| One driver failed short (its resistor straight across V_BURN) | ≤ 1.6 A at 7 V | — | ≤ 1.1 A | the one-shot still ends it in 12 s |

The simulation's 1.63 A (one line of two resistors at 0.8 A) is the A/B pairing drawn from VBAT; with the burn buck it
becomes 1.25 A. Bus voltage during a single-resistor burn at a cold pack (−20 °C, 55 % SOC) is ≈ 13.9 V (15.04 V less
0.67 A through the cold pack's 1.67 Ω; EMC_ANALYSIS §4).

**Pack protection** (details SQSO-BSA-001 §3): F2, non-resettable 5 A, MIL-PRF-23419/12 FM12A72V5.0A (interrupt 1000 A against a prospective ≤ 314 A), with the same part F0 on the battery sense board ahead of the pack leads; RT1 PTC
hold 2.5 A, trip ≤ 5 A (PDU_DESIGN 1.0's 8 A trip equalled the cell's continuous limit); BMS AFE (BQ76920 class)
cell OV 4.25 V, UV 2.8 V, over-current 4 A for 0.5 s (below F2, so the resettable trip acts first), short-circuit trip; secondary OV protector at 4.35 V with a
three-terminal fuse (RID EPS-11); pack over-temperature +60 °C on a hardware comparator; no low-temperature discharge
cut-off (RID EPS-08: a −10 °C cut-off would keep the bus dark after a cold deployment, THERMAL_ANALYSIS §6.7).

**Harness and derating** (SQSO-HAR-001 §5.2, 50 % derating):

| Path | Wire | Derated rating | Current | Verdict |
|------|------|----------------|---------|---------|
| Pack to EPS (W1) | 2 × 20 AWG per pole | 6 A | ≤ 1.8 A; fuse 5 A | Pass (wire above the fuse) |
| Each solar string (W2, H8) | 2 × 26 AWG per pole on two Pico-Clasp contacts (the crimps take 26–32 AWG) | 1.0 A | 0.55 A | Pass (0.28 A per wire; RID EPS-13) |
| Jetson supply from VBAT | 2 × 22 AWG | 4 A | 0.93 A; eFuse 1.2 A | Pass |
| PA 5 V (W4) | 22 AWG | 2 A | 0.54 A; trip 1.0 A | Pass |
| Burn resistors (W7) | 24 AWG each | 1.0 A | 0.88 A limited, ≤ 12 s | Pass |
| Deployment switches DS1, DS2, DS3 | own 3-wire harness each | switch rating | ≤ 8.4 µA (gate drives) | Pass: the switches drive FET gates only (RID EPS-05) |

## 6. Battery management and the low-voltage load shed

The BMS (BATTERY_DESIGN §4) monitors each cell, balances passively, and opens the CHG and DSG FETs on its thresholds.
A **hardware low-voltage disconnect** (EPS-F012) opens SW1 (Jetson) and SW3 (PA) through an AND into their enables
when VBAT falls below 13.6 V (3.4 V/cell) for 5 s, and releases above 14.4 V, independent of the OBC. The heater is
not shed: it protects the pack from cold charging and a stuck-on heater is survivable (SQSO-PBA-001 §8.4); the 5 s
filter rides through the deployment burn (EMC_ANALYSIS §4). It keeps the OBC, the
receiver and TX-OFF alive with a stuck-on Jetson (two switch failures), which otherwise empties the pack in about
41 h and leaves the spacecraft brown-out cycling (SQSO-PBA-001 §8.4). It sits above the flight software's EMERGENCY
threshold (15 % SOC, ≈ 14.0 V at rest) only under load, so in normal operation the software acts first.

**Implemented** (SQSO-EPS-BRD-001 §8.4): TLV7042-Q1 comparator against a 2.500 V reference (13.62 V falling,
14.42 V rising), 4.9 s RC timer and a second comparator, AND gates in SW1's and SW3's enables. The BMS is the
BQ76952 with its thresholds in OTP (BRD-001 §6.1).

## 7. End-of-life charge disconnect (97.207(g)(1)(iii))

F1 sits between the trackers' output and BATT+ (S1), so opening it disconnects the array from the pack **and** from
the bus; the loads then drain the pack to the BMS cut-off and the spacecraft goes dark for good.

| Element | Specification |
|---------|---------------|
| F1 | Three-terminal fuse Littelfuse ITV4030L1412NR (12 A, 36 VDC, heater 11.2–20 Ω, triggerable 10.5–19.6 V) between the trackers' output and PACK_P (SQSO-EPS-BRD-001 §7) |
| Fire circuit | F1's heater to GND through two series FETs: Q21 from the arm latch U61 (set by an EOL_ARM rising edge, cleared at power-up) and Q20 from EOL_FIRE; 9.8–24 W, cut < 1 min. Each line from its own OBC pin and authenticated command (PASSIVATE_ARM, PASSIVATE_EXECUTE of REG-01) |
| Fault tolerance | No single failure (switch short, software fault, latch upset) fires F1: two independent series elements, two commands. A failure to fire leaves the mission's TX-OFF and loss-of-contact inhibit as the controls |
| Verification | V-BSA-7: EM only (destructive) |
| Gap | Hardware done (RID EPS-06). OBC pins EOL_ARM PF4, EOL_FIRE PG1 (SQSO-ICX-001); the flight software's commands and state remain, for the chair (BRD-001 §13.2) |

## 8. Inhibit chains (CDS 2.3.7, 2.3.8; UG v11 §6.7.7)

An inhibit is a physical device between a power source and a hazard (CDS 2.3.7.1); a timer is not (2.3.7.2); the RBF
does not count (BASELINE §4). Sources: the pack (BATT+/BATT−) and the illuminated array (at BATT+ through the
trackers). Elements are listed from the source to the hazard; "independent" means that no single failure, and no
single command or software fault, removes more than one of them.

### 8.1 RF (2 m PA radiating)

The drawings (S1) have two deployment switches and a "third inhibit" left open (finding F5). The CDR design adds a
**third deployment switch DS3** (RID SMA-01 / EPS-04, adopted with the safety, launch and RF reviewers): a separate
switch on a third rail standoff driving a second high-side FET in VBAT+, with a second contact that holds every
tracker's enable low. Each switch drives its own FET gate (§8.4).

| # | Element | Type | Breaks | Independent of the others because |
|---|---------|------|--------|-----------------------------------|
| — | F2, RT1, BMS FETs | Protection | Pack | Closed in normal storage: not inhibits |
| — | RBF pin | GSE | Pack and array | Not counted |
| **1** | **DS2** → low-side FET in the pack return (PACK− to GND) | Mechanical separation signal | Pack | Own switch, rail, gate drive and wiring, in the negative leg |
| **2** | **DS1** → high-side FET in VBAT+ | Mechanical separation signal | Pack and array | Own switch on a diagonally opposite rail, positive leg |
| **3** | **DS3** → second high-side FET in VBAT+, and the trackers' enables | Mechanical separation signal | Pack and array (the only switch that also stops the array charging the pack and the MPPTs running) | Own switch on a third rail; its FET and DS1's are separate parts with separate gate drives, and their wiring is routed apart so one short cannot bridge both |
| (4) | SW3, PA supply switch (RF_TX_EN, PA3), off at reset | Load switch | Pack and array | Needs a software command after all three switches have closed; counted only as margin |
| — | SW2 (RADIO_EN), hardware TX permit into the PTT gate (BUILD_VS_BUY §3.2), 45-min RF timer | Same controller as SW3, or a timer | — | Not counted |

**Three independent inhibits for the pack and for the array, all from the mechanical separation signal** (UG v11
§6.7.7: "inhibited from a mechanical separation signal, and not because of other factors such as software delays"),
with SW3 as a fourth. Common cause between the three switches: they are released together by design at ejection; an
early release needs the spacecraft to move off three rails at once, so they sit on three different rails. The same
three switches give the NRCSD backup its ground-leg inhibit.

### 8.2 Deployables (solar wings; burn wires on SW6)

| # | Element | Breaks |
|---|---------|--------|
| **1** | DS2 | Pack |
| **2** | DS1 | Pack and array |
| **3** | DS3 | Pack and array |
| (4) | SW6 (DEPLOY_EN, PF3, high side) and the wing's low-side driver (one per wing, §5.1) | Both; counted as one, because one OBC drives both lines |
| — | Hardware on-time limit (§5.1), 30-minute timer | Not counted (timers) |

Three independent inhibits, plus the OBC-driven pair. The 30-minute timer (CDS 2.4.4) is in the software and does not
count.

### 8.3 Antennas (ISIS AntS, bought)

BUILD_VS_BUY §2.6 buys the ISIS AntS with its own release controller (two microcontrollers, an ARM command, thermal
knives). The chain is DS2, DS1, DS3, the AntS supply switch SW8 (DEPLOY_EN, downstream of the three switches) and the
AntS's own ARM switch: three independent. S5 and W7 are redrawn (RID EPS-07, SQSO-EPS-BRD-001 §9); ANTENNA_BURN (PB9)
is freed and becomes BURN_W2A. Still open: confirm from the AntS ICD which internal switches are physical devices
between its supply and its knives.

### 8.4 Requirements on the switches

- Each deployment switch drives a FET gate through an RC filter (≥ 10 ms) rather than carrying the bus current, so
  contact chatter during random vibration cannot power the OBC (UG v11 §6.7.7 asks for evidence of no inadvertent
  power-up) and the switches need no current rating (EPS-05).
- Toggling any deployment switch resets the timers (CDS 2.3.2.4): the OBC loses power, and the timers restart at
  boot.
- Pack-to-frame isolation ≥ 1 MΩ on both poles (a pack-negative-to-frame short would bypass DS2; SQSO-BSA-001 §5).
- No ground-referenced circuit on the pack-negative node, and ≥ 100 kΩ (or the I2C isolator) on every signal that
  crosses from the pack domain, so that no sneak path bypasses DS2 (SQSO-EPS-BRD-001 §6.4); DS2 is two FETs back to
  back so it blocks charge as well as discharge.
- Storage drain with the switches actuated ≤ 20 µA in all (LVR-051/071 of `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`):
  designed 2.3 µA typical, 7.7 µA worst with the AFE in SHUTDOWN, including the access-port board's UMB_VMON TVS
  (BRD-001 §6.6).

## 9. Telemetry

The controller's register block (SMART-QSO-ICD-001 v1.1 §4.2) carries pack voltage and current, temperature, SOC,
array voltage and current, the payload rail and status bits. Added at CDR (ICD §4.6): each cell voltage, the second
pack temperature, per-tracker input power, the charge-inhibit cause (cold, hot, OV, BMS), the load-shed and
passivation-latch states, and the heater current. Implemented (SQSO-EPS-BRD-001 §11): five analog lines to the OBC
(VBATT 1:6, IBATT 0.5 V/A, VSOLAR 1:4, ISOLAR 1 V/A, VBUS 1:2), two INA228 on I2C2 (pack gauge 0x44, array 0x40)
readable without the EPS controller, and the controller's block at 0x48 fed by the AFE, four INA3221 and an ADS7138.

## 10. Verification

| Item | Method | Procedure |
|------|--------|-----------|
| Power closure | Analysis, then EM flatsat measurement | SQSO-PBA-001 V-PWR-1 … 5 |
| Trackers and regulator | Test | TP-EPS-004 (MPPT efficiency), V-PWR-3 |
| Battery protection and screening | Test | SQSO-BSA-001 V-BSA-1 … 8; TP-EPS-001, TP-EPS-003 |
| Rails, switches, limits | Test: each limit tripped, each enable at reset | TP-EPS-002 |
| Inhibits | Inspection of the drawings against §8; inhibit-by-inhibit continuity test; test during random vibration at the flight SOC | SYS-704; V-BSA-6 |
| EOL disconnect | Test on the EM (destructive) | V-BSA-7 |
| Thermal | EM thermal balance | SQSO-THA-001 V-THM-1 … 7 |
| EMC | Unit and system tests | SQSO-EMC-001 V-EMC-1 … 7 |

## 11. Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| EPS-R001 | Battery degradation faster than expected | L | M | 4.10 V set point (D8); shallow DoD (≤ 4.5 %); LEO-profile cell test (V-BSA-8) |
| EPS-R002 | Tracker efficiency below 92 % at low input voltage | M | M | V-PWR-3 early on the EM; four trackers |
| EPS-R003 | Wing deployment failure | L | H | SAFE and IDLE close on 1.47 W (SQSO-PBA-001 §8.1); EPS-10 mode rule |
| EPS-R004 | Stuck-on payload | L | H | Two switches on two lines and the hardware load shed (§6) |
| EPS-R005 | Single-event latch-up in the EPS controller | L | M | Its own latch-up-tripping supply SW9 with auto-retry; no protection depends on it (BRD-001 §12) |
| EPS-R006 | Hot dispenser soak above +60 °C on the pack | M | M | Integrator analysis (V-THM-5); delivery SOC 60 ± 5 % (LVR-051) |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | SMART-QSO Team | §8.2 test IDs aligned with the procedure files (finding VV-18) |
| 1.2 | 2026-10-16 | Funding audit | Generation, solar cells and BOM aligned with SOLAR_ARRAY_DESIGN v2.0 (TB-14); battery DoD, sizing and charge-temperature rules aligned with BASELINE §5 and BATTERY_DESIGN v2.0 (TB-08) |
| 1.3 | 2026-10-16 | SMART-QSO Team | §2.4 vibration: interim SpaceX UG v11 level of TP-ENV-002 (≥ 5.57 Grms MPE), replacing 14.1 Grms (finding LA-05) |
| 2.0 | 2026-10-27 | CDR review, EPS/thermal | Rewritten (including, after the safety and reliability review: third deployment switch DS3, one current-limited burn driver per wing with a hardware on-time limit and the peak-current table, latched recharge lockout, BMS over-current 4 A below the 5 A fuse) — for the built 4S EPS: string layout and four trackers, bus and rails, load switches with limits and derating, protection, hardware load shed, end-of-life disconnect, inhibit chains with their independence; the module-based design of 1.x withdrawn |
| 2.1 | 2026-10-27 | CDR design (EPS board) | Parts selected and the board designed in SQSO-EPS-BRD-001 A (`hardware/electrical/eps/`): charge regulator (§3.3), rails incl. V_BURN, 3V3_OBC, 3V3_MCU (§4), switch table with parts and latch-up trips incl. the OBC rail (§5, RID AV-08; radio 0.12 A), four burn drivers on a 7.0 V burn buck with the peak-current table (§5.1), load shed, EOL fuse and pins, AntS chain, sneak-path rule and drain figures (§6–§9) |
| 2.2 | 2026-10-27 | CDR design (EPS board) | Interconnect of SQSO-ICX-001 §7 (SQSO-EPS-BRD-001 B): SW11 magnetorquer supply on ADCS_EN, SW4 sensors only, Jetson readbacks, 3V3_RTC, OBC pins PB2 (PAYLOAD_EN), PF3 (DEPLOY_EN), PE7 (BURN_W2B), PF4/PG1 (EOL), burn peak-current row with the magnetorquers |
| 2.3 | 2026-10-27 | CDR design (EPS board) | Pack fuse F2 (and F0 on the battery sense board) FM12A72V5.0A for breaking capacity; storage drain 7.7 µA worst with the access-port board's TVS (SQSO-EPS-BRD-001 C) |
