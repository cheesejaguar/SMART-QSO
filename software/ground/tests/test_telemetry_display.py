"""
Unit tests for Telemetry Display Module

Tests real-time telemetry display, limit monitoring, alarms,
and data export for SMART-QSO ground station.

Author: SMART-QSO Team
Date: 2026-01-02
Version: 1.0
"""

import unittest
from unittest.mock import Mock, patch, MagicMock
import os
import tempfile
import json
import csv
import time
from datetime import datetime

import sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from telemetry_display import (
    TelemetryDisplay, LimitStatus, AlarmPriority,
    TelemetryLimit, Alarm, TelemetryPoint, create_display
)


class TestLimitStatus(unittest.TestCase):
    """Test LimitStatus enumeration."""

    def test_members(self):
        """The five limit states, each distinct."""
        self.assertEqual([s.name for s in LimitStatus],
                         ["NOMINAL", "WARNING_LOW", "WARNING_HIGH",
                          "CRITICAL_LOW", "CRITICAL_HIGH"])
        self.assertEqual(len({int(s) for s in LimitStatus}), 5)


class TestAlarmPriority(unittest.TestCase):
    """Test AlarmPriority enumeration."""

    def test_info_lowest(self):
        """Test INFO is lowest priority."""
        self.assertEqual(AlarmPriority.INFO, 0)

    def test_warning_middle(self):
        """Test WARNING is middle priority."""
        self.assertEqual(AlarmPriority.WARNING, 1)

    def test_critical_highest(self):
        """Test CRITICAL is highest priority."""
        self.assertEqual(AlarmPriority.CRITICAL, 2)


class TestTelemetryLimit(unittest.TestCase):
    """Test TelemetryLimit dataclass."""

    def test_creation(self):
        """Test basic limit creation."""
        limit = TelemetryLimit(
            critical_low=3.0,
            warning_low=3.2,
            warning_high=4.2,
            critical_high=4.35
        )

        self.assertEqual(limit.critical_low, 3.0)
        self.assertEqual(limit.warning_low, 3.2)
        self.assertEqual(limit.warning_high, 4.2)
        self.assertEqual(limit.critical_high, 4.35)

    def test_defaults_none(self):
        """Test default values are None."""
        limit = TelemetryLimit()

        self.assertIsNone(limit.critical_low)
        self.assertIsNone(limit.warning_low)
        self.assertIsNone(limit.warning_high)
        self.assertIsNone(limit.critical_high)

    def test_partial_limits(self):
        """Test partial limit definition."""
        limit = TelemetryLimit(warning_high=80, critical_high=90)

        self.assertIsNone(limit.critical_low)
        self.assertIsNone(limit.warning_low)
        self.assertEqual(limit.warning_high, 80)
        self.assertEqual(limit.critical_high, 90)


class TestAlarm(unittest.TestCase):
    """Test Alarm dataclass."""

    def test_creation(self):
        """Test alarm creation."""
        alarm = Alarm(
            parameter="battery_voltage",
            value=2.9,
            limit_status=LimitStatus.CRITICAL_LOW,
            priority=AlarmPriority.CRITICAL,
            timestamp=1234567890.0,
            message="Battery critically low"
        )

        self.assertEqual(alarm.parameter, "battery_voltage")
        self.assertEqual(alarm.value, 2.9)
        self.assertEqual(alarm.priority, AlarmPriority.CRITICAL)
        self.assertFalse(alarm.acknowledged)

    def test_default_acknowledged_false(self):
        """Test alarm defaults to unacknowledged."""
        alarm = Alarm(
            parameter="test",
            value=0,
            limit_status=LimitStatus.NOMINAL,
            priority=AlarmPriority.INFO,
            timestamp=0
        )

        self.assertFalse(alarm.acknowledged)


class TestTelemetryDisplay(unittest.TestCase):
    """Test TelemetryDisplay class."""

    def setUp(self):
        """Set up test fixtures."""
        self.display = TelemetryDisplay()

    def test_init(self):
        """Test display initialization."""
        self.assertEqual(self.display._update_count, 0)
        self.assertEqual(len(self.display._points), 0)

    def test_init_custom_history_size(self):
        """Test custom history size."""
        display = TelemetryDisplay(history_size=500)
        self.assertEqual(display._history_size, 500)


