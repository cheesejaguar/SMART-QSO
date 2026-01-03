# Code Review Checklist
## SMART-QSO Flight Software

**Document Number:** SQSO-DEV-003
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, JPL D-60411

---

## 1. Purpose

This checklist ensures consistent, thorough code reviews for SMART-QSO flight software per NASA software assurance requirements.

## 2. Review Information

| Field | Value |
|-------|-------|
| Review Date | |
| Module/File | |
| Author | |
| Reviewer(s) | |
| Review Type | ☐ Inspection ☐ Walkthrough ☐ Desk Check |

---

## 3. JPL Power of Ten Rules

### Rule 1: Simple Control Flow
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 1.1 | No `goto` statements | ☐ | ☐ | |
| 1.2 | No `setjmp`/`longjmp` | ☐ | ☐ | |
| 1.3 | No recursion (direct or indirect) | ☐ | ☐ | |
| 1.4 | Control flow is clear and simple | ☐ | ☐ | |

### Rule 2: Fixed Loop Bounds
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 2.1 | All loops have fixed upper bounds | ☐ | ☐ | |
| 2.2 | Loop bounds are provably correct | ☐ | ☐ | |
| 2.3 | No infinite loops (except main) | ☐ | ☐ | |

### Rule 3: No Dynamic Memory After Init
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 3.1 | No `malloc`/`calloc`/`realloc` after init | ☐ | ☐ | |
| 3.2 | No `free` after init | ☐ | ☐ | |
| 3.3 | All buffers statically allocated | ☐ | ☐ | |

### Rule 4: Function Length
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 4.1 | Functions ≤ 60 lines (excluding comments) | ☐ | ☐ | |
| 4.2 | Functions do one thing well | ☐ | ☐ | |
| 4.3 | Complex functions are split | ☐ | ☐ | |

### Rule 5: Assertions
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 5.1 | Average ≥ 2 assertions per function | ☐ | ☐ | |
| 5.2 | Preconditions checked | ☐ | ☐ | |
| 5.3 | Postconditions verified | ☐ | ☐ | |
| 5.4 | Assertions have meaningful messages | ☐ | ☐ | |

### Rule 6: Minimal Scope
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 6.1 | Variables declared at smallest scope | ☐ | ☐ | |
| 6.2 | No unnecessary global variables | ☐ | ☐ | |
| 6.3 | Static used for file-scope variables | ☐ | ☐ | |

### Rule 7: Return Value Checking
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 7.1 | All return values checked | ☐ | ☐ | |
| 7.2 | Error handling is appropriate | ☐ | ☐ | |
| 7.3 | Errors are logged | ☐ | ☐ | |

### Rule 8: Limited Preprocessor
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 8.1 | No complex macros | ☐ | ☐ | |
| 8.2 | Macros are simple substitutions | ☐ | ☐ | |
| 8.3 | Conditional compilation is minimal | ☐ | ☐ | |

### Rule 9: Pointer Restrictions
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 9.1 | No more than one level of dereferencing | ☐ | ☐ | |
| 9.2 | Pointers are validated before use | ☐ | ☐ | |
| 9.3 | No pointer arithmetic (where avoidable) | ☐ | ☐ | |

### Rule 10: Compiler Warnings
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 10.1 | Compiles with -Wall -Wextra -Wpedantic | ☐ | ☐ | |
| 10.2 | No warnings generated | ☐ | ☐ | |
| 10.3 | Static analysis clean | ☐ | ☐ | |

---

## 4. Code Quality

### 4.1 Correctness
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 4.1.1 | Logic is correct | ☐ | ☐ | |
| 4.1.2 | Edge cases handled | ☐ | ☐ | |
| 4.1.3 | Boundary conditions correct | ☐ | ☐ | |
| 4.1.4 | Algorithm is appropriate | ☐ | ☐ | |

### 4.2 Robustness
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 4.2.1 | Input validation present | ☐ | ☐ | |
| 4.2.2 | Null pointer checks | ☐ | ☐ | |
| 4.2.3 | Array bounds checking | ☐ | ☐ | |
| 4.2.4 | Integer overflow prevented | ☐ | ☐ | |
| 4.2.5 | Division by zero prevented | ☐ | ☐ | |

### 4.3 Maintainability
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 4.3.1 | Code is readable | ☐ | ☐ | |
| 4.3.2 | Naming is clear and consistent | ☐ | ☐ | |
| 4.3.3 | No magic numbers | ☐ | ☐ | |
| 4.3.4 | Comments are accurate | ☐ | ☐ | |
| 4.3.5 | No dead code | ☐ | ☐ | |

---

## 5. Safety-Critical Checks

### 5.1 Memory Safety
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 5.1.1 | No buffer overflows | ☐ | ☐ | |
| 5.1.2 | No use-after-free | ☐ | ☐ | |
| 5.1.3 | No uninitialized variables | ☐ | ☐ | |
| 5.1.4 | No memory leaks | ☐ | ☐ | |

### 5.2 Concurrency (if applicable)
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 5.2.1 | Shared data is protected | ☐ | ☐ | |
| 5.2.2 | No race conditions | ☐ | ☐ | |
| 5.2.3 | No deadlocks | ☐ | ☐ | |
| 5.2.4 | Interrupt-safe | ☐ | ☐ | |

### 5.3 Fault Tolerance
| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 5.3.1 | Failures are detected | ☐ | ☐ | |
| 5.3.2 | Recovery actions defined | ☐ | ☐ | |
| 5.3.3 | Safe state reachable | ☐ | ☐ | |
| 5.3.4 | Faults are logged | ☐ | ☐ | |

---

## 6. Documentation

| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 6.1 | Function headers complete | ☐ | ☐ | |
| 6.2 | Parameters documented | ☐ | ☐ | |
| 6.3 | Return values documented | ☐ | ☐ | |
| 6.4 | Requirements traced | ☐ | ☐ | |
| 6.5 | Complex logic explained | ☐ | ☐ | |

---

## 7. Testing

| # | Check | Pass | N/A | Comments |
|---|-------|------|-----|----------|
| 7.1 | Unit tests exist | ☐ | ☐ | |
| 7.2 | Tests cover new code | ☐ | ☐ | |
| 7.3 | Edge cases tested | ☐ | ☐ | |
| 7.4 | Tests pass | ☐ | ☐ | |

---

## 8. Review Summary

### 8.1 Findings

| # | Severity | Description | Resolution |
|---|----------|-------------|------------|
| 1 | | | |
| 2 | | | |
| 3 | | | |

### 8.2 Metrics

| Metric | Value |
|--------|-------|
| Lines reviewed | |
| Major findings | |
| Minor findings | |
| Review time (hrs) | |

### 8.3 Disposition

| Decision | ☐ Approved | ☐ Approved with changes | ☐ Rejected |
|----------|------------|-------------------------|------------|

### 8.4 Signatures

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Author | | | |
| Reviewer 1 | | | |
| Reviewer 2 | | | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
