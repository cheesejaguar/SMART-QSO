#!/usr/bin/env python3
"""SMART-QSO RF board (SQSO-RF-BRD-001): schematic captured as code (SKiDL 3.0).

Generates, next to this file:
  rf_board.net      KiCad-format netlist (SKiDL generate_netlist)
  rf_board_bom.csv  bill of materials (reference, quantity, value, MPN, manufacturer, package, rating,
                    derated stress, notes)
  rf_board_erc.log  SKiDL electrical rule check output, followed by the list of intentional waivers

KiCad libraries are not installed, so every part's pins are defined here (Part(tool=SKIDL)). Pin numbers follow the
manufacturer's datasheet; where the datasheet could not be read the part is marked "pinout to confirm" in its notes
(RF_BOARD_DESIGN.md §12 lists them).

Run from any directory (outputs go next to this file):
  <venv with skidl>/bin/python hardware/electrical/rf/design.py
The calculation script rf_calcs.py (numpy only) must have been run first: the BOM stresses come from rf_calcs.json.
"""
from __future__ import annotations

import csv
import io
import json
import os
import sys
from collections import OrderedDict, defaultdict
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
os.chdir(HERE)          # SKiDL writes its .erc/.log next to the script's working directory

import skidl  # noqa: E402
import builtins  # noqa: E402
from skidl import ERC, POWER, SKIDL, Net, Part, Pin, generate_netlist  # noqa: E402

NC = builtins.NC  # SKiDL 3.0 installs the no-connect net as a builtin

T = Pin.types
I, O, B, P, PI, PO, OC, TS, NCt = T.INPUT, T.OUTPUT, T.BIDIR, T.PASSIVE, T.PWRIN, T.PWROUT, T.OPENCOLL, T.TRISTATE, T.NOCONNECT

CALC = json.loads((HERE / "rf_calcs.json").read_text()) if (HERE / "rf_calcs.json").exists() else {}
ST = CALC.get("stresses", {})

NETS: dict[str, Net] = {}
BOM: list[dict] = []
PINMAP: dict[str, dict[str, list[int]]] = {}
WAIVERS: list[str] = []

# Magnetic cleanliness (OBC item C16, RF_BOARD_DESIGN §10.1): magnetometer board A2 above (+24, +27) mm
MAG_FAR = "ferromagnetic (place >= 40 mm from (+24, +27), zone M2)"
MECH: list[dict] = []   # mechanical items (no pins): written to the BOM after the electrical parts

RAILS = ["GND", "RADIO_3V3_IN", "+3V3_RAD", "+3V0_RX", "+3V1_DIG", "+3V1_AUD", "PA_5V_IN", "+5V_PA", "+5V_PA_A",
         "+3V3_PA", "+3V3_LMX"]


def N(name: str) -> Net:
    if name not in NETS:
        NETS[name] = Net(name)
        if name in RAILS:
            NETS[name].drive = POWER
    return NETS[name]


def nv(x) -> str:
    return f"{float(x):g}"


def part(ref, kind, value, mpn, mfr, pkg, rating, pins, stress="", notes="", status="", dnp=False):
    """pins: list of (number, name, type, net or None). Net None -> NC (intentionally unconnected)."""
    p = Part(tool=SKIDL, name=kind, ref_prefix="".join(c for c in ref if c.isalpha()),
             pins=[Pin(num=n, name=nm, func=t) for n, nm, t, _ in pins], footprint=pkg)
    p.ref = ref
    p.value = value
    p.tag = ref
    PINMAP[ref] = defaultdict(list)
    for n, nm, t, net in pins:
        PINMAP[ref][nm].append(n)
        if net is None:
            p[n] += NC
        else:
            N(net).__iadd__(p[n])
    BOM.append(dict(ref=ref, kind=kind, value=value, mpn=mpn, mfr=mfr, pkg=pkg, rating=rating, stress=stress,
                    notes=notes, status=status, dnp=dnp))
    return p


def two(ref, kind, value, a, b, mpn, mfr, pkg, rating, stress="", notes="", status="", dnp=False):
    return part(ref, kind, value, mpn, mfr, pkg, rating, [(1, "1", P, a), (2, "2", P, b)], stress, notes, status, dnp)


# ---------------------------------------------------------------- passive families (one place for MPN patterns)
TIN = "matte-tin terminations: Sn63Pb37 hot-solder-dip re-tinning (EEE_PARTS_LIST Rev C §2A.3)"
ST_PASS = "series datasheet read; individual value P/N built from the series ordering code, not checked one by one"


def R(ref, value, a, b, vmax=3.4, pkg="0603", tol="1 %", notes="", dnp=False, thin=False):
    ohms = parse_value(value)
    pw = (vmax ** 2 / ohms) if ohms > 0 else 0.0
    rating_w = {"0402": 0.063, "0603": 0.1, "0805": 0.125}[pkg]
    if thin:
        mpn = f"ERA-3AEB{rcode(ohms)}V" if pkg == "0603" else f"ERA-2AEB{rcode(ohms)}X"
        mfr = "Panasonic (thin film 0.1 %, 25 ppm/K, AEC-Q200)"
        tol = "0.1 %"
    else:
        mpn = f"CRCW{pkg}{rcode_vishay(ohms)}FKEA" if ohms > 0 else f"CRCW{pkg}0000Z0EA"
        mfr = "Vishay Dale (thick film, AEC-Q200)"
    stress = f"{pw * 1e3:.2f} mW / {rating_w * 1e3:.0f} mW = {pw / rating_w * 100:.1f} % (rule 50 %)" if ohms > 0 else "0 ohm link"
    return two(ref, "R", value, a, b, mpn, mfr, f"R_{pkg}", f"{tol}, {rating_w * 1e3:.0f} mW, 75 V", stress,
               (notes + "; " if notes else "") + TIN, ST_PASS, dnp)


MURATA_X7R = {  # (value, package): (MPN, voltage)  AEC-Q200 GCM series
    ("100nF", "0603"): ("GCM188R71C104KA37D", 16), ("1uF", "0603"): ("GCM188R71C105KA64D", 16),
    ("2.2uF", "0603"): ("GCM188R71C225KE22D", 16), ("10uF", "0805"): ("GCM21BR71A106KE22L", 10),
    ("10uF", "1206"): ("GCM31CR71C106KA64L", 16), ("22uF", "1206"): ("GCM31CR71A226KE02L", 10),
    ("22uF", "1210"): ("GCM32ER71E226KE15L", 25),
}


def C(ref, value, a, b, v_work=3.4, kind="X7R", pkg="0603", notes="", dnp=False, rf=False):
    f = parse_value(value)
    if rf or kind == "C0G":
        vr = 100 if rf else 50
        series = {("0603", 100): "GRM1885C2A", ("0603", 50): "GRM1885C1H", ("0805", 100): "GRM2165C2A",
                  ("0805", 50): "GRM2195C1H", ("1206", 50): "GRM3195C1H"}[(pkg, vr)]
        tolc, tol = ("B", "+-0.1 pF") if f < 10e-12 else ("G", "+-2 %")
        if not rf and f >= 1e-9:
            tolc, tol = "J", "+-5 %"
        mpn = f"{series}{ccode(f)}{tolc}A01D"
        mfr = "Murata (C0G/NP0)"
        if value == "220nF" and pkg == "1206":
            mpn, vr = "GRM31C5C1E224JE01L", 25
        rating = f"C0G {tol} {vr} V"
    else:
        if (value, pkg) in MURATA_X7R:
            mpn, vr = MURATA_X7R[(value, pkg)]
        else:
            vr = 16
            mpn = f"GCM-series X7R {value} {pkg} {vr} V (P/N to select)"
        mfr = "Murata (X7R, AEC-Q200)"
        rating = f"X7R +-10 % {vr} V"
    stress = f"{v_work:.2f} V / {vr} V = {v_work / vr * 100:.0f} % (rule 50 %)"
    return two(ref, "C", value, a, b, mpn, mfr, f"C_{pkg}", rating, stress,
               (notes + "; " if notes else "") + TIN, ST_PASS, dnp)


def L(ref, value, a, b, family="0603HP", i_work=0.01, notes="", dnp=False):
    nh = parse_value(value) * 1e9
    if family == "1812SMS":
        mpn = f"1812SMS-{lcode(nh)}GLC"
        mfr = "Coilcraft (Midi Spring air core, +-2 %)"
        rating = "air core, Irms >= 2.5 A (series), Q > 100 at 150 MHz"
        irat = 2.5
        pkg = "L_1812SMS"
    elif family == "0806SQ":
        mpn = f"0806SQ-{lcode(nh)}GLC"
        mfr = "Coilcraft (Square air core, +-2 %)"
        rating = "air core, Irms >= 2.5 A (series), Q > 100 at 150 MHz"
        irat = 2.5
        pkg = "L_0806SQ"
    elif family == "0603HP":
        mpn = f"0603HP-{lcode(nh)}XGLU"
        mfr = "Coilcraft (wire-wound ceramic, +-2 %)"
        rating = "Irms >= 0.4 A (series), Q 50-80 at 450 MHz"
        irat = 0.4
        pkg = "L_0603"
    else:
        mpn = family
        mfr = "Coilcraft"
        rating = "see notes"
        irat = 1.0
        pkg = "L_0805"
    stress = f"{i_work:.3f} A / {irat} A = {i_work / irat * 100:.0f} % (rule 50 %)"
    return two(ref, "L", value, a, b, mpn, mfr, pkg, rating, stress,
               (notes + "; " if notes else "") + "order the tin-lead termination option ('S' in the code) or re-tin",
               ST_PASS, dnp)


def FB(ref, a, b, i_work, mpn="BLM18PG221SN1D", notes=""):
    return two(ref, "FB", "220R@100MHz", a, b, mpn, "Murata", "FB_0603", "1.4 A, 0.10 ohm max DCR",
               f"{i_work:.3f} A / 1.4 A = {i_work / 1.4 * 100:.0f} %",
               (notes + "; " if notes else "") + "ferrite: " + MAG_FAR + "; " + TIN,
               "series datasheet read")


def parse_value(v: str) -> float:
    s = v.replace("ohm", "").replace("R", "").strip()
    mult = {"p": 1e-12, "n": 1e-9, "u": 1e-6, "m": 1e-3, "k": 1e3, "M": 1e6}
    for k_, m in mult.items():
        if s.endswith(k_ + "F") or s.endswith(k_ + "H"):
            return float(s[:-2]) * m
        if s.endswith(k_):
            return float(s[:-1]) * m
    s = s.rstrip("FH")
    try:
        return float(s)
    except ValueError:
        return 0.0


def rcode(ohms: float) -> str:          # Panasonic ERA: 4 characters, e.g. 1002 = 10.0 k
    e = 0
    x = ohms
    while x >= 1000:
        x /= 10
        e += 1
    return f"{int(round(x)):03d}{e}" if ohms >= 100 else f"{ohms:.1f}".replace(".", "R").rjust(4, "0")


def rcode_vishay(ohms: float) -> str:   # Vishay CRCW: e.g. 10K0, 1M00, 100R
    for unit, div in (("M", 1e6), ("K", 1e3), ("R", 1.0)):
        if ohms >= div or unit == "R":
            v = ohms / div
            s = f"{v:.3g}"
            if "." in s:
                a_, b_ = s.split(".")
                s = (a_ + unit + b_).ljust(4, "0")[:4]
            else:
                s = (s + unit).ljust(4, "0")[:4]
            return s
    return "0R00"


def ccode(f: float) -> str:             # Murata capacitance code, e.g. 101 = 100 pF, 1R5 = 1.5 pF
    pf = f * 1e12
    if pf < 10:
        return f"{pf:.1f}".replace(".", "R")
    e = 0
    x = pf
    while x >= 100:
        x /= 10
        e += 1
    return f"{int(round(x)):02d}{e}"


