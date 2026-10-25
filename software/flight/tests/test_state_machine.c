/**
 * @file test_state_machine.c
 * @brief Unit tests for state machine module
 *
 * Tests state machine transitions and guards per NPR 7150.2.
 *
 * @requirement SRS-F012 Implement power modes SAFE, IDLE, ACTIVE (formal state machine)
 * @requirement SRS-F018 Log every operating-mode transition ([Proposed])
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

/* Include the module under test */
#include "state_machine.h"
#include "eps_control.h"
#include "payload_guard.h"
#include "smart_qso.h"

/*===========================================================================*/
/* Test Fixtures                                                              */
/*===========================================================================*/

static SmContext_t g_ctx;

/**
 * @brief Setup function - initialize state machine before each test
 */
static int setup(void **state)
{
    (void)state;
    memset(&g_ctx, 0, sizeof(g_ctx));
    /* Healthy battery and payload unless a test says otherwise */
    payload_guard_init();
    (void)eps_init();
    eps_update_soc(0.9);
    return 0;
}

/*===========================================================================*/
/* Initialization Tests                                                       */
/*===========================================================================*/

/**
 * @brief Test state machine initialization
 */
static void test_sm_init_success(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sm_init(&g_ctx, STATE_BOOT);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(g_ctx.is_initialized);
    assert_int_equal(g_ctx.current_state, STATE_BOOT);
    assert_int_equal(g_ctx.transition_count, 0);
}

/**
 * @brief Test initialization with NULL context
 */
static void test_sm_init_null_context(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sm_init(NULL, STATE_BOOT);

    assert_int_equal(ret, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Test initialization with invalid state
 */
static void test_sm_init_invalid_state(void **state)
{
    (void)state;

    SmartQsoResult_t ret = sm_init(&g_ctx, (SmState_t)99);

    assert_int_equal(ret, SMART_QSO_ERROR_INVALID);
}

/*===========================================================================*/
/* State Transition Tests                                                     */
/*===========================================================================*/

/**
 * @brief Test valid transition from BOOT to DETUMBLE
 */
static void test_sm_boot_to_detumble(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_BOOT);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_BOOT_COMPLETE, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_DETUMBLE);
    assert_int_equal(g_ctx.previous_state, STATE_BOOT);
    assert_int_equal(g_ctx.transition_count, 1);
}

/**
 * @brief Test valid transition from DETUMBLE to IDLE
 */
static void test_sm_detumble_to_idle(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_DETUMBLE);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_DETUMBLE_COMPLETE, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_IDLE);
}

/**
 * @brief Test transition from IDLE to ACTIVE
 */
static void test_sm_idle_to_active(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_IDLE);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_AI_READY, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_ACTIVE);
}

/**
 * @brief Test transition to SAFE mode on power low
 */
static void test_sm_idle_to_safe_power_low(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_IDLE);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_POWER_LOW, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_SAFE);
}

/**
 * @brief Test transition from ACTIVE to IDLE on AI fault
 */
static void test_sm_active_to_idle_ai_fault(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_ACTIVE);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_AI_FAULT, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_IDLE);
}

/**
 * @brief Test ground command transition to SAFE
 */
static void test_sm_ground_cmd_safe(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_ACTIVE);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_GROUND_CMD_SAFE, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_SAFE);
}

/**
 * @brief Test invalid transition (no transition defined)
 */
static void test_sm_no_transition(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_BOOT);

    SmResult_t result;
    /* Try to go directly to ACTIVE from BOOT - not allowed */
    SmartQsoResult_t ret = sm_process_event(&g_ctx, EVENT_AI_READY, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_NO_TRANSITION);
    assert_int_equal(g_ctx.current_state, STATE_BOOT);  /* State unchanged */
}

/**
 * @brief Test invalid event handling
 */
static void test_sm_invalid_event(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_BOOT);

    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(&g_ctx, (SmEvent_t)99, &result);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_INVALID_EVENT);
}

/*===========================================================================*/
/* State Query Tests                                                          */
/*===========================================================================*/

/**
 * @brief Test get current state
 */
static void test_sm_get_state(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_IDLE);

    SmState_t current;
    SmartQsoResult_t ret = sm_get_state(&g_ctx, &current);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(current, STATE_IDLE);
}

