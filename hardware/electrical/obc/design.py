#!/usr/bin/env python3
"""SMART-QSO OBC board, schematic captured as code (SKiDL 3.0). Document: SQSO-OBC-BRD-001 (OBC_BOARD_DESIGN.md).
Master pin allocation and board-to-board signal list: SQSO-ICX-001 (hardware/electrical/INTERCONNECT.md).

Run with the CAD venv (SKiDL 3.0; no KiCad libraries: every symbol is defined here from its datasheet pin table):

    python3 hardware/electrical/obc/obc_calcs.py          # first: writes obc_calcs.json
    <venv>/bin/python hardware/electrical/obc/design.py

Writes, next to this file: obc_board.net (KiCad 8 netlist), obc_board_bom.csv, obc_board_erc.log,
obc_board_nets.csv (net -> pins) and obc_pinmap.csv (every STM32L4R5ZIT6 pin: net, function, connector).

Domains (OBC_BOARD_DESIGN.md section 4):
  3V3_OBC   the MCU domain, from EPS SW10 (0.15 A latch-up trip, auto-retry) on J1-1/2
  3V3_ADCS  from EPS SW4 (ADCS_EN) on J6; feeds three on-board current-limited switches:
            3V3_GYRO (M-G370PDT0), 3V3_IMU2 (BMI088, cold spare), 3V3_SNS (RM3100, sun-sensor front end, ADS7138)
  3V3_RTC   3V3_AO through 1 kOhm on the EPS (J1-38): the RV-3028-C7 mission clock, alive across OBC power cycles
  VBAT_MTQ  VBAT through a new EPS switch on ADCS_EN (J1-36/39): the three magnetorquer H-bridges
  JET_3V3   the Jetson carrier's own 3.3 V I/O rail (J3-4), B side of the USART1 translators (RID AV-09)
  3V3_GNSS  EPS SW7 (GNSS_EN) routed through the OBC (J4 in, J5 out), B side of the LPUART1 translators
Every signal that crosses into a domain the MCU does not power goes through a part with Ioff / VCC isolation
(TXU0304/TXU0204, SN74LVC1T45, TCA9517), so no switched-off rail is back-powered from the MCU.
"""

import builtins
import csv
import io
import json
import logging
import os
import re
from collections import OrderedDict, defaultdict

from skidl import ERC, KICAD8, POWER, TEMPLATE, Net, Part, Pin, SKIDL, generate_netlist, set_default_tool
import skidl

set_default_tool(SKIDL)
HERE = os.path.dirname(os.path.abspath(__file__))
T = Pin.types
TY = dict(P=T.PASSIVE, I=T.INPUT, O=T.OUTPUT, B=T.BIDIR, W=T.PWRIN, WO=T.PWROUT, OC=T.OPENCOLL, N=T.NOCONNECT,
          TS=T.TRISTATE)

try:
    CALC = json.load(open(os.path.join(HERE, "obc_calcs.json")))
except FileNotFoundError:
    raise SystemExit("run obc_calcs.py first (writes obc_calcs.json)")


def cv(key, fmt="{:.3g}"):
    return fmt.format(CALC[key]["value"])


# ------------------------------------------------------------------------------------------------- helpers
_nets = {}
POWER_NETS = ["GND", "3V3_OBC", "3V3_ADCS", "3V3_GYRO", "3V3_IMU2", "3V3_SNS", "3V3_MAG", "3V3_RTC", "VBAT_MTQ", "JET_3V3",
              "3V3_GNSS", "VDDA", "3V3_VTREF"]


def N(name):
    if name not in _nets:
        _nets[name] = Net(name)
    return _nets[name]


def sym(name, prefix, pins):
    plist = []
    for tok in pins.split():
        num, pname, ty = tok.split(":")
        plist.append(Pin(num=num, name=pname, func=TY[ty]))
    return Part(tool=SKIDL, name=name, ref_prefix=prefix, dest=TEMPLATE, pins=plist)


BOMINFO = {}
AUTOREF = set()


def place(tm, ref, value, mpn, mfr, pkg, rating, stress="", notes="", finish="", checked="", dnp=False):
    p = tm(value=value)
    p.tag = f"obc{len(BOMINFO):04d}"  # deterministic tag: the netlist is the same on every run
    if ref:
        p.ref = ref
    else:
        AUTOREF.add(id(p))
        p.ref = f"{p.ref_prefix}TMP{len(AUTOREF)}"
    p.footprint = pkg
    p.fields["MPN"] = mpn
    p.fields["Manufacturer"] = mfr
    if dnp:
        p.fields["DNP"] = "DNP"
    BOMINFO[id(p)] = dict(value=value, mpn=mpn, mfr=mfr, pkg=pkg, rating=rating, stress=stress, notes=notes,
                          finish=finish, checked=checked, dnp=dnp)
    return p


def link(name, pin):
    N(name).__iadd__(pin)


def conn(p, mapping):
    for pin, net in mapping.items():
        if net is None:
            p[pin] += builtins.NC
        else:
            link(net, p[pin])


TR = sym("R", "R", "1:A:P 2:B:P")
TC = sym("C", "C", "1:A:P 2:B:P")
TFB = sym("FB", "FB", "1:A:P 2:B:P")
TD = sym("D", "D", "1:K:P 2:A:P")
TQN = sym("Q_NMOS", "Q", "1:G:P 2:S:P 3:D:P")
TLED = sym("LED", "D", "1:K:P 2:A:P")
TTP = sym("TP", "TP", "1:TP:P")
TCOIL = sym("COIL", "L", "1:A:P 2:B:P")


def rcode(v):
    """Panasonic ERJ value code: 3 significant digits + multiplier (10.0k -> 1002; 33 ohm -> 33R0)."""
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


def R(v, a, b, tol=1.0, note="", stress="", ref=None, dnp=False):
    """0603 chip resistor: 1 % Panasonic ERJ-3EKF, 0.1 % Panasonic ERA-3AEB (both AEC-Q200), > 1 M Vishay CRCW-e3."""
    if v == 0:
        mpn, rating, mfr = "ERJ-3GEY0R00V", "0 ohm jumper, 1 A", "Panasonic"
    elif tol <= 0.1:
        mpn, rating, mfr = f"ERA-3AEB{rcode(v)}V", "0.1 %, 25 ppm/K, 0.1 W, 75 V, AEC-Q200", "Panasonic"
    elif v > 1.0e6:
        mant = f"{v / 1e6:.3g}".replace(".", "M")
        mant = mant if "M" in mant else mant + "M"
        mpn, rating, mfr = f"CRCW0603{mant.ljust(4, '0')}FKEA", "1 %, 100 ppm/K, 0.1 W, 75 V, AEC-Q200", "Vishay Dale"
    else:
        mpn = f"ERJ-3EKF{rcode(v)}V" if v >= 10 else f"ERJ-3RQF{rcode(v)}V"
        rating, mfr = "1 %, 100 ppm/K, 0.1 W, 75 V, AEC-Q200", "Panasonic"
    p = place(TR, ref, fmt(v), mpn, mfr, "Resistor_SMD:R_0603", rating, stress or "< 10 % of rated power", note,
              "Sn over Ni (matte): re-tin or coat (EEE_PARTS_LIST 2A.3)", "series code", dnp)
    link(a, p["A"])
    link(b, p["B"])
    return p


CAPS = {  # (value, V, size) -> (Murata GCM AEC-Q200 part, dielectric)
    ("100n", 50, "0603"): ("GCM188R71H104KA57D", "X7R"),
    ("10n", 50, "0603"): ("GCM188R71H103KA37D", "X7R"),
    ("22n", 50, "0603"): ("GCM188R71H223KA37D", "X7R"),
    ("1n", 50, "0603"): ("GCM1885C1H102JA16D", "C0G"),
    ("10nC0G", 50, "0603"): ("GCM1885C1H103JA16D", "C0G"),
    ("10p", 50, "0603"): ("GCM1885C1H100JA16D", "C0G"),
    ("100nC0G", 50, "1206"): ("GCM31C5C1H104JA16L", "C0G"),
    ("470nC0G", 50, "1210"): ("C0G 470 nF 50 V 1210, soft termination (Murata GCJ / TDK CGA6 class; order code to select)", "C0G"),
    ("1u", 25, "0603"): ("GCM188R71E105KA64D", "X7R"),
    ("2.2u", 25, "0805"): ("GCM21BR71E225KA73L", "X7R"),
    ("4.7u", 50, "1210"): ("GCM32ER71H475KA55L", "X7R"),
    ("10u", 25, "1206"): ("GCM31CR71E106KA64L", "X7R"),
    ("22u", 16, "1210"): ("GCM32ER71C226KE19L", "X7R"),
}


CAP_DEFAULT = {"1u": (25, "0603"), "2.2u": (25, "0805"), "10u": (25, "1206"), "22u": (16, "1210"),
               "4.7u": (50, "1210"), "100nC0G": (50, "1206"), "470nC0G": (50, "1210")}


def C(val, a, b, v=None, size=None, vop=3.3, note="", ref=None, dnp=False):
    dv, ds = CAP_DEFAULT.get(val, (50, "0603"))
    v = v or dv
    size = size or ds
    mpn, die = CAPS[(val, v, size)]
    shown = val.replace("C0G", "")
    p = place(TC, ref, shown, mpn, "Murata", f"Capacitor_SMD:C_{size}", f"{v} V {die} AEC-Q200",
              f"{vop:.1f} V / {v} V = {vop / v:.0%} of rated V", note, "Sn over Ni (matte): re-tin or coat",
              "series code", dnp)
    link(a, p["A"])
    link(b, p["B"])
    return p


DIODES = {
    "BAT54": ("BAT54,215", "Nexperia", "SOT-23", "30 V, 200 mA Schottky", "not checked"),
    "1N4148W-Q": ("1N4148W-Q-7-F", "Diodes Inc.", "SOD-123", "100 V, 300 mA, AEC-Q101", "not checked"),
    "SMBJ20A": ("SMBJ20A", "Littelfuse", "DO-214AA (SMB)", "TVS 20 V VRWM, 22.2 V VBR min, 32.4 V Vc, 600 W", "Digi-Key listing (EPS board)"),
    "SMF5.0A": ("SMF5.0A", "Littelfuse", "SOD-123FL", "TVS 5.0 V VRWM, 6.4 V VBR min, 9.2 V Vc, 200 W", "not checked"),
    "SMF24A": ("SMF24A", "Littelfuse", "SOD-123FL", "TVS 24 V VRWM, 26.7 V VBR min, 38.9 V Vc, 200 W", "not checked"),
}


def D(kind, k, a, ref=None, note="", stress=""):
    mpn, mfr, pkg, rating, chk = DIODES[kind]
    p = place(TD, ref, kind, mpn, mfr, pkg, rating, stress, note, "Sn (matte): re-tin or coat", chk)
    link(k, p["K"])
    link(a, p["A"])
    return p


def TP(net, ref=None):
    p = place(TTP, ref, "TP", "5015", "Keystone", "TestPoint:Keystone_5015", "SMT test point", "", f"test point {net}",
              "Sn", "not checked")
    link(net, p["TP"])


def CONN(name, n, prefix="J"):
    return sym(name, prefix, " ".join(f"{i}:P{i}:P" for i in range(1, n + 1)))


def ic(tm, ref, value, mpn, mfr, pkg, rating, mapping, stress="", notes="", finish="NiPdAu",
       checked="TI orderable addendum read: Active", dnp=False):
    p = place(tm, ref, value, mpn, mfr, pkg, rating, stress, notes, finish, checked, dnp)
    conn(p, mapping)
    return p


# ------------------------------------------------------------------------------------------------- symbols
LQFP144 = ("1:PE2 2:PE3 3:PE4 4:PE5 5:PE6 6:VBAT 7:PC13 8:PC14 9:PC15 10:PF0 11:PF1 12:PF2 13:PF3 14:PF4 15:PF5 "
           "16:VSS 17:VDD 18:PF6 19:PF7 20:PF8 21:PF9 22:PF10 23:PH0 24:PH1 25:NRST 26:PC0 27:PC1 28:PC2 29:PC3 "
           "30:VSSA 31:VREF- 32:VREF+ 33:VDDA 34:PA0 35:PA1 36:PA2 37:PA3 38:VSS 39:VDD 40:PA4 41:PA5 42:PA6 43:PA7 "
           "44:PC4 45:PC5 46:PB0 47:PB1 48:PB2 49:PF11 50:PF12 51:VSS 52:VDD 53:PF13 54:PF14 55:PF15 56:PG0 57:PG1 "
           "58:PE7 59:PE8 60:PE9 61:VSS 62:VDD 63:PE10 64:PE11 65:PE12 66:PE13 67:PE14 68:PE15 69:PB10 70:PB11 71:VSS "
           "72:VDD 73:PB12 74:PB13 75:PB14 76:PB15 77:PD8 78:PD9 79:PD10 80:PD11 81:PD12 82:PD13 83:VSS 84:VDD "
           "85:PD14 86:PD15 87:PG2 88:PG3 89:PG4 90:PG5 91:PG6 92:PG7 93:PG8 94:VSS 95:VDDIO2 96:PC6 97:PC7 98:PC8 "
           "99:PC9 100:PA8 101:PA9 102:PA10 103:PA11 104:PA12 105:PA13 106:VDDUSB 107:VSS 108:VDD 109:PA14 110:PA15 "
           "111:PC10 112:PC11 113:PC12 114:PD0 115:PD1 116:PD2 117:PD3 118:PD4 119:PD5 120:VSS 121:VDD 122:PD6 "
           "123:PD7 124:PG9 125:PG10 126:PG11 127:PG12 128:PG13 129:PG14 130:VSS 131:VDDIO2 132:PG15 133:PB3 134:PB4 "
           "135:PB5 136:PB6 137:PB7 138:PH3 139:PB8 140:PB9 141:PE0 142:PE1 143:VSS 144:VDD")