def lcode(nh: float) -> str:            # Coilcraft inductance code: 22N, 6N8, R12
    if nh >= 100:
        return "R" + f"{int(round(nh / 10)):02d}"
    if abs(nh - round(nh)) < 1e-6:
        return f"{int(round(nh))}N"
    return f"{nh:.1f}".replace(".", "N")


# =============================================================================================== connectors
part("J1", "Conn 16p 1.27 mm right-angle", "OBC-RF signals (W4A)", "TFM-108-01-S-D-RA", "Samtec",
     "TFM-108 right-angle, 2 x 8, 1.27 mm (Tiger Eye)", "1.27 mm, -55..+125 C; mates Samtec SFSDT (latched, PTFE)",
     [(1, "GND1", P, "GND"), (2, "OBC_TX", P, "UART_OBC_TX_J"), (3, "TNC_TX", P, "UART_TNC_TX_J"), (4, "GND4", P, "GND"),
      (5, "TX_PWR_SEL", P, "TX_PWR_SEL_J"), (6, "XPDR_EN", P, "XPDR_EN_J"), (7, "GND7", P, "GND"),
      (8, "TONE_OK", P, "TONE_OK_RB"), (9, "XPDR_KEY", P, "XPDR_KEY_RB"), (10, "PTT_EXC", P, "PTT_EXC_RB"),
      (11, "TX_TEMP", P, "TX_TEMP"), (12, "TX_FWD", P, "TX_FWD"), (13, "TX_REV", P, "TX_REV"), (14, "AGND", P, "GND"),
      (15, "RX_CD", P, "RX_CD_J"), (16, "RX_FAULT", P, "RX_FAULT_J")],
     "signal currents < 5 mA (< 1 %)", "W4 pinout (16 positions; RX_CD and RX_FAULT, RID RFB-07). Right-angle TFM per SQSO-HAR-001 2.0 item R-1 (was "
     "FTSH-108 vertical with a PVC FFSD ribbon): gold contacts (-S plating code); tail finish to confirm, re-tin if tin "
     "(EEE_PARTS_LIST §2A.3). Placement: top side, (+X, -Y) corner, pin field along Y at y -39.3 ... -27.7 (clear of "
     "the (+40, -43) column keep-out), mating face toward +X so the SFSDT cable leaves toward +X with no bend over the "
     "top; plug's back at x <= +43.1; header and plug within x +30.0 ... +43.1; height <= 7.0 mm (10.0 mm allocation "
     "kept); the shield frame is notched at x > +30.0, y < -26.0 (§10)",
     "MPN from SQSO-HAR-001 2.0 §3; Samtec drawing not read (VI-RFB-12)")
part("J2", "Conn 6p 1.00 mm Pico-Clasp", "EPS-RF power (W4B)", "203556-0607", "Molex",
     "Pico-Clasp 1x6 vertical SMT, 0.38 um gold (gold version of 501331-0607)",
     "1.0 A/contact, 50 V", [(1, "RADIO_3V3", PO, "RADIO_3V3_IN"), (2, "GND2", P, "GND"), (3, "PA_5V_A", PO, "PA_5V_IN"),
                             (4, "PA_5V_B", P, "PA_5V_IN"), (5, "GND5", P, "GND"), (6, "GND6", P, "GND")],
     ST.get("connector_PA_contacts", {}).get("stress", "") + " (two contacts)",
     "Gold finish (SQSO-HAR-001 2.0 item R-1; tin prohibited, EEE_PARTS_LIST §2A.3): mates the 501330-0600 housing "
     "with gold 501334 terminals, W4B 28 AWG in three twisted pairs 3+2, 4+5, 1+6 (item R-2). Placement (C16): top "
     "side on the -Y edge strip outside the shield at about (-26, -44), next to the PA, so the PA current path "
     "(J2-FB803-L602/L604-Q601) stays >= 60 mm from (+24, +27); PA_5V and its GND contacts adjacent, routed as a pair",
     "6-circuit code inferred from the 203556-0307 and -1407 listings (0.38 um gold, vertical SMT); lock style and "
     "mating with 501330 to confirm on the Molex drawing (VI-RFB-12)")
for j, net, lbl, xy in (("J3", "VHF_A", "2 m element +X", "(-34.5, -19.0)"),
                        ("J4", "VHF_B", "2 m element -X", "(-34.5, -25.0)"),
                        ("J5", "UHF_A", "70 cm element +Y", "(-34.5, +19.0)"),
                        ("J6", "UHF_B", "70 cm element -Y", "(-34.5, +25.0)")):
    part(j, "MMCX jack straight PCB", lbl, "73415-1473", "Molex", "MMCX vertical jack, through-hole (4 ground legs)",
         "50 ohm, DC-6 GHz, 170 Vrms, -55..+165 C, 500 cycles", [(1, "SIG", P, net), (2, "G1", P, "GND"),
                                                                 (3, "G2", P, "GND"), (4, "G3", P, "GND"),
                                                                 (5, "G4", P, "GND")],
         "1 W at 146 MHz (0.5 W per element) / receive: < 10 %",
         f"Top side at {xy} mm, outside the shield (J3/J4 in the frame's -X notch); right-angle plug exits -X into the "
         "-X wall gap (RF_BOARD_DESIGN §4.3). Brass body and beryllium-copper contact, gold, listed non-magnetic (ANT-8; "
         "OBC item C16). Legs trimmed to <= 1.0 mm below the 1.6 mm board. Mates the W10 v3 cable plug",
         "distributor listing read 2026-10-27 (Active, non-magnetic, Au over brass/BeCu); drawing SD-73415-147-001 "
         "not read (VI-RFB-12)")
part("J7", "SWD pads (Tag-Connect TC2030)", "TNC MCU programming", "footprint only (TC2030-IDC-NL cable)",
     "Tag-Connect", "TC2030-NL footprint", "-", [(1, "VDD", P, "+3V1_DIG"), (2, "SWDIO", P, "SWDIO"),
                                                 (3, "NRST", P, "MCU_NRST"), (4, "SWCLK", P, "SWCLK"), (5, "GND", P, "GND"),
                                                 (6, "SWO", P, None)], "-", "No component fitted", "n/a")

# =============================================================================================== power (800)
FB("FB801", "RADIO_3V3_IN", "+3V3_RAD", 0.09, notes="receive rail entry")
C("C801", "22uF", "+3V3_RAD", "GND", 3.4, pkg="1206")
C("C802", "100nF", "+3V3_RAD", "GND", 3.4)
part("U801", "LDO 300 mA low noise", "3.0 V (V_RX)", "TPS7A2030PDQNR", "Texas Instruments", "X2SON-4 1x1 mm",
     "300 mA, 6.0 V in max, -40..125 C Tj",
     [(1, "OUT", PO, "+3V0_RX"), (2, "GND", PI, "GND"), (3, "EN", I, "+3V3_RAD"), (4, "IN", PI, "+3V3_RAD"),
      (5, "PAD", P, "GND")], ST.get("TPS7A2030_I", {}).get("stress", "") + " / 300 mA = 10 %",
     "Receiver rail (EMC rule E4): CC1200, LNA, TCXO. Headroom 0.19 V min. Finish NiPdAu",
     "TI orderable addendum read 2026-10-27: Active (TPS7A2030PDQNR listed)")
C("C803", "1uF", "+3V3_RAD", "GND", 3.4)
C("C804", "2.2uF", "+3V0_RX", "GND", 3.05)
C("C805", "100nF", "+3V0_RX", "GND", 3.05)
part("U802", "LDO 150 mA adjustable", "3.10 V (V_DIG)", "TPS73101DBVR", "Texas Instruments", "SOT-23-5",
     "150 mA, 5.5 V in, -40..125 C",
     [(1, "IN", PI, "+3V3_RAD"), (2, "GND", PI, "GND"), (3, "EN", I, "+3V3_RAD"), (4, "FB", I, "V_DIG_FB"),
      (5, "OUT", PO, "+3V1_DIG")], ST.get("TPS73101_I", {}).get("stress", ""),
     "Vout = 1.204 V x (1 + 31.6k/20.0k) = 3.106 V. TNC MCU, CMX138A, key logic. Finish SN: re-tin",
     "TI orderable addendum read: Active")
R("R801", "31.6k", "+3V1_DIG", "V_DIG_FB", 2.0, thin=True)
R("R802", "20.0k", "V_DIG_FB", "GND", 1.3, thin=True)
C("C806", "1uF", "+3V3_RAD", "GND", 3.4)
C("C807", "2.2uF", "+3V1_DIG", "GND", 3.15)
C("C808", "100nF", "+3V1_DIG", "GND", 3.15)
FB("FB802", "+3V1_DIG", "+3V1_AUD", 0.009, notes="CMX138A AVDD")
C("C809", "10uF", "+3V1_AUD", "GND", 3.15, pkg="0805")
C("C810", "100nF", "+3V1_AUD", "GND", 3.15)
FB("FB803", "PA_5V_IN", "+5V_PA", 0.47, mpn="BLM18KG221SN1D", notes="PA rail entry, 2.2 A part")
C("C811", "22uF", "+5V_PA", "GND", 5.15, pkg="1210")
C("C812", "22uF", "+5V_PA", "GND", 5.15, pkg="1210")
C("C813", "100nF", "+5V_PA", "GND", 5.15)
part("U803", "LDO 300 mA low noise", "3.3 V (V_PA3)", "TPS7A2033PDQNR", "Texas Instruments", "X2SON-4 1x1 mm",
     "300 mA, 6.0 V in max", [(1, "OUT", PO, "+3V3_PA"), (2, "GND", PI, "GND"), (3, "EN", I, "+5V_PA"),
                              (4, "IN", PI, "+5V_PA"), (5, "PAD", P, "GND")],
     ST.get("TPS7A2033_I", {}).get("stress", "") + "; " + ST.get("TPS7A2033_I", {}).get("thermal", ""),
     "PA-domain logic, detectors, PCM1808 VDD, clock squarer: unpowered unless SW3 (RF_TX_EN) is on",
     "TI orderable addendum read: Active")
C("C814", "1uF", "+5V_PA", "GND", 5.15)
C("C815", "1uF", "+3V3_PA", "GND", 3.35)
FB("FB804", "+5V_PA", "+5V_PA_A", 0.009, notes="PCM1808 VCC (analog)")
C("C816", "10uF", "+5V_PA_A", "GND", 5.15, pkg="0805")
C("C817", "100nF", "+5V_PA_A", "GND", 5.15)
# TX permit detectors: two independent dividers on PA_5V
R("R803", "10.0k", "+5V_PA", "PA_SENSE_A", 2.6)
R("R804", "10.0k", "PA_SENSE_A", "GND", 2.6)
R("R805", "10.0k", "+5V_PA", "PA_SENSE_B", 2.6)
R("R806", "10.0k", "PA_SENSE_B", "GND", 2.6)
part("U804", "Schmitt buffer", "TX_PERMIT_HW", "SN74LVC1G17DBVR", "Texas Instruments", "SOT-23-5",
     "5.5 V, -40..125 C, Ioff", [(1, "NC", NCt, None), (2, "A", I, "PA_SENSE_A"), (3, "GND", PI, "GND"),
                                 (4, "Y", O, "TX_PERMIT_HW"), (5, "VCC", PI, "+3V1_DIG")],
     "input 2.58 V max on 3.1 V VCC", "High when PA_5V > ~3.6 V (VT+ 1.8 V max x 2). Finish NiPdAu|SN",
     "TI orderable addendum read: Active")
