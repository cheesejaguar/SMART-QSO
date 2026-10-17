# Electrical Power Subsystem (EPS) Design Document

**Document Number**: SMART-QSO-EPS-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary Design

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- GSFC-STD-7000 (GEVS) - General Environmental Verification Standard
- ECSS-E-ST-20C - Space Engineering: Electrical and Electronic

**Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This document defines the detailed design of the Electrical Power Subsystem (EPS) for the SMART-QSO 1U CubeSat. It provides requirements, architecture, component specifications, and verification approach.

### 1.2 Scope

The EPS encompasses:
- Solar array power generation
- Battery energy storage
- Power management and distribution
- Load control and protection
- Power telemetry and monitoring

### 1.3 Reference Documents

| Document | Number | Description |
|----------|--------|-------------|
| Software Requirements Specification | SMART-QSO-SRS-001 | Flight software requirements |
| OBC-EPS Interface | SMART-QSO-ICD-001 | Interface control document |
| Systems Engineering Mgmt Plan | SMART-QSO-SEMP-001 | SE approach |

---

## 2. EPS Requirements

### 2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| EPS-F001 | Generate the BASELINE §5 EOL orbit-average power: 2.39 W tumbling, 3.58 W sun-pointed (2.14 W tumbling with self-shadowing; `hardware/eps/SOLAR_ARRAY_DESIGN.md` §9.3) | Analysis, Test |
| EPS-F002 | Deliver, EOL, at least 3.4 W sunlit average tumbling and 5.4 W sun-pointed within 20° (`hardware/eps/SOLAR_ARRAY_DESIGN.md` §9.3) | Analysis, Test |
| EPS-F003 | Store ≈ 50 Wh (4S1P 18650, BASELINE §5); usable energy per eclipse limited by the DoD limit in EPS-P004 | Test |
| EPS-F004 | Provide regulated 3.3V rail (OBC, sensors) | Test |
| EPS-F005 | Provide regulated 5V rail (Jetson payload) | Test |
| EPS-F006 | Provide regulated 12V rail (RF PA) | Test |
| EPS-F007 | Implement MPPT with >95% efficiency | Test |
| EPS-F008 | Support software-controlled load shedding | Test |
| EPS-F009 | Provide battery state-of-charge telemetry | Test |
| EPS-F010 | Protect battery from over/under voltage | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| EPS-P001 | MPPT efficiency | >95% | >97% |
| EPS-P002 | Power distribution efficiency | >90% | >92% |
| EPS-P003 | Total system efficiency | >85% | >89% |
| EPS-P004 | Battery depth of discharge per eclipse | ≤ 10 % nominal, ≤ 30 % contingency (BASELINE §5; SYS-112) | Analysis: 1.3–2.0 % (`hardware/eps/BATTERY_DESIGN.md` §3.2) |
| EPS-P005 | Voltage regulation accuracy | ±5% | ±3% |

### 2.3 Interface Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| EPS-I001 | I2C interface to OBC | Address 0x48, 100kHz |
| EPS-I002 | GPIO load enable signals | Active-high, 3.3V logic |
| EPS-I003 | Analog telemetry | 12-bit ADC resolution |
| EPS-I004 | Solar array connector | Standard PC-104 interface |
| EPS-I005 | Battery connector | 4-pin Molex with sense lines |

### 2.4 Environmental Requirements

| Parameter | Operating | Non-Operating |
|-----------|-----------|---------------|
| Temperature | -20°C to +50°C | -40°C to +85°C |
| Vibration | 14.1 Grms (random) | N/A |
| Shock | 40g (half-sine) | N/A |
| Radiation | 10 krad TID | N/A |

---

## 3. EPS Architecture

### 3.1 Block Diagram

```
                          SOLAR ARRAYS
                    +-------+ +-------+
                    |  +X   | |  -X   |
                    | Panel | | Panel |
                    +---+---+ +---+---+
                        |         |
                        v         v
              +-----------------------------+
              |        MPPT CONTROLLER      |
              |    (GomSpace NanoPower)     |
              |   - Maximum power tracking  |
              |   - Input: 4-40V            |
              |   - Efficiency: >97%        |
              +-------------+---------------+
                            |
                            v VBATT (14.4-16.8V)
              +-----------------------------+
              |   BATTERY MANAGEMENT (BMS)  |
              |      (ISIS iBMS)            |
              |   - 4S Li-ion protection    |
              |   - Cell balancing          |
              |   - Temperature monitoring  |
              +-------------+---------------+
                            |
              +-------------+---------------+
              |        4S Li-ion            |
              |     BATTERY PACK            |
              |   - 50Wh capacity           |
              |   - 18650 cells             |
              |   - Samsung INR18650-35E    |
              +-------------+---------------+
                            |
                            v VBATT
              +-----------------------------+
              |  POWER DISTRIBUTION UNIT    |
              |   (GomSpace NanoPower PDU)  |
              |                             |
              |  +-----+  +-----+  +-----+  |
              |  |3.3V |  | 5V  |  | 12V |  |
              |  |Rail |  |Rail |  |Rail |  |
              |  |2A   |  |3A   |  |1A   |  |
              +--+--+--+--+--+--+--+--+--+--+
                 |        |        |
    +------------+        |        +------------+
    |                     |                     |
    v 3.3V                v 5V                  v 12V
+-------+           +----------+           +-------+
|  OBC  |           |  Jetson  |           |  RF   |
|Sensors|           |  Orin    |           |  PA   |
| ADCS  |           |  Nano    |           |       |
+-------+           +----------+           +-------+
```

