/**
 * @file separation_switch.h
 * @brief The one reading of the separation (deployment) switches
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Used by the RF silence (sqso-ffi/src/rf_silence.rs) and the deployment
 * sequence (sqso-ffi/src/deployment.rs), so the polarity is defined in
 * exactly one place (sqso-fsw/src/separation.rs).
 *
 * DEPLOY_SENSE (PE2, SQSO-ICX-001 §3.1 pin 16): high = DS2 released. The EPS
 * drives the line through 1 MΩ from DS2's drive node, with 1 MΩ to ground
 * and a clamp to 3V3_AO (EPS board R113, R114, D23); the OBC input has no
 * internal pull, which would override that source (RID SMA-B02).
 *
 * The OBC is powered only after separation (the switches remove power from
 * the bus, CDS 2.3.2.4), and the RF silence restarts at every power-on. The
 * sense line restarts the timers only until separation is established on
 * this boot: released for SEPARATION_LATCH_MS without a break latches the
 * reading released until the next reset; a later stowed reading (an open
 * line) is counted for housekeeping (HK2 SEP) and restarts neither timer.
 * Before the latch a stowed reading counts at once (the bench: umbilical
 * power, switches stowed). Flatsat check V-BFM-07.
 */

#ifndef SMART_QSO_SEPARATION_SWITCH_H
#define SMART_QSO_SEPARATION_SWITCH_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stdint.h>

/** Released this long without a break latches separation, ms */
#define SEPARATION_LATCH_MS 10000U

/**
 * @brief Configure the sense input
 */
void separation_switch_init(void);

/**
 * @brief Whether the timers are to treat the switch as stowed: the line low
 *        before separation latched on this boot; false after it
 */
bool separation_switch_stowed(void);

/**
 * @brief Separation latched on this boot, and the stowed readings since
 * @param latched Out: latched (may be NULL)
 * @param stowed_after Out: stowed episodes after the latch (may be NULL)
 * @return SMART_QSO_OK, or SMART_QSO_ERROR_BUSY from a nested call
 */
int32_t separation_switch_status(bool *latched, uint32_t *stowed_after);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SEPARATION_SWITCH_H */
