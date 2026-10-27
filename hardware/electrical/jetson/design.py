#!/usr/bin/env python3
"""SMART-QSO Jetson Orin Nano carrier board, schematic captured as code (SKiDL 3.0). Document: SQSO-JET-BRD-001
(CARRIER_BOARD_DESIGN.md). Values are calculated in jetson_calcs.py.

Run with the CAD venv (SKiDL 3.0; no KiCad libraries needed: every symbol is defined here from its data-sheet pin
table):

    <venv>/bin/python hardware/electrical/jetson/design.py

Writes, next to this file: jetson_carrier.net (KiCad 8 netlist), jetson_carrier_bom.csv, jetson_carrier_erc.log,
jetson_carrier_nets.csv (net -> pins, for review without KiCad).

Power tree (CARRIER_BOARD_DESIGN.md section 4):
  VBAT_IN (J1, 11.2-16.4 V behind the EPS eFuse U19) -> D1 TVS -> L1/C filter -> VBAT_F
  VBAT_F -> U1 LM61440 -> 5V_BUCK (5.02 V) -> U2 TPS259474L (OVLO 5.38 V, 2.5 A circuit breaker, latch-off) -> VDD_IN
  VBAT_F -> U3 LMR43620 -> 3V3_PRE -> U7 TPS22918 (ON = SYS_RESET*) -> 3V3_IO -> U9 -> 1V8_IO
                                    -> U8 TPS22918 (ON = SYS_RESET*) -> 3V3_SSD (M.2 NVMe)
  POWER_EN = wired AND of U2 PG (VDD_IN good), U4 RESET (VBAT_F good for 19 ms, and not shutdown-latched).
  JET_3V3 (W5B pin 4) = 3V3_IO: high only when the module has completed its power sequence.
"""

import builtins
import csv
import io
import logging
import os
import re
from collections import OrderedDict, defaultdict

from skidl import ERC, KICAD8, POWER, TEMPLATE, Net, Part, Pin, SKIDL, generate_netlist, set_default_tool
import skidl

set_default_tool(SKIDL)
HERE = os.path.dirname(os.path.abspath(__file__))
T = Pin.types
TY = dict(P=T.PASSIVE, I=T.INPUT, O=T.OUTPUT, B=T.BIDIR, W=T.PWRIN, WO=T.PWROUT, OC=T.OPENCOLL, N=T.NOCONNECT)

# ------------------------------------------------------------------------------------------------- helpers
_nets = {}


def N(name):
    if name not in _nets:
        _nets[name] = Net(name)
    return _nets[name]


POWER_NETS = ["VBAT_IN", "VBAT_F", "GND", "5V_BUCK", "VDD_IN", "3V3_PRE", "3V3_IO", "3V3_SSD", "1V8_IO", "JET_3V3",
              "DAMP"]


def sym(name, prefix, pins):
    plist = []
    for tok in pins.split():
        num, pname, ty = tok.split(":")
        plist.append(Pin(num=num, name=pname, func=TY[ty]))
    return Part(tool=SKIDL, name=name, ref_prefix=prefix, dest=TEMPLATE, pins=plist)


BOMINFO = {}
AUTOREF = set()
EXTRA_BOM = []  # items with no pins (installed modules, mechanical, TIM)


def place(tm, ref, value, mpn, mfr, pkg, rating, stress="", notes="", finish="", checked=""):
    p = tm(value=value)
    if ref:
        p.ref = ref
    else:
        AUTOREF.add(id(p))
        p.ref = f"{p.ref_prefix}TMP{len(AUTOREF)}"
    p.footprint = pkg
    p.fields["MPN"] = mpn
    p.fields["Manufacturer"] = mfr
    BOMINFO[id(p)] = dict(value=value, mpn=mpn, mfr=mfr, pkg=pkg, rating=rating, stress=stress, notes=notes,
                          finish=finish, checked=checked)
    return p


def link(name, pins):
    N(name).__iadd__(pins)


def conn(p, mapping):
    for pin, net in mapping.items():
        if net is None:
            p[pin] += builtins.NC
        else:
            link(net, p[pin])


TR = sym("R", "R", "1:A:P 2:B:P")
TC = sym("C", "C", "1:A:P 2:B:P")
TL = sym("L", "L", "1:A:P 2:B:P")
TNTC = sym("NTC", "RT", "1:A:P 2:B:P")
TTP = sym("TP", "TP", "1:TP:P")
TMH = sym("MH", "H", "1:MH:P")


def rcode(v):
    """Panasonic value code: 1R00, 33R0, 1002 (10.0 k), 6983 (698 k)."""
    if v < 10:
        return f"{v:.2f}".replace(".", "R")[:4]
    if v < 100:
        return f"{v:.1f}".replace(".", "R")
    exp = 0
    while v >= 1000:
        v /= 10.0
        exp += 1
    while v < 100:
        v *= 10.0
        exp -= 1
    return f"{int(round(v)):03d}{exp}"


def fmt(v):
    for th, suf in ((1e6, "M"), (1e3, "k")):
        if v >= th:
            return f"{v / th:.3g}{suf}"
    return f"{v:.3g}"


def R(v, a, b, size="0603", tol=1.0, note="", stress="", ref=None):
    """Panasonic ERJ-3EKF (1 %, AEC-Q200), ERA-3AEB (0.1 %, 25 ppm/K, AEC-Q200), ERJ-6ENF 0805, ERJ-3GEY0R00 jumper."""
    if v == 0:
        mpn, rating, val = "ERJ-3GEY0R00V", "0 Ohm jumper, 1 A, AEC-Q200", "0"
    elif size == "0603" and tol <= 0.1:
        mpn, rating, val = f"ERA-3AEB{rcode(v)}V", "0.1 %, 25 ppm/K, 0.1 W, 75 V, AEC-Q200", fmt(v)
    elif size == "0603":
        mpn = f"ERJ-3EKF{rcode(v)}V" if v >= 10 else f"ERJ-3RQF{rcode(v)}V"
        rating, val = "1 %, 100 ppm/K, 0.1 W, 75 V, AEC-Q200", fmt(v)
    elif size == "0805":
        mpn = f"ERJ-6ENF{rcode(v)}V" if v >= 10 else f"ERJ-6RQF{rcode(v)}V"
        rating, val = "1 %, 0.125 W, 150 V, AEC-Q200", fmt(v)
    else:
        raise ValueError(size)
    p = place(TR, ref, val, mpn, "Panasonic", f"Resistor_SMD:R_{size}", rating, stress or "< 10 % power", note,
              "Sn over Ni (matte): re-tin or coat (EEE_PARTS_LIST 2A.3)", "series code")
    link(a, p["A"])
    link(b, p["B"])
    return p


