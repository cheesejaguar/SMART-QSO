/**
 * @file gnss.c
 * @brief GNSS receiver NMEA driver (decision D6)
 *
 * See gnss.h for the sentences used and the acceptance rule.
 */

#include "gnss.h"

#include <string.h>
#include <stdint.h>

/** GPS time of week wraps at one week */
#define GNSS_TOW_LIMIT_S        604800.0

/** GGA quality: 1 GPS fix, 2 differential fix (0 none, 6 dead reckoning) */
#define GNSS_GGA_GPS            1U
#define GNSS_GGA_DGPS           2U

#define GNSS_MAX_FIELDS         16U
#define GNSS_GGA_QUALITY_FIELD  6U
#define GNSS_PSLSP_FIELDS       8U   /* type + 7 values */

/*===========================================================================*/
/* Field parsing (ASCII only; no locale, exponents, inf or nan)              */
/*===========================================================================*/

static int hex_value(char c)
{
    if ((c >= '0') && (c <= '9')) {
        return c - '0';
    }
    if ((c >= 'A') && (c <= 'F')) {
        return (c - 'A') + 10;
    }
    if ((c >= 'a') && (c <= 'f')) {
        return (c - 'a') + 10;
    }
    return -1;
}

static bool parse_uint(const char *s, uint32_t max, uint32_t *out)
{
    uint32_t value = 0U;

    if (*s == '\0') {
        return false;
    }
    for (; *s != '\0'; s++) {
        if ((*s < '0') || (*s > '9')) {
            return false;
        }
        uint32_t digit = (uint32_t)(*s - '0');
        if ((value > (max / 10U)) || ((value * 10U) > (max - digit))) {
            return false;
        }
        value = (value * 10U) + digit;
    }
    *out = value;
    return true;
}

/** [-]digits[.digits] */
static bool parse_decimal(const char *s, double *out)
{
    bool negative = false;
    bool digits = false;
    double value = 0.0;
    double scale = 1.0;

    if (*s == '-') {
        negative = true;
        s++;
    }
    for (; (*s >= '0') && (*s <= '9'); s++) {
        value = (value * 10.0) + (double)(*s - '0');
        digits = true;
    }
    if (*s == '.') {
        s++;
        for (; (*s >= '0') && (*s <= '9'); s++) {
            scale /= 10.0;
            value += (double)(*s - '0') * scale;
            digits = true;
        }
    }
    if (!digits || (*s != '\0')) {
        return false;
    }
    *out = negative ? -value : value;
    return true;
}

/** Split body at commas, in place; returns the field count, 0 if too many */
static size_t split_fields(char *body, char **fields)
{
    size_t n = 0U;
    fields[n++] = body;
    for (char *p = body; *p != '\0'; p++) {
        if (*p == ',') {
            *p = '\0';
            if (n >= GNSS_MAX_FIELDS) {
                return 0U;
            }
            fields[n++] = p + 1;
        }
    }
    return n;
}

/*===========================================================================*/
/* Sentences                                                                  */
/*===========================================================================*/

static bool is_gga(const char *type)
{
    return (strlen(type) == 5U) && (memcmp(&type[2], "GGA", 3U) == 0);
}

static GnssStatus_t handle_gga(GnssRx_t *rx, char *const *fields, size_t count, uint64_t now_ms)
{
    uint32_t quality = 0U;

    if ((count <= GNSS_GGA_QUALITY_FIELD) ||
        !parse_uint(fields[GNSS_GGA_QUALITY_FIELD], 9U, &quality)) {
        return GNSS_ERR_FORMAT;
    }
    rx->gga_seen = true;
    rx->gga_quality = (uint8_t)quality;
    rx->gga_ms = now_ms;
    return GNSS_OK;
}

static bool recent_real_fix(const GnssRx_t *rx, uint64_t now_ms)
{
    return rx->gga_seen &&
           ((rx->gga_quality == GNSS_GGA_GPS) || (rx->gga_quality == GNSS_GGA_DGPS)) &&
           (now_ms >= rx->gga_ms) && ((now_ms - rx->gga_ms) <= GNSS_GGA_MAX_AGE_MS);
}

static bool plausible(const GnssFix_t *f)
{
    double r2 = (f->x_m * f->x_m) + (f->y_m * f->y_m) + (f->z_m * f->z_m);
    return (f->tow_s >= 0.0) && (f->tow_s < GNSS_TOW_LIMIT_S) &&
           (f->sats >= GNSS_MIN_SATS) && (f->pdop > 0.0) &&
           (r2 >= (GNSS_LEO_MIN_RADIUS_M * GNSS_LEO_MIN_RADIUS_M)) &&
           (r2 <= (GNSS_LEO_MAX_RADIUS_M * GNSS_LEO_MAX_RADIUS_M));
}

