# Lifecycle Cost — Rough Order of Magnitude (ROM)
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-AUD-005
**Revision:** C
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
| 03 | Safety & mission assurance (battery screening, inspections) | $3k | $10k | Cell screening lot, inspection tools; JSC 20793 testing **adds $15k–$40k [U] if the ISS backup is used** |
| 05 | AI payload: Jetson Orin Nano module, custom carrier, thermal strap, power switch | $2k | $10k | Module list price is in the hundreds of dollars [U]; carrier PCB design/fab dominates |
| 06 | Spacecraft bus, PFU: structure, EPS + battery, solar (body + wings), OBC, UHF/VHF radio + transponder, antenna deploy, ADCS (MTQ + magnetometer + sun sensors), harness | $45k | $119k | Analogous COTS pricing [U]: structure $3–6k; EPS + battery $8–20k; solar $10–30k; OBC $4–12k; radio/transponder $5–15k; antenna system $3–8k; ADCS $5–15k; harness $1–3k |
| 06 | GNSS receiver and patch antenna, EM and FM sets (D6) | $9k | $19k | piNAV-NG/FM EUR 3.3–6.2k or Celeste EUR 5.2k, plus ZENITH patch EUR 0.8k (list prices, not quotes) [U]; Czech export licence handled by the vendor |
| 06 | Flatsat / engineering-model hardware | $25k | $80k | ~50–70 % of PFU hardware |
| 07 | Mission operations: control-station equipment, SatNOGS integration, 2 yr operations consumables | $5k | $15k | Yagi/rotator/SDR/LNA station; ops labor uncompensated (Part 97) |
| 08 | Launch services (1U via integrator) | $85k | $150k | **[U]** No provider publishes a 1U price (seven checked, D7). SpaceX base $7,500/kg (2027), 50 kg minimum [P]; anchors: $86k (2026 community post), ~$85–90k (legacy Nanoracks) [U]. Hold 15 % of the slot price in the reserve for rebooking or storage |
| 08 | Export counsel, classification, possible TAA | $10k | $40k | Needed if the integrator is foreign ([export memo](../regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md)) [U] |
| 08 | Shipping, travel, customs, integration campaign | $5k | $15k | D7 research [U] |
| 09 | Ground data system (beacon decoder, dashboards; mostly existing software) and orbit determination with CCSDS OEM upload for SpaceX UG §9.5.7.2 (D6) | $0 | $10k | Existing `software/ground` tools; open-source astrodynamics library; effort is in labor (§4) |
| 10 | Integration & test, EM and protoflight: random vibration (protoflight 8.18 Grms envelope), sine, TVAC (8 cycles) and bake-out, EMC (unit and system), shock (if the integrator requires it), Helmholtz-cage magnetic test, mass properties (trifilar MOI), deployment hot and cold, test-pod rental, one re-test allowance | $35k | $101k | Bottom-up in `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §7 (`software/simulation/cdr/pm/test_cost.py`); lab day rates $2k–5k assumed [U] |
| 11 | Education / public outreach (beacon decoder releases, documentation) | $0 | $10k | — |
| 01 | Legal entity (formation, registered agent), launch-services legal review, insurance declarations and FAA cross-waivers (SQSO-LVR-001 LVR-009, LVR-011) | $3k | $20k | [U]; depends on the entity form and the integrator's terms |
| — | **Subtotal, non-labor** | **$227k** | **$599k** | |
| — | Reserve on non-labor (30 %) | $68k | $180k | Class D practice; reserve is held by the PI, not spread across lines. It must also cover about 5 months of storage or a rebooking if the launch slips from the 2028-11 planning date |
| — | **Total non-labor incl. reserve** | **$295k** | **$779k** | |

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
| Volunteer build | $0.30M–$0.78M | $0 | **$0.30M–$0.78M** |
| University build | $0.30M–$0.78M | $0.3M–$0.6M | **$0.60M–$1.38M** |
| Professional build | $0.30M–$0.78M | $0.9M–$1.3M | **$1.20M–$2.08M** |

## 6. Funding Profile (volunteer/university case, by fiscal year)

| Period | Main spend | Share |
|--------|-----------|-------|
| FY2027 (Oct 2026–Sep 2027: legal entity, PDR, EM build and test, integrator quotes, launch reservation deposit at the 2027-06-30 contract) | EM hardware and tests, launch deposit (integrators typically require a deposit at signing [U]), export counsel | ~40 % |
| FY2028 (Oct 2027–Sep 2028: CDR, PFU build, protoflight test by 2028-07-01, delivery in 2028-09) | PFU hardware, test labs, most of the launch balance | ~50 % |
| FY2029–FY2031 (launch about 2028-11, then operations) | Remaining launch balance, ground station upkeep | ~10 % |

## 7. Largest Cost Risks

1. **Launch price and availability** [U]: no published per-U price, plus reported post-2028 booking limits.
2. **Jetson thermal/power closure in 1U**: may force a redesign (larger form factor or a different AI compute), which would change every line above. See the findings register (technical budgets).
3. **ISS fallback**: adds JSC 20793 battery testing and safety-review labor.
4. **Export/TAA** with a foreign integrator.
5. **A paid contract (D2)** would add a second radio (≈ $6k UHF, ≈ $13–15k S-band), a Part 5 filing and ITU cost
   recovery. The contract would fund them; they are not in the totals above.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial ROM (funding audit) |
| C | 2026-10-27 | Design review (RID PM-06): integration and test line from the bottom-up test plan ($35k–101k), a legal-entity and insurance line, stale "RTOS port" labour text corrected, funding profile moved to the 2028-11 planning launch, reserve to cover storage or rebooking |
| B | 2026-10-15 | Decisions D6 and D7: GNSS receiver line (EM and FM), launch slot $85k–150k with rebooking reserve, shipping and campaign line, orbit-determination tooling; paid-contract (D2) costs noted as outside the totals |
