"""
Jetson side of the OBC-Jetson line protocol (docs/interfaces/ICD_OBC_JETSON.md §4).

The golden lines are shared with software/flight/tests/test_jetson_link.c, so a
change on either side fails both suites.
"""

import unittest

from obc_link import (
    LINE_MAX,
    TEXT_MAX,
    CrcError,
    LinkError,
    beacon_rsp,
    nack,
    ack,
    parse,
    parse_beacon_req,
    parse_telemetry,
    seal,
    status,
)

TELEMETRY_LINE = b"TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86\n"
BEACON_REQ_LINE = b"BEACON_REQ,42,176,5593\n"
BEACON_RSP_LINE = b"BEACON_RSP,42,Hello, world from orbit,0784\n"
STATUS_LINE = b"STATUS,0,45,6800,37,1234,D775\n"
STATUS_REQ_LINE = b"STATUS_REQ,5D03\n"
SHUTDOWN_LINE = b"SHUTDOWN,POWER,05D4\n"
ACK_LINE = b"ACK,BEACON_RSP,275C\n"
NACK_LINE = b"NACK,CRC,49EE\n"


class TestJetsonLinesMatchReference(unittest.TestCase):
    """Lines the Jetson sends match the OBC-side reference bytes."""

    def test_beacon_response(self):
        self.assertEqual(beacon_rsp(42, "Hello, world from orbit"), BEACON_RSP_LINE)

    def test_status(self):
        self.assertEqual(status(0, 45, 6800, 37, 1234), STATUS_LINE)

    def test_ack_nack(self):
        self.assertEqual(ack("BEACON_RSP"), ACK_LINE)
        self.assertEqual(nack("CRC"), NACK_LINE)

    def test_seal(self):
        self.assertEqual(seal("STATUS_REQ"), STATUS_REQ_LINE)


class TestParseObcLines(unittest.TestCase):
    """Lines the OBC sends parse on the Jetson."""

    def test_telemetry(self):
        msg = parse(TELEMETRY_LINE)
        self.assertEqual(msg.type, "TELEMETRY")
        t = parse_telemetry(msg)
        self.assertEqual(t.time_ms, 1704153600000)
        self.assertTrue(t.sunlit)
        self.assertAlmostEqual(t.soc, 0.75)
        self.assertEqual(t.power_mode, 2)
        self.assertAlmostEqual(t.power_limit_w, 3.0)
        self.assertEqual(t.extras, {"BV": "8.1", "EPS_PAYLOAD": "ON"})

    def test_beacon_request(self):
        req = parse_beacon_req(parse(BEACON_REQ_LINE))
        self.assertEqual((req.seq, req.max_len), (42, 176))

    def test_status_request_and_shutdown(self):
        self.assertEqual(parse(STATUS_REQ_LINE).type, "STATUS_REQ")
        msg = parse(SHUTDOWN_LINE)
        self.assertEqual((msg.type, msg.fields), ("SHUTDOWN", ["POWER"]))

    def test_beacon_response_text_keeps_commas(self):
        msg = parse(BEACON_RSP_LINE)
        self.assertEqual(msg.fields, ["42", "Hello, world from orbit"])


class TestLimits(unittest.TestCase):
    """Limits agreed with jetson_link.h."""

    def test_limits_match_flight(self):
        self.assertEqual((LINE_MAX, TEXT_MAX), (512, 176))


class TestRejects(unittest.TestCase):
    """Malformed lines are refused."""

    def test_bad_crc(self):
        line = bytearray(TELEMETRY_LINE)
        line[12] = ord("9")
        with self.assertRaises(CrcError):
            parse(bytes(line))

    def test_format_errors(self):
        for line in (b"", b"STATUS_REQ,5d03\n", b"STATUS_REQ,5D0\n", b"STATUS_REQ\n",
                     b"BEACON_REQ,42\t,176,0000\n", b"X" * (LINE_MAX + 1)):
            with self.assertRaises(LinkError):
                parse(line)

    def test_beacon_text_rules(self):
        with self.assertRaises(LinkError):
            beacon_rsp(1, "a" * (TEXT_MAX + 1))
        with self.assertRaises(LinkError):
            beacon_rsp(1, "")
        with self.assertRaises(LinkError):
            beacon_rsp(1, "tab\there")
        with self.assertRaises(LinkError):
            beacon_rsp(70000, "hi")
        self.assertTrue(beacon_rsp(1, "a" * TEXT_MAX).startswith(b"BEACON_RSP,1,"))

    def test_beacon_request_limits(self):
        with self.assertRaises(LinkError):
            parse_beacon_req(parse(seal("BEACON_REQ,42,177")))
        with self.assertRaises(LinkError):
            parse_beacon_req(parse(seal("BEACON_REQ,x,176")))

    def test_status_ranges(self):
        with self.assertRaises(LinkError):
            status(4, 45, 6800, 37, 1234)
        with self.assertRaises(LinkError):
            status(0, 45, 6800, 101, 1234)


if __name__ == "__main__":
    unittest.main()
