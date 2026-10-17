# SMART-QSO System Safety Data Package (Skeleton)

**Document ID**: SQSO-SDP-002
**Version**: 0.1 (skeleton)
**Date**: 2026-10-16
**Classification**: Unclassified
**Status**: **INCOMPLETE.** Structure only. No section contains accepted evidence, and no hazard is closed

> **Read this first.** This is the outline of the **system** safety data package that a launch provider will review: the
> whole spacecraft, its ground support equipment and its launch-site operations. It is not the
> [software safety data package](SAFETY_DATA_PACKAGE.md), which covers flight software only and is one input to section
> 13 here.
>
> **Interim provider: SpaceX Rideshare Payload User's Guide, Version 11, August 2026** ("UG v11",
> <https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf>), **pending a launch contract and
> PI confirmation.** The baseline launch path is a Transporter-class rideshare bought through a 1U-capable integrator
> (BASELINE §3, decision D7). The integrator is SpaceX's customer, will tailor these requirements, and may add its own
> format; its user guide and ICD will supersede this outline. The backup (ISS via the Voyager NRCSD) has different safety
> rules (JSC 20793 for batteries) and is not covered here.
>
> Each section ends with **Evidence still needed**. Until that evidence exists, the section is open.

## 1. Purpose and Scope

**Purpose.** Show the launch provider and its range-safety authority that SMART-QSO creates no unacceptable hazard to
personnel, the launch vehicle, co-payloads or facilities, from delivery to the launch site through separation and
initial operations.

**Scope.** Flight hardware, flight software, ground support equipment (GSE) and the hazardous operations planned at the
launch site and integration facility.

**Evidence still needed.** Signed launch contract naming the integrator and dispenser; the integrator's safety
requirements document and templates.

## 2. Applicable Documents

| Document | Use |
|----------|-----|
| SpaceX Rideshare Payload User's Guide v11, Aug 2026 (link above) | Interim provider requirements: §4 environments, §5 design requirements, §6 verification, §9.5 customer responsibilities, §11 safety, Appendices D, F and G |
| Space Systems Command Manual 91-710, Range User's Manual (as tailored by SpaceX; UG v11 §11.1) | Range safety requirements for flight and ground systems. Not yet obtained or read; SpaceX provides compliance templates |
| CubeSat Design Specification Rev 14.1, Cal Poly, Feb 2022 (<https://www.nasa.gov/wp-content/uploads/2018/01/cubesatdesignspecificationrev14_12022-02-09.pdf>) | Inhibits (§2.3), timers (§2.4), materials (§2.1.7), testing (§3) |
| Integrator user guide and ICD | TBD at contract |
| [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) | Design values |
| [SAFETY_DATA_PACKAGE.md](SAFETY_DATA_PACKAGE.md) | Software safety (SQSO-SDP-001) |
| [LAUNCH_PROVIDER_DELIVERABLES.md](../regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md) | Deliverables checklist, including the battery data package |

**Evidence still needed.** SSCMAN 91-710 as tailored for the mission; integrator documents.

## 3. Payload Description

| Item | Baseline value (BASELINE) | Evidence still needed |
|------|---------------------------|-----------------------|
| Form factor | 1U, 100 × 100 × 113.5 mm (§4) | Dimensional inspection (CubeSat acceptance checklist) |
| Mass | CBE ≈ 1.51 kg [audit estimate]; MEV ≤ 1.75 kg; limit 2.00 kg or the dispenser limit (§4) | Bottom-up mass equipment list; measured mass and centre of gravity |
| Deployables | VHF antenna, UHF antenna, two solar wings; burn-wire release (§4) | Mechanism drawings; secondary-lock design |
| Energy storage | 4S1P Li-ion 18650, ≈ 3.5 Ah, ≈ 50 Wh (§5) | Battery data package |
| RF transmitters | One VHF downlink transmitter, ≤ 1 W (§7). The 435 MHz uplink radio and the GNSS unit are receivers (§6, §7) | Transmitter list for the UG v11 payload-transmitter webform |
| AI payload | Jetson Orin Nano module, ≤ 8 W input, power-gated (§6) | Thermal and power-isolation design |
| Propulsion, pressure vessels, ordnance | None | Statement, confirmed by the integrator |

## 4. Hazardous Systems Inventory

UG v11 §11.2 lists the systems ranges usually treat as hazardous. Each needs a determination for SMART-QSO.

| UG v11 §11.2 category | SMART-QSO | Determination | Evidence still needed |
|-----------------------|-----------|---------------|-----------------------|
| Ordnance | None. Deployables use burn-wire release, not pyrotechnics | Proposed: not ordnance | Integrator confirmation; burn-wire circuit description |
| Pressurized systems (below 4:1 safety factor) | None. Sealed Li-ion cells only (batteries are "other pressurized equipment", UG v11 §5.5.4) | Proposed: not a pressure system | Cell data sheets; confirm with the integrator |
| Lifting operations | None expected (hand-carried 1U) | Proposed: none | Ground operations plan |
| Toxic or hazardous materials | Li-ion cell electrolyte | Hazardous material | List per 49 CFR 172.101 for the FAA payload-review template and the Appendix F certification |
| High-power RF | ≤ 1 W VHF/UHF | Proposed: not high-power | RF exposure analysis for personnel; transmitter inhibit evidence (§8) |
| Lasers | None | None | — |
| Batteries | 4S1P Li-ion, ≈ 50 Wh | Hazardous system | §7 |
| GSE | Battery charger, EGSE, key-load umbilical | Ground hazard review | GSE list and procedures (§12) |

## 5. System Hazard Analysis

Proposed top-level hazards. Severity and likelihood use the range's definitions once SSCMAN 91-710 is obtained.

| ID | Hazard | Causes (examples) | Controls (designed, not verified) | Evidence still needed |
|----|--------|-------------------|-----------------------------------|-----------------------|
| SH-01 | RF emission before or during launch, or too soon after separation | Inhibit failure; switch bounce; software fault | ≥ 3 independent physical RF inhibits (CDS 2.3.7; BASELINE §4); no RF before 45 min (CDS 2.4.5) | Inhibit schematic; inhibit test during random vibration (§6) |
| SH-02 | Inadvertent deployment in the dispenser | Burn-wire actuation; release under vibration | ≥ 3 independent inhibits per deployable (CDS 2.3.8); 30 min timer (CDS 2.4.4); secondary locks | Inhibit schematic; vibration test with stowed deployables |
| SH-03 | Battery fire, venting or thermal runaway | Over-charge, short circuit, cell defect, mechanical damage | Cell protection (≤ 4.25 V/cell), charge window 0–45 °C, fuse, restraint (BASELINE §5) | Battery data package (§7) |
| SH-04 | Structural failure; release of parts | Under-designed structure or fasteners | Factors of safety (UG v11 Table 5-1); fastener rules (§5.2); random-vibration test | Structural analysis (FEM) and test report (§9) |
| SH-05 | Contamination of co-payloads | Outgassing; particulates | TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS 2.1.7) | Materials list with outgassing data (§10) |
| SH-06 | Collision or debris after deployment | No manoeuvring; long orbital life | ≤ 525 km deployment; passive re-entry; GNSS ephemeris (BASELINE §3, §6) | ODAR with DAS output; ephemeris plan |

