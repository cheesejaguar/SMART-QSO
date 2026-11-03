#!/usr/bin/env python3
"""SMART-QSO downlink decoder: the beacon and the housekeeping texts.

Document ID: SMART-QSO-GND-001
Version: 2.0

The one ground decoder of the spacecraft's text frames (AX.25 UI frames from
KK6GDB-11 to CQ):

* the beacon, ``de KK6GDB-11: <text> | T=.. M=.. SOC=.. BV=.. SUN=.. RF=..
  QSO=..`` (docs/BEACON_SPEC.md §3-§4; flight formatter
  software/rust/sqso-fsw/src/beacon.rs), every field, ``-`` read as not
  measured;
* the housekeeping texts ``HK1`` and ``HK2`` (SQSO-TLM-001 §4.4, §4.4.1;
  software/rust/sqso-fsw/src/housekeeping.rs), every field, the frames of a
  text longer than one frame merged by their ``HKn N=<seq>`` head;
* the GNSS state frame ``GNS`` (through gnss_frame.py) and the command
  acknowledgement ``ack KK6GDB-11: C=.. R=.. N=..``.

The stored-housekeeping dump frames (``HKD``, binary, SQSO-ICD-005 §5.6) are
recognised but not decoded.

AX.25 and KISS come from ``sqso_protocol``, the flight software's Rust wire
formats (``pip install ./software/rust/sqso-py`` from the repository root).

Input: frames in hex (arguments, ``-f FILE``, ``--stdin``; one frame per line,
the hex as the line's last word, or a JSON line with ``frame_hex``), or a live
KISS byte stream (``--kiss-tcp HOST:PORT``, e.g. Direwolf's port 8001, or
``--kiss-file PATH`` for a serial device set up with ``stty`` or a recorded
stream). Output: one JSON object per line.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import socket
import sys
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import IO, Any, Dict, Iterable, Iterator, List, Optional, Sequence, Tuple

try:
    import sqso_protocol  # the Rust protocol code (software/rust/sqso-py)
except ImportError as exc:  # pragma: no cover - the CI installs it
    raise ImportError(
        "beacon_decoder needs sqso_protocol: pip install ./software/rust/sqso-py "
        "(from the repository root; it needs a Rust toolchain)"
    ) from exc

SPACECRAFT = "KK6GDB-11"
"""The source address of every frame from the spacecraft (station_id.h)."""

BEACON_PREFIX = f"de {SPACECRAFT}: "
ACK_PREFIX = f"ack {SPACECRAFT}: "

POWER_MODES = ("SAFE", "IDLE", "ACTIVE")
"""Beacon ``M=`` values (PowerMode_t 0, 1, 2)."""

# --- Field definitions ------------------------------------------------------
#
# A field is KEY=value; a value with several parts separates them with "/".
# Each part is an integer (or "-": not measured), turned into engineering units
# by its scale. A field sent as a single "-" (e.g. FL=- with no fault logged)
# leaves every part None.


@dataclass(frozen=True)
class Part:
    """One number of a field: its name in the output and its scale."""

    name: str
    scale: float = 1.0
    digits: int = 0


def _p(name: str, scale: float = 1.0) -> Part:
    digits = 0
    s = scale
    while s < 1.0 and digits < 6:
        s *= 10.0
        digits += 1
    return Part(name, scale, digits)


# Beacon telemetry, BEACON_SPEC §4, in the order sent
BEACON_FIELDS: Dict[str, Part] = {
    "T": _p("mission_time_s"),  # seconds since the last OBC boot, not UTC
    "M": _p("power_mode"),  # SAFE / IDLE / ACTIVE
    "SOC": _p("soc_pct"),
    "BV": _p("battery_v"),  # volts to one decimal
    "SUN": _p("sunlit"),
    "RF": _p("rf_power"),
    "QSO": _p("qso_count"),
}

# HK1, SQSO-TLM-001 §4.4 (housekeeping.rs `format`), in the order sent
HK1_FIELDS: Dict[str, Tuple[Part, ...]] = {
    "N": (_p("seq"),),
    "UP": (_p("uptime_s"),),
    "BOOT": (_p("boots"),),
    "RST": (_p("reset_cause"),),
    "MODE": (_p("mode"),),
    "HOLD": (_p("commissioning_hold"),),
    "XP": (_p("transponder_enabled"),),
    "TX": (_p("tx_permitted"),),
    "SOC": (_p("soc_pct", 0.1),),
    "VB": (_p("battery_v", 0.001),),
    "IB": (_p("battery_a", 0.001),),
    "PA": (_p("array_w", 0.001),),
    "TB": (_p("battery_temp_c", 0.1),),
    "TO": (_p("obc_temp_c", 0.1),),
    "TP": (_p("pa_temp_c", 0.1),),
    "TJ": (_p("jetson_temp_c", 0.1),),
    "DEP": (_p("deployment_state"), _p("ants_deployed_mask"), _p("wing_sense")),
    "RFS": (_p("rf_silence_state"), _p("rf_silence_s")),
    "CT": (_p("contact_timeout_d"),),
    "CS": (_p("since_contact_h"),),
    "FC": (_p("fault_count"),),
    "FL": (_p("last_fault_type"), _p("last_fault_severity")),
    "W": (_p("rate_x_dps", 0.01), _p("rate_y_dps", 0.01), _p("rate_z_dps", 0.01)),
    "AM": (_p("adcs_mode"),),
    "MR": (_p("mag_rejected"),),
    "GF": (_p("gnss_fix_age_s"), _p("gnss_sats")),
    "GS": (_p("gnss_sessions"), _p("gnss_sessions_with_fix")),
    "PS": (_p("passivation_phase"), _p("eol_state"), _p("eol_fire_attempts"), _p("eol_fire_reasons")),
    "RX": (_p("rx_recovery_cycles"), _p("rx_failed")),
    "CN": (_p("contacts"),),
    "ST": (_p("stored_summary_next"), _p("stored_detail_next")),
    "FR": (_p("fram_state"), _p("fram_failed_checks")),
    "AD": (_p("ants_temp_deferrals"), _p("ants_temp_c")),
}

# HK2, SQSO-TLM-001 §4.4.1 (housekeeping.rs `format2`), in the order sent
HK2_FIELDS: Dict[str, Tuple[Part, ...]] = {
    "N": (_p("seq"),),
    "CA": (_p("commands_executed"), _p("commands_refused"), _p("auth_failures")),
    "WD": (_p("watchdog_resets"),),
    "XE": (_p("transponder_enables"), _p("repeater_keyed_s"), _p("exciter_keyed_s")),
    "IA": (_p("array_a", 0.001),),
    "HL": (_p("hard_limit_violations"),),
    "JB": (_p("jetson_bursts"), _p("last_burst_j"), _p("last_burst_lines"), _p("energy_gate_ends")),
    "JP": (_p("jetson_off_clean"), _p("jetson_off_cut"), _p("jetson_pg_faults"), _p("jetson_readback_faults")),
    "EOL": (_p("eol_state"), _p("eol_fire_attempts")),
    "RT": (_p("rtc_unix_s"), _p("last_reset_unix_s")),
    "CK": (_p("mcu_clock_ppm"),),
    "TR": (_p("trips_gyro"), _p("trips_bmi088"), _p("trips_sun_sensors"), _p("trips_magnetometer")),
    "GY": (_p("gyro_in_use"), _p("mag_samples_gated")),
    "MF": (_p("mtq_bridge_faults"), _p("mtq_off_timer_expiries")),
    "PE": (_p("sram2_parity"),),
    "SEP": (_p("separation_latched"), _p("stowed_readings")),
    "PT": (_p("tx_temp_faulted"), _p("tx_temp_faults")),
    "SV": (_p("eps_supv"), _p("heartbeats_refused")),
    "CI": (_p("chg_inhibit"),),
    "TS": (
        _p("tnc_version"),
        _p("tnc_criteria"),
        _p("tnc_floor_dbm"),
        _p("tnc_floor_cal_dbm"),
        _p("tnc_carrier_count"),
        _p("tnc_frames"),
        _p("tnc_uptime_s"),
    ),
    "RL": (_p("rx_fault_selftest_ok"), _p("tnc_status_unanswered")),
    "SL": (
        _p("selector_enabled"),
        _p("h1_decisions"),
        _p("h1_abstentions"),
        _p("h2_decisions"),
        _p("h2_abstentions"),
        _p("h3_decisions"),
        _p("h3_abstentions"),
    ),
    "GE": (_p("geometry_preset"),),
    "SN": (_p("sun_x", 0.001), _p("sun_y", 0.001), _p("sun_z", 0.001)),
}

HK_FIELDS: Dict[str, Dict[str, Tuple[Part, ...]]] = {"HK1": HK1_FIELDS, "HK2": HK2_FIELDS}


class DecodeError(ValueError):
    """A frame or text that does not follow the downlink format."""


def _int(text: str) -> int:
    if not text or not (text.isdigit() or (text[0] == "-" and text[1:].isdigit())):
        raise DecodeError(f"not an integer: {text!r}")
    return int(text)


def _scaled(raw: Optional[int], part: Part) -> Optional[float]:
    if raw is None:
        return None
    if part.scale == 1.0:
        return raw
    return round(raw * part.scale, part.digits)


def decode_field(key: str, value: str, parts: Sequence[Part]) -> Dict[str, Any]:
    """The named values of one KEY=value field (None for ``-``)."""
    if value == "-":
        return {p.name: None for p in parts}
    numbers = value.split("/")
    if len(numbers) != len(parts):
        raise DecodeError(f"{key}={value}: {len(numbers)} numbers, expected {len(parts)}")
    return {p.name: _scaled(None if n == "-" else _int(n), p) for p, n in zip(parts, numbers)}


def split_pairs(text: str) -> List[Tuple[str, str]]:
    """``KEY=value`` words in order."""
    pairs = []
    for word in text.split():
        key, sep, value = word.partition("=")
        if not sep or not key:
            raise DecodeError(f"not a KEY=value field: {word!r}")
        pairs.append((key, value))
    return pairs


# --- Beacon -----------------------------------------------------------------


@dataclass
class Beacon:
    """A decoded beacon information field (BEACON_SPEC §3)."""

    callsign: str
    text: str
    fields: Dict[str, str]
    values: Dict[str, Any]
    unknown_keys: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "callsign": self.callsign,
            "text": self.text,
            "fields": self.fields,
            "values": self.values,
            "unknown_keys": self.unknown_keys,
        }


def parse_beacon(info: str) -> Beacon:
    """Parse ``de CALL: <text> | T=.. M=.. SOC=.. BV=.. SUN=.. RF=.. QSO=..``.

    Every field of BEACON_SPEC §4 must be present; ``-`` is None (not
    measured, never zero). The text is everything between the call sign and
    the last `` | `` (the content filter keeps ``|`` out of the text).
    """
    if not info.startswith("de "):
        raise DecodeError("not a beacon: no 'de ' prefix")
    callsign, sep, rest = info[3:].partition(": ")
    if not sep:
        raise DecodeError("beacon without 'de CALL: '")
    text, sep, telemetry = rest.rpartition(" | ")
    if not sep:
        raise DecodeError("beacon without the ' | ' telemetry separator")
    fields: Dict[str, str] = {}
    for key, value in split_pairs(telemetry):
        if key in fields:
            raise DecodeError(f"beacon field {key} twice")
        fields[key] = value
    missing = [k for k in BEACON_FIELDS if k not in fields]
    if missing:
        raise DecodeError(f"beacon fields missing: {' '.join(missing)}")
    values: Dict[str, Any] = {}
    for key, part in BEACON_FIELDS.items():
        raw = fields[key]
        if raw == "-":
            values[part.name] = None
        elif key == "M":
            if raw not in POWER_MODES:
                raise DecodeError(f"M={raw}: not one of {'/'.join(POWER_MODES)}")
            values[part.name] = raw
        elif key == "BV":
            whole, dot, tenth = raw.partition(".")
            if not (whole.isdigit() and dot and tenth.isdigit() and len(tenth) == 1):
                raise DecodeError(f"BV={raw}: not volts to one decimal")
            values[part.name] = round(int(whole) + int(tenth) / 10.0, 1)
        else:
            number = _int(raw)
            if number < 0:
                raise DecodeError(f"{key}={raw}: negative")
            values[part.name] = number
    unknown = [k for k in fields if k not in BEACON_FIELDS]
    return Beacon(callsign, text, fields, values, unknown)


# --- Housekeeping -----------------------------------------------------------


@dataclass
class HkPart:
    """One housekeeping frame: its tag (HK1, HK2), sequence and fields."""

    tag: str
    seq: int
    fields: List[Tuple[str, str]]

    @property
    def starts_text(self) -> bool:
        """Whether this is a text's first frame (its first field after N)."""
        first = list(HK_FIELDS[self.tag])[1]
        return bool(self.fields) and self.fields[0][0] == first

    @property
    def ends_text(self) -> bool:
        """Whether this frame carries the text's last field."""
        last = list(HK_FIELDS[self.tag])[-1]
        return any(k == last for k, _ in self.fields)


