#!/usr/bin/env python3
"""HIL-004: overrun the stack budget with the probe; the MPU guard must fault
and the independent watchdog must reset the processor.

The firmware arms an MPU region with no access over the 32 bytes below the
8 KB stack budget, 0x2003DFE0-0x2003DFFF (firmware/src/stack.rs). This
script halts the core, sets its SP to 0x2003E010 (16 bytes above the guard),
and resumes it. The next push of more than 16 bytes, or the next exception
entry, writes into the guard: MemManage, escalated to HardFault, whose
handler spins (or, if the stacking for the HardFault itself faults, the
core locks up) until the IWDG resets the processor, within the 28 s
operating timeout.

Steps, all through `probe-rs read` and `probe-rs write` (sqso_probe.py):

 1. Read SQSO_LOOP_TIMING (its `passes` must be non-zero: the loop runs) and
    the firmware's cached reset cause (RESET_CAUSE, wdt.rs), addresses from
    the ELF.
 2. Set DBGMCU_APB1FZR1.DBG_IWDG_STOP (0xE0042008 bit 12), so the IWDG
    stops while the core is halted and the halt cannot expire it; restored
    at the end.
 3. Halt (DHCSR = DBGKEY | C_HALT | C_DEBUGEN), write SP through the core
    register interface (DCRDR = 0x2003E010, DCRSR = REGWnR | 13), clear the
    vector catches so the HardFault runs the firmware's handler (DEMCR =
    TRCENA only), resume (DHCSR = DBGKEY | C_DEBUGEN). Nothing is read
    between the halt and the resume: probe-rs 0.32's `read` resumes a
    halted core when it ends.
 4. Poll the fault status (CFSR, HFSR, MMFAR at 0xE000ED28; DHCSR) and
    `passes` until the processor resets: the fault registers, which a
    system reset clears and the firmware never writes, go back to zero, or
    `passes` falls.
 5. After the reboot, read the reset cause the firmware cached at boot
    (hal_reset_cause(): RESET_WATCHDOG = 2, hal_reset.h) and RCC_CSR
    (0x40021094) for the record. The firmware clears the RCC_CSR reset
    flags at boot (RMVF), so the cached cause is the one checked, not
    IWDGRSTF (bit 29) itself.

Pass: a MemManage stacking error (CFSR.MSTKERR) or a data access violation
with MMFAR inside the guard; then a reset; then RESET_CAUSE = 2 (watchdog).

Register addresses: the Cortex-M debug and fault registers are those of the
ARMv7-M architecture (ARM DDI 0403). RCC_CSR (RCC 0x40021000 + 0x94,
IWDGRSTF bit 29, RMVF bit 23) and DBGMCU_APB1FZR1 (DBG_IWDG_STOP bit 12)
were checked against ST's STM32L4R5 SVD and stm32-metapac 21 (rcc_l4plus),
not against RM0432 itself.

Run it last in a session: it resets the board. Let the firmware run for a
minute first, so `passes` is well above what it reaches after the reboot.

    guard_test.py --elf target/thumbv7em-none-eabihf/release/sqso-firmware
    guard_test.py --dry-run     # no board: the same steps against a simulated board

Exit status: 0 pass, 1 fail, 2 error (no probe, the loop not running).
"""

from __future__ import annotations

import argparse
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Dict, List, Optional, Sequence

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sqso_probe as sp  # noqa: E402

# firmware/src/stack.rs: guard 32 bytes below the 8 KB under 0x20040000
# (the top of SRAM2, memory.x)
GUARD_LO = 0x2003_DFE0
GUARD_HI = 0x2003_E000
SP_VALUE = GUARD_HI + 16

# ARMv7-M debug registers (ARM DDI 0403, C1.6)
DHCSR = 0xE000_EDF0
DCRSR = 0xE000_EDF4
DCRDR = 0xE000_EDF8
DEMCR = 0xE000_EDFC
DBGKEY = 0xA05F << 16
C_DEBUGEN = 1 << 0
C_HALT = 1 << 1
S_LOCKUP = 1 << 19
DCRSR_REGWNR = 1 << 16
REG_SP = 13
DEMCR_TRCENA = 1 << 24

