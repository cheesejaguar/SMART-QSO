/**
 * @file contact_monitor.h
 * @brief Loss-of-contact transmit inhibit
 *
 * Implemented in Rust (software/rust/sqso-fsw/src/contact_monitor.rs,
 * software/rust/sqso-ffi/src/contact_monitor.rs).
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * If no authenticated telecommand arrives for the contact timeout (N days),
 * every transmitter is inhibited; the receiver stays on, and the next
 * authenticated telecommand lifts the inhibit (SQSO-ICD-013 Section 6.2).
 *
 * There is no reset-proof clock, so the monitor keeps an accumulated
 * seconds-without-contact counter, advanced by contact_monitor_tick() and
 * persisted every CONTACT_PERSIST_INTERVAL_S. Every error makes the inhibit
 * come earlier, never later:
 *  - each boot that finds a record adds CONTACT_PERSIST_INTERVAL_S (time that
 *    may have passed unrecorded), so a reset loop still reaches the inhibit;
 *  - a corrupt counter record counts as timed out (inhibited until contact).
 *
 * N is a persisted parameter, settable by telecommand within
 * CONTACT_TIMEOUT_MIN_DAYS..CONTACT_TIMEOUT_MAX_DAYS. The default is 30 days
 * (decision D3). End-of-mission passivation after a further M days is not
 * implemented; it is a question for the FCC consultation.
 *
 * @requirement SYS-205 Cease transmissions after N days without an authenticated telecommand
 */

#ifndef SMART_QSO_CONTACT_MONITOR_H
#define SMART_QSO_CONTACT_MONITOR_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include "nvm_record.h"
#include <stdbool.h>
#include <stdint.h>

/** Default contact timeout in days (decision D3): 30 days at launch, so a late
 *  first contact does not find the spacecraft silent; operations set 14 days
 *  by SET_CONTACT_TIMEOUT once commissioning is complete
 *  (docs/funding_audit/research/D3_LOSS_OF_CONTACT.md) */
#define CONTACT_TIMEOUT_DEFAULT_DAYS    30U

/** Allowed range for the contact timeout (decision D3) */
#define CONTACT_TIMEOUT_MIN_DAYS        1U
#define CONTACT_TIMEOUT_MAX_DAYS        90U

/** Counter persistence interval, also the boot penalty */
#define CONTACT_PERSIST_INTERVAL_S      60U

/** Layout of FLASH_REGION_CONTACT */
#define CONTACT_NVM_COUNTER             0U
#define CONTACT_NVM_TIMEOUT             NVM_RECORD_FOOTPRINT

/**
 * @brief Load the counter and timeout, apply the boot penalty, persist, and
 *        set the inhibit on rf_control
 *
 * Call at boot before anything can transmit (it may run before
 * rf_control_init()).
 */
SmartQsoResult_t contact_monitor_init(void);

/**
 * @brief Advance the counter by elapsed time; inhibit at the timeout
 *
 * @param[in] elapsed_ms Time since the previous tick
 */
void contact_monitor_tick(uint32_t elapsed_ms);

/**
 * @brief An authenticated telecommand was received: restart the count and
 *        lift the inhibit (never touches the TX-OFF latch)
 */
void contact_monitor_contact(void);

/**
 * @brief Set the contact timeout and persist it
 *
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID outside the allowed range;
 *         SMART_QSO_ERROR_IO if not persisted (applied until reset)
 */
SmartQsoResult_t contact_monitor_set_timeout_days(uint16_t days);

/**
 * @brief Current contact timeout in days
 */
uint16_t contact_monitor_get_timeout_days(void);

/**
 * @brief Accumulated seconds without contact
 */
uint32_t contact_monitor_seconds_since_contact(void);

/**
 * @brief Whether the loss-of-contact inhibit is in force
 */
bool contact_monitor_inhibited(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_CONTACT_MONITOR_H */
