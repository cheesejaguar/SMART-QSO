/**
 * @file test_uplink.c
 * @brief End to end: AX.25 frame from the receiver -> command path -> TX-OFF
 *
 * The frame is the Python reference shared with test_ax25.c and the ground
 * tests. Keys are TEST-ONLY patterns.
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "uplink.h"
#include "ax25.h"
#include "beacon.h"
#include "cmd_auth.h"
#include "cmd_handler.h"
#include "rf_control.h"
#include "station_id.h"
#include "system_state.h"
#include "state_machine.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_radio.h"
#include "hal/hal_sim.h"
#include "auth_test_helpers.h"

static const uint8_t UPLINK_TX_DISABLE[] = {
    0xA6, 0xA2, 0xA6, 0x9E, 0x40, 0x40, 0x62, 0x9C, 0x60, 0x86, 0x82, 0x98, 0x98, 0x61,
    0x03, 0xF0, 0xAA, 0x55, 0x32, 0x14, 0x00, 0x00, 0x00, 0x01, 0x9A, 0x83, 0xF5, 0xC7,
    0xB2, 0x1F, 0x75, 0x11, 0xE8, 0x2F, 0x5E, 0x94, 0xB9, 0xDE, 0x58, 0x51, 0x09, 0xBF,
    0x65, 0x59
};

static int setup(void **state)
{
    (void)state;
    uint8_t key[CMD_AUTH_KEY_SIZE];

    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_AUTH), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(sm_force_state(sys_get_sm_context(), STATE_IDLE), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(beacon_init(), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    test_key_fill(key, 0x00U);
    assert_int_equal(cmd_auth_provision_key(key, 1U), SMART_QSO_OK);
    assert_int_equal(uplink_init(SQSO_CALLSIGN, SQSO_SSID), SMART_QSO_OK);
    hal_sim_radio_clear();
    return 0;
}

/** The receiver delivers the frame; the station is silenced */
static void test_received_tx_disable_silences_station(void **state)
{
    (void)state;
    CmdResponse_t response;
    UplinkStats_t stats;

    assert_true(rf_control_tx_permitted());
    assert_int_equal(hal_sim_radio_inject(UPLINK_TX_DISABLE, sizeof(UPLINK_TX_DISABLE)), SMART_QSO_OK);

    assert_true(uplink_poll(&response));
    assert_int_equal(response.result, CMD_RESULT_SUCCESS);
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    BeaconContent_t content;
    memset(&content, 0, sizeof(content));
    (void)strncpy(content.text, "test", sizeof(content.text) - 1U);
    assert_int_equal(beacon_transmit(&content), SMART_QSO_ERROR);

    uplink_get_stats(&stats);
    assert_int_equal(stats.frames, 1U);
    assert_int_equal(stats.commands, 1U);
    assert_false(uplink_poll(&response));  /* nothing left */
}

/** Frames for another station, corrupted frames and non-UI frames are dropped */
static void test_other_frames_dropped(void **state)
{
    (void)state;
    uint8_t frame[sizeof(UPLINK_TX_DISABLE)];
    CmdResponse_t response;
    UplinkStats_t stats;

    /* Addressed to SQSO-2 */
    memcpy(frame, UPLINK_TX_DISABLE, sizeof(frame));
    frame[6] = 0x64U;
    uint16_t fcs = ax25_fcs(frame, sizeof(frame) - 2U);
    frame[sizeof(frame) - 2U] = (uint8_t)fcs;
    frame[sizeof(frame) - 1U] = (uint8_t)(fcs >> 8);
    assert_int_equal(hal_sim_radio_inject(frame, sizeof(frame)), SMART_QSO_OK);
    assert_false(uplink_poll(&response));

    /* Bit error */
    memcpy(frame, UPLINK_TX_DISABLE, sizeof(frame));
    frame[30] ^= 0x10U;
    assert_int_equal(hal_sim_radio_inject(frame, sizeof(frame)), SMART_QSO_OK);
    assert_false(uplink_poll(&response));

    uplink_get_stats(&stats);
    assert_int_equal(stats.frames, 2U);
    assert_int_equal(stats.not_for_us, 1U);
    assert_int_equal(stats.rejected, 1U);
    assert_int_equal(stats.commands, 0U);
    assert_true(rf_control_tx_permitted());
}

/** An authenticated frame whose command is refused still reports the result */
static void test_refused_command_reported(void **state)
{
    (void)state;
    CmdResponse_t response;

    assert_int_equal(hal_sim_radio_inject(UPLINK_TX_DISABLE, sizeof(UPLINK_TX_DISABLE)), SMART_QSO_OK);
    assert_true(uplink_poll(&response));
    /* Replay of the same frame */
    assert_int_equal(hal_sim_radio_inject(UPLINK_TX_DISABLE, sizeof(UPLINK_TX_DISABLE)), SMART_QSO_OK);
    assert_true(uplink_poll(&response));
    assert_int_equal(response.result, CMD_RESULT_AUTH_FAIL);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_received_tx_disable_silences_station, setup),
        cmocka_unit_test_setup(test_other_frames_dropped, setup),
        cmocka_unit_test_setup(test_refused_command_reported, setup),
    };

    return cmocka_run_group_tests_name("Uplink Receive Path Tests", tests, NULL, NULL);
}
