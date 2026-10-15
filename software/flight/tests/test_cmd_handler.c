/**
 * @file test_cmd_handler.c
 * @brief End-to-end telecommand path: framing -> authentication -> replay
 *        protection -> state gating -> TX-OFF / TX-ON execution
 *
 * Authenticated payloads were generated with Python hmac/hashlib using the
 * TEST-ONLY key 0x00..0x1F (see test_cmd_auth.c and
 * software/ground/tools/telecommand_builder.py).
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "cmd_handler.h"
#include "cmd_auth.h"
#include "rf_control.h"
#include "contact_monitor.h"
#include "system_state.h"
#include "state_machine.h"
#include "fault_mgmt.h"
#include "sha256.h"
#include "auth_test_helpers.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_sim.h"

/*===========================================================================*/
/* Test Data (payload = counter | args | tag)                                 */
/*===========================================================================*/

static const uint8_t TX_DISABLE_CTR1[] = {
    0x00, 0x00, 0x00, 0x01, 0x9A, 0x83, 0xF5, 0xC7, 0xB2, 0x1F,
    0x75, 0x11, 0xE8, 0x2F, 0x5E, 0x94, 0xB9, 0xDE, 0x58, 0x51
};
static const uint8_t TX_ENABLE_CTR2[] = {
    0x00, 0x00, 0x00, 0x02, 0x3B, 0x24, 0xD1, 0xFD, 0xF6, 0x58,
    0xA1, 0x47, 0xFA, 0xA5, 0xF9, 0x7B, 0x4B, 0xCF, 0x77, 0x9C
};
static const uint8_t TX_DISABLE_CTR3[] = {
    0x00, 0x00, 0x00, 0x03, 0x56, 0x6C, 0xA3, 0x4E, 0x13, 0xC9,
    0x6B, 0xA0, 0xDC, 0x25, 0xE0, 0xAA, 0x93, 0xDD, 0xD2, 0x3B
};
static const uint8_t TX_DISABLE_CTR1_EXTRA_ARG[] = {
    0x00, 0x00, 0x00, 0x01, 0x00, 0xFA, 0xFB, 0xE7, 0xA4, 0x0A, 0x21,
    0x1A, 0xD8, 0x53, 0xED, 0x49, 0x7C, 0xE9, 0x42, 0xAC, 0x16
};
static const uint8_t GET_STATUS_CTR1[] = {
    0x00, 0x00, 0x00, 0x01, 0x81, 0xA9, 0x53, 0xBF, 0x2B, 0x4C,
    0x03, 0x61, 0xFB, 0x2C, 0xC1, 0x7D, 0xA4, 0x78, 0x9B, 0x12
};

/** Complete uplink frame for TX_DISABLE counter 1, as built by the ground tool */
static const uint8_t TX_DISABLE_CTR1_FRAME[] = {
    0xAA, 0x55, 0x32, 0x14, 0x00, 0x00, 0x00, 0x01, 0x9A, 0x83, 0xF5, 0xC7, 0xB2,
    0x1F, 0x75, 0x11, 0xE8, 0x2F, 0x5E, 0x94, 0xB9, 0xDE, 0x58, 0x51, 0x09, 0xBF
};

/*===========================================================================*/
/* Helpers                                                                    */
/*===========================================================================*/

/** Wrap a payload in the uplink frame: sync | cmd_id | len | payload | sum16 */
static size_t make_frame(uint8_t cmd_id, const uint8_t *payload, size_t len, uint8_t *out)
{
    uint16_t sum = 0U;

    out[0] = 0xAAU;
    out[1] = 0x55U;
    out[2] = cmd_id;
    out[3] = (uint8_t)len;
    if (len > 0U) {
        memcpy(&out[4], payload, len);
    }
    for (size_t i = 0U; i < (4U + len); i++) {
        sum = (uint16_t)(sum + out[i]);
    }
    out[4U + len] = (uint8_t)(sum >> 8);
    out[5U + len] = (uint8_t)sum;
    return 6U + len;
}

static CmdResponse_t s_response;

