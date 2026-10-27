# Parts Temperature-Grade Audit
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MFG-003
**Revision:** D
**Date:** 2026-10-27
**Status:** CDR round 3 (RID P3T-01). EEE parts engineer's audit for the chair; the board designers own the BOMs and
make the changes listed in §6
**Line data:** `docs/manufacturing/PARTS_TEMPERATURE_AUDIT.csv` (443 rows, one per BOM line, harness item and bought item)

---

## 1. Result

Every line of the six flight-board BOMs, the harness connectors and wire of `hardware/harness/HARNESS_DESIGN.md`, and
the bought items of `docs/manufacturing/EEE_PARTS_LIST.md` (SQSO-MFG-002 E §2A.6, §6.4) was checked part by part
against an operating rating of −40 … +70 °C and a non-operating (storage) rating of −40 … +70 °C, from the
manufacturer's data sheet, a distributor's listing of the exact part, or the series data sheet with the part number
decoded. Nothing was taken from the BOMs' own "Rating" columns. The battery sense board (its FM12 fuse F0, EPS
board design §14) has no BOM file and is not counted (**since this audit it carries F0A and F0N (FM12A72V5.0A), F0B
(FM08A125V5A), the tap fuses and 3 × 68.1 Ω chains, and, from SQSO-EPS-BRD-001 F, the BQ771806, Q16 and F3: none of
these lines is audited here**; BATTERY_DESIGN 2.9 §7.2); the superseded part tables of SQSO-MFG-002 §4–§11 (CC1125,
RF5110G, LIS3MDL, W25Q128 and others) name no flight part and are not audited.

| Group | Lines | Verified | Decoded from the ordering code | Not confirmed | Not applicable | Non-compliant |
|-------|------:|---------:|-------------------------------:|--------------:|---------------:|--------------:|
| EPS board (`hardware/electrical/eps/eps_board_bom.csv`) | 157 | 62 | 89 | 5 | 1 | 1 |
| OBC board | 71 | 28 | 40 | 2 | 1 | 0 |
| Magnetometer board A2 | 9 | 3 | 6 | 0 | 0 | 0 |
| Access-port board A3 | 6 | 3 | 0 | 1 | 2 | 0 |
| RF board | 127 | 35 | 84 | 3 | 5 | 0 |
| Jetson carrier (incl. the module A1 and SSD A2) | 56 | 23 | 26 | 4 | 3 | 1 |
| Harness connectors, cable, wire, splices (revision C: part numbers of SQSO-HAR-001 §15 P-3/P-4) | 11 | 7 | 4 | 0 | 0 | 0 |
| Bought items (EEE_PARTS_LIST §2A.6, §6.4) | 6 | 3 | 1 | 2 | 0 | 3 |
| **Total** | **443** | **164** | **250** | **17** | **12** | **5** |

"Non-compliant" lines have a known rating that misses −40 °C or +70 °C. "Not confirmed" lines have no rating that can
be attributed to the part as written: either no part number exists yet, or the part number as written is not an
orderable part (§5). "Not applicable" lines are PCB features, pads, footprints, shields and plain metal parts.

**Findings beyond the thermal analyst's list** (EPS fuses, RF crystal, Jetson module):

1. **EPS F1, F3 (ITV4030L1412NR) fail at both ends**: −10 … +65 °C (F3 has since moved to the battery sense board,
   SQSO-EPS-BRD-001 F, where it has to blow only while charging; the rating finding stands). The hot end (+65 °C) also misses the +70 °C
   test, which `hardware/structure/THERMAL_DESIGN.md` §1.1 does not record. No three-terminal fuse rated −40 … +70 °C
   was found in the 4030 footprint (§4).
2. **RF Q601 (RD01MUS3) has no minimum operating temperature in its data sheet**; its storage minimum is −40 °C,
   exactly the first-orbit plateau (§4).
3. **The ISIS AntS** (bought item) is qualified to operate at −20 … +60 °C; it is absent from THERMAL_DESIGN §1.1 (§4).
4. **The CubeSpace CR0002 rods** are rated −20 … +80 °C in thermal vacuum, cold start −35 °C (§4).
5. **Fourteen BOM lines cannot be rated because the part number as written does not exist or is incomplete**: values
   outside the series range (ERA-3AEB 1 MΩ and 365 kΩ, WSL0805 0.5 Ω, Coilcraft 0603HP 20 nH and 0806SQ 10 nH),
   malformed codes (ERJ-3EKF100-1V, ERJ-6RQF1R00V, 1N4148W-Q-7-F), the OBC's C0G 470 nF placeholder, the Micro-D
   slash sheet, and the carrier's three "class" connectors (§5). Each has a replacement with its rating.
6. The RF crystal Y451 is now ABLS-…-L4-T, option L, −55 … +125 °C: confirmed (§7.3).
7. The capacitors are X7R or C0G only; there are no Y5V/Z5U, tantalum, aluminium-electrolytic or polymer capacitors and
   no battery-backed part on any board (§7).

## 2. Requirement and Method

### 2.1 Criterion

