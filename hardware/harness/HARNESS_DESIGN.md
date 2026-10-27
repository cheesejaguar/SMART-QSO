# SMART-QSO Harness Design

**Document Number:** SQSO-HAR-001
**Version:** 2.0
**Date:** 2026-10-27
**Status:** CDR build-to harness design (RID SE-19). Wire list, routes, fills, derating, drops and mass are computed
from the files in this directory; nothing has been built, crimped, routed on a structure or weighed.

**Governing documents**: `docs/BASELINE_PARAMETERS.md` Rev E (governs every value); the master pin allocation and
board-to-board signal lists [INTERCONNECT.md](../electrical/INTERCONNECT.md) (SQSO-ICX-001 B §3, which this document
wires); the board build-to designs [EPS_BOARD_DESIGN.md](../electrical/eps/EPS_BOARD_DESIGN.md) (SQSO-EPS-BRD-001 B),
[OBC_BOARD_DESIGN.md](../electrical/obc/OBC_BOARD_DESIGN.md) (SQSO-OBC-BRD-001 B),
[RF_BOARD_DESIGN.md](../electrical/rf/RF_BOARD_DESIGN.md) (SQSO-RF-BRD-001 C),
[CARRIER_BOARD_DESIGN.md](../electrical/jetson/CARRIER_BOARD_DESIGN.md) (SQSO-JET-BRD-001); the CAD
[MECHANICAL_DESIGN.md](../mechanical/MECHANICAL_DESIGN.md) (SQSO-MECH-CAD-001 B, §4.1 stack, §6.9 bundle routes,
`hardware/mechanical/model.py`); `docs/environment/MASS_BUDGET.md` (MEL rows 26, 31, 32);
`docs/compliance/MATERIALS_LIST.md`; `docs/manufacturing/EEE_PARTS_LIST.md` §2A.3 (finishes).

---

## 1. Result

| Item | Result | Section |
|------|--------|---------|
| Harnesses | W1–W12 and the coax W10 (four 1.37 mm cables) and W10E (GNSS, 1.13 mm): **206 conductors** (201 wires, 19.8 m; five coax, 1.1 m), every one in `hardware/harness/wire_list.csv` with its signal, both ends, gauge, type, twist group, currents, route and length | §4, §6 |
| Gauges | Set by the terminals: **28 AWG on every Pico-Clasp** (its crimp takes 28–32 AWG; the 24 and 26 AWG the interconnect and the WireViz sources gave W2, W4B, W6B, W6C, W7C and W9 cannot be crimped), 24 AWG on Micro-Fit (20–24 AWG), 22 AWG only for the pack leads, 30 AWG in the Samtec discrete-wire assemblies | §5.2 |
| Ampacity (EEE-INST-002 W1, bundle factor) | Worst normal case 60 % (W4B PA return at the SW3 limit); 81 % for W7C at the AntS switch's current limit (a fault, 8 ms per 0.28 s); pack leads 48 % at 2.0 A | §9.1 |
| Voltage drops | Pack 35 mV at 2.0 A, Jetson 43 mV at 1.4 A, PA 36 mV at 1 W, burn lines 44 mV (wing 1, 0.050 Ω loop) and 34 mV (wing 2), AntS 37 mV at 0.61 A: all within their limits | §9.2 |
| Band B (deck to EPS) over-fill | **Resolved by routing, not by more space**: the EPS-only harnesses (W1, W2, W7, W11, W12) terminate on connectors on the EPS bottom side and reach them through the open volume under the EPS, the skeleton deck's openings, inboard strips beside the battery end caps, hinge-side strips and shear-panel slots; of them only W7A's two burn pairs use a corner bundle, the (+X,−Y) one the interconnect kept for them. Band-B fill of the corner bundles: **(+X,+Y) 4.3, (−X,+Y) 5.2, (−X,−Y) 9.1, (+X,−Y) 3.1 mm²** (24, 29, 51 and 26 % of the allowance) | §8 |
| Highest fill anywhere | 65 % (hinge-side strip HS(+X)), 61 % (W4A on the shear panel); corner bundles 51 % at most | §8 |
| CAD check | Every one of the 58 harness zones checked by boolean intersection against every solid of the CAD: **no interference**; coax bands carry coax only; magnetometer zone carries W6E only | §7.3 |
| Mass | **84.5 g CBE** in all: 75.6 g in MEL row 32 (80.0 g CBE: −4.4 g), 6.6 g of ADCS sensor cables in row 26 (5.0 g with brackets: +1.6 g), 2.3 g of GNSS coax in row 31 | §11 |
| Changes needed elsewhere | 34 items for the EPS, OBC, RF, carrier, battery, mechanical, interconnect, materials and parts documents (connectors on the EPS bottom side, two EPS connector changes, gold finishes, Samtec TFM/SFSDT for the PVC ribbons, CAD zones and slots, a larger access-port envelope, isolated antenna mounts) | §15 |

The interconnect estimated 53 mm² of EPS-only harness against about 45 mm² left in band B (SQSO-ICX-001 B §2.1).
With the gauges the terminals force, the same harnesses need **41.7 mm²** against the **45.6 mm²** the four bundles
leave after the harnesses with an OBC end routed here: they would fit in total at 91 % of what is left, with nothing in
hand for the split between corners. Routing them inboard instead leaves the corner bundles at 51 % or less in every
band.

## 2. Standards, Rules and Files

### 2.1 Standards

| Subject | Standard or source | Applied as |
|---------|--------------------|------------|
| Crimping, harness build, lacing, inspection | NASA-STD-8739.4A; IPC/WHMA-A-620 Class 3 with the space addendum | Workmanship of every harness; crimp qualification V-HAR-2 |
| Solder terminations (pads, A3, burn boards, switches) | J-STD-001 with the space addendum, Sn63Pb37 | EEE_PARTS_LIST §2A.3 item 4 |
| Wire | SAE AS22759/33 (M22759/33): crosslinked ETFE, silver-coated high-strength copper alloy, 200 °C, 600 V, thin wall | Every discrete wire except the Samtec assemblies |
| Wire derating | NASA EEE-INST-002 section W1: single-wire currents for 200 °C-rated wire in vacuum (30/28/26/24/22/20 AWG: 1.3/1.8/2.5/3.3/5.0/7.5 A), bundle factor (29 − N)/28 for 1 < N ≤ 15 and 0.5 for N > 15 (the same expression as ESCC 3901/022; GSFC's 3.7 A for 20 AWG in a bundle of 15 or more is the 0.5 case) | N counted as all conductors in the busiest section the wire passes, including the open zones under the EPS (conservative); §5.2 |
| Connector derating | EEE-INST-002: contact current ≤ 50 % of the rated current | Pico-Clasp 1 A, Micro-Fit 3 A (22–24 AWG), Samtec 2 A |
| Fill | 60 % of the section filled with wire (SQSO-ICX-001 B §2.1) | Sum of wire cross-sections at maximum diameter |
| Finishes | EEE_PARTS_LIST §2A.3 item 5: no tin-plated connector contacts (SpaceX UG v11 Table 5-5 prohibits pure tin) | Gold-plated variants of every board header and harness contact |
| Materials | MATERIALS_LIST: ASTM E595 TML ≤ 1.0 %, CVCM ≤ 0.10 %; nylon tie wraps and PVC rejected | Lacing tape instead of tie wraps; no PVC ribbon (§5.3) |
| Grounding | Single-point ground through EPS mounting hole H1; no frame bond on the OBC (SQSO-OBC-BRD-001 B §21) | §12 |

### 2.2 Bend radius rule

Inside radius, static bends fixed by lacing and staking: single wires and
twisted pairs ≥ 3 × the wire's outside diameter at connector exits and ≥ 6 × elsewhere; flat-dressed groups (W3,
W4A, W8A, two layers) ≥ 3 × the group's thickness (4.0 mm); flexible coax ≥ 6 × its outside diameter (the NASA
workmanship value), 6.8 mm for W10E. Every bend is inspected against NASA-STD-8739.4A at the EM build (V-HAR-10).

### 2.3 Files

| File | Content |
|------|---------|
| `hardware/harness/wire_list.csv` | One row per conductor: harness, wire ID, signal, from (reference, pin), to (reference, pin), AWG, wire type, twist group, colour, normal and maximum current, route, power circuit and pole, minimum length, notes |
| `hardware/harness/routes.csv` | Every route as waypoints (x, y, z in the stack frame of MECHANICAL_DESIGN §4.1), each segment tagged with the zone it runs in |
| `hardware/harness/harness_calcs.py` | Lengths, zone and band fills, ampacity, contact currents, voltage drops, mass, the band-B comparison and the magnetometer field; `--cad` intersects every zone with the CAD solids (CadQuery; imports `hardware/mechanical/model.py` read-only); `--wireviz` checks the gauges and lengths of the WireViz sources against the wire list. `python3 hardware/harness/harness_calcs.py [--cad] [--wireviz]`; exit status 1 on any failed check |
| `hardware/harness/harness_results.json` | The script's results (lengths, fills, ampacity, drops, mass, CAD check) |
| `hardware/diagrams/wiring/W1_battery_eps.yml` … `W10_rf_coax.yml` | WireViz drawings (SQSO-HW-DIAG-001), rendered to `hardware/diagrams/svg/` by `hardware/diagrams/build.sh`; W7 carries W12, W8 carries W11 and the access-port board A3 |

