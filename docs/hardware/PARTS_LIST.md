# SMART-QSO Parts List / Bill of Materials

**Document ID**: SQSO-BOM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the complete parts list and bill of materials for SMART-QSO, including part numbers, quantities, and specifications per NASA-STD-8739.4.

## 2. BOM Summary

### 2.1 Component Count

| Category | Count | Mass (g) |
|----------|-------|----------|
| Structural | 12 | 350 |
| Electrical | 87 | 180 |
| Mechanical | 24 | 45 |
| Fasteners | 48 | 25 |
| Harness | 1 set | 80 |
| **Total** | **172** | **680** |

### 2.2 Cost Summary

| Category | Estimated Cost |
|----------|----------------|
| Structure | $X,XXX |
| EPS | $X,XXX |
| OBC | $X,XXX |
| RF System | $X,XXX |
| AI Payload | $X,XXX |
| Miscellaneous | $XXX |
| **Total** | $XX,XXX |

## 3. Structure Subsystem

### 3.1 Primary Structure

| Item | P/N | Qty | Material | Mass (g) | Supplier |
|------|-----|-----|----------|----------|----------|
| Frame rails (set) | SQSO-STR-001 | 1 | Al 7075-T6 | 150 | Custom |
| End plates | SQSO-STR-002 | 2 | Al 6061-T6 | 60 | Custom |
| Side panels | SQSO-STR-003 | 4 | Al 6061-T6 | 80 | Custom |
| PCB standoffs | M3×10 | 8 | Al | 8 | McMaster |
| Rail shoes | SQSO-STR-004 | 4 | Al 7075 | 20 | Custom |

### 3.2 Deployment Mechanism

| Item | P/N | Qty | Material | Mass (g) | Supplier |
|------|-----|-----|----------|----------|----------|
| Antenna mount | SQSO-ANT-001 | 1 | Al 6061 | 15 | Custom |
| Deploy spring | SQSO-SPR-001 | 2 | Elgiloy | 5 | Custom |
| Hinge pin | SQSO-PIN-001 | 2 | Stainless | 3 | Custom |
| Burn wire | 0.1mm nichrome | 0.5m | Nichrome | 1 | Vendor |

### 3.3 Fasteners

| Item | P/N | Qty | Material | Notes |
|------|-----|-----|----------|-------|
| M3×6 SHCS | 92855A310 | 24 | A2 SS | Structure |
| M3×8 SHCS | 92855A312 | 16 | A2 SS | PCB mount |
| M2×4 SHCS | 92855A304 | 8 | A2 SS | Components |
| M3 locknut | 90576A102 | 24 | A2 SS | Securing |
| M3 washer | 93475A210 | 48 | A2 SS | Under heads |

## 4. Electrical Power Subsystem

### 4.1 Solar Cells

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Triple junction cells | TJ-SC-3G30C | 6 | 29%, 30mm×80mm | Spectrolab |
| Cover glass | CMG 100 | 6 | 100μm, AR coated | Spectrolab |
| Cell interconnect | Cu ribbon | 12 | 0.1mm×2mm | Custom |

### 4.2 Battery

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Li-ion cell | INR18650-MJ1 | 2 | 3500mAh, 3.7V | LG Chem |
| Battery holder | SQSO-BAT-001 | 1 | 2P config | Custom |
| Protection IC | BQ29700 | 1 | 2S protection | TI |
| NTC thermistor | NTCG164BH | 2 | 100kΩ | TDK |

### 4.3 Power Management

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| MPPT charger | SPV1050 | 1 | Up to 18V in | ST |
| Buck converter | TPS62840 | 2 | 3.3V, 1.8V | TI |
| Power switch | TPS22810 | 3 | Load switches | TI |
| Shunt resistors | Various | 4 | Current sense | Vishay |
| TVS diodes | SMBJ6.0CA | 4 | ESD protection | Littelfuse |

### 4.4 EPS PCB Components

| Item | P/N | Qty | Value | Package |
|------|-----|-----|-------|---------|
| Ceramic caps | GRM188 | 24 | Various | 0603 |
| Tantalum caps | T491B | 6 | 100μF | 3528 |
| Inductors | SRP6540 | 3 | Various | 6.5×4.0 |
| Resistors | Various | 32 | Various | 0402 |

## 5. On-Board Computer

### 5.1 Processor

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| MCU | AMA4B2KK-KBR | 1 | Apollo4 Blue Plus | Ambiq |
| Crystal | ABM8G-32.768 | 1 | 32.768kHz | Abracon |
| Crystal | ABM8-24.000 | 1 | 24MHz | Abracon |
| FRAM | FM25V10 | 1 | 1Mbit | Cypress |
| Flash | IS25LP256D | 1 | 256Mbit | ISSI |

### 5.2 OBC Support

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| LDO regulator | TLV75533 | 1 | 3.3V, 500mA | TI |
| I2C buffer | PCA9517 | 2 | Level shift | NXP |
| Reset supervisor | MAX6369 | 1 | Watchdog | Maxim |
| ESD protection | TPD4E001 | 4 | 4-channel | TI |

### 5.3 OBC PCB Components

| Item | P/N | Qty | Value | Package |
|------|-----|-----|-------|---------|
| Decoupling caps | GRM155 | 32 | 100nF | 0402 |
| Bulk caps | GRM188 | 8 | 10μF | 0603 |
| Resistors | Various | 28 | Various | 0402 |
| Test points | Various | 12 | - | Through-hole |

## 6. RF Subsystem

