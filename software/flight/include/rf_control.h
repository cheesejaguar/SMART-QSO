/**
 * @file rf_control.h
 * @brief Transmitter permission and latched TX-OFF (cessation of emissions)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Single authority for whether any transmitter (beacon or transponder) may
 * radiate. TX-OFF drives the RF transmitter enable line (GPIO_PIN_RF_TX_EN)
 * low and latches the inhibit in non-volatile memory, so it survives resets
 * and is never cleared by mode logic (e.g. SAFE mode re-enabling the beacon).
 * Only rf_control_tx_on() clears it. The module has no dependency on the AI
 * payload.
 *
 * Telecommand reception is not switched by this module or by the EPS
 * "radio_enabled" flag, which controls the transponder path only: the command
 * receiver must stay powered in every mode so that TX-OFF can always be
 * received (47 CFR 97.207(b), ITU RR 22.1).
 *
 * Fail-safe rules:
 *  - Before rf_control_init(), transmission is not permitted.
 *  - Blank NVM (first boot) -> permitted.
 *  - Corrupt NVM record or unreadable flash -> inhibited until commanded on.
 *  - TX-OFF drives the enable line low before persisting, so a flash failure
 *    still leaves the transmitter off.
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 */

#ifndef SMART_QSO_RF_CONTROL_H
#define SMART_QSO_RF_CONTROL_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdbool.h>

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Downlink transmit power (BASELINE_PARAMETERS section 7: 0.5 W
 *        nominal, 1 W maximum)
 *
 * The level is persisted and reported here; applying it to the PA is the
 * transmitter driver's job, whose interface is TBD in the RF design.
 */
typedef enum {
    RF_POWER_0W5 = 0,          /**< 0.5 W (default) */
    RF_POWER_1W  = 1,          /**< 1 W */
    RF_POWER_LEVEL_COUNT = 2
} RfPowerLevel_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Load the latched TX state and drive the RF transmitter enable line
 *
 * Call once at boot, before anything can transmit.
 *
 * @return SMART_QSO_OK once the module is initialized (including when the
 *         persisted record was corrupt and the transmitter was inhibited)
 */
SmartQsoResult_t rf_control_init(void);

/**
 * @brief Cease all transmissions and latch the inhibit across resets
 *
 * @return SMART_QSO_OK if the inhibit was applied and persisted;
 *         SMART_QSO_ERROR_IO if persistence failed (the transmitter is still off)
 */
SmartQsoResult_t rf_control_tx_off(void);

/**
 * @brief Clear the inhibit and permit transmission
 *
 * @return SMART_QSO_OK if applied and persisted; SMART_QSO_ERROR_IO if
 *         persistence failed (permitted until the next reset only)
 */
SmartQsoResult_t rf_control_tx_on(void);

/**
 * @brief Whether any transmitter may radiate now
 *
 * @return false before initialization or while TX-OFF is latched
 */
bool rf_control_tx_permitted(void);

/**
 * @brief Select the downlink power level and persist it
 *
 * Never enables a transmitter that TX-OFF has latched off.
 *
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID for an unknown level;
 *         SMART_QSO_ERROR_IO if persistence failed (applied until reset)
 */
SmartQsoResult_t rf_control_set_power_level(RfPowerLevel_t level);

/**
 * @brief Current downlink power level (0.5 W if never set or unreadable)
 */
RfPowerLevel_t rf_control_get_power_level(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_RF_CONTROL_H */
