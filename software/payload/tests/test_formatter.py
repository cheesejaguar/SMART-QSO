"""
Unit tests for Formatter Module

Tests beacon text formatting, health data structures, and prompt building
for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
from unittest.mock import patch

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from formatter import (
    Health, to_ascii, clamp_bytes, build_system_prompt,
    build_user_prompt, fallback_text, build_tail, compose_info_field,
    now_iso_z
)


class TestHealth(unittest.TestCase):
    """Test Health dataclass."""

    def test_creation(self):
        """Test basic Health creation."""
        health = Health(
            call="SQSO-1",
            time_iso_z="2026-01-02Z",
            mode="ACTIVE",
            soc=85,
            sun=1,
            rf=1,
            pwr="GOOD",
            qso=42
        )
        self.assertEqual(health.call, "SQSO-1")
        self.assertEqual(health.soc, 85)
        self.assertEqual(health.sun, 1)

    def test_all_modes(self):
        """Test all valid modes."""
        for mode in ["ACTIVE", "IDLE", "SAFE"]:
            health = Health(
                call="TEST", time_iso_z="2026-01-02Z",
                mode=mode, soc=50, sun=1, rf=1, pwr="GOOD", qso=0
            )
            self.assertEqual(health.mode, mode)

    def test_all_power_states(self):
        """Test all valid power states."""
        for pwr in ["GOOD", "FAIR", "LOW"]:
            health = Health(
                call="TEST", time_iso_z="2026-01-02Z",
                mode="ACTIVE", soc=50, sun=1, rf=1, pwr=pwr, qso=0
            )
            self.assertEqual(health.pwr, pwr)

    def test_soc_range(self):
        """Test SOC values at boundaries."""
        for soc in [0, 50, 100]:
            health = Health(
                call="TEST", time_iso_z="2026-01-02Z",
                mode="ACTIVE", soc=soc, sun=1, rf=1, pwr="GOOD", qso=0
            )
            self.assertEqual(health.soc, soc)

    def test_sun_values(self):
        """Test sun flag values."""
        for sun in [0, 1]:
            health = Health(
                call="TEST", time_iso_z="2026-01-02Z",
                mode="ACTIVE", soc=50, sun=sun, rf=1, pwr="GOOD", qso=0
            )
            self.assertEqual(health.sun, sun)


class TestToAscii(unittest.TestCase):
    """Test to_ascii function."""

    def test_ascii_passthrough(self):
        """Test ASCII text passes through unchanged."""
        self.assertEqual(to_ascii("Hello World"), "Hello World")

    def test_unicode_removed(self):
        """Test non-ASCII characters are removed."""
        result = to_ascii("Hello 🛰️ World")
        self.assertEqual(result, "Hello  World")

    def test_accents_removed(self):
        """Test accented characters are removed."""
        result = to_ascii("café résumé")
        self.assertEqual(result, "caf rsum")

    def test_empty_string(self):
        """Test empty string handling."""
        self.assertEqual(to_ascii(""), "")

    def test_pure_unicode(self):
        """Test pure unicode string."""
        result = to_ascii("🛰️🌍🚀")
        self.assertEqual(result, "")


class TestClampBytes(unittest.TestCase):
    """Test clamp_bytes function."""

    def test_short_text_unchanged(self):
        """Test short text passes through unchanged."""
        self.assertEqual(clamp_bytes("Hello", 10), "Hello")

    def test_exact_limit(self):
        """Test text at exact limit."""
        self.assertEqual(clamp_bytes("Hello", 5), "Hello")

    def test_truncation(self):
        """Test text truncation."""
        result = clamp_bytes("Hello World", 5)
        self.assertEqual(result, "Hello")

    def test_unicode_truncation(self):
        """Test unicode is removed before truncation."""
        result = clamp_bytes("Hello 🛰️", 10)
        # Unicode emoji is removed, leaving "Hello " (possibly with trailing space)
        self.assertLessEqual(len(result), 10)
        self.assertTrue(result.startswith("Hello"))

    def test_zero_limit(self):
        """Test zero byte limit."""
        result = clamp_bytes("Hello", 0)
        self.assertEqual(result, "")


class TestBuildSystemPrompt(unittest.TestCase):
    """Test build_system_prompt function."""

    def test_contains_smart_qso(self):
        """Test prompt contains SMART-QSO."""
        prompt = build_system_prompt()
        self.assertIn("SMART-QSO", prompt)

    def test_contains_cubesat(self):
        """Test prompt mentions CubeSat."""
        prompt = build_system_prompt()
        self.assertIn("CubeSat", prompt)

    def test_mentions_call(self):
        """Test prompt mentions CALL field."""
        prompt = build_system_prompt()
        self.assertIn("CALL", prompt)

    def test_ascii_constraint(self):
        """Test prompt mentions ASCII constraint."""
        prompt = build_system_prompt()
        self.assertIn("ASCII", prompt)

    def test_byte_limit(self):
        """Test prompt mentions byte limit."""
        prompt = build_system_prompt()
        self.assertIn("120", prompt)


class TestBuildUserPrompt(unittest.TestCase):
    """Test build_user_prompt function."""

    def test_health_in_words_only(self):
        """Health appears as words; no numbers for the model to repeat."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        prompt = build_user_prompt(health)

        self.assertIn("MODE=ACTIVE", prompt)
        self.assertIn("SUN=sunlit", prompt)
        self.assertIn("RF=on", prompt)
        self.assertIn("PWR=GOOD", prompt)
        self.assertNotRegex(prompt, r"\d")

    def test_eclipse_and_rf_off(self):
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="IDLE", soc=40, sun=0, rf=0, pwr="FAIR", qso=0
        )
        prompt = build_user_prompt(health)
        self.assertIn("SUN=eclipse", prompt)
        self.assertIn("RF=off", prompt)


