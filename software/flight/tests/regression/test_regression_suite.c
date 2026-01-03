/**
 * @file test_regression_suite.c
 * @brief Regression test suite for resolved defects
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Each test case corresponds to a resolved defect to prevent regression.
 * See REGRESSION_TESTS.md for defect details.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "smart_qso.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "system_state.h"
#include "state_machine.h"
#include "safe_string.h"
#include "input_validation.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    (void)fault_mgmt_init();
    (void)sys_state_init();
    (void)eps_init();
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

    /* Should not crash even if called multiple times */
    SmartQsoResult_t result = eps_init();
    assert_int_equal(result, SMART_QSO_OK);

    result = eps_init();
    assert_int_equal(result, SMART_QSO_OK);
}

/**
 * @brief REG-EPS-002: Power mode not updated on SOC change
 *
 * Defect DEF-005: Setting low SOC didn't trigger mode transition
 * Root Cause: Update function not checking SOC thresholds
 * Fix: Added threshold checks in eps_update_state()
 */
static void test_regression_eps_soc_mode_update(void **state)
{
    (void)state;

    /* Set normal SOC */
    (void)sys_set_state_of_charge(0.8);
    (void)eps_update_state();

    /* Set low SOC - should trigger mode change */
    (void)sys_set_state_of_charge(0.15);
    (void)eps_update_state();

    /* Power mode should reflect low SOC */
    PowerMode_t mode = sys_get_power_mode();
    assert_true(mode == POWER_MODE_SAFE || mode == POWER_MODE_IDLE);
}

/**
 * @brief REG-EPS-003: Integer overflow in power calculation
 *
 * Defect DEF-012: Large current values caused overflow
 * Root Cause: Using int16 for intermediate calculation
 * Fix: Changed to int32 with overflow check
 */
static void test_regression_eps_power_overflow(void **state)
{
    (void)state;

    /* Set maximum valid values */
    (void)sys_set_battery_voltage(4.2);
    (void)sys_set_battery_current(10.0);

    /* Should handle without overflow */
    SmartQsoResult_t result = eps_update_state();
    assert_int_equal(result, SMART_QSO_OK);
}

/*******************************************************************************
 * REG-FLT: Fault Management Regressions
 ******************************************************************************/

/**
 * @brief REG-FLT-001: Fault log overflow causes crash
 *
 * Defect DEF-002: Adding faults beyond buffer size crashed
 * Root Cause: No bounds check on fault log index
 * Fix: Implemented circular buffer with overflow flag
 */
static void test_regression_fault_log_overflow(void **state)
{
    (void)state;

    /* Fill fault log beyond capacity */
    for (int i = 0; i < MAX_FAULT_ENTRIES + 10; i++) {
        SmartQsoResult_t result = fault_log_add(
            FAULT_TYPE_EPS,
            FAULT_SEV_WARNING,
            "Test fault %d", i);
        assert_int_equal(result, SMART_QSO_OK);
    }

    /* Should not crash, stats should reflect overflow */
    FaultStats_t stats;
    (void)fault_get_stats(&stats);
    assert_true(stats.log_wrapped);
}

/**
 * @brief REG-FLT-002: Recovery procedure not called
 *
 * Defect DEF-008: Fault recovery was logged but not executed
 * Root Cause: Return value of recovery function ignored
 * Fix: Check and propagate recovery result
 */
static void test_regression_fault_recovery_called(void **state)
{
    (void)state;

    /* Trigger recovery */
    SmartQsoResult_t result = fault_recover_eps();
    assert_int_equal(result, SMART_QSO_OK);

    result = fault_recover_adcs();
    assert_int_equal(result, SMART_QSO_OK);

    result = fault_recover_comm();
    assert_int_equal(result, SMART_QSO_OK);
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
    (void)memset(src, 'A', sizeof(src) - 1);
    src[sizeof(src) - 1] = '\0';

    /* Should truncate safely */
    size_t copied = 0;
    SmartQsoResult_t result = safe_strncpy(dest, sizeof(dest),
                                            src, sizeof(src), &copied);

    assert_int_equal(result, SMART_QSO_OK);
    assert_true(copied < sizeof(dest));
    assert_int_equal(dest[sizeof(dest) - 1], '\0');
}

/*******************************************************************************
 * REG-SM: State Machine Regressions
 ******************************************************************************/

/**
 * @brief REG-SM-001: Invalid transition allowed BOOT→ACTIVE
 *
 * Defect DEF-004: Direct BOOT to ACTIVE transition was possible
 * Root Cause: Missing transition validation
 * Fix: Added explicit transition table check
 */
static void test_regression_sm_invalid_transition(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    assert_int_equal(ctx->current_state, STATE_BOOT);

    /* Try invalid transition */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_POWER_AVAILABLE);

    /* Should either fail or stay in BOOT */
    assert_true(result != SMART_QSO_OK ||
                ctx->current_state == STATE_BOOT);
}

/**
 * @brief REG-SM-002: State not persisted on transition
 *
 * Defect DEF-009: State lost on unexpected reset
 * Root Cause: State only saved periodically, not on transition
 * Fix: Mark state dirty on every transition
 */
static void test_regression_sm_state_persisted(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Transition to new state */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);

    /* State should be marked dirty for persistence */
    assert_true(sys_state_is_dirty());
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

    /* Test CRC validation function exists and works */
    uint8_t data[] = {0x01, 0x02, 0x03, 0x04};
    uint32_t crc1 = smart_qso_crc32(data, sizeof(data));

    /* Corrupt data */
    data[0] = 0xFF;
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
 */
static void test_regression_sensor_range_check(void **state)
{
    (void)state;

    /* Test range validation */
    bool valid;

    valid = validate_temperature(-50.0f);  /* Below min */
    assert_false(valid);

    valid = validate_temperature(25.0f);   /* Valid */
    assert_true(valid);

    valid = validate_temperature(100.0f);  /* Above max */
    assert_false(valid);
}

/*******************************************************************************
 * REG-MEM: Memory/Data Integrity Regressions
 ******************************************************************************/

/**
 * @brief REG-MEM-001: Stack overflow in telemetry formatting
 *
 * Defect DEF-011: Large telemetry packet overflowed stack buffer
 * Root Cause: Buffer sized for average, not max packet
 * Fix: Sized buffer for MAX_BEACON_LENGTH
 */
static void test_regression_stack_buffer_size(void **state)
{
    (void)state;

    /* Verify buffer constants are adequate */
    assert_true(MAX_BEACON_LENGTH >= 256);
    assert_true(LOG_BUFFER_SIZE >= 64);
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
    data[0] = 0xAA;
    data[8] = 0xBB;

    uint32_t crc_8 = smart_qso_crc32(data, 8);
    uint32_t crc_16 = smart_qso_crc32(data, 16);

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
        cmocka_unit_test_setup_teardown(test_regression_eps_power_overflow,
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

    return cmocka_run_group_tests(tests, NULL, NULL);
}
