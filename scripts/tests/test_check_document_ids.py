"""Tests for the document-ID check (scripts/check_document_ids.py, PI-17 / VV-18)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from check_document_ids import check, main, scan  # noqa: E402

REPO = Path(__file__).resolve().parents[2]


def doc(root, rel, ident, title="A Document", header="**Document Number:** {}", body=""):
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    head = header.format(ident) if ident else ""
    path.write_text(f"# {title}\n\n{head}\n**Revision:** A\n\n## 1. Purpose\n\n{body}\n")
    return path


def register(root, rows):
    lines = ["# Document Register", "", "| ID | Title | Path | Revision | Status |",
             "|----|-------|------|----------|--------|"]
    lines += [f"| {ident} | Title | `{rel}` | A | Draft |" for ident, rel in rows]
    path = root / "docs/DOCUMENT_REGISTER.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n")


def test_repository_passes():
    assert check(REPO) == []


def test_clean_tree(tmp_path):
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001")
    doc(tmp_path, "hardware/b.md", "SMART-QSO-BBB-001")
    register(tmp_path, [("SQSO-AAA-001", "docs/a.md"), ("SMART-QSO-BBB-001", "hardware/b.md")])
    assert check(tmp_path) == []


def test_same_id_in_two_files(tmp_path):
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001")
    doc(tmp_path, "docs/b.md", "SQSO-AAA-001")
    register(tmp_path, [("SQSO-AAA-001", "docs/a.md"), ("SQSO-AAA-001", "docs/b.md")])
    problems = check(tmp_path)
    assert len(problems) == 1
    assert "SQSO-AAA-001" in problems[0]
    assert "docs/a.md" in problems[0] and "docs/b.md" in problems[0]


def test_unregistered_document(tmp_path):
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001")
    doc(tmp_path, "docs/b.md", "SQSO-BBB-001")
    register(tmp_path, [("SQSO-AAA-001", "docs/a.md")])
    problems = check(tmp_path)
    assert problems == ["SQSO-BBB-001 (docs/b.md) is not in docs/DOCUMENT_REGISTER.md"]


def test_register_entry_that_no_longer_matches(tmp_path):
    doc(tmp_path, "docs/a.md", "SQSO-AAA-002")
    register(tmp_path, [("SQSO-AAA-001", "docs/a.md"), ("SQSO-AAA-002", "docs/a.md")])
    problems = check(tmp_path)
    assert len(problems) == 1
    assert "SQSO-AAA-001" in problems[0] and "stale" in problems[0]


def test_all_header_spellings(tmp_path):
    spellings = ["**Document Number:** {}", "**Document Number**: {}",
                 "**Document ID**: {}", "**Document ID:** {}"]
    for n, header in enumerate(spellings, 1):
        doc(tmp_path, f"docs/d{n}.md", f"SQSO-HDR-00{n}", header=header)
    assert set(scan(tmp_path)) == {f"SQSO-HDR-00{n}" for n in range(1, 5)}


def test_stub_and_body_mentions_are_not_documents(tmp_path):
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001")
    doc(tmp_path, "docs/stub.md", None, body="Superseded by SQSO-AAA-001 (`docs/a.md`).")
    register(tmp_path, [("SQSO-AAA-001", "docs/a.md")])
    assert scan(tmp_path) == {"SQSO-AAA-001": ["docs/a.md"]}
    assert check(tmp_path) == []


def test_template_inside_document_is_not_a_header(tmp_path):
    body = "\n".join(["text"] * 30) + "\n```\nREPORT\n**Document ID:** SQSO-CAR-001\n```\n"
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001", body=body)
    assert scan(tmp_path) == {"SQSO-AAA-001": ["docs/a.md"]}


def test_funding_audit_and_hidden_directories_are_excluded(tmp_path):
    doc(tmp_path, "docs/funding_audit/x.md", "SQSO-AAA-001")
    doc(tmp_path, ".claude/worktrees/w/docs/a.md", "SQSO-AAA-001")
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001")
    assert scan(tmp_path) == {"SQSO-AAA-001": ["docs/a.md"]}


def test_procedure_id_defined_twice(tmp_path):
    doc(tmp_path, "docs/test_procedures/TP-INT-001_EPS.md", "TP-INT-001", title="EPS Integration")
    doc(tmp_path, "docs/verification/TEST_PROCEDURES.md", "SMART-QSO-TP-001",
        body="### 4.1 TP-INT-001: EPS + Sensors Integration\n")
    register(tmp_path, [("TP-INT-001", "docs/test_procedures/TP-INT-001_EPS.md"),
                        ("SMART-QSO-TP-001", "docs/verification/TEST_PROCEDURES.md"),
                        ("TP-INT-001", "docs/verification/TEST_PROCEDURES.md")])
    problems = check(tmp_path)
    assert len(problems) == 1
    assert "TP-INT-001" in problems[0] and "TEST_PROCEDURES.md" in problems[0]


def test_sqso_prefixed_procedure_id_is_the_same_procedure(tmp_path):
    doc(tmp_path, "docs/test_procedures/TP-OBC-002_Load.md", "SQSO-TP-OBC-002",
        body="## TP-OBC-002\n")
    doc(tmp_path, "docs/x.md", "SQSO-XXX-001", body="## 2.1 TP-OBC-002: Watchdog timeout\n")
    register(tmp_path, [("SQSO-TP-OBC-002", "docs/test_procedures/TP-OBC-002_Load.md"),
                        ("SQSO-XXX-001", "docs/x.md"), ("TP-OBC-002", "docs/x.md")])
    problems = check(tmp_path)
    assert len(problems) == 1 and "TP-OBC-002" in problems[0]


def test_procedure_defined_in_a_heading_must_be_registered(tmp_path):
    doc(tmp_path, "docs/x.md", "SQSO-XXX-001", body="### 3.1 TP-UNIT-001: CRC32 Module Tests\n")
    register(tmp_path, [("SQSO-XXX-001", "docs/x.md")])
    assert check(tmp_path) == ["TP-UNIT-001 (docs/x.md) is not in docs/DOCUMENT_REGISTER.md"]


def test_short_procedure_ids_in_headings_count(tmp_path):
    doc(tmp_path, "docs/test/A.md", "SQSO-TP-001", body="### 4.1 TP-001: Boot Test\n")
    doc(tmp_path, "docs/test/B.md", "SQSO-TP-009", body="### 2 TP-001: Mode Test\n")
    register(tmp_path, [("SQSO-TP-001", "docs/test/A.md"), ("TP-001", "docs/test/A.md"),
                        ("SQSO-TP-009", "docs/test/B.md"), ("TP-001", "docs/test/B.md")])
    problems = check(tmp_path)
    assert len(problems) == 1 and "TP-001 is defined" in problems[0]


def test_check_flag_sets_exit_status(tmp_path, capsys):
    doc(tmp_path, "docs/a.md", "SQSO-AAA-001")
    register(tmp_path, [("SQSO-AAA-001", "docs/a.md")])
    assert main(["--check"], root=tmp_path) == 0
    doc(tmp_path, "docs/b.md", "SQSO-AAA-001")
    assert main(["--check"], root=tmp_path) == 1
    assert main([], root=tmp_path) == 0   # report only
    assert "SQSO-AAA-001" in capsys.readouterr().out