class TestTelemetryDisplayUpdate(unittest.TestCase):
    """Test telemetry update functionality."""

    def setUp(self):
        self.display = TelemetryDisplay()

    def test_update_creates_point(self):
        """Test updating creates new telemetry point."""
        self.display.update("voltage", 3.8, "V")

        self.assertIn("voltage", self.display._points)

    def test_update_value(self):
        """Test updating value."""
        self.display.update("voltage", 3.8, "V")
        self.display.update("voltage", 3.9, "V")

        self.assertEqual(self.display.get_value("voltage"), 3.9)

    def test_update_with_timestamp(self):
        """Test updating with explicit timestamp."""
        ts = 1234567890.0
        self.display.update("temp", 25, "C", timestamp=ts)

        point = self.display.get_point("temp")
        self.assertEqual(point.timestamp, ts)

    def test_update_increments_counter(self):
        """Test update increments counter."""
        self.display.update("a", 1)
        self.display.update("b", 2)
        self.display.update("a", 3)

        self.assertEqual(self.display._update_count, 3)


class TestTelemetryDisplayBatchUpdate(unittest.TestCase):
    """Test batch update functionality."""

    def setUp(self):
        self.display = TelemetryDisplay()

    def test_update_batch(self):
        """Test batch update."""
        values = {
            "voltage": (3.8, "V"),
            "current": (1.2, "A"),
            "temp": (25, "C")
        }

        self.display.update_batch(values)

        self.assertEqual(self.display.get_value("voltage"), 3.8)
        self.assertEqual(self.display.get_value("current"), 1.2)
        self.assertEqual(self.display.get_value("temp"), 25)


class TestTelemetryDisplayGetters(unittest.TestCase):
    """Test getter methods."""

    def setUp(self):
        self.display = TelemetryDisplay()
        self.display.update("voltage", 3.8, "V")
        self.display.update("temp", 25, "C")

    def test_get_value(self):
        """Test getting single value."""
        self.assertEqual(self.display.get_value("voltage"), 3.8)

    def test_get_value_nonexistent(self):
        """Test getting nonexistent value."""
        self.assertIsNone(self.display.get_value("nonexistent"))

    def test_get_point(self):
        """Test getting full point data."""
        point = self.display.get_point("voltage")

        self.assertIsNotNone(point)
        self.assertEqual(point.name, "voltage")
        self.assertEqual(point.value, 3.8)
        self.assertEqual(point.unit, "V")

    def test_get_all_values(self):
        """Test getting all values."""
        values = self.display.get_all_values()

        self.assertEqual(values["voltage"], 3.8)
        self.assertEqual(values["temp"], 25)


class TestTelemetryDisplayHistory(unittest.TestCase):
    """Test history functionality."""

    def setUp(self):
        self.display = TelemetryDisplay(history_size=100)

    def test_history_recorded(self):
        """Test values are recorded in history."""
        self.display.update("voltage", 3.8)
        self.display.update("voltage", 3.9)
        self.display.update("voltage", 4.0)

        history = self.display.get_history("voltage")

        self.assertEqual(len(history), 3)

    def test_history_order(self):
        """Test history maintains order."""
        self.display.update("voltage", 1.0)
        self.display.update("voltage", 2.0)
        self.display.update("voltage", 3.0)

        history = self.display.get_history("voltage")

        self.assertEqual(history[0][1], 1.0)
        self.assertEqual(history[2][1], 3.0)

    def test_history_limit(self):
        """Test history respects size limit."""
        display = TelemetryDisplay(history_size=5)

        for i in range(10):
            display.update("voltage", float(i))

        history = display.get_history("voltage")

        self.assertEqual(len(history), 5)

    def test_get_history_count(self):
        """Test getting limited history count."""
        for i in range(10):
            self.display.update("voltage", float(i))

        history = self.display.get_history("voltage", count=3)

        self.assertEqual(len(history), 3)

    def test_get_history_nonexistent(self):
        """Test getting history for nonexistent parameter."""
        history = self.display.get_history("nonexistent")

        self.assertEqual(history, [])


