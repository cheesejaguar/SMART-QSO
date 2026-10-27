#!/usr/bin/env python3
"""
Protoflight environmental test levels for SMART-QSO (CDR review, PM/SE/V&V).

Derives the levels in docs/environment/ENVIRONMENTAL_TEST_PLAN.md Rev C §3 from
the sources, so the numbers in the plan can be re-derived and checked:

  Random vibration
    * SpaceX Rideshare Payload User's Guide v11 (Aug 2026) Table 4-6: MPE,
      P95/50, 5.57 Grms; Table 6-2: containerized CubeSat protoflight at MPE,
      1 min/axis (SpaceX minimum).
    * GSFC-STD-7000B Table 2.2-2: protoflight = limit level + 3 dB, 1 min/axis;
      Table 2.4-4: minimum workmanship 6.8 Grms; Appendix B §2.4.2.3: "The
      cubesat protoflight vibration environment should therefore be derived as
      the envelope of limit level + 3dB and minimum workmanship."
    * LSP-REQ-317.01 Rev B Table 1 (CSLI path only): protoflight MPE + 3 dB,
      2 min/axis.
  Thermal vacuum and bake-out
    * GSFC-STD-7000B §2.6.3.2.1.1 / Table 2.6-1: test margin >= 10 °C beyond
      the allowable flight temperature (AFT); >= 8 TV cycles for qualification,
      12 cycles before flight as the workmanship goal; Appendix B §2.6: dwell
      set by stabilisation plus performance test, not 24 h.
    * SpaceX UG v11 Table 6-2 (advised, not required): protoflight TV/TC at the
      acceptance range (envelope of MPT and -24..+61 °C) +/- 5 °C, 20 cycles
      total, >= 4 in vacuum.
    * LSP-REQ-317.01 Rev B Table 1: CubeSat bake-out >= 70 °C for 3 h after
      stabilisation at <= 1e-4 Torr, or >= 60 °C for 6 h if 70 °C cannot be
      reached (note 5); bake-out not above qualification temperatures (note 7).

Run:  python3 software/simulation/cdr/pm/test_levels.py
"""

from __future__ import annotations

import math
from typing import List, Sequence, Tuple

Spectrum = Sequence[Tuple[float, float]]   # (Hz, g^2/Hz) breakpoints, log-log straight lines

#: SpaceX UG v11 Table 4-6, random vibration MPE (all axes)
SPACEX_MPE: Spectrum = ((20, 0.01), (50, 0.015), (700, 0.015), (800, 0.03), (925, 0.03), (2000, 0.00644))

#: GSFC-STD-7000B Table 2.4-4, component minimum workmanship (<= 45.4 kg)
GEVS_WORKMANSHIP: Spectrum = ((20, 0.01), (80, 0.04), (500, 0.04), (2000, 0.01))

#: GSFC-STD-7000B Table 2.4-3, generalized ELV component qualification (<= 22.7 kg), for comparison
GEVS_COMPONENT_QUAL: Spectrum = ((20, 0.026), (50, 0.16), (800, 0.16), (2000, 0.026))


def asd(spec: Spectrum, f: float) -> float:
    """ASD at f, interpolated as straight lines on log-log axes."""
    for (f1, a1), (f2, a2) in zip(spec, spec[1:]):
        if f1 <= f <= f2:
            slope = math.log(a2 / a1) / math.log(f2 / f1)
            return a1 * (f / f1) ** slope
    raise ValueError(f"{f} Hz outside {spec[0][0]}-{spec[-1][0]} Hz")


def grms(spec: Spectrum) -> float:
    """Overall level: exact integral of each log-log segment."""
    total = 0.0
    for (f1, a1), (f2, a2) in zip(spec, spec[1:]):
        n = math.log(a2 / a1) / math.log(f2 / f1)
        if abs(n + 1) < 1e-9:
            total += a1 * f1 * math.log(f2 / f1)
        else:
            total += a1 * f1 / (n + 1) * ((f2 / f1) ** (n + 1) - 1)
    return math.sqrt(total)


def scale(spec: Spectrum, db: float) -> List[Tuple[float, float]]:
    k = 10 ** (db / 10)
    return [(f, a * k) for f, a in spec]


