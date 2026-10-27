# EPS Controller Firmware: Software Requirements Specification

**Document Number**: SQSO-SRS-EPSC-001
**Version**: A
**Date**: 2026-10-27
**Status**: Draft (CDR). Requirements of a Class C software item that is not yet written (design starts with the
schedule task EPS_DES, `mission/SCHEDULE.md`). Values follow `docs/BASELINE_PARAMETERS.md` Rev E, which governs.
**Software item**: EPS controller firmware, on the MSP430FR2355 (U37) of the EPS board SQSO-EPS-BRD-001
**Classification**: NPR 7150.2D Class C (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1, §3.4); safety-critical
requirements flagged per NASA-STD-8739.8B §4.2 from `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` (SQSO-SHA-001 G,
SW-HAZ-016 and SW-HAZ-017)
**Closes**: CDR finding P3A-03 (EPS part) and NPR 7150.2D SWE-050 for this item (compliance matrix open item C-7)

---

## 1. Purpose and Scope

This specification gives the requirements of the firmware of the EPS board's controller. The controller is an I2C
target of the OBC at 0x48 on I2C2 and the master of the EPS board's local bus. By the board design no battery
protection depends on it (SQSO-EPS-BRD-001 §12): its duties are telemetry, the MPPT trim, the state of charge, one of
the two signals of the recharge-lockout clear and the backup set of the lockout, one of the two sources of the
end-of-life arm, the supervision of the OBC's heartbeat, the AntS supply line and the per-tracker disables.

Not in scope: the OBC's side of the interface (SMART-QSO-SRS-001: SRS-F010, SRS-F013, SRS-F014, SRS-F110, SRS-F111,
SRS-F129, SRS-F149, SRS-F155, SRS-F157, SRS-F162), the board hardware, and the battery's own protection.

## 2. Applicable Documents

| Document | Title | Used for |
|----------|-------|----------|
| SQSO-BL-001 Rev E | Baseline parameters (`docs/BASELINE_PARAMETERS.md`) | §4, §5 power and battery values; governs |
| SQSO-EPS-BRD-001 G | EPS board build-to design (`hardware/electrical/eps/EPS_BOARD_DESIGN.md`) | §4 trackers and MPPT, §5.3 charge window and NTC1 plausibility, §5.4 lockout, §6.1 AFE settings, §7 EOL disconnect, §8.3 switches and fault lines, §8.4 load shed, §9 burn drivers, §11 telemetry, §12 the controller, §13.2 OBC lines, §19 verification items |
| `hardware/electrical/eps/design.py` | EPS board schematic as code | Controller pin allocation (U37) |
| SMART-QSO-ICD-001 1.6 | OBC-EPS ICD (`docs/interfaces/ICD_OBC_EPS.md`) | §2.3 I2C, §4 register map and protocol, §4.6 extended registers and the passivation sequence, §5 formats, §7 polling, §8 errors, §9 timing, §10 tests |
| SQSO-EPS-BAT-001 2.10 | Battery design (`hardware/eps/BATTERY_DESIGN.md`) | §4 protection parts, §8.1 telemetry points and resolutions |
| SQSO-BSA-001 G | Battery safety assessment (`docs/environment/BATTERY_SAFETY_ASSESSMENT.md`) | H1 to H9 and their controls |
| SQSO-FMEA-003 B | Board FMEA (`docs/safety/BOARD_FMEA.md`) | BF-EPS-13, -15, -43, -48, -49, -50; fix (c); SMA-B09 |
| SMART-QSO-SRS-001 1.10 | Flight software requirements (`docs/software/SOFTWARE_REQUIREMENTS.md`) | The OBC side of each register |
| `software/rust/sqso-fsw/src/eps_telemetry.rs`, `software/rust/sqso-fsw/src/eps_supervision.rs`, `software/rust/sqso-ffi/src/eps_lockout.rs` | The OBC's decoder and writers as built | Block 0x00–0x33, plausibility, keys, the lockout-clear sequence |
| `software/rust/sqso-sil/src/hw/eps.rs` | The mission simulation's EPS controller model | STATUS thresholds and the SOC method the flight software is tested against |
| SMART-QSO-SDP-001 1.6 | Software development plan | §3.4: the item, its standards and verification |

## 3. Context

### 3.1 Where the firmware sits

```
 OBC I2C2 (PF1/PF0) ── P1.3 SCL / P1.2 SDA: target 0x48 ┐
                                                         │  EPS controller firmware (MSP430FR2355, 3V3_MCU via SW9)
 local bus P4.7 / P4.6 (master, 100 kHz) ─┬─ ISO1540 ─ BQ76952 AFE 0x08 (PACK domain)
                                          ├─ INA3221 ×4 0x40–0x43 (tracker inputs, bus load, rails, heater)
                                          └─ ADS7138 0x10 (panel NTCs, NTC1, VHW)
 SAC DACs: P1.1, P1.5, P3.1, P3.5 ─ 75 kΩ ─> MPPT set points of trackers T1–T4 (high-Z = 4.40 V default)
 ADC: IBURN W1A/W1B/W2A/W2B (P1.0, P1.4, P1.6, P1.7), IMON_JET (P5.0), F1_SNS (P5.1), V_BURN (P5.2), VTRK (P5.3)
 Inputs (100 kΩ series): AFE_ALERT, PERMIT, WIN_OK, SHED, EOL_ARMED, NTC1_BAD, FLT_SW1/EFUSE/SW2/SW3/SW4/SW5/SW6/SW7/SW8/SW11
 Outputs (100 kΩ pull-downs on the board): EOL_KEY P4.2, LOCK_SET P4.0, UNLOCK P4.1, OBC_CYCLE P6.1, ANTS_EN P6.0,
          TDIS_T1–T4 P6.2–P6.5, EPS_ALERT_N P4.4 (open drain)
```

Pins as allocated in `hardware/electrical/eps/design.py` (U37). The controller has no crystal (internal oscillators)
and is programmed only on the ground through J22 (Spy-Bi-Wire).

### 3.2 Functions enforced in hardware, which this firmware must not be relied on for

None of these depends on the firmware, and no requirement below may be read as making them depend on it:

