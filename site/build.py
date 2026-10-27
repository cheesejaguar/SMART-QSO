#!/usr/bin/env python3
"""Build the SMART-QSO public website from the repository documents.

Standard library only, so CI needs nothing installed:

    python3 site/build.py                       # writes site/_build/
    python3 site/build.py --out /tmp/site       # another output directory
    python3 site/build.py --base /SMART-QSO/ --site-url https://cheesejaguar.github.io/SMART-QSO/

The numbers on the site come from the documents at build time: the margins table, the RID
summary, the launch chain and the timeline of the CDR 2026 review; the dated actions and the
milestones of the integrated master schedule; the headline of the mission simulation; the key
values of the baseline; the authorization status of the spectrum document. Hand-written prose
cites its document, and every cited claim carries a short phrase that must still appear in that
document (``Cite``), so a sentence cannot outlive the value it repeats. If a table, a section
or a phrase the build needs is missing, the build stops with a message naming the document and
what it looked for. It never publishes a guess.

Pages link to each other with relative URLs, so the output can be previewed from any directory
(``python3 -m http.server -d site/_build``). The base path is used only where an absolute URL is
needed: 404.html (served at any depth by GitHub Pages), the canonical links and the sitemap.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import hashlib
import html
import json
import os
import posixpath
import re
import shutil
import subprocess
import sys
from pathlib import Path

SITE_DIR = Path(__file__).resolve().parent
REPO = SITE_DIR.parent
GITHUB = "https://github.com/cheesejaguar/SMART-QSO"
BLOB = GITHUB + "/blob/main/"
TREE = GITHUB + "/tree/main/"
DEFAULT_BASE = "/SMART-QSO/"
DEFAULT_URL = "https://cheesejaguar.github.io/SMART-QSO/"

# Documents the site reads. Each is hashed into publication.json.
BASELINE = "docs/BASELINE_PARAMETERS.md"
CDR = "docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md"
SCHEDULE = "mission/SCHEDULE.md"
CONOPS = "mission/CONOPS.md"
TIMELINE = "docs/operations/MISSION_TIMELINE.md"
SIL = "docs/test/SIL_MISSION_SIMULATION.md"
SPECTRUM = "docs/regulatory/SPECTRUM_LICENSE.md"
IARU = "docs/regulatory/IARU_COORDINATION.md"
ROADMAP = "docs/regulatory/REGULATORY_ROADMAP.md"
MECH = "hardware/mechanical/MECHANICAL_DESIGN.md"
BEACON = "docs/BEACON_SPEC.md"
DRAWINGS = {
    "drawing_deployed.svg": "hardware/mechanical/drawing_deployed.svg",
    "drawing_stowed_3view.svg": "hardware/mechanical/drawing_stowed_3view.svg",
}

PAGES = [  # (output path, nav label)
    ("index.html", "Home"),
    ("mission/index.html", "Mission"),
    ("spacecraft/index.html", "Spacecraft"),
    ("engineering/index.html", "Engineering"),
    ("participate/index.html", "Participate"),
]


class SourceError(Exception):
    """A document the site needs is missing something; the build must stop."""


# ----------------------------------------------------------------------------------------------
# Reading documents
# ----------------------------------------------------------------------------------------------

_cache: dict[str, str] = {}
used_sources: set[str] = set()


def read(path: str) -> str:
    if path not in _cache:
        full = REPO / path
        if not full.is_file():
            raise SourceError(f"{path}: file not found")
        _cache[path] = full.read_text(encoding="utf-8")
        used_sources.add(path)
    return _cache[path]


def norm(text: str) -> str:
    """Text for phrase checks: no Markdown emphasis or code marks, single spaces."""
    text = text.replace("**", "").replace("`", "")
    return re.sub(r"\s+", " ", text)


def require(path: str, phrase: str) -> None:
    if norm(phrase) not in norm(read(path)):
        raise SourceError(f"{path}: the phrase {phrase!r} that the site cites is no longer in the document")


def header_field(path: str, *names: str) -> str | None:
    """A front-matter value such as **Version**: B or **Revision:** E — text."""
    for line in read(path).splitlines()[:25]:
        for name in names:
            m = re.match(r"^\*\*" + re.escape(name) + r"(?::\*\*|\*\*\s*:)\s*(.+)$", line.strip())
            if m:
                return m.group(1).strip()
    return None


def doc_id(path: str) -> str | None:
    return header_field(path, "Document ID", "Document Number")


def doc_rev(path: str) -> str | None:
    rev = header_field(path, "Version", "Revision")
    if rev:
        rev = re.split(r"\s+[—-]\s+", rev, maxsplit=1)[0].strip("* ")
    return rev


def doc_date(path: str) -> str | None:
    return header_field(path, "Date")


def section(path: str, heading: str) -> str:
    """The body under the first heading whose text starts with ``heading``, up to the next
    heading of the same or a higher level."""
    lines = read(path).splitlines()
    start = level = None
    for i, line in enumerate(lines):
        m = re.match(r"^(#{1,6})\s+(.*)$", line)
        if not m:
            continue
        if start is None:
            if m.group(2).strip().startswith(heading):
                start, level = i + 1, len(m.group(1))
        elif len(m.group(1)) <= level:
            return "\n".join(lines[start:i])
    if start is None:
        raise SourceError(f"{path}: section {heading!r} not found")
    return "\n".join(lines[start:])


def split_row(line: str) -> list[str]:
    line = line.strip()
    if line.startswith("|"):
        line = line[1:]
    if line.endswith("|") and not line.endswith("\\|"):
        line = line[:-1]
    cells, cur, code = [], [], False
    i = 0
    while i < len(line):
        ch = line[i]
        if ch == "\\" and i + 1 < len(line) and line[i + 1] == "|":
            cur.append("|")
            i += 2
            continue
        if ch == "`":
            code = not code
        if ch == "|" and not code:
            cells.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
        i += 1
    cells.append("".join(cur).strip())
    return cells


def tables(text: str) -> list[tuple[list[str], list[list[str]]]]:
    out, block = [], []
    for line in text.splitlines() + [""]:
        if line.strip().startswith("|"):
            block.append(line)
            continue
        if len(block) >= 2 and re.match(r"^\|?\s*:?-{2,}", block[1].strip()):
            head = split_row(block[0])
            rows = [split_row(b) for b in block[2:]]
            out.append((head, rows))
        block = []
    return out


def table(path: str, text: str, columns: list[str], what: str) -> list[dict[str, str]]:
    """The first table in ``text`` whose header starts with ``columns``, as dicts."""
    for head, rows in tables(text):
        if [h.strip() for h in head[: len(columns)]] == columns:
            result = []
            for r in rows:
                r = (r + [""] * len(head))[: len(head)]
                result.append(dict(zip(head, r)))
            if not result:
                raise SourceError(f"{path}: {what}: the table has no rows")
            return result
    raise SourceError(f"{path}: {what}: no table with the columns {columns}")


def grab(path: str, text: str, pattern: str, what: str, flags: int = 0) -> re.Match:
    m = re.search(pattern, text, flags)
    if not m:
        raise SourceError(f"{path}: {what}: pattern {pattern!r} not found")
    return m


def baseline_rows() -> dict[str, str]:
    rows: dict[str, str] = {}
    for head, body in tables(read(BASELINE)):
        if head[:2] == ["Parameter", "Baseline"]:
            for r in body:
                if len(r) >= 2:
                    rows[re.sub(r"\*", "", r[0]).strip()] = r[1]
    return rows


def brow(rows: dict[str, str], name: str) -> str:
    if name not in rows:
        raise SourceError(f"{BASELINE}: no baseline row named {name!r}")
    return rows[name]


# ----------------------------------------------------------------------------------------------
# Markdown to HTML (the subset the documents use)
# ----------------------------------------------------------------------------------------------

def gh_url(repo_path: str) -> str | None:
    """GitHub URL for a path in the tree, or None if it does not exist."""
    repo_path = repo_path.strip("/")
    if not repo_path or repo_path.startswith(".."):
        return None
    full = REPO / repo_path
    if full.is_file():
        return BLOB + repo_path
    if full.is_dir():
        return TREE + repo_path
    return None


REPO_PATH = re.compile(r"^(?:docs|software|hardware|mission|scripts|site)/[A-Za-z0-9_./-]+$")


def inline(text: str, doc: str | None = None) -> str:
    """Inline Markdown: code spans, links, bold, italics. Relative links resolve against the
    document's directory and become GitHub links only when the target exists."""
    out = []
    for i, part in enumerate(re.split(r"(`[^`]*`)", text)):
        if i % 2 == 1:
            code = part[1:-1]
            esc = f"<code>{html.escape(code)}</code>"
            target = code.split("#")[0].split(" ")[0]
            url = gh_url(target) if REPO_PATH.match(target) else None
            out.append(f'<a href="{url}">{esc}</a>' if url else esc)
            continue
        s = html.escape(part, quote=False)

        def link(m: re.Match) -> str:
            label, target = m.group(1), html.unescape(m.group(2))
            if re.match(r"^(https?:|mailto:)", target):
                return f'<a href="{html.escape(target)}">{label}</a>'
            if target.startswith("#") or doc is None:
                return label
            path = posixpath.normpath(posixpath.join(posixpath.dirname(doc), target.split("#")[0]))
            url = gh_url(path)
            return f'<a href="{url}">{label}</a>' if url else label

        s = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", link, s)
        s = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", s)
        s = re.sub(r"(?<![\w*])\*(?!\s)([^*]+?)(?<!\s)\*(?![\w*])", r"<em>\1</em>", s)
        out.append(s)
    return "".join(out)