From THERMAL_DESIGN §1.1 (SQSO-THM-001 1.5) and the protoflight rule of `docs/environment/ENVIRONMENTAL_TEST_PLAN.md`
§3.4 (allowable ± 10 °C): board cold allowable −30 °C, extended to −40 °C for the first orbit after separation and
tested at −40 °C without margin (ETP §9 deviation 8); hot allowable +50 °C operating, +60 °C non-operating. A part
therefore needs at least **−40 … +70 °C operating, a −40 °C cold start, and −40 … +70 °C non-operating**, as the chair
asked. The battery, the Jetson module and the solar array have their own unit allowables in THERMAL_DESIGN §1.1; they
are assessed against the same criterion here and the result is reported with the unit allowable that governs them.

### 2.2 Determination categories

| Category | Meaning |
|----------|---------|
| Verified | The range was read for the exact part number: in the manufacturer's data sheet or product page, or in a distributor's parametric listing of that exact part (the CSV names which) |
| Decoded from the ordering code | The part number was decoded against the manufacturer's ordering-code table (temperature grade, dielectric code, oscillator temperature code, crystal option, plating and series codes) and the range read from that series' data sheet; or, for a few discretes, the upper limit was read for the exact part and the lower limit from the family's limiting-values table |
| Not confirmed | No range can be attributed: no orderable part number, or the code as written is not an orderable part |
| Not applicable | No temperature-limited material (PCB feature, footprint, shield, metal test point or spacer) |

### 2.3 Storage ratings

The non-operating requirement (−40 … +70 °C) lies inside the operating requirement, so a part whose operating range
covers −40 … +70 °C meets it without a separate storage figure; the CSV column "Storage basis" says where a
published storage rating exists. The reel or packaging storage conditions that manufacturers publish (Murata and
Panasonic +5 … +35/40 °C, Coilcraft tape and reel −40/−55 … +80 °C, Littelfuse ITV 0 … +35 °C, NVIDIA module
+18 … +30 °C for its 5-year shelf life) govern the stores before assembly, not the flight hardware: they belong in the
stores control of the receiving inspection (SQSO-MFG-002 §2A.2), not in this audit's pass/fail.

### 2.4 Sources and access

The TI parts (69 orderable numbers) were read from each product page's orderable-part data (operating range and
status of the exact OPN) and the data sheet's absolute-maximum table (Tstg). Other manufacturers' sites that refused
automated access on 2026-10-27 (Nexperia, Littelfuse, Molex, Samtec site, ST, Analog Devices, Coilcraft product pages,
Lite-On, TE) were covered by their data sheets reached by direct link, by Digi-Key listings of the exact part, or by a
distributor's copy of the data sheet; §11 lists every source. Where only a distributor's figure exists the CSV says so.

## 3. Results per Board

The CSV holds every line with its range, determination, source and notes. Per board:

- **EPS** (157): all TI parts are AEC-Q100 or industrial, −40 … +85 °C or wider (MSP430FR2355 "T" grade −40 … +105 °C;
  BQ76952 −40 … +85 °C). Murata GCM X7R/C0G −55 … +125 °C; Panasonic ERJ/ERA −55 … +155 °C; Vishay CRCW
  −55 … +155 °C, WSLP/WSL −65 … +170 °C; Nexperia/Infineon MOSFETs −55 … +175 °C (TJ); TVS −55/−65 … +150 °C; Pico 466
  fuses −55 … +90 °C; FM12 fuse −55 … +125 °C (MIL-PRF-23419/12F); PPTC −40 … +85 °C; relay TQ2SA −40 … +85 °C; Molex
  Pico-Clasp 203556 and Micro-Fit 43045 −40 … +105 °C; Samtec TFM −55 … +125 °C; Coilcraft XGL4030 −40 … +125 °C
  ambient; Epson SG-8018CA "H" −40 … +105 °C. Non-compliant: F1, F3. Not confirmed: D40/D41, R86–R90, R129 group, R267,
  RS11.
- **OBC** (71): STM32L4R5ZIT6 suffix 6, −40 … +85 °C; FM25V20A-GTR −40 … +85 °C; M-G370PDT0 −40 … +85 °C; BMI088
  −40 … +85 °C (storage −50 … +150 °C); RV-3028-C7 TA −40 … +85 °C; CM7V-T1A TA −40 … +85 °C; Samtec CLM socket
  −55 … +125 °C. The 501331 Pico-Clasp headers (J3–J8, J10) are −40 … +105 °C but tin (finish item, §6). Not confirmed:
  C23–C26, D4.
- **Magnetometer board A2** (9): PNI RM3100 suite −40 … +85 °C. All compliant.
- **Access-port board A3** (6): TVS and ESD parts compliant. Not confirmed: J1 (Micro-D slash sheet).
- **RF** (127): CC1200, LMX2571, PCM1808, CMX138A, AD8361, STM32L431CCT6, TA2209A −40 … +85 °C; TG2016SMN "G"
  −40 … +85 °C; ABLS option L −55 … +125 °C; BFP640F TJ 150 °C, storage from −55 °C; Coilcraft 0603HP/1812SMS
  −40 … +125 °C; Murata GRM C0G −55 … +125 °C; MMCX jacks −55 … +165 °C. Not confirmed: L106, L605, Q601.
- **Jetson carrier** (56): TI parts −40 … +105 °C or wider (TPS22918-Q1 −40 … +105 °C); TE SO-DIMM −55 … +85 °C;
  M.2 connector −40 … +80 °C; Swissbit N-2600 "I" −40 … +85 °C (storage −40 … +85 °C); NTC GA10K3MCD1 −40 … +125 °C;
  gap pad −60 … +200 °C. Non-compliant: A1 (Jetson module). Not confirmed: J1, J2, J3, R1.

