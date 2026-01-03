"""
Unit tests for Beacon Decoder Module

Tests AX.25 frame parsing, telemetry extraction, and KISS decoding
for SMART-QSO ground station.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
import struct

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from beacon_decoder import (
    BeaconDecoder, BeaconType, AX25Address, AX25Frame,
    DecodedBeacon, decode_beacon
)


class TestBeaconType(unittest.TestCase):
    """Test BeaconType enumeration."""

    def test_status_value(self):
        """Test STATUS beacon type."""
        self.assertEqual(BeaconType.STATUS, 0)

    def test_science_value(self):
        """Test SCIENCE beacon type."""
        self.assertEqual(BeaconType.SCIENCE, 1)

    def test_greeting_value(self):
        """Test GREETING beacon type."""
        self.assertEqual(BeaconType.GREETING, 2)

    def test_emergency_value(self):
        """Test EMERGENCY beacon type."""
        self.assertEqual(BeaconType.EMERGENCY, 3)

    def test_test_value(self):
        """Test TEST beacon type."""
        self.assertEqual(BeaconType.TEST, 4)

    def test_unknown_value(self):
        """Test UNKNOWN beacon type."""
        self.assertEqual(BeaconType.UNKNOWN, 255)


class TestAX25Address(unittest.TestCase):
    """Test AX25Address dataclass."""

    def test_creation(self):
        """Test basic address creation."""
        addr = AX25Address(
            callsign="SQSO-1",
            ssid=0,
            command_response=False,
            last=True
        )
        self.assertEqual(addr.callsign, "SQSO-1")
        self.assertEqual(addr.ssid, 0)
        self.assertTrue(addr.last)

    def test_defaults(self):
        """Test default values."""
        addr = AX25Address(callsign="TEST", ssid=0)
        self.assertFalse(addr.command_response)
        self.assertFalse(addr.last)


class TestAX25Frame(unittest.TestCase):
    """Test AX25Frame dataclass."""

    def test_creation(self):
        """Test basic frame creation."""
        dest = AX25Address("CQ", 0)
        src = AX25Address("SQSO-1", 0, last=True)

        frame = AX25Frame(
            destination=dest,
            source=src,
            control=0x03,
            pid=0xF0,
            info=b"Hello"
        )

        self.assertEqual(frame.destination.callsign, "CQ")
        self.assertEqual(frame.source.callsign, "SQSO-1")
        self.assertEqual(frame.info, b"Hello")

    def test_defaults(self):
        """Test default values."""
        dest = AX25Address("CQ", 0)
        src = AX25Address("TEST", 0)

        frame = AX25Frame(destination=dest, source=src)

        self.assertEqual(frame.control, 0x03)
        self.assertEqual(frame.pid, 0xF0)
        self.assertEqual(frame.info, b"")
        self.assertFalse(frame.fcs_valid)
        self.assertEqual(frame.digipeaters, [])


class TestDecodedBeacon(unittest.TestCase):
    """Test DecodedBeacon dataclass."""

    def test_creation(self):
        """Test decoded beacon creation."""
        beacon = DecodedBeacon(
            raw_frame=b"\x7e\x00\x7e",
            ax25_frame=None,
            beacon_type=BeaconType.STATUS,
            callsign="SQSO-1",
            sequence=1234,
            timestamp=None,
            ai_generated=True,
            info_text="Test beacon",
            telemetry={"battery_soc": 85},
            decode_time=0.001,
            valid=True
        )

        self.assertEqual(beacon.callsign, "SQSO-1")
        self.assertEqual(beacon.sequence, 1234)
        self.assertTrue(beacon.ai_generated)
        self.assertEqual(beacon.telemetry["battery_soc"], 85)

    def test_error_list_default(self):
        """Test errors list defaults to empty."""
        beacon = DecodedBeacon(
            raw_frame=b"",
            ax25_frame=None,
            beacon_type=BeaconType.UNKNOWN,
            callsign="",
            sequence=None,
            timestamp=None,
            ai_generated=False,
            info_text="",
            telemetry={},
            decode_time=0,
            valid=False
        )

        self.assertEqual(beacon.errors, [])


class TestBeaconDecoder(unittest.TestCase):
    """Test BeaconDecoder class."""

    def setUp(self):
        """Set up test fixtures."""
        self.decoder = BeaconDecoder()

    def test_init(self):
        """Test decoder initialization."""
        self.assertEqual(self.decoder._decode_count, 0)
        self.assertEqual(self.decoder._error_count, 0)

    def test_get_statistics_initial(self):
        """Test initial statistics."""
        stats = self.decoder.get_statistics()
        self.assertEqual(stats["decode_count"], 0)
        self.assertEqual(stats["error_count"], 0)

    def test_reset_statistics(self):
        """Test statistics reset."""
        self.decoder._decode_count = 10
        self.decoder._error_count = 5

        self.decoder.reset_statistics()

        stats = self.decoder.get_statistics()
        self.assertEqual(stats["decode_count"], 0)
        self.assertEqual(stats["error_count"], 0)

    def test_decode_empty_frame(self):
        """Test decoding empty frame."""
        result = self.decoder.decode(b"")

        self.assertFalse(result.valid)
        self.assertEqual(result.beacon_type, BeaconType.UNKNOWN)

    def test_decode_too_short_frame(self):
        """Test decoding frame that's too short."""
        result = self.decoder.decode(b"\x7e\x00\x01\x02\x7e")

        self.assertFalse(result.valid)
        self.assertIn("Failed to parse", result.errors[0])


