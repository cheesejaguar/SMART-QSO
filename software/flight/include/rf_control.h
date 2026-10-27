/**
 * @file rf_control.h
 * @brief Transmitter permission and latched TX-OFF (cessation of emissions)
 *
 * Implemented in Rust (software/rust/sqso-fsw/src/rf_control.rs,
 * software/rust/sqso-ffi/src/rf_control.rs).
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
 * Telecommand reception is not switched by TX-OFF, by the loss-of-contact
 * inhibit or by the EPS "radio_enabled" flag (transponder path only): the
 * command receiver stays powered so that TX-OFF can always be received
 * (47 CFR 97.207(b), ITU RR 22.1). The one exception is the 45-minute
 * post-deployment silence (rf_silence.h, CDS 2.4.5), when the whole radio is
 * unpowered through GPIO_PIN_RADIO_EN, which this module owns; nothing can
 * transmit then either.
 *
 * Fail-safe rules:
 *  - Before rf_control_init(), transmission is not permitted.
 *  - Blank NVM (first boot) -> permitted.
 *  - Corrupt NVM record or unreadable flash -> inhibited until commanded on.
 *  - TX-OFF drives the enable line low before persisting, so a flash failure
 *    still leaves the transmitter off.
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 * @requirement SYS-224 No RF signal earlier than 45 minutes after deployment
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
 * The level is persisted and reported here; the downlink applies it to the
 * transmitter (GPIO_PIN_TX_PWR_SEL, downlink.h), capped at 0.5 W by its
 * VSWR protection.
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
 * @brief Automatic inhibits, independent of the TX-OFF latch and of each other
 */
typedef enum {
    RF_INHIBIT_CONTACT    = 0x01U, /**< Loss of contact (contact_monitor.h): transmitters off */
    RF_INHIBIT_RF_SILENCE = 0x02U, /**< Post-deployment silence (rf_silence.h): whole radio off */
    RF_INHIBIT_PASSIVATED = 0x04U, /**< End-of-life passivation (passivation.h): transmitters off for good, never lifted */
    RF_INHIBIT_RX_CYCLE   = 0x08U, /**< Receiver recovery power cycle (receiver.h): whole radio off */
    RF_INHIBIT_ANTS_PENDING = 0x10U /**< AntS deployment pending (deployment.h, decision C-19): transmitters off */
} RfInhibit_t;

/**
 * @brief Apply or lift one automatic inhibit
 *
 * Transmission is permitted only when the TX-OFF latch and every automatic
 * inhibit are clear; clearing one never clears another. Only
 * RF_INHIBIT_RF_SILENCE and RF_INHIBIT_RX_CYCLE also remove radio power
 * (GPIO_PIN_RADIO_EN), so the telecommand receiver stays on under TX-OFF,
 * loss of contact and passivation. RF_INHIBIT_PASSIVATED is never lifted.
 * May be called before rf_control_init(); the lines are driven once
 * initialized.
 */
void rf_control_set_inhibit(RfInhibit_t source, bool active);

/** The exciter may stay keyed unexplained this long, ms (C-014-9) */
#define RF_CONTROL_PTT_LIMIT_MS 6000U

/**
 * @brief The exciter's keying watch (SW-HAZ-014 control C-014-9)
 *
 * The beacon path's TNC_PTT bypasses the transponder heartbeat and its
 * 120 s timer. Called each main-loop pass: PTT_EXCITER (GPIO_PIN_PTT_EXC)
 * read high for more than RF_CONTROL_PTT_LIMIT_MS with XPDR_KEY
 * (GPIO_PIN_XPDR_KEY, the repeat path) low and no downlink frame
 * outstanding latches TX-OFF (rf_control_tx_off(): RF_TX_EN low,
 * persisted) and logs a CRITICAL fault, once per episode; the watch re-arms
 * when the condition clears. Recovery is TX-OFF's: TX_ENABLE.
 *
 * @param[in] now_ms             Time now (hal_timer), ms
 * @param[in] frame_outstanding  A downlink frame may be on the air (downlink.h)
 * @return true if it latched TX-OFF now
 *
 * @requirement SRS-F168 TX-OFF when PTT_EXCITER reads high for more than 6 s with the repeat path not keyed and no downlink frame outstanding; logged; recovered by TX_ENABLE
 */
bool rf_control_ptt_supervise(uint64_t now_ms, bool frame_outstanding);

/**
 * @brief Trips of the exciter watch since boot
 */
uint32_t rf_control_ptt_trips(void);

/**
 * @brief Check the RAM copy of the TX-OFF latch against its record (AV-07)
 *
 * Called every minute. A record saying TX-OFF, or one that cannot be
 * read, inhibits; a TX-OFF in RAM that the record does not hold is
 * persisted. Each disagreement is logged.
 *
 * @requirement SRS-F133 Periodic check of the critical persisted state against its CRC-protected records
 */
void rf_control_verify(void);

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
