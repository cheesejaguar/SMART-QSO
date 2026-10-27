#!/usr/bin/env python3
"""
Board-level FMEA tables (SQSO-FMEA-003, docs/safety/BOARD_FMEA.md).

The failure modes live in docs/safety/BOARD_FMEA_ROWS.csv, one row each. This
script checks the rows and writes, between the markers in BOARD_FMEA.md, the
severity counts and the per-board FMEA tables, so the document and the counts
cannot drift from the rows.

Checks on every row:
  * unique ID of the form BF-<BOARD>-NN, the board code matching the prefix;
  * severity 1-4 (the categories of SQSO-FMEA-002 section 1), criticality one of
    1, 1R, 2, 2R, 3, 4 with the same leading digit as the severity;
  * single-point flag Y (single failure that ends the mission: severity 2,
    criticality 2), L2 (single point for transponder service: severity 3) or
    the dash;
  * failure-mode class one of open, short, stuck, drift, loss of output,
    spurious;
  * no empty cell except the reference, and no TBD or TBR.

    python3 scripts/board_fmea.py            # write the tables, print counts
    python3 scripts/board_fmea.py --check    # exit 1 if a row is bad or the
                                             # document is out of date
"""

from __future__ import annotations

import argparse
import csv
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Dict, List

REPO = Path(__file__).resolve().parents[1]
ROWS = REPO / "docs/safety/BOARD_FMEA_ROWS.csv"
DOC = REPO / "docs/safety/BOARD_FMEA.md"

BOARDS = [
    ("EPS", "EPS board (SQSO-EPS-BRD-001 D) and the battery sense board"),
    ("OBC", "OBC board (SQSO-OBC-BRD-001 C)"),
    ("A2", "Magnetometer board A2 (SQSO-OBC-BRD-001 C §9.4)"),
    ("A3", "Access-port board A3 (SQSO-OBC-BRD-001 C §15.1)"),
    ("RF", "RF board (SQSO-RF-BRD-001 D)"),
    ("JET", "Jetson carrier (SQSO-JET-BRD-001 E)"),
    ("HAR", "Interconnect and harness (SQSO-ICX-001 C, SQSO-HAR-001 2.0)"),
]
MODES = {"open", "short", "stuck", "drift", "loss of output", "spurious"}
CRIT = {"1", "1R", "2", "2R", "3", "4"}
SPF = {"Y", "L2", "—"}
ID = re.compile(r"^BF-(EPS|OBC|A2|A3|RF|JET|HAR)-\d{2}$")
OPEN_WORD = re.compile(r"\bTB[DR]\b")
COLUMNS = ["id", "board", "item", "function", "mode", "failure_mode", "cause", "local_effect",
           "system_effect", "detection", "compensation", "sev", "crit", "spf", "refs"]
HEADER = ("| ID | Item | Function | Mode | Failure mode | Cause | Local effect | System effect | "
          "Detection | Compensating provision | Sev | Crit | SPF | Ref |")
RULE = "|" + "----|" * 14


def load() -> List[Dict[str, str]]:
    with ROWS.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames != COLUMNS:
            raise SystemExit(f"{ROWS}: columns {reader.fieldnames} != {COLUMNS}")
        return [dict(r) for r in reader]


def problems(rows: List[Dict[str, str]]) -> List[str]:
    out = []
    seen = set()
    for n, r in enumerate(rows, start=2):
        where = f"{ROWS.name}:{n} {r['id']}"
        m = ID.match(r["id"])
        if not m:
            out.append(f"{where}: bad ID")
        elif not r["board"].startswith(m.group(1)):
            out.append(f"{where}: board {r['board']} does not start with {m.group(1)}")
        if r["id"] in seen:
            out.append(f"{where}: duplicate ID")
        seen.add(r["id"])
        if r["sev"] not in {"1", "2", "3", "4"}:
            out.append(f"{where}: severity {r['sev']}")
        if r["crit"] not in CRIT or r["crit"][0] != r["sev"]:
            out.append(f"{where}: criticality {r['crit']} with severity {r['sev']}")
        if r["spf"] not in SPF:
            out.append(f"{where}: SPF flag {r['spf']}")
        if r["spf"] == "Y" and (r["sev"], r["crit"]) != ("2", "2"):
            out.append(f"{where}: SPF Y needs severity 2 and criticality 2")
        if r["spf"] == "L2" and r["sev"] != "3":
            out.append(f"{where}: SPF L2 needs severity 3")
        if r["mode"] not in MODES:
            out.append(f"{where}: mode {r['mode']}")
        for k in COLUMNS[:-1]:
            if not r[k].strip():
                out.append(f"{where}: empty {k}")
        for k, v in r.items():
            if OPEN_WORD.search(v):
                out.append(f"{where}: open-item word in {k}")
    return out


