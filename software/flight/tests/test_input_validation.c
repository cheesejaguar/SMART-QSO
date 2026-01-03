/**
 * @file test_input_validation.c
 * @brief Unit tests for input validation module
 *
 * Tests input validation functions per NPR 7150.2 and MISRA C:2012.
 *
 * @requirement SRS-SEC-001 All external inputs shall be validated
 * @requirement SRS-SEC-002 Commands shall have checksum verification
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <math.h>

/* Include the module under test */
#include "input_validation.h"

/*===========================================================================*/
/* Test Helpers                                                               */
/*===========================================================================*/

/**
 * @brief Build a valid test command
 */
static size_t build_test_command(uint8_t *buffer, uint8_t cmd_id,
                                  const uint8_t *payload, uint8_t payload_len)
{
    buffer[0] = 0xAA;  /* Sync byte 1 */
    buffer[1] = 0x55;  /* Sync byte 2 */
    buffer[2] = cmd_id;
    buffer[3] = payload_len;

    if ((payload != NULL) && (payload_len > 0)) {
        memcpy(&buffer[4], payload, payload_len);
    }

    /* Calculate checksum */
    uint16_t checksum = 0;
    size_t data_len = 4U + payload_len;
    for (size_t i = 0; i < data_len; i++) {
        checksum += buffer[i];
    }

    buffer[data_len] = (uint8_t)(checksum >> 8);
    buffer[data_len + 1] = (uint8_t)(checksum & 0xFF);

    return data_len + 2;
}

/*===========================================================================*/
/* Command Validation Tests                                                   */
/*===========================================================================*/

/**
 * @brief Test command validation with valid command
 */
