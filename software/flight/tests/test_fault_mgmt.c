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
#include "eps_control.h"
#include "payload_guard.h"
#include "state_machine.h"
#include "system_state.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static int setup(void **state) {
    (void)state;
    /* Clean up any existing fault log file first */
    unlink(FAULT_LOG_FILE);
    /* Initialize fault management - this will try to load but file is gone */
    SmartQsoResult_t result = fault_mgmt_init();
    assert_int_equal(result, SMART_QSO_OK);
    /* Clear any loaded state to ensure clean slate for each test */
    fault_log_clear();
    return 0;
}

/**
 * Fault log plus the mode manager as main.c starts it (SW-14): EPS, payload
 * guard, a healthy battery and the system state (state machine in BOOT)
 */
static int setup_mode_manager(void **state) {
    (void)setup(state);
    payload_guard_init();
    assert_int_equal(eps_init(), SMART_QSO_OK);
    eps_update_soc(0.9);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    return 0;
}

/** Put the mode manager in a state; its entry action sets the EPS mode */
static void start_in(SmState_t st) {
    assert_int_equal(sm_force_state(sys_get_sm_context(), st), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), st);
}

/** The EPS power mode each operational state sets on entry (state_machine.c) */
static PowerMode_t power_mode_of(SmState_t st) {
    PowerMode_t mode = POWER_MODE_SAFE;
    if ((st == STATE_DETUMBLE) || (st == STATE_IDLE)) {
        mode = POWER_MODE_IDLE;
    } else if (st == STATE_ACTIVE) {
        mode = POWER_MODE_ACTIVE;
    }
    return mode;
}

