"""
Unit tests for Formatter Module

Tests beacon text formatting, health data structures, and prompt building
for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import time
import unittest
from datetime import datetime, timezone

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

        self.assertEqual(tail, "T=2026-01-02Z M=ACTIVE SOC=85 SUN=1 RF=1 PWR=GOOD QSO=42")


class TestComposeInfoField(unittest.TestCase):
    """Test compose_info_field function."""

    def test_includes_text_and_tail(self):
        """Test info field includes both text and tail."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        info = compose_info_field(health, "Hello from space!")

        self.assertEqual(info, "Hello from space! | " + build_tail(health))

    def test_truncates_long_text(self):
        """Test long text is truncated."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        long_text = "A" * 300
        info = compose_info_field(health, long_text)

        # The text is cut so that text, separator and tail fill exactly 200 bytes
        tail = build_tail(health)
        self.assertEqual(len(info), 200)
        self.assertEqual(info, "A" * (200 - 3 - len(tail)) + " | " + tail)

    def test_removes_unicode(self):
        """Test unicode is removed from text."""
        health = Health(
            call="SQSO-1", time_iso_z="2026-01-02Z",
            mode="ACTIVE", soc=85, sun=1, rf=1, pwr="GOOD", qso=42
        )
        info = compose_info_field(health, "Hello 🛰️ World")

        self.assertEqual(info, "Hello  World | " + build_tail(health))


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

    @unittest.skipUnless(hasattr(time, "tzset"), "time.tzset is POSIX-only")
    def test_uses_utc(self):
        """The date is the UTC date, whatever the local time zone.

        At any instant the local date at UTC+14 or at UTC-12 differs from the
        UTC date, so a local-time implementation fails in one of the two zones.
        """
        old_tz = os.environ.get("TZ")
        try:
            for tz in ("Etc/GMT-14", "Etc/GMT+12"):  # POSIX sign: UTC+14, UTC-12
                os.environ["TZ"] = tz
                time.tzset()
                before = datetime.now(timezone.utc).strftime("%Y-%m-%dZ")
                result = now_iso_z()
                after = datetime.now(timezone.utc).strftime("%Y-%m-%dZ")
                self.assertIn(result, {before, after}, tz)
        finally:
            if old_tz is None:
                os.environ.pop("TZ", None)
            else:
                os.environ["TZ"] = old_tz
            time.tzset()


if __name__ == '__main__':
    unittest.main()
