#!/usr/bin/env python3
"""
Findings-register count check.

The register's status line and severity table summarise its rows. This
recounts both from the rows, so a row whose status changes cannot leave
the summary behind.

    python3 scripts/register_status.py           # print the counts
    python3 scripts/register_status.py --check   # exit 1 if the summary disagrees
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional

REGISTER = Path(__file__).resolve().parents[1] / "docs/funding_audit/FINDINGS_REGISTER.md"

ROW = re.compile(r"^\| ([A-Z]{2,3})-\d+ \|", re.M)
SEVERITIES = ("Critical", "High", "Medium", "Low")


def _status(cell: str) -> str:
    if cell.startswith("**Closed"):
        return "closed"
    if cell.startswith("**Partially closed"):
        return "partial"
    if cell == "—":
        return "informational"
    return "open"


def count(text: str) -> Dict[str, Dict]:
    status = {"closed": 0, "partial": 0, "open": 0, "informational": 0}
    severity: Dict[str, Dict[str, int]] = {}
    for line in text.splitlines():
        match = ROW.match(line)
        if not match:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
        status[_status(cells[-1])] += 1
        level = cells[1].split(" ")[0]          # "Low (positive)" counts as Low
        per = severity.setdefault(match.group(1), dict.fromkeys(SEVERITIES, 0))
        per[level] += 1
    return {"status": status, "severity": severity}


def status_line(status: Dict[str, int]) -> str:
    return (f"Status: **{status['closed']} closed** and **{status['partial']} partially closed**; "
            f"**{status['open']} open**; {status['informational']} informational")


def check(path: Path = REGISTER) -> List[str]:
    """One line per disagreement between the summary and the rows."""
    text = path.read_text(encoding="utf-8")
    counts = count(text)
    problems = []

    expected = status_line(counts["status"])
    if expected not in text:
        problems.append(f"status line is stale; it should start: {expected}")

    totals = dict.fromkeys(SEVERITIES, 0)
    for cat, per in counts["severity"].items():
        for level in SEVERITIES:
            totals[level] += per[level]
        want = [per[level] for level in SEVERITIES] + [sum(per.values())]
        row = re.search(rf"^\| {cat} — [^|]+\|((?: *\d+ *\|){{5}})", text, re.M)
        got = [int(n) for n in re.findall(r"\d+", row.group(1))] if row else None
        if got != want:
            problems.append(f"severity table row {cat}: says {got}, rows give {want} "
                            f"({', '.join(SEVERITIES)}, Total)")

    want_total = [totals[level] for level in SEVERITIES] + [sum(totals.values())]
    row = re.search(r"^\| \*\*Total\*\* \|((?: *\*\*\d+\*\* *\|){5})", text, re.M)
    got_total = [int(n) for n in re.findall(r"\d+", row.group(1))] if row else None
    if got_total != want_total:
        problems.append(f"severity table Total: says {got_total}, rows give {want_total}")
    return problems


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Findings-register count check")
    parser.add_argument("--check", action="store_true", help="fail if the summary disagrees with the rows")
    args = parser.parse_args(argv)

    counts = count(REGISTER.read_text(encoding="utf-8"))
    print(status_line(counts["status"]))
    if not args.check:
        return 0
    problems = check()
    for line in problems:
        print(f"::error::FINDINGS_REGISTER: {line}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
