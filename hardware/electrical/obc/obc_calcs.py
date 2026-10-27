#!/usr/bin/env python3
"""SMART-QSO OBC board (SQSO-OBC-BRD-001): every calculated value of OBC_BOARD_DESIGN.md.

Run: python3 hardware/electrical/obc/obc_calcs.py   (numpy not needed)
Writes obc_calcs.json next to this file and prints a summary. design.py reads the "stress" strings from it.

Inputs are the worst cases of docs/BASELINE_PARAMETERS.md Rev D, the EPS board (SQSO-EPS-BRD-001 §8.3), the RF
board (SQSO-RF-BRD-001 §4.1) and the datasheets named at each block. Values marked ASSUMED are not published by the
vendor and become verification items (OBC_BOARD_DESIGN.md §16).
"""

import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
R = {}


def put(key, value, unit="", note="", stress=""):
    R[key] = {"value": value, "unit": unit, "note": note}
    if stress:
        R[key]["stress"] = stress
    return value


# ------------------------------------------------------------------------------------------------ worst-case inputs
VBAT_MIN, VBAT_NOM, VBAT_MAX, VBAT_ABS = 11.2, 14.4, 16.4, 16.8      # BASELINE §5, EPS_DESIGN §4 (16.8 V abs max)
V3_NOM, V3_TOL = 3.30, 0.03                                          # 3V3_AO +/-3 % (EPS_DESIGN §4)
T_BOARD_HOT = 60.0                                                   # hot TVAC plateau cap (BASELINE §8), board
T_BOARD_COLD = -40.0                                                 # dispenser cold soak bound; flight min -11.7 C

# ------------------------------------------------------------------------------------------------ 1 magnetorquers
GAIN = 2.3              # A.m2/A, CubeSpace CR0002 (satsearch listing; datasheet not read)
M_SAT = 0.20            # A.m2, flight-software saturation per axis (ICD_OBC_ADCS 12.2)
R_ROD = 51.0            # ohm, ASSUMED (CubeSpace brochure table, column order unclear); range 30-80 ohm covered below
R_ROD_RANGE = (30.0, 80.0)
L_ROD_CASES = (5e-3, 20e-3, 50e-3)   # H, ASSUMED: inductance not published; ripple is computed for each
F_PWM = 40e3            # Hz, recommended TIM1 PWM (DRV8876 <= 100 kHz)
RDS_25, RDS_150 = 0.42, 0.66        # ohm max per FET, DRV8876-Q1 (HS + LS in the path)
A_IPROPI = 1000e-6      # A/A
AERR_LOW = 7.5e-3       # A, IPROPI error below 0.15 A
R_IPROPI = 10.0e3
VREF_R1, VREF_R2 = 18.7e3, 10.0e3

i_sat = put("mtq_i_sat", M_SAT / GAIN, "A", "coil current for 0.20 A.m2")
put("mtq_v_rod_sat", i_sat * R_ROD, "V", "voltage across the rod at saturation (CR0002 rated 5 V max)")
p_rod = put("mtq_p_rod_sat", i_sat ** 2 * R_ROD, "W", "dissipation per rod at saturation")
put("mtq_p_3axes_sat", 3 * p_rod, "W", "three rods saturated (detumble worst case)")
for v, tag in ((VBAT_MIN, "vmin"), (VBAT_NOM, "vnom"), (VBAT_MAX, "vmax")):
    d = i_sat * (R_ROD + 2 * RDS_150) / v
    put(f"mtq_duty_sat_{tag}", d, "", f"PH/EN slow-decay duty for 0.20 A.m2 at VBAT {v} V")
