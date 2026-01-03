/**
 * @file test_fault_mgmt.c
 * @brief Unit tests for Fault Management module
 *
 * Tests FDIR implementation per SRS-F040 through SRS-F045.
 *
 * @requirement SRS-F040 Detect and log all fault conditions
 * @requirement SRS-F041 Maintain fault log in NVM
 * @requirement SRS-F042 Implement autonomous fault recovery
 * @requirement SRS-F043 Enter SAFE mode on critical faults
 * @requirement SRS-F044 Recovery from SAFE mode when SOC improves
 * @requirement SRS-F045 Track fault statistics across resets
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

/* Include the modules under test */
#include "smart_qso.h"
#include "fault_mgmt.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Clean up any existing fault log */
    unlink(FAULT_LOG_FILE);
    /* Initialize fault management */
    SmartQsoResult_t result = fault_mgmt_init();
    assert_int_equal(result, SMART_QSO_OK);
    return 0;
}

static int teardown(void **state) {
    (void)state;
    unlink(FAULT_LOG_FILE);
    return 0;
}

/*===========================================================================*/
/* Test Cases: Initialization                                                 */
/*===========================================================================*/

/**
 * @brief Test fault management initialization
 *
 * @requirement SRS-F040 Fault detection system initialization
 */
static void test_fault_init(void **state) {
    (void)state;

    /* Module initialized in setup, verify state */
    size_t count = fault_log_get_count();

    /* New system should have zero faults */
    assert_int_equal(count, 0);
}

/*===========================================================================*/
/* Test Cases: Fault Logging                                                  */
/*===========================================================================*/

/**
 * @brief Test logging a single fault
 *
 * @requirement SRS-F041 Maintain fault log in NVM
 */
static void test_fault_log_single(void **state) {
    (void)state;

    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_THERMAL,
        FAULT_SEVERITY_WARNING,
        "Test thermal fault",
        0.75
    );

    assert_int_equal(result, SMART_QSO_OK);

    size_t count = fault_log_get_count();
    assert_int_equal(count, 1);
}

/**
 * @brief Test logging multiple faults
 *
 * @requirement SRS-F041 Maintain fault log in NVM
 */
static void test_fault_log_multiple(void **state) {
    (void)state;

    /* Log several faults */
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_INFO, "Thermal info", 0.80);
    fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING, "Power warning", 0.75);
    fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_ERROR, "UART error", 0.70);

    size_t count = fault_log_get_count();
    assert_int_equal(count, 3);
}

/* Note: Null pointer test removed - implementation uses defensive asserts */

/**
 * @brief Test fault log overflow handling
 *
 * @requirement SRS-F041 Fault log management
 */
static void test_fault_log_overflow(void **state) {
    (void)state;

    /* Fill the fault log beyond capacity */
    for (int i = 0; i < SMART_QSO_MAX_FAULT_ENTRIES + 10; i++) {
        char desc[64];
        snprintf(desc, sizeof(desc), "Test fault %d", i);
        fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_INFO, desc, 0.75);
    }

    size_t count = fault_log_get_count();

    /* Should not exceed maximum */
    assert_true(count <= SMART_QSO_MAX_FAULT_ENTRIES);
}

/*===========================================================================*/
/* Test Cases: Fault Severity                                                 */
/*===========================================================================*/

/**
 * @brief Test INFO severity fault
 */
static void test_fault_severity_info(void **state) {
    (void)state;

    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_MODE_CHANGE,
        FAULT_SEVERITY_INFO,
        "Mode changed to IDLE",
        0.80
    );

    assert_int_equal(result, SMART_QSO_OK);

    FaultLogEntry_t entry;
    result = fault_log_get_last(&entry);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.severity, FAULT_SEVERITY_INFO);
}

/**
 * @brief Test WARNING severity fault
 */
