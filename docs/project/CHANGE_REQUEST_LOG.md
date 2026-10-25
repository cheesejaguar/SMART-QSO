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

---

## 4. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-25 | Initial release: CR-2026-001, CR-2026-002 | SMART-QSO Team |
