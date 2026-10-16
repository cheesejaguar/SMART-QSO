"""
Beacon text content filter, Jetson side (SW-12).

The same rules, in the same order, as the OBC's software/flight/src/beacon_filter.c
(documented in software/flight/include/beacon_filter.h and docs/BEACON_SPEC.md
section 6). The Jetson checks its own output so it never offers text the OBC
would refuse; the OBC still checks everything it receives.

check() returns "OK" or the first rule broken: LENGTH, CHARSET, ENCODED,
NUMERIC, CALLSIGN, PROFANITY, COMMERCIAL.
"""

from __future__ import annotations

import re
from typing import Optional

MAX_LEN = 176   # BEACON_FILTER_MAX_LEN
MAX_WORD = 20   # BEACON_FILTER_MAX_WORD

OWN_CALL = "kk6gdb"   # station_id.h SQSO_CALLSIGN, lower case
OWN_SSID = 11         # station_id.h SQSO_SSID

#: Obscene or indecent words (97.113(a)(4)); identical to beacon_filter_profanity
PROFANITY = (
    "ass", "asshole", "bastard", "bitch", "bollocks", "bullshit", "cock",
    "cocksucker", "crap", "cunt", "damn", "dick", "dickhead", "dyke", "fag",
    "faggot", "fuck", "fucked", "fucker", "fucking", "goddamn", "motherfucker",
    "nigga", "nigger", "piss", "pissed", "prick", "pussy", "retard", "shit",
    "shitty", "slut", "spic", "twat", "wank", "wanker", "whore",
)

#: Business, advertising or pecuniary words (97.113(a)(3)); identical to beacon_filter_commercial
COMMERCIAL = (
    "advert", "advertise", "advertisement", "bitcoin", "buy", "buying", "cheap",
    "coupon", "crypto", "deal", "deals", "discount", "dollar", "dollars",
    "invest", "investment", "price", "prices", "profit", "promo", "promotion",
    "purchase", "sale", "sales", "sell", "selling", "shop", "sponsor",
    "sponsored", "subscribe", "usd",
)

#: URL fragments, matched anywhere in the text; identical to k_url_marks
URL_MARKS = ("http", "www.", ".com", ".net", ".org")

_ALLOWED = re.compile(r"[A-Za-z0-9 .,!?'\":;()/&-]*")
_WORD = re.compile(r"[A-Za-z0-9]+")
_CALLSIGN = re.compile(r"(?:[a-z]{1,2}|[0-9][a-z]|[a-z][0-9])[0-9][a-z]{1,4}")
_HEX = re.compile(r"[0-9a-f]+")
_VOWELS = set("aeiouy")


def _looks_encoded(word: str) -> bool:
    low = word.lower()
    case_changes = sum(1 for a, b in zip(word, word[1:]) if a.islower() and b.isupper())
    return ((word.isalpha() and len(word) >= 5 and not _VOWELS & set(low))
            or (len(word) >= 8 and bool(_HEX.fullmatch(low)))
            or case_changes > 2)


def _check_ssid(word: str) -> str:
    if not word.isdigit() or len(word) > 2:
        return "NUMERIC"
    return "OK" if int(word) == OWN_SSID else "CALLSIGN"


def _check_word(word: str, after_own_call: bool) -> str:
    low = word.lower()
    if len(word) > MAX_WORD:
        return "ENCODED"
    if after_own_call and word[0].isdigit():
        return _check_ssid(word)
    if _CALLSIGN.fullmatch(low) and low != OWN_CALL:
        return "CALLSIGN"
    if any(c.isdigit() for c in word) and low not in ("73", "1u", OWN_CALL):
        return "NUMERIC"
    if _looks_encoded(word):
        return "ENCODED"
    if low in PROFANITY:
        return "PROFANITY"
    if low in COMMERCIAL:
        return "COMMERCIAL"
    return "OK"


def check(text: Optional[str]) -> str:
    """"OK", or the first content rule the beacon text breaks."""
    if not text or len(text) > MAX_LEN:
        return "LENGTH"
    if not _ALLOWED.fullmatch(text):
        return "CHARSET"
    low = text.lower()
    if any(mark in low for mark in URL_MARKS):
        return "COMMERCIAL"
    prev_end, prev_own = -1, False
    for m in _WORD.finditer(text):
        after_own = prev_own and m.start() == prev_end + 1 and text[prev_end] == "-"
        result = _check_word(m.group(), after_own)
        if result != "OK":
            return result
        prev_own, prev_end = m.group().lower() == OWN_CALL, m.end()
    return "OK"
