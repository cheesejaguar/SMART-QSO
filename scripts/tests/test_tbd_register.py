"""Tests for the TBD/TBR register (scripts/tbd_register.py, VV-17)."""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tbd_register import check, compare, counts, main, render, scan  # noqa: E402

REPO = Path(__file__).resolve().parents[2]

TABLE = "| ID | Item | Owner | Target |\n|----|------|-------|--------|\n"


def write(root, rel, text):
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    return path


def baseline(root, tbd, tbr):
    write(root, "scripts/tbd_baseline.json", json.dumps({"TBD": tbd, "TBR": tbr}))


def test_repository_register_is_current():
    assert check(REPO) == []


def test_counts_standalone_upper_case_words(tmp_path):
    write(tmp_path, "docs/a.md",
          "Mass TBD.\n"
          "Frequency (TBR), launch L-TBD.\n"
          "TBDs and TBRs are plurals; tbd is lower case; TBDX is another word.\n")
    reg = scan(tmp_path)
    assert counts(reg) == {"TBD": 2, "TBR": 1}
    assert [(o.line, o.counts) for o in reg.occurrences] == [(1, {"TBD": 1}),
                                                            (2, {"TBD": 1, "TBR": 1})]
    assert reg.problems == []


def test_several_words_on_one_line_are_one_row_counted_each(tmp_path):
    write(tmp_path, "docs/a.md", "| Analyzer | TBD | TBD | TBD |\n")
    reg = scan(tmp_path)
    assert counts(reg) == {"TBD": 3, "TBR": 0}
    assert len(reg.occurrences) == 1


def test_headings_table_headers_and_category_name_do_not_count(tmp_path):
    write(tmp_path, "docs/a.md",
          "# 9. TBR/TBD Closure Plan\n\n"
          "| Category | Compliant | TBR |\n|----------|-----------|-----|\n| Mechanical | 9 | TBD |\n\n"
          "| TBD/TBR closure rate | 100% by CDR |\n")
    assert counts(scan(tmp_path)) == {"TBD": 1, "TBR": 0}


def test_revision_history_does_not_count(tmp_path):
    write(tmp_path, "docs/a.md",
          "Power TBR.\n\n"
          "## Revision History\n\n| Rev | Description |\n|-----|-------------|\n"
          "| B | TBD-001 renumbered TBD-MRD-001; TBD values added |\n\n"
          "## Appendix\n\nMass TBD.\n\n"
          "**Document History**\n\n| Version | Changes |\n|---------|---------|\n| 1.1 | TBR-001 renamed |\n")
    reg = scan(tmp_path)
    assert counts(reg) == {"TBD": 1, "TBR": 1}
    assert reg.problems == []


def test_fenced_code_counts_and_its_comment_lines_are_not_headings(tmp_path):
    write(tmp_path, "docs/a.md",
          "## Elements\n\n```\n# epoch: TBD\nRight Ascension | TBD\n# Revision history of the script\nTBR\n```\n")
    assert counts(scan(tmp_path)) == {"TBD": 2, "TBR": 1}


def test_tracked_item_counts_once_with_its_citations(tmp_path):
    write(tmp_path, "docs/req.md",
          "The frequency is TBD (TBD-SYS-001).\n\n" + TABLE +
          "| TBD-SYS-001 | Final frequency | Systems | TBD |\n"
          "| TBR-SYS-002 | Vibration levels | | CDR |\n")
    write(tmp_path, "docs/other.md", "See TBD-SYS-001.\n")
    reg = scan(tmp_path)
    assert reg.problems == []
    assert counts(reg) == {"TBD": 1, "TBR": 1}
    assert reg.occurrences == []
    item = reg.items["TBD-SYS-001"]
    assert (item.kind, item.file, item.line, item.text) == ("TBD", "docs/req.md", 5, "Final frequency")
    assert (item.owner, item.needed_by) == ("Systems", "PI")
    assert item.refs == [("docs/other.md", 1), ("docs/req.md", 1)]
    other = reg.items["TBR-SYS-002"]
    assert (other.owner, other.needed_by) == ("PI", "CDR")


def test_reused_id_fails(tmp_path):
    write(tmp_path, "docs/a.md", TABLE + "| TBD-REQ-001 | Orbit | | |\n| TBD-REQ-001 | Frequency | | |\n")
    problems = scan(tmp_path).problems
    assert any("TBD-REQ-001 is defined 2 times" in p and "docs/a.md:3" in p and "docs/a.md:4" in p
               for p in problems)


def test_id_reused_in_another_document_fails(tmp_path):
    write(tmp_path, "docs/a.md", TABLE + "| TBD-REQ-001 | Orbit | | |\n")
    write(tmp_path, "docs/b.md", TABLE + "| TBD-REQ-001 | Frequency | | |\n")
    problems = scan(tmp_path).problems
    assert any("TBD-REQ-001 is defined 2 times" in p for p in problems)
    assert any("code REQ" in p for p in problems)


