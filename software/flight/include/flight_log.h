/**
 * @file flight_log.h
 * @brief Flight-safe logging subsystem for SMART-QSO
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module provides a flight-safe logging interface that replaces
 * printf/fprintf (prohibited by MISRA C:2012 Rule 21.6). Features:
 * - Configurable log levels
 * - Compile-time log level filtering
 * - Optional buffered output for telemetry downlink
 * - Thread-safe design for RTOS integration
 * - Zero dynamic memory allocation
 *
 * @requirement MISRA-C:2012 Rule 21.6 - No stdio.h in production code
 * @requirement NPR-7150.2 - Logging and telemetry requirements
 *
 * Heritage: NASA Ames SmallSat Division logging patterns
 */

#ifndef SMART_QSO_FLIGHT_LOG_H
#define SMART_QSO_FLIGHT_LOG_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>

/*******************************************************************************
 * Configuration
 ******************************************************************************/

/** Maximum log message length */
#define LOG_MAX_MESSAGE_LEN     128U

/** Maximum log entries in buffer */
#define LOG_BUFFER_SIZE         64U

/** Maximum module name length */
#define LOG_MAX_MODULE_LEN      16U

/** Enable/disable logging at compile time */
#ifndef LOG_ENABLED
#define LOG_ENABLED             1
#endif

/** Minimum log level at compile time (lower = more verbose) */
#ifndef LOG_MIN_LEVEL
#define LOG_MIN_LEVEL           LOG_LEVEL_DEBUG
#endif

/*******************************************************************************
 * Log Levels
 ******************************************************************************/

/**
 * @brief Log severity levels
 *
 * Lower values are more verbose; higher values are more critical.
 */
typedef enum {
    LOG_LEVEL_TRACE    = 0,   /**< Very detailed tracing */
    LOG_LEVEL_DEBUG    = 1,   /**< Debug information */
    LOG_LEVEL_INFO     = 2,   /**< Informational messages */
    LOG_LEVEL_WARNING  = 3,   /**< Warning conditions */
    LOG_LEVEL_ERROR    = 4,   /**< Error conditions */
    LOG_LEVEL_CRITICAL = 5,   /**< Critical errors */
    LOG_LEVEL_OFF      = 6    /**< Logging disabled */
} LogLevel_t;

/**
 * @brief Log output destinations
 */
typedef enum {
    LOG_OUTPUT_NONE     = 0x00,  /**< Discard logs */
    LOG_OUTPUT_BUFFER   = 0x01,  /**< Store in ring buffer */
    LOG_OUTPUT_UART     = 0x02,  /**< Send to debug UART */
    LOG_OUTPUT_TELEMETRY = 0x04  /**< Include in telemetry */
} LogOutput_t;

/*******************************************************************************
 * Log Entry Structure
 ******************************************************************************/

/**
 * @brief Log entry stored in buffer
 */
typedef struct {
    uint32_t timestamp_ms;              /**< Timestamp (ms since boot) */
    LogLevel_t level;                   /**< Log level */
    char module[LOG_MAX_MODULE_LEN];    /**< Module name */
    char message[LOG_MAX_MESSAGE_LEN];  /**< Log message */
    uint16_t sequence;                  /**< Sequence number */
} LogEntry_t;

/**
 * @brief Logging statistics
 */
typedef struct {
    uint32_t total_logs;        /**< Total log calls */
    uint32_t filtered_logs;     /**< Logs filtered by level */
    uint32_t dropped_logs;      /**< Logs dropped (buffer full) */
    uint32_t trace_count;       /**< TRACE level logs */
    uint32_t debug_count;       /**< DEBUG level logs */
    uint32_t info_count;        /**< INFO level logs */
    uint32_t warning_count;     /**< WARNING level logs */
    uint32_t error_count;       /**< ERROR level logs */
    uint32_t critical_count;    /**< CRITICAL level logs */
    uint16_t buffer_entries;    /**< Current buffer entries */
    uint16_t buffer_high_water; /**< Maximum buffer usage */
} LogStats_t;

/*******************************************************************************
 * Output Callback Type
 ******************************************************************************/

/**
 * @brief Custom output callback function type
 *
 * @param entry Log entry to output
 */
typedef void (*LogOutputCallback_t)(const LogEntry_t *entry);

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize logging subsystem
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_init(void);