def parse_hk_frame(info: str) -> HkPart:
    """One HK1/HK2 frame: ``HKn N=<seq> KEY=value ...`` (any part of a text)."""
    tag, _, rest = info.partition(" ")
    if tag not in HK_FIELDS:
        raise DecodeError(f"not a housekeeping frame: {tag!r}")
    pairs = split_pairs(rest)
    if not pairs or pairs[0][0] != "N":
        raise DecodeError(f"{tag} frame without its 'N=' head")
    seq = _int(pairs[0][1])
    return HkPart(tag, seq, pairs[1:])


@dataclass
class Housekeeping:
    """A housekeeping text, merged from its frames."""

    tag: str
    seq: int
    fields: Dict[str, str]
    frames: int
    rx_utc: Optional[str] = None
    values: Dict[str, Any] = field(default_factory=dict)
    missing_keys: List[str] = field(default_factory=list)
    unknown_keys: List[str] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)

    @property
    def complete(self) -> bool:
        return not self.missing_keys and not self.errors

    def to_dict(self) -> Dict[str, Any]:
        return {
            "kind": self.tag.lower(),
            "seq": self.seq,
            "frames": self.frames,
            "complete": self.complete,
            "rx_utc": self.rx_utc,
            "fields": self.fields,
            "values": self.values,
            "missing_keys": self.missing_keys,
            "unknown_keys": self.unknown_keys,
            "errors": self.errors,
        }