def _h(**kw):
    base = dict(call="SQSO-1", time_iso_z="2026-01-02Z", mode="ACTIVE", soc=85,
                sun=1, rf=1, pwr="GOOD", qso=0)
    base.update(kw)
    return Health(**base)


class TestFallbackText(unittest.TestCase):
    """fallback_text: health in words, no telemetry values (SW-12)."""

    def test_safe_mode_text(self):
        text = fallback_text(_h(mode="SAFE", soc=50, pwr="LOW"))
        self.assertEqual(text, "de SQSO-1: Safe mode, conserving power.")
        self.assertNotIn("73", text)  # No 73 in SAFE mode

    def test_low_battery_text(self):
        self.assertEqual(fallback_text(_h(soc=20, pwr="LOW")),
                         "de SQSO-1: Battery low, conserving power.")
        self.assertEqual(fallback_text(_h(soc=24, pwr="FAIR")),
                         "de SQSO-1: Battery low, conserving power.")

    def test_sun_and_transponder_states(self):
        cases = (
            (1, 1, "de SQSO-1: Sunlit, transponder on. 73!"),
            (1, 0, "de SQSO-1: Sunlit, transponder off. 73!"),
            (0, 1, "de SQSO-1: In eclipse, transponder on. 73!"),
            (0, 0, "de SQSO-1: In eclipse, transponder off. 73!"),
        )
        for sun, rf, expected in cases:
            with self.subTest(sun=sun, rf=rf):
                self.assertEqual(fallback_text(_h(sun=sun, rf=rf, soc=25, pwr="FAIR")), expected)

    def test_fallback_is_ascii(self):
        """Test fallback text is pure ASCII."""
        fallback_text(_h(qso=42)).encode("ascii")


class TestBuildTail(unittest.TestCase):
    """Test build_tail function."""

    def test_contains_all_fields(self):
        """Test tail contains all health fields."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        tail = build_tail(health)

        self.assertIn("T=2026-01-02Z", tail)
        self.assertIn("M=ACTIVE", tail)
        self.assertIn("SOC=85", tail)
        self.assertIn("SUN=1", tail)
        self.assertIn("RF=1", tail)
        self.assertIn("PWR=GOOD", tail)
        self.assertIn("QSO=42", tail)


class TestComposeInfoField(unittest.TestCase):
    """Test compose_info_field function."""

    def test_includes_text_and_tail(self):
        """Test info field includes both text and tail."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        info = compose_info_field(health, "Hello from space!")

        self.assertIn("Hello from space!", info)
        self.assertIn(" | ", info)
        self.assertIn("SOC=85", info)

    def test_truncates_long_text(self):
        """Test long text is truncated."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        long_text = "A" * 300
        info = compose_info_field(health, long_text)

        self.assertLessEqual(len(info), 200)

    def test_removes_unicode(self):
        """Test unicode is removed from text."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        info = compose_info_field(health, "Hello 🛰️ World")

        # Should not contain emoji
        info.encode('ascii')  # Should not raise


class TestNowIsoZ(unittest.TestCase):
    """Test now_iso_z function."""

    def test_format(self):
        """Test date format."""
        result = now_iso_z()

        # Should match YYYY-MM-DDZ format
        self.assertRegex(result, r'^\d{4}-\d{2}-\d{2}Z$')

    def test_ends_with_z(self):
        """Test ends with Z for UTC."""
        result = now_iso_z()
        self.assertTrue(result.endswith('Z'))

    @patch('formatter.datetime')
    def test_uses_utc(self, mock_datetime):
        """Test uses UTC timezone."""
        from datetime import timezone

        mock_now = mock_datetime.now
        mock_now.return_value.strftime.return_value = "2026-01-02Z"

        result = now_iso_z()

        mock_now.assert_called_once()
        args = mock_now.call_args[0]
        self.assertEqual(args[0], timezone.utc)


if __name__ == '__main__':
    unittest.main()
