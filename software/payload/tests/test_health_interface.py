"""
Unit tests for Health Interface Module

Tests UART communication, JSON parsing, health data validation,
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


class TestFrameBuilding(unittest.TestCase):
    """Test frame building and parsing."""

    def setUp(self):
        self.interface = HealthInterface()

    def test_build_frame_structure(self):
        """Test frame structure with start/end markers."""
        payload = b'{"test": "data"}'
        frame = self.interface._build_frame(payload)

        # Check start marker
        self.assertEqual(frame[0], 0x7E)
        # Check end marker
        self.assertEqual(frame[-1], 0x7F)
        # Check payload is in frame
        self.assertIn(payload, frame)

    def test_extract_frame_complete(self):
        """Test extracting complete frame from buffer."""
        buffer = bytearray([0x7E, 0x01, 0x02, 0x03, 0x7F])
        frame = self.interface._extract_frame(buffer)
        self.assertEqual(frame, bytes([0x01, 0x02, 0x03]))
        self.assertEqual(len(buffer), 0)

    def test_extract_frame_incomplete(self):
        """Test extracting incomplete frame."""
        buffer = bytearray([0x7E, 0x01, 0x02])  # No end marker
        frame = self.interface._extract_frame(buffer)
        self.assertIsNone(frame)

    def test_extract_frame_no_start(self):
        """Test buffer with no start marker."""
        buffer = bytearray([0x01, 0x02, 0x03])
        frame = self.interface._extract_frame(buffer)
        self.assertIsNone(frame)
        self.assertEqual(len(buffer), 0)


class TestHealthDataParsing(unittest.TestCase):
    """Test health data parsing."""

    def setUp(self):
        self.interface = HealthInterface()

    def test_parse_valid_data(self):
        """Test parsing valid health data."""
        data = {
            "timestamp": 1234567890,
            "command": 1,
            "power_state": 2,
            "battery_soc": 85.5,
            "obc_temp": 25.0,
            "payload_enabled": True
        }
        result = self.interface._parse_health_data(data)
        self.assertIsNotNone(result)
        self.assertEqual(result.timestamp, 1234567890)
        self.assertEqual(result.command, HealthCommand.HEARTBEAT)

    def test_parse_missing_fields(self):
        """Test parsing with missing fields."""
        data = {"timestamp": 0}
        result = self.interface._parse_health_data(data)
        # Should handle missing fields gracefully
        self.assertIsNotNone(result)

    def test_parse_invalid_command(self):
        """Test parsing with invalid command value."""
        data = {
            "timestamp": 0,
            "command": 255,  # Invalid
            "power_state": 0,
            "battery_soc": 0,
            "obc_temp": 0,
            "payload_enabled": False
        }
        result = self.interface._parse_health_data(data)
        # Should return None for invalid data
        self.assertIsNone(result)


class TestConvenienceFunction(unittest.TestCase):
    """Test convenience functions."""

    def test_create_health_interface(self):
        """Test factory function."""
        interface = create_health_interface(port="/dev/test", baudrate=9600)
        self.assertEqual(interface.port, "/dev/test")
        self.assertEqual(interface.baudrate, 9600)


if __name__ == '__main__':
    unittest.main()
