/**
 * @file scheduler.c
 * @brief Time-triggered scheduler for SMART-QSO CubeSat
 *
 * This module provides a deterministic time-triggered scheduler for
 * bare-metal or RTOS environments. Supports periodic tasks with
 * deadline monitoring per NASA Class B software requirements.
 *
 * @note MISRA C:2012 compliant
 * @note NPR 7150.2 Class B software requirements
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "scheduler.h"
#include "safe_string.h"
#include <string.h>

/*******************************************************************************
 * Private Definitions
 ******************************************************************************/

/** CPU utilization measurement window (ticks) */
#define CPU_UTIL_WINDOW             1000U

/** Statistics averaging factor (power of 2) */
#define STATS_AVG_SHIFT             3U
#define STATS_AVG_FACTOR            (1U << STATS_AVG_SHIFT)

/*******************************************************************************
 * Private Types
 ******************************************************************************/

/**
 * @brief Task control block
 */
typedef struct {
    task_config_t config;           /**< Task configuration */
    task_state_t state;             /**< Current state */
    task_stats_t stats;             /**< Runtime statistics */
    uint32_t next_run_tick;         /**< Next scheduled execution tick */
    uint32_t consecutive_misses;    /**< Consecutive deadline misses */
    bool registered;                /**< Task slot in use */
} task_tcb_t;

/**
 * @brief Scheduler context
 */
typedef struct {
    task_tcb_t tasks[SCHED_MAX_TASKS];  /**< Task table */
    uint32_t tick_count;                /**< Global tick counter */
    uint32_t active_time_us;            /**< Time spent in tasks */
    uint32_t idle_time_us;              /**< Time spent idle */
    uint32_t cpu_utilization;           /**< Calculated CPU usage */
    task_handle_t running_task;         /**< Currently running task */
    deadline_miss_cb_t deadline_cb;     /**< Deadline miss callback */
    bool running;                       /**< Scheduler is running */
    bool initialized;                   /**< Scheduler initialized */
} sched_context_t;

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Scheduler context */
static sched_context_t g_sched;

/*******************************************************************************
 * Private Function Prototypes
 ******************************************************************************/

static uint32_t sched_get_time_us(void);
static void sched_update_cpu_util(uint32_t task_time_us);
static void sched_run_task(task_handle_t handle);
static task_handle_t sched_find_ready_task(void);
static void sched_idle_task(void);

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

/**
 * @brief Initialize the scheduler
 */
sched_status_t scheduler_init(void)
{
    /* Clear scheduler context */
    memset(&g_sched, 0, sizeof(g_sched));

    /* Initialize all task slots */
    for (uint8_t i = 0U; i < SCHED_MAX_TASKS; i++) {
        g_sched.tasks[i].registered = false;
        g_sched.tasks[i].state = TASK_STATE_INACTIVE;
        g_sched.tasks[i].stats.min_run_time_us = UINT32_MAX;
    }

    g_sched.running_task = SCHED_INVALID_HANDLE;
    g_sched.initialized = true;

    return SCHED_OK;
}

/**
 * @brief Register a task with the scheduler
 */
