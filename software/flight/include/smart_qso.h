/**
 * @file smart_qso.h
 * @brief SMART-QSO Flight Software Master Header
 *
 * This header provides common types, constants, and configuration for the
 * SMART-QSO CubeSat flight software. All modules should include this header.
 *
 * @requirement SRS-Q010 Modular software with defined interfaces
 * @note Coding standard SQSO-CS-001 (docs/development/CODING_STANDARDS.md): JPL Power of Ten rules
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @note Following GeneSat/PharmaSat/O/OREOS/EcAMSat design patterns
 */

#ifndef SMART_QSO_SMART_QSO_H
#define SMART_QSO_SMART_QSO_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/*===========================================================================*/
/* Version Information                                                        */
/*===========================================================================*/

#define SMART_QSO_VERSION_MAJOR  0
#define SMART_QSO_VERSION_MINOR  2
#define SMART_QSO_VERSION_PATCH  0
#define SMART_QSO_VERSION_STRING "0.2.0-dev"

/*===========================================================================*/
/* Build Configuration                                                        */
/*===========================================================================*/

/** Build target: FLIGHT_BUILD for flight hardware, else simulation */
#ifndef FLIGHT_BUILD
#define SIMULATION_BUILD 1
#endif

/** Enable/disable debug output */
#ifndef SMART_QSO_DEBUG_ENABLED
#  ifndef NDEBUG
#    define SMART_QSO_DEBUG_ENABLED 1
#  else
#    define SMART_QSO_DEBUG_ENABLED 0
#  endif
#endif

/*===========================================================================*/
/* System Constants                                                           */
/*===========================================================================*/

/** Maximum number of sensors supported */
#define SMART_QSO_MAX_SENSORS           32

/** Maximum fault log entries */
#define SMART_QSO_MAX_FAULT_ENTRIES     100

/** Watchdog timeout in milliseconds */
#define SMART_QSO_WATCHDOG_TIMEOUT_MS   30000

/** Default telemetry interval in milliseconds */
#define SMART_QSO_TELEMETRY_INTERVAL_MS 60000

/** Maximum sensor ID length */
#define SMART_QSO_SENSOR_ID_LEN         8

/** Maximum sensor name length */
#define SMART_QSO_SENSOR_NAME_LEN       64

/** Maximum sensor units length */
#define SMART_QSO_SENSOR_UNITS_LEN      16

/** Maximum sensor type length */
#define SMART_QSO_SENSOR_TYPE_LEN       32

/** Maximum sensor channel length */
#define SMART_QSO_SENSOR_CHANNEL_LEN    32

/** Maximum fault description length */
#define SMART_QSO_FAULT_DESC_LEN        64

/*===========================================================================*/
/* Power Mode Definitions                                                     */
/*===========================================================================*/

/**
 * @brief Power mode enumeration
 *
 * Defines the operational power modes for the spacecraft.
 * Mode transitions are managed by the EPS control module.
 */
typedef enum {
    POWER_MODE_SAFE   = 0,  /**< Safe mode: minimum power, beacon only */
    POWER_MODE_IDLE   = 1,  /**< Idle mode: normal operations, no AI payload */
    POWER_MODE_ACTIVE = 2   /**< Active mode: full operations including AI payload */
} PowerMode_t;

/*===========================================================================*/
/* Fault Severity Definitions                                                 */
/*===========================================================================*/

/**
 * @brief Fault severity enumeration
 */
typedef enum {
    FAULT_SEVERITY_INFO     = 1,  /**< Informational */
    FAULT_SEVERITY_WARNING  = 2,  /**< Warning condition */
    FAULT_SEVERITY_ERROR    = 3,  /**< Error condition */
    FAULT_SEVERITY_CRITICAL = 4   /**< Critical fault requiring immediate action */
} FaultSeverity_t;

/*===========================================================================*/
/* Fault Type Definitions                                                     */
/*===========================================================================*/

/**
 * @brief Fault type enumeration
 */
typedef enum {
    FAULT_TYPE_POWER           = 1,   /**< Power-related fault */
    FAULT_TYPE_MODE_CHANGE     = 2,   /**< Mode transition event */
    FAULT_TYPE_THERMAL         = 3,   /**< Thermal fault */
    FAULT_TYPE_POWER_CRITICAL  = 4,   /**< Critical power fault */
    FAULT_TYPE_WATCHDOG        = 5,   /**< Watchdog timeout */
    FAULT_TYPE_UART            = 6,   /**< UART communication fault */
    FAULT_TYPE_VOLTAGE_LOW     = 7,   /**< Low voltage fault */
    FAULT_TYPE_VOLTAGE_RANGE   = 8,   /**< Voltage out of range */
    FAULT_TYPE_CURRENT_HIGH    = 9,   /**< High current fault */
    FAULT_TYPE_TEMP_HIGH       = 10,  /**< High temperature fault */
    FAULT_TYPE_TEMP_LOW        = 11,  /**< Low temperature fault */
    FAULT_TYPE_RECOVERY        = 12,  /**< Recovery event */
    FAULT_TYPE_HEALTH_THERMAL  = 13,  /**< Health check thermal */
    FAULT_TYPE_HEALTH_POWER    = 14,  /**< Health check power */
    FAULT_TYPE_HEALTH_COMM     = 15,  /**< Health check communication */
    FAULT_TYPE_INIT            = 16,  /**< Initialization fault */
    FAULT_TYPE_ADCS            = 17,  /**< ADCS fault */
    FAULT_TYPE_ASSERTION       = 18,  /**< Assertion failure (JPL Rule 5) */
    FAULT_TYPE_COMMAND         = 19,  /**< Command processing fault */
    FAULT_TYPE_TELEMETRY       = 20,  /**< Telemetry generation fault */
    FAULT_TYPE_DEPLOYMENT      = 21,  /**< Deployment fault */
    FAULT_TYPE_SW_INTERNAL     = 22   /**< Internal software fault */
} FaultType_t;

