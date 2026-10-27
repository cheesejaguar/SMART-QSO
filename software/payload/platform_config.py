"""
Jetson platform configuration for the SMART-QSO payload (RID JETB-01).

The module is capped and configured as SQSO-JET-BRD-001 §13 item F4 and
TB-13 require:

- **6.5 W cap.** At every boot, before the agent runs, the VDD_IN average
  limit of the module's INA3221 (channel 1, VDD_IN; ``curr1_max``, mA) is set
  to ``VDD_IN_LIMIT_MA`` = 1300 mA (6.5 W at the module's 5.0 V VDD_IN). The
  module throttles its clocks in hardware to hold the average under it. The
  value is read back; a mismatch is an error (the boot service fails and the
  OBC sees no READY).
- **7 W mode.** The original (not ``-super``) flash configuration's
  nvpmodel ID 1 (7 W) is the default (``< PM_CONFIG DEFAULT=1 >`` in
  ``/etc/nvpmodel.conf``, set in the image) and is checked at boot.
- **PCIe C4 at Gen2 x1** (the NVMe, SQSO-JET-BRD-001 §10): the device-tree
  overlay ``config/tegra234-p3767-sqso-pcie-c4.dts``.
- **No carrier EEPROM**: the board configuration ``smart-qso-carrier.conf``
  is generated in the Jetson Linux tree from ``jetson-orin-nano-devkit.conf``
  with an MB2 misc BCT whose ``cvb_eeprom_read_size`` is 0, and the overlay
  added to ``OVERLAY_DTB_FILE``.

Usage:
    python3 platform_config.py boot              # on the module, at boot
    python3 platform_config.py image L4T_DIR     # on the build host

``boot`` is run by ``sqso-platform.service`` (``config/sqso-platform.service.j2``,
installed by ``ansible/deploy_payload.yml``; ``run_payload.sh`` starts the agent
only once it is active); ``image`` by the image build.
"""

from __future__ import annotations

import glob
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from typing import List, Optional

#: VDD_IN average limit, mA (6.5 W at 5.0 V; SQSO-JET-BRD-001 §13, JETB-01)
VDD_IN_LIMIT_MA = 1300
#: The default nvpmodel mode: 7 W in the original flash configuration (TB-13)
NVPMODEL_DEFAULT_ID = 1
#: The INA3221's channel label for the module input
VDD_IN_LABEL = "VDD_IN"
#: Where the INA3221 driver exposes its channels (relative to the root)
INA3221_HWMON_GLOB = "sys/bus/i2c/drivers/ina3221/*/hwmon/hwmon*"

#: The board configuration this tool derives from (original, not -super)
BASE_BOARD_CONF = "jetson-orin-nano-devkit.conf"
#: The board configuration it writes
BOARD_CONF = "smart-qso-carrier.conf"
#: The MB2 misc BCT of the P3767 modules and the copy without the EEPROM read
MISC_BCT_DIR = os.path.join("bootloader", "generic", "BCT")
MISC_BCT = "tegra234-mb2-bct-misc-p3767-0000.dts"
MISC_BCT_SQSO = "tegra234-mb2-bct-misc-p3767-sqso.dts"
#: The PCIe overlay: its source here and its compiled name in the tree
OVERLAY_DTS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "config",
                           "tegra234-p3767-sqso-pcie-c4.dts")
OVERLAY_DTBO = "tegra234-p3767-sqso-pcie-c4.dtbo"

_EEPROM_SIZE = re.compile(r"cvb_eeprom_read_size\s*=\s*<\s*0x[0-9a-fA-F]+\s*>\s*;")
_PM_DEFAULT = re.compile(r"<\s*PM_CONFIG\s+DEFAULT=(\d+)\s*>")


class ConfigError(RuntimeError):
    """A configuration step that did not take effect"""


@dataclass
class CapResult:
    """The power cap as written and read back"""
    hwmon: str
    written_ma: int
    read_ma: int


def find_vdd_in_hwmon(root: str = "/") -> str:
    """The INA3221 hwmon directory whose channel 1 is VDD_IN"""
    for path in sorted(glob.glob(os.path.join(root, INA3221_HWMON_GLOB))):
        try:
            with open(os.path.join(path, "in1_label"), encoding="ascii") as f:
                if f.read().strip() == VDD_IN_LABEL:
                    return path
        except OSError:
            continue
    raise ConfigError("no INA3221 channel labelled VDD_IN")


def apply_power_cap(root: str = "/", limit_ma: int = VDD_IN_LIMIT_MA) -> CapResult:
    """Write the VDD_IN average limit (``curr1_max``) and read it back"""
    hwmon = find_vdd_in_hwmon(root)
    path = os.path.join(hwmon, "curr1_max")
    with open(path, "w", encoding="ascii") as f:
        f.write(f"{limit_ma}\n")
    with open(path, encoding="ascii") as f:
        read = int(f.read().strip())
    if read != limit_ma:
        raise ConfigError(f"curr1_max reads {read} mA, {limit_ma} mA written")
    return CapResult(hwmon=hwmon, written_ma=limit_ma, read_ma=read)


