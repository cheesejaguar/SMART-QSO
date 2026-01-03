"""
Unit tests for Payload Supervisor Module

Tests process lifecycle management, health reporting, thermal
throttling, and power state awareness for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
from unittest.mock import Mock, patch, MagicMock
import time
import threading

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from supervisor import (
    PayloadSupervisor, SupervisorState, PowerState,
    SupervisorConfig, SystemMetrics, ProcessInfo
)


class TestSupervisorState(unittest.TestCase):
    """Test SupervisorState enumeration."""

    def test_states_exist(self):
        self.assertTrue(hasattr(SupervisorState, 'INIT'))
        self.assertTrue(hasattr(SupervisorState, 'STARTING'))
        self.assertTrue(hasattr(SupervisorState, 'RUNNING'))
        self.assertTrue(hasattr(SupervisorState, 'THROTTLING'))
        self.assertTrue(hasattr(SupervisorState, 'SHUTTING_DOWN'))
        self.assertTrue(hasattr(SupervisorState, 'STOPPED'))


class TestPowerState(unittest.TestCase):
    """Test PowerState enumeration."""

    def test_safe_value(self):
        self.assertEqual(PowerState.SAFE, 0)

    def test_idle_value(self):
        self.assertEqual(PowerState.IDLE, 1)

    def test_active_value(self):
        self.assertEqual(PowerState.ACTIVE, 2)


class TestSupervisorConfig(unittest.TestCase):
    """Test SupervisorConfig dataclass."""

    def test_defaults(self):
        config = SupervisorConfig()
        self.assertEqual(config.health_port, "/dev/ttyTHS1")
        self.assertEqual(config.health_baudrate, 115200)
        self.assertEqual(config.status_interval_s, 10.0)
        self.assertEqual(config.thermal_throttle_temp_c, 75.0)
        self.assertEqual(config.thermal_shutdown_temp_c, 85.0)

    def test_custom_values(self):
        config = SupervisorConfig(
            health_port="/dev/test",
            thermal_throttle_temp_c=70.0
        )
        self.assertEqual(config.health_port, "/dev/test")
        self.assertEqual(config.thermal_throttle_temp_c, 70.0)


class TestSystemMetrics(unittest.TestCase):
    """Test SystemMetrics dataclass."""

    def test_defaults(self):
        metrics = SystemMetrics()
        self.assertEqual(metrics.cpu_percent, 0.0)
        self.assertEqual(metrics.memory_percent, 0.0)
        self.assertEqual(metrics.gpu_utilization, 0.0)
        self.assertEqual(metrics.temperature_c, 0.0)

    def test_custom_values(self):
        metrics = SystemMetrics(
            cpu_percent=50.0,
            memory_percent=60.0,
            temperature_c=45.0
        )
        self.assertEqual(metrics.cpu_percent, 50.0)
        self.assertEqual(metrics.memory_percent, 60.0)
        self.assertEqual(metrics.temperature_c, 45.0)


class TestPayloadSupervisor(unittest.TestCase):
    """Test PayloadSupervisor class."""

    def setUp(self):
        """Set up test fixtures."""
        self.config = SupervisorConfig()
        self.supervisor = PayloadSupervisor(config=self.config)

    def tearDown(self):
        """Clean up after tests."""
        if self.supervisor._running:
            self.supervisor.stop()

    def test_init_state(self):
        """Test initial state."""
        self.assertEqual(self.supervisor.get_state(), SupervisorState.INIT)

    def test_init_power_state(self):
        """Test initial power state."""
        self.assertEqual(self.supervisor.get_power_state(), PowerState.IDLE)

    def test_get_metrics(self):
        """Test getting metrics."""
        metrics = self.supervisor.get_metrics()
        self.assertIsInstance(metrics, SystemMetrics)

    def test_is_healthy_when_not_running(self):
        """Test health check when not running."""
        self.assertFalse(self.supervisor.is_healthy())


class TestProcessManagement(unittest.TestCase):
    """Test process registration and management."""

    def setUp(self):
        self.supervisor = PayloadSupervisor()

    def tearDown(self):
        if self.supervisor._running:
            self.supervisor.stop()

    def test_register_process(self):
        """Test process registration."""
        self.supervisor.register_process(
            name="test_process",
            command=["python3", "-c", "print('test')"],
            restart_on_failure=True,
            max_restarts=3
        )
        self.assertIn("test_process", self.supervisor._processes)

    def test_register_process_config(self):
        """Test registered process configuration."""
        self.supervisor.register_process(
            name="test",
            command=["echo", "hello"],
            restart_on_failure=False,
            max_restarts=5
        )
        proc = self.supervisor._processes["test"]
        self.assertEqual(proc.name, "test")
        self.assertEqual(proc.command, ["echo", "hello"])
        self.assertFalse(proc.restart_on_failure)
        self.assertEqual(proc.max_restarts, 5)

    @patch('subprocess.Popen')
    def test_start_process(self, mock_popen):
        """Test starting a process."""
        mock_popen.return_value = MagicMock()
        mock_popen.return_value.pid = 12345

        self.supervisor.register_process(
            name="test",
            command=["echo", "test"]
        )
        result = self.supervisor.start_process("test")
        self.assertTrue(result)

    def test_start_unregistered_process(self):
        """Test starting unregistered process."""
        result = self.supervisor.start_process("nonexistent")
        self.assertFalse(result)

    @patch('subprocess.Popen')
    def test_stop_process(self, mock_popen):
        """Test stopping a process."""
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None  # Still running
        mock_popen.return_value = mock_proc

        self.supervisor.register_process(
            name="test",
            command=["sleep", "100"]
        )
        self.supervisor.start_process("test")
        result = self.supervisor.stop_process("test")
        self.assertTrue(result)
        mock_proc.terminate.assert_called_once()

    def test_stop_unregistered_process(self):
        """Test stopping unregistered process."""
        result = self.supervisor.stop_process("nonexistent")
        self.assertFalse(result)


class TestThermalManagement(unittest.TestCase):
    """Test thermal throttling behavior."""

    def setUp(self):
        self.config = SupervisorConfig(
            thermal_throttle_temp_c=70.0,
            thermal_shutdown_temp_c=85.0
        )
        self.supervisor = PayloadSupervisor(config=self.config)

    def tearDown(self):
        if self.supervisor._running:
            self.supervisor.stop()

    def test_check_thermal_normal(self):
        """Test thermal check at normal temperature."""
        self.supervisor._metrics.temperature_c = 50.0
        self.supervisor._state = SupervisorState.RUNNING
        self.supervisor._check_thermal()
        self.assertEqual(self.supervisor._state, SupervisorState.RUNNING)

    def test_check_thermal_throttle(self):
        """Test thermal throttling activation."""
        self.supervisor._metrics.temperature_c = 75.0
        self.supervisor._state = SupervisorState.RUNNING
        self.supervisor._check_thermal()
        self.assertEqual(self.supervisor._state, SupervisorState.THROTTLING)

    def test_check_thermal_recovery(self):
        """Test thermal throttling recovery."""
        self.supervisor._state = SupervisorState.THROTTLING
        self.supervisor._metrics.temperature_c = 50.0
        self.supervisor._check_thermal()
        self.assertEqual(self.supervisor._state, SupervisorState.RUNNING)


class TestPowerStateHandling(unittest.TestCase):
    """Test power state handling."""

    def setUp(self):
        self.supervisor = PayloadSupervisor()

    def tearDown(self):
        if self.supervisor._running:
            self.supervisor.stop()

    def test_handle_safe_power_state(self):
        """Test handling SAFE power state."""
        self.supervisor._state = SupervisorState.RUNNING
        self.supervisor._power_state = PowerState.SAFE
        self.supervisor._handle_power_state_change()
        self.assertEqual(self.supervisor._state, SupervisorState.THROTTLING)

    def test_handle_active_power_state(self):
        """Test handling ACTIVE power state."""
        self.supervisor._state = SupervisorState.THROTTLING
        self.supervisor._power_state = PowerState.ACTIVE
        self.supervisor._handle_power_state_change()
        self.assertEqual(self.supervisor._state, SupervisorState.RUNNING)

    def test_handle_idle_power_state(self):
        """Test handling IDLE power state."""
        self.supervisor._state = SupervisorState.THROTTLING
        self.supervisor._power_state = PowerState.IDLE
        self.supervisor._handle_power_state_change()
        self.assertEqual(self.supervisor._state, SupervisorState.RUNNING)


class TestProcessStatus(unittest.TestCase):
    """Test process status reporting."""

    def setUp(self):
        self.supervisor = PayloadSupervisor()

    def tearDown(self):
        if self.supervisor._running:
            self.supervisor.stop()

    def test_get_process_status_empty(self):
        """Test process status with no registered processes."""
        status = self.supervisor._get_process_status()
        self.assertEqual(status, {})

    @patch('subprocess.Popen')
    def test_get_process_status_with_processes(self, mock_popen):
        """Test process status with registered processes."""
        mock_proc = MagicMock()
        mock_proc.pid = 12345
        mock_proc.poll.return_value = None  # Still running
        mock_popen.return_value = mock_proc

        self.supervisor.register_process(
            name="test",
            command=["echo", "test"]
        )
        self.supervisor.start_process("test")

        status = self.supervisor._get_process_status()
        self.assertIn("test", status)
        self.assertTrue(status["test"]["running"])
        self.assertEqual(status["test"]["pid"], 12345)


if __name__ == '__main__':
    unittest.main()
