/**
 * @file test_telemetry_flow.c
 * @brief Telemetry path, sensors to downlink: integration of the real modules
 *
 * One main-loop pass (main.c run_tick) samples the sensors, gives the EPS the
 * state of charge, records the beacon measurements and sends a beacon when one
 * is due; when the payload is on it also sends a TELEMETRY line to the Jetson
 * and reads the Jetson's beacon text. These tests run that path through the
 * real sensors, EPS, beacon, AX.25, Jetson-link and telemetry modules. main.c
 * is not in the test library, so the glue it does (assembling the beacon
 * measurements, the start of the TELEMETRY line) is repeated here from public
 * calls; it is marked where it is.
 *
 * - Sensors and EPS to beacon text (BEACON_SPEC.md sections 3-4): measured
 *   fields as given, unmeasured or stale ones as "-", never a default.
 * - Beacon text to the AX.25 frame on the air (KK6GDB-11 to CQ, UI, FCS).
 * - TELEMETRY line to the Jetson and BEACON_RSP back (ICD_OBC_JETSON.md 4).
 *   The UART transport itself is not exercised: uart_init() opens a serial
 *   device, which the host test has no stand-in for.
 * - EPS state to the binary telemetry frames (telemetry.h), which nothing in
 *   the flight software sends yet.
 *
 * The earlier Unity version of this file also tested a telemetry store,
 * circular buffer, priority downlink queue and point registry; no such API
 * exists, so those scenarios are gone.
 *
 * @requirement SRS-F070 The software shall collect telemetry from all sensors
 * @requirement SRS-F071 The software shall sample sensors at configurable rates
 * @requirement SRS-F073 The software shall include telemetry in beacon transmissions
 * @requirement SRS-F030 The software shall send health telemetry to Jetson via UART
 * @requirement SRS-I005 Message format shall include CRC for integrity checking
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ax25.h"
#include "beacon.h"
#include "downlink.h"
#include "eps_control.h"
#include "hal/hal_flash.h"
#include "jetson_link.h"
#include "rf_control.h"
#include "sensors.h"
#include "smart_qso.h"
#include "station_id.h"
#include "system_state.h"
#include "telemetry.h"

static int setup(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(downlink_init(), SMART_QSO_OK);
    /* EPS state persists between runs: set what the tests rely on */
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_int_equal(eps_control_beacon(true), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(sensors_init(), SMART_QSO_OK);
    assert_int_equal(sensors_load_defaults(), SMART_QSO_OK);
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    assert_int_equal(tlm_init(), SMART_QSO_OK);
    return 0;
}

/** As main.c update_beacon_measurements(): SOC and SUN from the EPS model,
 *  BV only if the BV sensor was read within BEACON_MEAS_MAX_AGE_MS, no QSO */
static void record_measurements(bool sunlit, uint64_t now)
{
    BeaconMeasurements_t m;
    Sensor_t bv;

    (void)memset(&m, 0, sizeof(m));
    m.soc = (uint8_t)lround(eps_get_soc() * 100.0);
    m.sunlit = sunlit ? 1U : 0U;
    m.valid = BEACON_TLM_SOC | BEACON_TLM_SUN;
    if ((sensors_get_by_id("BV", &bv) == SMART_QSO_OK) && (bv.last_read_ms != 0U) &&
        ((now - bv.last_read_ms) <= BEACON_MEAS_MAX_AGE_MS) &&
        (bv.last_value > 0.0) && (bv.last_value < 65.0)) {
        m.battery_mv = (uint16_t)lround(bv.last_value * 1000.0);
        m.valid |= BEACON_TLM_BV;
    }
    assert_int_equal(beacon_update_measurements(&m, now), SMART_QSO_OK);
}

/** One main-loop pass up to the beacon: environment, EPS SOC, sensors, measurements */
static uint64_t obc_tick(bool sunlit, double soc)
{
    uint64_t now = smart_qso_now_ms();
    sensors_set_environment(sunlit, soc);
    eps_update_soc(soc);
    (void)sensors_poll(now);
    record_measurements(sunlit, now);
    return now;
}

