/**
 * @file safe_string.c
 * @brief Safe string handling functions for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module provides safe string functions that prevent buffer overflows
 * and ensure null termination. All functions are designed to be safe for
 * use in flight software per MISRA C:2012 and NASA coding standards.
 */

#include "safe_string.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Public Functions - Safe String Copy
 ******************************************************************************/

SmartQsoResult_t safe_strcpy(char *dest,
                              size_t dest_size,
                              const char *src,
                              size_t *chars_copied)
{
    /* Validate parameters */
    if ((dest == NULL) || (src == NULL) || (dest_size == 0U)) {
        return RESULT_ERROR_PARAM;
    }

    size_t copied = 0U;
    size_t max_copy = dest_size - 1U;  /* Reserve space for null terminator */

    /* Copy characters */
    while ((copied < max_copy) && (src[copied] != '\0')) {
        dest[copied] = src[copied];
        copied++;
    }

    /* Always null-terminate */
    dest[copied] = '\0';

    /* Report characters copied if requested */
    if (chars_copied != NULL) {
        *chars_copied = copied;
    }

    /* Check if source was fully copied */
    if (src[copied] != '\0') {
        return RESULT_ERROR_TRUNCATED;
    }

    return RESULT_OK;
}

SmartQsoResult_t safe_strncpy(char *dest,
                               size_t dest_size,
                               const char *src,
                               size_t max_chars,
                               size_t *chars_copied)
{
    /* Validate parameters */
    if ((dest == NULL) || (src == NULL) || (dest_size == 0U)) {
        return RESULT_ERROR_PARAM;
    }

    size_t copied = 0U;
    size_t max_copy = (dest_size - 1U < max_chars) ? (dest_size - 1U) : max_chars;

    /* Copy characters */
    while ((copied < max_copy) && (src[copied] != '\0')) {
        dest[copied] = src[copied];
        copied++;
    }

    /* Always null-terminate */
    dest[copied] = '\0';

    /* Report characters copied if requested */
    if (chars_copied != NULL) {
        *chars_copied = copied;
    }

    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Safe String Concatenation
 ******************************************************************************/

SmartQsoResult_t safe_strcat(char *dest,
                              size_t dest_size,
                              const char *src,
                              size_t *total_length)
{
    /* Validate parameters */
    if ((dest == NULL) || (src == NULL) || (dest_size == 0U)) {
        return RESULT_ERROR_PARAM;
    }

    /* Find end of destination string */
    size_t dest_len = 0U;
    while ((dest_len < dest_size) && (dest[dest_len] != '\0')) {
        dest_len++;
    }

    /* Check if destination is already full or not null-terminated */
    if (dest_len >= dest_size) {
        return RESULT_ERROR_PARAM;
    }

    /* Calculate remaining space */
    size_t remaining = dest_size - dest_len - 1U;
    size_t copied = 0U;

    /* Append source */
    while ((copied < remaining) && (src[copied] != '\0')) {
        dest[dest_len + copied] = src[copied];
        copied++;
    }

    /* Always null-terminate */
    dest[dest_len + copied] = '\0';

    /* Report total length if requested */
    if (total_length != NULL) {
        *total_length = dest_len + copied;
    }

    /* Check if source was fully copied */
    if (src[copied] != '\0') {
        return RESULT_ERROR_TRUNCATED;
    }

    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Safe String Length
 ******************************************************************************/

SmartQsoResult_t safe_strlen(const char *str,
                              size_t max_length,
                              size_t *length)
{
    /* Validate parameters */
    if ((str == NULL) || (length == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    size_t len = 0U;

    /* Count characters up to max_length */
    while ((len < max_length) && (str[len] != '\0')) {
        len++;
    }

    *length = len;

    /* Check if string is properly terminated */
    if (len == max_length) {
        return RESULT_ERROR_TRUNCATED;
    }

    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Safe String Comparison
 ******************************************************************************/

SmartQsoResult_t safe_strcmp(const char *str1,
                              const char *str2,
                              size_t max_length,
                              int *result)
{
    /* Validate parameters */
    if ((str1 == NULL) || (str2 == NULL) || (result == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    size_t i = 0U;

    while (i < max_length) {
        char c1 = str1[i];
        char c2 = str2[i];

        if (c1 != c2) {
            *result = (c1 < c2) ? -1 : 1;
            return RESULT_OK;
        }

        if (c1 == '\0') {
            *result = 0;
            return RESULT_OK;
        }

        i++;
    }

    /* Reached max_length without finding difference or null */
    *result = 0;
    return RESULT_ERROR_TRUNCATED;
}

SmartQsoResult_t safe_strncmp(const char *str1,
                               const char *str2,
                               size_t n,
                               int *result)
{
    /* Validate parameters */
    if ((str1 == NULL) || (str2 == NULL) || (result == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    if (n == 0U) {
        *result = 0;
        return RESULT_OK;
    }

    size_t i = 0U;

    while (i < n) {
        char c1 = str1[i];
        char c2 = str2[i];

        if (c1 != c2) {
            *result = (c1 < c2) ? -1 : 1;
            return RESULT_OK;
        }

        if (c1 == '\0') {
            *result = 0;
            return RESULT_OK;
        }

        i++;
    }

    *result = 0;
    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Safe Memory Operations
 ******************************************************************************/

SmartQsoResult_t safe_memcpy(void *dest,
                              size_t dest_size,
                              const void *src,
                              size_t count)
{
    /* Validate parameters */
    if ((dest == NULL) || (src == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    if (count > dest_size) {
        return RESULT_ERROR_PARAM;
    }

    /* Check for overlap */
    const uint8_t *s = (const uint8_t *)src;
    uint8_t *d = (uint8_t *)dest;

    if (((d >= s) && (d < (s + count))) ||
        ((s >= d) && (s < (d + count)))) {
        return RESULT_ERROR_PARAM;  /* Overlapping buffers */
    }

    /* Copy bytes */
    for (size_t i = 0U; i < count; i++) {
        d[i] = s[i];
    }

    return RESULT_OK;
}

SmartQsoResult_t safe_memmove(void *dest,
                               size_t dest_size,
                               const void *src,
                               size_t count)
{
    /* Validate parameters */
    if ((dest == NULL) || (src == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    if (count > dest_size) {
        return RESULT_ERROR_PARAM;
    }

    const uint8_t *s = (const uint8_t *)src;
    uint8_t *d = (uint8_t *)dest;

    /* Handle overlapping regions */
    if (d < s) {
        /* Copy forward */
        for (size_t i = 0U; i < count; i++) {
            d[i] = s[i];
        }
    } else if (d > s) {
        /* Copy backward */
        for (size_t i = count; i > 0U; i--) {
            d[i - 1U] = s[i - 1U];
        }
    }
    /* If d == s, no copy needed */

    return RESULT_OK;
}

SmartQsoResult_t safe_memset(void *dest,
                              size_t dest_size,
                              uint8_t value,
                              size_t count)
{
    /* Validate parameters */
    if (dest == NULL) {
        return RESULT_ERROR_PARAM;
    }

    if (count > dest_size) {
        return RESULT_ERROR_PARAM;
    }

    uint8_t *d = (uint8_t *)dest;

    for (size_t i = 0U; i < count; i++) {
        d[i] = value;
    }

    return RESULT_OK;
}

SmartQsoResult_t safe_memcmp(const void *ptr1,
                              const void *ptr2,
                              size_t count,
                              int *result)
{
    /* Validate parameters */
    if ((ptr1 == NULL) || (ptr2 == NULL) || (result == NULL)) {
        return RESULT_ERROR_PARAM;
    }

    const uint8_t *p1 = (const uint8_t *)ptr1;
    const uint8_t *p2 = (const uint8_t *)ptr2;

    for (size_t i = 0U; i < count; i++) {
        if (p1[i] != p2[i]) {
            *result = (p1[i] < p2[i]) ? -1 : 1;
            return RESULT_OK;
        }
    }

    *result = 0;
    return RESULT_OK;
}

/*******************************************************************************
 * Public Functions - Safe Integer to String
 ******************************************************************************/

SmartQsoResult_t safe_itoa(int32_t value,
                            char *buffer,
                            size_t buffer_size,
                            size_t *length)
{
    /* Validate parameters */
    if ((buffer == NULL) || (buffer_size < 2U)) {
        return RESULT_ERROR_PARAM;
    }

    char temp[12];  /* -2147483648 is 11 chars + null */
    size_t pos = 0U;
    bool negative = false;
    uint32_t uvalue;

    /* Handle negative numbers */
    if (value < 0) {
        negative = true;
        uvalue = (uint32_t)(-(value + 1)) + 1U;  /* Safe negation */
    } else {
        uvalue = (uint32_t)value;
    }

    /* Convert digits in reverse order */
    do {
        temp[pos++] = (char)('0' + (uvalue % 10U));
        uvalue /= 10U;
    } while (uvalue > 0U);

    /* Add sign */
    if (negative) {
        temp[pos++] = '-';
    }

    /* Check buffer size */
    if (pos >= buffer_size) {
        buffer[0] = '\0';
        return RESULT_ERROR_TRUNCATED;
    }

    /* Reverse into output buffer */
    size_t out = 0U;
    while (pos > 0U) {
        buffer[out++] = temp[--pos];
    }
    buffer[out] = '\0';

    if (length != NULL) {
        *length = out;
    }

    return RESULT_OK;
}

SmartQsoResult_t safe_utoa(uint32_t value,
                            char *buffer,
                            size_t buffer_size,
                            size_t *length)
{
    /* Validate parameters */
    if ((buffer == NULL) || (buffer_size < 2U)) {
        return RESULT_ERROR_PARAM;
    }

    char temp[11];  /* 4294967295 is 10 chars + null */
    size_t pos = 0U;

    /* Convert digits in reverse order */
    do {
        temp[pos++] = (char)('0' + (value % 10U));
        value /= 10U;
    } while (value > 0U);

    /* Check buffer size */
    if (pos >= buffer_size) {
        buffer[0] = '\0';
        return RESULT_ERROR_TRUNCATED;
    }

    /* Reverse into output buffer */
    size_t out = 0U;
    while (pos > 0U) {
        buffer[out++] = temp[--pos];
    }
    buffer[out] = '\0';

    if (length != NULL) {
        *length = out;
    }

    return RESULT_OK;
}
