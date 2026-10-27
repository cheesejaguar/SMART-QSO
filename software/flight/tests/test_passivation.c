/**
 * @file test_passivation.c
 * @brief End-of-life passivation by telecommand, and the C/Rust contract of
 *        the CDR additions (ids 400-449 of rust_contract.h)
 *
 * Expectations from passivation.h, SQSO-ICD-013 §6.4 and
 * docs/software/SOFTWARE_REQUIREMENTS.md §3.12:
 *
 * - PASSIVATE_EXECUTE alone does nothing; PASSIVATE_ARM needs its code;
 * - the arm lapses after 600 s;
 * - execute: the record first, then (in sunlight) EOL_ARM, then EOL_FIRE
 *   held at least 60 s, then both low, all while the transmitter is still
 *   permitted (RID REG-12);
 * - then the final report: three frames, 60 s apart; TX-OFF 180 s after the
 *   first; a fire the EPS never confirms is tried three times, reported,
 *   and silenced all the same;
 * - after that no telecommand and no reset lets a transmitter on; the
 *   receiver stays powered;
 * - end to end, ARM with the operational key and EXECUTE with the recovery
 *   key; EXECUTE with the operational key is refused.
 *
 * @requirement SRS-F110 Two-step authenticated end-of-life passivation
 * @requirement SRS-F111 Passivation order: record, charge disconnect through two series lines while transmitting, final report, then TX-OFF
 * @requirement SRS-F112 Transmitters off for good after passivation
 * @requirement SRS-F155 The final passivation report: three frames over three minutes carrying EOL_STATE, the fuse state and the attempts, before TX-OFF
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>
#include <math.h>

#include "passivation.h"
#include "rf_control.h"
#include "cmd_handler.h"
#include "cmd_auth.h"
#include "cmd_dictionary.h"
#include "contact_monitor.h"
#include "fault_mgmt.h"
#include "system_state.h"
#include "hk_store.h"
#include "receiver.h"
#include "eps_lockout.h"
#include "beacon.h"
#include "rust_contract.h"
#include "auth_test_helpers.h"
#include "hal/hal.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_i2c.h"
#include "hal/hal_sim.h"

static const uint8_t ARM_CODE[4] = {'E', 'O', 'L', '!'};

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_i2c_init(I2C_BUS_0, 100000U), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_EOL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_CONTACT), SMART_QSO_OK);
    (void)fault_mgmt_init();
    assert_int_equal(passivation_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    return 0;
}

/** The EPS controller's EOL_STATE register (bit 1: charge path open) */
static void eps_eol_state(uint8_t value)
{
    const uint8_t reg[2] = {0x7CU, value};
    assert_int_equal(hal_i2c_write(I2C_BUS_0, 0x48U, reg, sizeof(reg)), SMART_QSO_OK);
}

static void test_execute_needs_a_live_arm(void **state)
{
    (void)state;
    const uint8_t wrong[4] = {'E', 'O', 'L', '?'};
    assert_int_equal(passivation_phase(), PASSIVATION_IDLE);
    assert_int_equal(passivation_execute(), SMART_QSO_ERROR);
    assert_int_equal(passivation_arm(wrong, sizeof(wrong)), SMART_QSO_ERROR_INVALID);
    assert_int_equal(passivation_arm(ARM_CODE, sizeof(ARM_CODE)), SMART_QSO_OK);
    assert_int_equal(passivation_phase(), PASSIVATION_ARMED);
    assert_false(hal_gpio_get(GPIO_PIN_EOL_ARM));
    assert_true(rf_control_tx_permitted()); /* arming changes nothing on the air */
    smart_qso_sim_advance_ms(PASSIVATION_ARM_WINDOW_MS);
    passivation_tick(true);
    assert_int_equal(passivation_phase(), PASSIVATION_IDLE);
    assert_int_equal(passivation_execute(), SMART_QSO_ERROR); /* lapsed */
    assert_false(passivation_passivated());
}

