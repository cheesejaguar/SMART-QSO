/**
 * @file test_mode_transitions.c
 * @brief Integration tests for operational mode transitions
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The state machine (state_machine.c), its power policy (sm_power_event())
 * and the EPS together, as main.c drives them: the EPS holds the measured
 * SOC, the policy turns it into an event, the transition table and guards
 * decide, and each state's entry action sets the EPS power mode (SW-14).
 *
 * State model (state_machine.h; thresholds docs/BASELINE_PARAMETERS.md §5):
 *
 *   BOOT --boot complete--> DETUMBLE --detumble complete--> IDLE <--> ACTIVE
 *
 *   ACTIVE allowed at SOC >= 55 % (sunlit, payload guard clear); ACTIVE ->
 *   IDLE below 40 %; BOOT/DETUMBLE/IDLE/ACTIVE -> SAFE below 25 %; SAFE ->
 *   EMERGENCY (beacon shed) below 15 %; EMERGENCY -> SAFE at 25 %; SAFE ->
 *   IDLE above 40 %.
 *
 * @requirement SRS-F012 Implement power modes: SAFE, IDLE, ACTIVE
 * @requirement SRS-F013 Transition to SAFE mode when SOC < 25%
 * @requirement SRS-F014 Transition to IDLE mode when SOC < 40%
 * @requirement SRS-F015 Enable payload only when SOC > 55% and sunlit
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "smart_qso.h"
#include "system_state.h"
#include "state_machine.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "payload_guard.h"

/** SOC main.c's simulation starts with */
#define START_SOC 0.75

/** Just above / below a threshold */
#define STEP 0.01

/*******************************************************************************
 * Helpers
 ******************************************************************************/

/** Raise an event with the system's state machine; return the result */
static SmResult_t raise_event(SmEvent_t event)
{
    SmResult_t result = SM_RESULT_INVALID_STATE;
    assert_int_equal(sm_process_event(sys_get_sm_context(), event, &result), SMART_QSO_OK);
    return result;
}

/** One pass of main.c's mode logic (update_mission_state) at this SOC */
static void power_tick(double soc, bool sunlit)
{
    SmState_t st = STATE_BOOT;
    eps_update_soc(soc);
    assert_int_equal(sm_get_state(sys_get_sm_context(), &st), SMART_QSO_OK);
    SmEvent_t event = sm_power_event(st, soc, sunlit);
    if (event != EVENT_NONE) {
        (void)raise_event(event);
    }
}

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

/** The system is in this state and the EPS in that state's power mode */
static void assert_in(SmState_t expected)
{
    assert_int_equal(sys_get_operational_state(), expected);
    assert_int_equal(eps_get_power_mode(), power_mode_of(expected));
    assert_int_equal(eps_is_payload_enabled(), expected == STATE_ACTIVE);
    assert_int_equal(eps_is_beacon_enabled(), expected != STATE_EMERGENCY);
}

/** Boot as main.c's init_mode_manager() does: BOOT -> DETUMBLE -> IDLE */
static void boot_to_idle(void)
{
    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);
    assert_int_equal(raise_event(EVENT_DETUMBLE_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_IDLE);
}

/** IDLE -> ACTIVE the way the main loop gets there: sunlit, SOC >= 55 % */
static void boot_to_active(void)
{
    boot_to_idle();
    power_tick(START_SOC, true);
    assert_in(STATE_ACTIVE);
}

/** IDLE -> SAFE on a critical battery */
static void boot_to_safe(void)
{
    boot_to_idle();
    power_tick(EPS_SOC_SAFE_THRESHOLD - STEP, false);
    assert_in(STATE_SAFE);
}

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

/** System state as main.c starts it: EPS, payload guard, state machine in BOOT */
static int test_setup(void **state)
{
    (void)state;
    (void)fault_mgmt_init();
    payload_guard_init();
    assert_int_equal(eps_init(), SMART_QSO_OK);
    eps_update_soc(START_SOC);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    return 0;
}

static int test_teardown(void **state)
{
    (void)state;
    return 0;
}

/*******************************************************************************
 * Test Cases: BOOT State Transitions
 ******************************************************************************/

/**
 * @brief BOOT -> DETUMBLE on boot complete (the old model went BOOT -> SAFE)
 */
static void test_boot_to_detumble(void **state)
{
    (void)state;
    assert_in(STATE_BOOT);

    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_DETUMBLE);
}

/**
 * @brief BOOT -> SAFE on a critical battery; EMERGENCY is reached only from
 *        SAFE (the old model sent any critical fault straight to EMERGENCY)
 */
