# D2 Licensing Research: Paid Contracts vs. a Part 97 Amateur Spacecraft (SMART-QSO)

> Research input to decision D2, prepared 2026-10-15 and imported unchanged into the repository. Where it mentions working files (raw downloads, extracted texts, scripts) or says no repository files were modified, it refers to the research session; those files were not kept. How the decision was taken and applied is recorded in `docs/BASELINE_PARAMETERS.md` section 9.


Research date: 2026-10-15. eCFR text retrieved 2026-10-15 (version dated 2026-10-01). Raw downloads are in `.../scratchpad/research/raw/`.

> **Not legal advice.** This is a research synthesis from public sources. Items tagged **[judgment]** are my inferences, not FCC, IARU or NASA positions. Items tagged **[secondary]** come from AMSAT, ARRL or news pages rather than the rule text or an agency document. Communications counsel (ideally with FCC Space Bureau / OET experience) should confirm every point before money is accepted or a filing is made. No repository files were modified.

---

## 1. Executive Summary

**Part 97 and paid contracts do not mix on the same transmissions.** 47 CFR 97.113(a)(2) bars amateur "communications for hire or for material compensation, direct or indirect, paid or promised." 97.113(a)(3) bars communications in which the licensee or control operator "has a pecuniary interest, including communications on behalf of an employer." The only exceptions (emergency drills, occasional for-sale notices, a teacher's classroom use, W1AW-type club bulletins) do not fit a satellite data contract. The rule text regulates *communications*, not funding as such. Money for building or launching the spacecraft with nothing required in return is a different case from money paid for data or services that ride the amateur link. FCC's own small-satellite guidance (DA 13-445, 2013) classes "missions conducted under government contract" as experimental operations licensed under Part 5 or Part 97, and notes that Part 5 scope includes "experimentation under contractual agreement with the United States Government." The bar on Part 97 comes from 97.113, not from the notice. I found no FCC document that withdraws or supersedes DA 13-445. FCC 19-81 (2019) left the Part 5 and Part 97 processes unchanged. FCC 26-47 (adopted July 2026, not yet effective) keeps the amateur satellite service under Part 97 (new §100.5(f)) and makes only limited Part 5 changes (ITU filing timing, debris cross-reference), with broader space-experimental changes only proposed in its FNPRM.

**Part 5 is also non-commercial, and that is the point most likely to be missed.** The FCC Space Bureau Part 5 page (updated 2024-09-27) and its 2024-09-10 public notice say experimental licensees may not provide commercial service, charge fees, or receive payments for products or services of operation. Section 5.3(c) does allow work "under contractual agreement with the United States Government," so a NASA Ames contract fits Part 5 (with the agency and contract number disclosed under 5.63(c)(2)). Selling data to a private party is a Part 25 (commercial) activity. A university payer such as Santa Clara is not the "United States Government," so 5.3(c) does not apply on its face. That payer's use case would have to fit 5.3(a) or (d) (scientific/technical radio research, or communications essential to a research project). **[judgment]** Often the better fit is for the university itself to hold the Part 5 license for its experiment link, as the University of Washington did on HuskySat-1.

**Dual licensing one spacecraft has a precedent, but not the one the owner needs.** HuskySat-1 (launched 2019-11-02) flew a Part 5 license held by the University of Washington and a Part 97 station for AMSAT's transponder module. UW's Part 5 operations ran with the transponder off, then AMSAT took over under Part 97. AMSAT said the two licenses were the only way to make the launch (ARRL, 2019-11-13 **[secondary]**). The IARU coordinated both the telemetry and transponder frequencies. The distinguishing facts are that two different entities held the two authorizations and that they were time-sliced. I found no FCC statement on whether one individual may hold both a Part 97 station and a Part 5 station on the same spacecraft. The IARU "combined mission" rule requires each payload, *including its TT&C*, to use frequencies of its own service. IARU declined the NILA CubeSat (2025-01-27) as a commercial/scientific spacecraft. **[judgment]** The lowest-risk way to keep the amateur transponder is to stay Part 97 with grants and donations (option c). A hybrid (option a) is feasible only with a separate Part 5 licensee, a hard separation of data paths, and a second radio. All-Part-5 (option b) loses the public transponder.

| Question | Short answer |
|---|---|
| Can a paid procurement contract be performed over the Part 97 link? | No (97.113(a)(2),(3)). |
| Can NASA Ames pay under Part 5? | Yes, as an experiment "under contractual agreement with the United States Government" (5.3(c)). No commercial service or fees to customers. |
| Can a private university pay for data? | Not under 5.3(c). Possible under 5.3(a)/(d) if it is its own research. Commercial service goes to Part 25. |
| Can one spacecraft carry both? | Yes, with precedent (HuskySat-1) and IARU "combined mission" acceptance. Different licensees, separate frequencies, and separate TT&C per payload. |
| Best non-amateur spectrum? | UHF: 400.15-401 MHz down, 449.75-450.25 MHz up (needs ITU RR 9.21 agreement). S-band is restricted for non-Federal operators. |
| Does a NASA CSLI launch bar Part 97? | No rule says so. CSLI is a no-funds CRADA. Eligibility is institutions, nonprofits and NASA centers. The next solicitation is expected in 2028. |

---

## 2. Technical Overview: the three FCC regimes

| | **Part 97 (amateur-satellite)** | **Part 5 (experimental)** | **Part 25 streamlined small satellite** |
|---|---|---|---|
| Purpose | Self-training, intercommunication, technical investigation "solely with a personal aim and without pecuniary interest" (97.3(a)(4), RR 1.56/1.57) | Experimentation in the radio art, or "essential communications for research projects" (5.5); list in 5.3 | Any NGSO operation, commercial or not (FCC 19-81 ¶11-12) |
| Money for communications? | Prohibited (97.113(a)(2),(3)) | Government-contract experiments allowed (5.3(c)). No commercial service, fees or payments for services (FCC Part 5 page, 2024-09-27) | Yes (commercial service allowed) |
| Who holds it | Licensed amateur operator; organization cannot be the control operator (IARU spec rev 15.5, 2006) | Party that controls mission objectives, design, launch tender and on-orbit operation (DA 13-445) | Licensee entity |
| Spectrum | 144-146, 435-438, 2400-2450 MHz etc. (97.207(c)) | Any Table band if justified (5.85(a)(1)); unprotected, non-interference (5.84) | Allocated bands; protected only where allocation supports it |
| Term | Tied to operator license; new notifications per mission (DA 13-445) | 2 yr, up to 5, renewable up to 5 (5.71(a)); STA for 6 months or less | Up to 6 yr in orbit for this track (25.122(c)(2)) |
| FCC fee | None found for 97.207(g) notifications | $140 new station authorization or STA (47 CFR 1.1103, as amended 90 FR 17013, 2025-04-23) | $2,555 per call sign for small satellite application (1.1107 Table 9) plus earth station fee ($425 single-site transmit) plus annual regulatory fees (not retrieved) |
| ITU | Notification via FCC; API special-section publication for the amateur-satellite service is exempt from ITU charges (Decision 482 (2013) ¶11). A space-station notification exemption is not explicit in the 2013 text | SpaceCap/GIMS filing plus cost-recovery commitment, filed to OET (FCC Part 5 checklist, 2024-10-03) | Applicant's SpaceCap; FCC submits |
| Encryption | Not allowed except telecommand (97.113(a)(4), 97.211(b)) | Allowed | Allowed |
| Public access | Required in practice (IARU: open access) | Not required (and 5.125 limits whom you may talk to) | Not required |

---

## 3. Implementation Considerations

### 3.1 Required filings and sequence for each path

- **Part 97:** pre-space notification to the Space Bureau within 30 days of launch-vehicle determination and no later than 90 days before integration; in-space notification within 7 days after transmissions start; post-space notification within 3 months after transmissions end (97.207(g), source note shows amendment 89 FR 65223, 2024-08-09). The FCC pre-launch package also expects a draft ITU Appendix 4 (SpaceCap) and an IARU coordination letter (DA 13-445). NASA's 2015 flow chart says to submit IARU coordination about 10 months before integration.
- **Part 5:** Form 442 for more than 6 months, STA for 6 months or less. Needs an orbital debris plan (5.64(b)), an ITU SpaceCap filing, a cost-recovery commitment, a stop-buzzer contact, and the contract number if it is a U.S. Government contract (5.63(c)(2)). OET "strongly recommends" filing not later than 30 days after launch-vehicle determination and no later than 90 days before integration (DA 13-445).
- **Part 25 small satellite:** 25.122 certifications (10 or fewer satellites, six years or less in orbit, 600 km or lower or propulsive, 180 kg or less, 10 cm or larger in smallest dimension). A 1U meets the 10 cm floor.

### 3.2 Patterns and anti-patterns **[judgment]**

- *Pattern:* two clearly separate "stations" on one bus. Separate radio, separate frequencies, separate telecommand path, separate licensee, separate data path, and a documented rule for what content may go where.
- *Pattern:* paid scope limited to deliverables that travel only over the non-amateur link.
- *Anti-pattern:* using the amateur telecommand uplink for routine control of the contract payload (IARU spec rev 15.5 says amateur-band uplinks to a multi-service satellite "should not be used for routine space telecommand, space telemetry, or other operation involving the portion of the space station operating in another service").
- *Anti-pattern:* beacon text composed from paid experiment output (see Section 5, R3).
- *Anti-pattern:* a contract that gives a sponsor the right to direct or stop transmissions, which makes the station Federal (Section 6, Q4).

### 3.3 Performance and schedule

Typical FCC Part 5 review is 45 to 180 days according to NASA's 2015 guidance. A 2025 analysis of FCC data (2000-2022) reports an average of about 98 days for Part 5 applications and 47 for STAs **[secondary]**. NTIA coordination adds time and I found no published NTIA timeline. The repository baseline has launch-vehicle determination by 2027-06-30 (D7), so the Part 97 clock (30 days after, 90 days before integration) and any Part 5 clock both start there.

### 3.4 Security and openness

Telecommand may be coded to obscure meaning (97.211(b)). Downlink telemetry and communications may not be encrypted, and IARU expects published formats. The Part 5 link may be encrypted. If NASA or a sponsor wants authenticated or encrypted data, it belongs on the Part 5 link.

---

## 4. Comparative Analysis

### 4.1 Options matrix

| | **(a) Dual license: Part 97 + Part 5 radio** | **(b) All Part 5 (no amateur transponder)** | **(c) Stay Part 97; grants/donations only** |
|---|---|---|---|
| Keeps public FM transponder | Yes | No. Part 5 stations "may communicate only with other stations licensed in the Experimental Radio Service" unless a showing is made (5.125) | Yes |
| Can take NASA Ames contract money | Yes, for the Part 5 deliverable | Yes (5.3(c)) | No |
| Can take university payment | Possibly, if university is the Part 5 licensee or research fits 5.3(a)/(d) **[judgment]** | Same as (a) | No (payment for services) |
| New hardware | Second transmitter (UHF or S-band) plus antenna; second ground path | Replaces amateur radio with Part 5 radio | None |
| IARU | Coordinate as combined mission; risk of decline if mission reads as commercial/scientific | Only if using amateur bands, and IARU needs a written administration direction | Standard amateur coordination |
| Regulatory burden | Highest: two authorizations, NTIA possible, ITU cost recovery | Medium | Lowest |
| Pecuniary-interest risk to the amateur side | Moderate, managed by separation | None for amateur (no amateur service) | Low if grants have nothing required in return |
| Fits current power baseline (2.39 W tumbling / 3.58 W sun-pointed average) | Poor to marginal. Second TX load competes with the D4 descope trigger **[judgment]** | Depends on radio chosen | Baseline |

### 4.2 Hardware ROM for option (a) second radio **[judgment, ROM]**

Sources: NASA SmallSat State-of-the-Art ch. 9 (2025 ed., hosted 2026-05) Table 9-3; vendor prices from satsearch (page updated 2026-06-04) and the EnduroSat store as reported in web search results. I did not open the vendor pages directly, and quotes need confirming.

| Item | Mass | RF power | Price ROM | Notes |
|---|---|---|---|---|
| UHF transceiver (EnduroSat UHF Transceiver Type II; 400-403 / 430-440 MHz) | 94 g | 30-33 dBm | about $5.9k-6.3k | Data rate up to 9.6 kbps |
| UHF alternatives (GomSpace NanoCom AX100, 24.5 g; NanoAvionics SatCOM UHF, 7.5 g; ISISpace TRXVU, 75 g; AAC Clyde TRX-U, 140 g) | 7.5-140 g | 27-34 dBm | No current price found | |
| S-band transceiver (EnduroSat S-band Transceiver II, 2025-2110 / 2200-2290 MHz) | 246 g | 27-33 dBm | about $13k (EnduroSat store, labeled "S-Band Transceiver"); transmitter about $14.8k | |
| S-band alternatives (IQ Spacecom HISPICO tx 100 g; ISISpace TXS 132 g; SkyLabs NANOlink-S 126-391 g) | 100-460 g | 27-37 dBm | Older third-party figures $7.3k-9.6k | |
| Antenna (UHF turnstile / S-band patch) | not itemized | | Not priced here (one listing shows "4600" with no currency for an S-band patch) | Add |
| FCC fees | | | $140 (Part 5) plus ITU CHF (see Q4) | |
| Ground | | | Not researched | A UHF Yagi/SDR or S-band dish and licensing |

Volume is the harder constraint **[judgment]**. A 1U (10 x 10 x 11.35 cm) already has to hold the EPS, a roughly 50 Wh 4S1P 18650 pack, the OBC, the Jetson carrier, magnetorquers, and the 435 MHz receiver plus VHF/UHF transponder and beacon stack per the repo baseline. A second radio of 25-250 g plus a second antenna (and, for S-band, a patch that must see the ground) would need a mechanical trade that the baseline does not contain.

DC input power for each radio was not retrieved and must come from datasheets. Illustrative arithmetic: a 1-2 W RF transmitter at roughly 25-35 percent efficiency is about 3-8 W DC while keyed. At a 5-10 percent duty cycle that is roughly 0.2-0.6 W orbit-average. The baseline closes only against 2.39-3.58 W of generation, with Jetson at 0.4-0.8 W, so any second TX competes with the transponder duty cycle that D4 may already descope.

---

## 5. Practical Recommendations **[judgment throughout; not legal advice, counsel must confirm]**

**R1. [judgment] Default to option (c) until a signed paid contract exists.** Take grants and donations that require nothing in return. Do not sign any instrument with deliverables, data rights, or operations-labor payments.

**R2. [judgment] If a paid contract materializes, use the HuskySat-1 pattern, not a single-licensee pattern.** The sponsor or its research partner holds the Part 5 authorization and controls the experiment link (consistent with DA 13-445's statement that the Part 5 applicant "should be the party that ultimately controls decisions about the satellite's mission objectives"). KK6GDB stays the unpaid Part 97 licensee for the transponder and beacon.

**R3. [judgment] Protect the amateur beacon from the paid experiment.** The Smart Beacon Agent composes beacon text from onboard AI. AMSAT's 2018 paper says FCC generally treats data from an experiment that "can be used for financial gain" as pecuniary interest when it is sent via amateur radio, even if two-way amateur service is present **[secondary]**. NASA's SOA (2025) repeats the same restriction. Mitigation: paid deliverables (inference logs, power/thermal telemetry, model metrics) go only over the Part 5 link. The Part 97 beacon carries housekeeping and amateur-purpose text only, drawn from a defined amateur experiment (power-aware transponder scheduling). Do not let the beacon include content produced for a customer.

**R4. [judgment] Keep the amateur licensee and control operators out of the money.** No compensation for operating the amateur station (97.113(a)(2)). Avoid the licensee being an employee of the contracting entity acting as control operator on its behalf (97.113(a)(3), "on behalf of an employer"). If the licensee owns the contracting company, treat that as a pecuniary-interest question to put to the FCC in writing.

**R5. [judgment] Keep effective control with the non-Federal licensee.** NASA's 2015 spectrum guidance and NTIA Manual 8.2.17 treat a station as Federal if the agency can exercise effective control over the radio equipment and its operation. A contract clause that lets NASA direct or cease transmissions can convert the station to a Federal one (NTIA, not FCC). DA 13-445 also expects the amateur applicant to be able to pull the satellite from the manifest.

**R6. [judgment] Use D3 (FCC pre-filing consultation) to ask these questions in writing** (contacts historically `amsat.spacecap@fcc.gov` and the Space Bureau/OET; the 2013 notice's International Bureau address is obsolete because 97.207(g) now names the Space Bureau):
1. May one spacecraft carry a Part 97 station (licensed individual) and a Part 5 station held by a different entity, on separate radios and frequencies?
2. If the Part 97 licensee also controls the entity under contract for the Part 5 deliverable, does FCC regard the licensee as having a pecuniary interest in amateur communications?
3. Is AI-generated housekeeping text in an amateur beacon affected if the same AI experiment is also a contract deliverable?
4. Does FCC accept a NASA or university contract as the 5.3(c)/(d) basis, and is an SBIR/STTR award treated as a government contract?
5. Does FCC expect an IARU letter for the Part 5 radio?

**R7. [judgment] Ask IARU early, with the mission described as an amateur mission with a separate non-amateur payload.** IARU says it will continue to coordinate combined missions but "will only coordinate a non-amateur satellite if an administration directs in writing" (IARU news release 2017-06-30, updated 2020-01-30). NILA CubeSat (HEX20, 3U) was declined on 2025-01-27 as a commercial/scientific spacecraft.

**R8. [judgment] Do not rely on CSLI for the launch.** Eligibility is institutions, nonprofits and NASA centers, and the next solicitation is expected in 2028 (Section 6, Q5). The repo's D7 date (integrator determination by 2027-06-30) does not line up.

**Risks and mitigations**

| Risk | Consequence | Mitigation |
|---|---|---|
| FCC treats paid-experiment content on the amateur link as pecuniary interest | Part 97 notification rejected or station unlawful | R3, R4 |
| IARU declines combined mission | No amateur coordination, FCC may not accept pre-launch notification | R7; disclose separation; keep the amateur payload real |
| NASA contract confers effective control | Station becomes Federal; NTIA; no Part 97 | R5 |
| Part 5 grant slips | Cannot integrate (DA 18-368 warns operators against launching without authorization) | File 6+ months ahead; plan STA; keep Part 97 path independent |
| Second TX breaks power budget | D4 descope to beacon-only | Size DC load before committing |

---

## 6. Technical Deep Dive

### Q1. 47 CFR 97.113 as in force

Source: eCFR, 47 CFR 97.113, retrieved 2026-10-15. Source note: 58 FR 43072 (1993); 58 FR 47219 (1993), as amended 71 FR 25982 (2006); 71 FR 66462 (2006); 75 FR 46857 (2010); 79 FR 35291 (2014-06-20). No later amendment is listed, so the text below is the text in force.

- **(a)(2):** "Communications for hire or for material compensation, direct or indirect, paid or promised, except as otherwise provided in these rules."
- **(a)(3):** "Communications in which the station licensee or control operator has a pecuniary interest, including communications on behalf of an employer," with four exceptions:
  - (i) employer participation in emergency preparedness or disaster readiness tests or drills, limited in duration and scope (and one hour per week for tests not government-sponsored, with two 72-hour exceptions per year);
  - (ii) notices of availability for sale or trade of amateur apparatus, "not conducted on a regular basis";
  - (iii) a control operator "may accept compensation as an incident of a teaching position during periods of time when an amateur station is used by that teacher as a part of classroom instruction at an educational institution";
  - (iv) club-station bulletin/code-practice compensation, with 40 hours per week, six MF/HF bands, schedule published 30 days ahead.
- **(a)(4)** is not about compensation. It bars music on phone, communications facilitating a crime, "messages encoded for the purpose of obscuring their meaning, except as otherwise provided," obscene or indecent language, and false or deceptive messages, signals or identification. For this mission it means proprietary-encoded customer data cannot go over the amateur downlink. Telecommand is the exception (97.211(b)); space telemetry may be "specially coded messages" tied to spacecraft function (97.207(f)) but IARU expects the format published.
- **(a)(5):** "Communications, on a regular basis, which could reasonably be furnished alternatively through other radio services." **[judgment]** Routine delivery of a sponsor's data over the amateur link when a Part 5 link exists is open to challenge under this clause. I found no enforcement example.
- **(b)** bars broadcasting and one-way transmissions "except as specifically provided." One-way telemetry and telecommand are provided for (97.111(b)(3),(7); 97.207(e); 97.211(d)).
- **(c):** "No station shall retransmit programs or signals emanating from any type of radio station other than an amateur station," with narrow Government weather/propagation and manned-spacecraft exceptions.
- **(d):** "No amateur station, except an auxiliary, repeater, or space station, may automatically retransmit the radio signals of other amateur station." A space station may auto-retransmit (also 97.207(d)).

**Citation mismatch in DA 13-445.** Footnote 2 of the 2013 notice quotes the pecuniary-interest language and cites "47 C.F.R. § 97.113(c)." The quoted text is in (a)(3), and the 2015 CFR (govinfo) already numbers it (a)(3). Current (c) is retransmission. The owner may have looked at (c)/(d) because of that footnote.

**Why (c) and (d) still matter on a dual-license spacecraft.** **[judgment]** The amateur transponder may auto-retransmit amateur signals (97.207(d), 97.113(d)), but the Part 97 station must not retransmit signals "emanating from any type of radio station other than an amateur station" (97.113(c)). That means no relaying of the Part 5 radio's traffic, a NASA or university station, or customer ground stations through the amateur transponder or beacon.

**Exceptions for teachers or education.** Only 97.113(a)(3)(iii) mentions education, and it covers compensation incident to a teaching position when a teacher uses the station in classroom instruction. AMSAT calls this the "educational exemption" and says the FCC has accepted experiments whose purpose is student education or public interest (Virginia Tech imagers on Fox-1Cliff and Fox-1D; Kursk State University experiment on ARISSat-1), while experiments that could produce financial gain were not accepted **[secondary: AMSAT licensing paper, 2018]**. The Cal Poly workshop slides from AMSAT's VP Engineering (2019) say a university mission should tie design, construction, testing and operation to credit-granting classes and warn they are "not intended to represent the FCC" **[secondary]**. FCC staff have applied "on behalf of an employer" without regard to on- or off-duty status (WTB Mobility Division letter, 2009-07-16, as reported by ARRL **[secondary]**).

**Related primary text.** 97.1(a) calls the service a "voluntary noncommercial communication service." ITU RR 1.56 (amateur service) is "solely with a personal aim and without pecuniary interest."

### Q2. DA 13-445 and newer guidance

**DA 13-445** (Public Notice, *Guidance on Obtaining Licenses for Small Satellites*, released 2013-03-15, 28 FCC Rcd 2555, Engineering & Technology Bureau contacts). Key wording (U.S. Government work):

- Scope: "non-Federal small satellites, including satellites that fall within the categories of pico-satellites, nano-satellites and cubesats."
- Three regimes: "The Commission's Part 25 rules are the primary vehicle for satellite licensing... The Commission's Part 5 rules cover experimental operations. The Commission's Part 97 rules cover amateur radio service satellite operations."
- On government contracts: "Currently, many small satellite missions involve experimental operations – i.e. scientific and research missions, including missions conducted under government contract – and many operate in amateur frequency bands. These satellites are therefore licensed under Parts 5 or 97 of our rules."
- Eligibility: "Amateur radio transmissions are primarily for the purpose of exchanging messages with other amateur stations, and our rules prohibit 'communications in which the station licensee or control operator has a pecuniary interest, including communications on behalf of an employer....' For experimental licenses, the scope of permitted services includes experimentation under contractual agreement with the United States Government, and communications essential to a research project."
- Who applies: amateur = "the amateur satellite control operator. This operator should have the ability to remove the satellite from a launch manifest and disable space station operations." Experimental = "the party that ultimately controls decisions about the satellite's mission objectives, design, construction, tendering of the satellite to a launch service provider or designated launch integrator, and operations of the satellite once on orbit. This is in most cases a university or research institution, but may also be a commercial venture seeking to test equipment for developmental purposes."
- Frequencies: for experimental, "no specific bands identified in the rules," operations "on a temporary, non-interference basis." Items (h) and (i) of the experimental package: "If the satellite will operate in bands allocated to the amateur satellite service, but will provide no amateur service, the information required in connection with an amateur pre-launch notification and the results of any notification to or coordination with the IARU."
- Federal coordination: "OET will determine whether coordination with the National Telecommunication and Information Administration is necessary." Federal bands cited: 420-450 MHz, 902-928 MHz, 2.390-2.395 GHz, 2417-2450 [sic] GHz.
- License term: experimental two or five years; STA six months; amateur operation lasts "only so long as the operator's amateur license remains current. For communications associated with new launches/missions, the licensee must obtain a new license."

**Newer items that bear on D2 (none withdraws DA 13-445):**

| Date | Item | Relevance |
|---|---|---|
| 2018-04-12 | Enforcement Advisory 2018-01 (DA 18-368) | Cites DA 13-445 as current guidance; warns against launching without authorization; operator must retain a legal right and capability to remove the satellite from the launch vehicle |
| 2019-08-01/02 | *Streamlining Licensing Procedures for Small Satellites*, FCC 19-81 (IB Docket 18-86), 34 FCC Rcd 13077; rules effective 2020-08-19; fee category added; erratum DA 20-736 | New optional Part 25 track. ¶13: existing Part 25, Part 5 and Part 97 processes "remain available"; ¶119: declines to change Part 97/Part 5 rules despite AMSAT/ARRL/ORI comments. ¶11: experimental licensing "is limited to non-commercial uses"; footnote 30: Part 5 licensees "generally cannot market devices or services to customers." I read the circulation draft (FCC-CIRC1908-04, DOC-358437A1), not the adopted PDF, so paragraph numbers may shift. |
| 2020-07-20 | Part 25 small satellite / small spacecraft rules (25.122, 25.123); amended 89 FR 65217 (2024-08-09) | Commercial or non-commercial small-satellite alternative; 25.122(c): 10 or fewer, 6-year, 600 km, 180 kg, 10 cm |
| 2023 | Space Bureau created; 97.207(g) now names it | Replace "International Bureau" in DA 13-445 |
| 2024-09-10, 2024-09-27, 2024-10-03 | Space Bureau/OET Part 5 public notice (DOC-405337), Part 5 page, Part 5 space station checklist | "Not permitted to provide commercial service, charge fees, or receive payments for products or services of operation"; ITU filing and ODM plan before grant; ITU filing goes to OET; IARU letter "needed only if the satellite will operate using frequency bands allocated for amateur use" |
| 2026-07-22/23 | *Space Modernization for the 21st Century*, FCC 26-47 (SB Docket 25-306), Report and Order and FNPRM | New Part 100 replaces Part 25. **§100.5(f): "Space and earth stations in the Amateur Satellite Service are licensed under part 97"; §100.5(b): Experimental Radio Service "may be subject to licensing under part 5."** ¶222-223: FCC will submit ITU filings for Part 5 applications during pendency (with NTIA no-objection letter for NTIA-coordinated bands); applicant remains responsible for ITU cost recovery. ¶294: 5.64(b) debris rules replaced by cross-reference to Part 100. FNPRM ¶347-352: proposes unified space-experimental process, asks whether experiments under U.S. Government contract should be an explicit category and proposes experimental-to-Part 100 conversion. I did not determine the FNPRM comment deadline. |
| 2026-10-07 | DA 26-1080 | Part 100 rules "not yet effective"; Space Bureau will announce the effective date; current Part 25 rules control until then |

Conclusion for Q2: as of 2026-10-15 the 2013 PN, Part 97, Part 5 (5.3(c)) and Part 25 small-satellite rules are the operative framework. DA 13-445 classes government-contract missions as experimental operations and notes that Part 5 scope covers them; it does not itself exclude them from Part 97. The Part 97 bar comes from 97.113(a)(2)-(3). No FCC text I found bars a Part 97 mission from receiving unrestricted funding.

### Q3. Dual-service spacecraft: precedents, FCC handling, IARU

**Rules.** I found no rule that forbids two FCC authorizations on one spacecraft on separate radios. Part 97 is licensed per amateur station and Part 5 per project (5.54: "Widely divergent and unrelated experiments must be conducted under separate licenses"). FCC-side contacts: Space Bureau, OET (Part 5).

**Precedent 1: HuskySat-1 (University of Washington, with AMSAT).** Launched 2019-11-02 (Cygnus NG-12 to ISS), deployed 2020-01-31 **[secondary]**.
- IARU record serial 604 (primary IARU status page hosted by AMSAT-UK): responsible operator Jeffrey Chrisope KI7ORW, Husky Satellite Lab, applied 2018-03-07, coordination completed 2018-04-09; telemetry downlinks 435.800 MHz and 24049.00 MHz, V/U transponder uplink 145.910-145.940 MHz and downlink 435.840-435.810 MHz; launch listed as ELaNa XXIV.
- ARRL (2019-11-13) reports AMSAT's VP-Engineering: the Part 97 license "does not include or allow the use of any of the experiments on board"; UW obtained "a Part 5 Experimental license to cover the telemetry downlink of the AMSAT transponder module, but the transponder must remain off during that operation"; two licenses were needed because the experiments (including the K-band radio) could not conform to Part 97's educational exemption; delays on a single Part 97 license ran into the launch deadline. He also said the effort "developed an understanding with FCC as to how a mission such as HuskySat-1 could be fully licensed under Part 97" **[secondary]**.
- Distinguishing facts for SMART-QSO: UW and AMSAT were different entities; the two services were time-sliced; the Part 5 radios used amateur-band frequencies coordinated by IARU. This does not show that a single individual may hold both authorizations on the same craft.

**Precedent 2: AMSAT Fox-1 series (single Part 97 license, university experiments).** Student or university experiments (Vanderbilt radiation, Penn State gyro, Virginia Tech imagers) flew within a Part 97 mission launched through NASA's CubeSat Launch Initiative **[secondary: AMSAT/ARRL pages]**. This is the pattern in option (c), where experiments are justified as education or public interest.

**IARU position (primary IARU documents).**
- *Short Info Paper* (undated, hosted 2019/12): "Combined missions: A combined satellite mission has an amateur payload and a separate payload that belongs to a different service. Both payloads, including for tracking, telemetry and command for the operation of the satellite (TT&C) shall use frequencies allocated to their own specific service. It is not permitted to operate an amateur-satellite system in combination with other telecommunication services." It also classes missions as Educational/amateur, Experimental/research (generally in space operation, space research and EESS bands), or Commercial (the service-specific band).
- *News release* (2017-06-30; guidelines effective 2017-08-01; page updated 2020-01-30): strong preference that satellites using amateur spectrum operate under amateur licenses; "will only coordinate a non-amateur satellite if an administration directs in writing that it be operated in an amateur-satellite band under an experimental or other non-amateur license"; "Satellites with combined amateur and non-amateur missions will continue to be coordinated"; experimental satellites cannot be coordinated in 144-146 MHz since 2014-07-01. It cites WRC-15 Resolution 659, which notes non-amateur use of 144-146 and 435-438 MHz is not in accordance with the amateur-satellite service definition.
- *Developers' paper* rev 15.5 (2006-10-06; old): for a multi-service satellite, all operation on amateur-satellite frequencies must meet the amateur requirements, and amateur-band uplinks should not be used for routine TC/TM of the other-service portion. It also notes alternative services allow payments to participants and ciphering, with no open-access requirement.
- *Coordination database examples:* NILA CubeSat (HEX20 India, 3U; coordination request dated 2025-01-22, declined 2025-01-27): "IARU is not in a position to assign frequencies for commercial spacecraft and/or satellites with a scientific payload. IARU opposes the use of spectrum allocated to the amateur satellite service by satellites that have no amateur payload." Solaras S3 (3U tech demo; request 2025-11-10, coordination incomplete as of 2025-11-10) claims it keeps "amateur and other mission subsystems separate." VIOLET (CubeSat NB, Canadian Space Agency program; coordinated 2021-03-09) is a student mission with amateur use.
- *FCC link:* DA 13-445 item (h) and the Part 5 checklist require an IARU letter if a Part 5 satellite uses amateur-allocated bands.

**Judgment on the SMART-QSO design.** A shared bus with one Jetson and one power system serving both stations will strain the "own TT&C per payload" rule. A defensible architecture gives each radio its own telecommand authority and its own independent hardware transmit-off, so the Part 5 operator cannot silence the amateur radio and vice versa. Shared compute and power resources are scheduled by the onboard power agent rather than by amateur uplink command. **[judgment]**

### Q4. Part 5 spectrum, NTIA, ITU, timelines, fees

**What is actually obtainable for a non-Federal experimental 1U** (primary sources: 47 CFR 2.106 footnotes, retrieved 2026-10-15; NASA Spectrum Guidance Table 6/7, 2015-08-27, a NASA document that expired by its own terms 2017-08-26 and that I could not find replaced; the 2025 NASA SOA chapter gives only a summary):

| Band | Direction | Status for non-Federal Part 5 | Source / comment |
|---|---|---|---|
| 144-146 MHz | | Do not plan on it | IARU stopped coordinating experimental satellites there 2014-07-01 |
| 435-438 MHz | | Possible but the IARU problem applies | Amateur-satellite secondary (RR 5.282). WRC-15 Res. 659 says non-amateur use is not in accordance with the service. Part 5 use needs an IARU letter, and IARU coordinates only on written administration direction |
| 137-138 MHz / 148-149.9 MHz | down / up | Alternative | ITU Resolution 32 (WRC-19): short-duration missions (3 years max, 10 satellites max) may file API only in space operation bands; 137.025-138 and 148-149.9 MHz named; then notification within 2 months of bringing into use |
| 400.15-401 MHz | down | Most practical UHF downlink | Met-sat/space research/space operation (space-to-Earth); mobile-satellite use limited to non-voice NGSO (US320); RR 5.208A/B protect radio astronomy. NASA 2015 table: Gov and non-Gov |
| 401-402 / 402-403 MHz | | Possible | Met aids, EESS/Met-sat Earth-to-space (DCPs), space operation 401-402. NASA 2015 table: Gov and non-Gov. I did not confirm the current Table row text |
| 449.75-450.25 MHz | up | Most practical UHF uplink | US87: "Federal and non-Federal stations for space telecommand (Earth-to-space) at specific locations, subject to such conditions as may be applied on a case-by-case basis." RR 5.286 requires agreement under RR 9.21, so formal ITU coordination applies (NASA 2015 §4.5) |
| 902-928 MHz | | Experimental only | No allocation for space service; DA 13-445 lists it as commonly authorized and Federal-shared |
| 2025-2110 MHz | up | Restricted | US347: non-Federal Earth-to-space in space research / EESS "case-by-case"; US94: non-Federal space operation limited to pre-launch/launch telecommand with NTIA coordination per launch |
| 2200-2290 MHz | down | Very restricted | US96: non-Federal "restricted to use for pre-launch testing and space launch operations," secondary, NTIA coordination per launch; US303 lets non-Federal stations transmit to TDRSS in 2285-2290 MHz only. NASA 2015 table lists it "Gov Only" |
| 2400-2450 MHz | | Crowded | ISM plus amateur-satellite secondary; commonly authorized to small sats; unprotected |

Part 5 does allow "any Federal or non-Federal frequency designated in the Table... provided that the need for the frequency requested is fully justified" (5.85(a)(1)), unprotected and non-interference (5.84). **[judgment]** So S-band (2025-2110 / 2200-2290) is obtainable for a non-Federal licensee only through FCC discretion plus NTIA concurrence, and realistically with a NASA sponsor behind it. The practical non-amateur UHF pair is 400.15-401 MHz down and 449.75-450.25 MHz up. A 1U CubeSat with a deployable-wing power system and modest data volume fits a UHF link; S-band adds pointing and power demands.

**NTIA coordination when NASA is a party** (NASA Spectrum Guidance 2015 §4.2-4.4, Table 5, and NTIA Manual 8.2.17 as quoted there):
- *Non-Federal system (NASA only funds or buys):* FCC licenses; FCC coordinates with NTIA if bands are Federal or shared; NASA spectrum managers "can only speak on behalf of NASA and cannot directly represent non-federal (non-NASA) systems." NASA requires its contractors and grantees to obtain FCC licenses (NPR 2570 §3.5 as quoted). NASA says FCC "has increasingly inquired" about ownership and operation, "particularly where the mission is closely affiliated with a government entity."
- *Federal system (NASA owns it and operates it, or exercises effective control by contract):* NTIA authorizes through the NASA Center spectrum manager (Stage 1/2/3/4 certifications), ITU filing and fees handled by NASA HQ, and the command link must be on a protected primary allocation (NTIA 8.2.53). The FCC Part 5 page says that if a federal agency operates the station, "applications should be filed with your federal sponsor through the NTIA, not the FCC."
- The 2015 guidance notes that NASA-supported non-Federal systems are "typically not eligible to file as Amateur Radio Satellite systems." That statement is dated (the document expired 2017-08-26) and is contradicted by Fox-1 and HuskySat-1 experience, but it shows how NASA spectrum staff think.

**ITU responsibility.** The notifying administration is the FCC. The applicant prepares SpaceCap (API/CR) and GIMS files, SpaceVal output, an FCC cover letter and a cost-recovery letter on letterhead (university letterhead for educational institutions); Part 5 ITU filings go to OET, and the filing is not sent to the ITU until FCC and NTIA coordination is done (Part 5 checklist, updated 2024-10-03). Under FCC 26-47 ¶222, FCC will instead submit the filing during pendency (with an NTIA no-objection letter where NTIA coordination is required), but Part 100 is not yet effective. The ITU filing obligation applies below 300 GHz regardless of satellite size (NASA 2015 §4.5). RR 9.1 allows API 7 years to ideally 2 years before bringing into use. NASA notes the process "could take much longer" than planned for small satellites and that missions without a completed filing operate on a non-interference basis.

**Fees.**
- ITU cost recovery (Council Decision 482, version modified 2013, effective 2013-07-01): advance publication of a non-GSO network not subject to coordination CHF 570; notification of a non-GSO network not subject to coordination CHF 7,030. The same two figures appear in NASA's 2015 guidance. The current version (modified 2024) was not retrieved, so these may have changed. ¶11 exempts "publication of special sections for the amateur-satellite service" (the API publication); it does not expressly exempt notification of amateur space stations. FCC "does not pay these fees" (NASA 2015 §4.5.2).
- FCC: Part 5 $140 (1.1103); Part 25 small satellite $2,555 per call sign; Part 97 notifications: no fee found.

**Timelines.** FCC Part 5: 45-180 days (NASA 2015); about 98-day average (2025 academic analysis **[secondary]**); NTIA time unknown. ITU: months to years for full coordination; notification for short missions may follow launch. Launch providers require authorization before integration; NASA CSLI documents say the mission "must obtain frequency licensing/authorization prior to integration on the launch vehicle."

### Q5. NASA CubeSat Launch Initiative

- **Agreement type:** "a non-reimbursable Cooperative Research and Development Agreement" for entities external to NASA, with standard terms for all CSLI collaborators (NASA CSLI Announcement of Partnership Opportunity, page last updated 2026-10-14). The 2025 template is a CRADA under the Federal Technology Transfer Act, 15 U.S.C. §3710a. Article 5.0: "There will be no transfer of funds between the Parties under this Agreement and each Party will fund its own participation." NASA says it "does not provide funding for the development of the small satellites." This is a CRADA, not a cooperative agreement (grant-type), Space Act Agreement, or procurement.
- **Eligibility:** "eligible U.S. educational institutions, nonprofit organizations, and NASA centers." Individuals and for-profit entities are not listed. The current page says the next solicitation is anticipated in 2028 with eligibility to be published then, and 228 missions have been selected as of September 2026.
- **Obligations that matter to Part 97:**
  - Art. 10.1: the Collaborating Party "shall obtain radio frequency authorization from the Federal Communications Commission (FCC)." SOW A.3 requires the application, the license once obtained, and, "if amateur satellite service frequencies are employed," the IARU coordination letter. (NASA expects Part 97 and Part 5 missions.)
  - Art. 2.5 requires the Collaborating Party to "share raw data that it obtains" and deliver a final report to NASA.
  - Art. 9.1: Government Purpose Rights in data first produced under the CRADA; NASA may publish unclassified, non-Proprietary data (51 U.S.C. §20112).
  - Art. 12: no use of NASA name implying endorsement.
- **Does CSLI launch alone affect Part 97 eligibility?** No rule text links launch sponsorship to eligibility. Part 97 eligibility turns on the licensee and control operators, pecuniary interest, and the nature of the communications. **[judgment]** A no-funds CRADA that provides launch in exchange for sharing data and a report should not create a pecuniary interest in amateur communications, and sharing amateur-link data publicly is consistent with open access. Precedent: AMSAT's Fox-1 series flew via CSLI/ELaNa with a Part 97 mission **[secondary]**, and HuskySat-1 carried a Part 97 transponder on a NASA-sponsored launch (planned as ELaNa XXIV per the IARU record; it flew on Cygnus NG-12 to the ISS). The counterweight is NASA's own 2015 statement about NASA-supported non-Federal missions, and AMSAT's HuskySat-1 comment that the "pecuniary interest" issue complicated even the Part 97 half. Two practical blockers: the licensee needs a host institution or nonprofit (repo D1), and the 2028 solicitation date versus the baseline's 2028 launch plan.

### Q6. Options with consequences, and contract structures

**Option (a): dual license.** Needs a Part 5 licensee for the contract link, ideally the sponsor or a research partner; a second transmitter and antenna (ROM in 4.2); IARU coordination as a combined mission; FCC, possibly NTIA, and ITU filings for the Part 5 side; an architecture that separates TT&C, data and control. Consequences: cost and schedule up, power margin down, and a chance that FCC or IARU rejects the hybrid. Benefit: keeps the public transponder and enables paid work.

**Option (b): all Part 5 (or Part 25).** Under Part 5 there is no public transponder (5.125), 144-146 MHz is closed to experimental coordination, and nonamateur use of 435-438 MHz needs a written administration direction. Part 25 small-satellite licensing is the correct route for selling data as a service. Consequences: the amateur mission is gone; the Part 97 control-operator and open-data duties disappear; fees and protection differ (Part 25 gives interference protection only where the band allocation supports it).

**Option (c): stay Part 97; grants and donations only.** Lowest regulatory cost and risk. Consequences: no payments for deliverables, no operations compensation, no proprietary data, no sponsor control.

**Contract structure vs. a clean amateur station [judgment; the FCC has not published a grant/contract test]**

| Structure | Compatible with a clean Part 97 station? | Reason |
|---|---|---|
| Unrestricted donation or gift | Yes | Nothing required in return |
| Grant for the build or education with nothing delivered over the amateur link (grant/assistance in the sense of 31 U.S.C. §6304; not retrieved) | Likely yes | Money not tied to communications; keep deliverables off the amateur link |
| In-kind launch via CSLI CRADA (no funds exchange) | Likely yes | No pecuniary payment; data sharing consistent with openness |
| Educational grant with credit-bearing course tied to the mission | Likely yes | Matches the "education" reading AMSAT reports; AMSAT slides are not FCC |
| Procurement contract (SBIR/STTR, purchase order, service agreement) with data deliverables | No | Payment for services or data derived from amateur communications; 97.113(a)(2),(3) |
| Any payment to the licensee or control operators for operating the station | No | 97.113(a)(2) "direct or indirect" |
| Exclusive or proprietary data rights | No on the amateur link | Conflicts with (a)(4) obscuring and open access; fine for the Part 5 link |
| Contract clause giving the sponsor authority to direct or cease transmissions | No | Makes the station Federal (NTIA Manual 8.2.17) |
| Contract for deliverables carried only on a Part 5 link held by a separate licensee | Yes for the Part 97 side, if separation is real | HuskySat-1 pattern |

**Data-rights guidance [judgment].** Under the Part 5 side, negotiate sponsor-specific data rights freely. Under Part 97, all data on the amateur link must remain public and decodable by amateurs; do not grant exclusive, delayed or proprietary rights in anything transmitted over it. Because Government Purpose Rights apply to CSLI data, record in the CRADA statement of work that amateur-link data is public domain in practice.

---

## 7. References

**Primary rule and agency texts (U.S.)**
- 47 CFR 97.1, 97.3, 97.111, 97.113, 97.119, 97.207, 97.209, 97.211 (eCFR, version 2026-10-01, retrieved 2026-10-15). `raw/ecfr_97.*.xml`.
- 47 CFR Part 5 (eCFR, same version): §§5.3, 5.5, 5.51, 5.54, 5.63, 5.64, 5.71, 5.84, 5.85, 5.111, 5.125. `raw/ecfr_part5_clean.txt`.
- 47 CFR 25.122, 25.123 (eCFR). 47 CFR 1.1103 (Part 5 fees; 90 FR 17013, 2025-04-23), 1.1107 (Table 9 space station fees). 47 CFR 2.106 footnotes 5.208A/B, 5.282, 5.286, US87, US94, US96, US303, US319, US320, US347.
- FCC, *Guidance on Obtaining Licenses for Small Satellites*, DA 13-445, 28 FCC Rcd 2555 (2013-03-15). https://docs.fcc.gov/public/attachments/DA-13-445A1_Rcd.pdf
- FCC Enforcement Advisory 2018-01, DA 18-368 (2018-04-12). https://docs.fcc.gov/public/attachments/DA-18-368A1.pdf
- FCC, *Streamlining Licensing Procedures for Small Satellites*, FCC 19-81, 34 FCC Rcd 13077 (adopted 2019-08-01, released 2019-08-02); circulation draft FCC-CIRC1908-04 at https://docs.fcc.gov/public/attachments/DOC-358437A1.pdf
- FCC Space Bureau, public notice on Part 5 open house (2024-09-10). https://docs.fcc.gov/public/attachments/DOC-405337A1.pdf
- FCC Space Bureau, Part 5 Experimental Licensing page (updated 2024-09-27) and Part 5 Experimental and STA Space Station Application Checklist (updated 2024-10-03), read via Wayback copies dated 2025-11-30. fcc.gov/space/part-5-experimental-licensing
- FCC 26-47, *Space Modernization for the 21st Century* (adopted 2026-07-22, released 2026-07-23). https://docs.fcc.gov/public/attachments/FCC-26-47A1.pdf
- FCC Space Bureau, DA 26-1080 (2026-10-07). https://docs.fcc.gov/public/attachments/DA-26-1080A1.pdf

**NASA**
- NASA Spectrum Management Program, *Spectrum Guidance for NASA Small Satellite Missions*, v1.0 (2015-08-27; expiration 2017-08-26). https://www.nasa.gov/wp-content/uploads/2023/09/spectrum-guidance-smallsats-cubesats-2015.pdf
- NASA CSLI *Announcement of Partnership Opportunity* (updated 2026-10-14). nasa.gov/kennedy/launch-services-program/cubesat-launch-initiative/
- NASA CSLI nonreimbursable CRADA template (2025). https://www.nasa.gov/wp-content/uploads/2024/08/crada-cubesat-template-final-2025-website.pdf
- NASA, *State-of-the-Art of Small Spacecraft Technology*, ch. 9 Communications (2025 ed., hosted 2026-05). https://www.nasa.gov/wp-content/uploads/2026/05/9-soa-comm-2025-final.pdf

**IARU and ITU**
- IARU, *Amateur Satellites Short Info Paper* (hosted 2019/12, undated). https://www.iaru.org/wp-content/uploads/2019/12/short_info_paper.pdf
- IARU, *Satellite Coordination News Release* (2017-06-30; updated 2020-01-30). https://www.iaru.org/satellite-coordination-news-release/
- IARU, *Amateur Radio Satellites: Information for Developers*, rev 15.5 (2006-10-06). https://www.iaru.org/wp-content/uploads/2019/12/iarusatspec_rev15.7.pdf
- IARU coordination records (hosted by AMSAT-UK): HuskySat-1 (serial 604), NILA CubeSat (1016), Solaras S3 (1075), VIOLET (754). https://iaru.amsat-uk.org/formal_detail.php?serialnum=604
- ITU Council Decision 482 (modified 2013). https://itu.int/en/ITU-R/space/Documents/Decision%20482(C2013)-E.pdf
- ITU, Resolution 32 (WRC-19) page. https://itu.int/en/ITU-R/space/support/nonGSO/RES32/Pages/default.aspx

**Secondary (amateur community and trade)**
- ARRL, "AMSAT Says HuskySat-1 Paving the Way for Further Cooperation" (2019-11-13).
- AMSAT, *FCC Part 97 Amateur Radio Licensing for CubeSats* (2018-04) and J. Buxton, *Amateur Radio and Experimental Licensing for CubeSats*, Cal Poly workshop (2019).
- ARRL, "It Seems to Us: Pecuniary Interest" (2009-09-01).
- Vendor price pages (satsearch, EnduroSat, as reported by web search; not opened directly).

---

## 8. Gaps and Items Not Verified

1. **FCC Part 5 FAQ** and the FCC Space Bureau Part 5 and FAQ pages returned 403 to direct fetch. The Part 5 page and checklist were read from Wayback copies dated 2025-11-30.
2. **IARU current guidelines page** (`iaru.org/on-the-air/amateur-satellite-service/`) returned 404. I relied on the 2017 news release, the Short Info Paper, and the coordination database. The Developers' paper is rev 15.5 (2006) and may be superseded.
3. **ITU Decision 482 current text (modified 2024)** and current CHF fees were not retrieved. The 2013 figures (570 / 7,030 CHF) are stale in an unknown direction. The ¶11 amateur exemption wording is also from 2013.
4. **No FCC statement** found on one individual holding both a Part 97 station and a Part 5 station on the same spacecraft. HuskySat-1 split them across two entities.
5. **FCC's treatment of grants vs. procurement contracts** under 97.113 is not published; the table in Q6 is my inference. The Federal Grant and Cooperative Agreement Act reference (31 U.S.C. §§6301-6308) is from memory and not retrieved.
6. **Fox-1 / HuskySat-1 details** (Fox-1 CSLI launch dates and FCC licensing mechanics) come from AMSAT, ARRL and search summaries, not FCC records. I did not locate the FCC ELS file for UW's experimental license.
7. **FCC 19-81 adopted text**: I read the circulation draft. Paragraph numbers may differ.
8. **FCC 26-47 FNPRM** comment deadline and whether any Part 100 provisions touch Part 5 beyond ¶¶222, 294 and 347-352 were not fully reviewed. Part 100 is not yet effective.
9. **Current US Table (2.106) rows** for 400.15-402 MHz and 449.75-450.25 MHz: I confirmed the footnotes (US87, US319, US320, 5.286) but used NASA's 2015 table for the Federal/non-Federal row status.
10. **NTIA coordination timelines** and the NTIA Manual (Red Book) text were not retrieved; NASA's summary of 8.2.17 and 8.2.53 is the only source.
11. **Part 25 annual regulatory fees** for small satellites were not retrieved.
12. **Vendor prices** are single data points from web-search summaries and need quotes.
