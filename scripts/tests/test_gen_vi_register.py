"""Tests for the verification-item register (scripts/gen_vi_register.py, RID P3P-01)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import gen_vi_register as g  # noqa: E402

DESIGN = """# Board design
**Document ID**: SQSO-TST-001

| ID | Test | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-TST-1 | Ripple on the bus | ≤ 200 mV peak-to-peak | SYS-713 | EM |
| V-TST-2 | Drawing check | Mated height ≤ 7.7 mm | §4 | 2026-12-01 |
| V-TST-3 | Vibration | No damage | SYS-703 | L-4 M |
| V-TST-4 | Sine signature | Shift < 10 % | SYS-705 | With V-TST-3 |
"""

COMBINED = """# RF design
| ID | Test and pass criterion | Verifies | When |
|----|-------------------------|----------|------|
| VI-TST-01 | Conducted power at the port. Pass: 27.0 ± 0.5 dBm | LB-1 | EM, FM |
"""

SCHEDULE = """# Schedule
## 5. Decision Points and Dated Actions

| ID | Decision or action | Owner | Due | Pass criterion or outcome | Parent RID | Status |
|----|--------------------|-------|-----|---------------------------|------------|--------|
| DA-01 | Name the board | PI | 2026-12-01 | Names recorded | PM-01 | Open |
"""


def repo(tmp_path, design=DESIGN, schedule=SCHEDULE, extra=None):
    (tmp_path / "hardware").mkdir()
    (tmp_path / "hardware/DESIGN.md").write_text(design, encoding="utf-8")
    (tmp_path / "docs/verification").mkdir(parents=True)
    (tmp_path / "mission").mkdir()
    (tmp_path / "mission/SCHEDULE.md").write_text(schedule, encoding="utf-8")
    for name, text in (extra or {}).items():
        (tmp_path / name).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / name).write_text(text, encoding="utf-8")
    return tmp_path


def by_id(root):
    items, actions, problems, _ = g.build(root)
    return {i.ident: i for i in items}, actions, problems


def test_items_get_events_dates_and_criteria(tmp_path):
    items, actions, problems = by_id(repo(tmp_path))
    assert problems == []
    assert items["V-TST-1"].event == "EMT" and " to " in items["V-TST-1"].date
    assert items["V-TST-1"].criterion == "≤ 200 mV peak-to-peak"
    assert items["V-TST-2"].event == "date" and items["V-TST-2"].date == "2026-12-01"
    assert items["V-TST-3"].date == "2028-07-01"            # L-4 M from the 2028-11-01 planning launch
    assert items["V-TST-4"].sort_date == items["V-TST-3"].sort_date and "with V-TST-3" in items["V-TST-4"].date
    assert [a.ident for a in actions] == ["DA-01"]


def test_combined_test_and_criterion_cell_is_split(tmp_path):
    items, _, problems = by_id(repo(tmp_path, extra={"docs/RF.md": COMBINED}))
    assert problems == []
    assert items["VI-TST-01"].criterion == "27.0 ± 0.5 dBm"
    assert items["VI-TST-01"].test.startswith("Conducted power")


def test_missing_criterion_or_timing_is_a_problem(tmp_path):
    bad = DESIGN.replace("| ≤ 200 mV peak-to-peak |", "|  |").replace("| 2026-12-01 |", "| sometime |")
    _, _, problems = by_id(repo(tmp_path, design=bad))
    assert any("V-TST-1 has no pass criterion" in p for p in problems)
    assert any("V-TST-2 has no dated event" in p for p in problems)


def test_duplicate_in_one_document_is_a_problem(tmp_path):
    dup = DESIGN + "| V-TST-1 | Again | x | y | EM |\n"
    _, _, problems = by_id(repo(tmp_path, design=dup))
    assert any("V-TST-1 defined twice" in p for p in problems)


def test_action_needs_owner_date_and_criterion(tmp_path):
    bad = SCHEDULE.replace("| PI | 2026-12-01 | Names recorded |", "|  | soon |  |")
    _, _, problems = by_id(repo(tmp_path, schedule=bad))
    assert any("no owner" in p for p in problems)
    assert any("no due date" in p for p in problems)
    assert any("no pass criterion" in p for p in problems)


def test_no_tbd_words_are_copied(tmp_path):
    items, _, _ = by_id(repo(tmp_path, design=DESIGN.replace("No damage", "Level TBD")))
    assert "TBD" not in items["V-TST-3"].criterion


def test_check_fails_when_stale_and_passes_after_write(tmp_path):
    root = repo(tmp_path)
    assert g.main(["--check"], root=root) == 1
    assert g.main([], root=root) == 0
    assert g.main(["--check"], root=root) == 0
    text = (root / g.REGISTER).read_text(encoding="utf-8")
    assert "| V-TST-1 |" in text and "| DA-01 |" in text
    (root / "hardware/DESIGN.md").write_text(DESIGN.replace("200 mV", "150 mV"), encoding="utf-8")
    assert g.main(["--check"], root=root) == 1


def test_rid_citations_expand_ranges_and_lists():
    assert g.expand_ids("EM tests VI-RFB-01..03") == ["VI-RFB-01", "VI-RFB-02", "VI-RFB-03"]
    assert g.expand_ids("VI-01 to VI-03 on the EM") == ["VI-01", "VI-02", "VI-03"]
    assert g.expand_ids("VI-RF-01, 03, 08") == ["VI-RF-01", "VI-RF-03", "VI-RF-08"]
    assert g.expand_ids("SV-12 and AV-V4") == ["AV-V4"]
