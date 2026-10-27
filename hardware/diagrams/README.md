# System Diagrams: Block, Wiring and Schematic

**Document Number:** SQSO-HW-DIAG-001
**Revision:** D
**Date:** 2026-10-27
**Status:** Draft. Since Revision B the drawings follow the board build-to designs (`hardware/electrical/eps/EPS_BOARD_DESIGN.md`,
`hardware/electrical/obc/OBC_BOARD_DESIGN.md`, `hardware/electrical/rf/RF_BOARD_DESIGN.md`) and the master pin allocation
(`hardware/electrical/INTERCONNECT.md`, SQSO-ICX-001); none of these has been built

---

## 1. Purpose and Standing

This set draws the whole spacecraft at three levels: block diagrams (B1–B4), wiring diagrams for the harnesses (W1–W10)
and electrical schematics (S1–S9). Each drawing is generated from a text source in this directory, so a change shows up
in review as a diff.

The drawings record the design as the repository states it today. They do not select parts: the board designs do.
`docs/BASELINE_PARAMETERS.md` governs every value. Where a drawing disagrees with the baseline, the baseline wins and the
drawing is wrong. Pins come from the master allocation SQSO-ICX-001 §4, generated from the OBC schematic; where the
firmware (`software/rust/firmware/src/`) still differs, SQSO-ICX-001 §6 lists the change for the flight software.
Revision A drew the firmware's provisional pins and marked its own additions as proposed; the findings that came out of
that (§4) are now dispositioned against the board designs.

## 2. Contents

| ID | Drawing | Source | Main sources drawn from |
|----|---------|--------|-------------------------|
| B1 | System overview | `hardware/diagrams/block/B1_system_overview.dot` | BASELINE 2, 4–7 |
| B2 | Power distribution | `hardware/diagrams/block/B2_power_distribution.dot` | BASELINE 4–5; SQSO-EPS-BRD-001 §3–§9 |
| B3 | OBC data and control interfaces | `hardware/diagrams/block/B3_data_control.dot` | SQSO-ICX-001 §4 |
| B4 | RF chain | `hardware/diagrams/block/B4_rf_chain.dot` | SQSO-RF-BRD-001 §3–§8; BASELINE 7 |
| W1 | Battery pack to EPS | `hardware/diagrams/wiring/W1_battery_eps.yml` | SQSO-HAR-001 2.0; SQSO-EPS-BRD-001 §13.1 |
| W2 | Solar panels to EPS | `hardware/diagrams/wiring/W2_solar_eps.yml` | SQSO-HAR-001 2.0; `hardware/eps/SOLAR_ARRAY_DESIGN.md` |
| W3 | EPS to OBC | `hardware/diagrams/wiring/W3_eps_obc.yml` | SQSO-ICX-001 §3.1; SQSO-HAR-001 2.0 |
| W4 | OBC and EPS to RF board | `hardware/diagrams/wiring/W4_rf_board.yml` | SQSO-ICX-001 §3.2; SQSO-RF-BRD-001 §4; SQSO-HAR-001 2.0 |
| W5 | EPS and OBC to Jetson carrier | `hardware/diagrams/wiring/W5_jetson.yml` | SQSO-ICX-001 §3.3; SQSO-HAR-001 2.0 |
| W6 | ADCS | `hardware/diagrams/wiring/W6_adcs.yml` | SQSO-ICX-001 §3.4; SQSO-HAR-001 2.0 |
| W7 | Deployables and inhibits: burn and sense (W7A, W7B), AntS (W7C), deployment switches and RBF (W12A–W12D) | `hardware/diagrams/wiring/W7_deployables.yml` | SQSO-EPS-BRD-001 §6.2, §9, §13.1; SQSO-HAR-001 2.0 |
| W8 | GSE umbilical and charging: W8A, W11, access-port board A3 (Micro-D 21), GSE cable W8B | `hardware/diagrams/wiring/W8_gse_umbilical.yml` | SQSO-ICX-001 §3.6; SQSO-EPS-BRD-001 §6.5; SQSO-HAR-001 2.0 |
| W9 | GNSS receiver | `hardware/diagrams/wiring/W9_gnss.yml` | SQSO-ICX-001 §3.5; SQSO-HAR-001 2.0 |
| W10 | RF coaxial cables (W10) and the GNSS antenna cable (W10E) | `hardware/diagrams/wiring/W10_rf_coax.yml` | SQSO-RF-BRD-001 §4.3; SQSO-HAR-001 2.0 |
| S1 | EPS power path and inhibits | `hardware/diagrams/schematics/S1_eps_power_path.py` | BASELINE 4–5; SQSO-EPS-BRD-001 §5–§7 |
| S2 | EPS rails and load switches | `hardware/diagrams/schematics/S2_rails_load_switches.py` | SQSO-EPS-BRD-001 §8 |
| S3 | OBC core (STM32L4R5ZI) | `hardware/diagrams/schematics/S3_obc_core.py` | SQSO-OBC-BRD-001; SQSO-ICX-001 §4 |
| S4 | RF board | `hardware/diagrams/schematics/S4_rf_board.py` | SQSO-RF-BRD-001 |
| S5 | Burn-wire drivers and deployed-state sense | `hardware/diagrams/schematics/S5_burn_wire.py` | SQSO-EPS-BRD-001 §9; SQSO-ICX-001 §3.1 |
| S6 | ADCS on the OBC board | `hardware/diagrams/schematics/S6_adcs.py` | SQSO-OBC-BRD-001 §9–§10 |
| S7 | Jetson power gate and link | `hardware/diagrams/schematics/S7_jetson_power_link.py` | SQSO-OBC-BRD-001 §11; SQSO-EPS-BRD-001 §8.3 |
| S8 | GNSS receiver interface | `hardware/diagrams/schematics/S8_gnss.py` | BASELINE D6; SQSO-OBC-BRD-001 §12 |
| S9 | ADC front ends | `hardware/diagrams/schematics/S9_adc_front_ends.py` | SQSO-EPS-BRD-001 §11; SQSO-ICX-001 §4.3 |

