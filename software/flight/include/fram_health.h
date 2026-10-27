/**
 * @file fram_health.h
 * @brief The FRAM's health and the degraded mode without it (RID SMA-B10)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The one-minute check of the critical persisted state (AV-07, SRS-F133)
 * settles an unreadable record toward the safe state: with the FRAM dead
 * that would silence the transmitter for good. The check's reads are
 * counted: after FRAM_FAILURES_TO_DECLARE consecutive failed checks the
 * FRAM is declared failed (HK1 FR=1) and the safe defaults stay; only then
 * does FRAM_BYPASS (argument "FRM!") enter the degraded mode (FR=2): the
 * RAM copies of the TX-OFF latch and the operations record, each under a
 * CRC, are authoritative; the check verifies the CRCs (a mismatch settles
 * toward TX-OFF and the hold) and writes the FRAM, best effort. TX-OFF by
 * command still latches at once. The mode ends when FRAM_READS_TO_RECOVER
 * consecutive checks read back what was written, and at every reset.
 * Implemented in Rust (sqso-ffi/src/fram_health.rs, sqso-fsw/src/fram_health.rs).
 */

#ifndef SMART_QSO_FRAM_HEALTH_H
#define SMART_QSO_FRAM_HEALTH_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

/** Consecutive failed one-minute checks that declare the FRAM failed */
#define FRAM_FAILURES_TO_DECLARE 3U
/** Consecutive good checks in the degraded mode that end it */
#define FRAM_READS_TO_RECOVER 3U

/** HK1 FR states */
#define FRAM_STATE_OK       0U
#define FRAM_STATE_FAILED   1U
#define FRAM_STATE_DEGRADED 2U

/** @brief Boot: healthy, not degraded */
void fram_health_init(void);

/**
 * @brief The one-minute check of the critical persisted state: the TX-OFF
 *        latch, the operations record and the passivation record
 */
void fram_health_check(void);

/**
 * @brief FRAM_BYPASS: enter the degraded mode
 * @return SMART_QSO_OK; SMART_QSO_ERROR unless the FRAM has been declared
 *         failed; SMART_QSO_ERROR_INVALID for an argument other than "FRM!"
 */
int32_t fram_bypass(const uint8_t *arg, size_t len);

/**
 * @brief HK1 FR: the state (FRAM_STATE_*) and the failed checks since boot
 * @param state Out (may be NULL)
 * @param failures Out (may be NULL)
 */
int32_t fram_health_status(uint8_t *state, uint32_t *failures);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_FRAM_HEALTH_H */
