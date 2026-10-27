"""S1 EPS power path and inhibits: strings, ideal diodes, four trackers, EOL fuse F1, 4S pack with the BQ76952 AFE and
the BQ771806 secondary protector, F2 / RT1 / F3, and the three deployment-switch inhibits DS1, DS2, DS3 with the RBF.

Updated 2026-10-27 to the build-to design SQSO-EPS-BRD-001 (hardware/electrical/eps/EPS_BOARD_DESIGN.md, netlist
hardware/electrical/eps/design.py): reference designators on this sheet are the netlist's.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S1_eps_power_path")
sch.title(d, (-2, 16), "S1", "EPS power path and inhibits (SQSO-EPS-BRD-001)",
          "Sources: hardware/electrical/eps/design.py (netlist, refs), EPS_BOARD_DESIGN.md 4-7; "
          "docs/BASELINE_PARAMETERS.md 4-5; CDS 2.3.7, 2.3.8")

CELL = 2.2           # one battery cell's length = the BMS tap pitch
XB, YT = 18.0, 8.8   # battery column, BATT+ height

# ---- one solar string (of 8): TVS, ideal diode, tracker
d += elm.Ground().at((0, 3.2))
d += elm.Solar().at((0, 3.2)).up().label("AZUR 3G30A", loc="bottom", fontsize=7)
d += elm.Solar().up().label("AZUR 3G30A", loc="bottom", fontsize=7)
d += elm.Line().up().toy(YT)
d += elm.Line().right().length(0.6)
d += (dbl := elm.Schottky().right().label("U90-U97 LM74700-Q1\n+ Q31-Q38 (ideal diode)\nD1-D8 SMBJ7.0A TVS", loc="top", fontsize=7))
d += (bus := elm.Dot())
sch.note(d, (-3.4, 6.6), "String n of 8 (W2):\n2 cells in series\nVoc <= 6.31 V\nIsc <= 0.55 A")
sch.note(d, (bus.center[0] - 0.6, YT - 0.5), "T1: W1F+W2F  T2: PY\nT3: W1B+W2B  T4: PX+MX+MY", fontsize=7)

mppt = sch.ic("U4-U7 TPS55340-Q1 x4\nboost MPPT, 1.000 MHz sync\nCV 16.40 V / CC 0.394 A (hw loops)",
              left=["VIN"], right=["VOUT", "EN"], bottom=["COMP", "GND"])
d += mppt.at((bus.center[0] + 1.6, YT)).anchor("VIN")
d += elm.Line().at(bus.center).to(mppt.VIN)
d += elm.Ground().at(mppt.GND)
sch.flag(d, mppt.EN, "TRK_EN (DS3 NO, RBF B)", "right", 0.5)
d += elm.Line().at(mppt.COMP).down().length(0.8)
d += elm.Label().label("CTRL_DN: CV, CC, charge inhibit", loc="bottom", fontsize=7)

# ---- EOL passivation fuse F1 (three-terminal, heater fired by Q20 + Q21 in series)
d += elm.Line().at(mppt.VOUT).right().length(0.4)
d += (f1 := elm.Fuse().right().label("F1 ITV4030L1412NR\nEOL charge disconnect", loc="bottom", fontsize=7))
d += elm.Line().right().length(0.6)
d += elm.Wire("|-").to((XB, YT + 3.0))
sch.note(d, (XB - 9.0, YT + 2.4), "F1 heater: Q20 (EOL_FIRE) in series with\nQ21 (EOL_ARM latch U61): two OBC lines", fontsize=7)

# ---- 4S pack, cell taps to the AFE and the secondary protector
taps = [(XB, YT)]
for i in range(4, 0, -1):
    d += elm.BatteryCell().at(taps[-1]).down().length(CELL).label(f"cell {i}", loc="bottom", fontsize=7)
    taps.append((XB, taps[-1][1] - CELL))
d += elm.Dot().at(taps[0]).label("BAT_TOP", loc="left", fontsize=8)
d += elm.Dot().at(taps[-1]).label("BAT_BOT", loc="left", fontsize=8)
sch.note(d, (XB - 5.2, 4.8), "BT1 4S1P INR18650-35E\n11.2-16.4 V, 3.45 Ah\nW1: J1 power/NTC1/heater,\nJ2 taps + NTC2")

bms = sch.ic("U1 BQ76952 AFE (OTP)\nCOV 4.25 V  CUV 2.80/3.00 V\nOCD 4.0 A 0.5 s  SCD 10 A\nSUV PF 2.00 V -> K1 lockout",
             left=["VC0", "VC1", "VC2", "VC15", "VC16"], right=["SDA", "CFETOFF", "DFETOFF", "FUSE"],
             bottom=["SRP", "SRN", "CHG", "DSG"], pinspacing=CELL, size=(6.5, 4 * CELL + 1.2))
d += bms.at((XB + 2.4, taps[4][1])).anchor("VC0")
for pin, xy in zip(("VC0", "VC1", "VC2", "VC15", "VC16"), reversed(taps)):
    d += elm.Dot().at(xy)
    d += elm.Line().at(xy).to(getattr(bms, pin))
sch.note(d, (XB + 0.3, YT - 0.25), "taps 100 Ohm + 0.1 uF\nU2 BQ771806 4.35 V on\nown 1 kOhm taps -> Q16 -> F3", fontsize=7)
sch.flag(d, bms.SDA, "AFE I2C -> U50 ISO1540", "right", 0.6)
sch.flag(d, bms.CFETOFF, "CHG_INHIBIT (100 k)", "right", 0.6)
sch.flag(d, bms.DFETOFF, "OT60 BOTHOFF (100 k)", "right", 0.6)
sch.flag(d, bms.FUSE, "AFE_FUSE -> K1 SET", "right", 0.6)

# ---- low side: AFE shunt, then DS2 back-to-back FETs Q6/Q7 to GND
y_ls = taps[4][1] - 3.2
d += elm.Line().at(taps[4]).down().toy(y_ls)
d += elm.Line().right().tox(bms.SRP[0])
d += elm.Resistor().right().tox(bms.SRN[0]).label("RS7 10 mOhm", loc="bottom", fontsize=7)
d += (pminus := elm.Dot().label("PACK_N", loc="bottom", fontsize=8))
d += elm.Line().at(bms.SRP).toy(y_ls).linestyle(":")
d += elm.Line().at(bms.SRN).toy(y_ls).linestyle(":")
d += elm.Line().at(pminus.center).right().length(1.0)
d += (q6 := elm.NFet(bulk=True).right().anchor("drain").label("Q6", loc="top", fontsize=7))
d += elm.Line().at(q6.source).right().length(0.8)
d += (q7 := elm.NFet(bulk=True).left().anchor("source").label("Q7", loc="top", fontsize=7))
d += elm.Line().at(q7.drain).right().length(0.6)
d += elm.Ground()
sch.note(d, (pminus.center[0] + 0.6, y_ls - 2.0),
         "INHIBIT DS2: Q6+Q7 BUK9Y4R4-40E back to back (blocks charge and discharge)\n"
         "gate from BAT_F via 10 k, DS2 NC contact (J16), RBF pole A (J14), 1.2 M / 1.0 M, 47 nF\n"
         "Q23 wakes the AFE (TS2) when DS2 is released", fontsize=7)

# ---- high side: F2, RT1, F3, gauge shunt, AFE FETs Q1/Q2, PACK_P, DS1 (Q4) and DS3 (Q5) to VBAT
y_top = YT + 3.0
d += elm.Line().at(taps[0]).up().toy(y_top)
d += elm.Line().right().length(1.0)
d += elm.Fuse().right().label("F2 5 A", loc="top", fontsize=7)
d += elm.Thermistor().right().label("RT1 PTC 2.6 A", loc="top", fontsize=7)
d += elm.Fuse().right().label("F3 ITV (OVP fuse)", loc="top", fontsize=7)
d += elm.Resistor().right().label("RS6 10 mOhm\nU28 gauge", loc="top", fontsize=7)
d += elm.Switch(action="close").right().label("Q1/Q2 CHG/DSG\nBUK7Y3R5-40H", loc="top", fontsize=7)
d += (pp := elm.Dot().label("PACK_P", loc="bottom", fontsize=8))
d += elm.Switch(action="close").right().label("INHIBIT DS1\nQ4 IPD50P04P4L-11", loc="top", fontsize=7)
d += (n1 := elm.Dot().label("VB1", loc="bottom", fontsize=7))
d += elm.Switch(action="close").right().label("INHIBIT DS3\nQ5 IPD50P04P4L-11", loc="top", fontsize=7)
d += (vb := elm.Dot())
sch.flag(d, vb.center, "VBAT (RS8, to S2)", "right", 0.7)
sch.note(d, (XB + 15.0, YT - 1.0),
         "CDS 2.3.7 / 2.3.8: three independent physical inhibits,\n"
         "each from its own switch on its own rail (J15-J17):\n"
         "  DS1 -> Q4 high side in PACK_P (pack and array)\n"
         "  DS2 -> Q6/Q7 in the pack return (pack)\n"
         "  DS3 -> Q5 high side in VB1, and TRK_EN low (pack and array)\n"
         "Each switch drives a gate through >= 23 ms RC; no switch\n"
         "carries bus current. RBF (J14): pole A opens DS2's drive,\n"
         "pole B holds TRK_EN low (not counted).\n"
         "Stowed drain 2.2 uA typ, 6.7 uA worst (AFE in SHUTDOWN).\n"
         "Umbilical J13: charge into PACK_P via F5, D16; return PACK_N.", fontsize=7)

d.save(sch.out_path("S1_eps_power_path"))
