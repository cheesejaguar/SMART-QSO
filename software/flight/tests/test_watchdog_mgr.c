/**
 * @file test_watchdog_mgr.c
 * @brief Unit tests for watchdog_mgr module
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests the watchdog manager for hardware and task watchdog functionality.
 *
 * @requirement SRS-F146 External supervisor kick: a WDI falling edge every 10 s, only while every task has checked in
 * @requirement SRS-F050 Refresh the hardware watchdog (IWDG, 24.1 to 27.8 s) only at the external supervisor's WDI falling edges, every 10 s while every task checks in
 * @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on, the IWDG and the supervisor refreshed together at the initialisation checkpoints
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include "watchdog_mgr.h"
#include "fault_mgmt.h"
#include "smart_qso.h"
#include "hal/hal_timer.h"
#include "hal/hal_gpio.h"
#include "hal/hal_wdt.h"

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

/** The hardware watchdog is refreshed at each WDI falling edge, every 10 s,
 *  not at every pass (RID P3F-A-03) */
static void test_kick_increments_hw_kicks(void **state)
{
    (void)state;

    WdtStats_t stats_before, stats_after;
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_start(), SMART_QSO_OK);
    wdt_mgr_get_stats(&stats_before);

    wdt_mgr_kick();
    wdt_mgr_kick();
    wdt_mgr_kick();
    wdt_mgr_get_stats(&stats_after);
    assert_int_equal(stats_after.hw_kicks, stats_before.hw_kicks);   /* within the 10 s */

    smart_qso_sim_advance_ms(10000U);
    wdt_mgr_kick();                     /* WDI high */
    assert_true(hal_gpio_get(GPIO_PIN_WDI));
    wdt_mgr_kick();                     /* the falling edge: refreshed */
    assert_false(hal_gpio_get(GPIO_PIN_WDI));
    assert_true(wdt_mgr_time_remaining() >= (WDT_HW_TIMEOUT_MS - 1U));
    wdt_mgr_kick();
    wdt_mgr_get_stats(&stats_after);
    assert_int_equal(stats_after.hw_kicks, stats_before.hw_kicks + 1U);
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
 * Test Cases: Supervised refresh (SW-15)
 *
 * The hardware watchdog is refreshed only while every registered task checks
 * in. A stalled task stops the refresh, so the hardware resets the OBC.
 ******************************************************************************/

static int s_prereset_count = 0;

static void count_prereset(void)
{
    s_prereset_count++;
}

static int supervised_setup(void **state)
{
    (void)state;
    assert_int_equal(hal_timer_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_start(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "main", 1000U), SMART_QSO_OK);
    s_prereset_count = 0;
    assert_int_equal(wdt_mgr_register_prereset_callback(count_prereset), SMART_QSO_OK);
    return 0;
}

/** One second per step: optionally check in, then kick */
static void run_seconds(uint32_t seconds, bool checkin)
{
    for (uint32_t i = 0U; i < seconds; i++) {
        smart_qso_sim_advance_ms(1000U);
        if (checkin) {
            assert_int_equal(wdt_mgr_checkin(WDT_TASK_MAIN_LOOP), SMART_QSO_OK);
        }
        wdt_mgr_kick();
    }
}

/** Refreshed at each WDI edge: never more than one kick interval (10 s,
 *  plus the 1 s step here) behind */
static void test_healthy_task_keeps_refreshing(void **state)
{
    (void)state;
    for (uint32_t i = 0U; i < 60U; i++) {
        run_seconds(1U, true);
        assert_true(wdt_mgr_time_remaining() >= (WDT_HW_TIMEOUT_MS - 12000U));
    }
    assert_false(wdt_mgr_any_task_stalled());
    assert_int_equal(s_prereset_count, 0);
}