part("U805", "Load switch 2 A", "exciter rail", "TPS22918DBVR", "Texas Instruments", "SOT-23-6",
     "5.5 V, 2 A, -40..105 C", [(1, "VIN", PI, "+3V3_RAD"), (2, "GND", PI, "GND"), (3, "ON", I, "PA_SENSE_B"),
                                (4, "CT", P, "LMX_CT"), (5, "QOD", P, "+3V3_LMX"), (6, "VOUT", PO, "+3V3_LMX")],
     ST.get("TPS22918_LMX", {}).get("stress", ""),
     "Powers LMX2571 only while PA_5V is present (second, independent TX-permit path). Finish NiPdAu|SN",
     "TI orderable addendum read: Active")
C("C818", "1nF", "LMX_CT", "GND", 3.4, kind="C0G", notes="slew ~ 0.7 ms")
C("C819", "10uF", "+3V3_LMX", "GND", 3.4, pkg="0805")
C("C820", "100nF", "+3V3_LMX", "GND", 3.4)

# ================================================================================== receive front end (100)
R("R101", "100k", "UHF_A", "GND", 0.5, notes="static bleed of the deployed tape")
R("R102", "100k", "UHF_B", "GND", 0.5, notes="static bleed of the deployed tape")
L("L101", "22nH", "UHF_SE", "UHF_A", notes="70 cm lattice balun, 50 ohm to 72 ohm differential (two monopoles)")
C("C101", "6.0pF", "UHF_A", "GND", 1.0, rf=True)
C("C102", "6.0pF", "UHF_SE", "UHF_B", 1.0, rf=True)
L("L102", "22nH", "UHF_B", "GND")
R("R103", "0", "UHF_SE", "UHF_A", 0.1, notes="DNP: fit, and omit L101/C101/C102/L102, if the AntS supplies its own phasing",
  dnp=True)
fl1 = CALC.get("fl1", {}).get("elements", {"L1": 22, "C1s": 56, "C2": 6.4, "L3": 12, "C3s": 100, "C4": 5.8, "L5": 12,
                                            "C5s": 100, "C6": 6.4, "L7": 20})
L("L103", f"{nv(fl1['L1'])}nH", "UHF_SE", "FL1_T1", notes="FL1 arm 1 (trap 146.5 MHz)")
C("C103", f"{nv(fl1['C1s'])}pF", "FL1_T1", "GND", 1.0, rf=True)
C("C104", f"{nv(fl1['C2'])}pF", "UHF_SE", "FL1_N2", 1.0, rf=True)
L("L104", f"{nv(fl1['L3'])}nH", "FL1_N2", "FL1_T3", notes="FL1 arm 3 (trap 145.0 MHz)")
C("C105", f"{nv(fl1['C3s'])}pF", "FL1_T3", "GND", 1.0, rf=True)
C("C106", f"{nv(fl1['C4'])}pF", "FL1_N2", "FL1_N4", 1.0, rf=True)
L("L105", f"{nv(fl1['L5'])}nH", "FL1_N4", "FL1_T5", notes="FL1 arm 5 (trap 143.5 MHz)")
C("C107", f"{nv(fl1['C5s'])}pF", "FL1_T5", "GND", 1.0, rf=True)
C("C108", f"{nv(fl1['C6'])}pF", "FL1_N4", "LNA_IN", 1.0, rf=True)
L("L106", f"{nv(fl1['L7'])}nH", "LNA_IN", "GND", notes="FL1 last shunt arm; DC ground of the LNA input")

# ================================================================================ LNA, SAW, CC1200 (200)
C("C201", "100pF", "LNA_IN", "LNA_B_RF", 1.0, rf=True, notes="DC block")
L("L201", "27nH", "LNA_B_RF", "LNA_B", notes="input match (starting value; tuned for NF on the EM)")
part("Q201", "SiGe RF transistor", "BFP640F", "BFP640FH6327XTSA1", "Infineon", "TSFP-4",
     "VCEO 4.1 V, IC 50 mA, Ptot 200 mW, Tj 150 C",
     [(1, "B", P, "LNA_B"), (2, "E", P, "GND"), (3, "C", P, "LNA_C"), (4, "E2", P, "GND")],
     "Vce 2.0 V / 4.1 V = 49 %; Ic 6 mA / 50 mA = 12 %; 12 mW / 200 mW = 6 %",
     "NFmin 0.55 dB at 450 MHz, 6 mA (datasheet Rev 4.0, 2024). Order code and pinout (TSFP-4) to confirm. "
     "Infineon also offers a space-qualified BFP640 variant (HiRel) if radiation data are wanted",
     "datasheet found (Rev 4.0, 2024); order code not checked")
R("R201", "36.0k", "LNA_C", "LNA_B", 1.2, notes="collector-base feedback bias: Ib = (2.0-0.83)/36k = 33 uA, Ic ~ 6 mA")
L("L202", "120nH", "LNA_C", "LNA_VC", i_work=0.007, notes="collector RF choke")
R("R202", "160", "LNA_VC", "+3V0_RX", 1.0, notes="(3.0 V - 2.0 V)/6.1 mA = 164 ohm")
C("C202", "1nF", "LNA_VC", "GND", 3.05, kind="C0G")
C("C203", "100nF", "LNA_VC", "GND", 3.05)
R("R203", "47", "LNA_C", "LNA_STAB", 0.2, notes="low-frequency stability network")
C("C204", "22pF", "LNA_STAB", "GND", 2.0, rf=True)
C("C205", "100pF", "LNA_C", "LNA_OUT", 2.0, rf=True, notes="DC block")
part("U201", "SAW filter 435 MHz", "TA2209A", "TA2209A", "Tai-Saw Technology", "SMD 3.8 x 3.8 mm (8 pad)",
     "+10 dBm, 3 V DC, -40..+85 C",
     [(1, "G1", P, "GND"), (2, "IN", P, "LNA_OUT"), (3, "G3", P, "GND"), (4, "G4", P, "GND"), (5, "G5", P, "GND"),
      (6, "OUT", P, "SAW_OUT"), (7, "G7", P, "GND"), (8, "G8", P, "GND")],
     "max input -37 dBm (2 m fundamental after FL1 and LNA) vs +10 dBm", "Passband 433-437 MHz; >= 45 dB at 10-385 "
     "and 485-1000 MHz; TCF -36 ppm/K. Metal lid: treat as " + MAG_FAR, "approval sheet read (2017); production status not confirmed")
# SAW (single-ended) to CC1200 differential LNA input; starting values, final from TI's CC1200 420-470 MHz reference
L("L203", "22nH", "SAW_OUT", "CC_LNA_P", notes="RX balun (starting value; replace by the TI reference design values)")
C("C206", "5.6pF", "CC_LNA_P", "GND", 1.0, rf=True)
C("C207", "5.6pF", "SAW_OUT", "CC_LNA_N", 1.0, rf=True)
L("L204", "22nH", "CC_LNA_N", "GND", notes="DC path to ground (LNA_N)")
L("L205", "270nH", "CC_LNA_P", "GND", notes="DC path to ground (LNA_P), high impedance at 436 MHz")
L("L206", "47nH", "CC_PA", "+3V0_RX", notes="DC path to VDD for the unused PA pin (datasheet requirement)")
C("C208", "1.0pF", "CC_PA", "CC_TRX", 1.0, rf=True, dnp=True, notes="DNP: PA not used (receive only)")
cc_pins = [(1, "VDD_GUARD", PI, "+3V0_RX"), (2, "RESET_N", I, "CC_RESETN"), (3, "GPIO3", B, None),
           (4, "GPIO2", B, "CC_GPIO2"), (5, "DVDD", PI, "+3V0_RX"), (6, "DCPL", P, "CC_DCPL"), (7, "SI", I, "SPI_MOSI"),
           (8, "SCLK", I, "SPI_SCK"), (9, "SO", TS, "SPI_MISO"), (10, "GPIO0", B, "CC_CFMCLK"), (11, "CSN", I, "CC_CSN"),
           (12, "DVDD2", PI, "+3V0_RX"), (13, "AVDD_IF", PI, "+3V0_RX"), (14, "RBIAS", P, "CC_RBIAS"),
           (15, "AVDD_RF", PI, "+3V0_RX"), (16, "NC16", NCt, None), (17, "PA", P, "CC_PA"), (18, "TRX_SW", P, "CC_TRX"),
           (19, "LNA_P", P, "CC_LNA_P"), (20, "LNA_N", P, "CC_LNA_N"), (21, "DCPL_VCO", P, "CC_DCPL_VCO"),
           (22, "AVDD_SYNTH1", PI, "+3V0_RX"), (23, "LPF0", P, "CC_LPF0"), (24, "LPF1", P, "CC_LPF1"),
           (25, "AVDD_PFD_CHP", PI, "+3V0_RX"), (26, "DCPL_PFD_CHP", P, "CC_DCPL_PFD"), (27, "AVDD_SYNTH2", PI, "+3V0_RX"),
           (28, "AVDD_XOSC", PI, "+3V0_RX"), (29, "DCPL_XOSC", P, "CC_DCPL_XOSC"), (30, "XOSC_Q1", P, "GND"),
           (31, "XOSC_Q2", P, None), (32, "EXT_XOSC", I, "CC_EXTOSC"), (33, "GND_PAD", PI, "GND")]
part("U202", "Narrowband receiver (TRX used RX-only)", "CC1200", "CC1200RHBR", "Texas Instruments", "VQFN-32 5x5 (RHB)",
     "2.0-3.6 V, +10 dBm input, -40..+85 C", cc_pins,
     "21 mA x 3.05 V = 64 mW; input <= -82 dBm at 146 MHz vs +10 dBm saturation",
     "Receive only, 435.000-437.300 MHz, custom frequency modulation (CFM, analog FM) soft output read over SPI at "
     "12.5 kS/s; GPIO0 = CLKEN_CFM; GPIO1/SO high-impedance when CSn high (IOCFG1). Finish NiPdAu",
     "TI orderable addendum read 2026-10-27: Active")
C("C209", "100nF", "CC_DCPL", "GND", 1.9)
C("C210", "100nF", "CC_DCPL_VCO", "GND", 1.9)
C("C211", "100nF", "CC_DCPL_PFD", "GND", 1.9)
C("C212", "100nF", "CC_DCPL_XOSC", "GND", 1.9)
R("R204", "56.0k", "CC_RBIAS", "GND", 1.2, thin=True, notes="RBIAS precision resistor (TI reference value)")
C("C213", "1nF", "CC_LPF0", "CC_LPF1", 1.0, kind="C0G", notes="synthesizer loop capacitor (TI reference design value "
                                                                    "to confirm)")
for k_, pin in enumerate(["V1", "V5", "V12", "V13", "V15", "V22", "V25", "V27", "V28"]):
    C(f"C2{20 + k_}", "100nF", "+3V0_RX", "GND", 3.05, notes=f"CC1200 decoupling ({pin})")
C("C229", "1nF", "TCXO_OUT", "CC_EXTOSC", 3.05, kind="C0G", notes="TCXO clipped sine to EXT_XOSC (0.8-1.5 Vpp)")
R("R205", "10.0k", "CC_RESETN", "+3V1_DIG", 3.15, notes="RESET_N pull-up")

# ================================================================================ reference (TCXO, 300)
part("Y301", "TCXO 40 MHz clipped sine", "40.000 MHz +-0.5 ppm", "TG2016SMN 40.0000M-MCGNNM3", "Seiko Epson",
     "2.0 x 1.6 mm 4-pad", "2.8-3.3 V, -40..+85 C, +-0.5 ppm",
     [(1, "NC_VC", P, None), (2, "GND", PI, "GND"), (3, "OUT", O, "TCXO_OUT"), (4, "VCC", PI, "+3V0_RX")],
     "1.5 mA", "One reference for CC1200, LMX2571 and PCM1808. Pad assignment to confirm against the Epson sheet; "
     "40 MHz code inferred from the family naming (26 and 38.4 MHz codes listed by distributors). Metal lid: treat as "
     + MAG_FAR,
     "family listed by distributors (Future, Mouser); 40 MHz code not confirmed")
