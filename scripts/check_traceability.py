#!/usr/bin/env python3
"""
Requirements traceability check (FINDINGS_REGISTER VV-06).

Measures three traceability defects, so CI can stop them growing while they
are worked down:

  undefined_references    requirement IDs cited in code (`@requirement ID`)
                          that no requirements specification defines
  conflicting_ids         requirement IDs described somewhere with almost
                          no words in common with their specification:
                          the same ID meaning different requirements
  procedures_without_ids  test procedures that cite no defined requirement

    python3 scripts/check_traceability.py            # report
    python3 scripts/check_traceability.py --check    # fail if any count rose
                                                     # above scripts/traceability_baseline.json
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional

REPO = Path(__file__).resolve().parents[1]
BASELINE = Path(__file__).resolve().parent / "traceability_baseline.json"

#: Requirement specifications: the documents that define requirement IDs
SPECS = (
    "docs/requirements/SYSTEM_REQUIREMENTS.md",
    "docs/requirements/MISSION_REQUIREMENTS.md",
    "docs/software/SOFTWARE_REQUIREMENTS.md",
)

ID = r"[A-Z]{2,4}(?:-[A-Z]{1,5})?-[A-Z]?\d{1,4}"
ROW = re.compile(r"^\|\s*(" + ID + r")\s*\|\s*([^|]+?)\s*\|", re.M)
REF = re.compile(r"@requirement\s+(" + ID + r")\b")
WORD = re.compile(r"[a-z0-9]+")
STOP = {"the", "shall", "a", "an", "of", "to", "in", "and", "or", "be", "for", "with", "on",
        "at", "by", "is", "spacecraft", "system", "software", "satellite"}
SIMILAR = 0.2   # word overlap below this means a different requirement


def _words(text: str) -> set:
    return {w for w in WORD.findall(text.lower()) if w not in STOP}


def _definitions(root: Path, files) -> Dict[str, List[tuple]]:
    found: Dict[str, List[tuple]] = {}
    for path in files:
        for match in ROW.finditer(path.read_text(encoding="utf-8", errors="replace")):
            text = match.group(2)
            # Matrix marks ("X") and lists of IDs are not descriptions
            if re.search(r"[a-z]{3,}", text):
                found.setdefault(match.group(1), []).append((str(path.relative_to(root)), text))
    return found


def measure(root: Path = REPO, specs=SPECS) -> Dict[str, List[str]]:
    spec_paths = [root / s for s in specs if (root / s).exists()]
    defined = _definitions(root, spec_paths)
    all_docs = [p for p in (root / "docs").rglob("*.md") if "funding_audit" not in p.parts]
    described = _definitions(root, all_docs)

    refs = set()
    for path in (root / "software").rglob("*"):
        if path.suffix in (".c", ".h", ".py") and path.is_file():
            refs.update(REF.findall(path.read_text(encoding="utf-8", errors="replace")))

    # Only requirement IDs (defined in a spec): each description anywhere is
    # compared with the spec's own wording
    conflicting = []
    for rid, spec_entries in defined.items():
        spec_words = [_words(text) for _, text in spec_entries]
        for _, text in described.get(rid, []):
            other = _words(text)
            if other and all(a and len(a & other) / len(a | other) < SIMILAR for a in spec_words):
                conflicting.append(rid)
                break

    procedures = []
    tp_dir = root / "docs/test_procedures"
    if tp_dir.exists():
        for path in sorted(tp_dir.glob("*.md")):
            cited = set(re.findall(ID, path.read_text(encoding="utf-8", errors="replace")))
            if not cited & set(defined):
                procedures.append(str(path.relative_to(root)))

    return {
        "undefined_references": sorted(refs - set(defined)),
        "conflicting_ids": sorted(set(conflicting)),
        "procedures_without_ids": procedures,
    }


def compare(metrics: Dict[str, List[str]], baseline_path: Path) -> List[str]:
    """Counts above the baseline, one line each."""
    baseline = json.loads(baseline_path.read_text())
    return [f"{key}: {len(items)}, baseline {baseline[key]}"
            for key, items in metrics.items() if len(items) > baseline[key]]


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Requirements traceability check (VV-06)")
    parser.add_argument("--check", action="store_true", help="fail if a count rose above the baseline")
    args = parser.parse_args(argv)

    metrics = measure()
    for key, items in metrics.items():
        print(f"{key}: {len(items)}")
        for item in items[:15]:
            print(f"  {item}")
        if len(items) > 15:
            print(f"  ... {len(items) - 15} more")
    if args.check:
        rises = compare(metrics, BASELINE)
        for line in rises:
            print(f"::error::traceability got worse: {line}")
        return 1 if rises else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
