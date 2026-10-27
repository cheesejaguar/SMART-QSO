#!/usr/bin/env python3
"""
CDR radiation analysis for SMART-QSO (SQSO-ENV-002 Rev F, docs/environment/RADIATION_ANALYSIS.md).

    python3 software/simulation/cdr/avionics/radiation_tid_see.py [--markdown]

Total ionizing dose (TID) versus aluminium depth for the baseline orbit (475 km
target, 465-485 km band, sun-synchronous 97.31°; 2-year design life; BASELINE
§2, §3), with 500 km and 525 km kept as upper bounds (the mission simulation's
unit tests cite them), and single-event-effect (SEE) rates for the avionics
parts, per day of operation and, for the Jetson, per burst. No radiation
environment code (AE9/AP9, SHIELDOSE-2) runs here: the inputs are published
SHIELDOSE-2 outputs and measurements, cited inline, and the script combines,
scales and tabulates them so every number in the document can be reproduced.

Inputs
------
T1. Trapped electrons + bremsstrahlung + trapped protons, 1 year, centre of solid
    Al spheres: SPENVIS SHIELDOSE-2 with AE-8 MAX / AP-8 MAX at 500 km, published
    on the "Radiation Environment for a Sun Synchronous Orbit" page of
    cubesat.wikidot.com (report centre.doc, 2007-11-30). The report does not state
    the inclination (its SPENVIS project folder is named "45INC"), so T2 checks it.
T2. Shields-1 (NASA LaRC, NORAD 43850): SPENVIS SHIELDOSE-2, AP-8 MIN / AE-8 MAX,
    500 km, 85°, 1 year, Al sphere 0.535 g/cm² (1.98 mm): 117 rad(Si) trapped
    protons + 1266 rad(Si) trapped electrons = 1383 rad(Si). Measured in orbit
    (2019, solar minimum) behind slabs with thick backing: 90.9 ± 2.7 rad/yr behind
    1.26 g/cm², 84.3 ± 2.5 behind 1.69, 73.6 ± 3.2 behind 3.00, 70.0 ± 3.0 behind
    6.00 g/cm² (models of the same slabs: 41.0, 30.7, 22.1, 13.7 rad/yr).
    D. L. Thomsen III, "Shields-1 Dosimetry Measurements in Polar Low Earth Orbit",
    NASA LaRC, 14 September 2022.
T3. Solar protons, 1 year, 95 % confidence, King (SOLPRO) model, Shields-1 orbit:
    ≈ 750 rad(Si) behind 0.535 g/cm² Al (NASA, "State-of-the-Art of Small
    Spacecraft Technology", ch. 6 Structures, 2021, citing Shields-1: "yearly
    trapped dose of 1383 Rad ... additional estimated 750 Rad from solar particle
    events"), ≈ 400 rad behind 1.26 g/cm² (read from the King-model figure in T2).
    Between and beyond these the dose is interpolated exponentially in areal
    density (the solar-proton spectrum is hard, so the curve is shallow).
T4. Galactic cosmic rays and secondaries: the Shields-1 measurement behind
    6 g/cm² exceeds the trapped-proton model by 56 rad/yr (70.0 − 13.7), the
    part of the dose the trapped models do not include. 50 rad/yr is used at
    every depth.
T5. Altitude: trapped-particle dose scaled by exp((h − 500 km)/150 km), the
    scale height the mission simulation uses (sqso-sil/src/env/radiation.rs)
    and consistent with AP-8 flux growth between 500 and 550 km; solar protons
    and GCR are not scaled. The same factor takes the dose down to the baseline
    band (465-485 km); below 500 km it is an extrapolation of the same law.

SEE inputs
----------
P1. Trapped-proton integral flux, orbit average, AP-8 MAX, 500 km (T1's
    trapped.doc): > 10 MeV 21.5, > 30 MeV 17.4, > 50 MeV 14.8, > 100 MeV
    9.28 /cm²/s. Used unscaled for the baseline band (465-485 km), where the
    flux is lower: the SEE rates are upper bounds for the baseline orbit.
P2. Cross-sections are generic COTS values or published data where found
    (each row says which); every rate is an estimate to be replaced by a
    part-level test or a CREME96 run when the parts are frozen.
"""