static CmdResult_t send(uint8_t cmd_id, const uint8_t *payload, size_t len)
{
    uint8_t frame[300];
    size_t frame_len = make_frame(cmd_id, payload, len, frame);

    memset(&s_response, 0, sizeof(s_response));
    assert_int_equal(cmd_process(frame, frame_len, &s_response), SMART_QSO_OK);
    return s_response.result;
}

static void provision_test_key(void)
{
    uint8_t key[CMD_AUTH_KEY_SIZE];
    for (size_t i = 0U; i < sizeof(key); i++) {
        key[i] = (uint8_t)i;
    }
    assert_int_equal(cmd_auth_provision_key(key, 1U), SMART_QSO_OK);
}

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_CONTACT), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(sm_force_state(sys_get_sm_context(), STATE_IDLE), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    provision_test_key();
    assert_true(rf_control_tx_permitted());
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

/** The C framing matches the ground tool's frame byte for byte */
static void test_frame_matches_ground_tool(void **state)
{
    (void)state;
    uint8_t frame[64];
    size_t len = make_frame(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), frame);

    assert_int_equal(len, sizeof(TX_DISABLE_CTR1_FRAME));
    assert_memory_equal(frame, TX_DISABLE_CTR1_FRAME, len);
}

static void test_authenticated_tx_disable_silences(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_SUCCESS);
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_int_equal(s_response.sequence_num, 1U);
    assert_int_equal(cmd_auth_last_counter(), 1U);
}

static void test_authenticated_tx_enable_restores(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(send(CMD_COMM_TX_ENABLE, TX_ENABLE_CTR2, sizeof(TX_ENABLE_CTR2)),
                     CMD_RESULT_SUCCESS);
    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** A captured TX_ENABLE replayed after a later TX-OFF must not re-enable TX */
static void test_replayed_tx_enable_cannot_undo_tx_off(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(send(CMD_COMM_TX_ENABLE, TX_ENABLE_CTR2, sizeof(TX_ENABLE_CTR2)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR3, sizeof(TX_DISABLE_CTR3)),
                     CMD_RESULT_SUCCESS);

    assert_int_equal(send(CMD_COMM_TX_ENABLE, TX_ENABLE_CTR2, sizeof(TX_ENABLE_CTR2)),
                     CMD_RESULT_AUTH_FAIL);
    assert_false(rf_control_tx_permitted());
}

/** Replay protection holds across a reset of the flight computer */
static void test_replay_rejected_after_reset(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(send(CMD_COMM_TX_ENABLE, TX_ENABLE_CTR2, sizeof(TX_ENABLE_CTR2)),
                     CMD_RESULT_SUCCESS);

    /* Reset: RAM state lost, NVM kept */
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);

    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_AUTH_FAIL);
    assert_true(rf_control_tx_permitted());
}

/** TX-OFF must be accepted in every operational state */
static void test_tx_disable_accepted_in_every_state(void **state)
{
    (void)state;
    const SmState_t states[] = {
        STATE_BOOT, STATE_SAFE, STATE_DETUMBLE, STATE_IDLE, STATE_ACTIVE, STATE_EMERGENCY
    };

    for (size_t i = 0U; i < (sizeof(states) / sizeof(states[0])); i++) {
        provision_test_key();  /* reset the counter so counter 1 is fresh */
        assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
        assert_int_equal(sm_force_state(sys_get_sm_context(), states[i]), SMART_QSO_OK);

        assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                         CMD_RESULT_SUCCESS);
        assert_false(rf_control_tx_permitted());
    }
}

/** With no usable key, TX_DISABLE still silences the station (97.207(b)),
 *  but nothing else is accepted, including TX_ENABLE */
static void test_no_key_material_tx_disable_still_silences(void **state)
{
    (void)state;
    uint8_t forged[CMD_AUTH_OVERHEAD];

    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    memset(forged, 0xEE, sizeof(forged));

    assert_int_equal(send(CMD_SYS_GET_STATUS, forged, sizeof(forged)), CMD_RESULT_AUTH_FAIL);
    assert_int_equal(send(CMD_COMM_TX_DISABLE, forged, sizeof(forged)), CMD_RESULT_SUCCESS);
    assert_false(rf_control_tx_permitted());
    assert_int_equal(send(CMD_COMM_TX_ENABLE, forged, sizeof(forged)), CMD_RESULT_AUTH_FAIL);
    assert_false(rf_control_tx_permitted());
}

