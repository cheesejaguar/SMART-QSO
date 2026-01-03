/**
 * @file test_crc32.c
 * @brief Unit tests for CRC32 module
 *
 * Tests CRC32 calculation and verification per SRS-F061.
 *
 * @requirement SRS-F061 CRC32 protection for persisted data
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

/* Include the module under test */
#include "smart_qso.h"

/*===========================================================================*/
/* Test Cases                                                                 */
/*===========================================================================*/

/**
 * @brief Test CRC32 with known test vector
 *
 * IEEE 802.3 CRC32 of "123456789" should be 0xCBF43926
 */
static void test_crc32_known_vector(void **state) {
    (void)state;

    const char *test_data = "123456789";
    uint32_t expected_crc = 0xCBF43926;

    uint32_t calculated_crc = smart_qso_crc32(test_data, 9);

    assert_int_equal(calculated_crc, expected_crc);
}

/**
 * @brief Test CRC32 with empty data
 */
static void test_crc32_empty_data(void **state) {
    (void)state;

    /* CRC32 of empty data should be 0x00000000 (after final XOR) */
    uint32_t crc = smart_qso_crc32("", 0);

    /* Empty data CRC is 0xFFFFFFFF ^ 0xFFFFFFFF = 0x00000000 */
    assert_int_equal(crc, 0x00000000);
}

/**
 * @brief Test CRC32 with single byte
 */
static void test_crc32_single_byte(void **state) {
    (void)state;

    uint8_t byte = 0x00;
    uint32_t crc = smart_qso_crc32(&byte, 1);

    /* Verify non-zero result for single byte */
    assert_true(crc != 0);
}

/**
 * @brief Test CRC32 detects single bit change
 */
static void test_crc32_single_bit_change(void **state) {
    (void)state;

    uint8_t data1[] = {0x00, 0x00, 0x00, 0x00};
    uint8_t data2[] = {0x01, 0x00, 0x00, 0x00};  /* Single bit difference */

    uint32_t crc1 = smart_qso_crc32(data1, sizeof(data1));
    uint32_t crc2 = smart_qso_crc32(data2, sizeof(data2));

    assert_int_not_equal(crc1, crc2);
}

/**
 * @brief Test CRC32 verification success
 */
static void test_crc32_verify_success(void **state) {
    (void)state;

    const char *test_data = "Test data for CRC verification";
    size_t len = strlen(test_data);

    uint32_t crc = smart_qso_crc32(test_data, len);
    bool result = smart_qso_verify_crc32(test_data, len, crc);

    assert_true(result);
}

/**
 * @brief Test CRC32 verification failure
 */
static void test_crc32_verify_failure(void **state) {
    (void)state;

    const char *test_data = "Test data for CRC verification";
    size_t len = strlen(test_data);

    uint32_t crc = smart_qso_crc32(test_data, len);
    uint32_t wrong_crc = crc ^ 0x00000001;  /* Flip one bit */

    bool result = smart_qso_verify_crc32(test_data, len, wrong_crc);

    assert_false(result);
}

/**
 * @brief Test CRC32 with large data
 */
static void test_crc32_large_data(void **state) {
    (void)state;

    /* Create 4KB of test data */
    uint8_t data[4096];
    for (size_t i = 0; i < sizeof(data); i++) {
        data[i] = (uint8_t)(i & 0xFF);
    }

    uint32_t crc = smart_qso_crc32(data, sizeof(data));

    /* Verify we get a valid CRC */
    assert_true(crc != 0);

    /* Verify it's consistent */
    uint32_t crc2 = smart_qso_crc32(data, sizeof(data));
    assert_int_equal(crc, crc2);
}

/**
 * @brief Test CRC32 with structure data
 */
static void test_crc32_structure(void **state) {
    (void)state;

    /* Simulate a typical config structure with CRC at the end */
    typedef struct {
        uint32_t version;
        uint32_t reset_count;
        double energy;
        uint32_t crc32;
    } TestConfig_t;

    TestConfig_t config = {
        .version = 1,
        .reset_count = 42,
        .energy = 123.456,
        .crc32 = 0
    };

    /* Calculate CRC over data fields only (excluding CRC field at the end).
     * offsetof gives us the offset of crc32, which is the length to checksum.
     */
    size_t data_len = offsetof(TestConfig_t, crc32);
    config.crc32 = smart_qso_crc32(&config, data_len);

    /* Verify - should recalculate the same CRC since we're not including crc32 field */
    bool result = smart_qso_verify_crc32(&config, data_len, config.crc32);
    assert_true(result);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_crc32_known_vector),
        cmocka_unit_test(test_crc32_empty_data),
        cmocka_unit_test(test_crc32_single_byte),
        cmocka_unit_test(test_crc32_single_bit_change),
        cmocka_unit_test(test_crc32_verify_success),
        cmocka_unit_test(test_crc32_verify_failure),
        cmocka_unit_test(test_crc32_large_data),
        cmocka_unit_test(test_crc32_structure),
    };

    return cmocka_run_group_tests_name("CRC32 Tests", tests, NULL, NULL);
}
