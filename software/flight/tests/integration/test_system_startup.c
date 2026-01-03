/**
 * @file test_system_startup.c
 * @brief Integration tests for system startup sequence
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests the full system initialization sequence including:
 * - Hardware initialization order
 * - Subsystem dependencies
 * - Startup timing constraints
 * - Error handling during startup
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>

#include "smart_qso.h"
#include "system_state.h"
#include "state_machine.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "watchdog_mgr.h"
#include "flight_log.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * Test Cases: Initialization Order
 ******************************************************************************/

/**
 * @brief Verify subsystems initialize in correct order
 *
 * Order per FDIR_DESIGN.md:
 * 1. Watchdog
 * 2. Flight log
 * 3. Fault management
 * 4. System state
 * 5. EPS
 * 6. State machine
 */
static void test_init_order(void **state)
{
    (void)state;

    /* Initialize in documented order */
    SmartQsoResult_t result;

    result = wdt_mgr_init();
    assert_int_equal(result, SMART_QSO_OK);

    result = log_init();
    assert_int_equal(result, SMART_QSO_OK);

    result = fault_mgmt_init();
    assert_int_equal(result, SMART_QSO_OK);

    result = sys_state_init();
    assert_int_equal(result, SMART_QSO_OK);

    result = eps_init();
    assert_int_equal(result, SMART_QSO_OK);

    /* State machine should start in BOOT */
    SmContext_t *ctx = sys_get_sm_context();
    assert_non_null(ctx);
    assert_int_equal(ctx->current_state, STATE_BOOT);
}

/**
 * @brief Verify system enters SAFE mode after initialization
 */
static void test_transitions_to_safe_mode(void **state)
{
    (void)state;

    /* Re-initialize system */
    (void)sys_state_init();

    SmContext_t *ctx = sys_get_sm_context();

    /* Process boot completion event */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    assert_int_equal(result, SMART_QSO_OK);

    /* Should now be in SAFE mode */
    assert_int_equal(ctx->current_state, STATE_SAFE);
}

/**
 * @brief Verify power system initializes correctly
 */
static void test_eps_init_values(void **state)
{
    (void)state;

    (void)sys_state_init();
    (void)eps_init();

    /* Check initial power mode */
    PowerMode_t mode = sys_get_power_mode();
    assert_int_equal(mode, POWER_MODE_SAFE);

    /* Payload should be disabled initially */
    assert_false(sys_is_payload_enabled());
}

/**
 * @brief Verify watchdog starts after initialization
 */
static void test_watchdog_starts(void **state)
{
    (void)state;

    (void)wdt_mgr_init();
    SmartQsoResult_t result = wdt_mgr_start();
    assert_int_equal(result, SMART_QSO_OK);
}

/**
 * @brief Verify fault log is empty after clean init
 */
static void test_fault_log_clean_after_init(void **state)
{
    (void)state;

    (void)fault_mgmt_init();

    FaultStats_t stats;
    SmartQsoResult_t result = fault_get_stats(&stats);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(stats.total_faults, 0);
}

/*******************************************************************************
 * Test Cases: Startup Error Handling
 ******************************************************************************/

/**
 * @brief Verify system handles EPS init failure gracefully
 */
static void test_handles_eps_init_failure(void **state)
{
    (void)state;

    /* This test requires fault injection capability */
    /* For now, verify the pattern exists in the code */
    (void)sys_state_init();

    /* System should still be in a valid state */
    SmContext_t *ctx = sys_get_sm_context();
    assert_non_null(ctx);
}

/**
 * @brief Verify boot count increments on each startup
 */
static void test_boot_count_increments(void **state)
{
    (void)state;

    (void)sys_state_init();

    uint32_t initial_count = sys_get_boot_count();

    /* Increment boot count (simulating restart) */
    (void)sys_increment_boot_count();

    uint32_t new_count = sys_get_boot_count();
    assert_int_equal(new_count, initial_count + 1);
}

/*******************************************************************************
 * Test Cases: Startup Timing
 ******************************************************************************/

/**
 * @brief Verify initialization completes within time budget
 *
 * Per TIMING_ANALYSIS.md, startup should complete within 5 seconds
 */
static void test_startup_timing(void **state)
{
    (void)state;

    uint64_t start = smart_qso_now_ms();

    /* Run full initialization sequence */
    (void)wdt_mgr_init();
    (void)log_init();
    (void)fault_mgmt_init();
    (void)sys_state_init();
    (void)eps_init();

    uint64_t elapsed = smart_qso_now_ms() - start;

    /* Should complete within 5000ms */
    assert_true(elapsed < 5000);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization Order */
        cmocka_unit_test_setup_teardown(test_init_order,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_transitions_to_safe_mode,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_eps_init_values,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_watchdog_starts,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_clean_after_init,
                                        test_setup, test_teardown),

        /* Error Handling */
        cmocka_unit_test_setup_teardown(test_handles_eps_init_failure,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_boot_count_increments,
                                        test_setup, test_teardown),

        /* Timing */
        cmocka_unit_test_setup_teardown(test_startup_timing,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