_mcu_pins = []
for tok in LQFP144.split():
    num, nm = tok.split(":")
    uniq = f"{nm}_{num}" if nm in ("VSS", "VDD", "VDDIO2") else nm.replace("+", "P").replace("-", "M")
    ty = "W" if nm.startswith(("VDD", "VSS", "VBAT", "VREF")) else "P"
    _mcu_pins.append(f"{num}:{uniq}:{ty}")
S_MCU = sym("STM32L4R5ZIT6", "U", " ".join(_mcu_pins))
S_TPS3850 = sym("TPS3850H33-Q1", "U", "1:VDD:W 2:CWD:P 3:SET0:I 4:CRST:P 5:GND:W 6:SET1:I 7:WDI:I 8:WDO:OC 9:RESET:OC "
                "10:SENSE:I 11:PAD:W")
S_FRAM = sym("FM25V20A", "U", "1:CS:I 2:SO:TS 3:WP:I 4:VSS:W 5:SI:I 6:SCK:I 7:HOLD:I 8:VDD:W")
S_RTC = sym("RV-3028-C7", "U", "1:CLKOUT:O 2:INT:OC 3:SCL:I 4:SDA:B 5:VSS:W 6:VBACKUP:P 7:VDD:W 8:EVI:I")
S_XO = sym("SG-8018CA", "Y", "1:OE:I 2:GND:W 3:OUT:O 4:VCC:W")
S_XTAL = sym("XTAL32K", "Y", "1:X1:P 2:X2:P")
S_TPS2553 = sym("TPS2553-Q1", "U", "1:IN:W 2:GND:W 3:EN:I 4:FAULT:OC 5:ILIM:P 6:OUT:WO")
S_TXU0304 = sym("TXU0304-Q1", "U", "1:VCCA:W 2:A1:I 3:A2:I 4:A3:I 5:A4Y:O 6:NC6:N 7:GND:W 8:OE:I 9:NC9:N 10:B4:I "
                "11:B3Y:O 12:B2Y:O 13:B1Y:O 14:VCCB:W 15:PAD:P")
S_TXU0204 = sym("TXU0204-Q1", "U", "1:VCCA:W 2:A1:I 3:A2:I 4:A3Y:O 5:A4Y:O 6:NC6:N 7:GND:W 8:OE:I 9:NC9:N 10:B4:I "
                "11:B3:I 12:B2Y:O 13:B1Y:O 14:VCCB:W 15:PAD:P")
S_LVC1T45 = sym("SN74LVC1T45-Q1", "U", "1:VCCA:W 2:GND:W 3:A:B 4:B:B 5:DIR:I 6:VCCB:W")
S_TCA9517 = sym("TCA9517-Q1", "U", "1:VCCA:W 2:SCLA:B 3:SDAA:B 4:GND:W 5:EN:I 6:SDAB:B 7:SCLB:B 8:VCCB:W")
S_MG370 = sym("M-G370PDT0", "U", "1:SCLK:I 2:SDO:TS 3:GND3:W 4:GND4:W 5:SDI:I 6:CSN:I 7:SOUT:N 8:GND8:W 9:SIN:I "
              "10:VCC10:W 11:VCC11:W 12:VCC12:W 13:DRDY:O 14:GPIO2:N 15:GND15:W 16:RSTN:I 17:NC17:N 18:NC18:N "
              "19:NC19:N 20:NC20:N")
S_BMI088 = sym("BMI088", "U", "1:INT2:N 2:NC2:P 3:VDD:W 4:GNDA:W 5:CSB2:I 6:GNDIO:W 7:PS:I 8:SCK:I 9:SDI:I 10:SDO2:TS "
               "11:VDDIO:W 12:INT3:O 13:INT4:N 14:CSB1:I 15:SDO1:TS 16:INT1:N")
S_MAGI2C = sym("PNI13156", "U", "1:SDA:B 2:RES2:P 3:SA0:I 4:AVDD:W 5:AVSS:W 6:ZDRVP:P 7:ZINP:P 8:ZINN:P 9:ZDRVN:P "
               "10:YDRVP:P 11:YINP:P 12:YINN:P 13:YDRVN:P 14:DVDD14:W 15:XDRVP:P 16:XINP:P 17:XINN:P 18:XDRVN:P "
               "19:DVSS:W 20:RES20:P 21:NC21:N 22:I2CEN:I 23:DRDY:O 24:NC24:N 25:REXT:P 26:DVDD26:W 27:SCL:I 28:SA1:I")
S_ADS7138 = sym("ADS7138-Q1", "U", "1:AIN2:P 2:AIN3:P 3:AIN4:P 4:AIN5:P 5:AIN6:P 6:AIN7:P 7:AVDD:W 8:DECAP:P 9:GND:W "
                "10:DVDD:W 11:ADDR:P 12:ALERT:N 13:SCL:I 14:SDA:B 15:AIN0:P 16:AIN1:P 17:PAD:W")
S_OPA2 = sym("TLV9002-Q1", "U", "1:OUTA:O 2:INAM:I 3:INAP:I 4:VM:W 5:INBP:I 6:INBM:I 7:OUTB:O 8:VP:W")
S_TMP235 = sym("TMP235-Q1", "U", "1:VDD:W 2:VOUT:O 3:GND:W")
S_DRV8876 = sym("DRV8876-Q1", "U", "1:EN_IN1:I 2:PH_IN2:I 3:NSLEEP:I 4:NFAULT:OC 5:VREF:I 6:IPROPI:P 7:IMODE:P 8:OUT1:P "
                "9:PGND:W 10:OUT2:P 11:VM:W 12:VCP:P 13:CPH:P 14:CPL:P 15:GND:W 16:PMODE:P 17:PAD:W")
S_HC4060 = sym("CD74HC4060", "U", "1:Q12:N 2:Q13:N 3:Q14:N 4:Q6:N 5:Q5:N 6:Q7:N 7:Q4:N 8:GND:W 9:CTC:P 10:RTC:P "
               "11:RS:P 12:MR:I 13:Q9:N 14:Q8:N 15:Q10:O 16:VCC:W")
S_LVC2G04 = sym("SN74LVC2G04", "U", "1:A1:I 2:GND:W 3:A2:I 4:Y2:O 5:VCC:W 6:Y1:O")
S_LVC1G08 = sym("SN74LVC1G08-Q1", "U", "1:A:I 2:B:I 3:GND:W 4:Y:O 5:VCC:W")
S_ESD4 = sym("TPD4E05U06-Q1", "D", "1:IO1:P 2:IO2:P 3:GND3:P 4:IO3:P 5:IO4:P 6:NC6:N 7:NC7:N 8:GND8:P 9:NC9:N 10:NC10:N")
S_ESD1 = sym("TPD1E05U06-Q1", "D", "1:IO:P 2:GND:P")

# ------------------------------------------------------------------------------------------------- MCU pin allocation
# pin -> (net, function / peripheral, direction, level, default at reset, goes to)
# Directions: O out, I in, A analog in, B bidir, P power, - not used (analog, no connection).
PINMAP = OrderedDict()


def pm(pin, net, func, d, level, reset, dest):
    PINMAP[pin] = (net, func, d, level, reset, dest)


HZ = "analog (high-Z)"
# --- port A
pm("PA0", "TX_TEMP", "ADC1_IN5", "A", "0.3-1.7 V; open line reads 0 V (1 M pull-down)", HZ, "J2-11 RF board (10 k series, 10 nF, 1 M pull-down: SMA-B19)")
pm("PA1", "VSW1_RB", "ADC1_IN6", "A", "0-2.8 V (VBAT_SW1 / 6)", HZ, "J1-37 EPS (new; was DEPLOY_EN)")
pm("PA2", "PT_NTC", "ADC1_IN7", "A", "VDDA x R/(10.0 k + R): 2.93 V at -15 C, 0.49 V at 70 C", HZ,
   "J3-5 Jetson spreader NTC (new, E10; was JETSON_PWR)")
