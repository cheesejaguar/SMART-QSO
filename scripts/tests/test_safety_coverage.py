"""Tests for the safety-critical coverage gate (software/flight/scripts/safety_coverage.py, CR-2026-003)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "software" / "flight" / "scripts"))

import safety_coverage as sc  # noqa: E402

LCOV = """SF:/x/software/rust/sqso-fsw/src/demo.rs
DA:1,5
DA:2,0
DA:3,0
BRDA:1,0,0,4
BRDA:1,0,1,-
BRDA:3,0,0,1
BRDA:3,0,1,0
end_of_record
"""


def test_parse_lcov(tmp_path):
    p = tmp_path / "lcov.info"
    p.write_text(LCOV)
    cov = sc.parse_lcov(str(p))["/x/software/rust/sqso-fsw/src/demo.rs"]
    assert cov.lines == {1: 5, 2: 0, 3: 0}
    assert cov.outcomes[1] == [True, False]
    assert cov.outcomes[3] == [True, False]


def test_test_module_start_accepts_visibility_and_path():
    for header in ("mod tests {", "pub(crate) mod tests {", "pub mod tests {", '#[path = "x_tests.rs"]'):
        src = ["fn f() {}", "#[cfg(test)]", header]
        assert sc.test_module_start(src) == 2
    assert sc.test_module_start(["fn f() {}", "#[cfg(test)]", "fn helper() {}"]) == 4


def test_exempt_on_the_line_or_the_line_above():
    src = ["a", "// COVERAGE-EXEMPT: index bounded by the loop", "b", "c // COVERAGE-EXEMPT: unreachable", "d",
           "e // COVERAGE-EXEMPT:"]
    assert sc.exempt(src, 3)
    assert sc.exempt(src, 4)
    assert not sc.exempt(src, 1)
    assert not sc.exempt(src, 6)   # a reason is required


def test_check_reports_unrun_lines_and_untaken_outcomes(tmp_path, monkeypatch):
    rust = tmp_path / "rust"
    (rust / "sqso-fsw/src").mkdir(parents=True)
    (rust / "sqso-fsw/src/demo.rs").write_text("if a && b {\n    x();\n} // COVERAGE-EXEMPT: never false\n")
    monkeypatch.setattr(sc, "RUST", rust)
    p = tmp_path / "lcov.info"
    p.write_text(LCOV)
    cov = sc.parse_lcov(str(p))["/x/software/rust/sqso-fsw/src/demo.rs"]
    problems, totals = sc.check("sqso-fsw/src/demo.rs", cov)
    assert totals == (1, 3, 2, 4)
    assert len(problems) == 2           # line 2 not run; line 1 one outcome missed; line 3 exempt
    assert any(":2: line not run" in p for p in problems)
    assert any(":1: 1 of 2 condition outcomes" in p for p in problems)
    assert sc.check("sqso-fsw/src/demo.rs", None)[0] == ["sqso-fsw/src/demo.rs: not in the coverage data"]