CAPS = {  # (value, voltage, size) -> (mpn, dielectric)  Murata GCM automotive series (AEC-Q200)
    ("100n", 50, "0603"): ("GCM188R71H104KA57D", "X7R"),
    ("10n", 50, "0603"): ("GCM188R71H103KA37D", "X7R"),
    ("1n", 50, "0603"): ("GCM1885C1H102JA16D", "C0G"),
    ("2.2n", 50, "0603"): ("GCM1885C1H222JA16D", "C0G"),
    ("3.3n", 50, "0603"): ("GCM1885C1H332JA16D", "C0G"),
    ("220n", 25, "0603"): ("GCM188R71E224KA55D", "X7R"),
    ("220n", 16, "0402"): ("GCM155R71C224KE02D", "X7R"),
    ("1u", 25, "0603"): ("GCM188R71E105KA64D", "X7R"),
    ("10u", 25, "1206"): ("GCM31CR71E106KA64L", "X7R"),
    ("4.7u", 50, "1210"): ("GCM32ER71H475KA55L", "X7R"),
    ("22u", 16, "1210"): ("GCM32ER71C226KE19L", "X7R"),
}


def C(val, a, b, v=50, size="0603", vop=None, note="", ref=None):
    mpn, die = CAPS[(val, v, size)]
    st = f"{vop:.2f} V / {v} V = {vop / v:.0%} of rated V" if vop else "<= 50 % of rated V"
    p = place(TC, ref, val, mpn, "Murata", f"Capacitor_SMD:C_{size}", f"{v} V {die} AEC-Q200", st, note,
              "Sn over Ni (matte): re-tin or coat", "series code (GCM, used on the EPS/OBC boards)" if (val, v, size) in
              (("100n", 50, "0603"), ("1u", 25, "0603"), ("4.7u", 50, "1210"), ("22u", 16, "1210"), ("10u", 25, "1206"),
               ("10n", 50, "0603"), ("1n", 50, "0603")) else "series code (not checked)")
    link(a, p["A"])
    link(b, p["B"])
    return p


def TP(net, ref=None):
    p = place(TTP, ref, "TP", "5015", "Keystone", "TestPoint:Keystone_5015", "SMT test point", "", f"test point {net}",
              "Sn", "not checked")
    link(net, p["TP"])


def CONN(name, pins, prefix="J"):
    return sym(name, prefix, " ".join(f"{n}:{nm}:P" for n, nm in pins))


def ic(tm, ref, value, mpn, mfr, pkg, rating, mapping, stress="", notes="", finish="NiPdAu",
       checked="TI product page: orderable listed (2026-10-27)"):
    p = place(tm, ref, value, mpn, mfr, pkg, rating, stress, notes, finish, checked)
    conn(p, mapping)
    return p


# ------------------------------------------------------------------------------------------------- IC symbols
S_LM61440 = sym("LM61440-Q1", "U", "1:BIAS:W 2:VCC:WO 3:AGND:W 4:FB:I 5:PGOOD:OC 6:RT:P 7:EN_SYNC:I 8:VIN1:W "
                "9:PGND1:W 10:SW:P 11:PGND2:W 12:VIN2:W 13:RBOOT:P 14:CBOOT:P")             # SNVSBE4C Table 7-1
S_LMR436 = sym("LMR43620-Q1", "U", "1:SYNC:I 2:PGOOD:OC 3:EN:I 4:VIN:W 5:SW:P 6:BOOT:P 7:VCC:WO 8:FB:I 9:GND:W")
S_TPS25947 = sym("TPS259474L", "U", "1:EN_UVLO:I 2:OVLO:I 3:PG:OC 4:PGTH:I 5:IN:W 6:OUT:WO 7:DVDT:P 8:GND:W "
                 "9:ILM:P 10:ITIMER:P 11:PAD:P")                                                # SLVSFC9C Table 5-1
S_TPS3808 = sym("TPS3808G01-Q1", "U", "1:RESET:OC 2:GND:W 3:MR:I 4:CT:P 5:SENSE:I 6:VDD:W")    # DBV, SBVS085J
S_LVC74 = sym("SN74LVC1G74", "U", "1:CLK:I 2:D:I 3:QN:O 4:GND:W 5:Q:O 6:CLRN:I 7:PREN:I 8:VCC:W")  # DCU, SCES794G
S_LVC07 = sym("SN74LVC1G07-Q1", "U", "1:NC:N 2:A:I 3:GND:W 4:Y:OC 5:VCC:W")                  # DCK
S_LVC04 = sym("SN74LVC1G04-Q1", "U", "1:NC:N 2:A:I 3:GND:W 4:Y:O 5:VCC:W")                   # DCK
S_TPS22918 = sym("TPS22918-Q1", "U", "1:VIN:W 2:GND:W 3:ON:I 4:CT:P 5:QOD:P 6:VOUT:WO")      # DBV
S_TLV733 = sym("TLV73318P-Q1", "U", "1:IN:W 2:GND:W 3:EN:I 4:NC:N 5:OUT:WO")                 # DBV
S_LVC1T45 = sym("SN74LVC1T45-Q1", "U", "1:VCCA:W 2:GND:W 3:A:B 4:B:B 5:DIR:I 6:VCCB:W")       # DCK
S_XO = sym("SG-8018CA", "Y", "1:OE:I 2:GND:W 3:OUT:O 4:VCC:W")
S_TVS2200 = sym("TVS2200", "D", "1:GND1:P 2:GND2:P 3:GND3:P 4:IN4:P 5:IN5:P 6:IN6:P 7:PAD:P")
S_ESD4 = sym("TPD4E05U06-Q1", "D", "1:IO1:P 2:IO2:P 3:GND3:P 4:IO3:P 5:IO4:P 6:NC6:N 7:NC7:N 8:GND8:P 9:NC9:N 10:NC10:N")
S_ESD1 = sym("TPD1E05U06-Q1", "D", "1:IO:P 2:GND:P")


def esd4(ref, nets, note):
    p = place(S_ESD4, ref, "TPD4E05U06-Q1", "TPD4E05U06QDQARQ1", "TI", "USON-10 (DQA)",
              "4-ch ESD, 5.5 V VRWM, 0.5 pF, IEC 61000-4-2 12 kV, AEC-Q101", "<= 3.3 V on 5.5 V VRWM", note,
              "NiPdAu", "TI product page: orderable listed (2026-10-27); used on the OBC board")
    conn(p, {"IO1": nets[0], "IO2": nets[1], "IO3": nets[2], "IO4": nets[3], "GND3": "GND", "GND8": "GND",
             "NC6": None, "NC7": None, "NC9": None, "NC10": None})


