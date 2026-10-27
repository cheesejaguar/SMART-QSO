/**
 * @file test_ops_state.c
 * @brief The commissioning hold and the ground's transponder enable
 *
 * Expectations from ops_state.h, the CDR flight-software findings LV-01 and
 * LV-14 and docs/software/SOFTWARE_REQUIREMENTS.md §3.9:
 *
 * - ARM_FLIGHT sets the hold; only the release clears it; it survives resets;
 * - while held, every way into ACTIVE is refused (mode manager guard);
 * - a blank record (bench unit) holds nothing; a corrupt one holds;
 * - XPDR_DISABLE persists and never lifts the hold.
 *
 * @requirement SRS-F082 The software shall hold the mode at IDLE, the payload and the transponder off from ARM_FLIGHT until an authenticated commissioning release
 * @requirement SRS-F083 The software shall let the ground stop and allow the transponder by telecommand, persisted across resets
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "ops_state.h"
#include "state_machine.h"
#include "eps_control.h"
#include "payload_guard.h"
#include "fault_mgmt.h"
#include "hal/hal.h"
#include "hal/hal_flash.h"

static int setup(void **state)
{
    (void)state;
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_OPS), SMART_QSO_OK);
    (void)fault_mgmt_init();
    payload_guard_init();
    (void)eps_init();
    eps_update_soc(0.9);
    return 0;
}

/** A unit that was never armed holds nothing */
static void test_blank_record_holds_nothing(void **state)
{
    (void)state;
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_false(ops_commissioning_hold());
    assert_true(ops_xpdr_allowed());
}

/** The hold survives a reset and only the release clears it */
static void test_hold_persists_until_release(void **state)
{
    (void)state;
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_int_equal(ops_set_commissioning_hold(true), SMART_QSO_OK);
    assert_int_equal(ops_state_init(), SMART_QSO_OK); /* a reset */
    assert_true(ops_commissioning_hold());
    assert_false(ops_xpdr_allowed());
    assert_int_equal(ops_set_xpdr_enabled(true), SMART_QSO_OK);
    assert_false(ops_xpdr_allowed()); /* enabling does not lift the hold */
    assert_int_equal(ops_set_commissioning_hold(false), SMART_QSO_OK);
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_false(ops_commissioning_hold());
    assert_true(ops_xpdr_allowed());
}

/** While held, the mode manager refuses ACTIVE, onboard and from the ground */
static void test_hold_caps_the_mode_at_idle(void **state)
{
    (void)state;
    SmContext_t ctx;
    SmResult_t result = SM_RESULT_OK;
    memset(&ctx, 0, sizeof(ctx));
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_int_equal(ops_set_commissioning_hold(true), SMART_QSO_OK);
    assert_int_equal(sm_init(&ctx, STATE_IDLE), SMART_QSO_OK);
    assert_int_equal(sm_process_event(&ctx, EVENT_AI_READY, &result), SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_GUARD_FAILED);
    assert_int_equal(sm_process_event(&ctx, EVENT_GROUND_CMD_ACTIVE, &result), SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_GUARD_FAILED);
    assert_int_equal(ctx.current_state, STATE_IDLE);
    /* Released: ACTIVE as before */
    assert_int_equal(ops_set_commissioning_hold(false), SMART_QSO_OK);
    assert_int_equal(sm_process_event(&ctx, EVENT_AI_READY, &result), SMART_QSO_OK);
    assert_int_equal(result, SM_RESULT_OK);
    assert_int_equal(ctx.current_state, STATE_ACTIVE);
}

/** A corrupt record holds (a lost record must not release a held unit) */
static void test_corrupt_record_holds(void **state)
{
    (void)state;
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_int_equal(ops_set_commissioning_hold(false), SMART_QSO_OK);
    /* Flip a payload byte in both slots (64-byte slots, payload after the
       8-byte magic and sequence) */
    for (uint32_t slot = 0U; slot < 2U; slot++) {
        uint8_t b = 0U;
        assert_int_equal(hal_flash_read(FLASH_REGION_OPS, (slot * 64U) + 8U, &b, 1U), SMART_QSO_OK);
        b ^= 0xFFU;
        assert_int_equal(hal_flash_write(FLASH_REGION_OPS, (slot * 64U) + 8U, &b, 1U), SMART_QSO_OK);
    }
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_true(ops_commissioning_hold());
}

/** XPDR_DISABLE persists across a reset */
static void test_transponder_disable_persists(void **state)
{
    (void)state;
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_int_equal(ops_set_xpdr_enabled(false), SMART_QSO_OK);
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    assert_false(ops_xpdr_allowed());
    assert_false(ops_commissioning_hold());
    assert_int_equal(ops_set_xpdr_enabled(true), SMART_QSO_OK);
    assert_true(ops_xpdr_allowed());
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_blank_record_holds_nothing, setup),
        cmocka_unit_test_setup(test_hold_persists_until_release, setup),
        cmocka_unit_test_setup(test_hold_caps_the_mode_at_idle, setup),
        cmocka_unit_test_setup(test_corrupt_record_holds, setup),
        cmocka_unit_test_setup(test_transponder_disable_persists, setup),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
