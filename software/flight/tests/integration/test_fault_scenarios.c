/**
 * @file test_fault_scenarios.c
 * @brief Integration tests for fault injection and recovery
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * FDIR across the fault log (fault_mgmt.c), the recovery functions, the mode
 * manager (state machine and EPS), the payload guard and the watchdog
 * manager. Mode changes from a fault go through the state machine (SW-14),
 * so after every recovery the EPS holds the mode the state machine is in.
 *
 * @requirement SRS-F040 Detect and log all fault conditions
 * @requirement SRS-F041 Maintain fault log in NVM
 * @requirement SRS-F042 Implement autonomous fault recovery
 * @requirement SRS-F043 Enter SAFE mode on critical faults
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "smart_qso.h"
#include "system_state.h"
#include "state_machine.h"
#include "fault_mgmt.h"
#include "eps_control.h"
#include "payload_guard.h"
#include "watchdog_mgr.h"
#include "hal/hal_timer.h"

/** SOC main.c's simulation starts with */
#define START_SOC 0.75

/*******************************************************************************
 * Helpers
 ******************************************************************************/

/** The EPS power mode each state's entry action sets (state_machine.c) */
static PowerMode_t power_mode_of(SmState_t st)
{
    PowerMode_t mode = POWER_MODE_SAFE;
    if ((st == STATE_DETUMBLE) || (st == STATE_IDLE)) {
        mode = POWER_MODE_IDLE;
    } else if (st == STATE_ACTIVE) {
        mode = POWER_MODE_ACTIVE;
    }
    return mode;
}

/** The system is in this state and the EPS agrees */
static void assert_in(SmState_t expected)
{
    assert_int_equal(sys_get_operational_state(), expected);
    assert_int_equal(eps_get_power_mode(), power_mode_of(expected));
}

/** Raise an event with the system's state machine; return the result */
static SmResult_t raise_event(SmEvent_t event)
{
    SmResult_t result = SM_RESULT_INVALID_STATE;
    assert_int_equal(sm_process_event(sys_get_sm_context(), event, &result), SMART_QSO_OK);
    return result;
}

/** IDLE -> ACTIVE as the main loop does it (sunlit, SOC >= 55 %) */
static void go_active(void)
{
    assert_int_equal(raise_event(EVENT_AI_READY), SM_RESULT_OK);
    assert_in(STATE_ACTIVE);
    assert_true(eps_is_payload_enabled());
}

/** Number of log entries of one type */
static size_t count_type(FaultType_t type)
{
    size_t n = 0U;
    FaultLogEntry_t entry;
    for (size_t i = 0U; i < fault_log_get_count(); i++) {
        assert_int_equal(fault_log_get_entry(i, &entry), SMART_QSO_OK);
        if (entry.fault_type == (uint8_t)type) {
            n++;
        }
    }
    return n;
}

/** Persistent fault log file, as fault_mgmt.c chooses it */
static const char *fault_log_file(void)
{
    const char *path = getenv(FAULT_LOG_FILE_ENV);
    return ((path != NULL) && (path[0] != '\0')) ? path : FAULT_LOG_FILE;
}

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

/** Booted as main.c leaves it (IDLE), with an empty fault log */
static int test_setup(void **state)
{
    (void)state;
    (void)fault_mgmt_init();
    payload_guard_init();
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_int_equal(sys_set_state_of_charge(START_SOC), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);
    assert_int_equal(raise_event(EVENT_DETUMBLE_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_IDLE);
    assert_int_equal(hal_timer_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_init(), SMART_QSO_OK);
    fault_clear_watchdog_flag();
    assert_int_equal(fault_log_clear(), SMART_QSO_OK);
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * Test Cases: Fault Logging
 ******************************************************************************/

/**
 * @brief A fault is logged with its type, severity, text and SOC
 */
static void test_fault_logged(void **state)
{
    (void)state;

    assert_int_equal(fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING,
                                   "Test fault", START_SOC), SMART_QSO_OK);

    assert_int_equal(fault_log_get_count(), 1U);
    FaultLogEntry_t entry;
    assert_int_equal(fault_log_get_last(&entry), SMART_QSO_OK);
    assert_int_equal(entry.fault_type, FAULT_TYPE_POWER);
    assert_int_equal(entry.severity, FAULT_SEVERITY_WARNING);
    assert_string_equal(entry.description, "Test fault");
    assert_true(fabs(entry.soc_at_fault - START_SOC) < 1e-9);
    assert_false(entry.recovered);
}

/**
 * @brief The fault log survives a reset: re-initializing reloads every entry
 *        from non-volatile memory, in order
 */
static void test_fault_log_persistence(void **state)
{
    (void)state;
    (void)fault_log_add(FAULT_TYPE_ADCS, FAULT_SEVERITY_WARNING, "Fault 1", START_SOC);
    (void)fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_ERROR, "Fault 2", START_SOC);
    (void)fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_CRITICAL, "Fault 3", START_SOC);

    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);   /* as on the next boot */

    assert_int_equal(fault_log_get_count(), 3U);
    FaultLogEntry_t entry;
    assert_int_equal(fault_log_get_entry(0U, &entry), SMART_QSO_OK);
    assert_string_equal(entry.description, "Fault 1");
    assert_int_equal(fault_log_get_entry(2U, &entry), SMART_QSO_OK);
    assert_string_equal(entry.description, "Fault 3");
    assert_int_equal(entry.severity, FAULT_SEVERITY_CRITICAL);
}

