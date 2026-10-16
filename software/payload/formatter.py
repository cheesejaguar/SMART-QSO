from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Optional

try:
    import content_filter
except ImportError:  # running as software.payload.* from the repository root
    from software.payload import content_filter

#: Space station identity (licensee KK6GDB, decision D1); matches station_id.h
STATION_CALL = "KK6GDB-11"

_MODES = {0: "SAFE", 1: "IDLE", 2: "ACTIVE"}


@dataclass
class Health:
    call: str
    time_iso_z: str
    mode: str  # ACTIVE | IDLE | SAFE
    soc: int   # 0..100
    sun: int   # 0/1
    rf: int    # 0/1
    pwr: str   # GOOD | FAIR | LOW
    qso: int


def to_ascii(text: str) -> str:
    return text.encode("ascii", errors="ignore").decode("ascii")


def clamp_bytes(text: str, limit: int) -> str:
    data = text.encode("ascii", errors="ignore")
    if len(data) <= limit:
        return text
    return data[:limit].decode("ascii", errors="ignore")


def build_system_prompt() -> str:
    return (
        "You are SMART-QSO, a 1U CubeSat amateur radio satellite. Write one "
        "short, friendly sentence for radio amateurs about how you are doing, "
        "using the health words given. Rules: plain ASCII letters and basic "
        "punctuation, at most 120 characters; no numbers or digits (exact "
        "telemetry is added for you); no call signs (your CALL is added for "
        "you); no web addresses, prices, products or advertising; no "
        "profanity; no codes or abbreviations that hide meaning."
    )


def build_user_prompt(h: Health) -> str:
    """Health in words only, so the model has no numbers to repeat."""
    return (
        f"MODE={h.mode} SUN={'sunlit' if h.sun else 'eclipse'} "
        f"RF={'on' if h.rf else 'off'} PWR={h.pwr}"
    )


def fallback_text(h: Health) -> str:
    """Beacon text from health alone. No numbers: the OBC appends the
    measured telemetry, and stating values here could contradict it."""
    if h.mode == "SAFE":
        return f"de {h.call}: Safe mode, conserving power."
    if h.pwr == "LOW" or h.soc < 25:
        return f"de {h.call}: Battery low, conserving power."
    light = "Sunlit" if h.sun else "In eclipse"
    rf = "transponder on" if h.rf else "transponder off"
    return f"de {h.call}: {light}, {rf}. 73!"


def build_tail(h: Health) -> str:
    return (
        f"T={h.time_iso_z} M={h.mode} SOC={h.soc} SUN={h.sun} RF={h.rf} "
        f"PWR={h.pwr} QSO={h.qso}"
    )


def compose_info_field(h: Health, text: str) -> str:
    text = to_ascii(text)
    tail = build_tail(h)
    # include ' | ' separator (3 bytes)
    max_text = max(0, 200 - (3 + len(tail)))
    text = clamp_bytes(text, max_text)
    return f"{text} | {tail}"


def now_iso_z() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dZ")


def health_from_telemetry(t, call: str = STATION_CALL) -> Health:
    """Health fields for the beacon prompt from an OBC TELEMETRY message."""
    soc_pct = int(round(max(0.0, min(1.0, t.soc)) * 100))
    pwr = "GOOD" if soc_pct >= 60 else ("FAIR" if soc_pct >= 30 else "LOW")
    qso_text = t.extras.get("QSO", "0")
    return Health(
        call=call,
        time_iso_z=now_iso_z(),
        mode=_MODES.get(t.power_mode, "IDLE"),
        soc=soc_pct,
        sun=1 if t.sunlit else 0,
        rf=0 if t.extras.get("EPS_RADIO") == "OFF" else 1,
        pwr=pwr,
        qso=int(qso_text) if qso_text.isdigit() else 0,
    )


def _printable(text: str) -> str:
    """Printable ASCII; other characters become spaces, runs of spaces one."""
    text = text.replace("\u2014", " - ").replace("\u2013", "-")
    text = "".join(ch if " " <= ch <= "~" else " " for ch in text)
    return " ".join(text.split())


def _strip_call(h: Health, text: str) -> str:
    prefix = f"de {h.call}:"
    if text.lower().startswith(prefix.lower()):
        return text[len(prefix):].strip()
    return text


def _fit(text: str, max_len: int) -> str:
    """At most max_len characters, cut at a word boundary where possible."""
    if len(text) <= max_len:
        return text
    cut = text[:max_len]
    if text[max_len] != " " and " " in cut:
        cut = cut[:cut.rindex(" ")]
    return cut.rstrip()


def beacon_text(h: Health, generated: Optional[str], max_len: int) -> str:
    """Beacon text for a BEACON_RSP: no call-sign prefix (the OBC adds
    "de <CALL>:" and the telemetry), at most max_len characters, and passing
    the content filter the OBC applies (content_filter, SW-12). Generated
    text that is missing or fails the filter is replaced by the
    health-derived fallback."""
    max_len = min(max_len, content_filter.MAX_LEN)
    for candidate in (generated, fallback_text(h)):
        text = _fit(_strip_call(h, _printable(candidate or "")), max_len)
        if content_filter.check(text) == "OK":
            return text
    return "73" if max_len >= 2 else "."