### 6.1 Radio

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Transceiver IC | Si4463 | 1 | VHF/UHF | SiLabs |
| PA (VHF) | RD00HVS1 | 1 | 2W, 145MHz | Mitsubishi |
| LNA (UHF) | BGA725L7 | 1 | 0.8dB NF | Infineon |
| TCXO | ASTX-H11 | 1 | 30MHz, ±0.5ppm | Abracon |
| SAW filter (VHF) | SF2049B | 1 | 145MHz | Murata |
| SAW filter (UHF) | SF2098B | 1 | 435MHz | Murata |

### 6.2 RF Components

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| RF switch | SKY13348 | 1 | SPDT | Skyworks |
| Diplexer | SQSO-DIP-001 | 1 | VHF/UHF | Custom |
| SMA connector | 142-0701-801 | 1 | Panel mount | Cinch |
| Matching components | Various | 16 | Various | Various |

### 6.3 Antenna

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Antenna element | SQSO-ANT-002 | 2 | Tape spring | Custom |
| Feed point | SQSO-ANT-003 | 1 | Balun | Custom |

## 7. AI Payload

### 7.1 Compute Module

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Jetson Orin Nano | 945-13767-0005 | 1 | 8GB | NVIDIA |
| Carrier board | SQSO-JET-001 | 1 | Custom minimal | Custom |
| Heat sink | SQSO-HS-001 | 1 | Passive | Custom |
| Power converter | PTN78020W | 1 | 5V, 6A | TI |

### 7.2 Payload Interface

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| UART isolator | ISO7721 | 1 | Galvanic | TI |
| Power switch | TPS22965 | 1 | 6A switch | TI |
| Connector | DF12-30DS | 1 | Board-to-board | Hirose |

## 8. Sensors

### 8.1 ADCS Sensors

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Magnetometer | LIS3MDL | 1 | 3-axis | ST |
| Sun sensor | SFH2430 | 6 | Photodiode | Osram |
| Gyroscope | BMG250 | 1 | 3-axis | Bosch |
| Accelerometer | ADXL345 | 1 | 3-axis | ADI |

### 8.2 Housekeeping Sensors

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Temperature sensor | TMP117 | 4 | ±0.1°C | TI |
| Current sensor | INA219 | 3 | Bidirectional | TI |
| Voltage reference | REF3033 | 1 | 3.3V | TI |

## 9. Connectors and Harness

### 9.1 Board Connectors

| Item | P/N | Qty | Type | Supplier |
|------|-----|-----|------|----------|
| PC/104 stackthrough | 4-1617543-0 | 4 | 104-pin | TE |
| Debug header | FTSH-110 | 2 | 10-pin ARM | Samtec |
| Battery connector | DF13-6P | 2 | 6-pin | Hirose |
| Solar connector | DF13-4P | 6 | 4-pin | Hirose |

### 9.2 Harness Materials

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Wire AWG28 | M22759/32-28 | 5m | PTFE | Various |
| Wire AWG24 | M22759/32-24 | 2m | PTFE (power) | Various |
| Kapton tape | 7419D | 1 roll | 1" width | 3M |
| Cable ties | TY23MX | 50 | Miniature | TE |
| Lacing cord | A-A-52084 | 5m | Waxed | Various |

## 10. Miscellaneous

### 10.1 Thermal Materials

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Thermal pad | TG-A6200 | 1 sheet | 0.5mm | T-Global |
| MLI blanket | SQSO-MLI-001 | As needed | Custom | Custom |
| Black paint | MAP-PUK | 100mL | Polyurethane | MAP |
| White paint | MAP-SG121FD | 100mL | White | MAP |

### 10.2 Conformal Coating

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Conformal coat | HumiSeal 1B73 | 250mL | Acrylic | Chase |
| Masking compound | CS25M | 100mL | For connectors | Chase |

### 10.3 Labels and Markings

| Item | P/N | Qty | Specs | Supplier |
|------|-----|-----|-------|----------|
| Kapton labels | Custom | As needed | Laser-printed | Brady |
| Serial number tag | SQSO-TAG-001 | 1 | Engraved Al | Custom |

## 11. Approved Vendor List

### 11.1 Preferred Vendors

| Category | Vendor | Products |
|----------|--------|----------|
| Semiconductors | DigiKey | General ICs |
| Semiconductors | Mouser | General ICs |
| RF Components | Mini-Circuits | RF passives |
| Connectors | Newark | Hirose, TE |
| Mechanical | McMaster-Carr | Fasteners |
| Space Components | Cobham | Space-grade |
| Solar Cells | Spectrolab | III-V cells |

### 11.2 Custom Parts

| Part | Vendor | Drawing |
|------|--------|---------|
| Structure | TBD | SQSO-DWG-001 |
| Antenna elements | TBD | SQSO-DWG-002 |
| Carrier board | TBD | SQSO-PCB-001 |
| Harness | In-house | SQSO-HAR-001 |

## 12. Parts Control

### 12.1 Screening Requirements

| Part Class | Screening |
|------------|-----------|
| Class S | N/A (not used) |
| Class B | Burn-in per datasheet |
| Industrial | Temperature cycling |
| Commercial | Lot sampling |

### 12.2 Prohibited Materials

| Material | Reason | Alternative |
|----------|--------|-------------|
| Pure tin | Whiskers | Sn/Pb or matte |
| Zinc | Corrosion | Nickel plate |
| Cadmium | Toxic | Alternatives |
| PVC | Outgassing | PTFE/FEP |

### 12.3 Traceability

| Item | Tracking |
|------|----------|
| Lot number | All ICs |
| Date code | All parts |
| Serial number | Assemblies |
| Inspection date | All parts |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
