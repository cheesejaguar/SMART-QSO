"""Tests for the CI quality ratchet (software/flight/scripts/ratchet.sh)."""

import shutil
import subprocess
from pathlib import Path

RATCHET = Path(__file__).resolve().parents[2] / "software/flight/scripts/ratchet.sh"


def run(tmp_path, count, baseline=None, name="tool"):
    """Run a copy of ratchet.sh whose quality/ directory is tmp_path/quality."""
    (tmp_path / "scripts").mkdir(exist_ok=True)
    (tmp_path / "quality").mkdir(exist_ok=True)
    script = tmp_path / "scripts/ratchet.sh"
    shutil.copy(RATCHET, script)
    if baseline is not None:
        (tmp_path / "quality" / f"{name}.baseline").write_text(f"{baseline}\n")
    result = subprocess.run(["bash", str(script), name, str(count)],
                            capture_output=True, text=True)
    return result.returncode, result.stdout + result.stderr


def test_missing_baseline_fails(tmp_path):
    code, out = run(tmp_path, 5)
    assert code == 1
    assert "no baseline file" in out


def test_above_baseline_fails(tmp_path):
    code, out = run(tmp_path, 11, baseline=10)
    assert code == 1
    assert "11 findings, baseline 10" in out


def test_at_baseline_passes(tmp_path):
    code, _ = run(tmp_path, 10, baseline=10)
    assert code == 0


def test_below_baseline_passes_and_asks_to_lower(tmp_path):
    code, out = run(tmp_path, 7, baseline=10)
    assert code == 0
    assert "Lower" in out


def test_zero_against_nonzero_baseline_fails(tmp_path):
    # An empty report is what a crashed or misconfigured tool produces
    code, out = run(tmp_path, 0, baseline=10)
    assert code == 1
    assert "0 findings" in out


def test_zero_against_zero_baseline_passes(tmp_path):
    code, _ = run(tmp_path, 0, baseline=0)
    assert code == 0


def test_whitespace_padded_count_is_read(tmp_path):
    # BSD wc -l pads its count
    code, out = run(tmp_path, "       3", baseline=3)
    assert code == 0
    assert "3 findings (baseline 3)" in out


def test_non_numeric_count_fails(tmp_path):
    code, out = run(tmp_path, "", baseline=10)
    assert code == 1
    assert "not a count" in out