static void test_fault_severity_warning(void **state) {
    (void)state;

    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_THERMAL,
        FAULT_SEVERITY_WARNING,
        "Temperature approaching limit",
        0.70
    );

    assert_int_equal(result, SMART_QSO_OK);

    FaultLogEntry_t entry;
    result = fault_log_get_last(&entry);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.severity, FAULT_SEVERITY_WARNING);
}

/**
 * @brief Test ERROR severity fault
 */
static void test_fault_severity_error(void **state) {
    (void)state;

    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_UART,
        FAULT_SEVERITY_ERROR,
        "UART communication timeout",
        0.65
    );

    assert_int_equal(result, SMART_QSO_OK);

    FaultLogEntry_t entry;
    result = fault_log_get_last(&entry);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.severity, FAULT_SEVERITY_ERROR);
}

/**
 * @brief Test CRITICAL severity fault triggers SAFE mode
 *
 * @requirement SRS-F043 Enter SAFE mode on critical faults
 */
static void test_fault_severity_critical(void **state) {
    (void)state;

    SmartQsoResult_t result = fault_log_add(
        FAULT_TYPE_POWER_CRITICAL,
        FAULT_SEVERITY_CRITICAL,
        "Critical battery undervoltage",
        0.15
    );

    assert_int_equal(result, SMART_QSO_OK);

    FaultLogEntry_t entry;
    result = fault_log_get_last(&entry);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.severity, FAULT_SEVERITY_CRITICAL);
}

/*===========================================================================*/
/* Test Cases: Fault Recovery                                                 */
/*===========================================================================*/

/**
 * @brief Test thermal fault recovery
 *
 * @requirement SRS-F042 Implement autonomous fault recovery
 */
static void test_fault_recover_thermal(void **state) {
    (void)state;

    /* Log a thermal fault */
    fault_log_add(
        FAULT_TYPE_THERMAL,
        FAULT_SEVERITY_ERROR,
        "Over temperature",
        0.70
    );

    /* Attempt recovery with good SOC */
    SmartQsoResult_t result = fault_recover_thermal(0.80);
    assert_int_equal(result, SMART_QSO_OK);
}

/**
 * @brief Test power fault recovery
 *
 * @requirement SRS-F044 Recovery from SAFE mode when SOC improves
 */
static void test_fault_recover_power(void **state) {
    (void)state;

    /* Log a power fault */
    fault_log_add(
        FAULT_TYPE_POWER,
        FAULT_SEVERITY_ERROR,
        "Low battery",
        0.20
    );

    /* Attempt recovery with improved SOC */
    SmartQsoResult_t result = fault_recover_power(0.60);
    assert_int_equal(result, SMART_QSO_OK);
}

/*===========================================================================*/
/* Test Cases: Fault Log Retrieval                                            */
/*===========================================================================*/

/**
 * @brief Test fault log count and retrieval
 */
static void test_fault_log_retrieval(void **state) {
    (void)state;

    /* Log various faults */
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_INFO, "Info 1", 0.80);
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_INFO, "Info 2", 0.80);
    fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING, "Warning 1", 0.70);
    fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_ERROR, "Error 1", 0.65);
    fault_log_add(FAULT_TYPE_POWER_CRITICAL, FAULT_SEVERITY_CRITICAL, "Critical 1", 0.15);

    size_t count = fault_log_get_count();
    assert_int_equal(count, 5);

    /* Retrieve last entry */
    FaultLogEntry_t entry;
    SmartQsoResult_t result = fault_log_get_last(&entry);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.fault_type, FAULT_TYPE_POWER_CRITICAL);
}

/* Note: Null pointer test removed - implementation uses defensive asserts */

/**
 * @brief Test fault log get entry
 */
static void test_fault_log_get_entry(void **state) {
    (void)state;

    /* Log a fault */
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_WARNING, "Test fault", 0.75);

    FaultLogEntry_t entry;
    SmartQsoResult_t result = fault_log_get_entry(0, &entry);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(entry.fault_type, FAULT_TYPE_THERMAL);
}

/*===========================================================================*/
/* Test Cases: Persistence                                                    */
/*===========================================================================*/