/** The beacon information field as it would be sent now */
static void beacon_text(BeaconContent_t *content, char *text, size_t text_len,
                        const char *ai_text)
{
    size_t ai_len = (ai_text != NULL) ? strlen(ai_text) : 0U;
    assert_int_equal(beacon_generate_content(content, ai_text, ai_len), SMART_QSO_OK);
    assert_true(beacon_format_payload(content, text, text_len) > 0U);
}

static bool ends_with(const char *s, const char *suffix)
{
    size_t n = strlen(s);
    size_t k = strlen(suffix);
    return (n >= k) && (strcmp(&s[n - k], suffix) == 0);
}

/*===========================================================================*/
/* Sensors and EPS to the beacon                                              */
/*===========================================================================*/

/** Sensor readings and the EPS state of charge reach the beacon telemetry */
static void test_sensor_values_reach_beacon(void **cm_state)
{
    (void)cm_state;
    BeaconContent_t content;
    char text[BEACON_MAX_PAYLOAD_LEN];
    Sensor_t bv;

    assert_int_equal(eps_set_power_mode(POWER_MODE_IDLE, 0.5), SMART_QSO_OK);
    uint64_t now = obc_tick(true, 0.5);
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    assert_int_equal(bv.last_read_ms, now);

    beacon_text(&content, text, sizeof(text), NULL);
    assert_int_equal(strncmp(text, "de " BEACON_CALLSIGN ": ", strlen("de " BEACON_CALLSIGN ": ")), 0);
    assert_int_equal(content.telemetry.battery_mv, (uint16_t)lround(bv.last_value * 1000.0));
    /* Simulated battery at 50 % SOC: 8.1 V +/- 10 mV */
    assert_non_null(strstr(text, " M=IDLE SOC=50 BV=8.1 SUN=1 RF=1 "));
    assert_true(ends_with(text, " QSO=-"));

    /* Eclipse is reported as such */
    (void)obc_tick(false, 0.5);
    beacon_text(&content, text, sizeof(text), NULL);
    assert_non_null(strstr(text, " SUN=0 "));
}

/** EPS power mode, state of charge and radio switch reach the beacon */
static void test_eps_state_reaches_beacon(void **cm_state)
{
    (void)cm_state;
    BeaconContent_t content;
    char text[BEACON_MAX_PAYLOAD_LEN];
    const struct {
        PowerMode_t mode;
        double soc;
        const char *fields;
    } cases[] = {
        {POWER_MODE_SAFE, 0.20, " M=SAFE SOC=20 "},
        {POWER_MODE_IDLE, 0.45, " M=IDLE SOC=45 "},
        {POWER_MODE_ACTIVE, 0.80, " M=ACTIVE SOC=80 "},
    };

    for (size_t i = 0U; i < (sizeof(cases) / sizeof(cases[0])); i++) {
        assert_int_equal(eps_set_power_mode(cases[i].mode, cases[i].soc), SMART_QSO_OK);
        (void)obc_tick(true, cases[i].soc);
        beacon_text(&content, text, sizeof(text), NULL);
        assert_non_null(strstr(text, cases[i].fields));
        /* SAFE switches the radio off; the beacon reports the switch */
        assert_non_null(strstr(text, (cases[i].mode == POWER_MODE_SAFE) ? " RF=0 " : " RF=1 "));
    }
}

/** A battery sensor that stops reporting is sent as "-" once its last reading
 *  is older than 60 s; the EPS fields are still reported */
static void test_stale_sensor_sent_unmeasured(void **cm_state)
{
    (void)cm_state;
    BeaconContent_t content;
    char text[BEACON_MAX_PAYLOAD_LEN];

    (void)obc_tick(true, 0.5);
    beacon_text(&content, text, sizeof(text), NULL);
    assert_non_null(strstr(text, " BV=8.1 "));

    /* Sensor bus silent: measurements are still recorded, sensors not polled */
    smart_qso_sim_advance_ms(BEACON_MEAS_MAX_AGE_MS / 2U);
    record_measurements(true, smart_qso_now_ms());
    beacon_text(&content, text, sizeof(text), NULL);
    assert_non_null(strstr(text, " BV=8.1 "));

    smart_qso_sim_advance_ms(BEACON_MEAS_MAX_AGE_MS);
    record_measurements(true, smart_qso_now_ms());
    beacon_text(&content, text, sizeof(text), NULL);
    assert_non_null(strstr(text, " SOC=50 BV=- SUN=1 "));
    assert_int_equal(content.telemetry.battery_mv, 0U);

    /* Sampling again restores it */
    (void)obc_tick(true, 0.5);
    beacon_text(&content, text, sizeof(text), NULL);
    assert_non_null(strstr(text, " BV=8.1 "));
}

