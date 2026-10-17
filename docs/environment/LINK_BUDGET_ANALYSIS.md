# SMART-QSO Link Budget Analysis

**Document ID**: SQSO-LBA-001
**Version**: 2.0
**Date**: 2026-10-16
**Classification**: Unclassified
**Status**: Controlled link budget for the baseline in [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) (§3, §7). **Analysis, not measured.** Every other document that quotes a link margin should cite this one (finding TB-05).

## 1. Purpose

This document gives the RF link budgets for the SMART-QSO baseline: the 1200 bps telemetry beacon and the FM transponder
on the 2 m downlink, and authenticated telecommand and the transponder on the 70 cm uplink. All inputs are listed in §2;
each margin is computed from them in §3 and §4.

### 1.1 Correction to Version 1.0

Version 1.0 computed the ground-station noise density as −174 + 10·log(350) = −148.4 dBm/Hz. The −174 dBm/Hz term
already is kT at 290 K (k = −198.6 dBm/K/Hz, and −198.6 + 10·log 290 = −174.0), so adding 10·log T counted the
temperature twice. The correct density for 350 K is −198.6 + 10·log 350 = **−173.2 dBm/Hz**, 24.8 dB lower than
Version 1.0 used. Every margin in Version 1.0 was therefore about 25 dB too pessimistic. The "+1.6 dB at 30°", the
"2 W required" and the "20° minimum elevation" rules all came from that error and are withdrawn. Version 1.0 also used
the ISS orbit (400 km, 51.6°), 145.825 MHz and 2 W, none of which is in the baseline.

## 2. Link Parameters

### 2.1 Orbital Parameters

| Parameter | Value | Basis |
|-----------|-------|-------|
| Altitude | 500 km target, 525 km cap | BASELINE §3 |
| Orbit | Circular SSO, inclination ≈ 97.4° | BASELINE §3 |
| Period | 94.6 min (500 km) | 2π·√(a³/μ), a = 6,878 km, μ = 398,600 km³/s² |
| Orbital speed | 7.61 km/s | √(μ/a) |

Slant range d to a ground station at elevation E: d = √((Rₑ+h)² − (Rₑ cos E)²) − Rₑ sin E, with Rₑ = 6,378 km.

| Elevation | 5° | 10° | 20° | 30° | 45° | 90° |
|-----------|----|-----|-----|-----|-----|-----|
| Slant range, 500 km | 2,078 km | 1,695 km | 1,193 km | 910 km | 683 km | 500 km |
| Slant range, 525 km | 2,143 km | 1,756 km | 1,244 km | 951 km | 716 km | 525 km |

### 2.2 Frequency Allocations

| Link | Frequency used here | Band | Basis |
|------|---------------------|------|-------|
| Downlink (beacon and transponder) | 145.9 MHz | 145.800–146.000 MHz, not 145.825 MHz | BASELINE §7; IARU to coordinate the actual frequency |
| Uplink (transponder and telecommand) | 435.5 MHz | 435–438 MHz, away from 3 × f_downlink | BASELINE §7 |

Path loss changes by less than 0.1 dB across either band, so the margins hold for any coordinated frequency in it.

### 2.3 Modulation Parameters

| Parameter | Telemetry beacon and telecommand | FM transponder |
|-----------|----------------------------------|----------------|
| Emission | 10K4F2D: 1200 bps AFSK (Bell 202 tones) on FM, ±3 kHz deviation | 16K0F3E: FM voice, ±5 kHz deviation |
| Receiver noise bandwidth B | 12 kHz (40.8 dB-Hz) | 16 kHz (42.0 dB-Hz) |
| Required C/N in B | 12 dB | 12 dB |
| Required C/N₀ | 52.8 dB-Hz | 54.0 dB-Hz |
| Equivalent Eb/N₀ at 1200 bps | 22.0 dB | — |

