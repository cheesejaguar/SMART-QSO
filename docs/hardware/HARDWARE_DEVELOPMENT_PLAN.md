# SMART-QSO Hardware Development Plan

**Document ID**: SQSO-HDP-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft plan. No hardware has been built: the boards exist as build-to schematics captured as code (no PCB
layout), the mechanical design as CAD and FE models; no engineering model (EM) exists yet. `docs/BASELINE_PARAMETERS.md`
(Rev E) governs every design value; dates are those of the integrated master schedule `mission/SCHEDULE.md`
(SQSO-SCH-001), which governs.

## 1. Purpose

This document defines the Hardware Development Plan for SMART-QSO: the approach, models, methods and sequence for
developing the flight hardware of the 1U spacecraft. Test levels and the model philosophy are set by
`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` (Rev C); build or buy is decided in `docs/hardware/BUILD_VS_BUY.md`.

## 2. Hardware Development Approach

### 2.1 Development Philosophy

```
HARDWARE DEVELOPMENT PHILOSOPHY

For CubeSat-class missions:

1. Maximize COTS/Heritage
   └─ Use proven components where possible
   └─ Custom designs only where necessary

2. Prototype Early
   └─ Breadboard for concept validation
   └─ EDU for software development
   └─ EM for qualification

3. Test As You Fly
   └─ Flight-like test configurations
   └─ Realistic environmental conditions

4. Design for Manufacturability
   └─ Standard processes
   └─ Available materials
   └─ Repeatable assembly
```

### 2.2 Hardware Classification

| Classification | Definition | Items (BUILD_VS_BUY §1) |
|----------------|------------|-------------------------|
| COTS, bought | Vendor product, flown or catalogued | 1U frame, ISIS AntS antenna, CubeSpace CR0002 magnetorquer rods, Epson M-G370PDT0 gyro, SkyFox piNAV-NG/FM GNSS receiver, Jetson Orin Nano 8 GB module, Samsung INR18650-35E cells |
| Custom panels, bought | Built to the project's layout by a panel vendor | Body panels and the two deployable wings with AZUR 3G30A cells |
| Custom, built | Designed by the project | EPS board, OBC board (with the PNI RM3100 magnetometer board A2 and the access-port board A3), RF board, Jetson carrier board, battery pack and isolated mount, harness |

### 2.3 Development Models

Protoflight approach (ENVIRONMENTAL_TEST_PLAN Rev C §2): one EM for development tests and one flight model (FM) tested
to protoflight levels and flown.

| Model | Purpose | Fidelity |
|-------|---------|----------|
| Development boards | Firmware development before the EM (EPS controller and TNC: tasks EPS_UT, TNC_UT in `mission/SCHEDULE.md`) | Vendor evaluation boards |
| EM | Flatsat with the flight software on target, development tests that retire design risk before the independent CDR, ≥ 500 h reliability run, pathfinder for the protoflight procedures | Flight-design boards (EM lot), COTS frame and AntS, non-flight battery lot |
| FM (protoflight unit) | Protoflight test campaign, then delivery and flight | Flight boards, flight-lot screened cells, flight panels |

## 3. Hardware Architecture

### 3.1 Subsystem Breakdown

```
HARDWARE ARCHITECTURE

SMART-QSO Spacecraft
├── Structure Subsystem
│   ├── Primary structure (rails, panels)
│   ├── Deployment mechanism
│   └── Mounting hardware
│
├── Electrical Power Subsystem (EPS)
│   ├── Solar arrays
│   ├── Battery pack
│   ├── Power conditioning
│   └── Power distribution
│
├── On-Board Computer (OBC)
│   ├── Processor board
│   ├── Memory
│   └── Interfaces
│
├── RF Subsystem
│   ├── 435 MHz receiver and TNC
│   ├── 145.9 MHz exciter and power amplifier
│   ├── FM repeat path and XPDR_EN gate
│   └── Antenna system (ISIS AntS)
│
├── AI Payload
│   ├── Jetson Orin Nano
│   ├── Carrier board
│   └── Thermal management
│
├── ADCS (on the OBC board; no ADCS board)
│   ├── Magnetometer (board A2)
│   ├── Sun sensors (panel photodiodes)
│   ├── Gyros
│   └── Magnetorquer rods and H-bridges
│
└── GNSS receiver and patch antenna
```