/** No battery sensor configured: BV is "-", not a default voltage */
static void test_no_battery_sensor_sent_unmeasured(void **cm_state)
{
    (void)cm_state;
    BeaconContent_t content;
    char text[BEACON_MAX_PAYLOAD_LEN];

    assert_int_equal(sensors_init(), SMART_QSO_OK);   /* no sensors loaded */
    (void)obc_tick(true, 0.5);
    beacon_text(&content, text, sizeof(text), NULL);
    assert_non_null(strstr(text, " SOC=50 BV=- SUN=1 "));
    assert_true(ends_with(text, " QSO=-"));
}

/** Each sensor is read once per configured period, not on every pass */
static void test_sensor_sampling_period(void **cm_state)
{
    (void)cm_state;
    Sensor_t bv;
    Sensor_t bt;
    const uint64_t t0 = smart_qso_now_ms();

    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    assert_int_equal(sensors_get_by_id("BT", &bt), SMART_QSO_OK);
    assert_int_equal(bv.period_ms, 1000U);
    assert_int_equal(bt.period_ms, 2000U);

    size_t total = sensors_get_count();
    assert_int_equal(sensors_poll(t0), total);         /* first pass reads all */
    assert_int_equal(sensors_poll(t0 + 500U), 0U);     /* nothing due yet */

    const struct { uint64_t at; uint64_t bv_read; uint64_t bt_read; } passes[] = {
        {t0 + 999U, t0, t0},
        {t0 + 1000U, t0 + 1000U, t0},
        {t0 + 1500U, t0 + 1000U, t0},
        {t0 + 2000U, t0 + 2000U, t0 + 2000U},
        {t0 + 10000U, t0 + 10000U, t0 + 10000U},
    };
    for (size_t i = 0U; i < (sizeof(passes) / sizeof(passes[0])); i++) {
        (void)sensors_poll(passes[i].at);
        assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
        assert_int_equal(sensors_get_by_id("BT", &bt), SMART_QSO_OK);
        assert_int_equal(bv.last_read_ms, passes[i].bv_read);
        assert_int_equal(bt.last_read_ms, passes[i].bt_read);
    }
}

/*===========================================================================*/
/* Beacon on the air                                                          */
/*===========================================================================*/

/** Full cycle: measurements to a transmitted AX.25 UI frame from KK6GDB-11 to
 *  CQ, carrying the information field, with a valid FCS */
static void test_beacon_frame_on_air(void **cm_state)
{
    (void)cm_state;
    BeaconContent_t content;
    BeaconState_t before;
    BeaconState_t after;
    Ax25Frame_t frame;
    char text[BEACON_MAX_PAYLOAD_LEN];
    uint8_t air[AX25_MAX_FRAME_LEN];

    assert_int_equal(eps_set_power_mode(POWER_MODE_IDLE, 0.5), SMART_QSO_OK);
    uint64_t now = obc_tick(true, 0.5);

    /* As main.c run_beacon(): interval from the power mode, send when due */
    beacon_update_interval(eps_get_power_mode());
    assert_true(beacon_should_transmit(now));
    assert_int_equal(beacon_get_state(&before), SMART_QSO_OK);
    beacon_text(&content, text, sizeof(text), NULL);
    assert_int_equal(beacon_transmit(&content), SMART_QSO_OK);
    beacon_mark_transmitted(now);
    assert_false(beacon_should_transmit(now + 1000U));
    assert_true(beacon_should_transmit(now + BEACON_INTERVAL_IDLE_MS));

    /* The frame that was sent */
    assert_int_equal(beacon_build_ax25_frame(&content, &frame), SMART_QSO_OK);
    size_t len = beacon_serialize_ax25(&frame, air, sizeof(air));
    assert_int_equal(beacon_get_state(&after), SMART_QSO_OK);
    assert_int_equal(after.total_bytes_tx - before.total_bytes_tx, len);
    assert_int_equal(after.template_count - before.template_count, 1U);

    const size_t info = 1U + (2U * AX25_ADDR_LEN) + 2U;
    assert_int_equal(len, info + strlen(text) + 2U + 1U);
    assert_int_equal(air[0], AX25_FLAG);
    assert_int_equal(air[len - 1U], AX25_FLAG);
    assert_true(ax25_address_matches(&air[1], "CQ", 0U));
    assert_true(ax25_address_matches(&air[1U + AX25_ADDR_LEN], SQSO_CALLSIGN, SQSO_SSID));
    assert_int_equal(air[1U + (2U * AX25_ADDR_LEN)], AX25_CTRL_UI);
    assert_int_equal(air[2U + (2U * AX25_ADDR_LEN)], AX25_PID_NO_L3);
    assert_memory_equal(&air[info], text, strlen(text));

    uint16_t fcs = (uint16_t)(air[len - 3U] | ((uint16_t)air[len - 2U] << 8));
    assert_int_equal(beacon_crc16(&air[1], len - 4U), fcs);
}

