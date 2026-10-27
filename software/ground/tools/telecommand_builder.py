#!/usr/bin/env python3
"""
SMART-QSO Authenticated Telecommand Builder

Builds uplink command frames that the flight software accepts
(software/rust/sqso-ffi/src/cmd_handler.rs, software/rust/sqso-fsw/src/keys.rs).

Document ID: SMART-QSO-GND-005
Interface: docs/interfaces/TELECOMMAND_AUTHENTICATION.md

Frame:
    0xAA 0x55 | cmd_id (1) | len (1) | payload (len) | sum16 (2, big-endian)

Payload:
    counter (4, big-endian) | args (N) | tag (16)

    tag = HMAC-SHA-256(key, "SQSO-TC-v1" | cmd_id | N | counter | args)[:16]

The frame is authenticated, not encrypted: 47 CFR 97.211(b) permits
telecommand to a space station to use codes that obscure meaning; all
downlink stays in the clear.

Keys are never stored in the repository. The key file holds 64 hex
characters and must not be readable by group or other. Each counter is
reserved on disk before its frame is produced, so a crash can skip a
counter but never reuse one.

Counters (SQSO-ICD-013 §5.2):
  --counter-from-utc   two or more telecommand stations (§5.2.1, adopted):
                       counter = 4 x (UTC s since 2028-01-01T00:00:00Z) + station index.
                       Refused unless the kernel reports the clock synchronised with a
                       maximum error within --max-clock-error-ms; never below the last
                       counter this station reserved, nor the newest counter heard in an
                       acknowledgement (--newest-ack). The counter file is this station's.
  --counter-file       one station alone (integration, the EM): one shared sequence.

The MAC, the frame and the AX.25/KISS framing are built by sqso_protocol
(software/rust/sqso-py), the Rust code the spacecraft uses to check them.
"""

import argparse
import ctypes
import ctypes.util
import json
import math
import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Optional, Tuple

import sqso_protocol

from command_dictionary import COMMANDS  # generated; see scripts/gen_command_dictionary.py

DOMAIN = sqso_protocol.TC_DOMAIN
KEY_SIZE = sqso_protocol.TC_KEY_SIZE
TAG_SIZE = sqso_protocol.TC_TAG_SIZE
MAX_ARGS = sqso_protocol.TC_MAX_ARGS
COUNTER_LIMIT = sqso_protocol.TC_COUNTER_LIMIT  # UINT32_MAX is never accepted onboard
SYNC = b"\xAA\x55"


_BY_ID = {spec.cmd_id: spec for spec in COMMANDS.values()}


def build_payload(key: bytes, cmd_id: int, counter: int, args: bytes = b"") -> bytes:
    """Return counter | args | tag for one command."""
    if len(key) != KEY_SIZE:
        raise ValueError(f"key must be {KEY_SIZE} bytes")
    if not 0 <= cmd_id <= 0xFF:
        raise ValueError("cmd_id must fit in one byte")
    if not 1 <= counter <= COUNTER_LIMIT:
        raise ValueError(f"counter must be 1..{COUNTER_LIMIT:#x}")
    if len(args) > MAX_ARGS:
        raise ValueError(f"args must be at most {MAX_ARGS} bytes")
    return sqso_protocol.tc_payload(bytes(key), cmd_id, counter, bytes(args))


def build_frame(key: bytes, cmd_id: int, counter: int, args: bytes = b"",
                strict: bool = True) -> bytes:
    """Return the complete uplink frame.

    With strict (the default) the argument length must match the command
    dictionary, as the flight software requires.
    """
    if strict:
        spec = _BY_ID.get(cmd_id)
        if spec is None:
            raise ValueError(f"unknown command ID {cmd_id:#04x}")
        if not spec.min_args <= len(args) <= spec.max_args:
            raise ValueError(
                f"command {cmd_id:#04x} takes {spec.min_args}..{spec.max_args} argument bytes")

    build_payload(key, cmd_id, counter, args)  # range checks with Python errors
    return sqso_protocol.tc_frame(bytes(key), cmd_id, counter, bytes(args))


def ax25_fcs(data: bytes) -> int:
    """AX.25 frame check sequence (CRC-16/X.25)."""
    return sqso_protocol.ax25_fcs(bytes(data))


def ax25_ui_frame(dest: str, src: str, info: bytes, fcs: bool = True) -> bytes:
    """AX.25 UI frame (PID 0xF0) from the address field through the FCS.

    Pass fcs=False for a KISS TNC, which appends the FCS itself.
    """
    return sqso_protocol.ax25_ui_frame(dest, src, bytes(info), fcs)