### 3.2 Interface Summary

The signal lists and pins are in `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001), which governs.

| Interface | Type | Protocol |
|-----------|------|----------|
| OBC-EPS | I2C2 and discrete lines (harness W3) | Telemetry, commands, enables, burn and passivation lines (`docs/interfaces/ICD_OBC_EPS.md`) |
| OBC-RF | UART (KISS to the TNC) and discrete lines (W4A) | Frames, TX power select, `XPDR_EN` heartbeat (`docs/interfaces/ICD_OBC_RF.md`) |
| OBC-Jetson | USART1 (W5B) | Beacon-text requests and responses (`docs/interfaces/ICD_OBC_JETSON.md`) |
| OBC-Sensors | SPI and I2C, on the OBC board and W6 | Gyro, magnetometer, sun-sensor ADC (`docs/interfaces/ICD_OBC_ADCS.md`) |
| OBC-GNSS | LPUART1 and 1PPS (W9) | NMEA |
| OBC-AntS | I2C3 (W7) | Arm and deploy sequence |

## 4. Development Schedule

### 4.1 Hardware Development Timeline

Planning dates from `mission/SCHEDULE.md` Rev E §1–§2 (critical-path model `software/simulation/cdr/pm/schedule_cpm.py`;
planning launch 2028-11-01). The schedule governs; this table is a summary.

| Phase | Dates (planning) | Activities |
|-------|------------------|------------|
| EM layout | 2027-02 to 2027-04-27 | Layout of the EPS, OBC, RF, carrier, A2 and A3 boards from the build-to designs; cross-board check |
| EM fabrication | 2027-04-27 to 2027-06-01 | PCB fabrication and assembly |
| EM integration and flatsat | 2027-06-01 to 2027-07-27 | Flatsat with the flight software on target |
| EM development tests, ≥ 500 h flatsat run | 2027-07-27 to 2027-09-21 | Jetson heat path and burst energy, EM TVAC, pathfinder vibration, RF/EMC pre-scan, GNSS noise, antenna pattern |
| FM build | 2027-09-28 to 2028-02-01 | FM layout update, COTS buys, flight-lot battery, fabrication, integration |
| FM functional and burn-in | 2028-02-01 to 2028-03-03 | Comprehensive functional, day-in-the-life, ≥ 200 h burn-in |
| Protoflight campaign | 2028-03-03 to 2028-04-21 | Mass properties, vibration, TVAC and bake-out, deployment hot/cold, EMC, magnetic dipole |

### 4.2 Key Milestones

| Milestone | Target |
|-----------|--------|
| Independent PDR | 2027-01-12 |
| EM flatsat complete | Before the independent CDR |
| Independent CDR (with EM test data) | 2027-09-28 |
| Protoflight campaign complete | By L − 4 months (2028-07-01 for the planning launch) |
| Pre-ship review | Before delivery |
| Delivery to the integrator | L − 50 days (2028-09-12 for the planning launch) |

### 4.3 Build Flow

```
HARDWARE BUILD FLOW

Development boards ──→ EM ──→ FM (protoflight)
        │               │            │
        ▼               ▼            ▼
    Firmware       Flatsat and   Protoflight
   development     development   campaign,
                      tests      delivery
