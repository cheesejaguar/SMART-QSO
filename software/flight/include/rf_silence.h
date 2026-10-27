/**
 * @file rf_silence.h
 * @brief 45-minute post-deployment RF silence (CDS Rev 14.1 2.4.5, 2.3.2.4)
 *
 * Implemented in Rust (software/rust/sqso-fsw/src/rf_silence.rs,
 * software/rust/sqso-ffi/src/rf_silence.rs).
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * No RF signal may be generated or transmitted until 45 minutes after
 * deployment, and a deployment-switch toggle restarts the timer. While the
 * silence is active the whole radio is unpowered (GPIO_PIN_RADIO_EN low) as
 * well as the transmitter (GPIO_PIN_RF_TX_EN low), so no telecommand can be
 * received either (conservative reading of "generate any signal"; TBR with
 * the launch provider). No telecommand can shorten or clear the silence.
 *
 * The deployment switches are power inhibits, so deployment and any switch
 * toggle reach the OBC as a power-on reset. Therefore:
 *  - power-on, brownout, pin or unknown reset: restart the count from 0;
 *  - watchdog or software reset: resume from the persisted count (which lags
 *    by up to RF_SILENCE_PERSIST_INTERVAL_S, so the silence only lengthens);
 *  - a stowed switch seen while powered: restart from 0, until the silence
 *    has been served; once served it is latched (LV-15: a later stowed
 *    reading is a failed line in orbit, and a real re-stow removes power);
 *  - blank or corrupt record: count from 0.
 * Once complete, the state persists across later resets.
 *
 * Ground testing completes the silence on the bench, so before shipping the
 * GSE ARM operation (umbilical only) resets it. ARMED does not count while
 * powered; the next boot starts the count. ARM must be the last powered
 * action before integration (TP-FAT-002, PRE_SHIP_CHECKLIST).
 *
 * @requirement SYS-224 No RF signal earlier than 45 minutes after deployment; switch toggle resets the timer
 */

#ifndef SMART_QSO_RF_SILENCE_H
#define SMART_QSO_RF_SILENCE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdbool.h>
#include <stdint.h>

/** Silence after deployment (CDS Rev 14.1 2.4.5) */
#define RF_SILENCE_DURATION_S           (45U * 60U)

/** Count persistence interval */
#define RF_SILENCE_PERSIST_INTERVAL_S   60U

/** Silence state */
typedef enum {
    RF_SILENCE_ARMED    = 1, /**< Set by GSE ARM; count starts at the next boot */
    RF_SILENCE_COUNTING = 2, /**< Silent; counting toward RF_SILENCE_DURATION_S */
    RF_SILENCE_COMPLETE = 3  /**< Silence served; radio may operate */
} RfSilenceState_t;

/**
 * @brief Load the state, apply the reset-cause rules, persist, and set the
 *        inhibit on rf_control. Call at boot before rf_control_init().
 */
SmartQsoResult_t rf_silence_init(void);

/**
 * @brief Advance the count by elapsed time; restart it while stowed
 *
 * @param[in] elapsed_ms Time since the previous tick
 */
void rf_silence_tick(uint32_t elapsed_ms);

/**
 * @brief Re-arm for flight (GSE only): silent now, count from the next boot
 */
SmartQsoResult_t rf_silence_arm(void);

/**
 * @brief Whether the silence is in force
 */
bool rf_silence_active(void);

/**
 * @brief Current state
 */
RfSilenceState_t rf_silence_get_state(void);

/**
 * @brief Seconds counted toward the silence
 */
uint32_t rf_silence_elapsed_seconds(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_RF_SILENCE_H */