# ARMv7-M fault status (B3.2): CFSR, HFSR, DFSR, MMFAR, BFAR
CFSR = 0xE000_ED28
FAULT_WORDS = 5
MMFSR_DACCVIOL = 1 << 1
MMFSR_MSTKERR = 1 << 4
MMFSR_MMARVALID = 1 << 7
HFSR_FORCED = 1 << 30

# STM32L4R5 (ST SVD; RM0432)
DBGMCU_APB1FZR1 = 0xE004_2008
DBG_IWDG_STOP = 1 << 12
RCC_CSR = 0x4002_1094
CSR_FLAGS = {31: "LPWRRSTF", 30: "WWDGRSTF", 29: "IWDGRSTF", 28: "SFTRSTF", 27: "BORRSTF", 26: "PINRSTF",
             25: "OBLRSTF", 24: "FWRSTF"}

# The firmware
TIMING_SYMBOL = "SQSO_LOOP_TIMING"
TIMING_WORDS = 6
PASSES = 5
CAUSE_SYMBOL = "sqso_firmware::wdt::RESET_CAUSE"
CAUSES = {0: "POWER_ON", 1: "BROWNOUT", 2: "WATCHDOG", 3: "SOFTWARE", 4: "PIN", 5: "UNKNOWN",
          0xFFFF_FFFF: "not read yet"}
RESET_WATCHDOG = 2


def csr_flags(csr: int) -> List[str]:
    """The reset flags set in RCC_CSR"""
    return [name for bit, name in sorted(CSR_FLAGS.items(), reverse=True) if csr & (1 << bit)]


@dataclass
class Observation:
    """What the probe saw"""

    passes_before: int = 0
    cause_before: Optional[int] = None
    cfsr: Optional[int] = None       # the last non-zero fault status seen
    hfsr: Optional[int] = None
    mmfar: Optional[int] = None
    lockup: bool = False
    rebooted: bool = False
    reboot_after_s: Optional[float] = None
    cause_after: Optional[int] = None
    csr_after: Optional[int] = None
    passes_after: Optional[int] = None


def guard_fault(obs: Observation) -> bool:
    """A MemManage fault from the guard: a stacking error, or a data access with MMFAR in the guard"""
    if obs.cfsr is None:
        return False
    if obs.cfsr & MMFSR_MSTKERR:
        return True
    return bool(obs.cfsr & MMFSR_DACCVIOL and obs.cfsr & MMFSR_MMARVALID and obs.mmfar is not None
                and GUARD_LO <= obs.mmfar < GUARD_HI)


def evaluate(obs: Observation) -> List[str]:
    """The failures (none: pass)"""
    failures = []
    if obs.cfsr is None:
        failures.append("no fault seen after the SP was moved to the guard")
    elif not guard_fault(obs):
        failures.append(f"a fault, but not the guard's: CFSR 0x{obs.cfsr:08x}, HFSR 0x{obs.hfsr or 0:08x}, "
                        f"MMFAR 0x{obs.mmfar or 0:08x}")
    if not obs.rebooted:
        failures.append("no reset seen: the IWDG did not reset the processor")
    elif obs.cause_after != RESET_WATCHDOG:
        name = CAUSES.get(obs.cause_after, str(obs.cause_after)) if obs.cause_after is not None else "unread"
        failures.append(f"the next boot reports reset cause {name}, not WATCHDOG (hal_reset_cause)")
    return failures


