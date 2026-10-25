/**
 * @file test_regression_suite.c
 * @brief Regression test suite for resolved defects
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Each test case names the defect it guards against. DEF-xxx records are in
 * REGRESSION_TESTS.md; SW-xx findings are in
 * docs/funding_audit/FINDINGS_REGISTER.md.
 *
 * Not carried over from the original suite: REG-EPS-003 (DEF-012, integer
 * overflow in eps_update_state()'s power calculation); that function and
 * its calculation no longer exist.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "smart_qso.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "system_state.h"
#include "state_machine.h"
#include "payload_guard.h"
#include "safe_string.h"
#include "input_validation.h"
#include "sensors.h"
#include "beacon.h"
#include "flight_log.h"

/** SOC main.c's simulation starts with */
#define START_SOC 0.75

/*******************************************************************************
 * Helpers
 ******************************************************************************/

/** Raise an event with the system's state machine; return the result */
static SmResult_t raise_event(SmEvent_t event)
{
    SmResult_t result = SM_RESULT_INVALID_STATE;
    assert_int_equal(sm_process_event(sys_get_sm_context(), event, &result), SMART_QSO_OK);
    return result;
}

/** BOOT -> DETUMBLE -> IDLE, as main.c boots */
static void boot_to_idle(void)
{
    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);
    assert_int_equal(raise_event(EVENT_DETUMBLE_COMPLETE), SM_RESULT_OK);
    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
}

static bool temperature_valid(float celsius)
{
    bool valid = false;
    assert_int_equal(input_validate_temperature(celsius, &valid), SMART_QSO_OK);
    return valid;
}

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    (void)fault_mgmt_init();
    (void)fault_log_clear();
    payload_guard_init();
    (void)eps_init();
    eps_update_soc(START_SOC);
    (void)sys_state_init();
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * REG-EPS: EPS Subsystem Regressions
 ******************************************************************************/

/**
 * @brief REG-EPS-001: NULL pointer in eps_init when config missing
 *
 * Defect DEF-001: eps_init() crashed when called before config loaded
 * Root Cause: Missing NULL check on config pointer
 * Fix: Added NULL check with default config fallback
 */
static void test_regression_eps_null_config(void **state)
{
    (void)state;
    const char *path = getenv(EPS_CONFIG_FILE_ENV);
    (void)remove(((path != NULL) && (path[0] != '\0')) ? path : EPS_CONFIG_FILE);

    /* No configuration saved: defaults, and safe to call again */
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_false(eps_is_payload_enabled());
    assert_int_equal(eps_init(), SMART_QSO_OK);
}

/**
 * @brief REG-EPS-002: Power mode not updated on SOC change
 *
 * Defect DEF-005: Setting low SOC didn't trigger mode transition
 * Root Cause: Update function not checking SOC thresholds
 * Fix: Added threshold checks in eps_update_state(); since SW-14 the
 *      thresholds are sm_power_event()'s and the state machine sets the mode
 */
static void test_regression_eps_soc_mode_update(void **state)
{
    (void)state;
    boot_to_idle();

    /* Main loop pass at 15 % SOC */
    assert_int_equal(sys_set_state_of_charge(0.15), SMART_QSO_OK);
    SmEvent_t event = sm_power_event(sys_get_operational_state(), sys_get_state_of_charge(), true);
    assert_int_equal(event, EVENT_POWER_CRITICAL);
    assert_int_equal(raise_event(event), SM_RESULT_OK);

    assert_int_equal(sys_get_operational_state(), STATE_SAFE);
    assert_int_equal(sys_get_power_mode(), POWER_MODE_SAFE);
}

/*******************************************************************************
 * REG-FLT: Fault Management Regressions
 ******************************************************************************/

/**
 * @brief REG-FLT-001: Fault log overflow causes crash
 *
 * Defect DEF-002: Adding faults beyond buffer size crashed
 * Root Cause: No bounds check on fault log index
 * Fix: Full log drops its oldest entry for the new one
 */
