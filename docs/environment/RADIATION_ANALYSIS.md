# Radiation Environment Analysis
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ENV-002
**Revision:** F
**Date:** 2026-10-27
**Reference:** NASA-HDBK-4002A; NASA PD-ED-1260 (radiation design margin); ECSS-E-ST-10-04C; ECSS-Q-ST-60-15C; NASA/TM-20210018053 (avionics radiation hardness assurance)

---

## 1. Purpose

This document gives the radiation environment of the baseline orbit, the total ionizing dose (TID) as a function of
aluminium shielding, the TID capability and single-event-effect (SEE) behaviour of the selected parts, the mitigations
as implemented, and the verification items with their owners and dates. Revision C re-ran §3–§5 for the baseline orbit
and design life (finding TB-03); Revision B's values were for a 500 km, 51.6° orbit and a 12-month mission and are
withdrawn. Revision F (CDR panel findings P3F-G-01 and P3F-G-08) runs the dose at the baseline band (465–485 km),
reconciles §4.3, §5 and §7 with the selected parts (`docs/manufacturing/EEE_PARTS_LIST.md` and the board BOMs
`hardware/electrical/*/*_bom.csv`) and with the flight software as built, corrects the Jetson per-burst upset
probability, and gives each verification item an owner and a dated event.

Every number in §4 and §5 is produced by `software/simulation/cdr/avionics/radiation_tid_see.py` (run it with
`--markdown`). The script runs no environment model itself: it combines published SHIELDOSE-2 outputs and in-orbit
measurements (§3.1), scales them to the orbit and the life, and tabulates them.

## 2. Mission Orbit Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 475 km target, 465–485 km band, not above 485 km (`docs/BASELINE_PARAMETERS.md` §3). The analysis is run at 465, 475 and 485 km (§4.1); 500 km and 525 km, the former target and cap, are kept as upper bounds (the dose rises with altitude) because the mission simulation's tests cite them |
| Inclination | ≈ 97.31° at 475 km (sun-synchronous; 97.2–97.4° across the band, BASELINE §3) |
| Mission duration | 2 years design life; 12 months minimum (BASELINE §2) |
| Launch | 2028 (BASELINE §3, D7): the declining phase of solar cycle 25 |
| Storage | Ground storage before launch adds less than 1 rad(Si) (natural background); in orbit, the analysis covers the 2-year design life. After passivation (BASELINE §5) no avionics function is required, so the years to re-entry are not part of the design dose |

## 3. Method and Inputs

### 3.1 Sources

| ID | Source | What it gives | Use |
|----|--------|---------------|-----|
| T1 | SPENVIS SHIELDOSE-2, AE-8 MAX / AP-8 MAX, 500 km, 1 year, centre of solid Al spheres; report `centre.doc` (2007) on the "Radiation Environment for a Sun Synchronous Orbit" page of cubesat.wikidot.com | Dose-depth curve, 0.05–10 mm Al, split into trapped electrons, bremsstrahlung and trapped protons | The trapped dose. The report does not state the inclination (its SPENVIS project folder is named "45INC"), so T2 checks it |
| T2 | D. L. Thomsen III, *Shields-1 Dosimetry Measurements in Polar Low Earth Orbit*, NASA LaRC, 2022-09-14 (Shields-1, NORAD 43850, 500 km, 85°) | SPENVIS SHIELDOSE-2 (AP-8 MIN / AE-8 MAX) sphere dose behind 1.98 mm Al: 117 rad(Si)/yr protons + 1266 rad(Si)/yr electrons = 1383 rad(Si)/yr. Measured slab dose in orbit (2019, solar minimum): 90.9 ± 2.7 rad/yr behind 1.26 g/cm², 73.6 ± 3.2 behind 3.00 g/cm², 70.0 ± 3.0 behind 6.00 g/cm² | Cross-check of T1 at 2 mm in a polar orbit; GCR term (T4); evidence that the trapped models under-predict thick-shield dose by 2–5× |
| T3 | NASA, *State-of-the-Art of Small Spacecraft Technology*, ch. 6 (Structures, Materials and Mechanisms), 2021, for the Shields-1 orbit; the King (SOLPRO) curve in T2 | Solar protons, 1 year, 95 % confidence: ≈ 750 rad(Si) behind 0.535 g/cm² (2 mm), ≈ 400 rad(Si) behind 1.26 g/cm² | Solar-proton dose, interpolated exponentially in areal density |
| T4 | T2: measured dose behind 6 g/cm² less the trapped-proton model there (70.0 − 13.7 rad/yr) | ≈ 56 rad/yr not in the trapped models: galactic cosmic rays and their secondaries | 50 rad(Si)/yr at every depth |
| T5 | The mission simulation's altitude scaling (`software/rust/sqso-sil/src/env/radiation.rs`) | Trapped dose × exp((h − 500 km)/150 km) | The baseline band (465–485 km, a factor 0.79–0.94) and the 525 km bound. Below 500 km it extrapolates the same law; solar protons and GCR are not scaled, so the low-altitude totals are not optimistic in that part |
| P1 | T1's trapped-proton spectra (report `trapped.doc`, AP-8 MAX, 500 km) | Orbit-average integral flux: > 10 MeV 21.5, > 30 MeV 17.4, > 50 MeV 14.8, > 100 MeV 9.28 cm⁻² s⁻¹ | Proton SEE rates (§5), used unscaled at 465–485 km, where the flux is lower: the rates are upper bounds for the baseline band |

SPENVIS itself (AE9/AP9-IRENE with SHIELDOSE-2, or OMERE) could not be run from the review environment; the
verification item RAD-V1 (§10) is that run.

**Cross-check (T1 against T2, 1 year, 1.98 mm):** 1865 rad(Si) against 1383 rad(Si), a ratio of 1.35. T1 uses AP-8 MAX
(more protons) and is the larger, so it is used. Both runs use AE-8, which over-predicts LEO electron flux relative to
AE9 for sun-synchronous orbits by a factor that published comparisons put at 1–3 (AE9/AP9-IRENE v1.5 change report);
no credit is taken for it.

### 3.2 Combination

Dose(depth) = [T1 trapped × e^((h − 500)/150) + T3 solar protons + T4 GCR] × 2 years, solid sphere (4π), rad(Si).

## 4. Total Ionizing Dose

### 4.1 Dose-depth, 2 years

**Baseline band, 2 years, solid Al sphere: total krad(Si) / margin of the 10 krad design level**

