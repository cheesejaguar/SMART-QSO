"""S1 EPS power path and inhibits: solar strings, MPPT/BCR, passivation, 4S pack with BMS, fuse, inhibits."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S1_eps_power_path")
sch.title(d, (-2, 15), "S1", "EPS power path and inhibits",
          "Sources: docs/BASELINE_PARAMETERS.md 4-5; hardware/eps/BATTERY_DESIGN.md, PDU_DESIGN.md, "
          "SOLAR_ARRAY_DESIGN.md; CDS 2.3.7, 2.3.8")

CELL = 2.2           # one battery cell's length = the BMS tap pitch
XB, YT = 16.0, 8.8   # battery column, BATT+ height

# ---- one solar string (of 8) and its blocking diode
d += elm.Ground().at((0, 3.2))
d += elm.Solar().at((0, 3.2)).up().label("AZUR 3G30A", loc="bottom", fontsize=7)
d += (sa2 := elm.Solar().up().label("AZUR 3G30A", loc="bottom", fontsize=7))
d += elm.Line().up().toy(YT)
d += elm.Line().right().length(0.6)
d += (dbl := elm.Schottky().right().label("D1-D8 blocking\none per string (TBR)", loc="top", fontsize=7))
d += (bus := elm.Dot())
sch.note(d, (-3.4, 6.6), "String n of 8:\n2 cells in series\nVmp 4.2-4.8 V\nImp 0.50 A\n(W2 SA+ / SA-)")
sch.note(d, (bus.center[0] - 0.8, YT - 0.4), "SA_BUS: the\nother 7 strings\njoin here", fontsize=7)

# ---- MPPT / battery charge regulator (not selected)
mppt = sch.ic("U1 MPPT / BCR (TBD)\nboost 4.2-4.8 V to 12.0-16.8 V\ncharge <= 0.40 A, 0-45 C only",
              left=["VIN"], right=["VOUT", "NTC"], bottom=["EN", "GND"], tbd=True)
d += mppt.at((bus.center[0] + 1.6, YT)).anchor("VIN")
d += elm.Line().at(bus.center).to(mppt.VIN)
d += elm.Ground().at(mppt.GND)
d += elm.Line().at(mppt.EN).down().length(0.8)
d += elm.Label().label("CHG_OK from U2 (TBR)", loc="bottom", fontsize=7)
sch.flag(d, mppt.NTC, "TEMP1", "right", 0.7)

# ---- EOL passivation: non-resettable disconnect of the charge path
d += elm.Line().at(mppt.VOUT).right().length(0.4)
d += elm.Fuse().right().label("F1 EOL passivation\nnon-resettable (TBD)", loc="bottom", fontsize=7).style(ls="--")
d += elm.Line().right().length(0.6)
d += elm.Wire("|-").to((XB, YT))
d += (bplus := elm.Dot().at((XB, YT)).label("BATT+", loc="left", fontsize=8))

# ---- 4S pack, cell taps to the BMS
taps = [(XB, YT)]
for i in range(4, 0, -1):
    d += elm.BatteryCell().at(taps[-1]).down().length(CELL).label(f"cell {i}", loc="bottom", fontsize=7)
    taps.append((XB, taps[-1][1] - CELL))
d += elm.Dot().at(taps[-1]).label("BATT-", loc="left", fontsize=8)
sch.note(d, (XB - 5.0, 4.8), "BT1 4S1P Li-ion 18650\n14.4 V nom, 12.0-16.8 V\nabout 3.5 Ah, 50 Wh\nW1: P1/J1 power,\nP2/J2 cell taps")

bms = sch.ic("U2 BMS BQ76920 or eq. (TBD)\nOV 4.25 V · UV 2.8 V · OC 5 A\nbalancing; HW comparator backup",
             left=["VC0", "VC1", "VC2", "VC3", "VC4"], right=["SCL", "SDA", "ALERT"],
             bottom=["SRP", "SRN", "DSG", "CHG"], pinspacing=CELL, tbd=True, size=(6.5, 4 * CELL + 1.2))
d += bms.at((XB + 2.4, taps[4][1])).anchor("VC0")
for pin, xy in zip(("VC0", "VC1", "VC2", "VC3", "VC4"), reversed(taps)):
    d += elm.Dot().at(xy)
    d += elm.Line().at(xy).to(getattr(bms, pin))
sch.note(d, (XB + 0.3, YT - 0.25), "taps: 1 kOhm +\n0.1 uF each (TBR)", fontsize=7)
sch.flag(d, bms.SCL, "I2C2_SCL", "right", 0.6)
sch.flag(d, bms.SDA, "I2C2_SDA", "right", 0.6)
sch.flag(d, bms.ALERT, "BMS_ALERT (no pin, TBD)", "right", 0.6)

# ---- low side: sense resistor and back-to-back protection FETs
y_ls = taps[4][1] - 3.2
d += elm.Line().at(taps[4]).down().toy(y_ls)
d += elm.Line().right().tox(bms.SRP[0])
d += (rs := elm.Resistor().right().tox(bms.SRN[0]).label("R1 5 mOhm (TBR)", loc="bottom", fontsize=7))
d += elm.Line().right().tox(bms.DSG[0] - 1.0)
d += (q1 := elm.NFet(bulk=True).up().anchor("drain").label("Q1 DSG", loc="right", fontsize=7))
d += (q2 := elm.NFet(bulk=True).down().at(q1.source).anchor("source").label("Q2 CHG", loc="right", fontsize=7))
d += elm.Line().at(q2.drain).right().length(0.6)
d += (pminus := elm.Dot().label("PACK-", loc="bottom", fontsize=8))
d += elm.Line().at(bms.SRP).toy(y_ls).linestyle(":")
d += elm.Line().at(bms.SRN).toy(y_ls).linestyle(":")
d += elm.Wire("|-").at(bms.DSG).to(q1.gate).linestyle(":")
d += elm.Wire("|-").at(bms.CHG).to(q2.gate).linestyle(":")
d += elm.Line().at(pminus.center).right().length(1.0)
d += (ds2 := elm.Switch(action="close").right().label("DS2 deployment\nswitch, return", loc="bottom", fontsize=7))
d += elm.Line().right().length(0.4)
d += elm.Ground()

# ---- discharge path: fuse, PTC, RBF, inhibits
y_top = YT + 3.0
d += elm.Line().at(bplus.center).up().toy(y_top)
d += elm.Line().right().length(2.0)
d += elm.Fuse().right().label("F2 5 A pack fuse", loc="top", fontsize=7)
d += elm.Thermistor().right().label("RT1 PTC", loc="top", fontsize=7)
d += elm.Switch(action="open").right().label("RBF pin\n(not an inhibit)", loc="top", fontsize=7)
d += (ds1 := elm.Switch(action="close").right().label("DS1 deployment\nswitch, VBAT+", loc="top", fontsize=7))
d += (n1 := elm.Dot())
d += elm.Switch(action="close").right().label("Inhibit 3 (TBD)\nplacement open", loc="top", fontsize=7).style(ls="--")
d += (vb := elm.Dot())
sch.flag(d, vb.center, "VBAT", "right", 0.7)
d += elm.Resistor().at(n1.center).down().length(1.6).label("R2 100k (TBR)", loc="bottom", fontsize=7)
d += (n2 := elm.Dot())
d += elm.Resistor().down().length(1.6).label("R3 20k (TBR)", loc="bottom", fontsize=7)
d += elm.Ground()
sch.flag(d, n2.center, "DEPLOY_SENSE", "right", 0.7)

sch.note(d, (XB + 14.5, YT - 1.0),
         "CDS 2.3.7 / 2.3.8: >= 3 independent physical\n"
         "inhibits; RBF and timers do not count.\n"
         "DS1 and DS2 are open while the rails are\n"
         "depressed (stowed). The third inhibit is not\n"
         "defined anywhere yet (finding F5).\n"
         "DEPLOY_SENSE (PE2) reads DS1 through R2/R3:\n"
         "16.8 V gives 2.8 V (TBR).", fontsize=7)

d.save(sch.out_path("S1_eps_power_path"))
