/**
 * @file separation_switch.c
 * @brief The one reading of the separation (deployment) switches
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#include "separation_switch.h"
#include "hal/hal_gpio.h"

void separation_switch_init(void)
{
    (void)hal_gpio_config(GPIO_PIN_DEPLOY_SENSE, GPIO_DIR_INPUT, GPIO_PULL_UP);
}

bool separation_switch_stowed(void)
{
    /* TBR polarity: high = stowed (see separation_switch.h) */
    return hal_gpio_get(GPIO_PIN_DEPLOY_SENSE);
}