def cell(text: str) -> str:
    return text.replace("|", "\\|").strip()


def counts(rows: List[Dict[str, str]]) -> str:
    sev = Counter(r["sev"] for r in rows)
    crit = Counter(r["crit"] for r in rows)
    lines = [
        f"{len(rows)} failure modes. By severity: "
        + ", ".join(f"**{k}**: {sev.get(k, 0)}" for k in "1234")
        + ". By criticality: "
        + ", ".join(f"{k}: {crit.get(k, 0)}" for k in ["1", "1R", "2", "2R", "3", "4"])
        + f". Whole-mission single points (SPF Y): **{sum(r['spf'] == 'Y' for r in rows)}**; "
        + f"single points for transponder service (SPF L2): {sum(r['spf'] == 'L2' for r in rows)}.",
        "",
        "| Board | Modes | Sev 1 | Sev 2 | Sev 3 | Sev 4 | SPF Y | SPF L2 |",
        "|-------|------:|------:|------:|------:|------:|------:|-------:|",
    ]
    for code, name in BOARDS:
        b = [r for r in rows if r["id"].startswith(f"BF-{code}-")]
        s = Counter(r["sev"] for r in b)
        lines.append(
            f"| {code} | {len(b)} | {s.get('1', 0)} | {s.get('2', 0)} | {s.get('3', 0)} | {s.get('4', 0)} | "
            f"{sum(r['spf'] == 'Y' for r in b)} | {sum(r['spf'] == 'L2' for r in b)} |"
        )
    lines.append(
        f"| **Total** | **{len(rows)}** | {sev.get('1', 0)} | {sev.get('2', 0)} | {sev.get('3', 0)} | "
        f"{sev.get('4', 0)} | {sum(r['spf'] == 'Y' for r in rows)} | {sum(r['spf'] == 'L2' for r in rows)} |"
    )
    return "\n".join(lines)


def tables(rows: List[Dict[str, str]]) -> str:
    out = []
    for i, (code, name) in enumerate(BOARDS, start=1):
        b = [r for r in rows if r["id"].startswith(f"BF-{code}-")]
        out += [f"### 6.{i} {name}", "", HEADER, RULE]
        for r in b:
            out.append("| " + " | ".join(cell(r[k]) for k in COLUMNS if k != "board") + " |")
        out.append("")
    return "\n".join(out).rstrip()


def render(doc: str, rows: List[Dict[str, str]]) -> str:
    for name, body in (("COUNTS", counts(rows)), ("TABLES", tables(rows))):
        begin, end = f"<!-- BEGIN {name} (scripts/board_fmea.py) -->", f"<!-- END {name} -->"
        if begin not in doc or end not in doc:
            raise SystemExit(f"{DOC}: markers for {name} missing")
        head, rest = doc.split(begin, 1)
        _, tail = rest.split(end, 1)
        doc = f"{head}{begin}\n{body}\n{end}{tail}"
    return doc


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    ap.add_argument("--check", action="store_true", help="fail on a bad row or a stale document")
    args = ap.parse_args()
    rows = load()
    bad = problems(rows)
    for p in bad:
        print(f"::error::{p}")
    doc = DOC.read_text(encoding="utf-8")
    new = render(doc, rows)
    if args.check:
        if new != doc:
            print(f"::error::{DOC.relative_to(REPO)} is out of date: run python3 scripts/board_fmea.py")
            return 1
        return 1 if bad else 0
    if bad:
        return 1
    if new != doc:
        DOC.write_text(new, encoding="utf-8")
    print(counts(rows).splitlines()[0])
    return 0


if __name__ == "__main__":
    sys.exit(main())
