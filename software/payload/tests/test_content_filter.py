"""
Beacon text content filter, Jetson side (SW-12, 47 CFR 97.113(a)(3)-(4)).

The vectors are shared with software/flight/tests/test_beacon_filter.c, and
the word lists are compared with software/flight/src/beacon_filter.c, so the
Jetson never offers text the OBC would refuse.
"""

import random
import re
import unittest
from pathlib import Path

import content_filter
from content_filter import COMMERCIAL, MAX_LEN, PROFANITY, check
from formatter import STATION_CALL, Health, beacon_text, fallback_text

_FLIGHT = Path(__file__).resolve().parents[2] / "flight"
VECTORS = _FLIGHT / "tests" / "vectors" / "beacon_filter_vectors.txt"
FLIGHT_FILTER = _FLIGHT / "src" / "beacon_filter.c"


def _vectors():
    for line in VECTORS.read_text(encoding="utf-8").splitlines():
        if line and not line.startswith("#"):
            expected, _, text = line.partition("|")
            yield expected, text


def _c_list(name):
    source = FLIGHT_FILTER.read_text(encoding="ascii")
    body = re.search(name + r"\[\] = \{(.*?)\};", source, re.S).group(1)
    return tuple(re.findall(r'"([^"]*)"', body))


def _health(**kw):
    base = dict(call=STATION_CALL, time_iso_z="2026-10-15Z", mode="ACTIVE", soc=75,
                sun=1, rf=1, pwr="GOOD", qso=0)
    base.update(kw)
    return Health(**base)


class TestSharedVectors(unittest.TestCase):

    def test_every_vector(self):
        count = 0
        for expected, text in _vectors():
            with self.subTest(text=text):
                self.assertEqual(check(text), expected)
            count += 1
        self.assertGreaterEqual(count, 40)


class TestWordLists(unittest.TestCase):

    def test_lists_match_flight(self):
        self.assertEqual(PROFANITY, _c_list("beacon_filter_profanity"))
        self.assertEqual(COMMERCIAL, _c_list("beacon_filter_commercial"))
        self.assertEqual(content_filter.URL_MARKS, _c_list("s_url_marks"))

    def test_each_word_rejected_whole_word_any_case(self):
        for words, reason in ((PROFANITY, "PROFANITY"), (COMMERCIAL, "COMMERCIAL")):
            for word in words:
                with self.subTest(word=word):
                    self.assertEqual(check(f"Hello {word} world"), reason)
                    self.assertEqual(check(f"HELLO {word.upper()} WORLD"), reason)
                    self.assertNotEqual(check(f"Hello x{word}x world"), reason)


class TestEdgeCases(unittest.TestCase):

    def test_nul_and_none(self):
        self.assertEqual(check("Hello\0world"), "CHARSET")
        self.assertEqual(check(None), "LENGTH")


class TestFallbackText(unittest.TestCase):
    """The health-derived fallback is always acceptable beacon text."""

    def test_every_health_state_passes(self):
        for mode in ("ACTIVE", "IDLE", "SAFE"):
            for pwr in ("GOOD", "FAIR", "LOW"):
                for soc in (0, 24, 25, 60, 100):
                    for sun in (0, 1):
                        for rf in (0, 1):
                            h = _health(mode=mode, pwr=pwr, soc=soc, sun=sun, rf=rf, qso=999)
                            with self.subTest(h=h):
                                text = beacon_text(h, None, MAX_LEN)
                                self.assertEqual(check(text), "OK")
                                # No telemetry values; "73" is the sign-off
                                body = fallback_text(h).split(":", 1)[1]
                                self.assertNotRegex(body.replace("73!", ""), r"\d")


class TestBeaconTextFuzz(unittest.TestCase):
    """Whatever the model writes, the BEACON_RSP text passes the OBC filter."""

    POOL = ("hello", "orbit", "from", "73", "KK6GDB-11", "sunlit", "transponder",
            "W1AW", "99", "$", "|", "\t", "deadbeefcafe", "SGVsbG", "http", "buy",
            "KK6GDB-12", "Supercalifragilisticexpialidocious", "www.", ".com",
            "—", "é", "de KK6GDB-11:", "SOC=75%", "\n", PROFANITY[0])

    def test_output_always_passes(self):
        rng = random.Random(0x5EED)
        for _ in range(5000):
            if rng.random() < 0.25:
                generated = "".join(chr(rng.randrange(0, 0x250)) for _ in range(rng.randrange(0, 220)))
            else:
                generated = "".join(rng.choice(self.POOL) + rng.choice(("", " ", ", ", "-", "! "))
                                    for _ in range(rng.randrange(1, 14)))
            max_len = rng.choice((1, 2, 12, 40, 120, MAX_LEN))
            h = _health(mode=rng.choice(("ACTIVE", "IDLE", "SAFE")), sun=rng.randrange(2),
                        rf=rng.randrange(2), pwr=rng.choice(("GOOD", "FAIR", "LOW")))
            text = beacon_text(h, generated, max_len)
            self.assertTrue(0 < len(text) <= max_len, (generated, text))
            self.assertEqual(check(text), "OK", (generated, text))

    def test_rejected_generation_falls_back_to_health(self):
        h = _health(sun=0, rf=1)
        expected = beacon_text(h, None, MAX_LEN)
        for bad in ("Battery at 99 percent", "de W1AW hello", "Buy our kits",
                    "Hello | T=0 M=SAFE", "Visit www.example.org"):
            with self.subTest(bad=bad):
                self.assertEqual(beacon_text(h, bad, MAX_LEN), expected)

    def test_accepted_generation_kept(self):
        self.assertEqual(beacon_text(_health(), "Hello, world from orbit", MAX_LEN),
                         "Hello, world from orbit")

    def test_never_longer_than_the_obc_accepts(self):
        text = beacon_text(_health(), "hello " * 40, 500)
        self.assertTrue(text.startswith("hello hello"))
        self.assertTrue(150 < len(text) <= MAX_LEN)

    def test_control_characters_separate_words(self):
        self.assertEqual(beacon_text(_health(), "Hello\nworld\tfrom\x00orbit", MAX_LEN),
                         "Hello world from orbit")

    def test_cut_at_word_boundary(self):
        self.assertEqual(beacon_text(_health(), "Hello from the transponder", 16), "Hello from the")


if __name__ == "__main__":
    unittest.main()
