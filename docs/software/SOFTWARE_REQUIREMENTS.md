# Software Requirements Specification (SRS)

**Document Number**: SMART-QSO-SRS-001
**Version**: 1.7
**Date**: 2026-10-27
**Status**: Preliminary

**Governing Standards**:
- NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
- NPR 7150.2D - NASA Software Engineering Requirements (Class C; requirement mapping in `docs/software/NPR7150_COMPLIANCE_MATRIX.md`)
- NASA-STD-8739.8B - Software Assurance and Software Safety

---

## 1. Introduction

### 1.1 Purpose

This Software Requirements Specification (SRS) defines the functional, performance, interface, and quality requirements for the SMART-QSO flight software.

### 1.2 Scope

This document covers requirements for:
- OBC Flight Software
- Jetson AI Payload Software
- Ground Support Software

### 1.3 Requirements Identification

Requirements are identified as:
- **SRS-Fxxx**: Functional requirements
- **SRS-Pxxx**: Performance requirements
- **SRS-Ixxx**: Interface requirements
- **SRS-Qxxx**: Quality requirements
- **SRS-Sxxx**: Safety requirements

---

## 2. System Overview

### 2.1 System Context

```
                    +------------------+
                    |   Ground Station |
                    +--------+---------+
                             |
                         RF Link
                             |
+--------------------------------------------------+
|                    SMART-QSO CubeSat             |
|  +------------+   UART   +------------------+    |
|  |   OBC      |<-------->| Jetson AI Payload|    |
|  | (Embassy)  |          | (Linux)          |    |
|  +-----+------+          +------------------+    |
|        |                                         |
|   +----+----+----+----+                          |
|   |    |    |    |    |                          |
|   v    v    v    v    v                          |
|  EPS  ADCS  RF  Sensors Deployment               |
+--------------------------------------------------+
```

### 2.2 Software Functions

1. **Electrical Power Subsystem (EPS) Control**
2. **Attitude Determination and Control (ADCS)**
3. **RF Beacon Transmission**
4. **AI-based Beacon Generation**
5. **Telemetry Collection and Transmission**
6. **Fault Detection, Isolation, and Recovery (FDIR)**

---

## 3. Functional Requirements

### 3.1 System Initialization

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F001 | The software shall initialize all subsystems within 30 seconds of power-on | T | High |
| SRS-F002 | The software shall load configuration from non-volatile memory on startup | T | High |
| SRS-F003 | The software shall perform self-test and report status via beacon | T | High |
| SRS-F004 | The software shall increment reset counter on each boot | T | Medium |
| SRS-F005 | The software shall log boot reason (power-on, watchdog, commanded) | T | Medium |

### 3.2 EPS Control

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F010 | The software shall monitor battery state of charge (SOC) | T | High |
| SRS-F011 | The software shall control power to subsystems via load switches | T | High |
| SRS-F012 | The software shall implement power modes: SAFE, IDLE, ACTIVE | T | High |
| SRS-F013 | The software shall transition to SAFE mode when SOC < 25% | T | High |
| SRS-F014 | The software shall transition to IDLE mode when SOC < 40% | T | High |
| SRS-F015 | The software shall enable payload only when SOC > 55% and sunlit | T | High |
| SRS-F016 | The software shall enforce power budget limits per mode | T | Medium |
| SRS-F017 | The software shall persist EPS state across resets | T | Medium |
| SRS-F018 | [Proposed] The software shall log every operating-mode transition | T | Medium |

### 3.3 Beacon Transmission

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F020 | The software shall transmit AX.25 beacon at configurable intervals | T | High |
| SRS-F021 | The software shall include callsign in all beacon transmissions | I | High |
| SRS-F022 | The software shall include mission status in beacon info field | T | High |
| SRS-F023 | The software shall transmit AI-generated text when payload is active | T | High |
| SRS-F024 | The software shall fallback to template text when payload is inactive | T | High |
| SRS-F025 | The software shall limit beacon info field to 256 bytes | T | Medium |