## 4. Non-Compliant Lines

| Board | Line | Part | Rating found (source) | Disposition / replacement (source) |
|-------|------|------|-----------------------|------------------------------------|
| EPS | F1, F3 (line 21) | Littelfuse ITV4030L1412NR three-terminal fuse | −10 … +65 °C operating (Digi-Key listing; Littelfuse ITV4030 12 A data sheet rev 03/27/20); no mounted storage rating, packaged storage 0 … +35 °C | **Fails −40 °C and +70 °C.** No compliant part found in the 4030 footprint: Littelfuse ITV2718 and ITV9550 are also −10 … +65 °C (Littelfuse data sheet, distributor listing), Eaton SCF4030-12-04 is −20 … +60 °C (Eaton ELX1133, Jan 2022). Either qualify the ITV4030 on the flight lot by test (V-EPS-8 extended to hold current, blow time and heater-trigger at −40 °C **and +70 °C**, with the hot end added) or replace the secondary interrupt by a design change (EPS designer). THERMAL_DESIGN §1.1 must also record the +65 °C limit |
| Jetson carrier | A1 (line 52) | NVIDIA Jetson Orin Nano 8 GB 900-13767-0030-000 | TJ −25 … +105 °C; TSTG −40 … +80 °C (NVIDIA DS-11105-001 v1.5, Dec 2024) | Operating minimum −25 °C misses −40 °C; non-operating meets −40 … +70 °C with zero cold margin. No replacement (payload of record, BASELINE §6). The unit allowable of THERMAL_DESIGN §1.1 governs: −15 °C power-on gate (test −25 °C = the TJ minimum, no margin), unpowered to −40 °C qualified by the EM soak V-THM-8 |
| Bought | B2 | ISIS AntS 1U–3U | Qualified operating −20 … +60 °C, storage −50 … +85 °C (ISIS antenna systems brochure V2R; AntS electrical model data sheet); thermal qualification testing −40 … +80 °C (ISIS-ANTS-DSH-0001 v4.0) | **Fails −40 °C and +70 °C operating.** No replacement in this audit. Obtain ISIS-ANTS-DSH-0001 v4.0 and an ISIS statement covering the controller and the release at −40 … +70 °C; add the AntS to THERMAL_DESIGN §1.1 with its own allowable and its predicted temperature at the first-orbit release (dated action §12 V-1) |
| Bought | B3 | Samsung INR18650-35E cells | Charge 0 … +45 °C, discharge −10 … +60 °C, storage −20 … +60 °C for 1 month (Samsung specification v1.1) | Fails by design; the battery allowable (THERMAL_DESIGN §1.1, SYS-602) and the heater govern. No replacement |
| Bought | B5 | CubeSpace CR0002 magnetorquer rods | Thermal vacuum −20 … +80 °C, cold/hot start −35 … +70 °C (CubeSpace Gen 2 actuators brochure 2023, family table; no CR0002-specific figure) | **Fails −40 °C.** Obtain the CR0002 option sheet/ICD with a −40 °C rating, or qualify the rods at −40 °C (passive coils: wire insulation and potting are the limiting materials) (dated action §12 V-2) |

## 5. Lines Not Confirmed, with Replacements

