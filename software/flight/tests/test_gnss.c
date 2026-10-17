/**
 * @file test_gnss.c
 * @brief GNSS receiver NMEA driver (decision D6)
 *
 * Golden sentences: the NMEA 0183 GGA reference example (checksum 47) and
 * the $PSLSP example from the SkyFox piNAV-NG datasheet Rev. A/2025
 * (checksum 7F). A position record is accepted only when the receiver's
 * latest GGA says it has a real fix, so dead-reckoned (DROP, quality 6)
 * output never reaches the ephemeris.
 *
 * @note SpaceX Rideshare User's Guide v11 §9.5.7.2 (launch-provider rule): GNSS-based ephemeris
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <stdio.h>
#include <string.h>

#include "gnss.h"

#define GGA_REF     "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n"
#define PSLSP_REF   "$PSLSP,3061.1001020,801,-6602427.515,97187.285,-1721790.821,7,2.6*7F\r\n"

/** Build a sentence with a correct checksum: "$" body "*" XX "\r\n" */
static void sentence(char *out, size_t cap, const char *body)
{
    uint8_t sum = 0U;
    for (const char *p = body; *p != '\0'; p++) {
        sum ^= (uint8_t)*p;
    }
    (void)snprintf(out, cap, "$%s*%02X\r\n", body, (unsigned)sum);
}

/** Feed a whole string; return the status of the last completed line */
static GnssStatus_t feed(GnssRx_t *rx, const char *s, uint64_t now_ms, int *lines)
{
    GnssStatus_t last = GNSS_IGNORED;
    int count = 0;
    for (const char *p = s; *p != '\0'; p++) {
        GnssStatus_t st = GNSS_IGNORED;
        if (gnss_rx_feed(rx, *p, now_ms, &st)) {
            last = st;
            count++;
        }
    }
    if (lines != NULL) {
        *lines = count;
    }
    return last;
}

static void gga(char *out, size_t cap, int quality)
{
    char body[96];
    (void)snprintf(body, sizeof(body),
                   "GPGGA,123519,4807.038,N,01131.000,E,%d,08,0.9,545.4,M,46.9,M,,", quality);
    sentence(out, cap, body);
}

static void test_checksums(void **state)
{
    (void)state;
    assert_true(gnss_nmea_checksum_ok(GGA_REF, strlen(GGA_REF) - 2U));
    assert_true(gnss_nmea_checksum_ok(PSLSP_REF, strlen(PSLSP_REF) - 2U));
    assert_false(gnss_nmea_checksum_ok("$GPGGA,1*00", 11U));
    assert_false(gnss_nmea_checksum_ok("GPGGA,1*31", 10U));   /* no '$' */
    assert_false(gnss_nmea_checksum_ok("$GPGGA,1", 8U));      /* no '*' */
    assert_false(gnss_nmea_checksum_ok("$GPGGA,1*3", 10U));   /* one hex digit */
    assert_false(gnss_nmea_checksum_ok(NULL, 4U));
}

/** A real fix (quality 1) followed by a state record gives a fix */
static void test_fix_from_reference_sentences(void **state)
{
    (void)state;
    GnssRx_t rx;
    GnssFix_t fix;
    int lines = 0;

    gnss_rx_init(&rx);
    assert_int_equal(feed(&rx, GGA_REF, 1000U, &lines), GNSS_OK);
    assert_int_equal(lines, 1);
    assert_int_equal(feed(&rx, PSLSP_REF, 1200U, NULL), GNSS_OK);

    assert_true(gnss_latest_fix(&rx, 1500U, 60000U, &fix));
    assert_float_equal(fix.tow_s, 3061.100102, 1e-6);
    assert_int_equal(fix.week, 801U);
    assert_float_equal(fix.x_m, -6602427.515, 1e-3);
    assert_float_equal(fix.y_m, 97187.285, 1e-3);
    assert_float_equal(fix.z_m, -1721790.821, 1e-3);
    assert_int_equal(fix.sats, 7U);
    assert_float_equal(fix.pdop, 2.6, 1e-9);
    assert_int_equal(fix.rx_ms, 1200U);
}

/** Dead-reckoned (6), no fix (0), or no GGA at all: the record is refused */
static void test_record_needs_real_fix(void **state)
{
    (void)state;
    GnssRx_t rx;
    GnssFix_t fix;
    char line[128];

    gnss_rx_init(&rx);
    assert_int_equal(feed(&rx, PSLSP_REF, 1000U, NULL), GNSS_REJECTED);

    const int bad_quality[] = {0, 6, 7, 8};
    for (size_t i = 0U; i < (sizeof(bad_quality) / sizeof(bad_quality[0])); i++) {
        gga(line, sizeof(line), bad_quality[i]);
        assert_int_equal(feed(&rx, line, 2000U, NULL), GNSS_OK);
        assert_int_equal(feed(&rx, PSLSP_REF, 2100U, NULL), GNSS_REJECTED);
    }
    assert_false(gnss_latest_fix(&rx, 2200U, 60000U, &fix));
    assert_int_equal(rx.rejected, 5U);

    /* Differential fix (2) is a real fix */
    gga(line, sizeof(line), 2);
    assert_int_equal(feed(&rx, line, 3000U, NULL), GNSS_OK);
    assert_int_equal(feed(&rx, PSLSP_REF, 3100U, NULL), GNSS_OK);
    assert_true(gnss_latest_fix(&rx, 3200U, 60000U, &fix));
}

