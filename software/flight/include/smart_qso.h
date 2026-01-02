/**
 * @file smart_qso.h
 * @brief SMART-QSO Flight Software Master Header
 *
 * This header provides common types, constants, and configuration for the
 * SMART-QSO CubeSat flight software. All modules should include this header.
 *
 * @requirement SRS-001 System shall provide modular flight software architecture
 * @requirement SRS-002 System shall comply with JPL Power of Ten coding rules
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
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
#ifndef NDEBUG
#define SMART_QSO_DEBUG_ENABLED 1
#else
#define SMART_QSO_DEBUG_ENABLED 0
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
    FAULT_TYPE_ADCS            = 17   /**< ADCS fault */
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
    SMART_QSO_ERROR_IO        = -7   /**< I/O error */
} SmartQsoResult_t;

/*===========================================================================*/
/* Assertion Macros (JPL Rule 5)                                              */
/*===========================================================================*/

/**
 * @brief Custom assertion macro for flight software
 *
 * In flight builds, assertions log the fault and continue.
 * In debug builds, assertions may halt execution.
 *
 * @param condition The condition to assert
 * @param message   Error message if assertion fails
 */
#if SMART_QSO_DEBUG_ENABLED
#include <stdio.h>
#define SMART_QSO_ASSERT(condition, message) \
    do { \
        if (!(condition)) { \
            fprintf(stderr, "[ASSERT FAILED] %s:%d: %s\n", __FILE__, __LINE__, message); \
        } \
    } while (0)
#else
#define SMART_QSO_ASSERT(condition, message) ((void)0)
#endif

/**
 * @brief Precondition check macro
 */
#define SMART_QSO_REQUIRE(condition, message) SMART_QSO_ASSERT(condition, message)

/**
 * @brief Postcondition check macro
 */
#define SMART_QSO_ENSURE(condition, message) SMART_QSO_ASSERT(condition, message)

/**
 * @brief Null pointer check macro
 */
#define SMART_QSO_REQUIRE_NOT_NULL(ptr) \
    SMART_QSO_REQUIRE((ptr) != NULL, "Null pointer: " #ptr)

/*===========================================================================*/
/* Time Functions                                                             */
/*===========================================================================*/

/**
 * @brief Get current time in milliseconds
 *
 * @return Current time in milliseconds since epoch
 */
uint64_t smart_qso_now_ms(void);

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
