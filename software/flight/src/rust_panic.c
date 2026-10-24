/**
 * @file rust_panic.c
 * @brief Where a panic in the Rust modules ends (software/rust/sqso-ffi)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The Rust parsers have no panic paths (no unchecked indexing, no unwrap;
 * fuzzed). A panic can still come from a broken invariant or an integer
 * overflow, which flight builds check rather than wrap (SW-08). It is a
 * fatal assertion: reported to the assertion handler, then execution stops
 * here. On the target the processor waits for the watchdog to reset it; on
 * the host the process aborts, so a test that panics fails.
 */

#include "assert_handler.h"
#include <stdint.h>
#include <stdlib.h>

void smart_qso_rust_panic(const char *file, uint32_t line);

void smart_qso_rust_panic(const char *file, uint32_t line)
{
    (void)assert_handle_failure(file, (uint16_t)line, ASSERT_TYPE_INVARIANT,
                                ASSERT_SEVERITY_FATAL, "Rust panic");
#ifdef HAL_TARGET_STM32L4
    for (;;) {
        /* No kick: the independent watchdog resets the processor */
    }
#else
    abort();
#endif
}
