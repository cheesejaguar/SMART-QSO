/**
 * @file stm32l4_clock.h
 * @brief STM32L4R5 system clock (target only, FINDINGS_REGISTER SW-01)
 */

#ifndef SMART_QSO_STM32L4_CLOCK_H
#define SMART_QSO_STM32L4_CLOCK_H

#include <stdint.h>

#include "smart_qso.h"

/** Core clock after reset: MSI at 4 MHz */
#define STM32L4_CLOCK_RESET_HZ   4000000U

/** Core clock after stm32l4_clock_init() succeeds */
#define STM32L4_CLOCK_TARGET_HZ  80000000U

/**
 * @brief Raise the core clock from MSI 4 MHz to 80 MHz through the main PLL
 *
 * Called once from the reset handler, before main().
 *
 * @return SMART_QSO_OK at 80 MHz; SMART_QSO_ERROR if the flash wait states,
 *         PLL lock or clock switch did not take, leaving the core on MSI 4 MHz
 */
SmartQsoResult_t stm32l4_clock_init(void);

/**
 * @brief Current core clock in Hz (4 MHz until stm32l4_clock_init() succeeds)
 */
uint32_t stm32l4_core_clock_hz(void);

#endif /* SMART_QSO_STM32L4_CLOCK_H */