static void test_regression_fault_log_overflow(void **state)
{
    (void)state;
    char desc[SMART_QSO_FAULT_DESC_LEN];

    for (int i = 0; i < (SMART_QSO_MAX_FAULT_ENTRIES + 10); i++) {
        (void)snprintf(desc, sizeof(desc), "Test fault %d", i);
        assert_int_equal(fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING,
                                       desc, START_SOC), SMART_QSO_OK);
    }

    FaultLogEntry_t entry;
    assert_int_equal(fault_log_get_count(), (size_t)SMART_QSO_MAX_FAULT_ENTRIES);
    assert_int_equal(fault_log_get_entry(0U, &entry), SMART_QSO_OK);
    assert_string_equal(entry.description, "Test fault 10");    /* oldest dropped */
    assert_int_equal(fault_log_get_last(&entry), SMART_QSO_OK);
    (void)snprintf(desc, sizeof(desc), "Test fault %d", SMART_QSO_MAX_FAULT_ENTRIES + 9);
    assert_string_equal(entry.description, desc);
}

/**
 * @brief REG-FLT-002: Recovery procedure not called
 *
 * Defect DEF-008: Fault recovery was logged but not executed
 * Root Cause: Return value of recovery function ignored
 * Fix: Check and propagate recovery result
 *
 * Also SW-14 follow-up: fault_recover_thermal() and fault_recover_power()
 * set the EPS power mode behind the state machine, so the two disagreed and
 * the mode guards did not apply. Recovery must change the mode the state
 * machine is in, and the EPS must follow.
 */
static void test_regression_fault_recovery_called(void **state)
{
    (void)state;
    boot_to_idle();

    assert_int_equal(fault_recover_power(0.20), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_SAFE);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);

    assert_int_equal(fault_recover_power(START_SOC), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);

    assert_int_equal(raise_event(EVENT_AI_READY), SM_RESULT_OK);
    assert_true(eps_is_payload_enabled());
    assert_int_equal(fault_recover_thermal(START_SOC), SMART_QSO_OK);
    assert_false(eps_is_payload_enabled());
    assert_int_equal(sys_get_operational_state(), STATE_SAFE);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
}

/*******************************************************************************
 * REG-COM: Communication Regressions
 ******************************************************************************/

/**
 * @brief REG-COM-001: UART buffer overflow on long message
 *
 * Defect DEF-003: Long beacon text caused buffer overflow
 * Root Cause: No length check before copy
 * Fix: Use safe_strncpy with explicit bounds
 */
static void test_regression_uart_buffer_overflow(void **state)
{
    (void)state;

    char dest[32];
    char src[100];

    /* Fill source with long string */
    (void)memset(src, 'A', sizeof(src) - 1U);
    src[sizeof(src) - 1U] = '\0';

    /* Should truncate safely */
    size_t copied = 0U;
    SmartQsoResult_t result = safe_strncpy(dest, sizeof(dest), src, sizeof(src), &copied);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(copied, sizeof(dest) - 1U);
    assert_int_equal(dest[sizeof(dest) - 1U], '\0');
}

/*******************************************************************************
 * REG-SM: State Machine Regressions
 ******************************************************************************/

/**
 * @brief REG-SM-001: Invalid transition allowed BOOT->ACTIVE
 *
 * Defect DEF-004: Direct BOOT to ACTIVE transition was possible
 * Root Cause: Missing transition validation
 * Fix: Added explicit transition table check
 */
static void test_regression_sm_invalid_transition(void **state)
{
    (void)state;
    eps_update_soc(0.90);
    assert_int_equal(sys_get_operational_state(), STATE_BOOT);

    assert_int_equal(raise_event(EVENT_AI_READY), SM_RESULT_NO_TRANSITION);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_NO_TRANSITION);

    assert_int_equal(sys_get_operational_state(), STATE_BOOT);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
    assert_false(eps_is_payload_enabled());
}

/**
 * @brief REG-SM-002: State not persisted on transition
 *
 * Defect DEF-009: State lost on unexpected reset
 * Root Cause: State only saved periodically, not on transition
 * Fix: Mark state dirty on every transition
 *
 * The state that a transition changes and that survives a reset is the EPS
 * state (power mode and load switches; the operational state restarts at
 * BOOT on every boot). It must be saved by the transition itself, not later.
 */
static void test_regression_sm_state_persisted(void **state)
{
    (void)state;
    boot_to_idle();
    assert_int_equal(raise_event(EVENT_GROUND_CMD_SAFE), SM_RESULT_OK);

    /* Read back what is in non-volatile storage now */
    assert_int_equal(eps_load_config(), SMART_QSO_OK);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
}

/*******************************************************************************
 * REG-SEN: Sensor Regressions
 ******************************************************************************/

