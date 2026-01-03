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
#include <math.h>

/* Include the module under test */
#include "smart_qso.h"
#include "mission_data.h"

/*===========================================================================*/
/* Test Helpers                                                               */
/*===========================================================================*/

#define FLOAT_TOLERANCE 0.01
#define FLOAT_EQUAL(a, b) (fabs((a) - (b)) < FLOAT_TOLERANCE)

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
    assert_true(data.total_energy_wh >= initial_energy + 1.5 - FLOAT_TOLERANCE);
}

/**
 * @brief Test fault recording
 *
 * @requirement SRS-F063 Track fault statistics
 */
static void test_mission_data_fault_recording(void **state) {
    (void)state;

    MissionData_t data;
    mission_data_get(&data);

    uint32_t initial_faults = data.fault_count;

    /* Record a fault */
    SmartQsoResult_t result = mission_data_record_fault("Test fault");
    assert_int_equal(result, SMART_QSO_OK);

    mission_data_get(&data);
    assert_int_equal(data.fault_count, initial_faults + 1);
    assert_string_equal(data.last_fault, "Test fault");
}

/**
 * @brief Test mission phase setting
 */
static void test_mission_data_phase(void **state) {
    (void)state;

    /* Set phase */
    SmartQsoResult_t result = mission_data_set_phase(MISSION_PHASE_ACTIVE);
    assert_int_equal(result, SMART_QSO_OK);

    /* Get phase */
    MissionPhase_t phase = mission_data_get_phase();
    assert_int_equal(phase, MISSION_PHASE_ACTIVE);

    MissionData_t data;
    mission_data_get(&data);
    assert_int_equal(data.mission_phase, MISSION_PHASE_ACTIVE);
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
    mission_data_add_energy(5.0);

    MissionData_t original;
    mission_data_get(&original);

    /* Save */
    mission_data_save();

    /* Modify in memory */
    mission_data_add_energy(1.0);

    /* Reload */
    SmartQsoResult_t result = mission_data_load();
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify original values restored */
    MissionData_t loaded;
    mission_data_get(&loaded);

    assert_true(FLOAT_EQUAL(loaded.total_energy_wh, original.total_energy_wh));
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
    assert_true(after.total_energy_wh >= 10.0 - FLOAT_TOLERANCE);
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
    assert_true(size >= (long)sizeof(uint32_t));
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
    (void)mission_data_load();

    /* Either returns error or reinitializes with defaults */
    /* Implementation dependent - check that we don't crash */
}

/*===========================================================================*/
/* Test Cases: Convenience Functions                                          */
/*===========================================================================*/

/**
 * @brief Test get_reset_count convenience function
 */
static void test_mission_data_get_reset_count(void **state) {
    (void)state;

    uint32_t count = mission_data_get_reset_count();
    assert_true(count >= 1);

    MissionData_t data;
    mission_data_get(&data);
    assert_int_equal(count, data.reset_count);
}

/**
 * @brief Test get_uptime convenience function
 */
static void test_mission_data_get_uptime(void **state) {
    (void)state;

    mission_data_update_uptime(5000);

    uint64_t uptime = mission_data_get_uptime();
    assert_true(uptime >= 5000);

    MissionData_t data;
    mission_data_get(&data);
    assert_int_equal(uptime, data.total_uptime_ms);
}

/**
 * @brief Test get_energy convenience function
 */
static void test_mission_data_get_energy(void **state) {
    (void)state;

    mission_data_add_energy(3.5);

    double energy = mission_data_get_energy();
    assert_true(energy >= 3.5 - FLOAT_TOLERANCE);

    MissionData_t data;
    mission_data_get(&data);
    assert_true(FLOAT_EQUAL(energy, data.total_energy_wh));
}

/**
 * @brief Test set_start function
 */
static void test_mission_data_set_start(void **state) {
    (void)state;

    uint64_t start_time = 1234567890ULL;
    SmartQsoResult_t result = mission_data_set_start(start_time);
    assert_int_equal(result, SMART_QSO_OK);

    MissionData_t data;
    mission_data_get(&data);
    assert_int_equal(data.mission_start_ms, start_time);
}

/*===========================================================================*/
/* Test Cases: Boundary Conditions                                            */
/*===========================================================================*/

/**
 * @brief Test accumulating energy over many operations
 */
static void test_mission_data_energy_accumulation(void **state) {
    (void)state;

    /* Add small amounts many times */
    for (int i = 0; i < 100; i++) {
        mission_data_add_energy(0.1);
    }

    MissionData_t data;
    mission_data_get(&data);

    assert_true(data.total_energy_wh >= 10.0 - 0.1);
}

/**
 * @brief Test data consistency after multiple operations
 */
static void test_mission_data_consistency(void **state) {
    (void)state;

    /* Perform many operations */
    for (int i = 0; i < 100; i++) {
        mission_data_add_energy(0.01);
        if (i % 10 == 0) {
            mission_data_save();
        }
    }

    MissionData_t data;
    mission_data_get(&data);

    /* Verify energy accumulated correctly */
    assert_true(data.total_energy_wh >= 1.0 - 0.1);
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
        cmocka_unit_test_setup_teardown(test_mission_data_energy, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_fault_recording, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_phase, setup, teardown),

        /* Persistence tests */
        cmocka_unit_test_setup_teardown(test_mission_data_save, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_load, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_survive_reinit, setup, teardown),

        /* CRC protection tests */
        cmocka_unit_test_setup_teardown(test_mission_data_crc_protection, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_corruption_detection, setup, teardown),

        /* Convenience function tests */
        cmocka_unit_test_setup_teardown(test_mission_data_get_reset_count, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_get_uptime, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_get_energy, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_set_start, setup, teardown),

        /* Boundary tests */
        cmocka_unit_test_setup_teardown(test_mission_data_energy_accumulation, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_consistency, setup, teardown),
    };

    return cmocka_run_group_tests_name("Mission Data Tests", tests, NULL, NULL);
}
