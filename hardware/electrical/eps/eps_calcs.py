#!/usr/bin/env python3
"""SQSO-EPS-BRD-001 design calculations (EPS board, CDR build-to design).

Every value placed on the schematic (design.py) that is not a datasheet default is computed here, at the worst-case
inputs of docs/BASELINE_PARAMETERS.md Rev D and the analyses it cites. Run:

    python3 hardware/electrical/eps/eps_calcs.py            # prints the tables (Markdown)
    python3 hardware/electrical/eps/eps_calcs.py --json out  # also writes the numbers as JSON

Sources of the device constants are given beside each one (datasheet, revision, table). Derating rules: EEE-INST-002
(NASA GSFC, 2003, as tailored by SQSO-HAR-001 section 5.2): see DERATING below.
"""

import json
import math
import sys

# ---------------------------------------------------------------- environment and bus (BASELINE Rev D, analyses)
T_BOARD_HOT = 60.0      # degC, EPS board upper allowable operating temperature (THERMAL_DESIGN 1.3 section 1.1)
T_BOARD_COLD = -30.0    # degC, lower operating AFT
CELL_V = dict(uv=2.80, uv_rel=3.00, floor=3.00, cv=4.10, max=4.20, ov=4.25, ov2=4.35, suv=2.00)
NCELL = 4
VBAT_MIN = NCELL * CELL_V["uv"]          # 11.2 V BMS cut
VBAT_CV = NCELL * CELL_V["cv"]           # 16.4 V
VBAT_MAX = NCELL * CELL_V["max"]         # 16.8 V absolute
VBAT_OV2 = NCELL * CELL_V["ov2"]         # 17.4 V at the secondary protector
VBUS_BURN_COLD = 12.2                    # V, SQSO-EMC-001 section 4, cold-pack burn
STRING_VOC_MAX = 6.31                    # V, -45 degC BOL (EPS_DESIGN 2.0 section 3.1)
STRING_ISC_MAX = 0.55                    # A, 1414 W/m2 hot
STRING_VMP_MIN, STRING_VMP_MAX = 3.9, 5.7
ARRAY_P_MAX = 7.6                        # W, six +Y cells at a cold sunrise, BOL
MTQ_DETUMBLE_W = 1.16                    # W, OBC magnetorquer bridges, three CR0002 rods saturated (SQSO-ICX-001 E3)
MTQ_BRIDGE_LIMIT_A = 0.23                # A at 11.2 V, three DRV8876 bridges at their hardware limit (SQSO-ICX-001 3.1)
SW11_K_CL, SW11_V_CL, SW11_R_CL, SW11_ACC = 600, 0.8, 1.33e3, 0.10   # TPS1H000-Q1 (SLVSDO6E): +/-10 % for 0.3-1 A
SW11_LIMIT = SW11_V_CL * SW11_K_CL / SW11_R_CL
SW11_LIMIT_MAX_A = SW11_LIMIT * (1 + SW11_ACC) * 1.01   # +1 % resistor

# ---------------------------------------------------------------- derating (EEE-INST-002 / SQSO-HAR-001 5.2)
DERATING = {
    "resistor_power": 0.50, "resistor_voltage": 0.80,
    "ceramic_cap_voltage": 0.50, "mosfet_vds": 0.75, "mosfet_vgs": 0.75, "transistor_power": 0.50,
    "diode_piv": 0.70, "diode_current": 0.50, "ic_supply": 0.80, "fuse_current": 0.50, "relay_contact": 0.50,
    "connector_contact": 0.50, "tj_ic_max": 110.0, "tj_discrete_max": 125.0,
}


def ntc_r(t_c, r25=10e3, beta=3435.0):
    """NTC resistance (the pack thermistors: 10 kOhm, B25/85 = 3435 K, e.g. Murata NCP15XH103F03RC class)"""
    return r25 * math.exp(beta * (1.0 / (t_c + 273.15) - 1.0 / 298.15))


def par(a, b):
    return a * b / (a + b)


R = {}  # every computed value, for the JSON


def table(title, header, rows):
    print(f"\n### {title}\n")
    print("| " + " | ".join(header) + " |")
    print("|" + "---|" * len(header))
    for r in rows:
        print("| " + " | ".join(str(x) for x in r) + " |")


# ================================================================ 1 solar input
def solar():
    tvs = dict(vwm=7.0, vbr_min=7.78, vc=12.0, ipp=50.0)      # SMBJ7.0A (Littelfuse SMBJ series datasheet)
    fet_r = 0.012 * 1.6                                       # BUK7M12-40E, 12 mOhm at 10 V, x1.6 at Tj 100 degC
    vak_reg = 0.020                                           # LM74700-Q1 regulated forward drop, typ 20 mV
    p_diode = max(vak_reg, STRING_ISC_MAX * fet_r) * STRING_ISC_MAX
    rows = [
        ("String Voc max / TVS standoff", f"{STRING_VOC_MAX} V / {tvs['vwm']} V", f"{STRING_VOC_MAX / tvs['vwm']:.0%}", "TVS stays off at the cold sunrise"),
        ("TVS clamp / LM74700 ANODE abs max / TPS55340 VIN abs max", f"{tvs['vc']} V / 65 V / 40 V", f"{tvs['vc'] / 40:.0%}", "clamp below every input rating"),
        ("Ideal-diode loss per string at Isc", f"{p_diode * 1e3:.1f} mW", f"{p_diode / (STRING_VMP_MAX * 0.5):.2%} of string power", "SIL allows 3 % (hw/eps.rs DIODE_LOSS)"),
        ("Body-diode fallback (string < 3.1 V, LM74700 POR)", "0.75 V x 0.5 A = 0.38 W", "only below the MPP range", "graceful"),
    ]
    table("1. Solar inputs", ["Item", "Value", "Stress / ratio", "Note"], rows)
    R["solar"] = dict(p_ideal_diode_w=p_diode)


# ================================================================ 2 trackers (TPS55340-Q1 boost, 1 MHz synchronised)
TRK = dict(fsw=1.0e6, L=6.8e-6, dcr=0.048, rds_hot=0.110, vref=1.229, vf_d=0.40, csw=300e-12, tsw=10e-9, iq=0.5e-3)


