"""
Jetson side of the OBC-Jetson line protocol.

Every message is one ASCII line (docs/interfaces/ICD_OBC_JETSON.md §4):

    <TYPE>,<fields...>,<CRC>\\n

Printable ASCII (0x20-0x7E) only. CRC is four upper-case hex digits of
CRC-16/CCITT-FALSE over every byte before the last comma. A BEACON_RSP text
may contain commas: it runs from after the sequence number to the last comma.

OBC -> Jetson: TELEMETRY, BEACON_REQ, STATUS_REQ, SHUTDOWN
Jetson -> OBC: BEACON_RSP, STATUS, ACK, NACK

The OBC implementation is software/flight/src/jetson_link.c; both test suites
check the same golden lines.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Dict, List

try:
    from crc16 import crc16_ccitt
except ImportError:  # running as software.payload.* from the repository root
    from software.payload.crc16 import crc16_ccitt

LINE_MAX = 512          # bytes, including the CRC field and the newline
TEXT_MAX = 176          # longest beacon text the OBC accepts
_PRINTABLE = re.compile(rb"^[\x20-\x7e]*$")
_CRC_FIELD = re.compile(rb"^[0-9A-F]{4}$")


class LinkError(ValueError):
    """A line that does not follow the ICD."""


class CrcError(LinkError):
    """A well-formed line whose CRC does not match."""


@dataclass
class Message:
    """A checked line: its type and the fields after it."""
    type: str
    fields: List[str] = field(default_factory=list)


@dataclass
class Telemetry:
    """TELEMETRY,<time_ms>,<SUNLIT|ECLIPSE>,<soc>,<mode>,<limit_w>,<key=value>..."""
    time_ms: int
    sunlit: bool
    soc: float
    power_mode: int
    power_limit_w: float
    extras: Dict[str, str]


@dataclass
class BeaconRequest:
    """BEACON_REQ,<seq>,<max_len>"""
    seq: int
    max_len: int


def seal(body: str) -> bytes:
    """Append ",<CRC>\\n" to a message body."""
    data = body.encode("ascii")
    if not data or not _PRINTABLE.match(data):
        raise LinkError("line must be non-empty printable ASCII")
    line = data + b"," + f"{crc16_ccitt(data):04X}".encode("ascii") + b"\n"
    if len(line) > LINE_MAX:
        raise LinkError(f"line longer than {LINE_MAX} bytes")
    return line


def parse(line: bytes) -> Message:
    """Check one received line and split it into type and fields."""
    if line.endswith(b"\n"):
        line = line[:-1]
    if not line or len(line) >= LINE_MAX or not _PRINTABLE.match(line):
        raise LinkError("not a printable line of allowed length")
    body, sep, crc = line.rpartition(b",")
    if not sep or not body or not _CRC_FIELD.match(crc):
        raise LinkError("missing or malformed CRC field")
    if crc16_ccitt(body) != int(crc, 16):
        raise CrcError("CRC mismatch")

    text = body.decode("ascii")
    msg_type, _, rest = text.partition(",")
    if msg_type == "BEACON_RSP":
        seq, _, beacon_text = rest.partition(",")
        return Message(msg_type, [seq, beacon_text])
    return Message(msg_type, rest.split(",") if rest else [])


def _uint(text: str, maximum: int) -> int:
    if not text.isdigit() or int(text) > maximum:
        raise LinkError(f"field {text!r} out of range 0..{maximum}")
    return int(text)


def parse_telemetry(msg: Message) -> Telemetry:
    """Interpret a TELEMETRY message."""
    if msg.type != "TELEMETRY" or len(msg.fields) < 5:
        raise LinkError("not a TELEMETRY message")
    time_ms, sun, soc, mode, limit = msg.fields[:5]
    if sun not in ("SUNLIT", "ECLIPSE"):
        raise LinkError("sunlit field must be SUNLIT or ECLIPSE")
    extras = {}
    for item in msg.fields[5:]:
        key, eq, value = item.partition("=")
        if eq and key:
            extras[key] = value
    try:
        return Telemetry(_uint(time_ms, 2**64 - 1), sun == "SUNLIT", float(soc),
                         _uint(mode, 2), float(limit), extras)
    except ValueError as exc:
        raise LinkError(str(exc)) from exc


def parse_beacon_req(msg: Message) -> BeaconRequest:
    """Interpret a BEACON_REQ message."""
    if msg.type != "BEACON_REQ" or len(msg.fields) != 2:
        raise LinkError("not a BEACON_REQ message")
    seq = _uint(msg.fields[0], 0xFFFF)
    max_len = _uint(msg.fields[1], TEXT_MAX)
    if max_len == 0:
        raise LinkError("max_len must be at least 1")
    return BeaconRequest(seq, max_len)


def beacon_rsp(seq: int, text: str) -> bytes:
    """BEACON_RSP,<seq>,<text> (text: 1..TEXT_MAX printable ASCII characters)."""
    if not 0 <= seq <= 0xFFFF:
        raise LinkError("seq out of range")
    if not 1 <= len(text) <= TEXT_MAX or not _PRINTABLE.match(text.encode("ascii", "replace")):
        raise LinkError(f"beacon text must be 1..{TEXT_MAX} printable ASCII characters")
    return seal(f"BEACON_RSP,{seq},{text}")


def status(health: int, temp_c: int, power_mw: int, mem_pct: int, uptime_s: int) -> bytes:
    """STATUS,<health 0-3>,<temp_c>,<power_mw>,<mem_pct>,<uptime_s>"""
    if not (0 <= health <= 3 and -128 <= temp_c <= 200 and 0 <= power_mw <= 100000
            and 0 <= mem_pct <= 100 and 0 <= uptime_s <= 0xFFFFFFFF):
        raise LinkError("STATUS field out of range")
    return seal(f"STATUS,{health},{temp_c},{power_mw},{mem_pct},{uptime_s}")


def ack(what: str) -> bytes:
    """ACK,<message type acknowledged>"""
    return seal(f"ACK,{what}")


def nack(reason: str) -> bytes:
    """NACK,<reason>"""
    return seal(f"NACK,{reason}")
