/**
 * @file test_sensors.c
 * @brief Unit tests for Sensor module
 *
 * Tests sensor polling and telemetry per SRS-F070 through SRS-F073.
 *
 * @requirement SRS-F070 Collect telemetry from all sensors
 * @requirement SRS-F071 Sample sensors at configurable rates
 * @requirement SRS-F072 Validate sensor readings against limits
 * @requirement SRS-F073 Include telemetry in beacon transmissions
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <math.h>

/* Include the module under test */
#include "smart_qso.h"
#include "sensors.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Initialize sensor module */
    SmartQsoResult_t result = sensors_init();
    assert_int_equal(result, SMART_QSO_OK);
    return 0;
}

static int teardown(void **state) {
    (void)state;
    return 0;
}

/*===========================================================================*/
/* Test Cases: Initialization                                                 */
/*===========================================================================*/

/**
 * @brief Test sensor framework initialization
 *
 * @requirement SRS-F070 Sensor system initialization
 */
static void test_sensors_init(void **state) {
    (void)state;

    /* Module initialized in setup */
    size_t count = sensors_get_count();
    /* count is size_t (unsigned), so it's always >= 0 */
    assert_true(count <= SMART_QSO_MAX_SENSORS);
}

/**
 * @brief Test loading from non-existent file returns IO error
 */
static void test_sensors_load_yaml_missing(void **state) {
    (void)state;

    SmartQsoResult_t result = sensors_load_yaml("/nonexistent/file.yaml");
    assert_int_equal(result, SMART_QSO_ERROR_IO);
}

/*===========================================================================*/
/* Test Cases: Sensor Polling                                                 */
/*===========================================================================*/

/**
 * @brief Test polling with default sensors
 *
 * @requirement SRS-F070 Collect telemetry from all sensors
 */
/**
 * The shipped sensors.yaml puts each id on the list-item line ("- id: BV").
 * The loader used to start the item and drop the rest of that line, so
 * every sensor had an empty id.
 */
static void test_sensors_load_yaml_keeps_ids(void **state) {
    (void)state;
    /* Standard C only (the suite builds with -std=c11): a fixed name in the
     * test's working directory, removed below */
    const char *path = "test_sensors_keeps_ids.yaml";
    FILE *f = fopen(path, "w");
    assert_non_null(f);
    (void)fputs("sensors:\n"
                "  - id: BV\n"
                "    name: Battery Voltage\n"
                "    type: eps_voltage\n"
                "    channel: battery\n"
                "    units: V\n"
                "    period_ms: 1000\n"
                "  - id: BT\n"
                "    name: Battery Temperature\n"
                "    type: eps_temperature\n"
                "    channel: battery\n"
                "    units: C\n"
                "    period_ms: 2000\n", f);
    assert_int_equal(fclose(f), 0);

    assert_int_equal(sensors_load_yaml(path), SMART_QSO_OK);
    (void)remove(path);

    assert_int_equal(sensors_get_count(), 2);
    Sensor_t sensor;
    assert_int_equal(sensors_get_by_id("BV", &sensor), SMART_QSO_OK);
    assert_string_equal(sensor.name, "Battery Voltage");
    assert_int_equal(sensors_get_by_id("BT", &sensor), SMART_QSO_OK);
    assert_int_equal(sensor.period_ms, 2000);
}

static void test_sensors_poll(void **state) {
    (void)state;

    /* Load default sensors */
    sensors_load_defaults();

    /* Poll sensors */
    uint64_t now_ms = smart_qso_now_ms();
    size_t polled = sensors_poll(now_ms);

    /* polled is size_t (unsigned), just check it's reasonable */
    assert_true(polled <= SMART_QSO_MAX_SENSORS);
}

/**
 * @brief Test sensor polling respects timing
 *
 * @requirement SRS-F071 Sample sensors at configurable rates
 */