| Function | Hardware | Source |
|----------|----------|--------|
| Charge regulation: CV 16.40 V, CC 0.394 A, the tracker output clamp 17.08 V | U45A, U44 and the trackers' FB dividers (VCTL domain, works with a dead bus) | SQSO-EPS-BRD-001 §5.1 |
| Charge-temperature window +1.1 … +44.0 °C, the heater cutout at +40 °C, the pack BOTHOFF at +60 °C, the NTC1 plausibility hand-over to NTC2 | TLV7042 comparators on NTC1, U66–U69 | §5.3 |
| Cell over-voltage, under-voltage, over-current, short circuit, temperature trips, the SUV permanent fail | BQ76952 OTP settings, written and verified at acceptance (V-EPS-2) | §6.1 |
| Secondary over-voltage protection | BQ771806 + F3 on the battery sense board | §6.1 |
| Fuses F0A, F0B, F0N, F2, PTC RT1 | — | §6.1 |
| Lockout memory and its set from the AFE's permanent fail; the clear needs LOCK_CLR (OBC) AND UNLOCK (this firmware) | K1 latching relay, U56/U57, U58/U59, series coil drivers | §5.4 |
| Hardware load shed of the Jetson (13.6 V for 5 s, release 14.4 V) | U48, U51, U65 | §8.4 |
| Load-switch current limits and latch-up retry, the Jetson eFuse at 1.20 A | TPS2553, TPS1H000/100, TPS16630 | §8.3 |
| Burn drivers: 0.8 A per resistor, the 12 s one-shot per line, the burn buck's 2.0 A cap | TPL5111 U70–U73, OPA2991 loops, U17 | §9 |
| End-of-life fire: the arm latch needs the OBC's EOL_ARM edge AND this firmware's EOL_KEY, is cleared at every power-up, and only the OBC's EOL_FIRE fires F1 | U61, U64, Q20, Q21 | §7 |
| The OBC power cycle lasts 1.95 s per rising edge of OBC_CYCLE, whatever the line does afterwards | TPL5111 U82, Q42 on SW10's enable | §12 |
| MPPT fallback: with the DACs high-impedance every string is held at 4.40 V | TLV9002 integrators | §4.2 |
| The deployment and RF inhibits DS1–DS3, the RBF and the tracker clamps Q40/Q41 | — | §6.2 |

If the firmware stops, the DACs go high-impedance, every output falls to its pull-down (no lockout set or clear, no
EOL arm, trackers enabled, no OBC power cycle, AntS on DEPLOY_EN alone), and the OBC keeps its analog lines and the two
INA228s (SQSO-EPS-BRD-001 §12). What depends on the firmware, and is therefore safety-critical below: the second
signal of the lockout clear and the backup set of the lockout (H2), the values the OBC's mode, heater and payload
guards use (SOC, the pack temperatures, IMON_JET), and the end-of-life key (passivation, SW-HAZ-013).

**At launch the controller is unpowered**: DS1–DS3 open remove VBAT and 3V3_AO (SQSO-EPS-BRD-001 §6.2), so the
firmware has no part in the launch-phase battery hazards.

## 4. Conventions

**Identifiers.** EPSC-Fnnn functional, EPSC-Innn interface, EPSC-Pnnn performance, EPSC-Cnnn constraints. Every row
is a requirement ("shall"), defined once. "Parent" names the requirement it flows from where one exists
(SQSO-EPS-BRD-001 EPS-F…, SMART-QSO-SRS-001 SRS-…, the system SYS-…); "derived" rows give their reason.

**Verification methods.** T test, A analysis, I inspection, D demonstration.

**Verification activities** (schedule tasks of `mission/SCHEDULE.md` in brackets):
- **UT**: host unit tests in CI of the hardware-independent modules (C, CMocka), with the register-map check against
  the flight software (task EPS_UT);
- **DB**: bench test on an MSP430FR2355 development board with the local-bus devices on evaluation boards and a pack
  simulator (task EPS_UT);
- **EM**: EPS board engineering model with the board verification items of SQSO-EPS-BRD-001 §19 (task EPS_EM);
- **FS**: EM integration with the OBC flatsat, the interface tests IFT-EPS-001 to -006 of SMART-QSO-ICD-001 §10
  (task EPS_EM, before EM_TEST);
- **FQ**: qualification of the flight build on the FM EPS board (task EPS_QUAL);
- **IN**: inspection of the code, the build and the release image.

**Safety-critical (SC).** "Yes" with the hazard where the requirement implements a hazard control of SQSO-SHA-001 G.
The modules that implement them take the safety-critical coverage rule (EPSC-C005).

**Proposed register additions.** Five requirements use registers that SMART-QSO-ICD-001 1.6 §4.6 does not yet list
(0x90–0x9F, 0xF2). They are proposed to the ICD owner in §9; the OBC does not read them yet, and no OBC function
depends on them.

## 5. Requirements

### 5.1 Start-up, clocks, watchdog and FRAM

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F001 | From any reset until its initialisation is complete, the firmware shall hold EOL_KEY, LOCK_SET, UNLOCK, OBC_CYCLE, ANTS_EN and TDIS_T1–T4 low, keep the four SAC DAC outputs high-impedance and EPS_ALERT_N released | — | Each output's safe state is its pull-down; the DACs' high impedance is the 4.40 V MPPT fallback (SQSO-EPS-BRD-001 §4.2, §12) | T, I | UT (start-up sequence); EM: logic analyser on every output through 100 power cycles and SW9 trips | Yes (SW-HAZ-016, SW-HAZ-017) |
| EPSC-F002 | Within 2 s of any reset the firmware shall fill every measurement register from a complete acquisition; until then they shall read 0x00 | SRS-F010 | The OBC treats a pack voltage outside 9.0–17.6 V as no live controller and falls back to its own sensors (`VBATT_PLAUSIBLE_V` in `eps_telemetry.rs`) | T | UT; FS: OBC reads during the controller's start | — |
| EPSC-F003 | The watchdog (WDT_A) shall be running before any output is driven, with a timeout of at most 1.5 s worst case with its clock source, serviced only from the main loop after a completed pass, and the flight build shall never stop it | — (derived) | Bounds a hung controller's outputs: EOL_KEY (EPSC-F071), UNLOCK (EPSC-F062) and LOCK_SET fall to their pull-downs at the reset (SQSO-FMEA-003 fix (c): "the ≤ 10 s EOL_KEY limit is firmware") | T, I | IN: no WDTHOLD write after initialisation; EM: main loop halted by the test build, reset within 1.5 s | Yes (SW-HAZ-016, SW-HAZ-017) |
| EPSC-F004 | After any reset (watchdog, SW9 trip and retry, brown-out, power-up) the firmware shall start as from power-up, record the reset cause (register 0x9D, proposed) and increment BOOT_CNT (0x50) in FRAM | — | SMART-QSO-ICD-001 §4.2 BOOT_CNT; SW9 power-cycles the controller after a latch-up (SQSO-EPS-BRD-001 §8.3) | T | UT; EM: V-EPS-10 on SW9 | — |
| EPSC-F005 | The firmware shall keep its persistent data (BOOT_CNT, the tracker mask, the SOC state, the OBC-cycle history, ENERGY, the calibration constants, the AFE acceptance record) in FRAM in two copies, each with a CRC-16, write the second only after the first reads back correctly, take defaults when both fail (mask 0, no cycles, SOC from EPSC-F041, energy 0, calibration defaults flagged in register 0x9C, proposed), and protect its code and constants against writes with the FRAM memory protection | — (derived) | A reset or an upset during a write must not lose or corrupt the state; FRAM is the controller's only memory across resets | T, I | UT (interrupted writes, both copies corrupted); IN: memory-protection setting of the release image | — |
| EPSC-F006 | The firmware shall clock the CPU and peripherals from the internal DCO locked to REFO at an integer number of megahertz, with no clock output | — (derived) | Harmonics of integer-MHz clocks fall on integer MHz, which the uplink windows avoid by ≥ 200 kHz (SQSO-EPS-BRD-001 §8.2, EMC rule E1) | I, T | IN; EM: V-EPS-11 with the firmware running | — |

