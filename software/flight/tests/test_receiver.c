/**
 * @file test_receiver.c
 * @brief Command-receiver supervision (FSW-09, REL-04) and the EPS
 *        recharge-lockout clear (EPS-12)
 *
 * Expectations from receiver.h, contact_monitor.h, eps_lockout.h and
 * docs/software/SOFTWARE_REQUIREMENTS.md §3.12:
 *
 * - 24 h without a valid frame or a carrier, the rail on: the radio is
 *   power-cycled for 2 s (RADIO_EN low, RF_TX_EN low first), logged, and on
 *   again; not while keyed; not again for another 24 h of silence;
 * - a carrier restarts the count;
 * - known failed only with every piece of evidence; then the
 *   loss-of-contact inhibit no longer silences the beacon but TX-OFF still
 *   does; a carrier ends it and the inhibit applies again;
 * - EPS_CLEAR_LOCKOUT: wrong code refused; the unlock written over I2C and
 *   LOCK_CLR pulsed for at least 100 ms.
 *
 * @requirement SRS-F115 Hung-receiver recovery by a brief power cycle, never during a pass
 * @requirement SRS-F116 Evidence that the command receiver has failed
 * @requirement SRS-F117 With the command receiver known failed, the loss-of-contact inhibit shall not silence the beacon
 * @requirement SRS-F129 The software shall clear the EPS's latched recharge lockout only on an authenticated command with its code, by an I2C unlock and a LOCK_CLR pulse
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "receiver.h"
#include "rf_control.h"
#include "rf_silence.h"
#include "contact_monitor.h"
#include "eps_lockout.h"
#include "fault_mgmt.h"
#include "hal/hal.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_i2c.h"
#include "hal/hal_sim.h"

#define HOUR_MS 3600000U

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_i2c_init(I2C_BUS_0, 100000U), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_CONTACT), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_SILENCE), SMART_QSO_OK);
    (void)fault_mgmt_init();
    hal_sim_gpio_drive_input(GPIO_PIN_RX_CD, false);
    hal_sim_gpio_drive_input(GPIO_PIN_RX_FAULT, false);
    /* A completed RF silence (a software reset resumes it) */
    hal_sim_set_reset_cause(HAL_RESET_SOFTWARE);
    assert_int_equal(rf_silence_init(), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    receiver_init();
    return 0;
}

/** Advance the clock by `ms` in steps of `step` ms, a pass each */
static void pass_for(uint64_t ms, uint64_t step, bool keyed)
{
    for (uint64_t t = 0U; t < ms; t += step) {
        smart_qso_sim_advance_ms(step);
        receiver_step(keyed);
    }
}

static void serve_silence(void)
{
    rf_silence_tick(46U * 60U * 1000U);
    assert_false(rf_silence_active());
}

static void test_recovery_cycle(void **state)
{
    (void)state;
    serve_silence();
    pass_for(23U * HOUR_MS, HOUR_MS, false);
    assert_int_equal(receiver_cycles(), 0U);
    pass_for(2U * HOUR_MS, HOUR_MS, true); /* keyed: never in a pass */
    assert_int_equal(receiver_cycles(), 0U);
    pass_for(HOUR_MS, HOUR_MS, false);
    assert_int_equal(receiver_cycles(), 1U);
    assert_false(hal_gpio_get(GPIO_PIN_RADIO_EN));
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    pass_for(RECEIVER_CYCLE_OFF_MS, 1000U, false);
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN));
    assert_true(rf_control_tx_permitted());
    /* Not again for another day of silence; a carrier restarts the count */
    pass_for(20U * HOUR_MS, HOUR_MS, false);
    assert_int_equal(receiver_cycles(), 1U);
    hal_sim_gpio_drive_input(GPIO_PIN_RX_CD, true);
    pass_for(1000U, 1000U, false);
    hal_sim_gpio_drive_input(GPIO_PIN_RX_CD, false);
    pass_for(23U * HOUR_MS, HOUR_MS, false);
    assert_int_equal(receiver_cycles(), 1U);
}

/** REL-04: the beacon kept for a failed receiver, TX-OFF still effective */
static void test_known_failed_keeps_the_beacon(void **state)
{
    (void)state;
    serve_silence();
    assert_int_equal(contact_monitor_set_timeout_days(2U), SMART_QSO_OK);
    hal_sim_gpio_drive_input(GPIO_PIN_RX_FAULT, true);
    /* Two days and more of silence: one recovery cycle on the first day */
    for (int h = 0; h < 50; h++) {
        smart_qso_sim_advance_ms(HOUR_MS);
        receiver_step(false);
        contact_monitor_tick(HOUR_MS);
        smart_qso_sim_advance_ms(RECEIVER_CYCLE_OFF_MS);
        receiver_step(false);
    }
    assert_true(receiver_cycles() >= 1U);
    assert_true(contact_monitor_inhibited());
    assert_true(receiver_failed());
    contact_monitor_tick(1000U);
    assert_true(rf_control_tx_permitted()); /* the beacon goes on */
    /* TX-OFF still silences it */
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    /* The health line good again: not known failed, the inhibit applies */
    hal_sim_gpio_drive_input(GPIO_PIN_RX_FAULT, false);
    receiver_step(false);
    assert_false(receiver_failed());
    contact_monitor_tick(1000U);
    assert_false(rf_control_tx_permitted());
}

