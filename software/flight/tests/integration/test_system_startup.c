/**
 * @file test_system_startup.c
 * @brief Integration tests for the system startup sequence
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Runs the initialization sequence of main.c's init_system() (main.c itself
 * cannot be linked into a test) and checks the state it leaves:
 * - initialization order and every step succeeding
 * - the mode manager: BOOT (SAFE power) -> DETUMBLE -> IDLE
 * - the EPS, the watchdog and the fault log after a boot
 * - error handling (corrupt persisted data) and the reset counter
 * - startup time
 *
 * Left out of the sequence: signal handlers, srand(), sensors.yaml (the
 * built-in defaults are loaded, as main.c does when the file is missing)
 * and the Jetson UART.
 *
 * @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "smart_qso.h"
#include "system_state.h"
#include "state_machine.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "watchdog_mgr.h"
#include "mission_data.h"
#include "sensors.h"
#include "beacon.h"
#include "payload_guard.h"
#include "deployment.h"
#include "rf_silence.h"
#include "rf_control.h"
#include "contact_monitor.h"
#include "cmd_handler.h"
#include "uplink.h"
#include "gse_provision.h"
#include "station_id.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_timer.h"

/** SOC main.c's simulation starts with */
#define START_SOC 0.75

/** Startup budget (SRS-P008, SRS-F001) */
#define STARTUP_BUDGET_MS 30000U

/** main.c's health-check interval */
#define HEALTH_CHECK_INTERVAL_MS 10000U

/*******************************************************************************
 * Helpers
 ******************************************************************************/

static const char *file_or_default(const char *env, const char *fallback)
{
    const char *path = getenv(env);
    return ((path != NULL) && (path[0] != '\0')) ? path : fallback;
}

/** As main.c's watchdog pre-reset action (save_state_before_reset) */
static void save_state_before_reset(void)
{
    (void)mission_data_save();
    (void)eps_save_config();
}

/** Radio safety and telecommand (main.c: init_rf_and_telecommand) */
static void init_rf_and_telecommand(void)
{
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    assert_int_equal(uplink_init(SQSO_CALLSIGN, SQSO_SSID), SMART_QSO_OK);
    assert_int_equal(gse_init(), SMART_QSO_OK);
}

/** Mission data, EPS and sensors (main.c: init_power_and_sensors) */
static void init_power_and_sensors(void)
{
    assert_int_equal(mission_data_init(), SMART_QSO_OK);
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_int_equal(sensors_init(), SMART_QSO_OK);
    assert_int_equal(sensors_load_defaults(), SMART_QSO_OK);
}

/** Mode manager BOOT -> DETUMBLE -> IDLE (main.c: init_mode_manager) */
static void init_mode_manager(double soc)
{
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    payload_guard_init();
    eps_update_soc(soc);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    SmResult_t result = SM_RESULT_INVALID_STATE;
    assert_int_equal(sm_process_event(sys_get_sm_context(), EVENT_BOOT_COMPLETE, &result), SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(sm_process_event(sys_get_sm_context(), EVENT_DETUMBLE_COMPLETE, &result), SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
}

/** Supervised hardware watchdog (main.c: init_watchdog) */
static void init_watchdog(void)
{
    assert_int_equal(hal_timer_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_init(), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "main", 1000U), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_COMM, "uplink", 1000U), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_PAYLOAD, "payload", 1000U), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_BEACON, "beacon", 1000U), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_task(WDT_TASK_HEALTH, "health",
                                           2U * HEALTH_CHECK_INTERVAL_MS), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_register_prereset_callback(save_state_before_reset), SMART_QSO_OK);
    assert_int_equal(wdt_mgr_start(), SMART_QSO_OK);
}

/** main.c's init_system(), in its order */
static void boot_as_main(double soc)
{
    /* HAL first; burn-wire lines low early; fault logging before the rest */
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    (void)deployment_init();
    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);

    init_rf_and_telecommand();
    init_power_and_sensors();
    init_mode_manager(soc);
    init_watchdog();
}

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