static void test_execute_order_and_for_good(void **state)
{
    (void)state;
    eps_eol_state(0U);
    assert_int_equal(passivation_arm(ARM_CODE, sizeof(ARM_CODE)), SMART_QSO_OK);
    assert_int_equal(passivation_execute(), SMART_QSO_OK);
    assert_int_equal(passivation_phase(), PASSIVATION_FIRING);
    assert_true(rf_control_tx_permitted()); /* still transmitting during the fire */
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN));
    assert_int_equal(passivation_take_report(), 0U);
    passivation_tick(false); /* eclipse: nothing fired */
    assert_false(hal_gpio_get(GPIO_PIN_EOL_ARM));
    passivation_tick(true);
    assert_true(hal_gpio_get(GPIO_PIN_EOL_ARM));
    assert_false(hal_gpio_get(GPIO_PIN_EOL_FIRE));
    assert_int_equal(passivation_attempts(), 1U);
    smart_qso_sim_advance_ms(200U);
    passivation_tick(true);
    assert_true(hal_gpio_get(GPIO_PIN_EOL_FIRE));
    eps_eol_state(0x03U); /* opened early: still held for the minimum */
    smart_qso_sim_advance_ms(PASSIVATION_FIRE_MIN_MS - 1U);
    passivation_tick(true);
    assert_true(hal_gpio_get(GPIO_PIN_EOL_FIRE));
    assert_true(rf_control_tx_permitted());
    smart_qso_sim_advance_ms(1U);
    passivation_tick(true);
    assert_false(hal_gpio_get(GPIO_PIN_EOL_FIRE));
    assert_false(hal_gpio_get(GPIO_PIN_EOL_ARM));

    /* The final report: three frames 60 s apart, still on the air */
    assert_int_equal(passivation_phase(), PASSIVATION_REPORTING);
    assert_true(rf_control_tx_permitted());
    assert_false(passivation_passivated());
    assert_int_equal(passivation_take_report(), 1U);
    assert_int_equal(passivation_take_report(), 0U);
    smart_qso_sim_advance_ms(PASSIVATION_REPORT_INTERVAL_MS - 1U);
    assert_int_equal(passivation_take_report(), 0U);
    smart_qso_sim_advance_ms(1U);
    assert_int_equal(passivation_take_report(), 2U);
    smart_qso_sim_advance_ms(PASSIVATION_REPORT_INTERVAL_MS);
    passivation_tick(true);
    assert_int_equal(passivation_take_report(), 3U);
    assert_true(rf_control_tx_permitted());
    smart_qso_sim_advance_ms(PASSIVATION_REPORT_WINDOW_MS - (2U * PASSIVATION_REPORT_INTERVAL_MS) - 1U);
    passivation_tick(true);
    assert_int_equal(passivation_take_report(), 0U);
    assert_true(rf_control_tx_permitted());
    smart_qso_sim_advance_ms(1U);
    passivation_tick(true);

    /* Then TX-OFF and the inhibit */
    assert_int_equal(passivation_phase(), PASSIVATION_PASSIVATED);
    assert_true(passivation_passivated());
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN)); /* the receiver stays on */

    /* No command, no reset turns a transmitter on again */
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());
    rf_control_set_inhibit(RF_INHIBIT_PASSIVATED, false);
    assert_false(rf_control_tx_permitted());
    assert_int_equal(passivation_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());
    assert_true(passivation_passivated());
    assert_int_equal(passivation_arm(ARM_CODE, sizeof(ARM_CODE)), SMART_QSO_ERROR);
    passivation_verify();
    assert_true(passivation_passivated());
}

/** A fire the EPS never confirms: three attempts, the report, then silence
 * (runs last: the inhibit of the test before stays in RAM) */
