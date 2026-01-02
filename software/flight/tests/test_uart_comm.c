/**
 * @file test_uart_comm.c
 * @brief Unit tests for UART Communication module
 *
 * Tests Jetson communication per SRS-F030 through SRS-F035.
 *
 * @requirement SRS-F030 Send health telemetry to Jetson via UART
 * @requirement SRS-F031 Receive beacon text from Jetson via UART
 * @requirement SRS-F032 Enforce timeout on Jetson communication
 * @requirement SRS-F033 Disable payload on communication timeout
 * @requirement SRS-F034 Validate received beacon text
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

/* Include the module under test */
#include "smart_qso.h"
#include "uart_comm.h"

/*===========================================================================*/
/* Test Data                                                                  */
/*===========================================================================*/

/* Sample telemetry message */
static const char *sample_telemetry =
    "TELEMETRY,1704153600000,SUNLIT,0.75,2,3.0,BV=8.1V,BT=22.5C,BUSV=5.0V";

/* Sample beacon response */
static const char *sample_beacon_response =
    "BEACON_RSP,42,Hello from SMART-QSO AI!,C3D4";

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Initialize UART module */
    SmartQsoResult_t result = uart_init();
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
 * @brief Test UART initialization
 *
 * @requirement SRS-I001 UART interface configuration
 */
static void test_uart_init(void **state) {
    (void)state;

    /* Already initialized in setup */
    UartConfig_t config;
    SmartQsoResult_t result = uart_get_config(&config);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(config.baud_rate, UART_BAUD_RATE);
    assert_int_equal(config.data_bits, 8);
    assert_int_equal(config.stop_bits, 1);
}

/*===========================================================================*/
/* Test Cases: CRC Calculation                                                */
/*===========================================================================*/

/**
 * @brief Test CRC-16 CCITT calculation
 *
 * @requirement SRS-I005 Message format shall include CRC
 */
static void test_uart_crc16(void **state) {
    (void)state;

    /* Known test vector: "123456789" should give specific CRC */
    const char *test_data = "123456789";
    uint16_t crc = uart_calculate_crc16(test_data, strlen(test_data));

    /* CRC-16 CCITT of "123456789" is 0x29B1 */
    assert_int_equal(crc, 0x29B1);
}

/**
 * @brief Test CRC-16 with empty data
 */
static void test_uart_crc16_empty(void **state) {
    (void)state;

    uint16_t crc = uart_calculate_crc16("", 0);
    /* CRC of empty data with init 0xFFFF and no input */
    assert_int_equal(crc, 0xFFFF);
}

/**
 * @brief Test CRC-16 detects changes
 */
static void test_uart_crc16_change_detection(void **state) {
    (void)state;

    uint8_t data1[] = {0x01, 0x02, 0x03};
    uint8_t data2[] = {0x01, 0x02, 0x04};  /* Single byte different */

    uint16_t crc1 = uart_calculate_crc16(data1, sizeof(data1));
    uint16_t crc2 = uart_calculate_crc16(data2, sizeof(data2));

    assert_int_not_equal(crc1, crc2);
}

/*===========================================================================*/
/* Test Cases: Message Formatting                                             */
/*===========================================================================*/

/**
 * @brief Test telemetry message formatting
 *
 * @requirement SRS-I003 Telemetry messages shall be CSV format
 */
static void test_uart_format_telemetry(void **state) {
    (void)state;

    char buffer[512];
    TelemetryData_t telem = {
        .timestamp_ms = 1704153600000ULL,
        .sunlit = true,
        .soc = 0.75,
        .power_mode = 2,
        .power_limit = 3.0,
        .battery_voltage = 8.1,
        .battery_temp = 22.5,
        .bus_voltage = 5.0
    };

    SmartQsoResult_t result = uart_format_telemetry(&telem, buffer, sizeof(buffer));

    assert_int_equal(result, SMART_QSO_OK);
    assert_non_null(strstr(buffer, "TELEMETRY"));
    assert_non_null(strstr(buffer, "SUNLIT"));
    assert_non_null(strstr(buffer, "BV="));
}

/**
 * @brief Test telemetry formatting with null pointer
 */
