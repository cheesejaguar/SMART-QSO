/**
 * @file test_scheduler.c
 * @brief Unit tests for time-triggered scheduler module
 *
 * Tests task registration and lookup, enable/disable, suspend/resume,
 * statistics accessors, period management, forced execution and the tick.
 * Ported from Unity to CMocka (SW-09).
 *
 * Not exercised: deadline monitoring and CPU utilization. The scheduler's
 * time source, sched_get_time_us(), returns 0 in this build, so no task can
 * overrun its deadline and utilization stays 0.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <stdio.h>
#include <string.h>

#include "scheduler.h"

/*******************************************************************************
 * Test Data
 ******************************************************************************/

static volatile uint32_t g_task1_count = 0;
static volatile uint32_t g_task2_count = 0;
static volatile uint32_t g_deadline_miss_count = 0;

/*******************************************************************************
 * Test Task Functions
 ******************************************************************************/

static void test_task1_func(void)
{
    g_task1_count++;
}

static void test_task2_func(void)
{
    g_task2_count++;
}

static void deadline_miss_callback(task_handle_t handle, uint32_t overrun_us)
{
    (void)handle;
    (void)overrun_us;
    g_deadline_miss_count++;
}

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static void setUp(void)
{
    scheduler_init();
    g_task1_count = 0;
    g_task2_count = 0;
    g_deadline_miss_count = 0;
}

static void tearDown(void)
{
    scheduler_stop();
}

/*******************************************************************************
 * Test Cases - Initialization
 ******************************************************************************/

static void test_scheduler_init_returns_ok(void **cm_state)
{
    (void)cm_state;
    sched_status_t status = scheduler_init();
    assert_int_equal(SCHED_OK, status);
}

static void test_scheduler_not_running_initially(void **cm_state)
{
    (void)cm_state;
    assert_false(scheduler_is_running());
}

static void test_scheduler_tick_count_zero(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(0, scheduler_get_tick_count());
}

/*******************************************************************************
 * Test Cases - Task Registration
 ******************************************************************************/

static void test_register_task_null_config(void **cm_state)
{
    (void)cm_state;
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(NULL, &handle);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_register_task_null_handle(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Test",
        .func = test_task1_func,
        .period_ms = 100,
        .priority = SCHED_PRIORITY_NORMAL
    };
    sched_status_t status = scheduler_register_task(&config, NULL);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_register_task_null_func(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Test",
        .func = NULL,
        .period_ms = 100
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_register_task_period_too_small(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Test",
        .func = test_task1_func,
        .period_ms = 1  /* Below minimum */
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_register_task_period_too_large(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Test",
        .func = test_task1_func,
        .period_ms = 100000  /* Above maximum */
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_register_task_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .priority = SCHED_PRIORITY_NORMAL,
        .enabled = true
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    assert_int_equal(SCHED_OK, status);
    assert_int_not_equal(SCHED_INVALID_HANDLE, handle);
}

static void test_register_duplicate_name(void **cm_state)
{
    (void)cm_state;
    task_config_t config1 = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_config_t config2 = {
        .name = "Task1",  /* Same name */
        .func = test_task2_func,
        .period_ms = 200
    };
    task_handle_t handle1, handle2;
    scheduler_register_task(&config1, &handle1);
    sched_status_t status = scheduler_register_task(&config2, &handle2);
    assert_int_equal(SCHED_ERROR_ALREADY_EXISTS, status);
}

static void test_register_max_tasks(void **cm_state)
{
    (void)cm_state;
    task_handle_t handles[SCHED_MAX_TASKS + 1];
    sched_status_t status;

    for (uint8_t i = 0; i < SCHED_MAX_TASKS; i++) {
        task_config_t config = {
            .func = test_task1_func,
            .period_ms = 100
        };
        snprintf(config.name, SCHED_MAX_TASK_NAME, "Task%d", i);
        status = scheduler_register_task(&config, &handles[i]);
        assert_int_equal(SCHED_OK, status);
    }

    /* One more should fail */
    task_config_t extra = {
        .name = "Extra",
        .func = test_task1_func,
        .period_ms = 100
    };
    status = scheduler_register_task(&extra, &handles[SCHED_MAX_TASKS]);
    assert_int_equal(SCHED_ERROR_TABLE_FULL, status);
}

/*******************************************************************************
 * Test Cases - Task Unregistration
 ******************************************************************************/

static void test_unregister_invalid_handle(void **cm_state)
{
    (void)cm_state;
    sched_status_t status = scheduler_unregister_task(SCHED_MAX_TASKS + 1);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_unregister_not_registered(void **cm_state)
{
    (void)cm_state;
    sched_status_t status = scheduler_unregister_task(0);
    assert_int_equal(SCHED_ERROR_NOT_FOUND, status);
}

static void test_unregister_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_unregister_task(handle);
    assert_int_equal(SCHED_OK, status);
}

/*******************************************************************************
 * Test Cases - Enable/Disable
 ******************************************************************************/

static void test_enable_task_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = false
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_enable_task(handle);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(TASK_STATE_READY, scheduler_get_task_state(handle));
}

static void test_disable_task_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_disable_task(handle);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(TASK_STATE_INACTIVE, scheduler_get_task_state(handle));
}

/*******************************************************************************
 * Test Cases - Suspend/Resume
 ******************************************************************************/

static void test_suspend_task_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_suspend_task(handle);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(TASK_STATE_SUSPENDED, scheduler_get_task_state(handle));
}

