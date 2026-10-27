# Risk Register

**Document ID**: SQSO-RISK-001
**Version**: D
**Date**: 2026-10-27
**Status**: CDR update. Single controlled risk list (finding PI-08); the risk tables in the PDR/CDR packages, the
Executive Summary and `docs/hardware/EPS_DESIGN.md` §9 are superseded by it

Owners are roles. The people in most roles are not yet named (decision D1); until they are, the PI holds every role.
The register is not yet tracked as GitHub issues (no `risk/*` label exists).

## 1. Scoring

**Likelihood (L)**, probability the risk occurs before the mitigation completes:

| L | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| Probability | ≤ 10 % | 10–25 % | 25–50 % | 50–75 % | > 75 % |

**Consequence (C)**, the worst of the three:

| C | Technical / mission | Schedule | Cost |
|---|---------------------|----------|------|
| 1 | Negligible | < 2 weeks | < $5k |
| 2 | Minor degradation; success criteria still met | 2–6 weeks | $5–20k |
| 3 | Loss of an L2/L3 criterion, or a regulatory finding that delays a filing | 1–3 months | $20–50k |
| 4 | Loss of L2 or of the launch slot; non-compliant filing | 3–6 months | $50–100k |
| 5 | Loss of mission, launch-safety rejection, licence denial or a safety event | > 6 months, or loss of the 2028 launch | > $100k |

**Score** = L × C. **Red ≥ 12**, yellow 6–10, green ≤ 5. The matrix (current scores; IDs in the cells):

| L \ C | 1 | 2 | 3 | 4 | 5 |
|-------|---|---|---|---|---|
| **5** | | | | R11, R14, R24 | |
| **4** | | | R30, R31 | R16, R21, R25, R29 | R26, R27 |
| **3** | | R17 | R4, R6, R8, R9, R15, R33, R34 | R12, R13, R18, R19, R22, R23, R28, R32, R35 | |
| **2** | | R3 | R7, R10, R36 | R2, R5, R20 | |
| **1** | | | | | |

Trend: ↑ worse, → steady, ↓ better since the last review, ● new.

## 2. Register

### 2.1 Technical and safety risks

