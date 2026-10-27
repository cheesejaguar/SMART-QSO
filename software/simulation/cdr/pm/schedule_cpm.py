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

    python3 software/simulation/cdr/pm/schedule_cpm.py                 # baseline (launch 2028-06-01)
    python3 software/simulation/cdr/pm/schedule_cpm.py --launch 2028-11-01
    python3 software/simulation/cdr/pm/schedule_cpm.py --fm-at-risk    # FM long leads ordered at PDR
    python3 software/simulation/cdr/pm/schedule_cpm.py --em-funds 2027-03-01   # funding sensitivity
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


def tasks(fm_at_risk: bool, em_funds: date = date(2027, 1, 15), fm_funds: date = date(2027, 5, 31)) -> List[Task]:
    d = date
    fm_pred = ("PDR", "FUND_FM") if fm_at_risk else ("CDR", "FUND_FM")
    return [
        # ---- review and design (no schematics or layouts exist: HARDWARE_READINESS_TRACKER v2.0)
        Task("RID", "Close this review's RIDs; requirements rebaseline (CR-2026-007)", 6 * W),
        Task("SCH", "EM schematic capture: OBC, EPS, transponder, Jetson carrier, battery mount", 10 * W),
        Task("PDR", "Independent PDR (NPR 7123.1D Table G-6), held as a peer review", 1 * W, ("RID", "SCH")),
        Task("LAYOUT", "EM PCB layout and design-rule review", 8 * W, ("PDR",)),
        Task("FUND_EM", "EM funding available (assumption)", 0, snet=em_funds),
        Task("EM_LL", "EM COTS buys: frame, AntS, magnetometer, gyro, GNSS, Jetson (12 wk, BVB §2)", 12 * W,
             ("PDR", "FUND_EM")),
        Task("PANELS", "Solar panels, EM and FM sets ordered together (3-7 months, BVB §2; 7 used)", 30 * W,
             ("PDR", "FUND_EM")),
        Task("EM_FAB", "EM PCB fabrication and assembly", 5 * W, ("LAYOUT",)),
        Task("EM_INT", "EM integration, flatsat, flight software on target (TP-FLATSAT-001)", 8 * W,
             ("EM_FAB", "EM_LL")),
        Task("EM_TEST", "EM development tests: Jetson heat path and burst energy, EM TVAC, pathfinder vibe,"
             " RF/EMC pre-scan, GNSS noise, antenna pattern", 8 * W, ("EM_INT",)),
        Task("CDR", "Independent CDR (Table G-7) with EM data", 1 * W, ("EM_TEST", "LAYOUT")),
        Task("CDR_RID", "CDR RID closure", 3 * W, ("CDR",)),
        # ---- flight model
        Task("FUND_FM", "FM and launch funding committed (R11: decision by Q2 2027)", 0, snet=fm_funds),
        Task("FM_LL", "FM COTS buys", 12 * W, fm_pred),
        Task("FM_LAYOUT", "FM layout update from EM findings", 4 * W, ("CDR",)),
        Task("FM_FAB", "FM PCB fabrication, assembly, inspection", 6 * W, ("FM_LAYOUT", "FUND_FM")),
        Task("BATT", "Flight-lot cell screening, pack build, LEO cycling at 4.10 V (D8)", 10 * W, ("CDR",)),
        Task("FM_INT", "FM integration (TP-INT-001 to -006)", 6 * W, ("FM_FAB", "FM_LL", "PANELS", "BATT")),
        Task("FM_FUNC", "FM comprehensive functional and day-in-the-life (TP-SYS-001, -002)", 3 * W, ("FM_INT",)),
        Task("PF_ENV", "Protoflight campaign: mass properties, vibe, TVAC + bake-out, deployment hot/cold,"
             " EMC/self-compatibility, magnetic dipole, post-env CPT, transport", 7 * W, ("FM_FUNC",)),
        Task("PSR", "Pre-ship review and integrator data package", 3 * W, ("PF_ENV",)),
        # ---- launch contract (decision D7)
        Task("RFQ", "Integrator requests for quotes (D7: by 2026-12-31)", 9 * W, deadline=d(2026, 12, 31)),
        Task("QUOTES", "Quotes, evaluation, selection", 8 * W, ("RFQ",)),
        Task("NEGOT", "Contract negotiation, altitude clause, export terms", 12 * W, ("QUOTES",)),
        Task("CONTRACT", "Contract signature and deposit (D7: by 2027-06-30); launch-vehicle determination", 0,
             ("NEGOT", "FUND_FM"), deadline=d(2027, 6, 30)),
        Task("EXPORT", "Export counsel; TAA if the integrator is foreign (3-6 months, R15)", 20 * W, ("CONTRACT",)),
        # ---- regulatory (docs/regulatory/REGULATORY_ROADMAP.md §2)
        Task("TCS", "Name >= 2 telecommand stations (D1)", 10 * W),
        Task("FCC_CONS", "FCC Space Bureau consultation (D3)", 10 * W),
        Task("DAS", "NASA DAS run and ODAR final (after MEL re-run)", 4 * W, ("PDR",)),
        Task("IARU_REQ", "IARU coordination request submitted (form v40)", 2 * W, ("TCS", "FCC_CONS", "PDR")),
        Task("IARU", "IARU panel iteration to coordination letter (about 6 months)", 26 * W, ("IARU_REQ",)),
        Task("FCC_PRE", "FCC 97.207(g)(1) pre-space notification (<= 30 d after LV determination)", 4 * W,
             ("CONTRACT", "IARU_REQ", "DAS")),
        # ---- delivery
        Task("DELIVER", "Delivery to the integrator", 0, ("PSR", "IARU", "FCC_PRE", "EXPORT")),
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
        t.lf = min([by[s].ls for s in t.succ] + [end])
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
    ap.add_argument("--launch", default="2028-06-01", help="target launch date (default %(default)s)")
    ap.add_argument("--delivery-days", type=int, default=60, help="delivery to integrator before launch")
    ap.add_argument("--fm-at-risk", action="store_true", help="order FM long leads at PDR, not CDR")
    ap.add_argument("--em-funds", default="2027-01-15", help="EM funding available (default %(default)s)")
    ap.add_argument("--fm-funds", default="2027-05-31", help="FM and launch funding (default %(default)s)")
    a = ap.parse_args()
    launch = date.fromisoformat(a.launch)
    deliver = launch - timedelta(days=a.delivery_days)
    integ = launch - timedelta(days=45)
    by = solve(tasks(a.fm_at_risk, date.fromisoformat(a.em_funds), date.fromisoformat(a.fm_funds)), deliver)

    print(f"Launch {launch}, delivery to integrator {deliver} (L-{a.delivery_days} d), "
          f"dispenser integration about {integ} (L-45 d)")
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
    flt = d.lf - d.ef
    chain = critical_chain(by, "DELIVER")
    print(f"\nEarliest delivery {day(d.ef)}; required {deliver}; total float on the critical path {flt} d"
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
    need = months(TODAY, it_start) / 12 * 1.0 + months(it_start, deliver) / 12 * 2.0
    need_wk = need * 30.44 / 7
    ship_to_launch = months(deliver, launch) * 7 / 7  # weeks: 1 week per month
    print(f"GOLD 1.06 margin wanted to delivery: {need:.1f} months ({need_wk:.1f} weeks) = 1 mo/yr to I&T start"
          f" {it_start} + 2 mo/yr I&T to ship; plus {ship_to_launch:.1f} weeks ship-to-launch held by the integrator")
    print(f"Margin available {flt / 7:.1f} weeks -> shortfall {need_wk - flt / 7:.1f} weeks"
          if flt / 7 < need_wk else f"Margin available {flt / 7:.1f} weeks -> meets the rule")


if __name__ == "__main__":
    main()
