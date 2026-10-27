# SMART-QSO Design-Precedent Survey (formerly Heritage Assessment)

**Document ID**: SQSO-HA-001
**Version**: 2.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft survey from public sources (web search, 2026-10-16). No SMART-QSO hardware exists

> **Funding audit (2026-10-16, PI-15).** Version 1.0 traced the design to four NASA Ames missions without a single
> source, gave TRLs and "heritage percentages" with no basis, and reported a 68 % heritage score that left the novel AI
> payload out of the weighting. SMART-QSO has no NASA relationship and no flight heritage of its own. This version is a
> **design-precedent survey**: for each subsystem it cites public missions that flew a similar design, says how close
> the precedent is, and states what no precedent covers. The 68 % score is removed (§5).

## 1. Purpose

Show, from public sources, which parts of the SMART-QSO design have flown before on comparable spacecraft and which
have not, so that reviewers can see where the technical risk sits.

A precedent is evidence that a design approach can work. It is **not** heritage for this spacecraft and does not
replace verification: the interim launch provider "does not allow the use of qualification-by-analysis, similarity, or
qualification testing on lower levels of assembly as a standalone method of verification" (SpaceX Rideshare Payload
User's Guide v11, §6.1).

**Source confidence** (the convention of `docs/funding_audit/research/`): **[P]** primary source read this session
(the mission's own or the manufacturer's publication, or the paper); **[S]** secondary source read this session (press,
encyclopaedia, database); **[U]** reported by a search result or another document but not read here.

## 2. Precedent Missions

| Ref | Mission | Year | Size | What is relevant to SMART-QSO | Source |
|-----|---------|------|------|-------------------------------|--------|
| M1 | AMSAT Fox-1A (AO-85) | 2015 (launched 8 Oct, Atlas V) | 1U | Single-channel FM transponder, 435.172 MHz up / 145.980 MHz down, 400 mW; from Dec 2018 low battery voltage in eclipse led AMSAT to disable the transmitters except for periodic telemetry | [S] <https://en.wikipedia.org/wiki/Fox-1A> |
| M2 | SaudiSat-1C (SO-50) | 2002 (20 Dec) | Microsatellite (not a CubeSat) | Long-lived amateur FM repeater (145.850 MHz up, 436.795 MHz down); listed as operational | [S] <https://db.satnogs.org/satellite/27607> |
| M3 | CUTE-I (Tokyo Institute of Technology) | 2003 (30 Jun, Rockot) | 1U | AX.25 packet telemetry at 1.2 kbit/s, Bell 202 (AFSK) modulation, 430 MHz band; continuous CW beacon | [S] <https://eoportal.org/satellite-missions/cute-i> |
| M4 | UPSat (Libre Space Foundation, University of Patras) | 2017 (launched 18 Apr, deployed from the ISS 18 May) | 2U | STM32F4 on-board computer running FreeRTOS; ground segment integrated with the SatNOGS network; hardware and software fully open source | [P] <https://www.libre.space/projects/upsat/> |
| M5 | SwissCube (EPFL) | 2009 | 1U | B-dot detumble controller with magnetorquers, magnetometers, gyros and sun sensors; flight results after 4 years (full paper not read) | [S] abstract, SmallSat 2013: <https://digitalcommons.usu.edu/smallsat/2013/all2013/1> |
| M6 | ESA Φ-Sat-1 | 2020 (Sept, Vega) | 6U | First onboard AI inference on an Earth-observation CubeSat: cloud screening on an Intel Myriad 2 chip that needed about two years of radiation characterization | [S] <https://philab.esa.int/flagship-programmes/phi-sats-programme/>; <https://www.ubotica.com/news/successful-launch-of-ground-breaking-ai-enabled-earth-observation-satellite> |
| M7 | Aethero Deimos | 2024 (16 Aug, Transporter-11) | Small CubeSat (listed as 1.5U [U]) | Carried Aethero's NxN edge-computing module, a customized carrier board for an NVIDIA Jetson Orin module, to low Earth orbit. **No on-orbit performance data has been published** | [S] <https://antmicro.com/blog/2026/03/antmicro-supports-aethero-with-nxn-and-nxa-space-computers> (2026-03-24); [U] <https://www.nanosats.eu/sat/deimos> |
| M8 | Planet Pelican-2 | Launch planned early 2025 (Transporter-12) | Not a CubeSat | Earth-observation satellite equipped with an NVIDIA Jetson module for onboard processing | [U] <https://www.businesswire.com/news/home/20241209902943/en/Pelican-2> |
| M9 | Starcloud-1 | 2025 (Nov) | Small satellite, not a CubeSat (about 60 kg [U]) | First NVIDIA H100 in orbit; ran an open Google model (the company page says both "Gemma" and "Gemini") and trained nanoGPT in orbit (company claims) | [S] <https://www.starcloud.com/starcloud-1> |
| M10 | "Space Llama" on the ISS | 2025 (Apr) | ISS payload (HPE Spaceborne Computer-2) | A fine-tuned Llama 3.2 run on HPE Spaceborne Computer-2 with NVIDIA GPUs aboard the ISS National Lab | [P] <https://about.fb.com/news/2025/04/space-llama-metas-open-source-ai-model-heading-into-orbit/> |

