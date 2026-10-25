#!/usr/bin/env python3
"""
Worst-case stack use of the OBC firmware image (SRS-C003: at most 8 KB).

Static analysis of the linked flight image itself, from its machine code:

1. Frame of each function: the sum of every stack-pointer decrement in it
   (push, stmdb sp!, vpush, sub sp, str [sp, #-n]!). Compiled Rust moves SP
   only in prologues (no variable-length arrays or alloca), so this is the
   frame, and an upper bound if a function had more than one prologue. Any
   other write to SP stops the analysis.
2. Call graph: direct calls and tail calls from the disassembly. Indirect
   calls (blx/bx through a register, a load into pc) are resolved by the
   reviewed table INDIRECT below: each function that makes one names the
   classes of function it can reach, and each class is a pattern over the
   functions whose address the image contains (in data, literal pools or a
   movw/movt pair). A function pointer can only point to such a function,
   so the bound is sound as long as the table is. The check fails when a
   function makes an indirect call and has no row, or when an
   address-taken function is in no class, so new function pointers are
   reviewed before they count.
3. Roots: the reset handler (thread mode) and every handler in the vector
   table, grouped by priority. A Cortex-M exception preempts only one of
   lower priority, so at most one handler per priority level is on the
   stack, each with an extended (FPU) exception frame of 104 bytes plus 4
   for alignment. The levels: NMI (-2), HardFault (-1), and everything
   else at 0, the reset value: the firmware sets no priority (checked:
   its sources must not mention one) and embassy-stm32 sets only the DMA
   interrupts', to 0 by default; the image has no DMA handler. MemManage,
   BusFault and UsageFault are disabled, so they escalate to HardFault.

worst = deepest(Reset) + sum over levels of (deepest handler at that level + 108)

A cycle in the call graph has no static bound and fails the check, as does
an instruction the analysis does not understand. The one exception is the
table REENTRY: a function that guards its own re-entry (the panic handler)
may appear twice on a path, and in its nested entry the call it guards is
not made.

With --llvm-elf, the frames are also compared with the ones LLVM reports in
the .stack_sizes section of a build with -Z emit-stack-sizes (same compiler
and sources): every function LLVM reports must agree, or the check fails.
This checks the frame parser; the bound itself is computed on the flight
image.

    python3 scripts/stack_usage.py ELF [--llvm-elf ELF] [--limit 8192] [--report FILE]
"""

from __future__ import annotations

import argparse
import re
import struct
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Dict, Iterable, List, Optional, Set, Tuple

EXCEPTION_FRAME = 104 + 4   # extended frame with FPU state, plus alignment padding
LIMIT = 8 * 1024            # SRS-C003

# Functions that guard their own re-entry: (pattern, the callee the nested entry skips, why)
REENTRY: Tuple[Tuple[str, str, str], ...] = (
    (r"^__rustc::rust_begin_unwind$", r"^assert_handle_failure$",
     "the firmware panic handler reports only the first panic (PANICKING in firmware/src/main.rs); a panic "
     "while reporting one enters it again, without the report"),
)

# Classes of address-taken function (patterns over demangled names)
CLASSES: Dict[str, str] = {
    # The Embassy task: TaskHeader.poll_fn
    "task_poll": r"^<embassy_executor::raw::TaskStorage<.*>>::poll$",
    # RawWakerVTable entries of the Embassy waker
    "waker": r"^embassy_executor::raw::waker::(clone|wake|wake_by_ref|drop)$",
    # Formatting functions: fmt::Argument.formatter, and slot 0 of a dyn Debug/Display vtable
    "fmt": r"^<.* as core::fmt::(Display|Debug|LowerHex|UpperHex|Octal|Binary|LowerExp|UpperExp)>::fmt$",
    # dyn fmt::Write vtable slots 0 and 1 (offsets 12 and 16)
    "write_str": r"^<.* as core::fmt::Write>::(write_str|write_char)$",
    # dyn fmt::Write slot 2 (offset 20): no site in the image loads it
    "write_fmt": r"^<.* as core::fmt::Write>::write_fmt$",
    # extern "C" functions the flight software hands out as C function pointers, by C type:
    # Sensor_t.read, bool (*)(Sensor_t *, double *, char *)
    "c_sensor_read": r"^sqso_ffi::sensors::read_[a-z0-9_]+$",
    # void (*)(void): the watchdog pre-reset callback
    "c_void_fn": r"^sqso_ffi::executive::save_state_before_reset$",
}

