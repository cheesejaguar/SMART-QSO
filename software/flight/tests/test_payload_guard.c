/**
 * @file test_payload_guard.c
 * @brief OBC-enforced Jetson power and temperature limits (SW-16)
 *
 * The OBC cuts the payload rail on its own measurements, independent of the
 * Jetson: input power above 8 W (BASELINE §6), a single sample in latch-up
 * range, heat spreader at 70 C, Jetson-reported junction at 95 C
 * (hardware/payload-jetson/POWER_MANAGEMENT.md §5.1), or no measurements
 * while the rail is on. After a trip the rail stays off for a cool-down and
 * until the spreader has cooled.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "payload_guard.h"
#include "eps_control.h"

#define T0  1000000ULL

static PayloadSample_t sample(double watts, double spreader_c)
{
    PayloadSample_t s;
    (void)memset(&s, 0, sizeof(s));
    s.power_valid = true;
    s.input_w = watts;
    s.spreader_valid = true;
    s.spreader_c = spreader_c;
    return s;
}

static int setup(void **state)
{
    (void)state;
    payload_guard_init();
    return 0;
}

static void test_within_limits(void **state)
{
    (void)state;
    PayloadSample_t s = sample(6.5, 45.0);
    for (uint64_t t = 0U; t < 60000U; t += 1000U) {
        assert_int_equal(payload_guard_step(&s, true, T0 + t), PG_OK);
    }
    assert_true(payload_guard_may_enable(T0 + 60000U));
    assert_int_equal(payload_guard_trip_count(), 0U);
}

/** Above 8 W twice in a row trips; one sample is tolerated */
static void test_overpower(void **state)
{
    (void)state;
    PayloadSample_t hot = sample(8.5, 45.0);
    PayloadSample_t ok = sample(7.9, 45.0);

    assert_int_equal(payload_guard_step(&hot, true, T0), PG_OK);
    assert_int_equal(payload_guard_step(&ok, true, T0 + 1000U), PG_OK);
    assert_int_equal(payload_guard_step(&hot, true, T0 + 2000U), PG_OK);
    assert_int_equal(payload_guard_step(&hot, true, T0 + 3000U), PG_TRIP_OVERPOWER);
    assert_int_equal(payload_guard_trip_count(), 1U);
    assert_false(payload_guard_may_enable(T0 + 3000U));
}

/** A single sample in latch-up range trips at once */
static void test_latchup(void **state)
{
    (void)state;
    PayloadSample_t s = sample(PG_LATCHUP_INPUT_W, 45.0);
    assert_int_equal(payload_guard_step(&s, true, T0), PG_TRIP_LATCHUP);
}

static void test_temperatures(void **state)
{
    (void)state;
    PayloadSample_t s = sample(6.0, PG_SPREADER_MAX_C - 0.1);
    assert_int_equal(payload_guard_step(&s, true, T0), PG_OK);
    s.spreader_c = PG_SPREADER_MAX_C;
    assert_int_equal(payload_guard_step(&s, true, T0 + 1000U), PG_TRIP_SPREADER_TEMP);

    payload_guard_init();
    s = sample(6.0, 50.0);
    s.junction_valid = true;
    s.junction_c = PG_JUNCTION_MAX_C - 0.1;
    assert_int_equal(payload_guard_step(&s, true, T0), PG_OK);
    s.junction_c = PG_JUNCTION_MAX_C;
    assert_int_equal(payload_guard_step(&s, true, T0 + 1000U), PG_TRIP_JUNCTION_TEMP);

    /* A junction value the Jetson did not report is not used */
    payload_guard_init();
    s.junction_valid = false;
    assert_int_equal(payload_guard_step(&s, true, T0), PG_OK);
}

/** The rail is not left on without power and spreader measurements */
static void test_no_telemetry(void **state)
{
    (void)state;
    PayloadSample_t none;
    (void)memset(&none, 0, sizeof(none));
    PayloadSample_t no_temp = sample(6.0, 45.0);
    no_temp.spreader_valid = false;

    /* Grace starts when the rail turns on */
    assert_int_equal(payload_guard_step(&none, false, T0), PG_OK);
    assert_int_equal(payload_guard_step(&none, true, T0 + 100000U), PG_OK);
    assert_int_equal(payload_guard_step(&none, true, T0 + 100000U + PG_TELEMETRY_TIMEOUT_MS), PG_OK);
    assert_int_equal(payload_guard_step(&none, true, T0 + 100001U + PG_TELEMETRY_TIMEOUT_MS),
                     PG_TRIP_NO_TELEMETRY);

    /* Power alone is not enough */
    payload_guard_init();
    assert_int_equal(payload_guard_step(&no_temp, true, T0), PG_OK);
    assert_int_equal(payload_guard_step(&no_temp, true, T0 + PG_TELEMETRY_TIMEOUT_MS + 1U),
                     PG_TRIP_NO_TELEMETRY);
}