static void test_stalled_task_lets_hardware_reset(void **state)
{
    (void)state;
    WdtStats_t before;
    WdtStats_t after;

    assert_int_equal(wdt_mgr_get_stats(&before), SMART_QSO_OK);
    run_seconds((WDT_HW_TIMEOUT_MS / 1000U) + 10U, false);
    assert_true(wdt_mgr_any_task_stalled());
    assert_int_equal(wdt_mgr_time_remaining(), 0U);   /* hardware would reset */
    assert_int_equal(s_prereset_count, 1);            /* state saved once, first */
    assert_int_equal(wdt_mgr_get_stats(&after), SMART_QSO_OK);
    assert_true((after.hw_kicks - before.hw_kicks) <= WDT_TASK_MISS_THRESHOLD);
}

static void test_recovered_task_resumes_refresh(void **state)
{
    (void)state;
    WdtStats_t before;
    WdtStats_t after;
    run_seconds(5U, false);
    assert_true(wdt_mgr_any_task_stalled());
    run_seconds(2U, true);
    assert_false(wdt_mgr_any_task_stalled());
    assert_int_equal(wdt_mgr_get_stats(&before), SMART_QSO_OK);
    run_seconds(12U, true);
    assert_int_equal(wdt_mgr_get_stats(&after), SMART_QSO_OK);
    assert_true(after.hw_kicks > before.hw_kicks);   /* refreshed again */
    assert_true(wdt_mgr_time_remaining() > (WDT_HW_TIMEOUT_MS - 12000U));
}

/**
 * @brief RID P3F-A-03: the IWDG's last refresh is the supervisor's last WDI
 *        edge, so after a stall the IWDG runs out WDT_HW_TIMEOUT_MS after that
 *        edge, before the supervisor's 29.3 s minimum, whatever the stall's
 *        phase against the 10 s kick
 */
static void test_iwdg_acts_before_the_supervisor(void **state)
{
    (void)state;
    for (uint32_t phase = 0U; phase < 10U; phase += 3U) {
        assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
        assert_int_equal(wdt_mgr_init(), SMART_QSO_OK);
        assert_int_equal(wdt_mgr_start(), SMART_QSO_OK);
        assert_int_equal(wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "main", 1000U), SMART_QSO_OK);
        uint64_t t = 0U;
        uint64_t last_edge = 0U;
        bool level = hal_gpio_get(GPIO_PIN_WDI);
        /* Healthy for 25 s plus the phase, then the task stalls */
        for (uint32_t i = 0U; i < 25U + phase; i++) {
            run_seconds(1U, true);
            t += 1000U;
            const bool now = hal_gpio_get(GPIO_PIN_WDI);
            if (level && !now) {
                last_edge = t;
            }
            level = now;
        }
        assert_true(last_edge > 0U);
        /* The stall is detected within WDT_TASK_MISS_THRESHOLD checks; an
         * edge may still come before that, never after */
        while (wdt_mgr_time_remaining() > 0U) {
            run_seconds(1U, false);
            t += 1000U;
            const bool now = hal_gpio_get(GPIO_PIN_WDI);
            if (level && !now) {
                last_edge = t;
            }
            level = now;
        }
        assert_true(wdt_mgr_any_task_stalled());
        assert_true((t - last_edge) <= (WDT_HW_TIMEOUT_MS + 1000U));
        assert_true((t - last_edge) < 29300U);   /* the TPS3850's minimum */
    }
}

/**
 * @brief SRS-F001 checkpoints: with the boot IWDG running and the manager not
 *        started, nothing before 10 s from its last reload; then one WDI edge
 *        and a refresh together; the manager then times the supervisor from
 *        that refresh
 */
