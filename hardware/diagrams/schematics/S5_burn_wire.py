"""S5 deployment drivers: four current-limited burn drivers (two resistors per wing), each with a 12 s hardware one-shot,
the wing hinge sense, and the ISIS AntS supply and I2C (the AntS has its own thermal knives and controller).

Updated 2026-10-27 to the build-to design SQSO-EPS-BRD-001 (hardware/electrical/eps/EPS_BOARD_DESIGN.md, netlist
hardware/electrical/eps/design.py; RIDs SMA-05, REL-03, EPS-07, EPS-18): references are the netlist's.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S5_burn_wire")
sch.title(d, (-4, 5.5), "S5", "Deployment drivers and deployed-state sense (SQSO-EPS-BRD-001)",
          "Sources: hardware/electrical/eps/design.py; EPS_BOARD_DESIGN.md 10; docs/BASELINE_PARAMETERS.md 4; W7")

DRV = [("W1A", "BURN_W1A (PB10)", "J10 1-2", "U70", "Q24"), ("W1B", "BURN_W1B (PE6)", "J10 3-4", "U71", "Q25"),
       ("W2A", "BURN_W2A (PB9, was ANTENNA_BURN)", "J11 1-2", "U72", "Q26"),
       ("W2B", "BURN_W2B (PE7)", "J11 3-4", "U73", "Q27")]

for k, (w, line, pins, tpl, q) in enumerate(DRV):
    y = -k * 7.0
    sch.flag(d, (0, y + 2.5), "V_BURN 7.0 V (S2 U17)", "left", 0.01)
    d += elm.Line().at((0, y + 2.5)).right().length(3.0)
    d += elm.Resistor().down().length(2.2).label(f"R_{w} burn\n~5 Ohm\n{pins}", loc="bottom", fontsize=7)
    d += (q_ := elm.NFet().down().anchor("drain").label(f"{q}\nBUK9M14-40E", loc="left", fontsize=7))
    d += elm.Resistor().at(q_.source).down().length(1.6).label("0.25 Ohm", loc="bottom", fontsize=7)
    d += elm.Ground()
    d += elm.Line().at(q_.gate).right().length(1.0)
    amp = sch.ic(f"OPA2991 (U78/U79)\nI = 0.200 V / 0.25 Ohm\n= 0.80 A +/-2.2 %", left=["OUT"], right=["INP"], size=(5.0, 1.6))
    d += amp.at((q_.gate[0] + 1.0, q_.gate[1])).anchor("OUT")
    gate = sch.ic(f"{tpl} TPL5111 one-shot 12.0 s\npowered by the burn line\n+ AND (VCC = REF_A 2.500 V)",
                  left=["SET"], right=["LINE"], size=(6.2, 1.6))
    d += gate.at((amp.INP[0] + 1.2, amp.INP[1])).anchor("SET")
    d += elm.Line().at(amp.INP).to(gate.SET)
    sch.flag(d, gate.LINE, line, "right", 0.6)

sch.note(d, (24, 2.6),
         "One current-limited driver per burn resistor, two resistors per wing (REL-03, chair decision).\n"
         "Chain for each resistor: DS1, DS2, DS3 (bus, S1) -> SW6 (DEPLOY_EN, high side) -> U17 burn buck\n"
         "-> resistor -> driver FET (burn line AND one-shot). The FSW burns one resistor at a time for 8 s,\n"
         "the second of a wing only if it has not released.\n"
         "Pack current: one resistor 0.67 A (housekeeping and heater included); four at once (software\n"
         "fault) are held by U17's current limit to 2.0 A, below the 4.0 A BMS trip and the 5 A fuse.\n"
         "One-shot: a stuck-high line burns 12 s once; only a falling edge re-arms it.\n"
         "Regulates for burn resistors up to 6.9 Ohm.", fontsize=7)

# hinge sense: both wings' switches in series (J10 5-6, J11 5-6)
ys = -30.0
d += elm.Vdd().at((2, ys + 2.0)).label("3V3_AO", fontsize=7)
d += elm.Resistor().at((2, ys + 2.0)).down().length(1.6).label("10k", loc="top", fontsize=7)
d += (n := elm.Dot())
d += elm.Switch(action="close").at(n.center).down().length(1.6).label("S W1 (J10 5-6)", loc="top", fontsize=7)
d += elm.Switch(action="close").down().length(1.6).label("S W2 (J11 5-6)", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Resistor().at(n.center).right().length(1.8).label("1k", loc="top", fontsize=7)
d += (m := elm.Dot())
d += elm.Capacitor().at(m.center).down().length(1.2).label("100 nF", loc="right", fontsize=7)
d += elm.Ground()
sch.flag(d, m.center, "SOLAR_SENSE (PB12)", "right", 0.8)
sch.note(d, (9, ys + 1.0), "Switches closed when deployed: low = both wings deployed.\n"
         "AntS (J12): 3V3_ANTS from SW8 (DEPLOY_EN, 1.37 A limit); I2C2 through U80 TCA9517\n"
         "(isolated while the AntS is unpowered). The AntS's own ARM and knives complete its chain\n"
         "(DS1, DS2, DS3, SW8, AntS ARM). ANTENNA_SENSE (PB11) is no longer wired.", fontsize=7)

d.save(sch.out_path("S5_burn_wire"))
