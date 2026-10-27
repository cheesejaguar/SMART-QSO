"""
Tests for platform_config.py: the 6.5 W VDD_IN cap, the 7 W nvpmodel
default, the board configuration without the carrier EEPROM and the PCIe C4
Gen2 x1 overlay (RID JETB-01; SQSO-JET-BRD-001 §13 item F4).

The module's sysfs and the Jetson Linux tree are stand-ins built in a
temporary directory.
"""

import os
import subprocess

import pytest

import platform_config as pc


def _hwmon(root, name, label, curr="0"):
    d = os.path.join(root, "sys/bus/i2c/drivers/ina3221", name, "hwmon", "hwmon3")
    os.makedirs(d)
    with open(os.path.join(d, "in1_label"), "w") as f:
        f.write(label + "\n")
    with open(os.path.join(d, "curr1_max"), "w") as f:
        f.write(curr + "\n")
    return d


def test_constants_are_the_carrier_design():
    assert pc.VDD_IN_LIMIT_MA == 1300  # 6.5 W at 5.0 V
    assert pc.NVPMODEL_DEFAULT_ID == 1  # 7 W, original configuration
    assert pc.BASE_BOARD_CONF == "jetson-orin-nano-devkit.conf"  # not -super


def test_power_cap_written_and_read_back(tmp_path):
    root = str(tmp_path)
    _hwmon(root, "1-0041", "VDD_OTHER")
    d = _hwmon(root, "1-0040", "VDD_IN", "5600")
    cap = pc.apply_power_cap(root)
    assert cap.hwmon == d
    assert cap.read_ma == 1300
    with open(os.path.join(d, "curr1_max")) as f:
        assert f.read().strip() == "1300"


def test_power_cap_fails_without_vdd_in(tmp_path):
    _hwmon(str(tmp_path), "1-0040", "VDD_CPU_GPU_CV")
    with pytest.raises(pc.ConfigError, match="VDD_IN"):
        pc.apply_power_cap(str(tmp_path))


def test_power_cap_mismatch_is_an_error(tmp_path, monkeypatch):
    _hwmon(str(tmp_path), "1-0040", "VDD_IN")
    real_open = open

    def clamped_open(path, mode="r", **kw):
        f = real_open(path, mode, **kw)
        if path.endswith("curr1_max") and mode == "r":
            f.close()
            from io import StringIO
            return StringIO("1000\n")  # the driver clamped the value
        return f

    monkeypatch.setattr("builtins.open", clamped_open)
    with pytest.raises(pc.ConfigError, match="1000 mA"):
        pc.apply_power_cap(str(tmp_path))


NVPMODEL_CONF = """< POWER_MODEL ID=0 NAME=15W >
CPU_ONLINE CORE_0 1
< POWER_MODEL ID=1 NAME=7W >
CPU_ONLINE CORE_0 1
< PM_CONFIG DEFAULT=0 >
"""


def test_nvpmodel_default_set_once(tmp_path):
    conf = tmp_path / "nvpmodel.conf"
    conf.write_text(NVPMODEL_CONF)
    assert pc.set_nvpmodel_default(str(conf))
    assert "< PM_CONFIG DEFAULT=1 >" in conf.read_text()
    assert not pc.set_nvpmodel_default(str(conf))  # already


def test_nvpmodel_without_the_7w_mode_is_refused(tmp_path):
    conf = tmp_path / "nvpmodel.conf"
    conf.write_text("< POWER_MODEL ID=0 NAME=15W >\n< PM_CONFIG DEFAULT=0 >\n")  # a -super file
    with pytest.raises(pc.ConfigError, match="ID=1"):
        pc.set_nvpmodel_default(str(conf))
    conf.write_text("< POWER_MODEL ID=1 NAME=7W >\n")
    with pytest.raises(pc.ConfigError, match="PM_CONFIG"):
        pc.set_nvpmodel_default(str(conf))


class _Nvpmodel:
    """nvpmodel -q / -m stand-in"""

    def __init__(self, mode, sticks=True):
        self.mode, self.sticks, self.calls = mode, sticks, []

    def __call__(self, args, **kw):
        self.calls.append(args)
        if args[1] == "-m" and self.sticks:
            self.mode = int(args[2])
        if args[1] == "-q":
            if self.mode is None:
                raise subprocess.CalledProcessError(1, args)
            return subprocess.CompletedProcess(args, 0, stdout=f"NV Power Mode: 7W\n{self.mode}\n")
        return subprocess.CompletedProcess(args, 0, stdout="")


def test_boot_sets_the_cap_and_the_mode(tmp_path):
    _hwmon(str(tmp_path), "1-0040", "VDD_IN")
    run = _Nvpmodel(0)
    pc.boot(str(tmp_path), run)
    assert ["nvpmodel", "-m", "1"] in run.calls
    run = _Nvpmodel(1)
    pc.boot(str(tmp_path), run)
    assert ["nvpmodel", "-m", "1"] not in run.calls  # already in force
    with pytest.raises(pc.ConfigError, match="mode"):
        pc.boot(str(tmp_path), _Nvpmodel(0, sticks=False))
    assert pc.nvpmodel_mode(_Nvpmodel(None)) is None


def _l4t(tmp_path, bct_lines=1):
    l4t = tmp_path / "Linux_for_Tegra"
    bct = l4t / "bootloader" / "generic" / "BCT"
    bct.mkdir(parents=True)
    (l4t / pc.BASE_BOARD_CONF).write_text('source "${LDK_DIR}/p3768-0000-p3767-0000-a0.conf";\n')
    (bct / pc.MISC_BCT).write_text(
        "/ { misc { eeprom {\n" + "        cvb_eeprom_read_size = <0x100>;\n" * bct_lines + "    }; }; };\n")
    return l4t


def test_board_config_without_the_carrier_eeprom(tmp_path):
    l4t = _l4t(tmp_path)
    written = pc.make_board_config(str(l4t), compile_overlay=False)
    assert len(written) == 3
    bct = (l4t / pc.MISC_BCT_DIR / pc.MISC_BCT_SQSO).read_text()
    assert "cvb_eeprom_read_size = <0x0>;" in bct
    assert "0x100" not in bct
    conf = (l4t / pc.BOARD_CONF).read_text()
    assert 'source "${LDK_DIR}/jetson-orin-nano-devkit.conf";' in conf
    assert f'MISC_CONFIG="{pc.MISC_BCT_SQSO}";' in conf
    assert pc.OVERLAY_DTBO in conf
    assert (l4t / "kernel" / "dtb" / os.path.basename(pc.OVERLAY_DTS)).is_file()


def test_board_config_refuses_an_unexpected_tree(tmp_path):
    with pytest.raises(pc.ConfigError, match="cvb_eeprom_read_size"):
        pc.make_board_config(str(_l4t(tmp_path / "a", bct_lines=2)), compile_overlay=False)
    l4t = _l4t(tmp_path / "b")
    (l4t / pc.BASE_BOARD_CONF).unlink()
    with pytest.raises(pc.ConfigError, match="original devkit"):
        pc.make_board_config(str(l4t), compile_overlay=False)


def test_overlay_limits_pcie_c4_to_gen2_x1():
    with open(pc.OVERLAY_DTS) as f:
        dts = f.read()
    assert "/plugin/;" in dts
    assert 'target-path = "/bus@0/pcie@14160000";' in dts
    assert "max-link-speed = <0x2>;" in dts
    assert "num-lanes = <0x1>;" in dts


def test_cli_usage():
    assert pc.main(["platform_config.py"]) == 2