def decode_housekeeping(tag: str, seq: int, fields: Dict[str, str], frames: int = 1,
                        rx_utc: Optional[str] = None) -> Housekeeping:
    """The values of a whole HK1/HK2 text's fields."""
    spec = HK_FIELDS[tag]
    hk = Housekeeping(tag, seq, dict(fields), frames, rx_utc)
    hk.fields = {"N": str(seq), **hk.fields}
    for key, parts in spec.items():
        if key not in hk.fields:
            hk.missing_keys.append(key)
            hk.values.update({p.name: None for p in parts})
            continue
        try:
            hk.values.update(decode_field(key, hk.fields[key], parts))
        except DecodeError as exc:
            hk.errors.append(str(exc))
            hk.values.update({p.name: None for p in parts})
    hk.unknown_keys = [k for k in hk.fields if k not in spec]
    return hk


def parse_housekeeping(text: str) -> Housekeeping:
    """A whole HK1/HK2 text on one line (the frames' heads already merged)."""
    part = parse_hk_frame(text)
    return decode_housekeeping(part.tag, part.seq, dict(part.fields))


@dataclass
class _Pending:
    started: float
    rx_utc: Optional[str]
    frames: int = 0
    fields: Dict[str, str] = field(default_factory=dict)
    errors: List[str] = field(default_factory=list)