C("C301", "100nF", "+3V0_RX", "GND", 3.05, notes="TCXO decoupling")
C("C302", "1nF", "TCXO_OUT", "CLK_SQ_IN", 3.05, kind="C0G")
R("R301", "1.00M", "CLK_SQ_IN", "CLK_SQ", 1.7, notes="self-bias of the squarer")
part("U301", "Unbuffered inverter (squarer)", "SN74LVC1GU04", "SN74LVC1GU04DBVR", "Texas Instruments", "SOT-23-5",
     "5.5 V, -40..125 C, Ioff", [(1, "NC", NCt, None), (2, "A", I, "CLK_SQ_IN"), (3, "GND", PI, "GND"),
                                 (4, "Y", O, "CLK_SQ"), (5, "VCC", PI, "+3V3_PA")],
     "40 MHz into 3 loads, CpdV2f ~ 4 mW", "PA domain: runs only while TX is permitted. Input AC-coupled (+-0.4 V when "
     "unpowered: no diode conduction)", "TI orderable addendum read: Active")
R("R302", "47", "CLK_SQ", "CLK_LMX", 0.3, notes="series termination")
C("C303", "100pF", "CLK_LMX", "LMX_OSCIN", 3.3, rf=True, notes="AC coupling to OSCin")
part("U302", "Buffer with OE (SCKI gate)", "SN74LVC1G126", "SN74LVC1G126DBVR", "Texas Instruments", "SOT-23-5",
     "5.5 V, -40..125 C", [(1, "OE", I, "PTT_EXC"), (2, "A", I, "CLK_SQ"), (3, "GND", PI, "GND"), (4, "Y", TS, "SCKI_G"),
                           (5, "VCC", PI, "+3V3_PA")], "-",
     "SCKI runs only while PTT_EXCITER: the PCM1808 powers down when its clock stops (datasheet §7.4.2)",
     "TI orderable addendum read: Active")
R("R303", "33", "SCKI_G", "PCM_SCKI", 0.3, notes="series termination (edge currents only)")
R("R304", "100k", "PCM_SCKI", "GND", 3.3, notes="hold SCKI low when the buffer is Hi-Z")

# =============================================================================================== TNC MCU (300)
mcu = [(1, "VBAT", PI, "+3V1_DIG"), (2, "PC13", B, None), (3, "PC14", B, None), (4, "PC15", B, None),
       (5, "PH0", B, None), (6, "PH1", B, None), (7, "NRST", B, "MCU_NRST"), (8, "VSSA", PI, "GND"),
       (9, "VDDA", PI, "+3V1_DIG"), (10, "PA0", B, "TP_MCU_PA0"), (11, "PA1", I, "TX_PERMIT_HW"),
       (12, "PA2_USART2_TX", O, "UART_TNC_TX"), (13, "PA3_USART2_RX", I, "UART_OBC_TX"),
       (14, "PA4_DAC1_OUT1", O, "TNC_AF_RAW"), (15, "PA5_DAC1_OUT2", O, "RPT_AF_RAW"), (16, "PA6", I, "CMX_IRQN"),
       (17, "PA7", O, "TONE_OK"), (18, "PB0", O, "CC_CSN"), (19, "PB1", O, "CMX_CSN"), (20, "PB2", O, "LMX_LE_MCU"),
       (21, "PB10", I, "CC_CFMCLK"), (22, "PB11", I, "CC_GPIO2"), (23, "VSS", PI, "GND"), (24, "VDD", PI, "+3V1_DIG"),
       (25, "PB12", O, "CC_RESETN"), (26, "PB13", I, "EN_OK"), (27, "PB14", I, "LMX_LD"), (28, "PB15", I, "XPDR_KEY"),
       (29, "PA8", O, "TNC_PTT"), (30, "PA9_USART1_TX", O, "TP_DBG_TX"), (31, "PA10_USART1_RX", I, "TP_DBG_RX"),
       (32, "PA11", B, None), (33, "PA12", B, None), (34, "PA13_SWDIO", B, "SWDIO"), (35, "VSS2", PI, "GND"),
       (36, "VDD2", PI, "+3V1_DIG"), (37, "PA14_SWCLK", I, "SWCLK"), (38, "PA15", B, None),
       (39, "PB3_SPI1_SCK", O, "SPI_SCK"), (40, "PB4_SPI1_MISO", I, "SPI_MISO"), (41, "PB5_SPI1_MOSI", O, "SPI_MOSI"),
       (42, "PB6", I, "PTT_EXC"), (43, "PB7", O, "RX_ALIVE"), (44, "BOOT0", I, "MCU_BOOT0"), (45, "PB8", B, None),
       (46, "PB9", B, None), (47, "VSS3", PI, "GND"), (48, "VDD3", PI, "+3V1_DIG")]
part("U310", "MCU Cortex-M4 80 MHz, 256 KB flash", "STM32L431CCT6", "STM32L431CCT6", "STMicroelectronics",
     "LQFP-48 7x7", "1.71-3.6 V, -40..+85 C", mcu,
     "3-5 mA at 16 MHz; Tj ~ board + 1 C",
     "KISS TNC (AFSK 1200 modem, HDLC, KISS), CC1200/CMX138A/LMX2571 set-up and scrubbing, repeat-audio DAC, TONE_OK "
     "relay. HSI16 clock (no crystal), IWDG on the LSI. LQFP-48 pin numbers from the STM32L431 datasheet DS11453 "
     "Table 15 as remembered: to confirm at schematic review (ST site not reached)",
     "ST product page not reached; distributor listings show the family active")
C("C311", "100nF", "+3V1_DIG", "GND", 3.15, notes="VDD")
C("C312", "100nF", "+3V1_DIG", "GND", 3.15, notes="VDD2")
C("C313", "100nF", "+3V1_DIG", "GND", 3.15, notes="VDD3")
C("C314", "100nF", "+3V1_DIG", "GND", 3.15, notes="VBAT")
C("C315", "1uF", "+3V1_DIG", "GND", 3.15, notes="VDDA")
C("C316", "100nF", "MCU_NRST", "GND", 3.15, notes="NRST")
R("R305", "10.0k", "MCU_BOOT0", "GND", 0.1, notes="boot from flash")
R("R306", "1.00k", "UART_OBC_TX_J", "UART_OBC_TX", 0.1, notes="series, OBC USART2 TX (PD5)")
R("R307", "100", "UART_TNC_TX", "UART_TNC_TX_J", 0.1, notes="series, to OBC USART2 RX (PD6)")

# ======================================================================== XPDR_EN gate and key logic (400)
R("R401", "1.00k", "XPDR_EN_J", "XPDR_EN_F", 0.1, notes="series")
R("R402", "100k", "XPDR_EN_F", "GND", 3.4, notes="pull-down: an unpowered or reset OBC reads as no heartbeat")
part("U401", "Retriggerable monostable", "SN74LVC1G123", "SN74LVC1G123DCTR", "Texas Instruments", "SSOP-8 (DCT)",
     "5.5 V, -40..125 C, Ioff", [(1, "A", I, "GND"), (2, "B", I, "XPDR_EN_F"), (3, "CLR", I, "POR_N"),
                                 (4, "GND", PI, "GND"), (5, "Q", O, "EN_OK"), (6, "CEXT", P, "MONO_C"),
                                 (7, "REXT_CEXT", P, "MONO_RC"), (8, "VCC", PI, "+3V1_DIG")],
     "-", "tw = Rext x Cext = 226k x 0.22 uF = 49.7 ms (46.8-58.0 ms over K, C, R); retriggered every 10 ms by the "
     "100 Hz heartbeat; a level held high or low stops the edges", "TI orderable addendum read: Active")
C("C401", "220nF", "MONO_C", "MONO_RC", 3.15, kind="C0G", pkg="1206", notes="C0G 0.22 uF (GRM31C5C1E224JE01-class)")
R("R403", "226k", "MONO_RC", "+3V1_DIG", 3.15, thin=True)
R("R404", "100k", "POR_N", "+3V1_DIG", 3.15, notes="power-on clear")
C("C402", "100nF", "POR_N", "GND", 3.15, notes="power-on clear: 10 ms")
C("C403", "100nF", "+3V1_DIG", "GND", 3.15, notes="U401 decoupling")
part("U402", "Dual inverter", "SN74LVC2G04", "SN74LVC2G04DBVR", "Texas Instruments", "SOT-23-6", "5.5 V, -40..125 C",
     [(1, "1A", I, "TONE_OK"), (2, "GND", PI, "GND"), (3, "2A", I, "TOT_Q14"), (4, "2Y", O, "NOT_TOT"),
      (5, "VCC", PI, "+3V1_DIG"), (6, "1Y", O, "TONE_N")], "-", "TONE_N resets the key-down timer when the tone drops",
     "TI orderable addendum read: Active")
part("U403", "14-stage counter with oscillator", "CD74HC4060", "CD74HC4060PWR", "Texas Instruments", "TSSOP-16",
     "2-6 V, -55..125 C",
     [(1, "Q12", O, None), (2, "Q13", O, None), (3, "Q14", O, "TOT_Q14"), (4, "Q6", O, None), (5, "Q5", O, None),
      (6, "Q7", O, None), (7, "Q4", O, None), (8, "GND", PI, "GND"), (9, "PHI_O", O, "OSC_CT"),
      (10, "PHI_O_N", O, "OSC_RT"), (11, "PHI_I", I, "OSC_IN"), (12, "MR", I, "TONE_N"), (13, "Q9", O, None),
      (14, "Q8", O, None), (15, "Q10", O, None), (16, "VCC", PI, "+3V1_DIG")],
     "-", "Key-down timer: RC oscillator 68.3 Hz, Q14 high after 8192 periods = 120 s, then D401 stops the oscillator "
     "(latched until the tone drops). Finish: see addendum", "TI datasheet read; CD74HC4060PWR listed")
R("R405", "56.2k", "OSC_RT", "OSC_TRIM", 3.15, thin=True, notes="Rt (with R406)")
R("R406", "6.81k", "OSC_TRIM", "OSC_X", 3.15, thin=True, notes="select at test 0-10 k: 120 +- 1 s at 25 C")
C("C404", "100nF", "OSC_CT", "OSC_X", 3.15, kind="C0G", pkg="1206", notes="Ct, C0G")
R("R407", "120k", "OSC_X", "OSC_IN", 3.15, notes="Rs")
part("D401", "Switching diode", "BAS16", "BAS16-E3-08", "Vishay", "SOT-23", "100 V, 250 mA",
     [(1, "A", P, "TOT_Q14"), (2, "NC", NCt, None), (3, "K", P, "OSC_IN")], "3.1 V / 100 V; < 1 mA",
     "Stops the oscillator when Q14 rises (latches the timeout). Pinout: anode 1, cathode 3 (SOT-23 BAS16)",
     "not checked")
C("C405", "100nF", "+3V1_DIG", "GND", 3.15, notes="U403 decoupling")
part("U404", "3-input AND", "SN74LVC1G11", "SN74LVC1G11DBVR", "Texas Instruments", "SOT-23-6", "5.5 V, -40..125 C",
     [(1, "A", I, "EN_OK"), (2, "GND", PI, "GND"), (3, "B", I, "TONE_OK"), (4, "Y", O, "XPDR_KEY"),
      (5, "VCC", PI, "+3V1_DIG"), (6, "C", I, "NOT_TOT")], "-", "XPDR_KEY = EN_OK . TONE_OK . NOT_TOT",
     "TI orderable addendum read: Active")
