#!/usr/bin/env python3
"""
Critical-path schedule for SMART-QSO, 2026-10-27 to a 2028 launch (CDR review, PM/SE/V&V).

Feeds mission/SCHEDULE.md (SQSO-SCH-001). The network is a plain critical-path model: each task
has a duration in calendar days, predecessors (finish-to-start) and an optional "start no
earlier than" date (funding, a decision date). The forward pass gives early dates; the backward
pass from the required delivery date gives late dates and total float. The critical path is the
chain with the least float.

Durations are planning estimates (sources in the TASKS comments): no vendor quote, lab booking
or integrator schedule exists yet, so the result is a planning baseline, not a commitment.

Margin rule: GSFC-STD-1000 (GOLD Rules) Rule 1.06 funded schedule margin, as used at GSFC:
1 month per year from now to the start of I&T, 2 months per year from the start of I&T to
shipment, 1 week per month from shipment to launch. (Text not retrieved in this review; the
rule is applied as commonly stated.)

    python3 software/simulation/cdr/pm/schedule_cpm.py                 # baseline (launch 2028-11-01, BASELINE Rev E §3)
    python3 software/simulation/cdr/pm/schedule_cpm.py --launch 2028-06-01   # the former planning date
    python3 software/simulation/cdr/pm/schedule_cpm.py --fm-at-risk    # FM long leads ordered at PDR
    python3 software/simulation/cdr/pm/schedule_cpm.py --em-funds 2027-03-01   # funding sensitivity
    python3 software/simulation/cdr/pm/schedule_cpm.py --layout-designers 1    # one layout designer (R14)
    python3 software/simulation/cdr/pm/schedule_cpm.py --layout-at-risk       # layout starts before the PDR
    python3 software/simulation/cdr/pm/schedule_cpm.py --no-reliability       # without the reliability programme
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from datetime import date, timedelta
from typing import Dict, List, Optional, Tuple

TODAY = date(2026, 10, 27)
W = 7  # days per week


@dataclass
class Task:
    tid: str
    name: str
    days: int
    preds: Tuple[str, ...] = ()
    snet: Optional[date] = None     # start no earlier than
    deadline: Optional[date] = None  # must finish by (checked, not forced)
    es: int = 0
    ef: int = 0
    ls: int = 0
    lf: int = 0
    succ: List[str] = field(default_factory=list)


def months_before(launch: date, m: float) -> date:
    """L - m months, as the integrator's deliverable table counts them (whole months on the day of month)."""
    whole = int(m)
    y, mo = launch.year, launch.month - whole
    while mo <= 0:
        mo += 12
        y -= 1
    out = date(y, mo, launch.day)
    return out - timedelta(days=round((m - whole) * 30.44))


#: EM PCB layout estimate (RID P3P-05), from the round-2 build-to designs. Basis: designer-hours =
#: placements x 0.25 h x complexity + 40 h per board (stack-up and constraints, netlist import from the
#: schematic-as-code, fabrication and assembly outputs, design-rule and peer review), + 20 % for review changes;
#: 32 productive hours a week per designer. Complexity 1.0 for the power and digital boards, 1.5 for the RF board
#: (impedance-controlled microstrip, via fences, shield cans), 2.0 for the Jetson carrier (260-pin SO-DIMM, PCIe and
#: USB striplines on a 6-layer 2.0 mm stack-up); the two small boards take 12 h fixed. Rates are a planning
#: assumption for a contract designer working in KiCad, not a quote.
LAYOUT_BOARDS: Dict[str, Tuple[str, int, float, float]] = {
    # id: (board, placements, complexity, fixed hours)
    "EPS": ("EPS board, SQSO-EPS-BRD-001 (PC/104, 6 layers)", 700, 1.0, 40.0),
    "OBC": ("OBC board, SQSO-OBC-BRD-001 (PC/104, 6 layers)", 351, 1.0, 40.0),
    "RF": ("RF board, SQSO-RF-BRD-001 (6 layers, 300 fitted)", 300, 1.5, 40.0),
    "CAR": ("Jetson carrier, SQSO-JET-BRD-001 (6 layers, 2.0 mm)", 137, 2.0, 40.0),
    "SMALL": ("Magnetometer board A2 and access-port board A3 (2 layers)", 23, 1.0, 24.0),
}
HOURS_PER_PLACEMENT = 0.25
REVIEW_CHANGES = 0.20
DESIGNER_HOURS_PER_WEEK = 32.0