# ------------------------------------------------------------------------------------------------- module pin map
# NVIDIA DS-11105-001 v1.5 section 4.4 pin list (Jetson Orin Nano function), parsed into module_pins.json.
import json  # noqa: E402

MODULE_PINS = {int(k): v for k, v in json.load(open(os.path.join(HERE, "module_pins.json"))).items()}
MODULE_USE = {  # pin -> carrier net; every other pin is left open (no camera, display, USB 1/2, USB 3.2, GbE, CAN, ...)
    237: "POWER_EN", 239: "SYS_RESET_N", 233: "SHUTDOWN_REQ_N", 214: "FORCE_RECOVERY_N", 217: "MODULE_ID",
    178: "MOD_SLEEP_N", 203: "UART1_TXD", 205: "UART1_RXD", 236: "UART2_TXD", 238: "UART2_RXD",
    109: "USB0_DN", 111: "USB0_DP", 134: "PCIE0_TX0_N", 136: "PCIE0_TX0_P", 131: "PCIE0_RX0_N", 133: "PCIE0_RX0_P",
    160: "PCIE0_CLK_N", 162: "PCIE0_CLK_P", 181: "PCIE0_RST_N", 180: "PCIE0_CLKREQ_N", 179: "PCIE_WAKE_N",
}


def pin_name(n):
    return re.sub(r"[^A-Za-z0-9_]", "_", MODULE_PINS[n].replace("*", "_N")) + (f"_{n}" if MODULE_PINS[n] in ("GND", "VDD_IN", "RSVD") else "")


# ================================================================================================= blocks
def input_and_filter():
    j = place(CONN("MicroFit_2x2", [(1, "P1"), (2, "P2"), (3, "P3"), (4, "P4")]), "J1", "VBAT IN", "0430450412", "Molex",
              "Micro-Fit 3.0 2x2 vertical", "5 A/contact", "1.30 A (EPS eFuse max) on 2 contacts: 13 %",
              "W5A from EPS J18: VBAT_JET x2, GND x2", "Sn", "Digi-Key: Active (EPS board J18 mate)")
    conn(j, {"P1": "VBAT_IN", "P2": "VBAT_IN", "P3": "GND", "P4": "GND"})
    ic(S_TVS2200, "D1", "TVS2200", "TVS2200DRVR", "TI", "WSON-6 (DRV)",
       "22 V VRWM, VBR 24.6 V min, 28 V clamp at 40 A (8/20 us), -40..125 C",
       {"IN4": "VBAT_IN", "IN5": "VBAT_IN", "IN6": "VBAT_IN", "GND1": "GND", "GND2": "GND", "GND3": "GND", "PAD": "GND"},
       stress="18.4 V (EPS OVP) / 22 V VRWM; clamp 28 V < 36 V converter rating",
       notes="connector surge/ESD and hot-plug at integration (GSE)", finish="NiPdAu")
    C("4.7u", "VBAT_IN", "GND", 50, "1210", vop=18.4, note="C_F1")
    C("100n", "VBAT_IN", "GND", note="C_F1 HF")
    p = place(TL, "L1", "4.7uH", "XGL4030-472MEC", "Coilcraft", "4.0 x 4.0 x 3.1 mm",
              "4.7 uH, Isat about 4 A, DCR about 30 mOhm, AEC-Q200, -40..125 C", "1.30 A max / 4 A",
              "input filter L_F (CE102, V-EMC-1)", "Sn over Ag: coat", "not checked (vendor site 403); XGL4030 family used on the EPS")
    conn(p, {"A": "VBAT_IN", "B": "VBAT_F"})
    R(1.0, "VBAT_F", "DAMP", size="0805", note="filter damper R_D", stress="10 mW / 125 mW")
    for _ in range(2):
        C("4.7u", "DAMP", "GND", 50, "1210", vop=18.4, note="filter damper C_D")
    TP("VBAT_F")


def buck_5v():
    ic(S_LM61440, "U1", "LM61440-Q1", "LM61440AANQRJRRQ1", "TI", "VQFN-HR-14 (RJR) 4 x 3.5",
       "3-36 V (42 V abs), 4 A, sync 0.2-2.2 MHz, no spread spectrum (AAN), AEC-Q100 grade 1",
       {"BIAS": "5V_BUCK", "VCC": "U1_VCC", "AGND": "GND", "FB": "U1_FB", "PGOOD": None, "RT": "U1_RT",
        "EN_SYNC": "U1_EN", "VIN1": "VBAT_F", "PGND1": "GND", "SW": "U1_SW", "PGND2": "GND", "VIN2": "VBAT_F",
        "RBOOT": "U1_RBOOT", "CBOOT": "U1_CBOOT"},
       stress="VIN 18.4 V / 36 V = 51 %; 1.4 A / 4 A", notes="5 V buck for VDD_IN, synchronised to Y1 (1.000 MHz)",
       finish="Sn (matte): QFN-HR, re-tin not possible: conformal coat")
    for _ in range(2):
        C("4.7u", "VBAT_F", "GND", 50, "1210", vop=18.4, note="U1 VIN1/VIN2 (C_F2)")
        C("100n", "VBAT_F", "GND", note="U1 VIN1/VIN2 HF, at the pins")
    R(698e3, "VBAT_F", "U1_EN", note="EN UVLO on 10.1 V, off 7.3 V")
    R(100e3, "U1_EN", "GND")
    C("1n", "SYNC_CLK", "U1_EN", note="C_SYNC (data sheet 9.3.2)")
    R(13.0e3, "U1_RT", "GND", tol=0.1, note="RT 1.000 MHz free-running (Eq. 2)")
    R(100e3, "5V_BUCK", "U1_FB", tol=0.1, note="RFBT")
    R(24.9e3, "U1_FB", "GND", tol=0.1, note="RFBB: 5.016 V")
    C("1u", "U1_VCC", "GND", 25, note="VCC LDO")
    C("100n", "U1_CBOOT", "U1_SW", note="CBOOT")
    R(100, "U1_RBOOT", "U1_CBOOT", note="RBOOT: SW rise 2.7 ns (EMI)")
    C("100n", "5V_BUCK", "GND", note="BIAS")
    p = place(TL, "L2", "3.3uH", "XGL6030-332MEC", "Coilcraft", "6.5 x 6.5 x 3.1 mm",
              "3.3 uH, Isat >= 8.1 A (U1 IL-HS max), DCR about 11 mOhm, AEC-Q200", "3.3 A peak / 8.1 A",
              "U1 inductor (data sheet 10.2.2: Isat >= IL-HS)", "Sn over Ag: coat", "not checked (vendor site 403)")
    conn(p, {"A": "U1_SW", "B": "5V_BUCK"})
    for _ in range(4):
        C("10u", "5V_BUCK", "GND", 25, "1206", vop=5.07, note="U1 output; 25 V so a U1 high-side short does not short the bus")
    TP("5V_BUCK")


