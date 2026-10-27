# Antenna Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-ANT-001
**Version:** 2.2
**Date:** 2026-10-27
**Status:** CDR. Antenna bought (ISIS AntS 1U–3U, `docs/hardware/BUILD_VS_BUY.md` §2.6); configuration recommended here, to be confirmed by RFQ and a PI decision. Not measured.

Version 1.0 described a custom stainless-tape 2 m monopole at 145.825 MHz on the −Z face. The baseline buys the ISIS
AntS, adds the 70 cm receive element the telecommand and transponder need (BASELINE §7), and excludes 145.825 MHz.
Version 2.0 replaces it.

---

## 1. Requirements

| ID | Requirement | Source |
|----|-------------|--------|
| ANT-1 | Separate deployable quarter-wave elements for 2 m (145.800–146.000 MHz) and 70 cm (435.000–437.300 MHz); no shared diplexed element | BASELINE §7 |
| ANT-2 | Gain toward the ground over the pass-weighted directions: median ≥ −3 dBi, 10th percentile ≥ −10 dBi, each band, with the wings deployed (proposed replacement for SYS-222; RF_DESIGN RF-P013) | SQSO-LBA-001 §5 |
| ANT-3 | VSWR < 2:1 at the coordinated frequencies, deployed on the flight body | SYS-223 |
| ANT-4 | RF power rating ≥ 1 W continuous (2 m), in vacuum | BASELINE §7 |
| ANT-5 | Isolation between the 2 m and 70 cm elements: ≥ 25 dB at 146 MHz and ≥ 15 dB at 437 MHz (the desense analysis assumes these) | SQSO-LBA-001 §7.3–7.4 |
| ANT-6 | Deployment no earlier than 30 min after separation; ≥ 3 independent physical inhibits between the battery and each release actuator; deployment sensed | CDS Rev 14.1 §2.3.8, §2.4.4; SYS-221, SYS-225 |
| ANT-7 | Stowed within the CDS envelope; no part released | CDS §2.1.1, §2.2 |
| ANT-8 | No ferromagnetic contact in the RF path that can generate a 3rd harmonic of the 2 m signal (passive intermodulation) | SQSO-LBA-001 §7.5 |

## 2. Selected Product

**ISIS AntS 1U–3U** (BUILD_VS_BUY §2.6, public datasheet values; quotes pending):

| Parameter | Value |
|-----------|-------|
| Elements | Up to four tape springs, up to 55 cm (VHF) or 17 cm (UHF) |
| Configurations offered | 4 monopoles; 1 or 2 dipoles; turnstile (with phasing/balun circuitry); dipole plus monopoles |
| Deployment | Each element held by a wire melted by two redundant heating elements, two redundant microcontrollers, deployment-feedback switches; < 3 s above 15 °C |
| Interface | I²C, 3.3 V, < 40 mW |
| RF power | 2 W maximum |
| Mass, envelope | < 90 g; 98 × 98 × 7 mm on one ±Z face; 30 mm centre hole option |
| Heritage | Flown since 2010, > 500 units (vendor statement) |

The elements deploy from the sides of the frame **in the plane of the mounting face**, so each lies along ±X or ±Y of
the body (the face is ±Z). The direction of each element is an ordering option.

## 3. Configuration Against the Spin

The spacecraft spins at 6°/s about +Y, the wing normal, with +Y on the Sun (BASELINE §8, D9). SQSO-LBA-001 §5
evaluated three options along 60 days of passes:

| Option | 2 m elements | Fading | Beacon frames decoded, reference station | Handheld Yagi | Meets ANT-2? |
|--------|--------------|--------|------------------------------------------|---------------|--------------|
| A | One, along ±X (across the spin) | Null sweeps the sky twice a revolution: at 8°/s 8.3 fades below −10 dBi per pass, 5 s each | 96.6 % (98.2 % with the ±5 s beacon jitter as built) | 49 % | Median yes (−2.2 dBi); 10th percentile −9.2 dBi at low elevation, marginal |
| **B** | One, along **+Y** (the spin axis) | No spin modulation; null toward the Sun and away from it: 0.23 fades per pass, about 80 s each | 100 % | 58 % | Yes at low elevation (−6.3 dBi); −12.8 dBi at 30–45° (marginal) |
| **C** | Two, along ±X and ±Y, quadrature through a 90° hybrid | Never below −6 dBi | 100 % | 72 % | **Yes** (−3.6 dBi 10th percentile) |

**Baseline (BASELINE Rev D §4, §7, chair decision with STR-01):** two 2 m elements along +X and −X (across the spin
axis, keeping +Y the major axis of inertia) and two 70 cm elements along +Y and −Y, each pair fed in antiphase as a
dipole (§4.1). The options below are the review's pattern analysis, kept for the record.

**Recommendation (review, superseded by the baseline):** option C if the AntS can carry two 2 m elements and the 70 cm element(s) on one frame with a
phasing network for the 2 m pair (RFQ question 1 below); otherwise option B. The 70 cm element follows the same rule.
Option A, the arrangement the mission simulation assumes (−Z axis, equivalent to ±X), is the worst of the three.

Option C also gives near-circular polarization toward ±Z, which reduces the linear-to-linear fades that dominate
the losses for handheld stations (SQSO-LBA-001 §5.3, finding 2).

## 4. Electrical

