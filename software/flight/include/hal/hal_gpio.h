/**
 * @file hal_gpio.h
 * @brief HAL GPIO Interface
 *
 * Provides GPIO control abstraction for load switches and signals.
 *
 * @requirement SRS-F011 Control power to subsystems via load switches
 */

#ifndef SMART_QSO_HAL_GPIO_H
#define SMART_QSO_HAL_GPIO_H

#ifdef __cplusplus
extern "C" {
#endif

#include "../smart_qso.h"

/*===========================================================================*/
/* GPIO Pin Definitions                                                       */
/*===========================================================================*/

/**
 * @brief GPIO pin enumeration
 *
 * Logical pin names mapped to physical pins in implementation.
 */
typedef enum {
    GPIO_PIN_PAYLOAD_EN    = 0,   /**< Jetson payload power enable */
    GPIO_PIN_RADIO_EN      = 1,   /**< Radio/transponder power enable */
    GPIO_PIN_ADCS_EN       = 2,   /**< ADCS power enable */
    GPIO_PIN_HEATER_EN     = 3,   /**< Heater power enable */
    GPIO_PIN_DEPLOY_EN     = 4,   /**< Deployment mechanism enable */
    GPIO_PIN_LED_STATUS    = 5,   /**< Status LED */
    GPIO_PIN_JETSON_PWR    = 6,   /**< Jetson power control */
    GPIO_PIN_RF_TX_EN      = 7,   /**< RF transmitter enable */
    GPIO_PIN_ALERT         = 8,   /**< EPS alert input */
    GPIO_PIN_DEPLOY_SENSE  = 9,   /**< Deployment sense input */
    GPIO_PIN_COUNT         = 10
} HalGpioPin_t;

/**
 * @brief GPIO direction
 */
typedef enum {
    GPIO_DIR_INPUT  = 0,
    GPIO_DIR_OUTPUT = 1
} HalGpioDir_t;

/**
 * @brief GPIO pull configuration
 */
typedef enum {
    GPIO_PULL_NONE = 0,
    GPIO_PULL_UP   = 1,
    GPIO_PULL_DOWN = 2
} HalGpioPull_t;

/*===========================================================================*/
/* GPIO Functions                                                             */
/*===========================================================================*/

/**
 * @brief Initialize GPIO subsystem
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_init(void);

/**
 * @brief Configure a GPIO pin
 *
 * @param pin       Pin to configure
 * @param direction Input or output
 * @param pull      Pull-up/down configuration
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_config(HalGpioPin_t pin, HalGpioDir_t direction,
                                  HalGpioPull_t pull);

/**
 * @brief Set GPIO output state
 *
 * @param pin   Pin to set
 * @param state true for high, false for low
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_set(HalGpioPin_t pin, bool state);

/**
 * @brief Get GPIO input state
 *
 * @param pin Pin to read
 * @return true if high, false if low
 */
bool hal_gpio_get(HalGpioPin_t pin);

/**
 * @brief Toggle GPIO output
 *
 * @param pin Pin to toggle
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hal_gpio_toggle(HalGpioPin_t pin);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_GPIO_H */