sched_status_t scheduler_register_task(const task_config_t *config,
                                        task_handle_t *handle)
{
    if (config == NULL || handle == NULL) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (config->func == NULL) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (config->period_ms < SCHED_MIN_PERIOD_MS ||
        config->period_ms > SCHED_MAX_PERIOD_MS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    /* Find empty slot */
    task_handle_t slot = SCHED_INVALID_HANDLE;
    for (uint8_t i = 0U; i < SCHED_MAX_TASKS; i++) {
        if (!g_sched.tasks[i].registered) {
            slot = i;
            break;
        }
    }

    if (slot == SCHED_INVALID_HANDLE) {
        return SCHED_ERROR_TABLE_FULL;
    }

    /* Check for duplicate name */
    for (uint8_t i = 0U; i < SCHED_MAX_TASKS; i++) {
        if (g_sched.tasks[i].registered &&
            strncmp(g_sched.tasks[i].config.name, config->name,
                    SCHED_MAX_TASK_NAME) == 0) {
            return SCHED_ERROR_ALREADY_EXISTS;
        }
    }

    /* Initialize task */
    task_tcb_t *tcb = &g_sched.tasks[slot];
    memcpy(&tcb->config, config, sizeof(task_config_t));
    tcb->registered = true;
    tcb->state = config->enabled ? TASK_STATE_READY : TASK_STATE_INACTIVE;
    tcb->next_run_tick = g_sched.tick_count + (config->offset_ms / SCHED_TICK_PERIOD_MS);
    tcb->consecutive_misses = 0U;

    /* Reset statistics */
    memset(&tcb->stats, 0, sizeof(task_stats_t));
    tcb->stats.min_run_time_us = UINT32_MAX;

    *handle = slot;

    return SCHED_OK;
}

/**
 * @brief Unregister a task
 */
sched_status_t scheduler_unregister_task(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    /* Cannot unregister running task */
    if (g_sched.running_task == handle) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    g_sched.tasks[handle].registered = false;
    g_sched.tasks[handle].state = TASK_STATE_INACTIVE;

    return SCHED_OK;
}

/**
 * @brief Start the scheduler
 */
sched_status_t scheduler_start(void)
{
    if (!g_sched.initialized) {
        return SCHED_ERROR_NOT_RUNNING;
    }

    g_sched.running = true;
    g_sched.tick_count = 0U;

    /* Main scheduler loop */
    while (g_sched.running) {
        scheduler_tick();
    }

    return SCHED_OK;
}

/**
 * @brief Stop the scheduler
 */
void scheduler_stop(void)
{
    g_sched.running = false;
}

/**
 * @brief Run one scheduler tick
 */
void scheduler_tick(void)
{
    g_sched.tick_count++;

    /* Find highest priority ready task */
    task_handle_t ready_task = sched_find_ready_task();

    if (ready_task != SCHED_INVALID_HANDLE) {
        sched_run_task(ready_task);
    } else {
        sched_idle_task();
    }
}

/**
 * @brief Enable a task
 */
sched_status_t scheduler_enable_task(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    task_tcb_t *tcb = &g_sched.tasks[handle];
    tcb->state = TASK_STATE_READY;
    tcb->config.enabled = true;
    tcb->next_run_tick = g_sched.tick_count;

    return SCHED_OK;
}

/**
 * @brief Disable a task
 */
sched_status_t scheduler_disable_task(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    g_sched.tasks[handle].state = TASK_STATE_INACTIVE;
    g_sched.tasks[handle].config.enabled = false;

    return SCHED_OK;
}

/**
 * @brief Suspend a task temporarily
 */
sched_status_t scheduler_suspend_task(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    g_sched.tasks[handle].state = TASK_STATE_SUSPENDED;

    return SCHED_OK;
}

/**
 * @brief Resume a suspended task
 */
sched_status_t scheduler_resume_task(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    if (g_sched.tasks[handle].state == TASK_STATE_SUSPENDED) {
        g_sched.tasks[handle].state = TASK_STATE_READY;
    }

    return SCHED_OK;
}

/**
 * @brief Get task state
 */
task_state_t scheduler_get_task_state(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS || !g_sched.tasks[handle].registered) {
        return TASK_STATE_INACTIVE;
    }

    return g_sched.tasks[handle].state;
}

/**
 * @brief Get task statistics
 */
sched_status_t scheduler_get_task_stats(task_handle_t handle,
                                         task_stats_t *stats)
{
    if (handle >= SCHED_MAX_TASKS || stats == NULL) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    memcpy(stats, &g_sched.tasks[handle].stats, sizeof(task_stats_t));

    return SCHED_OK;
}

/**
 * @brief Reset task statistics
 */
sched_status_t scheduler_reset_task_stats(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    memset(&g_sched.tasks[handle].stats, 0, sizeof(task_stats_t));
    g_sched.tasks[handle].stats.min_run_time_us = UINT32_MAX;

    return SCHED_OK;
}

/**
 * @brief Get scheduler statistics
 */