/** The old public constant no longer authorizes anything: a payload too
 *  short to carry a counter and tag is malformed and never executed */
static void test_legacy_static_code_rejected(void **state)
{
    (void)state;
    const uint8_t legacy_auth[4] = {0xDEU, 0xADU, 0xBEU, 0xEFU};

    assert_int_equal(send(CMD_SYS_RESET, legacy_auth, sizeof(legacy_auth)),
                     CMD_RESULT_INVALID_PARAM);
    assert_int_equal(send(CMD_SYS_GET_STATUS, NULL, 0U), CMD_RESULT_INVALID_PARAM);
    assert_int_equal(cmd_auth_last_counter(), 0U);
}

/** A captured TX_DISABLE relabelled as TX_ENABLE fails authentication */
static void test_relabelled_frame_rejected(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_ENABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_AUTH_FAIL);
    assert_int_equal(cmd_auth_last_counter(), 0U);
}

/** Authentic but malformed commands are rejected without consuming the counter */
static void test_unexpected_arguments_rejected(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1_EXTRA_ARG,
                          sizeof(TX_DISABLE_CTR1_EXTRA_ARG)),
                     CMD_RESULT_INVALID_PARAM);
    assert_true(rf_control_tx_permitted());
    assert_int_equal(cmd_auth_last_counter(), 0U);
}

/** Payloads longer than the command buffer are rejected, never truncated */
static void test_oversized_payload_rejected(void **state)
{
    (void)state;
    uint8_t big[100];

    memset(big, 0x11, sizeof(big));
    assert_int_equal(send(CMD_COMM_TX_ENABLE, big, sizeof(big)), CMD_RESULT_INVALID_PARAM);
}

/** If the counter cannot be persisted, TX-OFF still executes (it is fail-safe) */
static void test_tx_disable_executes_despite_persist_failure(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(true);

    (void)send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1));
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** ... but any other command is refused when its counter cannot be persisted */
static void test_other_commands_refused_on_persist_failure(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_COMM_TX_DISABLE, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1)),
                     CMD_RESULT_SUCCESS);

    hal_sim_flash_fail_writes(true);
    assert_int_equal(send(CMD_COMM_TX_ENABLE, TX_ENABLE_CTR2, sizeof(TX_ENABLE_CTR2)),
                     CMD_RESULT_EXEC_FAIL);
    assert_false(rf_control_tx_permitted());
}

/** A flood of forged frames cannot drive unbounded fault-log flash writes:
 *  every failure is counted, but only the first is logged until a command
 *  is accepted */
static void test_forged_flood_logs_once(void **state)
{
    (void)state;
    uint8_t forged[sizeof(TX_DISABLE_CTR1)];
    CmdStats_t stats;

    memcpy(forged, TX_DISABLE_CTR1, sizeof(forged));
    forged[sizeof(forged) - 1U] ^= 0x01U;
    assert_int_equal(fault_log_clear(), SMART_QSO_OK);
    assert_int_equal(cmd_clear_stats(), SMART_QSO_OK);

    for (int i = 0; i < 50; i++) {
        assert_int_equal(send(CMD_COMM_TX_DISABLE, forged, sizeof(forged)), CMD_RESULT_AUTH_FAIL);
    }

    assert_int_equal(cmd_get_stats(&stats), SMART_QSO_OK);
    assert_int_equal(stats.auth_failures, 50U);
    assert_int_equal(fault_log_get_count(), 1U);
    assert_true(rf_control_tx_permitted());
}

