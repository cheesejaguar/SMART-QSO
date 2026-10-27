/**
 * @file test_rf_control.c
 * @brief Unit tests for latched TX-OFF and transmitter permission
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 * @requirement SRS-F168 TX-OFF when PTT_EXCITER reads high for more than 6 s with the repeat path not keyed and no downlink frame outstanding; logged; recovered by TX_ENABLE
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "rf_control.h"
#include "beacon.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "hal/hal.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_sim.h"

/*===========================================================================*/
/* Helpers                                                                    */
/*===========================================================================*/

/** Simulate a power cycle: GPIO outputs drop, NVM is retained */
static void power_cycle(void)
{
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
}

static int setup_blank_nvm(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
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

/** Must run first: nothing has initialized rf_control yet in this process */
static void test_not_permitted_before_init(void **state)
{
    (void)state;
    assert_false(rf_control_tx_permitted());
}

static void test_blank_nvm_permits_tx_and_enables_line(void **state)
{
    (void)state;
    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

static void test_tx_off_inhibits_and_drops_line(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

static void test_tx_off_survives_power_cycle(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);

    power_cycle();

    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

static void test_tx_on_clears_inhibit_and_persists(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));

    power_cycle();

    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** Flash failure must not leave the transmitter on */
static void test_tx_off_with_flash_failure_still_silences(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(true);

    assert_int_equal(rf_control_tx_off(), SMART_QSO_ERROR_IO);
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** A corrupt record cannot prove TX was permitted, so stay silent */
static void test_corrupt_record_inhibits(void **state)
{
    (void)state;
    uint8_t junk[8];

    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);

    /* Corrupt the payload of both slots (offset 8 in each 64-byte slot) */
    memset(junk, 0x00, sizeof(junk));
    assert_int_equal(hal_flash_write(FLASH_REGION_RF_CONTROL, 8U, junk, sizeof(junk)), SMART_QSO_OK);
    assert_int_equal(hal_flash_write(FLASH_REGION_RF_CONTROL, 72U, junk, sizeof(junk)), SMART_QSO_OK);

    power_cycle();

    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** SAFE mode re-enables the beacon rail; that must not override TX-OFF */
static void test_safe_mode_does_not_clear_tx_off(void **state)
{
    (void)state;
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);

    assert_int_equal(eps_set_power_mode(POWER_MODE_IDLE, 0.8), SMART_QSO_OK);
    assert_int_equal(eps_set_power_mode(POWER_MODE_SAFE, 0.1), SMART_QSO_OK);

    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

static void test_beacon_not_due_while_tx_off(void **state)
{
    (void)state;
    assert_int_equal(beacon_init(), SMART_QSO_OK);

    /* Interval long elapsed: due while permitted ... */
    assert_true(beacon_should_transmit(10000000U));

    /* ... but never while TX-OFF is latched */
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_false(beacon_should_transmit(10000000U));
}

static void test_beacon_transmit_refused_while_tx_off(void **state)
{
    (void)state;
    BeaconContent_t content;

    assert_int_equal(beacon_init(), SMART_QSO_OK);
    memset(&content, 0, sizeof(content));
    (void)strncpy(content.text, "TEST", sizeof(content.text) - 1U);
    content.text_len = 4U;

    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_int_not_equal(beacon_transmit(&content), SMART_QSO_OK);
}

/** Transmit power level: default 0.5 W, persisted, range-checked, and
 *  independent of the TX-OFF latch */
static void test_power_level_persists_and_validates(void **state)
{
    (void)state;
    assert_int_equal(rf_control_get_power_level(), RF_POWER_0W5);

    assert_int_equal(rf_control_set_power_level(RF_POWER_1W), SMART_QSO_OK);
    assert_int_equal(rf_control_get_power_level(), RF_POWER_1W);
    assert_int_equal(rf_control_set_power_level((RfPowerLevel_t)RF_POWER_LEVEL_COUNT),
                     SMART_QSO_ERROR_INVALID);
    assert_int_equal(rf_control_get_power_level(), RF_POWER_1W);

    /* Setting power never enables a latched-off transmitter */
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_int_equal(rf_control_set_power_level(RF_POWER_0W5), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());

    assert_int_equal(rf_control_init(), SMART_QSO_OK);  /* power cycle */
    assert_int_equal(rf_control_get_power_level(), RF_POWER_0W5);
    assert_false(rf_control_tx_permitted());
}

/** The newest fault log entry has this description */
static void assert_last_fault(const char *description)
{
    FaultLogEntry_t entry;
    assert_int_equal(fault_log_get_last(&entry), SMART_QSO_OK);
    assert_string_equal(entry.description, description);
}

/** A record that cannot be read at boot cannot prove TX was permitted:
 *  inhibited, and reported */
static void test_unreadable_record_inhibits(void **state)
{
    (void)state;
    assert_int_equal(hal_deinit(), SMART_QSO_OK);   /* flash not readable */
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());
    assert_last_fault("RF state unreadable: TX inhibited");

    assert_int_equal(hal_init(), SMART_QSO_OK);     /* the simulation again, flash blank */
}

/** TX enable that cannot be persisted applies until the next reset, and is
 *  reported; the latched TX-OFF in NVM comes back at boot */
static void test_tx_on_not_persisted(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    hal_sim_flash_fail_writes(true);

    assert_int_equal(rf_control_tx_on(), SMART_QSO_ERROR_IO);
    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_last_fault("TX enabled, not persisted");

    hal_sim_flash_fail_writes(false);
    power_cycle();
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** AV-07: the RAM copy is checked against the record, toward TX-OFF */
static void test_verify_toward_tx_off(void **state)
{
    (void)state;
    rf_control_verify(); /* blank record, permitted: agree */
    assert_true(rf_control_tx_permitted());
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    hal_sim_flash_fail_writes(true);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_ERROR_IO); /* RAM on, record off */
    hal_sim_flash_fail_writes(false);
    assert_true(rf_control_tx_permitted());
    rf_control_verify();
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_last_fault("RF state check: RAM copy disagreed, TX-OFF");
    /* A RAM TX-OFF the record does not hold: persisted */
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    hal_sim_flash_fail_writes(true);
    (void)rf_control_tx_off(); /* applied, not persisted */
    hal_sim_flash_fail_writes(false);
    assert_false(rf_control_tx_permitted());
    rf_control_verify();
    assert_last_fault("RF state check: TX-OFF re-persisted");
    assert_int_equal(rf_control_init(), SMART_QSO_OK); /* a reset keeps it */
    assert_false(rf_control_tx_permitted());
}

/** The receiver cycle removes radio power; nothing lifts passivation */
/** SW-HAZ-014 control C-014-9 (SRS-F168): the exciter keyed by the TNC
 *  alone for more than 6 s latches TX-OFF and logs it, once per episode; a
 *  frame on the air or the repeat path explains the keying; TX_ENABLE
 *  recovers, and a TNC still stuck trips it again */
static void test_exciter_keyed_alone_latches_tx_off(void **state)
{
    (void)state;
    const uint32_t trips = rf_control_ptt_trips();
    assert_true(rf_control_tx_permitted());
    hal_sim_gpio_drive_input(GPIO_PIN_XPDR_KEY, false);
    hal_sim_gpio_drive_input(GPIO_PIN_PTT_EXC, true);
    /* A frame's window, then the repeat path: explained */
    assert_false(rf_control_ptt_supervise(1000U, true));
    hal_sim_gpio_drive_input(GPIO_PIN_XPDR_KEY, true);
    assert_false(rf_control_ptt_supervise(30000U, false));
    hal_sim_gpio_drive_input(GPIO_PIN_XPDR_KEY, false);
    /* Alone: counted from here */
    assert_false(rf_control_ptt_supervise(40000U, false));
    assert_false(rf_control_ptt_supervise(40000U + RF_CONTROL_PTT_LIMIT_MS, false));
    assert_true(rf_control_tx_permitted());
    assert_true(rf_control_ptt_supervise(40001U + RF_CONTROL_PTT_LIMIT_MS, false));
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_last_fault("PTT_EXCITER keyed over 6 s with no frame: TX-OFF (C-014-9)");
    assert_false(rf_control_ptt_supervise(60000U, false)); /* once per episode */
    assert_int_equal(rf_control_ptt_trips(), trips + 1U);
    /* Latched across a power cycle, like any TX-OFF */
    power_cycle();
    assert_false(rf_control_tx_permitted());
    /* TX-OFF drops the line; TX_ENABLE recovers; still stuck: again */
    hal_sim_gpio_drive_input(GPIO_PIN_PTT_EXC, false);
    assert_false(rf_control_ptt_supervise(61000U, false));
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    assert_true(rf_control_tx_permitted());
    hal_sim_gpio_drive_input(GPIO_PIN_PTT_EXC, true);
    assert_false(rf_control_ptt_supervise(62000U, false));
    assert_true(rf_control_ptt_supervise(62001U + RF_CONTROL_PTT_LIMIT_MS, false));
    assert_false(rf_control_tx_permitted());
    assert_int_equal(rf_control_ptt_trips(), trips + 2U);
    hal_sim_gpio_drive_input(GPIO_PIN_PTT_EXC, false);
    assert_false(rf_control_ptt_supervise(70000U, false));
}

static void test_cycle_and_passivation_inhibits(void **state)
{
    (void)state;
    rf_control_set_inhibit(RF_INHIBIT_RX_CYCLE, true);
    assert_false(hal_gpio_get(GPIO_PIN_RADIO_EN));
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    rf_control_set_inhibit(RF_INHIBIT_RX_CYCLE, false);
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN));
    assert_true(rf_control_tx_permitted());
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_not_permitted_before_init),
        cmocka_unit_test_setup_teardown(test_blank_nvm_permits_tx_and_enables_line, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_tx_off_inhibits_and_drops_line, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_tx_off_survives_power_cycle, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_tx_on_clears_inhibit_and_persists, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_tx_off_with_flash_failure_still_silences, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_corrupt_record_inhibits, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_safe_mode_does_not_clear_tx_off, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_beacon_not_due_while_tx_off, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_beacon_transmit_refused_while_tx_off, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_power_level_persists_and_validates, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_unreadable_record_inhibits, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_tx_on_not_persisted, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_verify_toward_tx_off, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_cycle_and_passivation_inhibits, setup_blank_nvm, teardown),
        cmocka_unit_test_setup_teardown(test_exciter_keyed_alone_latches_tx_off, setup_blank_nvm, teardown),
    };

    return cmocka_run_group_tests_name("RF Control (TX-OFF) Tests", tests, NULL, NULL);
}
