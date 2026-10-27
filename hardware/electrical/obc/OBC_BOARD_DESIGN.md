# SMART-QSO OBC Board: Build-to Design

**Document ID**: SQSO-OBC-BRD-001
**Version**: C
**Date**: 2026-10-27
**Status**: CDR build-to electrical design for the engineering model (EM): schematic captured as code with selected,
orderable parts, netlist, BOM, ERC (clean, waivers listed) and calculations. No PCB layout, no hardware; every number
below is analysis until its verification item (§22) closes. Version B answers the chair's follow-up: the carrier's
spreader thermistor (E10, §11.3), the stack placement constraints (§21), the magnetometer location (§9.4: moved to a
small board A2 at the +Z end frame) and the gyro vibration assessment (§9.2). Version C takes the harness design
(SQSO-HAR-001 2.0 §15 items O-1 to O-3: right-angle Samtec TFM headers, gold Pico-Clasp headers, J8 as a 12-way part)
and adds the access-port board A3 (§15.1).

**Design files** (this directory): [`design.py`](design.py) (SKiDL 3.0 schematic, the source of truth for parts, nets
and the MCU pin map), [`obc_board.net`](obc_board.net) (KiCad 8 netlist, generated), [`obc_board_nets.csv`](obc_board_nets.csv)
(net to pins), [`obc_board_bom.csv`](obc_board_bom.csv) (BOM), [`obc_board_erc.log`](obc_board_erc.log) (ERC),
[`obc_pinmap.csv`](obc_pinmap.csv) (every STM32 pin), [`mag_board.net`](mag_board.net), [`mag_board_bom.csv`](mag_board_bom.csv),
[`mag_board_erc.log`](mag_board_erc.log) and [`mag_board_nets.csv`](mag_board_nets.csv) (the magnetometer board A2, built by
the same script), [`a3_board.net`](a3_board.net), [`a3_board_bom.csv`](a3_board_bom.csv), [`a3_board_erc.log`](a3_board_erc.log)
and [`a3_board_nets.csv`](a3_board_nets.csv) (the access-port board A3), [`obc_calcs.py`](obc_calcs.py) with its output `obc_calcs.json` (every computed value here). Regenerate: `python3 hardware/electrical/obc/obc_calcs.py`, then
`<CAD venv>/bin/python hardware/electrical/obc/design.py`. The master pin allocation and the board-to-board signal
lists are in [INTERCONNECT.md](../INTERCONNECT.md) (SQSO-ICX-001).

**Governing documents**: `docs/BASELINE_PARAMETERS.md` Rev D (governs); `hardware/obc/OBC_DESIGN.md` (SQSO-CDH-OBC-001
1.7, §13 open items); `docs/interfaces/ICD_OBC_ADCS.md` 1.1 §12, `ICD_OBC_JETSON.md` 1.2, `ICD_OBC_EPS.md` 1.1,
`ICD_OBC_RF.md` 2.1; `docs/hardware/ADCS_DESIGN.md` 1.5, `PAYLOAD_DESIGN.md` 1.1 §11, `BUILD_VS_BUY.md` §2.5–§2.7, §5;
`docs/environment/RADIATION_ANALYSIS.md` Rev C §5–§6; the EPS board SQSO-EPS-BRD-001 §8.3, §13.2 and the RF board
SQSO-RF-BRD-001 §4.1 (the other ends of every interface); `docs/manufacturing/EEE_PARTS_LIST.md` Rev C §2A.

---

## 1. Summary

| Item | Value |
|------|-------|
| Board | PC/104 90.17 × 95.89 mm, 6 layers, in the OBC/EPS stack (W3 ribbon to the EPS, no stack connector) |
| Placements / BOM lines / nets | OBC 351 / 69 / 253; magnetometer board A2 15 / 9 / 19; access-port board A3 8 / 6 / 13 (generated); ERC: no errors, no warnings (five waivers on the OBC, two each on A2 and A3) |
| Processor | STM32L4R5ZIT6 (LQFP-144) at 80 MHz from an 8 MHz XO (HSE bypass); LSE 32.768 kHz crystal |
| Memory | FM25V20A 2 Mbit SPI FRAM (no external flash: §7) |
| Supervisor | TPS3850H33-Q1: 3.3 V UV/OV (± 7 %) and a window watchdog, 5.6–33.3 s window, kicked every 10 s, wired to NRST (RID AV-02) |
| Mission clock | RV-3028-C7 on the always-on 3V3_AO (± 1 ppm, 32-bit UNIX counter), time-stamps every NRST edge (RID AV-10) |
| ADCS on the board | Epson M-G370PDT0 gyro (socket X1 at (−20, +30.5) mm, through the carrier window), Bosch BMI088 cold-spare gyro, six sun-sensor transimpedance amplifiers into an ADS7138-Q1, three DRV8876-Q1 H-bridges for the CR0002 rods with a hardware off-timer (RID AV-03); the PNI RM3100 magnetometer on board A2 at the +Z end frame (harness W6E from J10), powered, protected and isolated from this board |
| Jetson spreader temperature | carrier thermistor RT1 on J3-5/-6, 10.0 kΩ 0.1 % bias from VDDA, PA2 (ADC1_IN7): readable with the carrier off (carrier E10, §11.3) |
| Sensor rails | three TPS2553-Q1 latch-up switches (50–100 mA, auto-retry 0.13 s) from the EPS's 3V3_ADCS |
| Isolation | every line into a domain the MCU does not power passes a part with VCC isolation (Jetson: RID AV-09; GNSS; sensors) |
| Power (3V3_OBC) | 25.4 mA worst normal (84 mW); EPS SW10 trips at 130 mA minimum (5.1×) |
| Power (sensors, 3V3_ADCS) | 76 mW normal (gyro 53 mW); 49 mA worst with the backup gyro on |
| Magnetorquers (VBAT_MTQ) | 0.39 W per rod at 0.2 A·m² (1.16 W three saturated), bridges 0.12 W while awake; hardware current limit 0.104–0.126 A per rod |
| Worst junction | DRV8876 65 °C, MCU 63 °C at a 60 °C board (§17) |
| Derating | EEE-INST-002 (tailored, SQSO-HAR-001 §5.2) and the 50 % rule, whichever is stricter (§2.3); all parts within it (§18) |

## 2. Requirements, Standards and Departures

### 2.1 What this board implements

| Requirement | Source | Implementation | § |
|-------------|--------|----------------|---|
| OBC part, build-to | RID SE-01 (OBC share) | this document, `design.py`, netlist, BOM | all |
| Schematic and master pin allocation, no conflicts | RID AV-01 | `design.py` builds the MCU from one pin table; SQSO-ICX-001 §4 | ICX |
| External supervisor independent of the MCU | RID AV-02 | TPS3850H33-Q1 with its own oscillator, on NRST | §6 |
| Gyro, backup gyro, magnetometer, sun sensors, three-axis coil drive, switched sensor rail | RID AV-03, ICD-004 §12 | §9, §10 | §9–§10 |
| Repeat-path readbacks | RIDs RF-21, AV-05 | TONE_OK PC9, XPDR_KEY PG12, PTT_EXCITER PC7 (capture) | §13 |
| Receiver status inputs | flight-software round 2 | RX_CD PG0 (EXTI0), RX_FAULT PF13 (EXTI13, low = healthy when open) on W4A v3 pins 15–16 | §13 |
| PAYLOAD_EN and JETSON_PWR independent, with readback | RID AV-06, PAYLOAD_DESIGN §11.1 | ports B and A; three readbacks | §11 |
| No back-powering of the unpowered Jetson | RID AV-09, ICD-002 §9 | SN74LVC1T45-Q1 pair powered from the carrier | §11 |
| Mission clock across resets | RID AV-10 | RV-3028-C7 on 3V3_AO | §8 |
| GNSS_EN not on PE4 | RID EPS-14 | GNSS_EN PE5 (unchanged firmware pin), XPDR_EN PE4 | ICX §3.1 |
| 900 ms burst, 50 ms settle; the coils cannot stay on with the OBC hung | ICD-004 §12.2, SRS-F090–F092 | TIM15 one-pulse + hardware off-timer on the bridges' nSLEEP | §10 |
| Latch-up: current-limited, tripping supply for every COTS part without SEL data | RADIATION_ANALYSIS §6 | OBC on EPS SW10; sensors on U5–U7 | §9.1, §16 |
| ESD on every external line | brief | TPD4E05U06-Q1 / TPD1E05U06-Q1 at every connector; TVS on both rails | §16 |
| OBC-F002 to F006 (EPS I2C, Jetson UART, load-switch GPIO, NVM, hardware watchdog) | OBC_DESIGN A.2.1 | §4–§14 | — |
| −40 to +85 °C parts | OBC_DESIGN §1.1 | every part −40 °C or wider, +85 °C or wider | §19 |
| Jetson spreader temperature with the carrier off (guard 70 °C, power-on gate −15 °C) | carrier interface E10 (SQSO-JET-BRD-001 B) | NTC bias and ADC input on the always-on OBC | §11.3 |
| Stack placement constraints | MECHANICAL_DESIGN §4.3–§4.4 | gyro position, part heights, connector edges, wall-gap routing | §21 |

### 2.2 Departures from the architecture documents (for the chair)