/** Build counter | args | tag with the TEST-ONLY key (mirrors the ground tool) */
static size_t auth_payload(uint8_t cmd_id, uint32_t counter, const uint8_t *args,
                           size_t args_len, uint8_t *out)
{
    static const uint8_t domain[] = {'S', 'Q', 'S', 'O', '-', 'T', 'C', '-', 'v', '1'};
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t mac_input[sizeof(domain) + 2U + CMD_AUTH_COUNTER_SIZE + CMD_AUTH_MAX_ARGS];
    uint8_t mac[SHA256_DIGEST_SIZE];
    size_t pos = 0U;

    for (size_t i = 0U; i < sizeof(key); i++) {
        key[i] = (uint8_t)i;
    }
    out[0] = (uint8_t)(counter >> 24);
    out[1] = (uint8_t)(counter >> 16);
    out[2] = (uint8_t)(counter >> 8);
    out[3] = (uint8_t)counter;
    if (args_len > 0U) {
        memcpy(&out[CMD_AUTH_COUNTER_SIZE], args, args_len);
    }

    memcpy(mac_input, domain, sizeof(domain));
    pos = sizeof(domain);
    mac_input[pos++] = cmd_id;
    mac_input[pos++] = (uint8_t)args_len;
    memcpy(&mac_input[pos], out, CMD_AUTH_COUNTER_SIZE + args_len);
    pos += CMD_AUTH_COUNTER_SIZE + args_len;

    assert_int_equal(hmac_sha256(key, sizeof(key), mac_input, pos, mac), SMART_QSO_OK);
    memcpy(&out[CMD_AUTH_COUNTER_SIZE + args_len], mac, CMD_AUTH_TAG_SIZE);
    return CMD_AUTH_OVERHEAD + args_len;
}

/** Every implemented command passes framing, authentication and the
 *  argument check with its specified length, and is rejected with one extra
 *  byte; defined but unimplemented IDs are refused without using a counter */
static void test_every_dictionary_command(void **state)
{
    (void)state;
    static const struct {
        uint8_t cmd_id;
        uint8_t args_len;
    } dictionary[] = {
        {CMD_SYS_NOP, 0U}, {CMD_SYS_RESET, 0U}, {CMD_SYS_SET_MODE, 1U},
        {CMD_SYS_GET_STATUS, 0U}, {CMD_SYS_CLEAR_FAULTS, 0U}, {CMD_SYS_SET_CONTACT_TIMEOUT, 2U},
        {CMD_EPS_SET_MODE, 1U}, {CMD_EPS_GET_TELEMETRY, 0U},
        {CMD_ADCS_DETUMBLE, 0U}, {CMD_ADCS_GET_ATTITUDE, 0U},
        {CMD_COMM_SET_BEACON, 2U}, {CMD_COMM_TX_ENABLE, 0U}, {CMD_COMM_TX_DISABLE, 0U},
        {CMD_COMM_SET_POWER, 1U}, {CMD_PLD_ENABLE, 0U}, {CMD_PLD_DISABLE, 0U},
    };
    static const uint8_t unimplemented[] = {
        CMD_SYS_SET_TIME, CMD_EPS_ENABLE_HEATER, CMD_EPS_DISABLE_HEATER,
        CMD_ADCS_SET_MODE, CMD_ADCS_POINT_SUN,
        CMD_PLD_SEND_DATA, CMD_PLD_REQUEST_DATA, 0x7FU,
    };
    uint8_t args[CMD_AUTH_MAX_ARGS + 1U];
    uint8_t payload[CMD_AUTH_OVERHEAD + CMD_AUTH_MAX_ARGS + 1U];
    uint32_t counter = 0U;

    for (size_t i = 0U; i < (sizeof(dictionary) / sizeof(dictionary[0])); i++) {
        uint8_t id = dictionary[i].cmd_id;
        size_t n = dictionary[i].args_len;
        size_t len;
        CmdResult_t result;

        /* Valid values: mode 1 = IDLE keeps later commands allowed; beacon 60 s */
        memset(args, 1, sizeof(args));
        if (id == (uint8_t)CMD_COMM_SET_BEACON) {
            args[0] = 0x00U;
            args[1] = 0x3CU;
        }
        if (id == (uint8_t)CMD_SYS_SET_CONTACT_TIMEOUT) {
            args[0] = 0x00U;
            args[1] = 0x0EU;  /* 14 days */
        }

        counter++;
        len = auth_payload(id, counter, args, n, payload);
        result = send(id, payload, len);
        assert_int_not_equal(result, CMD_RESULT_INVALID_CMD);
        assert_int_not_equal(result, CMD_RESULT_INVALID_PARAM);
        assert_int_not_equal(result, CMD_RESULT_AUTH_FAIL);
        assert_int_equal(cmd_auth_last_counter(), counter);

        counter++;
        len = auth_payload(id, counter, args, n + 1U, payload);
        assert_int_equal(send(id, payload, len), CMD_RESULT_INVALID_PARAM);
        assert_int_equal(cmd_auth_last_counter(), counter - 1U);
        counter--;
    }

    for (size_t i = 0U; i < sizeof(unimplemented); i++) {
        uint8_t id = unimplemented[i];
        assert_int_equal(send(id, payload, auth_payload(id, counter + 1U, args, 0U, payload)),
                         CMD_RESULT_INVALID_CMD);
        assert_int_equal(cmd_auth_last_counter(), counter);
    }
}

