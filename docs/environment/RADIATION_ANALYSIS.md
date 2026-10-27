# Radiation Environment Analysis
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ENV-002
**Revision:** D
**Date:** 2026-10-27
**Reference:** NASA-HDBK-4002A; NASA PD-ED-1260 (radiation design margin); ECSS-E-ST-10-04C; ECSS-Q-ST-60-15C

---

## 1. Purpose

This document gives the radiation environment of the baseline orbit, the total ionizing dose (TID) as a function of
aluminium shielding, the single-event-effect (SEE) rates of the avionics parts, the mitigations, and the verification
items. Revision C re-runs §3–§5 for the baseline orbit and design life (finding TB-03); Revision B's values were for a
500 km, 51.6° orbit and a 12-month mission and are withdrawn.

Every number in §4 and §5 is produced by `software/simulation/cdr/avionics/radiation_tid_see.py` (run it with
`--markdown`). The script runs no environment model itself: it combines published SHIELDOSE-2 outputs and in-orbit
measurements (§3.1), scales them to the orbit and the life, and tabulates them.

## 2. Mission Orbit Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 475 km target, 465–485 km, not above 485 km (`docs/BASELINE_PARAMETERS.md` §3). The analysis is run at 500 km and 525 km, which bound the baseline from above (the dose falls with altitude) |
| Inclination | ≈ 97.4° (sun-synchronous; BASELINE §3) |
| Mission duration | 2 years design life; 12 months minimum (BASELINE §2) |
| Launch | 2028 (BASELINE §3, D7): the declining phase of solar cycle 25 |
| Storage | Ground storage before launch adds less than 1 rad(Si) (natural background); in orbit, the analysis covers the 2-year design life. After passivation (BASELINE §5) no avionics function is required, so the 3.1 years to re-entry are not part of the design dose |

## 3. Method and Inputs

### 3.1 Sources

| ID | Source | What it gives | Use |
|----|--------|---------------|-----|
| T1 | SPENVIS SHIELDOSE-2, AE-8 MAX / AP-8 MAX, 500 km, 1 year, centre of solid Al spheres; report `centre.doc` (2007) on the "Radiation Environment for a Sun Synchronous Orbit" page of cubesat.wikidot.com | Dose-depth curve, 0.05–10 mm Al, split into trapped electrons, bremsstrahlung and trapped protons | The trapped dose. The report does not state the inclination (its SPENVIS project folder is named "45INC"), so T2 checks it |
| T2 | D. L. Thomsen III, *Shields-1 Dosimetry Measurements in Polar Low Earth Orbit*, NASA LaRC, 2022-09-14 (Shields-1, NORAD 43850, 500 km, 85°) | SPENVIS SHIELDOSE-2 (AP-8 MIN / AE-8 MAX) sphere dose behind 1.98 mm Al: 117 rad(Si)/yr protons + 1266 rad(Si)/yr electrons = 1383 rad(Si)/yr. Measured slab dose in orbit (2019, solar minimum): 90.9 ± 2.7 rad/yr behind 1.26 g/cm², 73.6 ± 3.2 behind 3.00 g/cm², 70.0 ± 3.0 behind 6.00 g/cm² | Cross-check of T1 at 2 mm in a polar orbit; GCR term (T4); evidence that the trapped models under-predict thick-shield dose by 2–5× |
| T3 | NASA, *State-of-the-Art of Small Spacecraft Technology*, ch. 6 (Structures, Materials and Mechanisms), 2021, for the Shields-1 orbit; the King (SOLPRO) curve in T2 | Solar protons, 1 year, 95 % confidence: ≈ 750 rad(Si) behind 0.535 g/cm² (2 mm), ≈ 400 rad(Si) behind 1.26 g/cm² | Solar-proton dose, interpolated exponentially in areal density |
| T4 | T2: measured dose behind 6 g/cm² less the trapped-proton model there (70.0 − 13.7 rad/yr) | ≈ 56 rad/yr not in the trapped models: galactic cosmic rays and their secondaries | 50 rad(Si)/yr at every depth |
| T5 | The mission simulation's altitude scaling (`software/rust/sqso-sil/src/env/radiation.rs`) | Trapped dose × exp((h − 500 km)/150 km) | The 525 km bound |
| P1 | T1's trapped-proton spectra (report `trapped.doc`, AP-8 MAX, 500 km) | Orbit-average integral flux: > 10 MeV 21.5, > 30 MeV 17.4, > 50 MeV 14.8, > 100 MeV 9.28 cm⁻² s⁻¹ | Proton SEE rates (§5) |

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

