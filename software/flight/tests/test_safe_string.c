/**
 * @file test_safe_string.c
 * @brief Unit tests for safe string module
 *
 * Tests safe string and memory functions per NPR 7150.2 and MISRA C:2012.
 *
 * @requirement SRS-SEC-003 Safe string handling to prevent buffer overflows
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

/* Include the module under test */
#include "safe_string.h"

/*===========================================================================*/
/* Safe String Copy Tests                                                     */
/*===========================================================================*/

/**
 * @brief Test safe_strcpy with normal string
 */
static void test_safe_strcpy_normal(void **state)
{
    (void)state;

    char dest[32];
    size_t copied;

    SmartQsoResult_t ret = safe_strcpy(dest, sizeof(dest), "Hello", &copied);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(dest, "Hello");
    assert_int_equal(copied, 5);
}

/**
 * @brief Test safe_strcpy guarantees null termination
 */
static void test_safe_strcpy_truncation(void **state)
{
    (void)state;

    char dest[8];
    size_t copied;

    SmartQsoResult_t ret = safe_strcpy(dest, sizeof(dest), "Hello, World!", &copied);

    assert_int_equal(ret, RESULT_ERROR_TRUNCATED);
    assert_int_equal(copied, 7);  /* 8 - 1 for null */
    assert_int_equal(dest[7], '\0');  /* Null terminated */
    assert_memory_equal(dest, "Hello, ", 7);
}

/**
 * @brief Test safe_strcpy with empty string
 */
static void test_safe_strcpy_empty(void **state)
{
    (void)state;

    char dest[32] = "garbage";
    size_t copied;

    SmartQsoResult_t ret = safe_strcpy(dest, sizeof(dest), "", &copied);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(dest, "");
    assert_int_equal(copied, 0);
}

/**
 * @brief Test safe_strcpy with NULL parameters
 */
static void test_safe_strcpy_null_params(void **state)
{
    (void)state;

    char dest[32];

    /* NULL destination */
    SmartQsoResult_t ret = safe_strcpy(NULL, 32, "test", NULL);
    assert_int_equal(ret, RESULT_ERROR_PARAM);

    /* NULL source */
    ret = safe_strcpy(dest, 32, NULL, NULL);
    assert_int_equal(ret, RESULT_ERROR_PARAM);

    /* Zero size */
    ret = safe_strcpy(dest, 0, "test", NULL);
    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test safe_strncpy with max chars limit
 */
static void test_safe_strncpy_limit(void **state)
{
    (void)state;

    char dest[32];
    size_t copied;

    SmartQsoResult_t ret = safe_strncpy(dest, sizeof(dest), "Hello, World!", 5, &copied);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(dest, "Hello");
    assert_int_equal(copied, 5);
}

/*===========================================================================*/
/* Safe String Concatenation Tests                                            */
/*===========================================================================*/

/**
 * @brief Test safe_strcat normal operation
 */
static void test_safe_strcat_normal(void **state)
{
    (void)state;

    char dest[32] = "Hello";
    size_t total;

    SmartQsoResult_t ret = safe_strcat(dest, sizeof(dest), ", World!", &total);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(dest, "Hello, World!");
    assert_int_equal(total, 13);
}

/**
 * @brief Test safe_strcat with truncation
 */
static void test_safe_strcat_truncation(void **state)
{
    (void)state;

    char dest[12] = "Hello";
    size_t total;

    SmartQsoResult_t ret = safe_strcat(dest, sizeof(dest), ", World!", &total);

    assert_int_equal(ret, RESULT_ERROR_TRUNCATED);
    assert_int_equal(dest[11], '\0');  /* Null terminated */
    assert_int_equal(total, 11);
}

/**
 * @brief Test safe_strcat to empty string
 */
static void test_safe_strcat_to_empty(void **state)
{
    (void)state;

    char dest[32] = "";
    size_t total;

    SmartQsoResult_t ret = safe_strcat(dest, sizeof(dest), "Hello", &total);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(dest, "Hello");
    assert_int_equal(total, 5);
}

/*===========================================================================*/
/* Safe String Length Tests                                                   */
/*===========================================================================*/

/**
 * @brief Test safe_strlen normal operation
 */
static void test_safe_strlen_normal(void **state)
{
    (void)state;

    size_t length;

    SmartQsoResult_t ret = safe_strlen("Hello", 100, &length);

    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(length, 5);
}

/**
 * @brief Test safe_strlen with max limit reached
 */
static void test_safe_strlen_max_reached(void **state)
{
    (void)state;

    size_t length;

    SmartQsoResult_t ret = safe_strlen("Hello, World!", 5, &length);

    assert_int_equal(ret, RESULT_ERROR_TRUNCATED);
    assert_int_equal(length, 5);
}

/**
 * @brief Test safe_strlen with empty string
 */
static void test_safe_strlen_empty(void **state)
{
    (void)state;

    size_t length;

    SmartQsoResult_t ret = safe_strlen("", 100, &length);

    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(length, 0);
}

/*===========================================================================*/
/* Safe String Comparison Tests                                               */
/*===========================================================================*/

/**
 * @brief Test safe_strcmp equal strings
 */
static void test_safe_strcmp_equal(void **state)
{
    (void)state;

    int result;

    SmartQsoResult_t ret = safe_strcmp("Hello", "Hello", 100, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, 0);
}

/**
 * @brief Test safe_strcmp less than
 */
static void test_safe_strcmp_less(void **state)
{
    (void)state;

    int result;

    SmartQsoResult_t ret = safe_strcmp("Apple", "Banana", 100, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, -1);
}

/**
 * @brief Test safe_strcmp greater than
 */
static void test_safe_strcmp_greater(void **state)
{
    (void)state;

    int result;

    SmartQsoResult_t ret = safe_strcmp("Zebra", "Apple", 100, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, 1);
}

/**
 * @brief Test safe_strncmp with limited comparison
 */
static void test_safe_strncmp_limited(void **state)
{
    (void)state;

    int result;

    /* Compare only first 5 chars */
    SmartQsoResult_t ret = safe_strncmp("Hello, World", "Hello, Galaxy", 5, &result);

    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, 0);
}

