# Change Request Log
## SMART-QSO

**Document Number:** SQSO-CRL-001
**Revision:** C
**Date:** 2026-10-26
**Reference:** `docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` §4.3 to §4.4, §6.3

---

## 1. Purpose

The change requests against baselined items: requirements, `docs/BASELINE_PARAMETERS.md` and interface documents.
Each entry follows the template of the Configuration Management Plan §4.4. Editorial changes (Class III) are not
logged here; the Git history records them.

## 2. Log

| CR | Date | Class | Item | Change | Decision | Status |
|----|------|-------|------|--------|----------|--------|
| CR-2026-001 | 2026-10-25 | I | SRS-C001 (`docs/software/SOFTWARE_REQUIREMENTS.md` §8.1) | "C conforming to C11" becomes "Rust (2024 edition) or C conforming to C11" | Approved, PI | Implemented, verified |
| CR-2026-002 | 2026-10-25 | I | BASELINE §6 OBC row; OBC-F001 (`hardware/obc/OBC_DESIGN.md` A.2.1) | FreeRTOS becomes the Embassy executor, no RTOS | Approved, PI | Implemented; OBC-F001 verification pending (hardware) |
| CR-2026-003 | 2026-10-25 | I | SWE-219 tailoring (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.2 to §3.3) | MC/DC replaced by 100 % line and condition coverage of the safety-critical components | Approved, PI | Implemented: per-file CI gate |
| CR-2026-004 | 2026-10-25 | II | Tool qualification (SDP §3.3) | The flight image stays on the pinned, unqualified rustc 1.97.0; no qualified compiler (Ferrocene) | Approved, PI | Implemented |
| CR-2026-005 | 2026-10-25 | I | OBC–transmitter interface (`hardware/obc/OBC_DESIGN.md` §8.1 and A.5; SQSO-RF-TX-001 §10.1; SRS §5.3) | The downlink goes to a KISS TNC on USART2, shared with the uplink receiver; power select on one GPIO line (0.5 W / 1 W); three ADC monitors; SRS-I023 to SRS-I027 | Approved, PI | Implemented; verification of the TNC and transmitter pending (hardware) |
| CR-2026-006 | 2026-10-26 | I | BASELINE §5 battery row, §8 ADCS row, §9 D8–D9; `sqso_fsw::adcs` gains | Battery charged to 4.10 V/cell (size kept); gyro Epson M-G370PDT0 with BMI088 backup; B-dot gain 10⁴; spin 6°/s with a spin-hold term | Approved, PI delegation | Implemented in the software and the simulation; hardware pending |

## 3. Change Requests

### CR-2026-001: SRS-C001, implementation language

```
Change Request Number: CR-2026-001
Date Submitted: 2026-10-25
Submitter: Flight software

1. Configuration Item Affected:
   CI ID: SMART-QSO-SRS-001, requirement SRS-C001
   Current Version: 1.3

2. Description of Change:
   SRS-C001 "OBC code shall be written in C conforming to C11 standard" becomes
   "OBC code shall be written in Rust (2024 edition) or in C conforming to the C11 standard".

3. Reason for Change:
   The OBC flight software is Rust since 2026-10-24 (software/rust/README.md); the only C
   left is the flight-log printf shim and the C test suites. As written, SRS-C001 was not met.

4. Impact Assessment:
   - Technical: none to the code. Rust code follows docs/development/CODING_STANDARDS.md §11;
     the C shim stays C11.
   - Schedule: none.
   - Cost: none.
   - Risk: the Rust toolchain (1.97.0, pinned) is not qualified; whether a qualified compiler
     is needed follows from the software classification (SDP §3).

5. Affected Documents:
   docs/software/SOFTWARE_REQUIREMENTS.md (1.4), docs/verification/RTM.csv,
   docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md.

6. Verification Required:
   Inspection: the flight crates are Rust, the C is C11 (CI builds the shim with -std=c11).

7. CCB Decision:
   [x] Approved
   Reason: PI decision, 2026-10-25.

8. Implementation:
   Assigned to: Flight software
   Completed: 2026-10-25

9. Verification:
   Verified by: inspection of software/rust and software/flight/src
   Date: 2026-10-25
```

