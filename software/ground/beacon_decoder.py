"""
Beacon Decoder Module for SMART-QSO Ground Station

This module decodes AX.25 beacon frames from the SMART-QSO CubeSat,
extracts telemetry data, and provides parsed beacon information.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import re
import time
import struct
import logging
from typing import Optional, Dict, Any, List, Tuple
from dataclasses import dataclass, field
from enum import IntEnum

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class BeaconType(IntEnum):
    """Beacon content types."""
    STATUS = 0
    SCIENCE = 1
    GREETING = 2
    EMERGENCY = 3
    TEST = 4
    UNKNOWN = 255


@dataclass
class AX25Address:
    """AX.25 address field."""
    callsign: str
    ssid: int
    command_response: bool = False
    last: bool = False


@dataclass
class AX25Frame:
    """Parsed AX.25 frame."""
    destination: AX25Address
    source: AX25Address
    digipeaters: List[AX25Address] = field(default_factory=list)
    control: int = 0x03  # UI frame
    pid: int = 0xF0      # No layer 3
    info: bytes = b""
    fcs_valid: bool = False


@dataclass
class DecodedBeacon:
    """Decoded beacon content."""
    raw_frame: bytes
    ax25_frame: Optional[AX25Frame]
    beacon_type: BeaconType
    callsign: str
    sequence: Optional[int]
    timestamp: Optional[int]
    ai_generated: bool
    info_text: str
    telemetry: Dict[str, Any]
    decode_time: float
    valid: bool
    errors: List[str] = field(default_factory=list)


class BeaconDecoder:
    """
    AX.25 beacon decoder for SMART-QSO.

    Parses raw frames, validates checksums, and extracts
    structured data from beacon info fields.
    """

    # AX.25 constants
    FLAG = 0x7E
    MIN_FRAME_SIZE = 17  # dest(7) + src(7) + ctrl(1) + pid(1) + fcs(2)

    # Telemetry field patterns
    TELEMETRY_PATTERNS = {
        "battery_voltage": r"V:(\d+\.?\d*)V?",
        "battery_soc": r"B:(\d+)%",
        "temperature": r"T:(-?\d+)C",
        "power_mode": r"M:(\w+)",
        "orbit_number": r"O:#?(\d+)",
        "uptime": r"U:(\d+)s",
    }

    def __init__(self):
        """Initialize decoder."""
        self._decode_count = 0
        self._error_count = 0
        self._compiled_patterns = {
            key: re.compile(pattern)
            for key, pattern in self.TELEMETRY_PATTERNS.items()
        }

    def decode(self, raw_frame: bytes) -> DecodedBeacon:
        """
        Decode a raw AX.25 frame.

        Args:
            raw_frame: Raw frame bytes

        Returns:
            DecodedBeacon with parsed data
        """
        start_time = time.time()
        errors = []

        # Parse AX.25 frame
        ax25_frame = self._parse_ax25(raw_frame)
        if ax25_frame is None:
            errors.append("Failed to parse AX.25 frame")
            self._error_count += 1
            return DecodedBeacon(
                raw_frame=raw_frame,
                ax25_frame=None,
                beacon_type=BeaconType.UNKNOWN,
                callsign="",
                sequence=None,
                timestamp=None,
                ai_generated=False,
                info_text="",
                telemetry={},
                decode_time=time.time() - start_time,
                valid=False,
                errors=errors
            )

        # Extract info text
        try:
            info_text = ax25_frame.info.decode('ascii', errors='replace')
        except Exception as e:
            info_text = str(ax25_frame.info)
            errors.append(f"Info decode error: {e}")

        # Parse info field
        callsign = ax25_frame.source.callsign
        sequence = self._extract_sequence(info_text)
        timestamp = self._extract_timestamp(info_text)
        ai_generated = "[AI]" in info_text
        beacon_type = self._detect_beacon_type(info_text)
        telemetry = self._extract_telemetry(info_text)

        self._decode_count += 1

        return DecodedBeacon(
            raw_frame=raw_frame,
            ax25_frame=ax25_frame,
            beacon_type=beacon_type,
            callsign=callsign,
            sequence=sequence,
            timestamp=timestamp,
            ai_generated=ai_generated,
            info_text=info_text,
            telemetry=telemetry,
            decode_time=time.time() - start_time,
            valid=len(errors) == 0 and ax25_frame.fcs_valid,
            errors=errors
        )

    def decode_kiss(self, kiss_data: bytes) -> List[DecodedBeacon]:
        """
        Decode KISS-framed data.

        Args:
            kiss_data: KISS-encoded frame(s)

        Returns:
            List of decoded beacons
        """
        beacons = []
        frames = self._extract_kiss_frames(kiss_data)

        for frame in frames:
            beacon = self.decode(frame)
            beacons.append(beacon)

        return beacons

    def get_statistics(self) -> Dict[str, int]:
        """Get decoder statistics."""
        return {
            "decode_count": self._decode_count,
            "error_count": self._error_count,
        }

    def reset_statistics(self):
        """Reset statistics counters."""
        self._decode_count = 0
        self._error_count = 0

    def _parse_ax25(self, data: bytes) -> Optional[AX25Frame]:
        """Parse raw AX.25 frame."""
        # Remove flags if present
        if len(data) > 0 and data[0] == self.FLAG:
            data = data[1:]
        if len(data) > 0 and data[-1] == self.FLAG:
            data = data[:-1]

        if len(data) < self.MIN_FRAME_SIZE:
            return None

        try:
            # Parse destination (7 bytes)
            dest = self._parse_address(data[0:7])
            if dest is None:
                return None

            # Parse source (7 bytes)
            src = self._parse_address(data[7:14])
            if src is None:
                return None

            # Parse digipeaters
            digis = []
            offset = 14
            while not src.last and offset + 7 <= len(data) - 3:
                digi = self._parse_address(data[offset:offset+7])
                if digi is None:
                    break
                digis.append(digi)
                if digi.last:
                    break
                offset += 7

            # Control and PID
            control = data[offset]
            pid = data[offset + 1]

            # Info field (up to FCS)
            info = data[offset + 2:-2]

            # FCS
            fcs = struct.unpack('<H', data[-2:])[0]
            calculated_fcs = self._calculate_fcs(data[:-2])
            fcs_valid = (fcs == calculated_fcs)

            return AX25Frame(
                destination=dest,
                source=src,
                digipeaters=digis,
                control=control,
                pid=pid,
                info=info,
                fcs_valid=fcs_valid
            )

        except Exception as e:
            logger.error(f"AX.25 parse error: {e}")
            return None

    def _parse_address(self, data: bytes) -> Optional[AX25Address]:
        """Parse AX.25 address field."""
        if len(data) != 7:
            return None

        try:
            # Callsign: first 6 bytes, shifted right by 1
            callsign = ''.join(chr(b >> 1) for b in data[:6]).strip()

            # SSID byte
            ssid_byte = data[6]
            ssid = (ssid_byte >> 1) & 0x0F
            command_response = bool(ssid_byte & 0x80)
            last = bool(ssid_byte & 0x01)

            return AX25Address(
                callsign=callsign,
                ssid=ssid,
                command_response=command_response,
                last=last
            )
        except Exception:
            return None

    def _calculate_fcs(self, data: bytes) -> int:
        """Calculate AX.25 FCS (CRC-16-CCITT)."""
        fcs = 0xFFFF

        for byte in data:
            fcs ^= byte
            for _ in range(8):
                if fcs & 0x0001:
                    fcs = (fcs >> 1) ^ 0x8408
                else:
                    fcs >>= 1

        return fcs ^ 0xFFFF

    def _extract_kiss_frames(self, data: bytes) -> List[bytes]:
        """Extract AX.25 frames from KISS data."""
        frames = []
        FEND = 0xC0
        FESC = 0xDB
        TFEND = 0xDC
        TFESC = 0xDD

        # Split by FEND
        parts = data.split(bytes([FEND]))

        for part in parts:
            if len(part) < 2:
                continue

            # First byte is command (port/type)
            if (part[0] & 0x0F) != 0:  # Not data frame
                continue

            # Unescape
            frame = bytearray()
            i = 1
            while i < len(part):
                if part[i] == FESC:
                    if i + 1 < len(part):
                        if part[i + 1] == TFEND:
                            frame.append(FEND)
                        elif part[i + 1] == TFESC:
                            frame.append(FESC)
                        i += 2
                    else:
                        i += 1
                else:
                    frame.append(part[i])
                    i += 1

            if len(frame) >= self.MIN_FRAME_SIZE:
                frames.append(bytes(frame))

        return frames

    def _extract_sequence(self, info: str) -> Optional[int]:
        """Extract sequence number from info field."""
        match = re.search(r'#(\d{4})', info)
        if match:
            return int(match.group(1))
        return None

    def _extract_timestamp(self, info: str) -> Optional[int]:
        """Extract timestamp from info field."""
        match = re.search(r'T:(\d+)', info)
        if match:
            return int(match.group(1))
        return None

    def _detect_beacon_type(self, info: str) -> BeaconType:
        """Detect beacon type from info field."""
        info_upper = info.upper()

        if "EMERGENCY" in info_upper:
            return BeaconType.EMERGENCY
        if "TEST" in info_upper:
            return BeaconType.TEST
        if "SCIENCE" in info_upper:
            return BeaconType.SCIENCE
        if "GREETING" in info_upper or "CQ" in info_upper:
            return BeaconType.GREETING

        return BeaconType.STATUS

    def _extract_telemetry(self, info: str) -> Dict[str, Any]:
        """Extract telemetry values from info field."""
        telemetry = {}

        for key, pattern in self._compiled_patterns.items():
            match = pattern.search(info)
            if match:
                value = match.group(1)
                # Convert to appropriate type
                try:
                    if key in ("battery_voltage",):
                        telemetry[key] = float(value)
                    elif key in ("battery_soc", "temperature", "orbit_number", "uptime"):
                        telemetry[key] = int(value)
                    else:
                        telemetry[key] = value
                except ValueError:
                    telemetry[key] = value

        return telemetry


# Convenience function
def decode_beacon(raw_frame: bytes) -> DecodedBeacon:
    """
    Decode a raw AX.25 beacon frame.

    Args:
        raw_frame: Raw frame bytes

    Returns:
        DecodedBeacon with parsed data
    """
    decoder = BeaconDecoder()
    return decoder.decode(raw_frame)