Emissions and deviations are from BASELINE §7 and `docs/regulatory/filings/ITU_APPENDIX4_DATA.md` §C. The required C/N
is the conventional FM-receiver threshold of about 10 dB plus a 2 dB allowance for the AFSK demodulator or for voice
quality **[assumption]**. It replaces Version 1.0's "Eb/N₀ = 10 dB", which is a coherent-modem figure and does not apply
to AFSK through an FM discriminator. It must be confirmed by a packet-error-rate test of the flight radio against a
reference ground demodulator (§7).

### 2.4 Spacecraft Antenna Pattern Assumption

The spacecraft has separate deployable quarter-wave monopoles for 2 m (≈ 0.5 m) and 70 cm (≈ 0.17 m) (BASELINE §7).
The attitude is not controlled toward the ground station (tumbling, or coarse sun-pointing; BASELINE §8), so the gain
toward a station is a random draw from the pattern, not a fixed number. The model used here:

- **Pattern shape:** half-wave dipole, G(θ) ∝ [cos(π/2·cos θ)/sin θ]², with θ measured from the element axis. A
  quarter-wave monopole on a 1U body behaves roughly like an asymmetric dipole because the body (0.1 m) is small
  against the wavelength (2.06 m at 145.9 MHz) **[assumption; to be replaced by a measured or simulated pattern, §7]**.
- **Peak gain:** 0 dBi, as filed in `docs/regulatory/filings/ITU_APPENDIX4_DATA.md` §C. This is 2.2 dB below the ideal
  dipole directivity (2.15 dBi) and stands for mismatch and body losses.
- **Coverage:** integrating the pattern over the sphere gives the fraction of random orientations in which the gain
  toward the station is at least a given value:

| Gain toward station at least | 0 dBi (peak) | −3 dBi | −6 dBi | −10 dBi | −15 dBi | −20 dBi |
|------------------------------|--------------|--------|--------|---------|---------|---------|
| Fraction of orientations | ≈ 0 % | **63 %** | **81 %** | **92 %** | 97.5 % | 99.2 % |

Design values: **−3 dBi** (nominal; exceeded in 63 % of orientations) and **−6 dBi** (conservative; 81 %) for the
downlinks; **−10 dBi** (92 %) is also shown for telecommand, which must work in most orientations. In the remaining
orientations the element axis points near the station and the signal fades for a few seconds as the spacecraft turns.
At 5° elevation the station is about 67° off nadir, so no single pattern point can be assumed from geometry alone.

### 2.5 Ground Stations

Two reference stations are taken from the typical earth-station data in `docs/regulatory/filings/ITU_APPENDIX4_DATA.md` §E:

| Parameter | Station A: telecommand / well-equipped amateur | Station B: portable amateur |
|-----------|-----------------------------------------------|-----------------------------|
| Antenna | Cross-Yagi, circular, 12 dBic (§E: 12–14 dBi) | Handheld 2-element Yagi, linear, 5 dBi (§E: 5–12 dBi) |
| Receive system noise temperature at 145.9 MHz (at the antenna terminal, including feed and LNA) | 500 K | 1,000 K (§E: 500–1,000 K) |
| Transmit power at 435.5 MHz | 25 W (§E: 25–100 W) | 5 W (§E: 5–50 W) |
| Transmit line loss at 435.5 MHz | 1.5 dB **[assumption]** | 0.5 dB **[assumption]** |
| Polarization loss against the linear spacecraft antenna | 3.0 dB (linear to circular; constant, independent of Faraday rotation) | 3.0 dB average allowance (linear to linear; deep fades as the spacecraft turns and the polarization rotates) |

Station A uses 500 K rather than the "~300 K with LNA" in §E. At 146 MHz the sky itself is several hundred kelvin: the
ITU-R P.372 median galactic-noise expression (F = 52 − 23·log f MHz), extrapolated from its stated range (up to about
100 MHz) to 146 MHz, gives about 480 K, and a Yagi aimed at 5° elevation also sees the ground. Man-made noise raises the
temperature further at rural and residential sites (P.372). Sensitivity of the 5° beacon margin to this choice is in §3.5.

### 2.6 Propagation Losses

