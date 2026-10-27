/**
 * @file test_hk_store.c
 * @brief Stored housekeeping in FRAM and DUMP_TLM (BASELINE §8, DB-01)
 *
 * Expectations from hk_store.h and docs/software/SOFTWARE_REQUIREMENTS.md
 * §3.12:
 *
 * - one five-minute record every 300 s, one sixty-second record every
 *   60 s; sequence numbers continue across a reset;
 * - the five-minute ring holds at least 7 days (2,016 records);
 * - an event keeps the 3 h before and after it at 60 s;
 * - DUMP_TLM: an unknown store or a count above 600 is refused; frames
 *   carry at most four records, at most one every 12 s; 0 stops a dump;
 * - the executive's DUMP_TLM handler parses store, first and count.
 *
 * @requirement SRS-F120 Five-minute housekeeping records for at least 7 days in FRAM
 * @requirement SRS-F121 Sixty-second records for the 6 h around each fault and mode change
 * @requirement SRS-F122 DUMP_TLM downlinks a range of stored records within the downlink duty limit
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "hk_store.h"
#include "cmd_handler.h"
#include "cmd_dictionary.h"
#include "fault_mgmt.h"
#include "system_state.h"
#include "hal/hal.h"
#include "hal/hal_flash.h"

static uint64_t s_now;

static int setup(void **state)
{
    (void)state;
    assert_int_equal(hal_flash_init(), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_HK_SUMMARY), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_HK_DETAIL), SMART_QSO_OK);
    assert_int_equal(hal_flash_erase(FLASH_REGION_HK_INDEX), SMART_QSO_OK);
    assert_int_equal(hal_flash_region_size(FLASH_REGION_HK_SUMMARY), 2048U * HK_STORE_RECORD_LEN);
    assert_true(hal_flash_region_size(FLASH_REGION_HK_DETAIL) >= 1489U * HK_STORE_RECORD_LEN);
    assert_int_equal(hk_store_init(), SMART_QSO_OK);
    s_now = smart_qso_now_ms(); /* not before the store's own start */
    return 0;
}

/** Run the store for `minutes`, a pass every second */
static void run(uint32_t minutes)
{
    for (uint32_t i = 0U; i < minutes * 60U; i++) {
        s_now += 1000U;
        hk_store_step(s_now);
    }
}

static void counts(uint32_t *summary, uint32_t *detail)
{
    assert_int_equal(hk_store_counts(summary, detail), SMART_QSO_OK);
}

static void test_cadence_and_reset(void **state)
{
    (void)state;
    uint32_t a, b;
    uint8_t rec[HK_STORE_RECORD_LEN];
    run(60U);
    counts(&a, &b);
    assert_int_equal(a, 12U);
    assert_int_equal(b, 60U);
    assert_int_equal(hk_store_read(HK_STORE_SUMMARY, 11U, rec, sizeof(rec)), SMART_QSO_OK);
    assert_int_equal(rec[0], 'S');
    assert_int_equal(hk_store_read(HK_STORE_DETAIL, 59U, rec, sizeof(rec)), SMART_QSO_OK);
    assert_int_equal(rec[0], 'D');
    assert_int_equal(hk_store_read(HK_STORE_SUMMARY, 12U, rec, sizeof(rec)), SMART_QSO_ERROR);
    assert_int_equal(hk_store_read(2U, 0U, rec, sizeof(rec)), SMART_QSO_ERROR_INVALID);
    /* A reset: the numbers go on */
    assert_int_equal(hk_store_init(), SMART_QSO_OK);
    counts(&a, &b);
    assert_int_equal(a, 12U);
    assert_int_equal(b, 60U);
}

static void test_seven_days(void **state)
{
    (void)state;
    uint32_t a, b;
    uint8_t rec[HK_STORE_RECORD_LEN];
    run(7U * 24U * 60U + 5U);
    counts(&a, &b);
    assert_int_equal(a, 7U * 288U + 1U);
    assert_int_equal(hk_store_read(HK_STORE_SUMMARY, 0U, rec, sizeof(rec)), SMART_QSO_OK); /* 7 days back */
    /* The sixty-second ring keeps 6.8 h */
    assert_int_equal(hk_store_read(HK_STORE_DETAIL, b - 409U, rec, sizeof(rec)), SMART_QSO_OK);
    assert_int_equal(hk_store_read(HK_STORE_DETAIL, b - 410U, rec, sizeof(rec)), SMART_QSO_ERROR);
}