| Board | Line | As written | Why not confirmed (source) | Replacement, same footprint unless stated (source of its rating) |
|-------|------|-----------|----------------------------|-----------------------------------------------------------------|
| EPS | D40, D41 (20) | 1N4148W-Q-7-F | Not orderable as written (Digi-Key lists 1N4148WQ-7-F) | **1N4148WQ-7-F**, SOD-123: −55 … +150 °C TJ (Digi-Key, Active) |
| EPS | R86–R90 (77) | ERJ-3EKF100-1V | Not a valid ERJ code; 10 Ω is written 10R0 (Panasonic AOA0000C304) | **ERJ-3EKF10R0V**, 0603: −55 … +155 °C (Digi-Key, Active) |
| EPS | R129, R173, R175, R275, R276, R291 (80) | ERA-3AEB1004V (1 MΩ) | ERA3AEB covers 47 Ω – 330 kΩ only (Panasonic AOA0000C307, 24-Apr-24); Digi-Key: no such part | **ERA-6AEB105V** (0805, ERA6AEB 47 Ω – 1 MΩ, 0.1 %, 25 ppm/K): −55 … +155 °C. **Footprint 0603 → 0805** |
| EPS | R267 (106) | ERA-3AEB3653V (365 kΩ) | Above the ERA3AEB 330 kΩ limit (as above) | **ERA-6AEB3653V** (0805): −55 … +155 °C. **Footprint 0603 → 0805** |
| EPS | RS11 (122) | WSL0805R5000FEA (0.5 Ω) | WSL0805 covers 0.005 … 0.2 Ω (Vishay doc 30100, rev 23-Nov-2023) | **WSL2010R5000FEA** (as RS20–RS23): −65 … +170 °C (Vishay 30100). **Footprint 0805 → 2010** |
| OBC | C23–C26 (7) | "C0G 470 nF 50 V 1210 … order code to select" | No part number; Digi-Key has no CGA6P1C0G1H474J250AA or C3225C0G1H474J250AA | OBC designer to select an orderable C0G 470 nF part (larger case, or 2 × 220 nF C0G in parallel); any C0G is −55 … +125 °C (EIA class). The watchdog timing (tWDU) must be re-derived for the chosen part |
| OBC | D4 (16) | 1N4148W-Q-7-F | As EPS D40 | **1N4148WQ-7-F** |
| A3 | J1 (4) | M83513/11-C (21-way socket) | Dash number not chosen; Cinch lists M83513/11 as a PCB-mount plug (pin contacts) | Full slash-sheet and dash number of a 21-way **socket** PCB receptacle from a QPL source; MIL-DTL-83513 parts are −55 … +125 °C (Cinch Dura-Con) or −55 … +150 °C (Glenair) |
| RF | L106 (33) | 0603HP-20NXGLU (20 nH) | No 20 nH in the 0603HP series (Coilcraft Doc 537-2, 08/08/22) | RF designer to retune with a catalogue value, e.g. **0603HP-18NXGLW** or **0603HP-22NXGLW**: −40 … +125 °C (Doc 537-1) |
| RF | Q601 (96) | RD01MUS3-T614 (PA) | Mitsubishi data sheet (Jan 2026) gives Tj(op) max +150 °C and Tstg −40 … +125 °C but **no minimum operating temperature** | No same-footprint replacement identified. Obtain a Mitsubishi statement of the minimum operating (case) temperature, and characterise the PA (gain, stability, drain current, spurious) at −40 °C on the RF EM before the FM build; the storage minimum −40 °C leaves no margin at the first-orbit plateau |
| RF | L605 (104) | 0806SQ-10NGLC | The 0806SQ values are 5N5, 6N0, 8N9, 12N, 16N, 19N (Coilcraft Doc 617-1, 12/21/22) | **0807SQ-10NGLC** (10.2 nH): −40 … +125 °C (Doc 617-1). The 0807 body is longer: RF designer to check the pads |
| Jetson carrier | J1 (14) | "43045-0400 class, Au" | No part number | **0430450402** (Micro-Fit 3.0 2 × 2 right angle, gold 0.76 µm mating; as EPS J18): −40 … +105 °C (Digi-Key) |
| Jetson carrier | J2 (15) | "501331-0607 class, Au" | No part number | **2035560607** (Pico-Clasp 6, gold 0.38 µm; as RF J2): −40 … +105 °C (Digi-Key) |
| Jetson carrier | J3 (16) | "501331-0207 class, Au" | No part number | **2035560207** (Pico-Clasp 2, gold 0.38 µm): −40 … +105 °C (Digi-Key) |
| Jetson carrier | R1 (23) | ERJ-6RQF1R00V | Not found; ERJ-6RQ writes 1 Ω as 1R0 | **ERJ-6RQF1R0V**, 0805: −55 … +155 °C (Digi-Key, Active) |
| Harness | H11 | M81824/1-1 (TE D-436-36), named by SQSO-HAR-001 §15 P-3 | Temperature not the issue: the crimp splicer is **tin-plated** (ASTM B545; TE drawing D-436-36/-37/-38 Rev F, 2016-08-19), prohibited by SQSO-MFG-002 §2A.8 | **TE Raychem D-200-82** (TE P/N D17660-000), same M81824/1-1 size, nickel-plated barrel: −55 … +200 °C (Bürklin listing; TE D-200 data sheet). 28 AWG is below the 0.15 mm² minimum of the size: harness designer (SQSO-MFG-002 G §2A.8). Revision C: H7 and H9 are now named (Molex 73415-2261, −55 … +165 °C, Digi-Key; Multicomp Pro MINI-1.13, −55 … +155 °C, data sheet V1.0) and verified |
| Bought | B4 | Panasonic NCR18650GA (alternative cell, EEE_PARTS_LIST §6.4) | Not read: not used by the battery design | Delete the alternative, or record its specification before any PI choice |
| Bought | B6 | Body panels and wings | Vendor not selected | Vendor's qualification range at least −56 … +93 °C at selection (THERMAL_DESIGN §1.1, V-THM-4) |

## 6. Replacements per Board (for the chair to pass to the designers)

**EPS board (SQSO-EPS-BRD-001)**
1. F1, F3 ITV4030L1412NR: qualify by test at −40 °C and +70 °C (V-EPS-8 extended) or change the secondary interrupt;
   no compliant three-terminal fuse found.
2. D40, D41: 1N4148W-Q-7-F → 1N4148WQ-7-F.
3. R86–R90: ERJ-3EKF100-1V → ERJ-3EKF10R0V.
4. R129, R173, R175, R275, R276, R291: ERA-3AEB1004V → ERA-6AEB105V (0805 pads).
5. R267: ERA-3AEB3653V → ERA-6AEB3653V (0805 pads).
6. RS11: WSL0805R5000FEA → WSL2010R5000FEA (2010 pads).

**OBC board (SQSO-OBC-BRD-001)**
1. C23–C26: an orderable C0G 470 nF part (tWDU re-derived).
2. D4: 1N4148W-Q-7-F → 1N4148WQ-7-F.
3. (Finish, not temperature, SQSO-MFG-002 §2A.8) J3, J5, J7, J10: 5013310607 → 2035560607; J4, J6: 5013310207 →
   2035560207; J8: 5013311207 → 2035561207 (listed at Digi-Key). All −40 … +105 °C.