# Function making indirect calls (pattern) -> the classes it can reach, and why.
# "class@data": only members whose address is in .rodata or .data, where
# vtables and static tables are (a vtable slot holds nothing else).
INDIRECT: Tuple[Tuple[str, Tuple[str, ...], str], ...] = (
    (r"^<embassy_executor::platform::thread::Executor>::run::<.*>$", ("task_poll",),
     "polls the task through TaskHeader.poll_fn (ldr [r0, #36])"),
    (r"^core::fmt::write$", ("fmt", "write_str"),
     "calls each Argument's formatter and the output's write_str"),
    (r"^<core::fmt::Formatter>::(pad|pad_integral|pad_integral::write_prefix|pad_formatted_parts"
     r"|write_formatted_parts)$", ("write_str",),
     "write_str and write_char of the Formatter's dyn Write (vtable offsets 12 and 16)"),
    (r"^<&dyn core::fmt::Debug as core::fmt::Debug>::fmt$", ("fmt@data",),
     "the referent's Debug::fmt, from its dyn Debug vtable (offset 12)"),
    (r"^<embassy_executor::(fmt::NoneError|spawner::SpawnError) as core::fmt::Debug>::fmt$", ("write_str",),
     "Formatter::write_str of the derived Debug (offset 12)"),
    (r"^(<sqso_fsw::fixed::Fixed as core::fmt::Display>::fmt|sqso_fsw::fixed::write_\w+)$", ("write_str",),
     "Formatter::write_str for the sign, NaN and inf (offset 12)"),
    (r"^(_embassy_time_schedule_wake|<embassy_sync::waitqueue::atomic_waker::AtomicWaker>::wake"
     r"|<embassy_time_queue_utils::Queue>::next_expiration)$", ("waker",),
     "RawWakerVTable clone, wake, wake_by_ref and drop (offsets 0 to 12)"),
    (r"^(assert_handle_failure|wdt_mgr_kick)$", ("c_void_fn",),
     "void (*)(void) callbacks registered through the C API: assertion safe-mode and reset, watchdog pre-reset"),
    (r"^sqso_ffi::flight_log::record$", (),
     "LogOutputCallback, void (*)(const LogEntry_t *): no function of that type is in the image"),
    (r"^cmd_execute$", (),
     "custom command handlers, int (*)(const Command_t *, CommandResponse_t *): none is in the image"),
)

# Vector-table index -> priority level (all others, including every device interrupt, are at 0)
FIXED_PRIORITY = {2: -2, 3: -1}   # NMI, HardFault
# The firmware's sources must not set a priority, or the levels above no longer hold
PRIORITY_SOURCES = "software/rust/firmware/src"

INSN = re.compile(r"^\s*([0-9a-f]+):\t(\S+)(?:\t(.*))?$")
TARGET = re.compile(r"^(?:0x)?([0-9a-f]+)(?:\s+<[^>]*>)?$")
COND = r"(?:eq|ne|cs|hs|cc|lo|mi|pl|vs|vc|hi|ls|ge|lt|gt|le|al)?"
REG = r"r\d+|ip|lr|sl|fp|sb"


class AnalysisError(Exception):
    """The image contains something the analysis cannot bound"""


@dataclass
class Function:
    name: str
    start: int
    size: int
    frame: int = 0
    calls: Set[int] = field(default_factory=set)       # direct and tail-call targets
    indirect: List[int] = field(default_factory=list)  # addresses of indirect call sites


@dataclass
class Insn:
    addr: int
    mnemonic: str
    operands: str


def run(cmd: List[str]) -> str:
    return subprocess.run(cmd, check=True, capture_output=True, text=True).stdout


def reg_count(reglist: str) -> int:
    """Number of registers in '{r4, r5, r7-r9, lr}' or '{d8-d15}'"""
    n = 0
    for part in (p.strip() for p in reglist.strip().strip("{}").split(",") if p.strip()):
        m = re.fullmatch(r"([a-z]+)(\d+)-[a-z]+(\d+)", part)
        n += int(m.group(3)) - int(m.group(2)) + 1 if m else 1
    return n


def strip_comment(operands: str) -> str:
    return re.split(r"\s+[@;]", operands, maxsplit=1)[0].strip()


def base_of(mnemonic: str) -> str:
    return re.sub(r"\.(w|n)$", "", mnemonic)