d_worst_r = i_sat * (R_ROD_RANGE[1] + 2 * RDS_150) / VBAT_MIN
put("mtq_duty_sat_80ohm_vmin", d_worst_r, "", "duty at 80 ohm and 11.2 V: must be < 1 (headroom)")
i_supply = put("mtq_isupply_3axes_vnom", 3 * p_rod / VBAT_NOM / 0.97, "A", "VBAT_MTQ current, 3 axes saturated, 97 % bridge efficiency")
vref = put("mtq_vref", V3_NOM * VREF_R2 / (VREF_R1 + VREF_R2), "V", "VREF divider 18.7 k / 10.0 k from 3V3_OBC")
itrip = put("mtq_itrip_nom", vref / (R_IPROPI * A_IPROPI), "A", "hardware current limit ITRIP = VREF / (R_IPROPI x A_IPROPI)")
it_lo = (vref * (1 - V3_TOL) * 0.998) / (R_IPROPI * 1.001 * A_IPROPI) - AERR_LOW
it_hi = (vref * (1 + V3_TOL) * 1.002) / (R_IPROPI * 0.999 * A_IPROPI) + AERR_LOW
put("mtq_itrip_min", it_lo, "A", "3V3 -3 %, 0.1 % resistors, AERR -7.5 mA")
put("mtq_itrip_max", it_hi, "A", "3V3 +3 %, 0.1 % resistors, AERR +7.5 mA")
put("mtq_itrip_margin_over_sat", it_lo / i_sat, "", "ITRIP(min) over the saturation current: the limit never cuts a valid command")
put("mtq_p_rod_fault", it_hi ** 2 * R_ROD, "W", "rod dissipation with the software at 100 % duty, held by ITRIP(max)")
put("mtq_i_fault_unlimited", VBAT_MAX / (R_ROD_RANGE[0] + 2 * RDS_25), "A", "what 100 % duty would give on a 30-ohm rod without ITRIP")
put("mtq_p_supply_fault", 3 * it_hi ** 2 * R_ROD / VBAT_MIN / 0.95, "A", "VBAT_MTQ current, all three bridges in ITRIP at 11.2 V (sizes the EPS switch)")
for L in L_ROD_CASES:
    d = i_sat * R_ROD / VBAT_MAX
    ripple = VBAT_MAX * d * (1 - d) / (L * F_PWM)
    put(f"mtq_ripple_{int(L * 1e3)}mH", ripple, "A p-p", f"slow-decay current ripple at 40 kHz, 16.4 V, L {L * 1e3:.0f} mH (ASSUMED L)")
put("mtq_ipropi_scale", R_IPROPI * A_IPROPI, "V/A", "IPROPI into 10.0 k: 10 mV/mA; 0.87 V at saturation; 3.3 V full scale = 330 mA")
p_drv = put("mtq_p_drv", it_hi ** 2 * 2 * RDS_150 + VBAT_MAX * 7e-3, "W", "per DRV8876: conduction at ITRIP(max), 150 C RDS, plus IVM 7 mA max")
put("mtq_tj_drv", T_BOARD_HOT + p_drv * 39.6, "C", "Tj at 60 C board, RthJA 39.6 K/W (HTSSOP-16 PWP JEDEC, ASSUMED from TI family data)")
put("mtq_p_quiescent", 3 * VBAT_NOM * 3e-3 * 0.9, "W", "three bridges awake 90 % of each period (IVM 3 mA typ)")
# hardware off-timer: CD74HC4060 RC oscillator, Q10 after 512 periods
RT, CT, RS = 80.6e3, 10e-9, 162e3
f_osc = put("mtq_bs_fosc", 1 / (2.3 * RT * CT), "Hz", "f = 1/(2.3 Rt Ct), Rt 80.6 k, Ct 10 nF C0G")
t_bs = put("mtq_bs_t", 512 / f_osc, "s", "Q10 rises after 512 periods: coils cut")
put("mtq_bs_t_min", t_bs * 0.97, "s", "after trim, +/-3 % drift (-40..+85 C, RF board's measured class of the same part)")
put("mtq_bs_t_max", t_bs * 1.03, "s", "")
put("mtq_bs_margin_low", t_bs * 0.97 - 0.900, "s", "backstop above the 900 ms burst: never truncates a valid burst")
put("mtq_bs_margin_high", 1.000 - t_bs * 1.03, "s", "backstop below the 1000 ms period: re-armed every period")
put("mtq_end_of_burst", 1e-3 + 80e-3 * it_hi / VBAT_MIN, "s", "nSLEEP low to zero coil current: tSLEEP 1 ms + L*I/V decay against VM (L 80 mH bound)")

