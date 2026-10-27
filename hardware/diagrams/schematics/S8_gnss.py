"""S8 GNSS receiver interface, version 2 (2026-10-27): SkyFox piNAV-NG/FM (decision D6) on LPUART1 PG7/PG8 with
GNSS_EN on PE5 (RID EPS-14; PE4 is XPDR_EN), 1PPS on TIM3_CH1 (PC6). The receiver's 3.3 V from EPS SW7 passes through
the OBC so that the translators take their receiver side from it (SQSO-OBC-BRD-001 12). Version 1 proposed UART4 on
PC10/PC11 and GNSS_EN on PE4.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S8_gnss")
sch.note(d, (-8, 6.5), "S8  GNSS receiver interface (version 2)", fontsize=12, color="black")
sch.note(d, (-8, 5.9), "SQSO-HW-DIAG-001 · SQSO-OBC-BRD-001 12 · harness W9 version 2 · pins: SQSO-ICX-001 3.5 · receiver-side "
         "pin order per the SkyFox interface document", fontsize=8)

u = sch.ic("piNAV-NG/FM receiver\n(SkyFox, D6)", left=["1PPS", "TXD", "RXD"], right=["RF_IN"], top=["VIN"], bottom=["GND"],
           size=(5.6, 3.0), spread=("left",))
d += u.at((14, 0))
d += elm.Line().at(u.VIN).up().length(0.6)
d += (v := elm.Dot())
sch.flag(d, v.center, "3V3_GNSS: EPS SW7 (GNSS_EN PE5) -> W9B -> OBC J4 -> J5-1 -> W9A", "left", 1.0)
d += elm.Ground().at(u.GND)
d += elm.Line().at(u.RF_IN).right().length(0.8)
d += elm.Antenna().label("L1 patch (Spacemanic ZENITH V3a)", loc="right", fontsize=7)
for k, (ref, a, b, net, dirn) in enumerate((("U30", "RXD", "B", "LPUART1_TX (PG7)", "A to B"),
                                            ("U31", "TXD", "B", "LPUART1_RX (PG8, 100k up)", "B to A"),
                                            ("U32", "1PPS", "B", "GNSS_PPS (PC6, TIM3_CH1, 100k down)", "B to A"))):
    t = sch.ic(f"{ref} SN74LVC1T45-Q1\n{dirn}", left=["A"], right=["B"], top=["VCCA", "VCCB"], size=(4.2, 1.6), spread=("top",))
    d += t.at((0, -2.4 * k + 1.2)).anchor("A")
    sch.flag(d, t.A, net, "left", 0.8)
    d += elm.Line().at(t.VCCA).up().length(0.4)
    d += elm.Label().label("3V3_OBC", loc="top", fontsize=6)
    d += elm.Line().at(t.VCCB).up().length(0.4)
    d += elm.Label().label("3V3_GNSS", loc="top", fontsize=6)
    d += elm.Line().at(t.B).to(sch.pin(u, a))
sch.note(d, (-8, -6.5),
         "Each translator's receiver side (VCCB) is 3V3_GNSS and its OBC side (VCCA) 3V3_OBC: with GNSS_EN low or SW7 tripped\n"
         "(0.175-0.232 A latch-up limit, 1.3 s off), the UART and 1PPS lines are high-impedance on the receiver side, so the OBC\n"
         "neither back-powers the receiver nor feeds a latch-up. GNSS_PG: 3V3_GNSS -> 10k -> PG15 (100k down) shows a trip.\n"
         "10 uF + 100 nF at J5. ESD: TPD4E05U06-Q1 at J5.", fontsize=7)

d.save(sch.out_path("S8_gnss"))
