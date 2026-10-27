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
]


def final(rid):
    """(disposition, evidence) after round 2: an implementation RID takes its closure."""
    rid_id, _sev, _title, disp, evidence = rid
    if rid_id in CLOSURE:
        return CLOSURE[rid_id]
    return disp, evidence


def esc(text: str) -> str:
    return text.replace("|", "\\|")


def render() -> tuple[str, str]:
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
    lines.append(
        "Open implementation RIDs: " + (", ".join(open_imp) if open_imp else "none") + "."
    )
    return "\n".join(lines) + "\n", log


def splice(text: str, tag: str, body: str) -> str:
    begin, end = f"<!-- {tag}-BEGIN -->", f"<!-- {tag}-END -->"
    i, j = text.index(begin) + len(begin), text.index(end)
    return text[:i] + "\n" + body + text[j:]


def main() -> int:
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
