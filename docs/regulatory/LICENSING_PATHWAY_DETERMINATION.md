# Licensing Pathway Determination
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-003
**Revision:** A
**Date:** 2026-10-14
**Status:** Draft for licensee/PI decision — not legal advice; confirm with FCC Space Bureau staff before filing
**Baseline:** [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md)

---

## 1. Question

Which U.S. authorization regime applies to SMART-QSO's radio transmissions, and what must be true of the
program (including its funding arrangements) for that regime to remain valid?

## 2. Sources (primary, retrieved 2026-10-14)

| Ref | Source |
|-----|--------|
| [R1] | 47 CFR 97.207 Space station; 97.209 Earth station; 97.211 Space telecommand station; 97.113 Prohibited transmissions — eCFR, current as of 2026-10-13 (https://www.ecfr.gov/current/title-47/chapter-I/subchapter-D/part-97) |
| [R2] | 47 CFR 5.3 Scope of service; 5.63 Supplemental statements; 5.64 Special provisions for satellite systems — eCFR, current as of 2026-10-13 |
| [R3] | FCC Public Notice DA 13-445, *Guidance on Obtaining Licenses for Small Satellites*, 28 FCC Rcd 2555 (2013) (https://docs.fcc.gov/public/attachments/DA-13-445A1_Rcd.pdf) |
| [R4] | IARU Amateur Satellite Frequency Coordination Request, form v40 (effective 15 May 2020) and instructions (https://www.iaru.org/reference/satellites/, page updated 26 Jul 2026) |
| [R5] | ITU Radio Regulations Nos. 1.56, 1.57, 22.1, 25.2A, 25.11 (as excerpted in Annex 1 of [R4]) |

## 3. Options Considered

| Regime | Applies when | Fit for SMART-QSO |
|--------|-------------|-------------------|
| **FCC Part 97 (amateur-satellite service)** | Non-Federal station operated by licensed amateurs for self-training, intercommunication and technical investigation, **without pecuniary interest** ([R1] 97.113(a)(2)-(3); [R5] RR 1.56/1.57). No license document is issued; the control operator files pre-space, in-space and post-space notifications ([R1] 97.207(g); [R3] fn. 1). | **Recommended.** The mission's core function, an open amateur transponder plus public telemetry, is amateur in character. The onboard-AI experiment counts as a "technical investigation" only while amateurs carry it out for its own sake, not as deliverable research for a sponsor. |
| FCC Part 5 (experimental) | Scientific or technical research, experimentation under U.S. Government contract, communications essential to a research project ([R2] 5.3(a),(c),(d)). The applicant is the party controlling mission objectives, design, launch tendering and operations ([R3]). | **Contingency.** Required if the funding structure creates a pecuniary interest or a sponsor-directed research purpose (see §5). Amateur-band use under Part 5 is case-by-case and still needs IARU coordination ([R3] item h). |
| NTIA (Federal) | The station is owned or operated by a Federal agency. | **Not applicable** unless a Federal agency becomes the operator. Earlier repository text naming a NASA center as operator and listing Federal TT&C frequencies (401/449 MHz) is withdrawn (see §6). |
| FCC Part 25 (commercial) | Commercial satellite services. | Not applicable. |

## 4. Determination

**SMART-QSO shall be authorized under FCC Part 97 as an amateur space station**, with all transmissions in the
144–146 MHz and 435–438 MHz amateur-satellite segments ([R1] 97.207(c)(2), 97.209(b)(2), 97.211(c)(2)). This holds
only while every condition in §5 holds. If any condition fails, the program shall pivot to Part 5 before the
pre-space notification deadline (see the [Regulatory Roadmap](REGULATORY_ROADMAP.md)).

## 5. Conditions That Must Hold for Part 97

| # | Condition | Basis | How SMART-QSO satisfies it |
|---|-----------|-------|----------------------------|
| C1 | Space station licensee and every control operator hold valid U.S. amateur licenses; the licensee can remove the satellite from the manifest and disable it. | [R1] 97.207(a); [R3] "Who Should Apply?" | Licensee and backup control operators to be named in BASELINE_PARAMETERS §1 (**OPEN — decision D1**). |
| C2 | No person or organization is compensated, directly or indirectly, for operating the satellite or its ground stations. | [R1] 97.113(a)(2); [R4] form item 4.10 | Funding agreements shall exclude operations labor from any paid scope. Build and test may be funded by **grants or donations with nothing required in return** ([R4] p.1 note on "without pecuniary interest"). |
| C3 | No communications on behalf of an employer, and no sale of services or data. | [R1] 97.113(a)(3); [R4] p.1 | All telemetry, beacon formats and AI outputs are published openly (BEACON_SPEC, ICD_SPACE_GROUND). No exclusive data rights to any funder. |
| C4 | Nothing transmitted from the spacecraft is encoded to obscure its meaning; decoding information is published. | [R1] 97.113(a)(4); [R5] RR 25.2A; [R4] form §6 | Downlinks are plain AX.25/AFSK or FM. Formats are published before launch (IARU form item 6.1). |
| C5 | Only telecommand stations designated by the licensee may initiate, modify or terminate spacecraft functions. They may use codes that obscure command content (authentication/encryption of telecommand only). | [R1] 97.3(a)(45), 97.211(a)-(b) | Telecommand uplink is authenticated (HMAC with replay counter) from designated stations only. **Public uplinks, including federated-learning model contributions, are user data, not telecommand.** No public uplink may change spacecraft mode, RF state or power without an authenticated command from a designated station. |
| C6 | Capability to cease transmissions by telecommand whenever ordered by the FCC; sufficient command stations in place before launch. | [R1] 97.207(b); [R5] RR 22.1, 25.11 | A hardware-enforced TX-OFF command path that does not depend on the AI payload or application software. At least two geographically separate telecommand stations at launch. See IARU request item 7.8/7.9. |
| C7 | Operations stay within 144–146 MHz / 435–438 MHz and Part 97 emission standards (no non-amateur TT&C). | [R1] 97.207(c), 97.307 | Federal-band TT&C removed from baseline. |
| C8 | Orbital debris disclosures meet 97.207(g)(1)(i)–(vii), including re-entry no later than **5 years after mission completion** with ≥ 0.9 disposal success probability, and casualty risk < 1:10,000 (DAS). | [R1] 97.207(g)(1)(vii)(D) | [ODAR](filings/ODAR_SMART-QSO.md). The deployment altitude is selected in the [launch trade](../funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md) so that passive decay closes. |

## 6. Corrections to Prior Repository Statements

These earlier statements are inconsistent with this determination and are corrected in this revision:

| Prior statement | Location (pre-audit) | Correction |
|-----------------|----------------------|------------|
| "Operator: NASA Ames Research Center"; "Non-commercial: NASA research mission" | `docs/regulatory/IARU_COORDINATION.md` §2.1, §2.3 | No Federal operator is established. A NASA-operated station would be NTIA-authorized, not Part 97, and IARU coordination would then need a separately identified amateur component ([R4] item 4a). The operator is the amateur licensee (C1). |
| TT&C on "449.x MHz" uplink / "401.x MHz" downlink via NTIA; "NASA Form 1722 Submitted"; "Coordination with DoD/NOAA Complete" | `docs/regulatory/SPECTRUM_LICENSE.md` §2.1, §4, §6.2 | Withdrawn. No evidence of any such filing exists in the repository, and a non-Federal Part 97 station cannot hold Federal assignments. TT&C uses amateur telecommand (C5). |
| "No uplink required … beacon-only mission" | `docs/regulatory/IARU_COORDINATION.md` §3.3; `docs/REGULATORY.md` | Inconsistent with the transponder mission in README/CLAUDE.md. The baseline includes a 435–438 MHz uplink for the transponder and telecommand. |
| "Callsign identification every 10 minutes" stated as a Part 97 requirement | `docs/regulatory/SPECTRUM_LICENSE.md` §3.1 | Space stations are exempt from 97.119(a) station identification. SMART-QSO identifies anyway, as good practice and to make its signals attributable. |

## 7. Funding-Structure Guidance for Prospective Funders

| Funding instrument | Compatible with Part 97? | Notes |
|--------------------|--------------------------|-------|
| Unrestricted grant or donation (e.g., amateur-radio foundations, AMSAT member societies) | Yes | Preferred. |
| Educational grant for student training, with the amateur transponder as the primary mission | Yes, if C1–C5 hold | Mark the mission "Amateur combined with Educational" on IARU form item 5.1. |
| In-kind launch through NASA CSLI | Generally yes | CSLI provides the launch; the mission must still meet the ELaNa licensing requirements. Verify current CSLI terms (see launch trade). |
| Federal procurement contract with research deliverables (e.g., data rights, sponsor-defined experiment objectives) | **No — triggers Part 5** | [R2] 5.3(c); [R3] notes "missions conducted under government contract" are experimental. |
| Commercial sponsorship with paid placement or data rights | **No** | Pecuniary interest (C2, C3). |

## 8. Decisions Required

| ID | Decision | Owner | Needed by |
|----|----------|-------|-----------|
| D1 | Name the space station licensee (call sign, license class) and at least two control operators and telecommand stations | PI | Before IARU submission |
| D2 | Confirm that all funding instruments meet §7 (no operations compensation, no data rights) | PI + funders | Before accepting funds |
| D3 | Pre-filing consultation with FCC Space Bureau (amateur satellite staff) on the AI-experiment / federated-learning concept | Licensee | Before IARU submission |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial determination (funding audit) |
