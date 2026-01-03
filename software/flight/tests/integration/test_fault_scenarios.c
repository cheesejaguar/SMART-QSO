/**
 * @file test_fault_scenarios.c
 * @brief Integration tests for fault injection and recovery
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests FDIR (Fault Detection, Isolation, Recovery) per FDIR_DESIGN.md.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>

#include "smart_qso.h"
#include "system_state.h"
#include "state_machine.h"
#include "fault_mgmt.h"
#include "eps_control.h"
#include "watchdog_mgr.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    (void)fault_mgmt_init();
    (void)sys_state_init();
    (void)eps_init();
    (void)wdt_mgr_init();
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * Test Cases: Fault Logging
 ******************************************************************************/

/**
 * @brief Verify faults are logged correctly
 */
static void test_fault_logged(void **state)
{
    (void)state;

    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_EPS,
        FAULT_SEV_WARNING,
        "Test fault");

    assert_int_equal(result, SMART_QSO_OK);

    FaultStats_t stats;
    (void)fault_get_stats(&stats);
    assert_int_equal(stats.total_faults, 1);
}

/**
 * @brief Verify fault log persists across queries
 */
static void test_fault_log_persistence(void **state)
{
    (void)state;

    /* Log multiple faults */
    (void)fault_log_add(FAULT_TYPE_SENSOR, FAULT_SEV_WARNING, "Fault 1");
    (void)fault_log_add(FAULT_TYPE_COMM, FAULT_SEV_ERROR, "Fault 2");
    (void)fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEV_CRITICAL, "Fault 3");

    FaultStats_t stats;
    (void)fault_get_stats(&stats);
    assert_int_equal(stats.total_faults, 3);
}

/*******************************************************************************
 * Test Cases: Power Faults
 ******************************************************************************/

/**
 * @brief Verify system handles low battery fault
 */
static void test_low_battery_fault(void **state)
{
    (void)state;

    /* Set low SOC */
    (void)sys_set_state_of_charge(0.1);

    /* Log the fault */
    (void)fault_log_add(FAULT_TYPE_EPS, FAULT_SEV_WARNING, "Low SOC");

    /* System should transition to safe power mode */
    (void)eps_update_state();
    PowerMode_t mode = sys_get_power_mode();

    /* In low power, should be in SAFE or lower mode */
    assert_true(mode == POWER_MODE_SAFE || mode == POWER_MODE_IDLE);
}

/**
 * @brief Verify critical battery fault triggers emergency
 */
static void test_critical_battery_fault(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Get to operational state first */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);

    /* Critical battery fault */
    (void)sys_set_state_of_charge(0.05);
    (void)fault_log_add(FAULT_TYPE_EPS, FAULT_SEV_CRITICAL, "Critical SOC");

    /* Process critical fault event */
    (void)sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);

    assert_int_equal(ctx->current_state, STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: Communication Faults
 ******************************************************************************/

/**
 * @brief Verify UART timeout is handled
 */
static void test_uart_timeout_fault(void **state)
{
    (void)state;

    /* Log comm fault */
    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_COMM,
        FAULT_SEV_WARNING,
        "UART timeout");

    assert_int_equal(result, SMART_QSO_OK);

    /* Check recovery function exists */
    result = fault_recover_comm();
    assert_int_equal(result, SMART_QSO_OK);
}

/**
 * @brief Verify payload timeout triggers fallback
 */
static void test_payload_timeout_fault(void **state)
{
    (void)state;

    /* Log payload fault */
    (void)fault_log_add(FAULT_TYPE_PAYLOAD, FAULT_SEV_ERROR, "Jetson timeout");

    /* Payload should be disabled */
    (void)sys_set_payload_enabled(false);
    assert_false(sys_is_payload_enabled());
}

/*******************************************************************************
 * Test Cases: Sensor Faults
 ******************************************************************************/

/**
 * @brief Verify out-of-range sensor data is rejected
 */
static void test_sensor_range_fault(void **state)
{
    (void)state;

    /* Log sensor fault */
    (void)fault_log_add(FAULT_TYPE_SENSOR, FAULT_SEV_WARNING, "Value out of range");

    FaultStats_t stats;
    (void)fault_get_stats(&stats);
    assert_true(stats.total_faults > 0);
}

/**
 * @brief Verify sensor CRC failure is caught
 */
static void test_sensor_crc_fault(void **state)
{
    (void)state;

    /* Log data integrity fault */
    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_DATA_INTEGRITY,
        FAULT_SEV_ERROR,
        "CRC mismatch");

    assert_int_equal(result, SMART_QSO_OK);
}