| Loss | Value | Basis |
|------|-------|-------|
| Free-space path loss | 32.45 + 20·log f(MHz) + 20·log d(km) | Standard formula |
| Atmospheric and ionospheric absorption, scintillation allowance | 1.0 dB below 20° elevation; 0.5 dB at 20° and above | **[assumption]** |

| Free-space path loss | 5° | 10° | 20° | 30° | 45° | 90° |
|----------------------|----|-----|-----|-----|-----|-----|
| 145.9 MHz, 500 km | 142.1 dB | 140.3 dB | 137.3 dB | 134.9 dB | 132.4 dB | 129.7 dB |
| 435.5 MHz, 500 km | 151.6 dB | 149.8 dB | 146.8 dB | 144.4 dB | 141.9 dB | 139.2 dB |

At 525 km the 5° path loss is 0.27 dB higher (142.35 dB at 145.9 MHz).

## 3. Downlink Budget (145.9 MHz)

### 3.1 Transmitter Parameters

| Parameter | Value | Basis |
|-----------|-------|-------|
| TX power, nominal | 0.5 W (+27.0 dBm) | BASELINE §7 |
| TX power, maximum | 1.0 W (+30.0 dBm) | BASELINE §7 |
| Filter, cable and connector loss | 1.0 dB | **[assumption; measure on the EM]** |
| Antenna gain toward station | −3 dBi nominal, −6 dBi conservative | §2.4 |
| **EIRP at 0.5 W** | **+23.0 dBm** (−3 dBi) / **+20.0 dBm** (−6 dBi) | 27.0 − 1.0 + G |

### 3.2 Path Loss

From §2.6: 142.1 dB free-space loss plus 1.0 dB absorption at 5° elevation; 3.0 dB polarization loss (§2.5).

### 3.3 Ground Station Parameters

| Parameter | Station A | Station B |
|-----------|-----------|-----------|
| Antenna gain | 12 dBic | 5 dBi |
| System noise temperature | 500 K (27.0 dB-K) | 1,000 K (30.0 dB-K) |
| Noise density N₀ = −198.6 + 10·log T | **−171.6 dBm/Hz** | **−168.6 dBm/Hz** |
| Noise in 12 kHz | −130.8 dBm | −127.8 dBm |

### 3.4 Downlink Margin Calculation

#### Beacon, 0.5 W, 5° elevation, Station A, nominal pattern (−3 dBi)
```
EIRP                          +23.0 dBm   (27.0 − 1.0 − 3.0)
Free-space loss (2,078 km)   −142.1 dB
Absorption (5°)                −1.0 dB
Polarization (lin → circ)      −3.0 dB
Ground antenna                +12.0 dBic
──────────────────────────────────────
Received power               −111.1 dBm
Noise density N₀             −171.6 dBm/Hz  (−198.6 + 10·log 500)
──────────────────────────────────────
C/N₀                           60.5 dB-Hz   (C/N = 19.7 dB in 12 kHz)
Required C/N₀                  52.8 dB-Hz   (12 dB in 12 kHz)
──────────────────────────────────────
Margin                         +7.7 dB
```

#### Beacon, 0.5 W, 5° elevation, Station A, conservative pattern (−6 dBi)
```
Received power               −114.1 dBm
C/N₀                           57.5 dB-Hz
Margin                         +4.7 dB
```

#### Beacon, 0.5 W, 5° elevation, at the 525 km cap (−3 dBi)
```
Free-space loss (2,143 km)   −142.4 dB
Margin                         +7.5 dB
```

### 3.5 Downlink Summary

**Beacon (10K4F2D), Station A, 500 km — margin in dB:**

| Elevation | 0.5 W, 0 dBi | 0.5 W, −3 dBi | 0.5 W, −6 dBi | 0.5 W, −10 dBi | 1 W, −3 dBi | 1 W, −6 dBi |
|-----------|--------------|---------------|---------------|----------------|-------------|-------------|
| 5° | +10.7 | **+7.7** | **+4.7** | +0.7 | +10.7 | +7.7 |
| 10° | +12.5 | +9.5 | +6.5 | +2.5 | +12.5 | +9.5 |
| 20° | +16.0 | +13.0 | +10.0 | +6.0 | +16.1 | +13.1 |
| 30° | +18.4 | +15.4 | +12.4 | +8.4 | +18.4 | +15.4 |
| 45° | +20.9 | +17.9 | +14.9 | +10.9 | +20.9 | +17.9 |
| 90° | +23.6 | +20.6 | +17.6 | +13.6 | +23.6 | +20.6 |

