/**
 * @file ax25.h
 * @brief AX.25 v2.2 address encoding, FCS and UI-frame parsing
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Frames handled here run from the first address byte through the 2-byte FCS
 * (little-endian); HDLC flags and bit stuffing belong to the modem.
 */

#ifndef SMART_QSO_AX25_H
#define SMART_QSO_AX25_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define AX25_FLAG               0x7E
#define AX25_CTRL_UI            0x03
#define AX25_PID_NO_L3          0xF0
#define AX25_ADDR_LEN           7
#define AX25_MAX_ADDRS          10U   /**< Destination, source and up to 8 digipeaters */
#define AX25_MAX_FRAME_LEN      330

/** Parse outcome */
typedef enum {
    AX25_OK         = 0, /**< Valid UI frame */
    AX25_ERR_FORMAT = 1, /**< Too short or malformed address field */
    AX25_ERR_FCS    = 2, /**< Frame check sequence mismatch */
    AX25_ERR_NOT_UI = 3  /**< Not a UI frame with PID 0xF0 */
} Ax25Status_t;

/** View of a parsed UI frame (pointers into the frame buffer) */
typedef struct {
    const uint8_t *dest;     /**< Destination address (7 bytes) */
    const uint8_t *src;      /**< Source address (7 bytes) */
    const uint8_t *info;     /**< Information field */
    size_t info_len;         /**< Information field length */
} Ax25UiView_t;

/**
 * @brief Encode a call sign and SSID as a 7-byte address
 */
void ax25_encode_address(const char *callsign, uint8_t ssid, uint8_t out[AX25_ADDR_LEN],
                         bool is_last);

/**
 * @brief Whether an address field names the given call sign and SSID
 */
bool ax25_address_matches(const uint8_t *addr, const char *callsign, uint8_t ssid);

/**
 * @brief AX.25 frame check sequence (CRC-16/X.25)
 */
uint16_t ax25_fcs(const uint8_t *data, size_t len);

/**
 * @brief Check the FCS and parse a UI frame with PID 0xF0
 */
Ax25Status_t ax25_parse_ui(const uint8_t *frame, size_t len, Ax25UiView_t *view);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_AX25_H */