| Depth (mm Al) | 465 km (floor) | 475 km (target) | 485 km (cap) |
|---|---|---|---|
| 1.0 | 10.39 / 0.96 | 10.97 / 0.91 | 11.59 / 0.86 |
| 2.0 | 4.49 / 2.23 | 4.69 / 2.13 | 4.91 / 2.04 |
| 2.1 | 4.22 / 2.37 | 4.40 / 2.27 | **4.60 / 2.17** |
| 3.0 | 2.54 / 3.94 | 2.62 / 3.81 | 2.72 / 3.68 |

**475 km (target), 2 years, solid Al sphere, rad(Si)**

| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |
|---|---|---|---|---|---|
| 1.0 | 8,985 | 1,887 | 100 | **10,972** | 0.91 |
| 1.5 | 4,967 | 1,679 | 100 | **6,746** | 1.48 |
| 2.0 | 3,098 | 1,494 | 100 | **4,692** | 2.13 |
| 2.1 | 2,843 | 1,459 | 100 | **4,402** | 2.27 |
| 2.5 | 2,016 | 1,329 | 100 | **3,445** | 2.90 |
| 3.0 | 1,341 | 1,182 | 100 | **2,623** | 3.81 |

**485 km (cap), 2 years, solid Al sphere, rad(Si)**

| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |
|---|---|---|---|---|---|
| 1.0 | 9,604 | 1,887 | 100 | **11,591** | 0.86 |
| 1.5 | 5,310 | 1,679 | 100 | **7,089** | 1.41 |
| 2.0 | 3,312 | 1,494 | 100 | **4,905** | 2.04 |
| 2.1 | 3,039 | 1,459 | 100 | **4,598** | 2.17 |
| 2.5 | 2,155 | 1,329 | 100 | **3,584** | 2.79 |
| 3.0 | 1,433 | 1,182 | 100 | **2,715** | 3.68 |

**500 km (former target, upper bound), 2 years, solid Al sphere, rad(Si)**

| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |
|---|---|---|---|---|---|
| 1.0 | 10,614 | 1,887 | 100 | **12,601** | 0.79 |
| 1.5 | 5,868 | 1,679 | 100 | **7,647** | 1.31 |
| 2.0 | 3,660 | 1,494 | 100 | **5,254** | 1.90 |
| 2.1 | 3,359 | 1,459 | 100 | **4,918** | 2.03 |
| 2.5 | 2,382 | 1,329 | 100 | **3,811** | 2.62 |
| 3.0 | 1,584 | 1,182 | 100 | **2,866** | 3.49 |

**525 km (former cap, upper bound), 2 years, solid Al sphere, rad(Si)**

| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |
|---|---|---|---|---|---|
| 1.0 | 12,539 | 1,887 | 100 | **14,526** | 0.69 |
| 1.5 | 6,932 | 1,679 | 100 | **8,711** | 1.15 |
| 2.0 | 4,324 | 1,494 | 100 | **5,917** | 1.69 |
| 2.1 | 3,968 | 1,459 | 100 | **5,527** | 1.81 |
| 2.5 | 2,814 | 1,329 | 100 | **4,243** | 2.36 |
| 3.0 | 1,872 | 1,182 | 100 | **3,153** | 3.17 |

Electrons dominate below about 2.5 mm; the curve falls by a factor of 4.1–4.4 from 1 to 3 mm. Solar protons are 30–33 %
of the 2 mm total in the baseline band, so a quiet two years would give less, and a large event in the declining phase
of cycle 25 more: T3 is a 95 % confidence level for one year.

### 4.2 Against the baseline design level

BASELINE §6 says "design to 10 krad (≥ 2× the 2-year dose behind 2 mm Al)" and, after RID RAD-01, requires
**≥ 2.1 mm Al-equivalent by sector analysis, or a demonstrated ≥ 2× margin on the sector dose** for sensitive parts.
In the baseline band the 2-year dose behind 2 mm is **4.49–4.91 krad(Si)** (4.69 at the 475 km target), so 10 krad
is **2.23–2.04×**: the margin of 2 is met at 2 mm across the band, with 2 % to spare at the 485 km cap. The margin of
2 needs 1.85, 1.91 and 1.98 mm at 465, 475 and 485 km; a margin of 3 (the NASA PD-ED-1260 value for parts without
lot data) needs 2.50, 2.56 and 2.63 mm. The **2.1 mm rule of RAD-01 gives 2.17× at the cap** (4.60 krad) and is kept:
it was set at 500 km (where 2 mm gives 1.90× and 2.1 mm 2.03×), and the 0.1 mm covers the uncertainty of the
altitude extrapolation (T5) and of RAD-V1. At the former 500/525 km bounds the margin of 2 needs 2.08/2.25 mm.

A 1U wall gives about 1.5–2 mm Al-equivalent (frame, PCB panels and cells), and boards inside the stack are shielded
by the boards around them, so the electronics see more than the wall. The shielding of each part is not known until the
layout exists: the sector analysis RAD-V2 on the CAD model sets it at the EM layout freeze. A mission-critical part
that sees less than 2.1 mm there, and whose TID capability is not shown to be ≥ 2× its sector dose, gets spot
shielding or is replaced.

### 4.3 Part TID capability

Required capability: the 10 krad(Si) design level behind ≥ 2.1 mm (2.17× the 485 km dose there), or ≥ 2× the part's
sector dose (RAD-V2). Criticality as EEE_PARTS_LIST §2A.1: mission-critical parts are those of the bus chain, the
telecommand receiver (47 CFR 97.207(b): the TX-OFF path must survive the mission) and every Category 1 or 2 item.
References are the board BOMs: OBC `hardware/electrical/obc/obc_board_bom.csv`, magnetometer board A2
`hardware/electrical/obc/mag_board_bom.csv`, EPS `hardware/electrical/eps/eps_board_bom.csv`, RF
`hardware/electrical/rf/rf_board_bom.csv`, Jetson carrier `hardware/electrical/jetson/jetson_carrier_bom.csv`.

