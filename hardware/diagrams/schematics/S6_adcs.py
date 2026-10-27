"""S6 ADCS on the OBC board, version 4 (2026-10-27; SQSO-ICX-001 D: the magnetometer on its own switch U35 and repeater
U34, MAG_EN PF14 / MAG_RB PF15, rail 3V3_MAG; version 3: magnetometer on board A2 at +Z): sensor rails, gyro, backup gyro, magnetometer, one of the six
sun-sensor channels, and the three magnetorquer H-bridges with the burst gate and the hardware off-timer. Drawn from
the build-to design SQSO-OBC-BRD-001 sections 9-10 (hardware/electrical/obc/design.py). Version 1 drew a separate
ADCS board with DRV8833 drivers, an LIS3MDL and an LSM6DSO; none of that is in the design.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S6_adcs")
sch.note(d, (-6, 13), "S6  ADCS on the OBC board and the magnetometer board (version 4)", fontsize=12, color="black")
sch.note(d, (-6, 12.4), "SQSO-HW-DIAG-001 · from the build-to netlist SQSO-OBC-BRD-001 sections 9-10 · pins: SQSO-ICX-001 "
         "D section 4 · each sensor's bus crosses into its rail through a VCC-isolating translator", fontsize=8)

# ---- sensor rail switches
for k, (ref, en, rail, rb, what) in enumerate((("U5", "GYRO_EN (PF6)", "3V3_GYRO", "GYRO_RB (PF10)", "M-G370"),
                                                ("U6", "IMU2_EN (PF7)", "3V3_IMU2", "IMU2_RB (PF11)", "BMI088"),
                                                ("U7", "SNS_EN (PF8)", "3V3_SNS", "SNS_RB (PF12)", "CSS ADC, U14"),
                                                ("U35", "MAG_EN (PF14)", "3V3_MAG", "MAG_RB (PF15)", "RM3100, U34, U33"))):
    u = sch.ic(f"{ref} TPS2553-Q1\nILIM = IN: 50-100 mA", left=list(reversed(["IN", "EN", "FAULT"])), right=["OUT"],
               bottom=["GND"], size=(4.6, 2.2), spread=("left",))
    y = 8 - 4.2 * k
    d += u.at((0, y))
    sch.flag(d, u.IN, "3V3_ADCS (J6, EPS SW4)", "left", 0.6)
    sch.flag(d, u.EN, f"{en} via 100k, 2.2 uF retry", "left", 0.6)
    sch.flag(d, u.FAULT, f"EN node -> {rb}", "left", 0.6)
    sch.flag(d, u.OUT, f"{rail} ({what})", "right", 0.6)
    d += elm.Ground().at(u.GND)
sch.note(d, (-2, -7.6), "Trip in 2 us, FAULT after 8 ms pulls EN low: rail off 0.13 s, then retry (latch-up, RADIATION_ANALYSIS 5.3).",
         fontsize=7)

# ---- gyro, backup gyro, magnetometer
xg = 30
g = sch.ic("U8 Epson M-G370PDT0 (socket X1)\nSPI mode 3, <= 1 MHz burst", left=list(reversed(["SCLK", "SDI", "/CS", "SDO", "/RST", "DRDY"])),
           right=list(reversed(["VCC x3", "SIN", "GND x4"])), size=(6.2, 4.2), spread=("left", "right"))
d += g.at((xg, 7))
tx = sch.ic("U9 TXU0304-Q1 + U10 TXU0204-Q1\nVCCA 3V3_OBC / VCCB = OE = 3V3_GYRO", left=list(reversed(["PC10", "PC12", "PD1", "PC11", "PD14", "PD10"])),
            right=list(reversed(["B1", "B2", "B3", "B4", "B5", "B6"])), size=(6.6, 4.2), spread=("left", "right"))
d += tx.at((xg - 14, 7))
for a, b in (("B1", "SCLK"), ("B2", "SDI"), ("B3", "CS"), ("B4", "SDO"), ("B5", "RST"), ("B6", "DRDY")):
    d += elm.Line().at(getattr(tx, a)).to(getattr(g, b))
for a, net in (("PC10", "SPI3_SCK"), ("PC12", "SPI3_MOSI"), ("PD1", "GYRO_CS_N"), ("PC11", "SPI3_MISO"),
               ("PD14", "GYRO_RST_N"), ("PD10", "GYRO_DRDY")):
    sch.flag(d, getattr(tx, a), net, "left", 0.5)
sch.flag(d, g.VCC, "3V3_GYRO", "right", 0.6)
sch.flag(d, g.SIN, "10k to 3V3_GYRO", "right", 0.6)
d += elm.Line().at(g.GND).right().length(0.6)
d += elm.Ground()

b = sch.ic("U11 Bosch BMI088 (cold spare)\nPS = GND: gyro SPI; SDO1 + SDO2 joined", left=list(reversed(["SCK", "SDI", "CSB2", "CSB1", "SDO", "INT3"])),
           right=list(reversed(["VDD, VDDIO", "GND"])), size=(6.6, 3.8), spread=("left", "right"))
d += b.at((xg, -1))
sch.flag(d, b.SCK, "via U12/U13 (VCCB 3V3_IMU2): PB13, PB15, PD0, PD4, PB14, PD15", "left", 0.6)
sch.flag(d, b.VDD, "3V3_IMU2", "right", 0.6)
d += elm.Line().at(b.GND).right().length(0.6)
d += elm.Ground()

m = sch.ic("Magnetometer board A2 (+Z end frame): U1 PNI RM3100 (13156)\nI2C 0x20 via OBC J10 / W6E, single-shot in the coils-off window",
           left=list(reversed(["SDA", "SCL", "DRDY", "SA0, SA1"])), right=list(reversed(["XINP/XINN", "YINP/YINN", "ZINP/ZINN", "REXT", "AVDD, DVDD"])),
           size=(6.8, 3.8), spread=("left", "right"))
d += m.at((xg, -8.5))
sch.flag(d, m.SDA, "SDA_M / SCL_M via U34 TCA9517 (I2C4 PD12/PD13)", "left", 0.6)
sch.flag(d, m.DRDY, "via U33 LVC1T45 (B side 3V3_MAG) -> MAG_DRDY (PD11)", "left", 0.6)
d += elm.Line().at(m.SA0).left().length(0.6)
d += elm.Ground()
for a, nm in (("XINP/XINN", "L1 Sen-XY-f"), ("YINP/YINN", "L2 Sen-XY-f"), ("ZINP/ZINN", "L3 Sen-Z-f")):
    d += elm.Line().at(sch.pin(m, a)).right().length(0.6)
    d += elm.Label().label(f"{nm}, 2 x 121 Ohm bias", loc="right", fontsize=7)
d += elm.Resistor().at(m.REXT).right().length(1.4).label("33k", loc="top", fontsize=7)
d += elm.Ground()
sch.flag(d, m.AVDD, "3V3_MAG (U35; 10 uF + 100 nF)", "right", 0.6)

# ---- one sun-sensor channel and the ADC
ys = -16
d += (pd := elm.Photodiode().at((-2, ys)).up().reverse().label("panel photodiode\n(x6, J8, Isc <= 4 mA)", loc="left", fontsize=7))
d += elm.Ground().at(pd.start)
op = elm.Opamp(leads=True).at((0.5, ys + 2.0)).anchor("in1").right().label("U17-U19 TLV9002-Q1\n(6 channels)", loc="bottom", fontsize=7)
d += op
d += elm.Wire("|-").at(pd.end).to(op.in1)
d += elm.Line().at(op.in2).left().length(0.6)
d += elm.Ground()
d += elm.Line().at(op.in1).up().length(1.4)
d += elm.Resistor().right().tox(op.out[0]).label("Rf 619 Ohm 0.1 %, Cf 10 nF", loc="top", fontsize=7)
d += elm.Line().down().toy(op.out[1])
d += elm.Line().at(op.out).right().length(0.6)
d += elm.Resistor().right().length(1.4).label("1k", loc="top", fontsize=7)
d += (ain := elm.Dot())
d += elm.Capacitor().at(ain.center).down().length(1.0).label("100 nF", loc="left", fontsize=7)
d += elm.Ground()
adc = sch.ic("U16 ADS7138-Q1\n12-bit, I2C 0x10 (SNS side)", left=list(reversed(["AIN0-5 sun", "AIN6 temp", "AIN7 VBAT_MTQ/6"])),
             right=list(reversed(["SCL", "SDA", "AVDD"])), size=(5.4, 2.6), spread=("left", "right"))
d += adc.at((ain.center[0] + 1.4, ain.center[1])).anchor("AIN0_5")
d += elm.Line().at(ain.center).to(sch.pin(adc, "AIN0-5 sun"))
sch.flag(d, adc.SCL, "SCL_S (U14 TCA9517, I2C4)", "right", 0.6)
sch.flag(d, adc.SDA, "SDA_S", "right", 0.6)
sch.flag(d, adc.AVDD, "3V3_SNS", "right", 0.6)
sch.note(d, (-6, ys - 3.6), "Vout = Isc x 619 Ohm: 2.48 V at 4.0 mA; clips at 5.25 mA; 1.3 uA per LSB (1.5 deg at normal incidence).",
         fontsize=7)

# ---- magnetorquer drive
ym = -27
drv = sch.ic("U22/U23/U24 DRV8876-Q1 (X, Y, Z)\nPH/EN mode, ITRIP 104-126 mA", left=list(reversed(["EN", "PH", "nSLEEP", "VREF", "IPROPI", "nFAULT"])),
             right=list(reversed(["VM", "OUT1", "OUT2"])), size=(6.6, 4.2), spread=("left", "right"))
d += drv.at((10, ym))
sch.flag(d, drv.EN, "MTQ_x_PWM: TIM1 CH1-3 (PE9/PE11/PE13), 40 kHz", "left", 0.6)
sch.flag(d, drv.PH, "MTQ_x_DIR (PE8/PE10/PE12)", "left", 0.6)
sch.flag(d, drv.VREF, "1.150 V (18.7k/10.0k from 3V3_OBC)", "left", 0.6)
sch.flag(d, drv.IPROPI, "10.0k: 10 mV/mA -> MTQ_x_I (PA4/PA5/PA6)", "left", 0.6)
sch.flag(d, drv.nFAULT, "MTQ_FAULT_N (PE14, 10k up)", "left", 0.6)
sch.flag(d, drv.VM, "VBAT_MTQ (J1-36/39, EPS SW11; D3 SMBJ20A)", "right", 0.6)
d += elm.Line().at(drv.OUT1).right().length(1.0)
d += elm.Inductor2(loops=4).down().toy(drv.OUT2[1]).label("CR0002 rod\n(J7, 87 mA at 0.2 A.m2)", loc="bottom", fontsize=7)
d += elm.Line().left().tox(drv.OUT2[0])
gate = sch.ic("U27 AND (LVC1G08-Q1)\nnSLEEP = ARM . NOT expired", left=list(reversed(["A", "B"])), right=["Y"], size=(5.0, 1.8))
d += gate.at((-6, ym - 7))
d += elm.Wire("-|").at(gate.Y).to(drv.nSLEEP)
sch.flag(d, gate.A, "MTQ_ARM: TIM15_CH1 (PF9), one-pulse 900 ms, 100k down", "left", 0.6)
tmr = sch.ic("U25 CD74HC4060 RC counter\n539 Hz, Q10 after 512 periods\n= 0.92-0.98 s, latched by D4",
             left=list(reversed(["MR", "RS/RT/CT"])), right=["Q10"], size=(6.0, 2.4))
d += tmr.at((-6, ym - 11.5))
sch.flag(d, tmr.MR, "NOT MTQ_ARM (U26)", "left", 0.6)
d += elm.Line().at(sch.pin(tmr, "RS/RT/CT")).left().length(0.6)
d += elm.Label().label("Rt 80.6k, Ct 10 nF C0G, Rs 162k", loc="left", fontsize=7)
d += elm.Line().at(tmr.Q10).right().length(1.2)
d += (q := elm.Dot())
d += elm.Wire("|-").at(q.center).to(gate.B).label("via U26 inverter", loc="bottom", fontsize=7)
sch.flag(d, q.center, "MTQ_EXPIRED (PE15)", "right", 2.0)
sch.note(d, (-6, ym - 14.6), "Burst: TIM15 ends it at 900 ms by itself; nSLEEP low -> bridges high-Z -> coil current zero within 1.9 ms.\n"
         "A hung OBC, a stopped clock or an upset timer register cannot hold a coil on past 0.98 s (U25); re-armed only when MTQ_ARM falls.",
         fontsize=7)

d.save(sch.out_path("S6_adcs"))