static void test_boot_checkpoints_refresh_both_together(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_init(), SMART_QSO_OK);         /* not started */
    assert_int_equal(hal_wdt_init(WDT_HW_TIMEOUT_MS, WDT_MODE_RESET), SMART_QSO_OK);
    assert_int_equal(hal_wdt_start(), SMART_QSO_OK);       /* the boot IWDG */
    smart_qso_sim_advance_ms(5000U);
    wdt_mgr_boot_checkpoint();
    assert_true(hal_wdt_remaining_ms() <= (WDT_HW_TIMEOUT_MS - 5000U));   /* nothing yet */
    smart_qso_sim_advance_ms(6000U);
    wdt_mgr_boot_checkpoint();
    assert_true(hal_wdt_remaining_ms() >= (WDT_HW_TIMEOUT_MS - 1U));       /* refreshed */
    assert_false(hal_gpio_get(GPIO_PIN_WDI));                             /* after a falling edge */
    /* Started: the supervisor's count from that refresh; no edge before 10 s */
    assert_int_equal(wdt_mgr_start(), SMART_QSO_OK);
    smart_qso_sim_advance_ms(8000U);
    wdt_mgr_kick();
    wdt_mgr_kick();
    assert_false(hal_gpio_get(GPIO_PIN_WDI));
    assert_true(hal_wdt_remaining_ms() <= (WDT_HW_TIMEOUT_MS - 8000U));
    wdt_mgr_boot_checkpoint();                                            /* started: nothing */
    assert_true(hal_wdt_remaining_ms() <= (WDT_HW_TIMEOUT_MS - 8000U));
    smart_qso_sim_advance_ms(2000U);
    wdt_mgr_kick();
    wdt_mgr_kick();
    assert_true(hal_wdt_remaining_ms() >= (WDT_HW_TIMEOUT_MS - 1U));
}

/**
 * @brief The external supervisor (TPS3850 WDI, SRS-F146): a falling edge every
 *        10 s while every task checks in, none while a task is stalled
 */
static uint32_t wdi_falling_edges(uint32_t seconds, bool checkin)
{
    uint32_t edges = 0U;
    bool level = hal_gpio_get(GPIO_PIN_WDI);
    for (uint32_t i = 0U; i < seconds; i++) {
        run_seconds(1U, checkin);
        bool now = hal_gpio_get(GPIO_PIN_WDI);
        if (level && !now) {
            edges++;
        }
        level = now;
    }
    return edges;
}

static void test_external_supervisor_kicked_only_while_healthy(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_start(), SMART_QSO_OK); /* WDI configured, the count from now */
    assert_false(hal_gpio_get(GPIO_PIN_WDI));        /* low from the start */
    assert_int_equal(wdi_falling_edges(9U, true), 0U);
    assert_int_equal(wdi_falling_edges(23U, true), 2U); /* at 11 s and 22 s */
    (void)wdi_falling_edges(5U, false);                 /* the task stalls */
    assert_true(wdt_mgr_any_task_stalled());
    assert_int_equal(wdi_falling_edges(30U, false), 0U); /* the supervisor resets the OBC */
    assert_true(wdi_falling_edges(12U, true) >= 1U);     /* recovered */
}

/*******************************************************************************
 * Test Cases: Before initialization, forced reset
 ******************************************************************************/

/** Must run first: nothing has initialized the watchdog manager yet in this
 *  process, so it neither starts the hardware nor refreshes it */
static void test_start_and_kick_refused_before_init(void **state)
{
    (void)state;
    WdtStats_t stats;

    assert_int_equal(wdt_mgr_start(), SMART_QSO_ERROR);
    wdt_mgr_kick();
    assert_int_equal(wdt_mgr_get_stats(&stats), SMART_QSO_OK);
    assert_int_equal(stats.hw_kicks, 0U);
    assert_false(stats.hw_wdt_running);
}

/** Exit statuses of the forced-reset child process */
#define CHILD_SAVED_STATE    42
#define CHILD_STILL_WAITING  43

/** The forced reset was fault-logged before anything else ran */
static bool forced_reset_logged(void)
{
    FaultLogEntry_t entry;
    return (fault_log_get_last(&entry) == SMART_QSO_OK) &&
           (strcmp(entry.description, "Forced watchdog reset") == 0);
}

/** Pre-reset callback in the child: state is "saved" by ending the child */
static void save_state_and_exit(void)
{
    exit(forced_reset_logged() ? CHILD_SAVED_STATE : EXIT_FAILURE);
}

