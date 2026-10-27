# EEE Parts List
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MFG-002
**Revision:** C
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
| Board-level screening (replaces part burn-in) | Each flight board: functional test, ≥ 8 thermal cycles −30 to +70 °C powered at the extremes, then ≥ 200 h powered with the last 100 h failure-free (RELIABILITY_ANALYSIS v2.0 §6) |
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

Total dose: 5.3 krad(Si) in 2 years behind 2 mm Al at 500 km; design level 10 krad (`docs/environment/RADIATION_ANALYSIS.md`
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
| TI BQ76920 (class) | 4S BMS AFE | BATTERY_DESIGN §4.2 | Mission and safety | None | Candidate |
| Secondary over-voltage protector + three-terminal fuse | Dissimilar overcharge control | SQSO-BSA-001 H1 (EPS-11) | Safety | — | Not yet selected |
| TI SN74LVC1G123 | XPDR_EN heartbeat monostable | BUILD_VS_BUY §3.2 | Mission (transponder) | None | Candidate |
| TI SN74LVC1G11 | Transponder key gate | BUILD_VS_BUY §3.2 | Mission | None | Candidate |
| CML CMX138A | CTCSS decode and audio conditioning | BUILD_VS_BUY §3.2 | Mission | None | Candidate |
| TI TMUX1219 | Audio select | BUILD_VS_BUY §3.2 | Mission | None | Candidate |
| TI TPS22918 | Repeat-path load switch | BUILD_VS_BUY §3.2 | Non-critical | None | Candidate |
| TI TPS25947 class eFuse | Jetson rail | Schematic S7 | Safety for the bus (isolation) | None | Candidate |
| TI 74LVC1T45 | OBC–Jetson UART isolation (Ioff) | Schematic S7 (F13) | Mission | None | Candidate |
| Infineon FM25V20A | FRAM | RADIATION_ANALYSIS Rev C | Mission | None for the part | Candidate |
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
- §13.1: mission duration is the 2-year design life (BASELINE §2), and the dose is 5.3 krad(Si) behind 2 mm.

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
| Cell Type | Panasonic NCR18650GA; other documents name Samsung INR18650-35E (used in `hardware/eps/BATTERY_DESIGN.md`). PI to choose |
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
| Orbit altitude | 465–525 km (BASELINE §3; the ODAR Draft B recommends 465–485 km) |
| Inclination | ≈ 97.4°, sun-synchronous (`docs/BASELINE_PARAMETERS.md` §3) |
| Mission duration | 2 years design life (BASELINE §2) |
| Total Ionizing Dose | 5.3 krad(Si) in 2 years behind 2 mm Al at 500 km (RADIATION_ANALYSIS Rev C §4) |
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