def vdd_in_efuse():
    ic(S_TPS25947, "U2", "TPS259474L", "TPS259474LRPWR", "TI", "VQFN-HR-10 (RPW) 2 x 2",
       "2.7-23 V (28 V abs), 5.5 A, 28 mOhm, adj. OVLO, circuit breaker, PG, latch-off, -40..125 C",
       {"EN_UVLO": "U2_EN", "OVLO": "U2_OVLO", "PG": "POWER_EN", "PGTH": "U2_PGTH", "IN": "5V_BUCK", "OUT": "VDD_IN",
        "DVDT": "U2_DVDT", "GND": "GND", "ILM": "U2_ILM", "ITIMER": "U2_ITIMER", "PAD": "GND"},
       stress="5.1 V / 23 V = 22 % (18.4 V = 80 % only if U1 fails short); 1.4 A / 5.5 A",
       notes="VDD_IN protection: OVLO 5.38 V (module 5.5 V abs), 2.51 A breaker after 1.85 ms, PG -> POWER_EN",
       checked="TI product page and data sheet SLVSFC9C orderable addendum (2026-10-27); industrial, not AEC-Q100")
    R(28.7e3, "5V_BUCK", "U2_EN", tol=0.1, note="UVLO on 4.64 V")
    R(10.0e3, "U2_EN", "GND", tol=0.1)
    R(34.8e3, "5V_BUCK", "U2_OVLO", tol=0.1, note="OVLO 5.38 V (5.30-5.48)")
    R(10.0e3, "U2_OVLO", "GND", tol=0.1)
    R(28.7e3, "VDD_IN", "U2_PGTH", tol=0.1, note="PGTH: good above 4.64 V")
    R(10.0e3, "U2_PGTH", "GND", tol=0.1)
    C("2.2n", "U2_DVDT", "GND", note="dVdt 0.91 V/ms")
    R(1.33e3, "U2_ILM", "GND", tol=0.1, note="ILIM 2.51 A (RILM = 3334 / I)")
    C("2.2n", "U2_ITIMER", "GND", note="ITIMER 1.85 ms (1.13-3.19)")
    for _ in range(6):
        C("22u", "VDD_IN", "GND", 16, "1210", vop=5.21, note="VDD_IN bulk at the SO-DIMM power pins")
    for _ in range(2):
        C("100n", "VDD_IN", "GND", note="VDD_IN HF")
    TP("VDD_IN")
    TP("U2_ILM")


def rail_3v3():
    ic(S_LMR436, "U3", "LMR43620-Q1", "LMR43620MC3RPERQ1", "TI", "VQFN-HR-9 (RPE) 2 x 2.5",
       "3-36 V (42 V abs), 2 A, 3.3 V fixed, sync 0.2-2.5 MHz, no spread spectrum (MC), AEC-Q100",
       {"SYNC": "U3_SYNC", "PGOOD": None, "EN": "U3_EN", "VIN": "VBAT_F", "SW": "U3_SW", "BOOT": "U3_BOOT",
        "VCC": "U3_VCC", "FB": "3V3_PRE", "GND": "GND"},
       stress="VIN 18.4 V / 36 V = 51 %; 0.61 A / 2 A", notes="3V3_PRE: SSD, I/O rails, Y1, logic; synchronised (inverted clock)",
       finish="Sn (matte): QFN-HR, re-tin not possible: conformal coat",
       checked="TI product page: orderable listed (2026-10-27); same part as EPS U15")
    C("4.7u", "VBAT_F", "GND", 50, "1210", vop=18.4, note="U3 VIN (C_F2)")
    C("100n", "VBAT_F", "GND", note="U3 VIN HF")
    R(698e3, "VBAT_F", "U3_EN", note="EN on 9.8 V, off 7.0 V")
    R(100e3, "U3_EN", "GND")
    R(100, "SYNC_N", "U3_SYNC", note="MODE/SYNC")
    C("100n", "U3_BOOT", "U3_SW")
    C("1u", "U3_VCC", "GND", 25)
    p = place(TL, "L3", "6.8uH", "XGL4030-682MEC", "Coilcraft", "4.0 x 4.0 x 3.1 mm", "6.8 uH, Isat 3.65 A, AEC-Q200",
              "0.61 A / 3.65 A", "U3 inductor", "Sn over Ag: coat", "Digi-Key: Active (EPS board)")
    conn(p, {"A": "U3_SW", "B": "3V3_PRE"})
    for _ in range(2):
        C("22u", "3V3_PRE", "GND", 16, "1210", vop=3.33, note="U3 output")
    C("100n", "3V3_PRE", "GND")
    TP("3V3_PRE")


def clock():
    ic(S_XO, "Y1", "1.000MHz", "SG-8018CA 1.0000M-TJHPA0", "Epson", "7.0 x 5.0 mm 4-SMD",
       "programmable CMOS XO 1.62-3.63 V, +/-50 ppm, -40..105 C",
       {"OE": "Y1_OE", "GND": "GND", "OUT": "Y1_OUT", "VCC": "3V3_PRE"}, stress="3.3 V / 3.63 V operating",
       notes="sync clock for U1/U3 (EMC rule E1 frequency plan: harmonics n x 1.000 MHz as the EPS)", finish="Au",
       checked="not checked: frequency code per Epson SG-8018 ordering scheme (EPS uses 4.0000M-TJHPA0)")
    C("100n", "3V3_PRE", "GND", note="Y1")
    R(100e3, "3V3_PRE", "Y1_OE", note="OE delay 26 ms: clock off during converter start-up (LM61440 9.3.2)")
    C("220n", "Y1_OE", "GND", 25)
    R(33, "Y1_OUT", "SYNC_CLK", note="source termination")
    R(100e3, "SYNC_CLK", "GND", note="clock line low while Y1 is disabled")
    ic(S_LVC04, "U12", "SN74LVC1G04-Q1", "SN74LVC1G04QDCKRQ1", "TI", "SC70-5 (DCK)", "inverter 1.65-5.5 V, AEC-Q100",
       {"NC": None, "A": "SYNC_CLK", "GND": "GND", "Y": "SYNC_N", "VCC": "3V3_PRE"}, stress="3.3 V / 5.5 V",
       notes="180 deg clock for U3 (input-ripple interleave)")
    C("100n", "3V3_PRE", "GND", note="U12")
    TP("SYNC_CLK")


