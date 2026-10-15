#!/usr/bin/env python3
"""
SMART-QSO Telecommand Key Provisioning (GSE, integration only)

Generates a key set and builds the frames that load it into the spacecraft
over the GSE umbilical (software/flight/src/gse_provision.c). The flight
software accepts these frames only while the umbilical-sense line is high,
so nothing here can be sent over RF.

Document ID: SMART-QSO-GND-006
Interface: docs/interfaces/TELECOMMAND_AUTHENTICATION.md section 9

Key set (one per spacecraft):
    op0.key .. op3.key  operational keys (slot 0 is activated; 1-3 are spares)
    recovery.key        recovery key: keep offline; it authenticates only
                        KEY_ROTATE and TX_DISABLE

Frame:
    "SQGP" | op (1) | slot (1) | key_id (1) | key (32) | CRC-32 (4, BE, zlib)

Key files and frame files contain secrets: they are written mode 600 and must
never be committed (scripts/check_secrets.py enforces this in CI).
"""

import argparse
import json
import os
import secrets
import struct
import sys
import zlib
from pathlib import Path
from typing import Dict, List, Sequence

from telecommand_builder import KEY_SIZE, load_key

MAGIC = b"SQGP"
FRAME_SIZE = 43
OP_SLOTS = 4
RECOVERY_SLOT = OP_SLOTS

OP_PROVISION = 0x01
OP_ACTIVATE = 0x02
OP_ZEROIZE = 0x03
OP_ARM_FLIGHT = 0x04


def build_gse_frame(op: int, slot: int, key_id: int, key: bytes = b"") -> bytes:
    """Return one provisioning frame."""
    if not 0 <= slot <= RECOVERY_SLOT:
        raise ValueError(f"slot must be 0..{RECOVERY_SLOT}")
    if op == OP_PROVISION and len(key) != KEY_SIZE:
        raise ValueError(f"PROVISION needs a {KEY_SIZE}-byte key")
    body = MAGIC + bytes([op, slot, key_id]) + key.ljust(KEY_SIZE, b"\x00")
    return body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)


def _write_private(path: Path, data: bytes) -> None:
    """Create a file readable only by its owner; refuse to overwrite."""
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "wb") as handle:
        handle.write(data)


def generate_key_set(directory: Path, op_key_ids: Sequence[int], recovery_key_id: int) -> Dict:
    """Create fresh random keys in a new directory and return the manifest."""
    if len(op_key_ids) != OP_SLOTS:
        raise ValueError(f"need {OP_SLOTS} operational key IDs")
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    entries = [{"slot": slot, "key_id": key_id, "file": f"op{slot}.key"}
               for slot, key_id in enumerate(op_key_ids)]
    entries.append({"slot": RECOVERY_SLOT, "key_id": recovery_key_id, "file": "recovery.key"})

    existing = [e["file"] for e in entries if (directory / e["file"]).exists()]
    if existing:
        raise FileExistsError(f"refusing to overwrite {', '.join(existing)} in {directory}")
    for entry in entries:
        _write_private(directory / entry["file"], (secrets.token_bytes(KEY_SIZE).hex() + "\n").encode())

    manifest = {"version": 1, "keys": entries,
                "note": "Key IDs only. Key material stays in the .key files."}
    (directory / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


def provisioning_frames(directory: Path, activate_slot: int = 0) -> List[bytes]:
    """Frames that load every key in a set, then activate one operational key."""
    manifest = json.loads((directory / "manifest.json").read_text())
    frames = [build_gse_frame(OP_PROVISION, e["slot"], e["key_id"], load_key(directory / e["file"]))
              for e in manifest["keys"]]
    frames.append(build_gse_frame(OP_ACTIVATE, activate_slot, 0))
    return frames


def write_frames(path: Path, frames: Sequence[bytes]) -> None:
    """Write frames back to back into a new private file."""
    _write_private(path, b"".join(frames))


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="action", required=True)

    gen = sub.add_parser("generate", help="create a new key set")
    gen.add_argument("--dir", type=Path, required=True)
    gen.add_argument("--op-key-ids", default="1,2,3,4", help="four operational key IDs")
    gen.add_argument("--recovery-key-id", type=int, default=100)

    frm = sub.add_parser("frames", help="build provisioning frames for a key set")
    frm.add_argument("--dir", type=Path, required=True)
    frm.add_argument("--activate", type=int, default=0, help="operational slot to activate")
    frm.add_argument("--out", type=Path, required=True)

    zer = sub.add_parser("zeroize", help="build a frame that erases the key table")
    zer.add_argument("--out", type=Path, required=True)

    arm = sub.add_parser("arm-flight",
                         help="build a frame that re-arms the 45-minute RF silence and the deployment "
                              "sequence (last powered action before shipping)")
    arm.add_argument("--out", type=Path, required=True)

    opts = parser.parse_args(argv)
    try:
        if opts.action == "generate":
            ids = [int(x) for x in opts.op_key_ids.split(",")]
            generate_key_set(opts.dir, ids, opts.recovery_key_id)
            print(f"key set written to {opts.dir} (mode 600); keep recovery.key offline")
        elif opts.action == "frames":
            frames = provisioning_frames(opts.dir, opts.activate)
            write_frames(opts.out, frames)
            print(f"{len(frames)} frames written to {opts.out}; send over the umbilical UART, then delete")
        elif opts.action == "zeroize":
            write_frames(opts.out, [build_gse_frame(OP_ZEROIZE, 0, 0)])
            print(f"zeroize frame written to {opts.out}")
        else:
            write_frames(opts.out, [build_gse_frame(OP_ARM_FLIGHT, 0, 0)])
            print(f"arm-flight frame written to {opts.out}; send it last, check the reply, then power off")
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
