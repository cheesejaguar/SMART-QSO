# SMART-QSO Link Budget Analysis

**Document ID**: SQSO-LBA-001
**Version**: 3.2
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Controlled link budget for the baseline in [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) (§3, §7, §8). CDR re-issue (finding TB-05). **Analysis, not measured.** Every other document that quotes a link margin cites this one.

## 1. Purpose and Summary

This document gives the RF link budgets of the baseline: the 1200 bps AX.25 beacon and the FM transponder on the 2 m
downlink, the FM transponder and authenticated telecommand on the 70 cm uplink. Version 3.0 is the CDR re-issue that
BASELINE §7 asks for. Against Version 2.0 it adds:

- the demodulator thresholds from a baseband simulation of the actual waveforms, replacing the assumed "12 dB C/N"
  (§3);
- the antenna pattern sampled along real passes, with the spacecraft spinning at 8°/s about +Y on the Sun (BASELINE
  Rev E §8, D9; 6°/s, the earlier value, for comparison), for three element orientations, and the resulting fading,
  frame loss and pass statistics (§5);
- handheld and mobile user stations, the SINAD a transponder user hears, and the end-to-end transponder link (§4.3,
  §6.3);
- Doppler effects on AFSK decoding, CTCSS access and telecommand (§4.4);
- receiver desense from the spacecraft's own transmitter, which full-duplex operation makes part of the uplink budget
  (§7).

Every number is reproduced by the scripts in `software/simulation/cdr/rf/` (§9).

### 1.1 Results against the requirements

| Requirement (source) | Case | Result | Verdict |
|----------------------|------|--------|---------|
| Beacon downlink margin > 6 dB threshold, > 10 dB goal (`docs/project/SEMP.md` TPM) | 0.5 W, 5° elevation, reference station A (12 dBic, 500 K), median spacecraft gain toward the station (−3 dBi) | **+7.9 dB** (+10.9 dB at 1 W); goal met from 11° | Met (threshold) |
| Same, 10th-percentile spacecraft gain (−10 dBi) | 0.5 W, 5° | **+0.9 dB** (+3.9 dB at 1 W; +6.3 dB at 20°) | Closes, below threshold |
| Beacon at the 485 km altitude cap (the bound) | 0.5 W, 5°, −3 / −10 dBi | +7.8 / +0.8 dB | Met / closes |
| Beacon to a portable station (5 dBi Yagi, no Doppler correction) | 0.5 W, −3 dBi | −4.2 dB at 5°, +1.2 dB at 20°, +6.1 dB at 45° | Usable from about 17°, at 1 W from about 10° |
| Beacon to a SatNOGS omni station (0 dBic, 1,000 K) | 0.5 W, −3 dBi | −7.1 dB at 5°, +0.7 dB at 30°, +5.9 dB at zenith | Usable above about 28° only |
| Telecommand uplink margin ≥ 6 dB (proposed here, §1.2) at 10° | 25 W, 14 dBic, −10 dBi spacecraft gain, +6 dB terrestrial interference | **+2.6 dB** at 10°, +6.2 dB at 20°; +8.6 dB at 10° without interference. Receiver as designed (§6.4): +0.8 dB at 10° (25 W), **+3.8 dB at 50 W PEP** | **Not met at 10°**; met at 20°, or from about 11° with 50 W (§6.1); with the designed receiver from about 16° at 50 W |
| Telecommand, median gain, interference +6 dB | 25 W, 5° | +7.8 dB | Met |
| Transponder usable (12 dB SINAD end to end) | 5 W handheld + handheld Yagi up, handheld Yagi down | 20 dB at 5°, 27 dB at 20° | Met from 5° |
| Transponder usable for a handheld with a whip, both ways | 5 W, whip, −3 dBi | 1.8 dB at 5°, 14.9 dB at 20°, 26 dB at zenith | Usable from about 17° (12 dB SINAD) |
| Beacon pass decoded, 8°/s spin, element across the spin axis | station A, 0.5 W, 60 days of passes at 475 km | 99.6 % of passes, 98.2 % of frames with the ±5 s beacon-interval jitter as built (96.6 % without) | Met |

**Verdict.** The 2 m beacon closes with the required margin to a reference station from 5° at 0.5 W. The telecommand
uplink at 25 W does not hold a 6 dB margin at 10° once a 6 dB terrestrial-interference allowance and the antenna's
10th-percentile gain are both taken; with the budget receiver (1,000 K, the simulated threshold) telecommand at 50 W
(the 50 W PEP limit near Beale AFB applies at the Bay Area site, §6.1), or commanding above 20°, restores it. With the
receiver as designed on the RF board (SQSO-RF-BRD-001 Version C, §6.4: lower noise, but 2 dB more demodulator loss
allowed) 50 W gives +3.8 dB at 10° and meets 6 dB from about 16°. Handheld users with whips are transponder users above about 17°
only. The largest open item is not the budget itself but its inputs that need hardware: the antenna pattern on the
1U body with wings, the receiver noise figure, and the transmitter's 3rd harmonic and phase noise (§8).

### 1.2 Requirements this budget is held against

| ID | Requirement | Source |
|----|-------------|--------|
| LB-1 | Beacon: margin ≥ +6 dB (threshold), ≥ +10 dB (goal) at 5° elevation to the reference station, median spacecraft gain | SEMP TPM "link margin (downlink)" |
| LB-2 | Beacon: margin ≥ 0 dB at 5° with the 10th-percentile spacecraft gain (so ≥ 90 % of beacons close at a reference station) | This document (proposed) |
| LB-3 | Telecommand: margin ≥ +6 dB at 10° elevation, 10th-percentile spacecraft gain, +6 dB terrestrial-interference allowance | This document (proposed; IARU form 7.6 asks for the telecommand link budget) |
| LB-4 | Transponder: ≥ 12 dB SINAD end to end (the TIA-603 reference-sensitivity level) for a handheld-Yagi user at both ends from 10°; for a handheld with a whip from 30° | This document (proposed; MRD-031) |

LB-2 to LB-4 are proposed for the requirements baseline (for the chair; `docs/requirements/` is not owned by this
document).

### 1.3 Correction history

Version 1.0 computed the ground noise density as −174 + 10·log 350 dBm/Hz; −174 dBm/Hz already is kT at 290 K, so the
temperature was counted twice and every margin was about 25 dB too pessimistic (Version 2.0 §1.1). Version 2.0 fixed
that, used an assumed 12 dB C/N demodulator threshold and a random-orientation pattern. Version 3.0 replaces both
with the analyses in §3 and §5. The Version 2.0 headline margins change by less than 0.2 dB because the simulated AFSK
threshold (52.9 dB-Hz with implementation loss) is within 0.1 dB of the assumed one (52.8 dB-Hz).

## 2. Parameters

### 2.1 Orbit and geometry

