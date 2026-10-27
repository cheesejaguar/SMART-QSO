/**
 * @file receiver.h
 * @brief Supervision of the 435 MHz command receiver
 *
 * Activity is a valid uplink frame (an AX.25 UI frame with a good FCS, for
 * any station) or the receiver's carrier detect (GPIO_PIN_RX_CD).
 *
 * Recovery (FSW-09): after RECEIVER_SILENCE_MS with the radio powered and no
 * activity, the radio is switched off for RECEIVER_CYCLE_OFF_MS
 * (RF_INHIBIT_RX_CYCLE) and on again, logged; never while the downlink or
 * the transponder is keyed, never within 15 minutes of activity (a pass in
 * progress), at most once per silence of RECEIVER_SILENCE_MS.
 *
 * Known failed (REL-04): the radio powered, the loss-of-contact timeout
 * expired (contact_monitor_inhibited(), persisted across resets), the
 * receiver's health line GPIO_PIN_RX_FAULT high, and a recovery cycle tried
 * in the current silence (after a reset, a new one). Any activity clears
 * it. While it holds, the loss-of-contact inhibit does not silence the
 * beacon (contact_monitor.h); the transponder stays off.
 *
 * @requirement SRS-F115 Hung-receiver recovery by a brief power cycle, never during a pass
 * @requirement SRS-F116 Evidence that the command receiver has failed
 */

#ifndef SMART_QSO_RECEIVER_H
#define SMART_QSO_RECEIVER_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/** No activity for this long before a recovery cycle, ms (24 h) */
#define RECEIVER_SILENCE_MS   86400000ULL
/** Radio off this long in a recovery cycle, ms */
#define RECEIVER_CYCLE_OFF_MS 2000U

/** RX_FAULT ignored this long after RADIO_EN switches on, ms */
#define RECEIVER_FAULT_BLANK_MS 1500U
/** A carrier detect continuously high for longer is not activity, ms (RID SMA-B11) */
#define RECEIVER_CARRIER_MAX_MS 3600000U
/** The TNC STATUS loop query's period, ms (RF board E §8.6) */
#define RECEIVER_QUERY_PERIOD_MS 600000U
/** No STATUS answer this long with the radio on is a receiver-path fault, ms */
#define RECEIVER_NO_ANSWER_MS 86400000ULL
/** RX_FAULT's power-on self-test window after RADIO_EN rises, ms */
#define RECEIVER_POST_FROM_MS 10U
#define RECEIVER_POST_TO_MS 90U

/** Start counting from now (boot) */
void receiver_init(void);

/** Every main-loop pass (keyed: the downlink or the transponder is keyed) */
void receiver_step(bool keyed);

/** Whether the receiver is known failed */
bool receiver_failed(void);

/** Recovery cycles since boot */
uint32_t receiver_cycles(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_RECEIVER_H */