The rendered drawings are in [`svg/`](svg/), one file per ID (for example [`svg/B1_system_overview.svg`](svg/B1_system_overview.svg)).

## 3. Conventions

**Block diagrams.** Red is power, blue is data, purple is RF, grey is audio or baseband, and dashed green is a control or
enable line. In B3, orange is a discrete input and brown an analog input. A dashed box is a part that has not been
selected.

**Wiring diagrams.** These are WireViz harness drawings. Each one gives connectors, pin labels, wire colours, gauges and
lengths. Connector families follow SQSO-HAR-001 2.0 §3.1: Micro-Fit 3.0 (right-angle, gold) for the higher currents,
Pico-Clasp (gold) elsewhere, Samtec TFM with SFSDT discrete-wire assemblies board to board, a Micro-D 21 at the access
port and MMCX for RF. Gauges follow the terminals and its §5.2 derating; the wire list `hardware/harness/wire_list.csv`
governs, and `hardware/harness/harness_calcs.py --wireviz` checks every cable's gauge and length against it (a cable
designator is a harness or route of the wire list, with an optional -AWG or _A/_B suffix). Pinouts follow SQSO-ICX-001
and the board designs; the receiver-side GNSS and AntS connectors await the vendors' interface documents.

**Schematics.** These are schemdraw sheets. A signal that leaves a sheet ends in a flag carrying its net name. The same
names are used in the harnesses and on the other sheets, so `RF_TX_EN` on S2 is the net of that name on S3 and W3.
On every sheet:

- a dashed block is a part that has not been selected, and its label says so;
- a value that has not been confirmed is labelled as unconfirmed;
- a pin or net marked `*` is proposed here and is not yet in the firmware (SQSO-ICX-001 §6 lists the changes).

## 4. Design Findings

These came up while drawing Revision A from the firmware and the earlier documents. None was entered in
`docs/funding_audit/FINDINGS_REGISTER.md` or the open-items register. Revision B gives each one its disposition against the
board build-to designs (SQSO-EPS-BRD-001 Version B, SQSO-OBC-BRD-001, SQSO-RF-BRD-001) and SQSO-ICX-001; "closed" means
the design answers it, not that hardware has shown it.

