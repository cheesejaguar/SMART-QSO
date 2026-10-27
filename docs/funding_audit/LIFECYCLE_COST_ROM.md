# Lifecycle Cost — Rough Order of Magnitude (ROM)
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-AUD-005
**Revision:** F
**Date:** 2026-10-27
**Status:** **ROM, not a budget.** No vendor quotes have been obtained. Ranges come from analogous COTS CubeSat
pricing and engineering judgment unless a source is cited. Accuracy class: about −30 % / +60 %. Replace each line with
a quote before any funding agreement is signed.
**Basis:** the funding-audit recommended baseline ([BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md)) and launch path ([trade study](LAUNCH_RIDESHARE_TRADE_STUDY.md))

---

## 1. Why This Document Exists

The repository has no cost estimate, WBS-based budget or funding profile (see findings register). Funders cannot
evaluate a proposal without one. This ROM sets out the cost structure and the dominant cost drivers so a real budget
can be built.

## 2. Assumptions

1. Protoflight approach: one flatsat/engineering set plus one protoflight unit (PFU); no qualification unit (Class D).
2. Launch on a commercial SSO rideshare through an integrator in 2028 (trade baseline).
3. Part 97 licensing: no FCC, IARU or ITU fees. The ITU offers amateur-satellite registration "without any fee of cost
   recovery" (IARU coordination instructions) [P]. NOAA charges no fee and no license is expected [P].
4. Two years of operations by licensed amateurs, **uncompensated for operations** (a Part 97 condition, see
   [pathway determination](../regulatory/LICENSING_PATHWAY_DETERMINATION.md) §5 C2).
5. FY2027 dollars, no escalation.

## 3. Cost by NASA Standard WBS (NPR 7120.5 Appendix G structure)