part("U405", "2-input OR", "SN74LVC1G32", "SN74LVC1G32DBVR", "Texas Instruments", "SOT-23-5", "5.5 V, -40..125 C",
     [(1, "A", I, "TNC_PTT"), (2, "B", I, "XPDR_KEY"), (3, "GND", PI, "GND"), (4, "Y", O, "PTT_ANY"),
      (5, "VCC", PI, "+3V1_DIG")], "-", "TNC frames or repeat key", "TI orderable addendum read: Active")
part("U406", "2-input AND", "SN74LVC1G08", "SN74LVC1G08DBVR", "Texas Instruments", "SOT-23-5", "5.5 V, -40..125 C",
     [(1, "A", I, "PTT_ANY"), (2, "B", I, "TX_PERMIT_HW"), (3, "GND", PI, "GND"), (4, "Y", O, "PTT_EXC"),
      (5, "VCC", PI, "+3V1_DIG")], "-",
     "PTT_EXCITER = TX_PERMIT_HW . (TNC_PTT + XPDR_KEY); the audio select follows TNC_PTT, so frames take the "
     "transmitter first", "TI orderable addendum read: Active")
C("C406", "100nF", "+3V1_DIG", "GND", 3.15, notes="U404-U406 decoupling")
R("R408", "100k", "TNC_PTT", "GND", 3.15, notes="pull-down: MCU in reset = no PTT")
R("R409", "100k", "TONE_OK", "GND", 3.15, notes="pull-down: MCU in reset = no tone")
R("R410", "1.00k", "TONE_OK", "TONE_OK_RB", 0.1, notes="readback to an OBC input (J1-8; STM32 pin allocated by the OBC designer)")
R("R411", "1.00k", "XPDR_KEY", "XPDR_KEY_RB", 0.1, notes="readback to an OBC input (J1-9; STM32 pin allocated by the OBC designer)")
R("R412", "1.00k", "PTT_EXC", "PTT_EXC_RB", 0.1, notes="readback to an OBC input (J1-10; a timer-capture-capable pin is preferred for keyed-time measurement; allocated by the OBC designer)")
R("R413", "1.00k", "TX_PWR_SEL_J", "TX_PWR_SEL_F", 0.1, notes="series")
R("R414", "100k", "TX_PWR_SEL_F", "GND", 3.4, notes="pull-down: 0.5 W at OBC reset")
part("U407", "Dual inverter", "SN74LVC2G04", "SN74LVC2G04DBVR", "Texas Instruments", "SOT-23-6", "5.5 V, -40..125 C",
     [(1, "1A", I, "TX_PWR_SEL_F"), (2, "GND", PI, "GND"), (3, "2A", I, "GND"), (4, "2Y", O, None),
      (5, "VCC", PI, "+3V1_DIG"), (6, "1Y", O, "PWR_LO")], "-", "PWR_LO high selects the 0.5 W APC reference; "
     "gate 2 unused (RX_FAULT has its own inverter, U482)",
     "TI orderable addendum read: Active")

# ===================================================== receiver supervision lines to the OBC (480, RID RFB-07)
# RX_CD: CC1200 GPIO2 configured as carrier sense (IOCFG2 = CARRIER_SENSE, RSSI threshold AGC_CS_THR), buffered.
# RX_FAULT: hardware monostable on the TNC's RX_ALIVE pulse train; the TNC pulses RX_ALIVE every 100 ms only while
# the receive path passes its health checks (RF_BOARD_DESIGN §8.6). Missing pulses (TNC hung, in reset, receiver
# failed) -> RX_OK falls within 0.36-0.62 s -> RX_FAULT high. Unpowered board: buffers at 0 V, OBC pull-downs -> not fault.
part("U480", "Schmitt buffer", "RX_CD buffer", "SN74LVC1G17DBVR", "Texas Instruments", "SOT-23-5",
     "5.5 V, -40..125 C, Ioff", [(1, "NC", NCt, None), (2, "A", I, "CC_GPIO2"), (3, "GND", PI, "GND"),
                                 (4, "Y", O, "RX_CD"), (5, "VCC", PI, "+3V1_DIG")],
     "-", "Isolates the CC1200 GPIO2 (3.0 V domain) from the harness; Ioff: no back-powering when unpowered",
     "TI orderable addendum read: Active")
R("R480", "1.00k", "RX_CD", "RX_CD_J", 0.1, notes="series to the OBC input (J1-15)")
R("R481", "100k", "RX_CD_J", "GND", 3.15, notes="pull-down: unwired or unpowered reads no carrier")
part("U481", "Retriggerable monostable (RX health)", "SN74LVC1G123", "SN74LVC1G123DCTR", "Texas Instruments",
     "SSOP-8 (DCT)", "5.5 V, -40..125 C, Ioff",
     [(1, "A", I, "GND"), (2, "B", I, "RX_ALIVE"), (3, "CLR", I, "POR_N"), (4, "GND", PI, "GND"), (5, "Q", O, "RX_OK"),
      (6, "CEXT", P, "RXH_C"), (7, "REXT_CEXT", P, "RXH_RC"), (8, "VCC", PI, "+3V1_DIG")],
     "-", "tw = 470 k x 1.0 uF = 0.47 s (0.36-0.62 s: K 1.00-1.10, X7R -15/+10 % over temperature and bias, leakage); "
     "retriggered every 100 ms by RX_ALIVE", "TI orderable addendum read: Active")
C("C480", "1uF", "RXH_C", "RXH_RC", 3.15, notes="X7R; timing tolerance is loose by design (0.36-0.62 s)")
R("R482", "470k", "RXH_RC", "+3V1_DIG", 3.15)
R("R483", "100k", "RX_ALIVE", "GND", 3.15, notes="pull-down: MCU in reset gives no pulses")
part("U482", "Inverter", "RX_FAULT", "SN74LVC1G04DBVR", "Texas Instruments", "SOT-23-5", "5.5 V, -40..125 C, Ioff",
     [(1, "NC", NCt, None), (2, "A", I, "RX_OK"), (3, "GND", PI, "GND"), (4, "Y", O, "RX_FAULT"),
      (5, "VCC", PI, "+3V1_DIG")], "-", "RX_FAULT = NOT RX_OK; unpowered output 0 V (not fault)",
     "TI orderable addendum read: Active")
R("R484", "1.00k", "RX_FAULT", "RX_FAULT_J", 0.1, notes="series to the OBC input (J1-16)")
R("R485", "100k", "RX_FAULT_J", "GND", 3.15, notes="pull-down: unwired or unpowered reads not fault")
C("C481", "100nF", "+3V1_DIG", "GND", 3.15, notes="U480-U482 decoupling")
for tp, net in (("TP23", "RX_CD"), ("TP24", "RX_FAULT"), ("TP25", "RX_ALIVE")):
    part(tp, "Test pad", net, "pad 1.0 mm (no part)", "-", "TP_1.0mm", "-", [(1, "TP", P, net)], "-", "test pad", "n/a")

# ===================================================================== audio: repeat path and TX select (450)
part("U451", "Dual op-amp RRIO", "TLV9002", "TLV9002IDR", "Texas Instruments", "SOIC-8", "1.8-5.5 V, -40..125 C",
     [(1, "OUT1", O, "RPT_DISC"), (2, "IN1N", I, "RPT_FB"), (3, "IN1P", I, "RPT_SK2"), (4, "VM", PI, "GND"),
      (5, "IN2P", I, "TNC_SK2"), (6, "IN2N", I, "TNC_AF"), (7, "OUT2", O, "TNC_AF"), (8, "VP", PI, "+3V1_DIG")],
     "-", "Ch1: 3rd-order reconstruction low-pass (fc 4 kHz) of the repeat audio from DAC2 into the CMX138A; ch2: "
     "AFSK low-pass/buffer from DAC1. Finish NiPdAu", "TI orderable addendum read: Active")
R("R451", "10.0k", "RPT_AF_RAW", "RPT_SK1", 3.1)
C("C451", "3.3nF", "RPT_SK1", "GND", 3.1, kind="C0G", notes="1st-order pole 4.8 kHz")
R("R452", "10.0k", "RPT_SK1", "RPT_SK1B", 3.1)
R("R453", "10.0k", "RPT_SK1B", "RPT_SK2", 3.1)
C("C452", "5.6nF", "RPT_SK1B", "RPT_DISC", 3.1, kind="C0G", notes="Sallen-Key (unity gain) 4 kHz, Q 0.7")
C("C453", "2.7nF", "RPT_SK2", "GND", 3.1, kind="C0G")
R("R454", "0", "RPT_DISC", "RPT_FB", 0.0, notes="unity-gain follower link")
R("R455", "10.0k", "TNC_AF_RAW", "TNC_SK1", 3.1)
C("C454", "1.5nF", "TNC_SK1", "GND", 3.1, kind="C0G", notes="AFSK image filter 10.6 kHz")
R("R456", "10.0k", "TNC_SK1", "TNC_SK2", 3.1)
C("C455", "1.5nF", "TNC_SK2", "GND", 3.1, kind="C0G")
C("C456", "100nF", "+3V1_DIG", "GND", 3.15, notes="U451 decoupling")
# CMX138A: hardware CTCSS decode, 300 Hz HPF, 3 kHz LPF, level limit
C("C457", "1uF", "RPT_DISC", "CMX_DISC_AC", 1.6, notes="X7R film-equivalent; audio coupling")
R("R457", "100k", "CMX_DISC_AC", "CMX_DISCN1", 1.6, notes="DISC input amplifier: gain = R458/R457 = 1")
R("R458", "100k", "CMX_DISCFB", "CMX_DISCN1", 1.6)
cmx = [(1, "TXENA", O, None), (2, "VDEC", P, "CMX_VDEC"), (3, "SYSCLK", O, None), (4, "IRQN", OC, "CMX_IRQN"),
       (5, "RDATA", TS, "SPI_MISO"), (6, "SCLK", I, "SPI_SCK"), (7, "CDATA", I, "SPI_MOSI"), (8, "CSN", I, "CMX_CSN"),
       (9, "DVDD", PI, "+3V1_DIG"), (10, "XTAL", I, "CMX_X1"), (11, "XTALN", O, "CMX_X2"), (12, "DVSS", PI, "GND"),
       (13, "MOD", O, None), (14, "MICFB", O, "CMX_MICFB"), (15, "MICN", I, "CMX_MICFB"), (16, "MICP", I, "CMX_VBIAS"),
       (17, "AVDD", PI, "+3V1_AUD"), (18, "AUXADC", I, "CMX_VBIAS"), (19, "VBIAS", PO, "CMX_VBIAS"),
       (20, "DISCN2", I, None), (21, "DISCN1", I, "CMX_DISCN1"), (22, "DISCFB", O, "CMX_DISCFB"),
       (23, "AUDIO", O, "CMX_AUDIO"), (24, "AVSS", PI, "GND"), (25, "AUXDAC", O, None), (26, "DVSS2", PI, "GND"),
       (27, "CLKSEL", I, "GND"), (28, "RXENA", O, None)]
part("U452", "Audio and sub-audio processor", "CMX138A", "CMX138AE1", "CML Microcircuits", "TSSOP-28 (E1)",
     "3.0-3.6 V, -40..+85 C", cmx, "7.0 mA x 3.15 V = 22 mW in Rx mode",
     "Rx mode only, configured and scrubbed by the TNC MCU over C-BUS (SPI1 at 1 MHz): CTCSS 67.0 Hz decode (status "
     "$C6 b11 / tone $CC, IRQN), 300-3000 Hz band filter, de-emphasis off (flat repeat), output level limit. Unused "
     "MIC amplifier: MICN to MICFB, MICP to VBIAS; AUXADC to VBIAS; DISCN2 open (datasheet §4.2 to confirm)",
     "datasheet D/138A/4 (2014) read; DigiKey: Active, -40..85 C")
