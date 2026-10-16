/**
 * @file test_ax25.c
 * @brief AX.25 address encoding, FCS and UI-frame parsing
 *
 * The reference frame was generated in Python (shared with
 * software/ground/tests/test_telecommand_builder.py).
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "ax25.h"

/** UI frame KK6GDB-11 <- N0CALL-0 carrying TX_DISABLE counter 1 (TEST-ONLY key) */
static const uint8_t UPLINK_TX_DISABLE[] = {
    0x96, 0x96, 0x6C, 0x8E, 0x88, 0x84, 0x76, 0x9C, 0x60, 0x86, 0x82, 0x98, 0x98, 0x61,
    0x03, 0xF0, 0xAA, 0x55, 0x32, 0x14, 0x00, 0x00, 0x00, 0x01, 0x9A, 0x83, 0xF5, 0xC7,
    0xB2, 0x1F, 0x75, 0x11, 0xE8, 0x2F, 0x5E, 0x94, 0xB9, 0xDE, 0x58, 0x51, 0x09, 0xBF,
    0xC8, 0xB6
};

static void test_fcs_check_value(void **state)
{
    (void)state;
    const uint8_t check[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
    assert_int_equal(ax25_fcs(check, sizeof(check)), 0x906EU);  /* CRC-16/X.25 */
}

static void test_encode_address(void **state)
{
    (void)state;
    const uint8_t expected[AX25_ADDR_LEN] = {0xA6, 0xA2, 0xA6, 0x9E, 0x40, 0x40, 0x63};
    uint8_t out[AX25_ADDR_LEN];

    ax25_encode_address("sqso", 1U, out, true);
    assert_memory_equal(out, expected, sizeof(out));
}

static void test_parse_reference_frame(void **state)
{
    (void)state;
    Ax25UiView_t view;

    assert_int_equal(ax25_parse_ui(UPLINK_TX_DISABLE, sizeof(UPLINK_TX_DISABLE), &view), AX25_OK);
    assert_true(ax25_address_matches(view.dest, "KK6GDB", 11U));
    assert_false(ax25_address_matches(view.dest, "KK6GDB", 1U));
    assert_false(ax25_address_matches(view.dest, "KK6GD", 11U));
    assert_true(ax25_address_matches(view.src, "N0CALL", 0U));
    assert_int_equal(view.info_len, 26U);
    assert_int_equal(view.info[0], 0xAAU);
    assert_int_equal(view.info[25], 0xBFU);
}

static void test_parse_rejects_bad_frames(void **state)
{
    (void)state;
    uint8_t frame[sizeof(UPLINK_TX_DISABLE)];
    Ax25UiView_t view;

    memcpy(frame, UPLINK_TX_DISABLE, sizeof(frame));
    frame[20] ^= 0x01U;
    assert_int_equal(ax25_parse_ui(frame, sizeof(frame), &view), AX25_ERR_FCS);

    assert_int_equal(ax25_parse_ui(UPLINK_TX_DISABLE, 17U, &view), AX25_ERR_FORMAT);
    assert_int_equal(ax25_parse_ui(NULL, sizeof(frame), &view), AX25_ERR_FORMAT);

    /* Not a UI frame / wrong PID / no end-of-address bit: re-sign the FCS
     * so only the field under test is wrong */
    memcpy(frame, UPLINK_TX_DISABLE, sizeof(frame));
    frame[14] = 0x13U;
    uint16_t fcs = ax25_fcs(frame, sizeof(frame) - 2U);
    frame[sizeof(frame) - 2U] = (uint8_t)fcs;
    frame[sizeof(frame) - 1U] = (uint8_t)(fcs >> 8);
    assert_int_equal(ax25_parse_ui(frame, sizeof(frame), &view), AX25_ERR_NOT_UI);

    memcpy(frame, UPLINK_TX_DISABLE, sizeof(frame));
    frame[15] = 0xCFU;
    fcs = ax25_fcs(frame, sizeof(frame) - 2U);
    frame[sizeof(frame) - 2U] = (uint8_t)fcs;
    frame[sizeof(frame) - 1U] = (uint8_t)(fcs >> 8);
    assert_int_equal(ax25_parse_ui(frame, sizeof(frame), &view), AX25_ERR_NOT_UI);

    memcpy(frame, UPLINK_TX_DISABLE, sizeof(frame));
    frame[13] &= 0xFEU;  /* source no longer marked last */
    fcs = ax25_fcs(frame, sizeof(frame) - 2U);
    frame[sizeof(frame) - 2U] = (uint8_t)fcs;
    frame[sizeof(frame) - 1U] = (uint8_t)(fcs >> 8);
    assert_int_equal(ax25_parse_ui(frame, sizeof(frame), &view), AX25_ERR_FORMAT);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_fcs_check_value),
        cmocka_unit_test(test_encode_address),
        cmocka_unit_test(test_parse_reference_frame),
        cmocka_unit_test(test_parse_rejects_bad_frames),
    };

    return cmocka_run_group_tests_name("AX.25 Tests", tests, NULL, NULL);
}
