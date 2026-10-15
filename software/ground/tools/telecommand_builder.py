#!/usr/bin/env python3
"""
SMART-QSO Authenticated Telecommand Builder

Builds uplink command frames that the flight software accepts
(software/flight/src/cmd_handler.c, cmd_auth.c).

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
"""

import argparse
import hashlib
import hmac
import os
import struct
import sys
from pathlib import Path

from command_dictionary import COMMANDS  # generated; see scripts/gen_command_dictionary.py

DOMAIN = b"SQSO-TC-v1"
KEY_SIZE = 32
TAG_SIZE = 16
MAX_ARGS = 64
COUNTER_LIMIT = 0xFFFFFFFE  # UINT32_MAX is never accepted onboard
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

    counter_bytes = struct.pack(">I", counter)
    mac_input = DOMAIN + bytes([cmd_id, len(args)]) + counter_bytes + args
    tag = hmac.new(key, mac_input, hashlib.sha256).digest()[:TAG_SIZE]
    return counter_bytes + args + tag


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

    payload = build_payload(key, cmd_id, counter, args)
    body = SYNC + bytes([cmd_id, len(payload)]) + payload
    return body + struct.pack(">H", sum(body) & 0xFFFF)


def _parse_callsign(text: str):
    """Split "CALL-SSID" (SSID optional) and validate it for AX.25."""
    call, _, ssid_text = text.upper().partition("-")
    ssid = int(ssid_text) if ssid_text else 0
    if not (1 <= len(call) <= 6 and call.isalnum() and call.isascii()) or not 0 <= ssid <= 15:
        raise ValueError(f"invalid AX.25 call sign {text!r}")
    return call, ssid


def _ax25_address(text: str, last: bool) -> bytes:
    call, ssid = _parse_callsign(text)
    return bytes(ord(c) << 1 for c in call.ljust(6)) + bytes([0x60 | (ssid << 1) | (1 if last else 0)])


def ax25_fcs(data: bytes) -> int:
    """AX.25 frame check sequence (CRC-16/X.25)."""
    crc = 0xFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ 0x8408 if crc & 1 else crc >> 1
    return crc ^ 0xFFFF


def ax25_ui_frame(dest: str, src: str, info: bytes, fcs: bool = True) -> bytes:
    """AX.25 UI frame (PID 0xF0) from the address field through the FCS.

    Pass fcs=False for a KISS TNC, which appends the FCS itself.
    """
    body = _ax25_address(dest, False) + _ax25_address(src, True) + b"\x03\xF0" + info
    return body + struct.pack("<H", ax25_fcs(body)) if fcs else body


def kiss_frame(frame: bytes, port: int = 0) -> bytes:
    """Wrap a frame for a KISS TNC (data frame on the given port)."""
    escaped = frame.replace(b"\xDB", b"\xDB\xDD").replace(b"\xC0", b"\xDB\xDC")
    return b"\xC0" + bytes([(port & 0x0F) << 4]) + escaped + b"\xC0"


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

    tmp = path.with_name(path.name + ".tmp")
    with open(tmp, "w", encoding="ascii") as handle:
        handle.write(f"{counter}\n")
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, path)
    return counter


def _parse_args(argv):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("command", nargs="?", help="Command name (see --list)")
    parser.add_argument("--args", default="", help="Argument bytes as hex")
    parser.add_argument("--key-file", type=Path, help="Key file (64 hex characters, mode 600)")
    counter = parser.add_mutually_exclusive_group()
    counter.add_argument("--counter-file", type=Path, help="Counter state file (reserved before use)")
    counter.add_argument("--counter", type=int, help="Explicit counter (testing only)")
    parser.add_argument("--ax25", nargs=2, metavar=("DEST", "SRC"),
                        help="Wrap in an AX.25 UI frame, e.g. --ax25 SQSO-1 N0CALL (call sign is decision D1)")
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
    if opts.counter is None and opts.counter_file is None:
        print("--counter-file (or --counter) is required", file=sys.stderr)
        return 2

    try:
        key = load_key(opts.key_file)
        args = bytes.fromhex(opts.args)
        spec = COMMANDS[opts.command]
        # Validate before reserving a counter so a typo does not consume one
        build_frame(key, spec.cmd_id, 1, args)
        counter = opts.counter if opts.counter is not None else next_counter(opts.counter_file)
        frame = build_frame(key, spec.cmd_id, counter, args)
        if opts.ax25 is not None:
            frame = ax25_ui_frame(opts.ax25[0], opts.ax25[1], frame, fcs=not opts.kiss)
            if opts.kiss:
                frame = kiss_frame(frame)
        elif opts.kiss:
            raise ValueError("--kiss needs --ax25")
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    if opts.out is not None:
        opts.out.write_bytes(frame)
    else:
        print(frame.hex().upper())
    return 0


if __name__ == "__main__":
    sys.exit(main())