**500 km, 2 years, solid Al sphere, rad(Si)**

| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |
|---|---|---|---|---|---|
| 1.0 | 10,614 | 1,887 | 100 | **12,601** | 0.79 |
| 1.5 | 5,868 | 1,679 | 100 | **7,647** | 1.31 |
| 2.0 | 3,660 | 1,494 | 100 | **5,254** | 1.90 |
| 2.5 | 2,382 | 1,329 | 100 | **3,811** | 2.62 |
| 3.0 | 1,584 | 1,182 | 100 | **2,866** | 3.49 |

**525 km (the former altitude cap, kept as a bound), 2 years, solid Al sphere, rad(Si)**

| Depth (mm Al) | Trapped e⁻ + p | Solar protons (95 %) | GCR | Total | Margin of the 10 krad design level |
|---|---|---|---|---|---|
| 1.0 | 12,539 | 1,887 | 100 | **14,526** | 0.69 |
| 1.5 | 6,932 | 1,679 | 100 | **8,711** | 1.15 |
| 2.0 | 4,324 | 1,494 | 100 | **5,917** | 1.69 |
| 2.5 | 2,814 | 1,329 | 100 | **4,243** | 2.36 |
| 3.0 | 1,872 | 1,182 | 100 | **3,153** | 3.17 |

Electrons dominate below about 2.5 mm; the curve falls by a factor of 4.4 from 1 to 3 mm. Solar protons are 28 % of the
2 mm total, so a quiet two years would give less, and a large event in the declining phase of cycle 25 more: T3 is a
95 % confidence level for one year.

### 4.2 Against the baseline design level

BASELINE §6 says "design to 10 krad (≥ 2× the 2-year dose behind 2 mm Al)". The 2-year dose behind 2 mm is
**5.3 krad(Si) at 500 km and 5.9 krad(Si) at 525 km**, so 10 krad is **1.90× and 1.69×**: the stated margin of 2 is
**not met** at 2 mm. The margin of 2 needs **2.1 mm Al-equivalent at 500 km and 2.3 mm at 525 km**; a margin of 3 (the
NASA PD-ED-1260 value for parts without lot data) needs 2.7 and 2.9 mm. The earlier "2.6 krad/yr behind 2 mm" of
Revision B agrees with this analysis (2.63 krad/yr at 500 km) by coincidence; its 51.6° breakdown does not.

A 1U wall gives about 1.5–2 mm Al-equivalent (frame, PCB panels and cells), and boards inside the stack are shielded
by the boards around them, so the electronics see more than the wall. The shielding of each part is not known until the
layout exists: a sector analysis (RAD-V2) on the CAD model sets it. Parts whose TID capability is near 10 krad and that
see less than 2.1 mm in the sector analysis need spot shielding or a part with more TID margin.

### 4.3 Part TID capability