def boost_loss(pin, vin, vout):
    """Losses of one tracker at input power pin (W), input vin, output vout (V)."""
    iin = pin / vin
    d = 1.0 - vin / (vout + TRK["vf_d"])
    ripple = vin * d / (TRK["L"] * TRK["fsw"])
    irms_l = math.sqrt(iin ** 2 + ripple ** 2 / 12.0)
    p_l = irms_l ** 2 * TRK["dcr"] + 0.02 * pin * 0.1              # copper + core (core ~2 % of 10 % ripple energy)
    p_sw_cond = irms_l ** 2 * d * TRK["rds_hot"]
    iout = iin * (1 - d)
    p_diode = TRK["vf_d"] * iout
    p_sw_trans = 0.5 * vout * iin * TRK["tsw"] * TRK["fsw"] + 0.5 * TRK["csw"] * vout ** 2 * TRK["fsw"]
    p_ic = TRK["iq"] * vin + 0.002                                 # quiescent + gate drive
    p_tot = p_l + p_sw_cond + p_diode + p_sw_trans + p_ic
    return dict(iin=iin, d=d, ripple=ripple, ipk=iin + ripple / 2, p_l=p_l, p_ic=p_sw_cond + p_sw_trans + p_ic,
                p_diode=p_diode, p_tot=p_tot, eff=(pin - p_tot) / pin, iout=iout)


def trackers():
    # FB output clamp (internal reference, +/-2 %) above the precision CV loop (external, 16.40 V +/-0.44 %)
    rbot = 10.0e3
    rtop_e96 = 129e3
    vfb = TRK["vref"] * (1 + rtop_e96 / rbot)
    vfb_min = 1.204 * (1 + rtop_e96 * 0.999 / (rbot * 1.001))
    vfb_max = 1.254 * (1 + rtop_e96 * 1.001 / (rbot * 0.999))
    r_freq = 46.0e3                                                  # f ~ 46,000/R(kOhm) kHz (datasheet table 6.5)
    cases = [("T1 wing fronts, cold sunrise BOL (worst power)", ARRAY_P_MAX * 4 / 6, 5.0),
             ("T1 at the hot EOL Vmp (worst efficiency)", 2 * 1.07 * 0.9, STRING_VMP_MIN),
             ("T2 body +Y, nominal", 1.16, 4.7),
             ("T4 two body strings, tumbling", 2.3, 4.6)]
    rows = []
    out = []
    for name, p, v in cases:
        b = boost_loss(p, v, VBAT_CV)
        out.append((name, b))
        rows.append((name, f"{p:.2f} W at {v:.1f} V", f"{b['iin']:.2f} A / {b['ipk']:.2f} A", f"{b['d']:.2f}",
                     f"{b['p_tot'] * 1e3:.0f} mW ({b['p_ic'] * 1e3:.0f} IC, {b['p_diode'] * 1e3:.0f} diode, {b['p_l'] * 1e3:.0f} L)",
                     f"{b['eff']:.1%}"))
    table("2. Trackers (TPS55340-Q1, 1.000 MHz, 6.8 uH)", ["Case", "Input", "Iin avg / peak", "Duty", "Loss", "Efficiency"], rows)
    print(f"\nFB output clamp: R_top {rtop_e96 / 1e3:.0f} k / R_bot 10.0 k (0.1 %): {vfb:.2f} V nominal, {vfb_min:.2f}-{vfb_max:.2f} V "
          f"with the +/-2 % internal reference. Its minimum stays above the CV loop's maximum (16.47 V), so the precision loop "
          f"always governs; the clamp ({vfb_max / 4:.3f} V/cell worst) only limits VTRK if the CV loop fails, and the AFE's "
          f"4.25 V COV then opens the CHG FET: the clamp is not counted as an overcharge control. "
          f"Switch current limit 5.25 A min against {max(b['ipk'] for _, b in out):.2f} A peak "
          f"({max(b['ipk'] for _, b in out) / 5.25:.0%}). Free-running R_FREQ {r_freq / 1e3:.1f} k ~ 1.0 MHz if SYNC is lost.")
    R["trackers"] = dict(rtop=rtop_e96, rbot=rbot, vfb=vfb, vfb_min=vfb_min, vfb_max=vfb_max, cases={n: b for n, b in out})


# ================================================================ 3 charge control loops (VCTL domain)
def charge_loops():
    vref = 2.500
    # CV: 16.40 V on PACK_P through a 0.1 % divider into an OPA (TLV9002-Q1)
    rb = 10.0e3
    ra = rb * (VBAT_CV / vref - 1.0)          # 55.6 k
    err = dict(ref=0.05 + 6e-6 * 60 * 100, div=2 * 0.1, vos=1.5e-3 / vref * 100, path=0.4 * 0.040 / VBAT_CV * 100)
    tot = sum(err.values())
    v_cell_eoc = CELL_V["cv"] * (1 + tot / 100) + 0.050
    # CC: INA186A3 (100 V/V) on the 10 mOhm gauge shunt -> 1.0 V/A; setpoint from the reference
    r_sh = 0.010
    k = 100 * r_sh
    rs_top, rs_bot = 21.5e3, 4.02e3
    vset = vref * rs_bot / (rs_top + rs_bot)
    icc = vset / k
    icc_err = 1.0 + 1.0 + 0.2 + 0.05 + (50e-6 / r_sh / 100) / icc * 100 * 100 / 100
    i_resid = 50e-6 / r_sh
    rows = [("CV set point", f"R_top {ra / 1e3:.1f} k / R_bot 10.0 k, 0.1 %, REF3425 2.500 V", f"{VBAT_CV:.2f} V", f"+/-{tot:.2f} % (ref {err['ref']:.2f}, divider {err['div']:.2f}, offset {err['vos']:.2f}, path {err['path']:.2f})"),
            ("Worst cell at end of charge", "set point error + 50 mV balance window", f"{v_cell_eoc:.3f} V", "below 4.20 V cell max and 4.25 V AFE threshold"),
            ("CC limit", f"INA186A3 x100 on 10 mOhm = 1.0 V/A; divider {rs_top / 1e3:.1f} k / {rs_bot / 1e3:.2f} k", f"{icc:.3f} A", f"+/-{icc_err:.1f} %"),
            ("Charge inhibited (cold, hot, lockout)", "CC set point forced to 0 V (Q18 and relay K1 pole B in parallel)", f"<= {i_resid * 1e3:.0f} mA residual", "INA186 offset 50 uV max; AFE CHG FET also open"),
            ("Trackers' FB output clamp (if the VCTL loops are dead)", "internal 1.229 V +/-2 % reference", f"{R['trackers']['vfb']:.2f} V", f"{R['trackers']['vfb_min']:.2f}-{R['trackers']['vfb_max']:.2f} V; the AFE COV 4.25 V acts above 17.0 V")]
    table("3. Charge regulation (hardware, VCTL domain)", ["Loop", "Implementation", "Value", "Accuracy / note"], rows)
    R["charge"] = dict(cv_ra=ra, cv_err_pct=tot, v_cell_eoc=v_cell_eoc, icc=icc, icc_err_pct=icc_err)