class TestTelemetryDisplayLimits(unittest.TestCase):
    """Test limit checking functionality."""

    def setUp(self):
        self.display = TelemetryDisplay()
        self.limits = TelemetryLimit(
            critical_low=3.0,
            warning_low=3.2,
            warning_high=4.2,
            critical_high=4.35
        )
        self.display.set_limits("voltage", self.limits)

    def test_set_limits(self):
        """Test setting limits."""
        self.assertIn("voltage", self.display._limits)

    def test_limits_applied_to_point(self):
        """Test limits applied when point created."""
        self.display.update("voltage", 3.8)

        point = self.display.get_point("voltage")
        self.assertEqual(point.limits.warning_low, 3.2)

    def test_no_alarm_nominal(self):
        """Test no alarm for nominal value."""
        self.display.update("voltage", 3.8)

        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 0)

    def test_alarm_warning_low(self):
        """Test alarm raised for warning low."""
        self.display.update("voltage", 3.15)

        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)
        self.assertEqual(alarms[0].priority, AlarmPriority.WARNING)

    def test_alarm_critical_low(self):
        """Test alarm raised for critical low."""
        self.display.update("voltage", 2.9)

        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)
        self.assertEqual(alarms[0].priority, AlarmPriority.CRITICAL)

    def test_alarm_warning_high(self):
        """Test alarm raised for warning high."""
        self.display.update("voltage", 4.25)

        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)
        self.assertEqual(alarms[0].limit_status, LimitStatus.WARNING_HIGH)

    def test_alarm_critical_high(self):
        """Test alarm raised for critical high."""
        self.display.update("voltage", 4.4)

        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)
        self.assertEqual(alarms[0].limit_status, LimitStatus.CRITICAL_HIGH)


class TestTelemetryDisplayAlarms(unittest.TestCase):
    """Test alarm management functionality."""

    def setUp(self):
        self.display = TelemetryDisplay()
        limits = TelemetryLimit(critical_low=10)
        self.display.set_limits("temp", limits)

    def test_acknowledge_alarm(self):
        """Test acknowledging an alarm."""
        self.display.update("temp", 5)  # Trigger alarm

        result = self.display.acknowledge_alarm("temp")

        self.assertTrue(result)
        alarms = self.display.get_alarms(active_only=True)
        self.assertEqual(len(alarms), 0)

    def test_acknowledge_nonexistent_alarm(self):
        """Test acknowledging nonexistent alarm."""
        result = self.display.acknowledge_alarm("nonexistent")
        self.assertFalse(result)

    def test_clear_alarms(self):
        """Test clearing acknowledged alarms."""
        self.display.update("temp", 5)
        self.display.acknowledge_alarm("temp")

        self.display.clear_alarms()

        alarms = self.display.get_alarms(active_only=False)
        self.assertEqual(len(alarms), 0)

    def test_get_alarms_all(self):
        """Test getting all alarms including acknowledged."""
        self.display.update("temp", 5)
        self.display.acknowledge_alarm("temp")

        active = self.display.get_alarms(active_only=True)
        all_alarms = self.display.get_alarms(active_only=False)

        self.assertEqual(len(active), 0)
        self.assertEqual(len(all_alarms), 1)


class TestTelemetryDisplayCallbacks(unittest.TestCase):
    """Test callback functionality."""

    def setUp(self):
        self.display = TelemetryDisplay()

    def test_register_update_callback(self):
        """Test registering update callback."""
        callback = Mock()
        self.display.register_update_callback(callback)

        self.display.update("voltage", 3.8)

        callback.assert_called_once_with("voltage", 3.8)

    def test_register_alarm_callback(self):
        """Test registering alarm callback."""
        callback = Mock()
        self.display.register_alarm_callback(callback)

        limits = TelemetryLimit(critical_low=10)
        self.display.set_limits("temp", limits)
        self.display.update("temp", 5)

        callback.assert_called_once()
        alarm = callback.call_args[0][0]
        self.assertEqual(alarm.parameter, "temp")
        self.assertEqual(alarm.value, 5)
        self.assertEqual(alarm.limit_status, LimitStatus.CRITICAL_LOW)
        self.assertEqual(alarm.priority, AlarmPriority.CRITICAL)
        self.assertEqual(alarm.message, "temp critically low: 5 < 10")

    def test_callback_exception_handled(self):
        """Test callback exceptions are handled."""
        callback = Mock(side_effect=Exception("Test error"))
        self.display.register_update_callback(callback)

        self.display.update("voltage", 3.8)  # does not raise

        callback.assert_called_once_with("voltage", 3.8)
        self.assertEqual(self.display.get_value("voltage"), 3.8)