| Part class (BOM references) | Function | Criticality | TID data | Status and test |
|-----------------------------|----------|-------------|----------|-----------------|
| STM32L4R5ZIT6 (OBC U1) | OBC | Mission | No lot data found for the L4R5; ST L4-series parts in published CubeSat tests are commonly reported functional beyond 20 krad(Si) (not credited) | **Open: RAD-V3** lot test, or a heritage report for the L4R5 die |
| FM25V20A FRAM ×2 (OBC U3, mirror U36) | Non-volatile memory | Mission | Ferroelectric cells are TID-hard; the CMOS periphery sets the limit (no lot data found). Both chips are one part type and one lot, so the mirror (SRS-F160) does not cover a TID failure | **Open: RAD-V3** |
| TPS3850H33-Q1 (OBC U2) | External supervisor (AV-02) | Mission | None | **Open: RAD-V3**. A drift of its window or threshold resets the OBC early or late; its ±7 % window is checked in the test |
| RV-3028-C7 (OBC U4) | Mission clock on 3V3_RTC (AV-10) | Mission (time-stamps) | None | **Open: RAD-V3** |
| OBC interface logic: TXU0304-Q1, TXU0204-Q1, SN74LVC1T45-Q1, TCA9517-Q1, SN74LVC1G/2G gates, CD74HC4060, TLV9002-Q1, TMP235-Q1, DRV8876-Q1 | Isolation, buffers, coil drive and off-timer, sun-sensor amplifiers | Mission | None | **Open: RAD-V3** by one part type per process family (LVC logic, TXU translators, HC counter, TLV9002, DRV8876) |
| ADS7138-Q1 (OBC U16; EPS U36) | Sun-sensor ADC; panel and pack temperatures | Mission (attitude; battery temperature telemetry) | None | **Open: RAD-V3** |
| PNI RM3100 (A2 U1) | Flight magnetometer (selected; replaces the M315/MM200/Lodestone candidates) | Mission (B-dot and the spin law) | **None published** (SQSO-OBC-BRD-001 §9.4) | **Open: RAD-V3** (biased, through its I²C interface: offset, gain and noise at 0, 5 and 10 krad against the 15 nT noise and the 300 nT bias budget of AV-V4) |
| Epson M-G370PDT0 (OBC U8) | Primary gyro | Mission | Pass at 5, 10 and 20 krad(Si) (Epson data, `docs/hardware/BUILD_VS_BUY.md` §5.2) | **Met**: 20 krad is 4.1× the 485 km dose behind 2 mm |
| Bosch BMI088 (OBC U11) | Backup gyro | Non-critical while unpowered | None | Accepted without test: it accumulates dose unbiased (usually the milder case); its loss leaves the primary |
| MSP430FR2355 (EPS U37) | EPS controller | Mission (telemetry, OBC supervision SRS-F162, MPPT set points) | None | **Open: RAD-V3**. No protection depends on it (SQSO-EPS-BRD-001 §8.3) |
| BQ76952 AFE, BQ771806 secondary protector (EPS U1, U2) | Battery protection | Mission and safety | None | **Open: RAD-V3**: threshold drift of the over-voltage and under-voltage detection is the safety concern (functional and the COV/CUV trip points within the data-sheet limits) |
| TPS55340-Q1 trackers, LM74700-Q1 ideal diodes, LMR43620-Q1 bucks (3V3_AO, 5V_PA, V_BURN), TPS70933-Q1, REF3425-Q1, TLV7042-Q1, TPL5111, INA228-Q1, INA3221-Q1, INA186-Q1, ISO1540-Q1 (EPS) | Power path, always-on references and comparators, monitors | Mission | None | **Open: RAD-V3** by part type; the references and comparators set the load-shed, burn and charge thresholds, so the test records their drift |
| Load switches and eFuse: TPS2553-Q1 (EPS U22–U27, U38; OBC U5–U7, U35), TPS1H100B-Q1 (EPS U18, U20), TPS1H000-Q1 (EPS U21, U39), TPS16630 (EPS U19) | Rail switching and the latch-up trips of §5.3 | Mission (they isolate the bus) | None | **Open: RAD-V3**: current-limit and trip points measured at each dose step (a drifting trip is a lost latch-up protection or a nuisance trip) |
| RF receive path: TA2209A SAW filter, BFP640F SiGe LNA, CC1200 receiver, TG2016SMN TCXO, STM32L431CCT6 TNC MCU, CMX138A CTCSS decoder, TPS7A2030/TPS73101 regulators, SN74LVC1G17/1G04 buffers (RF U201, Q201, U202, Y301, U310, U452, U801, U802, U480, U482) | 435 MHz command receiver and TNC (TX-OFF path) | **Mission, 97.207(b)** | None (the SAW filter is passive and not dose-sensitive; SiGe HBTs are commonly tolerant; neither credited) | **Open: RAD-V3** at ≥ 10 krad for CC1200, STM32L431, CMX138A, the TCXO, BFP640F and the regulators (RF board samples, SQSO-RF-BRD-001 §12) |
| RF transmit path: LMX2571 synthesizer, PCM1808, RD01MUS3 PA, AD8361 detectors, TLV9001, REF35250Q, the transponder logic (SN74LVC1G123, CD74HC4060, LVC gates, TMUX1101/1219) | 145.9 MHz downlink and transponder | Mission (downlink) | None | **Open: RAD-V3** (LMX2571, PCM1808, RD01MUS3, AD8361). The PA's gate threshold shift is absorbed by the closed Vgg loop (TLV9001 integrator on TX_FWD), which RAD-V3 checks by holding 27.0/30.0 dBm on the irradiated sample |
| Jetson Orin Nano module (900-13767-0030-000) and carrier parts (LM61440-Q1, TPS259474L, TPS22918-Q1 and the carrier logic) | AI payload | Non-critical, expendable (BASELINE §6) | Orin AGX tested for SEE and TID ("Single Event Effects and Total Ionizing Dose Radiation Testing of NVIDIA Jetson Orin AGX System on Module", 2023; only the listing was reachable); none for the Nano module | Accepted without test: its loss ends the gated experiment only. The bus-side isolation (EPS U19 and SW1) is in RAD-V3 |
| SkyFox piNAV-NG/FM | GNSS receiver | Non-critical | Vendor statement (EEE_PARTS_LIST §2A.6) | Accepted on the vendor statement; on its own tripped rail |
| ISIS AntS | Antenna deployment | Mission (one-time) | Flight heritage | Accepted on heritage |

The panel's point that the parts without data are many is correct: apart from the M-G370PDT0 every mission-critical
part relies on RAD-V3. The test is planned by part type, with the samples bought with the EM order (§10).

### 4.4 Comparison with the mission simulation

`software/rust/sqso-sil/src/env/radiation.rs` (after RID RAD-02) models TID behind 2 mm as 1,830 rad(Si)/yr trapped at
500 km, scaled by e^((h − 500 km)/150 km) and accumulated with the region weights, plus 797 rad(Si)/yr solar protons
and GCR (2.63 krad(Si)/yr at 500 km), all scaled by (2 mm / t)^1.35:

| Depth (mm Al) | SIL 475 km | This analysis 475 km | SIL / analysis | SIL 500 km | This analysis 500 km | SIL / analysis |
|---|---|---|---|---|---|---|
| 1.0 | 5,980 | 5,486 | 1.09 | 6,697 | 6,301 | 1.06 |
| 1.5 | 3,459 | 3,373 | 1.03 | 3,874 | 3,823 | 1.01 |
| 2.0 | 2,346 | 2,346 | 1.00 | 2,627 | 2,627 | 1.00 |
| 2.1 | 2,197 | 2,201 | 1.00 | 2,460 | 2,459 | 1.00 |
| 2.5 | 1,736 | 1,722 | 1.01 | 1,944 | 1,905 | 1.02 |
| 3.0 | 1,357 | 1,311 | 1.03 | 1,520 | 1,433 | 1.06 |

The simulation agrees with this analysis to within 1 % at 1.5–2.5 mm and over-predicts by up to 9 % at 1 and 3 mm (the
single exponent does not follow the electron-dominated curve exactly): it is conservative everywhere. Revision B's model
(1.5 krad(Si)/yr, (2 mm / t)^1.2) was 1.6–1.8× low and is withdrawn; Revision E's §4.4 still showed it. The simulation's
dose feeds only the `tid_krad` total in the summary, not a pass criterion.

The simulation's SEE rates (one OBC reset per 90 days, one Jetson latch-up per 60 powered days, one FRAM flip per 180
days, one receiver hang per 120 powered days) are 70–700 times the estimates of §5. That is right for its purpose,
which is to exercise the fault responses often in a two-year run, and it must not be read as a prediction.

## 5. Single-Event Effects

### 5.1 Method

Proton-induced rates are σ × Φ(> E), with Φ from P1 (> 30 MeV for upsets, > 100 MeV for proton-induced latch-up, which
needs nuclear reactions in the parasitic structure), at 500 km: upper bounds for the 465–485 km band. Heavy-ion (GCR)
contributions are not computed: at 97.3° the polar passes are open to GCR, and for parts with low LET thresholds they
can match the proton term, so the high end of each range is used for design. The cross-sections are generic COTS
values unless the row cites part data: no heavy-ion or proton data were found for the STM32L4R5, the FM25V20A, the
RM3100, the EPS parts or the RF parts. A CREME96 run with part-level Weibull parameters (RAD-V4) replaces this table
when the part data exist.

### 5.2 Rates

| Part | Effect | Cross-section basis | Rate low (/day) | Rate high (/day) | High, 2 years |
|---|---|---|---|---|---|
| STM32L4R5 SRAM (640 KB, 5.24 Mbit) | SEU (proton) | generic COTS 90 nm SRAM, 1e-14 to 1e-13 cm²/bit | 0.079 | 0.79 | 580 |
| STM32L4R5 core and peripherals | SEFI needing a reset (proton) | generic COTS MCU, 1e-11 to 1e-10 cm²/device | 1.5e-05 | 1.5e-04 | 0.11 |
| STM32L4R5 | SEL (proton-induced) | no part data found; 0 to 1e-11 cm² | 0 | 8e-06 | 0.006 |
| FM25V20A FRAM (2 Mbit, each) | SEU in the cell array | ferroelectric cells are not upset: 0 | 0 | 0 | 0 |
| FM25V20A FRAM | SEFI or SEL (CMOS periphery) | no part data found; 0 to 1e-11 cm² | 0 | 8e-06 | 0.006 |
| Epson M-G370PDT0 gyro | SEL or SEFI | TID data only; SEE unknown: 1e-11 to 1e-10 cm² | 1.5e-05 | 1.5e-04 | 0.11 |
| Jetson Orin module (SoC), 10 % duty | SEFI (proton) | Orin at 200 MeV ≈ 7e-11 cm² (indicative, conference slides); 7e-11 to 7e-10 | 1.1e-05 | 1.1e-04 | 0.08 |
| Jetson LPDDR5 (8 GB), 10 % duty | SEU (proton) | DDR-class DRAM, 1e-19 to 1e-17 cm²/bit | 0.001 | 0.1 | 75 |
| Jetson module (SoC and DRAM), 10 % duty | SEL (proton-induced) | no part data; 0 to 1e-10 cm² | 0 | 8e-06 | 0.006 |
| RF: receiver/TNC MCU, CTCSS decoder, exciter | SEFI or SEL | COTS, no data: 1e-11 to 1e-10 cm² each, 3 devices | 4.5e-05 | 4.5e-04 | 0.33 |
| GNSS receiver, 5 % duty | SEFI or SEL | COTS, no data: 1e-11 to 1e-10 cm² | 7.5e-07 | 7.5e-06 | 0.006 |

The 10 % Jetson duty is a bound: the mission simulation's two years power the module about 6 % of the time (8,145 Wh at
about 7.3 W, SQSO-SIL-001 F).

What the numbers mean, and what the flight software does about them (as built, 2026-10-27):

- **OBC SRAM.** 0.08–0.8 upsets a day over the whole SRAM. The live state is in SRAM2, the STM32L4R5's only RAM with
  parity: every static (43,452 B) and the 8 KB stack (`software/flight/docs/MEMORY_POLICY.md` §3.2, RIDs AV-07,
  AV-13), 51,644 B or 7.9 % of the SRAM, so **0.006–0.06 upsets a day land in live state: 5–45 in two years**.
  `software/rust/firmware/src/parity.rs` (SRS-F132) erases SRAM2 before use (no read of power-on parity), and an
  upset is caught when the byte is read: the parity NMI records the cause in SRAM1 and resets at once; the cause is
  logged at the next boot and the state is reloaded from FRAM under CRC. An upset in a byte written before it is read
  does no harm. The option bit `SRAM2_PE` that enables parity is set when the board is programmed, never by the flight
  software; every boot reports it in the console and in HK2 `PE`, and logs it when off (SRS-F154). The expected cost is
  therefore 5–45 immediate, logged resets in two years. SRAM1 and SRAM3 hold nothing live (only the NMI record).
- **Critical persisted state.** The TX-OFF latch, the operations record and the passivation record are re-read every
  60 s and a disagreement with the RAM copy is settled toward the safe side (SRS-F133). Every FRAM record has a CRC-32
  and two slots and is written to both FRAMs (U3 and the mirror U36); the loader reads the mirror when U3 fails or
  gives no valid copy, and with both unreadable the FRAM is declared failed after three one-minute checks, the safe
  defaults stay, and FRAM_BYPASS makes the CRC-protected RAM copies authoritative (SRS-F160, MEMORY_POLICY §8.2).
- **OBC flash.** The flash has ECC (8 bits per 64-bit word): single errors are corrected in hardware; a double error
  raises the same NMI, which records the address and resets at once (`parity.rs`, SRS-F132). No periodic image CRC
  check is made (SQSO-FSW-RAD-001 C §11).