static void test_resume_task_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    scheduler_suspend_task(handle);
    sched_status_t status = scheduler_resume_task(handle);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(TASK_STATE_READY, scheduler_get_task_state(handle));
}

/*******************************************************************************
 * Test Cases - Task Lookup
 ******************************************************************************/

static void test_get_handle_by_name_found(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "MyTask",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t registered;
    scheduler_register_task(&config, &registered);

    task_handle_t found = scheduler_get_handle_by_name("MyTask");
    assert_int_equal(registered, found);
}

static void test_get_handle_by_name_not_found(void **cm_state)
{
    (void)cm_state;
    task_handle_t found = scheduler_get_handle_by_name("NonExistent");
    assert_int_equal(SCHED_INVALID_HANDLE, found);
}

static void test_get_handle_by_name_null(void **cm_state)
{
    (void)cm_state;
    task_handle_t found = scheduler_get_handle_by_name(NULL);
    assert_int_equal(SCHED_INVALID_HANDLE, found);
}

static void test_get_task_name_valid(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "MyTask",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    const char *name = scheduler_get_task_name(handle);
    assert_non_null(name);
    assert_string_equal("MyTask", name);
}

static void test_get_task_name_invalid(void **cm_state)
{
    (void)cm_state;
    const char *name = scheduler_get_task_name(SCHED_INVALID_HANDLE);
    assert_null(name);
}

/*******************************************************************************
 * Test Cases - Statistics
 ******************************************************************************/

static void test_get_task_stats_null(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_get_task_stats(handle, NULL);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_get_task_stats_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    task_stats_t stats;
    sched_status_t status = scheduler_get_task_stats(handle, &stats);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(0, stats.run_count);
}

static void test_reset_task_stats(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_reset_task_stats(handle);
    assert_int_equal(SCHED_OK, status);
}

static void test_get_scheduler_stats_null(void **cm_state)
{
    (void)cm_state;
    sched_status_t status = scheduler_get_stats(NULL);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_get_scheduler_stats_success(void **cm_state)
{
    (void)cm_state;
    sched_stats_t stats;
    sched_status_t status = scheduler_get_stats(&stats);
    assert_int_equal(SCHED_OK, status);
}

static void test_cpu_utilization_initial(void **cm_state)
{
    (void)cm_state;
    uint32_t util = scheduler_get_cpu_utilization();
    assert_int_equal(0, util);
}

/*******************************************************************************
 * Test Cases - Period Management
 ******************************************************************************/

static void test_set_period_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_set_period(handle, 200);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(200, scheduler_get_period(handle));
}

static void test_set_period_invalid(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_set_period(handle, 1); /* Too small */
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_get_period_invalid_handle(void **cm_state)
{
    (void)cm_state;
    uint32_t period = scheduler_get_period(SCHED_INVALID_HANDLE);
    assert_int_equal(0, period);
}

/*******************************************************************************
 * Test Cases - Deadline Callback
 ******************************************************************************/

static void test_register_deadline_callback(void **cm_state)
{
    (void)cm_state;
    sched_status_t status = scheduler_register_deadline_callback(deadline_miss_callback);
    assert_int_equal(SCHED_OK, status);
}

/*******************************************************************************
 * Test Cases - Task Execution
 ******************************************************************************/

static void test_run_now_success(void **cm_state)
{
    (void)cm_state;
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_run_now(handle);
    assert_int_equal(SCHED_OK, status);
    assert_int_equal(1, g_task1_count);
}

static void test_run_now_invalid_handle(void **cm_state)
{
    (void)cm_state;
    sched_status_t status = scheduler_run_now(SCHED_INVALID_HANDLE);
    assert_int_equal(SCHED_ERROR_INVALID_PARAM, status);
}

static void test_scheduler_tick_increments_count(void **cm_state)
{
    (void)cm_state;
    uint32_t initial = scheduler_get_tick_count();
    scheduler_tick();
    assert_int_equal(initial + 1, scheduler_get_tick_count());
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

static int fixture_setup(void **state)
{
    (void)state;
    setUp();
    return 0;
}

static int fixture_teardown(void **state)
{
    (void)state;
    tearDown();
    return 0;
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup_teardown(test_scheduler_init_returns_ok, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_scheduler_not_running_initially, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_scheduler_tick_count_zero, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_null_config, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_null_handle, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_null_func, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_period_too_small, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_period_too_large, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_task_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_duplicate_name, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_max_tasks, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_invalid_handle, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_not_registered, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_unregister_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_enable_task_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_disable_task_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_suspend_task_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_resume_task_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_handle_by_name_found, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_handle_by_name_not_found, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_handle_by_name_null, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_name_valid, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_name_invalid, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_stats_null, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_task_stats_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_reset_task_stats, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_scheduler_stats_null, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_scheduler_stats_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_cpu_utilization_initial, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_set_period_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_set_period_invalid, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_get_period_invalid_handle, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_register_deadline_callback, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_run_now_success, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_run_now_invalid_handle, fixture_setup, fixture_teardown),
        cmocka_unit_test_setup_teardown(test_scheduler_tick_increments_count, fixture_setup, fixture_teardown),
    };

    return cmocka_run_group_tests_name("Scheduler Tests", tests, NULL, NULL);
}
