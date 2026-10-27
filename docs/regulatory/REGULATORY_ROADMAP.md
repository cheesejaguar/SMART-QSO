# Regulatory and Launch-Authorization Roadmap
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-004
**Revision:** C
**Date:** 2026-10-27
**Status:** Plan. **No regulatory filing has been made as of this date.**
**Governing determination:** [LICENSING_PATHWAY_DETERMINATION.md](LICENSING_PATHWAY_DETERMINATION.md) (Part 97)
**Schedule source:** `mission/SCHEDULE.md` (PDR 2027-01-12, launch contract by 2027-06-30, dispenser integration about
2028-09-15, planning launch 2028-11-01, BASELINE Rev E §3). Where that schedule moves, the dates below move with it by the rules in §2.

---

## 1. Filing Inventory

| ID | Filing / approval | Authority | Rule / source | Draft in repo | Status |
|----|-------------------|-----------|---------------|---------------|--------|
| F1 | IARU amateur-satellite frequency coordination request (form v40) | IARU Satellite Advisor (satcoord@iaru.org) | IARU form v40 (in force since 2020-05-15; iaru.org page updated 2026-07-26); ITU RR 1.56/1.57/25 | [IARU_COORDINATION_REQUEST_v40.md](filings/IARU_COORDINATION_REQUEST_v40.md) | Not submitted |
| F2 | FCC Part 97 **pre-space** notification, with ITU Appendix 4 / Res. 642 data and orbital-debris disclosures | FCC Space Bureau | 47 CFR 97.207(g)(1) | [FCC_97-207_PRE_SPACE_NOTIFICATION.md](filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md) | Not submitted |
| F3 | ITU Advance Publication Information (API), then notification | ITU-BR, filed **by the FCC** on the licensee's data | RR Articles 9 and 11; Appendix 4; Resolution 32 (WRC-19) if filed as a short-duration mission | [ITU_APPENDIX4_DATA.md](filings/ITU_APPENDIX4_DATA.md) | Not submitted |
| F4 | Orbital Debris Assessment Report (NASA-STD-8719.14C format), basis for the F2 debris disclosures | FCC (via F2) | 97.207(g)(1)(i)–(vii) | [ODAR_SMART-QSO.md](filings/ODAR_SMART-QSO.md) | Draft. DAS run pending |
| F5 | Replacement pre-space notification if any material item changes | FCC Space Bureau | 97.207(g)(1)(viii): no later than 90 days before integration | Use F2 | — |
| F6 | FCC **in-space** notification | FCC Space Bureau | 97.207(g)(2): no later than 7 days after first transmission | [FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md](filings/FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md) | Template |
| F7 | FCC **post-space** notification | FCC Space Bureau | 97.207(g)(3): 3 months after termination; 24 hours if FCC-ordered | same | Template |
| F8 | NOAA remote-sensing license applicability (no license expected) | NOAA CRSRA | 15 CFR 960.4, 960.5(a) | [NOAA_REMOTE_SENSING_DETERMINATION.md](filings/NOAA_REMOTE_SENSING_DETERMINATION.md) | Determination drafted; confirmation not requested |
| F9 | Export classification (9A515.a.5; ITAR for integration services) and any licence or TAA | BIS / DDTC | 15 CFR 774 Supp. 1; ITAR | [EXPORT_CLASSIFICATION_MEMO.md](filings/EXPORT_CLASSIFICATION_MEMO.md) | Self-classification drafted; counsel not engaged |
| F10 | Space-Track / TraCSS registration; conjunction-response certification | USSF 18 SDS / Commerce OSC | 97.207(g)(1)(iv)–(v) | [SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md](filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md) | Not started |
| F11 | Launch-provider regulatory and safety deliverables | Integrator; NASA LSP if CSLI | CDS Rev 14.1; provider ICD | [LAUNCH_PROVIDER_DELIVERABLES.md](filings/LAUNCH_PROVIDER_DELIVERABLES.md) | Checklist drafted |
| F12 | FAA payload review information (through the launch operator) | FAA/AST | 14 CFR 450.43 | in F11 | — |
| F13 | **Designation of telecommand stations** (licensee's written designation of each station, 97.211(a)); listed in F1 item 7, F2 §1 and as Res. 642 stations in F3 | Licensee | 97.211(a); RR 25.11 | `docs/ground/GROUND_STATION_DESIGN.md` §3 | **No station agreed** (BASELINE §1) |

## 2. Rules and Lead Times (checked 2026-10-27)

| Item | Rule or measured lead time | Source |
|------|----------------------------|--------|
| FCC pre-space | "within 30 days after the date of launch vehicle determination, but no later than 90 days before integration of the space station into the launch vehicle". The FCC has accepted notifications before launch-vehicle determination when the orbit is definite (DA 13-445 fn. 6). **Plan: file early (F2) with the IARU request, then a replacement within 30 days of the launch contract** | 47 CFR 97.207(g)(1), eCFR version of 2026-10-01 |
| Rule changes after 2024 | **None to 97.207 or 97.211.** The last amendment of 97.207 is the five-year post-mission disposal rule (Orbital Debris Second Report and Order; 89 FR 65217, 2024-08-09), effective **2024-09-09** for 97.207(g)(1)(vii)(D)(1). Part 97 changes since (89 FR 100838, 2024-12-13; Delete, Delete, Delete, 90 FR 57698, 2025-12-12, effective 2026-02-10; WRC-15 implementation, 91 FR 1405, 2026-01-14, effective 2026-02-13) touch 97.27, 97.29, 97.301–97.315 (60 m, 5.9 GHz, VEC and licensing housekeeping), not space stations. An FCC orbital-debris record refresh (DA 24-412, 89 FR 46052, 2024-05-28) is still open: check before filing | federalregister.gov and eCFR version history, retrieved 2026-10-27 |
| 70 cm transmit power near military radars | 47 CFR 2.106 US270: 50 W PEP within 240 km of Beale AFB (39°08′ N, 121°26′ W) and other listed sites; 97.313(f): earth and telecommand stations may use up to 611 W ERP in 435–438 MHz if the antenna's lower half-power point stays above 10° elevation | eCFR 2026-10-01 |
| IARU coordination | Panel meets every two to three weeks (iaru.org). **Measured:** the 39 coordinations completed between 2025-08 and 2026-10 on the IARU status pages took a **median of 57 days**, 90th percentile about **112 days**, with outliers of 264–398 days (resubmissions, disputes) (`iaru.amsat-uk.org`, retrieved 2026-10-27). The IARU instructions recommend submitting "right after you have done your preliminary Design Review" and ask for the API/A number, or the date API data went to the administration | IARU instructions (rev 38); IARU status pages |
| ITU API | RR 9.1: not earlier than 7 years and **preferably not later than 2 years** before bringing into use (BIU late 2028 means late 2026: already late, which is allowed). API special section published about 2–3 months after the Bureau receives it (IARU instructions; ITU now targets about 2 months). Under Resolution 32 (WRC-19, short-duration missions, ≤ 3 years' validity, ≤ 10 satellites), the notification may go with the API and is treated as received 4 months after the API is published; the 3-year validity cannot be extended | ITU-R small-satellite pages; RR |
| NOAA | Informal consultation under 15 CFR 960.5(a); no statutory deadline. Allow 1–3 months | 15 CFR 960 |
| Export | Counsel opinion or CCATS 1–3 months; a TAA for a foreign integrator 3–6 months | Export memo §3 |
| Five-year disposal | Re-entry no later than 5 years after completion of the mission | 97.207(g)(1)(vii)(D)(1) |

## 3. Dated Plan, 2026-10 to Launch

Dates assume the `mission/SCHEDULE.md` baseline. "I" is dispenser integration (about 2028-09-15), "L" launch
(2028-11-01, the planning launch of BASELINE Rev E §3; it was 2028-06-01).

| Date (no later than) | Action | Filing | Owner | Depends on |
|----------------------|--------|--------|-------|------------|
| 2026-11-15 | Request the FCC Space Bureau consultation (D3): AI experiment, N = 30/14 days and M, the Part 5 questions of pathway §7.1, the 97.313(f)/US270 reading for the Bay Area telecommand station, early filing of F2 | — | Licensee | — |
| 2026-11-30 | Ask AMSAT-NA and ARRL for endorsement (form 4.8–4.9) | F1 prep | Licensee | — |
| 2026-11-30 | Send the NOAA CRSRA informal consultation (§2 of the determination) | F8 | PI | — |
| 2026-12-15 | Engage export counsel; include export terms in the launch RFQs (Exolaunch, ISISPACE and D-Orbit are foreign) | F9 | PI | D7 RFQs by 2026-12-31 |
| **2027-01-05** | **Telecommand stations agreed in writing (≥ 2, §3 of SQSO-GND-001), operators named, key-custody plan signed** | F13 | Licensee | Partners (D1) |
| 2027-01-05 | FCC consultation held; outcome recorded in the pathway determination | — | Licensee | 2026-11-15 request |
| 2027-01-12 | Independent PDR (IARU's recommended point for the request) | — | PI | |
| 2027-01-19 | **File F2 early** with the Appendix 4 data (SpaceCap) and the draft ODAR, asking the FCC to forward the API | F2, F3 | Licensee | Stations, consultation |
| 2027-01-26 | **Submit F1** with "API data sent to the FCC on 2027-01-19" | F1 | Licensee | F2 |
| 2027-02-09 | NASA DAS run; ODAR final | F4 | Mission design | Mass properties |
| 2027-03-25 (median) to 2027-05-18 (90th percentile) | **IARU coordination letter**; frequencies frozen in BASELINE §7 | F1 | Licensee | Panel |
| 2027-03 to 2027-05 | ITU API/A special section published (FCC forwarding time unknown: ask in the consultation) | F3 | FCC | F2 |
| 2027-03-31 | NOAA reply filed | F8 | PI | |
| 2027-05-31 (contract) | Launch-vehicle determination (signed contract, D7 deadline 2027-06-30) | — | PI | Funding |
| **2027-06-30** (contract + 30 days) | **Replacement F2** with the launch vehicle, integrator, orbit, IARU frequencies, final ODAR | F2/F5 | Licensee | Contract, IARU letter |
| 2027-09-30 | ISS/NRCSD backup decision (D7); if taken, replacement F2 and an IARU revision for the 51.6° orbit | F5, F1 | PI | |
| 2027-10-18 | Export counsel opinion; TAA in place if the integrator is foreign | F9 | PI | |
| 2027-12-15 | Space-Track registration and conjunction-response certification | F10 | Ops lead | Owned by SPACE_SAFETY plan |
| **2028-06-17** (I − 90 days) | **Last date for any replacement pre-space notification** | F5 | Licensee | |
| 2028-08-14 to 2028-09-02 | Provider deliverables with proof of F1, F2, F4, F9 | F11, F12 | PI | |
| L = 2028-11-01; first transmission L + 45 min | Launch; RF silence 45 min | — | | |
| **2028-11-08** (first transmission + 7 days) | **In-space notification** | F6 | Licensee | |
| 2030-11-01 (end of the 2-year design life) | TX-OFF and passivation by telecommand | — | Ops | Passivation command (SQSO-ICD-013 §6.4, proposed) |
| **2031-02-01** (termination + 3 months; 24 h if FCC-ordered) | **Post-space notification** | F7 | Licensee | |
| ≤ 2035-11-01 | Re-entry (5 years after mission end; 2.9–3.8 years after it predicted at 475 km with the wings deployed, nominal and conservative, 1.6–1.8 kg, BASELINE §3) | — | | ODAR |

**Critical-path observations.**

1. **The telecommand stations are the gate.** F1, F2 and F3 all name them, and none is agreed (BASELINE §1). If they
   are not agreed by 2027-01-05, every filing slips with them.
2. **The 30-day rule and the IARU letter no longer conflict** with the measured IARU lead time: a letter is likely by
   May 2027, before the contract. Filing F2 early and replacing it within 30 days of the contract covers either order.
3. **The 90-day backstop (2028-06-17)** is about 12 weeks before delivery (2028-09-12). A frequency or orbit change after it cannot be
   filed in time: freeze both at the replacement F2.
4. **Short-duration-mission filing (Res. 32)** fits a 2-year mission but cannot be extended past 3 years. Ask the FCC to
   file under the standard API/notification path if an extended mission is wanted (`ITU_APPENDIX4_DATA.md` §F).

## 4. Correction of Prior Status Claims

As of 2026-10-27, **no** IARU request, FCC notification, ITU filing, NTIA request, NASA Form 1722 or DoD/NOAA
coordination exists in the repository or is evidenced anywhere. Earlier text claiming "Application submitted",
"Coordination requested", "License pending", "Submitted" or "Complete" for these items has been corrected (see
`docs/regulatory/SPECTRUM_LICENSE.md` and `docs/regulatory/IARU_COORDINATION.md`).

## 5. Owners

All filings are signed by the **space station licensee**, KK6GDB (decision D1). The PI owns F8, F9 and F11. The
operations lead owns F6, F7 and F10. The licensee owns F13.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial roadmap (funding audit) |
| B | 2026-10-27 | CDR review (communications panel): rules re-checked against the eCFR of 2026-10-01 and the Federal Register (no change to 97.207/97.211 since the 2024-09-09 five-year rule); 47 CFR 2.106 US270 and 97.313(f) added; measured IARU lead time (median 57 days); ITU API timing and Res. 32; F13 (telecommand-station designation); dated plan from 2026-10 to the post-space notification |
| C | 2026-10-27 | CDR 2026 consistency pass: §3 launch-dependent dates for the planning launch 2028-11-01 and integration 2028-09-15 (BASELINE Rev E §3, `mission/SCHEDULE.md` Rev B): I − 90 days 2028-06-17, provider deliverables, in-space 2028-11-08, end of life 2030-11-01, post-space 2031-02-01, re-entry at 475 km |
