# EEE Parts List
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MFG-002
**Revision:** G
**Date:** 2026-10-27
**Reference:** NASA-STD-8739.10 (EEE Parts Assurance Standard, 2017), Level 4 commercial parts with project screening; GSFC EEE-INST-002 (2003, with Addendum 1, 2008) Level 3 as the screening guide; SMD SPD-39 (Class D mission assurance requirements, 2021) and *Guidance for Use of Commercial EEEE Parts in Class D Missions* v1.0 (2024-10-16); NASA/TM-20210018053 (Avionics Radiation Hardness Assurance Guidelines, 2021); SpaceX Rideshare Payload User's Guide v11 Table 5-5. MIL-STD-975 (cancelled) and "NASA Ames SmallSat Heritage" are withdrawn as references
**Status:** CDR draft. §2A is the parts assurance policy; the part tables (§4–§11) predate the build-versus-buy decisions and are corrected by §2A.6–§2A.7

---

## 1. Purpose

This document provides a comprehensive list of Electrical, Electronic, and Electromechanical (EEE) parts used in the SMART-QSO CubeSat, including part specifications, derating, and radiation tolerance.

## 2. Scope

This document covers:
- All EEE parts in flight configuration
- Part specifications and manufacturers
- Derating requirements
- Radiation tolerance data
- Approved alternatives

---

## 2A. CDR Parts Assurance Requirements

### 2A.1 Policy

SMART-QSO is a Class D payload (NPR 8705.4B). Parts are commercial (NASA-STD-8739.10 Level 4: manufacturer's standard
product, not independently verified) unless a space-grade part is cheap and available. Commercial parts are made
acceptable by the screening, traceability, radiation and finish controls below, applied in proportion to criticality.
Criticality comes from the hardware FMEA (`docs/safety/SYSTEM_FMEA_CIL.md`, SQSO-FMEA-002): **mission-critical** parts
are those of the bus chain and of every Category 1 or 2 item; the Jetson, GNSS and backup gyro are **non-critical**
(their failure is isolated by a load switch).

### 2A.2 Screening (mission-critical parts)

Adapted from EEE-INST-002 Level 3 for plastic-encapsulated microcircuits and discretes, at the board level where
part-level screening is impractical:

| Step | Requirement |
|------|-------------|
| Grade | Automotive (AEC-Q100/Q101/Q200) preferred, else industrial (−40 to +85 °C or wider) |
| Receiving inspection | Visual and marking check of 100 % of parts, dimension sample, certificate of conformance against the order |
| Counterfeit avoidance | Buy only from the manufacturer or its franchised distributors; any other source needs an SAE AS6081-class inspection and an X-ray of a sample |
| Lot control | One date/lot code per part type for the EM and FM where possible; record the lot of every part fitted (traveler) |
| Board-level screening (replaces part burn-in) | Each flight board: functional test, ≥ 8 thermal cycles −30 to +70 °C powered at the extremes, then ≥ 200 h powered with the last 100 h failure-free (RELIABILITY_ANALYSIS v2.0 §6) Screening cycles stop at −30 °C, inside the parts' −40 °C rating; the −40 °C first-orbit case is tested by the protoflight plateau (ETP §3.4, deviation 8), not by screening |
| Destructive physical analysis | One sample from each lot of the OBC MCU, FRAM, BMS AFE and RF PA, when a lot cannot be traced to a franchised source |
| Derating | §12 of this list, applied with the hot-case temperatures of the thermal analysis |
| Alerts | GIDEP and manufacturer errata checked at selection and before the FM build |

### 2A.3 Terminal finishes and tin whiskers (UG v11 Table 5-5; launch RID LV-26)

SpaceX prohibits **pure tin or tin electroplate** (unless alloyed with lead, antimony or bismuth), together with
cadmium, zinc plating and mercury; a containerized CubeSat is not exempt from this row (UG v11 §5.3). Most COTS parts
have matte-tin terminations. Requirement:

1. Record the terminal finish of every EEE part (column added to every part table in the FM issue of this list).
2. Prefer Sn-Pb, NiPdAu or gold finishes when the part is offered with them.
3. Pure-tin-finished parts: **hot-solder-dip re-tinning with Sn63Pb37** to the full termination (GEIA-STD-0006
   practice), or, where re-tinning risks the part (fine-pitch, moisture-sensitive or BGA parts, the Jetson module and
   its connector), **conformal coating** over the terminations as a whisker mitigation (GEIA-STD-0005-2 practice, as GSFC
   applies it), documented for the integrator as the rationale for an exception.
4. Assemble flight boards with Sn63Pb37 (J-STD-001 with the space addendum); no SAC305 lead-free solder on flight
   boards.
5. No tin-plated RF shields or connector contacts; no cadmium-plated or zinc-plated hardware
   (`docs/compliance/MATERIALS_LIST.md` Rev B §2.2).

Verification: VI-SAF-07 (SQSO-SDP-002), prohibited-materials and terminal-finish declaration at L−4 months.

### 2A.4 Radiation

Total dose: 4.69 krad(Si) in 2 years behind 2 mm Al at 475 km (4.91 at the 485 km cap; RADIATION_ANALYSIS F §4.1); design level 10 krad (`docs/environment/RADIATION_ANALYSIS.md`
Rev C §4). Parts are accepted for TID with published data at ≥ 10 krad, or a heritage report, or a lot sample test
(RAD-V3). Every mission-critical rail has a latch-up current limiter with OBC-commanded power cycling, because SEL data
are missing for most parts (NASA/TM-20210018053 approach for commercial parts).

### 2A.5 Parts control

A parts list owned by the S&MA lead; additions after the FM parts freeze go through the configuration-control board
(`docs/project/CONFIGURATION_MANAGEMENT_PLAN.md`). The Jetson module, the GNSS receiver and the bought assemblies
(AntS, magnetometer, panels) are controlled as assemblies by part number and lot, with the vendor's declaration of
finishes and materials.

### 2A.6 Parts added by the CDR design decisions (not yet in §4–§11)

| Part | Function | Source decision | Criticality | Radiation data | Status |
|------|----------|-----------------|-------------|----------------|--------|
| Epson M-G370PDT0 | Primary gyro | D9 | Mission (ADCS degraded if lost) | TID pass at 5, 10, 20 krad (Epson); no SEL data | Selected |
| Bosch BMI088 | Backup gyro | D9 | Non-critical while unpowered | None | Selected |
| TI BQ76952PFBR (was BQ76920 class) | 4S BMS AFE, −40 … +85 °C | BATTERY_DESIGN §4.2; EPS board BOM | Mission and safety | None | Selected (EPS U1) |
| TI BQ771806DPJR secondary over-voltage protector + Littelfuse ITV4030L1412NR three-terminal fuse (was 'not yet selected') | Dissimilar overcharge control | SQSO-BSA-001 H1 (EPS-11); EPS board BOM | Safety | — | Selected (EPS U2, F1, F3). **ITV4030 rated −10 … +65 °C**: below the −40 … +70 °C criterion, covered only by V-EPS-8 extended to −40 °C and +70 °C or a design change (§2A.9) |
| TI SN74LVC1G123 | XPDR_EN heartbeat monostable | BUILD_VS_BUY §3.2 | Mission (transponder) | None | Candidate |
| TI SN74LVC1G11 | Transponder key gate | BUILD_VS_BUY §3.2 | Mission | None | Candidate |
| CML CMX138A | CTCSS decode and audio conditioning | BUILD_VS_BUY §3.2 | Mission | None | Candidate |
| TI TMUX1219 | Audio select | BUILD_VS_BUY §3.2 | Mission | None | Candidate |
| TI TPS22918 | Repeat-path load switch | BUILD_VS_BUY §3.2 | Non-critical | None | Candidate |
| TI TPS25947 class eFuse | Jetson rail | Schematic S7 | Safety for the bus (isolation) | None | Candidate |
| TI 74LVC1T45 | OBC–Jetson UART isolation (Ioff) | Schematic S7 (F13) | Mission | None | Candidate |
| Infineon FM25V20A | FRAM | RADIATION_ANALYSIS Rev F | Mission | None for the part (RAD-V3, RAD-V4) | Selected (OBC U3, mirror U36) |
| SkyFox piNAV-NG/FM | GNSS receiver (assembly) | D6 | Non-critical | Vendor | Selected |
| ISIS AntS | Antenna assembly | BUILD_VS_BUY §2.6 | Mission | Flight heritage | Selected |
| Nvidia Jetson Orin Nano 8 GB (900-13767-0030-000) | AI payload (assembly) | BASELINE §6 | Non-critical | Orin AGX data only | Selected |