# ------------------------------------------------------------------------------------------------ 2 supervisor
C_CWD = 440e-9          # four parallel strings of two 220 nF C0G in series (KEMET C1210C224J3GACAUTO; SMA-B08)
CWD_TOL = 0.05 + 30e-6 * 65   # C0G: +/-5 % tolerance, 0 +/- 30 ppm/K over 25 C +/- 65 K, no ageing
twdu = put("wd_twdu_typ", 77.4 * C_CWD * 1e6 + 0.055, "s", "TPS3850 eq. 6: tWDU = 77.4 x C(uF) + 0.055")
twdu_min = put("wd_twdu_min", 0.905 * (77.4 * C_CWD * (1 - CWD_TOL) * 1e6 + 0.055), "s", "0.905 x, C -5.2 % (tolerance and temperature)")
twdu_max = put("wd_twdu_max", 1.095 * (77.4 * C_CWD * (1 + CWD_TOL) * 1e6 + 0.055), "s", "1.095 x, C +5.2 %")
put("wd_twdl_max", 0.125 * twdu_max, "s", "SET0 = SET1 = 0: lower boundary 1/8 of tWDU")
put("wd_kick", 10.0, "s", "flight software WDI falling edge every 10 s while every task has checked in")
put("wd_margin_low", 10.0 / (0.125 * twdu_max), "", "kick interval over the lower boundary")
put("wd_margin_high", twdu_min / 10.0, "", "upper boundary over the kick interval")
LSI_MIN, LSI_MAX = 29.5e3, 34.0e3   # STM32L4 LSI spread (ASSUMED datasheet class; DS12023 not read)
put("iwdg_t_min", 3750 * 256 / LSI_MAX, "s", "IWDG reload 3750, prescaler 256, at the fastest LSI")
iwdg_max = put("iwdg_t_max", 3750 * 256 / LSI_MIN, "s", "IWDG reload 3750 at the slowest LSI")
put("wd_vs_iwdg", twdu_min - iwdg_max, "s", "external upper bound minus the IWDG's longest: the IWDG acts first")
for tag, cw in (("one_short", 3 * 110e-9 + 220e-9), ("one_open", 3 * 110e-9)):
    put(f"wd_{tag}_twdu_min", 0.905 * (77.4 * cw * (1 - CWD_TOL) * 1e6 + 0.055), "s", f"CWD {cw * 1e9:.0f} nF after one capacitor {tag.split('_')[1]}")
    put(f"wd_{tag}_twdu_max", 1.095 * (77.4 * cw * (1 + CWD_TOL) * 1e6 + 0.055), "s", "upper bound, max")
    put(f"wd_{tag}_twdl_max", 0.125 * 1.095 * (77.4 * cw * (1 + CWD_TOL) * 1e6 + 0.055), "s", "lower bound, max (must stay below the 10 s kick)")
put("iwdg_3200_min", 3200 * 256 / 34e3, "s", "recommended IWDG reload 3200 / 256 at LSI 34 kHz")
put("iwdg_3200_max", 3200 * 256 / 29.5e3, "s", "at LSI 29.5 kHz: below the supervisor's minimum")
put("wd_vs_iwdg_3200", twdu_min - 3200 * 256 / 29.5e3, "s", "supervisor minimum minus the IWDG maximum at reload 3200: the IWDG acts first")
put("wd_uv", 3.30 * 0.93, "V", "TPS3850H33: UV 7 % (3.069 V nominal, +/-0.8 %)")
put("wd_ov", 3.30 * 1.07, "V", "OV 7 % (3.531 V)")
put("wd_trst", 0.200, "s", "CRST open: 170-230 ms")

# ------------------------------------------------------------------------------------------------ 3 clocks
put("hse_pll", "8 MHz /1 x20 /2 = 80 MHz", "", "HSE bypass from the XO: PLLM 1, PLLN 20, PLLR 2")
ESR, F32, C0, CL = 70e3, 32768.0, 1.4e-12, 7e-12
gm = put("lse_gmcrit", 4 * ESR * (2 * math.pi * F32) ** 2 * (C0 + CL) ** 2 * 1e6, "uA/V",
         "gm_crit = 4 ESR (2 pi f)^2 (C0+CL)^2 (AN2867), CM7V ESR 70 k max, C0 1.4 pF")
