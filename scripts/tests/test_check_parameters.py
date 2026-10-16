"""Tests for the parameter consistency check (scripts/check_parameters.py, VV-08)."""

import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from check_parameters import PARAMETERS, check  # noqa: E402

REPO = Path(__file__).resolve().parents[2]


def copy_files(tmp_path):
    """Copy every file the check reads into a scratch tree."""
    for param in PARAMETERS:
        for rel, _ in param.places:
            dest = tmp_path / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(REPO / rel, dest)
    return tmp_path


def test_repository_is_consistent():
    assert check(REPO) == []


def test_each_parameter_states_its_value_in_code_and_a_document():
    for param in PARAMETERS:
        files = [rel for rel, _ in param.places]
        assert any(f.startswith("software/") for f in files), param.name
        assert any(f.endswith(".md") for f in files), param.name


def test_a_changed_value_is_reported(tmp_path):
    root = copy_files(tmp_path)
    header = root / "software/flight/include/beacon.h"
    header.write_text(header.read_text().replace("BEACON_INTERVAL_SAFE_MS     180000",
                                                 "BEACON_INTERVAL_SAFE_MS     120000"))
    problems = check(root)
    assert any("beacon interval SAFE" in p for p in problems), problems


def test_a_missing_statement_is_reported(tmp_path):
    root = copy_files(tmp_path)
    baseline = root / "docs/BASELINE_PARAMETERS.md"
    baseline.write_text(baseline.read_text().replace("**N = 30 days at launch**", "N at launch"))
    problems = check(root)
    assert any("contact timeout" in p and "not found" in p for p in problems), problems