### CR-2026-002: BASELINE §6 and OBC-F001, OBC executive

```
Change Request Number: CR-2026-002
Date Submitted: 2026-10-25
Submitter: Flight software

1. Configuration Item Affected:
   CI ID: SQSO-BL-001 §6 (OBC row); SQSO-CDH-OBC-001 requirement OBC-F001
   Current Version: BASELINE Revision B; OBC design 1.3

2. Description of Change:
   BASELINE §6 OBC "STM32L4R5ZI, FreeRTOS" becomes "STM32L4R5ZI; flight software in Rust on
   the Embassy executor, no RTOS". OBC-F001 "Execute flight software in FreeRTOS" becomes
   "Execute flight software on the Embassy executor (Rust; no RTOS)".

3. Reason for Change:
   FreeRTOS was never integrated. The flight software runs as one Embassy task with a 20 ms
   main loop (software/flight/docs/RTOS_ARCHITECTURE.md). The baseline governs, so documents
   describing the implementation disagreed with it.

4. Impact Assessment:
   - Technical: none to the code. No RTOS kernel, task stacks or heap: one 8 KB stack
     (SRS-C003, software/flight/docs/MEMORY_POLICY.md §6).
   - Schedule: none.
   - Cost: none.
   - Risk: Embassy is less widely flown than FreeRTOS; mitigated by the image checks, the
     static stack and loop analyses, and the hardware tests (software/flight/tests/hil).

5. Affected Documents:
   docs/BASELINE_PARAMETERS.md (Revision C), hardware/obc/OBC_DESIGN.md (1.4), and the
   documents that described FreeRTOS as the design (updated or annotated as superseded).

6. Verification Required:
   OBC-F001 by test on the engineering-model OBC (HIL test plan); until then, CI builds and
   checks the image.

7. CCB Decision:
   [x] Approved
   Reason: PI decision, 2026-10-25.

8. Implementation:
   Assigned to: Flight software
   Completed: 2026-10-25 (documents)

9. Verification:
   Verified by: pending hardware (OBC_DESIGN A.11.1)
   Date: —
```

### CR-2026-003: SWE-219, MC/DC for the safety-critical components

```
Change Request Number: CR-2026-003
Date Submitted: 2026-10-25
Submitter: Flight software

1. Configuration Item Affected:
   CI ID: SMART-QSO-SDP-001 §3.3 (tailoring); NPR 7150.2D SWE-219
   Current Version: SDP 1.2

2. Description of Change:
   For the safety-critical components of SDP §3.2, replace "100 % code coverage by the
   MC/DC criterion" (SWE-219) with "100 % line coverage and 100 % condition coverage
   (each condition of each decision taken both ways)", measured by LLVM source-based
   coverage (-Zcoverage-options=condition) over the C suites and the Rust unit tests.

3. Reason for Change:
   The flight software is Rust. rustc has no MC/DC instrumentation (it was removed);
   condition coverage is the strongest decision-level measure it provides. No qualified
   MC/DC tool for Rust is known to the project.

4. Impact Assessment:
   - Technical: condition coverage does not show that each condition independently
     affects its decision. Mitigation: the decisions in these components are short (most
     have one or two conditions), and mutation testing of sqso-fsw left no survivor in
     them that changes behaviour (software/rust/mutants/RESULTS-sqso-fsw.md; deployment's
     three survivors are equivalent).
   - Schedule: tests to bring the components of SDP §3.2 from the coverage measured there
     to 100 %.
   - Cost: none beyond that effort.
   - Risk: a decision with an untested independent effect of one condition.

5. Affected Documents:
   docs/software/SOFTWARE_DEVELOPMENT_PLAN.md §3.2 to §3.3,
   docs/safety/SOFTWARE_SAFETY_PLAN.md §8.2.

6. Verification Required:
   A CI gate on the per-file condition coverage of the safety-critical components.

7. CCB Decision:
   [x] Approved
   Reason: PI decision, 2026-10-25. The PI also concurred with the list of
   safety-critical components (SDP §3.2).

8. Implementation:
   Assigned to: Flight software
   Completed: 2026-10-25: tests to 100 %, and a CI gate per safety-critical file
   (software/flight/scripts/safety_coverage.py)

9. Verification:
   Verified by: —
   Date: —
```