C("C458", "100nF", "CMX_VDEC", "GND", 2.5)
C("C459", "10uF", "CMX_VDEC", "GND", 2.5, pkg="0805")
C("C460", "100nF", "CMX_VBIAS", "GND", 1.6)
C("C461", "100nF", "+3V1_DIG", "GND", 3.15, notes="CMX138A DVDD")
part("Y451", "Crystal 3.6864 MHz", "3.6864 MHz", "ABLS-3.6864MHZ-B4-T", "Abracon", "HC-49/US SMD",
     "+-30 ppm, CL 18 pF", [(1, "X1", P, "CMX_X1"), (2, "X2", P, "CMX_X2")], "drive < 100 uW",
     "CLKSEL low = 3.6864 MHz default; harmonics 434.995 and 438.682 MHz bracket the uplink band (6.144 MHz rejected: "
     "71st harmonic 436.224 MHz). Temperature grade (-40..85 C variant) to confirm. Can: treat as " + MAG_FAR,
     "not checked")
C("C462", "22pF", "CMX_X1", "GND", 3.15, kind="C0G")
C("C463", "22pF", "CMX_X2", "GND", 3.15, kind="C0G")
R("R459", "10.0k", "CMX_IRQN", "+3V1_DIG", 3.15, notes="IRQN pull-up (open drain)")
R("R460", "100k", "CMX_CSN", "+3V1_DIG", 3.15, notes="CSN pull-up (no internal pull-up)")
R("R461", "100k", "CC_CSN", "+3V1_DIG", 3.15, notes="CSn pull-up")
C("C464", "1uF", "CMX_AUDIO", "RPT_AF_AC", 1.6, notes="AUDIO coupling")
part("U453", "SPST switch (mute)", "TMUX1101", "TMUX1101DBVR", "Texas Instruments", "SOT-23-5", "1.08-5.5 V, -40..125 C",
     [(1, "D", P, "RPT_AF_MUTED"), (2, "S", P, "RPT_AF_AC"), (3, "GND", PI, "GND"), (4, "SEL", I, "XPDR_KEY"),
      (5, "VDD", PI, "+3V1_DIG")], "-", "Repeat audio passes only while XPDR_KEY", "TI orderable addendum read: Active")
R("R462", "100k", "RPT_AF_MUTED", "+3V1_DIG", 3.15, notes="mid-rail bias of the repeat input (with R463)")
R("R463", "100k", "RPT_AF_MUTED", "GND", 3.15)
R("R464", "100k", "RPT_AF_AC", "CMX_VBIAS", 1.6, notes="bias the switch source when open")
part("U454", "SPDT switch (2:1 audio select)", "TMUX1219", "TMUX1219DBVR", "Texas Instruments", "SOT-23-6",
     "1.08-5.5 V, -40..125 C", [(1, "SEL", I, "TNC_PTT"), (2, "VDD", PI, "+3V1_DIG"), (3, "GND", PI, "GND"),
                                (4, "S1", P, "RPT_AF_MUTED"), (5, "D", P, "TX_AUDIO_DIG"), (6, "S2", P, "TNC_AF")],
     "-", "SEL low = S1 (repeat), high = S2 (TNC): the TNC's frames take the transmitter first, with no software "
     "arbitration", "TI orderable addendum read: Active")
C("C465", "1uF", "TX_AUDIO_DIG", "PCM_VINL", 1.6, notes="coupling into the PA-domain ADC (blocks the 1.55 V bias)")
C("C466", "100nF", "+3V1_DIG", "GND", 3.15, notes="U453/U454 decoupling")

# ============================================================================== exciter (500): ADC + synthesizer
part("U501", "Audio ADC 24-bit", "PCM1808", "PCM1808PWR", "Texas Instruments", "TSSOP-14", "VCC 4.5-5.5 V, -40..+85 C",
     [(1, "VREF", P, "PCM_VREF"), (2, "AGND", PI, "GND"), (3, "VCC", PI, "+5V_PA_A"), (4, "VDD", PI, "+3V3_PA"),
      (5, "DGND", PI, "GND"), (6, "SCKI", I, "PCM_SCKI"), (7, "LRCK", O, "I2S_FS_P"), (8, "BCK", O, "I2S_CK_P"),
      (9, "DOUT", O, "I2S_DATA_P"), (10, "MD0", I, "+3V3_PA"), (11, "MD1", I, "GND"), (12, "FMT", I, "GND"),
      (13, "VINL", I, "PCM_VINL"), (14, "VINR", I, "PCM_VINR")],
     "62 mW while keyed; 0.5 mW clock-halted",
     "Master 512 fs from the 40 MHz reference: fs = 78.125 kHz, BCK 5.000 MHz; I2S; full scale 0.6 VCC = 3.0 Vpp = "
     "+-6.3 kHz deviation (the ADC's clip is the hard deviation limit). Finish NiPdAu",
     "TI orderable addendum read: Active")
C("C501", "10uF", "PCM_VREF", "GND", 2.6, pkg="0805")
C("C502", "100nF", "+5V_PA_A", "GND", 5.15, notes="PCM1808 VCC")
C("C503", "100nF", "+3V3_PA", "GND", 3.35, notes="PCM1808 VDD")
C("C504", "1uF", "PCM_VINR", "GND", 2.6, notes="unused right channel AC-grounded")
R("R501", "1.00k", "I2S_FS_P", "I2S_FS", 3.3, notes="domain crossing: limits current into an unpowered LMX2571 (fault case 3.3 V across)")
R("R502", "1.00k", "I2S_CK_P", "I2S_CK", 3.3)
R("R503", "1.00k", "I2S_DATA_P", "I2S_DATA", 3.3)
lmx = [(1, "VCC3P3_1", PI, "+3V3_LMX"), (2, "BYPASS1", P, "LMX_BYP1"), (3, "BYPASS2", P, "LMX_BYP2"),
       (4, "FSK_DV", I, "I2S_CK"), (5, "FSK_D2", I, "GND"), (6, "FSK_D1", I, "I2S_DATA"), (7, "FSK_D0", I, "I2S_FS"),
       (8, "NC8", NCt, None), (9, "VCC3P3_9", PI, "+3V3_LMX"), (10, "MUXOUT", O, "LMX_LD"), (11, "CLK", I, "LMX_CLK"),
       (12, "DATA", I, "LMX_DATA"), (13, "LE", I, "LMX_LE"), (14, "NC14", NCt, None), (15, "VCCIO_15", PI, "+3V3_LMX"),
       (16, "RFOUTRX", O, None), (17, "RFOUTTX", O, "EXC_RF"), (18, "TRCTL", I, "LMX_TRCTL"), (19, "CE", I, "LMX_CE"),
       (20, "VCC3P3_20", PI, "+3V3_LMX"), (21, "VREGVCO", P, "LMX_VREG"), (22, "VREFVCO", P, "LMX_VREF"),
       (23, "GND23", PI, "GND"), (24, "FIN", I, None), (25, "CPOUT", P, "LMX_CP"), (26, "NC26", NCt, None),
       (27, "VCC3P3_27", PI, "+3V3_LMX"), (28, "FLOUT2", O, None), (29, "FLOUT1", O, None), (30, "CPOUTEXT", O, None),
       (31, "GND31", PI, "GND"), (32, "VCPEXT", PI, "+3V3_LMX"), (33, "VCCIO_33", PI, "+3V3_LMX"),
       (34, "OSCIN", I, "LMX_OSCIN"), (35, "GND35", PI, "GND"), (36, "OSCIN_N", I, "LMX_OSCINN"), (37, "DAP", PI, "GND")]
part("U502", "Synthesizer with FSK/analog FM", "LMX2571", "LMX2571NJKR", "Texas Instruments", "WQFN-36 6x6 (NJK)",
     "3.15-3.45 V, -40..+85 C, Tj 125 C", lmx, ST.get("LMX2571_Tj", {}).get("stress", ""),
     "145.800-146.000 MHz, fVCO 4668.8 MHz / 32, fPD 40 MHz; FSK I2S mode: the PCM1808 samples set the deviation "
     "(DEN 13,002,778: 0.192 Hz per LSB, +-6.3 kHz full scale); RFoutTx 0 dBm max drives the PA directly; CE = "
     "PTT_EXCITER; supply switched by the TX permit. Finish SN (QFN): conformal-coat mitigation",
     "TI orderable addendum read 2026-10-27: Active")
for k_, n_ in enumerate(["1", "9", "20", "27", "15", "33", "32"]):
    C(f"C5{10 + k_}", "100nF", "+3V3_LMX", "GND", 3.4, notes=f"LMX2571 pin {n_}")
C("C517", "100nF", "LMX_BYP1", "GND", 1.8)
C("C518", "100nF", "LMX_BYP2", "GND", 1.8)
C("C519", "2.2uF", "LMX_VREG", "GND", 2.5)
C("C520", "100nF", "LMX_VREF", "GND", 2.5)
C("C521", "3.9nF", "LMX_CP", "GND", 3.3, kind="C0G", notes="loop filter C1 (40 kHz loop, Icp 1.25 mA, Kvco 56 MHz/V)")
R("R504", "430", "LMX_CP", "LMX_LF2", 0.1, notes="loop filter R2")
C("C522", "39nF", "LMX_LF2", "GND", 3.3, kind="C0G", pkg="0805", notes="loop filter C2")
C("C523", "100pF", "LMX_OSCINN", "GND", 3.3, rf=True, notes="OSCin* AC ground (single-ended reference)")
R("R505", "1.00k", "PTT_EXC", "LMX_CE", 0.1, notes="CE series (domain crossing)")
R("R506", "10.0k", "LMX_CE", "GND", 3.3, notes="CE pull-down")
R("R507", "10.0k", "LMX_TRCTL", "GND", 3.3, notes="TrCtl low: F1/TX port as programmed")
part("U503", "Triple buffer (MICROWIRE crossing)", "SN74LVC3G34", "SN74LVC3G34DCTR", "Texas Instruments", "SSOP-8 (DCT)",
     "5.5 V, -40..125 C, Ioff", [(1, "1A", I, "SPI_SCK"), (2, "3Y", O, "LMX_LE"), (3, "2A", I, "SPI_MOSI"),
                                 (4, "GND", PI, "GND"), (5, "2Y", O, "LMX_DATA"), (6, "3A", I, "LMX_LE_MCU"),
                                 (7, "1Y", O, "LMX_CLK"), (8, "VCC", PI, "+3V3_LMX")],
     "-", "Powered from the switched LMX rail: Ioff keeps the shared SPI bus from back-powering an unpowered LMX2571",
     "TI orderable addendum read: Active")
R("R508", "100k", "LMX_LE_MCU", "GND", 3.15, notes="LE idle low")

