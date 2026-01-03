#!/usr/bin/env python3
"""
SMART-QSO Federated Learning Update Frame Builder

Builds FL-UPDATE v0.2 frames for model updates via amateur radio uplink.
Follows amateur radio regulations (no encryption, in-the-clear).

Document ID: SMART-QSO-GND-002
Version: 1.0

Frame Format (FL-UPDATE v0.2):
+--------+--------+--------+--------+--------+--------+--------+
| Magic  | Ver    | SeqNum | TotalF | ChkSum | PayLen | Payload...
| 2 byte | 1 byte | 2 byte | 2 byte | 2 byte | 2 byte | N bytes
+--------+--------+--------+--------+--------+--------+--------+

- Magic: 0x464C ("FL")
- Version: 0x02
- SeqNum: Frame sequence number (1-indexed)
- TotalF: Total frames in update
- ChkSum: CRC-16 of payload
- PayLen: Payload length in bytes
- Payload: Model gradient/weight data
"""

import argparse
import hashlib
import json
import struct
import sys
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Optional, BinaryIO
import math


# Frame constants
FL_MAGIC = 0x464C  # "FL"
FL_VERSION = 0x02
HEADER_SIZE = 11  # 2+1+2+2+2+2
MAX_PAYLOAD_SIZE = 245  # AX.25 max info field (256) - header (11)
MAX_FRAMES = 255  # Practical limit for single update


@dataclass
class FLUpdateFrame:
    """Single FL-UPDATE frame."""
    magic: int
    version: int
    seq_num: int
    total_frames: int
    checksum: int
    payload_len: int
    payload: bytes

    def to_bytes(self) -> bytes:
        """Serialize frame to bytes."""
        header = struct.pack(
            "<HBHHH",
            self.magic,
            self.version,
            self.seq_num,
            self.total_frames,
            self.checksum
        )
        length = struct.pack("<H", self.payload_len)
        return header + length + self.payload

    def to_hex(self) -> str:
        """Convert frame to hex string."""
        return self.to_bytes().hex().upper()


@dataclass
class FLUpdateManifest:
    """Manifest for complete FL update."""
    update_id: str
    created_utc: str
    model_name: str
    model_version: str
    total_frames: int
    total_bytes: int
    frame_checksums: List[str]
    overall_sha256: str


def calculate_crc16(data: bytes) -> int:
    """
    Calculate CRC-16-CCITT.

    Args:
        data: Input bytes

    Returns:
        16-bit CRC value
    """
    crc = 0xFFFF
    polynomial = 0x1021

    for byte in data:
        crc ^= (byte << 8)
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ polynomial) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF

    return crc


def split_data(data: bytes, chunk_size: int) -> List[bytes]:
    """
    Split data into chunks.

    Args:
        data: Input data
        chunk_size: Maximum chunk size

    Returns:
        List of data chunks
    """
    return [data[i:i + chunk_size] for i in range(0, len(data), chunk_size)]


def build_update_frames(
    model_data: bytes,
    model_name: str = "unknown",
    model_version: str = "0.0.0"
) -> tuple[List[FLUpdateFrame], FLUpdateManifest]:
    """
    Build FL-UPDATE frames from model data.

    Args:
        model_data: Raw model gradient/weight bytes
        model_name: Name of the model
        model_version: Version string

    Returns:
        Tuple of (list of frames, manifest)
    """
    # Calculate overall hash
    overall_hash = hashlib.sha256(model_data).hexdigest()

    # Split into chunks
    chunks = split_data(model_data, MAX_PAYLOAD_SIZE)

    if len(chunks) > MAX_FRAMES:
        raise ValueError(f"Data too large: requires {len(chunks)} frames, max is {MAX_FRAMES}")

    total_frames = len(chunks)
    frames: List[FLUpdateFrame] = []
    checksums: List[str] = []

    for i, chunk in enumerate(chunks):
        checksum = calculate_crc16(chunk)
        checksums.append(f"{checksum:04X}")

        frame = FLUpdateFrame(
            magic=FL_MAGIC,
            version=FL_VERSION,
            seq_num=i + 1,  # 1-indexed
            total_frames=total_frames,
            checksum=checksum,
            payload_len=len(chunk),
            payload=chunk
        )
        frames.append(frame)

    # Generate update ID
    update_id = f"FL-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}-{overall_hash[:8]}"

    manifest = FLUpdateManifest(
        update_id=update_id,
        created_utc=datetime.now(timezone.utc).isoformat(),
        model_name=model_name,
        model_version=model_version,
        total_frames=total_frames,
        total_bytes=len(model_data),
        frame_checksums=checksums,
        overall_sha256=overall_hash
    )

    return (frames, manifest)


