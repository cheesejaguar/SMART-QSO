/**
 * @file test_cmd_auth.c
 * @brief Unit tests for telecommand authentication, replay protection and the key table
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
#include "auth_test_helpers.h"

/*===========================================================================*/
/* Test Data                                                                  */
/*===========================================================================*/

/** TEST-ONLY key 0x00..0x1F. Flight keys are provisioned at integration. */
static uint8_t s_test_key[CMD_AUTH_KEY_SIZE];

/** TEST-ONLY spare operational key 0x40..0x5F and recovery key 0x80..0x9F */
static uint8_t s_spare_key[CMD_AUTH_KEY_SIZE];
static uint8_t s_recovery_key[CMD_AUTH_KEY_SIZE];

#define CMD_TX_DISABLE  0x32U
#define CMD_SET_BEACON  0x30U
#define CMD_KEY_ROTATE  0x06U
#define CMD_TX_ENABLE   0x31U

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
    test_key_fill(s_test_key, 0x00U);
    test_key_fill(s_spare_key, 0x40U);
    test_key_fill(s_recovery_key, 0x80U);
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

/** Operational slot 0 active, slot 1 spare, recovery key present */
static int setup_key_table(void **state)
{
    (void)setup_provisioned(state);
    assert_int_equal(cmd_auth_provision_slot(1U, s_spare_key, 11U), SMART_QSO_OK);
    assert_int_equal(cmd_auth_provision_slot(CMD_AUTH_RECOVERY_SLOT, s_recovery_key, 99U),
                     SMART_QSO_OK);
    return 0;
}

static int teardown(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    hal_sim_flash_fail_writes_after(-1);
    return 0;
}

/** Commit an operational counter directly */
static SmartQsoResult_t commit_op(uint32_t counter)
{
    CmdAuthVerified_t v = {counter, NULL, 0U, CMD_AUTH_KEY_OPERATIONAL};
    return cmd_auth_commit(&v);
}

/*===========================================================================*/
/* Tests                                                                      */
/*===========================================================================*/

static void test_unprovisioned_rejects_everything(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_false(cmd_auth_is_provisioned());
    assert_false(cmd_auth_has_key_material());
    assert_int_equal(cmd_auth_verify(CMD_SET_BEACON, SET_BEACON_CTR2, sizeof(SET_BEACON_CTR2), &v),
                     CMD_AUTH_ERR_NOT_PROVISIONED);
    assert_int_equal(cmd_auth_verify(CMD_KEY_ROTATE, SET_BEACON_CTR2, sizeof(SET_BEACON_CTR2), &v),
                     CMD_AUTH_ERR_NOT_PROVISIONED);
}

/** With no usable key, TX_DISABLE (and only TX_DISABLE) is accepted without
 *  authentication: TX-OFF can only reduce emissions (97.207(b)) */
static void test_tx_disable_accepted_without_any_key(void **state)
{
    (void)state;
    uint8_t forged[sizeof(TX_DISABLE_CTR1)];
    CmdAuthVerified_t v;

    memset(forged, 0xEE, sizeof(forged));
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, forged, sizeof(forged), &v), CMD_AUTH_OK);
    assert_int_equal(v.key, CMD_AUTH_KEY_NONE);
    assert_int_equal(cmd_auth_commit(&v), SMART_QSO_OK);

    /* Still well-formed: a truncated frame is rejected */
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, forged, CMD_AUTH_OVERHEAD - 1U, &v),
                     CMD_AUTH_ERR_FORMAT);
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
    assert_int_equal(v.key, CMD_AUTH_KEY_OPERATIONAL);
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
    assert_int_equal(cmd_auth_commit(&v), SMART_QSO_OK);

    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_REPLAY);
}

static void test_older_counter_rejected(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(commit_op(2U), SMART_QSO_OK);
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

    assert_int_equal(commit_op(1U), SMART_QSO_OK);

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

    assert_int_equal(commit_op(0xFFFFFFFEU), SMART_QSO_OK);
    /* Even a correctly tagged frame cannot use the last counter value */
    assert_int_equal(commit_op(0xFFFFFFFFU), SMART_QSO_ERROR_INVALID);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_EXHAUSTED);
}

static void test_commit_flash_failure_reported_but_blocks_replay(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    hal_sim_flash_fail_writes(true);
    assert_int_equal(commit_op(1U), SMART_QSO_ERROR_IO);
    hal_sim_flash_fail_writes(false);

    assert_int_equal(cmd_auth_last_counter(), 1U);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_REPLAY);
}