/*===========================================================================*/
/* Safe Memory Operations Tests                                               */
/*===========================================================================*/

/**
 * @brief Test safe_memcpy normal operation
 */
static void test_safe_memcpy_normal(void **state)
{
    (void)state;

    uint8_t src[] = {1, 2, 3, 4, 5};
    uint8_t dest[10];

    SmartQsoResult_t ret = safe_memcpy(dest, sizeof(dest), src, sizeof(src));

    assert_int_equal(ret, RESULT_OK);
    assert_memory_equal(dest, src, sizeof(src));
}

/**
 * @brief Test safe_memcpy insufficient destination
 */
static void test_safe_memcpy_insufficient(void **state)
{
    (void)state;

    uint8_t src[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    uint8_t dest[5];

    SmartQsoResult_t ret = safe_memcpy(dest, sizeof(dest), src, sizeof(src));

    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test safe_memcpy rejects overlapping buffers
 */
static void test_safe_memcpy_overlap(void **state)
{
    (void)state;

    uint8_t buffer[20] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};

    /* Overlapping regions should be rejected */
    SmartQsoResult_t ret = safe_memcpy(&buffer[5], 10, &buffer[0], 10);

    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test safe_memmove handles overlapping buffers
 */
static void test_safe_memmove_overlap(void **state)
{
    (void)state;

    uint8_t buffer[20] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    uint8_t expected[] = {1, 2, 3, 4, 5, 1, 2, 3, 4, 5};

    /* Forward overlap */
    SmartQsoResult_t ret = safe_memmove(&buffer[5], 10, &buffer[0], 5);

    assert_int_equal(ret, RESULT_OK);
    assert_memory_equal(buffer, expected, 10);
}

/**
 * @brief Test safe_memmove backward copy
 */
static void test_safe_memmove_backward(void **state)
{
    (void)state;

    uint8_t buffer[20] = {0, 0, 0, 1, 2, 3, 4, 5};
    uint8_t expected[] = {1, 2, 3, 4, 5, 3, 4, 5};

    /* Copy from later position to earlier */
    SmartQsoResult_t ret = safe_memmove(&buffer[0], 20, &buffer[3], 5);

    assert_int_equal(ret, RESULT_OK);
    assert_memory_equal(buffer, expected, 5);
}

/**
 * @brief Test safe_memset
 */
static void test_safe_memset(void **state)
{
    (void)state;

    uint8_t buffer[10];

    SmartQsoResult_t ret = safe_memset(buffer, sizeof(buffer), 0xAA, sizeof(buffer));

    assert_int_equal(ret, RESULT_OK);
    for (size_t i = 0; i < sizeof(buffer); i++) {
        assert_int_equal(buffer[i], 0xAA);
    }
}

/**
 * @brief Test safe_memset with count exceeding size
 */
static void test_safe_memset_overflow(void **state)
{
    (void)state;

    uint8_t buffer[10];

    SmartQsoResult_t ret = safe_memset(buffer, sizeof(buffer), 0xAA, 20);

    assert_int_equal(ret, RESULT_ERROR_PARAM);
}

/**
 * @brief Test safe_memcmp
 */
static void test_safe_memcmp(void **state)
{
    (void)state;

    uint8_t buf1[] = {1, 2, 3, 4, 5};
    uint8_t buf2[] = {1, 2, 3, 4, 5};
    uint8_t buf3[] = {1, 2, 3, 4, 6};
    int result;

    /* Equal */
    SmartQsoResult_t ret = safe_memcmp(buf1, buf2, sizeof(buf1), &result);
    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, 0);

    /* buf1 < buf3 */
    ret = safe_memcmp(buf1, buf3, sizeof(buf1), &result);
    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, -1);

    /* buf3 > buf1 */
    ret = safe_memcmp(buf3, buf1, sizeof(buf1), &result);
    assert_int_equal(ret, RESULT_OK);
    assert_int_equal(result, 1);
}