static void test_unconfirmed_fire_is_reported_then_silenced(void **state)
{
    (void)state;
    eps_eol_state(0U);
    assert_int_equal(passivation_arm(ARM_CODE, sizeof(ARM_CODE)), SMART_QSO_OK);
    assert_int_equal(passivation_execute(), SMART_QSO_OK);
    for (uint8_t attempt = 1U; attempt <= PASSIVATION_MAX_FIRE_ATTEMPTS; attempt++) {
        passivation_tick(true);
        assert_int_equal(passivation_attempts(), attempt);
        assert_true(hal_gpio_get(GPIO_PIN_EOL_ARM));
        smart_qso_sim_advance_ms(200U);
        passivation_tick(true);
        assert_true(hal_gpio_get(GPIO_PIN_EOL_FIRE));
        smart_qso_sim_advance_ms(PASSIVATION_FIRE_MAX_MS - 1U);
        passivation_tick(true);
        assert_true(hal_gpio_get(GPIO_PIN_EOL_FIRE));
        smart_qso_sim_advance_ms(1U);
        passivation_tick(true);
        assert_false(hal_gpio_get(GPIO_PIN_EOL_FIRE));
        assert_false(hal_gpio_get(GPIO_PIN_EOL_ARM));
    }
    assert_int_equal(passivation_phase(), PASSIVATION_REPORTING);
    passivation_tick(true);
    assert_int_equal(passivation_attempts(), PASSIVATION_MAX_FIRE_ATTEMPTS); /* no fourth */
    assert_false(hal_gpio_get(GPIO_PIN_EOL_ARM));
    assert_int_equal(passivation_take_report(), 1U);
    smart_qso_sim_advance_ms(PASSIVATION_REPORT_WINDOW_MS);
    passivation_tick(true);
    assert_int_equal(passivation_phase(), PASSIVATION_PASSIVATED);
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

static CmdResponse_t s_rsp;

static CmdResult_t send(const uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t id, uint32_t ctr, const uint8_t *args,
                        size_t len)
{
    uint8_t payload[96];
    uint8_t frame[128];
    size_t n = test_auth_payload(key, id, ctr, args, len, payload);
    uint16_t sum = 0U;
    frame[0] = 0xAAU;
    frame[1] = 0x55U;
    frame[2] = id;
    frame[3] = (uint8_t)n;
    memcpy(&frame[4], payload, n);
    for (size_t i = 0U; i < (4U + n); i++) {
        sum = (uint16_t)(sum + frame[i]);
    }
    frame[4U + n] = (uint8_t)(sum >> 8);
    frame[5U + n] = (uint8_t)sum;
    memset(&s_rsp, 0, sizeof(s_rsp));
    assert_int_equal(cmd_process(frame, 6U + n, &s_rsp), SMART_QSO_OK);
    return s_rsp.result;
}

/** End to end: the operational key arms, only the recovery key executes */
static void test_two_keys_end_to_end(void **state)
{
    (void)state;
    uint8_t op[CMD_AUTH_KEY_SIZE];
    uint8_t rec[CMD_AUTH_KEY_SIZE];
    test_key_fill(op, 0x00U);
    test_key_fill(rec, 0x80U);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    assert_int_equal(cmd_auth_provision_key(op, 1U), SMART_QSO_OK);
    assert_int_equal(cmd_auth_provision_slot(CMD_AUTH_RECOVERY_SLOT, rec, 99U), SMART_QSO_OK);

    assert_int_equal(send(op, (uint8_t)CMD_EOL_PASSIVATE_EXECUTE, 1U, NULL, 0U), CMD_RESULT_AUTH_FAIL);
    assert_int_equal(send(rec, (uint8_t)CMD_EOL_PASSIVATE_EXECUTE, 1U, NULL, 0U), CMD_RESULT_NOT_ALLOWED);
    assert_int_equal(send(op, (uint8_t)CMD_EOL_PASSIVATE_ARM, 1U, (const uint8_t *)"EOL?", 4U),
                     CMD_RESULT_INVALID_PARAM);
    assert_int_equal(send(op, (uint8_t)CMD_EOL_PASSIVATE_ARM, 2U, ARM_CODE, 4U), CMD_RESULT_SUCCESS);
    assert_true(rf_control_tx_permitted());
    assert_int_equal(send(op, (uint8_t)CMD_EOL_PASSIVATE_EXECUTE, 3U, NULL, 0U), CMD_RESULT_AUTH_FAIL);
    assert_int_equal(send(rec, (uint8_t)CMD_EOL_PASSIVATE_EXECUTE, 2U, NULL, 0U), CMD_RESULT_SUCCESS);
    assert_int_equal(passivation_phase(), PASSIVATION_FIRING);
    assert_true(rf_control_tx_permitted()); /* on the air until the final report */
    assert_int_equal(send(rec, (uint8_t)CMD_EOL_PASSIVATE_EXECUTE, 3U, NULL, 0U), CMD_RESULT_NOT_ALLOWED);
    assert_int_equal(send(op, (uint8_t)CMD_EOL_PASSIVATE_ARM, 3U, ARM_CODE, 4U), CMD_RESULT_NOT_ALLOWED);
}

/** What the Rust library assumes about the CDR additions to the headers */
static void test_contract_400(void **state)
{
    (void)state;
    static const struct {
        int id;
        double value;
    } k[] = {
        {RUST_CONST_FLASH_REGION_HK_SUMMARY, FLASH_REGION_HK_SUMMARY},
        {RUST_CONST_FLASH_REGION_HK_DETAIL, FLASH_REGION_HK_DETAIL},
        {RUST_CONST_FLASH_REGION_HK_INDEX, FLASH_REGION_HK_INDEX},
        {RUST_CONST_FLASH_REGION_EOL, FLASH_REGION_EOL},
        {RUST_CONST_PIN_EOL_ARM, GPIO_PIN_EOL_ARM},
        {RUST_CONST_PIN_EOL_FIRE, GPIO_PIN_EOL_FIRE},
        {RUST_CONST_PIN_RX_CD, GPIO_PIN_RX_CD},
        {RUST_CONST_PIN_RX_FAULT, GPIO_PIN_RX_FAULT},
        {RUST_CONST_PIN_BURN_W2B, GPIO_PIN_BURN_W2B},
        {RUST_CONST_PIN_LOCK_CLR, GPIO_PIN_LOCK_CLR},
        {RUST_CONST_RF_INHIBIT_PASSIVATED, RF_INHIBIT_PASSIVATED},
        {RUST_CONST_RF_INHIBIT_RX_CYCLE, RF_INHIBIT_RX_CYCLE},
        {RUST_CONST_PASSIVATION_ARM_WINDOW_MS, PASSIVATION_ARM_WINDOW_MS},
        {RUST_CONST_PASSIVATION_FIRE_MIN_MS, PASSIVATION_FIRE_MIN_MS},
        {RUST_CONST_PASSIVATION_FIRE_MAX_MS, PASSIVATION_FIRE_MAX_MS},
        {RUST_CONST_HK_STORE_RECORD_LEN, HK_STORE_RECORD_LEN},
        {RUST_CONST_HK_STORE_MAX_DUMP, HK_STORE_MAX_DUMP},
        {RUST_CONST_RECEIVER_SILENCE_MS, (double)RECEIVER_SILENCE_MS},
        {RUST_CONST_RECEIVER_CYCLE_OFF_MS, RECEIVER_CYCLE_OFF_MS},
        {RUST_CONST_BEACON_JITTER_MS, BEACON_JITTER_MS},
        {RUST_CONST_GPIO_PIN_COUNT, GPIO_PIN_COUNT},
        {RUST_CONST_FLASH_REGION_COUNT, FLASH_REGION_COUNT},
    };
    for (size_t i = 0U; i < sizeof(k) / sizeof(k[0]); i++) {
        assert_true(fabs(sqso_rust_constant((RustConst_t)k[i].id) - k[i].value) < 0.5);
    }
    assert_int_equal(sizeof(k) / sizeof(k[0]), (size_t)(RUST_CONST_GROUP_400_END - 400));
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_execute_needs_a_live_arm, setup),
        /* Before any test passivates: RF_INHIBIT_PASSIVATED is never lifted,
         * and only a power cycle clears the RAM that holds it */
        cmocka_unit_test_setup(test_two_keys_end_to_end, setup),
        cmocka_unit_test_setup(test_execute_order_and_for_good, setup),
        cmocka_unit_test_setup(test_unconfirmed_fire_is_reported_then_silenced, setup),
        cmocka_unit_test(test_contract_400),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
