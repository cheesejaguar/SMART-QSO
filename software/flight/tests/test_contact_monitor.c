/**
 * @file test_contact_monitor.c
 * @brief Loss-of-contact transmit inhibit
 *
 * @requirement SYS-205 Cease transmissions after N days without an authenticated telecommand
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "contact_monitor.h"
#include "rf_control.h"
#include "beacon.h"
#include "nvm_record.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_sim.h"

#define DAY_MS      (86400U * 1000U)

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_CONTACT), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    return 0;
}

static void advance_days(uint32_t days)
{
    for (uint32_t d = 0U; d < days; d++) {
        contact_monitor_tick(DAY_MS);
    }
}

static void test_first_boot_starts_at_zero(void **state)
{
    (void)state;
    assert_int_equal(contact_monitor_seconds_since_contact(), 0U);
    assert_int_equal(contact_monitor_get_timeout_days(), CONTACT_TIMEOUT_DEFAULT_DAYS);
    assert_false(contact_monitor_inhibited());
    assert_true(rf_control_tx_permitted());
}

/** Decision D3: 30 days at launch, leaving slack for a late first contact;
 *  operations set 14 days by SET_CONTACT_TIMEOUT after commissioning */
static void test_d3_launch_default(void **state)
{
    (void)state;
    assert_int_equal(contact_monitor_get_timeout_days(), 30U);
    advance_days(29U);
    assert_false(contact_monitor_inhibited());
    assert_int_equal(contact_monitor_set_timeout_days(14U), SMART_QSO_OK);
    assert_true(contact_monitor_inhibited());  /* already past 14 days */
}

/** After N days without contact every transmitter is off and the pin is low */
static void test_inhibits_after_timeout(void **state)
{
    (void)state;
    BeaconContent_t content;

    advance_days(CONTACT_TIMEOUT_DEFAULT_DAYS - 1U);
    contact_monitor_tick(DAY_MS - 1000U);
    assert_false(contact_monitor_inhibited());
    assert_true(rf_control_tx_permitted());

    contact_monitor_tick(1000U);
    assert_true(contact_monitor_inhibited());
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));

    assert_int_equal(beacon_init(), SMART_QSO_OK);
    memset(&content, 0, sizeof(content));
    (void)strncpy(content.text, "TEST", sizeof(content.text) - 1U);
    assert_int_equal(beacon_transmit(&content), SMART_QSO_ERROR);
}

/** An authenticated command restores transmission and restarts the count */
static void test_contact_clears_inhibit(void **state)
{
    (void)state;
    advance_days(CONTACT_TIMEOUT_DEFAULT_DAYS);
    assert_true(contact_monitor_inhibited());

    contact_monitor_contact();
    assert_false(contact_monitor_inhibited());
    assert_int_equal(contact_monitor_seconds_since_contact(), 0U);
    assert_true(rf_control_tx_permitted());
    assert_true(hal_gpio_get(GPIO_PIN_RF_TX_EN));
}

/** The two inhibits are independent: contact never clears a TX-OFF latch,
 *  and clearing the latch does not lift a loss-of-contact inhibit */
static void test_independent_of_tx_off_latch(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    contact_monitor_contact();
    assert_false(rf_control_tx_permitted());

    advance_days(CONTACT_TIMEOUT_DEFAULT_DAYS);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));

    contact_monitor_contact();
    assert_true(rf_control_tx_permitted());
}

/** Elapsed time survives a reset; each boot also adds the persist interval */
static void test_count_survives_reset(void **state)
{
    (void)state;
    advance_days(3U);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);  /* simulated reset */
    assert_int_equal(contact_monitor_seconds_since_contact(),
                     (3U * 86400U) + CONTACT_PERSIST_INTERVAL_S);
}