```

## 5. Subsystem Development

Status of every item: **designed, not built**. "Build-to design" means a schematic captured as code with selected
parts, netlist, BOM and ERC, and no PCB layout (BASELINE §1: Phase B, pre-PDR).

### 5.1 Structure Subsystem

| Item | Approach | Status |
|------|----------|--------|
| 1U frame and rails | Bought (ISIS, Pumpkin or Spacemanic; RFQ with a recessed −Z face) (BUILD_VS_BUY §2.2) | RFQ to issue |
| Body panels and deployable wings | Bought from one panel vendor to the project's layout (BUILD_VS_BUY §2.2) | Layout in `hardware/eps/SOLAR_ARRAY_DESIGN.md` |
| Wing hinges, springs, burn-wire hold-downs | Panel vendor with the project's deployment control | Design in `hardware/structure/STRUCTURAL_DESIGN.md` §6 |
| Internal layout, brackets, ballast | Custom | CAD and FE models `hardware/mechanical/MECHANICAL_DESIGN.md` (SQSO-MECH-CAD-001) |

**Key Design Features** (BASELINE §4):
- 100 × 100 × 113.5 mm envelope (CDS Rev 14.1); CBE 1.52 kg, MEV 1.75 kg, limit 2.00 kg
- Stack with the battery at the −Z end and the RF board at +Z; 2 × 10 g inertia ballast (decision C-13)
- Two deployable solar wings and the ISIS AntS, released after the 30 min timer; ≥ 3 independent inhibits (DS1–DS3)

### 5.2 EPS Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Solar cells | AZUR SPACE 3G30A on bought panels (BUILD_VS_BUY §1) | Layout and power table `hardware/eps/SOLAR_ARRAY_DESIGN.md` |
| Battery | Built: 4S1P Samsung INR18650-35E, isolated mount (decision D8) | `hardware/eps/BATTERY_DESIGN.md` |
| EPS board (trackers, charger, BMS, distribution, deployment drivers) | Built; ISIS ICEPS2 Type B the alternate | Build-to design `hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001) |

**Key Design Features** (BASELINE §5):
- 16 triple-junction cells: 2 on each long body face, 2 per side on two wings
- 4S1P ≈ 50 Wh pack, 14.4 V nominal, charged to 4.10 V/cell
- Hardware battery protection and inhibits; permanent end-of-life charge disconnect (47 CFR 97.207(g))
- Software power management with the SOC mode thresholds of BASELINE §5

### 5.3 OBC Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Processor | STM32L4R5ZIT6 (BASELINE §6) | Build-to design `hardware/electrical/obc/OBC_BOARD_DESIGN.md` (SQSO-OBC-BRD-001) |
| Memory | Two FM25V20A 2 Mbit FRAMs (the second a mirror); no external flash | As above |
| Supervisor | TPS3850H33-Q1 window watchdog wired to NRST | As above |
| ADCS sensors and drivers | On the OBC board; magnetometer on board A2 | As above, §9–§10 |

**Key Design Features**:
- Flight software in Rust on the Embassy executor, no RTOS (BASELINE §6)
- Latch-up switches on every switched sensor rail; mission clock on the always-on rail
- GSE umbilical through the access-port board A3

### 5.4 RF Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Receiver | TI CC1200 (receive only, analog FM) with a BFP640F LNA, high-pass filter and SAW; TNC on an STM32L431 | Build-to design `hardware/electrical/rf/RF_BOARD_DESIGN.md` (SQSO-RF-BRD-001) |
| Exciter and PA | TI LMX2571 with PCM1808 deviation; Mitsubishi RD01MUS3 PA; elliptic harmonic filter | As above |
| FM repeat path | Hardware `XPDR_EN` gate, CTCSS decode, key-down timer | As above, §8 |
| Antenna | Bought: ISIS AntS 1U–3U, separate 2 m and 70 cm monopoles | `hardware/rf/ANTENNA_DESIGN.md` |

**Key Design Features** (BASELINE §7):
- Downlink one coordinated frequency in 145.800–146.000 MHz; uplink in 435.222–435.778 MHz (or 436.222–436.500 MHz)
- 0.5 W nominal, 1 W maximum transmitter power
- Single-channel FM transponder (duty-gated) and 1200 bps AX.25 telemetry; hardware TX-OFF

### 5.5 AI Payload