from __future__ import annotations

import argparse
import math

import numpy as np

AL_DENSITY = 2.70  # g/cm³
YEAR_S = 365.25 * 86400.0
DAY_S = 86400.0

# T1: SPENVIS SHIELDOSE-2, AE-8 MAX / AP-8 MAX, 500 km, 1 year, solid sphere
# depth (mm), total, electrons, bremsstrahlung, protons (rad(Si))
T1 = np.array([
    (0.05, 2.428e5, 2.418e5, 4.590e2, 6.087e2),
    (0.10, 1.305e5, 1.299e5, 2.739e2, 3.556e2),
    (0.20, 5.848e4, 5.810e4, 1.459e2, 2.322e2),
    (0.30, 3.274e4, 3.246e4, 9.399e1, 1.940e2),
    (0.40, 2.107e4, 2.083e4, 6.723e1, 1.723e2),
    (0.50, 1.480e4, 1.459e4, 5.191e1, 1.575e2),
    (0.60, 1.111e4, 1.093e4, 4.230e1, 1.463e2),
    (0.80, 7.281e3, 7.119e3, 3.105e1, 1.304e2),
    (1.00, 5.307e3, 5.163e3, 2.474e1, 1.194e2),
    (1.50, 2.934e3, 2.814e3, 1.646e1, 1.036e2),
    (2.00, 1.830e3, 1.723e3, 1.220e1, 9.446e1),
    (2.50, 1.191e3, 1.094e3, 9.584e0, 8.782e1),
    (3.00, 7.921e2, 7.013e2, 7.878e0, 8.289e1),
    (4.00, 3.791e2, 2.968e2, 5.820e0, 7.645e1),
    (5.00, 1.991e2, 1.230e2, 4.628e0, 7.150e1),
    (6.00, 1.203e2, 4.824e1, 3.871e0, 6.819e1),
    (8.00, 7.221e1, 6.431e0, 2.971e0, 6.281e1),
    (10.0, 6.185e1, 6.771e-1, 2.473e0, 5.870e1),
])

# T2: Shields-1 model at 1.98 mm (sphere), rad(Si)/yr
SHIELDS1_2MM = {"protons": 117.0, "electrons": 1266.0}

# T3: solar protons, 1 year, 95 %: (g/cm², rad(Si))
SPE_POINTS = ((0.535, 750.0), (1.26, 400.0))
# T4
GCR_RAD_PER_YEAR = 50.0
# T5
SCALE_HEIGHT_KM = 150.0

DEPTHS_MM = (1.0, 1.5, 2.0, 2.1, 2.5, 3.0)
#: The baseline band (BASELINE §3): 465 km floor, 475 km target, 485 km cap
BASELINE_ALTS_KM = (465.0, 475.0, 485.0)
#: Former altitudes, kept as upper bounds (sqso-sil env/radiation.rs unit tests cite their totals)
BOUND_ALTS_KM = (500.0, 525.0)
#: Depths of the baseline-band summary table (BASELINE §6: the 2.1 mm rule, RID RAD-01)
SUMMARY_DEPTHS_MM = (1.0, 2.0, 2.1, 3.0)
DESIGN_KRAD = 10.0  # BASELINE §6
LIFE_YEARS = 2.0


def loginterp(x: float, xs: np.ndarray, ys: np.ndarray) -> float:
    """Log-linear interpolation of a positive curve"""
    return float(np.exp(np.interp(x, xs, np.log(ys))))


def trapped_per_year(mm: float, alt_km: float = 500.0) -> float:
    """T1 trapped total (rad(Si)/yr) at depth mm, scaled to alt_km (T5)"""
    return loginterp(mm, T1[:, 0], T1[:, 1]) * math.exp((alt_km - 500.0) / SCALE_HEIGHT_KM)


def spe_per_year(mm: float) -> float:
    """T3 solar protons (rad(Si)/yr, 95 %) at depth mm"""
    (x0, d0), (x1, d1) = SPE_POINTS
    k = math.log(d0 / d1) / (x1 - x0)
    return d0 * math.exp(-k * (mm * AL_DENSITY / 10.0 - x0))