# ================================================================ 4 battery protection
def protection():
    rs_afe = 0.010
    ocd_mv = 40.0
    ocd = ocd_mv / 1e3 / rs_afe
    ocd_lo, ocd_hi = (ocd_mv - 4) / 1e3 / rs_afe, (ocd_mv + 4) / 1e3 / rs_afe     # +/-4 mV at 20-56 mV settings
    scd = 0.100 / rs_afe
    i_legit = dict(jetson=8.0 / 0.95 / 13.4, jetson_guard=12.0 / 0.95 / 13.4, pa=2.42 / 0.94 / 13.4, hk=0.70 / 0.85 / 13.4,
                   heater=VBAT_CV / 207.0, mtq=MTQ_DETUMBLE_W / 13.4)
    i_max_legit = i_legit["jetson_guard"] + i_legit["pa"] + i_legit["hk"] + i_legit["heater"] + i_legit["mtq"]
    i_max_mtq_lim = i_max_legit - i_legit["mtq"] + MTQ_BRIDGE_LIMIT_A            # coil bridges at their hardware limit
    i_max_sw11 = i_max_legit - i_legit["mtq"] + SW11_LIMIT_MAX_A                 # SW11 in its current limit (short)
    # burns: 7.0 V burn buck, 0.88 A per resistor, buck efficiency 0.88
    i_burn_pack = 0.88 * 7.04 / (0.88 * 13.4)
    i_burn_peak = i_burn_pack + i_legit["hk"] + i_legit["heater"] + i_legit["mtq"]
    i_burn_mtq_lim = i_burn_peak - i_legit["mtq"] + MTQ_BRIDGE_LIMIT_A
    i_burn_buck_lim = 2.8 * 7.04 / (0.88 * 11.2)           # all four on: LMR43620 peak limit 2.8 A min holds the buck output
    fuse_rerate_60 = 0.80                                  # FM12 (MIL-PRF-23419/12) at 60 degC: assumed, to confirm on the slash sheet
    # prospective short-circuit current at BAT_TOP (fault right after F0): cells at a lower-bound 10 mOhm each (about a
    # third of the INR18650-35E's typical DC resistance), welded interconnects neglected, F0 at its 13.5 mOhm minimum
    r_cell_min, r_f0_min, r_tap_fuse_min = 0.010, 0.0135, 0.225
    i_psc_one = VBAT_MAX / (NCELL * r_cell_min + r_f0_min)
    r_f0b_min = 0.0126                                                # FM08A125V5A cold resistance minimum (NASA NEPP list)
    i_psc = VBAT_MAX / (NCELL * r_cell_min + 2 * r_f0_min + r_f0b_min)  # F0A, F0B (BAT_TOP) and F0N (BAT_BOT) in every loop
    i_psc_tap = VBAT_MAX / (NCELL * r_cell_min + r_tap_fuse_min)   # without the sense-board tap resistors (not fitted case)
    ptc_hold_60 = 2.6 * 0.74                               # 2920L hold current rerating at 60 degC
    ptc_hold_60_ = 2.6 * 0.74
    rows = [("Largest legitimate pack current (Jetson at the 12 W guard + PA 1 W + housekeeping + heater + magnetorquers "
             f"{MTQ_DETUMBLE_W:.2f} W, three rods saturated)", f"{i_max_legit:.2f} A", ""),
            (f"Same with the coil bridges at their hardware limit ({MTQ_BRIDGE_LIMIT_A:.2f} A, an OBC control fault)",
             f"{i_max_mtq_lim:.2f} A", f"{i_max_mtq_lim / ptc_hold_60_:.0%} of the RT1 hold at 60 degC: no trip"),
            (f"Same with SW11 in its current limit ({SW11_LIMIT_MAX_A:.2f} A max, VBAT_MTQ shorted; auto-retry 4 % duty)",
             f"{i_max_sw11:.2f} A", f"{i_max_sw11 / ptc_hold_60_:.0%} of the RT1 hold: no trip"),
            ("Single burn, normal sequence (one resistor, 0.88 A at +10 %) + housekeeping + heater + magnetorquers "
             f"{MTQ_DETUMBLE_W:.2f} W", f"{i_burn_peak:.2f} A", "chair limit 1.0 A"),
            (f"Same with the coil bridges at their hardware limit ({MTQ_BRIDGE_LIMIT_A:.2f} A)", f"{i_burn_mtq_lim:.2f} A", "chair limit 1.0 A"),
            ("Burn + transponder keyed at 1 W RF (2.42 W DC; not concurrent by design: RF silence to 45 min, burn at 30 min)",
             f"{i_burn_peak + i_legit['pa']:.2f} A", f"{i_burn_mtq_lim + i_legit['pa']:.2f} A with the coil bridges at their limit; BMS 4 A"),
            ("All four burn drivers on (software fault): burn buck in its 2.8 A current limit", f"{i_burn_buck_lim:.2f} A", "hardware cap"),
            ("AFE OCD1 (40 mV on 10 mOhm, 500 ms)", f"{ocd:.1f} A ({ocd_lo:.1f}-{ocd_hi:.1f})", f"{ocd_lo / i_max_legit:.1f}x the largest load; below F2"),
            ("AFE SCD (100 mV, 15 us)", f"{scd:.0f} A", "prospective short-circuit current 80-100 A typical, <= 314 A bound (below)"),
            ("F0A, F0N (sense board) and F2: FM12A72V5.0A (AEM, MIL-PRF-23419/12); F0B: FM08A125V5A (Littelfuse, /8), 5 A at 60 degC (x0.80, assumed)", f"{5 * fuse_rerate_60:.1f} A",
             f"largest load {i_max_legit / (5 * fuse_rerate_60):.0%}, four-driver fault {i_burn_buck_lim / (5 * fuse_rerate_60):.0%} of the rerated rating (rule 50 %)"),
            ("Prospective short-circuit current, short in W1 (16.8 V, 4 x 10 mOhm cells, F0A + F0B + F0N at 13.5 mOhm each)", f"{i_psc:.0f} A",
             f"FM12 (F0A, F0N) 1000 A at 72 VDC: {1000 / i_psc:.1f}x; FM08 (F0B) 300 A at 125 VDC: {300 / i_psc:.2f}x (F0A and F0N stay in every loop "
             f"while intact, so F0B never sees the {i_psc_one:.0f} A one-fuse bound)"),
            ("Tap short, tap to frame (one tap chain 3 x 68.1 Ohm + tap fuse, four cells)", f"{VBAT_MAX / 204.3 * 1e3:.0f} mA",
             f"{(VBAT_MAX / 204.3) ** 2 * 68.1:.2f} W per 2512 1 W resistor ({(VBAT_MAX / 204.3) ** 2 * 68.1:.0%}); the fuse need not open"),
            ("Tap short, tap to tap (two chains)", f"{VBAT_MAX / 408.6 * 1e3:.0f} mA", "28 AWG wire 0.90 A derated: no heating"),
            ("RT1 2920L260/33DR hold at 60 degC", f"{ptc_hold_60:.2f} A", f"{i_max_legit / ptc_hold_60:.0%} of hold"),
            ("F3 ITV4030L1412NR 12 A (secondary-protector fuse) at 60 degC (10 A)", "10 A", f"{i_max_legit / 10:.0%}")]
    table("4. Pack currents and protection coordination", ["Item", "Value", "Margin"], rows)
    # heaters of the three-terminal fuses
    rows = []
    for name, v in (("F3 at the secondary-protector trip (17.4 V)", VBAT_OV2), ("F1 EOL fire, pack at 14.0 V", 14.0),
                    ("F1 EOL fire, pack at 16.4 V", VBAT_CV)):
        p_lo, p_hi = v * v / 20.0, v * v / 11.2
        rows.append((name, f"{v / 20:.2f}-{v / 11.2:.2f} A", f"{p_lo:.1f}-{p_hi:.1f} W", "inside 6-35 W: cut < 1 min (datasheet); FET PMV40UN2 "
                     f"{(v / 11.2) ** 2 * 0.066 * 1e3:.0f} mW"))
    table("4a. Three-terminal fuse heaters (ITV4030L1412NR, R_heater 11.2-20.0 Ohm)", ["Case", "Heater current", "Heater power", "Check"], rows)
    R["protection"] = dict(ocd=ocd, i_max_legit=i_max_legit, i_burn_peak=i_burn_peak, i_burn_buck_lim=i_burn_buck_lim,
                           i_max_mtq_lim=i_max_mtq_lim, i_max_sw11=i_max_sw11, i_burn_mtq_lim=i_burn_mtq_lim)


