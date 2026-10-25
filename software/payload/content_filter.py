"""
Beacon text content filter, Jetson side (SW-12).

The Jetson checks its own output so it never offers text the OBC would
refuse; the OBC still checks everything it receives. Both run the same
code: the Rust filter in software/rust/sqso-protocol/src/filter.rs, here
through the sqso_protocol module (software/rust/sqso-py). The rules are
documented there and in docs/BEACON_SPEC.md section 6.

check() returns "OK" or the first rule broken: LENGTH, CHARSET, ENCODED,
NUMERIC, CALLSIGN, PROFANITY, COMMERCIAL.
"""

from __future__ import annotations

from typing import Optional

import sqso_protocol

MAX_LEN = sqso_protocol.FILTER_MAX_LEN     # BEACON_FILTER_MAX_LEN
MAX_WORD = sqso_protocol.FILTER_MAX_WORD   # BEACON_FILTER_MAX_WORD

OWN_CALL = sqso_protocol.CALLSIGN.lower()  # station_id.h SQSO_CALLSIGN, lower case
OWN_SSID = sqso_protocol.SSID              # station_id.h SQSO_SSID

#: Obscene or indecent words (97.113(a)(4))
PROFANITY = sqso_protocol.PROFANITY

#: Business, advertising or pecuniary words (97.113(a)(3))
COMMERCIAL = sqso_protocol.COMMERCIAL

#: URL fragments, matched anywhere in the text
URL_MARKS = sqso_protocol.URL_MARKS


def check(text: Optional[str]) -> str:
    """"OK", or the first content rule the beacon text breaks."""
    return sqso_protocol.filter_check(text)