def total_per_year(mm: float, alt_km: float = 500.0) -> float:
    """T1 + T3 + T4 (rad(Si)/yr) at depth mm and altitude alt_km"""
    return trapped_per_year(mm, alt_km) + spe_per_year(mm) + GCR_RAD_PER_YEAR


def dose_table(alt_km: float, years: float, depths=DEPTHS_MM):
    rows = []
    for mm in depths:
        tr = trapped_per_year(mm, alt_km)
        spe = spe_per_year(mm)
        total = (tr + spe + GCR_RAD_PER_YEAR) * years
        rows.append((mm, tr * years, spe * years, GCR_RAD_PER_YEAR * years, total, DESIGN_KRAD * 1000.0 / total))
    return rows


def depth_for_margin(alt_km: float, years: float, rdm: float) -> float:
    """Smallest Al depth (mm) at which the design level gives the margin rdm"""
    for mm in np.arange(1.0, 6.0, 0.01):
        tr = trapped_per_year(mm, alt_km)
        total = (tr + spe_per_year(mm) + GCR_RAD_PER_YEAR) * years
        if DESIGN_KRAD * 1000.0 / total >= rdm:
            return float(mm)
    return float("nan")


#: The mission simulation's TID model (sqso-sil/src/env/radiation.rs, RID RAD-02): rad(Si)/yr behind 2 mm at 500 km
SIL_TRAPPED_2MM = 1830.0
SIL_SOLAR_GCR_2MM = 797.0
SIL_DEPTH_EXPONENT = 1.35


def sil_model(mm: float, alt_km: float = 500.0) -> float:
    """The mission simulation's TID model (sqso-sil/src/env/radiation.rs `tid_rad_per_year`): the trapped part
    scaled with altitude, the solar and GCR part not, both scaled with depth as (2 mm / t)^1.35"""
    depth = (2.0 / max(mm, 0.1)) ** SIL_DEPTH_EXPONENT
    return (SIL_TRAPPED_2MM * math.exp((alt_km - 500.0) / 150.0) + SIL_SOLAR_GCR_2MM) * depth


# ----------------------------------------------------------------------------
# Single-event effects

FLUX_GT30 = 17.4   # /cm²/s, P1
FLUX_GT100 = 9.28


def per_day(sigma_cm2: float, flux: float) -> float:
    return sigma_cm2 * flux * DAY_S


SEE_ROWS = [
    # part, effect, sigma basis, sigma low, sigma high (cm² per device or per bit), bits (1 for device), flux, duty
    ("STM32L4R5 SRAM (640 KB, 5.24 Mbit)", "SEU (proton)",
     "generic COTS 90 nm SRAM, 1e-14 to 1e-13 cm²/bit", 1e-14, 1e-13, 640 * 1024 * 8, FLUX_GT30, 1.0),
    ("STM32L4R5 core and peripherals", "SEFI needing a reset (proton)",
     "generic COTS MCU, 1e-11 to 1e-10 cm²/device", 1e-11, 1e-10, 1, FLUX_GT30, 1.0),
    ("STM32L4R5", "SEL (proton-induced)",
     "no part data found; 0 to 1e-11 cm² (bounds a part with a low heavy-ion threshold)", 0.0, 1e-11, 1, FLUX_GT100, 1.0),
    ("FM25V20A FRAM (2 Mbit)", "SEU in the cell array",
     "ferroelectric cells are not upset by protons: 0; interface and write path as the MCU's SEFI", 0.0, 0.0, 1, FLUX_GT30, 1.0),
    ("FM25V20A FRAM", "SEFI or SEL (CMOS periphery)",
     "no part data found; 0 to 1e-11 cm²", 0.0, 1e-11, 1, FLUX_GT100, 1.0),
    ("Epson M-G370PDT0 gyro", "SEL or SEFI",
     "TID data only (20 krad pass); SEE unknown: 1e-11 to 1e-10 cm²", 1e-11, 1e-10, 1, FLUX_GT30, 1.0),
    ("Jetson Orin module (SoC)", "SEFI (proton)",
     "Orin, 200 MeV protons, ≈ 7e-11 cm² (indicative, conference slides); 7e-11 to 7e-10", 7e-11, 7e-10, 1, FLUX_GT30, 0.10),
    ("Jetson Orin LPDDR5 (8 GB, 6.9e10 bit)", "SEU (proton)",
     "DDR-class DRAM, 1e-19 to 1e-17 cm²/bit", 1e-19, 1e-17, 8 * 1024 ** 3 * 8, FLUX_GT30, 0.10),
    ("Jetson Orin module (SoC and DRAM)", "SEL (proton-induced)",
     "no part data (the Orin AGX test was not reachable); 0 to 1e-10 cm²", 0.0, 1e-10, 1, FLUX_GT100, 0.10),
    ("RF: receiver/TNC MCU, CTCSS decoder, exciter", "SEFI or SEL",
     "COTS, no data: 1e-11 to 1e-10 cm² per device, 3 devices", 3e-11, 3e-10, 1, FLUX_GT30, 1.0),
    ("GNSS receiver (piNAV-NG)", "SEFI or SEL",
     "COTS, no data: 1e-11 to 1e-10 cm²", 1e-11, 1e-10, 1, FLUX_GT30, 0.05),
]


