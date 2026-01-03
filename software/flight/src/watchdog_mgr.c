/**
 * @file watchdog_mgr.c
 * @brief Watchdog Manager implementation for SMART-QSO
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module manages both hardware and software (task) watchdogs:
 * - Hardware watchdog is kicked from the main loop
 * - Task watchdogs track individual task health
 * - Pre-reset callbacks allow state persistence
 *
 * @requirement SRS-F050 Kick hardware watchdog at least every 30 seconds
 * @requirement SRS-F051 Save state before watchdog reset
 */

#include "watchdog_mgr.h"
#include "hal/hal_wdt.h"
#include "fault_mgmt.h"
#include "safe_string.h"
#include <stddef.h>
#include <stdio.h>

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Task watchdog entries */
static WdtTaskEntry_t s_tasks[WDT_MAX_TASKS];

/** Watchdog statistics */
static WdtStats_t s_stats;

/** Pre-reset callback */
static WdtPreResetCallback_t s_prereset_callback = NULL;

/** Last task check time */
static uint64_t s_last_task_check_ms = 0;

/** Module initialized flag */
static bool s_initialized = false;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Hardware watchdog warning callback
 *
 * Called by HAL when watchdog timeout is imminent.
 */
static void hw_wdt_warning_handler(void *context)
{
    (void)context;

    /* Log the impending reset */
    (void)fault_log_add(FAULT_TYPE_WATCHDOG,
                        FAULT_SEVERITY_CRITICAL,
                        "Hardware watchdog timeout imminent",
                        0.0);

    /* Call application pre-reset callback */
    if (s_prereset_callback != NULL) {
        s_prereset_callback();
    }

    s_stats.resets_caused++;
}

/**
 * @brief Check all registered tasks for stalls
 */
static void check_task_watchdogs(void)
{
    uint64_t now = smart_qso_now_ms();

    for (uint8_t i = 0; i < WDT_MAX_TASKS; i++) {
        if (!s_tasks[i].active) {
            continue;
        }

        /* Calculate time since last check-in */
        uint64_t elapsed = now - s_tasks[i].last_checkin_ms;

        /* Check if task has missed its expected period */
        if (elapsed > s_tasks[i].expected_period_ms) {
            s_tasks[i].miss_count++;

            if (s_tasks[i].miss_count >= WDT_TASK_MISS_THRESHOLD) {
                if (!s_tasks[i].stalled) {
                    /* Task just became stalled */
                    s_tasks[i].stalled = true;
                    s_stats.task_stalls_detected++;
                    s_stats.last_stalled_task = i;
                    s_stats.last_reset_time_s = (uint32_t)(now / 1000U);

                    /* Log the stall */
                    char msg[SMART_QSO_FAULT_DESC_LEN];
                    int written = snprintf(msg, sizeof(msg),
                                           "Task %s stalled", s_tasks[i].name);
                    if ((written < 0) || ((size_t)written >= sizeof(msg))) {
                        /* Truncated or error - use simple message */
                        (void)safe_strncpy(msg, sizeof(msg), "Task stalled",
                                           sizeof(msg) - 1, NULL);
                    }
                    (void)fault_log_add(FAULT_TYPE_WATCHDOG,
                                        FAULT_SEVERITY_WARNING,
                                        msg,
                                        0.0);
                }
            }
        } else {
            /* Task is healthy */
            s_tasks[i].miss_count = 0;
            s_tasks[i].stalled = false;
        }
    }

    s_stats.task_check_runs++;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t wdt_mgr_init(void)
{
    /* Clear task entries */
    (void)safe_memset(s_tasks, sizeof(s_tasks), 0, sizeof(s_tasks));

    /* Clear statistics (except persistent fields) */
    uint32_t saved_resets = s_stats.resets_caused;
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));
    s_stats.resets_caused = saved_resets;

    s_prereset_callback = NULL;
    s_last_task_check_ms = 0;

    /* Initialize hardware watchdog */
    SmartQsoResult_t result = hal_wdt_init(WDT_HW_TIMEOUT_MS, WDT_MODE_RESET);
    if (result != SMART_QSO_OK) {
        return result;
    }

    /* Register for early warning if supported */
    (void)hal_wdt_set_warning_callback(hw_wdt_warning_handler, NULL);

    s_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t wdt_mgr_start(void)
{
    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    SmartQsoResult_t result = hal_wdt_start();
    if (result == SMART_QSO_OK) {
        s_stats.hw_wdt_running = true;
        s_last_task_check_ms = smart_qso_now_ms();
    }

    return result;
}

