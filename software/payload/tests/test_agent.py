"""
Unit tests for the beacon agent (Jetson side of the OBC-Jetson link).

The agent answers the OBC's BEACON_REQ with a BEACON_RSP line carrying only
the beacon text; the OBC adds the call sign and telemetry. Golden lines are
shared with software/flight/tests/test_jetson_link.c.
"""

import unittest

from agent import AgentState, handle_line
from formatter import Health, STATION_CALL, beacon_text, health_from_telemetry
import obc_link

TELEMETRY_LINE = b"TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86\n"
BEACON_REQ_LINE = b"BEACON_REQ,42,176,5593\n"


def _health(**kw):
    base = dict(call=STATION_CALL, time_iso_z="2026-10-15Z", mode="ACTIVE", soc=75,
                sun=1, rf=1, pwr="GOOD", qso=0)
    base.update(kw)
    return Health(**base)


class TestHealthFromTelemetry(unittest.TestCase):

    def test_fields(self):
        t = obc_link.parse_telemetry(obc_link.parse(TELEMETRY_LINE))
        h = health_from_telemetry(t)
        self.assertEqual(h.call, "KK6GDB-11")
        self.assertEqual(h.mode, "ACTIVE")
        self.assertEqual(h.soc, 75)
        self.assertEqual(h.sun, 1)
        self.assertEqual(h.pwr, "GOOD")

    def test_power_bands_and_modes(self):
        for soc, mode, pwr, name in ((0.20, 0, "LOW", "SAFE"), (0.45, 1, "FAIR", "IDLE")):
            line = obc_link.seal(f"TELEMETRY,1,ECLIPSE,{soc:.3f},{mode},1.0")
            h = health_from_telemetry(obc_link.parse_telemetry(obc_link.parse(line)))
            self.assertEqual((h.pwr, h.mode, h.sun), (pwr, name, 0))


class TestBeaconText(unittest.TestCase):

    def test_prefix_stripped_and_clamped(self):
        text = beacon_text(_health(), "de KK6GDB-11: Hello from orbit 73!", 10)
        self.assertEqual(text, "Hello from")

    def test_fallback_when_generation_missing(self):
        text = beacon_text(_health(mode="SAFE", soc=18), None, 176)
        self.assertTrue(0 < len(text) <= 176)
        self.assertFalse(text.startswith("de "))

    def test_non_ascii_and_blank_generation_falls_back(self):
        self.assertTrue(beacon_text(_health(), "   ", 176))
        self.assertTrue(beacon_text(_health(), "éè", 176).isascii())


class TestHandleLine(unittest.TestCase):

    def test_telemetry_then_beacon_request(self):
        state = AgentState()
        self.assertIsNone(handle_line(TELEMETRY_LINE, state))
        reply = handle_line(BEACON_REQ_LINE, state)
        msg = obc_link.parse(reply)
        self.assertEqual(msg.type, "BEACON_RSP")
        self.assertEqual(msg.fields[0], "42")
        self.assertTrue(0 < len(msg.fields[1]) <= 176)

    def test_beacon_request_respects_max_len(self):
        state = AgentState()
        reply = handle_line(obc_link.seal("BEACON_REQ,7,12"), state)
        msg = obc_link.parse(reply)
        self.assertTrue(len(msg.fields[1]) <= 12)

    def test_bad_crc_nacked(self):
        reply = handle_line(b"BEACON_REQ,42,176,5594\n", AgentState())
        self.assertEqual(reply, obc_link.nack("CRC"))

    def test_malformed_nacked(self):
        self.assertEqual(handle_line(b"nonsense\n", AgentState()), obc_link.nack("FORMAT"))

    def test_supervisor_messages_ignored(self):
        self.assertIsNone(handle_line(b"STATUS_REQ,5D03\n", AgentState()))
        self.assertIsNone(handle_line(b"SHUTDOWN,POWER,05D4\n", AgentState()))


if __name__ == "__main__":
    unittest.main()