/**
 * @brief REG-SEN-001: Sensor CRC not validated on read
 *
 * Defect DEF-006: Corrupted sensor data was accepted
 * Root Cause: CRC check bypassed in fast path
 * Fix: Made CRC check mandatory
 */
static void test_regression_sensor_crc_validated(void **state)
{
    (void)state;

    uint8_t data[] = {0x01U, 0x02U, 0x03U, 0x04U};
    uint32_t crc1 = smart_qso_crc32(data, sizeof(data));

    /* Corrupt data */
    data[0] = 0xFFU;
    uint32_t crc2 = smart_qso_crc32(data, sizeof(data));

    /* CRCs should differ */
    assert_int_not_equal(crc1, crc2);
}

/**
 * @brief REG-SEN-002: Range check uses wrong bounds
 *
 * Defect DEF-010: Temperature accepted outside valid range
 * Root Cause: Off-by-one error in bounds check
 * Fix: Corrected range validation logic
 *
 * The telemetry limits are -50 and +100 C, both inclusive
 * (software/rust/sqso-ffi/src/input_validation.rs TLM_MIN_TEMP, TLM_MAX_TEMP).
 */
static void test_regression_sensor_range_check(void **state)
{
    (void)state;

    assert_false(temperature_valid(-50.5f));   /* below min */
    assert_true(temperature_valid(-50.0f));
    assert_true(temperature_valid(25.0f));
    assert_true(temperature_valid(100.0f));
    assert_false(temperature_valid(100.5f));   /* above max */
    assert_false(temperature_valid(NAN));
}

/*******************************************************************************
 * REG-MEM: Memory/Data Integrity Regressions
 ******************************************************************************/

/**
 * @brief REG-MEM-001: Stack overflow in telemetry formatting
 *
 * Defect DEF-011: Large telemetry packet overflowed stack buffer
 * Root Cause: Buffer sized for average, not max packet
 * Fix: Sized buffer for the maximum beacon length
 */
static void test_regression_stack_buffer_size(void **state)
{
    (void)state;

    /* Buffer constants: the beacon info field holds 256 bytes (SRS-F025) */
    assert_true(BEACON_MAX_PAYLOAD_LEN >= 256);
    assert_true(LOG_BUFFER_SIZE >= 64U);

    /* Formatting every sensor into a short buffer stays inside it */
    assert_int_equal(sensors_init(), SMART_QSO_OK);
    assert_int_equal(sensors_load_defaults(), SMART_QSO_OK);
    char small[24];
    (void)memset(small, 'Z', sizeof(small));
    size_t written = sensors_format_telemetry(small, sizeof(small));
    assert_true(written < sizeof(small));
    assert_true(memchr(small, '\0', sizeof(small)) != NULL);
}

/**
 * @brief REG-MEM-002: CRC calculated over wrong length
 *
 * Defect DEF-013: CRC included padding bytes
 * Root Cause: sizeof(struct) vs actual data length
 * Fix: Use explicit length parameter
 */
static void test_regression_crc_length(void **state)
{
    (void)state;

    /* CRC of partial data should differ from full data */
    uint8_t data[16] = {0};
    data[0] = 0xAAU;
    data[8] = 0xBBU;

    uint32_t crc_8 = smart_qso_crc32(data, 8U);
    uint32_t crc_16 = smart_qso_crc32(data, 16U);

    assert_int_not_equal(crc_8, crc_16);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* EPS Regressions */
        cmocka_unit_test_setup_teardown(test_regression_eps_null_config,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_regression_eps_soc_mode_update,
                                        test_setup, test_teardown),

        /* Fault Management Regressions */
        cmocka_unit_test_setup_teardown(test_regression_fault_log_overflow,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_regression_fault_recovery_called,
                                        test_setup, test_teardown),

        /* Communication Regressions */
        cmocka_unit_test_setup_teardown(test_regression_uart_buffer_overflow,
                                        test_setup, test_teardown),

        /* State Machine Regressions */
        cmocka_unit_test_setup_teardown(test_regression_sm_invalid_transition,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_regression_sm_state_persisted,
                                        test_setup, test_teardown),

        /* Sensor Regressions */
        cmocka_unit_test_setup_teardown(test_regression_sensor_crc_validated,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_regression_sensor_range_check,
                                        test_setup, test_teardown),

        /* Memory Regressions */
        cmocka_unit_test_setup_teardown(test_regression_stack_buffer_size,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_regression_crc_length,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests_name("Regression Suite", tests, NULL, NULL);
}