### 5.2 I2C target interface (0x48 on I2C2) and alert

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-I001 | The firmware shall answer at the 7-bit address 0x48 on I2C2 (eUSCI_B0, P1.2 SDA, P1.3 SCL) in standard (100 kHz) and fast (400 kHz) mode, and shall never act as a master on I2C2 | — | SMART-QSO-ICD-001 §2.3; never a master: the OBC owns the bus, and a second master could block the gauge, the lockout clear and EOL_KEY (SQSO-FMEA-003 BF-EPS-50) | T | UT; FS: IFT-EPS-001 at both speeds | — |
| EPSC-I002 | The firmware shall implement the write (register, data) and read (register, repeated START, data) transactions of SMART-QSO-ICD-001 §4.1 with an auto-incrementing register pointer; unmapped addresses shall read 0x00; a read of 52 bytes from 0x00 shall return 0x00–0x33 | — | SMART-QSO-ICD-001 §4.1; the OBC reads the block 0x00–0x33 in one transaction (`BLOCK_LEN` in `eps_telemetry.rs`) | T | UT; FS: IFT-EPS-002 | — |
| EPSC-I003 | Every read transaction shall return values from one acquisition cycle, with no multi-byte value split between two updates | — (derived) | A torn 16-bit value or a mixed block would pass the OBC's plausibility check with a wrong value | T | UT (reads concurrent with updates) | — |
| EPSC-I004 | Multi-byte values shall be little-endian in the units of SMART-QSO-ICD-001 §5: voltages in mV (unsigned 16 bits), currents in mA (signed 16 bits, positive charging), temperatures in 0.1 °C (signed 16 bits), SOC in % (8 bits), energy in mWh (32 bits) | — | SMART-QSO-ICD-001 §5 | T | UT against the decoder `eps_telemetry::decode` | — |
| EPSC-I005 | Writes to read-only or unmapped registers shall have no effect; a write of a MODE value above 0x02 shall be refused with a NACK; every accepted write shall take effect within 10 ms | — | SMART-QSO-ICD-001 §6.1, §9.1 (transaction ≤ 10 ms) | T | UT; FS: IFT-EPS-003 | — |
| EPSC-I006 | The firmware shall never hold SCL low for more than 1 ms, and shall release SDA and SCL and reset its I2C target within 25 ms of a stalled transaction or a bus held low; a target reset shall change no output | — (derived) | I2C2 carries the gauge, the lockout clear and EOL_KEY: a controller that holds the bus blocks them (SQSO-FMEA-003 BF-EPS-50, Category 3 for EOL_KEY); the OBC times out at 100 ms and recovers the bus (SMART-QSO-ICD-001 §8.1, §8.2) | T | UT; FS: IFT-EPS-006 with a transaction cut mid-byte and SDA held by a fixture | Yes (SW-HAZ-017) |
| EPSC-I007 | The firmware shall assert EPS_ALERT_N (P4.4) while VBATT is below the low-voltage threshold of ALERT_CFG or TBATT above its high-temperature threshold, or when a new fault flag latches in STATUS or CHG_INHIBIT, and release it when the OBC has read STATUS and the condition has cleared; the thresholds at reset shall be 13.80 V and +45.0 °C | — | SMART-QSO-ICD-001 §6.3, §8.3, §9.2; defaults derived: 0.2 V above the hardware load shed (13.6 V) so the OBC hears first, and the top of the charge window | T | UT; FS: IFT-EPS-005 | — |
| EPSC-I008 | The firmware shall store ALERT_CFG (0x60, 4 bytes) when written, apply it to EPSC-I007, and return to the defaults at every reset | — | SMART-QSO-ICD-001 §4.2, §6.3 | T | UT | — |
| EPSC-I009 | The register addresses, keys, bit positions and units shall come from one machine-readable definition, checked in CI against the flight software's constants (`software/rust/sqso-fsw/src/eps_telemetry.rs`, `software/rust/sqso-fsw/src/eps_supervision.rs`, `software/rust/sqso-ffi/src/eps_lockout.rs`) | — (derived) | The two ends of the interface must not drift (as `sqso-protocol` does for the KISS link) | I, T | UT: the CI check fails on any difference | — |
| EPSC-I010 | The firmware shall report its version at VERSION (0xF0, 2 bytes) and the CRC-16 of its image, computed at start-up, at 0xF2 (2 bytes, proposed) | — | SMART-QSO-ICD-001 §4.2; configuration status accounting of the flight image (SWE-083) | T | UT; FS | — |