| Part | Function | TID data | Required (RDM 2, 2.1 mm, 500 km) | Status |
|------|----------|----------|-----------------------------------|--------|
| STM32L4R5ZI | OBC | No lot data found for the L4R5; ST L4-series parts in published CubeSat tests are commonly reported functional beyond 20 krad(Si) | 10.5 krad | Open: lot sample TID test (RAD-V3) or heritage report for the L4R5 |
| FM25V20A FRAM | Non-volatile memory | Ferroelectric cells are TID-hard; the CMOS periphery sets the limit (no lot data found) | 10.5 krad | Open (RAD-V3) |
| Epson M-G370PDT0 | Gyro | Pass at 5, 10 and 20 krad(Si) (Epson data, `docs/hardware/BUILD_VS_BUY.md` §5.2) | 10.5 krad | Met (RDM 3.8 behind 2 mm) |
| Bosch BMI088 | Backup gyro | None | 10.5 krad | Open; the backup is unpowered until used, but TID accumulates unbiased |
| Magnetometer (M315, MM200 or Lodestone) | ADCS | AAC MM200: 30 krad (vendor) | 10.5 krad | Met if MM200 is chosen |
| Jetson Orin Nano module | AI payload | Orin AGX tested for SEE and TID ("Single Event Effects and Total Ionizing Dose Radiation Testing of NVIDIA Jetson Orin AGX System on Module", 2023; only the listing was reachable); none found for the Nano module | Non-critical (BASELINE §6) | Expendable: its loss ends the gated experiment only |
| RF parts (CMX138A, exciter, PA, TNC MCU) | Transponder, beacon, telecommand receiver | None found | 10.5 krad | Open (RAD-V3): the 435 MHz receiver must survive the mission (97.207(b)) |
| Point-of-load regulators, load switches | EPS, PDU | None in the repository | 10.5 krad | Open: parts not yet selected |

### 4.4 Comparison with the mission simulation

`software/rust/sqso-sil/src/env/radiation.rs` models TID as 1.5 krad(Si)/yr behind 2 mm at 500 km, scaled by
(2 mm / t)^1.2:

| Depth (mm Al) | Simulation (rad(Si)/yr) | This analysis | Simulation / analysis |
|---|---|---|---|
| 1.0 | 3,446 | 6,301 | 0.55 |
| 1.5 | 2,118 | 3,823 | 0.55 |
| 2.0 | 1,500 | 2,627 | 0.57 |
| 2.5 | 1,148 | 1,905 | 0.60 |
| 3.0 | 922 | 1,433 | 0.64 |

The simulation under-predicts by a factor of 1.6–1.8 (it leaves out the solar protons and takes a lower trapped
value). Its dose only feeds the `tid_krad` total in the summary, not a pass criterion, so no SIL result changes. The
simulation's model should be set to 2.63 krad(Si)/yr at 2 mm with an exponent of 1.35 (the fit of §4.1 between 1 and
3 mm); that file belongs to the power and thermal reviewer and is recommended to the chair (RID RAD-02).

The simulation's SEE rates (one OBC reset per 90 days, one Jetson latch-up per 60 powered days, one FRAM flip per 180
days) are 70–700 times the estimates of §5. That is right for its purpose, which is to exercise the fault responses
often in a two-year run, and it must not be read as a prediction.

## 5. Single-Event Effects

### 5.1 Method

Proton-induced rates are σ × Φ(> E), with Φ from P1 (> 30 MeV for upsets, > 100 MeV for proton-induced latch-up, which
needs nuclear reactions in the parasitic structure). Heavy-ion (GCR) contributions are not computed: at 97.4° the polar
passes are open to GCR, and for parts with low LET thresholds they can match the proton term, so the high end of each
range is used for design. The cross-sections are generic COTS values unless the row cites part data: no heavy-ion or
proton data were found for the STM32L4R5, the FM25V20A or the RF parts. A CREME96 run with part-level Weibull
parameters (RAD-V4) replaces this table when the parts are frozen.

### 5.2 Rates