Length rule: route length (sum of the axis-aligned waypoint moves) × 1.10 + 15 mm per end, rounded up to 5 mm; W1 and
W10 have stated minimums (thermal; RF loss budget). Each end's 15 mm is the strip, the dress and one re-termination.

## 3. Connectors

### 3.1 Connector families

| Family | Harness-side parts | Board side | Used on |
|--------|--------------------|-----------|---------|
| Pico-Clasp 1.0 mm | 501330-0N00 housing, 501334 crimp terminal, **gold** (Molex lists 0.38 and 0.76 µm gold versions; the board headers' tin suffixes change to the gold code) | 501331-xx vertical SMT header, gold | EPS J2, J4–J9, J12, J14–J17, J19–J21; OBC J3–J8, J10; RF J2; carrier J2; A2 J1 |
| Micro-Fit 3.0 | 43025-0N00 receptacle, 43030 crimp terminal, gold | **43045 right-angle** dual-row header, gold (43045-0800 is the 8-way right-angle header; a mated vertical Micro-Fit with its wire bend does not fit 12.5 mm under the EPS or under the RF board above the carrier) | EPS J1, J10, J11, J13, J18; carrier J1 |
| Samtec Tiger Eye 1.27 mm | **SFSDT** PTFE discrete-wire socket assembly, latched | **TFM** header, gold (TFM-1xx-01-S-D-RA right-angle on the OBC and RF boards; vertical TFM on the EPS bottom side) | W3 (EPS J3, OBC J1), W4A (OBC J2, RF J1), W8A (OBC J9) |
| MMCX | Molex 73414-0351 right-angle crimp plug, 1.37 mm cable (RF board C §4.3); the 1.13 mm version for W10E | Molex 73415-1473 jack; receiver and patch ports | W10, W10E |
| Micro-D | — (flight side is the board receptacle) | MIL-DTL-83513-class 21-way socket receptacle on A3, gold | Access port (W8A, W11) |
| Splice | SAE AS81824/1 crimp splice, permanently insulated, under heat shrink, staggered, tied down within 10 mm | — | W6C at the rod leads, W7A/W7B at the deployed-state sensor leads, W7C at the AntS pigtail |

### 3.2 Access-port board A3

**Access-port board A3** (harness part, at the −Y access window, CAD request M-3): FR-4, 30 × 12 × 1.6 mm, on the −Y
shear panel with two Ti M2 screws, carrying the Micro-D 21 socket receptacle through the window. W8A (debug, key
loading) and W11 (charging) are soldered to it, so the window carries one GSE connector. Micro-D pin map (the GSE
adapter wires to it): 1 VTREF, 2 SWDIO, 3 GND, 4 SWCLK, 5 WDT_DIS, 6 NRST, 7 USART3_TX, 12 USART3_RX, 13
UMBILICAL_SENSE, 14 GND; 9 UMB_VMON, 10–11 PACK_N, 19 UMB_SHUT, 20–21 UMB_CHG; 8 and 15–18 not connected. The charge
cluster (9–11, 19–21) is separated from the debug pins by unused pins 8 and 18; PACK_N is never joined to GND on A3.
The shell bonds to the frame only. The 2 × 5 2.54 mm bulkhead of SQSO-ICX-001 B §3.6 is not used: its shrouded header
(about 20 × 9 mm) is larger than the 18 × 6 mm window the CAD carries.

## 4. Harness List

| Harness | From | To | Conductors | Gauge, type | Length (m) | Route (zones, §7) |
|---------|------|----|-----------:|-------------|-----------:|-------------------|
| W1 | Battery sense-board pads (flying leads) | EPS J1 (Micro-Fit 2×4), J2 (Pico-Clasp 6) | 14 | 4 × 22, 4 × 24, 6 × 28 AWG | 0.120 | Deck opening DX_MXPY, service loop in zone Z1 under the EPS |
| W2PX, W2MX, W2PY, W2MY | Body-panel pads (vendor pigtail) | EPS J4, J5, J6, J7 (Pico-Clasp 5) | 4 × 5 | 28 AWG | 0.040–0.050 | Shear-panel slot at z −10, straight to the EPS bottom side |
| W2W1, W2W2 | Wing hinge-flex termination tab TB-W1, TB-W2 | EPS J8, J9 (Pico-Clasp 8) | 2 × 8 | 28 AWG | 0.075, 0.125 | Hinge-side strip HS(±X) |
| W3 | EPS J3 (TFM-120, bottom side) | OBC J1 (TFM-120 right-angle) | 40 | 30 AWG Samtec SFSDT | 0.065 | Flat in the +X wall gap at \|y\| < 17 |
| W4A | OBC J2 (TFM-108 RA) | RF J1 (TFM-108 RA) | 16 | 30 AWG SFSDT | 0.075 | Flat on the +X shear panel, y −28.5 … −38.5 |
| W4B | EPS J19 (Pico-Clasp 6) | RF J2 (Pico-Clasp 6) | 6 | 28 AWG, three twisted pairs | 0.145 | (−X,−Y) bundle B–E, over the RF shield corner |
| W5A | EPS J18 (Micro-Fit 2×2) | Carrier J1 (Micro-Fit 2×2 RA) | 4 | 24 AWG twisted quad | 0.105 | (−X,−Y) bundle B–D, carrier −Y strip |
| W5B | OBC J3 (Pico-Clasp 6) | Carrier J2 (Pico-Clasp 6) | 6 | 28 AWG, three pairs | 0.060 | (+X,+Y) bundle D–E |
| W6B | EPS J20 (Pico-Clasp 2) | OBC J6 (Pico-Clasp 2) | 2 | 28 AWG pair | 0.075 | (−X,+Y) bundle B–D |
| W6C | OBC J7 (Pico-Clasp 6) | Rod leads X, Y, Z (splices) | 6 | 28 AWG, three pairs | X 0.110, Y 0.205, Z 0.050 | −X centre path MXC; X: Z1; Y: crossing U_X, (+X,+Y) bundle B–A |
| W6D | OBC J8 (Pico-Clasp 14) | Six photodiodes | 12 | 28 AWG, six unshielded pairs | 0.095–0.245 | (−X,+Y) bundle trunk, branches under the EPS |
| W6E | OBC J10 (Pico-Clasp 6) | Magnetometer board A2 J1 | 6 | 28 AWG, three pairs | 0.090 | (+X,+Y) bundle D–E, step-over TE_PXPY |
| W7A, W7B | EPS J10, J11 (Micro-Fit 2×3) | Burn boards (wing 1, 2) and deployed-state sensors | 2 × 6 | 24 AWG pairs | burn 0.150, 0.090; sense 0.095, 0.155 | W7A burn: +X wall gap below the EPS, (+X,−Y) bundle B–A, corner strip; W7B burn: deck opening, inboard strip, corner strip; sense: hinge-side strips |
| W7C | EPS J12 (Pico-Clasp 6) | AntS pigtail (splices) | 6 | 28 AWG, three pairs | 0.155 | −X centre strip beside the battery, under the end cap, end-frame cut-out |
| W8A | OBC J9 (TFM-105 RA) | Access-port board A3 (solder) | 10 | 30 AWG SFSDT single-ended | 0.145 | (−X,−Y) bundle D–A, corner strip, −Y shear panel |
| W9A | OBC J5 (Pico-Clasp 6) | GNSS receiver | 6 | 28 AWG, three pairs | 0.145 | (+X,+Y) bundle D–B, under EPS J3 |
| W9B | EPS J21 (Pico-Clasp 2) | OBC J4 (Pico-Clasp 2) | 2 | 28 AWG pair | 0.085 | (+X,+Y) bundle B–D |
| W10 | RF J3–J6 (MMCX) | AntS ports | 4 coax | 1.37 mm micro-coax | 0.220 | −X coax bands (RF board C §4.3) |
| W10E | GNSS receiver RF port | GNSS patch | 1 coax | 1.13 mm micro-coax | 0.195 | (+X,−Y) bundle upper section B–E, over the RF shield |
| W11 | EPS J13 (Micro-Fit 2×3) | A3 (solder) | 6 | 24 AWG | 0.175 | Under the EPS, −Y deck slot, −Y wall strip |
| W12A, W12B, W12C | EPS J15, J16, J17 (Pico-Clasp 3) | Deployment switches DS1, DS2, DS3 | 2, 2, 3 | 28 AWG | 0.090, 0.100, 0.110 | Each on its own rail and its own route (§7.2) |
| W12D | EPS J14 (Pico-Clasp 4) | RBF switch | 4 | 28 AWG, two pairs | 0.085 | −Y wall strip, −Y deck slot |

W12 is the number this document gives the inhibit-switch and RBF harnesses that SQSO-EPS-BRD-001 §6.2 defines ("one
harness per switch, on its own rail"). The GSE cable W8B (not flown) mates A3 from outside.

## 5. Wire and Materials

### 5.1 Wire

| AWG | Stranding | OD max (mm) | Area (mm²) | Mass (g/m, max) | R at +60 °C (Ω/m) | Single wire (A) | In a bundle of > 15 (A) |
|----:|-----------|------------:|-----------:|----------------:|------------------:|----------------:|------------------------:|
| 30 | 19/42 | 0.66 | 0.342 | 0.98 | 0.404 | 1.3 | 0.65 |
| 28 | 19/40 | 0.74 | 0.430 | 1.35 | 0.254 | 1.8 | 0.90 |
| 24 | 19/36 | 0.99 | 0.770 | 2.98 | 0.100 | 3.3 | 1.65 |
| 22 | 19/34 | 1.14 | 1.021 | 4.32 | 0.063 | 5.0 | 2.50 |

M22759/33 finished diameters (nominal 0.027 / 0.037 / 0.043 in ± 0.002 in for 28 / 24 / 22 AWG) and maximum weights from
the specification tables; the maximum diameter is used for fill and for the Pico-Clasp check (0.74 mm against the
terminal's 0.80 mm insulation limit). Resistance from the strand area with 85 % IACS (the lower bound for high-strength
copper alloy), lay factor 1.03, at +60 °C. The 200 °C rating applies the EEE-INST-002 table without an insulation
correction. High-strength alloy is used at every gauge (28 and 30 AWG copper is fragile in vibration). Silver-coated
conductors: humidity-controlled storage against red plague (NASA-STD-8739.4A practice). The Samtec SFSDT assemblies'
PTFE wire is taken at 30 AWG with the /33-30 diameter and weight.

### 5.2 Wire derating and gauge rules

Derating (this section is the wire derating that the board designs and the interconnect cite as SQSO-HAR-001 §5.2):
NASA EEE-INST-002 section W1, single-wire currents for 200 °C-rated wire in vacuum, times the bundle factor
(29 − N)/28 for 1 < N ≤ 15 and 0.5 for N > 15, N counted as every conductor in the busiest section the wire passes
(§2.1); contacts at ≤ 50 % of their rating. It replaces version 1.2's table (50 % of a commercial rating: 28 AWG ≤
0.25 A, 26 AWG ≤ 0.5 A, 24 AWG ≤ 1 A, 22 AWG ≤ 2 A, 20 AWG ≤ 3 A), which several wires here exceed while meeting
EEE-INST-002 (the body strings at 0.33 A and the wing returns at 0.44 A on 28 AWG, for instance): 28 AWG is forced by
the Pico-Clasp terminal, and the EEE-INST-002 values are the space-wire derating.

Gauge rules forced by the terminals:

| Connector family | Wire range | Consequence |
|------------------|-----------|-------------|
| Molex Pico-Clasp 501330 housing, 501334 terminal | **28–32 AWG**, insulation ≤ 0.80 mm | Every Pico-Clasp wire is 28 AWG. W2 (was 26 AWG), W4B (24), W6B and W6C (26), W7C (26), W9A and W9B (26) change |
| Molex Micro-Fit 3.0 43025 receptacle, 43030 terminal | 20–24 AWG, insulation ≤ 1.85 mm | 24 AWG (22 for the pack leads). The thin-wall insulation (0.99–1.14 mm) must hold in the insulation grip: crimp qualification V-HAR-2, with a 0.15 mm PTFE build-up sleeve where it does not |
| Samtec SFSDT (discrete-wire assembly for TFM headers) | 30 AWG PTFE | W3, W4A, W8A |

No single wire fits both a Micro-Fit and a Pico-Clasp terminal, so **a harness has one family at both ends**: EPS J19
(W4B, Micro-Fit to the RF board's Pico-Clasp J2) becomes a Pico-Clasp 6 with the RF J2 pinout, and EPS J12 (W7C) a
Pico-Clasp 6 with two contacts per supply pole (one Pico-Clasp contact would carry the AntS's 0.61 A, above 50 % of its
1 A rating). Recovery proposed by SQSO-ICX-001 B §2.1 for the pack leads (22 AWG instead of 20) is adopted: 1.2 A per
wire worst case (2.0 A pack current with a 60/40 split) against 2.50 A derated, 35 mV drop.

### 5.3 Materials

| Item | Choice | Why |
|------|--------|-----|
| Ribbon for W3, W4A, W8A | **Not used**: Samtec FFSD is PVC-insulated, 30 AWG tin-plated copper, −40 … +105 °C | PVC fails the outgassing rule and the project rejects it (MATERIALS_LIST, cell wrapper row); tin-plated conductors and FTSH friction mating. SFSDT on TFM gives PTFE insulation and a latch |
| Ties | MIL-T-43435 lacing tape (Nomex or polyester, finish with E595 data), spot ties; Scotch-Weld 2216 tie-down dots on structure | MATERIALS_LIST rejects nylon 6/6 tie wraps |
| Sleeving | PTFE (pack-lead over-sleeve, edge protection at the deck openings and slots) | Low outgassing |
| Connector housings | Pico-Clasp and Micro-Fit housings: E595 data per part number, finished harnesses baked out before integration (V-HAR-11) | Moisture-bearing polyamides are TML-limited |
| Staking | Arathane 5750 bead across each mated wire-to-board connector after final mating | Removable; the carrier's coating material |

## 6. Wire List

The complete list is `hardware/harness/wire_list.csv` (206 rows). The power and special conductors:

| Harness | Pins (from → to) | Signal | AWG | Twist | I normal / max per wire (A) | Notes |
|---------|------------------|--------|----:|-------|----------------------------|-------|
| W1 | BATT P1, P2 → J1-1, 2 | BAT_TOP | 22 | quad 1+3, 2+4 | 0.70 / 1.20 | Upstream of F2: PTFE over-sleeve, no metal clamps |
| W1 | BATT N3, N4 → J1-3, 4 | BAT_BOT | 22 | quad | 0.70 / 1.20 | |
| W1 | NTC1, NTC1R → J1-5, 6 | NTC1, its own return (GND) | 24 | pair | < 1 mA | |
| W1 | HTR± → J1-7, 8 | Heater | 24 | pair | 0.079 / 0.170 | |
| W1 | TAP0–TAP4, NTC2 → J2-1 … 6 | Cell taps, NTC2 | 28 | bundle | 0.018 / 0.05 | |
| W2PX/MX/PY/MY | SA+, SA+, SA−, SA−, TEMP → J4–J7-1 … 5 | String, returns, panel NTC | 28 | 1+3, 2+4 (+5) | 0.275 / 0.33 | Back-wired strings (residual dipole) |
| W2W1/W2W2 | TB T1 … T8 → J8/J9-1 … 8 | Front ×2, back ×2, GND ×3, TEMP | 28 | (1, 2, 5), (3, 4, 6), (7, 8) | + 0.275 / 0.33; returns 0.367 / 0.44 | |
| W3 | J3-n → J1-n, n = 1 … 40 (SQSO-ICX-001 §3.1) | 3V3_OBC (1, 2), VBAT_MTQ (36, 39), GND (3, 4, 7, 28, 35, 40), 32 signals | 30 | flat | 3V3_OBC 0.088, VBAT_MTQ 0.20, returns ≤ 0.13 | Straight through |
| W4A | J2-n → J1-n, n = 1 … 16 (§3.2) | Signals and monitors | 30 | flat | < 5 mA | Straight through |
| W4B | J19-n → J2-n, n = 1 … 6 | 1 RADIO_3V3, 3, 4 PA_5V, 2, 5, 6 GND | 28 | 3+2, 4+5, 1+6 | PA 0.235 / 0.536; radio 0.089 / 0.141 | ≤ 5 mm untwisted at each end |
| W5A | J18-1 … 4 → carrier J1-1 … 4 | VBAT_JET ×2, GND ×2 | 24 | quad 1+3, 2+4 | 0.70 / 0.84 | 1.4 A design current |
| W5B | J3-n → carrier J2-n (§3.3) | GND, RXD, TXD, JET_3V3, PT_NTC, PT_RTN | 28 | (1, 2), (3, 4), (5, 6) | < 5 mA | |
| W6B | J20-1, 2 → J6-1, 2 | 3V3_ADCS, GND | 28 | pair | 0.049 / 0.339 | |
| W6C | J7-1 … 6 → rod X, Y, Z (+, −) | MTQ_X/Y/Z_P/N | 28 | three pairs | 0.087 / 0.126 | Splices at the rod leads |
| W6D | J8-1 … 12 → photodiodes K, A | CSS_PX … MZ | 28 | six pairs, no shield | 0.004 / 0.005 | J8-13, 14 not used |
| W6E | J10-n → A2 J1-n (§3.4) | 3V3_SNS, GND, SDA, SCL, DRDY, GND | 28 | (1, 2), (3, 6), (4, 5) | < 2 mA | |
| W7A/W7B | J10/J11-1, 2 / 3, 4 → burn resistor A / B | V_BURN, BRET | 24 | pair per resistor | 0.88 for 12 s | One resistor at a time |
| W7A/W7B | J10/J11-5, 6 → sensor leads | Hinge sense (both wings in series) | 24 | pair | µA | |
| W7C | J12-1, 2 / 3, 4 / 5, 6 → AntS | 3V3_ANTS, GND, SDA, SCL | 28 | 1+3, 2+4, 5+6 | 0.305 / 0.733 | Peak 0.61 A, SW8 limit 1.47 A |
| W8A | J9-1 … 10 → A3 P1 … 7, 12, 13, 14 | SWD, NRST, console, sense, WDT_DIS | 30 | flat | < 10 mA | §3.2 pin map |
| W9A | J5-n → receiver (§3.5) | 3V3_GNSS, GND, RXD, TXD, 1PPS, GND | 28 | (1, 2), (3, 6), (4, 5) | 0.10 / 0.232 | Receiver contacts must take 28 AWG |
| W9B | J21-1, 2 → J4-1, 2 | 3V3_GNSS, GND | 28 | pair | 0.10 / 0.232 | |
| W11 | J13-1, 2 / 3, 4 / 5 / 6 → A3 P20, 21 / 10, 11 / 9 / 19 | UMB_CHG, PACK_N, UMB_VMON, UMB_SHUT | 24 | 1+3, 2+4 | 0.25 / 0.50 | |
| W12A–C | J15/J16-1, 2; J17-1 … 3 → DS1, DS2 (C, NC); DS3 (C, NC, NO) | DS1_C/S1N; DS2_C/S2N; GND/S3N/TRK_EN | 28 | pair / triple | µA (1.7 mA into a frame short) | Pin 3 of J15, J16 not used |
| W12D | J14-1 … 4 → RBF A1, A2, B1, B2 | S2N, S2R, TRK_EN, GND | 28 | two pairs | µA | |
| W10 | RF J3, J4 → 2 m ±X; J5, J6 → 70 cm ±Y | RF | coax 1.37 | — | 1 W | Pairs matched within 2 mm |
| W10E | Receiver RF → patch | GNSS L1, antenna bias | coax 1.13 | — | bias | |

Colours (WireViz codes, also in the CSV): RD supply, BK return, OG PA supply and VBAT_MTQ, VT burn supply, WH signal
or sense, others per the drawings. Each wire is identified by a printed sleeve with its wire ID at both ends.

## 7. Routing

### 7.1 Height bands and zones

Bands (z, mm; SQSO-ICX-001 B §2.1): A −46.9 … −26 (beside the battery), B −26 … −1 (deck to EPS), C −1 … +6.6 (EPS to
OBC), D +6.6 … +18 (OBC to carrier), E +18 … +48.4 (carrier to the +Z end frame).

| Zone | Box x / y / z (mm) | Use |
|------|--------------------|-----|
| PXPY, MXPY, MXMY | x ±(45.4 … 48.4), y ±(24.0 … 33.9), full height | The corner bundles of MECHANICAL_DESIGN §6.9 |
| PXMY_LO / PXMY_UP | +X; y −33.9 … −24.0 below z −26; y −27.0 … −20.5 above | (+X,−Y) bundle, narrowed beside W4A |
| MXC | −X wall gap, y +7.0 … +19.5, z −16 … +18 | Between the Z rod (\|y\| ≤ 5.25) and the 70 cm coax band: W6C leaves OBC J7 (y +12 … +22) without crossing the band |
| PXC_B | +X wall gap, \|y\| < 20, z −26 … −9 | Open below the EPS (the Y rod ends at z −27.65): W7A's burn pairs from J10 to the (+X,−Y) bundle |
| PXMY2 | +X wall gap, y −45.5 … −39.5, z −12 … +44.5 | DS3 leads beside W4A |
| HS_PX, HS_MX | ±X wall gap, y +34.5 … +39.5, z −42 … −5 | Hinge-side strips: wing harnesses, sensor pairs, DS1 |
| S_MXMY, S_MXC | x −45 … −36 beside the −X battery end cap, z −34 … −17 (S_MXC to −45) | Inboard strips below the deck |
| CS_PXMY, CS_MXMY | x ±(43.2 … 46.4), y −46.8 … −34, z −44.5 … −34 | Corner strips to the burn boards, DS2, W8A |
| UB_MXC, EF_MXC | Under the −X end cap (z −46.6 … −45.1), end-frame cut-out at x −35 … −28 | W7C to the AntS |
| U_MXPY, U_MXMY(2), U_PXPY, U_PXMY(2), U_PY, U_PYN | Under the EPS, z −15.0 … −4.05, clear of the GNSS receiver, X rod, Z rod, columns and centre supports | EPS connector zones Z1–Z5 and their wire dress |
| U_X | x −24.5 … +31, y +25 … +29.5, z −14 … −6 | Crossing between the (0, +21) centre support (1.1 mm) and the X rod (1.25 mm) |
| U_C, U_J3U | x −35.5 … −21, \|y\| < 7; under EPS J3 at z −15 … −12.5 | Crossings past the Z rod and under J3 |
| PG_PX, PG_MX, PG_PY, PG_MY | At slots (+48.5, −30), (−48.5, −12), (−30, +48.5), (−30, −48.5), z −10 | Body-panel pigtails (W2 and the W6D pair) |
| DX_*, DS_MY_* | In the deck's openings (inside its 3 mm rim) and the −Y slot (y −44.6 … −42.3) | Deck pass-throughs |
| YW_MY, YB_MY, AP_A3 | −Y wall strip; outboard of the (−40, −43) column; A3 envelope x −35 … −5, z −46.8 … −38 | W11, W12D, W8A, A3 |
| TE_* | Over the RF board under the +Z end frame, z +43.6 … +48.2 | Step-overs to RF J2, A2, the GNSS patch, the +Z sun sensor |
| CAR_MXMY, CARP | Carrier −Y strip (y −40 … −33.2, outside the SO-DIMM), carrier +X edge | W5A, W5B ends |
| W3Z, W4AZ | +X wall gap \|y\| < 17, z −8 … +12; +X shear panel y −38.5 … −28.5 | W3, W4A |
| COAX_P, COAX_M (and under-battery, end-frame parts) | −X, \|y\| 20 … 24 | W10 only (checked) |

### 7.2 Routes

- **W1** leaves the sense board's top face at (−30, +20), passes the deck opening and is coiled as a 0.12 m service
  loop in zone Z1 to J1/J2 on the EPS bottom side. It touches no bundle. Tied to the sense board within 10 mm of the
  pads and to the EPS tie points only; one spot tie to the deck strap over a PTFE sleeve; no metal clamp anywhere
  (thermal isolation of the pack).
- **Body panels**: each pigtail enters through a grommeted slot in its shear panel at z −10, below the EPS, and plugs
  directly into its EPS bottom-side connector; the panel's sun-sensor pair (W6D) uses the same slot.
- **Wings**: the hinge flex (mechanical design, MEL rows 10/11) ends on tab TB-W1/TB-W2 on the shear panel at
  (±48, +37, −37 … −31), inside the hinge-side strip; W2W1/W2W2 go up the strip to the EPS bottom side.
- **W7A burn pairs** leave J10 (zone Z3) into the +X wall gap below the EPS, run along it at \|y\| < 20 (1.65 mm above
  the Y rod, 4 mm below W3) and down the (+X,−Y) bundle (bands B, A) and the corner strip to the wing-1 burn board at
  the −Y free corner; **W7B burn pairs** drop from J11 (zone Z2) through a deck opening into the inboard strip beside
  the −X battery end cap and along the corner strip to the wing-2 burn board. The **sense pairs** go down the
  hinge-side strips to the deployed-state sensors, whose leads come through the body-panel notch beside the hinge
  flex.
- **W7C** runs under the EPS past the Z rod, down the −X centre strip beside the battery, under the −X end cap (one
  flat layer, 0.6 mm clear) and through the −Z end-frame cut-out to the AntS pigtail at (−31, −16).
- **W11** runs under the EPS, through the −Y deck slot and down the −Y wall strip to A3; **W8A** down the (−X,−Y)
  bundle, the corner strip, and flat along the −Y shear panel outboard of the (−40, −43) column to A3.
- **W12A** (DS1, rail +X+Y, −Z) leaves the switch beside the hinge doubler and goes up the +X hinge-side strip;
  **W12B** (DS2, rail −X−Y, −Z) up the (−X,−Y) corner strip and inboard strip; **W12C** (DS3, rail +X−Y, +Z) down the
  +X wall gap at y −39.5 … −45.5 beside W4A; **W12D** (RBF) along the −Y wall strip. The three inhibit harnesses share
  no zone with each other (SQSO-EPS-BRD-001 §6.3 independence).
- **W6C** leaves OBC J7 into the −X centre path MXC. The **Z-rod pair** is spliced in that gap at (y +10, z +12) to the
  rod's lead, which leaves the rod's top end (z +3) over its outer face at \|y\| ≈ 4.5. The **X-rod pair** goes under the
  EPS to the rod's −X end at (−23.5, +36, −10). The **Y-rod pair** goes under the EPS through the crossing U_X
  (y +25 … +29.5) to the +X side and down the (+X,+Y) bundle to z −30, spliced to the lead from the rod's +Y end at
  (+42, +23.5, −32.9), 0.5 mm inboard of the bundle's face: 0.205 m, 13 mV. The route under the battery (2 mm air gap,
  already used by the W10 coax) and along the deck rim were rejected (no room beside the coax; the rim is solid).
- **W6D** trunk down the (−X,+Y) bundle to z −8, branching under the EPS to the four panel slots; PZ up the same bundle
  to the +Z end frame at (−36, +36), MZ down to the termination at (−45, +37, −43.5).
- **W6E** up the (+X,+Y) bundle and inboard over the RF board's 2 mm parts to A2 J1 at its +X edge; its service loop is
  stowed in the bundle below z +20.
- **W9A** down the (+X,+Y) bundle and under the EPS, beneath J3, to the receiver's +X edge; **W10E** from the receiver
  up the (+X,−Y) bundle's upper section and over the RF shield (z +43.6 … +47) along y −22 and x +4 to the patch.
- **W3, W4A** flat (two layers) in the +X wall gap; the right-angle TFM headers send the cable toward +X, so W4A no
  longer bends over RF J1.

### 7.3 Clearances (CAD check, `harness_calcs.py --cad`)

Every zone box intersected with every solid of `hardware/mechanical/model.py` (stowed, ghost allocations excluded):
**no interference in any of the 58 zones**. Nearest solids: the corner bundles 0.1 mm to the shear panels and to the
W1/W2 burn boards; PXMY_UP 0.5 mm from the +X spreader flange (y −20); MXC 1.75 mm from the Z rod, 0.5 mm from the coax
band; PXC_B 1.65 mm above the Y rod; the bundles at \|y\| ≥ 24 are 0.5 mm from the Y rod's ends; U_X 1.1 mm to the
centre support and 1.25 mm to the X rod; U_C 0.75 mm to the Z rod; the hinge-side strips stop on the hinge doublers
(z −42) and LI_DS1 passes them 0.1 mm outboard; CS_MXMY 0.25 mm from DS2, 0.3 mm from the column spacer; TE_PXPY ends
0.28 mm from A2 (W6E's own end), TE_PXMY2 0.39 mm from the patch adapter (W10E's end); the zones under the EPS stop at
the EPS's bottom-side envelope and 0.5 mm above the deck. The coax bands carry W10 only. Inside the 20 mm zone round A2
only W6E runs (the (+X,+Y) bundle in band E holds W6E and the bottom 6 mm of W5B); the next harnesses are W10E (36 mm,
coax), W4B (37 mm) and the PZ pair (47 mm). AP_A3 replaces the CAD's access-port part (request M-3).

## 8. Bundle Fill and the Band-B Resolution

### 8.1 Corner bundles (mm², allowance 60 % of 3.0 × 9.9 mm = 17.8; (+X,−Y) above z −26: 11.7)

| Bundle | A | B | C | D | E | Contents |
|--------|---|---|---|---|---|----------|
| (+X,+Y) | 0.9 (5 %) | 4.3 (24 %) | 3.4 (19 %) | 8.6 (48 %) | 5.2 (29 %) | W5B, W6E (D, E); W9A, W9B (D, C, B); W6C Y pair (B, A) |
| (−X,+Y) | 0.9 (5 %) | 5.2 (29 %) | 5.2 (29 %) | 6.0 (34 %) | 0.9 (5 %) | W6B (D, C, B); W6D trunk (D, C, B), MZ (A), PZ (E) |
| (−X,−Y) | 3.4 (19 %) | 9.1 (51 %) | 9.1 (51 %) | 9.1 (51 %) | 5.7 (32 %) | W4B (B–E), W5A (B–D), W8A (D–A) |
| (+X,−Y) | 3.1 (17 %) | 3.1 (26 %) | 1.0 (9 %) | 1.0 (9 %) | 1.0 (9 %) | W7A burn pairs (B, A); W10E (B–E) |

### 8.2 Other zones (fill against 60 % of the zone's section)

HS_PX 5.8 / 9.0 mm² (65 %: W2W1, W7A sense, W12A); HS_MX 5.0 / 9.0 (55 %); W4AZ 5.5 / 9.0 (61 %); YB_MY 3.4 / 6.5
(53 %); UB_MXC 2.6 / 6.3 (41 %); CS_MXMY 7.4 / 20.2 (37 %); PG_* 3.0 / 9.6 (31 %); U_X 5.2 / 21.6 (24 %); W3Z 13.7 /
61.2 (22 %); every zone under the EPS ≤ 18 %; full table in the script's output. The coax's own passages under the
battery end cap and through the end frame are 88 % full (two 1.37 mm cables in 1.4 mm of height): the RF design's,
unchanged.

### 8.3 Resolution

1. **Gauges** (§5.2): 28 AWG on the Pico-Clasps, 24 AWG on the Micro-Fits, 22 AWG for the pack leads. The EPS-only
   harnesses' cross-section falls from the interconnect's 53 mm² to 41.7 mm².
2. **W6D without the overall shield** (proposed by SQSO-ICX-001 B §2.1): six unshielded twisted 28 AWG pairs (5.2 mm²
   instead of 9.6). EMC justification in §12.
3. **Routing** (the change that gives margin): every EPS-only connector moves to the EPS bottom side, in five zones
   Z1–Z5 where 12.5 mm of open height lies between the EPS and the deck (§10.1), and every EPS-only harness but W7A's
   burn pairs reaches it without a corner bundle: through the deck's openings, the inboard strips beside the −X battery
   end cap (x −45 … −36, which the corner-bundle model did not use), the hinge-side strips at y +34.5 … +39.5,
   shear-panel slots at z −10 and the −Y wall strip. The four corner bundles then carry the harnesses with an OBC end, the Y-rod pair, W10E and
   W7A's two burn pairs: 51 % at most in any band.
4. **No space is taken from the stack levels or asked of the bundles.** The CAD has to model the zones it now
   carries (requests M-1 to M-7 in §15): the zones under the EPS and the EPS bottom-side connector envelope, the four
   shear-panel slots and the termination tabs; the one envelope that grows is the access port's (M-3: 30 × 7 × 8.8 mm
   instead of 18 × 7 × 6 mm), for one GSE connector instead of two.

