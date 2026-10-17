/**
 * @file test_beacon.c
 * @brief Beacon scheduling, transmit gating, AX.25 framing and text source
 *
 * Rewritten for CMocka against the current beacon.h (SW-09); the Unity
 * suite it replaces targeted an API that no longer exists. Specification:
 * docs/BEACON_SPEC.md and docs/BASELINE_PARAMETERS.md. The information
 * field layout, telemetry validity and the content filter are covered by
 * test_beacon_format.c, test_beacon_telemetry.c and test_beacon_filter.c.
 *
 * @requirement SYS-210 The spacecraft shall transmit AX.25 formatted beacons
 * @requirement SYS-212 Beacons shall include spacecraft callsign
 * @requirement SYS-215 Beacon content by AI when available, template fallback otherwise
 * @requirement SRS-F020 Transmit AX.25 beacon at configurable intervals
 * @requirement SRS-F025 Limit beacon info field to 256 bytes
 * @requirement SYS-204 Cease all RF emissions on telecommand (TX-OFF)
 * @requirement SYS-205 Cease transmissions on loss of contact
 * @requirement SYS-224 No RF signal earlier than 45 minutes after deployment
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <stdio.h>
#include <string.h>

#include "ax25.h"
#include "beacon.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "rf_control.h"
#include "smart_qso.h"
#include "station_id.h"
#include "hal/hal_flash.h"

/** Beacon intervals per power mode (BEACON_SPEC.md section 7,
 *  BASELINE_PARAMETERS.md "Data": every 60/120/180 s) */
#define SPEC_INTERVAL_ACTIVE_MS  60000U
#define SPEC_INTERVAL_IDLE_MS    120000U
#define SPEC_INTERVAL_SAFE_MS    180000U

/** Flags, two addresses, control, PID and FCS around the information field */
#define FRAME_OVERHEAD  (1U + (2U * AX25_ADDR_LEN) + 1U + 1U + 2U + 1U)

/** Transmitter permitted, beacon powered, beacon module freshly initialized */
static int setup(void **state)
{
    (void)state;
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    /* rf_control_init() does not clear the automatic inhibits */
    rf_control_set_inhibit(RF_INHIBIT_CONTACT, false);
    rf_control_set_inhibit(RF_INHIBIT_RF_SILENCE, false);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_true(rf_control_tx_permitted());
    assert_int_equal(eps_init(), SMART_QSO_OK);
    /* EPS state persists between runs, so set the beacon switch explicitly */
    assert_int_equal(eps_control_beacon(true), SMART_QSO_OK);
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    return 0;
}

static BeaconState_t get_state(void)
{
    BeaconState_t st;
    assert_int_equal(beacon_get_state(&st), SMART_QSO_OK);
    return st;
}

static uint32_t interval_for(PowerMode_t mode)
{
    beacon_update_interval(mode);
    return get_state().interval_ms;
}

/** Serialized length of the frame built from content (0 on failure) */
static size_t frame_bytes(const BeaconContent_t *content)
{
    Ax25Frame_t frame;
    uint8_t buffer[AX25_MAX_FRAME_LEN];

    if (beacon_build_ax25_frame(content, &frame) != SMART_QSO_OK) {
        return 0U;
    }
    return beacon_serialize_ax25(&frame, buffer, sizeof(buffer));
}

/** Before beacon_init() the module transmits nothing and keeps no state.
 *  Must run first: once initialized the module cannot be uninitialized. */
static void test_uninitialized_module_is_inert(void **state)
{
    (void)state;
    BeaconState_t st;

    /* Everything else permits a beacon, so only the beacon module says no */
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_int_equal(eps_control_beacon(true), SMART_QSO_OK);
    assert_true(eps_is_beacon_enabled());
    assert_true(rf_control_tx_permitted());

    uint64_t now = smart_qso_now_ms();
    assert_false(beacon_should_transmit(now));
    beacon_update_interval(POWER_MODE_SAFE);
    beacon_mark_transmitted(now);
    beacon_set_ai_available(true);
    beacon_reset_stats();
    assert_int_equal(beacon_get_state(&st), SMART_QSO_ERROR);
    assert_int_equal(beacon_store_ai_text("Hello from orbit", 16U), SMART_QSO_ERROR);
    assert_int_equal(beacon_get_total_count(), 0U);
    assert_int_equal(beacon_get_ai_percentage(), 0U);

    /* Initialization starts at the ACTIVE rate, templates only, no history */
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    st = get_state();
    assert_int_equal(st.interval_ms, SPEC_INTERVAL_ACTIVE_MS);
    assert_false(st.ai_available);
    assert_int_equal(st.sequence, 0U);
    assert_int_equal(st.last_ai_text[0], '\0');
    assert_int_equal(beacon_get_total_count(), 0U);
}

