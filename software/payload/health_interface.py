"""
Health Interface Module for SMART-QSO Jetson Payload

This module provides the UART interface for receiving health commands
and telemetry requests from the OBC. It handles connection management,
data parsing, and validation per NPR 7150.2 Class B requirements.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import serial
import json
import threading
import time
import logging
from typing import Optional, Callable, Dict, Any
from dataclasses import dataclass
from enum import IntEnum
import struct
import crc

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class HealthCommand(IntEnum):
    """Command types received from OBC."""
    HEARTBEAT = 0x01
    STATUS_REQUEST = 0x02
    SHUTDOWN = 0x03
    RESTART = 0x04
    POWER_STATE = 0x05
    CONFIG_UPDATE = 0x06


class HealthStatus(IntEnum):
    """Health status codes."""
    OK = 0x00
    WARNING = 0x01
    ERROR = 0x02
    CRITICAL = 0x03


@dataclass
class HealthData:
    """Health data structure from OBC."""
    timestamp: int
    command: HealthCommand
    power_state: int
    battery_soc: float
    obc_temp: float
    payload_enabled: bool
    data: Optional[Dict[str, Any]] = None


class HealthInterface:
    """
    UART interface for health communication with OBC.

    This class manages the serial connection to the OBC, receives
    health commands, parses JSON data, and validates messages.
    """

    # Protocol constants
    FRAME_START = 0x7E
    FRAME_END = 0x7F
    MAX_FRAME_SIZE = 512
    HEARTBEAT_TIMEOUT_S = 30.0
    RECONNECT_INTERVAL_S = 5.0

    def __init__(
        self,
        port: str = "/dev/ttyTHS1",
        baudrate: int = 115200,
        timeout: float = 1.0
    ):
        """
        Initialize health interface.

        Args:
            port: Serial port path
            baudrate: Communication speed
            timeout: Read timeout in seconds
        """
        self.port = port
        self.baudrate = baudrate
        self.timeout = timeout

        self._serial: Optional[serial.Serial] = None
        self._running = False
        self._rx_thread: Optional[threading.Thread] = None
        self._last_heartbeat = 0.0
        self._connected = False

        # Callbacks
        self._command_callback: Optional[Callable[[HealthData], None]] = None
        self._disconnect_callback: Optional[Callable[[], None]] = None

        # Statistics
        self._rx_count = 0
        self._rx_errors = 0
        self._crc_errors = 0

        # CRC calculator
        self._crc = crc.Calculator(crc.Crc16.CCITT)

    def connect(self) -> bool:
        """
        Establish serial connection to OBC.

        Returns:
            True if connection successful
        """
        try:
            self._serial = serial.Serial(
                port=self.port,
                baudrate=self.baudrate,
                bytesize=serial.EIGHTBITS,
                parity=serial.PARITY_NONE,
                stopbits=serial.STOPBITS_ONE,
                timeout=self.timeout
            )
            self._connected = True
            self._last_heartbeat = time.time()
            logger.info(f"Connected to OBC on {self.port}")
            return True

        except serial.SerialException as e:
            logger.error(f"Failed to connect to {self.port}: {e}")
            self._connected = False
            return False

    def disconnect(self):
        """Close serial connection."""
        self._running = False
        if self._rx_thread and self._rx_thread.is_alive():
            self._rx_thread.join(timeout=2.0)

        if self._serial and self._serial.is_open:
            self._serial.close()

        self._connected = False
        logger.info("Disconnected from OBC")

    def start(self):
        """Start receive thread."""
        if not self._connected:
            if not self.connect():
                return

        self._running = True
        self._rx_thread = threading.Thread(target=self._receive_loop, daemon=True)
        self._rx_thread.start()
        logger.info("Health interface started")

    def stop(self):
        """Stop receive thread and disconnect."""
        self._running = False
        self.disconnect()
        logger.info("Health interface stopped")

    def is_connected(self) -> bool:
        """Check if connected to OBC."""
        return self._connected and self._serial is not None and self._serial.is_open

    def is_healthy(self) -> bool:
        """Check if connection is healthy (recent heartbeat)."""
        if not self.is_connected():
            return False
        return (time.time() - self._last_heartbeat) < self.HEARTBEAT_TIMEOUT_S

    def register_command_callback(self, callback: Callable[[HealthData], None]):
        """Register callback for received commands."""
        self._command_callback = callback

    def register_disconnect_callback(self, callback: Callable[[], None]):
        """Register callback for disconnection events."""
        self._disconnect_callback = callback

    def send_response(self, status: HealthStatus, data: Optional[Dict[str, Any]] = None) -> bool:
        """
        Send response to OBC.

        Args:
            status: Health status code
            data: Optional response data

        Returns:
            True if send successful
        """
        if not self.is_connected():
            return False

        try:
            response = {
                "status": int(status),
                "timestamp": int(time.time()),
            }
            if data:
                response["data"] = data

            payload = json.dumps(response).encode('utf-8')
            frame = self._build_frame(payload)

            self._serial.write(frame)
            return True

        except (serial.SerialException, OSError) as e:
            logger.error(f"Failed to send response: {e}")
            self._handle_disconnect()
            return False

    def get_statistics(self) -> Dict[str, int]:
        """Get interface statistics."""
        return {
            "rx_count": self._rx_count,
            "rx_errors": self._rx_errors,
            "crc_errors": self._crc_errors,
        }

    def _receive_loop(self):
        """Main receive loop - runs in separate thread."""
        buffer = bytearray()

        while self._running:
            try:
                if not self.is_connected():
                    self._attempt_reconnect()
                    continue

                # Read available data
                if self._serial.in_waiting > 0:
                    data = self._serial.read(self._serial.in_waiting)
                    buffer.extend(data)

                    # Process complete frames
                    while True:
                        frame = self._extract_frame(buffer)
                        if frame is None:
                            break
                        self._process_frame(frame)

                # Check heartbeat timeout
                if not self.is_healthy():
                    logger.warning("Heartbeat timeout")
                    self._handle_disconnect()

                time.sleep(0.01)  # Small delay to prevent busy loop

            except serial.SerialException as e:
                logger.error(f"Serial error in receive loop: {e}")
                self._handle_disconnect()
            except Exception as e:
                logger.error(f"Unexpected error in receive loop: {e}")
                self._rx_errors += 1

    def _extract_frame(self, buffer: bytearray) -> Optional[bytes]:
        """Extract a complete frame from buffer."""
        # Find frame start
        try:
            start_idx = buffer.index(self.FRAME_START)
        except ValueError:
            buffer.clear()
            return None

        # Remove bytes before start
        if start_idx > 0:
            del buffer[:start_idx]

        # Find frame end
        try:
            end_idx = buffer.index(self.FRAME_END, 1)
        except ValueError:
            # Check for buffer overflow
            if len(buffer) > self.MAX_FRAME_SIZE:
                buffer.clear()
            return None

        # Extract frame (excluding start/end markers)
        frame = bytes(buffer[1:end_idx])
        del buffer[:end_idx + 1]

        return frame

    def _process_frame(self, frame: bytes):
        """Process a received frame."""
        if len(frame) < 4:  # Minimum: 2 bytes CRC + some data
            self._rx_errors += 1
            return

        # Extract and verify CRC
        payload = frame[:-2]
        received_crc = struct.unpack('>H', frame[-2:])[0]
        calculated_crc = self._crc.checksum(payload)

        if received_crc != calculated_crc:
            logger.warning(f"CRC mismatch: {received_crc:04X} != {calculated_crc:04X}")
            self._crc_errors += 1
            return

        # Parse JSON payload
        try:
            data = json.loads(payload.decode('utf-8'))
            health_data = self._parse_health_data(data)

            if health_data:
                self._rx_count += 1
                self._handle_command(health_data)

        except json.JSONDecodeError as e:
            logger.warning(f"JSON parse error: {e}")
            self._rx_errors += 1
        except Exception as e:
            logger.error(f"Error processing frame: {e}")
            self._rx_errors += 1

    def _parse_health_data(self, data: Dict[str, Any]) -> Optional[HealthData]:
        """Parse health data from JSON."""
        try:
            return HealthData(
                timestamp=data.get("timestamp", 0),
                command=HealthCommand(data.get("command", 0)),
                power_state=data.get("power_state", 0),
                battery_soc=data.get("battery_soc", 0.0),
                obc_temp=data.get("obc_temp", 0.0),
                payload_enabled=data.get("payload_enabled", False),
                data=data.get("data"),
            )
        except (KeyError, ValueError) as e:
            logger.warning(f"Invalid health data: {e}")
            return None

    def _handle_command(self, health_data: HealthData):
        """Handle received command."""
        # Update heartbeat timestamp
        if health_data.command == HealthCommand.HEARTBEAT:
            self._last_heartbeat = time.time()

        # Invoke callback
        if self._command_callback:
            try:
                self._command_callback(health_data)
            except Exception as e:
                logger.error(f"Error in command callback: {e}")

    def _handle_disconnect(self):
        """Handle disconnection."""
        self._connected = False
        if self._serial and self._serial.is_open:
            try:
                self._serial.close()
            except:
                pass

        if self._disconnect_callback:
            try:
                self._disconnect_callback()
            except Exception as e:
                logger.error(f"Error in disconnect callback: {e}")

    def _attempt_reconnect(self):
        """Attempt to reconnect to OBC."""
        logger.info("Attempting reconnection...")
        time.sleep(self.RECONNECT_INTERVAL_S)
        self.connect()

    def _build_frame(self, payload: bytes) -> bytes:
        """Build a frame with start/end markers and CRC."""
        crc_value = self._crc.checksum(payload)
        crc_bytes = struct.pack('>H', crc_value)

        frame = bytes([self.FRAME_START])
        frame += payload
        frame += crc_bytes
        frame += bytes([self.FRAME_END])

        return frame


# Convenience function for testing
def create_health_interface(
    port: str = "/dev/ttyTHS1",
    baudrate: int = 115200
) -> HealthInterface:
    """Create and initialize health interface."""
    interface = HealthInterface(port=port, baudrate=baudrate)
    return interface
