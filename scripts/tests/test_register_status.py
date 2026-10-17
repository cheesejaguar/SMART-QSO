"""Tests for the findings-register count check (scripts/register_status.py)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from register_status import count, check  # noqa: E402

HEADER = """# Findings Register

| Category | Critical | High | Medium | Low | Total |
|---|---|---|---|---|---|
| PI — Program | {pc} | {ph} | 0 | {pl} | {pt} |
| SW — Software | 0 | {sh} | 0 | 0 | {st} |
| **Total** | **{tc}** | **{th}** | **0** | **{tl}** | **{tt}** |

{status}

| ID | Severity | Finding | Where | Fix | Status |
|----|----------|---------|-------|-----|--------|
"""

ROWS = """| PI-01 | Critical | **A** | `a` | b | **Closed (this PR):** done |
| PI-02 | High | B | `a` | b | **Partially closed (this PR):** x. Open: y |
| PI-03 | Low (positive) | C | `a` | b | — |
| SW-01 | High | D | `a` | b | Open |
| SW-02 | High | E | `a` | b | Open (decision D1) |
"""

GOOD_STATUS = "Status: **1 closed** and **1 partially closed**; **2 open**; 1 informational (2026-10-16)."
GOOD_TABLE = dict(pc=1, ph=1, pl=1, pt=3, sh=2, st=2, tc=1, th=3, tl=1, tt=5)


def register(tmp_path, status=GOOD_STATUS, **table):
    values = dict(GOOD_TABLE, **table)
    path = tmp_path / "FINDINGS_REGISTER.md"
    path.write_text(HEADER.format(status=status, **values) + ROWS)
    return path


def test_counts_each_status(tmp_path):
    counts = count(register(tmp_path).read_text())
    assert counts["status"] == {"closed": 1, "partial": 1, "open": 2, "informational": 1}
    assert counts["severity"]["PI"] == {"Critical": 1, "High": 1, "Medium": 0, "Low": 1}
    assert counts["severity"]["SW"] == {"Critical": 0, "High": 2, "Medium": 0, "Low": 0}


def test_consistent_register_passes(tmp_path):
    assert check(register(tmp_path)) == []


def test_stale_status_line_fails_with_the_right_line(tmp_path):
    stale = GOOD_STATUS.replace("**2 open**", "**3 open**")
    problems = check(register(tmp_path, status=stale))
    assert len(problems) == 1
    assert "**1 closed** and **1 partially closed**; **2 open**; 1 informational" in problems[0]


def test_stale_severity_table_fails(tmp_path):
    problems = check(register(tmp_path, sh=1, st=1, th=2, tt=4))
    assert any("SW" in p and "High" in p for p in problems)
    assert any("Total" in p for p in problems)