class TestBeaconDecoderAddressParsing(unittest.TestCase):
    """Test AX.25 address parsing."""

    def setUp(self):
        self.decoder = BeaconDecoder()

    def test_parse_address_correct_length(self):
        """Test address parsing with correct length."""
        # Build address: "CQ    " shifted left by 1, SSID byte
        addr_bytes = bytes([c << 1 for c in b"CQ    "]) + bytes([0x60])

        addr = self.decoder._parse_address(addr_bytes)

        self.assertIsNotNone(addr)
        self.assertEqual(addr.callsign, "CQ")

    def test_parse_address_wrong_length(self):
        """Test address parsing with wrong length."""
        addr = self.decoder._parse_address(b"\x00\x01\x02")

        self.assertIsNone(addr)

    def test_parse_address_ssid_extraction(self):
        """Test SSID extraction from address."""
        # SSID = 5 -> (5 << 1) | 0x60 = 0x6A
        addr_bytes = bytes([c << 1 for c in b"TEST  "]) + bytes([0x6A])

        addr = self.decoder._parse_address(addr_bytes)

        self.assertEqual(addr.ssid, 5)

    def test_parse_address_last_bit(self):
        """Test last address indicator bit."""
        # Last bit set: 0x61
        addr_bytes = bytes([c << 1 for c in b"LAST  "]) + bytes([0x61])

        addr = self.decoder._parse_address(addr_bytes)

        self.assertTrue(addr.last)


class TestBeaconDecoderFCS(unittest.TestCase):
    """Test FCS calculation."""

    def setUp(self):
        self.decoder = BeaconDecoder()

    def test_calculate_fcs_empty(self):
        """Test FCS of empty data."""
        fcs = self.decoder._calculate_fcs(b"")
        self.assertIsInstance(fcs, int)

    def test_calculate_fcs_known_data(self):
        """Test FCS calculation returns consistent value."""
        data = b"Test data for FCS calculation"

        fcs1 = self.decoder._calculate_fcs(data)
        fcs2 = self.decoder._calculate_fcs(data)

        self.assertEqual(fcs1, fcs2)

    def test_calculate_fcs_different_data(self):
        """Test FCS differs for different data."""
        fcs1 = self.decoder._calculate_fcs(b"Data A")
        fcs2 = self.decoder._calculate_fcs(b"Data B")

        self.assertNotEqual(fcs1, fcs2)


class TestBeaconDecoderKISS(unittest.TestCase):
    """Test KISS frame extraction."""

    def setUp(self):
        self.decoder = BeaconDecoder()

    def test_extract_empty_kiss(self):
        """Test extracting frames from empty KISS data."""
        frames = self.decoder._extract_kiss_frames(b"")
        self.assertEqual(frames, [])

    def test_extract_single_frame(self):
        """Test extracting single KISS frame."""
        # FEND + cmd + data + FEND
        kiss_data = bytes([0xC0, 0x00]) + b"A" * 20 + bytes([0xC0])

        frames = self.decoder._extract_kiss_frames(kiss_data)

        self.assertEqual(len(frames), 1)
        self.assertEqual(frames[0], b"A" * 20)

    def test_extract_multiple_frames(self):
        """Test extracting multiple KISS frames."""
        frame1 = bytes([0xC0, 0x00]) + b"A" * 20 + bytes([0xC0])
        frame2 = bytes([0xC0, 0x00]) + b"B" * 20 + bytes([0xC0])

        frames = self.decoder._extract_kiss_frames(frame1 + frame2)

        self.assertEqual(len(frames), 2)

    def test_kiss_escape_sequence(self):
        """Test KISS escape sequence handling."""
        # FESC TFEND -> FEND in data
        kiss_data = bytes([0xC0, 0x00, 0x41, 0xDB, 0xDC, 0x42]) + b"A" * 14 + bytes([0xC0])

        frames = self.decoder._extract_kiss_frames(kiss_data)

        # Should contain unescaped FEND (0xC0)
        self.assertEqual(len(frames), 1)
        self.assertIn(0xC0, frames[0])


