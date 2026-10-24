/**
 * @file flight_log_fmt.c
 * @brief log_write() and log_vwrite(): the printf-style entry points of the flight log
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The flight log is implemented in Rust (software/rust/sqso-ffi/src/flight_log.rs).
 * Stable Rust can neither define a C variadic function nor read a va_list,
 * so these two stay in C. They ask Rust whether the level passes the
 * filters (which also counts the call), format the message with vsnprintf
 * and hand the text to Rust, which records and outputs the entry.
 *
 * @note MISRA C:2012 Rule 21.6 (SQSO-CS-001): stdio.h is used for vsnprintf only
 */

#include "flight_log.h"
#include "smart_qso.h"
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>

/* Rust (flight_log.rs): count a log call; whether its level passes the filters */
bool flight_log_admit(LogLevel_t level);

/* Rust (flight_log.rs): record and output an admitted entry; message is a
 * LOG_MAX_MESSAGE_LEN buffer (NULL without a format), written the vsnprintf result */
SmartQsoResult_t flight_log_record(LogLevel_t level, const char *module,
                                   const char *message, int written);

SmartQsoResult_t log_write(LogLevel_t level,
                           const char *module,
                           const char *format, ...)
{
    va_list args;
    va_start(args, format);
    SmartQsoResult_t result = log_vwrite(level, module, format, args);
    va_end(args);
    return result;
}

SmartQsoResult_t log_vwrite(LogLevel_t level,
                            const char *module,
                            const char *format,
                            va_list args)
{
    SmartQsoResult_t result = SMART_QSO_OK;

    if (!flight_log_admit(level)) {
        /* Filtered: counted, not formatted */
    } else if (format == NULL) {
        result = flight_log_record(level, module, NULL, 0);
    } else {
        /* Every byte initialized: Rust copies the whole buffer */
        char message[LOG_MAX_MESSAGE_LEN] = {0};
        /*
         * A logging framework takes the caller's format string: a deliberate
         * deviation from -Wformat-nonliteral.
         */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wformat-nonliteral"
        int written = vsnprintf(message, sizeof(message), format, args);
#pragma GCC diagnostic pop
        result = flight_log_record(level, module, message, written);
    }
    return result;
}
