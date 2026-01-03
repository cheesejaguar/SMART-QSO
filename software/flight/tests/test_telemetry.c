/**
 * @file test_telemetry.c
 * @brief Unit tests for telemetry module
 *
 * Tests telemetry packet formatting, registration, sampling,
 * circular buffer storage, and downlink interface.
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#include "unity.h"
#include "telemetry.h"
#include <string.h>

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

void setUp(void)
{
    telemetry_init();
}

void tearDown(void)
{
    /* Cleanup after each test */
}

/*******************************************************************************
 * Test Cases - Initialization
 ******************************************************************************/

void test_telemetry_init_returns_ok(void)
{
    telem_error_t status = telemetry_init();
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

void test_telemetry_point_count_zero_after_init(void)
{
    telemetry_init();
    TEST_ASSERT_EQUAL(0, telemetry_get_point_count());
}

/*******************************************************************************
 * Test Cases - Point Registration
 ******************************************************************************/

void test_register_point_null_name(void)
{
    telem_point_t point = {
        .name = NULL,
        .type = TELEM_TYPE_UINT16,
        .period_ms = 1000
    };
    telem_error_t status = telemetry_register_point(&point);
    TEST_ASSERT_EQUAL(TELEM_ERROR_INVALID_PARAM, status);
}

void test_register_point_success(void)
{
    telem_point_t point = {
        .name = "voltage",
        .type = TELEM_TYPE_UINT16,
        .period_ms = 1000
    };
    telem_error_t status = telemetry_register_point(&point);
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

void test_register_multiple_points(void)
{
    telem_point_t points[] = {
        { .name = "voltage", .type = TELEM_TYPE_UINT16, .period_ms = 1000 },
        { .name = "current", .type = TELEM_TYPE_INT16, .period_ms = 1000 },
        { .name = "temp", .type = TELEM_TYPE_FLOAT, .period_ms = 5000 }
    };

    for (int i = 0; i < 3; i++) {
        telem_error_t status = telemetry_register_point(&points[i]);
        TEST_ASSERT_EQUAL(TELEM_OK, status);
    }

    TEST_ASSERT_EQUAL(3, telemetry_get_point_count());
}

/*******************************************************************************
 * Test Cases - Value Updates
 ******************************************************************************/

void test_update_value_not_found(void)
{
    uint16_t value = 1234;
    telem_error_t status = telemetry_update_value("nonexistent", &value, sizeof(value));
    TEST_ASSERT_EQUAL(TELEM_ERROR_NOT_FOUND, status);
}

void test_update_value_success(void)
{
    telem_point_t point = {
        .name = "voltage",
        .type = TELEM_TYPE_UINT16,
        .period_ms = 1000
    };
    telemetry_register_point(&point);

    uint16_t value = 3300;
    telem_error_t status = telemetry_update_value("voltage", &value, sizeof(value));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

void test_get_value_success(void)
{
    telem_point_t point = {
        .name = "voltage",
        .type = TELEM_TYPE_UINT16,
        .period_ms = 1000
    };
    telemetry_register_point(&point);

    uint16_t set_value = 3300;
    telemetry_update_value("voltage", &set_value, sizeof(set_value));

    uint16_t get_value = 0;
    telem_error_t status = telemetry_get_value("voltage", &get_value, sizeof(get_value));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
    TEST_ASSERT_EQUAL(3300, get_value);
}

/*******************************************************************************
 * Test Cases - Packet Generation
 ******************************************************************************/

void test_generate_packet_null_buffer(void)
{
    size_t size;
    telem_error_t status = telemetry_generate_packet(NULL, 100, &size);
    TEST_ASSERT_EQUAL(TELEM_ERROR_INVALID_PARAM, status);
}

void test_generate_packet_buffer_too_small(void)
{
    /* Register several points */
    telem_point_t point = {
        .name = "voltage",
        .type = TELEM_TYPE_UINT16,
        .period_ms = 1000
    };
    telemetry_register_point(&point);

    uint8_t buffer[2];  /* Too small */
    size_t size;
    telem_error_t status = telemetry_generate_packet(buffer, sizeof(buffer), &size);
    TEST_ASSERT_EQUAL(TELEM_ERROR_BUFFER_TOO_SMALL, status);
}

void test_generate_packet_success(void)
{
    telem_point_t point = {
        .name = "voltage",
        .type = TELEM_TYPE_UINT16,
        .period_ms = 1000
    };
    telemetry_register_point(&point);

    uint16_t value = 3300;
    telemetry_update_value("voltage", &value, sizeof(value));

    uint8_t buffer[256];
    size_t size = 0;
    telem_error_t status = telemetry_generate_packet(buffer, sizeof(buffer), &size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);
    TEST_ASSERT_GREATER_THAN(0, size);
}

/*******************************************************************************
 * Test Cases - Circular Buffer
 ******************************************************************************/

void test_store_packet_success(void)
{
    uint8_t packet[] = {0x01, 0x02, 0x03, 0x04};
    telem_error_t status = telemetry_store_packet(packet, sizeof(packet));
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

void test_get_stored_packet_count(void)
{
    telemetry_init();  /* Reset buffer */

    uint8_t packet[] = {0x01, 0x02, 0x03, 0x04};
    telemetry_store_packet(packet, sizeof(packet));
    telemetry_store_packet(packet, sizeof(packet));

    TEST_ASSERT_EQUAL(2, telemetry_get_stored_count());
}

void test_retrieve_packet_success(void)
{
    uint8_t packet[] = {0xAA, 0xBB, 0xCC, 0xDD};
    telemetry_store_packet(packet, sizeof(packet));

    uint8_t retrieved[4];
    size_t size;
    telem_error_t status = telemetry_retrieve_packet(retrieved, sizeof(retrieved), &size);
    TEST_ASSERT_EQUAL(TELEM_OK, status);
    TEST_ASSERT_EQUAL_MEMORY(packet, retrieved, sizeof(packet));
}

void test_retrieve_empty_buffer(void)
{
    telemetry_init();  /* Reset buffer */

    uint8_t buffer[32];
    size_t size;
    telem_error_t status = telemetry_retrieve_packet(buffer, sizeof(buffer), &size);
    TEST_ASSERT_EQUAL(TELEM_ERROR_BUFFER_EMPTY, status);
}

/*******************************************************************************
 * Test Cases - Sampling
 ******************************************************************************/

void test_sample_all_points(void)
{
    telem_point_t points[] = {
        { .name = "voltage", .type = TELEM_TYPE_UINT16, .period_ms = 100 },
        { .name = "current", .type = TELEM_TYPE_INT16, .period_ms = 100 }
    };

    for (int i = 0; i < 2; i++) {
        telemetry_register_point(&points[i]);
    }

    telem_error_t status = telemetry_sample_all();
    TEST_ASSERT_EQUAL(TELEM_OK, status);
}

/*******************************************************************************
 * Test Cases - Compression
 ******************************************************************************/

void test_compression_disabled(void)
{
    TEST_ASSERT_FALSE(telemetry_is_compression_enabled());
}

void test_enable_compression(void)
{
    telemetry_set_compression(true);
    TEST_ASSERT_TRUE(telemetry_is_compression_enabled());
    telemetry_set_compression(false);  /* Reset */
}

/*******************************************************************************
 * Test Cases - Error Handling
 ******************************************************************************/

void test_error_to_string_ok(void)
{
    const char *str = telemetry_error_to_string(TELEM_OK);
    TEST_ASSERT_NOT_NULL(str);
    TEST_ASSERT_EQUAL_STRING("No error", str);
}

void test_error_to_string_invalid_param(void)
{
    const char *str = telemetry_error_to_string(TELEM_ERROR_INVALID_PARAM);
    TEST_ASSERT_NOT_NULL(str);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    UNITY_BEGIN();

    /* Initialization */
    RUN_TEST(test_telemetry_init_returns_ok);
    RUN_TEST(test_telemetry_point_count_zero_after_init);

    /* Point registration */
    RUN_TEST(test_register_point_null_name);
    RUN_TEST(test_register_point_success);
    RUN_TEST(test_register_multiple_points);

    /* Value updates */
    RUN_TEST(test_update_value_not_found);
    RUN_TEST(test_update_value_success);
    RUN_TEST(test_get_value_success);

    /* Packet generation */
    RUN_TEST(test_generate_packet_null_buffer);
    RUN_TEST(test_generate_packet_buffer_too_small);
    RUN_TEST(test_generate_packet_success);

    /* Circular buffer */
    RUN_TEST(test_store_packet_success);
    RUN_TEST(test_get_stored_packet_count);
    RUN_TEST(test_retrieve_packet_success);
    RUN_TEST(test_retrieve_empty_buffer);

    /* Sampling */
    RUN_TEST(test_sample_all_points);

    /* Compression */
    RUN_TEST(test_compression_disabled);
    RUN_TEST(test_enable_compression);

    /* Error handling */
    RUN_TEST(test_error_to_string_ok);
    RUN_TEST(test_error_to_string_invalid_param);

    return UNITY_END();
}