# =============================================================================== PA, match, LPF, coupler (600)
C("C601", "100pF", "EXC_RF", "PA_PAD_IN", 3.3, rf=True, notes="DC block")
R("R601", "300", "PA_PAD_IN", "GND", 0.3, notes="3 dB pi pad (isolation LMX2571 to PA)")
R("R602", "18", "PA_PAD_IN", "PA_IN", 0.2)
R("R603", "300", "PA_IN", "GND", 0.3)
L("L601", "36nH", "PA_IN", "PA_G1_RF", notes="input match (Mitsubishi 135-175 MHz EVB values; tuned on the EM)")
C("C602", "16pF", "PA_G1_RF", "GND", 3.3, rf=True)
C("C603", "51pF", "PA_G1_RF", "PA_G1", 3.3, rf=True)
part("Q601", "2-stage Si RF MOSFET", "RD01MUS3", "RD01MUS3-T614", "Mitsubishi Electric", "24-pin 3x3 mm (RD01MUS3 outline)",
     "VDSS 25 V, ID2 0.9 A, PD2 8.3 W (Tc 25 C), Tch 150 C",
     [(19, "DRAIN1", P, "PA_D1"), (24, "GATE1", P, "PA_G1"), (4, "DRAIN2a", P, "PA_D2"), (5, "DRAIN2b", P, "PA_D2"),
      (6, "DRAIN2c", P, "PA_D2"), (13, "GATE2a", P, "PA_G2"), (14, "GATE2b", P, "PA_G2"), (15, "GATE2c", P, "PA_G2"),
      (25, "SOURCE_PAD", P, "GND")] + [(n, f"NC{n}", NCt, None) for n in (1, 2, 3, 7, 8, 9, 10, 11, 12, 16, 17, 18,
                                                                         20, 21, 22, 23)],
     "; ".join(ST[k]["stress"] + " (" + k + ")" for k in ("RD01MUS3_VDS", "RD01MUS3_ID2", "RD01MUS3_PD2", "RD01MUS3_Tj")
               if k in ST),
     "Datasheet Jan 2026: 175 MHz 1.0 W / 60 % at 7.2 V, 0.60 W / 58 % at 3.6 V, Pin 1 mW. Run at 5.0 V with a "
     "7.5 ohm load line; power set by the APC loop on the gate bias. Source = exposed pad (thermal vias to the ground "
     "planes). Lead finish to confirm (re-tin if tin)", "datasheet read (Mitsubishi, publication Jan 2026)")
R("R604", "4.70k", "PA_VGG", "PA_G1_B", 0.1, notes="gate-1 bias feed (EVB R2)")
C("C604", "1nF", "PA_G1_B", "GND", 3.0, kind="C0G")
R("R605", "270", "PA_G1_B", "PA_G1", 1.0, notes="gate-1 series (EVB R3)")
L("L602", "68nH", "+5V_PA", "PA_D1", family="1812SMS", i_work=0.03, notes="drain-1 feed (EVB L4)")
C("C605", "1nF", "+5V_PA", "GND", 5.15, kind="C0G", notes="drain-1 decoupling")
C("C606", "100pF", "PA_D1", "PA_IS", 15.5, rf=True, notes="interstage DC block")
L("L603", "39nH", "PA_IS", "PA_G2", notes="interstage match (EVB L5)")
C("C607", "30pF", "PA_G2", "GND", 15.5, rf=True)
R("R606", "4.70k", "PA_VGG", "PA_G2_B", 0.1, notes="gate-2 bias feed (EVB R5)")
C("C608", "1nF", "PA_G2_B", "GND", 3.0, kind="C0G")
R("R607", "560", "PA_G2_B", "PA_G2", 1.0, notes="gate-2 series (EVB R6)")
L("L604", "56nH", "+5V_PA", "PA_D2", family="1812SMS", i_work=0.42, notes="drain-2 feed (EVB L6 56 nH)")
C("C609", "1nF", "+5V_PA", "GND", 5.15, kind="C0G", notes="drain-2 RF decoupling")
C("C610", "22uF", "+5V_PA", "GND", 5.15, pkg="1210", notes="drain-2 bulk at the PA")
lpf = CALC.get("lpf", {})
me = lpf.get("match_elements", {"La_nH": 10.0, "Ca_pF": 68.0, "Lb_nH": 27.0, "Cb_pF": 27.0})
le = lpf.get("lpf_elements", {"C1": 20, "L2": 56, "C2t": 2.4, "C3": 27, "L4": 27, "C4t": 11, "C5": 27, "L6": 56,
                              "C6t": 2.4, "C7": 24})
L("L605", f"{nv(me['La_nH'])}nH", "PA_D2", "PM1", family="0806SQ", i_work=0.55,
  notes="output match section 1 (7.5 -> 19.4 ohm); value set by load-pull on the EM")
C("C611", f"{nv(me['Ca_pF'])}pF", "PM1", "GND", 15.5, rf=True)
L("L606", f"{nv(me['Lb_nH'])}nH", "PM1", "PM2", family="1812SMS", i_work=0.35, notes="output match section 2 (19.4 -> 50)")
C("C612", f"{nv(me['Cb_pF'])}pF", "PM2", "GND", 15.5, rf=True)
C("C613", "1nF", "PM2", "LPF_IN", 15.5, rf=True, pkg="0805", notes="DC block (drain at 5 V)")
C("C614", f"{nv(le['C1'])}pF", "LPF_IN", "GND", 24.0, rf=True)
L("L607", f"{nv(le['L2'])}nH", "LPF_IN", "LPF_N2", family="1812SMS", i_work=0.35, notes="LPF series arm 1")
C("C615", f"{nv(le['C2t'])}pF", "LPF_IN", "LPF_N2", 24.0, rf=True, notes="trap at 3f (437.7 MHz)")
C("C616", f"{nv(le['C3'])}pF", "LPF_N2", "GND", 24.0, rf=True)
L("L608", f"{nv(le['L4'])}nH", "LPF_N2", "LPF_N4", family="1812SMS", i_work=0.35, notes="LPF series arm 2")
C("C617", f"{nv(le['C4t'])}pF", "LPF_N2", "LPF_N4", 24.0, rf=True, notes="trap at 2f (291.8 MHz)")
C("C618", f"{nv(le['C5'])}pF", "LPF_N4", "GND", 24.0, rf=True)
L("L609", f"{nv(le['L6'])}nH", "LPF_N4", "LPF_OUT", family="1812SMS", i_work=0.35, notes="LPF series arm 3")
C("C619", f"{nv(le['C6t'])}pF", "LPF_N4", "LPF_OUT", 24.0, rf=True, notes="trap at 3f (437.7 MHz)")
C("C620", f"{nv(le['C7'])}pF", "LPF_OUT", "GND", 24.0, rf=True)
part("CPL601", "Printed sense-line coupler (PCB structure)", "45 mm edge-coupled line", "PCB feature (no part)", "-",
     "PCB: 45 x 0.37 mm line, 0.15 mm gap, L1/L2 prepreg", "-",
     [(1, "MAIN_IN", P, "LPF_OUT"), (2, "MAIN_OUT", P, "VHF_SE"), (3, "CPL_FWD", P, "CPL_FWD"),
      (4, "CPL_REV", P, "CPL_REV"), (5, "GND", P, "GND")], "1 W: no rating issue (copper)",
     "Coupling -25 dB (estimate), directivity >= 15 dB (to measure); after the LPF so that harmonics do not reach the "
     "detectors; no ferrite in the high-power path (ANT-8)", "n/a")
L("L610", "68nH", "VHF_SE", "VHF_A", family="1812SMS", i_work=0.2, notes="2 m lattice balun, 50 -> 72 ohm differential")
C("C621", "18pF", "VHF_A", "GND", 24.0, rf=True)
C("C622", "18pF", "VHF_SE", "VHF_B", 24.0, rf=True)
L("L611", "68nH", "VHF_B", "GND", family="1812SMS", i_work=0.2)
R("R608", "0", "VHF_SE", "VHF_A", 0.1, notes="DNP: fit, and omit L610/C621/C622/L611, if the AntS supplies its own "
                                             "phasing", dnp=True)
R("R609", "100k", "VHF_A", "GND", 15.0, pkg="0805", notes="static bleed (15 V peak at 1 W: 2.3 mW)")
R("R610", "100k", "VHF_B", "GND", 15.0, pkg="0805", notes="static bleed")

# ================================================================================ detectors, APC, TX_TEMP (700)
for ref0, port, out, pwd in (("70", "CPL_FWD", "DET_FWD", "DET_PWDN"), ("71", "CPL_REV", "DET_REV", "DET_PWDN")):
    R(f"R{ref0}1", "910", port, "GND", 0.4, notes="pi pad 1 dB nominal, select at test 0-4 dB; terminates the line")
    R(f"R{ref0}2", "5.6", port, f"{out}_PI", 0.05)
    R(f"R{ref0}3", "910", f"{out}_PI", "GND", 0.4)
    R(f"R{ref0}4", "68", f"{out}_PI", "GND", 0.4, notes="with the 225 ohm detector input: 52 ohm")
    C(f"C{ref0}1", "100pF", f"{out}_PI", f"{out}_IN", 1.0, rf=True)
    part(f"U{ref0}1", "TruPwr RMS detector", "AD8361", "AD8361ARMZ", "Analog Devices", "MSOP-8", "2.7-5.5 V, -40..+85 C",
         [(1, "VPOS", PI, "+3V3_PA"), (2, "IREF", P, "GND"), (3, "RFIN", I, f"{out}_IN"), (4, "PWDN", I, pwd),
          (5, "COMM", PI, "GND"), (6, "SREF", P, "GND"), (7, "VRMS", O, f"{out}_V"), (8, "FLTR", P, f"{out}_FLT")],
         "input +4.7 dBm max at 1 W (0.38 Vrms)",
         "Ground-reference mode (IREF, SREF strapping and pinout per the ADI datasheet: site not reachable in this "
         "session, to confirm); 7.5 V/Vrms: 2.5 V at 1 W. Alternate: TI LMH2121 (DSBGA, verified Active)",
         "not checked (analog.com unreachable 2026-10-27)")
    C(f"C{ref0}2", "100nF", f"{out}_FLT", "GND", 3.35, notes="detector filter")
    C(f"C{ref0}3", "100nF", "+3V3_PA", "GND", 3.35)
R("R705", "1.00k", "DET_FWD_V", "TX_FWD", 0.1, notes="RC to the OBC ADC (PC0)")
C("C705", "100nF", "TX_FWD", "GND", 3.3)
R("R715", "1.00k", "DET_REV_V", "TX_REV", 0.1, notes="RC to the OBC ADC (PC1)")
C("C715", "100nF", "TX_REV", "GND", 3.3)
part("U720", "Temperature sensor 10 mV/C", "TMP235A2", "TMP235A2DBZR", "Texas Instruments", "SOT-23-3",
     "2.3-5.5 V, -40..+150 C", [(1, "VDD", PI, "+3V1_DIG"), (2, "VOUT", O, "TMP_V"), (3, "GND", PI, "GND")],
     "14.5 uA", "On the PA ground pad, within 5 mm of Q601: TX_TEMP 10 mV/C, 500 mV at 0 C (the TMP36 scaling the "
     "firmware assumes). Always powered (V_DIG), so the 70 C hold works with the PA off. Finish NiPdAu|SN",
     "TI orderable addendum read: Active")
C("C720", "100nF", "+3V1_DIG", "GND", 3.15)
R("R720", "1.00k", "TMP_V", "TX_TEMP", 0.1, notes="RC to the OBC ADC (PA0)")
C("C721", "100nF", "TX_TEMP", "GND", 3.15)
# APC: reference, power select, PA enable gating, integrator
part("U730", "Voltage reference 2.5 V", "REF35250Q", "REF35250QDBVR", "Texas Instruments", "SOT-23-6",
     "1.8-5.5 V, +-0.05 %, 12 ppm/K, AEC-Q100", [(1, "GND1", PI, "GND"), (2, "GND2", PI, "GND"), (3, "EN", I, "+5V_PA"),
                                                (4, "VIN", PI, "+5V_PA"), (5, "NR", P, "REF_NR"), (6, "VREF", PO, "APC_REF25")],
     "2.6 uA", "1 W reference = 2.500 V (TX_FWD at 1 W). Finish NiPdAuAg", "TI orderable addendum read: Active")