put("lse_drive_margin", 1.7 / gm, "", "LSEDRV medium-high Gm_crit_max 1.7 uA/V (ASSUMED STM32L4 class value) over gm_crit")
put("lse_cload", 10e-12 / 2 + 2.5e-12, "F", "10 pF + 10 pF in series plus 2.5 pF stray = 7.5 pF (crystal CL 7 pF: about -9 ppm pull; the mission clock is the RV-3028)")
put("rtc_drift_day", 1e-6 * 86400, "s/day", "RV-3028-C7 +/-1 ppm at 25 C (factory calibrated)")
put("rtc_drift_worst_day", 3.5e-6 * 86400, "s/day", "+/-3.5 ppm over -40..+85 C with its temperature compensation (ASSUMED from application manual class)")

# ------------------------------------------------------------------------------------------------ 4 sensor rails
put("snsw_limit", (0.050, 0.075, 0.100), "A", "TPS2553-Q1 ILIM tied to IN: 50 / 75 / 100 mA")
LOADS = {"gyro": 0.016 * 1.6, "imu2": 0.005 * 1.6, "sns": 0.0006 + 0.0004 + 0.0015 + 0.012, "mag": 0.0005 * 1.6 + 0.0015}
for k, v in LOADS.items():
    put(f"snsw_{k}_load", v, "A", "normal maximum (gyro and IMU2: typ x 1.6; sns: ADS7138, TIAs, TCA9517 U14, 12 mA photocurrent; mag: RM3100 x 1.6, TCA9517 U34)")
    put(f"snsw_{k}_margin", 0.050 / v, "", "minimum limit over the normal maximum (>= 1.35 for no false trips)")
put("snsw_retry_off", 0.6 * 220e3 * 2.2e-6, "s", "off time after a trip: 0.6 x 220 k x 2.2 uF nominal (>= 0.1 s: RADIATION_ANALYSIS 5.3)")
C_RET_MIN = 2.2e-6 * 0.90 * 0.85 * 0.85 * 0.95   # X7R: -10 % tolerance, -15 % at -40 C, -15 % DC bias at 3.3 V (0805 25 V), -5 % ageing
C_RET_MAX = 2.2e-6 * 1.10 * 1.02                 # +10 % tolerance, +2 % near +25..85 C
put("snsw_retry_off_min", 0.6 * 220e3 * 0.99 * C_RET_MIN, "s", "worst short: X7R at -40 C with tolerance, bias and ageing; R -1 %")
put("snsw_retry_off_max", 0.6 * 220e3 * 1.01 * C_RET_MAX, "s", "worst long")
put("snsw_turn_on_max", 220e3 * 1.01 * C_RET_MAX * 1.0, "s", "commanded turn-on delay, at most about one RC to the EN threshold (flight software waits >= 0.6 s)")
v_gyro_min = put("gyro_vcc_min", V3_NOM * (1 - V3_TOL) - 0.034 * 0.085 - 0.034 * 0.013 - 0.026 * 0.085 - 0.026 * 0.003, "V",
                 "3V3_AO -3 %, SW4 85 mOhm x 34 mA, W6B 13 mOhm, OBC switch 85 mOhm x 26 mA, trace")
put("gyro_vcc_margin", v_gyro_min - 3.15, "V", "above the M-G370 3.15 V minimum (EPS set point +/-2 % recommended: +33 mV more)")

# ------------------------------------------------------------------------------------------------ 5 sun sensors
ISC_MAX = 4.0e-3        # A, ASSUMED panel photodiode short-circuit current at AM0 normal incidence (panel vendor)
RF_TIA = 619.0
put("css_vout_isc", ISC_MAX * RF_TIA, "V", "TIA output at 4.0 mA (Rf 619 ohm)")
put("css_vout_clip", 3.25 / RF_TIA, "A", "photocurrent at which the TIA clips (3.25 V output)")
put("css_lsb", 3.3 / 4096 / RF_TIA, "A", "ADS7138 12-bit LSB in photocurrent")
put("css_angle_res", math.degrees(math.acos(1 - 3.3 / 4096 / (ISC_MAX * RF_TIA))), "deg", "angle step at normal incidence from 1 LSB (cosine)")
put("css_p_rf", ISC_MAX ** 2 * RF_TIA, "W", "feedback resistor power at 4 mA (0.1 W part)", stress="10 mW / 100 mW = 10 %")

