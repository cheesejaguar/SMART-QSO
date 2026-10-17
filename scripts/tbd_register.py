#!/usr/bin/env python3
"""
TBD/TBR register (FINDINGS_REGISTER VV-17).

Lists every TBD and TBR in docs/ in docs/TBD_REGISTER.md, which this script
generates, so open items stay visible and their number cannot grow unnoticed.

What counts
  * The words TBD and TBR as upper-case tokens: "TBD", "(TBR)", "L-TBD".
    Plurals ("TBDs") and lower case do not count.
  * Tracked items, with IDs TBD-<CODE>-NNN or TBR-<CODE>-NNN, where CODE
    names the document that owns the item (TBD-MRD-001 belongs to the
    Mission Requirements Document). An item is defined by the row of its
    document's TBD/TBR table whose first cell is the ID; every other mention
    is a citation. A tracked item counts once, and a TBD or TBR word on a
    line that names an ID belongs to that item.
  * A definition row with a cell that starts "Closed" is a closed item. It
    keeps its number, which is not reused, and is not counted.

What does not count
  * Headings, table header rows and revision-history sections: they name or
    record items, they do not leave one open.
  * "TBD/TBR", the name of the category.
  * docs/funding_audit/, which quotes TBDs as evidence, and the register.

--check fails if the register is stale; an ID has no document code, is
defined more than once, or is cited but not defined; a code is used by items
of more than one document; or the open TBD and TBR counts differ from
scripts/tbd_baseline.json. As with the other ratchets, a lower count must
lower the baseline in the same change and a higher count fails.

    python3 scripts/tbd_register.py           # write docs/TBD_REGISTER.md
    python3 scripts/tbd_register.py --check   # CI
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Tuple

REPO = Path(__file__).resolve().parents[1]
REGISTER = "docs/TBD_REGISTER.md"
BASELINE = "scripts/tbd_baseline.json"
EXCLUDED = ("docs/funding_audit/", REGISTER)
KINDS = ("TBD", "TBR")
UNKNOWN = "PI"

ID = re.compile(r"\bTB[DR](?:-([A-Z][A-Z0-9]*))?-\d{3}\b")
WORD = re.compile(r"\bTB[DR]\b")
CATEGORY = re.compile(r"\bTB[DR]/TB[DR]\b")
HEADING = re.compile(r"^#{1,6}\s+(.*)$")
LABEL = re.compile(r"^\*\*([^*]+)\*\*:?$")
HISTORY = re.compile(r"revision history|document history|change log|changelog", re.I)
SEPARATOR = re.compile(r"^\|?(\s*:?-{3,}:?\s*\|)+\s*(:?-{3,}:?\s*)?$")
LINK = re.compile(r"\[([^\]]*)\]\([^)]*\)")
FENCE = ("```", "~~~")

#: Header words that name the owner and needed-by columns of a TBD/TBR table
OWNER_COLUMNS = ("owner",)
NEEDED_COLUMNS = ("target", "resolution", "needed", "due")


@dataclass
class Item:
    ident: str
    kind: str
    code: Optional[str]
    file: str
    line: int
    text: str
    owner: str
    needed_by: str
    closed: Optional[str] = None            # the "Closed ..." cell of a closed item
    refs: List[Tuple[str, int]] = field(default_factory=list)


@dataclass
class Occurrence:
    file: str
    line: int
    counts: Dict[str, int]
    text: str


@dataclass
class Register:
    items: Dict[str, Item]
    occurrences: List[Occurrence]
    problems: List[str]


def _cells(row: str) -> List[str]:
    return [c.strip() for c in re.split(r"(?<!\\)\|", row.strip().strip("|"))]


def _value(cells: List[str], header: Optional[List[str]], names) -> str:
    for index, name in enumerate(header or []):
        if any(word in name.lower() for word in names) and index < len(cells):
            value = cells[index].strip("* ")
            if value and not WORD.search(value) and value not in ("-", "—", "N/A"):
                return value
            break
    return UNKNOWN


def _plain(text: str) -> str:
    """Quoted text safe inside this register's tables: no links (they would be
    relative to another directory), no code or bold markers, pipes escaped."""
    text = text.strip()
    if text.startswith("|"):
        text = text.strip("|").strip()
    text = LINK.sub(r"\1", text).replace("`", "").replace("**", "")
    return re.sub(r"\s+", " ", text).strip()


def _context(line: str, limit: int = 160) -> str:
    text = _plain(line)
    if len(text) > limit:
        match = WORD.search(text)
        centre = match.start() if match else 0
        start = max(0, min(centre - limit // 2, len(text) - limit))
        cut = text[start:start + limit].strip()
        text = ("…" if start > 0 else "") + cut + ("…" if start + limit < len(text) else "")
    return text.replace("|", "\\|")


def _documents(root: Path) -> List[Path]:
    docs = root / "docs"
    if not docs.exists():
        return []
    out = []
    for path in sorted(docs.rglob("*.md")):
        rel = path.relative_to(root).as_posix()
        if not any(rel == ex or rel.startswith(ex) for ex in EXCLUDED):
            out.append(path)
    return out


def scan(root: Path = REPO) -> Register:
    definitions: Dict[str, List[Item]] = {}
    citations: Dict[str, List[Tuple[str, int]]] = {}
    occurrences: List[Occurrence] = []

    for path in _documents(root):
        rel = path.relative_to(root).as_posix()
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        in_fence = in_history = False
        header: Optional[List[str]] = None
        for number, raw in enumerate(lines, start=1):
            stripped = raw.strip()
            if stripped.startswith(FENCE):
                in_fence = not in_fence
                continue
            is_row = stripped.startswith("|")
            if not is_row:
                header = None
            if not in_fence:
                heading = HEADING.match(stripped)
                if heading:
                    in_history = bool(HISTORY.search(heading.group(1)))
                    continue
                label = LABEL.match(stripped)
                if label and HISTORY.search(label.group(1)):
                    in_history = True
                    continue
            if in_history:
                continue
            if is_row and SEPARATOR.match(stripped):
                continue
            if is_row and number < len(lines) and SEPARATOR.match(lines[number].strip()):
                header = _cells(stripped)          # table header row
                continue

            text = CATEGORY.sub(" ", raw)
            ids = list(ID.finditer(text))
            if ids:
                cells = _cells(stripped) if is_row else []
                first = cells[0].strip("* ") if cells else None
                for match in ids:
                    ident = match.group(0)
                    if ident == first and match is ids[0]:
                        closed = next((c.strip("* ") for c in cells[1:]
                                       if c.strip("* ").startswith("Closed")), None)
                        definitions.setdefault(ident, []).append(Item(
                            ident=ident, kind=ident[:3], code=match.group(1), file=rel,
                            line=number, text=_plain(cells[1]) if len(cells) > 1 else "",
                            owner=_value(cells, header, OWNER_COLUMNS),
                            needed_by=_value(cells, header, NEEDED_COLUMNS), closed=closed))
                    else:
                        citations.setdefault(ident, []).append((rel, number))
                continue        # TBD or TBR words on this line belong to the item

            words = WORD.findall(text)
            if words:
                occurrences.append(Occurrence(
                    file=rel, line=number,
                    counts={kind: words.count(kind) for kind in KINDS if kind in words},
                    text=_context(raw)))

    problems = []
    items: Dict[str, Item] = {}
    for ident in sorted(set(definitions) | set(citations)):
        defs = definitions.get(ident, [])
        places = [(d.file, d.line) for d in defs] + citations.get(ident, [])
        where = ", ".join(f"{f}:{n}" for f, n in sorted(places))
        if ID.match(ident).group(1) is None:
            problems.append(f"{ident} has no document code (use {ident[:3]}-<CODE>-NNN): {where}")
        if len(defs) > 1:
            problems.append(f"{ident} is defined {len(defs)} times: "
                            + ", ".join(f"{d.file}:{d.line}" for d in defs))
        if not defs:
            problems.append(f"{ident} is cited ({where}) but defined nowhere")
            continue
        item = defs[0]
        item.refs = sorted(citations.get(ident, []))
        items[ident] = item

    owners: Dict[str, set] = {}
    for defs in definitions.values():
        for d in defs:
            if d.code:
                owners.setdefault(d.code, set()).add(d.file)
    for code, files in sorted(owners.items()):
        if len(files) > 1:
            problems.append(f"code {code} is used by items defined in {len(files)} documents: "
                            + ", ".join(sorted(files)))
    return Register(items=items, occurrences=occurrences, problems=problems)


def counts(reg: Register) -> Dict[str, int]:
    """Open TBD and TBR: one per open tracked item plus one per untracked word."""
    out = dict.fromkeys(KINDS, 0)
    for item in reg.items.values():
        if not item.closed:
            out[item.kind] += 1
    for occ in reg.occurrences:
        for kind, n in occ.counts.items():
            out[kind] += n
    return out


def _kind(occ: Occurrence) -> str:
    return ", ".join(kind if n == 1 else f"{kind} ×{n}" for kind, n in sorted(occ.counts.items()))


def render(reg: Register) -> str:
    total = counts(reg)
    open_items = sorted((i for i in reg.items.values() if not i.closed),
                        key=lambda i: (i.code or "", i.ident))
    closed_items = sorted((i for i in reg.items.values() if i.closed),
                          key=lambda i: (i.code or "", i.ident))
    by_file: Dict[str, List[Occurrence]] = {}
    for occ in reg.occurrences:
        by_file.setdefault(occ.file, []).append(occ)

    out = [
        "# SMART-QSO TBD/TBR Register",
        "",
        "**Generated by `scripts/tbd_register.py` from the documents in docs/. Do not edit this file: change the "
        "owning document and run the script.**",
        "",
        f"Open: **{total['TBD']} TBD** and **{total['TBR']} TBR** (finding VV-17). {len(open_items)} of them are "
        f"tracked items with an ID (§2); the other {sum(total.values()) - len(open_items)} are untracked words in "
        f"{len(by_file)} documents (§4). Closed tracked items: {len(closed_items)} (§3).",
        "",
        "## 1. Rules",
        "",
        "- Counted: the words TBD and TBR in docs/, and tracked items. Not counted: headings, table header rows, "
        "revision histories, the category name \"TBD/TBR\", and docs/funding_audit/ (which quotes them as "
        "evidence).",
        "- A tracked item has an ID TBD-<CODE>-NNN or TBR-<CODE>-NNN. CODE names the one document that owns it, "
        "and the item is defined by the row of that document's TBD/TBR table whose first cell is the ID. "
        "Citations elsewhere point to it. An ID is never reused, including after its item is closed.",
        "- To track an item, add a row to the owning document's TBD/TBR table with the next free number for that "
        "document's code, then run the script. To close one, keep its row and put \"Closed (date): value and "
        "source\" in a cell.",
        "- Owner and Needed by come from the owning table where it gives them; PI otherwise (the PI assigns).",
        "- CI (`scripts/tbd_register.py --check`) fails if this register is stale, an ID is reused, has no "
        "document code or is cited but not defined, or the open counts differ from `scripts/tbd_baseline.json`. "
        "Closing an item must lower the baseline in the same change.",
        "",
        "## 2. Tracked Items",
        "",
        "| Kind | ID | Item | Defined in | Owner | Needed by | Also cited at |",
        "|------|----|------|------------|-------|-----------|---------------|",
    ]
    for item in open_items:
        cited = ", ".join(f"`{f}`" for f in sorted({f for f, _ in item.refs})) or "—"
        out.append(f"| {item.kind} | {item.ident} | {item.text.replace('|', chr(92) + '|')} | "
                   f"`{item.file}` | {item.owner} | {item.needed_by} | {cited} |")
    out += ["", "## 3. Closed Tracked Items", ""]
    if closed_items:
        out += ["| Kind | ID | Item | Defined in | Closure |",
                "|------|----|------|------------|---------|"]
        for item in closed_items:
            out.append(f"| {item.kind} | {item.ident} | {item.text} | `{item.file}` | "
                       f"{_plain(item.closed).replace('|', chr(92) + '|')} |")
    else:
        out.append("None.")
    out += ["", "## 4. Untracked Occurrences", "",
            "One row per line that has the words, in document order; Kind gives the number of words on the line "
            "when it is more than one. Line numbers are left out so that unrelated edits do not make this file "
            "stale.", "",
            "| Document | TBD | TBR |", "|----------|-----|-----|"]
    for rel, occs in sorted(by_file.items()):
        tbd = sum(o.counts.get("TBD", 0) for o in occs)
        tbr = sum(o.counts.get("TBR", 0) for o in occs)
        out.append(f"| `{rel}` | {tbd} | {tbr} |")
    for rel, occs in sorted(by_file.items()):
        out += ["", f"### `{rel}`", "",
                "| Kind | Text | Owner | Needed by |",
                "|------|------|-------|-----------|"]
        for occ in occs:
            out.append(f"| {_kind(occ)} | {occ.text} | {UNKNOWN} | {UNKNOWN} |")
    return "\n".join(out) + "\n"


def compare(current: Dict[str, int], baseline_path: Path) -> List[str]:
    """Counts that differ from the baseline, one line each."""
    baseline = json.loads(baseline_path.read_text())
    out = []
    for kind in KINDS:
        if current[kind] > baseline[kind]:
            out.append(f"{kind}: {current[kind]}, baseline {baseline[kind]}")
        elif current[kind] < baseline[kind]:
            out.append(f"{kind}: {current[kind]}, baseline {baseline[kind]} (lower the baseline)")
    return out


def check(root: Path = REPO, reg: Optional[Register] = None) -> List[str]:
    """One line per problem; empty if the register is current and the counts are at baseline."""
    reg = reg or scan(root)
    problems = list(reg.problems)
    register = root / REGISTER
    if not register.exists() or register.read_text(encoding="utf-8") != render(reg):
        problems.append(f"{REGISTER} is stale: run python3 scripts/tbd_register.py")
    baseline = root / BASELINE
    if baseline.exists():
        problems += compare(counts(reg), baseline)
    else:
        problems.append(f"{BASELINE} not found")
    return problems


def main(argv: Optional[List[str]] = None, root: Path = REPO) -> int:
    parser = argparse.ArgumentParser(description="TBD/TBR register (VV-17)")
    parser.add_argument("--check", action="store_true",
                        help="fail if the register is stale, an ID is reused or the counts moved")
    args = parser.parse_args(argv)

    reg = scan(root)
    total = counts(reg)
    tracked = sum(1 for i in reg.items.values() if not i.closed)
    print(f"open: {total['TBD']} TBD, {total['TBR']} TBR ({tracked} tracked by ID; "
          f"{len(reg.items) - tracked} closed)")
    if args.check:
        problems = check(root, reg)
        for line in problems:
            print(f"::error::{line}")
        return 1 if problems else 0
    (root / REGISTER).write_text(render(reg), encoding="utf-8")
    print(f"wrote {REGISTER}")
    for line in reg.problems:
        print(line)
    return 1 if reg.problems else 0


if __name__ == "__main__":
    sys.exit(main())