/*******************************************************************************
 * Test Cases: Power Faults
 ******************************************************************************/

/**
 * @brief Low battery (below 25 %): power recovery takes IDLE to SAFE in the
 *        state machine and the EPS, and logs a critical power fault
 *
 * @requirement SRS-F013 Transition to SAFE mode when SOC < 25%
 */
static void test_low_battery_fault(void **state)
{
    (void)state;
    assert_int_equal(sys_set_state_of_charge(0.20), SMART_QSO_OK);

    assert_int_equal(fault_recover_power(sys_get_state_of_charge()), SMART_QSO_OK);

    assert_in(STATE_SAFE);
    assert_int_equal(count_type(FAULT_TYPE_POWER_CRITICAL), 1U);
}

/**
 * @brief Critical battery (below 15 %): SAFE first, then EMERGENCY if it is
 *        still below 15 % in SAFE; EMERGENCY sheds the beacon
 */
static void test_critical_battery_fault(void **state)
{
    (void)state;
    assert_int_equal(sys_set_state_of_charge(0.05), SMART_QSO_OK);

    assert_int_equal(fault_recover_power(0.05), SMART_QSO_OK);
    assert_in(STATE_SAFE);
    assert_true(eps_is_beacon_enabled());

    assert_int_equal(fault_recover_power(0.05), SMART_QSO_OK);
    assert_in(STATE_EMERGENCY);
    assert_false(eps_is_beacon_enabled());
}

/*******************************************************************************
 * Test Cases: Communication Faults
 ******************************************************************************/

/**
 * @brief A UART timeout is logged as a UART fault
 */
static void test_uart_timeout_fault(void **state)
{
    (void)state;

    assert_int_equal(fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_WARNING,
                                   "UART timeout", START_SOC), SMART_QSO_OK);

    FaultLogEntry_t entry;
    assert_int_equal(fault_log_get_last(&entry), SMART_QSO_OK);
    assert_int_equal(entry.fault_type, FAULT_TYPE_UART);
    assert_string_equal(entry.description, "UART timeout");
}

/**
 * @brief No power or spreader measurement from the payload for longer than
 *        PG_TELEMETRY_TIMEOUT_MS: the payload guard trips, the rail goes off
 *        (as main.c's guard_payload() does) and cannot come back during the
 *        cool-down, not even by re-entering ACTIVE
 *
 * @requirement SRS-F033 Disable payload on communication timeout
 */
static void test_payload_timeout_fault(void **state)
{
    (void)state;
    go_active();
    uint64_t t0 = smart_qso_now_ms();
    assert_int_equal(payload_guard_step(NULL, true, t0), PG_OK);

    PayloadGuardResult_t trip = payload_guard_step(NULL, true, t0 + PG_TELEMETRY_TIMEOUT_MS + 1U);
    assert_int_equal(trip, PG_TRIP_NO_TELEMETRY);
    (void)fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_ERROR, payload_guard_reason(trip), START_SOC);
    assert_int_equal(eps_control_payload(false, START_SOC), SMART_QSO_OK);
    assert_false(eps_is_payload_enabled());

    assert_int_not_equal(eps_control_payload(true, START_SOC), SMART_QSO_OK);
    assert_false(eps_is_payload_enabled());
    assert_int_equal(raise_event(EVENT_GROUND_CMD_IDLE), SM_RESULT_OK);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_GUARD_FAILED);
    assert_in(STATE_IDLE);
    assert_false(eps_is_payload_enabled());
}