def power_control():
    # POWER_EN: wired AND of U2 PG and U4 RESET, pulled up to VDD_IN; the module has 100 k to GND (DS-11105-001 3-1)
    R(10e3, "VDD_IN", "POWER_EN", note="POWER_EN pull-up: 4.55 V high with the module's 100 k")
    ic(S_TPS3808, "U4", "TPS3808G01-Q1", "TPS3808G01QDBVRQ1", "TI", "SOT-23-6 (DBV)",
       "supervisor, 0.405 V adj. threshold +/-1.5 %, CT delay, MR input, open-drain RESET, AEC-Q100",
       {"RESET": "POWER_EN", "GND": "GND", "MR": "LATCH_QN", "CT": "U4_CT", "SENSE": "U4_SENSE", "VDD": "3V3_PRE"},
       stress="VDD 3.3 V / 6.5 V = 51 %; RESET 5.2 V / 7 V", notes="VBAT_F good (10.49 V) for 19.4 ms before POWER_EN; "
       "drops POWER_EN within 20 us of an input loss; MR = shutdown latch")
    C("100n", "3V3_PRE", "GND", note="U4")
    R(249e3, "VBAT_F", "U4_SENSE", tol=0.1, note="SENSE: 10.49 V (10.32-10.66)")
    R(10.0e3, "U4_SENSE", "GND", tol=0.1)
    C("3.3n", "U4_CT", "GND", note="tD = CT/175 + 0.5 ms = 19.4 ms")
    # shutdown latch: SHUTDOWN_REQ* (module, open drain, ~5 k to VDD_IN) -> divider -> U6 (powered only while the module
    # runs, Ioff otherwise) -> PRE* of U5 -> Q* low -> U4 MR -> POWER_EN low until the carrier loses power
    R(10e3, "SHUTDOWN_REQ_N", "SDREQ_DIV", note="divider: 5 V open-drain level to 2.9-3.5 V for U6")
    R(20e3, "SDREQ_DIV", "GND")
    ic(S_LVC07, "U6", "SN74LVC1G07-Q1", "SN74LVC1G07QDCKRQ1", "TI", "SC70-5 (DCK)",
       "buffer, open-drain output, Ioff, 1.65-5.5 V, AEC-Q100",
       {"NC": None, "A": "SDREQ_DIV", "GND": "GND", "Y": "LATCH_SET_N", "VCC": "3V3_IO"}, stress="VCC 3.3 V / 5.5 V; A 3.5 V",
       notes="gates SHUTDOWN_REQ* by the module's power-sequence state (VCC = 3V3_IO)")
    C("100n", "3V3_IO", "GND", note="U6")
    R(100e3, "3V3_PRE", "LATCH_SET_N", note="PRE* pull-up")
    ic(S_LVC74, "U5", "SN74LVC1G74", "SN74LVC1G74DCUT", "TI", "VSSOP-8 (DCU)",
       "D flip-flop with PRE/CLR, Ioff, 1.65-5.5 V, -40..125 C", {"CLK": "GND", "D": "GND", "QN": "LATCH_QN", "GND": "GND",
       "Q": "SD_LATCHED", "CLRN": "POR_N", "PREN": "LATCH_SET_N", "VCC": "3V3_PRE"}, stress="VCC 3.3 V / 5.5 V = 61 %",
       notes="shutdown latch: set by SHUTDOWN_REQ*, cleared at power-up only (no -Q1 grade offered: industrial)",
       checked="TI product page: SN74LVC1G74DCUT listed (2026-10-27)")
    C("100n", "3V3_PRE", "GND", note="U5")
    R(100e3, "3V3_PRE", "POR_N", note="power-on clear: 12 ms")
    C("100n", "POR_N", "GND")
    TP("POWER_EN")
    TP("SD_LATCHED")


def io_rails():
    ic(S_TPS22918, "U7", "TPS22918-Q1", "TPS22918TDBVRQ1", "TI", "SOT-23-6 (DBV)",
       "load switch 1-5.5 V, 2 A, 52 mOhm, QOD, AEC-Q100", {"VIN": "3V3_PRE", "GND": "GND", "ON": "SYS_RESET_N",
       "CT": "U7_CT", "QOD": "3V3_IO", "VOUT": "3V3_IO"}, stress="3.3 V / 5.5 V; 5 mA / 2 A",
       notes="3V3_IO enabled by SYS_RESET* (DS-11105-001 3.6.1); QOD to VOUT: < 0.5 V in 0.09 ms after SYS_RESET* low")
    C("1u", "3V3_PRE", "GND", 25, note="U7 VIN")
    C("1n", "U7_CT", "GND", note="U7 slew")
    C("1u", "3V3_IO", "GND", 25, note="U7 VOUT")
    ic(S_TPS22918, "U8", "TPS22918-Q1", "TPS22918TDBVRQ1", "TI", "SOT-23-6 (DBV)",
       "load switch 1-5.5 V, 2 A, 52 mOhm, QOD, AEC-Q100", {"VIN": "3V3_PRE", "GND": "GND", "ON": "SYS_RESET_N",
       "CT": "U8_CT", "QOD": "3V3_SSD", "VOUT": "3V3_SSD"}, stress="3.3 V / 5.5 V; 0.61 A / 2 A = 31 %",
       notes="3V3_SSD for the M.2 NVMe, on with SYS_RESET* (before PERST# release)")
    C("1u", "3V3_PRE", "GND", 25, note="U8 VIN")
    C("2.2n", "U8_CT", "GND", note="U8 slew: inrush 0.1 A into 44 uF")
    for _ in range(2):
        C("22u", "3V3_SSD", "GND", 16, "1210", vop=3.33, note="M.2 3.3 V bulk")
        C("100n", "3V3_SSD", "GND", note="M.2 3.3 V HF at J5")
    ic(S_TLV733, "U9", "TLV73318P-Q1", "TLV73318PQDBVRQ1", "TI", "SOT-23-5 (DBV)",
       "LDO 1.8 V 300 mA, active output pull-down 120 Ohm, AEC-Q100", {"IN": "3V3_IO", "GND": "GND", "EN": "3V3_IO",
       "NC": None, "OUT": "1V8_IO"}, stress="IN 3.3 V / 5.5 V; 1 mA / 300 mA",
       notes="1V8_IO: module-side supply of U10/U11 (module UARTs are 1.8 V CMOS, DS-11105-001 Table 2-13)")
    C("1u", "3V3_IO", "GND", 25, note="U9 IN")
    C("1u", "1V8_IO", "GND", 25, note="U9 OUT")
    R(100, "3V3_IO", "JET_3V3", note="JET_3V3 to W5B pin 4 (E9): <= 1 mA drawn, a harness short limited to 33 mA")
    TP("3V3_IO")
    TP("3V3_SSD")
    TP("1V8_IO")


