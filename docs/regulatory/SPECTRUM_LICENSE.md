# Spectrum Authorization Status
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-002
**Revision:** E
**Date:** 2026-10-27

---

## 1. Purpose

This document records SMART-QSO's spectrum authorization approach and the **actual** status of each authorization.
Rev A (2026-01-02) contained statements that were not true and has been withdrawn (see §6).

## 2. Authorization Regime

SMART-QSO is a non-Federal **amateur space station under FCC Part 97**. The determination and its conditions are in
[LICENSING_PATHWAY_DETERMINATION.md](LICENSING_PATHWAY_DETERMINATION.md). There is no Federal (NTIA) frequency use and no
Federal sponsor.

## 3. Frequency Plan (amateur bands only)

| Function | Band | Frequency | Emission | Authority |
|----------|------|-----------|----------|-----------|
| FM transponder downlink + AX.25 telemetry beacon | 144–146 MHz (IARU satellite sub-band 145.800–146.000 MHz) | To be coordinated by IARU. **Not 145.825 MHz** (ARISS/APRS packet) | 16K0F3E / 10K4F2D | 47 CFR 97.207(c)(2) |
| FM transponder uplink + authenticated telecommand | 435–438 MHz; requested in 435.222–435.778 MHz, or else 436.222–436.500 MHz (BASELINE §7; clear of the downlink's 3rd harmonic) | To be coordinated by IARU | 16K0F3E / 10K4F2D | 47 CFR 97.207(c)(2), 97.209(b)(2), 97.211(c)(2) |

Technical data: [ITU_APPENDIX4_DATA.md](filings/ITU_APPENDIX4_DATA.md). Baseline values: [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md).

## 4. Authorization Status (as of 2026-10-27)

| Authorization | Status | Draft / reference |
|---------------|--------|-------------------|
| Space station licensee and call sign | **KK6GDB**, station KK6GDB-11 (decision D1, 2026-10-15) | Pathway determination §8 |
| Designated telecommand stations (97.211(a)) | **None agreed**; ≥ 2, at least 1,000 km apart, needed before the IARU request (written agreements due 2027-01-05, `mission/SCHEDULE.md` DA-10) | `docs/ground/GROUND_STATION_DESIGN.md` §3 |
| IARU frequency coordination | **Not started**; planned submission 2027-01-26 | [IARU_COORDINATION_REQUEST_v40.md](filings/IARU_COORDINATION_REQUEST_v40.md) |
| FCC Part 97 pre-space notification (97.207(g)(1)) | **Not filed**; planned early filing 2027-01-19, replacement by 2027-06-30 | [FCC_97-207_PRE_SPACE_NOTIFICATION.md](filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md) |
| ITU API / notification (filed by the FCC) | **Not filed** | [ITU_APPENDIX4_DATA.md](filings/ITU_APPENDIX4_DATA.md) |
| FCC Space Bureau consultation (D3) | **Not yet requested**; request due 2026-11-15, meeting by 2027-01-05, outcome recorded by 2027-01-08 (`mission/SCHEDULE.md` DA-11) | [REGULATORY_ROADMAP.md](REGULATORY_ROADMAP.md) §6 |
| Orbital debris assessment (DAS) | **Draft; DAS not yet run**; NASA DAS usage agreement to be requested by 2026-11-06, DAS run by 2027-01-08 (DA-06) | [ODAR_SMART-QSO.md](filings/ODAR_SMART-QSO.md) |
| NOAA remote sensing | **No license required** (internal determination); NOAA confirmation not requested | [NOAA determination](filings/NOAA_REMOTE_SENSING_DETERMINATION.md) |
| Export classification | **Self-classified, counsel review pending** | [Export memo](filings/EXPORT_CLASSIFICATION_MEMO.md) |
| NTIA / NASA Form 1722 / DoD / NOAA spectrum coordination | **Not applicable** (no Federal frequencies) | — |

Timeline and deadlines: [REGULATORY_ROADMAP.md](REGULATORY_ROADMAP.md).

## 5. RF Exposure

At ≤ 1 W into a ~0 dBi antenna from ≥ 400 km, the power density at the ground is many orders of magnitude below the
FCC limits in 47 CFR 1.1310. Telecommand stations radiating ≥ 64 dBm EIRP (decision C-14: ≥ 100 W PEP into ≥ 15.5 dBic, or ≤ 50 W PEP into
≥ 17.8 dBic at 47 CFR 2.106 US270 sites; BASELINE Rev E §7) must make their own 47 CFR 1.1310
evaluation (controlled and uncontrolled exposure distances) as part of the station set-up (`docs/ground/GROUND_STATION_DESIGN.md` §3). Ground-test RF safety is covered by `docs/test_procedures/TP-RF-002_Conducted_Power.md`
(conducted into a dummy load).

## 6. Withdrawn Rev A Statements

| Rev A statement | Why withdrawn |
|-----------------|---------------|
| TT&C uplink "449.x MHz" and downlink "401.x MHz" (Government, NTIA) | A non-Federal Part 97 station cannot hold Federal assignments, and no Federal sponsor exists. Replaced by amateur telecommand (97.211). |
| "NASA Form 1722 – Submitted"; "Coordination with DoD – Complete"; "Coordination with NOAA – Complete"; "SPS analysis – No interference predicted" | No evidence that any of these occurred. NASA Form 1722 is a NASA-internal spectrum form. |
| "IARU Region 2 – Application submitted"; "AMSAT-NA – Coordination requested"; "FCC – License pending" | Nothing was submitted. IARU coordination is done by the IARU Satellite Advisor (satcoord@iaru.org), not by region. |
| "Non-pecuniary interest: NASA research mission" | The project is not a NASA mission. A NASA research mission would not qualify as an amateur operation. |
| "Callsign identification every 10 minutes" as a Part 97 requirement | Space stations are exempt from 97.119(a). SMART-QSO identifies anyway, as good practice. |
| Orbit "500 km, 51.6°" for the ITU filing | Superseded by the baseline orbit from the launch trade. |
| Export classification "EAR99" | Superseded by the export memo (ECCN 9A515.a.5). |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release (withdrawn) | SMART-QSO Team |
| B | 2026-10-14 | Rewritten after funding audit: amateur-only frequency plan, actual status, withdrawn claims listed | Funding audit |
| C | 2026-10-27 | Licensee designated (D1); telecommand stations status; uplink range; planned filing dates; ground-station RF exposure note | CDR review, communications panel |
| D | 2026-10-27 | CDR 2026 consistency pass: uplink request windows from BASELINE Rev E §7 | CDR documentation review |
| E | 2026-10-27 | Documentation audit (post-CDR 2026): §4 FCC consultation (DA-11) and the DAS agreement (DA-06) added, stations ≥ 1,000 km apart (DA-10); §5 telecommand station power and gain per C-14 (was 50 W into 14 dBic) | Documentation audit |
