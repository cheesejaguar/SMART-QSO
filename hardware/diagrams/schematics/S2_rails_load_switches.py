"""S2 EPS rails and load switches: 3V3_AO, 5V_PA and V_BURN bucks from the 4S bus, SW1-SW10, the Jetson eFuse and the
hardware load shed.

Updated 2026-10-27 to the build-to design SQSO-EPS-BRD-001 (hardware/electrical/eps/EPS_BOARD_DESIGN.md, netlist
hardware/electrical/eps/design.py; RID EPS-15, EMC-02 rule E3, AV-08): parts selected, references are the netlist's.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S2_rails_load_switches")
sch.title(d, (-3, 4.0), "S2", "EPS rails and load switches (SQSO-EPS-BRD-001)",
          "Sources: hardware/electrical/eps/design.py; EPS_BOARD_DESIGN.md 8-9; software/rust/firmware/src/gpio.rs (enables)")


def buck(x, y, name, label):
    """VBAT in, regulated rail out, with input and output capacitors, the inductor and the feedback."""
    u = sch.ic(label, left=["VIN"], right=["SW"], bottom=["EN", "GND", "FB"])
    d.add(u.at((x, y)).anchor("VIN"))
    sch.flag(d, u.VIN, "VBAT", "left", 2.2)
    d.add(elm.Capacitor().at((u.VIN[0] - 1.2, u.VIN[1])).down().length(1.4)
          .label("4.7 uF 50 V\nx1 + 0.1 uF", loc="bottom", fontsize=7))
    d.add(elm.Ground())
    d.add(elm.Dot().at((u.VIN[0] - 1.2, u.VIN[1])))
    d.add(elm.Line().at(u.EN).down().length(0.8))
    d.add(elm.Label().label("EN/UVLO divider; SYNC_A 1.000 MHz", loc="bottom", fontsize=7))
    d.add(elm.Ground().at(u.GND))
    d.add(elm.Inductor2().at(u.SW).right().label("6.8 uH XGL4030", loc="top", fontsize=7))
    d.add(out := elm.Dot())
    d.add(elm.Line().at(out.center).right().length(1.6))
    d.add(co := elm.Dot())
    d.add(elm.Capacitor().at(co.center).down().length(1.4).label("22 uF x2 16 V", loc="bottom", fontsize=7))
    d.add(elm.Ground())
    d.add(elm.Wire("|-").at(u.FB).to(co.center).linestyle(":"))
    sch.flag(d, co.center, name, "right", 0.8)
    return u


buck(0, 0, "3V3_AO", "U15 LMR43620MC3 (3.3 V)\n2 A, always on\nstart 10.5 V, stop 7.5 V")
buck(15, 0, "5V_PA", "U16 LMR43620MC5 (5.0 V)\n2 A, the PA's own buck (E3)\nstart 12.1 V, stop 8.7 V")
buck(30, 0, "V_BURN 7.0 V", "U17 LMR43620MC5 (7.04 V adj.)\nfrom VBAT_DEP (SW6)\nburn drivers only (S5)")
sch.note(d, (-3, -3.2),
         "All converters synchronised to 1.000 MHz from Y1 (4 MHz XO) / 4 (U62, U63), no spread spectrum (rule E1); "
         "the Jetson carrier takes VBAT through U19 (rule E3).", fontsize=7)

ROWS = [
    # (ref, name, input, enable net (pin), output net, limit / note)
    ("SW1 U20 + U19", "PAYLOAD", "VBAT", "PAYLOAD_EN (PB4) AND SHED_N", "VBAT_JET",
     "TPS1H100B 2.0 A; eFuse TPS16630 1.20 A latch-off on JETSON_PWR (PA2)"),
    ("SW2 U22", "RADIO", "3V3_AO", "RADIO_EN (PB5)", "3V3_RADIO", "TPS2553 0.30 A, retry 1.3 s"),
    ("SW3 U23", "PA", "5V_PA", "RF_TX_EN (PA3) AND SHED_N", "5V_PA_SW", "TPS2553 0.99 A; hardware TX-OFF"),
    ("SW4 U24", "ADCS", "3V3_AO", "ADCS_EN (PB6)", "3V3_ADCS", "TPS2553 0.30 A, retry 0.28 s"),
    ("SW5 U21", "HEATER", "VBAT", "HEATER_EN (PD2)", "HTR_P", "TPS1H000 0.148 A + F4 0.25 A + Q22 40 C cutout"),
    ("SW6 U18", "DEPLOY", "VBAT", "DEPLOY_EN (PA1)", "VBAT_DEP", "TPS1H100B 2.0 A; feeds U17 (S5)"),
    ("SW7 U25", "GNSS", "3V3_AO", "GNSS_EN (PE5)", "3V3_GNSS", "TPS2553 0.20 A, retry 1.3 s"),
    ("SW8 U26", "AntS", "3V3_AO", "DEPLOY_EN (PA1)", "3V3_ANTS", "TPS2553 1.37 A; AntS I2C via U80"),
    ("SW9 U27", "EPS CTRL", "3V3_AO", "always", "3V3_MCU", "TPS2553 75 mA, retry 0.28 s"),
    ("SW10 U38", "OBC", "3V3_AO", "always", "3V3_OBC", "TPS2553 0.15 A, retry 0.28 s (AV-08)"),
]
y0, pitch = -6.0, -3.0
for i, (ref, name, vin, en, vout, lim) in enumerate(ROWS):
    y = y0 + i * pitch
    u = sch.ic(f"{ref} {name}\n{lim}", left=["EN", "VIN"], right=["VOUT"], bottom=["GND", "FLT"], size=(7.4, 1.8))
    d.add(u.at((6, y)).anchor("VIN"))
    sch.flag(d, u.VIN, vin, "left", 1.6)
    d.add(elm.Line().at(u.EN).left().length(2.4))
    d.add(n := elm.Dot())
    d.add(elm.Resistor().at(n.center).down().length(1.2).label("100k", loc="bottom", fontsize=6))
    d.add(elm.Ground())
    sch.flag(d, n.center, en, "left", 0.6)
    d.add(elm.Ground().at(u.GND))
    d.add(elm.Line().at(u.VOUT).right().length(1.2))
    d.add(o := elm.Dot())
    d.add(elm.Capacitor().at(o.center).down().length(1.0).label("1 uF", loc="bottom", fontsize=6))
    d.add(elm.Ground())
    sch.flag(d, o.center, vout, "right", 0.8)
    d.add(elm.Line().at(u.FLT).down().length(0.5))
    sch.note(d, (u.FLT[0] + 0.2, u.FLT[1] - 0.2), "fault -> EPS controller U37 (register LOAD_SHED/FAULT)", fontsize=6)

sch.note(d, (21, y0 + 0.8),
         "Every enable is low at reset and held low by its pull-down.\n"
         "SW3's pull-down is mandatory: hardware TX-OFF (BASELINE 7).\n"
         "TPS2553 rails: current limit in 2 us, FAULT after 8 ms pulls EN\n"
         "low (R_FAULT 100 k, C_RETRY), so a latch-up power-cycles the rail.\n"
         "Load shed (U48): VBAT < 13.6 V for 5 s -> SHED_N low -> SW1 and\n"
         "SW3 off; released above 14.4 V; independent of the OBC.\n"
         "Rail currents: INA3221 U32-U35 on the EPS controller's bus.", fontsize=7)

d.save(sch.out_path("S2_rails_load_switches"))
