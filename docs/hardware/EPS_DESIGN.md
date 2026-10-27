# Electrical Power Subsystem (EPS) Design Document

**Document Number**: SMART-QSO-EPS-001
**Version**: 2.0
**Date**: 2026-10-27
**Status**: CDR design description (build decision, parts not yet selected where stated)

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
| EPS-F002 | Track the maximum power point of each string group of §3.2 with ≥ 92 % conversion efficiency over 3.5–6.4 V input | T | Part not yet selected |
| EPS-F003 | Store ≈ 50 Wh (4S1P INR18650-35E); charge CC ≤ 0.40 A, CV 4.10 V/cell (D8), only between 0 and +45 °C on the pack temperature, in hardware | T | Design |
| EPS-F004 | Supply the loads from the array while charging is inhibited (power-path regulation of the battery bus) | T | Design requirement added at CDR (§3.3) |
| EPS-F005 | Battery bus 11.2–16.4 V (16.8 V absolute maximum) to all VBAT loads; every converter rated ≥ 25 V input | T | §4 |
| EPS-F006 | 3V3_AO rail, always on (OBC, FRAM, receiver path, ADCS sensors) | T | §4 |
| EPS-F007 | Switched rails with the current limits of §5, every switch off at reset (pull-down) | T | §5 |
| EPS-F008 | Hardware TX-OFF: the PA supply switch (SW3) removed by RF_TX_EN low, independent of the AI payload (BASELINE §7) | T | §5, §8.1 |
| EPS-F009 | At least three independent physical inhibits each for RF and for every deployable (CDS 2.3.7, 2.3.8); timers and the RBF do not count | I, T | §8: DS1, DS2 and DS3 (DS3 added at CDR) |
| EPS-F010 | Non-resettable end-of-life charge-path disconnect (47 CFR 97.207(g)(1)(iii)), fired only by two independent signals | T | §7 |
| EPS-F011 | Battery protection with the fault tolerance of SQSO-BSA-001 §3 | A, T | §6 |
| EPS-F012 | Independent hardware low-voltage disconnect of the payload and PA rails | T | Added at CDR (§6, RID EPS-09) |
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

Candidates (not yet selected; each to be checked against the radiation, temperature and derating rules): a
multi-input buck-boost charger of the TI BQ25798 class (3.6–24 V input, 1–4S, MPPT, TS input, power path) per tracker
group; or discrete boost MPPT stages into one 4S CC/CV regulator. The BQ25570/SPV1040 class of EPS_ELECTRICAL_DESIGN
1.0 cannot charge a 4S pack.

## 4. Battery bus and rails

| Rail | Source | Voltage | Loads | Max steady / peak | Converter rating (≥ 2× load, ≥ 25 V in) |
|------|--------|---------|-------|-------------------|------------------------------------------|
| VBAT (unregulated) | Pack through F2, RT1, RBF, DS1, inhibit chain | 11.2 (BMS cut) … 16.4 V; 16.8 V abs max; 17.0 V at the 4.25 V OV threshold | Heater, burn wires, Jetson carrier (recommended), bucks | 1.0 A / 1.8 A (burn + housekeeping + heater) | — |
| 3V3_AO | Buck U1 from VBAT, always on | 3.3 V ± 3 % | OBC, FRAM, receiver/TNC/exciter (through SW2 and its own LDO, EMC rule E4), ADCS (SW4), GNSS (SW7) | 0.26 A / 0.30 A | ≥ 0.6 A, 25 V |
| 5V_PA | Buck U2 from VBAT | 5.0 V ± 3 % | PA through SW3 | 0.54 A at 1 W RF | ≥ 1.2 A, 25 V |
| VBAT_JET | eFuse U3 from VBAT (S7), carrier converter to the module's 5 V | VBAT | Jetson carrier | 0.62 A at 8 W; 0.93 A at the 12 W guard trip | eFuse ≥ 23 V operating |

**Change from the drawings (RID EMC-02):** S2 puts the PA and the Jetson on one 5 V buck (≥ 4 A). The Jetson's load
steps would then modulate the PA's supply. Feed the Jetson carrier from VBAT through the eFuse (its own converter on
the carrier) and give the PA its own 5 V buck; U2 then needs 1.2 A, not 4 A. If the carrier must take 5 V, use a
second 5 V buck for it.