static void test_uart_format_telemetry_null(void **state) {
    (void)state;

    char buffer[512];
    SmartQsoResult_t result = uart_format_telemetry(NULL, buffer, sizeof(buffer));
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);

    TelemetryData_t telem = {0};
    result = uart_format_telemetry(&telem, NULL, sizeof(buffer));
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Test telemetry formatting with small buffer
 */
static void test_uart_format_telemetry_small_buffer(void **state) {
    (void)state;

    char buffer[16];  /* Too small */
    TelemetryData_t telem = {0};

    SmartQsoResult_t result = uart_format_telemetry(&telem, buffer, sizeof(buffer));

    /* Should indicate buffer too small or truncate safely */
    assert_true(result == SMART_QSO_ERROR || strlen(buffer) < sizeof(buffer));
}

/*===========================================================================*/
/* Test Cases: Message Parsing                                                */
/*===========================================================================*/

/**
 * @brief Test beacon response parsing
 *
 * @requirement SRS-F034 Validate received beacon text
 */
static void test_uart_parse_beacon_response(void **state) {
    (void)state;

    BeaconResponse_t response;
    SmartQsoResult_t result = uart_parse_beacon_response(sample_beacon_response, &response);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(response.seq, 42);
    assert_string_equal(response.text, "Hello from SMART-QSO AI!");
}

/**
 * @brief Test parsing invalid beacon response
 */
static void test_uart_parse_beacon_invalid(void **state) {
    (void)state;

    BeaconResponse_t response;

    /* Missing fields */
    SmartQsoResult_t result = uart_parse_beacon_response("BEACON_RSP,42", &response);
    assert_int_not_equal(result, SMART_QSO_OK);

    /* Wrong message type */
    result = uart_parse_beacon_response("TELEMETRY,123", &response);
    assert_int_not_equal(result, SMART_QSO_OK);
}

/**
 * @brief Test parsing with null pointers
 */
