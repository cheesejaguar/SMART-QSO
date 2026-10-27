/**
 * @file eps_lockout.h
 * @brief Clearing the EPS's latched recharge lockout (RID EPS-12)
 *
 * After any cell has been below 2.0 V the EPS latches its charge path off
 * (docs/hardware/EPS_DESIGN.md §3.3). EPS_CLEAR_LOCKOUT (0x14, argument the
 * ASCII code "CHG!") writes the unlock key to the EPS controller (register
 * 0x7D at 0x48) and pulses GPIO_PIN_LOCK_CLR high for 150 ms; the EPS clears
 * the lockout only with both. Send it only after the ground has reviewed
 * the cell history.
 *
 * @requirement SRS-F129 The software shall clear the EPS's latched recharge lockout only on an authenticated command with its code, by an I2C unlock and a LOCK_CLR pulse
 */

#ifndef SMART_QSO_EPS_LOCKOUT_H
#define SMART_QSO_EPS_LOCKOUT_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/** Boot: no pulse, GPIO_PIN_LOCK_CLR low */
void eps_lockout_init(void);

/**
 * @brief EPS_CLEAR_LOCKOUT
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID for a wrong code;
 *         SMART_QSO_ERROR_IO if the unlock or the pulse failed
 */
SmartQsoResult_t eps_lockout_clear(const uint8_t *code, size_t len);

/** Every main-loop pass: ends the pulse */
void eps_lockout_poll(void);

/** Whether the LOCK_CLR pulse is running */
bool eps_lockout_pulsing(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_EPS_LOCKOUT_H */