/*===========================================================================*/
/* OBC - Jetson lines                                                         */
/*===========================================================================*/

/** The TELEMETRY line (ICD_OBC_JETSON.md 4.2) carries the sensor readings, is
 *  printable, fits a line and ends with the CRC of everything before it */
static void test_jetson_telemetry_line(void **cm_state)
{
    (void)cm_state;
    char line[JETSON_LINE_MAX];
    char expected[64];
    Sensor_t bv;

    assert_int_equal(eps_set_power_mode(POWER_MODE_ACTIVE, 0.75), SMART_QSO_OK);
    uint64_t now = obc_tick(true, 0.75);

    /* Start of the line as main.c send_telemetry_to_jetson() writes it */
    size_t room = sizeof(line) - 6U;   /* ",XXXX\n" */
    int head = snprintf(line, room, "TELEMETRY,%llu,%s,%.3f,%d,%.1f,",
                        (unsigned long long)now, "SUNLIT", eps_get_soc(),
                        (int)eps_get_power_mode(), eps_get_power_limit());
    assert_true((head > 0) && ((size_t)head < room));
    size_t len = (size_t)head + sensors_format_telemetry(&line[head], room - (size_t)head);
    assert_true(len > (size_t)head);
    assert_int_equal(line[len - 1U], ',');
    len--;

    size_t sealed = jetson_link_seal(line, len, sizeof(line));
    assert_int_equal(sealed, len + 6U);
    assert_true(sealed <= JETSON_LINE_MAX);
    assert_int_equal(line[sealed - 1U], '\n');
    for (size_t i = 0U; i < (sealed - 1U); i++) {
        assert_true((line[i] >= 0x20) && (line[i] <= 0x7E));
    }

    char crc[5];
    (void)snprintf(crc, sizeof(crc), "%04X", (unsigned int)jetson_crc16((const uint8_t *)line, len));
    assert_int_equal(line[len], ',');
    assert_memory_equal(&line[len + 1U], crc, 4U);

    (void)snprintf(expected, sizeof(expected), "TELEMETRY,%llu,SUNLIT,0.750,2,3.0,",
                   (unsigned long long)now);
    assert_int_equal(strncmp(line, expected, strlen(expected)), 0);
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    (void)snprintf(expected, sizeof(expected), ",BV=%.3fV,", bv.last_value);
    assert_non_null(strstr(line, expected));
    assert_non_null(strstr(line, ",ST="));
}

/** The beacon request goes out, the Jetson's matching answer comes back
 *  through the line receiver and is sent once, behind the call sign and with
 *  the measured telemetry (BEACON_SPEC.md section 5) */
