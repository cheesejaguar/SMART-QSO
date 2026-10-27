# Change Request Log
## SMART-QSO

**Document Number:** SQSO-CRL-001
**Revision:** F
**Date:** 2026-10-27
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
| CR-2026-007 | 2026-10-27 | I | BASELINE §1, §3–§9 (Rev E); SQSO-LVR-001; mission requirements MO-1 to MO-5 | CDR 2026 chair dispositions: launch path, orbit 475 km target / 485 km cap, planning launch 2028-11, mass properties and layout B, third inhibit DS3, burn drivers and sequence, commissioning hold, spin 8 °/s, uplink windows, transponder 1.8 W DC, telecommand stations | Proposed by the design review (recorded as approved by the chair, who cannot approve a Class I change: CM plan §2.2); PI ratification required by 2027-01-12 (DA-01) | Implemented in documents; software in progress |
| CR-2026-008 | 2026-10-27 | I | BASELINE §1, §3, §4, §5, §7 (Rev E, amended) | Inertia ballast 2 × 10 g (C-13; MEV 1.748 kg, +14.4 %); telecommand ≥ 64 dBm EIRP in two station variants (C-14); uplink ≥ 0.9 MHz below 3 × f_down; lifetimes, power and link values from the Rev E re-runs | Proposed by the design review (recorded as approved by the chair, who cannot approve a Class I change: CM plan §2.2); PI ratification required by 2027-01-12 (DA-01) | Implemented in documents |
| CR-2026-009 | 2026-10-27 | I | SQSO-REQ-002 Rev E, F and G (SYS-100, SYS-203, SYS-222, SYS-224, SYS-707 and the Rev E/F rows); EPS-F001 (`docs/hardware/EPS_DESIGN.md`); BASELINE §5 power-closure note | System requirements rebaselined by the CDR review (Rev E/F, omitted from CR-2026-007) and amended in round 3: SYS-100 2.21 W at the 1.84 W load; third harmonic ≥ 70 dB; SYS-222 the RF-P013 pattern; SYS-224 every emitter incl. the GNSS receiver and the completion latch; SYS-707 cycle count reconciled | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | Implemented in SQSO-REQ-002 G; EPS-F001 and BASELINE §5 by their owners after the decision |
| CR-2026-010 | 2026-10-27 | I | SYS-603; BASELINE §6 Jetson heat-path row; ETP VI-01, V-THM-3, V-JET-7 | The ≥ 1 W/K conductance requirement becomes a temperature requirement (spreader ≤ +40 °C in a 600 s batch, junction ≤ 95 °C, spreader ≤ +70 °C stuck on); the conductance becomes a measured design value | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | Implemented in SQSO-REQ-002 G, ETP D, VCRM D; BASELINE §6 by the chair |
| CR-2026-011 | 2026-10-27 | I | SQSO-ICD-013 0.7–0.10, SMART-QSO-ICD-001 1.1–1.5, SQSO-ICD-005 2.0–2.4, SQSO-ICD-010 E–G | Record of the ICD changes made during the CDR review without a CR: passivation opcodes 0x70/0x71 and their sequence; EPS register map (CHG_INHIBIT, LOCK_UNLOCK, EOL_STATE, EOL_KEY, IMON_JET) and JETSON_PWR on PA8; space-to-ground ICD rewrite and its updates; launch-provider ICD against the EXOpod Nova chain | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | Implemented in the ICDs; verification by their VIs |
| CR-2026-012 | 2026-10-27 | I | Safety-critical software components (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.2, versions 1.4 and 1.5; chair decision C-10) | `transponder.rs`, `ops_state.rs` (both crates) and `passivation.rs` (both crates) added to the safety-critical list, with the per-file coverage gate | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | Implemented in the SDP and the CI gate |
| CR-2026-013 | 2026-10-27 | I | SQSO-ETP-001 Rev D §9 deviations 1–3 and 7; §3.3 shock, §3.4 TVAC definition, §3.11 reliability programme; SQSO-LVR-001 LVR-046, VI-LV-16, VI-LV-17 | Tailoring of GSFC-STD-7000B (hot non-operating plateau +60 °C, 8 + 4 cycles, about 24 h failure-free in vacuum) and of the UG v11 advised shock (EM test if VI-07 needs it, FM by similarity); FM deployment before vibration; reliability programme added | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | Implemented in ETP D and LVR E (amended) |
| CR-2026-014 | 2026-10-27 | I | EPS-F002 (`docs/hardware/EPS_DESIGN.md`); V-EPS-1 (SQSO-EPS-BRD-001); V-PWR-3 and the MPPT factor (SQSO-PBA-001) | MPPT efficiency "≥ 90 % at every operating point ≥ 1 W per tracker over 3.5–6.4 V, ≥ 91 % power-weighted" (was ≥ 92 %, which the 90.9–92.3 % design does not meet at low power); V-EPS-1 and V-PWR-3 take the same values; the power analysis MPPT factor 0.92 → 0.91 | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | Proposed by the EPS designer; documents change after the decision |
| CR-2026-015 | 2026-10-27 | I | BASELINE §3 launch-cost row and backup trigger; SQSO-AUD-005 Rev F | Program-side launch cost $57k–195k (was $30k–100k), all-in $142k–345k; the backup trigger read on the integrator's slot price | Proposed by the design review; PI decision required by 2027-01-12 (DA-01) | ROM F implemented; BASELINE §3 by the chair after the decision |

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