### 5.3 Telemetry

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F010 | The firmware shall read the BQ76952 (0x08, through the ISO1540), the four INA3221s (0x40–0x43) and the ADS7138 (0x10) on its local bus at 100 kHz; on a stuck bus it shall send nine clocks and a STOP and re-initialise; a device that fails three consecutive transactions shall set STATUS.FAULT and register 0x9C (proposed) bit 5, and the registers it feeds shall read 0 until it answers again | EPS-F013 | SQSO-EPS-BRD-001 §11; a 0 pack voltage makes the OBC fall back to its own sensors (EPSC-F002) rather than use a stale value | T | UT; DB; EM: each device held in reset | — |
| EPSC-F011 | The firmware shall fill the registers from these sources: VBATT (0x10) the AFE's stack voltage; IBATT (0x12) the AFE's pack current on RS7, positive charging; VCELL1–4 (0x70–0x77) the AFE's cell voltages; TBATT (0x14) NTC1 through the ADS7138, or NTC2 (the AFE's TS1) while NTC1_BAD reads high; TBATT2 (0x78) NTC2; VSOLAR (0x20) the highest tracker input voltage; ISOLAR (0x22) the sum of the four tracker input currents; PSOLAR (0x24) and PTRACK1–4 (0x80–0x87) the tracker input powers; VBUS (0x30) and IBUS (0x32) the 5V_PA rail; IHEATER (0x88) the heater current (RS11); all through the INA3221s and the ADS7138 as SQSO-EPS-BRD-001 §11 allocates them | EPS-F013 | SMART-QSO-ICD-001 §4.2, §4.6; SQSO-EPS-BRD-001 §11. TBATT follows the hardware's hand-over to NTC2 (§5.3) so the OBC's heater thermostat never loses its temperature | T | UT; EM: each source against a reference; FS: IFT-EPS-002, INT-EPS-002 | — |
| EPSC-F012 | The firmware shall refresh every measurement register at least every 500 ms, and IMON_JET (0x8A) and F1_SNS (EPSC-F015) every 100 ms | — | The OBC polls at 1 Hz (SMART-QSO-ICD-001 §7.2); the OBC's guard on IMON_JET acts in 1–2 s (SQSO-JET-BRD-001 §12, latch-up coverage) | T | UT; DB: timing probe | — |
| EPSC-P001 | Voltages and currents shall be within ±2 % of reading or ±10 mV and ±10 mA, whichever is larger, from −40 to +70 °C after the acceptance calibration | EPS-F013 | INT-EPS-002, telemetry accuracy ±2 % of reference (SMART-QSO-ICD-001 §10.2); resolutions 10 mV and 10 mA (SQSO-EPS-BAT-001 §8.1) | T | EM: against calibrated references at −40, +25 and +70 °C (FS: INT-EPS-002); FQ | — |
| EPSC-P002 | TBATT and TBATT2 shall be within ±1.0 °C of the pack temperature at their NTC from −20 to +60 °C, the NTCs' own tolerance included | SRS-F130 | The OBC's heater thermostat (+5 °C on, +9 °C off) keeps the coldest cell at +4.4 °C with margin for the sensor's error (`heater.rs`, THM-02); the heater is BSA H5 control 3 and H6 control 1 | T | EM: pack thermocouples against TBATT/TBATT2 in a chamber at −20, +5, +25, +45 and +60 °C | Yes (SW-HAZ-016) |
| EPSC-F013 | IMON_JET (0x8A) shall report the Jetson eFuse U19's IMON (0.84 V/A at P5.0) in mA, averaged over at least 4 samples, within ±3 % of reading ±20 mA of the IMON voltage's value after the acceptance two-point calibration (the eFuse's own IMON tolerance excluded) | SRS-F149 | The OBC's payload guard (8 W to 12 W, safety-critical `payload_guard.rs`, SDP §3.2) uses IMON_JET × VJET_RB (SMART-QSO-ICD-002 §10.5; SQSO-JET-BRD-001 §12 and §19 item F3); the 1.20 A eFuse trip is independent of it | T | UT (scaling); EM: V-EPS-13 set-up, Jetson load simulator 0.1–1.2 A | Yes (SW-HAZ-016) |
| EPSC-F014 | The firmware shall measure the four burn currents (P1.0, P1.4, P1.6, P1.7), V_BURN (P5.2) and VTRK (P5.3) every 100 ms and report them at 0x90–0x9B (proposed: four currents in mA, then V_BURN and VTRK in mV); a burn current above 0.4 A for more than 13 s shall set STATUS.FAULT and register 0x9C bit 3. The firmware has no output that starts or stops a burn | — (derived) | The one-shots (12.0 ± 0.6 s), the 0.8 A drivers and the buck limit are hardware (SQSO-EPS-BRD-001 §9); the measurement shows the ground a driver that runs past its one-shot | T | UT; EM: V-EPS-6 | — |
| EPSC-F015 | The firmware shall classify F1_SNS (P5.1, 1 MΩ / 220 kΩ from F1's heater pin) as intact or open with a band for each, and report a reading between the bands as neither (EOL_STATE bits 1 and 2 both 0) and flag it in register 0x9C | — | F1_SNS tells whether F1 is intact (SQSO-EPS-BRD-001 §7); the OBC's PASSIVATE_ARM requires "intact" and its fire confirmation "open" (SMART-QSO-ICD-001 §4.6); an ambiguous reading must satisfy neither | T | UT; EM: F1 intact, F1 removed, the sense line opened; V-EPS-8 (d) EOL fire on the EM | Yes (SW-HAZ-017) |

### 5.4 Status, mode and fault flags

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F020 | STATUS (0x00) shall set CHARGING (bit 7) while IBATT exceeds +5 mA, SUNLIT (bit 6) while PSOLAR exceeds 0.3 W, LOW_BATT (bit 5) while SOC is below 25 %, OVER_TEMP (bit 4) while TBATT is above +45.0 °C, UNDER_TEMP (bit 3) while TBATT is below 0.0 °C, FAULT (bit 2) per EPSC-F010, F014 and F024 or an AFE fault or permanent fail, and bits 1–0 to MODE | SRS-F010 | SMART-QSO-ICD-001 §4.3; the thresholds of the mission simulation's controller model (`telemetry_now` in `software/rust/sqso-sil/src/hw/eps.rs`), against which the flight software is tested | T | UT (each threshold); FS | — |
| EPSC-F021 | The firmware shall store MODE (0x01, values 0x00–0x02) and LOAD_EN (0x02) when written and return them when read, with no other effect: it drives no load enable | — | The load switches are driven by the OBC's own lines (SMART-QSO-ICD-001 §3.2; SQSO-EPS-BRD-001 §13.2); §4.5 and §6.2 of the ICD still describe LOAD_EN as load enables (§9 item 2) | T | UT; FS: IFT-EPS-003, IFT-EPS-004 (the loads respond to the OBC's lines, not to LOAD_EN) | — |
| EPSC-F022 | CHG_INHIBIT (0x7A) shall report: bit 0 cold and bit 1 hot, when WIN_OK reads low with TBATT below +1.1 °C or above +44.0 °C; bit 2 a cell over-voltage from the AFE; bit 3 an AFE fault or permanent fail, or its settings disagreeing with the acceptance record (EPSC-F063); bit 4 the secondary protector's fuse blown (cell voltages present at the AFE with BAT_F absent); bit 5 the recharge lockout latched (PERMIT low while the charge window reads open, WIN_OK or NTC1_BAD high); bit 6 NTC1_BAD high | — | SMART-QSO-ICD-001 §4.6; SQSO-EPS-BRD-001 §5.3, §5.4, §6.1 | T | UT; EM: V-EPS-9, V-EPS-12, V-EPS-14 with the register read | — |
| EPSC-F023 | LOAD_SHED (0x7B) shall report SHED (bit 0) and the fault lines of SW1 (bit 1) and SW3 (bit 2); the firmware shall take no part in the shed | EPS-F012 | SMART-QSO-ICD-001 §4.6; the shed is hardware (SQSO-EPS-BRD-001 §8.4) | T | UT; EM: V-EPS-7 with the register read | — |
| EPSC-F024 | The firmware shall sample every switch fault line (FLT_SW1–SW8, FLT_SW11, FLT_EFUSE) at least every 10 ms, count each assertion longer than 10 ms, and latch STATUS.FAULT until the OBC has read STATUS | — | Every switch's fault state is read by the controller and reported (SQSO-EPS-BRD-001 §8.3); a latch-up retry lasts 0.28 s or more, so a 10 ms sample sees each trip | T | UT; EM: V-EPS-10 with the register read | — |

### 5.5 State of charge and energy

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F040 | The firmware shall compute SOC (0x16) by integrating IBATT (the AFE's pack current) against a learnt capacity: at each end of charge (pack at or above 16.30 V with the charge current below 20 mA) it shall reset the count to the charge that the end-of-charge voltage gives on the cell's OCV curve and update the capacity estimate with a weight of 0.1; SOC shall be clamped to 0–100 % | SRS-F010 | The method of the mission simulation's gauge model (`gauge_step` in `software/rust/sqso-sil/src/hw/eps.rs`); SQSO-EPS-BAT-001 §8.1 (SOC calculated, 1 %). From the AFE, on the controller's own bus: the INA228 gauge (0x44) is on I2C2, where the controller is only a target (EPSC-I001), and stays the OBC's independent gauge | T | UT (recorded and synthetic orbit profiles, 2 years of ageing); DB with a pack simulator; EM | Yes (SW-HAZ-016) |
| EPSC-F041 | When the FRAM's SOC state is invalid, the firmware shall start SOC from the lowest cell voltage through the OCV table and set register 0x9C bit 0 (SOC uncalibrated) until the first end of charge | — (derived) | A controller with lost state must still give the OBC a conservative SOC | T | UT | Yes (SW-HAZ-016) |
| EPSC-F042 | After the first end of charge, SOC shall be within ±5 % (absolute) of the pack's true state of charge throughout the mission, ageing included | SRS-F013, SRS-F014 | Derived: the flight software's mode thresholds are 15 points apart (SAFE 25 %, IDLE 40 %, payload and transponder 55 %; SRS-F013 to F015, F080); BSA H2 control 1. The hardware controls of H2 do not depend on it | T, A | UT (error over simulated mission profiles); EM: cycling the EM pack with a reference coulomb counter; A: error budget | Yes (SW-HAZ-016) |
| EPSC-F043 | While the lowest cell voltage is below 3.30 V, the SOC reported shall not exceed the OCV-table value for that voltage plus 5 % | — (derived) | A drifting coulomb count must not hide a low pack from the OBC's SAFE threshold (BSA H2 control 1) | T | UT | Yes (SW-HAZ-016) |
| EPSC-F044 | ENERGY (0x40) shall accumulate the energy delivered to the bus loads (VBAT × the bus-load current) in mWh and be saved to FRAM at least every 10 min | — | SMART-QSO-ICD-001 §4.2 | T | UT; EM | — |

### 5.6 MPPT

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F050 | For each enabled tracker the firmware shall run perturb-and-observe on its SAC DAC, from the tracker's input power (INA3221), with a step and period that give a tracking efficiency of at least 98 % at constant irradiance and at least 95 % with irradiance ramps of 10 % per second | EPS-F002 | SQSO-EPS-BRD-001 §4.2 (perturb-and-observe through 75 kΩ from the SAC DAC), §4.3 (the power closure assumes 98–99 % tracking); the ramp case derived from the spinning, sun-pointed attitude | T | DB with a solar-array simulator; EM: V-EPS-1, V-PWR-3; FQ | — |
| EPSC-F051 | The firmware shall command no string set point outside 3.3–5.4 V | EPS-F002 | The DAC's range through 75 kΩ (SQSO-EPS-BRD-001 §4.2) | T | UT; EM | — |
| EPSC-F052 | While the CV or CC loop governs (pack at or above 16.30 V, or charge current at or above 0.38 A) the firmware shall hold each set point and resume tracking when the loop releases | — (derived) | The three loops are a wired minimum on COMP: while CV or CC governs, a perturbation changes nothing the tracker can observe, and an unheld search would walk the set point to a limit (SQSO-EPS-BRD-001 §4.2, §5.1) | T | UT; DB; EM: charge to CV with the array simulator | — |
| EPSC-F053 | A tracker whose input power is below 50 mW, whose INA3221 does not answer, or which is disabled by TRK_DISABLE shall have its DAC output high-impedance | — | The analog 4.40 V default is the safe set point for a dark string and for a missing measurement (SQSO-EPS-BRD-001 §4.2, §12) | T | UT; EM | — |

### 5.7 Recharge lockout

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F060 | The firmware shall raise LOCK_SET (P4.0) for 1.0 ± 0.1 s when the AFE reports its Safety Cell Undervoltage permanent fail, or when every AFE reading over 5 s shows a cell below 2.00 V with each reading plausible (a valid transaction, four cells read, their sum within 5 % of the stack voltage); once per event, a new event needing every cell at or above 2.50 V and no permanent fail in between; each event recorded in FRAM and in register 0x9C bit 6 | EPS-F011 | The backup set path of the latched lockout (SQSO-EPS-BRD-001 §5.4: "the EPS controller's LOCK_SET, which it raises if it reads the PF or a cell < 2.0 V"); the only set path if the AFE's SUV detection fails (SQSO-FMEA-003 BF-EPS-15); the 5 s matches the AFE's SUV delay; U57 needs ≥ 50 ms; BSA H2 control 3 | T | UT (thresholds, plausibility, event logic); EM: V-EPS-9 (set by LOCK_SET with an emulated cell below 2.0 V) | Yes (SW-HAZ-016) |
| EPSC-F061 | The firmware shall raise LOCK_SET for no other reason | — (derived) | A spurious set re-latches K1 at every power-up while it persists: permanent charge inhibit (SQSO-FMEA-003 BF-EPS-49) | T, I | UT; IN: LOCK_SET driven at one place | Yes (SW-HAZ-016) |
| EPSC-F062 | On a write of 0xC5 to LOCK_UNLOCK (0x7D) the firmware shall raise UNLOCK (P4.1) for 1.0 ± 0.1 s and then lower it; any other value shall lower it at once; it shall be low at every reset; each raise needs its own write | SRS-F129 | UNLOCK is the second of the two conditions of the lockout clear (with the OBC's LOCK_CLR; SQSO-EPS-BRD-001 §5.4). The OBC writes the key and pulses LOCK_CLR for 150 ms in the same pass (`eps_lockout.rs`), so 1 s covers it; ICD-001 1.6 lets UNLOCK stay high until another value is written, which would leave every later clear to the one LOCK_CLR line (§9 item 3) | T | UT; EM: V-EPS-9 (clear needs both lines; LOCK_CLR alone 2 s after an unlock does nothing) | Yes (SW-HAZ-016) |
| EPSC-F063 | The firmware shall send the BQ76952 only read commands and the subcommands that read its data memory and status: it shall not enter CONFIG_UPDATE, write data memory or OTP, change its security state, reset a permanent fail or command its FETs, and the image shall contain no unseal or full-access key. Every 10 min it shall read the AFE's protection settings (COV, CUV, OCD1, SCD, OCC, UTC, OTC, OTD, SUV) and compare them with the acceptance record in FRAM, reporting a difference in CHG_INHIBIT bit 3 and register 0x9C bit 4 without any corrective write | EPS-F011 | No battery protection may depend on the controller (SQSO-EPS-BRD-001 §12), so it must not be able to weaken one; the settings are written and verified at acceptance (§6.1, V-EPS-2); a RAM-shadow upset of the AFE is a known cause (SQSO-FMEA-003 BF-EPS-13) | T, I | UT (the command set admits no write); IN: no key or write command in the image; EM: an AFE setting changed on the bench is reported within 10 min | Yes (SW-HAZ-016) |

### 5.8 End-of-life key

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F070 | The firmware shall raise EOL_KEY (P4.2) and set EOL_STATE bit 3 only on a write of 0x3C to EOL_KEY (0x7E) followed within 1.0 s by a write of 0xC3, and only if, at the second write, EOL_ARMED reads low and F1_SNS reads intact | SRS-F157 | SMART-QSO-ICD-001 §4.6 (EOL_KEY row); SQSO-EPS-BRD-001 §7, §12 (RID P3E-11): the arm needs a second, independent source besides the OBC's GPIOs | T | UT (every order, timing and precondition); FS: the passivation sequence with the OBC up to the arm (the latch cleared by power cycling); EM: V-EPS-8 (d) | Yes (SW-HAZ-017) |
| EPSC-F071 | The firmware shall lower EOL_KEY and clear bit 3 at the first of: 10.0 s after the raise; a write to 0x7E other than the expected next byte; EOL_ARMED reading high; any reset. EOL_KEY shall never be high for more than 10.0 s while the firmware runs, nor for more than 11.5 s with a hang (EPSC-F003) | SRS-F157 | The 10 s window of the ICD; lowering once the latch is set removes the key the moment it is no longer needed; the limit is firmware, and a key left high returns the arm to a single OBC edge (SQSO-FMEA-003 fix (c)) | T | UT; EM: logic analyser over 1,000 sequences; the hang case with the test build | Yes (SW-HAZ-017) |
| EPSC-F072 | A refused sequence (wrong value or order, more than 1.0 s between the writes, or a precondition false) shall leave EOL_KEY low and bit 3 clear, and be counted in register 0x9E (proposed) | SRS-F157 | SMART-QSO-ICD-001 §4.6 ("a refused sequence leaves bit 3 = 0") | T | UT | Yes (SW-HAZ-017) |
| EPSC-F073 | EOL_STATE (0x7C) shall report EOL_ARMED (bit 0), F1 open (bit 1) and F1 intact (bit 2) from EPSC-F015, and EOL_KEY high (bit 3), from samples at most 100 ms old | SRS-F155 | The OBC's arm checks bits 0 and 2 first, its fire confirmation is bit 1 set with bit 2 clear, and the final report carries the register (SMART-QSO-ICD-001 §4.6; SRS-F110, SRS-F111, SRS-F155) | T | UT; FS; EM: V-EPS-8 (d) | Yes (SW-HAZ-017) |

### 5.9 OBC heartbeat supervision

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F080 | A write of 0x5A to OBC_HEARTBEAT (0x8C) shall count as a heartbeat; any other value shall be ignored | SRS-F162 | SMART-QSO-ICD-001 §4.6 | T | UT | — |
| EPSC-F081 | After 600–660 s without a heartbeat, and not earlier than 600 s after the controller's start or its last cycle, the firmware shall raise OBC_CYCLE (P6.1) for 2.5 ± 0.3 s and then lower it | SRS-F162 | SMART-QSO-ICD-001 §4.6 (600 s); the one-shot U82 is powered by the line and needs it for its 2.0 s (SQSO-EPS-BRD-001 §12); the upper bound allows the internal clock's tolerance. The OBC's hang that no reset clears is otherwise detected by nothing (SQSO-FMEA-003 BF-OBC-02, SMA-B09) | T | UT (timers); EM: V-EPS-16 | Yes (SW-HAZ-017) |
| EPSC-F082 | The firmware shall make at most 3 cycles in any 24 h, then stop the supervision and set EPS_SUPV bit 7; 10 consecutive heartbeats shall clear the count and bit 7; the history shall be kept in FRAM so that a controller reset neither restarts the supervision early nor loses the count | SRS-F162 | SMART-QSO-ICD-001 §4.6; FRAM derived | T | UT; EM: V-EPS-16 (four stops in 24 h) | Yes (SW-HAZ-017) |
| EPSC-F083 | No command shall disable the supervision | SRS-F162 | SMART-QSO-ICD-001 §4.6 ("No command disables it") | T, I | UT; IN | — |
| EPSC-F084 | EPS_SUPV (0x8F) shall report the disabled trackers (bits 0–3), ANTS_EN from the controller (bit 4), the cycles in the last 24 h (bits 5–6) and the supervision stopped (bit 7) | SRS-F162 | SMART-QSO-ICD-001 §4.6 | T | UT; FS | — |

### 5.10 AntS supply and tracker disables

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-F090 | A write of 0xA6 to ANTS_PWR (0x8D) shall raise ANTS_EN (P6.0) for at most 1,800 s (−0/+60 s), then lower it; any other value shall lower it at once; it shall be low at every reset | SRS-F162 | SMART-QSO-ICD-001 §4.6 (RID SMA-B06); SW8 is downstream of DS1–DS3 and the AntS deploys only on the OBC's own I2C3 commands (SQSO-EPS-BRD-001 §9) | T | UT; EM | — |
| EPSC-F091 | A two-byte write to TRK_DISABLE (0x8E) of the key 0x96 and a mask (bits 0–3 disable T1–T4) shall set TDIS_T1–T4 (P6.2–P6.5) within 10 ms and store the mask in FRAM; a write without the key shall be ignored; at start-up the stored mask shall be applied, or none if both copies fail | SRS-F162 | SMART-QSO-ICD-001 §4.6 (RID SMA-B17: a tracker with a shorted output diode is switched off) | T | UT; EM: V-EPS-17 | — |
| EPSC-F092 | A mask that disables all four trackers, or sets a bit above bit 3, shall be refused (NACK, no change) | — (derived) | With all four off the pack is never charged again: a ground error must not end the mission (§9 item 4, proposed to the ICD owner) | T | UT | — |

### 5.11 Constraints

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| EPSC-C001 | The firmware shall be written in C11 (ISO/IEC 9899:2011) to MISRA C:2012, with a recorded deviation list, and the JPL D-60411 rules of `docs/development/CODING_STANDARDS.md`, compiled with TI's MSP430-GCC (`msp430-elf-gcc`) at a version pinned when coding starts and recorded in the software version description, without an RTOS | — | §7 | I | IN: build configuration and CI toolchain pin | — |
| EPSC-C002 | The hardware-independent logic (register map and keyed registers, timers, SOC, MPPT decisions, lockout, EOL key and supervision) shall be in modules built both for the target and for the host, and tested on the host with the project's CMocka suites in CI | — | Testable on every change, as the flight software (SMART-QSO-SDP-001 §8) | I, T | IN; UT | — |
| EPSC-C003 | The firmware shall allocate no dynamic memory, use no recursion and bound every loop | — | `docs/development/CODING_STANDARDS.md` (JPL Power of Ten rules 1–3) | I, A | IN; A: map file and call graph | — |
| EPSC-C004 | Code and constants shall use at most 75 % of the 32 KB FRAM, RAM (static plus the worst-case stack by analysis) at most 75 % of the 4 KB SRAM, and the main loop's worst-case period shall be at most 100 ms | — (derived) | 25 % margin at CDR; the 100 ms loop serves EPSC-F012 and F024 | A, T | A: map file, stack analysis; DB: loop-period probe | — |
| EPSC-C005 | The modules that implement the SC requirements shall reach 100 % MC/DC on the host with GCC's condition coverage (`-fcondition-coverage`, GCC 14 or later), every uncovered condition justified; the other modules at least 90 % of lines | — | SWE-219 met as written for this item (GCC measures MC/DC; the OBC's Rust cannot, CR-2026-003) | A | UT: the coverage gate in CI | — |
| EPSC-C006 | The firmware shall build without warnings at `-Wall -Wextra -Werror`, and a MISRA C:2012 checker shall run on every change | — | `docs/development/CODING_STANDARDS.md` Rule 10; SWE-135 | I | IN: CI configuration | — |
| EPSC-C007 | The flight build shall have no path that changes its own image (no call into the bootstrap loader, no FRAM program write outside EPSC-F005's data areas); the controller shall be programmed only on the ground through J22 | — | SQSO-EPS-BRD-001 §12 ("Programming through J22 (Spy-Bi-Wire), ground only"); no in-flight update path (SWE-075 row) | I | IN: link map and memory-protection setting of the release image | — |
| EPSC-C008 | The firmware shall be its own configuration item with a version number, built and tested in CI on every change, and released under the release procedure | — | SMART-QSO-SDP-001 §3.4, §7; SWE-187 | I | IN: `docs/software/CONFIGURATION_INDEX.md` entry when the item is created | — |

## 6. Requirement Count and Safety-Critical Requirements

| Group | Requirements | Safety-critical |
|-------|-------------:|----------------:|
| §5.1 Start-up, clocks, watchdog, FRAM | 6 | 2 |
| §5.2 I2C target interface and alert | 10 | 1 |
| §5.3 Telemetry | 8 | 3 |
| §5.4 Status, mode, fault flags | 5 | 0 |
| §5.5 State of charge and energy | 5 | 4 |
| §5.6 MPPT | 4 | 0 |
| §5.7 Recharge lockout | 4 | 4 |
| §5.8 End-of-life key | 4 | 4 |
| §5.9 OBC heartbeat supervision | 5 | 2 |
| §5.10 AntS supply and tracker disables | 3 | 0 |
| §5.11 Constraints | 8 | 0 |
| **Total** | **62** | **20** |

Methods: T on 56 requirements (alone or with another method), I on 14, A on 4, D on none.

The safety-critical requirements implement the controls of SQSO-SHA-001 G: SW-HAZ-016 (the battery hazard controls
and the values the OBC's guards use: EPSC-F001, F003, P002, F013, F040 to F043, F060 to F063) and SW-HAZ-017 (the
end-of-life key, the bus and the supervisor outputs: EPSC-F001, F003, I006, F015, F070 to F073, F081, F082).

## 7. Language and Toolchain

**Recommendation: C11, to MISRA C:2012, with TI's MSP430-GCC** (for the PI's ratification, dated action DA-22 in
`mission/SCHEDULE.md`).

Why not Rust here, honestly:
- The `msp430-none-elf` target is a Tier 3 target of the Rust compiler: no official prebuilt standard library, built
  with nightly `build-std`, and nothing in the Rust project's CI guarantees that it builds or that its code is right.
  The project's pinned stable toolchain (1.97.0, CR-2026-004) cannot build it, so the item would need a second,
  unpinned nightly compiler.
- Interrupt handlers on MSP430 need an unstable compiler feature (the MSP430 interrupt ABI), the hardware abstraction
  for the FR2355 is a small community crate, and the link step still needs TI's `msp430-elf-gcc` toolchain.
- The coverage tooling the project relies on does not exist for that target; host tests would cover the logic, as
  in C.
- The benefit Rust gives the TNC (sharing `sqso-protocol`) does not arise: the EPS register map is small and is shared
  by a generated check instead (EPSC-I009).

Why C fits:
- TI supports MSP430-GCC and its own compiler for the MSP430FR2355, with DriverLib and data-sheet examples; the
  toolchain is mature and can be pinned like the Rust one.
- The project already has a C coding standard (MISRA C:2012 and JPL D-60411, `docs/development/CODING_STANDARDS.md`),
  CMocka suites and CI for C.
- GCC 14 measures MC/DC (`-fcondition-coverage`), so SWE-219 can be met as written for the safety-critical modules
  of this item, without the CR-2026-003 tailoring the OBC's Rust needs.
- Risk accepted with C: memory safety rests on MISRA, static analysis and review instead of the language; the item is
  small (62 requirements, 32 KB of FRAM) and has no dynamic memory (EPSC-C003).

## 8. Traceability

The requirements of this specification are rows of the generated RTM (`docs/verification/RTM.csv`,
`scripts/gen_rtm.py`, specification `docs/software/EPS_CONTROLLER_FIRMWARE_REQUIREMENTS.md`) and of the VCRM's
generated block (`docs/verification/VERIFICATION_MATRIX.md`); `scripts/check_traceability.py` checks them with the
other specifications. The C code and its tests will tag them with `@requirement EPSC-…`, which
`scripts/check_traceability.py` reads from `software/` like the flight software's tags.

Upward, by parent:

| Parent | Requirements |
|--------|--------------|
| EPS-F002 (MPPT) | EPSC-F050, F051 |
| EPS-F011 (battery protection per SQSO-BSA-001) | EPSC-F060, F063 |
| EPS-F012 (hardware load shed) | EPSC-F023 |
| EPS-F013 (telemetry) | EPSC-F010, F011, P001 |
| SRS-F010, SRS-F013, SRS-F014 (SOC and the mode thresholds) | EPSC-F002, F020, F040, F042 |
| SRS-F129 (lockout clear) | EPSC-F062 |
| SRS-F130 (heater) | EPSC-P002 |
| SRS-F149 (Jetson current) | EPSC-F013 |
| SRS-F155, SRS-F157 (EOL state and the two-processor arm) | EPSC-F070 to F073 |
| SRS-F162 (heartbeat, AntS supply, tracker disables) | EPSC-F080 to F084, F090, F091 |
| Derived (reason in each row) | EPSC-F003, F005, F006, I003, I006, I009, F014, F041, F043, F052, F061, F092, C004 |

## 9. Findings for the Chair and the Owners

1. **SOC source.** SMART-QSO-SDP-001 1.5 §3.4 and SQSO-EPS-BRD-001 §12 say "SOC from the INA228". The INA228 (0x44)
   is on I2C2, where the controller is a target; reading it would make the controller a second master on the OBC's
   bus. EPSC-F040 takes the pack current from the AFE's coulomb-counter shunt (RS7) on the local bus instead, and
   leaves the INA228 to the OBC as an independent gauge. Owner: EPS designer (§12 text).
2. **LOAD_EN and MODE** (ICD-001 §4.4, §4.5, §6.2) are legacy: the controller drives no load. EPSC-F021 keeps them as
   stored values. Owner: ICD owner (state it in §4.5).
3. **UNLOCK duration.** ICD-001 1.6 §4.6 keeps UNLOCK high until another value is written; the OBC never writes one,
   so after the first clear the lockout would be clearable by LOCK_CLR alone. EPSC-F062 lowers it after 1 s. Owner:
   ICD owner (amend the LOCK_UNLOCK row).
4. **Proposed registers** for the ICD owner: 0x90–0x9B burn currents, V_BURN, VTRK (EPSC-F014); 0x9C controller flags
   (bit 0 SOC uncalibrated, bit 1 calibration default, bit 3 burn overrun, bit 4 AFE settings mismatch, bit 5 local
   device failed, bit 6 LOCK_SET event); 0x9D reset cause; 0x9E refused EOL sequences; 0xF2 image CRC; and the refusal
   of a four-tracker mask (EPSC-F092).
5. **Heartbeat timing in the board design.** SQSO-EPS-BRD-001 §8.3 (SW10 row) and V-EPS-16 still read "120 s" and "no
   cycle within 300 s", against the 600 s of §12 and ICD-001 1.6. The firmware follows the ICD (EPSC-F081). Owner: EPS
   designer.
6. **AFE sealed at acceptance.** EPSC-F063 assumes the BQ76952 is left sealed after its OTP and data memory are
   written; the V-EPS-2 procedure should say so. Owner: EPS designer.

---

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | SMART-QSO Team (CDR round 3, software systems; finding P3A-03) | Initial issue: 62 requirements of the EPS controller firmware, 20 safety-critical (SW-HAZ-016, SW-HAZ-017), derived from SQSO-EPS-BRD-001 G, SMART-QSO-ICD-001 1.6, the battery design and safety assessment, and the flight software as built; hardware functions it must not be relied on (§3.2); language C11 to MISRA C:2012 with MSP430-GCC recommended (§7); traceability through the generated RTM (§8); findings for the chair (§9) |