/** RID SMA-B11: a carrier detect stuck high counts as activity for its
 *  first hour only; the recovery cycle comes a day after that */
static void test_stuck_carrier_is_not_activity(void **state)
{
    (void)state;
    serve_silence();
    hal_sim_gpio_drive_input(GPIO_PIN_RX_CD, true);
    pass_for(24U * HOUR_MS, HOUR_MS / 4U, false);
    assert_int_equal(receiver_cycles(), 0U);
    pass_for(2U * HOUR_MS, HOUR_MS / 4U, false);
    assert_int_equal(receiver_cycles(), 1U);
    assert_true(hal_gpio_get(GPIO_PIN_RADIO_EN)); /* the cycle is over */
    hal_sim_gpio_drive_input(GPIO_PIN_RX_CD, false);
}

/** The STATUS loop query every 10 min with the radio on; a TNC that stops
 *  answering for a day is a receiver-path fault: with the contact lost and
 *  a recovery cycle tried, known failed although RX_FAULT reads healthy */
static void test_loop_query_and_the_path_fault(void **state)
{
    (void)state;
    serve_silence();
    uint32_t q0 = hal_sim_tnc_queries();
    pass_for(RECEIVER_QUERY_PERIOD_MS * 3U, 60000U, false);
    assert_true(hal_sim_tnc_queries() - q0 >= 3U);
    assert_int_equal(contact_monitor_set_timeout_days(2U), SMART_QSO_OK);
    hal_sim_tnc_mute(true);
    for (int h = 0; h < 50; h++) {
        smart_qso_sim_advance_ms(HOUR_MS);
        receiver_step(false);
        contact_monitor_tick(HOUR_MS);
        smart_qso_sim_advance_ms(RECEIVER_CYCLE_OFF_MS);
        receiver_step(false);
    }
    assert_true(contact_monitor_inhibited());
    assert_true(receiver_failed());
    /* The TNC answers again: within a query period, not failed */
    hal_sim_tnc_mute(false);
    pass_for(RECEIVER_QUERY_PERIOD_MS + 60000U, 60000U, false);
    assert_false(receiver_failed());
}

/** EPS-12: the code, the I2C unlock and the LOCK_CLR pulse */
static void test_lockout_clear(void **state)
{
    (void)state;
    const uint8_t wrong[4] = {'C', 'H', 'G', '?'};
    const uint8_t code[4] = {'C', 'H', 'G', '!'};
    const uint8_t reg = 0x7DU;
    uint8_t v = 0U;
    eps_lockout_init();
    assert_int_equal(eps_lockout_clear(wrong, sizeof(wrong)), SMART_QSO_ERROR_INVALID);
    assert_int_equal(eps_lockout_clear(NULL, 4U), SMART_QSO_ERROR_NULL_PTR);
    assert_false(eps_lockout_pulsing());
    assert_int_equal(eps_lockout_clear(code, sizeof(code)), SMART_QSO_OK);
    assert_int_equal(hal_i2c_write_read(I2C_BUS_0, 0x48U, &reg, 1U, &v, 1U), SMART_QSO_OK);
    assert_int_equal(v, 0xC5U);
    assert_true(hal_gpio_get(GPIO_PIN_LOCK_CLR));
    smart_qso_sim_advance_ms(100U);
    eps_lockout_poll();
    assert_true(hal_gpio_get(GPIO_PIN_LOCK_CLR)); /* at least 100 ms */
    smart_qso_sim_advance_ms(60U);
    eps_lockout_poll();
    assert_false(hal_gpio_get(GPIO_PIN_LOCK_CLR));
    assert_false(eps_lockout_pulsing());
    /* The line cannot be driven (GPIO released): refused, logged */
    (void)hal_deinit();
    assert_int_equal(eps_lockout_clear(code, sizeof(code)), SMART_QSO_ERROR_IO);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_recovery_cycle, setup),
        cmocka_unit_test_setup(test_known_failed_keeps_the_beacon, setup),
        cmocka_unit_test_setup(test_stuck_carrier_is_not_activity, setup),
        cmocka_unit_test_setup(test_loop_query_and_the_path_fault, setup),
        cmocka_unit_test_setup(test_lockout_clear, setup),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
