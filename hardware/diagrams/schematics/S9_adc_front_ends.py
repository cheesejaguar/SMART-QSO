"""S9 ADC1 front ends, version 2 (2026-10-27): the sixteen external ADC1 inputs of the STM32L4R5ZIT6 with the source
circuits on the EPS and RF boards (SQSO-EPS-BRD-001 11, SQSO-RF-BRD-001 7.6) and the OBC-side series resistors and
reservoir capacitors (SQSO-OBC-BRD-001 13-14). Version 1 drew the EPS dividers as proposals; they are now designed on
the EPS board (the 1:6 VBATT divider of finding F1 is built).
"""

import schemdraw.elements as elm

import sch

d = sch.drawing("S9_adc_front_ends")
sch.note(d, (-5, 4.5), "S9  ADC1 front ends (version 2)", fontsize=12, color="black")
sch.note(d, (-5, 3.9), "SQSO-HW-DIAG-001 · source circuits on the EPS and RF boards; OBC series R and reservoir C from SQSO-OBC-BRD-001 · "
         "firmware scale = adc.rs FRONT_END", fontsize=8)

ROWS = [
    ("EPS", "VBATT_SNS", "PACK_P: 1.00M / 200k, 100 nF", "1 k", "PC2 IN3", "x 6.0 (adc.rs Linear(6.0, 0): change from 3.0)"),
    ("EPS", "IBATT_SNS", "INA186 x 50 on RS6, REF 1.65 V", "1 k", "PC3 IN4", "0.5 V/A, 1.65 V at 0 A (unchanged)"),
    ("EPS", "VSOLAR_SNS", "string: 300k / 100k", "1 k", "PC4 IN13", "x 4.0 (unchanged)"),
    ("EPS", "ISOLAR_SNS", "INA186 x 50 on RS5", "1 k", "PC5 IN14", "1.0 V/A (unchanged)"),
    ("EPS", "VBUS_SNS", "5V_PA: 100k / 100k", "1 k", "PB0 IN15", "x 2.0 (unchanged)"),
    ("EPS", "VSW1_RB", "VBAT_SW1: 1.00M / 200k, 100 nF", "1 k", "PA1 IN6", "x 6.0, on above 1.0 V (new, RID AV-06)"),
    ("EPS", "VJET_RB", "VBAT_JET: 1.00M / 200k, 100 nF", "1 k", "PA7 IN12", "x 6.0, on above 1.0 V (new, RID AV-06)"),
    ("RF", "TX_TEMP", "PA sensor, 1k / 100 nF", "10 k", "PA0 IN5", "10 mV/C, 0.5 V at 0 C (unchanged)"),
    ("RF", "TX_FWD", "coupler + RMS detector, 1k / 100 nF", "10 k", "PC0 IN1", "P = 0.16 V^2 W (unchanged)"),
    ("RF", "TX_REV", "coupler reverse port, 1k / 100 nF", "10 k", "PC1 IN2", "as TX_FWD (unchanged)"),
    ("OBC", "MTQ_X_I", "DRV8876 IPROPI: 10.0k, 1 nF", "1 k", "PA4 IN9", "10 mV/mA (new)"),
    ("OBC", "MTQ_Y_I", "DRV8876 IPROPI: 10.0k, 1 nF", "1 k", "PA5 IN10", "10 mV/mA (new)"),
    ("OBC", "MTQ_Z_I", "DRV8876 IPROPI: 10.0k, 1 nF", "1 k", "PA6 IN11", "10 mV/mA (new)"),
    ("OBC", "TEMP_BOARD", "TMP235-Q1 U21", "1 k", "PB1 IN16", "10 mV/C, 0.5 V at 0 C (unchanged)"),
]

for i, (board, net, src, rser, pin, scale) in enumerate(ROWS):
    y = -1.6 * i
    sch.note(d, (-5, y + 0.25), f"{board}: {src}", fontsize=7)
    d += elm.Line().at((4.5, y)).right().length(0.6)
    d += elm.Resistor().right().length(1.6).label(rser, loc="top", fontsize=7)
    d += (n := elm.Dot())
    d += elm.Capacitor().at(n.center).down().length(0.9).label("10 nF", loc="left", fontsize=6)
    d += elm.Ground()
    sch.flag(d, n.center, f"{net} -> {pin}", "right", 0.6)
    sch.note(d, (16.5, y + 0.25), scale, fontsize=7)

sch.note(d, (-5, -1.6 * len(ROWS) - 0.5),
         "Series resistors: 1 kOhm on the EPS lines (their sources are >= 167 kOhm with 100 nF), 10 kOhm on the RF lines (the radio\n"
         "stays powered while the OBC is power-cycled: they limit the current into an unpowered OBC). The source's capacitor is the\n"
         "reservoir for the 247.5-cycle sample. Sun sensors, the sensor-area temperature and VBAT_MTQ / 6 are on the ADS7138 (S6).\n"
         "ADC1 IN7 (PA2) and IN8 (PA3) are the JETSON_PWR and RF_TX_EN outputs; internal channels: temperature, VREFINT.",
         fontsize=7)

d.save(sch.out_path("S9_adc_front_ends"))