def test_document_code_belongs_to_one_document(tmp_path):
    write(tmp_path, "docs/a.md", TABLE + "| TBD-ABC-001 | Orbit | | |\n")
    write(tmp_path, "docs/b.md", TABLE + "| TBR-ABC-002 | Frequency | | |\n")
    problems = scan(tmp_path).problems
    assert problems == ["code ABC is used by items defined in 2 documents: docs/a.md, docs/b.md"]


def test_id_without_document_code_fails(tmp_path):
    write(tmp_path, "docs/a.md", TABLE + "| TBD-001 | Orbit | | |\n")
    problems = scan(tmp_path).problems
    assert any(p.startswith("TBD-001 has no document code") for p in problems)


def test_cited_id_must_be_defined(tmp_path):
    write(tmp_path, "docs/a.md", "Tracked as TBR-ABC-004.\n")
    problems = scan(tmp_path).problems
    assert problems == ["TBR-ABC-004 is cited (docs/a.md:1) but defined nowhere"]


def test_closed_item_keeps_its_number_and_is_not_open(tmp_path):
    write(tmp_path, "docs/a.md", TABLE +
          "| TBD-ABC-001 | Beacon interval | Closed (2026-10-16): 60 s | |\n"
          "| TBD-ABC-002 | Orbit | | TBD |\n")
    reg = scan(tmp_path)
    assert counts(reg) == {"TBD": 1, "TBR": 0}
    assert reg.items["TBD-ABC-001"].closed
    write(tmp_path, "docs/a.md", TABLE +
          "| TBD-ABC-001 | Beacon interval | Closed (2026-10-16): 60 s | |\n"
          "| TBD-ABC-001 | Orbit | | TBD |\n")
    assert any("TBD-ABC-001 is defined 2 times" in p for p in scan(tmp_path).problems)


def test_only_docs_are_scanned_and_evidence_and_register_are_excluded(tmp_path):
    write(tmp_path, "docs/funding_audit/FINDINGS.md", "TBD-001 means four things; 343 TBD\n")
    write(tmp_path, "docs/TBD_REGISTER.md", "| TBD | TBD | TBR |\n")
    write(tmp_path, "software/x.md", "TBD\n")
    write(tmp_path, "docs/a.md", "Mass TBD\n")
    reg = scan(tmp_path)
    assert counts(reg) == {"TBD": 1, "TBR": 0}
    assert reg.problems == []


def test_register_rows(tmp_path):
    write(tmp_path, "docs/sub/a.md",
          "| Mass | TBD, see [the budget](mass.md) and `docs/sub/a.md` |\n\n" + TABLE +
          "| TBR-ABC-001 | Spring force | Mechanical | CDR |\n")
    text = render(scan(tmp_path))
    assert "](" not in text                       # links in quoted text would point elsewhere
    assert "| TBD | Mass \\| TBD, see the budget and docs/sub/a.md | PI | PI |" in text
    assert "| TBR | TBR-ABC-001 | Spring force | `docs/sub/a.md` | Mechanical | CDR |" in text
    assert "**1 TBD** and **1 TBR**" in text


def test_long_lines_are_cut_around_the_word(tmp_path):
    write(tmp_path, "docs/a.md", "x" * 300 + " value TBR here " + "y" * 300 + "\n")
    text = render(scan(tmp_path))
    row = next(line for line in text.splitlines() if "value TBR here" in line)
    assert "value TBR here" in row and len(row) < 260


def test_compare_is_exact(tmp_path):
    path = write(tmp_path, "base.json", json.dumps({"TBD": 5, "TBR": 2}))
    assert compare({"TBD": 5, "TBR": 2}, path) == []
    assert compare({"TBD": 6, "TBR": 2}, path) == ["TBD: 6, baseline 5"]
    assert compare({"TBD": 4, "TBR": 2}, path) == ["TBD: 4, baseline 5 (lower the baseline)"]


def test_write_then_check(tmp_path):
    write(tmp_path, "docs/a.md", "Mass TBD\n")
    baseline(tmp_path, 1, 0)
    assert main([], root=tmp_path) == 0
    assert (tmp_path / "docs/TBD_REGISTER.md").exists()
    assert main(["--check"], root=tmp_path) == 0
    write(tmp_path, "docs/a.md", "Mass TBD\nPower TBD\n")
    problems = check(tmp_path)
    assert any("stale" in p for p in problems)
    assert "TBD: 2, baseline 1" in problems
    assert main(["--check"], root=tmp_path) == 1


def test_check_fails_on_id_problems(tmp_path):
    write(tmp_path, "docs/a.md", TABLE + "| TBD-001 | Orbit | | |\n")
    baseline(tmp_path, 1, 0)
    main([], root=tmp_path)
    assert main(["--check"], root=tmp_path) == 1


def test_register_does_not_change_when_lines_shift(tmp_path):
    # Line numbers would make every unrelated edit above a TBD fail the check
    write(tmp_path, "docs/a.md", "Intro\n\nFrequency is TBD here\n")
    before = render(scan(tmp_path))
    write(tmp_path, "docs/a.md", "Intro\n\nA new paragraph.\n\nFrequency is TBD here\n")
    assert render(scan(tmp_path)) == before