Supporting references:

| Ref | Reference | Relevance | Source |
|-----|-----------|-----------|--------|
| R1 | "A Review of Battery Technology in CubeSats and Small Satellite Solutions", *Energies* 2020, 13, 4097 | "COTS battery cells are the most popular to be used in CubeSats" (§3.1) [P]; reported to state that cylindrical 18650 cells are widely used [S] | <https://doi.org/10.3390/en13164097> |
| R2 | SkyFox Labs piNAV-NG GNSS receiver | Product line delivered from 2015, first flight 2017, more than 50 units (vendor and satsearch claims) | [U] `docs/funding_audit/research/D6_GNSS_RECEIVER.md` |
| R3 | NTNU master's thesis on keyed-hash (HMAC) uplink authentication for the NUTS CubeSat, 2014 | Design study of authenticated amateur-band telecommand; not a flight precedent | [U] <https://ntnuopen.ntnu.no/ntnu-xmlui/handle/11250/263077> |

## 3. Subsystem Precedent Matrix

No SMART-QSO subsystem has been built or tested, so no subsystem has a TRL supported by evidence. Version 1.0's TRLs
(4 to 9) had no basis and are removed. The CDR 2026 design review assessed TRLs from analysis, simulation and vendor
records in `docs/project/TECHNOLOGY_READINESS_ASSESSMENT.md` (SQSO-TRA-001), to be re-assessed with the EM results.

| Subsystem | SMART-QSO baseline (BASELINE §) | Closest flown precedents | How close | Not covered by any precedent cited |
|-----------|---------------------------------|--------------------------|-----------|------------------------------------|
| Structure | 1U, CDS Rev 14.1 (§4) | M1, M3, M5 (1U) | Same form factor | Deployable solar wings on a 1U (no precedent cited yet) |
| EPS | 16 triple-junction cells, 4S1P 18650 Li-ion ≈ 50 Wh (§5) | R1 (COTS Li-ion cells in CubeSats) | Cell type common; pack design is ours | Power closure with a duty-gated transponder and an 8 W Jetson burst; battery ageing (compare M1) |
| OBC and flight software | STM32L4R5ZI; Rust flight software on the Embassy executor, no RTOS (§6, rebaselined 2026-10-25, CR-2026-002) | M4 (STM32F4 + FreeRTOS) | Same MCU family, different part (L4, not F4). M4's RTOS precedent does not apply: SMART-QSO has no RTOS | Our software has not run on any target |
| RF: transponder | Single-channel U/V FM, 0.5–1 W, duty-gated (§2, §7) | M1 (1U, same band plan), M2 | Close; M1 is the same concept on a 1U | Duty gating by onboard AI; M1's later battery trouble shows the energy risk |
| RF: telemetry | 1200 bps AX.25 AFSK beacon (§2) | M3 | Same modulation and protocol | — |
| Telecommand | HMAC-SHA-256 with monotonic counter, TX-OFF (§7) | R3 (design study only) | No flown amateur-band precedent cited | Flight demonstration of authenticated amateur telecommand (search did not find one) |
| ADCS | 3-axis magnetorquers, B-dot detumble, coarse sun-pointing (§8) | M5 | Same control approach on a 1U | Our coil design and controller tuning |
| GNSS | piNAV-NG/FM receiver, duty-cycled (§6) | R2 | Same unit (vendor claims) | Duty-cycled operation meeting the ephemeris rule (open with SpaceX, D6) |
| AI payload: compute | Jetson Orin Nano 8 GB, 7 W mode, ≤ 8 W input, power-gated (§6) | M7 (Orin-class on a small CubeSat), M8 (Jetson, not a CubeSat) | Same module family; M7 has no published results | Orin **Nano** in its 7 W mode, thermal path on a 1U, latch-up handling; no published on-orbit data for any Jetson on a CubeSat |
| AI payload: models | OBC selector (flight baseline); SmolLM2-360M or Qwen3-0.6B text as a gated experiment (§6) | M6 (onboard inference on a 6U); M9, M10 (LLMs in orbit, not CubeSats) | Inference on a CubeSat has flown (M6); LLMs have run in orbit only on far larger platforms | **An LLM on a CubeSat-class bus: no precedent found.** Energy per accepted beacon (≤ 0.1 Wh gate) is unmeasured |
| Ground | Amateur stations, open telemetry formats (§2) | M4 (SatNOGS integration) | Same open-network approach | — |

