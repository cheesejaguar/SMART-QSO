/**
 * @file test_jetson_link.c
 * @brief OBC side of the OBC-Jetson line protocol (ICD_OBC_JETSON section 4)
 *
 * The golden lines are shared with software/payload/tests/test_obc_link.py,
 * which checks the same bytes from the Jetson side (CRC computed with the
 * payload's crc16_ccitt), so a change on either side fails both suites.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "jetson_link.h"

#define TELEMETRY_LINE   "TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86\n"
#define BEACON_REQ_LINE  "BEACON_REQ,42,176,5593\n"
#define BEACON_RSP_LINE  "BEACON_RSP,42,Hello, world from orbit,0784\n"
#define STATUS_LINE      "STATUS,0,45,6800,37,1234,D775\n"
#define STATUS_REQ_LINE  "STATUS_REQ,5D03\n"
#define SHUTDOWN_LINE    "SHUTDOWN,POWER,05D4\n"
#define ACK_LINE         "ACK,BEACON_RSP,275C\n"
#define NACK_LINE        "NACK,CRC,49EE\n"

static void test_crc_check_value(void **state)
{
    (void)state;
    const uint8_t check[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
    assert_int_equal(jetson_crc16(check, sizeof(check)), 0x29B1U);  /* CRC-16/CCITT-FALSE */
}

/** The OBC's outgoing lines match the Jetson-side reference bytes */
static void test_obc_lines_match_reference(void **state)
{
    (void)state;
    char buf[JETSON_LINE_MAX];
    size_t len;

    (void)strcpy(buf, "TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON");
    len = jetson_link_seal(buf, strlen(buf), sizeof(buf));
    assert_int_equal(len, strlen(TELEMETRY_LINE));
    assert_memory_equal(buf, TELEMETRY_LINE, len);

    len = jetson_link_beacon_req(42U, 176U, buf, sizeof(buf));
    assert_int_equal(len, strlen(BEACON_REQ_LINE));
    assert_memory_equal(buf, BEACON_REQ_LINE, len);

    len = jetson_link_status_req(buf, sizeof(buf));
    assert_int_equal(len, strlen(STATUS_REQ_LINE));
    assert_memory_equal(buf, STATUS_REQ_LINE, len);

    len = jetson_link_shutdown("POWER", buf, sizeof(buf));
    assert_int_equal(len, strlen(SHUTDOWN_LINE));
    assert_memory_equal(buf, SHUTDOWN_LINE, len);
}

/** A beacon response's text may contain commas: it runs to the CRC field */
static void test_parse_beacon_response(void **state)
{
    (void)state;
    JetsonMsg_t msg;

    assert_int_equal(jetson_link_parse(BEACON_RSP_LINE, strlen(BEACON_RSP_LINE), &msg), JETSON_OK);
    assert_int_equal(msg.type, JETSON_MSG_BEACON_RSP);
    assert_int_equal(msg.seq, 42U);
    assert_int_equal(msg.text_len, strlen("Hello, world from orbit"));
    assert_string_equal(msg.text, "Hello, world from orbit");
}

static void test_parse_status_ack_nack(void **state)
{
    (void)state;
    JetsonMsg_t msg;

    assert_int_equal(jetson_link_parse(STATUS_LINE, strlen(STATUS_LINE), &msg), JETSON_OK);
    assert_int_equal(msg.type, JETSON_MSG_STATUS);
    assert_int_equal(msg.health, 0U);
    assert_int_equal(msg.temp_c, 45);
    assert_int_equal(msg.power_mw, 6800U);
    assert_int_equal(msg.mem_pct, 37U);
    assert_int_equal(msg.uptime_s, 1234U);

    assert_int_equal(jetson_link_parse(ACK_LINE, strlen(ACK_LINE), &msg), JETSON_OK);
    assert_int_equal(msg.type, JETSON_MSG_ACK);
    assert_string_equal(msg.text, "BEACON_RSP");

    assert_int_equal(jetson_link_parse(NACK_LINE, strlen(NACK_LINE), &msg), JETSON_OK);
    assert_int_equal(msg.type, JETSON_MSG_NACK);
    assert_string_equal(msg.text, "CRC");
}

/** Lines the OBC itself sends are not accepted as Jetson messages */
static void test_obc_bound_types_rejected(void **state)
{
    (void)state;
    JetsonMsg_t msg;

    assert_int_equal(jetson_link_parse(TELEMETRY_LINE, strlen(TELEMETRY_LINE), &msg), JETSON_ERR_TYPE);
    assert_int_equal(jetson_link_parse(BEACON_REQ_LINE, strlen(BEACON_REQ_LINE), &msg), JETSON_ERR_TYPE);
}

