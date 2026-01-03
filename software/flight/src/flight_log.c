/**
 * @file flight_log.c
 * @brief Flight-safe logging subsystem implementation
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module implements flight-safe logging without dynamic memory.
 * All logs are stored in a fixed-size ring buffer and can be retrieved
 * for telemetry downlink.
 *
 * @requirement MISRA-C:2012 Rule 21.6 - No stdio.h in production code
 */

#include "flight_log.h"
#include "safe_string.h"
#include <stddef.h>
#include <stdio.h>  /* Only for vsnprintf - can be removed in flight build */

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Log entry ring buffer */
static LogEntry_t s_buffer[LOG_BUFFER_SIZE];

/** Write index (next entry to write) */
static uint16_t s_write_index = 0;

/** Read index (oldest entry) */
static uint16_t s_read_index = 0;

/** Number of entries in buffer */
static uint16_t s_entry_count = 0;

/** Sequence counter */
static uint16_t s_sequence = 0;

/** Current runtime log level */
static LogLevel_t s_log_level = LOG_LEVEL_DEBUG;

/** Output destinations */
static uint8_t s_outputs = LOG_OUTPUT_BUFFER;

/** Custom output callback */
static LogOutputCallback_t s_callback = NULL;

/** Logging statistics */
static LogStats_t s_stats;

/** Module initialized flag */
static bool s_initialized = false;

/*******************************************************************************
 * Private Function Declarations
 ******************************************************************************/

static void output_entry(const LogEntry_t *entry);
static const char *level_to_string(LogLevel_t level);

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Convert log level to string
 */
static const char *level_to_string(LogLevel_t level)
{
    switch (level) {
        case LOG_LEVEL_TRACE:    return "TRACE";
        case LOG_LEVEL_DEBUG:    return "DEBUG";
        case LOG_LEVEL_INFO:     return "INFO ";
        case LOG_LEVEL_WARNING:  return "WARN ";
        case LOG_LEVEL_ERROR:    return "ERROR";
        case LOG_LEVEL_CRITICAL: return "CRIT ";
        case LOG_LEVEL_OFF:      return "OFF  ";
    }
    return "?????";
}

/**
 * @brief Output a log entry to configured destinations
 */
static void output_entry(const LogEntry_t *entry)
{
    if (entry == NULL) {
        return;
    }

    /* Output to UART (simulation only - would use HAL in flight) */
#ifdef SIMULATION_BUILD
    if ((s_outputs & LOG_OUTPUT_UART) != 0) {
        fprintf(stderr, "[%010u][%s][%s] %s\n",
                entry->timestamp_ms,
                level_to_string(entry->level),
                entry->module,
                entry->message);
    }
#endif

    /* Call custom callback if registered */
    if (s_callback != NULL) {
        s_callback(entry);
    }
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t log_init(void)
{
    /* Clear buffer */
    (void)safe_memset(s_buffer, sizeof(s_buffer), 0, sizeof(s_buffer));

    /* Clear statistics */
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));

    /* Reset indices */
    s_write_index = 0;
    s_read_index = 0;
    s_entry_count = 0;
    s_sequence = 0;

    /* Default settings */
    s_log_level = LOG_LEVEL_DEBUG;
    s_outputs = LOG_OUTPUT_BUFFER;
    s_callback = NULL;

#ifdef SIMULATION_BUILD
    /* Enable UART output in simulation */
    s_outputs |= LOG_OUTPUT_UART;
#endif

    s_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t log_set_level(LogLevel_t level)
{
    if (level > LOG_LEVEL_OFF) {
        return SMART_QSO_ERROR_PARAM;
    }
    s_log_level = level;
    return SMART_QSO_OK;
}

LogLevel_t log_get_level(void)
{
    return s_log_level;
}

SmartQsoResult_t log_set_outputs(uint8_t outputs)
{
    s_outputs = outputs;
    return SMART_QSO_OK;
}