# ================================================================ 5 inhibits
def inhibits():
    # DS1/DS3: P-FET IPD50P04P4L-11, Vgs from the 1M/1M divider; DS2: back-to-back BUK9Y4R4-40E, 1.2M/1.0M
    rows = []
    for name, rt, rb, c, vmin, vmax, lim in (("DS1 / DS3 high-side P-FET gate", 1.0e6, 1.0e6, 47e-9, VBAT_MIN, VBAT_MAX, 16.0),
                                            ("DS2 back-to-back N-FET gate", 1.3e6, 1.0e6, 47e-9, VBAT_MIN, VBAT_MAX, 10.0)):
        k = rb / (rt + rb)
        tau_on = par(rt, rb) * c
        tau_off = (rb if "DS2" in name else rt) * c
        rows.append((name, f"{rt / 1e6:.1f} M / {rb / 1e6:.1f} M, {c * 1e9:.0f} nF", f"{vmin * k:.1f}-{vmax * k:.1f} V ({vmax * k / lim:.0%} of {lim:.0f} V)",
                     f"{tau_on * 1e3:.1f} ms on / {tau_off * 1e3:.1f} ms off", f"{vmax / (rt + rb) * 1e6:.1f} uA when released, 0 when stowed"))
    table("5. Deployment-switch gate drives (>= 10 ms RC filters)", ["Inhibit", "Network", "|Vgs| (rating)", "Filter time constant", "Current"], rows)
    # time for the gate to reach threshold after release, P-FET Vth 2.2 V max at the minimum bus
    k = 0.5
    t_on = -par(1e6, 1e6) * 47e-9 * math.log(1 - 2.2 / (VBAT_MIN * k))
    print(f"\nA contact closure must last {t_on * 1e3:.1f} ms before a DS1/DS3 FET starts to conduct at the 11.2 V minimum "
          "(chatter shorter than this cannot power the bus).")
    # storage drain with the switches actuated (AFE in SHUTDOWN)
    drain = [("BQ76952 SHUTDOWN (incl. TS2 wake source)", 1.0, 3.1), ("BQ771806 secondary protector", 1.0, 2.0),
             ("BQ771806 cell inputs (4 x < 0.1 uA)", 0.1, 0.4), ("Q16 secondary-fuse FET leakage (PMV40UN2, 25 degC / 60 degC)", 0.05, 1.0),
             ("Q23 wake FET + DS2 gate network (contacts open)", 0.0, 0.1), ("Pack TVS D14 isolated by the open AFE FETs", 0.0, 0.0),
             ("AFE FETs Q1/Q2 off-state leakage to PACK_P (nothing returns)", 0.0, 0.0),
             ("Cell-tap filter capacitors (ceramic leakage, 9 x 0.1 uF)", 0.01, 0.1),
             ("Open-tap ladder 4 x 1.00 M across the cells (RID SMA-B03; 15.2 V / 16.8 V)", 3.8, 4.2),
             ("SMF24A TVS on UMB_VMON (access-port board A3, OBC item C18) through R 100 k from BAT_F", 0.1, 1.0)]
    tot_typ = sum(d[1] for d in drain)
    tot_max = sum(d[2] for d in drain)
    drain.append(("**Total**", round(tot_typ, 2), round(tot_max, 2)))
    table("5a. Storage drain with DS1, DS2, DS3 actuated and the AFE in SHUTDOWN (LVR-071 <= 20 uA)",
          ["Element", "Typical (uA)", "Worst (uA)"], drain)
    print(f"\nWorst {tot_max:.1f} uA = {tot_max / 20:.0%} of the 20 uA limit; 90 days at worst drain = {tot_max * 1e-3 * 90 * 24:.1f} mAh "
          f"({tot_max * 1e-6 * 90 * 24 / 3.45:.2%} of 3.45 Ah).")
    q47 = tot_max * 1e-3 * 47 * 24
    print(f"Margin {20 - tot_max:.1f} uA. Integration to separation (BASELINE section 4: 60 +/- 5 % SOC at L-47 d, >= 40 % at "
          f"separation): {q47:.1f} mAh in 47 days at the worst drain = {q47 / 3450:.2%} SOC, so >= {55 - q47 / 34.5:.1f} % SOC "
          "before the cells' own self-discharge (a cell property, outside this budget: 15 points of margin remain for it).")
    # array path with the trackers disabled (TRK_EN low: DS2 pole 2 or DS3 stowed): the boost passes the string through
    # its inductor and diode; no-load diode drop 0.15 V (PMEG40T30ER at uA), ideal diode FET ~0 V
    v_pass = STRING_VOC_MAX - 0.15
    en_rise_min = 1.16                                   # LMR43620 EN rising threshold minimum (data sheet)
    starts = [("U15 3V3_AO", 750e3), ("U16 5V_PA, U17 V_BURN", 887e3)]
    print(f"\nArray pass-through with the trackers disabled: <= {v_pass:.2f} V (Voc {STRING_VOC_MAX} V at -45 degC BOL less "
          "0.15 V). Lowest converter start: " + "; ".join(f"{n} {en_rise_min * (rt + 100e3) / 100e3:.1f} V" for n, rt in starts) +
          f"; eFuse U19 UVLO 12 V; every load switch needs an enable from 3V3_AO or the OBC, both unpowered. Margin "
          f">= {en_rise_min * 8.5 - v_pass:.1f} V: no rail, no RF, no deployment with any two of DS1-DS3 failed closed.")
    R["inhibits"] = dict(drain_typ_ua=tot_typ, drain_max_ua=tot_max, t_on_ms=t_on * 1e3, v_pass=v_pass)