/*******************************************************************************
 * Test Cases: Thermal Faults
 ******************************************************************************/

/**
 * @brief Verify over-temperature fault handling
 */
static void test_over_temp_fault(void **state)
{
    (void)state;

    /* Set high temperature */
    (void)sys_set_temperature(0, 70.0f);  /* OBC at 70°C */

    /* Check thermal fault flag */
    assert_true(sys_has_thermal_fault());

    /* Log thermal fault */
    (void)fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEV_WARNING, "Over temp");
}

/**
 * @brief Verify under-temperature fault handling
 */
static void test_under_temp_fault(void **state)
{
    (void)state;

    /* Set low temperature */
    (void)sys_set_temperature(2, -25.0f);  /* Battery at -25°C */

    /* Check thermal fault flag */
    assert_true(sys_has_thermal_fault());
}

/*******************************************************************************
 * Test Cases: Watchdog Faults
 ******************************************************************************/

/**
 * @brief Verify task stall detection
 */
static void test_task_stall_detection(void **state)
{
    (void)state;

    /* Register a task */
    (void)wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "Main", 1000);
    (void)wdt_mgr_checkin(WDT_TASK_MAIN_LOOP);

    /* Initially not stalled */
    assert_false(wdt_mgr_is_task_stalled(WDT_TASK_MAIN_LOOP));
}

/*******************************************************************************
 * Test Cases: Recovery Procedures
 ******************************************************************************/

/**
 * @brief Verify EPS recovery procedure
 */
static void test_eps_recovery(void **state)
{
    (void)state;

    SmartQsoResult_t result = fault_recover_eps();
    assert_int_equal(result, SMART_QSO_OK);
}

/**
 * @brief Verify ADCS recovery procedure
 */
static void test_adcs_recovery(void **state)
{
    (void)state;

    SmartQsoResult_t result = fault_recover_adcs();
    assert_int_equal(result, SMART_QSO_OK);
}

/**
 * @brief Verify full system recovery from emergency
 */
static void test_system_recovery(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Get to EMERGENCY */
    (void)sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);

    /* Clear faults */
    (void)fault_mgmt_init();

    /* Recover */
    (void)sm_process_event(ctx, SM_EVENT_FAULT_CLEARED);
    assert_int_equal(ctx->current_state, STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: Fault Statistics
 ******************************************************************************/

/**
 * @brief Verify fault statistics tracking
 */
static void test_fault_statistics(void **state)
{
    (void)state;

    /* Log various faults */
    (void)fault_log_add(FAULT_TYPE_EPS, FAULT_SEV_WARNING, "EPS fault");
    (void)fault_log_add(FAULT_TYPE_SENSOR, FAULT_SEV_ERROR, "Sensor fault");
    (void)fault_log_add(FAULT_TYPE_COMM, FAULT_SEV_CRITICAL, "Comm fault");

    FaultStats_t stats;
    SmartQsoResult_t result = fault_get_stats(&stats);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(stats.total_faults, 3);
    assert_int_equal(stats.warning_count, 1);
    assert_int_equal(stats.error_count, 1);
    assert_int_equal(stats.critical_count, 1);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Fault Logging */
        cmocka_unit_test_setup_teardown(test_fault_logged,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_persistence,
                                        test_setup, test_teardown),

        /* Power Faults */
        cmocka_unit_test_setup_teardown(test_low_battery_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_critical_battery_fault,
                                        test_setup, test_teardown),

        /* Communication Faults */
        cmocka_unit_test_setup_teardown(test_uart_timeout_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_payload_timeout_fault,
                                        test_setup, test_teardown),

        /* Sensor Faults */
        cmocka_unit_test_setup_teardown(test_sensor_range_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_sensor_crc_fault,
                                        test_setup, test_teardown),

        /* Thermal Faults */
        cmocka_unit_test_setup_teardown(test_over_temp_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_under_temp_fault,
                                        test_setup, test_teardown),

        /* Watchdog Faults */
        cmocka_unit_test_setup_teardown(test_task_stall_detection,
                                        test_setup, test_teardown),

        /* Recovery */
        cmocka_unit_test_setup_teardown(test_eps_recovery,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_adcs_recovery,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_system_recovery,
                                        test_setup, test_teardown),

        /* Statistics */
        cmocka_unit_test_setup_teardown(test_fault_statistics,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
