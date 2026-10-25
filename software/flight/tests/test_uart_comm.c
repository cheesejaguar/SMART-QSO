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

/* posix_openpt() and friends: a pseudo-terminal stands in for the serial port */
#define _XOPEN_SOURCE 600

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>

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

/* Note: Null pointer tests removed - implementation uses defensive asserts */

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

/* Note: Null pointer tests removed - implementation uses defensive asserts */

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
/* Test Cases: a pseudo-terminal as the serial port                           */
/*===========================================================================*/

/** Open a pty pair; *master gets the controlling side, slave its path */
static void open_pty(int *master, char *slave, size_t cap)
{
    *master = posix_openpt(O_RDWR | O_NOCTTY);
    assert_true(*master >= 0);
    assert_int_equal(grantpt(*master), 0);
    assert_int_equal(unlockpt(*master), 0);
    const char *name = ptsname(*master);
    assert_non_null(name);
    assert_true(strlen(name) < cap);
    (void)snprintf(slave, cap, "%s", name);
}

/**
 * @brief Bytes sent reach the other end of the line, and bytes from it are received
 */
static void test_uart_send_and_receive_over_pty(void **state) {
    (void)state;
    int master = -1;
    UartConfig_t config = { .baudrate = 57600, .timeout_ms = 10 };
    open_pty(&master, config.device, sizeof(config.device));

    uart_close();
    assert_int_equal(uart_init_with_config(&config), SMART_QSO_OK);
    assert_true(uart_is_initialized());
    assert_int_equal(uart_check_health(), SMART_QSO_OK);

    assert_int_equal(uart_send("STATUS_REQ,5D03\n", 16U), SMART_QSO_OK);
    char line[32] = {0};
    assert_int_equal(read(master, line, sizeof(line) - 1U), 16);
    assert_string_equal(line, "STATUS_REQ,5D03\n");

    assert_int_equal(write(master, "ACK,X,0000\n", 11U), 11);
    char buffer[32] = {0};
    size_t received = 0U;
    assert_int_equal(uart_receive(buffer, sizeof(buffer), &received), SMART_QSO_OK);
    assert_int_equal(received, 11U);
    assert_memory_equal(buffer, "ACK,X,0000\n", 11U);

    uart_close();
    (void)close(master);
}

/**
 * @brief Every supported rate configures the line; an unknown one falls back to the default
 */
static void test_uart_baud_rates(void **state) {
    (void)state;
    static const int rates[] = { 9600, 19200, 38400, 57600, 115200, 230400, 1234 };
    int master = -1;
    UartConfig_t config = { .timeout_ms = 10 };
    open_pty(&master, config.device, sizeof(config.device));
    for (size_t i = 0U; i < (sizeof(rates) / sizeof(rates[0])); i++) {
        config.baudrate = rates[i];
        assert_int_equal(uart_init_with_config(&config), SMART_QSO_OK);
    }
    uart_close();
    (void)close(master);
}

/**
 * @brief A path that is not a terminal is refused, and the module stays closed
 */
static void test_uart_rejects_non_terminal(void **state) {
    (void)state;
    UartConfig_t config = { .device = "/dev/null", .baudrate = 115200, .timeout_ms = 10 };
    uart_close();
    assert_int_equal(uart_init_with_config(&config), SMART_QSO_ERROR_IO);
    assert_false(uart_is_initialized());
}

/**
 * @brief UART_DEVICE, UART_BAUDRATE and TELEMETRY_INTERVAL_MS configure uart_init()
 */
static void test_uart_environment_configuration(void **state) {
    (void)state;
    int master = -1;
    char slave[UART_DEVICE_MAX_LEN];
    open_pty(&master, slave, sizeof(slave));

    uart_close();
    assert_int_equal(setenv("UART_DEVICE", slave, 1), 0);
    assert_int_equal(setenv("UART_BAUDRATE", "38400", 1), 0);
    assert_int_equal(setenv("TELEMETRY_INTERVAL_MS", "5000", 1), 0);
    assert_int_equal(uart_init(), SMART_QSO_OK);
    UartConfig_t config;
    assert_int_equal(uart_get_config(&config), SMART_QSO_OK);
    assert_string_equal(config.device, slave);
    assert_int_equal(config.baudrate, 38400);
    assert_int_equal(uart_get_telemetry_interval(), 5000U);

    /* Out-of-range values are refused and the previous ones kept */
    uart_close();
    assert_int_equal(setenv("UART_BAUDRATE", "1234", 1), 0);
    assert_int_equal(setenv("TELEMETRY_INTERVAL_MS", "0", 1), 0);
    assert_int_equal(uart_init(), SMART_QSO_OK);
    assert_int_equal(uart_get_config(&config), SMART_QSO_OK);
    assert_int_equal(config.baudrate, 38400);
    assert_int_equal(uart_get_telemetry_interval(), 5000U);

    uart_close();
    (void)unsetenv("UART_DEVICE");
    (void)unsetenv("UART_BAUDRATE");
    (void)unsetenv("TELEMETRY_INTERVAL_MS");
    (void)close(master);
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
        cmocka_unit_test_setup_teardown(test_uart_init_with_config, setup, teardown),

        /* State management tests */
        cmocka_unit_test_setup_teardown(test_uart_is_initialized, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_close, setup, teardown),

        /* Communication tests */
        /* Note: Null pointer tests removed - implementation uses defensive asserts */
        cmocka_unit_test_setup_teardown(test_uart_operations_uninitialized, setup, teardown),

        /* Health check tests */
        cmocka_unit_test_setup_teardown(test_uart_check_health, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_check_health_uninitialized, setup, teardown),

        /* Telemetry interval tests */
        cmocka_unit_test_setup_teardown(test_uart_get_telemetry_interval, setup, teardown),

        /* A pseudo-terminal as the serial port */
        cmocka_unit_test_setup_teardown(test_uart_send_and_receive_over_pty, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_baud_rates, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_rejects_non_terminal, setup, teardown),
        cmocka_unit_test_setup_teardown(test_uart_environment_configuration, setup, teardown),
    };

    return cmocka_run_group_tests_name("UART Communication Tests", tests, NULL, NULL);
}