/** Invert the first payload bytes of both slots of one record */
static void corrupt_record(uint32_t base)
{
    for (uint32_t slot = 0U; slot < 2U; slot++) {
        uint8_t bytes[4];
        uint32_t offset = base + (slot * NVM_RECORD_SLOT_SIZE) + 8U;

        assert_int_equal(hal_flash_read(FLASH_REGION_AUTH, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
        for (size_t i = 0U; i < sizeof(bytes); i++) {
            bytes[i] ^= 0xFFU;
        }
        assert_int_equal(hal_flash_write(FLASH_REGION_AUTH, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
    }
}

/** A corrupt active key record must fail closed */
static void test_corrupt_record_is_unprovisioned(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    corrupt_record(CMD_AUTH_NVM_OP(0U));

    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    assert_false(cmd_auth_is_provisioned());
    assert_int_equal(cmd_auth_verify(CMD_SET_BEACON, SET_BEACON_CTR2, sizeof(SET_BEACON_CTR2), &v),
                     CMD_AUTH_ERR_NOT_PROVISIONED);
}

/*===========================================================================*/
/* Key table: recovery key, rotation, interrupted writes                      */
/*===========================================================================*/

static CmdAuthStatus_t verify_with(const uint8_t *key, uint8_t cmd_id, uint32_t counter,
                                   const uint8_t *args, size_t args_len, CmdAuthVerified_t *v)
{
    uint8_t payload[CMD_AUTH_OVERHEAD + CMD_AUTH_MAX_ARGS];
    size_t len = test_auth_payload(key, cmd_id, counter, args, args_len, payload);
    return cmd_auth_verify(cmd_id, payload, len, v);
}

/** The recovery key authenticates TX_DISABLE and KEY_ROTATE only */
static void test_recovery_key_scope(void **state)
{
    (void)state;
    const uint8_t slot = 1U;
    const uint8_t beacon[2] = {0x00U, 0x3CU};
    CmdAuthVerified_t v;

    assert_int_equal(verify_with(s_recovery_key, CMD_TX_DISABLE, 1U, NULL, 0U, &v), CMD_AUTH_OK);
    assert_int_equal(v.key, CMD_AUTH_KEY_RECOVERY);
    assert_int_equal(verify_with(s_recovery_key, CMD_KEY_ROTATE, 1U, &slot, 1U, &v), CMD_AUTH_OK);
    assert_int_equal(v.key, CMD_AUTH_KEY_RECOVERY);

    assert_int_equal(verify_with(s_recovery_key, CMD_SET_BEACON, 1U, beacon, 2U, &v), CMD_AUTH_ERR_TAG);
    /* Restarting the transmitter needs the operational key */
    assert_int_equal(verify_with(s_recovery_key, CMD_TX_ENABLE, 1U, NULL, 0U, &v), CMD_AUTH_ERR_TAG);
    /* The operational key cannot rotate keys */
    assert_int_equal(verify_with(s_test_key, CMD_KEY_ROTATE, 1U, &slot, 1U, &v), CMD_AUTH_ERR_TAG);
    /* Once any key exists, TX_DISABLE is no longer accepted unauthenticated */
    assert_int_equal(verify_with(s_spare_key, CMD_TX_DISABLE, 1U, NULL, 0U, &v), CMD_AUTH_ERR_TAG);
}

/** Each key has its own counter, persisted across resets */
static void test_recovery_counter_is_separate(void **state)
{
    (void)state;
    CmdAuthVerified_t v;
    CmdAuthInfo_t info;

    assert_int_equal(verify_with(s_recovery_key, CMD_TX_DISABLE, 5U, NULL, 0U, &v), CMD_AUTH_OK);
    assert_int_equal(cmd_auth_commit(&v), SMART_QSO_OK);
    assert_int_equal(cmd_auth_last_counter(), 0U);

    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);  /* simulated reset */
    cmd_auth_get_info(&info);
    assert_int_equal(info.recovery_counter, 5U);
    assert_int_equal(info.recovery_key_id, 99U);
    assert_int_equal(verify_with(s_recovery_key, CMD_TX_DISABLE, 5U, NULL, 0U, &v), CMD_AUTH_ERR_REPLAY);
    /* The operational key is unaffected */
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_OK);
}

/** Rotation activates the spare, revokes the old key and restarts the counter */
static void test_rotate_replaces_key(void **state)
{
    (void)state;
    CmdAuthVerified_t v;
    CmdAuthInfo_t info;

    assert_int_equal(commit_op(7U), SMART_QSO_OK);
    assert_int_equal(cmd_auth_rotate(1U), SMART_QSO_OK);

    cmd_auth_get_info(&info);
    assert_int_equal(info.active_slot, 1U);
    assert_int_equal(info.active_key_id, 11U);
    assert_int_equal(cmd_auth_last_counter(), 0U);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_TAG);
    assert_int_equal(verify_with(s_spare_key, CMD_TX_DISABLE, 1U, NULL, 0U, &v), CMD_AUTH_OK);
    assert_int_equal(v.key, CMD_AUTH_KEY_OPERATIONAL);

    /* Survives a reset; the revoked key can never be reactivated */
    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    cmd_auth_get_info(&info);
    assert_int_equal(info.active_slot, 1U);
    assert_int_equal(verify_with(s_spare_key, CMD_TX_DISABLE, 1U, NULL, 0U, &v), CMD_AUTH_OK);
    assert_int_equal(cmd_auth_rotate(0U), SMART_QSO_ERROR_INVALID);
}

static void test_rotate_rejects_non_spare(void **state)
{
    (void)state;

    assert_int_equal(cmd_auth_rotate(0U), SMART_QSO_ERROR_INVALID);   /* active */
    assert_int_equal(cmd_auth_rotate(2U), SMART_QSO_ERROR_INVALID);   /* blank */
    assert_int_equal(cmd_auth_rotate(CMD_AUTH_OP_SLOTS), SMART_QSO_ERROR_INVALID);
    assert_true(cmd_auth_is_provisioned());
}

/** Power lost after the spare is marked active but before the state record
 *  is written: the old key stays in force and the half-activated key is
 *  revoked at the next boot */
static void test_interrupted_rotation_is_safe(void **state)
{
    (void)state;
    CmdAuthVerified_t v;
    CmdAuthInfo_t info;

    hal_sim_flash_fail_writes_after(1);
    assert_int_equal(cmd_auth_rotate(1U), SMART_QSO_ERROR_IO);
    hal_sim_flash_fail_writes_after(-1);

    /* This session: still the old key */
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_OK);

    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    cmd_auth_get_info(&info);
    assert_int_equal(info.active_slot, 0U);
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_OK);
    assert_int_equal(cmd_auth_rotate(1U), SMART_QSO_ERROR_INVALID);
}

