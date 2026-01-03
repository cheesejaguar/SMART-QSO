/**
 * @file input_validation.h
 * @brief Input validation module interface for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#ifndef SMART_QSO_INPUT_VALIDATION_H
#define SMART_QSO_INPUT_VALIDATION_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/*******************************************************************************
 * Public Constants
 ******************************************************************************/

/** Validation error codes */
#define VALIDATION_ERR_NONE         0U
#define VALIDATION_ERR_TOO_SHORT    1U
#define VALIDATION_ERR_TOO_LONG     2U
#define VALIDATION_ERR_SYNC         3U
#define VALIDATION_ERR_CMD_ID       4U
#define VALIDATION_ERR_LENGTH       5U
#define VALIDATION_ERR_CHECKSUM     6U
#define VALIDATION_ERR_PAYLOAD      7U

/** Command IDs */
#define CMD_ID_NOP              0x00U
#define CMD_ID_SET_MODE         0x01U
#define CMD_ID_SET_POWER        0x02U
#define CMD_ID_SET_BEACON       0x03U
#define CMD_ID_DEPLOY           0x10U
#define CMD_ID_RESET            0xFFU
#define CMD_ID_MAX              0xFFU

/** Authorization codes */
#define DEPLOY_AUTH_CODE        0xDEADBEEFU

/*******************************************************************************
 * Public Types
 ******************************************************************************/

/**
 * @brief 3D vector structure
 */
typedef struct {
    float x;
    float y;
    float z;
} Vector3_t;

/**
 * @brief Command validation result
 */
typedef struct {
    bool is_valid;          /**< True if command is valid */
    uint32_t error_code;    /**< Error code if invalid */
    uint8_t cmd_id;         /**< Command ID */
    uint8_t payload_length; /**< Payload length */
} CommandValidation_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Validate command structure and checksum
 *
 * @param[in] cmd_data Command data buffer
 * @param[in] cmd_length Length of command data
 * @param[out] result Validation result
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_command(const uint8_t *cmd_data,
                                        size_t cmd_length,
                                        CommandValidation_t *result);

/**
 * @brief Validate command payload for specific command
 *
 * @param[in] cmd_id Command ID
 * @param[in] payload Payload data
 * @param[in] payload_length Payload length
 * @param[out] is_valid True if payload is valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_command_payload(uint8_t cmd_id,
                                                 const uint8_t *payload,
                                                 size_t payload_length,
                                                 bool *is_valid);

/**
 * @brief Validate voltage reading
 *
 * @param[in] voltage Voltage value
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_voltage(float voltage, bool *is_valid);

/**
 * @brief Validate current reading
 *
 * @param[in] current Current value
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_current(float current, bool *is_valid);

/**
 * @brief Validate temperature reading
 *
 * @param[in] temperature Temperature value
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_temperature(float temperature, bool *is_valid);

/**
 * @brief Validate magnetometer reading
 *
 * @param[in] mag Magnetometer vector
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_magnetometer(const Vector3_t *mag, bool *is_valid);

/**
 * @brief Validate sun sensor reading
 *
 * @param[in] voltage Sun sensor voltage
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_sun_sensor(float voltage, bool *is_valid);

/**
 * @brief Validate configuration uint32 value
 *
 * @param[in] value Value to validate
 * @param[in] min Minimum allowed value
 * @param[in] max Maximum allowed value
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_config_uint32(uint32_t value,
                                               uint32_t min,
                                               uint32_t max,
                                               bool *is_valid);

/**
 * @brief Validate configuration int32 value
 *
 * @param[in] value Value to validate
 * @param[in] min Minimum allowed value
 * @param[in] max Maximum allowed value
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_config_int32(int32_t value,
                                              int32_t min,
                                              int32_t max,
                                              bool *is_valid);

/**
 * @brief Validate configuration float value
 *
 * @param[in] value Value to validate
 * @param[in] min Minimum allowed value
 * @param[in] max Maximum allowed value
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_config_float(float value,
                                              float min,
                                              float max,
                                              bool *is_valid);

/**
 * @brief Validate string (printable ASCII, null-terminated)
 *
 * @param[in] str String to validate
 * @param[in] max_length Maximum allowed length
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_string(const char *str,
                                        size_t max_length,
                                        bool *is_valid);

/**
 * @brief Validate buffer size
 *
 * @param[in] buffer Buffer pointer
 * @param[in] buffer_size Actual buffer size
 * @param[in] required_size Required minimum size
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_buffer(const void *buffer,
                                        size_t buffer_size,
                                        size_t required_size,
                                        bool *is_valid);

/**
 * @brief Validate pointer is non-NULL
 *
 * @param[in] ptr Pointer to validate
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_pointer(const void *ptr, bool *is_valid);

/**
 * @brief Validate pointer alignment
 *
 * @param[in] ptr Pointer to validate
 * @param[in] alignment Required alignment
 * @param[out] is_valid True if valid
 * @return RESULT_OK on success
 */
SmartQsoResult_t input_validate_alignment(const void *ptr,
                                           size_t alignment,
                                           bool *is_valid);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_INPUT_VALIDATION_H */