/** The GGA must be recent: a stale quality says nothing about this record */
static void test_gga_must_be_recent(void **state)
{
    (void)state;
    GnssRx_t rx;

    gnss_rx_init(&rx);
    assert_int_equal(feed(&rx, GGA_REF, 1000U, NULL), GNSS_OK);
    assert_int_equal(feed(&rx, PSLSP_REF, 1000U + GNSS_GGA_MAX_AGE_MS, NULL), GNSS_OK);
    assert_int_equal(feed(&rx, PSLSP_REF, 1001U + GNSS_GGA_MAX_AGE_MS, NULL), GNSS_REJECTED);
}

/** Implausible records are refused: too few satellites, no PDOP, not in LEO */
static void test_implausible_records(void **state)
{
    (void)state;
    GnssRx_t rx;
    char line[128];
    const char *bodies[] = {
        "PSLSP,3061.1,801,-6602427.515,97187.285,-1721790.821,3,2.6",     /* 3 satellites */
        "PSLSP,3061.1,801,-6602427.515,97187.285,-1721790.821,7,0.0",     /* no PDOP */
        "PSLSP,3061.1,801,-6378137.0,0.0,0.0,7,2.6",                      /* on the ground */
        "PSLSP,3061.1,801,-42164000.0,0.0,0.0,7,2.6",                     /* geostationary */
        "PSLSP,3061.1,801,-6602427.515,97187.285,7,2.6",                  /* field missing */
        "PSLSP,3061.1,801,-6602427.515,97187.285,-1721790.821,7,2.6,9",   /* extra field */
        "PSLSP,3061.1,80x,-6602427.515,97187.285,-1721790.821,7,2.6",     /* not a number */
        "PSLSP,3061.1,801,-6602427.515,97187.285,-1721790.821,7,2.6x",    /* trailing junk */
        "PSLSP,3061.1,801,-6602427.515,97187.285,-1721790.821,7,1e3",     /* exponent */
        "PSLSP,-1,801,-6602427.515,97187.285,-1721790.821,7,2.6",         /* negative time of week */
        "PSLSP,604800.0,801,-6602427.515,97187.285,-1721790.821,7,2.6",   /* time of week too large */
    };

    gnss_rx_init(&rx);
    for (size_t i = 0U; i < (sizeof(bodies) / sizeof(bodies[0])); i++) {
        assert_int_equal(feed(&rx, GGA_REF, 5000U, NULL), GNSS_OK);
        sentence(line, sizeof(line), bodies[i]);
        GnssStatus_t st = feed(&rx, line, 5000U, NULL);
        if (st == GNSS_OK) {
            print_error("accepted: %s", line);
        }
        assert_int_not_equal(st, GNSS_OK);
    }
}

static void test_framing_errors(void **state)
{
    (void)state;
    GnssRx_t rx;
    int lines = 0;

    gnss_rx_init(&rx);
    /* Bit error */
    char bad[sizeof(PSLSP_REF)];
    (void)memcpy(bad, PSLSP_REF, sizeof(PSLSP_REF));
    bad[10] = '9';
    assert_int_equal(feed(&rx, bad, 1000U, NULL), GNSS_ERR_CHECKSUM);
    /* No checksum */
    assert_int_equal(feed(&rx, "$GPGGA,123519\r\n", 1000U, NULL), GNSS_ERR_FORMAT);
    /* Other sentences are ignored, but still checked */
    char line[128];
    sentence(line, sizeof(line), "GPGSV,3,1,11,03,03,111,00");
    assert_int_equal(feed(&rx, line, 1000U, NULL), GNSS_IGNORED);
    /* Empty lines and bare CR/LF complete nothing */
    assert_int_equal(feed(&rx, "\r\n\n", 1000U, &lines), GNSS_IGNORED);
    assert_int_equal(lines, 0);

    /* An overlong line is dropped up to its newline, then reception resumes */
    for (size_t i = 0U; i < (GNSS_LINE_MAX + 40U); i++) {
        GnssStatus_t st = GNSS_IGNORED;
        assert_false(gnss_rx_feed(&rx, 'A', 1000U, &st));
    }
    assert_int_equal(feed(&rx, "\n", 1000U, &lines), GNSS_ERR_FORMAT);
    assert_int_equal(lines, 1);
    assert_int_equal(feed(&rx, GGA_REF, 1000U, NULL), GNSS_OK);
}

