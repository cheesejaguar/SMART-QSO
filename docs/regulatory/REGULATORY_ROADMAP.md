# Regulatory and Launch-Authorization Roadmap
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-004
**Revision:** A
**Date:** 2026-10-14
**Status:** Plan. **No regulatory filing has been made as of this date.**
**Governing determination:** [LICENSING_PATHWAY_DETERMINATION.md](LICENSING_PATHWAY_DETERMINATION.md) (Part 97)

---

## 1. Filing Inventory

| ID | Filing / approval | Authority | Rule / source | Draft in repo | Status |
|----|-------------------|-----------|---------------|---------------|--------|
| F1 | IARU amateur-satellite frequency coordination request (form v40) | IARU Satellite Advisor (satcoord@iaru.org) | IARU form v40; ITU RR 1.56/1.57/25 | [IARU_COORDINATION_REQUEST_v40.md](filings/IARU_COORDINATION_REQUEST_v40.md) | Not submitted |
| F2 | FCC Part 97 **pre-space** notification, with ITU Appendix 4 / Res. 642 data and orbital debris disclosures | FCC Space Bureau | 47 CFR 97.207(g)(1) | [FCC_97-207_PRE_SPACE_NOTIFICATION.md](filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md) | Not submitted |
| F3 | ITU Advance Publication Information (API) and later notification | ITU-BR, filed **by the FCC** on the applicant's data | RR Articles 9 and 11; Appendix 4 | [ITU_APPENDIX4_DATA.md](filings/ITU_APPENDIX4_DATA.md) | Not submitted |
| F4 | Orbital Debris Assessment Report (NASA-STD-8719.14C format), the basis for the F2 debris disclosures | FCC (via F2); NASA if launched through CSLI/ELaNa | 97.207(g)(1)(i)–(vii); NASA-STD-8719.14C | [ODAR_SMART-QSO.md](filings/ODAR_SMART-QSO.md) | Draft. **DAS run pending** |
| F5 | Replacement pre-space notification if any material item changes | FCC Space Bureau | 97.207(g)(1)(viii) | Use F2 | — |
| F6 | FCC **in-space** notification | FCC Space Bureau | 97.207(g)(2) | [FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md](filings/FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md) | Template |
| F7 | FCC **post-space** notification | FCC Space Bureau | 97.207(g)(3) | same | Template |
| F8 | NOAA remote-sensing license applicability (no license expected) | NOAA CRSRA | 15 CFR 960.4, 960.5(a) | [NOAA_REMOTE_SENSING_DETERMINATION.md](filings/NOAA_REMOTE_SENSING_DETERMINATION.md) | Determination drafted; NOAA confirmation not requested |
| F9 | Export classification (9A515.a.5; ITAR for integration services) and any license/TAA | BIS / DDTC | 15 CFR 774 Supp. 1; ITAR | [EXPORT_CLASSIFICATION_MEMO.md](filings/EXPORT_CLASSIFICATION_MEMO.md) | Self-classification drafted; counsel review not started |
| F10 | Space-Track / TraCSS registration; conjunction-response certification | USSF 18 SDS / Commerce OSC | 97.207(g)(1)(iv)–(v) | [SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md](filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md) | Not started |
| F11 | Launch-provider regulatory and safety deliverables (CubeSat Acceptance Checklist, battery data, mass properties, proof of FCC filing, ODAR, export statement) | Launch provider / integrator; NASA LSP if CSLI | CDS Rev 14.1; provider ICD; LSP-REQ-317.01 (CSLI) | [LAUNCH_PROVIDER_DELIVERABLES.md](filings/LAUNCH_PROVIDER_DELIVERABLES.md) | Checklist drafted |
| F12 | FAA payload review information (provided to the launch operator) | FAA/AST via launch operator | 14 CFR 450.43 | covered in F11 | — |

## 2. Sequence and Timing

Dates are relative to **L** (launch) and **I** (integration of the CubeSat into the dispenser or launch vehicle). For
rideshare CubeSats, I is typically 1–3 months before L, so "90 days before integration" can fall 4–6 months before launch.

```
 T-24..T-18 mo  Informal consults: AMSAT-NA / ARRL (IARU form 4.7-4.9), FCC Space Bureau amateur staff
                (pathway decision D3), NOAA CRSRA (F8), export counsel (F9)
 T-18 mo        F1 IARU request submitted with the date API data was sent to the FCC
                (or file F2 early: FCC accepts notifications before launch-vehicle determination
                 when the orbital range is definite - DA 13-445 fn. 6)
 T-18..T-12 mo  IARU panel iteration (meets every 2-3 weeks); coordination letter received
 LV determined  F2 pre-space notification due WITHIN 30 DAYS of launch-vehicle determination
                (attach IARU letter, Appendix 4 SpaceCap data, ODAR/debris disclosures)
 I-90 days      LATEST date for F2, and for F5 replacement filings
 I-90..I-30     Provider deliverables (F11); Space-Track registration (F10)
 L              Launch; 45-min RF inhibit after deployment (CDS Rev 14.1)
 L+<=7 days     F6 in-space notification (7 days after FIRST TRANSMISSION)
 End of tx      F7 post-space notification within 3 months (24 h if FCC-ordered)
```

**Critical-path observation for funders:** SMART-QSO cannot be integrated without F2 on file at least 90 days
beforehand, and providers usually want to see the IARU letter and FCC filing before they accept the spacecraft. An IARU
request submitted **less than about 12 months before the target integration date** is a schedule risk.

## 3. Correction of Prior Status Claims

As of 2026-10-14, **no** IARU request, FCC notification, ITU filing, NTIA request, NASA Form 1722 or DoD/NOAA
coordination exists in the repository or is evidenced anywhere. Earlier text claiming "Application submitted",
"Coordination requested", "License pending", "Submitted" or "Complete" for these items has been corrected (see
`docs/regulatory/SPECTRUM_LICENSE.md` Rev B and `docs/regulatory/IARU_COORDINATION.md` Rev B).

## 4. Owners

All filings are signed by the **space station licensee** (decision D1 in the pathway determination). The PI owns F8,
F9 and F11. The operations lead owns F6, F7 and F10.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial roadmap (funding audit) |