def kiss_frame(frame: bytes, port: int = 0) -> bytes:
    """Wrap a frame for a KISS TNC (data frame on the given port)."""
    return sqso_protocol.kiss_frame(bytes(frame), port & 0x0F)


def load_key(path: Path) -> bytes:
    """Read a 32-byte key stored as 64 hex characters."""
    if os.name == "posix" and (path.stat().st_mode & 0o077):
        raise PermissionError(f"{path} must not be readable by group or other (chmod 600)")
    text = "".join(path.read_text().split())
    try:
        key = bytes.fromhex(text)
    except ValueError as exc:
        raise ValueError(f"{path} is not hex") from exc
    if len(key) != KEY_SIZE:
        raise ValueError(f"{path} must hold {KEY_SIZE * 2} hex characters")
    return key


def next_counter(path: Path) -> int:
    """Reserve and return the next counter, persisting it first."""
    current = int(path.read_text().strip()) if path.exists() else 0
    counter = current + 1
    if counter > COUNTER_LIMIT:
        raise ValueError("counter space exhausted: provision a new key")

    _reserve(path, counter)
    return counter


# --- Counters from UTC (SQSO-ICD-013 §5.2.1) ----------------------------------

#: 2028-01-01T00:00:00Z
UTC_COUNTER_EPOCH = 1_830_297_600
#: Station indices 0-3: counter = 4 x seconds + index
STATION_SLOTS = 4
#: Largest clock error accepted, s: two stations then disagree by at most 1 s
MAX_CLOCK_ERROR_S = 0.5
#: The builder waits at most this long for its clock to pass a counter already used
MAX_WAIT_S = 2.0


class ClockError(Exception):
    """The station clock cannot be trusted for UTC-derived counters."""


@dataclass(frozen=True)
class ClockStatus:
    """What the operating system knows of the clock's synchronisation."""

    synchronised: bool
    max_error_s: Optional[float]
    source: str


class _Timeval(ctypes.Structure):
    _fields_ = [("tv_sec", ctypes.c_long), ("tv_usec", ctypes.c_long)]


class _Timex(ctypes.Structure):
    """struct timex (Linux, adjtimex(2))."""
    _fields_ = [("modes", ctypes.c_uint), ("offset", ctypes.c_long), ("freq", ctypes.c_long),
                ("maxerror", ctypes.c_long), ("esterror", ctypes.c_long), ("status", ctypes.c_int),
                ("constant", ctypes.c_long), ("precision", ctypes.c_long), ("tolerance", ctypes.c_long),
                ("time", _Timeval), ("tick", ctypes.c_long), ("ppsfreq", ctypes.c_long), ("jitter", ctypes.c_long),
                ("shift", ctypes.c_int), ("stabil", ctypes.c_long), ("jitcnt", ctypes.c_long),
                ("calcnt", ctypes.c_long), ("errcnt", ctypes.c_long), ("stbcnt", ctypes.c_long),
                ("tai", ctypes.c_int), ("_reserved", ctypes.c_int * 11)]


_TIME_ERROR = 5       # adjtimex(2) return value: clock not synchronised
_STA_UNSYNC = 0x0040


def kernel_clock_status() -> ClockStatus:
    """The kernel's synchronisation state and maximum error (read-only adjtimex).

    chrony, ntpd and systemd-timesyncd (a GNSS-disciplined or NTP source)
    clear STA_UNSYNC and keep maxerror; without them the kernel reports
    TIME_ERROR. Anywhere adjtimex is not available, the clock counts as not
    synchronised.
    """
    try:
        libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)
        tx = _Timex()
        state = libc.adjtimex(ctypes.byref(tx))
    except (OSError, AttributeError) as exc:
        return ClockStatus(False, None, f"adjtimex unavailable ({exc})")
    if state < 0:
        return ClockStatus(False, None, f"adjtimex failed (errno {ctypes.get_errno()})")
    synced = state != _TIME_ERROR and not tx.status & _STA_UNSYNC
    return ClockStatus(synced, tx.maxerror / 1e6, "kernel (adjtimex)")