def obc_link():
    j = place(CONN("PicoClasp_6", [(i, f"P{i}") for i in range(1, 7)]), "J2", "OBC LINK", "5013310607", "Molex",
              "Pico-Clasp 1.0 mm 6 pos SMT", "1 A/contact", "< 1 mA",
              "W5B: GND, JET_RXD (OBC TX), JET_TXD (OBC RX), JET_3V3, PT_NTC, PT_RTN (pins 5-6: proposed change E10)",
              "Sn", "Digi-Key: Active (EPS/OBC boards)")
    conn(j, {"P1": "GND", "P2": "JET_RXD", "P3": "JET_TXD", "P4": "JET_3V3", "P5": "PT_NTC", "P6": "PT_RTN"})
    esd4("D2", ["JET_RXD", "JET_TXD", "JET_3V3", "PT_NTC"], "ESD at J2 (W5B)")
    # UART1 of the module (1.8 V) <-> 3.3 V harness: U10 module TX -> OBC, U11 OBC -> module RX
    ic(S_LVC1T45, "U10", "SN74LVC1T45-Q1", "SN74LVC1T45QDCKRQ1", "TI", "SC70-6 (DCK)",
       "1-bit dual-supply translator 1.65-5.5 V, Ioff and VCC isolation, AEC-Q100",
       {"VCCA": "1V8_IO", "GND": "GND", "A": "UART1_TXD", "B": "JET_TXD_B", "DIR": "1V8_IO", "VCCB": "3V3_IO"},
       stress="3.3 V / 5.5 V", notes="UART1_TXD (module pin 203, 1.8 V) -> JET_TXD; DIR high = A to B")
    ic(S_LVC1T45, "U11", "SN74LVC1T45-Q1", "SN74LVC1T45QDCKRQ1", "TI", "SC70-6 (DCK)",
       "1-bit dual-supply translator 1.65-5.5 V, Ioff and VCC isolation, AEC-Q100",
       {"VCCA": "1V8_IO", "GND": "GND", "A": "UART1_RXD", "B": "JET_RXD_B", "DIR": "GND", "VCCB": "3V3_IO"},
       stress="3.3 V / 5.5 V", notes="JET_RXD -> UART1_RXD (module pin 205); DIR low = B to A")
    for net in ("1V8_IO", "3V3_IO", "1V8_IO", "3V3_IO"):
        C("100n", net, "GND", note="U10/U11 VCCA/VCCB")
    R(33, "JET_TXD_B", "JET_TXD", note="series, as the OBC side")
    R(33, "JET_RXD", "JET_RXD_B", note="series, as the OBC side")
    R(100e3, "3V3_IO", "JET_RXD_B", note="idle high while the OBC side is high-impedance")
    # spreader thermistor pass-through: J3 (flying leads of RT1 on the spreader) -> J2-5/6; never powered by the carrier
    j3 = place(CONN("PicoClasp_2", [(1, "P1"), (2, "P2")]), "J3", "SPREADER NTC", "5013310207", "Molex",
               "Pico-Clasp 1.0 mm 2 pos SMT", "1 A/contact", "< 0.2 mA", "RT1 leads from the heat spreader", "Sn",
               "Digi-Key: Active (EPS board)")
    conn(j3, {"P1": "PT_NTC", "P2": "PT_RTN"})
    p = place(TNTC, "RT1", "10k NTC", "GA10K3MCD1", "TE Connectivity (Measurement Specialties)", "epoxy bead, 0.25 mm leads",
              "10 kOhm at 25 C, +/-0.2 C 0-70 C, B25/85 3976 K, -80..150 C", "bias current set by the OBC (<= 0.2 mA)",
              "spreader temperature PT (payload_guard SPREADER_MAX_C 70 C, MIN_POWER_ON_C -15 C): bonded to the spreader "
              "with Stycast 2850FT/24LV, leads to J3", "Sn leads", "not checked")
    conn(p, {"A": "PT_NTC", "B": "PT_RTN"})


def module_socket():
    pins = [(n, pin_name(n)) for n in range(1, 261)]
    j = place(CONN("SODIMM_260", pins), "J4", "ORIN NANO", "2309407-2", "TE Connectivity", "DDR4 SO-DIMM 260 pos, 0.5 mm, 4.0 mm stack, SMT",
              "0.5 A/contact (distributor listing), -55..85 C", "VDD_IN 2.26 A / 10 contacts = 45 %",
              "module socket; pin functions DS-11105-001 v1.5 4.4", "Au contacts", "Mouser listing (DDR4 SODIMM 260P 4.0H STD); "
              "check against the DG-10931-001 connector list")
    m = {}
    for n in range(1, 261):
        nm = MODULE_PINS[n]
        if nm == "GND":
            m[pin_name(n)] = "GND"
        elif nm == "VDD_IN":
            m[pin_name(n)] = "VDD_IN"
        elif n in MODULE_USE:
            m[pin_name(n)] = MODULE_USE[n]
        else:
            m[pin_name(n)] = None
    conn(j, m)
    TP("SYS_RESET_N")
    TP("SHUTDOWN_REQ_N")
    TP("MODULE_ID")
    TP("MOD_SLEEP_N")
    TP("GND")
    TP("GND")