### CR-2026-007: CDR 2026 chair dispositions

```
Change Request Number: CR-2026-007
Date Submitted: 2026-10-27
Submitter: Design review chair (CDR 2026, SQSO-REV-CDR-002)

1. Configuration Item Affected:
   CI ID: SQSO-BL-001 §1, §3, §4, §5, §6, §7, §8, §9 (Rev E);
   SQSO-LVR-001 (B); docs/requirements/MISSION_REQUIREMENTS.md
   (objectives MO-1 to MO-5); mission/CONOPS.md (LEOP, commissioning,
   passivation)

2. Description of Change:
   - Launch: Falcon 9 rideshare from Vandenberg through Exolaunch
     (EXOpod Nova 1U slot); planning launch 2028-11; NRCSD backup.
   - Orbit: 475 km target, 465-485 km band, 485 km cap (FCC five-year
     rule with margin, ODAR Draft B).
   - Mass properties: CBE 1.49 kg, MEV 1.73 kg; Iyy at least 1.03 times
     the other principal moments, axis within 5 deg of +Y (layout B,
     AntS elements along +/-X).
   - Inhibits: three deployment switches DS1-DS3 on three rails;
     120 s actuator hold; RF silence latched; commissioning hold until
     COMMISSIONING_RELEASE.
   - Deployment: two resistors per wing on four independent 0.8 A burn
     lines, one resistor at a time for 8 s, a 12 s hardware monostable;
     wings first, then the AntS.
   - ADCS: spin 8 deg/s (was 6), nutation gains 3e-4 / 1e-4 N m s.
   - RF: uplink request 435.222-435.778 MHz, else 436.222-436.500 MHz;
     transponder allocation 1.8 W DC.
   - Operations: at least two telecommand stations at least 1,000 km
     apart, each able to command alone.
   - Mission objectives MO-1 to MO-5 aligned with the baseline.

3. Reason for Change:
   Round-1 RIDs of the CDR 2026 design review (SQSO-REV-CDR-002 §4,
   chair decisions C-1 to C-12): the 500 km altitude failed the FCC
   five-year rule; the burn current exceeded the fuse; no third
   inhibit; +Y was barely the major axis; no commissioning hold or
   passivation command; no designated stations; the June 2028 date
   missed the integrator's L-4M milestone.

4. Impact Assessment:
   - Technical: power margins +32 % / +75 % (allocation); mass margin
     +15.9 % against the 2.00 kg allocation, 2.5 kg slot limit.
   - Schedule: launch moves from 2028-06 to 2028-11.
   - Cost: SQSO-AUD-005 Rev C ($295k-$779k non-labor).
   - Risk: SQSO-RISK-001 Rev B.

5. Affected Documents:
   docs/BASELINE_PARAMETERS.md (E), docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md (B),
   mission/CONOPS.md, docs/ORBITAL_ACCESS.md (B), docs/funding_audit/LIFECYCLE_COST_ROM.md (C),
   docs/requirements/MISSION_REQUIREMENTS.md, the board build-to designs under
   hardware/electrical/ and hardware/mechanical/MECHANICAL_DESIGN.md.

6. Verification Required:
   Analysis: SIL mission simulation and ADCS Monte Carlo with the
   changed values; the RID closure panel (SQSO-REV-CDR-002 §6).
   Test: the verification items VI-* of the review on the EM and
   flight unit.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: proposed by the design review chair, 2026-10-27, and recorded
   then as "approved"; the PI is the only approver of a Class I change
   (CM plan section 2.2). PI ratification required by 2027-01-12
   (mission/SCHEDULE.md DA-01). The review was conducted by AI
   reviewers, not an independent board. Requirement changes of this
   review (SQSO-REQ-002 Rev E and F) are carried by CR-2026-009.

8. Implementation:
   Assigned to: Systems, flight software, hardware
   Completed: documents 2026-10-27; flight software in progress

9. Verification:
   Verified by: the review's closure panel; hardware pending
   Date: —
```

