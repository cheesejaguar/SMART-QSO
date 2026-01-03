#!/usr/bin/env python3
"""
SMART-QSO Beacon Decoder

Decodes AX.25 amateur radio frames into structured JSON format.
Follows NASA Ames SmallSat ground software heritage from GeneSat/PharmaSat.

Document ID: SMART-QSO-GND-001
Version: 1.0
"""

import json
import struct
import sys
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from enum import IntEnum
from typing import Optional, Dict, Any, List
import argparse


class BeaconType(IntEnum):
    """Beacon type identifiers matching flight software."""
    HEALTH = 0x01
    SCIENCE = 0x02
    AI_STATUS = 0x03
    EMERGENCY = 0xFF


class PowerMode(IntEnum):
    """Power mode identifiers."""
    EMERGENCY = 0
    SAFE = 1
    IDLE = 2
    ACTIVE = 3
    BURST = 4


class SystemState(IntEnum):
    """System state identifiers."""
    BOOT = 0
    SAFE = 1
    DETUMBLE = 2
    IDLE = 3
    ACTIVE = 4
    EMERGENCY = 5


@dataclass
class AX25Header:
    """AX.25 frame header."""
    dest_callsign: str
    dest_ssid: int
    src_callsign: str
    src_ssid: int
    control: int
    pid: int


@dataclass
class HealthTelemetry:
    """Health beacon telemetry payload."""
    beacon_type: int
    sequence_number: int
    timestamp: int
    system_state: str
    power_mode: str
    battery_voltage_mv: int
    battery_current_ma: int
    state_of_charge_pct: int
    obc_temp_c: float
    battery_temp_c: float
    solar_temp_c: float
    eps_fault: bool
    comm_fault: bool
    thermal_fault: bool
    adcs_fault: bool
    crc32: int


@dataclass
class DecodedBeacon:
    """Complete decoded beacon."""
    raw_hex: str
    timestamp_utc: str
    ax25_header: Optional[AX25Header]
    payload: Optional[Dict[str, Any]]
    valid: bool
    errors: List[str]


def decode_callsign(data: bytes) -> tuple[str, int]:
    """
    Decode AX.25 callsign from 7 bytes.

    Args:
        data: 7 bytes of callsign data

    Returns:
        Tuple of (callsign string, SSID)
    """
    if len(data) < 7:
        return ("", 0)

    # Callsign is 6 bytes, each shifted left by 1
    callsign_chars = []
    for i in range(6):
        char = (data[i] >> 1) & 0x7F
        if char != 0x20:  # Skip padding spaces
            callsign_chars.append(chr(char))

    callsign = "".join(callsign_chars).strip()

    # SSID is in byte 7
    ssid = (data[6] >> 1) & 0x0F

    return (callsign, ssid)


def calculate_crc32(data: bytes) -> int:
    """
    Calculate CRC-32 matching flight software implementation.

    Args:
        data: Input bytes

    Returns:
        32-bit CRC value
    """
    crc = 0xFFFFFFFF
    polynomial = 0xEDB88320

    for byte in data:
        crc ^= byte
        for _ in range(8):
            if crc & 1:
                crc = (crc >> 1) ^ polynomial
            else:
                crc >>= 1

    return crc ^ 0xFFFFFFFF


def decode_ax25_header(frame: bytes) -> tuple[Optional[AX25Header], bytes]:
    """
    Decode AX.25 UI frame header.

    Args:
        frame: Raw frame bytes

    Returns:
        Tuple of (header, payload bytes)
    """
    if len(frame) < 16:
        return (None, b"")

    dest_callsign, dest_ssid = decode_callsign(frame[0:7])
    src_callsign, src_ssid = decode_callsign(frame[7:14])

    control = frame[14]
    pid = frame[15]

    header = AX25Header(
        dest_callsign=dest_callsign,
        dest_ssid=dest_ssid,
        src_callsign=src_callsign,
        src_ssid=src_ssid,
        control=control,
        pid=pid
    )

    return (header, frame[16:])


def decode_health_beacon(payload: bytes) -> Optional[HealthTelemetry]:
    """
    Decode health beacon payload.

    Expected format (28 bytes):
    - 1 byte: beacon type
    - 2 bytes: sequence number (little-endian)
    - 4 bytes: timestamp (little-endian)
    - 1 byte: system state
    - 1 byte: power mode
    - 2 bytes: battery voltage mV
    - 2 bytes: battery current mA
    - 1 byte: SOC percent
    - 2 bytes: OBC temp (0.1C resolution, signed)
    - 2 bytes: battery temp
    - 2 bytes: solar temp
    - 1 byte: fault flags
    - 4 bytes: CRC32

    Args:
        payload: Raw payload bytes

    Returns:
        Decoded health telemetry or None
    """
    if len(payload) < 25:
        return None

    try:
        beacon_type = payload[0]
        seq_num = struct.unpack("<H", payload[1:3])[0]
        timestamp = struct.unpack("<I", payload[3:7])[0]
        state = payload[7]
        power = payload[8]
        voltage = struct.unpack("<H", payload[9:11])[0]
        current = struct.unpack("<H", payload[11:13])[0]
        soc = payload[13]
        obc_temp = struct.unpack("<h", payload[14:16])[0] / 10.0
        batt_temp = struct.unpack("<h", payload[16:18])[0] / 10.0
        solar_temp = struct.unpack("<h", payload[18:20])[0] / 10.0
        faults = payload[20]
        crc = struct.unpack("<I", payload[21:25])[0]

        # Map state and power to strings
        state_name = SystemState(state).name if state < len(SystemState) else f"UNKNOWN({state})"
        power_name = PowerMode(power).name if power < len(PowerMode) else f"UNKNOWN({power})"

        return HealthTelemetry(
            beacon_type=beacon_type,
            sequence_number=seq_num,
            timestamp=timestamp,
            system_state=state_name,
            power_mode=power_name,
            battery_voltage_mv=voltage,
            battery_current_ma=current,
            state_of_charge_pct=soc,
            obc_temp_c=obc_temp,
            battery_temp_c=batt_temp,
            solar_temp_c=solar_temp,
            eps_fault=bool(faults & 0x01),
            comm_fault=bool(faults & 0x02),
            thermal_fault=bool(faults & 0x04),
            adcs_fault=bool(faults & 0x08),
            crc32=crc
        )
    except (struct.error, ValueError):
        return None