### 3.4 AI Payload Control

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F030 | The software shall send health telemetry to Jetson via UART | T | High |
| SRS-F031 | The software shall receive beacon text from Jetson via UART | T | High |
| SRS-F032 | The software shall enforce timeout on Jetson communication | T | High |
| SRS-F033 | The software shall disable payload on communication timeout | T | High |
| SRS-F034 | The software shall validate received beacon text (ASCII, length) | T | High |
| SRS-F035 | The software shall monitor Jetson power consumption | T | Medium |

### 3.5 Fault Management

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F040 | The software shall detect and log all fault conditions | T | High |
| SRS-F041 | The software shall maintain fault log in non-volatile memory | T | High |
| SRS-F042 | The software shall implement autonomous fault recovery | T | High |
| SRS-F043 | The software shall enter SAFE mode on critical faults | T | High |
| SRS-F044 | The software shall attempt recovery from SAFE mode when SOC improves | T | Medium |
| SRS-F045 | The software shall track fault statistics across resets | T | Medium |

**Proposed (PI to ratify), SRS-F040, TBR-SRS-002:** "The software shall detect each fault type in the fault response matrix (`docs/software/SOFTWARE_DESIGN.md` §3.3.3) and, within 10 s of the fault condition starting, write one fault-log entry with time, fault type, severity and data." Verification: Test. Inject each listed fault type in the software-in-the-loop build and on the flatsat; pass if each produces one entry with the right type and severity. "All fault conditions" cannot be tested because the set is open; the list makes it finite. 10 s is one health-check cycle in the current flight code. The matrix lists 9 fault types and the flight code (`software/flight/include/smart_qso.h`) has 22, so the list itself is part of this item.

### 3.6 Watchdog

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F050 | The software shall kick hardware watchdog at least every 30 seconds | T | High |
| SRS-F051 | The software shall save state before watchdog reset | T | High |
| SRS-F052 | The software shall detect watchdog reset and log occurrence | T | Medium |

### 3.7 Data Persistence

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F060 | The software shall persist mission data to non-volatile memory | T | High |
| SRS-F061 | The software shall protect persisted data with CRC32 | T | High |
| SRS-F062 | The software shall detect and handle corrupted data | T | High |
| SRS-F063 | The software shall maintain mission statistics across resets | T | Medium |

### 3.8 Telemetry

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F070 | The software shall collect telemetry from all sensors | T | High |
| SRS-F071 | The software shall sample sensors at configurable rates | T | Medium |
| SRS-F072 | The software shall validate sensor readings against limits | T | High |
| SRS-F073 | The software shall include telemetry in beacon transmissions | T | High |

### 3.9 Transponder and Commissioning