/** One beacon every 60 s in ACTIVE, 120 s in IDLE, 180 s in SAFE */
static void test_interval_per_power_mode(void **state)
{
    (void)state;
    assert_int_equal(interval_for(POWER_MODE_SAFE), SPEC_INTERVAL_SAFE_MS);
    assert_int_equal(interval_for(POWER_MODE_IDLE), SPEC_INTERVAL_IDLE_MS);
    assert_int_equal(interval_for(POWER_MODE_ACTIVE), SPEC_INTERVAL_ACTIVE_MS);
    /* No mode beacons faster than ACTIVE, not even an unknown one */
    assert_true(interval_for((PowerMode_t)3) >= SPEC_INTERVAL_ACTIVE_MS);
}

/** A beacon is due exactly one interval after the last one, in every mode */
static void test_beacon_due_after_interval(void **state)
{
    (void)state;
    const struct { PowerMode_t mode; uint32_t interval_ms; } cases[] = {
        {POWER_MODE_ACTIVE, SPEC_INTERVAL_ACTIVE_MS},
        {POWER_MODE_IDLE, SPEC_INTERVAL_IDLE_MS},
        {POWER_MODE_SAFE, SPEC_INTERVAL_SAFE_MS},
    };
    uint64_t now = smart_qso_now_ms();

    for (size_t i = 0U; i < (sizeof(cases) / sizeof(cases[0])); i++) {
        beacon_update_interval(cases[i].mode);
        beacon_mark_transmitted(now);
        assert_int_equal(get_state().last_beacon_ms, now);
        assert_false(beacon_should_transmit(now));
        assert_false(beacon_should_transmit(now + cases[i].interval_ms - 1U));
        assert_true(beacon_should_transmit(now + cases[i].interval_ms));
    }
}

/** TX-OFF, loss of contact and the post-deployment silence each stop the
 *  beacon: none is due and none is sent, and nothing is counted */
static void test_no_beacon_while_tx_inhibited(void **state)
{
    (void)state;
    BeaconContent_t content;
    uint64_t now = smart_qso_now_ms();
    uint64_t due = now + SPEC_INTERVAL_ACTIVE_MS;
    const RfInhibit_t inhibits[] = {RF_INHIBIT_CONTACT, RF_INHIBIT_RF_SILENCE};

    beacon_mark_transmitted(now);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_true(beacon_should_transmit(due));

    /* TX-OFF (latched) */
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_false(beacon_should_transmit(due));
    assert_int_equal(beacon_transmit(&content), SMART_QSO_ERROR);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    assert_true(beacon_should_transmit(due));

    /* Automatic inhibits */
    for (size_t i = 0U; i < (sizeof(inhibits) / sizeof(inhibits[0])); i++) {
        rf_control_set_inhibit(inhibits[i], true);
        assert_false(beacon_should_transmit(due));
        assert_int_equal(beacon_transmit(&content), SMART_QSO_ERROR);
        rf_control_set_inhibit(inhibits[i], false);
        assert_true(beacon_should_transmit(due));
    }

    BeaconState_t st = get_state();
    assert_int_equal(st.template_count + st.ai_beacon_count, 0U);
    assert_int_equal(st.total_bytes_tx, 0U);

    /* Once nothing inhibits, the beacon goes out */
    assert_int_equal(beacon_transmit(&content), SMART_QSO_OK);
    assert_int_equal(beacon_get_total_count(), 1U);
}

/** BEACON_CALLSIGN is the text form of the station identity */
static void test_callsign_is_station_identity(void **state)
{
    (void)state;
    char expected[16];

    (void)snprintf(expected, sizeof(expected), "%s-%u", SQSO_CALLSIGN, (unsigned int)SQSO_SSID);
    assert_string_equal(BEACON_CALLSIGN, expected);
    assert_string_equal(BEACON_CALLSIGN, "KK6GDB-11");
}

/** UI frame CQ <- KK6GDB-11, no path, PID 0xF0, call sign at the start of
 *  the information field, FCS over everything between the flags */
