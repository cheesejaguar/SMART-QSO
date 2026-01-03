/**
 * @file input_validation.c
 * @brief Input validation module for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module provides input validation functions for all external
 * inputs including commands, sensor data, and configuration parameters.
 * Per NPR 7150.2 and MISRA C:2012 guidelines.
 */

#include "input_validation.h"
#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Command validation limits */
#define CMD_MIN_LENGTH          4U
#define CMD_MAX_LENGTH          256U
#define CMD_HEADER_SIZE         4U
#define CMD_CHECKSUM_SIZE       2U

/** Telemetry validation limits */
#define TLM_MIN_VOLTAGE         0.0f
#define TLM_MAX_VOLTAGE         10.0f
#define TLM_MIN_CURRENT         -5.0f
#define TLM_MAX_CURRENT         5.0f
#define TLM_MIN_TEMP            -50.0f
#define TLM_MAX_TEMP            100.0f

/** Sensor validation limits */
#define MAG_MIN_FIELD           -100.0f  /* uT */
#define MAG_MAX_FIELD           100.0f   /* uT */
#define SUN_MIN_VOLTAGE         0.0f     /* V */
#define SUN_MAX_VOLTAGE         3.3f     /* V */

/** Configuration validation */
#define CONFIG_MAX_STRING_LEN   64U
#define CONFIG_MAX_ARRAY_SIZE   32U

/*******************************************************************************
 * Private Types
 ******************************************************************************/

/**
 * @brief Validation result with details
 */
typedef struct {
    bool is_valid;
    uint32_t error_code;
    uint32_t error_offset;
} ValidationResult_t;

/*******************************************************************************
 * Private Function Declarations
 ******************************************************************************/

static bool validate_range_float(float value, float min, float max);
static bool validate_range_uint32(uint32_t value, uint32_t min, uint32_t max);
static bool validate_range_int32(int32_t value, int32_t min, int32_t max);
static uint16_t calculate_checksum(const uint8_t *data, size_t length);

/*******************************************************************************
 * Public Functions - Command Validation
 ******************************************************************************/

