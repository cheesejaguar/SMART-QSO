"""
Telemetry Display Module for SMART-QSO Ground Station

This module provides real-time telemetry display, historical trending,
limit monitoring, and data export capabilities.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import time
import threading
import json
import csv
import logging
from typing import Optional, Dict, Any, List, Callable, Tuple
from dataclasses import dataclass, field
from enum import IntEnum, auto
from collections import deque
from datetime import datetime

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class LimitStatus(IntEnum):
    """Limit check status."""
    NOMINAL = auto()
    WARNING_LOW = auto()
    WARNING_HIGH = auto()
    CRITICAL_LOW = auto()
    CRITICAL_HIGH = auto()


class AlarmPriority(IntEnum):
    """Alarm priority levels."""
    INFO = 0
    WARNING = 1
    CRITICAL = 2


@dataclass
class TelemetryLimit:
    """Limit definition for a telemetry point."""
    critical_low: Optional[float] = None
    warning_low: Optional[float] = None
    warning_high: Optional[float] = None
    critical_high: Optional[float] = None


@dataclass
class Alarm:
    """Active alarm."""
    parameter: str
    value: float
    limit_status: LimitStatus
    priority: AlarmPriority
    timestamp: float
    acknowledged: bool = False
    message: str = ""


@dataclass
class TelemetryPoint:
    """Telemetry point with history."""
    name: str
    value: float
    unit: str
    timestamp: float
    limits: Optional[TelemetryLimit] = None
    history: deque = field(default_factory=lambda: deque(maxlen=1000))


class TelemetryDisplay:
    """
    Real-time telemetry display and monitoring.

    Provides:
    - Real-time telemetry value display
    - Historical trending
    - Limit monitoring and alarms
    - Data export (CSV/JSON)
    """

    def __init__(self, history_size: int = 1000):
        """
        Initialize telemetry display.

        Args:
            history_size: Number of historical values to keep per point
        """
        self._history_size = history_size
        self._points: Dict[str, TelemetryPoint] = {}
        self._limits: Dict[str, TelemetryLimit] = {}
        self._alarms: List[Alarm] = []
        self._lock = threading.Lock()

        # Callbacks
        self._update_callback: Optional[Callable[[str, float], None]] = None
        self._alarm_callback: Optional[Callable[[Alarm], None]] = None

        # Statistics
        self._update_count = 0
        self._start_time = time.time()

        # Default limits for common parameters
        self._default_limits = {
            "battery_voltage": TelemetryLimit(
                critical_low=3.0, warning_low=3.2,
                warning_high=4.2, critical_high=4.35
            ),
            "battery_soc": TelemetryLimit(
                critical_low=10, warning_low=20,
                warning_high=None, critical_high=None
            ),
            "temperature": TelemetryLimit(
                critical_low=-30, warning_low=-20,
                warning_high=60, critical_high=70
            ),
        }

    def update(self, name: str, value: float, unit: str = "", timestamp: Optional[float] = None):
        """
        Update a telemetry point.

        Args:
            name: Parameter name
            value: Current value
            unit: Unit string
            timestamp: Value timestamp (default: now)
        """
        if timestamp is None:
            timestamp = time.time()

        with self._lock:
            if name not in self._points:
                limits = self._limits.get(name) or self._default_limits.get(name)
                self._points[name] = TelemetryPoint(
                    name=name,
                    value=value,
                    unit=unit,
                    timestamp=timestamp,
                    limits=limits,
                    history=deque(maxlen=self._history_size)
                )
            else:
                self._points[name].value = value
                self._points[name].timestamp = timestamp
                if unit:
                    self._points[name].unit = unit

            # Add to history
            self._points[name].history.append((timestamp, value))

            self._update_count += 1

        # Check limits
        self._check_limits(name, value)

        # Notify callback
        if self._update_callback:
            try:
                self._update_callback(name, value)
            except Exception as e:
                logger.error(f"Update callback error: {e}")

    def update_batch(self, values: Dict[str, Tuple[float, str]]):
        """
        Update multiple telemetry points.

        Args:
            values: Dict of {name: (value, unit)}
        """
        timestamp = time.time()
        for name, (value, unit) in values.items():
            self.update(name, value, unit, timestamp)

    def get_value(self, name: str) -> Optional[float]:
        """Get current value for a parameter."""
        with self._lock:
            if name in self._points:
                return self._points[name].value
        return None

    def get_point(self, name: str) -> Optional[TelemetryPoint]:
        """Get full telemetry point data."""
        with self._lock:
            if name in self._points:
                return self._points[name]
        return None

    def get_all_values(self) -> Dict[str, float]:
        """Get all current values."""
        with self._lock:
            return {name: point.value for name, point in self._points.items()}

    def get_history(self, name: str, count: Optional[int] = None) -> List[Tuple[float, float]]:
        """
        Get historical values for a parameter.

        Args:
            name: Parameter name
            count: Number of recent values (default: all)

        Returns:
            List of (timestamp, value) tuples
        """
        with self._lock:
            if name not in self._points:
                return []

            history = list(self._points[name].history)
            if count is not None:
                history = history[-count:]
            return history

    def set_limits(self, name: str, limits: TelemetryLimit):
        """Set limits for a parameter."""
        with self._lock:
            self._limits[name] = limits
            if name in self._points:
                self._points[name].limits = limits

    def get_alarms(self, active_only: bool = True) -> List[Alarm]:
        """Get current alarms."""
        with self._lock:
            if active_only:
                return [a for a in self._alarms if not a.acknowledged]
            return list(self._alarms)

    def acknowledge_alarm(self, parameter: str) -> bool:
        """Acknowledge an alarm."""
        with self._lock:
            for alarm in self._alarms:
                if alarm.parameter == parameter and not alarm.acknowledged:
                    alarm.acknowledged = True
                    return True
        return False

    def clear_alarms(self):
        """Clear all acknowledged alarms."""
        with self._lock:
            self._alarms = [a for a in self._alarms if not a.acknowledged]

    def register_update_callback(self, callback: Callable[[str, float], None]):
        """Register callback for telemetry updates."""
        self._update_callback = callback

    def register_alarm_callback(self, callback: Callable[[Alarm], None]):
        """Register callback for alarms."""
        self._alarm_callback = callback

    def export_csv(self, filename: str, parameters: Optional[List[str]] = None):
        """
        Export telemetry history to CSV.

        Args:
            filename: Output file path
            parameters: List of parameters to export (default: all)
        """
        with self._lock:
            if parameters is None:
                parameters = list(self._points.keys())

            # Collect all timestamps
            all_times = set()
            for name in parameters:
                if name in self._points:
                    for ts, _ in self._points[name].history:
                        all_times.add(ts)

            sorted_times = sorted(all_times)

            with open(filename, 'w', newline='') as f:
                writer = csv.writer(f)

                # Header
                writer.writerow(['timestamp', 'datetime'] + parameters)

                # Data rows
                for ts in sorted_times:
                    row = [ts, datetime.fromtimestamp(ts).isoformat()]
                    for name in parameters:
                        if name in self._points:
                            # Find value at this timestamp
                            value = None
                            for t, v in self._points[name].history:
                                if t == ts:
                                    value = v
                                    break
                            row.append(value if value is not None else '')
                        else:
                            row.append('')
                    writer.writerow(row)

        logger.info(f"Exported telemetry to {filename}")

    def export_json(self, filename: str, parameters: Optional[List[str]] = None):
        """
        Export telemetry history to JSON.

        Args:
            filename: Output file path
            parameters: List of parameters to export (default: all)
        """
        with self._lock:
            if parameters is None:
                parameters = list(self._points.keys())

            data = {
                "export_time": time.time(),
                "parameters": {}
            }

            for name in parameters:
                if name in self._points:
                    point = self._points[name]
                    data["parameters"][name] = {
                        "unit": point.unit,
                        "current_value": point.value,
                        "current_timestamp": point.timestamp,
                        "history": [
                            {"timestamp": ts, "value": v}
                            for ts, v in point.history
                        ]
                    }

            with open(filename, 'w') as f:
                json.dump(data, f, indent=2)

        logger.info(f"Exported telemetry to {filename}")

    def get_statistics(self) -> Dict[str, Any]:
        """Get display statistics."""
        with self._lock:
            return {
                "update_count": self._update_count,
                "parameter_count": len(self._points),
                "active_alarms": len([a for a in self._alarms if not a.acknowledged]),
                "uptime_s": time.time() - self._start_time,
            }

    def _check_limits(self, name: str, value: float):
        """Check value against limits and raise alarms if needed."""
        with self._lock:
            if name not in self._points:
                return

            limits = self._points[name].limits
            if limits is None:
                return

            status = LimitStatus.NOMINAL
            priority = AlarmPriority.INFO
            message = ""

            # Check limits
            if limits.critical_low is not None and value < limits.critical_low:
                status = LimitStatus.CRITICAL_LOW
                priority = AlarmPriority.CRITICAL
                message = f"{name} critically low: {value} < {limits.critical_low}"
            elif limits.warning_low is not None and value < limits.warning_low:
                status = LimitStatus.WARNING_LOW
                priority = AlarmPriority.WARNING
                message = f"{name} low: {value} < {limits.warning_low}"
            elif limits.critical_high is not None and value > limits.critical_high:
                status = LimitStatus.CRITICAL_HIGH
                priority = AlarmPriority.CRITICAL
                message = f"{name} critically high: {value} > {limits.critical_high}"
            elif limits.warning_high is not None and value > limits.warning_high:
                status = LimitStatus.WARNING_HIGH
                priority = AlarmPriority.WARNING
                message = f"{name} high: {value} > {limits.warning_high}"

            # Create alarm if not nominal
            if status != LimitStatus.NOMINAL:
                # Check if alarm already exists
                existing = None
                for alarm in self._alarms:
                    if alarm.parameter == name and not alarm.acknowledged:
                        existing = alarm
                        break

                if existing is None:
                    alarm = Alarm(
                        parameter=name,
                        value=value,
                        limit_status=status,
                        priority=priority,
                        timestamp=time.time(),
                        message=message
                    )
                    self._alarms.append(alarm)

                    # Notify callback
                    if self._alarm_callback:
                        try:
                            self._alarm_callback(alarm)
                        except Exception as e:
                            logger.error(f"Alarm callback error: {e}")

                    logger.warning(message)


# Convenience function
def create_display() -> TelemetryDisplay:
    """Create and return a new telemetry display instance."""
    return TelemetryDisplay()