# ================================================================ 6 load switches and converters
def switches():
    def tps2553(rk):
        return 23950 / rk ** 0.977 / 1e3, 25230 / rk ** 1.016 / 1e3, 22980 / rk ** 0.94 / 1e3
    rows = []
    for sw, load, rk in (("SW2 RADIO (3V3_AO)", 0.038, 226.0), ("SW3 PA (5V_PA), 1 W RF (RF board E)", 0.485, 26.1), ("SW4 ADCS sensors (3V3_AO, OBC J6)", 0.049, 88.7),
                         ("SW7 GNSS (3V3_AO)", 0.10, 133.0), ("SW8 AntS (3V3_AO)", 0.61, 18.7), ("SW9 EPS controller (3V3_AO)", 0.012, None),
                         ("SW10 OBC 3V3 (3V3_AO)", 0.040, 178.0)):
        if rk is None:
            nom, lo, hi = 0.075, 0.050, 0.100
            rs = "ILIM to IN"
        else:
            nom, lo, hi = tps2553(rk)
            rs = f"{rk:.1f} k"
        rows.append((sw, "TPS2553-Q1", rs, f"{nom:.3f} ({lo:.3f}-{hi:.3f}) A", f"{load:.3f} A", f"{lo / load:.1f}x"))
    vcl, kcl = 1.233, 2000
    for sw, load, rcl, acc in (("SW1 PAYLOAD (VBAT)", 0.93, 1.24e3, 0.14), ("SW6 DEPLOY (VBAT)", 1.25, 1.24e3, 0.14)):
        il = vcl * kcl / rcl
        rows.append((sw, "TPS1H100B-Q1", f"{rcl / 1e3:.2f} k", f"{il:.2f} ({il * (1 - acc):.2f}-{il * (1 + acc):.2f}) A", f"{load:.2f} A", f"{il * (1 - acc) / load:.1f}x"))
    il = 0.8 * 600 / 3.24e3
    rows.append(("SW5 HEATER (VBAT)", "TPS1H000-Q1", "3.24 k", f"{il:.3f} ({il * 0.85:.3f}-{il * 1.15:.3f}) A", f"{VBAT_CV / 207:.3f} A", f"{il * 0.85 / (VBAT_CV / 207):.1f}x"))
    lo, hi = SW11_LIMIT * (1 - SW11_ACC) * 0.99, SW11_LIMIT_MAX_A
    rows.append(("SW11 MTQ bridges (VBAT, OBC J3-36/39)", "TPS1H000-Q1", "1.33 k", f"{SW11_LIMIT:.3f} ({lo:.3f}-{hi:.3f}) A",
                 f"{MTQ_BRIDGE_LIMIT_A:.3f} A (bridge limit; {MTQ_DETUMBLE_W / VBAT_MIN:.3f} A at {MTQ_DETUMBLE_W} W)",
                 f"{lo / MTQ_BRIDGE_LIMIT_A:.1f}x"))
    R["sw11"] = dict(nom=SW11_LIMIT, lo=lo, hi=hi)
    il = 18.0 / 15.0
    rows.append(("U3 Jetson eFuse (VBAT)", "TPS16630 (latch-off)", "15.0 k", f"{il:.2f} ({il * 0.92:.2f}-{il * 1.08:.2f}) A", "0.93 A (12 W guard)", f"{il * 0.92 / 0.93:.2f}x"))
    table("6. Load switches: current limits (datasheet equations)", ["Switch", "Part", "R_ILIM / R_CL", "Limit nominal (min-max)", "Peak load", "Min / load"], rows)
    print("\nThe Jetson eFuse minimum (1.10 A) sits above the 12 W flight-software guard (0.93 A); the OBC trip stays the "
          "first line (payload_guard.rs), the eFuse the hardware backstop.\nTPS2553 rails (RID AV-08): the current is limited "
          "within 2 us; FAULT asserts after the 8 ms deglitch and pulls EN low, which removes the rail; it returns when "
          "C_RETRY recharges through R_FAULT to the EN threshold: off time ~0.6 x 100 k x C_RETRY = 1.3 s with 22 uF "
          "(radio, GNSS: >= 1 s) and 0.28 s with 4.7 uF (OBC, EPS controller, sensors, PA, AntS: >= 100 ms). A persisting "
          "latch-up repeats the cycle at that rate (the load sees <= 8 ms of limited current per cycle). The OBC rail SW10 "
          "needs no command: it is the power-cycling supervisor the OBC cannot be for itself.")
    print(f"\nSW11 (SQSO-ICX-001 E3: limit >= 0.25 A min, <= 0.5 A max): {lo:.3f}-{hi:.3f} A meets both. Auto-retry "
          "(DELAY pulled up): a sustained overload runs 40 ms on / 1 s off (3.8 % duty), so a short at the OBC draws "
          f"{hi * 0.04 / 1.04 * 1e3:.0f} mA average and the 28 AWG W3 wires carry {hi / 2:.2f} A peak each for 40 ms. "
          f"Drop at {MTQ_BRIDGE_LIMIT_A} A: {MTQ_BRIDGE_LIMIT_A * 1.0:.2f}-{MTQ_BRIDGE_LIMIT_A * 2.0:.2f} V (1-2 Ohm, 25-150 degC).")
    print("3V3_RTC (E4): 1.0 k from 3V3_AO; at the RTC's <= 2 uA the drop is 2 mV (>= 3.23 V); a short at J3-38 draws "
          "3.4 mA and 11 mW in the 0.1 W resistor (11 %).")
    # converters
    rows = []
    for name, vo, io, vin in (("U15 3V3_AO", 3.3, 0.30, VBAT_MAX), ("U16 5V_PA", 5.0, 0.54, VBAT_MAX), ("U17 V_BURN 7.0 V", 7.04, 1.76, VBAT_MAX)):
        d = vo / vin
        ripple = (vin - vo) * d / (6.8e-6 * 1.0e6)
        p_cond = io ** 2 * (0.26 * d + 0.075 * 1.6 * (1 - d))
        p_sw = 0.5 * vin * io * 8e-9 * 1e6 * 2 + 0.008
        p_l = (io ** 2 + ripple ** 2 / 12) * 0.048
        p = p_cond + p_sw + p_l
        eff = vo * io / (vo * io + p)
        rows.append((name, f"{vo:.2f} V / {io:.2f} A", f"{ripple:.2f} A p-p", f"{p * 1e3:.0f} mW", f"{eff:.1%}"))
    table("6a. Bucks (LMR43620-Q1, 1.000 MHz, 6.8 uH, worst VIN 16.8 V)", ["Converter", "Output", "Inductor ripple", "Loss at full load", "Efficiency"], rows)
    print("\nFirst-order loss model (conduction at the maximum Rds(on), 8 ns edges, 8 mW drive and quiescent); forced PWM at "
          "light load adds switching and core loss the model does not hold: expect 85-90 % for 3V3_AO at 0.1 A (PDU_DESIGN "
          ">= 85 %), to be measured on the EM (V-PWR-3).")
    # UVLO dividers (LMR43620 EN: 1.23 V rising, 0.35 V hysteresis)
    rows = []
    for name, rt, rb in (("3V3_AO", 750e3, 100e3), ("5V_PA, V_BURN", 887e3, 100e3)):
        vr = 1.23 * (rt + rb) / rb
        vf = 0.88 * (rt + rb) / rb
        rows.append((name, f"{rt / 1e3:.0f} k / {rb / 1e3:.0f} k", f"{vr:.1f} V", f"{vf:.1f} V"))
    table("6b. Converter under-voltage lock-out (EN/UVLO dividers)", ["Converter", "Divider", "Start (rising)", "Stop (falling)"], rows)
    print("\nRule E8 asks for 10.8 V falling; the LMR43620's fixed 0.35 V EN hysteresis gives a rise/fall ratio of 1.40, so a "
          "10.8 V fall would mean a 15.1 V start, which would keep the rails off after an over-discharge. The stops are set "
          "below the 11.2 V BMS cut (the intent of E8: the BMS, not a converter, decides) with starts at 10.5 V and 12.1 V.")