static void test_jetson_text_reaches_beacon(void **cm_state)
{
    (void)cm_state;
    char line[JETSON_LINE_MAX];
    BeaconContent_t content;
    char text[BEACON_MAX_PAYLOAD_LEN];
    JetsonRx_t rx;
    JetsonMsg_t msg;
    JetsonStatus_t status = JETSON_ERR_FORMAT;
    bool complete = false;

    size_t req = jetson_link_beacon_req(42U, (uint8_t)JETSON_TEXT_MAX, line, sizeof(line));
    assert_int_equal(req, strlen("BEACON_REQ,42,176,5593\n"));
    assert_memory_equal(line, "BEACON_REQ,42,176,5593\n", req);

    /* The ICD golden reply, byte by byte as the UART delivers it */
    const char *reply = "BEACON_RSP,42,Hello, world from orbit,0784\n";
    jetson_rx_init(&rx);
    for (size_t i = 0U; reply[i] != '\0'; i++) {
        complete = jetson_rx_feed(&rx, reply[i], &msg, &status);
    }
    assert_true(complete);
    assert_int_equal(status, JETSON_OK);
    assert_int_equal(msg.type, JETSON_MSG_BEACON_RSP);
    assert_int_equal(msg.seq, 42U);

    (void)obc_tick(true, 0.5);
    beacon_text(&content, text, sizeof(text), msg.text);
    assert_int_equal(content.source, BEACON_SOURCE_AI);
    assert_int_equal(strncmp(text, "de " BEACON_CALLSIGN ": Hello, world from orbit | T=",
                             strlen("de " BEACON_CALLSIGN ": Hello, world from orbit | T=")), 0);
    assert_non_null(strstr(text, " SOC=50 BV=8.1 SUN=1 "));

    /* The next beacon, with no new text, is a template */
    beacon_text(&content, text, sizeof(text), NULL);
    assert_int_equal(content.source, BEACON_SOURCE_TEMPLATE);
}

/*===========================================================================*/
/* EPS to the binary telemetry frames                                         */
/*===========================================================================*/

/** EPS state of charge and mode, through the system state, into the EPS and
 *  housekeeping frames; serialized, the CRC still checks */
static void test_eps_state_reaches_binary_frames(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    TlmEps_t eps;
    TlmHousekeeping_t hk;
    uint8_t bytes[TLM_MAX_FRAME_SIZE];
    size_t len = 0U;
    size_t written = 0U;

    assert_int_equal(eps_set_power_mode(POWER_MODE_IDLE, 0.45), SMART_QSO_OK);
    (void)obc_tick(true, 0.45);

    assert_int_equal(tlm_generate_eps(&frame, &len), SMART_QSO_OK);
    assert_int_equal(tlm_serialize(&frame, sizeof(TlmEps_t), bytes, sizeof(bytes), &written),
                     SMART_QSO_OK);
    assert_int_equal(written, len);
    (void)memcpy(&eps, &bytes[sizeof(TlmHeader_t)], sizeof(eps));
    assert_int_equal(eps.state_of_charge, 45U);
    assert_int_equal(eps.power_mode, POWER_MODE_IDLE);

    size_t covered = sizeof(TlmHeader_t) + sizeof(TlmEps_t);
    uint32_t crc = ((uint32_t)bytes[covered] << 24) | ((uint32_t)bytes[covered + 1U] << 16) |
                   ((uint32_t)bytes[covered + 2U] << 8) | (uint32_t)bytes[covered + 3U];
    assert_int_equal(smart_qso_crc32(bytes, covered), crc);

    /* A mode change shows in the next frame */
    assert_int_equal(eps_set_power_mode(POWER_MODE_SAFE, 0.20), SMART_QSO_OK);
    (void)obc_tick(true, 0.20);
    assert_int_equal(tlm_generate_housekeeping(&frame, &len), SMART_QSO_OK);
    (void)memcpy(&hk, frame.payload, sizeof(hk));
    assert_int_equal(hk.state_of_charge, 20U);
    assert_int_equal(hk.power_mode, POWER_MODE_SAFE);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_sensor_values_reach_beacon, setup),
        cmocka_unit_test_setup(test_eps_state_reaches_beacon, setup),
        cmocka_unit_test_setup(test_stale_sensor_sent_unmeasured, setup),
        cmocka_unit_test_setup(test_no_battery_sensor_sent_unmeasured, setup),
        cmocka_unit_test_setup(test_sensor_sampling_period, setup),
        cmocka_unit_test_setup(test_beacon_frame_on_air, setup),
        cmocka_unit_test_setup(test_jetson_telemetry_line, setup),
        cmocka_unit_test_setup(test_jetson_text_reaches_beacon, setup),
        cmocka_unit_test_setup(test_eps_state_reaches_binary_frames, setup),
    };

    return cmocka_run_group_tests_name("Telemetry Flow Tests", tests, NULL, NULL);
}