### 3.2 Power Budget

#### 3.2.1 Generation Budget

The array layout and the power-versus-attitude analysis are in `hardware/eps/SOLAR_ARRAY_DESIGN.md` (§3, §9). Summary (EOL, analysis):

| Case | Orbit average | Sunlit instantaneous | Basis |
|------|---------------|----------------------|-------|
| Tumbling, BASELINE method (no shadowing) | 2.39 W | 3.84 W | BASELINE §5; SOLAR_ARRAY_DESIGN §9.3 |
| Tumbling, with self-shadowing | 2.14 W | 3.43 W | SOLAR_ARRAY_DESIGN §9.3 |
| Sun-pointed (±Y face), no error / 20° error | 3.59 W / 3.38 W | 5.76 W / 5.44 W | SOLAR_ARRAY_DESIGN §9.3 |
| Both wings stowed, tumbling | 1.20 W | 1.92 W | SOLAR_ARRAY_DESIGN §9.3 |

*Assumptions (SOLAR_ARRAY_DESIGN §9.1–9.2): 16 cells, 1361 W/m², 500 km, β = 0 (35.8 min eclipse), cells at 60 °C,
5E13 e/cm² at EOL, 0.903 for harness, MPPT and UV losses. The previous table (11.3 W sunlit, 5.0 W orbit average from
280 cm²) assumed more cell area than the baseline array has and is withdrawn.*

#### 3.2.2 Consumption Budget

| Subsystem | Mode | Avg Power (W) | Peak Power (W) | Duty Cycle |
|-----------|------|---------------|----------------|------------|
| OBC | All | 0.15 | 0.25 | 100% |
| EPS | All | 0.10 | 0.15 | 100% |
| ADCS | Active | 0.30 | 0.80 | 50% |
| Sensors | Polling | 0.05 | 0.10 | 100% |
| Jetson | Inference | 5.00 | 7.00 | 20% |
| RF TX | Beacon | 1.50 | 2.00 | 5% |
| Heaters | Eclipse | 0.50 | 1.00 | 30% |
| **Total** | | **2.45** | **11.30** | |

#### 3.2.3 Power Margin

| Parameter | Value | Requirement | Margin |
|-----------|-------|-------------|--------|
| Generation (orbit avg) | 2.39 W tumbling / 3.58 W sun-pointed (BASELINE §5) | Load 1.80 W / 2.20 W | +33 % / +63 %; +19 % tumbling with self-shadowing (SOLAR_ARRAY_DESIGN §9.4) |
| Consumption (orbit avg) | 2.45 W | 3.5 W | +30% |
| Peak load | 11.3 W | 10.0 W | +13% |

### 3.3 Energy Storage

#### 3.3.1 Battery Configuration

| Parameter | Specification |
|-----------|---------------|
| Chemistry | Li-ion (NMC) |
| Configuration | 4S1P |
| Cell Type | Samsung INR18650-35E |
| Nominal Voltage | 14.4V |
| Voltage Range | 12.0V - 16.8V |
| Capacity | 3.45 Ah typical (49.7 Wh; ≈ 50 Wh, BASELINE §5) |
| Mass | 249 g CBE including holder and interconnect board (`docs/environment/MASS_BUDGET.md` rows 7–9) |

#### 3.3.2 Battery Sizing

Summary of the cycle-life analysis in `hardware/eps/BATTERY_DESIGN.md` §3.2–§3.3 (analysis, not measured):

```
Eclipse duration:   35.8 min (500 km, beta = 0)
Eclipse load:       1.00 W nominal, 1.50 W with heater (0.90 conversion efficiency)
DoD per eclipse:    1.3 % nominal, 2.0 % bounding (limit 10 %)
Eclipses per year:  5,559
Equivalent full
cycles in 2 years:  148 nominal, 408 bounding, against 500 full-depth 1C cycles
                    to 60 % capacity in the cell specification
Calendar aging:     not quantified; cell test required
```

The previous "> 3 years at 80 % capacity" had no cell data behind it and is withdrawn.

---