class HkAssembler:
    """Merges the frames of each housekeeping text by its ``HKn N=`` head.

    A text longer than 250 bytes goes in two or more frames 5 s apart, each
    after the first starting with the first's ``HKn N=<seq> `` (SQSO-TLM-001
    §4.4). A text is given out when the frame with its last field (``AD`` for
    HK1, ``SN`` for HK2) arrives; a text whose frames stop coming is given
    out incomplete by :meth:`expire` or :meth:`flush`, and when a new text
    with the same head begins (the sequence restarts at 0 at every reset).
    """

    def __init__(self, max_age_s: float = 60.0):
        self.max_age_s = max_age_s
        self._pending: Dict[Tuple[str, int], _Pending] = {}

    def add(self, part: HkPart, now: Optional[float] = None, rx_utc: Optional[str] = None) -> List[Housekeeping]:
        """Take one frame; the texts it completes (or pushes out)."""
        now = time.time() if now is None else now
        out = self.expire(now)
        key = (part.tag, part.seq)
        if part.starts_text and key in self._pending:
            out.append(self._finish(key))
        pending = self._pending.setdefault(key, _Pending(now, rx_utc))
        pending.frames += 1
        for k, v in part.fields:
            if k in pending.fields and pending.fields[k] != v:
                pending.errors.append(f"{k} sent twice ({pending.fields[k]}, {v})")
            pending.fields[k] = v
        if part.ends_text:
            out.append(self._finish(key))
        return out

    def expire(self, now: float) -> List[Housekeeping]:
        """Texts whose first frame is older than ``max_age_s``, incomplete."""
        old = [k for k, p in self._pending.items() if now - p.started > self.max_age_s]
        return [self._finish(k) for k in old]

    def flush(self) -> List[Housekeeping]:
        """Every text still waiting for frames, incomplete."""
        return [self._finish(k) for k in list(self._pending)]

    def _finish(self, key: Tuple[str, int]) -> Housekeeping:
        p = self._pending.pop(key)
        hk = decode_housekeeping(key[0], key[1], p.fields, p.frames, p.rx_utc)
        hk.errors.extend(p.errors)
        return hk


