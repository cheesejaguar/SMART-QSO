/**
 * @file test_rf_silence.c
 * @brief 45-minute post-deployment RF silence (CDS Rev 14.1 2.4.5, 2.3.2.4)
 *
 * @requirement SYS-224 No RF signal earlier than 45 minutes after deployment; switch toggle resets the timer
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "rf_silence.h"
#include "rf_control.h"
#include "beacon.h"
#include "cmd_handler.h"
#include "cmd_auth.h"
#include "system_state.h"
#include "state_machine.h"
#include "nvm_record.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_reset.h"
#include "hal/hal_sim.h"
#include "auth_test_helpers.h"

#define MINUTE_MS   (60U * 1000U)

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    hal_sim_set_reset_cause(HAL_RESET_POWER_ON);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);  /* switches released */
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_SILENCE), SMART_QSO_OK);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    return 0;
}

static void advance_minutes(uint32_t minutes)
{
    for (uint32_t m = 0U; m < minutes; m++) {
        rf_silence_tick(MINUTE_MS);
    }
}

static void assert_silent(void)
{
    assert_true(rf_silence_active());
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_false(hal_gpio_get(GPIO_PIN_RADIO_EN));
}

static void assert_radiating_allowed(void)
{
    assert_false(rf_silence_active());
    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN));
}

/** A fresh (blank) unit is silent, radio unpowered, for exactly 45 minutes */
static void test_silent_for_45_minutes(void **state)
{
    (void)state;
    BeaconContent_t content;

    assert_silent();
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    memset(&content, 0, sizeof(content));
    (void)strncpy(content.text, "TEST", sizeof(content.text) - 1U);
    assert_int_equal(beacon_transmit(&content), SMART_QSO_ERROR);

    advance_minutes(44U);
    rf_silence_tick(MINUTE_MS - 1000U);
    assert_silent();

    rf_silence_tick(1000U);
    assert_radiating_allowed();
}

/** A watchdog reset mid-countdown resumes from the persisted count */
static void test_watchdog_reset_resumes(void **state)
{
    (void)state;
    advance_minutes(30U);
    hal_sim_set_reset_cause(HAL_RESET_WATCHDOG);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_silent();
    assert_int_equal(rf_silence_elapsed_seconds(), 30U * 60U);

    advance_minutes(15U);
    assert_radiating_allowed();
}

/** Power-on, brownout, pin or unknown resets restart the count: removing
 *  power is what a deployment-switch toggle does (CDS 2.3.2.4) */
static void test_power_loss_restarts(void **state)
{
    (void)state;
    const HalResetCause_t causes[] = {
        HAL_RESET_POWER_ON, HAL_RESET_BROWNOUT, HAL_RESET_PIN, HAL_RESET_UNKNOWN
    };

    for (size_t i = 0U; i < (sizeof(causes) / sizeof(causes[0])); i++) {
        advance_minutes(40U);
        hal_sim_set_reset_cause(causes[i]);
        assert_int_equal(rf_silence_init(), SMART_QSO_OK);
        assert_int_equal(rf_silence_elapsed_seconds(), 0U);
        assert_silent();
    }
    advance_minutes(45U);
    assert_radiating_allowed();
}

/** A switch toggle seen while powered restarts the count */
static void test_switch_toggle_restarts(void **state)
{
    (void)state;
    advance_minutes(40U);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, true);   /* stowed */
    rf_silence_tick(MINUTE_MS);
    assert_int_equal(rf_silence_elapsed_seconds(), 0U);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);  /* released */

    advance_minutes(44U);
    assert_silent();
    advance_minutes(1U);
    assert_radiating_allowed();
}

/** Completion persists across later power-on resets */
static void test_completion_persists(void **state)
{
    (void)state;
    advance_minutes(45U);
    hal_sim_set_reset_cause(HAL_RESET_POWER_ON);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_radiating_allowed();
}

/** Once served, the silence is latched: a sense line reading stowed later
 *  (in orbit, a failed switch or line) does not restart it (LV-15). Only
 *  GSE ARM re-arms a completed unit (test_arm_resets_completed_unit) */
static void test_stowed_after_completion_stays_complete(void **state)
{
    (void)state;
    advance_minutes(45U);
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, true);
    rf_silence_tick(MINUTE_MS);
    assert_radiating_allowed();
    assert_int_equal(rf_silence_get_state(), RF_SILENCE_COMPLETE);
    advance_minutes(60U);
    assert_radiating_allowed();
    hal_sim_gpio_drive_input(GPIO_PIN_DEPLOY_SENSE, false);
}

/** GSE ARM makes the next deployment silent again. ARMED does not count
 *  while powered, so bench time after ARM cannot complete the silence; the
 *  count starts at the next boot */