/** A first boot: no persisted files and blank non-volatile memory */
static int test_setup(void **state)
{
    (void)state;
    (void)remove(file_or_default(FAULT_LOG_FILE_ENV, FAULT_LOG_FILE));
    (void)remove(file_or_default(EPS_CONFIG_FILE_ENV, EPS_CONFIG_FILE));
    (void)remove(file_or_default(MISSION_DATA_FILE_ENV, MISSION_DATA_FILE));
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    for (int region = 0; region < (int)FLASH_REGION_COUNT; region++) {
        assert_int_equal(hal_flash_erase((HalFlashRegion_t)region), SMART_QSO_OK);
    }
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * Test Cases: Initialization Order
 ******************************************************************************/

/**
 * @brief The main.c sequence succeeds step by step and ends with the mode
 *        manager in IDLE, reached through DETUMBLE, the EPS in IDLE and the
 *        radio silent (post-deployment RF silence on a first boot)
 *
 * @requirement SRS-S001 Do not enable RF transmission until after separation
 */
static void test_init_order(void **state)
{
    (void)state;
    boot_as_main(START_SOC);

    SmState_t previous = STATE_BOOT;
    uint32_t transitions = 0U;
    assert_int_equal(sm_get_statistics(sys_get_sm_context(), &transitions, &previous), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_int_equal(previous, STATE_DETUMBLE);
    assert_int_equal(transitions, 2U);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);
    assert_true(sensors_get_count() > 0U);
    assert_true(rf_silence_active());
    assert_false(rf_control_tx_permitted());
}

/**
 * @brief Until boot completes the state machine is in BOOT, which holds the
 *        EPS in SAFE power (payload off, beacon on); boot complete leads to
 *        DETUMBLE, then IDLE (the old model left boot for SAFE)
 */
static void test_boot_holds_safe_power(void **state)
{
    (void)state;
    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);
    init_power_and_sensors();
    eps_update_soc(START_SOC);

    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_BOOT);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
    assert_false(eps_is_payload_enabled());
    assert_true(eps_is_beacon_enabled());

    assert_int_equal(sm_process_event(sys_get_sm_context(), EVENT_BOOT_COMPLETE, NULL), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_DETUMBLE);
    assert_int_equal(sm_process_event(sys_get_sm_context(), EVENT_DETUMBLE_COMPLETE, NULL), SMART_QSO_OK);
    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
}

/**
 * @brief After startup the payload is off, even if the EPS state saved
 *        before the reset had it on
 *
 * @requirement SRS-F017 Persist EPS state across resets
 */
static void test_eps_init_values(void **state)
{
    (void)state;
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_false(eps_is_payload_enabled());           /* first-boot defaults */
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);

    payload_guard_init();
    assert_int_equal(eps_control_payload(true, 0.9), SMART_QSO_OK);   /* saved on */
    assert_true(eps_is_payload_enabled());

    boot_as_main(START_SOC);

    assert_false(eps_is_payload_enabled());
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);
}

/**
 * @brief After startup the hardware watchdog runs, the five main-loop stages
 *        are supervised, and a pass in which every stage checks in refreshes
 *        the hardware
 *
 * @requirement SRS-F050 Kick hardware watchdog at least every 30 seconds
 */
static void test_watchdog_starts(void **state)
{
    (void)state;
    boot_as_main(START_SOC);

    WdtStats_t stats;
    assert_int_equal(wdt_mgr_get_stats(&stats), SMART_QSO_OK);
    assert_true(stats.hw_wdt_running);
    static const WdtTaskId_t stages[] = {
        WDT_TASK_MAIN_LOOP, WDT_TASK_COMM, WDT_TASK_PAYLOAD, WDT_TASK_BEACON, WDT_TASK_HEALTH
    };
    WdtTaskEntry_t entry;
    for (size_t i = 0U; i < (sizeof(stages) / sizeof(stages[0])); i++) {
        assert_int_equal(wdt_mgr_get_task_info(stages[i], &entry), SMART_QSO_OK);
        assert_true(entry.active);
        assert_int_equal(wdt_mgr_checkin(stages[i]), SMART_QSO_OK);
    }

    uint32_t kicks = stats.hw_kicks;
    wdt_mgr_kick();
    assert_int_equal(wdt_mgr_get_stats(&stats), SMART_QSO_OK);
    assert_int_equal(stats.hw_kicks, kicks + 1U);
    assert_false(wdt_mgr_any_task_stalled());
}