---

### CR-2026-008: CDR 2026 round-2 baseline amendments

```
Change Request Number: CR-2026-008
Date Submitted: 2026-10-27
Submitter: Design review chair (CDR 2026, SQSO-REV-CDR-002)

1. Configuration Item Affected:
   CI ID: SQSO-BL-001 Rev E (amended) §1, §3, §4, §5, §7

2. Description of Change:
   - §4: 2 x 10 g W-Ni-Cu ballast on the +Z end frame (C-13); CBE
     1.51 kg, MEV 1.75 kg, margin +14.4 %; deployed inertia ratio
     1.055 / 1.046 with all four AntS elements.
   - §7: telecommand stations radiate >= 64 dBm EIRP above 10 deg:
     >= 100 W PEP into >= 15.5 dBic, or <= 50 W PEP into a >= 17.5
     dBic stacked pair inside the 47 CFR 2.106 US270 areas (C-14);
     uplink windows >= 0.9 MHz below 3 x f_down (was stated 2 MHz).
   - §1, §3, §5, §7: contact statistics, lifetimes (ODAR Draft D,
     deployment 2028-11-01), generation and margins (PBA 2.2) and the
     beacon margin (LBA 3.4) from the Rev E re-runs.

3. Reason for Change:
   Round-2 RIDs RFB-11, MCAD-09 (inertia margin 0.007 with four AntS
   elements), RF-23 (telecommand +3.8 dB at 10 deg with the receiver as
   designed), SE-18 (uplink separation misstated), SE-16 (analyses at
   superseded values).

4. Impact Assessment:
   - Technical: inertia ratio holds >= 1.03 to 3.1 mm placement
     scatter; telecommand >= +8.8 dB at 10 deg in either variant.
   - Schedule: none.
   - Cost: ground stations about $6,650 (A) or $7,350 (B) each;
     lifecycle $300k-$784k (SQSO-AUD-005 Rev E).
   - Risk: none added.

5. Affected Documents:
   docs/BASELINE_PARAMETERS.md, hardware/mechanical/MECHANICAL_DESIGN.md,
   docs/environment/MASS_BUDGET.md, docs/environment/LINK_BUDGET_ANALYSIS.md,
   docs/ground/GROUND_STATION_DESIGN.md, docs/regulatory/filings/ODAR_SMART-QSO.md,
   docs/environment/POWER_BUDGET_ANALYSIS.md.

6. Verification Required:
   Analysis: the mass properties, link budget and lifetime re-runs.
   Test: measured inertia (VI-MCAD-03); station EIRP (VI-GND-01).

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: proposed by the design review chair, 2026-10-27, and recorded
   then as "approved"; the PI is the only approver (CM plan section 2.2).
   PI ratification required by 2027-01-12 (mission/SCHEDULE.md DA-01).

8. Implementation:
   Assigned to: Systems, mechanical, ground
   Completed: 2026-10-27

9. Verification:
   Verified by: the review's closure panel; hardware pending
   Date: —
```