**Beacon (10K4F2D), Station B (portable), 500 km — margin in dB:**

| Elevation | 0.5 W, −3 dBi | 0.5 W, −6 dBi | 1 W, −3 dBi | 1 W, −6 dBi |
|-----------|---------------|---------------|-------------|-------------|
| 5° | −2.3 | −5.3 | +0.7 | −2.3 |
| 10° | −0.5 | −3.5 | +2.5 | −0.5 |
| 20° | +3.0 | 0.0 | +6.0 | +3.0 |
| 30° | +5.4 | +2.4 | +8.4 | +5.4 |
| 45° | +7.9 | +4.9 | +10.9 | +7.9 |
| 90° | +10.6 | +7.6 | +13.6 | +10.6 |

**Sensitivity of the 5° beacon margin (Station A, 0.5 W, −3 dBi) to the ground noise temperature:**

| System noise temperature | 300 K (ITU §E value, cold sky) | 500 K (this budget) | 1,000 K (ITU §E upper value) | 1,540 K (ITU-R P.372 rural man-made noise median: F = 67.2 − 27.7·log f = 7.3 dB at 146 MHz, T = 290 × 10^(F/10)) |
|--------------------------|--------------------------------|---------------------|------------------------------|-------------------------------------------------------------|
| Margin | +9.9 dB | +7.7 dB | +4.7 dB | +2.8 dB |

**Transponder downlink (16K0F3E), 500 km — margin in dB (preliminary; the transponder RF design is open, TB-04):**

| Elevation | Station A, 0.5 W, −3 dBi | Station A, 0.5 W, −6 dBi | Station B, 0.5 W, −3 dBi | Station B, 0.5 W, −6 dBi | Station B, 1 W, −3 dBi |
|-----------|--------------------------|--------------------------|--------------------------|--------------------------|------------------------|
| 5° | +6.5 | +3.5 | −3.5 | −6.5 | −0.5 |
| 10° | +8.2 | +5.2 | −1.8 | −4.8 | +1.2 |
| 20° | +11.8 | +8.8 | +1.8 | −1.2 | +4.8 |
| 30° | +14.2 | +11.2 | +4.1 | +1.1 | +7.2 |
| 45° | +16.6 | +13.6 | +6.6 | +3.6 | +9.6 |
| 90° | +19.3 | +16.3 | +9.3 | +6.3 | +12.3 |

**Result:** the 1200 bps beacon closes at 0.5 W from 5° elevation for Station A with **+7.7 dB** (nominal pattern, 63 %
of orientations) and **+4.7 dB** (conservative pattern, 81 % of orientations), at both 500 km and the 525 km cap
(−0.3 dB). This agrees with the audit estimate of about +6 dB in BASELINE §7, which lies between the two pattern
cases. A portable station (B) decodes the beacon from about 12° elevation at 0.5 W in the nominal pattern case, and from
5° at 1 W.

## 4. Uplink Budget (435.5 MHz)

### 4.1 Ground Transmitter Parameters

| Parameter | Station A (telecommand) | Station B (portable transponder user) |
|-----------|-------------------------|---------------------------------------|
| TX power | 25 W (+44.0 dBm) | 5 W (+37.0 dBm) |
| Line loss | 1.5 dB | 0.5 dB |
| Antenna gain | 12 dBic | 5 dBi |
| **EIRP** | **+54.5 dBm** | **+41.5 dBm** |

### 4.2 Path Loss (435.5 MHz)

From §2.6: 151.6 dB at 5°, 144.4 dB at 30°, 139.2 dB at 90°; absorption 1.0 dB below 20°, 0.5 dB above; polarization
3.0 dB.

### 4.3 Spacecraft Receiver Parameters