pm("PA3", "RF_TX_EN", "GPIO out", "O", "3.3 V, high = PA rail (TX-OFF low)", HZ + "; EPS 100 k pull-down", "J1-14 EPS")
pm("PA4", "MTQ_X_I", "ADC1_IN9", "A", "10 mV/mA (IPROPI)", HZ, "U22 X bridge (was FRAM CS#)")
pm("PA5", "MTQ_Y_I", "ADC1_IN10", "A", "10 mV/mA", HZ, "U23 Y bridge (was SPI1 SCK)")
pm("PA6", "MTQ_Z_I", "ADC1_IN11", "A", "10 mV/mA", HZ, "U24 Z bridge (was SPI1 MISO)")
pm("PA7", "VJET_RB", "ADC1_IN12", "A", "0-2.8 V (VBAT_JET / 6)", HZ, "J1-19 EPS (new; was SPI1 MOSI)")
pm("PA8", "JETSON_PWR", "GPIO out", "O", "3.3 V, high = eFuse U19 on", HZ + "; EPS 100 k pull-down", "J1-29 EPS (moved from PA2)")
pm("PA9", "USART1_TX", "USART1_TX AF7", "O", "3.3 V UART 115200", HZ, "U28 -> J3-2 Jetson")
pm("PA10", "USART1_RX", "USART1_RX AF7", "I", "3.3 V UART", HZ + "; 100 k pull-up", "U29 <- J3-3 Jetson")
pm("PA11", None, "spare (USB DM unused)", "-", "", HZ, "")
pm("PA12", None, "spare (USB DP unused)", "-", "", HZ, "")
pm("PA13", "SWDIO", "SWDIO AF0", "B", "3.3 V", "SWDIO, pull-up", "J9-2 access port")
pm("PA14", "SWCLK", "SWCLK AF0", "I", "3.3 V", "SWCLK, pull-down", "J9-4 access port")
pm("PA15", None, "spare (JTDI)", "-", "", "JTDI, pull-up: do not use as an enable", "")
# --- port B
pm("PB0", "VBUS_SNS", "ADC1_IN15", "A", "0-2.5 V (5V_PA / 2)", HZ, "J1-26 EPS")
pm("PB1", "TEMP_BOARD", "ADC1_IN16", "A", "10 mV/C, 0.5 V at 0 C", HZ, "U21 TMP235 on the OBC")
pm("PB2", "PAYLOAD_EN", "GPIO out", "O", "3.3 V, high = SW1 on", HZ + "; EPS 100 k pull-down", "J1-9 EPS (moved from PB4)")
pm("PB3", "SWO", "spare (JTDO/SWO)", "-", "", "JTDO", "TP only")
pm("PB4", "I2C3_SDA", "I2C3_SDA AF4", "B", "open drain, 2 x 4.7 k on the OBC, 100 kHz", "NJTRST with its internal pull-up until configured (harmless on an open-drain bus)", "J1-28 EPS: AntS bus via U80 (SMA-B07)")
pm("PB5", "RADIO_EN", "GPIO out", "O", "3.3 V, high = SW2 on", HZ + "; EPS 100 k pull-down", "J1-10 EPS")
pm("PB6", "ADCS_EN", "GPIO out", "O", "3.3 V, high = SW4 + VBAT_MTQ switch on", HZ + "; EPS 100 k pull-down", "J1-11 EPS")
pm("PB7", "EPS_ALERT_N", "GPIO in, EXTI7", "I", "open drain, active low (EPS 4.7 k)", HZ + "; OBC 1 M pull-up (open reads no alert)", "J1-8 EPS (10 k series)")
pm("PB8", "UMBILICAL_SENSE", "GPIO in", "I", "high = GSE plug present", HZ + "; 100 k pull-down", "J9-9 access port")
pm("PB9", "BURN_W2A", "GPIO out", "O", "3.3 V, high = burn (12 s hardware limit)", HZ + "; EPS 100 k pull-down", "J1-17 EPS (was ANTENNA_BURN)")
pm("PB10", "BURN_W1A", "GPIO out", "O", "3.3 V, high = burn", HZ + "; EPS 100 k pull-down", "J1-18 EPS (was SOLAR_BURN)")
pm("PB11", "LPUART1_TX", "LPUART1_TX AF8", "O", "3.3 V UART 9600", HZ, "U30 -> J5-3 GNSS (moved from PG7)")
pm("PB12", "SOLAR_SENSE", "GPIO in", "I", "low = both wings deployed", HZ + "; EPS 10 k pull-up, OBC 1 M pull-up (open reads high)", "J1-20 EPS (10 k series)")
pm("PB13", "IMU2_SCK", "SPI2_SCK AF5", "O", "3.3 V, <= 10 MHz", HZ, "U12 -> BMI088")
pm("PB14", "IMU2_MISO", "SPI2_MISO AF5", "I", "3.3 V", HZ + "; 100 k pull-down", "U12 <- BMI088 SDO1/SDO2")
pm("PB15", "IMU2_MOSI", "SPI2_MOSI AF5", "O", "3.3 V", HZ, "U12 -> BMI088")
# --- port C
pm("PC0", "TX_FWD", "ADC1_IN1", "A", "0-2.5 V", HZ, "J2-12 RF board")
pm("PC1", "TX_REV", "ADC1_IN2", "A", "0-2.5 V", HZ, "J2-13 RF board")
pm("PC2", "VBATT_SNS", "ADC1_IN3", "A", "0-2.8 V (pack / 6)", HZ, "J1-22 EPS")
pm("PC3", "IBATT_SNS", "ADC1_IN4", "A", "0-3.3 V", HZ, "J1-23 EPS")
pm("PC4", "VSOLAR_SNS", "ADC1_IN13", "A", "0-3.3 V (/4)", HZ, "J1-24 EPS")
pm("PC5", "ISOLAR_SNS", "ADC1_IN14", "A", "0-3.3 V", HZ, "J1-25 EPS")
pm("PC6", "GNSS_PPS", "TIM3_CH1 AF2 capture", "I", "3.3 V pulse", HZ + "; 100 k pull-down", "U32 <- J5-5 GNSS")
pm("PC7", "PTT_EXC_RB", "TIM8_CH2 AF3 capture", "I", "3.1 V CMOS, high = keyed", HZ + "; 100 k pull-down", "J2-10 RF board (10 k series)")
pm("PC8", "RTC_1HZ", "TIM3_CH3 AF2 capture", "I", "RTC CLKOUT (1 Hz once set)", HZ, "U4 RV-3028 CLKOUT (1 M series)")
pm("PC9", "TONE_OK", "GPIO in, EXTI9", "I", "3.1 V CMOS, high = CTCSS", HZ + "; 100 k pull-down", "J2-8 RF board (10 k series)")
pm("PC10", "GYRO_SCK", "SPI3_SCK AF6", "O", "3.3 V, mode 3, <= 1 MHz burst", HZ + "; 100 k pull-up", "U9 -> M-G370")
pm("PC11", "GYRO_MISO", "SPI3_MISO AF6", "I", "3.3 V", HZ + "; 100 k pull-down", "U9 <- M-G370")
pm("PC12", "GYRO_MOSI", "SPI3_MOSI AF6", "O", "3.3 V", HZ, "U9 -> M-G370")
pm("PC13", None, "spare", "-", "", HZ, "")
pm("PC14", "LSE_IN", "OSC32_IN", "-", "32.768 kHz", "LSE off", "Y2")
pm("PC15", "LSE_OUT", "OSC32_OUT", "-", "32.768 kHz", "LSE off", "Y2")
# --- port D
pm("PD0", "IMU2_GCS_N", "GPIO out (SPI2 CS)", "O", "3.3 V, low = select", HZ + "; 100 k pull-up", "U12 -> BMI088 CSB2")
pm("PD1", "GYRO_CS_N", "GPIO out (SPI3 CS)", "O", "3.3 V, low = select", HZ + "; 100 k pull-up", "U9 -> M-G370 /CS")
pm("PD2", "HEATER_EN", "GPIO out", "O", "3.3 V, high = SW5 on", HZ + "; EPS 100 k pull-down", "J1-12 EPS")
pm("PD3", "LED_STATUS", "GPIO out", "O", "3.3 V", HZ, "D1 (DNP in flight)")
pm("PD4", "IMU2_ACS_N", "GPIO out (SPI2 CS2)", "O", "3.3 V, low = select", HZ + "; 100 k pull-up", "U13 -> BMI088 CSB1")
pm("PD5", "USART2_TX", "USART2_TX AF7", "O", "3.3 V UART 9600", HZ, "J2-2 RF TNC")
pm("PD6", "USART2_RX", "USART2_RX AF7", "I", "3.1 V UART", HZ + "; 100 k pull-up", "J2-3 RF TNC (10 k series)")
pm("PD7", "FRAM2_CS_N", "GPIO out", "O", "3.3 V, low = second FRAM selected", HZ + "; 10 k pull-up", "U36 FM25V20A CS# (SMA-B10)")
pm("PD8", "USART3_TX", "USART3_TX AF7", "O", "3.3 V UART 115200", HZ, "J9-7 access port")
pm("PD9", "USART3_RX", "USART3_RX AF7", "I", "3.3 V UART", HZ + "; 100 k pull-up", "J9-8 access port (10 k series)")
pm("PD10", "GYRO_DRDY", "GPIO in, EXTI10", "I", "3.3 V", HZ + "; 100 k pull-down", "U10 <- M-G370 DRDY")
pm("PD11", "MAG_DRDY", "GPIO in, EXTI11", "I", "3.3 V", HZ + "; 100 k pull-down", "U33 <- J10-5 magnetometer board (RM3100 DRDY)")
pm("PD12", "I2C4_SCL", "I2C4_SCL AF4", "B", "open drain, 2.2 k", HZ, "U14 -> ADS7138; J10 -> RM3100 board")
pm("PD13", "I2C4_SDA", "I2C4_SDA AF4", "B", "open drain, 2.2 k", HZ, "U14 -> ADS7138; J10 -> RM3100 board")
pm("PD14", "GYRO_RST_N", "GPIO out", "O", "3.3 V, low = reset (>= 100 ms)", HZ + "; 100 k pull-down", "U10 -> M-G370 /RST")
pm("PD15", "IMU2_DRDY", "GPIO in, EXTI15", "I", "3.3 V", HZ + "; 100 k pull-down", "U13 <- BMI088 INT3")
# --- port E
pm("PE0", "FRAM_WP_N", "GPIO out", "O", "3.3 V, high = writes allowed", HZ + "; 10 k pull-up", "U3")
pm("PE1", "FRAM_HOLD_N", "GPIO out", "O", "3.3 V, high", HZ + "; 10 k pull-up", "U3")
pm("PE2", "DEPLOY_SENSE", "GPIO in, EXTI2", "I", "high = DS2 released (1 M source); no internal pull", HZ + "; 10 M pull-down (open reads low)", "J1-16 EPS (10 k series)")
pm("PE3", "TX_PWR_SEL", "GPIO out", "O", "3.3 V, low 0.5 W / high 1 W", HZ + "; RF 100 k pull-down", "J2-5 RF board")
pm("PE4", "XPDR_EN", "GPIO out (TIM7 heartbeat)", "O", "100 Hz square wave", HZ + "; RF 100 k pull-down", "J2-6 RF board")
pm("PE5", "GNSS_EN", "GPIO out", "O", "3.3 V, high = SW7 on", HZ + "; EPS 100 k pull-down", "J1-15 EPS")
pm("PE6", "BURN_W1B", "GPIO out", "O", "3.3 V, high = burn", HZ + "; EPS 100 k pull-down", "J1-30 EPS (was SOLAR_BURN_B)")
pm("PE7", "BURN_W2B", "GPIO out", "O", "3.3 V, high = burn", HZ + "; EPS 100 k pull-down", "J1-31 EPS (new)")
pm("PE8", "MTQ_X_DIR", "GPIO out", "O", "3.3 V, PH", HZ + "; DRV pull-down", "U22 PH")
pm("PE9", "MTQ_X_PWM", "TIM1_CH1 AF1", "O", "3.3 V PWM 40 kHz", HZ + "; DRV pull-down", "U22 EN")
pm("PE10", "MTQ_Y_DIR", "GPIO out", "O", "3.3 V, PH", HZ + "; DRV pull-down", "U23 PH")
pm("PE11", "MTQ_Y_PWM", "TIM1_CH2 AF1", "O", "3.3 V PWM 40 kHz", HZ + "; DRV pull-down", "U23 EN")
pm("PE12", "MTQ_Z_DIR", "GPIO out", "O", "3.3 V, PH", HZ + "; DRV pull-down", "U24 PH")
pm("PE13", "MTQ_Z_PWM", "TIM1_CH3 AF1", "O", "3.3 V PWM 40 kHz", HZ + "; DRV pull-down", "U24 EN")
pm("PE14", "MTQ_FAULT_N", "GPIO in, EXTI14", "I", "open drain, 10 k pull-up", HZ, "U22-U24 nFAULT")
pm("PE15", "MTQ_EXPIRED", "GPIO in", "I", "high = hardware off-timer expired", HZ, "U25 Q10")
# --- port F
pm("PF0", "I2C2_SDA", "I2C2_SDA AF4", "B", "open drain, 2 x 4.7 k on the OBC", HZ, "J1-6 EPS (0x40, 0x44, 0x48; heartbeat register in 0x48)")
pm("PF1", "I2C2_SCL", "I2C2_SCL AF4", "B", "open drain, 2 x 4.7 k on the OBC", HZ, "J1-5 EPS")
pm("PF2", None, "spare", "-", "", HZ, "")
pm("PF3", "DEPLOY_EN", "GPIO out", "O", "3.3 V, high = SW6/SW8 on (master arm)", HZ + "; EPS 100 k pull-down", "J1-13 EPS (moved from PA1)")
pm("PF4", "EOL_ARM", "GPIO out", "O", "3.3 V, rising edge sets the arm latch", HZ + "; EPS 100 k pull-down", "J1-32 EPS (new)")
pm("PF5", "LOCK_CLR", "GPIO out", "O", "3.3 V, high >= 100 ms with UNLOCK", HZ + "; EPS 100 k pull-down", "J1-34 EPS (new)")
pm("PF6", "GYRO_EN", "GPIO out", "O", "3.3 V, high = 3V3_GYRO", HZ + "; 100 k pull-down", "U5 (100 k + 2.2 uF retry)")
pm("PF7", "IMU2_EN", "GPIO out", "O", "3.3 V, high = 3V3_IMU2", HZ + "; 100 k pull-down", "U6")
pm("PF8", "SNS_EN", "GPIO out", "O", "3.3 V, high = 3V3_SNS", HZ + "; 100 k pull-down", "U7")
pm("PF9", "MTQ_ARM", "TIM15_CH1 AF14, one-pulse 900 ms", "O", "3.3 V, high = coil burst", HZ + "; 100 k pull-down", "U26/U27 -> nSLEEP")
pm("PF10", "GYRO_RB", "GPIO in", "I", "low with GYRO_EN high = latch-up trip", HZ, "U5 EN node (100 k)")
pm("PF11", "IMU2_RB", "GPIO in", "I", "as GYRO_RB", HZ, "U6 EN node (100 k)")
pm("PF12", "SNS_RB", "GPIO in", "I", "as GYRO_RB", HZ, "U7 EN node (100 k)")
pm("PF13", "RX_FAULT", "GPIO in, EXTI13", "I", "3.1 V CMOS, high = receiver fault", HZ + "; 100 k pull-down (unwired or radio off reads healthy)", "J2-16 RF board (10 k series; new, FSW round 2)")
pm("PF14", "MAG_EN", "GPIO out", "O", "3.3 V, high = 3V3_MAG on", HZ + "; 100 k pull-down", "U35 TPS2553-Q1 (magnetometer switch, SMA-B18)")
pm("PF15", "MAG_RB", "GPIO in", "I", "low with MAG_EN high = latch-up trip", HZ, "U35 EN node (100 k)")
# --- port G (PG2-PG15 on VDDIO2)
pm("PG0", "RX_CD", "GPIO in, EXTI0", "I", "3.1 V CMOS, high = receiver carrier detect", HZ + "; 100 k pull-down", "J2-15 RF board (10 k series; new, FSW round 2)")
pm("PG1", "EOL_FIRE", "GPIO out", "O", "3.3 V, high = fire (>= 60 s)", HZ + "; EPS 100 k pull-down", "J1-33 EPS (new)")
pm("PG2", "FRAM_SCK", "SPI1_SCK AF5", "O", "3.3 V, 20 MHz mode 0", HZ, "U3 (moved from PA5)")
pm("PG3", "FRAM_MISO", "SPI1_MISO AF5", "I", "3.3 V", HZ, "U3 (moved from PA6)")
pm("PG4", "FRAM_MOSI", "SPI1_MOSI AF5", "O", "3.3 V", HZ, "U3 (moved from PA7)")
pm("PG5", "FRAM_CS_N", "GPIO out", "O", "3.3 V, low = select", HZ + "; 10 k pull-up", "U3 (moved from PA4)")
pm("PG6", "JET_PG", "GPIO in, EXTI6", "I", "high = carrier 3.3 V up", HZ + "; 100 k pull-down", "J3-4 Jetson (10 k series)")
pm("PG7", "I2C3_SCL", "I2C3_SCL AF4", "B", "open drain, 2 x 4.7 k on the OBC, 100 kHz", HZ, "J1-27 EPS: AntS bus via U80 (SMA-B07; LPUART1_TX moved to PB11)")
pm("PG8", "LPUART1_RX", "LPUART1_RX AF8", "I", "3.3 V UART", HZ + "; 100 k pull-up", "U31 <- J5-4 GNSS")
pm("PG9", None, "spare", "-", "", HZ, "")
pm("PG10", None, "spare", "-", "", HZ, "")
pm("PG11", "WDI", "GPIO out", "O", "falling edge every 10 s", HZ + "; 100 k pull-down", "U2 TPS3850 WDI")
pm("PG12", "XPDR_KEY", "GPIO in, EXTI12", "I", "3.1 V CMOS, high = repeat keyed", HZ + "; 100 k pull-down", "J2-9 RF board (10 k series)")
pm("PG13", "I2C1_SDA", "I2C1_SDA AF4", "B", "open drain, 4.7 k", HZ, "U4 RV-3028 (0x52)")
pm("PG14", "I2C1_SCL", "I2C1_SCL AF4", "B", "open drain, 4.7 k", HZ, "U4 RV-3028")
pm("PG15", "GNSS_PG", "GPIO in", "I", "high = 3V3_GNSS up", HZ + "; 100 k pull-down", "J4 3V3_GNSS (10 k series)")
# --- port H, system
pm("PH0", "HSE_IN", "OSC_IN (HSE bypass)", "I", "8 MHz CMOS", "HSE off", "Y1 XO")
pm("PH1", None, "spare (OSC_OUT unused in bypass)", "-", "", HZ, "")
pm("PH3", "BOOT0", "BOOT0", "I", "low = boot from flash", "BOOT0", "10 k pull-down")
pm("NRST", "NRST", "NRST", "B", "open drain wired-OR", "low until released", "U2 RESET/WDO, J9-6, U4 EVI")

SPARES = [p for p, v in PINMAP.items() if v[0] is None]