## 4. Detailed Precedent Notes

### 4.1 Electrical Power System

- **Precedents.** COTS Li-ion cells are the usual CubeSat energy storage (R1). M1 shows a real failure mode: a 1U
  amateur transponder whose battery could no longer carry it through eclipse after about three years.
- **Differences.** 4S1P pack at about 50 Wh, a power-gated Jetson rail (≤ 8 W input), GNSS duty cycling, and a
  transponder gated by state of charge (BASELINE §5).
- **Not covered.** The power budget closes by analysis and simulation only (+33 % tumbling, +70 % sun-pointed on the
  allocation; depth of discharge ≤ 2.3 % as flown in the mission simulation; BASELINE §5); the engineering-model test
  is still needed.

### 4.2 Command and Data Handling

- **Precedents.** M4 flew an STM32F4 with FreeRTOS on a 2U and published its flight software.
- **Differences.** STM32L4R5ZI, not F4. No RTOS: the Rust flight software runs on the Embassy executor with one task
  (BASELINE §6, rebaselined 2026-10-25). CI builds and checks the image; it has not run on hardware
  (`docs/reviews/FLIGHT_SOFTWARE_DELIVERY.md`).
- **Not covered.** Radiation behaviour of the specific part and board; our software on target hardware.

### 4.3 RF Communications

- **Precedents.** M1 is the closest precedent for the mission as a whole: a 1U with a single-channel FM transponder
  (uplink 435.172 MHz, downlink 145.980 MHz). M2 shows an amateur FM repeater can serve users for decades. M3 flew 1200 bps
  AX.25 telemetry on a 1U in 2003.
- **Differences.** Duty gating of the transponder by an onboard scheduler; authenticated telecommand with a hardware
  TX-OFF path (BASELINE §7).
- **Not covered.** A flown amateur-band authenticated telecommand scheme (only a design study, R3, was found).

### 4.4 Attitude Determination and Control

- **Precedents.** M5 flew a B-dot controller with magnetorquers on a 1U and reported results after four years.
- **Differences.** CubeSpace CR0002 rods (> 0.2 A·m²; air-core coils the alternate) and spin-stabilized coarse
  sun-pointing of +Y at 8°/s (BASELINE §8).
- **Not covered.** Detumble and capture on the real spacecraft: the Monte Carlo of the flight code gives a slowest
  in-specification capture of 1.8 h (tip-off σ 10°/s, up to 30°/s;
  `docs/hardware/ADCS_DESIGN.md` 1.9 §11.4); hardware-in-the-loop and flight are still to come.

### 4.5 Software

- **Precedents.** M4 published open flight software for an STM32 and FreeRTOS CubeSat.
- **Differences.** Rust flight software on the Embassy executor, no RTOS (2026-10-25: the earlier text described the C
  host simulation, since replaced). It runs as a host simulation and as an image that has not run on hardware; the C
  test suites specify it (`docs/reviews/FLIGHT_SOFTWARE_DELIVERY.md` §4).
- **Not covered.** Software on the flight target; integration with the payload over the real link.

### 4.6 AI Payload

- **Precedents.**
  - *Onboard inference on a CubeSat* has flown: M6 (2020, 6U, Myriad 2).
  - *Jetson-class compute on a CubeSat* has flown once in the sources found: M7 (2024, Jetson Orin carrier on a small
    CubeSat). Its operator has published no on-orbit results. M8 put a Jetson on a larger Earth-observation satellite.
  - *Large language models in orbit* have run only on much larger platforms: M9 (H100 GPU, about 60 kg, 2025) and M10
    (ISS, 2025). Both results are company or press claims.