| Parameter | Value | Basis |
|-----------|-------|-------|
| Altitude | **475 km** (the BASELINE Rev E §3 target; band 465–485 km). The 485 km cap is the bound: 0.1 dB less at 5° | BASELINE §3 |
| Orbit | Circular SSO, i ≈ 97.4°, LTAN 22:30 (descending node 10:30) | BASELINE §3; SIL default |
| Period, speed | 94.1 min, 7.63 km/s | 2π√(a³/μ), √(μ/a) |
| Slant range | d = √((Rₑ+h)² − (Rₑ cos E)²) − Rₑ sin E | Rₑ = 6,378 km |

| Elevation | 5° | 10° | 20° | 30° | 45° | 90° |
|-----------|----|-----|-----|-----|-----|-----|
| Slant range, 475 km | 2,012 km | 1,633 km | 1,142 km | 867 km | 650 km | 475 km |
| Slant range, 485 km (cap) | 2,039 km | 1,658 km | 1,162 km | 884 km | 663 km | 485 km |
| Free-space loss, 145.9 MHz, 475 km | 141.8 dB | 140.0 dB | 136.9 dB | 134.5 dB | 132.0 dB | 129.3 dB |
| Free-space loss, 435.5 MHz, 475 km | 151.3 dB | 149.5 dB | 146.4 dB | 144.0 dB | 141.5 dB | 138.8 dB |
| Free-space loss, 145.9 MHz, 485 km | 141.9 dB | 140.1 dB | 137.0 dB | 134.7 dB | 132.2 dB | 129.4 dB |

### 2.2 Frequencies and emissions

| Link | Planning frequency | Band | Emission | Deviation |
|------|--------------------|------|----------|-----------|
| Downlink: beacon | 145.9 MHz | 145.800–146.000 MHz, not 145.825 MHz (BASELINE §7) | 10K4F2D: AFSK Bell 202 on FM | ±3.0 kHz peak (Bn = 2·2.2 + 2·3.0 = 10.4 kHz) |
| Downlink: transponder | same transmitter | same | 16K0F3E | ±5.0 kHz peak, 3 kHz audio |
| Uplink: transponder | 435.5 MHz | requested in 435.222–435.778 MHz, or else 436.222–436.500 MHz (BASELINE Rev E §7), inside 435.000–437.300 MHz, the part of 435–438 MHz clear of 3 × f_down for any downlink in the segment (§7.1) | 16K0F3E + CTCSS 67.0 Hz | ±5.0 kHz |
| Uplink: telecommand | same receiver | same | 10K4F2D | ±3.0 kHz |

The free-space loss changes by less than 0.1 dB across either band, so the margins hold for any coordinated frequency.
The ±3.0 kHz beacon deviation is the filed value; `hardware/rf/TRANSMITTER_DESIGN.md` §5.2 said ±3.5 kHz, which would
make the emission 11K4F2D, and is corrected there.

### 2.3 Propagation

| Effect | Allowance | Basis |
|--------|-----------|-------|
| Free-space loss | 32.45 + 20 log f(MHz) + 20 log d(km) | |
| Gaseous absorption | Included in the allowance below | ITU-R P.676: ≈ 0.04 dB at zenith at VHF/UHF, ≈ 0.5 dB at 5° |
| Ionospheric absorption | Included below | ITU-R P.531: < 0.1 dB at 146 MHz at mid latitude outside polar-cap absorption events |
| Scintillation | Included below (mid latitude) | ITU-R P.531: about 1 dB peak-to-peak at VHF at mid latitude; **equatorial post-sunset and auroral scintillation can exceed 10 dB at 146 MHz** and is not budgeted (a Fairbanks station, `docs/ground/GROUND_STATION_DESIGN.md` §3.3, sees auroral events) |
| **Total allowance** | **1.0 dB below 20° elevation, 0.5 dB at and above 20°** | |
| Faraday rotation | Not a loss to a circular station (constant 3 dB, linear to circular). To a linear ground antenna it randomizes the polarization: 2.36 × 10⁴ · B · TEC / f² ≈ 5.5 rad at 146 MHz for 10 TECU and 50 µT (P.531), 0.6 rad at 435.5 MHz, varying by radians during a pass | §5 models it as a random start angle drifting 0.5–3 rad per pass |
| Polarization mismatch | 3.0 dB fixed in the tables (linear spacecraft element to circular station; and the average for linear to linear). Linear-to-linear fades below the average: the loss exceeds 10 dB 20 % of the time and 16 dB 10 % of the time for a uniformly random angle | §5 computes it per sample |

### 2.4 Spacecraft

| Parameter | Value | Basis |
|-----------|-------|-------|
| Transmitter power | 0.5 W nominal (+27 dBm), 1 W maximum (+30 dBm) | BASELINE §7 |
| Filter, cable and connector loss | 1.0 dB allocated (the analysed LPF of §7.2 has 0.6 dB with inductor Q 80; 0.4 dB for cable and connectors). As designed (SQSO-RF-BRD-001 Version C §4.3) the transmitter sets its power at the board port after the LPF, and port to antenna is **0.44 dB** worst at 146 MHz (0.22 m of 1.37 mm coax, two MMCX pairs, worst-case mismatch; return loss ≥ 13.6 dB): the downlink margins here are conservative by 0.56 dB (1 W setting: 29.1–29.7 dBm at the antenna) | `software/simulation/cdr/rf/desense_emc.py`; `hardware/electrical/rf/rf_calcs.py` §16 |
| Antennas | ISIS AntS 1U–3U on a ±Z face: separate 2 m (≈ 0.5 m) and 70 cm (≈ 0.17 m) tape elements deploying in the plane of the face (BASELINE §7; `docs/hardware/BUILD_VS_BUY.md` §2.6) | |
| Pattern | Half-wave-dipole-shaped about the element axis, **0 dBi peak** (filed), null filled to −20 dBi by the body and wings [assumption until the EM pattern measurement, VI-RF-01] | §5 |
| Gain toward the station, along passes, element across the spin axis | Median −1.9 dBi (−2.2 dBi at 5–10° elevation); 10th percentile −9.4 dBi | §5.2 |
| Design values in the tables | **−3 dBi (median) and −10 dBi (10th percentile)**; 0 dBi (peak) for the uplink upper bound | §5.2 |
| Receiver system noise temperature | **1,000 K** at the antenna port in the tables (Earth at 290 K in view, RX filter 1.5 dB, LNA NF ≤ 2 dB: 650 K; 350 K allowance for in-band interference at a quiet site). This is a requirement on the receiver (`docs/hardware/RF_DESIGN.md` RF-P006). As designed: **548–598 K** at the antenna terminals with 0.62 dB of cable, connectors and mismatch at 436 MHz (was 503–550 K with 0.25 dB), so 898–948 K with the allowance; the design also carries a 2 dB demodulator implementation loss (§6.4) | ITU_APPENDIX4_DATA §D; SQSO-RF-BRD-001 §6.1 |
| Terrestrial interference at 435 MHz | A further **+6 dB** (and +10 dB sensitivity) on the receiver noise over populated areas. 420–450 MHz is a radiolocation band in the United States in which the amateur service is secondary (47 CFR 97.303(b)); 435–438 MHz is a satellite-only sub-band by IARU band plan but is used terrestrially in places [assumption: on-orbit measurements by earlier missions show 5–15 dB rises; to be measured on orbit, VI-RF-09] | |