---

### CR-2026-009: system requirements Rev E, F and G

```
Change Request Number: CR-2026-009
Date Submitted: 2026-10-27
Submitter: Design review, round 3 (CDR 2026, SQSO-REV-CDR-002), systems and programmatics

1. Configuration Item Affected:
   CI ID: SQSO-REQ-002 (System Requirements Specification), Rev D -> G:
   the Rev E rows (SYS-002, -003, -010, -012, -100, -111, -112, -203,
   -420, -422, -501, -502, -510, -512, -513, -603, -701, -702, -712;
   SYS-080 to -089, SYS-703 to -709, SYS-713, SYS-900 to -919), the Rev F
   rows (SYS-012, -422, -907) and the Rev G rows (SYS-100, -203, -222,
   -224, -707); EPS-F001 (docs/hardware/EPS_DESIGN.md);
   SQSO-BL-001 section 5 power-closure note
   Current Version: SQSO-REQ-002 G (proposed)

2. Description of Change:
   - Rev E and F: as recorded in SQSO-REQ-002 section 16 (CDR review,
     2026-10-27). CR-2026-007 carried the MRD and the baseline but not
     these rows, although CM plan section 4.3 said it did.
   - SYS-100: at least 2.21 W orbit-average delivered to the bus at
     end of life while tumbling = the 1.84 W tumbling load of
     BASELINE section 5 x 1.2 (was 2.16 W at a 1.80 W load). Met by
     the simulated 2.48 W (PBA v2.2): +12 %. EPS-F001 takes the same
     value (its status still quotes 2.43 / 3.93 W; BASELINE gives
     2.48 / 3.92 W). BASELINE section 5 note: "SYS-100 (2.21 W) met
     with 12 % to spare" (was 2.16 W, 14.8 %).
   - SYS-203: third harmonic at least 70 dB below the carrier at the
     antenna port, antennas deployed (was 60 dB; BASELINE section 7).
   - SYS-222: median >= -3 dBi and 10th percentile >= -10 dBi toward
     the ground over the pass-weighted directions, each band (RF-P013)
     (was "omnidirectional coverage").
   - SYS-224: the 45-minute silence covers every RF emitter including
     the GNSS receiver; once complete it is latched (no later reset or
     stowed reading restarts it).
   - SYS-707: the integrator's higher cycle count is met by board-level
     and ambient cycles (ETP VI-08); the SpaceX +/-5 C range mapped
     onto the GEVS +10 C margin (one TVAC definition, RID P3S-10).

3. Reason for Change:
   RIDs SE-03 and P3E-08 (SYS-100 used the 1.80 W load against
   BASELINE section 5's 1.84 W), RF-07 (60 dB against 70 dB), RF-05
   (a monopole cannot be omnidirectional), P3A-01 (GNSS receiver
   powered inside the silence) and LV-15 (latch), P3S-10 (three TVAC
   definitions); P3P-02 (Class I changes without a CR).

4. Impact Assessment:
   - Technical: SYS-100 margin over the requirement +12 % (generation 2.48 W against 2.21 W); the 2.14 W isotropic bound would miss it, as before. The others align the requirements with the baseline and the design; no design change.
   - Schedule: None.
   - Cost: None.
   - Risk: None added; R23 updated.

5. Affected Documents:
   docs/requirements/SYSTEM_REQUIREMENTS.md (G), docs/verification/RTM.csv,
   docs/verification/VERIFICATION_MATRIX.md (D), docs/hardware/EPS_DESIGN.md
   (EPS-F001, by the EPS designer), docs/BASELINE_PARAMETERS.md section 5
   note (by the chair), docs/environment/THERMAL_ANALYSIS.md V-THM-6 (by the
   thermal analyst).

6. Verification Required:
   Analysis: power closure (V-PWR-1 on the EM flatsat); link and pattern
   (VI-RF-01); RF silence (VI-LV-07, VI-RF-13 with the GNSS receiver).
   Test: as the VCRM rows.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: Systems; EPS designer (EPS-F001); chair (BASELINE)
   Completed: SQSO-REQ-002 G, 2026-10-27 (proposed text)

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

### CR-2026-010: Jetson heat path as a temperature requirement

```
Change Request Number: CR-2026-010
Date Submitted: 2026-10-27
Submitter: Design review, round 3 (CDR 2026, SQSO-REV-CDR-002), systems and programmatics