/*******************************************************************************
 * Test Cases: Sensor Faults
 ******************************************************************************/

/**
 * @brief An out-of-range bus voltage fails the regulation check and is
 *        logged as a voltage-range fault (main.c's health check, SW-16)
 */
static void test_sensor_range_fault(void **state)
{
    (void)state;
    assert_true(eps_bus_voltage_ok(5.0));
    assert_false(eps_bus_voltage_ok(EPS_BUS_MAX_V + 0.1));
    assert_false(eps_bus_voltage_ok(EPS_BUS_MIN_V - 0.1));
    assert_false(eps_bus_voltage_ok((double)NAN));

    (void)fault_log_add(FAULT_TYPE_VOLTAGE_RANGE, FAULT_SEVERITY_ERROR,
                        "Bus voltage out of range", START_SOC);
    assert_int_equal(count_type(FAULT_TYPE_VOLTAGE_RANGE), 1U);
}

/**
 * @brief A corrupted fault log entry fails its CRC on reload and is
 *        discarded; the intact entry is kept
 *
 * @requirement SRS-F062 Detect and handle corrupted data
 */
static void test_sensor_crc_fault(void **state)
{
    (void)state;
    (void)fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING, "Corrupt me", START_SOC);
    (void)fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_ERROR, "Keep me", START_SOC);

    FILE *f = fopen(fault_log_file(), "r+b");
    assert_non_null(f);
    assert_int_equal(fseek(f, (long)offsetof(FaultLogEntry_t, description), SEEK_SET), 0);
    assert_int_equal(fputc('X', f), 'X');
    assert_int_equal(fclose(f), 0);

    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);

    assert_int_equal(fault_log_get_count(), 1U);
    FaultLogEntry_t entry;
    assert_int_equal(fault_log_get_entry(0U, &entry), SMART_QSO_OK);
    assert_string_equal(entry.description, "Keep me");
}

/*******************************************************************************
 * Test Cases: Thermal Faults
 ******************************************************************************/

/**
 * @brief Over-temperature with the payload on: thermal recovery turns the
 *        payload off and the state machine takes ACTIVE to SAFE, with the
 *        EPS in the same mode
 *
 * @requirement SRS-S005 Limit power consumption to prevent thermal damage
 */
static void test_over_temp_fault(void **state)
{
    (void)state;
    go_active();
    assert_int_equal(sys_set_temperature(0U, 70.0f), SMART_QSO_OK);   /* OBC at 70 C */
    assert_true(sys_has_thermal_fault());

    assert_int_equal(fault_recover_thermal(START_SOC), SMART_QSO_OK);

    assert_false(eps_is_payload_enabled());
    assert_in(STATE_SAFE);
    assert_int_equal(count_type(FAULT_TYPE_THERMAL), 1U);
}

/**
 * @brief Under-temperature is a thermal fault too
 */
static void test_under_temp_fault(void **state)
{
    (void)state;
    assert_int_equal(sys_set_temperature(2U, -25.0f), SMART_QSO_OK);   /* battery at -25 C */
    assert_true(sys_has_thermal_fault());

    assert_int_equal(sys_set_temperature(2U, 20.0f), SMART_QSO_OK);
    assert_false(sys_has_thermal_fault());
}

/*******************************************************************************
 * Test Cases: Watchdog Faults
 ******************************************************************************/

/**
 * @brief A task that stops checking in is detected as stalled, and the
 *        watchdog manager logs it; the watchdog-timeout handler logs again
 *        and sets its flag; checking in again clears the stall
 *
 * @requirement SRS-F052 Detect watchdog reset and log occurrence
 */
static void test_task_stall_detection(void **state)
{
    (void)state;
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "main", 1000U), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_start(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_checkin(WDT_TASK_MAIN_LOOP), SMART_QSO_OK);
    assert_false(wdt_mgr_is_task_stalled(WDT_TASK_MAIN_LOOP));

    for (uint32_t i = 0U; i <= WDT_TASK_MISS_THRESHOLD + 1U; i++) {
        smart_qso_sim_advance_ms(1000U);
        wdt_mgr_kick();
    }
    assert_true(wdt_mgr_is_task_stalled(WDT_TASK_MAIN_LOOP));
    assert_int_equal(count_type(FAULT_TYPE_WATCHDOG), 1U);   /* the stall */

    assert_int_equal(fault_handle_watchdog(START_SOC), SMART_QSO_OK);
    assert_true(fault_was_watchdog_triggered());
    assert_int_equal(count_type(FAULT_TYPE_WATCHDOG), 2U);

    assert_int_equal(wdt_mgr_checkin(WDT_TASK_MAIN_LOOP), SMART_QSO_OK);
    assert_false(wdt_mgr_is_task_stalled(WDT_TASK_MAIN_LOOP));
}

