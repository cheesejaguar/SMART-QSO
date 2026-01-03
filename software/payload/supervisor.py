"""
Payload Supervisor Module for SMART-QSO Jetson

This module provides process lifecycle management, health reporting
to OBC, graceful shutdown handling, and power/thermal awareness.
It serves as the main entry point for the Jetson payload software.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import os
import sys
import signal
import time
import threading
import logging
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
from enum import IntEnum, auto
import subprocess
import psutil

from health_interface import (
    HealthInterface, HealthData, HealthCommand, HealthStatus
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class SupervisorState(IntEnum):
    """Supervisor state machine states."""
    INIT = auto()
    STARTING = auto()
    RUNNING = auto()
    THROTTLING = auto()
    SHUTTING_DOWN = auto()
    STOPPED = auto()


class PowerState(IntEnum):
    """Power states from OBC."""
    SAFE = 0
    IDLE = 1
    ACTIVE = 2


@dataclass
class ProcessInfo:
    """Information about a managed process."""
    name: str
    command: List[str]
    restart_on_failure: bool = True
    max_restarts: int = 3
    restart_count: int = 0
    process: Optional[subprocess.Popen] = None
    last_start_time: float = 0.0


@dataclass
class SupervisorConfig:
    """Supervisor configuration."""
    health_port: str = "/dev/ttyTHS1"
    health_baudrate: int = 115200
    status_interval_s: float = 10.0
    thermal_throttle_temp_c: float = 75.0
    thermal_shutdown_temp_c: float = 85.0
    max_memory_percent: float = 80.0
    gpu_utilization_target: float = 70.0


@dataclass
class SystemMetrics:
    """Current system metrics."""
    cpu_percent: float = 0.0
    memory_percent: float = 0.0
    gpu_utilization: float = 0.0
    gpu_memory_used_mb: float = 0.0
    temperature_c: float = 0.0
    power_draw_w: float = 0.0
    uptime_s: float = 0.0


class PayloadSupervisor:
    """
    Main supervisor for Jetson payload.

    Manages process lifecycle, health reporting, power states,
    and thermal throttling per NPR 7150.2 requirements.
    """

    def __init__(self, config: Optional[SupervisorConfig] = None):
        """
        Initialize supervisor.

        Args:
            config: Supervisor configuration
        """
        self.config = config or SupervisorConfig()

        self._state = SupervisorState.INIT
        self._power_state = PowerState.IDLE
        self._running = False
        self._shutdown_requested = False
        self._start_time = time.time()

        # Health interface
        self._health: Optional[HealthInterface] = None

        # Managed processes
        self._processes: Dict[str, ProcessInfo] = {}
        self._process_lock = threading.Lock()

        # Threads
        self._monitor_thread: Optional[threading.Thread] = None
        self._status_thread: Optional[threading.Thread] = None

        # Current metrics
        self._metrics = SystemMetrics()
        self._metrics_lock = threading.Lock()

        # Register signal handlers
        signal.signal(signal.SIGTERM, self._signal_handler)
        signal.signal(signal.SIGINT, self._signal_handler)

    def start(self):
        """Start the supervisor."""
        logger.info("Starting payload supervisor")
        self._state = SupervisorState.STARTING
        self._running = True

        # Initialize health interface
        self._health = HealthInterface(
            port=self.config.health_port,
            baudrate=self.config.health_baudrate
        )
        self._health.register_command_callback(self._handle_health_command)
        self._health.register_disconnect_callback(self._handle_obc_disconnect)
        self._health.start()

        # Start monitoring thread
        self._monitor_thread = threading.Thread(
            target=self._monitor_loop,
            daemon=True
        )
        self._monitor_thread.start()

        # Start status reporting thread
        self._status_thread = threading.Thread(
            target=self._status_loop,
            daemon=True
        )
        self._status_thread.start()

        self._state = SupervisorState.RUNNING
        logger.info("Payload supervisor started")

    def stop(self):
        """Stop the supervisor and all managed processes."""
        logger.info("Stopping payload supervisor")
        self._state = SupervisorState.SHUTTING_DOWN
        self._running = False

        # Stop all managed processes
        self._stop_all_processes()

        # Stop health interface
        if self._health:
            self._health.stop()

        # Wait for threads
        if self._monitor_thread and self._monitor_thread.is_alive():
            self._monitor_thread.join(timeout=5.0)
        if self._status_thread and self._status_thread.is_alive():
            self._status_thread.join(timeout=5.0)

        self._state = SupervisorState.STOPPED
        logger.info("Payload supervisor stopped")

    def register_process(
        self,
        name: str,
        command: List[str],
        restart_on_failure: bool = True,
        max_restarts: int = 3
    ):
        """
        Register a process to be managed.

        Args:
            name: Process identifier
            command: Command to execute
            restart_on_failure: Auto-restart on failure
            max_restarts: Maximum restart attempts
        """
        with self._process_lock:
            self._processes[name] = ProcessInfo(
                name=name,
                command=command,
                restart_on_failure=restart_on_failure,
                max_restarts=max_restarts
            )
        logger.info(f"Registered process: {name}")

    def start_process(self, name: str) -> bool:
        """
        Start a registered process.

        Args:
            name: Process identifier

        Returns:
            True if started successfully
        """
        with self._process_lock:
            if name not in self._processes:
                logger.error(f"Process not registered: {name}")
                return False

            proc_info = self._processes[name]

            try:
                proc_info.process = subprocess.Popen(
                    proc_info.command,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE
                )
                proc_info.last_start_time = time.time()
                logger.info(f"Started process: {name} (PID: {proc_info.process.pid})")
                return True

            except Exception as e:
                logger.error(f"Failed to start process {name}: {e}")
                return False

    def stop_process(self, name: str, timeout: float = 5.0) -> bool:
        """
        Stop a managed process.

        Args:
            name: Process identifier
            timeout: Graceful shutdown timeout

        Returns:
            True if stopped successfully
        """
        with self._process_lock:
            if name not in self._processes:
                return False

            proc_info = self._processes[name]
            if proc_info.process is None:
                return True

            try:
                # Try graceful termination first
                proc_info.process.terminate()
                try:
                    proc_info.process.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    # Force kill if not responding
                    proc_info.process.kill()
                    proc_info.process.wait()

                proc_info.process = None
                logger.info(f"Stopped process: {name}")
                return True

            except Exception as e:
                logger.error(f"Error stopping process {name}: {e}")
                return False

    def get_state(self) -> SupervisorState:
        """Get current supervisor state."""
        return self._state

    def get_power_state(self) -> PowerState:
        """Get current power state from OBC."""
        return self._power_state

    def get_metrics(self) -> SystemMetrics:
        """Get current system metrics."""
        with self._metrics_lock:
            return self._metrics

    def is_healthy(self) -> bool:
        """Check if supervisor is healthy."""
        if self._state not in (SupervisorState.RUNNING, SupervisorState.THROTTLING):
            return False
        if not self._health or not self._health.is_healthy():
            return False
        return True

    def _signal_handler(self, signum, frame):
        """Handle termination signals."""
        logger.info(f"Received signal {signum}")
        self._shutdown_requested = True
        self.stop()

    def _handle_health_command(self, data: HealthData):
        """Handle command from OBC."""
        logger.debug(f"Received command: {data.command}")

        if data.command == HealthCommand.HEARTBEAT:
            # Update power state
            self._power_state = PowerState(data.power_state)
            self._handle_power_state_change()

        elif data.command == HealthCommand.STATUS_REQUEST:
            # Send full status
            self._send_status()

        elif data.command == HealthCommand.SHUTDOWN:
            logger.info("Shutdown command received from OBC")
            self.stop()

        elif data.command == HealthCommand.RESTART:
            logger.info("Restart command received from OBC")
            self._restart_all_processes()

        elif data.command == HealthCommand.POWER_STATE:
            self._power_state = PowerState(data.power_state)
            self._handle_power_state_change()

        elif data.command == HealthCommand.CONFIG_UPDATE:
            self._handle_config_update(data.data)

    def _handle_obc_disconnect(self):
        """Handle OBC disconnection."""
        logger.warning("Lost connection to OBC")
        # Enter safe state - reduce activity
        self._state = SupervisorState.THROTTLING

    def _handle_power_state_change(self):
        """Handle power state change from OBC."""
        if self._power_state == PowerState.SAFE:
            logger.info("Entering SAFE power state - reducing activity")
            self._state = SupervisorState.THROTTLING
            # Could stop non-essential processes here

        elif self._power_state == PowerState.IDLE:
            logger.info("Entering IDLE power state")
            if self._state == SupervisorState.THROTTLING:
                self._state = SupervisorState.RUNNING

        elif self._power_state == PowerState.ACTIVE:
            logger.info("Entering ACTIVE power state")
            self._state = SupervisorState.RUNNING

    def _handle_config_update(self, config_data: Optional[Dict[str, Any]]):
        """Handle configuration update from OBC."""
        if not config_data:
            return

        logger.info(f"Received config update: {config_data}")

        # Update applicable configuration
        if "thermal_throttle_temp" in config_data:
            self.config.thermal_throttle_temp_c = config_data["thermal_throttle_temp"]
        if "status_interval" in config_data:
            self.config.status_interval_s = config_data["status_interval"]

    def _monitor_loop(self):
        """Monitor system resources and processes."""
        while self._running:
            try:
                # Update metrics
                self._update_metrics()

                # Check thermal state
                self._check_thermal()

                # Check process health
                self._check_processes()

                time.sleep(1.0)

            except Exception as e:
                logger.error(f"Error in monitor loop: {e}")

    def _status_loop(self):
        """Periodic status reporting loop."""
        while self._running:
            try:
                if self._health and self._health.is_connected():
                    self._send_status()

                time.sleep(self.config.status_interval_s)

            except Exception as e:
                logger.error(f"Error in status loop: {e}")

    def _update_metrics(self):
        """Update system metrics."""
        with self._metrics_lock:
            self._metrics.cpu_percent = psutil.cpu_percent()
            self._metrics.memory_percent = psutil.virtual_memory().percent
            self._metrics.uptime_s = time.time() - self._start_time

            # GPU metrics (Jetson-specific)
            try:
                self._metrics.temperature_c = self._read_temperature()
                self._metrics.gpu_utilization = self._read_gpu_utilization()
                self._metrics.power_draw_w = self._read_power()
            except Exception:
                pass  # GPU metrics may not be available in simulation

    def _read_temperature(self) -> float:
        """Read GPU/SoC temperature."""
        try:
            # Jetson thermal zone
            with open("/sys/devices/virtual/thermal/thermal_zone0/temp") as f:
                return float(f.read()) / 1000.0
        except:
            return 0.0

    def _read_gpu_utilization(self) -> float:
        """Read GPU utilization."""
        try:
            # Jetson GPU load
            with open("/sys/devices/gpu.0/load") as f:
                return float(f.read()) / 10.0
        except:
            return 0.0

    def _read_power(self) -> float:
        """Read power consumption."""
        try:
            # Jetson power monitor
            with open("/sys/bus/i2c/drivers/ina3221x/1-0040/iio:device0/in_power0_input") as f:
                return float(f.read()) / 1000.0
        except:
            return 0.0

    def _check_thermal(self):
        """Check thermal state and throttle if necessary."""
        with self._metrics_lock:
            temp = self._metrics.temperature_c

        if temp >= self.config.thermal_shutdown_temp_c:
            logger.critical(f"Thermal shutdown at {temp}°C")
            self.stop()

        elif temp >= self.config.thermal_throttle_temp_c:
            if self._state != SupervisorState.THROTTLING:
                logger.warning(f"Thermal throttling at {temp}°C")
                self._state = SupervisorState.THROTTLING
        else:
            if self._state == SupervisorState.THROTTLING:
                logger.info("Thermal throttling ended")
                self._state = SupervisorState.RUNNING

    def _check_processes(self):
        """Check managed process health."""
        with self._process_lock:
            for name, proc_info in self._processes.items():
                if proc_info.process is None:
                    continue

                # Check if process is still running
                if proc_info.process.poll() is not None:
                    exit_code = proc_info.process.returncode
                    logger.warning(f"Process {name} exited with code {exit_code}")
                    proc_info.process = None

                    # Restart if configured
                    if proc_info.restart_on_failure:
                        if proc_info.restart_count < proc_info.max_restarts:
                            proc_info.restart_count += 1
                            logger.info(f"Restarting {name} ({proc_info.restart_count}/{proc_info.max_restarts})")
                            self.start_process(name)
                        else:
                            logger.error(f"Process {name} exceeded max restarts")

    def _send_status(self):
        """Send status to OBC."""
        if not self._health:
            return

        with self._metrics_lock:
            status_data = {
                "state": self._state.name,
                "cpu_percent": self._metrics.cpu_percent,
                "memory_percent": self._metrics.memory_percent,
                "gpu_utilization": self._metrics.gpu_utilization,
                "temperature_c": self._metrics.temperature_c,
                "power_w": self._metrics.power_draw_w,
                "uptime_s": self._metrics.uptime_s,
                "processes": self._get_process_status(),
            }

        # Determine overall health status
        if self._state == SupervisorState.RUNNING:
            status = HealthStatus.OK
        elif self._state == SupervisorState.THROTTLING:
            status = HealthStatus.WARNING
        else:
            status = HealthStatus.ERROR

        self._health.send_response(status, status_data)

    def _get_process_status(self) -> Dict[str, Any]:
        """Get status of all managed processes."""
        with self._process_lock:
            return {
                name: {
                    "running": proc.process is not None and proc.process.poll() is None,
                    "pid": proc.process.pid if proc.process else None,
                    "restarts": proc.restart_count,
                }
                for name, proc in self._processes.items()
            }

    def _stop_all_processes(self):
        """Stop all managed processes."""
        with self._process_lock:
            for name in list(self._processes.keys()):
                self.stop_process(name)

    def _restart_all_processes(self):
        """Restart all managed processes."""
        with self._process_lock:
            for name in list(self._processes.keys()):
                self.stop_process(name)
                time.sleep(0.5)
                self.start_process(name)


def main():
    """Main entry point."""
    supervisor = PayloadSupervisor()

    # Register inference process
    supervisor.register_process(
        name="llama_runner",
        command=["python3", "llama_runner.py"],
        restart_on_failure=True,
        max_restarts=3
    )

    try:
        supervisor.start()

        # Start managed processes
        supervisor.start_process("llama_runner")

        # Main loop
        while supervisor.is_healthy():
            time.sleep(1.0)

    except KeyboardInterrupt:
        pass
    finally:
        supervisor.stop()


if __name__ == "__main__":
    main()