The (+X,−Y) section above z −26 (3 × 6.5 mm) carries W7A's burn pairs (the wing-1 hold-down is on this side, as the
interconnect allocated) and W10E: 3.1 of 11.7 mm²; nothing moves to the (+X,+Y) bundle.

## 9. Ampacity and Voltage Drop

### 9.1 Ampacity (worst wire of each power harness; N = conductors in the busiest section passed)

| Harness | Wire | I max (A) | N | Derated (A) | Ratio | Contact (normal current / 50 % rating) |
|---------|------|----------:|--:|------------:|------:|---------------------------------------:|
| W1 | BAT_TOP 22 AWG | 1.20 | 43 | 2.50 | 48 % | 47 % (Micro-Fit) |
| W2 body | S_face 28 AWG | 0.33 | 43 | 0.90 | 37 % | 55 % |
| W2 wings | GND return 28 AWG | 0.44 | 43 | 0.90 | 49 % | 73 % |
| W3 | VBAT_MTQ 30 AWG | 0.20 | 40 | 0.65 | 31 % | 4 % |
| W4B | PA return 28 AWG (SW3 limit) | 0.536 | 35 | 0.90 | 60 % | 47 % |
| W5A | VBAT_JET 24 AWG | 0.84 | 35 | 1.65 | 51 % | 47 % |
| W6B | 3V3_ADCS 28 AWG (SW4 trip) | 0.339 | 43 | 0.90 | 38 % | 10 % |
| W7A | V_BURN 24 AWG | 0.88 | 28 | 1.65 | 53 % | 59 % |
| W7B | V_BURN 24 AWG | 0.88 | 16 | 1.65 | 53 % | 59 % |
| W7C | 3V3_ANTS 28 AWG (SW8 limit) | 0.733 | 35 | 0.90 | 81 % | 61 % |
| W9A, W9B | 3V3_GNSS 28 AWG (SW7 trip) | 0.232 | 28 | 0.90 | 26 % | 20 % |
| W11 | UMB_CHG 24 AWG (F5 1 A) | 0.50 | 35 | 1.65 | 30 % | 17 % |

