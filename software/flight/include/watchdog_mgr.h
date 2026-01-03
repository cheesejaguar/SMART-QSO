/**
 * @file watchdog_mgr.h
 * @brief Watchdog Manager for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module provides application-level watchdog management including:
 * - Hardware watchdog initialization and kicking
 * - Task watchdog tracking for multi-task systems
 * - Pre-reset state saving
 * - Watchdog event logging and statistics
 *
 * @requirement SRS-F050 Kick hardware watchdog at least every 30 seconds
 * @requirement SRS-F051 Save state before watchdog reset
 * @requirement SRS-S003 Software shall implement hardware watchdog
 * @requirement NPR-7150.2 Watchdog timer requirement
 *
 * Heritage: NASA Ames SmallSat Division (GeneSat, PharmaSat, O/OREOS)
 */

#ifndef SMART_QSO_WATCHDOG_MGR_H
#define SMART_QSO_WATCHDOG_MGR_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** Maximum number of monitored tasks */
#define WDT_MAX_TASKS               8U

/** Maximum task name length */
#define WDT_TASK_NAME_LEN           16U

/** Default hardware watchdog timeout (ms) */
#define WDT_HW_TIMEOUT_MS           30000U

/** Task watchdog check interval (ms) */
#define WDT_TASK_CHECK_INTERVAL_MS  1000U

/** Number of missed check-ins before task is considered stalled */
#define WDT_TASK_MISS_THRESHOLD     3U

/*******************************************************************************
 * Watchdog Task IDs
 ******************************************************************************/

/**
 * @brief Predefined task IDs for watchdog registration
 */
typedef enum {
    WDT_TASK_MAIN_LOOP      = 0,  /**< Main processing loop */
    WDT_TASK_EPS            = 1,  /**< EPS monitoring task */
    WDT_TASK_ADCS           = 2,  /**< ADCS control task */
    WDT_TASK_COMM           = 3,  /**< Communication task */
    WDT_TASK_TELEMETRY      = 4,  /**< Telemetry generation task */
    WDT_TASK_BEACON         = 5,  /**< Beacon transmission task */
    WDT_TASK_PAYLOAD        = 6,  /**< Payload management task */
    WDT_TASK_HEALTH         = 7,  /**< Health monitoring task */
    WDT_TASK_COUNT          = 8
} WdtTaskId_t;

/*******************************************************************************
 * Watchdog Types
 ******************************************************************************/

/**
 * @brief Task watchdog entry
 */
typedef struct {
    char name[WDT_TASK_NAME_LEN];   /**< Task name */
    uint32_t expected_period_ms;     /**< Expected check-in period */
    uint64_t last_checkin_ms;        /**< Last check-in timestamp */
    uint32_t miss_count;             /**< Consecutive missed check-ins */
    uint32_t total_checkins;         /**< Total check-ins since registration */
    bool active;                     /**< Task is registered and active */
    bool stalled;                    /**< Task is considered stalled */
} WdtTaskEntry_t;

/**
 * @brief Watchdog manager statistics
 */
typedef struct {
    uint32_t hw_kicks;               /**< Hardware watchdog kicks */
    uint32_t task_check_runs;        /**< Task watchdog check runs */
    uint32_t task_stalls_detected;   /**< Tasks detected as stalled */
    uint32_t resets_caused;          /**< Resets caused by watchdog (persistent) */
    uint32_t last_reset_time_s;      /**< Mission time of last WDT reset */
    uint8_t last_stalled_task;       /**< Task ID that caused last stall */
    bool hw_wdt_running;             /**< Hardware watchdog is running */
} WdtStats_t;

/**
 * @brief Pre-reset callback type
 *
 * Called when watchdog reset is imminent to allow state saving.
 */
typedef void (*WdtPreResetCallback_t)(void);

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize watchdog manager
 *
 * Initializes hardware watchdog and task monitoring system.
 * Does NOT start the hardware watchdog - call wdt_mgr_start() for that.
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_init(void);

/**
 * @brief Start hardware watchdog
 *
 * Once started, the hardware watchdog cannot be stopped and must be
 * kicked periodically or the system will reset.
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_start(void);

/**
 * @brief Kick hardware watchdog
 *
 * Call this periodically from the main loop to prevent hardware reset.
 * This also runs the task watchdog check.
 */
void wdt_mgr_kick(void);

/**
 * @brief Register a task for watchdog monitoring
 *
 * @param[in] task_id Task identifier
 * @param[in] name Human-readable task name (max WDT_TASK_NAME_LEN-1 chars)
 * @param[in] expected_period_ms Expected check-in period in milliseconds
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_register_task(WdtTaskId_t task_id,
                                        const char *name,
                                        uint32_t expected_period_ms);

/**
 * @brief Unregister a task from watchdog monitoring
 *
 * @param[in] task_id Task identifier
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_unregister_task(WdtTaskId_t task_id);

/**
 * @brief Task check-in (heartbeat)
 *
 * Each registered task must call this periodically to indicate it is alive.
 *
 * @param[in] task_id Task identifier
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_checkin(WdtTaskId_t task_id);

/**
 * @brief Check if a specific task is stalled
 *
 * @param[in] task_id Task identifier
 * @return true if task is stalled
 */
bool wdt_mgr_is_task_stalled(WdtTaskId_t task_id);

/**
 * @brief Check if any task is stalled
 *
 * @return true if any registered task is stalled
 */
bool wdt_mgr_any_task_stalled(void);

/**
 * @brief Get task watchdog information
 *
 * @param[in] task_id Task identifier
 * @param[out] entry Task entry data
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_get_task_info(WdtTaskId_t task_id,
                                        WdtTaskEntry_t *entry);

/**
 * @brief Get watchdog statistics
 *
 * @param[out] stats Statistics output
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_get_stats(WdtStats_t *stats);

/**
 * @brief Register pre-reset callback
 *
 * The callback is invoked when a watchdog reset is imminent,
 * allowing the application to save critical state.
 *
 * @param[in] callback Function to call before reset
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t wdt_mgr_register_prereset_callback(WdtPreResetCallback_t callback);

/**
 * @brief Check if last reset was due to watchdog
 *
 * @return true if last reset was watchdog timeout
 */
bool wdt_mgr_was_wdt_reset(void);

/**
 * @brief Clear watchdog reset status
 *
 * Call after processing watchdog reset cause.
 */
void wdt_mgr_clear_reset_status(void);

/**
 * @brief Get time remaining before hardware watchdog timeout
 *
 * @return Remaining time in milliseconds
 */
uint32_t wdt_mgr_time_remaining(void);

/**
 * @brief Force a watchdog reset (for testing/recovery)
 *
 * This function does not return.
 */
void wdt_mgr_force_reset(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_WATCHDOG_MGR_H */