def nvme():
    # PCI-SIG M.2 Socket 3 Key M pinout (pins 59-66 are the key); host transmit -> PERp/n0 (module receive -> PETp/n0)
    m2 = {1: "GND", 2: "3V3", 3: "GND", 4: "3V3", 9: "GND", 12: "3V3", 14: "3V3", 15: "GND", 16: "3V3", 18: "3V3",
          21: "GND", 27: "GND", 33: "GND", 39: "GND", 41: "PERn0", 43: "PERp0", 45: "GND", 47: "PETn0", 49: "PETp0",
          50: "PERST_N", 51: "GND", 52: "CLKREQ_N", 53: "REFCLKn", 54: "PEWAKE_N", 55: "REFCLKp", 57: "GND", 69: "PEDET",
          70: "3V3", 71: "GND", 72: "3V3", 73: "GND", 74: "3V3", 75: "GND"}
    nums = [n for n in range(1, 76) if not 59 <= n <= 66]
    pins = [(n, (m2.get(n, "NC") + f"_{n}")) for n in nums] + [("MP1", "MP1"), ("MP2", "MP2")]
    j = place(CONN("M2_KEY_M", pins), "J5", "M.2 KEY M", "1-2199230-4", "TE Connectivity", "M.2 Key M 67 pos, 0.5 mm, 4.2 mm, SMT",
              "0.5 A/contact (M.2), Au", "3.3 V 0.61 A / 9 contacts", "NVMe SSD socket (2230 card, screw retained, staked)",
              "Au contacts", "distributor listing (RS, Digi-Key: Key M variant of 2199230)")
    net = {"GND": "GND", "3V3": "3V3_SSD", "PERn0": "M2_PERN0", "PERp0": "M2_PERP0", "PETn0": "PCIE0_RX0_N",
           "PETp0": "PCIE0_RX0_P", "PERST_N": "PCIE0_RST_N", "CLKREQ_N": "PCIE0_CLKREQ_N", "REFCLKn": "PCIE0_CLK_N",
           "REFCLKp": "PCIE0_CLK_P", "PEWAKE_N": "PCIE_WAKE_N", "PEDET": None}
    mp = {}
    for n, nm in pins:
        base = nm.rsplit("_", 1)[0] if nm not in ("MP1", "MP2") else nm
        mp[nm] = "GND" if nm in ("MP1", "MP2") else net.get(base)
    conn(j, mp)
    C("220n", "PCIE0_TX0_N", "M2_PERN0", 16, "0402", vop=1.0,
      note="PCIe TX AC coupling (host side, PCIe CEM convention); fit 0 Ohm if DG-10931-001 puts it on the module")
    C("220n", "PCIE0_TX0_P", "M2_PERP0", 16, "0402", vop=1.0,
      note="PCIe TX AC coupling (host side, PCIe CEM convention); fit 0 Ohm if DG-10931-001 puts it on the module")


def debug_header():
    pins = [(i, f"P{i}") for i in range(1, 11)]
    j = place(CONN("FTSH_2x5", pins), "J6", "DEBUG", "FTSH-105-01-L-DV-K", "Samtec", "FTSH 1.27 mm 2x5 SMT", "2.4 A/contact",
              "signals only", "GSE only (flashing and console); unmated in flight, covered", "Au", "Samtec part family as OBC/EPS")
    conn(j, {"P1": "USB0_DP", "P2": "USB0_DN", "P3": "GND", "P4": "DBG_TXD", "P5": "DBG_RXD", "P6": "GND",
             "P7": "FORCE_RECOVERY_N", "P8": "DBG_SYS_RESET", "P9": "GND", "P10": "DBG_LATCH"})
    R(100, "UART2_TXD", "DBG_TXD", note="debug console (UART2, 1.8 V CMOS, /dev/ttyTCU0)")
    R(100, "DBG_RXD", "UART2_RXD", note="debug console")
    R(1e3, "SYS_RESET_N", "DBG_SYS_RESET", note="observation only")
    R(1e3, "SD_LATCHED", "DBG_LATCH", note="observation only")
    esd4("D3", ["USB0_DP", "USB0_DN", "DBG_TXD", "DBG_RXD"], "ESD at J6 (USB0 recovery/flash, console)")
    p = place(S_ESD1, "D4", "TPD1E05U06-Q1", "TPD1E05U06QDPYRQ1", "TI", "X1SON-2 (DPY)", "1-ch ESD, 5.5 V VRWM, AEC-Q101",
              "1.8 V / 5.5 V", "ESD at J6-7 FORCE_RECOVERY*", "NiPdAu", "TI orderable addendum read: Active (OBC board)")
    conn(p, {"IO": "FORCE_RECOVERY_N", "GND": "GND"})


def mechanical():
    for ref, note in (("H1", "PC/104 (+40, +43)"), ("H2", "PC/104 (-40, +43)"), ("H3", "PC/104 (-40, -43)"),
                      ("H4", "PC/104 (+40, -43)"), ("H5", "centre support (0, +21)"), ("H6", "centre support (0, -21)")):
        p = TMH(value="MH")
        p.ref = ref
        p.footprint = "MountingHole:MountingHole_3.2mm_M3" if ref <= "H4" else "MountingHole:MountingHole_3.2mm_M3_7mm_keepout"
        p["MH"] += builtins.NC  # isolated from GND: copper-free ring (single frame bond is on the EPS board, H1)
    for ref in ("H7", "H8", "H9"):
        p = TMH(value="MH")
        p.ref = ref
        p.footprint = "MountingHole:MountingHole_2.2mm_M2_Pad_Via" if ref == "H9" else "MountingHole:MountingHole_3.2mm_M3_Pad_Via"
        link("GND", p["MH"])  # module standoffs (H7, H8) and the M.2 standoff (H9): plated, to GND
    EXTRA_BOM.extend([
        ["A1", 1, "Jetson Orin Nano 8GB", "900-13767-0030-000", "NVIDIA", "260-pin SO-DIMM module 69.6 x 45 mm, 28 g",
         "VDD_IN 4.75-5.25 V (MODULE_ID low), Tj -25..105 C", "7 W mode (capped 6.5 W)", "P3767-0003; flashed with the "
         "original (non-Super) jetson-orin-nano-devkit configuration (TB-13)", "Au fingers", "BASELINE 6"],
        ["A2", 1, "NVMe SSD 40 GB pSLC", "SN2600MA040GI-1TB2-2DB-STD", "Swissbit", "M.2 2230 (S4), 3.3 V only",
         "PCIe Gen3 x4 (runs x1), -40..85 C ambient, 95 C case, power-fail protection", "Gen2 x1 (DT max-link-speed 2)",
         "rootfs A/B, model and data partitions (N2600 fact sheet rev 1.04)", "Au fingers", "Mouser listing (new product)"],
        ["MP1", 1, "M.2 standoff M2", "9774035243R", "Wurth Elektronik", "SMT steel spacer M2 x 3.5 mm",
         "height to the TE 1-2199230-4 card plane", "", "with an M2 x 3 A2 screw, thread-locked (NASA-STD-5020)", "Sn",
         "not checked: height to be matched to the connector drawing"],
        ["MP2 MP3", 2, "module standoff", "9774040360R", "Wurth Elektronik", "SMT steel spacer M3 x 4.0 mm",
         "module far-edge holes; height = 4.0 mm stack", "TIM reaction <= 38 N", "thread and hole size per the NVIDIA "
         "module drawing (DS-11105-001 5.5)", "Sn", "not checked"],
        ["MP4", 1, "gap pad over the SoC", "GAP PAD 3500ULM (to the pedestal size)", "Henkel Bergquist",
         "1.0 mm die-cut", ">= 3.5 W/m K, <= 70 kPa at 35 % deflection, ASTM E595 TML <= 1 %, CVCM <= 0.1 %",
         "28 N at the largest deflection", "THM-10 path (section 9)", "-", "not checked: confirm in the NASA outgassing database"],
    ])


