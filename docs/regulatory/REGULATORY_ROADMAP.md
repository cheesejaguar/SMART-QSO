# Regulatory and Launch-Authorization Roadmap
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-004
**Revision:** D.1
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
| F4 | Orbital Debris Assessment Report (NASA-STD-8719.14C format), basis for the F2 debris disclosures | FCC (via F2) | 97.207(g)(1)(i)–(vii) | [ODAR_SMART-QSO.md](filings/ODAR_SMART-QSO.md) | Draft D. NASA DAS run before F2 (§7) |
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
| **2026-11-06** | **Request the NASA DAS software usage agreement** (NASA Software Catalog, DAS 3.2.6) | F4 | Licensee | — |
| 2026-11-15 | Request the FCC Space Bureau consultation (D3), sending the agenda of §6 (Q1–Q10) | — | Licensee | — |
| 2026-11-30 | Ask AMSAT-NA and ARRL for endorsement (form 4.8–4.9) | F1 prep | Licensee | — |
| 2026-11-30 | Send the NOAA CRSRA informal consultation (§2 of the determination) | F8 | PI | — |
| 2026-12-15 | Engage export counsel; include export terms in the launch RFQs (Exolaunch, ISISPACE and D-Orbit are foreign) | F9 | PI | D7 RFQs by 2026-12-31 |
| **2027-01-05** | **Telecommand stations agreed in writing (≥ 2, at least 1,000 km apart, each able to command alone; §3 of SQSO-GND-001; `mission/SCHEDULE.md` DA-10), operators named, key-custody plan signed** | F13 | Licensee | Partners (D1) |
| 2027-01-05 | FCC consultation held on the §6 agenda; outcome recorded in the pathway determination by 2027-01-08 | — | Licensee | 2026-11-15 request |
| **2027-01-08** | **NASA DAS run** with the controlled mass properties; pass criterion §7; ODAR Draft E with the DAS results by 2027-01-12 | F4 | S&MA (run); licensee (agreement) | DAS agreement |
| 2027-01-12 | Independent PDR (IARU's recommended point for the request) | — | PI | |
| 2027-01-19 | **File F2 early** with the Appendix 4 data (SpaceCap) and the ODAR with its DAS results (97.207(g)(1)(ii) and (iv)(A)(1) ask for them), asking the FCC to forward the API | F2, F3 | Licensee | Stations, consultation, DAS |
| 2027-01-26 | **Submit F1** with "API data sent to the FCC on 2027-01-19" | F1 | Licensee | F2 |
| 2027-03-25 (median) to 2027-05-18 (90th percentile) | **IARU coordination letter**; frequencies frozen in BASELINE §7 by CR; with no letter by 2027-05-18 the licensee asks the IARU Satellite Advisor for status and the contract proceeds on the requested windows (`mission/SCHEDULE.md` DA-09) | F1 | Licensee | Panel |
| 2027-03 to 2027-05 | ITU API/A special section published (FCC forwarding time unknown: ask in the consultation) | F3 | FCC | F2 |
| 2027-03-31 | NOAA reply filed | F8 | PI | |
| 2027-05-31 (contract) | Launch-vehicle determination (signed contract, D7 deadline 2027-06-30) | — | PI | Funding |
| **2027-06-30** (contract + 30 days) | **Replacement F2** with the launch vehicle, integrator, orbit, IARU frequencies, final ODAR | F2/F5 | Licensee | Contract, IARU letter |
| 2027-06-30 (with the replacement F2) | ODAR final with the contract orbit (DAS re-run if the orbit or mass changed) | F4 | S&MA | Contract |
| 2027-09-30 | ISS/NRCSD backup decision (D7); if taken, replacement F2 and an IARU revision for the 51.6° orbit | F5, F1 | PI | |
| 2027-10-18 | Export counsel opinion; TAA in place if the integrator is foreign | F9 | PI | |
| 2027-12-15 | Space-Track registration and conjunction-response certification | F10 | Ops lead | Owned by SPACE_SAFETY plan |
| **2028-06-17** (I − 90 days) | **Last date for any replacement pre-space notification** | F5 | Licensee | |
| 2028-08-14 to 2028-09-02 | Provider deliverables with proof of F1, F2, F4, F9 | F11, F12 | PI | |
| L = 2028-11-01; first transmission L + 45 min | Launch; RF silence 45 min | — | | |
| **2028-11-08** (first transmission + 7 days) | **In-space notification** | F6 | Licensee | |
| 2030-11-01 (end of the 2-year design life) | Passivation by telecommand early in a sunlit pass: the charge disconnect fired while transmitting, the final report, then TX-OFF (`docs/operations/END_OF_LIFE_PLAN.md` §3) | — | Ops | Passivation commands (SQSO-ICD-013 v0.10 §6.4, implemented; EM test V-EPS-8) |
| **2031-02-01** (termination + 3 months; 24 h if FCC-ordered) | **Post-space notification** | F7 | Licensee | |
| ≤ 2035-11-01 | Re-entry (5 years after mission end; 2.8–3.7 years after it predicted at 475 km with the wings deployed, nominal and conservative, 1.6–1.85 kg, ODAR Draft D §6.2) | — | | ODAR |

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

## 6. FCC Space Bureau Consultation Agenda (RID REG-10)

Sent with the consultation request by **2026-11-15**; meeting by **2027-01-05**, before the early pre-space filing of
**2027-01-19**. **Owner: the licensee (KK6GDB).** Each question names the disposition that waits on the answer and what
the project proposes; the answers are recorded in [LICENSING_PATHWAY_DETERMINATION.md](LICENSING_PATHWAY_DETERMINATION.md)
by 2027-01-08 and carried into F2 and F1.

| # | Question | Disposition that depends on it | Project's proposal | If the answer differs |
|---|----------|--------------------------------|--------------------|-----------------------|
| Q1 | **Loss-of-contact timeout (D3):** is N = 30 days at launch, 14 days after commissioning (settable 1–90) acceptable as the 97.207(b) assurance; does a reversible transmit inhibit count as termination of transmissions for the post-space notification (97.207(g)(3)); is an autonomous passivation M days after the inhibit wanted | D3 (N, M); SQSO-EOL-001 EOL-4; F2 §3 "Loss of command contact" | N = 30 / 14 days; M not implemented (ODAR §4 recommends against it) | N changed by `SET_CONTACT_TIMEOUT` (no flight change); M would need a flight-software change before launch |
| Q2 | **ITU API path and timing (REG-05):** will the Commission forward the API from the early F2; how long forwarding takes; standard API/notification or Resolution 32 short-duration mission (≤ 3 years, not extendable) | F3 dates; F1 item "API data sent" | Standard path if an extension beyond 3 years may be wanted; otherwise Res. 32 | F1 and F3 dates move; `ITU_APPENDIX4_DATA.md` §F |
| Q3 | **Early pre-space filing (REG-09):** is a notification before launch-vehicle determination accepted when the orbit is definite (DA 13-445 fn. 6), followed by a replacement within 30 days of the contract and a last replacement 90 days before integration | F2 plan (2027-01-19, 2027-06-30, 2028-06-17) | File early, replace after the contract | File at the contract instead (by 2027-06-30); the IARU request then cites the later API date |
| Q4 | **Beacon kept with a failed receiver (REG-11, REL-04):** may the beacon and housekeeping frames continue after N days when the receiver is known failed (recovery tried, health line in fault), with the transponder off and TX-OFF still effective if a command gets through | SRS-F116/F117; IARU item 7.9; F2 §3 | Keep the exception (it preserves the evidence of the failure) | Remove before launch: one condition in `contact_monitor.rs` |
| Q5 | **Completion of the mission for a failed spacecraft (ODA-05):** reading A (an early failure ends the mission, the 5-year clock starts then) or reading B (the planned 2 years) for 97.207(g)(1)(vii)(D)(1) | BASELINE §3 band: 475 km target, 485 km cap (reading B) or 460 km (reading A); the RFQs ask for both (SQSO-OAP-001 §2.2) | Reading B, with the 485 km cap on the 3σ-high mean altitude | Contract at 460 km; ODAR and F2 at 460 km |
| Q6 | **US270 power limit:** does the 50 W PEP limit in the US270 areas apply to the transmitter's PEP (at its output), not to EIRP, so that a station near Beale AFB may radiate ≥ 64 dBm EIRP from ≤ 50 W PEP into a stacked pair; and the 97.313(f) reading for telecommand stations | Decision C-14 variant B (SQSO-GND-001 §3.1; the Bay Area and Fairbanks sites) | US270 limits transmitter PEP | Variant B needs the case-by-case authorization (HQ SpOC Spectrum Management Office) or a variant A site outside US270 |
| Q7 | **AI-generated beacon text:** is plain-language text composed on board (a selector among pre-approved templates in the baseline; generated lines from a small language model as a gated experiment, with a content filter, no numbers, the station's own call sign only) consistent with 97.113 and 97.119, and with the station being under the control operators' responsibility (97.207(a)) | D5; `docs/BEACON_SPEC.md` §6; F1 item 5.4; F2 statement of amateur character | Templates in the baseline; the generated-text experiment only behind the filter | Fly templates only (the experiment stays disabled; no flight change) |
| Q8 | **Part 5, only if a paid contract arises (D2):** can one spacecraft carry the Part 97 station and a separate Part 5 station with its own licensee and radio; the pathway's §7.1 questions (research §5 R6) | D2 terms | No paid contract today; Part 97 only | A second radio, a Part 5 filing and combined-mission coordination |
| Q9 | **Debris analysis tool:** is NASA DAS 3.2.6 with the project's lifetime and casualty models as cross-checks acceptable, and are DAS results expected in the early filing | F4 and F2 (ii), (iv)(A)(1) | DAS run before the early filing (§7) | A higher-fidelity tool, or the early filing waits for DAS |
| Q10 | **Filing channel:** ICFS or email (DA 13-445 listed amsat.spacecap@fcc.gov in 2013) | F2, F5, F6, F7 | As the staff advise | — |

## 7. NASA DAS Before the Early Filing (RIDs ODA-02, SE-14)

NASA DAS is software distributed under a NASA software usage agreement (NASA Software Catalog), not a hardware test.
97.207(g)(1)(ii) and (iv)(A)(1) ask for its results (or a higher-fidelity tool's) in the pre-space notification, so the
run comes before the early filing of 2027-01-19, not after it.

| Step | Date (no later than) | Owner |
|------|----------------------|-------|
| Request the DAS 3.2.6 usage agreement | 2026-11-06 | Licensee |
| Agreement in hand; DAS installed on a Windows machine | 2026-12-15 | Licensee, S&MA |
| DAS run with the controlled MEL (`docs/environment/MASS_BUDGET.md` v3.1) and the ODAR Draft D inputs | 2027-01-08 | S&MA |
| ODAR Draft E with the DAS results; F2 debris section filled | 2027-01-12 | S&MA, licensee |

**Pass criterion: DAS reproduces ODAR Draft D:** re-entry ≤ 5 years after the end of the 2-year mission at the 485 km
cap with 1.85 kg (wings deployed, DAS's solar-flux table), so that P ≥ 0.9 holds (reading B, with the 0.29 %
wings-failed probability); casualty risk ≤ 1:10,000 with no fragment above 15 J; collision with objects ≥ 10 cm
< 0.001. If DAS gives a longer life than the project model, the cap is lowered to the highest altitude DAS passes and
BASELINE §3 is changed before the filing. If the agreement is not in hand by 2026-12-15, the chair decides between
holding the early filing for DAS (the 30-day and 90-day deadlines leave room) and a higher-fidelity tool.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial roadmap (funding audit) |
| B | 2026-10-27 | CDR review (communications panel): rules re-checked against the eCFR of 2026-10-01 and the Federal Register (no change to 97.207/97.211 since the 2024-09-09 five-year rule); 47 CFR 2.106 US270 and 97.313(f) added; measured IARU lead time (median 57 days); ITU API timing and Res. 32; F13 (telecommand-station designation); dated plan from 2026-10 to the post-space notification |
| C | 2026-10-27 | CDR 2026 consistency pass: §3 launch-dependent dates for the planning launch 2028-11-01 and integration 2028-09-15 (BASELINE Rev E §3, `mission/SCHEDULE.md` Rev B): I − 90 days 2028-06-17, provider deliverables, in-space 2028-11-08, end of life 2030-11-01, post-space 2031-02-01, re-entry at 475 km |
| D | 2026-10-27 | CDR round 3 (RIDs REG-10, REG-11, ODA-02, ODA-05, SE-14): §6 the FCC consultation agenda (Q1–Q10: D3 N and M, REG-05, REG-09, REG-11, ODA-05, the US270 reading, AI-generated text, Part 5, the debris tool, the filing channel), owner the licensee, request 2026-11-15, meeting by 2027-01-05; §7 NASA DAS before the early filing (agreement requested 2026-11-06, run 2027-01-08, ODAR Draft E 2027-01-12) with its pass criterion; §3 dates (DAS moved from 2027-02-09; ODAR final with the contract orbit), passivation implemented and fired before TX-OFF, re-entry from ODAR Draft D |
| D.1 | 2026-10-27 | Documentation audit (post-CDR 2026): §3 the station criterion of DA-10 (≥ 1,000 km apart, each able to command alone, BASELINE Rev E §1) and the DA-09 fallback for a late IARU letter. No date changed |
