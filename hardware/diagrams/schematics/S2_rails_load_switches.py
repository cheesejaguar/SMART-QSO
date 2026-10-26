"""S2 EPS rails and load switches: 3V3_AO and 5 V regulators from the 4S bus, SW1-SW7."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S2_rails_load_switches")
sch.title(d, (-3, 4.0), "S2", "EPS rails and load switches",
          "Sources: hardware/eps/PDU_DESIGN.md (limits); software/rust/firmware/src/gpio.rs (enables, TBR); "
          "docs/BASELINE_PARAMETERS.md 5")


def buck(x, y, name, label):
    """VBAT in, regulated rail out, with input and output capacitors, the inductor and the feedback divider."""
    u = sch.ic(label, left=["VIN"], right=["SW"], bottom=["EN", "GND", "FB"], tbd=True)
    d.add(u.at((x, y)).anchor("VIN"))
    sch.flag(d, u.VIN, "VBAT", "left", 2.2)
    d.add(elm.Capacitor().at((u.VIN[0] - 1.2, u.VIN[1])).down().length(1.4)
          .label("10 uF x2\n25 V (TBR)", loc="bottom", fontsize=7))
    d.add(elm.Ground())
    d.add(elm.Dot().at((u.VIN[0] - 1.2, u.VIN[1])))
    d.add(elm.Line().at(u.EN).down().length(0.8))
    d.add(elm.Label().label("UVLO divider (TBR)", loc="bottom", fontsize=7))
    d.add(elm.Ground().at(u.GND))
    d.add(elm.Inductor2().at(u.SW).right().label("L (TBR)", loc="top", fontsize=7))
    d.add(out := elm.Dot())
    d.add(elm.Resistor().at(out.center).down().length(1.6).label("Rfb1", loc="bottom", fontsize=7))
    d.add(fbm := elm.Dot())
    d.add(elm.Resistor().down().length(1.4).label("Rfb2", loc="bottom", fontsize=7))
    d.add(elm.Ground())
    d.add(elm.Wire("|-").at(u.FB).to(fbm.center).linestyle(":"))
    d.add(elm.Line().at(out.center).right().length(1.6))
    d.add(co := elm.Dot())
    d.add(elm.Capacitor().at(co.center).down().length(1.4).label("22 uF x2 (TBR)", loc="bottom", fontsize=7))
    d.add(elm.Ground())
    sch.flag(d, co.center, name, "right", 0.8)
    return u


buck(0, 0, "3V3_AO", "U1 3V3_AO buck (TBD)\n>= 0.5 A, always on\nVIN 12.0-16.8 V")
buck(15, 0, "5V", "U2 5 V buck (TBD)\n>= 4 A (Jetson 3 A + PA 0.8 A)\nVIN 12.0-16.8 V")
sch.note(d, (-3, -3.2),
         "No regulator in the hardware documents takes a 4S input (finding F6): both are TBD, with input rated >= 25 V.\n"
         "The 3V3 rail and the 3V3_AO rail are the same regulator in this drawing (TBR: a separate switched 3V3).",
         fontsize=7)

# ---- load switches, one row each
ROWS = [
    # (ref, name, input, enable net (pin), output net, limit / note)
    ("SW1", "PAYLOAD", "5V", "PAYLOAD_EN (PB4)", "5V_PAYLOAD", "2.0 A nom · 3.5 A trip; to S7"),
    ("SW2", "RADIO", "3V3_AO", "RADIO_EN (PB5)", "RADIO_3V3", "RX + TNC + exciter; 5 V TBD"),
    ("SW3", "PA", "5V", "RF_TX_EN (PA3)", "PA_5V", "0.5 A nom · 1.0 A trip; hardware TX-OFF"),
    ("SW4", "ADCS", "3V3_AO", "ADCS_EN (PB6)", "3V3_ADCS", "0.2 A nom · 0.5 A trip"),
    ("SW5", "HEATER", "VBAT", "HEATER_EN (PD2)", "HEATER", "0.3 A nom · 0.5 A trip; 0.5 W"),
    ("SW6", "DEPLOY", "VBAT", "DEPLOY_EN (PA1)", "VBAT_DEPLOY", "2.0 A peak; to S5 (arm)"),
    ("SW7", "GNSS", "3V3_AO", "GNSS_EN (PE4, proposed)", "3V3_GNSS", "latch-up trip; to S8"),
]
y0, pitch = -6.0, -3.0
for i, (ref, name, vin, en, vout, lim) in enumerate(ROWS):
    y = y0 + i * pitch
    u = sch.ic(f"{ref} {name} load switch (TBD)\n{lim}", left=["EN", "VIN"], right=["VOUT"], bottom=["GND", "FLT"],
               tbd=True, size=(6.4, 1.8))
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
    sch.note(d, (u.FLT[0] + 0.2, u.FLT[1] - 0.2), "fault out: no OBC pin (TBD)", fontsize=6)

sch.note(d, (19, y0 + 0.8),
         "Every enable is low at reset and held low by its pull-down,\n"
         "so a reset or an unprogrammed MCU leaves every load off.\n"
         "SW3's pull-down is mandatory: it is the hardware TX-OFF\n"
         "path (BASELINE 7), independent of the AI payload.\n"
         "OBC + FRAM run from 3V3_AO directly (about 50 mA).\n"
         "None of the switches' fault outputs reaches the OBC;\n"
         "the INA226 ALERT (PB7) is the only interrupt (TBR).", fontsize=7)

d.save(sch.out_path("S2_rails_load_switches"))