/** Control characters are refused even when they leave the checksum
 *  unchanged (a byte XORed in twice, or a NUL) */
static void test_control_characters(void **state)
{
    (void)state;
    GnssRx_t rx;
    char line[128];
    uint8_t sum = 0U;
    const char body[] = "GPGGA,123519\x01\x01,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,";

    for (size_t i = 0U; i < (sizeof(body) - 1U); i++) {
        sum ^= (uint8_t)body[i];
    }
    (void)snprintf(line, sizeof(line), "$%s*%02X\n", body, (unsigned)sum);
    gnss_rx_init(&rx);
    assert_int_equal(feed(&rx, line, 1000U, NULL), GNSS_ERR_FORMAT);
    assert_false(rx.gga_seen);
}

/** An overlong line is refused whole, even if its first GNSS_LINE_MAX - 1
 *  characters form a valid sentence */
static void test_overlong_line_refused_whole(void **state)
{
    (void)state;
    GnssRx_t rx;
    char body[GNSS_LINE_MAX];
    char line[GNSS_LINE_MAX + 16U];
    const char *head = "GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,";

    /* Pad the last (station id) field so "$" body "*XX" is GNSS_LINE_MAX - 1 long */
    size_t body_len = (GNSS_LINE_MAX - 1U) - 4U;
    (void)memset(body, '0', body_len);
    (void)memcpy(body, head, strlen(head));
    body[body_len] = '\0';
    sentence(line, sizeof(line), body);
    assert_int_equal(strlen(line), (GNSS_LINE_MAX - 1U) + 2U);
    gnss_rx_init(&rx);
    assert_int_equal(feed(&rx, line, 1000U, NULL), GNSS_OK);  /* fits exactly */

    gnss_rx_init(&rx);
    line[GNSS_LINE_MAX - 1U] = '\0';            /* drop CR LF */
    (void)strcat(line, "ZZ\n");                    /* two bytes too many */
    assert_int_equal(feed(&rx, line, 1000U, NULL), GNSS_ERR_FORMAT);
    assert_false(rx.gga_seen);
}

/** A fix older than the caller allows is not offered */
static void test_fix_age(void **state)
{
    (void)state;
    GnssRx_t rx;
    GnssFix_t fix;

    gnss_rx_init(&rx);
    (void)feed(&rx, GGA_REF, 1000U, NULL);
    (void)feed(&rx, PSLSP_REF, 1000U, NULL);
    assert_true(gnss_latest_fix(&rx, 61000U, 60000U, &fix));
    assert_false(gnss_latest_fix(&rx, 61001U, 60000U, &fix));
    assert_false(gnss_latest_fix(&rx, 500U, 60000U, &fix));  /* from the future */
    assert_false(gnss_latest_fix(NULL, 1000U, 60000U, &fix));
}

/** Random input never yields a fix that breaks the acceptance rules */
static void test_fuzz(void **state)
{
    (void)state;
    GnssRx_t rx;
    uint32_t seed = 0x1234567U;
    const char *pieces[] = {GGA_REF, PSLSP_REF, "$PSLSP,", ",", "*7F", "\r\n", "6", "-", ".", "$GPGGA,"};

    gnss_rx_init(&rx);
    for (uint32_t i = 0U; i < 200000U; i++) {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        GnssStatus_t st = GNSS_IGNORED;
        if ((seed & 7U) == 0U) {
            const char *piece = pieces[(seed >> 8) % (sizeof(pieces) / sizeof(pieces[0]))];
            (void)feed(&rx, piece, (uint64_t)i, NULL);
        } else {
            (void)gnss_rx_feed(&rx, (char)(seed >> 16), (uint64_t)i, &st);
        }
        GnssFix_t fix;
        if (gnss_latest_fix(&rx, (uint64_t)i, 0xFFFFFFFFU, &fix)) {
            double r2 = (fix.x_m * fix.x_m) + (fix.y_m * fix.y_m) + (fix.z_m * fix.z_m);
            assert_true(fix.sats >= 4U);
            assert_true(fix.pdop > 0.0);
            assert_true((r2 >= (GNSS_LEO_MIN_RADIUS_M * GNSS_LEO_MIN_RADIUS_M)) &&
                        (r2 <= (GNSS_LEO_MAX_RADIUS_M * GNSS_LEO_MAX_RADIUS_M)));
        }
    }
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_checksums),
        cmocka_unit_test(test_fix_from_reference_sentences),
        cmocka_unit_test(test_record_needs_real_fix),
        cmocka_unit_test(test_gga_must_be_recent),
        cmocka_unit_test(test_implausible_records),
        cmocka_unit_test(test_framing_errors),
        cmocka_unit_test(test_control_characters),
        cmocka_unit_test(test_overlong_line_refused_whole),
        cmocka_unit_test(test_fix_age),
        cmocka_unit_test(test_fuzz),
    };

    return cmocka_run_group_tests_name("GNSS NMEA Driver Tests", tests, NULL, NULL);
}
