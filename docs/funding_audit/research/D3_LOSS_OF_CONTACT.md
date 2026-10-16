# D3 research: "loss of command contact" timers for amateur-band CubeSats

> Research input to decision D3, prepared 2026-10-15 and imported unchanged into the repository. Where it mentions working files (raw downloads, extracted texts, scripts) or says no repository files were modified, it refers to the research session; those files were not kept. How the decision was taken and applied is recorded in `docs/BASELINE_PARAMETERS.md` section 9.


Prepared 2026-10-15 for SMART-QSO (1U, US Part 97 licensee). No repository files were modified.
Scope: pick N (days without a valid authenticated telecommand -> inhibit all transmitters; receiver stays on; lifted by next valid command) and M (further days -> autonomous end-of-mission passivation: charge path permanently disconnected, transmitters off).

Evidence labels used below:
- [R] = I read the primary text itself (downloaded or fetched raw).
- [S] = read through a page-fetch summarizer; wording not independently verified against raw text.
- Anything marked "judgment" is my inference, not a sourced fact.

---------------------------------------------------------------------------

## 1. Executive summary

1. There is no regulator-prescribed value for either N or M. Neither the FCC (Part 97, Part 5, Part 25, DA 13-445, the 2019 small-sat order, the 5-year disposal rule) nor the IARU prescribes a number of days for a command-loss timer. The FCC has acknowledged the concept in writing (FCC 19-81, para. 62-63 and fn. 182) and expressly declined to prescribe a design or value.
2. The public record of what other missions chose is very thin, and this is structural:
   - IARU coordination form v40 puts the whole telecommand section, including Q7.9 ("mechanism to ensure the space station is turned off in the event communications is lost with the command station"), under a heading that says "NOT published". Other missions' Q7.9 answers are confidential by design. The "IARU form" column of any table will be empty for everyone.
   - Part 97 pre-space notifications are letters to the Space Bureau; I could not find them indexed anywhere I could query. The Part 5 experimental-license ODARs I did pull (6 of them) are the wrong exhibit type and contain no command-loss language.
3. What I could confirm with a quotable source: exactly two designs in the 70 cm amateur band with a 14-day command-loss rule (TechEdSat-1, 2012: beacon stops after 2 weeks without commands; Cactus-1, 2021: inverse design, silent unless commanded, 1-day beacon after 14 days without commands). Licence status caveat: I found no Part 97-licensed mission with a command-loss timer. Both 14-day precedents appear to be Part 5 experimental stations that use amateur frequencies (Cactus-1's licensed 434.03 MHz is outside the 435-438 MHz segment that 97.207(c) gives space stations; TechEdSat's ODAR is filed in the FCC experimental-licensing system; neither licence was read, so this is inference). One codified FCC timer exists outside amateur service: 47 CFR 25.259(c), 72 consecutive hours without a verified reset signal -> automatic turn-off (commercial NVNG/NOAA time-sharing). Everything else is either a non-RF timer (12 h reboot, 30 d ADCS) or "no command-loss timer documented".
4. For M I found no precedent at all: no amateur mission, FCC filing or IARU document I could read describes autonomous irreversible passivation after command loss, and no regulator expects one. The strongest evidence points the other way (spacecraft recovered after months to 949 days of silence).
5. Recommendation (judgment, section 4): N = 30 days as the launch default, reduced by telecommand to 14 days once commissioning is declared complete (14 days is the only value seen twice in amateur-band practice); M = 180 days, with the whole M mechanism taken to the D3 FCC Space Bureau pre-filing consultation as a question rather than a settled design. Details and safeguards in section 4.

---------------------------------------------------------------------------

## 2. Table of missions and rules

"Source type" distinguishes IARU form / FCC filing / paper / mission page / rule text. No row below is an IARU form answer to Q7.9 or an FCC pre-space notification, because I could not find any (see section 1, item 2). Absence of a timer in the "none documented" rows means only that public sources do not describe one.

### 2a. Rows with a command-loss timer (value confirmed)

