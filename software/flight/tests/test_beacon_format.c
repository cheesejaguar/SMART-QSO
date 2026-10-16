/**
 * @file test_beacon_format.c
 * @brief Beacon information field always fits its frame
 *
 * A maximum-length beacon text used to overflow the 256-byte information
 * field, so beacon_format_payload() returned 0 and the beacon was silently
 * dropped (GCC 16 -Wformat-truncation flagged it). The text is now cut to the
 * space left after the fixed telemetry fields, which are never cut.
 *
 * @requirement SYS-210 The spacecraft shall transmit AX.25 formatted beacons
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "beacon.h"

static void fill(BeaconContent_t *content, size_t text_len)
{
    memset(content, 0, sizeof(*content));
    memset(content->text, 'A', text_len);
    content->text[text_len] = '\0';
    content->text_len = text_len;
    content->telemetry.valid = BEACON_TLM_SOC | BEACON_TLM_BV | BEACON_TLM_SUN |
                               BEACON_TLM_RF | BEACON_TLM_QSO;
}

static void set_max_telemetry(BeaconContent_t *content)
{
    content->telemetry.timestamp = UINT32_MAX;
    content->telemetry.power_mode = 2U;
    content->telemetry.soc = UINT8_MAX;
    content->telemetry.battery_mv = UINT16_MAX;
    content->telemetry.sunlit = UINT8_MAX;
    content->telemetry.rf_power = UINT8_MAX;
    content->telemetry.qso_count = UINT16_MAX;
}

static bool ends_with(const char *s, size_t len, const char *suffix)
{
    size_t n = strlen(suffix);
    return (len >= n) && (memcmp(&s[len - n], suffix, n) == 0);
}

/** Longest text, largest telemetry values: still fits, fields intact */
static void test_max_text_fits_frame(void **state)
{
    (void)state;
    BeaconContent_t content;
    char buffer[BEACON_MAX_PAYLOAD_LEN];

    fill(&content, BEACON_MAX_TEXT_LEN - 1U);
    set_max_telemetry(&content);

    size_t len = beacon_format_payload(&content, buffer, sizeof(buffer));
    assert_true(len > 0U);
    assert_true(len < sizeof(buffer));
    assert_int_equal(strncmp(buffer, "de " BEACON_CALLSIGN ": ", sizeof("de " BEACON_CALLSIGN ": ") - 1U), 0);
    assert_true(ends_with(buffer, len, " BV=65.5 SUN=255 RF=255 QSO=65535"));
    assert_non_null(strstr(buffer, " | T=4294967295 M=ACTIVE SOC=255"));
}

/** The frame builder no longer rejects a maximum-length text */
static void test_max_text_builds_frame(void **state)
{
    (void)state;
    BeaconContent_t content;
    Ax25Frame_t frame;

    fill(&content, BEACON_MAX_TEXT_LEN - 1U);
    set_max_telemetry(&content);
    assert_int_equal(beacon_build_ax25_frame(&content, &frame), SMART_QSO_OK);
    assert_true(frame.info_len > 0U);
    assert_true(frame.info_len < sizeof(frame.info));
}

/** Short text is not cut */
static void test_short_text_unchanged(void **state)
{
    (void)state;
    BeaconContent_t content;
    char buffer[BEACON_MAX_PAYLOAD_LEN];

    fill(&content, 0U);
    (void)strncpy(content.text, "Hello from orbit", sizeof(content.text) - 1U);
    content.telemetry.battery_mv = 7400U;
    content.telemetry.power_mode = 1U;

    size_t len = beacon_format_payload(&content, buffer, sizeof(buffer));
    assert_string_equal(buffer,
        "de " BEACON_CALLSIGN ": Hello from orbit | T=0 M=IDLE SOC=0 BV=7.4 SUN=0 RF=0 QSO=0");
    assert_int_equal(len, strlen(buffer));
}

/** Battery voltage keeps one decimal, rounded */
static void test_battery_voltage_format(void **state)
{
    (void)state;
    BeaconContent_t content;
    char buffer[BEACON_MAX_PAYLOAD_LEN];
    const struct { uint16_t mv; const char *text; } cases[] = {
        {0U, "BV=0.0 "}, {7400U, "BV=7.4 "}, {7449U, "BV=7.4 "}, {7450U, "BV=7.5 "},
        {8399U, "BV=8.4 "}, {65535U, "BV=65.5 "},
    };

    for (size_t i = 0U; i < (sizeof(cases) / sizeof(cases[0])); i++) {
        fill(&content, 1U);
        content.telemetry.battery_mv = cases[i].mv;
        assert_true(beacon_format_payload(&content, buffer, sizeof(buffer)) > 0U);
        assert_non_null(strstr(buffer, cases[i].text));
    }
}

/** A smaller buffer cuts the text further but keeps every field; a buffer
 *  too small for the fields yields nothing */
static void test_small_buffers(void **state)
{
    (void)state;
    BeaconContent_t content;
    char buffer[100];

    fill(&content, 150U);
    size_t len = beacon_format_payload(&content, buffer, sizeof(buffer));
    assert_true((len > 0U) && (len < sizeof(buffer)));
    assert_true(ends_with(buffer, len, " QSO=0"));

    assert_int_equal(beacon_format_payload(&content, buffer, 40U), 0U);
    assert_int_equal(beacon_format_payload(NULL, buffer, sizeof(buffer)), 0U);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_max_text_fits_frame),
        cmocka_unit_test(test_max_text_builds_frame),
        cmocka_unit_test(test_short_text_unchanged),
        cmocka_unit_test(test_battery_voltage_format),
        cmocka_unit_test(test_small_buffers),
    };

    return cmocka_run_group_tests_name("Beacon Format Tests", tests, NULL, NULL);
}
