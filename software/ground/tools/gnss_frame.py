#!/usr/bin/env python3
"""
SMART-QSO GNSS state frames: decoding and GPS time (SQSO-GND-002 §3).

The flight software downlinks a GNSS state frame after each receiver session
that ended with a fix, and repeats the latest on GET_HOUSEKEEPING (0x09)
(software/rust/sqso-fsw/src/gnss_frame.rs, SRS-F158; telemetry dictionary
SQSO-TLM-001). Plain ASCII KEY=value, integers, `-` for a value not
available:

    GNS N=<session> WK=<GPS week> TOW=<GPS time of week, ms> X=<m> Y=<m> Z=<m>
        VX=<mm/s> VY=<mm/s> VZ=<mm/s> VDT=<s> Q=<GGA fix quality> SV=<satellites>
        PD=<PDOP x 10> AGE=<s>

    X, Y, Z, VX, VY, VZ: ECEF, WGS-84. The velocity is derived onboard from
    the session's first fix and this one, VDT seconds apart (its noise is
    about sqrt(2) x position sigma / VDT). WK as the receiver reports it (the
    piNAV-NG reports a 10-bit week, resolved here). Q: 1 GPS fix, 2
    differential; anything else is not a real fix. AGE: seconds from the fix
    to the frame (the repeat on GET_HOUSEKEEPING keeps N, WK and TOW, so it
    is dropped as a duplicate).

parse_gnss_frame() is the only function that knows this layout. Unknown keys
are ignored and decimals are accepted, so a frame with extra fields still
decodes.

Archive input (read_archive): JSON lines from the station or mission
database (`info` as text, or `frame_hex`: the AX.25 frame from the address
field, with or without FCS), or plain text lines holding the information
field or a hex AX.25 frame.
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable, List, Optional, Tuple

#: Information-field tag of the GNSS state frame (gnss_frame.rs PREFIX)
FRAME_TAG = "GNS"
#: Spacecraft call sign; frames from any other source are not ours
SPACECRAFT = ("KK6GDB", 11)

#: Start of GPS time, 1980-01-06T00:00:00Z, as a POSIX time
GPS_EPOCH_UNIX = 315_964_800
WEEK_S = 604_800
#: TAI - UTC since 2017-01-01 (IERS Bulletin C); GPS - UTC = TAI - UTC - 19 s
TAI_UTC_S = 37

#: Plausible LEO radius (the flight software's own acceptance limits, gnss.rs)
LEO_MIN_RADIUS_M = 6.5e6
LEO_MAX_RADIUS_M = 7.5e6
MIN_SATS = 4
REAL_FIX_QUALITY = (1, 2)
#: PDOP assumed when a frame does not carry one
DEFAULT_PDOP = 2.0


@dataclass(frozen=True)
class GnssFix:
    """One GNSS state record, as downlinked."""

    week: int            # GPS week, as reported (possibly modulo 1024)
    tow_ms: float        # GPS time of week, ms
    r_ecef_m: Tuple[float, float, float]
    v_ecef_ms: Optional[Tuple[float, float, float]]
    quality: Optional[int]
    sats: Optional[int]
    pdop: Optional[float]
    seq: Optional[int] = None
    vdt_s: Optional[float] = None   # chord the velocity was derived over
    age_s: Optional[float] = None   # fix age when the frame was sent

    def problems(self) -> List[str]:
        """Reasons this record must not be used (empty: usable)."""
        out = []
        if self.quality is not None and self.quality not in REAL_FIX_QUALITY:
            out.append(f"fix quality {self.quality} is not a real fix")
        if self.sats is not None and self.sats < MIN_SATS:
            out.append(f"{self.sats} satellites")
        radius = math.sqrt(sum(c * c for c in self.r_ecef_m))
        if not LEO_MIN_RADIUS_M <= radius <= LEO_MAX_RADIUS_M:
            out.append(f"radius {radius:.0f} m outside LEO")
        if not 0 <= self.tow_ms < WEEK_S * 1000:
            out.append("time of week out of range")
        if self.pdop is not None and self.pdop <= 0:
            out.append("PDOP not positive")
        return out


def _number(text: str) -> Optional[float]:
    if text == "-":
        return None
    return float(text)


def parse_gnss_frame(info) -> Optional[GnssFix]:
    """Decode one information field; None if it is not a GNSS state frame.

    Raises ValueError for a GNSS frame that lacks the time or the position.
    This is the single point of alignment with the telemetry dictionary.
    """
    if isinstance(info, (bytes, bytearray)):
        info = bytes(info).decode("ascii", errors="replace")
    info = info.strip()
    if not info.startswith(FRAME_TAG + " "):
        return None
    fields = {}
    for token in info[len(FRAME_TAG):].split():
        key, sep, value = token.partition("=")
        if sep:
            fields[key] = value

    def get(key: str) -> Optional[float]:
        value = fields.get(key)
        if value is None:
            return None
        try:
            return _number(value)
        except ValueError as exc:
            raise ValueError(f"GNSS frame field {key}={value!r} is not a number") from exc

    week, tow = get("WK"), get("TOW")
    pos = [get(k) for k in ("X", "Y", "Z")]
    if week is None or tow is None or any(p is None for p in pos):
        raise ValueError("GNSS frame without time or position")
    vel = [get(k) for k in ("VX", "VY", "VZ")]
    v_ecef = None if any(v is None for v in vel) else tuple(v / 1000.0 for v in vel)
    quality, sats, pdop10, seq = get("Q"), get("SV"), get("PD"), get("N")
    vdt, age = get("VDT"), get("AGE")
    return GnssFix(
        week=int(week),
        tow_ms=tow,
        r_ecef_m=(pos[0], pos[1], pos[2]),
        v_ecef_ms=v_ecef,
        quality=None if quality is None else int(quality),
        sats=None if sats is None else int(sats),
        pdop=None if pdop10 is None else pdop10 / 10.0,
        seq=None if seq is None else int(seq),
        vdt_s=None if v_ecef is None else vdt,
        age_s=age,
    )


def format_gnss_frame(fix: GnssFix) -> str:
    """The flight layout, for simulation and tests (inverse of parse_gnss_frame)."""
    def val(x, scale=1.0):
        return "-" if x is None else str(int(round(x * scale)))
    v = fix.v_ecef_ms or (None, None, None)
    return (f"{FRAME_TAG} N={val(fix.seq)} WK={fix.week} TOW={val(fix.tow_ms)} "
            f"X={val(fix.r_ecef_m[0])} Y={val(fix.r_ecef_m[1])} Z={val(fix.r_ecef_m[2])} "
            f"VX={val(v[0], 1000)} VY={val(v[1], 1000)} VZ={val(v[2], 1000)} VDT={val(fix.vdt_s)} "
            f"Q={val(fix.quality)} SV={val(fix.sats)} PD={val(fix.pdop, 10)} AGE={val(fix.age_s)}")


# --- GPS time ---------------------------------------------------------------

def gps_utc_offset_s(tai_utc_s: float = TAI_UTC_S) -> float:
    """GPS - UTC, s (18 s while TAI - UTC is 37 s)."""
    return tai_utc_s - 19.0


def resolve_week(week: int, near_unix: Optional[float] = None) -> int:
    """Full GPS week from a reported week that may be modulo 1024.

    With near_unix (when the frame was received) the week closest to it is
    chosen; otherwise the 1024-week era that began on 2019-04-07 (week 2048).
    A full week number (>= 1024) is returned as it is.
    """
    if week >= 1024:
        return week
    if near_unix is None:
        return week + 2048
    near_week = (near_unix - GPS_EPOCH_UNIX) / WEEK_S
    era = round((near_week - week) / 1024.0)
    return week + 1024 * max(era, 0)


def gps_to_unix(week: int, tow_s: float, tai_utc_s: float = TAI_UTC_S) -> float:
    """POSIX (UTC) time of a full GPS week and time of week."""
    return GPS_EPOCH_UNIX + week * WEEK_S + tow_s - gps_utc_offset_s(tai_utc_s)


def unix_to_gps(unix_s: float, tai_utc_s: float = TAI_UTC_S) -> Tuple[int, float]:
    """Full GPS week and time of week (s) of a POSIX (UTC) time."""
    gps = unix_s - GPS_EPOCH_UNIX + gps_utc_offset_s(tai_utc_s)
    week = int(gps // WEEK_S)
    return week, gps - week * WEEK_S


def fix_time_unix(fix: GnssFix, near_unix: Optional[float] = None,
                  tai_utc_s: float = TAI_UTC_S) -> float:
    """POSIX (UTC) time of a fix, with the week rollover resolved."""
    return gps_to_unix(resolve_week(fix.week, near_unix), fix.tow_ms / 1000.0, tai_utc_s)


# --- AX.25 and archives -----------------------------------------------------

def _callsign(field: bytes) -> Tuple[str, int]:
    call = "".join(chr(b >> 1) for b in field[:6]).strip()
    return call, (field[6] >> 1) & 0x0F


def ax25_info(frame: bytes) -> Optional[Tuple[Tuple[str, int], bytes]]:
    """(source, information field) of an AX.25 UI frame, or None.

    The frame starts at the address field. A trailing FCS is removed when it
    checks (with the shared protocol code, sqso_protocol, when installed).
    """
    for end in range(14, min(len(frame), 70) + 1, 7):
        if frame[end - 1] & 0x01:
            break
    else:
        return None
    if len(frame) < end + 2 or frame[end] != 0x03 or frame[end + 1] != 0xF0:
        return None
    source = _callsign(frame[7:14])
    info = frame[end + 2:]
    try:
        import sqso_protocol  # the Rust protocol code (software/rust/sqso-py)
        if len(info) >= 2 and sqso_protocol.ax25_fcs(frame[:-2]) == int.from_bytes(frame[-2:], "little"):
            info = info[:-2]
    except ImportError:
        pass
    return source, info


@dataclass(frozen=True)
class Received:
    """A decoded GNSS frame with its reception metadata."""

    fix: GnssFix
    rx_unix: Optional[float]
    station: Optional[str]


def _iso_to_unix(text: Optional[str]) -> Optional[float]:
    if not text:
        return None
    stamp = datetime.fromisoformat(text.replace("Z", "+00:00"))
    if stamp.tzinfo is None:
        stamp = stamp.replace(tzinfo=timezone.utc)
    return stamp.timestamp()


def _decode_line(line: str) -> Optional[Received]:
    line = line.strip()
    if not line or line.startswith("#"):
        return None
    rx, station, info = None, None, None
    if line.startswith("{"):
        record = json.loads(line)
        rx = _iso_to_unix(record.get("rx_utc") or record.get("utc"))
        station = record.get("station")
        if "info" in record:
            info = record["info"]
        elif "frame_hex" in record:
            info = bytes.fromhex(record["frame_hex"])
    elif FRAME_TAG + " " in line:
        info = line[line.index(FRAME_TAG + " "):]
    else:
        try:
            info = bytes.fromhex(line.split()[-1])
        except ValueError:
            return None
    if isinstance(info, bytes) and not info.startswith(FRAME_TAG.encode()):
        decoded = ax25_info(info)
        if decoded is None:
            return None
        source, info = decoded
        if source != SPACECRAFT:
            return None
    fix = parse_gnss_frame(info) if info is not None else None
    return None if fix is None else Received(fix, rx, station)


def read_archive(paths: Iterable[Path]) -> Tuple[List[Received], List[str]]:
    """Every GNSS frame in the files, and the lines that could not be decoded."""
    out, errors = [], []
    for path in paths:
        for number, line in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
            try:
                received = _decode_line(line)
            except (ValueError, KeyError) as exc:
                errors.append(f"{path}:{number}: {exc}")
                continue
            if received is not None:
                out.append(received)
    return out, errors