# ------------------------------------------------------------------------------------------------- sections
def mcu():
    u = place(S_MCU, "U1", "STM32L4R5ZIT6", "STM32L4R5ZIT6", "STMicroelectronics", "LQFP-144 20x20 mm",
              "Cortex-M4F 120 MHz (80 MHz used), 2 MB flash, 640 KB SRAM, 1.71-3.6 V, -40..85 C",
              f"Tj {cv('tj_mcu', '{:.0f}')} C at a 60 C board (66 mW)", "OBC processor (BASELINE 6)",
              "NiPdAu (ST LQFP: verify in the material declaration)", "ST site not reachable from this session; EEE_PARTS_LIST 4.1")
    for pinname, (net, *_rest) in PINMAP.items():
        key = pinname
        if net is None:
            u[key] += builtins.NC
        elif net == "SWO":
            link("SWO", u[key])
        else:
            link(net, u[key])
    for p in u.pins:
        if p.name.startswith(("VDD_", "VDDIO2", "VDDUSB", "VBAT")):
            link("3V3_OBC", p)
        elif p.name.startswith("VSS"):
            link("GND", p)
    link("VDDA", u["VDDA"])
    link("VDDA", u["VREFP"])
    link("GND", u["VSSA"])
    link("GND", u["VREFM"])
    TP("SWO")
    # supply: input protection, bulk, per-pin decoupling (ST AN5200 layout guide pattern)
    D("SMF5.0A", "3V3_OBC", "GND", ref="D2", note="3V3_OBC input TVS (J1-1/2)", stress="3.4 V / 5.0 V VRWM = 68 %")
    C("22u", "3V3_OBC", "GND", 16, "1210", note="input bulk (inrush 1.0 ms at the SW10 limit, below its 8 ms deglitch)")
    C("10u", "3V3_OBC", "GND", 25, "1206", note="MCU bulk")
    for i in range(9):
        C("100n", "3V3_OBC", "GND", note="VDD pin decoupling (one per VDD pin)")
    for i in range(2):
        C("100n", "3V3_OBC", "GND", note="VDDIO2 decoupling (port G)")
    C("100n", "3V3_OBC", "GND", note="VDDUSB (USB unused: tied to VDD)")
    C("100n", "3V3_OBC", "GND", note="VBAT pin (no backup cell: the mission clock is U4)")
    p = place(TFB, "FB1", "600R@100MHz", "BLM18AG601SN1D", "Murata", "Inductor_SMD:L_0603", "600 ohm at 100 MHz, 0.5 A",
              "< 5 mA: 1 %", "VDDA / VREF+ filter", "Sn: coat", "not checked (automotive BLM18 variant to confirm)")
    link("3V3_OBC", p["A"])
    link("VDDA", p["B"])
    C("1u", "VDDA", "GND", note="VDDA")
    C("10nC0G", "VDDA", "GND", note="VDDA HF")
    C("1u", "VDDA", "GND", note="VREF+")
    C("100n", "VDDA", "GND", note="VREF+ HF")
    R(10e3, "BOOT0", "GND", note="BOOT0 low: boot from flash")
    TP("3V3_OBC")
    TP("GND")
    TP("VDDA")


def clocks():
    ic(S_XO, "Y1", "8.000 MHz", "SG-8018CA 8.0000M-TJHPA0", "Epson", "SMD 7.0 x 5.0 mm 4-pad",
       "CMOS XO 1.62-3.63 V, +/-50 ppm, -40..105 C", {"OE": "XO_OE", "GND": "GND", "OUT": "XO_OUT", "VCC": "3V3_OBC"},
       notes="HSE bypass into PH0: PLL M 1, N 20, R 2 = 80 MHz (firmware change from MSI; MSI stays the CSS fallback)",
       finish="Au (Epson XO)", checked="not checked (part code built from the EPS board's SG-8018CA order code)")
    R(10e3, "3V3_OBC", "XO_OE", note="XO always enabled")
    R(33, "XO_OUT", "HSE_IN", note="series termination")
    C("100n", "3V3_OBC", "GND", note="XO")
    p = place(S_XTAL, "Y2", "32.768 kHz", "CM7V-T1A-32.768kHz-7pF-20PPM-TA-QA", "Micro Crystal", "SMD 3.2 x 1.5 mm",
              "32.768 kHz, CL 7 pF, +/-20 ppm, ESR 70 k max, -40..85 C, AEC-Q200",
              f"gm_crit {cv('lse_gmcrit', '{:.2f}')} uA/V: {cv('lse_drive_margin', '{:.1f}')}x below LSEDRV medium-high",
              "LSE for the MCU RTC and LSE-trimmed MSI", "Au", "not checked")
    link("LSE_IN", p["X1"])
    link("LSE_OUT", p["X2"])
    C("10p", "LSE_IN", "GND", note="LSE load (CL 7.5 pF with 2.5 pF stray)")
    C("10p", "LSE_OUT", "GND", note="LSE load")


def reset_supervisor():
    ic(S_TPS3850, "U2", "TPS3850H33-Q1", "TPS3850H33QDRCRQ1", "TI", "VSON-10 (DRC) 3x3",
       "supervisor UV/OV +/-7 % of 3.3 V, window watchdog, AEC-Q100 grade 1",
       {"VDD": "3V3_OBC", "CWD": "WD_CWD", "SET0": "WD_SET0", "CRST": None, "GND": "GND", "SET1": "GND",
        "WDI": "WDI", "WDO": "NRST", "RESET": "NRST", "SENSE": "3V3_OBC", "PAD": "GND"},
       stress="VDD 3.4 V / 7 V abs = 49 %",
       notes=(f"RID AV-02: window watchdog {cv('wd_twdl_max', '{:.1f}')} s (max lower) to {cv('wd_twdu_min', '{:.1f}')} s "
              f"(min upper), kick every 10 s; UV 3.069 V, OV 3.531 V; reset 200 ms (CRST open); RESET and WDO wired to NRST"))
    for k in (1, 2):   # SMA-B08: two parallel strings of two in series; one short or one open keeps a working window
        C("470nC0G", "WD_CWD", f"WD_CM{k}", note=f"CWD string {k}, upper (470 nF total: tWDU 36.4 s typ)")
        C("470nC0G", f"WD_CM{k}", "GND", note=f"CWD string {k}, lower; one short -> 705 nF, one open -> 235 nF")
    C("100n", "3V3_OBC", "GND", note="TPS3850 VDD")
    R(100e3, "WD_SET0", "GND", note="SET0 low: watchdog enabled in flight")
    R(10e3, "WDT_DIS", "WD_SET0", note="GSE pulls SET0 high: watchdog disabled (latched at the next supervisor reset)")
    R(100e3, "WDI", "GND", note="WDI defined while the MCU is in reset")
    R(10e3, "3V3_OBC", "NRST", note="NRST pull-up (TPS3850 open-drain outputs)")
    C("100n", "NRST", "GND", note="NRST filter (ST recommendation)")
    TP("NRST")
    TP("WDI")


def fram():
    ic(S_FRAM, "U3", "FM25V20A", "FM25V20A-GTR", "Infineon", "SOIC-8", "2 Mbit SPI FRAM, 40 MHz, 2.0-3.6 V, -40..85 C",
       {"CS": "FRAM_CS_N", "SO": "FRAM_MISO", "WP": "FRAM_WP_N", "VSS": "GND", "SI": "FRAM_MOSI", "SCK": "FRAM_SCK_R",
        "HOLD": "FRAM_HOLD_N", "VDD": "3V3_OBC"}, stress="20 MHz / 40 MHz = 50 %",
       notes="NVM: 12 regions (STORAGE_DESIGN 3.3); on SPI1 PG2-PG5 (moved from PA4-PA7 to free ADC inputs)",
       finish="Sn (matte): re-tin or coat", checked="not checked (Infineon site not read; EEE_PARTS_LIST)")
    R(33, "FRAM_SCK", "FRAM_SCK_R", note="SCK series termination")
    for n in ("FRAM_CS_N", "FRAM_WP_N", "FRAM_HOLD_N"):
        R(10e3, "3V3_OBC", n, note="FRAM control pull-up (deselected, not held, during MCU reset)")
    C("100n", "3V3_OBC", "GND", note="FRAM")
    # second FRAM (SMA-B10): own chip select, its own WP#/HOLD# pull-ups (not shared with U3's driven lines)
    ic(S_FRAM, "U36", "FM25V20A", "FM25V20A-GTR", "Infineon", "SOIC-8", "2 Mbit SPI FRAM, 40 MHz, 2.0-3.6 V, -40..85 C",
       {"CS": "FRAM2_CS_N", "SO": "FRAM_MISO", "WP": "FRAM2_WP_N", "VSS": "GND", "SI": "FRAM_MOSI", "SCK": "FRAM_SCK_R",
        "HOLD": "FRAM2_HOLD_N", "VDD": "3V3_OBC"}, stress="20 MHz / 40 MHz = 50 %",
       notes="mirror NVM (SMA-B10): every record mirrored; read from the mirror when U3 fails; CS# PD7",
       finish="Sn (matte): re-tin or coat", checked="not checked (Infineon site not read; EEE_PARTS_LIST)")
    for n in ("FRAM2_CS_N", "FRAM2_WP_N", "FRAM2_HOLD_N"):
        R(10e3, "3V3_OBC", n, note="second FRAM control pull-up (WP#, HOLD# not driven: no shared control line)")
    C("100n", "3V3_OBC", "GND", note="FRAM 2")


def rtc():
    ic(S_RTC, "U4", "RV-3028-C7", "RV-3028-C7-32.768kHz-1ppm-TA-QA", "Micro Crystal", "C7 3.2 x 1.5 mm 8-pad",
       "RTC, 32-bit UNIX counter, +/-1 ppm at 25 C, 45 nA, 1.1-5.5 V, -40..85 C, AEC-Q200",
       {"CLKOUT": "RTC_CLKOUT", "INT": None, "SCL": "I2C1_SCL", "SDA": "I2C1_SDA", "VSS": "GND", "VBACKUP": "RTC_VB",
        "VDD": "3V3_RTC", "EVI": "RTC_EVI"}, stress="45 nA",
       notes="RID AV-10 mission clock on 3V3_AO (J1-38): survives OBC resets and OBC power cycles; EVI time-stamps every "
             "NRST edge; I2C1 0x52", finish="Au", checked="Micro Crystal application manual Rev 1.4 read (ordering code TA QA)")
    R(10e3, "RTC_VB", "GND", note="VBACKUP unused: 10 k to VSS (application manual 2.2)")
    C("1u", "3V3_RTC", "GND", note="RTC supply hold-up")
    C("100n", "3V3_RTC", "GND", note="RTC")
    R(100e3, "NRST", "RTC_EVI", note="EVI from NRST, 100 k (domain crossing)")
    R(1e6, "RTC_EVI", "GND", note="EVI defined low while the OBC is unpowered")
    R(1e6, "RTC_CLKOUT", "RTC_1HZ", note="CLKOUT to TIM3_CH3, 1 M: 3.3 uA into an unpowered MCU")
    R(4.7e3, "3V3_OBC", "I2C1_SCL", note="I2C1 pull-up")
    R(4.7e3, "3V3_OBC", "I2C1_SDA", note="I2C1 pull-up")


def sensor_switch(ref, sw, en_gpio, rail, rb, note):
    node = f"{sw}_ENN"
    ic(S_TPS2553, ref, "TPS2553-Q1", "TPS2553QDBVRQ1", "TI", "SOT-23-6 (DBV)", "2.5-6.5 V, 85 mOhm, ILIM = IN: 50-100 mA, AEC-Q100",
       {"IN": "3V3_ADCS", "GND": "GND", "EN": node, "FAULT": node, "ILIM": "3V3_ADCS", "OUT": rail},
       stress=f"IN 3.4 V / 6.5 V = 52 %; load {CALC[f'snsw_{sw.lower()}_load']['value'] * 1e3:.0f} mA, limit min 50 mA "
              f"({CALC[f'snsw_{sw.lower()}_margin']['value']:.1f}x)",
       notes=f"{note}: latch-up trip (2 us limit, FAULT after 8 ms pulls EN low, off {cv('snsw_retry_off', '{:.2f}')} s, retry)",
       checked="TI orderable addendum read: Active (EPS board)")
    R(100e3, en_gpio, node, note="R_FAULT (auto-retry)")
    C("2.2u", node, "GND", 25, "0805", note="C_RETRY: >= 0.1 s off after a trip")
    D("BAT54", en_gpio, node, note="fast turn-off when the enable goes low")
    R(100e3, node, rb, note="trip readback to the MCU")
    R(100e3, en_gpio, "GND", note="enable pull-down (MCU reset = rail off)")
    C("1u", "3V3_ADCS", "GND", note=f"{ref} input")
    C("1u", rail, "GND", note=f"{ref} output")


def gyro():
    sensor_switch("U5", "GYRO", "GYRO_EN", "3V3_GYRO", "GYRO_RB", "3V3_GYRO, M-G370PDT0")
    ic(S_MG370, "U8", "M-G370PDT0", "M-G370PDT0", "Epson", "module 24 x 24 x 10 mm, Samtec FTMH-110-02-L-DV-ES header",
       "gyro + accel IMU, bias 0.10 deg/s 1 sigma, 3.15-3.45 V, 16 mA, -40..85 C",
       {"SCLK": "GY_SCLK", "SDO": "GY_SDO", "GND3": "GND", "GND4": "GND", "SDI": "GY_SDI", "CSN": "GY_CSN",
        "SOUT": None, "GND8": "GND", "SIN": "GY_SIN", "VCC10": "3V3_GYRO", "VCC11": "3V3_GYRO", "VCC12": "3V3_GYRO",
        "DRDY": "GY_DRDY", "GPIO2": None, "GND15": "GND", "RSTN": "GY_RSTN", "NC17": None, "NC18": None,
        "NC19": None, "NC20": None},
       stress=f"VCC min {cv('gyro_vcc_min', '{:.3f}')} V vs 3.15 V", notes="primary gyro (D9); plugs into X1; SPI mode 3",
       finish="module", checked="Epson M-G370PDG0 full data sheet Rev 1.2 read (pinout); PDT0 order code to confirm with Epson")
    x = place(CONN("CLM-110-02-L-D", 20, "X"), "X1", "socket 2x10", "CLM-110-02-L-D", "Samtec",
              "1.27 mm 2x10 SMT socket", "1.27 mm, 20 pos", "< 20 mA per contact", "host socket for U8 (Epson table 2.2)",
              "Au", "not checked")
    for i, pin in enumerate(U8_ORDER, 1):
        link(pin, x[f"P{i}"]) if pin else (x[f"P{i}"].__iadd__(builtins.NC))
    R(10e3, "3V3_GYRO", "GY_SIN", note="unused UART input to VCC (Epson table 1.6 note 2)")
    C("10u", "3V3_GYRO", "GND", 25, "1206", note="gyro bulk")
    C("100n", "3V3_GYRO", "GND", note="gyro")
    ic(S_TXU0304, "U9", "TXU0304-Q1", "TXU0304QPWRQ1", "TI", "TSSOP-14 (PW)", "4-bit translator 3 A->B / 1 B->A, Ioff, VCC isolation, AEC-Q100",
       {"VCCA": "3V3_OBC", "A1": "GYRO_SCK", "A2": "GYRO_MOSI", "A3": "GYRO_CS_N", "A4Y": "GYRO_MISO", "NC6": None,
        "GND": "GND", "OE": "3V3_GYRO", "NC9": None, "B4": "GY_SDO", "B3Y": "GY_CSN", "B2Y": "GY_SDI", "B1Y": "GY_SCLK",
        "VCCB": "3V3_GYRO", "PAD": "GND"},
       notes="gyro SPI isolation: B side unpowered = high-Z, no back-power into the switched-off gyro")
    ic(S_TXU0204, "U10", "TXU0204-Q1", "TXU0204QPWRQ1", "TI", "TSSOP-14 (PW)", "4-bit translator 2 A->B / 2 B->A, Ioff, AEC-Q100",
       {"VCCA": "3V3_OBC", "A1": "GYRO_RST_N", "A2": "3V3_OBC", "A3Y": "GYRO_DRDY", "A4Y": None, "NC6": None,
        "GND": "GND", "OE": "3V3_GYRO", "NC9": None, "B4": "GND", "B3": "GY_DRDY", "B2Y": None, "B1Y": "GY_RSTN",
        "VCCB": "3V3_GYRO", "PAD": "GND"}, notes="gyro /RST and DRDY isolation (spare A2 tied high, B4 tied low)")
    C("100n", "3V3_OBC", "GND", note="U9/U10 VCCA")
    C("100n", "3V3_GYRO", "GND", note="U9/U10 VCCB")
    R(100e3, "3V3_OBC", "GYRO_SCK", note="SCK idle high (mode 3) during MCU reset")
    R(100e3, "3V3_OBC", "GYRO_CS_N", note="CS deselected during MCU reset")
    R(100e3, "GYRO_MOSI", "GND", note="defined input")
    R(100e3, "GYRO_MISO", "GND", note="defined while U9 is isolated")
    R(100e3, "GYRO_RST_N", "GND", note="gyro held in reset while the MCU is in reset")
    R(100e3, "GYRO_DRDY", "GND", note="defined while U10 is isolated")