static void test_boot_critical_power_to_safe(void **state)
{
    (void)state;
    power_tick(EPS_SOC_SAFE_THRESHOLD - STEP, false);
    assert_in(STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: SAFE State Transitions
 ******************************************************************************/

/**
 * @brief SAFE -> IDLE only above 40 %: hysteresis over the 25 % entry
 *
 * @requirement SRS-F044 Attempt recovery from SAFE mode when SOC improves
 */
static void test_safe_to_idle_above_40(void **state)
{
    (void)state;
    boot_to_safe();

    power_tick(0.30, true);
    assert_in(STATE_SAFE);
    power_tick(EPS_SOC_IDLE_THRESHOLD, true);
    assert_in(STATE_SAFE);
    power_tick(EPS_SOC_IDLE_THRESHOLD + STEP, true);
    assert_in(STATE_IDLE);
}

/**
 * @brief SAFE -> EMERGENCY when SOC keeps falling below 15 %; the beacon is
 *        shed, the receiver stays on
 */
static void test_safe_to_emergency(void **state)
{
    (void)state;
    boot_to_safe();

    power_tick(0.20, false);
    assert_in(STATE_SAFE);
    power_tick(EPS_SOC_EMERGENCY_THRESHOLD - STEP, false);
    assert_in(STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: DETUMBLE State Transitions
 ******************************************************************************/

/**
 * @brief DETUMBLE -> IDLE on detumble complete
 */
static void test_detumble_to_idle(void **state)
{
    (void)state;
    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_DETUMBLE);

    assert_int_equal(raise_event(EVENT_DETUMBLE_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_IDLE);
}

/**
 * @brief DETUMBLE -> SAFE on a critical battery (old model: EMERGENCY)
 */
static void test_detumble_critical_power_to_safe(void **state)
{
    (void)state;
    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);

    power_tick(EPS_SOC_SAFE_THRESHOLD - STEP, false);
    assert_in(STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: IDLE State Transitions
 ******************************************************************************/

/**
 * @brief IDLE -> ACTIVE at 55 % in sunlight, not in eclipse or below 55 %
 */
static void test_idle_to_active(void **state)
{
    (void)state;
    boot_to_idle();

    power_tick(0.90, false);
    assert_in(STATE_IDLE);
    power_tick(EPS_SOC_PAYLOAD_THRESHOLD - STEP, true);
    assert_in(STATE_IDLE);
    power_tick(EPS_SOC_PAYLOAD_THRESHOLD, true);
    assert_in(STATE_ACTIVE);
}

/**
 * @brief The ACTIVE guard: the battery and the payload guard must allow it,
 *        for the onboard event and the ground command alike
 */
static void test_idle_to_active_guarded(void **state)
{
    (void)state;
    boot_to_idle();

    eps_update_soc(EPS_SOC_PAYLOAD_THRESHOLD - STEP);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_GUARD_FAILED);
    assert_in(STATE_IDLE);

    eps_update_soc(0.90);
    PayloadSample_t latchup;
    memset(&latchup, 0, sizeof(latchup));
    latchup.power_valid = true;
    latchup.input_w = PG_LATCHUP_INPUT_W;
    (void)payload_guard_step(&latchup, true, smart_qso_now_ms());   /* trip: cool-down */
    assert_int_equal(raise_event(EVENT_AI_READY), SM_RESULT_GUARD_FAILED);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_GUARD_FAILED);
    assert_in(STATE_IDLE);

    payload_guard_init();
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_OK);
    assert_in(STATE_ACTIVE);
}

/**
 * @brief IDLE -> SAFE on a critical battery (old model: EMERGENCY)
 */
static void test_idle_critical_power_to_safe(void **state)
{
    (void)state;
    boot_to_idle();

    power_tick(0.30, true);
    assert_in(STATE_IDLE);
    power_tick(EPS_SOC_SAFE_THRESHOLD - STEP, true);
    assert_in(STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: ACTIVE State Transitions
 ******************************************************************************/

/**
 * @brief ACTIVE -> IDLE below 40 %, not between 40 % and 55 % (hysteresis)
 */
static void test_active_to_idle(void **state)
{
    (void)state;
    boot_to_active();

    power_tick(0.45, true);
    assert_in(STATE_ACTIVE);
    power_tick(EPS_SOC_IDLE_THRESHOLD - STEP, true);
    assert_in(STATE_IDLE);
}

/**
 * @brief ACTIVE -> SAFE directly on a critical battery (old model: EMERGENCY)
 */
static void test_active_critical_power_to_safe(void **state)
{
    (void)state;
    boot_to_active();

    power_tick(EPS_SOC_SAFE_THRESHOLD - STEP, true);
    assert_in(STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: EMERGENCY State Transitions
 ******************************************************************************/

/**
 * @brief EMERGENCY -> SAFE once SOC is back at 25 %; the beacon returns
 */
static void test_emergency_to_safe(void **state)
{
    (void)state;
    boot_to_safe();
    power_tick(EPS_SOC_EMERGENCY_THRESHOLD - STEP, false);
    assert_in(STATE_EMERGENCY);

    power_tick(EPS_SOC_SAFE_THRESHOLD - STEP, true);
    assert_in(STATE_EMERGENCY);
    power_tick(EPS_SOC_SAFE_THRESHOLD, true);
    assert_in(STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: Invalid Transitions
 ******************************************************************************/

/**
 * @brief Transitions not in the table are refused and change nothing:
 *        BOOT -> ACTIVE, and SAFE -> ACTIVE without IDLE first
 */
static void test_invalid_transition_rejected(void **state)
{
    (void)state;
    eps_update_soc(0.90);

    assert_int_equal(raise_event(EVENT_AI_READY), SM_RESULT_NO_TRANSITION);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_NO_TRANSITION);
    assert_in(STATE_BOOT);

    assert_int_equal(raise_event(EVENT_POWER_CRITICAL), SM_RESULT_OK);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_ACTIVE), SM_RESULT_NO_TRANSITION);
    assert_int_equal(raise_event(EVENT_AI_READY), SM_RESULT_NO_TRANSITION);
    assert_in(STATE_SAFE);
}

/**
 * @brief The EPS power mode follows every state, from boot to EMERGENCY
 *        and back
 */
static void test_power_mode_follows_state(void **state)
{
    (void)state;

    assert_in(STATE_BOOT);                       /* BOOT holds SAFE power */
    assert_int_equal(raise_event(EVENT_BOOT_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_DETUMBLE);
    assert_int_equal(raise_event(EVENT_DETUMBLE_COMPLETE), SM_RESULT_OK);
    assert_in(STATE_IDLE);
    power_tick(START_SOC, true);
    assert_in(STATE_ACTIVE);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_SAFE), SM_RESULT_OK);
    assert_in(STATE_SAFE);
    power_tick(0.10, false);
    assert_in(STATE_EMERGENCY);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_SAFE), SM_RESULT_OK);
    assert_in(STATE_SAFE);
    assert_int_equal(raise_event(EVENT_GROUND_CMD_IDLE), SM_RESULT_OK);
    assert_in(STATE_IDLE);
}

/**
 * @brief A discharge from ACTIVE to EMERGENCY and the recharge back, one
 *        main-loop pass per step
 */
static void test_discharge_and_recharge(void **state)
{
    (void)state;
    static const struct {
        double soc;
        bool sunlit;
        SmState_t expect;
    } steps[] = {
        {0.50, false, STATE_ACTIVE},     /* between 40 and 55 %: stays */
        {0.39, false, STATE_IDLE},
        {0.26, false, STATE_IDLE},
        {0.24, false, STATE_SAFE},
        {0.16, false, STATE_SAFE},
        {0.14, false, STATE_EMERGENCY},
        {0.24, true, STATE_EMERGENCY},
        {0.25, true, STATE_SAFE},
        {0.40, true, STATE_SAFE},
        {0.41, true, STATE_IDLE},
        {0.54, true, STATE_IDLE},
        {0.55, true, STATE_ACTIVE},
    };
    boot_to_active();

    for (size_t i = 0U; i < (sizeof(steps) / sizeof(steps[0])); i++) {
        power_tick(steps[i].soc, steps[i].sunlit);
        assert_in(steps[i].expect);
    }
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* BOOT Transitions */
        cmocka_unit_test_setup_teardown(test_boot_to_detumble,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_boot_critical_power_to_safe,
                                        test_setup, test_teardown),

        /* SAFE Transitions */
        cmocka_unit_test_setup_teardown(test_safe_to_idle_above_40,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_safe_to_emergency,
                                        test_setup, test_teardown),

        /* DETUMBLE Transitions */
        cmocka_unit_test_setup_teardown(test_detumble_to_idle,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_detumble_critical_power_to_safe,
                                        test_setup, test_teardown),

        /* IDLE Transitions */
        cmocka_unit_test_setup_teardown(test_idle_to_active,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_idle_to_active_guarded,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_idle_critical_power_to_safe,
                                        test_setup, test_teardown),

        /* ACTIVE Transitions */
        cmocka_unit_test_setup_teardown(test_active_to_idle,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_active_critical_power_to_safe,
                                        test_setup, test_teardown),

        /* EMERGENCY Transitions */
        cmocka_unit_test_setup_teardown(test_emergency_to_safe,
                                        test_setup, test_teardown),

        /* Invalid Transitions and the EPS */
        cmocka_unit_test_setup_teardown(test_invalid_transition_rejected,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_power_mode_follows_state,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_discharge_and_recharge,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests_name("Mode Transition Integration Tests", tests, NULL, NULL);
}