### 2.5 Ground stations

**Downlink (receive at 145.9 MHz).** The noise temperature is at the antenna terminal, including sky and ground noise
and the receiver. ITU-R P.372-16 at 145.9 MHz gives a galactic median of F_a = 52 − 23 log f = 2.2 dB (480 K) and
man-made noise medians F_am = c − d log f of 7.3 dB (1,540 K) rural, 12.6 dB (5,220 K) residential and 16.9 dB
(14,100 K) city, for a short vertical monopole. A handheld radio's 12 dB SINAD sensitivity of 0.18 µV (−121.9 dBm),
with the 47.4 dB-Hz threshold of §3.2, implies 560 K of receiver noise.

| Class | Station | Gain | T_sys | Polarization | Doppler corrected |
|-------|---------|------|-------|--------------|-------------------|
| A | Telecommand / well-equipped amateur: 2 m cross-Yagi with mast LNA, az-el tracking | 12 dBic | 500 K (quiet site; sensitivity in §6.1) | Circular | Yes |
| S | SatNOGS station with an omni antenna (turnstile, QFH, eggbeater: 80 % of the 253 2 m-capable SatNOGS stations seen in the 7 days to 2026-10-27, `network.satnogs.org/api/stations`) | 0 dBic | 1,000 K | Circular | Yes |
| B | Portable: handheld 3-element 2 m Yagi on an FM handheld radio | 5 dBi | 1,000 K | Linear | No |
| W | Handheld radio with a quarter-wave whip (−2 dBi including the operator's body) | −2 dBi | 1,000 K | Linear | No |
| D | Handheld radio with its stock helical antenna | −8 dBi | 1,000 K | Linear | No |

**Uplink (transmit at 435.5 MHz).**

| Class | Station | Power | Line loss | Gain | Polarization |
|-------|---------|-------|-----------|------|--------------|
| TC | Designated telecommand station: 70 cm cross-Yagi, tracking | 25 W (50 W option, §6.1) | 1.5 dB | 14 dBic | Circular |
| M | Mobile: roof-mounted vertical | 50 W | 1.0 dB | 2 dBi at low elevation (null overhead) | Linear |
| BY | Handheld 5 W with a handheld 7-element 70 cm Yagi | 5 W | 0.3 dB | 8 dBi | Linear |
| HW | Handheld 5 W with its whip | 5 W | 0 dB | −2 dBi with body | Linear |

## 3. Demodulator Thresholds (simulated)

### 3.1 Method

`software/simulation/cdr/rf/fm_demod_sim.py` simulates the waveforms at complex baseband (96 kHz): the audio, an FM
modulator at the filed deviation, a carrier offset for Doppler, complex white noise at a set C/N₀, the receiver's IF
filter (±7.5 kHz, a 25 kHz-channel FM filter; ±6 kHz also run), a limiter-discriminator, audio filtering and the
demodulator.

- **AFSK (beacon and telecommand):** 120,000 random bits a point; mark/space correlators over each bit with ideal bit
  timing; NRZI decoding (one tone error flips two data bits). Frame success = (1 − BER)^bits for a 200-byte information
  field (1,850 bits with header, FCS and 3 % bit stuffing).
- **Voice (transponder):** the TIA-603 SINAD test: 1 kHz tone at ±3 kHz deviation, 750 µs de-emphasis, 300–3000 Hz
  audio; SINAD = total over residual after removing the tone.

The ideal bit timing makes the AFSK result optimistic by a fraction of a dB; a **1.0 dB implementation loss** is added
to every AFSK threshold (clock recovery, filter shape, ±0.5 kHz deviation tolerance).

### 3.2 Results

| Waveform and case | C/N₀ for BER 10⁻⁴ | C/N₀ for 90 % of 200-byte frames | Required C/N₀ (with 1.0 dB) | Equivalent Eb/N₀ at 1200 bps | C/N in the 15 kHz IF |
|-------------------|-------------------|----------------------------------|-----------------------------|------------------------------|----------------------|
| AFSK, Doppler corrected (stations A and S) | 51.6 dB-Hz | 51.9 dB-Hz | **52.9 dB-Hz** | 22.1 dB | 11.1 dB |
| AFSK, 3.5 kHz uncorrected Doppler (B, W, D) | 53.6 dB-Hz | 54.0 dB-Hz | **55.0 dB-Hz** | 24.2 dB | 13.2 dB |
| AFSK, 3.5 kHz offset in a ±6 kHz (12.5 kHz-channel) IF | 55.8 dB-Hz | 56.3 dB-Hz | 57.3 dB-Hz | | |
| Telecommand, residual offset 1 kHz (ground-corrected) | 51.6 dB-Hz | 52.0 dB-Hz | **53.0 dB-Hz** | 22.2 dB | |
| Telecommand, 10 kHz uncorrected Doppler, ±7.5 kHz IF | **No decoding at any C/N₀** (the signal leaves the filter) | | | | |
| Telecommand, 10 kHz uncorrected, ±12.5 kHz IF | BER 2.5 × 10⁻² at 56 dB-Hz | | > 60 dB-Hz | | |
| Telecommand, no offset, ±12.5 kHz IF | 53.9 dB-Hz | 54.6 dB-Hz | 55.6 dB-Hz | | |

| Voice (transponder) | C/N₀ for 12 dB SINAD | C/N₀ for 20 dB SINAD |
|---------------------|----------------------|----------------------|
| No offset | **47.4 dB-Hz** (C/N 5.6 dB in 15 kHz) | 49.7 dB-Hz |
| 3.5 kHz uncorrected offset | 48.2 dB-Hz | 50.5 dB-Hz |

The 12 dB SINAD point at 5.6 dB C/N agrees with FM click-noise theory (Rice; Taub and Schilling) and with handheld
sensitivity specifications (0.18–0.25 µV). AFSK needs about 4.5 dB more than 12 dB SINAD voice: 1200 bps AFSK on FM
needs the FM detector above its threshold. The equivalent Eb/N₀ of 22 dB is why Version 1.0's "Eb/N₀ = 10 dB"
(coherent FSK) did not apply.

**Above threshold** the SINAD grows about 1 dB per dB of C/N₀ (24 dB at 52 dB-Hz, 32 dB at 60 dB-Hz).

## 4. Downlink Budget (145.9 MHz)

### 4.1 Worked example: beacon, 0.5 W, 5°, station A, median gain

```
Transmitter                    +27.0 dBm   (0.5 W)
Filter, cable, connectors       −1.0 dB
Spacecraft antenna              −3.0 dBi   (median toward the station, §5.2)
EIRP                           +23.0 dBm
Free-space loss (2,012 km)    −141.8 dB
Absorption, scintillation       −1.0 dB
Polarization (linear → circ.)   −3.0 dB
Ground antenna                 +12.0 dBic
Received carrier              −110.8 dBm
Noise density (500 K)         −171.6 dBm/Hz  (−198.6 + 10 log 500)
C/N₀                            60.8 dB-Hz
Required (§3.2)                 52.9 dB-Hz
Margin                          +7.9 dB
```

### 4.2 Beacon margin (dB)

| Station | Power, spacecraft gain | 5° | 10° | 20° | 30° | 45° | 90° |
|---------|------------------------|----|-----|-----|-----|-----|-----|
| A | 0.5 W, −3 dBi | **+7.9** | +9.7 | +13.3 | +15.7 | +18.2 | +20.9 |
| A | 0.5 W, −10 dBi | **+0.9** | +2.7 | +6.3 | +8.7 | +11.2 | +13.9 |
| A | 1 W, −3 dBi | +10.9 | +12.7 | +16.3 | +18.7 | +21.2 | +23.9 |
| A | 1 W, −10 dBi | +3.9 | +5.7 | +9.3 | +11.7 | +14.2 | +16.9 |
| S (SatNOGS omni) | 0.5 W, −3 dBi | −7.1 | −5.3 | −1.7 | +0.7 | +3.2 | +5.9 |
| S | 1 W, −3 dBi | −4.1 | −2.3 | +1.3 | +3.7 | +6.2 | +8.9 |
| B (handheld Yagi) | 0.5 W, −3 dBi | −4.2 | −2.4 | +1.2 | +3.6 | +6.1 | +8.9 |
| B | 1 W, −3 dBi | −1.2 | +0.6 | +4.3 | +6.6 | +9.2 | +11.9 |
| W (handheld whip) | 0.5 W, −3 dBi | −11.2 | −9.4 | −5.8 | −3.4 | −0.9 | +1.9 |
| W | 1 W, −3 dBi | −8.2 | −6.4 | −2.7 | −0.4 | +2.2 | +4.9 |
| D (rubber duck) | 1 W, −3 dBi | −14.2 | −12.4 | −8.7 | −6.4 | −3.8 | −1.1 |

**At the 485 km cap** (the bound), station A, 5°: +7.8 dB (−3 dBi), +0.8 dB (−10 dBi).

**Station A noise sensitivity** (0.5 W, −3 dBi, 5°):

| T_sys | 300 K | 500 K | 1,000 K | 1,500 K (P.372 rural man-made) | 2,000 K | 5,800 K (P.372 residential, omni) |
|-------|-------|-------|---------|--------------------------------|---------|-----------------------------------|
| Margin | +10.1 dB | **+7.9 dB** | +4.9 dB | +3.1 dB | +1.8 dB | −2.8 dB |

The telecommand sites must therefore be quiet: a Yagi aimed at 5° in a residential neighbourhood sees man-made noise
from the horizon and can lose the margin. A noise survey of each candidate site (2 m noise floor at 5–20° elevation in
all azimuths) is verification item VI-GND-02 (`docs/ground/GROUND_STATION_DESIGN.md`).

### 4.3 Transponder downlink SINAD a user hears (0.5 W; downlink only)

| Station | Gain | 5° | 20° | Zenith | (10° / 30° / 45°) |
|---------|------|----|-----|--------|-------------------|
| A | −3 dBi | 33 dB | > 36 dB | > 36 dB | 35 / > 36 / > 36 |
| B (handheld Yagi) | −3 dBi | **21 dB** | **28 dB** | **33 dB** | 24 / 30 / 31 |
| B | −10 dBi | 2 dB | 16 dB | 28 dB | 5 / 23 / 26 |
| W (handheld, whip) | −3 dBi | **< 3 dB** | **16 dB** | **28 dB** (whip tilted to the satellite) | 5 / 23 / 26 |
| W | −10 dBi | < 3 dB | < 3 dB | 18 dB | < 3 / 3 / 9 |
| D (rubber duck) | −3 dBi | < 3 dB | < 3 dB | 21 dB | < 3 / 5 / 11 |
| S (SatNOGS omni) | −3 dBi | 7 dB | 23 dB | 31 dB | 13 / 26 / 29 |

A vertical whip has a null overhead: the zenith value assumes the user points the whip at the satellite. 12 dB SINAD
is the TIA-603 reference sensitivity (intelligible); 20 dB is a comfortable contact. A handheld with a whip therefore
hears the transponder from about 17° (median gain), with 5–6 s dropouts every 22.5 s while the element null sweeps past
(§5). A handheld with the stock helical antenna hears it only within about 45° of zenith.

### 4.4 Doppler

| Quantity | 145.9 MHz | 435.5 MHz | Basis |
|----------|-----------|-----------|-------|
| Maximum shift | **±3.49 kHz** | **±10.42 kHz** | Pass simulation, 60 days at 475 km, Santa Clara (range rate up to 7.17 km/s with Earth rotation) |
| Maximum rate (overhead pass at TCA) | 57 Hz/s | 169 Hz/s | same |

Effects:

1. **AFSK beacon:** an uncorrected 3.5 kHz offset costs 2.1 dB (§3.2) in a 15 kHz IF and 4.4 dB in a 12 kHz IF (many
   modern handhelds default to 12.5 kHz "narrow" channels: users should select wide). Stations A and S correct Doppler
   (Gpredict with CAT, SatNOGS).
2. **Telecommand:** an uncorrected 10 kHz uplink offset takes the signal out of a ±7.5 kHz receiver filter: **nothing
   decodes** (§3.2). Telecommand stations shall pre-correct the uplink to within ±1 kHz of the spacecraft receiver
   frequency (requirement GND-R5, `docs/ground/GROUND_STATION_DESIGN.md`). The spacecraft receiver bandwidth stays at
   ±7.5 kHz: widening it to ±12.5 kHz to tolerate uncorrected users would cost 2.6 dB on every uplink (§3.2) and still
   would not decode a 10 kHz offset.
3. **Transponder users** step the uplink in 5 kHz increments (435.510 → 435.505 → 435.500 → 435.495 → 435.490 MHz for a
   435.500 MHz receiver, as on SO-50 and AO-91); a residual of ±2.5 kHz costs about 1 dB.
4. **CTCSS access:** Doppler shifts the carrier, not the audio, so the 67.0 Hz tone is unchanged (its relative change is
   v/c = 2.4 × 10⁻⁵). The discriminator's DC term from the offset (up to 10.4 kHz, changing at ≤ 169 Hz/s) is removed by
   the decoder's high-pass. The tone detector's own threshold is far below the voice threshold: above the FM threshold
   the discriminator noise density near 67 Hz is (N₀/C)·f², giving a tone SNR above 40 dB in a 10 Hz detector at
   47 dB-Hz; below threshold, click noise (rate ≈ (B/2√3)·erfc√ρ) still leaves about 10 dB SNR at a C/N of 0 dB.
   CTCSS access therefore never limits the link, provided the uplink is inside the receiver filter. Verification:
   VI-RF-05.

## 5. Spin, Antenna Pattern and Fading

### 5.1 Method

`software/simulation/cdr/rf/pass_geometry_mc.py` propagates the baseline orbit (475 km, LTAN 22:30, from 2028-11-01)
for 60 days (1 s steps), with +Y on the Sun and the spacecraft spinning about +Y at 8°/s (BASELINE Rev E §8, D9) or
6°/s (the value before the design review, for comparison), and evaluates the gain toward the Santa Clara station every
second of every pass above 5° (226 passes). Element orientations (the AntS
deploys its tapes in the plane of a ±Z face, so each tape lies along ±X or ±Y):

| Option | Element | Effect of the spin |
|--------|---------|--------------------|
| **perp** | Along X, across the spin axis (the SIL's −Z assumption is equivalent) | The pattern null sweeps the sky twice per revolution |
| **para** | Along Y, the spin axis | No modulation; the null points at the Sun and away from it |
| **cross** | Two orthogonal elements (X and Y) fed in quadrature through a 90° hybrid | Nulls filled |

Each 1.2 s beacon frame (110-byte information field plus 300 ms TXDELAY) is evaluated at three points across its time
on air against the simulated BER curve (§3). Linear ground antennas get the geometric polarization mismatch and a
Faraday rotation drifting 0.5–3 rad per pass.

### 5.2 Gain toward the station along passes

| Option | Median gain, 5–10° elevation | 10th percentile, 5–10° | Time below −6 dBi | Time below −10 dBi | Fades below −10 dBi, 8°/s | Same, 6°/s |
|--------|------------------------------|------------------------|-------------------|--------------------|---------------------------|------------|
| perp | −2.2 dBi | −9.2 dBi | 21 % | 8.7 % | 8.3 per pass, median 5 s, 95th percentile 6 s | 6.3 per pass, median 6 s, 95th percentile 8 s |
| para | −1.4 dBi | −6.3 dBi (−12.8 dBi at 30–45°) | 17 % | 4.4 % | 0.23 per pass, median 78 s, 95th percentile 129 s | same (no spin effect) |
| cross | −2.4 dBi | −3.6 dBi | **0 %** | **0 %** | none | none |

The −3 dBi and −10 dBi design values of §4 and §6 are the median and 10th percentile of the perp case. Version 2.0's
random-orientation coverage (−3 dBi in 63 % of orientations, −10 dBi in 92 %) is close to the pass-weighted values.

The spin does not change the fraction of time in a fade (8.7 % at 8°/s and at 6°/s, perp); it changes the fade length
and how often fades recur: 8°/s gives 5 s fades every 22.5 s (6°/s gave 6 s every 30 s), which a voice user hears as
regular dropouts.

### 5.3 Frame loss and passes heard (0.5 W, Santa Clara, 60 days, 475 km)

| Station | Option | Spin | Beacon interval | Passes with a beacon decoded | Frames decoded | Transponder time above 12 / 20 dB SINAD |
|---------|--------|------|-----------------|------------------------------|----------------|------------------------------------------|
| A | perp | 8°/s | 60 s | 99.6 % | **96.6 %** | 99 / 98 % |
| A | perp | 8°/s | 67 s | 99.6 % | 97.5 % | 99 / 98 % |
| A | perp | 8°/s | 60 ± 5 s jitter | 99.6 % | **98.2 %** | 99 / 98 % |
| A | perp | 6°/s (former) | 60 s | 99.2 % | 96.3 % | 99 / 98 % |
| A | perp | 8°/s | 120 s (IDLE) | 98.2 % | 96.6 % | |
| A | para or cross | 8°/s | 60 s | 99.6 % / 99.1 % | 100.0 % | 100 / 100 % |
| B (handheld Yagi) | perp | 8°/s | 60 s | 92.8 % | 48.8 % | 69 / 59 % |
| B | perp | 6°/s (former) | 60 s | 85.7 % | 54.7 % | 71 / 60 % |
| B | para | 8°/s | 60 s | 84.4 % | 57.7 % | 75 / 66 % |
| B | cross | 8°/s | 60 s | 97.5 % | 71.5 % | 92 / 83 % |
| S (SatNOGS omni) | perp | 8°/s | 60 s | 71.7 % | 34.9 % | 63 / 34 % |
| S | cross | 8°/s | 60 s | 67.5 % | 36.9 % | 70 / 36 % |
| W (handheld whip) | perp | 8°/s | 60 s | 47.6 % | 12.9 % | 39 / 21 % |

Each row draws its own spin phase, beacon schedule and (for linear stations) Faraday rotation over the same 226 passes,
so the weak-station figures carry a sampling scatter of several points (passes heard by B range over 78–98 % between
otherwise similar rows); the station-A frame fractions are steady to about ±0.5 point. A pass with no beacon scheduled
inside it counts as not heard, which is why station A stays just below 100 % of passes even with every frame decoded.

Findings:

1. **At 8°/s the 60 s beacon interval no longer locks to the spin.** The spin period is 45 s and the pattern null
   recurs every 22.5 s; a 60 s interval advances the pattern by 120°, so successive beacons cycle through three
   phases instead of repeating one. At 6°/s (60 s period) successive beacons went out at the same phase, and a station
   that lost one beacon to the null tended to lose the next. Station A's frames rise from 96.3 % (6°/s, same orbit)
   to 96.6 %; ±5 s of random jitter on each interval still recovers another 1.6 points (98.2 %), and 67 s gives 97.5 %.
   The flight software now applies that jitter (a uniform ±5 s on each beacon interval; RID RF-04, SQSO-ICD-005 v2.1
   §5.3), so **98.2 % is the as-built figure**. It keeps the beacon power line of BASELINE §5. The mission simulation's
   passes-heard figures at 6°/s and 2°/s (`docs/test/SIL_MISSION_SIMULATION.md`) predate the 8°/s baseline.
2. **Weak stations are limited by polarization, not by the spin.** For linear ground antennas the frame loss is
   dominated by linear-to-linear polarization fades; the spin rate changes it by no more than the sampling scatter. The
   crossed-element option (cross) raises a handheld-Yagi station from about 50 % to 72 % of frames and fills the 5 s
   dropouts.
3. **IDLE's 120 s interval** costs short passes: station A hears 98.2 % of passes at 120 s, against 99.6 % at 60 s.
4. **Recommendation (RF-03 in the review):** baseline the 2 m element along the spin axis (para) if one element is
   used, or two orthogonal 2 m elements on a quadrature hybrid (cross; the AntS offers phasing for its turnstile
   configuration) if the 70 cm element can share a face. The cross option is the only one that meets the proposed
   SYS-222 wording ("at least −6 dBi over at least 90 % of the sphere"); a single monopole reaches −6 dBi over 81 % of
   random orientations only, so the proposed wording cannot be met as it stands (for the chair).

## 6. Uplink Budget (435.5 MHz)

### 6.1 Telecommand

Worked example, station TC, 25 W, 10°, spacecraft gain −10 dBi, +6 dB interference:

```
Transmitter                    +44.0 dBm   (25 W)
Line loss                       −1.5 dB
Ground antenna                 +14.0 dBic
EIRP                           +56.5 dBm
Free-space loss (1,633 km)    −149.5 dB
Absorption, scintillation       −1.0 dB
Polarization (circ. → linear)   −3.0 dB
Spacecraft antenna             −10.0 dBi
Received carrier              −107.0 dBm
Noise density                 −162.6 dBm/Hz  (1,000 K, +6 dB interference)
C/N₀                            55.6 dB-Hz
Required (§3.2)                 53.0 dB-Hz
Margin                          +2.6 dB
```

| Spacecraft gain, interference | 5° | 10° | 20° | 30° | 45° | 90° |
|-------------------------------|----|-----|-----|-----|-----|-----|
| 0 dBi, +0 dB | +16.8 | +18.6 | +22.2 | +24.6 | +27.1 | +29.8 |
| −3 dBi, +0 dB | +13.8 | +15.6 | +19.2 | +21.6 | +24.1 | +26.8 |
| −3 dBi, +6 dB | **+7.8** | +9.6 | +13.2 | +15.6 | +18.1 | +20.8 |
| −10 dBi, +0 dB | +6.8 | +8.6 | +12.2 | +14.6 | +17.1 | +19.8 |
| −10 dBi, +6 dB | +0.8 | **+2.6** | +6.2 | +8.6 | +11.1 | +13.8 |
| −10 dBi, +10 dB | −3.2 | −1.4 | +2.2 | +4.6 | +7.1 | +9.8 |

LB-3 (≥ +6 dB at 10°, −10 dBi, +6 dB) is **not met** at 25 W: +2.6 dB. Options, all on the ground:

- **50 W** (+3 dB, +5.6 dB at 10°; meets LB-3 from about 11°). 47 CFR 2.106 US270 limits amateur stations to 50 W PEP
  within 240 km of Beale AFB (39°08′ N, 121°26′ W); Santa Clara is about 204 km away, so 50 W is the limit there unless
  97.313(f) is used: an earth or telecommand station may then use up to 611 W ERP (1 kW EIRP) in 435–438 MHz provided
  the lower half-power point of its antenna is always more than 10° above the horizon. With a 30° beamwidth cross-Yagi
  that means commanding only above about 25° elevation. **Baseline: 50 W PEP at the Bay Area station, no elevation
  limit; more power only under 97.313(f) above 25°** (to confirm with the FCC in the D3 consultation).
- Command above 20° (+6.2 dB at 25 W).
- Repeat each command (the counter rules of SQSO-ICD-013 §5.2 allow it with a new counter).

The contact analysis (`docs/ground/GROUND_STATION_DESIGN.md` §5) assumes commanding passes above 15° and four
attempts a pass; it shows the inhibit risk is set by ground availability, not by the link.

### 6.2 Transponder uplink SINAD at the spacecraft (users Doppler-corrected to ±2.5 kHz)

| Uplink | Spacecraft gain, interference | 5° | 10° | 20° | 30° | 45° | 90° |
|--------|-------------------------------|----|-----|-----|-----|-----|-----|
| M (50 W mobile vertical) | −3 dBi, +0 | 31 | 32 | > 35 | > 35 | > 35 | (overhead null) |
| M | −3 dBi, +6 dB | 25 | 27 | 30 | 32 | 35 | |
| BY (5 W, handheld Yagi) | −3 dBi, +0 | 27 | 29 | 33 | 35 | > 35 | > 35 |
| BY | −3 dBi, +6 dB | 17 | 22 | 27 | 29 | 32 | 34 |
| BY | −10 dBi, +6 dB | < 3 | 4 | 12 | 20 | 25 | 27 |
| HW (5 W, whip) | −3 dBi, +0 | **7** | 11 | **22** | 25 | 28 | **31** |
| HW | −3 dBi, +6 dB | < 3 | < 3 | 6 | 11 | 20 | 25 |

The mobile vertical's pattern has a null overhead (5/8-wave: about −10 dB at 60° elevation); its 45° and zenith
values are optimistic. A 5 W handheld with a whip reaches the spacecraft with 12 dB SINAD from about 11° in quiet
conditions, and only from about 30° over populated regions.

### 6.3 End-to-end transponder SINAD (both users at the elevation shown, −3 dBi each way, no interference)

| Uplink → listener | 5° | 10° | 20° | 30° | 45° | 90° |
|-------------------|----|-----|-----|-----|-----|-----|
| BY → B (handheld Yagis both ends) | **20** | 23 | **27** | 29 | 30 | **31** |
| M → B | 20 | 24 | 27 | 29 | 30 | 31 |
| BY → W (Yagi up, whip listener) | < 3 | 5 | 16 | 22 | 26 | 28 |
| HW → W (whips both ends) | < 3 | 4 | **15** | 21 | 24 | **26** |
| HW → D (whip up, rubber-duck listener) | < 3 | < 3 | < 3 | 5 | 11 | 20 |

The listener's downlink sets the result for every combination except a whip uplink at low elevation. LB-4 is **met**: for
handheld Yagis from 5°, and for whips from about 17° (the requirement is 30°).

### 6.4 Uplink with the receiver as designed (SQSO-RF-BRD-001 Version C)

The RF board (`hardware/electrical/rf/RF_BOARD_DESIGN.md` §4.3, §6.1; `hardware/electrical/rf/rf_calcs.py` §1, §16)
gives, at the antenna terminals with the 0.62 dB worst-case cable path at 436 MHz: system noise 548 K typical / 598 K
worst, and sensitivities of **−121.4 dBm** (12 dB SINAD voice) and **−115.8 dBm** (telecommand, 90 % of frames) worst
case. Those sensitivities carry a **2 dB implementation loss** for the CC1200's frequency output demodulated in the TNC
(not yet characterized, VI-RFB-01) on top of the thresholds of §3.2: they correspond to 55.0 dB-Hz for telecommand
(53.0 dB-Hz in §6.1) and 49.4 dB-Hz for voice (47.4 dB-Hz). Against the tables above (1,000 K, the §3.2 thresholds)
the design changes every uplink margin by +10 log(1000 / 948) − 2.0 = **−1.8 dB** worst (−1.5 dB typical), with the
350 K interference allowance kept on top of the design noise.

Telecommand margin (dB), 475 km, receiver as designed, worst case (typical in brackets):

| Station power | Spacecraft gain, interference | 5° | 10° | 20° | 30° |
|---------------|-------------------------------|----|-----|-----|-----|
| 25 W | −3 dBi, +6 dB | +6.0 (+6.3) | +7.8 (+8.1) | +11.4 | +13.8 |
| 25 W | −10 dBi, +6 dB | −1.0 (−0.7) | **+0.8** (+1.1) | +4.4 | +6.8 |
| **50 W PEP** | −3 dBi, +6 dB | +9.0 (+9.3) | +10.8 (+11.1) | +14.4 | +16.8 |
| **50 W PEP** | −10 dBi, +6 dB | +2.0 (+2.3) | **+3.8** (+4.1) | +7.4 | +9.8 |
| 50 W PEP | −10 dBi, +0 dB | +8.0 | +9.8 | +13.4 | +15.8 |

LB-3 (≥ +6 dB at 10°, −10 dBi, +6 dB) is therefore **not met at 10° even at 50 W PEP** with the designed receiver
(+3.8 dB); 50 W meets it from about 16° and 25 W from about 27°. The command-pass mask of 15° in the contact analysis
(SQSO-GND-001 §5) leaves +5.6 dB at 50 W in the worst case. The 2 dB is an allowance, not a measurement: if VI-RFB-01
measures the TNC demodulator within 0.5 dB of the simulated one, the budget values of §6.1 return (50 W: +5.6 dB at
10° worst). Transponder uplink SINAD near threshold (§6.2) falls by the same 1.8 dB: a 5 W handheld with a whip reaches
12 dB SINAD from about 12° instead of 11° in quiet conditions. The downlink is unaffected by the receiver and gains the
0.56 dB of §2.4 that the tables do not credit.

## 7. Receiver Desense and EMC in the RF Chain

`software/simulation/cdr/rf/desense_emc.py`; inputs stated there. The transponder is full duplex on one spacecraft: the
435 MHz receiver must hear a weak uplink while the 2 m transmitter radiates 0.5–1 W a few centimetres away.

### 7.1 Harmonics and the uplink choice

| Harmonic of 145.800–146.000 MHz | Falls on | Consequence |
|---------------------------------|----------|-------------|
| 2nd | 291.6–292.0 MHz | Federal band (225–400 MHz military aeronautical). 47 CFR 97.307(e): ≤ 25 µW and ≥ 40 dB below the carrier, i.e. **≥ 46 dBc at 1 W** |
| **3rd** | **437.4–438.0 MHz** | **Inside the 435–438 MHz uplink segment.** The tripled transponder emission occupies 2(3·5 + 3) = 36 kHz |
| 4th | 583.2–584.0 MHz | Broadcast TV; ≥ 46 dBc |

If 3 × f_down fell in the uplink channel, the harmonic would need 152 dB of suppression to stay 10 dB below the
receiver noise: unachievable. **The frequency plan is the protection:** the uplink must be at least 50 kHz from
3 × f_down (half the harmonic's 36 kHz, half the 15 kHz receiver filter, a 25 kHz guard), and for any downlink in the
segment that means **uplink in 435.000–437.300 MHz**. The IARU request states the narrower windows of BASELINE §7
(435.222–435.778 MHz, or else 436.222–436.500 MHz), which lie inside this range.

### 7.2 Output filter

The 7-element low-pass of `hardware/rf/TRANSMITTER_DESIGN.md` §7.3 (L 47, 82, 82, 47 nH; C 33, 47, 33 pF), analysed as
written:

| | 145.9 MHz | 291.8 MHz | 437.7 MHz | 583.6 MHz | −3 dB point |
|-|-----------|-----------|-----------|-----------|-------------|
| Ideal components | −0.06 dB, return loss 18 dB | −50.3 dB | −77.4 dB | −95.7 dB | 164 MHz |
| Inductor Q 80, capacitor Q 500 | **−0.66 dB** | −50.5 dB | −77.5 dB | −95.8 dB | |

The values work as a filter. A printed-board filter rarely realizes more than 50–60 dB of stopband at 437 MHz
because of coupling around the filter and inductor self-resonance, so the 3rd-harmonic figure that counts is the
measured one at the antenna port, with the PA's own harmonic level added (VI-RF-04). `hardware/rf/RF_ELECTRICAL_DESIGN.md`
§2.1 ("2nd < −30 dBc, 3rd < −40 dBc") does not meet 97.307(e) and is corrected there.

### 7.3 Full-duplex desense

The transmitter's 3rd harmonic is off channel when the plan of §7.1 holds, but its phase-noise skirt and the
transmitter's wideband noise still land in the uplink channel. With a TCXO-referenced PLL (fundamental phase noise
−118 dBc/Hz at 100 kHz, −135 dBc/Hz at 1 MHz [assumption]; the harmonic's skirt is 9.5 dB worse at three times the
offset), 1 W, and 15 dB of isolation between the 2 m element (three-quarter-wave resonant at 437 MHz) and the 70 cm
element on one face [assumption]:

| Uplink offset from 3 × f_down | Desense at 60 dBc 3rd harmonic | at 70 dBc | at 80 dBc | 3rd-harmonic suppression for ≤ 0.5 dB |
|-------------------------------|--------------------------------|-----------|-----------|--------------------------------------|
| 0.1 MHz | 23.8 dB | 13.9 dB | 5.3 dB | 93 dBc |
| 0.5 MHz | 11.7 dB | 3.8 dB | 0.6 dB | 81 dBc |
| 1.0 MHz | 7.2 dB | 1.5 dB | 0.2 dB | 75 dBc |
| **2.2 MHz** (435.5 against 437.7) | **3.2 dB** | **0.5 dB** | 0.05 dB | **70 dBc** |
| 5.0 MHz | | | | 65 dBc |

The fundamental's far-out noise (−150 dBc/Hz, attenuated 72 dB by the filter at 435.5 MHz) contributes nothing
measurable. **Finding (RF-07):** BASELINE §7's "≥ 60 dB 3rd-harmonic suppression target" costs 3.2 dB of uplink
sensitivity at 1 W whenever the transmitter is keyed with the planning frequencies, and more if the IARU assigns
frequencies closer together. Recommended baseline change (for the chair): "**≥ 70 dB** 3rd-harmonic suppression
measured at the antenna port, and the uplink at least 2 MHz below 3 × f_down; otherwise the harmonic suppression for
≤ 0.5 dB desense from `software/simulation/cdr/rf/desense_emc.py`". Verification: the full-duplex desense test VI-RF-06.

### 7.4 Receive filter and front-end linearity

The 2 m fundamental reaches the 70 cm receiver at about +5 dBm at 1 W through 25 dB of antenna isolation
[assumption]. An unfiltered front end (input IP3 0 dBm) would be 30 dB into blocking and would generate its own
437.7 MHz harmonic at +5 dBm.

| RX filter rejection at 145.9 MHz | 0 dB | 20 dB | 40 dB | **60 dB** | 70 dB |
|----------------------------------|------|-------|-------|-----------|-------|
| 2 m power at the LNA | +5 dBm | −15 dBm | −35 dBm | **−55 dBm** | −65 dBm |
| Harmonic generated in the LNA | +5.5 dBm | −55 dBm | −115 dBm | **−175 dBm** | −205 dBm |
| Margin to 1 dB blocking (−25 dBm) | −30 dB | −10 dB | +10 dB | **+30 dB** | +40 dB |

**Requirement: the RX filter shall reject 144–146 MHz by ≥ 60 dB** with ≤ 1.5 dB loss at 435–438 MHz (RF-P007 in
`docs/hardware/RF_DESIGN.md`). A 3-pole helical or cavity band-pass, or a high-pass plus SAW, meets it.

### 7.5 Intermodulation and other self-interference

| Mechanism | Product | Assessment |
|-----------|---------|------------|
| Third order, 2 × f_down ± f_x = f_up | f_x = 143.7 MHz (terrestrial 2 m) | Arrives from Earth at < −100 dBm; with the 60 dB RX filter, nothing |
| Second order, f_x ± f_down = f_up | f_x = 289.6 or 581.4 MHz | Military UHF and broadcast TV from Earth, < −90 dBm at the spacecraft; rejected by the RX filter's stopband (specify ≥ 40 dB at 250–330 MHz and 560–620 MHz) |
| Passive intermodulation (harmonic generation) in the antenna's hinge or tape contacts and the burn-wire mechanism | 3 × f_down after the filter | PIM bypasses the LPF. Stainless tape and steel springs at the AntS hinges are candidates. Covered by the frequency plan (off channel) and by measuring the radiated 3rd harmonic with the antennas deployed (VI-RF-04) |
| Transponder loop | Downlink at 145.9 MHz received by the 70 cm receiver as the 3rd harmonic | Off channel by the frequency plan; no regeneration path |
| Jetson, OBC clocks and switching regulators | Broadband EMI into the 70 cm receiver and the 2 m downlink | Radiated-emissions survey of the EM in each power state at 435–438 MHz and 145.8–146.0 MHz against the receiver noise floor (−127 dBm in 15 kHz) (VI-RF-07) |
| GNSS L1 (1575.42 MHz) | 2 m 11th harmonic 1604.9 MHz, 70 cm transmitter none | No conflict; the open GNSS L1 noise test (BASELINE §6) is separate |

## 8. Verification

Items that need hardware. Each is a defined verification item (VI); test procedures are `docs/test_procedures/TP-RF-*`.

| ID | What it replaces | Test and pass criterion | Verifies | When |
|----|------------------|-------------------------|----------|------|
| VI-RF-01 | Pattern assumption (§2.4, §5) | EM antenna pattern on the flight-like 1U body with wings deployed, both bands, ≥ 2 orthogonal cuts, NEC/FEKO model agreeing within 2 dB. Pass: median ≥ −3 dBi and 10th percentile ≥ −10 dBi over the pass-weighted directions of §5.2 | LB-1, LB-2, SYS-222 | EM, before FM build |
| VI-RF-02 | Transmitter output and losses (§2.4) | Conducted power at the antenna port at 0.5 and 1 W, −20 to +60 °C (TP-RF-002). Pass: 27.0 ± 0.5 dBm and 30.0 ± 0.5 dBm after the filter | LB-1 | EM, FM |
| VI-RF-03 | Demodulator thresholds (§3) | Packet error rate against input level for the flight TNC and receiver (uplink) and for a Dire Wolf ground demodulator against the flight transmitter (downlink), at 0 and 3.5 kHz offset (TP-RF-004). Pass: 90 % frame success at C/N₀ ≤ 52.9 dB-Hz (corrected) and ≤ 53.0 dB-Hz (telecommand) | LB-1, LB-3 | EM |
| VI-RF-04 | Harmonics and spurious (§7.1–7.2) | Spectrum at the antenna port, and radiated with antennas deployed, 1 W (TP-RF-003). Pass: 2nd and 4th ≤ −46 dBc and ≤ 25 µW (97.307(e)); **3rd ≤ −70 dBc** (proposed baseline, §7.3) | 97.307(e); RF-07 | EM, FM |
| VI-RF-05 | CTCSS and Doppler tolerance (§4.4) | Receiver with 67.0 Hz at ±0.5 kHz, carrier offsets 0 to ±2.5 kHz. Pass: tone decodes at all offsets at 6 dB SINAD; repeat path keys within 250 ms | MRD-031 | EM |
| VI-RF-06 | Full-duplex desense (§7.3) | Receiver sensitivity at the uplink frequency with the transmitter keyed at 1 W into the deployed antennas, coupling measured. Pass: ≤ 0.5 dB loss of 12 dB SINAD sensitivity | LB-3, LB-4 | EM (antenna mock-up), FM |
| VI-RF-07 | Self-generated EMI (§7.5) | Radiated noise at both receive bands in every power state (Jetson on, PA keyed, magnetorquers). Pass: ≤ −137 dBm in 15 kHz at the 70 cm receiver input (10 dB under its noise) | LB-3 | EM |
| VI-RF-08 | Receiver noise temperature (§2.4) | Noise figure of the receiver chain including the RX filter. Pass: T_sys ≤ 650 K with a 290 K source (NF ≤ 4.7 dB at the antenna port) | LB-3 | EM |
| VI-RF-09 | Interference allowance (§2.4) | On orbit: receiver RSSI over populated and quiet regions in commissioning. Pass: the +6 dB allowance holds over the telecommand stations | LB-3 | Commissioning |
| VI-RF-10 | Link margin on orbit | C/N₀ against elevation at station A over 20 passes. Pass: within 3 dB of §4.2 | LB-1 | Commissioning |

## 9. Reproduction

```bash
cd software/simulation/cdr/rf
python3 fm_demod_sim.py --out fm.json              # §3, about 12 minutes
python3 link_budget_cdr.py --fm fm.json --md lb.md  # §2, §4, §6
python3 pass_geometry_mc.py --fm fm.json --days 60 --out geom.json   # §4.4, §5; GND-001 §4 (475 km; --alt for the band)
python3 desense_emc.py --out desense.json           # §7
```

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-16 | Funding audit (TB-05) | Re-issued at the baseline orbit, frequencies and power: noise floor corrected; antenna pattern and ground stations stated; telecommand and transponder links added |
| 3.0 | 2026-10-27 | CDR review, communications panel | CDR re-issue (BASELINE §7, TB-05): simulated demodulator thresholds; pass-sampled pattern with the 6°/s spin and three element options; handheld, mobile and SatNOGS stations; transponder SINAD end to end; Doppler, CTCSS; desense, filter and intermodulation analysis; telecommand at 25 W found short of the proposed 6 dB at 10°; verification items VI-RF-01 to VI-RF-10 |
| 3.1 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: altitude row (500 km analysed; baseline 475 km target, 485 km cap), uplink request windows (§2, §7.1) and the 8°/s spin of BASELINE Rev E noted against the 6°/s pass analysis (§1); 525 km rows labelled as a bound. Analysis values unchanged |
| 3.2 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | RF board Version C (MMCX, 0.22 m of 1.37 mm coax: 0.44 dB at 146 MHz, 0.62 dB at 436 MHz; T_sys 548–598 K; sensitivity −121.4 / −115.8 dBm worst): §2.4 and new §6.4, telecommand with the designed receiver +0.8 dB at 10° at 25 W, +3.8 dB at 50 W PEP (worst), LB-3 met from about 16° at 50 W. Re-run at the Rev E baseline: every budget at 475 km with the 485 km cap as the bound (was 500 km, with 525 km); pass and fading analysis (§4.4, §5) at 8°/s (6°/s compared) from 2028-11-01. Beacon margin at 5° +7.9 dB (was +7.6), +0.9 dB at the 10th-percentile gain (was +0.6); telecommand +2.6 dB at 10° (was +2.3); station A 98.2 % of frames with the ±5 s jitter as built, 96.6 % without (was 97.2 % and 95.6 % at 6°/s); fades 5 s every 22.5 s; Doppler ±3.49 / ±10.42 kHz |
