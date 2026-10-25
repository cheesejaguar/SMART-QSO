"""loop_timing.py (HIL-040) without hardware: saved probe-rs reads"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import loop_timing as lt  # noqa: E402

HZ = 80_000_000
LIMIT = 1_600_000


def sample(last=400_000, mx=520_000, overruns=0, passes=3000, hz=HZ):
    return {"core_hz": hz, "limit_cycles": LIMIT, "last_cycles": last, "max_cycles": mx,
            "overruns": overruns, "passes": passes}


def dump(s, address=0x20000000):
    return f"{address:08x}: " + " ".join(f"{s[f]:08x}" for f in lt.FIELDS) + "\n"


def test_layout_matches_the_firmware():
    # firmware/src/timing.rs: six u32 in this order; nm -S reports 0x18 bytes
    src = (Path(__file__).resolve().parents[2] / "src" / "timing.rs").read_text()
    order = [f for f in lt.FIELDS if f"pub {f}:" in src]
    assert order == list(lt.FIELDS)
    assert lt.SIZE == 0x18
    assert f"CORE_HZ: u32 = {lt.CORE_HZ:_}".replace("_", "") in src.replace("_", "")


def test_decode_and_ms():
    s = lt.decode([HZ, LIMIT, 400_000, 520_000, 0, 3000])
    assert s["max_cycles"] == 520_000 and s["passes"] == 3000
    assert lt.to_ms(520_000, HZ) == 6.5
    assert lt.to_ms(LIMIT, HZ) == 20.0


def test_pass():
    assert lt.evaluate([sample(passes=3000), sample(passes=3050)]) == []


def test_exactly_20_ms_passes():
    assert lt.evaluate([sample(mx=LIMIT)]) == []


def test_longest_pass_over_20_ms_fails():
    failures = lt.evaluate([sample(mx=LIMIT + 1, overruns=1)])
    assert any("exceeds 20 ms" in f for f in failures)
    assert any("1 pass(es) over" in f for f in failures)


def test_no_pass_recorded():
    assert any("no pass recorded" in f for f in lt.evaluate([sample(last=0, mx=0, passes=0)]))


def test_loop_stopped_or_reset_between_samples():
    assert any("not running" in f for f in lt.evaluate([sample(passes=3000), sample(passes=3000)]))
    assert any("reset between samples" in f for f in lt.evaluate([sample(passes=3000), sample(passes=12)]))


def test_saturated_counter_is_not_a_stopped_loop():
    assert lt.evaluate([sample(passes=0xFFFF_FFFF), sample(passes=0xFFFF_FFFF)]) == []


def test_other_clock_is_flagged():
    assert any("core_hz is 4000000" in f for f in lt.evaluate([sample(hz=4_000_000)]))
    assert any("core_hz is 0" in f for f in lt.evaluate([sample(hz=0)]))


def test_main_with_dumps(tmp_path, capsys):
    a, b = tmp_path / "a.txt", tmp_path / "b.txt"
    a.write_text(dump(sample(passes=3000)))
    b.write_text(dump(sample(passes=3050, mx=1_200_000)))
    assert lt.main(["--dump", str(a), "--dump", str(b)]) == 0
    out = capsys.readouterr().out
    assert "max 15.000 ms" in out and "PASS (HIL-040)" in out


def test_main_fails_on_overrun(tmp_path, capsys):
    a = tmp_path / "a.txt"
    a.write_text(dump(sample(mx=2_000_000, overruns=3)))
    assert lt.main(["--dump", str(a)]) == 1
    assert "longest pass 25.000 ms exceeds 20 ms" in capsys.readouterr().out


def test_main_reads_the_board_through_probe_rs(monkeypatch, capsys):
    reads = iter([sample(passes=3000), sample(passes=3051)])
    calls = []

    def runner(cmd):
        calls.append(cmd)
        if "--version" in cmd:
            return "probe-rs 0.32.0\n"
        if cmd[0].endswith("nm"):
            return "20000000 00000018 D SQSO_LOOP_TIMING\n"
        return dump(next(reads))

    monkeypatch.setattr(lt.sp, "run", runner)
    assert lt.main(["--elf", "fw.elf", "--nm", "arm-none-eabi-nm", "--interval", "0"]) == 0
    reads_made = [c for c in calls if c[1:2] == ["read"]]
    assert reads_made[0][-3:] == ["b32", "0x20000000", "6"] and len(reads_made) == 2
    assert "PASS (HIL-040)" in capsys.readouterr().out