static void test_event_window(void **state)
{
    (void)state;
    uint32_t a, b;
    uint8_t rec[HK_STORE_RECORD_LEN];
    run(5U * 60U);
    counts(&a, &b);
    const uint32_t trigger = b;
    hk_store_note_event(HK_STORE_EVENT_FAULT);
    run(12U * 60U);
    for (uint32_t seq = trigger - 180U; seq < trigger + 180U; seq++) {
        assert_int_equal(hk_store_read(HK_STORE_DETAIL, seq, rec, sizeof(rec)), SMART_QSO_OK);
    }
    assert_int_equal(hk_store_read(HK_STORE_DETAIL, trigger, rec, sizeof(rec)), SMART_QSO_OK);
    assert_true((rec[13] & 0x20U) != 0U); /* the event flag */
    assert_int_equal(hk_store_read(HK_STORE_DETAIL, trigger + 180U, rec, sizeof(rec)), SMART_QSO_ERROR);
}

static void test_dump(void **state)
{
    (void)state;
    uint8_t info[256];
    run(30U); /* six five-minute records */
    assert_int_equal(hk_store_dump(2U, 0U, 1U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(hk_store_dump(HK_STORE_SUMMARY, 0U, HK_STORE_MAX_DUMP + 1U), SMART_QSO_ERROR_INVALID);
    assert_int_equal(hk_store_dump_peek(s_now, info, sizeof(info)), 0U);
    assert_int_equal(hk_store_dump(HK_STORE_SUMMARY, 0U, 6U), SMART_QSO_OK);
    assert_true(hk_store_dump_active());
    size_t n = hk_store_dump_peek(s_now, info, sizeof(info));
    assert_int_equal(n, 5U + (4U * HK_STORE_RECORD_LEN));
    assert_memory_equal(info, "HKD", 3U);
    assert_int_equal(info[3], HK_STORE_SUMMARY);
    assert_int_equal(info[4], 4U);
    assert_int_equal(hk_store_dump_peek(s_now, info, 10U), 0U); /* too small */
    hk_store_dump_commit(s_now);
    assert_int_equal(hk_store_dump_peek(s_now + 11999U, info, sizeof(info)), 0U); /* paced */
    n = hk_store_dump_peek(s_now + 12000U, info, sizeof(info));
    assert_int_equal(n, 5U + (2U * HK_STORE_RECORD_LEN));
    hk_store_dump_commit(s_now + 12000U);
    assert_false(hk_store_dump_active());
    assert_int_equal(hk_store_dump(HK_STORE_DETAIL, 0U, 100U), SMART_QSO_OK);
    hk_store_dump_stop();
    assert_false(hk_store_dump_active());
}

/** DUMP_TLM through the command handler: store, first (BE), count (BE) */
static void test_dump_command(void **state)
{
    (void)state;
    Command_t cmd;
    CmdResponse_t rsp;
    run(10U);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(cmd_handler_init(), SMART_QSO_OK);
    memset(&cmd, 0, sizeof(cmd));
    cmd.cmd_id = (uint8_t)CMD_SYS_DUMP_TLM;
    cmd.payload_len = 7U;
    const uint8_t args[7] = {HK_STORE_DETAIL, 0U, 0U, 0U, 2U, 0x00U, 0x05U};
    memcpy(cmd.payload, args, sizeof(args));
    assert_int_equal(cmd_execute(&cmd, &rsp), SMART_QSO_OK);
    assert_int_equal(rsp.result, CMD_RESULT_SUCCESS);
    assert_true(hk_store_dump_active());
    cmd.payload[5] = 0x03U; /* 0x0305 = 773 > 600 */
    assert_int_equal(cmd_execute(&cmd, &rsp), SMART_QSO_OK);
    assert_int_equal(rsp.result, CMD_RESULT_INVALID_PARAM);
    cmd.payload[5] = 0U;
    cmd.payload[6] = 0U; /* count 0: stop */
    assert_int_equal(cmd_execute(&cmd, &rsp), SMART_QSO_OK);
    assert_int_equal(rsp.result, CMD_RESULT_SUCCESS);
    assert_false(hk_store_dump_active());
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test_setup(test_cadence_and_reset, setup),
        cmocka_unit_test_setup(test_seven_days, setup),
        cmocka_unit_test_setup(test_event_window, setup),
        cmocka_unit_test_setup(test_dump, setup),
        cmocka_unit_test_setup(test_dump_command, setup),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