**Magnetometer board A2**: (finish) J1 5013310607 → 2035560607. No temperature change.

**Access-port board A3**: J1 complete MIL-DTL-83513 socket-receptacle part number.

**RF board (SQSO-RF-BRD-001)**
1. Q601 RD01MUS3: manufacturer statement of the minimum operating temperature and an EM characterisation at −40 °C.
2. L106: 0603HP-20NXGLU → a catalogue value (0603HP-18NXGLW or 0603HP-22NXGLW) with the match re-tuned.
3. L605: 0806SQ-10NGLC → 0807SQ-10NGLC (pad check).

**Jetson carrier (SQSO-JET-BRD-001)**
1. J1 → 0430450402; J2 → 2035560607; J3 → 2035560207.
2. R1: ERJ-6RQF1R00V → ERJ-6RQF1R0V.
3. A1: no change; unit allowable and V-THM-8 as in §4.

**Harness (SQSO-HAR-001)**: W10E plug 73415-2261 and cable MINI-1.13 are compliant (H7, H9); splices D-200-82 instead of the tin-barrel D-436-36 (H11), and resolve the 28 AWG range; reject any SFSD
(PVC, −10 … +80 °C) substitute for SFSDT at receiving inspection.

**Bought assemblies**: ISIS AntS statement and allowable; CR0002 −40 °C rating or qualification; panel vendor range.

## 7. Capacitors, Crystals and Battery-Backed Parts

### 7.1 Capacitor dielectrics

Every MLCC on every board is X7R ("R7") or C0G ("5C") by the Murata ordering code (GCM series AEC-Q200 on the EPS,
OBC, magnetometer and carrier boards and the RF supply; GRM C0G in the RF signal path), −55 … +125 °C. **No Y5V, Z5U,
X5R or X6S part is fitted.** X7R allows ±15 % capacitance change over −55 … +125 °C (EIA): the worst-case analyses of
the X7R parts that set times must include it at −40 °C, notably OBC C33/C40/C47/C50 (2.2 µF C_RETRY, ≥ 0.1 s off
time) and the EPS 10 nF X7R integrators ("1 ms", C14 group). The OBC watchdog timing capacitors C23–C26 are C0G by
intent (§5).

### 7.2 Electrolytic and polymer capacitors

**None on any flight board** (no tantalum, aluminium-electrolytic, polymer or EDLC part in any BOM), so no
low-temperature ESR rise applies. The tantalum row of SQSO-MFG-002 §11.2 is superseded by the board BOMs.

### 7.3 Crystals and oscillators

| Board | Ref | Part | Code decoded | Range |
|-------|-----|------|--------------|-------|
| EPS | Y1 | Epson SG-8018CA 4.0000M-TJHPA0 | T 1.8–3.3 V, J ±50 ppm, **H −40 … +105 °C**, P OE | Op −40 … +105 °C, Tstg −40 … +125 °C |
| OBC | Y1 | Epson SG-8018CA 8.0000M-TJHPA0 | as EPS Y1 | as EPS Y1 |
| OBC | Y2 | Micro Crystal CM7V-T1A 32.768 kHz 7 pF 20 ppm TA QA | **TA −40 … +85 °C**, QA AEC-Q200 | Storage −55 … +125 °C |
| OBC | U4 | Micro Crystal RV-3028-C7 TA QA | **TA −40 … +85 °C**, QA AEC-Q200 | Storage −55 … +125 °C |
| RF | Y301 | Epson TG2016SMN 40.0000M-MCGNNM3 | M 2.8–3.3 V, C ±0.5 ppm, **G −40 … +85 °C** | Tstg −40 … +90 °C |
| RF | Y451 | Abracon ABLS-3.6864MHZ-L4-T | **Option L −55 … +125 °C** (stability limited to ±50/±100/±150 ppm) | Storage −55 … +125 °C |
| Jetson carrier | Y1 | Epson SG-8018CA 1.0000M-TJHPA0 | as EPS Y1 | as EPS Y1 |

All compliant. The 32.768 kHz tuning-fork crystal Y2 runs about −148 ppm slow at −40 °C (−0.035 ppm/K² × 65 K²,
CM7V-T1A data sheet) and its ESR rises at cold: the OBC LSE drive margin (2.0× in the OBC BOM, computed with the
70 kΩ maximum ESR) should be confirmed with the ESR at −40 °C, and the LSE-trimmed MSI carries the same offset. The
RTC U4 (RV-3028-C7, ±1 ppm at 25 °C) has a tuning-fork crystal with the same parabolic characteristic, so the mission
clock drifts the same way at cold (flight-software time correction, not a parts issue). The TCXO stability
verifications (VI-RF-14) run at −20 … +60 °C only (§10).

### 7.4 Battery-backed parts

None. The RTC U4 runs from the EPS always-on rail (3V3_AO, J1-38) and the STM32 VBAT pin has no backup cell (OBC BOM);
the carrier has no RTC cell. Nothing on the boards depends on a coin cell at cold.

## 8. Parts at the −40 °C Limit (No Margin)

