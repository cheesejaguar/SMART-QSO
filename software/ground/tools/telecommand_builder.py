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

The MAC, the frame and the AX.25/KISS framing are built by sqso_protocol
(software/rust/sqso-py), the Rust code the spacecraft uses to check them.
"""

import argparse
import os
import sys
from pathlib import Path

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
