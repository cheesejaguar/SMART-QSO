"""Tests for the worst-case stack analysis (scripts/stack_usage.py, SRS-C003)."""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import stack_usage as su  # noqa: E402
from stack_usage import AnalysisError, Function, Insn  # noqa: E402


def insn(mnemonic, operands=""):
    return Insn(0x08000100, mnemonic, operands)


@pytest.mark.parametrize("mnemonic, operands, pushed", [
    ("push", "{r4, r5, r6, r7, lr}", 20),
    ("push.w", "{r4-r11, lr}", 36),
    ("stmdb", "sp!, {r4, r5, lr}", 12),
    ("vpush", "{d8-d15}", 64),
    ("vpush", "{s16-s17}", 8),
    ("sub", "sp, #72\t@ 0x48", 72),
    ("sub.w", "sp, sp, #1024\t@ 0x400", 1024),
    ("subw", "sp, sp, #1196", 1196),
    ("str.w", "fp, [sp, #-4]!", 4),
    ("pop", "{r4, r5, pc}", 0),
    ("ldmia.w", "sp!, {r4, r5, pc}", 0),
    ("add", "sp, #72", 0),
    ("ldr.w", "fp, [sp], #4", 0),
    ("ldmia.w", "sp, {r0, r1, r2}", 0),      # no writeback: SP unchanged
    ("str", "r0, [sp, #4]", 0),
    ("add", "r0, sp, #8", 0),
    ("mov", "r7, sp", 0),
])
def test_stack_pointer_effects(mnemonic, operands, pushed):
    assert su.sp_push(insn(mnemonic, operands)) == pushed


@pytest.mark.parametrize("mnemonic, operands", [
    ("mov", "sp, r7"),
    ("sub.w", "sp, sp, r3"),
    ("msr", "MSP, r0"),
    ("ldr", "sp, [r0]"),
])
def test_unknown_stack_pointer_writes_stop_the_analysis(mnemonic, operands):
    with pytest.raises(AnalysisError):
        su.sp_push(insn(mnemonic, operands))


@pytest.mark.parametrize("mnemonic, operands, kind", [
    ("bl", "8016a8c <__aeabi_memclr4>", "call"),
    ("blx", "r3", "indirect"),
    ("blxne", "r9", "indirect"),
    ("bx", "ip", "indirect"),
    ("bx", "lr", "return"),
    ("pop", "{r4, r5, r7, pc}", "return"),
    ("ldr.w", "pc, [sp], #4", "return"),
    ("ldr", "pc, [r0, #4]", "indirect"),
    ("b.n", "80006d4 <f+0x36>", "branch"),
    ("beq.w", "8002560 <f+0x15c>", "branch"),
    ("cbz", "r0, 8000722 <f+0x84>", "branch"),
    ("tbb", "[pc, r0]", "other"),
])
def test_control_flow(mnemonic, operands, kind):
    assert su.classify(insn(mnemonic, operands))[0] == kind


def test_branch_targets():
    assert su.classify(insn("bl", "8016a8c <__aeabi_memclr4>")) == ("call", 0x8016a8c)
    assert su.classify(insn("cbz", "r0, 8000722 <f+0x84>")) == ("branch", 0x8000722)


def test_reg_count():
    assert su.reg_count("{r4, r5, r7-r9, lr}") == 6
    assert su.reg_count("{d8-d15}") == 8


def graph(*specs):
    """Functions from (address, name, frame, calls)"""
    funcs = {}
    for addr, name, frame, calls in specs:
        funcs[addr] = Function(name, addr, 4, frame, set(calls))
    return funcs


def test_deepest_path_sums_frames():
    funcs = graph((1, "a", 8, [2, 3]), (2, "b", 100, [4]), (3, "c", 300, []), (4, "d", 50, []))
    d = su.deepest(funcs, {})
    assert d.of(1) == 308
    assert [funcs[a].name for a in d.chain(1)] == ["a", "c"]


def test_a_cycle_has_no_bound():
    funcs = graph((1, "a", 8, [2]), (2, "b", 8, [1]))
    with pytest.raises(AnalysisError, match="cycle"):
        su.deepest(funcs, {})


def test_the_guarded_panic_handler_nests_once_without_its_report():
    panic, handler, report, reset = 1, 2, 3, 4
    funcs = graph(
        (panic, "core::panicking::panic_fmt", 32, [handler]),
        (handler, "__rustc::rust_begin_unwind", 80, [report, reset]),
        (report, "assert_handle_failure", 168, [panic]),   # a panic while reporting
        (reset, "sqso_firmware::halt", 8, []),
    )
    d = su.deepest(funcs, {})
    # first entry: panic, handler, report; nested: panic, handler, reset
    assert d.of(panic) == 32 + 80 + 168 + 32 + 80 + 8


def test_a_cycle_through_the_handler_that_skips_the_report_still_fails():
    funcs = graph(
        (1, "__rustc::rust_begin_unwind", 80, [2]),
        (2, "sqso_firmware::halt", 8, [1]),               # not the guarded call
    )
    with pytest.raises(AnalysisError, match="cycle"):
        su.deepest(funcs, {})


def test_indirect_calls_need_a_row():
    funcs = graph((1, "some::unreviewed::function", 8, []))
    funcs[1].indirect = [0x0800_0000]
    with pytest.raises(AnalysisError, match="INDIRECT"):
        su.resolve(funcs, {})


def test_address_taken_functions_need_a_class():
    funcs = graph((1, "some::callback", 8, []))
    with pytest.raises(AnalysisError, match="CLASSES"):
        su.resolve(funcs, {1: {"data"}})


def test_indirect_calls_reach_their_classes_only():
    funcs = graph(
        (1, "core::fmt::write", 56, []),
        (2, "<u32 as core::fmt::Display>::fmt", 64, []),
        (3, "<x::Out as core::fmt::Write>::write_str", 8, []),
        (4, "embassy_executor::raw::waker::wake", 8, []),
    )
    funcs[1].indirect = [0x0800_0000]
    targets = su.resolve(funcs, {2: {"code"}, 3: {"data"}, 4: {"data"}})
    assert targets == {1: {2, 3}}


def test_data_only_classes():
    funcs = graph(
        (1, "<&dyn core::fmt::Debug as core::fmt::Debug>::fmt", 8, []),
        (2, "<u32 as core::fmt::Debug>::fmt", 64, []),
        (3, "<u8 as core::fmt::Debug>::fmt", 24, []),
    )
    funcs[1].indirect = [0x0800_0000]
    # only 3 is in a vtable; 1 and 2 are taken in code (fmt::Argument)
    assert su.resolve(funcs, {1: {"code"}, 2: {"code"}, 3: {"data"}}) == {1: {3}}


def test_cross_check():
    funcs = graph((1, "a", 8, []), (2, "b", 16, []))
    assert su.cross_check(funcs, {1: ("a", 8), 2: ("b", 16)}) == []
    assert su.cross_check(funcs, {2: ("b", 24)}) == ["b: parsed 16, LLVM 24"]
    assert "not the same function" in su.cross_check(funcs, {1: ("z", 8)})[0]


def test_priorities_in_firmware_sources_are_refused(tmp_path):
    (tmp_path / "ok.rs").write_text("// interrupt priority stays at reset\nfn f() {}\n")
    su.check_priorities(str(tmp_path))
    (tmp_path / "bad.rs").write_text("fn f() { irq.set_priority(Priority::P1); }\n")
    with pytest.raises(AnalysisError, match="priority"):
        su.check_priorities(str(tmp_path))