U8_ORDER = ["GY_SCLK", "GY_SDO", "GND", "GND", "GY_SDI", "GY_CSN", None, "GND", "GY_SIN", "3V3_GYRO", "3V3_GYRO",
            "3V3_GYRO", "GY_DRDY", None, "GND", "GY_RSTN", None, None, None, None]


def imu2():
    sensor_switch("U6", "IMU2", "IMU2_EN", "3V3_IMU2", "IMU2_RB", "3V3_IMU2, BMI088 cold spare")
    ic(S_BMI088, "U11", "BMI088", "BMI088", "Bosch Sensortec", "LGA-16 3.0 x 4.5 mm",
       "6-axis IMU (gyro +/-2000 deg/s), VDD 2.4-3.6 V, 5 mA, -40..85 C",
       {"INT2": None, "NC2": "GND", "VDD": "3V3_IMU2", "GNDA": "GND", "CSB2": "IM_CSB2", "GNDIO": "GND", "PS": "GND",
        "SCK": "IM_SCK", "SDI": "IM_SDI", "SDO2": "IM_SDO", "VDDIO": "3V3_IMU2", "INT3": "IM_INT3", "INT4": None,
        "CSB1": "IM_CSB1", "SDO1": "IM_SDO", "INT1": None},
       stress="VDD 3.4 V / 3.6 V operating max", notes="backup gyro (D9), powered only if the M-G370 fails; PS = GND: gyro "
       "in SPI; SDO1/SDO2 joined (tri-state while their CSB is high); calibrated on a rate table before integration",
       finish="Au (LGA)", checked="Bosch data sheet BST-BMI088-DS000-19 read (pinout)")
    C("100n", "3V3_IMU2", "GND", note="BMI088 VDD")
    C("100n", "3V3_IMU2", "GND", note="BMI088 VDDIO")
    ic(S_TXU0304, "U12", "TXU0304-Q1", "TXU0304QPWRQ1", "TI", "TSSOP-14 (PW)", "4-bit translator, Ioff, AEC-Q100",
       {"VCCA": "3V3_OBC", "A1": "IMU2_SCK", "A2": "IMU2_MOSI", "A3": "IMU2_GCS_N", "A4Y": "IMU2_MISO", "NC6": None,
        "GND": "GND", "OE": "3V3_IMU2", "NC9": None, "B4": "IM_SDO", "B3Y": "IM_CSB2", "B2Y": "IM_SDI", "B1Y": "IM_SCK",
        "VCCB": "3V3_IMU2", "PAD": "GND"}, notes="BMI088 SPI2 isolation")
    ic(S_TXU0204, "U13", "TXU0204-Q1", "TXU0204QPWRQ1", "TI", "TSSOP-14 (PW)", "4-bit translator, Ioff, AEC-Q100",
       {"VCCA": "3V3_OBC", "A1": "IMU2_ACS_N", "A2": "3V3_OBC", "A3Y": "IMU2_DRDY", "A4Y": None, "NC6": None,
        "GND": "GND", "OE": "3V3_IMU2", "NC9": None, "B4": "GND", "B3": "IM_INT3", "B2Y": None, "B1Y": "IM_CSB1",
        "VCCB": "3V3_IMU2", "PAD": "GND"}, notes="BMI088 accelerometer CS and gyro data-ready isolation")
    C("100n", "3V3_OBC", "GND", note="U12/U13 VCCA")
    C("100n", "3V3_IMU2", "GND", note="U12/U13 VCCB")
    R(100e3, "3V3_OBC", "IMU2_GCS_N", note="CS deselected in reset")
    R(100e3, "3V3_OBC", "IMU2_ACS_N", note="CS deselected in reset")
    R(100e3, "IMU2_SCK", "GND", note="defined input")
    R(100e3, "IMU2_MOSI", "GND", note="defined input")
    R(100e3, "IMU2_MISO", "GND", note="defined while isolated")
    R(100e3, "IMU2_DRDY", "GND", note="defined while isolated")


def sns():
    sensor_switch("U7", "SNS", "SNS_EN", "3V3_SNS", "SNS_RB", "3V3_SNS, sun-sensor front end")
    # SMA-B18: the magnetometer on its own latch-up switch and its own buffered I2C4 segment
    sensor_switch("U35", "MAG", "MAG_EN", "3V3_MAG", "MAG_RB", "3V3_MAG, magnetometer board A2")
    # I2C4 into the SNS domain
    ic(S_TCA9517, "U14", "TCA9517-Q1", "TCA9517DGKRQ1", "TI", "VSSOP-8 (DGK)", "I2C repeater, high-Z when unpowered, AEC-Q100",
       {"VCCA": "3V3_OBC", "SCLA": "I2C4_SCL", "SDAA": "I2C4_SDA", "GND": "GND", "EN": "SNS_I2C_EN", "SDAB": "SDA_S",
        "SCLB": "SCL_S", "VCCB": "3V3_SNS"}, notes="I2C4 to the SNS segment (ADS7138 0x10)",
       checked="TI orderable addendum read: Active (EPS board)")
    R(10e3, "3V3_SNS", "SNS_I2C_EN", note="repeater enabled only while 3V3_SNS is up")
    R(2.2e3, "3V3_OBC", "I2C4_SCL", note="I2C4 A-side pull-up")
    R(2.2e3, "3V3_OBC", "I2C4_SDA", note="I2C4 A-side pull-up")
    R(4.7e3, "3V3_SNS", "SCL_S", note="B-side pull-up")
    R(4.7e3, "3V3_SNS", "SDA_S", note="B-side pull-up")
    C("100n", "3V3_OBC", "GND", note="U14 VCCA")
    C("100n", "3V3_SNS", "GND", note="U14 VCCB")
    ic(S_TCA9517, "U34", "TCA9517-Q1", "TCA9517DGKRQ1", "TI", "VSSOP-8 (DGK)", "I2C repeater, high-Z when unpowered, AEC-Q100",
       {"VCCA": "3V3_OBC", "SCLA": "I2C4_SCL", "SDAA": "I2C4_SDA", "GND": "GND", "EN": "MAG_I2C_EN", "SDAB": "SDA_M",
        "SCLB": "SCL_M", "VCCB": "3V3_MAG"}, notes="I2C4 to the magnetometer segment (RM3100 0x20 on board A2 via J10; SMA-B18)",
       checked="TI orderable addendum read: Active (EPS board)")
    R(10e3, "3V3_MAG", "MAG_I2C_EN", note="repeater enabled only while 3V3_MAG is up")
    R(4.7e3, "3V3_MAG", "SCL_M", note="B-side pull-up")
    R(4.7e3, "3V3_MAG", "SDA_M", note="B-side pull-up")
    C("100n", "3V3_MAG", "GND", note="U34 VCCB")
    ic(S_LVC1T45, "U33", "SN74LVC1T45-Q1", "SN74LVC1T45QDCKRQ1", "TI", "SC70-6 (DCK)", "1-bit translator, Ioff, VCC isolation, AEC-Q100",
       {"VCCA": "3V3_OBC", "GND": "GND", "A": "MAG_DRDY", "B": "MG_DRDY", "DIR": "GND", "VCCB": "3V3_MAG"},
       notes="RM3100 DRDY (J10-5) to the MCU (B to A)")
    R(100e3, "MAG_DRDY", "GND", note="defined while isolated")
    # magnetometer: PNI RM3100 on the magnetometer board (A2, mag_board()) at the +Z end frame, on J10 / harness W6E
    j = place(CONN("PicoClasp_6", 6), "J10", "MAGNETOMETER", "5013310607", "Molex", "Pico-Clasp 1.0 mm 6 pos SMT",
              "1 A/contact", "< 5 mA", "W6E to the magnetometer board A2 (RM3100) at the +Z end frame: 3V3_MAG, GND, SDA, SCL, DRDY, GND",
              "Au (gold variant, SQSO-HAR-001 2.0 3.1)", "Digi-Key: Active (EPS board); gold order code to confirm (Molex drawing)")
    conn(j, {"P1": "3V3_MAG", "P2": "GND", "P3": "SDA_M", "P4": "SCL_M", "P5": "MG_DRDY", "P6": "GND"})
    R(100e3, "MG_DRDY", "GND", note="DRDY defined with the harness unplugged")
    esd4("D20", ["3V3_MAG", "SDA_M", "SCL_M", "MG_DRDY"], "J10 magnetometer harness")
    # sun-sensor front end: six TIAs into ADS7138
    faces = ["PX", "MX", "PY", "MY", "PZ", "MZ"]
    for k in range(3):
        a, b = faces[2 * k], faces[2 * k + 1]
        ic(S_OPA2, f"U{17 + k}", "TLV9002-Q1", "TLV9002QDGKRQ1", "TI", "VSSOP-8 (DGK)", "1.8-5.5 V RRIO, 1 MHz, AEC-Q100 grade 1",
           {"OUTA": f"TIA_{a}", "INAM": f"CSS_{a}_K", "INAP": "GND", "VM": "GND", "INBP": "GND", "INBM": f"CSS_{b}_K",
            "OUTB": f"TIA_{b}", "VP": "3V3_SNS"}, stress="VP 3.4 V / 6 V abs = 57 %",
           notes=f"sun-sensor transimpedance, faces {a}/{b}: Vout = Isc x 619 ohm (zero bias)",
           checked="TI orderable addendum read: Active")
        C("100n", "3V3_SNS", "GND", note=f"U{17 + k}")
    for f in faces:
        R(619, f"TIA_{f}", f"CSS_{f}_K", tol=0.1, note=f"Rf {f}: {cv('css_vout_isc', '{:.2f}')} V at 4.0 mA",
          stress=CALC["css_p_rf"]["stress"])
        C("10nC0G", f"TIA_{f}", f"CSS_{f}_K", note="Cf: 6 us, stable with the photodiode capacitance")
        R(1e3, f"TIA_{f}", f"ADC_{f}", note="ADC input RC")
        C("100n", f"ADC_{f}", "GND", note="ADC input reservoir")
    ic(S_ADS7138, "U16", "ADS7138-Q1", "ADS7138QRTERQ1", "TI", "WQFN-16 (RTE) 3x3", "8-ch 12-bit SAR ADC, I2C, AEC-Q100",
       {"AIN0": "ADC_PX", "AIN1": "ADC_MX", "AIN2": "ADC_PY", "AIN3": "ADC_MY", "AIN4": "ADC_PZ", "AIN5": "ADC_MZ",
        "AIN6": "TEMP_SNS_F", "AIN7": "VMTQ_SNS", "AVDD": "3V3_SNS", "DECAP": "ADS_DECAP", "GND": "GND", "DVDD": "3V3_SNS",
        "ADDR": "GND", "ALERT": None, "SCL": "SCL_S", "SDA": "SDA_S", "PAD": "GND"},
       notes="sun sensors AIN0-5, sensor-area temperature AIN6, VBAT_MTQ / 6 AIN7; address 0x10 (ADDR to GND)")
    C("1u", "3V3_SNS", "GND", note="ADS7138 AVDD")
    C("1u", "3V3_SNS", "GND", note="ADS7138 DVDD")
    C("1u", "ADS_DECAP", "GND", note="ADS7138 DECAP")
    ic(S_TMP235, "U20", "TMP235-Q1", "TMP235AQDBZRQ1", "TI", "SOT-23-3 (DBZ)", "analog 10 mV/C, 500 mV at 0 C, +/-2.5 C, AEC-Q100",
       {"VDD": "3V3_SNS", "VOUT": "TEMP_SNS", "GND": "GND"}, notes="temperature beside the gyro and IMU (ADS7138 AIN6)")
    R(1e3, "TEMP_SNS", "TEMP_SNS_F", note="RC")
    C("10n", "TEMP_SNS_F", "GND", note="RC")
    C("100n", "3V3_SNS", "GND", note="U20")
    R(1e6, "VBAT_MTQ", "VMTQ_SNS", note="VBAT_MTQ / 6 to AIN7 (2.8 V at 16.8 V)")
    R(200e3, "VMTQ_SNS", "GND")
    C("100n", "VMTQ_SNS", "GND", note="reservoir")
    # J8 sun sensors
    j = place(CONN("PicoClasp_12", 12), "J8", "SUN SENSORS", "5013311207", "Molex", "Pico-Clasp 1.0 mm 12 pos SMT",
              "1 A/contact", "< 6 mA", "W6D: six unshielded pairs, panel photodiodes (K, A) (Version C: no drains, was 14 pos)",
              "Au (gold variant, SQSO-HAR-001 2.0 3.1)", "pattern (501331-xx07); gold order code to confirm")
    pins = {}
    for i, f in enumerate(faces):
        pins[f"P{2 * i + 1}"] = f"CSS_{f}_K"
        pins[f"P{2 * i + 2}"] = "GND"
    conn(j, pins)
    esd4("D21", [f"CSS_{f}_K" for f in faces[:4]], "J8 sun sensors 1-4")
    esd4("D22", [f"CSS_{f}_K" for f in faces[4:]] + [None, None], "J8 sun sensors 5-6")


