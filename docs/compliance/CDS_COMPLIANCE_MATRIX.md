# CubeSat Design Specification Compliance Matrix
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-CDS-001
**Revision:** B
**Date:** 2026-10-27
**Reference:** Cal Poly CubeSat Design Specification (1U–12U) Rev 14.1, 2022-02-09 (text read from
`https://www.cubesat.org/s/CDS-REV14_1-2022-02-09.pdf` on 2026-10-27)

> **Statuses are design intent, not verification.** No flight hardware exists. "Design" means the documented design
> meets the clause and the named verification is still to run; nothing here is "Compliant" until a test or
> inspection report is cited. Governing values: `docs/BASELINE_PARAMETERS.md`. Requirements of the dispenser and the
> launch vehicle that go beyond the CDS: `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001), whose
> verification items (VI-LV-nn) are named below.

---

## 1. Purpose

A clause-by-clause matrix of CDS Rev 14.1 for the SMART-QSO 1U CubeSat, rebuilt from the specification's text for
the CDR (finding VV-14). Every numbered "shall" and "should" is a row; notes are rows where they bear on the design.
The dispenser's manual takes priority where it conflicts with the CDS (EXOpod Nova UM Rev 1.2 §1.2); such conflicts
are noted in the last column.

Status words: **Design** (documented design meets it; verification open), **Open** (the design does not yet contain
what is needed), **Not met** (the design conflicts), **N/A** (does not apply), **Note** (no requirement).

## 2. General specifications (CDS §2.1)

| CDS | Clause (abridged) | SMART-QSO design | Verification | Status |
|-----|-------------------|------------------|--------------|--------|
| 2.1.1 | All parts remain attached during launch, ejection and operation | No detachable parts; burn-wire line remnants retained in the hold-downs | Post-vibration inspection (VI-LV-04) | Design |
| 2.1.2 | Pyrotechnics conform to AFSPCMAN 91-710 Vol. 3 | No pyrotechnics (burn wires) | Review | N/A |
| 2.1.3 | Propulsion per 91-710 Vol. 3 | No propulsion | — | N/A |
| 2.1.4 | Propulsion: ≥ 3 inhibits | No propulsion | — | N/A |
| 2.1.5 | Note: consider FAA airline battery guidance (100 Wh per battery) | 4S1P ≈ 50 Wh (BASELINE §5) | — | Note (met) |
| 2.1.6 | Hazardous materials conform to 91-710 Vol. 3 | Li-ion cells declared in the SDHS (LVR-008, LVR-072) | Review | Open |
| 2.1.7, 2.1.7.1, 2.1.7.2 | Low outgassing: TML ≤ 1.0 %, CVCM ≤ 0.1 % | `docs/compliance/MATERIALS_LIST.md`; sources to add | E595 data or rationale (VI-LV-09) | Open |
| 2.1.8 | Passive magnets ≤ 0.5 G above Earth's field outside the static envelope | No passive magnets; magnetorquer rods unpowered at launch; rod remanence to check | Measurement (VI-LV-18) | Design |
| 2.1.9 | Ascent venting: ventable volume / vent area < 50.8 m | Open frame; ≥ 22.3 mm² vent for the 1.135 L gross volume, each sub-volume on its own (`leop_feasibility.py` §4) | Analysis (VI-LV-12) | Open |

## 3. Mechanical specifications (CDS §2.2)

| CDS | Clause (abridged) | SMART-QSO design | Verification | Status |
|-----|-------------------|------------------|--------------|--------|
| 2.2.1, 2.2.1.1 | Coordinate system and dimensions per Appendix B | Bought 1U frame (BUILD_VS_BUY §1); 100.0 × 100.0 × 113.5 mm | Rail measurement, fit check (VI-LV-01) | Design |
| 2.2.2 | −Z face inserted first | −Z face carries the switches and plungers | Inspection | Design |
| 2.2.3 | No component on the shaded faces protrudes > 6.5 mm from the rail plane | Wings and RBF ≤ 6.5 mm by design; the 7 mm AntS module sits on a ±Z face and must fit the ±Z standoff (Appendix B) | Inspection (VI-LV-01) | Open (AntS fit) |
| 2.2.4 | Deployables constrained by the CubeSat, not the dispenser | Burn-wire hold-downs on wings; AntS release | Inspection | Design |
| 2.2.5 | Rails ≥ 8.5 mm wide to the first protrusion | Bought frame | Inspection | Design |
| 2.2.6 | Rails should have Ra < 1.6 µm | Hard-anodized rails | Inspection, vendor certificate | Design |
| 2.2.7 | Rail edges should be rounded to ≥ 1 mm | Bought frame. **Conflict:** the NRCSD IDD (backup) requires 0.5 ± 0.1 mm | Drawing, certificate | Open |
| 2.2.8 | ±Z rail ends ≥ 6.5 × 6.5 mm contact with neighbours | Bought frame | Inspection | Design |
| 2.2.9 | ≥ 75 % of each rail should contact the dispenser rails | Bought frame | Inspection | Design |
| 2.2.10 | Note: typical 1U maximum 2.00 kg; dispenser limits govern | Predicted 1.92–2.00 kg (MASS_BUDGET v2.0); EXOpod Nova 1U limit 2.5 kg (margin 20–23 %) | Weighing (VI-LV-02) | Note (at the 2.00 kg figure) |
| 2.2.11 | CG within ±2 cm of the geometric centre on each axis | Not computed (no layout) | CG fixture (VI-LV-02) | Open |
| 2.2.12 | Structure should be aluminium alloy | Aluminium frame | Review | Design |
| 2.2.13 | Aluminium surfaces in contact with the dispenser rails hard anodized | Rails and standoffs hard anodized | Certificate | Design |
| 2.2.14, 2.2.14.2 | When sharing a dispenser, a separation mechanism; not beyond the standoff when stowed | Four spring plungers in the −Z standoffs; force open (TBR-CDS-001). A 1U in an EXOpod always shares a 3U slot, with other spacecraft or an adapter | Inspection, force measurement | Open (force) |

## 4. Electrical specifications (CDS §2.3)

| CDS | Clause (abridged) | SMART-QSO design | Verification | Status |
|-----|-------------------|------------------|--------------|--------|
| 2.3.1 | Power system off from delivery to the LV through on-orbit deployment | Separation switches in the battery path | Inhibit test (VI-LV-06); power-inhibit monitoring during vibration (VI-LV-04) | Design |
| 2.3.1.2 | Powered battery protection permitted per 2.3.6 | BMS on the battery side; its drain sets the storage limit (≤ 20 µA, LVR-071) | Drain measurement (VI-LV-14) | Open (drain) |
| 2.3.2 | ≥ 1 deployment switch, actuated while integrated | Two rail-end switches in the −Z standoffs | Switch test (VI-LV-05) | Design |
| 2.3.2.1 | Actuated switch disconnects the power system from the powered functions | Switch contacts in the battery path; contact form and sense polarity open (TBR-CDS-005) | Schematic; VI-LV-06 | Open |
| 2.3.2.2 | Switch actuated at all times while integrated | Switch travel against the Nova adapter or neighbour to check | Fit check (VI-LV-01) | Open |
| 2.3.2.3 | Actuated switch should be at or below the interfacing surface | Plunger switches in the standoffs | Inspection | Design |
| 2.3.2.4 | A toggle resets the satellite to the pre-launch state, including the transmission and deployable timers | Power-on reset restarts both counts; a stowed reading while powered restarts both (`software/rust/sqso-fsw/src/rf_silence.rs`, `deployment.rs`); simulation-tested | Toggle test on the flight unit (VI-LV-05, VI-LV-07) | Design |
| 2.3.3–2.3.3.3 | RTC permitted if isolated, < 320 kHz, < 10 mA | No RTC fitted (the onboard time starts at separation) | — | N/A |
| 2.3.4 | RBF and umbilical connectors within the dispenser's access ports, if any | Nova access windows on three sides (NOVA §4.2.5); RBF and umbilical positions against them open | Fit check (VI-LV-01) | Open |
| 2.3.5 | RBF pin cuts all power once inserted | RBF in the battery path | Inhibit test (VI-LV-06) | Design |
| 2.3.5.1 | RBF removed before integration if the dispenser has no access port | Removed at integration in any case (no access planned afterwards) | Procedure (`docs/operations/LAUNCH_SITE_OPS.md`) | Design |
| 2.3.5.2 | RBF ≤ 6.5 mm from the rail surface when inserted | By design | Inspection | Design |
| 2.3.6 | Battery protection for charge and discharge (balance); documentation for non-UL cells | 4S BMS; UL-listed INR18650-35E cells (`hardware/eps/BATTERY_DESIGN.md`) | Protection trip test (VI-LV-11) | Design |
| 2.3.7 | ≥ 3 independent RF inhibits; a timer is not an inhibit (2.3.7.2) | **No inhibit schematic** (VV-03); two switches and the RBF documented; the RBF is removed at integration and the timers do not count | Inhibit-by-inhibit test (VI-LV-06) | **Open** |
| 2.3.8 | ≥ 3 independent inhibits against inadvertent release of deployables | Same | VI-LV-06 | **Open** |

## 5. Operational specifications (CDS §2.4)

| CDS | Clause (abridged) | SMART-QSO design | Verification | Status |
|-----|-------------------|------------------|--------------|--------|
| 2.4.1, 2.4.1.1 | Licences documented; for amateur frequencies, proof of IARU coordination | FCC Part 97 (KK6GDB-11); IARU request not submitted (`docs/regulatory/IARU_COORDINATION.md`) | Coordination letter (TBR-CDS-002) | Open |
| 2.4.2 | Comply with the country's radio licence | Part 97, pre-space notification draft | FCC filing record | Open |
| 2.4.3 | Mission design and hardware per NPR 8715.6 (orbital debris) | ODAR draft; passive decay with wings deployed (BASELINE §3) | DAS run | Open |
| 2.4.3.1 | Any component re-enters with < 15 J | Demise expected (aluminium, small parts); DAS to confirm, including the Jetson module and the battery cells | DAS | Open |
| 2.4.3.2 | Orbital debris data ready on request | `docs/regulatory/filings/ODAR_SMART-QSO.md` | — | Design |
| 2.4.4 | Deployables wait ≥ 30 min after switch activation | 30 min (`deployment.rs`, SYS-221); simulation 30.0 min | Timer test (VI-LV-07); burn-wire HIL (SW-10) | Design |
| 2.4.5 | No signal generated or transmitted earlier than 45 min after deployment | 45 min, whole radio unpowered (`rf_silence.rs`, SYS-224); simulation 45.0 min | Spectrum analyser test across resets (VI-LV-07; TP-RF-006 §8.8) | Design |
| 2.4.5.1 | Note: may power on immediately after deployment | The OBC boots at separation | — | Note |
| 2.4.6 | Note: NOAA remote-sensing licence if applicable | No imager: `docs/regulatory/filings/NOAA_REMOTE_SENSING_DETERMINATION.md` | — | Note (N/A) |
| 2.4.7 | Dispenser developer conducts ≥ 1 fit check with the flight unit, and a final one before integration | Nova TestPod fit during vibration; final at the integrator | VI-LV-01 | Open |

## 6. Testing requirements (CDS §3)

| CDS | Clause (abridged) | SMART-QSO plan | Verification | Status |
|-----|-------------------|----------------|--------------|--------|
| 3 (intro) | Launch provider requirements supersede; GEVS or SMC-S-016 only if the LV is unknown | SpaceX UG v11 levels (interim, LVR-044) | — | Design |
| 3.1.1 | Random vibration to the launch provider's levels and duration | Protoflight 5.57 Grms, 1 min/axis (TP-ENV-002) | VI-LV-04 (TBR-CDS-003) | Open |
| 3.2.1, 3.2.2 | Thermal vacuum bakeout, specification from the launch provider | In the TVAC campaign (TP-ENV-003) | VI-LV-08 (TBR-CDS-004) | Open |
| 3.3.1 | Shock testing as the launch provider defines | SpaceX: advised only; not planned, risk acknowledged | VI-LV-16 | Design (not required) |
| 3.4.1 | Visual inspection and measurement per the CIFP or the provider | Exolaunch rail measurement guide | VI-LV-01 | Open |
| 3.5, 3.5.2 | Qualification or protoflight; no disassembly or modification after protoflight without a waiver | Protoflight on the flight unit; burn-wire reset after deployment tests is allowed for containerized CubeSats if identical (UG §6.3 item 7) | Test records | Open |

## 7. Summary

| Section | Rows | Design | Open | Not met | N/A or note |
|---------|------|--------|------|---------|-------------|
| 2.1 General | 9 | 2 | 3 | 0 | 4 |
| 2.2 Mechanical | 14 | 9 | 4 | 0 | 1 |
| 2.3 Electrical | 15 | 8 | 6 | 0 | 1 |
| 2.4 Operational | 10 | 3 | 5 | 0 | 2 |
| 3 Testing | 6 | 2 | 4 | 0 | 0 |
| **Total** | **54** | **24** | **22** | **0** | **8** |

No row is verified. The open rows that block the launch contract's safety review are 2.3.7 and 2.3.8 (the inhibit
architecture, VV-03) and 2.3.2.1 (switch contact form).

## 8. Changes from revision A

Revision A (A5) misstated the CDS in several places. Corrected here: the corner and edge-radius row (A said 1.0 mm
maximum; CDS §2.2.7 says at least 1 mm, "should"); solar panels called body-mounted (the wings deploy); test levels
attributed to the CDS (the CDS defers to the launch provider); §2.3.2.4 (timer reset), §2.1.8, §2.1.9, §2.2.14,
§2.3.3, §2.4.3.1 and §3.2 added; the RBF counted as an inhibit removed; "Post-deployment power 2 W max" corrected to
the baseline's 1 W; "All materials qualified" and "CCB process defined: Compliant" replaced by the actual status.

## 9. TBR Closure Plan

| Item | Description | Closure Activity | Target |
|------|-------------|------------------|--------|
| TBR-CDS-001 | Separation spring force | Frame vendor's plunger data; measure at integration | Frame purchase |
| TBR-CDS-002 | IARU frequency coordination | Submit the request ≥ 12 months before integration; letter before the L-2 M licence deadline | IARU letter |
| TBR-CDS-003 | Vibration test | Protoflight test VI-LV-04 at the SpaceX UG v11 levels | L-4 M |
| TBR-CDS-004 | Thermal vacuum test | Protoflight TVAC with bakeout, VI-LV-08 | L-4 M |
| TBR-CDS-005 | Deployment-switch contact form and sense polarity. This matrix (§4.1 of revision A) and `docs/compliance/LAUNCH_PROVIDER_ICD.md` §5.1 said "normally open", `hardware/structure/STRUCTURAL_DESIGN.md` §5.2 "normally closed"; none cites a part or a schematic, and `docs/BASELINE_PARAMETERS.md` does not set it. The documents may describe different contacts: the power-inhibit contact, which must be open while the switch is actuated in the dispenser (the switches are power inhibits, `software/flight/include/rf_silence.h`), and the OBC sense line, which the flight software reads as high (pulled up) while stowed, itself marked open in `software/flight/include/separation_switch.h` | Select the switch part and draw the separation-switch schematic; state the contact form of each contact and the sense polarity here, in the launch ICD §5, the structural design §5.2 and `separation_switch.h`; the pre-ship test checks that released switches read released | Separation-switch schematic |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| A1 | 2026-10-14 | Inhibit and timer rows corrected to CDS Rev 14.1; audit notice | Funding audit |
| A2 | 2026-10-16 | Mass rows cite the bottom-up MEL (TB-06) | Funding audit |
| A3 | 2026-10-16 | §9 TBR IDs made unique to this document: TBR-001 to TBR-004 became TBR-CDS-001 to TBR-CDS-004. Each matrix row marked TBR now names the §9 item that closes it (finding VV-17) | SMART-QSO Team |
| A4 | 2026-10-16 | Local TBRs numbered TBR-CDS-001 to -004, so they do not collide with the specifications' TBR-001 (finding VV-06) | SMART-QSO Team |
| A5 | 2026-10-16 | §4.1 switch type: "normally open" conflicted with the structural design's "normally closed" and nothing decides it; tracked as TBR-CDS-005 (§9), summary counts updated, compliance rate restated as design intent | SMART-QSO Team |
| A5, note | 2026-10-24 | §2.4.5 and §2.4.4 rows name the Rust files that replaced `rf_silence.c` and `deployment.c` | SMART-QSO Team |
| B | 2026-10-27 | CDR: rebuilt clause by clause from the CDS Rev 14.1 text (54 rows, finding VV-14); statuses as design intent with the verification item of SQSO-LVR-001 named in each row; dispenser conflicts noted (EXOpod Nova mass and protrusion, NRCSD edge radius); the TBR closure plan given activities and targets | CDR review (LV) |
