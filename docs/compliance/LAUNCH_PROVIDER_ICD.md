# Launch Provider Interface Control Document
## SMART-QSO CubeSat Mission

> **Status (2026-10-27): not an agreed ICD.** No launch contract exists. The interface is written against the
> intended provider chain (SpaceX Transporter-class rideshare, Exolaunch, EXOpod Nova; `docs/ORBITAL_ACCESS.md`) and
> will be replaced by the integrator's mission ICD. No value here is measured: every value is design intent, an
> estimate or an analysis, as its row says. Governing values: `docs/BASELINE_PARAMETERS.md` Rev E (amended), §3, §4, §8. The requirements and their verification are in
> `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001); the CDS clauses in `CDS_COMPLIANCE_MATRIX.md`.

**Document Number:** SQSO-ICD-010
**Revision:** G (draft)
**Date:** 2026-10-27
**Reference:** SpaceX Rideshare Payload User's Guide Version 11 (August 2026, current on 2026-10-27); Cal Poly CDS
Rev 14.1 (2022-02-09); Exolaunch EXOpod Nova User Manual Rev 1.2 (June 2024); Exolaunch EXOpod User Manual Rev 6.0
(2022-09-06); NRCSD IDD NR-NRCSD-S0003 Rev "-" (2018, backup only)

---

## 1. Purpose and scope

The interface between the SMART-QSO 1U CubeSat and its dispenser, integrator and launch vehicle: mechanical,
electrical, environmental, RF, operational and documentation. Interface values only; the requirement flow-down with
sources and verification is SQSO-LVR-001.

## 2. Parties

| Role | Organization | Status |
|------|--------------|--------|
| Launch vehicle provider | SpaceX (Falcon 9, Transporter-class dedicated rideshare, VSFB SLC-4E) | Through the integrator only |
| Integrator and dispenser provider | Exolaunch (lead request for quotes); ISISPACE and SEOPS as alternatives | Requests for quotes due 2026-12-31 |
| Spacecraft operator | SMART-QSO; FCC Part 97 licensee KK6GDB, station KK6GDB-11 | The contracting entity is not yet named: the PI and licensee (KK6GDB) names it by 2026-12-31 (SQSO-OAP-001 DP-1; RID LV-09) |

## 3. Applicable documents

| Document | Title | Use |
|----------|-------|-----|
| SpaceX UG v11 | Rideshare Payload User's Guide, Version 11, Aug 2026 | Environments, verification, space safety, schedule |
| CDS Rev 14.1 | CubeSat Design Specification | Envelope, electrical safety, timers |
| EXOpod Nova UM Rev 1.2 | Exolaunch EXOpod Nova User Manual | Dispenser interface, 1U limits |
| EXOpod UM Rev 6.0 | Exolaunch EXOpod User Manual | 1U adapter and spring data |
| SSCMAN 91-710 | Range Safety User Requirements (as tailored by SpaceX) | Hazardous systems |
| NR-NRCSD-S0003 | NRCSD IDD | Backup path only |

## 4. Mechanical interface

### 4.1 Envelope, mass and mass properties

| Parameter | Requirement | SMART-QSO value | Basis | Status |
|-----------|-------------|-----------------|-------|--------|
| Form factor | 1U | 1U in a 3U Nova slot with a non-deployable adapter, or sharing a 3U slot with other 1U/2U spacecraft | NOVA §2.2, §3.4 | Slot type open |
| Rail length (Z) | 113.5 mm (Nova ±0.5 mm) | 113.5 mm | Bought frame | Not measured (VI-LV-01) |
| Rail width (X, Y) | 100.0 ± 0.1 mm | 100.0 mm | Bought frame | Not measured |
| Protrusion from the rail plane | CDS ≤ 6.5 mm (Nova allows ≤ 25 mm) | Designed ≤ 6.5 mm on the side faces (wings, RBF). The stowed AntS module is 7 mm thick on a ±Z face (BUILD_VS_BUY §2.6): check it against the CDS ±Z standoff height (0.5–7.0 mm, CDS Appendix B) and the switch positions | CDS §2.2.3, Appendix B | Open; not measured |
| Mass | Nova 1U ≤ 2.5 kg; CDS typical 2.00 kg; design limit 2.00 kg (SYS-002) | CBE **1.51 kg** (1,514 g), MEV **1.75 kg** (1,748 g) with the CubeSpace CR0002 rods, the CAD heat-spreader pedestal and the 20 g inertia ballast (`docs/environment/MASS_BUDGET.md` v3.1; BASELINE Rev E §4) | NOVA Table 1; CDS Table 1 | Estimate (bottom-up MEL, carried exactly by the CAD); MEV margin +14.4 % on 2.00 kg, 30 % on the Nova's 2.5 kg |
| Mass reported at L-8 M | ± 15 % | **1.75 kg ± 15 % (1.75 ± 0.26 kg, 1.49–2.01 kg)**, the MEV, so that the ± 5 % and as-measured values stay inside it (MASS_BUDGET v3.1 §3.3; SQSO-LVR-001 LVR-005) | UG Table G-3 | Proposed |
| CG | ±20 mm (CDS: ±2 cm) from the geometric centre | Stowed, from the CAD (CDS frame): **(+0.1, −0.3, −1.3) mm CBE**, (+0.2, −0.5, −0.3) mm MEV; ±1.2 mm (1σ) for ±5 mm placement scatter (`hardware/mechanical/MECHANICAL_DESIGN.md` B §5.2; MASS_BUDGET v3.1 §5) | CDS §2.2.11; NOVA Table 2 | Analysis (CAD); measured at L-4 M (VI-LV-02) |
| Moments of inertia | Reported | From the CAD about the CG, tensor convention, ×10⁻³ kg·m² (Ixx, Iyy, Izz; Ixy, Ixz, Iyz): **stowed CBE 2.746, 3.052, 2.732; +0.012, −0.009, +0.032** (MEV 3.158, 3.509, 3.189; +0.013, −0.017, +0.038); **deployed CBE 2.846, 4.190, 3.970; +0.013, −0.009, −0.003** (MEV 3.267, 4.745, 4.535; +0.014, −0.017, +0.003), wings and four AntS elements out | `hardware/mechanical/MECHANICAL_DESIGN.md` B §5.2; MASS_BUDGET v3.1 §6.1 | Analysis (CAD); measured at L-4 M (VI-LV-02) |
| Hard-body radius after deployment (first 24 h) | Reported to SpaceX | 0.6 m (VHF monopole tip 0.55 m from the centre, wing tip 0.17 m) | UG §3.3.4; `software/simulation/cdr/launch/leop_feasibility.py` §7 | Analysis |

Revision D's measured-looking values (99.9 mm, 113.4 mm, 85 mm rail length, 0.8 µm Ra, CG 48/51/55 mm, protrusions
of 2–5 mm, RBF pull 3 N, 3 A switch rating) had no measurement behind them and are withdrawn. Revision F's MASS_BUDGET
v2.0 mass (1.92–2.00 kg), its 1.85 kg ± 15 % report and its box inertia estimate are replaced by the MEL v3.1 and the
CAD values above (Revision G).

### 4.2 Rails and contact

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Rail width to first protrusion | ≥ 8.5 mm (CDS); EXOpod ≥ 6.4 mm | Bought frame | Design |
| Rail finish | Hard anodized (Type III), Ra ≤ 1.6 µm; other finishes need Exolaunch's written approval (also for EMs in a TestPod) | Bought frame | Design |
| Rail edge radius | CDS ≥ 1 mm (should); NRCSD 0.5 ± 0.1 mm (backup) | Choose a frame with heritage in both | Open |
| ±Z rail ends | ≥ 6.5 × 6.5 mm bare contact | Bought frame | Design |
| Separation springs | Required when sharing a slot; < 6.7 N, > 2.5 mm stroke (guideline) | Four plungers in the −Z standoffs; force open (TBR-CDS-001) | Open |

### 4.3 Ejection

| Parameter | Value | Basis |
|-----------|-------|-------|
| Ejection velocity | 1.10–1.48 m/s for 1.51–2.00 kg in a 1U slot with its adapter (spring 2.19 J ± 20 %) | EXO Eq. 1, Tables 3–4; `leop_feasibility.py` §3 |
| Tip-off | < 10°/s expected (Nova); SMART-QSO detumbles from 10°/s | NOVA §2.5; BASELINE §8 |
| Insertion | −Z face first; deployables hinged so that a premature release cannot jam the slot | CDS §2.2.2 |

## 5. Electrical interface

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Electrical interface to dispenser or LV | None after integration (the Nova has no standard umbilical) | None | Design |
| Deployment switches | ≥ 1 (CDS); three for the NRCSD backup (IDD §4.1.4) | **Three switches on three rails**: DS1 and DS2 at the −Z rail ends, DS3 at +Z, flush when actuated (`hardware/mechanical/MECHANICAL_DESIGN.md` §6.4); each a changeover contact whose closure drives one FET gate through a ≥ 10 ms RC filter (an 11.7 ms closure before conduction), never the bus (`hardware/electrical/eps/EPS_BOARD_DESIGN.md` §6.2); DS2 two-pole. Switch part and the OBC sense polarity: TBR-CDS-005 | Design (switch part open) |
| Independent RF inhibits | ≥ 3 physical (CDS §2.3.7) | **Three, on three rails**: DS2 opens the low-side FET pair in the pack return (the ground leg), DS1 a high-side FET in VBAT+, DS3 a second high-side FET in VBAT+; **the array path is broken by the same three**: DS1 and DS3 in series in it and DS2's second pole holding the trackers' enables low (DS3's NO contact holds them too), so with any two failed closed no rail, no RF and no deployment (`hardware/electrical/eps/EPS_BOARD_DESIGN.md` §6.2–§6.3; BASELINE Rev E §4). RBF and timers not counted | Design; inhibit-by-inhibit test VI-LV-06, V-EPS-3 |
| Independent deployable inhibits | ≥ 3 physical per deployable (CDS §2.3.8) | The same three (DS1–DS3), which remove the burn supply and the AntS rail for every deployable; the burn-driver load switch and the AntS ARM are a fourth, not counted | Design; VI-LV-06 |
| RBF pin | Cuts all power; ≤ 6.5 mm; removed before integration unless under a Nova access window | Removed at integration (Nova window positions: NOVA Fig. 37; position against them open) | Open |
| Powered while integrated | Only the battery-side protection (CDS §2.3.1.2; UG §7.6) | BMS on the battery side | Design |
| Battery-side drain while integrated | ≤ 20 µA (sets the storage limit, LVR-071) | Not measured (VI-LV-14) | Open |
| Charging after integration | None (scenario A of UG Appendix D) | None | Design |

## 6. Environments

The SpaceX maximum predicted environments apply at the plate (UG v11 §4) until the integrator gives
dispenser-derived levels. Test levels for a fully containerized CubeSat: UG Table 6-2.

| Environment | Level | SMART-QSO verification | Status |
|-------------|-------|------------------------|--------|
| Random vibration | MPE 5.57 Grms (20–2000 Hz); protoflight MPE 1 min/axis; qualification MPE + 3 dB = 7.87 Grms, 2 min | **Protoflight at 8.18 Grms**, the envelope of MPE + 3 dB and the GSFC-STD-7000B workmanship spectrum, 1 min/axis, flight unit in a TestPod, with power-inhibit monitoring (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C; VI-LV-04); exceeds the UG minimum | Not tested |
| Natural frequency | No elastic mode below 40 Hz | FE first mode **169 Hz** stowed (`hardware/mechanical/MECHANICAL_DESIGN.md` §8; `docs/environment/STRUCTURAL_ANALYSIS.md` B); low-level sine sweep before and after (VI-LV-03) | Analysis |
| Quasi-static | 10 g axial, 17 g lateral (dispenser assembly); no CubeSat test required | FE analysis at the larger of 10 g / 17 g and the 3σ random response at 8.18 Grms, with NASA-STD-5001B protoflight factors (1.25 yield / 1.40 ultimate, above the UG's 1.10 / 1.25): all margins positive, lowest +1.00 yield (STRUCTURAL_ANALYSIS B; MECHANICAL_DESIGN §8.5) | Analysis |
| Sine | 1.4 g axial, 1.5 g lateral, 5–100 Hz; not required | — | Not required |
| Shock | 30 g at 100 Hz to 1000 g at 1–10 kHz (Q = 10); advised | Not tested; risk acknowledged | Acknowledged |
| Acoustic | OASPL 139.3 dB; not required | — | Not required |
| Fairing pressure decay | ≤ 2.8 kPa/s (4.5 kPa/s ≤ 5 s); volume / vent area ≤ 400 m (UG), ≤ 50.8 m (CDS) | Vent analysis; ≥ 22.3 mm² of vent for the gross 1U volume (VI-LV-12) | Open |
| Flight thermal | Conductive boundary −20 to +69 °C to 7200 s; fairing radiative profile; FMH < 3,500 W/m² | TVAC per ETP Rev C: allowable flight temperatures ± 10 °C, hot plateaus capped at +60 °C by the battery, 8 vacuum cycles (8 ambient added if the integrator requires 20); bake-out +60 °C, 6 h, ≤ 1·10⁻⁴ Torr (VI-LV-08) | Not tested |
| Ground | PPF 15–25 °C, 30–65 % RH, ISO Class 8, ≤ 45 d; hangar ≤ 40 d; pad 15–30 °C, ≤ 5 d | Battery at 60 % SOC; materials and humidity compatible | Analysis |
| EMI susceptibility | LV 120 dBµV/m; launch site 132–150 dBµV/m (26 dB facility shielding assumed) | Unpowered with mechanical inhibits | Design |
| EMI emissions while integrated | ≤ 108 dBµV/m 30–1000 MHz (test); 33 dBµV/m in the GNSS bands | Unpowered: no emitter | Design |

The CubeSat Nova manual notes that the Nova amplifies vibration near its first natural frequency and damps it above,
and damps shock strongly at high frequency (NOVA §3.5); transfer functions are available from Exolaunch on request.

## 7. RF interface

| Item | Requirement | SMART-QSO | Basis |
|------|-------------|-----------|-------|
| Transmission while integrated | None (UG §4.1.7.3, §9.5.2); no open-air RF checkout | Radio unpowered by the inhibits; conducted checks into a load only | Design |
| RF after separation | Not before 45 min (CDS §2.4.5); UG Table 4-10 minimum ≈ 7.5–8.6 s at 1.6 W EIRP and 1.0–1.2 m/s | First RF at 45:00 min (`rf_silence.rs`; simulation 45.0 min) | Analysis |
| Deployables | Not before 30 min (CDS §2.4.4) and not before 120 s (UG §3.3.4) | First burn at 30:00 min (`deployment.rs`) | Analysis |
| Attitude maneuvers | Not before 120 s (UG §3.3.4) | No magnetorquer output until 120 s after separation (SRS-F086); the mission simulation starts B-dot at S+121 s (RID LV-02 closed) | Met (design; simulation) |
| Downlink | One IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz | SYS-200 | BASELINE §7 |
| Uplink | One IARU-coordinated frequency in 435–438 MHz | — | BASELINE §7 |
| Power | 1 W maximum, 0.5 W nominal; spurious ≤ 25 µW and ≥ 40 dB below carrier (47 CFR 97.307(e)) | SYS-201, SYS-203 | BASELINE §7 |
| Transmitter verification | SpaceX webform at L-8 M | VI-LV-10 | UG Table G-3 |

## 8. Materials and contamination

| Item | Requirement | SMART-QSO | Status |
|------|-------------|-----------|--------|
| Outgassing | TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS §2.1.7); RML ≤ 1.0 % with rationale codes (UG Table 6-7) | `docs/compliance/MATERIALS_LIST.md` (sources to add) | Open (VI-LV-09) |
| Prohibited materials | Cadmium, zinc plating, mercury, pure tin or tin electroplate (UG Table 5-5; applies to containerized CubeSats too) | Applied: terminal-finish policy of `docs/manufacturing/EEE_PARTS_LIST.md` Rev E §2A.3 (Sn-Pb, NiPdAu or gold preferred; pure-tin terminations re-tinned with Sn63Pb37 or conformally coated; Sn63Pb37 assembly), every connector in its gold finish (§2A.8; `hardware/harness/HARNESS_DESIGN.md`), ENIG boards; no tin-plated shields, no cadmium or zinc plating | Design (RID LV-26); declaration at L-4 M (VI-SAF-07) |
| Contamination report | Vacuum-exposed non-metallic materials with quantities (UG §6.7.12) | To prepare | Open |
| Bakeout | Per the launch provider (CDS §3.2) | In the TVAC campaign | Open |
| Markers | No Sharpie; Mighty Marker MM07/MM08 if needed (UG Table 5-5) | Assembly procedure | Open |

## 9. Operational interface

### 9.1 Schedule

The full deliverable schedule, with dates for the planning launch of about 2028-11 (BASELINE Rev E §3; 2028-11-01 in `mission/SCHEDULE.md`), is SQSO-LVR-001 §9. Key interface
events:

| Event | Timing | Notes |
|-------|--------|-------|
| Mission analysis inputs (mass ± 15 %, CAD, dynamic model, transmitter webform, test approach) | L-8 M | UG Table G-3 |
| Orbit selection (≤ 485 km must be offered; target 475 km) | L-5 M | UG Appendix G; BASELINE §3 |
| Environmental verification report; mass ± 5 % | L-3.5 M | UG Table G-3 |
| Licences, 18th SDS registration, Appendix F letter, ephemeris capability | L-2 M | UG §9.5.6, §9.5.7 |
| Flight model at the integrator | About L-50 d (set by the mission ICD) | — |
| Final charge, key load, `ARM_FLIGHT`, RBF removal, integration | About L-47 d | The simulation's integration date |
| Dispenser at the launch site | ≤ 8 weeks before launch | UG §9.5.1 |

### 9.2 Access

| Phase | Access |
|-------|--------|
| Before integration | Umbilical: functional checks, final charge (RBF in), key load, `ARM_FLIGHT` last |
| After integration | **None.** The Nova access windows give physical access only; no charging or checkout is planned through them (NOVA §4.2.5, §5.4). SpaceX gives no connectivity after mate except optional services (UG Table 7-4) |

### 9.3 State at integration

| Parameter | State |
|-----------|-------|
| Power | Off: separation switches actuated by the slot; RBF removed at integration |
| Software | Flight release loaded; deployment sequence and RF silence ARMED by `ARM_FLIGHT` |
| Keys | Operational and recovery keys loaded through the umbilical (SQSO-ICD-013) |
| Battery | **60 % ± 5 % SOC** after the final charge at the integrator. Revision D's 50 % ± 10 % and the launch-site plan's 30 % / 50–70 % are replaced. The separation SOC must stay ≥ 40 % (LVR-071) |
| Time | A mission clock is fitted (Micro Crystal RV-3028-C7 on the OBC, supplied by 3V3_RTC from the EPS's 3V3_AO rail), but that rail is downstream of the deployment-switch inhibits, so the clock is unpowered from integration to separation; the onboard time starts at separation (the clock's power-loss flag set). UTC comes from the GNSS receiver or ground correlation of the first frames. Revision D's "within 1 s of UTC" cannot be met and is withdrawn |

## 10. Documentation

| Document | Due | Status |
|----------|-----|--------|
| This ICD → the integrator's mission ICD | Contract | Draft |
| Mass properties (± 15 %, then ± 5 %, then as-measured) | L-8 M, L-3.5 M, L-2 M | MEL v3.1 with the CAD mass, CG and full tensors (§4.1); report 1.75 kg ± 15 % |
| Environmental test approach; verification report | Before test; L-3.5 M | Not started |
| SDHS and SSCMAN 91-710 compliance letter | L-6 M | Skeleton: `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` |
| Ground Operating Plan and hazardous procedures | L-2 M | None needed at the site if no charging is done there |
| Licensing certification letter (Appendix F), licence copies | Before shipment; L-2 M | FCC and IARU not filed |
| FAA payload review template, cross-waivers | L-6 M | Contracting entity needed |
| Insurance declarations and waivers of subrogation | Per SOW | Contracting entity needed |
| Export classification and agreements | Kickoff | `docs/regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md`; counsel |
| Contamination report, materials list | L-4 M | Open |
| Encapsulation and launch readiness inputs (Appendix D) | At integration; before rollout | — |

## 11. Waivers and deviations

None requested. Expected: the integrator's acceptance of 60 % SOC at integration in place of "fully charged"
(UG Appendix D). (The magnetorquer deviation of Revision F is no longer needed: RID LV-02 is closed.)

## 12. Points of contact

To be named in the contract: mission manager and systems engineer (SMART-QSO), integrator mission manager, safety.

## 13. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | Document number changed from SQSO-ICD-004, which is the OBC-to-ADCS ICD (`docs/interfaces/ICD_OBC_ADCS.md`), to SQSO-ICD-010, the number `INTERFACE_CONTROL_MASTER.md` reserves for the launch vehicle interface (finding PI-17). No content change | SMART-QSO Team |
| C (draft) | 2026-10-16 | Funding audit VV-21: umbilical access made consistent (no access after integration unless the dispenser has ports; RBF out at integration). VV-20: deliverable statuses corrected (no "Complete"); safety data package is a system-level skeleton. TB-15: one environment specification from SpaceX UG v11 (interim) in §4.5 and §6.2, replacing unsourced "Qualified" and "Compliant" entries; rail edge radius per CDS §2.2.7 | SMART-QSO Team |
| D (draft) | 2026-10-16 | §4.1 mass from MASS_BUDGET v2.0 (was 1.075 kg, never weighed; TB-06); §5.1 switch type open as TBR-CDS-005 (was "normally open", against "normally closed" in the structural design); §7.1 timer 45 min (SYS-224; was 30 min); §7.2 frequency, power and spurious limits from SYS-200, SYS-201 and SYS-203 (were 145.825 MHz, 2 W, "Per FCC Part 97") | SMART-QSO Team |
| D, note | 2026-10-24 | RF-silence references name the Rust file that replaced `rf_silence.c` | SMART-QSO Team |
| E (draft) | 2026-10-27 | CDR launch review: written against the intended chain (SpaceX Transporter from VSFB, Exolaunch, EXOpod Nova 1U slot) with UG v11 confirmed current; requirements moved to SQSO-LVR-001; measured-looking values withdrawn; deployer list (P-POD Mk III, ISIPOD) replaced by the EXOpod Nova; ejection velocity, hard-body radius, turn-on delay and vent area from `software/simulation/cdr/launch/`; the 120 s rule and the prohibited-materials row added; SOC at integration 60 % ± 5 %; "time within 1 s of UTC" withdrawn; delivery schedule from UG Fig. 1-2 and Appendix G instead of L-60/L-14 days | CDR review (LV) |
| F (draft) | 2026-10-27 | CDR 2026 consistency pass: planning launch about 2028-11 (BASELINE Rev E §3) and orbit selection ≤ 485 km in §9.1 | CDR documentation review |
| G (draft) | 2026-10-27 | CDR 2026 round 3 (RIDs LV-11, P3A-02, P3E-04, P3S-06), aligned to BASELINE Rev E (amended): mass CBE 1.51 / MEV 1.75 kg from MASS_BUDGET v3.1 and the L-8 M report 1.75 kg ± 15 %; CG and full stowed and deployed inertia tensors from the CAD (MECHANICAL_DESIGN B §5.2); three deployment switches DS1–DS3 as three inhibits on three rails for the pack and the array (EPS_BOARD_DESIGN §6.2); protoflight random 8.18 Grms, FE first mode 169 Hz and positive margins, TVAC per ETP Rev C; magnetorquers from 120 s (LV-02 closed); prohibited materials applied (LV-26); the RV-3028 mission clock, unpowered in storage; contracting-entity owner; the LV-02 deviation withdrawn | CDR round 3, launch and operations |