def run(probe: sp.Probe, timing: int, cause: int, timeout: float = 60.0, interval: float = 1.0,
        boot_wait: float = 3.0, clock: Callable[[], float] = time.monotonic,
        sleep: Callable[[float], None] = time.sleep, say: Callable[[str], None] = print) -> Observation:
    """Steps 1-5 of the module documentation"""
    obs = Observation()
    obs.passes_before = probe.read32(timing, TIMING_WORDS)[PASSES]
    obs.cause_before = probe.read32(cause, 1)[0]
    if obs.passes_before == 0:
        raise sp.ProbeError("SQSO_LOOP_TIMING.passes is 0: the main loop is not running")
    if obs.passes_before < 500:
        say(f"note: only {obs.passes_before} passes so far; a reboot is detected more surely after a minute")
    freeze = probe.read32(DBGMCU_APB1FZR1, 1)[0]
    try:
        probe.write32(DBGMCU_APB1FZR1, freeze | DBG_IWDG_STOP)
        probe.write32(DHCSR, DBGKEY | C_HALT | C_DEBUGEN)
        probe.write32(DCRDR, SP_VALUE)
        probe.write32(DCRSR, DCRSR_REGWNR | REG_SP)
        probe.write32(DEMCR, DEMCR_TRCENA)
        probe.write32(DHCSR, DBGKEY | C_DEBUGEN)
        start = clock()
        say(f"SP set to 0x{SP_VALUE:08x} and the core resumed; waiting up to {timeout:g} s for the reset")
        last = obs.passes_before
        while clock() - start < timeout:
            sleep(interval)
            cfsr, hfsr, _dfsr, mmfar, _bfar = probe.read32(CFSR, FAULT_WORDS)
            dhcsr = probe.read32(DHCSR, 1)[0]
            passes = probe.read32(timing, TIMING_WORDS)[PASSES]
            faulted = bool(cfsr or hfsr)
            if faulted:
                if obs.cfsr is None:
                    say(f"fault after {clock() - start:.1f} s: CFSR 0x{cfsr:08x}, HFSR 0x{hfsr:08x}, "
                        f"MMFAR 0x{mmfar:08x}{', core locked up' if dhcsr & S_LOCKUP else ''}")
                obs.cfsr, obs.hfsr, obs.mmfar = cfsr, hfsr, mmfar
                obs.lockup = obs.lockup or bool(dhcsr & S_LOCKUP)
            if (obs.cfsr is not None and not faulted) or passes < last:
                obs.rebooted = True
                obs.reboot_after_s = clock() - start
                break
            last = passes
        if obs.rebooted:
            sleep(boot_wait)
            obs.cause_after = probe.read32(cause, 1)[0]
            obs.csr_after = probe.read32(RCC_CSR, 1)[0]
            obs.passes_after = probe.read32(timing, TIMING_WORDS)[PASSES]
    finally:
        probe.write32(DBGMCU_APB1FZR1, freeze)
    return obs


