/**
 * @file hal_reset.h
 * @brief Cause of the most recent reset
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Contract: the first call after boot reads the MCU reset flags, clears them
 * (they are sticky on the STM32L4, so a watchdog reset after a power-on reset
 * would otherwise read as both) and caches the result; later calls return the
 * cached value. Only the host simulation implements this today.
 */

#ifndef SMART_QSO_HAL_RESET_H
#define SMART_QSO_HAL_RESET_H

#ifdef __cplusplus
extern "C" {
#endif

/** Reset causes */
typedef enum {
    HAL_RESET_POWER_ON = 0,  /**< Power applied (deployment, switch toggle, power cycle) */
    HAL_RESET_BROWNOUT = 1,  /**< Supply dipped below the brownout threshold */
    HAL_RESET_WATCHDOG = 2,  /**< Independent or window watchdog */
    HAL_RESET_SOFTWARE = 3,  /**< Software-requested reset */
    HAL_RESET_PIN      = 4,  /**< External reset pin */
    HAL_RESET_UNKNOWN  = 5   /**< No flag set or unrecognized */
} HalResetCause_t;

/**
 * @brief Cause of the most recent reset (read once, then cached)
 */
HalResetCause_t hal_reset_cause(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_RESET_H */