# ================================================================ 7 deployment drivers
def burn():
    vref = 2.500
    rt, rb = 11.5e3, 1.00e3
    vset = vref * rb / (rt + rb)
    rsense = 0.250
    i = vset / rsense
    err = math.sqrt(0.1 ** 2 + 0.4 ** 2 + 0.2 ** 2 + 1.0 ** 2 + (1.0e-3 / vset * 100) ** 2) * 1.0
    err_wc = 0.1 + 0.4 + 0.2 + 1.0 + 1.0e-3 / vset * 100
    vburn = 7.04
    rows = []
    for rburn in (4.0, 4.5, 5.0, 5.5, 6.0, 7.0):
        for vb in (vburn * 0.97, vburn * 1.03):
            imax = i * (1 + err_wc / 100)
            v_fet = vb - imax * (rburn + rsense) - 0.88 * 0.060 - 0.05
            p_fet = imax * max(v_fet, 0)
            rows.append((f"{rburn:.1f}", f"{vb:.2f}", f"{v_fet:.2f}", f"{p_fet:.2f}", "regulates" if v_fet > 0.15 else "drop-out"))
    table("7. Burn drivers (OPA2991-Q1 + BUK9M14-40E + 0.25 Ohm, from the 7.0 V burn buck)",
          ["Burn resistor (Ohm)", "V_BURN (V)", "FET Vds (V)", "FET loss (W)", "State"], rows)
    print(f"\nSet point {vset * 1e3:.1f} mV -> {i:.3f} A; worst-case error +/-{err_wc:.1f} % (REF 0.1, gate VOH 0.4, divider 0.2, "
          f"shunt 1.0, op-amp offset {1.0e-3 / vset * 100:.1f}) inside the +/-10 % requirement; RSS {err:.1f} %. Regulates for burn "
          "resistors up to 6.9 Ohm at the low V_BURN. FET loss <= 3.0 W for <= 12 s (4.0 Ohm resistor at the high V_BURN).")
    # TPL5111 resistance for tIP = 12.05 s (DRVn = tIP - 50 ms = 12.0 s), coefficient set 3
    a, b, c = 0.1972, -19.3450, 692.1201
    t = 12.05
    rext = 100 * (-b + math.sqrt(b * b - 4 * a * (c - 100 * t))) / (2 * a)
    t2 = 0.1
    a1, b1, c1 = 0.2253, -20.7654, 570.5679
    print(f"TPL5111 one-shot: R_EXT = {rext / 1e3:.2f} kOhm for tIP {t} s (DRVn high {t - 0.05:.2f} s); fitted with "
          f"11.8 k (0.1 %) in series with 182 Ohm = {11.8 + 0.182:.3f} k. Accuracy: setting 0.6 %, oscillator 0.5 % + 400 ppm/K x 70 K = 2.8 %, resistors 1 % "
          f"-> 12.0 s +/- 0.6 s. Relay pulsers: tIP 100 ms (R_EXT 500 Ohm) -> 50 ms coil pulse.")
    # wing hold-down energy: 0.8 A^2 x 5 Ohm x 8 s
    e = 0.8 ** 2 * 5.0 * 8.0
    print(f"Energy into one 5 Ohm resistor in the 8 s burn: {e:.1f} J (SIL release mean 6 J at 20 degC; MECHANISMS_ANALYSIS ratio 1.67).")
    R["burn"] = dict(vset=vset, i=i, err_wc=err_wc, rext=rext)


