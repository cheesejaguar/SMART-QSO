/**
 * @file jetson_link.c
 * @brief OBC side of the OBC-Jetson line protocol
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#include "jetson_link.h"
#include <stdio.h>
#include <string.h>

#define CRC_INIT    0xFFFFU
#define CRC_POLY    0x1021U
#define CRC_FIELD   5U      /* ",XXXX" */

static const char HEX[] = "0123456789ABCDEF";

uint16_t jetson_crc16(const uint8_t *data, size_t len)
{
    uint16_t crc = CRC_INIT;

    for (size_t i = 0U; i < len; i++) {
        crc = (uint16_t)(crc ^ ((uint32_t)data[i] << 8));
        for (uint32_t bit = 0U; bit < 8U; bit++) {
            uint32_t shifted = (uint32_t)crc << 1;
            crc = (uint16_t)(((crc & 0x8000U) != 0U) ? (shifted ^ CRC_POLY) : shifted);
        }
    }
    return crc;
}

static bool printable(const char *s, size_t len)
{
    for (size_t i = 0U; i < len; i++) {
        if ((s[i] < 0x20) || (s[i] > 0x7E)) {
            return false;
        }
    }
    return true;
}

size_t jetson_link_seal(char *buf, size_t len, size_t cap)
{
    size_t line_len = len + CRC_FIELD + 1U;  /* + ",XXXX" + "\n" */

    if ((buf == NULL) || (len == 0U) || (line_len > cap) || (line_len > JETSON_LINE_MAX) ||
        !printable(buf, len)) {
        return 0U;
    }

    uint16_t crc = jetson_crc16((const uint8_t *)buf, len);
    buf[len] = ',';
    buf[len + 1U] = HEX[(crc >> 12) & 0xFU];
    buf[len + 2U] = HEX[(crc >> 8) & 0xFU];
    buf[len + 3U] = HEX[(crc >> 4) & 0xFU];
    buf[len + 4U] = HEX[crc & 0xFU];
    buf[len + 5U] = '\n';
    if ((len + 6U) < cap) {
        buf[len + 6U] = '\0';
    }
    return len + 6U;
}

static size_t seal_formatted(char *out, size_t cap, int written)
{
    if ((written <= 0) || ((size_t)written >= cap)) {
        return 0U;
    }
    return jetson_link_seal(out, (size_t)written, cap);
}

size_t jetson_link_beacon_req(uint16_t seq, uint8_t max_len, char *out, size_t cap)
{
    if (out == NULL) {
        return 0U;
    }
    return seal_formatted(out, cap, snprintf(out, cap, "BEACON_REQ,%u,%u", (unsigned)seq, (unsigned)max_len));
}

size_t jetson_link_status_req(char *out, size_t cap)
{
    if (out == NULL) {
        return 0U;
    }
    return seal_formatted(out, cap, snprintf(out, cap, "STATUS_REQ"));
}

size_t jetson_link_shutdown(const char *reason, char *out, size_t cap)
{
    if ((out == NULL) || (reason == NULL) || (strchr(reason, ',') != NULL)) {
        return 0U;
    }
    return seal_formatted(out, cap, snprintf(out, cap, "SHUTDOWN,%s", reason));
}

/** Parse an unsigned decimal field [s, s+len) no larger than max */
static bool parse_uint(const char *s, size_t len, uint32_t max, uint32_t *value)
{
    uint32_t v = 0U;

    if ((len == 0U) || (len > 10U)) {
        return false;
    }
    for (size_t i = 0U; i < len; i++) {
        if ((s[i] < '0') || (s[i] > '9')) {
            return false;
        }
        uint32_t digit = (uint32_t)(s[i] - '0');
        if ((digit > max) || (v > ((max - digit) / 10U))) {
            return false;
        }
        v = (v * 10U) + digit;
    }
    *value = v;
    return true;
}

static bool parse_int(const char *s, size_t len, int32_t min, int32_t max, int32_t *value)
{
    uint32_t magnitude = 0U;
    bool negative = (len > 0U) && (s[0] == '-');
    uint32_t limit = negative ? (uint32_t)(-(int64_t)min) : (uint32_t)max;

    if (!parse_uint(negative ? &s[1] : s, negative ? (len - 1U) : len, limit, &magnitude)) {
        return false;
    }
    *value = negative ? -(int32_t)magnitude : (int32_t)magnitude;
    return true;
}

static int hex_digit(char c)
{
    if ((c >= '0') && (c <= '9')) {
        return c - '0';
    }
    if ((c >= 'A') && (c <= 'F')) {
        return (c - 'A') + 10;
    }
    return -1;
}

/** Split body [0, len) at commas into up to max_fields fields */
static size_t split(const char *body, size_t len, const char **start, size_t *flen, size_t max_fields)
{
    size_t n = 0U;
    size_t field_start = 0U;

    for (size_t i = 0U; i <= len; i++) {
        if ((i == len) || (body[i] == ',')) {
            if (n == max_fields) {
                return max_fields + 1U;  /* too many */
            }
            start[n] = &body[field_start];
            flen[n] = i - field_start;
            n++;
            field_start = i + 1U;
        }
    }
    return n;
}

/** Check framing and CRC; on success *body_len is the length before ",XXXX" */
static JetsonStatus_t check_frame(const char *line, size_t len, size_t *body_len)
{
    if ((len < (CRC_FIELD + 1U)) || (len >= JETSON_LINE_MAX) || !printable(line, len) ||
        (line[len - CRC_FIELD] != ',')) {
        return JETSON_ERR_FORMAT;
    }

    uint16_t received = 0U;
    for (size_t i = len - 4U; i < len; i++) {
        int d = hex_digit(line[i]);
        if (d < 0) {
            return JETSON_ERR_FORMAT;
        }
        received = (uint16_t)((received << 4) | (uint16_t)d);
    }
    *body_len = len - CRC_FIELD;
    return (jetson_crc16((const uint8_t *)line, *body_len) == received) ? JETSON_OK : JETSON_ERR_CRC;
}