| WBS | Element | Low | High | Basis / driver |
|-----|---------|-----|------|----------------|
| 01 | Project management | in labor | in labor | See §4 |
| 02 | Systems engineering (incl. DAS/ODAR, budgets) | in labor | in labor | See §4 |
| 03 | Safety & mission assurance: battery screening, inspections, custom-battery abuse tests (V-BSA-9) | $13k | $25k | Cell screening lot and inspection tools $3k–10k; abuse-test set for the Falcon 9 path $10k–15k on top of UN 38.3, 3–4 weeks (`docs/environment/BATTERY_SAFETY_ASSESSMENT.md` D §3.1). The ISS backup's set is **$50k–75k [U], 8–12 weeks** instead |
| 05 | AI payload: three Jetson Orin Nano 8 GB modules (EM, FM, spare), NVMe card, heat spreader machining | $2k | $6k | Module list price is in the hundreds of dollars [U]; the carrier board is in the custom-board line of WBS 06 |
| 06 | Spacecraft bus, PFU, bought items: structure with the three deployment switches (DS2 two-pole for the third array inhibit, P3E-01), solar panels and wings, ISIS AntS, three CubeSpace CR0002 rods, Epson M-G370PDT0 gyro, flight-lot cells and pack hardware, harness materials | $20k | $55k | [U]: structure and switches $3–6k; solar $10–30k; AntS $3–8k; rods $1.5–4k; gyro $1–1.5k; cells and pack hardware $0.5–2k; harness $1–3k. The EPS, OBC, transponder and Jetson carrier are custom boards (`docs/hardware/BUILD_VS_BUY.md`), costed in the next lines, not as COTS units (RID P3P-08) |
| 06 | GNSS receiver and patch antenna, EM and FM sets (D6) | $9k | $19k | piNAV-NG/FM EUR 3.3–6.2k or Celeste EUR 5.2k, plus ZENITH patch EUR 0.8k (list prices, not quotes) [U]; Czech export licence handled by the vendor |
| 06 | Custom boards, EM and FM sets: EPS, OBC, RF, Jetson carrier, A2, A3 (≈ 1,560 placements a set) | $26k | $66k | Per set: parts $4k–9k (the BOMs of SQSO-EPS/OBC/RF/JET-BRD-001); fabrication, 6 layers, IPC-6012 Class 3/A, impedance-controlled, small lots $6k–16k; assembly and inspection $3k–8k [U] |
| 06 | Engineering-model bought items (structure, AntS, panel set ordered with the FM, rods, gyro, cells) | $14k | $50k | 70–100 % of the PFU bought items |
| 06 | EM PCB layout by contract (27 designer-weeks, 864 h; `mission/SCHEDULE.md` §2 basis; DA-04) | $0 | $105k | $0 if in kind or within the electrical labour of §4; contract rate $60–120/h [U] |
| 07 | Mission operations: two telecommand stations (decision C-14: ≥ 100 W PEP at the 70 cm feed), SatNOGS integration, 2 yr operations consumables | $9k | $19k | SQSO-GND-001 Rev F §3.1: variant A (outside US270) ≈ $6.65k a station (≈ $4.9k of antennas, rotator, transceiver, keys and time, plus ≈ $1.75k for the class AB ≥ 150 W amplifier, sequencer, 40 A supply and low-loss feed); variant B (US270 sites, e.g. the Bay Area: 50 W into a stacked pair, ±2° rotator, hardline) ≈ $7.35k; one of each ≈ $14k; low: one station's equipment in kind; ops labor uncompensated (Part 97) |
| 08 | Launch services (1U via integrator) | $85k | $150k | **[U]** No provider publishes a 1U price (seven checked, D7). SpaceX base $7,500/kg (2027), 50 kg minimum [P]; anchors: $86k (2026 community post), ~$85–90k (legacy Nanoracks) [U]. Hold 15 % of the slot price in the reserve for rebooking or storage |
| 08 | Export counsel, classification, possible TAA | $10k | $40k | Needed if the integrator is foreign ([export memo](../regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md)) [U] |
| 08 | Shipping, travel, customs, integration campaign | $5k | $15k | D7 research [U] |
| 09 | Ground data system (beacon decoder, dashboards; mostly existing software) and orbit determination with CCSDS OEM upload for SpaceX UG §9.5.7.2 (D6) | $0 | $10k | Existing `software/ground` tools; open-source astrodynamics library; effort is in labor (§4) |
| 10 | Integration & test, EM and protoflight: random vibration (protoflight 8.18 Grms envelope), sine, TVAC (8 cycles) and bake-out, EMC (EM pre-scan and FM system), EM shock at MPE + 3 dB (carried until VI-07 decides), Helmholtz-cage magnetic test, mass properties (trifilar MOI), deployment hot and cold, test-pod rental, the reliability programme (≥ 500 h EM flatsat, ≥ 200 h FM burn-in in a thermal chamber), one re-test allowance | $42k | $130k | Bottom-up in `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev D §7 (`software/simulation/cdr/pm/test_cost.py`, every item named here costed); lab day rates $2k–5k assumed [U] |
| 11 | Education / public outreach (beacon decoder releases, documentation) | $0 | $10k | — |
| 01 | Legal entity (formation, registered agent), launch-services legal review, insurance declarations and FAA cross-waivers (SQSO-LVR-001 LVR-009, LVR-011) | $3k | $20k | [U]; depends on the entity form and the integrator's terms |
| 02 | Independent reviews: PDR, CDR and pre-ship review boards (reviewer travel and honoraria; `docs/project/PROJECT_PLAN.md` §6.2) | $0 | $15k | $0–5k a review [U]; volunteer reviewers at the low end |
| — | **Subtotal, non-labor** | **$238k** | **$735k** | |
| — | Reserve on non-labor (30 %) | $71k | $221k | Class D practice; reserve is held by the PI, not spread across lines. It must also cover about 5 months of storage or a rebooking if the launch slips from the 2028-11 planning date |
| — | **Total non-labor incl. reserve** | **$309k** | **$956k** | |

## 4. Labor (the dominant uncertainty)

The program today has a design-phase document set, flight software in Rust that builds for the STM32L4R5 (Embassy
firmware) and runs in the mission simulation but has not run on hardware, and no hardware (see findings register).
Estimated effort to reach delivery and launch:

| Role | FTE | Duration | FTE-years |
|------|-----|----------|-----------|
| PI / project manager / systems engineer | 0.75 | 24 mo | 1.5 |
| Electrical / RF (EPS integration, transponder, OBC-Jetson interface) | 1.0 | 18 mo | 1.5 |
| Flight software (bring-up on the target, drivers for the selected parts, hardware-in-the-loop, FDIR on target) | 1.0 | 24 mo | 2.0 |
| Mechanical / thermal (CAD and dynamic model for the integrator at L-8 M, Jetson thermal path, structure, deployables) | 0.5 | 18 mo | 0.75 |
| Integration & test | 0.5 | 12 mo | 0.5 |
| **Total** | | | **≈ 6.25** |

| Labor model | ROM |
|-------------|-----|
| Fully volunteer / in-kind (amateur-radio club model) | $0 cash. Funders should still see in-kind value of about **$0.6M–$1.1M** at $100k–$175k per FTE-year |
| University (students plus part-time staff) | **$0.3M–$0.6M** |
| Professional engineering contract | **$0.9M–$1.3M** (burdened $150k–$200k per FTE-year) |

> Paying people to **build** the spacecraft is compatible with Part 97 under grant-type funding. Paying anyone to
> **operate** it is not (IARU form item 4.10; 47 CFR 97.113(a)(2)–(3)).

## 5. Lifecycle Total (ROM)

| Scenario | Non-labor incl. reserve | Labor (cash) | **Total cash** |
|----------|------------------------|--------------|----------------|
| Volunteer build | $0.31M–$0.96M | $0 | **$0.31M–$0.96M** |
| University build | $0.31M–$0.96M | $0.3M–$0.6M | **$0.61M–$1.56M** |
| Professional build | $0.31M–$0.96M | $0.9M–$1.3M | **$1.21M–$2.26M** |

Launch-related cost, as BASELINE §3 states it: the integrator's slot $85k–150k plus the program side $57k–195k
(tests, WBS 10, $42k–130k; export counsel $10k–40k; shipping, travel and the integration campaign $5k–15k; the
ephemeris and orbit-determination service, WBS 09, up to $10k), **$142k–345k all-in**, before the 15 % slot contingency
for rebooking. Revision E's BASELINE row ($30k–100k program side) predated the bottom-up test costs (RID P3P-08); the
row change is proposed in CR-2026-015.

## 6. Funding Profile (volunteer/university case, by fiscal year)

| Period | Main spend | Share |
|--------|-----------|-------|
| FY2027 (Oct 2026–Sep 2027: legal entity, PDR, EM build and test, integrator quotes, launch reservation deposit at the 2027-06-30 contract) | EM hardware and tests, launch deposit (integrators typically require a deposit at signing [U]), export counsel | ~40 % |
| FY2028 (Oct 2027–Sep 2028: CDR, PFU build, protoflight test by 2028-07-01, delivery in 2028-09) | PFU hardware, test labs, most of the launch balance | ~50 % |
| FY2029–FY2031 (launch about 2028-11, then operations) | Remaining launch balance, ground station upkeep | ~10 % |

## 7. Largest Cost Risks

1. **Launch price and availability** [U]: no published per-U price, plus reported post-2028 booking limits.
2. **Jetson thermal/power closure in 1U**: may force a redesign (larger form factor or a different AI compute), which would change every line above. See the findings register (technical budgets).
3. **ISS fallback**: adds the JSC 20793 battery abuse-test set ($50k–75k, 8–12 weeks, instead of $10k–15k) and safety-review labor.
6. **PCB layout**: a contract layout of the six custom boards is up to $105k cash; with one designer instead of two the
   schedule loses its margin (`mission/SCHEDULE.md` §3).
4. **Export/TAA** with a foreign integrator.
5. **A paid contract (D2)** would add a second radio (≈ $6k UHF, ≈ $13–15k S-band), a Part 5 filing and ITU cost
   recovery. The contract would fund them; they are not in the totals above.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial ROM (funding audit) |
| C | 2026-10-27 | Design review (RID PM-06): integration and test line from the bottom-up test plan ($35k–101k), a legal-entity and insurance line, stale "RTOS port" labour text corrected, funding profile moved to the 2028-11 planning launch, reserve to cover storage or rebooking |
| D | 2026-10-27 | WBS 07 for two telecommand stations with the 70 cm amplifier chain of decision C-14 ($5k–15k → $9k–19k); totals $300k–784k non-labor (§5 unchanged at $0.30M–0.78M) |
| E | 2026-10-27 | WBS 07 basis: the two telecommand-station variants of the restated C-14 (A ≈ $6.65k, B ≈ $7.35k); range and totals unchanged |
| F | 2026-10-27 | CDR round 3 (P3P-08, P3P-09): WBS 06 the bought items only, the custom boards (parts, Class 3 fabrication, assembly; EM and FM sets) and the contract PCB layout as their own lines, EM bought items re-based; WBS 05 without the carrier; WBS 03 the custom-battery abuse tests (Falcon 9 $10k–15k; ISS $50k–75k); WBS 02 the independent review boards; WBS 10 from the round-3 `test_cost.py` (EM shock, FM system EMC, reliability programme): $42k–130k; totals $309k–956k non-labor; §5 launch-related cost $142k–345k all-in; registered under document control (SQSO-DOC-001) |
| B | 2026-10-15 | Decisions D6 and D7: GNSS receiver line (EM and FM), launch slot $85k–150k with rebooking reserve, shipping and campaign line, orbit-determination tooling; paid-contract (D2) costs noted as outside the totals |
