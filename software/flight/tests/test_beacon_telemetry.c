/**
 * @file test_beacon_telemetry.c
 * @brief Beacon telemetry comes from measurements, with validity (SW-13)
 *
 * The beacon used to downlink fixed values (SOC 75, 7.4 V, sunlit) and the
 * reset count as the QSO count. It now reports only measurements it was
 * given, fresh, and marks every other field "-".
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
#include "downlink.h"
#include "eps_control.h"
#include "mission_data.h"
#include "rf_control.h"
#include "smart_qso.h"
#include "hal/hal_flash.h"

#define MEASURED (BEACON_TLM_SOC | BEACON_TLM_BV | BEACON_TLM_SUN | BEACON_TLM_QSO)

static int setup(void **state)
{
    (void)state;
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    /* Beacon power on: EPS state persists between runs, so set it explicitly */
    (void)eps_control_beacon(true);
    return 0;
}

static void generate(BeaconContent_t *content, char *payload, size_t payload_len)
{
    assert_int_equal(beacon_generate_content(content, NULL, 0U), SMART_QSO_OK);
    assert_true(beacon_format_payload(content, payload, payload_len) > 0U);
}

static bool ends_with(const char *s, const char *suffix)
{
    size_t n = strlen(s);
    size_t k = strlen(suffix);
    return (n >= k) && (strcmp(&s[n - k], suffix) == 0);
}

/** Nothing measured: no values, not defaults */
static void test_no_measurements_no_values(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];

    generate(&content, payload, sizeof(payload));
    assert_int_equal(content.telemetry.valid & MEASURED, 0U);
    assert_non_null(strstr(payload, " SOC=- BV=- SUN=- "));
    assert_true(ends_with(payload, " QSO=-"));
}

/** Fresh measurements are reported as given */
static void test_measurements_reported(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];
    const BeaconMeasurements_t m = {
        .soc = 42U, .battery_mv = 8123U, .sunlit = 0U, .qso_count = 7U,
        .valid = MEASURED,
    };

    assert_int_equal(beacon_update_measurements(&m, smart_qso_now_ms()), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_int_equal(content.telemetry.soc, 42U);
    assert_int_equal(content.telemetry.battery_mv, 8123U);
    assert_int_equal(content.telemetry.sunlit, 0U);
    assert_int_equal(content.telemetry.qso_count, 7U);
    assert_int_equal(content.telemetry.valid & MEASURED, MEASURED);
    assert_non_null(strstr(payload, " SOC=42 BV=8.1 SUN=0 "));
    assert_true(ends_with(payload, " QSO=7"));
}

/** A field the source did not mark valid is not reported */
static void test_unmarked_fields_not_reported(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];
    const BeaconMeasurements_t m = {
        .soc = 99U, .battery_mv = 8000U, .sunlit = 1U, .qso_count = 5U,
        .valid = BEACON_TLM_BV,
    };

    assert_int_equal(beacon_update_measurements(&m, smart_qso_now_ms()), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_non_null(strstr(payload, " SOC=- BV=8.0 SUN=- "));
    assert_true(ends_with(payload, " QSO=-"));
    /* Unflagged values are not carried at all */
    assert_int_equal(content.telemetry.soc, 0U);
    assert_int_equal(content.telemetry.sunlit, 0U);
    assert_int_equal(content.telemetry.qso_count, 0U);
}

/** Re-initializing the beacon forgets earlier measurements */
static void test_init_forgets_measurements(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];
    const BeaconMeasurements_t m = {.soc = 42U, .valid = BEACON_TLM_SOC};

    assert_int_equal(beacon_update_measurements(&m, smart_qso_now_ms()), SMART_QSO_OK);
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_non_null(strstr(payload, " SOC=- "));
}

/** RF is the radio power switch state, when the EPS can report it */
static void test_rf_from_eps(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];

    assert_int_equal(eps_init(), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_int_not_equal(content.telemetry.valid & BEACON_TLM_RF, 0U);
    assert_true((strstr(payload, " RF=0 ") != NULL) || (strstr(payload, " RF=1 ") != NULL));
}

