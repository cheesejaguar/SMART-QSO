/**
 * @file yaml_parser.h
 * @brief Simple YAML configuration parser for SMART-QSO CubeSat
 *
 * This module provides a lightweight YAML parser for reading configuration
 * files. It supports a subset of YAML suitable for embedded systems.
 *
 * @note MISRA C:2012 compliant
 * @note Memory-safe with bounded operations
 *
 * @author SMART-QSO Team
 * @date 2026-01-02
 * @version 1.0
 */

#ifndef YAML_PARSER_H
#define YAML_PARSER_H

#ifdef __cplusplus
extern "C" {
#endif

/*******************************************************************************
 * Includes
 ******************************************************************************/

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/*******************************************************************************
 * Definitions
 ******************************************************************************/

/** Maximum key path length */
#define YAML_MAX_PATH_LENGTH    256U

/** Maximum value length */
#define YAML_MAX_VALUE_LEN      128U

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief YAML parser error codes
 */
typedef enum {
    YAML_OK = 0,                /**< Success */
    YAML_ERROR_INVALID_PARAM,   /**< Invalid parameter */
    YAML_ERROR_PARSE,           /**< Parse error */
    YAML_ERROR_MEMORY,          /**< Memory allocation error */
    YAML_ERROR_DEPTH,           /**< Maximum nesting depth exceeded */
    YAML_ERROR_SYNTAX           /**< Syntax error */
} yaml_error_t;

/**
 * @brief Callback function type for parsed key-value pairs
 *
 * @param path Full path to the key (e.g., "sensors.magnetometer.rate")
 * @param value Value as a string
 * @param user_data User-provided context
 */
typedef void (*yaml_callback_t)(const char *path, const char *value, void *user_data);

/*******************************************************************************
 * Public Function Prototypes
 ******************************************************************************/

/**
 * @brief Initialize the YAML parser
 *
 * Resets the parser state and prepares for parsing.
 *
 * @return YAML_OK on success
 */
yaml_error_t yaml_parser_init(void);

/**
 * @brief Register a callback for parsed key-value pairs
 *
 * The callback will be invoked for each key-value pair found in the YAML.
 *
 * @param callback Function to call for each key-value pair
 * @param user_data User data to pass to callback (can be NULL)
 * @return YAML_OK on success, YAML_ERROR_INVALID_PARAM if callback is NULL
 */
yaml_error_t yaml_register_callback(yaml_callback_t callback, void *user_data);

/**
 * @brief Parse a YAML string
 *
 * Parses the provided null-terminated YAML content and invokes the
 * registered callback for each key-value pair.
 *
 * @param yaml_string Null-terminated YAML content
 * @param length Length of the string
 * @return YAML_OK on success, error code otherwise
 */
yaml_error_t yaml_parse_string(const char *yaml_string, size_t length);

/**
 * @brief Parse a YAML buffer from memory
 *
 * Convenience wrapper for yaml_parse_string that accepts uint8_t buffer.
 *
 * @param buffer Pointer to YAML content
 * @param length Length of buffer
 * @return YAML_OK on success
 */
yaml_error_t yaml_parse_buffer(const uint8_t *buffer, size_t length);

/**
 * @brief Get the last parser error
 *
 * @return Last error code
 */
yaml_error_t yaml_get_last_error(void);

/**
 * @brief Get the line number of the last error
 *
 * @return Line number (1-based), 0 if no error
 */
uint32_t yaml_get_error_line(void);

/**
 * @brief Convert error code to string
 *
 * @param error Error code
 * @return Error description string (never NULL)
 */
const char *yaml_error_to_string(yaml_error_t error);

/*******************************************************************************
 * Inline Helper Functions
 ******************************************************************************/

/**
 * @brief Parse integer value from YAML string
 *
 * @param value Value string
 * @param result Pointer to store result
 * @return true if parsing succeeded
 */
static inline bool yaml_parse_int(const char *value, int32_t *result)
{
    if (value == NULL || result == NULL) {
        return false;
    }

    char *endptr;
    long val = strtol(value, &endptr, 10);

    if (endptr == value || *endptr != '\0') {
        return false;
    }

    *result = (int32_t)val;
    return true;
}

/**
 * @brief Parse unsigned integer value from YAML string
 *
 * @param value Value string
 * @param result Pointer to store result
 * @return true if parsing succeeded
 */
static inline bool yaml_parse_uint(const char *value, uint32_t *result)
{
    if (value == NULL || result == NULL) {
        return false;
    }

    char *endptr;
    unsigned long val = strtoul(value, &endptr, 10);

    if (endptr == value || *endptr != '\0') {
        return false;
    }

    *result = (uint32_t)val;
    return true;
}

/**
 * @brief Parse boolean value from YAML string
 *
 * Accepts: true, false, yes, no, on, off, 1, 0
 *
 * @param value Value string
 * @param result Pointer to store result
 * @return true if parsing succeeded
 */
static inline bool yaml_parse_bool(const char *value, bool *result)
{
    if (value == NULL || result == NULL) {
        return false;
    }

    if (strcmp(value, "true") == 0 || strcmp(value, "yes") == 0 ||
        strcmp(value, "on") == 0 || strcmp(value, "1") == 0) {
        *result = true;
        return true;
    }

    if (strcmp(value, "false") == 0 || strcmp(value, "no") == 0 ||
        strcmp(value, "off") == 0 || strcmp(value, "0") == 0) {
        *result = false;
        return true;
    }

    return false;
}

/**
 * @brief Parse floating point value from YAML string
 *
 * @param value Value string
 * @param result Pointer to store result
 * @return true if parsing succeeded
 */
static inline bool yaml_parse_float(const char *value, float *result)
{
    if (value == NULL || result == NULL) {
        return false;
    }

    char *endptr;
    float val = strtof(value, &endptr);

    if (endptr == value || *endptr != '\0') {
        return false;
    }

    *result = val;
    return true;
}

#ifdef __cplusplus
}
#endif

#endif /* YAML_PARSER_H */
