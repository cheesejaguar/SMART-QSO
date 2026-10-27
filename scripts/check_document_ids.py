#!/usr/bin/env python3
"""
Document-ID check (FINDINGS_REGISTER PI-17 and VV-18).

Each controlled document carries its ID in a header line near the top:

    **Document Number:** SQSO-ICD-013        (also **Document ID**: ...)

Test procedures carry IDs such as TP-INT-001, either as a file's document
number (written TP-INT-001 or SQSO-TP-INT-001) or as a heading inside a
multi-procedure document ("### 4.1 TP-SWI-001: ...", "### 4.1 TP-001: ...").

The check fails when:
  * one document ID is the header of more than one file;
  * one test-procedure ID is defined in more than one file;
  * a document or procedure with an ID is missing from docs/DOCUMENT_REGISTER.md;
  * a register row names an ID that its file no longer carries (stale row);
  * a register row's revision differs from the revision in its document's header.

IDs are compared exactly as written: SMART-QSO-SDP-001 and SQSO-SDP-001 are
different IDs (the two prefixes are a known inconsistency, listed in the
register). Only the first 20 lines are read for the header, outside code
blocks, so report templates inside a document are not taken as documents.

    python3 scripts/check_document_ids.py            # report
    python3 scripts/check_document_ids.py --check    # exit 1 on any problem
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

REPO = Path(__file__).resolve().parents[1]
REGISTER = "docs/DOCUMENT_REGISTER.md"
HEADER_LINES = 20

#: Directories that are not part of the controlled document set (hidden
#: directories such as .claude/ and .git/ are skipped as well)
EXCLUDED = ("docs/funding_audit",)
BUILD_DIRS = {"build", "node_modules", "venv", "__pycache__"}

DOC_ID = r"(?:SQSO|SMART-QSO|TP)(?:-[A-Z0-9]+)+"
HEADER = re.compile(r"^\*\*Document (?:Number|ID)(?::\*\*|\*\*:)\s*(" + DOC_ID + r")\b")
TP_ID = re.compile(r"^(?:SQSO-)?(TP-[A-Z]+-\d{3})$")
TP_HEADING = re.compile(r"^#{1,6}\s+(?:\d+(?:\.\d+)*\.?\s+)?(TP-(?:[A-Z]+-)?\d{3}):", re.M)
REGISTER_ROW = re.compile(r"^\|\s*(" + DOC_ID + r")\s*\|.*?`([^`]+\.md)`", re.M)
# ID | title | path | revision | ...: the revision cell, to compare with the document
REGISTER_REVISION = re.compile(r"^\|\s*(" + DOC_ID + r")\s*\|[^|]*\|\s*`([^`]+\.md)`\s*\|\s*([^|\s(]+)", re.M)
HEADER_REVISION = re.compile(r"^\*\*(?:Revision|Version)(?::\*\*|\*\*:)\s*([^\s(|]+)", re.M)
FENCE = re.compile(r"```.*?```", re.S)


def _documents(root: Path) -> List[Path]:
    out = []
    for path in sorted(root.rglob("*.md")):
        rel = path.relative_to(root)
        if any(part.startswith(".") or part in BUILD_DIRS for part in rel.parts[:-1]):
            continue
        if any(rel.as_posix().startswith(prefix + "/") for prefix in EXCLUDED):
            continue
        out.append(path)
    return out


def _header_id(text: str) -> Optional[str]:
    in_fence = False
    for line in text.splitlines()[:HEADER_LINES]:
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        match = HEADER.match(line.strip())
        if match and not in_fence:
            return match.group(1)
    return None


def _inventory(root: Path) -> List[Tuple[str, Optional[str], Set[str]]]:
    """(path, header ID or None, test-procedure IDs defined in headings) per document."""
    out = []
    for path in _documents(root):
        text = path.read_text(encoding="utf-8", errors="replace")
        out.append((path.relative_to(root).as_posix(), _header_id(text),
                    set(TP_HEADING.findall(FENCE.sub("", text)))))
    return out


def _procedure(ident: Optional[str]) -> Optional[str]:
    match = TP_ID.match(ident or "")
    return match.group(1) if match else None


def scan(root: Path = REPO) -> Dict[str, List[str]]:
    """Document ID -> files whose header carries it."""
    found: Dict[str, List[str]] = {}
    for rel, header, _ in _inventory(root):
        if header:
            found.setdefault(header, []).append(rel)
    return found


def procedures(root: Path = REPO) -> Dict[str, List[str]]:
    """Test-procedure ID -> files that define it (file header or `TP-...:` heading)."""
    found: Dict[str, List[str]] = {}
    for rel, header, headings in _inventory(root):
        ids = set(headings)
        if _procedure(header):
            ids.add(_procedure(header))
        for ident in sorted(ids):
            found.setdefault(ident, []).append(rel)
    return found


def registered(root: Path = REPO) -> Set[Tuple[str, str]]:
    """(ID, path) pairs listed in the document register."""
    path = root / REGISTER
    if not path.exists():
        return set()
    return set(REGISTER_ROW.findall(path.read_text(encoding="utf-8")))


def check(root: Path = REPO) -> List[str]:
    """Return one line per problem; empty if document control is consistent."""
    problems = []
    for ident, files in sorted(scan(root).items()):
        if len(files) > 1:
            problems.append(f"{ident} is the document ID of {len(files)} files: {', '.join(files)}")
    for ident, files in sorted(procedures(root).items()):
        if len(files) > 1:
            problems.append(f"{ident} is defined as a test procedure in {len(files)} files: "
                            f"{', '.join(files)}")

    # What the register must list: each header ID, and each procedure defined
    # in a heading of a document whose own header is not that procedure
    items: Set[Tuple[str, str]] = set()
    for rel, header, headings in _inventory(root):
        if header:
            items.add((header, rel))
        items |= {(ident, rel) for ident in headings if ident != _procedure(header)}
    listed = registered(root)
    register_text = (root / REGISTER).read_text(encoding="utf-8") if (root / REGISTER).exists() else ""
    counts: Dict[Tuple[str, str], int] = {}
    for pair in REGISTER_ROW.findall(register_text):
        counts[pair] = counts.get(pair, 0) + 1
    for (ident, rel), n in sorted(counts.items()):
        if n > 1:
            problems.append(f"{REGISTER}: row {ident} `{rel}` is listed {n} times")

    for ident, rel in sorted(items - listed):
        problems.append(f"{ident} ({rel}) is not in {REGISTER}")
    for ident, rel in sorted(listed - items):
        problems.append(f"{REGISTER}: stale row {ident} `{rel}` "
                        f"(that file does not carry or define this ID)")

    # A document's revision bumped without its register row
    register_path = root / REGISTER
    rows = REGISTER_REVISION.findall(register_path.read_text(encoding="utf-8")) if register_path.exists() else []
    for ident, rel, rev in rows:
        doc_path = root / rel
        if not doc_path.exists():
            continue
        text = doc_path.read_text(encoding="utf-8", errors="replace")
        if _header_id(text) != ident:
            continue    # a procedure defined inside a larger document: the cell names its section
        match = HEADER_REVISION.search("\n".join(text.splitlines()[:HEADER_LINES]))
        if match and match.group(1) != rev:
            problems.append(f"{REGISTER}: {ident} is at revision {rev}, but {rel} says {match.group(1)}")
    return problems


def main(argv: Optional[List[str]] = None, root: Path = REPO) -> int:
    parser = argparse.ArgumentParser(description="Document-ID check (PI-17, VV-18)")
    parser.add_argument("--check", action="store_true", help="exit 1 on any problem")
    args = parser.parse_args(argv)

    problems = check(root)
    for line in problems:
        print(("::error::" if args.check else "") + line)
    docs = scan(root)
    print(f"{len(docs)} document IDs in {sum(len(v) for v in docs.values())} files; "
          f"{len(procedures(root))} test-procedure IDs; {len(problems)} problem(s)")
    return 1 if (args.check and problems) else 0


if __name__ == "__main__":
    sys.exit(main())