# ================================================================================================= build
def build():
    input_and_filter()
    buck_5v()
    vdd_in_efuse()
    rail_3v3()
    clock()
    power_control()
    io_rails()
    obc_link()
    module_socket()
    nvme()
    debug_header()
    mechanical()
    for name, n in _nets.items():
        if name in POWER_NETS:
            n.drive = POWER
    used = defaultdict(set)
    auto = []
    for prt in builtins.default_circuit.parts:
        if id(prt) in AUTOREF:
            auto.append(prt)
        else:
            mm = re.match(r"([A-Z]+)(\d+)$", prt.ref)
            if mm:
                used[mm.group(1)].add(int(mm.group(2)))
    nxt = defaultdict(lambda: 1)
    for prt in auto:
        pre = prt.ref_prefix
        while nxt[pre] in used[pre]:
            nxt[pre] += 1
        prt.ref = f"{pre}{nxt[pre]}"
        used[pre].add(nxt[pre])


def outputs():
    circ = builtins.default_circuit
    erc_path = os.path.join(HERE, "jetson_carrier_erc.log")
    buf = io.StringIO()
    h = logging.StreamHandler(buf)
    h.setLevel(logging.INFO)
    lg = skidl.erc_logger if hasattr(skidl, "erc_logger") else logging.getLogger("ERC_Logger")
    lg.addHandler(h)
    ERC()
    lg.removeHandler(h)
    with open(erc_path, "w") as f:
        f.write("SQSO-JET-BRD-001 electrical rule check (SKiDL 3.0 ERC), generated by design.py\n\n")
        f.write(buf.getvalue() or "No errors or warnings found while running ERC.\n")
        f.write("\nWaivers (intentional modelling choices, reviewed):\n"
                "W1  Connector pins (J1-J6) are PASSIVE; the module socket J4 carries the module's own drivers (POWER_EN is a\n"
                "    module input, SYS_RESET* and SHUTDOWN_REQ* are module open-drain outputs with module pull-ups, the UARTs\n"
                "    are 1.8 V CMOS): nets driven only through J4 are correct.\n"
                "W2  Power nets fed through passive elements (connector, filter inductor, buck inductors, damper) are declared\n"
                "    driven (net.drive = POWER): VBAT_IN, VBAT_F, DAMP, 5V_BUCK, 3V3_PRE, JET_3V3.\n"
                "W3  Pins marked NOCONNECT are open on purpose: PGOOD of U1 and U3 (not used: U2 PG and the module's own\n"
                "    sequence govern), the NC pins of U6, U9, U12 and the ESD arrays, every module pin not in MODULE_USE (no\n"
                "    camera, display, USB 1/2, USB 3.2, GbE, CAN, I2S, SPI, PCIe 1-3, PMIC_BBAT, CLK_32K_OUT, SLEEP/WAKE*;\n"
                "    DS-11105-001 4.4), the unused M.2 pins (lanes 1-3, DEVSLP, SMBus, SUSCLK, PEDET) and the isolated\n"
                "    mounting holes H1-H6.\n"
                "W4  POWER_EN has three open-drain/open-collector drivers (U2 PG, U4 RESET) and one pull-up: a wired AND.\n"
                "W5  PT_NTC/PT_RTN connect only passives (RT1, J2, J3, D2): the OBC biases and reads the thermistor.\n")
    generate_netlist(file_=os.path.join(HERE, "jetson_carrier.net"), tool=KICAD8)
    with open(os.path.join(HERE, "jetson_carrier_nets.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["net", "pins"])
        for net in sorted(circ.nets, key=lambda n: n.name):
            if not net.pins:
                continue
            w.writerow([net.name, " ".join(sorted(f"{p.part.ref}.{p.num}" for p in net.pins))])
    groups = OrderedDict()
    for p in circ.parts:
        info = BOMINFO.get(id(p))
        if info is None:
            continue
        groups.setdefault((info["mpn"], info["value"]), []).append((p.ref, info))

    def refkey(r):
        mm = re.match(r"([A-Z]+)(\d+)", r)
        return (mm.group(1), int(mm.group(2))) if mm else (r, 0)

    rows = []
    for (mpn, value), items in groups.items():
        refs = sorted((r for r, _ in items), key=refkey)
        info = items[0][1]
        stresses = sorted({i["stress"] for _, i in items if i["stress"]})
        notes = sorted({i["notes"] for _, i in items if i["notes"]})
        rows.append([" ".join(refs), len(refs), value, mpn, info["mfr"], info["pkg"], info["rating"],
                     "; ".join(stresses)[:300], "; ".join(notes)[:400], info["finish"], info["checked"]])
    rows.sort(key=lambda r: refkey(r[0].split()[0]))
    with open(os.path.join(HERE, "jetson_carrier_bom.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["Reference", "Qty", "Value", "MPN", "Manufacturer", "Package", "Rating", "Derated stress", "Notes",
                    "Terminal finish / tin mitigation", "MPN check"])
        w.writerows(rows)
        w.writerows(EXTRA_BOM)
    for junk in ("design.erc", "design.log", "design_sklib.py", "skidl_REPL.erc", "skidl_REPL.log"):
        if os.path.exists(os.path.join(HERE, junk)):
            os.remove(os.path.join(HERE, junk))
    total = sum(r[1] for r in rows)
    print(f"parts: {total} placements in {len(rows)} BOM lines (+ {sum(r[1] for r in EXTRA_BOM)} installed/mechanical items), "
          f"{len([n for n in circ.nets if n.pins])} nets")


if __name__ == "__main__":
    os.chdir(HERE)
    build()
    outputs()