def layout_weeks(board: str) -> int:
    """Whole weeks of one designer for one board (rounded up)."""
    _, n, cx, fixed = LAYOUT_BOARDS[board]
    hours = (n * HOURS_PER_PLACEMENT * cx + fixed) * (1.0 + REVIEW_CHANGES)
    return -int(-hours // DESIGNER_HOURS_PER_WEEK)


def tasks(fm_at_risk: bool, em_funds: date = date(2027, 1, 15), fm_funds: date = date(2027, 5, 31),
          launch: date = date(2028, 11, 1), designers: int = 2, layout_at_risk: bool = False,
          reliability: bool = True) -> List[Task]:
    d = date
    l8, l6, l4, l35 = (months_before(launch, m) for m in (8, 6, 4, 3.5))
    fm_pred = ("PDR", "FUND_FM") if fm_at_risk else ("CDR", "FUND_FM")
    lw = {b: layout_weeks(b) for b in LAYOUT_BOARDS}
    gate = "PDR_PKG" if layout_at_risk else "PDR"   # at risk: start on the frozen PDR package (DA-04)
    if designers >= 2:
        # designer A: EPS, then the carrier (after the NVIDIA guides, JETB-06); designer B: OBC, RF, then A2/A3
        lanes = {"L_EPS": (gate,), "L_CAR": ("L_EPS", "JET_DG"),
                 "L_OBC": (gate,), "L_RF": ("L_OBC", "RFB_PRE"), "L_SMALL": ("L_RF",)}
    else:
        lanes = {"L_EPS": (gate,), "L_OBC": ("L_EPS",), "L_RF": ("L_OBC", "RFB_PRE"),
                 "L_CAR": ("L_RF", "JET_DG"), "L_SMALL": ("L_CAR",)}
    layout = [Task(tid, f"EM layout: {LAYOUT_BOARDS[tid[2:]][0]} ({lw[tid[2:]]} wk, basis LAYOUT_BOARDS)",
                   lw[tid[2:]] * W, preds) for tid, preds in lanes.items()]
    return [
        # ---- review and design. The round-2 build-to designs captured every board's schematic as code with an
        # ---- ERC and BOM, and the mechanical CAD and FE model exist (SQSO-MECH-CAD-001): SCH and MECH are complete.
        Task("RID", "Close this review's RIDs (rounds 1-3); requirements rebaseline (CR-2026-007 to -015)", 6 * W),
        Task("SCH", "EM schematic capture (complete: schematic as code, ERC, BOM; SQSO-EPS/OBC/RF/JET-BRD-001)", 0),
        Task("MECH", "Mechanical CAD, FE and dynamic model (complete: SQSO-MECH-CAD-001, STRUCTURAL_ANALYSIS)", 0),
        Task("PDR_BOARD", "PI names the independent PDR board and its chair (DA-03, by 2026-12-01)", 5 * W,
             deadline=d(2026, 12, 1)),
        Task("PDR_PKG", "PDR data package to the board, two weeks before the review", 1 * W, ("RID", "PDR_BOARD", "SCH")),
        Task("PDR", "Independent PDR (NPR 7123.1D Table G-6), held as a peer review", 1 * W,
             ("PDR_PKG", "MECH"), snet=d(2027, 1, 5)),
        # ---- dated pre-layout and pre-order verification items (VI register, docs/verification/VI_REGISTER.md)
        Task("RFB_PRE", "VI-RFB-12 (RF lead): AntS and coax drawings, plating, attenuation, inner conductor (before RF layout)",
             2 * W + 3, deadline=d(2026, 11, 13)),
        Task("JET_DG", "JETB-06/V-JET-2, JETB-12: NVIDIA DG-10931-001 and TDG-11127-001, Molex drawings; conformance"
             " review before the carrier layout", 6 * W, deadline=d(2027, 2, 1)),
        Task("PRE_ORDER", "Procurement data reviews before the EM order: V-HAR-3, V-HAR-11, EPSB-09, JETB-07", 6 * W,
             deadline=d(2027, 1, 15)),
        *layout,
        Task("LAYOUT", "EM layout cross-board check (stack, harness, VI-MCAD-01 fit) and fabrication release", 1 * W,
             tuple(lanes)),
        Task("FUND_EM", "EM funding available (assumption)", 0, snet=em_funds),
        Task("EM_LL", "EM COTS buys: frame, AntS, magnetometer, gyro, GNSS, Jetson (12 wk, BVB §2)", 12 * W,
             ("PDR", "FUND_EM", "PRE_ORDER")),
        Task("PANELS", "Solar panels, EM and FM sets ordered together (3-7 months, BVB §2; 7 used)", 30 * W,
             ("PDR", "FUND_EM")),
        Task("EM_FAB", "EM PCB fabrication and assembly", 5 * W, ("LAYOUT", "PRE_ORDER")),
        Task("EM_INT", "EM integration, flatsat, flight software on target (TP-FLATSAT-001)", 8 * W,
             ("EM_FAB", "EM_LL")),
        Task("EM_TEST", "EM development tests: Jetson heat path and burst energy, EM TVAC, pathfinder vibe,"
             " shock if VI-07 requires it, RF/EMC pre-scan, GNSS noise, antenna pattern", 8 * W, ("EM_INT",)),
        Task("BATT_ABUSE", "Custom-battery abuse tests on sacrificial EM-design packs (BSA D §3.1, V-BSA-9;"
             " Falcon 9 set, 3-4 weeks; ISS backup set 8-12 weeks)", 4 * W, ("EM_FAB",)),
        Task("FLATSAT_500H", "Reliability programme: >= 500 h powered on the EM boards with flight software"
             " (REL §6; ETP §6, VI-11)", 4 * W if reliability else 0, ("EM_INT",)),
        Task("CDR", "Independent CDR (Table G-7) with EM data", 1 * W, ("EM_TEST", "LAYOUT", "FLATSAT_500H",
                                                                     "BATT_ABUSE")),
        Task("CDR_RID", "CDR RID closure", 3 * W, ("CDR",)),
        # ---- flight model
        Task("FUND_FM", "FM and launch funding committed (R11: decision by Q2 2027)", 0, snet=fm_funds),
        Task("FM_LL", "FM COTS buys", 12 * W, fm_pred),
        Task("FM_LAYOUT", "FM layout update from EM findings", 4 * W, ("CDR",)),
        Task("FM_FAB", "FM PCB fabrication, assembly, inspection", 6 * W, ("FM_LAYOUT", "FUND_FM")),
        Task("BATT", "Flight-lot cell screening, pack build, LEO cycling at 4.10 V (D8)", 10 * W, ("CDR",)),
        Task("FM_INT", "FM integration (TP-INT-001 to -006)", 6 * W, ("FM_FAB", "FM_LL", "PANELS", "BATT")),
        Task("FM_FUNC", "FM comprehensive functional and day-in-the-life (TP-SYS-001, -002)", 3 * W, ("FM_INT",)),
        Task("FM_BURNIN", "Reliability programme: FM burn-in >= 200 h powered, last 100 h failure-free, 4 ambient"
             " thermal cycles (REL §6; ETP §3.11, VI-12)", 10 if reliability else 0, ("FM_FUNC",)),
        Task("PF_ENV", "Protoflight campaign: mass properties, vibe, TVAC + bake-out, deployment hot/cold,"
             " EMC/self-compatibility, magnetic dipole, post-env CPT, transport (complete by L-4 M)", 7 * W,
             ("FM_BURNIN",), deadline=l4),
        Task("PSR", "Pre-ship review and integrator data package", 3 * W, ("ENV_REPORT",)),
        # ---- integrator deliverables (docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md §9, LVR-003, LVR-007)
        Task("MA_IN", "Mission-analysis inputs to the integrator (L-8 M: CAD, dynamic model, webforms)", 2 * W,
             ("MECH", "CONTRACT"), deadline=l8),
        Task("SDHS", "Hazardous-systems declaration and 91-710 letter (L-6 M)", 4 * W, ("CDR", "CONTRACT"), deadline=l6),
        Task("ENV_REPORT", "Environmental verification report on the SpaceX template (L-3.5 M)", 1 * W, ("PF_ENV",),
             deadline=l35),
        # ---- launch contract (decision D7)
        Task("ENTITY", "Contracting legal entity in place (ORBITAL_ACCESS DP-1)", 8 * W, deadline=d(2026, 12, 31)),
        Task("RFQ", "Integrator requests for quotes (D7: by 2026-12-31)", 9 * W, deadline=d(2026, 12, 31)),
        Task("QUOTES", "Quotes, evaluation, selection", 8 * W, ("RFQ",)),
        Task("NEGOT", "Contract negotiation, altitude clause, export terms", 12 * W, ("QUOTES", "ENTITY")),
        Task("CONTRACT", "Contract signature and deposit (D7: by 2027-06-30); launch-vehicle determination", 0,
             ("NEGOT", "FUND_FM"), deadline=d(2027, 6, 30)),
        Task("INT_DATA", "Integrator data at the contract: mission ICD, attenuated shock MPE, dispenser thermal"
             " analysis, coupled-loads approach (SE-05, VV-07, THM-06/07)", 0, ("CONTRACT",), deadline=d(2027, 6, 30)),
        Task("EXPORT", "Export counsel; TAA if the integrator is foreign (3-6 months, R15)", 20 * W, ("CONTRACT",)),
        # ---- regulatory (docs/regulatory/REGULATORY_ROADMAP.md §2)
        Task("TCS", "Name >= 2 telecommand stations (D1)", 10 * W),
        Task("FCC_REQ", "FCC Space Bureau consultation requested with the agenda (REGULATORY_ROADMAP D §6; DA-11)",
             19, deadline=d(2026, 11, 15)),
        Task("FCC_CONS", "FCC Space Bureau consultation (D3): meeting", 51, ("FCC_REQ",), deadline=d(2027, 1, 5)),
        Task("FCC_OUT", "FCC consultation outcome recorded (DA-11)", 3, ("FCC_CONS",), deadline=d(2027, 1, 8)),
        Task("DAS_REQ", "NASA DAS software usage agreement requested (DA-06)", 10, deadline=d(2026, 11, 6)),
        Task("DAS_SW", "NASA DAS agreement in hand (DA-06)", 39, ("DAS_REQ",), deadline=d(2026, 12, 15)),
        Task("DAS", "NASA DAS run reproducing ODAR Draft D (DA-06)", 24, ("DAS_SW",), deadline=d(2027, 1, 8)),
        Task("ODAR_E", "ODAR Draft E with the DAS output, for the early filing (DA-06)", 4, ("DAS",),
             deadline=d(2027, 1, 12)),
        Task("ANTS_RFQ", "ISIS AntS RFQ, with the ICD (VI-RFB-13, AV-V7, DA-17)", 10, deadline=d(2026, 11, 6)),
        Task("FCC_EARLY", "Early FCC pre-space notification with the API data and ODAR Draft E", 1 * W,
             ("FCC_OUT", "TCS", "ODAR_E"), snet=d(2027, 1, 12), deadline=d(2027, 1, 19)),
        Task("IARU_REQ", "IARU coordination request submitted (form v40), after the PDR and the early FCC filing",
             1 * W, ("TCS", "FCC_OUT", "PDR", "FCC_EARLY")),
        Task("IARU", "IARU panel iteration to coordination letter (90th percentile 112 d; median 57 d)", 16 * W,
             ("IARU_REQ",)),
        Task("FCC_PRE", "FCC 97.207(g)(1) replacement notification (<= 30 d after LV determination)", 4 * W,
             ("CONTRACT", "IARU", "DAS", "FCC_EARLY")),
        # ---- delivery
        Task("DELIVER", "Delivery to the integrator", 0, ("PSR", "IARU", "FCC_PRE", "EXPORT", "MA_IN", "SDHS",
                                                          "INT_DATA", "CDR_RID")),
    ]


def solve(ts: List[Task], finish: date) -> Dict[str, Task]:
    by = {t.tid: t for t in ts}
    for t in ts:
        for p in t.preds:
            by[p].succ.append(t.tid)
    order: List[str] = []
    seen = set()

    def visit(tid: str) -> None:
        if tid in seen:
            return
        seen.add(tid)
        for p in by[tid].preds:
            visit(p)
        order.append(tid)

    for t in ts:
        visit(t.tid)
    for tid in order:
        t = by[tid]
        t.es = max([by[p].ef for p in t.preds] + [(t.snet - TODAY).days if t.snet else 0])
        t.ef = t.es + t.days
    end = (finish - TODAY).days
    for tid in reversed(order):
        t = by[tid]
        t.lf = min([by[s].ls for s in t.succ] + [end] + ([(t.deadline - TODAY).days] if t.deadline else []))
        t.ls = t.lf - t.days
    return by


def day(n: int) -> date:
    return TODAY + timedelta(days=n)


def critical_chain(by: Dict[str, Task], last: str) -> List[str]:
    chain = [last]
    while by[chain[-1]].preds:
        t = by[chain[-1]]
        # the driving predecessor: the one whose early finish sets this task's early start
        drv = max(t.preds, key=lambda p: by[p].ef)
        if by[drv].ef < t.es:   # driven by a start-no-earlier-than date, not a predecessor
            break
        chain.append(drv)
    return list(reversed(chain))


def months(a: date, b: date) -> float:
    return (b - a).days / 30.44


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--launch", default="2028-11-01", help="target launch date (default %(default)s)")
    ap.add_argument("--delivery-days", type=int, default=50,
                    help="delivery to integrator before launch (LAUNCH_VEHICLE_REQUIREMENTS §9: L-50 d)")
    ap.add_argument("--fm-at-risk", action="store_true", help="order FM long leads at PDR, not CDR")
    ap.add_argument("--em-funds", default="2027-01-15", help="EM funding available (default %(default)s)")
    ap.add_argument("--fm-funds", default="2027-05-31", help="FM and launch funding (default %(default)s)")
    ap.add_argument("--layout-designers", type=int, default=2, choices=(1, 2),
                    help="PCB layout designers working in parallel (default %(default)s; DA-04)")
    ap.add_argument("--layout-at-risk", action="store_true",
                    help="start the EM layout on the PDR data package, before the PDR (DA-04)")
    ap.add_argument("--no-reliability", action="store_true",
                    help="drop the reliability programme: EM flatsat 500 h and FM burn-in (DA-02)")
    a = ap.parse_args()
    launch = date.fromisoformat(a.launch)
    deliver = launch - timedelta(days=a.delivery_days)
    integ = launch - timedelta(days=47)
    by = solve(tasks(a.fm_at_risk, date.fromisoformat(a.em_funds), date.fromisoformat(a.fm_funds), launch,
                     a.layout_designers, a.layout_at_risk, not a.no_reliability), deliver)

    print(f"Launch {launch}, delivery to integrator {deliver} (L-{a.delivery_days} d), "
          f"dispenser integration about {integ} (L-47 d)")
    print(f"{'ID':9s} {'early start':11s} {'early finish':12s} {'late finish':11s} {'float':>6s}  task")
    for t in sorted(by.values(), key=lambda t: (t.es, t.tid)):
        flag = " *" if t.lf - t.ef <= 0 else ""
        print(f"{t.tid:9s} {day(t.es)} {day(t.ef)}   {day(t.lf)} {t.lf - t.ef:5d}d{flag} {t.name}")
    for t in by.values():
        if t.deadline and day(t.ef) > t.deadline:
            print(f"!! {t.tid} finishes {day(t.ef)}, after its deadline {t.deadline}")
        elif t.deadline:
            print(f"   {t.tid} finishes {day(t.ef)}, deadline {t.deadline}: {(t.deadline - day(t.ef)).days} d to spare")
    d = by["DELIVER"]
    # margin of the flight-hardware chain: the protoflight campaign (L-4 M), its report (L-3.5 M), the
    # integrator inputs and the delivery; the administrative deadlines (RFQ, entity) are reported above
    hw = [by[k] for k in ("PF_ENV", "ENV_REPORT", "MA_IN", "SDHS", "DELIVER")]
    worst = min(hw, key=lambda t: (t.lf - t.ef, -t.ef))
    flt = worst.lf - worst.ef
    chain = critical_chain(by, worst.tid)
    print(f"\nDelivery float {d.lf - d.ef} d; least float {flt} d at {worst.tid} ({worst.name})")
    print(f"Earliest delivery {day(d.ef)}; required {deliver}; total float on the critical path {flt} d"
          f" ({flt / 7:.1f} weeks)")
    print("Critical path: " + " -> ".join(chain))
    # regulatory windows
    fcc = by["FCC_PRE"]; con = by["CONTRACT"]
    print(f"FCC pre-space notification filed {day(fcc.ef)}: {(day(fcc.ef) - day(con.ef)).days} d after the"
          f" contract (rule <= 30 d); {(integ - day(fcc.ef)).days} d before integration (rule >= 90 d)")
    iaru = by["IARU_REQ"]
    print(f"IARU request {day(iaru.ef)}: {months(day(iaru.ef), integ):.1f} months before integration"
          f" (roadmap: >= 12 months; T-18 months target)")
    # GOLD Rule 1.06 margin
    it_start = day(by["FM_INT"].es)
    bind = day(worst.lf)   # the binding deadline of the flight-hardware chain
    need = months(TODAY, it_start) / 12 * 1.0 + months(it_start, bind) / 12 * 2.0
    need_wk = need * 30.44 / 7
    ship_to_launch = months(deliver, launch) * 7 / 7  # weeks: 1 week per month
    print(f"GOLD 1.06 margin wanted to the binding deadline {bind}: {need:.1f} months ({need_wk:.1f} weeks) ="
          f" 1 mo/yr to I&T start {it_start} + 2 mo/yr I&T to that deadline; plus {ship_to_launch:.1f} weeks ship-to-launch held by the integrator")
    print(f"Margin available {flt / 7:.1f} weeks -> shortfall {need_wk - flt / 7:.1f} weeks"
          if flt / 7 < need_wk else f"Margin available {flt / 7:.1f} weeks -> meets the rule")


if __name__ == "__main__":
    main()