/** A reset loop faster than the persist interval still reaches the inhibit */
static void test_reset_loop_still_inhibits(void **state)
{
    (void)state;
    assert_int_equal(contact_monitor_set_timeout_days(1U), SMART_QSO_OK);

    uint32_t boots = (86400U / CONTACT_PERSIST_INTERVAL_S) + 1U;
    for (uint32_t i = 0U; i < boots; i++) {
        assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    }
    assert_true(contact_monitor_inhibited());
    assert_false(rf_control_tx_permitted());
}

/** Inhibit state is derived from the counter at boot */
static void test_inhibit_restored_at_boot(void **state)
{
    (void)state;
    advance_days(CONTACT_TIMEOUT_DEFAULT_DAYS);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_true(contact_monitor_inhibited());
    assert_false(rf_control_tx_permitted());
}

/** A corrupt counter record inhibits until contact (fail safe) */
static void test_corrupt_counter_inhibits(void **state)
{
    (void)state;
    uint8_t bytes[4];

    contact_monitor_tick(120U * 1000U);
    for (uint32_t slot = 0U; slot < 2U; slot++) {
        uint32_t offset = CONTACT_NVM_COUNTER + (slot * NVM_RECORD_SLOT_SIZE) + 8U;
        assert_int_equal(hal_flash_read(FLASH_REGION_CONTACT, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
        for (size_t i = 0U; i < sizeof(bytes); i++) {
            bytes[i] ^= 0xFFU;
        }
        assert_int_equal(hal_flash_write(FLASH_REGION_CONTACT, offset, bytes, sizeof(bytes)), SMART_QSO_OK);
    }

    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_true(contact_monitor_inhibited());
    contact_monitor_contact();
    assert_false(contact_monitor_inhibited());
}

static void test_timeout_bounds_and_persistence(void **state)
{
    (void)state;
    assert_int_equal(contact_monitor_set_timeout_days(0U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(contact_monitor_set_timeout_days(CONTACT_TIMEOUT_MAX_DAYS + 1U),
                     SMART_QSO_ERROR_INVALID);
    assert_int_equal(contact_monitor_set_timeout_days(30U), SMART_QSO_OK);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_get_timeout_days(), 30U);

    /* Shortening the timeout below the elapsed time inhibits at once */
    advance_days(5U);
    assert_int_equal(contact_monitor_set_timeout_days(4U), SMART_QSO_OK);
    assert_true(contact_monitor_inhibited());
}

/** Persistence is rate-limited: a short tick is not written, and the boot
 *  penalty covers whatever was lost */
static void test_persists_at_interval(void **state)
{
    (void)state;
    contact_monitor_tick(1000U);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    /* Record written at first boot (0); the 1 s was lost; penalty added */
    assert_int_equal(contact_monitor_seconds_since_contact(), CONTACT_PERSIST_INTERVAL_S);

    contact_monitor_tick(CONTACT_PERSIST_INTERVAL_S * 1000U);
    assert_int_equal(contact_monitor_init(), SMART_QSO_OK);
    assert_int_equal(contact_monitor_seconds_since_contact(), 3U * CONTACT_PERSIST_INTERVAL_S);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_first_boot_starts_at_zero, setup),
        cmocka_unit_test_setup(test_d3_launch_default, setup),
        cmocka_unit_test_setup(test_inhibits_after_timeout, setup),
        cmocka_unit_test_setup(test_contact_clears_inhibit, setup),
        cmocka_unit_test_setup(test_independent_of_tx_off_latch, setup),
        cmocka_unit_test_setup(test_count_survives_reset, setup),
        cmocka_unit_test_setup(test_reset_loop_still_inhibits, setup),
        cmocka_unit_test_setup(test_inhibit_restored_at_boot, setup),
        cmocka_unit_test_setup(test_corrupt_counter_inhibits, setup),
        cmocka_unit_test_setup(test_timeout_bounds_and_persistence, setup),
        cmocka_unit_test_setup(test_persists_at_interval, setup),
    };

    return cmocka_run_group_tests_name("Loss-of-Contact Inhibit Tests", tests, NULL, NULL);
}