- **Not covered.** A Jetson Orin **Nano** in the 7 W mode on a 1U (NVIDIA lists that mode only for the original
  configuration, finding TB-13); its thermal path (now a temperature requirement, spreader ≤ +40 °C in a 600 s batch, with a 1.10 W/K design path to be
measured on the EM; SYS-603 Rev G, BASELINE §6); and **any
  language model on a CubeSat-class bus**. The flight baseline therefore keeps generation as a gated experiment behind a
  measured energy limit, with a fixed-template fallback (BASELINE §6, decision D5).

## 5. Heritage Score (removed)

Version 1.0 reported an "overall heritage score" of 68 %. It is removed because:

1. none of its subsystem percentages (50 % to 90 %) had a source; and
2. its weights (EPS 20 %, OBC 20 %, RF 15 %, ADCS 10 %, structure 10 %, software 15 %, ground 10 %) summed to 100 %
   **without the AI payload**, the most novel part of the spacecraft, which version 1.0 itself rated at 10 %.

A weighted percentage cannot be computed from citations. What can be counted is the number of **flown** precedents
cited in §2 for each subsystem. Method: count the distinct missions M1–M10 in the §3 "closest flown precedents" column;
references R1–R3 are not missions and are not counted.

| Subsystem | Flown precedents cited |
|-----------|------------------------|
| Structure | 3 (M1, M3, M5) |
| EPS | 0 missions (R1 is a review) |
| OBC and flight software | 1 (M4) |
| RF: transponder | 2 (M1, M2) |
| RF: telemetry | 1 (M3) |
| Telecommand authentication | 0 |
| ADCS | 1 (M5) |
| GNSS | 0 missions (R2 is a vendor claim) |
| AI payload: compute | 2 (M7, M8), neither with published results |
| AI payload: language model on a CubeSat | 0 (M9 and M10 are not CubeSats) |
| Ground | 1 (M4) |

These counts show where precedent exists; they do not measure risk.

## 6. Where the Risk Sits

| Area | Precedent | Risk |
|------|-----------|------|
| 1U structure, AX.25 telemetry, FM transponder concept | Flown (M1, M3) | Low to moderate; depends on our build and test |
| Power budget with transponder and Jetson bursts | Partial (M1 shows the failure mode) | High until the power simulation and EM test close |
| Authenticated amateur telecommand | None flown (design study only) | Moderate; regulatory and implementation |
| Jetson Orin Nano on a 1U | One related demonstrator without results | High (power, thermal, radiation) |
| Language model on a CubeSat | None | Highest; mitigated by keeping it a gated experiment with a template fallback |

## 7. How Precedents Will Be Used

| Use | Method |
|-----|--------|
| Design choices | Read the cited mission papers before each design review (for example the full SwissCube paper before the ADCS review) |
| Verification | Test and analysis of SMART-QSO hardware at the launch provider's levels; similarity is not accepted on its own (SpaceX UG v11 §6.1) |
| Risk register | Carry the "not covered" items in §3 as risks |

## 8. Conclusion

The bus follows designs that have flown: a 1U U/V FM transponder (Fox-1A), 1200 bps AX.25 telemetry (CUTE-I), an
STM32 on-board computer (UPSat, which ran FreeRTOS; SMART-QSO has no RTOS) and B-dot magnetorquer control (SwissCube). The AI payload is largely without precedent:
one Jetson Orin-class module has flown on a small CubeSat with no published results, and no language model has been
found running on a CubeSat-class bus. That is why the flight baseline does not depend on the payload.

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 2.0 | 2026-10-16 | SMART-QSO Team | Funding audit PI-15: rewritten as a design-precedent survey with public citations, including the AI payload; unsourced NASA Ames heritage, TRLs and the 68 % heritage score removed; precedent counts shown with their method |
| 2.0, note | 2026-10-24 | SMART-QSO Team | OBC row: the implementation is Rust on the Embassy executor, so M4's FreeRTOS precedent covers the MCU family only |
| 2.0, note | 2026-10-25 | SMART-QSO Team | Embassy executor, no RTOS, is now the baseline (BASELINE §6, CR-2026-002): OBC row, §4.2, §4.5 and §8 |
| 2.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): §3 points to the TRA (SQSO-TRA-001); §4.1 power closure by simulation; §4.4 CR0002 rods, 8°/s, detumble Monte Carlo; §4.6 Jetson heat path as the SYS-603 Rev G temperature requirement |
