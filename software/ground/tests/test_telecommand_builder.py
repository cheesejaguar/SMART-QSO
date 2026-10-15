"""
Unit tests for the authenticated telecommand builder

The byte vectors below are the same ones the flight software tests check
(software/flight/tests/test_cmd_auth.c, test_cmd_handler.c), so a change on
either side that breaks interoperability fails both suites.

The key is the TEST-ONLY key 00 01 .. 1F. It is never a flight key.
"""

import os
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "tools"))

from telecommand_builder import (  # noqa: E402
    COMMANDS,
    MAX_ARGS,
    build_frame,
    build_payload,
    load_key,
    next_counter,
)

TEST_KEY = bytes(range(32))

TX_DISABLE_CTR1 = bytes.fromhex("00000001 9A83F5C7B21F7511E82F5E94B9DE5851")
SET_BEACON_CTR2 = bytes.fromhex("00000002 003C B49634055CFFC160654E1B7E96DA3C13")
TX_ENABLE_CTR2 = bytes.fromhex("00000002 3B24D1FDF658A147FAA5F97B4BCF779C")
TX_DISABLE_CTR3 = bytes.fromhex("00000003 566CA34E13C96BA0DC25E0AA93DDD23B")
GET_STATUS_CTR1 = bytes.fromhex("00000001 81A953BF2B4C0361FB2CC17DA4789B12")

TX_DISABLE_CTR1_FRAME = bytes.fromhex(
    "AA55 32 14 00000001 9A83F5C7B21F7511E82F5E94B9DE5851 09BF")
TX_ENABLE_CTR3_FRAME = bytes.fromhex(
    "AA55 31 14 00000003 64B167923681E291200754A1BB670845 080A")


class TestPayloadVectors(unittest.TestCase):
    """Payloads match the flight software reference vectors."""

    def test_tx_disable_counter_1(self):
        self.assertEqual(build_payload(TEST_KEY, 0x32, 1), TX_DISABLE_CTR1)

    def test_set_beacon_counter_2_with_args(self):
        self.assertEqual(build_payload(TEST_KEY, 0x30, 2, bytes([0x00, 0x3C])), SET_BEACON_CTR2)

    def test_tx_enable_counter_2(self):
        self.assertEqual(build_payload(TEST_KEY, 0x31, 2), TX_ENABLE_CTR2)

    def test_tx_disable_counter_3(self):
        self.assertEqual(build_payload(TEST_KEY, 0x32, 3), TX_DISABLE_CTR3)

    def test_get_status_counter_1(self):
        self.assertEqual(build_payload(TEST_KEY, 0x03, 1), GET_STATUS_CTR1)


class TestFrameVectors(unittest.TestCase):
    """Complete uplink frames match the flight software reference vectors."""

    def test_tx_disable_frame(self):
        self.assertEqual(build_frame(TEST_KEY, 0x32, 1), TX_DISABLE_CTR1_FRAME)

    def test_tx_enable_frame(self):
        self.assertEqual(build_frame(TEST_KEY, 0x31, 3), TX_ENABLE_CTR3_FRAME)


class TestLimits(unittest.TestCase):
    """Inputs the flight software would reject are refused on the ground."""

    def test_counter_zero_rejected(self):
        with self.assertRaises(ValueError):
            build_payload(TEST_KEY, 0x32, 0)

    def test_counter_max_rejected(self):
        with self.assertRaises(ValueError):
            build_payload(TEST_KEY, 0x32, 0xFFFFFFFF)

    def test_counter_limit_accepted(self):
        self.assertEqual(len(build_payload(TEST_KEY, 0x32, 0xFFFFFFFE)), 20)

    def test_args_too_long_rejected(self):
        with self.assertRaises(ValueError):
            build_payload(TEST_KEY, 0x42, 1, bytes(MAX_ARGS + 1))

    def test_wrong_key_length_rejected(self):
        with self.assertRaises(ValueError):
            build_payload(TEST_KEY[:16], 0x32, 1)

    def test_dictionary_arg_length_enforced(self):
        with self.assertRaises(ValueError):
            build_frame(TEST_KEY, COMMANDS["TX_DISABLE"].cmd_id, 1, b"\x00", strict=True)


class TestDictionary(unittest.TestCase):
    """The ground dictionary matches the flight cmd_args_spec() exactly."""

    FLIGHT = {
        0x00: (0, 0), 0x01: (0, 0), 0x02: (1, 1), 0x03: (0, 0), 0x05: (0, 0),
        0x10: (1, 1), 0x13: (0, 0), 0x21: (0, 0), 0x23: (0, 0),
        0x30: (2, 2), 0x31: (0, 0), 0x32: (0, 0), 0x40: (0, 0), 0x41: (0, 0),
    }

    def test_matches_flight(self):
        ground = {spec.cmd_id: (spec.min_args, spec.max_args) for spec in COMMANDS.values()}
        self.assertEqual(ground, self.FLIGHT)

    def test_unimplemented_command_refused(self):
        with self.assertRaises(ValueError):
            build_frame(TEST_KEY, 0x33, 1, b"\x01")  # SET_POWER has no flight handler


class TestKeyFile(unittest.TestCase):
    """Key files are 64 hex characters; anything else is refused."""

    def test_load_hex_key(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "key.hex"
            path.write_text(TEST_KEY.hex() + "\n")
            os.chmod(path, 0o600)
            self.assertEqual(load_key(path), TEST_KEY)

    def test_short_key_refused(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "key.hex"
            path.write_text("00" * 16)
            os.chmod(path, 0o600)
            with self.assertRaises(ValueError):
                load_key(path)

    @unittest.skipIf(os.name != "posix", "file modes are POSIX-only")
    def test_world_readable_key_refused(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "key.hex"
            path.write_text(TEST_KEY.hex())
            os.chmod(path, 0o644)
            with self.assertRaises(PermissionError):
                load_key(path)


class TestCounterFile(unittest.TestCase):
    """Counters are reserved on disk before a frame is produced."""

    def test_counter_starts_at_one(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "counter"
            self.assertEqual(next_counter(path), 1)
            self.assertEqual(path.read_text().strip(), "1")

    def test_counter_increments(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "counter"
            path.write_text("41\n")
            self.assertEqual(next_counter(path), 42)
            self.assertEqual(next_counter(path), 43)

    def test_counter_exhausted(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "counter"
            path.write_text(str(0xFFFFFFFE))
            with self.assertRaises(ValueError):
                next_counter(path)


if __name__ == "__main__":
    unittest.main()
