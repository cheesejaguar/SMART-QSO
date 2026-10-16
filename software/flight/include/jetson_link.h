/**
 * @file jetson_link.h
 * @brief OBC side of the OBC-Jetson line protocol
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Every message is one ASCII line (docs/interfaces/ICD_OBC_JETSON.md §4):
 *
 *     <TYPE>,<fields...>,<CRC>\n
 *
 * Printable ASCII (0x20-0x7E) only; CRC is four upper-case hex digits of
 * CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF, no reflection, no final
 * XOR) over every byte before the last comma. A BEACON_RSP text may contain
 * commas: it runs from after the sequence number to the last comma.
 *
 * OBC -> Jetson: TELEMETRY, BEACON_REQ, STATUS_REQ, SHUTDOWN
 * Jetson -> OBC: BEACON_RSP, STATUS, ACK, NACK
 */

#ifndef SMART_QSO_JETSON_LINK_H
#define SMART_QSO_JETSON_LINK_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/** Longest line including the CRC field and the newline */
#define JETSON_LINE_MAX     512U

/** Longest beacon text the OBC accepts (the beacon's free text space) */
#define JETSON_TEXT_MAX     176U

/** Messages the OBC receives */
typedef enum {
    JETSON_MSG_BEACON_RSP = 0,
    JETSON_MSG_STATUS     = 1,
    JETSON_MSG_ACK        = 2,
    JETSON_MSG_NACK       = 3
} JetsonMsgType_t;

/** Parse outcome */
typedef enum {
    JETSON_OK         = 0,
    JETSON_ERR_FORMAT = 1, /**< Not a well-formed line or a field out of range */
    JETSON_ERR_CRC    = 2, /**< CRC mismatch */
    JETSON_ERR_TYPE   = 3  /**< Not a Jetson -> OBC message */
} JetsonStatus_t;

/** A parsed Jetson -> OBC message */
typedef struct {
    JetsonMsgType_t type;
    uint16_t seq;                      /**< BEACON_RSP */
    char text[JETSON_TEXT_MAX + 1U];   /**< BEACON_RSP text; ACK/NACK field */
    size_t text_len;
    uint8_t health;                    /**< STATUS: 0 OK .. 3 CRITICAL */
    int32_t temp_c;                    /**< STATUS: module temperature */
    uint32_t power_mw;                 /**< STATUS: input power */
    uint8_t mem_pct;                   /**< STATUS: memory used */
    uint32_t uptime_s;                 /**< STATUS: uptime */
} JetsonMsg_t;

/** Receive accumulator: bytes in, complete lines out */
typedef struct {
    char line[JETSON_LINE_MAX];
    size_t len;
    bool overflow;       /**< Discarding an overlong line up to its newline */
} JetsonRx_t;

/**
 * @brief CRC-16/CCITT-FALSE (check value 0x29B1)
 */
uint16_t jetson_crc16(const uint8_t *data, size_t len);

/**
 * @brief Append ",<CRC>\n" to the message in buf
 *
 * @param[in,out] buf Message text (no CRC, no newline)
 * @param[in]     len Message length
 * @param[in]     cap Buffer capacity
 * @return Line length, or 0 if it would not fit or is not printable ASCII
 */
size_t jetson_link_seal(char *buf, size_t len, size_t cap);

/** @brief BEACON_REQ,<seq>,<max_len> */
size_t jetson_link_beacon_req(uint16_t seq, uint8_t max_len, char *out, size_t cap);

/** @brief STATUS_REQ */
size_t jetson_link_status_req(char *out, size_t cap);

/** @brief SHUTDOWN,<reason> */
size_t jetson_link_shutdown(const char *reason, char *out, size_t cap);

/**
 * @brief Check and parse one received line (trailing newline optional)
 */
JetsonStatus_t jetson_link_parse(const char *line, size_t len, JetsonMsg_t *msg);

/** @brief Reset a receive accumulator */
void jetson_rx_init(JetsonRx_t *rx);

/**
 * @brief Feed one received byte
 *
 * @return true when a line ended; *status then holds its parse result (an
 *         overlong line reports JETSON_ERR_FORMAT once) and *msg its content
 */
bool jetson_rx_feed(JetsonRx_t *rx, char c, JetsonMsg_t *msg, JetsonStatus_t *status);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_JETSON_LINK_H */
