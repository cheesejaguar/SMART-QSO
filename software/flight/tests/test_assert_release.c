/**
 * @file test_assert_release.c
 * @brief Assertions keep working in flight builds (SW-08)
 *
 * This file is compiled as a flight (non-debug) translation unit. A failed
 * SMART_QSO_REQUIRE used to compile to an empty body there, so the function
 * carried on with the bad value. Now every build reports the failure to the
 * assertion handler (logged and counted) and returns an error.
 */

/* Compile this unit as the flight build does */
#undef SMART_QSO_DEBUG_ENABLED
#define SMART_QSO_DEBUG_ENABLED 0

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>

#include "smart_qso.h"
#include "assert_handler.h"
#include "eps_control.h"

static SmartQsoResult_t needs_positive(int x)
{
    SMART_QSO_REQUIRE(x > 0, "x must be positive");
    return SMART_QSO_OK;
}

static SmartQsoResult_t needs_pointer(const int *p)
{
    SMART_QSO_REQUIRE_NOT_NULL(p);
    return SMART_QSO_OK;
}

static size_t length_or_zero(const char *text, size_t cap)
{
    SMART_QSO_REQUIRE_NOT_NULL_OR(text, 0U);
    SMART_QSO_REQUIRE_OR(cap > 0U, "capacity", 0U);
    return 7U;
}

static uint32_t failures(void)
{
    AssertStats_t stats;
    assert_int_equal(assert_get_stats(&stats), SMART_QSO_OK);
    return stats.total_failures;
}

static int setup(void **state)
{
    (void)state;
    assert_int_equal(assert_init(), SMART_QSO_OK);
    return 0;
}

static void test_pass_is_silent(void **state)
{
    (void)state;
    int value = 1;
    assert_int_equal(needs_positive(1), SMART_QSO_OK);
    assert_int_equal(needs_pointer(&value), SMART_QSO_OK);
    assert_int_equal(length_or_zero("x", 4U), 7U);
    assert_int_equal(failures(), 0U);
}

static void test_failure_reported_and_returns_error(void **state)
{
    (void)state;
    assert_int_equal(needs_positive(0), SMART_QSO_ERROR_INVALID);
    assert_int_equal(failures(), 1U);
    assert_int_equal(needs_pointer(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(failures(), 2U);
    assert_int_equal(length_or_zero(NULL, 4U), 0U);
    assert_int_equal(length_or_zero("x", 0U), 0U);
    assert_int_equal(failures(), 4U);

    AssertRecord_t log[ASSERT_MAX_FAILURE_LOG];
    uint16_t count = 0U;
    assert_int_equal(assert_get_log(log, ASSERT_MAX_FAILURE_LOG, &count), SMART_QSO_OK);
    assert_int_equal(count, 4U);
    assert_string_equal(log[0].message, "x must be positive");
}

/** A production precondition stops the bad value from being used */
static void test_flight_code_refuses_bad_input(void **state)
{
    (void)state;
    assert_int_equal(eps_init(), SMART_QSO_OK);
    PowerMode_t before = eps_get_power_mode();
    assert_int_not_equal(eps_set_power_mode((PowerMode_t)7, 0.6), SMART_QSO_OK);
    assert_int_equal(eps_get_power_mode(), before);
    assert_int_not_equal(eps_control_payload(true, 1.5), SMART_QSO_OK);
    assert_true(failures() >= 2U);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_pass_is_silent, setup),
        cmocka_unit_test_setup(test_failure_reported_and_returns_error, setup),
        cmocka_unit_test_setup(test_flight_code_refuses_bad_input, setup),
    };

    return cmocka_run_group_tests_name("Flight-Build Assertion Tests", tests, NULL, NULL);
}