Maximum currents are the switch limits or trips (fault cases, limited in microseconds and removed after 8 ms by the
TPS2553 auto-retry) except W1 (2.0 A, four burn drivers on, with a 60/40 split between the paired wires) and W5A
(1.4 A design current, 20 % split allowance). W1 is upstream of the EPS fuse F2: a short in W1 is not protected by any
fuse, and the control is construction (double insulation, routing away from edges, no metal clamps, hipot V-HAR-1).

### 9.2 Voltage drop (design current, conductor at +60 °C, Pico-Clasp 20 mΩ, Micro-Fit 10 mΩ, Samtec 20 mΩ per contact)

| Circuit | Harness | Current (A) | Loop R (mΩ) | Drop (mV) | Limit (mV) |
|---------|---------|------------:|------------:|----------:|-----------:|
| Pack | W1 | 2.00 | 17.6 | 35.2 | 100 |
| Jetson VBAT_JET | W5A | 1.40 | 30.5 | 42.8 | 100 |
| PA_5V at 1 W | W4B | 0.469 | 76.8 | 36.0 | 100 (2 %) |
| RADIO_3V3 keyed | W4B | 0.089 | 153.6 | 13.7 | 66 (2 %) |
| Burn W1A / W1B / W2A / W2B | W7A, W7B | 0.88 | 50.1 / 50.1 / 38.1 / 38.1 | 44.1 / 44.1 / 33.5 / 33.5 | 88 (0.10 Ω loop, 1.4 % of the driver's 6.9 Ω compliance) |
| AntS 3V3_ANTS | W7C | 0.61 | 61.3 | 37.4 | 66 (2 %) |
| GNSS 3.3 V (W9B then W9A) | W9B + W9A | 0.10 | 276.7 | 27.7 | 66 |
| Strings (body / wings) | W2 | 0.55 / 1.10 | 30–33 / 23–30 | 17–18 / 25–33 | 48 (1 % of Vmp) |
| Umbilical charge | W11 | 0.50 | 27.6 | 13.8 | 100 |
| Magnetorquer rods | W6C | 0.087 | 69–148 | 6–13 | 100 |
| VBAT_MTQ, 3V3_OBC | W3 | 0.104, 0.040 | 55.2 | 5.7, 2.2 | 100, 33 |
| Heater | W1 | 0.079 | 44.1 | 3.5 | 100 |

## 10. Connectors, Mated Heights, Strain Relief, Tie-Downs and Bends

### 10.1 EPS bottom-side connector zones (request E-1 to the EPS designer)

The EPS top side has 5 mm under the OBC (0.4 mm left above the 5 mm envelope): no mated wire-to-board connector with
its wire bend fits there, so every EPS harness connector is on the bottom side, where 12.5 mm lies between the EPS
substrate (z −3.0) and the deck (z −15.5), outside the GNSS receiver (3.5 mm) and the X rod (2.0 mm).

| Zone | Area (x, y mm) | Connectors | Mated footprint (mm², allocation) |
|------|----------------|------------|-----------------------------------|
| Z1 | −44 … −24.5, +7 … +40 | J1 (W1, Micro-Fit 2×4 RA), J2 (W1), J6 (W2PY), J20 (W6B); the PACK-domain corner | 444 of 644 (69 %) |
| Z2 | −44 … −21, −40 … −7 and −36.5 … −21, −46 … −40 | J5 (W2MX), J7 (W2MY), J11 (W7B, Micro-Fit 2×3 RA), J16 (W12B), J18 (W5A, Micro-Fit 2×2 RA), J19 (W4B) | 622 of 852 (73 %) |
| Z3 | +24.5 … +44, +6 … +40 (less J3) | J8 (W2W1), J10 (W7A, Micro-Fit 2×3 RA), J15 (W12A), J21 (W9B, +X edge, +Y side) | 375 of 558 (67 %) |
| Z4 | +31 … +44, −40 … −16 and +31 … +36.5, −46 … −40 | J4 (W2PX), J17 (W12C); J14 (W12D) at the −Y edge | 94 of 345 (27 %) |
| Z5 | −20 … +24, +6 … +17.5 (and −20 … −3.5 to +25) | J9 (W2W2), J12 (W7C), J13 (W11, Micro-Fit 2×3 RA) | 366 of 630 (58 %) |
| +X edge | +33 … +45, \|y\| < 15.5 | J3 (W3, TFM-120 vertical, SFSDT) | dedicated |

Footprint allocations (to confirm on the Molex and Samtec drawings, V-HAR-3): Micro-Fit 2×N right-angle with plug and
wire bend (3.0 N + 3.0) × 23 mm, ≤ 9.0 mm below the substrate (z −12.0, 3.5 mm above the deck); Pico-Clasp 1×N
vertical (1.0 N + 3.0) × 4.5 mm, ≤ 8.0 mm including the 3 × OD wire bend; computed by the script against each zone's
plan area (limit 80 %). J10's plug points toward +X, so W7A's burn pairs leave straight into the +X wall gap.

### 10.2 Mated heights and strain relief per end

| End | Allocation (mated, with the wire exit) | Strain relief and retention |
|-----|----------------------------------------|-----------------------------|
| Pico-Clasp, OBC top side (J3–J8, J10) | 8.0 mm under the 2.0 mm carrier (9.0 mm free; SQSO-OBC-BRD-001 §21) | Tie point within 15 mm of the housing; Arathane 5750 bead across housing and header after final mating |
| Pico-Clasp, EPS bottom side | 8.0 mm (z −3.0 … −11.0) | As above |
| Pico-Clasp RF J2 | 8.0 mm (RF board C §4.3), over the shield corner | As above; ≤ 5 mm untwisted at the end (magnetics, §12) |
| Pico-Clasp carrier J2, A2 J1 | ≤ 7.7 mm (carrier §18, under the spreader edge); A2 J1 entered from +X under the board | As above |
| Micro-Fit right-angle (EPS J1, J10, J11, J13, J18; carrier J1) | 9.0 mm high, plug and bend 23 mm along the board; carrier J1 top at z +28.6, 4.7 mm below the RF board's bottom envelope | Positive latch; tie point within 15 mm |
| Samtec TFM right-angle + SFSDT (OBC J1, J2, J9; RF J1) | ≤ 7.0 mm high (OBC: 8.0 allocation; RF J1: 10.0); **plug's back ≤ 2.0 mm inside the board edge** (x ≤ ±43.1) so the flat cable bends into the 3 mm wall gap at 4.0 mm radius | Latched; first tie point on the shear panel within 15 mm of the bend |
| Samtec TFM vertical + SFSDT (EPS J3) | ≤ 9.0 mm below the EPS | Latched |
| MMCX (W10, W10E) | 6.0 mm (RF board C §4.3) | Snap-on; cable tied within 15 mm of the plug (rotation) |
| Micro-D 21 on A3 | Envelope x −35 … −5, z −46.8 … −38 (8.8 mm) | Board-mount jackscrews; A3 by two Ti M2 screws; W8A, W11 staked at A3 |
| Solder ends (pads, TB, burn boards, switches, A3) | — | Wire staked within 10 mm; service loop of one re-termination |
| Splices (rods, sensors, AntS) | 1.6 × 10 mm, staggered | Tied down on both sides within 10 mm |

### 10.3 Tie-down points

Lacing-tape spot ties at ≤ 30 mm along every run, on Scotch-Weld 2216 tie-down dots on the shear panels, end frames,
deck straps and the battery clamp (isolated side only); none on the stack columns or between boards (STR rules); one
at each zone change, within 15 mm of every connector, at each side of every splice and slot; slots and deck openings
carry PTFE edge protection. 104 tie points by the script's count (one per 30 mm of every occupied zone and band).
Harness and ties stay inside the frame's inner envelope (no harness in the 6.5 mm protrusion envelope or in any wing
or tape sweep) and leave the vent paths between stack levels open (the zones under the EPS are open volumes).

