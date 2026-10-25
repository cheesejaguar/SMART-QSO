"""S4 RF board: UHF receive chain, KISS TNC, transponder path, VHF exciter, PA, coupler, 7-pole LPF, monitors."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S4_rf_board")
sch.title(d, (-4, 6.5), "S4", "RF board",
          "Sources: hardware/rf/TRANSMITTER_DESIGN.md 7.2-7.3 (LPF), 8 (monitors), 10.1 (OBC interface); "
          "hardware/rf/RF_ELECTRICAL_DESIGN.md (LNA); docs/hardware/PARTS_LIST.md (SAW); BASELINE 7")


def supply(u, pin, net):
    d.add(elm.Line().at(sch.pin(u, pin)).up().length(0.5))
    d.add(elm.Vdd().label(net, fontsize=7))


# ---- receive: antenna, SAW, LNA, receiver
d += (ant := elm.Antenna().at((0, 0)).label("70 cm λ/4\nmonopole", loc="left", fontsize=7))
d += elm.Line().at((0, 0)).down().length(0.6)
d += elm.Line().right().length(0.6)
d += elm.Label().label("J2 SMA (W10)", loc="top", fontsize=7)
p = (1.2, -0.6)
saw = sch.ic("U1 SAW\nSF2098B\n435 MHz (TBR)", left=["IN"], right=["OUT"], bottom=["GND"])
d += saw.at((p[0] + 0.6, p[1])).anchor("IN")
d += elm.Line().at(p).to(saw.IN)
d += elm.Ground().at(saw.GND)
lna = sch.ic("U2 LNA BGA725\n(alt. SPF5189Z)", left=["RFIN"], right=["RFOUT"], top=["VCC"], bottom=["GND"])
d += lna.at((saw.OUT[0] + 1.4, saw.OUT[1])).anchor("RFIN")
d += elm.Line().at(saw.OUT).to(lna.RFIN)
supply(lna, "VCC", "RADIO_3V3")
d += elm.Ground().at(lna.GND)
rx = sch.ic("U3 UHF FM receiver (TBD)\n435-438 MHz, always on", left=["RF_IN"], right=["SQL", "AF_OUT"],
            top=["VCC"], bottom=["GND"], tbd=True)
d += rx.at((lna.RFOUT[0] + 1.4, lna.RFOUT[1])).anchor("RF_IN")
d += elm.Line().at(lna.RFOUT).to(rx.RF_IN)
supply(rx, "VCC", "RADIO_3V3")
d += elm.Ground().at(rx.GND)

# ---- baseband: TNC and transponder / arbiter
tnc = sch.ic("U4 KISS TNC (TBD)\n1200 bps AFSK, full duplex\nadds FCS, modulates, keys",
             left=["PTT", "TX_AF", "RX_AF"], right=["TXD", "RXD"], top=["VCC"], bottom=["GND"], tbd=True)
d += tnc.at((rx.AF_OUT[0] + 4.5, rx.AF_OUT[1] + 0.8)).anchor("RX_AF")
sch.flag(d, rx.AF_OUT, "RX_AF", "right", 0.6)
sch.flag(d, rx.SQL, "RX_SQL", "right", 0.6)
sch.flag(d, tnc.RX_AF, "RX_AF", "left", 0.6)
sch.flag(d, tnc.TX_AF, "TNC_AF", "left", 0.6)
sch.flag(d, tnc.PTT, "TNC_PTT", "left", 0.6)
supply(tnc, "VCC", "RADIO_3V3")
d += elm.Ground().at(tnc.GND)
sch.flag(d, tnc.TXD, "USART2_RX (PD6)", "right", 0.6)
sch.flag(d, tnc.RXD, "USART2_TX (PD5)", "right", 0.6)

arb = sch.ic("U5 transponder audio path\n+ TX audio / PTT arbiter (TBD)\nno control line defined (finding F11)",
             left=["SQL", "RX_AF", "TNC_AF", "TNC_PTT"], right=["PTT", "MOD_AF"], tbd=True)
d += arb.at((tnc.PTT[0], tnc.PTT[1] - 5.0)).anchor("TNC_PTT")
for a, net in (("SQL", "RX_SQL"), ("RX_AF", "RX_AF"), ("TNC_AF", "TNC_AF"), ("TNC_PTT", "TNC_PTT")):
    sch.flag(d, sch.pin(arb, a), net, "left", 0.6)
sch.flag(d, arb.MOD_AF, "MOD_AF", "right", 0.6)
sch.flag(d, arb.PTT, "TX_PTT", "right", 0.6)

# ---- transmit: exciter, PA, coupler, LPF, antenna
yt = arb.MOD_AF[1] - 6.0
exc = sch.ic("U6 VHF FM exciter (TBD)\nDRA818V / TX2M or eq.", left=["PTT", "MOD_AF"], right=["RF_OUT"],
             top=["VCC"], bottom=["GND"], tbd=True)
d += exc.at((2, yt)).anchor("MOD_AF")
sch.flag(d, exc.MOD_AF, "MOD_AF", "left", 0.6)
sch.flag(d, exc.PTT, "TX_PTT", "left", 0.6)
supply(exc, "VCC", "RADIO_3V3")
d += elm.Ground().at(exc.GND)

pa = sch.ic("U7 PA (TBD)\n0.5 W nom / 1 W max", left=["PWR_SEL", "RF_IN"], right=["RF_OUT"], top=["VCC"],
            bottom=["GND"], tbd=True)
d += pa.at((exc.RF_OUT[0] + 5.0, exc.RF_OUT[1])).anchor("RF_IN")
d += elm.Line().at(exc.RF_OUT).to(pa.RF_IN)
supply(pa, "VCC", "PA_5V (SW3)")
d += elm.Ground().at(pa.GND)
sch.flag(d, pa.PWR_SEL, "TX_PWR_SEL (PE3)", "left", 0.6)

cpl = sch.ic("U8 directional\ncoupler -20 dB", left=["IN"], right=["OUT"], bottom=["FWD", "REV"])
d += cpl.at((pa.RF_OUT[0] + 1.4, pa.RF_OUT[1])).anchor("IN")
d += elm.Line().at(pa.RF_OUT).to(cpl.IN)

# 7-pole LPF, TRANSMITTER_DESIGN 7.3 values
d += elm.Line().at(cpl.OUT).right().length(0.6)
for i, (lv, cv) in enumerate([("47 nH", "33 pF"), ("82 nH", "47 pF"), ("82 nH", "33 pF"), ("47 nH", None)]):
    d += elm.Inductor2(loops=3).right().length(1.6).label(f"L{i + 1}\n{lv}", loc="top", fontsize=7)
    if cv:
        d += (n := elm.Dot())
        d += elm.Capacitor().at(n.center).down().length(1.2).label(f"C{i + 1}\n{cv}", loc="right", fontsize=7)
        d += elm.Ground()
        d += elm.Line().at(n.center).right().length(0.01)
d += elm.Line().right().length(0.8)
d += (j1 := elm.Dot().label("J1 SMA (W10)", loc="bottom", fontsize=7))
d += elm.Line().up().length(0.6)
d += elm.Antenna().label("2 m λ/4\nmonopole", loc="right", fontsize=7)
sch.note(d, (cpl.OUT[0] + 0.6, cpl.OUT[1] + 2.4),
         "7-pole LPF, f_c 160 MHz: TRANSMITTER_DESIGN 7.3 values. Ideal-part ABCD check:\n"
         "-0.1 dB at 145.9 MHz, -50 dB at 2f, -77 dB at 3f; meets 7.2 (> 50 / > 60 dB) and the BASELINE\n"
         "3rd-harmonic target. Values are not the textbook 0.5 dB Chebyshev prototype; confirm by EM test (TBR).",
         fontsize=7)

# ---- monitors
def detector(x, y, ref, tap, net, pin):
    u = sch.ic(f"{ref} AD8361 or eq.\nRMS detector", left=["RFIN"], right=["VRMS"], top=["VPOS"], bottom=["COM"])
    d.add(u.at((x, y)).anchor("RFIN"))
    d.add(elm.Wire("|-").at(tap).to(u.RFIN))
    supply(u, "VPOS", "RADIO_3V3")
    d.add(elm.Ground().at(u.COM))
    d.add(elm.Resistor().at(u.VRMS).right().length(1.4).label("1k", loc="top", fontsize=7))
    d.add(n := elm.Dot())
    d.add(elm.Capacitor().at(n.center).down().length(1.0).label("100 nF", loc="right", fontsize=7))
    d.add(elm.Ground())
    sch.flag(d, n.center, f"{net} ({pin})", "right", 0.6)


detector(cpl.FWD[0] + 1.0, yt - 4.5, "U9", cpl.FWD, "TX_FWD", "PC0")
detector(cpl.REV[0] + 1.0, yt - 8.5, "U10", cpl.REV, "TX_REV", "PC1")
sch.note(d, (cpl.OUT[0] + 7.5, yt - 3.2), "Coupled ports through 50 Ohm pads (TBR);\n0-3 V out, 2.5 V at 1 W (adc.rs)\n"
         "VSWR > 3:1: 0.5 W, then off 10 min", fontsize=7)

tmp = sch.ic("U11 TMP36 on the PA\n10 mV/C, 0.5 V at 0 C", left=["+VS"], right=["VOUT"], bottom=["GND"])
d += tmp.at((0, yt - 6.5)).anchor("VS")
sch.flag(d, tmp.VS, "RADIO_3V3", "left", 0.6)
d += elm.Ground().at(tmp.GND)
d += elm.Resistor().at(tmp.VOUT).right().length(1.4).label("1k", loc="top", fontsize=7)
d += (n := elm.Dot())
d += elm.Capacitor().at(n.center).down().length(1.0).label("100 nF", loc="right", fontsize=7)
d += elm.Ground()
sch.flag(d, n.center, "TX_TEMP (PA0)", "right", 0.6)
sch.note(d, (0, yt - 9.0), "> 70 C: downlink held until 60 C", fontsize=7)

d.save(sch.out_path("S4_rf_board"))