static void test_frame_envelope(void **state)
{
    (void)state;
    BeaconContent_t content;
    Ax25Frame_t frame;
    uint8_t expected_addr[AX25_ADDR_LEN];
    const char prefix[] = "de " BEACON_CALLSIGN ": ";

    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(beacon_build_ax25_frame(&content, &frame), SMART_QSO_OK);

    ax25_encode_address("CQ", 0U, expected_addr, false);
    assert_memory_equal(frame.dest_addr, expected_addr, AX25_ADDR_LEN);
    ax25_encode_address(SQSO_CALLSIGN, SQSO_SSID, expected_addr, true);
    assert_memory_equal(frame.src_addr, expected_addr, AX25_ADDR_LEN);
    assert_int_equal(frame.ctrl, 0x03U);
    assert_int_equal(frame.pid, 0xF0U);
    assert_true((frame.info_len > 0U) && (frame.info_len <= 256U));
    assert_memory_equal(frame.info, prefix, sizeof(prefix) - 1U);

    uint8_t fcs_data[(2U * AX25_ADDR_LEN) + 2U + BEACON_MAX_PAYLOAD_LEN];
    memcpy(fcs_data, frame.dest_addr, AX25_ADDR_LEN);
    memcpy(&fcs_data[AX25_ADDR_LEN], frame.src_addr, AX25_ADDR_LEN);
    fcs_data[2U * AX25_ADDR_LEN] = frame.ctrl;
    fcs_data[(2U * AX25_ADDR_LEN) + 1U] = frame.pid;
    memcpy(&fcs_data[(2U * AX25_ADDR_LEN) + 2U], frame.info, frame.info_len);
    assert_int_equal(frame.fcs, beacon_crc16(fcs_data, (2U * AX25_ADDR_LEN) + 2U + frame.info_len));
}

/** The serialized frame is flagged at both ends and parses as a valid UI
 *  frame from the station, with the information field intact */
static void test_serialized_frame_parses(void **state)
{
    (void)state;
    BeaconContent_t content;
    Ax25Frame_t frame;
    Ax25UiView_t view;
    uint8_t buffer[AX25_MAX_FRAME_LEN];

    assert_int_equal(beacon_generate_content(&content, "Hello from orbit", 16U), SMART_QSO_OK);
    assert_int_equal(beacon_build_ax25_frame(&content, &frame), SMART_QSO_OK);
    size_t len = beacon_serialize_ax25(&frame, buffer, sizeof(buffer));

    assert_int_equal(len, frame.info_len + FRAME_OVERHEAD);
    assert_int_equal(buffer[0], 0x7EU);
    assert_int_equal(buffer[len - 1U], 0x7EU);
    /* FCS little-endian, last before the closing flag */
    assert_int_equal(buffer[len - 3U], frame.fcs & 0xFFU);
    assert_int_equal(buffer[len - 2U], (frame.fcs >> 8) & 0xFFU);

    assert_int_equal(ax25_parse_ui(&buffer[1], len - 2U, &view), AX25_OK);
    assert_true(ax25_address_matches(view.dest, "CQ", 0U));
    assert_true(ax25_address_matches(view.src, SQSO_CALLSIGN, SQSO_SSID));
    assert_int_equal(view.info_len, frame.info_len);
    assert_memory_equal(view.info, frame.info, frame.info_len);
    assert_non_null(strstr((const char *)frame.info, ": Hello from orbit | T="));
}

/** Serialization refuses a buffer one byte short and fits one exactly */
static void test_serialize_buffer_size(void **state)
{
    (void)state;
    BeaconContent_t content;
    Ax25Frame_t frame;
    uint8_t buffer[AX25_MAX_FRAME_LEN];

    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(beacon_build_ax25_frame(&content, &frame), SMART_QSO_OK);
    size_t needed = frame.info_len + FRAME_OVERHEAD;

    assert_int_equal(beacon_serialize_ax25(&frame, buffer, needed - 1U), 0U);
    assert_int_equal(beacon_serialize_ax25(&frame, buffer, needed), needed);
}

/** Without usable AI text the beacon cycles through every template, then
 *  starts again; each beacon gets the next sequence number */
static void test_template_fallback_cycles(void **state)
{
    (void)state;
    BeaconContent_t content;

    assert_true(beacon_template_count > 1U);
    for (size_t round = 0U; round < 2U; round++) {
        for (size_t i = 0U; i < beacon_template_count; i++) {
            assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
            assert_int_equal(content.source, BEACON_SOURCE_TEMPLATE);
            assert_string_equal(content.text, beacon_templates[i]);
            assert_int_equal(content.text_len, strlen(beacon_templates[i]));
            assert_int_equal(content.sequence, (round * beacon_template_count) + i + 1U);
        }
    }
}

/** Stored AI text is used only while AI is available; otherwise a template */
static void test_stored_ai_text(void **state)
{
    (void)state;
    BeaconContent_t content;
    const char text[] = "Hello from orbit";

    /* Available but nothing stored yet: template */
    beacon_set_ai_available(true);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(content.source, BEACON_SOURCE_TEMPLATE);

    assert_int_equal(beacon_store_ai_text(text, strlen(text)), SMART_QSO_OK);
    assert_string_equal(get_state().last_ai_text, text);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(content.source, BEACON_SOURCE_AI);
    assert_string_equal(content.text, text);
    assert_int_equal(content.text_len, strlen(text));

    /* AI unavailable: back to templates even though text is stored */
    beacon_set_ai_available(false);
    assert_false(get_state().ai_available);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(content.source, BEACON_SOURCE_TEMPLATE);
}

