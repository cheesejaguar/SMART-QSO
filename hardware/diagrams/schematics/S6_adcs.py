"""S6 ADCS board: magnetorquer drivers, magnetometer, gyro, sun-sensor amplifiers and ADC."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S6_adcs")
sch.title(d, (-4, 9), "S6", "ADCS board",
          "Sources: hardware/adcs/ADCS_ELECTRICAL_DESIGN.md (LIS3MDL, LSM6DSO, DRV8833), SENSOR_DESIGN.md, "
          "ACTUATOR_DESIGN.md; docs/hardware/ADCS_DESIGN.md ADCS-I001/I002; W6")

# ---- magnetorquer drivers
drv = []
for k, (ref, a, b) in enumerate([("U1", "X", "Y"), ("U2", "Z", None)]):
    u = sch.ic(f"{ref} DRV8833\ndual H-bridge", left=list(reversed(["AIN1", "AIN2", "BIN1", "BIN2", "nSLEEP", "nFAULT"])),
               right=list(reversed(["AOUT1", "AOUT2", "BOUT1", "BOUT2"])), top=["VM"], bottom=["AISEN", "BISEN", "GND"],
               size=(4.6, 5.0), spread=("left", "right", "bottom"))
    d += u.at((0, -k * 9.5))
    for pin, net in (("AIN1", f"MTQ_{a}_IN1"), ("AIN2", f"MTQ_{a}_IN2")) + (
            (("BIN1", f"MTQ_{b}_IN1"), ("BIN2", f"MTQ_{b}_IN2")) if b else ()):
        sch.flag(d, sch.pin(u, pin), net + "*", "left", 0.6)
    if not b:
        for pin in ("BIN1", "BIN2"):
            d += elm.Line().at(sch.pin(u, pin)).left().length(0.6)
            d += elm.Label().label("GND", loc="left", fontsize=7)
    sch.flag(d, u.nSLEEP, "MTQ_nSLEEP*", "left", 0.6)
    sch.flag(d, u.nFAULT, "MTQ_nFAULT*", "left", 0.6)
    d += elm.Line().at(u.VM).up().length(0.5)
    d += elm.Vdd().label("3V3_ADCS (VM, TBR)", fontsize=7)
    for pin in ("AISEN", "BISEN", "GND"):
        d += elm.Line().at(sch.pin(u, pin)).down().length(0.4)
        d += elm.Ground()
    for axis, p1, p2 in ((a, "AOUT1", "AOUT2"), (b, "BOUT1", "BOUT2")):
        if not axis:
            for p_ in (p1, p2):
                d += elm.Line().at(sch.pin(u, p_)).right().length(0.4)
                d += elm.Label().label("n.c.", loc="right", fontsize=7)
            continue
        d += elm.Line().at(sch.pin(u, p1)).right().length(1.0)
        d += elm.Inductor2(loops=4).down().toy(sch.pin(u, p2)[1]).label(f"L{axis} coil\n250 turns\n(J_MTQ {axis})",
                                                                         loc="bottom", fontsize=7)
        d += elm.Line().left().tox(sch.pin(u, p2)[0])
    drv.append(u)
sch.note(d, (-4.5, -15.5), "* proposed: no timer pins or nSLEEP/nFAULT pins in the firmware (finding F3).\n"
         "nFAULT open-drain, 10k pull-up to 3V3_ADCS. AISEN/BISEN grounded: no current\n"
         "sense (TBR). Coil current about 100 mA (ADCS-I002).", fontsize=7)

# ---- magnetometer and gyro on I2C4
xm = 14
mag = sch.ic("U3 LIS3MDL\nmagnetometer\nI2C4 0x1E", left=list(reversed(["SCL", "SDA", "SDO/SA1", "CS"])),
             right=list(reversed(["VDD", "VDD_IO", "C1", "GND"])), size=(4.6, 3.4), spread=("left", "right"))
d += mag.at((xm, 0))
gyr = sch.ic("U4 LSM6DSO\ngyro / accelerometer\nI2C4 0x6A", left=list(reversed(["SCL", "SDA", "SDO/SA0", "CS"])),
             right=list(reversed(["VDD", "VDDIO", "INT1", "GND"])), size=(4.6, 3.4), spread=("left", "right"))
d += gyr.at((xm, -6))
for u, sa, add in ((mag, "SDO/SA1", "3V3_ADCS"), (gyr, "SDO/SA0", None)):
    sch.flag(d, u.SCL, "I2C4_SCL", "left", 0.6)
    sch.flag(d, u.SDA, "I2C4_SDA", "left", 0.6)
    sch.flag(d, u.CS, "3V3_ADCS", "left", 0.6)
    if add:
        sch.flag(d, sch.pin(u, sa), add, "left", 0.6)
    else:
        d += elm.Line().at(sch.pin(u, sa)).left().length(0.6)
        d += elm.Ground()
    sch.flag(d, u.VDD, "3V3_ADCS", "right", 0.6)
    d += elm.Line().at(u.GND).right().length(0.6)
    d += elm.Ground()
sch.flag(d, mag.VDD_IO, "3V3_ADCS", "right", 0.6)
sch.flag(d, gyr.VDDIO, "3V3_ADCS", "right", 0.6)
d += elm.Capacitor().at(mag.C1).right().length(1.2).label("100 nF", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Line().at(gyr.INT1).right().length(0.6)
d += elm.Label().label("n.c. (TBR)", loc="right", fontsize=7)
sch.note(d, (xm - 2, -9.8), "SA1 high: LIS3MDL at 0x1E (ADCS-I001); SA0 low: LSM6DSO at 0x6A.\n"
         "CS high selects I2C on both. 100 nF at each supply pin (TBR).", fontsize=7)

# ---- sun sensors: one TIA channel of six, and the ADC
ys = -15
d += (pd := elm.Photodiode().at((xm - 1, ys)).up().reverse().label("D1-D6 photodiode\n(one per face, TBD)", loc="left", fontsize=7))
d += elm.Ground().at(pd.start)
op = elm.Opamp(leads=True).at((xm + 1.5, ys + 2.0)).anchor("in1").right().label("U5 OPA333\n(x6, TBR)", loc="bottom", fontsize=7)
d += op
d += elm.Wire("|-").at(pd.end).to(op.in1)
d += elm.Line().at(op.in2).left().length(0.6)
d += elm.Ground()
d += elm.Line().at(op.in1).up().length(1.4)
d += (fb := elm.Resistor().right().tox(op.out[0]).label("Rf 100k (TBR)", loc="top", fontsize=7))
d += elm.Line().down().toy(op.out[1])
d += elm.Capacitor().at((op.in1[0], op.in1[1] + 1.4)).up().length(1.0)
d += elm.Line().right().tox(op.out[0])
d += elm.Line().down().length(1.0)
d += elm.Label().at((op.in1[0] + 1.6, op.in1[1] + 2.7)).label("Cf 10 pF (TBR)", fontsize=7)
d += elm.Line().at(op.out).right().length(1.0)
d += (ain := elm.Dot())
adc = sch.ic("U6 8-ch I2C ADC (TBD)\ne.g. ADS7828, I2C4 0x4B\n(A1 = A0 = 1)", left=list(reversed(["CH0", "CH1-CH5", "CH6-CH7"])),
             right=list(reversed(["SCL", "SDA", "A0", "A1"])), size=(5.6, 3.0), tbd=True, spread=("left", "right"))
d += adc.at((ain.center[0] + 1.2, ain.center[1])).anchor("CH0")
d += elm.Line().at(ain.center).to(adc.CH0)
d += elm.Line().at(sch.pin(adc, "CH1-CH5")).left().length(0.6)
d += elm.Label().label("D2-D6 TIAs", loc="left", fontsize=7)
d += elm.Line().at(sch.pin(adc, "CH6-CH7")).left().length(0.6)
d += elm.Ground()
sch.flag(d, adc.SCL, "I2C4_SCL", "right", 0.6)
sch.flag(d, adc.SDA, "I2C4_SDA", "right", 0.6)
sch.flag(d, adc.A0, "3V3_ADCS", "right", 0.6)
sch.flag(d, adc.A1, "3V3_ADCS", "right", 0.6)
sch.note(d, (xm - 2, ys - 2.4), "Photovoltaic mode (zero bias), Vout = Isc x Rf: 10 uA full sun gives 1 V (TBR).\n"
         "ADCS-I001 lists 0x40-0x45 for sun sensors, which suggests digital sensors; this analog\n"
         "chain is a proposal (finding F3). OPA333 single supply, 3V3_ADCS.", fontsize=7)

d.save(sch.out_path("S6_adcs"))