1. Configuration Item Affected:
   CI ID: SQSO-REQ-002 SYS-603; SQSO-BL-001 section 6 "Jetson heat path";
   SQSO-ETP-001 VI-01; SQSO-THA-001 V-THM-3; SQSO-JET-BRD-001 V-JET-7
   Current Version: SQSO-REQ-002 G (proposed); BASELINE Rev E (amended)

2. Description of Change:
   SYS-603 becomes: with the module at 8 W input in the hot case, the
   spreader stays <= +40 C at the BASELINE section 6 burst duty including
   a 600 s batch (+50 C operating limit less the 10 C margin before
   correlation); the junction <= 95 C (payload guard); stuck on, the
   spreader <= +70 C (guard). Verified by analysis with the correlated
   model using the heat-path conductance and the junction-to-spreader
   resistance measured on the EM (VI-01, V-THM-3, V-JET-7).
   BASELINE section 6 row (proposed): "Jetson heat path: temperature
   limits of SYS-603; design 1.10 W/K spreader to frame and 1.09-1.16 K/W
   junction to spreader (estimates), measured on the EM". 

3. Reason for Change:
   RID SE-07: SYS-603 and BASELINE required >= 1 W/K against a
   documented 0.44 W/K, and VCRM section 6 row 3 still showed it as a
   non-compliance. The thermal analysis (THA 2.1 section 6.3) shows the
   conductance does not drive: 39.0 C for a 600 s batch at 0.44 W/K,
   35.3 C at 1.0 W/K; the mechanical design now estimates 1.10 W/K.

4. Impact Assessment:
   - Technical: The requirement states what the payload needs; the 600 s batch is 1 C inside its limit at the audit's 0.44 W/K and about 5 C inside at the 1.10 W/K design.
   - Schedule: None.
   - Cost: None.
   - Risk: R16 unchanged; the heat-path term of R16 is retired by test (VI-01).

5. Affected Documents:
   docs/requirements/SYSTEM_REQUIREMENTS.md (G), docs/environment/
   ENVIRONMENTAL_TEST_PLAN.md (D), docs/verification/VERIFICATION_MATRIX.md
   (D), docs/verification/VI_REGISTER.md (V-THM-3, V-JET-7 overrides),
   docs/BASELINE_PARAMETERS.md section 6 (by the chair), docs/project/SEMP.md.

6. Verification Required:
   Test: VI-01 and V-THM-3 on the EM (EM tests 2027-07 to 2027-09).
   Analysis: the correlated thermal model at the CDR.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: Systems; thermal analyst; payload lead
   Completed: documents 2026-10-27 (proposed)

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

### CR-2026-011: interface control documents changed during the CDR review

