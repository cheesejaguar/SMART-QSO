/**
 * @file test_flight_log.c
 * @brief Unit tests for flight_log module
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests the flight-safe logging subsystem.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "flight_log.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    log_init();
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    log_clear();
    return 0;
}

/*******************************************************************************
 * Test Cases: Initialization
 ******************************************************************************/

static void test_log_init_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_init();
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_log_init_clears_buffer(void **state)
{
    (void)state;

    /* Add some entries first */
    log_write(LOG_LEVEL_INFO, "TEST", "message 1");
    log_write(LOG_LEVEL_INFO, "TEST", "message 2");

    /* Reinitialize */
    log_init();

    /* Buffer should be empty */
    assert_int_equal(log_get_count(), 0);
}

/*******************************************************************************
 * Test Cases: Log Level
 ******************************************************************************/

static void test_set_level_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_set_level(LOG_LEVEL_WARNING);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(log_get_level(), LOG_LEVEL_WARNING);
}

static void test_set_level_invalid(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_set_level((LogLevel_t)100);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

static void test_level_filtering(void **state)
{
    (void)state;

    log_set_level(LOG_LEVEL_WARNING);

    /* These should be filtered */
    log_write(LOG_LEVEL_TRACE, "TEST", "trace");
    log_write(LOG_LEVEL_DEBUG, "TEST", "debug");
    log_write(LOG_LEVEL_INFO, "TEST", "info");

    /* These should pass */
    log_write(LOG_LEVEL_WARNING, "TEST", "warning");
    log_write(LOG_LEVEL_ERROR, "TEST", "error");

    assert_int_equal(log_get_count(), 2);
}

/*******************************************************************************
 * Test Cases: Log Write
 ******************************************************************************/

static void test_write_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_write(LOG_LEVEL_INFO, "MODULE", "Test message");
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_write_null_format_ok(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_write(LOG_LEVEL_INFO, "MODULE", NULL);
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_write_null_module_ok(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_write(LOG_LEVEL_INFO, NULL, "message");
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_write_with_format_args(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_write(LOG_LEVEL_INFO, "TEST",
                                         "Value: %d, String: %s", 42, "hello");
    assert_int_equal(result, SMART_QSO_OK);

    LogEntry_t entry;
    log_get_entry(0, &entry);
    assert_string_equal(entry.message, "Value: 42, String: hello");
}

static void test_write_increments_count(void **state)
{
    (void)state;

    assert_int_equal(log_get_count(), 0);

    log_write(LOG_LEVEL_INFO, "TEST", "msg1");
    assert_int_equal(log_get_count(), 1);

    log_write(LOG_LEVEL_INFO, "TEST", "msg2");
    assert_int_equal(log_get_count(), 2);
}

/*******************************************************************************
 * Test Cases: Get Entry
 ******************************************************************************/

static void test_get_entry_null_returns_error(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_get_entry(0, NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_get_entry_invalid_index(void **state)
{
    (void)state;

    LogEntry_t entry;
    SmartQsoResult_t result = log_get_entry(100, &entry);
    assert_int_equal(result, SMART_QSO_ERROR_PARAM);
}

static void test_get_entry_success(void **state)
{
    (void)state;

    log_write(LOG_LEVEL_WARNING, "MYMOD", "Test entry");

    LogEntry_t entry;
    SmartQsoResult_t result = log_get_entry(0, &entry);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.level, LOG_LEVEL_WARNING);
    assert_string_equal(entry.module, "MYMOD");
    assert_string_equal(entry.message, "Test entry");
}

static void test_get_entry_order(void **state)
{
    (void)state;

    log_write(LOG_LEVEL_INFO, "MOD", "First");
    log_write(LOG_LEVEL_INFO, "MOD", "Second");
    log_write(LOG_LEVEL_INFO, "MOD", "Third");

    LogEntry_t entry;

    log_get_entry(0, &entry);
    assert_string_equal(entry.message, "First");

    log_get_entry(1, &entry);
    assert_string_equal(entry.message, "Second");

    log_get_entry(2, &entry);
    assert_string_equal(entry.message, "Third");
}

/*******************************************************************************
 * Test Cases: Clear
 ******************************************************************************/

static void test_clear_success(void **state)
{
    (void)state;

    log_write(LOG_LEVEL_INFO, "TEST", "msg1");
    log_write(LOG_LEVEL_INFO, "TEST", "msg2");

    SmartQsoResult_t result = log_clear();
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(log_get_count(), 0);
}

/*******************************************************************************
 * Test Cases: Statistics
 ******************************************************************************/

static void test_get_stats_null_returns_error(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_get_stats(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

static void test_get_stats_success(void **state)
{
    (void)state;

    LogStats_t stats;
    SmartQsoResult_t result = log_get_stats(&stats);
    assert_int_equal(result, SMART_QSO_OK);
}

static void test_stats_level_counts(void **state)
{
    (void)state;

    log_set_level(LOG_LEVEL_TRACE);

    log_write(LOG_LEVEL_TRACE, "T", "t");
    log_write(LOG_LEVEL_DEBUG, "T", "d");
    log_write(LOG_LEVEL_DEBUG, "T", "d");
    log_write(LOG_LEVEL_INFO, "T", "i");
    log_write(LOG_LEVEL_INFO, "T", "i");
    log_write(LOG_LEVEL_INFO, "T", "i");
    log_write(LOG_LEVEL_WARNING, "T", "w");
    log_write(LOG_LEVEL_ERROR, "T", "e");

    LogStats_t stats;
    log_get_stats(&stats);

    assert_int_equal(stats.trace_count, 1);
    assert_int_equal(stats.debug_count, 2);
    assert_int_equal(stats.info_count, 3);
    assert_int_equal(stats.warning_count, 1);
    assert_int_equal(stats.error_count, 1);
}

/*******************************************************************************
 * Test Cases: Buffer Overflow
 ******************************************************************************/

static void test_buffer_wraps(void **state)
{
    (void)state;

    /* Fill the buffer beyond capacity */
    for (int i = 0; i < LOG_BUFFER_SIZE + 10; i++) {
        log_write(LOG_LEVEL_INFO, "TEST", "Message %d", i);
    }

    /* Should still have LOG_BUFFER_SIZE entries */
    assert_int_equal(log_get_count(), LOG_BUFFER_SIZE);

    LogStats_t stats;
    log_get_stats(&stats);
    assert_int_equal(stats.dropped_logs, 10);
}

/*******************************************************************************
 * Test Cases: Flush
 ******************************************************************************/

static void test_flush_success(void **state)
{
    (void)state;

    log_write(LOG_LEVEL_INFO, "TEST", "message");

    SmartQsoResult_t result = log_flush();
    assert_int_equal(result, SMART_QSO_OK);
}

/*******************************************************************************
 * Test Cases: Output Control
 ******************************************************************************/

static void test_set_outputs_success(void **state)
{
    (void)state;

    SmartQsoResult_t result = log_set_outputs(LOG_OUTPUT_BUFFER | LOG_OUTPUT_UART);
    assert_int_equal(result, SMART_QSO_OK);
}

static bool s_callback_called = false;
static LogEntry_t s_last_callback_entry;

static void mock_output_callback(const LogEntry_t *entry)
{
    s_callback_called = true;
    if (entry != NULL) {
        memcpy(&s_last_callback_entry, entry, sizeof(LogEntry_t));
    }
}

static void test_callback_called(void **state)
{
    (void)state;

    s_callback_called = false;
    log_register_callback(mock_output_callback);

    log_write(LOG_LEVEL_INFO, "CBTEST", "callback test");

    assert_true(s_callback_called);
    assert_string_equal(s_last_callback_entry.module, "CBTEST");
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization */
        cmocka_unit_test_setup_teardown(test_log_init_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_log_init_clears_buffer,
                                        test_setup, test_teardown),

        /* Log Level */
        cmocka_unit_test_setup_teardown(test_set_level_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_set_level_invalid,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_level_filtering,
                                        test_setup, test_teardown),

        /* Log Write */
        cmocka_unit_test_setup_teardown(test_write_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_write_null_format_ok,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_write_null_module_ok,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_write_with_format_args,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_write_increments_count,
                                        test_setup, test_teardown),

        /* Get Entry */
        cmocka_unit_test_setup_teardown(test_get_entry_null_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_entry_invalid_index,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_entry_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_entry_order,
                                        test_setup, test_teardown),

        /* Clear */
        cmocka_unit_test_setup_teardown(test_clear_success,
                                        test_setup, test_teardown),

        /* Statistics */
        cmocka_unit_test_setup_teardown(test_get_stats_null_returns_error,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_stats_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_stats_level_counts,
                                        test_setup, test_teardown),

        /* Buffer Overflow */
        cmocka_unit_test_setup_teardown(test_buffer_wraps,
                                        test_setup, test_teardown),

        /* Flush */
        cmocka_unit_test_setup_teardown(test_flush_success,
                                        test_setup, test_teardown),

        /* Output Control */
        cmocka_unit_test_setup_teardown(test_set_outputs_success,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_callback_called,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