Added at CDR (2026-10-27) for the duty-gated FM transponder (BASELINE §2, D4), the launch reviewer's findings LV-01,
LV-02, LV-04, LV-07, LV-14, LV-15 and LV-27, and the CDR avionics review. Verification: the named C suites, Rust unit
tests and the mission simulation (`software/rust/sqso-sil`).

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F080 | The software shall enable the FM transponder only while transmission is permitted, the radio is powered, the ground allows it, the mode is ACTIVE, the array is sunlit, the SOC is at least 55 %, the PA is below 70 °C (resuming at 60 °C) and the keyed time is under 25 % of the last 100 minutes | T | High |
| SRS-F081 | The software shall drive the transponder enable XPDR_EN as a heartbeat renewed by every main-loop pass while the transponder gates hold, with a lease of at most 40 ms, so that the radio board's 50 ms monostable drops the transponder within 90 ms of the last pass; a level on the pin shall not enable it | T | High |
| SRS-F082 | The software shall hold the mode at IDLE, the payload and the transponder off from ARM_FLIGHT until an authenticated commissioning release telecommand, across resets, and shall hold after an unreadable operations record | T | High |
| SRS-F083 | The software shall let the ground stop and allow the transponder by telecommand, persisted across resets, without changing its other gates | T | Medium |
| SRS-F084 | The software shall power the GNSS receiver for sessions, every orbit in launch and early operations then four a day, keep the latest fix with its time and quality, and start a session on command; a session shall run only with the Jetson off and the SOC at least 40 %, and shall hold the transponder off | T | Medium |
| SRS-F085 | The software shall downlink a housekeeping frame every fifth beacon and on command, carrying temperatures, battery current and voltage, array power, deployment, RF-silence, boot and reset, fault-log head, body rates, contact timeout and GNSS fix age, through the downlink's inhibits and duty limit | T | High |
| SRS-F086 | The software shall command no magnetorquer output until 120 s after separation (SpaceX Rideshare User's Guide v11 §3.3.4) | T | High |
| SRS-F087 | The software shall latch the served 45-minute RF silence, so that a sense line reading stowed afterwards does not restart it | T | High |
| SRS-F088 | The software shall space deployment attempts 50 minutes apart and accept a telecommand to retry an element with fresh attempts once the 30-minute wait is served | T | Medium |
| SRS-F089 | The software shall deploy the solar wings first, one burn resistor at a time on four independent lines (wing 1 resistor A, then wing 2 resistor A, the retries on the B resistors, never two at once), and then the antennas through the bought AntS controller over I2C (arm, deploy, cancel, disarm), reading each antenna element's deployment switch from the AntS status and reporting the deployed elements in housekeeping | T | High |

### 3.10 Attitude Determination and Control

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F090 | The software shall use a magnetometer sample only when the magnetorquers have been off for at least 50 ms, discard and count any other sample, and log the first discarded sample after init as a fault | T | High |
| SRS-F091 | The software shall command each magnetorquer burst for 900 ms of the 1 s control period, with the dipole scaled by the period over the on-time and saturated at 0.2 A·m², so the delivered impulse is the control law's | T | High |
| SRS-F092 | The software shall not complete a detumble on a magnetometer rate estimate that a discarded sample left stale | T | High |
| SRS-F093 | The software shall run B-dot detumble (gain 10⁴ A·m²·s/T) and spin-stabilized sun pointing of +Y at 8°/s (momentum gain 0.005 s⁻¹, spin hold 0.005 s⁻¹, nutation damping 3·10⁻⁴ N·m·s with the gyro and 1·10⁻⁴ N·m·s without it) at 1 Hz, and treat a detumble lasting over 285 minutes as a fault | T, A | High |
| SRS-F094 | The software shall let the ground select, by authenticated telecommand persisted across resets, the spin axis and inertia tensor from the presets for both wings, wing 1 only, wing 2 only and no wings deployed, and shall refuse any other preset | T | Medium |

### 3.11 OBC Selector (flight AI, decision D5)

Design: `docs/payload/OBC_SELECTOR_DESIGN.md` (SQSO-PAY-SEL-001). Not implemented at CDR (fabrication-phase task).

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F100 | The software shall select the beacon template, propose the transponder bucket and propose the power-mode preference with the OBC selector of SQSO-PAY-SEL-001, or abstain to the fixed rules | T | Medium |
| SRS-F101 | The selector shall choose only among the pre-approved templates, and every value in a beacon shall come from measurements | T, I | High |
| SRS-F102 | The selector's proposals shall only restrict the transponder gates and the mode manager, never extend them | T, A | High |
| SRS-F103 | The flight inference shall reproduce the trained model bit for bit on the shared test vectors | T | Medium |
| SRS-F104 | The combined decision shall never be less conservative than the fixed rules, for any inputs and any valid model table | T | High |
| SRS-F105 | A head shall abstain when a feature is stale, missing or outside its training range, or the calibrated confidence is below its threshold | T | Medium |
| SRS-F106 | A model table that fails its CRC, version or schema check shall disable the selector for the boot and be logged, and parsing any table shall be bounded and free of panics | T | Medium |
| SRS-F107 | A selector decision shall take at most 0.2 ms and 10 µJ on the STM32L4R5, and the mission simulation with the selector shall show no check worse than with the fixed rules | T, A | Medium |

### 3.12 End of Life, Stored Housekeeping, Contact and Robustness (CDR round 2)

Added at CDR (2026-10-27, flight software round 2) for the review items REG-01, EPS-06, FSW-07 and SMA-13 (end-of-life
passivation), DB-01 (stored housekeeping), GND-03 (command acknowledgement and contact count), RF-04 (beacon jitter),
REL-04 (beacon with a failed receiver), FSW-09 (receiver recovery), EPS-10 (wings not deployed), EPS-12 (recharge lockout),
THM-02 and THM-08 (thermal thresholds) and AV-07 (memory upsets), with the EPS board (SQSO-EPS-BRD-001). Command codes:
`software/flight/command_dictionary.json` (SQSO-ICD-013 §7). Verification: the named C suites (`test_passivation`,
`test_hk_store`, `test_receiver`, `test_cmd_handler`, `test_beacon`, `test_deployment`, `test_ops_state`, `test_rf_control`,
`test_payload_guard`), the Rust unit tests of the modules that carry each tag, and the mission simulation
(`software/rust/sqso-sil`, the checks of SQSO-SIL-001 §5).

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F110 | The software shall passivate only on two authenticated telecommands: PASSIVATE_ARM (0x70) with the operational key and the argument `EOL!`, then PASSIVATE_EXECUTE (0x71) with the recovery key within 600 s of the arm; an execute without a live arm shall be refused, and an arm not executed within 600 s shall lapse | T | High |
| SRS-F111 | On PASSIVATE_EXECUTE the software shall, in this order, latch TX-OFF, persist the passivation record, and fire the EPS one-shot charge disconnect through two separate outputs in series: EOL_ARM (its rising edge sets the EPS arm latch), then 200 ms later EOL_FIRE, a level held at least 60 s and at most 120 s and only in sunlight (an eclipse before 60 s drops both and the fire is repeated at the next sunlight, at most 3 attempts); it shall confirm the disconnect from the EPS EOL_STATE register and leave both outputs low | T | High |
| SRS-F112 | After passivation the software shall keep every transmitter off for good, across resets, whatever the telecommand: the passivation inhibit shall never be lifted, and a passivation the EPS reports done shall count as done without the record | T | High |
| SRS-F115 | Hung-receiver recovery by a brief power cycle, never during a pass: the software shall power-cycle the radio (RF_TX_EN low, then RADIO_EN low for 2 s) when the command receiver has had neither a valid frame nor a carrier for 24 h with its rail on, never while the downlink or the transponder is keyed or within 15 min of a carrier, at most once per 24 h of silence, and log each cycle | T | High |
| SRS-F116 | Evidence that the command receiver has failed: the software shall hold the receiver known failed only when a recovery cycle was made in the current silence (after an OBC reset, a new one), the receiver's health line reports a fault and the loss-of-contact timeout has expired (the persisted count of the contact monitor); a valid frame, a carrier or a healthy line shall clear it | T | High |
| SRS-F117 | With the command receiver known failed, the loss-of-contact inhibit shall not silence the beacon; TX-OFF, the RF silence and passivation shall still silence it, and the transponder shall stay off | T, A | High |
| SRS-F120 | Five-minute housekeeping records for at least 7 days in FRAM: the software shall store a 44-byte record under CRC every 5 minutes in a ring of 2,048 records (7.1 days, 90,112 B), numbered across resets | T | High |
| SRS-F121 | Sixty-second records for the 6 h around each fault and mode change: the software shall store a record every 60 s in a ring of 409 records (6.8 h) and copy the 3 h before and the 3 h after each fault and mode change into one of three event windows | T | Medium |
| SRS-F122 | DUMP_TLM downlinks a range of stored records within the downlink duty limit: on DUMP_TLM (0x0A, operational key; store, first record, count of at most 600, a count of 0 stops a dump) the software shall downlink the records, four to a frame, at most one frame every 12 s, through the downlink's inhibits and duty limit | T, A | Medium |
| SRS-F125 | Acknowledgement frame after each authenticated command: after each command it counts as contact the software shall downlink `ack KK6GDB-11: C=<counter> R=<result> N=<days since the previous contact>` through the downlink's inhibits, within 30 s or not at all | T | Medium |
| SRS-F126 | The housekeeping frame shall carry the loss-of-contact count (days since contact) and the commands counted as contact since boot (`CN=`), with the passivation phase (`PS=`), the receiver recovery cycles and known-failed flag (`RX=`) and the stored-record counts (`ST=`) | T | Medium |
| SRS-F127 | The software shall jitter each beacon interval by up to 5 s either way, keeping the mean interval of each mode | T | Medium |
| SRS-F128 | The software shall hold the mode at IDLE while the wings read not deployed after their attempts are spent; a DEPLOY_RETRY of the wings shall release the hold | T | High |
| SRS-F129 | The software shall clear the EPS's latched recharge lockout only on an authenticated command with its code (EPS_CLEAR_LOCKOUT, 0x14, operational key, argument `CHG!`), by an I2C unlock (key 0xC5 to register 0x7D of the EPS controller) and a LOCK_CLR pulse of at least 100 ms, logged | T | High |
| SRS-F130 | The software shall switch the battery heater on below +5 °C and off above +9 °C | T | High |
| SRS-F131 | The software shall not power the Jetson while its heat spreader reads below −15 °C (a valid reading of the last minute) | T | High |
| SRS-F132 | SRAM2 parity and flash ECC errors: an NMI that resets at once, the cause logged at the next boot. The image shall link its static data in SRAM2, erase SRAM2 before using it, and log every boot that finds SRAM2 parity disabled | T, I | Medium |
| SRS-F133 | Periodic check of the critical persisted state against its CRC-protected records: every 60 s the software shall re-read the TX-OFF latch, the operations record and the passivation record and settle a disagreement with the RAM copy toward the safe side (TX-OFF, held, transponder stopped, passivated), logged | T | Medium |

**REL-04 against 47 CFR 97.207(b) and decision D3 (SRS-F117).** 97.207(b) requires the space station to be capable of
ceasing transmissions by telecommand when the FCC orders it. The loss-of-contact inhibit of D3 (N = 30 days, 14 after
commissioning) answers IARU coordination item 7.9 (what stops the transmitters if command contact is lost): when no
authenticated command has arrived for N days, the software cannot show that the ground could still stop it, so it stops.
With a receiver that has failed, the capability of 97.207(b) is lost whatever the software does; the inhibit cannot give it
back, and silencing the beacon then removes the only evidence of the spacecraft's state and of the failure. The exception is
therefore limited to the case where the evidence shows a receiver hardware failure rather than a ground that is silent:
all three conditions of SRS-F116 (a recovery cycle made, the health line in fault, the N days expired). After an OBC
reset the exception waits for a new recovery cycle (24 h of silence), so the beacon can be silent for up to a day after
each reset in that state. Only the
beacon and the housekeeping frames continue (at their duty cycle, through the downlink's duty limit); the transponder stays
off (its gate includes the inhibit), TX-OFF still latches if a command gets through, passivation still ends all
transmission, and a valid frame or carrier ends the exception. The exception keeps a transmitter on that cannot be turned
off by telecommand: it is to be declared in the IARU coordination request (item 7.9) and put to the FCC consultation of
D3. If either rejects it, it is removed before launch: one condition in `software/rust/sqso-ffi/src/contact_monitor.rs`
(the inhibit then applies whatever the receiver state; `test_receiver` changes with it).

---

## 4. Performance Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-P001 | The main control loop shall execute within 20 ms | T | High |
| SRS-P002 | Beacon transmission shall complete within 5 seconds | T | High |
| SRS-P003 | Fault detection shall occur within 100 ms of condition | T | High |
| SRS-P004 | Mode transition shall complete within 1 second | T | Medium |
| SRS-P005 | UART communication timeout shall be configurable (default 5s) | T | Medium |
| SRS-P006 | Sensor polling shall meet configured period within 10% | T | Medium |
| SRS-P007 | Data persistence shall complete within 500 ms | T | Medium |
| SRS-P008 | System initialization shall complete within 30 seconds | T | High |

---

## 5. Interface Requirements

### 5.1 UART Interface (OBC-Jetson)

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-I001 | The UART interface shall operate at 115200 baud, 8N1 | T | High |
| SRS-I002 | The UART shall use LVTTL voltage levels (3.3V) | I | High |
| SRS-I003 | Telemetry messages shall be formatted as CSV text | T | High |
| SRS-I004 | Beacon text responses shall be ASCII with newline terminator | T | High |
| SRS-I005 | Message format shall include CRC for integrity checking | T | Medium |

### 5.2 Sensor Interfaces

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-I010 | The software shall interface with EPS via I2C | T | High |
| SRS-I011 | The software shall read battery voltage, current, and temperature | T | High |
| SRS-I012 | The software shall read solar panel voltage and current | T | High |

### 5.3 RF Interface

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-I020 | The software shall control beacon transmitter via GPIO | T | High |
| SRS-I021 | The software shall format AX.25 UI frames | T | High |
| SRS-I022 | The software shall use AFSK 1200 bps modulation | T | High |
| SRS-I023 | The software shall hand each downlink AX.25 frame to the TNC on the radio UART as a KISS data frame, preceded by the TNC parameters (TXDELAY, persistence, slot time, TX tail, full duplex) | T | High |
| SRS-I024 | The software shall refuse a downlink frame unless transmission is permitted (TX-OFF, RF silence, loss-of-contact inhibit) at the moment it is handed to the TNC | T | High |
| SRS-I025 | The software shall send one downlink frame at a time, limit each to 5 s worst-case air time, and hold keyed time to 25 % (TBR-SRS-003) with a credit of at most one 60 s window | T | High |
| SRS-I026 | The software shall select the downlink power at the commanded level (0.5 W or 1 W, never above 1 W) | T | High |
| SRS-I027 | The software shall monitor the transmitter's temperature, forward and reflected power while keyed: VSWR above 3:1 on 3 samples in a row caps the power at 0.5 W, and at 0.5 W stops the downlink for 10 minutes; above 70 °C (TBR-SRS-003) nothing is sent until below 60 °C; a keyed frame without forward power is reported | T | High |

The downlink interface (SRS-I023) is a PI decision, CR-2026-005 (`docs/project/CHANGE_REQUEST_LOG.md`): a KISS TNC on
USART2, shared with the uplink receiver, adds the FCS, modulates (SRS-I022, AFSK 1200 in the TNC) and keys the
transmitter. Implemented in `software/rust/sqso-fsw/src/downlink.rs` and `software/flight/include/downlink.h`; the
TNC and transmitter are not selected, so SRS-I022 and the thresholds (TBR-SRS-003) are verified on hardware.

---

## 6. Quality Requirements

### 6.1 Reliability

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-Q001 | The software shall operate continuously for minimum 1 year | A | High |
| SRS-Q002 | The software shall recover from single-bit memory errors | T | Medium |
| SRS-Q003 | MTBF for software-induced failures shall exceed 30 days | A | Medium |

### 6.2 Maintainability

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-Q010 | The software shall be modular with defined interfaces | I | High |
| SRS-Q011 | Code coverage shall exceed 80% for unit tests | A | High |
| SRS-Q012 | All public functions shall have Doxygen documentation | I | Medium |
| SRS-Q013 | The safety-critical components of SOFTWARE_DEVELOPMENT_PLAN §3.2, the FM transponder control and the operations record included, shall have every line run and every condition outcome taken by the tests; a line no test can reach shall be marked as such and reviewed | A | High |

### 6.3 Portability

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-Q020 | The software shall compile on ARM and x86 platforms | T | High |
| SRS-Q021 | Hardware dependencies shall be isolated in HAL layer | I | High |
| SRS-Q022 | The software shall support both simulation and flight builds | T | High |

---

## 7. Safety Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-S001 | The software shall not enable RF transmission until after separation | T | High |
| SRS-S002 | The software shall disable all loads in SAFE mode except beacon | T | High |
| SRS-S003 | The software shall implement hardware watchdog | T | High |
| SRS-S004 | The software shall validate all commands before execution | T | High |
| SRS-S005 | The software shall limit power consumption to prevent thermal damage | T | High |

---

## 8. Constraints

### 8.1 Design Constraints

| ID | Constraint | Verification |
|----|------------|--------------|
| SRS-C001 | OBC code shall be written in Rust (2024 edition) or in C conforming to the C11 standard | I |
| SRS-C002 | No dynamic memory allocation after initialization | T |
| SRS-C003 | Stack size shall not exceed 8 KB | A, T |
| SRS-C004 | Total code size shall not exceed 256 KB | I |

How each constraint is verified: SRS-C001 by inspection of the build configuration and the CI toolchain pin; SRS-C002 by
the ctest `no_dynamic_memory` and CI's symbol check of the image; SRS-C003 by analysis (`scripts/stack_usage.py` on the
image, a CI gate) and test (HIL-044, the MPU guard); SRS-C004 by inspection (`arm-none-eabi-size` of the release image at
each release).

**Implementation status of the design constraints (2026-10-25).** Since 2026-10-24 the OBC flight software is Rust
(`software/rust/README.md`). On the OBC there is no RTOS: the image runs on the Embassy executor
(`software/flight/docs/RTOS_ARCHITECTURE.md`). SRS-C001 was amended on 2026-10-25 by PI decision (it required C11 only;
change request CR-2026-001, `docs/project/CHANGE_REQUEST_LOG.md`).

| ID | Status |
|----|--------|
| SRS-C001 | Met. The flight code is Rust (2024 edition), written to `docs/development/CODING_STANDARDS.md` §11. The C is `software/flight/src/flight_log_fmt.c` (C11, the printf-style flight-log shim, not linked into the image) and the C test suites |
| SRS-C002 | Met: no dynamic allocation at any time. Checked by the ctest `no_dynamic_memory` and by CI's check that the image links no `malloc`, `free` or `_sbrk` (`software/flight/docs/MEMORY_POLICY.md`) |
| SRS-C003 | Met by analysis: worst case 6,004 B of 8,192 B (73.3 %, 2026-10-27, after the CDR round-2 changes and the NMI handler), computed from the image's machine code by `scripts/stack_usage.py`, a CI gate (`software/flight/docs/MEMORY_POLICY.md` §6). At run time an MPU guard below the 8 KB faults an overrun. Measurement on hardware (HIL-044) pending |
| SRS-C004 | Release image text 160,004 B (2026-10-27, the CDR round-2 image; 115,876 B on 2026-10-25), below 256 KB (62 %). Checked at each release (`docs/development/RELEASE_PROCEDURE.md` §5.3); not a CI gate |

### 8.2 External Constraints

| ID | Constraint | Verification |
|----|------------|--------------|
| SRS-C010 | Beacon transmissions shall comply with amateur radio regulations | I, T |
| SRS-C011 | RF transmissions shall not occur until deployment timer expires | T |
| SRS-C012 | Callsign shall be included in all RF transmissions | T, I |

How each is verified: SRS-C010 by inspection (`docs/BEACON_SPEC.md` §6 against 47 CFR 97) and test (the content-filter
vectors); SRS-C011 by test (C suites `test_rf_silence`, `test_downlink`; SIL check `rf-silence`); SRS-C012 by test (C
suites `test_beacon_format`, `test_downlink`) and inspection.

---

## 9. Traceability

### 9.1 Parent Requirements

| SRS ID | Parent (SYS) | Description |
|--------|--------------|-------------|
| SRS-F012 | SYS-304 | Power mode implementation |
| SRS-F020 | SYS-210 | Beacon transmission |
| SRS-F030 | SYS-321 | AI payload interface |

### 9.2 Child Traceability

See Requirements Traceability Matrix (RTM) document for complete traceability to design and code.

---

## Appendix A: TBDs/TBRs

| ID | Item | Resolution Date |
|----|------|-----------------|
| TBD-SRS-001 | Exact beacon interval range | PDR |
| TBD-SRS-002 | Uplink command set definition | CDR |
| TBR-SRS-001 | Watchdog timeout value (currently 30s) | PDR |
| TBR-SRS-002 | SRS-F040 ("all fault conditions") cannot be tested; PI ratifies, amends or rejects the proposed wording beneath §3.5 (finding VV-16) | TBD |
| TBR-SRS-003 | Downlink values to confirm with the selected TNC and transmitter (SRS-I025 to SRS-I027, `downlink.h`): duty 25 %, TXDELAY 300 ms, VSWR 3:1 on 3 samples, 70/60 °C, 0.1 W output present, the monitor scaling (`firmware/src/adc.rs`) | EM transmitter test |

---

## Appendix B: Verification Methods

| Code | Method | Description |
|------|--------|-------------|
| A | Analysis | Mathematical or logical analysis |
| I | Inspection | Visual examination of documentation or hardware |
| D | Demonstration | Operational demonstration of function |
| T | Test | Execution with specific inputs and measured outputs |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | SMART-QSO Team | Appendix A TBD/TBR IDs made unique to this document: TBD-001, TBD-002 and TBR-001 became TBD-SRS-001, TBD-SRS-002 and TBR-SRS-001 (finding VV-17). Proposed testable wording, for the PI to ratify, beneath SRS-F040; requirement text unchanged; tracked as TBR-SRS-002 (finding VV-16) |
| 1.2 | 2026-10-16 | SMART-QSO Team | SRS-F018 added as [Proposed] (mode-transition logging, cited by state_machine.h); §9.1 parents corrected to defined system requirements (SYS-304, SYS-210, SYS-321) (findings VV-06, VV-19) |
| 1.3 | 2026-10-24 | SMART-QSO Team | The flight software is Rust: implementation status of SRS-C001 to SRS-C004 beneath §8.1 (SRS-C001 not met as written; PI decision); §2.1 OBC runs on the Embassy executor, not FreeRTOS. No requirement text changed |
| 1.4 | 2026-10-25 | SMART-QSO Team | SRS-C001 amended by PI decision to Rust (2024 edition) or C11 (was C11 only; CR-2026-001). Status beneath §8.1: SRS-C001 met; SRS-C003 met by analysis (static worst case 4,892 B, CI gate; hardware measurement pending) |
| 1.5 | 2026-10-25 | SMART-QSO Team | SRS-I023 to SRS-I027 added: the downlink transmitter interface, permission at hand-over, key-down and duty limits, power selection, transmitter protection (CR-2026-005) |
| 1.6 | 2026-10-27 | SMART-QSO Team (CDR flight-software review) | §3.9 transponder and commissioning (SRS-F080 to SRS-F088: gates, XPDR_EN heartbeat, commissioning hold, ground transponder enable, GNSS sessions, housekeeping frame, 120 s magnetorquer delay, RF-silence latch, deployment retries, SRS-F089 wings first and AntS over I2C); §3.10 ADCS (SRS-F090 to SRS-F094: magnetometer window, coil burst, stale rate, control laws at 8°/s, geometry presets by telecommand); §3.11 OBC selector (SRS-F100 to SRS-F107); verification methods for SRS-C001 to SRS-C012; governing standard NPR 7150.2D and NASA-STD-8739.8B (was NPR 7150.2 Rev B) |
| 1.7 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 2) | §3.12 added: end-of-life passivation (SRS-F110 to SRS-F112), receiver recovery and REL-04 (SRS-F115 to SRS-F117, with the 97.207(b) and D3 reasoning), stored housekeeping and DUMP_TLM (SRS-F120 to SRS-F122), command acknowledgement and contact count (SRS-F125, SRS-F126), beacon jitter (SRS-F127), wings-failed IDLE hold (SRS-F128), recharge-lockout clear (SRS-F129), heater +5/+9 °C (SRS-F130), Jetson cold inhibit −15 °C (SRS-F131), SRAM2 parity and NMI (SRS-F132), periodic check of persisted state (SRS-F133); SRS-Q013 safety-critical coverage; SRS-F081 lease at most 40 ms, drop within 90 ms (was 60 and 110 ms); SRS-F089 four burn lines, one resistor at a time; §8 constraint verification column reduced to the method codes, the detail moved beneath the tables; SRS-C003 status 6,004 B; SRS-C004 status 160,004 B |