### 10.4 Bends

| Location | Required inside radius | Available | Status |
|----------|------------------------|-----------|--------|
| W4A, W3, W8A at the right-angle TFM (flat, 1.32 mm thick) | 4.0 mm | Plug back at x ≤ 43.1, gap to x 48.4: 4.0 mm + 1.3 mm | Meets with the placement rule of §10.2 |
| Pico-Clasp vertical exits (28 AWG) | 2.2 mm | Inside the 8.0 mm allocation | Meets |
| W1 pack leads (22 AWG) at J1 | 3.4 mm | Loop in Z1 | Meets |
| W5A at carrier J1 (24 AWG) | 3.0 mm | Right-angle plug, horizontal exit | Meets |
| W10E 1.13 mm coax at the +Z step-over | 6.8 mm | Bundle top at z +38.6 to the step-over at +45.4 | Meets |
| W10 1.37 mm coax over the RF board's −X edge (RF board C §4.3) | 8.2 mm by the 6 × OD rule | 7 mm (5.1 × OD) | **Below the rule**: needs the cable maker's minimum static radius ≤ 7 mm, or acceptance by the RF and mechanical designers (§15 item R-3, V-HAR-10) |

## 11. Mass

| Item | Mass (g CBE) |
|------|-------------:|
| Wire, M22759/33 and SFSDT (201 conductors, 19.8 m) | 31.8 |
| Coax: W10 (4 × 0.22 m) 3.5 g, W10E 0.5 g | 4.1 |
| Harness-side connector halves and splices (Pico-Clasp, Micro-Fit, SFSDT; ten MMCX plugs 9.0 g) | 27.6 |
| Access-port board A3 with the Micro-D 21 | 6.0 |
| Termination tabs, pack-lead over-sleeve, identification sleeves | 4.8 |
| Lacing and tie-down dots (104 points) | 10.4 |
| **Total** | **84.5** |