- **OBC SEFI and SEL.** One SEFI in 7–70 years and a latch-up chance of under 1 % in two years. A SEFI is cleared by
  the IWDG (25.6 s, refreshed only when every task has checked in), by the external supervisor TPS3850 (SRS-F146,
  AV-02), or by the EPS controller, which power-cycles the OBC after 600 s without its heartbeat (SRS-F162). A
  latch-up is cleared by SW10 (§5.3), which the OBC does not control (AV-08).
- **FRAM SEFI or SEL.** Under 1 % in two years per chip. Both chips are on the OBC rail, so a latch-up is cleared by
  SW10; a SEFI is cleared by the OBC reset that follows a failed read, and the mirror carries the records meanwhile
  (SRS-F160).
- **Jetson.** While powered the module sees 0.01–1.0 upsets a day (SoC SEFI and DRAM SEU together; the DRAM term
  dominates). The finding TB-11 figure (68,700 a day, 10⁻⁶/bit/day) is 5–7 orders of magnitude pessimistic for
  DDR-class DRAM. **Per burst the chance of any upset is 1.4 × 10⁻⁵ to 1.4 × 10⁻³ for a 120 s batch and 7.2 × 10⁻⁵
  to 7.2 × 10⁻³ for the 10-minute burst the design allows** (BASELINE §6, SRS-F151). Revision E's "under 1.4 × 10⁻⁴
  per 120 s burst" applied the 10 %-duty rate as if the module were on all the time and was 10× low (the panel's
  6.9 × 10⁻³ for 10 min uses the rounded 0.1 a day). The payload's outputs pass the OBC's content filter and line
  CRC, so an upset is a lost or rejected batch, not a bad beacon.
- **Jetson latch-up.** No SEL data exist for the Orin Nano; the proton bound above is under 1 % in two years, and
  heavy ions can add to it. The protection is in hardware on the carrier and the EPS (SQSO-JET-BRD-001 E §12,
  latch-up coverage): the carrier's U2 TPS259474L breaks VDD_IN above 2.26–2.76 A within 3.2 ms and latches; U3
  limits the 3.3 V side above 2.8 A; the EPS eFuse U19 TPS16630 latches off above 1.10–1.30 A at VBAT (14.7–17.4 W)
  within 1 ms; and the OBC's guard (`payload_guard`, SQSO-PAYLOAD-001 §11.2) cuts both lines on one sample ≥ 12 W
  (≤ 1 s) or two above 8 W (≤ 2 s). **A micro-latch-up adding less than about 0.5 W to the capped 7.5 W burst stays
  below the 8 W guard and is not detected: it lasts until the burst ends, at most 10 minutes** (the burst limit and the
  3.6 kJ energy gate of SRS-F151, which such a latch-up only shortens), and is accepted for an expendable payload. The
  mission simulation's one Jetson latch-up in two years (9,216 power-ons, modelled at one per 60 powered days) was cut
  by the 8 W guard within 1.0 s (SQSO-SIL-001 F §7).
- **RF receiver.** Under one event in the mission. The always-on receiver path recovers without ground action
  (97.207(b)): `software/rust/sqso-fsw/src/receiver.rs` power-cycles `RADIO_EN` for 2 s after 24 h with the rail on and
  neither a valid frame nor a carrier, never during a pass and at most once a day (SRS-F115, RID FSW-09); RX_CD and
  RX_FAULT are qualified with `RADIO_EN` and RX_FAULT is ignored for 1.5 s after switch-on (SRS-F152); a carrier stuck
  for more than 1 h is not activity, and the receiver is held "known failed" only with a cycle tried, a path fault and
  the loss-of-contact timeout expired (SRS-F116, RIDs REL-04, SMA-B11). On the RF board the TNC re-writes and verifies
  the CC1200, LMX2571 and CMX138A registers every 10 s and at every key-up (SQSO-RF-BRD-001 §12), which clears a
  register upset without a power cycle. The mission simulation's receiver hangs (one per 120 powered days) are each
  cleared within 26 h.
- **GNSS.** Under one event in the mission; powered only for sessions of ≤ 5 min on its own tripped rail.

### 5.3 Latch-up thresholds for the switched rails

The latch-up trip must sit above the load's worst operating current and below the current a latch-up adds, and act
faster than the latch-up can damage the part (typically milliseconds to seconds; a rule of thumb is ≤ 10 ms trip,
≥ 1 s off).

The trips are those of the EPS board (`hardware/electrical/eps/EPS_BOARD_DESIGN.md` SQSO-EPS-BRD-001 §8.3) and,
for the three sensor rails, of the OBC board (`hardware/electrical/obc/OBC_BOARD_DESIGN.md` SQSO-OBC-BRD-001 §9.1).
Every switch limits the current within 2 µs; a TPS2553-Q1 then removes the rail after its 8 ms fault deglitch and
retries by itself.

