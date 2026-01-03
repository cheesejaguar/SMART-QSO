/**
 * @file test_system_state.c
 * @brief Unit tests for system state management module
 *
 * Tests centralized state management per NPR 7150.2 modularity requirements.
 *
 * @requirement SRS-MOD-001 All system state shall be encapsulated
 * @requirement SRS-MOD-002 State access shall be through accessor functions
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <math.h>

/* Include the module under test */
#include "system_state.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

/**
 * @brief Setup function - initialize state before each test
 */
static int setup(void **state)
{
    (void)state;
    sys_state_init();
    return 0;
}

/*===========================================================================*/
/* Initialization Tests                                                       */
/*===========================================================================*/

/**
 * @brief Test system state initialization
 */
static void test_sys_state_init(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sys_state_init();

    assert_int_equal(ret, SMART_QSO_OK);
    assert_false(sys_state_is_dirty());
}

/**
 * @brief Test default values after initialization
 */
static void test_sys_state_defaults(void **state)
{
    (void)state;

    /* Power defaults */
    assert_int_equal(sys_get_power_mode(), POWER_MODE_SAFE);
    assert_false(sys_is_payload_enabled());
    assert_true(fabs(sys_get_state_of_charge() - 0.5) < 0.01);

    /* ADCS defaults */
    assert_false(sys_is_detumbled());

    /* Mission defaults */
    assert_int_equal(sys_get_boot_count(), 0);
    assert_int_equal(sys_get_uptime_s(), 0);

    /* State machine defaults */
    assert_int_equal(sys_get_operational_state(), STATE_BOOT);
}

/*===========================================================================*/
/* Power State Tests                                                          */
/*===========================================================================*/

/**
 * @brief Test battery voltage accessor
 */
static void test_sys_set_battery_voltage(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sys_set_battery_voltage(4.2);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(sys_state_is_dirty());

    PowerState_t power;
    ret = sys_get_power_state(&power);
    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(fabs(power.battery_voltage - 4.2) < 0.01);
}

/**
 * @brief Test state of charge clamping
 */
static void test_sys_set_soc_clamping(void **state)
{
    (void)state;

    /* Test upper clamp */
    sys_set_state_of_charge(1.5);
    assert_true(fabs(sys_get_state_of_charge() - 1.0) < 0.01);

    /* Test lower clamp */
    sys_set_state_of_charge(-0.5);
    assert_true(fabs(sys_get_state_of_charge() - 0.0) < 0.01);

    /* Test valid value */
    sys_set_state_of_charge(0.75);
    assert_true(fabs(sys_get_state_of_charge() - 0.75) < 0.01);
}

/**
 * @brief Test power mode accessor
 */
static void test_sys_power_mode(void **state)
{
    (void)state;

    sys_set_power_mode(POWER_MODE_ACTIVE);
    assert_int_equal(sys_get_power_mode(), POWER_MODE_ACTIVE);

    sys_set_power_mode(POWER_MODE_IDLE);
    assert_int_equal(sys_get_power_mode(), POWER_MODE_IDLE);
}

/**
 * @brief Test payload enable accessor
 */
static void test_sys_payload_enabled(void **state)
{
    (void)state;

    assert_false(sys_is_payload_enabled());

    sys_set_payload_enabled(true);
    assert_true(sys_is_payload_enabled());

    sys_set_payload_enabled(false);
    assert_false(sys_is_payload_enabled());
}

/**
 * @brief Test get power state with NULL
 */