def mtq():
    """Three DRV8876-Q1 in PH/EN mode; nSLEEP = MTQ_ARM AND NOT(off-timer expired)."""
    D("SMBJ20A", "VBAT_MTQ", "GND", ref="D3", note="VBAT_MTQ TVS", stress="16.8 V / 20 V VRWM; Vc 32.4 V < DRV8876 40 V abs")
    C("4.7u", "VBAT_MTQ", "GND", 50, "1210", vop=16.8, note="VBAT_MTQ input bulk")
    C("4.7u", "VBAT_MTQ", "GND", 50, "1210", vop=16.8, note="VBAT_MTQ input bulk")
    R(18.7e3, "3V3_OBC", "MTQ_VREF", tol=0.1, note=f"VREF {cv('mtq_vref', '{:.3f}')} V: ITRIP {cv('mtq_itrip_nom', '{:.3f}')} A")
    R(10e3, "MTQ_VREF", "GND", tol=0.1)
    C("100n", "MTQ_VREF", "GND", note="VREF")
    R(10e3, "3V3_OBC", "MTQ_FAULT_N", note="nFAULT pull-up (wired-OR)")
    for k, ax in enumerate("XYZ"):
        ref = f"U{22 + k}"
        ic(S_DRV8876, ref, "DRV8876-Q1", "DRV8876QPWPRQ1", "TI", "HTSSOP-16 (PWP)", "4.5-37 V H-bridge, 3.5 A, current regulation, AEC-Q100",
           {"EN_IN1": f"MTQ_{ax}_ENF", "PH_IN2": f"MTQ_{ax}_PHF", "NSLEEP": "MTQ_SLEEP_N", "NFAULT": "MTQ_FAULT_N",
            "VREF": "MTQ_VREF", "IPROPI": f"MTQ_{ax}_IPR", "IMODE": "GND", "OUT1": f"MTQ_{ax}_P", "PGND": "GND",
            "OUT2": f"MTQ_{ax}_N", "VM": "VBAT_MTQ", "VCP": f"MTQ_{ax}_VCP", "CPH": f"MTQ_{ax}_CPH",
            "CPL": f"MTQ_{ax}_CPL", "GND": "GND", "PMODE": "GND", "PAD": "GND"},
           stress=(f"VM 16.8 V / 37 V = 45 %; Iout {cv('mtq_itrip_max', '{:.3f}')} A / 3.5 A = 4 %; "
                   f"P {cv('mtq_p_drv', '{:.2f}')} W, Tj {cv('mtq_tj_drv', '{:.0f}')} C"),
           notes=f"{ax} rod (CR0002): PH/EN mode (PMODE low), fixed off-time chopping at ITRIP, IPROPI 10 mV/mA to the ADC")
        R(100, f"MTQ_{ax}_PWM", f"MTQ_{ax}_ENF", note="series")
        R(100, f"MTQ_{ax}_DIR", f"MTQ_{ax}_PHF", note="series")
        C("100n", f"MTQ_{ax}_VCP", "VBAT_MTQ", vop=5.0, note="VCP to VM (charge pump)")
        C("22n", f"MTQ_{ax}_CPH", f"MTQ_{ax}_CPL", vop=16.8, note="charge-pump flying cap, VM rated")
        C("100n", "VBAT_MTQ", "GND", vop=16.8, note=f"{ref} VM bypass")
        C("4.7u", "VBAT_MTQ", "GND", 50, "1210", vop=16.8, note=f"{ref} VM bulk")
        R(10e3, f"MTQ_{ax}_IPR", "GND", tol=0.1, note="R_IPROPI 10.0 k (ITRIP and the current telemetry)")
        C("1n", f"MTQ_{ax}_IPR", "GND", note="IPROPI filter (datasheet 7.3.3)")
        R(1e3, f"MTQ_{ax}_IPR", f"MTQ_{ax}_I", note="to the MCU ADC")
        C("10n", f"MTQ_{ax}_I", "GND", note="ADC reservoir")
    j = place(CONN("PicoClasp_6", 6), "J7", "MAGNETORQUERS", "5013310607", "Molex", "Pico-Clasp 1.0 mm 6 pos SMT",
              "1 A/contact", f"{cv('mtq_itrip_max', '{:.3f}')} A max: 13 %", "W6C: X+, X-, Y+, Y-, Z+, Z- (CR0002 rods)",
              "Au (gold variant, SQSO-HAR-001 2.0 3.1)", "Digi-Key: Active (EPS board); gold order code to confirm (Molex drawing)")
    conn(j, {"P1": "MTQ_X_P", "P2": "MTQ_X_N", "P3": "MTQ_Y_P", "P4": "MTQ_Y_N", "P5": "MTQ_Z_P", "P6": "MTQ_Z_N"})
    # burn gate and hardware off-timer
    R(100e3, "MTQ_ARM", "GND", note="MCU reset = no burst")
    ic(S_LVC2G04, "U26", "SN74LVC2G04", "SN74LVC2G04DBVR", "TI", "SOT-23-6 (DBV)", "dual inverter, 1.65-5.5 V, -40..125 C",
       {"A1": "MTQ_ARM", "GND": "GND", "A2": "MTQ_EXPIRED", "Y2": "MTQ_OK", "VCC": "3V3_OBC", "Y1": "MTQ_ARM_N"},
       notes="MR = NOT MTQ_ARM; MTQ_OK = NOT expired", checked="TI orderable addendum read: Active (not AEC-Q100)")
    ic(S_LVC1G08, "U27", "SN74LVC1G08-Q1", "SN74LVC1G08QDBVRQ1", "TI", "SOT-23-5 (DBV)", "AND, AEC-Q100",
       {"A": "MTQ_ARM", "B": "MTQ_OK", "GND": "GND", "Y": "MTQ_SLEEP_N", "VCC": "3V3_OBC"},
       notes="nSLEEP of all three bridges = MTQ_ARM AND NOT expired")
    ic(S_HC4060, "U25", "CD74HC4060", "CD74HC4060PWR", "TI", "TSSOP-16 (PW)", "14-stage counter + RC oscillator, 2-6 V, -55..125 C",
       {"GND": "GND", "CTC": "BS_CT", "RTC": "BS_RT", "RS": "BS_RS", "MR": "MTQ_ARM_N", "Q10": "MTQ_EXPIRED",
        "VCC": "3V3_OBC", "Q12": None, "Q13": None, "Q14": None, "Q6": None, "Q5": None, "Q7": None, "Q4": None,
        "Q9": None, "Q8": None},
       notes=(f"hardware coil off-timer: {cv('mtq_bs_fosc', '{:.0f}')} Hz, Q10 after 512 periods = "
              f"{cv('mtq_bs_t', '{:.3f}')} s ({cv('mtq_bs_t_min', '{:.3f}')}-{cv('mtq_bs_t_max', '{:.3f}')} s); latched by D4 until MTQ_ARM falls"),
       checked="TI data sheet read; CD74HC4060PWR listed (RF board uses it); not AEC-Q100")
    C("100n", "3V3_OBC", "GND", note="U25-U27")
    C("10nC0G", "BS_CT", "BS_X", note="Ct 10 nF C0G")
    R(80.6e3, "BS_RT", "BS_X", note="Rt 80.6 k, select at test to 0.950 s at 25 C")
    R(162e3, "BS_X", "BS_RS", note="Rs = 2 Rt")
    D("1N4148W-Q", "BS_RS", "MTQ_EXPIRED", ref="D4", note="latch: Q10 high holds the oscillator input high")
    TP("MTQ_ARM")
    TP("MTQ_SLEEP_N")
    TP("MTQ_EXPIRED")


def jetson_link():
    ic(S_LVC1T45, "U28", "SN74LVC1T45-Q1", "SN74LVC1T45QDCKRQ1", "TI", "SC70-6 (DCK)", "1-bit translator, Ioff, VCC isolation, AEC-Q100",
       {"VCCA": "3V3_OBC", "GND": "GND", "A": "USART1_TX", "B": "JET_RXD_B", "DIR": "3V3_OBC", "VCCB": "JET_3V3"},
       notes="RID AV-09: OBC TX to the carrier; B side powered by the carrier's own 3.3 V, so an unpowered carrier sees high-Z")
    ic(S_LVC1T45, "U29", "SN74LVC1T45-Q1", "SN74LVC1T45QDCKRQ1", "TI", "SC70-6 (DCK)", "1-bit translator, Ioff, AEC-Q100",
       {"VCCA": "3V3_OBC", "GND": "GND", "A": "USART1_RX", "B": "JET_TXD_B", "DIR": "GND", "VCCB": "JET_3V3"},
       notes="carrier TX to the OBC (B to A)")
    R(33, "JET_RXD_B", "JET_RXD", note="series")
    R(33, "JET_TXD", "JET_TXD_B", note="series")
    R(100e3, "3V3_OBC", "USART1_RX", note="RX idles high while the carrier is off")
    R(100e3, "JET_TXD_B", "GND", note="defined input while the carrier boots")
    C("100n", "3V3_OBC", "GND", note="U28/U29 VCCA")
    C("100n", "JET_3V3", "GND", note="U28/U29 VCCB")
    R(10e3, "JET_3V3", "JET_PG", note="carrier power-good readback (RID AV-06)")
    R(100e3, "JET_PG", "GND", note="low with the carrier off")
    j = place(CONN("PicoClasp_6", 6), "J3", "JETSON", "5013310607", "Molex", "Pico-Clasp 1.0 mm 6 pos SMT", "1 A/contact",
              "< 5 mA", "W5B (6 pins, E10): GND, OBC TX, OBC RX, carrier 3.3 V, spreader NTC, NTC return", "Au (gold variant, SQSO-HAR-001 2.0 3.1)",
              "Digi-Key: Active (EPS board)")
    conn(j, {"P1": "GND", "P2": "JET_RXD", "P3": "JET_TXD", "P4": "JET_3V3", "P5": "PT_NTC_J", "P6": "GND"})
    esd4("D23", ["JET_RXD", "JET_TXD", "JET_3V3", "PT_NTC_J"], "J3 Jetson")
    # spreader thermistor RT1 (TE GA10K3MCD1, on the carrier, never powered by it): ratiometric to VDDA (the ADC reference)
    R(10e3, "VDDA", "PT_NTC_J", tol=0.1, note=f"NTC bias 10.0 k 0.1 % from VDDA: {cv('ntc_v_m15', '{:.2f}')} V at -15 C, "
      f"{cv('ntc_v_70', '{:.2f}')} V at 70 C", stress=f"{cv('ntc_p_bias_max', '{:.1f}')} mW of 100 mW")
    R(1e3, "PT_NTC_J", "PT_NTC", note="ADC series")
    C("100n", "PT_NTC", "GND", note="NTC reservoir and filter (1.6 kHz)")


def gnss_link():
    j = place(CONN("PicoClasp_2", 2), "J4", "GNSS PWR IN", "5013310207", "Molex", "Pico-Clasp 1.0 mm 2 pos SMT", "1 A/contact",
              f"0.232 A max (SW7 trip): 23 %", "W9B from EPS J21 (SW7, GNSS_EN): routed through the OBC to J5", "Au (gold variant, SQSO-HAR-001 2.0 3.1)",
              "Digi-Key: Active (EPS board)")
    conn(j, {"P1": "3V3_GNSS", "P2": "GND"})
    j = place(CONN("PicoClasp_6", 6), "J5", "GNSS", "5013310607", "Molex", "Pico-Clasp 1.0 mm 6 pos SMT", "1 A/contact",
              "0.232 A: 23 %", "W9A to the piNAV-NG/FM: 3V3, GND, RXD, TXD, 1PPS, GND", "Au (gold variant, SQSO-HAR-001 2.0 3.1)", "Digi-Key: Active (EPS board); gold order code to confirm (Molex drawing)")
    conn(j, {"P1": "3V3_GNSS", "P2": "GND", "P3": "GNSS_RXD", "P4": "GNSS_TXD", "P5": "GNSS_1PPS", "P6": "GND"})
    C("10u", "3V3_GNSS", "GND", 25, "1206", note="GNSS supply at J5")
    C("100n", "3V3_GNSS", "GND", note="translators VCCB")
    for ref, a, b, d, what in (("U30", "LPUART1_TX", "GNSS_RXD", "3V3_OBC", "OBC TX to the receiver"),
                               ("U31", "LPUART1_RX", "GNSS_TXD", "GND", "receiver TX to the OBC"),
                               ("U32", "GNSS_PPS", "GNSS_1PPS", "GND", "1PPS to TIM3_CH1")):
        ic(S_LVC1T45, ref, "SN74LVC1T45-Q1", "SN74LVC1T45QDCKRQ1", "TI", "SC70-6 (DCK)", "1-bit translator, Ioff, AEC-Q100",
           {"VCCA": "3V3_OBC", "GND": "GND", "A": a, "B": b, "DIR": d, "VCCB": "3V3_GNSS"},
           notes=f"GNSS isolation: {what}; B side on 3V3_GNSS, so a switched-off receiver is never back-powered")
    C("100n", "3V3_OBC", "GND", note="U30-U32 VCCA")
    R(100e3, "3V3_OBC", "LPUART1_RX", note="RX idle high while the receiver is off")
    R(100e3, "GNSS_PPS", "GND", note="PPS low while isolated")
    R(100e3, "GNSS_TXD", "GND", note="defined input")
    R(100e3, "GNSS_1PPS", "GND", note="defined input")
    R(10e3, "3V3_GNSS", "GNSS_PG", note="GNSS rail readback (SW7 trip)")
    R(100e3, "GNSS_PG", "GND")
    esd4("D24", ["3V3_GNSS", "GNSS_RXD", "GNSS_TXD", "GNSS_1PPS"], "J5 GNSS")


