/**
 * @file test_xpdr_heartbeat.c
 * @brief The XPDR_EN heartbeat (hal_gpio_heartbeat) on the simulated HAL
 *
 * The expectations come from the design, not from the code:
 * docs/hardware/BUILD_VS_BUY.md §3.2 (XPDR_EN is a ≈ 100 Hz heartbeat into a
 * ≈ 50 ms retriggerable monostable, so a hung, reset or browned-out OBC drops
 * the transponder), hardware/rf/TRANSMITTER_DESIGN.md §10.1 and hal_gpio.h:
 *
 * - the wave runs for the lease from the latest call, then stops low;
 * - a renewal before the lease ends keeps it running; 0 stops it now;
 * - only GPIO_PIN_XPDR_EN has a heartbeat; nothing runs before hal_gpio_init()
 *   or after it (a reset stops the wave);
 * - a level written with hal_gpio_set() is not a heartbeat.
 *
 * @requirement SRS-F081 The software shall drive the transponder enable XPDR_EN as a heartbeat renewed by every main-loop pass while the transponder gates hold
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>

#include "hal/hal.h"
#include "hal/hal_gpio.h"
#include "hal/hal_sim.h"
#include "hal/hal_timer.h"

/** Half period of the heartbeat, ms */
#define HALF_PERIOD_MS (1000U / (2U * HAL_GPIO_HEARTBEAT_HZ))

static int setup(void **state)
{
    (void)state;
    assert_int_equal(hal_timer_init(), SMART_QSO_OK);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);
    assert_int_equal(hal_gpio_config(GPIO_PIN_XPDR_EN, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN), SMART_QSO_OK);
    return 0;
}

/** Microseconds from now to the end of the heartbeat (negative: ended) */
static int64_t left_us(void)
{
    return (int64_t)hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN) - (int64_t)hal_timer_get_us();
}

/** Edges seen by sampling the pin every millisecond for ms */
static uint32_t edges_over(uint32_t ms)
{
    uint32_t edges = 0U;
    bool last = hal_gpio_get(GPIO_PIN_XPDR_EN);
    for (uint32_t i = 0U; i < ms; i++) {
        smart_qso_sim_advance_ms(1U);
        bool now = hal_gpio_get(GPIO_PIN_XPDR_EN);
        if (now != last) {
            edges++;
        }
        last = now;
    }
    return edges;
}

/** No heartbeat at boot: the pin is low and nothing has run */
static void test_no_heartbeat_at_boot(void **state)
{
    (void)state;
    assert_false(hal_gpio_get(GPIO_PIN_XPDR_EN));
    assert_int_equal(hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN), 0U);
    assert_int_equal(edges_over(50U), 0U);
}

/** The wave toggles at the heartbeat rate for the lease, then stops low */
static void test_wave_runs_for_the_lease_then_stops_low(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 60U), SMART_QSO_OK);
    int64_t left = left_us();
    assert_true(left > 59000 && left <= 60000);
    /* 40 ms: about 8 edges at 100 Hz (one every 5 ms) */
    uint32_t edges = edges_over(40U);
    assert_true(edges >= (40U / HALF_PERIOD_MS) - 1U && edges <= (40U / HALF_PERIOD_MS) + 1U);
    /* Past the lease: no more edges, pin low */
    (void)edges_over(30U);
    assert_false(hal_gpio_get(GPIO_PIN_XPDR_EN));
    assert_int_equal(edges_over(100U), 0U);
    assert_true(left_us() < 0);
}

/** Renewed on every 20 ms pass, the wave never stops */
static void test_renewal_keeps_it_running(void **state)
{
    (void)state;
    uint32_t edges = 0U;
    for (uint32_t pass = 0U; pass < 50U; pass++) {
        assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 60U), SMART_QSO_OK);
        edges += edges_over(20U);
        assert_true(left_us() > 0);
    }
    /* One second of heartbeat: about 200 edges */
    assert_true(edges >= 195U && edges <= 201U);
    /* The loop stalls: the wave stops by itself within the lease */
    (void)edges_over(60U);
    assert_int_equal(edges_over(100U), 0U);
    assert_false(hal_gpio_get(GPIO_PIN_XPDR_EN));
}

/** A zero lease stops the wave at once */
static void test_zero_lease_stops_now(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 60U), SMART_QSO_OK);
    (void)edges_over(12U);
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 0U), SMART_QSO_OK);
    assert_true(left_us() <= 0);
    assert_false(hal_gpio_get(GPIO_PIN_XPDR_EN));
    assert_int_equal(edges_over(50U), 0U);
    /* Stopping a stopped heartbeat changes nothing */
    uint64_t end = hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN);
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 0U), SMART_QSO_OK);
    assert_int_equal(hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN), end);
}

/** A level is not a heartbeat; a reset stops the wave */
static void test_level_is_not_a_heartbeat_and_reset_stops_it(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_set(GPIO_PIN_XPDR_EN, true), SMART_QSO_OK);
    assert_true(hal_gpio_get(GPIO_PIN_XPDR_EN));
    assert_int_equal(hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN), 0U);
    assert_int_equal(edges_over(50U), 0U);

    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 60U), SMART_QSO_OK);
    assert_true(hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN) > 0U);
    assert_int_equal(hal_gpio_init(), SMART_QSO_OK);  /* a reset */
    assert_int_equal(hal_sim_gpio_heartbeat_end_us(GPIO_PIN_XPDR_EN), 0U);
    assert_false(hal_gpio_get(GPIO_PIN_XPDR_EN));
}

/** Only XPDR_EN has a heartbeat; nothing before hal_gpio_init() */
static void test_other_pins_and_before_init(void **state)
{
    (void)state;
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_RF_TX_EN, 60U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_COUNT, 60U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(hal_sim_gpio_heartbeat_end_us(GPIO_PIN_RF_TX_EN), 0U);
    assert_false(hal_gpio_get(GPIO_PIN_RF_TX_EN));
    assert_int_equal(hal_deinit(), SMART_QSO_OK);
    assert_int_equal(hal_gpio_heartbeat(GPIO_PIN_XPDR_EN, 60U), SMART_QSO_ERROR);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_no_heartbeat_at_boot, setup),
        cmocka_unit_test_setup(test_wave_runs_for_the_lease_then_stops_low, setup),
        cmocka_unit_test_setup(test_renewal_keeps_it_running, setup),
        cmocka_unit_test_setup(test_zero_lease_stops_now, setup),
        cmocka_unit_test_setup(test_level_is_not_a_heartbeat_and_reset_stops_it, setup),
        cmocka_unit_test_setup(test_other_pins_and_before_init, setup),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