### 2A.7 Corrections to the tables below

- §4.1 STM32L4R5ZIT6: "TID ~30 krad (tested)" and "SEL threshold > 60 MeV·cm²/mg" have **no source**; RADIATION_ANALYSIS
  Rev C §4.3 found no lot data for the L4R5 (open: RAD-V3). Treat as unknown until then.
- §4.2 MSP430FR5969 backup MCU: not in the baseline architecture (BASELINE §6 has one OBC and a hardware watchdog);
  remove or justify.
- The TID and SEL values for the other parts in §4–§9 have no sources and are treated as unknown.
- §7 RF parts (CC1125, RF5110G) are superseded by the transponder build of BUILD_VS_BUY §3 and the RF review.
- §13.1: mission duration is the 2-year design life (BASELINE §2), and the dose is 4.69 krad(Si) behind 2 mm at 475 km (4.91 at 485 km).
- §10 Connectors (ESQ-126 PC/104 stack connector, FTSH-105 debug header, U.FL RF connector) is superseded: there is no
  stack connector, and the flight connectors and their finishes are §2A.8.

### 2A.8 Connector finishes (harness design SQSO-HAR-001 2.0 §3.1; §2A.3 item 5)

Every board header and every harness-side contact of the flight connectors is **gold** (tin-plated contacts are
prohibited by §2A.3 item 5). The board build-to designs' BOMs list the tin variants; their finish codes change as
below (part numbers recorded with the finish in the FM issue of this list).