| Parameter | Value | Basis |
|-----------|-------|-------|
| Antenna | Separate 70 cm quarter-wave monopole with RX filter | BASELINE §7 |
| Antenna gain toward station | −3, −6 or −10 dBi (63 %, 81 %, 92 % of orientations) | §2.4 model applied at 70 cm **[assumption]** |
| System noise temperature, Earth-facing | 1,000 K (30.0 dB-K) | `docs/regulatory/filings/ITU_APPENDIX4_DATA.md` §D; confirm with the radio datasheet |
| Noise density N₀ | −168.6 dBm/Hz | −198.6 + 30.0 |
| Noise in 12 kHz | −127.8 dBm | |

### 4.4 Uplink Margin Calculation

#### Telecommand, Station A (25 W), 5° elevation, spacecraft antenna −6 dBi
```
EIRP                          +54.5 dBm   (44.0 − 1.5 + 12.0)
Free-space loss (2,078 km)   −151.6 dB
Absorption (5°)                −1.0 dB
Polarization (circ → lin)      −3.0 dB
Spacecraft antenna             −6.0 dBi
──────────────────────────────────────
Received power               −107.1 dBm
Noise density N₀             −168.6 dBm/Hz
──────────────────────────────────────
C/N₀                           61.5 dB-Hz
Required C/N₀                  52.8 dB-Hz
──────────────────────────────────────
Margin                         +8.7 dB
```

#### Transponder uplink, Station B (5 W), 30° elevation, spacecraft antenna −3 dBi
```
EIRP                          +41.5 dBm
Free-space loss (910 km)     −144.4 dB
Absorption + polarization      −3.5 dB
Spacecraft antenna             −3.0 dBi
──────────────────────────────────────
Received power               −109.4 dBm
C/N₀                           59.2 dB-Hz
Required C/N₀ (16 kHz)         54.0 dB-Hz
──────────────────────────────────────
Margin                         +5.1 dB
```

### 4.5 Uplink Summary

**Telecommand (10K4F2D), Station A, 25 W, 500 km — margin in dB:**

| Elevation | 0 dBi | −3 dBi | −6 dBi | −10 dBi |
|-----------|-------|--------|--------|---------|
| 5° | +14.7 | +11.7 | +8.7 | **+4.7** |
| 10° | +16.5 | +13.5 | +10.5 | +6.5 |
| 20° | +20.0 | +17.0 | +14.0 | +10.0 |
| 30° | +22.4 | +19.4 | +16.4 | +12.4 |
| 45° | +24.9 | +21.9 | +18.9 | +14.9 |
| 90° | +27.6 | +24.6 | +21.6 | +17.6 |

**Transponder uplink (16K0F3E), Station B, 5 W, 500 km — margin in dB (preliminary, TB-04):**

| Elevation | 0 dBi | −3 dBi | −6 dBi |
|-----------|-------|--------|--------|
| 5° | +0.5 | −2.5 | −5.5 |
| 10° | +2.2 | −0.8 | −3.8 |
| 20° | +5.8 | +2.8 | −0.2 |
| 30° | +8.1 | +5.1 | +2.1 |
| 45° | +10.6 | +7.6 | +4.6 |
| 90° | +13.3 | +10.3 | +7.3 |

**Result:** telecommand from Station A at 25 W closes from 5° elevation with **+4.7 dB in 92 % of orientations**
(+8.7 dB in 81 %). These margins assume a quiet band. Not included: terrestrial users of 420–450 MHz (the amateur service
is secondary there in the United States), which raise the effective receiver noise over populated areas, and the
spacecraft's own 2 m third harmonic (3 × 145.9 = 437.7 MHz), which BASELINE §7 controls by placing the uplink away from
3 × f_downlink and by a ≥ 60 dB suppression target.

## 5. Link Budget Summary

### 5.1 Operating Envelope

Margins at 500 km, nominal pattern (−3 dBi) unless stated:

