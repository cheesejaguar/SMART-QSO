"""S9 OBC/EPS analog front ends for ADC1, sized for the 4S pack, with the firmware scale each one implies."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S9_adc_front_ends")
sch.title(d, (-5, 4.5), "S9", "ADC front ends",
          "Sources: software/rust/firmware/src/adc.rs (FRONT_END, pins TBR); docs/BASELINE_PARAMETERS.md 5 (4S pack); "
          "hardware/eps/PDU_DESIGN.md (ranges)")


def rc_out(at, net):
    """1k + 10 nF anti-alias filter (TBR) to the ADC pin flag."""
    d.add(elm.Resistor().at(at).right().length(1.6).label("1k", loc="top", fontsize=7))
    d.add(n := elm.Dot())
    d.add(elm.Capacitor().at(n.center).down().length(1.2).label("10 nF", loc="bottom", fontsize=7))
    d.add(elm.Ground())
    sch.flag(d, n.center, net, "right", 0.6)


def divider(x, y, src, top, bot, net, note):
    sch.flag(d, (x, y), src, "left", 0.01)
    d.add(elm.Line().at((x, y)).right().length(0.6))
    d.add(elm.Resistor().down().length(1.6).label(top, loc="bottom", fontsize=7))
    d.add(n := elm.Dot())
    d.add(elm.Resistor().down().length(1.6).label(bot, loc="bottom", fontsize=7))
    d.add(elm.Ground())
    rc_out(n.center, net)
    sch.note(d, (x + 6.5, y + 0.2), note, fontsize=7)


def shunt_amp(x, y, src, rsh, amp, ref, net, note):
    sch.flag(d, (x, y), src, "left", 0.01)
    d.add(elm.Line().at((x, y)).right().length(0.6))
    d.add(s1 := elm.Dot())
    d.add(elm.Resistor().right().length(1.8).label(rsh, loc="top", fontsize=7))
    d.add(s2 := elm.Dot())
    d.add(elm.Line().right().length(0.6))
    d.add(elm.Label().label("load", loc="right", fontsize=7))
    u = sch.ic(amp, left=["IN-", "IN+"], right=["OUT"], bottom=["GND", "REF"], size=(4.4, 2.0), tbd=True,
               spread=("bottom",))
    d.add(u.at((s1.center[0] + 0.2, y - 3.2)).anchor("INN"))
    d.add(elm.Wire("|-").at(s1.center).to(sch.pin(u, "IN+")).linestyle(":"))
    d.add(elm.Line().at(s2.center).toy(y - 1.2).linestyle(":"))
    d.add(elm.Line().linestyle(":").left().tox(sch.pin(u, "IN-")[0] - 0.6))
    d.add(elm.Line().linestyle(":").toy(sch.pin(u, "IN-")[1]))
    d.add(elm.Line().linestyle(":").to(sch.pin(u, "IN-")))
    d.add(elm.Line().at(u.REF).down().length(0.5))
    d.add(elm.Label().label(ref, loc="bottom", fontsize=7))
    d.add(elm.Line().at(u.GND).down().length(0.4))
    d.add(elm.Ground())
    rc_out(u.OUT, net)
    sch.note(d, (x + 12.5, y + 0.2), note, fontsize=7)


divider(0, 0, "VBAT", "R1 100k", "R2 20k", "VBATT_SNS (PC2)",
        "1:6, 16.8 V reads 2.8 V. The firmware's FRONT_END says 1:3 (a 2S value, 8.4 V max):\n"
        "a 4S pack on a 1:3 divider would put 5.6 V on PC2. Change adc.rs to Linear(6.0, 0) with\n"
        "this divider (finding F1). Divider draws 140 uA from the pack: gate it or raise R (TBR).")
shunt_amp(0, -6, "BATT+ path", "R3 10 mOhm", "U1 INA240A2 or eq. (TBD)\ngain 50, bidirectional", "REF = 1.65 V",
          "IBATT_SNS (PC3)", "0.5 V/A, 1.65 V at 0 A: adc.rs Linear(2.0, -3.3).\n+/-3 A spans 0.15-3.15 V (PDU_DESIGN).")
divider(0, -13, "SA_BUS", "R4 30k", "R5 10k", "VSOLAR_SNS (PC4)",
        "1:4, as adc.rs. Two-cell strings, Voc about 5.4 V, read 1.35 V\n(TBR: the divider could be 1:2 for resolution).")
shunt_amp(0, -19, "SA_BUS path", "R6 20 mOhm", "U2 INA240A2 or eq. (TBD)\ngain 50, unidirectional", "REF = GND",
          "ISOLAR_SNS (PC5)", "1 V/A: adc.rs Linear(1.0, 0). 0-2 A (PDU_DESIGN).")
divider(0, -26, "5V", "R7 10k", "R8 10k", "VBUS_SNS (PB0)", "1:2, as adc.rs (5 V bus).")
tmp = sch.ic("U3 TMP36\nboard temperature", left=["+VS"], right=["VOUT"], bottom=["GND"])
d += tmp.at((2.0, -32)).anchor("VS")
sch.flag(d, tmp.VS, "3V3_AO", "left", 0.6)
d += elm.Ground().at(tmp.GND)
rc_out(tmp.VOUT, "TEMP_BOARD (PB1)")
sch.note(d, (12.5, -31.8), "10 mV/C, 0.5 V at 0 C: adc.rs Linear(100, -50). The TMP117 (S3) also\n"
         "measures the board; one of the two may go (TBR).", fontsize=7)

d.save(sch.out_path("S9_adc_front_ends"))
