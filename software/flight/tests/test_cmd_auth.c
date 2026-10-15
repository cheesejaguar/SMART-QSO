/**
 * @file test_cmd_auth.c
 * @brief Unit tests for telecommand authentication and replay protection
 *
 * The expected payloads were generated independently with Python hmac /
 * hashlib and are shared with software/ground/tools/telecommand_builder.py,
 * so these tests also prove flight/ground interoperability.
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "cmd_auth.h"
#include "hal/hal_flash.h"
#include "hal/hal_sim.h"

/*===========================================================================*/
/* Test Data                                                                  */
/*===========================================================================*/

/** TEST-ONLY key 0x00..0x1F. Flight keys are provisioned at integration. */
static uint8_t s_test_key[CMD_AUTH_KEY_SIZE];

#define CMD_TX_DISABLE  0x32U
#define CMD_SET_BEACON  0x30U

/** TX_DISABLE, counter 1, no args (Python reference) */
static const uint8_t TX_DISABLE_CTR1[] = {
    0x00, 0x00, 0x00, 0x01, 0x9A, 0x83, 0xF5, 0xC7, 0xB2, 0x1F,
    0x75, 0x11, 0xE8, 0x2F, 0x5E, 0x94, 0xB9, 0xDE, 0x58, 0x51
};

/** SET_BEACON, counter 2, args 00 3C (60 s) (Python reference) */
static const uint8_t SET_BEACON_CTR2[] = {
    0x00, 0x00, 0x00, 0x02, 0x00, 0x3C, 0xB4, 0x96, 0x34, 0x05, 0x5C,
    0xFF, 0xC1, 0x60, 0x65, 0x4E, 0x1B, 0x7E, 0x96, 0xDA, 0x3C, 0x13
};

/*===========================================================================*/
/* Fixtures                                                                   */
/*===========================================================================*/

static int setup_blank(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(s_test_key); i++) {
        s_test_key[i] = (uint8_t)i;
    }
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    return 0;
}

static int setup_provisioned(void **state)
{
    (void)setup_blank(state);
    assert_int_equal(cmd_auth_provision_key(s_test_key, 1U), SMART_QSO_OK);
    return 0;
}

static int teardown(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    return 0;
}

/*===========================================================================*/
/* Tests                                                                      */
/*===========================================================================*/

static void test_unprovisioned_rejects_everything(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_false(cmd_auth_is_provisioned());
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_NOT_PROVISIONED);
}

static void test_ground_vector_verifies(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_true(cmd_auth_is_provisioned());
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_OK);
    assert_int_equal(v.counter, 1U);
    assert_int_equal(v.args_len, 0U);
}

static void test_ground_vector_with_args_verifies(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_verify(CMD_SET_BEACON, SET_BEACON_CTR2, sizeof(SET_BEACON_CTR2), &v),
                     CMD_AUTH_OK);
    assert_int_equal(v.counter, 2U);
    assert_int_equal(v.args_len, 2U);
    assert_int_equal(v.args[0], 0x00U);
    assert_int_equal(v.args[1], 0x3CU);
}

static void test_tampered_args_rejected(void **state)
{
    (void)state;
    uint8_t p[sizeof(SET_BEACON_CTR2)];
    CmdAuthVerified_t v;

    memcpy(p, SET_BEACON_CTR2, sizeof(p));
    p[5] = 0x0A;  /* 60 s -> 10 s */
    assert_int_equal(cmd_auth_verify(CMD_SET_BEACON, p, sizeof(p), &v), CMD_AUTH_ERR_TAG);
}

/** A captured TX_DISABLE must not be relabelled as another command */
static void test_relabelled_command_rejected(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_verify(0x31U, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_TAG);
}

static void test_tampered_counter_rejected(void **state)
{
    (void)state;
    uint8_t p[sizeof(TX_DISABLE_CTR1)];
    CmdAuthVerified_t v;

    memcpy(p, TX_DISABLE_CTR1, sizeof(p));
    p[3] = 0x09;
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, p, sizeof(p), &v), CMD_AUTH_ERR_TAG);
}