| Rail | Normal maximum | Trip, nominal (min–max) | Trip time | Off time | Element | Status |
|------|----------------|-------------------------|-----------|----------|---------|--------|
| Jetson (eFuse U19 on `JETSON_PWR` PA8, in series with SW1 on `PAYLOAD_EN` PB2; SQSO-ICX-001 §6 row 26, PA2 is the spreader thermistor) | 8 W ≈ 0.62 A at 13 V; 0.93 A at the 12 W software guard | eFuse **1.20 A (1.10–1.30 A)**; SW1 1.99 A (1.71–2.27 A) behind it | ≤ 1 ms (hardware) | latch-off; the OBC re-enables | TPS16630 eFuse; TPS1H100B-Q1 switch | Designed. The 0.75 A first proposed here would sit only 1.34× above the 8 W load at the low bus; a latch-up (amperes) trips 1.20 A. The software guard (`payload_guard`, 12 W single sample, 0.93 A) acts first (`docs/hardware/PAYLOAD_DESIGN.md` §11.2); the carrier's own breaker U2 is in §5.2 |
| OBC 3.3 V (SW10) | 25–40 mA | **0.15 A (0.130–0.176 A)** | ≤ 10 ms | 0.28 s, auto-retry, no command | TPS2553-Q1 U38 | Designed: the power-cycling supervisor the OBC cannot be for itself (AV-08) |
| EPS controller (SW9) | 12 mA | **75 mA (50–100 mA)** | ≤ 10 ms | 0.28 s, auto-retry | TPS2553-Q1 U27 | Designed; no protection depends on the controller |
| Radio (`RADIO_EN` PB5, SW2) | 38 mA (receiver, TNC, exciter; SQSO-RF-BRD-001) | **0.12 A (0.102–0.141 A)** | ≤ 10 ms | 1.3 s, auto-retry | TPS2553-Q1 U22 | Designed. The 0.3 A first proposed could not see a latch-up of a 38 mA load; the EPS controller also reports the rail current so the OBC can trip it in software |
| PA (`RF_TX_EN` PA3, SW3, the TX-OFF switch) | 0.54 A at 1 W RF | **0.99 A (0.918–1.071 A)** | ≤ 10 ms | 0.28 s, auto-retry | TPS2553-Q1 U23 | Designed |
| ADCS sensors (`ADCS_EN` PB6, SW4 3V3_ADCS) | 49 mA with the backup gyro on | **0.30 A (0.265–0.339 A)**, the backstop for the three rails below | ≤ 10 ms | 0.28 s, auto-retry | TPS2553-Q1 U24 | Designed; since EPS board Version B the coils are no longer on this rail |
| Gyro M-G370PDT0 (`GYRO_EN` PF6) | 26 mA | 75 mA (50–100 mA) | ≤ 10 ms | 0.13 s, auto-retry | TPS2553-Q1 U5 on the OBC | Designed (AV-03). The 40 mA first proposed is below the part's lowest setting; a latch-up (hundreds of mA) still trips 50 mA |
| Backup gyro BMI088 (`IMU2_EN` PF7) | 8 mA | 75 mA (50–100 mA) | ≤ 10 ms | 0.13 s, auto-retry | TPS2553-Q1 U6 on the OBC | Designed (AV-03) |
| Magnetometer RM3100 and sun-sensor front end (`SNS_EN` PF8) | 15 mA | 75 mA (50–100 mA) | ≤ 10 ms | 0.13 s, auto-retry | TPS2553-Q1 U7 on the OBC | Designed (AV-03): no longer shared with the coil drive |
| Magnetorquer bridges (`ADCS_EN` PB6, SW11 VBAT_MTQ) | 0.104 A at 1.16 W (three rods saturated, 11.2 V); 0.23 A at the bridges' hardware limit | **0.361 A (0.322–0.401 A)** | ≤ 10 ms | 1 s off, 40 ms on, auto-retry | TPS1H000-Q1 U39 | Designed (EPS board Version B) |
| GNSS receiver (`GNSS_EN` PE5, SW7) | ≈ 100 mA peak | **0.20 A (0.175–0.232 A)** | ≤ 10 ms | 1.3 s, auto-retry | TPS2553-Q1 U25 | Designed (AV-04). The flight software also powers the receiver only for sessions (`gnss_schedule.rs`, ≤ 5 min) |
| AntS (`DEPLOY_EN` PF3, SW8) | 0.61 A while burning | **1.37 A (1.287–1.465 A)** | ≤ 10 ms | 0.28 s, auto-retry | TPS2553-Q1 U26 | Designed; powered only for deployment |

TB-11's point stands: a 3 A, 100 ms over-current trip on a 2 A load cannot detect a latch-up. The trips above are
1.6–9× the normal maximum at their nominal setting (the Jetson eFuse 1.9× its 8 W load) and act in hardware. They are
verified by RAD-V5 and V-EPS-10.

**Parts not on a switched rail.** Some parts cannot sit behind a latch-up switch because they feed the switches or must
never lose power:

| Part | Supply | What limits a latch-up | Residual |
|------|--------|------------------------|----------|
| RV-3028-C7 mission clock (OBC U4) | 3V3_RTC: 3V3_AO through 1.0 kΩ (≤ 2 µA normal) | The 1.0 kΩ limits the current to ≤ 3.3 mA, which cannot overheat the part | A latch-up that holds at 3.3 mA stops the clock until 3V3_AO is lost; the mission clock is then re-set from a GNSS fix or by the ground (SQSO-OBC-BRD-001 §8). Not mission-ending; RAD-V4 |
| EPS always-on chain: BQ76952 AFE and BQ771806 protector (pack side); TPS55340-Q1 trackers and LM74700-Q1 ideal diodes (string side); LMR43620-Q1 U15 (3V3_AO) and the references, comparators, timers and logic of the 3V3_AO and VHW domains; the input side of every load switch | Pack, strings, 3V3_AO | Nothing short of the AFE's own discharge protection (which opens the bus) | **No designed power cycle.** These parts need SEL data or a heavy-ion screen (RAD-V4); most are automotive BiCMOS power and analogue parts, for which SEL is less common than for digital CMOS, but that is not data. If RAD-V4 finds a susceptible part, the design adds a current limit or a supply cycle for it |

## 6. Radiation Design Requirements

| Requirement | Value | Basis |
|-------------|-------|-------|
| TID capability of mission-critical parts (OBC, FRAM, supervisor, receiver path, EPS controller, power path, load switches, magnetometer) | ≥ 10 krad(Si) with ≥ 2.1 mm Al-equivalent (sector analysis), or ≥ 2× the sector-analysis dose (BASELINE §6, RAD-01) | §4.2 |
| SEL | Every COTS part with no SEL data on a current-limited switched rail, or shown latch-up-immune by test or published data (the always-on parts of §5.3) | §5.3 |
| SEU, OBC | Live state (statics and stack) in parity-protected SRAM2, NMI reset on a parity or flash double-bit error, NVM records under CRC with two slots on two FRAMs, critical state re-checked every 60 s | §5.2, AV-07, AV-13, SRS-F132, F133, F160 |
| SEFI | IWDG and external supervisor reset; EPS-controller power cycle of the OBC after 600 s without its heartbeat; receiver rail power-cycle after 24 h of silence (FSW-09) | §5.2, SRS-F115, F146, F162 |
| Jetson | Non-critical, expendable (BASELINE §6); hardware latch-up breakers on the carrier and the EPS, the OBC guard, the 10-minute burst limit; output filtered by the OBC | §5.2 |

## 7. Mitigation Status (as built, 2026-10-27)

