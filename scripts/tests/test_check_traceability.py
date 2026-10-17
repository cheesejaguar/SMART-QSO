"""Tests for the traceability check (scripts/check_traceability.py, VV-06)."""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from check_traceability import measure, compare  # noqa: E402

SPEC = """# System Requirements
| ID | Requirement | Method |
|----|-------------|--------|
| SYS-001 | The spacecraft shall transmit a beacon every 60 seconds | T |
| SYS-002 | The spacecraft shall cease transmission on telecommand | T |
"""


def tree(tmp_path, rtm_text, code_refs, tps):
    (tmp_path / "docs/requirements").mkdir(parents=True)
    (tmp_path / "docs/requirements/SYSTEM_REQUIREMENTS.md").write_text(SPEC)
    (tmp_path / "docs/verification").mkdir(parents=True)
    (tmp_path / "docs/verification/RTM.md").write_text(rtm_text)
    (tmp_path / "software/flight/src").mkdir(parents=True)
    body = "\n".join(f" * @requirement {ref} text" for ref in code_refs)
    (tmp_path / "software/flight/src/x.c").write_text("/**\n" + body + "\n */\n")
    (tmp_path / "docs/test_procedures").mkdir(parents=True)
    for name, text in tps.items():
        (tmp_path / "docs/test_procedures" / name).write_text(text)
    return tmp_path


def test_clean_tree(tmp_path):
    root = tree(tmp_path, "| SYS-001 | Transmit a beacon every 60 seconds |\n",
                ["SYS-001"], {"TP-A.md": "| 1 | Check beacon | SYS-001 |\n"})
    m = measure(root, specs=["docs/requirements/SYSTEM_REQUIREMENTS.md"])
    assert m["undefined_references"] == []
    assert m["conflicting_ids"] == []
    assert m["procedures_without_ids"] == []


def test_undefined_reference(tmp_path):
    root = tree(tmp_path, "", ["SYS-001", "SRS-DATA-007"], {})
    m = measure(root, specs=["docs/requirements/SYSTEM_REQUIREMENTS.md"])
    assert m["undefined_references"] == ["SRS-DATA-007"]


def test_same_id_different_requirement(tmp_path):
    root = tree(tmp_path, "| SYS-002 | Battery capacity at least 20 Wh |\n", [], {})
    m = measure(root, specs=["docs/requirements/SYSTEM_REQUIREMENTS.md"])
    assert m["conflicting_ids"] == ["SYS-002"]


def test_procedure_without_requirement(tmp_path):
    root = tree(tmp_path, "", [], {"TP-B.md": "| 1 | Power on | LED lit |\n"})
    m = measure(root, specs=["docs/requirements/SYSTEM_REQUIREMENTS.md"])
    assert m["procedures_without_ids"] == ["docs/test_procedures/TP-B.md"]


def test_compare_fails_only_on_increase(tmp_path):
    baseline = tmp_path / "base.json"
    baseline.write_text(json.dumps({"undefined_references": 2, "conflicting_ids": 1,
                                    "procedures_without_ids": 3}))
    same = {"undefined_references": ["A-1", "B-2"], "conflicting_ids": ["C-3"],
            "procedures_without_ids": ["x", "y", "z"]}
    assert compare(same, baseline) == []
    worse = dict(same, conflicting_ids=["C-3", "D-4"])
    assert compare(worse, baseline) == ["conflicting_ids: 2, baseline 1"]


def test_compare_fails_on_decrease_until_baseline_lowered(tmp_path):
    # A baseline above the count would let fixed defects come back
    baseline = tmp_path / "base.json"
    baseline.write_text(json.dumps({"undefined_references": 2, "conflicting_ids": 1,
                                    "procedures_without_ids": 3}))
    better = {"undefined_references": ["A-1"], "conflicting_ids": ["C-3"],
              "procedures_without_ids": ["x", "y", "z"]}
    assert compare(better, baseline) == ["undefined_references: 1, baseline 2 (lower the baseline)"]
