#!/usr/bin/env python3
"""SMART-QSO EPS board, schematic captured as code (SKiDL 3.0). Document: SQSO-EPS-BRD-001 (EPS_BOARD_DESIGN.md).

Run with the CAD venv (SKiDL 3.0, no KiCad libraries needed: every symbol is defined here from its datasheet pin
table):

    <venv>/bin/python hardware/electrical/eps/design.py

Writes, next to this file: eps_board.net (KiCad 8 netlist), eps_board_bom.csv, eps_board_erc.log,
eps_board_nets.csv (net -> pins, for review without KiCad).

Domains (EPS_BOARD_DESIGN.md section 3):
  PACK domain  referenced to BAT_BOT (cell 1 negative): cells, fuses F2/F3, the AFE U1 and its FETs, the secondary
               protector U2, DS2's back-to-back FETs (between PACK_N and GND).
  GND domain   everything else. Signals that cross go through the I2C isolator U50 or series resistors >= 100 kOhm,
               so that nothing but DS2 joins PACK_N to GND (the inhibit cannot be bypassed by a sneak path).
Rails: VBAT (bus, downstream of DS1/DS3), 3V3_AO, 3V3_MCU (SW9), 5V_PA, V_BURN (burn buck, behind SW6), VCTL
(3.3 V from the strings: charge-control loops), VHW (diode-OR of VCTL and 3V3_AO: NTC1 comparators), REG1 (AFE LDO),
REF_C / REF_A (2.500 V references in the VCTL and 3V3_AO domains).
"""

import builtins
import csv
import os
import sys
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
    """Named net (every net on this board is named)."""
    if name not in _nets:
        _nets[name] = Net(name)
    return _nets[name]


POWER_NETS = ["BAT_TOP", "BAT_BOT", "PACK_N", "PACK_P", "BAT_F", "BAT_F3", "GND", "VBAT", "VB1", "VB3", "3V3_AO",
              "3V3_MCU", "5V_PA", "V_BURN", "VBAT_DEP", "VCTL", "VCTL_IN", "VHW", "REG1", "REF_C", "REF_A", "VTRK",
              "VTRK_RAW", "VBAT_SW1", "VBAT_JET", "3V3_RADIO", "5V_PA_SW", "3V3_ADCS", "3V3_GNSS", "3V3_ANTS", "3V3_OBC",
              "AFE_REGIN", "REG18", "AFE_BATP", "VBAT_MTQ", "3V3_RTC"]


def sym(name, prefix, pins):
    """Symbol template: pins = 'num:NAME:type ...' (type in TY)."""
    plist = []
    for tok in pins.split():
        num, pname, ty = tok.split(":")
        plist.append(Pin(num=num, name=pname, func=TY[ty]))
    return Part(tool=SKIDL, name=name, ref_prefix=prefix, dest=TEMPLATE, pins=plist)


BOMINFO = {}  # id(part) -> dict
AUTOREF = set()


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
    n = N(name)
    n += pins


def noconn(pins):
    pins += builtins.NC


def conn(p, mapping):
    for pin, net in mapping.items():
        if net is None:
            noconn(p[pin])
        else:
            link(net, p[pin])


# ---- passive templates
TR = sym("R", "R", "1:A:P 2:B:P")
TC = sym("C", "C", "1:A:P 2:B:P")
TL = sym("L", "L", "1:A:P 2:B:P")
TFB = sym("FB", "FB", "1:A:P 2:B:P")
TD = sym("D", "D", "1:K:P 2:A:P")
TTVS = sym("TVS", "D", "1:K:P 2:A:P")
TDDUAL_CC = sym("BAT54C", "D", "1:A1:P 2:A2:P 3:K:P")          # common cathode
TDDUAL_S = sym("BAT54S", "D", "1:A:P 2:K:P 3:CA:P")            # series pair: pin3 = K1/A2 midpoint
TF = sym("F", "F", "1:A:P 2:B:P")
TPTC = sym("PTC", "RT", "1:A:P 2:B:P")
TNTC = sym("NTC", "RT", "1:A:P 2:B:P")
TTP = sym("TP", "TP", "1:TP:P")

E96 = None


def rcode(v):
    """Panasonic ERJ value code: 3 significant digits + multiplier (10.0k -> 1002)."""
    if v < 10:
        s = f"{v:.2f}".replace(".", "R")
        return s[:4]
    exp = 0
    while v >= 1000:
        v /= 10.0
        exp += 1
    while v < 100:
        v *= 10.0
        exp -= 1
    return f"{int(round(v)):03d}{exp}"


def fmt(v, unit="Ohm"):
    for th, suf in ((1e6, "M"), (1e3, "k")):
        if v >= th:
            x = v / th
            return (f"{x:.3g}{suf}")
    return f"{v:.3g}"


_rn = [0]


def R(v, a, b, size="0603", tol=1.0, pw=None, note="", stress="", ref=None):
    """Chip resistor: 1 % Panasonic ERJ-3EKF (AEC-Q200), 0.1 % Panasonic ERA-3AEB (AEC-Q200, 25 ppm)."""
    if size == "0603" and tol <= 0.1:
        mpn, rating = f"ERA-3AEB{rcode(v)}V", "0.1 %, 25 ppm/K, 0.1 W, 75 V"
    elif size == "0603" and v > 1.0e6:
        mant = f"{v / 1e6:.3g}".replace(".", "M") if v < 10e6 else "10M0"
        mant = mant if "M" in mant else mant + "M"
        mpn, rating = f"CRCW0603{mant.ljust(4, '0')}FKEA", "1 %, 100 ppm/K, 0.1 W, 75 V (Vishay, AEC-Q200)"
        p = place(TR, ref, fmt(v), mpn, "Vishay Dale", "Resistor_SMD:R_0603", rating, stress or "< 10 % power", note,
                  "Sn over Ni (matte); re-tin or coat", "series code")
        link(a, p["A"])
        link(b, p["B"])
        return p
    elif size == "0603":
        mpn, rating = (f"ERJ-3EKF{rcode(v)}V" if v >= 10 else f"ERJ-3RQF{rcode(v)}V"), "1 %, 100 ppm/K, 0.1 W, 75 V"
    elif size == "0805":
        mpn, rating = f"ERJ-6ENF{rcode(v)}V", "1 %, 0.125 W, 150 V"
    elif size == "2512":
        mpn, rating = f"ERJ-1TNF{rcode(v)}U", "1 %, 1 W, 200 V"
    else:
        raise ValueError(size)
    p = place(TR, ref, fmt(v), mpn, "Panasonic", f"Resistor_SMD:R_{size}", rating, stress or "< 10 % power", note,
              "Sn over Ni (matte); re-tin or coat per EEE_PARTS_LIST 2A.3", "series code")
    link(a, p["A"])
    link(b, p["B"])
    return p


def RSH(v, a, b, mpn, size, pw, note="", stress="", ref=None):
    p = place(TR, ref, f"{v * 1e3:.0f}m" if v < 1 else f"{v}", mpn, "Vishay Dale", f"Resistor_SMD:R_{size}",
              f"1 %, {pw} W, current sense", stress, note, "Sn (matte); coat", "series code")
    link(a, p["A"])
    link(b, p["B"])
    return p


CAPS = {  # (value, voltage, size) -> (mpn, dielectric)
    ("100n", 50, "0603"): ("GCM188R71H104KA57D", "X7R"),
    ("10n", 50, "0603"): ("GCM188R71H103KA37D", "X7R"),
    ("47n", 50, "0603"): ("GCM188R71H473KA55D", "X7R"),
    ("1n", 50, "0603"): ("GCM1885C1H102JA16D", "C0G"),
    ("100p", 50, "0603"): ("GCM1885C1H101JA16D", "C0G"),
    ("1u", 25, "0603"): ("GCM188R71E105KA64D", "X7R"),
    ("1u", 50, "0805"): ("GCM21BR71H105KA03L", "X7R"),
    ("2.2u", 25, "0805"): ("GCM21BR71E225KA73L", "X7R"),
    ("10u", 25, "1206"): ("GCM31CR71E106KA64L", "X7R"),
    ("4.7u", 50, "1210"): ("GCM32ER71H475KA55L", "X7R"),
    ("4.7u", 25, "0805"): ("GCM21BR71E475KA73L", "X7R"),
    ("22u", 16, "1210"): ("GCM32ER71C226KE19L", "X7R"),
}


def C(val, a, b, v=50, size="0603", stress="", note="", ref=None, vop=None):
    mpn, die = CAPS[(val, v, size)]
    st = stress or (f"{vop / v:.0%} of rated V" if vop else "<= 50 % of rated V")
    p = place(TC, ref, val, mpn, "Murata", f"Capacitor_SMD:C_{size}", f"{v} V {die} AEC-Q200", st, note,
              "Sn over Ni (matte); re-tin or coat", "series code")
    link(a, p["A"])
    link(b, p["B"])
    return p


def D(kind, k, a, ref=None, note=""):
    lib = {
        "BAT54": ("BAT54,215", "Nexperia", "SOD-523/SOT-23", "30 V, 200 mA Schottky", "Sn"),
        "PMEG40T30ER-Q": ("PMEG40T30ER-QX", "Nexperia", "SOD-123W", "40 V, 3 A Schottky, AEC-Q101", "Sn"),
        "SMBJ7.0A": ("SMBJ7.0A", "Littelfuse", "DO-214AA (SMB)", "TVS 7.0 V VRWM, 7.78 V VBR min, 12.0 V Vc at 50 A, 600 W", "Sn"),
        "SMBJ20A": ("SMBJ20A", "Littelfuse", "DO-214AA (SMB)", "TVS 20 V VRWM, 22.2 V VBR min, 32.4 V Vc, 600 W", "Sn"),
        "BZX84-C3V6": ("BZX84-C3V6,215", "Nexperia", "SOT-23", "Zener 3.6 V 250 mW", "Sn"),
        "1N4148W": ("1N4148W-Q-7-F", "Diodes Inc.", "SOD-123", "100 V 300 mA switching, AEC-Q101", "Sn"),
    }[kind]
    mpn, mfr, pkg, rating, fin = lib
    p = place(TTVS if kind.startswith("SMBJ") else TD, ref, kind, mpn, mfr, pkg, rating, "", note,
              f"{fin} (matte); re-tin or coat", "Digi-Key listing" if kind in ("SMBJ7.0A", "PMEG40T30ER-Q") else "not checked")
    link(k, p["K"])
    link(a, p["A"])
    return p


# ---- semiconductors and ICs (pins from the datasheets named in the notes)
TNFET = sym("NMOS", "Q", "1:G:P 2:S:P 3:D:P")
TPFET = sym("PMOS", "Q", "1:G:P 2:S:P 3:D:P")
TNPN = sym("NPN", "Q", "1:B:P 2:C:P 3:E:P")

FETS = {
    "BUK7Y3R5-40H": ("BUK7Y3R5-40HX", "Nexperia", "LFPAK56 (SOT669)", "N 40 V, 3.5 mOhm at 10 V, Vgs +20/-10 V, AEC-Q101", TNFET),
    "BUK9Y4R4-40E": ("BUK9Y4R4-40E,115", "Nexperia", "LFPAK56 (SOT669)", "N 40 V logic, 3.7 mOhm at 10 V, Vgs +/-10 V, AEC-Q101", TNFET),
    "BUK7M12-40E": ("BUK7M12-40EX", "Nexperia", "LFPAK33 (SOT1210)", "N 40 V, 12 mOhm at 10 V, Vgs +/-20 V, AEC-Q101", TNFET),
    "BUK9M14-40E": ("BUK9M14-40EX", "Nexperia", "LFPAK33 (SOT1210)", "N 40 V logic, 11 mOhm at 10 V, Vgs +/-10 V, AEC-Q101", TNFET),
    "IPD50P04P4L-11": ("IPD50P04P4L11ATMA2", "Infineon", "TO-252 (DPAK)", "P 40 V, 10.6 mOhm at -10 V, Vgs +5/-16 V, AEC-Q101", TPFET),
    "PMV250EPEA": ("PMV250EPEAR", "Nexperia", "SOT-23", "P 40 V, 240 mOhm, Vgs +/-20 V, AEC-Q101", TPFET),
    "PMV40UN2": ("PMV40UN2R", "Nexperia", "SOT-23", "N 30 V 3.7 A, 44 mOhm at 4.5 V, Vgs +/-12 V", TNFET),
    "2N7002BK": ("2N7002BK,215", "Nexperia", "SOT-23", "N 60 V 350 mA, Vgs +/-20 V, AEC-Q101", TNFET),
}


def Q(kind, ref, g, s, d, stress="", note=""):
    mpn, mfr, pkg, rating, tm = FETS[kind]
    p = place(tm, ref, kind, mpn, mfr, pkg, rating, stress, note, "Sn (matte) LFPAK/SOT; coat", "Digi-Key listing")
    conn(p, {"G": g, "S": s, "D": d})
    return p


# IC symbols: datasheet pin tables
S_BQ76952 = sym("BQ76952", "U", " ".join([
    "1:VC15:P", "2:VC14:P", "3:VC13:P", "4:VC12:P", "5:VC11:P", "6:VC10:P", "7:VC9:P", "8:VC8:P", "9:VC7:P",
    "10:VC6:P", "11:VC5:P", "12:VC4:P", "13:VC3:P", "14:VC2:P", "15:VC1:P", "16:VC0:P", "17:VSS:W", "18:SRP:P",
    "19:NC19:N", "20:SRN:P", "21:TS1:P", "22:TS2:P", "23:TS3:P", "24:REG18:WO", "25:ALERT:OC", "26:SCL:B",
    "27:SDA:B", "28:HDQ:B", "29:CFETOFF:I", "30:DFETOFF:I", "31:DCHG:O", "32:DDSG:O", "33:RST_SHUT:I", "34:REG2:WO",
    "35:REG1:WO", "36:REGIN:W", "37:BREG:O", "38:FUSE:B", "39:PDSG:O", "40:PCHG:O", "41:LD:P", "42:PACK:P",
    "43:DSG:O", "44:NC44:N", "45:CHG:O", "46:CP1:P", "47:BAT:W", "48:VC16:P"]))
S_BQ7718 = sym("BQ771806", "U", "1:VDD:W 2:V5:P 3:V4:P 4:V3:P 5:V2:P 6:V1:P 7:VSS:W 8:OUT:O")
S_TPS55340 = sym("TPS55340-Q1", "U", "1:SW1:P 2:VIN:W 3:EN:I 4:SS:P 5:SYNC:I 6:AGND:W 7:COMP:P 8:FB:I 9:FREQ:P "
                 "10:NC10:P 11:PGND11:W 12:PGND12:W 13:PGND13:W 14:NC14:P 15:SW15:P 16:SW16:P 17:PAD:W")
S_LM74700 = sym("LM74700-Q1", "U", "1:EN:I 2:GND:W 3:NC3:N 4:VCAP:P 5:ANODE:W 6:GATE:O 7:NC7:N 8:CATHODE:P")
S_LMR436 = sym("LMR43620-Q1", "U", "1:SYNC:I 2:PGOOD:OC 3:EN:I 4:VIN:W 5:SW:P 6:BOOT:P 7:VCC:WO 8:FB:I 9:GND:W")
S_TPS1H100 = sym("TPS1H100B-Q1", "U", "1:NC1:N 2:GND:W 3:IN:I 4:NC4:N 5:OUT5:WO 6:OUT6:P 7:OUT7:P 8:VS8:W "
                 "9:VS9:W 10:VS10:W 11:NC11:N 12:DIAG_EN:I 13:CL:P 14:CS:P 15:PAD:W")