# ================================================================ 8 heater, NTC thresholds, load shed
def heater_ntc_shed():
    rh = 207.0
    rows = [(f"{v:.1f}", f"{v / rh * 1e3:.0f}", f"{v * v / rh:.2f}") for v in (12.0, 14.0, VBAT_CV)]
    table("8. Battery heater (207 Ohm film)", ["VBAT (V)", "Current (mA)", "Power (W)"], rows)
    print(f"\nF4 0.25 A fuse: {VBAT_CV / rh / 0.25:.0%} of rating at 16.4 V (rule 50 %). SW5 current limit 0.148 A stops a "
          "partial short before F4; F4 clears a hard short if SW5 fails short.")
    # NTC1 ladder (VHW-referenced, 10 k bias): thresholds as fractions of VHW
    rows = []
    for name, t in (("COLD (charge inhibit below)", 1.0), ("HOT (charge inhibit above)", 44.0), ("HEATER CUTOUT (open above)", 40.0),
                    ("PACK OVER-TEMPERATURE (both AFE FETs open above)", 60.0)):
        rn = ntc_r(t)
        frac = rn / (rn + 10e3)
        rows.append((name, f"{t:.0f}", f"{rn / 1e3:.2f}", f"{frac:.4f}"))
    table("8a. NTC1 thresholds (10 k, B 3435 K; ratiometric to VHW)", ["Comparator", "T (degC)", "R_NTC (k)", "V_NTC / VHW"], rows)
    # ladder: VHW - Ra - n1 - Rb - n2 - Rc - n3 - Rd - n4 - Re - GND
    fr = [ntc_r(t) / (ntc_r(t) + 10e3) for t in (1.0, 40.0, 44.0, 60.0)]
    # choose total 1.000 M; node fractions fr[0]>fr[1]>fr[2]>fr[3]
    total = 1.0e6
    ra = (1 - fr[0]) * total
    rb_ = (fr[0] - fr[1]) * total
    rc = (fr[1] - fr[2]) * total
    rd = (fr[2] - fr[3]) * total
    re = fr[3] * total
    chosen = (267e3, 365e3, 31.6e3, 104e3, 229e3)
    tot = sum(chosen)
    nodes = [1 - chosen[0] / tot, 1 - sum(chosen[:2]) / tot, 1 - sum(chosen[:3]) / tot, 1 - sum(chosen[:4]) / tot]
    def t_of(frac):
        lo, hi = -40.0, 100.0
        for _ in range(60):
            mid = (lo + hi) / 2
            f = ntc_r(mid) / (ntc_r(mid) + 10e3)
            lo, hi = (mid, hi) if f > frac else (lo, mid)
        return (lo + hi) / 2
    print("\nThreshold ladder, exact: " + ", ".join(f"{x / 1e3:.1f} k" for x in (ra, rb_, rc, rd, re)) +
          "; fitted to E192 0.1 % values " + ", ".join(f"{x / 1e3:.1f} k" for x in chosen) + " -> thresholds " +
          ", ".join(f"{t_of(fr):.1f} degC" for fr in nodes) + " (targets +1, 40, 44, 60 degC: the charge window sits 1 K inside 0-45 degC). Ladder and NTC bias 0.1 %, "
          "NTC 1 % (+/-0.25 K), comparator offset and 6.5 mV internal hysteresis (+/-0.3 K): each threshold within +/-1 K.")
    # load shed: VBAT/(1.00 M, 221 k) vs REF_A 2.500 V; hysteresis 4.12 M from the divider node to the open-drain
    # output (pulled up to 3.3 V by 10 k): output high (bus OK) lifts the node -> falling threshold; output low -> rising
    vref = 2.500
    r1, r2, rh, vpu = 1.00e6, 221e3, 4.12e6, 3.3
    g = 1 / r1 + 1 / r2 + 1 / rh
    vth = r1 * (vref * g - vpu / rh)             # falling (output high)
    vth_rise = r1 * vref * g                     # rising (output low)
    tau = 4.75e6 * 1.0e-6
    v0 = 3.3 - 0.2                               # timing node starts one BAT54 drop below 3.3 V
    t_shed = tau * math.log(v0 / 1.100)
    rows = [("Shed (falling)", f"{vth:.2f} V", "VBAT < 13.6 V"), ("Release (rising)", f"{vth_rise:.2f} V", "VBAT > 14.4 V"),
            ("Delay before shedding", f"{t_shed:.1f} s (4.75 M x 1.0 uF from {v0:.1f} V to the 1.100 V threshold)",
             "5 s; X7R +/-15 % and resistor 1 % -> 4.2-5.7 s"),
            ("Release delay", "< 1 ms (BAT54 charges the timing node)", "")]
    table("8b. Hardware load shed (SW1 and U19, Jetson; the PA is not shed)", ["Item", "Value", "Requirement"], rows)
    R["shed"] = dict(vth=vth, vth_rise=vth_rise, t_shed=t_shed, ladder=[ra, rb_, rc, rd, re])


# ================================================================ 9 frequency plan
def freq_plan():
    f0 = 1.000e6
    ppm = 50e-6
    windows = []
    lo, hi = 435.000e6, 437.300e6
    n = math.ceil(lo / f0)
    while n * f0 <= hi + f0:
        windows.append((n * f0 + 0.2e6 + n * f0 * ppm, (n + 1) * f0 - 0.2e6 - (n + 1) * f0 * ppm))
        n += 1
    w = [(max(a, lo), min(b, hi)) for a, b in windows if min(b, hi) > max(a, lo)]
    print("\n### 9. Converter frequency plan\n")
    print("All seven converters run at 1.000 MHz +/-50 ppm (Epson SG-8018CA 4 MHz divided by 4 in a SN74LVC2G74), trackers "
          "T1/T3 and the bucks on Q, T2/T4 on Q-bar. Harmonics fall on n x 1.000 MHz +/- 22 kHz at n = 435-437. Uplink "
          "channels keeping >= 200 kHz from every harmonic (rule E1):")
    for a, b in w:
        print(f"- {a / 1e6:.3f} - {b / 1e6:.3f} MHz")
    R["freq_windows_mhz"] = [(a / 1e6, b / 1e6) for a, b in w]


# ================================================================ 10 telemetry scaling
def telemetry():
    rows = [("VBATT_SNS (PC2)", "PACK_P, 1.00 M / 200 k (0.1 %), 0.1 uF", "x6.0", f"{VBAT_MAX / 6:.2f} V at 16.8 V", "adc.rs Linear(6.0, 0)"),
            ("IBATT_SNS (PC3)", "INA186A2 x50 on 10 mOhm, REF 1.65 V", "0.5 V/A, 1.65 V at 0 A", "+/-3.3 A", "adc.rs Linear(2.0, -3.3) unchanged"),
            ("VSOLAR_SNS (PC4)", "highest string (VCTL_IN), 300 k / 100 k", "x4.0", f"{STRING_VOC_MAX / 4:.2f} V at Voc max", "unchanged"),
            ("ISOLAR_SNS (PC5)", "INA186A2 x50 on 20 mOhm array shunt (tracker output side)", "1.0 V/A", "0-3.3 A", "unchanged; current at bus voltage"),
            ("VBUS_SNS (PB0)", "5V_PA, 100 k / 100 k", "x2.0", "2.5 V at 5.0 V", "unchanged"),
            ("VJET_RB (PA7, J3-19)", "VBAT_JET after the eFuse U19, 1.00 M / 200 k (0.1 %), 0.1 uF", "x6.0",
             f"{VBAT_MIN / 6:.2f}-{VBAT_MAX / 6:.2f} V on, < 0.1 V off", "new HalAdcChannel (RID AV-06)"),
            ("VSW1_RB (PA1, J3-37)", "VBAT_SW1 after SW1, 1.00 M / 200 k (0.1 %), 0.1 uF", "x6.0",
             f"{VBAT_MIN / 6:.2f}-{VBAT_MAX / 6:.2f} V on, < 0.1 V off", "new HalAdcChannel (RID AV-06)")]
    table("10. Analog telemetry to the OBC ADC (W3)", ["Signal (OBC pin)", "Source", "Scale", "Full scale", "Firmware"], rows)