156 lines are rated exactly −40 °C operating (industrial and AEC-Q100 grade 1 ICs, Pico-Clasp and Micro-Fit
connectors, PPTC, relay, XGL inductors, Coilcraft RF inductors, SAW filter, crystals' TA grade, SSD). This is the
basis of the first-orbit allowable and of ETP §9 deviation 8: the −40 °C plateau tests them at their rating with no
margin. Lines whose **storage** minimum is also exactly −40 °C: EPS U37 (MSP430FR2355), U90–U97 (LM74700-Q1), Y1;
OBC Y1; RF U201 (TA2209A), U202 (CC1200), Y301, Q601 and the Coilcraft 0603HP/1812SMS inductors (component storage
−40 … +140 °C); carrier U1 (LM61440-Q1), Y1, A1 (Jetson TSTG), A2 (Swissbit); harness W10 cable (Taoglas storage
−40 … +85 °C). Any test or soak colder than −40 °C (none is planned for the boards; the mechanism tests at −55 °C are
on the hinges and burn boards, ETP §3.8) must exclude these assemblies.

## 9. Ordering-Code Observations (outside the temperature scope)

Found while decoding; they do not change a temperature result but stop an order:

1. **Samtec TFM plating code.** In catalog F-226 (Rev 20OCT26) the TFM plating options are –F (gold flash) and –L
   (15 µin gold); "–S" is the single-row option. TFM-120-02-S-D-A (EPS J3), TFM-120-01-S-D-RA (OBC J1),
   TFM-108-01-S-D-RA (OBC J2, RF J1) and TFM-105-01-S-D-RA (OBC J9) need Samtec confirmation (for gold, for example
   TFM-1xx-01-L-D-RA). SFSDT platings are –F and –G (10 µin), not the 30 µin of SQSO-MFG-002 §2A.8.
2. **Pico-Clasp crimp terminal finish.** The Molex 501330 housing listing names 501334 as the tin terminal and 501193 as
   the gold terminal; HARNESS_DESIGN §3.1 and SQSO-MFG-002 §2A.8 call 501334 gold.
3. **Panasonic ERA E24 values.** ERA-3AEB1002V (and 1001, 1003, 1302, 1502, 2002, 2003, 5602) write E24 values in the
   4-digit form; Panasonic's E24 code has three digits (ERA-3AEB103V is listed at Digi-Key; ERA-3AEB1002V returns no
   result). Confirm with Panasonic or re-code (EPS R18 group, R130 group, R220 group, R274; OBC R72 group; RF R204,
   R730, R802; carrier R4, R5, R9 group). Temperature unaffected (ERA3A −55 … +155 °C).
4. **Obsolescence noted on the way**: BFP640F (RF Q201) Last Time Buy 2027-05-15 (Digi-Key); ERJ-1TNF2000U (EPS R83)
   and NCP15XH103F03RC (EPS RT2) Not For New Designs; GRM1885C2A6R0BA01D Not For New Designs. Coilcraft packaging
   codes U (0603HP) and B (SQ) are being replaced by W and C.

## 10. Corrections to Other Documents

Items 1–3 were applied at the chair's request (revision B): THERMAL_DESIGN 1.6 §1.1, THERMAL_ANALYSIS 2.3 §3 and
EEE_PARTS_LIST F §2A.2, §2A.6, §2A.8, §2A.9 and §6.4. Item 4 stays with the RF designer.

1. `hardware/structure/THERMAL_DESIGN.md` §1.1, basis column: EPS fuses ITV4030 −10 … **+65 °C** (hot end too); RF
   board basis to note Q601's unspecified minimum; add rows for the ISIS AntS (−20 … +60 °C qualified) and the CR0002
   rods (−20 … +80 °C, cold start −35 °C); the Jetson carrier and stack bases are confirmed by this audit except as
   listed in §4–§5.
2. `docs/environment/THERMAL_ANALYSIS.md` §3 table still lists SYS-600 −20 … +50 °C and BQ76920 (the EPS AFE is
   BQ76952, −40 … +85 °C).
3. `docs/manufacturing/EEE_PARTS_LIST.md`: §2A.8 terminal 501334 vs 501193 and SFSDT plating (§9); §2A.6 rows for the
   secondary over-voltage protector and fuse are now BQ771806 and ITV4030 (EPS BOM); §6.4 cell alternative (B4);
   §2A.2 board-level screening cycles at −30 … +70 °C reach neither the −40 °C plateau nor the parts' limit, which is
   acceptable for screening but should say so.
4. Verification items `docs/verification/VI_REGISTER.md` VI-RF-02, VI-RF-14, VI-RFB-07, VI-RFB-09 and VI-RFB-10 test
   the RF functions at −20 … +60 °C; with the −30 °C allowable and the −40 °C first orbit they should run at the test
   range of ETP §3.4 (RF designer).

## 11. Sources (read 2026-10-27)

