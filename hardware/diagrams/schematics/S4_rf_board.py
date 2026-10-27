"""S4 RF board (version 2, 2026-10-27): the build-to design of SQSO-RF-BRD-001, drawn at block level.

The netlist of record is hardware/electrical/rf/design.py (SKiDL) and its rf_board.net; this sheet shows the same
parts and net names so that the harness drawings (W4 v2, W10 v2) and the board design read together.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S4_rf_board")
sch.title(d, (-4, 7.0), "S4", "RF board v2 (SQSO-RF-BRD-001)",
          "Netlist of record: hardware/electrical/rf/design.py (SKiDL) -> rf_board.net, BOM, ERC log; "
          "calculations: rf_calcs.py; BASELINE 7")

# ---------------------------------------------------------------- receive chain (top row)
y = 3.0
d += elm.Antenna().at((-3.5, y + 0.6)).label("70 cm pair\n+Y / -Y (AntS)", loc="left", fontsize=7)
bal_u = sch.ic("L101/C101/C102/L102\nlattice balun\n(J5, J6)", left=["A", "B"], right=["SE"])
d += bal_u.at((-2.5, y)).anchor("A")
fl1 = sch.ic("FL1 7th-order HPF\n3 traps 143.5-146.5 MHz\n0.70 dB, >= 60 dB at 144-146", left=["IN"], right=["OUT"])
d += fl1.at((bal_u.SE[0] + 1.2, bal_u.SE[1])).anchor("IN")
d += elm.Line().at(bal_u.SE).to(fl1.IN)
lna = sch.ic("Q201 BFP640F LNA\n6 mA, NF 0.8 dB, 18 dB", left=["IN"], right=["OUT"], top=["VCC"])
d += lna.at((fl1.OUT[0] + 1.2, fl1.OUT[1])).anchor("IN")
d += elm.Line().at(fl1.OUT).to(lna.IN)
saw = sch.ic("U201 SAW TA2209A\n433-437 MHz", left=["IN"], right=["OUT"])
d += saw.at((lna.OUT[0] + 1.2, lna.OUT[1])).anchor("IN")
d += elm.Line().at(lna.OUT).to(saw.IN)
cc = sch.ic("U202 CC1200 (RX only)\nCFM analog-FM soft output\n12.5 kS/s over SPI", left=["LNA_P"],
            right=["SPI", "CFMCLK"], top=["EXT_XOSC"])
d += cc.at((saw.OUT[0] + 1.2, saw.OUT[1])).anchor("LNA_P")
d += elm.Line().at(saw.OUT).to(cc.LNA_P)
sch.flag(d, cc.SPI, "SPI1", "right", 0.6)
sch.flag(d, cc.CFMCLK, "CC_CFMCLK", "right", 0.6)
tcxo = sch.ic("Y301 TCXO 40 MHz\n+-0.5 ppm, -40..85 C", right=["OUT"])
d += tcxo.at((cc.EXT_XOSC[0] - 4.5, cc.EXT_XOSC[1] + 1.6)).anchor("OUT")
d += elm.Wire("-|").at(tcxo.OUT).to(cc.EXT_XOSC)
sch.note(d, (tcxo.OUT[0] - 4.0, tcxo.OUT[1] - 1.0), "also to the squarer U301 -> LMX2571 OSCin, PCM1808 SCKI "
         "(PA domain)", fontsize=7)

# ---------------------------------------------------------------- TNC and key logic (middle row)
ym = -2.2
mcu = sch.ic("U310 STM32L431 KISS TNC\nAFSK 1200 modem, HDLC, KISS\nconfigures CC1200/CMX138A/LMX2571",
             left=["SPI1", "CFMCLK", "IRQN"], right=["USART2", "DAC1", "DAC2", "TONE_OK", "TNC_PTT"])
d += mcu.at((0, ym)).anchor("SPI1")
for p_, n_ in (("SPI1", "SPI1"), ("CFMCLK", "CC_CFMCLK"), ("IRQN", "CMX_IRQN")):
    sch.flag(d, sch.pin(mcu, p_), n_, "left", 0.6)
sch.flag(d, mcu.USART2, "USART2 9600 (J1-2/3)", "right", 0.6)
sch.flag(d, mcu.DAC1, "TNC_AF", "right", 0.6)
sch.flag(d, mcu.DAC2, "RPT_AF_RAW", "right", 0.6)
sch.flag(d, mcu.TONE_OK, "TONE_OK", "right", 0.6)
sch.flag(d, mcu.TNC_PTT, "TNC_PTT", "right", 0.6)

cmx = sch.ic("U452 CMX138A\nCTCSS 67.0 Hz decode,\n300-3000 Hz, level limit", left=["DISC"], right=["AUDIO", "IRQN"])
d += cmx.at((16.0, ym + 0.8)).anchor("DISC")
sch.flag(d, cmx.DISC, "RPT_AF_RAW (4 kHz LPF)", "left", 0.6)
sch.flag(d, cmx.IRQN, "CMX_IRQN", "right", 0.6)
sch.flag(d, cmx.AUDIO, "RPT_AF_AC", "right", 0.6)

mono = sch.ic("U401 LVC1G123\nretriggerable 49.7 ms", left=["B"], right=["Q"])
d += mono.at((0, ym - 5.0)).anchor("B")
sch.flag(d, mono.B, "XPDR_EN (J1-6, 100 k pull-down)", "left", 0.6)
sch.flag(d, mono.Q, "EN_OK", "right", 0.6)
tot = sch.ic("U403 HC4060 + D401\n120 s key-down timer\nreset = NOT TONE_OK", left=["MR"], right=["Q14"])
d += tot.at((0, ym - 8.5)).anchor("MR")
sch.flag(d, tot.MR, "TONE_N", "left", 0.6)
sch.flag(d, tot.Q14, "NOT_TOT (via U402)", "right", 0.6)
and3 = sch.ic("U404 LVC1G11\nXPDR_KEY = EN_OK.TONE_OK.NOT_TOT", left=["A", "B", "C"], right=["Y"])
d += and3.at((10.5, ym - 6.5)).anchor("B")
for p_, n_ in (("A", "EN_OK"), ("B", "TONE_OK"), ("C", "NOT_TOT")):
    sch.flag(d, sch.pin(and3, p_), n_, "left", 0.6)
sch.flag(d, and3.Y, "XPDR_KEY", "right", 0.6)
gate = sch.ic("U405 OR + U406 AND\nPTT_EXCITER = TX_PERMIT_HW .\n(TNC_PTT + XPDR_KEY)",
              left=["TNC_PTT", "XPDR_KEY", "PERMIT"], right=["PTT_EXC"])
d += gate.at((23.0, ym - 6.5)).anchor("XPDR_KEY")
for p_, n_ in (("TNC_PTT", "TNC_PTT"), ("XPDR_KEY", "XPDR_KEY"), ("PERMIT", "TX_PERMIT_HW")):
    sch.flag(d, sch.pin(gate, p_), n_, "left", 0.6)
sch.flag(d, gate.PTT_EXC, "PTT_EXC", "right", 0.6)
sch.note(d, (10.5, ym - 9.5), "Readbacks to OBC inputs through 1 k: TONE_OK (J1-8), XPDR_KEY (J1-9), PTT_EXCITER (J1-10); "
         "STM32 pins allocated by the OBC designer (RID RF-21)\nTX_PERMIT_HW = PA_5V present (U804 Schmitt); a second, "
         "independent divider enables U805, the switch that powers the LMX2571", fontsize=7)

mux = sch.ic("U453 mute (XPDR_KEY)\nU454 2:1 select (TNC_PTT)", left=["RPT", "TNC"], right=["OUT"])
d += mux.at((28.0, ym)).anchor("RPT")
sch.flag(d, mux.RPT, "RPT_AF_AC", "left", 0.6)
sch.flag(d, mux.TNC, "TNC_AF", "left", 0.6)
sch.flag(d, mux.OUT, "TX_AUDIO", "right", 0.6)

# ---------------------------------------------------------------- transmit chain (bottom row)
yt = ym - 15.0
adc = sch.ic("U501 PCM1808\n512 fs, I2S\nFS = +-6.3 kHz", left=["VINL"], right=["I2S"])
d += adc.at((0, yt)).anchor("VINL")
sch.flag(d, adc.VINL, "TX_AUDIO", "left", 0.6)
lmx = sch.ic("U502 LMX2571\n145.9 MHz, FSK I2S (analog FM)\nCE = PTT_EXC", left=["I2S"], right=["RFOUT"], top=["CE"])
d += lmx.at((adc.I2S[0] + 1.4, adc.I2S[1])).anchor("I2S")
d += elm.Line().at(adc.I2S).to(lmx.I2S)
pa = sch.ic("Q601 RD01MUS3 (5 V)\n2-stage, APC on gate bias\n0.5 W / 1 W", left=["IN"], right=["OUT"], top=["VDD"])
d += pa.at((lmx.RFOUT[0] + 1.4, lmx.RFOUT[1])).anchor("IN")
d += elm.Line().at(lmx.RFOUT).to(pa.IN)
sch.flag(d, pa.VDD, "PA_5V (SW3: RF_TX_EN)", "up", 0.5)
lpf = sch.ic("match + elliptic LPF\n2f >= 65.5 dB, 3f >= 88.2 dB\n(worst of 3000, LPF alone)", left=["IN"], right=["OUT"])
d += lpf.at((pa.OUT[0] + 1.4, pa.OUT[1])).anchor("IN")
d += elm.Line().at(pa.OUT).to(lpf.IN)
cpl = sch.ic("CPL601 sense line\nU701/U711 AD8361", left=["IN"], right=["OUT"], bottom=["FWD", "REV"])
d += cpl.at((lpf.OUT[0] + 1.4, lpf.OUT[1])).anchor("IN")
d += elm.Line().at(lpf.OUT).to(cpl.IN)
sch.flag(d, cpl.FWD, "TX_FWD", "down", 0.6)
sch.flag(d, cpl.REV, "TX_REV", "down", 0.6)
balv = sch.ic("L610/C621/C622/L611\nlattice balun (J3, J4)", left=["SE"], right=["A", "B"])
d += balv.at((cpl.OUT[0] + 1.4, cpl.OUT[1])).anchor("SE")
d += elm.Line().at(cpl.OUT).to(balv.SE)
d += elm.Antenna().at((balv.A[0] + 1.0, balv.A[1] + 0.6)).label("2 m pair\n+X / -X (AntS)", loc="right", fontsize=7)
sch.note(d, (0, yt - 2.6), "PA domain (unpowered unless SW3/RF_TX_EN): PCM1808, squarer, detectors, APC, PA. "
         "LMX2571 on a switch enabled by PA_5V presence. TX_TEMP: TMP235 on the PA pad (always powered).\n"
         "TX-OFF removes the PA supply and, through TX_PERMIT_HW, the exciter's power and CE: two hardware paths.",
         fontsize=7)

d.save(sch.out_path("S4_rf_board"))