static void test_sys_get_power_state_null(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sys_get_power_state(NULL);
    assert_int_equal(ret, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Thermal State Tests                                                        */
/*===========================================================================*/

/**
 * @brief Test temperature sensor accessors
 */
static void test_sys_set_temperature(void **state)
{
    (void)state;

    /* OBC temperature */
    SmartQsoResult_t ret = sys_set_temperature(0, 35.0f);
    assert_int_equal(ret, SMART_QSO_OK);

    /* EPS temperature */
    ret = sys_set_temperature(1, 28.0f);
    assert_int_equal(ret, SMART_QSO_OK);

    /* Invalid sensor */
    ret = sys_set_temperature(10, 25.0f);
    assert_int_equal(ret, SMART_QSO_ERROR_INVALID);

    ThermalState_t thermal;
    ret = sys_get_thermal_state(&thermal);
    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(fabs(thermal.obc_temp_c - 35.0f) < 0.1f);
    assert_true(fabs(thermal.eps_temp_c - 28.0f) < 0.1f);
}

/**
 * @brief Test thermal fault detection
 */
static void test_sys_thermal_fault(void **state)
{
    (void)state;

    assert_false(sys_has_thermal_fault());

    /* Over temperature */
    sys_set_temperature(0, 70.0f);
    assert_true(sys_has_thermal_fault());

    /* Return to normal */
    sys_set_temperature(0, 25.0f);
    assert_false(sys_has_thermal_fault());

    /* Under temperature */
    sys_set_temperature(0, -30.0f);
    assert_true(sys_has_thermal_fault());
}

/*===========================================================================*/
/* ADCS State Tests                                                           */
/*===========================================================================*/

/**
 * @brief Test magnetometer accessor
 */
static void test_sys_set_magnetometer(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sys_set_magnetometer(10.5f, -20.3f, 5.1f);
    assert_int_equal(ret, SMART_QSO_OK);

    AdcsState_t adcs;
    ret = sys_get_adcs_state(&adcs);
    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(fabs(adcs.mag_x_ut - 10.5f) < 0.1f);
    assert_true(fabs(adcs.mag_y_ut - (-20.3f)) < 0.1f);
    assert_true(fabs(adcs.mag_z_ut - 5.1f) < 0.1f);
}

/**
 * @brief Test gyroscope accessor
 */
static void test_sys_set_gyroscope(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sys_set_gyroscope(1.0f, 2.0f, 3.0f);
    assert_int_equal(ret, SMART_QSO_OK);

    AdcsState_t adcs;
    ret = sys_get_adcs_state(&adcs);
    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(fabs(adcs.gyro_x_dps - 1.0f) < 0.1f);
    assert_true(fabs(adcs.gyro_y_dps - 2.0f) < 0.1f);
    assert_true(fabs(adcs.gyro_z_dps - 3.0f) < 0.1f);
}

/**
 * @brief Test detumble flag
 */
static void test_sys_detumbled(void **state)
{
    (void)state;

    assert_false(sys_is_detumbled());

    sys_set_detumbled(true);
    assert_true(sys_is_detumbled());

    sys_set_detumbled(false);
    assert_false(sys_is_detumbled());
}

/*===========================================================================*/
/* Communications State Tests                                                 */
/*===========================================================================*/

/**
 * @brief Test packet counters
 */
static void test_sys_packet_counters(void **state)
{
    (void)state;

    CommState_t comm;
    sys_get_comm_state(&comm);
    assert_int_equal(comm.packets_sent, 0);
    assert_int_equal(comm.packets_received, 0);

    sys_increment_packets_sent();
    sys_increment_packets_sent();
    sys_increment_packets_received();

    sys_get_comm_state(&comm);
    assert_int_equal(comm.packets_sent, 2);
    assert_int_equal(comm.packets_received, 1);
}

/**
 * @brief Test beacon counter
 */
static void test_sys_beacon_counter(void **state)
{
    (void)state;

    CommState_t comm;
    sys_get_comm_state(&comm);
    assert_int_equal(comm.beacon_count, 0);

    sys_increment_beacon_count();
    sys_increment_beacon_count();
    sys_increment_beacon_count();

    sys_get_comm_state(&comm);
    assert_int_equal(comm.beacon_count, 3);
}

/**
 * @brief Test beacon interval validation
 */
static void test_sys_beacon_interval(void **state)
{
    (void)state;

    /* Valid interval */
    SmartQsoResult_t ret = sys_set_beacon_interval(60);
    assert_int_equal(ret, SMART_QSO_OK);

    /* Too short */
    ret = sys_set_beacon_interval(5);
    assert_int_equal(ret, SMART_QSO_ERROR_INVALID);

    /* Too long */
    ret = sys_set_beacon_interval(150);
    assert_int_equal(ret, SMART_QSO_ERROR_INVALID);

    /* Check valid value was kept */
    CommState_t comm;
    sys_get_comm_state(&comm);
    assert_int_equal(comm.beacon_interval_s, 60);
}

/*===========================================================================*/
/* Mission State Tests                                                        */
/*===========================================================================*/

/**
 * @brief Test boot counter
 */
static void test_sys_boot_counter(void **state)
{
    (void)state;

    assert_int_equal(sys_get_boot_count(), 0);

    sys_increment_boot_count();
    assert_int_equal(sys_get_boot_count(), 1);

    sys_increment_boot_count();
    sys_increment_boot_count();
    assert_int_equal(sys_get_boot_count(), 3);
}

/**
 * @brief Test QSO counter
 */
static void test_sys_qso_counter(void **state)
{
    (void)state;

    MissionState_t mission;
    sys_get_mission_state(&mission);
    assert_int_equal(mission.qso_count, 0);

    sys_increment_qso_count();
    sys_increment_qso_count();

    sys_get_mission_state(&mission);
    assert_int_equal(mission.qso_count, 2);
}

/**
 * @brief Test command counter
 */
static void test_sys_command_counter(void **state)
{
    (void)state;

    MissionState_t mission;
    sys_get_mission_state(&mission);
    assert_int_equal(mission.command_count, 0);

    sys_increment_command_count();

    sys_get_mission_state(&mission);
    assert_int_equal(mission.command_count, 1);
}

/**
 * @brief Test anomaly counter
 */
static void test_sys_anomaly_counter(void **state)
{
    (void)state;

    MissionState_t mission;
    sys_get_mission_state(&mission);
    assert_int_equal(mission.anomaly_count, 0);

    sys_increment_anomaly_count();
    sys_increment_anomaly_count();

    sys_get_mission_state(&mission);
    assert_int_equal(mission.anomaly_count, 2);
}

/*===========================================================================*/
/* State Machine Tests                                                        */
/*===========================================================================*/

/**
 * @brief Test state machine context access
 */
static void test_sys_sm_context(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    assert_non_null(ctx);
    assert_true(ctx->is_initialized);
}

/**
 * @brief Test operational state accessor
 */
static void test_sys_operational_state(void **state)
{
    (void)state;

    assert_int_equal(sys_get_operational_state(), STATE_BOOT);

    /* Process event through state machine */
    SmContext_t *ctx = sys_get_sm_context();
    sm_process_event(ctx, EVENT_BOOT_COMPLETE, NULL);

    assert_int_equal(sys_get_operational_state(), STATE_DETUMBLE);
}

/*===========================================================================*/
/* Integrity Tests                                                            */
/*===========================================================================*/

/**
 * @brief Test CRC update and verify
 */
static void test_sys_state_crc(void **state)
{
    (void)state;

    /* Update CRC */
    SmartQsoResult_t ret = sys_state_update_crc();
    assert_int_equal(ret, SMART_QSO_OK);

    /* Verify CRC */
    assert_true(sys_state_verify_crc());
}

/**
 * @brief Test dirty flag
 */
static void test_sys_state_dirty_flag(void **state)
{
    (void)state;

    /* Start clean */
    sys_state_clear_dirty();
    assert_false(sys_state_is_dirty());

    /* Modify state */
    sys_set_battery_voltage(4.0);
    assert_true(sys_state_is_dirty());

    /* Clear */
    sys_state_clear_dirty();
    assert_false(sys_state_is_dirty());
}

/**
 * @brief Test full state snapshot
 */
static void test_sys_get_full_state(void **state)
{
    (void)state;

    /* Set some values */
    sys_set_battery_voltage(4.1);
    sys_set_power_mode(POWER_MODE_IDLE);
    sys_increment_boot_count();

    /* Get full snapshot */
    SystemState_t snapshot;
    SmartQsoResult_t ret = sys_get_full_state(&snapshot);
    assert_int_equal(ret, SMART_QSO_OK);

    /* Verify values */
    assert_true(fabs(snapshot.power.battery_voltage - 4.1) < 0.01);
    assert_int_equal(snapshot.power.power_mode, POWER_MODE_IDLE);
    assert_int_equal(snapshot.mission.boot_count, 1);
    assert_true(snapshot.initialized);
}

/**
 * @brief Test full state with NULL
 */
static void test_sys_get_full_state_null(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sys_get_full_state(NULL);
    assert_int_equal(ret, SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup(test_sys_state_init, setup),
        cmocka_unit_test_setup(test_sys_state_defaults, setup),

        /* Power state tests */
        cmocka_unit_test_setup(test_sys_set_battery_voltage, setup),
        cmocka_unit_test_setup(test_sys_set_soc_clamping, setup),
        cmocka_unit_test_setup(test_sys_power_mode, setup),
        cmocka_unit_test_setup(test_sys_payload_enabled, setup),
        cmocka_unit_test_setup(test_sys_get_power_state_null, setup),

        /* Thermal state tests */
        cmocka_unit_test_setup(test_sys_set_temperature, setup),
        cmocka_unit_test_setup(test_sys_thermal_fault, setup),

        /* ADCS state tests */
        cmocka_unit_test_setup(test_sys_set_magnetometer, setup),
        cmocka_unit_test_setup(test_sys_set_gyroscope, setup),
        cmocka_unit_test_setup(test_sys_detumbled, setup),

        /* Communications state tests */
        cmocka_unit_test_setup(test_sys_packet_counters, setup),
        cmocka_unit_test_setup(test_sys_beacon_counter, setup),
        cmocka_unit_test_setup(test_sys_beacon_interval, setup),

        /* Mission state tests */
        cmocka_unit_test_setup(test_sys_boot_counter, setup),
        cmocka_unit_test_setup(test_sys_qso_counter, setup),
        cmocka_unit_test_setup(test_sys_command_counter, setup),
        cmocka_unit_test_setup(test_sys_anomaly_counter, setup),

        /* State machine tests */
        cmocka_unit_test_setup(test_sys_sm_context, setup),
        cmocka_unit_test_setup(test_sys_operational_state, setup),

        /* Integrity tests */
        cmocka_unit_test_setup(test_sys_state_crc, setup),
        cmocka_unit_test_setup(test_sys_state_dirty_flag, setup),
        cmocka_unit_test_setup(test_sys_get_full_state, setup),
        cmocka_unit_test_setup(test_sys_get_full_state_null, setup),
    };

    return cmocka_run_group_tests_name("System State Tests", tests, NULL, NULL);
}
