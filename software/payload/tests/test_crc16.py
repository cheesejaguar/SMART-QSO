"""
Unit tests for the OBC-Jetson CRC-16 (ICD_OBC_JETSON.md §4.4).
"""

import unittest

from crc16 import CRC16_CHECK, CRC16_INIT, crc16_ccitt


class TestCrc16(unittest.TestCase):
    """Verify the CRC matches the ICD parameters and catalogue check value."""

    def test_check_value(self):
        # CRC-16/CCITT-FALSE catalogue check value, stated literally so a wrong
        # constant in crc16.py cannot hide a wrong CRC
        self.assertEqual(crc16_ccitt(b"123456789"), 0x29B1)
        self.assertEqual(CRC16_CHECK, 0x29B1)

    def test_empty_input_returns_init(self):
        self.assertEqual(crc16_ccitt(b""), 0xFFFF)
        self.assertEqual(CRC16_INIT, 0xFFFF)

    def test_incremental_matches_one_shot(self):
        data = b'{"command": 1, "timestamp": 12345}'
        partial = crc16_ccitt(data[:10])
        self.assertEqual(crc16_ccitt(data[10:], partial), crc16_ccitt(data))

    def test_detects_single_bit_error(self):
        data = bytearray(b"TELEMETRY,1000,SUNLIT,0.850")
        good = crc16_ccitt(bytes(data))
        data[3] ^= 0x01
        self.assertNotEqual(crc16_ccitt(bytes(data)), good)

    def test_result_is_16_bit(self):
        self.assertLessEqual(crc16_ccitt(bytes(range(256)) * 4), 0xFFFF)


if __name__ == "__main__":
    unittest.main()
