"""stack_high_water.py (HIL-044) without hardware: saved probe-rs reads"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import stack_high_water as shw  # noqa: E402

PAINT = shw.PAINT
REPORT = """\
Functions: 337; 35 indirect call sites in 17 functions, resolved over 44 address-taken functions (table INDIRECT)
Thread mode (Reset): 3376 B
Handlers: one per priority level, 108 B exception frame each: 1524 B
Worst case: 4900 B of 8192 B (59.8 %)
"""


def dump(words, start=0x2009E000):
    """probe-rs 0.32 hex-table output"""
    lines = []
    for i in range(0, len(words), 8):
        lines.append(f"{start + 4 * i:08x}: " + " ".join(f"{w:08x}" for w in words[i:i + 8]))
    return "\n".join(lines) + "\n"


def budget(used):
    """The 2048 words of a budget of which the top `used` bytes were used"""
    n = (8192 - used) // 4
    return [PAINT] * n + [0x0800_1234] * (2048 - n)


def test_high_water_counts_from_the_bottom():
    words = budget(3000)
    words[-1] = PAINT          # a used word may hold the paint pattern again: only the bottom run counts
    assert shw.painted_from_bottom(words) == (8192 - 3000) // 4
    assert shw.used_bytes(words) == 3000


def test_untouched_and_full_budget():
    assert shw.used_bytes([PAINT] * 2048) == 0
    assert shw.used_bytes([0] * 2048) == 8192


def test_static_worst_case_from_the_report():
    assert shw.static_worst_case(REPORT) == 4900


def test_report_without_worst_case():
    try:
        shw.static_worst_case("Thread mode (Reset): 3376 B\n")
    except ValueError as e:
        assert "Worst case" in str(e)
    else:
        raise AssertionError("no error")


def test_evaluate():
    assert shw.evaluate(4000, 4900) == []
    assert shw.evaluate(4900, 4900) == []
    assert "static worst case" in shw.evaluate(4904, 4900)[0]
    assert any("no painted word" in f for f in shw.evaluate(8192, 8192))


def run(tmp_path, words, report=REPORT, extra=()):
    (tmp_path / "stack_usage.txt").write_text(report)
    (tmp_path / "dump.txt").write_text(dump(words))
    return shw.main(["--stack-usage", str(tmp_path / "stack_usage.txt"), "--dump", str(tmp_path / "dump.txt"),
                     *extra])


def test_main_pass(tmp_path, capsys):
    assert run(tmp_path, budget(2400)) == 0
    out = capsys.readouterr().out
    assert "Stack used: 2400 B of 8192 B" in out and "margin 2500 B" in out and "PASS (HIL-044)" in out


def test_main_fail_above_static_worst_case(tmp_path, capsys):
    assert run(tmp_path, budget(5200)) == 1
    assert "FAIL: used 5200 B exceeds the static worst case 4900 B" in capsys.readouterr().out


def test_main_short_dump_is_an_error(tmp_path, capsys):
    assert run(tmp_path, budget(2400)[:100]) == 2
    assert "100 words read, expected 2048" in capsys.readouterr().out


def test_main_dump_of_another_region_is_an_error(tmp_path, capsys):
    (tmp_path / "stack_usage.txt").write_text(REPORT)
    (tmp_path / "dump.txt").write_text(dump(budget(2400), start=0x20000000))
    assert shw.main(["--stack-usage", str(tmp_path / "stack_usage.txt"), "--dump", str(tmp_path / "dump.txt")]) == 2
    assert "expected 0x2009e000" in capsys.readouterr().out


def test_main_saves_the_raw_read(tmp_path):
    assert run(tmp_path, budget(2400), extra=("--save", str(tmp_path / "saved.txt"))) == 0
    assert (tmp_path / "saved.txt").read_text() == (tmp_path / "dump.txt").read_text()
