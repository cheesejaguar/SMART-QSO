#!/usr/bin/env python3
"""
Generate the requirements traceability matrix (FINDINGS_REGISTER VV-06).

Sources: the three requirement specifications (mission, system, software),
         `@requirement ID` tags in software/ (.c, .h, .py),
         requirement IDs cited in docs/test_procedures/*.md
Output:  docs/verification/RTM.csv

One row per requirement, in specification order:

  id             requirement ID
  specification  the specification that defines it
  parent         parent requirement (system requirements only)
  method         verification method from the specification (A, D, I, T)
  requirement    requirement text as written in the specification
  code           source files that tag it with @requirement
  tests          test files that tag it with @requirement
  procedures     test procedures that cite it

The RTM records traceability: where each requirement is implemented and
which procedure is meant to verify it. It records no verification status;
that is kept only in the VCRM, docs/verification/VERIFICATION_MATRIX.md.
Requirement IDs cited but defined nowhere are reported by
scripts/check_traceability.py, not listed here. Code locations are files,
not lines, so the CSV does not go stale when unrelated lines move.

A procedure line that starts with "**Not cited" or "**To be corrected"
explains why a step does not verify a requirement (its value conflicts
with it); IDs on such a line are not citations.

    python3 scripts/gen_rtm.py           # write docs/verification/RTM.csv
    python3 scripts/gen_rtm.py --check   # fail if the committed CSV is stale (CI)
"""

from __future__ import annotations

import argparse
import csv
import io
import re
import sys
from pathlib import Path
from typing import Dict, Iterator, List, Optional, Sequence

from check_traceability import ID, REF

REPO = Path(__file__).resolve().parents[1]
OUT = REPO / "docs" / "verification" / "RTM.csv"

#: Requirement specifications, top down
SPECS = (
    "docs/requirements/MISSION_REQUIREMENTS.md",
    "docs/requirements/SYSTEM_REQUIREMENTS.md",
    "docs/software/SOFTWARE_REQUIREMENTS.md",
)

COLUMNS = ("id", "specification", "parent", "method", "requirement", "code", "tests", "procedures")

#: A requirement table's header: first column "ID", second one of these
TEXT_COLUMNS = {"requirement", "objective", "constraint"}
SKIP_DIRS = {"build", "node_modules", "venv", "__pycache__"}
SEPARATOR = re.compile(r"^\|(\s*:?-+:?\s*\|)+\s*$")
ROW_ID = re.compile(r"^" + ID + r"$")
NOTE = re.compile(r"^\*\*(?:Not cited|To be corrected)\b.*$", re.M)


def _cells(line: str) -> List[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def _spec_rows(root: Path, spec: str) -> Iterator[Dict[str, str]]:
    lines = (root / spec).read_text(encoding="utf-8").splitlines()
    header: Optional[List[str]] = None
    for i, line in enumerate(lines):
        if not line.startswith("|"):
            header = None
            continue
        if i + 1 < len(lines) and SEPARATOR.match(lines[i + 1]) and not SEPARATOR.match(line):
            names = [c.lower() for c in _cells(line)]
            ok = len(names) >= 2 and names[0] == "id" and names[1] in TEXT_COLUMNS
            header = names if ok else None
            continue
        if header is None or SEPARATOR.match(line):
            continue
        cells = _cells(line)
        if not cells or not ROW_ID.match(cells[0]):
            continue
        field = dict(zip(header, cells))
        yield {
            "id": cells[0],
            "specification": spec,
            "parent": field.get("parent", ""),
            "method": field.get("verification", ""),
            "requirement": cells[1] if len(cells) > 1 else "",
        }


def _sources(root: Path, top: str, suffixes: Sequence[str], recursive: bool = True) -> Iterator[Path]:
    base = root / top
    if not base.exists():
        return
    for path in sorted(base.rglob("*") if recursive else base.glob("*")):
        rel = path.relative_to(root).parts
        if path.suffix in suffixes and path.is_file() and \
                not any(p in SKIP_DIRS or p.startswith(".") for p in rel):
            yield path


def _is_test(rel: str) -> bool:
    parts = rel.split("/")
    return "tests" in parts[:-1] or parts[-1].startswith("test_")


def rows(root: Path = REPO, specs: Sequence[str] = SPECS) -> List[Dict[str, str]]:
    """The RTM rows, one per requirement definition, in specification order."""
    table = [row for spec in specs if (root / spec).exists() for row in _spec_rows(root, spec)]
    defined = {row["id"] for row in table}

    code: Dict[str, set] = {}
    tests: Dict[str, set] = {}
    for path in _sources(root, "software", (".c", ".h", ".py")):
        rel = path.relative_to(root).as_posix()
        for rid in set(REF.findall(path.read_text(encoding="utf-8", errors="replace"))):
            (tests if _is_test(rel) else code).setdefault(rid, set()).add(rel)

    procedures: Dict[str, set] = {}
    for path in _sources(root, "docs/test_procedures", (".md",), recursive=False):
        rel = path.relative_to(root).as_posix()
        text = NOTE.sub("", path.read_text(encoding="utf-8", errors="replace"))
        for rid in set(re.findall(ID, text)) & defined:
            procedures.setdefault(rid, set()).add(rel)

    for row in table:
        rid = row["id"]
        row["code"] = "; ".join(sorted(code.get(rid, ())))
        row["tests"] = "; ".join(sorted(tests.get(rid, ())))
        row["procedures"] = "; ".join(sorted(procedures.get(rid, ())))
    return table


def render(root: Path = REPO, specs: Sequence[str] = SPECS) -> str:
    buffer = io.StringIO()
    writer = csv.DictWriter(buffer, fieldnames=COLUMNS, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows(root, specs))
    return buffer.getvalue()


def check(root: Path = REPO, specs: Sequence[str] = SPECS, out: Path = OUT) -> bool:
    """True if the committed CSV matches what the sources generate."""
    return out.exists() and out.read_text(encoding="utf-8") == render(root, specs)


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Generate docs/verification/RTM.csv (VV-06)")
    parser.add_argument("--check", action="store_true", help="fail if the committed CSV is stale")
    args = parser.parse_args(argv)

    if args.check:
        if check():
            print(f"{OUT.relative_to(REPO)} is current")
            return 0
        print(f"::error::{OUT.relative_to(REPO)} is stale (run python3 scripts/gen_rtm.py)")
        return 1

    text = render()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)
    print(f"wrote {OUT.relative_to(REPO)} ({text.count(chr(10)) - 1} requirements)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