def decode_beacon(raw_hex: str) -> DecodedBeacon:
    """
    Decode a complete beacon from hex string.

    Args:
        raw_hex: Hexadecimal string of raw frame

    Returns:
        Decoded beacon with all fields
    """
    errors: List[str] = []

    # Parse hex string
    try:
        raw_hex_clean = raw_hex.replace(" ", "").replace("\n", "")
        frame = bytes.fromhex(raw_hex_clean)
    except ValueError as e:
        return DecodedBeacon(
            raw_hex=raw_hex,
            timestamp_utc=datetime.now(timezone.utc).isoformat(),
            ax25_header=None,
            payload=None,
            valid=False,
            errors=[f"Invalid hex string: {e}"]
        )

    # Decode AX.25 header
    header, payload_bytes = decode_ax25_header(frame)
    if header is None:
        errors.append("Failed to decode AX.25 header")

    # Decode payload based on type
    payload_dict: Optional[Dict[str, Any]] = None
    if len(payload_bytes) > 0:
        beacon_type = payload_bytes[0]

        if beacon_type == BeaconType.HEALTH:
            health = decode_health_beacon(payload_bytes)
            if health:
                payload_dict = asdict(health)
            else:
                errors.append("Failed to decode health beacon payload")
        elif beacon_type == BeaconType.EMERGENCY:
            payload_dict = {"type": "EMERGENCY", "raw": payload_bytes.hex()}
            errors.append("Emergency beacon detected")
        else:
            payload_dict = {"type": f"UNKNOWN({beacon_type})", "raw": payload_bytes.hex()}

    return DecodedBeacon(
        raw_hex=raw_hex_clean,
        timestamp_utc=datetime.now(timezone.utc).isoformat(),
        ax25_header=header,
        payload=payload_dict,
        valid=len(errors) == 0,
        errors=errors
    )


def beacon_to_json(beacon: DecodedBeacon, pretty: bool = True) -> str:
    """
    Convert decoded beacon to JSON string.

    Args:
        beacon: Decoded beacon
        pretty: Enable pretty printing

    Returns:
        JSON string
    """
    data = {
        "raw_hex": beacon.raw_hex,
        "timestamp_utc": beacon.timestamp_utc,
        "valid": beacon.valid,
        "errors": beacon.errors
    }

    if beacon.ax25_header:
        data["ax25_header"] = asdict(beacon.ax25_header)

    if beacon.payload:
        data["payload"] = beacon.payload

    if pretty:
        return json.dumps(data, indent=2)
    return json.dumps(data)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="SMART-QSO Beacon Decoder - AX.25 to JSON",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s "A0A0A0A0A0A060..."
  %(prog)s -f beacon.hex
  %(prog)s -f beacon.hex -o decoded.json
  cat beacon.hex | %(prog)s --stdin
        """
    )

    parser.add_argument("hex_data", nargs="?", help="Hex string of beacon frame")
    parser.add_argument("-f", "--file", help="Read hex data from file")
    parser.add_argument("-o", "--output", help="Output JSON file (default: stdout)")
    parser.add_argument("--stdin", action="store_true", help="Read hex data from stdin")
    parser.add_argument("--compact", action="store_true", help="Compact JSON output")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    # Get input hex data
    hex_data = ""
    if args.stdin:
        hex_data = sys.stdin.read().strip()
    elif args.file:
        try:
            with open(args.file, "r") as f:
                hex_data = f.read().strip()
        except IOError as e:
            print(f"Error reading file: {e}", file=sys.stderr)
            return 1
    elif args.hex_data:
        hex_data = args.hex_data
    else:
        parser.print_help()
        return 1

    # Decode beacon
    beacon = decode_beacon(hex_data)

    # Output JSON
    json_output = beacon_to_json(beacon, pretty=not args.compact)

    if args.output:
        try:
            with open(args.output, "w") as f:
                f.write(json_output)
            if args.verbose:
                print(f"Output written to {args.output}")
        except IOError as e:
            print(f"Error writing output: {e}", file=sys.stderr)
            return 1
    else:
        print(json_output)

    return 0 if beacon.valid else 2


if __name__ == "__main__":
    sys.exit(main())