/** Measurements older than BEACON_MEAS_MAX_AGE_MS are not reported */
static void test_stale_measurements_dropped(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];
    const BeaconMeasurements_t m = {
        .soc = 42U, .battery_mv = 8123U, .sunlit = 1U, .qso_count = 1U, .valid = MEASURED,
    };
    uint64_t now = smart_qso_now_ms();

    assert_int_equal(beacon_update_measurements(&m, now - BEACON_MEAS_MAX_AGE_MS + 2000U), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_int_equal(content.telemetry.valid & MEASURED, MEASURED);

    assert_int_equal(beacon_update_measurements(&m, now - BEACON_MEAS_MAX_AGE_MS - 1000U), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_int_equal(content.telemetry.valid & MEASURED, 0U);
    assert_non_null(strstr(payload, " SOC=- BV=- SUN=- "));
}

/** The reset count is not a QSO count */
static void test_qso_is_not_reset_count(void **state)
{
    (void)state;
    BeaconContent_t content;
    char payload[BEACON_MAX_PAYLOAD_LEN];

    assert_int_equal(mission_data_init(), SMART_QSO_OK);
    assert_int_equal(mission_data_increment_reset(), SMART_QSO_OK);
    assert_int_equal(mission_data_increment_reset(), SMART_QSO_OK);
    generate(&content, payload, sizeof(payload));
    assert_true(ends_with(payload, " QSO=-"));
}

static void test_update_rejects_bad_input(void **state)
{
    (void)state;
    BeaconMeasurements_t m = {.soc = 101U, .valid = BEACON_TLM_SOC};

    assert_int_equal(beacon_update_measurements(NULL, 0U), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(beacon_update_measurements(&m, smart_qso_now_ms()), SMART_QSO_ERROR_INVALID);
    m.soc = 100U;
    m.sunlit = 2U;
    m.valid = BEACON_TLM_SUN;
    assert_int_equal(beacon_update_measurements(&m, smart_qso_now_ms()), SMART_QSO_ERROR_INVALID);
}

/** Wall-clock milliseconds exceed 32 bits: the interval still holds */
static void test_interval_with_epoch_time(void **state)
{
    (void)state;
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_true(rf_control_tx_permitted());

    uint64_t now = smart_qso_now_ms();
    assert_true(now > UINT32_MAX);
    beacon_mark_transmitted(now);
    assert_false(beacon_should_transmit(now + 1000U));
    assert_true(beacon_should_transmit(now + BEACON_INTERVAL_ACTIVE_MS));
}

/** EMERGENCY sheds the beacon: none is due, or sent, while the EPS has the
 *  beacon powered off (state-machine entry action, SW-14) */
static void test_no_beacon_while_eps_beacon_off(void **state)
{
    (void)state;
    BeaconContent_t content;

    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(downlink_init(), SMART_QSO_OK);
    assert_int_equal(eps_init(), SMART_QSO_OK);
    uint64_t now = smart_qso_now_ms();
    beacon_mark_transmitted(now);

    assert_int_equal(eps_control_beacon(false), SMART_QSO_OK);
    assert_false(beacon_should_transmit(now + BEACON_INTERVAL_SAFE_MS));
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_not_equal(beacon_transmit(&content), SMART_QSO_OK);

    assert_int_equal(eps_control_beacon(true), SMART_QSO_OK);
    assert_true(beacon_should_transmit(now + BEACON_INTERVAL_SAFE_MS));
    assert_int_equal(beacon_transmit(&content), SMART_QSO_OK);
}

/** No mode beacons more often than the baseline allows: the beacon power line
 *  (BASELINE_PARAMETERS.md section 3) assumes ~2 s of transmission per 60 s */
static void test_interval_within_power_baseline(void **state)
{
    (void)state;
    BeaconState_t st;
    const PowerMode_t modes[] = {POWER_MODE_SAFE, POWER_MODE_IDLE, POWER_MODE_ACTIVE};
    uint32_t previous = UINT32_MAX;

    for (size_t i = 0U; i < (sizeof(modes) / sizeof(modes[0])); i++) {
        beacon_update_interval(modes[i]);
        assert_int_equal(beacon_get_state(&st), SMART_QSO_OK);
        assert_true(st.interval_ms >= 60000U);
        assert_true(st.interval_ms <= previous);  /* less power, fewer beacons */
        previous = st.interval_ms;
    }
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_no_measurements_no_values, setup),
        cmocka_unit_test_setup(test_measurements_reported, setup),
        cmocka_unit_test_setup(test_unmarked_fields_not_reported, setup),
        cmocka_unit_test_setup(test_stale_measurements_dropped, setup),
        cmocka_unit_test_setup(test_init_forgets_measurements, setup),
        cmocka_unit_test_setup(test_rf_from_eps, setup),
        cmocka_unit_test_setup(test_qso_is_not_reset_count, setup),
        cmocka_unit_test_setup(test_update_rejects_bad_input, setup),
        cmocka_unit_test_setup(test_interval_with_epoch_time, setup),
        cmocka_unit_test_setup(test_interval_within_power_baseline, setup),
        cmocka_unit_test_setup(test_no_beacon_while_eps_beacon_off, setup),
    };

    return cmocka_run_group_tests_name("Beacon Telemetry Tests", tests, NULL, NULL);
}
