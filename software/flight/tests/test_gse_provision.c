/**
 * @file test_gse_provision.c
 * @brief Key provisioning over the GSE umbilical
 *
 * The reference frame was generated with software/ground/tools/key_provision.py
 * logic (Python zlib.crc32), so the first test also proves ground/flight CRC
 * agreement. Keys here are TEST-ONLY patterns.
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "gse_provision.h"
#include "cmd_auth.h"
#include "fault_mgmt.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_uart.h"
#include "hal/hal_sim.h"
#include "auth_test_helpers.h"
#include "rf_silence.h"
#include "rf_control.h"

/** PROVISION slot 0, key ID 1, TEST-ONLY key 00..1F (Python reference) */
static const uint8_t PROVISION_SLOT0[GSE_FRAME_SIZE] = {
    0x53, 0x51, 0x47, 0x50, 0x01, 0x00, 0x01, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
    0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0xE9, 0x33, 0x64, 0xB5
};

/** TX_DISABLE, counter 1, TEST-ONLY key 00..1F (shared with test_cmd_auth.c) */
static const uint8_t TX_DISABLE_CTR1[] = {
    0x00, 0x00, 0x00, 0x01, 0x9A, 0x83, 0xF5, 0xC7, 0xB2, 0x1F,
    0x75, 0x11, 0xE8, 0x2F, 0x5E, 0x94, 0xB9, 0xDE, 0x58, 0x51
};

/** Build a GSE frame (mirrors key_provision.py) */
static void make_frame(uint8_t op, uint8_t slot, uint8_t key_id, const uint8_t *key,
                       uint8_t out[GSE_FRAME_SIZE])
{
    memset(out, 0, GSE_FRAME_SIZE);
    out[0] = 'S';
    out[1] = 'Q';
    out[2] = 'G';
    out[3] = 'P';
    out[4] = op;
    out[5] = slot;
    out[6] = key_id;
    if (key != NULL) {
        memcpy(&out[7], key, CMD_AUTH_KEY_SIZE);
    }
    uint32_t crc = smart_qso_crc32(out, GSE_FRAME_SIZE - 4U);
    out[39] = (uint8_t)(crc >> 24);
    out[40] = (uint8_t)(crc >> 16);
    out[41] = (uint8_t)(crc >> 8);
    out[42] = (uint8_t)crc;
}

/** Whether slot 0 holds a spare key: activating it succeeds */
static bool slot0_was_stored(void)
{
    uint8_t activate[GSE_FRAME_SIZE];
    make_frame(GSE_OP_ACTIVATE, 0U, 0U, NULL, activate);
    hal_sim_gpio_drive_input(GPIO_PIN_UMBILICAL_SENSE, true);
    return gse_process_frame(activate, sizeof(activate)) == GSE_STATUS_OK;
}

static int setup(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(cmd_auth_init(), SMART_QSO_OK);
    assert_int_equal(gse_init(), SMART_QSO_OK);
    assert_int_equal(hal_uart_flush(UART_PORT_DEBUG), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_UMBILICAL_SENSE, true);
    return 0;
}

static void test_reference_frame_provisions_and_activates(void **state)
{
    (void)state;
    uint8_t activate[GSE_FRAME_SIZE];
    CmdAuthVerified_t v;

    assert_int_equal(gse_process_frame(PROVISION_SLOT0, sizeof(PROVISION_SLOT0)), GSE_STATUS_OK);
    assert_false(cmd_auth_is_provisioned());  /* stored as a spare */

    make_frame(GSE_OP_ACTIVATE, 0U, 0U, NULL, activate);
    assert_int_equal(gse_process_frame(activate, sizeof(activate)), GSE_STATUS_OK);
    assert_true(cmd_auth_is_provisioned());
    assert_int_equal(cmd_auth_verify(0x32U, TX_DISABLE_CTR1, sizeof(TX_DISABLE_CTR1), &v), CMD_AUTH_OK);
}

/** Without the umbilical (on orbit) nothing can be provisioned */
static void test_locked_without_umbilical(void **state)
{
    (void)state;
    hal_sim_gpio_drive_input(GPIO_PIN_UMBILICAL_SENSE, false);
    assert_false(gse_umbilical_present());
    assert_int_equal(gse_process_frame(PROVISION_SLOT0, sizeof(PROVISION_SLOT0)), GSE_STATUS_LOCKED);
    assert_false(slot0_was_stored());
}

