"""S3 OBC core, version 3 (2026-10-27; JETSON_PWR on PA8, spreader NTC on PA2, RX_CD PG0, RX_FAULT PF13): STM32L4R5ZIT6 with its supply, clocks, supervisor, FRAM, mission clock and
access port, drawn from the build-to design SQSO-OBC-BRD-001 (hardware/electrical/obc/design.py, the netlist source).
Every MCU pin is the master allocation of SQSO-ICX-001 (hardware/electrical/INTERCONNECT.md 4); the sheet groups the
signal pins by destination. Version 1 drew the firmware's provisional map with a TMP117 and proposed UART4 pins.
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S3_obc_core")
sch.note(d, (-17, 33.5), "S3  OBC core: STM32L4R5ZIT6 (version 3)", fontsize=12, color="black")
sch.note(d, (-17, 32.9), "SQSO-HW-DIAG-001 · drawn from the build-to netlist SQSO-OBC-BRD-001 (hardware/electrical/obc) · "
         "pins: SQSO-ICX-001 section 4 · flags name the nets of W3, W4, W5, W8, W9 and sheets S6-S9", fontsize=8)

# right-hand pins, top to bottom: (pin, net)
RIGHT = [
    ("PB2", "PAYLOAD_EN"), ("PB5", "RADIO_EN"), ("PB6", "ADCS_EN"), ("PD2", "HEATER_EN"), ("PF3", "DEPLOY_EN"),
    ("PA3", "RF_TX_EN"), ("PA8", "JETSON_PWR"), ("PE5", "GNSS_EN"), ("PB10", "BURN_W1A"), ("PE6", "BURN_W1B"),
    ("PB9", "BURN_W2A"), ("PE7", "BURN_W2B"), ("PF4", "EOL_ARM"), ("PG1", "EOL_FIRE"), ("PF5", "LOCK_CLR"),
    ("PB7", "EPS_ALERT_N"), ("PE2", "DEPLOY_SENSE"), ("PB12", "SOLAR_SENSE"), ("PF1", "I2C2_SCL"), ("PF0", "I2C2_SDA"),
    ("PC2", "VBATT_SNS"), ("PC3", "IBATT_SNS"), ("PC4", "VSOLAR_SNS"), ("PC5", "ISOLAR_SNS"), ("PB0", "VBUS_SNS"),
    ("PA1", "VSW1_RB"), ("PA7", "VJET_RB"), ("PA2", "PT_NTC <- J3-5 (S7, S9)"),
    ("PD5", "USART2_TX (TNC)"), ("PD6", "USART2_RX (TNC)"), ("PE3", "TX_PWR_SEL"), ("PE4", "XPDR_EN (TIM7)"),
    ("PC9", "TONE_OK"), ("PG12", "XPDR_KEY"), ("PC7", "PTT_EXC_RB (TIM8_CH2)"), ("PG0", "RX_CD"), ("PF13", "RX_FAULT"), ("PA0", "TX_TEMP"), ("PC0", "TX_FWD"),
    ("PC1", "TX_REV"),
    ("PA9", "USART1_TX -> S7"), ("PA10", "USART1_RX <- S7"), ("PG6", "JET_PG <- S7"),
    ("PG7", "LPUART1_TX -> S8"), ("PG8", "LPUART1_RX <- S8"), ("PC6", "GNSS_PPS (TIM3_CH1) <- S8"), ("PG15", "GNSS_PG <- S8"),
    ("PC10-12, PD1, PD10, PD14", "gyro SPI3 -> S6"), ("PB13-15, PD0, PD4, PD15", "BMI088 SPI2 -> S6"),
    ("PD12, PD13, PD11", "I2C4, MAG_DRDY -> S6"), ("PF6-PF8, PF10-PF12", "sensor rails -> S6"),
    ("PE8-PE15, PF9", "magnetorquers -> S6"), ("PA4-PA6, PB1", "coil currents, TEMP_BOARD -> S9"),
]
LEFT = ["VDD x9", "VDDIO2 x2", "VDDUSB", "VBAT", "VDDA", "VREF+", "NRST", "PH3-BOOT0", "PH0-OSC_IN",
        "PC14-OSC32_IN", "PC15-OSC32_OUT", "PA13 SWDIO", "PA14 SWCLK", "PD8 USART3_TX", "PD9 USART3_RX",
        "PB8 UMB_SENSE", "PG11 WDI", "PG2-5 SPI1 FRAM", "PG13-14 I2C1 RTC", "PC8 RTC_1HZ (TIM3_CH3)"]

PS = 0.6
mcu = sch.ic("U1\nSTM32L4R5ZIT6\nLQFP-144\n\n80 MHz (8 MHz XO, PLL)\n2 MB flash, 640 KB SRAM\nIWDG 30 s on LSI\n"
             "13 spare pins: analog", left=list(reversed(LEFT)), right=[p for p, _ in reversed(RIGHT)],
             bottom=["VSS x11", "VSSA, VREF-"], pinspacing=PS, size=(8.0, PS * (len(RIGHT) + 1)), spread=("left", "bottom"))
d += mcu.at((0, 0))
for p, net in RIGHT:
    sch.flag(d, sch.pin(mcu, p), net, "right", 0.6)
d += elm.Ground().at(mcu.VSS)
d += elm.Ground().at(mcu.VSSA)
TOP = sch.pin(mcu, RIGHT[0][0])[1]

# supplies
for pn in ("VDD", "VDDIO2", "VDDUSB", "VBAT"):
    sch.flag(d, getattr(mcu, pn), "3V3_OBC", "left", 0.8)
d += elm.Line().at(mcu.VDDA).left().length(0.8)
d += (va := elm.Dot())
d += elm.Line().at(mcu.VREFP).left().tox(va.center[0])
d += elm.Line().at(va.center).toy(mcu.VREFP[1])
d += elm.Inductor2(loops=2).at(va.center).left().length(1.6).label("FB1 600 Ohm", loc="top", fontsize=7)
d += (fbn := elm.Dot())
sch.flag(d, fbn.center, "3V3_OBC", "left", 0.6)
d += elm.Capacitor().at(va.center).down().length(1.0).label("2 x (1 uF + 10/100 nF)", loc="left", fontsize=7)
d += elm.Ground()
x0, y0 = -16.5, TOP + 0.5
d += elm.Line().at((x0, y0)).right().length(10.5)
sch.flag(d, (x0, y0), "3V3_OBC (J1-1/2)", "left", 0.4)
for i, lbl in enumerate(["D2 SMF5.0A", "22 uF", "10 uF"] + ["100 nF"] * 8):
    d += elm.Dot().at((x0 + 0.5 + i * 0.95, y0))
    e = elm.Zener() if i == 0 else elm.Capacitor()
    d += e.at((x0 + 0.5 + i * 0.95, y0)).down().length(1.2)
    d += elm.Ground()
sch.note(d, (x0, y0 + 1.0), "D2 TVS, C1 22 uF, C2 10 uF; 100 nF at each of the 9 VDD, 2 VDDIO2, VDDUSB and VBAT pins (13).\n"
         "VBAT tied to VDD: the mission clock is U4 on the always-on rail. Inrush 1.0 ms at SW10's 0.13 A limit.", fontsize=7)

# reset chain and supervisor
d += elm.Line().at(mcu.NRST).left().length(1.6)
d += (nr := elm.Dot())
d += elm.Capacitor().at(nr.center).down().length(1.0).label("100 nF", loc="left", fontsize=7)
d += elm.Ground()
sch.flag(d, nr.center, "NRST", "left", 0.8)
d += elm.Resistor().at(mcu.PH3_BOOT0).left().length(1.6).label("10k", loc="top", fontsize=7)
d += elm.Ground().right()
sch.flag(d, mcu.PH0_OSC_IN, "XO_OUT (Y1 8 MHz, 33 Ohm)", "left", 0.8)
d += elm.Line().at(mcu.PC14_OSC32_IN).left().length(2.6)
d += (xa := elm.Dot())
d += elm.Line().at(mcu.PC15_OSC32_OUT).left().length(2.6)
d += (xb := elm.Dot())
d += elm.Crystal().at(xa.center).to(xb.center).label("Y2 32.768 kHz CM7V", loc="bottom", fontsize=7)
d += elm.Capacitor().at(xa.center).left().length(1.4).label("10 pF", loc="top", fontsize=6)
d += (ga := elm.Dot())
d += elm.Capacitor().at(xb.center).left().length(1.4).label("10 pF", loc="bottom", fontsize=6)
d += (gb := elm.Dot())
d += elm.Line().at(ga.center).to(gb.center)
d += elm.Line().at(gb.center).down().length(0.5)
d += elm.Ground()
for pn, net in (("PA13 SWDIO", "SWDIO"), ("PA14 SWCLK", "SWCLK"), ("PD8 USART3_TX", "USART3_TX"),
                ("PD9 USART3_RX", "USART3_RX"), ("PB8 UMB_SENSE", "UMBILICAL_SENSE"), ("PG11 WDI", "WDI"),
                ("PG2-5 SPI1 FRAM", "FRAM SPI1"), ("PG13-14 I2C1 RTC", "I2C1_SCL/SDA"), ("PC8 RTC_1HZ (TIM3_CH3)", "RTC_1HZ")):
    sch.flag(d, sch.pin(mcu, pn), net, "left", 0.8)

# peripherals to the right of the flags
xp = 27
sup = sch.ic("U2 TPS3850H33-Q1\nUV 3.069 V / OV 3.531 V\nwindow 5.6-33.3 s, reset 200 ms",
             left=list(reversed(["VDD", "SENSE", "WDI", "SET0", "SET1"])), right=list(reversed(["RESET", "WDO", "CWD", "CRST", "GND"])),
             size=(6.4, 3.6), spread=("left", "right"))
d += sup.at((xp, TOP - 3))
for a, net in (("VDD", "3V3_OBC"), ("SENSE", "3V3_OBC"), ("WDI", "WDI"), ("SET0", "WDT_DIS (100k down)")):
    sch.flag(d, getattr(sup, a), net, "left", 0.6)
d += elm.Line().at(sup.SET1).left().length(0.6)
d += elm.Ground()
for a in ("RESET", "WDO"):
    sch.flag(d, getattr(sup, a), "NRST (10k up)", "right", 0.6)
d += elm.Capacitor().at(sup.CWD).right().length(1.6).label("5 x 100 nF C0G", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Line().at(sup.CRST).right().length(0.6)
d += elm.Label().label("open", loc="right", fontsize=7)
d += elm.Line().at(sup.GND).right().length(0.6)
d += elm.Ground()

fram = sch.ic("U3 FM25V20A\n2 Mbit SPI FRAM, 20 MHz", left=list(reversed(["CS#", "SCK", "SI", "SO"])),
              right=list(reversed(["VDD", "WP#", "HOLD#", "VSS"])), size=(5.4, 3.0))
d += fram.at((xp, TOP - 10))
for a, net in (("CS", "FRAM_CS_N (PG5)"), ("SCK", "FRAM_SCK (PG2, 33 Ohm)"), ("SI", "FRAM_MOSI (PG4)"), ("SO", "FRAM_MISO (PG3)")):
    sch.flag(d, getattr(fram, a), net, "left", 0.6)
sch.flag(d, fram.VDD, "3V3_OBC", "right", 0.6)
sch.flag(d, fram.WP, "FRAM_WP_N (PE0, 10k up)", "right", 0.6)
sch.flag(d, fram.HOLD, "FRAM_HOLD_N (PE1, 10k up)", "right", 0.6)
d += elm.Line().at(fram.VSS).right().length(0.6)
d += elm.Ground()

rtc = sch.ic("U4 RV-3028-C7\nmission clock, 0x52\n32-bit UNIX counter, +/-1 ppm", left=list(reversed(["SCL", "SDA", "EVI", "CLKOUT"])),
             right=list(reversed(["VDD", "VBACKUP", "INT", "VSS"])), size=(6.0, 3.2), spread=("left", "right"))
d += rtc.at((xp, TOP - 17))
sch.flag(d, rtc.SCL, "I2C1_SCL (PG14, 4.7k)", "left", 0.6)
sch.flag(d, rtc.SDA, "I2C1_SDA (PG13, 4.7k)", "left", 0.6)
sch.flag(d, rtc.EVI, "NRST via 100k (1M down)", "left", 0.6)
sch.flag(d, rtc.CLKOUT, "RTC_1HZ via 1M (PC8)", "left", 0.6)
sch.flag(d, rtc.VDD, "3V3_RTC (J1-38: 3V3_AO via 1k)", "right", 0.6)
d += elm.Resistor().at(rtc.VBACKUP).right().length(1.4).label("10k", loc="top", fontsize=7)
d += elm.Ground()
d += elm.Line().at(rtc.INT).right().length(0.6)
d += elm.Label().label("n.c.", loc="right", fontsize=7)
d += elm.Line().at(rtc.VSS).right().length(0.6)
d += elm.Ground()

acc = sch.ic("J9 access port (W8)\nSamtec TST-105, 2.54 mm 2x5",
             left=list(reversed(["1 VTREF", "2 SWDIO", "3 GND", "4 SWCLK", "5 WDT_DIS"])),
             right=list(reversed(["6 NRST", "7 USART3_TX", "8 USART3_RX", "9 UMBILICAL_SENSE", "10 GND"])), size=(6.0, 3.6))
d += acc.at((xp, TOP - 24.5))
for a, net, side in (("1", "3V3_OBC via 100 Ohm", "left"), ("2", "SWDIO", "left"), ("4", "SWCLK", "left"),
                     ("5", "WDT_DIS (10k)", "left"), ("6", "NRST (100 Ohm)", "right"), ("7", "USART3_TX (100 Ohm)", "right"),
                     ("8", "USART3_RX (10k)", "right"), ("9", "UMBILICAL_SENSE (1k, 100k down)", "right")):
    sch.flag(d, sch.pin(acc, a), net, side, 0.6)
d += elm.Line().at(sch.pin(acc, "3")).left().length(0.6)
d += elm.Ground().left()
d += elm.Line().at(sch.pin(acc, "10")).right().length(0.6)
d += elm.Ground()
sch.note(d, (xp - 2, TOP - 29.2), "GSE plug ties pin 9 to pin 1: UMBILICAL_SENSE high = present (key loading).\n"
         "ESD: TPD4E05U06-Q1 at every connector (J1 x7, J2 x3, J3, J5, J8 x2, J9 x2, J10).", fontsize=7)

d.save(sch.out_path("S3_obc_core"))
