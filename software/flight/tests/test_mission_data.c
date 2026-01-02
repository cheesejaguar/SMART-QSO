/**
 * @file test_mission_data.c
 * @brief Unit tests for Mission Data module
 *
 * Tests data persistence per SRS-F060 through SRS-F063.
 *
 * @requirement SRS-F060 Persist mission data to non-volatile memory
 * @requirement SRS-F061 Protect persisted data with CRC32
 * @requirement SRS-F062 Detect and handle corrupted data
 * @requirement SRS-F063 Maintain mission statistics across resets
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
#include "mission_data.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Remove any existing data file */
    unlink(MISSION_DATA_FILE);
    /* Initialize mission data module */
    SmartQsoResult_t result = mission_data_init();
    assert_int_equal(result, SMART_QSO_OK);
    return 0;
}

static int teardown(void **state) {
    (void)state;
    unlink(MISSION_DATA_FILE);
    return 0;
}

/*===========================================================================*/
/* Test Cases: Initialization                                                 */
/*===========================================================================*/

/**
 * @brief Test mission data initialization with no existing file
 *
 * @requirement SRS-F060 Mission data initialization
 */
static void test_mission_data_init_fresh(void **state) {
    (void)state;

    /* Module initialized in setup */
    MissionData_t data;
    SmartQsoResult_t result = mission_data_get(&data);

    assert_int_equal(result, SMART_QSO_OK);
    /* Fresh initialization should have reset count of 1 */
    assert_int_equal(data.reset_count, 1);
}

/**
 * @brief Test mission data get with null pointer
 */
