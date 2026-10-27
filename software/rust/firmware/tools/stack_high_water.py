#!/usr/bin/env python3
"""HIL-044: the stack high-water mark of the running firmware, through the probe.

At boot the firmware paints the unused part of the 8 KB stack budget,
0x2003E000-0x2003FFFF, with 0x57ACC0DE (firmware/src/stack.rs,
software/flight/docs/MEMORY_POLICY.md section 6). The stack grows down from
0x20040000, so the words still painted at the bottom of the budget were
never used:

    used bytes = 8192 - 4 x (leading words still painted, from 0x2003E000 up)

The check fails if the used depth exceeds the static worst case of the same
image (the "Worst case: N B of 8192 B" line of the stack_usage.txt that
scripts/stack_usage.py --report and scripts/release_package.sh write) or
the 8 KB budget (SRS-C003). Run it after the timing and fault-injection
tests, so the deepest paths have run (HIL_TEST_PLAN.md section 3.5). The
read does not halt the core.

    stack_high_water.py --stack-usage stack_usage.txt [--elf ELF] [--save dump.txt]
    stack_high_water.py --stack-usage stack_usage.txt --dump dump.txt   # no board: a saved read

The dump is the output of
    probe-rs read --chip STM32L4R5ZITx b32 0x2003E000 2048

Exit status: 0 pass, 1 fail, 2 error (no probe, unreadable input).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import List, Optional

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sqso_probe as sp  # noqa: E402

#: SRS-C003, firmware/src/stack.rs BUDGET
BUDGET = 8192
#: The initial SP (cortex-m-rt _stack_start; MEMORY_POLICY section 6.1)
STACK_TOP = 0x2004_0000
#: firmware/src/stack.rs PAINT
PAINT = 0x57AC_C0DE

WORST_CASE = re.compile(r"^Worst case: (\d+) B of (\d+) B", re.M)


def painted_from_bottom(words: List[int], paint: int = PAINT) -> int:
    """Words still painted, counted from the lowest address up to the first used one"""
    n = 0
    for w in words:
        if w != paint:
            break
        n += 1
    return n


def used_bytes(words: List[int], budget: int = BUDGET, paint: int = PAINT) -> int:
    """Bytes of the budget ever used: the budget less the untouched words at its bottom"""
    return budget - 4 * painted_from_bottom(words, paint)


def static_worst_case(report: str) -> int:
    """The worst case in bytes from a scripts/stack_usage.py report"""
    m = WORST_CASE.search(report)
    if not m:
        raise ValueError("no 'Worst case: N B of M B' line (is this scripts/stack_usage.py --report output?)")
    return int(m.group(1))


def evaluate(used: int, worst: int, budget: int = BUDGET) -> List[str]:
    """The failures (none: pass)"""
    failures = []
    if used > worst:
        failures.append(f"used {used} B exceeds the static worst case {worst} B: the static analysis "
                        "(scripts/stack_usage.py) missed a path")
    # used_bytes() cannot exceed the budget: all of it used means no painted
    # word is left at the bottom
    if used >= budget:
        failures.append(f"used {used} B: no painted word is left at the bottom of the {budget} B budget "
                        "(SRS-C003): the stack reached the guard, or the budget was not painted")
    return failures


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--stack-usage", required=True, help="scripts/stack_usage.py report of the image under test")
    ap.add_argument("--dump", help="parse this saved probe-rs read output instead of reading the board")
    ap.add_argument("--save", help="also write the raw read to this file (for the test record)")
    ap.add_argument("--elf", help="the image: check that its initial SP (_stack_start) is the expected one")
    ap.add_argument("--nm", help="nm program (default: the first of arm-none-eabi-nm, llvm-nm, rust-nm)")
    sp.add_probe_args(ap)
    args = ap.parse_args(argv)

    start = STACK_TOP - BUDGET
    try:
        worst = static_worst_case(Path(args.stack_usage).read_text())
        if args.elf:
            top, _ = sp.symbol(args.elf, "_stack_start", args.nm)
            if top != STACK_TOP:
                raise sp.ProbeError(f"_stack_start is 0x{top:08x}, not 0x{STACK_TOP:08x}: update STACK_TOP")
        if args.dump:
            text = Path(args.dump).read_text()
        else:
            probe = sp.probe_from_args(args)
            sp.report_version(probe)
            text = probe.read32_text(start, BUDGET // 4)
        if args.save:
            Path(args.save).write_text(text)
        words = sp.parse_words(text, start)
        if len(words) != BUDGET // 4:
            raise sp.ProbeError(f"{len(words)} words read, expected {BUDGET // 4} (0x{start:08x}-0x{STACK_TOP - 1:08x})")
    except (OSError, ValueError, sp.ProbeError) as e:
        print(f"error: {e}")
        return sp.ERROR

    used = used_bytes(words)
    painted = sum(1 for w in words if w == PAINT)
    print(f"Budget 0x{start:08x}-0x{STACK_TOP - 1:08x}: {painted_from_bottom(words)} words painted from the "
          f"bottom, {painted} painted in all")
    print(f"Stack used: {used} B of {BUDGET} B ({100.0 * used / BUDGET:.1f} %)")
    print(f"Static worst case: {worst} B ({args.stack_usage}); margin {worst - used} B")
    failures = evaluate(used, worst)
    for f in failures:
        print(f"FAIL: {f}")
    if not failures:
        print("PASS (HIL-044)")
    return sp.FAIL if failures else sp.PASS


if __name__ == "__main__":
    sys.exit(main())
