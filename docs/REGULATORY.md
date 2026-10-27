# Regulatory Summary (Public)

**Updated:** 2026-10-27 (CDR review). This page summarizes; the controlling documents are in `docs/regulatory/`.
**No filing has been made as of this date.**

| Topic | Position | Document |
|-------|----------|----------|
| Service / licensing | FCC Part 97 amateur-satellite service under licensee KK6GDB (station KK6GDB-11), subject to the conditions in the pathway determination (no compensated operations, no data sales, open formats) | [LICENSING_PATHWAY_DETERMINATION.md](regulatory/LICENSING_PATHWAY_DETERMINATION.md) |
| Filings and deadlines | IARU request (recent coordinations took a median of 57 days, 90th percentile about 4 months); FCC pre-space notification within 30 days after launch-vehicle determination and no later than 90 days before integration (47 CFR 97.207(g)(1); unchanged since the five-year disposal rule took effect on 2024-09-09); in-space notification within 7 days after first transmission; post-space within 3 months (24 hours if FCC-ordered). Dated plan to the 2028 launch | [REGULATORY_ROADMAP.md](regulatory/REGULATORY_ROADMAP.md) |
| Frequencies | Downlink 145.800–146.000 MHz; uplink requested in 435.222–435.778 MHz, or else 436.222–436.500 MHz (clear of the downlink's 3rd harmonic); both to be coordinated by the IARU | [RF_PLAN.md](RF_PLAN.md) |
| Transmissions in the clear | All downlink content is plain text or published formats (97.113(a)(4), RR 25.2A). Only telecommand is authenticated, not encrypted (97.211(b)) | [IARU request §6–7](regulatory/filings/IARU_COORDINATION_REQUEST_v40.md) |
| Cessation of emissions | Telecommand TX-OFF (97.207(b), RR 22.1): implemented and tested in the flight software; the hardware path is designed and not yet built | [TELECOMMAND_AUTHENTICATION.md](interfaces/TELECOMMAND_AUTHENTICATION.md) §6 |
| Telecommand stations | At least two designated, geographically separated stations (decision D1): **none agreed yet**. Each radiates ≥ 64 dBm EIRP (2.5 kW) toward the spacecraft and commands only above 10° (decision C-14): outside the 47 CFR 2.106 US270 areas by ≥ 100 W PEP into ≥ 14 dBic; inside them (the Bay Area site near Beale AFB, Fairbanks near Clear SFS) by ≤ 50 W PEP into ≥ 17.5 dBic, a stacked pair of cross-Yagis | [GROUND_STATION_DESIGN.md](ground/GROUND_STATION_DESIGN.md) |
| Orbital debris | Re-entry no later than 5 years after mission completion; casualty risk < 1:10,000 (97.207(g)(1)(vii)) | [ODAR](regulatory/filings/ODAR_SMART-QSO.md) |
| Remote sensing | No NOAA license required (no imaging instrument) | [NOAA determination](regulatory/filings/NOAA_REMOTE_SENSING_DETERMINATION.md) |
| Export control | Spacecraft ECCN 9A515.a.5 (presumptive); telecommand authentication only, no encryption for confidentiality; launch-integration services are ITAR | [Export memo](regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md) |

Earlier versions of this page described a "transmit-only beacon" with "no uplink planned". That design cannot meet the
cessation-of-emissions rule and contradicted the transponder mission, so it has been superseded.

| Date | Change |
|------|--------|
| 2026-10-14 | Funding audit rewrite |
| 2026-10-27 | CDR review: TX-OFF status, filing lead times, telecommand stations and the Beale AFB power limit, uplink range, export note |
| 2026-10-27 | CDR 2026 consistency pass: uplink request windows (BASELINE Rev E §7) |
| 2026-10-27 | Telecommand stations: the EIRP requirement of decision C-14 and its two station variants (≤ 50 W PEP at US270 sites) |
