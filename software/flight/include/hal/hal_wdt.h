/**
 * @file hal_wdt.h
 * @brief HAL Watchdog Timer Interface
 *
 * Provides watchdog timer abstraction for system health monitoring.
 *
 * @requirement SRS-F050 Kick hardware watchdog at least every 30 seconds
 * @requirement SRS-F051 Save state before watchdog reset
 * @requirement SRS-S003 Software shall implement hardware watchdog
 */

#ifndef SMART_QSO_HAL_WDT_H
#define SMART_QSO_HAL_WDT_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* Watchdog Configuration                                                     */
/*===========================================================================*/

/**
 * @brief Default watchdog timeout in milliseconds
 */
#define HAL_WDT_DEFAULT_TIMEOUT_MS  30000

/**
 * @brief Minimum watchdog timeout
 */
#define HAL_WDT_MIN_TIMEOUT_MS      1000

/**
 * @brief Maximum watchdog timeout
 */
#define HAL_WDT_MAX_TIMEOUT_MS      60000

/**
 * @brief Watchdog mode
 */
typedef enum {
    WDT_MODE_RESET     = 0,  /**< Reset system on timeout */
    WDT_MODE_INTERRUPT = 1   /**< Generate interrupt on timeout (if supported) */
} HalWdtMode_t;

/*===========================================================================*/
/* Watchdog Functions                                                         */
/*===========================================================================*/

/**
 * @brief Initialize watchdog timer
 *
 * @param timeout_ms Timeout in milliseconds
 * @param mode       Watchdog mode
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_wdt_init(uint32_t timeout_ms, HalWdtMode_t mode);

/**
 * @brief Start watchdog timer
 *
 * Once started, the watchdog must be kicked periodically or the system
 * will reset.
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_wdt_start(void);

/**
 * @brief Stop watchdog timer (if supported)
 *
 * Note: Many hardware watchdogs cannot be stopped once started.
 *
 * @return SMART_QSO_OK on success, SMART_QSO_ERROR if not supported
 */
SmartQsoResult_t hal_wdt_stop(void);

/**
 * @brief Kick (refresh) watchdog timer
 *
 * Must be called periodically to prevent watchdog reset.
 */
void hal_wdt_kick(void);

/**
 * @brief Get remaining time before timeout
 *
 * @return Remaining time in milliseconds (0 if not available)
 */
uint32_t hal_wdt_remaining_ms(void);

/**
 * @brief Check if last reset was due to watchdog
 *
 * @return true if last reset was watchdog timeout
 */
bool hal_wdt_caused_reset(void);

/**
 * @brief Clear watchdog reset flag
 *
 * Call after processing watchdog reset cause.
 */
void hal_wdt_clear_reset_flag(void);

/**
 * @brief Set early warning callback (if supported)
 *
 * Called before actual timeout to allow saving state.
 *
 * @param callback Function to call before timeout
 * @param context  Context pointer passed to callback
 * @return SMART_QSO_OK on success, SMART_QSO_ERROR if not supported
 */
SmartQsoResult_t hal_wdt_set_warning_callback(void (*callback)(void *context),
                                               void *context);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_WDT_H */
