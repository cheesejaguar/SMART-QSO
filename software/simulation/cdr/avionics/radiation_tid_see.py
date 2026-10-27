#!/usr/bin/env python3
"""
CDR radiation analysis for SMART-QSO (SQSO-ENV-002 Rev C, docs/environment/RADIATION_ANALYSIS.md).

    python3 software/simulation/cdr/avionics/radiation_tid_see.py [--markdown]

Total ionizing dose (TID) versus aluminium depth for the baseline orbit (500 km
circular, sun-synchronous ≈ 97.4°, 525 km cap; 2-year design life; BASELINE §2,
§3), and single-event-effect (SEE) rates for the avionics parts. No radiation
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
    and GCR are not scaled.

SEE inputs
----------
P1. Trapped-proton integral flux, orbit average, AP-8 MAX, 500 km (T1's
    trapped.doc): > 10 MeV 21.5, > 30 MeV 17.4, > 50 MeV 14.8, > 100 MeV
    9.28 /cm²/s.
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

DEPTHS_MM = (1.0, 1.5, 2.0, 2.5, 3.0)
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


def dose_table(alt_km: float, years: float):
    rows = []
    for mm in DEPTHS_MM:
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


def sil_model(mm: float, alt_km: float = 500.0) -> float:
    """The mission simulation's TID model (sqso-sil/src/env/radiation.rs)"""
    return 1500.0 * math.exp((alt_km - 500.0) / 150.0) * (2.0 / mm) ** 1.2


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
    ("RF: receiver/TNC MCU, CTCSS decoder, exciter", "SEFI or SEL",
     "COTS, no data: 1e-11 to 1e-10 cm² per device, 3 devices", 3e-11, 3e-10, 1, FLUX_GT30, 1.0),
    ("GNSS receiver (piNAV-NG)", "SEFI or SEL",
     "COTS, no data: 1e-11 to 1e-10 cm²", 1e-11, 1e-10, 1, FLUX_GT30, 0.05),
]


def see_table():
    out = []
    for part, effect, basis, lo, hi, bits, flux, duty in SEE_ROWS:
        rlo = per_day(lo, flux) * bits * duty
        rhi = per_day(hi, flux) * bits * duty
        out.append((part, effect, basis, rlo, rhi, rhi * 365.25 * LIFE_YEARS))
    return out


def tid_tables_markdown() -> str:
    lines = []
    for alt in (500.0, 525.0):
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
    print(tid_tables_markdown() if args.markdown else "")
    for alt in (500.0, 525.0):
        for rdm in (2.0, 3.0):
            print(f"Depth for a radiation design margin of {rdm:.0f} on 10 krad at {alt:.0f} km: "
                  f"{depth_for_margin(alt, LIFE_YEARS, rdm):.2f} mm Al")
    print("\nMission simulation's TID model against this analysis (rad(Si)/yr, 500 km):")
    print("| Depth (mm Al) | SIL | This analysis | SIL / analysis |")
    print("|---|---|---|---|")
    for mm in DEPTHS_MM:
        a = trapped_per_year(mm) + spe_per_year(mm) + GCR_RAD_PER_YEAR
        print(f"| {mm:.1f} | {sil_model(mm):,.0f} | {a:,.0f} | {sil_model(mm) / a:.2f} |")
    print("\nSingle-event rates (per day of operation; the Jetson and GNSS at their duty):")
    print("| Part | Effect | Cross-section basis | Rate low (/day) | Rate high (/day) | High, 2 years |")
    print("|---|---|---|---|---|---|")
    for part, effect, basis, rlo, rhi, two in see_table():
        print(f"| {part} | {effect} | {basis} | {rlo:.2g} | {rhi:.2g} | {two:.2g} |")


if __name__ == "__main__":
    main()