# --- Frames -----------------------------------------------------------------


@dataclass
class Frame:
    """One received AX.25 UI frame and what its information field is."""

    kind: str  # beacon, hk1_part, hk2_part, gns, ack, hkd, other
    dest: str
    source: str
    digipeaters: List[str]
    fcs: str  # "ok" (checked here), "absent" (as a KISS TNC delivers it, checked by the TNC)
    info: bytes
    rx_utc: Optional[str] = None
    beacon: Optional[Beacon] = None
    hk: Optional[HkPart] = None
    decoded: Optional[Dict[str, Any]] = None
    errors: List[str] = field(default_factory=list)

    @property
    def from_spacecraft(self) -> bool:
        return self.source == SPACECRAFT

    def to_dict(self) -> Dict[str, Any]:
        out: Dict[str, Any] = {
            "kind": self.kind,
            "rx_utc": self.rx_utc,
            "dest": self.dest,
            "source": self.source,
            "digipeaters": self.digipeaters,
            "fcs": self.fcs,
            "info": self.info.decode("ascii", "replace") if self.kind != "hkd" else self.info.hex().upper(),
            "errors": self.errors,
        }
        if self.beacon is not None:
            out["beacon"] = self.beacon.to_dict()
        if self.hk is not None:
            out["seq"] = self.hk.seq
            out["fields"] = dict(self.hk.fields)
        if self.decoded is not None:
            out["decoded"] = self.decoded
        return out


def parse_ax25(raw: bytes) -> Tuple[str, str, List[str], bytes, str]:
    """``(dest, source, digipeaters, info, fcs)`` of a UI frame.

    ``raw`` is the frame with its flags and FCS (as on the air), from the
    address field through the FCS, or without the FCS (as a KISS TNC delivers
    it). With flags the FCS must check. Without them a frame whose FCS checks
    is taken to carry one, any other to have none.
    """
    flagged = raw[:1] == b"\x7e"
    body = raw.strip(b"\x7e")
    try:
        dest, src, digis, info = sqso_protocol.ax25_parse_ui(body, fcs=True)
        return dest, src, list(digis), bytes(info), "ok"
    except sqso_protocol.FcsError:
        if flagged:
            raise DecodeError("FCS mismatch") from None
    except sqso_protocol.Ax25Error as exc:
        if flagged:
            raise DecodeError(str(exc)) from None
    try:
        dest, src, digis, info = sqso_protocol.ax25_parse_ui(body, fcs=False)
    except sqso_protocol.Ax25Error as exc:
        raise DecodeError(str(exc)) from None
    return dest, src, list(digis), bytes(info), "absent"


def _decode_gns(info: str) -> Optional[Dict[str, Any]]:
    try:
        from gnss_frame import parse_gnss_frame  # same directory
    except ImportError:  # pragma: no cover
        return None
    fix = parse_gnss_frame(info)
    if fix is None:
        return None
    return dataclasses.asdict(fix)