void wdt_mgr_kick(void)
{
    if (!s_initialized) {
        return;
    }

    /* Kick hardware watchdog */
    hal_wdt_kick();
    s_stats.hw_kicks++;

    /* Check task watchdogs periodically */
    uint64_t now = smart_qso_now_ms();
    if ((now - s_last_task_check_ms) >= WDT_TASK_CHECK_INTERVAL_MS) {
        check_task_watchdogs();
        s_last_task_check_ms = now;
    }
}

SmartQsoResult_t wdt_mgr_register_task(WdtTaskId_t task_id,
                                        const char *name,
                                        uint32_t expected_period_ms)
{
    if (task_id >= WDT_MAX_TASKS) {
        return SMART_QSO_ERROR_PARAM;
    }

    if (name == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (expected_period_ms == 0) {
        return SMART_QSO_ERROR_PARAM;
    }

    WdtTaskEntry_t *entry = &s_tasks[task_id];

    (void)safe_strncpy(entry->name, sizeof(entry->name),
                       name, WDT_TASK_NAME_LEN - 1, NULL);
    entry->expected_period_ms = expected_period_ms;
    entry->last_checkin_ms = smart_qso_now_ms();
    entry->miss_count = 0;
    entry->total_checkins = 0;
    entry->active = true;
    entry->stalled = false;

    return SMART_QSO_OK;
}

SmartQsoResult_t wdt_mgr_unregister_task(WdtTaskId_t task_id)
{
    if (task_id >= WDT_MAX_TASKS) {
        return SMART_QSO_ERROR_PARAM;
    }

    s_tasks[task_id].active = false;
    return SMART_QSO_OK;
}

SmartQsoResult_t wdt_mgr_checkin(WdtTaskId_t task_id)
{
    if (task_id >= WDT_MAX_TASKS) {
        return SMART_QSO_ERROR_PARAM;
    }

    if (!s_tasks[task_id].active) {
        return SMART_QSO_ERROR;
    }

    s_tasks[task_id].last_checkin_ms = smart_qso_now_ms();
    s_tasks[task_id].miss_count = 0;
    s_tasks[task_id].stalled = false;
    s_tasks[task_id].total_checkins++;

    return SMART_QSO_OK;
}

bool wdt_mgr_is_task_stalled(WdtTaskId_t task_id)
{
    if (task_id >= WDT_MAX_TASKS) {
        return false;
    }

    return s_tasks[task_id].stalled;
}

bool wdt_mgr_any_task_stalled(void)
{
    for (uint8_t i = 0; i < WDT_MAX_TASKS; i++) {
        if (s_tasks[i].active && s_tasks[i].stalled) {
            return true;
        }
    }
    return false;
}

SmartQsoResult_t wdt_mgr_get_task_info(WdtTaskId_t task_id,
                                        WdtTaskEntry_t *entry)
{
    if (entry == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (task_id >= WDT_MAX_TASKS) {
        return SMART_QSO_ERROR_PARAM;
    }

    (void)safe_memcpy(entry, sizeof(*entry),
                      &s_tasks[task_id], sizeof(WdtTaskEntry_t));
    return SMART_QSO_OK;
}

SmartQsoResult_t wdt_mgr_get_stats(WdtStats_t *stats)
{
    if (stats == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(stats, sizeof(*stats), &s_stats, sizeof(s_stats));
    return SMART_QSO_OK;
}

SmartQsoResult_t wdt_mgr_register_prereset_callback(WdtPreResetCallback_t callback)
{
    s_prereset_callback = callback;
    return SMART_QSO_OK;
}

bool wdt_mgr_was_wdt_reset(void)
{
    return hal_wdt_caused_reset();
}

void wdt_mgr_clear_reset_status(void)
{
    hal_wdt_clear_reset_flag();
}

uint32_t wdt_mgr_time_remaining(void)
{
    return hal_wdt_remaining_ms();
}

void wdt_mgr_force_reset(void)
{
    /* Log the forced reset */
    (void)fault_log_add(FAULT_TYPE_WATCHDOG,
                        FAULT_SEVERITY_CRITICAL,
                        "Forced watchdog reset",
                        0.0);

    /* Call pre-reset callback */
    if (s_prereset_callback != NULL) {
        s_prereset_callback();
    }

    /* Stop kicking watchdog and wait for reset */
    /* In simulation, just loop forever */
    while (1) {
        /* Wait for hardware watchdog to reset us */
    }
}