static void test_malformed_frames_rejected(void **state)
{
    (void)state;
    uint8_t frame[GSE_FRAME_SIZE];

    memcpy(frame, PROVISION_SLOT0, sizeof(frame));
    frame[20] ^= 0x01U;  /* key byte: CRC no longer matches */
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_BAD_FRAME);

    memcpy(frame, PROVISION_SLOT0, sizeof(frame));
    frame[0] = 'X';
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_BAD_FRAME);
    assert_int_equal(gse_process_frame(PROVISION_SLOT0, sizeof(PROVISION_SLOT0) - 1U),
                     GSE_STATUS_BAD_FRAME);
    assert_int_equal(gse_process_frame(NULL, GSE_FRAME_SIZE), GSE_STATUS_BAD_FRAME);
    assert_false(slot0_was_stored());
}

static void test_bad_operation_and_slot(void **state)
{
    (void)state;
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t frame[GSE_FRAME_SIZE];

    test_key_fill(key, 0x40U);
    make_frame(0x7FU, 0U, 1U, key, frame);
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_BAD_OP);

    make_frame(GSE_OP_PROVISION, CMD_AUTH_RECOVERY_SLOT + 1U, 1U, key, frame);
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_REJECTED);

    make_frame(GSE_OP_ACTIVATE, 2U, 0U, NULL, frame);  /* blank slot */
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_REJECTED);
}

static void test_recovery_key_and_zeroize(void **state)
{
    (void)state;
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t frame[GSE_FRAME_SIZE];
    CmdAuthInfo_t info;

    test_key_fill(key, 0x80U);
    make_frame(GSE_OP_PROVISION, CMD_AUTH_RECOVERY_SLOT, 99U, key, frame);
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_OK);
    cmd_auth_get_info(&info);
    assert_true(info.recovery_usable);
    assert_int_equal(info.recovery_key_id, 99U);

    make_frame(GSE_OP_ZEROIZE, 0U, 0U, NULL, frame);
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_OK);
    assert_false(cmd_auth_has_key_material());
}

/** gse_poll() reads frames from the umbilical UART only while it is connected */
static void test_poll_reads_umbilical_uart(void **state)
{
    (void)state;

    hal_sim_gpio_drive_input(GPIO_PIN_UMBILICAL_SENSE, false);
    assert_int_equal(hal_sim_uart_inject(UART_PORT_DEBUG, PROVISION_SLOT0, sizeof(PROVISION_SLOT0)),
                     SMART_QSO_OK);
    assert_false(gse_poll());
    assert_int_equal(hal_uart_available(UART_PORT_DEBUG), 0U);  /* discarded */
    assert_false(slot0_was_stored());

    assert_int_equal(hal_sim_uart_inject(UART_PORT_DEBUG, PROVISION_SLOT0, sizeof(PROVISION_SLOT0)),
                     SMART_QSO_OK);
    assert_true(gse_poll());
    assert_false(gse_poll());  /* nothing left */
    assert_true(slot0_was_stored());
    assert_true(cmd_auth_is_provisioned());
}

/** GSE ARM re-arms the post-deployment silence (umbilical only) */
static void test_arm_rf_silence(void **state)
{
    (void)state;
    uint8_t frame[GSE_FRAME_SIZE];

    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_SILENCE), SMART_QSO_OK);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    for (uint32_t m = 0U; m < 45U; m++) {
        rf_silence_tick(60U * 1000U);
    }
    assert_false(rf_silence_active());

    make_frame(GSE_OP_ARM_RF_SILENCE, 0U, 0U, NULL, frame);
    hal_sim_gpio_drive_input(GPIO_PIN_UMBILICAL_SENSE, false);
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_LOCKED);
    assert_false(rf_silence_active());

    hal_sim_gpio_drive_input(GPIO_PIN_UMBILICAL_SENSE, true);
    assert_int_equal(gse_process_frame(frame, sizeof(frame)), GSE_STATUS_OK);
    assert_true(rf_silence_active());
    assert_int_equal(rf_silence_get_state(), RF_SILENCE_ARMED);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_reference_frame_provisions_and_activates, setup),
        cmocka_unit_test_setup(test_locked_without_umbilical, setup),
        cmocka_unit_test_setup(test_malformed_frames_rejected, setup),
        cmocka_unit_test_setup(test_bad_operation_and_slot, setup),
        cmocka_unit_test_setup(test_recovery_key_and_zeroize, setup),
        cmocka_unit_test_setup(test_poll_reads_umbilical_uart, setup),
        cmocka_unit_test_setup(test_arm_rf_silence, setup),
    };

    return cmocka_run_group_tests_name("GSE Key Provisioning Tests", tests, NULL, NULL);
}