/**
 * @brief Test get state name
 */
static void test_sm_get_state_name(void **state)
{
    (void)state;

    assert_string_equal(sm_get_state_name(STATE_BOOT), "BOOT");
    assert_string_equal(sm_get_state_name(STATE_SAFE), "SAFE");
    assert_string_equal(sm_get_state_name(STATE_DETUMBLE), "DETUMBLE");
    assert_string_equal(sm_get_state_name(STATE_IDLE), "IDLE");
    assert_string_equal(sm_get_state_name(STATE_ACTIVE), "ACTIVE");
    assert_string_equal(sm_get_state_name(STATE_EMERGENCY), "EMERGENCY");
    assert_string_equal(sm_get_state_name((SmState_t)99), "UNKNOWN");
}

/**
 * @brief Test get event name
 */
static void test_sm_get_event_name(void **state)
{
    (void)state;

    assert_string_equal(sm_get_event_name(EVENT_BOOT_COMPLETE), "BOOT_COMPLETE");
    assert_string_equal(sm_get_event_name(EVENT_POWER_LOW), "POWER_LOW");
    assert_string_equal(sm_get_event_name(EVENT_AI_READY), "AI_READY");
    assert_string_equal(sm_get_event_name((SmEvent_t)99), "UNKNOWN");
}

/**
 * @brief Test transition validity check
 */
static void test_sm_is_transition_valid(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_BOOT);

    bool is_valid;

    /* Valid transition */
    SmartQsoResult_t ret = sm_is_transition_valid(&g_ctx, EVENT_BOOT_COMPLETE, &is_valid);
    assert_int_equal(ret, SMART_QSO_OK);
    assert_true(is_valid);

    /* Invalid transition */
    ret = sm_is_transition_valid(&g_ctx, EVENT_AI_READY, &is_valid);
    assert_int_equal(ret, SMART_QSO_OK);
    assert_false(is_valid);
}

/*===========================================================================*/
/* Force State Tests                                                          */
/*===========================================================================*/

/**
 * @brief Test force state change (recovery)
 */
static void test_sm_force_state(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_ACTIVE);

    SmartQsoResult_t ret = sm_force_state(&g_ctx, STATE_SAFE);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(g_ctx.current_state, STATE_SAFE);
    assert_int_equal(g_ctx.previous_state, STATE_ACTIVE);
}

/**
 * @brief Test force state with invalid state
 */
static void test_sm_force_state_invalid(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_ACTIVE);

    SmartQsoResult_t ret = sm_force_state(&g_ctx, (SmState_t)99);

    assert_int_equal(ret, SMART_QSO_ERROR_INVALID);
    assert_int_equal(g_ctx.current_state, STATE_ACTIVE);  /* Unchanged */
}

/*===========================================================================*/
/* Statistics Tests                                                           */
/*===========================================================================*/

/**
 * @brief Test get statistics
 */
static void test_sm_get_statistics(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_BOOT);
    sm_process_event(&g_ctx, EVENT_BOOT_COMPLETE, NULL);
    sm_process_event(&g_ctx, EVENT_DETUMBLE_COMPLETE, NULL);

    uint32_t count;
    SmState_t prev;
    SmartQsoResult_t ret = sm_get_statistics(&g_ctx, &count, &prev);

    assert_int_equal(ret, SMART_QSO_OK);
    assert_int_equal(count, 2);
    assert_int_equal(prev, STATE_DETUMBLE);
}

/*===========================================================================*/
/* Error Handling Tests                                                       */
/*===========================================================================*/

/**
 * @brief Test operations on uninitialized context
 */
static void test_sm_uninitialized(void **state)
{
    (void)state;

    SmContext_t uninit_ctx;
    memset(&uninit_ctx, 0, sizeof(uninit_ctx));

    SmState_t current;
    SmartQsoResult_t ret = sm_get_state(&uninit_ctx, &current);

    assert_int_equal(ret, SMART_QSO_ERROR_INVALID);
}

/**
 * @brief Test NULL parameter handling
 */
