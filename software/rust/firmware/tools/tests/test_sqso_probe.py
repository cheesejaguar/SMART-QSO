"""sqso_probe: probe-rs output, nm output and the command line (no hardware)"""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import sqso_probe as sp  # noqa: E402


def test_hex_table_of_probe_rs_0_32():
    text = ("2003e000: 57acc0de 57acc0de 57acc0de 57acc0de 57acc0de 57acc0de 57acc0de 57acc0de\n"
            "2003e020: 00000001 deadbeef\n")
    assert sp.parse_words(text, 0x2003E000) == [0x57ACC0DE] * 8 + [1, 0xDEADBEEF]


def test_plain_list_and_0x_prefix():
    assert sp.parse_words("57acc0de 0x00000010\n  ffffffff \n\n") == [0x57ACC0DE, 0x10, 0xFFFFFFFF]


def test_address_column_must_follow_on():
    text = "2003e000: 00000000 00000000\n2003e020: 00000000\n"   # second line should be at 2003e008
    with pytest.raises(sp.ProbeError, match="expected 0x2003e008"):
        sp.parse_words(text, 0x2003E000)


def test_wrong_region_is_refused():
    with pytest.raises(sp.ProbeError, match="address 0x20000000"):
        sp.parse_words("20000000: 00000000\n", 0x2003E000)


def test_non_hex_output_is_refused():
    with pytest.raises(sp.ProbeError, match="not a 32-bit hex word"):
        sp.parse_words("WARN probe connection slow\n")
    with pytest.raises(sp.ProbeError, match="bad address column"):
        sp.parse_words("Error: probe not found\n")


def test_version():
    assert sp.parse_version("probe-rs 0.32.0 (git commit: 1a2b3c)\n") == "0.32.0"
    assert sp.parse_version("probe-rs-tools v0.24.0\n") == "0.24.0"
    assert sp.parse_version("something else") is None


NM = """\
08000000 00000400 R __RESET_VECTOR
200003f0 00000004 d sqso_firmware::wdt::RESET_CAUSE
20000000 00000018 D SQSO_LOOP_TIMING
20040000 A _stack_start
08002f16 00000052 T <sqso_ffi::c::HalFlash as sqso_fsw::nvm::Flash>::read
"""


def test_nm_symbols():
    assert sp.parse_nm(NM, "SQSO_LOOP_TIMING") == (0x20000000, 0x18)
    assert sp.parse_nm(NM, "sqso_firmware::wdt::RESET_CAUSE") == (0x200003F0, 4)
    assert sp.parse_nm(NM, "_stack_start") == (0x20040000, None)
    assert sp.parse_nm(NM, "<sqso_ffi::c::HalFlash as sqso_fsw::nvm::Flash>::read") == (0x08002F16, 0x52)


def test_nm_missing_or_duplicate_symbol():
    with pytest.raises(sp.ProbeError, match="not in the image"):
        sp.parse_nm(NM, "SQSO_LOOP")
    with pytest.raises(sp.ProbeError, match="defined 2 times"):
        sp.parse_nm(NM + "20000100 00000018 D SQSO_LOOP_TIMING\n", "SQSO_LOOP_TIMING")


def test_symbol_runs_nm_with_size_and_demangling():
    calls = []

    def runner(cmd):
        calls.append(cmd)
        return NM

    assert sp.symbol("fw.elf", "SQSO_LOOP_TIMING", nm="llvm-nm", runner=runner) == (0x20000000, 0x18)
    assert calls == [["llvm-nm", "-S", "-C", "--defined-only", "fw.elf"]]


def test_probe_command_lines():
    calls = []

    def runner(cmd):
        calls.append(cmd)
        return "20000000: 00000001 00000002\n" if cmd[1] == "read" else ""

    p = sp.Probe(chip="STM32L4R5ZITxP", probe="0483:374b", protocol="swd", runner=runner)
    assert p.read32(0x20000000, 2) == [1, 2]
    p.write32(0xE000EDF0, 0xA05F0003)
    assert calls == [
        ["probe-rs", "read", "--chip", "STM32L4R5ZITxP", "--probe", "0483:374b", "--protocol", "swd",
         "b32", "0x20000000", "2"],
        ["probe-rs", "write", "--chip", "STM32L4R5ZITxP", "--probe", "0483:374b", "--protocol", "swd",
         "b32", "0xe000edf0", "0xa05f0003"],
    ]


def test_short_read_is_an_error():
    p = sp.Probe(runner=lambda cmd: "20000000: 00000001\n")
    with pytest.raises(sp.ProbeError, match="read 1 words"):
        p.read32(0x20000000, 6)