static void test_validate_command_valid(void **state)
{
    (void)state;

    uint8_t cmd_buffer[32];
    uint8_t payload[] = {0x01};  /* Mode 1 */
    size_t cmd_len = build_test_command(cmd_buffer, CMD_ID_SET_MODE, payload, 1);

    CommandValidation_t result;
    SmartQsoResult_t ret = input_validate_command(cmd_buffer, cmd_len, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_true(result.is_valid);
    assert_int_equal(result.cmd_id, CMD_ID_SET_MODE);
    assert_int_equal(result.payload_length, 1);
}

/**
 * @brief Test command validation with NULL parameters
 */
static void test_validate_command_null_params(void **state)
{
    (void)state;

    uint8_t cmd_buffer[32];
    CommandValidation_t result;

    /* NULL command data */
    SmartQsoResult_t ret = input_validate_command(NULL, 10, &result);
    assert_int_equal(ret, RESULT_ERROR_PARAM);

    /* NULL result */
    ret = input_validate_command(cmd_buffer, 10, NULL);
    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test command validation with too short command
 */
static void test_validate_command_too_short(void **state)
{
    (void)state;

    uint8_t cmd_buffer[] = {0xAA, 0x55, 0x00};  /* Only 3 bytes */
    CommandValidation_t result;

    SmartQsoResult_t ret = input_validate_command(cmd_buffer, 3, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_false(result.is_valid);
    assert_int_equal(result.error_code, VALIDATION_ERR_TOO_SHORT);
}

/**
 * @brief Test command validation with invalid sync pattern
 */
static void test_validate_command_invalid_sync(void **state)
{
    (void)state;

    uint8_t cmd_buffer[] = {0xAB, 0x55, 0x00, 0x00, 0x00, 0x55};  /* Wrong sync */
    CommandValidation_t result;

    SmartQsoResult_t ret = input_validate_command(cmd_buffer, 6, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_false(result.is_valid);
    assert_int_equal(result.error_code, VALIDATION_ERR_SYNC);
}

/**
 * @brief Test command validation with bad checksum
 */
static void test_validate_command_bad_checksum(void **state)
{
    (void)state;

    uint8_t cmd_buffer[32];
    uint8_t payload[] = {0x01};
    size_t cmd_len = build_test_command(cmd_buffer, CMD_ID_SET_MODE, payload, 1);

    /* Corrupt checksum */
    cmd_buffer[cmd_len - 1] ^= 0xFF;

    CommandValidation_t result;
    SmartQsoResult_t ret = input_validate_command(cmd_buffer, cmd_len, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_false(result.is_valid);
    assert_int_equal(result.error_code, VALIDATION_ERR_CHECKSUM);
}

/**
 * @brief Test command payload validation for SET_MODE
 */
static void test_validate_payload_set_mode(void **state)
{
    (void)state;

    bool is_valid;
    uint8_t valid_payload[] = {0x02};  /* Valid mode */
    uint8_t invalid_payload[] = {0x05};  /* Invalid mode */

    /* Valid mode */
    SmartQsoResult_t ret = input_validate_command_payload(
        CMD_ID_SET_MODE, valid_payload, 1, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Invalid mode */
    ret = input_validate_command_payload(
        CMD_ID_SET_MODE, invalid_payload, 1, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/**
 * @brief Test command payload validation for SET_BEACON
 */
static void test_validate_payload_set_beacon(void **state)
{
    (void)state;

    bool is_valid;
    uint8_t valid_payload[] = {0x00, 60};    /* 60 seconds - valid */
    uint8_t invalid_payload[] = {0x01, 0x00}; /* 256 seconds - invalid */

    /* Valid interval */
    SmartQsoResult_t ret = input_validate_command_payload(
        CMD_ID_SET_BEACON, valid_payload, 2, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Invalid interval (too long) */
    ret = input_validate_command_payload(
        CMD_ID_SET_BEACON, invalid_payload, 2, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/*===========================================================================*/
/* Sensor Data Validation Tests                                               */
/*===========================================================================*/

/**
 * @brief Test voltage validation
 */
static void test_validate_voltage(void **state)
{
    (void)state;

    bool is_valid;

    /* Valid voltage */
    SmartQsoResult_t ret = input_validate_voltage(3.7f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Invalid voltage (negative) */
    ret = input_validate_voltage(-1.0f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);

    /* Invalid voltage (too high) */
    ret = input_validate_voltage(15.0f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/**
 * @brief Test temperature validation
 */
static void test_validate_temperature(void **state)
{
    (void)state;

    bool is_valid;

    /* Valid temperatures */
    SmartQsoResult_t ret = input_validate_temperature(25.0f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    ret = input_validate_temperature(-40.0f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Invalid temperature (too cold) */
    ret = input_validate_temperature(-60.0f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);

    /* Invalid temperature (too hot) */
    ret = input_validate_temperature(120.0f, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/**
 * @brief Test magnetometer validation
 */
static void test_validate_magnetometer(void **state)
{
    (void)state;

    bool is_valid;
    Vector3_t valid_mag = {10.0f, -20.0f, 30.0f};
    Vector3_t invalid_mag = {150.0f, 0.0f, 0.0f};  /* X out of range */

    /* Valid reading */
    SmartQsoResult_t ret = input_validate_magnetometer(&valid_mag, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Invalid reading */
    ret = input_validate_magnetometer(&invalid_mag, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);

    /* NULL pointer */
    ret = input_validate_magnetometer(NULL, &is_valid);
    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test NaN handling in float validation
 */
static void test_validate_float_nan(void **state)
{
    (void)state;

    bool is_valid;
    float nan_val = NAN;

    SmartQsoResult_t ret = input_validate_voltage(nan_val, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);  /* NaN should be rejected */
}

/*===========================================================================*/
/* String and Buffer Validation Tests                                         */
/*===========================================================================*/

/**
 * @brief Test string validation
 */
static void test_validate_string(void **state)
{
    (void)state;

    bool is_valid;

    /* Valid string */
    SmartQsoResult_t ret = input_validate_string("Hello, World!", 64, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Empty string is valid */
    ret = input_validate_string("", 64, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* NULL string */
    ret = input_validate_string(NULL, 64, &is_valid);
    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test string with non-printable characters
 */
static void test_validate_string_nonprintable(void **state)
{
    (void)state;

    bool is_valid;
    char bad_string[] = "Hello\x01World";  /* Contains non-printable */

    SmartQsoResult_t ret = input_validate_string(bad_string, 64, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/**
 * @brief Test buffer validation
 */
static void test_validate_buffer(void **state)
{
    (void)state;

    bool is_valid;
    uint8_t buffer[100];

    /* Sufficient buffer */
    SmartQsoResult_t ret = input_validate_buffer(buffer, 100, 50, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Insufficient buffer */
    ret = input_validate_buffer(buffer, 100, 150, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);

    /* NULL buffer */
    ret = input_validate_buffer(NULL, 100, 50, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/**
 * @brief Test pointer alignment validation
 */
static void test_validate_alignment(void **state)
{
    (void)state;

    bool is_valid;
    uint32_t aligned_var;
    uint8_t byte_array[10];

    /* 4-byte aligned pointer */
    SmartQsoResult_t ret = input_validate_alignment(&aligned_var, 4, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Potentially misaligned pointer (byte array + 1) */
    ret = input_validate_alignment(&byte_array[1], 4, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    /* Result depends on actual address - just verify function runs */

    /* NULL pointer */
    ret = input_validate_alignment(NULL, 4, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);

    /* Zero alignment */
    ret = input_validate_alignment(&aligned_var, 0, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/*===========================================================================*/
/* Configuration Validation Tests                                             */
/*===========================================================================*/

/**
 * @brief Test uint32 range validation
 */
static void test_validate_config_uint32(void **state)
{
    (void)state;

    bool is_valid;

    /* In range */
    SmartQsoResult_t ret = input_validate_config_uint32(50, 0, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* At boundaries */
    ret = input_validate_config_uint32(0, 0, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    ret = input_validate_config_uint32(100, 0, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Out of range */
    ret = input_validate_config_uint32(101, 0, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/**
 * @brief Test int32 range validation
 */
static void test_validate_config_int32(void **state)
{
    (void)state;

    bool is_valid;

    /* In range */
    SmartQsoResult_t ret = input_validate_config_int32(-50, -100, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_true(is_valid);

    /* Out of range (negative) */
    ret = input_validate_config_int32(-150, -100, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);

    /* Out of range (positive) */
    ret = input_validate_config_int32(150, -100, 100, &is_valid);
    assert_int_equal(ret, RESULT_OK);
    assert_false(is_valid);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Command validation tests */
        cmocka_unit_test(test_validate_command_valid),
        cmocka_unit_test(test_validate_command_null_params),
        cmocka_unit_test(test_validate_command_too_short),
        cmocka_unit_test(test_validate_command_invalid_sync),
        cmocka_unit_test(test_validate_command_bad_checksum),
        cmocka_unit_test(test_validate_payload_set_mode),
        cmocka_unit_test(test_validate_payload_set_beacon),

        /* Sensor validation tests */
        cmocka_unit_test(test_validate_voltage),
        cmocka_unit_test(test_validate_temperature),
        cmocka_unit_test(test_validate_magnetometer),
        cmocka_unit_test(test_validate_float_nan),

        /* String/buffer tests */
        cmocka_unit_test(test_validate_string),
        cmocka_unit_test(test_validate_string_nonprintable),
        cmocka_unit_test(test_validate_buffer),
        cmocka_unit_test(test_validate_alignment),

        /* Config validation tests */
        cmocka_unit_test(test_validate_config_uint32),
        cmocka_unit_test(test_validate_config_int32),
    };

    return cmocka_run_group_tests_name("Input Validation Tests", tests, NULL, NULL);
}
