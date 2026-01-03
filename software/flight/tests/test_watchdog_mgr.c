/**
 * @file test_watchdog_mgr.c
 * @brief Unit tests for watchdog_mgr module
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests the watchdog manager for hardware and task watchdog functionality.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "watchdog_mgr.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    wdt_mgr_init();
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * Test Cases: Initialization
 ******************************************************************************/

static void test_wdt_init_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_init();
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_wdt_start_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_start();
    assert_int_equal(result, SMART_QSO_OK);
}

/*******************************************************************************
 * Test Cases: Task Registration
 ******************************************************************************/

static void test_register_task_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_register_task(
        WDT_TASK_MAIN_LOOP, "MainLoop", 1000);
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_register_task_invalid_id(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_register_task(
        (WdtTaskId_t)100, "Invalid", 1000);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

static void test_register_task_null_name(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_register_task(
        WDT_TASK_EPS, NULL, 1000);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_register_task_zero_period(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_register_task(
        WDT_TASK_ADCS, "ADCS", 0);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

static void test_unregister_task_success(void **state)
{
    (void)state;

    wdt_mgr_register_task(WDT_TASK_COMM, "Comm", 1000);

    SmartQsoResult_t result = wdt_mgr_unregister_task(WDT_TASK_COMM);
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_unregister_task_invalid_id(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_unregister_task((WdtTaskId_t)100);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

/*******************************************************************************
 * Test Cases: Task Check-in
 ******************************************************************************/

static void test_checkin_success(void **state)
{
    (void)state;

    wdt_mgr_register_task(WDT_TASK_TELEMETRY, "Telemetry", 1000);

    SmartQsoResult_t result = wdt_mgr_checkin(WDT_TASK_TELEMETRY);
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_checkin_invalid_id(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_checkin((WdtTaskId_t)100);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

static void test_checkin_unregistered_task(void **state)
{
    (void)state;

    /* Don't register the task */
    SmartQsoResult_t result = wdt_mgr_checkin(WDT_TASK_BEACON);
    assert_int_equal(result, SMART_QSO_ERROR);
}

/*******************************************************************************
 * Test Cases: Task Stall Detection
 ******************************************************************************/

static void test_task_not_stalled_initially(void **state)
{
    (void)state;

    wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "Main", 1000);
    wdt_mgr_checkin(WDT_TASK_MAIN_LOOP);

    assert_false(wdt_mgr_is_task_stalled(WDT_TASK_MAIN_LOOP));
}

static void test_any_task_stalled_false_initially(void **state)
{
    (void)state;

    wdt_mgr_register_task(WDT_TASK_EPS, "EPS", 1000);
    wdt_mgr_checkin(WDT_TASK_EPS);

    assert_false(wdt_mgr_any_task_stalled());
}

/*******************************************************************************
 * Test Cases: Get Task Info
 ******************************************************************************/

static void test_get_task_info_null_returns_error(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_get_task_info(WDT_TASK_MAIN_LOOP, NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_get_task_info_invalid_id(void **state)
{
    (void)state;

    WdtTaskEntry_t entry;
    SmartQsoResult_t result = wdt_mgr_get_task_info((WdtTaskId_t)100, &entry);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

static void test_get_task_info_success(void **state)
{
    (void)state;

    wdt_mgr_register_task(WDT_TASK_ADCS, "ADCS_Task", 2000);

    WdtTaskEntry_t entry;
    SmartQsoResult_t result = wdt_mgr_get_task_info(WDT_TASK_ADCS, &entry);

    assert_int_equal(result, SMART_QSO_OK);
    assert_string_equal(entry.name, "ADCS_Task");
    assert_int_equal(entry.expected_period_ms, 2000);
    assert_true(entry.active);
}

/*******************************************************************************
 * Test Cases: Get Stats
 ******************************************************************************/

static void test_get_stats_null_returns_error(void **state)
{
    (void)state;

    SmartQsoResult_t result = wdt_mgr_get_stats(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_get_stats_success(void **state)
{
    (void)state;

    WdtStats_t stats;
    SmartQsoResult_t result = wdt_mgr_get_stats(&stats);

    assert_int_equal(result, SMART_QSO_OK);
}

static void test_kick_increments_hw_kicks(void **state)
{
    (void)state;

    WdtStats_t stats_before, stats_after;
    wdt_mgr_get_stats(&stats_before);

    wdt_mgr_kick();
    wdt_mgr_kick();
    wdt_mgr_kick();

    wdt_mgr_get_stats(&stats_after);
    assert_int_equal(stats_after.hw_kicks, stats_before.hw_kicks + 3);
}

/*******************************************************************************
 * Test Cases: Pre-reset Callback
 ******************************************************************************/

static bool s_prereset_called = false;

static void mock_prereset_callback(void)
{
    s_prereset_called = true;
}

static void test_register_prereset_callback(void **state)
{
    (void)state;

    s_prereset_called = false;
    SmartQsoResult_t result = wdt_mgr_register_prereset_callback(
        mock_prereset_callback);
    assert_int_equal(result, SMART_QSO_OK);
}

/*******************************************************************************
 * Test Cases: Reset Status
 ******************************************************************************/

static void test_was_wdt_reset_returns_bool(void **state)
{
    (void)state;

    /* Just verify it returns without error */
    bool was_reset = wdt_mgr_was_wdt_reset();
    (void)was_reset;  /* May be true or false depending on state */
}

static void test_clear_reset_status(void **state)
{
    (void)state;

    /* Just verify it doesn't crash */
    wdt_mgr_clear_reset_status();
}

static void test_time_remaining(void **state)
{
    (void)state;

    uint32_t remaining = wdt_mgr_time_remaining();
    /* Should return a valid value */
    (void)remaining;
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization */
        cmocka_unit_test_setup_teardown(test_wdt_init_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_wdt_start_success,
                                        test_setup, test_teardown),

        /* Task Registration */
        cmocka_unit_test_setup_teardown(test_register_task_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_invalid_id,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_null_name,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_zero_period,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_task_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_task_invalid_id,
                                        test_setup, test_teardown),

        /* Task Check-in */
        cmocka_unit_test_setup_teardown(test_checkin_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_checkin_invalid_id,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_checkin_unregistered_task,
                                        test_setup, test_teardown),

        /* Stall Detection */
        cmocka_unit_test_setup_teardown(test_task_not_stalled_initially,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_any_task_stalled_false_initially,
                                        test_setup, test_teardown),

        /* Get Task Info */
        cmocka_unit_test_setup_teardown(test_get_task_info_null_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_info_invalid_id,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_info_success,
                                        test_setup, test_teardown),

        /* Get Stats */
        cmocka_unit_test_setup_teardown(test_get_stats_null_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_stats_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_kick_increments_hw_kicks,
                                        test_setup, test_teardown),

        /* Callbacks */
        cmocka_unit_test_setup_teardown(test_register_prereset_callback,
                                        test_setup, test_teardown),

        /* Reset Status */
        cmocka_unit_test_setup_teardown(test_was_wdt_reset_returns_bool,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_clear_reset_status,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_time_remaining,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
