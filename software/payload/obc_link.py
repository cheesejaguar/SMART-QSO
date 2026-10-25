"""
Jetson side of the OBC-Jetson line protocol.

Every message is one ASCII line (docs/interfaces/ICD_OBC_JETSON.md §4):

    <TYPE>,<fields...>,<CRC>\\n

Printable ASCII (0x20-0x7E) only. CRC is four upper-case hex digits of
CRC-16/CCITT-FALSE over every byte before the last comma. A BEACON_RSP text
may contain commas: it runs from after the sequence number to the last comma.

OBC -> Jetson: TELEMETRY, BEACON_REQ, STATUS_REQ, SHUTDOWN
Jetson -> OBC: BEACON_RSP, STATUS, ACK, NACK

Framing, CRC, field parsing and line building are the Rust implementation
the OBC runs (software/rust/sqso-protocol/src/jetson.rs), through the
sqso_protocol module; both test suites check the same golden lines.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List

import sqso_protocol

LINE_MAX = sqso_protocol.LINE_MAX   # bytes, including the CRC field and the newline
TEXT_MAX = sqso_protocol.TEXT_MAX   # longest beacon text the OBC accepts

#: A line that does not follow the ICD (a ValueError)
LinkError = sqso_protocol.LinkError

#: A well-formed line whose CRC does not match (a LinkError)
CrcError = sqso_protocol.CrcError


@dataclass
class Message:
    """A checked line: its type and the fields after it."""
    type: str
    fields: List[str] = field(default_factory=list)

    def line(self) -> bytes:
        """The message as a sealed line again."""
        return seal(",".join([self.type, *self.fields]))


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
    try:
        data = body.encode("ascii")
    except UnicodeEncodeError as exc:
        raise LinkError("line must be printable ASCII") from exc
    return sqso_protocol.jetson_seal(data)


def parse(line: bytes) -> Message:
    """Check one received line and split it into type and fields."""
    text = sqso_protocol.jetson_check(bytes(line)).decode("ascii")
    msg_type, _, rest = text.partition(",")
    if msg_type == "BEACON_RSP":
        seq, _, beacon_text = rest.partition(",")
        return Message(msg_type, [seq, beacon_text])
    return Message(msg_type, rest.split(",") if rest else [])


def parse_telemetry(msg: Message) -> Telemetry:
    """Interpret a TELEMETRY message."""
    if msg.type != "TELEMETRY":
        raise LinkError("not a TELEMETRY message")
    _, time_ms, sunlit, soc, mode, limit, extras = sqso_protocol.jetson_parse_from_obc(msg.line())
    return Telemetry(time_ms, sunlit, soc, mode, limit, extras)


def parse_beacon_req(msg: Message) -> BeaconRequest:
    """Interpret a BEACON_REQ message."""
    if msg.type != "BEACON_REQ":
        raise LinkError("not a BEACON_REQ message")
    _, seq, max_len = sqso_protocol.jetson_parse_from_obc(msg.line())
    return BeaconRequest(seq, max_len)


def beacon_rsp(seq: int, text: str) -> bytes:
    """BEACON_RSP,<seq>,<text> (text: 1..TEXT_MAX printable ASCII characters)."""
    return sqso_protocol.jetson_beacon_rsp(seq, text)


def status(health: int, temp_c: int, power_mw: int, mem_pct: int, uptime_s: int) -> bytes:
    """STATUS,<health 0-3>,<temp_c>,<power_mw>,<mem_pct>,<uptime_s>"""
    return sqso_protocol.jetson_status(health, temp_c, power_mw, mem_pct, uptime_s)


def ack(what: str) -> bytes:
    """ACK,<message type acknowledged>"""
    return sqso_protocol.jetson_ack(what)


def nack(reason: str) -> bytes:
    """NACK,<reason>"""
    return sqso_protocol.jetson_nack(reason)
