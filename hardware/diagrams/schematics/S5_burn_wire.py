"""S5 deployment burn-wire drivers and deployed-state sense (antenna pair and solar-wing pair)."""

import schemdraw.elements as elm

import sch

d = sch.drawing("S5_burn_wire")
sch.title(d, (-4, 5.5), "S5", "Burn-wire drivers and deployed-state sense",
          "Sources: software/rust/firmware/src/gpio.rs (PB9-PB12, PA1); hardware/rf/ANTENNA_DESIGN.md; "
          "docs/BASELINE_PARAMETERS.md 4 (timers, inhibits); W7")

CH = [("antenna", "ANTENNA_BURN (PB9)", "ANTENNA_SENSE (PB11)", ("VHF", "UHF"), "J_DEP_ANT"),
      ("solar wing", "SOLAR_BURN (PB10)", "SOLAR_SENSE (PB12)", ("W1", "W2"), "J_DEP_SA")]

for k, (what, burn, sense, pair, conn) in enumerate(CH):
    y = -k * 14.0
    x = 0
    # supply from SW6 (arm), two burn resistors in parallel
    sch.flag(d, (x + 3, y + 2.5), "VBAT_DEPLOY (SW6)", "left", 0.01)
    d += elm.Line().at((x + 3, y + 2.5)).right().length(5.0)
    for j, el in enumerate(pair):
        xr = x + 4.5 + j * 3.0
        d += elm.Dot().at((xr, y + 2.5))
        d += elm.Resistor().at((xr, y + 2.5)).down().length(2.2).label(f"R{el} burn\n~5 Ohm (TBR)\n{conn} {2 * j + 1}-{2 * j + 2}",
                                                                      loc="bottom", fontsize=7)
        d += elm.Line().down().toy(y - 0.6)
    d += elm.Line().at((x + 4.5, y - 0.6)).right().length(3.0)
    d += elm.Line().at((x + 6.0, y - 0.6)).down().length(0.4)
    d += (q := elm.NFet().right().anchor("drain").label(f"Q{k + 1} low-side\nNMOS (TBD)", loc="left", fontsize=7))
    d += elm.Resistor().at(q.source).down().length(1.6).label("Rs 0.1 Ohm (TBR)", loc="bottom", fontsize=7)
    d += elm.Ground()
    d += elm.Line().at(q.gate).right().length(1.2)
    d += (g := elm.Dot())
    d += elm.Resistor().at(g.center).down().length(1.6).label("100k", loc="bottom", fontsize=7)
    d += elm.Ground()
    d += elm.Resistor().at(g.center).right().length(1.6).label("100 Ohm", loc="top", fontsize=7)
    sch.flag(d, (g.center[0] + 1.6, g.center[1]), burn, "right", 0.4)
    sch.note(d, (x + 9.5, y + 2.6),
             f"One burn line for the two {what}s (finding F4): two resistors in parallel on one driver.\n"
             "Two series switches in the deploy path: SW6 (DEPLOY_EN, high side) and Q (low side),\n"
             "after DS1/DS2 and the 30-minute timer (CDS 2.4.4); a third deployable inhibit is TBD (F5).\n"
             "5 Ohm on 14.4 V draws about 2.9 A per resistor: the drive must limit it to 0.5-1 A\n"
             "(PWM on PB9 TIM4_CH4 / PB10 TIM2_CH3, or a linear limit; finding F7, TBR).", fontsize=7)

    # deployed-state sense: the pair's switches in series, pull-up, RC filter
    ys = y - 3.5
    xs = x + 16
    d += elm.Vdd().at((xs, ys + 2.0)).label("3V3_AO", fontsize=7)
    d += elm.Resistor().at((xs, ys + 2.0)).down().length(1.6).label("10k", loc="top", fontsize=7)
    d += (n := elm.Dot())
    d += elm.Switch(action="close").at(n.center).down().length(1.6).label(f"S {pair[0]}", loc="top", fontsize=7)
    d += elm.Switch(action="close").down().length(1.6).label(f"S {pair[1]}", loc="top", fontsize=7)
    d += elm.Ground()
    d += elm.Resistor().at(n.center).right().length(1.8).label("1k", loc="top", fontsize=7)
    d += (m := elm.Dot())
    d += elm.Capacitor().at(m.center).down().length(1.2).label("100 nF", loc="right", fontsize=7)
    d += elm.Ground()
    sch.flag(d, m.center, sense, "right", 0.8)
    sch.note(d, (xs + 6.5, ys + 1.0), "Switches closed when deployed.\nLow = both deployed (series chain, TBR):\n"
             "one stuck switch reads 'not deployed',\nso the software retries the burn (fail-safe side).",
             fontsize=7)

d.save(sch.out_path("S5_burn_wire"))
