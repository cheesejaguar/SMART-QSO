/**
 * @file scheduler.h
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

#ifndef SCHEDULER_H
#define SCHEDULER_H

#ifdef __cplusplus
extern "C" {
#endif

/*******************************************************************************
 * Includes
 ******************************************************************************/

#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Definitions
 ******************************************************************************/

/** Maximum number of tasks */
#define SCHED_MAX_TASKS             16U

/** Maximum task name length */
#define SCHED_MAX_TASK_NAME         16U

/** Scheduler tick period in milliseconds */
#define SCHED_TICK_PERIOD_MS        1U

/** Minimum task period in milliseconds */
#define SCHED_MIN_PERIOD_MS         10U

/** Maximum task period in milliseconds */
#define SCHED_MAX_PERIOD_MS         60000U

/** CPU utilization warning threshold (percent) */
#define SCHED_CPU_WARN_THRESHOLD    80U

/** Task deadline miss threshold before fault */
#define SCHED_DEADLINE_MISS_LIMIT   3U

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Task priority levels
 *
 * Lower numeric value = higher priority
 */
typedef enum {
    SCHED_PRIORITY_CRITICAL = 0,    /**< Safety-critical tasks (watchdog, faults) */
    SCHED_PRIORITY_HIGH,            /**< Time-sensitive tasks (EPS, ADCS) */
    SCHED_PRIORITY_NORMAL,          /**< Normal operations (telemetry, comms) */
    SCHED_PRIORITY_LOW,             /**< Background tasks (housekeeping) */
    SCHED_PRIORITY_IDLE             /**< Idle task only */
} sched_priority_t;

/**
 * @brief Task state enumeration
 */
typedef enum {
    TASK_STATE_INACTIVE = 0,        /**< Task not scheduled */
    TASK_STATE_READY,               /**< Ready to run */
    TASK_STATE_RUNNING,             /**< Currently executing */
    TASK_STATE_SUSPENDED,           /**< Temporarily suspended */
    TASK_STATE_FAULT                /**< Task in fault state */
} task_state_t;

/**
 * @brief Scheduler status codes
 */
typedef enum {
    SCHED_OK = 0,                   /**< Success */
    SCHED_ERROR_INVALID_PARAM,      /**< Invalid parameter */
    SCHED_ERROR_TABLE_FULL,         /**< Task table full */
    SCHED_ERROR_NOT_FOUND,          /**< Task not found */
    SCHED_ERROR_ALREADY_EXISTS,     /**< Task already registered */
    SCHED_ERROR_NOT_RUNNING,        /**< Scheduler not running */
    SCHED_ERROR_TIMEOUT             /**< Operation timed out */
} sched_status_t;

/**
 * @brief Task function pointer type
 */
typedef void (*task_func_t)(void);

/**
 * @brief Task configuration structure
 */
typedef struct {
    char name[SCHED_MAX_TASK_NAME]; /**< Task name for debugging */
    task_func_t func;               /**< Task function pointer */
    uint32_t period_ms;             /**< Execution period in ms */
    uint32_t offset_ms;             /**< Phase offset from start */
    uint32_t deadline_ms;           /**< Maximum execution time */
    sched_priority_t priority;      /**< Task priority level */
    bool enabled;                   /**< Task enabled at start */
} task_config_t;

/**
 * @brief Task runtime statistics
 */
typedef struct {
    uint32_t run_count;             /**< Number of executions */
    uint32_t last_run_time_us;      /**< Last execution time (microseconds) */
    uint32_t max_run_time_us;       /**< Maximum execution time */
    uint32_t min_run_time_us;       /**< Minimum execution time */
    uint32_t avg_run_time_us;       /**< Average execution time */
    uint32_t deadline_misses;       /**< Number of deadline misses */
    uint32_t skip_count;            /**< Number of skipped executions */
} task_stats_t;

/**
 * @brief Task handle type
 */
typedef uint8_t task_handle_t;

/** Invalid task handle */
#define SCHED_INVALID_HANDLE        0xFFU

/**
 * @brief Scheduler statistics
 */
typedef struct {
    uint32_t tick_count;            /**< Total tick count */
    uint32_t cpu_utilization;       /**< CPU usage (0-100%) */
    uint32_t idle_time_us;          /**< Time spent in idle */
    uint32_t active_tasks;          /**< Number of active tasks */
    uint32_t total_deadline_misses; /**< Total deadline misses */
} sched_stats_t;

/**
 * @brief Deadline miss callback type
 *
 * @param handle Task handle that missed deadline
 * @param overrun_us Overrun time in microseconds
 */
