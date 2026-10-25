#!/usr/bin/env python3
"""
Coverage gate for the safety-critical components (SDP §3.2; CR-2026-003).

    safety_coverage.py LCOV_INFO [--report FILE]

LCOV_INFO is the lcov export of a coverage build made with
-Zcoverage-options=condition (software/flight/scripts/rust_coverage.sh), so
its branch records hold every condition's outcomes. Each file in
SAFETY_CRITICAL must have every line run and every branch and condition
outcome taken, by the C suites and the Rust unit tests together.

A line no test can reach (defensive code: a fallback the types make
impossible) passes only if it, or the line above it, carries
`// COVERAGE-EXEMPT: <reason>`; review checks each one. A file's test
module (from its `#[cfg(test)]` line on) is not counted.

The firmware's watchdog driver (software/rust/firmware/src/wdt.rs) runs
only on the target; the hardware tests cover it (HIL-003, HIL-043).
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

RUST = Path(__file__).resolve().parents[2] / "rust"

# SDP §3.2, the safety-critical components (PI concurrence 2026-10-25)
SAFETY_CRITICAL: Tuple[str, ...] = (
    # EPS control, battery protection (SW-HAZ-001)
    "sqso-fsw/src/eps.rs", "sqso-ffi/src/eps_control.rs",
    # Watchdog supervision (SW-HAZ-004)
    "sqso-fsw/src/watchdog.rs", "sqso-ffi/src/watchdog_mgr.rs",
    # Deployment timer and sequence (SW-HAZ-007)
    "sqso-fsw/src/deployment.rs", "sqso-ffi/src/deployment.rs",
    # Safe-mode entry, fault response (SW-HAZ-008)
    "sqso-fsw/src/mode.rs", "sqso-ffi/src/state_machine.rs",
    "sqso-fsw/src/fault_log.rs", "sqso-ffi/src/fault_mgmt.rs",
    "sqso-fsw/src/assert_handler.rs", "sqso-ffi/src/assert_handler.rs",
    # Payload thermal and over-current protection
    "sqso-fsw/src/payload_guard.rs", "sqso-ffi/src/payload_guard.rs",
    # RF transmit inhibit (SH-008)
    "sqso-fsw/src/rf_control.rs", "sqso-ffi/src/rf_control.rs",
    "sqso-fsw/src/rf_silence.rs", "sqso-ffi/src/rf_silence.rs",
    "sqso-fsw/src/contact_monitor.rs", "sqso-ffi/src/contact_monitor.rs",
    "sqso-ffi/src/separation_switch.rs",
)

EXEMPT = re.compile(r"//\s*COVERAGE-EXEMPT:\s*\S")


class FileCoverage:
    def __init__(self) -> None:
        self.lines: Dict[int, int] = {}                       # line -> hits
        self.outcomes: Dict[int, List[bool]] = defaultdict(list)  # line -> taken?


def parse_lcov(path: str) -> Dict[str, FileCoverage]:
    files: Dict[str, FileCoverage] = {}
    cur: Optional[FileCoverage] = None
    for raw in Path(path).read_text().splitlines():
        if raw.startswith("SF:"):
            cur = files.setdefault(raw[3:], FileCoverage())
        elif cur is None:
            continue
        elif raw.startswith("DA:"):
            line, hits = raw[3:].split(",")[:2]
            cur.lines[int(line)] = cur.lines.get(int(line), 0) + int(hits)
        elif raw.startswith("BRDA:"):
            line, _block, _branch, taken = raw[5:].split(",")
            cur.outcomes[int(line)].append(taken not in ("-", "0"))
        elif raw == "end_of_record":
            cur = None
    return files


def test_module_start(source: List[str]) -> int:
    """First line (1-based) of the file's test module, or past the end"""
    for i, text in enumerate(source):
        if text.strip() == "#[cfg(test)]" and i + 1 < len(source) and re.match(r"\s*(#\[path|mod\s)", source[i + 1]):
            return i + 1
    return len(source) + 1


def exempt(source: List[str], line: int) -> bool:
    here = source[line - 1] if 0 < line <= len(source) else ""
    above = source[line - 2] if 1 < line <= len(source) + 1 else ""
    return bool(EXEMPT.search(here) or EXEMPT.search(above))


def check(rel: str, cov: Optional[FileCoverage]) -> Tuple[List[str], Tuple[int, int, int, int]]:
    """Problems in one file, and (lines run, lines, outcomes taken, outcomes)"""
    source = (RUST / rel).read_text().splitlines()
    if cov is None:
        return [f"{rel}: not in the coverage data"], (0, 0, 0, 0)
    end = test_module_start(source)
    problems = []
    lines = {n: h for n, h in cov.lines.items() if n < end}
    outcomes = {n: o for n, o in cov.outcomes.items() if n < end}
    for n in sorted(lines):
        if lines[n] == 0 and not exempt(source, n):
            problems.append(f"{rel}:{n}: line not run: {source[n - 1].strip()}")
    for n in sorted(outcomes):
        missed = outcomes[n].count(False)
        if missed and not exempt(source, n):
            problems.append(f"{rel}:{n}: {missed} of {len(outcomes[n])} condition outcomes not taken: "
                            f"{source[n - 1].strip()}")
    run = sum(1 for h in lines.values() if h)
    taken = sum(o.count(True) for o in outcomes.values())
    return problems, (run, len(lines), taken, sum(len(o) for o in outcomes.values()))


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("lcov")
    ap.add_argument("--report")
    args = ap.parse_args(argv)
    data = parse_lcov(args.lcov)
    by_rel = {}
    for path, cov in data.items():
        for rel in SAFETY_CRITICAL:
            if path.endswith("/" + rel):
                by_rel[rel] = cov
    out, problems = [], []
    for rel in SAFETY_CRITICAL:
        p, (run, lines, taken, outcomes) = check(rel, by_rel.get(rel))
        problems += p
        pct = lambda a, b: f"{100.0 * a / b:5.1f} %" if b else "    -  "
        out.append(f"{rel:38s} lines {run:4d}/{lines:<4d} {pct(run, lines)}   "
                   f"conditions {taken:4d}/{outcomes:<4d} {pct(taken, outcomes)}")
    exemptions = sum(1 for rel in SAFETY_CRITICAL for t in (RUST / rel).read_text().splitlines() if EXEMPT.search(t))
    out.append(f"{len(SAFETY_CRITICAL)} safety-critical files; {exemptions} COVERAGE-EXEMPT lines; "
               f"{len(problems)} problem(s)")
    text = "\n".join(out + [""] + problems)
    print(text)
    if args.report:
        Path(args.report).write_text(text + "\n")
    for p in problems:
        print(f"::error::{p}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
