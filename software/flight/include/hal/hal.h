/**
 * @file hal.h
 * @brief Hardware Abstraction Layer (HAL) Master Header
 *
 * This header provides the hardware abstraction layer interface for the
 * SMART-QSO flight software. All hardware-specific code is isolated behind
 * this interface to enable portability between simulation and flight builds.
 *
 * @requirement SRS-Q020 Software shall compile on ARM and x86 platforms
 * @requirement SRS-Q021 Hardware dependencies shall be isolated in HAL layer
 * @requirement SRS-Q022 Software shall support simulation and flight builds
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 * @note Following GeneSat/PharmaSat/O/OREOS/EcAMSat HAL patterns
 */

#ifndef SMART_QSO_HAL_H
#define SMART_QSO_HAL_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* HAL Configuration                                                          */
/*===========================================================================*/

/**
 * @brief HAL target selection
 *
 * Define one of these in CMakeLists.txt to select the HAL target:
 * - HAL_TARGET_STM32L4: STM32L4 microcontroller
 * - HAL_TARGET_APOLLO4: Ambiq Apollo4 microcontroller
 * - HAL_TARGET_SIMULATION: Host simulation (Linux/macOS/Windows)
 */
#if !defined(HAL_TARGET_STM32L4) && !defined(HAL_TARGET_APOLLO4) && !defined(HAL_TARGET_SIMULATION)
#define HAL_TARGET_SIMULATION 1
#endif

/*===========================================================================*/
/* HAL Initialization                                                         */
/*===========================================================================*/

/**
 * @brief Initialize all HAL subsystems
 *
 * Must be called before any other HAL functions. Initializes:
 * - System clock
 * - GPIO
 * - I2C
 * - SPI
 * - UART
 * - Timers
 * - ADC
 * - Flash/NVM
 * - Watchdog
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t hal_init(void);

/**
 * @brief De-initialize HAL (for simulation cleanup)
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_deinit(void);

/*===========================================================================*/
/* Include HAL Component Headers                                              */
/*===========================================================================*/

#include "hal_gpio.h"
#include "hal_i2c.h"
#include "hal_spi.h"
#include "hal_uart.h"
#include "hal_timer.h"
#include "hal_adc.h"
#include "hal_flash.h"
#include "hal_wdt.h"

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_H */
