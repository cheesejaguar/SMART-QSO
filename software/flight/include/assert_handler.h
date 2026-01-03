/**
 * @file assert_handler.h
 * @brief Defensive assertion framework for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module provides a flight-safe assertion framework following JPL Power of
 * Ten Rule 5: "Use a minimum of two runtime assertions per function on average."
 *
 * Key features:
 * - Assertions are NEVER compiled out in flight builds
 * - Failed assertions trigger safe mode transition
 * - All failures are logged with file/line information
 * - Supports preconditions, postconditions, and invariants
 *
 * @requirement NPR-7150.2 Software safety requirements
 * @requirement JPL-POT-005 Minimum two assertions per function average
 *
 * Inspired by NASA Ames heritage: GeneSat, PharmaSat, O/OREOS assertion patterns
 */

#ifndef SMART_QSO_ASSERT_HANDLER_H
#define SMART_QSO_ASSERT_HANDLER_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** Maximum length of assertion message */
#define ASSERT_MAX_MSG_LEN          64U

/** Maximum number of recent assertion failures to log */
#define ASSERT_MAX_FAILURE_LOG      16U

/** Assert log entry filename max length */
#define ASSERT_MAX_FILENAME_LEN     32U

/*******************************************************************************
 * Assertion Types
 ******************************************************************************/

/**
 * @brief Types of assertions
 */
typedef enum {
    ASSERT_TYPE_PRECONDITION    = 0,  /**< Function precondition */
    ASSERT_TYPE_POSTCONDITION   = 1,  /**< Function postcondition */
    ASSERT_TYPE_INVARIANT       = 2,  /**< Loop or state invariant */
    ASSERT_TYPE_PARAM_NULL      = 3,  /**< NULL pointer check */
    ASSERT_TYPE_PARAM_RANGE     = 4,  /**< Parameter range check */
    ASSERT_TYPE_ARRAY_BOUNDS    = 5,  /**< Array bounds check */
    ASSERT_TYPE_STATE           = 6,  /**< State validity check */
    ASSERT_TYPE_INTERNAL        = 7,  /**< Internal consistency check */
    ASSERT_TYPE_UNREACHABLE     = 8,  /**< Should never reach here */
    ASSERT_TYPE_COUNT           = 9
} AssertType_t;

/**
 * @brief Assertion severity levels
 */
typedef enum {
    ASSERT_SEVERITY_WARNING     = 0,  /**< Log but continue */
    ASSERT_SEVERITY_ERROR       = 1,  /**< Log and return error */
    ASSERT_SEVERITY_CRITICAL    = 2,  /**< Log and enter safe mode */
    ASSERT_SEVERITY_FATAL       = 3   /**< Log and reset system */
} AssertSeverity_t;

/**
 * @brief Assertion response actions
 */
typedef enum {
    ASSERT_ACTION_CONTINUE      = 0,  /**< Continue execution */
    ASSERT_ACTION_RETURN_ERROR  = 1,  /**< Return from function with error */
    ASSERT_ACTION_SAFE_MODE     = 2,  /**< Transition to safe mode */
    ASSERT_ACTION_RESET         = 3   /**< Reset the system */
} AssertAction_t;

/**
 * @brief Assertion failure record
 */
typedef struct {
    char filename[ASSERT_MAX_FILENAME_LEN];  /**< Source file */
    uint16_t line;                           /**< Line number */
    AssertType_t type;                       /**< Assertion type */
    AssertSeverity_t severity;               /**< Severity level */
    uint32_t timestamp_s;                    /**< Mission time of failure */
    uint32_t occurrence_count;               /**< Times this assertion failed */
    char message[ASSERT_MAX_MSG_LEN];        /**< Optional message */
} AssertRecord_t;

/**
 * @brief Assertion statistics
 */
typedef struct {
    uint32_t total_checks;           /**< Total assertions evaluated */
    uint32_t total_failures;         /**< Total assertion failures */
    uint32_t warnings;               /**< Warning-level failures */
    uint32_t errors;                 /**< Error-level failures */
    uint32_t criticals;              /**< Critical-level failures */
    uint32_t fatals;                 /**< Fatal-level failures */
    uint32_t safe_mode_triggers;     /**< Times safe mode was triggered */
    uint32_t reset_triggers;         /**< Times reset was triggered */
    uint16_t log_count;              /**< Current log entries */
    uint16_t log_overflows;          /**< Times log wrapped */
} AssertStats_t;

/*******************************************************************************
 * Assertion Macros - Primary Interface
 ******************************************************************************/

/**
 * @brief Check precondition (function entry requirement)
 * @param condition Condition that must be true
 *
 * Use at the beginning of functions to validate parameters and state.
 */
#define SMART_QSO_REQUIRE(condition) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_PRECONDITION, ASSERT_SEVERITY_ERROR, #condition); \
        } \
    } while (0)

/**
 * @brief Check precondition with custom severity
 */
#define SMART_QSO_REQUIRE_EX(condition, severity) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_PRECONDITION, (severity), #condition); \
        } \
    } while (0)

/**
 * @brief Check postcondition (function exit requirement)
 * @param condition Condition that must be true
 *
 * Use at the end of functions to validate return values and state.
 */
#define SMART_QSO_ENSURE(condition) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_POSTCONDITION, ASSERT_SEVERITY_ERROR, #condition); \
        } \
    } while (0)

/**
 * @brief Check invariant (loop or state consistency)
 * @param condition Condition that must remain true
 *
 * Use within loops or to verify state consistency.
 */
#define SMART_QSO_INVARIANT(condition) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_INVARIANT, ASSERT_SEVERITY_ERROR, #condition); \
        } \
    } while (0)