static void test_sensors_poll_timing(void **state) {
    (void)state;

    sensors_load_defaults();

    uint64_t now_ms = smart_qso_now_ms();

    /* First poll should poll all due sensors */
    size_t first_poll = sensors_poll(now_ms);

    /* Immediate second poll should poll fewer (respecting periods) */
    size_t second_poll = sensors_poll(now_ms);

    /* Second poll should be equal or less */
    assert_true(second_poll <= first_poll);
}

/**
 * @brief A sensor records when it was last read successfully (SW-13: the
 * beacon reports only fresh readings)
 */
static void test_sensors_last_read_time(void **state) {
    (void)state;
    Sensor_t bv;

    sensors_load_defaults();
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    assert_true(bv.last_read_ms == 0U);  /* never read */

    uint64_t now_ms = smart_qso_now_ms();
    (void)sensors_poll(now_ms);
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    assert_true(bv.last_read_ms == now_ms);

    /* Not due: no new reading, time unchanged */
    (void)sensors_poll(now_ms + 1U);
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    assert_true(bv.last_read_ms == now_ms);
}

/**
 * @brief The payload heat-spreader thermistor (PT) is read by the OBC for
 * the payload guard, separately from the EPS temperatures (SW-16)
 */
static void test_payload_spreader_sensor(void **state) {
    (void)state;
    Sensor_t pt;

    sensors_load_defaults();
    assert_int_equal(sensors_get_by_id("PT", &pt), SMART_QSO_OK);
    assert_string_equal(pt.type, "payload_temperature");
    assert_string_equal(pt.units, "C");
    assert_non_null(pt.read);
    (void)sensors_poll(smart_qso_now_ms());
    assert_int_equal(sensors_get_by_id("PT", &pt), SMART_QSO_OK);
    assert_true(pt.last_read_ms != 0U);
    assert_true((pt.last_value > -40.0) && (pt.last_value < 70.0));
}

/*===========================================================================*/
/* Test Cases: Sensor Access                                                  */
/*===========================================================================*/

/**
 * @brief Test getting sensor for unknown ID returns error
 */
static void test_sensors_get_by_id_unknown(void **state) {
    (void)state;

    sensors_load_defaults();

    Sensor_t sensor;
    SmartQsoResult_t result = sensors_get_by_id("NONEXISTENT_SENSOR_ID", &sensor);

    /* Should return error for unknown sensor */
    assert_int_not_equal(result, SMART_QSO_OK);
}

/**
 * @brief Test getting sensor by index with defaults
 */
static void test_sensors_get(void **state) {
    (void)state;

    sensors_load_defaults();

    size_t count = sensors_get_count();
    if (count > 0) {
        Sensor_t sensor;
        SmartQsoResult_t result = sensors_get(0, &sensor);
        assert_int_equal(result, SMART_QSO_OK);
        assert_true(strlen(sensor.id) > 0);
    }
    /* If no default sensors, test passes trivially */
}

/*===========================================================================*/
/* Test Cases: Telemetry Formatting                                           */
/*===========================================================================*/

/**
 * @brief Test telemetry formatting with small buffer
 *
 * @requirement SRS-F073 Telemetry formatting handles buffer limits
 */
static void test_sensors_format_telemetry_small_buffer(void **state) {
    (void)state;

    sensors_load_defaults();
    sensors_poll(smart_qso_now_ms());

    char buffer[16];  /* Very small buffer */
    size_t len = sensors_format_telemetry(buffer, sizeof(buffer));

    /* Should not overflow */
    assert_true(len < sizeof(buffer));
    buffer[sizeof(buffer) - 1] = '\0';  /* Ensure null termination */
}

/**
 * @brief Test telemetry formatting with null buffer returns 0
 */
static void test_sensors_format_telemetry_null(void **state) {
    (void)state;

    /* Null buffer should return 0 - implementation may assert instead */
    size_t len = sensors_format_telemetry(NULL, 100);
    assert_int_equal(len, 0);
}

/*===========================================================================*/
/* Test Cases: Sensor Configuration                                           */
/*===========================================================================*/

/**
 * @brief Test sensor count with defaults
 */