| Family | Board side | Harness side | Gold finish | Used on |
|--------|-----------|--------------|-------------|---------|
| Molex Pico-Clasp 1.0 mm | 501331-xx vertical SMT header (the BOMs' 5013310207 … 5013311407 are the tin code "-xx07") | 501330-0N00 housing; 501334 crimp terminal (28–32 AWG) is the **tin** terminal; the gold terminal is **501193-9000** (socket, gold 0.76 µm, 28–32 AWG, tape and reel; Digi-Key 5011939000, Active) (Molex 501330 housing listing; SQSO-MFG-003 §9) | Molex's gold-plated versions (0.38 or 0.76 µm Au, per the Pico-Clasp datasheet 987651-6111): the gold finish code of each header and of the terminal replaces the tin code; 0.76 µm preferred | EPS J2, J4–J9, J12, J14–J17, J19–J21; OBC J3–J8, J10; RF J2; carrier J2, J3; magnetometer board A2 J1 |
| Molex Micro-Fit 3.0 | 43045 dual-row **right-angle** through-hole header: 43045-0811 (2×4, gold, solder tab); the 2×3 and 2×2 right-angle headers in the same gold finish (the BOMs' 0430450812, -0612, -0412 are vertical, tin) | 43025-0N00 receptacle; crimp terminal **43030-0003** (socket, gold, 20–24 AWG; Digi-Key 0430300003 'CONN SOCKET 20-24AWG CRIMP GOLD'; 0.762 µm per SQSO-HAR-001) | Selective gold over nickel | EPS J1, J10, J11, J13, J18; carrier J1 |
| Samtec Tiger Eye 1.27 mm (replaces FTSH + FFSD) | TFM-1xx-01-S-D-RA right-angle (OBC J1, J2, J9; RF J1), vertical TFM-1xx-01-S-D (EPS J3) | SFSDT discrete-wire socket assembly (PTFE wire) | Plating code to confirm with Samtec: catalog F-226 (Rev 20OCT26) gives TFM platings –F (gold flash) and –L (15 µin gold), "–S" being the single-row option; SFSDT platings –F and –G (10 µin gold). SFSDT (Teflon) only: −40 … +125 °C; the PVC SFSD (−10 … +80 °C) is not acceptable | W3 (40), W4A (16), W8A (10) |
| Samtec FTSH (unmated in flight) | FTSH-105-01-L-DV-K (EPS J22 FTSH-102, carrier J6) | — | "-L": 10 µin gold | Programming headers, GSE only |
| MMCX | Molex 73415-1473 straight jack (gold, brass body, BeCu contact, listed non-magnetic) | Molex 73414-0351 right-angle crimp plug for 1.37 mm cable (gold); W10E (GNSS, 1.13 mm cable): **Molex 73415-2261** right-angle plug for 1.13/1.32 mm cable, solder centre contact, crimp shield, gold over brass, −55 … +165 °C, Active (Digi-Key 0734152261) | Gold | RF J3–J6, W10, W10E |
| Micro-D | MIL-DTL-83513-class 21-way socket receptacle, board mount, on the access-port board A3 | (GSE plug, not flown) | Gold contacts per MIL-DTL-83513; shell passivated, not cadmium | Access port (W8A, W11) |
| Splices | — | **TE Raychem D-200-82** (TE P/N D17660-000) MiniSeal sealed in-line crimp splice, the M81824/1-1 size (26–20 AWG, conductor 0.15–0.75 mm²), meets SAE-AS81824/1 modified for 200 °C, −55 … +200 °C (TE D-200 data sheet; Bürklin listing). **Not M81824/1-1 (TE D-436-36):** its crimp splicer is copper alloy 101/102 per ASTM B75 **tin-plated per ASTM B545** (TE drawing D-436-36/-37/-38 Rev F, 2016-08-19), which this row prohibits. The D-200-82 is not a QPL M81824 number: the substitution of the AS81824/1 call-out is recorded as a harness-design deviation (harness designer). These splices have **no solder preform** (that is the SolderSleeve family): the seal is two meltable rings under the sleeve. **28 AWG** conductors (≈ 0.08 mm²) are below the smallest size, so the 28 AWG leads of W6C and W7C end on bonded polyimide solder tabs instead (HARNESS_DESIGN 2.0 amended, TB-RX/RY/RZ, TB-ANT).| Nickel per SAE AMS-QQ-N-290 over copper alloy 101/102 (D-200-82); sleeve radiation-crosslinked modified fluoropolymer, rings modified thermoplastic fluoro-elastomer (outgassing: SQSO-MAT-001 §7) | W7A/W7B sensor leads only (4 splices, 24 AWG) |

Wire: SAE AS22759/33 (silver-coated high-strength copper alloy) and the PTFE wire of the SFSDT assemblies: no tin.

### 2A.9 Temperature grades (parts temperature audit SQSO-MFG-003, RID P3T-01)

Every flight part is rated at least −40 … +70 °C operating and −40 … +70 °C non-operating (THERMAL_DESIGN §1.1 and
ETP §3.4: the allowables ± 10 °C, with the −40 °C first-orbit plateau tested without margin), shown from the data
sheet or the decoded ordering code line by line in `docs/manufacturing/PARTS_TEMPERATURE_AUDIT.md` and its CSV, except:

| Item | Rating | How it is covered | Owner, due |
|------|--------|-------------------|------------|
| EPS F1, F3 ITV4030L1412NR | −10 … +65 °C | V-EPS-8 extended to −40 °C and +70 °C on the flight lot, or a design change; no compliant part exists in the footprint | EPS designer, 2027-01-15 |
| RF Q601 RD01MUS3 | No published minimum; Tstg −40 … +125 °C | Mitsubishi statement of the minimum operating temperature; PA characterised at −40 °C on the RF EM | RF designer, statement 2026-12-15 |
| ISIS AntS | −20 … +60 °C qualified; storage −50 … +85 °C | ISIS statement on operation and survival at −40 °C; AntS node in the thermal analysis (SQSO-MFG-003 §12 V-1) | §12 V-1 |
| CubeSpace CR0002 rods | −20 … +80 °C (TV), cold start −35 °C (family data) | CubeSpace −40 °C rating or rod qualification by test (§12 V-2) | §12 V-2 |
| Panasonic NCR18650GA (alternative cell, §6.4) | Not recorded | Specification ranges recorded, or the alternative deleted (§12 V-3) | §12 V-3 |
| Body panels and wings | Vendor not selected | Qualified range ≥ −56 … +93 °C at selection (V-THM-4; §12 V-4) | §12 V-4 |
| Jetson module 900-13767-0030-000 | TJ −25 … +105 °C; TSTG −40 … +80 °C | Unit allowable: −15 °C power-on gate; unpowered to −40 °C by the EM soak V-THM-8 | Thermal analyst (V-THM-8) |
| Battery cells INR18650-35E | Charge 0 … +45 °C, discharge −10 … +60 °C | Battery allowable and heater (SYS-602) | — (by design) |

Capacitors are X7R or C0G only (no Y5V/Z5U, tantalum, electrolytic or polymer parts), and no part is battery-backed.
Receiving inspection rejects any substitute whose ordering code changes the temperature grade (the ABLS option letter,
the Epson temperature code, the TI suffix, the Micro Crystal TA/TB/TC code, the Swissbit "I", the Samtec SFSDT
cable type). Reel and packaging storage conditions (Murata, Panasonic +5 … +35/40 °C; Coilcraft tape −40 … +80 °C;
Littelfuse ITV 0 … +35 °C; NVIDIA module +18 … +30 °C) are stores controls of the receiving inspection, not flight
limits.

---

## 3. Parts Classification

### 3.1 Grade Levels

| Grade | Description | Application |
|-------|-------------|-------------|
| Level 1 | Space-qualified (Class S) | Critical functions |
| Level 2 | Hi-rel commercial | Primary functions |
| Level 3 | Industrial grade | Non-critical functions |
| COTS | Commercial off-the-shelf | Payload, non-flight-critical |

### 3.2 Part Categories

| Category | Count | Notes |
|----------|-------|-------|
| Microcontrollers | 2 | OBC, backup |
| Memory (Flash) | 2 | Code, data storage |
| Memory (SRAM) | 1 | Working memory |
| Power Regulators | 6 | Various rails |
| Battery | 1 | Li-ion pack |
| RF Components | 8 | Transceiver, PA, filters |
| Sensors | 12 | Magnetometer, sun, temp |
| Passives | ~150 | R, C, L |
| Connectors | 8 | PC104, debug |

---

## 4. Microcontrollers

### 4.1 Primary OBC

| Parameter | Value |
|-----------|-------|
| Part Number | STM32L4R5ZIT6 |
| Manufacturer | STMicroelectronics |
| Description | Arm Cortex-M4 MCU, 2MB Flash |
| Package | LQFP144 |
| Grade | Industrial (-40°C to +85°C) |
| Quantity | 1 |
| TID Tolerance | ~30 krad (tested) |
| SEL Threshold | >60 MeV-cm²/mg |
| Derating | 80% max clock |

### 4.2 Backup/Watchdog MCU

| Parameter | Value |
|-----------|-------|
| Part Number | MSP430FR5969 |
| Manufacturer | Texas Instruments |
| Description | Ultra-low-power MCU |
| Package | VQFN-48 |
| Grade | Industrial |
| Quantity | 1 |
| TID Tolerance | ~20 krad |
| Function | Watchdog, safe mode |

---

## 5. Memory Devices

### 5.1 Code Flash

| Parameter | Value |
|-----------|-------|
| Part Number | Internal to STM32L4R5 |
| Type | Embedded Flash |
| Capacity | 2 MB |
| TID Tolerance | ~30 krad |
| Notes | Triple-voted critical sections |

### 5.2 Data Storage Flash

| Parameter | Value |
|-----------|-------|
| Part Number | W25Q128JVSIQ |
| Manufacturer | Winbond |
| Description | 128Mbit SPI Flash |
| Package | SOIC-8 |
| Grade | Industrial |
| Quantity | 2 (redundant) |
| TID Tolerance | ~15 krad |
| Derating | EDAC enabled |

### 5.3 SRAM

| Parameter | Value |
|-----------|-------|
| Part Number | Internal to STM32L4R5 |
| Type | Embedded SRAM |
| Capacity | 640 KB |
| TID Tolerance | ~30 krad |
| Notes | ECC enabled on critical regions |

---

## 6. Power Management

### 6.1 Battery Management IC

| Parameter | Value |
|-----------|-------|
| Part Number | BQ25703A |
| Manufacturer | Texas Instruments |
| Description | Battery charger/manager |
| Package | QFN-32 |
| Grade | Industrial |
| Quantity | 1 |
| TID Tolerance | ~10 krad |

### 6.2 3.3V Regulator (Primary)

| Parameter | Value |
|-----------|-------|
| Part Number | TPS62840 |
| Manufacturer | Texas Instruments |
| Description | 750mA DC-DC converter |
| Package | VSON-6 |
| Grade | Industrial |
| Quantity | 2 (redundant) |
| Efficiency | >90% |
| Derating | 70% max load |

### 6.3 5V Regulator

| Parameter | Value |
|-----------|-------|
| Part Number | TPS63070 |
| Manufacturer | Texas Instruments |
| Description | 2A buck-boost converter |
| Package | VQFN-14 |
| Grade | Industrial |
| Quantity | 1 |
| Derating | 60% max load |

### 6.4 Battery Pack

| Parameter | Value |
|-----------|-------|
| Part Number | Custom 4S1P (BASELINE §5) |
| Cell Type | Panasonic NCR18650GA; other documents name Samsung INR18650-35E (used in `hardware/eps/BATTERY_DESIGN.md`). PI to choose The INR18650-35E is rated charge 0 … +45 °C, discharge −10 … +60 °C (Samsung specification 1.1); the NCR18650GA's ranges are not recorded (SQSO-MFG-003 §12 V-3) |
| Capacity | ≈ 3.5 Ah (one string) |
| Voltage | 14.4V nominal (≈ 50 Wh) |
| Quantity | 1 pack (4 cells) |
| Protection | OV, UV, OC, OT |

---

## 7. RF Components

### 7.1 Transceiver IC

| Parameter | Value |
|-----------|-------|
| Part Number | CC1125 |
| Manufacturer | Texas Instruments |
| Description | Sub-1GHz RF transceiver |
| Package | QFN-32 |
| Grade | Industrial |
| Quantity | 1 |
| Frequency | 145/435 MHz |
| TID Tolerance | ~15 krad |

### 7.2 Power Amplifier

| Parameter | Value |
|-----------|-------|
| Part Number | RF5110G |
| Manufacturer | Qorvo |
| Description | 3W linear PA |
| Package | QFN-16 |
| Grade | Industrial |
| Quantity | 1 |
| Output Power | 0.5-2W (configurable) |

### 7.3 LNA

| Parameter | Value |
|-----------|-------|
| Part Number | SKY67151-396LF |
| Manufacturer | Skyworks |
| Description | Low-noise amplifier |
| Package | QFN-6 |
| Grade | Industrial |
| Quantity | 1 |
| NF | 0.6 dB |

### 7.4 SAW Filters

| Parameter | Value |
|-----------|-------|
| Part Number | SF2049E (145 MHz), SF2098E (435 MHz) |
| Manufacturer | Murata |
| Description | SAW bandpass filters |
| Package | SMD |
| Quantity | 2 each |

---

## 8. Sensors

### 8.1 Magnetometer

| Parameter | Value |
|-----------|-------|
| Part Number | LIS3MDL |
| Manufacturer | STMicroelectronics |
| Description | 3-axis magnetometer |
| Package | LGA-12 |
| Grade | Industrial |
| Quantity | 2 (redundant) |
| Range | ±16 gauss |
| Interface | I2C |

### 8.2 Sun Sensors (Photodiodes)

| Parameter | Value |
|-----------|-------|
| Part Number | TEMD6200FX01 |
| Manufacturer | Vishay |
| Description | Silicon photodiode |
| Package | 0805 SMD |
| Grade | Industrial |
| Quantity | 6 (one per face) |
| Spectral Response | 440-1050 nm |

### 8.3 Temperature Sensors

| Parameter | Value |
|-----------|-------|
| Part Number | TMP117 |
| Manufacturer | Texas Instruments |
| Description | ±0.1°C digital temp sensor |
| Package | WSON-6 |
| Grade | Industrial |
| Quantity | 6 |
| Interface | I2C |
| Range | -55°C to +150°C |

### 8.4 Current Sensors

| Parameter | Value |
|-----------|-------|
| Part Number | INA226 |
| Manufacturer | Texas Instruments |
| Description | Current/power monitor |
| Package | VSSOP-10 |
| Grade | Industrial |
| Quantity | 4 |
| Interface | I2C |

---

## 9. Actuators

### 9.1 Magnetorquer Driver

| Parameter | Value |
|-----------|-------|
| Part Number | DRV8837 |
| Manufacturer | Texas Instruments |
| Description | H-bridge motor driver |
| Package | WSON-8 |
| Grade | Industrial |
| Quantity | 3 (X, Y, Z axes) |
| Current | 1.8A peak |

### 9.2 Antenna Deployment

| Parameter | Value |
|-----------|-------|
| Part Number | Nichrome wire |
| Type | Burn wire |
| Resistance | 2 ohm |
| Quantity | 2 (redundant) |
| Activation | 1A for 5 seconds |

---

## 10. Connectors

### 10.1 PC/104 Stack Connector

| Parameter | Value |
|-----------|-------|
| Part Number | ESQ-126-39-G-D |
| Manufacturer | Samtec |
| Description | 104-pin stack connector |
| Quantity | 2 per board |

### 10.2 Debug Connector

| Parameter | Value |
|-----------|-------|
| Part Number | FTSH-105-01-L-DV-K |
| Manufacturer | Samtec |
| Description | 10-pin SWD header |
| Quantity | 1 |

### 10.3 RF Connector

| Parameter | Value |
|-----------|-------|
| Part Number | U.FL-R-SMT |
| Manufacturer | Hirose |
| Description | Micro coax connector |
| Quantity | 2 |

---

## 11. Passive Components

### 11.1 Resistors

| Type | Value Range | Package | Tolerance | Quantity |
|------|-------------|---------|-----------|----------|
| Thick film | 10Ω - 1MΩ | 0402, 0603 | 1% | ~80 |
| Thin film | 1kΩ - 100kΩ | 0603 | 0.1% | ~10 |

### 11.2 Capacitors

| Type | Value Range | Package | Voltage | Quantity |
|------|-------------|---------|---------|----------|
| MLCC (C0G) | 10pF - 100nF | 0402, 0603 | 25V | ~40 |
| MLCC (X7R) | 100nF - 10µF | 0603, 0805 | 16V | ~30 |
| Tantalum | 10µF - 100µF | 1206, 1210 | 10V | ~10 |

### 11.3 Inductors

| Type | Value Range | Package | DCR | Quantity |
|------|-------------|---------|-----|----------|
| Chip inductor | 1µH - 100µH | 0603, 0805 | <1Ω | ~15 |
| Power inductor | 2.2µH - 22µH | Shielded | <50mΩ | ~5 |

---

## 12. Derating Requirements

### 12.1 Derating Table

| Component | Parameter | Derating |
|-----------|-----------|----------|
| Resistors | Power | 50% |
| Capacitors (MLCC) | Voltage | 50% |
| Capacitors (Tantalum) | Voltage | 60% |
| Transistors | Power | 50% |
| Voltage regulators | Current | 70% |
| ICs | Junction temp | 80% of max |

### 12.2 Thermal Derating

| Condition | Additional Derating |
|-----------|---------------------|
| Hot case (+60°C) | +10% |
| Vacuum operation | Power components +20% |
| Radiation environment | Per part analysis |

---

## 13. Radiation Considerations

### 13.1 Mission Environment

| Parameter | Value |
|-----------|-------|
| Orbit altitude | 465–485 km, target 475 km (BASELINE Rev E §3); radiation analysed at 500 and 525 km as bounds |
| Inclination | ≈ 97.4°, sun-synchronous (`docs/BASELINE_PARAMETERS.md` §3) |
| Mission duration | 2 years design life (BASELINE §2) |
| Total Ionizing Dose | 4.69 krad(Si) in 2 years behind 2 mm Al at 475 km, 4.91 at the 485 km cap (RADIATION_ANALYSIS Rev F §4.1) |
| SEE flux | Low (LEO) |

### 13.2 Mitigation Strategy

| Component | Mitigation |
|-----------|------------|
| MCU | Triple voting, watchdog |
| Memory | EDAC, scrubbing |
| Analog | Redundancy, filtering |
| Power | Current limiting, OVP |

---

## 14. Approved Alternates

### 14.1 Alternate Parts

| Primary | Alternate | Notes |
|---------|-----------|-------|
| STM32L4R5 | STM32L4S9 | Pin compatible |
| TPS62840 | TPS62842 | Same family |
| LIS3MDL | LSM303AGR | Different pinout |
| CC1125 | CC1120 | Lower data rate |

---

## 15. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| A.1 | 2026-10-16 | §6.4 battery pack configuration aligned with BASELINE §5 (PI-14) | Funding audit |
| B | 2026-10-16 | §13.1 inclination TBD replaced by the baseline value, ≈ 97.4° sun-synchronous (`docs/BASELINE_PARAMETERS.md` §3; finding VV-17) | SMART-QSO Team |
| C | 2026-10-27 | CDR S&MA review: §2A parts assurance policy (NASA-STD-8739.10 Level 4 with EEE-INST-002 Level 3 screening adapted for Class D; counterfeit avoidance; lot control; board-level screening); tin-whisker and prohibited-finish requirements from UG v11 Table 5-5 (launch RID LV-26); radiation acceptance; parts added by D6, D9 and the CDR designs; unsourced radiation claims withdrawn; §13.1 updated; MIL-STD-975 reference withdrawn | CDR S&MA review |
| D | 2026-10-27 | CDR 2026 consistency pass: orbit altitude row from BASELINE Rev E §3 | CDR documentation review |
| E | 2026-10-27 | §2A.8 connector finishes: gold variants of every flight connector (Pico-Clasp, Micro-Fit right-angle, Samtec TFM/SFSDT replacing FTSH/FFSD, MMCX, Micro-D, splices); §10 marked superseded (harness design SQSO-HAR-001 2.0 §15 item P-1) | Harness design |
| F | 2026-10-27 | Parts temperature audit (RID P3T-01, SQSO-MFG-003 A): new §2A.9 temperature grades, exceptions with their coverage, owners and dates; §2A.6 AFE, secondary protector and fuse rows as selected on the EPS board (ITV4030 −10 … +65 °C); §2A.8 Pico-Clasp gold terminal 501193 (501334 is tin), Samtec TFM/SFSDT plating codes to confirm, SFSDT Teflon only; §2A.2 screening range noted; §6.4 cell ranges | EEE parts engineer, CDR round 3 |
| G | 2026-10-27 | §2A.8 harness-side part numbers (SQSO-HAR-001 §15 P-3): Pico-Clasp terminal 501193-9000, Micro-Fit terminal 43030-0003, W10E plug Molex 73415-2261; splice D-436-36 (M81824/1-1) rejected for its tin-plated barrel (TE drawing Rev F, ASTM B545), TE Raychem D-200-82 (nickel) instead, with the AS81824/1 deviation and the 28 AWG range open to the harness designer | EEE parts engineer, CDR round 3 |