def eps_connector():
    pins = J1_PINS
    j = place(CONN("TFM_2x20", 40), "J1", "EPS", "TFM-120-01-S-D-RA", "Samtec", "Tiger Eye TFM 1.27 mm 2x20 right-angle",
              "contact rating per Samtec (not read)", "0.088 A per 3V3 contact, 0.115 A per VBAT_MTQ contact",
              "W3 v3: SFSDT 30 AWG to EPS J3 (TFM-120 vertical); cable toward +X, mated plug's back <= x +43.1 (SQSO-HAR-001 2.0 10.2)",
              "Au 30 uin", "not checked (Samtec site not read)")
    for i, nm in enumerate(pins, 1):
        link(nm, j[f"P{i}"])
    C("100n", "3V3_RTC", "GND", note="J1-38 entry")
    # series / back-power resistors on lines from the EPS
    for src, dst, v, why in (("EPS_ALERT_N_J", "EPS_ALERT_N", 10e3, "back-power limit (EPS 4.7 k pull-up)"),
                             ("SOLAR_SENSE_J", "SOLAR_SENSE", 10e3, "back-power limit"),
                             ("DEPLOY_SENSE_J", "DEPLOY_SENSE", 10e3, "series")):
        R(v, src, dst, note=why)
    for nm in ("VBATT_SNS", "IBATT_SNS", "VSOLAR_SNS", "ISOLAR_SNS", "VBUS_SNS", "VSW1_RB", "VJET_RB"):
        R(1e3, f"{nm}_J", nm, note="analog series (injection limit)")
        C("10n", nm, "GND", note="ADC reservoir")
    for nm in ("PAYLOAD_EN", "RADIO_EN", "ADCS_EN", "HEATER_EN", "DEPLOY_EN", "RF_TX_EN", "GNSS_EN", "BURN_W2A",
               "BURN_W1A", "JETSON_PWR", "BURN_W1B", "BURN_W2B", "EOL_ARM", "EOL_FIRE", "LOCK_CLR"):
        R(100, nm, f"{nm}_J", note="series (ESD current, edge rate)")
    for bus, what in (("I2C2", "EPS bus (the only pull-ups on it)"), ("I2C3", "AntS bus to EPS U80 (SMA-B07)")):
        for ln in ("SCL", "SDA"):
            for k in (1, 2):
                R(4.7e3, "3V3_OBC", f"{bus}_{ln}", note=f"{what}: pull-up {k} of 2 in parallel, each works alone (SMA-B07)")
    R(1e6, "3V3_OBC", "EPS_ALERT_N", note="defined state with the line open: no alert")
    R(1e6, "3V3_OBC", "SOLAR_SENSE", note="defined state with the line open: reads not deployed")
    R(10e6, "DEPLOY_SENSE", "GND", note="defined state with the line open: reads low (stowed); 3.0 V with the EPS 1 M source")
    sig = [n for n in pins if n not in ("3V3_OBC", "GND", "VBAT_MTQ", "3V3_RTC")]
    for k in range(0, len(sig), 4):
        esd4(None, sig[k:k + 4] + [None] * (4 - len(sig[k:k + 4])), "J1 EPS lines")
    esd1("3V3_RTC", "J1-38 3V3_RTC")
    TP("VBAT_MTQ")
    TP("3V3_RTC")


J1_PINS = ["3V3_OBC", "3V3_OBC", "GND", "GND", "I2C2_SCL", "I2C2_SDA", "GND", "EPS_ALERT_N_J", "PAYLOAD_EN_J",
           "RADIO_EN_J", "ADCS_EN_J", "HEATER_EN_J", "DEPLOY_EN_J", "RF_TX_EN_J", "GNSS_EN_J", "DEPLOY_SENSE_J",
           "BURN_W2A_J", "BURN_W1A_J", "VJET_RB_J", "SOLAR_SENSE_J", "GND", "VBATT_SNS_J", "IBATT_SNS_J", "VSOLAR_SNS_J",
           "ISOLAR_SNS_J", "VBUS_SNS_J", "I2C3_SCL", "I2C3_SDA", "JETSON_PWR_J", "BURN_W1B_J", "BURN_W2B_J", "EOL_ARM_J",
           "EOL_FIRE_J", "LOCK_CLR_J", "GND", "VBAT_MTQ", "VSW1_RB_J", "3V3_RTC", "VBAT_MTQ", "GND"]


def rf_connector():
    j = place(CONN("TFM_2x8", 16), "J2", "RF", "TFM-108-01-S-D-RA", "Samtec", "Tiger Eye TFM 1.27 mm 2x8 right-angle",
              "contact rating per Samtec (not read)", "signals < 5 mA",
              "W4A v4: SFSDT 30 AWG to RF J1 (TFM-108 RA); at (+41.0, -33.5), cable toward +X, plug's back <= x +43.1",
              "Au 30 uin", "not checked")
    conn(j, {"P1": "GND", "P2": "TNC_RXD_J", "P3": "TNC_TXD_J", "P4": "GND", "P5": "TX_PWR_SEL_J", "P6": "XPDR_EN_J",
             "P7": "GND", "P8": "TONE_OK_J", "P9": "XPDR_KEY_J", "P10": "PTT_EXC_J", "P11": "TX_TEMP_J",
             "P12": "TX_FWD_J", "P13": "TX_REV_J", "P14": "GND",
             "P15": "RX_CD_J", "P16": "RX_FAULT_J"})
    R(100, "USART2_TX", "TNC_RXD_J", note="series")
    R(100, "TX_PWR_SEL", "TX_PWR_SEL_J", note="series")
    R(100, "XPDR_EN", "XPDR_EN_J", note="series")
    for src, dst in (("TNC_TXD_J", "USART2_RX"), ("TONE_OK_J", "TONE_OK"), ("XPDR_KEY_J", "XPDR_KEY"),
                     ("PTT_EXC_J", "PTT_EXC_RB"), ("RX_CD_J", "RX_CD"), ("RX_FAULT_J", "RX_FAULT")):
        R(10e3, src, dst, note="back-power limit into an unpowered OBC")
    R(100e3, "3V3_OBC", "USART2_RX", note="RX idle high with the radio off")
    for n in ("TONE_OK", "XPDR_KEY", "PTT_EXC_RB", "RX_CD", "RX_FAULT"):
        R(100e3, n, "GND", note="reads low with the radio unpowered (RF board 4.1); RX_FAULT low = healthy")
    for src, dst in (("TX_TEMP_J", "TX_TEMP"), ("TX_FWD_J", "TX_FWD"), ("TX_REV_J", "TX_REV")):
        R(10e3, src, dst, note="analog series (back-power limit)")
        C("10n", dst, "GND", note="ADC reservoir")
    R(1e6, "TX_TEMP", "GND", note="open W4A pin 11 reads 0 V = -50 C, a detectable sensor fault (SMA-B19); reading low by 1.1 %")
    esd4(None, ["TNC_RXD_J", "TNC_TXD_J", "TX_PWR_SEL_J", "XPDR_EN_J"], "J2 RF")
    esd4(None, ["TONE_OK_J", "XPDR_KEY_J", "PTT_EXC_J", "RX_CD_J"], "J2 RF")
    esd4(None, ["TX_TEMP_J", "TX_FWD_J", "TX_REV_J", "RX_FAULT_J"], "J2 RF")


def adcs_power():
    j = place(CONN("PicoClasp_2", 2), "J6", "ADCS PWR IN", "5013310207", "Molex", "Pico-Clasp 1.0 mm 2 pos SMT", "1 A/contact",
              "0.339 A max (SW4 trip): 34 %", "W6B from EPS J20 (SW4, ADCS_EN): 3V3_ADCS", "Au (gold variant, SQSO-HAR-001 2.0 3.1)", "Digi-Key: Active (EPS board); gold order code to confirm (Molex drawing)")
    conn(j, {"P1": "3V3_ADCS", "P2": "GND"})
    C("10u", "3V3_ADCS", "GND", 25, "1206", note="3V3_ADCS entry")
    esd1("3V3_ADCS", "J6 3V3_ADCS")
    TP("3V3_ADCS")
    TP("3V3_GYRO")
    TP("3V3_SNS")


def board_misc():
    ic(S_TMP235, "U21", "TMP235-Q1", "TMP235AQDBZRQ1", "TI", "SOT-23-3 (DBZ)", "analog 10 mV/C, 500 mV at 0 C, AEC-Q100",
       {"VDD": "3V3_OBC", "VOUT": "TEMP_B_RAW", "GND": "GND"}, notes="TEMP_BOARD (adc.rs Linear(100, -50) unchanged)")
    R(1e3, "TEMP_B_RAW", "TEMP_BOARD", note="RC")
    C("10n", "TEMP_BOARD", "GND", note="RC")
    C("100n", "3V3_OBC", "GND", note="U21")
    p = place(TLED, "D1", "LED red", "LTST-C191KRKT", "Lite-On", "LED_SMD:LED_0603", "20 mA, 2.0 V", "2 mA", "status LED, DNP in flight",
              "Au", "not checked", dnp=True)
    link("GND", p["K"])
    link("LED_A", p["A"])
    R(680, "LED_STATUS", "LED_A", note="2 mA", dnp=True)


def access_port():
    j = place(CONN("TFM_2x5", 10), "J9", "ACCESS PORT", "TFM-105-01-S-D-RA", "Samtec", "Tiger Eye TFM 1.27 mm 2x5 right-angle",
              "contact rating per Samtec (not read)", "signals",
              "W8A (SFSDT 30 AWG) to the access-port board A3 (SWD, NRST, console, umbilical sense, watchdog disable); cable toward -X, plug's back <= x -43.1",
              "Au 30 uin", "not checked")
    conn(j, {"P1": "3V3_VTREF", "P2": "SWDIO_J", "P3": "GND", "P4": "SWCLK_J", "P5": "WDT_DIS_J", "P6": "RST_REQ_J",
             "P7": "USART3_TX_J", "P8": "USART3_RX_J", "P9": "UMB_J", "P10": "GND"})
    R(100, "3V3_OBC", "3V3_VTREF", note="VTREF for the probe; the GSE plug also ties pin 9 to it")
    R(22, "SWDIO", "SWDIO_J", note="series")
    R(22, "SWCLK", "SWCLK_J", note="series")
    # SMA-B14: the access port asks for a reset; it cannot hold NRST. High on J9-6 turns Q1 on, which pulls NRST;
    # a short of the line or its ESD channel to ground leaves Q1 off.
    R(1e3, "RST_REQ_J", "RST_REQ_G", note="gate series (ESD current)")
    R(100e3, "RST_REQ_G", "GND", note="gate pull-down: no request = no reset; a shorted line = no reset")
    p = place(TQN, "Q1", "2N7002BK", "2N7002BK,215", "Nexperia", "SOT-23", "60 V, 0.35 A N-MOSFET, AEC-Q101",
              "Vgs 3.3 V / 20 V = 17 %", "NRST pull-down on a GSE reset request (active high, from the adapter's inverter)",
              "Sn (matte): re-tin or coat", "not checked")
    link("RST_REQ_G", p["G"])
    link("GND", p["S"])
    link("NRST", p["D"])
    R(100, "USART3_TX", "USART3_TX_J", note="series")
    R(10e3, "USART3_RX_J", "USART3_RX", note="back-power limit from a powered GSE adapter")
    R(100e3, "3V3_OBC", "USART3_RX", note="RX idle")
    R(1e3, "UMB_J", "UMBILICAL_SENSE", note="series")
    R(100e3, "UMBILICAL_SENSE", "GND", note="low in flight (no plug)")
    R(0, "WDT_DIS_J", "WDT_DIS", note="to the SET0 network")
    esd4(None, ["SWDIO_J", "SWCLK_J", "RST_REQ_J", "WDT_DIS_J"], "J9 access port")
    esd4(None, ["USART3_TX_J", "USART3_RX_J", "UMB_J", "3V3_VTREF"], "J9 access port")


_esd_n = [0]


def esd4(ref, nets, what):
    p = place(S_ESD4, ref, "TPD4E05U06-Q1", "TPD4E05U06QDQARQ1", "TI", "USON-10 (DQA)", "4-ch ESD, 5.5 V VRWM, 0.5 pF, IEC 61000-4-2 12 kV, AEC-Q101",
              "", f"ESD at {what}", "NiPdAu", "TI orderable addendum read: Active")
    for pin, net in zip(("IO1", "IO2", "IO3", "IO4"), nets):
        if net is None:
            p[pin] += builtins.NC
        else:
            link(net, p[pin])
    for nc in ("NC6", "NC7", "NC9", "NC10"):
        p[nc] += builtins.NC
    link("GND", p["GND3"])
    link("GND", p["GND8"])


def esd1(net, what):
    p = place(S_ESD1, None, "TPD1E05U06-Q1", "TPD1E05U06QDPYRQ1", "TI", "X1SON-2 (DPY)", "1-ch ESD, 5.5 V VRWM, AEC-Q101", "",
              f"ESD at {what}", "NiPdAu", "TI orderable addendum read: Active")
    link(net, p["IO"])
    link("GND", p["GND"])


# ------------------------------------------------------------------------------------------------- build and outputs
def build():
    mcu()
    clocks()
    reset_supervisor()
    fram()
    rtc()
    gyro()
    imu2()
    sns()
    mtq()
    jetson_link()
    gnss_link()
    eps_connector()
    rf_connector()
    adcs_power()
    board_misc()
    access_port()
    finish_refs()