S_TPS1H000 = sym("TPS1H000-Q1", "U", "1:IN:I 2:DIAG_EN:I 3:FAULT:OC 4:CL:P 5:DELAY:P 6:GND:W 7:OUT:WO 8:VS:W 9:PAD:W")
S_TPS1663 = sym("TPS16630", "U", "1:IN1:W 2:IN2:W 3:NC3:N 4:NC4:N 5:P_IN:W 6:UVLO:I 7:OVP:I 8:GND:W 9:DVDT:P "
                "10:ILIM:P 11:MODE:N 12:SHDN:I 13:IMON:P 14:FLT:OC 15:NC15:N 16:PGOOD:OC 17:OUT17:WO 18:OUT18:P "
                "19:NC19:N 20:NC20:N 21:NC21:N 22:NC22:N 23:NC23:N 24:NC24:N 25:PAD:W")
S_TPS2553 = sym("TPS2553-Q1", "U", "1:IN:W 2:GND:W 3:EN:I 4:FAULT:OC 5:ILIM:P 6:OUT:WO")
S_INA228 = sym("INA228-Q1", "U", "1:A1:I 2:A0:I 3:ALERT:OC 4:SDA:B 5:SCL:I 6:VS:W 7:GND:W 8:VBUS:P 9:INM:P 10:INP:P")
S_INA3221 = sym("INA3221-Q1", "U", "1:INM3:P 2:INP3:P 3:GND:W 4:VS:W 5:A0:I 6:SCL:I 7:SDA:B 8:WARN:OC 9:CRIT:OC "
                "10:PV:OC 11:INM1:P 12:INP1:P 13:TC:OC 14:INM2:P 15:INP2:P 16:VPU:P 17:PAD:W")
S_INA186 = sym("INA186-Q1", "U", "1:REF:P 2:GND:W 3:VS:W 4:INP:P 5:INM:P 6:OUT:O")
S_OPA2 = sym("OPAMP2", "U", "1:OUTA:O 2:INAM:I 3:INAP:I 4:VM:W 5:INBP:I 6:INBM:I 7:OUTB:O 8:VP:W")
S_CMP2 = sym("TLV7042-Q1", "U", "1:OUTA:OC 2:INAM:I 3:INAP:I 4:VEE:W 5:INBP:I 6:INBM:I 7:OUTB:OC 8:VCC:W")
S_REF34 = sym("REF3425-Q1", "U", "1:GND_F:W 2:GND_S:W 3:EN:I 4:IN:W 5:OUT_S:P 6:OUT_F:WO")
S_TPS709 = sym("TPS70933-Q1", "U", "1:IN:W 2:GND:W 3:EN:I 4:NC:N 5:OUT:WO")
S_TPL5111 = sym("TPL5111", "U", "1:VDD:W 2:GND:W 3:DELAY:P 4:DONE:I 5:DRVN:O 6:ONESHOT:I")
S_G1 = sym("LVC1G-2IN", "U", "1:A:I 2:B:I 3:GND:W 4:Y:O 5:VCC:W")
S_INV = sym("LVC1G04", "U", "1:NC:N 2:A:I 3:GND:W 4:Y:O 5:VCC:W")
S_SCH = sym("LVC1G17", "U", "1:NC:N 2:A:I 3:GND:W 4:Y:O 5:VCC:W")
S_FF = sym("SN74LVC2G74-Q1", "U", "1:CLK:I 2:D:I 3:QN:O 4:GND:W 5:Q:O 6:CLRN:I 7:PREN:I 8:VCC:W")
S_TCA9517 = sym("TCA9517-Q1", "U", "1:VCCA:W 2:SCLA:B 3:SDAA:B 4:GND:W 5:EN:I 6:SDAB:B 7:SCLB:B 8:VCCB:W")
S_ISO1540 = sym("ISO1540-Q1", "U", "1:VCC1:W 2:SDA1:B 3:SCL1:I 4:GND1:W 5:GND2:W 6:SCL2:O 7:SDA2:B 8:VCC2:W")
S_ADS7138 = sym("ADS7138-Q1", "U", "1:AIN2:P 2:AIN3:P 3:AIN4:P 4:AIN5:P 5:AIN6:P 6:AIN7:P 7:AVDD:W 8:DECAP:P 9:GND:W "
                "10:DVDD:W 11:ADDR:P 12:ALERT:OC 13:SCL:I 14:SDA:B 15:AIN0:P 16:AIN1:P 17:PAD:W")
S_OSC = sym("XO", "Y", "1:OE:I 2:GND:W 3:OUT:O 4:VCC:W")
S_RELAY = sym("TQ2SA-L2", "K", "1:SET_P:P 2:COM1:P 3:NO1:P 4:NC1:P 5:SET_N:P 6:RST_N:P 7:NC2:P 8:NO2:P 9:COM2:P 10:RST_P:P")
S_ITV = sym("ITV4030L1412", "F", "1:T1:P 2:HTR:P 3:T3:P")
MSP_PINS = {1: "P1_2", 2: "P1_1", 3: "P1_0", 4: "TEST", 5: "RSTN", 6: "DVCC", 7: "DVSS", 8: "P2_7", 9: "P2_6", 10: "P2_5",
            11: "P2_4", 12: "P4_7", 13: "P4_6", 14: "P4_5", 15: "P4_4", 16: "P6_6", 17: "P6_5", 18: "P6_4", 19: "P6_3",
            20: "P6_2", 21: "P6_1", 22: "P6_0", 23: "P4_3", 24: "P4_2", 25: "P4_1", 26: "P4_0", 27: "P2_3", 28: "P2_2",
            29: "P2_1", 30: "P2_0", 31: "P1_7", 32: "P1_6", 33: "P1_5", 34: "P1_4", 35: "P3_7", 36: "P3_6", 37: "P3_5",
            38: "P3_4", 39: "P5_4", 40: "P5_3", 41: "P5_2", 42: "P5_1", 43: "P5_0", 44: "P3_3", 45: "P3_2", 46: "P3_1",
            47: "P3_0", 48: "P1_3"}
S_MSP = sym("MSP430FR2355", "U", " ".join(f"{n}:{nm}:{'W' if nm in ('DVCC', 'DVSS') else 'P'}" for n, nm in MSP_PINS.items()))


def CONN(name, n, prefix="J"):
    return sym(name, prefix, " ".join(f"{i}:P{i}:P" for i in range(1, n + 1)))


def ic(tm, ref, value, mpn, mfr, pkg, rating, mapping, stress="", notes="", finish="NiPdAu", checked="TI addendum: Active"):
    p = place(tm, ref, value, mpn, mfr, pkg, rating, stress, notes, finish, checked)
    conn(p, mapping)
    return p


def TP(net, ref=None):
    p = place(TTP, ref, "TP", "5015", "Keystone", "TestPoint:Keystone_5015", "SMT test point", "", "", "Sn", "not checked")
    link(net, p["TP"])


# ================================================================================================= 1 connectors
def connectors():
    # Every harness connector on the bottom side in the SQSO-HAR-001 2.0 section 10.1 zones (Z1-Z5, +X edge); headers
    # in gold (EEE_PARTS_LIST 2A.3 item 5: no tin contacts). Pico-Clasp: Molex 203556 (0.38 um Au mating, Au flash post),
    # mates the 501330 housings with gold 501334 terminals. Micro-Fit 3.0: 43045 right-angle, 0.76 um Au mating.
    def pico(n, ref, val, rating_use, note, zone):
        chk = {6: "Digi-Key: Active (Au 0.38 um)", 8: "Digi-Key: listed"}.get(n, "pattern (203556-xx07): confirm (V-HAR-3)")
        return place(CONN(f"PicoClasp_{n}", n), ref, val, f"2035560{n}07", "Molex", f"Pico-Clasp 1.0 mm {n} pos vertical SMT, bottom side",
                     "1 A/contact", rating_use, f"{note}; zone {zone}", "Au 0.38 um mating, Au flash post", chk)

    def mfit(n, ref, val, rating_use, note, zone, chk):
        return place(CONN(f"MicroFit_2x{n // 2}", n), ref, val, f"043045{n:02d}02", "Molex",
                     f"Micro-Fit 3.0 2x{n // 2} right-angle THT, bottom side", "5 A/contact", rating_use, f"{note}; zone {zone}",
                     "Au 0.76 um mating; Sn post (soldered: SnPb assembly, coat)", chk)

    j = mfit(8, "J1", "BATTERY", "1.71 A max (fault) on 2 contacts per pole: 17 %", "W1 (pack leads, NTC1 and return, heater)",
             "Z1 (PACK corner, -X +Y)", "Digi-Key 0430450802: Active (R/A, Au 30 uin)")
    conn(j, {"P1": "BAT_TOP", "P2": "BAT_TOP", "P3": "BAT_BOT", "P4": "BAT_BOT", "P5": "NTC1", "P6": "GND",
             "P7": "HTR_P", "P8": "HTR_N"})
    j = pico(6, "J2", "CELL TAPS", "< 20 mA", "W1: cell taps and NTC2", "Z1")
    conn(j, {"P1": "TAP0", "P2": "TAP1", "P3": "TAP2", "P4": "TAP3", "P5": "TAP4", "P6": "NTC2"})
    # J3 OBC: W3 per SQSO-ICX-001 section 3.1 (40 positions; pins 21 and 27 are the analog return AGND, joined to GND at
    # the board's analog star point at layout); Samtec TFM vertical SMT on the bottom side for the SFSDT cable (HAR-001)
    pins = ["3V3_OBC", "3V3_OBC", "GND", "GND", "I2C2_SCL", "I2C2_SDA", "GND", "EPS_ALERT_N", "PAYLOAD_EN", "RADIO_EN",
            "ADCS_EN", "HEATER_EN", "DEPLOY_EN", "RF_TX_EN", "GNSS_EN", "DEPLOY_SENSE", "BURN_W2A", "BURN_W1A",
            "VJET_RB", "SOLAR_SENSE", "GND", "VBATT_SNS", "IBATT_SNS", "VSOLAR_SNS", "ISOLAR_SNS", "VBUS_SNS",
            "GND", "GND", "JETSON_PWR", "BURN_W1B", "BURN_W2B", "EOL_ARM", "EOL_FIRE", "LOCK_CLR", "GND", "VBAT_MTQ",
            "VSW1_RB", "3V3_RTC", "VBAT_MTQ", "GND"]
    j = place(CONN("TFM_2x20", 40), "J3", "OBC", "TFM-120-02-S-D-A", "Samtec", "Tiger Eye TFM 1.27 mm 2x20 vertical SMT, bottom side",
              "Samtec rating per contact (SFSDT 30 AWG: 0.5 A derated, SQSO-HAR-001)",
              "3V3_OBC 0.18 A and VBAT_MTQ 0.40 A (SW11 limit max) on 2 pins each",
              "W3 (SFSDT discrete-wire cable) per SQSO-ICX-001 3.1; +X edge, |y| < 15.5 mm", "Au (code -S, 30 uin)",
              "pattern from the Samtec TFM series: lead style -02 (SMT), plating -S and latching option to confirm with Samtec (V-HAR-3)")
    for i, nm in enumerate(pins, 1):
        link(nm, j[f"P{i}"])
    # solar strings: body J4 PX, J5 MX, J6 PY, J7 MY (SA+, SA-, TEMP); wings J8 W1, J9 W2 (SA1+ front, SA2+ back, SA-, SA-, TEMP)
    for ref, name, zone in (("J4", "PX", "Z4"), ("J5", "MX", "Z2"), ("J6", "PY", "Z1"), ("J7", "MY", "Z2")):
        j = pico(5, ref, f"STRING {name}", "0.55 A Isc on two contacts and two 28 AWG wires per pole: 0.28 A each "
                 "(28 % of the contact, 37 % of the 28 AWG derated 0.90 A, SQSO-HAR-001 5.2)", f"W2{name}", zone)
        conn(j, {"P1": f"S_{name}", "P2": f"S_{name}", "P3": "GND", "P4": "GND", "P5": f"TPNL_{name}"})
    for ref, w, zone in (("J8", "W1", "Z3"), ("J9", "W2", "Z5")):
        j = pico(8, ref, f"WING {w}", "0.55 A per string on two contacts; 1.1 A return on three", f"W2{w}", zone)
        conn(j, {"P1": f"S_{w}F", "P2": f"S_{w}F", "P3": f"S_{w}B", "P4": f"S_{w}B", "P5": "GND", "P6": "GND", "P7": "GND",
                 "P8": f"TPNL_{w}"})
    # wing burn resistors (two per wing) and hinge sense, J10 W1, J11 W2
    for ref, w, s_in, s_out, hz, zone in (("J10", "W1", "SA_SENSE_PU", "SA_SENSE_MID", "W7A", "Z3 (plug toward +X)"),
                                          ("J11", "W2", "SA_SENSE_MID", "GND", "W7B", "Z2")):
        j = mfit(6, ref, f"BURN {w}", "0.88 A: 18 %", f"{hz}: R_A+, R_A-, R_B+, R_B-, sense in, sense out", zone,
                 "Digi-Key 0430450602: Active (R/A, Au 30 uin)")
        conn(j, {"P1": "V_BURN", "P2": f"BRET_{w}A", "P3": "V_BURN", "P4": f"BRET_{w}B", "P5": s_in, "P6": s_out})
    j = pico(6, "J12", "AntS", "0.61 A AntS peak on two contacts: 31 %", "W7C: AntS supply and I2C (SQSO-HAR-001 pinout)", "Z5")
    conn(j, {"P1": "3V3_ANTS", "P2": "3V3_ANTS", "P3": "GND", "P4": "GND", "P5": "ANTS_SDA", "P6": "ANTS_SCL"})
    j = mfit(6, "J13", "UMBILICAL", "0.5 A charge: 10 %", "W11 to the access-port board A3: charge in, return, pack monitor, AFE shutdown",
             "Z5", "Digi-Key 0430450602: Active (R/A, Au 30 uin)")
    conn(j, {"P1": "UMB_CHG", "P2": "UMB_CHG", "P3": "PACK_N", "P4": "PACK_N", "P5": "UMB_VMON", "P6": "UMB_SHUT"})
    j = pico(4, "J14", "RBF", "uA", "W12D: RBF switch, two poles: A opens DS2's drive; B (closed with the pin in) carries "
             "VCTL to the tracker-clamp gate (active high: a wire to the frame cannot disable the trackers)", "Z4 (-Y edge)")
    conn(j, {"P1": "S2N", "P2": "S2R", "P3": "TCL_B_W", "P4": "RBF_C2"})
    # DS2 is a two-pole switch (DPDT): pole 1 (C, NC) drives the pack-return FETs from the PACK domain; pole 2 (C2 fed
    # from VCTL through 10 k, NO2 closed when stowed) drives the gate of the tracker clamp Q40 (active high: a chafed
    # wire to the frame removes the clamp, it cannot disable the trackers; RIDs P3E-01, SMA-B13)
    j = pico(5, "J16", "DS2", "uA", "W12B: DS2 deployment switch, two poles: 1 C, 2 NC (pack-return FET drive, PACK domain), "
             "3 NO1 not used, 4 C2 (VCTL via 10 k), 5 NO2 (tracker-clamp gate, closed when stowed); own harness on its own rail", "Z2")
    conn(j, {"P1": "DS2_C", "P2": "S2N", "P3": None, "P4": "DS2_C2", "P5": "TCL_A_W"})
    for ref, ds, c, nc, no, hz, zone in (("J15", "DS1", "DS1_C", "S1N", None, "W12A", "Z3"),
                                         ("J17", "DS3", "GND", "S3N", None, "W12C", "Z4")):
        j = pico(3, ref, ds, "uA", f"{hz}: {ds} deployment switch (C, NC = closed when released, NO = closed when stowed); "
                 "own harness on its own rail", zone)
        conn(j, {"P1": c, "P2": nc, "P3": no})
    j = mfit(4, "J18", "JETSON PWR", "0.93 A on 2 contacts: 9 %", "W5A: VBAT_JET, 2 x 24 AWG twisted quad", "Z2",
             "Digi-Key: 0430450402 listed (R/A 4 pos); finish and status to confirm")
    conn(j, {"P1": "VBAT_JET", "P2": "VBAT_JET", "P3": "GND", "P4": "GND"})
    # J19 RF power: Pico-Clasp 6 with the pinout of RF J2 (SQSO-HAR-001 E-3)
    j = pico(6, "J19", "RF PWR", "PA 0.55 A on 2 contacts: 28 %", "W4B: radio 3.3 V (switched), PA 5 V (switched, TX-OFF)", "Z2")
    conn(j, {"P1": "3V3_RADIO", "P2": "GND", "P3": "5V_PA_SW", "P4": "5V_PA_SW", "P5": "GND", "P6": "GND"})
    for ref, net, what, hz, zone in (("J20", "3V3_ADCS", "ADCS", "W6B to OBC J6 (ADCS sensor supply)", "Z1 (-X edge, +Y side)"),
                                     ("J21", "3V3_GNSS", "GNSS", "W9B to OBC J4 (GNSS receiver supply via the OBC)",
                                      "Z3 (+X edge, +Y side)")):
        j = pico(2, ref, f"{what} PWR", "<= 0.34 A at the trip", f"{hz}: {what} 3.3 V switched", zone)
        conn(j, {"P1": net, "P2": "GND"})
    j = place(CONN("Header_1x4", 4), "J22", "SBW", "FTSH-102-01-L-DV-K", "Samtec", "FTSH 1.27 mm 2x2", "", "",
              "MSP430 Spy-Bi-Wire programming (ground use only)", "Au", "not checked")
    conn(j, {"P1": "3V3_MCU", "P2": "MCU_TEST", "P3": "MCU_RSTN", "P4": "GND"})


