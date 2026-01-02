/**
 * @file hal_timer.h
 * @brief HAL Timer Interface
 *
 * Provides timing and delay abstraction.
 *
 * @requirement SRS-P001 Main control loop shall execute within 20 ms
 */

#ifndef SMART_QSO_HAL_TIMER_H
#define SMART_QSO_HAL_TIMER_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* Timer Functions                                                            */
/*===========================================================================*/

/**
 * @brief Initialize timer subsystem
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_timer_init(void);

/**
 * @brief Get current time in milliseconds
 *
 * @return Milliseconds since system start (wraps at 2^64)
 */
uint64_t hal_timer_get_ms(void);

/**
 * @brief Get current time in microseconds
 *
 * @return Microseconds since system start (wraps at 2^64)
 */
uint64_t hal_timer_get_us(void);

/**
 * @brief Delay for specified milliseconds
 *
 * @param ms Milliseconds to delay
 */
void hal_timer_delay_ms(uint32_t ms);

/**
 * @brief Delay for specified microseconds
 *
 * @param us Microseconds to delay
 */
void hal_timer_delay_us(uint32_t us);

/**
 * @brief Check if timeout has elapsed
 *
 * @param start_ms  Start time in milliseconds
 * @param timeout_ms Timeout duration in milliseconds
 * @return true if timeout has elapsed
 */
bool hal_timer_timeout(uint64_t start_ms, uint32_t timeout_ms);

/**
 * @brief Get elapsed time since start
 *
 * @param start_ms Start time in milliseconds
 * @return Elapsed milliseconds
 */
uint64_t hal_timer_elapsed_ms(uint64_t start_ms);

/*===========================================================================*/
/* Periodic Timer Support                                                     */
/*===========================================================================*/

/**
 * @brief Timer callback function type
 */
typedef void (*HalTimerCallback_t)(void *context);

/**
 * @brief Timer ID type
 */
typedef uint8_t HalTimerId_t;

/**
 * @brief Start a periodic timer
 *
 * @param period_ms Timer period in milliseconds
 * @param callback  Function to call on each period
 * @param context   Context pointer passed to callback
 * @param timer_id  Output: Timer ID for later reference
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_timer_start_periodic(uint32_t period_ms,
                                           HalTimerCallback_t callback,
                                           void *context,
                                           HalTimerId_t *timer_id);

/**
 * @brief Stop a periodic timer
 *
 * @param timer_id Timer ID to stop
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_timer_stop(HalTimerId_t timer_id);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_TIMER_H */