class TestTelemetryDisplayExport(unittest.TestCase):
    """Test data export functionality."""

    T0 = 1_700_000_000.0

    def setUp(self):
        self.display = TelemetryDisplay()
        self.display.update("voltage", 3.8, "V", timestamp=self.T0)
        self.display.update("voltage", 3.9, "V", timestamp=self.T0 + 10)
        self.display.update("temp", 25, "C", timestamp=self.T0)
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.tmp = tmp.name

    def _rows(self, filename):
        with open(filename, newline='') as f:
            return list(csv.reader(f))

    def test_export_csv(self):
        """CSV export: one row per timestamp, blank where a parameter has no sample."""
        filename = os.path.join(self.tmp, "out.csv")
        self.display.export_csv(filename)

        rows = self._rows(filename)
        self.assertEqual(rows[0], ["timestamp", "datetime", "voltage", "temp"])
        self.assertEqual(rows[1], [str(self.T0), datetime.fromtimestamp(self.T0).isoformat(),
                                   "3.8", "25"])
        self.assertEqual(rows[2], [str(self.T0 + 10),
                                   datetime.fromtimestamp(self.T0 + 10).isoformat(), "3.9", ""])
        self.assertEqual(len(rows), 3)

    def test_export_csv_specific_params(self):
        """CSV export of chosen parameters only."""
        filename = os.path.join(self.tmp, "out.csv")
        self.display.export_csv(filename, parameters=["voltage"])

        rows = self._rows(filename)
        self.assertEqual(rows[0], ["timestamp", "datetime", "voltage"])
        self.assertEqual([r[2] for r in rows[1:]], ["3.8", "3.9"])

    def test_export_json(self):
        """JSON export: every parameter with its unit, current value and history."""
        filename = os.path.join(self.tmp, "out.json")
        before = time.time()
        self.display.export_json(filename)

        with open(filename) as f:
            data = json.load(f)

        self.assertGreaterEqual(data["export_time"], before)
        self.assertEqual(set(data["parameters"]), {"voltage", "temp"})

    def test_export_json_structure(self):
        """JSON export structure for one parameter."""
        filename = os.path.join(self.tmp, "out.json")
        self.display.export_json(filename, parameters=["voltage"])

        with open(filename) as f:
            data = json.load(f)

        self.assertEqual(data["parameters"], {"voltage": {
            "unit": "V",
            "current_value": 3.9,
            "current_timestamp": self.T0 + 10,
            "history": [{"timestamp": self.T0, "value": 3.8},
                        {"timestamp": self.T0 + 10, "value": 3.9}],
        }})


class TestTelemetryDisplayStatistics(unittest.TestCase):
    """Test statistics functionality."""

    def setUp(self):
        self.display = TelemetryDisplay()

    def test_get_statistics(self):
        """Test getting statistics."""
        self.display.update("a", 1)
        self.display.update("b", 2)

        stats = self.display.get_statistics()

        self.assertEqual(stats["update_count"], 2)
        self.assertEqual(stats["parameter_count"], 2)
        self.assertIn("uptime_s", stats)
        self.assertIn("active_alarms", stats)


class TestCreateDisplay(unittest.TestCase):
    """Test create_display convenience function."""

    def test_create_display(self):
        """Test create_display returns TelemetryDisplay."""
        display = create_display()

        self.assertIsInstance(display, TelemetryDisplay)


class TestDefaultLimits(unittest.TestCase):
    """Test default limits are applied."""

    def setUp(self):
        self.display = TelemetryDisplay()

    def test_battery_voltage_default_limits(self):
        """Test battery_voltage has default limits."""
        self.display.update("battery_voltage", 2.5)

        # Should trigger critical low alarm
        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)

    def test_battery_soc_default_limits(self):
        """Test battery_soc has default limits."""
        self.display.update("battery_soc", 5)

        # Should trigger critical low alarm
        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)

    def test_temperature_default_limits(self):
        """Test temperature has default limits."""
        self.display.update("temperature", 75)

        # Should trigger critical high alarm
        alarms = self.display.get_alarms()
        self.assertEqual(len(alarms), 1)


if __name__ == '__main__':
    unittest.main()