Against the MEL (`docs/environment/MASS_BUDGET.md` §3.1, not edited): **row 32** (harness, 80.0 g CBE, 100.0 g MEV)
holds 75.6 g of this, **−4.4 g** CBE; the ADCS sensor cables W6B–W6E (6.6 g) are row 26's (5.0 g CBE with the sensor
brackets): **+1.6 g** before the brackets; the GNSS coax W10E and its plugs (2.3 g) are part of row 31 (26.0 g with the
patch adapter and bracket). Net against rows 32 and 26 together: 82.2 g against 85.0 g, −2.8 g. The hinge flex
jumpers are in rows 10/11 (2 g each) as the CAD carries them, although row 32's title names them.

The mass distribution changes: about 22 g (15 g of wire and the 7 g of the EPS-side connector halves) now lies under
the EPS (zones Z1–Z5, z ≈ −8, mostly at −X), instead of the four symmetric bundles the mass properties assumed (MECHANICAL_DESIGN §5.3; request M-6).

## 12. EMC

**Grounding.** Single-point ground through EPS mounting hole H1; no OBC frame bond. Every supply has its return in the
same harness (pairs or quads), so no supply current returns through the frame. Nothing in the harness bonds GND to the
frame: no shield is grounded to structure, tie-down dots are non-conductive, A3's Micro-D shell is bonded to the frame
and isolated from every pin (the GSE must keep its signal ground off the connector shell). PACK_N stays isolated from
GND except through DS2 (SQSO-EPS-BRD-001 §6.4): W11 carries PACK_N on its own pins. Two parts attached to the frame
carry harness grounds, the AntS (W7C GND and the W10 shields) and the GNSS patch (W10E shield): **their mounts must be
DC-isolated from the frame** (≥ 1 MΩ at 50 V) so that they do not become a second ground point (request M-9; V-HAR-9).

