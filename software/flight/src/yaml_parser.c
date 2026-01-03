/**
 * @file yaml_parser.c
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

#include "yaml_parser.h"
#include "safe_string.h"
#include "crc32.h"
#include <string.h>
#include <stdlib.h>
#include <ctype.h>

/*******************************************************************************
 * Private Definitions
 ******************************************************************************/

/** Maximum line length in YAML file */
#define YAML_MAX_LINE_LENGTH    256U

/** Maximum key length */
#define YAML_MAX_KEY_LENGTH     64U

/** Maximum value length */
#define YAML_MAX_VALUE_LENGTH   128U

/** Maximum nesting depth */
#define YAML_MAX_DEPTH          8U

/** Comment character */
#define YAML_COMMENT_CHAR       '#'

/** Key-value separator */
#define YAML_SEPARATOR          ':'

/*******************************************************************************
 * Private Types
 ******************************************************************************/

/**
 * @brief Parser state machine states
 */
typedef enum {
    YAML_STATE_IDLE,
    YAML_STATE_KEY,
    YAML_STATE_VALUE,
    YAML_STATE_COMMENT,
    YAML_STATE_ERROR
} yaml_state_t;

/**
 * @brief Parser context
 */
typedef struct {
    yaml_state_t state;
    uint8_t indent_level;
    uint8_t indent_stack[YAML_MAX_DEPTH];
    char current_key[YAML_MAX_KEY_LENGTH];
    char current_value[YAML_MAX_VALUE_LENGTH];
    char path[YAML_MAX_KEY_LENGTH * YAML_MAX_DEPTH];
    uint32_t line_number;
    yaml_error_t last_error;
} yaml_context_t;

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Parser context */
static yaml_context_t g_yaml_ctx;

/** Registered callback */
static yaml_callback_t g_yaml_callback = NULL;

/** User data for callback */
static void *g_yaml_user_data = NULL;

/*******************************************************************************
 * Private Function Prototypes
 ******************************************************************************/

static yaml_error_t yaml_parse_line(const char *line);
static uint8_t yaml_count_indent(const char *line);
static yaml_error_t yaml_extract_key_value(const char *line, char *key, char *value);
static void yaml_trim_whitespace(char *str);
static bool yaml_is_comment_or_empty(const char *line);
static void yaml_update_path(uint8_t new_indent);

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

/**
 * @brief Initialize the YAML parser
 *
 * @return YAML_OK on success
 */
yaml_error_t yaml_parser_init(void)
{
    memset(&g_yaml_ctx, 0, sizeof(g_yaml_ctx));
    g_yaml_ctx.state = YAML_STATE_IDLE;
    g_yaml_callback = NULL;
    g_yaml_user_data = NULL;

    return YAML_OK;
}

/**
 * @brief Register a callback for parsed key-value pairs
 *
 * @param callback Function to call for each key-value pair
 * @param user_data User data to pass to callback
 * @return YAML_OK on success
 */
yaml_error_t yaml_register_callback(yaml_callback_t callback, void *user_data)
{
    if (callback == NULL) {
        return YAML_ERROR_INVALID_PARAM;
    }

    g_yaml_callback = callback;
    g_yaml_user_data = user_data;

    return YAML_OK;
}

/**
 * @brief Parse a YAML string
 *
 * @param yaml_string Null-terminated YAML content
 * @param length Length of the string
 * @return YAML_OK on success, error code otherwise
 */