```
Change Request Number: CR-2026-011
Date Submitted: 2026-10-27
Submitter: Design review, round 3 (CDR 2026, SQSO-REV-CDR-002), systems and programmatics

1. Configuration Item Affected:
   CI ID: SQSO-ICD-013 (versions 0.7 to 0.10); SMART-QSO-ICD-001
   (1.1 to 1.5); SQSO-ICD-005 (2.0 to 2.4); SQSO-ICD-010 (E to G)

2. Description of Change:
   Record of the changes, each listed in its ICD's revision history:
   - ICD-013: telecommand-station status and the US270 limits; UTC-
     derived counters (proposed); PASSIVATE_ARM 0x70 and
     PASSIVATE_EXECUTE 0x71 with the recovery key; the passivation
     sequence (fire while transmitting, final report, then TX-OFF);
     ADCS_SET_GYRO_ALIGN 0x25 and ADCS_SET_MAG_BIAS 0x26.
   - ICD-001: 4S rails and load switches; I2C addresses and pins per
     SQSO-ICX-001; CHG_INHIBIT, LOCK_UNLOCK 0x7D, EOL_STATE, IMON_JET
     0x8A, EOL_KEY 0x7E; JETSON_PWR on PA8.
   - ICD-005: rewritten to the baseline (1200 bps AFSK UI frames),
     beacon jitter, acknowledgements, stored housekeeping and DUMP_TLM,
     uplink windows, link and pass statistics at 475 km, 64 dBm EIRP.
   - ICD-010: written against the Falcon 9 / Exolaunch / EXOpod Nova
     chain; consistency pass; mass, inertia, inhibits and levels to
     BASELINE Rev E (amended).

3. Reason for Change:
   RID P3P-02: CM plan section 4.3 makes every ICD change Class I; these
   were made without a CR.

4. Impact Assessment:
   - Technical: Records changes already made; the flight software and the C headers implement ICD-013 and ICD-001.
   - Schedule: None.
   - Cost: None.
   - Risk: None.

5. Affected Documents:
   docs/interfaces/TELECOMMAND_AUTHENTICATION.md, docs/interfaces/
   ICD_OBC_EPS.md, docs/interfaces/ICD_SPACE_GROUND.md,
   docs/compliance/LAUNCH_PROVIDER_ICD.md

6. Verification Required:
   Test: the flight-software suites (CI); the EM interface tests
   (V-ICX-*, V-OBC-*, TP-INT); the integrator's mission ICD supersedes
   ICD-010 at the contract.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: Flight software; EPS designer; launch and mission integration
   Completed: 2026-10-27 (documents)

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

### CR-2026-012: safety-critical software components

```
Change Request Number: CR-2026-012
Date Submitted: 2026-10-27
Submitter: Design review, round 3 (CDR 2026, SQSO-REV-CDR-002), systems and programmatics

1. Configuration Item Affected:
   CI ID: SMART-QSO-SDP-001 section 3.2 (versions 1.4, 1.5); the per-file
   coverage gate of the flight-software CI

2. Description of Change:
   transponder.rs, ops_state.rs (sqso-fsw and sqso-ffi) and
   passivation.rs (both crates) added to the safety-critical list,
   under the 100 % line and condition coverage gate of CR-2026-003
   (chair decision C-10; RIDs FSW-05, REG-12).

3. Reason for Change:
   CM plan section 4.3: a change to a safety-critical software component
   is Class I. The additions were made by the review without a CR
   (RID P3P-02).

4. Impact Assessment:
   - Technical: More code under the coverage gate; no functional change.
   - Schedule: None.
   - Cost: None.
   - Risk: Reduces the software contribution to the hazards SQSO-SHA-001 lists.

5. Affected Documents:
   docs/software/SOFTWARE_DEVELOPMENT_PLAN.md (1.4, 1.5),
   docs/safety/SOFTWARE_HAZARD_ANALYSIS.md

6. Verification Required:
   Test: the CI coverage gate on each listed file.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: Flight software
   Completed: 2026-10-27

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

### CR-2026-013: environmental test tailoring and verification approach

