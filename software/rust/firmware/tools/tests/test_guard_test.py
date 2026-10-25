"""guard_test.py (HIL-004) without hardware: the evaluation, and the probe-rs
sequence against the simulated board"""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import guard_test as gt  # noqa: E402
import sqso_probe as sp  # noqa: E402


def test_addresses_match_the_firmware():
    # firmware/src/stack.rs: an 8 KB budget under 0x200A0000 and a 32 B guard below it
    src = (Path(__file__).resolve().parents[2] / "src" / "stack.rs").read_text()
    assert "BUDGET: u32 = 8 * 1024" in src and "GUARD: u32 = 32" in src
    assert gt.GUARD_HI == 0x200A0000 - 8 * 1024 and gt.GUARD_HI - gt.GUARD_LO == 32
    assert gt.SP_VALUE == 0x2009E010
    # wdt.rs and hal_reset.h: RESET_WATCHDOG = 2, the cached cause in RESET_CAUSE
    wdt = (Path(__file__).resolve().parents[2] / "src" / "wdt.rs").read_text()
    assert "const RESET_WATCHDOG: u32 = 2;" in wdt and "static RESET_CAUSE: AtomicU32" in wdt


def test_register_constants():
    # ARMv7-M: DHCSR key and bits, DCRSR write of R13
    assert gt.DBGKEY | gt.C_HALT | gt.C_DEBUGEN == 0xA05F0003
    assert gt.DCRSR_REGWNR | gt.REG_SP == 0x0001000D
    # STM32L4R5 (ST SVD): RCC_CSR at 0x40021000 + 0x94, IWDGRSTF bit 29
    assert gt.RCC_CSR == 0x40021000 + 0x94 and gt.CSR_FLAGS[29] == "IWDGRSTF"
    assert gt.csr_flags(1 << 29 | 1 << 26) == ["IWDGRSTF", "PINRSTF"]
    assert gt.csr_flags(0x600) == []


def obs(**kw):
    o = gt.Observation(passes_before=3000, cause_before=4, rebooted=True, reboot_after_s=29.0,
                       cause_after=gt.RESET_WATCHDOG, csr_after=0x600, passes_after=150)
    o.cfsr, o.hfsr, o.mmfar = gt.MMFSR_MSTKERR, gt.HFSR_FORCED, 0
    for k, v in kw.items():
        setattr(o, k, v)
    return o


def test_pass_on_stacking_error_then_watchdog_reset():
    assert gt.evaluate(obs()) == []


def test_pass_on_access_violation_in_the_guard():
    v = gt.MMFSR_DACCVIOL | gt.MMFSR_MMARVALID
    assert gt.evaluate(obs(cfsr=v, mmfar=0x2009DFF8)) == []
    assert "not the guard's" in gt.evaluate(obs(cfsr=v, mmfar=0x2009E000))[0]
    assert "not the guard's" in gt.evaluate(obs(cfsr=gt.MMFSR_DACCVIOL, mmfar=0x2009DFF8))[0]


def test_other_fault_fails():
    assert "not the guard's" in gt.evaluate(obs(cfsr=1 << 16))[0]   # UNDEFINSTR


def test_no_fault_no_reset():
    failures = gt.evaluate(obs(cfsr=None, hfsr=None, mmfar=None, rebooted=False))
    assert "no fault seen" in failures[0] and "no reset seen" in failures[1]


def test_reset_by_something_else_fails():
    assert "SOFTWARE, not WATCHDOG" in gt.evaluate(obs(cause_after=3))[0]


def run_simulated(board, **kw):
    probe = sp.Probe(runner=board)
    o = gt.run(probe, board.timing, board.cause, interval=0, boot_wait=0, sleep=lambda _s: None,
               say=lambda _m: None, **kw)
    return probe, o


def test_sequence_against_the_simulated_board():
    board = gt.SimulatedBoard()
    probe, o = run_simulated(board)
    assert gt.evaluate(o) == []
    writes = [(int(c[-2], 16), int(c[-1], 16)) for c in probe.log if c[1] == "write"]
    assert writes == [
        (gt.DBGMCU_APB1FZR1, gt.DBG_IWDG_STOP),          # IWDG stops while halted
        (gt.DHCSR, 0xA05F0003),                           # halt
        (gt.DCRDR, 0x2009E010), (gt.DCRSR, 0x0001000D),   # SP
        (gt.DEMCR, gt.DEMCR_TRCENA),                      # no vector catch
        (gt.DHCSR, 0xA05F0001),                           # resume
        (gt.DBGMCU_APB1FZR1, 0),                          # restored
    ]
    # nothing is read between the halt and the resume (probe-rs read resumes a halted core)
    halt = next(i for i, c in enumerate(probe.log) if c[1] == "write" and c[-1] == "0xa05f0003")
    resume = next(i for i, c in enumerate(probe.log) if c[1] == "write" and c[-1] == "0xa05f0001")
    assert all(c[1] == "write" for c in probe.log[halt:resume + 1])


def test_simulated_board_that_never_resets():
    _, o = run_simulated(gt.SimulatedBoard(resets=False), timeout=0.05)
    assert o.cfsr == gt.MMFSR_MSTKERR and not o.rebooted
    assert any("no reset seen" in f for f in gt.evaluate(o))


def test_simulated_reset_with_wrong_cause():
    _, o = run_simulated(gt.SimulatedBoard(cause_after=3))
    assert any("SOFTWARE" in f for f in gt.evaluate(o))


def test_freeze_bit_is_restored_when_the_probe_fails():
    board = gt.SimulatedBoard()
    calls = []

    def flaky(cmd):
        calls.append(cmd)
        if cmd[1] == "read" and cmd[-2] == f"0x{gt.CFSR:08x}":
            raise sp.ProbeError("probe disconnected")
        return board(cmd)

    with pytest.raises(sp.ProbeError):
        gt.run(sp.Probe(runner=flaky), board.timing, board.cause, interval=0, sleep=lambda _s: None,
               say=lambda _m: None)
    assert calls[-1][1] == "write" and calls[-1][-2:] == [f"0x{gt.DBGMCU_APB1FZR1:08x}", "0x00000000"]


def test_loop_not_running_is_an_error():
    board = gt.SimulatedBoard()
    board.passes = 0
    board.halted = True   # the simulated loop does not count while halted
    with pytest.raises(sp.ProbeError, match="not running"):
        gt.run(sp.Probe(runner=board), board.timing, board.cause, say=lambda _m: None)


def test_dry_run(capsys):
    assert gt.main(["--dry-run"]) == 0
    out = capsys.readouterr().out
    assert "probe-rs write --chip STM32L4R5ZITx b32 0xe000edf8 0x2009e010" in out
    assert "PASS (HIL-004) [simulated]" in out