| Mitigation | Claimed in earlier documents | In the flight software or design today | Gap |
|------------|------------------------------|----------------------------------------|-----|
| Independent watchdog, supervised refresh | Yes | Yes: IWDG 25.6 s, refreshed only when every task checks in (`software/rust/sqso-ffi/src/watchdog_mgr.rs`); external supervisor TPS3850 kicked every 10 s on the same condition (SRS-F146, AV-02); EPS controller power-cycles the OBC after 600 s without its heartbeat (SRS-F162) | Not run on hardware |
| CRC on stored data | Yes | Yes: NVM records with CRC-32 and two slots (`software/rust/sqso-fsw/src/nvm.rs`) | None |
| FRAM mirror | No | Yes: every record written to both FRAMs (U3, U36), read from the mirror when U3 fails; degraded mode and FRAM_BYPASS with both failed (SRS-F160, MEMORY_POLICY §8.2) | Both FRAMs are one part type and lot: no cover for a common TID failure (RAD-V3) |
| SRAM parity | Yes (`software/flight/docs/RADIATION_MITIGATION.md` Rev A) | Yes: statics and stack in SRAM2, erased before use, parity NMI records and resets, cause logged at the next boot (`software/rust/firmware/src/parity.rs`, SRS-F132); `SRAM2_PE` set at programming and reported at every boot (SRS-F154) | Not run on hardware: RAD-V6 |
| TMR of critical variables, memory scrubbing | Yes (Rev A) | Not implemented; replaced by parity, persistence and the 60 s check of the TX-OFF, operations and passivation records (SRS-F133) | Tailored (SQSO-FSW-RAD-001 C §11) |
| Flash ECC handling | Implied | Hardware corrects single errors; double errors raise the NMI, which records the address and resets (SRS-F132) | No periodic image CRC |
| RF register scrub | No | The TNC re-writes and verifies the CC1200, LMX2571 and CMX138A registers every 10 s and at every key-up (SQSO-RF-BRD-001 §12) | TNC firmware not written (new software item) |
| SAA handling | Yes | No | Not needed: the rates of §5 do not justify it |
| Per-rail latch-up trips | Required (BASELINE §6) | Designed in hardware on every switched rail (EPS board §8.3, OBC board §9.1; §5.3), with the carrier breaker U2 and the software guard for the Jetson (≤ 1 s) | Not built: RAD-V5, V-EPS-10. Always-on parts: RAD-V4 (§5.3) |
| Receiver recovery from SEFI | Not addressed | Yes: `software/rust/sqso-fsw/src/receiver.rs` power-cycles the radio after 24 h of silence, never in a pass (SRS-F115); RX_CD and RX_FAULT qualified (SRS-F152); "known failed" evidence (SRS-F116) | The RX_CD and RX_FAULT lines need the RF board EM (RFB-07) |

## 8. Component Radiation Data

See §4.3 (TID) and §5.2 (SEE). The values Revision B gave in its §8.1 (STM32L4R5 30 krad, SEL LET > 60, W25Q128 and
LIS3MDL rows) had no sources and two of the parts are not in the design; they are withdrawn. Revision E's magnetometer
row (M315, MM200 or Lodestone, "met if MM200") is withdrawn: the PNI RM3100 is selected and has no data (RAD-V3).

## 9. SAA Operations

The SAA dominates the trapped-proton dose and the proton SEE rates. No operational SAA avoidance is planned: the rates
of §5 are low enough, the Jetson's bursts are sunlit and short, and the fault responses handle the events. The mission
simulation weights its events 30× in the SAA box (`software/rust/sqso-sil/src/env/radiation.rs`).

## 10. Verification Items

Owners are roles (PROJECT_PLAN §4: the PI holds each until it is named). Dates come from the critical-path schedule
(`mission/SCHEDULE.md`: EM order 2027-01-15, EM layout released 2027-04-27, EM fabrication 2027-04-27 to 2027-06-01,
CDR 2027-09-28); the generated register `docs/verification/VI_REGISTER.md` carries them.

| ID | Item | Method | Pass criterion | Verifies | Owner | When |
|----|------|--------|----------------|----------|-------|------|
| RAD-V1 | AE9/AP9-IRENE (V1.5 or later) with SHIELDOSE-2 and a solar-proton model (ESP-PSYCHIC or SAPPHIRE, 95 %) for 465, 475 and 485 km, 97.31°, 2028-11 + 2 years | SPENVIS or OMERE run, archived | Dose-depth within ±30 % of §4.1 at 1–3 mm, or §4–§6 updated and RAD-V2 and RAD-V3 re-checked against the new dose | BASELINE §6 radiation row | Radiation analyst | Before the EM order, 2027-01-15 (its dose sets the RAD-V3 test level); no hardware needed |
| RAD-V2 | Sector (ray-trace) shielding analysis of each mission-critical part of §4.3 on the CAD model with the EM layout | Analysis (FASTRAD, NOVICE or SPENVIS sector shielding) with the §4.1 485 km dose-depth curve | Every mission-critical part sees ≥ 2.1 mm Al-equivalent, or its TID capability (RAD-V3 or data) is ≥ 2× its sector dose; spot shields for the others are in the EM build | RAD-01, §6 TID requirement | Mechanical lead, with the radiation analyst | EM layout freeze, 2027-04-27 (cross-board check and fabrication release, CPM task LAYOUT), before the EM build |
| RAD-V3 | TID lot sample test of every mission-critical part type without data (§4.3): STM32L4R5, FM25V20A, TPS3850H33-Q1, RV-3028-C7, RM3100, ADS7138-Q1, MSP430FR2355, BQ76952, BQ771806, TPS55340-Q1, LM74700-Q1, LMR43620-Q1, REF3425-Q1, TLV7042-Q1, TPL5111, INA228-Q1, INA3221-Q1, TPS2553-Q1, TPS1H100B-Q1, TPS1H000-Q1, TPS16630, the OBC logic families, DRV8876-Q1, CC1200, LMX2571, STM32L431, CMX138A, PCM1808, the TCXO, BFP640F, RD01MUS3, AD8361 and the RF regulators; or published lot data for the same die | Co-60, biased, MIL-STD-883 TM 1019, steps to 20 krad(Si), with anneal; samples from the EM and FM date/lot code (EEE_PARTS_LIST §2A.2) | Functional with parametrics in the data-sheet limits at 10 krad(Si) (the trip points of the switches and the AFE, the supervisor window, the references and the RM3100 offset and noise as §4.3 says); a failure below 10 krad is a part change or spot shielding, decided before the FM parts order | §6, §4.3 | EEE parts engineer | Samples bought with the EM order, 2027-01-15; test before the CDR (2027-09-28) |
| RAD-V4 | SEL screening with part data: heavy-ion (or 200 MeV proton) of the STM32L4R5, FM25V20A, RM3100, M-G370PDT0, RV-3028-C7, MSP430FR2355, the EPS always-on parts of §5.3 and the RF receive parts, or published data; CREME96 rates replacing §5.2 | Literature search, then test of the parts without data; CREME96 | No SEL to LET 37 MeV·cm²/mg, or the part on a switched, tripped rail; for the always-on parts of §5.3 no SEL, or a design change that limits and cycles the supply | §5.3 | EEE parts engineer, with the radiation analyst | Literature search and test plan with the EM order, 2027-01-15; test before the CDR (2027-09-28) |
| RAD-V5 | Latch-up trip of each switched rail, the Jetson eFuse U19 and the carrier breaker U2 | Test on the EM: inject a step load of 1.5× the trip | Current limited within 1 ms and the rail off within 10 ms, restored by the auto-retry (the Jetson eFuse by the OBC), no damage | §5.3 | EPS designer | EM development tests, with V-EPS-10 and V-JET-6 |
| RAD-V6 | SRAM2 parity and flash ECC handling | Fault injection on the EM flatsat (flip an SRAM2 bit through the debugger with parity on; flash ECC test mode) | NMI, cause recorded and reset at once, cause logged at the next boot, state reloaded from FRAM; HK2 `PE` reports the option bit | AV-07, SRS-F132, SRS-F154 | Flight software lead | EM flatsat software test |

