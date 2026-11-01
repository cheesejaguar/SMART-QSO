"""
Unit tests for Payload Supervisor Module

Tests process lifecycle management, health reporting, thermal
throttling, and power state awareness for SMART-QSO Jetson payload.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
import subprocess
import time
import threading

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from supervisor import (
    PayloadSupervisor, SupervisorState, PowerState,
    SupervisorConfig, SystemMetrics, ProcessInfo
)

# A real, harmless child process that runs until it is stopped
SLEEPER = [sys.executable, "-c", "import time; time.sleep(60)"]


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
        self.supervisor._stop_all_processes()
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

    def test_start_process(self):
        """start_process launches the registered command as a real child process."""
        self.supervisor.register_process(name="test", command=SLEEPER)
        result = self.supervisor.start_process("test")
        self.assertTrue(result)
        proc = self.supervisor._processes["test"].process
        self.assertIsInstance(proc, subprocess.Popen)
        self.assertIsNone(proc.poll())  # still running
        self.assertGreater(self.supervisor._processes["test"].last_start_time, 0)

    def test_start_process_bad_command(self):
        """A command that cannot be executed is reported as a failed start."""
        self.supervisor.register_process(
            name="test", command=["/nonexistent/smart-qso-binary"])
        self.assertFalse(self.supervisor.start_process("test"))
        self.assertIsNone(self.supervisor._processes["test"].process)

    def test_start_unregistered_process(self):
        """Test starting unregistered process."""
        result = self.supervisor.start_process("nonexistent")
        self.assertFalse(result)

    def test_stop_process(self):
        """stop_process terminates the running child and forgets it."""
        self.supervisor.register_process(name="test", command=SLEEPER)
        self.supervisor.start_process("test")
        proc = self.supervisor._processes["test"].process

        result = self.supervisor.stop_process("test", timeout=5.0)
        self.assertTrue(result)
        self.assertIsNotNone(proc.poll())  # the child has exited
        self.assertIsNone(self.supervisor._processes["test"].process)

    def test_stop_process_not_started(self):
        """Stopping a registered process that never started succeeds."""
        self.supervisor.register_process(name="test", command=SLEEPER)
        self.assertTrue(self.supervisor.stop_process("test"))

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
        self.supervisor._stop_all_processes()
        if self.supervisor._running:
            self.supervisor.stop()

    def test_get_process_status_empty(self):
        """Test process status with no registered processes."""
        status = self.supervisor._get_process_status()
        self.assertEqual(status, {})

    def test_get_process_status_with_processes(self):
        """Process status reports the real child's PID while it runs, and not after."""
        self.supervisor.register_process(name="test", command=SLEEPER)
        self.supervisor.start_process("test")
        pid = self.supervisor._processes["test"].process.pid

        status = self.supervisor._get_process_status()
        self.assertEqual(status, {"test": {"running": True, "pid": pid, "restarts": 0}})

        self.supervisor.stop_process("test")
        status = self.supervisor._get_process_status()
        self.assertEqual(status, {"test": {"running": False, "pid": None, "restarts": 0}})


class TestBeaconRequest(unittest.TestCase):
    """The supervisor owns the UART and answers the OBC's BEACON_REQ."""

    def test_beacon_request_answered_with_beacon_response(self):
        import serial
        from health_interface import HealthCommand, HealthData, HealthInterface
        import obc_link

        sup = PayloadSupervisor()
        # The real health interface, on pyserial's loopback port in place of
        # the Jetson UART: what the supervisor writes can be read back
        sup._health = HealthInterface()
        sup._health._serial = serial.serial_for_url("loop://", timeout=1.0)
        sup._health._connected = True
        self.addCleanup(sup._health._serial.close)

        sup._handle_health_command(HealthData(
            timestamp=1, command=HealthCommand.HEARTBEAT, power_state=1, battery_soc=0.2,
            obc_temp=20.0, payload_enabled=True, data={"sunlit": False}))
        sup._handle_health_command(HealthData(
            timestamp=2, command=HealthCommand.BEACON_REQUEST, power_state=0, battery_soc=0.0,
            obc_temp=0.0, payload_enabled=True, data={"seq": 9, "max_len": 40}))

        line = sup._health._serial.readline()
        self.assertEqual(sup._health._serial.in_waiting, 0)  # exactly one line
        msg = obc_link.parse(line)
        self.assertEqual(msg.type, "BEACON_RSP")
        self.assertEqual(msg.fields[0], "9")
        self.assertTrue(0 < len(msg.fields[1]) <= 40)


class TestShutdownWithProcesses(unittest.TestCase):
    """Stopping and restarting managed processes must not deadlock on the process lock."""

    def _reap(self, sup, children):
        """Kill and reap the children directly: the supervisor's lock may be held."""
        children = children + [i.process for i in sup._processes.values() if i.process]
        for proc in children:
            proc.kill()
            proc.wait()
            for pipe in (proc.stdout, proc.stderr):
                if pipe is not None:
                    pipe.close()

    def _run_with_timeout(self, fn, timeout=10.0):
        worker = threading.Thread(target=fn, daemon=True)
        worker.start()
        worker.join(timeout)
        self.assertFalse(worker.is_alive(), f"{fn.__name__} did not return (deadlock)")

    def test_stop_terminates_running_processes(self):
        sup = PayloadSupervisor()
        sup.register_process(name="worker", command=SLEEPER)
        sup.start_process("worker")
        proc = sup._processes["worker"].process
        self.addCleanup(self._reap, sup, [proc])

        self._run_with_timeout(sup.stop)
        self.assertIsNotNone(proc.poll())
        self.assertEqual(sup.get_state(), SupervisorState.STOPPED)

    def test_exited_process_is_restarted(self):
        sup = PayloadSupervisor()
        sup.register_process(name="worker", command=[sys.executable, "-c", "pass"],
                             restart_on_failure=True, max_restarts=2)
        sup.start_process("worker")
        first = sup._processes["worker"].process
        first.wait(timeout=10)
        self.addCleanup(self._reap, sup, [first])

        self._run_with_timeout(sup._check_processes)
        info = sup._processes["worker"]
        self.assertEqual(info.restart_count, 1)
        self.assertIsNotNone(info.process)  # started again


if __name__ == '__main__':
    unittest.main()