static void test_sensors_get_count(void **state) {
    (void)state;

    sensors_load_defaults();
    size_t count = sensors_get_count();

    /* Should have at least some default sensors */
    assert_true(count <= SMART_QSO_MAX_SENSORS);
}

/**
 * @brief Test loading default sensors
 */
static void test_sensors_load_defaults(void **state) {
    (void)state;

    SmartQsoResult_t result = sensors_load_defaults();
    assert_int_equal(result, SMART_QSO_OK);

    /* Should have some sensors after loading defaults */
    size_t count = sensors_get_count();
    /* May or may not have sensors depending on configuration */
    assert_true(count <= SMART_QSO_MAX_SENSORS);
}

/**
 * @brief Test setting environment variables
 */
static void test_sensors_set_environment(void **state) {
    (void)state;

    /* This function is void, just verify it doesn't crash */
    sensors_set_environment(true, 0.75);
    sensors_set_environment(false, 0.25);
}

/**
 * @brief The table holds at most SMART_QSO_MAX_SENSORS: loading the
 * defaults again adds sensors until it is full, never past it
 */
static void test_sensors_table_never_overflows(void **state) {
    (void)state;

    for (int i = 0; i < 5; i++) {
        assert_int_equal(sensors_load_defaults(), SMART_QSO_OK);
    }
    assert_int_equal(sensors_get_count(), SMART_QSO_MAX_SENSORS);

    Sensor_t last;
    assert_int_equal(sensors_get(SMART_QSO_MAX_SENSORS - 1U, &last), SMART_QSO_OK);
    assert_string_equal(last.id, "BV");   /* 3 x 10 defaults, then SET and BV */
}

/**
 * @brief A sensor copy's read function still works from C, and the
 * telemetry text and the status sensor have their C formats
 */
static void test_sensors_read_function_and_formats(void **state) {
    (void)state;
    Sensor_t st;
    Sensor_t bv;
    double value = 0.0;
    char text[8] = {0};
    char line[128];

    sensors_load_defaults();
    sensors_set_environment(true, 0.75);
    assert_int_equal(sensors_get_by_id("ST", &st), SMART_QSO_OK);
    assert_int_equal(st.value_type, SENSOR_VALUE_HEX2);
    assert_true(st.read(&st, &value, text));
    assert_int_equal(strlen(text), 2U);
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    assert_true(bv.read(&bv, &value, text));
    assert_true((value > 8.0) && (value < 8.2));

    assert_int_equal(sensors_poll_one(1U), SMART_QSO_OK);   /* BV */
    assert_int_equal(sensors_get_by_id("BV", &bv), SMART_QSO_OK);
    size_t len = sensors_format_telemetry(line, sizeof(line));
    char expected[32];
    (void)snprintf(expected, sizeof(expected), "SET=0.000s,BV=%.3fV,", bv.last_value);
    assert_true(len > strlen(expected));
    assert_memory_equal(line, expected, strlen(expected));
    assert_int_equal(line[len - 1U], ',');
    assert_int_equal(line[len], '\0');
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_sensors_init, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_yaml_missing, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_yaml_keeps_ids, setup, teardown),

        /* Polling tests */
        cmocka_unit_test_setup_teardown(test_sensors_poll, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_poll_timing, setup, teardown),

        /* Sensor access tests */
        cmocka_unit_test_setup_teardown(test_sensors_get_by_id_unknown, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_get, setup, teardown),

        /* Telemetry formatting tests */
        cmocka_unit_test_setup_teardown(test_sensors_format_telemetry_small_buffer, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_format_telemetry_null, setup, teardown),

        /* Configuration tests */
        cmocka_unit_test_setup_teardown(test_sensors_get_count, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_defaults, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_set_environment, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_last_read_time, setup, teardown),
        cmocka_unit_test_setup_teardown(test_payload_spreader_sensor, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_table_never_overflows, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_read_function_and_formats, setup, teardown),
    };

    return cmocka_run_group_tests_name("Sensor Tests", tests, NULL, NULL);
}