def validate_frame(frame_bytes: bytes) -> tuple[bool, str]:
    """
    Validate an FL-UPDATE frame.

    Args:
        frame_bytes: Raw frame bytes

    Returns:
        Tuple of (valid, error_message)
    """
    if len(frame_bytes) < HEADER_SIZE:
        return (False, f"Frame too short: {len(frame_bytes)} < {HEADER_SIZE}")

    magic = struct.unpack("<H", frame_bytes[0:2])[0]
    if magic != FL_MAGIC:
        return (False, f"Invalid magic: 0x{magic:04X} != 0x{FL_MAGIC:04X}")

    version = frame_bytes[2]
    if version != FL_VERSION:
        return (False, f"Invalid version: {version} != {FL_VERSION}")

    payload_len = struct.unpack("<H", frame_bytes[9:11])[0]
    expected_len = HEADER_SIZE + payload_len

    if len(frame_bytes) != expected_len:
        return (False, f"Length mismatch: {len(frame_bytes)} != {expected_len}")

    # Verify checksum
    stored_checksum = struct.unpack("<H", frame_bytes[5:7])[0]
    payload = frame_bytes[11:]
    calculated_checksum = calculate_crc16(payload)

    if stored_checksum != calculated_checksum:
        return (False, f"Checksum mismatch: 0x{stored_checksum:04X} != 0x{calculated_checksum:04X}")

    return (True, "OK")


def write_frames_to_file(
    frames: List[FLUpdateFrame],
    output_dir: Path,
    prefix: str = "frame"
) -> List[Path]:
    """
    Write frames to individual files.

    Args:
        frames: List of frames
        output_dir: Output directory
        prefix: Filename prefix

    Returns:
        List of written file paths
    """
    output_dir.mkdir(parents=True, exist_ok=True)
    paths: List[Path] = []

    for frame in frames:
        filename = f"{prefix}_{frame.seq_num:03d}_of_{frame.total_frames:03d}.bin"
        path = output_dir / filename
        with open(path, "wb") as f:
            f.write(frame.to_bytes())
        paths.append(path)

    return paths


def write_hex_file(
    frames: List[FLUpdateFrame],
    output_file: Path
) -> None:
    """
    Write all frames as hex to single file.

    Args:
        frames: List of frames
        output_file: Output file path
    """
    with open(output_file, "w") as f:
        for frame in frames:
            f.write(f"# Frame {frame.seq_num}/{frame.total_frames}\n")
            f.write(frame.to_hex() + "\n\n")


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="SMART-QSO Federated Learning Update Frame Builder",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s -i model_gradients.bin -o frames/
  %(prog)s -i weights.bin --name "qso-priority" --version "1.2.0"
  %(prog)s -i data.bin --hex-output update.hex
  %(prog)s --validate frame_001.bin
        """
    )

    parser.add_argument("-i", "--input", help="Input model data file")
    parser.add_argument("-o", "--output", help="Output directory for frame files")
    parser.add_argument("--hex-output", help="Output all frames as hex to single file")
    parser.add_argument("--manifest", help="Output manifest JSON file")
    parser.add_argument("--name", default="unknown", help="Model name")
    parser.add_argument("--version", default="0.0.0", help="Model version")
    parser.add_argument("--validate", help="Validate a frame file")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    # Validation mode
    if args.validate:
        try:
            with open(args.validate, "rb") as f:
                frame_data = f.read()
            valid, message = validate_frame(frame_data)
            print(f"Validation: {'PASS' if valid else 'FAIL'} - {message}")
            return 0 if valid else 1
        except IOError as e:
            print(f"Error reading file: {e}", file=sys.stderr)
            return 1

    # Build mode - requires input
    if not args.input:
        parser.print_help()
        return 1

    # Read input data
    try:
        with open(args.input, "rb") as f:
            model_data = f.read()
    except IOError as e:
        print(f"Error reading input: {e}", file=sys.stderr)
        return 1

    if args.verbose:
        print(f"Input size: {len(model_data)} bytes")

    # Build frames
    try:
        frames, manifest = build_update_frames(
            model_data,
            model_name=args.name,
            model_version=args.version
        )
    except ValueError as e:
        print(f"Error building frames: {e}", file=sys.stderr)
        return 1

    if args.verbose:
        print(f"Generated {len(frames)} frames")
        print(f"Update ID: {manifest.update_id}")

    # Write output
    if args.output:
        output_dir = Path(args.output)
        paths = write_frames_to_file(frames, output_dir)
        if args.verbose:
            for p in paths:
                print(f"  Written: {p}")

    if args.hex_output:
        write_hex_file(frames, Path(args.hex_output))
        if args.verbose:
            print(f"Hex output: {args.hex_output}")

    # Write manifest
    manifest_path = args.manifest or (
        Path(args.output) / "manifest.json" if args.output else None
    )
    if manifest_path:
        with open(manifest_path, "w") as f:
            json.dump(asdict(manifest), f, indent=2)
        if args.verbose:
            print(f"Manifest: {manifest_path}")

    # Print summary
    if not args.verbose:
        print(json.dumps(asdict(manifest), indent=2))

    return 0


if __name__ == "__main__":
    sys.exit(main())