| Source | Used for |
|--------|----------|
| TI product pages (orderable-part data: operating range and status of each OPN) and data sheets (absolute-maximum Tstg), 69 OPNs; literature numbers in the CSV | All TI lines |
| Digi-Key listings of the exact part (operating temperature, status, finish) | Littelfuse SMBJ/SMF/466/2920L/ITV4030, Nexperia, Infineon IPD50P04P4L-11, Diodes 1N4148WQ-7-F, Murata representatives, BLM18, NCP15, Panasonic representatives, Molex 203556/43045, Coilcraft XGL4030-682MEC, TQ2SA, FM25V20A, CMX138A, TE 1-2199230-4 and GA10K3MCD1 |
| Murata ordering-code scheme (temperature characteristic R7 = X7R, 5C = C0G) | All GCM/GRM MLCCs |
| Panasonic AOA0000C304 (ERJ, 29-May-25) and AOA0000C307 (ERA, 24-Apr-24) | ERJ/ERA resistors, ranges and ordering codes |
| Vishay docs 20035 (CRCW e3, rev 14-Apr-2026), 30122 (WSLP, rev 09-Sep-2024), 30100 (WSL, rev 23-Nov-2023), 86366 (BAS16) | Vishay resistors, BAS16 |
| Littelfuse ITV4030 12 A data sheet (rev 03/27/20, distributor copy), ITV2718 data sheet; Eaton ELX1133 (SCF4030, Jan 2022) | Three-terminal fuses |
| MIL-PRF-23419/12F (6 Aug 2018) and AEM FM12 data sheet (Jan 2018) | EPS F2 |
| Nexperia/NXP limiting-values tables (BAT54, BCX56, 2N7002BK family, PMEG40T30ER 6 Mar 2018) via distributor copies | Lower limits of the Nexperia small-signal parts |
| Infineon BFP640F data sheet Rev 4.0 (2024-07-01) | RF Q201 |
| Mitsubishi RD01MUS3 data sheet (publication Jan 2026) | RF Q601 |
| Samtec catalog F-226 (Rev 20OCT26) TFM and SFSD/SFSDT pages; Samtec FTSH data sheet copies; RS listing of CLM-110-02-L-D | Samtec connectors and cable assemblies |
| Molex MMCX literature 987650-3243; RS and Heilind listings of 501331, 501330, 501334 | Molex MMCX and Pico-Clasp |
| Coilcraft Doc 537 (0603HP, 08/08/22), Doc 184 (Midi Spring, 12/02/21), Doc 617 (SQ, 12/21/22); XGL4030/XGL6030 series pages | Coilcraft inductors |
| Epson SG-8018CA and TG2016SMN briefs (download.epsondevice.com); Epson M-G370PDT0 brief sheet | Oscillators, gyro |
| Micro Crystal CM7V-T1A data sheet V8.0/10.2016; RV-3028-C7 data sheet V1.3/09.2021 | OBC Y2, U4 |
| Abracon ABLS options (option L), LCSC listing of ABLS-…-L4Q-T; RF designer's reading of drawing 450669 rev AD | RF Y451 |
| Bosch BST-BMI088-DS000-19 rev 1.9; PNI RM3100 product page | OBC U11, magnetometer board |
| Tai-Saw TA2209A approval sheet Rev 1.0 (2017); ADI AD8361 data sheet (mirror); Lite-On LTST-C191KRKT data sheet (via Future) | RF U201, U701, OBC D1 |
| NVIDIA DS-11105-001 v1.5 (Dec 2024); Swissbit N-2600 fact sheet Rev 1.04; Heilind listing of TE 2309407-2; Henkel GAP PAD TGP 3500ULM data sheet | Jetson carrier |
| Taoglas 1.37 mm micro-coax 001221 specification; TE Raychem M22759/33 data sheets; Farnell listing of TE D-436-37; TE drawing D-436-36/-37/-38 Rev F; TE D-200 data sheet and Bürklin D17660-000 listing; Digi-Key 0734152261, 0430300003, 5011939000; Multicomp Pro MINI-1.13 data sheet V1.0 | Harness |
| ISIS antenna systems brochure V2R, AntS electrical model data sheet, ISIS-ANTS-DSH-0001 v4.0; SatNow piNAV-NG listing; Samsung INR18650-35E specification v1.1; CubeSpace Gen 2 actuators brochure 2023 | Bought items |

## 12. Vendor Actions for the Bought Items (dated, before the EM order 2027-01-15)

