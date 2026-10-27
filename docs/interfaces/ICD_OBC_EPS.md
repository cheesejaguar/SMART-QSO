# Interface Control Document: OBC-EPS

**Document Number**: SMART-QSO-ICD-001
**Version**: 1.8
**Date**: 2026-10-27
**Status**: Preliminary. Electrical values from the EPS board build-to design (`hardware/electrical/eps/EPS_BOARD_DESIGN.md`,
SQSO-EPS-BRD-001) and pins from the master allocation (`hardware/electrical/INTERCONNECT.md`, SQSO-ICX-001);
the register block 0x00–0x33 is the one the flight software reads (`sqso_fsw::eps_telemetry`) and is unchanged

---

## 1. Introduction

### 1.1 Purpose

This Interface Control Document (ICD) defines the hardware and software interface between the On-Board Computer (OBC) and the Electrical Power Subsystem (EPS).

### 1.2 Scope

This document covers:
- Physical/electrical interface specifications
- I2C communication protocol
- Register map and commands
- Power control interface
- Telemetry data formats

### 1.3 Interface Diagram

The OBC and EPS boards are PC/104 boards in one stack with no stack connector: harness W3 (a 40-way, 65 mm SFSDT discrete-wire assembly)
joins EPS J3 to OBC J1. The pin allocation is the master one of `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001
§3.1, §4); the EPS side is `hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001 §11, §13.2). Where this
document and SQSO-ICX-001 differ, SQSO-ICX-001 governs.

```
+----------------+   W3: EPS J3 <-> OBC J1 (TFM 40)   +----------------------------+
|                |  I2C2 SCL PF1 / SDA PF0            |  INA228 pack gauge   0x44   |
|      OBC       |<---------------------------------->|  INA228 array output 0x40   |
| (STM32L4R5ZI)  |                                    |  EPS controller      0x48   |
|                |  15 enables, 4 burn lines,         |  (MSP430FR2355)             |
|                |  EOL_ARM/EOL_FIRE, LOCK_CLR        |                             |
|                |----------------------------------->|  load switches SW1-SW11     |
|                |  EPS_ALERT_N PB7, DEPLOY_SENSE,    |  burn drivers, EOL fuse F1  |
|                |  SOLAR_SENSE, 7 analog lines       |                             |
|                |<-----------------------------------|  battery, array             |
|                |  3V3_OBC, VBAT_MTQ, 3V3_RTC        |                             |
|                |<-----------------------------------|                             |
+----------------+                                    +----------------------------+
```

---

## 2. Physical Interface

### 2.1 Connector Specification

| Parameter | OBC Side | EPS Side |
|-----------|----------|----------|
| Connector | J1, Samtec TFM-120 right-angle (40 positions) | J3, Samtec TFM-120 vertical, EPS bottom side |
| Harness | W3, Samtec SFSDT PTFE discrete-wire assembly, 30 AWG, 0.065 m (SQSO-ICX-001 §2; the FFSD ribbon is PVC and withdrawn) | W3 |
| I2C pins | I2C2_SCL J1-5 (PF1), I2C2_SDA J1-6 (PF0) | J3-5, J3-6 |
| Power | 3V3_OBC J1-1, J1-2 | SW10 output (latch-up auto-retry) |
| GND | J1-3, 4, 7, 35; AGND J1-21, 27 (analog return). J1-28 and J1-40 are I2C3_SCL (PG7) and I2C3_SDA (PB4) for the AntS (SQSO-ICX-001 D §3.1; they were GND) | GND; AGND joins GND at the EPS analog star point |
| Load control | enables on J1-9 to J1-15 (§3.2) | load switches, 100 kΩ pull-down on every enable |
| Alert | EPS_ALERT_N J1-8 (PB7, 10 kΩ series) | open drain, active low, 4.7 kΩ pull-up: INA228 alerts and the controller's interrupt |

The full 40-pin list, with the burn, passivation, lockout, sense and readback lines, is SQSO-ICX-001 §3.1.