SmartQsoResult_t input_validate_command(const uint8_t *cmd_data,
                                        size_t cmd_length,
                                        CommandValidation_t *result)
{
    /* Validate parameters */
    if ((cmd_data == NULL) || (result == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    /* Initialize result */
    result->is_valid = false;
    result->error_code = 0U;
    result->cmd_id = 0U;
    result->payload_length = 0U;

    /* Check minimum length */
    if (cmd_length < CMD_MIN_LENGTH) {
        result->error_code = VALIDATION_ERR_TOO_SHORT;
        return RESULT_OK;
    }

    /* Check maximum length */
    if (cmd_length > CMD_MAX_LENGTH) {
        result->error_code = VALIDATION_ERR_TOO_LONG;
        return RESULT_OK;
    }

    /* Validate sync pattern (first 2 bytes) */
    if ((cmd_data[0] != 0xAA) || (cmd_data[1] != 0x55)) {
        result->error_code = VALIDATION_ERR_SYNC;
        return RESULT_OK;
    }

    /* Extract and validate command ID */
    result->cmd_id = cmd_data[2];
    if (result->cmd_id > CMD_ID_MAX) {
        result->error_code = VALIDATION_ERR_CMD_ID;
        return RESULT_OK;
    }

    /* Extract payload length */
    result->payload_length = cmd_data[3];

    /* Validate total length matches */
    size_t expected_length = CMD_HEADER_SIZE + result->payload_length + CMD_CHECKSUM_SIZE;
    if (cmd_length != expected_length) {
        result->error_code = VALIDATION_ERR_LENGTH;
        return RESULT_OK;
    }

    /* Validate checksum */
    uint16_t received_checksum = (uint16_t)(((uint16_t)cmd_data[cmd_length - 2] << 8) |
                                  (uint16_t)cmd_data[cmd_length - 1]);
    uint16_t calculated_checksum = calculate_checksum(cmd_data, cmd_length - CMD_CHECKSUM_SIZE);

    if (received_checksum != calculated_checksum) {
        result->error_code = VALIDATION_ERR_CHECKSUM;
        return RESULT_OK;
    }

    /* Command is valid */
    result->is_valid = true;
    return RESULT_OK;
}

SmartQsoResult_t input_validate_command_payload(uint8_t cmd_id,
                                                 const uint8_t *payload,
                                                 size_t payload_length,
                                                 bool *is_valid)
{
    /* Validate parameters */
    if ((payload == NULL) || (is_valid == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = false;

    /* Validate based on command type */
    switch (cmd_id) {
        case CMD_ID_SET_MODE:
            /* Mode must be 0-3 */
            if ((payload_length >= 1U) && (payload[0] <= 3U)) {
                *is_valid = true;
            }
            break;

        case CMD_ID_SET_POWER:
            /* Power level must be 0-3 */
            if ((payload_length >= 1U) && (payload[0] <= 3U)) {
                *is_valid = true;
            }
            break;

        case CMD_ID_SET_BEACON:
            /* Beacon interval 10-120 seconds */
            if (payload_length >= 2U) {
                uint16_t interval = (uint16_t)(((uint16_t)payload[0] << 8) | (uint16_t)payload[1]);
                if ((interval >= 10U) && (interval <= 120U)) {
                    *is_valid = true;
                }
            }
            break;

        case CMD_ID_DEPLOY:
            /* Deploy requires authorization code */
            if (payload_length >= 4U) {
                uint32_t auth_code = ((uint32_t)payload[0] << 24) |
                                     ((uint32_t)payload[1] << 16) |
                                     ((uint32_t)payload[2] << 8) |
                                     (uint32_t)payload[3];
                if (auth_code == DEPLOY_AUTH_CODE) {
                    *is_valid = true;
                }
            }
            break;

        case CMD_ID_RESET:
            /* Reset requires confirmation byte */
            if ((payload_length >= 1U) && (payload[0] == 0xFFU)) {
                *is_valid = true;
            }
            break;

        default:
            /* Unknown command */
            break;
    }

    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Sensor Data Validation
 ******************************************************************************/

SmartQsoResult_t input_validate_voltage(float voltage, bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_float(voltage, TLM_MIN_VOLTAGE, TLM_MAX_VOLTAGE);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_current(float current, bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_float(current, TLM_MIN_CURRENT, TLM_MAX_CURRENT);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_temperature(float temperature, bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_float(temperature, TLM_MIN_TEMP, TLM_MAX_TEMP);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_magnetometer(const Vector3_t *mag, bool *is_valid)
{
    if ((mag == NULL) || (is_valid == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_float(mag->x, MAG_MIN_FIELD, MAG_MAX_FIELD) &&
                validate_range_float(mag->y, MAG_MIN_FIELD, MAG_MAX_FIELD) &&
                validate_range_float(mag->z, MAG_MIN_FIELD, MAG_MAX_FIELD);

    return RESULT_OK;
}

SmartQsoResult_t input_validate_sun_sensor(float voltage, bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_float(voltage, SUN_MIN_VOLTAGE, SUN_MAX_VOLTAGE);
    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Configuration Validation
 ******************************************************************************/

SmartQsoResult_t input_validate_config_uint32(uint32_t value,
                                               uint32_t min,
                                               uint32_t max,
                                               bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_uint32(value, min, max);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_config_int32(int32_t value,
                                              int32_t min,
                                              int32_t max,
                                              bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_int32(value, min, max);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_config_float(float value,
                                              float min,
                                              float max,
                                              bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = validate_range_float(value, min, max);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_string(const char *str,
                                        size_t max_length,
                                        bool *is_valid)
{
    if ((str == NULL) || (is_valid == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = false;

    /* Check for null terminator within max length */
    size_t len = 0U;
    while ((len < max_length) && (str[len] != '\0')) {
        /* Check for printable ASCII only */
        if ((str[len] < 0x20) || (str[len] > 0x7E)) {
            return RESULT_OK;  /* Invalid character */
        }
        len++;
    }

    /* String must be null-terminated within bounds */
    if ((len < max_length) && (str[len] == '\0')) {
        *is_valid = true;
    }

    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Buffer Validation
 ******************************************************************************/

SmartQsoResult_t input_validate_buffer(const void *buffer,
                                        size_t buffer_size,
                                        size_t required_size,
                                        bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = (buffer != NULL) && (buffer_size >= required_size);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_pointer(const void *ptr, bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    *is_valid = (ptr != NULL);
    return RESULT_OK;
}

SmartQsoResult_t input_validate_alignment(const void *ptr,
                                           size_t alignment,
                                           bool *is_valid)
{
    if (is_valid == NULL) {
        return RESULT_ERROR_PARAM;
    }

    if ((ptr == NULL) || (alignment == 0U)) {
        *is_valid = false;
        return RESULT_OK;
    }

    /* Check pointer alignment */
    uintptr_t addr = (uintptr_t)ptr;
    *is_valid = ((addr % alignment) == 0U);

    return RESULT_OK;
}

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Validate float value is within range
 */
static bool validate_range_float(float value, float min, float max)
{
    /* Check for NaN */
    if (value != value) {  /* NaN check: value != value is true for NaN */
        return false;
    }

    return (value >= min) && (value <= max);
}

/**
 * @brief Validate uint32 value is within range
 */
static bool validate_range_uint32(uint32_t value, uint32_t min, uint32_t max)
{
    return (value >= min) && (value <= max);
}

/**
 * @brief Validate int32 value is within range
 */
static bool validate_range_int32(int32_t value, int32_t min, int32_t max)
{
    return (value >= min) && (value <= max);
}

/**
 * @brief Calculate 16-bit checksum
 */
static uint16_t calculate_checksum(const uint8_t *data, size_t length)
{
    uint16_t checksum = 0U;

    for (size_t i = 0U; i < length; i++) {
        checksum += (uint16_t)data[i];
    }

    return checksum;
}