def _decode_ack(info: str) -> Dict[str, Any]:
    pairs = dict(split_pairs(info[len(ACK_PREFIX):]))
    names = {"C": "counter", "R": "result", "N": "days_since_contact"}
    out: Dict[str, Any] = {}
    for key, name in names.items():
        if key not in pairs:
            raise DecodeError(f"ack field {key} missing")
        out[name] = None if pairs[key] == "-" else _int(pairs[key])
    return out


def decode_frame(raw: bytes, rx_utc: Optional[str] = None) -> Frame:
    """Decode one frame (see :func:`parse_ax25` for the forms it takes)."""
    dest, source, digis, info, fcs = parse_ax25(raw)
    frame = Frame("other", dest, source, digis, fcs, info, rx_utc)
    if info.startswith(b"HKD"):
        frame.kind = "hkd"  # binary stored records, SQSO-ICD-005 §5.6: not decoded here
        return frame
    try:
        text = info.decode("ascii")
    except UnicodeDecodeError:
        frame.errors.append("information field is not ASCII")
        return frame
    try:
        if text.startswith("de "):
            frame.kind = "beacon"
            frame.beacon = parse_beacon(text)
            if frame.beacon.callsign != SPACECRAFT:
                frame.errors.append(f"beacon call sign {frame.beacon.callsign}, not {SPACECRAFT}")
        elif text.startswith(("HK1 ", "HK2 ")):
            frame.hk = parse_hk_frame(text)
            frame.kind = frame.hk.tag.lower() + "_part"
        elif text.startswith("GNS "):
            frame.kind = "gns"
            frame.decoded = _decode_gns(text)
            if frame.decoded is None:
                frame.errors.append("GNS frame did not parse")
        elif text.startswith(ACK_PREFIX):
            frame.kind = "ack"
            frame.decoded = _decode_ack(text)
    except DecodeError as exc:
        frame.errors.append(str(exc))
    if source != SPACECRAFT:
        frame.errors.append(f"source {source}, not {SPACECRAFT}")
    return frame


def decode_hex(text: str, rx_utc: Optional[str] = None) -> Frame:
    """Decode a frame given in hex (spaces allowed)."""
    try:
        raw = bytes.fromhex(text.replace(" ", ""))
    except ValueError as exc:
        raise DecodeError(f"not hex: {exc}") from None
    return decode_frame(raw, rx_utc)


# --- The stream of frames ---------------------------------------------------


class Decoder:
    """Frames in, decoded records out, the housekeeping frames merged."""

    def __init__(self, max_age_s: float = 60.0, show_parts: bool = False):
        self.assembler = HkAssembler(max_age_s)
        self.show_parts = show_parts

    def feed(self, raw: bytes, rx_utc: Optional[str] = None, now: Optional[float] = None) -> List[Dict[str, Any]]:
        """The records one frame gives: the frame itself (housekeeping frames
        only with ``show_parts``) and any housekeeping text it completes."""
        try:
            frame = decode_frame(raw, rx_utc)
        except DecodeError as exc:
            return [{"kind": "error", "rx_utc": rx_utc, "frame_hex": raw.hex().upper(), "errors": [str(exc)]}]
        out = []
        if frame.hk is None or frame.errors or self.show_parts:
            out.append(frame.to_dict())
        if frame.hk is not None and frame.from_spacecraft:
            out.extend(hk.to_dict() for hk in self.assembler.add(frame.hk, now, rx_utc))
        return out

    def flush(self) -> List[Dict[str, Any]]:
        return [hk.to_dict() for hk in self.assembler.flush()]