**Evidence still needed.** Severity and likelihood for each hazard per the tailored range manual; a hazard report per
hazard in the range's format; integrator review.

## 6. Inhibits and Safing

- **Requirements.** ≥ 3 independent physical inhibits between the battery and each RF transmitter, and ≥ 3 per
  deployable; RBF pins and timers do not count (CDS 2.3.7, 2.3.8; BASELINE §4). Power off from delivery to deployment
  (CDS 2.3.1). Deployment switch toggling resets timers (CDS 2.3.2.4).
- **Verification (UG v11 §6.1.2, §6.7.7).** Inhibits must be verified **during random vibration**, with all power
  components integrated, batteries at flight-like charge and RBF/GSE inhibits removed. Evidence such as boot logs or
  oscilloscope data must show no inadvertent power-up, and that power was inhibited by the mechanical separation signal,
  not by a software delay. Separation-detection circuits are checked before and after dynamic tests.
- **Current state.** The design is described in the repository; the launch deliverables checklist notes that three
  *physical* RF inhibits have not been confirmed.

**Evidence still needed.** Inhibit diagram; electrical schematics; test procedure
(`docs/test_procedures/TP-MECH-003_Deployment_Switches.md` and `docs/test_procedures/TP-ENV-002_Vibration.md`); test
report with boot logs or oscilloscope traces.

## 7. Battery Safety

- **Design.** 4S1P Li-ion 18650; cell over-voltage protection ≤ 4.25 V/cell; no charging below 0 °C or above +45 °C;
  permanent passivation at end of life (BASELINE §5).
- **Launch-site charging.** UG v11 Appendix D offers three scenarios: no charging after stand-alone operations, charging
  before rollout, or charging on the pad (optional services). Integrated in a dispenser without access ports, the
  satellite cannot be charged after integration (CDS 2.3.4.1), so the charge plan must cover storage until launch.

**Evidence still needed.** Battery data package (`docs/regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md` §2): cell
data and certification, protection schematic, screening and pack acceptance records, charging procedure; shipping
classification for lithium batteries; self-discharge analysis for the chosen Appendix D scenario.