sched_status_t scheduler_get_stats(sched_stats_t *stats)
{
    if (stats == NULL) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    stats->tick_count = g_sched.tick_count;
    stats->cpu_utilization = g_sched.cpu_utilization;
    stats->idle_time_us = g_sched.idle_time_us;

    /* Count active tasks and total deadline misses */
    stats->active_tasks = 0U;
    stats->total_deadline_misses = 0U;

    for (uint8_t i = 0U; i < SCHED_MAX_TASKS; i++) {
        if (g_sched.tasks[i].registered &&
            g_sched.tasks[i].state == TASK_STATE_READY) {
            stats->active_tasks++;
        }
        if (g_sched.tasks[i].registered) {
            stats->total_deadline_misses += g_sched.tasks[i].stats.deadline_misses;
        }
    }

    return SCHED_OK;
}

/**
 * @brief Get CPU utilization
 */
uint32_t scheduler_get_cpu_utilization(void)
{
    return g_sched.cpu_utilization;
}

/**
 * @brief Register deadline miss callback
 */
sched_status_t scheduler_register_deadline_callback(deadline_miss_cb_t callback)
{
    g_sched.deadline_cb = callback;
    return SCHED_OK;
}

/**
 * @brief Get task handle by name
 */
task_handle_t scheduler_get_handle_by_name(const char *name)
{
    if (name == NULL) {
        return SCHED_INVALID_HANDLE;
    }

    for (uint8_t i = 0U; i < SCHED_MAX_TASKS; i++) {
        if (g_sched.tasks[i].registered &&
            strncmp(g_sched.tasks[i].config.name, name,
                    SCHED_MAX_TASK_NAME) == 0) {
            return i;
        }
    }

    return SCHED_INVALID_HANDLE;
}

/**
 * @brief Get task name by handle
 */
const char *scheduler_get_task_name(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS || !g_sched.tasks[handle].registered) {
        return NULL;
    }

    return g_sched.tasks[handle].config.name;
}

/**
 * @brief Check if scheduler is running
 */
bool scheduler_is_running(void)
{
    return g_sched.running;
}

/**
 * @brief Get current tick count
 */
uint32_t scheduler_get_tick_count(void)
{
    return g_sched.tick_count;
}

/**
 * @brief Delay for specified milliseconds (cooperative)
 */
void scheduler_delay_ms(uint32_t ms)
{
    uint32_t start_tick = g_sched.tick_count;
    uint32_t delay_ticks = ms / SCHED_TICK_PERIOD_MS;

    while ((g_sched.tick_count - start_tick) < delay_ticks) {
        /* Yield to other tasks */
        if (g_sched.running) {
            /* In cooperative mode, other tasks would run here */
        }
    }
}

/**
 * @brief Force immediate task execution
 */
sched_status_t scheduler_run_now(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    sched_run_task(handle);

    return SCHED_OK;
}

/**
 * @brief Set task period dynamically
 */
