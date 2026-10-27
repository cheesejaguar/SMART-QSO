"""Shared code of the bring-up probe scripts: the probe-rs command line, its
`read` output, and symbol addresses from the image.

The scripts drive the `probe-rs` command-line tool (crate probe-rs-tools)
with two of its subcommands only, `probe-rs read` and `probe-rs write`,
which every release since 0.2x has. They were written against probe-rs
0.32.0 (2026-07), whose `read` prints a hex table by default
("2003e000: 57acc0de 57acc0de ..."); the plain list of earlier releases
and of `--format simple-hex` is read as well. Two behaviours of 0.32.0
the scripts allow for: `read` resumes a halted core when it ends, and on
STM32L4 a session leaves DHCSR and DEMCR as they were when it ends (the
ST sequence restores DBGMCU_CR only). Older or newer releases may differ;
`probe-rs --version` is printed by each script.

Python 3 standard library only. Symbol addresses come from the ELF via
`nm` (arm-none-eabi-nm, llvm-nm or rust-nm, whichever is installed).
"""

from __future__ import annotations

import re
import shutil
import subprocess
from dataclasses import dataclass, field
from typing import Callable, List, Optional, Sequence, Tuple

#: The probe-rs release the scripts were written against
PROBE_RS_TESTED = "0.32.0"
#: probe-rs's name for the part (firmware/.cargo/config.toml runner)
CHIP = "STM32L4R5ZITx"
#: nm programs tried in order when none is given
NM_TOOLS = ("arm-none-eabi-nm", "llvm-nm", "rust-nm")

#: Exit codes of the scripts
PASS, FAIL, ERROR = 0, 1, 2

_HEX_WORD = re.compile(r"(?:0x)?([0-9a-fA-F]{1,8})")


class ProbeError(Exception):
    """probe-rs or nm failed, or printed something the scripts cannot read"""


def parse_words(text: str, start: Optional[int] = None) -> List[int]:
    """The 32-bit words of `probe-rs read b32` output.

    Accepts the hex table of probe-rs 0.32 (an address column ending in
    ':' then up to eight words per line) and a plain whitespace-separated
    list. With `start`, the address column (when present) must follow on
    from it, so a dump of the wrong region is not taken for the right one.
    """
    words: List[int] = []
    for n, line in enumerate(text.splitlines(), 1):
        tokens = line.split()
        if not tokens:
            continue
        if tokens[0].endswith(":"):
            try:
                addr = int(tokens[0][:-1], 16)
            except ValueError as e:
                raise ProbeError(f"line {n}: bad address column {tokens[0]!r}") from e
            if start is not None and addr != start + 4 * len(words):
                raise ProbeError(f"line {n}: address 0x{addr:08x}, expected 0x{start + 4 * len(words):08x}")
            tokens = tokens[1:]
        for t in tokens:
            m = _HEX_WORD.fullmatch(t)
            if not m:
                raise ProbeError(f"line {n}: not a 32-bit hex word: {t!r}")
            words.append(int(m.group(1), 16))
    return words


def parse_version(text: str) -> Optional[str]:
    """'0.32.0' from `probe-rs --version` output ("probe-rs 0.32.0 (git commit: ...)")"""
    m = re.search(r"probe-rs\S*\s+v?(\d+\.\d+\.\d+)", text)
    return m.group(1) if m else None


def parse_nm(text: str, name: str) -> Tuple[int, Optional[int]]:
    """(address, size) of `name` in `nm -S -C` output; size is None when
    nm printed none. Raises ProbeError if the symbol is absent or defined
    more than once."""
    hits = []
    for line in text.splitlines():
        parts = line.split(None, 3)
        if len(parts) == 4 and parts[3] == name and len(parts[2]) == 1:
            hits.append((int(parts[0], 16), int(parts[1], 16)))
        elif len(parts) >= 3 and " ".join(parts[2:]) == name and len(parts[1]) == 1:
            hits.append((int(parts[0], 16), None))
    if not hits:
        raise ProbeError(f"symbol {name} not in the image (nm)")
    if len(set(hits)) > 1:
        raise ProbeError(f"symbol {name} is defined {len(hits)} times")
    return hits[0]