def set_nvpmodel_default(conf_path: str, mode_id: int = NVPMODEL_DEFAULT_ID) -> bool:
    """Make ``mode_id`` the default in an nvpmodel.conf; True if changed"""
    with open(conf_path, encoding="utf-8") as f:
        text = f.read()
    if len(_PM_DEFAULT.findall(text)) != 1:
        raise ConfigError(f"{conf_path}: not exactly one PM_CONFIG line")
    if not re.search(rf"<\s*POWER_MODEL\s+ID={mode_id}\b", text):
        raise ConfigError(f"{conf_path}: no POWER_MODEL ID={mode_id} (the original, not -super, configuration?)")
    new = _PM_DEFAULT.sub(f"< PM_CONFIG DEFAULT={mode_id} >", text)
    if new != text:
        with open(conf_path, "w", encoding="utf-8") as f:
            f.write(new)
    return new != text


def nvpmodel_mode(run=subprocess.run) -> Optional[int]:
    """The mode in force (``nvpmodel -q``), None if it cannot be read"""
    try:
        out = run(["nvpmodel", "-q"], capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    lines = [ln.strip() for ln in out.splitlines() if ln.strip()]
    return int(lines[-1]) if lines and lines[-1].isdigit() else None


def boot(root: str = "/", run=subprocess.run) -> CapResult:
    """At boot: the cap, then the 7 W mode (set again if another is in force)"""
    cap = apply_power_cap(root)
    if nvpmodel_mode(run) != NVPMODEL_DEFAULT_ID:
        run(["nvpmodel", "-m", str(NVPMODEL_DEFAULT_ID)], check=True)
        if nvpmodel_mode(run) != NVPMODEL_DEFAULT_ID:
            raise ConfigError(f"nvpmodel mode is not {NVPMODEL_DEFAULT_ID}")
    return cap


def board_conf_text() -> str:
    """The board configuration: the original devkit's, without the carrier
    EEPROM read, with the PCIe overlay"""
    return (
        "# SMART-QSO carrier (SQSO-JET-BRD-001), generated by software/payload/platform_config.py\n"
        "# The original Orin Nano developer-kit configuration (not -super: nvpmodel ID 1, 7 W, TB-13);\n"
        "# no carrier EEPROM (MB2 misc BCT cvb_eeprom_read_size = 0); PCIe C4 at Gen2 x1 (overlay).\n"
        f'source "${{LDK_DIR}}/{BASE_BOARD_CONF}";\n'
        f'MISC_CONFIG="{MISC_BCT_SQSO}";\n'
        f'OVERLAY_DTB_FILE="${{OVERLAY_DTB_FILE}},{OVERLAY_DTBO}";\n'
    )


def make_board_config(l4t_dir: str, compile_overlay: bool = True) -> List[str]:
    """Write the board configuration, the EEPROM-less misc BCT and the
    overlay into a Jetson Linux tree; the files written"""
    base = os.path.join(l4t_dir, BASE_BOARD_CONF)
    if not os.path.isfile(base):
        raise ConfigError(f"{base} missing (Jetson Linux tree without the original devkit configuration)")
    src = os.path.join(l4t_dir, MISC_BCT_DIR, MISC_BCT)
    with open(src, encoding="utf-8") as f:
        bct = f.read()
    new, n = _EEPROM_SIZE.subn("cvb_eeprom_read_size = <0x0>;", bct)
    if n != 1:
        raise ConfigError(f"{src}: {n} cvb_eeprom_read_size lines, expected 1")
    written = []
    out_bct = os.path.join(l4t_dir, MISC_BCT_DIR, MISC_BCT_SQSO)
    with open(out_bct, "w", encoding="utf-8") as f:
        f.write(new)
    written.append(out_bct)
    conf = os.path.join(l4t_dir, BOARD_CONF)
    with open(conf, "w", encoding="utf-8") as f:
        f.write(board_conf_text())
    written.append(conf)
    dtb_dir = os.path.join(l4t_dir, "kernel", "dtb")
    os.makedirs(dtb_dir, exist_ok=True)
    if compile_overlay:
        dtbo = os.path.join(dtb_dir, OVERLAY_DTBO)
        subprocess.run(["dtc", "-@", "-I", "dts", "-O", "dtb", "-o", dtbo, OVERLAY_DTS], check=True)
        written.append(dtbo)
    else:
        dts = os.path.join(dtb_dir, os.path.basename(OVERLAY_DTS))
        shutil.copyfile(OVERLAY_DTS, dts)
        written.append(dts)
    return written


def main(argv: List[str]) -> int:
    if len(argv) >= 2 and argv[1] == "boot":
        cap = boot()
        print(f"[PLATFORM] VDD_IN limit {cap.read_ma} mA ({cap.hwmon}); nvpmodel {NVPMODEL_DEFAULT_ID}")
        return 0
    if len(argv) == 3 and argv[1] == "image":
        for path in make_board_config(argv[2]):
            print(f"[PLATFORM] wrote {path}")
        return 0
    print(__doc__.split("Usage:")[1].strip(), file=sys.stderr)
    return 2


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except ConfigError as e:
        print(f"[PLATFORM] error: {e}", file=sys.stderr)
        sys.exit(1)