## 8. RF Safety

- No open-air RF checkouts during integration; no transmission while mated to launch-vehicle hardware (UG v11 §9.5.2).
- Transmitter turn-on only after separation and the delay in UG v11 Table 4-10 for the transmitter's EIRP and the
  separation velocity; SMART-QSO waits 45 min (CDS 2.4.5), which exceeds every entry in that table.
- Payload emissions within UG v11 Table 4-9 while integrated (the satellite is powered off, so it emits nothing).

**Evidence still needed.** Payload-transmitter webform data; RF-silence timer test record; RF exposure analysis for
personnel at the integration site.

## 9. Structural and Mechanical Safety

- Random vibration at no less than the UG v11 maximum predicted environment (5.57 Grms, Table 4-6), with inhibit
  verification (UG v11 Table 6-2); levels in `docs/test_procedures/TP-ENV-002_Vibration.md`.
- No elastic modes below 40 Hz (UG v11 §4.1.1, §6.7.1).
- Minimum factors of safety: 1.10 yield, 1.25 ultimate (flight), 1.40 ultimate (ground operations) (UG v11 Table 5-1).
- Fastener locking features and rework rules (UG v11 §5.2); no passive deployment of appendages during ejection
  (UG v11 §5.7).

**Evidence still needed.** Finite-element model and margin report (none exists; see
`hardware/structure/STRUCTURAL_DESIGN.md`); random-vibration test report; fastener list with locking features.

## 10. Materials and Contamination

- TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS 2.1.7). UG v11 Table 5-5 prohibits cadmium, zinc plating, mercury and pure tin.
  Fully containerized CubeSats are exempt from some UG v11 contamination requirements (§5.3), which then fall on the
  integrator's dispenser assembly.

**Evidence still needed.** Materials list with outgassing sources (`docs/compliance/MATERIALS_LIST.md` needs sources);
prohibited-materials statement; contamination report if the integrator requires one (UG v11 §6.7.12).

## 11. Venting

- Ventable volume per area ratio within the fairing pressure-decay environment (UG v11 §4.1.8; CDS 2.1.9).

**Evidence still needed.** Venting analysis (advised by UG v11 §6.7.9).

## 12. Ground Operations and Hazardous Procedures

- A Payload Ground Operations Plan and advance copies of every hazardous procedure, reviewed and approved by the range
  safety authority (UG v11 §9.5.3, Appendix G); hazardous operations need 72 hours' notice (UG v11 §9.5.2).
- Expected hazardous operations: battery charging; handling of the charged spacecraft; RBF removal at dispenser
  integration.

**Evidence still needed.** Ground operations plan; charging, handling and integration procedures; GSE list with
electrical safety data; personal protective equipment list.

## 13. Software Safety

Summary of the [software safety data package](SAFETY_DATA_PACKAGE.md). Software provides the 30 min deployment and
45 min RF timers and TX-OFF, but **launch safety must not depend on software**: UG v11 §6.7.7 requires inhibition by the
mechanical separation signal, and CDS 2.3.7.2 says a timer is not an inhibit.

**Evidence still needed.** Software hazard controls verified on hardware (all open in the software package).

## 14. Verification Status

| Hazard | Control | Method | Evidence | Status |
|--------|---------|--------|----------|--------|
| SH-01 | RF inhibits | Test during random vibration | — | Open |
| SH-02 | Deployable inhibits and secondary locks | Test and inspection | — | Open |
| SH-03 | Battery protection and charging limits | Test and analysis | — | Open |
| SH-04 | Structural margins | Analysis (FEM) and test | — | Open |
| SH-05 | Outgassing | Analysis (materials data) | — | Open |
| SH-06 | Disposal and ephemeris | Analysis (DAS) | — | Open |

## 15. Certifications and Letters

| Item | Source | Status |
|------|--------|--------|
| FAA payload review information template | UG v11 Appendix G | Not started |
| Payload licensing certification (licences, hazardous materials per 49 CFR 172.101, registration) | UG v11 Appendix F | Not started |
| Encapsulation and launch readiness letters (RBF removed, ABF installed, battery charge state) | UG v11 Appendix D | Not started |
| Environmental test approach worksheet and verification report | UG v11 §6.4, Appendix G | Not started |

## 16. Waivers

None requested. Range waivers are a last resort (UG v11 §11.3).

## 17. Open Items

Every section above is open. The first steps are: sign the launch contract and obtain the integrator's safety
requirements; obtain SSCMAN 91-710 as tailored; draw the inhibit diagram; assemble the battery data package.

## 18. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 0.1 | 2026-10-16 | SMART-QSO Team | Skeleton created (funding audit VV-20), structured on SpaceX Rideshare Payload User's Guide v11 as the interim provider |