# ================================================================================================= 2 solar inputs, trackers
STRINGS = [("W1F", "T1"), ("W2F", "T1"), ("PY", "T2"), ("W1B", "T3"), ("W2B", "T3"), ("PX", "T4"), ("MX", "T4"), ("MY", "T4")]


def solar_and_trackers():
    for i, (s, t) in enumerate(STRINGS, 1):
        D("SMBJ7.0A", f"S_{s}", "GND", ref=f"D{i}", note=f"string {s} TVS (EPS-13: 7.0 V standoff above the 6.31 V cold Voc)")
        ic(S_LM74700, f"U{89 + i}", "LM74700-Q1", "LM74700QDDFRQ1", "TI", "SOT-23-THIN-8 (DDF)",
           "3.2-65 V ideal-diode controller, AEC-Q100 grade 1",
           {"EN": f"S_{s}", "GND": "GND", "NC3": None, "VCAP": f"VCAP_{s}", "ANODE": f"S_{s}", "GATE": f"IDG_{s}",
            "NC7": None, "CATHODE": f"{t}_IN"}, stress=f"ANODE 12 V clamp / 65 V = 18 %", notes=f"blocking diode for string {s}")
        C("100n", f"VCAP_{s}", f"S_{s}", 50, note="LM74700 charge pump")
        Q("BUK7M12-40E", f"Q{30 + i}", f"IDG_{s}", f"S_{s}", f"{t}_IN", stress="Vds 12 V/40 V = 30 %; 0.55 A/48 A",
          note="ideal-diode FET (source = string, drain = tracker input)")
    # VCTL: diode-OR of the four tracker inputs -> TPS70933 (charge-control domain, powered by the array)
    p = place(TDDUAL_CC, "D20", "BAT54C", "BAT54C,215", "Nexperia", "SOT-23", "30 V 200 mA dual Schottky, common cathode", "", "VCTL OR",
              "Sn", "not checked")
    conn(p, {"A1": "T1_IN", "A2": "T2_IN", "K": "VCTL_IN"})
    p = place(TDDUAL_CC, "D21", "BAT54C", "BAT54C,215", "Nexperia", "SOT-23", "30 V 200 mA dual Schottky, common cathode", "", "VCTL OR",
              "Sn", "not checked")
    conn(p, {"A1": "T3_IN", "A2": "T4_IN", "K": "VCTL_IN"})
    C("1u", "VCTL_IN", "GND", 25, note="LDO input")
    ic(S_TPS709, "U40", "TPS70933-Q1", "TPS70933QDBVRQ1", "TI", "SOT-23-5 (DBV)", "30 V in, 3.3 V 150 mA, 1 uA Iq",
       {"IN": "VCTL_IN", "GND": "GND", "EN": "VCTL_EN", "NC": None, "OUT": "VCTL"}, stress="VIN 12 V clamp / 30 V = 40 %; EN <= 3.2 V / 6.5 V")
    R(100e3, "VCTL_IN", "VCTL_EN", note="EN divider (EN abs max 6.5 V)")
    R(100e3, "VCTL_EN", "GND")
    C("2.2u", "VCTL", "GND", 25, size="0805")
    R(300e3, "VCTL_IN", "VSOLAR_SNS", note="VSOLAR_SNS 1:4 (adc.rs Linear(4.0))")
    R(100e3, "VSOLAR_SNS", "GND")
    C("100n", "VSOLAR_SNS", "GND")
    ic(S_REF34, "U41", "REF3425-Q1", "REF3425QDBVRQ1", "TI", "SOT-23-6 (DBV)", "2.500 V +/-0.05 %, 6 ppm/K, AEC-Q100",
       {"GND_F": "GND", "GND_S": "GND", "EN": "VCTL", "IN": "VCTL", "OUT_S": "REF_C", "OUT_F": "REF_C"},
       finish="NiPdAuAg", notes="charge-control reference (VCTL domain)")
    C("1u", "REF_C", "GND", 25)
    C("100n", "VCTL", "GND")
    # tracker enables: pulled up from VCTL, clamped to GND by DS3 (NO, stowed) and RBF pole B (inserted)
    R(100e3, "VCTL", "TRK_EN", note="tracker enable pull-up; clamps Q40 (DS2 pole 2) and Q41 (RBF pole B) hold it low")
    # active-high clamps: the switch contact carries VCTL to the clamp gate; the TRK_EN node never leaves the board
    for qref, w, gate, src in (("Q40", "TCL_A_W", "TCL_A", "DS2_C2"), ("Q41", "TCL_B_W", "TCL_B", "RBF_C2")):
        R(10e3, "VCTL", src, note="contact feed (a short of this wire to the frame draws 0.33 mA)")
        R(10e3, w, gate, note="series from the harness")
        R(1e6, gate, "GND", note="clamp off when the contact is open or its wire is grounded")
        C("10n", gate, "GND")
        Q("2N7002BK", qref, gate, "GND", "TRK_EN", stress="Vds 3.3 V / 60 V", note=f"tracker clamp ({'DS2 pole 2' if qref == 'Q40' else 'RBF pole B'})")
    C("100n", "TRK_EN", "GND")
    TP("TRK_EN")
    syncs = {"T1": "SYNC_A", "T2": "SYNC_B", "T3": "SYNC_A", "T4": "SYNC_B"}
    for k, t in enumerate(("T1", "T2", "T3", "T4")):
        u = f"U{4 + k}"
        # input current shunt (INA3221) and input capacitors
        RSH(0.010, f"{t}_IN", f"{t}_L", "WSLP1206R0100FEA", "1206", 1.0, note=f"{t} input current (INA3221)",
            stress="1.1 A^2 x 10 mOhm = 12 mW of 1 W", ref=f"RS{1 + k}")
        C("10u", f"{t}_L", "GND", 25, "1206", vop=12.0)
        C("10u", f"{t}_L", "GND", 25, "1206", vop=12.0)
        p = place(TL, f"L{1 + k}", "6.8uH", "XGL4030-682MEC", "Coilcraft", "4.0 x 4.0 x 3.1 mm",
                  "6.8 uH, Isat 3.65 A, Irms 4.7 A, 47.9 mOhm max, AEC-Q200, -40..165 C", "1.27 A peak: 35 % of Isat",
                  f"{t} boost inductor", "Sn over Ag (Coilcraft RoHS); coat", "Digi-Key: Active")
        conn(p, {"A": f"{t}_L", "B": f"{t}_SW"})
        ic(S_TPS55340, u, "TPS55340-Q1", "TPS55340QRTERQ1", "TI", "WQFN-16 (RTE) 3x3",
           "2.9-38 V in, 40 V/5.25 A switch, sync 100 kHz-2.5 MHz, AEC-Q100 grade 1",
           {"SW1": f"{t}_SW", "SW15": f"{t}_SW", "SW16": f"{t}_SW", "VIN": f"{t}_L", "EN": f"{t}_ENP", "SS": f"{t}_SS",
            "SYNC": f"{t}_SYNC", "AGND": "GND", "COMP": f"{t}_COMP", "FB": f"{t}_FB", "FREQ": f"{t}_FREQ",
            "NC10": "GND", "NC14": "GND", "PGND11": "GND", "PGND12": "GND", "PGND13": "GND", "PAD": "GND"},
           stress="SW 16.8 V + 0.4 V / 40 V = 43 %; 1.27 A / 5.25 A", notes=f"MPPT tracker {t}")
        R(10e3, "TRK_EN", f"{t}_ENP")
        # per-tracker disable by the EPS controller (isolates a tracker with a shorted output diode; RID SMA-B17)
        Q("2N7002BK", f"Q{44 + k}", f"TDIS_{t}", "GND", f"{t}_ENP", note=f"{t} disable (EPS controller)")
        R(100e3, f"TDIS_{t}", "GND", note="enabled when the controller is off")
        R(100, syncs[t], f"{t}_SYNC")
        R(46.4e3, f"{t}_FREQ", "GND", note="free-running 1.0 MHz if SYNC is lost")
        C("47n", f"{t}_SS", "GND")
        R(10e3, f"{t}_COMP", f"{t}_CZ", note="type-II compensation")
        C("10n", f"{t}_CZ", "GND")
        C("100p", f"{t}_COMP", "GND", size="0603")
        D("PMEG40T30ER-Q", "VTRK_RAW", f"{t}_SW", ref=f"D{9 + k}", note=f"{t} boost diode: 17.2 V reverse / 40 V = 43 %, 0.35 A / 3 A")
        R(129e3, "VTRK_RAW", f"{t}_FB", tol=0.1, note="FB output clamp 17.08 V nominal (16.70-17.46 V): above the CV loop; not an overcharge control")
        R(10e3, f"{t}_FB", "GND", tol=0.1)
        C("4.7u", "VTRK_RAW", "GND", 50, "1210", vop=16.8)
        C("4.7u", "VTRK_RAW", "GND", 50, "1210", vop=16.8)
        # MPPT loop: VIN divider vs set point (REF_C divider, trimmed by the EPS controller's SAC DAC)
        R(69.8e3, f"{t}_L", f"{t}_VDIV", note="VIN x 0.3013")
        R(30.1e3, f"{t}_VDIV", "GND")
        C("1n", f"{t}_VDIV", "GND")
        R(47.5e3, "REF_C", f"{t}_VSET", note="default MPP 4.40 V with the DAC high-Z; 3.3-5.4 V with the DAC")
        R(53.6e3, f"{t}_VSET", "GND")
        R(75.0e3, f"DAC_{t}", f"{t}_VSET")
        C("100n", f"{t}_VSET", "GND")
        R(100e3, f"{t}_VSET", f"{t}_INTM", note="integrator input")
        C("10n", f"{t}_MPPO", f"{t}_INTM", note="integrator 1 ms")
        R(1e3, f"{t}_MPPO", f"{t}_MPPD")
        D("BAT54", f"{t}_MPPD", f"{t}_COMP", note="MPPT loop pulls COMP down")
        D("BAT54", "CTRL_DN", f"{t}_COMP", note="CV/CC loops pull COMP down")
    # MPPT op-amps (VCTL domain)
    for ref, (a, b) in (("U42", ("T1", "T2")), ("U43", ("T3", "T4"))):
        ic(S_OPA2, ref, "TLV9002-Q1", "TLV9002QDGKRQ1", "TI", "VSSOP-8 (DGK)", "1.8-5.5 V RRIO, 1 MHz, AEC-Q100 grade 1",
           {"OUTA": f"{a}_MPPO", "INAM": f"{a}_INTM", "INAP": f"{a}_VDIV", "VM": "GND", "INBP": f"{b}_VDIV",
            "INBM": f"{b}_INTM", "OUTB": f"{b}_MPPO", "VP": "VCTL"}, notes="MPPT input-voltage regulation")
        C("100n", "VCTL", "GND")
    # combined array output: shunt (array monitor) and the EOL fuse F1
    RSH(0.020, "VTRK_RAW", "VTRK", "WSLP1206R0200FEA", "1206", 1.0, note="array output current (INA228 U28, INA186 U46)",
        stress="0.6 A^2 x 20 mOhm = 7 mW", ref="RS5")
    C("4.7u", "VTRK", "GND", 50, "1210", vop=16.8)
    D("SMBJ20A", "VTRK_RAW", "GND", ref="D13", note="array-bus TVS: 32.4 V clamp < 40 V switch rating")


# ================================================================================================= 3 charge control (CV, CC)
def charge_control():
    # CV: PACK_P divider vs REF_C, inverting integrator: output low when PACK_P > 16.40 V
    R(55.6e3, "PACK_P", "CV_DIV", tol=0.1, note="CV 16.40 V (4.10 V/cell); 0.25 mA only while the AFE FETs are on")
    R(10.0e3, "CV_DIV", "GND", tol=0.1)
    R(100e3, "CV_DIV", "CV_INTM")
    C("10n", "CV_OUT", "CV_INTM")
    R(1e3, "CV_OUT", "CV_OUTR")
    D("BAT54", "CV_OUTR", "CTRL_DN", note="CV loop")
    # CC: INA186A3 (x100, VCTL) on the gauge shunt -> 1.0 V/A; set point 0.394 V; inhibit pulls the set point to 0
    ic(S_INA186, "U44", "INA186A3-Q1", "INA186A3QDCKRQ1", "TI", "SC70-6 (DCK)", "40 V CM, x100, Vos 50 uV, AEC-Q100",
       {"REF": "GND", "GND": "GND", "VS": "VCTL", "INP": "BAT_F", "INM": "BAT_F3", "OUT": "ICHG_SNS"},
       stress="CM 16.8 V / 40 V = 42 %", notes="charge current for the CC loop (positive = charging)", finish="NiPdAu|Sn")
    C("100n", "VCTL", "GND")
    R(21.5e3, "REF_C", "CC_SET", note="CC 0.394 A")
    R(4.02e3, "CC_SET", "GND")
    C("100n", "CC_SET", "GND")
    Q("2N7002BK", "Q18", "CHG_INHIBIT", "GND", "CC_SET", note="charge inhibit: CC set point to 0 V")
    R(100e3, "ICHG_SNS", "CC_INTM")
    C("10n", "CC_OUT", "CC_INTM")
    R(1e3, "CC_OUT", "CC_OUTR")
    D("BAT54", "CC_OUTR", "CTRL_DN", note="CC loop")
    R(100e3, "VCTL", "CTRL_DN", note="CTRL_DN idles high")
    ic(S_OPA2, "U45", "TLV9002-Q1", "TLV9002QDGKRQ1", "TI", "VSSOP-8 (DGK)", "1.8-5.5 V RRIO, AEC-Q100 grade 1",
       {"OUTA": "CV_OUT", "INAM": "CV_INTM", "INAP": "REF_C", "VM": "GND", "INBP": "CC_SET", "INBM": "CC_INTM",
        "OUTB": "CC_OUT", "VP": "VCTL"}, notes="A: CV loop; B: CC loop")
    C("100n", "VCTL", "GND")


