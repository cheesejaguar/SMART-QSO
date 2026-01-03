# Regression Test Suite

**Document ID**: SMART-QSO-REG-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Engineering

## 1. Purpose

This document tracks regression tests added for each resolved defect or anomaly. Each test ensures the defect does not recur in future releases.

## 2. Regression Test Policy

Per NPR 7150.2 and NASA Ames SmallSat heritage:

1. **Every closed defect** must have an associated regression test
2. **Tests must be automated** and run as part of CI/CD
3. **Tests must verify** the specific fix, not just general behavior
4. **Tests must document** the original defect ID and description

## 3. Test Naming Convention

```
test_regression_<DEFECT_ID>_<short_description>
```

Example: `test_regression_DEF001_null_pointer_in_eps_init`

## 4. Regression Test Catalog

### 4.1 EPS Subsystem

| Test ID | Defect ID | Description | Status |
|---------|-----------|-------------|--------|
| REG-EPS-001 | DEF-001 | NULL pointer in eps_init when config missing | PASS |
| REG-EPS-002 | DEF-005 | Power mode not updated on SOC change | PASS |
| REG-EPS-003 | DEF-012 | Integer overflow in power calculation | PASS |

### 4.2 Fault Management

| Test ID | Defect ID | Description | Status |
|---------|-----------|-------------|--------|
| REG-FLT-001 | DEF-002 | Fault log overflow causes crash | PASS |
| REG-FLT-002 | DEF-008 | Recovery procedure not called | PASS |

### 4.3 Communications

| Test ID | Defect ID | Description | Status |
|---------|-----------|-------------|--------|
| REG-COM-001 | DEF-003 | UART buffer overflow on long message | PASS |
| REG-COM-002 | DEF-007 | Timeout not reset after successful RX | PASS |

### 4.4 State Machine

| Test ID | Defect ID | Description | Status |
|---------|-----------|-------------|--------|
| REG-SM-001 | DEF-004 | Invalid transition allowed BOOT→ACTIVE | PASS |
| REG-SM-002 | DEF-009 | State not persisted on transition | PASS |

### 4.5 Sensors

| Test ID | Defect ID | Description | Status |
|---------|-----------|-------------|--------|
| REG-SEN-001 | DEF-006 | Sensor CRC not validated on read | PASS |
| REG-SEN-002 | DEF-010 | Range check uses wrong bounds | PASS |

### 4.6 Memory/Data Integrity

| Test ID | Defect ID | Description | Status |
|---------|-----------|-------------|--------|
| REG-MEM-001 | DEF-011 | Stack overflow in telemetry formatting | PASS |
| REG-MEM-002 | DEF-013 | CRC calculated over wrong length | PASS |

## 5. Running Regression Tests

```bash
# Run all regression tests
cd software/flight
cmake -B build
cmake --build build
ctest --test-dir build -R regression

# Run specific regression test
ctest --test-dir build -R REG-EPS-001
```

## 6. Adding New Regression Tests

1. Create defect record in issue tracker
2. Identify root cause and fix
3. Create regression test in `tests/regression/`
4. Update this document with test entry
5. Verify test passes with fix applied
6. Verify test fails with fix reverted (optional)
7. Commit test with defect fix

## 7. Regression Test Template

```c
/**
 * @file test_regression_DEFxxx.c
 * @brief Regression test for DEF-xxx
 *
 * Defect: <Brief description>
 * Root Cause: <Technical cause>
 * Fix: <How it was fixed>
 */

static void test_regression_DEFxxx_description(void **state)
{
    (void)state;

    /* Setup: Create conditions that triggered defect */

    /* Action: Perform operation that caused failure */

    /* Assert: Verify defect no longer occurs */
}
```

## 8. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |

---

*Following NASA Ames SmallSat regression testing heritage.*