def blocks(text: str, doc: str) -> str:
    """Block Markdown: paragraphs, nested lists, tables, fences. HTML comments are dropped."""
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    lines = text.splitlines()
    out: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip() or line.strip() == "---":
            i += 1
            continue
        if line.strip().startswith("```"):
            j = i + 1
            while j < len(lines) and not lines[j].strip().startswith("```"):
                j += 1
            out.append("<pre><code>" + html.escape("\n".join(lines[i + 1:j])) + "</code></pre>")
            i = j + 1
            continue
        if line.strip().startswith("|"):
            j = i
            while j < len(lines) and lines[j].strip().startswith("|"):
                j += 1
            for head, rows in tables("\n".join(lines[i:j])):
                out.append(render_table(head, rows, doc))
            i = j
            continue
        m = re.match(r"^(#{1,6})\s+(.*)$", line)
        if m:
            out.append(f"<h4>{inline(m.group(2), doc)}</h4>")
            i += 1
            continue
        if re.match(r"^\s*(?:[-*]|\d+\.)\s+", line):
            j = i
            while j < len(lines) and lines[j].strip() and (
                re.match(r"^\s*(?:[-*]|\d+\.)\s+", lines[j]) or lines[j].startswith("  ")
            ):
                j += 1
            out.append(render_list(lines[i:j], doc))
            i = j
            continue
        j = i
        para = []
        while j < len(lines) and lines[j].strip() and not lines[j].strip().startswith(("|", "```", "#")) \
                and not re.match(r"^\s*(?:[-*]|\d+\.)\s+", lines[j]):
            para.append(lines[j].strip())
            j += 1
        out.append(f"<p>{inline(' '.join(para), doc)}</p>")
        i = j
    return "\n".join(out)


def render_list(lines: list[str], doc: str) -> str:
    items: list[tuple[int, bool, str]] = []
    for line in lines:
        m = re.match(r"^(\s*)([-*]|\d+\.)\s+(.*)$", line)
        if m:
            items.append((len(m.group(1)), m.group(2)[0].isdigit(), m.group(3)))
        elif items:
            ind, ordered, txt = items[-1]
            items[-1] = (ind, ordered, txt + " " + line.strip())

    def build(idx: int, indent: int) -> tuple[str, int]:
        ordered = items[idx][1]
        tag = "ol" if ordered else "ul"
        parts = [f"<{tag}>"]
        while idx < len(items) and items[idx][0] >= indent:
            ind, _, txt = items[idx]
            if ind > indent:
                sub, idx = build(idx, ind)
                parts[-1] = parts[-1].replace("</li>", sub + "</li>")
                continue
            parts.append(f"<li>{inline(txt, doc)}</li>")
            idx += 1
        parts.append(f"</{tag}>")
        return "".join(parts), idx

    html_out, idx = "", 0
    while idx < len(items):
        frag, idx = build(idx, items[idx][0])
        html_out += frag
    return html_out


def render_table(head: list[str], rows: list[list[str]], doc: str, caption: str | None = None,
                 cls: str = "") -> str:
    h = "".join(f'<th scope="col">{inline(c, doc)}</th>' for c in head)
    body = []
    for r in rows:
        r = (r + [""] * len(head))[: len(head)]
        cells = [f'<th scope="row">{inline(r[0], doc)}</th>'] + [f"<td>{inline(c, doc)}</td>" for c in r[1:]]
        body.append("<tr>" + "".join(cells) + "</tr>")
    if len(head) >= 4:
        cls = (cls + " wide").strip()
    cap = f"<caption>{caption}</caption>" if caption else ""
    label = html.escape(re.sub(r"<[^>]+>", "", caption or "Table"), quote=True)
    return (f'<div class="table-wrap" role="region" aria-label="{label}" tabindex="0">'
            f'<table class="{cls}">{cap}<thead><tr>{h}</tr></thead><tbody>{"".join(body)}</tbody></table></div>')


def dict_table(rows: list[dict[str, str]], columns: list[str], doc: str, caption: str | None = None,
               cls: str = "") -> str:
    return render_table(columns, [[r.get(c, "") for c in columns] for r in rows], doc, caption, cls)


# ----------------------------------------------------------------------------------------------
# Citations: a link to the document and a phrase that must still be in it
# ----------------------------------------------------------------------------------------------

def cite(path: str, *phrases: str, label: str | None = None, where: str = "") -> str:
    for p in phrases:
        require(path, p)
    read(path)
    url = gh_url(path)
    if url is None:
        raise SourceError(f"{path}: cited, but not in the tree")
    name = label or doc_id(path) or posixpath.basename(path)
    loc = f" {where}" if where else ""
    return f'<a class="src" href="{url}">{html.escape(name)}{html.escape(loc)}</a>'


def esc(s: str) -> str:
    return html.escape(s, quote=False)


# ----------------------------------------------------------------------------------------------
# Extraction
# ----------------------------------------------------------------------------------------------