| # | Mission / rule | Timer | What happens at expiry | Source type | Source URL (date) | Ev. |
|---|---|---|---|---|---|---|
| 1 | TechEdSat-1 (NASA Ames; 437.465 MHz 1 W StenSat beacon in the amateur band; commanding via Orbcomm/Iridium modems; licence type unverified, apparently Part 5) | 2 weeks (14 d) without commands | Watchdog timer stops the beacon | Mission page (eoPortal), secondary; not an IARU/FCC document | https://www.eoportal.org/satellite-missions/techedsat (page "Last updated: Oct 15, 2012") | R |
| 2 | Cactus-1 (Capitol Technology University, 3U, launched Jan 2021; licensed 434.03 MHz, outside 97.207(c)'s 435-438 MHz space-station segment, so apparently Part 5, unverified) | 14 d without commands | Inverse pattern: spacecraft is silent unless commanded (designed that way for FCC shut-down compliance, authors say they were deliberately conservative); after 14 d with no commands it beacons for 1 day, and repeats every 14 d | University blog (mission team) | https://www.captechu.edu/blog/cactus-1-cubesats-next-steps (2 Mar 2021) | R |
| 3 | CuPID CubeSat (400.5 MHz space-research band; NOT amateur, FCC licence did not permit beaconing) | 12 h without ground commands; separately a 6 h radio reset | Whole-spacecraft reboot via EPS reset (not a transmitter inhibit) | Paper (arXiv) | https://arxiv.org/abs/2304.04702 (arXiv ID = Apr 2023) | S |
| 4 | 3UCubed (Sonoma State / NASA CSLI; 437.01 MHz downlink) | 30 d without contact | Automatic ADCS commissioning (no RF action) | Mission news page | https://edeon.sonoma.edu/3u3/news/3ucubed-launched/ (page shows 11 Dec 2025) | S |
| 5 | 47 CFR 25.259(c) (NVNG MSS satellites time-sharing 137-138 MHz with NOAA; commercial, Part 25) | 72 consecutive hours with no reset signal from the licensee's gateway earth station, verified by the satellite | Satellite "shall automatically turn off and cease satellite transmissions"; all satellites must also be capable of instantaneous shutdown by command | Rule text (eCFR via API, date 2026-10-01) | https://www.ecfr.gov/current/title-47/section-25.259 | R |

### 2b. Guidance / comment rows that name a timer concept or period

| # | Source | Period | Substance | Source URL (date) | Ev. |
|---|---|---|---|---|---|
| 6 | IARU "Controlling Space Station Transmitters", Rev 27 (IARU Satellite Adviser + Panel) | "About the length of a single pass" (minutes) | For the special case of a LEO station talking to few Earth stations and lacking a separate command receiver: an independent transmitter time-out timer, re-enable only by positive ground command; acceptability to be confirmed with the national administration | https://www.iaru.org/wp-content/uploads/2019/12/controllingsatellites_v27.pdf (27 Aug 2010) | R |
| 7 | FCC 19-81 fn. 182 (comments quoted by the FCC) | Boeing example: no more than 30 s per transmission; CSSMA example: cease "if and when a ground contact has not been established for a certain period of time" (no number) | FCC recorded both, did not adopt or prescribe | https://docs.fcc.gov/public/attachments/FCC-19-81A1.pdf (adopted 1 Aug 2019) | R |

### 2c. Rows where the loss-of-command behaviour is documented but no days-scale timer is described (design-space datapoints)

| # | Mission | Documented behaviour | Source type / URL (date) | Ev. |
|---|---|---|---|---|
| 8 | ANDE (US Naval Academy Amateur Radio Club; 145.825 MHz digipeater; launch then planned Dec 2003) | An actual (old-format) IARU request: transmitter "always off", receiver wakes about every 20 s, TNC works only on packet activity, transmitter can be commanded off at any time. Q7.9-type question did not exist yet; no loss-of-contact mechanism stated | IARU form, filled in by applicant: https://www.aprs.org/ande/ANDE-IARU.doc (undated; text references Dec 2003 launch) | R |
| 9 | FUNcube-1 / AO-73 | "Autonomous mode": transponder turns on in eclipse and off in sunlight. Power-driven; no command-loss timer described | https://amsat-uk.org/2013/11/23/funcube-1-transponder-in-autonomous-mode/ (23 Nov 2013) | S |
| 10 | Fox-1Cliff (AMSAT) | Startup Beacon/Safe Mode limited to 10 s on / 2 min off until commanded; no command-loss timer described | https://www.arrl.org/news/amsat-s-fox-1cliff-amateur-radio-cubesat-launched-successfully (3-4 Dec 2018) | S |
| 11 | Fox-1A (AO-85) | Ground deliberately disabled all transmitters after low-battery eclipse problems (Dec 2018 onward); periodically re-enabled. Ground decision, not a timer | search-result description of AMSAT/Wikipedia text; page not read; treat as low confidence | S (low) |
| 12 | ARMADILLO (UT Austin 3U, launched 2019) | An onboard reset required a "ground contact condition" that was never met; satellite recovered after 949 days. Timer value not in the abstract | https://digitalcommons.usu.edu/smallsat/2022/all2022/4 (SmallSat 2022, SSC22-VII-04) | S |
| 13 | Two LASP CubeSats (not named) | Lost communication then "came back to life after a few months" | https://lasp.colorado.edu/minxss/2019/02/14/minxss-2-communications-anomaly/ (14 Feb 2019) | S |

Missions I searched and could not confirm any command-loss rule for (left out of the tables on purpose): HuskySat-1, QUBIK, OreSat0, TJREVERB, MinXSS-1/-2, PhoneSat/EDSN, BIRDS series, ESTCube-1, AAUSAT-II, GRBAlpha, Masat-1 (paper blocked, HTTP 403), LituanicaSAT-1, ICube-1, KickSat-2, CSUNSat1, RadFxSat, CHOMPTT, QubScout-S1. The six FCC ELS ODARs I downloaded (TechEdSat, QubScout-S1, KickSat-2, ELaNa-XIV, ELaNa-17, ELaNa-XIX) contain no command-loss language.

---------------------------------------------------------------------------

## 3. Spread

Computed from confirmed values only (judgment: the sample is tiny, and the two 14 d values come from different mechanisms, so the "median" is more a statement of where the evidence clusters than a statistic).

| Subset | Values | Min | Median | Max |
|---|---|---|---|---|
| Timers that end in a transmitter action (rows 1, 2, 5) | 14 d, 14 d, 3 d | 3 d (72 h, commercial rule) | 14 d | 14 d |
| 70 cm amateur-band designs, apparently Part 5 (rows 1, 2); no Part 97-licensed example found | 14 d, 14 d | 14 d | 14 d | 14 d |
| All command-loss timers incl. non-RF (rows 1-5) | 12 h, 72 h, 14 d, 14 d, 30 d | 0.5 d | 14 d | 30 d |
| M (autonomous passivation after command loss) | none found | n/a | n/a | n/a |

Short-period guidance (IARU one-pass timeout; Boeing 30 s) is a different mechanism (a within-contact transmit timer, not a days-scale command-loss timer) and is excluded from the day statistics.

---------------------------------------------------------------------------

## 4. Recommendation (all of this section is judgment, grounded in the evidence above)

### 4.1 Recommended N

- Launch default N = 30 days. Operational value N = 14 days, set by authenticated telecommand once commissioning is declared complete. Settable range stays 1-90 d, but state the nominal value and the software cap in the filings.
- Clock start (t0): the end of the post-deployment RF-silence period, not separation. Each valid authenticated telecommand resets the counter. The inhibit state must be persisted across resets; per IARU Rev 27 sec. III.2 a positive ground command should be required to re-enable, and a power reset after eclipse must not re-enable a transmitter. Receiver stays on (supported by Klofas's CubeSat communications survey: never turn off command receivers).

Rationale:
1. 14 d is the only value that appears twice in 70 cm amateur-band practice (TechEdSat-1, Cactus-1), though both appear to be Part 5 stations and I found no Part 97-licensed precedent. It is the steady-state number I can defend with a citation; for a Part 97 filing it is a design choice to justify, not a precedent to cite.
2. A 14-day default is too tight for launch. Commissioning is planned at about 2 weeks, and real first-contact delays run longer: Cactus-1 reported no contact at 30 days; TJREVERB's team reportedly tried its primary Iridium radio for a week without success, then fell back to an APRS radio and recruited about twenty volunteer amateurs because it lacked an APRS ground station (secondary, from search-result summaries, not read in full). If N fires before first contact, the beacon is silenced exactly when outside observers (SatNOGS, hams) are needed to find the spacecraft. 30 d gives about 2x slack on the nominal plan.
3. Going shorter (3-7 d, closer to the 72 h in 25.259(c)) buys little regulatory value. That 72 h rule belongs to commercial NVNG systems with continuous gateway coverage; it is not transferable to a volunteer-operated mission with probably one designated command station. Because the receiver stays on and the next valid command lifts the inhibit, a false positive is cheap to recover, but it still costs community service and weakens the ability to verify the lift by downlink.
4. Going longer than 30 d at steady state is hard to justify against the only precedent; it also weakens the story for the FCC that an uncontrolled station goes quiet within weeks.
5. If the filings need one immutable number, use 30 d. (Judgment; nothing in the sources discriminates between 14 and 30 on regulatory grounds.)

### 4.2 Recommended M

- M = 180 days after the N inhibit has taken effect (so about 210 days from the last valid command at the launch default of N = 30). Treat M as a proposal to put to the FCC Space Bureau in the D3 pre-filing consultation, not a settled design.
- State machine: NORMAL -> TX_INHIBIT (after N) -> PASSIVATED (after a further M in TX_INHIBIT). Any valid authenticated command during TX_INHIBIT returns to NORMAL and clears both counters. Nothing leaves PASSIVATED.

Rationale:
1. No precedent exists, so there is nothing to anchor a short value. I found no amateur mission, FCC filing or IARU text that describes autonomous irreversible passivation after command loss, and no regulator expectation of one. FCC wording on end-of-life is about stored energy being removed, not about when.
2. The recovery evidence is on months-to-years timescales: ARMADILLO recovered after 949 days; LASP reports two CubeSats that came back after "a few months". Passivation is irreversible and kills a recoverable spacecraft, so M should exceed the longest outage you are willing to bet against. 180 d clears the "few months" cases; it does not clear ARMADILLO-class cases, which is a conscious trade-off against leaving a charged battery on a dead bus (judgment).
3. 25.283(c) (Part 25, not binding on Part 97, but it shows the FCC's approach) already excuses discharge duties where "prevented by technical failures beyond its control". For Part 97, 97.207(g)(1)(iii) asks the pre-space notification to address stored-energy removal, naming "leaving all batteries in a permanent discharge state" or other equivalent procedures that are specifically disclosed. A passivation timer is one way to make that disclosure concrete; it is not mandated.
4. Safeguards I would require before shipping an irreversible timer (judgment): counters kept in redundant, monotonic non-volatile storage and independent of the RTC; plausibility check against telemetry (for example require repeated confirmation across N+M, not a single counter read) so one SEU cannot passivate a healthy bus; no autonomous passivation in the first year after launch (command-initiated passivation always allowed). Layering: the first-year lockout dominates, so autonomous passivation cannot occur before about day 365 regardless of M; M governs after that point.

### 4.3 Questions worth putting to the FCC Space Bureau (D3 consultation)

- Is a command-loss transmitter inhibit (N) something the FCC wants described in the 97.207(g) pre-space notification, and does it expect a maximum value?
- Does the FCC see autonomous passivation (M) as desirable, neutral, or a risk to the licensee's ability to meet a later FCC cessation order? (97.207(b) requires only the capability to cease by telecommand when ordered; a passivated spacecraft cannot be ordered, but it is also silent.)
- Does a reversible N inhibit count as "termination of the space station transmissions" for the 97.207(g)(3) post-space notification (3 months), or only a permanent cessation such as M passivation?

---------------------------------------------------------------------------

## 5. What the FCC and IARU have said explicitly

All [R] unless noted. Quotation kept to rule text and short phrases.

FCC
- 47 CFR 97.207(b): a space station "must be capable of effecting a cessation of transmissions by telecommand whenever such cessation is ordered by the FCC." No timer, no loss-of-contact language. (eCFR text dated 2026-10-01; amendment list includes 89 FR 65223, 9 Aug 2024.)
- 97.207(g): pre-space notification due within 30 days of launch-vehicle determination and no later than 90 days before integration; in-space notice within 7 days of starting transmissions; post-space notice within 3 months of termination of transmissions (24 hours if the FCC ordered the termination). Autonomous M passivation would be a real termination of transmissions and would start the 3-month post-space clock (judgment). Whether a reversible N inhibit counts as "termination" is unclear; I read it as not terminating, but this is a question for the Space Bureau (see 4.3), since it could otherwise force spurious filings.
- 97.207(g)(1)(vii)(D)(1): for LEO, successful disposal means re-entry "as soon as practicable, but no later than five years" after mission completion.
- 25.283(e): same five-year rule for Part 25; "end of mission" is defined by capability (no longer able to maneuver; for spacecraft without that capability, when the primary mission is complete), not by contact status. (Content of the 2022 Second Report and Order, FCC 22-74, as codified; this is what the brief called the 2024 Space Innovation rules. I did not read FCC 22-74 itself.)
- 25.283(c): stored energy sources discharged "unless prevented by technical failures beyond its control" (Part 25).
- 97.213(b): terrestrial telecommand stations must limit transmission to 3 minutes on control-link failure. This applies only to stations "on or within 50 km of the Earth's surface", so it does not apply to a space station; it does show the FCC's philosophy for a remotely controlled amateur station.
- 25.207 (Part 25): space stations "shall be made capable of ceasing radio emissions by the use of appropriate devices (battery life, timing devices, ground command, etc.) that will ensure definite cessation of emissions." Timing devices are explicitly acceptable.
- 25.259(c): the only codified command-loss timer found, 72 h (row 5).
- FCC 19-81 (Streamlining Licensing Procedures for Small Satellites, 2 Aug 2019), paras. 61-64 and fn. 182: requires the capability for immediate cessation by telecommand; records commenters' arguments that onboard timers or ground-contact-loss shutdown are acceptable alternatives; "we will not prescribe specific designs." Part 25 streamlined process only, but it is the FCC's most direct text on the idea.
- DA 13-445 (15 Mar 2013, Guidance on Obtaining Licenses for Small Satellites): no timer or cessation-design guidance. Requires for amateur pre-launch notification a draft ITU Appendix 4 notification and an IARU coordination-complete letter; also post-launch and 3-month-after-termination notifications.
- Draft Part 100 (circulation document FCC-CIRC2607-02, fact sheet dated 1 July 2026; "Adopted" and "Released" fields blank in my copy): draft sec. 100.202(b) carries 25.207 forward including "timing devices"; draft 100.5(f) says amateur-satellite stations are licensed under Part 97. Search results report adoption on 22 July 2026 (SB Docket 25-306); I did not read the adopted order, so verify before citing.
- Nothing found in the 2020 orbital-debris order or the 5-year rule text that addresses command-loss timers.

NASA (not the FCC, but relevant to CSLI-launched CubeSats)
- NASA Spectrum Guidance for Small Satellite Missions v1.0 (27 Aug 2015), sec. on control and cessation of emissions: acceptable cessation methods are ground telecommand, a "fail-safe" mode that transmits only while an uplink is detected, and "indirect methods (e.g., battery life, timing devices, etc.)". It also cites ITU RR 15.1 (no unnecessary transmissions) and says satellites should not transmit when out of view of an associated Earth station. The document's stated expiry was 26 Aug 2017; I did not find a newer version.

IARU
- Coordination form v40, items 7.8 and 7.9: 7.8 asks how telecommand stations will turn the transmitter(s) off "immediately, even in the presence of user traffic and/or space station computer system failure"; 7.9 asks for "the mechanism to ensure the space station is turned off in the event communications is lost with the command station" and details of systems "to prevent unwanted transmissions from satellite if command capability is lost." No value or form of mechanism is prescribed. Section 7 is headed "Telecommand Information (NOT published)". https://www.iaru.org/wp-content/uploads/2020/06/iaru_amateur_satellite_coordination_request_v40.doc
- Instructions to the form (item 5b/5d): cite ITU RR 22.1; the IARU Satellite Adviser will optionally keep the turn-off procedure as a backup. No timer guidance. https://www.iaru.org/wp-content/uploads/2019/12/instructions_iaru_amateur_satellite_coordination_request.pdf
- "Controlling Space Station Transmitters" Rev 27 (27 Aug 2010): independent telecommand receiver is the preferred design; "immediate" means minutes for safety services and hours for other interference; once turned off a positive ground command should be required to re-enable and power reset following eclipse should not re-enable; time-out timer of about one pass only for the special low-orbit, few-ground-station case without a separate command receiver. This is the closest IARU text to SMART-QSO's design question (row 6).
- RR 25.11 and RR 22.1 are reproduced in the form's annex: administrations must ensure enough Earth command stations exist before launch so harmful interference can be terminated immediately; space stations must have devices ensuring immediate cessation of emissions by telecommand.

Others
- Klofas CubeSat communications survey (2009-2012) and presentation: recommendation never to turn off command receivers for any reason (supports the "receiver stays on" half of N). https://www.klofas.com/papers/Klofas_Comm_Survey_Presentation.pdf

---------------------------------------------------------------------------

## 6. Gaps and limits

- No IARU Q7.9 answers are public; no completed v40 form was found. Only an old-format filled form (ANDE) exists publicly.
- Could not locate Part 97 pre-space notifications or Part 5 ELS narrative exhibits for amateur-band university CubeSats; the FCC ELS search interface was not queryable with the tools available, and general web search does not index those attachments. An ELS/ICFS search by a person with direct access (narrative/technical-description exhibits, not ODARs) is the most likely way to find more FCC-filed values. Candidates: HuskySat-1 (UW Part 5 grant), CSUNSat1, Phoenix, LightCube, EagleSat-1.
- The FCC 19-81 text I read is the adopted order; the Part 100 text is a circulation draft.
- Several rows are [S] (read through a summarizer). The two timer rows that carry the recommendation (TechEdSat-1, Cactus-1) and 25.259(c) are [R].
- TechEdSat-1's page is a 2012 eoPortal compilation, not the mission's licence filing. Cactus-1's blog is a team statement from 2021; its IARU/FCC filings were not found.
- The recommendation depends on facts I do not have: SMART-QSO's orbit, number of command stations, and the actual commissioning schedule. If there is only one command station, lean toward the longer end of the ranges above; if several geographically separate stations, 14 d from launch becomes more defensible.

---------------------------------------------------------------------------

## 7. References (all accessed 2026-10-15)

Regulatory / FCC
- eCFR, 47 CFR 97.207, 97.211, 97.213, 97.109, 97.103, 97.3, 25.207, 25.283, 25.259, 5.64 (API snapshot 2026-10-01): https://www.ecfr.gov/current/title-47/
- FCC 19-81, Streamlining Licensing Procedures for Small Satellites: https://docs.fcc.gov/public/attachments/FCC-19-81A1.pdf
- FCC DA 13-445, Guidance on Obtaining Licenses for Small Satellites: https://docs.fcc.gov/public/attachments/DA-13-445A1_Rcd.pdf
- FCC Space Modernization (Part 100) circulation draft and fact sheet: https://docs.fcc.gov/public/attachments/DOC-422740A1.txt

IARU
- Coordination request form v40 (Jun 2020): https://www.iaru.org/wp-content/uploads/2020/06/iaru_amateur_satellite_coordination_request_v40.doc
- Instructions: https://www.iaru.org/wp-content/uploads/2019/12/instructions_iaru_amateur_satellite_coordination_request.pdf
- Controlling Space Station Transmitters, Rev 27: https://www.iaru.org/wp-content/uploads/2019/12/controllingsatellites_v27.pdf
- Filled old-format request (ANDE): https://www.aprs.org/ande/ANDE-IARU.doc

NASA
- Spectrum Guidance for NASA Small Satellite Missions v1.0 (27 Aug 2015): https://www.nasa.gov/wp-content/uploads/2023/02/spectrum-guidance-smallsats-cubesats-27aug2015.pdf

Mission sources
- TechEdSat-1: https://www.eoportal.org/satellite-missions/techedsat
- Cactus-1: https://www.captechu.edu/blog/cactus-1-cubesats-next-steps
- CuPID: https://arxiv.org/abs/2304.04702
- 3UCubed: https://edeon.sonoma.edu/3u3/news/3ucubed-launched/
- ARMADILLO: https://digitalcommons.usu.edu/smallsat/2022/all2022/4
- LASP MinXSS-2 anomaly: https://lasp.colorado.edu/minxss/2019/02/14/minxss-2-communications-anomaly/
- FUNcube-1: https://amsat-uk.org/2013/11/23/funcube-1-transponder-in-autonomous-mode/
- Fox-1Cliff: https://www.arrl.org/news/amsat-s-fox-1cliff-amateur-radio-cubesat-launched-successfully
- Klofas survey presentation: https://www.klofas.com/papers/Klofas_Comm_Survey_Presentation.pdf