# ================================================================================================= 4 pack, AFE, protection
def pack_and_protection():
    # series pack line: BAT_TOP - F2 - RT1 - F3 (secondary-protector fuse) - gauge shunt - AFE FETs - PACK_P
    p = place(TF, "F2", "5A", "FM12A72V5.0A", "AEM / MIL-PRF-23419/12 QPL", "FM12 solid-body SMD", "5 A, 72 VDC, interrupt 1000 A at 72 VDC (MIL-PRF-23419/12), cold 13.5-22.5 mOhm",
              "1.40 A / (5 A x 0.80 at 60 C, assumed) = 35 %; 2.0 A fault case 50 %", "pack fuse, non-resettable; same part as F0 on the battery sense board",
              "per MIL-PRF-23419/12 (Au or SnPb terminations: order the non-tin finish)", "NASA NEPP NPSL list: FM12A72V5.0A (cold resistance); interrupt rating from the slash sheet")
    conn(p, {"A": "BAT_TOP", "B": "N_F2"})
    p = place(TPTC, "RT1", "2.6A", "2920L260/33DR", "Littelfuse", "2920", "PPTC hold 2.6 A, trip 5.0 A, 33 V, 40 A",
              "1.40 A / 1.92 A hold at 60 C = 73 %", "pack PTC", "Sn; coat", "Digi-Key: Active")
    conn(p, {"A": "N_F2", "B": "N_RT1"})
    p = place(S_ITV, "F3", "ITV 12A", "ITV4030L1412NR", "Littelfuse", "4030 (3-terminal SMD)",
              "12 A, 36 VDC, 50 A break, heater 11.2-20 Ohm, triggerable 10.5-19.6 V",
              "1.31 A / 10 A at 60 C = 13 %; operating temp -10..+65 C (qualification item)", "secondary over-voltage fuse",
              "Sn; coat", "Digi-Key: Active")
    conn(p, {"T1": "N_RT1", "HTR": "F3_HTR", "T3": "BAT_F3"})
    RSH(0.010, "BAT_F3", "BAT_F", "WSLP1206R0100FEA", "1206", 1.0, note="gauge / IBATT / CC shunt (high side)",
        stress="1.31 A^2 x 10 mOhm = 17 mW", ref="RS6")
    # AFE FETs (high side, common drain): CHG source at BAT_F, DSG source at PACK_P
    Q("BUK7Y3R5-40H", "Q1", "CHG_G", "BAT_F", "FET_CD", stress="Vds 17.4 V/40 V = 44 %; Vgs 11 V/20 V = 55 %", note="AFE CHG FET")
    Q("BUK7Y3R5-40H", "Q2", "DSG_G", "PACK_P", "FET_CD", stress="Vds 17.4 V/40 V = 44 %; Vgs 11 V/20 V = 55 %", note="AFE DSG FET")
    R(100, "AFE_CHG", "CHG_G")
    R(10e6, "CHG_G", "BAT_F")
    R(100, "AFE_DSG", "DSG_G")
    R(10e6, "DSG_G", "PACK_P")
    # precharge (P-FET across the CHG FET, 100 Ohm 1 W)
    Q("PMV250EPEA", "Q3", "PCHG_G", "FET_CD", "PCHG_D", stress="Vgs 9.7 V/20 V = 49 %", note="precharge below 3.0 V/cell")
    R(1e3, "AFE_PCHG", "PCHG_G")
    R(1e6, "PCHG_G", "FET_CD")
    R(200, "PCHG_D", "BAT_F", size="2512", note="precharge <= 45 mA", stress="(16.7-8.0 V)^2/200 = 0.38 W of 1 W: 38 %")
    D("SMBJ20A", "PACK_P", "BAT_BOT", ref="D14", note="pack TVS (PACK_P to pack negative)")
    C("4.7u", "PACK_P", "PACKP_CM", 50, "1210", vop=16.8, note="series pair (fail-safe, RID SMA-B15)")
    C("4.7u", "PACKP_CM", "BAT_BOT", 50, "1210", vop=16.8, note="series pair (fail-safe, RID SMA-B15)")
    # low side: cell 1 negative - AFE sense resistor - PACK_N
    RSH(0.010, "BAT_BOT", "PACK_N", "WSLP1206R0100FEA", "1206", 1.0, note="AFE coulomb-counter / OCD / SCD shunt",
        stress="4.0 A^2 x 10 mOhm = 0.16 W at the OCD trip", ref="RS7")
    R(100, "BAT_BOT", "AFE_SRP")
    R(100, "PACK_N", "AFE_SRN")
    C("100n", "AFE_SRP", "AFE_SRN")
    C("100n", "AFE_SRP", "BAT_BOT")
    C("100n", "AFE_SRN", "BAT_BOT")
    # cell taps -> AFE: the current-limiting tap resistors (3 x 68.1 Ohm 2512 per tap, with a 0.75 A fuse) are on the
    # battery sense board (BATTERY_DESIGN 7.2: a short in the W1 tap wires is limited there); 10 Ohm + 0.1 uF here
    # (local damping); internal balancing ~9 mA through the 2 x 214 Ohm loop
    for tap, vc in (("TAP0", "VC0"), ("TAP1", "VC1"), ("TAP2", "VC2"), ("TAP3", "VC15"), ("TAP4", "VC16")):
        R(10, tap, f"AFE_{vc}", note="cell-tap local filter (204 Ohm limiting chain on the sense board)")
    # 1 M ladder across the taps (RID SMA-B03): an open tap is held between its neighbours, so neither the AFE nor
    # the secondary protector reads a false over- or under-voltage (4.2 uA from the pack at 16.8 V)
    for a_, b_ in (("TAP0", "TAP1"), ("TAP1", "TAP2"), ("TAP2", "TAP3"), ("TAP3", "TAP4")):
        R(1e6, b_, a_, note="open-tap ladder")
    C("100n", "AFE_VC0", "BAT_BOT")
    C("100n", "AFE_VC1", "AFE_VC0")
    C("100n", "AFE_VC2", "AFE_VC1")
    C("100n", "AFE_VC15", "AFE_VC2")
    C("100n", "AFE_VC16", "AFE_VC15")
    afe = {f"VC{n}": "AFE_VC2" for n in range(3, 15)}
    afe.update({"VC0": "AFE_VC0", "VC1": "AFE_VC1", "VC2": "AFE_VC2", "VC15": "AFE_VC15", "VC16": "AFE_VC16",
                "VSS": "BAT_BOT", "SRP": "AFE_SRP", "SRN": "AFE_SRN", "NC19": None, "NC44": None, "TS1": "NTC2",
                "TS2": "AFE_TS2", "TS3": "AFE_TS3", "REG18": "REG18", "ALERT": "AFE_ALERT", "SCL": "AFE_SCL",
                "SDA": "AFE_SDA", "HDQ": "BAT_BOT", "CFETOFF": "AFE_CFETOFF", "DFETOFF": "AFE_DFETOFF", "DCHG": None,
                "DDSG": None, "RST_SHUT": "AFE_RSTSHUT", "REG2": None, "REG1": "REG1", "REGIN": "AFE_REGIN",
                "BREG": "AFE_BREG", "FUSE": "AFE_FUSE", "PDSG": None, "PCHG": "AFE_PCHG", "LD": "AFE_LD",
                "PACK": "AFE_PACK", "DSG": "AFE_DSG", "CHG": "AFE_CHG", "CP1": "AFE_CP1", "BAT": "AFE_BATP"})
    ic(S_BQ76952, "U1", "BQ76952", "BQ76952PFBR", "TI", "TQFP-48 (PFB)",
       "3-16S AFE, high-side NFET drivers, coulomb counter, OTP config, -40..85 C", afe,
       stress="BAT 17.4 V / 85 V max = 20 %", notes="primary BMS (OTP: COV 4.25 V, CUV 2.80/3.00 V, OCD1 40 mV 500 ms, SCD 100 mV, OTC 45 C, UTC 0 C, OTD 60 C, SUV PF 2.00 V 5 s, PCHG below 3.0 V)")
    C("2.2u", "REG18", "BAT_BOT", 25, "0805")
    C("2.2u", "REG1", "BAT_BOT", 25, "0805")
    C("2.2u", "AFE_CP1", "AFE_BATP", 25, "0805")
    D("BAT54", "AFE_BATP", "BAT_F", note="BAT pin series diode (TI recommendation for a pack short)")
    C("4.7u", "AFE_BATP", "BATP_CM", 50, "1210", vop=16.8, note="series pair, 2.35 uF (fail-safe, RID SMA-B15)")
    C("4.7u", "BATP_CM", "BAT_BOT", 50, "1210", vop=16.8, note="series pair (fail-safe, RID SMA-B15)")
    p = place(TNPN, "Q8", "BCX56-16", "BCX56-16-QX", "Nexperia", "SOT-89", "NPN 80 V 1 A, AEC-Q101", "Vce 17.4 V/80 V = 22 %",
              "REGIN pre-regulator", "Sn", "Digi-Key: Active")
    conn(p, {"B": "AFE_BREG", "C": "AFE_BJTC", "E": "AFE_REGIN"})
    D("BAT54", "AFE_BJTC", "BAT_F", note="collector diode (TI recommendation)")
    C("2.2u", "AFE_REGIN", "BAT_BOT", 25, "0805")
    R(10e3, "PACK_P", "AFE_LD", note="load / charger detect (LD wake)")
    R(10e3, "PACK_P", "AFE_PACK")
    p = place(TNTC, "RT2", "10k NTC", "NCP15XH103F03RC", "Murata", "0402", "10 kOhm 1 %, B 3380 K", "", "AFE TS3: FET temperature",
              "Sn", "not checked")
    conn(p, {"A": "AFE_TS3", "B": "BAT_BOT"})
    # NTC2 (pack) on TS1: returns to TAP0 in the pack; FUSE sense pull-down; ALERT and I2C to the GND domain
    R(100e3, "AFE_FUSE", "BAT_BOT", note="FUSE idle low")
    R(1e6, "AFE_CFETOFF", "BAT_BOT")
    R(1e6, "AFE_DFETOFF", "BAT_BOT")
    R(100e3, "CHG_INHIBIT", "AFE_CFETOFF", note="cross-domain: series 100 k (CHG FET off when inhibited)")
    R(100e3, "OT60", "AFE_DFETOFF", note="cross-domain: series 100 k (BOTHOFF above +60 C)")
    R(100e3, "AFE_RSTSHUT", "BAT_BOT")
    R(10e3, "UMB_SHUT", "AFE_RSTSHUT", note="GSE holds high > 1 s to enter SHUTDOWN (pack domain: umbilical return is PACK_N)")
    R(100e3, "BAT_F", "UMB_VMON", note="umbilical pack-voltage monitor (100 k series)")
    R(10e3, "REG1", "AFE_SCL")
    R(10e3, "REG1", "AFE_SDA")
    R(100e3, "REG1", "AFE_ALERT")
    ic(S_ISO1540, "U50", "ISO1540-Q1", "ISO1540QDRQ1", "TI", "SOIC-8 (D)", "bidirectional I2C isolator, AEC-Q100 grade 1",
       {"VCC1": "REG1", "SDA1": "AFE_SDA", "SCL1": "AFE_SCL", "GND1": "BAT_BOT", "GND2": "GND", "SCL2": "LSCL",
        "SDA2": "LSDA", "VCC2": "3V3_MCU"}, notes="AFE I2C to the EPS controller across the PACK/GND domain boundary")
    C("100n", "REG1", "BAT_BOT")
    C("100n", "3V3_MCU", "GND")
    R(100e3, "AFE_ALERT", "AFE_ALERT_X", note="cross-domain series")
    R(1e6, "AFE_ALERT_X", "GND")
    # secondary over-voltage protector (own taps, own filters): BQ771806 4.35 V, 3 s; OUT -> divider -> FET -> F3 heater
    for tap, pin in (("TAP1", "V1"), ("TAP2", "V2"), ("TAP3", "V3"), ("TAP4", "V4")):
        R(1e3, tap, f"OVP_{pin}", note="independent tap filter")
        C("100n", f"OVP_{pin}", "BAT_BOT")
    R(100, "N_RT1", "OVP_VDD", note="protector supply ahead of F3")
    C("1u", "OVP_VDD", "OVPV_CM", 50, "0805", note="series pair, 0.5 uF (fail-safe, RID SMA-B15)")
    C("1u", "OVPV_CM", "BAT_BOT", 50, "0805", note="series pair (fail-safe, RID SMA-B15)")
    ic(S_BQ7718, "U2", "BQ771806", "BQ771806DPJR", "TI", "WSON-8 (DPJ)", "2-5S OV protector, 4.350 V +/-10 mV, 3 s, CMOS active high",
       {"VDD": "OVP_VDD", "V5": "OVP_V4", "V4": "OVP_V4", "V3": "OVP_V3", "V2": "OVP_V2", "V1": "OVP_V1",
        "VSS": "BAT_BOT", "OUT": "OVP_OUT"}, stress="VDD 17.4 V / 25 V = 70 %", notes="secondary OV (V5 tied to V4 for 4S)",
       finish="NiPdAu|Sn")
    R(47e3, "OVP_OUT", "OVP_G", note="gate divider: 17.4 V -> 11.8 V")
    R(100e3, "OVP_G", "BAT_BOT")
    Q("PMV40UN2", "Q16", "OVP_G", "BAT_BOT", "F3_HTR", stress="Vds 17.4 V/30 V = 58 %; Vgs <= 8.2 V (D22) / 12 V = 68 %",
      note="F3 heater driver; gate divider and 8.2 V zener D22")
    p = place(TD, "D22", "BZX84-C8V2", "BZX84-C8V2,215", "Nexperia", "SOT-23", "Zener 8.2 V 250 mW", "0.2 mA", "Q16 Vgs clamp",
              "Sn", "not checked")
    conn(p, {"K": "OVP_G", "A": "BAT_BOT"})
    # wake: DS2 released -> Q23 pulls TS2 to VSS
    Q("2N7002BK", "Q23", "WAKE_G", "BAT_BOT", "AFE_TS2", note="AFE wake from SHUTDOWN on DS2 release")
    R(1e6, "S2R", "WAKE_G")
    R(1e6, "WAKE_G", "BAT_BOT")


