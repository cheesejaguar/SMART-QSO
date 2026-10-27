# Materials and Outgassing Compliance List
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MAT-001
**Revision:** E
**Date:** 2026-10-27
**Reference:** CDS Rev 14.1 §2.1.7; SpaceX Rideshare Payload User's Guide v11 (Aug 2026) §5.3, Table 5-5, §6.7.12, Table 6-7; NASA-STD-6016C with Change 1 (2023-11-15), tailored for Class D (`docs/project/MISSION_ASSURANCE_PLAN.md` §7); ASTM E595-15(2021)
**Status:** CDR draft. **Not compliant yet:** one known non-compliant material, several values without a source, and the terminal finishes of the EEE parts not surveyed. Revision A's "48 of 48 compliant, 100 %" had no sources and is withdrawn

---

## 1. Purpose

List every material of the flight unit with its outgassing data (ASTM E595: TML, CVCM, and WVR where known), the source
of each value, and its status against the requirements below, and record the materials the launch provider prohibits.
This list feeds the SpaceX contamination report (UG v11 §6.7.12, VI-LV-09) and the hazard analysis (SQSO-SDP-002 HR-06).

## 2. Requirements

### 2.1 Outgassing

| Source | Requirement |
|--------|-------------|
| CDS Rev 14.1 §2.1.7 | TML ≤ 1.0 %, CVCM ≤ 0.1 % (ASTM E595) |
| UG v11 Table 5-5, Non-Metallic Materials | RML ≤ 1.0 % (RML = TML − WVR; WVR = 0 if unknown) and CVCM ≤ 0.1 %; exceptions by the rationale codes of Table 6-7 (A: RML < 3 %, CVCM < 0.1 %, exposed area < 12.9 cm²; B: exposed area < 1.6 cm²; C: thermal-vacuum stability shown in configuration; D: hermetically sealed, ≤ 10⁻⁴ sccs; E: other rationale) |
| UG v11 §5.3 | **Fully containerized CubeSats are exempt** from the visual-cleanliness, non-metallic, metallic and silicone-sensitivity rows of Table 5-5 (they apply to the dispenser assembly). They are **not** exempt from the prohibited-materials or particulate-generation rows. The CDS §2.1.7 limit applies regardless |

Rule for this project: every non-metallic material meets the CDS limits by ASTM E595 data from the NASA GSFC outgassing
database, the manufacturer's E595 certificate, or a project test; otherwise it is replaced or carries a UG v11 rationale
code agreed with the integrator.

### 2.2 Prohibited materials (UG v11 Table 5-5; apply to SMART-QSO)

| Prohibited | Where it could enter the design | Control |
|------------|---------------------------------|---------|
| Cadmium parts and cadmium plating | Aerospace fasteners and inserts (A286 and steel screws are commonly cadmium-plated), connector shells, some terminal lugs | Fasteners specified passivated (ASTM A967) or silver-plated A286, never cadmium; receiving inspection with an XRF check of a sample of each lot |
| Zinc plating | Commercial steel hardware, standoffs, some connector backshells, chromate-zinc finished brackets | Stainless or anodized/chromate-converted aluminium hardware only; no zinc-plated commercial hardware |
| Mercury and mercury compounds | Tilt switches, some relays and old thermostats | None used; the heater cut-out must be a bimetal or solid-state device, not a mercury switch |
| Pure tin or tin electroplate (unless alloyed with Pb, Sb or Bi) | **Most COTS EEE terminations (matte tin)**, RF shields and cans, connector contacts, solder lugs, the Jetson module and its connector | Terminal-finish survey of every EEE part (EEE_PARTS_LIST Rev C §2.3); pure-tin finishes mitigated by hot-solder-dip re-tinning with Sn63Pb37 (or replaced), or, where re-tinning is impractical (fine-pitch, the Jetson module), by conformal coating per §5 over the termination, documented as a deviation for the integrator; assemblies soldered with Sn63Pb37 (J-STD-001 with the space addendum). SAC305 lead-free solder is not used on flight boards |