| ID | Risk (if … then …) | L | C | Score | Trend | Owner | Mitigation and burn-down (date → target L×C) |
|----|--------------------|---|---|-------|-------|-------|----------------------------------------------|
| R2 | If the Jetson suffers TID or SEE beyond its tolerance, the experiment ends early | 2 | 4 | 8 | → | Payload lead | Non-critical, power-gated, OBC trip independent of it; the experiment is not a success-level gate (MSC v2.0 §6). 2027-06 EM latch-up trip test → 2×3 |
| R3 | If generation falls short of the orbit-average load, the transponder duty drops | 2 | 2 | 4 | → | EPS lead | Duty and SOC gates; SIL shows the gates bind before the pack (BUILD_VS_BUY §7.3). See R31 for the tumbling margin |
| R4 | If IARU coordination is delayed or conditioned, the frequency plan changes late | 3 | 3 | 9 | → | Licensee | Request by 2027-01-26 (SCHEDULE IARU_REQ); configurable frequency and cadence → 2027-07 coordination letter → 1×3 |
| R5 | If an antenna or wing fails to deploy, L2 is lost (both wings) or the link is lost (VHF) | 2 | 4 | 8 | → | Mechanisms lead | AntS redundant release; VI-LV-17 cold deployment test after environments (2028-01); see R36 for the shared wing driver → 1×4 |
| R6 | If Jetson, PA or OBC emissions desense the receiver or the GNSS, telecommand margin and ephemeris suffer | 3 | 3 | 9 | → | RF lead | EMC pre-scan on the EM (2027-06); L1 noise test with Jetson, PA and magnetorquers (BASELINE §6 GNSS row) → 2×3 |
| R7 | If the battery mount or Jetson heat path differs from the model, the pack falls below 0 °C or the Jetson overheats | 2 | 3 | 6 | ↓ | Thermal lead | 1 W heater (BUILD_VS_BUY §4.5); EM thermal balance (TB-07), 2027-07 → 1×3 |
| R8 | If the battery program (screening, protections, UN 38.3) is incomplete, the safety review or shipping is blocked | 3 | 3 | 9 | ↑ | EPS lead | SQSO-BSA-001 §4 screening and V-BSA-1–8; UN 38.3 pack test before first shipment (2027-10) → 1×3 |
| R10 | If an environmental test fails, the campaign repeats | 2 | 3 | 6 | → | Systems engineer | EM qualification-level tests first (2027-07); protoflight campaign by L−4 M → 2×2 |
| R16 | If the Jetson cannot meet power, thermal (≥ 1 W/K) or the 0.1 Wh energy gate in 1U, the experiment is descoped | 4 | 4 | 16 | → | Payload lead | Merged with R1. EM carrier measurement of energy per accepted beacon (D5) and heat path, 2027-06; descope trigger D4/D5 → decision 2027-07 → 2×2 |
| R18 | If flight software or firmware has defects found only on hardware, commissioning fails | 3 | 4 | 12 | → | FSW lead | Bring-up on an STM32L4R5ZI board and the flatsat (TP-FLATSAT-001) 2027-04 to 2027-06; ≥ 500 h flatsat running before the flight build (RELIABILITY v2.0 §6) → 2027-09 → 2×4 |
| R24 | **No build-to design and no engineering model** (findings PM-01, SE-01): no schematics with selected parts, PCB layouts, mechanical CAD or structural model, so no analysis at CDR rests on a design | 5 | 4 | 20 | ● | Systems engineer | EM schematic capture 2027-01-05, layout 2027-03-09, CAD and structural model 2027-03-23, EM integrated 2027-06-08 (SCHEDULE §3); hold the independent CDR only with EM data (2027-08-03) → 2×4 |
| R25 | **If the deployment altitude stays at the former baseline (target ≤ 500 km, cap 525 km), the FCC 5-year rule is not met** (ODAR Draft B §6: non-compliant at 525 km in every case; at 500 km only under nominal solar activity) and the pre-space notification is deficient. *Rebaselined 2026-10-27 to a 475 km target and 485 km cap (BASELINE Rev E §3, CR-2026-007); the residual risk is that no integrator commits in writing to ≤ 485 km* | 4 | 4 | 16 | ● | PI | Rebaseline to 465–485 km, cap 485 km (RID ODA-01), 2026-12; altitude clause in the integrator RFQs (2026-12-31) and contract (2027-06-30); FCC consultation on failed-spacecraft reading (D3), 2027-01; DAS run 2027-02 → 2×3 |
| R26 | **If the third RF and deployable inhibit is not designed in, the launch safety review rejects the payload** (CDS 2.3.7–2.3.8). *Designed 2026-10-27: DS3 on its own rail and harness (SQSO-EPS-BRD-001 §6.2; BASELINE Rev E §4); open until VI-LV-06* | 4 | 5 | 20 | ● | EPS lead | Add DS3 (SDP-002 §3.2) at EM schematic capture, 2027-01-05; VI-LV-06 on the EM 2027-06 → 1×5 |
| R27 | **If the wing burn line draws ≈ 5.8 A through the 5 A fuse and BMS trip, the first wing deployment ends the mission**; with no on-time limit a stuck driver is a fire hazard (SMA-05). *Designed 2026-10-27: four 0.8 A drivers, one per resistor, from a 7.0 V burn buck, 12 s one-shot, one resistor at a time, 0.67 A at the pack (SQSO-EPS-BRD-001 §9); open until VI-SAF-04* | 4 | 5 | 20 | ● | EPS lead | Current limit 0.5–1 A per resistor and hardware on-time cut-off at schematic capture (2027-01-05); VI-SAF-04 on the EM (2027-06) → 1×5 |
| R28 | If the secondary over-voltage protector, recharge lockout and end-of-life disconnect are not designed, overcharge stays single-fault tolerant, explosion probability is ×3 higher and passivation option 1 is unavailable (EPS-11, EPS-12, SMA-06). *Designed 2026-10-27: BQ771806 + F3, latching lockout K1, EOL fuse F1 with two signals (SQSO-EPS-BRD-001 §5.4, §6.1, §7); open until V-BSA-2, V-BSA-7* | 3 | 4 | 12 | ● | EPS lead | Add at schematic capture (2027-01-05); V-BSA-2, V-BSA-7 on the EM (2027-06) → 1×4 |
| R29 | If early (infant-mortality) failures occur at the CubeSat rate, L1/L2/L3 probabilities are 0.75 / 0.61 / 0.51 (RELIABILITY_ANALYSIS v2.1; 0.60 / 0.50 with the former single wing driver), below the earlier goals | 4 | 4 | 16 | ● | Systems engineer | PI accepts Class D estimates (REL-01, 2026-12); flatsat ≥ 500 h, flight burn-in ≥ 200 h, day-in-the-life tests (RELIABILITY v2.0 §6) through 2028-01 → 3×4 |
| R30 | If the mass margin stays at 0–4 % (MEL v2.0), the dispenser or CDS limit is exceeded, and the lifetime and collision analyses must move to 2.0 kg. *MEL v3.0 (2026-10-27): CBE 1.49 kg, MEV 1.73 kg, +15.9 % to 2.00 kg (BASELINE Rev E §4)* | 4 | 3 | 12 | → | Systems engineer | MEL re-run with quoted COTS masses (−0.20 to −0.25 kg, BUILD_VS_BUY §1), 2027-02; 2S1P trigger at < 10 % margin (D8) → 2×3 |
| R31 | If the tumbling power margin stays at +19 % with self-shadowing (below the 20 % requirement), transponder duty is cut when sun-pointing is lost | 4 | 3 | 12 | → | EPS lead | PI decision (BASELINE §5 note): load or layout change, 2026-12 → 2×3 |
| R32 | If the 435 MHz receiver fails, the loss-of-contact inhibit (N = 30/14 days) silences the beacon too, turning a partial failure into mission loss (SQSO-FMEA-002 HF-21) | 3 | 4 | 12 | ● | Licensee | Raise in the D3 consultation (2027-01): keep the beacon when the receiver is known failed; receiver burn-in → 2×4 |
| R33 | If materials or finishes are non-compliant (Loctite 242 not on the outgassing list; pure-tin terminations prohibited by UG v11 Table 5-5; unsourced outgassing values), the contamination report is rejected | 3 | 3 | 9 | ● | M&P lead | MATERIALS_LIST Rev B substitutions; terminal-finish survey and re-tinning (EEE_PARTS_LIST Rev C §2.3) by L−4 M (2028-02); VI-LV-09, VI-SAF-07 → 1×3 |
| R34 | If radiation data for the STM32L4R5, FRAM and RF parts are lacking or poor, SEL or TID shortens the mission | 3 | 3 | 9 | ● | Parts lead | Heritage reports or lot TID test (RAD-V3), 2027-09; SEL current limiters on every rail (MAP §7) → 2×3 |
| R35 | If NASA DAS is not run with the controlled MEL before the pre-space notification, the debris disclosures rest on project models only | 3 | 4 | 12 | ● | S&MA lead | Obtain DAS 3.2.6 (software usage agreement), 2026-12; run after the MEL re-run, 2027-02 (SCHEDULE DAS) → 1×4 |
| R36 | If the shared wing burn driver fails, both wings stay stowed (1.6 % per the model with one driver; 0.29 % with the four-line design of SQSO-EPS-BRD-001 §9, of which 0.2 % is the SW6/U17 burn supply the drivers still share): L2 lost and the longer wings-failed lifetime applies | 2 | 3 | 6 | ● | EPS lead | One driver per wing at schematic capture (2027-01-05) → 1×3 |