/** The child's alarm: wdt_mgr_force_reset() was still waiting for the hardware */
static void still_waiting(int sig)
{
    (void)sig;
    exit(forced_reset_logged() ? CHILD_STILL_WAITING : EXIT_FAILURE);
}

/** Run wdt_mgr_force_reset() in a child process (it never returns); the
 *  child's exit status */
static int force_reset_in_child(WdtPreResetCallback_t callback)
{
    int status = 0;

    (void)fflush(stdout);
    (void)fflush(stderr);
    pid_t pid = fork();
    assert_true(pid >= 0);
    if (pid == 0) {
        (void)wdt_mgr_init();
        (void)wdt_mgr_register_prereset_callback(callback);
        (void)signal(SIGALRM, still_waiting);
        (void)alarm(1U);
        wdt_mgr_force_reset();
        exit(EXIT_FAILURE);   /* it returned */
    }
    assert_int_equal(waitpid(pid, &status, 0), pid);
    assert_true(WIFEXITED(status));
    return WEXITSTATUS(status);
}

/** A forced reset is logged, then state is saved through the pre-reset
 *  callback before the hardware resets the OBC
 *
 * (SRS-F051: save state before watchdog reset) */
static void test_force_reset_logs_and_saves_state(void **state)
{
    (void)state;
    assert_int_equal(force_reset_in_child(save_state_and_exit), CHILD_SAVED_STATE);
}

/** Without a pre-reset callback the forced reset is still logged, and it
 *  never returns: it waits for the hardware watchdog */
static void test_force_reset_waits_for_hardware(void **state)
{
    (void)state;
    assert_int_equal(force_reset_in_child(NULL), CHILD_STILL_WAITING);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Before initialization: must be first */
        cmocka_unit_test(test_start_and_kick_refused_before_init),

        /* Initialization */
        cmocka_unit_test_setup_teardown(test_wdt_init_success, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_wdt_start_success, test_setup, test_teardown),

        /* Task Registration */
        cmocka_unit_test_setup_teardown(test_register_task_success, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_invalid_id, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_null_name, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_zero_period, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_task_success, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_task_invalid_id, test_setup, test_teardown),

        /* Task Check-in */
        cmocka_unit_test_setup_teardown(test_checkin_success, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_checkin_invalid_id, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_checkin_unregistered_task, test_setup, test_teardown),

        /* Stall Detection */
        cmocka_unit_test_setup_teardown(test_task_not_stalled_initially, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_any_task_stalled_false_initially, test_setup, test_teardown),

        /* Get Task Info */
        cmocka_unit_test_setup_teardown(test_get_task_info_null_returns_error, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_info_invalid_id, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_info_success, test_setup, test_teardown),

        /* Get Stats */
        cmocka_unit_test_setup_teardown(test_get_stats_null_returns_error, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_get_stats_success, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_kick_increments_hw_kicks, test_setup, test_teardown),

        /* Callbacks */
        cmocka_unit_test_setup_teardown(test_register_prereset_callback, test_setup, test_teardown),

        /* Reset Status */
        cmocka_unit_test_setup_teardown(test_was_wdt_reset_returns_bool, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_clear_reset_status, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_healthy_task_keeps_refreshing, supervised_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_stalled_task_lets_hardware_reset, supervised_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_recovered_task_resumes_refresh, supervised_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_iwdg_acts_before_the_supervisor, test_setup, test_teardown),
        cmocka_unit_test(test_boot_checkpoints_refresh_both_together),
        cmocka_unit_test_setup_teardown(test_external_supervisor_kicked_only_while_healthy, supervised_setup,
                                        test_teardown),
        cmocka_unit_test_setup_teardown(test_time_remaining, test_setup, test_teardown),

        /* Forced reset (in a child process: it does not return) */
        cmocka_unit_test_setup_teardown(test_force_reset_logs_and_saves_state, test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_force_reset_waits_for_hardware, test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