/** Transmissions are counted by text source and size; reset clears them */
static void test_transmit_statistics(void **state)
{
    (void)state;
    BeaconContent_t tmpl;
    BeaconContent_t ai;

    assert_int_equal(beacon_generate_content(&tmpl, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(beacon_generate_content(&ai, "Hello from orbit", 16U), SMART_QSO_OK);
    assert_int_equal(ai.source, BEACON_SOURCE_AI);
    size_t tmpl_bytes = frame_bytes(&tmpl);
    size_t ai_bytes = frame_bytes(&ai);
    assert_true((tmpl_bytes > 0U) && (ai_bytes > 0U));

    assert_int_equal(beacon_transmit(&tmpl), SMART_QSO_OK);
    assert_int_equal(beacon_get_total_count(), 1U);
    assert_int_equal(beacon_get_ai_percentage(), 0U);

    assert_int_equal(beacon_transmit(&ai), SMART_QSO_OK);
    assert_int_equal(beacon_get_total_count(), 2U);
    assert_int_equal(beacon_get_ai_percentage(), 50U);

    assert_int_equal(beacon_transmit(&ai), SMART_QSO_OK);
    assert_int_equal(beacon_get_ai_percentage(), 66U);

    BeaconState_t st = get_state();
    assert_int_equal(st.template_count, 1U);
    assert_int_equal(st.ai_beacon_count, 2U);
    assert_int_equal(st.total_bytes_tx, tmpl_bytes + (2U * ai_bytes));

    beacon_reset_stats();
    st = get_state();
    assert_int_equal(beacon_get_total_count(), 0U);
    assert_int_equal(beacon_get_ai_percentage(), 0U);
    assert_int_equal(st.total_bytes_tx, 0U);
    assert_int_equal(st.sequence, 0U);
    assert_int_equal(beacon_generate_content(&tmpl, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(tmpl.sequence, 1U);
}

/** fault_flags bit 0 is set while the last logged fault is ERROR or worse */
static void test_fault_flags_from_last_fault(void **state)
{
    (void)state;
    BeaconContent_t content;

    assert_int_equal(fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_ERROR,
                                   "test: error", 0.5), SMART_QSO_OK);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(content.telemetry.fault_flags, 0x01U);

    assert_int_equal(fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_INFO,
                                   "test: info", 0.5), SMART_QSO_OK);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(content.telemetry.fault_flags, 0x00U);
}

static void test_null_arguments(void **state)
{
    (void)state;
    BeaconContent_t content;
    Ax25Frame_t frame;
    uint8_t buffer[AX25_MAX_FRAME_LEN];

    assert_int_equal(beacon_generate_content(NULL, NULL, 0U), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(beacon_generate_content(&content, NULL, 0U), SMART_QSO_OK);
    assert_int_equal(beacon_build_ax25_frame(NULL, &frame), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(beacon_build_ax25_frame(&content, NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(beacon_build_ax25_frame(&content, &frame), SMART_QSO_OK);
    assert_int_equal(beacon_serialize_ax25(NULL, buffer, sizeof(buffer)), 0U);
    assert_int_equal(beacon_serialize_ax25(&frame, NULL, sizeof(buffer)), 0U);
    assert_int_equal(beacon_transmit(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(beacon_get_state(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(beacon_store_ai_text(NULL, 5U), SMART_QSO_ERROR_NULL_PTR);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* First and without setup: it needs the module never initialized */
        cmocka_unit_test(test_uninitialized_module_is_inert),
        cmocka_unit_test_setup(test_interval_per_power_mode, setup),
        cmocka_unit_test_setup(test_beacon_due_after_interval, setup),
        cmocka_unit_test_setup(test_no_beacon_while_tx_inhibited, setup),
        cmocka_unit_test_setup(test_callsign_is_station_identity, setup),
        cmocka_unit_test_setup(test_frame_envelope, setup),
        cmocka_unit_test_setup(test_serialized_frame_parses, setup),
        cmocka_unit_test_setup(test_serialize_buffer_size, setup),
        cmocka_unit_test_setup(test_template_fallback_cycles, setup),
        cmocka_unit_test_setup(test_stored_ai_text, setup),
        cmocka_unit_test_setup(test_transmit_statistics, setup),
        cmocka_unit_test_setup(test_fault_flags_from_last_fault, setup),
        cmocka_unit_test_setup(test_null_arguments, setup),
    };

    return cmocka_run_group_tests_name("Beacon Tests", tests, NULL, NULL);
}