/*===========================================================================*/
/* Integer to String Tests                                                    */
/*===========================================================================*/

/**
 * @brief Test safe_itoa with positive number
 */
static void test_safe_itoa_positive(void **state)
{
    (void)state;

    char buffer[16];
    size_t length;

    SmartQsoResult_t ret = safe_itoa(12345, buffer, sizeof(buffer), &length);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(buffer, "12345");
    assert_int_equal(length, 5);
}

/**
 * @brief Test safe_itoa with negative number
 */
static void test_safe_itoa_negative(void **state)
{
    (void)state;

    char buffer[16];
    size_t length;

    SmartQsoResult_t ret = safe_itoa(-12345, buffer, sizeof(buffer), &length);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(buffer, "-12345");
    assert_int_equal(length, 6);
}

/**
 * @brief Test safe_itoa with zero
 */
static void test_safe_itoa_zero(void **state)
{
    (void)state;

    char buffer[16];
    size_t length;

    SmartQsoResult_t ret = safe_itoa(0, buffer, sizeof(buffer), &length);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(buffer, "0");
    assert_int_equal(length, 1);
}

/**
 * @brief Test safe_itoa with minimum int32
 */
static void test_safe_itoa_min_int32(void **state)
{
    (void)state;

    char buffer[16];
    size_t length;

    SmartQsoResult_t ret = safe_itoa(INT32_MIN, buffer, sizeof(buffer), &length);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(buffer, "-2147483648");
}

/**
 * @brief Test safe_itoa buffer overflow
 */
static void test_safe_itoa_overflow(void **state)
{
    (void)state;

    char buffer[4];  /* Too small for "12345" */
    size_t length;

    SmartQsoResult_t ret = safe_itoa(12345, buffer, sizeof(buffer), &length);

    assert_int_equal(ret, RESULT_ERROR_TRUNCATED);
}

/**
 * @brief Test safe_utoa
 */
static void test_safe_utoa(void **state)
{
    (void)state;

    char buffer[16];
    size_t length;

    SmartQsoResult_t ret = safe_utoa(4294967295U, buffer, sizeof(buffer), &length);

    assert_int_equal(ret, RESULT_OK);
    assert_string_equal(buffer, "4294967295");
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* String copy tests */
        cmocka_unit_test(test_safe_strcpy_normal),
        cmocka_unit_test(test_safe_strcpy_truncation),
        cmocka_unit_test(test_safe_strcpy_empty),
        cmocka_unit_test(test_safe_strcpy_null_params),
        cmocka_unit_test(test_safe_strncpy_limit),

        /* String concatenation tests */
        cmocka_unit_test(test_safe_strcat_normal),
        cmocka_unit_test(test_safe_strcat_truncation),
        cmocka_unit_test(test_safe_strcat_to_empty),

        /* String length tests */
        cmocka_unit_test(test_safe_strlen_normal),
        cmocka_unit_test(test_safe_strlen_max_reached),
        cmocka_unit_test(test_safe_strlen_empty),

        /* String comparison tests */
        cmocka_unit_test(test_safe_strcmp_equal),
        cmocka_unit_test(test_safe_strcmp_less),
        cmocka_unit_test(test_safe_strcmp_greater),
        cmocka_unit_test(test_safe_strncmp_limited),

        /* Memory operation tests */
        cmocka_unit_test(test_safe_memcpy_normal),
        cmocka_unit_test(test_safe_memcpy_insufficient),
        cmocka_unit_test(test_safe_memcpy_overlap),
        cmocka_unit_test(test_safe_memmove_overlap),
        cmocka_unit_test(test_safe_memmove_backward),
        cmocka_unit_test(test_safe_memset),
        cmocka_unit_test(test_safe_memset_overflow),
        cmocka_unit_test(test_safe_memcmp),

        /* Integer to string tests */
        cmocka_unit_test(test_safe_itoa_positive),
        cmocka_unit_test(test_safe_itoa_negative),
        cmocka_unit_test(test_safe_itoa_zero),
        cmocka_unit_test(test_safe_itoa_min_int32),
        cmocka_unit_test(test_safe_itoa_overflow),
        cmocka_unit_test(test_safe_utoa),
    };

    return cmocka_run_group_tests_name("Safe String Tests", tests, NULL, NULL);
}