def sp_push(insn: Insn) -> int:
    """Bytes this instruction pushes (0 for a pop or no effect on SP);
    raises for a write to SP the analysis does not understand"""
    base, ops = base_of(insn.mnemonic), strip_comment(insn.operands)
    if re.fullmatch("push" + COND, base) or (re.fullmatch("stmdb" + COND, base) and ops.startswith("sp!")):
        return 4 * reg_count(ops[ops.index("{"):])
    if re.fullmatch("vpush" + COND, base) or (re.fullmatch("vstmdb" + COND, base) and ops.startswith("sp!")):
        regs = ops[ops.index("{"):]
        return (8 if regs.lstrip("{ ").startswith("d") else 4) * reg_count(regs)
    if re.fullmatch("(pop|vpop)" + COND, base):
        return 0
    if re.fullmatch("(ldmia|ldm|vldmia)" + COND, base) and ops.startswith("sp!"):
        return 0
    m = re.fullmatch(r"sp, (?:sp, )?#(\d+)", ops)
    if m and re.fullmatch("(sub|subw)" + COND, base):
        return int(m.group(1))
    if m and re.fullmatch("(add|addw)" + COND, base):
        return 0
    m = re.search(r"\[sp, #-(\d+)\]!$", ops)
    if m and base.startswith(("str", "vstr")):
        return int(m.group(1))
    if re.search(r"\[sp\], #\d+$", ops) and base.startswith(("ldr", "vldr")):
        return 0
    dest = ops.split(",")[0].strip()
    # stores, compares and ldm/stm with sp as a base without writeback leave SP alone
    writes_sp = dest == "sp" and not base.startswith(("str", "cmp", "cmn", "tst", "teq", "vstr", "stm", "ldm"))
    # msr to MSP or PSP: objdump writes the register in capitals
    if writes_sp or "sp!" in ops or (base.startswith("msr") and re.search(r"\b[mp]?sp\b", ops, re.I)):
        raise AnalysisError(f"0x{insn.addr:08x}: unanalysable stack-pointer write '{insn.mnemonic} {ops}'")
    return 0


def branch_target(ops: str) -> Optional[int]:
    m = TARGET.match(strip_comment(ops).split(",")[-1].strip())
    return int(m.group(1), 16) if m else None


def classify(insn: Insn) -> Tuple[str, Optional[int]]:
    """('call' | 'branch' | 'indirect' | 'return' | 'other', target)"""
    base, ops = base_of(insn.mnemonic), strip_comment(insn.operands)
    if base in ("bl", "blx") and not re.fullmatch(REG, ops):
        return "call", branch_target(ops)
    if re.fullmatch("blx" + COND, base):
        return "indirect", None
    if re.fullmatch("bx" + COND, base):
        return ("return", None) if ops == "lr" else ("indirect", None)
    if re.fullmatch("b" + COND, base) or base in ("cbz", "cbnz"):
        return "branch", branch_target(ops)
    if re.fullmatch("(pop|ldmia|ldm)" + COND, base) and "pc" in ops:
        return "return", None
    if ops.split(",")[0].strip() == "pc" and not base.startswith(("cmp", "str")):
        if re.fullmatch("ldr" + COND, base) and re.search(r"\[sp\], #\d+$", ops):
            return "return", None
        return "indirect", None
    return "other", None   # including tbb/tbh: jump tables stay inside the function


def functions_of(elf: str) -> Dict[int, Function]:
    funcs: Dict[int, Function] = {}
    for line in run(["arm-none-eabi-nm", "-C", "-S", "--defined-only", elf]).splitlines():
        parts = line.split(None, 3)
        if len(parts) == 4 and parts[2] in ("t", "T"):
            start, size = int(parts[0], 16) & ~1, int(parts[1], 16)
            if size and start not in funcs:
                funcs[start] = Function(parts[3], start, size)
    return funcs


def disassemble(elf: str) -> List[Insn]:
    out = []
    for line in run(["arm-none-eabi-objdump", "-d", "--no-show-raw-insn", elf]).splitlines():
        m = INSN.match(line)
        if m:
            out.append(Insn(int(m.group(1), 16), m.group(2), m.group(3) or ""))
    return out