/** After a trip: off for the cool-down, then only once the spreader has cooled */
static void test_cooldown(void **state)
{
    (void)state;
    PayloadSample_t s = sample(PG_LATCHUP_INPUT_W, 50.0);
    assert_int_equal(payload_guard_step(&s, true, T0), PG_TRIP_LATCHUP);

    PayloadSample_t warm = sample(0.0, PG_SPREADER_MAX_C - PG_RESTART_MARGIN_C);
    PayloadSample_t cool = sample(0.0, PG_SPREADER_MAX_C - PG_RESTART_MARGIN_C - 0.1);

    /* Rail still on during the cool-down: it must go off */
    assert_int_equal(payload_guard_step(&cool, true, T0 + 1000U), PG_COOLDOWN);
    assert_int_equal(payload_guard_step(&cool, false, T0 + PG_COOLDOWN_MS - 1U), PG_OK);
    assert_false(payload_guard_may_enable(T0 + PG_COOLDOWN_MS - 1U));

    /* Cool-down over, spreader not cool enough */
    assert_int_equal(payload_guard_step(&warm, false, T0 + PG_COOLDOWN_MS), PG_OK);
    assert_false(payload_guard_may_enable(T0 + PG_COOLDOWN_MS));
    /* Cooled, but the reading is stale */
    assert_int_equal(payload_guard_step(&cool, false, T0 + PG_COOLDOWN_MS + 1U), PG_OK);
    assert_false(payload_guard_may_enable(T0 + PG_COOLDOWN_MS + 2U + PG_TELEMETRY_TIMEOUT_MS));
    /* Cooled and recent */
    assert_true(payload_guard_may_enable(T0 + PG_COOLDOWN_MS + 1U));
}

/** Before any trip: a hot spreader blocks enabling; no reading does not */
static void test_enable_without_trip(void **state)
{
    (void)state;
    assert_true(payload_guard_may_enable(T0));
    PayloadSample_t hot = sample(0.0, PG_SPREADER_MAX_C - PG_RESTART_MARGIN_C);
    assert_int_equal(payload_guard_step(&hot, false, T0), PG_OK);
    assert_false(payload_guard_may_enable(T0));
}

/** The EPS refuses to power the payload while the guard says no */
static void test_eps_honours_guard(void **state)
{
    (void)state;
    assert_int_equal(eps_init(), SMART_QSO_OK);
    (void)eps_control_payload(false, 0.9);

    PayloadSample_t s = sample(PG_LATCHUP_INPUT_W, 50.0);
    uint64_t now = smart_qso_now_ms();
    assert_int_equal(payload_guard_step(&s, true, now), PG_TRIP_LATCHUP);
    assert_int_equal(eps_control_payload(true, 0.9), SMART_QSO_ERROR);
    assert_false(eps_is_payload_enabled());

    payload_guard_init();
    assert_int_equal(eps_control_payload(true, 0.9), SMART_QSO_OK);
    assert_true(eps_is_payload_enabled());
    (void)eps_control_payload(false, 0.9);
}

static void test_reason_names(void **state)
{
    (void)state;
    assert_string_equal(payload_guard_reason(PG_OK), "OK");
    assert_string_equal(payload_guard_reason(PG_TRIP_LATCHUP), "Payload latch-up current: rail off");
    assert_non_null(payload_guard_reason((PayloadGuardResult_t)99));
    /* NULL counts as no measurements: allowed only within the rail-on grace */
    assert_int_equal(payload_guard_step(NULL, true, T0), PG_OK);
    assert_int_equal(payload_guard_step(NULL, true, T0 + PG_TELEMETRY_TIMEOUT_MS + 1U),
                     PG_TRIP_NO_TELEMETRY);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_within_limits, setup),
        cmocka_unit_test_setup(test_overpower, setup),
        cmocka_unit_test_setup(test_latchup, setup),
        cmocka_unit_test_setup(test_temperatures, setup),
        cmocka_unit_test_setup(test_no_telemetry, setup),
        cmocka_unit_test_setup(test_cooldown, setup),
        cmocka_unit_test_setup(test_enable_without_trip, setup),
        cmocka_unit_test_setup(test_eps_honours_guard, setup),
        cmocka_unit_test_setup(test_reason_names, setup),
    };

    return cmocka_run_group_tests_name("Payload Guard Tests", tests, NULL, NULL);
}