## 4. Component Specifications

### 4.1 MPPT Controller

**Selected Component**: GomSpace NanoPower MPPT

| Parameter | Specification |
|-----------|---------------|
| Input Voltage | 4-40V |
| Output Voltage | Battery voltage |
| Max Input Power | 40W |
| MPPT Efficiency | >97% |
| Algorithm | Perturb & Observe |
| Update Rate | 10 Hz |
| Mass | 45g |
| Interface | I2C |

### 4.2 Battery Management System

**Selected Component**: ISIS iBMS

| Parameter | Specification |
|-----------|---------------|
| Cell Configuration | 4S Li-ion |
| Overcharge Protection | 4.25V/cell |
| Overdischarge Protection | 2.7V/cell |
| Overcurrent Protection | 5A |
| Balance Current | 100mA |
| Temperature Range | -20°C to +60°C |
| Mass | 38g |
| Interface | I2C |

### 4.3 Power Distribution Unit

**Selected Component**: GomSpace NanoPower PDU

| Parameter | Specification |
|-----------|---------------|
| Input Voltage | 10-20V |
| Output Rails | 3.3V, 5V, 12V |
| 3.3V Current | 2A continuous |
| 5V Current | 3A continuous |
| 12V Current | 1A continuous |
| Efficiency | >92% |
| Load Switches | 6 channels, GPIO controlled |
| Mass | 28g |
| Interface | I2C |

### 4.4 Solar Cells

**Selected Component**: AzurSpace 3G30C

| Parameter | Specification |
|-----------|---------------|
| Cell Type | Triple-junction InGaP/GaAs/Ge, 40 × 80 mm (30.18 cm²), as 3G30A assembly |
| Efficiency (BOL) | 29.2 % (assembly, AM0 1367 W/m², 28 °C) |
| Efficiency (EOL) | 28.1 % at 5E13 e/cm², 25.9 % at 1E15 e/cm² (1-MeV electrons) |
| Vmp | 2.411 V (BOL) |
| Imp | 499 mA per cell (BOL) |
| Temperature Coefficients | ΔVmp/ΔT −6.7 mV/°C, ΔImp/ΔT +0.24 mA/°C (BOL) |
| Cover Glass | 100 µm, AR coated |
| Source | AZUR SPACE datasheet DB 00010891-01 (2025); see `hardware/eps/SOLAR_ARRAY_DESIGN.md` §2 |

---

## 5. Power Modes

### 5.1 Mode Definitions

| Mode | SOC Range | Loads Enabled | Beacon Rate |
|------|-----------|---------------|-------------|
| SAFE | <25% | OBC, EPS only | 180s |
| IDLE | 25-55% | + ADCS, Radio | 60s |
| ACTIVE | >55% | + Jetson payload | 30s |

### 5.2 Mode Transitions

```
                    +--------+
           SOC<25%  |  SAFE  | SOC<25%
        +---------->|  MODE  |<----------+
        |           +---+----+           |
        |               |                |
        |          SOC>=40%              |
        |               |                |
        |               v                |
        |           +--------+           |
        | SOC<25%   |  IDLE  |  SOC<25%  |
        +-----------|  MODE  |-----------+
                    +---+----+
                        |
                   SOC>=55%
                        |
                        v
                    +--------+
                    | ACTIVE |
                    |  MODE  |
                    +--------+
```

### 5.3 Load Control

| Load | GPIO Pin | Enable Level | Current Limit |
|------|----------|--------------|---------------|
| Jetson Power | GPIO_0 | High | 3A @ 5V |
| Radio TX | GPIO_1 | High | 0.5A @ 12V |
| ADCS | GPIO_2 | High | 0.5A @ 3.3V |
| Heaters | GPIO_3 | High | 0.5A @ 3.3V |
| Sensors | GPIO_4 | High | 0.2A @ 3.3V |
| Spare | GPIO_5 | High | 0.2A @ 3.3V |

---

## 6. Telemetry

### 6.1 EPS Telemetry Points

| Parameter | Address | Resolution | Update Rate |
|-----------|---------|------------|-------------|
| Battery Voltage | 0x00-0x01 | 1 mV | 1 Hz |
| Battery Current | 0x02-0x03 | 1 mA | 1 Hz |
| State of Charge | 0x04 | 1% | 1 Hz |
| Solar Array Current | 0x05-0x06 | 1 mA | 1 Hz |
| 3.3V Rail Voltage | 0x07-0x08 | 1 mV | 1 Hz |
| 5V Rail Voltage | 0x09-0x0A | 1 mV | 1 Hz |
| 12V Rail Voltage | 0x0B-0x0C | 1 mV | 1 Hz |
| Battery Temperature | 0x0D-0x0E | 0.1°C | 1 Hz |
| Power Mode | 0x0F | Enum | 1 Hz |
| Fault Status | 0x10-0x11 | Bitmask | 1 Hz |