| Item | Approach | Status |
|------|----------|--------|
| Compute | NVIDIA Jetson Orin Nano 8 GB module (P/N 900-13767-0030-000), 7 W mode | Bought |
| Carrier | Custom: 5 V from VBAT, eFuse, NVMe storage, UART to the OBC | Build-to design `hardware/electrical/jetson/CARRIER_BOARD_DESIGN.md` (SQSO-JET-BRD-001) |
| Thermal | Passive: spreader pedestal to the frame, duty-cycled | `hardware/structure/THERMAL_DESIGN.md`; measured on the EM (VI-01) |
| Power | Two series switches, OBC-enforced current and temperature trips, hardware load shed below 13.6 V | `docs/hardware/PAYLOAD_DESIGN.md` §11 |

**Key Design Features** (BASELINE §6):
- Gated experiment only: generative beacon text, enabled only if the measured energy per accepted beacon line is ≤ 0.1 Wh
- Flight baseline is the template selector on the OBC; the Jetson is not needed for the mission
- Bursts of ≤ 10 min, then ≥ 100 min off; sunlit and SOC ≥ 55 % only

## 6. Fabrication

### 6.1 Fabrication Methods

| Component | Method | Vendor |
|-----------|--------|--------|
| Structure | Bought frame; brackets CNC machined | Frame vendor per RFQ (BUILD_VS_BUY §2.2); machine shop to select |
| PCBs | 6-layer FR-4 (2-layer for boards A2, A3) | To select with the layout designers (`mission/SCHEDULE.md` DA-04) |
| Harness | Hand assembly | In-house |
| Integration | In-house | - |

### 6.2 Fabrication Standards

| Standard | Application |
|----------|-------------|
| IPC-A-600 | PCB fabrication |
| IPC-A-610 | PCB assembly |
| IPC J-STD-001 with the space addendum (NASA-STD-8739.6B) | Soldering (NASA-STD-8739.3 is cancelled) |
| NASA-STD-8739.4A | Crimping, cabling and harnessing |

### 6.3 Fabrication Flow

```
FABRICATION FLOW

1. Procurement
   └─ Parts, materials, services

2. Receiving Inspection
   └─ Verify compliance

3. Board Fabrication
   └─ External vendor

4. Board Assembly
   └─ In-house or vendor

5. Board Test
   └─ Functional verification

6. Integration
   └─ Mechanical and electrical

7. System Test
   └─ End-to-end verification
```

## 7. Assembly and Integration

### 7.1 Assembly Sequence

| Step | Activity | Prerequisites |
|------|----------|---------------|
| 1 | Structure assembly | Parts received |
| 2 | Harness installation | Structure complete |
| 3 | EPS installation | Harness installed |
| 4 | OBC installation | EPS installed |
| 5 | RF installation | OBC installed |
| 6 | Payload installation | RF installed |
| 7 | Antenna installation | All boards installed |
| 8 | Close-out | All installation complete |

### 7.2 Integration Environment

| Requirement | Specification |
|-------------|---------------|
| Cleanliness | Class 100,000 minimum |
| ESD control | Per ANSI/ESD S20.20 |
| Temperature | 20-25°C |
| Humidity | 30-60% RH |

### 7.3 Integration Documentation

| Document | Purpose |
|----------|---------|
| Assembly drawings | Build instructions |
| Procedures | Step-by-step guide |
| Travelers | As-built record |
| Photos | Visual record |

## 8. Testing

### 8.1 Test Program

| Level | Tests | Purpose |
|-------|-------|---------|
| Board | Functional | Verify board operation |
| Subsystem | Integration | Verify interfaces |
| System | Functional | Verify system operation |
| Environmental | Qualification | Verify robustness |
| Acceptance | Workmanship | Flight readiness |

### 8.2 Environmental Tests

| Test | Level | Duration |
|------|-------|----------|
Levels and durations: `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C, which governs (BASELINE §8).

| Test | Level | Duration |
|------|-------|----------|
| Random vibration | Protoflight: envelope of MPE + 3 dB and the GSFC-STD-7000B workmanship spectrum (8.18 Grms on the interim levels) | 1 min/axis |
| Thermal vacuum | Protoflight: allowable flight temperatures ± 10 °C, hot plateaus capped at +60 °C | 8 cycles; bake-out +60 °C, 6 h |
| EMC/EMI | Tailored | Per ENVIRONMENTAL_TEST_PLAN |

### 8.3 Test Flow

```
TEST FLOW

