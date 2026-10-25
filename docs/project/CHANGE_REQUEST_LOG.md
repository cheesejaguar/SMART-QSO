# Change Request Log
## SMART-QSO

**Document Number:** SQSO-CRL-001
**Revision:** A
**Date:** 2026-10-25
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

---

## 4. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-25 | Initial release: CR-2026-001 to CR-2026-004 | SMART-QSO Team |
