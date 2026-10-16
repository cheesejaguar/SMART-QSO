#!/usr/bin/env python3
"""
Parameter consistency check (FINDINGS_REGISTER VV-08).

The same parameter used to carry different values in different places:
beacon interval, SOC thresholds, watchdog timeout, contact timeout and
others. This check holds each key parameter once, with every place that
states it (code constants and the governing documents). It fails if any
place disagrees, or no longer states the value, so a change has to be made
everywhere at once. docs/BASELINE_PARAMETERS.md governs the values.

    python3 scripts/check_parameters.py        # exit 1 on any disagreement
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import List, Tuple

REPO = Path(__file__).resolve().parents[1]

FLIGHT = "software/flight/include/"


@dataclass(frozen=True)
class Parameter:
    name: str
    expected: str
    places: Tuple[Tuple[str, str], ...]   # (repository path, regex with one group)


PARAMETERS: Tuple[Parameter, ...] = (
    Parameter("beacon interval ACTIVE (s)", "60", (
        (FLIGHT + "beacon.h", r"BEACON_INTERVAL_ACTIVE_MS\s+(\d+)000\b"),
        ("docs/BEACON_SPEC.md", r"\| ACTIVE \| (\d+) s \("),
        ("docs/operations/MISSION_OPERATIONS_PLAN.md", r"\| ACTIVE \| (\d+) seconds \|"),
        ("docs/BASELINE_PARAMETERS.md", r"Beacon every (\d+) s in ACTIVE"),
        ("docs/requirements/MISSION_REQUIREMENTS.md", r"beacon at least every (\d+) seconds in ACTIVE"),
    )),
    Parameter("beacon interval IDLE (s)", "120", (
        (FLIGHT + "beacon.h", r"BEACON_INTERVAL_IDLE_MS\s+(\d+)000\b"),
        ("docs/BEACON_SPEC.md", r"\| IDLE \| (\d+) s \|"),
        ("docs/operations/MISSION_OPERATIONS_PLAN.md", r"\| IDLE \| (\d+) seconds \|"),
        ("docs/BASELINE_PARAMETERS.md", r"(\d+) s in IDLE"),
    )),
    Parameter("beacon interval SAFE (s)", "180", (
        (FLIGHT + "beacon.h", r"BEACON_INTERVAL_SAFE_MS\s+(\d+)000\b"),
        ("docs/BEACON_SPEC.md", r"\| SAFE \| (\d+) s \|"),
        ("docs/operations/MISSION_OPERATIONS_PLAN.md", r"\| SAFE \| (\d+) seconds \|"),
        ("docs/BASELINE_PARAMETERS.md", r"(\d+) s in SAFE"),
        ("docs/requirements/MISSION_REQUIREMENTS.md", r"at least every (\d+) seconds in IDLE and SAFE"),
    )),
    Parameter("SOC: ACTIVE allowed at (%)", "55", (
        (FLIGHT + "eps_control.h", r"EPS_SOC_PAYLOAD_THRESHOLD\s+0\.(\d\d)"),
        ("docs/BASELINE_PARAMETERS.md", r"ACTIVE allowed at ≥ (\d+) %"),
    )),
    Parameter("SOC: ACTIVE to IDLE below (%)", "40", (
        (FLIGHT + "eps_control.h", r"EPS_SOC_IDLE_THRESHOLD\s+0\.(\d\d)"),
        ("docs/BASELINE_PARAMETERS.md", r"ACTIVE → IDLE below (\d+) %"),
        ("docs/software/SOFTWARE_REQUIREMENTS.md", r"IDLE mode when SOC < (\d+)%"),
    )),
    Parameter("SOC: SAFE below (%)", "25", (
        (FLIGHT + "eps_control.h", r"EPS_SOC_SAFE_THRESHOLD\s+0\.(\d\d)"),
        ("docs/BASELINE_PARAMETERS.md", r"→ SAFE below (\d+) %"),
        ("docs/operations/MISSION_RULES.md", r"\| SOC < (\d+)% \| Transition to SAFE mode \|"),
    )),
    Parameter("SOC: EMERGENCY below (%)", "15", (
        (FLIGHT + "eps_control.h", r"EPS_SOC_EMERGENCY_THRESHOLD\s+0\.(\d\d)"),
        ("docs/BASELINE_PARAMETERS.md", r"EMERGENCY \(beacon shed\) below (\d+) %"),
        ("docs/operations/MISSION_RULES.md", r"\| SOC < (\d+)% \| Absolute minimum"),
    )),
    Parameter("hardware watchdog timeout (s)", "30", (
        (FLIGHT + "watchdog_mgr.h", r"WDT_HW_TIMEOUT_MS\s+(\d+)000U"),
        (FLIGHT + "hal/hal_wdt.h", r"HAL_WDT_DEFAULT_TIMEOUT_MS\s+(\d+)000\b"),
        ("hardware/obc/OBC_DESIGN.md", r"\| Timeout \| (\d+) seconds"),
        ("docs/test_procedures/TP-OBC-004_Watchdog.md", r"Measure timeout \| (\d+)s"),
    )),
    Parameter("loss-of-contact timeout at launch (days)", "30", (
        (FLIGHT + "contact_monitor.h", r"CONTACT_TIMEOUT_DEFAULT_DAYS\s+(\d+)U"),
        ("docs/BASELINE_PARAMETERS.md", r"\*\*N = (\d+) days at launch\*\*"),
        ("docs/interfaces/TELECOMMAND_AUTHENTICATION.md", r"\*\*(\d+) days at launch\*\*"),
    )),
    Parameter("post-deployment RF silence (min)", "45", (
        (FLIGHT + "rf_silence.h", r"RF_SILENCE_DURATION_S\s+\((\d+)U \* 60U\)"),
        ("docs/BASELINE_PARAMETERS.md", r"no RF before (\d+) min"),
    )),
    Parameter("beacon text maximum (characters)", "176", (
        (FLIGHT + "beacon_filter.h", r"BEACON_FILTER_MAX_LEN\s+(\d+)U"),
        (FLIGHT + "jetson_link.h", r"JETSON_TEXT_MAX\s+(\d+)U"),
        ("software/payload/content_filter.py", r"^MAX_LEN = (\d+)"),
        ("software/payload/obc_link.py", r"^TEXT_MAX = (\d+)"),
        ("docs/BEACON_SPEC.md", r"`<text>` is at most (\d+) characters"),
    )),
    Parameter("space station identifier", "KK6GDB-11", (
        (FLIGHT + "beacon.h", r'BEACON_CALLSIGN\s+"([A-Z0-9-]+)"'),
        ("software/payload/formatter.py", r'^STATION_CALL = "([A-Z0-9-]+)"'),
        ("docs/BEACON_SPEC.md", r"\| Source \| `([A-Z0-9-]+)`"),
        ("docs/BASELINE_PARAMETERS.md", r"identifies as \*\*([A-Z0-9-]+)\*\*"),
    )),
)


def check(root: Path = REPO) -> List[str]:
    """Return one line per disagreement; empty if consistent."""
    problems = []
    for param in PARAMETERS:
        for rel, pattern in param.places:
            path = root / rel
            if not path.exists():
                problems.append(f"{param.name}: {rel} not found")
                continue
            match = re.search(pattern, path.read_text(encoding="utf-8"), flags=re.M)
            if match is None:
                problems.append(f"{param.name}: value not found in {rel} (/{pattern}/)")
            elif match.group(1) != param.expected:
                problems.append(f"{param.name}: {rel} says {match.group(1)}, expected {param.expected}")
    return problems


def main() -> int:
    problems = check()
    for line in problems:
        print(line)
    if problems:
        print(f"{len(problems)} parameter disagreement(s); docs/BASELINE_PARAMETERS.md governs")
        return 1
    places = sum(len(p.places) for p in PARAMETERS)
    print(f"{len(PARAMETERS)} parameters consistent across {places} places")
    return 0


if __name__ == "__main__":
    sys.exit(main())
