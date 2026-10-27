#!/usr/bin/env python3
"""
Launch and LEOP feasibility numbers for the CDR (launch, mission integration and ConOps review).

Each block prints its inputs, its source and its result, and the whole set is written as JSON:

  1. Battery state of charge from integration to separation: self-discharge (the SIL's model,
     `sqso-sil/src/env/battery.rs`: 1.5 % a month at 25 °C, doubling every 10 °C) plus a parasitic drain from
     the battery-side electronics that the separation switches do not disconnect (CDS Rev 14.1 §2.3.1.2 allows
     powered battery protection). The parasitic current is not designed yet, so it is swept.
     The result is the longest storage that keeps the separation SOC above the floors the SIL found
     (`sqso_sil --scenario commissioning --set integration_soc=...`: below 25 % the spacecraft boots into SAFE;
     below 15 % into EMERGENCY, which sheds the beacon for 2–4 h and delays the detumble past 3 orbits).
  2. Energy from separation to first sunlight in the worst case (separation at eclipse entry), from the SIL's
     measured pre-RF load, and the deployment burns with every retry.
  3. Deployment velocity from a 1U slot of an Exolaunch EXOpod (spring energy and accelerated masses from the
     EXOpod User Manual, Rev 6.0, 2022-09-06, Tables 3 and 4) and the SpaceX transmitter turn-on delay
     (Rideshare Payload User's Guide v11, Aug 2026, Table 4-10).
  4. Ascent venting: smallest vent area that meets each provider's volume-to-vent-area limit.
  5. Random vibration: overall Grms of the SpaceX MPE and the NRCSD profiles, recomputed from the breakpoints.
  6. Mass margin against each dispenser's 1U limit.
  7. Hard-body radius to report to SpaceX for the appendage deployments in the first 24 h (UG v11 §3.3.4).
  8. The deadline for the first operator ephemeris (UG v11 §9.5.7.2) from the first-contact statistics of
     `leop_passes.py`.

    python3 leop_feasibility.py --out <dir> [--passes <dir with passes_summary.json>]
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

import numpy as np

CAP_AH = 3.40                 # INR18650-35E pack (4S1P), SIL start capacity 3.406 Ah
V_NOM = 14.4
SELF_DIS_25C_PER_MONTH = 0.015
MONTH_D = 30.0


def storage(soc0: float, months: float, temp_c: float, i_par_ua: float) -> float:
    k = SELF_DIS_25C_PER_MONTH * 2.0 ** ((temp_c - 25.0) / 10.0)
    par = i_par_ua * 1e-6 * months * MONTH_D * 24.0 / CAP_AH
    return soc0 - k * months - par


def max_months(soc0: float, floor: float, temp_c: float, i_par_ua: float) -> float:
    k = SELF_DIS_25C_PER_MONTH * 2.0 ** ((temp_c - 25.0) / 10.0) + i_par_ua * 1e-6 * MONTH_D * 24.0 / CAP_AH
    return (soc0 - floor) / k


def psd_grms(points: list[tuple[float, float]]) -> float:
    """Overall Grms of a piecewise log-log PSD."""
    total = 0.0
    for (f1, a1), (f2, a2) in zip(points[:-1], points[1:]):
        if a1 == a2:
            total += a1 * (f2 - f1)
            continue
        m = math.log10(a2 / a1) / math.log10(f2 / f1)
        if abs(m + 1.0) < 1e-9:
            total += a1 * f1 * math.log(f2 / f1)
        else:
            total += a1 / (f1 ** m) / (m + 1.0) * (f2 ** (m + 1.0) - f1 ** (m + 1.0))
    return math.sqrt(total)


def turn_on_delay(eirp_w: float, v_mps: float) -> float:
    """UG v11 Table 4-10, interpolated (log EIRP, log velocity)."""
    eirp = np.array([0.001, 0.01, 0.1, 1.0, 10.0, 20.0, 100.0, 1000.0])
    vel = np.array([0.3, 0.5, 1.0, 2.0, 5.0])
    tab = np.array([[1, 2, 6, 19, 58, 82, 183, 578],
                    [1, 2, 4, 11, 35, 49, 110, 347],
                    [1, 1, 2, 6, 18, 25, 55, 174],
                    [1, 1, 1, 3, 9, 13, 28, 87],
                    [1, 1, 1, 2, 4, 5, 11, 35]], dtype=float)
    le, lv = np.log10(eirp), np.log10(vel)
    x, y = math.log10(eirp_w), math.log10(min(max(v_mps, 0.3), 5.0))
    per_v = [np.interp(x, le, row) for row in tab]
    return float(np.interp(y, lv, per_v))


def run(out: Path, passes_dir: Path | None) -> dict:
    res: dict = {}

    # 1. Storage
    floors = {"SAFE boot (25 %)": 0.25, "proposed separation floor (40 %)": 0.40, "EMERGENCY boot (15 %)": 0.15}
    table = []
    for soc0 in (0.60, 0.50, 0.30):
        for temp in (20.0, 25.0, 30.0):
            for ipar in (0.0, 10.0, 50.0, 100.0):
                row = {"soc0": soc0, "temp_c": temp, "i_parasitic_uA": ipar,
                       "soc_after_47d": round(storage(soc0, 47 / 30, temp, ipar), 3),
                       "soc_after_6mo": round(storage(soc0, 6, temp, ipar), 3),
                       "soc_after_12mo": round(storage(soc0, 12, temp, ipar), 3)}
                for k, f in floors.items():
                    row["months_to_" + k] = round(max_months(soc0, f, temp, ipar), 1)
                table.append(row)
    res["storage"] = {
        "model": "self-discharge 1.5 %/month at 25 C doubling per 10 C (SIL battery.rs) + parasitic current",
        "check_against_sil": {"sil_storage_days": 47, "sil_soc": [0.599, 0.583],
                              "model_soc_after": round(storage(0.599, 47 / 30, 20.0, 0.0), 3)},
        "rows": table,
    }

    # 2. Separation to first sunlight, burns
    load_pre_rf_w = 0.426       # SIL commissioning run, mean load before the burns (timeseries.csv)
    dark_max_min = 28.8         # leop_passes.py at 475 km from 2028-11-01: longest darkness after separation
    # Wing burns (SQSO-EPS-BRD-001 §9, BASELINE Rev E §4): two resistors per wing on four independent lines, one
    # resistor at a time, 0.8 A driver fed from the 7.0 V burn buck: 0.51 A at the pack for the burn itself, 0.67 A
    # with housekeeping and heater, 0.75 A with the magnetorquers; 8 s per resistor, 12 s hardware on-time limit.
    # The ISIS AntS burns its own thermal knives from 3V3_ANTS after the wings (not included here).
    burn_a = 0.51               # pack current of one burn (the driver's share)
    burn_pack_a = 0.67          # peak pack current with housekeeping and heater
    burn_pack_worst_a = 0.75    # with the magnetorquers at 1.16 W
    v_batt = 15.1               # pack voltage at 58 % SOC
    burn_s_nominal = 2 * 8.0    # resistor A of each wing, 8 s
    burn_s_worst = 4 * 12.0     # both resistors of both wings at the 12 s one-shot limit
    e_dark_wh = load_pre_rf_w * dark_max_min / 60.0
    e_burn_nom_wh = burn_a * v_batt * burn_s_nominal / 3600.0
    e_burn_worst_wh = burn_a * v_batt * burn_s_worst / 3600.0
    pack_wh = CAP_AH * V_NOM
    res["leop_energy"] = {
        "pre_rf_load_w": load_pre_rf_w, "darkness_max_min": dark_max_min,
        "energy_dark_wh": round(e_dark_wh, 3), "soc_drop_dark_pct": round(100 * e_dark_wh / pack_wh, 2),
        "burn_current_a": burn_a, "burn_pack_peak_a": burn_pack_a, "burn_pack_worst_a": burn_pack_worst_a,
        "burn_c_rate": round(burn_pack_worst_a / CAP_AH, 2),
        "high_discharge_alarm_a": 2.5, "bms_overcurrent_a": 4.0,
        "energy_burns_nominal_wh": round(e_burn_nom_wh, 4), "energy_burns_all_retries_wh": round(e_burn_worst_wh, 4),
        "soc_drop_burns_all_retries_pct": round(100 * e_burn_worst_wh / pack_wh, 3),
        "pack_wh": round(pack_wh, 1),
    }

    # 3. EXOpod 1U deployment velocity and the transmitter turn-on delay
    e_spring = 2.19            # J, 1U spring, +/- 20 %
    m_spring, n_spring, m_wagon, m_adapter = 0.15, 1, 0.245, 0.555
    vel = {}
    for m in (1.51, 1.69, 1.92, 2.00):
        for f in (0.8, 1.0, 1.2):
            v = math.sqrt(2 * e_spring * f / (m + 0.5 * n_spring * m_spring + m_wagon + m_adapter))
            vel[f"m={m} kg, E x{f}"] = round(v, 2)
    eirp_w = 1.0 * 10 ** (2.15 / 10)   # 1 W into a monopole of about 2.15 dBi
    res["separation"] = {
        "exopod_1u_velocity_mps": vel,
        "eirp_w_max": round(eirp_w, 2),
        "ug_table_4_10_min_delay_s": {f"v={v} m/s": round(turn_on_delay(eirp_w, v), 1) for v in (0.5, 1.0, 1.2, 2.0)},
        "smart_qso_first_rf_s": 2700,
        "ug_3_3_4_attitude_and_appendage_delay_s": 120,
        "smart_qso_first_magnetorquer_s": 1, "smart_qso_first_burn_s": 1800,
    }

    # 4. Venting: volume / vent area <= L
    v_gross = 0.100 * 0.100 * 0.1135      # m^3, 1U envelope
    limits = {"CDS Rev 14.1 §2.1.9 (50.8 m)": 50.8, "SpaceX UG v11 §4.1.8 (400 m)": 400.0,
              "NRCSD IDD §4.4.2 (MEVR 5080 cm)": 50.8}
    res["venting"] = {"gross_volume_m3": round(v_gross, 6),
                      "min_vent_area_mm2": {k: round(v_gross / L * 1e6, 1) for k, L in limits.items()},
                      "note": "every enclosed sub-volume (battery box, Jetson enclosure, connector backshells) "
                              "must meet the same ratio on its own"}

    # 5. Random vibration
    spacex = [(20, 0.01), (50, 0.015), (700, 0.015), (800, 0.03), (925, 0.03), (2000, 0.00644)]
    soft = [(20, 0.04), (25, 0.04), (31.5, 0.04), (40, 0.04), (50, 0.04), (63, 0.0449), (80, 0.05062),
            (100, 0.0566), (125, 0.062), (160, 0.062), (200, 0.062), (250, 0.05558), (315, 0.04102),
            (400, 0.02998), (500, 0.02236), (630, 0.01651), (800, 0.01206), (1000, 0.009), (1250, 0.006034),
            (1600, 0.003878), (2000, 0.0026)]
    hard = [(20, 0.057), (153, 0.057), (190, 0.099), (250, 0.099), (750, 0.055), (2000, 0.018)]
    g = psd_grms(spacex)
    res["random_vibration_grms"] = {
        "spacex_mpe": round(g, 2), "spacex_mpe_plus_3db_qual": round(g * math.sqrt(2), 2),
        "nrcsd_soft_stow": round(psd_grms(soft), 2), "nrcsd_hard_mount": round(psd_grms(hard), 2),
        "stated": {"spacex_mpe": 5.57, "nrcsd_soft_stow": 5.76, "nrcsd_hard_mount": 9.47},
    }

    # 6. Mass
    masses = {"BASELINE CBE": 1.51, "MEL CBE": 1.69, "MEL predicted low": 1.92, "MEL predicted high": 2.00,
              "BASELINE MEV": 1.75}
    limits_kg = {"CDS Rev 14.1 Table 1": 2.00, "EXOpod Nova 1U (UM Rev 1.2 Table 1)": 2.50,
                 "NRCSD IDD Table 4.1.2-1": 2.40, "JAXA J-SSOD (not recommended)": 1.33}
    res["mass_margin_pct"] = {lk: {mk: round(100 * (lv - mv) / lv, 1) for mk, mv in masses.items()}
                              for lk, lv in limits_kg.items()}

    # 7. Hard-body radius: from the geometric centre to the farthest point, wings and antennas deployed
    half = (0.050, 0.050, 0.05675)
    r_body = math.sqrt(sum(h * h for h in half))
    vhf_len = 0.50             # m, 2 m quarter-wave (BASELINE §7)
    # the AntS sits on a +/-Z face; an element root is about 0.045 m from the face centre
    r_vhf = math.hypot(0.045 + vhf_len, half[2])
    wing_len = 0.1135          # a 1U-face wing hinged at a body edge, deployed in the face plane
    r_wing = math.hypot(0.050 + wing_len, 0.05675)
    res["hard_body_radius_m"] = {"stowed": round(r_body, 3), "wing_tip": round(r_wing, 3), "vhf_tip": round(r_vhf, 3),
                                 "report": round(max(r_body, r_vhf, r_wing) * 1.1, 2),
                                 "note": "10 % margin on the VHF tip; confirm with the AntS and wing drawings"}

    # 8. Ephemeris deadline
    if passes_dir and (passes_dir / "passes_summary.json").exists():
        p = json.loads((passes_dir / "passes_summary.json").read_text())
        fb = p["stations"]["Santa Clara University CA"]["first_beacon_pass_5deg_h"]
        res["ephemeris"] = {"first_contact_h": fb,
                            "first_ephemeris_due_h_after_separation": {k: round(v + 24.0, 1) for k, v in fb.items()
                                                                       if k != "n"},
                            "best_practice_due_h": 3.0}
    out.mkdir(parents=True, exist_ok=True)
    (out / "leop_feasibility.json").write_text(json.dumps(res, indent=2))
    return res


def main() -> None:
    ap = argparse.ArgumentParser(description="Launch and LEOP feasibility numbers")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--passes", type=Path, default=None)
    a = ap.parse_args()
    print(json.dumps(run(a.out, a.passes), indent=2))


if __name__ == "__main__":
    main()