| Part | Effect | Cross-section basis | Rate low (/day) | Rate high (/day) | High, 2 years |
|---|---|---|---|---|---|
| STM32L4R5 SRAM (640 KB, 5.24 Mbit) | SEU (proton) | generic COTS 90 nm SRAM, 1e-14 to 1e-13 cm²/bit | 0.079 | 0.79 | 580 |
| STM32L4R5 core and peripherals | SEFI needing a reset (proton) | generic COTS MCU, 1e-11 to 1e-10 cm²/device | 1.5e-05 | 1.5e-04 | 0.11 |
| STM32L4R5 | SEL (proton-induced) | no part data found; 0 to 1e-11 cm² | 0 | 8e-06 | 0.006 |
| FM25V20A FRAM (2 Mbit) | SEU in the cell array | ferroelectric cells are not upset: 0 | 0 | 0 | 0 |
| FM25V20A FRAM | SEFI or SEL (CMOS periphery) | no part data found; 0 to 1e-11 cm² | 0 | 8e-06 | 0.006 |
| Epson M-G370PDT0 gyro | SEL or SEFI | TID data only; SEE unknown: 1e-11 to 1e-10 cm² | 1.5e-05 | 1.5e-04 | 0.11 |
| Jetson Orin module (SoC), 10 % duty | SEFI (proton) | Orin at 200 MeV ≈ 7e-11 cm² (indicative, conference slides); 7e-11 to 7e-10 | 1.1e-05 | 1.1e-04 | 0.08 |
| Jetson LPDDR5 (8 GB), 10 % duty | SEU (proton) | DDR-class DRAM, 1e-19 to 1e-17 cm²/bit | 0.001 | 0.1 | 75 |
| RF: receiver/TNC MCU, CTCSS decoder, exciter | SEFI or SEL | COTS, no data: 1e-11 to 1e-10 cm² each, 3 devices | 4.5e-05 | 4.5e-04 | 0.33 |
| GNSS receiver, 5 % duty | SEFI or SEL | COTS, no data: 1e-11 to 1e-10 cm² | 7.5e-07 | 7.5e-06 | 0.006 |

What the numbers mean:

- **OBC SRAM.** 0.08–0.8 upsets a day over the whole SRAM. The image uses 46.6 KB (`.data` 22.0 KB, `.bss` 16.6 KB, the
  8 KB stack; `arm-none-eabi-size` of the 2026-10-27 release image), 7.3 % of it, so **0.006–0.06 upsets a day land in
  live state: 4–42 in two years**. The STM32L4R5's SRAM1 and SRAM3 have no parity or ECC; only SRAM2 (64 KB) has
  hardware parity, and the firmware links all 640 KB as one region with no parity handling. Today an upset in live
  state is caught only if it hangs a stage (watchdog) or corrupts a value a range check rejects. **RID AV-07**: link
  `.data`, `.bss` and the stack into SRAM2 (they fit: 46.6 of 64 KB), enable SRAM2 parity (option bit `SRAM2_PE`) and
  handle the parity NMI by logging and resetting (state is reloaded from FRAM under CRC).
- **OBC flash.** The flash has ECC (8 bits per 64-bit word): single errors are corrected, double errors raise an NMI.
  The firmware has no NMI handler beyond the default; a double error therefore ends in the 30 s watchdog reset rather
  than an immediate, logged reset (part of AV-07).
- **OBC SEFI and SEL.** One SEFI in 7–70 years and a latch-up chance of under 1 % in two years. The watchdog covers the
  SEFI; the latch-up needs the OBC's own rail to be current-limited and power-cycled by the EPS, independent of the OBC
  (**RID AV-08**: the EPS design must say so; no OBC-rail latch-up protection is described in the repository).
- **Jetson.** At 10 % duty, a SEFI is expected in about 25 years of operation and DRAM upsets at 0.001–0.1 a day. The
  finding TB-11 figure (68,700 a day, 10⁻⁶/bit/day) is 5–7 orders of magnitude pessimistic for DDR-class DRAM. Per 120 s
  burst the chance of any upset is under 1.4 × 10⁻⁴. The payload's outputs pass the OBC's content filter and line CRC,
  so an upset is a lost batch, not a bad beacon.
- **RF and GNSS.** Under one event in the mission each. The always-on receiver path must recover without ground action
  (97.207(b)): a SEFI there is cleared only by the OBC power-cycling the radio rail, which the flight software does not
  do today (**RID FSW-09**: a receiver-silence watchdog, e.g. power-cycle `RADIO_EN` after no valid TNC traffic and no
  TNC status for 24 h).

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
| Jetson (eFuse U19 on `JETSON_PWR` PA2, in series with SW1 on `PAYLOAD_EN` PB2) | 8 W ≈ 0.62 A at 13 V; 0.93 A at the 12 W software guard | eFuse **1.20 A (1.10–1.30 A)**; SW1 1.99 A (1.71–2.27 A) behind it | ≤ 1 ms (hardware) | latch-off; the OBC re-enables | TPS16630 eFuse; TPS1H100B-Q1 switch | Designed. The 0.75 A first proposed here would sit only 1.34× above the 8 W load at the low bus; a latch-up (amperes) trips 1.20 A. The software guard (`payload_guard`, 12 W single sample, 0.93 A) acts first (`docs/hardware/PAYLOAD_DESIGN.md` §11.2) |
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
1.6–9× the normal maximum at their nominal setting (the Jetson eFuse 1.9× its 8 W load) and act in hardware. They are verified by RAD-V5 and
V-EPS-10.

