/**
 * @file assert_handler.c
 * @brief Defensive assertion framework implementation for SMART-QSO
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module implements the flight-safe assertion framework following JPL
 * Power of Ten Rule 5. Assertions are NEVER compiled out in flight builds.
 *
 * Failure handling hierarchy:
 * 1. WARNING: Log and continue
 * 2. ERROR: Log, return error code to caller
 * 3. CRITICAL: Log, transition to safe mode
 * 4. FATAL: Log, persist state, trigger watchdog reset
 *
 * @requirement NPR-7150.2 Software safety requirements
 * @requirement JPL-POT-005 Minimum two assertions per function average
 */

#include "safe_string.h"
#include "fault_mgmt.h"
#include "system_state.h"
#include <stddef.h>

/* Include assert_handler.h after smart_qso.h macros are undefined */
#undef SMART_QSO_REQUIRE
#undef SMART_QSO_ENSURE
#undef SMART_QSO_REQUIRE_NOT_NULL
#include "assert_handler.h"

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Assertion statistics */
static AssertStats_t s_stats;

/** Assertion failure log (circular buffer) */
static AssertRecord_t s_failure_log[ASSERT_MAX_FAILURE_LOG];

/** Current log write index */
static uint16_t s_log_index = 0;

/** Safe mode callback */
static void (*s_safe_mode_callback)(void) = NULL;

/** Reset callback */
static void (*s_reset_callback)(void) = NULL;

/** Module initialized flag */
static bool s_initialized = false;

/** Critical failure flag */
static bool s_has_critical_failure = false;

/*******************************************************************************
 * Private Function Declarations
 ******************************************************************************/

static void log_failure(const char *file,
                        uint16_t line,
                        AssertType_t type,
                        AssertSeverity_t severity,
                        const char *message);
static const char *extract_filename(const char *path);
static void trigger_safe_mode(void);
static void trigger_reset(void);

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Extract filename from full path
 */
static const char *extract_filename(const char *path)
{
    if (path == NULL) {
        return "unknown";
    }

    const char *filename = path;
    const char *p = path;

    /* Find last path separator */
    while (*p != '\0') {
        if ((*p == '/') || (*p == '\\')) {
            filename = p + 1;
        }
        p++;
    }

    return filename;
}

/**
 * @brief Log an assertion failure
 */
static void log_failure(const char *file,
                        uint16_t line,
                        AssertType_t type,
                        AssertSeverity_t severity,
                        const char *message)
{
    const char *filename = extract_filename(file);
    int cmp_result = 0;

    /* Check for existing entry with same file/line */
    for (uint16_t i = 0; i < s_stats.log_count; i++) {
        (void)safe_strncmp(s_failure_log[i].filename,
                           filename,
                           ASSERT_MAX_FILENAME_LEN,
                           &cmp_result);
        if ((s_failure_log[i].line == line) && (cmp_result == 0)) {
            /* Existing entry - increment count */
            s_failure_log[i].occurrence_count++;
            s_failure_log[i].timestamp_s = sys_get_uptime_s();
            return;
        }
    }

    /* Create new entry */
    AssertRecord_t *record = &s_failure_log[s_log_index];

    (void)safe_strncpy(record->filename, sizeof(record->filename),
                       filename, ASSERT_MAX_FILENAME_LEN, NULL);
    record->line = line;
    record->type = type;
    record->severity = severity;
    record->timestamp_s = sys_get_uptime_s();
    record->occurrence_count = 1;

    if (message != NULL) {
        (void)safe_strncpy(record->message, sizeof(record->message),
                           message, ASSERT_MAX_MSG_LEN, NULL);
    } else {
        record->message[0] = '\0';
    }

    /* Advance log index */
    s_log_index++;
    if (s_log_index >= ASSERT_MAX_FAILURE_LOG) {
        s_log_index = 0;
        s_stats.log_overflows++;
    }

    if (s_stats.log_count < ASSERT_MAX_FAILURE_LOG) {
        s_stats.log_count++;
    }
}

/**
 * @brief Trigger safe mode transition
 */
static void trigger_safe_mode(void)
{
    s_stats.safe_mode_triggers++;
    s_has_critical_failure = true;

    /* Log to fault management */
    (void)fault_log_add(FAULT_TYPE_ASSERTION,
                        FAULT_SEVERITY_CRITICAL,
                        "Assertion triggered safe mode",
                        0.0);

    /* Call registered callback */
    if (s_safe_mode_callback != NULL) {
        s_safe_mode_callback();
    }
}

/**
 * @brief Trigger system reset
 */
