#!/usr/bin/env python3
"""Battery-bus voltage under load steps, by pack temperature and state of charge (SQSO-EMC-001 §4, EPS_DESIGN §4).

The pack model is the mission simulation's (`sqso-sil/src/env/battery.rs`): per cell
R0 = 40 mOhm at 25 °C and mid SOC, Arrhenius 28 kJ/mol, up to x1.6 at empty; R1 = 15 mOhm
(the RC pair, tau 30 s, counted fully for steps longer than a minute); OCV table of a
high-energy 18650. Harness and fuse: 30 mOhm. The steps are the largest the loads make:

* Jetson power-on to 8 W on the 5 V rail (buck 90 %): 0.62 A at 14.4 V
* Jetson latch-up to the guard's 12 W trip: 0.93 A
* PA keyed at 1 W RF (2.7 W DC, buck 90 %): 0.21 A
* Wing deployment burn, one resistor at a time (SQSO-EPS-BRD-001 §9; four independent burn
  lines PB10, PE6, PB9, PE7; 0.8 A driver fed from the 7.0 V burn buck, 8 s nominal, 12 s
  hardware on-time limit). Pack current: 0.67 A nominal (burn 0.51 A with housekeeping and
  heater), 0.75 A worst case (magnetorquers at 1.16 W as well), 1.0 A the chair's limit
  (it also covers 0.90 A with the coil bridges at their hardware limit). The RC pair is
  counted fully for the burn, which is conservative for an 8-12 s step.
* Bound: all four burn drivers on (a software fault), held to 2.0 A at the pack by the
  burn buck's current limit.

The two-resistor 1.7 A burn of earlier revisions is no longer a design case.

It prints the bus voltage at the end of each step against the BMS under-voltage cut
(2.8 V/cell, 11.2 V), the flight software's low-battery level (12.0 V) and the
buck under-voltage lock-out it implies, and the peak-to-peak bus disturbance of the
PA keying that the environmental test plan bounds at 200 mV.

Usage: python3 bus_transients.py
"""

import math

R0_25, R1_25, EA, R_GAS = 0.040, 0.015, 28000.0, 8.314
SOC_TAB = [0.0, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 1.00]
OCV_TAB = [2.90, 3.30, 3.43, 3.50, 3.55, 3.58, 3.61, 3.66, 3.72, 3.80, 3.88, 3.97, 4.07, 4.19]
HARNESS = 0.030
CELLS = 4


def interp(x, xs, ys):
    if x <= xs[0]:
        return ys[0]
    for i in range(1, len(xs)):
        if x <= xs[i]:
            f = (x - xs[i - 1]) / (xs[i] - xs[i - 1])
            return ys[i - 1] + f * (ys[i] - ys[i - 1])
    return ys[-1]


def arr(temp_c):
    return math.exp(EA / R_GAS * (1.0 / (temp_c + 273.15) - 1.0 / 298.15))


def pack_r(temp_c, soc, long_step):
    empty = 1.0 + 0.6 * max(0.0, 0.3 - soc) / 0.3
    r = R0_25 * arr(temp_c) * empty + (R1_25 * arr(temp_c) if long_step else 0.0)
    return CELLS * r + HARNESS


def main():
    steps = [("Jetson on, 8 W", 0.62, True), ("Jetson latch-up, 12 W", 0.93, False),
             ("PA keyed, 1 W RF", 0.21, False),
             ("Burn, one resistor, 0.67 A (nominal)", 0.67, True),
             ("Burn, one resistor, 0.75 A (worst case)", 0.75, True),
             ("Burn at the 1.0 A limit", 1.0, True),
             ("Four drivers (fault), 2.0 A (buck limit)", 2.0, True)]
    print("| Pack | SOC | OCV (V) | R pack (Ω, short / long) | " + " | ".join(s[0] for s in steps) + " |")
    print("|---|---|---|---|" + "---|" * len(steps))
    for temp in (25.0, 0.0, -10.0, -20.0):
        for soc in (0.90, 0.55, 0.25, 0.15):
            ocv = CELLS * interp(soc, SOC_TAB, OCV_TAB)
            cells = []
            for name, amps, long_step in steps:
                v = ocv - amps * pack_r(temp, soc, long_step)
                flag = " **< 11.2 UV**" if v < 11.2 else (" *< 12.0*" if v < 12.0 else "")
                cells.append(f"{v:.2f}{flag}")
            print(f"| {temp:+.0f} °C | {soc * 100:.0f} % | {ocv:.2f} | {pack_r(temp, soc, False):.3f} / {pack_r(temp, soc, True):.3f} | "
                  + " | ".join(cells) + " |")
    dv = 0.21 * pack_r(25.0, 0.9, False)
    dv_cold = 0.21 * pack_r(0.0, 0.25, False)
    print(f"\nPA keying step on the bus: {dv * 1000:.0f} mV at 25 °C and 90 % SOC, {dv_cold * 1000:.0f} mV at 0 °C and 25 % "
          "(the 200 mV ripple criterion is for ripple; the step is below it in every case listed)")
    # Lowest SOC at which a burn keeps the bus above each level, by pack temperature
    print("\nLowest state of charge (%) at which the burn keeps the bus at or above 12.0 V / 11.2 V:")
    print("| Pack | 0.67 A | 0.75 A | 1.0 A | 2.0 A (fault) |")
    print("|---|---|---|---|---|")
    for temp in (25.0, 0.0, -10.0, -16.0, -20.0):
        cells = []
        for amps in (0.67, 0.75, 1.0, 2.0):
            out = []
            for level in (12.0, 11.2):
                soc_min = None
                for k in range(0, 101):
                    soc = k / 100.0
                    if CELLS * interp(soc, SOC_TAB, OCV_TAB) - amps * pack_r(temp, soc, True) >= level:
                        soc_min = k
                        break
                out.append("—" if soc_min is None else f"{soc_min}")
            cells.append(" / ".join(out))
        print(f"| {temp:+.0f} °C | " + " | ".join(cells) + " |")


if __name__ == "__main__":
    main()
