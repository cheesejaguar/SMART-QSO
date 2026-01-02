/**
 * @file sensors.h
 * @brief Sensor Management Interface
 *
 * This module provides sensor framework functionality including sensor
 * registration, polling, data conversion, and YAML configuration.
 *
 * @requirement SRS-SENS-001 System shall support configurable sensor framework
 * @requirement SRS-SENS-002 System shall poll sensors at configurable intervals
 * @requirement SRS-SENS-003 System shall validate sensor readings
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#ifndef SMART_QSO_SENSORS_H
#define SMART_QSO_SENSORS_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* Forward Declarations                                                       */
/*===========================================================================*/

struct Sensor;

/*===========================================================================*/
/* Sensor Callback Types                                                      */
/*===========================================================================*/

/**
 * @brief Sensor read function pointer type
 *
 * @param self      Pointer to sensor structure
 * @param out_value Output for numeric value
 * @param out_text  Output for text value (8 bytes)
 * @return true on success, false on failure
 */
typedef bool (*SensorReadFn_t)(struct Sensor *self, double *out_value, char *out_text);

/*===========================================================================*/
/* Sensor Structure                                                           */
/*===========================================================================*/

/**
 * @brief Sensor Definition
 *
 * Holds complete definition and state for a single sensor.
 */
typedef struct Sensor {
    char id[SMART_QSO_SENSOR_ID_LEN];         /**< Short identifier (e.g., "BV") */
    char name[SMART_QSO_SENSOR_NAME_LEN];     /**< Full name */
    char units[SMART_QSO_SENSOR_UNITS_LEN];   /**< Units string */
    char type[SMART_QSO_SENSOR_TYPE_LEN];     /**< Sensor type for binding */
    char channel[SMART_QSO_SENSOR_CHANNEL_LEN]; /**< Channel identifier */
    uint32_t period_ms;                        /**< Poll period in ms */
    uint64_t next_poll_ms;                     /**< Next poll time */
    SensorValueType_t value_type;              /**< Type of value */
    SensorReadFn_t read;                       /**< Read function pointer */
    double last_value;                         /**< Last numeric reading */
    char last_text[8];                         /**< Last text reading */
} Sensor_t;

/*===========================================================================*/
/* Sensor Management API                                                      */
/*===========================================================================*/

/**
 * @brief Initialize sensor subsystem
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre None
 * @post Sensor subsystem is ready for configuration
 */
SmartQsoResult_t sensors_init(void);

/**
 * @brief Load sensor configuration from YAML file
 *
 * @param path Path to YAML configuration file
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre path != NULL
 * @post Sensors are configured from file
 */
SmartQsoResult_t sensors_load_yaml(const char *path);

/**
 * @brief Load default sensor configuration
 *
 * Used when YAML file is not available.
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t sensors_load_defaults(void);

/**
 * @brief Get number of registered sensors
 *
 * @return Number of sensors
 */
size_t sensors_get_count(void);

/**
 * @brief Get sensor by index
 *
 * @param index Sensor index (0-based)
 * @param[out] sensor Pointer to sensor structure to fill
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre sensor != NULL
 * @pre index < sensors_get_count()
 */
SmartQsoResult_t sensors_get(size_t index, Sensor_t *sensor);

/**
 * @brief Get sensor by ID
 *
 * @param id Sensor ID string
 * @param[out] sensor Pointer to sensor structure to fill
 * @return SMART_QSO_OK on success, SMART_QSO_ERROR if not found
 *
 * @pre id != NULL
 * @pre sensor != NULL
 */
SmartQsoResult_t sensors_get_by_id(const char *id, Sensor_t *sensor);

/**
 * @brief Poll all sensors that are due for reading
 *
 * @param current_ms Current time in milliseconds
 * @return Number of sensors polled
 */
size_t sensors_poll(uint64_t current_ms);

/**
 * @brief Poll a specific sensor
 *
 * @param index Sensor index
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t sensors_poll_one(size_t index);

/**
 * @brief Set external state variables for sensor simulation
 *
 * @param sunlit  Whether spacecraft is in sunlight
 * @param soc     Current state of charge (0.0-1.0)
 */
void sensors_set_environment(bool sunlit, double soc);

/**
 * @brief Format all sensor values as telemetry string
 *
 * @param[out] buffer     Output buffer
 * @param      buffer_len Size of output buffer
 * @return Number of bytes written (excluding null terminator)
 *
 * @pre buffer != NULL
 * @pre buffer_len > 0
 */
size_t sensors_format_telemetry(char *buffer, size_t buffer_len);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SENSORS_H */