### 2.2 Electrical Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| I2C SCL frequency | - | 100 | 100 | kHz |
| I2C Logic High | 2.1 | 3.3 | 3.6 | V |
| I2C Logic Low | 0 | 0 | 0.6 | V |
| I2C Pull-up (on the OBC: two 4.7 kΩ in parallel, each works alone; SQSO-ICX-001 §3.1) | - | 2.35 | - | kΩ |
| Load Control High | 2.4 | 3.3 | 3.6 | V |
| Load Control Low | 0 | 0 | 0.4 | V |

### 2.3 I2C Configuration

| Parameter | Value |
|-----------|-------|
| Bus | I2C2 (`hal_i2c` bus 0), PF1 SCL / PF0 SDA |
| Mode | Standard/Fast |
| Clock Speed | 100 kHz (SQSO-ICX-001 §3.1: two 4.7 kΩ pull-ups in parallel, 100 kHz recommended) |
| Address Mode | 7-bit |
| EPS controller (MSP430FR2355, register block of §4) | 0x48 |
| Pack gauge (INA228-Q1 U28 on the high-side pack shunt: voltage, current, charge, energy) | 0x44 (was "battery monitor 0x55") |
| Array output monitor (INA228-Q1 U29) | 0x40 (was "solar monitor 0x40") |
| AntS | **not on I2C2** from SQSO-EPS-BRD-001 E: on the OBC's I2C3 (J3-28 SCL, J3-40 SDA) through the EPS's TCA9517-Q1 U80, side A 0x31 and side B 0x32 (SQSO-ICX-001 E14/E15) |
| OBC board temperature | none on I2C2: the OBC board uses an analog TMP235-Q1 on PB1 instead of the TMP117 that clashed with 0x48 (finding F10 of SQSO-HW-DIAG-001, closed; SQSO-OBC-BRD-001 §2.2) |

The two INA228s are read directly by the OBC, independently of the EPS controller. The battery AFE (BQ76952, 0x08),
the four INA3221s and the ADS7138 sit on the controller's local bus and are not on I2C2.

### 2.4 Analog and Discrete Lines

| Signal (J1 pin, OBC pin) | Source on the EPS | Scale at the OBC ADC | Firmware scale |
|--------------------------|-------------------|----------------------|----------------|
| VBATT_SNS (22, PC2 ADC1_IN3) | PACK_P, 1.00 MΩ / 200 kΩ (0.1 %), 100 nF | pack / 6: 2.80 V at 16.8 V | `Linear(6.0, 0.0)` (was `Linear(3.0, 0.0)`, a 2S value; finding F1) |
| IBATT_SNS (23, PC3 ADC1_IN4) | INA186 × 50 on the pack shunt, 1.65 V reference | 0.5 V/A, 1.65 V at 0 A, ±3.3 A | unchanged |
| VSOLAR_SNS (24, PC4 ADC1_IN13) | highest string, 300 kΩ / 100 kΩ | string / 4 | unchanged |
| ISOLAR_SNS (25, PC5 ADC1_IN14) | INA186 × 50 on the array-output shunt | 1.0 V/A | unchanged |
| VBUS_SNS (26, PB0 ADC1_IN15) | 5V_PA, 100 kΩ / 100 kΩ | 5V_PA / 2 | unchanged |
| VJET_RB (19, PA7 ADC1_IN12) | VBAT_JET after the eFuse, 1.00 MΩ / 200 kΩ | VBAT_JET / 6 | new channel |
| VSW1_RB (37, PA1 ADC1_IN6) | VBAT_SW1 after SW1, 1.00 MΩ / 200 kΩ | VBAT_SW1 / 6 | new channel |
| DEPLOY_SENSE (16, PE2) | DS2 released: high (1 MΩ source, clamped) | digital | — |
| SOLAR_SENSE (20, PB12) | both wing hinge switches in series | low = both wings deployed | — |

---

## 3. Power Interface

### 3.1 Power Rails

