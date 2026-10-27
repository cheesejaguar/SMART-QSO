"""S7 Jetson power switches, readbacks, the USART1 link and the carrier, version 3 (2026-10-27). The two switches are on
the EPS board (SQSO-EPS-BRD-001 8.3: SW1 TPS1H100B-Q1 on PAYLOAD_EN, U19 TPS16630 eFuse on JETSON_PWR); the readbacks and
the back-power isolation are on the OBC board (SQSO-OBC-BRD-001 11); the carrier (SQSO-JET-BRD-001 A) makes the module's
5 V VDD_IN, sequences POWER_EN and switches its I/O rails with SYS_RESET*. Version 1 drew a 5 V payload rail, an INA226
and an eFuse with no readback; version 2 had no carrier.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S7_jetson_power_link")
sch.note(d, (-8, 8), "S7  Jetson power switches, readbacks, link and carrier (version 3)", fontsize=12, color="black")
sch.note(d, (-8, 7.4), "SQSO-HW-DIAG-001 · EPS side: SQSO-EPS-BRD-001 8.3 · OBC side: SQSO-OBC-BRD-001 11 (RIDs AV-06, AV-09) · "
         "carrier: SQSO-JET-BRD-001 · pins: SQSO-ICX-001", fontsize=8)

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

# ---- the carrier (SQSO-JET-BRD-001 A)
yc = -27
sch.note(d, (-8, yc + 2.6), "Carrier (SQSO-JET-BRD-001): VBAT_JET -> 5 V VDD_IN (Orin Nano: 4.75-5.25 V only, DS-11105-001 Table 5-1)",
         fontsize=9, color="black")
u1 = sch.ic("U1 LM61440-Q1\n5.02 V, 1.000 MHz sync", left=["VIN"], right=["VOUT"], bottom=["GND"], size=(4.6, 2.0))
d += u1.at((0, yc)).anchor("VIN")
sch.flag(d, u1.VIN, "J1 VBAT_JET -> TVS2200, 4.7 uH + damper", "left", 1.0)
d += elm.Ground().at(u1.GND)
u2 = sch.ic("U2 TPS259474L\nOVLO 5.38 V, 2.51 A breaker\nlatch-off", left=["IN"], right=["OUT", "PG"], bottom=["GND"], size=(5.0, 2.4))
d += elm.Line().at(u1.VOUT).right().length(1.2)
d += (q := elm.Dot(open=True))
d += u2.at(q.center).anchor("IN")
d += elm.Ground().at(u2.GND)
sch.flag(d, u2.OUT, "VDD_IN -> module pins 251-260", "right", 0.8)
sch.flag(d, u2.PG, "POWER_EN (pin 237), 10 k to VDD_IN", "right", 0.8)
u4 = sch.ic("U4 TPS3808G01-Q1\nVBAT_F > 10.49 V for 19 ms", left=["SENSE", "MR"], right=["RESET"], bottom=["GND"], size=(5.0, 2.2))
d += u4.at((0, yc - 5)).anchor("SENSE")
sch.flag(d, u4.SENSE, "VBAT_F (249 k / 10 k)", "left", 0.8)
sch.flag(d, u4.MR, "U5 latch Q* <- U6 <- SHUTDOWN_REQ* (pin 233)", "left", 0.8)
sch.flag(d, u4.RESET, "POWER_EN (wired AND with U2 PG)", "right", 0.8)
d += elm.Ground().at(u4.GND)
u7 = sch.ic("U7 TPS22918-Q1\nQOD", left=["VIN", "ON"], right=["VOUT"], bottom=["GND"], size=(4.0, 2.0))
d += u7.at((0, yc - 10)).anchor("VIN")
sch.flag(d, u7.VIN, "3V3_PRE (U3 LMR43620-Q1)", "left", 0.8)
sch.flag(d, u7.ON, "SYS_RESET* (pin 239, module)", "left", 0.8)
d += elm.Ground().at(u7.GND)
sch.flag(d, u7.VOUT, "3V3_IO -> 100 Ohm -> J2-4 JET_3V3; U9 -> 1V8_IO; U10/U11 VCCB", "right", 0.8)
sch.note(d, (-8, yc - 13.5),
         "U10/U11 SN74LVC1T45-Q1 (VCCA 1V8_IO, VCCB 3V3_IO): module UART1 TXD pin 203 -> J2-3, J2-2 -> RXD pin 205 (1.8 V CMOS).\n"
         "JET_3V3 rises only after the module's power sequence (SYS_RESET* high) and falls within 0.1 ms of a carrier trip or\n"
         "a module shutdown: the carrier power-good (AV-06). No module I/O sees > 0.5 V before SYS_RESET* (DS 3.6.1; AV-09).\n"
         "J2-5/-6: spreader NTC passed through to the OBC (change E10). Storage: M.2 2230 NVMe on PCIe C4 (3V3_SSD via U8).",
         fontsize=7)

d.save(sch.out_path("S7_jetson_power_link"))