static void test_wrong_key_rejected(void **state)
{
    (void)state;
    uint8_t other_key[CMD_AUTH_KEY_SIZE];
    CmdAuthVerified_t v;

    memset(other_key, 0x5A, sizeof(other_key));
    assert_int_equal(cmd_auth_provision_key(other_key, 2U), SMART_QSO_OK);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_TAG);
}

static void test_short_payload_rejected(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, CMD_AUTH_OVERHEAD - 1U, &v),
                     CMD_AUTH_ERR_FORMAT);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, NULL, 0U, &v), CMD_AUTH_ERR_FORMAT);
}

static void test_replay_rejected_after_commit(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_OK);
    assert_int_equal(cmd_auth_commit(v.counter), SMART_QSO_OK);

    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_REPLAY);
}

static void test_older_counter_rejected(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_commit(2U), SMART_QSO_OK);
    /* Counter 1 is now in the past */
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_REPLAY);
}

/** Rejected frames must not move the counter */
static void test_failed_verify_does_not_advance_counter(void **state)
{
    (void)state;
    uint8_t p[sizeof(TX_DISABLE_CTR1)];
    CmdAuthVerified_t v;

    memcpy(p, TX_DISABLE_CTR1, sizeof(p));
    p[sizeof(p) - 1U] ^= 0x01U;
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, p, sizeof(p), &v), CMD_AUTH_ERR_TAG);
    assert_int_equal(cmd_auth_last_counter(), 0U);
}

/** The accepted counter survives a reset, so replays stay rejected */
static void test_counter_survives_reset(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_commit(1U), SMART_QSO_OK);

    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);  /* simulated reset */

    assert_true(cmd_auth_is_provisioned());
    assert_int_equal(cmd_auth_last_counter(), 1U);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_REPLAY);
}

static void test_key_survives_reset(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);  /* simulated reset */
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_OK);
}

static void test_exhausted_counter_rejected(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_commit(0xFFFFFFFEU), SMART_QSO_OK);
    /* Even a correctly tagged frame cannot use the last counter value */
    assert_int_equal(cmd_auth_commit(0xFFFFFFFFU), SMART_QSO_ERROR_INVALID);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_EXHAUSTED);
}

static void test_commit_flash_failure_reported_but_blocks_replay(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    hal_sim_flash_fail_writes(true);
    assert_int_equal(cmd_auth_commit(1U), SMART_QSO_ERROR_IO);
    hal_sim_flash_fail_writes(false);

    assert_int_equal(cmd_auth_last_counter(), 1U);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_REPLAY);
}

/** A corrupt key record must fail closed */
static void test_corrupt_record_is_unprovisioned(void **state)
{
    (void)state;
    uint8_t junk[8];

    memset(junk, 0x00, sizeof(junk));
    assert_int_equal(hal_flash_write(FLASH_REGION_AUTH, 8U, junk, sizeof(junk)), SMART_QSO_OK);
    assert_int_equal(hal_flash_write(FLASH_REGION_AUTH, 72U, junk, sizeof(junk)), SMART_QSO_OK);

    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    assert_false(cmd_auth_is_provisioned());
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup_teardown(test_unprovisioned_rejects_everything, setup_blank, teardown),
        cmocka_unit_test_setup_teardown(test_ground_vector_verifies, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_ground_vector_with_args_verifies, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_tampered_args_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_relabelled_command_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_tampered_counter_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_wrong_key_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_short_payload_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_replay_rejected_after_commit, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_older_counter_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_failed_verify_does_not_advance_counter, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_counter_survives_reset, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_key_survives_reset, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_exhausted_counter_rejected, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_commit_flash_failure_reported_but_blocks_replay, setup_provisioned, teardown),
        cmocka_unit_test_setup_teardown(test_corrupt_record_is_unprovisioned, setup_provisioned, teardown),
    };

    return cmocka_run_group_tests_name("Telecommand Authentication Tests", tests, NULL, NULL);
}
