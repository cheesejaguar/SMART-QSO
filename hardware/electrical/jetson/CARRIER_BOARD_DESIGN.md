# Jetson Orin Nano Carrier Board Design

**Document ID**: SQSO-JET-BRD-001
**Version**: D
**Date**: 2026-10-27
**Status**: CDR design completion (RID SE-01, carrier part). Schematic captured as code with selected parts, netlist,
BOM, ERC and calculations. Not built; PCB layout not started (section 18 is the layout input).

**Sources (all values trace to these)**: `docs/BASELINE_PARAMETERS.md` Rev E §5–§7 (the values used here are unchanged from Rev D); `docs/hardware/PAYLOAD_DESIGN.md` 1.2
§11–§12; `docs/interfaces/ICD_OBC_JETSON.md` 1.3; `hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001)
§8.3, §11, §13; `hardware/electrical/obc/OBC_BOARD_DESIGN.md` (SQSO-OBC-BRD-001) §11; `hardware/electrical/INTERCONNECT.md`
(SQSO-ICX-001) §3.3, §7; `hardware/mechanical/MECHANICAL_DESIGN.md` §4.1, §4.3–§4.5, §6.10, §11 (items 5, 9, 12); `docs/environment/THERMAL_ANALYSIS.md`
2.0 §6.3; `docs/environment/EMC_ANALYSIS.md` (SQSO-EMC-001) A; `docs/environment/RADIATION_ANALYSIS.md` C/D §5.3.
NVIDIA: *Jetson Orin Nano Series Modules Data Sheet* DS-11105-001 v1.5 (December 2024), cited below as **DS**;
*Jetson Linux Developer Guide* r36.4.4, "Platform Power and Performance" and "Quick Start" (docs.nvidia.com), cited as
**JLG**. Not retrieved in this session (mirrors returned 403/503, NVIDIA's download centre needs a login): the *Orin
NX/Nano Product Design Guide* DG-10931-001 (**DG**) and the *Thermal Design Guide* TDG-11127-001 (**TDG**); every value
that needs them is marked and is a verification item.

Files: `design.py` (SKiDL 3.0 schematic), `jetson_calcs.py` (every calculation below; `jetson_calcs.json`),
`jetson_carrier.net` (KiCad 8 netlist), `jetson_carrier_bom.csv`, `jetson_carrier_erc.log`, `jetson_carrier_nets.csv`,
`module_pins.json` (the 260 module pins, DS §4.4).

---

## 0. Summary

| Item | Result |
|------|--------|
| What the module needs | **VDD_IN 4.75–5.25 V (5.5 V absolute)**, not the battery bus: the Orin Nano is a "MODULE_ID low" module that accepts 5 V only (DS Tables 5-1, 5-2; 5–20 V is the Orin NX). The carrier therefore converts VBAT 11.2–16.4 V to 5.0 V |
| Power path | J1 → TVS → LC filter (damped) → **U1 LM61440-Q1** 5.02 V buck at 1.000 MHz (synchronised) → **U2 TPS259474L** eFuse: 5.38 V overvoltage lock-out, 2.51 A circuit breaker, latch-off, power-good → VDD_IN |
| Sequencing | POWER_EN = wired AND of U2 power-good (VDD_IN ≥ 4.64 V) and U4 supervisor (input ≥ 10.49 V for 19.4 ms, and no shutdown latched); carrier I/O rails switched by the module's SYS_RESET* (DS §3.6.1), discharged within 0.09 ms (3.3 V) and 0.19 ms (1.8 V) of reset (limits 1.5 and 4 ms, DS §3.6.2) |
| OBC interface | UART1 (module pins 203/205, 1.8 V) through SN74LVC1T45-Q1 translators to 3.3 V; **JET_3V3 on W5B pin 4** = the switched 3.3 V I/O rail (≤ 1 mA drawn), high only after the module's power sequence: the carrier power-good (AV-06) and the isolation reference (AV-09) |
| Storage | The production module (P3767-0003) has only its QSPI-NOR boot flash; no eMMC and no SD card (JLG Quick Start). Rootfs, model and data on a **Swissbit N2600 40 GB pSLC M.2 2230 NVMe** (3.3 V only, −40…85 °C), PCIe C4 lane 0 |
| Burst power (≤ 8 W) | 7 W mode at its full budget: **7.72–7.87 W modelled, 8.03 W with the converter at its 93 % data-sheet bound: the 8 W limit is met with no usable margin**. With the module capped at 6.5 W (VDD_IN average limit) **7.19–7.48 W (+6.5 to +10 %)**. Recommended: cap at 6.5 W |
| 7 W mode (TB-13) | Exists only in the **original** flash configuration (`jetson-orin-nano-devkit`, nvpmodel ID 1: 4 cores ≤ 960 MHz, 2 TPC ≤ 408 MHz, EMC 2133 MHz); the Super configuration has 15 W, 25 W, MAXN_SUPER (JLG). The design assumes the original configuration with 7 W as the default mode |
| Thermal interface | Gap pad from the SoC lid to a spreader pedestal (no thermal transfer plate on this module): junction-to-spreader **0.9–1.2 K/W estimated** against the 5.7 K/W THM-10 allowable; clamp force ≤ 38 N (48 % of the 80 N module limit) |
| EMC | CE102 differential-mode estimate ≥ 37 dB under the MIL-STD-461G basic curve; all carrier converters on one crystal clock at 1.000 MHz (the EPS frequency plan): no harmonic within 200 kHz of either uplink window. L1 cannot be protected from the module's own 38.4 MHz reference (41st harmonic 1.02 MHz below L1, EMC-04): GNSS sessions stay with the Jetson off |
| Parts | 137 placements, 51 BOM lines, plus the module, the SSD and 5 mechanical items; 12 ICs (10 TI AEC-Q100, TPS25947 and SN74LVC1G74 industrial); all stresses within EEE-INST-002 / 50 % rules (§17) |
| Carrier dissipation | 0.72 W at 7 W module power (U1 0.28 W, 3.3 V side 0.38 W of which the SSD 0.30 W, U2 0.06 W); hottest junction 70 °C at a 60 °C board |
| Gyro window, heights (B, C), magnetometer (D) | 27 × 27 mm window for the OBC's M-G370 gyro at y +17.0 … +44.0 with its bottom-side Al frame (MECHANICAL_DESIGN §4.3–§4.5, D-11): the power train moved to the −Y strip and the −X column, the I/O parts to the +X edge; 0 placement conflicts and every part within the top-side height limits (7.7 mm under the spreader, 13.2 mm beyond; `jetson_calcs.py` table 14). The RM3100 keep-out of version B is withdrawn (magnetometer moved to (+24, +27), OBC item E11); the parts stay as placed (§18.2) |
| Not met or not verifiable | 8 W at the full 7 W budget (no margin: cap the module); hold-up after an uncommanded power cut 0.11 ms (the normal shutdown must be commanded, §7.4); DG/TDG values (sequence timing, AC-coupling location, SoC lid area and θjc) |

## 1. Requirements and Where They Are Met

| Requirement | Source | Design | § |
|-------------|--------|--------|---|
| Module power from VBAT 11.2–16.4 V behind the EPS eFuse (1.20 A, latch-off) | brief; EPS-BRD §8.3, §13.1 (J18) | J1 Micro-Fit 2×2; U1 5 V buck, U2 eFuse | 5, 6 |
| Module input and sequencing | DS Table 3-1, §3.6, §3.7, Tables 5-1/5-2 | VDD_IN 4.75–5.21 V; POWER_EN logic; SYS_RESET*-switched I/O rails | 6, 7 |
| 7 W mode, ≤ 8 W input including the carrier, no fan | BASELINE §6 | budget §13; passive heat path §14 | 13, 14 |
| ≤ 10 minutes per power-on | BASELINE §6 | OBC-timed; commanded shutdown handshake | 7.4 |
| UART to the OBC, isolated when the carrier is off | ICD-002 §9; ICX-001 E9; RIDs AV-06, AV-09 | U10/U11 translators on 1V8_IO/3V3_IO; JET_3V3 on pin 4 | 8, 9 |
| Independent current and temperature telemetry for the OBC's trips | BASELINE §6; `payload_guard.rs` | EPS IMON_JET (existing); spreader NTC RT1 passed through (new pins E10) | 12 |
| Storage for the boot image and the model | brief | QSPI on module (boot firmware); M.2 NVMe | 10 |
| Minimum interfaces; no camera; USB only on a debug header | brief | §11 table | 11 |
| Heat path ≥ 1 W/K to structure; junction-to-spreader for THM-10 | BASELINE §6; THA §6.3 | §14 | 14 |
| CE102 (V-EMC-1); frequency plan near 435 MHz and L1 | SQSO-EMC-001 §5 E1–E3, §6 | §15 | 15 |
| Latch-up trip on every rail | BASELINE §6 (radiation row) | U2 breaker, U3 limit, EPS U19, OBC guard | 12 |

## 2. What the Module Data Sheet Fixes (DS-11105-001 v1.5)

1. **5 V only.** Table 5-1: VDD_IN (MODULE_ID low) 4.75–5.25 V, "MODULE_ID low is for Nano"; Table 5-2 absolute 5.5 V,
   I_DD max 5 A. The interconnect's VBAT feed (W5A v2) is correct, but the module cannot take it directly: the carrier's
   own converter is mandatory. Pin 217 (MODULE_ID) is grounded on the Nano; the carrier leaves it open (test point) and
   supplies 5 V, which also suits an Orin NX if one were fitted.
2. **Sequencing (§3.6).** The carrier asserts POWER_EN (pin 237, 100 kΩ pull-down on the module, Schmitt input) only with
   VDD_IN stable; the module drives SYS_RESET* (pin 239, open drain, 10 kΩ to its 1.8 V) high when its own sequence is
   done, and that signal is "used as carrier board supply enable". **No I/O pin may exceed 0.5 V while SYS_RESET* is
   low.** Power-down: the carrier deasserts POWER_EN (≥ 10 µs, Table 3-3), the module asserts SYS_RESET*, and carrier
   3.3 V I/O must be below 0.5 V within 1.5 ms, 1.8 V I/O within 4 ms. SHUTDOWN_REQ* (pin 233, open drain, ~5 kΩ to
   VDD_IN) asks the carrier to drop POWER_EN "as soon as possible" (thermal, software, under-voltage).
3. **Storage.** "Supports external storage: NVMe through PCIe; SSD through USB 3.2"; the module boots from its QSPI
   (§4.1). JLG Quick Start: the production Orin Nano 8 GB (P3767-0003) flashes "QSPI-NOR and USB/NVMe drive"; only the
   developer-kit module P3767-0005 has a microSD slot. PAYLOAD_DESIGN §3.2 ("16 GB eMMC on-module") and
   INTEGRATION_DESIGN §5 (microSD) were wrong.
4. **UARTs are 1.8 V CMOS** (Table 2-13), and "external devices must use two stop bits" because the receiver has low
   baud-rate tolerance with one (§2.10.4). The OBC must transmit 8N2 (§9; flight-software change for the chair).
5. **Mechanics and environment.** 69.6 × 45 mm, 28 g; mounting force on the module PCB ≤ 8.2 kgf (80 N, Table 5-2);
   Tj −25…105 °C (since v1.3); operating lifetime 5 years 24×7; screening includes 4,000 hard power on/off cycles
   (Table 5-5), random vibration only 2 Grms non-operating (the 8.18 Grms protoflight level is beyond NVIDIA's test:
   EM vibration of the populated carrier is a verification item, V-JET-8).
6. **Power modes (JLG r36.4.4).** Orin Nano 8 GB, original configuration: 15 W (ID 0, default) and **7 W (ID 1)**; Super
   configuration: 15 W, 25 W (default), MAXN_SUPER. "MAXN_SUPER … available only when flashing with
   jetson-orin-nano-devkit-super". The module enforces its budget with the on-module INA3221 (channel 1 = VDD_IN) through
   hardware throttling, and the average and instantaneous limits are settable (`curr1_max`, `curr1_crit`).

## 3. Derating and Part Rules

EEE-INST-002 (as tailored by SQSO-HAR-001 §5.2) with the stricter of it and the common 50 % rule, as on the EPS and OBC
boards: ceramic capacitors ≤ 50 % voltage; resistors ≤ 50 % power; microcircuit supplies ≤ 80 % of the operating maximum;
connectors and inductors ≤ 50 % current; junctions ≤ 110 °C. Worst-case input for voltage stress is **18.4 V**, the EPS
eFuse overvoltage cut-off (EPS design.py: 1.43 MΩ / 100 kΩ), not the 16.4 V charge limit. Parts are AEC-Q100/Q101/Q200
where offered; the exceptions are U2 (TPS25947, industrial −40…125 °C), U5 (SN74LVC1G74, no -Q1 grade, −40…125 °C), D1
(TVS2200, −40…125 °C) and the bought items. Matte-tin terminations (QFN-HR converters, passives, connectors) take the
EEE_PARTS_LIST §2A.3 mitigation: Sn63Pb37 assembly, re-tin where possible, conformal coat (Arathane 5750) over all
terminations.

## 4. Architecture

```
 W5A ── J1 ─ D1 TVS ─ C_F1 ─ L1 4.7 µH ─┬─ VBAT_F ─┬─ U1 LM61440 (1.000 MHz) ─ L2 ─ 5V_BUCK ─ U2 TPS259474L ─ VDD_IN ─ J4 SO-DIMM
 (VBAT_JET)                     R_D+C_D ┘          │    5.02 V                     OVLO 5.38 V, 2.51 A    (10 pins)
                                                   │                               breaker, PG ──┐
                                                   ├─ U3 LMR43620 (1 MHz, 180°) ─ L3 ─ 3V3_PRE ──┼─ U7 TPS22918 ─ 3V3_IO ─┬─ U9 ─ 1V8_IO
                                                   │                                    │        │   (ON = SYS_RESET*)    ├─ JET_3V3 (W5B-4)
                                                   └─ U4 SENSE (10.49 V)                ├─ Y1 1.000 MHz ─ SYNC (U1), U12 ─ SYNC_N (U3)
                                                                                        ├─ U8 TPS22918 ─ 3V3_SSD ─ J5 M.2 NVMe
  POWER_EN (J4-237) ── 10 k to VDD_IN; pulled low by U2 PG, U4 RESET ◄── U4 MR ◄── U5 latch ◄── U6 ◄── SHUTDOWN_REQ* (J4-233)
  UART1 (J4-203/205, 1.8 V) ── U10/U11 SN74LVC1T45 (VCCA 1V8_IO, VCCB 3V3_IO) ── 33 Ω ── W5B-3/-2 ── OBC U29/U28
  Spreader NTC RT1 ── J3 ── J2-5/-6 (passed through, never powered by the carrier) ── OBC