# ================================================================================================= 5 inhibits, bus, umbilical
def inhibits():
    # DS1: P-FET, source PACK_P, drain VB1; gate divider to PACK_N through the DS1 NC contact (harness via 10 k)
    Q("IPD50P04P4L-11", "Q4", "G1", "PACK_P", "VB1", stress="Vds 17.4 V/40 V = 44 %; Vgs -8.4 V/-16 V = 53 %", note="DS1 inhibit FET")
    R(1e6, "G1", "PACK_P")
    C("47n", "G1", "PACK_P")
    R(1e6, "G1", "S1N")
    R(10e3, "PACK_N", "DS1_C", note="harness short protection")
    # DS3: P-FET, source VB1, drain VB3; gate divider to GND through the DS3 NC contact
    Q("IPD50P04P4L-11", "Q5", "G3", "VB1", "VB3", stress="Vds 17.4 V/40 V = 44 %; Vgs -8.4 V/-16 V = 53 %", note="DS3 inhibit FET")
    R(1e6, "G3", "VB1")
    C("47n", "G3", "VB1")
    R(1e6, "G3", "S3N")
    # DS2: back-to-back N-FETs PACK_N - CS2 - GND; gate from BAT_F through 10 k, DS2 NC, RBF pole A, 1.2 M
    Q("BUK9Y4R4-40E", "Q6", "G2", "CS2", "PACK_N", stress="Vgs 7.3 V/10 V = 73 %", note="DS2 inhibit FET A")
    Q("BUK9Y4R4-40E", "Q7", "G2", "CS2", "GND", stress="Vgs 7.3 V/10 V = 73 %", note="DS2 inhibit FET B (back to back)")
    # DS2 drive from BAT_F OR PACK_P, so a pack-path open (F0A/F0B/F0N/F2/F3) leaves the array able to power the bus
    p = place(TDDUAL_CC, None, "BAT54C", "BAT54C,215", "Nexperia", "SOT-23", "30 V 200 mA dual Schottky, common cathode", "",
              "DS2 drive OR (BAT_F, PACK_P); RID SMA-B01", "Sn", "not checked")
    conn(p, {"A1": "BAT_F", "A2": "PACK_P", "K": "DS2_SRC"})
    R(10e3, "DS2_SRC", "DS2_C", note="harness short protection")
    R(1.3e6, "S2R", "G2")
    R(1.0e6, "G2", "CS2")
    C("47n", "G2", "CS2")
    # DEPLOY_SENSE (OBC PE2): DS2 released, 1 M series and clamp
    R(1e6, "S2R", "DEPLOY_SENSE", note="cross-domain series 1 M")
    R(1e6, "DEPLOY_SENSE", "GND")
    p = place(TDDUAL_S, "D23", "BAT54S", "BAT54S,215", "Nexperia", "SOT-23", "30 V dual series Schottky", "", "DEPLOY_SENSE clamp",
              "Sn", "not checked")
    conn(p, {"A": "GND", "K": "3V3_AO", "CA": "DEPLOY_SENSE"})
    # bus filter and shunt
    RSH(0.010, "VB3", "VBAT", "WSLP1206R0100FEA", "1206", 1.0, note="bus load current (INA3221)", stress="17 mW", ref="RS8")
    C("4.7u", "VBAT", "GND", 50, "1210", vop=16.8)
    C("4.7u", "VBAT", "GND", 50, "1210", vop=16.8)
    C("4.7u", "VBAT", "GND", 50, "1210", vop=16.8)
    C("4.7u", "VBAT", "GND", 50, "1210", vop=16.8)
    D("SMBJ20A", "VBAT", "GND", ref="D15", note="bus TVS")
    TP("VBAT")
    TP("PACK_P")
    TP("GND")
    # VBATT_SNS 1:6 (OBC PC2)
    R(1.0e6, "PACK_P", "VBATT_SNS", tol=0.1, note="VBATT_SNS 1:6 (adc.rs Linear(6.0))")
    R(200e3, "VBATT_SNS", "GND", tol=0.1)
    C("100n", "VBATT_SNS", "GND")
    # umbilical charge input: fuse, Schottky, to PACK_P (return PACK_N on J13)
    p = place(TF, "F5", "1A", "0466001.NR", "Littelfuse", "1206 (Pico 466)", "1 A, 63 V, fast", "0.5 A GSE charge: 50 %",
              "umbilical charge fuse", "Sn; coat", "not checked")
    conn(p, {"A": "UMB_CHG", "B": "UMB_F"})
    D("PMEG40T30ER-Q", "PACK_P", "UMB_F", ref="D16", note="umbilical charge diode (no back-feed to the GSE)")


# ================================================================================================= 6 EOL disconnect F1
def eol():
    p = place(S_ITV, "F1", "ITV 12A", "ITV4030L1412NR", "Littelfuse", "4030 (3-terminal SMD)",
              "12 A, 36 VDC, heater 11.2-20 Ohm, triggerable 10.5-19.6 V", "0.6 A array: 6 %",
              "EOL permanent charge disconnect (97.207(g)(1)(iii)): array side T1, pack side T3", "Sn; coat", "Digi-Key: Active")
    conn(p, {"T1": "VTRK", "HTR": "F1_HTR", "T3": "PACK_P"})
    # arm latch: EOL_ARM -> RC -> Schmitt -> FF clock (D high); cleared at power-up
    R(10e3, "EOL_ARM", "EOL_ARM_F")
    C("100n", "EOL_ARM_F", "GND")
    R(100e3, "EOL_ARM_F", "GND")
    ic(S_SCH, "U60", "SN74LVC1G17-Q1", "SN74LVC1G17QDCKRQ1", "TI", "SC70-5 (DCK)", "Schmitt buffer",
       {"NC": None, "A": "EOL_ARM_F", "GND": "GND", "Y": "EOL_ARM_CK", "VCC": "3V3_AO"}, finish="NiPdAu|Sn")
    C("100n", "3V3_AO", "GND")
    # arm needs two independent sources: the OBC's EOL_ARM line AND the EPS controller's EOL_KEY, which the controller
    # raises only after a keyed two-write I2C sequence (ICD extended registers); fire is the OBC's EOL_FIRE (RID P3E-11)
    ic(S_G1, "U64", "SN74LVC1G08-Q1", "SN74LVC1G08QDCKRQ1", "TI", "SC70-5 (DCK)", "AND",
       {"A": "EOL_ARM_CK", "B": "EOL_KEY", "GND": "GND", "Y": "EOL_ARM_G", "VCC": "3V3_AO"}, finish="NiPdAu|Sn",
       notes="EOL arm: OBC EOL_ARM AND EPS-controller EOL_KEY")
    C("100n", "3V3_AO", "GND")
    R(100e3, "EOL_KEY", "GND", note="low at controller reset or power loss")
    ic(S_FF, "U61", "SN74LVC2G74-Q1", "SN74LVC2G74QDCURQ1", "TI", "VSSOP-8 (DCU)", "D flip-flop with preset/clear",
       {"CLK": "EOL_ARM_G", "D": "3V3_AO", "QN": None, "GND": "GND", "Q": "EOL_ARMED", "CLRN": "EOL_POR",
        "PREN": "3V3_AO", "VCC": "3V3_AO"}, notes="EOL arm latch, cleared at power-up")
    C("100n", "3V3_AO", "GND")
    R(1e6, "3V3_AO", "EOL_POR")
    C("1u", "EOL_POR", "GND", 25)
    R(100, "EOL_ARMED", "ARM_G")
    R(100e3, "ARM_G", "GND")
    R(10e3, "EOL_FIRE", "FIRE_F")
    C("100n", "FIRE_F", "GND")
    R(100e3, "FIRE_F", "GND")
    Q("PMV40UN2", "Q20", "FIRE_F", "EOL_MID", "F1_HTR", stress="Vds 16.8 V/30 V = 56 %; 1.46 A pulse / 3.7 A",
      note="EOL fire switch (EOL_FIRE)")
    Q("PMV40UN2", "Q21", "ARM_G", "GND", "EOL_MID", stress="1.46 A pulse / 3.7 A", note="EOL arm switch (latched EOL_ARM)")
    R(1e6, "F1_HTR", "F1_SNS", note="F1 intact sense (PACK_P via the heater)")
    R(220e3, "F1_SNS", "GND")
    C("100n", "F1_SNS", "GND")