#: OBC live state in SRAM2 (parity): statics 43,452 B and the 8 KB stack (software/flight/docs/MEMORY_POLICY.md §3.2)
LIVE_STATE_BYTES = 43_452 + 8 * 1024
SRAM_BYTES = 640 * 1024
SRAM2_BYTES = 64 * 1024


def obc_sram_rates():
    """(live-state upsets per day low, high; SRAM2 upsets per day low, high)"""
    part, _e, _b, lo, hi, bits, flux, duty = SEE_ROWS[0]
    whole_lo, whole_hi = per_day(lo, flux) * bits * duty, per_day(hi, flux) * bits * duty
    f_live, f_sram2 = LIVE_STATE_BYTES / SRAM_BYTES, SRAM2_BYTES / SRAM_BYTES
    return whole_lo * f_live, whole_hi * f_live, whole_lo * f_sram2, whole_hi * f_sram2


#: Jetson bursts (BASELINE §6, SRS-F151): ≤ 10 min, 3.6 kJ; 120 s is the batch of SYS-512 and VI-02
BURST_S = (120.0, 600.0)
JETSON_PARTS = ("Jetson Orin module (SoC)", "Jetson Orin LPDDR5 (8 GB, 6.9e10 bit)")


def jetson_powered_rate_per_s(high: bool = True) -> float:
    """Rate of any SEFI or DRAM upset while the Jetson is powered (per s): the duty taken out of the rows"""
    total = 0.0
    for part, _effect, _basis, lo, hi, bits, flux, duty in SEE_ROWS:
        if part in JETSON_PARTS:
            total += (hi if high else lo) * flux * bits
    return total


def burst_upset_probability(burst_s: float, high: bool = True) -> float:
    """Probability of at least one SEFI or DRAM upset in one burst of burst_s seconds"""
    return 1.0 - math.exp(-jetson_powered_rate_per_s(high) * burst_s)


def see_table():
    out = []
    for part, effect, basis, lo, hi, bits, flux, duty in SEE_ROWS:
        rlo = per_day(lo, flux) * bits * duty
        rhi = per_day(hi, flux) * bits * duty
        out.append((part, effect, basis, rlo, rhi, rhi * 365.25 * LIFE_YEARS))
    return out


def baseline_summary_markdown() -> str:
    """Baseline band (465/475/485 km), 2 years, total and margin at the summary depths"""
    head = " | ".join(f"{a:.0f} km: total (krad) / margin" for a in BASELINE_ALTS_KM)
    lines = [f"| Depth (mm Al) | {head} |", "|---|" + "---|" * len(BASELINE_ALTS_KM)]
    for mm in SUMMARY_DEPTHS_MM:
        cells = []
        for alt in BASELINE_ALTS_KM:
            total = total_per_year(mm, alt) * LIFE_YEARS
            cells.append(f"{total / 1000.0:.2f} / {DESIGN_KRAD * 1000.0 / total:.2f}")
        lines.append(f"| {mm:.1f} | " + " | ".join(cells) + " |")
    return "\n".join(lines)