### 2.2 Programmatic risks

| ID | Risk (if … then …) | L | C | Score | Trend | Owner | Mitigation and burn-down (date → target L×C) |
|----|--------------------|---|---|-------|-------|-------|----------------------------------------------|
| R9 | If supply-chain lead times grow (solar panels 3–7 months, a shrinking 1U catalogue), the EM or FM slips | 3 | 3 | 9 | → | Systems engineer | Order EM and FM panels together (2027-01-15); RFQ three frame vendors (BUILD_VS_BUY §2.2) → 2×3 |
| R11 | If funding is not secured in time to reserve a 2028 launch, the launch is lost | 5 | 4 | 20 | → | PI | Funding decision by 2027-05-31; integrator quotes by 2026-12-31; phased funding tied to milestones → 2027-05 → 2×4 |
| R12 | If funding terms compromise Part 97 validity (compensated operation, data rights), a Part 5 pivot is forced | 3 | 4 | 12 | → | PI | Grants and donations only; conditions C1–C8 reviewed before accepting funds; FCC consultation (D3) 2027-01 → 2×4 |
| R13 | If no 1U integrator slot at the required altitude exists before 2029, launch slips or moves to the ISS (Level 3 lost) | 3 | 4 | 12 | ↑ | PI | Worse with R25 (≤ 485 km): ask OTV-capable integrators (D-Orbit, Exolaunch) for a drop-off in the band, 2026-12-31; decide primary/backup by 2027-09 → 2×4 |
| R14 | If the project keeps one contributor and no named team, every activity is single-threaded | 5 | 4 | 20 | → | PI | Name the team and licensee roles (D1); at least two trained control operators; 2027-01 → 3×3 |
| R15 | If the integrator is foreign or the GNSS needs a Czech export licence, a TAA or licence adds 3–6 months | 3 | 3 | 9 | → | PI | Export counsel by 2027-01-31; 20-week allowance in the schedule (EXPORT) → 2×3 |
| R17 | If the ISS backup is taken, JSC 20793 battery certification adds 6–9 months | 3 (if ISS) | 2 | 6 | → | EPS lead | Battery program to JSC 20793 practice on both paths (SQSO-BSA-001 §4) → 2×2 |
| R19 | If IARU coordination and the FCC pre-space notification (≥ 90 days before integration) slip, integration is blocked | 3 | 4 | 12 | → | Licensee | IARU request by 2027-01-26; FCC notification within 30 days of the launch-vehicle determination (2027-06-28) → 2×4 |
| R20 | If unsubstantiated legacy claims reach a funder or regulator, credibility is lost | 2 | 4 | 8 | ↓ | PI | Audit notices; findings register; this CDR's RIDs → 1×4 |
| R21 | **Schedule:** a 2028-06-01 launch misses the integrator's L−4-month protoflight milestone by 21 days (SCHEDULE §3: SCH, PDR, LAYOUT, EM_FAB, EM_INT at −21 d float) | 4 | 4 | 16 | ● | PI | Plan on the last 2028 Transporter flight (≈ 2028-11-01: +130 d float); hold the RID closure to 2026-12-08 (7 d float) → 2027-01 replan → 2×4 |
| R22 | **Funding timing:** EM money arriving after 2027-01-15 moves the protoflight campaign week for week (FUND_EM on the critical path, −17 d float on the June date) | 3 | 4 | 12 | ● | PI | Secure EM funding by 2027-01-15; buy the long-lead COTS (EM_LL) first; plan on the November 2028 flight → 2027-01 → 2×4 |
| R23 | **Staffing:** there is no mechanical designer, but the integrator needs CAD and a dynamic model at L−8 months (LVR-003) | 3 | 4 | 12 | ● | PI | Recruit or contract a mechanical designer by 2027-01-12 (MECH starts then); use the frame vendor's CAD → 2027-03 → 1×4 |

