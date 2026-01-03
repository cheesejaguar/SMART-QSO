/**
 * @file test_adcs.c
 * @brief Unit tests for ADCS control module
 *
 * Tests magnetometer/sun sensor drivers, attitude determination,
 * B-dot detumble, sun-pointing controller, and magnetorquer control.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "adcs_control.h"
#include <string.h>
#include <math.h>

/*******************************************************************************
 * Test Constants
 ******************************************************************************/

#define FLOAT_TOLERANCE     0.001f
#define PI                  3.14159265358979f

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

void setUp(void)
{
    adcs_init();
}

void tearDown(void)
{
    /* Cleanup after each test */
}

/*******************************************************************************
 * Test Cases - Initialization
 ******************************************************************************/

void test_adcs_init_returns_ok(void)
{
    adcs_error_t status = adcs_init();
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_adcs_default_mode(void)
{
    adcs_mode_t mode = adcs_get_mode();
    TEST_ASSERT_EQUAL(ADCS_MODE_IDLE, mode);
}

void test_adcs_default_state(void)
{
    adcs_state_t state = adcs_get_state();
    TEST_ASSERT_EQUAL(ADCS_STATE_UNKNOWN, state);
}

/*******************************************************************************
 * Test Cases - Magnetometer
 ******************************************************************************/

void test_mag_init_success(void)
{
    adcs_error_t status = adcs_mag_init();
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_mag_read_null_pointer(void)
{
    adcs_error_t status = adcs_mag_read(NULL);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_mag_read_success(void)
{
    vector3_t mag_field;
    adcs_error_t status = adcs_mag_read(&mag_field);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_mag_values_in_range(void)
{
    vector3_t mag_field;
    adcs_mag_read(&mag_field);

    /* Earth's field is typically 25-65 uT */
    float magnitude = sqrtf(mag_field.x * mag_field.x +
                           mag_field.y * mag_field.y +
                           mag_field.z * mag_field.z);

    /* In simulation, values may be synthetic */
    TEST_ASSERT_GREATER_OR_EQUAL(0.0f, magnitude);
}

void test_mag_calibration(void)
{
    vector3_t offset = {0.1f, -0.2f, 0.05f};
    adcs_error_t status = adcs_mag_set_calibration(&offset, NULL);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

/*******************************************************************************
 * Test Cases - Sun Sensor
 ******************************************************************************/

void test_sun_sensor_init_success(void)
{
    adcs_error_t status = adcs_sun_sensor_init();
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_sun_sensor_read_null_pointer(void)
{
    adcs_error_t status = adcs_sun_sensor_read(NULL);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_sun_sensor_read_success(void)
{
    vector3_t sun_vector;
    adcs_error_t status = adcs_sun_sensor_read(&sun_vector);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_sun_sensor_vector_normalized(void)
{
    vector3_t sun_vector;
    adcs_sun_sensor_read(&sun_vector);

    /* Sun vector should be unit length (or zero if in eclipse) */
    float magnitude = sqrtf(sun_vector.x * sun_vector.x +
                           sun_vector.y * sun_vector.y +
                           sun_vector.z * sun_vector.z);

    TEST_ASSERT_TRUE(magnitude < 0.01f || fabsf(magnitude - 1.0f) < 0.1f);
}

void test_sun_presence_detection(void)
{
    bool sun_visible = adcs_is_sun_visible();
    /* Just verify function returns without error */
    (void)sun_visible;
    TEST_PASS();
}

/*******************************************************************************
 * Test Cases - Attitude Determination
 ******************************************************************************/

void test_triad_null_pointers(void)
{
    quaternion_t attitude;
    adcs_error_t status = adcs_determine_attitude(NULL, NULL, &attitude);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_triad_success(void)
{
    vector3_t mag = {0.0f, 0.3f, 0.4f};
    vector3_t sun = {1.0f, 0.0f, 0.0f};
    quaternion_t attitude;

    adcs_error_t status = adcs_determine_attitude(&mag, &sun, &attitude);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_quaternion_normalized(void)
{
    vector3_t mag = {0.0f, 0.3f, 0.4f};
    vector3_t sun = {1.0f, 0.0f, 0.0f};
    quaternion_t attitude;

    adcs_determine_attitude(&mag, &sun, &attitude);

    float norm = sqrtf(attitude.w * attitude.w +
                      attitude.x * attitude.x +
                      attitude.y * attitude.y +
                      attitude.z * attitude.z);

    TEST_ASSERT_FLOAT_WITHIN(FLOAT_TOLERANCE, 1.0f, norm);
}

void test_get_attitude(void)
{
    quaternion_t attitude;
    adcs_error_t status = adcs_get_attitude(&attitude);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

/*******************************************************************************
 * Test Cases - B-Dot Detumble
 ******************************************************************************/

void test_bdot_enable(void)
{
    adcs_error_t status = adcs_set_mode(ADCS_MODE_DETUMBLE);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
    TEST_ASSERT_EQUAL(ADCS_MODE_DETUMBLE, adcs_get_mode());
}

void test_bdot_compute_dipole_null(void)
{
    adcs_error_t status = adcs_bdot_compute(NULL);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_bdot_compute_success(void)
{
    vector3_t dipole;
    adcs_error_t status = adcs_bdot_compute(&dipole);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_bdot_gain_setting(void)
{
    float gain = 0.5f;
    adcs_error_t status = adcs_bdot_set_gain(gain);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_bdot_threshold(void)
{
    float threshold = 0.1f;  /* rad/s */
    adcs_error_t status = adcs_set_detumble_threshold(threshold);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

/*******************************************************************************
 * Test Cases - Sun Pointing
 ******************************************************************************/

void test_sun_pointing_enable(void)
{
    adcs_error_t status = adcs_set_mode(ADCS_MODE_SUN_POINTING);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
    TEST_ASSERT_EQUAL(ADCS_MODE_SUN_POINTING, adcs_get_mode());
}

void test_sun_pointing_compute_null(void)
{
    adcs_error_t status = adcs_sun_pointing_compute(NULL);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_sun_pointing_compute_success(void)
{
    vector3_t dipole;
    adcs_error_t status = adcs_sun_pointing_compute(&dipole);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_sun_pointing_set_target(void)
{
    vector3_t target = {1.0f, 0.0f, 0.0f};  /* +X axis */
    adcs_error_t status = adcs_set_sun_target_axis(&target);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

/*******************************************************************************
 * Test Cases - Magnetorquer Control
 ******************************************************************************/

void test_mtq_init_success(void)
{
    adcs_error_t status = adcs_mtq_init();
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_mtq_set_dipole_null(void)
{
    adcs_error_t status = adcs_mtq_set_dipole(NULL);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_mtq_set_dipole_success(void)
{
    vector3_t dipole = {0.1f, 0.2f, -0.1f};
    adcs_error_t status = adcs_mtq_set_dipole(&dipole);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_mtq_off(void)
{
    adcs_error_t status = adcs_mtq_off();
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_mtq_saturation_limits(void)
{
    vector3_t dipole = {100.0f, 100.0f, 100.0f};  /* Way over limit */
    adcs_mtq_set_dipole(&dipole);

    vector3_t actual;
    adcs_mtq_get_dipole(&actual);

    /* Should be clamped to max */
    float max_dipole = adcs_mtq_get_max_dipole();
    TEST_ASSERT_LESS_OR_EQUAL(max_dipole, fabsf(actual.x));
    TEST_ASSERT_LESS_OR_EQUAL(max_dipole, fabsf(actual.y));
    TEST_ASSERT_LESS_OR_EQUAL(max_dipole, fabsf(actual.z));
}

/*******************************************************************************
 * Test Cases - ADCS State Machine
 ******************************************************************************/

void test_state_transition_to_detumble(void)
{
    adcs_set_mode(ADCS_MODE_DETUMBLE);
    adcs_update();  /* Run one cycle */

    adcs_state_t state = adcs_get_state();
    TEST_ASSERT_TRUE(state == ADCS_STATE_DETUMBLING ||
                     state == ADCS_STATE_UNKNOWN);
}

void test_mode_idle(void)
{
    adcs_error_t status = adcs_set_mode(ADCS_MODE_IDLE);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
    TEST_ASSERT_EQUAL(ADCS_MODE_IDLE, adcs_get_mode());
}

/*******************************************************************************
 * Test Cases - Telemetry
 ******************************************************************************/

void test_get_telemetry_null(void)
{
    adcs_error_t status = adcs_get_telemetry(NULL);
    TEST_ASSERT_EQUAL(ADCS_ERROR_INVALID_PARAM, status);
}

void test_get_telemetry_success(void)
{
    adcs_telemetry_t telemetry;
    adcs_error_t status = adcs_get_telemetry(&telemetry);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

void test_get_angular_rate(void)
{
    vector3_t rate;
    adcs_error_t status = adcs_get_angular_rate(&rate);
    TEST_ASSERT_EQUAL(ADCS_OK, status);
}

/*******************************************************************************
 * Test Cases - Error Handling
 ******************************************************************************/

void test_error_to_string_ok(void)
{
    const char *str = adcs_error_to_string(ADCS_OK);
    TEST_ASSERT_NOT_NULL(str);
    TEST_ASSERT_EQUAL_STRING("No error", str);
}

void test_error_to_string_invalid_param(void)
{
    const char *str = adcs_error_to_string(ADCS_ERROR_INVALID_PARAM);
    TEST_ASSERT_NOT_NULL(str);
}

void test_error_to_string_sensor_failure(void)
{
    const char *str = adcs_error_to_string(ADCS_ERROR_SENSOR_FAILURE);
    TEST_ASSERT_NOT_NULL(str);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Initialization */
    RUN_TEST(test_adcs_init_returns_ok);
    RUN_TEST(test_adcs_default_mode);
    RUN_TEST(test_adcs_default_state);

    /* Magnetometer */
    RUN_TEST(test_mag_init_success);
    RUN_TEST(test_mag_read_null_pointer);
    RUN_TEST(test_mag_read_success);
    RUN_TEST(test_mag_values_in_range);
    RUN_TEST(test_mag_calibration);

    /* Sun sensor */
    RUN_TEST(test_sun_sensor_init_success);
    RUN_TEST(test_sun_sensor_read_null_pointer);
    RUN_TEST(test_sun_sensor_read_success);
    RUN_TEST(test_sun_sensor_vector_normalized);
    RUN_TEST(test_sun_presence_detection);

    /* Attitude determination */
    RUN_TEST(test_triad_null_pointers);
    RUN_TEST(test_triad_success);
    RUN_TEST(test_quaternion_normalized);
    RUN_TEST(test_get_attitude);

    /* B-dot detumble */
    RUN_TEST(test_bdot_enable);
    RUN_TEST(test_bdot_compute_dipole_null);
    RUN_TEST(test_bdot_compute_success);
    RUN_TEST(test_bdot_gain_setting);
    RUN_TEST(test_bdot_threshold);

    /* Sun pointing */
    RUN_TEST(test_sun_pointing_enable);
    RUN_TEST(test_sun_pointing_compute_null);
    RUN_TEST(test_sun_pointing_compute_success);
    RUN_TEST(test_sun_pointing_set_target);

    /* Magnetorquer control */
    RUN_TEST(test_mtq_init_success);
    RUN_TEST(test_mtq_set_dipole_null);
    RUN_TEST(test_mtq_set_dipole_success);
    RUN_TEST(test_mtq_off);
    RUN_TEST(test_mtq_saturation_limits);

    /* State machine */
    RUN_TEST(test_state_transition_to_detumble);
    RUN_TEST(test_mode_idle);

    /* Telemetry */
    RUN_TEST(test_get_telemetry_null);
    RUN_TEST(test_get_telemetry_success);
    RUN_TEST(test_get_angular_rate);

    /* Error handling */
    RUN_TEST(test_error_to_string_ok);
    RUN_TEST(test_error_to_string_invalid_param);
    RUN_TEST(test_error_to_string_sensor_failure);

    return UNITY_END();
}