static GnssStatus_t handle_pslsp(GnssRx_t *rx, char *const *fields, size_t count, uint64_t now_ms)
{
    GnssFix_t f;
    uint32_t week = 0U;
    uint32_t sats = 0U;

    (void)memset(&f, 0, sizeof(f));
    if ((count != GNSS_PSLSP_FIELDS) ||
        !parse_decimal(fields[1], &f.tow_s) ||
        !parse_uint(fields[2], UINT16_MAX, &week) ||
        !parse_decimal(fields[3], &f.x_m) ||
        !parse_decimal(fields[4], &f.y_m) ||
        !parse_decimal(fields[5], &f.z_m) ||
        !parse_uint(fields[6], UINT8_MAX, &sats) ||
        !parse_decimal(fields[7], &f.pdop)) {
        return GNSS_ERR_FORMAT;
    }
    f.week = (uint16_t)week;
    f.sats = (uint8_t)sats;

    if (!recent_real_fix(rx, now_ms) || !plausible(&f)) {
        rx->rejected++;
        return GNSS_REJECTED;
    }
    f.valid = true;
    f.rx_ms = now_ms;
    rx->fix = f;
    return GNSS_OK;
}

/** One complete line, without its line ending */
static GnssStatus_t process_line(GnssRx_t *rx, size_t len, uint64_t now_ms)
{
    char body[GNSS_LINE_MAX];
    char *fields[GNSS_MAX_FIELDS];

    for (size_t i = 0U; i < len; i++) {
        if ((rx->line[i] < 0x20) || (rx->line[i] > 0x7E)) {
            return GNSS_ERR_FORMAT;  /* a NUL would not change the checksum */
        }
    }
    if ((len < 4U) || (rx->line[0] != '$') || (rx->line[len - 3U] != '*') ||
        (hex_value(rx->line[len - 2U]) < 0) || (hex_value(rx->line[len - 1U]) < 0)) {
        return GNSS_ERR_FORMAT;
    }
    if (!gnss_nmea_checksum_ok(rx->line, len)) {
        return GNSS_ERR_CHECKSUM;
    }

    size_t body_len = len - 4U;
    (void)memcpy(body, &rx->line[1], body_len);
    body[body_len] = '\0';
    size_t count = split_fields(body, fields);
    if (count == 0U) {
        return GNSS_ERR_FORMAT;
    }
    if (is_gga(fields[0])) {
        return handle_gga(rx, fields, count, now_ms);
    }
    if (strcmp(fields[0], "PSLSP") == 0) {
        return handle_pslsp(rx, fields, count, now_ms);
    }
    return GNSS_IGNORED;
}

/*===========================================================================*/
/* Public                                                                     */
/*===========================================================================*/

void gnss_rx_init(GnssRx_t *rx)
{
    if (rx != NULL) {
        (void)memset(rx, 0, sizeof(*rx));
    }
}

bool gnss_rx_feed(GnssRx_t *rx, char c, uint64_t now_ms, GnssStatus_t *status)
{
    if ((rx == NULL) || (status == NULL) || (c == '\r')) {
        return false;
    }
    if (c != '\n') {
        if (rx->len < (GNSS_LINE_MAX - 1U)) {
            rx->line[rx->len] = c;
            rx->len++;
        } else {
            rx->overflow = true;
        }
        return false;
    }

    size_t len = rx->len;
    bool overflow = rx->overflow;
    rx->len = 0U;
    rx->overflow = false;
    if (overflow) {
        *status = GNSS_ERR_FORMAT;
        return true;
    }
    if (len == 0U) {
        return false;
    }
    *status = process_line(rx, len, now_ms);
    return true;
}

bool gnss_nmea_checksum_ok(const char *line, size_t len)
{
    if ((line == NULL) || (len < 4U) || (line[0] != '$') || (line[len - 3U] != '*')) {
        return false;
    }
    int hi = hex_value(line[len - 2U]);
    int lo = hex_value(line[len - 1U]);
    if ((hi < 0) || (lo < 0)) {
        return false;
    }
    uint8_t sum = 0U;
    for (size_t i = 1U; i < (len - 3U); i++) {
        sum ^= (uint8_t)line[i];
    }
    return sum == (uint8_t)((hi * 16) + lo);
}

bool gnss_latest_fix(const GnssRx_t *rx, uint64_t now_ms, uint32_t max_age_ms, GnssFix_t *out)
{
    if ((rx == NULL) || (out == NULL) || !rx->fix.valid || (now_ms < rx->fix.rx_ms) ||
        ((now_ms - rx->fix.rx_ms) > max_age_ms)) {
        return false;
    }
    *out = rx->fix;
    return true;
}