| Item | Action | Owner | Due | Fallback if not obtained by the due date |
|------|--------|-------|-----|------------------------------------------|
| **V-1** ISIS AntS (THERMAL_DESIGN §1.1 row; VI-STR-13) | Obtain ISIS-ANTS-DSH-0001 v4.0 and a written ISIS statement covering (a) operation of the controller and of the thermal-knife release at −40 °C, (b) unpowered survival to −57 °C (the first-orbit minimum with the 10 °C margin, SQSO-THA-001 2.4 §4.4) and at +70 °C, (c) the hot operating limit against +60 °C, (d) the AntS temperature telemetry over I²C (command, resolution, accuracy, and that it may be read with the controller powered only for the read), on which the deployment gate (decision C-18) relies. **Update (THERMAL_ANALYSIS 2.4 §4.4):** the AntS node now exists: −44.0 °C at an ungated release after an eclipse separation, first-orbit minimum −47.0 °C (−57.0 °C with the margin); the deployment is gated on the AntS temperature (C-18). The original note follows. **What the analysis predicts:** the AntS is outside the structure on the −Z face (black anodize, α 0.88, ε 0.85) and is **not a node** of the thermal network (SQSO-THA-001 2.3 §3). At its release, which follows the wing burns in the first orbit (separation + ≈ 31 min, in eclipse after the dispenser cold soak; MECHANISMS_ANALYSIS §7.1), the nearest predicted nodes are −30.2 °C (frame-side hinge) to −40.6 °C (stowed wing), −40.2 … −50.6 °C with the 10 °C margin (SQSO-THA-001 §4.3); in orbit the body panels span −22.7 … +51.5 °C (−32.7 … +61.5 °C with the margin, §5). Both ends are therefore expected outside the −20 … +60 °C qualified range. The thermal analyst adds an AntS node (−Z face, elements and controller) and reports its first-orbit and orbit extremes | Mechanical lead (PI until named), AntS procurement; thermal analyst for the node | Statement and data sheet 2026-12-15; AntS node 2026-12-15 | (1) Power the AntS controller only for deployment and status reads, and move the AntS deployment out of the first eclipse to a sunlit window in which the AntS node (or the −Z face) is above the ISIS-stated minimum, with the per-orbit retries kept (`sqso_fsw::deployment` timing change by the FSW lead; the unpowered rating −50 … +85 °C then covers the eclipses); (2) add an AntS cold deployment at the ISIS minimum and an unpowered cold soak at −40 °C to the deployment test of ETP §3.8 (VI-STR-13 "deployment at the minimum predicted temperature"); (3) if ISIS states neither, a different deployable antenna is a chair decision before the EM order |
| **V-2** CubeSpace CR0002 rods (THERMAL_DESIGN §1.1 row) | Obtain the CR0002 option sheet/ICD with its operating and survival range; request a −40 °C rating (operating, cold start) | GNC lead (PI until named) | 2026-12-15 | Qualify the rods by test: one flight-lot rod, unpowered soak and powered cold start at −40 °C, 3 cycles −40 … +70 °C, coil resistance, insulation resistance to the core and dipole moment at −40 °C and +70 °C, visual for potting cracks (on the EM, before the FM order); the test is planned and the rod ordered by 2027-01-15. If the rod fails, select a rod with a published −40 °C rating (chair decision) |
| **V-3** Panasonic NCR18650GA (EEE_PARTS_LIST §6.4 alternative) | Record the cell specification's charge, discharge and storage ranges, or delete the alternative | EPS designer (PI until named), PI decision on the cell | 2026-11-30 | Delete the alternative: the battery design, the battery safety assessment and the thermal analysis all use the INR18650-35E (charge 0 … +45 °C, discharge −10 … +60 °C) |
| **V-4** Body panels and wings (THERMAL_DESIGN §1.1; V-THM-4) | At vendor selection, obtain the qualified range of the panels (cells, panel PCB, thermistors, pigtails and their connectors): at least −56 … +93 °C for ≥ 11,000 cycles | EPS designer (PI until named), panel procurement | 2027-01-15 | Qualification by test of one EM panel and one wing over −56 … +93 °C (cycle count per V-THM-4) before the FM order, at the project's cost; a vendor whose range is narrower is not selected |
| **V-5** Mitsubishi RD01MUS3 (RF Q601) | Written statement of the minimum operating (case) temperature | RF designer (PI until named) | 2026-12-15 | Characterise the PA at −40 °C on the RF EM (gain, stability, drain current, spurious, at 0.5 W and 1 W) and accept by test; PA keyed only above −30 °C by flight-software constant if the EM shows a cold anomaly |
| **V-6** SkyFox piNAV-NG/FM | Obtain the vendor data sheet confirming the −40 … +85 °C operating and −55 … +100 °C storage figures of the catalogue listing | GNC lead (PI until named) | 2026-12-15 | Receiver powered only above −30 °C (GNSS_EN by flight software) until the vendor confirms; the receiver is non-critical (SQSO-MFG-002 §2A.1) |

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-27 | Initial issue (CDR round 3, RID P3T-01): part-by-part temperature audit of the six flight-board BOMs, the harness connectors and wire and the bought items; 443 lines; non-compliant and not-confirmed lines with replacements; line data in `PARTS_TEMPERATURE_AUDIT.csv` | EEE parts engineer, CDR round 3 |
| B | 2026-10-27 | Chair action: §12 dated vendor actions for the bought items (AntS with the analysis predictions, CR0002, NCR18650GA, panels, RD01MUS3, piNAV-NG) with owners, due dates and fallbacks; §10 items 1–3 applied to THERMAL_DESIGN 1.6, THERMAL_ANALYSIS 2.3 and EEE_PARTS_LIST F | EEE parts engineer, CDR round 3 |
| C | 2026-10-27 | Harness rows from SQSO-HAR-001 §15 P-3/P-4: H2 501193-9000 and H4 43030-0003 decoded (Pico-Clasp, Micro-Fit series), H7 73415-2261 and H9 MINI-1.13 verified; H11 D-436-36 rejected (tin barrel), D-200-82 instead; totals 164 verified, 250 decoded, 17 not confirmed. Board lines not re-audited after the designers' BOM changes | EEE parts engineer, CDR round 3 |
| D | 2026-10-27 | Documentation audit (post-CDR 2026): §1 records that the battery sense board's parts (F0A, F0B, F0N, tap fuses and chains, and from EPS F the BQ771806, Q16 and F3) are not covered by this audit, and that F3 moved there; results unchanged | Documentation audit |