# ================================================================ 11 thermal
def thermal():
    t = R["trackers"]["cases"]
    worst_trk = t["T1 wing fronts, cold sunrise BOL (worst power)"]
    parts = [
        # name, steady W (worst operating case), transient W (and duration), psi/theta J-board K/W, spreading K/W
        ("U4 tracker T1 (TPS55340-Q1) IC", worst_trk["p_ic"], None, 14.5, 15.0),
        ("D9 T1 output Schottky (PMEG40T30ER-Q)", worst_trk["p_diode"], None, 30.0, 20.0),
        ("L1 T1 inductor (XGL4030-682)", worst_trk["p_l"], None, 20.0, 15.0),
        ("U15 3V3_AO buck (LMR43620)", 0.060, None, 25.9, 15.0),
        ("U16 5V_PA buck (LMR43620) at 1 W RF", 0.150, None, 25.9, 15.0),
        ("U17 burn buck (LMR43620), 2 drivers on (fault)", 0.0, (0.95, 12.0), 25.9, 15.0),
        ("Q24 burn driver FET (BUK9M14-40E), 4.0 Ohm resistor", 0.0, (3.0, 12.0), 1.5 + 2.0, 12.0),
        ("RS20 burn shunt 0.25 Ohm", 0.0, (0.19, 12.0), 30.0, 20.0),
        ("U20 SW1 Jetson switch (TPS1H100) at 0.93 A", 0.93 ** 2 * 0.200, None, 24.0, 15.0),
        ("U19 Jetson eFuse (TPS16630) at 0.93 A", 0.93 ** 2 * 0.050, None, 10.2, 15.0),
        ("Q1+Q2 AFE FETs (BUK7Y3R5-40H) at 1.3 A", 1.3 ** 2 * 0.006 * 2, None, 1.5, 15.0),
        ("Q4+Q5 DS1/DS3 P-FETs at 1.3 A", 1.3 ** 2 * 0.016 * 2, None, 1.0, 15.0),
        ("Q6+Q7 DS2 N-FETs at 1.3 A", 1.3 ** 2 * 0.006 * 2, None, 1.5, 15.0),
        ("Shunts (AFE, gauge, array) at 1.3 A", 1.3 ** 2 * 0.020 + 0.5 ** 2 * 0.020, None, 30.0, 20.0),
        ("U1 BQ76952 (NORMAL, balancing one cell 9 mA)", 0.016 * 0.3 + 4.1 * 0.009 * 0.5, None, 29.3, 15.0),
        ("U21 heater switch (TPS1H000) at 79 mA", 0.079 ** 2 * 1.6, None, 30.0, 15.0),
        ("U39 SW11 MTQ switch (TPS1H000) at the bridges' 0.23 A limit", MTQ_BRIDGE_LIMIT_A ** 2 * 1.6, None, 30.0, 15.0),
        ("Q16 F3 heater FET at 1.55 A (one event, < 1 min)", 0.0, (1.55 ** 2 * 0.066, 60.0), 60.0, 20.0),
        ("Q20+Q21 EOL fire FETs at 1.46 A (one event)", 0.0, (1.46 ** 2 * 0.066, 60.0), 60.0, 20.0),
        ("U37 EPS controller + 4 INA3221 + 2 INA228 + logic", 0.040, None, 30.0, 15.0),
        ("Y1 oscillator + clock divider", 0.012, None, 40.0, 15.0),
        ("Ideal diodes (8 x LM74700 + FET) at Isc", 8 * R["solar"]["p_ideal_diode_w"], None, 40.0, 15.0),
    ]
    rows = []
    tot = 0.0
    for name, p, tr, psi, spread in parts:
        tot += p
        tj = T_BOARD_HOT + p * (psi + spread)
        trs = ""
        if tr:
            ptr, dur = tr
            # transient: board-local rise limited by the spreading capacitance; use psi + 0.6 x spreading at 8-12 s
            tj_tr = T_BOARD_HOT + ptr * (psi + 0.6 * spread)
            trs = f"{ptr:.2f} W for {dur:.0f} s -> Tj {tj_tr:.0f} degC"
            tj = max(tj, tj_tr)
        rows.append((name, f"{p * 1e3:.0f}", trs, f"{tj:.0f}"))
    table("11. Dissipation and junction temperatures (board at +60 degC, vacuum: conduction to the board only)",
          ["Part", "Steady worst (mW)", "Transient", "Tj worst (degC)"], rows)
    # board totals by case
    trk_all = sum(boost_loss(p, v, VBAT_CV)["p_tot"] for p, v in ((ARRAY_P_MAX * 4 / 6, 5.0), (ARRAY_P_MAX * 2 / 6, 5.0)))
    hk = 0.060 + 0.040 + 0.012 + 0.05
    i_mtq = MTQ_DETUMBLE_W / VBAT_MIN
    p_mtq = i_mtq ** 2 * 1.6                       # SW11 at the detumble load
    active = trk_all + hk + 0.150 + 0.93 ** 2 * 0.25 + (1.3 + i_mtq) ** 2 * 0.06 + 8 * R["solar"]["p_ideal_diode_w"] + p_mtq
    eclipse = hk + 0.150 + 0.9 ** 2 * 0.05 + (0.1 + i_mtq) ** 2 * 0.06 + p_mtq
    burn = hk + 0.30 + 3.0 + 0.19 + 0.1 + p_mtq
    print(f"\nBoard totals: sunlit ACTIVE at the maximum array power with the Jetson and PA on {active:.2f} W; eclipse "
          f"housekeeping {eclipse:.2f} W; deployment burn (worst driver) {burn:.2f} W for 12 s; orbit average in ACTIVE "
          f"sun-pointed about {0.62 * (trk_all * 0.6 + hk + 0.05) + 0.38 * eclipse:.2f} W.")
    R["thermal"] = dict(active_w=active, eclipse_w=eclipse, burn_w=burn, steady_sum_w=tot)


def main():
    print("# SQSO-EPS-BRD-001 calculations (generated by eps_calcs.py)")
    solar()
    trackers()
    charge_loops()
    protection()
    inhibits()
    switches()
    burn()
    heater_ntc_shed()
    freq_plan()
    telemetry()
    thermal()
    if "--json" in sys.argv:
        path = sys.argv[sys.argv.index("--json") + 1]
        with open(path, "w") as f:
            json.dump(R, f, indent=1, default=lambda o: o if not isinstance(o, dict) else dict(o))


if __name__ == "__main__":
    main()