SmartQsoResult_t log_register_callback(LogOutputCallback_t callback)
{
    s_callback = callback;
    return SMART_QSO_OK;
}

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
    /* Initialize if needed */
    if (!s_initialized) {
        (void)log_init();
    }

    /* Update total count */
    s_stats.total_logs++;

    /* Check compile-time filter */
    if (level < LOG_MIN_LEVEL) {
        s_stats.filtered_logs++;
        return SMART_QSO_OK;
    }

    /* Check runtime filter */
    if (level < s_log_level) {
        s_stats.filtered_logs++;
        return SMART_QSO_OK;
    }

    /* Update level-specific counts */
    switch (level) {
        case LOG_LEVEL_TRACE:    s_stats.trace_count++;    break;
        case LOG_LEVEL_DEBUG:    s_stats.debug_count++;    break;
        case LOG_LEVEL_INFO:     s_stats.info_count++;     break;
        case LOG_LEVEL_WARNING:  s_stats.warning_count++;  break;
        case LOG_LEVEL_ERROR:    s_stats.error_count++;    break;
        case LOG_LEVEL_CRITICAL: s_stats.critical_count++; break;
        case LOG_LEVEL_OFF:      /* Should not happen */   break;
    }

    /* Get buffer entry */
    LogEntry_t *entry = &s_buffer[s_write_index];

    /* Fill entry */
    entry->timestamp_ms = (uint32_t)(smart_qso_now_ms() & 0xFFFFFFFFU);
    entry->level = level;
    entry->sequence = s_sequence++;

    /* Copy module name */
    if (module != NULL) {
        (void)safe_strncpy(entry->module, sizeof(entry->module),
                           module, LOG_MAX_MODULE_LEN - 1, NULL);
    } else {
        entry->module[0] = '\0';
    }

    /* Format message */
    if (format != NULL) {
        /*
         * Note: vsnprintf with non-literal format is intentional here
         * as this is a logging framework that accepts caller-provided
         * format strings. This is a deviation from -Wformat-nonliteral.
         */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wformat-nonliteral"
        int written = vsnprintf(entry->message, sizeof(entry->message),
                                format, args);
#pragma GCC diagnostic pop
        if (written < 0) {
            entry->message[0] = '\0';
        } else if ((size_t)written >= sizeof(entry->message)) {
            /* Truncated - mark with ellipsis */
            entry->message[sizeof(entry->message) - 4] = '.';
            entry->message[sizeof(entry->message) - 3] = '.';
            entry->message[sizeof(entry->message) - 2] = '.';
            entry->message[sizeof(entry->message) - 1] = '\0';
        }
    } else {
        entry->message[0] = '\0';
    }

    /* Output immediately if configured */
    output_entry(entry);

    /* Advance write index */
    s_write_index++;
    if (s_write_index >= LOG_BUFFER_SIZE) {
        s_write_index = 0;
    }

    /* Update entry count and handle overflow */
    if (s_entry_count < LOG_BUFFER_SIZE) {
        s_entry_count++;
    } else {
        /* Buffer full - overwrite oldest */
        s_read_index++;
        if (s_read_index >= LOG_BUFFER_SIZE) {
            s_read_index = 0;
        }
        s_stats.dropped_logs++;
    }

    /* Update statistics */
    s_stats.buffer_entries = s_entry_count;
    if (s_entry_count > s_stats.buffer_high_water) {
        s_stats.buffer_high_water = s_entry_count;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t log_get_entry(uint16_t index, LogEntry_t *entry)
{
    if (entry == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (index >= s_entry_count) {
        return SMART_QSO_ERROR_PARAM;
    }

    /* Calculate actual buffer index */
    uint16_t buf_index = (uint16_t)((s_read_index + index) % LOG_BUFFER_SIZE);

    (void)safe_memcpy(entry, sizeof(*entry),
                      &s_buffer[buf_index], sizeof(LogEntry_t));

    return SMART_QSO_OK;
}

uint16_t log_get_count(void)
{
    return s_entry_count;
}

SmartQsoResult_t log_clear(void)
{
    s_write_index = 0;
    s_read_index = 0;
    s_entry_count = 0;
    s_stats.buffer_entries = 0;

    return SMART_QSO_OK;
}

SmartQsoResult_t log_get_stats(LogStats_t *stats)
{
    if (stats == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(stats, sizeof(*stats), &s_stats, sizeof(s_stats));
    return SMART_QSO_OK;
}

SmartQsoResult_t log_flush(void)
{
    /* In buffered mode, output all entries */
    if ((s_outputs & LOG_OUTPUT_BUFFER) != 0) {
        for (uint16_t i = 0; i < s_entry_count; i++) {
            uint16_t buf_index = (uint16_t)((s_read_index + i) % LOG_BUFFER_SIZE);
            output_entry(&s_buffer[buf_index]);
        }
    }

    return SMART_QSO_OK;
}
