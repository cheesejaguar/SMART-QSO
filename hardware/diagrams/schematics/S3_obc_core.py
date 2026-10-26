"""S3 OBC core: STM32L4R5ZI with supply decoupling, LSE, reset and boot, SWD/access port, FRAM, TMP117,
I2C pull-ups, status LED and sense pull-ups. Every other MCU pin leaves the sheet as a flag.

Pins from software/rust/firmware/src (gpio.rs, main.rs, adc.rs, uart.rs, i2c.rs, spi.rs, fram.rs): TBR.
Pins marked * are proposed here (findings F2, F3), not in the firmware.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S3_obc_core")
TITLE = ("S3", "OBC core: STM32L4R5ZI",
          "Sources: software/rust/firmware/src (pins, TBR); hardware/obc/OBC_DESIGN.md, STORAGE_DESIGN.md; "
          "STM32L4R5 datasheet (supply, LSE). * = proposed here, not in firmware")


# right-hand pins, top to bottom: (pin, net)
RIGHT = [
    ("PA4", "FRAM_CS#"), ("PA5", "SPI1_SCK"), ("PA6", "SPI1_MISO"), ("PA7", "SPI1_MOSI"),
    ("PE0", "FRAM_WP#"), ("PE1", "FRAM_HOLD#"), ("PD3", "LED_STATUS"),
    ("PF1", "I2C2_SCL"), ("PF0", "I2C2_SDA"), ("PD12", "I2C4_SCL"), ("PD13", "I2C4_SDA"),
    ("PA9", "USART1_TX"), ("PA10", "USART1_RX"), ("PD5", "USART2_TX"), ("PD6", "USART2_RX"),
    ("PD8", "USART3_TX"), ("PD9", "USART3_RX"), ("PC10", "UART4_TX*"), ("PC11", "UART4_RX*"),
    ("PB4", "PAYLOAD_EN"), ("PB5", "RADIO_EN"), ("PB6", "ADCS_EN"), ("PD2", "HEATER_EN"),
    ("PA1", "DEPLOY_EN"), ("PA2", "JETSON_PWR"), ("PA3", "RF_TX_EN"), ("PE3", "TX_PWR_SEL"),
    ("PE4", "GNSS_EN*"), ("PB9", "ANTENNA_BURN"), ("PB10", "SOLAR_BURN"),
    ("PB7", "INA_ALERT#"), ("PE2", "DEPLOY_SENSE"), ("PB8", "UMBILICAL_SENSE"),
    ("PB11", "ANTENNA_SENSE"), ("PB12", "SOLAR_SENSE"),
    ("PC2", "VBATT_SNS"), ("PC3", "IBATT_SNS"), ("PC4", "VSOLAR_SNS"), ("PC5", "ISOLAR_SNS"),
    ("PB0", "VBUS_SNS"), ("PB1", "TEMP_BOARD"), ("PA0", "TX_TEMP"), ("PC0", "TX_FWD"), ("PC1", "TX_REV"),
    ("(TBD)", "MTQ PWM x6*, PPS*"),
]
LEFT = ["VDD x9", "VDDIO2", "VDDUSB", "VBAT", "VDDA", "VREF+", "NRST", "PH3-BOOT0",
        "PC14-OSC32_IN", "PC15-OSC32_OUT", "PA13 SWDIO", "PA14 SWCLK", "PB3 SWO"]

PS = 0.6
mcu = sch.ic("U1\nSTM32L4R5ZIT6\nLQFP144\n\n2 MB flash\n640 KB SRAM\nIWDG on LSI\nRTC on LSE",
             left=list(reversed(LEFT)), right=[p for p, _ in reversed(RIGHT)], bottom=["VSS x10", "VSSA"],
             pinspacing=PS, size=(7.0, PS * (len(RIGHT) + 1)), spread=("left", "bottom"))
d += mcu.at((0, 0))
TOP = mcu.PA4[1]
sch.title(d, (-16, TOP + 4.5), *TITLE)
for pin, net in RIGHT:
    sch.flag(d, getattr(mcu, sch.anchor(pin)), net, "right", 0.6)
d += elm.Ground().at(mcu.VSS)
d += elm.Ground().at(mcu.VSSA)

# ---- supplies
for pin in ("VDD", "VDDIO2", "VDDUSB", "VBAT"):
    sch.flag(d, getattr(mcu, pin), "3V3_AO", "left", 0.8)
d += elm.Line().at(mcu.VDDA).left().length(0.8)
d += (va := elm.Dot())
d += elm.Line().at(mcu.VREFP).left().tox(va.center[0])
d += elm.Line().at(va.center).toy(mcu.VREFP[1])
d += elm.Inductor2(loops=2).at(va.center).left().length(1.6).label("FB1 ferrite\n600 Ohm @100 MHz", loc="top", fontsize=7)
d += (fbn := elm.Dot())
sch.flag(d, fbn.center, "3V3_AO", "left", 0.6)
d += elm.Capacitor().at(va.center).down().length(1.0).label("1 uF + 10 nF", loc="left", fontsize=7)
d += elm.Ground()

# decoupling bank
x0, y0 = -15.5, TOP + 0.5
d += elm.Line().at((x0, y0)).right().length(9.5)
sch.flag(d, (x0, y0), "3V3_AO", "left", 0.4)
for i, lbl in enumerate(["100 nF"] * 9 + ["4.7 uF"]):
    d += elm.Dot().at((x0 + 0.5 + i * 0.95, y0))
    d += elm.Capacitor().at((x0 + 0.5 + i * 0.95, y0)).down().length(1.2)
    d += elm.Ground()
sch.note(d, (x0, y0 + 0.9), "C1-C9 100 nF, one at each VDD pin; C10 4.7 uF bulk; VDDIO2 and VDDUSB each 100 nF (TBR).\n"
         "VBAT tied to 3V3_AO: no backup cell, the RTC runs from the always-on rail (TBR).", fontsize=7)

# reset and boot
d += elm.Line().at(mcu.NRST).left().length(1.6)
d += (nr := elm.Dot())
d += elm.Capacitor().at(nr.center).down().length(1.0).label("100 nF", loc="left", fontsize=7)
d += elm.Ground()
sch.flag(d, nr.center, "NRST", "left", 0.8)
d += elm.Line().at(mcu.PH3_BOOT0).left().length(1.0)
d += elm.Resistor().down().length(1.2).label("10k", loc="left", fontsize=7)
d += elm.Ground()

# LSE
d += elm.Line().at(mcu.PC14_OSC32_IN).left().length(2.6)
d += (xa := elm.Dot())
d += elm.Line().at(mcu.PC15_OSC32_OUT).left().length(2.6)
d += (xb := elm.Dot())
d += elm.Crystal().at(xa.center).to(xb.center).label("Y1 32.768 kHz", loc="bottom", fontsize=7)
d += elm.Capacitor().at(xa.center).left().length(1.4).label("6.8 pF", loc="top", fontsize=6)
d += (ga := elm.Dot())
d += elm.Capacitor().at(xb.center).left().length(1.4).label("6.8 pF (TBR)", loc="bottom", fontsize=6)
d += (gb := elm.Dot())
d += elm.Line().at(ga.center).to(gb.center)
d += elm.Line().at(gb.center).down().length(0.5)
d += elm.Ground()

# SWD
sch.flag(d, mcu.PA13, "SWDIO", "left", 0.8)
sch.flag(d, mcu.PA14, "SWCLK", "left", 0.8)
d += elm.Line().at(mcu.PB3).left().length(0.8)
d += elm.Label().label("SWO n.c.", loc="left", fontsize=7)

# ---- access port J1 (W8)
yj = mcu.PB3[1] - 4.5
acc = sch.ic("J1 access port\n2.54 mm 2x5 (W8)",
             left=list(reversed(["1 VTREF", "2 SWDIO", "3 GND", "4 SWCLK", "5 GND"])),
             right=list(reversed(["6 NRST", "7 USART3_TX", "8 USART3_RX", "9 UMBILICAL_SENSE", "10 GND"])))
d += acc.at((-12, yj))
for a, net, side in (("1", "3V3_AO", "left"), ("2", "SWDIO", "left"), ("4", "SWCLK", "left"),
                     ("6", "NRST", "right"), ("7", "USART3_TX", "right"), ("8", "USART3_RX", "right"),
                     ("9", "UMBILICAL_SENSE", "right")):
    sch.flag(d, sch.pin(acc, a), net, side, 0.6)
for a in ("3", "5", "10"):
    d += elm.Line().at(sch.pin(acc, a)).down().length(0.4) if a != "10" else elm.Line().at(sch.pin(acc, a)).right().length(0.4)
    d += elm.Ground()
sch.note(d, (-14.5, yj - 1.0), "J1 pins 9-10 are jumpered in the GSE plug:\nUMBILICAL_SENSE low = umbilical present", fontsize=7)

# ---- peripherals on the OBC board, to the right of the flags
xp = 22
fram = sch.ic("U2 FM25V20A\n2 Mbit SPI FRAM",
              left=list(reversed(["CS#", "SCK", "SI", "SO"])), right=list(reversed(["VDD", "WP#", "HOLD#", "VSS"])))
d += fram.at((xp, TOP - 4))
for a, net in (("CS", "FRAM_CS#"), ("SCK", "SPI1_SCK"), ("SI", "SPI1_MOSI"), ("SO", "SPI1_MISO")):
    sch.flag(d, getattr(fram, a), net, "left", 0.6)
sch.flag(d, fram.VDD, "3V3_AO", "right", 0.6)
sch.flag(d, fram.WP, "FRAM_WP#", "right", 0.6)
sch.flag(d, fram.HOLD, "FRAM_HOLD#", "right", 0.6)
d += elm.Line().at(fram.VSS).right().length(0.6)
d += elm.Ground()
sch.note(d, (fram.VSS[0] - 6, fram.VSS[1] - 0.8), "100 nF at VDD; 10k pull-up on CS# (TBR)", fontsize=7)

tmp = sch.ic("U3 TMP117\nI2C2 0x48 (ADD0 = GND)", left=list(reversed(["SCL", "SDA", "ALERT"])),
             right=list(reversed(["V+", "ADD0", "GND"])))
d += tmp.at((xp, TOP - 11))
sch.flag(d, tmp.SCL, "I2C2_SCL", "left", 0.6)
sch.flag(d, tmp.SDA, "I2C2_SDA", "left", 0.6)
d += elm.Line().at(tmp.ALERT).left().length(0.6)
d += elm.Label().label("n.c.", loc="left", fontsize=7)
sch.flag(d, tmp.VP, "3V3_AO", "right", 0.6)
for a in ("ADD0", "GND"):
    d += elm.Line().at(getattr(tmp, a)).right().length(0.6)
    d += elm.Ground()
sch.note(d, (tmp.GND[0] - 7, tmp.GND[1] - 1.2), "0x48 is also the EPS controller's address in the\nsimulation and SMART-QSO-ICD-001 (finding F10)",
         fontsize=7)

# pull-ups: I2C, open-drain sense lines
yp = TOP - 17
PULLS = [("I2C2_SCL", "2.2k"), ("I2C2_SDA", "2.2k"), ("I2C4_SCL", "2.2k"), ("I2C4_SDA", "2.2k"),
         ("INA_ALERT#", "10k"), ("UMBILICAL_SENSE", "10k"), ("ANTENNA_SENSE", "10k"), ("SOLAR_SENSE", "10k")]
d += elm.Line().at((xp - 1, yp)).right().length(len(PULLS) * 1.6)
sch.flag(d, (xp - 1, yp), "3V3_AO", "left", 0.4)
for i, (net, val) in enumerate(PULLS):
    x = xp + i * 1.6
    d += elm.Dot().at((x, yp))
    d += elm.Resistor().at((x, yp)).down().length(1.6).label(val, loc="bottom", fontsize=6)
    d += elm.Tag(width=2.4, height=0.4).down().label(net, fontsize=6, rotate=90)
sch.note(d, (xp - 1, yp + 1.2), "Pull-ups to 3V3_AO (TBR). ANTENNA/SOLAR_SENSE also get the RC filter of S5.",
         fontsize=7)

# LED
d += elm.Line().at((xp, yp - 6.5)).right().length(0.6)
sch.flag(d, (xp, yp - 6.5), "LED_STATUS", "left", 0.4)
d += elm.Resistor().right().length(1.6).label("1k", loc="top", fontsize=7)
d += elm.LED().right().label("D1 status\n(disable in flight, TBR)", loc="bottom", fontsize=7)
d += elm.Line().right().length(0.4)
d += elm.Ground()

d.save(sch.out_path("S3_obc_core"))