Board Test
    │
    ▼
Subsystem Test
    │
    ▼
Flatsat Test ◄─── SW Development
    │
    ▼
System Integration Test
    │
    ▼
Pre-Environmental Functional
    │
    ▼
Environmental Test ──► Vibration
    │                  Thermal Vacuum
    │                  EMC
    ▼
Post-Environmental Functional
    │
    ▼
Acceptance Test
    │
    ▼
FLIGHT READY
```

## 9. Quality Assurance

### 9.1 QA Activities

| Activity | When |
|----------|------|
| Design review | Independent PDR (2027-01-12) and CDR (2027-09-28) |
| Receiving inspection | Parts arrival |
| In-process inspection | During assembly |
| Final inspection | Before close-out |
| Test witness | Key tests |

### 9.2 Critical Items

| Item | Controls |
|------|----------|
| Battery | Special handling, testing |
| Solar cells | ESD, handling |
| Antenna mechanism | Functional test |
| RF connectors | Torque verification |

### 9.3 Workmanship

| Standard | Application |
|----------|-------------|
| IPC J-STD-001 with the space addendum (NASA-STD-8739.6B) | Soldering |
| NASA-STD-8739.4A | Crimping |
| IPC-A-610 Class 3 | Inspection |

Workmanship plan: `docs/manufacturing/WORKMANSHIP_STANDARDS.md`.

## 10. Configuration Control

### 10.1 Hardware Baseline

| Baseline | When | Content |
|----------|------|---------|
| Design | CDR | Drawings, specs |
| As-built | Integration | Serial numbers |
| Flight | FRR | Final configuration |

### 10.2 Change Control

| Change Type | Authority |
|-------------|-----------|
| Drawing (minor) | Lead engineer |
| Drawing (major) | CCB |
| Parts substitution | CCB |
| Design change | CCB |

### 10.3 Traceability

| Item | Tracking |
|------|----------|
| Parts | Lot/date codes |
| Assemblies | Serial numbers |
| Configuration | As-built list |
| Changes | ECO numbers |

## 11. Risk Management

### 11.1 Hardware Risks

| Risk | Mitigation |
|------|------------|
| Parts obsolescence | Early procurement |
| Vendor delays | Schedule margin |
| Test failures | Retest/repair capability |
| Workmanship | Training, inspection |

### 11.2 Technical Risks

| Risk | Mitigation |
|------|------------|
| Thermal | Analysis, testing |
| Power budget | Margin, testing |
| Mass budget | Tracking, design |
| RF performance | Analysis, testing |

## 12. Deliverables

### 12.1 Hardware Deliverables

| Deliverable | Description |
|-------------|-------------|
| Flight unit | Integrated spacecraft |
| GSE | Ground support equipment |
| Spares | Critical spares |
| Documentation | As-built records |

### 12.2 Documentation Deliverables

| Document | Purpose |
|----------|---------|
| As-built drawings | Final configuration |
| Test reports | Verification evidence |
| Travelers | Assembly records |
| Calibration records | Equipment status |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): status corrected (nothing built, no layout); model philosophy to EM + protoflight FM (ETP Rev C); §2.2 and §5 to the build-versus-buy decisions and the board designs (STM32L4R5, 4S1P battery, AZUR cells, CC1200/LMX2571/RD01MUS3 radio, ISIS AntS, bought frame; Apollo4, SPV1050, Si4463, 2-cell pack and tape-spring antenna removed); §3.2 interfaces per SQSO-ICX-001; §4 dates from SQSO-SCH-001 Rev E; §8.2 levels per BASELINE §8; soldering standard (8739.3 cancelled); "NASA Ames SmallSat heritage" reference removed |