def tid_tables_markdown() -> str:
    lines = []
    for alt in (475.0, 485.0) + BOUND_ALTS_KM:
        lines.append(f"\n**{alt:.0f} km, {LIFE_YEARS:.0f} years, solid Al sphere, rad(Si)**\n")
        lines.append("| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |")
        lines.append("|---|---|---|---|---|---|")
        for mm, tr, spe, gcr, total, rdm in dose_table(alt, LIFE_YEARS):
            lines.append(f"| {mm:.1f} | {tr:,.0f} | {spe:,.0f} | {gcr:,.0f} | **{total:,.0f}** | {rdm:.2f} |")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--markdown", action="store_true", help="print the document tables")
    args = ap.parse_args()

    # Cross-check of T1 against T2 at 2 mm
    t1_2 = trapped_per_year(1.98)
    t2_2 = sum(SHIELDS1_2MM.values())
    print(f"Cross-check at 1.98 mm, 1 year: T1 (AE-8/AP-8 MAX) {t1_2:,.0f} rad, "
          f"T2 Shields-1 (AE-8 MAX/AP-8 MIN, 85°) {t2_2:,.0f} rad: ratio {t1_2 / t2_2:.2f}")
    print("\nBaseline band, 2 years, solid Al sphere: total krad(Si) / margin of the 10 krad design level")
    print(baseline_summary_markdown())
    print(tid_tables_markdown() if args.markdown else "")
    for alt in BASELINE_ALTS_KM + BOUND_ALTS_KM:
        for rdm in (2.0, 3.0):
            print(f"Depth for a radiation design margin of {rdm:.0f} on 10 krad at {alt:.0f} km: "
                  f"{depth_for_margin(alt, LIFE_YEARS, rdm):.2f} mm Al")
    print("\nMission simulation's TID model against this analysis (rad(Si)/yr, at 475 km and 500 km):")
    print("| Depth (mm Al) | SIL 475 km | Analysis 475 km | SIL / analysis | SIL 500 km | Analysis 500 km | SIL / analysis |")
    print("|---|---|---|---|---|---|---|")
    for mm in DEPTHS_MM:
        row = [f"{mm:.1f}"]
        for alt in (475.0, 500.0):
            a = total_per_year(mm, alt)
            row += [f"{sil_model(mm, alt):,.0f}", f"{a:,.0f}", f"{sil_model(mm, alt) / a:.2f}"]
        print("| " + " | ".join(row) + " |")
    print("\nSingle-event rates (per day of operation; the Jetson and GNSS at their duty):")
    print("| Part | Effect | Cross-section basis | Rate low (/day) | Rate high (/day) | High, 2 years |")
    print("|---|---|---|---|---|---|")
    for part, effect, basis, rlo, rhi, two in see_table():
        print(f"| {part} | {effect} | {basis} | {rlo:.2g} | {rhi:.2g} | {two:.2g} |")
    llo, lhi, slo, shi = obc_sram_rates()
    print(f"\nOBC live state ({LIVE_STATE_BYTES:,} B in SRAM2): {llo:.2g} to {lhi:.2g} upsets a day, "
          f"{llo * 365.25 * LIFE_YEARS:.0f} to {lhi * 365.25 * LIFE_YEARS:.0f} in {LIFE_YEARS:.0f} years; "
          f"all of SRAM2: {slo:.2g} to {shi:.2g} a day")
    lo, hi = jetson_powered_rate_per_s(False), jetson_powered_rate_per_s(True)
    print(f"\nJetson SEFI or DRAM upset while powered: {lo * DAY_S:.2g} to {hi * DAY_S:.2g} per powered day")
    for b in BURST_S:
        print(f"Probability of any upset in one {b:.0f} s burst: {burst_upset_probability(b, False):.1e} to "
              f"{burst_upset_probability(b, True):.1e}")


if __name__ == "__main__":
    main()
