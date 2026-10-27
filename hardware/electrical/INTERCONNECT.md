# SMART-QSO Spacecraft Electrical Interconnect

**Document ID**: SQSO-ICX-001
**Version**: B
**Date**: 2026-10-27
**Status**: CDR build-to interconnect: master pin allocation of the OBC's STM32L4R5ZIT6 and the board-to-board signal
list. Generated in part from the OBC schematic source (`hardware/electrical/obc/design.py` → `obc_pinmap.csv`); where
this document and the source disagree, the source is right and this document is regenerated. Nothing here has been
built or measured.

**Owners**: OBC board and interconnect (this document, [OBC_BOARD_DESIGN.md](obc/OBC_BOARD_DESIGN.md), SQSO-OBC-BRD-001);
EPS board [EPS_BOARD_DESIGN.md](eps/EPS_BOARD_DESIGN.md) (SQSO-EPS-BRD-001); RF board
[RF_BOARD_DESIGN.md](rf/RF_BOARD_DESIGN.md) (SQSO-RF-BRD-001). Harness drawings: `hardware/diagrams/wiring/W*.yml`
(WireViz; W3, W5, W6, W8, W9 version 2 from this document; W4, W7, W10 from the RF and EPS boards).

**Governing documents**: `docs/BASELINE_PARAMETERS.md` Rev D (governs every value); `docs/interfaces/ICD_OBC_EPS.md`
(SMART-QSO-ICD-001), `ICD_OBC_JETSON.md` (SMART-QSO-ICD-002), `ICD_OBC_RF.md` (SQSO-ICD-003 2.1), `ICD_OBC_ADCS.md`
(SQSO-ICD-004); `hardware/harness/HARNESS_DESIGN.md` (SQSO-HAR-001: connector families §3.1, wire derating §5.2).

---

## 1. Purpose and Rules

This document fixes, for every electrical signal that leaves a board: the connector and pin at both ends, the wire
gauge, the direction, the logic or analog level, the state at reset, and, at the OBC, the STM32L4R5ZIT6 pin and
peripheral. It closes RID AV-01 (no master pin map; the firmware's provisional map in conflict with OBC_DESIGN) and
gives RIDs AV-03, AV-05, AV-06, AV-09, AV-10, RF-21 and EPS-14 their pins.

Rules applied to the allocation:

1. **Every function the flight software uses has a pin; no pin has two functions** (checked: §4.4).
2. **Safe at reset.** Every OBC output that enables power, a burn, a transmitter or passivation is high-impedance at
   reset (STM32 analog mode) and pulled to its safe level on the receiving board (EPS: 100 kΩ to GND; RF: 100 kΩ to
   GND). No such output is on a pin whose reset state has a pull-up: PA13, PA15 and **PB4** (NJTRST, internal pull-up at
   reset) carry nothing that enables anything. The firmware's PAYLOAD_EN on PB4 would have switched the Jetson's SW1 on
   at every OBC reset until the firmware reconfigured the pin (the 100 kΩ EPS pull-down loses to the 25–55 kΩ internal
   pull-up); it moves to PB2.
3. **Independence by port.** The two lines of every two-step hazard are on different GPIO ports, so a single upset
   of one port's output register (or one bad write to it) cannot assert both: PAYLOAD_EN PB2 / JETSON_PWR PA8 (RID
   AV-06); EOL_ARM PF4 / EOL_FIRE PG1 (passivation); DEPLOY_EN PF3 / burn lines PB9, PB10, PE6, PE7; RF_TX_EN PA3 /
   XPDR_EN PE4.
4. **No back-power.** No OBC output drives a board or sensor whose supply may be off unless it passes a part with
   Ioff/VCC isolation powered from that supply (§5); lines from boards that stay powered while the OBC is off carry
   series resistance that limits the current into the unpowered OBC to 2.7 mA in total (OBC_BOARD_DESIGN §16).
5. **Wire gauge** per SQSO-HAR-001 §5.2 with 50 % derating: 28 AWG carries ≤ 0.25 A, 26 AWG ≤ 0.5 A, 24 AWG ≤ 1 A,
   22 AWG ≤ 2 A, 20 AWG ≤ 3 A.

## 2. Boards, Connectors and Harnesses

| Harness | From (board, connector) | To (board, connector) | Contents | Connector parts | Gauge |
|---------|-------------------------|-----------------------|----------|-----------------|-------|
| W1 | Battery pack | EPS J1, J2 | pack ±, NTC1, heater; cell taps, NTC2 | Micro-Fit 2×4, Pico-Clasp 6 | 20 AWG (EPS) |
| W2 | Panels and wings | EPS J4–J9 | strings, panel NTCs | Pico-Clasp 5/8 | 24 AWG (EPS) |
| **W3** v2 | **EPS J3** | **OBC J1** | 3V3_OBC, I2C2, 15 enables, 4 burn lines, passivation, lockout clear, sense and analog lines, readbacks, VBAT_MTQ, 3V3_RTC (§3.1) | Samtec FTSH-120-01-L-DV-K both ends, FFSD-20-D ribbon | 28 AWG |
| W4A v3 | OBC J2 | RF J1 | KISS UART, TX_PWR_SEL, XPDR_EN, three readbacks, three monitors, RX_CD, RX_FAULT (§3.2) | FTSH-108-01-L-DV-K both ends | 28 AWG |
| W4B v3 | EPS J19 | RF J2 at (−26, −44) | RADIO_3V3 (SW2), PA_5V (SW3), each with its return as a twisted pair | Micro-Fit 2×3 / Pico-Clasp 6 | 24 AWG twisted pairs |
| **W5** v3 | EPS J18; OBC J3 | Jetson carrier | VBAT_JET (W5A); USART1, the carrier's 3.3 V and the spreader NTC (W5B, 6 pins, E10) (§3.3) | Micro-Fit 2×2; Pico-Clasp 6 | 22 AWG; 28 AWG |
| **W6** v3 | EPS J20; OBC J7, J8, J10 | OBC J6; rods; panel photodiodes; magnetometer board A2 | 3V3_ADCS (W6B), three CR0002 rods (W6C), six sun sensors (W6D), magnetometer (W6E, 3V3_SNS, I2C, DRDY) (§3.4) | Pico-Clasp 2 / 6 / 14 / 6 | 26 / 26 / 28 AWG shielded / 28 AWG |
| W7 | EPS J10, J11, J12 | wing hold-downs, AntS | burn resistors, hinge switch, AntS 3.3 V and I2C | Micro-Fit 2×3, Pico-Clasp 4 | 24 AWG (EPS) |
| **W8** v3 | OBC J9 → access-window bulkhead (W8A, internal) → GSE (W8B) | GSE | SWD, NRST, console, umbilical sense, watchdog disable (§3.6) | Samtec FTSH-105 on the OBC; 2×5 2.54 mm bulkhead at the window | 28 AWG |
| **W9** v2 | EPS J21 → OBC J4; OBC J5 | GNSS receiver | 3V3_GNSS through the OBC; LPUART1, 1PPS (§3.5) | Pico-Clasp 2 / 6 | 26 AWG |
| W10 | RF J3–J6 | AntS | 2 m and 70 cm coax | SMA | coax (RF) |
| W11 (proposed by the EPS board) | EPS J13 | GSE (access window) | charge input, pack monitor, AFE shutdown | Micro-Fit 2×3 | 20 AWG (EPS) |