def envelope(*specs: Spectrum) -> List[Tuple[float, float]]:
    """Maximum of several spectra, as breakpoints: every input breakpoint plus every
    crossing point, then collinear points removed."""
    freqs = sorted({f for s in specs for f, _ in s})
    pts: List[float] = []
    for f1, f2 in zip(freqs, freqs[1:]):
        pts.append(f1)
        # crossings inside (f1, f2): each spectrum is a power law there, so solve pairwise
        for i, s in enumerate(specs):
            for t in specs[i + 1:]:
                a1, a2 = asd(s, f1), asd(s, f2)
                b1, b2 = asd(t, f1), asd(t, f2)
                if (a1 - b1) * (a2 - b2) < 0:
                    na = math.log(a2 / a1) / math.log(f2 / f1)
                    nb = math.log(b2 / b1) / math.log(f2 / f1)
                    x = math.log(b1 / a1) / (na - nb)          # ln(f/f1)
                    pts.append(f1 * math.exp(x))
    pts.append(freqs[-1])
    pts = sorted(set(round(p, 3) for p in pts))
    env = [(f, max(asd(s, f) for s in specs)) for f in pts]
    # drop points that lie on the log-log line between their neighbours
    out = [env[0]]
    for prev, cur, nxt in zip(env, env[1:], env[2:]):
        n1 = math.log(cur[1] / prev[1]) / math.log(cur[0] / prev[0])
        n2 = math.log(nxt[1] / cur[1]) / math.log(nxt[0] / cur[0])
        if abs(n1 - n2) > 1e-6:
            out.append(cur)
    out.append(env[-1])
    return out


def db_per_octave(spec: Spectrum) -> List[float]:
    return [10 * math.log10(a2 / a1) / math.log2(f2 / f1) for (f1, a1), (f2, a2) in zip(spec, spec[1:])]


# ----------------------------------------------------------------------------------------------
# Thermal: allowable flight temperatures (AFT) and the resulting test temperatures
# ----------------------------------------------------------------------------------------------

#: Allowable flight temperatures used for the interim levels, operating (min, max) and
#: non-operating (min, max), °C, with the source. These are requirement and part limits, not
#: thermal-model predictions: the correlated model (THERMAL_ANALYSIS re-issue) replaces them.
AFT = {
    "System (SYS-600 operate / SYS-601 survive)": ((-20, 50), (-40, 60)),
    "Battery, INR18650-35E: charge 0..45 (SYS-602), discharge -10..60 (cell data sheet)": ((-10, 45), (-20, 60)),
    "Jetson module, +70 °C limit used by the audit (TB-07); OBC throttles above +60": ((-25, 70), (-25, 80)),
}

GEVS_MARGIN = 10.0          # °C beyond AFT, GSFC-STD-7000B §2.6.3.2.1.1
SPACEX_ACCEPT_RANGE = (-24.0, 61.0)   # UG v11 Table 6-2 minimum acceptance range
SPACEX_PF_MARGIN = 5.0      # UG v11 Table 6-2, protoflight = acceptance +/- 5 °C


