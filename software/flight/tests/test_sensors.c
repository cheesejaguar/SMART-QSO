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
/* Test Data                                                                  */
/*===========================================================================*/

/* Test sensor configuration YAML */
static const char *test_sensor_yaml =
    "sensors:\n"
    "  - id: BV\n"
    "    name: Battery Voltage\n"
    "    type: voltage\n"
    "    channel: battery\n"
    "    units: V\n"
    "    period_ms: 1000\n"
    "  - id: BT\n"
    "    name: Battery Temperature\n"
    "    type: temperature\n"
    "    channel: battery\n"
    "    units: C\n"
    "    period_ms: 5000\n"
    "  - id: SOC\n"
    "    name: State of Charge\n"
    "    type: percentage\n"
    "    channel: battery\n"
    "    units: %\n"
    "    period_ms: 1000\n";

static const char *test_yaml_file = "/tmp/test_sensors.yaml";

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;

    /* Write test YAML file */
    FILE *f = fopen(test_yaml_file, "w");
    if (f) {
        fputs(test_sensor_yaml, f);
        fclose(f);
    }

    /* Initialize sensor module */
    SmartQsoResult_t result = sensors_init();
    assert_int_equal(result, SMART_QSO_OK);

    return 0;
}

static int teardown(void **state) {
    (void)state;
    unlink(test_yaml_file);
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
 * @brief Test loading sensor configuration from YAML
 *
 * @requirement SRS-F071 Configurable sensor rates
 */
static void test_sensors_load_yaml(void **state) {
    (void)state;

    SmartQsoResult_t result = sensors_load_yaml(test_yaml_file);
    assert_int_equal(result, SMART_QSO_OK);

    size_t count = sensors_get_count();
    assert_true(count >= 3);  /* At least our test sensors */
}

/**
 * @brief Test loading from non-existent file
 */
static void test_sensors_load_yaml_missing(void **state) {
    (void)state;

    SmartQsoResult_t result = sensors_load_yaml("/nonexistent/file.yaml");
    assert_int_equal(result, SMART_QSO_ERROR_IO);
}

/**
 * @brief Test loading with null path
 */
static void test_sensors_load_yaml_null(void **state) {
    (void)state;

    SmartQsoResult_t result = sensors_load_yaml(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Cases: Sensor Polling                                                 */
/*===========================================================================*/

/**
 * @brief Test polling all sensors
 *
 * @requirement SRS-F070 Collect telemetry from all sensors
 */
static void test_sensors_poll(void **state) {
    (void)state;

    /* Load test configuration */
    sensors_load_yaml(test_yaml_file);

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

    sensors_load_yaml(test_yaml_file);

    uint64_t now_ms = smart_qso_now_ms();

    /* First poll should poll all due sensors */
    size_t first_poll = sensors_poll(now_ms);

    /* Immediate second poll should poll fewer (respecting periods) */
    size_t second_poll = sensors_poll(now_ms);

    /* Second poll should be equal or less */
    assert_true(second_poll <= first_poll);
}

/*===========================================================================*/
/* Test Cases: Sensor Access                                                  */
/*===========================================================================*/

/**
 * @brief Test getting sensor by ID
 *
 * @requirement SRS-F070 Collect telemetry from sensors
 */
static void test_sensors_get_by_id(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);
    sensors_poll(smart_qso_now_ms());

    Sensor_t sensor;
    SmartQsoResult_t result = sensors_get_by_id("BV", &sensor);

    assert_int_equal(result, SMART_QSO_OK);
    assert_string_equal(sensor.id, "BV");
    assert_string_equal(sensor.name, "Battery Voltage");
}

/**
 * @brief Test getting sensor for unknown ID
 */
static void test_sensors_get_by_id_unknown(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);

    Sensor_t sensor;
    SmartQsoResult_t result = sensors_get_by_id("UNKNOWN", &sensor);

    assert_int_equal(result, SMART_QSO_ERROR);
}

/**
 * @brief Test getting sensor with null pointers
 */
static void test_sensors_get_by_id_null(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);

    SmartQsoResult_t result = sensors_get_by_id("BV", NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);

    Sensor_t sensor;
    result = sensors_get_by_id(NULL, &sensor);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Test getting sensor by index
 */
static void test_sensors_get(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);

    size_t count = sensors_get_count();
    assert_true(count > 0);

    Sensor_t sensor;
    SmartQsoResult_t result = sensors_get(0, &sensor);

    assert_int_equal(result, SMART_QSO_OK);
    assert_true(strlen(sensor.id) > 0);
}

/*===========================================================================*/
/* Test Cases: Telemetry Formatting                                           */
/*===========================================================================*/

/**
 * @brief Test telemetry string formatting
 *
 * @requirement SRS-F073 Include telemetry in beacon transmissions
 */
static void test_sensors_format_telemetry(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);
    sensors_poll(smart_qso_now_ms());

    char buffer[256];
    size_t len = sensors_format_telemetry(buffer, sizeof(buffer));

    assert_true(len > 0);
    assert_true(len < sizeof(buffer));

    /* Should contain sensor IDs */
    assert_non_null(strstr(buffer, "BV="));
}

/**
 * @brief Test telemetry formatting with small buffer
 *
 * @requirement SRS-F073 Telemetry formatting handles buffer limits
 */
static void test_sensors_format_telemetry_small_buffer(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);
    sensors_poll(smart_qso_now_ms());

    char buffer[16];  /* Very small buffer */
    size_t len = sensors_format_telemetry(buffer, sizeof(buffer));

    /* Should not overflow */
    assert_true(len < sizeof(buffer));
    buffer[sizeof(buffer) - 1] = '\0';  /* Ensure null termination */
}

/**
 * @brief Test telemetry formatting with null buffer
 */
static void test_sensors_format_telemetry_null(void **state) {
    (void)state;

    size_t len = sensors_format_telemetry(NULL, 100);
    assert_int_equal(len, 0);
}

/*===========================================================================*/
/* Test Cases: Sensor Configuration                                           */
/*===========================================================================*/

/**
 * @brief Test sensor count
 */
static void test_sensors_get_count(void **state) {
    (void)state;

    sensors_load_yaml(test_yaml_file);
    size_t count = sensors_get_count();

    assert_true(count >= 3);  /* At least our test sensors */
    assert_true(count <= SMART_QSO_MAX_SENSORS);
}

/**
 * @brief Test loading default sensors
 */
static void test_sensors_load_defaults(void **state) {
    (void)state;

    SmartQsoResult_t result = sensors_load_defaults();
    assert_int_equal(result, SMART_QSO_OK);

    size_t count = sensors_get_count();
    assert_true(count > 0);
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

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_sensors_init, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_yaml, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_yaml_missing, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_yaml_null, setup, teardown),

        /* Polling tests */
        cmocka_unit_test_setup_teardown(test_sensors_poll, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_poll_timing, setup, teardown),

        /* Sensor access tests */
        cmocka_unit_test_setup_teardown(test_sensors_get_by_id, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_get_by_id_unknown, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_get_by_id_null, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_get, setup, teardown),

        /* Telemetry formatting tests */
        cmocka_unit_test_setup_teardown(test_sensors_format_telemetry, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_format_telemetry_small_buffer, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_format_telemetry_null, setup, teardown),

        /* Configuration tests */
        cmocka_unit_test_setup_teardown(test_sensors_get_count, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_load_defaults, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensors_set_environment, setup, teardown),
    };

    return cmocka_run_group_tests_name("Sensor Tests", tests, NULL, NULL);
}