static SmState_t current_state(void)
{
    SmState_t st = STATE_BOOT;
    assert_int_equal(sm_get_state(sys_get_sm_context(), &st), SMART_QSO_OK);
    return st;
}

static CmdResult_t send_set_mode(uint32_t counter, uint8_t mode)
{
    uint8_t payload[CMD_AUTH_OVERHEAD + 1U];
    return send(CMD_SYS_SET_MODE, payload,
                auth_payload(CMD_SYS_SET_MODE, counter, &mode, 1U, payload));
}

/** SET_MODE goes through the state machine's transition table: a ground
 *  command cannot jump SAFE -> ACTIVE (must pass through IDLE) */
static void test_set_mode_obeys_transition_table(void **state)
{
    (void)state;
    assert_int_equal(current_state(), STATE_IDLE);

    assert_int_equal(send_set_mode(1U, 0U), CMD_RESULT_SUCCESS);
    assert_int_equal(current_state(), STATE_SAFE);

    assert_int_equal(send_set_mode(2U, 2U), CMD_RESULT_NOT_ALLOWED);
    assert_int_equal(current_state(), STATE_SAFE);

    assert_int_equal(send_set_mode(3U, 1U), CMD_RESULT_SUCCESS);
    assert_int_equal(current_state(), STATE_IDLE);

    /* Already in the requested state */
    assert_int_equal(send_set_mode(4U, 1U), CMD_RESULT_SUCCESS);
    assert_int_equal(current_state(), STATE_IDLE);

    assert_int_equal(send_set_mode(5U, 2U), CMD_RESULT_SUCCESS);
    assert_int_equal(current_state(), STATE_ACTIVE);

    /* Unknown mode value */
    assert_int_equal(send_set_mode(6U, 7U), CMD_RESULT_INVALID_PARAM);
    assert_int_equal(current_state(), STATE_ACTIVE);
}

/** KEY_ROTATE: recovery key only, spare target only, any state; afterwards
 *  the old operational key is dead and the new one starts at counter 1 */
static void test_key_rotate_end_to_end(void **state)
{
    (void)state;
    uint8_t old_key[CMD_AUTH_KEY_SIZE];
    uint8_t new_key[CMD_AUTH_KEY_SIZE];
    uint8_t recovery_key[CMD_AUTH_KEY_SIZE];
    uint8_t payload[CMD_AUTH_OVERHEAD + 1U];
    uint8_t slot;

    test_key_fill(old_key, 0x00U);
    test_key_fill(new_key, 0x40U);
    test_key_fill(recovery_key, 0x80U);
    assert_int_equal(cmd_auth_provision_slot(1U, new_key, 11U), SMART_QSO_OK);
    assert_int_equal(cmd_auth_provision_slot(CMD_AUTH_RECOVERY_SLOT, recovery_key, 99U), SMART_QSO_OK);
    assert_int_equal(sm_force_state(sys_get_sm_context(), STATE_EMERGENCY), SMART_QSO_OK);

    /* The operational key cannot rotate keys */
    slot = 1U;
    assert_int_equal(send(CMD_SYS_KEY_ROTATE, payload,
                          test_auth_payload(old_key, CMD_SYS_KEY_ROTATE, 1U, &slot, 1U, payload)),
                     CMD_RESULT_AUTH_FAIL);
    /* Only a spare can be activated */
    slot = 2U;
    assert_int_equal(send(CMD_SYS_KEY_ROTATE, payload,
                          test_auth_payload(recovery_key, CMD_SYS_KEY_ROTATE, 1U, &slot, 1U, payload)),
                     CMD_RESULT_INVALID_PARAM);
    slot = 1U;
    assert_int_equal(send(CMD_SYS_KEY_ROTATE, payload,
                          test_auth_payload(recovery_key, CMD_SYS_KEY_ROTATE, 2U, &slot, 1U, payload)),
                     CMD_RESULT_SUCCESS);

    assert_int_equal(sm_force_state(sys_get_sm_context(), STATE_IDLE), SMART_QSO_OK);
    assert_int_equal(send(CMD_SYS_GET_STATUS, payload,
                          test_auth_payload(old_key, CMD_SYS_GET_STATUS, 50U, NULL, 0U, payload)),
                     CMD_RESULT_AUTH_FAIL);
    assert_int_equal(send(CMD_SYS_GET_STATUS, payload,
                          test_auth_payload(new_key, CMD_SYS_GET_STATUS, 1U, NULL, 0U, payload)),
                     CMD_RESULT_SUCCESS);
}

