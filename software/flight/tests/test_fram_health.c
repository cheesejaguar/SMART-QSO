/**
 * @file test_fram_health.c
 * @brief A dead FRAM: the safe defaults, the declaration after three failed
 *        checks, FRAM_BYPASS and the degraded mode, recovery (RID SMA-B10)
 *
 * @requirement SRS-F160 FRAM failed after three consecutive failed one-minute checks; FRAM_BYPASS then makes the CRC-protected RAM copies of the critical state authoritative until the FRAM reads back again or the next reset
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <setjmp.h>
#include <cmocka.h>

#include "fram_health.h"
#include "ops_state.h"
#include "rf_control.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_sim.h"

static const uint8_t k_code[4] = {'F', 'R', 'M', '!'};

static int setup(void **state)
{
    (void)state;
    hal_sim_flash_fail_reads(false);
    hal_sim_flash_fail_mirror_reads(false);
    hal_sim_flash_fail_writes(false);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_RF_CONTROL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_OPS), SMART_QSO_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    fram_health_init();
    return 0;
}

static int teardown(void **state)
{
    (void)state;
    hal_sim_flash_fail_reads(false);
    hal_sim_flash_fail_mirror_reads(false);
    return 0;
}

static uint8_t fr_state(void)
{
    uint8_t st = 0xFFU;
    uint32_t failures = 0U;
    assert_int_equal(fram_health_status(&st, &failures), SMART_QSO_OK);
    return st;
}

/** Both FRAMs dead (or the shared SPI line stuck) */
static void dead(bool fail)
{
    hal_sim_flash_fail_reads(fail);
    hal_sim_flash_fail_mirror_reads(fail);
}

static void fail_until_declared(void)
{
    dead(true);
    for (uint32_t i = 0U; i < (FRAM_FAILURES_TO_DECLARE - 1U); i++) {
        fram_health_check();
        assert_int_equal(fr_state(), FRAM_STATE_OK);
    }
    fram_health_check();
    assert_int_equal(fr_state(), FRAM_STATE_FAILED);
}

/** The safe default stays: an unreadable latch forces TX-OFF, TX_ENABLE
 *  lasts one check, the hold returns; three failed checks declare it */
static void test_dead_fram_keeps_the_safe_default(void **state)
{
    (void)state;
    assert_true(rf_control_tx_permitted());
    assert_int_equal(fram_bypass(k_code, sizeof(k_code)), SMART_QSO_ERROR);  /* not failed */
    fail_until_declared();
    assert_false(rf_control_tx_permitted());
    assert_true(ops_commissioning_hold());
    assert_int_equal(rf_control_tx_on(), SMART_QSO_ERROR_IO);  /* applied, not persisted */
    assert_true(rf_control_tx_permitted());
    fram_health_check();
    assert_false(rf_control_tx_permitted());                /* forced off again */
    uint32_t failures = 0U;
    assert_int_equal(fram_health_status(NULL, &failures), SMART_QSO_OK);
    assert_int_equal(failures, 4U);
    assert_int_equal(fram_health_status(NULL, NULL), SMART_QSO_OK);
}

/** FRAM_BYPASS: the RAM copies hold; TX-OFF still latches; a reset
 *  returns the defaults */
static void test_bypass_runs_from_ram(void **state)
{
    (void)state;
    fail_until_declared();
    const uint8_t wrong[4] = {'F', 'R', 'M', '?'};
    assert_int_equal(fram_bypass(wrong, sizeof(wrong)), SMART_QSO_ERROR_INVALID);
    assert_int_equal(fram_bypass(k_code, 3U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(fram_bypass(k_code, sizeof(k_code)), SMART_QSO_OK);
    assert_int_equal(fr_state(), FRAM_STATE_DEGRADED);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    assert_int_equal(ops_set_commissioning_hold(false), SMART_QSO_OK);
    for (int i = 0; i < 10; i++) {
        fram_health_check();
    }
    assert_true(rf_control_tx_permitted());
    assert_false(ops_commissioning_hold());
    assert_int_equal(fr_state(), FRAM_STATE_DEGRADED);
    (void)rf_control_tx_off();
    fram_health_check();
    assert_false(rf_control_tx_permitted());
    /* A reset: the defaults, not the RAM copies */
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_int_equal(ops_state_init(), SMART_QSO_OK);
    fram_health_init();
    assert_false(rf_control_tx_permitted());
    assert_true(ops_commissioning_hold());
    assert_int_equal(fr_state(), FRAM_STATE_OK);
}

/** The FRAM reading back again ends the degraded mode; it then holds the
 *  RAM state */
static void test_recovery_ends_the_mode(void **state)
{
    (void)state;
    fail_until_declared();
    assert_int_equal(fram_bypass(k_code, sizeof(k_code)), SMART_QSO_OK);
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK);
    dead(false);
    for (uint32_t i = 0U; i < (FRAM_READS_TO_RECOVER - 1U); i++) {
        fram_health_check();
        assert_int_equal(fr_state(), FRAM_STATE_DEGRADED);
    }
    fram_health_check();
    assert_int_equal(fr_state(), FRAM_STATE_OK);
    fram_health_check();
    assert_true(rf_control_tx_permitted());
    assert_int_equal(rf_control_init(), SMART_QSO_OK);   /* the record holds TX-ON */
    assert_true(rf_control_tx_permitted());
}

/** The second FRAM (U36): with the first one dead or upset, the records
 *  come from the mirror and nothing degrades */
static void test_the_mirror_serves(void **state)
{
    (void)state;
    assert_int_equal(rf_control_tx_off(), SMART_QSO_OK);
    hal_sim_flash_fail_reads(true);
    for (int i = 0; i < 5; i++) {
        fram_health_check();
    }
    assert_int_equal(fr_state(), FRAM_STATE_OK);
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_false(rf_control_tx_permitted());          /* TX-OFF from the mirror */
    assert_int_equal(rf_control_tx_on(), SMART_QSO_OK); /* saved through the mirror */
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_true(rf_control_tx_permitted());
    hal_sim_flash_fail_reads(false);
    /* An upset of the first FRAM's copies only */
    for (uint32_t off = 0U; off < 64U; off++) {
        hal_sim_flash_corrupt_primary(FLASH_REGION_RF_CONTROL, off);
    }
    assert_int_equal(rf_control_init(), SMART_QSO_OK);
    assert_true(rf_control_tx_permitted());
    fram_health_check();
    assert_int_equal(fr_state(), FRAM_STATE_OK);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup_teardown(test_dead_fram_keeps_the_safe_default, setup, teardown),
        cmocka_unit_test_setup_teardown(test_bypass_runs_from_ram, setup, teardown),
        cmocka_unit_test_setup_teardown(test_recovery_ends_the_mode, setup, teardown),
        cmocka_unit_test_setup_teardown(test_the_mirror_serves, setup, teardown),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
