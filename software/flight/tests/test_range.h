/**
 * @file test_range.h
 * @brief Unsigned range assertion that builds the same on cmocka 1.1 and 2.0
 *
 * cmocka 2.0 deprecates assert_in_range, and its assert_uint_in_range
 * converts through intmax_t, which -Wsign-conversion rejects; cmocka 1.1
 * has only assert_in_range. Include after <cmocka.h>.
 */
#ifndef SMART_QSO_TEST_RANGE_H
#define SMART_QSO_TEST_RANGE_H

#include <inttypes.h>
#include <stdint.h>

/** Fail the test unless minimum <= value <= maximum (all unsigned) */
#define assert_u64_in_range(value, minimum, maximum)                         \
    do {                                                                     \
        const uint64_t range_v_ = (uint64_t)(value);                         \
        const uint64_t range_lo_ = (uint64_t)(minimum);                      \
        const uint64_t range_hi_ = (uint64_t)(maximum);                      \
        if ((range_v_ < range_lo_) || (range_v_ > range_hi_)) {              \
            print_error("%" PRIu64 " is not within [%" PRIu64 ", %" PRIu64 "]\n", \
                        range_v_, range_lo_, range_hi_);                     \
            fail();                                                          \
        }                                                                    \
    } while (0)

#endif /* SMART_QSO_TEST_RANGE_H */