```
                     Beacon 0.5 W          Telecommand 25 W       Transponder (preliminary)
Elevation   Station A   Station B    Station A (−10 dBi)   Down A    Down B    Up B
  5°         +7.7        −2.3          +4.7                +6.5      −3.5      −2.5
 10°         +9.5        −0.5          +6.5                +8.2      −1.8      −0.8
 20°        +13.0        +3.0         +10.0               +11.8      +1.8      +2.8
 30°        +15.4        +5.4         +12.4               +14.2      +4.1      +5.1
 90°        +20.6       +10.6         +17.6               +19.3      +9.3     +10.3
```

### 5.2 Operational Constraints

| Constraint | Value | Basis |
|------------|-------|-------|
| Minimum elevation, telemetry (Station A) | 5° | §3.5: +7.7 dB nominal, +4.7 dB conservative |
| Minimum elevation, telecommand (Station A, 25 W) | 5° | §4.5: +4.7 dB in 92 % of orientations |
| Beacon TX power | 0.5 W nominal; 1 W extends portable-station coverage to 5° | §3.5 |
| Transponder, portable stations | Usable from about 20° elevation | §3.5, §4.5 (both directions ≥ +1.8 dB at 20°, nominal pattern) |
| Margin requirement | > 6 dB threshold, > 10 dB goal (downlink TPM, `docs/project/SEMP.md` §3.3) | Met for Station A from 5° in the nominal pattern case (+7.7 dB); the 10 dB goal is met from about 12°; the conservative case meets 6 dB from about 9° |

Version 1.0's "2 W recommended TX power" and "20° minimum elevation" are withdrawn (§1.1).

### 5.3 Doppler Effects

| Parameter | Value | Basis |
|-----------|-------|-------|
| Maximum range rate | 7.06 km/s | Orbital speed × sine of the nadir angle at the horizon (68.0° at 500 km); Earth rotation neglected |
| Maximum Doppler shift, 145.9 MHz | ±3.4 kHz | f × v/c |
| Maximum Doppler shift, 435.5 MHz | ±10.3 kHz | f × v/c |

**Mitigation**: ground-station Doppler correction (for example Gpredict with radio CAT control). The VHF shift is a
third of the 10.4 kHz necessary bandwidth, so stations without correction should tune during the pass.

## 6. Link Enhancement Options

### 6.1 If Margin Is Insufficient

| Option | Effect | Implementation |
|--------|--------|----------------|
| Transmit at 1 W | +3.0 dB downlink | Within the BASELINE §7 maximum; costs DC power |
| Larger ground antenna | +2 dB (12 → 14 dBic) | §E upper range |
| Lower ground noise | Up to +2.2 dB (500 → 300 K) | Quiet site, mast-mounted LNA |
| Software demodulator with better AFSK performance | To be measured | Reduces the 2 dB implementation allowance |
| Lower data rate | +3 dB per halving | Changes the beacon format and the filed emission |

### 6.2 Network Enhancement

More stations in the amateur community increase the number of passes heard per day; this does not change the margins
above.

## 7. Verification

### 7.1 Pre-Flight Verification

| Item to replace an assumption | Method | Status |
|-------------------------------|--------|--------|
| TX power and filter/cable loss (§3.1) | RF power meter on the EM | [ ] |
| Antenna pattern on the 1U body, both bands (§2.4) | Electromagnetic simulation, then range or anechoic measurement of the EM | [ ] |
| Required C/N for the beacon and telecommand (§2.3) | Packet-error rate versus input level, flight radio against a reference ground demodulator | [ ] |
| Spacecraft receiver noise temperature (§4.3) | Noise-figure measurement of the EM receiver chain | [ ] |
| Uplink in-band interference | Literature and on-orbit data for 435–438 MHz receivers; add an allowance | [ ] |

### 7.2 Post-Launch Verification

| Test | Method | Status |
|------|--------|--------|
| Signal acquisition | First pass after the 45-minute RF silence | [ ] |
| Link margin measurement | Received C/N₀ versus elevation at Station A | [ ] |
| Doppler verification | Frequency tracking | [ ] |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-16 | Funding audit (TB-05) | Re-issued at the baseline orbit, frequencies and power: noise floor corrected (§1.1); antenna pattern and ground stations stated; telecommand and transponder links added |
