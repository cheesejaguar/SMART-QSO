# SMART-QSO Electromagnetic Compatibility Assessment

**Document ID**: SQSO-EMC-001
**Version**: A
**Date**: 2026-10-27
**Status**: CDR assessment (analysis and design rules). No unit exists; the emission levels of the Jetson carrier,
the converters and the PA are not yet measured. The tests that close it are in §6 and in
`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §3.6.

## 1. Scope and requirements

Self-compatibility of a 1U that carries a 1 W 2 m transmitter, an always-on 70 cm receiver (97.207(b)), a GNSS
receiver (D6), a magnetometer, magnetorquers and an 8 W Jetson, all within 10 cm of each other; and compatibility with
the launch vehicle.

| Source | Requirement |
|--------|-------------|
| SYS-713 | Emissions measured at the 435 MHz receiver input, every mode including a Jetson burst and magnetorquer actuation, ≥ 6 dB below the receiver's minimum usable signal; normal operation while each transmitter radiates full power |
| SYS-203; BASELINE §7 | Spurious ≥ 46 dBc and ≤ 25 µW at 1 W (47 CFR 97.307(e)); third harmonic ≥ 60 dBc |
| SYS-085, SYS-704, SYS-905 | Powered off in the dispenser; inhibits verified in vibration; launch-vehicle EMI compatibility by the inhibits |
| SpaceX UG v11 §4.1.7, Tables 4-7 to 4-9, §6.7.8, Table 6-2 | LV and range fields 120–150 dBµV/m; maximum payload emissions 108 dBµV/m (30–1000 MHz) and 33 dBµV/m in 1127–1327 and 1475–1675 MHz, by test with 6 dB margin; EMC verification advised (MIL-STD-461 or an accredited lab) |
| GSFC-STD-7000B §2.5, Table 2.5-2; Appendix B §2.5 | Tailored MIL-STD-461G CE101, CE102, RE102, RS103, CS101, CS06 at unit level; CubeSat self-compatibility at system level |

## 2. Victims

| Victim | Frequency | Sensitivity | Why it matters |
|--------|-----------|-------------|----------------|
| 70 cm receiver (telecommand, transponder input) | 435–438 MHz, one coordinated channel, ≈ 16 kHz | Minimum usable signal from the link analysis (SQSO-LBA-001), of order −120 dBm | Loss of telecommand, including TX-OFF; transponder desense |
| GNSS receiver (piNAV-NG) | L1 1575.42 MHz ± 1 MHz | Signals ≈ −130 dBm; C/N0 loss ≤ 3 dB required (ETP §3.6) | SpaceX ephemeris requirement (UG v11 §9.5.7.2) |
| Magnetometer | DC–10 Hz | 15–30 nT noise | ADCS sun capture and spin hold |
| BMS cell-voltage taps, NTCs, OBC ADC, I²C | DC–kHz, rectification of RF | mV | False over/under-voltage trips (H1, H2 in SQSO-BSA-001), false temperatures |
| Deployment and umbilical sense lines | DC | logic | False deployment state |

## 3. Sources and the coupling paths

| # | Source | Emission | Path to the victims | Assessment |
|---|--------|----------|---------------------|------------|
| S1 | Jetson Orin Nano module and carrier | Broadband digital: SoC and memory clocks, LPDDR5, PCIe, USB; the module's reference oscillator (38.4 MHz in the Orin family, to be confirmed for the module) has its 41st harmonic at 1574.4 MHz, inside the L1 C/A main lobe; carrier DC-DC converters | Radiated to the 70 cm and GNSS antennas and the GNSS patch on a body face; conducted on the 5 V rail shared with the PA (S2 sheet: SW1 and SW3 on one buck) | **Highest risk.** Not predictable to the needed accuracy; must be measured on the EM (V-EMC-1, V-EMC-2). The Jetson runs only in sunlit bursts, so the GNSS sessions and the telecommand passes can be scheduled away from them as an operational fallback |
| S2 | EPS converters: MPPT boost (≈ 0.5–2 MHz), 3V3_AO buck, 5 V buck (≈ 2 MHz), burn-wire PWM | Harmonic combs n × f_sw; free-running oscillators drift ±10 %, so near 435 MHz (n ≈ 200) a comb line can sit anywhere | Conducted on VBAT and the rails; radiated from the switch nodes and inductors | Medium. Design rule E1–E3 below; measure CE102 at unit level |
| S3 | Magnetorquer PWM drivers | kHz PWM, coil currents up to ≈ 90 mA | Conducted on 3V3_AO, which also feeds the receiver through SW2; magnetic to the magnetometer | Medium. Rule E4, E5 |
| S4 | PA, 1 W at 145.8–146.0 MHz | Fundamental, harmonics (3f = 437.4–438.0 MHz falls in the uplink band), supply-current pulses 0 → 0.54 A at 5 V | Near-field of ≈ 70 V/m at 10 cm from a quarter-wave monopole at 1 W (E ≈ √(30 P G)/r, far-field formula as an order of magnitude); conducted on the 5 V rail and VBAT | Harmonics: RF discipline (BASELINE §7). Self-susceptibility of the BMS taps, ADCs and sense lines: rules E6, E7 and the RF self-compatibility test |
| S5 | Load steps: Jetson on (0.62 A at the pack), latch-up (0.93 A), burn wires (1.7 A) | Bus droop through the pack impedance (§4) | All loads on VBAT | Low at 25 °C; the cold-pack burn sets the UVLO rule E8 |
| S6 | BMS AFE and secondary protector while integrated | µA-level internal oscillators, if running | Radiated, while the spacecraft is "powered off" | Negligible, but the AFE should be in ship mode (clocks stopped) at delivery so that the claim "no emissions while integrated" (UG v11 Table 4-9) is literally true (rule E9) |

No 2 m harmonic falls on L1 (10 × 145.9 = 1459 MHz; 11 × 145.9 = 1605 MHz), and no uplink harmonic does
(3.61 × 436 MHz). The 2 m third harmonic is in the 70 cm uplink band; BASELINE §7 already requires the uplink
frequency away from 3 × f_downlink and ≥ 60 dBc suppression.

## 4. Bus transients (analysis)

Pack model of the mission simulation (R0 = 40 mΩ/cell at 25 °C, Arrhenius 28 kJ/mol, ×1.6 at empty; R1 = 15 mΩ;
30 mΩ harness), `software/simulation/cdr/power_thermal/bus_transients.py`:

| Pack | SOC | OCV (V) | Jetson on, 8 W | Latch-up, 12 W | PA keyed, 1 W RF | Burn, 1.7 A |
|------|-----|---------|----------------|----------------|------------------|-------------|
| +25 °C | 90 % | 16.28 | 16.12 | 16.10 | 16.24 | 15.86 |
| +25 °C | 15 % | 14.00 | 13.82 | 13.78 | 13.95 | 13.49 |
| 0 °C | 55 % | 15.04 | 14.64 | 14.59 | 14.94 | 13.94 |
| −10 °C | 55 % | 15.04 | 14.41 | 14.34 | 14.88 | 13.31 |
| −10 °C | 15 % | 14.00 | 13.23 | 13.10 | 13.80 | **11.90** (below the 12.0 V low-battery level) |
| −20 °C | 55 % | 15.04 | 14.01 | 13.90 | 14.78 | 12.20 |
| −20 °C | 15 % | 14.00 | 12.74 | 12.53 | 13.67 | **10.56** (below the 11.2 V BMS cut) |

The PA keying step is 40 mV at 25 °C and 110 mV in the cold, low-SOC corner: within the ETP's 200 mV bus criterion.
The deployment burn after a cold soak (pack ≈ −10 … −16 °C at ≈ 58 % SOC; THERMAL_ANALYSIS §4) leaves 12.2–13.3 V;
it would trip the BMS only below ≈ 25 % SOC at −20 °C. This sets the launch state of charge (SQSO-BSA-001 §6:
≥ 40 % at deployment) and the converters' under-voltage lock-out (rule E8).

## 5. Design rules (EPS and harness)

| Rule | Requirement | Owner | Closes |
|------|-------------|-------|--------|
| E1 | Synchronise the EPS converters (MPPT, 3V3_AO, 5 V) to a crystal-derived clock from the OBC or a local oscillator, with a frequency plan putting no harmonic within 200 kHz of the coordinated uplink channel; no spread-spectrum modulation on the converters (it turns comb lines into a raised floor across the 70 cm channel) | EPS | S2 |
| E2 | LC input filters on each converter; switch nodes and inductors shielded or on the side of the board away from the antennas; ground plane unbroken under the converters (EPS_ELECTRICAL_DESIGN §4.5) | EPS | S2 |
| E3 | Separate the PA supply from the Jetson's: a dedicated PA converter, or the Jetson carrier fed from VBAT through its own converter behind the eFuse; the 5 V buck shared by both (S2 sheet) is not acceptable for supply-induced modulation of the downlink | EPS, payload | S1, S4 |
| E4 | Receiver, CTCSS decoder and TNC supplied through their own LDO from 3V3_AO (PSRR ≥ 40 dB at the PWM frequency); magnetorquer drivers on a separately filtered branch | EPS, RF | S3 |
| E5 | Magnetometer samples only with the coils off (time-multiplexed in the flight software) and ≥ 40 mm from the coils and the Jetson's 5 V wiring; twisted pairs for coil and Jetson power | ADCS | S3 |
| E6 | RC filters (1 kΩ + 0.1 µF, as drawn on S1) on every BMS cell tap and NTC, at the AFE; ferrites on the pack harness | EPS | S4 |
| E7 | Shielded or twisted RF-adjacent harness; coax for the 70 cm and GNSS antennas (W10) routed away from the 2 m element base | Harness | S4 |
| E8 | Converter under-voltage lock-out at 10.8 V falling (below the BMS cut at 11.2 V, so the BMS, not a converter brown-out, decides); OBC brown-out reset above the 3V3_AO drop-out | EPS | S5 |
| E9 | BMS AFE in ship mode from pack assembly to deployment; woken by the deployment switch closure | EPS | S6 |

## 6. Verification

The environmental test plan's §3.6 already carries the system-level self-compatibility tests (bus ripple,
receiver-input emissions in every mode, RF self-compatibility at full power, spurious and harmonics, GNSS L1 noise,
magnetometer). What it lacks are the unit-level tests that let the design be fixed before the system test, and the
pass criteria in numbers:

| ID | Test | Article | Pass criterion | Verifies | When |
|----|------|---------|----------------|----------|------|
| V-EMC-1 | Jetson carrier, unit level: CE102 (tailored, GSFC-STD-7000B Table 2.5-2) on its input; near-field scan 30 MHz–2 GHz with the module at 7 W inferring | Jetson EM carrier | CE102 tailored limit; no line within ± 100 kHz of the coordinated uplink or within L1 ± 2 MHz above −110 dBm at a reference antenna 10 cm away (to be re-derived from the measured receiver sensitivity) | S1, SYS-713 | Payload EM, before carrier layout freeze |
| V-EMC-2 | GNSS L1 noise with the Jetson inferring, PA keyed, magnetorquers at full PWM | EM | C/N0 loss ≤ 3 dB; fix kept (ETP §3.6) | S1, D6 | EM |
| V-EMC-3 | EPS board, unit level: CE102 tailored on VBAT and on each rail; frequency plan check of the synchronised converters with a spectrum analyser at the 70 cm port | EPS EM | Tailored CE102; no converter harmonic within 200 kHz of the uplink channel | E1, E2 | EPS EM |
| V-EMC-4 | Receiver sensitivity with every mode and load running (the SYS-713 test), including the Jetson burst and magnetorquer PWM | EM, FM | Degradation ≤ 1 dB of the receiver's sensitivity measured with everything off; equivalently, emissions ≥ 6 dB below the minimum usable signal | SYS-713 | EM, FM |
| V-EMC-5 | RF self-compatibility at 1 W: BMS cell voltages, NTCs, sun sensors, magnetometer, I²C error counters logged with the PA keyed 120 s | EM, FM | No false trip; no change in any telemetry above its noise | S4, E6 | EM, FM |
| V-EMC-6 | Load-step test on the bus: Jetson on, latch-up emulation at 12 W, burn at 1.7 A, at a cold pack (0 °C, 25 % SOC on the EM) | EM | Bus stays above 12.0 V; no OBC reset; matches §4 within 0.2 V | S5, E8 | EM |
| V-EMC-7 | Powered-off emissions while integrated: confirm by inspection that only the AFE (ship mode) and the secondary protector are powered with the deployment switches actuated | FM | No clock running; UG v11 Table 4-9 met by having no source | S6, SYS-905 | FM, with the inhibit test |

## 7. RIDs

| RID | Severity | Finding | Status |
|-----|----------|---------|--------|
| EMC-01 | 2 | No EMC analysis existed; EPS_ELECTRICAL_DESIGN §8 cited "CE102/RE102 tailored" with no limits, sources or victims | Closed by analysis (this document); test items V-EMC-1 to V-EMC-7 |
| EMC-02 | 2 | The Jetson and the PA share one 5 V buck (S2 sheet) | Design change for the EPS designer (rule E3); for the chair to direct |
| EMC-03 | 2 | Converter harmonics near 435 MHz unplanned; spread spectrum recommended in PDU_DESIGN §11.2 would raise the receiver's noise floor | Design rule E1 (PDU_DESIGN updated) |
| EMC-04 | 3 | The Jetson's 38.4 MHz-family reference may put its 41st harmonic in the L1 band | Verification item V-EMC-1/2 |
| EMC-05 | 3 | ETP §3.6 has no unit-level tests or numeric criteria for the Jetson carrier and the EPS | For the chair (the ETP's owner): add V-EMC-1, -3, -6 |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR review, EPS/thermal | Initial issue: victims, sources, bus transients, design rules, verification |