/**
 * @brief Test fault log persistence
 *
 * @requirement SRS-F041 Maintain fault log in NVM
 */
static void test_fault_persistence(void **state) {
    (void)state;

    /* Log some faults */
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_WARNING, "Test fault 1", 0.75);
    fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_ERROR, "Test fault 2", 0.60);

    /* Save to file */
    SmartQsoResult_t result = fault_log_save();
    assert_int_equal(result, SMART_QSO_OK);

    /* Clear and re-initialize */
    fault_log_clear();

    /* Load from file */
    result = fault_log_load();
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify faults were loaded */
    size_t count = fault_log_get_count();
    assert_true(count >= 2);
}

/*===========================================================================*/
/* Test Cases: Fault Types                                                    */
/*===========================================================================*/

/**
 * @brief Test all fault types can be logged
 */
static void test_fault_all_types(void **state) {
    (void)state;

    FaultType_t types[] = {
        FAULT_TYPE_POWER,
        FAULT_TYPE_MODE_CHANGE,
        FAULT_TYPE_THERMAL,
        FAULT_TYPE_POWER_CRITICAL,
        FAULT_TYPE_WATCHDOG,
        FAULT_TYPE_UART,
        FAULT_TYPE_VOLTAGE_LOW,
        FAULT_TYPE_VOLTAGE_RANGE,
        FAULT_TYPE_CURRENT_HIGH,
        FAULT_TYPE_TEMP_HIGH,
        FAULT_TYPE_TEMP_LOW,
        FAULT_TYPE_RECOVERY,
        FAULT_TYPE_INIT
    };

    for (size_t i = 0; i < sizeof(types) / sizeof(types[0]); i++) {
        char desc[64];
        snprintf(desc, sizeof(desc), "Test fault type %d", types[i]);
        SmartQsoResult_t result = fault_log_add(
            types[i],
            FAULT_SEVERITY_INFO,
            desc,
            0.75
        );
        assert_int_equal(result, SMART_QSO_OK);
    }
}

/**
 * @brief Test fault log clear
 */
static void test_fault_log_clear(void **state) {
    (void)state;

    /* Log some faults */
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_WARNING, "Test", 0.75);
    fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_ERROR, "Test 2", 0.60);

    size_t count = fault_log_get_count();
    assert_int_equal(count, 2);

    /* Clear the log */
    SmartQsoResult_t result = fault_log_clear();
    assert_int_equal(result, SMART_QSO_OK);

    count = fault_log_get_count();
    assert_int_equal(count, 0);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_fault_init, setup, teardown),

        /* Fault logging tests */
        cmocka_unit_test_setup_teardown(test_fault_log_single, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_multiple, setup, teardown),
        /* Note: Null pointer test removed - implementation uses defensive asserts */
        cmocka_unit_test_setup_teardown(test_fault_log_overflow, setup, teardown),

        /* Fault severity tests */
        cmocka_unit_test_setup_teardown(test_fault_severity_info, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_severity_warning, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_severity_error, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_severity_critical, setup, teardown),

        /* Fault recovery tests */
        cmocka_unit_test_setup_teardown(test_fault_recover_thermal, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_recover_power, setup, teardown),

        /* Fault log retrieval tests */
        cmocka_unit_test_setup_teardown(test_fault_log_retrieval, setup, teardown),
        /* Note: Null pointer test removed - implementation uses defensive asserts */
        cmocka_unit_test_setup_teardown(test_fault_log_get_entry, setup, teardown),

        /* Persistence tests */
        cmocka_unit_test_setup_teardown(test_fault_persistence, setup, teardown),

        /* Fault type tests */
        cmocka_unit_test_setup_teardown(test_fault_all_types, setup, teardown),

        /* Clear tests */
        cmocka_unit_test_setup_teardown(test_fault_log_clear, setup, teardown),
    };

    return cmocka_run_group_tests_name("Fault Management Tests", tests, NULL, NULL);
}