def extract() -> dict:
    d: dict = {}
    b = baseline_rows()
    d["baseline_rev"] = doc_rev(BASELINE)
    d["baseline_date"] = doc_date(BASELINE)

    ff = brow(b, "Form factor")
    d["envelope"] = grab(BASELINE, ff, r"1U, ([\d.]+ × [\d.]+ × [\d.]+ mm)", "form factor").group(1)
    mass = brow(b, "Mass")
    d["mass_cbe"] = grab(BASELINE, mass, r"CBE \*\*([\d.]+) kg\*\*", "mass CBE").group(1)
    d["mass_mev"] = grab(BASELINE, mass, r"MEV \*\*([\d.]+) kg\*\*", "mass MEV").group(1)
    d["mass_limit"] = grab(BASELINE, mass, r"Design limit \*\*([\d.]+) kg\*\*", "mass limit").group(1)
    d["mass_margin"] = grab(BASELINE, mass, r"margin \*\*\+([\d.]+) %\*\*", "mass margin").group(1)
    alt = brow(b, "Deployment altitude")
    m = grab(BASELINE, alt, r"Target (\d+) km; acceptable (\d+)–(\d+) km; not above (\d+) km", "altitude band")
    d["alt_target"], d["alt_lo"], d["alt_hi"], d["alt_cap"] = m.groups()
    d["inclination"] = grab(BASELINE, brow(b, "Orbit"), r"inclination ≈ ([\d.]+)°", "inclination").group(1)
    life = brow(b, "Design life")
    d["design_life"] = grab(BASELINE, life, r"\*\*(\d+) years\*\*", "design life").group(1)
    d["min_mission"] = grab(BASELINE, life, r"minimum mission \*\*(\d+) months\*\*", "minimum mission").group(1)
    tx = brow(b, "Transmitter power")
    m = grab(BASELINE, tx, r"\*\*Maximum ([\d.]+) W\*\*, nominal ([\d.]+) W", "transmitter power")
    d["tx_max"], d["tx_nom"] = m.groups()
    d["battery_wh"] = grab(BASELINE, brow(b, "Battery"), r"\*\*≈ (\d+) Wh\*\*", "battery energy").group(1)
    gen = brow(b, "Generation, EOL orbit average delivered to the bus")
    m = grab(BASELINE, gen, r"\*\*([\d.]+) W tumbling; ([\d.]+) W sun-pointed\*\*", "generation")
    d["gen_tumble"], d["gen_sun"] = m.groups()
    d["downlink"] = grab(BASELINE, brow(b, "Downlink"), r"(145\.\d+–146\.\d+ MHz)", "downlink band").group(1)
    d["uplink_windows"] = grab(BASELINE, brow(b, "Uplink"), r"requested in \*\*([^*]+)\*\*", "uplink windows").group(1)
    d["callsign"] = grab(BASELINE, brow(b, "Space station licensee / call sign"),
                         r"identifies as \*\*([A-Z0-9]+-\d+)\*\*", "call sign").group(1)
    d["phase"] = grab(BASELINE, brow(b, "Project phase"), r"\*\*(Phase [A-Z] \([^)]+\))\.?\*\*", "phase").group(1)

    # CDR report
    d["cdr_id"], d["cdr_rev"], d["cdr_date"] = doc_id(CDR), doc_rev(CDR), doc_date(CDR)
    d["cdr_status"] = header_field(CDR, "Status") or ""
    chain = table(CDR, section(CDR, "3.1"), ["Step", "Who", "What (source)"], "launch chain (§3.1)")
    d["chain"] = chain
    steps = {r["Step"]: r for r in chain}
    for need in ("Launch vehicle", "Integrator", "Dispenser", "Planning launch", "Orbit"):
        if need not in steps:
            raise SourceError(f"{CDR}: §3.1 launch chain has no {need!r} row")
    d["launch_date"] = grab(CDR, steps["Planning launch"]["Who"], r"about (\d{4}-\d{2}-\d{2})", "planning date").group(1)
    d["ltan"] = grab(CDR, steps["Orbit"]["Who"], r"LTAN (\d{2}:\d{2})", "LTAN").group(1)
    d["timeline"] = table(CDR, section(CDR, "3.2"), ["Time", "Event", "Evidence"], "timeline (§3.2)")
    margins = table(CDR, section(CDR, "3.3"), ["Quantity", "Requirement", "Predicted", "Margin", "Source"],
                    "margins (§3.3)")
    if len(margins) < 10:
        raise SourceError(f"{CDR}: §3.3 margins table has only {len(margins)} rows")
    d["margins"] = margins
    d["decisions"] = table(CDR, section(CDR, "4. Chair decisions"), ["#", "Decision", "RIDs"], "chair decisions (§4)")
    d["design_items"] = table(CDR, section(CDR, "5. Design completion"),
                              ["Item", "Document", "What exists", "Key results"], "design completion (§5)")
    d["design_missing"] = grab(CDR, section(CDR, "5. Design completion"),
                               r"(What the design does not yet have:.*?)(?:\n\n|$)", "design gaps", re.S).group(1)

    # RID summary
    text = read(CDR)
    m = grab(CDR, text, r"<!-- RID-SUMMARY-BEGIN -->(.*?)<!-- RID-SUMMARY-END -->", "RID summary block", re.S)
    rs = m.group(1)
    m = grab(CDR, rs, r"(\d+) RIDs: severity 1: (\d+), severity 2: (\d+), severity 3: (\d+)", "RID totals")
    total, s1, s2, s3 = map(int, m.groups())
    disp = table(CDR, rs, ["Disposition", "Round 1", "Final"], "RID dispositions")
    rid = {"total": total, "severity": {"1": s1, "2": s2, "3": s3}, "dispositions": []}
    for r in disp:
        rid["dispositions"].append({"disposition": r["Disposition"], "round_1": int(r["Round 1"]),
                                    "final": int(r["Final"])})
    for key in ("round_1", "final"):
        ssum = sum(x[key] for x in rid["dispositions"])
        if ssum != total:
            raise SourceError(f"{CDR}: RID summary: the {key} dispositions add to {ssum}, not {total}")
    if s1 + s2 + s3 != total:
        raise SourceError(f"{CDR}: RID summary: severities add to {s1 + s2 + s3}, not {total}")
    rid["open_implementation"] = grab(CDR, rs, r"Open implementation RIDs: ([^\n]+)", "open RIDs").group(1).strip()
    rid["reopened"] = grab(CDR, rs, r"(Rounds 3 and 4[^\n]+)", "reopened sentence").group(1).strip()
    d["rid"] = rid

    v = section(CDR, "7. Verdict")
    d["closed_means"] = table(CDR, v, ["Kind", "Count", "What remains"], "what closed means (§7.1)")
    d["pi_actions"] = table(CDR, section(CDR, "7.3"), ["By", "Action"], "PI actions (§7.3)")
    d["maturity_md"] = section(CDR, "7.2")
    d["risks_md"] = section(CDR, "7.4")
    d["nature_md"] = section(CDR, "1.2")
    if "not an independent review board" not in norm(d["nature_md"]):
        raise SourceError(f"{CDR}: §1.2 no longer says the review is not an independent review board")
    if "strong PDR" not in norm(d["maturity_md"]):
        raise SourceError(f"{CDR}: §7.2 no longer states the maturity as a strong PDR")
    d["verdict_lead"] = grab(CDR, v, r"\*\*(No review action is open\.)\*\*", "verdict").group(1)

    # Schedule
    d["sch_id"], d["sch_rev"] = doc_id(SCHEDULE), doc_rev(SCHEDULE)
    actions = table(SCHEDULE, section(SCHEDULE, "5."),
                    ["ID", "Decision or action", "Owner", "Due", "Pass criterion or outcome", "Parent RID", "Status"],
                    "dated actions (§5)")
    for a in actions:
        if not re.match(r"^DA-\d+$", a["ID"]):
            raise SourceError(f"{SCHEDULE}: §5: unexpected action ID {a['ID']!r}")
        if not re.search(r"\d{4}-\d{2}-\d{2}", a["Due"]):
            raise SourceError(f"{SCHEDULE}: §5: {a['ID']} has no dated due date")
    d["actions"] = actions
    d["da01_due"] = next(a["Due"] for a in actions if a["ID"] == "DA-01") if any(
        a["ID"] == "DA-01" for a in actions) else None
    if not d["da01_due"]:
        raise SourceError(f"{SCHEDULE}: §5 has no DA-01 (ratification of the chair decisions)")
    d["milestones"] = table(SCHEDULE, section(SCHEDULE, "4. Milestone Summary"), ["Milestone", "Date"],
                            "milestones (§4)")

    # Mission timeline phases, ConOps modes
    d["phases"] = table(TIMELINE, section(TIMELINE, "2. Mission phases"), ["Phase", "From", "To", "Duration"],
                        "mission phases (§2)")
    modes = []
    for line in section(CONOPS, "3. Modes").splitlines():
        m = re.match(r"^-\s+([^:]+):\s*(.+)$", line)
        if m:
            modes.append((m.group(1).strip(), m.group(2).strip()))
    if len(modes) < 5:
        raise SourceError(f"{CONOPS}: §3 modes list has {len(modes)} entries")
    d["modes"] = modes

    # Mission simulation headline
    s7 = section(SIL, "7. Results")
    d["sil_rev"] = doc_rev(SIL)
    m = grab(SIL, s7, r"found \*\*(\d+) anomalies and (\d+) warnings in (\d+) checks\*\*", "SIL headline")
    d["sil_anomalies"], d["sil_warnings"], d["sil_checks"] = m.groups()
    d["sil_days"] = grab(SIL, s7, r"(\d+) days of operations", "SIL duration").group(1)
    d["sil_wh_beacon"] = grab(SIL, s7, r"\*\*([\d.]+) Wh per\s+accepted beacon\*\*", "SIL AI energy").group(1)
    d["sil_dod"] = grab(SIL, s7, r"depth of discharge ([\d.]+) %", "SIL depth of discharge").group(1)
    m = grab(SIL, s7, r"([\d.]+) % of ([\d,]+) passes heard", "SIL passes heard")
    d["sil_heard"], d["sil_passes"] = m.groups()

    # Regulatory status
    d["auth"] = table(SPECTRUM, section(SPECTRUM, "4. Authorization Status"),
                      ["Authorization", "Status", "Draft / reference"], "authorization status (§4)")
    d["auth_asof"] = grab(SPECTRUM, read(SPECTRUM), r"## 4\. Authorization Status \(as of (\d{4}-\d{2}-\d{2})\)",
                          "status date").group(1)
    iaru = table(IARU, section(IARU, "1. Current Status"), ["Item", "Status"], "IARU status (§1)")
    req = next((r for r in iaru if r["Item"] == "Coordination request"), None)
    if req is None:
        raise SourceError(f"{IARU}: §1 has no 'Coordination request' row")
    d["iaru_request"] = req["Status"]
    d["iaru_submitted"] = "not submitted" not in norm(req["Status"]).lower()
    d["roadmap_status"] = header_field(ROADMAP, "Status") or ""

    # Beacon format
    fmt = grab(BEACON, section(BEACON, "3. Information Field"), r"```\n(.*?)\n```", "beacon format", re.S).group(1)
    d["beacon_format"] = fmt.strip()

    # The deployed CAD sheet: does it still draw the former two-element antenna?
    svg = (REPO / DRAWINGS["drawing_deployed.svg"]).read_text(encoding="utf-8")
    used_sources.add(DRAWINGS["drawing_deployed.svg"])
    used_sources.add(DRAWINGS["drawing_stowed_3view.svg"])
    d["sheet2_two_elements"] = "AntS option A" in svg
    return d


# ----------------------------------------------------------------------------------------------
# Illustration: the 1U deployed, drawn from the mechanical design's dimensions
# ----------------------------------------------------------------------------------------------

def illustration() -> tuple[str, str]:
    """Oblique view from +Y (the sun side) and below, so the -Z antenna face shows. Body and wing
    dimensions from MECHANICAL_DESIGN §3.2, §4 and sheet 2: body 100 x 100 x 113.5 mm, wings
    81.6 x 92 mm in the plane y = 38.75 mm from the hinge axes at x = +-54.1 mm, z -43 to +49 mm;
    AntS on -Z with 2 m tapes along +-X (500 mm, drawn shortened) and 70 cm tapes along +-Y
    (170 mm, to scale)."""
    for p in ("100 × 100 × 113.5 mm", "81.6 × 92 mm", "2 m ±X, 70 cm ±Y"):
        require(BASELINE if "113.5" in p else MECH, p)
    c, k = 0.42, 0.30

    def P(x: float, y: float, z: float) -> tuple[float, float]:
        return (x - c * y, -z - k * y)

    def poly(pts, cls, extra=""):
        s = " ".join(f"{u:.1f},{v:.1f}" for u, v in (P(*p) for p in pts))
        return f'<polygon class="{cls}" points="{s}"{extra}/>'

    def line(a, b, cls):
        (u1, v1), (u2, v2) = P(*a), P(*b)
        return f'<line class="{cls}" x1="{u1:.1f}" y1="{v1:.1f}" x2="{u2:.1f}" y2="{v2:.1f}"/>'

    hx, hz = 50.0, 56.75
    zt = -52.0
    el2 = 250.0  # drawn length of a 2 m element (500 mm), shortened
    el7 = 170.0
    parts = []
    # far 70 cm element (-Y) first: the body hides its root
    parts.append(line((0, -49, zt), (0, -49 - el7, zt), "ant ant7"))
    # body: bottom (-Z), right (+X), front (+Y)
    parts.append(poly([(-hx, -hx, -hz), (hx, -hx, -hz), (hx, hx, -hz), (-hx, hx, -hz)], "body-bot"))
    parts.append(poly([(-49, -49, -hz - 0.1), (49, -49, -hz - 0.1), (49, 49, -hz - 0.1), (-49, 49, -hz - 0.1)],
                      "ants"))
    parts.append(poly([(hx, hx, -hz), (hx, -hx, -hz), (hx, -hx, hz), (hx, hx, hz)], "body-side"))
    # +X body cells
    parts.append(poly([(hx, 40, -41), (hx, -40, -41), (hx, -40, 39), (hx, 40, 39)], "cell"))
    parts.append(poly([(-hx, hx, -hz), (hx, hx, -hz), (hx, hx, hz), (-hx, hx, hz)], "body-front"))
    for x0 in (-40.0, 0.5):
        parts.append(poly([(x0, hx, -41), (x0 + 39.5, hx, -41), (x0 + 39.5, hx, 39), (x0, hx, 39)], "cell"))
    # rails at the four vertical edges seen
    for x, y in ((-hx, hx), (hx, hx), (hx, -hx)):
        parts.append(line((x, y, -hz), (x, y, hz), "rail"))
    # 2 m elements along +-X from the AntS box sides
    for s in (-1, 1):
        parts.append(line((s * 49, 0, zt), (s * (49 + el2), 0, zt), "ant ant2"))
        # break mark
        bx = s * (49 + el2 * 0.62)
        parts.append(line((bx - 5, 0, zt - 9), (bx + 5, 0, zt + 9), "brk"))
        parts.append(line((bx + 3, 0, zt - 9), (bx + 13, 0, zt + 9), "brk"))
    # wings in the plane y = 38.75, hinge at x = +-54.1
    yw = 38.75
    for s in (-1, 1):
        x0, x1 = s * 54.1, s * (54.1 + 81.6)
        parts.append(poly([(x0, yw, -43), (x1, yw, -43), (x1, yw, 49), (x0, yw, 49)], "wing"))
        a, b = (min(x0, x1) + 3, max(x0, x1) - 3)
        mid = (a + b) / 2
        parts.append(poly([(a, yw + 0.1, -40), (mid - 1, yw + 0.1, -40), (mid - 1, yw + 0.1, 41), (a, yw + 0.1, 41)],
                          "cell"))
        parts.append(poly([(mid + 1, yw + 0.1, -40), (b, yw + 0.1, -40), (b, yw + 0.1, 41), (mid + 1, yw + 0.1, 41)],
                          "cell"))
        parts.append(line((s * 50, yw, -43), (x0, yw, -43), "hinge"))
        parts.append(line((s * 50, yw, 49), (x0, yw, 49), "hinge"))
    # near 70 cm element (+Y)
    parts.append(line((0, 49, zt), (0, 49 + el7, zt), "ant ant7"))

    # callouts: (number, anchor point, label offset)
    callouts = [
        (1, (-20, 50, 30), (-70, -72), "1U body, 100 × 100 × 113.5 mm"),
        (2, (54.1 + 60, yw, 30), (55, -80), "Solar wing (one of two)"),
        (3, (49 + el2 * 0.9, 0, zt), (0, 46), "2 m element (≈ 0.5 m, shortened here)"),
        (4, (0, 49 + el7, zt), (-45, -85), "70 cm element (≈ 0.17 m)"),
        (5, (-25, 0, -hz), (-105, 62), "ISIS AntS on the −Z face"),
    ]
    marks = []
    for n, pt, (du, dv), _ in callouts:
        u, v = P(*pt)
        lu, lv = u + du, v + dv
        marks.append(f'<line class="lead" x1="{u:.1f}" y1="{v:.1f}" x2="{lu:.1f}" y2="{lv:.1f}"/>'
                     f'<circle class="dot" cx="{u:.1f}" cy="{v:.1f}" r="3"/>'
                     f'<circle class="badge" cx="{lu:.1f}" cy="{lv:.1f}" r="15"/>'
                     f'<text class="badge-t" x="{lu:.1f}" y="{lv + 6:.1f}" text-anchor="middle">{n}</text>')
    # axes hint
    ax = ('<g class="axes"><text x="-322" y="172">Seen from the sun side (+Y) and below: +Z up, +X right</text></g>')
    legend = "".join(f"<li><span class=\"badge-n\" aria-hidden=\"true\">{n}</span> {esc(t)}</li>"
                     for n, _, _, t in callouts)
    svg = (f'<svg class="sc-illus" viewBox="-330 -135 660 318" role="img" aria-labelledby="illus-t illus-d">'
           f'<title id="illus-t">SMART-QSO with its wings and antennas deployed</title>'
           f'<desc id="illus-d">A 1U cube seen from the sun side and slightly below. Two solar wings stand out '
           f'to the left and right from hinges at the edges of the side faces; from the antenna unit on the bottom '
           f'face, two long 2 metre band elements point left and right and two short 70 centimetre band elements '
           f'point toward and away from the viewer.</desc>'
           f'{"".join(parts)}{"".join(marks)}{ax}</svg>')
    return svg, legend


