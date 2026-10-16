"""
Health Interface Module for SMART-QSO Jetson Payload

This module provides the UART interface for receiving health commands
and telemetry requests from the OBC. It handles connection management,
data parsing, and validation per NPR 7150.2 Class B requirements.

The link carries ICD lines (docs/interfaces/ICD_OBC_JETSON.md section 4,
obc_link.py): TELEMETRY (heartbeat), STATUS_REQ, SHUTDOWN and BEACON_REQ
from the OBC; STATUS, BEACON_RSP, ACK and NACK to it.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import serial
import threading
import time
import logging
from typing import Optional, Callable, Dict, Any
from dataclasses import dataclass
from enum import IntEnum

import obc_link

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
    BEACON_REQUEST = 0x07


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
    ICD lines, checks their CRC, and turns them into commands.
    """

    # Protocol constants
    MAX_LINE_SIZE = obc_link.LINE_MAX
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
        Send a STATUS line to the OBC.

        Args:
            status: Health status code
            data: Supervisor metrics (temperature_c, power_w, memory_percent, uptime_s)

        Returns:
            True if send successful
        """
        data = data or {}
        try:
            line = obc_link.status(
                int(status),
                int(round(float(data.get("temperature_c", 0.0)))),
                int(round(float(data.get("power_w", 0.0)) * 1000.0)),
                int(round(float(data.get("memory_percent", 0.0)))),
                int(data.get("uptime_s", 0)),
            )
        except (obc_link.LinkError, TypeError, ValueError) as e:
            logger.error(f"Cannot build STATUS line: {e}")
            return False
        return self.send_line(line)

    def send_line(self, line: bytes) -> bool:
        """Send one already-sealed ICD line to the OBC."""
        if not self.is_connected():
            return False
        try:
            self._serial.write(line)
            return True
        except (serial.SerialException, OSError) as e:
            logger.error(f"Failed to send line: {e}")
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

                    # Process complete lines
                    while True:
                        line = self._extract_line(buffer)
                        if line is None:
                            break
                        self._process_line(line)

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

    def _extract_line(self, buffer: bytearray) -> Optional[bytes]:
        """Take one complete line (through the newline) from the buffer."""
        end = buffer.find(b"\n")
        if end < 0:
            if len(buffer) > self.MAX_LINE_SIZE:
                buffer.clear()
                self._rx_errors += 1
            return None
        line = bytes(buffer[:end + 1])
        del buffer[:end + 1]
        return line

    def _process_line(self, line: bytes):
        """Check one ICD line and dispatch it as a command."""
        try:
            msg = obc_link.parse(line)
            health_data = self._to_health_data(msg)
        except obc_link.CrcError:
            self._crc_errors += 1
            return
        except obc_link.LinkError as e:
            logger.warning(f"Rejected line: {e}")
            self._rx_errors += 1
            return

        self._rx_count += 1
        self._handle_command(health_data)

    def _to_health_data(self, msg: obc_link.Message) -> HealthData:
        """Map an OBC -> Jetson message to a command (LinkError if not one)."""
        now = int(time.time() * 1000)
        if msg.type == "TELEMETRY":
            t = obc_link.parse_telemetry(msg)
            extras: Dict[str, Any] = dict(t.extras)
            extras["sunlit"] = t.sunlit
            extras["power_limit_w"] = t.power_limit_w
            obc_temp = 0.0
            try:
                obc_temp = float(t.extras.get("OBC_T", "0").rstrip("C"))
            except ValueError:
                pass
            return HealthData(
                timestamp=t.time_ms,
                command=HealthCommand.HEARTBEAT,
                power_state=t.power_mode,
                battery_soc=t.soc,
                obc_temp=obc_temp,
                payload_enabled=t.extras.get("EPS_PAYLOAD", "ON") == "ON",
                data=extras,
            )
        if msg.type == "STATUS_REQ" and not msg.fields:
            return HealthData(now, HealthCommand.STATUS_REQUEST, 0, 0.0, 0.0, True)
        if msg.type == "SHUTDOWN" and len(msg.fields) == 1:
            return HealthData(now, HealthCommand.SHUTDOWN, 0, 0.0, 0.0, True,
                              {"reason": msg.fields[0]})
        if msg.type == "BEACON_REQ":
            req = obc_link.parse_beacon_req(msg)
            return HealthData(now, HealthCommand.BEACON_REQUEST, 0, 0.0, 0.0, True,
                              {"seq": req.seq, "max_len": req.max_len})
        raise obc_link.LinkError(f"{msg.type} is not an OBC -> Jetson message")

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


# Convenience function for testing
def create_health_interface(
    port: str = "/dev/ttyTHS1",
    baudrate: int = 115200
) -> HealthInterface:
    """Create and initialize health interface."""
    interface = HealthInterface(port=port, baudrate=baudrate)
    return interface