/** BEACON_RSP,<seq>,<text>: the text may contain commas */
static JetsonStatus_t parse_beacon_rsp(const char *rest, size_t rest_len, JetsonMsg_t *msg)
{
    const char *seq_end = memchr(rest, ',', rest_len);
    uint32_t seq = 0U;
    if ((seq_end == NULL) || !parse_uint(rest, (size_t)(seq_end - rest), 0xFFFFU, &seq)) {
        return JETSON_ERR_FORMAT;
    }
    size_t text_len = rest_len - (size_t)(seq_end - rest) - 1U;
    if ((text_len == 0U) || (text_len > JETSON_TEXT_MAX)) {
        return JETSON_ERR_FORMAT;
    }
    msg->type = JETSON_MSG_BEACON_RSP;
    msg->seq = (uint16_t)seq;
    (void)memcpy(msg->text, seq_end + 1, text_len);
    msg->text[text_len] = '\0';
    msg->text_len = text_len;
    return JETSON_OK;
}

/** STATUS,<health>,<temp_c>,<power_mw>,<mem_pct>,<uptime_s> */
static JetsonStatus_t parse_status(const char *rest, size_t rest_len, JetsonMsg_t *msg)
{
    const char *f[5];
    size_t fl[5];
    uint32_t health = 0U;
    uint32_t power = 0U;
    uint32_t mem = 0U;
    uint32_t uptime = 0U;
    int32_t temp = 0;
    if ((rest_len == 0U) || (split(rest, rest_len, f, fl, 5U) != 5U) ||
        !parse_uint(f[0], fl[0], 3U, &health) ||
        !parse_int(f[1], fl[1], -128, 200, &temp) ||
        !parse_uint(f[2], fl[2], 100000U, &power) ||
        !parse_uint(f[3], fl[3], 100U, &mem) ||
        !parse_uint(f[4], fl[4], 0xFFFFFFFFU, &uptime)) {
        return JETSON_ERR_FORMAT;
    }
    msg->type = JETSON_MSG_STATUS;
    msg->health = (uint8_t)health;
    msg->temp_c = temp;
    msg->power_mw = power;
    msg->mem_pct = (uint8_t)mem;
    msg->uptime_s = uptime;
    return JETSON_OK;
}

/** ACK,<what> or NACK,<reason>: one field, no commas */
static JetsonStatus_t parse_ack(bool ack, const char *rest, size_t rest_len, JetsonMsg_t *msg)
{
    if ((rest_len == 0U) || (rest_len > JETSON_TEXT_MAX) || (memchr(rest, ',', rest_len) != NULL)) {
        return JETSON_ERR_FORMAT;
    }
    msg->type = ack ? JETSON_MSG_ACK : JETSON_MSG_NACK;
    (void)memcpy(msg->text, rest, rest_len);
    msg->text[rest_len] = '\0';
    msg->text_len = rest_len;
    return JETSON_OK;
}

static bool type_is(const char *line, size_t type_len, const char *name)
{
    return (type_len == strlen(name)) && (memcmp(line, name, type_len) == 0);
}

JetsonStatus_t jetson_link_parse(const char *line, size_t len, JetsonMsg_t *msg)
{
    size_t body_len = 0U;

    if ((line == NULL) || (msg == NULL) || (len == 0U)) {
        return JETSON_ERR_FORMAT;
    }
    if (line[len - 1U] == '\n') {
        len--;
    }
    JetsonStatus_t status = check_frame(line, len, &body_len);
    if (status != JETSON_OK) {
        return status;
    }

    (void)memset(msg, 0, sizeof(*msg));
    const char *type_end = memchr(line, ',', body_len);
    size_t type_len = (type_end != NULL) ? (size_t)(type_end - line) : body_len;
    const char *rest = (type_end != NULL) ? (type_end + 1) : &line[body_len];
    size_t rest_len = (type_end != NULL) ? (body_len - type_len - 1U) : 0U;

    if (type_is(line, type_len, "BEACON_RSP")) {
        return parse_beacon_rsp(rest, rest_len, msg);
    }
    if (type_is(line, type_len, "STATUS")) {
        return parse_status(rest, rest_len, msg);
    }
    if (type_is(line, type_len, "ACK") || type_is(line, type_len, "NACK")) {
        return parse_ack(type_is(line, type_len, "ACK"), rest, rest_len, msg);
    }
    return JETSON_ERR_TYPE;
}

void jetson_rx_init(JetsonRx_t *rx)
{
    if (rx != NULL) {
        rx->len = 0U;
        rx->overflow = false;
    }
}

bool jetson_rx_feed(JetsonRx_t *rx, char c, JetsonMsg_t *msg, JetsonStatus_t *status)
{
    if ((rx == NULL) || (msg == NULL) || (status == NULL)) {
        return false;
    }

    if (c == '\n') {
        *status = rx->overflow ? JETSON_ERR_FORMAT : jetson_link_parse(rx->line, rx->len, msg);
        rx->len = 0U;
        rx->overflow = false;
        return true;
    }
    if (rx->overflow) {
        return false;
    }
    if (rx->len >= (sizeof(rx->line) - 1U)) {
        rx->overflow = true;  /* drop until the newline */
        return false;
    }
    rx->line[rx->len] = c;
    rx->len++;
    return false;
}
