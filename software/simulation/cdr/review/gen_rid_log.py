#!/usr/bin/env python3
"""Render the CDR 2026 RID log into docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md.

The log sits between the markers <!-- RID-LOG-BEGIN --> and <!-- RID-LOG-END -->; the
summary counts between <!-- RID-SUMMARY-BEGIN --> and <!-- RID-SUMMARY-END -->.

    python3 software/simulation/cdr/review/gen_rid_log.py          # write
    python3 software/simulation/cdr/review/gen_rid_log.py --check  # exit 1 if stale
"""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from rids_round1 import RIDS  # noqa: E402
from rids_status import CLOSURE  # noqa: E402
from rids_round3 import FINAL3, NEW, REOPEN  # noqa: E402

RIDS = RIDS + NEW

REPO = HERE.parents[3]
REPORT = REPO / "docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md"

NAMES = {
    "CA": "Closed by analysis",
    "CD": "Closed by design change",
    "CH": "Closed by chair decision",
    "VI": "Verification item",
    "DP": "Decision point (dated)",
    "IMP": "Implementation",
}
GROUPS = [
    ("LV", "Launch, mission integration and ConOps"),
    ("PM", "Programmatics"), ("SE", "Systems engineering"), ("VV", "Verification and validation"),
    ("RF", "RF link and hardware"), ("GND", "Ground segment"), ("REG", "Regulatory"), ("DB", "Data budget"),
    ("ODA", "Orbital debris"), ("SMA", "Safety and mission assurance"), ("REL", "Reliability"),
    ("STR", "Structures and mechanisms"), ("MCAD", "Mechanical CAD and FE"),
    ("EPS", "Electrical power"), ("THM", "Thermal"), ("EMC", "EMC"),
    ("AV", "Avionics"), ("FSW", "Flight software"), ("RAD", "Radiation"), ("GNC", "Guidance, navigation and control"),
    ("RFB", "RF board (round 2)"), ("EPSB", "EPS board (round 2)"), ("OBCB", "OBC board (round 2)"),
    ("JETB", "Jetson carrier board (round 2)"),
    ("P3P", "Round 3: programmatics, SE, V&V"), ("P3E", "Round 3: power, safety, EMC"),
    ("P3S", "Round 3: structures, mechanisms, thermal"), ("P3A", "Round 3: launch, RF, ground, regulatory"),
    ("P3F", "Round 3: avionics, flight software, GN&C"), ("P3T", "Round 3: thermal and parts"),
]


def final(rid):
    """(disposition, evidence) after round 2: an implementation RID takes its closure."""
    rid_id, _sev, _title, disp, evidence = rid
    if rid_id in FINAL3:
        return FINAL3[rid_id]
    if rid_id in REOPEN:
        return "IMP", "Reopened in round 3: " + REOPEN[rid_id]
    if rid_id in CLOSURE:
        return CLOSURE[rid_id]
    return disp, evidence


def esc(text: str) -> str:
    return text.replace("|", "\\|")


def render() -> tuple[str, str]:
    known = {prefix for prefix, _ in GROUPS}
    missing = sorted({r[0].split("-")[0] for r in RIDS} - known)
    if missing:
        raise SystemExit(f"RID prefixes without a group: {missing}")
    dupes = sorted({r[0] for r in RIDS if [x[0] for x in RIDS].count(r[0]) > 1})
    if dupes:
        raise SystemExit(f"duplicate RIDs: {dupes}")
    unknown = sorted((set(CLOSURE) | set(REOPEN) | set(FINAL3)) - {r[0] for r in RIDS})
    if unknown:
        raise SystemExit(f"closures for unknown RIDs: {unknown}")
    rows = []
    for prefix, name in GROUPS:
        group = [r for r in RIDS if r[0].split("-")[0] == prefix]
        if not group:
            continue
        rows.append(f"\n#### {name} ({prefix})\n")
        rows.append("| RID | Sev | Finding | Round 1 | Final disposition | Evidence |")
        rows.append("|-----|-----|---------|---------|-------------------|----------|")
        for r in group:
            disp, ev = final(r)
            rows.append(f"| {r[0]} | {r[1]} | {esc(r[2])} | {r[3]} | {NAMES[disp]} | {esc(ev)} |")
    log = "\n".join(rows).strip() + "\n"

    first = Counter(r[3] for r in RIDS)
    last = Counter(final(r)[0] for r in RIDS)
    sev = Counter(r[1] for r in RIDS)
    open_imp = [r[0] for r in RIDS if final(r)[0] == "IMP"]
    lines = [
        f"{len(RIDS)} RIDs: severity 1: {sev[1]}, severity 2: {sev[2]}, severity 3: {sev[3]}.",
        "",
        "| Disposition | Round 1 | Final |",
        "|-------------|---------|-------|",
    ]
    for key in ["CA", "CD", "CH", "VI", "DP", "IMP"]:
        lines.append(f"| {NAMES[key]} | {first[key]} | {last[key]} |")
    lines.append("")
    reopened = sorted(REOPEN)
    lines.append(
        f"Round 3 (independent closure verification) reopened {len(reopened)} RIDs and raised {len(NEW)} new findings; "
        f"{sum(1 for r in reopened if r in FINAL3)} of the reopened and "
        f"{sum(1 for n in NEW if n[0] in FINAL3)} of the new are since closed."
    )
    lines.append("")
    lines.append(
        "Open implementation RIDs: " + (", ".join(open_imp) if open_imp else "none") + "."
    )
    return "\n".join(lines) + "\n", log


def splice(text: str, tag: str, body: str) -> str:
    begin, end = f"<!-- {tag}-BEGIN -->", f"<!-- {tag}-END -->"
    i, j = text.index(begin) + len(begin), text.index(end)
    return text[:i] + "\n" + body + text[j:]


def main() -> int:
    args = sys.argv[1:]
    if any(a not in ("--check",) for a in args):
        print(__doc__)
        return 0 if args in (["--help"], ["-h"]) else 2
    summary, log = render()
    text = REPORT.read_text(encoding="utf-8")
    new = splice(splice(text, "RID-SUMMARY", summary), "RID-LOG", log)
    if "--check" in sys.argv:
        if new != text:
            print("RID log is stale: run python3 software/simulation/cdr/review/gen_rid_log.py")
            return 1
        print("RID log is current")
        return 0
    REPORT.write_text(new, encoding="utf-8")
    print(f"wrote {REPORT.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