/**
 * @brief Check that pointer is not NULL
 * @param ptr Pointer to check
 *
 * Returns SMART_QSO_ERROR_NULL_PTR if the pointer is NULL.
 */
#define SMART_QSO_REQUIRE_NOT_NULL(ptr) \
    do { \
        if ((ptr) == NULL) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_PARAM_NULL, ASSERT_SEVERITY_ERROR, #ptr " != NULL"); \
            return SMART_QSO_ERROR_NULL_PTR; \
        } \
    } while (0)

/**
 * @brief Check that pointer is not NULL (void return version)
 * @param ptr Pointer to check
 */
#define SMART_QSO_REQUIRE_NOT_NULL_VOID(ptr) \
    do { \
        if ((ptr) == NULL) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_PARAM_NULL, ASSERT_SEVERITY_ERROR, #ptr " != NULL"); \
            return; \
        } \
    } while (0)

/**
 * @brief Check that value is within range [min, max]
 * @param value Value to check
 * @param min_val Minimum allowed value (inclusive)
 * @param max_val Maximum allowed value (inclusive)
 */
#define SMART_QSO_REQUIRE_RANGE(value, min_val, max_val) \
    do { \
        if (((value) < (min_val)) || ((value) > (max_val))) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_PARAM_RANGE, ASSERT_SEVERITY_ERROR, \
                #value " in [" #min_val ", " #max_val "]"); \
        } \
    } while (0)

/**
 * @brief Check that array index is within bounds
 * @param index Index to check
 * @param array_size Size of the array
 */
#define SMART_QSO_REQUIRE_BOUNDS(index, array_size) \
    do { \
        if ((size_t)(index) >= (size_t)(array_size)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_ARRAY_BOUNDS, ASSERT_SEVERITY_CRITICAL, \
                #index " < " #array_size); \
        } \
    } while (0)

/**
 * @brief Check state validity
 * @param condition State condition that must be true
 *
 * Use to verify system state before operations.
 */
#define SMART_QSO_REQUIRE_STATE(condition) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_STATE, ASSERT_SEVERITY_CRITICAL, #condition); \
        } \
    } while (0)

/**
 * @brief General assertion (internal consistency check)
 * @param condition Condition that must be true
 */
#define SMART_QSO_ASSERT(condition) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_INTERNAL, ASSERT_SEVERITY_ERROR, #condition); \
        } \
    } while (0)

/**
 * @brief Critical assertion (triggers safe mode on failure)
 * @param condition Condition that must be true
 */
#define SMART_QSO_ASSERT_CRITICAL(condition) \
    do { \
        if (!(condition)) { \
            (void)assert_handle_failure(__FILE__, __LINE__, \
                ASSERT_TYPE_INTERNAL, ASSERT_SEVERITY_CRITICAL, #condition); \
        } \
    } while (0)

/**
 * @brief Mark unreachable code (indicates programmer error if executed)
 */
#define SMART_QSO_UNREACHABLE() \
    do { \
        (void)assert_handle_failure(__FILE__, __LINE__, \
            ASSERT_TYPE_UNREACHABLE, ASSERT_SEVERITY_CRITICAL, \
            "unreachable code executed"); \
    } while (0)

/**
 * @brief Static assertion (compile-time check)
 * @param condition Condition that must be true at compile time
 * @param message Error message if condition is false
 */
#define SMART_QSO_STATIC_ASSERT(condition, message) \
    _Static_assert((condition), message)

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize assertion handler module
 *
 * @return SMART_QSO_OK on success
 *
 * Must be called before any assertions are evaluated.
 */
SmartQsoResult_t assert_init(void);

/**
 * @brief Handle an assertion failure
 *
 * @param[in] file Source file name
 * @param[in] line Source line number
 * @param[in] type Type of assertion
 * @param[in] severity Severity level
 * @param[in] message Assertion expression or message
 * @return Action taken in response to the failure
 *
 * This function is called by the assertion macros. It logs the failure,
 * updates statistics, and takes appropriate action based on severity.
 */
AssertAction_t assert_handle_failure(const char *file,
                                      uint16_t line,
                                      AssertType_t type,
                                      AssertSeverity_t severity,
                                      const char *message);

/**
 * @brief Get assertion statistics
 *
 * @param[out] stats Statistics output structure
 * @return SMART_QSO_OK on success, SMART_QSO_ERROR_NULL_PTR if stats is NULL
 */
SmartQsoResult_t assert_get_stats(AssertStats_t *stats);

/**
 * @brief Get recent assertion failure log
 *
 * @param[out] log Array of assertion records
 * @param[in] max_entries Maximum entries to retrieve
 * @param[out] count Actual number of entries retrieved
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t assert_get_log(AssertRecord_t *log,
                                 uint16_t max_entries,
                                 uint16_t *count);

/**
 * @brief Clear assertion statistics and log
 *
 * @return SMART_QSO_OK on success
 *
 * Should be called after transmitting assertion data to ground.
 */
SmartQsoResult_t assert_clear_stats(void);

/**
 * @brief Check if any critical assertions have failed
 *
 * @return true if critical or fatal assertions have occurred
 */
bool assert_has_critical_failures(void);

/**
 * @brief Register callback for safe mode trigger
 *
 * @param[in] callback Function to call when safe mode is triggered
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t assert_register_safe_mode_callback(void (*callback)(void));

/**
 * @brief Register callback for system reset trigger
 *
 * @param[in] callback Function to call before system reset
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t assert_register_reset_callback(void (*callback)(void));

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_ASSERT_HANDLER_H */