C("C730", "100nF", "REF_NR", "GND", 2.5)
C("C731", "1uF", "APC_REF25", "GND", 2.5)
R("R730", "10.0k", "APC_REF25", "APC_REF", 2.5, thin=True)
R("R731", "24.1k", "APC_REF", "APC_LO_SW", 1.8, thin=True, notes="0.5 W reference 2.5 x 24.1/34.1 = 1.767 V")
part("U731", "SPST switch (power select)", "TMUX1101", "TMUX1101DBVR", "Texas Instruments", "SOT-23-5",
     "1.08-5.5 V", [(1, "D", P, "APC_LO_SW"), (2, "S", P, "GND"), (3, "GND", PI, "GND"), (4, "SEL", I, "PWR_LO_PA"),
                    (5, "VDD", PI, "+3V3_PA")], "-", "Closed (0.5 W) when PWR_LO", "TI orderable addendum read: Active")
R("R732", "10.0k", "PWR_LO", "PWR_LO_PA", 3.15, notes="domain crossing (limits current into an unpowered switch)")
part("U732", "SPST switch (PA off)", "TMUX1101", "TMUX1101DBVR", "Texas Instruments", "SOT-23-5", "1.08-5.5 V",
     [(1, "D", P, "APC_REF"), (2, "S", P, "GND"), (3, "GND", PI, "GND"), (4, "SEL", I, "PA_OFF"), (5, "VDD", PI, "+3V3_PA")],
     "-", "Grounds the APC reference unless PA_EN: the integrator drives the gates to 0 V (PA off, ~1 ms ramp)",
     "TI orderable addendum read: Active")
part("U733", "2-input AND (PA enable)", "SN74LVC1G08", "SN74LVC1G08DBVR", "Texas Instruments", "SOT-23-5",
     "5.5 V, Ioff", [(1, "A", I, "PTT_EXC"), (2, "B", I, "LMX_LD"), (3, "GND", PI, "GND"), (4, "Y", O, "PA_EN"),
                     (5, "VCC", PI, "+3V3_PA")], "-",
     "PA_EN = PTT_EXCITER . lock detect: no emission until the synthesizer is locked", "TI orderable addendum read: Active")
part("U734", "Dual inverter (PA domain)", "SN74LVC2G04", "SN74LVC2G04DBVR", "Texas Instruments", "SOT-23-6", "5.5 V",
     [(1, "1A", I, "PA_EN"), (2, "GND", PI, "GND"), (3, "2A", I, "PTT_EXC"), (4, "2Y", O, "DET_PWDN"),
      (5, "VCC", PI, "+3V3_PA"), (6, "1Y", O, "PA_OFF")], "-", "PA_OFF = NOT PA_EN; detectors powered down unless "
     "PTT_EXCITER", "TI orderable addendum read: Active")
C("C732", "100nF", "+3V3_PA", "GND", 3.35, notes="U731-U734 decoupling")
part("U735", "Op-amp RRIO (APC integrator)", "TLV9001", "TLV9001IDBVR", "Texas Instruments", "SOT-23-5",
     "1.8-5.5 V, -40..125 C", [(1, "OUT", O, "APC_OUT"), (2, "VM", PI, "GND"), (3, "INP", I, "APC_REF"),
                               (4, "INN", I, "APC_SUM"), (5, "VP", PI, "+5V_PA")], "-",
     "Vgg = integral of (Vref - TX_FWD): holds 27.0/30.0 dBm at the coupler within the detector's +-0.25 dB. Finish SN",
     "TI orderable addendum read: Active")
R("R733", "10.0k", "TX_FWD", "APC_SUM", 3.3)
C("C733", "47nF", "APC_SUM", "APC_OUT", 5.15, kind="C0G", pkg="0805", notes="integrator: 0.47 ms")
R("R734", "100", "APC_OUT", "PA_VGG", 0.1, notes="series to the gate-bias node (gate current ~0)")
C("C734", "100nF", "PA_VGG", "GND", 3.0)
C("C735", "100nF", "+5V_PA", "GND", 5.15, notes="U735 decoupling")

# ============================================================================================ test points
for tp, net in (("TP1", "+3V3_RAD"), ("TP2", "+3V0_RX"), ("TP3", "+3V1_DIG"), ("TP4", "+5V_PA"), ("TP5", "+3V3_PA"),
                ("TP6", "+3V3_LMX"), ("TP7", "EN_OK"), ("TP8", "TONE_OK"), ("TP9", "NOT_TOT"), ("TP10", "XPDR_KEY"),
                ("TP11", "PTT_EXC"), ("TP12", "TX_PERMIT_HW"), ("TP13", "RPT_DISC"), ("TP14", "TX_AUDIO_DIG"),
                ("TP15", "LMX_LD"), ("TP16", "PA_VGG"), ("TP17", "APC_REF"), ("TP18", "GND"), ("TP19", "TP_DBG_TX"),
                ("TP20", "TP_DBG_RX"), ("TP21", "TP_MCU_PA0"), ("TP22", "OSC_IN")):
    part(tp, "Test pad", net, "pad 1.0 mm (no part)", "-", "TP_1.0mm", "-", [(1, "TP", P, net)], "-",
         "test pad", "n/a")

# ====================================================================================== mechanical (no pins)
SHIELD_MAT = ("nickel-silver CuNi18Zn20 (EN CW409J, UNS C75200), 0.20 mm, unplated: non-magnetic (relative "
              "permeability about 1.0); not tin-plated steel, no nickel plating, no tin (EEE_PARTS_LIST §2A.3)")
MECH += [
    dict(ref="SH1", qty=1, value="shield frame 84 x 54 x 7 mm, three cavities, notches: (+X, -Y) corner (x > +30.0, y < -26.0) for J1, -X edge "
                                    "(x < -30.0, y -28.0 ... -16.0) for J3/J4",
         mpn="made to drawing (RF board layout)", mfr="shield vendor (open)", pkg="SMT frame, solder tabs every 5 mm",
         notes=SHIELD_MAT + ". Wall and lid reach 13 mm from the magnetometer (§10.1)"),
    dict(ref="SH2 SH3 SH4", qty=3, value="shield lids (receive, transmit, digital cavity)",
         mpn="made to drawing (RF board layout)", mfr="shield vendor (open)", pkg="solder-mounted lid",
         notes=SHIELD_MAT + ". Soldered to the frame: no shield clips, no steel spring fingers. EM only: lids "
               "held by the frame's own nickel-silver dimples for FL1 and match tuning"),
]


# ============================================================================================ outputs
def write_bom():
    groups: "OrderedDict[tuple, list]" = OrderedDict()
    for b in BOM:
        if b["mpn"].startswith("pad 1.0 mm") or b["mpn"].startswith("PCB feature"):
            key = (b["kind"], "-", b["mpn"], b["dnp"])
        else:
            key = (b["value"], b["mpn"], b["pkg"], b["dnp"])
        groups.setdefault(key, []).append(b)
    with open(HERE / "rf_board_bom.csv", "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["Reference", "Quantity", "Value", "MPN", "Manufacturer", "Package", "Rating", "Derated stress",
                    "Notes", "Verification"])
        for key, items in groups.items():
            b = items[0]
            refs = " ".join(x["ref"] for x in items)
            qty = 0 if b["dnp"] else len(items)
            notes = b["notes"] + ("; DNP" if b["dnp"] else "")
            w.writerow([refs, qty, b["value"], b["mpn"], b["mfr"], b["pkg"], b["rating"], b["stress"], notes, b["status"]])
        for m in MECH:
            w.writerow([m["ref"], m["qty"], m["value"], m["mpn"], m["mfr"], m["pkg"], "-", "-", m["notes"],
                        "mechanical item, drawing open"])
    return len(BOM), sum(1 for b in BOM if not b["dnp"] and b["mpn"] not in ("pad 1.0 mm (no part)",)
                         and not b["mpn"].startswith("PCB feature") and not b["mpn"].startswith("footprint only"))


WAIVERS += [
    "Rails (+3V3_RAD, +3V0_RX, +3V1_DIG, +3V1_AUD, +5V_PA, +5V_PA_A, +3V3_PA, +3V3_LMX, GND) are marked as driven "
    "(drive = POWER): they are fed from the connector J2 through ferrite beads, LDOs or the TPS22918, which the "
    "pin-type check cannot follow.",
    "Pins marked NC are intentionally unconnected: unused MCU GPIO (configured analog/pull-down in firmware), CC1200 "
    "GPIO3/XOSC_Q2/NC16, LMX2571 RFoutRx/Fin/FLout1-2/CPoutExt/NC, CMX138A TXENA/SYSCLK/MOD/DISCN2/AUXDAC/RXENA, "
    "HC4060 unused Q outputs, RD01MUS3 NC pins, TCXO pin 1, LVC1G17/LVC1GU04/LVC1G04 NC, LVC2G04 2Y (U407), SWO pad.",
    "RX_CD and RX_FAULT (J1-15, J1-16) are driven only while RADIO_3V3 is on; with the board unpowered the LVC "
    "outputs (Ioff) sit at 0 V and the 100 k pull-downs (and the OBC's) give 'no carrier' and 'not fault'.",
    "SPI_MISO has two tri-state drivers (CC1200 SO, CMX138A RDATA): selected by their chip selects; CC1200 GPIO1 is "
    "set high-impedance in IOCFG1 so it does not drive when CSn is high.",
    "TNC_AF is both the op-amp output and its inverting input (unity-gain follower): intended.",
    "PTT_EXC, PA_EN, LMX_LD and PWR_LO cross from the always-on domain into the PA domain: the PA-domain inputs are LVC "
    "(Ioff) or TMUX (series 10 k), and PTT_EXC is low whenever the PA domain is unpowered (it contains TX_PERMIT_HW).",
    "Passive-only RF nets (filters, matches, balun, coupler) carry no driver by design.",
]


def main():
    erc_file = HERE / "design.erc"
    if erc_file.exists():
        erc_file.unlink()
    import logging
    import skidl.logger as sklog
    buf = io.StringIO()
    handler = logging.StreamHandler(buf)
    handler.setFormatter(logging.Formatter("ERC %(levelname)s: %(message)s"))
    sklog.erc_logger.addHandler(handler)
    with redirect_stdout(buf), redirect_stderr(buf):
        ERC()
    sklog.erc_logger.removeHandler(handler)
    erc_text = buf.getvalue() + (erc_file.read_text() if erc_file.exists() else "")
    buf2 = io.StringIO()
    with redirect_stdout(buf2), redirect_stderr(buf2):
        generate_netlist(file_=str(HERE / "rf_board.net"))
    n_parts, n_fitted = write_bom()
    errors = [ln for ln in erc_text.splitlines() if "ERC ERROR" in ln]
    warnings = [ln for ln in erc_text.splitlines() if "ERC WARNING" in ln]
    with open(HERE / "rf_board_erc.log", "w") as f:
        f.write("SMART-QSO RF board (SQSO-RF-BRD-001) - SKiDL %s electrical rule check\n" % skidl.__version__)
        f.write("Parts: %d placed (%d fitted, orderable), nets: %d\n\n" % (n_parts, n_fitted, len(NETS)))
        f.write("".join(ln + "\n" for ln in erc_text.splitlines() if ln.startswith("ERC")))
        f.write("\nResult: %d error(s), %d warning(s)\n" % (len(errors), len(warnings)))
        f.write("\nIntentional waivers:\n")
        for k_, w_ in enumerate(WAIVERS, 1):
            f.write(f"  W{k_}. {w_}\n")
    # SKiDL leaves design.erc / design.log next to the script; keep only our log
    for junk in ("design.erc", "design.log", "design_sklib.py"):
        try:
            (HERE / junk).unlink()
        except FileNotFoundError:
            pass
    print(f"parts {n_parts} (fitted {n_fitted}), mechanical items {sum(m['qty'] for m in MECH)}, nets {len(NETS)}, "
          f"ERC errors {len(errors)}, warnings {len(warnings)}")
    for ln in errors + warnings:
        print(ln)
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