### 6.2 Fault Detection

| Fault | Detection Criteria | Response |
|-------|-------------------|----------|
| Battery Undervoltage | VBATT < 12.0V | Enter SAFE mode |
| Battery Overvoltage | VBATT > 17.0V | Disable charging |
| Overcurrent | Any rail > limit | Disable load |
| Battery temperature outside the charge window | TBATT < 0°C or > 45°C | Inhibit charging (hardware; BASELINE §5) |
| Solar Array Fault | I_solar < expected | Log fault |
| MPPT Fault | No MPPT output | Switch to direct |

---

## 7. Thermal Design

### 7.1 Battery Thermal Control

| Condition | Action |
|-----------|--------|
| T < 0°C | Enable heaters, charging inhibited (SYS-602; BASELINE §5) |
| 0°C ≤ T ≤ 45°C | Normal operation, charging allowed |
| T > 45°C | Charging inhibited |
| T > 60°C | Discharge cutoff (cell specification), enter SAFE mode |

### 7.2 Heat Dissipation

| Component | Power Dissipation | Mounting |
|-----------|-------------------|----------|
| PDU | 1.0 W (worst case) | Thermally coupled to structure |
| MPPT | 0.5 W (worst case) | Direct mount to -Z panel |
| BMS | 0.2 W | Thermally coupled to battery |
| Battery | 0.5 W charging | Thermally isolated |

---

## 8. Verification

### 8.1 Verification Matrix

| Requirement | Method | Status |
|-------------|--------|--------|
| EPS-F001 | Analysis, Test | Pending |
| EPS-F002 | Test | Pending |
| EPS-F003 | Test | Pending |
| EPS-F004 | Test | Pending |
| EPS-F005 | Test | Pending |
| EPS-F006 | Test | Pending |
| EPS-F007 | Test | Pending |
| EPS-F008 | Test | Pending |
| EPS-F009 | Test | Pending |
| EPS-F010 | Test | Pending |

### 8.2 Test Procedures

| Test | Description | Equipment |
|------|-------------|-----------|
| TP-EPS-001 | MPPT efficiency | Solar simulator, power analyzer |
| TP-EPS-002 | Battery capacity | Charge/discharge cycler |
| TP-EPS-003 | Load regulation | Electronic load, oscilloscope |
| TP-EPS-004 | Protection testing | Fault injection equipment |
| TP-EPS-005 | Thermal cycling | Thermal chamber |
| TP-EPS-006 | EMC testing | Shielded room, spectrum analyzer |

---

## 9. Risk Assessment

### 9.1 EPS-Specific Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| EPS-R001 | Battery degradation faster than expected | M | M | Limit DOD, temperature control |
| EPS-R002 | MPPT efficiency below spec | L | M | Component screening |
| EPS-R003 | Solar cell degradation | M | L | BOL/EOL margin |
| EPS-R004 | Connector failure | L | H | Redundant connections |
| EPS-R005 | Single event upset in PDU | L | M | Watchdog, periodic reset |

---

## Appendix A: Bill of Materials

| Item | Part Number | Quantity | Unit Cost |
|------|-------------|----------|-----------|
| MPPT Module | GomSpace NanoPower MPPT | 1 | $2,500 |
| BMS | ISIS iBMS | 1 | $1,800 |
| PDU | GomSpace NanoPower PDU | 1 | $1,700 |
| Battery Cell | Samsung INR18650-35E | 4 | $15 |
| Solar Cell | AzurSpace 3G30A (3G30C assembly) | 16 | $400 |
| Harness | Custom | 1 | $500 |
| **Total** | | | **$12,960** |

---

## Appendix B: Interface Pinout

### B.1 EPS I2C Interface

| Pin | Signal | Description |
|-----|--------|-------------|
| 1 | VCC | 3.3V power |
| 2 | GND | Ground |
| 3 | SDA | I2C data |
| 4 | SCL | I2C clock |
| 5 | INT | Interrupt (active low) |
| 6 | RST | Reset (active low) |

### B.2 Load Switch Interface

| Pin | Signal | Description |
|-----|--------|-------------|
| 1-6 | EN_0-5 | Load enable (active high) |
| 7-12 | STAT_0-5 | Load status (active high) |
| 13 | FAULT | Fault output (active low) |
| 14 | GND | Ground |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | Funding audit | Generation (EPS-F001, EPS-F002, §3.2.1, §3.2.3), solar cells (§4.4) and BOM aligned with SOLAR_ARRAY_DESIGN v2.0 (TB-14); battery DoD limit (EPS-P004), sizing (§3.3) and charge-temperature rules (§6.2, §7.1) aligned with BASELINE §5 and BATTERY_DESIGN v2.0 (TB-08) |
