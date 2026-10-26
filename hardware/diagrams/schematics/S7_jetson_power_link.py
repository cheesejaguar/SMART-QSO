"""S7 Jetson power gate (soft start, current limit, monitor) and the USART1 link with back-power isolation."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S7_jetson_power_link")
sch.title(d, (-4, 6.5), "S7", "Jetson power gate and link",
          "Sources: hardware/payload-jetson/POWER_MANAGEMENT.md, INTEGRATION_DESIGN.md; hardware/eps/PDU_DESIGN.md "
          "(SW1, INA226); software/rust/firmware/src/gpio.rs (PA2), uart.rs (USART1); W5")

# ---- power gate
ef = sch.ic("U1 eFuse / hot-swap (TBD)\nTPS25947 class: soft start,\n4 A limit, reverse blocking",
            left=["EN"], top=["VIN"], right=["PG", "VOUT"], bottom=["ILIM", "dVdt", "GND"], tbd=True, size=(6.0, 2.6),
            spread=("bottom",))
d += ef.at((0, 0))
d += elm.Line().at(ef.VIN).up().length(1.0)
d += (vi := elm.Dot())
sch.flag(d, vi.center, "5V_PAYLOAD (SW1)", "left", 1.6)
d += elm.Capacitor().at(vi.center).right().length(1.4).label("10 uF", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Line().at(ef.EN).left().length(1.0)
d += (en := elm.Dot())
d += elm.Resistor().at(en.center).down().length(1.4).label("100k", loc="top", fontsize=7)
d += elm.Ground()
sch.flag(d, en.center, "JETSON_PWR (PA2)", "left", 0.6)
d += elm.Resistor().at(ef.ILIM).down().length(1.4).label("R_ILIM\n4 A (TBR)", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Capacitor().at(ef.dVdt).down().length(1.4).label("C_dVdt\n~10 ms ramp", loc="bottom", fontsize=7)
d += elm.Ground()
d += elm.Line().at(ef.GND).down().length(0.4)
d += elm.Ground()
d += elm.Line().at(ef.PG).right().length(0.6)
d += elm.Label().label("PG: no OBC pin (TBD)", loc="right", fontsize=7)

# shunt and INA226
d += elm.Line().at(ef.VOUT).right().length(0.8)
d += (s1 := elm.Dot())
d += (rsh := elm.Resistor().right().length(2.0).label("R1 10 mOhm\nshunt (TBR)", loc="top", fontsize=7))
d += (s2 := elm.Dot())
d += elm.Line().right().length(1.0)
d += (o := elm.Dot())
d += elm.Capacitor().at(o.center).down().length(1.2).label("100 uF bulk (TBR)", loc="bottom", fontsize=7)
d += elm.Ground()
sch.flag(d, o.center, "+5V_JET (W5 J_EPS_JET 1-2)", "right", 0.8)
ina = sch.ic("U2 INA226\nI2C2 (address TBD)", top=["IN+", "IN-"], right=["SDA", "SCL", "ALERT"],
             bottom=["VBUS", "A0", "A1", "GND"], size=(5.0, 2.6), spread=("top", "bottom"))
d += ina.at((s1.center[0], s1.center[1] - 4.5)).anchor("INP")
d += elm.Line().at(s1.center).to(ina.INP).linestyle(":")
d += elm.Line().at(s2.center).down().length(2.8).linestyle(":")
d += elm.Line().right().tox(ina.INN[0]).linestyle(":")
d += elm.Line().to(ina.INN).linestyle(":")
sch.flag(d, ina.SDA, "I2C2_SDA", "right", 0.6)
sch.flag(d, ina.SCL, "I2C2_SCL", "right", 0.6)
sch.flag(d, ina.ALERT, "INA_ALERT#", "right", 0.6)
for a in ("A0", "A1", "GND"):
    d += elm.Line().at(getattr(ina, a)).down().length(0.4)
    d += elm.Ground()
d += elm.Line().at(ina.VBUS).down().length(0.4)
d += elm.Label().label("to +5V_JET", loc="bottom", fontsize=7)

# ---- UART link through translators with Ioff (no back-power into an unpowered Jetson)
yl = -14
tx = sch.ic("U3 74LVC1T45 (TBR)\nDIR = 1: A to B", left=["DIR", "A"], right=["B"], top=["VCCA", "VCCB"],
            bottom=["GND"], size=(4.6, 2.0), spread=("top",))
d += tx.at((2, yl)).anchor("A")
sch.flag(d, tx.A, "USART1_TX (PA9)", "left", 1.0)
sch.flag(d, tx.DIR, "3V3_AO", "left", 1.0)
sch.flag(d, tx.B, "JET_UART_RXD (W5)", "right", 0.8)
rx = sch.ic("U4 74LVC1T45 (TBR)\nDIR = 0: B to A", left=["DIR", "A"], right=["B"], top=["VCCA", "VCCB"],
            bottom=["GND"], size=(4.6, 2.0), spread=("top",))
d += rx.at((2, yl - 6)).anchor("A")
sch.flag(d, rx.A, "USART1_RX (PA10)", "left", 1.0)
d += elm.Line().at(rx.DIR).left().length(0.6)
d += elm.Ground()
sch.flag(d, rx.B, "JET_UART_TXD (W5)", "right", 0.8)
for u in (tx, rx):
    d += elm.Line().at(u.VCCA).up().length(0.5)
    d += elm.Vdd().label("3V3_AO", fontsize=7)
    d += elm.Line().at(u.VCCB).up().length(0.5)
    d += elm.Vdd().label("JET_3V3 (carrier)", fontsize=7)
    d += elm.Ground().at(u.GND)
d += elm.Resistor().at((rx.A[0] - 0.5, rx.A[1])).up().length(1.5).label("100k pull-up\nidle high", loc="bottom", fontsize=6)
sch.note(d, (10, yl + 1.5),
         "Finding F13: while the Jetson is off, the OBC's USART1 TX would back-power the carrier through its\n"
         "I/O clamp diodes, and an unpowered Jetson RX line would load PA9. Translators with Ioff on both\n"
         "sides (74LVC1T45: outputs off when either supply is 0 V) isolate the two. JET_3V3 is the carrier's\n"
         "own 3.3 V on its UART header (TBR); the Orin Nano 40-pin header gives UART1 on pins 8 / 10.\n"
         "The carrier, its connector and the Jetson's own power-up sequencing are TBD.", fontsize=7)

d.save(sch.out_path("S7_jetson_power_link"))