def utc_counter(unix_s: float, station_index: int, epoch: int = UTC_COUNTER_EPOCH) -> int:
    """4 x (whole UTC seconds since the epoch) + station index."""
    if not 0 <= station_index < STATION_SLOTS:
        raise ValueError(f"station index must be 0..{STATION_SLOTS - 1}")
    seconds = math.floor(unix_s) - epoch
    if seconds < 0:
        raise ValueError("UTC counters start at 2028-01-01T00:00:00Z")
    counter = STATION_SLOTS * seconds + station_index
    if counter > COUNTER_LIMIT:
        raise ValueError("counter space exhausted")
    return counter


def decode_utc_counter(counter: int, epoch: int = UTC_COUNTER_EPOCH) -> Tuple[int, int]:
    """(UTC second as a POSIX time, station index) of a UTC-derived counter."""
    return epoch + counter // STATION_SLOTS, counter % STATION_SLOTS


def check_clock(status: ClockStatus, max_error_s: float = MAX_CLOCK_ERROR_S) -> None:
    """Raise ClockError unless the clock is synchronised within max_error_s."""
    if not status.synchronised:
        raise ClockError(f"station clock not synchronised ({status.source}): check the GNSS or NTP time source; "
                         "no counter reserved")
    if status.max_error_s is None or status.max_error_s > max_error_s:
        shown = "unknown" if status.max_error_s is None else f"{status.max_error_s * 1000:.0f} ms"
        raise ClockError(f"station clock maximum error {shown} exceeds {max_error_s * 1000:.0f} ms; "
                         "no counter reserved")


def _read_counter(path: Path) -> int:
    return int(path.read_text().strip()) if path.exists() else 0


def _reserve(path: Path, counter: int) -> None:
    tmp = path.with_name(path.name + ".tmp")
    with open(tmp, "w", encoding="ascii") as handle:
        handle.write(f"{counter}\n")
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)


def next_utc_counter(path: Path, station_index: int, *, newest_heard: Optional[int] = None,
                     clock: Optional[Callable[[], ClockStatus]] = None,
                     now: Optional[Callable[[], float]] = None, sleep: Optional[Callable[[float], None]] = None,
                     max_clock_error_s: float = MAX_CLOCK_ERROR_S, epoch: int = UTC_COUNTER_EPOCH) -> int:
    """Reserve and return this station's next UTC-derived counter (§5.2.1).

    path is this station's counter file for the key (the last counter it
    reserved). newest_heard is the newest counter seen in an acknowledgement
    from the spacecraft (`ack ... C=`), sent by any station. Guarantees:
    - strictly above the last counter this station reserved, and above
      newest_heard (the onboard rule: strictly above the last accepted);
    - the builder waits up to MAX_WAIT_S for its clock to pass either, and
      refuses beyond that (its clock stepped back, or another station's
      clock was fast), naming the size of the step;
    - refused unless the clock is synchronised within max_clock_error_s;
    - written and fsynced before it is returned (§5.2 rule 2).
    """
    clock = clock or kernel_clock_status
    now = now or time.time
    sleep = sleep or time.sleep
    check_clock(clock(), max_clock_error_s)
    last = _read_counter(path)
    if last and last % STATION_SLOTS != station_index:
        raise ValueError(f"{path} holds a counter of station {last % STATION_SLOTS}, not station {station_index}: "
                         "each station keeps its own counter file")
    floor = max(last, newest_heard or 0)
    deadline = now() + MAX_WAIT_S
    while True:
        candidate = utc_counter(now(), station_index, epoch)
        if candidate > floor:
            break
        behind = (floor - candidate) / STATION_SLOTS
        if now() + behind > deadline:
            who = ("this station's last counter" if floor == last else
                   f"the newest acknowledged counter (station {floor % STATION_SLOTS})")
            raise ClockError(f"the clock is {behind:.0f} s behind {who}: a clock stepped back or ran fast. "
                             "Check both stations' clocks against GNSS; commands built now would be rejected "
                             "onboard as replays. No counter reserved")
        sleep(max(behind, 0.0) + 1.0 - (now() % 1.0) + 0.001)
    _reserve(path, candidate)
    return candidate


def load_station_index(path: Path) -> int:
    """station_index from a station configuration file ({"station": ..., "station_index": N})."""
    data = json.loads(Path(path).read_text())
    index = data["station_index"]
    if not isinstance(index, int) or not 0 <= index < STATION_SLOTS:
        raise ValueError(f"{path}: station_index must be an integer 0..{STATION_SLOTS - 1}")
    return index


