# Regulatory Summary (Public)

**Updated:** 2026-10-14 (funding audit). This page summarizes; the controlling documents are in `docs/regulatory/`.

| Topic | Position | Document |
|-------|----------|----------|
| Service / licensing | FCC Part 97 amateur-satellite service, subject to the conditions in the pathway determination (no compensated operations, no data sales, open formats) | [LICENSING_PATHWAY_DETERMINATION.md](regulatory/LICENSING_PATHWAY_DETERMINATION.md) |
| Filings and deadlines | IARU request; FCC pre-space notification (within 30 days of launch-vehicle determination, ≥ 90 days before integration); in-space (≤ 7 days after first transmission); post-space (≤ 3 months) | [REGULATORY_ROADMAP.md](regulatory/REGULATORY_ROADMAP.md) |
| Frequencies | Downlink 145.800–146.000 MHz; uplink 435–438 MHz; both to be coordinated by the IARU | [RF_PLAN.md](RF_PLAN.md) |
| Transmissions in the clear | All downlink content is plain text or published formats (97.113(a)(4), RR 25.2A). Only telecommand may be authenticated (97.211(b)) | [IARU request §6–7](regulatory/filings/IARU_COORDINATION_REQUEST_v40.md) |
| Cessation of emissions | Telecommand TX-OFF required (97.207(b), RR 22.1). **Not yet implemented in flight software** | Findings register |
| Orbital debris | Re-entry no later than 5 years after mission completion; casualty risk < 1:10,000 (97.207(g)(1)(vii)) | [ODAR](regulatory/filings/ODAR_SMART-QSO.md) |
| Remote sensing | No NOAA license required (no imaging instrument) | [NOAA determination](regulatory/filings/NOAA_REMOTE_SENSING_DETERMINATION.md) |
| Export control | Spacecraft ECCN 9A515.a.5 (presumptive); launch-integration services are ITAR | [Export memo](regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md) |

Earlier versions of this page described a "transmit-only beacon" with "no uplink planned". That design cannot meet the
cessation-of-emissions rule and contradicted the transponder mission, so it has been superseded.