/*******************************************************************************
 * Test Cases: Recovery Procedures
 ******************************************************************************/

/**
 * @brief Power recovery from SAFE once the battery is above 40 % again
 *        (was fault_recover_eps(); the EPS recovery is fault_recover_power())
 *
 * @requirement SRS-F044 Attempt recovery from SAFE mode when SOC improves
 */
static void test_eps_recovery(void **state)
{
    (void)state;
    assert_int_equal(fault_recover_power(0.20), SMART_QSO_OK);
    assert_in(STATE_SAFE);

    assert_int_equal(sys_set_state_of_charge(0.45), SMART_QSO_OK);
    assert_int_equal(fault_recover_power(0.45), SMART_QSO_OK);
    assert_in(STATE_IDLE);
}

/**
 * @brief Full recovery from EMERGENCY: SAFE at 25 % with the beacon back,
 *        then IDLE above 40 %
 */
static void test_system_recovery(void **state)
{
    (void)state;
    (void)fault_recover_power(0.10);
    (void)fault_recover_power(0.10);
    assert_in(STATE_EMERGENCY);

    assert_int_equal(fault_recover_power(EPS_SOC_SAFE_THRESHOLD), SMART_QSO_OK);
    assert_in(STATE_SAFE);
    assert_true(eps_is_beacon_enabled());

    assert_int_equal(fault_recover_power(0.45), SMART_QSO_OK);
    assert_in(STATE_IDLE);
}

/*******************************************************************************
 * Test Cases: Fault Statistics
 ******************************************************************************/

/**
 * @brief Each entry keeps its severity across a reset, so the log can be
 *        tallied by severity (there is no separate statistics API)
 *
 * @requirement SRS-F045 Track fault statistics across resets
 */
static void test_fault_statistics(void **state)
{
    (void)state;
    (void)fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING, "EPS fault", START_SOC);
    (void)fault_log_add(FAULT_TYPE_ADCS, FAULT_SEVERITY_ERROR, "Sensor fault", START_SOC);
    (void)fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_CRITICAL, "Comm fault", START_SOC);
    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);   /* across a reset */

    size_t by_severity[FAULT_SEVERITY_CRITICAL + 1] = {0U};
    FaultLogEntry_t entry;
    for (size_t i = 0U; i < fault_log_get_count(); i++) {
        assert_int_equal(fault_log_get_entry(i, &entry), SMART_QSO_OK);
        assert_true(entry.severity <= (uint8_t)FAULT_SEVERITY_CRITICAL);
        by_severity[entry.severity]++;
    }

    assert_int_equal(fault_log_get_count(), 3U);
    assert_int_equal(by_severity[FAULT_SEVERITY_WARNING], 1U);
    assert_int_equal(by_severity[FAULT_SEVERITY_ERROR], 1U);
    assert_int_equal(by_severity[FAULT_SEVERITY_CRITICAL], 1U);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Fault Logging */
        cmocka_unit_test_setup_teardown(test_fault_logged,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_persistence,
                                        test_setup, test_teardown),

        /* Power Faults */
        cmocka_unit_test_setup_teardown(test_low_battery_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_critical_battery_fault,
                                        test_setup, test_teardown),

        /* Communication Faults */
        cmocka_unit_test_setup_teardown(test_uart_timeout_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_payload_timeout_fault,
                                        test_setup, test_teardown),

        /* Sensor Faults */
        cmocka_unit_test_setup_teardown(test_sensor_range_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_sensor_crc_fault,
                                        test_setup, test_teardown),

        /* Thermal Faults */
        cmocka_unit_test_setup_teardown(test_over_temp_fault,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_under_temp_fault,
                                        test_setup, test_teardown),

        /* Watchdog Faults */
        cmocka_unit_test_setup_teardown(test_task_stall_detection,
                                        test_setup, test_teardown),

        /* Recovery */
        cmocka_unit_test_setup_teardown(test_eps_recovery,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_system_recovery,
                                        test_setup, test_teardown),

        /* Statistics */
        cmocka_unit_test_setup_teardown(test_fault_statistics,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests_name("Fault Scenario Integration Tests", tests, NULL, NULL);
}