| Rail | Voltage | Current Max | Purpose |
|------|---------|-------------|---------|
| 3V3_AO | 3.3 V ±2 % (3.23–3.37 V) | 0.30 A | Source of the switched 3.3 V rails: OBC, EPS controller, receiver path, ADCS sensors, GNSS, AntS |
| 3V3_OBC | 3V3_AO through SW10 | 0.040 A (trip 0.15 A) | OBC, J1-1/2 |
| 3V3_RTC | 3V3_AO through 1.0 kΩ, always on | ≤ 2 µA | OBC mission clock RV-3028-C7, J1-38 |
| 5V_PA | 5.0 V ±3 % | 0.54 A (switch trip 0.99 A) | PA only |
| VBAT_JET | VBAT through SW1 and the eFuse | 0.93 A (eFuse 1.20 A) | Jetson carrier |
| VBAT_MTQ | VBAT through SW11 | 0.104 A at 1.16 W; 0.23 A at the bridges' limit (trip 0.361 A) | OBC magnetorquer bridges, J1-36/39 |
| VBAT | 11.2–16.4 V (16.8 V abs max) | 1.7 A peak | Raw battery bus: heater, burn buck, converters |

### 3.2 Load Switch Control

Enables are 3.3 V CMOS outputs of the OBC, active high, high-impedance at OBC reset, held low by 100 kΩ on the EPS.
Pins per SQSO-ICX-001 §3.1; switches, limits and loads per SQSO-EPS-BRD-001 §8.3.

| Load | Enable (OBC pin, J1 pin) | Switch | Normal max | Trip, nominal (min–max) |
|------|--------------------------|--------|------------|-------------------------|
| OBC 3.3 V | none (always on) | SW10 TPS2553-Q1 | 0.040 A | 0.152 A (0.130–0.176 A) |
| EPS controller | none (always on) | SW9 TPS2553-Q1 | 0.012 A | 0.075 A (0.050–0.100 A) |
| Payload (Jetson) | PAYLOAD_EN (PB2, 9) on SW1 and JETSON_PWR (PA8, 29; SQSO-ICX-001 B) on the eFuse; SW1 and the eFuse both shed-gated (SQSO-EPS-BRD-001 D) | SW1 TPS1H100B-Q1; eFuse TPS16630 | 0.93 A at VBAT | SW1 1.99 A; eFuse 1.20 A (1.10–1.30 A) |
| Radio (RX, TNC, exciter) | RADIO_EN (PB5, 10) | SW2 TPS2553-Q1 | 0.038 A at 3.3 V | 0.120 A (0.102–0.141 A) |
| PA (hardware TX-OFF) | RF_TX_EN (PA3, 14), AND the hardware load shed | SW3 TPS2553-Q1 | 0.54 A at 5 V | 0.989 A (0.918–1.071 A) |
| ADCS sensors (3V3_ADCS) | ADCS_EN (PB6, 11) | SW4 TPS2553-Q1 | 0.049 A at 3.3 V | 0.299 A (0.265–0.339 A) |
| Magnetorquer bridges (VBAT_MTQ) | ADCS_EN (PB6, 11) | SW11 TPS1H000-Q1 | 0.23 A at VBAT | 0.361 A (0.322–0.401 A) |
| Heater | HEATER_EN (PD2, 12) | SW5 TPS1H000-Q1 | 0.079 A at VBAT | 0.148 A (0.126–0.170 A) |
| Deployment (burn buck) | DEPLOY_EN (PF3, 13) | SW6 TPS1H100B-Q1 | 1.25 A at VBAT | 1.99 A (1.71–2.27 A) |
| AntS 3.3 V | DEPLOY_EN (PF3, 13) OR the EPS controller's ANTS_PWR (0x8D) | SW8 TPS2553-Q1 | 0.61 A at 3.3 V | 1.370 A (1.287–1.465 A) |
| GNSS | GNSS_EN (PE5, 15) | SW7 TPS2553-Q1 | 0.100 A at 3.3 V | 0.202 A (0.175–0.232 A) |

