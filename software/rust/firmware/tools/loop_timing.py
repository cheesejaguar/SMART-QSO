#!/usr/bin/env python3
"""HIL-040: the main-loop pass times of the running firmware, through the probe.

The firmware times every Executive::step() pass with the DWT cycle counter
and keeps the result in the static SQSO_LOOP_TIMING (firmware/src/timing.rs),
six little-endian 32-bit words:

    core_hz, limit_cycles, last_cycles, max_cycles, overruns, passes

This script finds the static's address in the image (nm), reads it with
`probe-rs read` (the core is not halted) and reports the last and longest
pass and the passes over the limit. Two or more samples (default 2, 1 s
apart) also check that the loop runs: `passes` must grow between them.

Fails if the longest pass exceeds 20 ms (SRS-P001), if any pass did
(`overruns`), if no pass was recorded or `passes` did not grow, or if the
OBC reset between samples (`passes` fell).

    loop_timing.py --elf target/thumbv7em-none-eabihf/release/sqso-firmware
    loop_timing.py --address 0x20000000 --samples 5 --interval 10
    loop_timing.py --dump first.txt --dump second.txt   # no board: saved reads

A dump is the output of
    probe-rs read --chip STM32L4R5ZITx b32 <address of SQSO_LOOP_TIMING> 6

Exit status: 0 pass, 1 fail, 2 error (no probe, unreadable input).
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sqso_probe as sp  # noqa: E402

SYMBOL = "SQSO_LOOP_TIMING"
FIELDS = ("core_hz", "limit_cycles", "last_cycles", "max_cycles", "overruns", "passes")
SIZE = 4 * len(FIELDS)
#: SRS-P001
LIMIT_MS = 20.0
#: firmware/src/timing.rs CORE_HZ (clocks() in main.rs)
CORE_HZ = 80_000_000


def decode(words: List[int]) -> Dict[str, int]:
    """The fields of SQSO_LOOP_TIMING from its six words"""
    if len(words) != len(FIELDS):
        raise ValueError(f"{len(words)} words, SQSO_LOOP_TIMING has {len(FIELDS)}")
    return dict(zip(FIELDS, words))


def to_ms(cycles: int, core_hz: int) -> float:
    return 1000.0 * cycles / core_hz


def evaluate(samples: List[Dict[str, int]], limit_ms: float = LIMIT_MS) -> List[str]:
    """The failures over one or more samples in time order (none: pass)"""
    failures = []
    last = samples[-1]
    hz = last["core_hz"]
    if hz != CORE_HZ:
        failures.append(f"core_hz is {hz}, expected {CORE_HZ}: not this firmware, or the clock changed "
                        "(update CORE_HZ here and in firmware/src/timing.rs)")
        if hz == 0:
            return failures
    if last["passes"] == 0:
        failures.append("no pass recorded: the main loop has not run, or the cycle counter is off")
    worst = max(s["max_cycles"] for s in samples)
    if to_ms(worst, hz) > limit_ms:
        failures.append(f"longest pass {to_ms(worst, hz):.3f} ms exceeds {limit_ms:g} ms (SRS-P001)")
    over = max(s["overruns"] for s in samples)
    if over:
        failures.append(f"{over} pass(es) over the firmware's limit of {to_ms(last['limit_cycles'], hz):g} ms")
    for a, b in zip(samples, samples[1:]):
        if b["passes"] < a["passes"]:
            failures.append(f"passes fell from {a['passes']} to {b['passes']}: the OBC reset between samples")
        elif b["passes"] == a["passes"] and b["passes"] != 0xFFFF_FFFF:
            failures.append(f"passes stayed at {a['passes']}: the main loop is not running")
    return failures


def describe(s: Dict[str, int]) -> str:
    hz = s["core_hz"] or CORE_HZ
    return (f"last {to_ms(s['last_cycles'], hz):.3f} ms, max {to_ms(s['max_cycles'], hz):.3f} ms "
            f"({s['max_cycles']} cycles), overruns {s['overruns']}, passes {s['passes']}")


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    src = ap.add_mutually_exclusive_group(required=True)
    src.add_argument("--elf", help="the image under test (SQSO_LOOP_TIMING's address is read from it)")
    src.add_argument("--address", type=lambda s: int(s, 0), help="SQSO_LOOP_TIMING's address, without an ELF")
    src.add_argument("--dump", action="append", help="a saved probe-rs read of the static (repeat for samples)")
    ap.add_argument("--samples", type=int, default=2, help="reads to take (default 2)")
    ap.add_argument("--interval", type=float, default=1.0, help="seconds between reads (default 1)")
    ap.add_argument("--limit-ms", type=float, default=LIMIT_MS, help=f"pass limit (default {LIMIT_MS:g} ms, SRS-P001)")
    ap.add_argument("--nm", help="nm program (default: the first of arm-none-eabi-nm, llvm-nm, rust-nm)")
    sp.add_probe_args(ap)
    args = ap.parse_args(argv)

    try:
        if args.dump:
            samples = [decode(sp.parse_words(Path(d).read_text())) for d in args.dump]
        else:
            address = args.address
            if args.elf:
                address, size = sp.symbol(args.elf, SYMBOL, args.nm)
                if size is not None and size != SIZE:
                    raise sp.ProbeError(f"{SYMBOL} is {size} B, this script reads {SIZE} B: update FIELDS")
            print(f"{SYMBOL} at 0x{address:08x}")
            probe = sp.probe_from_args(args)
            sp.report_version(probe)
            samples = []
            for i in range(max(1, args.samples)):
                if i:
                    time.sleep(args.interval)
                samples.append(decode(probe.read32(address, len(FIELDS))))
    except (OSError, ValueError, sp.ProbeError) as e:
        print(f"error: {e}")
        return sp.ERROR

    for i, s in enumerate(samples, 1):
        print(f"Sample {i}: {describe(s)}")
    failures = evaluate(samples, args.limit_ms)
    for f in failures:
        print(f"FAIL: {f}")
    if not failures:
        print("PASS (HIL-040)")
    return sp.FAIL if failures else sp.PASS


if __name__ == "__main__":
    sys.exit(main())
