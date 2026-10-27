/**
 * @file hal_capture.h
 * @brief HAL timer input captures (SQSO-ICX-001 B §4.3)
 *
 * Three inputs are timed by timer captures rather than read as levels:
 * the GNSS receiver's 1PPS (PC6, TIM3 CH1), the mission clock's 1 Hz
 * CLKOUT (PC8, TIM3 CH3) and the exciter's keyed line PTT_EXCITER (PC7,
 * TIM8 CH2, both edges). The first two give the MCU's own clock against an
 * outside one; the third the time the exciter was keyed, measured instead
 * of estimated from frame lengths (SQSO-ICD-003 §5.3).
 *
 * @requirement SRS-F145 Mission clock: reset time stamps and the clock check
 */

#ifndef SMART_QSO_HAL_CAPTURE_H
#define SMART_QSO_HAL_CAPTURE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/**
 * @brief Captured inputs
 */
typedef enum {
    CAPTURE_GNSS_PPS = 0,   /**< PC6 TIM3 CH1: GNSS 1PPS (rising edges) */
    CAPTURE_RTC_1HZ  = 1,   /**< PC8 TIM3 CH3: RV-3028 CLKOUT, 1 Hz (rising edges) */
    CAPTURE_PTT_EXC  = 2,   /**< PC7 TIM8 CH2: PTT_EXCITER (both edges) */
    CAPTURE_COUNT
} HalCaptureInput_t;

/**
 * @brief The latest period of a periodic input, µs of the MCU's clock
 *
 * The time between its two latest rising edges, if both arrived within the
 * last 3 s.
 *
 * @param input  CAPTURE_GNSS_PPS or CAPTURE_RTC_1HZ
 * @param period_us Output: the period, µs
 * @return SMART_QSO_OK; SMART_QSO_ERROR_TIMEOUT with no recent period;
 *         SMART_QSO_ERROR_INVALID for another input; SMART_QSO_ERROR_NULL_PTR
 */
SmartQsoResult_t hal_capture_period_us(HalCaptureInput_t input, uint32_t *period_us);

/**
 * @brief Time an input has been high since boot, ms
 *
 * @param input CAPTURE_PTT_EXC
 * @return The accumulated high time, ms (0 for another input)
 */
uint32_t hal_capture_high_ms(HalCaptureInput_t input);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_CAPTURE_H */