/*===========================================================================*/
/* Sensor Types                                                               */
/*===========================================================================*/

/**
 * @brief Sensor value type enumeration
 */
typedef enum {
    SENSOR_VALUE_NUMERIC = 0,  /**< Numeric (double) value */
    SENSOR_VALUE_HEX2    = 1   /**< Two-character hex string */
} SensorValueType_t;

/*===========================================================================*/
/* Common Return Codes                                                        */
/*===========================================================================*/

/**
 * @brief Common return codes for module functions
 */
typedef enum {
    SMART_QSO_OK              = 0,   /**< Success */
    SMART_QSO_ERROR           = -1,  /**< General error */
    SMART_QSO_ERROR_NULL_PTR  = -2,  /**< Null pointer error */
    SMART_QSO_ERROR_INVALID   = -3,  /**< Invalid parameter */
    SMART_QSO_ERROR_TIMEOUT   = -4,  /**< Operation timeout */
    SMART_QSO_ERROR_BUSY      = -5,  /**< Resource busy */
    SMART_QSO_ERROR_NO_MEM    = -6,  /**< Out of memory */
    SMART_QSO_ERROR_IO        = -7,  /**< I/O error */
    SMART_QSO_ERROR_TRUNCATED = -8,  /**< Data was truncated */
    SMART_QSO_ERROR_PARAM     = -9   /**< Invalid parameter */
} SmartQsoResult_t;

/* Convenience aliases for result codes */
#define RESULT_OK               SMART_QSO_OK
#define RESULT_ERROR            SMART_QSO_ERROR
#define RESULT_ERROR_PARAM      SMART_QSO_ERROR_PARAM
#define RESULT_ERROR_TRUNCATED  SMART_QSO_ERROR_TRUNCATED

/*===========================================================================*/
/* Assertion Macros (JPL Rule 5)                                              */
/*===========================================================================*/

/**
 * @brief Assertions: one API, active in every build (SW-08, JPL Rule 5)
 *
 * A failed check is reported to the assertion handler (assert_handler.h,
 * implemented in software/rust/sqso-ffi/src/assert_handler.rs: logged,
 * counted, and fault-logged) in every build, flight included. The
 * function then returns an error, so it never carries on with the bad value.
 *
 * - SMART_QSO_REQUIRE(condition, message)   precondition; returns SMART_QSO_ERROR_INVALID
 * - SMART_QSO_ENSURE(condition, message)    postcondition; returns SMART_QSO_ERROR
 * - SMART_QSO_REQUIRE_NOT_NULL(ptr)         returns SMART_QSO_ERROR_NULL_PTR
 * - SMART_QSO_REQUIRE_OR(condition, message, retval) and
 *   SMART_QSO_REQUIRE_NOT_NULL_OR(ptr, retval) for functions that do not
 *   return SmartQsoResult_t
 *
 * assert_handler.h adds checks with an explicit severity (critical: safe
 * mode; fatal: reset).
 */
void smart_qso_assert_failed(const char *file, uint32_t line, const char *message);

#define SMART_QSO_REQUIRE_OR(condition, message, retval) \
    do { \
        if (!(condition)) { \
            smart_qso_assert_failed(__FILE__, (uint32_t)__LINE__, (message)); \
            return (retval); \
        } \
    } while (0)

#define SMART_QSO_REQUIRE(condition, message) \
    SMART_QSO_REQUIRE_OR((condition), (message), SMART_QSO_ERROR_INVALID)
#define SMART_QSO_ENSURE(condition, message) \
    SMART_QSO_REQUIRE_OR((condition), (message), SMART_QSO_ERROR)
#define SMART_QSO_REQUIRE_NOT_NULL_OR(ptr, retval) \
    SMART_QSO_REQUIRE_OR((ptr) != NULL, "Null pointer: " #ptr, (retval))
#define SMART_QSO_REQUIRE_NOT_NULL(ptr) \
    SMART_QSO_REQUIRE_NOT_NULL_OR((ptr), SMART_QSO_ERROR_NULL_PTR)

/*===========================================================================*/
/* Time Functions                                                             */
/*===========================================================================*/

/**
 * @brief Get current time in milliseconds
 *
 * @return Mission time in ms: wall-clock time since the Unix epoch (plus
 *         the test offset) in the host simulation; milliseconds since boot
 *         on the target (monotonic). Implemented in
 *         software/rust/sqso-ffi/src/time.rs
 */
uint64_t smart_qso_now_ms(void);

#ifdef SIMULATION_BUILD
/**
 * @brief Simulation only: move the clock forward
 *
 * Shifts smart_qso_now_ms() and the simulated HAL timer together, so tests
 * can run time-dependent code without sleeping.
 */
void smart_qso_sim_advance_ms(uint64_t ms);

/** Simulation only: total time added by smart_qso_sim_advance_ms() */
uint64_t smart_qso_sim_offset_ms(void);
#endif

/*===========================================================================*/
/* CRC Functions                                                              */
/*===========================================================================*/

/**
 * @brief Calculate CRC32 checksum
 *
 * @param data Pointer to data buffer
 * @param len  Length of data in bytes
 * @return CRC32 checksum
 */
uint32_t smart_qso_crc32(const void *data, size_t len);

/**
 * @brief Verify CRC32 checksum
 *
 * @param data         Pointer to data buffer
 * @param len          Length of data in bytes
 * @param expected_crc Expected CRC32 value
 * @return true if CRC matches, false otherwise
 */
bool smart_qso_verify_crc32(const void *data, size_t len, uint32_t expected_crc);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SMART_QSO_H */