| Item | Document | This design | Why |
|------|----------|-------------|-----|
| ADCS board | ADCS_DESIGN §6, W6 v1, S6 | **none**: sensors and coil drivers on the OBC | one board fewer, no stack connector; the 1U stack has no room for a fifth board |
| Magnetometer | BUILD_VS_BUY §2.7: buy (M315, MM200, Lodestone) | **PNI RM3100 on a 24 × 24 mm board A2 at the +Z end frame** (the CAD's magnetometer slot), on harness W6E from J10; its switch, bus isolation and data-ready buffer stay on the OBC. Version A had it in the OBC's (+X, −Y) corner | §9.4: noise, single-shot timing under the MCU's control, cost, mass; at +Z the rod remanence couples 1.4× less than in the OBC corner and the Jetson's input current is 63 mm away instead of 12 mm |
| JETSON_PWR pin | SQSO-ICX-001 A: PA2 | **PA8** (still port A) | PA2 (ADC1_IN7) is the only ADC input that can be freed for the spreader NTC (§11.3) |
| GSE header | W8 v2: TST-105 behind the access window | **TFM-105 right-angle (J9) on an internal harness W8A** to the access-port board A3, whose Micro-D 21 is the one GSE connector at the window (§15.1) | the OBC sits mid-stack (z +5), not at the −Y access window (§15) |
| Coil driver | ADCS_DESIGN §6.1: DRV8833 on 3.3 V | **DRV8876-Q1 from VBAT** with hardware current regulation | a CR0002 rod (≈ 51 Ω) needs 4.4 V for 0.2 A·m²: 3.3 V gives at most 0.15 A·m² (§10.1) |
| Coil supply | EPS SW4 3V3_ADCS "sensors, gyro, magnetorquer drivers" | 3V3_ADCS feeds only the sensor switches; **a new EPS switch SW11 (VBAT_MTQ)** feeds the bridges | as above; and a coil current no longer masks a sensor latch-up (RADIATION_ANALYSIS §5.3) |
| PAYLOAD_EN pin | firmware, EPS §13.2: PB4 | **PB2** | PB4 resets as NJTRST with a pull-up: SW1 on at every reset (§11.2) |
| FRAM pins | STORAGE_DESIGN §3.4, firmware: PA4–PA7 | **PG2–PG5** | ADC1 has only 16 external inputs; PA4–PA7 are four of them |
| AV-09 fix | ICD-002 §9: buffer on the carrier, or firmware | **buffer on the OBC**, referenced to the carrier's rail | works for any carrier; the carrier only supplies its 3.3 V on one pin |
| Watchdog kick | OBC_DESIGN §13.2: TPS3851-class | TPS3850 **window** watchdog | a runaway loop kicking too fast is caught too |
| TMP117 on I2C2 (0x48 clash, finding F10) | S3 | **TMP235-Q1 analog** on PB1 | no I2C2 device on the OBC; adc.rs scale unchanged |
| GSE plug | W8 v1: UMBILICAL_SENSE to GND | to VTREF (high) | the firmware reads high = present (`hal_gpio.h`) |

### 2.3 Derating standard and part rules

Derating to **EEE-INST-002** (NASA GSFC, as tailored by SQSO-HAR-001 §5.2), with the 50 % rule where stricter, the same
rule set as the EPS and RF boards: resistors ≤ 50 % power, ≤ 80 % voltage; ceramic capacitors ≤ 50 % voltage;
microcircuit supply ≤ 80 % of absolute maximum (≤ 90 % of the maximum operating voltage where the operating range is
narrow, as for the 3.3 V parts); output currents ≤ 75 %; junction ≤ 110 °C for microcircuits; connectors ≤ 50 % of
contact current; wire per SQSO-HAR-001 §5.2. Parts: AEC-Q100/Q101/Q200 where offered, otherwise industrial −40 °C or
wider and Active (§19). Matte-tin terminations take the EEE_PARTS_LIST Rev C §2A.3 mitigation: Sn63Pb37 assembly,
hot-solder-dip re-tin of leaded parts where possible, conformal coat (Arathane 5750, NASA-STD-8739.1) over every
termination except the connector mating faces and X1.

## 3. Architecture and Domains

```
 EPS J3 ──W3──> J1: 3V3_OBC ─D2/22u─┬─ U1 STM32L4R5ZIT6 ── U3 FRAM (SPI1 PG2-5)      J9 ─W8A─> access port (SWD,
                                    │      │  │  │           U2 TPS3850 -> NRST          NRST, USART3, UMB_SENSE, WDT_DIS)
                                    │      │  │  └ I2C1 ──── U4 RV-3028 (3V3_RTC <─ J1-38, always on)
                                    │      │  ├─ USART1 ─ U28/U29 (VCCB = JET_3V3) ── J3 Jetson carrier (W5B)
                                    │      │  ├─ PA2 ADC1_IN7 <─ 1k ─┬─ 10.0k ─ VDDA     J3-5/-6 spreader NTC RT1
                                    │      │  ├─ LPUART1/PPS ─ U30-U32 (VCCB = 3V3_GNSS) ── J5 receiver <─ J4 <─ EPS SW7
                                    │      │  ├─ USART2, readbacks, monitors ── J2 RF board
                                    │      │  └─ enables, burns, EOL, ADC ── J1 EPS
 EPS J20 ──W6B──> J6: 3V3_ADCS ─┬─ U5 ─ 3V3_GYRO ─ U9/U10 ─ U8 M-G370 (X1) on SPI3
                                ├─ U6 ─ 3V3_IMU2 ─ U12/U13 ─ U11 BMI088 on SPI2
                                └─ U7 ─ 3V3_SNS ── U14 (I2C4) ─ U16 ADS7138 <─ U17-U19 TIAs <─ J8 sun sensors
                                                   └─ U14, U33 ─ J10 ──W6E──> board A2: RM3100 (+Z end frame)
 EPS SW11 ──W3──> J1-36/39: VBAT_MTQ ─D3─ U22/U23/U24 DRV8876 ── J7 rods X, Y, Z
                 nSLEEP = MTQ_ARM (TIM15 one-pulse, PF9) AND NOT U25 off-timer (0.92-0.98 s)
```

| Domain | Source | Switched by | Loads | Normal / limit |
|--------|--------|-------------|-------|----------------|
| 3V3_OBC | EPS SW10 TPS2553-Q1 from 3V3_AO | always on, auto-retry after a latch-up trip | MCU, FRAM, supervisor, XO, logic, translators' A side, spreader NTC bias | 25.4 mA / 130–176 mA |
| 3V3_ADCS | EPS SW4 TPS2553-Q1 | ADCS_EN (PB6) | U5, U6, U7 only | 49 mA / 265–339 mA |
| 3V3_GYRO, 3V3_IMU2, 3V3_SNS | U5, U6, U7 TPS2553-Q1 (ILIM to IN) | GYRO_EN PF6, IMU2_EN PF7, SNS_EN PF8 | M-G370; BMI088; RM3100 (on A2, through J10), ADS7138, TIAs, TMP235, U14 B side | 26 / 8 / 15 mA; 50–100 mA |
| 3V3_RTC | EPS 3V3_AO through 1.0 kΩ | never | RV-3028-C7 | 45 nA |
| VBAT_MTQ | new EPS SW11 (VBAT) | ADCS_EN | three DRV8876 | 0.083 A at 3 × 0.2 A·m²; 0.23 A at the bridges' limit |
| JET_3V3 | the carrier's own 3.3 V | the carrier being powered | U28/U29 B side, JET_PG | < 1 mA |
| 3V3_GNSS | EPS SW7 | GNSS_EN PE5 | receiver (through J4 → J5), U30–U32 B side | 0.10 A / 0.175–0.232 A |

## 4. MCU Supply and Decoupling

- **Entry.** 3V3_OBC arrives on J1-1/2 (two 28 AWG wires, 0.088 A each at the SW10 trip). D2 SMF5.0A (5.0 V standoff,
  9.2 V clamp; 3.4 V is 68 % of the standoff, leakage < 1 µA), C1 22 µF and C2 10 µF bulk.
- **Inrush.** 39.7 µF on 3V3_OBC charges in 1.0 ms at SW10's minimum limit (0.130 A), well inside its 8 ms FAULT
  deglitch: the OBC does not trip its own supply at power-up. Keep the 3V3_OBC capacitance below 150 µF at layout.
- **Decoupling** (ST AN5200 pattern): 100 nF at each of the nine VDD pins, both VDDIO2 pins (port G is used, so VDDIO2
  is tied to VDD), VDDUSB (USB unused, tied to VDD) and VBAT (tied to VDD; no backup cell: the mission clock is the
  external RTC on the always-on rail, §8). VDDA and VREF+ through FB1 (600 Ω at 100 MHz) with 1 µF + 10 nF and
  1 µF + 100 nF; VSSA and VREF− to the ground plane at the MCU.
- **Current.** 25.4 mA worst normal (MCU 20 mA allocation at 80 MHz with the peripherals on, XO 3 mA, the spreader NTC
  bias 0.28 mA at 70 °C, the rest 2.1 mA;
  `obc_calcs.json` i_3v3_obc_items). The MCU figure is an allocation: ST's datasheet DS12023 could not be reached from
  this session (§25); the EM power test (V-OBC-12) replaces it.
- **Unused pins**: 13 spare pins (SQSO-ICX-001 §4.4) go to analog mode in the firmware; PB3 (SWO) reaches a test point.

## 5. Clocks

| Clock | Part | Use | Numbers |
|-------|------|-----|---------|
| HSE | Y1 Epson SG-8018CA 8.000 MHz CMOS XO (± 50 ppm, −40…+105 °C), 33 Ω into PH0, HSE bypass | PLL M 1, N 20, R 2 = 80 MHz (Range 1) | ± 50 ppm on the 900 ms burst (45 µs), the UART baud rates and the time base; MSI stays the clock-security fallback |
| LSE | Y2 Micro Crystal CM7V-T1A 32.768 kHz, CL 7 pF, ± 20 ppm, AEC-Q200; 10 pF + 10 pF C0G | MCU RTC, MSI PLL-mode trim | gm_crit 0.84 µA/V (ESR 70 kΩ, C0 1.4 pF; AN2867), 2.0× below the LSEDRV medium-high limit: set LSEDRV = medium-high; load 7.5 pF with 2.5 pF stray (about −9 ppm pull for a 2.5 fF motional capacitance, irrelevant: the mission clock is U4) |
| LSI | internal | IWDG only | 29.5–34 kHz (class value) |

The firmware today runs from MSI 4 MHz through the PLL. Switching `clocks()` to the HSE is recommended, not required
(ICX §6 row 23): the board works either way. Clock security on the HSE falls back to the internal oscillator.

## 6. Reset and External Supervisor (RID AV-02)

U2 TPS3850H33-Q1 (AEC-Q100 grade 1) watches 3V3_OBC and the MCU, with its own oscillator and capacitor-set timing,
independent of the MCU's clocks and of the IWDG's LSI.

| Function | Setting | Value (`obc_calcs.json` wd_*) |
|----------|---------|-------------------------------|
| Under/over-voltage | H33 variant, ± 7 % window, ± 0.8 % accuracy | reset below 3.069 V or above 3.531 V; 3V3_AO ± 3 % (3.20–3.40 V) stays inside |
| Reset delay | CRST open | 200 ms (170–230 ms) after the rail is good |
| Window watchdog upper bound | CWD = 5 × 100 nF C0G (± 5 %): tWDU = 77.4 × C(µF) + 0.055 s | 38.8 s typical, **33.3 s minimum**, 44.6 s maximum |
| Lower bound | SET0 = SET1 = 0: 1/8 of tWDU | **5.6 s maximum** |
| Kick | flight software: WDI falling edge every 10 s while every task has checked in (the same condition as the IWDG refresh); the first edge 10 s after boot, never within 6 s of the previous one (an early edge is a window fault) | 1.8× above the lower bound, 3.3× below the upper; boot to first kick 10 s, inside the 33 s bound with SRS-F001's 30 s initialisation |
| Against the IWDG | IWDG reload 3750 / 256 on LSI: 28.2–32.5 s | the IWDG always acts first (0.8 s margin at the extremes): U2 is the backstop for a dead LSI, an upset IWDG or a loop that kicks too fast |
| Outputs | RESET and WDO, open drain, wired to NRST with 10 kΩ pull-up and 100 nF | a watchdog fault resets the MCU for 200 ms |
| Disable for ground debugging | SET0 high (SET1 low) through J9-5 (WDT_DIS) | latched at the next supervisor reset (datasheet §7.3), i.e. after a power cycle; 100 kΩ holds SET0 low in flight |

**Reset chain.** NRST is driven by: U2 (voltage or watchdog), the MCU itself (IWDG, software, brown-out), the GSE probe
(J9-6, 100 Ω). It feeds the RTC's event input (§8), so every reset is time-stamped outside the MCU. The MCU's reset
cause register reports U2's resets as "pin reset"; the RTC event log tells a supervisor reset from a GSE reset by its
time and the IWDG flag. A latch-up in the MCU that NRST cannot clear is cleared by SW10 on the EPS (current trip and
retry); an OBC rail stuck in a non-tripping fault is not detected by anything on this board (for the chair: an EPS
controller check of the OBC heartbeat over I2C would cover it; not in either board's design).

## 7. Non-Volatile Memory

U3 FM25V20A (Infineon, 2 Mbit, SOIC-8, 40 MHz, −40…+85 °C) on SPI1 at 20 MHz (50 % of its rating), PG2 SCK (33 Ω
series), PG3 MISO, PG4 MOSI, PG5 CS# (10 kΩ pull-up), WP# PE0 and HOLD# PE1 (10 kΩ pull-ups; the firmware drives them
high). The 12 regions of STORAGE_DESIGN §3.3 and `fram.rs` are unchanged; only the pins move (ICX §6 row 3). FRAM cells
are not upset at these doses (RADIATION_ANALYSIS §5.2).

No external flash: the 2 MB dual-bank internal flash holds two images (a golden and an operational one, OBC_DESIGN
§3.1), and the housekeeping store (≈ 88 kB, BASELINE §8) fits the FRAM. A second FRAM would remove the FRAM as a single
point (SEFI 8 × 10⁻⁶/day high estimate); not added, because every record is already duplicated with CRC inside the
part and a FRAM SEFI is cleared by the OBC power cycle.

## 8. Mission Clock (RID AV-10)

U4 RV-3028-C7 (Micro Crystal, TA QA: −40…+85 °C, AEC-Q200) on its own supply 3V3_RTC, which the EPS derives from the
always-on 3V3_AO through 1.0 kΩ on J1-38 (interface change E4). It keeps running across OBC resets, OBC rail power
cycles and latch-up trips, and stops only if 3V3_AO is lost (the whole spacecraft dark), after which the GNSS fix or a
ground time command re-sets it.

| Item | Design |
|------|--------|
| Time | 32-bit UNIX seconds counter plus calendar; factory calibrated ± 1 ppm at 25 °C (0.09 s/day); ≈ 0.3 s/day worst over −40…+85 °C (class value) |
| Bus | I2C1 (PG14 SCL / PG13 SDA, 4.7 kΩ to 3V3_OBC, 100 kHz), address 0x52, alone on its bus so a fault on the EPS or sensor bus cannot stop the clock |
| Reset time stamps | EVI from NRST through 100 kΩ (1 MΩ to GND keeps it defined with the OBC off): the RTC records the time and count of NRST events, including the OBC losing power |
| Clock check | CLKOUT (set to 1 Hz in the RTC's EEPROM at integration) through 1 MΩ to PC8 (TIM3_CH3): the firmware can measure its own clock against the RTC; 3.3 µA into an unpowered MCU |
| VBACKUP | not used, 10 kΩ to VSS (application manual §2.2) |
| Flight software | read the counter at boot and on each GNSS fix; keep the GNSS-to-RTC offset in FRAM; log the EVI time stamp as the reset time (driver is flight-software work, ICX §6 row 13) |

## 9. ADCS Sensors (RID AV-03)

### 9.1 Latch-up-protected sensor rails

Each sensor has its own TPS2553-Q1 (U5 gyro, U6 BMI088, U7 magnetometer and sun-sensor front end) from 3V3_ADCS, with
ILIM tied to IN: **current limit 50 / 75 / 100 mA (min / typ / max)**, reached in 2 µs; after the 8 ms FAULT deglitch
FAULT pulls EN low (R_FAULT 100 kΩ, C_RETRY 2.2 µF): the rail is off for **0.13 s** (0.6 × 100 kΩ × 2.2 µF; ≥ 0.1 s,
RADIATION_ANALYSIS §5.3) and retries; a persisting latch-up repeats the cycle. BAT54 diodes make a commanded turn-off
fast. Each EN node is read back through 100 kΩ (GYRO_RB PF10, IMU2_RB PF11, SNS_RB PF12): low with the enable high
means a trip.

| Rail | Normal maximum | Limit minimum | Margin |
|------|----------------|---------------|--------|
| 3V3_GYRO (M-G370, 16 mA typical × 1.6) | 26 mA | 50 mA | 1.95× |
| 3V3_IMU2 (BMI088, 5 mA × 1.6) | 8 mA | 50 mA | 6.3× |
| 3V3_SNS (RM3100 on A2, ADS7138, three TLV9002, TMP235, TCA9517 B side, up to 12 mA photocurrent) | 15 mA | 50 mA | 3.3× |

RADIATION_ANALYSIS §5.3 proposed 40 mA for the gyro; the lowest TPS2553-Q1 setting is 50–100 mA, which a latch-up
(typically hundreds of milliamperes) still trips. The EPS's SW4 (0.265–0.339 A) is the upstream breaker for all three.

The gyro needs VCC 3.15–3.45 V. Worst case at the gyro: 3V3_AO − 3 % (3.201 V) less SW4, the W6B harness, U5 and the
trace: **3.195 V, 45 mV above the minimum**. Recommended to the EPS designer: a ± 2 % set point (interface change E8).

### 9.2 Primary gyro: Epson M-G370PDT0

The module (24 × 24 × 10 mm, 10 g, Samtec FTMH-110-02-L-DV-ES header) plugs into X1, a Samtec CLM-110-02-L-D socket
(Epson's recommended host part, data sheet table 2.2), and is screwed to the board through its two mounting holes.

| Signal | Module pin | OBC | Note |
|--------|-----------|-----|------|
| SCLK, SDI, /CS | 1, 5, 6 | PC10, PC12, PD1 via U9 TXU0304-Q1 | SPI mode 3, ≤ 1 MHz in burst mode (2 MHz otherwise) |
| SDO | 2 | PC11 via U9 | — |
| DRDY | 13 | PD10 (EXTI10) via U10 TXU0204-Q1 | data ready, polarity set by MSC_CTRL |
| /RST | 16 | PD14 via U10 | ≥ 100 ms low pulse (data sheet); 100 kΩ holds it low while the MCU is in reset |
| SIN | 9 | 10 kΩ to 3V3_GYRO | unused UART input to VCC (data sheet table 1.6 note 2); SOUT, GPIO2, NC open |
| VCC | 10, 11, 12 | 3V3_GYRO, 10 µF + 100 nF | 16 mA typical |

U9 and U10 take their B side from 3V3_GYRO and their OE from the same rail, so with the gyro switched off (or tripped)
the SPI, reset and data-ready lines are high-impedance on the gyro side (VCC isolation) and nothing back-powers the
module. The firmware needs no pin juggling when it switches the rail.

**Mechanical (MECHANICAL_DESIGN §4.3, decision D-11; closes item C3).** X1 is centred at **(−20, +30.5) mm** on the
top side; the module (footprint x −32 … −8, y +18.5 … +42.5) stands 14.0 mm, top at z +20.6, through a 27 × 27 mm
window in the Jetson carrier (2.0 mm thick, no frame: MECHANICAL_DESIGN D-11 as amended), 1.5 mm clear on every
side; nothing is above it below the spreader (z +27.8), 7.2 mm. The chair's note gave (−20, +27.5); the mechanical design then moved window and gyro 3 mm toward +Y to clear the
carrier's module standoff MP2 at (−31.3, +12.6), and this board follows MECHANICAL_DESIGN. The footprint stays clear of
the Ø7 mm keep-outs of the corner column (−40, +43) and the centre support (0, +21) (§21). With the frame withdrawn only
X1's own clearance in the window applies; elsewhere under the carrier 9.0 mm is free (§21).

**Vibration and shock.** The FE gives the module, mounted as above, these responses to the 8.18 Grms protoflight
random input (MECHANICAL_DESIGN §4.3.2, in-axis, mean of the footprint nodes):

| Input axis | Response at the module (in axis) | 3σ peak | Cross axes |
|------------|----------------------------------|---------|------------|
| X | 22.2 Grms | 67 g | 4.5 / 10.5 Grms |
| Y | 25.0 Grms | 75 g | 3.1 / 11.7 Grms |
| Z | 21.7 Grms | 65 g | 1.8 / 3.2 Grms |

Epson's data sheet (M-G370PDG0, Rev 1.2, table 1.1, absolute maximum ratings) gives only **shock 1000 g, half-sine
0.5 ms**; it gives no random or sine vibration rating, and no qualification levels were found in Epson's public
material (the PDT0 variant's sheet was not obtained; the PDG0 sheet is the same family and package). As a quasi-static
load the 3σ peaks are 13–15× below the shock rating, so the module will not be overstressed by a single peak; but a shock
rating says nothing about fatigue of the module's internal interconnects, its FTMH header in the CLM socket or the
sensing elements' bias under 60 s per axis of 22–25 Grms broadband input, and the stack modes (326–396 Hz) put most of
that energy in one band. The rating is therefore **neither shown exceeded nor shown met**.

Decision: **no isolation; a verification item instead (V-OBC-16)**. An isolator would add a resonance (an elastomer
mount for a 10 g module tuned well below 326 Hz is tens of hertz to about 150 Hz), would need a flexible link in place
of the socket (the module has only its rigid header), and would make the 0.1° alignment of VI-MCAD-05 depend on the
elastomer's creep and temperature; the 1.5 mm window clearance also leaves no room for isolator travel. V-OBC-16: the
OBC EM with an M-G370 of the flight lot fitted on X1 with its two screws (locked by prevailing-torque, all-metal
self-locking female threads in the board-side standoffs, no anaerobic thread-locker (MATERIALS_LIST: Loctite 242 is
non-compliant for outgassing); screw heads and the module edge staked with Scotch-Weld 2216, staking only, not
structural), random vibration at the FE-predicted module levels (the stack input at
protoflight level and duration, or a board-level input shaped to give ≥ 22.2 / 25.0 / 21.7 Grms at the module), and a
shock test to the launch provider's SRS when it is issued; functional tests before and after on a rate table: bias
change ≤ 0.01 °/s (10 % of the 0.1 °/s budget of SENSOR_DESIGN §4.1), scale factor change ≤ 0.2 %, noise within 10 %
of the pre-test value, no DIAG_STAT flags, no SPI errors. The gyro is unpowered during launch, so no in-test function
is needed; a continuity monitor on the header during the test catches intermittent contact. Also asked of Epson
(§24 C12): its vibration and shock qualification conditions for the M-G370 family. If V-OBC-16 fails, the fallback
is a bracket-free isolation at X1 (silicone grommets under the two screws with a short flexible jumper in place of the
socket; the 7 mm of free height above the module, below the spreader, allows it), re-analysed and re-tested; the mechanical designer's
VI-STR-02 board-level random test is the same test and can carry it.

### 9.3 Backup gyro: Bosch BMI088

U11 (LGA-16) on 3V3_IMU2, powered only if the M-G370 fails (BUILD_VS_BUY §5.3), and calibrated on a rate table over
−20…+50 °C before integration. PS = GND selects SPI for the gyro; the accelerometer is selected by its own CSB1 (PD4),
put in SPI mode by the driver's first dummy read. SDO1 and SDO2 are joined (each is high-impedance while its CSB is
high). SPI2 (PB13/PB14/PB15, gyro CS PD0, the former "RF transceiver" bus that the RF board freed) through U12
TXU0304-Q1 and U13 TXU0204-Q1 on 3V3_IMU2; INT3 (gyro data ready) to PD15. INT1, INT2, INT4 open (data sheet: do not
connect if unused).

### 9.4 Magnetometer: selection and design

| Candidate | Noise / resolution | Mass | Interface and timing | Radiation | Cost | Notes |
|-----------|--------------------|------|----------------------|-----------|------|-------|
| GomSpace M315 | 15 nT (BVB §2.7) | 8 g + harness | module, I2C | heritage | not quoted | GomSpace has shed small-sat lines (BVB §2.1); availability to confirm |
| AAC Clyde MM200 | not read | 10–12 g | module | 30 krad | not quoted | — |
| Spacemanic Lodestone | not published | < 10 g | module, magnetometer + gyro | 3+ missions | €800 | duplicates the gyro |
| **PNI RM3100 on board A2** | **13 nT resolution, 15 nT noise at 200 cycle counts** (PNI table 3-1) | ≈ 1 g of parts | I2C at 0x20, single-shot measurement on command with DRDY | no published data | ≈ $40 | ± 800 µT range, linear ± 200 µT; 260 µA at 8 Hz three-axis |
**Selected: the RM3100, on a small board A2 at the +Z end frame.** Reasons for the part: (1) its single-shot mode
starts a conversion only when the MCU asks, in the 950–1000 ms coils-off window, with DRDY marking the end, which is
exactly the timing ICD-004 §12.2 requires ("single-shot conversions triggered in the window"); (2) it is 3× better than
the Monte Carlo's 50 nT noise assumption; (3) about 3 g with board and harness; (4) cost. Against it: no radiation data
(covered by its own latch-up switch U7 and the RAD-V3 lot test, like every other COTS part without data).

**Location (Version B re-assessment).** The question is how far the sensor is from the three CR0002 rods, whose
remanence after each burst (hysteresis) the 50 ms settle cannot remove (RID GNC-04, AV-V4), and from the currents
that change while it measures. The rod field is computed with each rod as two magnetic charges at its ends (47 mm
rods at the CAD centres of MECHANICAL_DESIGN §4.1: X (0, +36, −10) along X, Y (+42, 0, −33) along Y, Z (−41.5, 0,
−20.5) along Z), per 1 mA·m² of remanence (`obc_calcs.json` mag_rod_*):

| Position (x, y, z mm) | X rod | Y rod | Z rod | Sum (nT per mA·m²) | Nearest rod end | Per-rod remanence for 300 nT on a polarity reversal | Other sources |
|-----------------------|------:|------:|------:|------:|-----------------|------|------|
| OBC (+X, −Y) corner, (+33, −38, +7) (Version A) | 196 | 846 | 152 | **1194** | Y rod, 43.5 mm | 0.13 mA·m² | Jetson VDD_IN ≈ 12 mm if at the SO-DIMM's +X end (≈ 10 µT, carrier §18.2) |
| OBC −Y edge, best spot (+2, −42, +7) | 174 | 386 | 366 | 926 | — | 0.16 | under the SO-DIMM; centre support (0, −21) |
| OBC moved toward +Y, x +33: y 0 / +20 / +30 / +38 | 981 / 2977 / 4712 / 5131 | 948 / 1002 / 951 / 846 | 209 / 190 / 170 / 152 | **2138 / 4169 / 5833 / 6129** | X rod end 41 / 25 / 20 / 20 mm | 0.07 … 0.02 | on J1 (+X edge, \|y\| < 24) and J3 |
| **+Z end-frame slot of the CAD (+24, +27, +44)** | 480 | 178 | 173 | **830** | X rod, 54.7 mm | **0.18** | Jetson VDD_IN 63 mm; carrier M.2 card about 20 mm below; RF board 4 mm below |
| +Z mirror slot (+24, −27, +44) | 156 | 178 | 173 | 507 | Y rod, 79 mm | 0.30 | Jetson VDD_IN 25 mm if at the +X end; needs a mass-property re-trim |

**Moving the RM3100 toward +Y on the OBC is not feasible.** The X rod lies along y = +36 only 17 mm below the OBC
(z −10), so every step toward +Y brings the sensor toward its +X end: the coupling rises 1.8× at y 0 and 5× at
y +38, while the distance to the Y rod's ends hardly changes (the Y rod is centred at y = 0 and its two ends are at
± 23.5 mm, so moving from −Y to +Y trades one end for the other). The +X edge at |y| < 24 is also where J1 has to be
(the W3 ribbon's only path, §21), and the +Y half is J3 and the gyro window. On the OBC the best spot is the −Y edge
near x = 0 (926), under the SO-DIMM connector, which is no better for the Jetson current.

**Recommended hardware: board A2 at the CAD's +Z magnetometer slot, (+24, +27) mm, z +40.4 … +48.4** (where
MECHANICAL_DESIGN §4.1 already reserves a magnetometer). A2 is a 24 × 24 mm, two-layer, 1.6 mm board with the RM3100
(U1 MagI2C, L1/L2 Sen-XY-f, L3 Sen-Z-f, six 121 Ω, REXT 33 kΩ, 10 µF + 100 nF), its harness connector J1 and one ESD
array; it carries no switch and no isolation, which stay on the OBC (U7, U14, U33), so a fault on A2 or W6E trips U7
and nothing else. It is mounted to the +Z end frame by non-magnetic fasteners (titanium or brass, not A286 near the
coils) with the coils' axes on the body axes. What this gives:

- **Rods:** 830 nT per mA·m², 1.44× less than the OBC corner, with no change to the mass properties. The mirror slot
  (+24, −27) is 1.6× better again (507), but it is 25 mm above the SO-DIMM's +X end and its 5 g moves the CG toward −Y
  (ΔIyz ≈ 1.2 × 10⁻⁵ kg·m², about 4° of principal-axis tilt without a re-trim); it is the option to take only if the
  carrier confirms VDD_IN at the −X end and AV-V4 shows the rods dominate.
- **Rod-remanence distance:** 54.7 mm to the nearest rod end (the X rod's +X end), against 43.5 mm (Y rod) in the OBC
  corner and 20 mm at the OBC's +Y corner; 79–82 mm to the Y and Z rods.
- **Jetson input current:** 63 mm from the SO-DIMM's +X end (if VDD_IN is there) instead of about 12 mm: the carrier's
  10 µT at 12 mm falls to 0.07 µT for a compact loop (1/r³) and at most 1.9 µT for an unpaired line (1/r). The chair's
  fallback covers what remains (below).
- **Carrier keep-out:** the 20–30 mm keep-out around (+33, −38) on the carrier is **withdrawn** (interface item E11);
  the carrier may keep the placement it has. The carrier's nearest parts are about 20 mm below A2 (the M.2 card and its
  standoff MP1, behind the spreader and the RF board); a ferromagnetic part there is a fixed bias that the hard-iron
  calibration removes, and the SSD's current flows only with the Jetson on.
- **Magnetic keep-out around A2 (20 mm, for the mechanical and RF designers):** no ferromagnetic material within 20 mm
  of the sensor coils: the RF board's top-side parts under A2 (no ferrite-cored inductors, no steel shield lid or
  clips near (+24, +27)), the GNSS patch adapter's fasteners, the trim masses at y ±45 (tungsten or brass, not steel),
  the end-frame fasteners; and no unpaired current above 10 mA within 30 mm (the RF board's supply and PA return
  paired beneath). The wing hinge's secondary spring at (+54.1, +38.75) is 32 mm away; if it is a ferromagnetic steel
  (17-7PH) it is a fixed bias, and it moves once, at deployment.

**Residual for AV-V4 (what the coupling test has to show).** At the recommended slot each 1 mA·m² of rod remanence
gives up to 830 nT at the sensor (worst alignment of the three rods). A polarity reversal of a rod changes its remanent
moment by 2 m_r, so the Monte Carlo's 300 nT bias allowance holds without calibration only if every rod's remanence
stays at or below **0.18 mA·m²** (0.09 % of its 0.2 A·m² rating; CubeSpace does not publish it). If AV-V4 finds more,
the flight software keeps a bias per last-polarity state of the three rods (2³ = 8 states, measured in AV-V4 on the
EM rods and refined in orbit), and the residual becomes the non-repeatable part. **AV-V4 pass criterion
(V-OBC-11):** with A2 and the EM rods in their flight geometry (a fixture to the CAD positions), the burst-to-burst
change of the sensor reading 50 ms after the coils go off is ≤ 300 nT after the per-state calibration, over at least
100 bursts of random polarity and amplitude; plus the Jetson-on and PTT-keyed steps measured once with the Jetson EM
and the RF EM running, for the record (they are gated out).

**Flight-software gating (for the chair; the chair's fallback adopted).** The magnetometer is not used for actuation
while **JET_PG is high** (Jetson bursts ≤ 10 min; the spin holds the attitude) and, recommended here in addition, while
**PTT_EXCITER is high** (the RF board directly below A2 is keyed; PC7 already captures it): samples taken then are
logged but not used. With these two gates, the coils-off window and the per-state calibration, the only field changes
left during a measurement are the rods' non-repeatable remanence, which is what AV-V4 measures.

**Circuit** (PNI Doc 1017252 R07, figure 4-2, on A2): MagI2C U1 (13156) with two Sen-XY-f (13104) and one Sen-Z-f
(13101); 121 Ω bias resistors at both ends of each coil (six); REXT 33 kΩ (180 kHz clock); I2CEN high, SA0 = SA1 = 0
(0x20); AVDD and DVDD together on 3V3_SNS (10 µF + 100 nF). On the OBC: 3V3_SNS from U7, I2C4 through U14 TCA9517-Q1
(enabled only while 3V3_SNS is up, 4.7 kΩ pull-ups on the sensor side), DRDY back through U33 SN74LVC1T45-Q1 to PD11,
J10 (Pico-Clasp 6: 3V3_SNS, GND, SDA, SCL, DRDY, GND) with an ESD array and a 100 kΩ pull-down on DRDY. Harness W6E,
0.10 m of 28 AWG up the (+X, +Y) bundle (about 0.06 m routed plus a service loop; excess stowed below z +20, away from
A2): sensor-side rise time 139 ns (4.7 kΩ, about 35 pF), inside 300 ns for
400 kHz; 100 kHz recommended. Layout rules for A2: §21.


### 9.5 Coarse sun sensors

The six panel photodiodes (bought with the panels; BVB §2.3) come in on J8 (Pico-Clasp 12, W6D: six unshielded twisted
28 AWG pairs, Version C; SQSO-HAR-001 2.0). Each cathode drives a zero-bias transimpedance amplifier (TLV9002-Q1, three duals, on 3V3_SNS;
anode to ground): **Vout = Isc × 619 Ω** (0.1 % resistor), Cf 10 nF C0G. The panel vendor must keep the photodiode's
short-circuit current at AM0 normal incidence at or below 4.0 mA (2.48 V out); the amplifier clips at 5.25 mA (3.25 V).
Each output reaches the ADS7138-Q1 (U16, 12 bits, I2C 0x10, AVDD = 3V3_SNS as reference) through 1 kΩ / 100 nF.

| Item | Value |
|------|-------|
| Resolution | 1.30 µA per LSB; 1.46° angle step at normal incidence (cosine), far below the 5° per sensor of the Monte Carlo |
| Feedback resistor power | 9.9 mW at 4 mA (10 % of 0.1 W) |
| Other ADS7138 inputs | AIN6: TMP235-Q1 U20 beside the gyro and IMU; AIN7: VBAT_MTQ / 6 (1 MΩ / 200 kΩ) |
| If the panel diode is smaller | Rf is the only change (up to 10 kΩ for 0.25 mA); fixed when the panel vendor names the diode |

## 10. Magnetorquer Drive (RID AV-03)

### 10.1 Rods and supply

CubeSpace CR0002: gain 2.3 A·m²/A, > 0.2 A·m², 5 V maximum (satsearch listing; the CubeSpace datasheet and ICD were not
reachable: §25). Resistance ≈ 51 Ω from the CubeSpace brochure table (column order uncertain); the design covers
30–80 Ω.

| Quantity | Value |
|----------|-------|
| Current for 0.20 A·m² | **87 mA**; 4.43 V across a 51 Ω rod; 0.39 W per rod, 1.16 W with all three saturated (detumble) |
| From 3.3 V | at most 3.3 V / 52 Ω = 63 mA = 0.15 A·m²: the 3V3_ADCS rail cannot reach the flight software's 0.2 A·m² saturation |
| From VBAT | duty 0.41 at 11.2 V, 0.32 at 14.4 V, 0.28 at 16.4 V; 0.63 for an 80 Ω rod at 11.2 V (headroom) |
| Supply current, three saturated, 14.4 V | 0.083 A (97 % bridge efficiency) |

So the bridges run from VBAT: VBAT_MTQ from a new EPS switch SW11 on ADCS_EN (interface change E3), entering on
J1-36 and J1-39 (0.115 A per 28 AWG wire at the worst fault, 46 % of its derated 0.25 A), with D3 SMBJ20A (20 V
standoff, 32.4 V clamp, below the DRV8876's 40 V absolute maximum) and 5 × 4.7 µF 50 V.

### 10.2 H-bridges

Three DRV8876-Q1 (U22 X, U23 Y, U24 Z; HTSSOP-16, AEC-Q100, VM 4.5–37 V, 3.5 A) in **PH/EN mode** (PMODE low): EN =
TIM1 CH1/CH2/CH3 PWM (PE9, PE11, PE13), PH = direction GPIO (PE8, PE10, PE12). PWM low brakes through the low sides
(slow decay), so the average coil current is linear in the duty (I ≈ D · VM / R).

| Item | Design | Value |
|------|--------|-------|
| PWM | 40 kHz recommended (≤ 100 kHz) | slow-decay ripple at 16.4 V: 16 / 4.0 / 1.6 mA p-p for 5 / 20 / 50 mH (rod inductance not published) |
| Hardware current limit | IMODE = GND (fixed off-time chopping, 25 µs), VREF 1.150 V (18.7 k / 10.0 k, 0.1 %, from 3V3_OBC), R_IPROPI 10.0 kΩ: ITRIP = VREF / (R_IPROPI · 1000 µA/A) | **115 mA nominal, 104–126 mA** (3V3 ± 3 %, ± 7.5 mA mirror error below 0.15 A): 1.19× above the 87 mA saturation current, so it never cuts a valid command, and it holds a rod to 0.81 W if the software commands 100 % (instead of 0.53 A and 8.5 W into a 30 Ω rod) |
| Current telemetry | IPROPI → 1 nF → 1 kΩ → MCU ADC (PA4, PA5, PA6), 10 mV/mA, 3.3 V = 330 mA | lets the flight software check the coils are off at the 950 ms sample (AV-V2) |
| Faults | nFAULT open drain, wired-OR, 10 kΩ pull-up → PE14 (EXTI14) | over-current (automatic retry), thermal, under-voltage |
| Sleep | nSLEEP of all three = MTQ_SLEEP_N (§10.3); internal pull-downs on EN, PH and nSLEEP | sleep current ≤ 5 µA each; awake 3 mA typical each (0.12 W for three at 90 % of the time: the firmware should hold MTQ_ARM low in periods with a zero command) |
| Dissipation | ITRIP(max)² × 2 RDS(on) at 150 °C + VM × 7 mA | 0.14 W per bridge, Tj 65 °C at a 60 °C board |
| Coil leads | OUT1/OUT2 to J7 directly | the bridge's body diodes clamp to VM and GND; D3 absorbs the energy; no low-voltage ESD array on these lines |

### 10.3 Burst timing and the hardware off-timer

```
 period                    0 ms                           900 ms   950 ms          1000 ms
 MTQ_ARM (TIM15 CH1, PF9)  ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|________________________|‾‾‾ (one-pulse mode)
 U25 off-timer (Q10)       ____________________________________________|‾‾ (only if ARM is still high at 0.92-0.98 s)
 nSLEEP = ARM · ¬Q10       ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|________________________|‾‾‾
 coil current              /‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾\_ (≤ 1.9 ms)               magnetometer sample 950-1000 ms
```

- **Normal end of the burst (firmware, hardware timer).** The flight software sets the PH lines and the TIM1 duties,
  then starts TIM15 in one-pulse mode for 900 ms (80 MHz / 8000 → 10 kHz, ARR 9000). At 900 ms TIM15 drops MTQ_ARM by
  itself, whatever the software is doing; nSLEEP falls; the bridges go high-impedance (tSLEEP ≤ 1 ms) and the coil
  current decays through the body diodes against VM (L · I / VM ≤ 0.9 ms for L ≤ 80 mH): **zero within 1.9 ms of the
  900 ms point**. ICD-004 AV-V3 asks for 1 ms: change its criterion to 2 ms (for the chair; the 50 ms settle is not
  affected).
- **Hardware off-timer (a hung or mis-programmed OBC).** U25 CD74HC4060 is held in reset while MTQ_ARM is low
  (MR = ¬MTQ_ARM through U26). When MTQ_ARM rises, its RC oscillator (Rt 80.6 kΩ, Ct 10 nF C0G, Rs 162 kΩ:
  f = 1 / (2.3 Rt Ct) = 539 Hz) counts; after 512 periods Q10 rises at **0.949 s (0.921–0.978 s after trim, ± 3 %)**,
  forces nSLEEP low through U26/U27 and latches itself (D4 holds the oscillator input high) until MTQ_ARM falls. So
  the coils cannot be on for more than 0.98 s per rising edge of MTQ_ARM, whether the MCU hangs, its clock stops with
  TIM15's output high, or a register upset leaves the output set. The window sits 21 ms above the 900 ms burst (never
  truncates a valid burst) and 22 ms below the 1000 ms period (re-armed every period). Q10 is read back on PE15
  (MTQ_EXPIRED): high during a period means the firmware's timer failed.
- **Reset.** MTQ_ARM has a 100 kΩ pull-down: an OBC in reset or unpowered cannot enable a bridge; VBAT_MTQ is also off
  with ADCS_EN low (EPS 100 kΩ pull-down).

## 11. Jetson Link and Power Switches (RIDs AV-09, AV-06)

### 11.1 Back-power isolation (AV-09)

USART1 TX (PA9) reaches the carrier through U28 SN74LVC1T45-Q1 (A → B), RX (PA10) through U29 (B → A). Both take
VCCA from 3V3_OBC and **VCCB from the carrier's own 3.3 V I/O rail, brought back on J3-4 (JET_3V3)**. With the carrier
unpowered VCCB is 0 V and both ports are high-impedance (VCC isolation; Ioff ≤ µA): the idle-high TX line cannot reach
the module's clamp diodes, whatever the firmware does with PA9. PA10 has a 100 kΩ pull-up, so the receiver idles
without false start bits while the carrier is off. 33 Ω series on both carrier-side lines; TPD4E05U06-Q1 at J3. The
ICD-002 §9 test (TX idle, carrier off, carrier 3.3 V below 0.3 V) becomes V-OBC-9. The carrier puts its 3.3 V I/O
rail on its J2-4 (≤ 1 mA drawn; interface change E9, done in SQSO-JET-BRD-001 B).

### 11.2 Two independent switches with readback (AV-06)

| Element | Design | Independence |
|---------|--------|--------------|
| PAYLOAD_EN → EPS SW1 (TPS1H100B-Q1, shed-gated) | **PB2** (port B), 100 Ω, EPS 100 kΩ pull-down | different port from JETSON_PWR: one port-register upset or bad write cannot set both |
| JETSON_PWR → EPS U19 eFuse (TPS16630, 1.2 A latch-off) | **PA8** (port A; PA2 in Version A, given to the spreader NTC, §11.3) | different part, different line (BVB §7.3) |
| Readback 1: SW1 output | VSW1_RB, 1.00 MΩ / 200 kΩ on the EPS → J1-37 → PA1 (ADC1_IN6): 1.87–2.80 V on, < 0.1 V off | measures the switch output itself, not the command |
| Readback 2: eFuse output | VJET_RB, same divider after U19 → J1-19 → PA7 (ADC1_IN12) | as above |
| Readback 3: carrier alive | JET_PG: JET_3V3 through 10 kΩ → PG6 (EXTI6), 100 kΩ pull-down | the carrier's converter is running |
| Reset behaviour | both lines high-impedance at reset, EPS pull-downs → both switches off | **the firmware's PB4 was NJTRST with an internal pull-up at reset**: SW1 would have switched on at every OBC reset; PB2 has no reset pull |

Flight-software side (for the chair; PAYLOAD_DESIGN §11.1's recommendation): drive the two lines from two state bits,
switch off JETSON_PWR first and PAYLOAD_EN 10 ms later, compare each readback with its command (threshold 1.0 V on the
ADC readbacks) and log a disagreement as a fault; test each switch alone opening the rail (RAD-V5).

### 11.3 Jetson heat-spreader temperature (carrier interface E10)

The carrier passes the spreader thermistor RT1 (TE GA10K3MCD1, 10 kΩ at 25 °C NTC, bonded to the heat spreader) through,
unpowered, on W5B pins 5 and 6 (carrier J2-5/-6, OBC J3-5/-6). The OBC biases and reads it, so the 70 °C guard and the
−15 °C power-on gate work **with the carrier off**, from the always-on MCU domain.

| Item | Design |
|------|--------|
| Circuit | VDDA → **10.0 kΩ 0.1 % (25 ppm/K)** → J3-5 (PT_NTC_J) → RT1 → J3-6 → GND at J3; PT_NTC_J → 1 kΩ → **PA2 (ADC1_IN7)** with 100 nF at the pin; J3-5 in the TPD4E05U06-Q1 array at J3 |
| Pin | PA2 was JETSON_PWR in Version A; JETSON_PWR moves to PA8 (port A, still a different port from PAYLOAD_EN PB2). ADC1 has 16 external inputs, all used; PA2 is the only one that can be freed without moving an analog front end |
| Ratiometric | VDDA is the ADC reference (VREF+ on VDDA), so the code N = 4095 · R / (10.0 kΩ + R) does not depend on the 3.3 V tolerance: **R = 10.0 kΩ · N / (4095 − N)** (or V / (VDDA − V)) |
| Curve | β = 3976 K (25/85 °C) assumed for the TE "10K3" curve; the flight software should use TE's resistance table (Steinhart–Hart) once read (§25) |
| Values (`obc_calcs.json` ntc_*) | −40 °C: 412 kΩ, 3.22 V; **−15 °C: 79.0 kΩ, 2.93 V**, −19.6 mV/K, 0.041 K per LSB, ± 0.24 K; 25 °C: 10.0 kΩ, 1.65 V; **70 °C: 1.74 kΩ, 0.49 V**, −14.1 mV/K, 0.057 K per LSB, ± 0.38 K; 95 °C: 0.79 kΩ, 0.24 V (errors: bias 0.1 %, NTC 1 %, ADC 4 LSB, RSS) |
| Sampling | source 10 kΩ ∥ R plus 1 kΩ, with the 100 nF as the reservoir: the 5 pF sample capacitor takes 0.005 %; 247.5-cycle sampling as the other channels |
| Faults | open (connector off, broken lead): N ≥ 4090, reads below −50 °C; short: N ≤ 5, reads above 150 °C. Recommended flight-software rule: valid only for 0.10–3.25 V; outside it, a sensor fault that **inhibits Jetson power-on** (the gate fails safe) and, if the Jetson is on, falls back to the Jetson's own temperature report for the guard |
| Power | bias 0.28 mA at 70 °C (0.31 mA at 95 °C) from VDDA, always on (included in the 25.4 mA); 0.94 mW worst in the 10.0 kΩ (0.1 W part: 1 %); 0.14 mW self-heating in RT1 at 70 °C (bonded, < 0.1 K) |
| Isolation | none needed: the carrier passes RT1 through with no connection to its circuits (SQSO-JET-BRD-001 B, E10), so there is no carrier node to back-power |

## 12. GNSS Link

The receiver's 3V3_GNSS (EPS SW7 on GNSS_EN PE5) is routed through the OBC: in on J4 (W9B), out on J5-1 (W9A), so the
three SN74LVC1T45-Q1 translators U30 (TX, PG7 → receiver RXD), U31 (receiver TXD → PG8) and U32 (1PPS → PC6,
TIM3_CH1) take their receiver side from the receiver's rail: a switched-off or tripped receiver is never back-powered
and its latch-up current is not fed through the UART (closes the back-power half of finding F13 and AV-V8's "current
zero with GNSS_EN low"). GNSS_PG (3V3_GNSS through 10 kΩ → PG15) shows an SW7 trip. PG8 has a 100 kΩ pull-up; PC6 a
100 kΩ pull-down. 10 µF + 100 nF at J5.

## 13. RF Board Interface (RIDs RF-21, AV-05)

J2 (Samtec TFM-108 right-angle, 16 positions, W4A version 4 with an SFSDT 30 AWG assembly; version A had FTSH-107) is the OBC side of the RF board's J1
(SQSO-RF-BRD-001 §4.1; pins 1–14 as there, pins 15–16 new). Outputs (PD5 USART2_TX,
PE3 TX_PWR_SEL, PE4 XPDR_EN) leave through 100 Ω; the RF board adds 1 kΩ and 100 kΩ pull-downs. Inputs from the RF
board, which stays powered when the OBC is power-cycled, carry 10 kΩ series resistors to limit the current into an
unpowered OBC: PD6 USART2_RX (100 kΩ pull-up), and the three readbacks with the **100 kΩ pull-downs the RF board asks
for**: **TONE_OK → PC9** (EXTI9), **XPDR_KEY → PG12** (EXTI12), **PTT_EXCITER → PC7 (TIM8_CH2 input capture**, so the
firmware measures keyed time instead of estimating it from the frame length, ICD-003 §5.3), and, new in Version B for
the flight software, **RX_CD → PG0** (EXTI0, high = receiver carrier detect; J2-15) and **RX_FAULT → PF13** (EXTI13,
high = receiver fault; J2-16): both through 10 kΩ with 100 kΩ pull-downs, so RX_FAULT reads healthy with the line
open or the radio unpowered, as the flight software asked. The RF board drives them like TONE_OK, with its own 100 kΩ pull-downs (SQSO-RF-BRD-001 B §4.1, item E13); the
firmware ignores RX_FAULT for 1 s after RADIO_EN switches on. The monitors TX_TEMP (PA0),
TX_FWD (PC0), TX_REV (PC1) take 10 kΩ series and 10 nF at the pin (the RF board's 100 nF is the sampling reservoir).
Three TPD4E05U06-Q1 at J2 (the two new lines use their spare channels).

## 14. EPS Interface, Deployment and Passivation Lines

J1 (Samtec TFM-120 right-angle, W3 version 3: SFSDT 30 AWG to the EPS's vertical TFM-120 J3) mates EPS J3; pin list in SQSO-ICX-001 §3.1. Every OBC output to the EPS goes through
100 Ω and is high-impedance at reset, held low by the EPS's 100 kΩ. Burn lines: four, one per resistor (BURN_W1A PB10,
BURN_W1B PE6, BURN_W2A PB9, BURN_W2B **PE7**), each behind the EPS's 12 s one-shot and 0.8 A limit; the master arm
DEPLOY_EN moves to **PF3** (a different port). Passivation: EOL_ARM **PF4** (rising edge sets the EPS latch) and
EOL_FIRE **PG1** (held ≥ 60 s), on different ports and different ribbon rows; the recharge-lockout clear LOCK_CLR
**PF5** (≥ 100 ms, ANDed on the EPS with its controller's UNLOCK). Inputs: EPS_ALERT_N PB7 (10 kΩ series: the EPS's
4.7 kΩ pull-up would otherwise put 0.7 mA into an unpowered OBC), DEPLOY_SENSE PE2 and SOLAR_SENSE PB12 (10 kΩ).
Analog lines through 1 kΩ with 10 nF at the pin. I2C2: the OBC's 2.2 kΩ pull-ups are the only ones on the bus
(EPS waiver W5): rise time 224 ns at 120 pF (≤ 300 ns for 400 kHz), 1.5 mA sink.

## 15. GSE Umbilical

J9 (Samtec TFM-105-01-S-D-RA, right-angle, 2 × 5, 1.27 mm, gold; pin list SQSO-ICX-001 §3.6) on the OBC's top side at
the −X edge, its cable toward −X with the mated plug's back at most 2.0 mm inside the edge (x ≥ −43.1), carries W8A
(10 × 30 AWG Samtec SFSDT, 0.145 m) down the (−X, −Y) bundle to the access-port board A3 at the −Y window (§15.1). On
this board: SWD (22 Ω), NRST (100 Ω), USART3 console and key loading (TX 100 Ω; RX 10 kΩ, the GSE adapter may be powered
while the OBC is not), UMBILICAL_SENSE (1 kΩ, 100 kΩ pull-down; the plug ties it to VTREF, so key provisioning is
possible only with the plug in, SQSO-ICD-013 §9.2) and WDT_DIS (§6). VTREF is 3V3_OBC through 100 Ω (sense only). Two
TPD4E05U06-Q1 at J9.

### 15.1 Access-port board A3 (new, Version C)

A3 puts one flight connector at the −Y access window for the OBC's debug lines (W8A) and the EPS's charging path (W11,
SQSO-EPS-BRD-001 §6.5); it is the "harness part" A3 of SQSO-HAR-001 2.0 §3.2, built to this section. Schematic in
`design.py` (`a3_board()`): 8 placements, 6 BOM lines, 13 nets, ERC clean.

| Item | Design |
|------|--------|
| Envelope and board | Inside the CAD envelope x −35 … −5, y 7 mm inboard of the −Y shear panel, z −46.8 … −38 (8.8 mm; MECHANICAL request M-3): board **30 × 8.8 × 1.6 mm**, FR-4, 2 layers, ENIG (SQSO-HAR-001 §3.2 gives 30 × 12 mm, which does not fit the 8.8 mm height: the envelope governs), parallel to the panel |
| Connector J1 | **Micro-D 21 socket receptacle**, MIL-DTL-83513 class, straight PCB tails, gold contacts (M83513/11 shell C class; dash number to confirm), its mating face through the 18 × 6 mm window; 3 A per contact |
| Pin map (SQSO-HAR-001 §3.2) | debug: 1 VTREF, 2 SWDIO, 3 GND, 4 SWCLK, 5 WDT_DIS, 6 NRST, 7 USART3_TX, 12 USART3_RX, 13 UMBILICAL_SENSE, 14 GND; charge: 9 UMB_VMON, 10–11 PACK_N, 19 UMB_SHUT, 20–21 UMB_CHG; **8 and 15–18 not connected**, so the charge cluster (9–11, 19–21) is one contact pitch or more from every debug pin |
| Wires | W8A's ten 30 AWG wires and W11's six 24 AWG wires in plated holes at the two ends of the board (debug at the pin-1 end, charge at the other), soldered and staked within 10 mm; one re-termination service loop |
| Protection, debug group | two TPD4E05U06-Q1 (D1, D2) on SWDIO, SWCLK, NRST, WDT_DIS, USART3_TX/RX, UMBILICAL_SENSE and VTREF, to GND (the OBC's ground through W8A pins 3 and 10); the series resistors and the second ESD stage are at OBC J9 |
| Protection, charge group | D3 SMF24A UMB_CHG → PACK_N (24 V VRWM against the 16.8 V charge voltage, 70 %; 38.9 V clamp below the EPS's 40 V VBAT parts; EPS F5 1 A and D16 behind it, so it cannot drain the pack); D4 SMF24A UMB_VMON → PACK_N (the line has the EPS's 100 kΩ from BAT_F; leakage ≤ 1 µA worst at VRWM, far less at 16.8 V: for the EPS storage-drain budget, item C18); D5 TPD1E05U06-Q1 UMB_SHUT → PACK_N |
| Separation | **PACK_N is never joined to GND on A3** (the EPS's DS2 is the only GND–PACK_N path, SQSO-EPS-BRD-001 §6.4); no copper of the two groups within 0.5 mm of each other (≤ 16.8 V between them with DS2 open); the Micro-D shell and the mounting holes have no copper connection: the shell bonds to the frame through the panel only |
| Heights | parts on the board's inner face only, ≤ 1.4 mm (USON, X1SON, SOD-123FL), inside the 7 mm depth with the receptacle body and the board |
| Mounting | two jackposts (Ti, M2 or the receptacle's flange thread) through the −Y shear panel into the receptacle flange's threaded inserts, their outer female thread taking the GSE plug's jackscrews; **locking by a prevailing-torque (self-locking) thread** (self-locking jackposts or inserts), **no anaerobic thread-locker** (MATERIALS_LIST: Loctite 242 non-compliant, outgassing); a Scotch-Weld 2216 witness and staking bead at each jackpost (staking only, not structural). A3 is soldered to the receptacle's PCB tails and carries no fastener or nut (none would fit the 7 mm depth); its corner holes stay copper-free |
| Current | 0.5 A charge on two contacts per pole (0.25 A each, 8 % of 3 A); signals < 5 mA |
| Verification | V-OBC-18 (§22) |

## 16. ESD, Latch-up and Back-Power

| Threat | Protection |
|--------|------------|
| ESD at integration on every connector signal | TPD4E05U06-Q1 (IEC 61000-4-2 ± 12 kV contact, 0.5 pF, 5.5 V standoff) at J1 (7), J2 (3), J3 (with the NTC line), J5, J8 (2), J9 (2), J10, one on board A2 at its J1 and two on A3 at the window (the charge lines there: SMF24A ×2 and a TPD1E05U06 to PACK_N, §15.1); TPD1E05U06-Q1 on 3V3_RTC and 3V3_ADCS; SMF5.0A on 3V3_OBC; SMBJ20A on VBAT_MTQ |
| Latch-up, MCU, FRAM, supervisor, logic | EPS SW10 (0.130–0.176 A, 0.28 s off, auto-retry) on the whole 3V3_OBC domain (RID AV-08 closed by the EPS board) |
| Latch-up, sensors | U5–U7 (§9.1) |
| Latch-up, GNSS, Jetson | EPS SW7, U19; isolation keeps the OBC from feeding a latched part through its I/O |
| Back-power into a switched-off sensor, carrier or receiver | VCC-isolating translators (U9, U10, U12–U14, U28–U33) |
| Back-power into an unpowered OBC | series resistors on every line from the EPS and RF boards: **2.7 mA worst case in total** with every readback high (about 0.8 mA typical), far below latch-up holding currents; `obc_calcs.json` backpower_lines |

## 17. Power and Thermal

| Load | Rail | Normal | Worst |
|------|------|--------|-------|
| OBC (MCU, XO, FRAM, logic, NTC bias) | 3V3_OBC | 84 mW | 25.4 mA |
| Gyro, magnetometer, ADC, amplifiers | 3V3_ADCS | 76 mW (gyro 53 mW) | 49 mA with the backup gyro on |
| Magnetorquers, three saturated | VBAT_MTQ | 1.16 W coils + 0.12 W bridges (detumble only) | 0.23 A at 11.2 V with all bridges at their limit |
| RTC | 3V3_RTC | 0.15 µW | — |

Against POWER_BUDGET_ANALYSIS (OBC 0.165 W, ADCS sensors 0.12 W, magnetorquers ≤ 0.30 W orbit average): the OBC and
sensors fit (0.16 W together); the magnetorquer average depends on the commands (1.16 W peak during detumble; the
spin-hold law uses far less): the power analyst should take 1.3 W for the first orbits of detumble (for the chair).

Junction temperatures at a 60 °C board (the hot TVAC plateau cap; the flight hot case is 28–39 °C): DRV8876 65 °C
(0.14 W, 39.6 K/W), MCU 63 °C (66 mW, 45 K/W), TPS2553 60.2 °C at its limit, everything else within 1 K of the board:
all below the 110 °C derating limit. Cold: the board's flight minimum is −11.7 °C (THERMAL_ANALYSIS); every part is
rated −40 °C. Heat sources for the thermal model: three DRV8876 (0.14 W each while bursting, 0.01 W asleep), the MCU
(0.066 W), the gyro (0.053 W).

## 18. Stress Summary (derated)

| Part | Parameter | Applied | Rating | Use | Limit | Pass |
|------|-----------|---------|--------|-----|-------|------|
| DRV8876-Q1 | VM | 16.8 V | 37 V (operating) | 45 % | 80 % | yes |
| DRV8876-Q1 | output current | 0.126 A | 3.5 A | 4 % | 75 % | yes |
| SMBJ20A | standoff vs VBAT | 16.8 V | 20 V | 84 % (TVS: standoff above the maximum) | < 100 % | yes |
| 4.7 µF 50 V X7R (VM) | voltage | 16.8 V | 50 V | 34 % | 50 % | yes |
| 22 nF / 100 nF 50 V (charge pump) | voltage | 16.8 V / 5 V | 50 V | 34 % / 10 % | 50 % | yes |
| 3.3 V MLCCs | voltage | 3.4 V | 16–50 V | ≤ 21 % | 50 % | yes |
| TPS2553-Q1 | IN | 3.4 V | 6.5 V | 52 % | 80 % | yes |
| M-G370PDT0 | VCC | 3.195–3.40 V | 3.15–3.45 V | inside | inside | yes (45 mV margin) |
| BMI088, RM3100, ADS7138, TLV9002, TXU, LVC, TPS3850, FRAM, RTC, STM32 | supply | 3.4 V | 3.6–6.5 V | ≤ 94 % of the 3.6 V operating maximum | 90 % of operating for 3.6 V parts | **3.6 V parts at 94 %**: within their operating range but above a 90 % rule; accepted (the rail cannot exceed 3.40 V in operation; the supervisor resets the OBC above 3.53 V) |
| Rf 619 Ω (0.1 W) | power | 9.9 mW | 100 mW | 10 % | 50 % | yes |
| NTC bias 10.0 kΩ (0.1 W) | power | 0.94 mW at 95 °C | 100 mW | 1 % | 50 % | yes |
| R_IPROPI 10 kΩ | power | 0.16 mW | 100 mW | < 1 % | 50 % | yes |
| VBAT_MTQ divider 1 MΩ | voltage / power | 14 V / 0.2 mW | 75 V / 0.1 W | 19 % / 0.2 % | 80 % / 50 % | yes |
| TFM-120 contacts | current | 0.115 A | per Samtec (not read; > 1 A for the series) | < 12 % | 50 % | yes |
| A3 SMF24A (UMB_CHG, UMB_VMON) | standoff vs 16.8 V | 16.8 V | 24 V | 70 % (TVS: standoff above the maximum) | < 100 % | yes |
| A3 Micro-D contacts | current | 0.25 A | 3 A | 8 % | 50 % | yes |
| Pico-Clasp contacts (J3–J8, J10; A2 J1) | current | ≤ 0.34 A (trip) | 1 A | ≤ 34 % | 50 % | yes |
| W3 28 AWG (VBAT_MTQ) | current | 0.115 A | 0.25 A derated | 46 % | 100 % of derated | yes |
| W6B 26 AWG (3V3_ADCS) | current | 0.339 A at the SW4 trip | 0.5 A derated | 68 % | 100 % | yes |
| FM25V20A | SPI clock | 20 MHz | 40 MHz | 50 % | — | yes |

## 19. Parts

The BOM ([`obc_board_bom.csv`](obc_board_bom.csv)) gives every line's MPN, manufacturer, package, rating, derated
stress, finish and how its MPN was checked. Summary of the active parts:

| Function | Part (MPN) | Grade | Checked | Radiation |
|----------|-----------|-------|---------|-----------|
| MCU | STMicroelectronics STM32L4R5ZIT6 | industrial −40…+85 °C | ST site not reachable; EEE_PARTS_LIST §4.1 | no data (RADIATION_ANALYSIS §5.2) |
| FRAM | Infineon FM25V20A-GTR | industrial | not checked (Infineon site not read) | cells immune; periphery no data |
| Supervisor | TI TPS3850H33QDRCRQ1 | AEC-Q100 grade 1 | TI addendum: Active | no data |
| RTC | Micro Crystal RV-3028-C7-32.768kHz-1ppm-TA-QA | AEC-Q200, −40…+85 °C | application manual read (ordering code) | no data |
| XO | Epson SG-8018CA 8.0000M-TJHPA0 | −40…+105 °C | order code built from the EPS board's SG-8018CA code: not checked | no data |
| LSE crystal | Micro Crystal CM7V-T1A-32.768kHz-7pF-20PPM-TA-QA | AEC-Q200 | not checked | passive |
| Sensor switches | TI TPS2553QDBVRQ1 ×3 | AEC-Q100 | Active (EPS board) | no data |
| Gyro | Epson M-G370PDT0 | −40…+85 °C | data sheet (PDG0, Rev 1.2) read; PDT0 order code to confirm | TID pass 5/10/20 krad (BVB §5.2) |
| Backup gyro | Bosch BMI088 | −40…+85 °C | data sheet read | no data |
| Magnetometer (board A2) | PNI 13156 + 13104 ×2 + 13101 | −40…+85 °C | PNI user manual R07 read | no data |
| ADC | TI ADS7138QRTERQ1 | AEC-Q100 | Active | no data |
| Amplifiers | TI TLV9002QDGKRQ1 ×3 | AEC-Q100 grade 1 | Active | no data |
| Temperature | TI TMP235AQDBZRQ1 ×2 | AEC-Q100 | Active | no data |
| H-bridges | TI DRV8876QPWPRQ1 ×3 | AEC-Q100 | Active | no data |
| Off-timer | TI CD74HC4060PWR | −55…+125 °C, not AEC (the -Q1 SN74HC4060 is "last time buy") | listed (RF board) | no data |
| Gates | TI SN74LVC2G04DBVR (not AEC), SN74LVC1G08QDBVRQ1 | −40…+125 °C | Active | no data |
| Translators | TI TXU0304QPWRQ1 ×2, TXU0204QPWRQ1 ×2, SN74LVC1T45QDCKRQ1 ×6, TCA9517DGKRQ1 | AEC-Q100 | Active | no data |
| ESD | TI TPD4E05U06QDQARQ1 ×17 (+1 on A2, +2 on A3), TPD1E05U06QDPYRQ1 ×2 (+1 on A3); Littelfuse SMF24A ×2 on A3 | AEC-Q101 (TI) | Active (TI); SMF24A not checked | — |
| Passives | Murata GCM (AEC-Q200), Panasonic ERJ-3EKF / ERA-3AEB (AEC-Q200), Vishay CRCW-e3 | automotive | series codes checked, values not | — |
| Connectors | Samtec TFM-120/108/105-01-S-D-RA (right-angle, gold), CLM-110; Molex Pico-Clasp 501331 in the gold finish (J8 a 12-way part); Micro-D 21 on A3 | — | Samtec not checked; Pico-Clasp series Active (EPS board), the 14-position part by pattern | — |

No radiation data was found for any of these parts except the M-G370's TID. All are covered by the latch-up trips of
§16 and by RAD-V3 lot-sample testing (EEE_PARTS_LIST §2A.4). TID: the board sits mid-stack (RADIATION_ANALYSIS §4:
5.25 krad behind 2 mm Al in two years at 500 km); sector shielding per RID RAD-01 is the mechanical designer's.

## 20. Firmware Configuration the Board Assumes

Listed in full in SQSO-ICX-001 §6 for the flight-software team. Essentials: the pin moves (PAYLOAD_EN PB2, DEPLOY_EN
PF3, FRAM PG2–PG5, JETSON_PWR PA8); the spreader NTC on PA2 (ADC1_IN7, R = 10.0 kΩ · N / (4095 − N), §11.3);
the magnetometer not used for actuation while JET_PG or PTT_EXCITER is high (§9.4); VDDIO2 enabled (PWR_CR2.IOSV) before port G is used (already needed for LPUART1); the time driver
pinned to TIM2; WDI kicked every 10 s; MTQ bursts by TIM15 one-pulse, PH before ARM, ARM low in zero-command periods;
sensor rails switched without any pin juggling (the translators isolate); RTC driver on I2C1.

## 21. Layout Guidance (layout is not in scope; these are the constraints to start from)

Coordinates are body X, Y with the origin on the stack axis (board edges at x ± 45.1, y ± 47.9 mm); the OBC substrate
is at z +5.0 … +6.6 (MECHANICAL_DESIGN §4.1). The stack constraints are those of MECHANICAL_DESIGN §4.3–§4.4; where
Version A's guidance conflicted with them it has been changed (marked "Version B").

| Item | Requirement |
|------|-------------|
| Outline and mounting | PC/104 90.17 × 95.89 mm; four Ø 3.18 mm holes at the corner columns (± 40, ± 43) and the two centre supports at (0, ± 21); **Ø 7 mm free of parts on both sides** at all six, with Ø 6.4 mm (columns) and Ø 7 mm (supports) copper-free rings. **Version B: no frame bond on the OBC**; every hole is isolated from GND (the Al spacers are conductive) and the OBC's ground reaches the frame through the W3 ground wires and the EPS's single bond H1 (EPS-BRD §17), as on the carrier. If the mechanical designer prefers a second bond, it must be at the same column as EPS H1 |
| Stack-up | 6 layers, 1.6 mm (the FE and Steinberg margins assume 1.6 mm: OBC +0.14), FR-4 Tg ≥ 170 °C (polyimide preferred for TVAC): L1 parts and signals, L2 solid GND, L3 signals, L4 power (3V3_OBC plane, 3V3_ADCS and the three sensor-rail islands, VBAT_MTQ as a 1 mm trace), L5 solid GND, L6 low-profile parts and signals; 1 oz; IPC-6012 Class 3/A |
| Heights (Version B) | **Bottom side (L6) ≤ 1.0 mm**: only 0402/0603 resistors and capacitors (≤ 0.95 mm; no 0603 MLCC thicker than 0.9 mm), the TPD4E05U06 (USON, 0.55 mm) and TPD1E05U06 (X1SON) arrays and test pads; everything else on the top side (SOT-23, SOT-563, 1206/1210 capacitors, SMF/SMBJ diodes, the LQFP, HTSSOP, SOIC, LGA and every connector). **Top side ≤ 3.0 mm** except X1 and the connectors; **9.0 mm is free under the 2.0 mm carrier** (its 1 mm bottom-side envelope plus 1 mm clearance; the window frame and its 7.4 mm limit are withdrawn). The mated connectors J1–J10 (header, plug and cable bend) take the mechanical allocation of **8.0 mm** (MECHANICAL_DESIGN VI-MCAD-08), 1.0 mm inside the 9.0 mm, to be checked against the Samtec and Molex drawings (§25); X1 passes through the window and needs only its 1.5 mm lateral clearance there |
| X1 (gyro) | Socket centred at **(−20, +30.5)**, footprint x −32 … −8, y +18.5 … +42.5, square to the board axes (in-plane rotation within ± 0.3°, MECHANICAL_DESIGN §4.3.1); the module's two screw holes plated and isolated from GND; module top z +20.6 in the 2.0 mm carrier's window (1.5 mm lateral clearance, 7.2 mm below the spreader); BMI088 U11 and TMP235 U20 within 15 mm of X1 |
| J1, J2, J9 headers (Version C) | Samtec **TFM right-angle** (TFM-120, -108, -105) with SFSDT 30 AWG assemblies, cable toward the nearer wall gap, **mated plug's back at most 2.0 mm inside the board edge (x ≤ +43.1 for J1 and J2, x ≥ −43.1 for J9)** for the 4.0 mm bend into the 3 mm gap; ≤ 7.0 mm mated height (SQSO-HAR-001 2.0 §10.2) |
| J1 (W3 to the EPS, 40-way) | **Top side, on the +X edge, pin field along Y within \|y\| < 24 mm** (centred at y = 0 recommended), the same edge and y range as the EPS's J3 (for the EPS layout): the ribbon's only path is the +X wall gap (x +45.4 … +48.4) between the harness bundles at \|y\| = 24 … 34; the −X edge is ruled out by the Z rod |
| J2 (W4A to the RF board) | **Top side** (TFM-108 right-angle, pin field about 10 mm), cable up through a ±X wall gap at \|y\| = 34 … 44 mm (the spreader flanges fill \|y\| < 20 above the carrier, the bundles \|y\| = 24 … 34). **Placed at (+41.0, −33.5), pin field along Y (y −28.5 … −38.5)**, directly below the RF board's J1 at the same x, y (SQSO-RF-BRD-001 B §15 item 14), clear of the (+40, −43) column keep-out: the ribbon runs flat and vertical on the +X shear panel with no fold; above z −26 the (+X, −Y) bundle narrows to y −20.5 … −27.0 beside it (SQSO-ICX-001 §2, §2.1) |
| J3 (W5B to the carrier) and J10 (W6E to board A2) | Top side, +X edge, y +24 … +34 (beside the (+X, +Y) bundle; the carrier's J2 is at its +X edge, y ≈ +28) |
| J4, J5 (GNSS) | Top side, **+X edge at y +14 … +24** (between J1 and J3; Version B, moved from the −Y side): W9A and W9B in the (+X, +Y) bundle, W9A then along the deck to the receiver at (+5, −20) (SQSO-ICX-001 §2.1) |
| J6, J7, J8 and the DRV8876s | −X half. **J7 on the top side** (Version B) and outside the Z rod's footprint (x −46.8 … −36.2, \|y\| < 5.3; the rod ends at z +3.0, 1 mm below the OBC's bottom envelope): **J7 at y +12 … +22 on the −X edge** (moved from −Y with the bundle allocation, SQSO-ICX-001 §2.1), its cable leaving into the −X wall gap at y +7 … +19.5 (**keep that edge strip, x −45.1 … −40, y +7 … +19.5, free of other parts and cables above the board**: SQSO-HAR-001 2.0 O-3, between the Z rod and the 70 cm coax band), the three DRV8876s within 20 mm of it and outside X1's footprint; J6 (3V3_ADCS) and J8 (sun sensors) at the −X edge, y +24 … +34, beside the (−X, +Y) bundle |
| J9 (W8A to the access port) | Top side, −X edge at y −24 … −34, TFM-105 right-angle (W8A down the (−X, −Y) bundle to A3 at (−20, −50, −43), MECHANICAL_DESIGN §6.9) |
| Board A2 (magnetometer) | 24 × 24 mm, 2 layers, 1.6 mm, at (+24, +27) on the +Z end frame, z +40.4 … +48.4; Sen-XY-f coils along body X and Y, Sen-Z-f along body Z (PNI figure 4-4); J1 at the edge toward the +X wall gap; **no ferromagnetic part on A2** (Sn-finished parts only, no Ni-barrier connector shells where a non-magnetic one exists, no ferrite, no steel fasteners); 3V3_SNS and its GND return as a paired trace; and the 20 mm keep-out around it in the stack (§9.4) |
| Magnetic rules on the OBC (Version B) | the magnetometer is no longer on this board; the Version A rules (30 mm from the bridges, 20 mm non-ferromagnetic zone) lapse. Kept: VBAT_MTQ and each coil pair routed go-and-return together (L4/L5, and adjacent pins on J7), so the OBC's coil currents add no dipole of their own |
| Coil drive | VBAT_MTQ from J1-36/39 to the bridges ≥ 0.5 mm wide on L4 with its GND return on L5; each bridge's VM capacitors within 3 mm; exposed pads with ≥ 9 × 0.3 mm thermal vias to L2/L5 and ≥ 4 cm² copper each |
| Sun-sensor inputs | TIA inputs guarded, ≤ 15 mm from J8; W6D unshielded (no drains); J8 pins in pairs (K, A); ADS7138 analog ground at its pad; no digital traces under the TIAs |
| Spreader NTC | the 10.0 kΩ bias, the 1 kΩ and the 100 nF within 10 mm of PA2; PT_NTC_J from J3-5 as an analog trace away from the USART1 lines; J3-6 to the analog ground at the MCU's VSSA side |
| Clocks | Y2 guard ring to GND, no traces beneath on L1–L2; Y1 ≤ 10 mm from PH0 with the 33 Ω at the source; U2, Y1, Y2 within 10 mm of U1 |
| Back-power and isolation parts | each translator on the boundary of its domain: A side traces from the MCU, B side to the sensor or connector; the sensor-rail islands do not extend under the MCU |
| ESD | each array within 5 mm of its connector pin, ground via at the array |
| Creepage / clearance | ≤ 17 V everywhere: IPC-2221B B4 (coated) ≥ 0.13 mm; 0.3 mm around VBAT_MTQ |
| Test points | TP 3V3_OBC, GND, VDDA, NRST, WDI, SWO, MTQ_ARM, MTQ_SLEEP_N, MTQ_EXPIRED, VBAT_MTQ, 3V3_RTC, 3V3_ADCS, 3V3_GYRO, 3V3_SNS (in the netlist); add at layout: each IPROPI, each TIA output, 3V3_IMU2, JET_3V3, 3V3_GNSS, PT_NTC, I2C1/2/4; all test points on the top side, outside the carrier window's footprint |
| Coating | conformal coat except the connector mating faces, X1's contacts and the test points; on A2 the coat may cover the coils (non-magnetic) |

## 22. Verification Items (this board)

| ID | Item | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-OBC-1 | Supervisor: ramp 3V3_OBC through 3.0–3.6 V; stop WDI; kick WDI at 3 s and at 50 s intervals | reset below 3.04–3.09 V and above 3.50–3.56 V; reset ≥ 33 s and ≤ 45 s after the last kick; reset on a kick faster than 5.6 s | AV-02, §6 | OBC EM |
| V-OBC-2 | IWDG before the supervisor | with LSI running and the loop hung, the reset cause is IWDG, not pin | §6 | OBC EM |
| V-OBC-3 | Reset states of every OBC output (MCU held in reset, unpowered) | all enables, burns, EOL, RF_TX_EN, MTQ_ARM < 0.4 V at the far board | ICX V-ICX-2 | flatsat |
| V-OBC-4 | Mission clock across an OBC power cycle (SW10 forced off 10 s) and 100 resets | counter continuous; EVI count and time of each reset correct | AV-10 | OBC EM |
| V-OBC-5 | Sensor-rail trips: 60 mA, 120 mA and a short on each rail | limits 50–100 mA; off 0.10–0.2 s; retry; readback low; other rails and 3V3_OBC unaffected | AV-03, RADIATION_ANALYSIS §6 | OBC EM |
| V-OBC-6 | Isolation: each sensor rail off with its bus active | rail < 0.3 V; no current into it | §5 ICX | OBC EM |
| V-OBC-7 | Coil drive: 40 kHz, 0–100 % duty, 11.2 / 14.4 / 16.4 V, rods (or 51 Ω + measured L) | current within ± 5 % of D·V/R below 87 mA; limit 104–126 mA; IPROPI 10 mV/mA ± 8 mA | §10.2 | OBC EM with rods |
| V-OBC-8 | End of burst (= AV-V3): TIM15 end, MCU halted mid-burst, TIM15 forced on | current zero within 900 + 2 ms; with the MCU halted and ARM held high, zero at 0.92–0.98 s; MTQ_EXPIRED high | §10.3 | OBC EM |
| V-OBC-9 | Jetson isolation (= ICD-002 §9 test): carrier off, USART1 transmitting | carrier 3.3 V < 0.3 V | AV-09 | flatsat with the carrier |
| V-OBC-10 | Jetson switch readbacks: each switch alone on and off | VSW1_RB, VJET_RB, JET_PG follow each switch; reset leaves both off | AV-06 | flatsat |
| V-OBC-11 | Magnetometer on A2: noise in a zero-gauss chamber; coupling and remanence change after bursts with the EM rods in the flight geometry (= AV-V4); Jetson-on and PTT-keyed steps for the record | noise ≤ 20 nT at 200 cycles; burst-to-burst change 50 ms after the coils go off ≤ 300 nT after the per-polarity-state calibration, ≥ 100 random bursts; each rod's remanence measured (≤ 0.18 mA·m² needs no calibration) | §9.4, GNC-04 | EM, Helmholtz cage |
| V-OBC-12 | Power: 3V3_OBC at 80 MHz, all peripherals active; 3V3_ADCS with each sensor | ≤ 25 mA; ≤ 49 mA | §17 | OBC EM |
| V-OBC-13 | Thermal vacuum of the OBC EM at −20…+60 °C with the coils driven | all functions; DRV8876 case ≤ 70 °C | §17 | OBC EM |
| V-OBC-14 | ESD: IEC 61000-4-2 ± 8 kV contact at each connector pin (unpowered) | no damage; functional | §16 | OBC EM (one unit) |
| V-OBC-15 | Back-power into the unpowered OBC (= V-ICX-3) | ≤ 3.0 mA, 3V3_OBC < 0.5 V | §16 | flatsat |
| V-OBC-16 | Gyro random vibration and shock: OBC EM with a flight-lot M-G370 on X1 (screws in prevailing-torque self-locking threads, staked with Scotch-Weld 2216; no thread-locker), random at the FE-predicted module levels (≥ 22.2 / 25.0 / 21.7 Grms in axis) for the protoflight duration, shock to the launch SRS; header continuity monitored | before/after on a rate table: bias change ≤ 0.01 °/s, scale factor change ≤ 0.2 %, noise within 10 %, no DIAG_STAT flags or SPI errors; no continuity interruption > 1 µs | §9.2; with VI-STR-02 | OBC EM |
| V-OBC-17 | Spreader NTC: carrier off and on, RT1 (or a decade box) at −40, −15, 25, 70, 95 °C equivalents; open and short at J3 | reading within ± 0.5 K of the reference at −15 and 70 °C; open reads < −50 °C, short > 150 °C; reading unchanged with the carrier off | §11.3, E10 | flatsat with the carrier |
| V-OBC-18 | Access-port board A3: continuity of every Micro-D pin to its OBC J9 / EPS J13 pin through W8A and W11; isolation GND–PACK_N, debug group to charge group and both to the shell, at 50 V; ESD ± 8 kV contact at each Micro-D pin (unpowered); charge at 0.5 A through the GSE plug | continuity as SQSO-ICX-001 §3.6; isolation ≥ 100 MΩ (GND–PACK_N ≥ 100 kΩ with the EPS connected and DS2 open, V-EPS-4); no damage; charge drop ≤ 50 mV across A3 | §15.1 | flatsat |

## 23. RID Closure

| RID | Status | Evidence |
|-----|--------|----------|
| SE-01 (OBC share) | **Closed for the OBC board**: schematic with selected parts, netlist, BOM, ERC, calculations, layout rules | this document; `design.py`, `obc_board.net`, `obc_board_bom.csv`, `obc_board_erc.log`, `obc_calcs.py`. Layout and the EM build remain (V-OBC-*) |
| AV-01 | **Closed** (design); firmware change open | SQSO-ICX-001 B §4 (every pin, one function), §6 (28 changes for the flight software) |
| AV-02 | **Closed** (design); flight-software kick open | §6: TPS3850H33-Q1, window 5.6–33.3 s, kick every 10 s; V-OBC-1, -2 |
| AV-03 | **Pins and hardware closed**; drivers are flight software; the magnetometer's rod coupling is verified by AV-V4 | §9 (magnetometer on board A2, §9.4), §10; ICX §6 rows 10–17; V-OBC-5 to -8, -11, -16 |
| AV-05 / RF-21 | **Closed** (pins and circuits); firmware reading them open | §13: PC9, PG12, PC7 (capture) |
| AV-06 | **Closed in hardware**: separate ports, three readbacks; the flight software's two state bits and order open | §11.2; V-OBC-10 |
| AV-09 | **Closed in hardware** (no firmware workaround needed); the carrier supplies its rail on J2-4 (E9, done) | §11.1; V-OBC-9 |
| AV-10 | **Closed in hardware**; needs EPS J3-38 (E4) and the RTC driver | §8; V-OBC-4 |
| EPS-14 | **Closed**: GNSS_EN PE5, XPDR_EN PE4; W3 v2 corrected | ICX §3.1 |
| AV-04 (back-power part) | closed by U30–U32 | §12 |
| AV-08 (OBC rail) | closed by the EPS board (SW10); this board keeps its inrush below the trip | §4 |
| Finding F10 (0x48 clash) | closed: no TMP117 | §2.2 |
| Finding F13 | closed for the Jetson and the GNSS | §11.1, §12 |
| Carrier E10 (spreader NTC) | **taken**: PA2 (ADC1_IN7), 10.0 kΩ 0.1 % bias, readable with the carrier off | §11.3; V-OBC-17 |

## 24. For the Chair: Interface and Document Changes

Interface changes on the EPS board (E1–E8), the Jetson carrier (E9–E11) and the mechanical and harness designs (E12)
are in SQSO-ICX-001 §7; the flight-software
changes in SQSO-ICX-001 §6. In addition:

| # | Change | Owner | Recommended action or text |
|--:|--------|-------|----------------------------|
| C1 | Magnetometer from "buy a module" to the RM3100 on board A2 at the +Z end frame (Version B; Version A had it on the OBC) | chair (BUILD_VS_BUY §1, §2.7; MASS_BUDGET row 24: 10 g → 3 g with board and harness; ADCS_DESIGN §3.2.1) | "Magnetometer: PNI RM3100 on board A2 at the +Z end-frame slot (+24, +27) mm, harness W6E to OBC J10 (SQSO-OBC-BRD-001 §9.4); a bought module with I2C at 3.3 V fits the same slot and harness if AV-V4 fails" |
| C2 | Coil driver DRV8876-Q1 from VBAT (not DRV8833 on 3.3 V); new EPS switch SW11 | chair (ADCS_DESIGN §6.1), EPS designer (E3) | as §10 |
| C3 | Gyro stack clearance | mechanical designer | **closed** by MECHANICAL_DESIGN D-11 (gyro through the carrier window, X1 at (−20, +30.5)); adopted in §9.2 and §21 |
| C4 | ICD-004 AV-V3 pass criterion "900 ms + 1 ms" → "900 ms + 2 ms" | chair (ICD_OBC_ADCS §12.4) | bridge tSLEEP is up to 1 ms |
| C5 | ICD-004 §12.1 table: the pins of SQSO-ICX-001 §4 | chair | replace "open" with the allocated pins |
| C6 | ICD-002 §2 and §9: J3 Pico-Clasp 6 (GND, OBC TX, OBC RX, carrier 3.3 V, spreader NTC, NTC return), translators on the OBC; the 70 °C guard and −15 °C gate read from PA2 | chair | as SQSO-ICX-001 §3.3 |
| C7 | ICD-001 §2.1, §3.2: 40-pin J3 and the allocated pins | chair | as SQSO-ICX-001 §3.1 |
| C8 | RADIATION_ANALYSIS §5.3: gyro trip 50–100 mA (TPS2553 minimum); magnetometer on its own switch | radiation reviewer | update the two rows |
| C9 | Power budget: magnetorquers 1.3 W peak during detumble (three axes saturated) | power analyst | confirm the detumble energy in the SIL commissioning run |
| C10 | OBC rail fault that does not trip SW10 is undetected | chair / EPS | optional: EPS controller watches an OBC heartbeat register and power-cycles SW10 |
| C11 | hardware/diagrams README findings F2, F3, F8, F9, F10, F13, F14, F16 (OBC part) are answered by SQSO-OBC-BRD-001 / SQSO-ICX-001 | chair (SQSO-HW-DIAG-001) | mark them |
| C12 | Epson vibration and shock qualification data for the M-G370 family (the data sheet gives only 1000 g shock) | procurement / OBC | request from Epson; V-OBC-16 regardless |
| C13 | MECHANICAL_DESIGN §4.1, §4.4, §6.9 | mechanical designer | magnetometer slot (+24, +27) holds board A2 (24 × 24 × 8 mm, ≈ 2 g plus harness), non-magnetic fasteners; the §4.4 rows "RM3100 in the (+X, −Y) corner" and "magnetometer to the rods" replaced by §9.4 of this document (carrier keep-out withdrawn); the 20 mm non-ferromagnetic zone round A2 (RF shield lid and clips, patch-adapter fasteners, trim masses at y ± 45, end-frame fasteners); harness W6E (0.10 m) and W8A (0.15 m) in the bundles, excess stowed below z +20; the connector envelope exceptions of §21 (heights); OBC frame bond: none (§21) |
| C14 | Rod remanence | ADCS owner / procurement | ask CubeSpace for the CR0002 remanence; ≤ 0.18 mA·m² per rod needs no flight-software calibration at the +Z slot (§9.4) |
| C15 | Magnetometer gating | chair / flight software | the magnetometer is not used for actuation while JET_PG or PTT_EXCITER is high (logged only); per-polarity-state bias table if AV-V4 needs it (SQSO-ICX-001 §6 row 12) |
| C16 | RF board | RF designer | no ferromagnetic top-side parts or shield clips within 20 mm of (+24, +27) (A2 is 4 mm above the RF board's parts there); PA supply and return paired; J1 of the RF board in the (+X, −Y) corner to meet the OBC's J2 ribbon (§21) |
| C17 | Spreader NTC | carrier designer / flight software | E10 taken as proposed; the TE resistance table for GA10K3MCD1 to replace the assumed β = 3976 K in the conversion |
| C18 | EPS storage drain and the A3 protection | EPS designer | D4 (SMF24A on UMB_VMON) leaks ≤ 1 µA worst at its 24 V standoff (far less at 16.8 V) through the EPS's 100 kΩ from BAT_F: add to the LVR-071 storage-drain budget (≤ 20 µA), or accept |
| C19 | A3 size and mounting | harness and mechanical designers | A3 is 30 × 8.8 mm (the CAD envelope's height), not the 30 × 12 mm of SQSO-HAR-001 2.0 §3.2; mounted by two self-locking Ti jackposts through the panel into the Micro-D flange (no nuts, no thread-locker; Scotch-Weld 2216 staking only), in place of SQSO-HAR-001's "two Ti M2 screws" |

## 25. Not Verified and Open

- **STM32L4R5 datasheet (DS12023) not reached** from this session (st.com refused the connection): the run current
  (20 mA allocation), LSI spread, LSE drive levels and thermal resistance are class values. The pin positions and
  alternate functions are from ST's pin database as published in stm32-data (the embassy-stm32 source of truth), not
  from the PDF.
- **CR0002**: resistance, inductance, remanence and the 5 V limit's meaning were not confirmed (CubeSpace datasheet
  unreachable). The drive covers 30–80 Ω; the current limit protects the rod whatever its resistance.
- **Panel photodiodes** not yet named: Rf assumes ≤ 4.0 mA at AM0.
- **MPNs not checked** (BOM column): FM25V20A-GTR, SG-8018CA code, CM7V code, Samtec connectors, BLM18AG601SN1D,
  SMF5.0A, BAT54, 1N4148W-Q, LTST-C191KRKT, Keystone 5015, Pico-Clasp 12-position part number (pattern) and the gold order codes of every Pico-Clasp header, the Samtec TFM right-angle contact ratings and mated heights, the Micro-D 21 slash-sheet dash number, SMF24A, the M-G370
  "PDT0" order code (data sheet read for PDG0).
- **Hardware off-timer** accuracy (± 3 % after trim) follows the RF board's use of the same CD74HC4060 RC oscillator; to
  be measured (V-OBC-8).
- **Receiver-side GNSS connector** awaits the SkyFox interface document.
- **Gyro vibration rating**: the M-G370 data sheet read (PDG0, Rev 1.2) gives only the 1000 g shock; no random or sine
  rating and no Epson qualification levels were found; V-OBC-16 and item C12.
- **Spreader NTC curve**: β = 3976 K is assumed for the "10K3" curve; the TE GA10K3MCD1 table was not read.
- **Connector mated heights** (FTSH with FFSD, Pico-Clasp with plug and wire bend) against the 8.0 mm allocation (9.0 mm free
  under the 2.0 mm carrier) were not checked against the Samtec and Molex drawings (§21).
- **Magnetometer coupling numbers** come from a two-charge model of the rods at the CAD centres; the rods' remanence is
  not published, so §9.4 states the allowable remanence rather than a predicted field; AV-V4 measures it. The Sen-Z-f
  coil height against A2's 8 mm slot is from the PNI outline class, to be checked with the PNI drawing.
- **Not done**: PCB layout, EM build and test, the flight-software drivers (gyro, BMI088, RM3100, ADS7138, coil PWM,
  RTC, supervisor kick, readbacks, spreader NTC).

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-27 | First issue (CDR design completion): OBC board schematic as code with parts, netlist, BOM, ERC and calculations; supervisor (AV-02), mission clock (AV-10), on-board ADCS sensors and coil drive (AV-03), Jetson isolation and switch readbacks (AV-09, AV-06), repeat-path readbacks (RF-21, AV-05); master pin allocation in SQSO-ICX-001 |
| B | 2026-10-27 | Chair follow-up: carrier interface E10 taken (W5B 6 pins; spreader NTC on PA2 ADC1_IN7 with a 10.0 kΩ 0.1 % bias, §11.3; JETSON_PWR PA2 → PA8); stack placement constraints of MECHANICAL_DESIGN §4.3–§4.4 in §21 (gyro at (−20, +30.5), bottom side ≤ 1.0 mm, J1/J2/J7 edges and sides, no OBC frame bond); magnetometer re-assessed and moved to board A2 at the +Z end frame (§9.4; carrier keep-out withdrawn; AV-V4 residual and gating); gyro vibration against the Epson rating (§9.2, V-OBC-16); J9 FTSH-105 on internal harness W8A; RX_CD (PG0) and RX_FAULT (PF13) from the RF board on J2 (FTSH-108, W4A v3; back-power total 2.7 mA); V-OBC-16, -17; C3 closed, C12–C17 |
| C | 2026-10-27 | Harness design SQSO-HAR-001 2.0 §15 O-1 to O-3: J1, J2, J9 as Samtec TFM right-angle (TFM-120/-108/-105, gold) with SFSDT 30 AWG, plug's back ≤ 2.0 mm inside the edge; Pico-Clasp headers gold; J8 a Pico-Clasp 12 (W6D unshielded, drains removed); J7 cable strip y +7 … +19.5 kept free; access-port board A3 designed (§15.1: Micro-D 21, debug and charge groups separated, ESD and TVS, netlist, BOM, ERC); V-OBC-18; C18, C19 |