### CR-2026-004: tool qualification

```
Change Request Number: CR-2026-004
Date Submitted: 2026-10-25
Submitter: Flight software

1. Configuration Item Affected:
   CI ID: SMART-QSO-SDP-001 §3.3 (tool qualification)

2. Description of Change:
   Record the decision: the flight image is built with the pinned rustc 1.97.0
   (software/rust/rust-toolchain.toml), which is not qualified. No qualified compiler.

3. Reason for Change:
   Class C software on a project outside NASA; a qualified toolchain (Ferrocene) is a
   paid subscription. The compiler's output is checked by the tests on the host, the
   image checks and static stack analysis in CI, and the hardware tests.

4. Impact Assessment:
   - Technical: a compiler defect would not be caught by tool qualification; it would
     have to show in the tests, the image checks or the hardware tests.
   - Schedule, cost: none.
   - Risk: accepted by the PI.

7. CCB Decision:
   [x] Approved
   Reason: PI decision, 2026-10-25.
```

### CR-2026-005: the downlink transmitter interface

```
Change Request Number: CR-2026-005
Date Submitted: 2026-10-25
Submitter: Flight software

1. Configuration Item Affected:
   CI ID: SQSO-CDH-OBC-001 §8.1 and A.5 (UART allocation, RF transceiver
   link, GPIO, ADC: "Not set" in the conflicts table); SQSO-RF-TX-001
   §10.1; SMART-QSO-SRS-001 §5.3

2. Description of Change:
   The OBC hands each downlink AX.25 frame (no FCS) to a KISS TNC on
   USART2 at 9600 baud, the same TNC and UART as the 435 MHz receiver,
   with the TNC's parameters before every frame. The TNC adds the FCS,
   modulates AFSK 1200 and keys the 145.8 MHz transmitter. The OBC
   selects the power on one GPIO line (TX_PWR_SEL: low 0.5 W, high 1 W;
   BASELINE §7 allows no more than 1 W, so the RF design's third, 2 W,
   level is not used) and reads three ADC monitors (TX_TEMP, TX_FWD,
   TX_REV). The PA power stays on RF_TX_EN (rf_control, TX-OFF).
   New requirements SRS-I023 to SRS-I027 (gate, limits, protection).

3. Reason for Change:
   No downlink driver existed (KNOWN_ISSUES KI-001): a beacon frame was
   built and counted but sent nowhere. The OBC design gave the RF link
   as a UART (§8.1) and as SPI (A.5.3); the RF design as audio from the
   MCU's DAC (§5.3). A TNC keeps the OBC's part independent of the
   transceiver chosen and matches the receive side, which already
   takes KISS frames from a TNC.

4. Impact Assessment:
   - Technical: the transceiver must have, or be paired with, a KISS
     TNC that can send while its receiver listens (full duplex). Pins
     (provisional): TX_PWR_SEL PE3; TX_TEMP PA0, TX_FWD PC0, TX_REV PC1;
     FRAM WP#/HOLD# move to PE0/PE1 and the deployment sense to PE2.
   - Schedule: none to the software; the TNC is part of the transceiver
     selection (SQSO-ICD-005).
   - Cost: a TNC if the transceiver has none.
   - Risk: the air-time estimates depend on the TNC's behaviour
     (TBR-SRS-003).

5. Affected Documents:
   docs/software/SOFTWARE_REQUIREMENTS.md (1.5), hardware/obc/OBC_DESIGN.md,
   hardware/rf/TRANSMITTER_DESIGN.md, docs/reviews/KNOWN_ISSUES.md (KI-001),
   software/flight/tests/hil/HIL_TEST_PLAN.md, software/rust/firmware/BRINGUP.md.

6. Verification Required:
   Test: the C suites test_downlink and Downlink_TNC_served/_silent
   (host); the hardware tests HIL-060 to HIL-064 with the TNC and
   transmitter.

7. CCB Decision:
   [x] Approved
   Reason: PI decision, 2026-10-25 (KISS to a TNC on the UART; beacon
   and telemetry frames; the transponder's keying is separate).

8. Implementation:
   Assigned to: Flight software
   Completed: 2026-10-25 (software/rust/sqso-fsw/src/downlink.rs,
   software/flight/include/downlink.h, software/rust/firmware/src/radio.rs)

9. Verification:
   Verified by: host tests; hardware pending
   Date: —
```