static void test_uart_parse_beacon_null(void **state) {
    (void)state;

    BeaconResponse_t response;

    SmartQsoResult_t result = uart_parse_beacon_response(NULL, &response);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);

    result = uart_parse_beacon_response(sample_beacon_response, NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Cases: Beacon Text Validation                                         */
/*===========================================================================*/

/**
 * @brief Test valid ASCII beacon text
 *
 * @requirement SRS-F034 Validate received beacon text (ASCII, length)
 */
static void test_uart_validate_beacon_ascii(void **state) {
    (void)state;

    const char *valid_text = "Hello from SMART-QSO! 73 de K1ABC";
    bool result = uart_validate_beacon_text(valid_text, strlen(valid_text));

    assert_true(result);
}

/**
 * @brief Test beacon text with non-ASCII characters
 */
static void test_uart_validate_beacon_non_ascii(void **state) {
    (void)state;

    /* String with non-printable character */
    char text[] = "Hello\x01World";
    bool result = uart_validate_beacon_text(text, strlen(text));

    assert_false(result);
}

/**
 * @brief Test beacon text length validation
 *
 * @requirement SRS-F025 Limit beacon info field to 256 bytes
 */
static void test_uart_validate_beacon_length(void **state) {
    (void)state;

    /* Create text that exceeds maximum */
    char long_text[512];
    memset(long_text, 'A', sizeof(long_text));
    long_text[sizeof(long_text) - 1] = '\0';

    bool result = uart_validate_beacon_text(long_text, UART_MAX_BEACON_LEN + 10);

    assert_false(result);
}

/**
 * @brief Test empty beacon text
 */
static void test_uart_validate_beacon_empty(void **state) {
    (void)state;

    bool result = uart_validate_beacon_text("", 0);

    /* Empty text should be invalid */
    assert_false(result);
}

/*===========================================================================*/
/* Test Cases: Communication Health                                           */
/*===========================================================================*/

/**
 * @brief Test communication health check
 *
 * @requirement SRS-F032 Enforce timeout on Jetson communication
 */
static void test_uart_check_health(void **state) {
    (void)state;

    SmartQsoResult_t result = uart_check_health();

    /* Health check should return OK or specific error */
    assert_true(result == SMART_QSO_OK || result == SMART_QSO_ERROR_TIMEOUT);
}

/**
 * @brief Test communication statistics
 */
static void test_uart_get_stats(void **state) {
    (void)state;

    UartStats_t stats;
    SmartQsoResult_t result = uart_get_stats(&stats);

    assert_int_equal(result, SMART_QSO_OK);
    /* Stats should be initialized */
    assert_true(stats.tx_count >= 0);
    assert_true(stats.rx_count >= 0);
}

/**
 * @brief Test stats with null pointer
 */
static void test_uart_get_stats_null(void **state) {
    (void)state;

    SmartQsoResult_t result = uart_get_stats(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Cases: Status Message Parsing                                         */
/*===========================================================================*/

/**
 * @brief Test parsing Jetson status message
 */
static void test_uart_parse_status(void **state) {
    (void)state;

    const char *status_msg = "STATUS,45.5,42.0,4096,12345,A1B2";

    JetsonStatus_t status;
    SmartQsoResult_t result = uart_parse_status(status_msg, &status);

    assert_int_equal(result, SMART_QSO_OK);
    assert_true(status.cpu_temp >= 45.0 && status.cpu_temp <= 46.0);
    assert_true(status.gpu_temp >= 41.0 && status.gpu_temp <= 43.0);
    assert_int_equal(status.mem_used_mb, 4096);
    assert_int_equal(status.uptime_s, 12345);
}

/**
 * @brief Test parsing invalid status message
 */
static void test_uart_parse_status_invalid(void **state) {
    (void)state;

    JetsonStatus_t status;

    SmartQsoResult_t result = uart_parse_status("STATUS,invalid", &status);
    assert_int_not_equal(result, SMART_QSO_OK);
}

/*===========================================================================*/
/* Test Cases: Retry Logic                                                    */
/*===========================================================================*/

/**
 * @brief Test retry configuration
 */
static void test_uart_retry_config(void **state) {
    (void)state;

    UartConfig_t config;
    uart_get_config(&config);

    assert_int_equal(config.max_retries, UART_MAX_RETRIES);
    assert_int_equal(config.timeout_ms, UART_TIMEOUT_MS);
}

/*===========================================================================*/
/* Test Cases: Eclipse/Sunlit Status                                          */
/*===========================================================================*/

/**
 * @brief Test telemetry with eclipse status
 */
static void test_uart_telemetry_eclipse(void **state) {
    (void)state;

    char buffer[512];
    TelemetryData_t telem = {
        .timestamp_ms = 1704153600000ULL,
        .sunlit = false,  /* Eclipse */
        .soc = 0.50,
        .power_mode = 1,
        .power_limit = 1.5,
        .battery_voltage = 7.5,
        .battery_temp = 15.0,
        .bus_voltage = 5.0
    };

    SmartQsoResult_t result = uart_format_telemetry(&telem, buffer, sizeof(buffer));

    assert_int_equal(result, SMART_QSO_OK);
    assert_non_null(strstr(buffer, "ECLIPSE"));
    assert_null(strstr(buffer, "SUNLIT"));
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_uart_init, setup, teardown),

        /* CRC tests */
        cmocka_unit_test_setup_teardown(test_uart_crc16, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_crc16_empty, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_crc16_change_detection, setup, teardown),

        /* Message formatting tests */
        cmocka_unit_test_setup_teardown(test_uart_format_telemetry, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_format_telemetry_null, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_format_telemetry_small_buffer, setup, teardown),

        /* Message parsing tests */
        cmocka_unit_test_setup_teardown(test_uart_parse_beacon_response, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_parse_beacon_invalid, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_parse_beacon_null, setup, teardown),

        /* Beacon validation tests */
        cmocka_unit_test_setup_teardown(test_uart_validate_beacon_ascii, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_validate_beacon_non_ascii, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_validate_beacon_length, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_validate_beacon_empty, setup, teardown),

        /* Health check tests */
        cmocka_unit_test_setup_teardown(test_uart_check_health, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_get_stats, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_get_stats_null, setup, teardown),

        /* Status parsing tests */
        cmocka_unit_test_setup_teardown(test_uart_parse_status, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_parse_status_invalid, setup, teardown),

        /* Retry config tests */
        cmocka_unit_test_setup_teardown(test_uart_retry_config, setup, teardown),

        /* Eclipse/sunlit tests */
        cmocka_unit_test_setup_teardown(test_uart_telemetry_eclipse, setup, teardown),
    };

    return cmocka_run_group_tests_name("UART Communication Tests", tests, NULL, NULL);
}