### 2.3 ISS backup only

On the NRCSD path the CubeSat travels in a crewed cabin: flammability (NASA-STD-6001B Test 1 or a configuration
analysis) and offgassing toxicity (Test 7), with a materials usage agreement for exceptions (SQSO-SDP-002 Appendix B).
Not assessed here.

## 3. Structural and Mechanical Materials

| Component | Material | TML / CVCM (%) | Source | Status |
|-----------|----------|----------------|--------|--------|
| 1U frame and rails (bought) | Al 7075-T6 or 6061-T6, hard anodized rails | Metal | — | Compliant (vendor finish to confirm: no zinc or cadmium) |
| Brackets, battery clamp plates | Al 6061-T6, clear chromate conversion or bare (BUILD_VS_BUY §4.3) | Metal | — | Compliant |
| Fasteners | A286 or 300-series stainless, passivated | Metal | — | Compliant if not cadmium-plated (§2.2) |
| **Battery-mount screws** | **Ti-6Al-4V** (M2.5, BUILD_VS_BUY §4.3) | Metal | — | Compliant. Specify a dry-film lubricant or anti-galling coating (Ti galls in Ti or stainless threads) and Al helicoil or stainless inserts in the frame; galvanic couple Ti/Al acceptable with the conversion coat |
| **Battery-mount isolator bushings** | **G-10CR** (MIL-I-24768/27, preferred) | **0.35 / 0.02** | NASA GSFC outgassing database, as quoted in BUILD_VS_BUY §4.3 | Compliant |
| Alternative bushings | **PEEK 450G** (Victrex) | **0.26 / 0.00** | NASA GSFC outgassing database, as quoted in BUILD_VS_BUY §4.3 | Compliant |
| Rejected bushing material | Vespel SP-1 | Fails TML | BUILD_VS_BUY §4.3 | Not used |
| Wing hinge pins (SQSO-MECH-001 D §3A, §5A) | **A286** (AMS 5732, passivated, not cadmium-plated), non-ferromagnetic (magnetometer zone) | Metal | — | Compliant |
| Hinge lubricant | Sputtered MoS₂ on the pin; burnished, binder-free MoS₂ in the bore | Inorganic dry film, no binder | — | Compliant (no organic content); VI-MECH-03 for friction and wear |
| Hinge bushings and frame ears | Ti-6Al-4V bushings; Al 6061-T6 ears | Metal | — | Compliant |
| Deployment springs | 17-7PH CH900 wire (SQSO-MECH-001 §3A) | Metal | — | Compliant (ferromagnetic: the W1 secondary spring is an accepted fixed bias for the magnetometer, SQSO-MECH-CAD-001 §4.4) |
| Hold-down line series spring and seat (SQSO-MECH-001 E §6; SQSO-MECH-CAD-001 §6.6, part `W1_line_spring` / `W2_line_spring`) | 17-7PH CH900 compression spring (AMS 5678 wire, passivated, not cadmium-plated), 17-7PH or A286 washer, Ti-6Al-4V seat angle, PTFE-lined line guides | Metal; PTFE | — | Compliant (17-7PH is ferromagnetic: on the burn boards at (±45.4, −37.5, −28) mm, about 100 mm from the magnetometer board A2 and outside its 20 mm zone; its bias is part of the hard-iron calibration, SQSO-MECH-CAD-001 VI-MCAD-07) |
| End-of-travel stop and snubber | Damped silicone bumper; PTFE-faced silicone snubber | **Not yet sourced** | Vendor | **Open**: E595 data of the silicone grade (space-grade, e.g. a low-volatility RTV, certificate required) |
| Magnetometer board A2 fasteners (SQSO-MECH-CAD-001 §4.4) | Ti-6Al-4V M2 screws, phosphor-bronze (C51000) nuts, polyimide washers | Metal; polyimide film | — | Compliant (metals); polyimide washer TML is water-dominated (use RML per UG v11) |
| Stack column stud at (+40, +43) | Ti-6Al-4V M3 (non-ferromagnetic zone), Ti or phosphor-bronze nut | Metal | — | Compliant |
| Iyz trim masses (D-9, 2 × 5 g at y ±45 mm on the +Z end frame) | W-Ni-Cu tungsten heavy alloy (ASTM B777, non-magnetic Ni-Cu binder, relative permeability ≤ 1.05 on the certificate) or pure tungsten; not W-Ni-Fe (ferromagnetic), not steel | Metal | — | Compliant |
| Battery cell retention (MEL row 37; SQSO-MECH-CAD-001 §4.6, RID STR-08) | G-10CR cell-locator plates and spring seats (MIL-I-24768/27), polyimide (Kapton HN) liners and end insulators, 17-7PH disc springs, stainless shims; **no adhesive, no potting** | G-10CR 0.35 / 0.02; polyimide film | NASA GSFC outgassing database (G-10CR) | Compliant (polyimide TML water-dominated: RML per UG v11) |
| Inertia ballast, 2 × 10 g discs Ø14 × 3.6 mm on the +Z end frame (MEL row 36, chair decision C-13; SQSO-MECH-CAD-001 D-9) | Tungsten heavy alloy W-Ni-Cu, ASTM B777 Class 3 (95 % W, 17.75–18.35 g/cm³) with the non-magnetic Ni-Cu binder (not W-Ni-Fe, which is ferromagnetic; relative permeability ≤ 1.05 to be on the certificate); M2 Ti-6Al-4V screws | Metal | — | Compliant |
| AntS and GNSS patch isolated mounts (SQSO-MECH-CAD-001 D-13; SQSO-HAR-001 2.0 M-9) | Polyimide (Kapton HN) or G-10CR washers and sleeves under every fastener, isolated faying faces (≥ 1 MΩ to the frame) | Polyimide film; G-10CR 0.35 / 0.02 | NASA GSFC outgassing database (G-10CR, as for the battery bushings) | Compliant (polyimide TML is water-dominated: RML per UG v11) |
| Shear-panel slot grommets (SQSO-MECH-CAD-001 §6.11, M-2) | Silicone grommet or PTFE edge strip | Silicone: **not yet sourced**; PTFE compliant | Vendor | **Open** for silicone (E595 certificate of the grade); PTFE preferred |
| ±Z sun-sensor carriers (SQSO-MECH-CAD-001 §6.11, M-5) | FR-4 (polyimide preferred) carrier, bonded with the harness design's tie-down adhesive (Scotch-Weld 2216) | As the PCB laminate and the adhesive of the harness design | SQSO-HAR-001 2.0 §5.3 | As those entries (harness request P-2) |
| Gyro window in the Jetson carrier | **No bonded frame**: the window is in a 2.0 mm carrier board (SQSO-MECH-CAD-001 §4.3.3); no structural adhesive is used | — | — | Not applicable |
| Wing restraint line | Dyneema or Vectran cord (panel vendor's choice) | Not yet sourced | Vendor | **Open**: vendor E595 data |

## 4. Printed Circuit Boards and Solder

| Component | Material | TML / CVCM (%) | Source | Status |
|-----------|----------|----------------|--------|--------|
| PCB laminate | FR-4 (IPC-4101/126 class, high-Tg) or polyimide (IPC-4101/40) | Not yet sourced | Laminate grade not selected | **Open**: select a laminate with E595 data in the GSFC database |
| Solder mask | Liquid photoimageable (e.g. Taiyo PSR-4000 series) | Revision A's 0.78/0.02 had no source | — | **Open** |
| Solder | Sn63Pb37 (J-STD-006) | Metal | — | Compliant; required for tin-whisker control (§2.2) |
| Solder (lead-free) | SAC305 | — | — | **Not used** on flight boards |

## 5. Conformal Coating, Staking and Potting

| Component | Material | TML / CVCM (%) | Source | Status |
|-----------|----------|----------------|--------|--------|
| Conformal coat (preferred) | Arathane 5750 A/B (LV) polyurethane (formerly Uralane 5750) | Not quoted here | Qualified for flight boards by JPL (NASA NTRS 19920069306); the E595 values were not retrievable from the review environment | **Open**: E595 certificate with the cure schedule used |
| Conformal coat (alternate) | HumiSeal 1B73 acrylic | No E595 data found | Revision A's 0.32/0.01 had no source | **Open** |
| Conformal coat (alternate) | Dow 1-2577 silicone | Revision A's 0.18/0.03 had no source | — | **Open**; silicone: integrator coordination not needed for a containerized CubeSat (UG v11 §5.3) |
| Staking, component and harness | Arathane 5750 or 3M Scotch-Weld 2216 B/A Gray | 2216 Gray: **0.77 / 0.04** (7 days at 25 °C cure) | 3M Scotch-Weld 2216 B/A technical data sheet (values cited from NASA RP-1124 Rev 4); Revision A's 0.65/0.02 is corrected | Compliant (2216); Arathane open. **Staking only, never structural** (no structural adhesive in the design): in the harness (SQSO-HAR-001 2.0 §10.3, §5.3) 2216 makes the tie-down dots on structure and stakes wires at solder ends and splices; Arathane 5750 stakes mated wire-to-board connectors (removable bead) |
| Silicone encapsulant (solar cells, if used by the panel vendor) | DOWSIL 93-500 Space Grade | ≈ 0.25 / 0.05 reported | Secondary source (a patent document); Dow states it meets NASA E595 requirements | **Open**: lot certificate from the panel vendor |
| Potting (if needed) | Loctite Stycast 2850FT with Catalyst 9 | **0.50 / 0.01** (24 h at 25 °C) | Henkel, *NASA Outgassing Approved Products*, DSGN0007528 (09/2020) | Compliant |
| Electrically conductive adhesive (if needed) | Loctite Ablestik 84-1LMI | **0.12 / 0.00** | Henkel DSGN0007528 | Compliant |
| **Thread-locker** | **Loctite 242** (Revision A) | **Not on Henkel's NASA outgassing-approved list; no E595 data found** | Henkel DSGN0007528 | **Non-compliant until data exist: replace.** Use prevailing-torque or locking inserts plus staking with Scotch-Weld 2216, never thread-locker alone on the polymer-bushed battery mount (SQSO-SDP-002 HR-05) |

## 6. Thermal Materials

| Component | Material | TML / CVCM (%) | Source | Status |
|-----------|----------|----------------|--------|--------|
| Gap pad (Jetson, battery) | Bergquist Gap Pad TGP 3000 (formerly Gap Pad 3000) | **0.65 / 0.03** | Henkel DSGN0007528; Revision A's 0.40/0.02 is corrected | Compliant |
| Gap pad (lower outgassing alternative) | Bergquist Gap Pad TGP 3500ULM | **0.07 / 0.02** | Henkel DSGN0007528 | Compliant; preferred next to the GNSS patch and sun sensors |
| Thermal grease | Dow 340 | Revision A's 0.09/0.00 had no source | — | **Open** (silicone grease: migration risk to optics; prefer a pad) |
| Thermal tape | 3M 8810 | Revision A's 0.55/0.02 had no source | — | **Open** |
| Low-ε tape on the battery enclosure | **VDA or aluminised polyimide tape** (e.g. Sheldahl aluminised polyimide with acrylic adhesive) (BUILD_VS_BUY §4.3) | Not yet sourced | Vendor certificate | **Open**: Sheldahl or equivalent E595 data for the film and adhesive together |
| Heater | Polyimide film heater (≈ 200 Ω) with acrylic or epoxy adhesive | Not yet sourced | Vendor | **Open** |

## 7. Wire, Cable and Harness

Harness design SQSO-HAR-001 2.0 (`hardware/harness/HARNESS_DESIGN.md` §5.3); verification V-HAR-11 (E595 data of every
harness material and a bake-out of the finished harnesses before flight integration).

| Component | Part number / material | TML / CVCM (%) | Source | Status |
|-----------|------------------------|----------------|--------|--------|
| Discrete wire, all gauges (22, 24, 28 AWG) | **SAE AS22759/33** (M22759/33-22, -24, -28): crosslinked ETFE, silver-coated high-strength copper alloy, 200 °C | Per part number and lot | No value is quoted here: crosslinked ETFE is a low-outgassing fluoropolymer, but no E595 entry per part number was retrieved | **Open**: E595 entry (GSFC database) or the wire maker's certificate for each of the three part numbers (V-HAR-11) |
| Discrete-wire assemblies W3, W4A, W8A | **Samtec SFSDT** (Tiger Eye), PTFE-insulated 30 AWG, with its socket housing | Per assembly | No Samtec E595 data retrieved | **Open**: Samtec outgassing declaration for SFSDT wire and housing (V-HAR-11) |
| Ribbon cable (FFSD IDC), withdrawn | Samtec FFSD: PVC insulation, tin-plated copper | PVC typically above the limits | Distributor data (PVC, −40 … +105 °C) | **Not used**: replaced by SFSDT in SQSO-HAR-001 2.0 |
| Coax W10 | 1.37 mm micro-coax, FEP insulation and jacket, silver-plated copper (candidate Taoglas PRT.001221) | Per part number | RF board design §4.3 requires TML ≤ 1 %, CVCM ≤ 0.1 % | **Open**: cable maker's certificate (V-HAR-11) |
| Coax W10E (GNSS, 0.195 m) | **Multicomp Pro MINI-1.13**: FEP jacket (grey) and FEP dielectric (0.68 mm), inner 7/0.08 mm and shield silver-plated copper; −55 … +155 °C (Multicomp Pro MINI-1.13 data sheet V1.0, 2020, Farnell copy) | No entry for this cable in the NASA outgassing database (searched 2026-10-27). FEP cable entries there: FEP cable jacket **0.01 / 0.00** (GSFC31882), FEP outer sheath 421-010 **0.01 / 0.00** (GSFC17714), foamed FEP coax insulation M44276F **0.02 / 0.00** (GSFC19120) | NASA GSFC outgassing database (etd.gsfc.nasa.gov) | **Open, expected compliant by material class**: E595 certificate from the maker for the procured lot, or a project E595 test of a 0.5 m sample; owner RF designer (PI until named), due 2027-01-15 (before the EM order). Fallback: the W10 cable maker's 1.13 mm FEP cable with a certificate |
| Splice sealing sleeve (W6C, W7A/W7B sensor leads, W7C; TE Raychem D-200-82, SQSO-MFG-002 §2A.8) | Radiation-crosslinked modified fluoropolymer heat-shrink sleeve (TE D-200 data sheet; meets SAE-AMS-DTL-23053/13) | **No entry** for D-200 (or D-436) splices in the NASA database. Related Raychem entries do not carry compliance to this part: Kynar sealing sleeve with white inserts **0.66 / 0.04** (GSFC08580, 1977); Kynar M23053/8 tubing 0.07 / 0.01 (GSFC26032) but a clear Kynar M23053/8 lot **0.62 / 0.13** (GSFC26494, fails CVCM) | NASA GSFC outgassing database | **Open**: TE E595 certificate on the shrunk splice (sleeve and rings together, the installed condition); owner harness designer (PI until named), due 2027-01-15 (before the EM harness order). Fallback: project E595 test of ten shrunk D-200-82 splices; if it fails, the UG v11 Table 6-7 rationale (exposed area) agreed with the integrator, or an uninsulated crimp splice under fluoropolymer heat shrink with E595 data (e.g. FEP shrink, GFC36834 0.00 / 0.00) |
| Splice meltable sealing rings (inside the D-200-82 sleeve) | Modified thermoplastic fluoro-elastomer (TE D-200 data sheet) | No entry in the NASA database | — | **Open**: covered by the same TE certificate or project test on the shrunk splice (row above); owner and date as above |
| Splice solder preform | **None**: AS81824/1 crimp splices (D-200-82, D-436-36) seal with two meltable rings and contain no solder preform (the SolderSleeve family does, and is not used) | — | TE D-436-36/-37/-38 drawing Rev F; TE D-200 data sheet | Not applicable |
| Sleeving (pack-lead over-sleeve, edge protection at the deck openings and slots, identification sleeves) | PTFE | Per part number | — | **Open** (data, V-HAR-11) |
| Heat shrink at splices and ends | PVDF (Kynar) or other fluoropolymer heat shrink (no polyolefin) | Per part number | — | **Open** (data, V-HAR-11) |
| Harness ties | **MIL-T-43435 lacing tape**, Nomex or polyester, with a finish that has E595 data; spot ties | Per part number and finish | — | **Open** (data, V-HAR-11). Revision A's nylon 6/6 tie wraps (0.89/0.03, no source) are **replaced** in SQSO-HAR-001 2.0 |
| Tie-down dots, wire staking | 3M Scotch-Weld 2216 B/A Gray (§5) | **0.77 / 0.04** | §5 | Compliant; staking only |
| Connector staking | Arathane 5750 A/B (§5) | Not quoted | §5 | **Open** (§5) |
| Polyimide tape | Kapton with silicone or acrylic adhesive | Film TML is dominated by water (use RML per UG v11) | Vendor | **Open** |

## 8. Connectors and Electronic Parts (non-metallic portions)

| Component | Material | Status |
|-----------|----------|--------|
| Pico-Clasp housings (501330) and headers (501331) | Molex housing resin per part number | **Open**: E595 per part number; bake-out of the finished harness (V-HAR-11) |
| Micro-Fit 3.0 receptacles (43025) and right-angle headers (43045) | Polyamide (nylon) class housing per Molex | **Open**: moisture-bearing polyamide, TML dominated by water: E595 with WVR (RML per UG v11), bake-out (V-HAR-11) |
| Samtec TFM headers, SFSDT sockets | Samtec LCP-class insulator per part number | **Open**: Samtec declaration (V-HAR-11) |
| Micro-D 21 receptacle on A3 | Insulator per MIL-DTL-83513 (glass-filled polymer) | **Open**: maker's E595 data (V-HAR-11) |
| MMCX jacks and plugs | PTFE dielectric, brass/BeCu, gold | **Open** (data) |
| Access-port board A3 | FR-4, solder mask | As §4 |
| Connector contacts | Phosphor bronze or BeCu, gold over nickel (EEE_PARTS_LIST §2A.8) | Compliant; **no tin-plated contacts** (§2.2) |
| EEE parts | Moulded plastic packages | Plastic packages are generally accepted; terminal finish governed by §2.2 |

## 9. Battery

| Component | Material | Status |
|-----------|----------|--------|
| Cell can | **Nickel-plated steel** (Revision A said aluminium) | Metal |
| Cell wrapper | PET or PVC sleeve per the cell lot | **Open**: if PVC, re-sleeve with PET or polyimide (SQSO-BSA-001 §4 step 2 removes the wrap for inspection) |
| Electrolyte | Sealed inside the can | UG v11 rationale code D claimed, verified by the vacuum mass-loss test of the pack (V-BSA-3; LVR-073) |
| BMS board | As §4 | **Open** |

## 10. Antennas, Solar Panels, ADCS (bought assemblies)

| Assembly | Materials | Status |
|----------|-----------|--------|
| ISIS AntS | Aluminium housing, 301 stainless tapes, Dyneema release line, PCB | **Open**: ISIS materials list and E595 declaration |
| Solar panels and wings (panel vendor) | AZUR 3G30A cells, CMG coverglass, adhesive, PCB or aluminium substrate | **Open**: vendor materials list |
| Magnetometer, gyro, GNSS receiver and patch, magnetorquer rods | Vendor | **Open**: vendor declarations, including terminal finishes |

## 11. Jetson Payload

| Component | Material | Status |
|-----------|----------|--------|
| Jetson Orin Nano module and carrier | Commercial PCB, solder mask, connector plastics, tin finishes | **Open**: no materials data from NVIDIA. Plan: bake the module and carrier at the highest non-operating temperature the module allows, measure mass loss, and claim rationale code C (thermal-vacuum stability in configuration) or E; conformal coat over pure-tin terminations of the carrier (§2.2) |

## 12. Marking

| Item | Requirement | Status |
|------|-------------|--------|
| Markers | No Sharpie; Mighty Marker MM07/MM08 black where marking is needed, e.g. torque stripes (UG v11 Table 5-5) | Compliant by procedure |
| Labels | Polyimide labels with E595 data | **Open** |

## 13. Summary

| Status | Count | Items |
|--------|-------|-------|
| Compliant with a cited source | 9 | G-10CR, PEEK 450G, Scotch-Weld 2216 Gray, Stycast 2850FT/9, Ablestik 84-1LMI, Gap Pad TGP 3000, TGP 3500ULM, metals, Sn63Pb37 |
| **Non-compliant or to replace** | 3 | Loctite 242, nylon 6/6 tie wraps (replaced by lacing tape in SQSO-HAR-001 2.0), SAC305 on flight boards; the PVC FFSD ribbon is withdrawn from the design (§7) |
| Open (data needed) | All others | Vendor certificates or GSFC database entries |

Verification: VI-LV-09 (materials list and contamination report, L−4 M) and VI-SAF-07 (prohibited materials and terminal
finishes) of SQSO-SDP-002.

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | CDR S&MA review: requirements from CDS 2.1.7 and UG v11 §5.3 / Table 5-5 (containerized exemption and what it does not cover); prohibited materials applied to the design (cadmium-plated fasteners, zinc plating, mercury, pure-tin finishes, with mitigation); every value given a source or marked open; isolated-mount materials (G-10CR, PEEK 450G, Ti-6Al-4V screws, VDA tape) added; Loctite 242, nylon ties and SAC305 found non-compliant; Gap Pad 3000 and 2216 values corrected; cell can corrected to steel; "100 % compliant" withdrawn | CDR S&MA review |
| C | 2026-10-27 | Hinge pins A286 with MoS₂ (was 300-series), springs 17-7PH, bushings, stop and snubber silicone (open), magnetometer-zone fasteners, Ti column stud, trim masses, W-Ni-Cu inertia ballast (C-13); AntS and GNSS patch isolated mounts, slot grommets and ±Z sun-sensor carriers (harness design M-2, M-5, M-9); Iyz trim masses W-Ni-Cu or pure W (P3S-11); battery cell retention without adhesive (STR-08); gyro window without a bonded frame (no structural adhesive) | Mechanical design (SQSO-MECH-CAD-001) |
| D | 2026-10-27 | Harness materials from SQSO-HAR-001 2.0 (§15 item P-2): §7 per part number (AS22759/33 wire, Samtec SFSDT, coax, PTFE sleeving, fluoropolymer heat shrink, MIL-T-43435 lacing tape replacing the nylon ties, PVC FFSD ribbon withdrawn), §8 connector housings (Pico-Clasp, Micro-Fit, TFM/SFSDT, Micro-D, MMCX), §5 2216 and Arathane 5750 as staking only; data gaps referred to V-HAR-11; hold-down line series spring and seat (17-7PH, Ti-6Al-4V, PTFE guides; mechanical design, SQSO-MECH-001 E §6) | Harness design; mechanical design |
| E | 2026-10-27 | §7 harness materials (SQSO-HAR-001 §15 P-4): W10E cable Multicomp Pro MINI-1.13 (FEP) with the NASA database FEP entries; D-200-82 splice sleeve and meltable rings (no database entry: TE certificate or project test, owner and date before the EM order 2027-01-15; the related Kynar entries cited); no solder preform in these splices; the tin-barrel D-436-36 is not used (SQSO-MFG-002 G §2A.8). P-4 named §5; the rows belong to §7 (wire, cable and harness), where the coax and heat-shrink rows are | EEE parts engineer, CDR round 3 |
