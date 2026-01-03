/**
 * @file test_assert_handler.c
 * @brief Unit tests for assert_handler module
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests the JPL Power of Ten Rule 5 assertion framework.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

/* Module under test - include after undefining conflicting macros */
#undef SMART_QSO_REQUIRE
#undef SMART_QSO_ENSURE
#undef SMART_QSO_REQUIRE_NOT_NULL
#include "assert_handler.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    assert_init();
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    assert_clear_stats();
    return 0;
}

/*******************************************************************************
 * Test Cases: Initialization
 ******************************************************************************/

static void test_assert_init_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = assert_init();
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_assert_init_clears_stats(void **state)
{
    (void)state;

    /* Trigger some failures first */
    (void)assert_handle_failure("test.c", 10, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "test");

    /* Reinitialize */
    assert_init();

    /* Stats should be cleared */
    AssertStats_t stats;
    assert_get_stats(&stats);
    assert_int_equal(stats.total_failures, 0);
}

/*******************************************************************************
 * Test Cases: Handle Failure
 ******************************************************************************/

static void test_handle_warning_continues(void **state)
{
    (void)state;

    AssertAction_t action = assert_handle_failure(
        "test.c", 100, ASSERT_TYPE_PRECONDITION,
        ASSERT_SEVERITY_WARNING, "test warning");

    assert_int_equal(action, ASSERT_ACTION_CONTINUE);
}

static void test_handle_error_returns_error(void **state)
{
    (void)state;

    AssertAction_t action = assert_handle_failure(
        "test.c", 100, ASSERT_TYPE_PRECONDITION,
        ASSERT_SEVERITY_ERROR, "test error");

    assert_int_equal(action, ASSERT_ACTION_RETURN_ERROR);
}

static void test_handle_critical_triggers_safe_mode(void **state)
{
    (void)state;

    AssertAction_t action = assert_handle_failure(
        "test.c", 100, ASSERT_TYPE_STATE,
        ASSERT_SEVERITY_CRITICAL, "critical failure");

    assert_int_equal(action, ASSERT_ACTION_SAFE_MODE);
    assert_true(assert_has_critical_failures());
}

static void test_handle_failure_updates_stats(void **state)
{
    (void)state;

    assert_init();

    (void)assert_handle_failure("a.c", 1, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "warn1");
    (void)assert_handle_failure("b.c", 2, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_ERROR, "err1");
    (void)assert_handle_failure("c.c", 3, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_ERROR, "err2");

    AssertStats_t stats;
    assert_get_stats(&stats);

    assert_int_equal(stats.total_failures, 3);
    assert_int_equal(stats.warnings, 1);
    assert_int_equal(stats.errors, 2);
}

static void test_duplicate_failure_increments_count(void **state)
{
    (void)state;

    assert_init();

    /* Same file and line */
    (void)assert_handle_failure("same.c", 50, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "msg");
    (void)assert_handle_failure("same.c", 50, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "msg");
    (void)assert_handle_failure("same.c", 50, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "msg");

    /* Should only have one log entry with count=3 */
    AssertStats_t stats;
    assert_get_stats(&stats);
    assert_int_equal(stats.log_count, 1);

    AssertRecord_t record;
    assert_get_log(&record, 1, &stats.log_count);
    assert_int_equal(record.occurrence_count, 3);
}

/*******************************************************************************
 * Test Cases: Get Stats
 ******************************************************************************/

static void test_get_stats_null_returns_error(void **state)
{
    (void)state;

    SmartQsoResult_t result = assert_get_stats(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_get_stats_returns_valid_data(void **state)
{
    (void)state;

    AssertStats_t stats;
    SmartQsoResult_t result = assert_get_stats(&stats);

    assert_int_equal(result, SMART_QSO_OK);
}

/*******************************************************************************
 * Test Cases: Get Log
 ******************************************************************************/

static void test_get_log_null_returns_error(void **state)
{
    (void)state;

    uint16_t count;
    SmartQsoResult_t result = assert_get_log(NULL, 10, &count);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);

    AssertRecord_t log[10];
    result = assert_get_log(log, 10, NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_get_log_zero_entries_ok(void **state)
{
    (void)state;

    AssertRecord_t log[10];
    uint16_t count = 99;

    SmartQsoResult_t result = assert_get_log(log, 0, &count);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(count, 0);
}

static void test_get_log_returns_entries(void **state)
{
    (void)state;

    assert_init();

    (void)assert_handle_failure("first.c", 10, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "first");
    (void)assert_handle_failure("second.c", 20, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_ERROR, "second");

    AssertRecord_t log[10];
    uint16_t count;

    SmartQsoResult_t result = assert_get_log(log, 10, &count);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(count, 2);
}

/*******************************************************************************
 * Test Cases: Clear Stats
 ******************************************************************************/

static void test_clear_stats_resets_counts(void **state)
{
    (void)state;

    (void)assert_handle_failure("test.c", 1, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_WARNING, "test");

    assert_clear_stats();

    AssertStats_t stats;
    assert_get_stats(&stats);
    assert_int_equal(stats.total_failures, 0);
    assert_int_equal(stats.warnings, 0);
    assert_int_equal(stats.log_count, 0);
}

/*******************************************************************************
 * Test Cases: Critical Failures Flag
 ******************************************************************************/

static void test_has_critical_false_initially(void **state)
{
    (void)state;

    assert_init();
    assert_false(assert_has_critical_failures());
}

static void test_has_critical_true_after_critical(void **state)
{
    (void)state;

    assert_init();
    (void)assert_handle_failure("test.c", 1, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_CRITICAL, "critical");
    assert_true(assert_has_critical_failures());
}

/*******************************************************************************
 * Test Cases: Callbacks
 ******************************************************************************/

static bool s_safe_mode_called = false;

static void mock_safe_mode_callback(void)
{
    s_safe_mode_called = true;
}

static void test_safe_mode_callback_called(void **state)
{
    (void)state;

    s_safe_mode_called = false;
    assert_init();
    assert_register_safe_mode_callback(mock_safe_mode_callback);

    (void)assert_handle_failure("test.c", 1, ASSERT_TYPE_INTERNAL,
                                 ASSERT_SEVERITY_CRITICAL, "trigger");

    assert_true(s_safe_mode_called);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization */
        cmocka_unit_test_setup_teardown(test_assert_init_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_assert_init_clears_stats,
                                        test_setup, test_teardown),

        /* Handle Failure */
        cmocka_unit_test_setup_teardown(test_handle_warning_continues,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_handle_error_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_handle_critical_triggers_safe_mode,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_handle_failure_updates_stats,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_duplicate_failure_increments_count,
                                        test_setup, test_teardown),

        /* Get Stats */
        cmocka_unit_test_setup_teardown(test_get_stats_null_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_stats_returns_valid_data,
                                        test_setup, test_teardown),

        /* Get Log */
        cmocka_unit_test_setup_teardown(test_get_log_null_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_log_zero_entries_ok,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_log_returns_entries,
                                        test_setup, test_teardown),

        /* Clear Stats */
        cmocka_unit_test_setup_teardown(test_clear_stats_resets_counts,
                                        test_setup, test_teardown),

        /* Critical Failures */
        cmocka_unit_test_setup_teardown(test_has_critical_false_initially,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_has_critical_true_after_critical,
                                        test_setup, test_teardown),

        /* Callbacks */
        cmocka_unit_test_setup_teardown(test_safe_mode_callback_called,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