typedef void (*deadline_miss_cb_t)(task_handle_t handle, uint32_t overrun_us);

/*******************************************************************************
 * Public Function Prototypes
 ******************************************************************************/

/**
 * @brief Initialize the scheduler
 *
 * Clears task table and prepares scheduler for operation.
 *
 * @return SCHED_OK on success
 */
sched_status_t scheduler_init(void);

/**
 * @brief Register a task with the scheduler
 *
 * @param config Task configuration
 * @param handle Output: task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_register_task(const task_config_t *config,
                                        task_handle_t *handle);

/**
 * @brief Unregister a task
 *
 * @param handle Task handle to unregister
 * @return SCHED_OK on success
 */
sched_status_t scheduler_unregister_task(task_handle_t handle);

/**
 * @brief Start the scheduler
 *
 * Begins task scheduling. Does not return unless stopped.
 *
 * @return SCHED_OK if stopped cleanly
 */
sched_status_t scheduler_start(void);

/**
 * @brief Stop the scheduler
 *
 * Signals scheduler to stop at next opportunity.
 */
void scheduler_stop(void);

/**
 * @brief Run one scheduler tick
 *
 * For cooperative scheduling, call this periodically from main loop.
 */
void scheduler_tick(void);

/**
 * @brief Enable a task
 *
 * @param handle Task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_enable_task(task_handle_t handle);

/**
 * @brief Disable a task
 *
 * @param handle Task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_disable_task(task_handle_t handle);

/**
 * @brief Suspend a task temporarily
 *
 * @param handle Task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_suspend_task(task_handle_t handle);

/**
 * @brief Resume a suspended task
 *
 * @param handle Task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_resume_task(task_handle_t handle);

/**
 * @brief Get task state
 *
 * @param handle Task handle
 * @return Task state
 */
task_state_t scheduler_get_task_state(task_handle_t handle);

/**
 * @brief Get task statistics
 *
 * @param handle Task handle
 * @param stats Output: task statistics
 * @return SCHED_OK on success
 */
sched_status_t scheduler_get_task_stats(task_handle_t handle,
                                         task_stats_t *stats);

/**
 * @brief Reset task statistics
 *
 * @param handle Task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_reset_task_stats(task_handle_t handle);

/**
 * @brief Get scheduler statistics
 *
 * @param stats Output: scheduler statistics
 * @return SCHED_OK on success
 */
sched_status_t scheduler_get_stats(sched_stats_t *stats);

/**
 * @brief Get CPU utilization
 *
 * @return CPU utilization percentage (0-100)
 */
uint32_t scheduler_get_cpu_utilization(void);

/**
 * @brief Register deadline miss callback
 *
 * @param callback Function to call on deadline miss
 * @return SCHED_OK on success
 */
sched_status_t scheduler_register_deadline_callback(deadline_miss_cb_t callback);

/**
 * @brief Get task handle by name
 *
 * @param name Task name
 * @return Task handle or SCHED_INVALID_HANDLE if not found
 */
task_handle_t scheduler_get_handle_by_name(const char *name);

/**
 * @brief Get task name by handle
 *
 * @param handle Task handle
 * @return Task name or NULL if invalid
 */
const char *scheduler_get_task_name(task_handle_t handle);

/**
 * @brief Check if scheduler is running
 *
 * @return true if scheduler is running
 */
bool scheduler_is_running(void);

/**
 * @brief Get current tick count
 *
 * @return Scheduler tick count
 */
uint32_t scheduler_get_tick_count(void);

/**
 * @brief Delay for specified milliseconds (cooperative)
 *
 * Yields to other tasks during delay period.
 *
 * @param ms Milliseconds to delay
 */
void scheduler_delay_ms(uint32_t ms);

/**
 * @brief Force immediate task execution
 *
 * Executes task immediately regardless of schedule.
 *
 * @param handle Task handle
 * @return SCHED_OK on success
 */
sched_status_t scheduler_run_now(task_handle_t handle);

/**
 * @brief Set task period dynamically
 *
 * @param handle Task handle
 * @param period_ms New period in milliseconds
 * @return SCHED_OK on success
 */
sched_status_t scheduler_set_period(task_handle_t handle, uint32_t period_ms);

/**
 * @brief Get task period
 *
 * @param handle Task handle
 * @return Period in milliseconds, 0 if invalid handle
 */
uint32_t scheduler_get_period(task_handle_t handle);

#ifdef __cplusplus
}
#endif

#endif /* SCHEDULER_H */