# ------------------------------------------------------------------------------------------------ 5b Jetson spreader NTC (E10)
B_NTC, R25 = 3976.0, 10.0e3      # TE GA10K3MCD1, curve 3 (beta 25/85 = 3976 K, ASSUMED from the TE curve family), 10 k at 25 C
RB = 10.0e3                      # bias from VDDA, 0.1 %


def r_ntc(tc):
    return R25 * math.exp(B_NTC * (1 / (tc + 273.15) - 1 / 298.15))


def v_ntc(tc, vref=3.3):
    r = r_ntc(tc)
    return vref * r / (RB + r)


for tc, tag in ((-15, "m15"), (25, "25"), (70, "70"), (-40, "m40"), (95, "95")):
    put(f"ntc_r_{tag}", r_ntc(tc), "ohm", f"RT1 at {tc} C")
    put(f"ntc_v_{tag}", v_ntc(tc), "V", f"PA2 at {tc} C (VDDA 3.3 V; the reading is ratiometric)")
for tc, tag in ((-15, "m15"), (70, "70")):
    dv = (v_ntc(tc + 0.5) - v_ntc(tc - 0.5))
    put(f"ntc_slope_{tag}", dv, "V/K", f"sensitivity at {tc} C")
    put(f"ntc_lsb_{tag}", 3.3 / 4096 / abs(dv), "K/LSB", f"12-bit resolution at {tc} C")
    # 0.1 % bias + 1 % NTC resistance (about +/-0.2 C class interchangeability over 0-70 C) + ADC +/-4 LSB
    r = r_ntc(tc)
    dr = abs(B_NTC * r / (tc + 273.15) ** 2)
    err = math.sqrt((0.001 * r / dr) ** 2 + (0.01 * r / dr) ** 2 + (4 * 3.3 / 4096 / abs(dv)) ** 2)
    put(f"ntc_err_{tag}", err, "K", f"error RSS at {tc} C (bias 0.1 %, NTC 1 %, ADC 4 LSB)")
put("ntc_p_bias_max", 1e3 * max((3.3 ** 2 * RB / (RB + r_ntc(t)) ** 2) for t in (-40, 25, 70, 95)), "mW",
    "bias resistor power, worst (at 95 C)")
put("ntc_self_heat", 1e3 * (v_ntc(70) ** 2 / r_ntc(70)), "mW", "NTC dissipation at 70 C (bonded: < 0.1 K rise at about 3 mW/K)")

# ------------------------------------------------------------------------------------------------ 5c rod remanence at the magnetometer
MU0_4PI = 1e-7
ROD_L = 0.047
RODS = {"X": ((0.0, 0.036, -0.010), (1, 0, 0)), "Y": ((0.042, 0.0, -0.033), (0, 1, 0)),
        "Z": ((-0.0415, 0.0, -0.0205), (0, 0, 1))}      # MECHANICAL_DESIGN 4.1 (centres; Z rod -44 ... +3)


def rod_field(p, m=1e-3):
    """|B| in nT from each rod for a remanent dipole m (A.m2), rod as two magnetic charges at its ends."""
    out = {}
    for k, (c, a) in RODS.items():
        q = m / ROD_L
        b = [0.0, 0.0, 0.0]
        for sgn in (1, -1):
            e = [c[i] + sgn * a[i] * ROD_L / 2 for i in range(3)]
            r = [p[i] - e[i] for i in range(3)]
            d = math.sqrt(sum(x * x for x in r))
            for i in range(3):
                b[i] += sgn * MU0_4PI * q * r[i] / d ** 3
        out[k] = math.sqrt(sum(x * x for x in b)) * 1e9
    return out


MAG_POS = {"obc_corner": (0.033, -0.038, 0.007), "obc_best": (0.002, -0.042, 0.007),
           "plusz_cad": (0.024, 0.027, 0.044), "plusz_mirror": (0.024, -0.027, 0.044)}