Bus voltage under load steps (EMC_ANALYSIS §4): the burn at a cold pack (−20 °C, 55 % SOC) leaves 12.2 V; every
converter's under-voltage lock-out is set at 10.8 V falling (rule E8) so the BMS, not a converter, decides.

## 5. Load switches, current limits and protection

| Switch | Rail in | Enable (firmware pin) | Load | Nominal | Peak | Limit / trip (≈ 1.5–2 × peak) | Part class | Notes |
|--------|---------|-----------------------|------|---------|------|-------------------------------|------------|-------|
| SW1 + U3 | VBAT | PAYLOAD_EN (PB4) on SW1; JETSON_PWR (PA2) on U3 | Jetson carrier | 0.62 A | 0.93 A (guard) | U3 eFuse 1.2 A, latch-off, ≤ 1 ms; flight-software guard 8 W for 2 s and 12 W latch-up (`payload_guard.rs`) | SW1: high-side switch rated ≥ 25 V (TPS1H100 class); U3: TPS25947 class (23 V) | Two different parts on two different lines (BUILD_VS_BUY §7.3); EPS-F012 shed acts on SW1 |
| SW2 | 3V3_AO | RADIO_EN (PB5) | Receiver, TNC, exciter | 0.05 A | 0.1 A | 0.25 A | TPS22918 | Never off in normal modes (97.207(b)); off only in the RF-silence window |
| SW3 | 5V_PA | RF_TX_EN (PA3) | PA | 0.30 A | 0.54 A | 1.0 A | TPS22918 | Hardware TX-OFF; pull-down mandatory; RF inhibit 3 (§8.1) |
| SW4 | 3V3_AO | ADCS_EN (PB6) | Sensors, gyro, magnetorquer drivers | 0.04 A | 0.13 A | 0.3 A | TPS22918 | |
| SW5 | VBAT | HEATER_EN (PD2) | Battery heater ≈ 200 Ω, two elements | 0.07 A | 0.08 A (16.4 V) | **0.15 A** (was 0.5 A on S2) plus a series fuse 0.25 A and the +40 °C cutout | High-side ≥ 25 V | 1 W at 14.4 V; 0.95 W at 14.0 V (THERMAL_ANALYSIS §2.3) |
| SW6 | VBAT | DEPLOY_EN (PA1) | Burn-wire drivers | 0 | 0.88 A per resistor, one resistor at a time; 3.4 A worst fault | 3.6 A (above the worst software-fault burn load of 3.36 A, below the BMS 4 A and F2), plus 0.8 A per-resistor limit and a 12 s hardware on-time limit (§5.1) | High-side ≥ 25 V | Deployable inhibit (§8.2) |
| SW7 | 3V3_AO | GNSS_EN (pin open: PE4 is XPDR_EN in the firmware, RID EPS-14) | GNSS receiver | 0.04 A | 0.1 A | 0.15 A with latch-up trip | TPS22918 | |

TPS22918-class switches are rated 5.5 V: they may switch only the 3.3 V and 5 V rails. Every VBAT switch needs a part
rated ≥ 25 V (derating 16.8 V to ≤ 70 % of the rating, EEE-INST-002).

### 5.1 Deployment drivers and peak battery current

The burn resistors are ≈ 5 Ω. Driven straight from VBAT, each draws 2.9–3.3 A, and the two in parallel on one line
(S5) about 5.8 A, above the 5 A pack fuse: the first wing deployment would open F2 and end the mission (RID SMA-05).
The CDR design:

- **One current-limited driver per burn resistor**, 0.8 A ± 10 % (a linear limit or a constant-current sink; finding
  F7), the value the mission simulation uses (`sqso-sil/src/hw/deploy.rs`: `BURN_OHMS` 5.0, `BURN_LIMIT_A` 0.8; peak
  battery current in its runs 1.63 A with one line burning).
- **One driver per wing** (W1, W2) instead of one for both (RID REL-03: the shared driver's common cause gives
  P(both wings stowed) ≈ 1.6 %, separate drivers ≈ 0.1 %). This needs a second wing burn line in the pin map (for the
  chair; PB10 serves one today). The antenna knives are the AntS's own (§8.3).
