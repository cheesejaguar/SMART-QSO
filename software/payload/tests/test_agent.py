"""
Unit tests for Agent Module

Tests JSON parsing, health data extraction, and stdio loop behavior
for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
from unittest.mock import patch, MagicMock
import json
import io

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from agent import parse_health_line


class TestParseHealthLine(unittest.TestCase):
    """Test parse_health_line function."""

    def test_valid_json(self):
        """Test parsing valid JSON health data."""
        line = json.dumps({
            "call": "SQSO-1",
            "time": "2026-01-02Z",
            "mode": "ACTIVE",
            "soc": 85,
            "sun": True,
            "rf": True,
            "pwr": "GOOD",
            "qso": 42
        })

        health = parse_health_line(line)

        self.assertIsNotNone(health)
        self.assertEqual(health.call, "SQSO-1")
        self.assertEqual(health.mode, "ACTIVE")
        self.assertEqual(health.soc, 85)
        self.assertEqual(health.sun, 1)
        self.assertEqual(health.rf, 1)
        self.assertEqual(health.pwr, "GOOD")
        self.assertEqual(health.qso, 42)

    def test_minimal_json(self):
        """Test parsing JSON with minimal fields."""
        line = json.dumps({})
        health = parse_health_line(line)

        self.assertIsNotNone(health)
        # Should use defaults
        self.assertEqual(health.call, "SMARTQ-1")
        self.assertEqual(health.mode, "IDLE")
        self.assertEqual(health.soc, 50)

    def test_invalid_json(self):
        """Test handling of invalid JSON."""
        health = parse_health_line("not valid json")
        self.assertIsNone(health)

    def test_empty_string(self):
        """Test handling of empty string."""
        health = parse_health_line("")
        self.assertIsNone(health)

    def test_sun_boolean_true(self):
        """Test sun field with boolean true."""
        line = json.dumps({"sun": True})
        health = parse_health_line(line)

        self.assertEqual(health.sun, 1)

    def test_sun_boolean_false(self):
        """Test sun field with boolean false."""
        line = json.dumps({"sun": False})
        health = parse_health_line(line)

        self.assertEqual(health.sun, 0)

    def test_rf_boolean_conversion(self):
        """Test rf field boolean conversion."""
        line_true = json.dumps({"rf": True})
        line_false = json.dumps({"rf": False})

        health_true = parse_health_line(line_true)
        health_false = parse_health_line(line_false)

        self.assertEqual(health_true.rf, 1)
        self.assertEqual(health_false.rf, 0)

    def test_mode_uppercase(self):
        """Test mode is converted to uppercase."""
        line = json.dumps({"mode": "active"})
        health = parse_health_line(line)

        self.assertEqual(health.mode, "ACTIVE")

    def test_pwr_uppercase(self):
        """Test pwr is converted to uppercase."""
        line = json.dumps({"pwr": "good"})
        health = parse_health_line(line)

        self.assertEqual(health.pwr, "GOOD")

    def test_soc_integer_conversion(self):
        """Test soc is converted to integer."""
        line = json.dumps({"soc": "85"})
        health = parse_health_line(line)

        # JSON strings should work
        self.assertIsInstance(health.soc, int)

    def test_qso_integer_conversion(self):
        """Test qso is converted to integer."""
        line = json.dumps({"qso": 42.5})
        health = parse_health_line(line)

        self.assertIsInstance(health.qso, int)
        self.assertEqual(health.qso, 42)


class TestParseHealthLineDefaults(unittest.TestCase):
    """Test default values in parse_health_line."""

    def test_default_call(self):
        """Test default callsign."""
        health = parse_health_line("{}")
        self.assertEqual(health.call, "SMARTQ-1")

    def test_default_mode(self):
        """Test default mode."""
        health = parse_health_line("{}")
        self.assertEqual(health.mode, "IDLE")

    def test_default_soc(self):
        """Test default SOC."""
        health = parse_health_line("{}")
        self.assertEqual(health.soc, 50)

    def test_default_sun(self):
        """Test default sun (True -> 1)."""
        health = parse_health_line("{}")
        self.assertEqual(health.sun, 1)

    def test_default_rf(self):
        """Test default rf (True -> 1)."""
        health = parse_health_line("{}")
        self.assertEqual(health.rf, 1)

    def test_default_pwr(self):
        """Test default power state."""
        health = parse_health_line("{}")
        self.assertEqual(health.pwr, "FAIR")

    def test_default_qso(self):
        """Test default QSO count."""
        health = parse_health_line("{}")
        self.assertEqual(health.qso, 0)


class TestParseHealthLineEdgeCases(unittest.TestCase):
    """Test edge cases in parse_health_line."""

    def test_extra_fields_ignored(self):
        """Test extra fields are ignored."""
        line = json.dumps({
            "call": "TEST",
            "extra_field": "ignored",
            "another": 123
        })
        health = parse_health_line(line)

        self.assertIsNotNone(health)
        self.assertEqual(health.call, "TEST")

    def test_null_values(self):
        """Test null values are handled."""
        line = json.dumps({
            "call": None,
            "soc": None
        })
        health = parse_health_line(line)

        # null values may cause parse to fail or use defaults
        # depending on implementation - just verify no crash
        if health is not None:
            self.assertIsInstance(health.call, str)

    def test_whitespace_in_json(self):
        """Test JSON with extra whitespace."""
        line = '  { "call" : "SQSO-1" , "soc" : 80 }  '
        health = parse_health_line(line)

        self.assertIsNotNone(health)
        self.assertEqual(health.call, "SQSO-1")

    def test_newline_in_input(self):
        """Test input with trailing newline."""
        line = '{"call": "SQSO-1"}\n'
        health = parse_health_line(line)

        # json.loads handles trailing whitespace
        self.assertIsNotNone(health)

    def test_unicode_callsign(self):
        """Test unicode in callsign is converted to string."""
        line = json.dumps({"call": "SQSO-1🛰️"})
        health = parse_health_line(line)

        self.assertIsNotNone(health)
        self.assertIsInstance(health.call, str)


class TestHealthDataIntegration(unittest.TestCase):
    """Integration tests for health data parsing."""

    def test_complete_health_data_flow(self):
        """Test complete health data parsing flow."""
        # Simulate OBC sending health data
        obc_data = {
            "call": "SQSO-1",
            "time": "2026-01-02T12:00:00Z",
            "mode": "ACTIVE",
            "soc": 78,
            "sun": True,
            "rf": True,
            "pwr": "GOOD",
            "qso": 15
        }

        line = json.dumps(obc_data)
        health = parse_health_line(line)

        self.assertIsNotNone(health)
        self.assertEqual(health.call, "SQSO-1")
        self.assertEqual(health.soc, 78)
        self.assertEqual(health.qso, 15)

    def test_safe_mode_health_data(self):
        """Test safe mode health data."""
        obc_data = {
            "call": "SQSO-1",
            "mode": "SAFE",
            "soc": 15,
            "sun": False,
            "rf": False,
            "pwr": "LOW",
            "qso": 0
        }

        line = json.dumps(obc_data)
        health = parse_health_line(line)

        self.assertEqual(health.mode, "SAFE")
        self.assertEqual(health.soc, 15)
        self.assertEqual(health.sun, 0)
        self.assertEqual(health.rf, 0)
        self.assertEqual(health.pwr, "LOW")


class TestRunStdioLoopMocked(unittest.TestCase):
    """Test run_stdio_loop with mocks."""

    @patch('agent.sys.stdin', io.StringIO(''))
    @patch.dict(os.environ, {'LLAMA_MODEL': '/test/model.gguf'})
    def test_empty_stdin(self):
        """Test empty stdin exits cleanly."""
        from agent import run_stdio_loop

        # Should not raise
        run_stdio_loop()

    @patch('agent.sys.stdin')
    @patch('agent.print')
    @patch.dict(os.environ, {'LLAMA_MODEL': '/test/model.gguf'})
    def test_processes_valid_line(self, mock_print, mock_stdin):
        """Test processing valid JSON line."""
        valid_json = json.dumps({
            "call": "SQSO-1",
            "mode": "ACTIVE",
            "soc": 80,
            "sun": True,
            "rf": True,
            "pwr": "GOOD",
            "qso": 10
        })

        mock_stdin.__iter__ = MagicMock(return_value=iter([valid_json]))

        from agent import run_stdio_loop
        run_stdio_loop()

        # Should have printed output
        mock_print.assert_called()

    @patch('agent.sys.stdin')
    @patch('agent.print')
    @patch.dict(os.environ, {'LLAMA_MODEL': '/test/model.gguf'})
    def test_skips_invalid_line(self, mock_print, mock_stdin):
        """Test skipping invalid JSON line."""
        mock_stdin.__iter__ = MagicMock(return_value=iter(['invalid json']))

        from agent import run_stdio_loop
        run_stdio_loop()

        # Should not print anything for invalid input
        mock_print.assert_not_called()


if __name__ == '__main__':
    unittest.main()
