from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Optional

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
        "You are SMART-QSO, a 1U CubeSat. Given health fields (TIME, MODE, "
        "SOC, SUN, RF, PWR, QSO, CALL), write one short, human-readable "
        "status line that begins with 'de <CALL>:' and ends with '73!' unless "
        "MODE=SAFE. Keep <=120 ASCII bytes; no emojis; plain text."
    )


def build_user_prompt(h: Health) -> str:
    return (
        f"CALL={h.call} TIME={h.time_iso_z} MODE={h.mode} SOC={h.soc} "
        f"SUN={h.sun} RF={h.rf} PWR={h.pwr} QSO={h.qso}"
    )


def fallback_text(h: Health) -> str:
    if h.mode == "SAFE" or h.soc < 25:
        return to_ascii(f"de {h.call}: Conserving power—SOC {h.soc}%.")
    if h.sun and h.rf:
        return to_ascii(
            f"de {h.call}: I am sunlit and operating—SOC {h.soc}%, "
            f"transponder ACTIVE. 73!"
        )
    if h.sun and not h.rf:
        return to_ascii(
            f"de {h.call}: Sunlit but conserving—SOC {h.soc}%. RF OFF. 73!"
        )
    if (not h.sun) and h.rf:
        return to_ascii(
            f"de {h.call}: Eclipse ops—RF ACTIVE, SOC {h.soc}%. 73!"
        )
    return to_ascii(
        f"de {h.call}: Coasting in eclipse, conserving power—SOC {h.soc}%."
    )


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
    text = to_ascii(text.replace("\u2014", " - ").replace("\u2013", "-"))
    return "".join(ch for ch in text if " " <= ch <= "~").strip()


def _strip_call(h: Health, text: str) -> str:
    prefix = f"de {h.call}:"
    if text.lower().startswith(prefix.lower()):
        return text[len(prefix):].strip()
    return text


def beacon_text(h: Health, generated: Optional[str], max_len: int) -> str:
    """Beacon text for a BEACON_RSP: printable ASCII, no call-sign prefix
    (the OBC adds "de <CALL>:" and the telemetry), at most max_len characters.
    Falls back to the template when generation is missing or empty."""
    text = _strip_call(h, _printable(generated or ""))
    if not text:
        text = _strip_call(h, _printable(fallback_text(h)))
    text = clamp_bytes(text, max_len).rstrip()
    return text or "73"