static void trigger_reset(void)
{
    s_stats.reset_triggers++;
    s_has_critical_failure = true;

    /* Log to fault management */
    (void)fault_log_add(FAULT_TYPE_ASSERTION,
                        FAULT_SEVERITY_CRITICAL,
                        "Assertion triggered reset",
                        0.0);

    /* Call registered callback to persist state */
    if (s_reset_callback != NULL) {
        s_reset_callback();
    }

    /* In simulation, just log. In flight, this would trigger watchdog */
#ifdef SIMULATION_BUILD
    /* Just return - let simulation continue */
#else
    /* TODO: Trigger watchdog reset */
    /* hal_wdt_force_reset(); */
#endif
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t assert_init(void)
{
    /* Clear statistics */
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));

    /* Clear failure log */
    (void)safe_memset(s_failure_log, sizeof(s_failure_log), 0, sizeof(s_failure_log));

    s_log_index = 0;
    s_safe_mode_callback = NULL;
    s_reset_callback = NULL;
    s_has_critical_failure = false;
    s_initialized = true;

    return SMART_QSO_OK;
}

AssertAction_t assert_handle_failure(const char *file,
                                      uint16_t line,
                                      AssertType_t type,
                                      AssertSeverity_t severity,
                                      const char *message)
{
    AssertAction_t action = ASSERT_ACTION_CONTINUE;

    /* Initialize if needed (defensive) */
    if (!s_initialized) {
        (void)assert_init();
    }

    /* Update statistics */
    s_stats.total_checks++;  /* Note: Only counted when failing */
    s_stats.total_failures++;

    /* Log the failure */
    log_failure(file, line, type, severity, message);

    /* Update severity-specific counts and determine action */
    switch (severity) {
        case ASSERT_SEVERITY_WARNING:
            s_stats.warnings++;
            action = ASSERT_ACTION_CONTINUE;
            break;

        case ASSERT_SEVERITY_ERROR:
            s_stats.errors++;
            action = ASSERT_ACTION_RETURN_ERROR;
            break;

        case ASSERT_SEVERITY_CRITICAL:
            s_stats.criticals++;
            action = ASSERT_ACTION_SAFE_MODE;
            trigger_safe_mode();
            break;

        case ASSERT_SEVERITY_FATAL:
            s_stats.fatals++;
            action = ASSERT_ACTION_RESET;
            trigger_reset();
            break;

        default:
            /* Unknown severity - treat as error */
            s_stats.errors++;
            action = ASSERT_ACTION_RETURN_ERROR;
            break;
    }

    return action;
}

SmartQsoResult_t assert_get_stats(AssertStats_t *stats)
{
    if (stats == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(stats, sizeof(*stats), &s_stats, sizeof(s_stats));
    return SMART_QSO_OK;
}

SmartQsoResult_t assert_get_log(AssertRecord_t *log,
                                 uint16_t max_entries,
                                 uint16_t *count)
{
    if ((log == NULL) || (count == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (max_entries == 0) {
        *count = 0;
        return SMART_QSO_OK;
    }

    uint16_t entries_to_copy = s_stats.log_count;
    if (entries_to_copy > max_entries) {
        entries_to_copy = max_entries;
    }

    /* Copy from oldest to newest */
    uint16_t start_idx = 0;
    if (s_stats.log_count >= ASSERT_MAX_FAILURE_LOG) {
        /* Buffer has wrapped - start from current index (oldest) */
        start_idx = s_log_index;
    }

    for (uint16_t i = 0; i < entries_to_copy; i++) {
        uint16_t src_idx = (uint16_t)((start_idx + i) % ASSERT_MAX_FAILURE_LOG);
        (void)safe_memcpy(&log[i], sizeof(log[i]),
                          &s_failure_log[src_idx], sizeof(AssertRecord_t));
    }

    *count = entries_to_copy;
    return SMART_QSO_OK;
}

SmartQsoResult_t assert_clear_stats(void)
{
    s_stats.total_checks = 0;
    s_stats.total_failures = 0;
    s_stats.warnings = 0;
    s_stats.errors = 0;
    s_stats.criticals = 0;
    s_stats.fatals = 0;
    /* Don't clear safe_mode_triggers and reset_triggers - persist for mission analysis */
    s_stats.log_count = 0;
    /* Don't clear log_overflows */

    (void)safe_memset(s_failure_log, sizeof(s_failure_log), 0, sizeof(s_failure_log));
    s_log_index = 0;

    return SMART_QSO_OK;
}

bool assert_has_critical_failures(void)
{
    return s_has_critical_failure;
}

SmartQsoResult_t assert_register_safe_mode_callback(void (*callback)(void))
{
    s_safe_mode_callback = callback;
    return SMART_QSO_OK;
}

SmartQsoResult_t assert_register_reset_callback(void (*callback)(void))
{
    s_reset_callback = callback;
    return SMART_QSO_OK;
}
