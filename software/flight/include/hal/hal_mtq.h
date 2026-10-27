/**
 * @file hal_mtq.h
 * @brief HAL magnetorquer drive (SQSO-OBC-BRD-001 B §10)
 *
 * Three DRV8876-Q1 H-bridges in PH/EN mode from VBAT_MTQ. The direction of
 * each coil is its PH line (GPIO_PIN_MTQ_X_DIR to GPIO_PIN_MTQ_Z_DIR,
 * hal_gpio.h), set by the caller before a burst; EN is a 40 kHz PWM (TIM1
 * CH1-CH3) whose duty sets the average coil current; MTQ_ARM (TIM15 CH1,
 * one-pulse mode) wakes the bridges for one burst and drops by itself at
 * its end, whatever the software does next. A hardware off-timer on the
 * board ends any burst after 0.92-0.98 s and reports it on
 * GPIO_PIN_MTQ_EXPIRED; the bridges' faults are GPIO_PIN_MTQ_FAULT_N.
 *
 * @requirement SRS-F141 Magnetorquer drive: PH before ARM, PWM duty, hardware-timed burst
 */

#ifndef SMART_QSO_HAL_MTQ_H
#define SMART_QSO_HAL_MTQ_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/** PWM frequency of the bridge enables, Hz */
#define HAL_MTQ_PWM_HZ 40000U

/** Longest burst the HAL accepts, ms (the off-timer's lower bound is 920 ms) */
#define HAL_MTQ_MAX_BURST_MS 900U

/** Duty resolution: duty is given in parts per thousand */
#define HAL_MTQ_DUTY_FULL 1000U

/**
 * @brief Start one coil burst
 *
 * Sets the three PWM duties, then raises MTQ_ARM for @p on_ms; the timer
 * lowers it at the end of the burst without the software. A burst running
 * when this is called is replaced (its timer restarts).
 *
 * @param duty_permille Duty of the X, Y and Z bridges, 0 to HAL_MTQ_DUTY_FULL
 * @param on_ms         Burst length, 1 to HAL_MTQ_MAX_BURST_MS
 * @return SMART_QSO_OK; SMART_QSO_ERROR_NULL_PTR; SMART_QSO_ERROR_INVALID for
 *         a duty above full or a length outside 1 to HAL_MTQ_MAX_BURST_MS
 */
SmartQsoResult_t hal_mtq_burst(const uint16_t duty_permille[3], uint32_t on_ms);

/**
 * @brief End any burst now: MTQ_ARM low, duties 0
 *
 * @return SMART_QSO_OK
 */
SmartQsoResult_t hal_mtq_off(void);

/**
 * @brief Whether MTQ_ARM is high now (a burst is running)
 */
bool hal_mtq_active(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_MTQ_H */