def sections(elf: str) -> Dict[str, Tuple[int, bytes]]:
    """Allocated sections with contents: name -> (address, bytes)"""
    result = {}
    for line in run(["arm-none-eabi-objdump", "-h", elf]).splitlines():
        p = line.split()
        if len(p) >= 7 and p[0].isdigit() and p[1] in (".vector_table", ".text", ".rodata", ".data"):
            size, vma, offset = int(p[2], 16), int(p[3], 16), int(p[5], 16)
            with open(elf, "rb") as f:
                f.seek(offset)
                result[p[1]] = (vma, f.read(size))
    return result


def address_taken(funcs: Dict[int, Function], secs: Dict[str, Tuple[int, bytes]],
                  insns: List[Insn]) -> Dict[int, Set[str]]:
    """Functions whose address (with the Thumb bit) the image contains, and
    where: 'data' (.rodata or .data) or 'code' (literal pools, movw/movt)"""
    taken: Dict[int, Set[str]] = {}
    for name, (_, blob) in secs.items():
        if name == ".vector_table":
            continue
        for i in range(0, len(blob) - 3, 2):   # halfword steps cover any 4-byte literal in Thumb code
            w = struct.unpack_from("<I", blob, i)[0]
            if w & 1 and (w & ~1) in funcs:
                taken.setdefault(w & ~1, set()).add("code" if name == ".text" else "data")
    low: Dict[str, int] = {}
    for insn in insns:
        m = re.fullmatch(rf"({REG}), #(\d+)", strip_comment(insn.operands))
        if not m:
            continue
        if insn.mnemonic.startswith("movw"):
            low[m.group(1)] = int(m.group(2))
        elif insn.mnemonic.startswith("movt") and m.group(1) in low:
            v = (int(m.group(2)) << 16) | low.pop(m.group(1))
            if v & 1 and (v & ~1) in funcs:
                taken.setdefault(v & ~1, set()).add("code")
    return taken


def owner(ordered: List[Function], addr: int) -> Optional[Function]:
    lo, hi = 0, len(ordered)
    while lo < hi:
        mid = (lo + hi) // 2
        if ordered[mid].start <= addr:
            lo = mid + 1
        else:
            hi = mid
    f = ordered[lo - 1] if lo else None
    return f if f and addr < f.start + f.size else None


