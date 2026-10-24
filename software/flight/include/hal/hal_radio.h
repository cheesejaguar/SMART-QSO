/**
 * @file hal_radio.h
 * @brief Telecommand receiver HAL (435 MHz uplink)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Contract: hal_radio_receive() delivers one complete AX.25 frame per call,
 * from the first address byte through the 2-byte FCS, with HDLC flags and bit
 * stuffing removed. The FCS is delivered even if the transceiver has checked
 * it; the OBC checks it again (software/rust/sqso-ffi/src/uplink.rs).
 *
 * Only the host simulation implements this today (src/hal/hal_sim.c). The
 * target driver depends on the transceiver selection (SQSO-ICD-005).
 */

#ifndef SMART_QSO_HAL_RADIO_H
#define SMART_QSO_HAL_RADIO_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stddef.h>
#include <stdint.h>

/** Largest frame the receiver delivers */
#define HAL_RADIO_MAX_FRAME     330U

/**
 * @brief Start the receiver
 */
SmartQsoResult_t hal_radio_init(void);

/**
 * @brief Take one received frame, if any
 *
 * @param[out] frame   Destination buffer
 * @param[in]  max_len Buffer size
 * @param[out] len     Frame length
 * @return SMART_QSO_OK with a frame; SMART_QSO_ERROR_TIMEOUT if none is waiting
 */
SmartQsoResult_t hal_radio_receive(uint8_t *frame, size_t max_len, size_t *len);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_RADIO_H */
