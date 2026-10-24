/**
 * @file separation_switch.h
 * @brief The one reading of the separation (deployment) switches
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Used by the RF silence (sqso-ffi/src/rf_silence.rs) and the deployment
 * sequence (sqso-ffi/src/deployment.rs), so the polarity is defined in
 * exactly one place.
 *
 * TBR: the line is pulled up and reads high while stowed (in the
 * dispenser). Confirm against the separation-switch schematic. If inverted on
 * orbit, the RF-silence and deployment timers restart forever: the spacecraft
 * stays silent with nothing deployed (fail safe, but mission-ending), so the
 * pre-ship test checks that released switches read released.
 */

#ifndef SMART_QSO_SEPARATION_SWITCH_H
#define SMART_QSO_SEPARATION_SWITCH_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>

/**
 * @brief Configure the sense input
 */
void separation_switch_init(void);

/**
 * @brief Whether the switches read stowed (in the dispenser)
 */
bool separation_switch_stowed(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SEPARATION_SWITCH_H */