def _parse_args(argv):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("command", nargs="?", help="Command name (see --list)")
    parser.add_argument("--args", default="", help="Argument bytes as hex")
    parser.add_argument("--key-file", type=Path, help="Key file (64 hex characters, mode 600)")
    parser.add_argument("--counter-file", type=Path,
                        help="Counter state file (reserved before use); with --counter-from-utc, this station's")
    parser.add_argument("--counter", type=int, help="Explicit counter (testing only)")
    parser.add_argument("--counter-from-utc", action="store_true",
                        help="UTC-derived counter (SQSO-ICD-013 §5.2.1; two or more stations)")
    parser.add_argument("--station-index", type=int, help="This station's index 0-3 (SQSO-GND-001 §8)")
    parser.add_argument("--station-config", type=Path, help='JSON file with "station_index"')
    parser.add_argument("--newest-ack", type=int,
                        help="Newest counter heard in an acknowledgement (ack ... C=), from any station")
    parser.add_argument("--max-clock-error-ms", type=float, default=MAX_CLOCK_ERROR_S * 1000)
    parser.add_argument("--test-counter-epoch", type=int,
                        help="EM and integration tests with test keys only: counter epoch (POSIX s) before 2028")
    parser.add_argument("--ax25", nargs=2, metavar=("DEST", "SRC"),
                        help="Wrap in an AX.25 UI frame, e.g. --ax25 KK6GDB-11 N0CALL (spacecraft, then your station)")
    parser.add_argument("--kiss", action="store_true", help="With --ax25: KISS-encode for a TNC (TNC adds the FCS)")
    parser.add_argument("--out", type=Path, help="Write the binary frame here instead of printing hex")
    parser.add_argument("--list", action="store_true", help="List the command dictionary")
    return parser.parse_args(argv)


def main(argv=None) -> int:
    """Command-line entry point."""
    opts = _parse_args(argv)

    if opts.list:
        for name, spec in COMMANDS.items():
            print(f"{spec.cmd_id:#04x}  {name:<20} args {spec.min_args}..{spec.max_args}  {spec.summary}")
        return 0

    if opts.command is None or opts.key_file is None:
        print("command and --key-file are required", file=sys.stderr)
        return 2
    if opts.command not in COMMANDS:
        print(f"unknown command {opts.command}", file=sys.stderr)
        return 2
    if opts.counter is not None and (opts.counter_file is not None or opts.counter_from_utc):
        print("--counter excludes --counter-file and --counter-from-utc", file=sys.stderr)
        return 2
    if opts.counter is None and opts.counter_file is None:
        print("--counter-file (or --counter) is required", file=sys.stderr)
        return 2
    station_index = None
    if opts.counter_from_utc:
        try:
            from_config = load_station_index(opts.station_config) if opts.station_config else None
        except (OSError, ValueError, KeyError) as exc:
            print(f"error: station configuration: {exc}", file=sys.stderr)
            return 2
        if opts.station_index is not None and from_config is not None and opts.station_index != from_config:
            print("--station-index disagrees with --station-config", file=sys.stderr)
            return 2
        station_index = opts.station_index if opts.station_index is not None else from_config
        if station_index is None:
            print("--counter-from-utc needs --station-index or --station-config", file=sys.stderr)
            return 2

    try:
        key = load_key(opts.key_file)
        args = bytes.fromhex(opts.args)
        spec = COMMANDS[opts.command]
        # Validate before reserving a counter so a typo does not consume one
        build_frame(key, spec.cmd_id, 1, args)
        if opts.counter is not None:
            counter = opts.counter
        elif opts.counter_from_utc:
            epoch = UTC_COUNTER_EPOCH
            if opts.test_counter_epoch is not None:
                print("warning: test counter epoch: EM and integration test keys only", file=sys.stderr)
                epoch = opts.test_counter_epoch
            counter = next_utc_counter(opts.counter_file, station_index, newest_heard=opts.newest_ack,
                                       max_clock_error_s=opts.max_clock_error_ms / 1000.0, epoch=epoch)
            print(f"counter {counter} (station {station_index})", file=sys.stderr)
        else:
            counter = next_counter(opts.counter_file)
        frame = build_frame(key, spec.cmd_id, counter, args)
        if opts.ax25 is not None:
            frame = ax25_ui_frame(opts.ax25[0], opts.ax25[1], frame, fcs=not opts.kiss)
            if opts.kiss:
                frame = kiss_frame(frame)
        elif opts.kiss:
            raise ValueError("--kiss needs --ax25")
    except (OSError, ValueError, ClockError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if opts.out is not None:
        opts.out.write_bytes(frame)
    else:
        print(frame.hex().upper())
    return 0


if __name__ == "__main__":
    sys.exit(main())
