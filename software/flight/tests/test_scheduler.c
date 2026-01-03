/**
 * @file test_scheduler.c
 * @brief Unit tests for time-triggered scheduler module
 *
 * Tests task registration, scheduling, priority handling,
 * deadline monitoring, and CPU utilization measurement.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "scheduler.h"
#include <string.h>

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

void setUp(void)
{
    scheduler_init();
    g_task1_count = 0;
    g_task2_count = 0;
    g_deadline_miss_count = 0;
}

void tearDown(void)
{
    scheduler_stop();
}

/*******************************************************************************
 * Test Cases - Initialization
 ******************************************************************************/

void test_scheduler_init_returns_ok(void)
{
    sched_status_t status = scheduler_init();
    TEST_ASSERT_EQUAL(SCHED_OK, status);
}

void test_scheduler_not_running_initially(void)
{
    TEST_ASSERT_FALSE(scheduler_is_running());
}

void test_scheduler_tick_count_zero(void)
{
    TEST_ASSERT_EQUAL(0, scheduler_get_tick_count());
}

/*******************************************************************************
 * Test Cases - Task Registration
 ******************************************************************************/

void test_register_task_null_config(void)
{
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(NULL, &handle);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_register_task_null_handle(void)
{
    task_config_t config = {
        .name = "Test",
        .func = test_task1_func,
        .period_ms = 100,
        .priority = SCHED_PRIORITY_NORMAL
    };
    sched_status_t status = scheduler_register_task(&config, NULL);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_register_task_null_func(void)
{
    task_config_t config = {
        .name = "Test",
        .func = NULL,
        .period_ms = 100
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_register_task_period_too_small(void)
{
    task_config_t config = {
        .name = "Test",
        .func = test_task1_func,
        .period_ms = 1  /* Below minimum */
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_register_task_period_too_large(void)
{
    task_config_t config = {
        .name = "Test",
        .func = test_task1_func,
        .period_ms = 100000  /* Above maximum */
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_register_task_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .priority = SCHED_PRIORITY_NORMAL,
        .enabled = true
    };
    task_handle_t handle;
    sched_status_t status = scheduler_register_task(&config, &handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_NOT_EQUAL(SCHED_INVALID_HANDLE, handle);
}

void test_register_duplicate_name(void)
{
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
    TEST_ASSERT_EQUAL(SCHED_ERROR_ALREADY_EXISTS, status);
}

void test_register_max_tasks(void)
{
    task_handle_t handles[SCHED_MAX_TASKS + 1];
    sched_status_t status;

    for (uint8_t i = 0; i < SCHED_MAX_TASKS; i++) {
        task_config_t config = {
            .func = test_task1_func,
            .period_ms = 100
        };
        snprintf(config.name, SCHED_MAX_TASK_NAME, "Task%d", i);
        status = scheduler_register_task(&config, &handles[i]);
        TEST_ASSERT_EQUAL(SCHED_OK, status);
    }

    /* One more should fail */
    task_config_t extra = {
        .name = "Extra",
        .func = test_task1_func,
        .period_ms = 100
    };
    status = scheduler_register_task(&extra, &handles[SCHED_MAX_TASKS]);
    TEST_ASSERT_EQUAL(SCHED_ERROR_TABLE_FULL, status);
}

/*******************************************************************************
 * Test Cases - Task Unregistration
 ******************************************************************************/

void test_unregister_invalid_handle(void)
{
    sched_status_t status = scheduler_unregister_task(SCHED_MAX_TASKS + 1);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_unregister_not_registered(void)
{
    sched_status_t status = scheduler_unregister_task(0);
    TEST_ASSERT_EQUAL(SCHED_ERROR_NOT_FOUND, status);
}

void test_unregister_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_unregister_task(handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
}

/*******************************************************************************
 * Test Cases - Enable/Disable
 ******************************************************************************/

void test_enable_task_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = false
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_enable_task(handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(TASK_STATE_READY, scheduler_get_task_state(handle));
}

void test_disable_task_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_disable_task(handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(TASK_STATE_INACTIVE, scheduler_get_task_state(handle));
}

/*******************************************************************************
 * Test Cases - Suspend/Resume
 ******************************************************************************/

void test_suspend_task_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_suspend_task(handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(TASK_STATE_SUSPENDED, scheduler_get_task_state(handle));
}

void test_resume_task_success(void)
{
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
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(TASK_STATE_READY, scheduler_get_task_state(handle));
}

/*******************************************************************************
 * Test Cases - Task Lookup
 ******************************************************************************/

void test_get_handle_by_name_found(void)
{
    task_config_t config = {
        .name = "MyTask",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t registered;
    scheduler_register_task(&config, &registered);

    task_handle_t found = scheduler_get_handle_by_name("MyTask");
    TEST_ASSERT_EQUAL(registered, found);
}

void test_get_handle_by_name_not_found(void)
{
    task_handle_t found = scheduler_get_handle_by_name("NonExistent");
    TEST_ASSERT_EQUAL(SCHED_INVALID_HANDLE, found);
}

void test_get_handle_by_name_null(void)
{
    task_handle_t found = scheduler_get_handle_by_name(NULL);
    TEST_ASSERT_EQUAL(SCHED_INVALID_HANDLE, found);
}

void test_get_task_name_valid(void)
{
    task_config_t config = {
        .name = "MyTask",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    const char *name = scheduler_get_task_name(handle);
    TEST_ASSERT_NOT_NULL(name);
    TEST_ASSERT_EQUAL_STRING("MyTask", name);
}

void test_get_task_name_invalid(void)
{
    const char *name = scheduler_get_task_name(SCHED_INVALID_HANDLE);
    TEST_ASSERT_NULL(name);
}

/*******************************************************************************
 * Test Cases - Statistics
 ******************************************************************************/

void test_get_task_stats_null(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);
    sched_status_t status = scheduler_get_task_stats(handle, NULL);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_get_task_stats_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    task_stats_t stats;
    sched_status_t status = scheduler_get_task_stats(handle, &stats);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(0, stats.run_count);
}

void test_reset_task_stats(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_reset_task_stats(handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
}

void test_get_scheduler_stats_null(void)
{
    sched_status_t status = scheduler_get_stats(NULL);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_get_scheduler_stats_success(void)
{
    sched_stats_t stats;
    sched_status_t status = scheduler_get_stats(&stats);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
}

void test_cpu_utilization_initial(void)
{
    uint32_t util = scheduler_get_cpu_utilization();
    TEST_ASSERT_EQUAL(0, util);
}

/*******************************************************************************
 * Test Cases - Period Management
 ******************************************************************************/

void test_set_period_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_set_period(handle, 200);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(200, scheduler_get_period(handle));
}

void test_set_period_invalid(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_set_period(handle, 1); /* Too small */
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_get_period_invalid_handle(void)
{
    uint32_t period = scheduler_get_period(SCHED_INVALID_HANDLE);
    TEST_ASSERT_EQUAL(0, period);
}

/*******************************************************************************
 * Test Cases - Deadline Callback
 ******************************************************************************/

void test_register_deadline_callback(void)
{
    sched_status_t status = scheduler_register_deadline_callback(deadline_miss_callback);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
}

/*******************************************************************************
 * Test Cases - Task Execution
 ******************************************************************************/

void test_run_now_success(void)
{
    task_config_t config = {
        .name = "Task1",
        .func = test_task1_func,
        .period_ms = 100,
        .enabled = true
    };
    task_handle_t handle;
    scheduler_register_task(&config, &handle);

    sched_status_t status = scheduler_run_now(handle);
    TEST_ASSERT_EQUAL(SCHED_OK, status);
    TEST_ASSERT_EQUAL(1, g_task1_count);
}

void test_run_now_invalid_handle(void)
{
    sched_status_t status = scheduler_run_now(SCHED_INVALID_HANDLE);
    TEST_ASSERT_EQUAL(SCHED_ERROR_INVALID_PARAM, status);
}

void test_scheduler_tick_increments_count(void)
{
    uint32_t initial = scheduler_get_tick_count();
    scheduler_tick();
    TEST_ASSERT_EQUAL(initial + 1, scheduler_get_tick_count());
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Initialization */
    RUN_TEST(test_scheduler_init_returns_ok);
    RUN_TEST(test_scheduler_not_running_initially);
    RUN_TEST(test_scheduler_tick_count_zero);

    /* Task registration */
    RUN_TEST(test_register_task_null_config);
    RUN_TEST(test_register_task_null_handle);
    RUN_TEST(test_register_task_null_func);
    RUN_TEST(test_register_task_period_too_small);
    RUN_TEST(test_register_task_period_too_large);
    RUN_TEST(test_register_task_success);
    RUN_TEST(test_register_duplicate_name);
    RUN_TEST(test_register_max_tasks);

    /* Task unregistration */
    RUN_TEST(test_unregister_invalid_handle);
    RUN_TEST(test_unregister_not_registered);
    RUN_TEST(test_unregister_success);

    /* Enable/Disable */
    RUN_TEST(test_enable_task_success);
    RUN_TEST(test_disable_task_success);

    /* Suspend/Resume */
    RUN_TEST(test_suspend_task_success);
    RUN_TEST(test_resume_task_success);

    /* Task lookup */
    RUN_TEST(test_get_handle_by_name_found);
    RUN_TEST(test_get_handle_by_name_not_found);
    RUN_TEST(test_get_handle_by_name_null);
    RUN_TEST(test_get_task_name_valid);
    RUN_TEST(test_get_task_name_invalid);

    /* Statistics */
    RUN_TEST(test_get_task_stats_null);
    RUN_TEST(test_get_task_stats_success);
    RUN_TEST(test_reset_task_stats);
    RUN_TEST(test_get_scheduler_stats_null);
    RUN_TEST(test_get_scheduler_stats_success);
    RUN_TEST(test_cpu_utilization_initial);

    /* Period management */
    RUN_TEST(test_set_period_success);
    RUN_TEST(test_set_period_invalid);
    RUN_TEST(test_get_period_invalid_handle);

    /* Deadline callback */
    RUN_TEST(test_register_deadline_callback);

    /* Task execution */
    RUN_TEST(test_run_now_success);
    RUN_TEST(test_run_now_invalid_handle);
    RUN_TEST(test_scheduler_tick_increments_count);

    return UNITY_END();
}