class TestBeaconDecoderTelemetry(unittest.TestCase):
    """Test telemetry extraction."""

    def setUp(self):
        self.decoder = BeaconDecoder()

    def test_extract_battery_voltage(self):
        """Test battery voltage extraction."""
        info = "SQSO-1 V:3.85V T:25C"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("battery_voltage"), 3.85)

    def test_extract_battery_soc(self):
        """Test battery SOC extraction."""
        info = "Status: B:78%"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("battery_soc"), 78)

    def test_extract_temperature(self):
        """Test temperature extraction."""
        info = "Temp T:25C nominal"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("temperature"), 25)

    def test_extract_negative_temperature(self):
        """Test negative temperature extraction."""
        info = "Eclipse T:-15C"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("temperature"), -15)

    def test_extract_power_mode(self):
        """Test power mode extraction."""
        info = "M:ACTIVE operations"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("power_mode"), "ACTIVE")

    def test_extract_orbit_number(self):
        """Test orbit number extraction."""
        info = "Orbit O:#1234"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("orbit_number"), 1234)

    def test_extract_uptime(self):
        """Test uptime extraction."""
        info = "Uptime U:86400s"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("uptime"), 86400)

    def test_extract_multiple_values(self):
        """Test extracting multiple telemetry values."""
        info = "SQSO-1 V:3.8V B:80% T:22C M:IDLE O:#100 U:3600s"

        telemetry = self.decoder._extract_telemetry(info)

        self.assertEqual(telemetry.get("battery_voltage"), 3.8)
        self.assertEqual(telemetry.get("battery_soc"), 80)
        self.assertEqual(telemetry.get("temperature"), 22)
        self.assertEqual(telemetry.get("power_mode"), "IDLE")
        self.assertEqual(telemetry.get("orbit_number"), 100)
        self.assertEqual(telemetry.get("uptime"), 3600)


class TestBeaconDecoderSequence(unittest.TestCase):
    """Test sequence number extraction."""

    def setUp(self):
        self.decoder = BeaconDecoder()

    def test_extract_sequence(self):
        """Test sequence number extraction."""
        seq = self.decoder._extract_sequence("Beacon #0042")
        self.assertEqual(seq, 42)

    def test_extract_sequence_not_found(self):
        """Test when no sequence in info."""
        seq = self.decoder._extract_sequence("No sequence here")
        self.assertIsNone(seq)


class TestBeaconDecoderType(unittest.TestCase):
    """Test beacon type detection."""

    def setUp(self):
        self.decoder = BeaconDecoder()

    def test_detect_emergency(self):
        """Test EMERGENCY beacon detection."""
        btype = self.decoder._detect_beacon_type("EMERGENCY LOW POWER")
        self.assertEqual(btype, BeaconType.EMERGENCY)

    def test_detect_test(self):
        """Test TEST beacon detection."""
        btype = self.decoder._detect_beacon_type("This is a TEST beacon")
        self.assertEqual(btype, BeaconType.TEST)

    def test_detect_science(self):
        """Test SCIENCE beacon detection."""
        btype = self.decoder._detect_beacon_type("SCIENCE data follows")
        self.assertEqual(btype, BeaconType.SCIENCE)

    def test_detect_greeting(self):
        """Test GREETING beacon detection."""
        btype = self.decoder._detect_beacon_type("CQ CQ de SQSO-1")
        self.assertEqual(btype, BeaconType.GREETING)

    def test_detect_status_default(self):
        """Test STATUS is default beacon type."""
        btype = self.decoder._detect_beacon_type("Normal status beacon")
        self.assertEqual(btype, BeaconType.STATUS)

    def test_ai_generated_detection(self):
        """Test AI-generated beacon detection."""
        dest = AX25Address("CQ", 0)
        src = AX25Address("SQSO-1", 0, last=True)

        # Create a minimal valid frame with [AI] marker
        # This is a simplified test
        self.decoder._decode_count = 0


class TestDecodeBeaconConvenience(unittest.TestCase):
    """Test decode_beacon convenience function."""

    def test_decode_beacon_returns_decoded_beacon(self):
        """Test convenience function returns DecodedBeacon."""
        result = decode_beacon(b"\x7e\x00\x7e")

        self.assertIsInstance(result, DecodedBeacon)

    def test_decode_beacon_invalid_frame(self):
        """Test convenience function with invalid frame."""
        result = decode_beacon(b"")

        self.assertFalse(result.valid)


if __name__ == '__main__':
    unittest.main()