Runner = Callable[[Sequence[str]], str]


def run(cmd: Sequence[str]) -> str:
    """Run a command; its stdout, or ProbeError with its stderr"""
    try:
        p = subprocess.run(list(cmd), capture_output=True, text=True, check=False)
    except FileNotFoundError as e:
        raise ProbeError(f"{cmd[0]} not found: install it or pass its path") from e
    if p.returncode != 0:
        raise ProbeError(f"{' '.join(cmd)} failed ({p.returncode}): {p.stderr.strip()}")
    return p.stdout


def symbol(elf: str, name: str, nm: Optional[str] = None,
           runner: Optional[Runner] = None) -> Tuple[int, Optional[int]]:
    """(address, size) of a symbol of the image, demangled name"""
    runner = runner or run
    tools = [nm] if nm else [t for t in NM_TOOLS if shutil.which(t)]
    if not tools:
        raise ProbeError("no nm found (arm-none-eabi-nm, llvm-nm or rust-nm); pass --nm or --address")
    return parse_nm(runner([tools[0], "-S", "-C", "--defined-only", elf]), name)


@dataclass
class Probe:
    """The probe-rs command line for one board"""

    probe_rs: str = "probe-rs"
    chip: str = CHIP
    probe: Optional[str] = None      # --probe VID:PID[:SERIAL], when several are attached
    protocol: Optional[str] = None   # --protocol swd|jtag
    speed: Optional[int] = None      # --speed kHz
    runner: Optional[Runner] = None  # default: run()
    log: List[List[str]] = field(default_factory=list)

    def _cmd(self, sub: str, *args: str) -> List[str]:
        cmd = [self.probe_rs, sub, "--chip", self.chip]
        if self.probe:
            cmd += ["--probe", self.probe]
        if self.protocol:
            cmd += ["--protocol", self.protocol]
        if self.speed:
            cmd += ["--speed", str(self.speed)]
        return cmd + list(args)

    def _run(self, cmd: List[str]) -> str:
        self.log.append(cmd)
        return (self.runner or run)(cmd)

    def version(self) -> Optional[str]:
        return parse_version(self._run([self.probe_rs, "--version"]))

    def read32_text(self, address: int, count: int) -> str:
        """The output of `probe-rs read b32` for `count` words from `address`"""
        return self._run(self._cmd("read", "b32", f"0x{address:08x}", str(count)))

    def read32(self, address: int, count: int) -> List[int]:
        """`count` words from `address` (does not halt the core)"""
        words = parse_words(self.read32_text(address, count), address)
        if len(words) != count:
            raise ProbeError(f"read {len(words)} words at 0x{address:08x}, asked for {count}")
        return words

    def write32(self, address: int, *values: int) -> None:
        """Write words from `address` (does not halt or resume the core)"""
        self._run(self._cmd("write", "b32", f"0x{address:08x}", *(f"0x{v:08x}" for v in values)))


def add_probe_args(ap) -> None:
    """The probe options every script takes"""
    g = ap.add_argument_group("probe")
    g.add_argument("--probe-rs", default="probe-rs", help="the probe-rs program (default: probe-rs)")
    g.add_argument("--chip", default=CHIP, help=f"probe-rs chip name (default {CHIP})")
    g.add_argument("--probe", help="probe selector VID:PID[:SERIAL], when several probes are attached")
    g.add_argument("--protocol", choices=("swd", "jtag"), help="debug protocol (probe-rs default: swd)")
    g.add_argument("--speed", type=int, help="probe speed in kHz")


def probe_from_args(args) -> Probe:
    return Probe(probe_rs=args.probe_rs, chip=args.chip, probe=args.probe, protocol=args.protocol,
                 speed=args.speed)


def report_version(probe: Probe) -> None:
    """Print the probe-rs version, with a note if it is not the tested one"""
    try:
        v = probe.version()
    except ProbeError as e:
        print(f"probe-rs --version failed: {e}")
        return
    note = "" if v == PROBE_RS_TESTED else f" (the scripts were written against {PROBE_RS_TESTED})"
    print(f"probe-rs {v or 'version unknown'}{note}")