**Shields.** The only shields are the coax outer conductors, terminated 360° in the MMCX plugs at both ends (RF return
of the RF board and the AntS / patch). W6D's overall shield is removed: the photodiode lines are DC (≤ 5.25 mA) into a
619 Ω transimpedance stage with 10 nF feedback (26 kHz) and a 1 kΩ / 100 nF filter (1.6 kHz) before the ADC; the
inputs are low-impedance virtual grounds that shunt capacitive pickup, and the twisted pairs reject magnetic pickup.
The branches run under the EPS near its 1 MHz trackers: the EPS keeps tracker inductors and switch nodes away from
zones Z1–Z3 where they can (request E-9), and the flatsat checks the sun-sensor channel noise (V-HAR-12).

**Separation.** Burn pairs (W7A/W7B) run only in bands A and B and never share a zone with the magnetometer bus W6E
(bands D–E, (+X,+Y): ≥ 44 mm) or the receiver lines (W4A in (+X,−Y) above z +13.6: ≥ 33 mm; W10 70 cm in the +Y coax
band: ≥ 40 mm). W7A passes 4 mm below W3 in the +X wall gap for 12 s per burn (DC; W3's analog lines are filtered
on the OBC). The PA supply W4B is in the (−X,−Y) bundle, the corner diagonally opposite W6E, ≥ 37 mm
from A2's parts and ≥ 44 mm from the 70 cm coax band; it shares the bundle with W5A (Jetson power) and W8A (not used in
flight), all twisted. The 2 m coax band (y −20 … −24) touches the (−X,−Y) bundle at y −24; the coax shields carry the
separation there.

**Magnetics.** All supplies twisted with their returns (lay ≤ 15 mm), ≤ 5 mm untwisted at each end. The PA supply's
keyed step (0.469 A) through a 5 mm × 1.0 mm untwisted end gives 0.72 nT at A2 from the RF J2 end (87 mm) and 0.47 nT
from the EPS J19 end (100 mm), worst orientation, against the RM3100's 15 nT noise and the RF board's 0.5 nT budget of
its own currents (SQSO-RF-BRD-001 §10.1). Pack leads (twisted quad, 76 mm from A2) and burn lines (coils-off,
deployment only) are not in a measurement window. Conductors are silver-coated copper alloy and the coax inner
conductors silver-plated copper (no copper-clad steel, ANT-8); the only ferromagnetic material near A2 in the harness
is the nickel underplate of A2's own connector.

## 13. Tests