---

### CR-2026-006: battery charge voltage, gyro and ADCS gains

```
Change Request Number: CR-2026-006
Date Submitted: 2026-10-26
Submitter: Systems (build-versus-buy study)

1. Configuration Item Affected:
   CI ID: SQSO-BL-001 §5 (battery), §8 (ADCS), §9 (D8, D9);
   SMART-QSO-ADCS-001 §4.1, §4.2.1, §5; SQSO-ADCS-SEN-001 §4;
   sqso_fsw::adcs (BDOT_GAIN, SPIN_RATE, NUTATION_GAIN, SPIN_HOLD_GAIN,
   the capture criterion); ADCS_BDOT_GAIN (adcs_control.h)

2. Description of Change:
   - Battery: size kept (4S1P, about 50 Wh); the charge regulator's
     constant-voltage set point is 4.10 V a cell (16.4 V), not 4.20 V.
   - Gyro: Epson M-G370PDT0 primary, Bosch BMI088 backup on the OBC
     board; bias in use at most 0.1 deg/s (1 sigma per axis).
   - ADCS: B-dot gain 1e4 A m^2 s/T (was 5e6); SUNPOINT spin 6 deg/s
     (was 2) with a spin-hold term (gain 0.005 1/s) and nutation gain
     1e-4 N m s (was 4e-5); with a gyro, the detumble is complete when
     the rate across +Y is below 0.5 deg/s and the spin about +Y below
     twice the spin rate, for 5 minutes.

3. Reason for Change:
   The ADCS Monte Carlo of the flight code (docs/hardware/BUILD_VS_BUY.md
   section 6): at 5e6, 15 of 40 trials were still tumbling after two
   days; at 2 deg/s the spin law settled with -Y on the Sun in 23 of 40
   and held +Y within 15 deg for 75 % of sunlit time. With the change,
   every trial captures within 1.5 h and holds +Y within 15 deg for
   100 % of sunlit time. The pack sits at the end of charge for most of
   the mission; a lower set point slows calendar ageing. A smaller pack
   would need a 2S bus.

4. Impact Assessment:
   - Technical: about 10 % less usable capacity at 4.10 V; the worst
     simulated eclipse still uses under 4 %. The C header value
     ADCS_BDOT_GAIN and its contract check change with the Rust value.
   - Schedule: none.
   - Cost: the M-G370PDT0 (about $1-1.5k) and a BMI088 calibration rig.
   - Risk: the tip-off distribution and inertia are assumptions; the
     Monte Carlo is re-run with the measured values.

5. Affected Documents:
   docs/BASELINE_PARAMETERS.md (D), docs/hardware/ADCS_DESIGN.md (1.4),
   hardware/adcs/SENSOR_DESIGN.md (1.1), hardware/eps/BATTERY_DESIGN.md,
   docs/hardware/BUILD_VS_BUY.md (A), docs/test/SIL_MISSION_SIMULATION.md,
   docs/software/CONFIGURATION_INDEX.md, docs/test_procedures/TP-FAT-002_Flight_Config.md.

6. Verification Required:
   Analysis: the Monte Carlo (sqso_adcs_mc) and the mission simulation
   (sqso_sil --scenario full). Test: the Rust and C suites; the gyro and
   magnetic cleanliness on the EM.

7. CCB Decision:
   [x] Approved
   Reason: decisions delegated by the PI, 2026-10-26.

8. Implementation:
   Assigned to: Systems, flight software
   Completed: 2026-10-26 (software/rust/sqso-fsw/src/adcs.rs,
   software/flight/include/adcs_control.h, software/rust/sqso-sil)

9. Verification:
   Verified by: simulation and host tests; hardware pending
   Date: —
```

---

## 4. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-25 | Initial release: CR-2026-001 to CR-2026-004 | SMART-QSO Team |
| B | 2026-10-25 | CR-2026-005: the downlink transmitter interface | SMART-QSO Team |
| C | 2026-10-26 | CR-2026-006: battery charge voltage, gyro and ADCS gains | SMART-QSO Team |
