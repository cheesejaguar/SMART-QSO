"""S8 GNSS receiver interface (decision D6: pNAV-NG/FM). Everything here is proposed (finding F2)."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S8_gnss")
sch.title(d, (-6, 6), "S8", "GNSS receiver interface (proposed)",
          "Sources: docs/BASELINE_PARAMETERS.md D6 (pNAV-NG/FM); W9, W10. Pin order of the receiver: vendor ICD (TBD)")

u = sch.ic("U1 GNSS receiver\npNAV-NG/FM (D6)\npinout per vendor ICD (TBD)",
           left=["1PPS", "RXD", "TXD"], right=["RF_IN"], top=["VIN"], bottom=["GND"], tbd=True, size=(5.6, 3.0),
           spread=("left",))
d += u.at((0, 0))
d += elm.Line().at(u.VIN).up().length(0.5)
d += (v := elm.Dot())
sch.flag(d, v.center, "3V3_GNSS (SW7, S2)", "left", 0.8)
d += elm.Capacitor().at(v.center).right().length(1.4).label("10 uF + 100 nF", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Ground().at(u.GND)
for pin, net in (("TXD", "UART4_RX* (PC11)"), ("RXD", "UART4_TX* (PC10)"), ("1PPS", "PPS* (timer input TBD)")):
    d += elm.Resistor().at(getattr(u, sch.anchor(pin))).left().length(1.6).label("1k", loc="top", fontsize=7)
    sch.flag(d, d.here, net, "left", 0.6)
d += elm.Line().at(u.RF_IN).right().length(0.8)
d += elm.Label().label("U.FL / MMCX (TBD)", loc="top", fontsize=7)
d += elm.Line().right().length(1.2)
d += elm.Antenna().label("L1 patch, active (TBD)\nbias from the receiver", loc="right", fontsize=7)
sch.note(d, (-6, -3.2),
         "Neither the firmware nor any ICD has a GNSS UART, enable or 1PPS (finding F2). Proposed: UART4 on PC10/PC11 (the pins\n"
         "of SPI3, which nothing uses), GNSS_EN on PE4 switching SW7 (S2), 1PPS to a timer input capture (pin TBD).\n"
         "The 1k series resistors limit back-power while SW7 is off (TBR; a translator as in S7 is the alternative).",
         fontsize=7)

d.save(sch.out_path("S8_gnss"))