## 6. Radiation Design Requirements

| Requirement | Value | Basis |
|-------------|-------|-------|
| TID capability of mission-critical parts (OBC, FRAM, receiver path, EPS controller, load switches) | ≥ 10 krad(Si) with ≥ 2.1 mm Al-equivalent (sector analysis), or ≥ 2× the sector-analysis dose | §4.2 |
| SEL | Every COTS part with no SEL data on a current-limited switched rail, or shown latch-up-immune by test | §5.3 |
| SEU, OBC | Live state in parity-protected SRAM, NVM records under CRC with two copies, image under flash ECC | §5.2, AV-07 |
| SEFI | Independent watchdog reset (30 s, `software/flight/docs/RTOS_ARCHITECTURE.md`); receiver rail power-cycle after 24 h of silence (FSW-09) | §5.2 |
| Jetson | Non-critical, expendable (BASELINE §6); OBC-enforced rail trip and temperature limits; output filtered by the OBC | §5.2 |

## 7. Mitigation Status (as built, 2026-10-27)

| Mitigation | Claimed in earlier documents | In the flight software or design today | Gap |
|------------|------------------------------|----------------------------------------|-----|
| Independent watchdog, supervised refresh | Yes | Yes: IWDG 30 s, refreshed only when every stage checks in (`software/rust/sqso-ffi/src/watchdog_mgr.rs`) | None |
| CRC on stored data | Yes | Yes: NVM records with CRC-32 and two slots (`software/rust/sqso-fsw/src/nvm.rs`) | None |
| SRAM ECC/parity, TMR of critical variables, memory scrubbing | Yes (`software/flight/docs/RADIATION_MITIGATION.md` Rev A) | **No.** No TMR, no scrubbing, no parity configuration in the Rust code | AV-07 |
| Flash ECC handling | Implied | Hardware corrects single errors; no NMI handler | AV-07 |
| SAA handling | Yes | No | Not needed: the rates of §5 do not justify it |
| Per-rail latch-up trips | Required (BASELINE §6) | Designed in hardware on every switched rail (EPS board §8.3, OBC board §9.1; §5.3), with the software guard for the Jetson (1 s) as the first response | Not built: RAD-V5, V-EPS-10 |
| Receiver recovery from SEFI | Not addressed | No | FSW-09 |

## 8. Component Radiation Data

See §4.3 (TID) and §5.2 (SEE). The values Revision B gave in its §8.1 (STM32L4R5 30 krad, SEL LET > 60, W25Q128 and
LIS3MDL rows) had no sources and two of the parts are not in the design; they are withdrawn.

## 9. SAA Operations

The SAA dominates the trapped-proton dose and the proton SEE rates. No operational SAA avoidance is planned: the rates
of §5 are low enough, the Jetson's bursts are sunlit and short, and the fault responses handle the events. The mission
simulation weights its events 30× in the SAA box (`software/rust/sqso-sil/src/env/radiation.rs`).

## 10. Verification Items

