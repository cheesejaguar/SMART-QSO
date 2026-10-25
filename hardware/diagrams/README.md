# System Diagrams: Block, Wiring and Schematic

**Document Number:** SQSO-HW-DIAG-001
**Revision:** A
**Date:** 2026-10-25
**Status:** Draft (preliminary: drawn from the repository's documents and firmware, not from a released design)

---

## 1. Purpose and Standing

This set draws the whole spacecraft at three levels: block diagrams (B1–B4), wiring diagrams for the harnesses (W1–W10)
and electrical schematics (S1–S9). Each drawing is generated from a text source in this directory, so a change shows up
in review as a diff.

The drawings record the design as the repository states it today, and the gaps in it. They do not select parts.
`docs/BASELINE_PARAMETERS.md` governs every value. Where a drawing disagrees with the baseline, the baseline wins and the
drawing is wrong. Pins come from the firmware (`software/rust/firmware/src/gpio.rs`, `main.rs`, `adc.rs`, `uart.rs`,
`i2c.rs`, `spi.rs`, `fram.rs`), which are themselves provisional (TBR) and have not run on hardware. Anything a drawing
adds that no document or firmware file has is marked **proposed**, and is listed in §4.

## 2. Contents

| ID | Drawing | Source | Main sources drawn from |
|----|---------|--------|-------------------------|
| B1 | System overview | `hardware/diagrams/block/B1_system_overview.dot` | BASELINE 2, 4–7 |
| B2 | Power distribution | `hardware/diagrams/block/B2_power_distribution.dot` | BASELINE 5; `hardware/eps/PDU_DESIGN.md` |
| B3 | OBC data and control interfaces | `hardware/diagrams/block/B3_data_control.dot` | firmware pin map |
| B4 | RF chain | `hardware/diagrams/block/B4_rf_chain.dot` | `hardware/rf/TRANSMITTER_DESIGN.md`, `hardware/rf/RF_ELECTRICAL_DESIGN.md` |
| W1 | Battery pack to EPS | `hardware/diagrams/wiring/W1_battery_eps.yml` | SQSO-HAR-001 H1 |
| W2 | Solar panels to EPS | `hardware/diagrams/wiring/W2_solar_eps.yml` | SQSO-HAR-001 H8; `hardware/eps/SOLAR_ARRAY_DESIGN.md` |
| W3 | EPS to OBC | `hardware/diagrams/wiring/W3_eps_obc.yml` | replaces H2 (proposed) |
| W4 | OBC and EPS to RF board | `hardware/diagrams/wiring/W4_rf_board.yml` | replaces H4 (proposed) |
| W5 | EPS and OBC to Jetson carrier | `hardware/diagrams/wiring/W5_jetson.yml` | H3, H6 |
| W6 | ADCS | `hardware/diagrams/wiring/W6_adcs.yml` | replaces H5 (proposed) |
| W7 | Deployables, burn and sense | `hardware/diagrams/wiring/W7_deployables.yml` | proposed |
| W8 | GSE umbilical | `hardware/diagrams/wiring/W8_gse_umbilical.yml` | proposed |
| W9 | GNSS receiver | `hardware/diagrams/wiring/W9_gnss.yml` | proposed (finding F2) |
| W10 | RF coaxial cables | `hardware/diagrams/wiring/W10_rf_coax.yml` | H7 |
| S1 | EPS power path and inhibits | `hardware/diagrams/schematics/S1_eps_power_path.py` | BASELINE 4–5; battery, PDU, solar designs |
| S2 | EPS rails and load switches | `hardware/diagrams/schematics/S2_rails_load_switches.py` | PDU_DESIGN limits; firmware enables |
| S3 | OBC core (STM32L4R5ZI) | `hardware/diagrams/schematics/S3_obc_core.py` | firmware pin map; OBC and storage designs |
| S4 | RF board | `hardware/diagrams/schematics/S4_rf_board.py` | TRANSMITTER_DESIGN 7–10 |
| S5 | Burn-wire drivers and deployed-state sense | `hardware/diagrams/schematics/S5_burn_wire.py` | firmware PB9–PB12, PA1 |
| S6 | ADCS board | `hardware/diagrams/schematics/S6_adcs.py` | `hardware/adcs/ADCS_ELECTRICAL_DESIGN.md` |
| S7 | Jetson power gate and link | `hardware/diagrams/schematics/S7_jetson_power_link.py` | payload power management; firmware PA2, USART1 |
| S8 | GNSS receiver interface | `hardware/diagrams/schematics/S8_gnss.py` | BASELINE D6 (proposed) |
| S9 | ADC front ends | `hardware/diagrams/schematics/S9_adc_front_ends.py` | `software/rust/firmware/src/adc.rs` |

The rendered drawings are in [`svg/`](svg/), one file per ID (for example [`svg/B1_system_overview.svg`](svg/B1_system_overview.svg)).

## 3. Conventions

**Block diagrams.** Red is power, blue is data, purple is RF, grey is audio or baseband, and dashed green is a control or
enable line. In B3, orange is a discrete input and brown an analog input. A dashed box is a part that has not been
selected.

**Wiring diagrams.** These are WireViz harness drawings. Each one gives connectors, pin labels, wire colours, gauges and
lengths. Connector families follow SQSO-HAR-001 §3.1: Micro-Fit 3.0 for high-current power, Pico-Clasp for low-current
power, Samtec FTSH board to board, a 2.54 mm header for debug, and SMA or U.FL for RF. Gauges follow its §5.2 derating
table. Every pinout is proposed and TBR. The header comment of each source says which SQSO-HAR-001 harness (H1–H8) it
keeps or replaces.

**Schematics.** These are schemdraw sheets. A signal that leaves a sheet ends in a flag carrying its net name. The same
names are used in the harnesses and on the other sheets, so `RF_TX_EN` on S2 is the net of that name on S3 and W3.
On every sheet:

- a dashed block is a part that has not been selected, and its label says "(TBD)";
- a value that has not been confirmed carries "TBR";
- a pin or net marked `*` is proposed here and is not in the firmware.

## 4. Design Findings

These came up while drawing. Each one is something a schematic cannot be finished without. None has been entered in
`docs/funding_audit/FINDINGS_REGISTER.md` or the TBD register. Whether to do that is the PI's decision.

| ID | Finding | Evidence | Drawn as |
|----|---------|----------|----------|
| F1 | The VBATT divider is a 2S value. `FRONT_END` scales VBATT by 3 ("1:3 divider, 2S pack, 8.4 V max"), but the pack is 4S (12.0–16.8 V). On a 1:3 divider a full pack puts 5.6 V on PC2, above the MCU's supply. | `software/rust/firmware/src/adc.rs`; BASELINE 5 | S9: 100k/20k (1:6, 2.8 V at 16.8 V); `adc.rs` needs `Linear(6.0, 0.0)` with it |
| F2 | The GNSS receiver (D6) has no UART, no enable and no 1PPS input in the firmware or any ICD. | `software/rust/firmware/src/main.rs` | Proposed UART4 on PC10/PC11 (SPI3's pins; nothing uses SPI3), GNSS_EN on PE4 via SW7, and 1PPS to a timer input (B3, W9, S8) |
| F3 | The ADCS has no magnetorquer drive pins (PWM, nSLEEP, nFAULT) and no sun-sensor input. ADCS-I001 reserves 0x40–0x45 for sun sensors, which points to digital sensors, while the sensor design describes photodiodes. | `docs/hardware/ADCS_DESIGN.md` 2.3; `hardware/adcs/SENSOR_DESIGN.md` | Proposed: 6 timer PWM lines with nSLEEP/nFAULT (W6), and photodiodes into OPA333 TIAs and an 8-channel I2C ADC (S6) |
| F4 | There is one burn line and one sense line for two antennas, and one of each for two wings. | `software/rust/firmware/src/gpio.rs` (PB9–PB12) | One driver feeding two resistors in parallel; the two switches of a pair in series (W7, S5) |
| F5 | The third independent physical inhibit is not defined. CDS 2.3.7 and 2.3.8 require at least three, and the RBF does not count. | BASELINE 4 | Dashed "Inhibit 3 (TBD)" (B2, S1) |
| F6 | No documented regulator takes a 4S input. PDU_DESIGN's regulators are specified for 6.0–8.4 V. | `hardware/eps/PDU_DESIGN.md` | Both bucks TBD, input rated at least 25 V (S2) |
| F7 | A 5 Ω burn resistor on 14.4 V draws about 2.9 A, so the drive must limit the current to 0.5–1 A. | `hardware/rf/ANTENNA_DESIGN.md`; BASELINE 5 | PWM on PB9/PB10 (TIM4_CH4, TIM2_CH3) or a linear limit, TBR (S5) |
| F8 | The OBC design's pin tables (§8.4 and A.5.4) disagree with each other and with the firmware. For example, §8.4 has PA4–PA7 as status LEDs, where the firmware has the FRAM SPI. | `hardware/obc/OBC_DESIGN.md` | The drawings follow the firmware |
| F9 | SPI2 and its chip select are vestigial: the transceiver they served was replaced by the KISS TNC in CR-2026-005. | `software/rust/firmware/src/spi.rs` | Dotted, "no device" (B3, S3) |
| F10 | Two devices on I2C2 share address 0x48: the EPS controller (SMART-QSO-ICD-001 and the simulation) and the TMP117 (OBC_DESIGN). | `docs/interfaces/ICD_OBC_EPS.md` 4; `software/rust/sqso-ffi/src/hal_sim.rs` | Flagged in B3 and S3 |
| F11 | The FM transponder path has no control line. Nothing arbitrates the single VHF transmitter between TNC frames and transponder audio. | `hardware/rf/TRANSMITTER_DESIGN.md` 10.1 | A TBD "transponder audio path + TX arbiter" block (B4, S4) |
| F12 | The 4S BMS needs five cell taps, and harness H1 carries none. | `hardware/harness/HARNESS_DESIGN.md` 4.1 | A separate 5-pin balance connector (W1, S1) |
| F13 | While the Jetson is off, the OBC's USART1 TX would back-power the carrier through its I/O clamp diodes. The GNSS receiver behind SW7 has the same problem. | design gap | 74LVC1T45 translators with Ioff (S7); series resistors for GNSS (S8) |
| F14 | H5 and H6 use JST-SH, which is not in SQSO-HAR-001's connector list (§3.1). | `hardware/harness/HARNESS_DESIGN.md` | Kept for H6 and noted (W5); FTSH for ADCS (W6) |
| F15 | The contents of H2 (+5V, OBC_EN, EPS_INT) and H4 (+5V, RF_EN, RF_TEMP) do not match the flight software's signals. | `hardware/harness/HARNESS_DESIGN.md` 4.2, 4.4 | W3 and W4 replace them |
| F16 | No fault output reaches the OBC. That covers the load switches' fault pins, the eFuse's power-good and the BMS alert; only the INA226 ALERT (PB7) is wired. | firmware pin map | "no OBC pin (TBD)" on S1, S2, S7 |
| F17 | Several hardware documents still describe superseded designs. PDU_DESIGN and EPS_ELECTRICAL_DESIGN describe a 2S pack, and TRANSMITTER_DESIGN 4 lists the Si4463. | the documents named | Not drawn; the drawings follow the baseline |

Checked, and not a finding: the 7-pole LPF values in TRANSMITTER_DESIGN 7.3 (L 47/82/82/47 nH, C 33/47/33 pF) are not
the textbook 0.5 dB Chebyshev prototype. An ideal-part ABCD calculation still gives −0.1 dB at 145.9 MHz, −50 dB at
2f and −77 dB at 3f, which meets its §7.2 and the baseline's 3rd-harmonic target. Real parts need an engineering-model
measurement (S4).

## 5. Regenerating

```bash
# graphviz from the system (apt-get install graphviz), then:
pip install -r hardware/diagrams/requirements.txt   # wireviz 0.4.1, schemdraw 0.23
hardware/diagrams/build.sh                          # rewrites hardware/diagrams/svg/*.svg
```

`hardware/diagrams/build.sh` renders `block/*.dot` with dot, `wiring/*.yml` with WireViz, and `schematics/S*.py` with
schemdraw's SVG backend (no matplotlib). The schematics share `hardware/diagrams/schematics/sch.py` for blocks, flags
and the title. Two runs give byte-identical SVGs. Edit the source, run the script and commit both. Nothing checks in CI
that the SVGs are current.

## 6. Tools

The text-based tools were chosen so that a drawing can be reviewed as a diff: Graphviz for the block diagrams, WireViz
for the harnesses and schemdraw for the schematics. SKiDL (Python netlists for KiCad) would not install in the build
environment.

The schematics here are documentation-grade sheets, not a PCB capture. The next step, once parts are selected, is a KiCad
capture with ERC. These Claude Code skills and plugins can help with that. They were found on 2026-10-25, none is
installed in this repository, and none has been evaluated:

- [skidl-skills](https://www.claudepluginhub.com/plugins/nickkraakman-skidl-skills): SKiDL circuit and netlist
  generation for KiCad.
- [circuit-synth](https://claudskills.com/skills/circuit-synth/): Python-to-KiCad schematic generation.
- [eda-schematics](https://claudemarketplaces.com/skills/l3wi/claude-eda/eda-schematics): KiCad schematic editing, from
  the claude-eda plugin.
- [kicad-schematic](https://skillselion.com/skills/kenchangh/kicad-schematic): KiCad schematic creation.
- [kicad-sch-api](https://forum.kicad.info/t/kicad-sch-api-python-library-for-kicad-schematic-manipulation/65363): a
  Python library for reading and writing `.kicad_sch` files, which several of these skills build on.
- In the claude.ai plugin directory: BoardRepo (PCB design review and KiCad checks) and Diagrams (Kroki rendering of
  Graphviz, PlantUML and similar).

## 7. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-25 | Initial set: B1–B4, W1–W10, S1–S9; findings F1–F17 | SMART-QSO Team |
