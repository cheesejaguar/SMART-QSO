#!/usr/bin/env python3
"""
Bottom-up ROM of the engineering-model and protoflight test campaign, and its check against
docs/funding_audit/LIFECYCLE_COST_ROM.md (SQSO-AUD-005) WBS 10 "Integration & test" (Rev B: $23k-78k;
Rev C to E: $35k-101k). CDR review, PM/SE/V&V; round 3 (RIDs VV-07, P3P-08, P3P-03) added the EM shock
test, the FM system EMC and the reliability programme. Feeds docs/environment/ENVIRONMENTAL_TEST_PLAN.md §7.

No laboratory has been contacted; rates are public benchmarks or engineering judgment:
  * Hawaii Space Flight Laboratory published rates: TVAC $3,500/day, vibration table $3,500/day
    (https://www.hsfl.hawaii.edu/services-2/integration-and-testing/, read 2026-10-27).
  * Small-chamber and shaker day rates at California commercial labs (Experior, Kelly Space,
    NTS) are not published; a $2k-5k/day band is assumed and must be replaced by quotes.
TVAC duration is computed from the cycle profile of ENVIRONMENTAL_TEST_PLAN Rev C §3.4.

    python3 software/simulation/cdr/pm/test_cost.py
"""

from __future__ import annotations

ROM_WBS10 = (35_000, 101_000)   # SQSO-AUD-005 Rev E, before round 3


def tvac_days(cycles: int, span_c: float, rate_c_per_min: float, plateau_h: float,
              bake_h: float, setup_days: float) -> float:
    """Chamber days for a cycling test: two transitions and two plateaus per cycle, plus bake-out,
    pump-down, set-up and tear-down."""
    ramp_h = span_c / rate_c_per_min / 60.0
    cycle_h = 2 * ramp_h + 2 * plateau_h
    return (cycles * cycle_h + bake_h) / 24.0 + setup_days


def main() -> None:
    # Protoflight TVAC: -30..+60 °C operational span 90 °C (§3.4), 0.5 °C/min achievable for a
    # 1.6 kg cube on a platen (GEVS Table 2.6-1 allows up to 1 °C/min), 3 h plateaus (stabilise
    # + CPT), 8 cycles; bake-out 6 h at +60 °C plus 4 h stabilisation; 1.5 days set-up.
    pf_days = tvac_days(8, 90, 0.5, 3.0, 10.0, 1.5)
    em_days = tvac_days(4, 90, 0.5, 3.0, 0.0, 1.5) + 1.0     # + 1 day thermal-balance points
    lines = [
        # (phase, item, low, high, basis)
        ("EM", "Pathfinder random vibe in a TestPod, 3 axes", 4_000, 9_000,
         "1 shaker day at $2.5k-5k + TestPod rental $1.5k-4k [U]"),
        ("EM", f"EM TVAC and thermal balance ({em_days:.1f} chamber days)", round(em_days * 2_000), round(em_days * 4_000),
         "Jetson heat path (SYS-603), burst energy, 4 cycles; $2k-4k/day"),
        ("EM", "EMC pre-scan: RE at the uplink receiver, CE, self-compatibility", 1_500, 6_000, "1-2 lab days"),
        ("EM", "Antenna pattern, EM", 0, 5_000, "outdoor range (club) to 1 anechoic day"),
        ("EM", "Shock, EM, at MPE + 3 dB, 2 actuations per axis (VI-07; carried until the ICD)", 3_000, 12_000,
         "1-2 days on a resonant-plate or drop shock machine at $2.5k-5k + fixture; $0 if VI-07 closes by analysis"),
        ("EM", "Reliability programme: >= 500 h on the EM boards (VI-11)", 0, 1_000,
         "own bench, logging, bench supplies"),
        ("FM", "Mass properties and CG", 500, 2_000, "in-house fixture to lab measurement"),
        ("FM", "Protoflight random vibe, 3 axes, inhibit monitoring", 4_000, 14_000,
         "1-2 shaker days + TestPod; GEVS App. B envelope 8.18 Grms"),
        ("FM", f"Protoflight TVAC, 8 cycles + bake-out ({pf_days:.1f} chamber days)", round(pf_days * 2_000),
         round(pf_days * 4_000), "$2k-4k/day; HSFL benchmark $3.5k/day"),
        ("FM", "Deployment tests hot and cold, re-stow kit", 500, 2_000, "burn-wire consumables, AntS re-arm"),
        ("FM", "Residual dipole, Helmholtz cage", 0, 3_000, "university facility"),
        ("FM", "RF conducted power, spurious, TX-OFF, end-to-end", 0, 3_000, "in-house analyser to 1 lab day"),
        ("FM", "System EMC and RF self-compatibility (receiver-input emissions, spurious), protoflight step 8", 1_500,
         8_000, "1-2 lab days; in-house with a borrowed analyser at the low end"),
        ("FM", "Reliability programme: FM burn-in >= 200 h, 4 ambient thermal cycles (VI-12)", 2_000, 8_000,
         "10 days of a thermal chamber at $200-800/day; $0 with an owned chamber"),
        ("Both", "Transport, travel, fixtures", 1_000, 5_000, ""),
        ("Both", "Re-test allowance: 1 vibe axis + 2 TV cycles", 3_000, 10_000, "one anomaly"),
    ]
    lo = sum(l[2] for l in lines)
    hi = sum(l[3] for l in lines)
    print(f"{'Phase':5s} {'Item':62s} {'Low':>8s} {'High':>8s}  Basis")
    for ph, item, a, b, basis in lines:
        print(f"{ph:5s} {item:62s} {a:8,d} {b:8,d}  {basis}")
    print(f"{'':5s} {'Total':62s} {lo:8,d} {hi:8,d}")
    em_lo = sum(l[2] for l in lines if l[0] == "EM")
    em_hi = sum(l[3] for l in lines if l[0] == "EM")
    print(f"\nEM share {em_lo:,}-{em_hi:,}; FM and common {lo-em_lo:,}-{hi-em_hi:,}")
    print(f"ROM WBS 10: {ROM_WBS10[0]:,}-{ROM_WBS10[1]:,}; bottom-up {lo:,}-{hi:,}:"
          f" low end {100*(lo/ROM_WBS10[0]-1):+.0f} %, high end {100*(hi/ROM_WBS10[1]-1):+.0f} %")
    print("Battery abuse tests (V-BSA-9, BATTERY_SAFETY_ASSESSMENT D §3.1) are costed in ROM WBS 03, not here:"
          " Falcon 9 set $10k-15k on top of UN 38.3; ISS backup set $50k-75k.")
    print(f"Owning a bench vacuum chamber with a heated/cooled platen ($8k-20k [U]) would replace about"
          f" {em_days + pf_days:.0f} chamber days ({round((em_days+pf_days)*2_000):,}-{round((em_days+pf_days)*4_000):,})"
          " and allow the GEVS 12-cycle goal; the integrator may still require an accredited lab for bake-out.")


if __name__ == "__main__":
    main()
