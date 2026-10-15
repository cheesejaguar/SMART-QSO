# Lifecycle Cost — Rough Order of Magnitude (ROM)
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-AUD-005
**Revision:** A
**Date:** 2026-10-14
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
| 06 | Spacecraft bus, PFU: structure, EPS + battery, solar (body + wings), OBC, UHF/VHF radio + transponder, antenna deploy, ADCS (MTQ + magnetometer + sun sensors), harness | $47k | $127k | Analogous COTS pricing [U]: structure $3–6k; EPS + battery $8–20k; solar $10–30k; OBC $4–12k; radio/transponder $5–15k; antenna system $3–8k; ADCS $5–15k; harness $1–3k; GNSS (if adopted) $2–8k |
| 06 | Flatsat / engineering-model hardware | $25k | $80k | ~50–70 % of PFU hardware |
| 07 | Mission operations: control-station equipment, SatNOGS integration, 2 yr operations consumables | $5k | $15k | Yagi/rotator/SDR/LNA station; ops labor uncompensated (Part 97) |
| 08 | Launch services (1U via integrator) | $90k | $150k | **[U]** No published per-U price. SpaceX base $7,500/kg (2027), 50 kg minimum [P]; historical ~$85k/unit anchor [U] |
| 08 | Export counsel, classification, possible TAA | $10k | $40k | Needed if the integrator is foreign ([export memo](../regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md)) [U] |
| 09 | Ground data system (beacon decoder, dashboards; mostly existing software) | $0 | $5k | Existing `software/ground` tools |
| 10 | Integration & test: random vibe (≥ 5.57 Grms MPE), TVAC/bakeout, EMC pre-scan, fit check, test-pod rental | $23k | $78k | Commercial test-lab day rates [U] |
| 11 | Education / public outreach (beacon decoder releases, documentation) | $0 | $10k | — |
| — | **Subtotal, non-labor** | **$205k** | **$525k** | |
| — | Reserve on non-labor (30 %) | $62k | $158k | Class D practice; reserve is held by the PI, not spread across lines |
| — | **Total non-labor incl. reserve** | **$267k** | **$683k** | |

## 4. Labor (the dominant uncertainty)

The program today has a design-phase document set, simulation-only flight software (no flight-target HAL or RTOS port),
and no hardware (see findings register). Estimated effort to reach delivery and launch:

| Role | FTE | Duration | FTE-years |
|------|-----|----------|-----------|
| PI / project manager / systems engineer | 0.75 | 24 mo | 1.5 |
| Electrical / RF (EPS integration, transponder, OBC-Jetson interface) | 1.0 | 18 mo | 1.5 |
| Flight software (HAL, RTOS port, telecommand authentication, FDIR on target) | 1.0 | 24 mo | 2.0 |
| Mechanical / thermal (Jetson thermal path, structure, deployables) | 0.5 | 18 mo | 0.75 |
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
| Volunteer build | $0.27M–$0.68M | $0 | **$0.27M–$0.68M** |
| University build | $0.27M–$0.68M | $0.3M–$0.6M | **$0.57M–$1.28M** |
| Professional build | $0.27M–$0.68M | $0.9M–$1.3M | **$1.17M–$1.98M** |

## 6. Funding Profile (volunteer/university case, by fiscal year)

| Period | Main spend | Share |
|--------|-----------|-------|
| FY2027 (re-baseline, flatsat/EM, integrator quotes, launch reservation deposit) | EM hardware, launch deposit (integrators typically require a deposit at signing [U]), export counsel | ~45 % |
| FY2028 (PFU build, environmental test, delivery, launch) | PFU hardware, test labs, launch balance | ~50 % |
| FY2029–FY2030 (operations) | Ground station upkeep | ~5 % |

## 7. Largest Cost Risks

1. **Launch price and availability** [U]: no published per-U price, plus reported post-2028 booking limits.
2. **Jetson thermal/power closure in 1U**: may force a redesign (larger form factor or a different AI compute), which would change every line above. See the findings register (technical budgets).
3. **ISS fallback**: adds JSC 20793 battery testing and safety-review labor.
4. **Export/TAA** with a foreign integrator.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial ROM (funding audit) |
