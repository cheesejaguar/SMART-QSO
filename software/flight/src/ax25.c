/**
 * @file ax25.c
 * @brief AX.25 v2.2 address encoding, FCS and UI-frame parsing
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#include "ax25.h"
#include <ctype.h>
#include <string.h>

#define CALL_LEN        6U
#define FCS_LEN         2U
#define FCS_INIT        0xFFFFU
#define FCS_POLY        0x8408U   /* CCITT, reflected */
#define SSID_MASK       0x1EU
#define END_OF_ADDRESS  0x01U

void ax25_encode_address(const char *callsign, uint8_t ssid, uint8_t out[AX25_ADDR_LEN],
                         bool is_last)
{
    size_t len = (callsign != NULL) ? strlen(callsign) : 0U;

    /* Call sign padded with spaces, each character shifted left by one */
    for (size_t i = 0U; i < CALL_LEN; i++) {
        char c = (i < len) ? (char)toupper((unsigned char)callsign[i]) : ' ';
        out[i] = (uint8_t)((uint8_t)c << 1);
    }

    /* SSID byte: reserved bits 5-6 set, SSID in bits 1-4, bit 0 ends the field */
    out[CALL_LEN] = (uint8_t)(0x60U | ((ssid & 0x0FU) << 1) | (is_last ? END_OF_ADDRESS : 0U));
}

bool ax25_address_matches(const uint8_t *addr, const char *callsign, uint8_t ssid)
{
    uint8_t expected[AX25_ADDR_LEN];

    if ((addr == NULL) || (callsign == NULL) || (strlen(callsign) > CALL_LEN)) {
        return false;
    }
    ax25_encode_address(callsign, ssid, expected, false);
    return (memcmp(addr, expected, CALL_LEN) == 0) &&
           ((addr[CALL_LEN] & SSID_MASK) == (expected[CALL_LEN] & SSID_MASK));
}

uint16_t ax25_fcs(const uint8_t *data, size_t len)
{
    uint16_t crc = FCS_INIT;

    for (size_t i = 0U; i < len; i++) {
        crc ^= data[i];
        for (uint32_t bit = 0U; bit < 8U; bit++) {
            crc = ((crc & 0x0001U) != 0U) ? (uint16_t)((crc >> 1) ^ FCS_POLY) : (uint16_t)(crc >> 1);
        }
    }

    return (uint16_t)(crc ^ 0xFFFFU);
}

Ax25Status_t ax25_parse_ui(const uint8_t *frame, size_t len, Ax25UiView_t *view)
{
    const size_t minimum = (2U * AX25_ADDR_LEN) + 2U + FCS_LEN;

    if ((frame == NULL) || (view == NULL) || (len < minimum)) {
        return AX25_ERR_FORMAT;
    }

    uint16_t received = (uint16_t)(frame[len - 2U] | ((uint16_t)frame[len - 1U] << 8));
    if (ax25_fcs(frame, len - FCS_LEN) != received) {
        return AX25_ERR_FCS;
    }

    /* Address field: 7-byte entries until one has the end bit set */
    size_t count = 0U;
    for (size_t i = 0U; (i < AX25_MAX_ADDRS) && (count == 0U); i++) {
        size_t base = i * AX25_ADDR_LEN;
        if ((base + AX25_ADDR_LEN + 2U + FCS_LEN) > len) {
            return AX25_ERR_FORMAT;
        }
        for (size_t c = 0U; c < CALL_LEN; c++) {
            if ((frame[base + c] & END_OF_ADDRESS) != 0U) {
                return AX25_ERR_FORMAT;  /* call sign bytes never have bit 0 set */
            }
        }
        if ((frame[base + CALL_LEN] & END_OF_ADDRESS) != 0U) {
            count = i + 1U;
        }
    }
    if (count < 2U) {
        return AX25_ERR_FORMAT;
    }

    size_t ctrl = count * AX25_ADDR_LEN;
    if ((frame[ctrl] != AX25_CTRL_UI) || (frame[ctrl + 1U] != AX25_PID_NO_L3)) {
        return AX25_ERR_NOT_UI;
    }

    view->dest = &frame[0];
    view->src = &frame[AX25_ADDR_LEN];
    view->info = &frame[ctrl + 2U];
    view->info_len = len - (ctrl + 2U) - FCS_LEN;
    return AX25_OK;
}