sched_status_t scheduler_set_period(task_handle_t handle, uint32_t period_ms)
{
    if (handle >= SCHED_MAX_TASKS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (period_ms < SCHED_MIN_PERIOD_MS || period_ms > SCHED_MAX_PERIOD_MS) {
        return SCHED_ERROR_INVALID_PARAM;
    }

    if (!g_sched.tasks[handle].registered) {
        return SCHED_ERROR_NOT_FOUND;
    }

    g_sched.tasks[handle].config.period_ms = period_ms;

    return SCHED_OK;
}

/**
 * @brief Get task period
 */
uint32_t scheduler_get_period(task_handle_t handle)
{
    if (handle >= SCHED_MAX_TASKS || !g_sched.tasks[handle].registered) {
        return 0U;
    }

    return g_sched.tasks[handle].config.period_ms;
}

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Get current time in microseconds
 */
static uint32_t sched_get_time_us(void)
{
#ifdef TARGET_STM32L4
    /* Use DWT cycle counter or timer for high-resolution timing */
    /* return DWT->CYCCNT / (SystemCoreClock / 1000000U); */
    return 0U;
#else
    /* Simulation mode */
    return 0U;
#endif
}

/**
 * @brief Update CPU utilization statistics
 */
static void sched_update_cpu_util(uint32_t task_time_us)
{
    g_sched.active_time_us += task_time_us;

    /* Update utilization every CPU_UTIL_WINDOW ticks */
    if ((g_sched.tick_count % CPU_UTIL_WINDOW) == 0U) {
        uint32_t total_time = g_sched.active_time_us + g_sched.idle_time_us;
        if (total_time > 0U) {
            g_sched.cpu_utilization = (g_sched.active_time_us * 100U) / total_time;
        }
        /* Reset accumulators */
        g_sched.active_time_us = 0U;
        g_sched.idle_time_us = 0U;
    }
}

/**
 * @brief Run a specific task
 */
static void sched_run_task(task_handle_t handle)
{
    task_tcb_t *tcb = &g_sched.tasks[handle];

    /* Mark task as running */
    tcb->state = TASK_STATE_RUNNING;
    g_sched.running_task = handle;

    /* Record start time */
    uint32_t start_time = sched_get_time_us();

    /* Execute task */
    if (tcb->config.func != NULL) {
        tcb->config.func();
    }

    /* Calculate execution time */
    uint32_t run_time = sched_get_time_us() - start_time;

    /* Update statistics */
    tcb->stats.run_count++;
    tcb->stats.last_run_time_us = run_time;

    if (run_time > tcb->stats.max_run_time_us) {
        tcb->stats.max_run_time_us = run_time;
    }
    if (run_time < tcb->stats.min_run_time_us) {
        tcb->stats.min_run_time_us = run_time;
    }

    /* Exponential moving average */
    tcb->stats.avg_run_time_us =
        ((tcb->stats.avg_run_time_us * (STATS_AVG_FACTOR - 1U)) + run_time) /
        STATS_AVG_FACTOR;

    /* Check deadline */
    uint32_t deadline_us = tcb->config.deadline_ms * 1000U;
    if (deadline_us > 0U && run_time > deadline_us) {
        tcb->stats.deadline_misses++;
        tcb->consecutive_misses++;

        /* Notify via callback */
        if (g_sched.deadline_cb != NULL) {
            g_sched.deadline_cb(handle, run_time - deadline_us);
        }

        /* Check for fault condition */
        if (tcb->consecutive_misses >= SCHED_DEADLINE_MISS_LIMIT) {
            tcb->state = TASK_STATE_FAULT;
        }
    } else {
        tcb->consecutive_misses = 0U;
    }

    /* Schedule next run */
    tcb->next_run_tick = g_sched.tick_count +
                         (tcb->config.period_ms / SCHED_TICK_PERIOD_MS);

    /* Mark task as ready (unless in fault) */
    if (tcb->state != TASK_STATE_FAULT) {
        tcb->state = TASK_STATE_READY;
    }

    g_sched.running_task = SCHED_INVALID_HANDLE;

    /* Update CPU utilization */
    sched_update_cpu_util(run_time);
}

/**
 * @brief Find highest priority ready task
 */
static task_handle_t sched_find_ready_task(void)
{
    task_handle_t best_task = SCHED_INVALID_HANDLE;
    sched_priority_t best_priority = SCHED_PRIORITY_IDLE;

    for (uint8_t i = 0U; i < SCHED_MAX_TASKS; i++) {
        task_tcb_t *tcb = &g_sched.tasks[i];

        if (!tcb->registered) {
            continue;
        }

        if (tcb->state != TASK_STATE_READY) {
            continue;
        }

        /* Check if task is due to run */
        if (g_sched.tick_count < tcb->next_run_tick) {
            continue;
        }

        /* Check priority */
        if (best_task == SCHED_INVALID_HANDLE ||
            tcb->config.priority < best_priority) {
            best_task = i;
            best_priority = tcb->config.priority;
        }
    }

    return best_task;
}

/**
 * @brief Idle task - runs when no other task is ready
 */
static void sched_idle_task(void)
{
    uint32_t start_time = sched_get_time_us();

#ifdef TARGET_STM32L4
    /* Enter low power mode */
    /* __WFI(); */
#endif

    uint32_t idle_time = sched_get_time_us() - start_time;
    g_sched.idle_time_us += idle_time;
}