The OBC and EPS boards are PC/104 boards in one stack with no stack connector (SQSO-EPS-BRD-001 §1); W3 is a ribbon
between their FTSH headers, which sit on the same +X edge at |y| < 24 mm so the ribbon passes the +X wall gap
(MECHANICAL_DESIGN §4.4). Every other harness leaves the stack in the ±X wall gaps (bundles at |y| = 24–34 mm, except the
(+X, −Y) bundle, below). **W4A** runs flat and vertical, without a fold, in the +X wall gap at y −28.5 … −38.5 mm,
from OBC J2 straight up to RF J1 at (+41.0, −33.5) (SQSO-RF-BRD-001 B §15 item 14; both pin fields along Y at the
same x and y). It runs flat on the +X shear panel. The (+X, −Y) bundle cannot move clear of it over its whole height (the
Y rod's −Y end and the +X spreader flange are in the way, MECHANICAL_DESIGN §6.9): it keeps y −24.0 … −33.9 below
z −26 and runs at **y −20.5 … −27.0, a 3 × 6.5 mm section (66 % of a bundle), above z −26**, 1.5 mm from W4A. A 45°
fold of the ribbon was rejected (two ribbon thicknesses and a crease in the 3.3 mm gap). The bundle contents are
allocated in §2.1. The gyro module (Epson M-G370PDT0) plugs into socket X1 on the OBC at (−20, +30.5) mm and stands
through a window in the Jetson carrier; the backup gyro, the sun-sensor amplifiers and the magnetorquer drivers are on
the OBC board; the RM3100 magnetometer is on a small board (A2) at the +Z end frame on harness W6E (SQSO-OBC-BRD-001
§9.4). There is no ADCS board and no ADCS stack connector.

### 2.1 Harness bundle allocation (Version B)

Four bundles in the ±X wall gaps (MECHANICAL_DESIGN §6.9). Allowance: 60 % of the section filled with wire (18 mm² in a
3 × 10 mm bundle, 11.7 mm² in the 3 × 6.5 mm (+X, −Y) section above z −26). Wire areas for thin-wall ETFE (28 / 26 / 24
/ 22 / 20 AWG: 0.49 / 0.62 / 0.85 / 1.17 / 1.58 mm²). Segments: B deck to EPS (z −26 … −1), C EPS to OBC (−1 … +6.6),
D OBC to carrier (+6.6 … +18), E carrier to the +Z end frame (+18 … +48). Fills are of the harnesses with an OBC end
(this document); the EPS-only harnesses (W1, W2, W7, W11) take the remainder, placed and counted by the harness design.

| Bundle | Harnesses with an OBC end (segments) | ICX fill per segment | Left for the EPS-only harnesses | Version B change |
|--------|--------------------------------------|----------------------|---------------------------------|------------------|
| (+X, +Y), y +24 … +34 | W5B (D), W6E (D, E), **W9A** OBC J5 → receiver (C, B, then on the deck to (+5, −20)), **W9B** EPS J21 → OBC J4 (C) | B 3.7, C 5.0, D 5.9, E 2.9 mm² (21–33 %) | 12–15 mm² per segment: the wing-1 (+X) harness of W2 from the +Y hinge end, and whatever leaves the (+X, −Y) bundle | **W9A, W9B move here** from (+X, −Y); OBC J4/J5 to the +X edge at y +14 … +24, EPS J21 to its +X edge, +Y side (for the EPS designer) |
| (+X, −Y), y −20.5 … −27.0 above z −26; −24.0 … −33.9 below | none (W4A is on the shear panel beside it, not in the bundle) | 0 | **11.7 mm² above z −26**, 18 below: W7 to the wing-1 hold-down at the −Y free corner (it has to be on this side); nothing else above z −26 | GNSS lines moved out; any other EPS-only wires the harness design had here move to (+X, +Y) |
| (−X, −Y), y −24 … −34 | W4B EPS J19 → RF J2 (C, D, E), W5A EPS J18 → carrier J1 (C, D), W8A OBC J9 → access bulkhead (C, B, A) | C 14.7 (82 %), D 9.8, E 5.1, B 4.9 mm² | B 13.1 mm²: W7 to the wing-2 hold-down; W11 (6 × 20 AWG, 9.5 mm²) fits only with W7 at ≤ 3.6 mm², otherwise W11 goes down the (+X, −Y) bundle below z −26 after crossing on the EPS | none for the ICX harnesses (mechanical routes kept) |
| (−X, +Y), y +24 … +34 | W6B EPS J20 → OBC J6 (C), W6C rod pairs from J7 (C, B; the Z-rod pair goes direct to the rod's top at z +3), W6D sun-sensor trunk to J8 (C, B; about 9.6 mm² shielded) | C 13.3 (74 %), B 12.1 mm² | B 5.9 mm²: the battery flying leads (W1, about 16 mm² as 8 × 20 AWG and 6 × 26 AWG) do not fit here whole; see below | **W6B, W6C, W6D here**; OBC J7 moves to the −X edge at y +12 … +22 (still outside the Z rod footprint), J6 and J8 to y +24 … +34 |

The X-rod pair of W6C leaves the bundle at the deck (z −16) for the X rod at (0, +36); the Y-rod pair continues to the
−Z end and crosses to the Y rod's +Y end at (+42, +23.5, −33) under the battery (2 mm air gap) or along the deck: the
route is the harness and mechanical designers' (VI-MCAD-08). These are allocations from connector pin counts; the
harness design's wire list governs and must keep every segment within its allowance.

**Segment B (deck to EPS) is the tight one.** After the ICX harnesses, the four bundles leave about 45 mm² there
(14.3 + 11.7 + 13.1 + 5.9); an estimate of the EPS-only harnesses passing it is about 53 mm² (W1 16, the two wing
harnesses of W2 about 17, W7 about 10, W11 9.5). Two changes recover about 7 mm² and are recommended to the harness
design: W6D as six twisted 28 AWG pairs without the overall shield (5.9 instead of 9.6 mm²; DC photocurrents of a
few milliamperes, drains not needed) and the W1 pack leads in 22 AWG (≤ 2 A per §1 rule 5; 9.4 instead of 12.6 mm²)
if the pack current allows; the rest is the harness design's to confirm with its wire list (item E12).

## 3. Board-to-Board Signal Lists

Direction is seen from the OBC (out = OBC drives). "Reset" is the line's state while the OBC is in reset or unpowered.

### 3.1 W3: EPS J3 ↔ OBC J1 (40 positions, 28 AWG ribbon, 0.06 m)

| Pin | Signal | Dir | Level | Reset / OBC off | OBC pin (series part) | EPS function |
|----:|--------|-----|-------|-----------------|-----------------------|--------------|
| 1, 2 | 3V3_OBC | power in | 3.3 V ± 3 %, 0.025 A normal | — | VDD (TVS D2, 22 µF) | SW10 TPS2553-Q1, trip 0.130–0.176 A, auto-retry 0.28 s |
| 3, 4, 7, 28, 35, 40 | GND | — | — | — | GND | — |
| 21, 27 | AGND | — | analog return | — | GND (EPS joins AGND to GND at one point) | — |
| 5 | I2C2_SCL | bidir | open drain, 2.2 kΩ on the OBC, 400 kHz | pull-ups unpowered | PF1 | INA228 0x40, 0x44; controller 0x48; AntS 0x31 via U80 |
| 6 | I2C2_SDA | bidir | as SCL | — | PF0 | as SCL |
| 8 | EPS_ALERT_N | in | open drain, active low, 4.7 kΩ on the EPS | — | PB7 (10 kΩ) | INA228 alerts, controller interrupt |
| 9 | PAYLOAD_EN | out | high = SW1 on | low (100 kΩ) | **PB2** (100 Ω) | SW1 TPS1H100B-Q1 (Jetson, shed-gated) |
| 10 | RADIO_EN | out | high = SW2 on | low | PB5 (100 Ω) | SW2 receiver/TNC/exciter |
| 11 | ADCS_EN | out | high = SW4 and SW11 on | low | PB6 (100 Ω) | SW4 3V3_ADCS (sensors); **SW11 VBAT_MTQ (new)** |
| 12 | HEATER_EN | out | high = SW5 on | low | PD2 (100 Ω) | SW5 heater |
| 13 | DEPLOY_EN | out | high = SW6 and SW8 on | low | **PF3** (100 Ω) | burn buck, AntS supply (master arm) |
| 14 | RF_TX_EN | out | high = PA rail; low = TX-OFF | low (mandatory) | PA3 (100 Ω) | SW3 PA (shed-gated) |
| 15 | GNSS_EN | out | high = SW7 on | low | PE5 (100 Ω) | SW7 3V3_GNSS (RID EPS-14: PE5, not PE4) |
| 16 | DEPLOY_SENSE | in | high = DS2 released, 1 MΩ source, clamped | — | PE2 (10 kΩ) | separation state |
| 17 | BURN_W2A | out | high = burn; 12 s one-shot per rising edge | low | PB9 (100 Ω) | wing 2 resistor A driver |
| 18 | BURN_W1A | out | as 17 | low | PB10 (100 Ω) | wing 1 resistor A |
| 19 | **VJET_RB** | analog in | VBAT_JET / 6: 1.87–2.80 V on, < 0.1 V off | — | **PA7** ADC1_IN12 (1 kΩ, 10 nF) | **new**: 1.00 MΩ / 200 kΩ + 100 nF after the eFuse U19 (RID AV-06) |
| 20 | SOLAR_SENSE | in | low = both wings deployed (10 k + 1 k pull-up on the EPS) | — | PB12 (10 kΩ) | hinge switches in series |
| 22 | VBATT_SNS | analog in | pack / 6 (2.80 V at 16.8 V) | — | PC2 ADC1_IN3 (1 kΩ, 10 nF) | 1.00 MΩ / 200 kΩ, 100 nF |
| 23 | IBATT_SNS | analog in | 1.65 V at 0 A, 0.5 V/A | — | PC3 ADC1_IN4 | INA186 × 50 |
| 24 | VSOLAR_SNS | analog in | string / 4 | — | PC4 ADC1_IN13 | 300 k / 100 k |
| 25 | ISOLAR_SNS | analog in | 1.0 V/A | — | PC5 ADC1_IN14 | INA186 × 50 |
| 26 | VBUS_SNS | analog in | 5V_PA / 2 | — | PB0 ADC1_IN15 | 100 k / 100 k |
| 29 | JETSON_PWR | out | high = eFuse U19 on | low | **PA8** (100 Ω; was PA2) | U19 TPS16630 SHDN |
| 30 | BURN_W1B | out | as 17 | low | PE6 (100 Ω) | wing 1 resistor B |
| 31 | BURN_W2B | out | as 17 | low | **PE7** (100 Ω, new) | wing 2 resistor B |
| 32 | EOL_ARM | out | rising edge sets the arm latch | low | **PF4** (100 Ω, new) | U61 latch → Q21 |
| 33 | EOL_FIRE | out | high = fire (≥ 60 s) | low | **PG1** (100 Ω, new) | Q20 (in series with Q21) |
| 34 | LOCK_CLR | out | high ≥ 100 ms, ANDed with the controller's UNLOCK | low | **PF5** (100 Ω, new) | recharge-lockout clear one-shot |
| 36, 39 | **VBAT_MTQ** | power in | VBAT 11.2–16.4 V, 0.083 A at 3 × 0.2 A·m², 0.23 A at the bridges' limit | off | DRV8876 VM (TVS D3, 3 × 4.7 µF) | **new: SW11 TPS1H000-Q1 on ADCS_EN** (pin 39 was GND) |
| 37 | **VSW1_RB** | analog in | VBAT_SW1 / 6 | — | **PA1** ADC1_IN6 (1 kΩ, 10 nF) | **new**: 1.00 MΩ / 200 kΩ + 100 nF after SW1 (RID AV-06) |
| 38 | **3V3_RTC** | power in | 3V3_AO through 1.0 kΩ, ≤ 2 µA | **stays on** | RV-3028-C7 VDD (1 µF) | **new**: always-on supply of the mission clock (RID AV-10) |

Wire currents: 3V3_OBC 0.088 A per wire at the SW10 trip (35 % of 0.25 A); VBAT_MTQ 0.115 A per wire at the bridges'
limit (46 %); all others < 5 mA.

### 3.2 W4A: OBC J2 ↔ RF J1 (16 positions, 28 AWG, 0.06 m); W4B power EPS J19 ↔ RF J2

From SQSO-RF-BRD-001 §4.1 (the RF side) and the OBC board (the OBC side). RF→OBC digital lines are 3.1 V CMOS
push-pull: VOH ≥ 2.9 V against the STM32's VIH 2.31 V.

| Pin | Signal | Dir | Level | Reset / OBC off | OBC pin (OBC-side parts) | RF board |
|----:|--------|-----|-------|-----------------|--------------------------|----------|
| 1, 4, 7 | GND | — | — | — | — | — |
| 2 | TNC_RXD (OBC TX) | out | 3.3 V UART 9600 8N1 | high-Z | PD5 USART2_TX (100 Ω) | 1 kΩ into the TNC MCU |
| 3 | TNC_TXD (OBC RX) | in | 3.1 V UART | — | PD6 USART2_RX (10 kΩ, 100 kΩ pull-up) | 100 Ω series |
| 5 | TX_PWR_SEL | out | low 0.5 W, high 1 W | low (RF 100 kΩ) | PE3 (100 Ω) | 1 kΩ + 100 kΩ |
| 6 | XPDR_EN | out | 100 Hz heartbeat, level = disabled | low (RF 100 kΩ) | PE4 (100 Ω), TIM7 interrupt | U401 SN74LVC1G123 retriggerable, 50 ms |
| 8 | TONE_OK | in | high = CTCSS 67.0 Hz decoded | — | **PC9** EXTI9 (10 kΩ, 100 kΩ pull-down) | 1 kΩ (RID RF-21 / AV-05) |
| 9 | XPDR_KEY | in | high = repeat path keyed | — | **PG12** EXTI12 (10 kΩ, 100 kΩ pull-down) | 1 kΩ |
| 10 | PTT_EXCITER | in | high = transmitter keyed | — | **PC7** TIM8_CH2 input capture (10 kΩ, 100 kΩ pull-down) | 1 kΩ |
| 11 | TX_TEMP | analog in | 0.3–1.7 V, 10 mV/°C | — | PA0 ADC1_IN5 (10 kΩ, 10 nF) | 1 kΩ / 100 nF |
| 12 | TX_FWD | analog in | 0–2.5 V, P = 0.16 V² W | — | PC0 ADC1_IN1 (10 kΩ, 10 nF) | 1 kΩ / 100 nF |
| 13 | TX_REV | analog in | as TX_FWD | — | PC1 ADC1_IN2 (10 kΩ, 10 nF) | 1 kΩ / 100 nF |
| 14 | AGND | — | monitor return | — | GND | — |
| 15 | RX_CD | in | high = receiver carrier detect (not TONE_OK) | low (OBC 100 kΩ pull-down) | **PG0** EXTI0 (10 kΩ, 100 kΩ pull-down) | 1 kΩ, push-pull from V_DIG, 100 kΩ pull-down (**new, version 3**, SQSO-RF-BRD-001 B §4.1) |
| 16 | RX_FAULT | in | high = receiver fault | **low = healthy** when unwired, open or the radio off (OBC 100 kΩ pull-down) | **PF13** EXTI13 (10 kΩ, 100 kΩ pull-down) | 1 kΩ, push-pull from V_DIG, 100 kΩ pull-down (**new, version 3**, SQSO-RF-BRD-001 B §4.1) |

RADIO_EN and RF_TX_EN switch the RF board's rails on the EPS (W3 pins 10, 14) and reach the RF board as power on W4B
(RADIO_3V3 J2-1, PA_5V J2-3/4, 24 AWG). W4B runs **PA_5V and its returns as twisted pairs** (J2-3 with J2-2,
J2-4 with J2-5; RADIO_3V3 J2-1 twisted with J2-6) so the 0.47 A PA current adds no loop field at magnetometer A2;
RF J2 is on the RF board's −Y edge at (−26, −44), by the PA (SQSO-RF-BRD-001 B §4.2). The readbacks are valid while RADIO_3V3 is on and read low through the OBC's
pull-downs when it is off. The receiver is ready ≤ 50 ms after RADIO_EN (RF board §15 item 12).

### 3.3 W5: Jetson carrier

| Harness / pin | Signal | Dir | Level | Reset / OBC off | OBC side | Carrier side |
|---------------|--------|-----|-------|-----------------|----------|--------------|
| W5A EPS J18-1/2 | VBAT_JET | — | 11.2–16.4 V, 0.62 A at 8 W, eFuse 1.2 A | off (both switches) | — | carrier DC input |
| W5A EPS J18-3/4 | GND | — | — | — | — | — |
| W5B OBC J3-1 | GND | — | — | — | GND | GND |
| W5B OBC J3-2 | JET_RXD | out | 3.3 V UART 115200 | **high-Z** (U28 isolated with the carrier off) | PA9 USART1_TX → U28 SN74LVC1T45 (B side on JET_3V3) → 33 Ω | carrier J2-2 → its translator → module UART1 (J4-203/205) |
| W5B OBC J3-3 | JET_TXD | in | 3.3 V UART | — | 33 Ω → U29 (B → A) → PA10 (100 kΩ pull-up) | carrier J2-3 ← its translator ← module UART1 (J4-203/205) |
| W5B OBC J3-4 | JET_3V3 | in | the carrier's switched 3.3 V I/O rail, ≤ 1 mA drawn | 0 V with the carrier off | U28/U29 VCCB; JET_PG → PG6 (10 kΩ, 100 kΩ pull-down) | 3V3_IO, high after the module's power sequence (carrier SQSO-JET-BRD-001) |
| W5B OBC J3-5 | PT_NTC | analog in | spreader thermistor RT1 (TE GA10K3MCD1, 10 kΩ NTC) biased by the OBC: 10.0 kΩ 0.1 % from VDDA; 2.93 V at −15 °C, 1.65 V at 25 °C, 0.49 V at 70 °C | **readable with the carrier off** (the carrier passes the leads through, unpowered) | 1 kΩ → **PA2 ADC1_IN7** (100 nF) | RT1 on the spreader, J2-5 on the carrier |
| W5B OBC J3-6 | PT_RTN | — | NTC return | — | GND at J3 | J2-6 on the carrier |

W5A 2 × 22 AWG per pole (0.47 A per wire at the guard trip, 23 % of 2 A). W5B 28 AWG, 6 wires (Pico-Clasp 5013310607 at both ends; interface change E10 of the carrier board).

### 3.4 W6: ADCS (sensor rail, rods, sun sensors)

| Harness / pin | Signal | Level / current | OBC part | Far end |
|---------------|--------|-----------------|----------|---------|
| W6B EPS J20-1 → OBC J6-1 | 3V3_ADCS | 3.3 V, 0.049 A worst normal; SW4 trip 0.265–0.339 A | U5, U6, U7 TPS2553-Q1 (50–100 mA latch-up trips) | EPS SW4 (ADCS_EN PB6) |
| W6B J20-2 → J6-2 | GND | — | — | — |
| W6C OBC J7-1/2 | MTQ_X_P / MTQ_X_N | ± 0.087 A at 0.2 A·m², hardware limit 0.104–0.126 A | U22 DRV8876-Q1 OUT1/OUT2 | CR0002 rod X |
| W6C J7-3/4 | MTQ_Y_P / N | as X | U23 | rod Y |
| W6C J7-5/6 | MTQ_Z_P / N | as X | U24 | rod Z |
| W6D OBC J8-1/2 … 11/12 | CSS_PX_K/A, MX, PY, MY, PZ, MZ | photocurrent ≤ 4.0 mA (at AM0 normal incidence: requirement on the panel vendor) | U17–U19 TLV9002-Q1 transimpedance (619 Ω) → U16 ADS7138-Q1 AIN0–5 | panel photodiodes, cathode to K |
| W6D J8-13/14 | shield drains | — | GND at J8 only | — |
| W6E OBC J10-1/2/6 | 3V3_SNS, GND | 3.3 V, < 1 mA (RM3100) | U7 latch-up switch (50–100 mA) | magnetometer board A2 J1 |
| W6E J10-3/4 | SDA_S, SCL_S | I2C, 4.7 kΩ on the OBC SNS side, 100 kHz recommended | U14 TCA9517-Q1 (SNS side) | RM3100 at 0x20 |
| W6E J10-5 | MG_DRDY | 3.3 V, high = conversion done | U33 → PD11 (EXTI11) | RM3100 DRDY |

W6B, W6C 26 AWG (W6C twisted pairs, 25 % of 0.5 A at the hardware limit; W6B 68 % at the SW4 trip, transient);
W6D 28 AWG twisted pairs under one shield. W6E 28 AWG, 0.10 m (about 0.06 m routed up the (+X, +Y) bundle, MECHANICAL_DESIGN §6.9, plus a service loop) to the
+Z end frame; any excess is stowed below z +20, away from A2.

### 3.5 W9: GNSS receiver (SkyFox piNAV-NG/FM)

| Harness / pin | Signal | Dir | Level | OBC side | Receiver |
|---------------|--------|-----|-------|----------|----------|
| W9B EPS J21-1/2 → OBC J4-1/2 | 3V3_GNSS, GND | power | 3.3 V, 0.10 A peak, SW7 trip 0.175–0.232 A | through the OBC to J5-1; VCCB of U30–U32; GNSS_PG → PG15 | — |
| W9A OBC J5-1/2 | 3V3_GNSS, GND | power | as above | — | VIN, GND |
| W9A J5-3 | GNSS_RXD | out | 3.3 V UART 9600 | PG7 LPUART1_TX → U30 (B side on 3V3_GNSS) | RXD |
| W9A J5-4 | GNSS_TXD | in | 3.3 V UART | U31 → PG8 LPUART1_RX (100 kΩ pull-up) | TXD |
| W9A J5-5 | GNSS_1PPS | in | 3.3 V pulse | U32 → PC6 TIM3_CH1 input capture | 1PPS |
| W9A J5-6 | GND | — | — | — | GND |

26 AWG (0.232 A at the trip, 46 %). The receiver-side connector and pin order follow the SkyFox interface document
(open: closure when the EM unit is bought; the wire functions do not change).

### 3.6 W8: GSE access port (OBC J9) and the charging path

The access window is on the −Y face in the −Z end zone (MECHANICAL_DESIGN §6.4), 48 mm below the OBC: OBC J9
(Samtec FTSH-105, top side) connects through an internal harness W8A (28 AWG, 0.15 m: about 0.10 m routed plus a service loop, excess stowed below z +20, away from A2) to a
2 × 5 2.54 mm bulkhead header at the window (harness designer's part), into which the GSE cable W8B plugs. The pin
numbers below hold at both ends of W8A.

| Pin | Signal | Dir | OBC side | GSE |
|----:|--------|-----|----------|-----|
| 1 | VTREF | out | 3V3_OBC through 100 Ω | probe reference sense; jumpered to pin 9 in the plug |
| 2 | SWDIO | bidir | PA13 (22 Ω) | SWD probe |
| 3, 10 | GND | — | — | — |
| 4 | SWCLK | in | PA14 (22 Ω) | SWD probe |
| 5 | WDT_DIS | in | TPS3850 SET0 (10 kΩ; 100 kΩ pull-down) | switch to VTREF: external watchdog disabled after the next supervisor reset |
| 6 | NRST | bidir | NRST (100 Ω) | probe reset |
| 7 | USART3_TX | out | PD8 (100 Ω) | console, key loading |
| 8 | USART3_RX | in | PD9 (10 kΩ, 100 kΩ pull-up) | console |
| 9 | UMBILICAL_SENSE | in | PB8 (1 kΩ, 100 kΩ pull-down): high = present | tied to pin 1 in the plug |

Key loading (SQSO-ICD-013 §9.2) needs UMBILICAL_SENSE high, which only the GSE plug provides. The battery is charged
through the EPS's access-port connector J13 (W11: UMB_CHG through F5 and D16 to PACK_P, return PACK_N, pack monitor
UMB_VMON through 100 kΩ, AFE shutdown UMB_SHUT; SQSO-EPS-BRD-001 §6.5), reached through the same access window. The
ESD arrays at J9 and J13 protect both. W8 is not flown.

## 4. Master Pin Allocation: STM32L4R5ZIT6 (LQFP-144)

### 4.1 Signal pins

Generated from `hardware/electrical/obc/obc_pinmap.csv`. Dir: O out, I in, A analog in, B bidirectional, - not used
(the firmware sets unused pins to analog mode). "Reset" is the pin's state from reset until the firmware configures it.

| Pin | Port | Net | Function | Dir | Level | Default at reset | Goes to |
|----:|------|-----|----------|-----|-------|------------------|---------|
| 34 | PA0 | TX_TEMP | ADC1_IN5 | A | 0.3-1.7 V | analog (high-Z) | J2-11 RF board (10 k series, 10 nF) |
| 35 | PA1 | VSW1_RB | ADC1_IN6 | A | 0-2.8 V (VBAT_SW1 / 6) | analog (high-Z) | J1-37 EPS (new; was DEPLOY_EN) |
| 36 | PA2 | PT_NTC | ADC1_IN7 | A | VDDA x R/(10.0 k + R): 2.93 V at -15 C, 0.49 V at 70 C | analog (high-Z) | J3-5 Jetson spreader NTC (new, E10; was JETSON_PWR) |
| 37 | PA3 | RF_TX_EN | GPIO out | O | 3.3 V, high = PA rail (TX-OFF low) | analog (high-Z); EPS 100 k pull-down | J1-14 EPS |
| 40 | PA4 | MTQ_X_I | ADC1_IN9 | A | 10 mV/mA (IPROPI) | analog (high-Z) | U22 X bridge (was FRAM CS#) |
| 41 | PA5 | MTQ_Y_I | ADC1_IN10 | A | 10 mV/mA | analog (high-Z) | U23 Y bridge (was SPI1 SCK) |
| 42 | PA6 | MTQ_Z_I | ADC1_IN11 | A | 10 mV/mA | analog (high-Z) | U24 Z bridge (was SPI1 MISO) |
| 43 | PA7 | VJET_RB | ADC1_IN12 | A | 0-2.8 V (VBAT_JET / 6) | analog (high-Z) | J1-19 EPS (new; was SPI1 MOSI) |
| 100 | PA8 | JETSON_PWR | GPIO out | O | 3.3 V, high = eFuse U19 on | analog (high-Z); EPS 100 k pull-down | J1-29 EPS (moved from PA2) |
| 101 | PA9 | USART1_TX | USART1_TX AF7 | O | 3.3 V UART 115200 | analog (high-Z) | U28 -> J3-2 Jetson |
| 102 | PA10 | USART1_RX | USART1_RX AF7 | I | 3.3 V UART | analog (high-Z); 100 k pull-up | U29 <- J3-3 Jetson |
| 103 | PA11 | - | spare (USB DM unused) | - |  | analog (high-Z) |  |
| 104 | PA12 | - | spare (USB DP unused) | - |  | analog (high-Z) |  |
| 105 | PA13 | SWDIO | SWDIO AF0 | B | 3.3 V | SWDIO, pull-up | J9-2 access port |
| 109 | PA14 | SWCLK | SWCLK AF0 | I | 3.3 V | SWCLK, pull-down | J9-4 access port |
| 110 | PA15 | - | spare (JTDI) | - |  | JTDI, pull-up: do not use as an enable |  |
| 46 | PB0 | VBUS_SNS | ADC1_IN15 | A | 0-2.5 V (5V_PA / 2) | analog (high-Z) | J1-26 EPS |
| 47 | PB1 | TEMP_BOARD | ADC1_IN16 | A | 10 mV/C, 0.5 V at 0 C | analog (high-Z) | U21 TMP235 on the OBC |
| 48 | PB2 | PAYLOAD_EN | GPIO out | O | 3.3 V, high = SW1 on | analog (high-Z); EPS 100 k pull-down | J1-9 EPS (moved from PB4) |
| 133 | PB3 | SWO | spare (JTDO/SWO) | - |  | JTDO | TP only |
| 134 | PB4 | - | spare (NJTRST) | - |  | NJTRST, pull-up: never an enable |  |
| 135 | PB5 | RADIO_EN | GPIO out | O | 3.3 V, high = SW2 on | analog (high-Z); EPS 100 k pull-down | J1-10 EPS |
| 136 | PB6 | ADCS_EN | GPIO out | O | 3.3 V, high = SW4 + VBAT_MTQ switch on | analog (high-Z); EPS 100 k pull-down | J1-11 EPS |
| 137 | PB7 | EPS_ALERT_N | GPIO in, EXTI7 | I | open drain, active low (EPS 4.7 k) | analog (high-Z) | J1-8 EPS (10 k series) |
| 139 | PB8 | UMBILICAL_SENSE | GPIO in | I | high = GSE plug present | analog (high-Z); 100 k pull-down | J9-9 access port |
| 140 | PB9 | BURN_W2A | GPIO out | O | 3.3 V, high = burn (12 s hardware limit) | analog (high-Z); EPS 100 k pull-down | J1-17 EPS (was ANTENNA_BURN) |
| 69 | PB10 | BURN_W1A | GPIO out | O | 3.3 V, high = burn | analog (high-Z); EPS 100 k pull-down | J1-18 EPS (was SOLAR_BURN) |
| 70 | PB11 | - | spare (was ANTENNA_SENSE) | - |  | analog (high-Z) |  |
| 73 | PB12 | SOLAR_SENSE | GPIO in | I | low = both wings deployed | analog (high-Z); EPS pull-up | J1-20 EPS (10 k series) |
| 74 | PB13 | IMU2_SCK | SPI2_SCK AF5 | O | 3.3 V, <= 10 MHz | analog (high-Z) | U12 -> BMI088 |
| 75 | PB14 | IMU2_MISO | SPI2_MISO AF5 | I | 3.3 V | analog (high-Z); 100 k pull-down | U12 <- BMI088 SDO1/SDO2 |
| 76 | PB15 | IMU2_MOSI | SPI2_MOSI AF5 | O | 3.3 V | analog (high-Z) | U12 -> BMI088 |
| 26 | PC0 | TX_FWD | ADC1_IN1 | A | 0-2.5 V | analog (high-Z) | J2-12 RF board |
| 27 | PC1 | TX_REV | ADC1_IN2 | A | 0-2.5 V | analog (high-Z) | J2-13 RF board |
| 28 | PC2 | VBATT_SNS | ADC1_IN3 | A | 0-2.8 V (pack / 6) | analog (high-Z) | J1-22 EPS |
| 29 | PC3 | IBATT_SNS | ADC1_IN4 | A | 0-3.3 V | analog (high-Z) | J1-23 EPS |
| 44 | PC4 | VSOLAR_SNS | ADC1_IN13 | A | 0-3.3 V (/4) | analog (high-Z) | J1-24 EPS |
| 45 | PC5 | ISOLAR_SNS | ADC1_IN14 | A | 0-3.3 V | analog (high-Z) | J1-25 EPS |
| 96 | PC6 | GNSS_PPS | TIM3_CH1 AF2 capture | I | 3.3 V pulse | analog (high-Z); 100 k pull-down | U32 <- J5-5 GNSS |
| 97 | PC7 | PTT_EXC_RB | TIM8_CH2 AF3 capture | I | 3.1 V CMOS, high = keyed | analog (high-Z); 100 k pull-down | J2-10 RF board (10 k series) |
| 98 | PC8 | RTC_1HZ | TIM3_CH3 AF2 capture | I | RTC CLKOUT (1 Hz once set) | analog (high-Z) | U4 RV-3028 CLKOUT (1 M series) |
| 99 | PC9 | TONE_OK | GPIO in, EXTI9 | I | 3.1 V CMOS, high = CTCSS | analog (high-Z); 100 k pull-down | J2-8 RF board (10 k series) |
| 111 | PC10 | GYRO_SCK | SPI3_SCK AF6 | O | 3.3 V, mode 3, <= 1 MHz burst | analog (high-Z); 100 k pull-up | U9 -> M-G370 |
| 112 | PC11 | GYRO_MISO | SPI3_MISO AF6 | I | 3.3 V | analog (high-Z); 100 k pull-down | U9 <- M-G370 |
| 113 | PC12 | GYRO_MOSI | SPI3_MOSI AF6 | O | 3.3 V | analog (high-Z) | U9 -> M-G370 |
| 7 | PC13 | - | spare | - |  | analog (high-Z) |  |
| 8 | PC14 | LSE_IN | OSC32_IN | - | 32.768 kHz | LSE off | Y2 |
| 9 | PC15 | LSE_OUT | OSC32_OUT | - | 32.768 kHz | LSE off | Y2 |
| 114 | PD0 | IMU2_GCS_N | GPIO out (SPI2 CS) | O | 3.3 V, low = select | analog (high-Z); 100 k pull-up | U12 -> BMI088 CSB2 |
| 115 | PD1 | GYRO_CS_N | GPIO out (SPI3 CS) | O | 3.3 V, low = select | analog (high-Z); 100 k pull-up | U9 -> M-G370 /CS |
| 116 | PD2 | HEATER_EN | GPIO out | O | 3.3 V, high = SW5 on | analog (high-Z); EPS 100 k pull-down | J1-12 EPS |
| 117 | PD3 | LED_STATUS | GPIO out | O | 3.3 V | analog (high-Z) | D1 (DNP in flight) |
| 118 | PD4 | IMU2_ACS_N | GPIO out (SPI2 CS2) | O | 3.3 V, low = select | analog (high-Z); 100 k pull-up | U13 -> BMI088 CSB1 |
| 119 | PD5 | USART2_TX | USART2_TX AF7 | O | 3.3 V UART 9600 | analog (high-Z) | J2-2 RF TNC |
| 122 | PD6 | USART2_RX | USART2_RX AF7 | I | 3.1 V UART | analog (high-Z); 100 k pull-up | J2-3 RF TNC (10 k series) |
| 123 | PD7 | - | spare | - |  | analog (high-Z) |  |
| 77 | PD8 | USART3_TX | USART3_TX AF7 | O | 3.3 V UART 115200 | analog (high-Z) | J9-7 access port |
| 78 | PD9 | USART3_RX | USART3_RX AF7 | I | 3.3 V UART | analog (high-Z); 100 k pull-up | J9-8 access port (10 k series) |
| 79 | PD10 | GYRO_DRDY | GPIO in, EXTI10 | I | 3.3 V | analog (high-Z); 100 k pull-down | U10 <- M-G370 DRDY |
| 80 | PD11 | MAG_DRDY | GPIO in, EXTI11 | I | 3.3 V | analog (high-Z); 100 k pull-down | U33 <- J10-5 magnetometer board (RM3100 DRDY) |
| 81 | PD12 | I2C4_SCL | I2C4_SCL AF4 | B | open drain, 2.2 k | analog (high-Z) | U14 -> ADS7138; J10 -> RM3100 board |
| 82 | PD13 | I2C4_SDA | I2C4_SDA AF4 | B | open drain, 2.2 k | analog (high-Z) | U14 -> ADS7138; J10 -> RM3100 board |
| 85 | PD14 | GYRO_RST_N | GPIO out | O | 3.3 V, low = reset (>= 100 ms) | analog (high-Z); 100 k pull-down | U10 -> M-G370 /RST |
| 86 | PD15 | IMU2_DRDY | GPIO in, EXTI15 | I | 3.3 V | analog (high-Z); 100 k pull-down | U13 <- BMI088 INT3 |
| 141 | PE0 | FRAM_WP_N | GPIO out | O | 3.3 V, high = writes allowed | analog (high-Z); 10 k pull-up | U3 |
| 142 | PE1 | FRAM_HOLD_N | GPIO out | O | 3.3 V, high | analog (high-Z); 10 k pull-up | U3 |
| 1 | PE2 | DEPLOY_SENSE | GPIO in, EXTI2 | I | high = DS2 released (1 M source) | analog (high-Z) | J1-16 EPS (10 k series) |
| 2 | PE3 | TX_PWR_SEL | GPIO out | O | 3.3 V, low 0.5 W / high 1 W | analog (high-Z); RF 100 k pull-down | J2-5 RF board |
| 3 | PE4 | XPDR_EN | GPIO out (TIM7 heartbeat) | O | 100 Hz square wave | analog (high-Z); RF 100 k pull-down | J2-6 RF board |
| 4 | PE5 | GNSS_EN | GPIO out | O | 3.3 V, high = SW7 on | analog (high-Z); EPS 100 k pull-down | J1-15 EPS |
| 5 | PE6 | BURN_W1B | GPIO out | O | 3.3 V, high = burn | analog (high-Z); EPS 100 k pull-down | J1-30 EPS (was SOLAR_BURN_B) |
| 58 | PE7 | BURN_W2B | GPIO out | O | 3.3 V, high = burn | analog (high-Z); EPS 100 k pull-down | J1-31 EPS (new) |
| 59 | PE8 | MTQ_X_DIR | GPIO out | O | 3.3 V, PH | analog (high-Z); DRV pull-down | U22 PH |
| 60 | PE9 | MTQ_X_PWM | TIM1_CH1 AF1 | O | 3.3 V PWM 40 kHz | analog (high-Z); DRV pull-down | U22 EN |
| 63 | PE10 | MTQ_Y_DIR | GPIO out | O | 3.3 V, PH | analog (high-Z); DRV pull-down | U23 PH |
| 64 | PE11 | MTQ_Y_PWM | TIM1_CH2 AF1 | O | 3.3 V PWM 40 kHz | analog (high-Z); DRV pull-down | U23 EN |
| 65 | PE12 | MTQ_Z_DIR | GPIO out | O | 3.3 V, PH | analog (high-Z); DRV pull-down | U24 PH |
| 66 | PE13 | MTQ_Z_PWM | TIM1_CH3 AF1 | O | 3.3 V PWM 40 kHz | analog (high-Z); DRV pull-down | U24 EN |
| 67 | PE14 | MTQ_FAULT_N | GPIO in, EXTI14 | I | open drain, 10 k pull-up | analog (high-Z) | U22-U24 nFAULT |
| 68 | PE15 | MTQ_EXPIRED | GPIO in | I | high = hardware off-timer expired | analog (high-Z) | U25 Q10 |
| 10 | PF0 | I2C2_SDA | I2C2_SDA AF4 | B | open drain, 2.2 k on the OBC | analog (high-Z) | J1-6 EPS (0x40, 0x44, 0x48, AntS 0x31) |
| 11 | PF1 | I2C2_SCL | I2C2_SCL AF4 | B | open drain, 2.2 k on the OBC | analog (high-Z) | J1-5 EPS |
| 12 | PF2 | - | spare | - |  | analog (high-Z) |  |
| 13 | PF3 | DEPLOY_EN | GPIO out | O | 3.3 V, high = SW6/SW8 on (master arm) | analog (high-Z); EPS 100 k pull-down | J1-13 EPS (moved from PA1) |
| 14 | PF4 | EOL_ARM | GPIO out | O | 3.3 V, rising edge sets the arm latch | analog (high-Z); EPS 100 k pull-down | J1-32 EPS (new) |
| 15 | PF5 | LOCK_CLR | GPIO out | O | 3.3 V, high >= 100 ms with UNLOCK | analog (high-Z); EPS 100 k pull-down | J1-34 EPS (new) |
| 18 | PF6 | GYRO_EN | GPIO out | O | 3.3 V, high = 3V3_GYRO | analog (high-Z); 100 k pull-down | U5 (100 k + 2.2 uF retry) |
| 19 | PF7 | IMU2_EN | GPIO out | O | 3.3 V, high = 3V3_IMU2 | analog (high-Z); 100 k pull-down | U6 |
| 20 | PF8 | SNS_EN | GPIO out | O | 3.3 V, high = 3V3_SNS | analog (high-Z); 100 k pull-down | U7 |
| 21 | PF9 | MTQ_ARM | TIM15_CH1 AF14, one-pulse 900 ms | O | 3.3 V, high = coil burst | analog (high-Z); 100 k pull-down | U26/U27 -> nSLEEP |
| 22 | PF10 | GYRO_RB | GPIO in | I | low with GYRO_EN high = latch-up trip | analog (high-Z) | U5 EN node (100 k) |
| 49 | PF11 | IMU2_RB | GPIO in | I | as GYRO_RB | analog (high-Z) | U6 EN node (100 k) |
| 50 | PF12 | SNS_RB | GPIO in | I | as GYRO_RB | analog (high-Z) | U7 EN node (100 k) |
| 53 | PF13 | RX_FAULT | GPIO in, EXTI13 | I | 3.1 V CMOS, high = receiver fault | analog (high-Z); 100 k pull-down (unwired or radio off reads healthy) | J2-16 RF board (10 k series; new, FSW round 2) |
| 54 | PF14 | - | spare | - |  | analog (high-Z) |  |
| 55 | PF15 | - | spare | - |  | analog (high-Z) |  |
| 56 | PG0 | RX_CD | GPIO in, EXTI0 | I | 3.1 V CMOS, high = receiver carrier detect | analog (high-Z); 100 k pull-down | J2-15 RF board (10 k series; new, FSW round 2) |
| 57 | PG1 | EOL_FIRE | GPIO out | O | 3.3 V, high = fire (>= 60 s) | analog (high-Z); EPS 100 k pull-down | J1-33 EPS (new) |
| 87 | PG2 | FRAM_SCK | SPI1_SCK AF5 | O | 3.3 V, 20 MHz mode 0 | analog (high-Z) | U3 (moved from PA5) |
| 88 | PG3 | FRAM_MISO | SPI1_MISO AF5 | I | 3.3 V | analog (high-Z) | U3 (moved from PA6) |
| 89 | PG4 | FRAM_MOSI | SPI1_MOSI AF5 | O | 3.3 V | analog (high-Z) | U3 (moved from PA7) |
| 90 | PG5 | FRAM_CS_N | GPIO out | O | 3.3 V, low = select | analog (high-Z); 10 k pull-up | U3 (moved from PA4) |
| 91 | PG6 | JET_PG | GPIO in, EXTI6 | I | high = carrier 3.3 V up | analog (high-Z); 100 k pull-down | J3-4 Jetson (10 k series) |
| 92 | PG7 | LPUART1_TX | LPUART1_TX AF8 | O | 3.3 V UART 9600 | analog (high-Z) | U30 -> J5-3 GNSS |
| 93 | PG8 | LPUART1_RX | LPUART1_RX AF8 | I | 3.3 V UART | analog (high-Z); 100 k pull-up | U31 <- J5-4 GNSS |
| 124 | PG9 | - | spare | - |  | analog (high-Z) |  |
| 125 | PG10 | - | spare | - |  | analog (high-Z) |  |
| 126 | PG11 | WDI | GPIO out | O | falling edge every 10 s | analog (high-Z); 100 k pull-down | U2 TPS3850 WDI |
| 127 | PG12 | XPDR_KEY | GPIO in, EXTI12 | I | 3.1 V CMOS, high = repeat keyed | analog (high-Z); 100 k pull-down | J2-9 RF board (10 k series) |
| 128 | PG13 | I2C1_SDA | I2C1_SDA AF4 | B | open drain, 4.7 k | analog (high-Z) | U4 RV-3028 (0x52) |
| 129 | PG14 | I2C1_SCL | I2C1_SCL AF4 | B | open drain, 4.7 k | analog (high-Z) | U4 RV-3028 |
| 132 | PG15 | GNSS_PG | GPIO in | I | high = 3V3_GNSS up | analog (high-Z); 100 k pull-down | J4 3V3_GNSS (10 k series) |
| 23 | PH0 | HSE_IN | OSC_IN (HSE bypass) | I | 8 MHz CMOS | HSE off | Y1 XO |
| 24 | PH1 | - | spare (OSC_OUT unused in bypass) | - |  | analog (high-Z) |  |
| 138 | PH3 | BOOT0 | BOOT0 | I | low = boot from flash | BOOT0 | 10 k pull-down |
| 25 | NRST | NRST | NRST | B | open drain wired-OR | low until released | U2 RESET/WDO, J9-6, U4 EVI |

### 4.2 Supply pins

| Pins | Name | Connection |
|------|------|------------|
| 17, 39, 52, 62, 72, 84, 108, 121, 144 | VDD | 3V3_OBC, 100 nF each, 10 µF + 22 µF bulk |
| 95, 131 | VDDIO2 | 3V3_OBC, 100 nF each (port G[15:2]: the firmware must enable it, PWR_CR2.IOSV, before using PG2–PG15) |
| 106 | VDDUSB | 3V3_OBC, 100 nF (USB not used) |
| 6 | VBAT | 3V3_OBC, 100 nF (no backup cell; the mission clock is the RV-3028-C7 on the always-on rail) |
| 33, 32 | VDDA, VREF+ | VDDA: 3V3_OBC through FB1 (600 Ω at 100 MHz); 1 µF + 10 nF and 1 µF + 100 nF |
| 30, 31 | VSSA, VREF− | GND |
| 16, 38, 51, 61, 71, 83, 94, 107, 120, 130, 143 | VSS | GND |

### 4.3 Peripheral, timer and interrupt allocation

| Resource | Use | Pins |
|----------|-----|------|
| ADC1 (16 external channels) | IN1 TX_FWD, IN2 TX_REV, IN3 VBATT, IN4 IBATT, IN5 TX_TEMP, IN6 VSW1_RB, **IN7 PT_NTC (Jetson spreader)**, IN9–IN11 MTQ_X/Y/Z_I, IN12 VJET_RB, IN13 VSOLAR, IN14 ISOLAR, IN15 VBUS, IN16 TEMP_BOARD; internal: temperature, VREFINT | 15 used; IN8 (PA3) is the RF_TX_EN output |
| SPI1 | FRAM FM25V20A, 20 MHz mode 0 | PG2/PG3/PG4, CS PG5 |
| SPI2 (`hal_spi` bus 0) | BMI088 (gyro CS PD0, accelerometer CS PD4), ≤ 10 MHz | PB13/PB14/PB15 |
| SPI3 (`hal_spi` bus 1) | Epson M-G370PDT0, mode 3, ≤ 1 MHz burst | PC10/PC11/PC12, CS PD1 |
| I2C1 (new `hal_i2c` bus 2) | RV-3028-C7 mission clock, 0x52, 100 kHz | PG14 SCL / PG13 SDA |
| I2C2 (`hal_i2c` bus 0) | EPS: INA228 0x40, 0x44; controller 0x48; AntS 0x31 | PF1 / PF0 |
| I2C4 (`hal_i2c` bus 1) | through U14 TCA9517: ADS7138 0x10; RM3100 0x20 on board A2 via J10 | PD12 / PD13 |
| USART1 (`hal_uart` port 0) | Jetson, 115200 | PA9 / PA10 |
| USART2 (`radio.rs`) | KISS TNC, 9600 | PD5 / PD6 |
| USART3 (`hal_uart` port 1) | console and key loading | PD8 / PD9 |
| LPUART1 (`hal_uart` port 2) | GNSS NMEA, 9600 | PG7 / PG8 |
| TIM1 CH1–CH3 | magnetorquer PWM, 40 kHz | PE9, PE11, PE13 |
| TIM15 CH1 | MTQ_ARM, one-pulse 900 ms per control period | PF9 |
| TIM3 CH1, CH3 | input capture: GNSS 1PPS, RTC 1 Hz | PC6, PC8 |
| TIM8 CH2 | input capture: PTT_EXCITER keyed time | PC7 |
| TIM7 | XPDR_EN heartbeat interrupt (no pin function) | PE4 as GPIO |
| TIM2 | Embassy time driver (`time-driver-any` picks TIM2 today; pin it with `time-driver-tim2`) | none |
| IWDG | 30 s, LSI | — |
| EXTI lines | 2 DEPLOY_SENSE, 6 JET_PG, 7 EPS_ALERT_N, 8 UMBILICAL_SENSE, 9 TONE_OK, 10 GYRO_DRDY, 11 MAG_DRDY, 12 XPDR_KEY, 14 MTQ_FAULT_N, 15 IMU2_DRDY (one pin per line number: no collisions) | — |
| HSE (bypass) | 8 MHz XO Y1 | PH0 (PH1 free) |
| LSE | 32.768 kHz crystal Y2 | PC14 / PC15 |

### 4.4 Checks

- Every one of the 144 pins appears once in §4.1 or §4.2 (116 signal and system pins, 28 supply pins); every net
  appears on one MCU pin (`design.py` builds the MCU from this table; the netlist has 144 MCU pin connections).
- Version B: JETSON_PWR moved from PA2 to PA8 (port A kept, so it stays on a different port from PAYLOAD_EN PB2) to
  give PA2 (ADC1_IN7) to the Jetson spreader NTC, the only way to read it in the always-on MCU domain with the carrier
  off; the ADS7138 inputs are all used and depend on the switched sensor rail.
- Alternate functions checked against ST's pin database for the STM32L4R5ZITx LQFP144 (stm32-data, the source of the
  embassy-stm32 metadata the firmware is built with): every peripheral above is on a pin and AF the part offers.
- Spare (13 pins, analog mode): PA11, PA12, PA15 (JTDI), PB4 (NJTRST), PB11, PC13, PD7, PF2, PF14, PF15, PG9, PG10,
  PH1. Version B gave PG0 and PF13 to RX_CD and RX_FAULT. EXTI lines in use: 0 (RX_CD), 2, 6, 7, 9, 10, 11, 12,
  13 (RX_FAULT), 14, 15; free: 1, 3, 4, 5, 8. PB3 (SWO) goes to a test point only.

## 5. Domain Crossings and Back-Power

| Crossing | Part | Why |
|----------|------|-----|
| MCU → Jetson carrier (USART1) | U28, U29 SN74LVC1T45-Q1, B side on the carrier's JET_3V3 | RID AV-09: an unpowered carrier sees high-impedance pins (VCC isolation); no firmware rule needed |
| MCU → GNSS receiver (LPUART1, 1PPS) | U30–U32 SN74LVC1T45-Q1, B side on 3V3_GNSS | same, and a latch-up trip of SW7 is not fed by the UART |
| MCU → M-G370 (SPI3, /RST, DRDY) | U9 TXU0304-Q1, U10 TXU0204-Q1, B side on 3V3_GYRO | the gyro rail is switched for latch-up recovery |
| MCU → BMI088 (SPI2, two CS, INT3) | U12 TXU0304-Q1, U13 TXU0204-Q1, B side on 3V3_IMU2 | the backup is a cold spare |
| MCU → RM3100 (board A2, W6E), ADS7138 (I2C4, DRDY) | U14 TCA9517-Q1, U33 SN74LVC1T45-Q1, B side on 3V3_SNS | switched sensor rail |
| Jetson spreader NTC | passive, biased from VDDA on the OBC | readable whether the carrier is on or off; no carrier supply involved |
| MCU → EPS | EPS gates on 3V3_AO (always on) | the EPS is powered whenever the OBC is |
| EPS, RF → unpowered OBC | series resistors (10 kΩ digital, 1–10 kΩ analog) | ≤ 2.7 mA into the OBC's clamps with every line high (OBC_BOARD_DESIGN §16) |
| MCU → RF board | RF inputs: 1 kΩ series, Ioff gates | the radio is off only in the RF-silence window; RF board §4.1 |
| RTC domain (3V3_AO) ↔ MCU | EVI from NRST through 100 kΩ; CLKOUT through 1 MΩ; I2C open drain | the clock runs while the OBC is off |

## 6. Changes from the Firmware's Current Pin Use (for the flight-software team)

The firmware (`software/rust/firmware/src/gpio.rs`, `main.rs`, `adc.rs`, `spi.rs`, `i2c.rs`, `uart.rs`,
`heartbeat.rs`, as of 2026-10-27) is the baseline these rows change. Nothing in the flight software was edited by this
design.

| # | Function | Firmware today | Allocation | Change and reason |
|--:|----------|----------------|------------|-------------------|
| 1 | PAYLOAD_EN (`GPIO_PIN_PAYLOAD_EN`) | PB4 | **PB2** | PB4 is NJTRST with an internal pull-up at reset: SW1 switched on at every reset (§1 rule 2) |
| 2 | DEPLOY_EN | PA1 | **PF3** | frees ADC1_IN6 for VSW1_RB; arm on a different port from the burn lines |
| 3 | FRAM SPI1 SCK / MISO / MOSI / CS# | PA5 / PA6 / PA7 / PA4 | **PG2 / PG3 / PG4 / PG5** (SPI1 AF5) | frees ADC1_IN9–IN12 for the coil currents and VJET_RB; FRAM WP# PE0, HOLD# PE1 unchanged |
| 4 | `GPIO_PIN_ANTENNA_BURN` (PB9, held low) | unused | **BURN_W2A**, driven | four independent burn lines (EPS decision): W1A PB10, W1B PE6, W2A PB9, W2B PE7; one resistor at a time |
| 5 | `GPIO_PIN_SOLAR_BURN` (PB10) | driver A of both wings | **BURN_W1A** (wing 1 only) | as 4 |
| 6 | `GPIO_PIN_SOLAR_BURN_B` (PE6) | driver B of both wings | **BURN_W1B** | as 4 |
| 7 | BURN_W2B | none | **PE7** (new) | as 4 |
| 8 | `GPIO_PIN_ANTENNA_SENSE` (PB11) | input, not read | removed (spare) | the AntS reports its elements over I2C |
| 9 | EOL_ARM, EOL_FIRE, LOCK_CLR | none | **PF4, PG1, PF5** (new outputs) | passivation (RID EPS-06, REG-01) and the recharge-lockout clear (EPS-12) |
| 10 | SPI2 (`hal_spi` bus 0, CS PD0) | "RF transceiver" | **BMI088** (gyro CS PD0) + accelerometer CS **PD4** (new) | the RF board no longer needs SPI2 (RF board §15 item 3) |
| 11 | SPI3 (`hal_spi` bus 1, CS PD1) | generic | **M-G370PDT0**, mode 3; /RST **PD14**, DRDY **PD10** (new) | RID AV-03 |
| 12 | I2C4 (`hal_i2c` bus 1) | generic | RM3100 0x20 and ADS7138 0x10 behind U14; MAG_DRDY **PD11** (new) | RID AV-03 |
| 13 | I2C1 | none | **PG14 / PG13**, RV-3028-C7 0x52 (new `hal_i2c` bus 2) | RID AV-10 |
| 14 | ADC1 | 9 external channels | + **IN6 PA1 VSW1_RB, IN9 PA4 MTQ_X_I, IN10 PA5 MTQ_Y_I, IN11 PA6 MTQ_Z_I, IN12 PA7 VJET_RB** | new `HalAdcChannel_t` entries |
| 15 | ADC scale VBATT | `Linear(3.0, 0.0)` (2S) | `Linear(6.0, 0.0)` | EPS divider 1.00 M / 200 k (finding F1); the other EPS scales are unchanged (EPS board §11) |
| 16 | Magnetorquers | none | TIM1 CH1–3 **PE9/PE11/PE13** (EN, PWM 40 kHz), PH **PE8/PE10/PE12**, MTQ_ARM **PF9** (TIM15_CH1 one-pulse 900 ms), MTQ_FAULT_N **PE14**, MTQ_EXPIRED **PE15** | RID AV-03; OBC_BOARD_DESIGN §8 |
| 17 | Sensor rails | none | GYRO_EN **PF6**, IMU2_EN **PF7**, SNS_EN **PF8**; trip readbacks **PF10–PF12** | RID AV-03 (latch-up) |
| 18 | Repeat-path readbacks | none | TONE_OK **PC9**, XPDR_KEY **PG12**, PTT_EXCITER **PC7** (TIM8_CH2 capture) | RIDs RF-21, AV-05 |
| 19 | Jetson readbacks | none | VSW1_RB (ADC), VJET_RB (ADC), JET_PG **PG6** | RID AV-06 |
| 20 | GNSS | LPUART1 PG7/PG8, GNSS_EN PE5 | unchanged; + 1PPS **PC6** (TIM3_CH1), GNSS_PG **PG15** | LV-04 kept; EPS-14 closed |
| 21 | External watchdog | none | WDI **PG11** (falling edge every 10 s while every task has checked in) | RID AV-02 |
| 22 | Mission clock 1 Hz | none | RTC CLKOUT → **PC8** (TIM3_CH3) | clock check of the MCU against the RTC |
| 23 | Clocks | MSI 4 MHz → PLL 80 MHz | optional: HSE bypass 8 MHz on PH0, PLL M 1, N 20, R 2; LSE drive medium-high | the XO is fitted either way |
| 24 | Time driver | `time-driver-any` | `time-driver-tim2` | TIM1, TIM3, TIM8, TIM15 are now allocated |
| 25 | USART1 back-power (ICD-002 §9 alternative) | — | no firmware action | hardware isolation (§5) |
| 26 | JETSON_PWR | PA2 | **PA8** (Version B) | PA2 becomes ADC1_IN7 for the Jetson spreader NTC; port A kept |
| 27 | Jetson spreader temperature | none (the guard uses the Jetson's report) | **PT_NTC on PA2 (ADC1_IN7)**: R = 10.0 kΩ × V/(VDDA − V); β 3976 K (to confirm from the TE curve) | carrier interface E10; the 70 °C guard and the −15 °C power-on gate need it with the carrier off |
| 28 | RX_CD, RX_FAULT (new RF board inputs, flight-software round 2) | none | **RX_CD PG0** (EXTI0, high = carrier detect), **RX_FAULT PF13** (EXTI13, high = fault; reads low = healthy with the line open or the radio off) | W4A version 3 pins 15, 16; qualify both with RADIO_EN; **ignore RX_FAULT for 1 s after RADIO_EN switches on** (receiver start-up) |
| 29 | Unchanged | — | RADIO_EN PB5, ADCS_EN PB6, HEATER_EN PD2, LED PD3, RF_TX_EN PA3, EPS_ALERT_N PB7, DEPLOY_SENSE PE2, UMBILICAL_SENSE PB8, SOLAR_SENSE PB12, TX_PWR_SEL PE3, XPDR_EN PE4 (TIM7), GNSS_EN PE5, USART1/2/3, LPUART1, I2C2, I2C4, SPI3 pins, FRAM WP#/HOLD#, the nine existing ADC channels | — |

New `HalGpioPin_t` entries implied (names suggested; indices are the flight software's choice): BURN_W2A (replacing
ANTENNA_BURN), BURN_W2B, EOL_ARM, EOL_FIRE, LOCK_CLR, GYRO_EN, IMU2_EN, SNS_EN, GYRO_RB, IMU2_RB, SNS_RB, GYRO_RST_N,
GYRO_DRDY, IMU2_DRDY, MAG_DRDY, MTQ_X_DIR, MTQ_Y_DIR, MTQ_Z_DIR, MTQ_FAULT_N, MTQ_EXPIRED, TONE_OK, XPDR_KEY, JET_PG,
GNSS_PG, WDI; ANTENNA_SENSE removed. The BMI088's second chip select needs either a GPIO the driver drives or a
second CS in `hal_spi`.

## 7. Interface Changes Needed on Other Boards (for the chair)

| # | Board | Change | Reason |
|--:|-------|--------|--------|
| E1 | EPS (SQSO-EPS-BRD-001 §13.2) | J3-19: VJET_RB, 1.00 MΩ / 200 kΩ (0.1 %) + 100 nF from VBAT_JET (after U19) | RID AV-06 readback of the eFuse |
| E2 | EPS | J3-37: VSW1_RB, 1.00 MΩ / 200 kΩ + 100 nF from VBAT_SW1 (after SW1) | RID AV-06 readback of SW1 |
| E3 | EPS | J3-36 and J3-39 (was GND): VBAT_MTQ from a new switch SW11, TPS1H000-Q1 class on VBAT, enable ADCS_EN, current limit ≥ 0.25 A min (the three bridges at their hardware limit draw 0.23 A at 11.2 V) and ≤ 0.5 A | the CR0002 rods need more than 3.3 V for 0.2 A·m² (§3.4; OBC_BOARD_DESIGN §8.1) |
| E4 | EPS | J3-38: 3V3_RTC = 3V3_AO through 1.0 kΩ | RID AV-10: the mission clock is not on the OBC's switched rail |
| E5 | EPS | J20 (3V3_ADCS) goes to OBC J6 on W6B, not to an ADCS board; SW4 unchanged (0.30 A) | there is no ADCS board |
| E6 | EPS | J21 (3V3_GNSS) goes to OBC J4 on W9B | the OBC's GNSS translators take their receiver side from this rail |
| E7 | EPS | the "OBC pin" column of §13.2: J3-9 PB2, J3-13 PF3, J3-31 PE7, J3-32 PF4, J3-33 PG1, J3-34 PF5 | §6 rows 1, 2, 7, 9 |
| E8 | EPS | 3V3_AO set point ± 2 % (both the RF board's LMX2571 and this board's M-G370 need ≥ 3.15 V; at ± 3 % the gyro has 45 mV) | margin |
| E9 | Jetson carrier | its 3.3 V I/O rail on W5B pin 4, ≤ 1 mA; **done** in SQSO-JET-BRD-001 | RID AV-09 isolation reference; AV-06 power-good |
| E10 | Jetson carrier (from the carrier) | W5B 6 pins, spreader NTC on pins 5–6: **taken** (OBC J3, PA2) | spreader temperature with the carrier off |
| E11 | Jetson carrier | the 20–30 mm magnetic keep-out around (+33, −38) asked by MECHANICAL_DESIGN §4.4 is **withdrawn**: the magnetometer leaves the OBC (SQSO-OBC-BRD-001 §9.4) | — |
| E12 | Mechanical / harness | magnetometer board A2 (24 × 24 mm, ≈ 3 g with harness) at the +Z end frame slot of the CAD (+24, +27) mm, or (+24, −27) if the mass properties can be re-trimmed (0.51 against 0.83 µT per mA·m² of rod remanence); 20 mm non-ferromagnetic zone around it (the RF shield's top, the patch adapter, the trim masses and fasteners non-magnetic); harness W6E through a ±X wall gap; W8A internal access-port harness | §9.4 of the OBC board |
| E13 | RF board (SQSO-RF-BRD-001 §4.1) | W4A version 3: RF J1 becomes FTSH-108-01-L-DV-K (16 positions; pins 1–14 unchanged); pin 15 RX_CD (high = receiver carrier detect), pin 16 RX_FAULT (high = receiver fault), both 3.1 V push-pull through 1 kΩ with 100 kΩ pull-downs on the RF board (taken in SQSO-RF-BRD-001 B §4.1); W4 wiring source to version 3; J1 placed in the (+X, −Y) corner to meet OBC J2 (SQSO-OBC-BRD-001 §21) | flight-software round 2 |
| E10 | RF (SQSO-RF-BRD-001) | none: the OBC side of J1 is as the RF board asks (pull-downs fitted on the OBC) | — |

## 8. Verification Items (interconnect)

| ID | Item | Method | Pass | When |
|----|------|--------|------|------|
| V-ICX-1 | Pin-to-pin continuity and isolation of W3, W4A, W5B, W6, W8, W9 against this document | Test (harness tester) | every pin as §3, isolation ≥ 100 MΩ at 50 V | each harness, before integration |
| V-ICX-2 | Reset states: hold the OBC in reset and unpowered; measure every EPS enable, burn, passivation, RF_TX_EN, XPDR_EN, TX_PWR_SEL | Test | each < 0.4 V (low) at the far board | flatsat |
| V-ICX-3 | Back-power: OBC rail off, every other board on, readbacks forced high | Test | 3V3_OBC < 0.5 V; current into the OBC ≤ 3.0 mA | flatsat |
| V-ICX-4 | Carrier and receiver isolation: Jetson carrier off with USART1 active; GNSS off with LPUART1 active | Test | carrier 3.3 V rail < 0.3 V (ICD-002 §9); receiver VIN < 0.3 V | flatsat |
| V-ICX-5 | Firmware pin map against §4.1 | Inspection of `gpio.rs`/`main.rs` after the flight-software change, and a GPIO walk on the EM | every pin as §4.1 | EM software release |

## 9. Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-27 | First issue: master pin allocation of the STM32L4R5ZIT6, the board-to-board signal lists for W3, W4, W5, W6, W8, W9, domain crossings, the firmware change list and the interface changes for the EPS and the Jetson carrier (CDR design completion, OBC board SQSO-OBC-BRD-001) |
| B | 2026-10-27 | Carrier interface E10 taken (W5B 6 pins, spreader NTC on PA2 ADC1_IN7; JETSON_PWR PA2 → PA8); magnetometer moved to board A2 at +Z (W6E); W8A internal access-port harness and J9 as FTSH-105; stack routing rules of MECHANICAL_DESIGN §4.4; changes 26–28 for the flight software (28: RX_CD PG0, RX_FAULT PF13 on W4A v3 pins 15–16, FTSH-108); EOL_ARM PF4, EOL_FIRE PG1, BURN_W2B PE7, LOCK_CLR PF5 confirmed low at reset; gyro at (−20, +30.5); back-power total 2.7 mA; E10–E13; W4A routing (no fold, flat on the +X shear panel); bundle allocation §2.1 (W9A/W9B to (+X, +Y); W6B/W6C/W6D in (−X, +Y)); W4B twisted pairs |