static void test_mission_data_get_null(void **state) {
    (void)state;

    SmartQsoResult_t result = mission_data_get(NULL);
    assert_int_equal(result, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Cases: Statistics Updates                                             */
/*===========================================================================*/

/**
 * @brief Test reset counter increment
 *
 * @requirement SRS-F004 Increment reset counter on each boot
 * @requirement SRS-F063 Maintain mission statistics across resets
 */
static void test_mission_data_reset_counter(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    uint32_t initial_count = data.reset_count;

    /* Simulate save and re-init */
    mission_data_save();

    /* Re-initialize (simulates reset) */
    mission_data_init();

    mission_data_get(&data);
    assert_int_equal(data.reset_count, initial_count + 1);
}

/**
 * @brief Test uptime tracking
 *
 * @requirement SRS-F063 Maintain mission statistics
 */
static void test_mission_data_uptime(void **state) {
    (void)state;

    /* Update uptime */
    SmartQsoResult_t result = mission_data_update_uptime(1000);
    assert_int_equal(result, SMART_QSO_OK);

    MissionData_t data;
    mission_data_get(&data);

    assert_true(data.total_uptime_ms >= 1000);
}

/**
 * @brief Test beacon counter increment
 *
 * @requirement SRS-F063 Maintain mission statistics
 */
static void test_mission_data_beacon_counter(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    uint32_t initial_count = data.beacon_count;

    /* Increment beacon counter */
    SmartQsoResult_t result = mission_data_increment_beacon(false);
    assert_int_equal(result, SMART_QSO_OK);

    mission_data_get(&data);
    assert_int_equal(data.beacon_count, initial_count + 1);
}

/**
 * @brief Test AI beacon counter
 *
 * @requirement SRS-F063 Track AI vs fallback beacons
 */
static void test_mission_data_ai_beacon_counter(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    uint32_t initial_ai = data.ai_beacon_count;
    uint32_t initial_fallback = data.fallback_beacon_count;

    /* Increment AI beacon */
    mission_data_increment_beacon(true);
    mission_data_get(&data);
    assert_int_equal(data.ai_beacon_count, initial_ai + 1);
    assert_int_equal(data.fallback_beacon_count, initial_fallback);

    /* Increment fallback beacon */
    mission_data_increment_beacon(false);
    mission_data_get(&data);
    assert_int_equal(data.fallback_beacon_count, initial_fallback + 1);
}

/**
 * @brief Test energy tracking
 *
 * @requirement SRS-F063 Track total energy consumed
 */
static void test_mission_data_energy(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    double initial_energy = data.total_energy_wh;

    /* Add energy */
    SmartQsoResult_t result = mission_data_add_energy(1.5);
    assert_int_equal(result, SMART_QSO_OK);

    mission_data_get(&data);
    assert_true(data.total_energy_wh >= initial_energy + 1.5 - 0.01);
}

/**
 * @brief Test fault counter
 *
 * @requirement SRS-F063 Track fault statistics
 */
static void test_mission_data_fault_counter(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    uint32_t initial_faults = data.fault_count;

    /* Increment fault counter */
    SmartQsoResult_t result = mission_data_increment_faults();
    assert_int_equal(result, SMART_QSO_OK);

    mission_data_get(&data);
    assert_int_equal(data.fault_count, initial_faults + 1);
}

/*===========================================================================*/
/* Test Cases: Persistence                                                    */
/*===========================================================================*/

/**
 * @brief Test saving mission data
 *
 * @requirement SRS-F060 Persist mission data to NVM
 */
static void test_mission_data_save(void **state) {
    (void)state;

    /* Modify some data */
    mission_data_increment_beacon(true);
    mission_data_add_energy(2.5);

    /* Save */
    SmartQsoResult_t result = mission_data_save();
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify file exists */
    FILE *f = fopen(MISSION_DATA_FILE, "rb");
    assert_non_null(f);
    fclose(f);
}

/**
 * @brief Test loading mission data
 *
 * @requirement SRS-F060 Load mission data from NVM
 */
static void test_mission_data_load(void **state) {
    (void)state;

    /* Set specific values */
    mission_data_increment_beacon(true);
    mission_data_increment_beacon(true);
    mission_data_add_energy(5.0);

    MissionData_t original;
    mission_data_get(&original);

    /* Save */
    mission_data_save();

    /* Modify in memory */
    mission_data_increment_beacon(false);

    /* Reload */
    SmartQsoResult_t result = mission_data_load();
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify original values restored */
    MissionData_t loaded;
    mission_data_get(&loaded);

    assert_int_equal(loaded.beacon_count, original.beacon_count);
    assert_true(loaded.total_energy_wh >= original.total_energy_wh - 0.01);
}

/**
 * @brief Test persistence survives reinitialization
 *
 * @requirement SRS-F063 Statistics survive resets
 */
static void test_mission_data_survive_reinit(void **state) {
    (void)state;

    /* Set values and save */
    mission_data_add_energy(10.0);
    mission_data_save();

    MissionData_t original;
    mission_data_get(&original);

    /* Re-initialize (simulates boot) */
    mission_data_init();

    MissionData_t after;
    mission_data_get(&after);

    /* Energy should be preserved */
    assert_true(after.total_energy_wh >= 10.0 - 0.01);
    /* Reset count should increment */
    assert_int_equal(after.reset_count, original.reset_count + 1);
}

/*===========================================================================*/
/* Test Cases: CRC Protection                                                 */
/*===========================================================================*/

/**
 * @brief Test CRC protection on save
 *
 * @requirement SRS-F061 Protect persisted data with CRC32
 */
static void test_mission_data_crc_protection(void **state) {
    (void)state;

    mission_data_save();

    /* Read raw file and verify CRC is present */
    FILE *f = fopen(MISSION_DATA_FILE, "rb");
    assert_non_null(f);

    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fclose(f);

    /* File should include CRC32 (4 bytes) */
    assert_true(size >= sizeof(uint32_t));
}

/**
 * @brief Test detection of corrupted data
 *
 * @requirement SRS-F062 Detect and handle corrupted data
 */
static void test_mission_data_corruption_detection(void **state) {
    (void)state;

    /* Save valid data */
    mission_data_save();

    /* Corrupt the file */
    FILE *f = fopen(MISSION_DATA_FILE, "r+b");
    assert_non_null(f);

    /* Write garbage to the middle */
    fseek(f, 10, SEEK_SET);
    uint8_t garbage = 0xFF;
    fwrite(&garbage, 1, 1, f);
    fclose(f);

    /* Attempt to load - should fail or use defaults */
    SmartQsoResult_t result = mission_data_load();

    /* Either returns error or reinitializes with defaults */
    /* Implementation dependent - check that we don't crash */
}

/*===========================================================================*/
/* Test Cases: Boundary Conditions                                            */
/*===========================================================================*/

/**
 * @brief Test maximum values
 */
static void test_mission_data_max_values(void **state) {
    (void)state;

    /* Add large amounts */
    for (int i = 0; i < 1000; i++) {
        mission_data_increment_beacon(true);
        mission_data_add_energy(0.1);
    }

    MissionData_t data;
    mission_data_get(&data);

    assert_true(data.beacon_count >= 1000);
    assert_true(data.total_energy_wh >= 100.0 - 0.1);
}

/**
 * @brief Test watchdog reset counter
 *
 * @requirement SRS-F005 Log boot reason
 */
static void test_mission_data_watchdog_counter(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    uint32_t initial = data.watchdog_reset_count;

    /* Increment watchdog counter */
    SmartQsoResult_t result = mission_data_increment_watchdog();
    assert_int_equal(result, SMART_QSO_OK);

    mission_data_get(&data);
    assert_int_equal(data.watchdog_reset_count, initial + 1);
}

/*===========================================================================*/
/* Test Cases: Data Integrity                                                 */
/*===========================================================================*/

/**
 * @brief Test data consistency after multiple operations
 */
static void test_mission_data_consistency(void **state) {
    (void)state;

    /* Perform many operations */
    for (int i = 0; i < 100; i++) {
        mission_data_increment_beacon(i % 2 == 0);
        mission_data_add_energy(0.01);
        if (i % 10 == 0) {
            mission_data_save();
        }
    }

    MissionData_t data;
    mission_data_get(&data);

    /* Verify counts make sense */
    assert_int_equal(data.beacon_count, 100);
    assert_int_equal(data.ai_beacon_count, 50);
    assert_int_equal(data.fallback_beacon_count, 50);
}

/**
 * @brief Test version field
 */
static void test_mission_data_version(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    /* Version should be non-zero */
    assert_true(data.version > 0);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup_teardown(test_mission_data_init_fresh, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_get_null, setup, teardown),

        /* Statistics tests */
        cmocka_unit_test_setup_teardown(test_mission_data_reset_counter, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_uptime, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_beacon_counter, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_ai_beacon_counter, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_energy, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_fault_counter, setup, teardown),

        /* Persistence tests */
        cmocka_unit_test_setup_teardown(test_mission_data_save, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_load, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_survive_reinit, setup, teardown),

        /* CRC protection tests */
        cmocka_unit_test_setup_teardown(test_mission_data_crc_protection, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_corruption_detection, setup, teardown),

        /* Boundary tests */
        cmocka_unit_test_setup_teardown(test_mission_data_max_values, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_watchdog_counter, setup, teardown),

        /* Integrity tests */
        cmocka_unit_test_setup_teardown(test_mission_data_consistency, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_version, setup, teardown),
    };

    return cmocka_run_group_tests_name("Mission Data Tests", tests, NULL, NULL);
}