| ID | Finding (Revision A) | Disposition (Revision B) | Where |
|----|----------------------|--------------------------|-------|
| F1 | The VBATT divider was a 2S value (1:3; 5.6 V on PC2 at 16.8 V) | **Closed.** 1.00 MΩ / 200 kΩ on the EPS, pack / 6 (2.80 V at 16.8 V); firmware scale `Linear(6.0, 0.0)` (SQSO-ICX-001 §6 row 15) | SQSO-EPS-BRD-001 §11; S9, B2 |
| F2 | The GNSS receiver had no UART, enable or 1PPS | **Closed.** LPUART1 PG7/PG8 (TX moved to PB11 in SQSO-ICX-001 D, PG7 became I2C3_SCL), GNSS_EN PE5 on SW7, 1PPS PC6 (TIM3_CH1), GNSS_PG PG15, translators on 3V3_GNSS | SQSO-ICX-001 §3.5; W9, S8, B3 |
| F3 | No magnetorquer drive pins and no sun-sensor input | **Closed.** Three DRV8876-Q1 on TIM1 CH1–CH3 (PE9/PE11/PE13), PH PE8/PE10/PE12, MTQ_ARM PF9 (TIM15), hardware off-timer; six transimpedance amplifiers into an ADS7138-Q1 (I2C4, 0x10) | SQSO-OBC-BRD-001 §9.5, §10; W6, S6, B3 |
| F4 | One burn line and one sense line for two antennas, one of each for two wings | **Closed.** Four independent burn lines, one per wing resistor (BURN_W1A PB10, BURN_W1B PE6, BURN_W2A PB9, BURN_W2B PE7), each with its own 0.8 A driver; the AntS deploys by its own I2C controller; hinge switches of both wings in series on PB12 | SQSO-EPS-BRD-001 §9; SQSO-ICX-001 §3.1; W7, S5 |
| F5 | The third independent physical inhibit was not defined | **Closed.** DS3, a second high-side FET in VBAT+ that also holds the trackers' enables low; three deployment switches DS1–DS3 on three rails (BASELINE §4) | SQSO-EPS-BRD-001 §6.2; S1, B1, B2 |
| F6 | No documented regulator took a 4S input | **Closed.** LMR43620-Q1 bucks (42 V input) for 3V3_AO, 5V_PA and V_BURN | SQSO-EPS-BRD-001 §8.1; S2, B2 |
| F7 | A 5 Ω burn resistor on 14.4 V draws about 2.9 A | **Closed.** 7.0 V burn buck and a linear 0.8 A driver per resistor; one resistor at a time, 0.67 A at the pack (0.75 A with the magnetorquers, 1.0 A limit) | SQSO-EPS-BRD-001 §9; S5 |
| F8 | The OBC design's pin tables disagreed with each other and with the firmware | **Closed.** SQSO-ICX-001 §4 is the master allocation and governs; `hardware/obc/OBC_DESIGN.md` keeps its tables as the record of the earlier plan | SQSO-ICX-001; S3, B3 |
| F9 | SPI2 and its chip select were vestigial | **Closed.** SPI2 serves the BMI088 backup gyro (CS PD0, PD4) | SQSO-OBC-BRD-001 §9.3; S3, B3 |
| F10 | EPS controller and TMP117 shared 0x48 on I2C2 | **Closed.** The OBC board has no TMP117 (TMP235-Q1 analog on PB1); I2C2 carries the EPS controller 0x48 and the INA228s 0x40 and 0x44; the AntS (0x31, 0x32) moved to its own I2C3, PG7/PB4 (SQSO-ICX-001 D, SMA-B07) | SQSO-OBC-BRD-001 §2.2; SQSO-EPS-BRD-001 §11 |
| F11 | The FM transponder path had no control line or TX arbiter | **Closed.** XPDR_EN heartbeat (PE4) into a monostable, CTCSS decode, 120 s key-down timer, TMUX mute and select, hardware TX permit; readbacks TONE_OK PC9, XPDR_KEY PG12, PTT_EXCITER PC7 | SQSO-RF-BRD-001 §8; S4, B4 |
| F12 | The 4S BMS needed five cell taps; harness H1 carried none | **Closed.** J2 Pico-Clasp 6 (TAP0–TAP4, NTC2) in W1 | SQSO-EPS-BRD-001 §13.1; W1, S1 |
| F13 | USART1 TX (and the GNSS lines) would back-power an unpowered carrier or receiver | **Closed.** SN74LVC1T45-Q1 translators powered from the far side's rail (Ioff/VCC isolation) | SQSO-ICX-001 §5; S7, S8 |
| F14 | H5 and H6 used JST-SH, not in SQSO-HAR-001 §3.1 | **Closed.** W5B and W6 use Pico-Clasp | SQSO-ICX-001 §2; W5, W6 |
| F15 | H2 and H4 did not match the flight software's signals | **Closed.** W3 (40-way) and W4A/W4B replace them | SQSO-ICX-001 §3.1–§3.2; W3, W4 |
| F16 | No fault output reached the OBC except the INA226 ALERT | **Closed.** Every switch's fault line is read by the EPS controller and reported in its LOAD_SHED/fault register; EPS_ALERT_N (PB7) carries the INA228 alerts and the controller interrupt; Jetson and sensor-rail readbacks go to the OBC directly | SQSO-EPS-BRD-001 §8.3, §11; SQSO-ICX-001 §3.1 |
| F17 | Several hardware documents described superseded designs (2S pack in PDU_DESIGN and EPS_ELECTRICAL_DESIGN, Si4463 in TRANSMITTER_DESIGN) | **Closed for those three** (PDU_DESIGN 1.3, EPS_ELECTRICAL_DESIGN 1.3, TRANSMITTER_DESIGN 1.4). `hardware/harness/HARNESS_DESIGN.md` 1.2 §4 noted that SQSO-ICX-001 and W1–W10 supersede its H2–H6 tables; version 2.0 replaced those tables with the build-to harness design and its wire list | the documents named |

