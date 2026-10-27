"""S7 Jetson power switches, readbacks and the USART1 link, version 2 (2026-10-27). The two switches are on the EPS
board (SQSO-EPS-BRD-001 8.3: SW1 TPS1H100B-Q1 on PAYLOAD_EN, U19 TPS16630 eFuse on JETSON_PWR); the readbacks and the
back-power isolation are on the OBC board (SQSO-OBC-BRD-001 11). Version 1 drew a 5 V payload rail, an INA226 and an
eFuse with no readback; the carrier takes VBAT (RID EMC-02).
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S7_jetson_power_link")
sch.note(d, (-8, 8), "S7  Jetson power switches, readbacks and link (version 2)", fontsize=12, color="black")
sch.note(d, (-8, 7.4), "SQSO-HW-DIAG-001 · EPS side: SQSO-EPS-BRD-001 8.3 · OBC side: SQSO-OBC-BRD-001 11 (RIDs AV-06, AV-09) · "
         "pins: SQSO-ICX-001", fontsize=8)

# ---- the two switches in series (EPS board)
sw1 = sch.ic("EPS SW1 TPS1H100B-Q1\nlimit 2.0 A, shed-gated", left=["IN"], top=["VS"], right=["OUT"], bottom=["GND"], size=(5.0, 2.2))
d += sw1.at((0, 0))
sch.flag(d, sw1.IN, "PAYLOAD_EN (PB2, port B) AND SHED_N", "left", 0.8)
d += elm.Line().at(sw1.VS).up().length(0.6)
d += elm.Vdd().label("VBAT", fontsize=7)
d += elm.Ground().at(sw1.GND)
d += elm.Line().at(sw1.OUT).right().length(1.6)
d += (n1 := elm.Dot())
ef = sch.ic("EPS U19 TPS16630 eFuse\n1.20 A latch-off, <= 1 ms", left=["IN", "SHDN"], right=["OUT"], bottom=["GND"], size=(5.0, 2.2))
d += elm.Line().at(n1.center).right().length(4.5)
d += (p_in := elm.Dot(open=True))
d += ef.at(p_in.center).anchor("IN")
sch.flag(d, ef.SHDN, "JETSON_PWR (PA2, port A)", "left", 0.6)
d += elm.Ground().at(ef.GND)
d += elm.Line().at(ef.OUT).right().length(1.6)
d += (n2 := elm.Dot())
sch.flag(d, n2.center, "VBAT_JET -> EPS J18 -> W5A -> carrier", "right", 1.0)
# readbacks
for node, net in ((n1, "VSW1_RB -> J3-37 -> PA1 (ADC1_IN6)"), (n2, "VJET_RB -> J3-19 -> PA7 (ADC1_IN12)")):
    d += elm.Resistor().at(node.center).down().length(1.8).label("1.00M", loc="bottom", fontsize=7)
    d += (m := elm.Dot())
    d += elm.Resistor().down().length(1.6).label("200k", loc="bottom", fontsize=7)
    d += elm.Ground()
    sch.flag(d, m.center, net, "left" if node is n1 else "right", 0.6)
sch.note(d, (-8, -6.2), "Readbacks measure each switch's output (VBAT / 6: 1.87-2.80 V on, < 0.1 V off), not its command. The two\n"
         "enables are on different MCU ports, so one port-register upset cannot set both. PB4 (the firmware's pin) resets as\n"
         "NJTRST with a pull-up and would switch SW1 on at every OBC reset; PB2 has no reset pull.", fontsize=7)

# ---- UART link through translators powered from the carrier
yl = -12
tx = sch.ic("U28 SN74LVC1T45-Q1\nDIR = 1: A to B", left=["DIR", "A"], right=["B"], top=["VCCA", "VCCB"], bottom=["GND"],
            size=(4.6, 2.0), spread=("top",))
d += tx.at((2, yl)).anchor("A")
sch.flag(d, tx.A, "USART1_TX (PA9)", "left", 1.0)
sch.flag(d, tx.DIR, "3V3_OBC", "left", 1.0)
sch.flag(d, tx.B, "33 Ohm -> J3-2 -> carrier RXD", "right", 0.8)
rx = sch.ic("U29 SN74LVC1T45-Q1\nDIR = 0: B to A", left=["DIR", "A"], right=["B"], top=["VCCA", "VCCB"], bottom=["GND"],
            size=(4.6, 2.0), spread=("top",))
d += rx.at((2, yl - 6)).anchor("A")
sch.flag(d, rx.A, "USART1_RX (PA10, 100k up)", "left", 1.0)
d += elm.Line().at(rx.DIR).left().length(0.6)
d += elm.Ground()
sch.flag(d, rx.B, "33 Ohm <- J3-3 <- carrier TXD", "right", 0.8)
for u in (tx, rx):
    d += elm.Line().at(u.VCCA).up().length(0.5)
    d += elm.Vdd().label("3V3_OBC", fontsize=7)
    d += elm.Line().at(u.VCCB).up().length(0.5)
    d += elm.Vdd().label("JET_3V3 (J3-4)", fontsize=7)
    d += elm.Ground().at(u.GND)
sch.flag(d, (16, yl - 3), "JET_3V3 (J3-4) -> 10k -> JET_PG (PG6), 100k down", "right", 0.01)
sch.note(d, (12, yl + 1.5),
         "RID AV-09: the translators' carrier side is powered by the carrier's own 3.3 V I/O rail on J3-4. With the carrier\n"
         "off, VCCB = 0 V and both ports are high-impedance (VCC isolation, Ioff): the idle-high TX cannot back-power the\n"
         "module, whatever the firmware does. The carrier must provide its 3.3 V rail on its pin 4 (<= 1 mA).\n"
         "JET_PG is the third readback (RID AV-06): the carrier's converter is running.", fontsize=7)

d.save(sch.out_path("S7_jetson_power_link"))