- **Hardware on-time limit**: a monostable on each driver's enable, 12 s (the simulated burns release in ≤ 4 s;
  thermal-knife practice is ≤ 10 s), re-armed only by a falling edge of DEPLOY_EN. A stuck OBC line then cannot burn
  for more than 12 s per cycle.
- The flight software burns the lines one at a time (antennas, then W1, then W2).

| Deployment case | Burn current | Plus housekeeping and heater | Peak at the pack | Against BMS 4 A / fuse 5 A |
|-----------------|--------------|------------------------------|------------------|----------------------------|
| One wing (W1 or W2), normal sequence | 0.8 A (0.88 A at +10 %) | 0.13 A | **1.0 A** | 4.0× / 5.0× |
| AntS antennas (its own controller; to be confirmed from the AntS ICD) | ≤ 1.6 A allocation | 0.13 A | ≤ 1.7 A | 2.3× / 2.9× |
| Both wings together (software fault) | 1.76 A | 0.13 A | 1.9 A | 2.1× / 2.6× |
| Everything at once (software fault: both wings and AntS) | 3.36 A | 0.13 A | **3.5 A** | 1.14× / 1.4×: the BMS trips before the fuse would |
| One current limiter failed short, one wing | 3.3 A (16.4 V / 5 Ω) | 0.13 A | 3.4 A | Below both; the on-time limit ends it in 12 s |
| Two limiters failed short (two failures) | 6.6 A | — | 6.7 A | BMS trips (resettable); the fuse is the backup |

