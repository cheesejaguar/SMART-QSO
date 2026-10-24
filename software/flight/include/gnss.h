/**
 * @file gnss.h
 * @brief GNSS receiver NMEA driver (decision D6)
 *
 * Implemented in Rust (software/rust/sqso-ffi/src/gnss.rs, logic in
 * software/rust/sqso-fsw/src/gnss.rs).
 *
 * Parses the receiver's UART output, one NMEA 0183 sentence per line
 * ("$<body>*<XX>\r\n", XX = XOR of the body). Two sentences matter:
 *
 *   $--GGA  fix quality (field 6). 1 (GPS) and 2 (differential) are real
 *           fixes; 0 is none and 6 is dead reckoning, which the piNAV-NG
 *           emits from its DROP propagator after losing lock.
 *   $PSLSP  piNAV-NG state record: GPS time of week [s], GPS week, ECEF
 *           X, Y, Z [m, WGS-84], satellites used, PDOP. Field order is
 *           from the datasheet example (Rev. A/2025); confirm against the
 *           datasheet table before flight.
 *
 * A $PSLSP record is kept as the latest fix only if a GGA with a real fix
 * arrived within GNSS_GGA_MAX_AGE_MS before it and the record is
 * plausible for LEO. DROP output is never used: it drifts about 6 km in
 * 15 minutes and must not reach the ephemeris (SpaceX User's Guide v11
 * §9.5.7.2; docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md §3a).
 *
 * The velocity sentence ($PSLSV) is not parsed: ground orbit
 * determination fits positions only.
 */

#ifndef GNSS_H
#define GNSS_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Longest sentence kept, including "$", "*XX" and CR LF (NMEA allows 82) */
#define GNSS_LINE_MAX           96U

/** A $PSLSP record needs a real-fix GGA no older than this */
#define GNSS_GGA_MAX_AGE_MS     2000U

/** Plausible orbit radius for LEO: about 120 km to 1,120 km altitude */
#define GNSS_LEO_MIN_RADIUS_M   6.5e6
#define GNSS_LEO_MAX_RADIUS_M   7.5e6

/** Fewest satellites for a 3D fix */
#define GNSS_MIN_SATS           4U

typedef enum {
    GNSS_OK = 0,         /**< Sentence used: GGA recorded or fix accepted */
    GNSS_IGNORED,        /**< Valid sentence of a type not used */
    GNSS_REJECTED,       /**< $PSLSP refused: no recent real fix, or implausible */
    GNSS_ERR_FORMAT,     /**< Not a sentence, overlong, or malformed fields */
    GNSS_ERR_CHECKSUM    /**< Checksum mismatch */
} GnssStatus_t;

typedef struct {
    bool     valid;
    uint64_t rx_ms;      /**< Time the record was received (caller's clock) */
    double   tow_s;      /**< GPS time of week */
    uint16_t week;       /**< GPS week as reported */
    double   x_m;        /**< ECEF position, WGS-84 */
    double   y_m;
    double   z_m;
    uint8_t  sats;
    double   pdop;
} GnssFix_t;

typedef struct {
    char      line[GNSS_LINE_MAX];
    size_t    len;
    bool      overflow;
    bool      gga_seen;
    uint8_t   gga_quality;
    uint64_t  gga_ms;
    GnssFix_t fix;
    uint32_t  rejected;  /**< $PSLSP records refused */
} GnssRx_t;

void gnss_rx_init(GnssRx_t *rx);

/**
 * @brief Feed one received byte
 * @param now_ms Receive time (any monotonic millisecond clock)
 * @param status Set when a line completes
 * @return true when a line completed (status is then valid)
 */
bool gnss_rx_feed(GnssRx_t *rx, char c, uint64_t now_ms, GnssStatus_t *status);

/**
 * @brief Check "$<body>*XX" (no line ending) against its checksum
 */
bool gnss_nmea_checksum_ok(const char *line, size_t len);

/**
 * @brief Latest accepted fix, if no older than max_age_ms at now_ms
 */
bool gnss_latest_fix(const GnssRx_t *rx, uint64_t now_ms, uint32_t max_age_ms, GnssFix_t *out);

#ifdef __cplusplus
}
#endif

#endif /* GNSS_H */