| Parameter | Value | Basis |
|-----------|-------|-------|
| λ/4 at 145.9 MHz / 436 MHz | 514 mm / 172 mm free space; tape length per ISIS tuning | |
| Pattern | Dipole-shaped about each element, peak 0 dBi as filed (2.15 dBi directivity less ≈ 2 dB for mismatch and body loss); nulls filled to about −20 dBi by the body and wings [assumption] | SQSO-LBA-001 §2.4 |
| Ground plane | The 1U body (0.05 λ at 2 m, 0.15 λ at 70 cm): at 2 m the body is the other half of an asymmetric dipole, not a ground plane | |
| Wings | Two 1U-face wings with normal +Y, conductive cell substrates near the elements: they shadow and detune; their effect is unknown until VI-RF-01 | |
| Polarization | Linear per element; option C elliptical to circular toward ±Z | |
| Matching | Per ISIS; measured VSWR on the flight body (ANT-3) | |

### 4.1 Feed interface to the RF board (2.1)

| Item | Design (SQSO-RF-BRD-001 §9; W10 v2) |
|------|-------------------------------------|
| 2 m pair (+X, −X) | Fed in antiphase from a lumped lattice balun on the RF board (68 nH, 18 pF; 50 Ω to 72 Ω balanced, 0.25 dB), through two 1.37 mm cables (≤ 0.22 m, Molex 73414-0351 MMCX plugs) from MMCX jacks J3/J4 on the board, matched within 2 mm (0.5°); W10 v3, SQSO-RF-BRD-001 C §4.3 |
| 70 cm pair (+Y, −Y) | Same, 22 nH, 6.0 pF, 0.30 dB; cables matched within 2 mm (1.5°) |
| If ISIS supplies the phasing on the AntS | One cable per band; the board's baluns are depopulated (0 Ω links fitted). RFQ question 1 |
| Static bleed | 100 kΩ to ground at each RF connector on the board |
| Isolation (ANT-5) | ≥ 25 dB at 146 MHz and ≥ 15 dB at 437 MHz between the 2 m and 70 cm ports; the orthogonal pairs help. With these values the board's receive filter holds the 2 m signal at ≤ −55 dBm at the LNA and the desense is ≤ 0.01 dB at 1 W (SQSO-RF-BRD-001 §6.3, §7.5) |
| Power | 1 W continuous at 2 m, split 0.5 W per element (ANT-4) |
| Contacts | Gold-plated connectors at the board; feed contacts on the AntS to be free of ferromagnetic material (ANT-8) |

## 5. Mechanical and Operational

| Item | Value |
|------|-------|
| Face | One ±Z face (98 × 98 × 7 mm). Check against the separation switches, the access port and the GNSS patch (BUILD_VS_BUY §2.6) |
| Release | AntS heaters through the deployment inhibits; the flight software's persisted 30-minute deployment sequence (SYS-221) commands the AntS over I²C or drives its release lines (interface to be fixed with the vendor's ICD) |
| Sensing | AntS feedback switches to the OBC (`GPIO_PIN_ANTENNA_SENSE`, or read over I²C) |
| Materials | Tape springs and hinge contacts to be checked for ferromagnetic contact in the RF path (ANT-8) |

## 6. RFQ Questions to ISISPACE

1. (2.1) Baseline: 2 m elements on opposite sides (±X) and 70 cm elements on the other two (±Y). Does ISIS supply the
   antiphase feed (balun) for each pair on the AntS, and with what connectors? Earlier form of the question: can one
   AntS frame carry two 2 m elements (on adjacent sides, for a quadrature pair) and one or two 70 cm elements,
   with a 90° hybrid for the 2 m pair and a separate 70 cm feed? Isolation between the 2 m and 70 cm ports at 146 and
   437 MHz?
2. Element directions selectable (+Y for the 2 m element)?
3. Measured patterns on a 1U body, both bands, and the effect of deployable panels.
4. Materials of the tape, hinge and feed contacts (passive intermodulation at 3 × 146 MHz).
5. Release interface: I²C commands only, or direct heater lines? Can each heater be fed through the spacecraft's
   three inhibits?
6. Lead time, price, export classification.

## 7. Verification

| ID | Test | Pass criterion | When |
|----|------|----------------|------|
| VI-RF-01 | Pattern of the EM on a flight-like 1U body with wings, both bands, ≥ 2 orthogonal cuts; EM model (NEC/FEKO) agreeing within 2 dB | ANT-2 over the pass-weighted directions of SQSO-LBA-001 §5.2 | EM |
| VI-RF-16 | Deployment, thermal-vacuum cold case, then VSWR | All elements deployed and sensed; VSWR < 2:1 (ANT-3) | EM, FM (TP-RF-005) |
| VI-RF-17 | Port-to-port isolation, deployed | ANT-5 | EM |
| VI-RF-04 | Radiated 3rd harmonic at 1 W, deployed (captures passive intermodulation in the antenna) | ≤ −70 dBc | EM, FM |

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial (custom 2 m monopole, 145.825 MHz) |
| 2.0 | 2026-10-27 | CDR review (communications panel): ISIS AntS 1U–3U as bought (BUILD_VS_BUY §2.6); 70 cm element added; configuration options against the 6°/s spin with their fading and frame statistics (SQSO-LBA-001 §5), option C recommended, B as fall-back; requirements ANT-1 to ANT-8; RFQ questions; verification items. The custom design and 145.825 MHz are removed |
| 2.1 | 2026-10-27 | CDR design completion, RF board | Baseline configuration recorded (2 m pair ±X, 70 cm pair ±Y, antiphase); §4.1 feed interface to the RF board (lattice baluns, matched coax, static bleed, isolation); RFQ question 1 restated |
| 2.2 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Option table re-run at 8°/s and 475 km (SQSO-LBA-001 v3.2 §5) |