/** SET_POWER selects 0.5 W or 1 W (BASELINE section 7) and rejects other values */
static void test_set_power(void **state)
{
    (void)state;
    uint8_t payload[CMD_AUTH_OVERHEAD + 1U];
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t level;

    test_key_fill(key, 0x00U);
    level = 1U;
    assert_int_equal(send(CMD_COMM_SET_POWER, payload,
                          test_auth_payload(key, CMD_COMM_SET_POWER, 1U, &level, 1U, payload)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(rf_control_get_power_level(), RF_POWER_1W);
    level = 2U;
    assert_int_equal(send(CMD_COMM_SET_POWER, payload,
                          test_auth_payload(key, CMD_COMM_SET_POWER, 2U, &level, 1U, payload)),
                     CMD_RESULT_INVALID_PARAM);
    assert_int_equal(rf_control_get_power_level(), RF_POWER_1W);
}

/** Let the loss-of-contact timeout expire */
static void lose_contact(void)
{
    for (uint32_t d = 0U; d < contact_monitor_get_timeout_days(); d++) {
        contact_monitor_tick(86400U * 1000U);
    }
    assert_true(contact_monitor_inhibited());
    assert_false(rf_control_tx_permitted());
}

/** Any authentic command is contact, even one refused in the current state */
static void test_authentic_command_restores_contact(void **state)
{
    (void)state;
    uint8_t payload[CMD_AUTH_OVERHEAD + 2U];
    uint8_t key[CMD_AUTH_KEY_SIZE];
    const uint8_t beacon[2] = {0x00U, 0x3CU};

    test_key_fill(key, 0x00U);
    lose_contact();
    assert_int_equal(sm_force_state(sys_get_sm_context(), STATE_EMERGENCY), SMART_QSO_OK);
    assert_int_equal(send(CMD_COMM_SET_BEACON, payload,
                          test_auth_payload(key, CMD_COMM_SET_BEACON, 1U, beacon, 2U, payload)),
                     CMD_RESULT_NOT_ALLOWED);
    assert_false(contact_monitor_inhibited());
    assert_true(rf_control_tx_permitted());
}

/** Forged frames and the keyless TX_DISABLE are not contact */
static void test_unauthenticated_frames_are_not_contact(void **state)
{
    (void)state;
    uint8_t forged[CMD_AUTH_OVERHEAD];

    memset(forged, 0xEE, sizeof(forged));
    lose_contact();
    assert_int_equal(send(CMD_SYS_GET_STATUS, forged, sizeof(forged)), CMD_RESULT_AUTH_FAIL);
    assert_true(contact_monitor_inhibited());

    /* Authentic but rejected before its counter is spent (wrong argument
     * length): replayable, so it must not count as contact */
    {
        uint8_t key[CMD_AUTH_KEY_SIZE];
        uint8_t payload[CMD_AUTH_OVERHEAD + 1U];
        const uint8_t extra = 0x00U;
        test_key_fill(key, 0x00U);
        assert_int_equal(send(CMD_SYS_GET_STATUS, payload,
                              test_auth_payload(key, CMD_SYS_GET_STATUS, 1U, &extra, 1U, payload)),
                         CMD_RESULT_INVALID_PARAM);
        assert_true(contact_monitor_inhibited());
    }

    /* No key at all: TX_DISABLE executes, but is still not contact */
    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    assert_int_equal(send(CMD_COMM_TX_DISABLE, forged, sizeof(forged)), CMD_RESULT_SUCCESS);
    assert_true(contact_monitor_inhibited());
}

static void test_set_contact_timeout(void **state)
{
    (void)state;
    uint8_t payload[CMD_AUTH_OVERHEAD + 2U];
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t days[2] = {0x00U, 30U};

    test_key_fill(key, 0x00U);
    assert_int_equal(send(CMD_SYS_SET_CONTACT_TIMEOUT, payload,
                          test_auth_payload(key, CMD_SYS_SET_CONTACT_TIMEOUT, 1U, days, 2U, payload)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(contact_monitor_get_timeout_days(), 30U);

    days[1] = 0U;
    assert_int_equal(send(CMD_SYS_SET_CONTACT_TIMEOUT, payload,
                          test_auth_payload(key, CMD_SYS_SET_CONTACT_TIMEOUT, 2U, days, 2U, payload)),
                     CMD_RESULT_INVALID_PARAM);
    days[1] = CONTACT_TIMEOUT_MAX_DAYS + 1U;
    assert_int_equal(send(CMD_SYS_SET_CONTACT_TIMEOUT, payload,
                          test_auth_payload(key, CMD_SYS_SET_CONTACT_TIMEOUT, 3U, days, 2U, payload)),
                     CMD_RESULT_INVALID_PARAM);
    assert_int_equal(contact_monitor_get_timeout_days(), 30U);
}

static void test_authenticated_get_status(void **state)
{
    (void)state;
    assert_int_equal(send(CMD_SYS_GET_STATUS, GET_STATUS_CTR1, sizeof(GET_STATUS_CTR1)),
                     CMD_RESULT_SUCCESS);
    assert_int_equal(s_response.data_len, 4U);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_frame_matches_ground_tool),
        cmocka_unit_test_setup_teardown(test_authenticated_tx_disable_silences, setup, teardown),
        cmocka_unit_test_setup_teardown(test_authenticated_tx_enable_restores, setup, teardown),
        cmocka_unit_test_setup_teardown(test_replayed_tx_enable_cannot_undo_tx_off, setup, teardown),
        cmocka_unit_test_setup_teardown(test_replay_rejected_after_reset, setup, teardown),
        cmocka_unit_test_setup_teardown(test_tx_disable_accepted_in_every_state, setup, teardown),
        cmocka_unit_test_setup_teardown(test_no_key_material_tx_disable_still_silences, setup, teardown),
        cmocka_unit_test_setup_teardown(test_legacy_static_code_rejected, setup, teardown),
        cmocka_unit_test_setup_teardown(test_relabelled_frame_rejected, setup, teardown),
        cmocka_unit_test_setup_teardown(test_unexpected_arguments_rejected, setup, teardown),
        cmocka_unit_test_setup_teardown(test_oversized_payload_rejected, setup, teardown),
        cmocka_unit_test_setup_teardown(test_tx_disable_executes_despite_persist_failure, setup, teardown),
        cmocka_unit_test_setup_teardown(test_other_commands_refused_on_persist_failure, setup, teardown),
        cmocka_unit_test_setup_teardown(test_authenticated_get_status, setup, teardown),
        cmocka_unit_test_setup_teardown(test_forged_flood_logs_once, setup, teardown),
        cmocka_unit_test_setup_teardown(test_every_dictionary_command, setup, teardown),
        cmocka_unit_test_setup_teardown(test_set_mode_obeys_transition_table, setup, teardown),
        cmocka_unit_test_setup_teardown(test_key_rotate_end_to_end, setup, teardown),
        cmocka_unit_test_setup_teardown(test_set_power, setup, teardown),
        cmocka_unit_test_setup_teardown(test_authentic_command_restores_contact, setup, teardown),
        cmocka_unit_test_setup_teardown(test_unauthenticated_frames_are_not_contact, setup, teardown),
        cmocka_unit_test_setup_teardown(test_set_contact_timeout, setup, teardown),
    };

    return cmocka_run_group_tests_name("Command Handler (Telecommand Path) Tests", tests, NULL, NULL);
}