static void test_malformed_lines_rejected(void **state)
{
    (void)state;
    JetsonMsg_t msg;
    char line[JETSON_LINE_MAX + 8U];

    /* Bit error in the text */
    (void)strcpy(line, BEACON_RSP_LINE);
    line[20] = 'X';
    assert_int_equal(jetson_link_parse(line, strlen(line), &msg), JETSON_ERR_CRC);

    /* Lower-case or short CRC field */
    assert_int_equal(jetson_link_parse("STATUS,0,45,6800,37,1234,d775\n", 30U, &msg), JETSON_ERR_FORMAT);
    assert_int_equal(jetson_link_parse("STATUS_REQ,5D0\n", 15U, &msg), JETSON_ERR_FORMAT);

    /* Control character inside the line */
    (void)strcpy(line, "BEACON_RSP,42,Hello\tworld");
    size_t len = jetson_link_seal(line, strlen(line), sizeof(line));
    assert_int_equal(len, 0U);  /* sealing refuses non-printable content */

    /* Missing fields, out-of-range values */
    (void)strcpy(line, "BEACON_RSP,42");
    len = jetson_link_seal(line, strlen(line), sizeof(line));
    assert_int_equal(jetson_link_parse(line, len, &msg), JETSON_ERR_FORMAT);
    (void)strcpy(line, "BEACON_RSP,70000,Hi");
    len = jetson_link_seal(line, strlen(line), sizeof(line));
    assert_int_equal(jetson_link_parse(line, len, &msg), JETSON_ERR_FORMAT);
    (void)strcpy(line, "STATUS,9,45,6800,37,1234");
    len = jetson_link_seal(line, strlen(line), sizeof(line));
    assert_int_equal(jetson_link_parse(line, len, &msg), JETSON_ERR_FORMAT);

    /* Empty and NULL */
    assert_int_equal(jetson_link_parse("", 0U, &msg), JETSON_ERR_FORMAT);
    assert_int_equal(jetson_link_parse(NULL, 10U, &msg), JETSON_ERR_FORMAT);
}

/** Beacon text longer than the beacon can carry is rejected */
static void test_beacon_text_limit(void **state)
{
    (void)state;
    char line[JETSON_LINE_MAX];
    JetsonMsg_t msg;

    (void)strcpy(line, "BEACON_RSP,1,");
    size_t base = strlen(line);
    memset(&line[base], 'a', JETSON_TEXT_MAX + 1U);
    line[base + JETSON_TEXT_MAX + 1U] = '\0';
    size_t len = jetson_link_seal(line, strlen(line), sizeof(line));
    assert_true(len > 0U);
    assert_int_equal(jetson_link_parse(line, len, &msg), JETSON_ERR_FORMAT);

    line[base + JETSON_TEXT_MAX] = '\0';
    len = jetson_link_seal(line, strlen(line), sizeof(line));
    assert_int_equal(jetson_link_parse(line, len, &msg), JETSON_OK);
    assert_int_equal(msg.text_len, JETSON_TEXT_MAX);
}

/** Sealing refuses a line that would not fit */
static void test_seal_overflow(void **state)
{
    (void)state;
    char buf[12];
    (void)strcpy(buf, "STATUS_REQ");
    assert_int_equal(jetson_link_seal(buf, strlen(buf), sizeof(buf)), 0U);
}

/** Bytes from the UART accumulate into lines; an overlong line is dropped
 *  up to its newline and reported once, then reception resynchronizes */
static void test_receive_accumulator(void **state)
{
    (void)state;
    JetsonRx_t rx;
    JetsonMsg_t msg;
    JetsonStatus_t st = JETSON_OK;
    const char *line = BEACON_RSP_LINE;
    int completed = 0;

    jetson_rx_init(&rx);
    for (size_t i = 0U; i < strlen(line); i++) {
        if (jetson_rx_feed(&rx, line[i], &msg, &st)) {
            completed++;
        }
    }
    assert_int_equal(completed, 1);
    assert_int_equal(st, JETSON_OK);
    assert_string_equal(msg.text, "Hello, world from orbit");

    completed = 0;
    for (size_t i = 0U; i < (JETSON_LINE_MAX + 50U); i++) {
        if (jetson_rx_feed(&rx, 'A', &msg, &st)) {
            completed++;
        }
    }
    assert_int_equal(completed, 0);
    assert_true(jetson_rx_feed(&rx, '\n', &msg, &st));
    assert_int_equal(st, JETSON_ERR_FORMAT);

    completed = 0;
    for (size_t i = 0U; i < strlen(STATUS_LINE); i++) {
        if (jetson_rx_feed(&rx, STATUS_LINE[i], &msg, &st)) {
            completed++;
        }
    }
    assert_int_equal(completed, 1);
    assert_int_equal(st, JETSON_OK);
    assert_int_equal(msg.type, JETSON_MSG_STATUS);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_crc_check_value),
        cmocka_unit_test(test_obc_lines_match_reference),
        cmocka_unit_test(test_parse_beacon_response),
        cmocka_unit_test(test_parse_status_ack_nack),
        cmocka_unit_test(test_obc_bound_types_rejected),
        cmocka_unit_test(test_malformed_lines_rejected),
        cmocka_unit_test(test_beacon_text_limit),
        cmocka_unit_test(test_seal_overflow),
        cmocka_unit_test(test_receive_accumulator),
    };

    return cmocka_run_group_tests_name("OBC-Jetson Link Tests", tests, NULL, NULL);
}