| Test | Requirement |
|------|-------------|
| Continuity | Every conductor of `hardware/harness/wire_list.csv`, pin to pin, ≤ 1 Ω (power conductors by four-wire measurement against §9.2 within +20 %) |
| Isolation | ≥ 100 MΩ at 100 VDC between every conductor and all others, and to A3's shell |
| Dielectric withstanding | 250 VDC for 60 s, leakage ≤ 1 µA (below every connector's rating; ≥ 10 × the 16.8 V maximum working voltage) |
| Crimp | Pull test and cross-section of every terminal and splice type with M22759/33 at each gauge (NASA-STD-8739.4A) |
| RF | W10 and W10E: insertion loss and return loss, pairs matched within 2 mm (RF board C §4.3) |
| Installed | Flatsat functional of every signal; ground isolation with H1 lifted (V-HAR-9) |

## 14. Verification Items

| ID | Item | Method | Pass | When |
|----|------|--------|------|------|
| V-HAR-1 | Continuity, isolation, dielectric withstanding of every harness against the wire list | Test (harness tester) | §13 | Each harness, before integration |
| V-HAR-2 | Crimp qualification: Pico-Clasp 501334 at 28 AWG, Micro-Fit 43030 at 22 and 24 AWG thin-wall (insulation grip; build-up sleeve if needed), Samtec SFSDT, AS81824/1 splices | Test | NASA-STD-8739.4A pull values; cross-sections | Before the EM harness build |
| V-HAR-3 | Mated heights and footprints of every connector with its wire dress (Pico-Clasp 8.0 mm, Micro-Fit RA 9.0 mm and 23 mm, TFM RA 7.0 mm with the plug's back at x ≤ ±43.1, Micro-D envelope) | Inspection on the Molex/Samtec drawings, then measurement on the EM boards | Within the allocations of §10 | EM board layout (with VI-MCAD-01) |
| V-HAR-4 | Fit check on the EM structure: every harness in its zones, the coax bands and A2's 20 mm zone free, no harness in the wing or tape sweeps, slots and deck openings protected, tie points as §10.3 | Inspection, photographs, measured route lengths | No contact with rods, flanges, columns; lengths within the wire list's service allowance | EM integration |
| V-HAR-5 | Harness mass | Weigh each harness and A3 | Total ≤ 84.5 g × 1.25; row-32 content ≤ 100 g | EM and flight builds |
| V-HAR-6 | Loop resistance of the power circuits (four-wire) | Test | §9.2 values within +20 % | V-HAR-1 |
| V-HAR-7 | W1 conductance as part of the battery mount | Test (battery mount thermal test) | ≤ 0.008 W/K for the leads | EM thermal balance |
| V-HAR-8 | Field step at A2 with the PA keyed at 1 W and the Jetson switched | Test (flatsat, with AV-V4/VI-RFB-11) | ≤ 1 nT from the harness | Flatsat |
| V-HAR-9 | Single-point ground: with EPS H1 lifted, GND to frame with every harness, the AntS and the patch installed | Test | ≥ 1 MΩ at 50 VDC | Integration |
| V-HAR-10 | Bend radii against NASA-STD-8739.4A and §2.2, including W10's 7 mm bend against the procured cable's minimum | Inspection | Every bend at or above its minimum | EM harness build |
| V-HAR-11 | Outgassing: E595 data of every harness material (wire, SFSDT, housings, lacing tape, 2216, Arathane 5750, sleeving); bake-out of the finished harnesses | Analysis, test | TML ≤ 1.0 %, CVCM ≤ 0.10 % | Before flight integration |
| V-HAR-12 | Sun-sensor channel noise with the EPS trackers at full power (unshielded W6D) | Test (flatsat) | ≤ 1 LSB (1.3 µA) rms | Flatsat |
| V-HAR-13 | Continuity and connector retention after protoflight vibration | Test | No intermittent > 1 µs (event detector on W1, W5A, W4B), no backed-out contact | Environmental test |

## 15. Changes Needed in Other Documents (for the chair)

| # | Document | Change | Reason |
|--:|----------|--------|--------|
| E-1 | EPS (SQSO-EPS-BRD-001 §13.1, §17) | Every harness connector on the **bottom side** in zones Z1–Z5 and the +X edge (§10.1); bottom-side envelope 9.0 mm (z −3 … −12) in those zones | The top side has 5 mm under the OBC |
| E-2 | EPS | J1, J10, J11, J13, J18 as **right-angle** Micro-Fit 3.0 headers (43045-0800/-0600/-0400 class), gold | 12.5 mm under the EPS; tin prohibited |
| E-3 | EPS | **J19 → Pico-Clasp 6**, pinout of RF J2: 1 RADIO_3V3, 2 GND, 3, 4 PA_5V, 5, 6 GND | No wire fits a Micro-Fit and a Pico-Clasp terminal |
| E-4 | EPS | **J12 → Pico-Clasp 6**: 1, 2 3V3_ANTS; 3, 4 GND; 5 SDA; 6 SCL | One contact would carry 0.61 A (> 50 % of 1 A) |
| E-5 | EPS | J3 → Samtec TFM-120 vertical (SFSDT), bottom side, +X edge, \|y\| < 15.5 | PVC FFSD ribbon; mated height under the OBC |
| E-6 | EPS | PACK-domain corner at (−X, +Y), zone Z1 (J1, J2) | W1 route |
| E-7 | EPS | All Pico-Clasp and Micro-Fit headers in the gold finish (BOM finish column) | EEE_PARTS_LIST §2A.3 item 5 |
| E-8 | EPS | §13.1: name W12A–W12D for J15–J17 and J14; J21 at the +X edge, +Y side is already in §13.1 and §17 (the OBC designer's request: confirmed by this design) | Harness numbering |
| E-9 | EPS | Keep tracker inductors and switch nodes away from the bottom-side zones Z1–Z3 where the layout allows | Unshielded W6D branches |
| O-1 | OBC (SQSO-OBC-BRD-001 §21, BOM) | J1, J2, J9 → Samtec **TFM right-angle** (TFM-120, -108, -105), plug's back ≤ x ±43.1; Pico-Clasp headers gold | PVC ribbon; bend into the 3 mm gap |
| O-2 | OBC | J8 pins 13, 14 not used (W6D drains removed); J8 may become a Pico-Clasp 12 | Shield removed |
| O-3 | OBC | J7's cable leaves into the −X wall gap at y +7 … +19.5 (keep that edge strip free above the board) | Does not cross the 70 cm coax band |
| R-1 | RF (SQSO-RF-BRD-001 §4.1, §4.3, BOM) | J1 → Samtec **TFM-108 right-angle**, cable toward +X, plug's back ≤ x 43.1 (no bend over the top; the 10 mm allocation is kept); J2 gold | As O-1 |
| R-2 | RF | W4B: 28 AWG twisted pairs (was 24 AWG) | Pico-Clasp range |
| R-3 | RF §4.3 | W10 1.37 mm coax top bend R 7 mm is 5.1 × OD, below the 6 × OD workmanship rule for flexible coax: confirm with the procured cable's minimum static radius or accept by analysis | §10.4 |
| J-1 | Carrier (SQSO-JET-BRD-001 §18, BOM) | J1 → **right-angle** Micro-Fit 2×2 (43045-0400 class), gold, plug from −X; J2 gold; keep x +42 … +45.4, y +26 … +31 above the carrier free for W5B | Vertical mated J1 and its bend reach the RF board's bottom envelope |
| B-1 | Battery (`hardware/eps/BATTERY_DESIGN.md` §7.2) | Pack leads 22 AWG (was 20), all leads 0.12 m from the sense board, PTFE over-sleeve; consider a fuse on the sense board in BAT_TOP (W1 is upstream of F2) | §5.2, §9.1 |
| M-1 | Mechanical (SQSO-MECH-CAD-001 §6.9, `model.py`) | Model the harness zones of §7.1 (under the EPS, inboard and corner strips, hinge-side strips, MXC, PXC_B, PXMY2, step-overs) and the EPS bottom-side connector envelope | CAD holds the routes |
| M-2 | Mechanical / frame RFQ | Four grommeted 6 × 4 mm slots in the shear panels at (+48.5, −30, −10), (−48.5, −12, −10), (−30, +48.5, −10), (−30, −48.5, −10) | Body-panel pigtails |
| M-3 | Mechanical | Access-port envelope 30 × 7 × 8.8 mm (x −35 … −5, z −46.8 … −38) for A3 and the Micro-D 21, and the −Y body-panel window to match (the −Z end zone has no cells) | One GSE connector for W8 and W11 |
| M-4 | Mechanical | Termination tabs TB-W1, TB-W2 on the ±X shear panels at (±48, +37, −37 … −31); deployed-state sensor leads through the body-panel notch beside the hinge flex | Wing harness |
| M-5 | Mechanical / ADCS | Positions of the ±Z photodiodes: PZ on the +Z end frame at (−36, +36); the −Z face is covered by the AntS, so the −Z sensor needs a position that sees −Z (harness termination at (−45, +37, −43.5), 30 mm pigtail) | W6D |
| M-6 | Mechanical | Re-run the mass properties with the harness distribution of §11 (about 22 g under the EPS at z ≈ −8, mostly at −X; `hardware/harness/routes.csv` gives every segment) | MECHANICAL_DESIGN §5.3 assumed four symmetric bundles |
| M-7 | Mechanical | Deck openings DX_* and the −Y slot stay open as harness pass-throughs, edges protected | Routing |
| M-8 | Mechanical | W4A as a flat discrete-wire assembly in the gap without the fold over RF J1 (`ribbon_W4A`); §6.9 text: W6E 0.09 m (was 0.25 m), battery leads not in the −X bundle | Right-angle TFM |
| M-9 | Mechanical | AntS and GNSS patch mounts DC-isolated from the frame (≥ 1 MΩ), as A2 is | Single-point ground |
| I-1 | Interconnect (SQSO-ICX-001 §1 rule 5, header line) | Rule 5's values are version 1.2's: the wire derating of SQSO-HAR-001 2.0 §5.2 is EEE-INST-002 with the bundle factor, and the gauge follows the terminal; governing baseline Rev E | §5.2 |
| I-2 | Interconnect §3.1–§3.6 notes | W3, W4A: SFSDT 30 AWG on TFM; W4B, W6B, W6C, W9: 28 AWG; W5A: 24 AWG; W6D: no shield, J8-13/14 unused; §3.6: the bulkhead is A3's Micro-D 21 (pin map §3.2) | §5 |
| I-3 | Interconnect | A revision row for the §2 table and §2.1 rewritten by this design (ICX owner) | §2, §2.1 edited here |
| S-1 | Solar array (`hardware/eps/SOLAR_ARRAY_DESIGN.md` §4.2) | Strings on 28 AWG M22759/33, two wires per pole on the Pico-Clasp (0.28 A each, 37 % of 0.90 A derated), not 24 AWG | Pico-Clasp range; §5.2 |
| P-1 | `docs/manufacturing/EEE_PARTS_LIST.md` | Connector finishes gold: Pico-Clasp, Micro-Fit, Samtec TFM/SFSDT, MMCX, Micro-D | §2A.3 item 5 |
| P-2 | `docs/compliance/MATERIALS_LIST.md` | Wire M22759/33 (ETFE), SFSDT PTFE, MIL-T-43435 lacing tape, Scotch-Weld 2216, Arathane 5750, PTFE sleeving, connector housings: E595 data per part number | V-HAR-11 |
| D-1 | Diagrams (`hardware/diagrams/README.md` §2) | W7 now draws W12A–W12D, W8 draws W11 and A3; the "main sources" column cites SQSO-HAR-001 2.0 instead of H1 and H8 | Drawings updated here |
| D-2 | MEL (`docs/environment/MASS_BUDGET.md`) | Row 32 note: harness design 75.6 g (−4.4 g); row 26 sensor cables 6.6 g (+1.6 g before brackets) | §11 |

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-27 | CDR structures review (STR-25), mechanical routing only: battery flying leads ≥ 100 mm with no stack connector (§4.1); routing zones F and G and the mechanical routing rules (hinge flex jumper, back-wired strings, deployment-sweep clearance, isolated battery, cable ties per UG v11 §5.2.3, envelope, vent paths) (§6.2, §6.3); hinge crossing (§8.1); mass note against the MEL (§9) |
| 1.2 | 2026-10-27 | CDR 2026 consistency pass: note in §4 that SQSO-ICX-001 and the board designs (harnesses W1–W10) supersede the H1–H6 and H8 contents, W11 proposed (finding F15 of SQSO-HW-DIAG-001) |
| 2.0 | 2026-10-27 | Rewritten as the build-to harness design (RID SE-19): the H1–H8 definitions and the superseded notice removed; wire list of W1–W12, W10 and W10E (206 conductors) in `hardware/harness/wire_list.csv`, routes in `hardware/harness/routes.csv`, calculations in `hardware/harness/harness_calcs.py`; gauges from the terminal ranges, AS22759/33 wire, EEE-INST-002 derating; band-B over-fill resolved by routing the EPS-only harnesses under the EPS (connectors on the EPS bottom side), W6D unshielded, pack leads 22 AWG; Samtec TFM/SFSDT replaces the PVC ribbons; access-port board A3 with one Micro-D 21; gold finishes; CAD check of 58 zones; mass 84.5 g; EMC; V-HAR-1 to V-HAR-13; changes for the other documents (§15); WireViz W1–W10 updated |