| ID | Item | Method | Pass criterion | Verifies | When |
|----|------|--------|----------------|----------|------|
| RAD-V1 | AE9/AP9-IRENE (V1.5 or later) with SHIELDOSE-2 and a solar-proton model (ESP-PSYCHIC or SAPPHIRE, 95 %) for 500 km and 525 km, 97.4°, 2028-11 + 2 years | SPENVIS or OMERE run, archived | Dose-depth within ±30 % of §4.1 at 1–3 mm, or §4–§6 updated | BASELINE §6 radiation row | Before the parts freeze (fabrication start) |
| RAD-V2 | Sector (ray-trace) shielding analysis of each mission-critical part on the CAD model | Analysis (e.g. FASTRAD, NOVICE, or SPENVIS sector shielding) | Each part's 2-year dose × 2 ≤ its TID capability | §6 TID requirement | When the layout is frozen, before the EM build |
| RAD-V3 | TID lot sample test of the STM32L4R5, FM25V20A, the receiver-path RF parts and the EPS/PDU load switches and regulators, or published lot data | Co-60 test to 20 krad(Si), biased, MIL-STD-883 TM 1019 | Functional with parametrics in limits at 10.5 krad(Si) | §6 | Before the flight build |
| RAD-V4 | SEE rates with part data: heavy-ion (or 200 MeV proton) SEL screening of the STM32L4R5, FM25V20A, M-G370PDT0 and RF parts, or published data; CREME96 rates | Test or literature; CREME96 | No SEL to LET 37 MeV·cm²/mg, or the part on a switched, tripped rail | §5.3 | Before the flight build |
| RAD-V5 | Latch-up trip of each switched rail | Test on the EM: inject a step load of 1.5× the trip | Current limited within 1 ms and the rail off within 10 ms, restored by the auto-retry (the Jetson eFuse by the OBC), no damage (with V-EPS-10) | §5.3 | EM functional test |
| RAD-V6 | SRAM parity and flash ECC handling | Fault injection on the EM (flip an SRAM2 bit through the debugger; flash ECC test mode) | NMI, fault logged, reset, state reloaded from FRAM | AV-07 | EM software test |

## 11. Risk Assessment

| Risk | Likelihood | Consequence | Mitigation |
|------|------------|-------------|------------|
| TID failure of an OBC or receiver part within 2 years | Low if RAD-V2 shows ≥ 2.1 mm; moderate if a part sees ≈ 1.5 mm (RDM 1.3) | Loss of mission; loss of the TX-OFF receiver | RAD-V2, RAD-V3, spot shielding |
| Live-state SEU in OBC SRAM | High (4–42 in two years) | Low to moderate (a wrong value until reset) | AV-07 |
| OBC latch-up | Low (< 1 %) | Loss of mission if not power-cycled | AV-08 |
| Receiver SEFI | Low | Loss of telecommand until power-cycled | FSW-09 |
| Jetson SEFI or DRAM upset | Low per burst | Lost batch | OBC filter, burst power-off |
| Large solar particle event | Moderate in 2028–2030 | +1–2 krad behind 2 mm | Included at 95 % in §4.1 |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §2 orbit, design life and launch year from BASELINE §2 and §3 (SSO ≈ 97.4°, 500 km target, 525 km cap; was 51.6°, ISS); note that the §3–§5 values depend on the old orbit and must be re-run (finding TB-03). No analysis values changed | SMART-QSO Team |
| C | 2026-10-27 | CDR re-run for the baseline orbit and life (TB-03, TB-11): §3 sources (SPENVIS SHIELDOSE-2 AE-8/AP-8 at 500 km, Shields-1 polar LEO model and measurements, King solar protons, GCR from Shields-1), §4 dose-depth for 1–3 mm at 500 and 525 km (5.3 / 5.9 krad behind 2 mm in 2 years; the 10 krad design level is 1.90× / 1.69×, below the stated 2×), part TID, comparison with the mission simulation; §5 SEE rates per part and rail latch-up trips; §6–§7 requirements and as-built mitigation status; §10 verification items RAD-V1–V6. Script `software/simulation/cdr/avionics/radiation_tid_see.py`. Revision B's §3–§5 and §8.1 values withdrawn | SMART-QSO Team (CDR avionics review) |
| D | 2026-10-27 | CDR 2026 consistency pass: §2 altitude from BASELINE Rev E (475 km target, 485 km cap; 500 and 525 km kept as bounds, §2–§4 labels); §5.3 trips replaced by the EPS board's current-limited rails (SQSO-EPS-BRD-001 §8.3: OBC 0.15 A, radio 0.12 A, sensors 0.30 A, GNSS 0.20 A, EPS controller 75 mA, AntS 1.37 A, PA 0.99 A, VBAT_MTQ 0.361 A, Jetson eFuse 1.20 A) and the OBC board's sensor rails; §7 trip status; RAD-V1 launch 2028-11, RAD-V5 pass criterion | SMART-QSO Team |