class SimulatedBoard:
    """probe-rs answers from a model of the board, for --dry-run and the tests:
    the loop runs until the SP is moved into the guard's reach, then the core
    faults, and after `fault_polls` reads of the fault status it resets"""

    def __init__(self, timing: int = 0x2000_0000, cause: int = 0x2000_03F0, fault_polls: int = 3,
                 fault: Sequence[int] = (MMFSR_MSTKERR, HFSR_FORCED, 0, 0, 0), cause_after: int = RESET_WATCHDOG,
                 resets: bool = True):
        self.timing, self.cause = timing, cause
        self.mem: Dict[int, int] = {DBGMCU_APB1FZR1: 0, DHCSR: 0x0003_0001, DEMCR: DEMCR_TRCENA,
                                    RCC_CSR: 0x0C00_0600, cause: 4}
        self.passes = 3000
        self.fault_polls, self.fault, self.cause_after, self.resets = fault_polls, list(fault), cause_after, resets
        self.halted = self.faulted = False
        self.sp: Optional[int] = None
        self.polls = 0

    def __call__(self, cmd: Sequence[str]) -> str:
        if "--version" in cmd:
            return "probe-rs 0.32.0 (git commit: simulated)\n"
        i = list(cmd).index("b32")
        addr, rest = int(cmd[i + 1], 0), list(cmd[i + 2:])
        if cmd[1] == "write":
            for k, v in enumerate(rest):
                self.write(addr + 4 * k, int(v, 0))
            return ""
        words = [self.read(addr + 4 * k) for k in range(int(rest[0]))]
        lines = [f"{addr + 32 * j:08x}: " + " ".join(f"{w:08x}" for w in words[8 * j:8 * j + 8])
                 for j in range(0, (len(words) + 7) // 8)]
        return "\n".join(lines) + "\n"

    def write(self, addr: int, value: int) -> None:
        if addr == DHCSR:
            was = self.halted
            self.halted = bool(value & C_HALT)
            if was and not self.halted and self.sp is not None and self.sp - 32 < GUARD_HI:
                self.faulted = True
        elif addr == DCRSR and value == DCRSR_REGWNR | REG_SP and self.halted:
            self.sp = self.mem.get(DCRDR)
        self.mem[addr] = value

    def read(self, addr: int) -> int:
        if not self.halted and not self.faulted:
            self.passes += 50
        if addr == CFSR and self.faulted:
            self.polls += 1
            if self.polls > self.fault_polls and self.resets:
                self.faulted, self.sp, self.passes = False, None, 10
                self.mem[self.cause] = self.cause_after
                self.mem[RCC_CSR] = 0x0C00_0600 & ~(0xFF << 24)
        if self.faulted and CFSR <= addr < CFSR + 4 * FAULT_WORDS:
            return self.fault[(addr - CFSR) // 4]
        if CFSR <= addr < CFSR + 4 * FAULT_WORDS:
            return 0
        if addr == DHCSR:  # S_REGRDY, S_HALT, C_HALT, C_DEBUGEN
            return 0x0003_0003 if self.halted else 0x0001_0001
        if self.timing <= addr < self.timing + 4 * TIMING_WORDS:
            k = (addr - self.timing) // 4
            return [80_000_000, 1_600_000, 400_000, 520_000, 0, self.passes][k]
        return self.mem.get(addr, 0)


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    src = ap.add_mutually_exclusive_group(required=True)
    src.add_argument("--elf", help="the image under test (symbol addresses are read from it)")
    src.add_argument("--dry-run", action="store_true", help="run the steps against a simulated board and "
                     "print the probe-rs commands")
    ap.add_argument("--timeout", type=float, default=60.0, help="seconds to wait for the reset (default 60)")
    ap.add_argument("--interval", type=float, default=1.0, help="seconds between polls (default 1)")
    ap.add_argument("--nm", help="nm program (default: the first of arm-none-eabi-nm, llvm-nm, rust-nm)")
    sp.add_probe_args(ap)
    args = ap.parse_args(argv)

    probe = sp.probe_from_args(args)
    try:
        if args.dry_run:
            board = SimulatedBoard()
            probe.runner = board
            timing, cause = board.timing, board.cause
            obs = run(probe, timing, cause, args.timeout, interval=0, boot_wait=0, sleep=lambda _s: None)
            print("probe-rs commands (simulated board):")
            for cmd in probe.log:
                print("  " + " ".join(cmd))
        else:
            timing, _ = sp.symbol(args.elf, TIMING_SYMBOL, args.nm)
            cause, _ = sp.symbol(args.elf, CAUSE_SYMBOL, args.nm)
            sp.report_version(probe)
            obs = run(probe, timing, cause, args.timeout, args.interval)
    except (OSError, ValueError, sp.ProbeError) as e:
        print(f"error: {e}")
        return sp.ERROR

    print(f"Before: {obs.passes_before} passes, reset cause {CAUSES.get(obs.cause_before or 0, obs.cause_before)}")
    if obs.cfsr is not None:
        print(f"Fault: CFSR 0x{obs.cfsr:08x}, HFSR 0x{obs.hfsr or 0:08x}, MMFAR 0x{obs.mmfar or 0:08x}"
              f"{' (locked up)' if obs.lockup else ' (HardFault handler)'}; guard: {'yes' if guard_fault(obs) else 'no'}")
    if obs.rebooted:
        print(f"Reset {obs.reboot_after_s:.1f} s after the resume; next boot: reset cause "
              f"{CAUSES.get(obs.cause_after or 0, obs.cause_after)}, RCC_CSR 0x{obs.csr_after or 0:08x} "
              f"(flags {', '.join(csr_flags(obs.csr_after or 0)) or 'cleared'}), {obs.passes_after} passes")
    failures = evaluate(obs)
    for f in failures:
        print(f"FAIL: {f}")
    if not failures:
        print("PASS (HIL-004)" + (" [simulated]" if args.dry_run else ""))
    return sp.FAIL if failures else sp.PASS


if __name__ == "__main__":
    sys.exit(main())
