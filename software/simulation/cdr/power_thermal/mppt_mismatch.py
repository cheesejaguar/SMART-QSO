#!/usr/bin/env python3
"""MPPT channel count: power lost when strings at different temperatures share one tracker.

The mission simulation sums each string's own maximum power (one tracker per
string). A COTS or custom EPS may parallel several strings behind blocking
diodes into one tracker (ISIS ICEPS2 Type B: 3 MPPT inputs for 8 strings).
Strings on one tracker run at one voltage; a hot wing string (low Vmp) and a
cooler body string (higher Vmp) cannot both sit at their maximum power point.

Cell model: AZUR SPACE 3G30A at 5E13 e/cm² (DB 00010891-01, as in
`sqso-sil/src/env/solar.rs`), the simplified explicit I-V curve
I = Isc [1 - C1 (exp(V / (C2 Voc)) - 1)], with dVmp/dT -6.95 mV/K,
dImp/dT +0.26 mA/K, dVoc/dT -6.2 mV/K, dIsc/dT +0.36 mA/K; two cells a string.

Temperatures from the CDR thermal cases (THERMAL_ANALYSIS v2.0 §5): hot
sun-pointed (H1: wings 67-75 °C, +Y body about 35 °C) and cold sunrise
(C4: wings -39 °C rising, body -17 °C).

Usage: python3 mppt_mismatch.py
"""

import math

VOC, ISC, VMP, IMP = 2.633, 0.513, 2.336, 0.496  # 5E13 column, 28 °C, 1367 W/m²
DVMP, DIMP, DVOC, DISC = -6.95e-3, 0.26e-3, -6.2e-3, 0.36e-3
T_REF = 28.0


def string_params(temp_c, g=1361.0 / 1367.0 * 0.97):
    dt = temp_c - T_REF
    voc = 2 * (VOC + DVOC * dt)
    vmp = 2 * (VMP + DVMP * dt)
    isc = (ISC + DISC * dt) * g
    imp = (IMP + DIMP * dt) * g
    return voc, isc, vmp, imp


def current(v, p):
    voc, isc, vmp, imp = p
    if v >= voc:
        return 0.0
    if v <= 0:
        return isc
    ri = min(max(imp / isc, 0.01), 0.995)
    rv = min(max(vmp / voc, 0.01), 0.995)
    c2 = (rv - 1.0) / math.log(1.0 - ri)
    c1 = (1.0 - ri) * math.exp(-rv / c2)
    return max(0.0, isc * (1.0 - c1 * (math.exp(v / (c2 * voc)) - 1.0)))


def pmax(params, diode_v=0.0):
    """Best power of strings sharing one tracker (common bus voltage behind
    blocking diodes of drop `diode_v`)."""
    best = 0.0
    vmax = max(p[0] for p in params)
    for k in range(1, 2000):
        v = vmax * k / 2000.0
        p = sum(v * current(v + diode_v, q) for q in params)
        best = max(best, p)
    return best


def main():
    cases = [
        ("Hot sun-pointed: +Y body 35 °C, two wing fronts 75 °C", [35.0, 75.0, 75.0]),
        ("Hot sun-pointed, wings 83 °C (real hot orbit H5)", [32.0, 83.0, 83.0]),
        ("Sunrise after eclipse: +Y body -12 °C, wings -35 °C", [-12.0, -35.0, -35.0]),
        ("Mid-sunlight: +Y body 15 °C, wings 60 °C", [15.0, 60.0, 60.0]),
    ]
    print("| Case | Sum of string MPPs (W) | One tracker for the three strings (W) | Loss | Body string alone + wing pair on a second tracker (W) |")
    print("|---|---|---|---|---|")
    for name, temps in cases:
        ps = [string_params(t) for t in temps]
        indiv = sum(pmax([p]) for p in ps)
        one = pmax(ps)
        two = pmax(ps[:1]) + pmax(ps[1:])
        print(f"| {name} | {indiv:.3f} | {one:.3f} | {100 * (1 - one / indiv):.1f} % | {two:.3f} ({100 * (1 - two / indiv):.1f} %) |")


if __name__ == "__main__":
    main()