def _utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def read_lines(lines: Iterable[str]) -> Iterator[Tuple[bytes, Optional[str]]]:
    """Frames from logged lines: a JSON object with ``frame_hex`` (and
    ``rx_utc``), or text whose last word is the frame in hex. Blank lines and
    ``#`` comments are skipped; a line that is neither raises DecodeError."""
    for number, line in enumerate(lines, 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("{"):
            record = json.loads(line)
            hexed, rx = record.get("frame_hex"), record.get("rx_utc") or record.get("utc")
        else:
            hexed, rx = line.split()[-1], None
        try:
            yield bytes.fromhex(hexed), rx
        except (TypeError, ValueError):
            raise DecodeError(f"line {number}: no frame in hex") from None


def kiss_frames(stream: IO[bytes], port: int = 0, chunk: int = 512) -> Iterator[bytes]:
    """The data frames of a KISS byte stream, as they complete."""
    kiss = sqso_protocol.KissDecoder(port)
    while True:
        data = stream.read(chunk) if not hasattr(stream, "recv") else stream.recv(chunk)
        if not data:
            return
        yield from kiss.feed(data)


def _emit(records: Iterable[Dict[str, Any]], out: IO[str]) -> int:
    n = 0
    for r in records:
        out.write(json.dumps(r) + "\n")
        out.flush()
        n += 1
    return n


def _log(log: Optional[IO[str]], raw: bytes, rx_utc: str) -> None:
    if log is not None:
        log.write(json.dumps({"rx_utc": rx_utc, "frame_hex": raw.hex().upper()}) + "\n")
        log.flush()


def run_live(stream: Any, decoder: Decoder, out: IO[str], log: Optional[IO[str]], port: int) -> None:
    for raw in kiss_frames(stream, port):
        rx = _utc_now()
        _log(log, raw, rx)
        _emit(decoder.feed(raw, rx), out)
        _emit((hk.to_dict() for hk in decoder.assembler.expire(time.time())), out)


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Decode SMART-QSO downlink frames (beacon, HK1, HK2, GNS, ack) to JSON lines.")
    parser.add_argument("frames", nargs="*", help="frames in hex")
    parser.add_argument("-f", "--file", action="append", default=[],
                        help="a frame log: one frame per line (hex as the last word, or JSON with frame_hex)")
    parser.add_argument("--stdin", action="store_true", help="read a frame log from standard input")
    parser.add_argument("--kiss-tcp", metavar="HOST:PORT", help="decode live from a KISS TCP port (Direwolf: localhost:8001)")
    parser.add_argument("--kiss-file", metavar="PATH",
                        help="decode a KISS byte stream from a file or serial device ('-': standard input)")
    parser.add_argument("--kiss-port", type=int, default=0, help="KISS port of the frames (default 0)")
    parser.add_argument("--log", metavar="FILE", help="with --kiss-*: append every frame received, as JSON lines "
                        "{rx_utc, frame_hex} (a log -f reads back)")
    parser.add_argument("--parts", action="store_true", help="also print each housekeeping frame before merging")
    parser.add_argument("-o", "--output", help="write the records here instead of standard output")
    args = parser.parse_args(argv)

    out = open(args.output, "w", encoding="utf-8") if args.output else sys.stdout
    decoder = Decoder(show_parts=args.parts)
    try:
        if args.kiss_tcp or args.kiss_file:
            log = open(args.log, "a", encoding="utf-8") if args.log else None
            try:
                if args.kiss_tcp:
                    host, _, port = args.kiss_tcp.rpartition(":")
                    with socket.create_connection((host or "localhost", int(port))) as sock:
                        run_live(sock, decoder, out, log, args.kiss_port)
                elif args.kiss_file == "-":
                    run_live(sys.stdin.buffer, decoder, out, log, args.kiss_port)
                else:
                    with open(args.kiss_file, "rb", buffering=0) as stream:
                        run_live(stream, decoder, out, log, args.kiss_port)
            except KeyboardInterrupt:
                pass
            finally:
                if log is not None:
                    log.close()
        else:
            sources: List[Iterable[Tuple[bytes, Optional[str]]]] = []
            for text in args.frames:
                try:
                    sources.append([(bytes.fromhex(text.replace(" ", "")), None)])
                except ValueError:
                    print(f"not hex: {text}", file=sys.stderr)
                    return 2
            for path in args.file:
                with open(path, encoding="utf-8") as fh:
                    sources.append(list(read_lines(fh)))
            if args.stdin:
                sources.append(read_lines(sys.stdin))
            if not sources:
                parser.print_help(sys.stderr)
                return 2
            for source in sources:
                for raw, rx in source:
                    _emit(decoder.feed(raw, rx), out)
        _emit(decoder.flush(), out)
    except DecodeError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    finally:
        if out is not sys.stdout:
            out.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