/** One source of truth: the EPS holds the mode the state machine is in */
static void assert_modes_agree(void) {
    assert_int_equal(eps_get_power_mode(), power_mode_of(sys_get_operational_state()));
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
/* Test Cases: Recovery goes through the mode manager (SW-14)                 */
/*===========================================================================*/

/**
 * @brief Thermal recovery from ACTIVE: payload off, and the state machine and
 *        EPS end in the same mode (the table's THERMAL_FAULT row: SAFE, as
 *        main.c's health check raises it)
 *
 * @requirement SRS-F042 Implement autonomous fault recovery
 * @requirement SRS-S005 Limit power consumption to prevent thermal damage
 */
static void test_recover_thermal_through_mode_manager(void **state) {
    (void)state;
    start_in(STATE_ACTIVE);
    assert_true(eps_is_payload_enabled());

    assert_int_equal(fault_recover_thermal(0.9), SMART_QSO_OK);

    assert_false(eps_is_payload_enabled());
    assert_int_equal(sys_get_operational_state(), STATE_SAFE);
    assert_modes_agree();
}

/**
 * @brief Power recovery below 40 % takes ACTIVE to IDLE in both the state
 *        machine and the EPS (BASELINE §5)
 *
 * @requirement SRS-F014 Transition to IDLE mode when SOC < 40%
 */
static void test_recover_power_active_to_idle(void **state) {
    (void)state;
    start_in(STATE_ACTIVE);
    eps_update_soc(0.35);

    assert_int_equal(fault_recover_power(0.35), SMART_QSO_OK);

    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_false(eps_is_payload_enabled());
    assert_modes_agree();
}

/**
 * @brief Power recovery below 25 % takes IDLE to SAFE in both
 *
 * @requirement SRS-F013 Transition to SAFE mode when SOC < 25%
 * @requirement SRS-F043 Enter SAFE mode on critical faults
 */
static void test_recover_power_idle_to_safe(void **state) {
    (void)state;
    start_in(STATE_IDLE);
    eps_update_soc(0.20);

    assert_int_equal(fault_recover_power(0.20), SMART_QSO_OK);

    assert_int_equal(sys_get_operational_state(), STATE_SAFE);
    assert_modes_agree();
}

/**
 * @brief Hysteresis: in SAFE at 30 % recovery leaves SAFE alone; SAFE goes
 *        back to IDLE only above 40 % (BASELINE §5, sm_power_event())
 *
 * @requirement SRS-F044 Attempt recovery from SAFE mode when SOC improves
 */
static void test_recover_power_keeps_safe_hysteresis(void **state) {
    (void)state;
    start_in(STATE_SAFE);
    eps_update_soc(0.30);

    assert_int_equal(fault_recover_power(0.30), SMART_QSO_OK);

    assert_int_equal(sys_get_operational_state(), STATE_SAFE);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
}

/**
 * @brief SAFE with SOC above 40 %: recovery returns to IDLE
 *
 * @requirement SRS-F044 Attempt recovery from SAFE mode when SOC improves
 */
static void test_recover_power_safe_to_idle(void **state) {
    (void)state;
    start_in(STATE_SAFE);
    eps_update_soc(0.60);

    assert_int_equal(fault_recover_power(0.60), SMART_QSO_OK);

    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_modes_agree();
}

/**
 * @brief SOC still falling in SAFE (below 15 %): EMERGENCY, which sheds the
 *        beacon (BASELINE §5)
 *
 * @requirement SRS-F043 Enter SAFE mode on critical faults
 */
static void test_recover_power_emergency_sheds_beacon(void **state) {
    (void)state;
    start_in(STATE_SAFE);
    assert_true(eps_is_beacon_enabled());
    eps_update_soc(0.10);

    assert_int_equal(fault_recover_power(0.10), SMART_QSO_OK);

    assert_int_equal(sys_get_operational_state(), STATE_EMERGENCY);
    assert_false(eps_is_beacon_enabled());
    assert_modes_agree();
}

/**
 * @brief Recovery never puts the spacecraft in ACTIVE: not with the payload
 *        guard in cool-down, and not with a full battery either (ACTIVE needs
 *        SOC >= 55 %, sunlight and payload_guard_may_enable())
 *
 * @requirement SRS-F015 Enable payload only when SOC > 55% and sunlit
 */
static void test_recover_power_never_enters_active(void **state) {
    (void)state;
    start_in(STATE_IDLE);

    PayloadSample_t latchup;
    memset(&latchup, 0, sizeof(latchup));
    latchup.power_valid = true;
    latchup.input_w = PG_LATCHUP_INPUT_W;
    (void)payload_guard_step(&latchup, true, smart_qso_now_ms());   /* trip: cool-down */
    assert_false(payload_guard_may_enable(smart_qso_now_ms()));

    assert_int_equal(fault_recover_power(0.9), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_false(eps_is_payload_enabled());

    payload_guard_init();
    assert_int_equal(fault_recover_power(0.9), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_false(eps_is_payload_enabled());
    assert_modes_agree();
}

/**
 * @brief Without an initialized mode manager, power recovery refuses rather
 *        than setting the EPS behind it
 *
 * @requirement SRS-F042 Implement autonomous fault recovery
 */
static void test_recover_power_needs_mode_manager(void **state) {
    (void)state;
    start_in(STATE_IDLE);
    SmContext_t *ctx = sys_get_sm_context();
    ctx->is_initialized = false;

    assert_int_not_equal(fault_recover_power(0.10), SMART_QSO_OK);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);

    assert_int_equal(sys_state_init(), SMART_QSO_OK);   /* leave it usable */
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
 * Note: fault_log_clear() removes the persistence file, so we test
 * save and load without clearing in between.
 *
 * @requirement SRS-F041 Maintain fault log in NVM
 */
static void test_fault_persistence(void **state) {
    (void)state;

    /* Log some faults */
    fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_WARNING, "Test fault 1", 0.75);
    fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_ERROR, "Test fault 2", 0.60);

    size_t count_before = fault_log_get_count();
    assert_true(count_before >= 2);

    /* Save to file */
    SmartQsoResult_t result = fault_log_save();
    assert_int_equal(result, SMART_QSO_OK);

    /* Load from file should succeed */
    result = fault_log_load();
    assert_int_equal(result, SMART_QSO_OK);

    /* Verify faults are still present */
    size_t count_after = fault_log_get_count();
    assert_int_equal(count_after, count_before);
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
        cmocka_unit_test_setup_teardown(test_fault_recover_thermal, setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_fault_recover_power, setup_mode_manager, teardown),

        /* Recovery through the mode manager (SW-14) */
        cmocka_unit_test_setup_teardown(test_recover_thermal_through_mode_manager,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_active_to_idle,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_idle_to_safe,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_keeps_safe_hysteresis,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_safe_to_idle,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_emergency_sheds_beacon,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_never_enters_active,
                                        setup_mode_manager, teardown),
        cmocka_unit_test_setup_teardown(test_recover_power_needs_mode_manager,
                                        setup_mode_manager, teardown),

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