The simulation's 1.63 A (one line of two resistors at 0.8 A) and this table agree. Bus voltage during a single-wing
burn at a cold pack (−20 °C, 55 % SOC) is ≈ 13.4 V (15.04 V less 1.0 A through the cold pack's 1.67 Ω; EMC_ANALYSIS §4).

**Pack protection** (details SQSO-BSA-001 §3): F2, non-resettable 5 A, breaking capacity ≥ 100 A at 32 VDC; RT1 PTC
hold 2.5 A, trip ≤ 5 A (PDU_DESIGN 1.0's 8 A trip equalled the cell's continuous limit); BMS AFE (BQ76920 class)
cell OV 4.25 V, UV 2.8 V, over-current 4 A for 0.5 s (below F2, so the resettable trip acts first), short-circuit trip; secondary OV protector at 4.35 V with a
three-terminal fuse (RID EPS-11); pack over-temperature +60 °C on a hardware comparator; no low-temperature discharge
cut-off (RID EPS-08: a −10 °C cut-off would keep the bus dark after a cold deployment, THERMAL_ANALYSIS §6.7).

**Harness and derating** (SQSO-HAR-001 §5.2, 50 % derating):

| Path | Wire | Derated rating | Current | Verdict |
|------|------|----------------|---------|---------|
| Pack to EPS (W1) | 2 × 20 AWG per pole | 6 A | ≤ 1.8 A; fuse 5 A | Pass (wire above the fuse) |
| Each solar string (W2, H8) | 24 AWG | 1.0 A | 0.55 A | Pass. SOLAR_ARRAY_DESIGN §4.2's 26 AWG (0.5 A) fails by 10 % (RID EPS-13) |
| Jetson supply from VBAT | 2 × 22 AWG | 4 A | 0.93 A; eFuse 1.2 A | Pass |
| PA 5 V (W4) | 22 AWG | 2 A | 0.54 A; trip 1.0 A | Pass |
| Burn resistors (W7) | 24 AWG each | 1.0 A | 0.8 A limited | Pass |
| Deployment switches DS1, DS2 | — | switch rating | 1.8 A if they carry the bus | **Rating not specified** (RID EPS-05): use the switches to drive FET gates (µA), or a switch rated ≥ 3.6 A DC |

## 6. Battery management and the low-voltage load shed

The BMS (BATTERY_DESIGN §4) monitors each cell, balances passively, and opens the CHG and DSG FETs on its thresholds.
A **hardware low-voltage disconnect** (EPS-F012) opens SW1 (Jetson) and SW3 (PA) through an AND into their enables
when VBAT falls below 13.6 V (3.4 V/cell) for 5 s, and releases above 14.4 V, independent of the OBC. The heater is
not shed: it protects the pack from cold charging and a stuck-on heater is survivable (SQSO-PBA-001 §8.4); the 5 s
filter rides through the deployment burn (EMC_ANALYSIS §4). It keeps the OBC, the
receiver and TX-OFF alive with a stuck-on Jetson (two switch failures), which otherwise empties the pack in about
41 h and leaves the spacecraft brown-out cycling (SQSO-PBA-001 §8.4). It sits above the flight software's EMERGENCY
threshold (15 % SOC, ≈ 14.0 V at rest) only under load, so in normal operation the software acts first.

## 7. End-of-life charge disconnect (97.207(g)(1)(iii))

F1 sits between the trackers' output and BATT+ (S1), so opening it disconnects the array from the pack **and** from
the bus; the loads then drain the pack to the BMS cut-off and the spacecraft goes dark for good.

| Element | Specification |
|---------|---------------|
| F1 | One-shot thermal cut-off (TCO) rated ≥ 1 A at 32 VDC, bonded to a 20–50 Ω heater resistor; or a three-terminal fuse whose heater element is driven the same way. Not a fuse blown by battery current (that would need a crowbar on the pack) |
| Fire circuit | Heater resistor from VBAT through two series switches: EOL_ARM (a hardware latch set by an authenticated ARM_PASSIVATION telecommand) and EOL_FIRE (a second GPIO, a second authenticated command, ≥ 1 orbit later) |
| Fault tolerance | No single failure (switch short, software fault, latch upset) fires F1: two independent series elements, two commands. A failure to fire leaves the mission's TX-OFF and loss-of-contact inhibit as the controls |
| Verification | V-BSA-7: EM only (destructive) |
| Gap | The flight software has no command, pin or state for it; the mission simulation fires it directly on the end-of-life TX_DISABLE (`mission.rs`). RID EPS-06 for the chair (flight software and pin map) |

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
| (4) | SW6 (DEPLOY_EN, PA1, high side) and the wing's low-side driver (one per wing, §5.1) | Both; counted as one, because one OBC drives both lines |
| — | Hardware on-time limit (§5.1), 30-minute timer | Not counted (timers) |

Three independent inhibits, plus the OBC-driven pair. The 30-minute timer (CDS 2.4.4) is in the software and does not
count.

### 8.3 Antennas (ISIS AntS, bought)

BUILD_VS_BUY §2.6 buys the ISIS AntS with its own release controller (two microcontrollers, an ARM command, thermal
knives). S5 and W7 still draw the antenna burn on the OBC's ANTENNA_BURN driver. With AntS the chain is DS2, DS1, DS3, the
AntS supply switch (SW6 or a dedicated switch), and the AntS's own ARM switch: at least three independent, as long as
the AntS supply is taken downstream of the three switches. **RID EPS-07:** redraw S5/W7 for AntS and confirm from the AntS ICD which internal switches are
physical devices between its supply and its knives.

### 8.4 Requirements on the switches

- Each deployment switch drives a FET gate through an RC filter (≥ 10 ms) rather than carrying the bus current, so
  contact chatter during random vibration cannot power the OBC (UG v11 §6.7.7 asks for evidence of no inadvertent
  power-up) and the switches need no current rating (EPS-05).
- Toggling any deployment switch resets the timers (CDS 2.3.2.4): the OBC loses power, and the timers restart at
  boot.
- Pack-to-frame isolation ≥ 1 MΩ on both poles (a pack-negative-to-frame short would bypass DS2; SQSO-BSA-001 §5).
- Storage drain with the switches actuated ≤ 20 µA in all (LVR-051/071 of `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`):
  the BMS AFE in ship mode (≤ 1 µA), the secondary protector (≈ 1 µA), the gate-drive pull-ups ≤ 5 µA.

## 9. Telemetry

The controller's register block (SMART-QSO-ICD-001 v1.1 §4.2) carries pack voltage and current, temperature, SOC,
array voltage and current, the payload rail and status bits. Added at CDR (ICD §4.6): each cell voltage, the second
pack temperature, per-tracker input power, the charge-inhibit cause (cold, hot, OV, BMS), the load-shed and
passivation-latch states, and the heater current.

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
| EPS-R005 | Single-event latch-up in the EPS controller | L | M | Watchdog; the BMS and the load shed do not depend on it |
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
