# IARU Satellite Frequency Coordination — Status
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-001
**Revision:** G
**Date:** 2026-10-27

---

## 1. Current Status

| Item | Status |
|------|--------|
| Coordination request | **Not submitted.** Draft on the mandatory IARU v40 form layout: [IARU_COORDINATION_REQUEST_v40.md](filings/IARU_COORDINATION_REQUEST_v40.md) |
| Submitted by | The prospective space station licensee, KK6GDB (decision D1 in [LICENSING_PATHWAY_DETERMINATION.md](LICENSING_PATHWAY_DETERMINATION.md)). The form also needs at least two designated telecommand stations, at least 1,000 km apart, **none agreed yet**; written agreements due 2027-01-05 (`mission/SCHEDULE.md` DA-10; BASELINE Rev E §1) |
| Submit to | IARU Satellite Advisor, satcoord@iaru.org (one global process; not "IARU Region 2" or AMSAT-NA) |
| API/A number | None yet. The FCC files the API from the pre-space notification data ([ITU_APPENDIX4_DATA.md](filings/ITU_APPENDIX4_DATA.md)) |
| Recommended submission | **2027-01-26**, after the PDR (2027-01-12) and a week after the early FCC pre-space filing that carries the API data ([REGULATORY_ROADMAP.md](REGULATORY_ROADMAP.md) Rev D §3; `mission/SCHEDULE.md` DA-09) |
| Expected duration | Median 57 days, 90th percentile about 112 days (39 coordinations completed 2025-08 to 2026-10, IARU status pages); letter expected 2027-03-25 to 2027-05-18; with no letter by 2027-05-18 the licensee asks the IARU Satellite Advisor for status (DA-09) |
| Frequencies to request | Downlink one frequency in 145.800–146.000 MHz (not 145.825); uplink one frequency in **435.222–435.778 MHz, or else 436.222–436.500 MHz** (BASELINE §7: the receive SAW and the receiver clock limit the band to 435.050–436.500 MHz, SQSO-RF-BRD-001; the EPS converters' 1.000 MHz harmonics leave these windows clear, SQSO-EPS-BRD-001, SQSO-EMC-001), at least 0.9 MHz below 3 × the downlink (BASELINE Rev E §7; SQSO-RF-BRD-001 §7.5; CR-2026-008) |

## 2. Corrections to Rev A

Rev A (2026-01-02) was written as an application. It contained material errors that would have led to rejection or a
misrepresentation to the IARU, and it has been withdrawn:

| Rev A content | Correction |
|---------------|------------|
| "Operator: NASA Ames Research Center"; technical contact "@nasa.gov"; primary ground station "NASA Ames Research Center"; "Non-commercial: NASA research mission" | No NASA affiliation exists. The operator is the amateur licensee. A NASA research mission would also not meet RR 1.56 (personal aim, no pecuniary interest). |
| "SMART-QSO operates as a beacon-only mission with no command uplink on amateur frequencies. All commanding is performed via separate non-amateur frequencies." | The mission is an amateur **transponder** with a 435–438 MHz uplink, and telecommand is in the amateur band (97.211). A spacecraft with no telecommand cannot meet RR 22.1 or 47 CFR 97.207(b), and IARU form items 7.8/7.9 require a cessation-of-emissions method. |
| Primary downlink 145.825 MHz | 145.825 MHz carries ARISS/ISS packet and APRS digipeater traffic. The v40 draft asks the panel to assign a frequency in 145.800–146.000 MHz. |
| FCC Part 97 "Application pending"; NTIA "Coordination in progress" | Nothing was filed. Part 97 uses notifications, not a license application, and NTIA does not apply. |
| "First Beacon: Launch + 30 minutes" | Violates the CDS Rev 14.1 §2.4.5 rule of no transmission earlier than **45 minutes** after deployment. |
| Link margin 14 dB with −145 dB path loss | Superseded by `docs/environment/LINK_BUDGET_ANALYSIS.md` v3.5 (analysis at the 475 km baseline target; BASELINE Rev E §7): the 1200 bps beacon at 0.5 W closes from 5° elevation with +7.9 dB at the median spacecraft gain and +0.9 dB at the 10th-percentile gain; telecommand at ≥ 64 dBm EIRP (decision C-14) has ≥ +8.0 dB at 10° and ≥ +6.2 dB at 5° with the 10th-percentile gain and a 6 dB interference allowance. Free-space loss at 145.9 MHz is 141.8 dB at 5° and 129.3 dB overhead. |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release (withdrawn) | SMART-QSO Team |
| B | 2026-10-14 | Replaced by v40-form draft; Rev A errors listed | Funding audit |
| C | 2026-10-16 | Link-margin correction cites the re-issued link budget (SQSO-LBA-001 v2.0) | Funding audit (TB-05) |
| D | 2026-10-27 | Header revision brought up to the history (it said B after Rev C). Status: licensee designated, stations not; submission date, expected duration and frequency ranges; link margins to SQSO-LBA-001 v3.0 | CDR review, communications panel |
| E | 2026-10-27 | CDR 2026 consistency pass: uplink request 435.222–435.778 MHz, else 436.222–436.500 MHz (BASELINE Rev E §7); link-budget altitude note | CDR documentation review |
| F | 2026-10-27 | Link-margin row from SQSO-LBA-001 v3.2 at 475 km (RID SE-16) |
| G | 2026-10-27 | Documentation audit (post-CDR 2026): uplink at least 0.9 MHz below 3 × f_down (BASELINE Rev E §7; was 2 MHz); link-margin row from LBA v3.5 with the 64 dBm EIRP telecommand (C-14; was 25 W, +2.6 dB); REGULATORY_ROADMAP Rev D; stations DA-10 and the IARU letter DA-09 | Documentation audit |