yaml_error_t yaml_parse_string(const char *yaml_string, size_t length)
{
    if (yaml_string == NULL) {
        return YAML_ERROR_INVALID_PARAM;
    }

    if (length == 0U) {
        return YAML_ERROR_INVALID_PARAM;
    }

    /* Reset parser state */
    g_yaml_ctx.state = YAML_STATE_IDLE;
    g_yaml_ctx.indent_level = 0U;
    g_yaml_ctx.line_number = 0U;
    g_yaml_ctx.path[0] = '\0';
    memset(g_yaml_ctx.indent_stack, 0, sizeof(g_yaml_ctx.indent_stack));

    char line_buffer[YAML_MAX_LINE_LENGTH];
    size_t line_start = 0U;
    size_t i;

    for (i = 0U; i <= length; i++) {
        /* Check for end of line or end of string */
        if ((i == length) || (yaml_string[i] == '\n') || (yaml_string[i] == '\r')) {
            size_t line_len = i - line_start;

            if (line_len > 0U) {
                /* Copy line to buffer */
                if (line_len >= YAML_MAX_LINE_LENGTH) {
                    line_len = YAML_MAX_LINE_LENGTH - 1U;
                }
                memcpy(line_buffer, &yaml_string[line_start], line_len);
                line_buffer[line_len] = '\0';

                /* Parse the line */
                g_yaml_ctx.line_number++;
                yaml_error_t err = yaml_parse_line(line_buffer);
                if (err != YAML_OK) {
                    g_yaml_ctx.last_error = err;
                    return err;
                }
            }

            /* Skip carriage return if followed by newline */
            if ((i < length) && (yaml_string[i] == '\r') &&
                ((i + 1U) < length) && (yaml_string[i + 1U] == '\n')) {
                i++;
            }

            line_start = i + 1U;
        }
    }

    return YAML_OK;
}

/**
 * @brief Parse a YAML buffer from memory
 *
 * @param buffer Pointer to YAML content
 * @param length Length of buffer
 * @return YAML_OK on success
 */
yaml_error_t yaml_parse_buffer(const uint8_t *buffer, size_t length)
{
    if (buffer == NULL) {
        return YAML_ERROR_INVALID_PARAM;
    }

    return yaml_parse_string((const char *)buffer, length);
}

/**
 * @brief Get the last parser error
 *
 * @return Last error code
 */
yaml_error_t yaml_get_last_error(void)
{
    return g_yaml_ctx.last_error;
}

/**
 * @brief Get the line number of the last error
 *
 * @return Line number (1-based)
 */
uint32_t yaml_get_error_line(void)
{
    return g_yaml_ctx.line_number;
}

/**
 * @brief Convert error code to string
 *
 * @param error Error code
 * @return Error description string
 */
const char *yaml_error_to_string(yaml_error_t error)
{
    switch (error) {
        case YAML_OK:
            return "No error";
        case YAML_ERROR_INVALID_PARAM:
            return "Invalid parameter";
        case YAML_ERROR_PARSE:
            return "Parse error";
        case YAML_ERROR_MEMORY:
            return "Memory error";
        case YAML_ERROR_DEPTH:
            return "Maximum nesting depth exceeded";
        case YAML_ERROR_SYNTAX:
            return "Syntax error";
        default:
            return "Unknown error";
    }
}

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Parse a single line of YAML
 */
static yaml_error_t yaml_parse_line(const char *line)
{
    /* Check for comment or empty line */
    if (yaml_is_comment_or_empty(line)) {
        return YAML_OK;
    }

    /* Count indentation */
    uint8_t indent = yaml_count_indent(line);

    /* Update path based on indentation */
    yaml_update_path(indent);

    /* Extract key and value */
    char key[YAML_MAX_KEY_LENGTH];
    char value[YAML_MAX_VALUE_LENGTH];

    yaml_error_t err = yaml_extract_key_value(line, key, value);
    if (err != YAML_OK) {
        return err;
    }

    /* Build full path */
    char full_path[YAML_MAX_KEY_LENGTH * YAML_MAX_DEPTH];
    if (g_yaml_ctx.path[0] != '\0') {
        snprintf(full_path, sizeof(full_path), "%s.%s", g_yaml_ctx.path, key);
    } else {
        strncpy(full_path, key, sizeof(full_path) - 1U);
        full_path[sizeof(full_path) - 1U] = '\0';
    }

    /* If value is empty, this is a parent key - update path */
    if (value[0] == '\0') {
        strncpy(g_yaml_ctx.path, full_path, sizeof(g_yaml_ctx.path) - 1U);
        g_yaml_ctx.path[sizeof(g_yaml_ctx.path) - 1U] = '\0';
        g_yaml_ctx.indent_level = indent;
        if (indent < YAML_MAX_DEPTH) {
            g_yaml_ctx.indent_stack[indent] = 1U;
        }
    } else {
        /* Call callback with key-value pair */
        if (g_yaml_callback != NULL) {
            g_yaml_callback(full_path, value, g_yaml_user_data);
        }
    }

    return YAML_OK;
}