static void test_arm_resets_completed_unit(void **state)
{
    (void)state;
    advance_minutes(45U);
    assert_int_equal(rf_silence_arm(), SMART_QSO_OK);
    assert_silent();
    assert_int_equal(rf_silence_get_state(), RF_SILENCE_ARMED);

    advance_minutes(120U);
    assert_silent();
    assert_int_equal(rf_silence_get_state(), RF_SILENCE_ARMED);

    hal_sim_set_reset_cause(HAL_RESET_WATCHDOG);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_silent();
    assert_int_equal(rf_silence_get_state(), RF_SILENCE_COUNTING);
}

/** A corrupt record means armed (longer silence, never shorter) */
static void test_corrupt_record_is_armed(void **state)
{
    (void)state;
    uint8_t bytes[4];

    advance_minutes(45U);
    for (uint32_t slot = 0U; slot < 2U; slot++) {
        uint32_t offset = (slot * NVM_RECORD_SLOT_SIZE) + 8U;
        assert_int_equal(hal_flash_read(FLASH_REGION_RF_SILENCE, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
        for (size_t i = 0U; i < sizeof(bytes); i++) {
            bytes[i] ^= 0xFFU;
        }
        assert_int_equal(hal_flash_write(FLASH_REGION_RF_SILENCE, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
    }
    hal_sim_set_reset_cause(HAL_RESET_WATCHDOG);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_silent();
    assert_int_equal(rf_silence_elapsed_seconds(), 0U);
}

/** No telecommand shortens the silence: an authenticated TX_ENABLE clears
 *  the TX-OFF latch but the silence holds */
static void test_tx_enable_cannot_shorten_silence(void **state)
{
    (void)state;
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t payload[CMD_AUTH_OVERHEAD];
    uint8_t frame[CMD_AUTH_OVERHEAD + 6U];
    CmdResponse_t response;
    uint16_t sum = 0U;
    size_t len;

    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(sm_force_state(sys_get_sm_context(), STATE_IDLE), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    test_key_fill(key, 0x00U);
    assert_int_equal(cmd_auth_provision_key(key, 1U), SMART_QSO_OK);
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);

    len = test_auth_payload(key, (uint8_t)CMD_COMM_TX_ENABLE, 1U, NULL, 0U, payload);
    frame[0] = 0xAAU;
    frame[1] = 0x55U;
    frame[2] = (uint8_t)CMD_COMM_TX_ENABLE;
    frame[3] = (uint8_t)len;
    memcpy(&frame[4], payload, len);
    for (size_t i = 0U; i < (4U + len); i++) {
        sum = (uint16_t)(sum + frame[i]);
    }
    frame[4U + len] = (uint8_t)(sum >> 8);
    frame[5U + len] = (uint8_t)sum;

    assert_int_equal(cmd_process(frame, 6U + len, &response), SMART_QSO_OK);
    assert_int_equal(response.result, CMD_RESULT_SUCCESS);
    assert_silent();
}

/** The silence never clears a TX-OFF latch */
static void test_completion_keeps_tx_off_latch(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    advance_minutes(45U);
    assert_false(rf_silence_active());
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN));  /* receiver back on */
}

/** GSE ARM that cannot be persisted reports the I/O error and still
 *  silences the radio now */
static void test_arm_not_persisted_still_silences(void **state)
{
    (void)state;
    advance_minutes(45U);
    assert_radiating_allowed();
    hal_sim_flash_fail_writes(true);

    assert_int_equal(rf_silence_arm(), SMART_QSO_ERROR_IO);
    assert_silent();
    assert_int_equal(rf_silence_get_state(), RF_SILENCE_ARMED);
    hal_sim_flash_fail_writes(false);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_silent_for_45_minutes, setup),
        cmocka_unit_test_setup(test_watchdog_reset_resumes, setup),
        cmocka_unit_test_setup(test_power_loss_restarts, setup),
        cmocka_unit_test_setup(test_switch_toggle_restarts, setup),
        cmocka_unit_test_setup(test_completion_persists, setup),
        cmocka_unit_test_setup(test_stowed_after_completion_stays_complete, setup),
        cmocka_unit_test_setup(test_arm_resets_completed_unit, setup),
        cmocka_unit_test_setup(test_corrupt_record_is_armed, setup),
        cmocka_unit_test_setup(test_tx_enable_cannot_shorten_silence, setup),
        cmocka_unit_test_setup(test_completion_keeps_tx_off_latch, setup),
        cmocka_unit_test_setup(test_arm_not_persisted_still_silences, setup),
    };

    return cmocka_run_group_tests_name("RF Silence Tests", tests, NULL, NULL);
}