# ----------------------------------------------------------------------------------------------
# Page frame
# ----------------------------------------------------------------------------------------------

LOGO = ('<svg class="logo" viewBox="0 0 32 32" aria-hidden="true" focusable="false">'
        '<rect x="11" y="11" width="10" height="10" rx="1.5" class="logo-body"/>'
        '<rect x="2" y="12" width="8" height="8" class="logo-wing"/><rect x="22" y="12" width="8" height="8" class="logo-wing"/>'
        '<path d="M16 21v9M16 11V2" class="logo-ant"/></svg>')

FAVICON = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
<style>.b{fill:#e8edf3;stroke:#12355b;stroke-width:1.5}.w{fill:#1f4f8a}.a{stroke:#c2410c;stroke-width:2;stroke-linecap:round}
@media (prefers-color-scheme:dark){.b{fill:#1b2a3a;stroke:#9cc4f0}.w{fill:#5b8fd0}.a{stroke:#fb923c}}</style>
<rect x="11" y="11" width="10" height="10" rx="1.5" class="b"/><rect x="2" y="12" width="8" height="8" class="w"/>
<rect x="22" y="12" width="8" height="8" class="w"/><path d="M16 21v9M16 11V2" class="a"/></svg>
"""


class Site:
    def __init__(self, base: str, url: str, commit: str, built: str):
        self.base = base if base.endswith("/") else base + "/"
        self.url = url if url.endswith("/") else url + "/"
        self.commit = commit
        self.built = built

    def rel(self, page: str, target: str, absolute: bool = False) -> str:
        """Link from output page ``page`` to output path ``target`` (a directory index or a file)."""
        if absolute:
            return self.base + target.replace("index.html", "")
        depth = page.count("/")
        prefix = "../" * depth
        t = target.replace("index.html", "")
        return (prefix + t) or "./"

    def frame(self, page: str, title: str, description: str, body: str, absolute: bool = False) -> str:
        r = lambda t: self.rel(page, t, absolute)  # noqa: E731
        nav = []
        for path, label in PAGES:
            cur = ' aria-current="page"' if path == page else ""
            nav.append(f'<li><a href="{r(path)}"{cur}>{label}</a></li>')
        canonical = self.url + page.replace("index.html", "")
        canonical_tag = "" if page == "404.html" else f'<link rel="canonical" href="{canonical}">\n'
        full_title = "SMART-QSO" if page == "index.html" else f"{title} · SMART-QSO"
        short = self.commit[:10]
        return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{esc(full_title)}</title>
<meta name="description" content="{html.escape(description)}">
<meta name="color-scheme" content="light dark">
{canonical_tag}<link rel="icon" href="{r('favicon.svg')}" type="image/svg+xml">
<link rel="stylesheet" href="{r('assets/site.css')}">
</head>
<body>
<a class="skip" href="#main">Skip to content</a>
<header class="site-header">
<div class="wrap header-inner">
<a class="brand" href="{r('index.html')}">{LOGO}<span>SMART-QSO</span></a>
<nav aria-label="Main"><ul>{''.join(nav)}</ul></nav>
</div>
</header>
<main id="main" class="wrap">
{body}
</main>
<footer class="site-footer">
<div class="wrap">
<p>SMART-QSO is a design-phase amateur radio CubeSat project. Nothing has been built or launched. Every value on this
site is taken from, or checked against, the repository documents cited beside it, when the site is built.</p>
<p>Built from commit <a href="{GITHUB}/commit/{self.commit}"><code>{short}</code></a> ({self.built[:10]}) by
<a href="{BLOB}site/build.py"><code>site/build.py</code></a>. Data:
<a href="{r('evidence/margins.json')}">margins</a>, <a href="{r('evidence/rid_summary.json')}">RID summary</a>,
<a href="{r('evidence/schedule_actions.json')}">dated actions</a>, <a href="{r('publication.json')}">publication record</a>.
Source and documents: <a href="{GITHUB}">github.com/cheesejaguar/SMART-QSO</a> (MIT licence).</p>
</div>
</footer>
</body>
</html>
"""


# ----------------------------------------------------------------------------------------------
# Pages
# ----------------------------------------------------------------------------------------------

def status_box(d: dict) -> str:
    return f"""<section class="status" aria-labelledby="status-h">
<h2 id="status-h">Where the project stands</h2>
<ul>
<li><strong>Design phase.</strong> The project is in {esc(d['phase'])}. No hardware has been bought or built, and
every figure is a prediction, none a measurement. {cite(BASELINE, d['phase'], where='§1')}
{cite(CDR, 'Every figure is predicted, none measured', where='§7.2')}</li>
<li><strong>Software runs in simulation.</strong> The flight software (Rust, on the Embassy executor) runs as a host
simulation and builds as an STM32L4R5ZI image that CI checks; it has not run on hardware.
{cite(CDR, 'The firmware has not run on hardware', where='§7.2')}</li>
<li><strong>Reviewed, but not by an independent board.</strong> A design review against the NASA CDR criteria
finished on {esc(d['cdr_date'])}. It was carried out by AI reviewers, not an independent review board, and it rates
the project at the level of a strong PDR, not past a CDR. The independent PDR is planned for 2027-01-12.
{cite(CDR, 'not an independent review board', 'the independent PDR is planned for 2027-01-12', where='§1.2, §7.2')}</li>
<li><strong>No licence, no frequencies, no launch contract yet.</strong> No regulatory filing has been made; the
IARU coordination request is not submitted, so there are no coordinated frequencies. The launch contract is due by
2027-06-30. {cite(ROADMAP, 'No regulatory filing has been made as of this date.')}
{cite(SCHEDULE, 'deadline **2027-06-30**', where='§1')}</li>
</ul>
</section>"""


def page_home(site: Site, d: dict, illus: tuple[str, str]) -> str:
    svg, legend = illus
    r = lambda t: site.rel("index.html", t)  # noqa: E731
    stats = [
        ("1U", "CubeSat", f"{d['envelope']}", cite(BASELINE, d['envelope'], where='§4')),
        (f"{d['mass_cbe']} kg", "current best estimate",
         f"{d['mass_mev']} kg with growth; limit {d['mass_limit']} kg (+{d['mass_margin']} %)",
         cite(BASELINE, "Design limit", where='§4')),
        (f"{d['alt_target']} km", "sun-synchronous orbit",
         f"{d['inclination']}° inclination; cap {d['alt_cap']} km; LTAN {d['ltan']}",
         cite(BASELINE, f"Target {d['alt_target']} km", where='§3')),
        (f"{d['design_life']} years", "design life", f"minimum mission {d['min_mission']} months",
         cite(BASELINE, "Design life", where='§2')),
        (f"{d['tx_nom']} W", "downlink, nominal", f"{d['tx_max']} W maximum; 2 m band down, 70 cm up",
         cite(BASELINE, "Transmitter power", where='§7')),
        (f"{d['gen_sun']} W", "power, sun-pointed, end of life", f"{d['gen_tumble']} W tumbling; ≈ {d['battery_wh']} Wh battery",
         cite(BASELINE, "Generation, EOL orbit average", where='§5')),
        (d['launch_date'], "planning launch date", "the last Transporter rideshare of 2028; not contracted",
         cite(CDR, "Planning launch", where='§3.1')),
        (str(d['rid']['total']), "review findings (RIDs)",
         f"{d['rid']['open_implementation'].rstrip('.').lower()} open for implementation",
         cite(CDR, "RID-SUMMARY-BEGIN", where='§6')),
    ]
    cards = "".join(
        f'<div class="stat"><dt><span class="stat-v">{esc(v)}</span> <span class="stat-l">{esc(lab)}</span></dt>'
        f'<dd>{esc(note)} <span class="stat-src">{src}</span></dd></div>' for v, lab, note, src in stats)
    chain = [
        ("SpaceX Falcon 9", "A Transporter-class rideshare to sun-synchronous orbit from Vandenberg SLC-4E.",
         cite(CDR, "Vandenberg SLC-4E", where="§3.1")),
        ("Exolaunch", "The integrator: SpaceX's customer. SMART-QSO would contract with the integrator, not with SpaceX.",
         cite(CDR, "SMART-QSO contracts with the integrator, not with SpaceX", where="§3.1")),
        ("EXOpod Nova", "The dispenser. The CubeSat is loaded about 47 days before launch, unpowered, and is not touched again.",
         cite(CDR, "Exolaunch EXOpod Nova", "integration into the Nova at L-47 d", where="§3.1")),
        (f"{d['alt_target']} km SSO", f"Drop-off at a {d['alt_target']} km target, never above {d['alt_cap']} km, a limit the contract must state.",
         cite(CDR, "475 km target, 485 km hard cap", where="§3.1")),
        ("Re-entry", "No propulsion: the orbit decays naturally with the wings deployed, within 5 years of the end of the mission.",
         cite(BASELINE, "Passive atmospheric drag", "No separate drag device", where="§3")),
    ]
    chain_html = "".join(f'<li><h3>{esc(t)}</h3><p>{esc(p)} {s}</p></li>' for t, p, s in chain)
    body = f"""
<section class="hero">
<div class="hero-text">
<p class="eyebrow">Amateur radio CubeSat · design phase</p>
<h1>SMART-QSO</h1>
<p class="lede">A 1U CubeSat that will carry a single-channel FM transponder for amateur radio contacts
worldwide, switched on only when the power budget allows, and a 1200 bps AX.25 telemetry beacon. Its experiment is
onboard AI that schedules the transponder within the power budget and chooses the beacon's plain-language text.
If the AI is off or fails, it is a conventional transponder with template beacons.
{cite(BASELINE, 'U/V FM transponder', 'duty-gated', 'Falls back to a conventional transponder and template beacon', where='§2')}</p>
<p class="cta"><a class="btn" href="{r('mission/index.html')}">The mission</a>
<a class="btn btn-2" href="{r('engineering/index.html')}">The design review</a></p>
</div>
<figure class="hero-fig">
{svg}
<figcaption><ol class="fig-legend">{legend}</ol>
<p>Drawn from the mechanical design's dimensions; the 2 m elements are shortened to fit.
{cite(MECH, '2 m ±X, 70 cm ±Y', where='§3.2, §5.2')}</p></figcaption>
</figure>
</section>
{status_box(d)}
<section aria-labelledby="numbers-h">
<h2 id="numbers-h">Key numbers</h2>
<p class="muted">From the baseline, revision {esc(d['baseline_rev'] or '')} of {esc(d['baseline_date'] or '')}, which governs
every design value, and the CDR 2026 review report.</p>
<dl class="stats">{cards}</dl>
</section>
<section aria-labelledby="ai-h">
<h2 id="ai-h">What the AI does, and what it does not</h2>
<div class="cols">
<div><h3>Flight baseline: a selector on the flight computer</h3>
<p>A small calibrated model on the STM32L4R5 flight computer picks among pre-approved beacon templates and proposes
the power mode and the transponder schedule. Hard limits stay outside the model: no AI output can override the
power gates, the transmit inhibits or TX-OFF. It is designed, not yet implemented; today the flight software sends
its fixed templates in turn.
{cite(BASELINE, 'chooses among pre-approved templates', 'Designed, not yet implemented', where='§6')}
{cite(CONOPS, 'the selector and the transponder scheduler are not implemented', where='§5')}</p></div>
<div><h3>Gated experiment: generated text on the Jetson</h3>
<p>A power-gated NVIDIA Jetson Orin Nano module can generate beacon lines with a small open language model
(SmolLM2-360M or Qwen3-0.6B) in bursts of at most 10 minutes. It flies as an experiment only if the energy per
accepted beacon line, measured on the engineering model, is at most 0.1 Wh. Generated text carries no numbers and
passes the same content filter as the templates; the flight computer writes every value from measurements.
{cite(BASELINE, 'SmolLM2-360M-Instruct or Qwen3-0.6B', 'is ≤ 0.1 Wh (360 J)', 'no numbers in generated text', where='§6')}</p></div>
</div>
</section>
<section aria-labelledby="orbit-h">
<h2 id="orbit-h">How it reaches orbit</h2>
<p>The planned path, as the design review traced it. None of it is contracted yet: requests for quotes go out by
2026-12-31. {cite(SCHEDULE, 'Integrator requests for quotes out', where='§1')}</p>
<ol class="chain">{chain_html}</ol>
<p><a href="{r('mission/index.html')}">The launch chain in full, and the timeline from separation to re-entry →</a></p>
</section>
"""
    return site.frame("index.html", "Home",
                      "SMART-QSO: a 1U amateur radio CubeSat in design, with a power-gated FM transponder and an "
                      "onboard AI experiment. Honest status, key numbers and how it would reach orbit.", body)


def page_mission(site: Site, d: dict) -> str:
    steps = [
        ("L − 47 days", "Into the dispenser",
         "Final charge to 60 % at the integrator, key load, then into the EXOpod Nova. Unpowered from here: three "
         "deployment switches hold every circuit open, and nobody touches the spacecraft again.",
         cite(BASELINE, "60 % ± 5 % SOC", "three deployment switches on three rails", where="§4")),
        ("Launch", "Falcon 9 from Vandenberg",
         "A Transporter-class rideshare to sun-synchronous orbit; the dispenser opens on the integrator's signal.",
         cite(CDR, "Vandenberg SLC-4E", where="§3.1")),
        ("S", "Separation",
         "The switches close and the flight computer boots. The 30-minute deployment wait and the 45-minute radio "
         "silence start counting.",
         cite(CDR, "Switches close; the OBC boots", where="§3.2")),
        ("S + 120 s", "Detumble starts",
         "Nothing moves for the first 120 s (a launch-provider rule). Then the magnetorquers start slowing the "
         "spacecraft's tumble.",
         cite(BASELINE, "No magnetorquer actuation or deployment before 120 s", where="§4")),
        ("S + 30 min", "Wings, then antennas",
         "Burn resistors release the two solar wings, one resistor at a time. The antenna unit deploys next, but only "
         "once it reads −17 °C or warmer; after a cold eclipse separation it waits for the next 50-minute slot.",
         cite(CDR, "Wings deployed", "≥ −17 °C", where="§3.2, C-18")),
        ("S + 45 min", "First beacon",
         "The radio silence ends and is latched. The receiver comes on and the first beacon goes out, in IDLE mode "
         "with template text every 120 s, once the antennas are out.",
         cite(CDR, "RF silence ends and is latched", "first beacon in IDLE", where="§3.2")),
        ("S + ≈ 1 h", "Sun pointing",
         "Detumbled, the spacecraft spins at 8°/s about the axis that faces its wings to the Sun.",
         cite(CDR, "spin-stabilized sun pointing of +Y at 8°/s", where="§3.2")),
        ("S + 0.7–13.7 h", "First contact",
         "The first pass over the San Francisco Bay Area comes 5.3 h after separation in the median case; the first "
         "commanding pass 6.4 h after it (at most 13.7 h).",
         cite(CDR, "median 5.3 h", "first commanding pass median 6.4 h", where="§3.2")),
        ("Days 1–14", "Commissioning",
         "Authenticated telecommand checked from each station, TX-OFF proven, power and thermal characterised. Only "
         "then are the transponder and the Jetson released.",
         cite(CDR, "TX-OFF proven", "transponder and Jetson only after the release", where="§3.2")),
        (f"{d['design_life']} years", "Operations",
         "The transponder opens only in sunlight, with the battery at 55 % or more, for at most 25 % of an orbit. The "
         "beacon runs every 60 to 180 s depending on the power mode.",
         cite(CDR, "transponder duty-gated (sunlit, SOC ≥ 55 %, ≤ 25 % of an orbit)", where="§3.2")),
        ("End of mission", "Passivation",
         "A two-step authenticated command permanently disconnects the battery charge path while the transmitter still "
         "reports, sends a final frame, then latches the transmitter off.",
         cite(CDR, "Two-step passivation command", where="§3.2")),
        ("≤ 5 years after", "Re-entry",
         "Natural decay with the wings deployed, within the FCC's five-year rule. There is no propulsion and no "
         "separate drag device.",
         cite(CDR, "Re-entry by natural decay with the wings deployed", where="§3.2")),
    ]
    tl = "".join(f'<li><p class="tl-t">{esc(t)}</p><h3>{esc(h)}</h3><p>{esc(p)} {s}</p></li>'
                 for t, h, p, s in steps)
    modes = "".join(f"<li><strong>{inline(n, CONOPS)}</strong>: {inline(t, CONOPS)}</li>" for n, t in d["modes"])
    body = f"""
<h1>The mission</h1>
<p class="lede">SMART-QSO would ride a SpaceX Transporter rideshare to a {d['alt_target']} km sun-synchronous orbit,
serve amateur radio operators for {d['design_life']} years, then passivate and decay. This page follows it from the
dispenser to re-entry. It is a plan: no launch is contracted and nothing has been built.
{cite(CONOPS, 'Concept of Operations', label='Concept of operations')}</p>

<section aria-labelledby="chain-h">
<h2 id="chain-h">The launch chain</h2>
<p>As traced by the CDR 2026 design review. The planning date is the last Transporter flight of 2028, about
{esc(d['launch_date'])}; June 2028 is an opportunity only, because it misses the integrator's protoflight
milestone. {cite(CDR, where='§3.1', label=d['cdr_id'] or 'CDR report')}</p>
{dict_table(d['chain'], ['Step', 'Who', 'What (source)'], CDR, 'Launch chain (CDR report §3.1)')}
</section>

<section aria-labelledby="tl-h">
<h2 id="tl-h">From the dispenser to re-entry</h2>
<p>The concept of operations in plain language. Times after separation (S) come from the mission simulation and the
pass analysis. {cite(TIMELINE, where='§3–§7', label='Mission timeline')}</p>
<ol class="timeline">{tl}</ol>
<details>
<summary>The timeline as reviewed, minute by minute, with its evidence (CDR report §3.2)</summary>
{dict_table(d['timeline'], ['Time', 'Event', 'Evidence'], CDR, 'Minute-by-minute timeline (CDR report §3.2)')}
</details>
</section>

<section aria-labelledby="phases-h">
<h2 id="phases-h">Mission phases</h2>
{dict_table(d['phases'], ['Phase', 'From', 'To', 'Duration'], TIMELINE, 'Mission phases (mission timeline §2)')}
<h3>Operating modes</h3>
<p>The flight computer moves between modes on the battery's state of charge, sunlight, temperature and the watchdog;
faults drop to SAFE, and TX-OFF and the loss-of-contact inhibit override every mode.
{cite(CONOPS, 'Faults drop to SAFE', where='§3')}</p>
<ul class="modes">{modes}</ul>
<h3>Fail-graceful by design</h3>
<p>The Jetson is power-gated and used only in short bursts in sunlight. If it is off, faulty, or its text fails the
content filter, the beacon uses a template and the transponder runs on fixed rules.
{cite(CONOPS, 'the beacon uses a template and the transponder runs on fixed rules', where='§5')}</p>
</section>

<section aria-labelledby="road-h">
<h2 id="road-h">The road to launch</h2>
<p>Milestones from the integrated master schedule ({esc(d['sch_id'] or '')} version {esc(d['sch_rev'] or '')}), a
planning baseline from a critical-path model: no vendor quote, laboratory booking, integrator schedule or funding
commitment exists yet. {cite(SCHEDULE, 'No vendor quote, laboratory booking, integrator schedule or', where='§4')}</p>
{dict_table(d['milestones'], ['Milestone', 'Date'], SCHEDULE, 'Milestones (integrated master schedule §4)', 'compact')}
</section>
"""
    return site.frame("mission/index.html", "Mission",
                      "The SMART-QSO launch chain, the concept of operations from separation to re-entry, the mission "
                      "phases and the road to launch, from the repository documents.", body)


def page_spacecraft(site: Site, d: dict, illus: tuple[str, str]) -> str:
    svg, legend = illus
    r = lambda t: site.rel("spacecraft/index.html", t)  # noqa: E731
    subs = [
        ("Structure and mechanisms",
         f"A bought 1U frame, {d['envelope']}. Two solar wings on torsion-spring hinges, each held down by a line cut "
         f"by two redundant burn resistors on their own current-limited drivers. Mass {d['mass_cbe']} kg current best "
         f"estimate, {d['mass_mev']} kg with growth, against a {d['mass_limit']} kg limit.",
         [cite(BASELINE, "two burn resistors per wing", where="§4"), cite(MECH, "two 17-7PH torsion springs", where="§1")]),
        ("Power",
         f"16 triple-junction cells, two on each long body face and two on each side of the two wings. A 4S1P "
         f"Li-ion 18650 pack of ≈ {d['battery_wh']} Wh, charged only to 4.10 V per cell. End-of-life generation "
         f"{d['gen_sun']} W sun-pointed, {d['gen_tumble']} W tumbling.",
         [cite(BASELINE, "16 triple-junction cells", "charged to 4.10 V/cell", where="§5")]),
        ("Flight computer and software",
         "An STM32L4R5ZI microcontroller running all the flight software, written in Rust on the Embassy executor with "
         "no RTOS. It enforces every hard limit itself, whatever the AI proposes.",
         [cite(BASELINE, "STM32L4R5ZI", "Embassy executor, no RTOS", where="§6"),
          cite("software/rust/README.md", label="software/rust")]),
        ("AI payload",
         "An NVIDIA Jetson Orin Nano 8 GB module on a custom carrier in its 7 W mode, with no fan. Each power-on lasts at "
         "most 10 minutes including boot, then at least 100 minutes off, in sunlight only. The flight computer cuts its "
         "rail on its own current and temperature measurements.",
         [cite(BASELINE, "Jetson Orin Nano 8 GB module", "≥ 100 minutes off", where="§6")]),
        ("Radio",
         f"A single-channel FM transponder (70 cm up, 2 m down) and the 1200 bps AX.25 beacon, {d['tx_nom']} W "
         f"nominal and {d['tx_max']} W maximum. Telecommand is authenticated (HMAC-SHA-256) and sent in the clear; "
         "TX-OFF removes the power amplifier's supply through a hardware switch.",
         [cite(BASELINE, "HMAC-SHA-256", "TX-OFF removes PA power through a hardware switch", where="§7")]),
        ("Antennas",
         "An ISIS AntS unit on the −Z face with four tape monopoles: two for 2 m (≈ 0.5 m) along ±X and two for 70 cm "
         "(≈ 0.17 m) along ±Y. They deploy after the wings.",
         [cite(BASELINE, "ISIS AntS 1U–3U", "Antennas deploy after the wings", where="§7"),
          cite(MECH, "2 m ±X, 70 cm ±Y", where="§5.2")]),
        ("Attitude control",
         "Magnetorquer rods, a magnetometer, the panel photodiodes as sun sensors and a MEMS gyro. B-dot detumble, then "
         "a spin of 8°/s that keeps the wings' side toward the Sun. No reaction wheels and no propulsion.",
         [cite(BASELINE, "CubeSpace CR0002 rods", "Epson M-G370PDT0", where="§8")]),
        ("Navigation",
         "A duty-cycled GNSS receiver (SkyFox piNAV-NG/FM) gives the operator ephemeris the launch provider requires "
         "within 24 hours of first contact.",
         [cite(BASELINE, "SkyFox piNAV-NG/FM", "within 24 h of first contact", where="§6")]),
        ("Thermal",
         "No fan or active cooling: a ≈ 207 Ω battery heater that switches on below +5 °C and off above +9 °C, and a heat spreader "
         "from the Jetson to the frame.",
         [cite(BASELINE, "Battery heater ≈ 207 Ω", where="§8")]),
    ]
    cards = "".join(f'<article class="card"><h3>{esc(t)}</h3><p>{esc(p)}</p><p class="srcs">{" ".join(s)}</p></article>'
                    for t, p, s in subs)
    note2 = ""
    if d["sheet2_two_elements"]:
        note2 = (" This sheet was generated before the CAD carried the ±Y 70 cm elements, so it draws two antenna "
                 "elements; the baseline and the CAD's mass properties have four.")
    body = f"""
<h1>The spacecraft</h1>
<p class="lede">A 1U CubeSat with two deployable solar wings and four deployable antenna elements, designed to
schematic level with selected, orderable parts. No part has been bought, made or weighed.
{cite(MECH, 'no part has been bought, made or weighed', where='status')}</p>

<figure class="wide-fig">
{svg}
<figcaption><ol class="fig-legend">{legend}</ol>
<p>Illustration drawn from the CAD dimensions: the wings in the plane 38.75 mm toward +Y, hinged at x = ±54.1 mm;
the 2 m elements are shortened to fit. {cite(MECH, '(±54.1, +38.75) mm', where='§4.2')}</p></figcaption>
</figure>

<section aria-labelledby="sub-h">
<h2 id="sub-h">Subsystems and key parts</h2>
<div class="cards">{cards}</div>
</section>

<section aria-labelledby="cad-h">
<h2 id="cad-h">CAD drawings</h2>
<p>Hidden-line projections of the parametric CAD model ({esc(doc_id(MECH) or '')} version {esc(doc_rev(MECH) or '')}),
generated by <code>hardware/mechanical/model.py</code>. They are layout drawings for review, not manufacturing
drawings. {cite(MECH, 'They are layout drawings for review, not manufacturing', where='§7')}</p>
<figure class="drawing">
<a href="{r('assets/drawing_stowed_3view.svg')}"><img src="{r('assets/drawing_stowed_3view.svg')}" width="1100" height="780" loading="lazy"
alt="CAD sheet 1: three views of the stowed 1U CubeSat with the CubeSat Design Specification envelope, rail dimensions, the 6.5 mm protrusion envelope, the internal board stack, and the locations of the deployment switches, separation plungers, remove-before-flight pin and access port."></a>
<figcaption>Sheet 1, stowed: three views with the CDS envelope, the rails, the protrusion envelope and the switches.
Open the drawing for full size.</figcaption>
</figure>
<figure class="drawing">
<a href="{r('assets/drawing_deployed.svg')}"><img src="{r('assets/drawing_deployed.svg')}" width="1100" height="780" loading="lazy"
alt="CAD sheet 2: the deployed configuration seen from the antenna face and from the sun side, with the two solar wings spanning 267.7 mm tip to tip and the antenna element extents."></a>
<figcaption>Sheet 2, deployed: wing and antenna extents, 267.7 mm tip to tip across the wings.{esc(note2)}</figcaption>
</figure>
<p>STEP models of both configurations are in <a href="{TREE}hardware/mechanical"><code>hardware/mechanical</code></a>.</p>
</section>

<section aria-labelledby="design-h">
<h2 id="design-h">How far the design has got</h2>
<p>Each board is captured as code (SKiDL) with selected parts, a netlist, a bill of materials with derating and an
electrical rules check; the mechanical design is a CAD model with interference checks and a finite-element model.
{cite(CDR, 'Each board is captured as code', where='§5')}</p>
{dict_table(d['design_items'], ['Item', 'Document', 'What exists', 'Key results'], CDR, 'Design completion (CDR report §5)')}
<p class="note">{inline(d['design_missing'], CDR)}</p>
<p>Margins on mass, power, structure, links and more: <a href="{r('engineering/index.html')}#margins-h">engineering</a>.</p>
</section>
"""
    return site.frame("spacecraft/index.html", "Spacecraft",
                      "The SMART-QSO 1U CubeSat: subsystems, key parts, CAD drawings and how far the design has got.",
                      body)


def page_engineering(site: Site, d: dict) -> str:
    rid = d["rid"]
    disp_rows = "".join(f'<tr><th scope="row">{esc(x["disposition"])}</th><td>{x["round_1"]}</td><td>{x["final"]}</td></tr>'
                        for x in rid["dispositions"])
    sev = rid["severity"]
    key_docs = [CDR, "docs/reviews/CDR/CDR_TECHNICAL_BASELINE.md", "docs/reviews/CDR/CDR_REVIEW_MINUTES.md",
                BASELINE, SCHEDULE, "docs/verification/VI_REGISTER.md", "docs/project/CHANGE_REQUEST_LOG.md",
                SIL, MECH, "docs/project/CDR_CRITERIA_MATRIX.md"]
    doc_rows = []
    for p in key_docs:
        url = gh_url(p)
        if not url:
            raise SourceError(f"{p}: listed on the engineering page but not in the tree")
        read(p)
        title = next((ln[2:].strip() for ln in read(p).splitlines() if ln.startswith("# ")), p)
        rev = doc_rev(p)
        doc_rows.append(f'<li><a href="{url}">{esc(title)}</a> <span class="muted">{esc(doc_id(p) or "")}'
                        f'{(" · " + esc(rev)) if rev else ""}</span></li>')
    body = f"""
<h1>Engineering: the CDR 2026 design review</h1>
<p class="lede">On {esc(d['cdr_date'])} the project finished a design review against the NASA NPR 7123.1D Critical
Design Review criteria, tailored for a Class D CubeSat. This page shows what it found, what it decided, and what it
left for hardware and for the principal investigator (PI). Report: {cite(CDR, label=(d['cdr_id'] or 'CDR report') + ' version ' + (d['cdr_rev'] or ''))}</p>

<section class="status" aria-labelledby="nature-h">
<h2 id="nature-h">What the review is, and is not</h2>
<p class="muted">In the report's own words (§1.2):</p>
{blocks(d['nature_md'], CDR)}
</section>

<section aria-labelledby="maturity-h">
<h2 id="maturity-h">Verdict and maturity</h2>
<p><strong>{esc(d['verdict_lead'])}</strong> Every one of the {rid['total']} findings carries a final disposition.
"Closed" means the evidence is recorded in the repository; where only hardware or an outside party can supply it,
the finding is closed as a dated verification item or decision point, not as a result.
{cite(CDR, 'A closure records evidence in the repository', where='§7.1')}</p>
<p class="muted">The report's maturity statement (§7.2):</p>
{blocks(d['maturity_md'], CDR)}
</section>

<section aria-labelledby="rid-h">
<h2 id="rid-h">Review item discrepancies (RIDs)</h2>
<p>{rid['total']} RIDs were raised: {sev['1']} of severity 1 (mission or safety critical), {sev['2']} of severity 2
(must close before fabrication), {sev['3']} of severity 3 (should fix). {esc(rid['reopened'])}
Open implementation RIDs: {esc(rid['open_implementation'])} {cite(CDR, 'RID-SUMMARY-BEGIN', where='§6')}</p>
<div class="table-wrap" role="region" aria-label="RID dispositions" tabindex="0">
<table class="compact"><caption>RID dispositions when raised (round 1) and final (CDR report §6)</caption>
<thead><tr><th scope="col">Disposition</th><th scope="col">Round 1</th><th scope="col">Final</th></tr></thead>
<tbody>{disp_rows}<tr class="total"><th scope="row">Total</th><td>{rid['total']}</td><td>{rid['total']}</td></tr></tbody></table></div>
</section>

<section aria-labelledby="margins-h">
<h2 id="margins-h">Margins at the review</h2>
<p>Every margin is a prediction from analysis or simulation; none is measured. The mission simulation (version
{esc(d['sil_rev'] or '')}) ran the unchanged flight software through {esc(d['sil_days'])} days of operations and found
{esc(d['sil_anomalies'])} anomalies and {esc(d['sil_warnings'])} warnings in {esc(d['sil_checks'])} checks; a beacon
was decoded on {esc(d['sil_heard'])} % of {esc(d['sil_passes'])} simulated passes, and the generated beacon text cost
{esc(d['sil_wh_beacon'])} Wh per accepted line under assumed inputs. {cite(SIL, where='§7', label='Mission simulation')}</p>
{dict_table(d['margins'], ['Quantity', 'Requirement', 'Predicted', 'Margin', 'Source'], CDR, 'Margins at the review (CDR report §3.3)', 'margins')}
</section>

<section aria-labelledby="dec-h">
<h2 id="dec-h">Chair decisions: proposals awaiting the PI</h2>
<p>The review chair made {len(d['decisions'])} decisions ({esc(d['decisions'][0]['#'])} to {esc(d['decisions'][-1]['#'])}).
They are proposals by AI reviewers, not decisions of a board, and bind nothing until the PI ratifies or rejects
them (action DA-01, due {esc(d['da01_due'])}). {cite(CDR, 'they bind nothing until the PI ratifies them', where='§7.1')}
{cite(SCHEDULE, 'DA-01', where='§5')}</p>
<details>
<summary>The {len(d['decisions'])} chair decisions (CDR report §4)</summary>
{dict_table(d['decisions'], ['#', 'Decision', 'RIDs'], CDR, 'Chair decisions, pending PI ratification (CDR report §4)')}
</details>
</section>

<section aria-labelledby="remain-h">
<h2 id="remain-h">What remains</h2>
{dict_table(d['closed_means'], ['Kind', 'Count', 'What remains'], CDR, 'What closed means (CDR report §7.1)')}
<h3>What the PI must do next</h3>
{dict_table(d['pi_actions'], ['By', 'Action'], CDR, 'Dated PI actions (CDR report §7.3)', 'compact')}
<details>
<summary>All {len(d['actions'])} dated actions with owners and pass criteria (schedule §5)</summary>
{dict_table(d['actions'], ['ID', 'Decision or action', 'Owner', 'Due', 'Pass criterion or outcome', 'Status'], SCHEDULE, 'Decision points and dated actions (integrated master schedule §5)')}
</details>
<h3>Risks the review accepts</h3>
{blocks(d['risks_md'], CDR)}
</section>

<section aria-labelledby="docs-h">
<h2 id="docs-h">The documents</h2>
<ul class="doclist">{''.join(doc_rows)}</ul>
<p>Every design value is governed by the baseline; where another document disagrees, the baseline is right.
{cite(BASELINE, 'this document governs', where='§0')}</p>
</section>
"""
    return site.frame("engineering/index.html", "Engineering",
                      "The SMART-QSO CDR 2026 design review: what it is and is not, the RID result, the margins, the "
                      "chair decisions awaiting the PI, and what remains.", body)


def page_participate(site: Site, d: dict) -> str:
    r = lambda t: site.rel("participate/index.html", t)  # noqa: E731
    if d["iaru_submitted"]:
        raise SourceError(f"{IARU}: the coordination request now reads as submitted; update the participate page")
    body = f"""
<h1>Participate</h1>
<p class="lede">SMART-QSO is not in orbit, and it has no frequencies yet. There is nothing to listen for today. There is
useful work to do now, and this page says what it is.</p>

<section class="status" aria-labelledby="reg-h">
<h2 id="reg-h">Licensing and frequencies</h2>
<p>SMART-QSO would be an FCC Part 97 amateur space station. The bands below are the ones the project will ask the
IARU to coordinate a frequency in; they are not assigned frequencies. Downlink in {esc(d['downlink'])} (not 145.825
MHz); uplink requested in {esc(d['uplink_windows'])}. {cite(BASELINE, 'not 145.825 MHz', where='§7')}</p>
<p>IARU coordination request: {inline(d['iaru_request'], IARU)} {cite(IARU, where='§1')}</p>
{dict_table(d['auth'], ['Authorization', 'Status', 'Draft / reference'], SPECTRUM, f"Authorization status as of {d['auth_asof']} (spectrum authorization status §4)")}
</section>

<section aria-labelledby="ops-h">
<h2 id="ops-h">For amateur radio operators, once it flies</h2>
<p>The planning launch date is about {esc(d['launch_date'])}. If it launches and is commissioned, any licensed amateur
with an FM handheld or mobile rig and a small Yagi could work other stations through the transponder, in sunlit passes
when the power gates open. Community and SatNOGS stations would receive and decode the beacon, with logs published
openly. {cite(CONOPS, 'any licensed amateur with an FM handheld or mobile rig and a small Yagi', 'SatNOGS', where='§6')}</p>
<p>The beacon is a 1200 bps AFSK AX.25 UI frame from {esc(d['callsign'])} with one line of plain text and a telemetry
tail; every value in it is a measurement, and an unmeasured field is sent as <code>-</code>.
{cite(BEACON, '1200 bps AFSK (Bell 202', where='§1–§4')}</p>
<pre class="fmt"><code>{esc(d['beacon_format'])}</code></pre>
<p>A decoder is in <a href="{BLOB}software/ground/tools/beacon_decoder.py"><code>software/ground/tools/beacon_decoder.py</code></a>.</p>
</section>

<section aria-labelledby="help-h">
<h2 id="help-h">Help wanted now</h2>
<div class="cards">
<article class="card"><h3>Telecommand stations</h3>
<p>The mission needs at least two designated amateur stations at least 1,000 km apart, each able to command the
spacecraft alone, with written agreements by 2027-01-05. None is agreed yet. A station must radiate at least 64 dBm
EIRP toward the spacecraft on 70 cm and commands only above 10° elevation.</p>
<p class="srcs">{cite(BASELINE, 'at least 1,000 km apart, each able to command alone', where='§1')}
{cite(CONOPS, '≥ 64 dBm EIRP', where='§6')} {cite('docs/ground/GROUND_STATION_DESIGN.md', label='Ground station design')}</p></article>
<article class="card"><h3>Independent reviewers</h3>
<p>The CDR 2026 review was done by AI reviewers. The independent PDR, planned for 2027-01-12, needs a chair and at
least two reviewers from outside the team, covering systems, electrical and RF, and flight software.</p>
<p class="srcs">{cite(SCHEDULE, 'Name the independent PDR board', where='§5 DA-03')}</p></article>
<article class="card"><h3>PCB layout</h3>
<p>The boards exist as schematics. The plan needs two PCB layout designers (KiCad, IPC-6012 Class 3) to start in
January 2027; with one, the schedule loses its float.</p>
<p class="srcs">{cite(SCHEDULE, 'Engage two PCB layout designers', where='§5 DA-04')}</p></article>
<article class="card"><h3>Review the documents</h3>
<p>Every claim the project makes is in the repository with its evidence. Reading a document against its sources and
opening an issue for what is wrong is the most useful contribution of all.</p>
<p class="srcs"><a class="src" href="{GITHUB}/issues">Issues</a> {cite(BASELINE, label='Baseline')}</p></article>
<article class="card"><h3>Software</h3>
<p>Flight software in Rust (<code>no_std</code>, <code>unsafe</code> forbidden in the protocol and logic crates), the
Jetson payload and the ground tools in Python. Tests run in CI.</p>
<p class="srcs">{cite('software/rust/README.md', label='software/rust')} {cite('software/ground/tools/README.md', label='Ground tools')}</p></article>
</div>
<p>Start with <a href="{BLOB}CONTRIBUTING.md">CONTRIBUTING.md</a>: discuss in an issue first, then a pull request with
tests. The project follows a <a href="{BLOB}CODE_OF_CONDUCT.md">code of conduct</a> and a
<a href="{BLOB}GOVERNANCE.md">governance</a> document; report security problems as <a href="{BLOB}SECURITY.md">SECURITY.md</a> says.
Amateur operation of the station is unpaid. {cite(BASELINE, 'None for operations', where='§1')}</p>
</section>
"""
    return site.frame("participate/index.html", "Participate",
                      "How amateur radio operators and contributors can help SMART-QSO, and the real status of its "
                      "licensing and frequency coordination.", body)


def page_404(site: Site) -> str:
    r = lambda t: site.rel("404.html", t, absolute=True)  # noqa: E731
    body = f"""
<h1>Page not found</h1>
<p class="lede">That page is not on this site. The site was rebuilt in October 2026 for the current 1U design, and
the older pages and downloads were removed.</p>
<ul><li><a href="{r('index.html')}">Home</a></li><li><a href="{r('mission/index.html')}">Mission</a></li>
<li><a href="{r('spacecraft/index.html')}">Spacecraft</a></li><li><a href="{r('engineering/index.html')}">Engineering</a></li>
<li><a href="{r('participate/index.html')}">Participate</a></li></ul>
"""
    return site.frame("404.html", "Page not found", "Page not found.", body, absolute=True)


def page_redirect(site: Site, page: str, target: str) -> str:
    to = site.rel(page, target)
    return (f'<!doctype html>\n<html lang="en"><head><meta charset="utf-8"><title>Moved · SMART-QSO</title>'
            f'<meta name="robots" content="noindex"><meta http-equiv="refresh" content="0; url={to}">'
            f'<link rel="canonical" href="{site.url}{target.replace("index.html", "")}"></head>'
            f'<body><p>This page has moved to <a href="{to}">{to}</a>.</p></body></html>\n')


# ----------------------------------------------------------------------------------------------
# Output
# ----------------------------------------------------------------------------------------------

def git(*args: str) -> str | None:
    try:
        return subprocess.run(["git", *args], cwd=REPO, check=True, capture_output=True, text=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(out: Path, base: str, url: str) -> list[str]:
    d = extract()
    illus = illustration()
    commit = git("rev-parse", "HEAD") or "unknown"
    if os.environ.get("SOURCE_DATE_EPOCH"):
        built = _dt.datetime.fromtimestamp(int(os.environ["SOURCE_DATE_EPOCH"]), _dt.timezone.utc).isoformat()
    else:
        built = git("log", "-1", "--format=%cI") or _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")
    site = Site(base, url, commit, built)

    pages = {
        "index.html": page_home(site, d, illus),
        "mission/index.html": page_mission(site, d),
        "spacecraft/index.html": page_spacecraft(site, d, illus),
        "engineering/index.html": page_engineering(site, d),
        "participate/index.html": page_participate(site, d),
        "404.html": page_404(site),
        # the old site's research page
        "research/index.html": page_redirect(site, "research/index.html", "engineering/index.html"),
    }

    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    for rel, text in pages.items():
        p = out / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")
    (out / "assets").mkdir(exist_ok=True)
    shutil.copyfile(SITE_DIR / "site.css", out / "assets/site.css")
    for name, src in DRAWINGS.items():
        shutil.copyfile(REPO / src, out / "assets" / name)
    (out / "favicon.svg").write_text(FAVICON, encoding="utf-8")
    (out / ".nojekyll").write_text("", encoding="utf-8")
    (out / "robots.txt").write_text(f"User-agent: *\nAllow: /\nSitemap: {site.url}sitemap.xml\n", encoding="utf-8")
    urls = "".join(f"<url><loc>{site.url}{p.replace('index.html', '')}</loc><lastmod>{built[:10]}</lastmod></url>\n"
                   for p, _ in PAGES)
    (out / "sitemap.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        + urls + "</urlset>\n", encoding="utf-8")

    def source(path: str, section_: str) -> dict:
        return {"document": path, "document_id": doc_id(path), "revision": doc_rev(path), "section": section_,
                "url": BLOB + path}

    ev = out / "evidence"
    ev.mkdir()
    evidence = {
        "margins.json": {"source": source(CDR, "§3.3"), "rows": [
            {"quantity": m["Quantity"], "requirement": m["Requirement"], "predicted": m["Predicted"],
             "margin": m["Margin"], "source": m["Source"]} for m in d["margins"]]},
        "rid_summary.json": {"source": source(CDR, "§6 (RID-SUMMARY block)"), **d["rid"]},
        "schedule_actions.json": {"source": source(SCHEDULE, "§5"), "actions": [
            {"id": a["ID"], "action": a["Decision or action"], "owner": a["Owner"], "due": a["Due"],
             "due_dates": re.findall(r"\d{4}-\d{2}-\d{2}", a["Due"]), "criterion": a["Pass criterion or outcome"],
             "parent_rids": a["Parent RID"], "status": a["Status"]} for a in d["actions"]]},
        "milestones.json": {"source": source(SCHEDULE, "§4"),
                            "milestones": [{"milestone": m["Milestone"], "date": m["Date"]} for m in d["milestones"]]},
        "chair_decisions.json": {"source": source(CDR, "§4"),
                                 "status": f"Proposals awaiting PI ratification (DA-01, due {d['da01_due']})",
                                 "decisions": [{"id": x["#"], "decision": x["Decision"], "rids": x["RIDs"]}
                                               for x in d["decisions"]]},
        "authorization_status.json": {"source": source(SPECTRUM, "§4"), "as_of": d["auth_asof"],
                                      "rows": [{"authorization": a["Authorization"], "status": a["Status"]}
                                               for a in d["auth"]]},
        "key_values.json": {"source": source(BASELINE, "§1–§8"), "values": {
            k: d[k] for k in ("envelope", "mass_cbe", "mass_mev", "mass_limit", "mass_margin", "alt_target", "alt_lo",
                              "alt_hi", "alt_cap", "inclination", "design_life", "min_mission", "tx_nom", "tx_max",
                              "battery_wh", "gen_tumble", "gen_sun", "downlink", "uplink_windows", "callsign",
                              "phase", "launch_date", "ltan")}},
    }
    for name, obj in evidence.items():
        (ev / name).write_text(json.dumps(obj, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    files = sorted(p.relative_to(out).as_posix() for p in out.rglob("*") if p.is_file())
    lines = [f"{sha256(out / f)}  {f}" for f in files]
    (out / "checksums.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")
    files.append("checksums.sha256")
    publication = {
        "built_at_utc": built,
        "source_commit": commit,
        "url": site.url,
        "base_path": site.base,
        "generator": "site/build.py",
        "scope": "Static public mission site; no command service. Content is generated from the repository documents.",
        "status": "Design phase: no hardware built, not launched, no frequencies coordinated",
        "sources": {p: {"sha256": sha256(REPO / p), "document_id": doc_id(p) if p.endswith(".md") else None,
                        "revision": doc_rev(p) if p.endswith(".md") else None}
                    for p in sorted(used_sources)},
        "artifacts": {f: sha256(out / f) for f in sorted(files)},
    }
    (out / "publication.json").write_text(json.dumps(publication, indent=2, ensure_ascii=False) + "\n",
                                          encoding="utf-8")
    return sorted(pages)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--out", type=Path, default=SITE_DIR / "_build", help="output directory (replaced)")
    ap.add_argument("--base", default=os.environ.get("SITE_BASE", DEFAULT_BASE),
                    help=f"base path on the web server (default {DEFAULT_BASE}, or $SITE_BASE)")
    ap.add_argument("--site-url", default=os.environ.get("SITE_URL", DEFAULT_URL),
                    help=f"public URL for canonical links and the sitemap (default {DEFAULT_URL})")
    args = ap.parse_args(argv)
    try:
        pages = build(args.out.resolve(), args.base, args.site_url)
    except SourceError as e:
        print(f"site build failed: {e}", file=sys.stderr)
        print("Nothing was published. Fix the document (or the parser in site/build.py) and rebuild.",
              file=sys.stderr)
        return 1
    print(f"site built: {len(pages)} pages in {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