/** A corrupt state record locks operational commands; the recovery key
 *  restores control by rotating to a spare, revoking the old active key */
static void test_recovery_after_corrupt_state(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    corrupt_record(CMD_AUTH_NVM_STATE);
    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    assert_false(cmd_auth_is_provisioned());
    assert_true(cmd_auth_has_key_material());
    assert_int_equal(cmd_auth_verify(CMD_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v),
                     CMD_AUTH_ERR_TAG);
    assert_int_equal(verify_with(s_recovery_key, CMD_TX_DISABLE, 1U, NULL, 0U, &v), CMD_AUTH_OK);

    assert_int_equal(cmd_auth_rotate(1U), SMART_QSO_OK);
    assert_true(cmd_auth_is_provisioned());
    assert_int_equal(verify_with(s_spare_key, CMD_TX_DISABLE, 1U, NULL, 0U, &v), CMD_AUTH_OK);
    assert_int_equal(cmd_auth_rotate(0U), SMART_QSO_ERROR_INVALID);  /* old key revoked */
}

static void test_zeroize_removes_all_keys(void **state)
{
    (void)state;
    CmdAuthVerified_t v;

    assert_int_equal(cmd_auth_zeroize(), SMART_QSO_OK);
    assert_false(cmd_auth_has_key_material());
    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    assert_false(cmd_auth_has_key_material());
    assert_int_equal(cmd_auth_verify(CMD_SET_BEACON, SET_BEACON_CTR2, sizeof(SET_BEACON_CTR2), &v),
                     CMD_AUTH_ERR_NOT_PROVISIONED);
}

static void test_provision_rejects_bad_slot(void **state)
{
    (void)state;
    assert_int_equal(cmd_auth_provision_slot(CMD_AUTH_RECOVERY_SLOT + 1U, s_spare_key, 1U),
                     SMART_QSO_ERROR_INVALID);
    assert_int_equal(cmd_auth_provision_slot(0U, NULL, 1U), SMART_QSO_ERROR_NULL_PTR);
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
        cmocka_unit_test_setup_teardown(test_tx_disable_accepted_without_any_key, setup_blank, teardown),
        cmocka_unit_test_setup_teardown(test_recovery_key_scope, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_recovery_counter_is_separate, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_rotate_replaces_key, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_rotate_rejects_non_spare, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_interrupted_rotation_is_safe, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_recovery_after_corrupt_state, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_zeroize_removes_all_keys, setup_key_table, teardown),
        cmocka_unit_test_setup_teardown(test_provision_rejects_bad_slot, setup_blank, teardown),
    };

    return cmocka_run_group_tests_name("Telecommand Authentication Tests", tests, NULL, NULL);
}