def finish_refs():
    for name, n in _nets.items():
        if name in POWER_NETS:
            n.drive = POWER
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


def mag_board():
    """Magnetometer board A2 (SQSO-OBC-BRD-001 9.4): PNI RM3100 on a 24 x 24 mm two-layer board at the +Z end frame,
    on harness W6E from OBC J10. Power, latch-up switch (U35) and bus isolation (U34, U33) stay on the OBC."""
    j = place(CONN("PicoClasp_6", 6), "J1", "TO OBC J10", "5013310607", "Molex", "Pico-Clasp 1.0 mm 6 pos SMT", "1 A/contact",
              "< 5 mA", "W6E from OBC J10: 3V3_MAG, GND, SDA, SCL, DRDY, GND", "Au (gold variant, SQSO-HAR-001 2.0 3.1)", "Digi-Key: Active (EPS board); gold order code to confirm (Molex drawing)")
    conn(j, {"P1": "3V3_MAG", "P2": "GND", "P3": "SDA_M", "P4": "SCL_M", "P5": "MG_DRDY", "P6": "GND"})
    esd4("D1", ["3V3_MAG", "SDA_M", "SCL_M", "MG_DRDY"], "J1 (harness end)")
    ic(S_MAGI2C, "U1", "RM3100 MagI2C", "13156", "PNI Sensor", "MLF-28 (PNI MagI2C)",
       "magneto-inductive ASIC, 2.0-3.6 V, I2C 0x20, 13 nT at 200 cycles, -40..85 C",
       {"SDA": "SDA_M", "RES2": "GND", "SA0": "GND", "AVDD": "3V3_MAG", "AVSS": "GND", "ZDRVP": "MZ_DP", "ZINP": "MZ_IP",
        "ZINN": "MZ_IN", "ZDRVN": "MZ_DN", "YDRVP": "MY_DP", "YINP": "MY_IP", "YINN": "MY_IN", "YDRVN": "MY_DN",
        "DVDD14": "3V3_MAG", "XDRVP": "MX_DP", "XINP": "MX_IP", "XINN": "MX_IN", "XDRVN": "MX_DN", "DVSS": "GND",
        "RES20": "GND", "NC21": None, "I2CEN": "3V3_MAG", "DRDY": "MG_DRDY", "NC24": None, "REXT": "MG_REXT",
        "DVDD26": "3V3_MAG", "SCL": "SCL_M", "SA1": "GND"},
       notes="flight magnetometer (SQSO-OBC-BRD-001 9.4); single-shot conversions in the coils-off window",
       finish="Sn (QFN): conformal coat", checked="PNI Doc 1017252 R07 read (pinout, reference schematic fig. 4-2)")
    for ax, mpn, val in (("X", "13104", "Sen-XY-f"), ("Y", "13104", "Sen-XY-f"), ("Z", "13101", "Sen-Z-f")):
        p = place(TCOIL, f"L{'XYZ'.index(ax) + 1}", val, mpn, "PNI Sensor", "PNI sensor coil", "magneto-inductive sensor coil, -40..85 C",
                  "", f"{ax} axis coil between {ax}INP and {ax}INN, oriented to the body {ax} axis", "Sn", "PNI Doc 1017252 R07 read")
        link(f"M{ax}_IP", p["A"])
        link(f"M{ax}_IN", p["B"])
        R(121, f"M{ax}_DP", f"M{ax}_IP", note=f"R_bias {ax}P (PNI table 3-1)")
        R(121, f"M{ax}_IN", f"M{ax}_DN", note=f"R_bias {ax}N")
    R(33e3, "MG_REXT", "GND", note="REXT 33 k: 180 kHz clock (PNI table 3-1)")
    C("10u", "3V3_MAG", "GND", 25, "1206", note="RM3100 (PNI reference: 10 uF + 0.1 uF)")
    C("100n", "3V3_MAG", "GND", note="RM3100")
    finish_refs()


def a3_board():
    """Access-port board A3 (SQSO-OBC-BRD-001 15.1; SQSO-HAR-001 2.0 3.2): one gold Micro-D 21 socket receptacle at the
    -Y access window carrying W8A (OBC debug and key loading, ground GND) and W11 (EPS charging path, return PACK_N).
    The two groups share no conductor: PACK_N is never joined to GND here, and the shell bonds to the frame only."""
    j = place(CONN("MicroD_21", 21), "J1", "ACCESS PORT (GSE)", "M83513/11-C (21-way socket, straight PCB tails)",
              "MIL-DTL-83513 (Glenair, ITT Cannon, Omnetics)", "Micro-D 21 socket receptacle, PCB mount, flange to the -Y shear panel",
              "3 A/contact (MIL-DTL-83513)", "charge 0.25 A per contact (8 %); signals < 5 mA",
              "GSE cable W8B mates from outside; pins 8, 15-18 unused (separate the charge cluster 9-11, 19-21 from debug)",
              "Au (gold over Ni, MIL-DTL-83513)", "slash-sheet dash number to confirm")
    conn(j, {"P1": "VTREF", "P2": "SWDIO", "P3": "GND", "P4": "SWCLK", "P5": "WDT_DIS", "P6": "RST_REQ", "P7": "U3_TX",
             "P8": None, "P9": "UMB_VMON", "P10": "PACK_N", "P11": "PACK_N", "P12": "U3_RX", "P13": "UMB_SENSE",
             "P14": "GND", "P15": None, "P16": None, "P17": None, "P18": None, "P19": "UMB_SHUT", "P20": "UMB_CHG",
             "P21": "UMB_CHG"})
    w8 = place(CONN("W8A_pads", 10), "J2", "W8A SOLDER PADS", "-", "-", "10 plated holes, 1.27 mm pitch, for 30 AWG SFSDT wires",
               "-", "signals", "W8A from OBC J9 (same pin order as J9); wires staked within 10 mm", "Au (ENIG)", "PCB feature")
    conn(w8, {"P1": "VTREF", "P2": "SWDIO", "P3": "GND", "P4": "SWCLK", "P5": "WDT_DIS", "P6": "RST_REQ", "P7": "U3_TX",
              "P8": "U3_RX", "P9": "UMB_SENSE", "P10": "GND"})
    w11 = place(CONN("W11_pads", 6), "J3", "W11 SOLDER PADS", "-", "-", "6 plated holes for 24 AWG wires", "-",
                "0.25 A per hole (charge)", "W11 from EPS J13 (pin order of J13: UMB_CHG x2, PACK_N x2, UMB_VMON, UMB_SHUT)",
                "Au (ENIG)", "PCB feature")
    conn(w11, {"P1": "UMB_CHG", "P2": "UMB_CHG", "P3": "PACK_N", "P4": "PACK_N", "P5": "UMB_VMON", "P6": "UMB_SHUT"})
    # debug group: ESD to GND (the OBC's ground through W8A); the OBC's J9 arrays and series resistors are behind it
    esd4("D1", ["SWDIO", "SWCLK", "RST_REQ", "WDT_DIS"], "A3 Micro-D debug lines")
    esd4("D2", ["U3_TX", "U3_RX", "UMB_SENSE", "VTREF"], "A3 Micro-D console, sense and VTREF")
    # charge group: protection referenced to PACK_N, never to GND
    D("SMF24A", "UMB_CHG", "PACK_N", ref="D3", note="charge input TVS at the window (EPS F5 and D16 behind it)",
      stress="16.8 V / 24 V VRWM = 70 %; Vc 38.9 V below the 40 V VBAT parts")
    D("SMF24A", "UMB_VMON", "PACK_N", ref="D4", note="pack-monitor line TVS (100 k from BAT_F on the EPS)",
      stress="16.8 V / 24 V VRWM = 70 %; leakage <= 1 uA worst at VRWM")
    p = place(S_ESD1, "D5", "TPD1E05U06-Q1", "TPD1E05U06QDPYRQ1", "TI", "X1SON-2 (DPY)", "1-ch ESD, 5.5 V VRWM, AEC-Q101", "",
              "ESD on UMB_SHUT, referenced to PACK_N (the AFE's ground)", "NiPdAu", "TI orderable addendum read: Active")
    link("UMB_SHUT", p["IO"])
    link("PACK_N", p["GND"])
    finish_refs()


def refkey(r):
    m = re.match(r"([A-Z]+)(\d+)", r)
    return (m.group(1), int(m.group(2))) if m else (r, 0)


def outputs(stem="obc_board", docid="SQSO-OBC-BRD-001", pinmap=True, waivers=None):
    circ = builtins.default_circuit
    buf = io.StringIO()
    h = logging.StreamHandler(buf)
    h.setLevel(logging.INFO)
    lg = skidl.erc_logger if hasattr(skidl, "erc_logger") else logging.getLogger("ERC_Logger")
    lg.addHandler(h)
    ERC()
    lg.removeHandler(h)
    with open(os.path.join(HERE, f"{stem}_erc.log"), "w") as f:
        f.write(f"{docid} electrical rule check (SKiDL 3.0 ERC), generated by design.py\n\n")
        f.write(buf.getvalue() or "No ERC messages.\n")
        f.write("\nWaivers (intentional modelling choices, reviewed):\n" + waivers if waivers else "\nWaivers (intentional modelling choices, reviewed):\n" + (
                "W1  STM32L4R5ZIT6 port pins are typed PASSIVE: each pin's direction is set by the firmware (obc_pinmap.csv).\n"
                "W2  Rails entering through connectors or switches (3V3_OBC, 3V3_ADCS, 3V3_RTC, VBAT_MTQ, JET_3V3, 3V3_GNSS,\n"
                "    VDDA, 3V3_VTREF and the TPS2553 outputs) are declared driven (net.drive = POWER).\n"
                "W3  Pins marked NOCONNECT are open on purpose: datasheet NC pins, unused M-G370 SOUT/GPIO2, BMI088 INT1/2/4,\n"
                "    unused HC4060 stages, ADS7138 ALERT, RV-3028 INT, unused ESD channels and the MCU spare pins.\n"
                "W4  TPS3850 RESET and WDO, and the three DRV8876 nFAULT outputs, are open drain wired-OR by design.\n"
                "W5  NRST is bidirectional: the MCU drives it low on internal resets; the supervisor's outputs are open drain.\n"
                if pinmap else
                "W1  3V3_MAG enters on J1 from the OBC's latch-up switch U35 and is declared driven (net.drive = POWER).\n"
                "W2  MagI2C NC pins 21 and 24 are open (PNI table 4-1).\n"))
    generate_netlist(file_=os.path.join(HERE, f"{stem}.net"), tool=KICAD8)
    with open(os.path.join(HERE, f"{stem}_nets.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["net", "pins"])
        for net in sorted(circ.nets, key=lambda n: n.name):
            if net.pins:
                w.writerow([net.name, " ".join(sorted(f"{p.part.ref}.{p.num}" for p in net.pins))])
    groups = OrderedDict()
    for p in circ.parts:
        info = BOMINFO.get(id(p))
        if info is None:
            continue
        groups.setdefault((info["mpn"], info["value"], info["dnp"]), []).append((p.ref, info))
    rows = []
    for (mpn, value, dnp), items in groups.items():
        refs = sorted((r for r, _ in items), key=refkey)
        info = items[0][1]
        stresses = sorted({i["stress"] for _, i in items if i["stress"]})
        notes = sorted({i["notes"] for _, i in items if i["notes"]})
        rows.append([" ".join(refs), len(refs), value + (" (DNP)" if dnp else ""), mpn, info["mfr"], info["pkg"],
                     info["rating"], "; ".join(stresses)[:400], "; ".join(notes)[:400], info["finish"], info["checked"]])
    rows.sort(key=lambda r: refkey(r[0].split()[0]))
    with open(os.path.join(HERE, f"{stem}_bom.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["Reference", "Qty", "Value", "MPN", "Manufacturer", "Package", "Rating", "Derated stress", "Notes",
                    "Terminal finish / tin mitigation", "MPN check"])
        w.writerows(rows)
    with open(os.path.join(HERE, "obc_pinmap.csv"), "w", newline="") if pinmap else io.StringIO() as f:
        w = csv.writer(f)
        w.writerow(["LQFP144 pin", "port pin", "net", "function", "dir", "level", "default at reset", "connector / board"])
        num = {tok.split(":")[1]: tok.split(":")[0] for tok in LQFP144.split()}
        for pin, (net, func, d, level, reset, dest) in PINMAP.items():
            w.writerow([num.get(pin, ""), pin, net or "-", func, d, level, reset, dest])
    for junk in ("design.erc", "design.log", "design_sklib.py", "skidl_REPL.erc", "skidl_REPL.log"):
        if os.path.exists(os.path.join(HERE, junk)):
            os.remove(os.path.join(HERE, junk))
    total = sum(r[1] for r in rows)
    print(f"{stem}: {total} placements, {len(rows)} BOM lines, {len([n for n in circ.nets if n.pins])} nets"
          + (f", {len(PINMAP)} MCU signal pins mapped, {len(SPARES)} spare" if pinmap else ""))


if __name__ == "__main__":
    os.chdir(HERE)
    build()
    outputs()
    # the magnetometer board (A2) as its own netlist, BOM and ERC
    skidl.reset()
    _nets.clear()
    BOMINFO.clear()
    AUTOREF.clear()
    set_default_tool(SKIDL)
    mag_board()
    outputs("mag_board", "SQSO-OBC-BRD-001 A2 (magnetometer board)", pinmap=False)
    # the access-port board (A3)
    skidl.reset()
    _nets.clear()
    BOMINFO.clear()
    AUTOREF.clear()
    set_default_tool(SKIDL)
    a3_board()
    outputs("a3_board", "SQSO-OBC-BRD-001 A3 (access-port board)", pinmap=False,
            waivers="W1  Micro-D pins 8 and 15-18 are open on purpose (they separate the charge cluster from the debug pins).\n"
                    "W2  All nets are passive (connectors, solder pads, protection): no source on this board.\n")