/**
 * @brief With no persisted fault log, the log starts empty; a saved log is
 *        reloaded on the next boot
 *
 * @requirement SRS-F041 Maintain fault log in NVM
 */
static void test_fault_log_clean_after_init(void **state)
{
    (void)state;
    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);
    assert_int_equal(fault_log_get_count(), 0U);

    (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_WARNING, "Before reset", START_SOC);
    assert_int_equal(fault_mgmt_init(), SMART_QSO_OK);
    assert_int_equal(fault_log_get_count(), 1U);
}

/*******************************************************************************
 * Test Cases: Startup Error Handling
 ******************************************************************************/

/**
 * @brief A corrupt EPS configuration (CRC mismatch) is replaced by the
 *        defaults and startup completes
 *
 * @requirement SRS-F062 Detect and handle corrupted data
 */
static void test_handles_eps_init_failure(void **state)
{
    (void)state;
    FILE *f = fopen(file_or_default(EPS_CONFIG_FILE_ENV, EPS_CONFIG_FILE), "wb");
    assert_non_null(f);
    static const uint8_t garbage[sizeof(EpsControlState_t)] = {0x01U, 0x02U, 0x03U};
    assert_int_equal(fwrite(garbage, sizeof(garbage), 1U, f), 1U);
    assert_int_equal(fclose(f), 0);

    assert_int_not_equal(eps_load_config(), SMART_QSO_OK);
    boot_as_main(START_SOC);

    assert_int_equal(sys_get_operational_state(), STATE_IDLE);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);
    assert_false(eps_is_payload_enabled());
}

/**
 * @brief The reset counter in mission data goes up by one per boot, with the
 *        state saved before the reset as main.c's watchdog pre-reset action
 *        does
 *
 * @requirement SRS-F004 Increment reset counter on each boot
 */
static void test_boot_count_increments(void **state)
{
    (void)state;
    boot_as_main(START_SOC);
    uint32_t first = mission_data_get_reset_count();
    assert_int_equal(first, 1U);                       /* first boot */

    save_state_before_reset();
    boot_as_main(START_SOC);
    assert_int_equal(mission_data_get_reset_count(), first + 1U);

    save_state_before_reset();
    boot_as_main(START_SOC);
    assert_int_equal(mission_data_get_reset_count(), first + 2U);
}

/*******************************************************************************
 * Test Cases: Startup Timing
 ******************************************************************************/

/**
 * @brief Initialization completes within the startup budget
 *
 * @requirement SRS-P008 System initialization shall complete within 30 seconds
 */
static void test_startup_timing(void **state)
{
    (void)state;
    uint64_t start = smart_qso_now_ms();

    boot_as_main(START_SOC);

    assert_true((smart_qso_now_ms() - start) < STARTUP_BUDGET_MS);
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization Order */
        cmocka_unit_test_setup_teardown(test_init_order,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_boot_holds_safe_power,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_eps_init_values,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_watchdog_starts,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_clean_after_init,
                                        test_setup, test_teardown),

        /* Error Handling */
        cmocka_unit_test_setup_teardown(test_handles_eps_init_failure,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_boot_count_increments,
                                        test_setup, test_teardown),

        /* Timing */
        cmocka_unit_test_setup_teardown(test_startup_timing,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests_name("System Startup Integration Tests", tests, NULL, NULL);
}