# ================================================================================================= 7 rails and load switches
def rails_and_switches():
    def buck(ref, mpn, out, vin, en_top, fb_net, sync, note, outcap=2, rating=""):
        ic(S_LMR436, ref, "LMR43620-Q1", mpn, "TI", "VQFN-HR-9 (RPE) 2x2.5",
           "3-36 V in (42 V abs), 2 A, sync 0.2-2.5 MHz, no spread spectrum (MC variant), AEC-Q100",
           {"SYNC": f"{ref}_SYNC", "PGOOD": f"{ref}_PG", "EN": f"{ref}_EN", "VIN": vin, "SW": f"{ref}_SW",
            "BOOT": f"{ref}_BOOT", "VCC": f"{ref}_VCC", "FB": fb_net, "GND": "GND"},
           stress="VIN 16.8 V / 42 V = 40 %", notes=note, finish="Sn (matte): re-tin not possible (QFN-HR): conformal coat")
        R(100, "SYNC_A", f"{ref}_SYNC")
        R(en_top, vin, f"{ref}_EN", note="UVLO divider")
        R(100e3, f"{ref}_EN", "GND")
        C("100n", f"{ref}_BOOT", f"{ref}_SW")
        C("1u", f"{ref}_VCC", "GND", 25)
        R(100e3, f"{ref}_VCC", f"{ref}_PG")
        C("4.7u", vin, "GND", 50, "1210", vop=16.8)
        C("100n", vin, "GND")
        p = place(TL, None, "6.8uH", "XGL4030-682MEC", "Coilcraft", "4.0 x 4.0 x 3.1 mm", "6.8 uH, Isat 3.65 A, AEC-Q200",
                  "", f"{ref} inductor", "Sn over Ag; coat", "Digi-Key: Active")
        conn(p, {"A": f"{ref}_SW", "B": out})
        for _ in range(outcap):
            C("22u", out, "GND", 16, "1210", vop=7.1)

    # 3V3_AO buck (output shunt for the INA3221 between the buck and the rail; FB at the rail)
    buck("U15", "LMR43620MC3RPERQ1", "3V3_BK", "VBAT", 750e3, "3V3_AO", "SYNC_A", "3V3_AO, always on")
    RSH(0.050, "3V3_BK", "3V3_AO", "WSLP0805R0500FEA", "0805", 0.25, note="3V3_AO current (INA3221)", stress="4.5 mW", ref="RS9")
    C("22u", "3V3_AO", "GND", 16, "1210", vop=3.3)
    TP("3V3_AO")
    buck("U16", "LMR43620MC5RPERQ1", "5V_BK", "VBAT", 887e3, "5V_PA", "SYNC_A", "5V_PA, the PA's own buck (EMC rule E3)")
    RSH(0.050, "5V_BK", "5V_PA", "WSLP0805R0500FEA", "0805", 0.25, note="5V_PA current", stress="15 mW", ref="RS10")
    C("22u", "5V_PA", "GND", 16, "1210", vop=5.0)
    R(100e3, "5V_PA", "VBUS_SNS", note="VBUS_SNS 1:2")
    R(100e3, "VBUS_SNS", "GND")
    C("100n", "VBUS_SNS", "GND")
    TP("5V_PA")
    # burn buck: VBAT_DEP (behind SW6) -> V_BURN 7.04 V (60.4 k / 10.0 k on FB)
    buck("U17", "LMR43620MC5RPERQ1", "V_BURN", "VBAT_DEP", 887e3, "U17_FB", "SYNC_A", "V_BURN 7.0 V burn supply (on only with DEPLOY_EN)")
    R(60.4e3, "V_BURN", "U17_FB")
    R(10.0e3, "U17_FB", "GND")
    # SW6 DEPLOY (VBAT high side)
    ic(S_TPS1H100, "U18", "TPS1H100B-Q1", "TPS1H100BQPWPRQ1", "TI", "HTSSOP-14 (PWP)", "40 V, 100 mOhm, adj. current limit, current sense",
       {"NC1": None, "GND": "GND", "IN": "DEPLOY_EN", "NC4": None, "OUT5": "VBAT_DEP", "OUT6": "VBAT_DEP", "OUT7": "VBAT_DEP",
        "VS8": "VBAT", "VS9": "VBAT", "VS10": "VBAT", "NC11": None, "DIAG_EN": "DIAG_EN", "CL": "SW6_CL", "CS": "SW6_CS",
        "PAD": "GND"}, stress="VS 16.8 V/40 V = 42 %", notes="SW6 DEPLOY_EN: burn buck supply (deployable control, not counted)")
    R(100e3, "DEPLOY_EN", "GND", note="pull-down: off at reset")
    R(1.24e3, "SW6_CL", "GND", note="current limit 2.0 A")
    R(470, "SW6_CS", "GND")
    R(10e3, "SW6_CS", "FLT_SW6")
    C("4.7u", "VBAT_DEP", "GND", 50, "1210", vop=16.8)
    R(10e3, "3V3_AO", "DIAG_EN")
    # SW1 PAYLOAD (VBAT) gated by the hardware load shed; U19 eFuse (JETSON_PWR)
    ic(S_G1, "U51", "SN74LVC1G08-Q1", "SN74LVC1G08QDCKRQ1", "TI", "SC70-5 (DCK)", "AND",
       {"A": "PAYLOAD_EN", "B": "SHED_N", "GND": "GND", "Y": "SW1_IN", "VCC": "3V3_AO"}, finish="NiPdAu|Sn",
       notes="load shed gate for SW1")
    C("100n", "3V3_AO", "GND")
    R(100e3, "PAYLOAD_EN", "GND")
    ic(S_TPS1H100, "U20", "TPS1H100B-Q1", "TPS1H100BQPWPRQ1", "TI", "HTSSOP-14 (PWP)", "40 V, 100 mOhm, adj. current limit",
       {"NC1": None, "GND": "GND", "IN": "SW1_IN", "NC4": None, "OUT5": "VBAT_SW1", "OUT6": "VBAT_SW1", "OUT7": "VBAT_SW1",
        "VS8": "VBAT", "VS9": "VBAT", "VS10": "VBAT", "NC11": None, "DIAG_EN": "DIAG_EN", "CL": "SW1_CL", "CS": "SW1_CS",
        "PAD": "GND"}, stress="VS 16.8/40 = 42 %; 0.93 A / 2.0 A", notes="SW1 PAYLOAD_EN (Jetson)")
    R(1.24e3, "SW1_CL", "GND", note="current limit 2.0 A")
    R(470, "SW1_CS", "GND")
    R(10e3, "SW1_CS", "FLT_SW1")
    C("4.7u", "VBAT_SW1", "GND", 50, "1210", vop=16.8)
    ic(S_TPS1663, "U19", "TPS16630", "TPS16630RGER", "TI", "VQFN-24 (RGE) 4x4", "4.5-60 V eFuse, 31 mOhm, 0.6-6 A +/-7 %, latch-off",
       {"IN1": "VBAT_SW1", "IN2": "VBAT_SW1", "NC3": None, "NC4": None, "P_IN": "VBAT_SW1", "UVLO": "EF_UVLO", "OVP": "EF_OVP",
        "GND": "GND", "DVDT": "EF_DVDT", "ILIM": "EF_ILIM", "MODE": None, "SHDN": "EF_SHDN", "IMON": "IMON_JET",
        "FLT": "FLT_EFUSE", "NC15": None, "PGOOD": None, "OUT17": "VBAT_JET", "OUT18": "VBAT_JET", "NC19": None, "NC20": None,
        "NC21": None, "NC22": None, "NC23": None, "NC24": None, "PAD": "GND"},
       stress="VIN 16.8 V/60 V = 28 %", notes="Jetson eFuse U3 of EPS_DESIGN (MODE open = latch-off)", checked="TI addendum: Active (industrial, not AEC-Q100)")
    R(887e3, "VBAT_SW1", "EF_UVLO")
    R(100e3, "EF_UVLO", "GND")
    R(1.43e6, "VBAT_SW1", "EF_OVP", note="OVP 18.4 V")
    R(100e3, "EF_OVP", "GND")
    C("10n", "EF_DVDT", "GND", note="output slew")
    R(15.0e3, "EF_ILIM", "GND", note="1.20 A")
    R(10e3, "3V3_MCU", "FLT_EFUSE")
    R(30.1e3, "IMON_JET", "GND", note="IMON 0.84 V/A")
    C("1n", "IMON_JET", "GND")
    # Jetson readbacks to the OBC (RID AV-06, SQSO-ICX-001 E1/E2): VBAT / 6 after SW1 and after the eFuse
    for net, rb in (("VBAT_SW1", "VSW1_RB"), ("VBAT_JET", "VJET_RB")):
        R(1.00e6, net, rb, tol=0.1, note=f"{rb} 1:6 (OBC ADC, J3)")
        R(200e3, rb, "GND", tol=0.1)
        C("100n", rb, "GND")
    # the hardware load shed also removes the eFuse's enable, so it covers SW1 and U19 each failed closed (RID P3E-06)
    ic(S_G1, "U65", "SN74LVC1G08-Q1", "SN74LVC1G08QDCKRQ1", "TI", "SC70-5 (DCK)", "AND",
       {"A": "JETSON_PWR", "B": "SHED_N", "GND": "GND", "Y": "JET_EN_G", "VCC": "3V3_AO"}, finish="NiPdAu|Sn",
       notes="load shed gate for the Jetson eFuse U19")
    C("100n", "3V3_AO", "GND")
    R(100e3, "JETSON_PWR", "GND", note="pull-down: off at reset")
    R(10e3, "JET_EN_G", "EF_SHDN")
    R(100e3, "EF_SHDN", "GND")
    C("4.7u", "VBAT_JET", "GND", 50, "1210", vop=16.8)
    # SW5 HEATER (TPS1H000, 0.148 A) -> F4 -> shunt -> J1 HTR+; HTR- -> Q22 (40 C cutout) -> GND
    ic(S_TPS1H000, "U21", "TPS1H000-Q1", "TPS1H000AQDGNRQ1", "TI", "HVSSOP-8 (DGN)", "40 V, 1 Ohm, adj. limit 0.05-1 A",
       {"IN": "HEATER_EN", "DIAG_EN": "DIAG_EN", "FAULT": "FLT_SW5", "CL": "SW5_CL", "DELAY": None, "GND": "GND",
        "OUT": "HTR_SW", "VS": "VBAT", "PAD": "GND"}, stress="VS 16.8/40 = 42 %", notes="SW5 heater", finish="NiPdAu|NiPdAuAg")
    R(100e3, "HEATER_EN", "GND")
    R(3.24e3, "SW5_CL", "GND", note="0.148 A limit")
    R(10e3, "3V3_AO", "FLT_SW5")
    p = place(TF, "F4", "250mA", "0466.250NR", "Littelfuse", "1206 (Pico 466)", "0.25 A, 125 V", "79 mA: 32 %", "heater fuse",
              "Sn; coat", "Digi-Key listing")
    conn(p, {"A": "HTR_SW", "B": "HTR_F"})
    RSH(0.5, "HTR_F", "HTR_P", "WSL0805R5000FEA", "0805", 0.125, note="heater current (INA3221)", stress="3 mW", ref="RS11")
    Q("PMV40UN2", "Q22", "HCUT_EFF", "GND", "HTR_N", stress="Vds 16.8/30 = 56 %; 79 mA", note="heater +40 C hardware cutout (series, low side)")
    # SW11 magnetorquer bridge supply (SQSO-ICX-001 E3): VBAT -> J3-36/39, enable ADCS_EN, limit 0.361 A (0.325-0.397 A),
    # auto-retry (DELAY pulled up: 40 ms on / 1 s off in a sustained overload); diagnostics only while enabled
    ic(S_TPS1H000, "U39", "TPS1H000-Q1", "TPS1H000AQDGNRQ1", "TI", "HVSSOP-8 (DGN)", "40 V, 1 Ohm, adj. limit 0.05-1 A",
       {"IN": "SW11_IN", "DIAG_EN": "SW11_IN", "FAULT": "FLT_SW11", "CL": "SW11_CL", "DELAY": "SW11_DLY", "GND": "GND",
        "OUT": "VBAT_MTQ", "VS": "VBAT", "PAD": "GND"}, stress="VS 16.8/40 = 42 %; 0.23 A / 0.36 A limit",
       notes="SW11 VBAT_MTQ (OBC magnetorquer H-bridges, ADCS_EN)", finish="NiPdAu|NiPdAuAg")
    R(10e3, "ADCS_EN", "SW11_IN", note="series input resistor (datasheet R_SER)")
    R(1.33e3, "SW11_CL", "GND", note="0.361 A limit (0.8 V x 600 / 1.33 k)")
    R(1.0e3, "3V3_AO", "SW11_DLY", note="DELAY pulled up: auto-retry mode")
    R(10e3, "3V3_AO", "FLT_SW11")
    C("1u", "VBAT_MTQ", "GND", 50, "0805", vop=16.8)
    C("100n", "VBAT_MTQ", "GND")
    # 3V3_RTC: the OBC's mission clock on the always-on rail (SQSO-ICX-001 E4, RID AV-10)
    R(1.0e3, "3V3_AO", "3V3_RTC", note="3V3_RTC: <= 2 uA load; 3.4 mA into a short")
    # 3V3 switched rails (TPS2553 with auto-retry: EN driven through 100 k, FAULT pulls EN down, C_RETRY 1 uF)
    # (ref, switch, enable source, input rail, switch output, R_ILIM, shunt ref, rail after the shunt, shunt, C_RETRY)
    # trips against RADIATION_ANALYSIS Rev C 5.3 (RID AV-08): limit in microseconds, FAULT after 8 ms removes the rail,
    # off time 0.6 x R_FAULT x C_RETRY (EN threshold), fast turn-off through the BAT54 when the enable goes low
    sws = [("U22", "SW2", "RADIO_EN", "3V3_AO", "3V3_RADIO_U", 226e3, "RS12", "3V3_RADIO", 0.100, "22u"),
           ("U23", "SW3", "RF_TX_EN", "5V_PA", "5V_PA_U", 26.1e3, "RS13", "5V_PA_SW", 0.050, "4.7u"),
           ("U24", "SW4", "ADCS_EN", "3V3_AO", "3V3_ADCS_U", 88.7e3, "RS14", "3V3_ADCS", 0.100, "4.7u"),
           ("U25", "SW7", "GNSS_EN", "3V3_AO", "3V3_GNSS_U", 133e3, "RS15", "3V3_GNSS", 0.100, "22u"),
           ("U26", "SW8", "ANTS_EN", "3V3_AO", "3V3_ANTS_U", 18.7e3, "RS16", "3V3_ANTS", 0.050, "4.7u"),
           ("U27", "SW9", "3V3_AO", "3V3_AO", "3V3_MCU", None, None, None, None, "4.7u"),
           ("U38", "SW10", "3V3_AO", "3V3_AO", "3V3_OBC", 178e3, None, None, None, "4.7u")]
    for ref, sw, en, vin, vout, rlim, rsref, rail, rsh, cret in sws:
        en_node = f"{sw}_EN"
        what = {"SW9": "EPS controller supply", "SW10": "OBC supply (the OBC cannot switch its own rail)"}.get(sw, sw)
        ic(S_TPS2553, ref, "TPS2553-Q1", "TPS2553QDBVRQ1", "TI", "SOT-23-6 (DBV)", "2.5-6.5 V, 85 mOhm, adj. 75 mA-1.3 A",
           {"IN": vin, "GND": "GND", "EN": en_node, "FAULT": en_node, "ILIM": f"{sw}_ILIM" if rlim else vin, "OUT": vout},
           stress=f"IN {('5.0' if vin == '5V_PA' else '3.3')} V/6.5 V = {(5.0 if vin == '5V_PA' else 3.3) / 6.5:.0%}",
           notes=f"{what}: current limit, latch-up trip and auto-retry (FAULT pulls EN low; off 0.6 x 100 k x {cret}F)")
        R(100e3, en, en_node, note="R_FAULT (retry, TPS2553 data sheet figure 28)")
        if cret == "22u":
            C("22u", en_node, "GND", 16, "1210", note="C_RETRY: >= 1 s off after a trip", vop=3.3)
        else:
            C("4.7u", en_node, "GND", 25, "0805", note="C_RETRY: >= 0.2 s off after a trip", vop=3.3)
        if en not in ("3V3_AO",):
            D("BAT54", en, en_node, note="fast turn-off when the enable goes low")
            R(100e3, en_node, f"FLT_{sw}", note="fault readback: EN node low while the enable is high")
        if rlim:
            R(rlim, f"{sw}_ILIM", "GND")
        C("1u", vout, "GND", 25)
        if rsh:
            RSH(rsh, vout, rail, f"WSLP0805R{int(rsh * 1000):03d}0FEA", "0805", 0.25, note=f"{rail} current (INA3221)", ref=rsref)
        if en not in ("3V3_AO", "RF_TX_EN", "ANTS_EN"):
            R(100e3, en, "GND", note="pull-down: off at reset")
    # SW3 enable = RF_TX_EN only (hardware TX-OFF); the PA is not shed (RID SMA-B05: a spurious shed would hold the
    # transmitter off with no override; a stuck PA is energy-positive, SQSO-PBA-001 8.4)
    R(100e3, "RF_TX_EN", "GND", note="mandatory pull-down (hardware TX-OFF)")
    # SW8 enable = DEPLOY_EN OR the EPS controller's ANTS_EN_MCU (RID SMA-B06); SW8 is downstream of DS1-DS3
    ic(S_G1, "U81", "SN74LVC1G32-Q1", "SN74LVC1G32QDCKRQ1", "TI", "SC70-5 (DCK)", "OR",
       {"A": "DEPLOY_EN", "B": "ANTS_EN_MCU", "GND": "GND", "Y": "ANTS_EN", "VCC": "3V3_AO"}, finish="NiPdAu|Sn",
       notes="AntS supply: OBC DEPLOY_EN or EPS-controller ANTS_PWR")
    C("100n", "3V3_AO", "GND")
    R(100e3, "ANTS_EN_MCU", "GND", note="off at controller reset")
    # OBC heartbeat supervision: the EPS controller power-cycles the OBC rail by pulling SW10's EN low (RID SMA-B09)
    Q("2N7002BK", "Q42", "OBC_CYCLE", "GND", "SW10_EN", note="OBC rail power cycle (EPS controller, heartbeat timeout)")
    R(100e3, "OBC_CYCLE", "GND", note="off at controller reset")
    C("22u", "3V3_MCU", "GND", 16, "1210", vop=3.3)


# ================================================================================================= 8 deployment drivers
def burn_drivers():
    ic(S_REF34, "U47", "REF3425-Q1", "REF3425QDBVRQ1", "TI", "SOT-23-6 (DBV)", "2.500 V +/-0.05 %, AEC-Q100",
       {"GND_F": "GND", "GND_S": "GND", "EN": "3V3_AO", "IN": "3V3_AO", "OUT_S": "REF_A", "OUT_F": "REF_A"},
       finish="NiPdAuAg", notes="3V3_AO-domain reference: burn set points, load shed")
    C("1u", "REF_A", "GND", 25)
    C("100n", "3V3_AO", "GND")
    for k, w in enumerate(("W1A", "W1B", "W2A", "W2B")):
        R(100, f"BURN_{w}", f"BXP_{w}", note="burn line to the one-shot supply")
        R(100e3, f"BURN_{w}", "GND", note="pull-down: off at reset")
        C("100n", f"BXP_{w}", "GND")
        ic(S_TPL5111, f"U{70 + k}", "TPL5111", "TPL5111DDCR", "TI", "SOT-23-6 (DDC)", "1.8-5.5 V nano timer, one-shot 12.0 s",
           {"VDD": f"BXP_{w}", "GND": "GND", "DELAY": f"TDLY_{w}", "DONE": "GND", "DRVN": f"TIMED_{w}", "ONESHOT": "GND"},
           notes=f"{w} 12 s hardware on-time; powered by the burn line, so only a falling edge re-arms it",
           checked="TI addendum: Active (industrial, not AEC-Q100)")
        R(11.8e3, f"TDLY_{w}", f"TDLY2_{w}", tol=0.1, note="R_EXT 11.98 k -> tIP 12.05 s")
        R(182, f"TDLY2_{w}", "GND", tol=0.1)
        R(100e3, f"TIMED_{w}", "GND")
        ic(S_G1, f"U{74 + k}", "SN74LVC1G08-Q1", "SN74LVC1G08QDCKRQ1", "TI", "SC70-5 (DCK)", "AND (VCC = REF_A 2.500 V)",
           {"A": f"TIMED_{w}", "B": f"BXP_{w}", "GND": "GND", "Y": f"BSET_{w}", "VCC": "REF_A"}, finish="NiPdAu|Sn",
           notes="burn set point = 2.500 V only while the line is high and the one-shot runs")
        R(11.5e3, f"BSET_{w}", f"VSET_{w}", tol=0.1)
        R(1.00e3, f"VSET_{w}", "GND", tol=0.1, note="0.200 V -> 0.800 A")
        Q("BUK9M14-40E", f"Q{24 + k}", f"BG_{w}", f"BSNS_{w}", f"BRET_{w}", stress="Vds 7.3 V/40 V; 3.0 W for 12 s (Tj 88 C)",
          note=f"{w} current-limited burn driver (linear)")
        R(100, f"BO_{w}", f"BG_{w}")
        R(100e3, f"BG_{w}", "GND")
        RSH(0.250, f"BSNS_{w}", "GND", "WSL2010R2500FEA", "2010", 0.5, note=f"{w} burn current sense",
            stress="0.88 A^2 x 0.25 = 0.19 W of 0.5 W (39 %), 12 s max", ref=f"RS{20 + k}")
        R(1e3, f"BSNS_{w}", f"BFB_{w}")
        C("1n", f"BO_{w}", f"BFB_{w}", note="op-amp compensation")
        R(10e3, f"BSNS_{w}", f"IBURN_{w}", note="to the EPS controller ADC")
        C("10n", f"IBURN_{w}", "GND")
    for ref, (a, b) in (("U78", ("W1A", "W1B")), ("U79", ("W2A", "W2B"))):
        ic(S_OPA2, ref, "OPA2991-Q1", "OPA2991QDGKRQ1", "TI", "VSSOP-8 (DGK)", "2.7-40 V RRIO, AEC-Q100 grade 1",
           {"OUTA": f"BO_{a}", "INAM": f"BFB_{a}", "INAP": f"VSET_{a}", "VM": "GND", "INBP": f"VSET_{b}", "INBM": f"BFB_{b}",
            "OUTB": f"BO_{b}", "VP": "V_BURN"}, stress="V+ 7.3 V / 40 V = 18 %", notes="burn current regulators (powered from V_BURN)")
        C("100n", "V_BURN", "GND")
    # wing hinge sense: both wings in series (W7), 10 k pull-up, RC filter to SOLAR_SENSE (OBC PB12)
    R(10e3, "3V3_AO", "SA_SENSE_PU")
    R(1e3, "SA_SENSE_PU", "SOLAR_SENSE")
    C("100n", "SOLAR_SENSE", "GND")
    # AntS I2C buffer (enabled only while the AntS is powered)
    ic(S_TCA9517, "U80", "TCA9517-Q1", "TCA9517DGKRQ1", "TI", "VSSOP-8 (DGK)", "I2C repeater, high-Z when unpowered, AEC-Q100",
       {"VCCA": "3V3_AO", "SCLA": "I2C2_SCL", "SDAA": "I2C2_SDA", "GND": "GND", "EN": "ANTS_EN", "SDAB": "ANTS_SDA",
        "SCLB": "ANTS_SCL", "VCCB": "3V3_ANTS"}, finish="NiPdAu|NiPdAuAg", notes="OBC I2C2 to the AntS")
    R(10e3, "3V3_ANTS", "ANTS_EN")
    R(4.7e3, "3V3_ANTS", "ANTS_SDA")
    R(4.7e3, "3V3_ANTS", "ANTS_SCL")
    C("100n", "3V3_ANTS", "GND")