### 2.3 Closed or merged

| ID | Disposition |
|----|-------------|
| R1 | Merged into R16 (Jetson power and thermal) |

## 3. Notes

- Linked RIDs: R24 (PM-01, SE-01), R25 (ODA-01), R26 (SMA-01), R27 (SMA-05), R28 (SMA-06, EPS-11, EPS-12), R29
  (REL-01), R33 (LV-26), R35 (ODA-02), R36 (REL-03). RID details are in the CDR S&MA review report and the documents
  named in each row.
- Review the register monthly and at each milestone; a risk retires when its burn-down target is reached and evidenced.

## 4. Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-14 | Programmatic risks R11–R20 added by the funding audit (PI-08) |
| B | 2026-10-27 | CDR update: 5×5 likelihood and consequence scales with a matrix; every risk rescored with owner, burn-down dates and target; R1 merged into R16; CDR risks R24–R36 (build-to design and EM, debris altitude, inhibits, burn-line current, battery protections, reliability, mass, tumbling margin, receiver and loss-of-contact coupling, materials, radiation data, DAS, wing driver); programmatic risks R21–R23 (schedule, funding timing, staffing) from the programmatics review |
| C | 2026-10-27 | CDR 2026 consistency pass: status notes on R25 (altitude rebaselined, BASELINE Rev E), R26 (DS3 designed), R27 (burn drivers designed), R28 (protection and EOL disconnect designed), R30 (MEL v3.0 margin); scores unchanged pending the risk owners |
| D | 2026-10-27 | Re-run values (RID SE-16): R29 probabilities 0.75 / 0.61 / 0.51; R36 both-wings-lost 0.29 % with the four-line design. Scores unchanged pending the risk owners |