for tag, pos in MAG_POS.items():
    f = rod_field(pos)
    put(f"mag_rod_{tag}", {k: round(v, 0) for k, v in f.items()}, "nT per mA.m2", f"rod field at {pos} m per 1 mA.m2 of remanence")
    put(f"mag_rod_{tag}_sum", sum(f.values()), "nT per mA.m2", "sum of the three rods (worst alignment)")
    put(f"mag_mr_allow_{tag}", 300.0 / (2 * sum(f.values())), "mA.m2",
        "per-rod remanence whose polarity reversal (2 m_r) changes the field by the Monte Carlo's 300 nT bias")

# ------------------------------------------------------------------------------------------------ 6 readback dividers
for v in (VBAT_MIN, VBAT_NOM, VBAT_MAX, VBAT_ABS):
    put(f"rb_{v}", v * 200e3 / 1.2e6, "V", f"VSW1_RB / VJET_RB at {v} V (1 M / 200 k, EPS side)")
put("rb_threshold", 1.0, "V", "software: on above 1.0 V (6.0 V rail), off below; 1.87 V at the lowest bus")

# ------------------------------------------------------------------------------------------------ 7 back-power into an unpowered OBC
BP = {
    "EPS_ALERT_N (4.7 k EPS pull-up + 10 k)": 3.3 / 14.7e3,
    "SOLAR_SENSE (11 k EPS + 10 k)": 3.3 / 21e3,
    "DEPLOY_SENSE (1 M)": 3.3 / 1e6,
    "five EPS analog lines (>= 167 k)": 5 * 2.8 / 167e3,
    "VSW1_RB, VJET_RB (167 k + 1 k)": 2 * 2.8 / 168e3,
    "TNC_TXD idle high (100 + 10 k)": 3.1 / 10.1e3,
    "TONE_OK, XPDR_KEY, PTT_EXC, RX_CD, RX_FAULT if high (1 k + 10 k)": 5 * 3.1 / 11e3,
    "TX_TEMP, TX_FWD, TX_REV (1 k + 10 k, at 1.7/2.5/1.25 V)": (1.7 + 2.5 + 1.25) / 11e3,
    "RTC CLKOUT (1 M), EVI (input)": 3.3 / 1e6,
    "Jetson, GNSS, sensor buses (translators: Ioff)": 0.0,
}
bp_total = sum(BP.values())
put("backpower_lines", BP, "A", "current into the clamps of an unpowered OBC with every other board powered")
put("backpower_total", bp_total, "A", "worst case (every readback high); typical with readbacks low about 0.8 mA")

# ------------------------------------------------------------------------------------------------ 8 power budget
OBC = {"STM32L4R5 at 80 MHz, peripherals on (allocation; DS12023 not read)": 20e-3, "8 MHz XO": 3e-3,
       "FM25V20A (active 20 MHz, 15 % of time)": 0.45e-3, "TPS3850, TMP235, HC4060, gates": 0.2e-3,
       "translators and TCA9517 A side": 0.3e-3, "pull-downs on driven-high EPS/RF lines": 0.25e-3,
       "I2C pull-ups (average)": 0.8e-3, "DRV8876 VREF divider": 0.12e-3,
       "spreader NTC bias (10.0 k + RT1 at 70 C)": 3.3 / (10e3 + r_ntc(70))}
i_obc = put("i_3v3_obc", sum(OBC.values()), "A", "3V3_OBC worst normal; EPS SW10 trips at 0.130 A min")
put("i_3v3_obc_items", OBC, "A")
put("i_3v3_obc_margin", 0.130 / i_obc, "", "trip over load")
i_adcs = put("i_3v3_adcs", LOADS["gyro"] + LOADS["imu2"] + LOADS["sns"] + LOADS["mag"], "A", "3V3_ADCS worst with the backup gyro on; SW4 trips at 0.265 A min")
put("p_obc", i_obc * V3_NOM, "W", "OBC board from 3V3_OBC")
put("p_sensors", (0.016 + 0.0005 + 0.0006 + 0.0004 + 0.0015 + 0.004) * V3_NOM, "W", "sensors normal: gyro, mag, ADC, TIAs, buffer, 4 mA photocurrent average")
put("c_bulk_3v3", 22e-6 + 10e-6 + 4.7e-6 + 30 * 0.1e-6, "F", "3V3_OBC capacitance on the OBC")
put("t_inrush", (22e-6 + 10e-6 + 4.7e-6 + 3e-6) * 3.3 / 0.130, "s", "charge time at the SW10 minimum limit (< 8 ms FAULT deglitch: no trip at power-up)")