# ================================================================================================= 9 NTC1 comparators, lockout, load shed, clock
def hw_logic():
    # VHW = OR(VCTL, 3V3_AO)
    p = place(TDDUAL_CC, "D24", "BAT54C", "BAT54C,215", "Nexperia", "SOT-23", "30 V dual Schottky, common cathode", "", "VHW OR",
              "Sn", "not checked")
    conn(p, {"A1": "VCTL", "A2": "3V3_AO", "K": "VHW"})
    C("1u", "VHW", "GND", 25)
    # NTC1 (own return wire): 10 k bias from VHW
    R(10e3, "VHW", "NTC1", tol=0.1, note="NTC1 bias")
    C("100n", "NTC1", "GND")
    # threshold ladder (0.1 %): VHW - n0 - n40 - n45 - n60 - GND
    R(267e3, "VHW", "TH_0C", tol=0.1, note="ladder: +1.1 C node (charge allowed only above 0 C with the +/-1 K tolerance)")
    R(365e3, "TH_0C", "TH_40C", tol=0.1, note="40.0 C node (heater cutout)")
    R(31.6e3, "TH_40C", "TH_45C", tol=0.1, note="44.0 C node (charge allowed only below 45 C)")
    R(104e3, "TH_45C", "TH_60C", tol=0.1, note="60.0 C node (both AFE FETs open)")
    R(229e3, "TH_60C", "GND", tol=0.1)
    ic(S_CMP2, "U53", "TLV7042-Q1", "TLV7042QDGKRQ1", "TI", "VSSOP-8 (DGK)", "dual nanopower comparator, open drain",
       {"OUTA": "WIN_OK", "INAM": "NTC1", "INAP": "TH_0C", "VEE": "GND", "INBP": "NTC1", "INBM": "TH_45C", "OUTB": "WIN_OK",
        "VCC": "VHW"}, notes="A: cold (low below 0 C); B: hot (low above 45 C); wired-AND = WIN_OK")
    C("100n", "VHW", "GND")
    R(100e3, "VHW", "WIN_OK")
    ic(S_CMP2, "U54", "TLV7042-Q1", "TLV7042QDGKRQ1", "TI", "VSSOP-8 (DGK)", "dual nanopower comparator, open drain",
       {"OUTA": "HCUT_G", "INAM": "TH_40C", "INAP": "NTC1", "VEE": "GND", "INBP": "TH_60C", "INBM": "NTC1", "OUTB": "OT60",
        "VCC": "VHW"}, notes="A: heater cutout (low above 40 C); B: pack over-temperature (released above 60 C)")
    C("100n", "VHW", "GND")
    R(100e3, "VHW", "HCUT_G")
    R(100e3, "VHW", "OT60")
    # NTC1 plausibility (RID SMA-B04): open (< -55 C, ratio > 0.985) or short (> +150 C, ratio < 0.030) -> NTC1_BAD;
    # then the charge window passes to the AFE's UTC/OTC on NTC2, the 60 C BOTHOFF to the AFE's OTD on NTC2, and the
    # heater cutout to the flight software, SW5's limit, F4 and the AFE's OTD
    R(15.0e3, "VHW", "TH_OPEN", tol=0.1, note="NTC1 open threshold 0.985 VHW")
    R(1.00e6, "TH_OPEN", "GND", tol=0.1)
    R(1.00e6, "VHW", "TH_SHORT", tol=0.1, note="NTC1 short threshold 0.030 VHW")
    R(30.9e3, "TH_SHORT", "GND", tol=0.1)
    ic(S_CMP2, "U66", "TLV7042-Q1", "TLV7042QDGKRQ1", "TI", "VSSOP-8 (DGK)", "dual nanopower comparator, open drain",
       {"OUTA": "NTC1_OPEN", "INAM": "TH_OPEN", "INAP": "NTC1", "VEE": "GND", "INBP": "TH_SHORT", "INBM": "NTC1", "OUTB": "NTC1_SHORT",
        "VCC": "VHW"}, notes="NTC1 plausibility: A open (released when NTC1 > 0.985 VHW), B short (released when NTC1 < 0.030 VHW)")
    C("100n", "VHW", "GND")
    R(100e3, "VHW", "NTC1_OPEN")
    R(100e3, "VHW", "NTC1_SHORT")
    ic(S_G1, "U67", "SN74LVC1G32-Q1", "SN74LVC1G32QDCKRQ1", "TI", "SC70-5 (DCK)", "OR",
       {"A": "NTC1_OPEN", "B": "NTC1_SHORT", "GND": "GND", "Y": "NTC1_BAD", "VCC": "VHW"}, finish="NiPdAu|Sn", notes="NTC1 implausible")
    C("100n", "VHW", "GND")
    ic(S_G1, "U68", "SN74LVC1G32-Q1", "SN74LVC1G32QDCKRQ1", "TI", "SC70-5 (DCK)", "OR",
       {"A": "WIN_OK", "B": "NTC1_BAD", "GND": "GND", "Y": "WIN_EFF", "VCC": "VHW"}, finish="NiPdAu|Sn",
       notes="charge window: NTC1 comparators, or the AFE's UTC/OTC (NTC2) alone when NTC1 is implausible")
    C("100n", "VHW", "GND")
    ic(S_G1, "U69", "SN74LVC1G32-Q1", "SN74LVC1G32QDCKRQ1", "TI", "SC70-5 (DCK)", "OR",
       {"A": "HCUT_G", "B": "NTC1_BAD", "GND": "GND", "Y": "HCUT_EFF", "VCC": "VHW"}, finish="NiPdAu|Sn",
       notes="heater cutout bypassed when NTC1 is implausible")
    C("100n", "VHW", "GND")
    Q("2N7002BK", "Q43", "NTC1_BAD", "GND", "OT60", note="60 C BOTHOFF from NTC1 suppressed when NTC1 is implausible (AFE OTD on NTC2 remains)")
    R(100e3, "NTC1_BAD", "NTC1_BAD_RB", note="readback to the EPS controller (series 100 k)")
    # lockout relay K1 (2-coil latching): pole 1 COM=WIN_OK, NC1 -> PERMIT; pole 2 COM=GND, NO2 -> CC_SET (locked)
    p = place(S_RELAY, "K1", "TQ2SA-L2-3V", "TQ2SA-L2-3V-Z", "Panasonic", "TQ2 SMD (10 pin)",
              "2 Form C, 2-coil latching, 3 V coil 46.7 mA / 64.3 Ohm, 0.5 A contacts", "uA dry circuit",
              "latched recharge lockout (non-volatile); pin numbers to be checked against the TQ2 2-coil drawing at symbol entry",
              "Au-clad contacts; Sn terminals", "Digi-Key: Active")
    conn(p, {"SET_P": "3V3_AO", "SET_N": "SET_COIL", "RST_P": "3V3_AO", "RST_N": "RST_COIL", "COM1": "WIN_EFF", "NC1": "PERMIT",
             "NO1": None, "COM2": "GND", "NO2": "CC_SET", "NC2": None})
    R(1e6, "PERMIT", "GND")
    ic(S_INV, "U55", "SN74LVC1G04-Q1", "SN74LVC1G04QDCKRQ1", "TI", "SC70-5 (DCK)", "inverter",
       {"NC": None, "A": "PERMIT", "GND": "GND", "Y": "CHG_INHIBIT", "VCC": "VHW"}, finish="NiPdAu|Sn",
       notes="CHG_INHIBIT = not PERMIT (CC set point to 0, AFE CFETOFF)")
    C("100n", "VHW", "GND")
    D("1N4148W", "3V3_AO", "SET_COIL", note="SET coil flyback")
    D("1N4148W", "3V3_AO", "RST_COIL", note="RESET coil flyback")
    # each coil through two FETs in series (RID SMA-B12): one shorted FET leaves the coil off
    Q("2N7002BK", "Q28", "SET_DRV", "SET_MID", "SET_COIL", note="SET coil driver, upper")
    Q("2N7002BK", "Q30", "SET_DRV", "GND", "SET_MID", note="SET coil driver, lower (series redundant)")
    Q("2N7002BK", "Q29", "RST_DRV", "RST_MID", "RST_COIL", note="RESET coil driver, upper")
    Q("2N7002BK", "Q39", "RST_DRV", "GND", "RST_MID", note="RESET coil driver, lower (series redundant)")
    # SET request: AFE FUSE (PF, SUV 2.0 V) via 100 k/100 k OR EPS-controller LOCK_SET -> 50 ms one-shot
    R(100e3, "AFE_FUSE", "FUSE_LVL", note="cross-domain series 100 k")
    R(100e3, "FUSE_LVL", "GND")
    ic(S_G1, "U56", "SN74LVC1G32-Q1", "SN74LVC1G32QDCKRQ1", "TI", "SC70-5 (DCK)", "OR",
       {"A": "FUSE_LVL", "B": "LOCK_SET_MCU", "GND": "GND", "Y": "SET_REQ", "VCC": "3V3_AO"}, finish="NiPdAu|Sn")
    R(100e3, "LOCK_SET_MCU", "GND")
    C("100n", "SET_REQ", "GND")
    ic(S_TPL5111, "U57", "TPL5111", "TPL5111DDCR", "TI", "SOT-23-6 (DDC)", "one-shot 50 ms",
       {"VDD": "SET_REQ", "GND": "GND", "DELAY": "SETP_DLY", "DONE": "GND", "DRVN": "SET_DRV", "ONESHOT": "GND"},
       checked="TI addendum: Active (industrial)")
    R(499, "SETP_DLY", "GND", note="tIP 100 ms")
    R(100e3, "SET_DRV", "GND")
    # RESET (clear) request: OBC LOCK_CLR AND EPS-controller UNLOCK (two independent signals) -> 50 ms one-shot
    ic(S_G1, "U58", "SN74LVC1G08-Q1", "SN74LVC1G08QDCKRQ1", "TI", "SC70-5 (DCK)", "AND",
       {"A": "LOCK_CLR", "B": "UNLOCK_MCU", "GND": "GND", "Y": "RST_REQ", "VCC": "3V3_AO"}, finish="NiPdAu|Sn")
    R(100e3, "LOCK_CLR", "GND")
    R(100e3, "UNLOCK_MCU", "GND")
    C("100n", "RST_REQ", "GND")
    ic(S_TPL5111, "U59", "TPL5111", "TPL5111DDCR", "TI", "SOT-23-6 (DDC)", "one-shot 50 ms",
       {"VDD": "RST_REQ", "GND": "GND", "DELAY": "RSTP_DLY", "DONE": "GND", "DRVN": "RST_DRV", "ONESHOT": "GND"},
       checked="TI addendum: Active (industrial)")
    R(499, "RSTP_DLY", "GND")
    R(100e3, "RST_DRV", "GND")
    # hardware load shed: VBAT < 13.6 V for 5 s -> SHED_N low (released above 14.4 V)
    R(1.00e6, "VBAT", "SHED_DIV", tol=0.1, note="13.6 V falling / 14.4 V rising")
    R(221e3, "SHED_DIV", "GND", tol=0.1)
    R(4.12e6, "SHED_DIV", "UV_OK", note="hysteresis")
    C("1n", "SHED_DIV", "GND")
    R(10e3, "3V3_AO", "UV_OK")
    D("BAT54", "SHED_T", "UV_OK", note="fast release")
    R(4.75e6, "UV_OK", "SHED_T", note="5 s delay (4.75 M x 1.0 uF)")
    C("1u", "SHED_T", "GND", 25)
    R(56.2e3, "REF_A", "SHED_TH", tol=0.1, note="1.100 V threshold")
    R(44.2e3, "SHED_TH", "GND", tol=0.1)
    ic(S_CMP2, "U48", "TLV7042-Q1", "TLV7042QDGKRQ1", "TI", "VSSOP-8 (DGK)", "dual comparator, open drain",
       {"OUTA": "UV_OK", "INAM": "REF_A", "INAP": "SHED_DIV", "VEE": "GND", "INBP": "SHED_T", "INBM": "SHED_TH", "OUTB": "SHED_N",
        "VCC": "3V3_AO"}, notes="A: VBAT under-voltage; B: 5 s timer -> SHED_N")
    C("100n", "3V3_AO", "GND")
    R(10e3, "3V3_AO", "SHED_N")
    TP("SHED_N")
    # clock: 4 MHz XO / 4 -> 1.000 MHz Q (SYNC_A) and Q-bar (SYNC_B)
    ic(S_OSC, "Y1", "4.000MHz", "SG-8018CA 4.0000M-TJHPA0", "Epson", "7.0 x 5.0 mm 4-SMD", "CMOS XO 1.62-3.63 V, +/-50 ppm, -40..105 C",
       {"OE": "3V3_AO", "GND": "GND", "OUT": "CLK4M", "VCC": "3V3_AO"}, finish="Au", checked="Digi-Key: Active")
    C("100n", "3V3_AO", "GND")
    ic(S_FF, "U62", "SN74LVC2G74-Q1", "SN74LVC2G74QDCURQ1", "TI", "VSSOP-8 (DCU)", "divide by 2",
       {"CLK": "CLK4M", "D": "CLK2M_N", "QN": "CLK2M_N", "GND": "GND", "Q": "CLK2M", "CLRN": "3V3_AO", "PREN": "3V3_AO", "VCC": "3V3_AO"})
    ic(S_FF, "U63", "SN74LVC2G74-Q1", "SN74LVC2G74QDCURQ1", "TI", "VSSOP-8 (DCU)", "divide by 2",
       {"CLK": "CLK2M", "D": "SYNC_B", "QN": "SYNC_B", "GND": "GND", "Q": "SYNC_A", "CLRN": "3V3_AO", "PREN": "3V3_AO", "VCC": "3V3_AO"},
       notes="SYNC_A (1 MHz) and SYNC_B (1 MHz, 180 deg)")
    C("100n", "3V3_AO", "GND")
    C("100n", "3V3_AO", "GND")
    R(100e3, "SYNC_A", "GND", note="clock lost -> converters' auto mode")
    R(100e3, "SYNC_B", "GND")