def build(elf: str) -> Tuple[Dict[int, Function], Dict[int, Set[str]], List[Tuple[int, int]]]:
    """Functions with frames and calls, the address-taken functions, and the
    vector-table roots with their priority levels (Reset first, level 1:
    thread mode)"""
    funcs = functions_of(elf)
    ordered = sorted(funcs.values(), key=lambda f: f.start)
    insns = disassemble(elf)
    for insn in insns:
        f = owner(ordered, insn.addr)
        if f is None or insn.mnemonic.startswith("."):
            continue
        f.frame += sp_push(insn)
        kind, target = classify(insn)
        if kind == "call":
            if target is None or target not in funcs:
                raise AnalysisError(f"0x{insn.addr:08x} in {f.name}: call to a non-function")
            f.calls.add(target)
        elif kind == "branch" and target is not None and not f.start <= target < f.start + f.size:
            if target not in funcs:
                raise AnalysisError(f"0x{insn.addr:08x} in {f.name}: branch out of the function to 0x{target:x}")
            f.calls.add(target)   # tail call
        elif kind == "indirect":
            f.indirect.append(insn.addr)
    secs = sections(elf)
    _, vec = secs[".vector_table"]
    roots: List[Tuple[int, int]] = []   # (handler, priority level); Reset first
    for i in range(1, len(vec) // 4):   # word 0 is the initial SP
        entry = struct.unpack_from("<I", vec, 4 * i)[0]
        root = (entry & ~1, FIXED_PRIORITY.get(i, 0) if i > 1 else 1)
        if entry & 1 and (entry & ~1) in funcs and root not in roots:
            roots.append(root)
    return funcs, address_taken(funcs, secs, insns), roots


def resolve(funcs: Dict[int, Function], taken: Dict[int, Set[str]]) -> Dict[int, Set[int]]:
    """Targets of each function's indirect calls, from CLASSES and INDIRECT"""
    members: Dict[str, Set[int]] = {}
    for a, where in taken.items():
        hits = [c for c, pat in CLASSES.items() if re.search(pat, funcs[a].name)]
        if len(hits) != 1:
            raise AnalysisError(f"address-taken function {funcs[a].name} is in {len(hits)} classes "
                                f"({', '.join(hits) or 'none'}): review it and update CLASSES")
        members.setdefault(hits[0], set()).add(a)
        if "data" in where:
            members.setdefault(hits[0] + "@data", set()).add(a)
    targets: Dict[int, Set[int]] = {}
    for f in funcs.values():
        if not f.indirect:
            continue
        rows = [r for r in INDIRECT if re.search(r[0], f.name)]
        if len(rows) != 1:
            raise AnalysisError(f"{f.name} makes {len(f.indirect)} indirect call(s) and matches {len(rows)} "
                                "rows of INDIRECT: review what it calls and add a row")
        targets[f.start] = set().union(*(members.get(c, set()) for c in rows[0][1]))
    return targets


Node = Tuple[int, int]   # (function, REENTRY functions on the path up to and including it)


@dataclass
class Depths:
    depth: Dict[Node, int]            # worst stack depth from a node
    via: Dict[Node, Optional[Node]]   # the callee on that path
    guarded: Dict[int, str]           # REENTRY functions -> the callee their nested entry skips

    def node(self, addr: int) -> Node:
        """A function entered at the top of a path"""
        return addr, 1 if addr in self.guarded else 0

    def of(self, addr: int) -> int:
        return self.depth[self.node(addr)]

    def chain(self, addr: int) -> List[int]:
        out: List[int] = []
        n: Optional[Node] = self.node(addr)
        while n is not None:
            out.append(n[0])
            n = self.via[n]
        return out


def deepest(funcs: Dict[int, Function], targets: Dict[int, Set[int]]) -> Depths:
    """Worst stack depth from each function, and the path that reaches it.
    The nested entry of a REENTRY function (the second on a path) skips its
    guarded callee; a third entry, or any other cycle, fails."""
    guarded = {a: skip for a, f in funcs.items() for pat, skip, _ in REENTRY if re.search(pat, f.name)}
    result = Depths({}, {}, guarded)
    on_path: Set[Node] = set()

    def callees(addr: int, nesting: int) -> Iterable[Node]:
        for c in sorted(funcs[addr].calls | targets.get(addr, set())):
            if nesting >= 2 and addr in guarded and re.search(guarded[addr], funcs[c].name):
                continue   # the nested entry does not make the guarded call
            yield c, nesting + (1 if c in guarded else 0)

    def visit(node: Node, path: List[Node]) -> int:
        if node in result.depth:
            return result.depth[node]
        if node in on_path or node[1] > 2:
            cycle = path[path.index(node):] if node in on_path else path
            names = " -> ".join(funcs[a].name for a, _ in cycle + [node])
            raise AnalysisError(f"call-graph cycle (no static bound): {names}")
        on_path.add(node)
        path.append(node)
        best, best_via = 0, None
        for c in callees(*node):
            d = visit(c, path)
            if d > best:
                best, best_via = d, c
        path.pop()
        on_path.discard(node)
        result.depth[node] = funcs[node[0]].frame + best
        result.via[node] = best_via
        return result.depth[node]

    sys.setrecursionlimit(20000)
    for a in sorted(funcs):
        visit(result.node(a), [])
    return result


def llvm_frames(elf: str) -> Dict[int, Tuple[str, int]]:
    """Function address -> (name, stack size) from .stack_sizes (pairs of an
    address and a ULEB128 size)"""
    names = {f.start: f.name for f in functions_of(elf).values()}
    header = [l.split() for l in run(["arm-none-eabi-objdump", "-h", elf]).splitlines() if " .stack_sizes " in l]
    if not header:
        raise AnalysisError(f"{elf} has no .stack_sizes section (build it with -Z emit-stack-sizes)")
    size, offset = int(header[0][2], 16), int(header[0][5], 16)
    with open(elf, "rb") as f:
        f.seek(offset)
        blob = f.read(size)
    frames, i = {}, 0
    while i + 4 <= len(blob):
        addr = struct.unpack_from("<I", blob, i)[0] & ~1
        i += 4
        value, shift = 0, 0
        while True:
            b = blob[i]
            i += 1
            value |= (b & 0x7F) << shift
            shift += 7
            if not b & 0x80:
                break
        frames[addr] = (names.get(addr, "?"), value)
    return frames


def cross_check(funcs: Dict[int, Function], llvm: Dict[int, Tuple[str, int]]) -> List[str]:
    """Compare frames function by function. The two builds differ only in a
    build-script path inside .rodata, so .text is laid out the same and each
    function is at the same address under the same name."""
    mismatches = []
    for addr, (name, size) in sorted(llvm.items()):
        f = funcs.get(addr)
        if f is None or f.name != name:
            mismatches.append(f"0x{addr:08x} {name}: not the same function in both images")
        elif f.frame != size:
            mismatches.append(f"{name}: parsed {f.frame}, LLVM {size}")
    return mismatches


def check_priorities(root: str) -> None:
    """The firmware sets no interrupt priority (FIXED_PRIORITY's premise)"""
    import pathlib
    for path in sorted(pathlib.Path(root).rglob("*.rs")):
        for n, line in enumerate(path.read_text().splitlines(), 1):
            code = line.split("//")[0]
            if re.search(r"priority|Priority", code):
                raise AnalysisError(f"{path}:{n} sets an interrupt priority: update FIXED_PRIORITY and this check")


def report(funcs: Dict[int, Function], taken: Dict[int, Set[str]], targets: Dict[int, Set[int]],
           depths: Depths, roots: List[Tuple[int, int]], limit: int) -> Tuple[int, List[str]]:
    reset = roots[0][0]
    levels: Dict[int, List[int]] = {}
    for h, level in roots[1:]:
        levels.setdefault(level, []).append(h)
    thread = depths.of(reset)
    deepest_at = {level: max(hs, key=depths.of) for level, hs in levels.items()}
    nested = sum(depths.of(h) + EXCEPTION_FRAME for h in deepest_at.values())
    worst = thread + nested
    sites = sum(len(f.indirect) for f in funcs.values())
    lines = [
        f"Functions: {len(funcs)}; {sites} indirect call sites in {len(targets)} functions, resolved over "
        f"{len(taken)} address-taken functions (table INDIRECT)",
        f"Thread mode (Reset): {thread} B",
        f"Handlers: one per priority level, {EXCEPTION_FRAME} B exception frame each: {nested} B",
    ]
    for level in sorted(levels, reverse=True):
        hs = sorted(levels[level], key=lambda h: -depths.of(h))
        lines.append(f"  level {level:2d}: " + ", ".join(f"{funcs[h].name} {depths.of(h) + EXCEPTION_FRAME} B"
                                                       for h in hs))
    lines += [f"Worst case: {worst} B of {limit} B ({100.0 * worst / limit:.1f} %)",
              "", "Deepest thread-mode path (frame in bytes, function):"]
    lines += [f"  {funcs[a].frame:6d}  {funcs[a].name}" for a in depths.chain(reset)]
    for level in sorted(levels, reverse=True):
        h = deepest_at[level]
        lines += ["", f"Deepest handler at level {level} ({funcs[h].name}):"]
        lines += [f"  {funcs[a].frame:6d}  {funcs[a].name}" for a in depths.chain(h)]
    return worst, lines


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("elf")
    ap.add_argument("--llvm-elf", help="the same image built with -Z emit-stack-sizes, to cross-check the frames")
    ap.add_argument("--limit", type=int, default=LIMIT)
    ap.add_argument("--report", help="write the report here as well")
    ap.add_argument("--firmware-src", default=PRIORITY_SOURCES, help="sources checked for interrupt priorities")
    args = ap.parse_args(argv)

    try:
        check_priorities(args.firmware_src)
        funcs, taken, roots = build(args.elf)
        targets = resolve(funcs, taken)
        depths = deepest(funcs, targets)
        lines = []
        if args.llvm_elf:
            llvm = llvm_frames(args.llvm_elf)
            mismatches = cross_check(funcs, llvm)
            if mismatches:
                raise AnalysisError("frame parser disagrees with LLVM:\n  " + "\n  ".join(mismatches))
            lines.append(f"Frames agree with LLVM's .stack_sizes for all {len(llvm)} functions it reports; "
                         f"{len(funcs) - len(llvm)} others (compiler-builtins, assembly) are parsed only")
    except AnalysisError as e:
        print(f"::error::stack analysis: {e}")
        return 1

    worst, body = report(funcs, taken, targets, depths, roots, args.limit)
    text = "\n".join(lines + body)
    print(text)
    if args.report:
        with open(args.report, "w") as f:
            f.write(text + "\n")
    if worst > args.limit:
        print(f"::error::worst-case stack {worst} B exceeds {args.limit} B (SRS-C003)")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