/**
 * @brief Count leading spaces/tabs for indentation
 */
static uint8_t yaml_count_indent(const char *line)
{
    uint8_t indent = 0U;

    while ((*line == ' ') || (*line == '\t')) {
        if (*line == '\t') {
            indent += 2U;  /* Treat tab as 2 spaces */
        } else {
            indent++;
        }
        line++;
    }

    /* Normalize to indent levels (2 spaces = 1 level) */
    return indent / 2U;
}

/**
 * @brief Extract key and value from a line
 */
static yaml_error_t yaml_extract_key_value(const char *line, char *key, char *value)
{
    /* Skip leading whitespace */
    while ((*line == ' ') || (*line == '\t')) {
        line++;
    }

    /* Find separator */
    const char *sep = strchr(line, YAML_SEPARATOR);
    if (sep == NULL) {
        return YAML_ERROR_SYNTAX;
    }

    /* Extract key */
    size_t key_len = (size_t)(sep - line);
    if (key_len >= YAML_MAX_KEY_LENGTH) {
        key_len = YAML_MAX_KEY_LENGTH - 1U;
    }
    memcpy(key, line, key_len);
    key[key_len] = '\0';
    yaml_trim_whitespace(key);

    /* Extract value */
    const char *val_start = sep + 1;
    while (*val_start == ' ') {
        val_start++;
    }

    strncpy(value, val_start, YAML_MAX_VALUE_LENGTH - 1U);
    value[YAML_MAX_VALUE_LENGTH - 1U] = '\0';
    yaml_trim_whitespace(value);

    /* Remove quotes if present */
    size_t val_len = strlen(value);
    if (val_len >= 2U) {
        if ((value[0] == '"' && value[val_len - 1U] == '"') ||
            (value[0] == '\'' && value[val_len - 1U] == '\'')) {
            memmove(value, value + 1, val_len - 2U);
            value[val_len - 2U] = '\0';
        }
    }

    return YAML_OK;
}

/**
 * @brief Trim leading and trailing whitespace
 */
static void yaml_trim_whitespace(char *str)
{
    if (str == NULL || str[0] == '\0') {
        return;
    }

    /* Trim trailing */
    size_t len = strlen(str);
    while (len > 0U && (str[len - 1U] == ' ' || str[len - 1U] == '\t' ||
                        str[len - 1U] == '\r' || str[len - 1U] == '\n')) {
        str[--len] = '\0';
    }

    /* Trim leading */
    char *start = str;
    while (*start == ' ' || *start == '\t') {
        start++;
    }

    if (start != str) {
        memmove(str, start, strlen(start) + 1U);
    }
}

/**
 * @brief Check if line is a comment or empty
 */
static bool yaml_is_comment_or_empty(const char *line)
{
    /* Skip whitespace */
    while (*line == ' ' || *line == '\t') {
        line++;
    }

    /* Check for empty or comment */
    return (*line == '\0' || *line == '\n' || *line == '\r' ||
            *line == YAML_COMMENT_CHAR);
}

/**
 * @brief Update the current path based on new indentation level
 */
static void yaml_update_path(uint8_t new_indent)
{
    if (new_indent < g_yaml_ctx.indent_level) {
        /* Going back up - truncate path */
        char *last_dot = NULL;
        uint8_t levels_to_remove = g_yaml_ctx.indent_level - new_indent;

        char *p = g_yaml_ctx.path;
        while (levels_to_remove > 0U && (last_dot = strrchr(p, '.')) != NULL) {
            *last_dot = '\0';
            levels_to_remove--;
        }

        if (levels_to_remove > 0U) {
            g_yaml_ctx.path[0] = '\0';
        }
    }

    g_yaml_ctx.indent_level = new_indent;
}