# ------------------------------------------------------------------------------------------------ 9 buses
CB = 120e-12
put("i2c2_tr", 0.8473 * 2.35e3 * CB, "s", "I2C2 rise time, two 4.7 k in parallel (2.35 k), 120 pF: <= 300 ns for 400 kHz")
put("i2c2_tr_one", 0.8473 * 4.7e3 * CB, "s", "I2C2 rise time with one pull-up open (4.7 k): <= 1000 ns for 100 kHz")
put("i2c3_tr", 0.8473 * 2.35e3 * 80e-12, "s", "I2C3 (AntS) rise time, 2.35 k, 80 pF (W3 + EPS U80 A side)")
put("i2c3_tr_one", 0.8473 * 4.7e3 * 80e-12, "s", "I2C3 with one pull-up open: <= 1000 ns for 100 kHz")
put("tx_temp_pd_err", 11e3 / 1e6, "", "TX_TEMP reading low by 11 k / 1 M (1.1 %) with the 1 M open-line pull-down on PA0")
put("i2c2_iol", 3.3 / 2.35e3, "A", "sink current at VOL with both pull-ups (<= 3 mA)")
put("i2c4_tr", 0.8473 * 2.2e3 * 50e-12, "s", "I2C4 OBC side, 50 pF")
put("i2c4b_tr", 0.8473 * 4.7e3 * (0.10 * 100e-12 + 25e-12), "s",
    "I2C4 sensor side (4.7 k): W6E 0.10 m at 100 pF/m plus 25 pF of parts and ESD on both boards; <= 300 ns at 400 kHz")
put("i2c1_tr", 0.8473 * 4.7e3 * 30e-12, "s", "I2C1 to the RTC, 4.7 k, 30 pF (100 kHz)")

# ------------------------------------------------------------------------------------------------ 10 harness
WIRES = {"W3 3V3_OBC (2 x 28 AWG, SW10 trip 0.176 A max)": (0.176 / 2, 0.25),
         "W3 VBAT_MTQ (2 x 28 AWG, all bridges in ITRIP)": (3 * it_hi ** 2 * R_ROD / VBAT_MIN / 0.95 / 2, 0.25),
         "W6B 3V3_ADCS (26 AWG, SW4 trip 0.339 A max)": (0.339, 0.5),
         "W6C coil (26 AWG, ITRIP max)": (it_hi, 0.5),
         "W9B 3V3_GNSS (26 AWG, SW7 trip 0.232 A max)": (0.232, 0.5)}
put("wires", {k: {"current_A": a, "derated_A": b, "use": a / b} for k, (a, b) in WIRES.items()}, "",
    "SQSO-HAR-001 5.2 (28 AWG 0.25 A, 26 AWG 0.5 A after 50 % derating)")

# ------------------------------------------------------------------------------------------------ 11 thermal
put("tj_mcu", T_BOARD_HOT + 0.066 * 45.0, "C", "STM32L4R5 66 mW, RthJA 45 K/W LQFP144 (ASSUMED)")
put("tj_tps2553", T_BOARD_HOT + 0.1 ** 2 * 0.135 * 120 + 0.0001, "C", "at the 100 mA limit in steady state, 135 mOhm hot, 120 K/W")
put("tj_txu", T_BOARD_HOT + 0.002, "C", "microwatts")

with open(os.path.join(HERE, "obc_calcs.json"), "w") as f:
    json.dump(R, f, indent=1, default=str)

if __name__ == "__main__":
    for k, v in R.items():
        val = v["value"]
        if isinstance(val, float):
            print(f"{k:28s} {val:12.5g} {v['unit']:6s} {v['note']}")
        elif isinstance(val, dict):
            print(f"{k:28s} " + "; ".join(f"{a}: {b if not isinstance(b, float) else round(b, 6)}" for a, b in val.items()))
        else:
            print(f"{k:28s} {val} {v['unit']} {v['note']}")