The four burn lines (BURN_W1A PB10, BURN_W1B PE6, BURN_W2A PB9, BURN_W2B PE7; J1-18, 30, 17, 31) each start one
driver's 12 s one-shot; the passivation lines EOL_ARM (PF4, J1-32) and EOL_FIRE (PG1, J1-33) and the lockout clear
LOCK_CLR (PF5, J1-34) are listed in SQSO-ICX-001 §3.1. SW1 and the Jetson eFuse are also gated by the EPS's hardware low-voltage
load shed (the PA's SW3 is not, from SQSO-EPS-BRD-001 E) (VBAT below 13.6 V for 5 s; EPS_DESIGN §6, SQSO-EPS-BRD-001 §8.4).

### 3.3 Power Sequencing

#### Boot Sequence
```
T=0:      EPS enables 3.3V_OBC
T+10ms:   OBC starts boot
T+100ms:  OBC initializes I2C
T+200ms:  OBC reads EPS status
T+500ms:  OBC enables loads per mode
```

#### Safe Mode Entry
```
T=0:      Alert pin asserted OR SOC < 25%
T+10ms:   OBC disables non-essential loads
T+50ms:   OBC enters SAFE mode
T+100ms:  Only beacon transmissions active
```

---

## 4. Communication Protocol

### 4.1 I2C Transaction Format

#### Write Transaction
```
START | ADDR+W | REG | DATA[0] | ... | DATA[n] | STOP
```

#### Read Transaction
```
START | ADDR+W | REG | RESTART | ADDR+R | DATA[0] | ... | DATA[n] | STOP
```

### 4.2 Register Map - EPS Controller (0x48)

| Register | Address | R/W | Size | Description |
|----------|---------|-----|------|-------------|
| STATUS | 0x00 | R | 1 | System status |
| (retired) | 0x01 | — | 1 | Formerly MODE: retired in 1.7 (the controller sets no mode); reads 0x00, writes acknowledged and ignored |
| (retired) | 0x02 | — | 1 | Formerly LOAD_EN: retired in 1.7 (the controller drives no load; the loads are the OBC's own lines, §3); reads 0x00, writes acknowledged and ignored |
| VBATT | 0x10 | R | 2 | Battery voltage (mV) |
| IBATT | 0x12 | R | 2 | Battery current (mA, signed) |
| TBATT | 0x14 | R | 2 | Battery temp (0.1°C) |
| SOC | 0x16 | R | 1 | State of charge (%) |
| VSOLAR | 0x20 | R | 2 | Solar voltage (mV) |
| ISOLAR | 0x22 | R | 2 | Solar current (mA) |
| PSOLAR | 0x24 | R | 2 | Solar power (mW) |
| VBUS | 0x30 | R | 2 | 5V bus voltage (mV) |
| IBUS | 0x32 | R | 2 | 5V bus current (mA) |
| ENERGY | 0x40 | R | 4 | Energy consumed (mWh) |
| BOOT_CNT | 0x50 | R | 2 | Boot counter |
| ALERT_CFG | 0x60 | R/W | 4 | Alert configuration |
| VERSION | 0xF0 | R | 2 | Firmware version |
| IMAGE_CRC | 0xF2 | R | 2 | CRC-16 of the controller's flight image, computed at start-up (EPSC-I010; adopted in 1.7) |

### 4.3 Status Register (0x00)

| Bit | Name | Description |
|-----|------|-------------|
| 7 | CHARGING | Battery charging active |
| 6 | SUNLIT | Solar power available |
| 5 | LOW_BATT | Low battery warning |
| 4 | OVER_TEMP | Over temperature |
| 3 | UNDER_TEMP | Under temperature |
| 2 | FAULT | Fault condition active |
| 1:0 | — | Reserved, read 0 (MODE retired in 1.7) |

### 4.4 Mode Register (0x01) and 4.5 Load Enable Register (0x02): retired

Retired in 1.7 (SQSO-SRS-EPSC-001 §9 item 2): the EPS controller drives no load enable and sets no power mode; the
OBC's own lines switch the loads (§3) and the OBC keeps the mode. The addresses stay reserved (read 0x00, writes
acknowledged and ignored) so that no old image misreads another register.

---

### 4.6 Extended Registers (added at CDR; the flight software reads CHG_INHIBIT, EOL_STATE, IMON_JET and EPS_SUPV and writes LOCK_UNLOCK, EOL_KEY, OBC_HEARTBEAT, ANTS_PWR and TRK_DISABLE)

| Register | Address | R/W | Size | Description |
|----------|---------|-----|------|-------------|
| VCELL1–4 | 0x70–0x77 | R | 4 × 2 | Cell voltages (mV) |
| TBATT2 | 0x78 | R | 2 | Second pack temperature (0.1 °C) |
| CHG_INHIBIT | 0x7A | R | 1 | Charge-inhibit cause: bit 0 cold, 1 hot, 2 cell OV, 3 BMS fault, 4 secondary protector blown, 5 recharge lockout latched, 6 NTC1 implausible (window and 60 °C trip handed to the AFE on NTC2; SQSO-EPS-BRD-001 E §5.3) |
| LOAD_SHED | 0x7B | R | 1 | Hardware low-voltage load shed active (bit 0: SW1 and the eFuse U19 shed; the PA is not shed from SQSO-EPS-BRD-001 E), SW1/SW3 fault flags (bits 1–2) |
| EOL_STATE | 0x7C | R | 1 | Passivation: bit 0 armed, bit 1 fired (charge path open), bit 2 fuse F1 intact (F1_SNS; SQSO-EPS-BRD-001 §19), bit 3 EOL key active (EOL_KEY high, see 0x7E). Read every housekeeping frame and in the final passivation report (HK1 `PS`, HK2 `EOL`; SRS-F155) |
| LOCK_UNLOCK | 0x7D | W | 1 | Recharge-lockout unlock: writing 0xC5 raises UNLOCK, the second of the two conditions for a lockout clear (with LOCK_CLR; EPS_CLEAR_LOCKOUT 0x14, SRS-F129), **for 1.0 s, then UNLOCK clears itself** (1.7); any other value lowers it at once. Sequence: write 0xC5, then within 0.5 s raise LOCK_CLR (PF5) for ≥ 200 ms, so the two overlap ≥ 100 ms (the reset one-shot U59 needs ≥ 100 ms) |
| EOL_KEY | 0x7E | W | 1 | Passivation arm key (SQSO-EPS-BRD-001 D §7, §12; RID P3E-11): writing **0x3C then 0xC3** within 1 s, accepted only while EOL_STATE bit 0 = 0 (not armed) and bit 2 = 1 (F1 intact), makes the EPS controller raise EOL_KEY for 10 s (EOL_STATE bit 3 = 1); the arm latch sets only while EOL_KEY and the OBC's EOL_ARM (PF4, J3-32) are both high. Any other value, a write out of order, or the 10 s timeout lowers EOL_KEY; a refused sequence leaves bit 3 = 0 |
| PTRACK1–4 | 0x80–0x87 | R | 4 × 2 | Tracker input power (mW) |
| IHEATER | 0x88 | R | 2 | Heater current (mA) |
| IMON_JET | 0x8A | R | 2 | Jetson rail current from the eFuse U19 IMON (0.84 V/A at the controller's ADC), mA, little-endian like the other words: the flight software's JPC (SQSO-JET-BRD-001 §13 item F3, RID JETB-03) |
| OBC_HEARTBEAT | 0x8C | W | 1 | OBC heartbeat (RID SMA-B09; SQSO-ICX-001 E14): the flight software writes **0x5A at least every 60 s**. After **600 s** without a valid write, and not within 600 s of the controller's start or of the last cycle, the controller power-cycles the OBC rail SW10 once (hardware one-shot U82: SW10 off 1.95 s, back after 0.28 s). At most **3 cycles in 24 h**, then supervision stops (EPS_SUPV bit 7); 10 consecutive valid heartbeats reset the count. Any other value is ignored (not a heartbeat). No command disables it |
| ANTS_PWR | 0x8D | W | 1 | AntS supply by the EPS controller (RID SMA-B06): **0xA6** raises ANTS_EN (SW8 on, in OR with DEPLOY_EN) for at most 30 min; any other value lowers it. SW8 is downstream of DS1–DS3 |
| TRK_DISABLE | 0x8E | W | 2 | Per-tracker disable (RID SMA-B17): byte 0 key **0x96**, byte 1 mask (bits 0–3 = T1–T4 off; 0 = all enabled), in one write; a write without the key is ignored; **a mask that disables all four trackers (0x0F) or sets a bit above bit 3 is refused with a NACK and no change** (1.7, EPSC-F092: with all four off the pack is never charged again); the mask is held in FRAM and reported in EPS_SUPV |
| EPS_SUPV | 0x8F | R | 1 | Supervisor status: bits 0–3 trackers disabled (T1–T4), bit 4 ANTS_EN from the controller, bits 5–6 OBC power cycles in the last 24 h (0–3), bit 7 heartbeat supervision stopped (limit reached) |
| BURN_I1–4, V_BURN, VTRK | 0x90–0x9B | R | 6 × 2 | Burn currents W1A, W1B, W2A, W2B (mA), V_BURN and VTRK (mV), every 100 ms (SQSO-SRS-EPSC-001 EPSC-F014; adopted in 1.7) |
| EPSC_FLAGS | 0x9C | R | 1 | Controller flags: bit 0 SOC uncalibrated, bit 1 calibration default, bit 3 burn overrun (> 0.4 A for > 13 s), bit 4 AFE settings differ from the acceptance record, bit 5 a local-bus device failed, bit 6 LOCK_SET event; bits 2, 7 reserved (EPSC-F005, F010, F014, F041, F060, F063) |
| RESET_CAUSE | 0x9D | R | 1 | Cause of the controller's last reset (EPSC-F004 codes) |
| EOL_REFUSED | 0x9E | R | 1 | Count of refused EOL_KEY sequences, saturating at 255 (EPSC-F072) |

**Passivation sequence (OBC, SQSO-EPS-BRD-001 D §7).** On PASSIVATE_ARM (authenticated): (1) read EOL_STATE and
require bit 0 = 0 and bit 2 = 1; (2) write 0x3C then 0xC3 to EOL_KEY (0x7E) within 1 s; (3) read EOL_STATE bit 3 = 1;
(4) raise EOL_ARM (PF4) within 10 s of step 2, confirm EOL_STATE bit 0 = 1, lower PF4. On PASSIVATE_EXECUTE
(authenticated, later, in sunlight; the OBC waits up to 6 h per attempt for the EPS board to read ≥ 0 °C by the INA228
die temperatures and then fires anyway, or at once if the temperature is unreadable (SRS-F111, SMA-B20); F1's cold
blow is qualified at −20 °C, SQSO-EPS-BRD-001 G §7, V-EPS-8): hold EOL_FIRE (PG1) high ≥ 60 s, then confirm EOL_STATE bit 1 = 1 and bit 2 = 0.
The arm latch clears at every EPS power-up; a failed step leaves the latch clear and is reported.

## 5. Data Formats

### 5.1 Voltage Format

16-bit unsigned, LSB first, units of millivolts (mV)

```
Example: VBATT = 0x3A98 = 15000 mV = 15.0 V (4S pack at about 55 % SOC)
```

### 5.2 Current Format

16-bit signed (two's complement), LSB first, units of milliamps (mA)

```
Example (positive): IBATT = 0x03E8 = +1000 mA (charging)
Example (negative): IBATT = 0xFC18 = -1000 mA (discharging)
```

### 5.3 Temperature Format

16-bit signed, LSB first, units of 0.1°C

```
Example: TBATT = 0x00FA = 250 = 25.0°C
Example: TBATT = 0xFF38 = -200 = -20.0°C
```

### 5.4 State of Charge Format

8-bit unsigned, units of percent (%)

```
Example: SOC = 0x4B = 75%
```

### 5.5 Energy Format

32-bit unsigned, LSB first, units of milliwatt-hours (mWh)

```
Example: ENERGY = 0x00001388 = 5000 mWh = 5.0 Wh
```

---

## 6. Command Interface

### 6.1 Set Power Mode, 6.2 Set Load Enables: retired in 1.7

MODE (0x01) and LOAD_EN (0x02) are retired (§4.4). Writes are acknowledged and ignored.

### 6.3 Configure Alerts

**Write to ALERT_CFG register (0x60)**

| Byte | Description |
|------|-------------|
| 0-1 | Low voltage threshold (mV) |
| 2-3 | High temperature threshold (0.1°C) |

---

## 7. Telemetry Collection

### 7.1 Polling Sequence

```
1. Read STATUS (0x00) - 1 byte
2. Read VBATT (0x10) - 2 bytes
3. Read IBATT (0x12) - 2 bytes
4. Read TBATT (0x14) - 2 bytes
5. Read SOC (0x16) - 1 byte
6. Read VSOLAR (0x20) - 2 bytes
7. Read ISOLAR (0x22) - 2 bytes
8. Read VBUS (0x30) - 2 bytes
```

### 7.2 Polling Rate

| Parameter | Rate | Requirement |
|-----------|------|-------------|
| Battery voltage | 1 Hz | SRS-F010 |
| Battery current | 1 Hz | SRS-I011 |
| Battery temp | 0.2 Hz | SRS-I011 |
| SOC | 1 Hz | SRS-F010 |
| Solar | 1 Hz | SRS-I012 |
| Bus | 1 Hz | SRS-I011 |

### 7.3 Telemetry Format for Beacon

The beacon carries `M=`, `SOC=`, `BV=` and `SUN=` from these readings in its telemetry tail, in the format of
`docs/BEACON_SPEC.md` §3–§4; housekeeping values go in HK1 and HK2 (`docs/operations/TELEMETRY_DICTIONARY.md` §4.4).
The comma-separated `BV=…V,BI=…mA,…` string of Version 1.0 is not what the flight software sends.

---

## 8. Error Handling

### 8.1 I2C Error Handling

| Error | Detection | Recovery |
|-------|-----------|----------|
| NACK | No ACK after address | Retry (3x), log fault |
| Timeout | No response within 100ms | Reset I2C, retry |
| Arbitration | Multi-master conflict | Retry after delay |
| Bus stuck | SDA or SCL stuck low | Clock recovery sequence |

### 8.2 I2C Recovery Sequence

```
1. Generate 9 clock pulses on SCL
2. Generate STOP condition
3. Re-initialize I2C peripheral
4. Retry transaction
```

### 8.3 Alert Handling

When Alert GPIO is asserted:
1. Read STATUS register immediately
2. Identify alert source
3. Take corrective action per fault type
4. Log fault event

---

## 9. Timing Requirements

### 9.1 I2C Timing

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Transaction time | - | 2 | 10 | ms |
| Bus turnaround | 10 | - | - | μs |
| Retry delay | 10 | - | - | ms |

### 9.2 Alert Response

| Parameter | Max | Unit |
|-----------|-----|------|
| Alert to read | 50 | ms |
| Alert to load disable | 100 | ms |
| Alert to mode change | 500 | ms |

---

## 10. Test Requirements

### 10.1 Interface Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| IFT-EPS-001 | I2C connectivity | ACK received |
| IFT-EPS-002 | Register read | All registers readable |
| IFT-EPS-003 | Mode setting | Mode changes correctly |
| IFT-EPS-004 | Load control | Loads respond to enables |
| IFT-EPS-005 | Alert detection | Alert GPIO functional |
| IFT-EPS-006 | Error recovery | Recovery from bus errors |

### 10.2 Integration Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| INT-EPS-001 | Power mode sequence | All transitions verified |
| INT-EPS-002 | Telemetry accuracy | ±2% of reference |
| INT-EPS-003 | Load switch timing | Within spec |
| INT-EPS-004 | Fault response | Correct mode on fault |

---

## Appendix A: Reference Documents

1. STM32L4 Reference Manual (RM0351)
2. I2C-bus specification (UM10204)
3. SMART-QSO Software Requirements Specification
4. SMART-QSO EPS Design Document (`docs/hardware/EPS_DESIGN.md`)
5. EPS board build-to design, SQSO-EPS-BRD-001 (`hardware/electrical/eps/EPS_BOARD_DESIGN.md`)
6. Spacecraft electrical interconnect, SQSO-ICX-001 (`hardware/electrical/INTERCONNECT.md`)

---

## Appendix B: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | CDR review (EPS) | 4S values: rails, VBAT range, load switches and limits (§3.1, §3.2), examples (§5.1, §7.3); OBC named STM32L4R5; I²C address conflict noted (§2.3); extended registers proposed (§4.6) |
| 1.2 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: connector, pins and harness W3 per SQSO-ICX-001 (§1.3, §2.1); I²C addresses INA228 0x44 (pack) and 0x40 (array), EPS controller MSP430FR2355 0x48, no OBC device on I2C2 (§2.3); analog lines with the VBATT ×6 scaling (§2.4); rails and load switches per SQSO-EPS-BRD-001 §8.3, SW4 load 0.049 A, SW11 VBAT_MTQ, 3V3_RTC (§3.1, §3.2) |
| 1.3 | 2026-10-27 | CDR flight software round 2 | §4.6: CHG_INHIBIT bit 5 (recharge lockout latched), LOCK_UNLOCK 0x7D (key 0xC5) for EPS_CLEAR_LOCKOUT; the flight software reads CHG_INHIBIT and EOL_STATE |
| 1.4 | 2026-10-27 | CDR flight software round 3 | §4.6: EOL_STATE bit 2 fuse F1 intact; IMON_JET 0x8A (Jetson rail current, the JPC sensor) |
| 1.5 | 2026-10-27 | CDR design (EPS board) | §3 payload row: JETSON_PWR on PA8 (PA2 is the spreader thermistor), both Jetson switches shed-gated; §4.6 (amended): EOL_KEY register 0x7E (key 0x3C, 0xC3), EOL_STATE bit 3, the OBC passivation sequence (RID P3E-11) |
| 1.6 | 2026-10-27 | CDR design (EPS board) | SQSO-EPS-BRD-001 E: §4.6 OBC_HEARTBEAT 0x8C (every ≤ 60 s, 600 s timeout, ≤ 3 cycles per 24 h), ANTS_PWR 0x8D, TRK_DISABLE 0x8E, EPS_SUPV 0x8F; CHG_INHIBIT bit 6 (NTC1 implausible); LOAD_SHED without the PA; §2 AntS moved to the OBC's I2C3 (0x31, 0x32); AntS supply also by the controller; amended: passivation fire only with the EPS board ≥ 0 °C |
| 1.7 | 2026-10-27 | CDR design (EPS board) | From SQSO-SRS-EPSC-001 §9: LOCK_UNLOCK self-clears after 1.0 s with the clear sequence stated; MODE (0x01) and LOAD_EN (0x02) retired (§4.2–§4.5, §6.1–§6.2); registers 0x90–0x9E and IMAGE_CRC 0xF2 adopted; TRK_DISABLE refuses an all-four mask; §3 load shed without the PA; amended: the passivation fire's temperature rule as SRS-F111 (6 h wait, then fire) |
| 1.8 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): §1.3, §2.1 connector and harness per SQSO-ICX-001 D §2 (TFM-120 with an SFSDT 30 AWG assembly, 0.065 m; was FTSH with an FFSD ribbon), J1-28 and J1-40 are I2C3 for the AntS, not GND; §2.2, §2.3 pull-up two 4.7 kΩ in parallel (was 2.2 kΩ) and I2C2 at 100 kHz per SQSO-ICX-001 (was "designed for 400 kHz"); §7.3 beacon tail per BEACON_SPEC (was the Version 1.0 string) |