```

## 5. Input Protection and Filter

| Element | Part | Value | Why |
|---------|------|-------|-----|
| Connector J1 | Molex 0430450412 (mates EPS J18 over W5A) | 2 contacts per pole, 13 % of rating at 1.30 A | as fixed by W5 |
| D1 | TI TVS2200DRVR | 22 V standoff, 24.6 V breakdown minimum, 28 V clamp at 40 A | sits above the 18.4 V EPS OVP; clamps below the 36 V converter rating; ESD and hot-plug at integration |
| C_F1 | 4.7 µF/50 V X7R + 100 nF | at J1 | LISN-side shunt of the filter |
| L1 | Coilcraft XGL4030-472MEC | 4.7 µH, Isat ≈ 4 A | 32 % of Isat at the 1.30 A eFuse maximum |
| C_F2 | 3 × 4.7 µF/50 V + 3 × 100 nF at U1 and U3 | ≈ 9 µF effective at 14 V | converter input |
| Damper | 1.0 Ω (0805) + 2 × 4.7 µF/50 V | | Middlebrook: peak filter output impedance 1.7 Ω against the converter's 16 Ω input at 11.2 V and 8 W: **19.6 dB** |

No reverse-polarity element: the harness is keyed and the EPS eFuse limits any fault. Inrush: the EPS TPS16630 ramps
VBAT_JET at I_dVdT × gain / C = 2 µA × 25 / 10 nF = 5 V/ms, so the 21 µF of effective input capacitance draws
**0.11 A** (10× below the 1.10 A minimum trip). The EPS ramp allows up to 176 µF at 80 % of the trip; this is the limit
on any future input capacitance.

## 6. VDD_IN Supply

**U1, LM61440AANQRJRRQ1** (3–36 V, 4 A, AEC-Q100 grade 1, auto mode, no spread spectrum). Feedback 100 kΩ / 24.9 kΩ (0.1 %):
**5.016 V (4.958–5.074 V** with the ±1 % reference). RT 13.0 kΩ: 1.000 MHz free-running (Eq. 2); synchronised to Y1
through 1 nF on EN/SYNC (data sheet §9.3.2: the clock must be off at start-up, so Y1's OE is delayed 26 ms). EN divider
698 kΩ / 100 kΩ: on 10.08 V, off 7.26 V (below the 11.2 V BMS cut: rule E8). Duty 0.27–0.45; shortest on-time 273 ns at
18.4 V (minimum 70 ns). L2 3.3 µH (Eq. 9 gives 2.9 µH at K = 0.3 of 4 A; Eq. 10 2.5 µH); ripple 1.06 A p-p at 16.4 V;
Isat ≥ 8.1 A because the data sheet requires the inductor not to saturate at the high-side limit. RBOOT 100 Ω slows the
switch edge to 2.7 ns (EMI). Output 4 × 10 µF/**25 V**, so that a failed-short high side (18.4 V on 5V_BUCK) does not
also short the bus capacitors (the EPS eFuse then trips on U2's OVLO-protected load only).

**U2, TPS259474LRPWR** (adjustable OVLO, circuit breaker, PG, latch-off):

| Function | Setting | Result |
|----------|---------|--------|
| Overvoltage lock-out (34.8 k / 10 k on 5V_BUCK) | 1.20 V threshold | **5.38 V (5.30–5.48 V), 1.2 µs**: below the 5.5 V absolute maximum; 0.13 V above the highest normal VDD_IN |
| Circuit breaker (RILM 1.33 kΩ, RILM = 3334 / I) | 2.51 A (2.26–2.76 A) | 1.6× the 1.41 A average of the 7 W mode; latched until VBAT is removed |
| Transient blanking (ITIMER 2.2 nF) | 1.85 ms (1.13–3.19 ms) | load steps of the module pass; fast trip at 2 × ILIM (5 A) in 500 ns |
| UVLO (28.7 k / 10 k on 5V_BUCK) | on 4.64 V, off 4.22 V | |
| Power good (PGTH 28.7 k / 10 k on VDD_IN) | good above 4.64 V, bad below 4.22 V | releases POWER_EN |
| Slew (dVdt 2.2 nF, C = 2000 pF / SR) | 0.91 V/ms, 5.5 ms ramp | 0.19 A into 106 µF on the carrier plus 100 µF assumed in the module |

VDD_IN at the module (`jetson_calcs.py` table 3): static **4.85–5.07 V** at 2 A (U2 45 mΩ maximum, 10 connector contacts,
plane), **4.75–5.17 V** including a ±100 mV load-step excursion: inside 4.75–5.25 V, at the lower edge only at 2 A
with the reference at −1 %. VDD_IN bulk: 6 × 22 µF/16 V X7R at the SO-DIMM power pins.

## 7. Power Sequencing, Power-Good and Reset

### 7.1 POWER_EN

POWER_EN (J4-237) is pulled up to VDD_IN by 10 kΩ (4.55 V high against the module's 100 kΩ) and held low by either of two
open-drain outputs: **U2 PG** (VDD_IN below 4.64 V or U2 not fully on) and **U4 TPS3808G01-Q1 RESET** (VBAT_F below
10.49 V (10.32–10.66 V), for 19.4 ms after it rises (CT 3.3 nF), or MR low). U4 runs from 3V3_PRE and detects an input
collapse in 20 µs, so POWER_EN falls before VDD_IN does.

### 7.2 Shutdown request latch

SHUTDOWN_REQ* (J4-233) → 10 kΩ / 20 kΩ divider (5 V open-drain level to ≤ 3.5 V) → **U6 SN74LVC1G07-Q1**, powered from
3V3_IO → PRE* of **U5 SN74LVC1G74** → Q* → U4 MR → POWER_EN low. U5 is cleared only at power-up (100 kΩ / 100 nF on
CLR*, 12 ms). Because U6 is powered by 3V3_IO, which exists only while SYS_RESET* is high, a SHUTDOWN_REQ* level before
the module has finished its own sequence cannot latch the carrier off; with VCC = 0 its output is high-impedance (Ioff).
After a software `poweroff`, a thermal shutdown or an under-voltage event the module stays off, drawing only the
carrier's housekeeping, until the OBC removes VBAT. The latch state is on test point SD_LATCHED and the debug header.

### 7.3 Power-up sequence (`jetson_calcs.py` table 9)

| t | Event |
|---|-------|
| 0 | OBC raises JETSON_PWR (PA2) and PAYLOAD_EN (PB2); EPS U19 ramps VBAT_JET at 5 V/ms |
| 2.0 ms | VBAT_F reaches 10.1 V: U1 and U3 enabled (soft start 3.5–7 ms) |
| ≈ 9 ms | 5V_BUCK and 3V3_PRE in regulation; U2 ramps VDD_IN at 0.91 V/ms |
| ≈ 15 ms | VDD_IN above 4.64 V: U2 PG released |
| 23.4 ms | U4 RESET released: **POWER_EN high** |
| ≈ 26–28 ms | Y1 enabled; U1/U3 lock to 1.000 MHz after 2048 cycles |
| + ≈ 80 ms | module power sequence complete, SYS_RESET* high (80 ms is NVIDIA's forum guidance for POWER_EN to SYS_RESET*; the DG governs and is not retrieved) |
| + 1–2 ms | U7 and U8 on: 3V3_IO, 1V8_IO, 3V3_SSD; **JET_3V3 high → OBC JET_PG (PG6)** |
| + 20–45 s (to be measured, V-JET-3) | UEFI from QSPI, kernel and rootfs from NVMe, application; first STATUS line |

### 7.4 Power-down

**Commanded (normal).** OBC sends `SHUTDOWN` → payload runs `poweroff` → SHUTDOWN_REQ* low → U5 latches → POWER_EN low →
module asserts SYS_RESET* → U7/U8 off with quick output discharge (3V3_IO below 0.5 V in 0.09 ms, 1V8_IO in 0.19 ms:
DS limits 1.5 and 4 ms) → **JET_3V3 low → OBC JET_PG falls** → OBC drives JETSON_PWR then PAYLOAD_EN low. This
reverses ICD-002 1.2 §9 ("no SHUTDOWN message is sent"); the flight-software change is for the chair (§19).

**Uncommanded (guard trip, eFuse trip, OBC reset, load shed).** U4 sees VBAT_F fall below 10.49 V and drops POWER_EN
within 20 µs; the input capacitance then holds the converters for only **0.11 ms** at 7 W (0.84 mJ between 10.49 V and
U1's drop-out). That is not a controlled power-down. NVIDIA's own screening survives 4,000 hard power cuts (DS
Table 5-5), and the payload software keeps a read-only root and a journalled data partition (README), so the design
accepts it as the fault-case behaviour; V-JET-5 proves it on the EM. A 5 ms hold-up would need about 380 µF of
derated bulk capacitance and its own inrush limiter (the EPS ramp allows 176 µF): not worth it for the fault case.

### 7.5 Power-good to the OBC (RID AV-06, carrier side)

JET_3V3 is high only when, in series: both EPS switches are on (VBAT_JET present), U1 and U3 run, U2 is on and VDD_IN is
good, U4 has released POWER_EN, the module has finished its power sequence (SYS_RESET* high) and U7 is on. It falls
within 0.1 ms of any of these failing, including a U2 breaker trip, a U3 current-limit hiccup and a module shutdown.
The OBC reads it on PG6 (EXTI6) next to its two analogue readbacks (VSW1_RB, VJET_RB).

## 8. Carrier Rails

| Rail | Source | Loads | Current | Notes |
|------|--------|-------|---------|-------|
| 5V_BUCK | U1 | U2 | 1.42 A at 7 W | |
| VDD_IN | U2 | module | 1.41 A average, breaker 2.51 A | |
| 3V3_PRE | U3 LMR43620MC3RPERQ1 (3.27–3.33 V, 2 A, sync 0.2–2.5 MHz; free-runs at 2.2 MHz until the clock arrives) | U7, U8, Y1, U4, U5, U12 | 0.61 A peak (limit 2.8 A min: 4.6×) | always on with the carrier |
| 3V3_IO | U7 TPS22918-Q1, ON = SYS_RESET* (VIH 1.0 V: driven directly by the module's 1.8 V signal) | U9, U10/U11 VCCB, U6, JET_3V3 | < 5 mA | QOD tied to VOUT (≤ 35 Ω) |
| 1V8_IO | U9 TLV73318P-Q1 from 3V3_IO | U10/U11 VCCA | < 1 mA | 120 Ω active pull-down |
| 3V3_SSD | U8 TPS22918-Q1, ON = SYS_RESET* | M.2 NVMe | 0.61 A peak allocated | on before PERST# is released by UEFI; QOD |
| JET_3V3 | 3V3_IO through 100 Ω | OBC U28/U29 VCCB, JET_PG 10 k / 100 k | ≈ 50 µA (≤ 1 mA, E9) | a harness short is limited to 33 mA |

## 9. UART Link and Back-Power Isolation (RID AV-09, carrier side)

Module UART1 (J4-203 TXD, J4-205 RXD; `/dev/ttyTHS*` per the device tree) is translated by **U10** (A → B, DIR high) and
**U11** (B → A, DIR low), SN74LVC1T45QDCKRQ1, VCCA = 1V8_IO, VCCB = 3V3_IO, 33 Ω series on the harness side (as on the
OBC), 100 kΩ pull-up on the receive input, TPD4E05U06-Q1 at J2. RTS/CTS unused (no flow control, ICD §2.3).

Isolation: with the carrier unpowered, the OBC's U28/U29 have VCCB = JET_3V3 = 0 V (high-impedance by VCC isolation), and
U10/U11 have both supplies at 0 V (Ioff); the module's pins see nothing above 0.5 V before SYS_RESET* rises, because every
carrier-driven line into the module (U11's output, U6's input) is powered from a SYS_RESET*-switched rail. POWER_EN is the
one exception and is the module's power control input by definition. V-OBC-9 (carrier 3.3 V < 0.3 V with USART1 active)
stays the test.

Framing: the OBC transmits **8N2** (DS §2.10.4); the module's transmitter may send 8N1, which the OBC's receiver accepts in
2-stop mode. ICD-002 1.3 §2.3 records it; `firmware/src/uart.rs` must set it (for the chair).

## 10. Storage

| Store | Device | Contents | Why |
|-------|--------|----------|-----|
| Boot firmware | QSPI-NOR on the module | MB1/MB2, BCTs, UEFI (flashed with `l4t_initrd_flash.sh`) | the only on-module storage (DS §4.1) |
| Root file system, models, data | **Swissbit N2600, SN2600MA040GI-1TB2-2DB-STD**, M.2 2230 (S4) NVMe, 40 GB, 3D TLC in pSLC mode, 3.3 V only, −40…85 °C ambient (95 °C case), power-fail data protection, PCIe Gen3 x4 that runs x1 (fact sheet rev 1.04) | rootfs A and B (read-only, dm-verity), model partition (SmolLM2-360M or Qwen3-0.6B INT4, ≈ 0.25–0.45 GB, two copies with checksums), journalled data partition | the production module has no eMMC or SD (JLG); NVMe on PCIe C4 is NVIDIA's documented boot path; pSLC for endurance and charge-loss margin under TID; a 2230 card keeps a single 3.3 V rail (the 16 × 20 mm BGA alternative, Swissbit E2600, needs 3.3, 1.8 and 0.9 V rails and an NDA data sheet) |

Link: PCIe C4 (module pins 131/133/134/136, reference clock 160/162, PERST* 181, CLKREQ* 180, WAKE* 179) to M.2 lane 0;
lanes 1–3 open; link limited to **Gen2 x1** in the device tree (`max-link-speed = <2>`, `num-lanes = <1>`): 500 MB/s is
ample for boot and model load, and it lowers the PHY power and the spectral content near L1. Host-transmit AC-coupling
220 nF 0402 (C57/C58) per the PCIe convention; if the DG states the module carries them, they become 0 Ω (V-JET-2).
Flashing: JLG notes a 64 GB minimum for SD and USB root devices; the partition layout for a 40 GB NVMe is set with
`l4t_initrd_flash.sh` and a custom `flash_l4t_t234_nvme.xml` (V-JET-3 proves it). Retention: M2 standoff and screw,
thread-locked; card staked to the standoff (NASA-STD-8739.1).

## 11. Module Interfaces Used

| Interface | Module pins | Carrier | Flight |
|-----------|-------------|---------|--------|
| VDD_IN, GND | 251–260; 66 GND pins | §6 | yes |
| POWER_EN, SYS_RESET*, SHUTDOWN_REQ* | 237, 239, 233 | §7 | yes |
| UART1 | 203, 205 | OBC link (§9) | yes |
| PCIe C4 lane 0, reference clock, sideband | 131–136, 160/162, 179–181 | NVMe (§10) | yes |
| UART2 (combined debug console) | 236, 238 | J6 via 100 Ω, 1.8 V | GSE only |
| USB0 (recovery and flashing; the only port with device mode) | 109, 111 | J6, ESD | GSE only |
| FORCE_RECOVERY* | 214 | J6 (module pull-up assumed; DG to confirm, V-JET-2) | open |
| MODULE_ID, MOD_SLEEP* | 217, 178 | test points | — |
| Everything else (CSI, DP/HDMI, USB1/2, USB 3.2, GbE, CAN, I2S, SPI, I2C, PCIe 1–3, GPIO, PMIC_BBAT, CLK_32K_OUT, SLEEP/WAKE*) | | open | — |

No carrier EEPROM: MB2 BCT `cvb_eeprom_read_size = <0x0>` (JLG "EEPROM modifications"). No RTC cell (the OBC sends time
in TELEMETRY). Debug header J6 (Samtec FTSH-105-01-L-DV-K): 1 USB0_D+, 2 USB0_D−, 3/6/9 GND, 4 console TXD, 5 console RXD,
7 FORCE_RECOVERY*, 8 SYS_RESET* (1 kΩ, observe), 10 shutdown latch (1 kΩ, observe). Unmated and covered in flight; the
console RXD is left to the pinmux pull-up, and magic SysRq is disabled in the kernel so line noise cannot act.

## 12. Telemetry for the OBC's Independent Trips, and Latch-up Coverage

**Current.** The OBC's guard (`payload_guard.rs`: > 8 W for two samples, ≥ 12 W once) needs the rail current from a source
independent of the Jetson. The EPS already provides it: U19's IMON (0.84 V/A) on the EPS controller's ADC, read by the
OBC over I2C2, with VJET_RB as the voltage. The carrier adds no current monitor the OBC could read (no free OBC ADC
channel, and a carrier-powered monitor would vanish with the fault). For the chair: the sensor `JPC` of
`sensors.yaml` must be bound to that EPS register in flight.

**Temperature.** The guard's spreader limit (70 °C) and power-on gate (−15 °C, SRS-F131) need the spreader temperature
**before the carrier is powered**, so the sensor cannot be powered by the carrier. RT1 (10 kΩ NTC, TE GA10K3MCD1 (curve in §12.1), bonded
to the spreader with Stycast 2850FT) has flying leads to J3 and is passed through to **J2 pins 5 and 6** untouched (ESD
only). The OBC biases it (10.0 kΩ 0.1 % from its 3.3 V reference) and reads it on an ADC channel. No OBC or EPS channel is
free in the current pin allocation (ICX-001 §4.3, OBC-BRD ADS7138 AIN0–AIN7, EPS ADS7138): interface change **E10**
(§19). The Jetson-reported junction temperature (STATUS) remains a second input.

### 12.1 RT1 conversion for the OBC (version D)

RT1 is TE GA10K3MCD1: 10 kΩ at 25 °C, β25/85 3976 K, ±0.2 °C interchangeable from 0 to 70 °C, rated −40 to +125 °C (TE
data sheet as listed; its full R–T table could not be retrieved). The table uses the published **10K3A curve** of the
same 3976 K material, Steinhart–Hart 1/T = A + B·ln R + C·(ln R)³ with **A = 1.129241 × 10⁻³, B = 2.341077 × 10⁻⁴,
C = 0.877547 × 10⁻⁷** (T in K, R in Ω; Newport 710 mount manual addendum, "10K3A thermistor"); the curve gives 10,000 Ω at
25 °C and β25/85 = 3977 K, matching the data sheet. The OBC's reading (PA2) with the 10.0 kΩ 0.1 % bias from its ADC
reference and RT1 to PT_RTN is V/Vref = R / (R + 10 kΩ); sensitivity at the guard points: 24 counts/K at −15 °C and
18 counts/K at 70 °C (12 bit). `jetson_calcs.py` table 16 lists 1 °C-interpolable values; the flight software converts
with the Steinhart–Hart equation above, not a table.

| T (°C) | R (Ω) | V/Vref |
|-------:|------:|-------:|
| -40 | 336,050 | 0.9711 |
| -35 | 242,397 | 0.9604 |
| -30 | 176,785 | 0.9465 |
| -25 | 130,295 | 0.9287 |
| -20 | 96,999 | 0.9065 |
| -15 | 72,906 | 0.8794 |
| -10 | 55,301 | 0.8469 |
| -5 | 42,315 | 0.8089 |
| 0 | 32,650 | 0.7655 |
| 5 | 25,394 | 0.7175 |
| 10 | 19,903 | 0.6656 |
| 15 | 15,714 | 0.6111 |
| 20 | 12,493 | 0.5554 |
| 25 | 10,000 | 0.5000 |
| 30 | 8,056 | 0.4462 |
| 35 | 6,530 | 0.3951 |
| 40 | 5,325 | 0.3475 |
| 45 | 4,367 | 0.3040 |
| 50 | 3,601 | 0.2648 |
| 55 | 2,985 | 0.2299 |
| 60 | 2,487 | 0.1992 |
| 65 | 2,082 | 0.1723 |
| 70 | 1,752 | 0.1491 |
| 75 | 1,480 | 0.1289 |
| 80 | 1,256 | 0.1116 |
| 85 | 1,070 | 0.0967 |
| 90 | 916 | 0.0839 |
| 95 | 787 | 0.0729 |
| 100 | 678 | 0.0635 |
| 105 | 587 | 0.0555 |
| 110 | 510 | 0.0485 |
| 115 | 444 | 0.0425 |
| 120 | 388 | 0.0374 |
| 125 | 341 | 0.0329 |

**Latch-up coverage** (`jetson_calcs.py` table 10):

| Fault | Caught by | Time | Result |
|-------|-----------|------|--------|
| Module or VDD_IN load above 2.26–2.76 A | U2 circuit breaker, latched | ≤ 3.2 ms | JET_PG falls |
| Carrier input 8 W to 12 W | OBC guard on EPS IMON_JET × BUSV | 2 s / 1 s | both lines low |
| 3.3 V side above 2.8 A | U3 current limit (hiccup) | µs | 3V3_PRE collapses, JET_PG falls |
| Anything above 1.10–1.30 A at VBAT (14.7–17.4 W) | EPS U19, latched | ≤ 1 ms | OBC re-enables |
| Less than ≈ 0.5 W above the capped 7.5 W burst | not detected | — | ends with the burst (≤ 10 min); accepted for an expendable payload (RAD-V5 tests the trips) |

## 13. Burst Power Budget (BASELINE §6: ≤ 8 W including the carrier)

Losses per element (`jetson_calcs.py` table 5): U1 modelled from the data-sheet efficiency points (93 % at 2.1 MHz and
4 A, 95 % at 400 kHz) with RDS(on), DCR and switching terms; U2 I²R; the 3.3 V side at 80 % (FPWM at light load) with the
SSD allocated 0.30 W average (2.0 W peak during boot reads; Swissbit publishes no figures, V-JET-4 measures them).

| Module setting | Carrier input | Margin to 8 W |
|----------------|---------------|---------------|
| 7 W mode at its full budget, modelled, 11.2 / 14.4 / 16.4 V | 7.83 / 7.72 / 7.87 W | +2.1 / +3.5 / +1.7 % |
| 7 W mode, U1 at the data-sheet bound (93 %) | 8.03 W | **−0.4 %** |
| **6.5 W cap**, modelled | 7.19–7.32 W | +8.5 to +10.1 % |
| 6.5 W cap, U1 at 93 % | 7.48 W | +6.5 % |

**Verdict.** The 7 W mode is a budget the module throttles to, not a ceiling on instantaneous power; at its full value the
carrier input sits at 8 W with no margin, and the guard (> 8 W twice in 2 s) would trip legitimate bursts. **Cap the
module at 6.5 W** by setting the VDD_IN average limit (`curr1_max` = 1300 mA on the module's INA3221 channel 1, JLG) at
boot, and keep the 7 W nvpmodel clock set (ID 1) as the default mode. Measured on the EM (V-JET-4); the cap is software
on the Jetson, so the OBC's guard remains the independent limit.

Energy per burst (boot allocated at 30 s × 4.5 W): 30 s boot + 150 s generation = 1.2 kJ (0.34 Wh); with 10 lines,
121 J per line against the 360 J gate (BASELINE §6, D5). A full 10-minute power-on is 4.2 kJ (1.18 Wh), 424 J per line
with 10 lines: the energy gate, not the 10-minute limit, sets the burst length (≥ 12 accepted lines per 10 minutes).

## 14. Thermal Interface (THM-10 Design Input)

**Geometry.** The Orin Nano module has no thermal transfer plate (unlike AGX Orin); the heat sink contacts the SoC lid and
the memory and regulator parts directly (TDG; the NVIDIA STEP model gives the component heights). The spreader
(80 × 60 × 5 mm Al, MECHANICAL_DESIGN §4.1, §6.10) carries a machined pedestal over the SoC lid and a second, lower face
over the LPDDR5 and the module regulators.

| Item | Specification |
|------|---------------|
| SoC pad | 1.0 mm gap pad, ≥ 3.5 W/m·K, compressive stress ≤ 70 kPa at 35 % deflection, ASTM E595 TML ≤ 1.0 %, CVCM ≤ 0.10 % (candidate Bergquist GAP PAD 3500ULM, to be confirmed in the NASA outgassing database) |
| Other pads | 1.5–2.0 mm, softer (≤ 35 kPa at 35 %), over the DRAM and regulators |
| Deflection | nominal 25 % (0.25 mm) by the pedestal height; the stack tolerance ±0.1 mm gives 15–35 %; shim the spreader flanges at integration to the measured gap |
| Flatness | pedestal ≤ 0.05 mm over its face, parallel to the flange plane within 0.05 mm; Ra ≤ 1.6 µm |
| Clamp force | ≤ 28 N (SoC pad) + ≤ 10 N (others) = **38 N, 48 % of the 80 N (8.2 kgf) module limit** (DS Table 5-2), reacted by the module standoffs MP2/MP3, the SO-DIMM and the carrier's centre support at (0, −21) |
| Junction-to-spreader | θjc 0.30 K/W (allocation: lidded FCBGA; TDG value not retrieved) + pad 0.69–0.76 K/W over a 4.0 cm² lid (area allocation) + spreading 0.10 K/W = **1.09–1.16 K/W** (0.88 K/W with a 0.5 mm pad if flatness allows) |
| Against THM-10 | allowable (95 − 39 − 10) / 8 = 5.7 K/W (THA §6.3): margin **4.9×**; Tj at a 39 °C spreader and 7 W ≈ 47 °C |

The spreader-to-frame path (≥ 1 W/K, gasket and 3 × M2.5 per flange) is the mechanical design's (§6.10, 1.10 W/K
estimated). Carrier dissipation at 7 W: **0.72 W** (U1 0.28, 3.3 V side 0.38 including the SSD 0.30, U2 0.06, filter
0.01 W), into the stack columns through the board copper. Junctions at a 60 °C board: U1 70 °C, U3 69 °C, U2 65 °C,
U7/U8 63 °C (table 13; limit 110 °C). The SSD case limit is 95 °C; it sits on the carrier, not under the spreader.

## 15. EMC

**CE102 (V-EMC-1).** Differential-mode estimate at the LISN (`jetson_calcs.py` table 7): U1's input-current harmonics
(fundamental 0.73–0.88 A peak) across C_F2, divided by L1 into C_F1 ∥ 50 Ω: 21.6–23.2 dBµV at 1 MHz against the 60 dBµV
MIL-STD-461G basic curve (no relaxation below 28 V; GSFC-STD-7000B Table 2.5-2 tailoring): **≥ 37 dB margin**; higher
harmonics ≥ 68 dB. Common-mode emission depends on layout (ground plane continuity, the isolated mounting holes) and is
what the test measures.

**Frequency plan (rule E1).** Y1 (Epson SG-8018CA, 1.000 MHz ± 50 ppm) clocks U1 directly and U3 inverted (180°: halves
the input ripple). Harmonics sit at n × 1.000 MHz ± 22 kHz, the EPS comb: **200 kHz clear of both uplink windows**
(435.222–435.778 and 436.222–436.500 MHz). No spread spectrum (AAN and MC variants). Fallbacks: before lock (the first
≈ 28 ms of each power-on) and if Y1 fails, U1 free-runs at 0.90–1.10 MHz and U3 at 2.1–2.3 MHz, and their combs are
uncontrolled (as the EPS's are); V-EMC-3/-4 check the locked case, and a Y1 failure is a fault the receiver test covers.

**The module itself** is the large source (S1, V-EMC-1/-2): its 38.4 MHz reference (DS §2.10.10) has its 41st harmonic at
1574.4 MHz, **1.02 MHz below L1**, at the edge of the C/A main lobe (**EMC-04 confirmed**); CLK_M 19.2 MHz and the
PCIe 100 MHz reference are ≥ 5 MHz from the uplink windows; the 7 W mode's GPU clock (408 MHz) and LPDDR5 clocks are
broadband sources. The carrier does what it can: no module clock is routed on the carrier (CLK_32K_OUT and GPIO clock
outputs open); PCIe and USB are inner-layer striplines between solid ground planes, short (≤ 40 mm), PCIe at Gen2 x1 with
spread-spectrum reference clock enabled in the device tree if supported; the spreader (bonded to the frame) covers the
module on the RF-board side. L1 is protected operationally: GNSS sessions run with the Jetson off (BASELINE §5 table).

**Layout rules for the converters**: shielded inductors (XGL); hot loops (VIN–PGND pairs of U1, both sides of its
symmetric pinout) minimal on L1 with an unbroken L2 ground; switch nodes small and away from J2 and the module edge; input
filter at J1 with the damper beside it; a board-level shield frame over U1/L2/U3/L3 is reserved at layout (to be fitted if
V-EMC-1 needs it).

## 16. Radiation and Parts

The carrier is powered ≤ 10 % of the time (unbiased otherwise). TID: 5.25 krad(Si) in two years behind 2 mm Al
(RADIATION_ANALYSIS C); the carrier parts are bulk-CMOS/BiCMOS automotive parts with no radiation data; the module is
"non-critical and expendable" (BASELINE §6) and so is its carrier. SEL: every rail is current-limited and latched or
tripped (§12). The SSD controller's SEFI shows as a boot or I/O failure, which the OBC's burst timeout ends with a power
cycle. NAND in pSLC mode keeps a larger charge margin against TID-induced loss than TLC.

## 17. Part Stress (`jetson_calcs.py` table 12)

All within the rules: ceramics ≤ 37 % (VBAT side at 18.4 V on 50 V), 33 % (VDD_IN on 16 V); converter inputs 51 % of the
36 V operating rating at 18.4 V; logic supplies ≤ 61 %; U4 RESET at 74 % of 7 V; inductors ≤ 41 % of Isat; J4 VDD_IN
contacts 45 % at the highest steady current below the U2 trip (2.26 A / 10 × 0.5 A); J1 13 %. U2 reaches 80 % of its 23 V
operating rating only if U1 fails short (fault survival, not a normal stress). D1's standoff carries 18.4 V of 22 V (a TVS
is rated by standoff, not derated).

## 18. Layout Input

| Item | Rule |
|------|------|
| Outline and mounting | PC/104 90.17 × 95.89 mm; four Ø3.18 mm holes at (±40, ±43) and the centre supports at (0, ±21), all with Ø6.4 mm (Ø7 mm at the supports) copper-free rings, **isolated from GND** (the single frame bond is on the EPS board, EPS-BRD §17) |
| Module | SO-DIMM J4 (TE 2309407-2, 4.0 mm stack) on the −Y edge, module centred at (0, −6): module footprint x ±34.8, y −28.5 … +16.5; **no top-side parts inside it** (MECHANICAL_DESIGN §11 item 5); standoffs MP2/MP3 at (±31.3, +12.6) (MECHANICAL_DESIGN §4.5, to be checked against the NVIDIA drawing), MP2 1.85 mm from the window. Since version C the window starts 0.5 mm beyond the module's +Y edge. The (0, −21) support lies under the module: flush or countersunk fastener on the top side; check the module's underside parts against the NVIDIA STEP model (for the mechanical designer) |
| Zones (top side, versions B/C) | **Z1, +Y strip right of the window** (x −3.5 … +45, y +17.5 … +47.9): M.2 J5 and the card (connector x +1.5 … +10, card to x ≈ +36, y +25 … +47; 4.3 mm from the (0, +21) support, 5.7 mm from the (+40, +43) column), MP1; below the card (y +19.5 … +22.5) U7, U6, U9, U8 and the 3V3_SSD bulk; at the +X edge J2 (y ≈ +28.5, beside the harness gap at y 24–34), J3 (y ≈ +37), D2 (y ≈ +24.5), U10/U11 (y ≈ +19.5). **Z2, −X column** (x −45.1 … −37, y −35 … +30, outside the module footprint and the frame keep-out): U2 and the VDD_IN bulk at the bottom (if the VDD_IN pins are at the −X end), J6 debug, U3/L3, Y1, U12, U4, U5. **Z3, −Y strip** (y −47.9 … −35, x −36.8 … +3; the +3 limit came from the version-B magnetometer keep-out, now withdrawn, and is kept): J1 (near the −Y harness gap), D1, C_F1, L1, damper, U1, L2, U1 output caps. Centres and checks: `jetson_calcs.py` table 14 (0 conflicts). Routing between the zones goes on inner layers under the module footprint (the band beyond the window, y +44.5 … +47.9, is too narrow next to the frame screws H12/H13); nothing crosses the window |
| Bottom side | parts ≤ 1.0 mm only (the OBC is 1.4 mm below); none inside the frame keep-out (x −36.5 … −3.5, y +14.0 … +47.0) or within 20 mm of (+33, −38) |
| Top-side heights (version C) | ≤ 7.7 mm under the spreader (x −40 … +40, y −33 … +27), ≤ 13.2 mm beyond it (MECHANICAL_DESIGN §4.5). BOM heights: inductors 3.1 mm, 1210 capacitors 2.5 mm, ICs ≤ 1.45 mm, Y1 1.5 mm, M.2 card plane plus card about 5.5 mm, J6 4.7 mm (unmated in flight): all inside. Allocations to confirm on the Molex drawings: J2 mated ≤ 7.7 mm (it overlaps the spreader's +Y edge; if taller, move it to y ≥ +27.5 and J3 below the (+40, +43) column), J1 mated ≤ 13.2 mm in the −Y strip beyond the spreader (if taller, the right-angle Micro-Fit 2×2 header of the same series) |
| VDD_IN | L4 plane over the L5 ground (a paired path) from U2 to J4 pins 251–260, ≥ 10 mm wide. If pins 251–260 are at the −X end: U2 and 6 × 22 µF in Z2 at the pins. If they are at the +X end: U2 and the bulk stay at x ≤ +13 on the −Y strip (20 mm outside the magnetometer circle) and the plane pair runs to the pins; see §18.2 |
| Stack-up | 6 layers, 1.6 mm, FR-4 Tg ≥ 170 °C (polyimide preferred for TVAC), 1 oz: L1 parts/signals, L2 GND (solid), L3 PCIe/USB striplines, L4 power (VDD_IN, 3V3 islands, VBAT_F trace), L5 GND (solid), L6 parts ≤ 1 mm; IPC-6012 Class 3/A |
| Controlled impedance | PCIe TX/RX and REFCLK 85 Ω differential ±10 %, intra-pair skew ≤ 0.15 mm, ≤ 40 mm, ground-referenced on L2/L5, AC caps within 10 mm of J5; USB0 90 Ω differential to J6 (DG values to be confirmed, V-JET-2) |
| Thermal | U1 and U2 thermal pads with ≥ 9 × 0.3 mm vias to L2/L5; ≥ 4 cm² copper around U1; direct (no thermal relief) connections on VDD_IN and VBAT_F |
| Creepage | ≤ 18.4 V on the board: ≥ 0.2 mm everywhere (IPC-2221B B4 for conformal-coated external conductors) |
| Test points | VBAT_F, 5V_BUCK, VDD_IN, 3V3_PRE, 3V3_IO, 3V3_SSD, 1V8_IO, POWER_EN, SYS_RESET*, SHUTDOWN_REQ*, SD_LATCHED, U2_ILM (current monitor, 1.33 kΩ), SYNC_CLK, MODULE_ID, MOD_SLEEP*, GND × 2 |
| Coating and staking | Arathane 5750 conformal coat except J4/J5 contact areas and the pad faces; stake L1–L3, Y1, J1, J2, J3, the M.2 card end and the module far edge |

### 18.1 Gyro window (MECHANICAL_DESIGN §4.3, D-11; version B)

| Item | Rule |
|------|------|
| Cut-out | 27 × 27 mm, x −33.5 … −6.5, **y +17.0 … +44.0** (version C: moved +3 mm by the mechanical design to clear MP2; gyro at (−20, +30.5)), routed |
| Frame | Al 6061, 2.5 mm wide × 3.0 mm high, on the bottom face round the window (z +15.0 … +18.0), bonded and held by four M2 A286 button-head screws from the top into local tapped pads on the frame (the 2.5 mm bar is too thin for a tapped M2; positions set by the mechanical designer, version D amended): isolated holes **H10 (−35.5, +24.0), H11 (−35.5, +36.0), H12 (−4.5, +30.0), H13 (−4.5, +42.0)**; Ø4.5 mm top-side keep-out at each head; the bottom-face keep-out grows locally to x −37.8 (H10, H11) and x −2.2 (H12, H13) for ±3 mm in y at each screw. The head keep-outs clear every keep-out, MP2, the placed parts and the edge by ≥ 0.75 mm (`jetson_calcs.py` table 14 note; the U12/U4/U5 group in Z2 sits at x −44.5 … −38.5) |
| Keep-out | bottom side and L6: frame footprint + 0.5 mm, x −36.5 … −3.5, y +14.0 … +47.0 (no copper, no parts); top side and inner layers: window + 0.5 mm, x −34.0 … −6.0, y +16.5 … +44.5 (no parts, no copper, no routing; planes cut back). MP2's top-side pad sits over the frame band, which the frame does not touch. The window is 1.5 mm clear of the gyro on every side; the gyro top (z +20.6) stands 1.0 mm above the carrier top face |
| Assembly notes (MP2 over the frame band) | (1) reflow MP2 (and the rest of the top side) before the frame is bonded; (2) the frame band's bottom face is bare laminate or solder mask only, with any via under MP2 tented or filled; (3) no frame screw under MP2 (nearest, H10, is 7.4 mm away). Accepted by the mechanical designer on these conditions |
| What it displaced | Version A put the whole power train in this area (+Y, −X): moved to Z2 and Z3. J2 moved from the −X edge to the +X edge (both harness gaps at y 24–34 are reachable; W5B length unchanged at 0.10 m, to be confirmed by the harness design) |
| Clear of it | SO-DIMM (y ≤ −27), module (since version C the window starts 0.5 mm beyond the module's +Y edge), M.2 connector and card (x ≥ +1.5: 5.0 mm), converters and connectors (table 14), centre support (0, +21) (3.0 mm, per the mechanical design) |

### 18.2 Magnetometer (version D: keep-out withdrawn)

The OBC designer moved the RM3100 off the OBC to board A2 at (+24, +27) in the +Z end-frame slot (their item E11), 63 mm
from the Jetson input current; the 20 mm / 30 mm keep-out round (+33, −38) that version B applied to the carrier is
**withdrawn**. The parts stay as placed in §18 (zones Z1–Z3): that placement still has no inductor, steel standoff or
oscillator near the old corner, and nothing in it depended on the keep-out except the choice of zones.

| Item | Status |
|------|--------|
| Ferromagnetic parts | L1–L3, Y1 (metal lid), the steel standoffs MP1–MP3 and the SO-DIMM latch hardware are ordinary magnetic-cleanliness items for the ADCS residual-dipole budget, no longer a local keep-out; a non-ferromagnetic SO-DIMM latch is preferred, not required |
| Current paths | VBAT_IN and VDD_IN stay plane pairs (L4 over L5) as good practice |
| Which SO-DIMM end carries VDD_IN (pins 251–260) | **no longer a blocker**: a field-magnitude check. At 63 mm the module's 1.4 A input through its connector pins (return about 5 mm away) gives B ≈ µ0·I·d / (2π r²) ≈ 0.35 µT as a line-pair upper bound, and the carrier's plane pairs a few nT; the actual dipole of the module and carrier currents is measured in V-JET-9 and, if needed, calibrated out against IMON_JET by the ADCS |

## 19. Interface Changes (for the chair)

| # | Owner | Change | Reason |
|---|-------|--------|--------|
| E9 | carrier (this board) | **Done**: JET_3V3 on J2-4, ≤ 1 mA, = 3V3_IO switched by SYS_RESET* | AV-06, AV-09 |
| E10 | **taken by the OBC designer**: OBC J3 6-pin, pin 5 thermistor, pin 6 return, ADC on PA2; INTERCONNECT W5B, harness | W5B becomes Pico-Clasp 6 (5013310607): pins 5 PT_NTC, 6 PT_RTN; OBC biases RT1 (10.0 kΩ 0.1 % from its 3.3 V reference) and reads it on one ADC channel (none free today: the OBC designer allocates, e.g. a third ADS7138 input freed or a second ADS7138); `sensors.yaml` PT bound to it | spreader temperature for the guard and the −15 °C power-on gate, readable with the carrier off |
| F1 | flight software (`firmware/src/uart.rs`) | USART1 transmit 2 stop bits (8N2) | DS §2.10.4 |
| F2 | flight software (`eps_control.rs`, payload schedule) | end each burst with `SHUTDOWN`, wait for JET_PG to fall (≤ 5 s, then cut anyway), then JETSON_PWR low and PAYLOAD_EN 10 ms later; treat an unexpected JET_PG fall with the rail commanded on as a payload fault and cut both lines | §7.4; reverses ICD-002 1.2 §9 |
| F3 | flight software (`sensors.yaml`, EPS driver) | bind JPC to the EPS controller's IMON_JET register (U19 IMON, 0.84 V/A) | §12 |
| F4 | payload software (`software/payload`) | at boot: `curr1_max` = 1300 mA on INA3221 channel 1 (VDD_IN 6.5 W cap); nvpmodel default ID 1 (7 W) in `/etc/nvpmodel.conf`; device tree PCIe C4 x1 Gen2; carrier board config derived from `jetson-orin-nano-devkit` (not `-super`), no carrier EEPROM | §2, §10, §13; TB-13 |
| M1 | mechanical | spreader pedestal over the SoC and second face over DRAM/regulators; flatness and shim per §14; module standoff positions and the (0, −21) fastener under the module; window frame screw positions H10–H13 as assumed in §18.1 | §14, §18 |
| M2 | OBC, ADCS | closed by OBC item E11 (RM3100 moved to (+24, +27)); remaining: V-JET-9 measures the burst-induced field at the new location; the SO-DIMM VDD_IN end is a field-magnitude check only | §18.2 |

## 20. Verification Items

| ID | Test | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-JET-1 | Power-up and power-down sequence on the EM: scope VBAT_IN, 5V_BUCK, VDD_IN, POWER_EN, SYS_RESET*, 3V3_IO, 1V8_IO, JET_3V3, at 11.2 / 14.4 / 16.4 V and −20 / +50 °C | order of §7.3; POWER_EN only with VDD_IN ≥ 4.75 V; no module I/O > 0.5 V while SYS_RESET* low; 3V3_IO < 0.5 V within 1.5 ms and 1V8_IO within 4 ms of SYS_RESET* low; DG timing met | DS §3.6 | carrier EM |
| V-JET-2 | Design-guide conformance review against DG-10931-001 (current revision) and the NVIDIA pinmux table | PCIe AC-cap location, FORCE_RECOVERY* pull-up, routing rules, SO-DIMM part, sequence timing all agree or changed | DG | before layout release |
| V-JET-3 | Flash and boot from the 40 GB NVMe with the custom partition layout; boot time | boots to the first STATUS line in ≤ 45 s, 100 consecutive boots | §10 | EM |
| V-JET-4 | Carrier input power during bursts (boot, generation, idle) at 7 W mode with and without the 6.5 W cap; SSD power | ≤ 8.0 W, 2 s average, every sample at the 6.5 W cap; record SSD peak and average | BASELINE §6 | EM |
| V-JET-5 | 1,000 uncommanded power cuts at the carrier input during boot and generation, plus 1,000 commanded shutdowns | every following boot succeeds; rootfs verity intact; data partition clean | §7.4 | EM (long run) |
| V-JET-6 | Fault injection: VDD_IN overload 3 A, 3.3 V overload 3 A, 5V_BUCK raised to 5.6 V (OVLO) | U2 breaks within 3.2 ms and latches; OVLO cuts within 2 µs; U3 limits; JET_PG falls; the OBC cuts both lines | §6, §12; RAD-V5 | EM |
| V-JET-7 | Junction-to-spreader resistance at 7 W (module sensors vs RT1) and clamp force (pressure film) | ≤ 5.7 K/W (design 1.2); force ≤ 40 N | THM-10 (V-THM-3) | EM thermal |
| V-JET-8 | Random vibration of the populated carrier (module, card, spreader) at protoflight level | no resonance shift > 5 %, functional after, no fretting at J4/J5 | DS Table 5-5 is 2 Grms only | EM vibration |
| V-EMC-1 | CE102 at the carrier input; near-field scan | as SQSO-EMC-001 §6 | §15 | carrier EM |
| V-JET-9 | Magnetic screening: populated carrier with the module, unpowered and at a 7 W burst, field at the RM3100 position (A2, (+24, +27) in the +Z end-frame slot) | static and burst-induced field below the limit the ADCS owner sets from the RM3100 noise (15–30 nT); the nT/A coefficient against IMON_JET recorded for calibration | §18.2 | EM, with the OBC |

## 21. RID Status

| RID | Status | Evidence |
|-----|--------|----------|
| SE-01 (carrier part) | **Closed for the carrier at schematic level**: parts selected, netlist, BOM, ERC (0 errors, waivers listed), calculations; layout and EM open | this document; `design.py`, `jetson_carrier.net`, `jetson_carrier_bom.csv`, `jetson_carrier_erc.log`, `jetson_calcs.py` |
| AV-06 (carrier side) | **Closed in hardware on the carrier**: JET_3V3 is a power-good of the whole carrier and module chain (§7.5); the flight software's two state bits and readback comparison remain (F2) | §7.5; V-OBC-10, V-JET-1 |
| AV-09 (carrier side) | **Closed in hardware**: JET_3V3 provided (E9); every carrier line into the module powered from SYS_RESET*-switched rails; Ioff translators on both sides | §9; V-OBC-9, V-JET-1 |
| THM-10 | **Design input supplied**: 1.09–1.16 K/W junction-to-spreader by design (allocations for θjc and lid area), against 5.7 K/W; remains a verification item | §14; V-JET-7 (= V-THM-3) |
| EMC-04 | **Source confirmed** (38.4 MHz, DS §2.10.10; 41st harmonic 1574.4 MHz); not removable by the carrier; mitigated by operations (GNSS with the Jetson off); verification item V-EMC-1/-2 | §15 |
| TB-13 | **Design assumption stated**: original flash configuration, nvpmodel 7 W (ID 1) as the default, plus a 6.5 W VDD_IN cap; Super's modes are not used; 8 W is met only with the cap (or with no margin at the full 7 W budget) | §2 item 6, §13 |

## 22. Not Verified

- DG-10931-001 and TDG-11127-001 were not available: sequence timing (POWER_EN to SYS_RESET*), the PCIe TX AC-cap location,
  FORCE_RECOVERY* pull-up, routing rules, the recommended SO-DIMM part, the SoC lid area, θjc and component heights.
- MPNs not confirmed from a manufacturer or distributor page in this session: Coilcraft XGL4030-472MEC and XGL6030-332MEC
  (site 403), Epson SG-8018CA frequency code for 1.000 MHz, Murata GCM188R71E224KA55D, GCM155R71C224KE02D,
  GCM1885C1H222JA16D, GCM1885C1H332JA16D (series codes), TE 2309407-2 and 1-2199230-4 (distributor listings only),
  Würth standoffs, TE GA10K3MCD1, Bergquist GAP PAD 3500ULM. The TI parts were checked on ti.com product pages
  (orderable lists); the Swissbit N2600 on its fact sheet and a Mouser listing.
- Which end of the SO-DIMM carries VDD_IN (pins 251–260) and the latch material of TE 2309407-2 (§18.2: magnetic-cleanliness items only since version D).
- TE's own resistance-temperature table for GA10K3MCD1 (§12.1 uses the published 10K3A Steinhart–Hart constants of the same 3976 K curve).
- No power figures for the Swissbit N2600; the 0.30 W average and 2.0 W peak are allocations.
- The U1 efficiency model is fitted to two data-sheet points; the 93 % bound is shown alongside it.
- M.2 lane naming: host transmit is connected to the card's PERp0/PERn0 (pins 43/41) and the card's PETp0/PETn0 (49/47)
  to the module receive, per the PCI-SIG M.2 Socket 3 convention; confirm against the M.2 specification and the NVIDIA
  P3768 reference schematic at schematic review (V-JET-2).

## 23. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| A | 2026-10-27 | Payload carrier design (CDR design completion) | First issue: carrier schematic as code with parts (VBAT input filter, 5 V buck and VDD_IN eFuse, sequencing and shutdown latch, switched I/O rails, UART translators, NVMe storage, debug header), netlist, BOM, ERC, calculations, layout input, interface changes and verification items |
| B | 2026-10-27 | Payload carrier design (chair follow-up) | Gyro window and frame of MECHANICAL_DESIGN §4.3 (D-11): §18.1, frame screw holes H10–H13 and the frame MP5 in the design files; RM3100 magnetic keep-out of §4.4: §18.2; placement re-zoned (power to the −Y strip and −X column, I/O to the +X edge) with a placement check (`jetson_calcs.py` tables 14–15); M2, V-JET-9 |
| C | 2026-10-27 | Payload carrier design (chair follow-up) | Window moved to y +17.0 … +44.0 (MECHANICAL_DESIGN §4.5, item 12): keep-outs split by side, frame screws H10–H13 moved to the side bars, J3/D2/U10/U11 re-placed; top-side height limits (7.7 / 13.2 mm) added to the placement check; MP2/MP3 at (±31.3, +12.6); BASELINE cited as Rev E |
| D | 2026-10-27 | Payload carrier design (chair follow-up) | (amended) frame screws H10–H13 moved to the mechanical designer's positions with Ø4.5 mm head and local bottom keep-outs, MP2 assembly notes (§18.1), Z2 logic group nudged 0.5 mm; RM3100 moved to (+24, +27) by the OBC designer (E11): §18.2 keep-out withdrawn, parts kept as placed, SO-DIMM VDD_IN end now a field-magnitude check, M2 closed; E10 taken (OBC J3 6-pin, ADC on PA2); §12.1 RT1 resistance-temperature table (10K3A Steinhart–Hart curve) for the OBC's conversion; RT1 range corrected to −40 … +125 °C |
