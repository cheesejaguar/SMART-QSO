"""
Unit tests for Health Interface Module

Tests UART communication, ICD line parsing, health data validation,
and connection management for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
from unittest.mock import Mock, patch, MagicMock
import json
import struct
import time
import threading

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from health_interface import (
    HealthInterface, HealthCommand, HealthStatus, HealthData,
    create_health_interface
)


class TestHealthCommand(unittest.TestCase):
    """Test HealthCommand enumeration."""

    def test_heartbeat_value(self):
        self.assertEqual(HealthCommand.HEARTBEAT, 0x01)

    def test_status_request_value(self):
        self.assertEqual(HealthCommand.STATUS_REQUEST, 0x02)

    def test_shutdown_value(self):
        self.assertEqual(HealthCommand.SHUTDOWN, 0x03)

    def test_power_state_value(self):
        self.assertEqual(HealthCommand.POWER_STATE, 0x05)


class TestHealthStatus(unittest.TestCase):
    """Test HealthStatus enumeration."""

    def test_ok_value(self):
        self.assertEqual(HealthStatus.OK, 0x00)

    def test_warning_value(self):
        self.assertEqual(HealthStatus.WARNING, 0x01)

    def test_error_value(self):
        self.assertEqual(HealthStatus.ERROR, 0x02)

    def test_critical_value(self):
        self.assertEqual(HealthStatus.CRITICAL, 0x03)


class TestHealthData(unittest.TestCase):
    """Test HealthData dataclass."""

    def test_creation(self):
        data = HealthData(
            timestamp=1234567890,
            command=HealthCommand.HEARTBEAT,
            power_state=2,
            battery_soc=85.0,
            obc_temp=25.0,
            payload_enabled=True
        )
        self.assertEqual(data.timestamp, 1234567890)
        self.assertEqual(data.command, HealthCommand.HEARTBEAT)
        self.assertEqual(data.power_state, 2)
        self.assertEqual(data.battery_soc, 85.0)
        self.assertTrue(data.payload_enabled)

    def test_optional_data(self):
        data = HealthData(
            timestamp=0,
            command=HealthCommand.STATUS_REQUEST,
            power_state=0,
            battery_soc=0.0,
            obc_temp=0.0,
            payload_enabled=False,
            data={"key": "value"}
        )
        self.assertEqual(data.data, {"key": "value"})

    def test_default_data_none(self):
        data = HealthData(
            timestamp=0,
            command=HealthCommand.HEARTBEAT,
            power_state=0,
            battery_soc=0.0,
            obc_temp=0.0,
            payload_enabled=False
        )
        self.assertIsNone(data.data)


class TestHealthInterface(unittest.TestCase):
    """Test HealthInterface class."""

    def setUp(self):
        """Set up test fixtures."""
        self.interface = HealthInterface(
            port="/dev/ttyTest",
            baudrate=115200,
            timeout=1.0
        )

    def tearDown(self):
        """Clean up after tests."""
        if hasattr(self, 'interface'):
            self.interface.stop()

    def test_init_defaults(self):
        """Test default initialization."""
        interface = HealthInterface()
        self.assertEqual(interface.port, "/dev/ttyTHS1")
        self.assertEqual(interface.baudrate, 115200)
        self.assertEqual(interface.timeout, 1.0)

    def test_init_custom(self):
        """Test custom initialization."""
        self.assertEqual(self.interface.port, "/dev/ttyTest")
        self.assertEqual(self.interface.baudrate, 115200)

    def test_not_connected_initially(self):
        """Test initial connection state."""
        self.assertFalse(self.interface.is_connected())

    def test_not_healthy_when_disconnected(self):
        """Test health check when disconnected."""
        self.assertFalse(self.interface.is_healthy())

    @patch('serial.Serial')
    def test_connect_success(self, mock_serial):
        """Test successful connection."""
        mock_serial.return_value = MagicMock()
        result = self.interface.connect()
        self.assertTrue(result)
        self.assertTrue(self.interface.is_connected())

    @patch('serial.Serial')
    def test_connect_failure(self, mock_serial):
        """Test connection failure."""
        import serial
        mock_serial.side_effect = serial.SerialException("Port not found")
        result = self.interface.connect()
        self.assertFalse(result)
        self.assertFalse(self.interface.is_connected())

    def test_register_command_callback(self):
        """Test callback registration."""
        callback = Mock()
        self.interface.register_command_callback(callback)
        # No assertion needed - just verify no exception

    def test_register_disconnect_callback(self):
        """Test disconnect callback registration."""
        callback = Mock()
        self.interface.register_disconnect_callback(callback)
        # No assertion needed - just verify no exception

    def test_get_statistics_initial(self):
        """Test initial statistics."""
        stats = self.interface.get_statistics()
        self.assertEqual(stats["rx_count"], 0)
        self.assertEqual(stats["rx_errors"], 0)
        self.assertEqual(stats["crc_errors"], 0)

    @patch('serial.Serial')
    def test_send_response_when_disconnected(self, mock_serial):
        """Test sending response when disconnected."""
        result = self.interface.send_response(HealthStatus.OK)
        self.assertFalse(result)

    @patch('serial.Serial')
    def test_send_response_success(self, mock_serial):
        """Test successful response send."""
        mock_instance = MagicMock()
        mock_instance.is_open = True
        mock_serial.return_value = mock_instance

        self.interface.connect()
        result = self.interface.send_response(HealthStatus.OK, {"test": "data"})
        self.assertTrue(result)


class TestLineFraming(unittest.TestCase):
    """ICD line framing (docs/interfaces/ICD_OBC_JETSON.md section 4)."""

    def setUp(self):
        self.interface = HealthInterface()

    def test_extract_line_complete(self):
        buffer = bytearray(b"STATUS_REQ,5D03\nTELE")
        self.assertEqual(self.interface._extract_line(buffer), b"STATUS_REQ,5D03\n")
        self.assertEqual(bytes(buffer), b"TELE")

    def test_extract_line_incomplete(self):
        buffer = bytearray(b"STATUS_REQ,5D")
        self.assertIsNone(self.interface._extract_line(buffer))
        self.assertEqual(bytes(buffer), b"STATUS_REQ,5D")

    def test_extract_line_overflow_discarded(self):
        buffer = bytearray(b"X" * (HealthInterface.MAX_LINE_SIZE + 1))
        self.assertIsNone(self.interface._extract_line(buffer))
        self.assertEqual(len(buffer), 0)
        self.assertEqual(self.interface.get_statistics()["rx_errors"], 1)

    @patch('serial.Serial')
    def test_send_response_writes_status_line(self, mock_serial):
        """send_response emits the ICD STATUS line (golden vector shared with the OBC)."""
        mock_instance = MagicMock()
        mock_instance.is_open = True
        mock_serial.return_value = mock_instance
        self.interface.connect()

        data = {"temperature_c": 45.2, "power_w": 6.8, "memory_percent": 37.4, "uptime_s": 1234}
        self.assertTrue(self.interface.send_response(HealthStatus.OK, data))
        mock_instance.write.assert_called_once_with(b"STATUS,0,45,6800,37,1234,D775\n")

    @patch('serial.Serial')
    def test_send_line(self, mock_serial):
        mock_instance = MagicMock()
        mock_instance.is_open = True
        mock_serial.return_value = mock_instance
        self.interface.connect()
        self.assertTrue(self.interface.send_line(b"ACK,BEACON_RSP,275C\n"))
        mock_instance.write.assert_called_once_with(b"ACK,BEACON_RSP,275C\n")


class TestLineProcessing(unittest.TestCase):
    """OBC lines become HealthData commands."""

    def setUp(self):
        self.interface = HealthInterface()
        self.received = []
        self.interface.register_command_callback(self.received.append)

    def test_telemetry_is_heartbeat(self):
        self.interface._process_line(
            b"TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86\n")
        self.assertEqual(len(self.received), 1)
        data = self.received[0]
        self.assertEqual(data.command, HealthCommand.HEARTBEAT)
        self.assertEqual(data.timestamp, 1704153600000)
        self.assertEqual(data.power_state, 2)
        self.assertAlmostEqual(data.battery_soc, 0.75)
        self.assertTrue(data.payload_enabled)
        self.assertTrue(data.data["sunlit"])
        self.assertEqual(data.data["BV"], "8.1")

    def test_status_request_shutdown_beacon_request(self):
        self.interface._process_line(b"STATUS_REQ,5D03\n")
        self.interface._process_line(b"SHUTDOWN,POWER,05D4\n")
        self.interface._process_line(b"BEACON_REQ,42,176,5593\n")
        commands = [d.command for d in self.received]
        self.assertEqual(commands, [HealthCommand.STATUS_REQUEST, HealthCommand.SHUTDOWN,
                                    HealthCommand.BEACON_REQUEST])
        self.assertEqual(self.received[1].data, {"reason": "POWER"})
        self.assertEqual(self.received[2].data, {"seq": 42, "max_len": 176})

    def test_bad_crc_counted(self):
        self.interface._process_line(b"STATUS_REQ,5D04\n")
        self.assertEqual(self.received, [])
        self.assertEqual(self.interface.get_statistics()["crc_errors"], 1)

    def test_malformed_and_wrong_direction_counted(self):
        self.interface._process_line(b"garbage\n")
        self.interface._process_line(b"BEACON_RSP,42,Hello, world from orbit,0784\n")
        self.assertEqual(self.received, [])
        self.assertEqual(self.interface.get_statistics()["rx_errors"], 2)


class TestConvenienceFunction(unittest.TestCase):
    """Test convenience functions."""

    def test_create_health_interface(self):
        """Test factory function."""
        interface = create_health_interface(port="/dev/test", baudrate=9600)
        self.assertEqual(interface.port, "/dev/test")
        self.assertEqual(interface.baudrate, 9600)


if __name__ == '__main__':
    unittest.main()