```
Change Request Number: CR-2026-013
Date Submitted: 2026-10-27
Submitter: Design review, round 3 (CDR 2026, SQSO-REV-CDR-002), systems and programmatics

1. Configuration Item Affected:
   CI ID: SQSO-ETP-001 Rev D (section 9 deviations 1-3 and 7; sections 3.3,
   3.4, 3.11); SQSO-LVR-001 LVR-046, VI-LV-16, VI-LV-17

2. Description of Change:
   - Deviation 1: hot non-operating plateau +60 C instead of +70 C
     (battery).
   - Deviation 2: 8 thermal-vacuum + 4 board-level cycles for the GEVS
     12-cycle total.
   - Deviation 3: about 24 h hot and 24 h cold failure-free in vacuum
     instead of 100 h each, offset by the DITL, the EM burn-in and the
     reliability programme (>= 500 h EM flatsat, VI-11; >= 200 h FM
     burn-in, VI-12).
   - Deviation 7: no FM shock test; an EM shock test at MPE + 3 dB if
     the integrator's attenuated Nova MPE is not enveloped by the
     vibe-equivalent SRS (VI-07); the UG v11 MPE itself is not.
   - One TVAC definition (8 cycles at AFT +/-10 C, capped at +60 C).
   - FM deployment tests before protoflight vibration (VI-LV-17).

3. Reason for Change:
   RIDs VV-08 (deviations not approved), VV-07 (LVR "Met (acknowledged)"
   against a 1000 g MPE), VV-06 (VI-LV-17 required post-environment
   deployment against ETP section 3.8 and CDS section 3.5.2), P3S-10,
   REL-01 / P3P-03 / P3E-07 (the reliability programme was in no plan).

4. Impact Assessment:
   - Technical: Workmanship screening below GEVS in failure-free hours, offset as stated; shock verified on the EM or by analysis.
   - Schedule: FM burn-in adds 10 days to the critical path (SQSO-SCH-001 D: +71 d float remains).
   - Cost: Test ROM $42k-130k (was $35k-101k): EM shock, FM system EMC, burn-in chamber.
   - Risk: An unattenuated shock environment would need the EM test; the FM is accepted by similarity.

5. Affected Documents:
   docs/environment/ENVIRONMENTAL_TEST_PLAN.md (D), docs/mission/
   LAUNCH_VEHICLE_REQUIREMENTS.md (E, amended), mission/SCHEDULE.md (D),
   software/simulation/cdr/pm/test_cost.py, docs/funding_audit/
   LIFECYCLE_COST_ROM.md (F)

6. Verification Required:
   Test: VI-07, VI-08, VI-11, VI-12, VI-LV-16, VI-LV-17. The integrator
   accepts deviations 4-6 at the contract.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: Test lead (PI until named)
   Completed: documents 2026-10-27 (proposed)

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

### CR-2026-014: MPPT efficiency requirement

```
Change Request Number: CR-2026-014
Date Submitted: 2026-10-27
Submitter: EPS designer (SQSO-EPS-BRD-001 D section 21 item 12), raised by the design review

1. Configuration Item Affected:
   CI ID: EPS-F002 (docs/hardware/EPS_DESIGN.md); V-EPS-1
   (SQSO-EPS-BRD-001); V-PWR-3 and the MPPT factor (SQSO-PBA-001)

