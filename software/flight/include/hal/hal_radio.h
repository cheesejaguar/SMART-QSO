/**
 * @file hal_radio.h
 * @brief Radio HAL: the KISS TNC of the 435 MHz telecommand receiver and the
 *        145.8 MHz downlink transmitter
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * One KISS TNC on the radio UART (USART2, 9600 baud) serves both directions
 * (SQSO-CRL-001 CR-2026-005; the transceiver itself is not selected,
 * SQSO-ICD-005).
 *
 * Receive: hal_radio_receive() delivers one complete AX.25 frame per call,
 * from the first address byte through the 2-byte FCS, with HDLC flags and bit
 * stuffing removed. The TNC checks the FCS and leaves it off its KISS frame,
 * so the target HAL appends the FCS of the frame it received; the OBC checks
 * it again (software/rust/sqso-ffi/src/uplink.rs).
 *
 * Transmit: hal_radio_transmit() takes one AX.25 frame without its FCS (the
 * TNC adds it) and writes the TNC's parameters (KISS TXDELAY, persistence,
 * slot time, TX tail, full duplex) and the frame as a KISS data frame on
 * port 0. It never blocks: if the UART cannot take everything at once,
 * nothing is written. Whether a frame may be sent is decided above it
 * (downlink.h); the PA's power is GPIO_PIN_RF_TX_EN (rf_control.h).
 *
 * Implemented by the host simulation (software/rust/sqso-ffi/src/hal_sim.rs)
 * and the STM32L4R5 firmware (software/rust/firmware/src/radio.rs).
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

/**
 * @brief Hand one frame to the TNC for transmission
 *
 * @param[in] frame AX.25 frame from the first address byte through the
 *                  information field (no flags, no FCS)
 * @param[in] len   1 to HAL_RADIO_MAX_FRAME - 2 bytes
 * @return SMART_QSO_OK written; SMART_QSO_ERROR_BUSY the UART could not take
 *         it all (nothing written); SMART_QSO_ERROR before hal_radio_init();
 *         SMART_QSO_ERROR_INVALID bad length; SMART_QSO_ERROR_NULL_PTR
 */
SmartQsoResult_t hal_radio_transmit(const uint8_t *frame, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HAL_RADIO_H */