static void test_sm_null_params(void **state)
{
    (void)state;

    sm_init(&g_ctx, STATE_BOOT);

    /* Process event with NULL context */
    SmResult_t result;
    SmartQsoResult_t ret = sm_process_event(NULL, EVENT_BOOT_COMPLETE, &result);
    assert_int_equal(ret, SMART_QSO_ERROR_NULL_PTR);

    /* Get state with NULL state output */
    ret = sm_get_state(&g_ctx, NULL);
    assert_int_equal(ret, SMART_QSO_ERROR_NULL_PTR);

    /* Get duration with NULL output */
    ret = sm_get_state_duration(&g_ctx, NULL);
    assert_int_equal(ret, SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief An event on an uninitialized context is refused and the result
 *        left alone
 */
static void test_sm_process_event_uninitialized(void **state)
{
    (void)state;
    SmResult_t result = SM_RESULT_GUARD_FAILED;

    assert_int_equal(sm_process_event(&g_ctx, EVENT_BOOT_COMPLETE, &result), SMART_QSO_ERROR_INVALID);
    assert_int_equal(result, SM_RESULT_GUARD_FAILED);
    assert_false(g_ctx.is_initialized);
}

/**
 * @brief A context whose state is not a state (corrupted memory) takes no
 *        transition and is left as it is
 */
static void test_sm_corrupt_state_takes_no_transition(void **state)
{
    (void)state;
    SmResult_t result = SM_RESULT_OK;
    bool is_valid = true;

    sm_init(&g_ctx, STATE_BOOT);
    g_ctx.current_state = (SmState_t)99;

    assert_int_equal(sm_process_event(&g_ctx, EVENT_BOOT_COMPLETE, &result), SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_NO_TRANSITION);
    assert_int_equal(g_ctx.current_state, 99);
    assert_int_equal(g_ctx.transition_count, 0);

    assert_int_equal(sm_is_transition_valid(&g_ctx, EVENT_BOOT_COMPLETE, &is_valid), SMART_QSO_OK);
    assert_false(is_valid);
}

/**
 * @brief The transition table can be had without its length
 */
static void test_sm_transitions_without_count(void **state)
{
    (void)state;
    size_t count = 0U;

    const SmTransition_t *table = sm_transitions(&count);
    assert_true(count > 0U);
    assert_ptr_equal(sm_transitions(NULL), table);
}

/**
 * @brief Time in the current state; NULL arguments and an uninitialized
 *        context are refused
 */
static void test_sm_get_state_duration(void **state)
{
    (void)state;
    uint64_t duration = 7U;
    SmContext_t uninit_ctx;

    memset(&uninit_ctx, 0, sizeof(uninit_ctx));
    assert_int_equal(sm_get_state_duration(NULL, &duration), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sm_get_state_duration(&uninit_ctx, &duration), SMART_QSO_ERROR_INVALID);
    assert_int_equal(duration, 7U);

    sm_init(&g_ctx, STATE_IDLE);
    smart_qso_sim_advance_ms(250U);
    assert_int_equal(sm_get_state_duration(&g_ctx, &duration), SMART_QSO_OK);
    assert_in_range(duration, 250U, 250U + 1000U);   /* the host clock runs too */
}

/**
 * @brief Transition check: NULL arguments and an uninitialized context are
 *        refused; an unknown event has no transition
 */
static void test_sm_is_transition_valid_args(void **state)
{
    (void)state;
    bool is_valid = true;
    SmContext_t uninit_ctx;

    memset(&uninit_ctx, 0, sizeof(uninit_ctx));
    assert_int_equal(sm_is_transition_valid(NULL, EVENT_BOOT_COMPLETE, &is_valid), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sm_is_transition_valid(&uninit_ctx, EVENT_BOOT_COMPLETE, &is_valid),
                     SMART_QSO_ERROR_INVALID);

    sm_init(&g_ctx, STATE_BOOT);
    assert_int_equal(sm_is_transition_valid(&g_ctx, EVENT_BOOT_COMPLETE, NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_true(is_valid);
    assert_int_equal(sm_is_transition_valid(&g_ctx, (SmEvent_t)99, &is_valid), SMART_QSO_OK);
    assert_false(is_valid);
}

/**
 * @brief Forcing a state needs a context
 */
static void test_sm_force_state_null(void **state)
{
    (void)state;
    assert_int_equal(sm_force_state(NULL, STATE_SAFE), SMART_QSO_ERROR_NULL_PTR);
}

/**
 * @brief Statistics: NULL context and an uninitialized one are refused;
 *        either output may be omitted
 */
static void test_sm_get_statistics_args(void **state)
{
    (void)state;
    uint32_t count = 7U;
    SmState_t prev = STATE_EMERGENCY;
    SmContext_t uninit_ctx;

    memset(&uninit_ctx, 0, sizeof(uninit_ctx));
    assert_int_equal(sm_get_statistics(NULL, &count, &prev), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sm_get_statistics(&uninit_ctx, &count, &prev), SMART_QSO_ERROR_INVALID);
    assert_int_equal(count, 7U);

    sm_init(&g_ctx, STATE_BOOT);
    sm_process_event(&g_ctx, EVENT_BOOT_COMPLETE, NULL);
    assert_int_equal(sm_get_statistics(&g_ctx, &count, NULL), SMART_QSO_OK);
    assert_int_equal(count, 1U);
    assert_int_equal(prev, STATE_EMERGENCY);
    assert_int_equal(sm_get_statistics(&g_ctx, NULL, &prev), SMART_QSO_OK);
    assert_int_equal(prev, STATE_BOOT);
}

/*===========================================================================*/
/* Full Mission Sequence Test                                                 */
/*===========================================================================*/

/**
 * @brief Test complete mission state sequence
 */
static void test_sm_mission_sequence(void **state)
{
    (void)state;

    /* Start in BOOT */
    sm_init(&g_ctx, STATE_BOOT);
    assert_int_equal(g_ctx.current_state, STATE_BOOT);

    /* Boot complete -> DETUMBLE */
    sm_process_event(&g_ctx, EVENT_BOOT_COMPLETE, NULL);
    assert_int_equal(g_ctx.current_state, STATE_DETUMBLE);

    /* Detumble complete -> IDLE */
    sm_process_event(&g_ctx, EVENT_DETUMBLE_COMPLETE, NULL);
    assert_int_equal(g_ctx.current_state, STATE_IDLE);

    /* AI ready -> ACTIVE */
    sm_process_event(&g_ctx, EVENT_AI_READY, NULL);
    assert_int_equal(g_ctx.current_state, STATE_ACTIVE);

    /* Power low -> IDLE (graceful degradation) */
    sm_process_event(&g_ctx, EVENT_POWER_LOW, NULL);
    assert_int_equal(g_ctx.current_state, STATE_IDLE);

    /* Power OK -> can become ACTIVE again */
    sm_process_event(&g_ctx, EVENT_GROUND_CMD_ACTIVE, NULL);
    assert_int_equal(g_ctx.current_state, STATE_ACTIVE);

    /* Thermal fault -> SAFE */
    sm_process_event(&g_ctx, EVENT_THERMAL_FAULT, NULL);
    assert_int_equal(g_ctx.current_state, STATE_SAFE);

    /* Recovery -> IDLE */
    sm_process_event(&g_ctx, EVENT_RECOVERY_OK, NULL);
    assert_int_equal(g_ctx.current_state, STATE_IDLE);

    /* Verify transition count: one per event above (7) */
    assert_int_equal(g_ctx.transition_count, 7);
}


/*===========================================================================*/
/* Mode manager (SW-14)                                                       */
/*===========================================================================*/

/** Each (state, event) pair has at most one row, so no row is shadowed */
static void test_sm_no_shadowed_rows(void **state)
{
    (void)state;
    size_t count = 0U;
    const SmTransition_t *table = sm_transitions(&count);
    assert_non_null(table);
    assert_true(count > 0U);
    for (size_t i = 0U; i < count; i++) {
        for (size_t j = i + 1U; j < count; j++) {
            assert_false((table[i].from_state == table[j].from_state) &&
                         (table[i].event == table[j].event));
        }
    }
}

/** Every state can be reached from BOOT */
static void test_sm_every_state_reachable(void **state)
{
    (void)state;
    size_t count = 0U;
    const SmTransition_t *table = sm_transitions(&count);
    bool reached[STATE_COUNT] = {false};
    reached[STATE_BOOT] = true;
    for (int pass = 0; pass < STATE_COUNT; pass++) {
        for (size_t i = 0U; i < count; i++) {
            if (reached[table[i].from_state]) {
                reached[table[i].to_state] = true;
            }
        }
    }
    for (int st = 0; st < STATE_COUNT; st++) {
        assert_true(reached[st]);
    }
}

/** Critical power: IDLE/ACTIVE go to SAFE; SAFE goes to EMERGENCY; back on recovery */
static void test_sm_critical_power(void **state)
{
    (void)state;
    SmResult_t result;

    sm_init(&g_ctx, STATE_IDLE);
    sm_process_event(&g_ctx, EVENT_POWER_CRITICAL, &result);
    assert_int_equal(g_ctx.current_state, STATE_SAFE);

    sm_process_event(&g_ctx, EVENT_POWER_CRITICAL, &result);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_EMERGENCY);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
    assert_false(eps_is_beacon_enabled());     /* deeper load shed than SAFE */

    sm_process_event(&g_ctx, EVENT_POWER_OK, &result);
    assert_int_equal(g_ctx.current_state, STATE_SAFE);
    assert_true(eps_is_beacon_enabled());      /* SAFE restores the beacon */
}

/** ACTIVE needs the battery above the payload threshold and the payload guard */
static void test_sm_active_guard(void **state)
{
    (void)state;
    SmResult_t result;

    sm_init(&g_ctx, STATE_IDLE);
    eps_update_soc(EPS_SOC_PAYLOAD_THRESHOLD - 0.01);
    sm_process_event(&g_ctx, EVENT_GROUND_CMD_ACTIVE, &result);
    assert_int_equal(result, SM_RESULT_GUARD_FAILED);
    assert_int_equal(g_ctx.current_state, STATE_IDLE);

    eps_update_soc(EPS_SOC_PAYLOAD_THRESHOLD);
    PayloadSample_t hot;
    memset(&hot, 0, sizeof(hot));
    hot.power_valid = true;
    hot.input_w = PG_LATCHUP_INPUT_W;
    (void)payload_guard_step(&hot, true, smart_qso_now_ms());   /* trip: cool-down */
    sm_process_event(&g_ctx, EVENT_AI_READY, &result);
    assert_int_equal(result, SM_RESULT_GUARD_FAILED);

    payload_guard_init();
    sm_process_event(&g_ctx, EVENT_AI_READY, &result);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(g_ctx.current_state, STATE_ACTIVE);
}

/** Entering a state sets the EPS power mode: one source of truth */
static void test_sm_entry_sets_power_mode(void **state)
{
    (void)state;
    sm_init(&g_ctx, STATE_IDLE);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);
    sm_process_event(&g_ctx, EVENT_AI_READY, NULL);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_ACTIVE);
    sm_process_event(&g_ctx, EVENT_GROUND_CMD_SAFE, NULL);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_SAFE);
    sm_process_event(&g_ctx, EVENT_GROUND_CMD_IDLE, NULL);
    assert_int_equal(eps_get_power_mode(), POWER_MODE_IDLE);
}