**Drawings behind SQSO-ICX-001 D (open, Revision D of this README).** The board FMEA changes of the interconnect's
Version D (2026-10-27) are drawn in W3, W6 and W9 but not yet in these sources: B3 (`block/B3_data_control.dot`)
still shows LPUART1_TX on PG7 and the AntS on I2C2 behind the TCA9517, and lists only the sensor switches U5–U7; S3
(`schematics/S3_obc_core.py`) shows LPUART1_TX on PG7 and no I2C3, MAG_EN/MAG_RB or second FRAM; S6
(`schematics/S6_adcs.py`) puts the RM3100 on U7 with the sun sensors (it has its own switch U35 and repeater U34);
S8 (`schematics/S8_gnss.py`) shows LPUART1_TX on PG7 (now PB11). Until they are redrawn, SQSO-ICX-001 §3–§4 and
SQSO-OBC-BRD-001 govern these pins.

Checked, and not a finding: the 7-pole LPF values in TRANSMITTER_DESIGN 7.3 (L 47/82/82/47 nH, C 33/47/33 pF) are not
the textbook 0.5 dB Chebyshev prototype. An ideal-part ABCD calculation still gives −0.1 dB at 145.9 MHz, −50 dB at
2f and −77 dB at 3f, which meets its §7.2 and the baseline's 3rd-harmonic target. The RF board replaces that filter with
a 7th-order elliptic low-pass (SQSO-RF-BRD-001 §7).

## 5. Regenerating

```bash
# graphviz from the system (apt-get install graphviz), then:
pip install -r hardware/diagrams/requirements.txt   # wireviz 0.4.1, schemdraw 0.23
hardware/diagrams/build.sh                          # rewrites hardware/diagrams/svg/*.svg
```

`hardware/diagrams/build.sh` renders `block/*.dot` with dot, `wiring/*.yml` with WireViz, and `schematics/S*.py` with
schemdraw's SVG backend (no matplotlib). The schematics share `hardware/diagrams/schematics/sch.py` for blocks, flags
and the title. Two runs give byte-identical SVGs. Edit the source, run the script and commit both. Nothing checks in CI
that the SVGs are current. A WireViz label must not contain `<` or `>` (WireViz passes them unescaped into Graphviz
HTML labels and `dot` fails): write "≤", "to" or a word instead.

## 6. Tools

The text-based tools were chosen so that a drawing can be reviewed as a diff: Graphviz for the block diagrams, WireViz
for the harnesses and schemdraw for the schematics. The board build-to designs capture their schematics in SKiDL
(`hardware/electrical/*/design.py`); these drawings stay documentation-grade views of them.

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
| B | 2026-10-27 | CDR 2026 consistency pass: B1–B4 redrawn from the board build-to designs and SQSO-ICX-001 (B2: the third inhibit DS3, three deployment switches DS1–DS3 on three rails, the EPS board's converters, switches and limits; B3: master pin allocation; B4: RF board chain); all SVGs rendered; findings F1–F17 dispositioned (§4); W1, W2, W7 labels without angle brackets; sources table | SMART-QSO Team |
| C | 2026-10-27 | Wiring drawings W1–W10 from the harness design SQSO-HAR-001 2.0: gauges from the terminal ranges, lengths from the routes, gold connectors, Samtec TFM/SFSDT for W3, W4A, W8A; W7 adds W12A–W12D, W8 adds W11 and the access-port board A3, W10 adds W10E; §2 sources and §3 conventions updated (§15 item D-1) | Harness design |
| D | 2026-10-27 | Documentation audit (post-CDR 2026): F2, F10 and F17 dispositions brought to SQSO-ICX-001 D and SQSO-HAR-001 2.0 (LPUART1_TX PB11, AntS on I2C3, harness tables replaced); §4 lists the drawings not yet redrawn for SQSO-ICX-001 D (B3, S3, S6, S8). No drawing changed | Documentation audit |
