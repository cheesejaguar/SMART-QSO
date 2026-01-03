/**
 * @file test_state_machine.c
 * @brief Unit tests for state machine module
 *
 * Tests state machine transitions and guards per NPR 7150.2.
 *
 * @requirement SRS-030 Formal state machine for mode control
 * @requirement SRS-031 All state transitions shall be logged
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

/* Include the module under test */
#include "state_machine.h"

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

    /* Verify transition count */
    assert_int_equal(g_ctx.transition_count, 8);
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

        /* Integration test */
        cmocka_unit_test_setup(test_sm_mission_sequence, setup),
    };

    return cmocka_run_group_tests_name("State Machine Tests", tests, NULL, NULL);
}