/** Power conditions become events: hysteresis 0.40/0.55 for ACTIVE, 0.25/0.40
 *  for SAFE, and EMERGENCY only if SOC keeps falling while SAFE */
static void test_sm_power_event_policy(void **state)
{
    (void)state;
    const struct {
        SmState_t st; double soc; bool sun; SmEvent_t expect;
    } cases[] = {
        {STATE_ACTIVE, EPS_SOC_IDLE_THRESHOLD - 0.01, true, EVENT_POWER_LOW},
        {STATE_ACTIVE, EPS_SOC_IDLE_THRESHOLD + 0.01, false, EVENT_NONE},
        {STATE_ACTIVE, EPS_SOC_SAFE_THRESHOLD - 0.01, true, EVENT_POWER_CRITICAL},
        {STATE_IDLE, EPS_SOC_SAFE_THRESHOLD - 0.01, true, EVENT_POWER_CRITICAL},
        {STATE_IDLE, 0.30, true, EVENT_NONE},
        {STATE_IDLE, EPS_SOC_PAYLOAD_THRESHOLD, true, EVENT_AI_READY},
        {STATE_IDLE, EPS_SOC_PAYLOAD_THRESHOLD - 0.01, true, EVENT_NONE},
        {STATE_IDLE, 0.90, false, EVENT_NONE},                      /* eclipse */
        {STATE_SAFE, EPS_SOC_EMERGENCY_THRESHOLD - 0.01, true, EVENT_POWER_CRITICAL},
        {STATE_SAFE, 0.20, true, EVENT_NONE},
        {STATE_SAFE, EPS_SOC_IDLE_THRESHOLD, true, EVENT_NONE},
        {STATE_SAFE, EPS_SOC_IDLE_THRESHOLD + 0.01, true, EVENT_POWER_OK},
        {STATE_EMERGENCY, EPS_SOC_SAFE_THRESHOLD - 0.01, true, EVENT_NONE},
        {STATE_EMERGENCY, EPS_SOC_SAFE_THRESHOLD, true, EVENT_POWER_OK},
        {STATE_DETUMBLE, EPS_SOC_SAFE_THRESHOLD - 0.01, true, EVENT_POWER_CRITICAL},
        {STATE_DETUMBLE, 0.90, true, EVENT_NONE},
        {STATE_BOOT, 0.10, true, EVENT_POWER_CRITICAL},
    };
    for (size_t i = 0U; i < (sizeof(cases) / sizeof(cases[0])); i++) {
        assert_int_equal(sm_power_event(cases[i].st, cases[i].soc, cases[i].sun), cases[i].expect);
    }
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* Initialization tests */
        cmocka_unit_test_setup(test_sm_init_success, setup),
        cmocka_unit_test_setup(test_sm_init_null_context, setup),
        cmocka_unit_test_setup(test_sm_init_invalid_state, setup),

        /* Transition tests */
        cmocka_unit_test_setup(test_sm_boot_to_detumble, setup),
        cmocka_unit_test_setup(test_sm_detumble_to_idle, setup),
        cmocka_unit_test_setup(test_sm_idle_to_active, setup),
        cmocka_unit_test_setup(test_sm_idle_to_safe_power_low, setup),
        cmocka_unit_test_setup(test_sm_active_to_idle_ai_fault, setup),
        cmocka_unit_test_setup(test_sm_ground_cmd_safe, setup),
        cmocka_unit_test_setup(test_sm_no_transition, setup),
        cmocka_unit_test_setup(test_sm_invalid_event, setup),

        /* Query tests */
        cmocka_unit_test_setup(test_sm_get_state, setup),
        cmocka_unit_test_setup(test_sm_get_state_name, setup),
        cmocka_unit_test_setup(test_sm_get_event_name, setup),
        cmocka_unit_test_setup(test_sm_is_transition_valid, setup),

        /* Force state tests */
        cmocka_unit_test_setup(test_sm_force_state, setup),
        cmocka_unit_test_setup(test_sm_force_state_invalid, setup),

        /* Statistics tests */
        cmocka_unit_test_setup(test_sm_get_statistics, setup),

        /* Error handling tests */
        cmocka_unit_test_setup(test_sm_uninitialized, setup),
        cmocka_unit_test_setup(test_sm_null_params, setup),
        cmocka_unit_test_setup(test_sm_process_event_uninitialized, setup),
        cmocka_unit_test_setup(test_sm_corrupt_state_takes_no_transition, setup),
        cmocka_unit_test_setup(test_sm_transitions_without_count, setup),
        cmocka_unit_test_setup(test_sm_get_state_duration, setup),
        cmocka_unit_test_setup(test_sm_is_transition_valid_args, setup),
        cmocka_unit_test_setup(test_sm_force_state_null, setup),
        cmocka_unit_test_setup(test_sm_get_statistics_args, setup),

        /* Integration test */
        cmocka_unit_test_setup(test_sm_mission_sequence, setup),
        cmocka_unit_test_setup(test_sm_no_shadowed_rows, setup),
        cmocka_unit_test_setup(test_sm_every_state_reachable, setup),
        cmocka_unit_test_setup(test_sm_critical_power, setup),
        cmocka_unit_test_setup(test_sm_active_guard, setup),
        cmocka_unit_test_setup(test_sm_entry_sets_power_mode, setup),
        cmocka_unit_test_setup(test_sm_power_event_policy, setup),
    };

    return cmocka_run_group_tests_name("State Machine Tests", tests, NULL, NULL);
}
