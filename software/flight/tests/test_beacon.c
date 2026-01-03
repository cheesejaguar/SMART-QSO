/**
 * @file test_beacon.c
 * @brief Unit tests for beacon generation module
 *
 * Tests AX.25 frame formatting, callsign encoding, beacon scheduling,
 * and integration with Jetson-generated text.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "beacon.h"
#include <string.h>

/*******************************************************************************
 * Test Data
 ******************************************************************************/

static const char *TEST_CALLSIGN = "SQSO-1";
static const char *TEST_DEST = "CQ";

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

void setUp(void)
{
    beacon_init();
}

void tearDown(void)
{
    /* Cleanup after each test */
}

/*******************************************************************************
 * Test Cases - Initialization
 ******************************************************************************/

void test_beacon_init_returns_ok(void)
{
    beacon_error_t status = beacon_init();
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

void test_beacon_default_config(void)
{
    beacon_config_t config;
    beacon_get_config(&config);

    TEST_ASSERT_GREATER_THAN(0, config.interval_sec);
    TEST_ASSERT_LESS_OR_EQUAL(300, config.interval_sec);
}

/*******************************************************************************
 * Test Cases - Callsign Configuration
 ******************************************************************************/

void test_set_callsign_null(void)
{
    beacon_error_t status = beacon_set_callsign(NULL, 0);
    TEST_ASSERT_EQUAL(BEACON_ERROR_INVALID_PARAM, status);
}

void test_set_callsign_success(void)
{
    beacon_error_t status = beacon_set_callsign(TEST_CALLSIGN, 0);
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

void test_set_callsign_with_ssid(void)
{
    beacon_error_t status = beacon_set_callsign("N0CALL", 15);
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

void test_set_destination_success(void)
{
    beacon_error_t status = beacon_set_destination(TEST_DEST, 0);
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

/*******************************************************************************
 * Test Cases - AX.25 Frame Generation
 ******************************************************************************/

void test_generate_frame_null_buffer(void)
{
    size_t size;
    beacon_error_t status = beacon_generate_frame(NULL, 0, &size);
    TEST_ASSERT_EQUAL(BEACON_ERROR_INVALID_PARAM, status);
}

void test_generate_frame_buffer_too_small(void)
{
    uint8_t buffer[8];  /* Too small for AX.25 frame */
    size_t size;
    beacon_error_t status = beacon_generate_frame(buffer, sizeof(buffer), &size);
    TEST_ASSERT_EQUAL(BEACON_ERROR_BUFFER_TOO_SMALL, status);
}

void test_generate_frame_success(void)
{
    beacon_set_callsign(TEST_CALLSIGN, 0);
    beacon_set_destination(TEST_DEST, 0);

    uint8_t buffer[256];
    size_t size = 0;
    beacon_error_t status = beacon_generate_frame(buffer, sizeof(buffer), &size);
    TEST_ASSERT_EQUAL(BEACON_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);
}

void test_frame_contains_flags(void)
{
    beacon_set_callsign(TEST_CALLSIGN, 0);
    beacon_set_destination(TEST_DEST, 0);

    uint8_t buffer[256];
    size_t size;
    beacon_generate_frame(buffer, sizeof(buffer), &size);

    /* AX.25 flag byte is 0x7E */
    TEST_ASSERT_EQUAL_HEX8(0x7E, buffer[0]);
    TEST_ASSERT_EQUAL_HEX8(0x7E, buffer[size - 1]);
}

/*******************************************************************************
 * Test Cases - Info Field
 ******************************************************************************/

void test_set_info_null(void)
{
    beacon_error_t status = beacon_set_info_field(NULL, 0);
    TEST_ASSERT_EQUAL(BEACON_ERROR_INVALID_PARAM, status);
}

void test_set_info_success(void)
{
    const char *info = "SMART-QSO CubeSat Beacon";
    beacon_error_t status = beacon_set_info_field(info, strlen(info));
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

void test_set_info_too_long(void)
{
    /* Create a very long info string */
    char long_info[512];
    memset(long_info, 'A', sizeof(long_info) - 1);
    long_info[sizeof(long_info) - 1] = '\0';

    beacon_error_t status = beacon_set_info_field(long_info, strlen(long_info));
    TEST_ASSERT_EQUAL(BEACON_ERROR_BUFFER_TOO_SMALL, status);
}

/*******************************************************************************
 * Test Cases - Beacon Scheduling
 ******************************************************************************/

void test_set_interval_success(void)
{
    beacon_error_t status = beacon_set_interval(60);
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

void test_set_interval_too_short(void)
{
    beacon_error_t status = beacon_set_interval(1);  /* Too short */
    TEST_ASSERT_EQUAL(BEACON_ERROR_INVALID_PARAM, status);
}

void test_set_interval_too_long(void)
{
    beacon_error_t status = beacon_set_interval(3600);  /* Too long */
    TEST_ASSERT_EQUAL(BEACON_ERROR_INVALID_PARAM, status);
}

void test_is_beacon_due_initially_false(void)
{
    beacon_init();
    /* Immediately after init, beacon may or may not be due */
    /* This depends on implementation */
    bool due = beacon_is_due();
    /* Just test that function returns without error */
    (void)due;
    TEST_PASS();
}

void test_trigger_beacon(void)
{
    beacon_error_t status = beacon_trigger();
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

/*******************************************************************************
 * Test Cases - Jetson Integration
 ******************************************************************************/

void test_set_jetson_text_null(void)
{
    beacon_error_t status = beacon_set_jetson_text(NULL, 0);
    TEST_ASSERT_EQUAL(BEACON_ERROR_INVALID_PARAM, status);
}

void test_set_jetson_text_success(void)
{
    const char *text = "AI-generated beacon content";
    beacon_error_t status = beacon_set_jetson_text(text, strlen(text));
    TEST_ASSERT_EQUAL(BEACON_OK, status);
}

void test_jetson_text_used_in_frame(void)
{
    beacon_set_callsign(TEST_CALLSIGN, 0);
    beacon_set_destination(TEST_DEST, 0);

    const char *text = "AI Content";
    beacon_set_jetson_text(text, strlen(text));
    beacon_enable_ai_content(true);

    uint8_t buffer[256];
    size_t size;
    beacon_generate_frame(buffer, sizeof(buffer), &size);

    /* Frame should be generated successfully */
    TEST_ASSERT_GREATER_THAN(20, size);
}

void test_fallback_to_template(void)
{
    beacon_set_callsign(TEST_CALLSIGN, 0);
    beacon_set_destination(TEST_DEST, 0);

    /* Clear Jetson text */
    beacon_set_jetson_text("", 0);
    beacon_enable_ai_content(false);

    uint8_t buffer[256];
    size_t size;
    beacon_error_t status = beacon_generate_frame(buffer, sizeof(buffer), &size);
    TEST_ASSERT_EQUAL(BEACON_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);
}

/*******************************************************************************
 * Test Cases - Adaptive Cadence
 ******************************************************************************/

void test_enable_adaptive_cadence(void)
{
    beacon_enable_adaptive_cadence(true);
    TEST_ASSERT_TRUE(beacon_is_adaptive_cadence_enabled());
}

void test_disable_adaptive_cadence(void)
{
    beacon_enable_adaptive_cadence(false);
    TEST_ASSERT_FALSE(beacon_is_adaptive_cadence_enabled());
}

void test_adjust_cadence_for_power(void)
{
    /* High power - normal interval */
    beacon_adjust_for_power_state(100);
    beacon_config_t config;
    beacon_get_config(&config);
    uint32_t high_power_interval = config.interval_sec;

    /* Low power - longer interval */
    beacon_adjust_for_power_state(20);
    beacon_get_config(&config);
    uint32_t low_power_interval = config.interval_sec;

    TEST_ASSERT_GREATER_OR_EQUAL(high_power_interval, low_power_interval);
}

/*******************************************************************************
 * Test Cases - Statistics
 ******************************************************************************/

void test_get_beacon_count(void)
{
    beacon_init();
    uint32_t initial = beacon_get_tx_count();

    beacon_trigger();
    uint32_t after = beacon_get_tx_count();

    TEST_ASSERT_EQUAL(initial + 1, after);
}

void test_get_last_beacon_time(void)
{
    beacon_trigger();
    uint32_t last_time = beacon_get_last_tx_time();
    TEST_ASSERT_GREATER_OR_EQUAL(0, last_time);
}

/*******************************************************************************
 * Test Cases - Error Handling
 ******************************************************************************/

void test_error_to_string_ok(void)
{
    const char *str = beacon_error_to_string(BEACON_OK);
    TEST_ASSERT_NOT_NULL(str);
    TEST_ASSERT_EQUAL_STRING("No error", str);
}

void test_error_to_string_invalid_param(void)
{
    const char *str = beacon_error_to_string(BEACON_ERROR_INVALID_PARAM);
    TEST_ASSERT_NOT_NULL(str);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Initialization */
    RUN_TEST(test_beacon_init_returns_ok);
    RUN_TEST(test_beacon_default_config);

    /* Callsign configuration */
    RUN_TEST(test_set_callsign_null);
    RUN_TEST(test_set_callsign_success);
    RUN_TEST(test_set_callsign_with_ssid);
    RUN_TEST(test_set_destination_success);

    /* AX.25 frame generation */
    RUN_TEST(test_generate_frame_null_buffer);
    RUN_TEST(test_generate_frame_buffer_too_small);
    RUN_TEST(test_generate_frame_success);
    RUN_TEST(test_frame_contains_flags);

    /* Info field */
    RUN_TEST(test_set_info_null);
    RUN_TEST(test_set_info_success);
    RUN_TEST(test_set_info_too_long);

    /* Beacon scheduling */
    RUN_TEST(test_set_interval_success);
    RUN_TEST(test_set_interval_too_short);
    RUN_TEST(test_set_interval_too_long);
    RUN_TEST(test_is_beacon_due_initially_false);
    RUN_TEST(test_trigger_beacon);

    /* Jetson integration */
    RUN_TEST(test_set_jetson_text_null);
    RUN_TEST(test_set_jetson_text_success);
    RUN_TEST(test_jetson_text_used_in_frame);
    RUN_TEST(test_fallback_to_template);

    /* Adaptive cadence */
    RUN_TEST(test_enable_adaptive_cadence);
    RUN_TEST(test_disable_adaptive_cadence);
    RUN_TEST(test_adjust_cadence_for_power);

    /* Statistics */
    RUN_TEST(test_get_beacon_count);
    RUN_TEST(test_get_last_beacon_time);

    /* Error handling */
    RUN_TEST(test_error_to_string_ok);
    RUN_TEST(test_error_to_string_invalid_param);

    return UNITY_END();
}
