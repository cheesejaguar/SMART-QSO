/**
 * @file safe_string.h
 * @brief Safe string handling functions interface
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#ifndef SMART_QSO_SAFE_STRING_H
#define SMART_QSO_SAFE_STRING_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/*******************************************************************************
 * Safe String Copy Functions
 ******************************************************************************/

/**
 * @brief Safe string copy with guaranteed null termination
 *
 * @param[out] dest Destination buffer
 * @param[in] dest_size Size of destination buffer
 * @param[in] src Source string
 * @param[out] chars_copied Number of characters copied (optional, may be NULL)
 * @return RESULT_OK on success, RESULT_ERROR_TRUNCATED if source was truncated
 */
SmartQsoResult_t safe_strcpy(char *dest,
                              size_t dest_size,
                              const char *src,
                              size_t *chars_copied);

/**
 * @brief Safe string copy with maximum character limit
 *
 * @param[out] dest Destination buffer
 * @param[in] dest_size Size of destination buffer
 * @param[in] src Source string
 * @param[in] max_chars Maximum characters to copy
 * @param[out] chars_copied Number of characters copied (optional)
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_strncpy(char *dest,
                               size_t dest_size,
                               const char *src,
                               size_t max_chars,
                               size_t *chars_copied);

/*******************************************************************************
 * Safe String Concatenation Functions
 ******************************************************************************/

/**
 * @brief Safe string concatenation
 *
 * @param[in,out] dest Destination buffer (must be null-terminated)
 * @param[in] dest_size Size of destination buffer
 * @param[in] src Source string to append
 * @param[out] total_length Total length after concatenation (optional)
 * @return RESULT_OK on success, RESULT_ERROR_TRUNCATED if source was truncated
 */
SmartQsoResult_t safe_strcat(char *dest,
                              size_t dest_size,
                              const char *src,
                              size_t *total_length);

/*******************************************************************************
 * Safe String Length Functions
 ******************************************************************************/

/**
 * @brief Safe string length with maximum limit
 *
 * @param[in] str String to measure
 * @param[in] max_length Maximum length to check
 * @param[out] length Resulting length
 * @return RESULT_OK on success, RESULT_ERROR_TRUNCATED if no null found
 */
SmartQsoResult_t safe_strlen(const char *str,
                              size_t max_length,
                              size_t *length);

/*******************************************************************************
 * Safe String Comparison Functions
 ******************************************************************************/

/**
 * @brief Safe string comparison with maximum length
 *
 * @param[in] str1 First string
 * @param[in] str2 Second string
 * @param[in] max_length Maximum length to compare
 * @param[out] result Comparison result (-1, 0, or 1)
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_strcmp(const char *str1,
                              const char *str2,
                              size_t max_length,
                              int *result);

/**
 * @brief Safe string comparison for n characters
 *
 * @param[in] str1 First string
 * @param[in] str2 Second string
 * @param[in] n Number of characters to compare
 * @param[out] result Comparison result (-1, 0, or 1)
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_strncmp(const char *str1,
                               const char *str2,
                               size_t n,
                               int *result);

/*******************************************************************************
 * Safe Memory Functions
 ******************************************************************************/

/**
 * @brief Safe memory copy (non-overlapping)
 *
 * @param[out] dest Destination buffer
 * @param[in] dest_size Size of destination buffer
 * @param[in] src Source buffer
 * @param[in] count Number of bytes to copy
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_memcpy(void *dest,
                              size_t dest_size,
                              const void *src,
                              size_t count);

/**
 * @brief Safe memory move (handles overlapping)
 *
 * @param[out] dest Destination buffer
 * @param[in] dest_size Size of destination buffer
 * @param[in] src Source buffer
 * @param[in] count Number of bytes to move
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_memmove(void *dest,
                               size_t dest_size,
                               const void *src,
                               size_t count);

/**
 * @brief Safe memory set
 *
 * @param[out] dest Destination buffer
 * @param[in] dest_size Size of destination buffer
 * @param[in] value Value to set
 * @param[in] count Number of bytes to set
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_memset(void *dest,
                              size_t dest_size,
                              uint8_t value,
                              size_t count);

/**
 * @brief Safe memory comparison
 *
 * @param[in] ptr1 First buffer
 * @param[in] ptr2 Second buffer
 * @param[in] count Number of bytes to compare
 * @param[out] result Comparison result (-1, 0, or 1)
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_memcmp(const void *ptr1,
                              const void *ptr2,
                              size_t count,
                              int *result);

/*******************************************************************************
 * Safe Integer to String Functions
 ******************************************************************************/

/**
 * @brief Safe signed integer to string
 *
 * @param[in] value Integer value
 * @param[out] buffer Output buffer
 * @param[in] buffer_size Size of output buffer
 * @param[out] length Length of resulting string (optional)
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_itoa(int32_t value,
                            char *buffer,
                            size_t buffer_size,
                            size_t *length);

/**
 * @brief Safe unsigned integer to string
 *
 * @param[in] value Unsigned integer value
 * @param[out] buffer Output buffer
 * @param[in] buffer_size Size of output buffer
 * @param[out] length Length of resulting string (optional)
 * @return RESULT_OK on success
 */
SmartQsoResult_t safe_utoa(uint32_t value,
                            char *buffer,
                            size_t buffer_size,
                            size_t *length);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SAFE_STRING_H */
