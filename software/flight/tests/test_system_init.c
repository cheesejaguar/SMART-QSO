/**
 * @file test_system_init.c
 * @brief Unit tests for system initialization module
 *
 * Tests hardware initialization, clock configuration, memory tests,
 * boot reason determination, and self-test execution.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "system_init.h"
#include <string.h>

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

void setUp(void)
{
    /* Initialize system before each test */
}

void tearDown(void)
{
    /* Cleanup after each test */
}

/*******************************************************************************
 * Test Cases - Initialization
 ******************************************************************************/

void test_system_init_returns_ok(void)
{
    init_status_t status = system_init();
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

void test_system_init_sets_magic(void)
{
    system_init();
    const system_context_t *ctx = system_get_context();
    TEST_ASSERT_EQUAL_HEX32(SYSTEM_INIT_MAGIC, ctx->magic);
}

void test_system_clocks_init(void)
{
    init_status_t status = system_init_clocks();
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

void test_system_peripherals_init(void)
{
    init_status_t status = system_init_peripherals();
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

void test_system_watchdog_init(void)
{
    init_status_t status = system_init_watchdog(SYSTEM_WATCHDOG_TIMEOUT_MS);
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

/*******************************************************************************
 * Test Cases - Memory Test
 ******************************************************************************/

void test_memory_test_passes(void)
{
    init_status_t status = system_memory_test();
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

void test_memory_test_sets_flag(void)
{
    system_memory_test();
    const system_context_t *ctx = system_get_context();
    TEST_ASSERT_TRUE(ctx->selftest.ram_test_passed);
}

/*******************************************************************************
 * Test Cases - Configuration
 ******************************************************************************/

void test_load_config_returns_ok(void)
{
    init_status_t status = system_load_config();
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

/*******************************************************************************
 * Test Cases - Boot Reason
 ******************************************************************************/

void test_boot_reason_default(void)
{
    system_init();
    boot_reason_t reason = system_get_boot_reason();
    /* In simulation mode, should be power-on */
    TEST_ASSERT_EQUAL(BOOT_REASON_POWER_ON, reason);
}

void test_boot_reason_string_power_on(void)
{
    const char *str = system_boot_reason_to_string(BOOT_REASON_POWER_ON);
    TEST_ASSERT_NOT_NULL(str);
    TEST_ASSERT_EQUAL_STRING("Power-on reset", str);
}

void test_boot_reason_string_watchdog(void)
{
    const char *str = system_boot_reason_to_string(BOOT_REASON_WATCHDOG);
    TEST_ASSERT_NOT_NULL(str);
    TEST_ASSERT_EQUAL_STRING("Watchdog reset", str);
}

void test_boot_reason_string_software(void)
{
    const char *str = system_boot_reason_to_string(BOOT_REASON_SOFTWARE);
    TEST_ASSERT_EQUAL_STRING("Software reset", str);
}

void test_boot_reason_string_unknown(void)
{
    const char *str = system_boot_reason_to_string(BOOT_REASON_UNKNOWN);
    TEST_ASSERT_EQUAL_STRING("Unknown reset", str);
}

/*******************************************************************************
 * Test Cases - Self-Tests
 ******************************************************************************/

void test_selftests_with_null_returns_error(void)
{
    init_status_t status = system_run_selftests(NULL);
    TEST_ASSERT_EQUAL(INIT_ERROR_SELFTEST, status);
}

void test_selftests_passes(void)
{
    selftest_results_t results;
    init_status_t status = system_run_selftests(&results);
    TEST_ASSERT_EQUAL(INIT_OK, status);
}

void test_selftests_cpu_test_passes(void)
{
    selftest_results_t results;
    system_run_selftests(&results);
    TEST_ASSERT_TRUE(results.cpu_test_passed);
}

void test_selftests_ram_test_passes(void)
{
    selftest_results_t results;
    system_memory_test();  /* Run memory test first */
    system_run_selftests(&results);
    TEST_ASSERT_TRUE(results.ram_test_passed);
}

void test_selftests_flash_crc_passes(void)
{
    selftest_results_t results;
    system_run_selftests(&results);
    TEST_ASSERT_TRUE(results.flash_crc_passed);
}

/*******************************************************************************
 * Test Cases - Context and Status
 ******************************************************************************/

void test_get_context_not_null(void)
{
    const system_context_t *ctx = system_get_context();
    TEST_ASSERT_NOT_NULL(ctx);
}

void test_boot_count_increments(void)
{
    uint32_t initial = system_get_boot_count();
    system_init();
    uint32_t after = system_get_boot_count();
    TEST_ASSERT_EQUAL(initial + 1, after);
}

void test_uptime_returns_value(void)
{
    system_init();
    uint32_t uptime = system_get_uptime();
    TEST_ASSERT_GREATER_OR_EQUAL(0, uptime);
}

void test_status_to_string_ok(void)
{
    const char *str = system_status_to_string(INIT_OK);
    TEST_ASSERT_EQUAL_STRING("OK", str);
}

void test_status_to_string_clock_error(void)
{
    const char *str = system_status_to_string(INIT_ERROR_CLOCK);
    TEST_ASSERT_EQUAL_STRING("Clock configuration error", str);
}

void test_status_to_string_memory_error(void)
{
    const char *str = system_status_to_string(INIT_ERROR_MEMORY);
    TEST_ASSERT_EQUAL_STRING("Memory test error", str);
}

/*******************************************************************************
 * Test Cases - Safe Mode
 ******************************************************************************/

void test_should_enter_safe_mode_normal(void)
{
    system_init();
    bool safe = system_should_enter_safe_mode();
    /* Should not require safe mode under normal conditions */
    TEST_ASSERT_FALSE(safe);
}

/*******************************************************************************
 * Test Cases - Progress Callback
 ******************************************************************************/

static uint8_t g_callback_count = 0;
static void test_progress_callback(uint8_t stage, const char *name)
{
    (void)stage;
    (void)name;
    g_callback_count++;
}

void test_progress_callback_called(void)
{
    g_callback_count = 0;
    system_register_progress_callback(test_progress_callback);
    system_init();
    TEST_ASSERT_GREATER_THAN(0, g_callback_count);
}

/*******************************************************************************
 * Test Cases - Reset Request
 ******************************************************************************/

void test_reset_request_with_reason(void)
{
    /* In simulation mode, this just logs */
    system_request_reset("Test reset");
    /* No crash = success */
    TEST_PASS();
}

void test_reset_request_null_reason(void)
{
    system_request_reset(NULL);
    TEST_PASS();
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Initialization tests */
    RUN_TEST(test_system_init_returns_ok);
    RUN_TEST(test_system_init_sets_magic);
    RUN_TEST(test_system_clocks_init);
    RUN_TEST(test_system_peripherals_init);
    RUN_TEST(test_system_watchdog_init);

    /* Memory test */
    RUN_TEST(test_memory_test_passes);
    RUN_TEST(test_memory_test_sets_flag);

    /* Configuration */
    RUN_TEST(test_load_config_returns_ok);

    /* Boot reason */
    RUN_TEST(test_boot_reason_default);
    RUN_TEST(test_boot_reason_string_power_on);
    RUN_TEST(test_boot_reason_string_watchdog);
    RUN_TEST(test_boot_reason_string_software);
    RUN_TEST(test_boot_reason_string_unknown);

    /* Self-tests */
    RUN_TEST(test_selftests_with_null_returns_error);
    RUN_TEST(test_selftests_passes);
    RUN_TEST(test_selftests_cpu_test_passes);
    RUN_TEST(test_selftests_ram_test_passes);
    RUN_TEST(test_selftests_flash_crc_passes);

    /* Context and status */
    RUN_TEST(test_get_context_not_null);
    RUN_TEST(test_boot_count_increments);
    RUN_TEST(test_uptime_returns_value);
    RUN_TEST(test_status_to_string_ok);
    RUN_TEST(test_status_to_string_clock_error);
    RUN_TEST(test_status_to_string_memory_error);

    /* Safe mode */
    RUN_TEST(test_should_enter_safe_mode_normal);

    /* Progress callback */
    RUN_TEST(test_progress_callback_called);

    /* Reset request */
    RUN_TEST(test_reset_request_with_reason);
    RUN_TEST(test_reset_request_null_reason);

    return UNITY_END();
}