## 11. Risk Assessment

| Risk | Likelihood | Consequence | Mitigation |
|------|------------|-------------|------------|
| TID failure of an OBC, EPS or receiver part within 2 years | Low if RAD-V2 shows ≥ 2.1 mm and RAD-V3 passes at 10 krad; moderate if a part sees ≈ 1.5 mm (margin 1.4 at 485 km) | Loss of mission; loss of the TX-OFF receiver | RAD-V2, RAD-V3, spot shielding |
| A selected part with no data fails RAD-V3 (for example the RM3100 or the STM32L4R5) | Low to moderate | A part change after the EM order, or spot shielding | RAD-V3 before the CDR; samples with the EM order |
| Live-state SEU in OBC SRAM | High (5–45 in two years) | Low: an immediate, logged reset, state from FRAM | Parity NMI (AV-07, SRS-F132); RAD-V6 |
| OBC latch-up | Low (< 1 %) | Cleared by SW10 without the OBC | AV-08, RAD-V5 |
| Latch-up of an always-on EPS part | Low (no data) | No power cycle short of the AFE: possible loss of mission | RAD-V4 |
| Receiver SEFI | Low | Loss of telecommand for up to 24 h | FSW-09 recovery (SRS-F115) |
| Jetson SEFI, DRAM upset or latch-up | ≤ 7.2 × 10⁻³ per 10-minute burst (upsets); latch-up below 1 % in two years (proton bound) | Lost batch; a micro-latch-up for up to 10 min | OBC filter, burst limit, carrier and EPS breakers, OBC guard |
| Large solar particle event | Moderate in 2028–2030 | +1–2 krad behind 2 mm | Included at 95 % in §4.1 |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §2 orbit, design life and launch year from BASELINE §2 and §3 (SSO ≈ 97.4°, 500 km target, 525 km cap; was 51.6°, ISS); note that the §3–§5 values depend on the old orbit and must be re-run (finding TB-03). No analysis values changed | SMART-QSO Team |
| C | 2026-10-27 | CDR re-run for the baseline orbit and life (TB-03, TB-11): §3 sources (SPENVIS SHIELDOSE-2 AE-8/AP-8 at 500 km, Shields-1 polar LEO model and measurements, King solar protons, GCR from Shields-1), §4 dose-depth for 1–3 mm at 500 and 525 km (5.3 / 5.9 krad behind 2 mm in 2 years; the 10 krad design level is 1.90× / 1.69×, below the stated 2×), part TID, comparison with the mission simulation; §5 SEE rates per part and rail latch-up trips; §6–§7 requirements and as-built mitigation status; §10 verification items RAD-V1–V6. Script `software/simulation/cdr/avionics/radiation_tid_see.py`. Revision B's §3–§5 and §8.1 values withdrawn | SMART-QSO Team (CDR avionics review) |
| D | 2026-10-27 | CDR 2026 consistency pass: §2 altitude from BASELINE Rev E (475 km target, 485 km cap; 500 and 525 km kept as bounds, §2–§4 labels); §5.3 trips replaced by the EPS board's current-limited rails (SQSO-EPS-BRD-001 §8.3: OBC 0.15 A, radio 0.12 A, sensors 0.30 A, GNSS 0.20 A, EPS controller 75 mA, AntS 1.37 A, PA 0.99 A, VBAT_MTQ 0.361 A, Jetson eFuse 1.20 A) and the OBC board's sensor rails; §7 trip status; RAD-V1 launch 2028-11, RAD-V5 pass criterion | SMART-QSO Team |
| E | 2026-10-27 | CDR flight software, round 3 (fourth part) | Latch-up table: `JETSON_PWR` on PA8 (SQSO-ICX-001; was PA2) |
| F | 2026-10-27 | CDR panel findings P3F-G-01 and P3F-G-08: §2 inclination 97.31°; §4.1 dose at 465, 475 and 485 km with 2.1 mm rows (2 mm: 4.69 / 4.91 krad at 475 / 485 km, margins 2.13 / 2.04; 2.1 mm at 485 km 4.60 krad, 2.17), 500 and 525 km kept as bounds; §4.2 margins in the band; §4.3 rebuilt from the EEE parts list and the board BOMs (RM3100 selected, no data; load switches, eFuse, EPS controller, AFE, power path, supervisor, RTC, RF parts named); §4.4 against the SIL's RAD-02 model (2.63 krad/yr at 2 mm, exponent 1.35; within 1 % at 1.5–2.5 mm); §5.2 SEU, SEFI and SEL handling as implemented (SRAM2 parity SRS-F132/F154, 60 s check SRS-F133, FRAM mirror SRS-F160, supervisor SRS-F146, EPS heartbeat SRS-F162, receiver recovery SRS-F115/F116/F152, RF register scrub), live state 5–45 upsets in two years; Jetson per-burst upset probability corrected (1.4 × 10⁻³ per 120 s, 7.2 × 10⁻³ per 10-minute burst; Revision E 10× low), Jetson SEL row and latch-up protection (carrier U2, EPS U19, OBC guard; a micro-latch-up below the 8 W guard ends with the burst, ≤ 10 min; SIL latch-up cut in 1.0 s); §5.3 parts not on a switched rail; §6, §7, §11 updated; §10 owners and dated events for RAD-V1 to RAD-V6 (RAD-V2 at the EM layout freeze 2027-04-27, RAD-V3 and RAD-V4 with the EM order 2027-01-15), RAD-V3 and RAD-V4 extended to the selected parts without data; script updated (baseline band, SIL model, burst probabilities) | SMART-QSO Team (CDR radiation analysis) |