def main() -> None:
    mpe = list(SPACEX_MPE)
    mpe3 = scale(mpe, 3.0)
    pf = envelope(mpe3, GEVS_WORKMANSHIP)
    print("Random vibration (20-2000 Hz)")
    rows = [
        ("SpaceX UG v11 MPE (Table 4-6)", mpe),
        ("MPE + 3 dB (GEVS-B Table 2.2-2 protoflight level)", mpe3),
        ("GEVS-B Table 2.4-4 minimum workmanship", list(GEVS_WORKMANSHIP)),
        ("GEVS-B App. B protoflight: envelope(MPE+3 dB, workmanship)", pf),
        ("GEVS-B Table 2.4-3 component qualification (comparison only)", list(GEVS_COMPONENT_QUAL)),
    ]
    for name, spec in rows:
        print(f"  {name:62s} {grms(spec):5.2f} Grms")
    print("\n  Protoflight envelope breakpoints (Hz, g^2/Hz) and slopes (dB/oct):")
    slopes = db_per_octave(pf) + [float('nan')]
    for (f, a), s in zip(pf, slopes):
        print(f"    {f:8.1f}  {a:.4f}   {'' if math.isnan(s) else f'{s:+.2f}'}")
    ratio_db = 20 * math.log10(grms(pf) / grms(mpe))
    print(f"\n  Envelope over MPE: {grms(pf)/grms(mpe):.3f}x ({ratio_db:+.2f} dB overall);"
          f" MPE alone is {20*math.log10(grms(GEVS_WORKMANSHIP)/grms(mpe)):+.2f} dB below the workmanship minimum")
    # fatigue-equivalent comparison: Miles-type damage ~ (Grms)^b * t, b ~ 4..8 (structural S-N exponent)
    for b in (4, 6, 8):
        print(f"  Damage ratio envelope/MPE for 1 min each, S-N exponent b={b}: {(grms(pf)/grms(mpe))**b:.1f}x")

    print("\nShock by analysis (LSP-REQ-317.01 Rev B Table 1 note 3 criterion; Miles' equation, Q = 10, 3 sigma)")
    print("  Random-vibe-equivalent SRS of the protoflight envelope; the shock test may be waived where it")
    print("  exceeds the integrator's qualification shock SRS below 2000 Hz (integrator value not yet known):")
    for fn in (100, 200, 500, 1000, 1500, 2000):
        g = 3 * math.sqrt(math.pi / 2 * fn * 10 * asd(pf, fn))
        print(f"    {fn:5d} Hz  {g:6.1f} g")

    print("\nThermal vacuum test temperatures (°C)")
    print(f"  {'Item':84s} {'op test':>13s} {'non-op test':>13s}")
    for item, ((omin, omax), (smin, smax)) in AFT.items():
        print(f"  {item:84s} {omin-GEVS_MARGIN:+5.0f}/{omax+GEVS_MARGIN:+4.0f}  "
              f"{smin-GEVS_MARGIN:+5.0f}/{smax+GEVS_MARGIN:+4.0f}")
    lo, hi = SPACEX_ACCEPT_RANGE
    print(f"  SpaceX UG v11 protoflight (advised): {lo-SPACEX_PF_MARGIN:+.0f} to {hi+SPACEX_PF_MARGIN:+.0f} °C"
          " at minimum (envelope of MPT and -24..+61, +/- 5), 20 cycles, >= 4 in vacuum")
    sys_op = AFT["System (SYS-600 operate / SYS-601 survive)"][0]
    batt = AFT["Battery, INR18650-35E: charge 0..45 (SYS-602), discharge -10..60 (cell data sheet)"]
    print(f"  Conflict: the GEVS system hot operating test ({sys_op[1]+GEVS_MARGIN:+.0f} °C) is already at the"
          f" battery's non-operating maximum ({batt[1][1]:+.0f} °C), {sys_op[1]+GEVS_MARGIN-batt[0][1]:.0f} °C above"
          f" its charge limit; the GEVS system non-operating test ({AFT['System (SYS-600 operate / SYS-601 survive)'][1][1]+GEVS_MARGIN:+.0f} °C)"
          f" and the SpaceX advised level ({hi+SPACEX_PF_MARGIN:+.0f} °C) exceed the battery maximum by"
          f" {AFT['System (SYS-600 operate / SYS-601 survive)'][1][1]+GEVS_MARGIN-batt[1][1]:.0f} and"
          f" {hi+SPACEX_PF_MARGIN-batt[1][1]:.0f} °C")
    print("  -> the plan caps the hot plateau at the battery limit (+60 °C) and records the shortfall")
    print("     against the GEVS margin as a deviation, closed by the correlated model (MPT + 10 <= 60)")

    print("\nBake-out (LSP-REQ-317.01 Rev B Table 1, notes 5 and 7)")
    print("  >= 70 °C for 3 h after stabilisation, or >= 60 °C for 6 h if 70 °C cannot be reached;")
    print(f"  battery non-operating maximum {batt[1][1]:+.0f} °C -> bake at +60 °C (+0/-2) for >= 6 h at <= 1e-4 Torr")


if __name__ == "__main__":
    main()