# ================================================================================================= 10 telemetry and EPS controller
def telemetry_and_mcu():
    # INA228 pack gauge (OBC bus 0x44) and array monitor (0x40)
    ic(S_INA228, "U28", "INA228-Q1", "INA228AQDGSRQ1", "TI", "VSSOP-10 (DGS)", "85 V, 20-bit power/energy/charge monitor",
       {"A1": "3V3_AO", "A0": "GND", "ALERT": "EPS_ALERT_N", "SDA": "I2C2_SDA", "SCL": "I2C2_SCL", "VS": "3V3_AO", "GND": "GND",
        "VBUS": "BAT_F3", "INM": "BAT_F3", "INP": "BAT_F"}, stress="CM 17.4 V / 85 V = 20 %",
       notes="fuel gauge (coulomb counter), OBC I2C2 0x44; positive = charging", finish="Sn (matte): coat")
    C("100n", "3V3_AO", "GND")
    ic(S_INA228, "U29", "INA228-Q1", "INA228AQDGSRQ1", "TI", "VSSOP-10 (DGS)", "85 V, 20-bit power monitor",
       {"A1": "GND", "A0": "GND", "ALERT": "EPS_ALERT_N", "SDA": "I2C2_SDA", "SCL": "I2C2_SCL", "VS": "3V3_AO", "GND": "GND",
        "VBUS": "VTRK", "INM": "VTRK", "INP": "VTRK_RAW"}, notes="array output monitor, OBC I2C2 0x40", finish="Sn (matte): coat")
    C("100n", "3V3_AO", "GND")
    R(4.7e3, "3V3_AO", "EPS_ALERT_N")
    # IBATT_SNS (bidirectional, REF 1.65 V buffered) and ISOLAR_SNS (1 V/A)
    R(10e3, "3V3_AO", "REF165_D", tol=0.1)
    R(10e3, "REF165_D", "GND", tol=0.1)
    ic(S_OPA2, "U49", "TLV9002-Q1", "TLV9002QDGKRQ1", "TI", "VSSOP-8 (DGK)", "RRIO",
       {"OUTA": "REF165", "INAM": "REF165", "INAP": "REF165_D", "VM": "GND", "INBP": "GND", "INBM": "U49_BM", "OUTB": "U49_BM",
        "VP": "3V3_AO"}, notes="A: 1.65 V reference buffer for IBATT; B: unused, follower to GND")
    C("100n", "3V3_AO", "GND")
    ic(S_INA186, "U30", "INA186A2-Q1", "INA186A2QDCKRQ1", "TI", "SC70-6 (DCK)", "40 V CM, x50", {"REF": "REF165", "GND": "GND",
       "VS": "3V3_AO", "INP": "BAT_F", "INM": "BAT_F3", "OUT": "IBATT_SNS"}, notes="IBATT_SNS 0.5 V/A, 1.65 V at 0 A", finish="NiPdAu|Sn")
    ic(S_INA186, "U31", "INA186A2-Q1", "INA186A2QDCKRQ1", "TI", "SC70-6 (DCK)", "40 V CM, x50", {"REF": "GND", "GND": "GND",
       "VS": "3V3_AO", "INP": "VTRK_RAW", "INM": "VTRK", "OUT": "ISOLAR_SNS"}, notes="ISOLAR_SNS 1.0 V/A", finish="NiPdAu|Sn")
    C("100n", "3V3_AO", "GND")
    C("100n", "3V3_AO", "GND")
    C("10n", "IBATT_SNS", "GND")
    C("10n", "ISOLAR_SNS", "GND")
    # INA3221 x4 on the local bus (0x40-0x43)
    chans = [("U32", "GND", [("T1_IN", "T1_L"), ("T2_IN", "T2_L"), ("T3_IN", "T3_L")]),
             ("U33", "3V3_MCU", [("T4_IN", "T4_L"), ("VB3", "VBAT"), ("3V3_BK", "3V3_AO")]),
             ("U34", "LSDA", [("5V_BK", "5V_PA"), ("3V3_RADIO_U", "3V3_RADIO"), ("3V3_ADCS_U", "3V3_ADCS")]),
             ("U35", "LSCL", [("3V3_GNSS_U", "3V3_GNSS"), ("3V3_ANTS_U", "3V3_ANTS"), ("HTR_F", "HTR_P")])]
    for ref, a0, ch in chans:
        m = {"GND": "GND", "VS": "3V3_MCU", "A0": a0, "SCL": "LSCL", "SDA": "LSDA", "WARN": None, "CRIT": None, "PV": None,
             "TC": None, "VPU": "3V3_MCU", "PAD": "GND"}
        for i, (p_, n_) in enumerate(ch, 1):
            m[f"INP{i}"], m[f"INM{i}"] = p_, n_
        ic(S_INA3221, ref, "INA3221-Q1", "INA3221AQRGVRQ1", "TI", "VQFN-16 (RGV) 4x4", "26 V, 3-channel shunt/bus monitor", m,
           stress="CM 16.8 V / 26 V = 65 % (heater channel)", notes="local-bus rail telemetry")
        C("100n", "3V3_MCU", "GND")
    # ADS7138: panel NTCs, NTC1 and VHW (local bus 0x10)
    ic(S_ADS7138, "U36", "ADS7138-Q1", "ADS7138QRTERQ1", "TI", "WQFN-16 (RTE) 3x3", "8-ch 12-bit I2C ADC, AEC-Q100",
       {"AIN0": "TPNL_PX", "AIN1": "TPNL_MX", "AIN2": "TPNL_PY", "AIN3": "TPNL_MY", "AIN4": "TPNL_W1", "AIN5": "TPNL_W2",
        "AIN6": "NTC1", "AIN7": "VHW_SNS", "AVDD": "3V3_MCU", "DECAP": "ADS_DECAP", "GND": "GND", "DVDD": "3V3_MCU",
        "ADDR": "GND", "ALERT": None, "SCL": "LSCL", "SDA": "LSDA", "PAD": "GND"}, notes="panel and pack temperatures")
    C("1u", "ADS_DECAP", "GND", 25)
    C("1u", "3V3_MCU", "GND", 25)
    for pnl in ("PX", "MX", "PY", "MY", "W1", "W2"):
        R(10e3, "3V3_MCU", f"TPNL_{pnl}", tol=0.1, note=f"panel {pnl} NTC bias")
        C("100n", f"TPNL_{pnl}", "GND")
    R(100e3, "VHW", "VHW_SNS")
    R(100e3, "VHW_SNS", "GND")
    R(4.7e3, "3V3_MCU", "LSDA")
    R(4.7e3, "3V3_MCU", "LSCL")
    # MSP430FR2355 EPS controller (I2C target 0x48 on the OBC bus; master on the local bus)
    msp = {nm: None for nm in MSP_PINS.values()}
    msp.update({
        "P1_2": "I2C2_SDA", "P1_3": "I2C2_SCL", "P4_6": "LSDA", "P4_7": "LSCL",
        "P1_1": "DAC_T1", "P1_5": "DAC_T2", "P3_1": "DAC_T3", "P3_5": "DAC_T4",
        "P1_0": "IBURN_W1A", "P1_4": "IBURN_W1B", "P1_6": "IBURN_W2A", "P1_7": "IBURN_W2B",
        "P5_0": "IMON_JET", "P5_1": "F1_SNS", "P5_2": "V_BURN_SNS", "P5_3": "VTRK_SNS",
        "P2_0": "AFE_ALERT_X", "P2_1": "PERMIT_RB", "P2_2": "WIN_OK_RB", "P2_3": "SHED_RB", "P2_4": "EOL_ARMED_RB",
        "P2_5": "FLT_SW1", "P2_6": "FLT_EFUSE", "P2_7": "FLT_SW5", "P3_0": "FLT_SW2", "P3_2": "FLT_SW3", "P3_3": "FLT_SW4",
        "P3_4": "FLT_SW7", "P3_6": "FLT_SW8", "P3_7": "FLT_SW6", "P4_5": "FLT_SW11",
        "P4_2": "EOL_KEY", "P4_3": "NTC1_BAD_RB", "P6_0": "ANTS_EN_MCU", "P6_1": "OBC_CYCLE", "P6_2": "TDIS_T1", "P6_3": "TDIS_T2", "P6_4": "TDIS_T3", "P6_5": "TDIS_T4", "P4_0": "LOCK_SET_MCU", "P4_1": "UNLOCK_MCU", "P4_4": "MCU_INT_N",
        "TEST": "MCU_TEST", "RSTN": "MCU_RSTN", "DVCC": "3V3_MCU", "DVSS": "GND"})
    ic(S_MSP, "U37", "MSP430FR2355", "MSP430FR2355TPTR", "TI", "LQFP-48 (PT) 7x7", "16-bit FRAM MCU, 12-bit ADC, 4 SAC op-amp/DAC, -40..105 C",
       msp, notes="EPS controller: ICD register block 0x00-0x88, MPPT set points, telemetry; no safety function depends on it",
       checked="TI addendum: Active (not AEC-Q100)")
    C("100n", "3V3_MCU", "GND")
    C("4.7u", "3V3_MCU", "GND", 50, "1210")
    R(47e3, "3V3_MCU", "MCU_RSTN")
    C("1n", "MCU_RSTN", "GND")
    R(47e3, "MCU_TEST", "GND")
    # readbacks from other domains (100 k series)
    for src, dst in (("PERMIT", "PERMIT_RB"), ("WIN_OK", "WIN_OK_RB"), ("SHED_N", "SHED_RB"), ("EOL_ARMED", "EOL_ARMED_RB")):
        R(100e3, src, dst, note="readback (series 100 k)")
    R(10e3, "MCU_INT_N", "EPS_ALERT_N", note="MCU open-drain alert onto EPS_ALERT_N")
    R(200e3, "V_BURN", "V_BURN_SNS")
    R(100e3, "V_BURN_SNS", "GND")
    R(1e6, "VTRK", "VTRK_SNS")
    R(220e3, "VTRK_SNS", "GND")
    C("10n", "VTRK_SNS", "GND")


def build():
    connectors()
    solar_and_trackers()
    charge_control()
    pack_and_protection()
    inhibits()
    eol()
    rails_and_switches()
    burn_drivers()
    hw_logic()
    telemetry_and_mcu()
    import re
    pat = re.compile(r"^(S_(W[12][FB]|P[XY]|M[XY])|T[1-4]_(IN|L)|BXP_W[12][AB]|OVP_VDD|SET_REQ|RST_REQ|3V3_BK|5V_BK|"
                     r"3V3_RADIO_U|5V_PA_U|3V3_ADCS_U|3V3_GNSS_U|3V3_ANTS_U|HTR_.*|UMB_.*|N_F2|N_RT1|FET_CD)$")
    for name, n in _nets.items():
        if name in POWER_NETS or pat.match(name):
            n.drive = POWER
    # number the parts placed without an explicit reference after the explicit ones (no collisions)
    used = defaultdict(set)
    auto = []
    for prt in builtins.default_circuit.parts:
        if id(prt) in AUTOREF:
            auto.append(prt)
        else:
            m = re.match(r"([A-Z]+)(\d+)$", prt.ref)
            if m:
                used[m.group(1)].add(int(m.group(2)))
    nxt = defaultdict(lambda: 1)
    for prt in auto:
        pre = prt.ref_prefix
        while nxt[pre] in used[pre]:
            nxt[pre] += 1
        prt.ref = f"{pre}{nxt[pre]}"
        used[pre].add(nxt[pre])


def outputs():
    circ = builtins.default_circuit
    # ERC to a log
    import io
    import logging
    erc_path = os.path.join(HERE, "eps_board_erc.log")
    buf = io.StringIO()
    h = logging.StreamHandler(buf)
    h.setLevel(logging.INFO)
    lg = skidl.erc_logger if hasattr(skidl, "erc_logger") else logging.getLogger("ERC_Logger")
    lg.addHandler(h)
    ERC()
    lg.removeHandler(h)
    with open(erc_path, "w") as f:
        f.write("SQSO-EPS-BRD-001 electrical rule check (SKiDL 3.0 ERC), generated by design.py\n\n")
        f.write(buf.getvalue())
        f.write("\nWaivers (intentional modelling choices, reviewed):\n"
                "W1  MSP430FR2355 port pins are typed PASSIVE: their direction is set by the EPS-controller firmware.\n"
                "W2  Multi-pin outputs of one device (TPS1H100 OUT5-7, TPS16630 OUT17-18) have one POWER-OUT pin; the others are\n"
                "    PASSIVE, so the checker does not report the device driving its own net.\n"
                "W3  Power nets fed through passive elements (FETs, fuses, shunts, switches, connectors) are declared driven\n"
                "    (net.drive = POWER): the pack and string nets, switched rails and the one-shot supplies (BXP_*, SET_REQ,\n"
                "    RST_REQ, which are logic outputs powering a 35 nA timer).\n"
                "W4  Pins marked NOCONNECT are left open on purpose (datasheet NC pins, unused AFE outputs DCHG/DDSG/PDSG/REG2,\n"
                "    TPS16630 MODE open = latch-off, TPS1H000 DELAY open = default, INA3221 alert outputs, unused MCU ports).\n"
                "W5  The OBC I2C2 pull-ups are on the OBC board (ICD-001 section 2.2); the EPS adds none on I2C2_SCL/SDA.\n")
    generate_netlist(file_=os.path.join(HERE, "eps_board.net"), tool=KICAD8)
    # nets CSV
    with open(os.path.join(HERE, "eps_board_nets.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["net", "pins"])
        for net in sorted(circ.nets, key=lambda n: n.name):
            if not net.pins:
                continue
            w.writerow([net.name, " ".join(sorted(f"{p.part.ref}.{p.num}" for p in net.pins))])
    # BOM grouped by MPN + value
    groups = OrderedDict()
    for p in circ.parts:
        info = BOMINFO.get(id(p))
        if info is None:
            continue
        key = (info["mpn"], info["value"])
        groups.setdefault(key, []).append((p.ref, info))
    def refkey(r):
        import re
        m = re.match(r"([A-Z]+)(\d+)", r)
        return (m.group(1), int(m.group(2))) if m else (r, 0)
    rows = []
    for (mpn, value), items in groups.items():
        refs = sorted((r for r, _ in items), key=refkey)
        info = items[0][1]
        stresses = sorted({i["stress"] for _, i in items if i["stress"]})
        notes = sorted({i["notes"] for _, i in items if i["notes"]})
        rows.append([" ".join(refs), len(refs), value, mpn, info["mfr"], info["pkg"], info["rating"],
                     "; ".join(stresses)[:300], "; ".join(notes)[:300], info["finish"], info["checked"]])
    rows.sort(key=lambda r: refkey(r[0].split()[0]))
    with open(os.path.join(HERE, "eps_board_bom.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["Reference", "Qty", "Value", "MPN", "Manufacturer", "Package", "Rating", "Derated stress", "Notes",
                    "Terminal finish / tin mitigation", "MPN check"])
        w.writerows(rows)
    for junk in ("design.erc", "design.log", "design_sklib.py", "skidl_REPL.erc", "skidl_REPL.log"):
        if os.path.exists(os.path.join(HERE, junk)):
            os.remove(os.path.join(HERE, junk))
    total = sum(r[1] for r in rows)
    print(f"parts: {total} placements, {len(rows)} BOM lines, {len([n for n in circ.nets if n.pins])} nets")


if __name__ == "__main__":
    os.chdir(HERE)
    build()
    outputs()
