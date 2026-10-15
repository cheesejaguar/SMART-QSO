/**
 * @file test_nvm_record.c
 * @brief Unit tests for the two-slot power-fail-safe record store
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "nvm_record.h"
#include "hal/hal_flash.h"

#define TEST_REGION     FLASH_REGION_BACKUP
#define TEST_MAGIC      0x54455354U  /* "TEST" */

typedef struct {
    uint32_t value;
    uint32_t other;
} TestPayload_t;

static int setup(void **state)
{
    (void)state;
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(TEST_REGION), SMART_QSO_OK);
    return 0;
}

/** Overwrite one slot byte to simulate corruption or an interrupted write */
static void corrupt_slot_byte(uint32_t slot, uint32_t byte_offset)
{
    uint8_t b;
    uint32_t offset = (slot * NVM_RECORD_SLOT_SIZE) + byte_offset;
    assert_int_equal(hal_flash_read(TEST_REGION, offset, &b, 1U), SMART_QSO_OK);
    b ^= 0xFFU;
    assert_int_equal(hal_flash_write(TEST_REGION, offset, &b, 1U), SMART_QSO_OK);
}

static void test_erased_region_is_blank(void **state)
{
    (void)state;
    TestPayload_t p = {0};
    NvmRecordStatus_t status = NVM_RECORD_VALID;

    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &p, sizeof(p), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_BLANK);
}

/** Simulation flash starts zero-filled; real erased flash is 0xFF */
static void test_zeroed_region_is_blank(void **state)
{
    (void)state;
    uint8_t zeros[2U * NVM_RECORD_SLOT_SIZE];
    TestPayload_t p = {0};
    NvmRecordStatus_t status = NVM_RECORD_VALID;

    memset(zeros, 0, sizeof(zeros));
    assert_int_equal(hal_flash_write(TEST_REGION, 0U, zeros, sizeof(zeros)), SMART_QSO_OK);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &p, sizeof(p), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_BLANK);
}

static void test_save_then_load_round_trip(void **state)
{
    (void)state;
    TestPayload_t in = {0x12345678U, 0xCAFEF00DU};
    TestPayload_t out = {0};
    NvmRecordStatus_t status = NVM_RECORD_BLANK;

    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &in, sizeof(in)), SMART_QSO_OK);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &out, sizeof(out), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_VALID);
    assert_memory_equal(&in, &out, sizeof(in));
}

static void test_latest_save_wins(void **state)
{
    (void)state;
    TestPayload_t out = {0};
    NvmRecordStatus_t status = NVM_RECORD_BLANK;

    for (uint32_t v = 1U; v <= 5U; v++) {
        TestPayload_t in = {v, 0U};
        assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &in, sizeof(in)), SMART_QSO_OK);
    }
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &out, sizeof(out), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_VALID);
    assert_int_equal(out.value, 5U);
}

/** A torn write of the newest slot falls back to the previous copy */
static void test_corrupt_newest_slot_falls_back(void **state)
{
    (void)state;
    TestPayload_t first = {1U, 0U};
    TestPayload_t second = {2U, 0U};
    TestPayload_t out = {0};
    NvmRecordStatus_t status = NVM_RECORD_BLANK;

    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &first, sizeof(first)), SMART_QSO_OK);
    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &second, sizeof(second)), SMART_QSO_OK);

    /* first went to slot A, second to slot B; corrupt B's payload */
    corrupt_slot_byte(1U, 8U);

    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &out, sizeof(out), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_VALID);
    assert_int_equal(out.value, 1U);
}

/** After a fallback, the next save must not overwrite the surviving copy */
static void test_save_after_fallback_preserves_survivor(void **state)
{
    (void)state;
    TestPayload_t first = {1U, 0U};
    TestPayload_t second = {2U, 0U};
    TestPayload_t third = {3U, 0U};
    TestPayload_t out = {0};
    NvmRecordStatus_t status = NVM_RECORD_BLANK;

    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &first, sizeof(first)), SMART_QSO_OK);
    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &second, sizeof(second)), SMART_QSO_OK);
    corrupt_slot_byte(1U, 8U);

    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &third, sizeof(third)), SMART_QSO_OK);
    /* Now corrupt the slot just written (B again): slot A (value 1) must survive */
    corrupt_slot_byte(1U, 8U);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &out, sizeof(out), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_VALID);
    assert_int_equal(out.value, 1U);
}

static void test_both_slots_corrupt_reports_corrupt(void **state)
{
    (void)state;
    TestPayload_t in = {7U, 0U};
    TestPayload_t out = {0};
    NvmRecordStatus_t status = NVM_RECORD_VALID;

    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &in, sizeof(in)), SMART_QSO_OK);
    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, &in, sizeof(in)), SMART_QSO_OK);
    corrupt_slot_byte(0U, 8U);
    corrupt_slot_byte(1U, 8U);

    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &out, sizeof(out), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_CORRUPT);
}

static void test_other_magic_is_not_this_record(void **state)
{
    (void)state;
    TestPayload_t in = {7U, 0U};
    TestPayload_t out = {0};
    NvmRecordStatus_t status = NVM_RECORD_VALID;

    assert_int_equal(nvm_record_save(TEST_REGION, 0x4F544852U, &in, sizeof(in)), SMART_QSO_OK);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &out, sizeof(out), &status),
                     SMART_QSO_OK);
    assert_int_equal(status, NVM_RECORD_BLANK);
}

static void test_rejects_oversized_payload(void **state)
{
    (void)state;
    uint8_t big[NVM_RECORD_MAX_PAYLOAD + 1U];
    NvmRecordStatus_t status = NVM_RECORD_VALID;

    memset(big, 0, sizeof(big));
    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, big, sizeof(big)),
                     SMART_QSO_ERROR_INVALID);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, big, sizeof(big), &status),
                     SMART_QSO_ERROR_INVALID);
}

static void test_null_arguments(void **state)
{
    (void)state;
    TestPayload_t p = {0};
    NvmRecordStatus_t status;

    assert_int_equal(nvm_record_save(TEST_REGION, TEST_MAGIC, NULL, sizeof(p)),
                     SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, NULL, sizeof(p), &status),
                     SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(nvm_record_load(TEST_REGION, TEST_MAGIC, &p, sizeof(p), NULL),
                     SMART_QSO_ERROR_NULL_PTR);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_erased_region_is_blank, setup),
        cmocka_unit_test_setup(test_zeroed_region_is_blank, setup),
        cmocka_unit_test_setup(test_save_then_load_round_trip, setup),
        cmocka_unit_test_setup(test_latest_save_wins, setup),
        cmocka_unit_test_setup(test_corrupt_newest_slot_falls_back, setup),
        cmocka_unit_test_setup(test_save_after_fallback_preserves_survivor, setup),
        cmocka_unit_test_setup(test_both_slots_corrupt_reports_corrupt, setup),
        cmocka_unit_test_setup(test_other_magic_is_not_this_record, setup),
        cmocka_unit_test_setup(test_rejects_oversized_payload, setup),
        cmocka_unit_test_setup(test_null_arguments, setup),
    };

    return cmocka_run_group_tests_name("NVM Record Tests", tests, NULL, NULL);
}
