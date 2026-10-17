"""Tests for the RTM generator (scripts/gen_rtm.py, VV-06)."""

import csv
import io
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import gen_rtm  # noqa: E402

MISSION = """# Mission Requirements
## Objectives
| ID | Objective | Priority |
|----|-----------|----------|
| MO-1 | Demonstrate a beacon from orbit | Primary |

## Applicable Documents
| Document | Title |
|----------|-------|
| SQSO-REQ-002 | System Requirements Specification |

## Requirements
| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-020 | The spacecraft shall transmit a beacon | T | Critical |
"""

SYSTEM = """# System Requirements
| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-001 | The spacecraft shall transmit a beacon every 60 seconds | MRD-020 | T |
| SYS-002 | The spacecraft shall cease transmission on telecommand, in every mode | MRD-020 | I,T |

## TBDs
| ID | Item | Closure Plan | Target |
|----|------|--------------|--------|
| TBD-001 | Final frequency | Coordination | TBD |
"""

SOFTWARE = """# Software Requirements
| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F001 | The software shall initialize within 30 seconds | T | High |

| ID | Constraint |
|----|------------|
| SRS-C001 | OBC code shall be written in C11 |

| SRS ID | Parent (SYS) | Description |
|--------|--------------|-------------|
| SRS-F001 | SYS-001 | Initialization |
"""

SPECS = ("docs/MRD.md", "docs/SYS.md", "docs/SRS.md")


def tree(tmp_path, code=None, procedures=None):
    (tmp_path / "docs").mkdir()
    (tmp_path / "docs/MRD.md").write_text(MISSION)
    (tmp_path / "docs/SYS.md").write_text(SYSTEM)
    (tmp_path / "docs/SRS.md").write_text(SOFTWARE)
    for rel, text in (code or {}).items():
        path = tmp_path / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    tp_dir = tmp_path / "docs/test_procedures"
    tp_dir.mkdir()
    for name, text in (procedures or {}).items():
        (tp_dir / name).write_text(text)
    return tmp_path


def by_id(root):
    return {row["id"]: row for row in gen_rtm.rows(root, specs=SPECS)}


def test_requirement_rows_only(tmp_path):
    rows = by_id(tree(tmp_path))
    # Objectives, requirements and constraints; not TBDs, documents or parent tables
    assert list(rows) == ["MO-1", "MRD-020", "SYS-001", "SYS-002", "SRS-F001", "SRS-C001"]


def test_columns_found_by_header_name(tmp_path):
    rows = by_id(tree(tmp_path))
    assert rows["SYS-002"]["parent"] == "MRD-020"
    assert rows["SYS-002"]["method"] == "I,T"
    assert rows["SYS-002"]["specification"] == "docs/SYS.md"
    assert rows["SYS-002"]["requirement"].endswith("on telecommand, in every mode")
    assert rows["MRD-020"]["method"] == "T"          # Verification is the third column here
    assert rows["MRD-020"]["parent"] == ""
    assert rows["MO-1"]["method"] == ""
    assert rows["SRS-C001"]["requirement"] == "OBC code shall be written in C11"


def test_code_tags_split_into_code_and_tests(tmp_path):
    code = {
        "software/flight/src/beacon.c": "/**\n * @requirement SYS-001 Beacon every 60 s\n"
                                        " * @requirement SYS-001 again\n */\n",
        "software/flight/include/beacon.h": " * @requirement SYS-001 Beacon every 60 s\n",
        "software/flight/tests/test_beacon.c": " * @requirement SYS-001 Beacon every 60 s\n"
                                               " * @requirement SRS-X999 not defined\n",
        "software/payload/link.py": '"""@requirement SRS-F001 init"""\n',
        "software/flight/build/gen.c": " * @requirement SYS-002 build output\n",
    }
    rows = by_id(tree(tmp_path, code=code))
    assert rows["SYS-001"]["code"] == "software/flight/include/beacon.h; software/flight/src/beacon.c"
    assert rows["SYS-001"]["tests"] == "software/flight/tests/test_beacon.c"
    assert rows["SRS-F001"]["code"] == "software/payload/link.py"
    assert rows["SYS-002"]["code"] == ""             # build directories are not source
    assert "SRS-X999" not in rows                    # undefined IDs are check_traceability's job


def test_procedures_citing_each_requirement(tmp_path):
    procedures = {
        "TP-B.md": "**Requirements verified:** SYS-001, SYS-002\n| 1 | Key | Off | SYS-002 |\n",
        "TP-A.md": "| 1 | Beacon | 60 s | SYS-001 |\n",
        "TP-SYS-002_Name.md": "# TP-SYS-002\nCites no requirement\n",
    }
    rows = by_id(tree(tmp_path, procedures=procedures))
    assert rows["SYS-001"]["procedures"] == "docs/test_procedures/TP-A.md; docs/test_procedures/TP-B.md"
    # TP-SYS-002 is a procedure ID, not a citation of SYS-002
    assert rows["SYS-002"]["procedures"] == "docs/test_procedures/TP-B.md"
    assert rows["MRD-020"]["procedures"] == ""


def test_correction_notes_are_not_citations(tmp_path):
    procedures = {
        # Says why it does not verify SYS-002: not a citation
        "TP-A.md": "**Requirements verified:** SYS-001\n\n"
                   "**Not cited until corrected:** step 3 (2 W; SYS-002 caps it)\n",
        # Cites SYS-002 in a heading as well: still a citation
        "TP-B.md": "**To be corrected:** step 4 conflicts with SYS-002\n### 3.1 Power [SYS-002]\n",
    }
    rows = by_id(tree(tmp_path, procedures=procedures))
    assert rows["SYS-001"]["procedures"] == "docs/test_procedures/TP-A.md"
    assert rows["SYS-002"]["procedures"] == "docs/test_procedures/TP-B.md"


def test_render_is_csv_with_header(tmp_path):
    text = gen_rtm.render(tree(tmp_path), specs=SPECS)
    assert text.startswith("id,specification,parent,method,requirement,code,tests,procedures\n")
    assert "\r" not in text
    parsed = list(csv.DictReader(io.StringIO(text)))
    assert [r["id"] for r in parsed][:2] == ["MO-1", "MRD-020"]
    assert parsed[3]["requirement"] == "The spacecraft shall cease transmission on telecommand, in every mode"
    assert gen_rtm.render(tmp_path, specs=SPECS) == text   # deterministic


def test_check_fails_when_stale(tmp_path):
    root = tree(tmp_path)
    out = root / "docs/verification/RTM.csv"
    assert gen_rtm.check(root, specs=SPECS, out=out) is False      # missing
    out.parent.mkdir(parents=True)
    out.write_text(gen_rtm.render(root, specs=SPECS))
    assert gen_rtm.check(root, specs=SPECS, out=out) is True
    (root / "docs/test_procedures/TP-C.md").write_text("| 1 | Init | 30 s | SRS-F001 |\n")
    assert gen_rtm.check(root, specs=SPECS, out=out) is False


def test_repository_rtm_is_current():
    assert gen_rtm.check() is True
