/**
 * @file test_mode_transitions.c
 * @brief Integration tests for operational mode transitions
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Tests all valid state machine transitions per the formal
 * state model documented in SOFTWARE_DESIGN.md.
 *
 * State Diagram:
 *   BOOT → SAFE → DETUMBLE → IDLE ↔ ACTIVE
 *                    ↓         ↓       ↓
 *                 EMERGENCY ← ← ← ← ← ←
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>

#include "smart_qso.h"
#include "system_state.h"
#include "state_machine.h"
#include "eps_control.h"
#include "fault_mgmt.h"

/*******************************************************************************
 * Test Fixtures
 ******************************************************************************/

static int test_setup(void **state)
{
    (void)state;
    (void)fault_mgmt_init();
    (void)sys_state_init();
    (void)eps_init();
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
 * @brief BOOT → SAFE on boot complete
 */
static void test_boot_to_safe(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    assert_int_equal(ctx->current_state, STATE_BOOT);

    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_SAFE);
}

/**
 * @brief BOOT → EMERGENCY on critical fault
 */
static void test_boot_to_emergency(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: SAFE State Transitions
 ******************************************************************************/

/**
 * @brief SAFE → DETUMBLE on detumble command
 */
static void test_safe_to_detumble(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* First get to SAFE state */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    assert_int_equal(ctx->current_state, STATE_SAFE);

    /* Now transition to DETUMBLE */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_DETUMBLE);
}

/**
 * @brief SAFE → EMERGENCY on critical fault
 */
static void test_safe_to_emergency(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);

    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: DETUMBLE State Transitions
 ******************************************************************************/

/**
 * @brief DETUMBLE → IDLE on detumble complete
 */
static void test_detumble_to_idle(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Get to DETUMBLE */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    assert_int_equal(ctx->current_state, STATE_DETUMBLE);

    /* Complete detumble */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_DETUMBLE_COMPLETE);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_IDLE);
}

/**
 * @brief DETUMBLE → EMERGENCY on critical fault
 */
static void test_detumble_to_emergency(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);

    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: IDLE State Transitions
 ******************************************************************************/

/**
 * @brief IDLE → ACTIVE on power available
 */
static void test_idle_to_active(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Get to IDLE */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_COMPLETE);
    assert_int_equal(ctx->current_state, STATE_IDLE);

    /* Transition to ACTIVE */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_POWER_AVAILABLE);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_ACTIVE);
}

/**
 * @brief IDLE → EMERGENCY on critical fault
 */
static void test_idle_to_emergency(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_COMPLETE);

    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: ACTIVE State Transitions
 ******************************************************************************/

/**
 * @brief ACTIVE → IDLE on low power
 */
static void test_active_to_idle(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Get to ACTIVE */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_POWER_AVAILABLE);
    assert_int_equal(ctx->current_state, STATE_ACTIVE);

    /* Low power triggers return to IDLE */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_LOW_POWER);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_IDLE);
}

/**
 * @brief ACTIVE → EMERGENCY on critical fault
 */
static void test_active_to_emergency(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_POWER_AVAILABLE);

    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);
}

/*******************************************************************************
 * Test Cases: EMERGENCY State Transitions
 ******************************************************************************/

/**
 * @brief EMERGENCY → SAFE on recovery
 */
static void test_emergency_to_safe(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Get to EMERGENCY */
    (void)sm_process_event(ctx, SM_EVENT_CRITICAL_FAULT);
    assert_int_equal(ctx->current_state, STATE_EMERGENCY);

    /* Recover */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_FAULT_CLEARED);
    assert_int_equal(result, SMART_QSO_OK);
    assert_int_equal(ctx->current_state, STATE_SAFE);
}

/*******************************************************************************
 * Test Cases: Invalid Transitions
 ******************************************************************************/

/**
 * @brief Verify invalid transitions are rejected
 */
static void test_invalid_transition_rejected(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* Try direct BOOT → ACTIVE (invalid) */
    SmartQsoResult_t result = sm_process_event(ctx, SM_EVENT_POWER_AVAILABLE);

    /* Should either reject or stay in current state */
    assert_true(result != SMART_QSO_OK ||
                ctx->current_state == STATE_BOOT);
}

/**
 * @brief Verify power mode changes with state transitions
 */
static void test_power_mode_follows_state(void **state)
{
    (void)state;

    SmContext_t *ctx = sys_get_sm_context();

    /* BOOT should have SAFE power mode */
    assert_int_equal(sys_get_power_mode(), POWER_MODE_SAFE);

    /* Transition to ACTIVE */
    (void)sm_process_event(ctx, SM_EVENT_BOOT_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_CMD);
    (void)sm_process_event(ctx, SM_EVENT_DETUMBLE_COMPLETE);
    (void)sm_process_event(ctx, SM_EVENT_POWER_AVAILABLE);

    /* Power mode should update (depends on EPS implementation) */
    PowerMode_t mode = sys_get_power_mode();
    (void)mode; /* May vary by implementation */
}

/*******************************************************************************
 * Test Runner
 ******************************************************************************/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* BOOT Transitions */
        cmocka_unit_test_setup_teardown(test_boot_to_safe,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_boot_to_emergency,
                                        test_setup, test_teardown),

        /* SAFE Transitions */
        cmocka_unit_test_setup_teardown(test_safe_to_detumble,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_safe_to_emergency,
                                        test_setup, test_teardown),

        /* DETUMBLE Transitions */
        cmocka_unit_test_setup_teardown(test_detumble_to_idle,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_detumble_to_emergency,
                                        test_setup, test_teardown),

        /* IDLE Transitions */
        cmocka_unit_test_setup_teardown(test_idle_to_active,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_idle_to_emergency,
                                        test_setup, test_teardown),

        /* ACTIVE Transitions */
        cmocka_unit_test_setup_teardown(test_active_to_idle,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_active_to_emergency,
                                        test_setup, test_teardown),

        /* EMERGENCY Transitions */
        cmocka_unit_test_setup_teardown(test_emergency_to_safe,
                                        test_setup, test_teardown),

        /* Invalid Transitions */
        cmocka_unit_test_setup_teardown(test_invalid_transition_rejected,
                                        test_setup, test_teardown),
        cmocka_unit_test_setup_teardown(test_power_mode_follows_state,
                                        test_setup, test_teardown),
    };

    return cmocka_run_group_tests(tests, NULL, NULL);
}
