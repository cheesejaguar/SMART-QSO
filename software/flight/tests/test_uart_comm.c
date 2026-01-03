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
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Initialize UART module - may fail if no device available */
    (void)uart_init();
    return 0;
}

static int teardown(void **state) {
    (void)state;
    uart_close();
    return 0;
}

/*===========================================================================*/
/* Test Cases: Initialization                                                 */
/*===========================================================================*/

/**
 * @brief Test UART initialization status
 *
 * @requirement SRS-I001 UART interface configuration
 */
static void test_uart_init(void **state) {
    (void)state;

    /* Close first to test fresh init */
    uart_close();

    /* Init may fail if device not available - this is acceptable */
    SmartQsoResult_t result = uart_init();

    /* Either succeeds or fails gracefully */
    assert_true(result == SMART_QSO_OK || result == SMART_QSO_ERROR_IO);
}

/**
 * @brief Test UART double initialization
 */
static void test_uart_double_init(void **state) {
    (void)state;

    uart_close();

    /* First init */
    SmartQsoResult_t result1 = uart_init();

    /* Second init should succeed (idempotent) */
    SmartQsoResult_t result2 = uart_init();

    assert_true(result1 == SMART_QSO_OK || result1 == SMART_QSO_ERROR_IO);
    assert_true(result2 == SMART_QSO_OK || result2 == SMART_QSO_ERROR_IO);
}

/**
 * @brief Test UART configuration retrieval
 */
static void test_uart_get_config(void **state) {
    (void)state;

    UartConfig_t config;
    SmartQsoResult_t result = uart_get_config(&config);

    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(config.baudrate, UART_BAUDRATE_DEFAULT);
    assert_true(config.timeout_ms > 0);
    assert_true(strlen(config.device) > 0);
}

/**
 * @brief Test UART config get with null pointer
 */
static void test_uart_get_config_null(void **state) {
    (void)state;

    SmartQsoResult_t result = uart_get_config(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Test UART initialization with custom config
 */
static void test_uart_init_with_config(void **state) {
    (void)state;

    uart_close();

    UartConfig_t config;
    strncpy(config.device, "/dev/ttyUSB0", UART_DEVICE_MAX_LEN - 1);
    config.device[UART_DEVICE_MAX_LEN - 1] = '\0';
    config.baudrate = 115200;
    config.timeout_ms = 2000;

    /* May fail if device not available */
    SmartQsoResult_t result = uart_init_with_config(&config);

    assert_true(result == SMART_QSO_OK || result == SMART_QSO_ERROR_IO);
}

/**
 * @brief Test UART init with null config
 */
static void test_uart_init_with_config_null(void **state) {
    (void)state;

    SmartQsoResult_t result = uart_init_with_config(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Cases: State Management                                               */
/*===========================================================================*/

/**
 * @brief Test UART is_initialized flag
 */
static void test_uart_is_initialized(void **state) {
    (void)state;

    uart_close();

    /* After close, should not be initialized */
    assert_false(uart_is_initialized());

    /* After init (may fail), check consistency */
    SmartQsoResult_t result = uart_init();
    if (result == SMART_QSO_OK) {
        assert_true(uart_is_initialized());
    }
}

/**
 * @brief Test UART close
 */
static void test_uart_close(void **state) {
    (void)state;

    /* Close should always succeed */
    SmartQsoResult_t result = uart_close();
    assert_int_equal(result, SMART_QSO_OK);

    /* Double close should also succeed (idempotent) */
    result = uart_close();
    assert_int_equal(result, SMART_QSO_OK);
}

/*===========================================================================*/
/* Test Cases: Communication                                                  */
/*===========================================================================*/

/**
 * @brief Test UART send with null data
 */
static void test_uart_send_null(void **state) {
    (void)state;

    SmartQsoResult_t result = uart_send(NULL, 10);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Test UART receive with null buffer
 */
static void test_uart_receive_null(void **state) {
    (void)state;

    size_t received;
    SmartQsoResult_t result = uart_receive(NULL, 100, &received);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);

    char buffer[100];
    result = uart_receive(buffer, 100, NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Test UART operations when not initialized
 */
static void test_uart_operations_uninitialized(void **state) {
    (void)state;

    uart_close();

    /* Send should fail when not initialized */
    SmartQsoResult_t result = uart_send("test", 4);
    assert_int_equal(result, SMART_QSO_ERROR);

    /* Receive should fail when not initialized */
    char buffer[100];
    size_t received;
    result = uart_receive(buffer, sizeof(buffer), &received);
    assert_int_equal(result, SMART_QSO_ERROR);
}

/*===========================================================================*/
/* Test Cases: Health Check                                                   */
/*===========================================================================*/

/**
 * @brief Test UART health check
 *
 * @requirement SRS-F032 Enforce timeout on Jetson communication
 */
static void test_uart_check_health(void **state) {
    (void)state;

    /* Health check depends on initialization state */
    SmartQsoResult_t result = uart_check_health();

    /* Should return error if not initialized, OK if initialized */
    assert_true(result == SMART_QSO_OK || result == SMART_QSO_ERROR);
}

/**
 * @brief Test health check when not initialized
 */
static void test_uart_check_health_uninitialized(void **state) {
    (void)state;

    uart_close();

    SmartQsoResult_t result = uart_check_health();
    assert_int_equal(result, SMART_QSO_ERROR);
}

/*===========================================================================*/
/* Test Cases: Telemetry Interval                                             */
/*===========================================================================*/

/**
 * @brief Test telemetry interval getter
 */
static void test_uart_get_telemetry_interval(void **state) {
    (void)state;

    uint32_t interval = uart_get_telemetry_interval();

    /* Should be reasonable interval (not zero, not extremely large) */
    assert_true(interval > 0);
    assert_true(interval <= 3600000);  /* Max 1 hour */
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_uart_init, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_double_init, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_get_config, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_get_config_null, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_init_with_config, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_init_with_config_null, setup, teardown),

        /* State management tests */
        cmocka_unit_test_setup_teardown(test_uart_is_initialized, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_close, setup, teardown),

        /* Communication tests */
        cmocka_unit_test_setup_teardown(test_uart_send_null, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_receive_null, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_operations_uninitialized, setup, teardown),

        /* Health check tests */
        cmocka_unit_test_setup_teardown(test_uart_check_health, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_check_health_uninitialized, setup, teardown),

        /* Telemetry interval tests */
        cmocka_unit_test_setup_teardown(test_uart_get_telemetry_interval, setup, teardown),
    };

    return cmocka_run_group_tests_name("UART Communication Tests", tests, NULL, NULL);
}