/**
 * @brief Set runtime log level filter
 *
 * Messages below this level will be discarded.
 *
 * @param[in] level Minimum log level to record
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_set_level(LogLevel_t level);

/**
 * @brief Get current log level
 *
 * @return Current minimum log level
 */
LogLevel_t log_get_level(void);

/**
 * @brief Set output destinations
 *
 * @param[in] outputs Bitmask of LogOutput_t values
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_set_outputs(uint8_t outputs);

/**
 * @brief Register custom output callback
 *
 * @param[in] callback Function to call for each log entry
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_register_callback(LogOutputCallback_t callback);

/**
 * @brief Log a message
 *
 * @param[in] level Log level
 * @param[in] module Module name (max LOG_MAX_MODULE_LEN-1 chars)
 * @param[in] format printf-style format string
 * @param[in] ... Format arguments
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_write(LogLevel_t level,
                           const char *module,
                           const char *format, ...);

/**
 * @brief Log a message with va_list
 *
 * @param[in] level Log level
 * @param[in] module Module name
 * @param[in] format Format string
 * @param[in] args Variable arguments
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_vwrite(LogLevel_t level,
                            const char *module,
                            const char *format,
                            va_list args);

/**
 * @brief Get log entry from buffer
 *
 * @param[in] index Entry index (0 = oldest)
 * @param[out] entry Entry data
 * @return SMART_QSO_OK on success, SMART_QSO_ERROR_PARAM if index invalid
 */
SmartQsoResult_t log_get_entry(uint16_t index, LogEntry_t *entry);

/**
 * @brief Get number of entries in buffer
 *
 * @return Number of buffered entries
 */
uint16_t log_get_count(void);

/**
 * @brief Clear log buffer
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_clear(void);

/**
 * @brief Get logging statistics
 *
 * @param[out] stats Statistics structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_get_stats(LogStats_t *stats);

/**
 * @brief Flush buffered logs (if applicable)
 *
 * Forces output of all buffered logs to registered callbacks.
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t log_flush(void);

/*******************************************************************************
 * Convenience Macros
 ******************************************************************************/

#if LOG_ENABLED

/** Log at TRACE level */
#define LOG_TRACE(module, ...) \
    do { \
        if (LOG_LEVEL_TRACE >= LOG_MIN_LEVEL) { \
            (void)log_write(LOG_LEVEL_TRACE, module, __VA_ARGS__); \
        } \
    } while (0)

/** Log at DEBUG level */
#define LOG_DEBUG(module, ...) \
    do { \
        if (LOG_LEVEL_DEBUG >= LOG_MIN_LEVEL) { \
            (void)log_write(LOG_LEVEL_DEBUG, module, __VA_ARGS__); \
        } \
    } while (0)

/** Log at INFO level */
#define LOG_INFO(module, ...) \
    do { \
        if (LOG_LEVEL_INFO >= LOG_MIN_LEVEL) { \
            (void)log_write(LOG_LEVEL_INFO, module, __VA_ARGS__); \
        } \
    } while (0)

/** Log at WARNING level */
#define LOG_WARNING(module, ...) \
    do { \
        if (LOG_LEVEL_WARNING >= LOG_MIN_LEVEL) { \
            (void)log_write(LOG_LEVEL_WARNING, module, __VA_ARGS__); \
        } \
    } while (0)

/** Log at ERROR level */
#define LOG_ERROR(module, ...) \
    do { \
        if (LOG_LEVEL_ERROR >= LOG_MIN_LEVEL) { \
            (void)log_write(LOG_LEVEL_ERROR, module, __VA_ARGS__); \
        } \
    } while (0)

/** Log at CRITICAL level */
#define LOG_CRITICAL(module, ...) \
    do { \
        if (LOG_LEVEL_CRITICAL >= LOG_MIN_LEVEL) { \
            (void)log_write(LOG_LEVEL_CRITICAL, module, __VA_ARGS__); \
        } \
    } while (0)

#else /* LOG_ENABLED */

#define LOG_TRACE(module, ...)    ((void)0)
#define LOG_DEBUG(module, ...)    ((void)0)
#define LOG_INFO(module, ...)     ((void)0)
#define LOG_WARNING(module, ...)  ((void)0)
#define LOG_ERROR(module, ...)    ((void)0)
#define LOG_CRITICAL(module, ...) ((void)0)

#endif /* LOG_ENABLED */

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_FLIGHT_LOG_H */