2. Description of Change:
   EPS-F002: "Track the maximum power point of each string group with
   >= 90 % conversion efficiency at every operating point >= 1 W per
   tracker over 3.5-6.4 V input, and >= 91 % power-weighted over the
   orbit" (was ">= 92 % over 3.5-6.4 V"). V-EPS-1 and V-PWR-3 take
   the same pass criteria (were ">= 92 % above 2 W, >= 90 % above
   1 W" and ">= 0.92"). The power analysis MPPT factor 0.92 -> 0.91.

3. Reason for Change:
   RIDs EPSB-04, P3E-09: the TPS55340-Q1 design gives 90.9-92.3 %,
   below 92 % at low power; three documents carried three criteria.

4. Impact Assessment:
   - Technical: Delivered power -1.1 %; ACTIVE margins stay inside +54 % / +34 %; SYS-100 margin about +11 % with the 0.91 factor.
   - Schedule: None.
   - Cost: None.
   - Risk: None.

5. Affected Documents:
   docs/hardware/EPS_DESIGN.md, hardware/electrical/eps/
   EPS_BOARD_DESIGN.md, docs/environment/POWER_BUDGET_ANALYSIS.md,
   docs/verification/VI_REGISTER.md (regenerated)

6. Verification Required:
   Test: V-EPS-1 / V-PWR-3 on the EPS EM.

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: EPS designer, after the decision
   Due date: 2027-01-19

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

### CR-2026-015: launch cost and the backup trigger

```
Change Request Number: CR-2026-015
Date Submitted: 2026-10-27
Submitter: Design review, round 3 (CDR 2026, SQSO-REV-CDR-002), systems and programmatics

1. Configuration Item Affected:
   CI ID: SQSO-BL-001 section 3 "Launch cost (ROM)" and "Backup" rows;
   SQSO-AUD-005 Rev F

2. Description of Change:
   BASELINE section 3 launch cost (proposed): "$142k-345k all-in:
   integrator 1U slot $85k-150k plus $57k-195k program-side (tests
   $42k-130k, export counsel $10k-40k, logistics $5k-15k, ephemeris
   service up to $10k; SQSO-AUD-005 Rev F). Add 15 % contingency on
   the slot for rebooking." (was $115k-250k with $30k-100k program
   side). Backup row: the trigger "the primary exceeds $250k" reads
   "the integrator's slot price exceeds $250k".

3. Reason for Change:
   RID P3P-08: the program-side figure predated the bottom-up test
   costs; the ROM lines summed to $50k-166k before round 3.

4. Impact Assessment:
   - Technical: None.
   - Schedule: None.
   - Cost: Non-labour total $309k-956k (was $300k-784k).
   - Risk: With the program side in the comparison, a $250k all-in trigger would fire on the central case; stating it on the slot keeps its intent.

5. Affected Documents:
   docs/funding_audit/LIFECYCLE_COST_ROM.md (F); docs/BASELINE_PARAMETERS.md
   section 3 (by the chair)

6. Verification Required:
   Quotes replace the ROM lines (DA-07, DA-08).

7. CCB Decision:
   [ ] Approved   [ ] Approved with conditions   [ ] Rejected
   Reason: Proposed by the design review; PI decision required by
   2027-01-12 (mission/SCHEDULE.md DA-01). The review chair cannot
   approve a Class I change (CM plan section 2.2).

8. Implementation:
   Assigned to: PI (costs); chair (BASELINE row)
   Completed: ROM F 2026-10-27

9. Verification:
   Verified by: the review's closure panel (documents); hardware pending
   Date: -
```

---

## 4. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-25 | Initial release: CR-2026-001 to CR-2026-004 | SMART-QSO Team |
| B | 2026-10-25 | CR-2026-005: the downlink transmitter interface | SMART-QSO Team |
| C | 2026-10-26 | CR-2026-006: battery charge voltage, gyro and ADCS gains | SMART-QSO Team |
| D | 2026-10-27 | CR-2026-007: CDR 2026 chair dispositions | SMART-QSO Team |
| E | 2026-10-27 | CR-2026-008: CDR 2026 round-2 baseline amendments | SMART-QSO Team |
| F | 2026-10-27 | CR-2026-009 to CR-2026-015 (CDR round 3: system requirements E/F/G, Jetson heat path, ICDs, safety-critical list, test tailoring, MPPT, launch cost), each proposed by the design review for a PI decision by 2027-01-12; CR-2026-007 and -008 recorded as proposed by the chair, PI ratification required (CM plan §2.2; RID PM-04) | Design review, round 3 |
