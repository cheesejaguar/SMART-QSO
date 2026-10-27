# SMART-QSO Mechanisms Analysis

**Document ID**: SQSO-MECH-001
**Version**: D
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR issue. Analysis of the two solar-wing hinges and hold-downs and of the bought ISIS AntS antenna system;
no mechanism hardware exists. Version B replaces the wing hinge and spring of Version A, which do not fit the envelope
(finding MCAD-01 of SQSO-MECH-CAD-001, reopening STR-13), with a design that fits (§3A, §4A, §5A). Scripts:
`software/simulation/cdr/structures/mechanisms.py` (Version A analyses, burn wire, deployment dynamics model) and
`hardware/mechanical/hinge.py` (Version B hinge and spring design, torque margins, dynamics with the new spring; results
in `hardware/mechanical/hinge_results.json`). Mass properties: `docs/environment/MASS_BUDGET.md`; structure:
`docs/environment/STRUCTURAL_ANALYSIS.md`; CAD: `hardware/mechanical/MECHANICAL_DESIGN.md`.

## 1. Purpose and Result

| Check | Result | Section |
|-------|--------|---------|
| **Version B** wing static torque margin, NASA-STD-5017B Eq. 4-1, "theory or analysis" factors, worst case (cold, end of travel), with the hinge and springs that fit the envelope and the flex at the bend radius that fits | **+0.14** (start of travel +0.98; hot +0.37 to +1.31); qualification-test factors +0.37; **one spring out +0.13** (FS 1.0, DDMR 53 two springs) | §4A |
| Version B spring stress (5017B Table 4, mission-critical 1.5 / 1.65 at the maximum torque) | Primary +0.12 / +0.36; secondary **+0.04** / +0.27 (891 and 958 MPa, 59–64 % of the 1,500 MPa allowable: above the 50 % that A.2.5.1 recommends against relaxation, hence VI-MECH-01) | §3A |
| Version A torque margin (springs that do not fit, withdrawn) | +0.29 (one spring out +0.88) | §4 |
| Same with a contact (plunger) deployed-state switch at the stop | **−0.54**: the deployed sensor must be non-contact | §4 |
| Holding margin on the stop (tumbling at 50°/s; spin 8°/s puts no torque on the hinge) | +46 | §4.4 |
| Deployment time | 0.16 s from release to the stop (requirement < 5 s, `hardware/eps/SOLAR_ARRAY_DESIGN.md` §8.1) | §5 |
| End-of-travel shock (Version B spring) | 13.7 rad/s at the stop: **0.157 N·m with the damped soft stop (2 N·m/rad, ζ 0.3): 214 µε at the wing root, rebound 13°**; 0.62 N·m (853 µε, rebound 50°) with a metal stop; deployment 0.21 s | §5A |
| Deployed-position latch | **No latch**: a latch pin (Ø0.8 mm) would see 974 MPa in shear at the rebound impact (allowable 662 MPa) and its drag lowers the end-of-travel margin to +0.07. The springs hold the wing on the damped stop: holding margin +17 (DDMR 14) | §5A |
| Wing deployment while tumbling (15–50°/s) | No effect on the mechanism (inertial torque ≤ 0.1 mN·m); body rate after deployment scales with the inertia ratio | §5 |
| Burn-wire release energy (simulation model, flight timing) | Wing line, 5 s: 16.0 J against a 3σ need of 15.4 J at −40 °C (ratio **1.04**); 8 s gives 1.67. Antenna line, 3 s: ratio 0.63 at −40 °C, **11 % not released after three attempts** in the model; but the bought AntS has its own release electronics, so the model and the flight timing do not represent it | §6 |
| Partial deployment | One wing out: CG moves 1.4 mm in X, the major principal axis tilts 12° from +Y, generation and drag area drop; survivable; the flight software has no one-wing configuration | §8 |
| Wing and tape flexible modes vs spin and ADCS bandwidth | Wing 21–83 Hz, VHF tape 0.4–4 Hz, against a 0.017 Hz spin and a < 0.001 Hz control bandwidth: no coupling; dissipation makes the major-axis rule apply (MASS_BUDGET §6) | §9 |

Verdict for this discipline: **ready for fabrication with actions**: spring and sensor specification (§4), a soft or
damped stop (§5), a longer wing burn or a higher burn current (§6), the AntS command interface in place of the
simulated antenna burn line (§7), and the deployment order wings-then-antennas or a verified clearance (§7.2). The
torque margins rest on estimated resistive torques; the hinge torque test (VI-STR-04) replaces them.

**Version B.** The Version A springs (Ø0.55 mm wire, mean Ø4 mm, 22 coils, 12 mm long, two per wing on the hinge line)
cannot be built into the envelope: their 2.28 mm radius about the hinge axis reaches 6.38 mm from the rail plane and
y = 41.03 mm (inside the 8.5 mm rail zone), and the wing's end zones leave 5.9 mm, not 12 mm (SQSO-MECH-CAD-001 §6.8).
The Version B hinge puts the primary spring on the hinge axis **below the wing** (the wing was shortened to z ≥ −43 mm for
the AntS clearance, which frees the axis from z −47.6 to −40.1 mm) and a secondary spring in the +Z end zone; the
torque margin closes with positive values everywhere, but with little to spare (+0.13 to +0.14 at the cold end of
travel, +0.04 on the secondary spring's yield). It rests on three design choices that the hinge torque test must confirm:
MoS₂-coated pins, a PTFE-faced snubber and a heat-set clock-spring flex (§4A).

## 2. Applicable Documents

| Document | Paragraphs used |
|----------|-----------------|
| NASA-STD-5017B (2022-12-06), *Design and Development Requirements for Mechanisms* | §4.3 Eq. 4-1 and Table 1; §4.3.1.1 DDMR 9 (worst case); §4.3.1.3 DDMR 11 (acceptance at the highest level); §4.3.2–§4.3.4 DDMR 12–14 (static, dynamic, holding margins); Appendix A.2.5 Table 4 (springs) and A.2.5.1 (stress relaxation) |
| AIAA S-114A, *Moving Mechanical Assemblies for Space and Launch Vehicles* | Cited by 5017B (its torque-margin section was aligned with S-114A-2020); not obtained for this review, 5017B used |
| NASA-STD-5001B w/Change 3 | Table 6 (softgood lines), Table 1 (metallic parts) |
| CDS Rev 14.1 | §2.2.4 (deployables constrained by the CubeSat), §2.3.8 (three inhibits per deployable), §2.4.4 (30 min wait) |
| SpaceX Rideshare Payload User's Guide v11 | §5.7 (no passive deployment of appendages on ejection), §6.3 item 7 (burn wires may be reset after test if identical and the test shows the original did not fail) |
| Mission simulation | `software/rust/sqso-sil/src/hw/deploy.rs` (release-energy model), `sqso_fsw::deployment` (burn 5 s wing / 3 s antenna, 3 attempts 60 s apart, 30 min wait) |

## 3. Mechanisms (Version A; the wing hinge, springs and flex are superseded by §3A)

| Mechanism | Design (proposed here where the documents are silent) |
|-----------|--------------------------------------------------------|
| Wing hinge | Two hinges per wing on the ±X/+Y edge (axis parallel to Z, at x = ±54.0, y = +39.5 mm), 300-series pins Ø1.5 mm in Al knuckles with a dry-film lubricant; a torsion spring at each hinge; a hard stop at 90° with a soft (silicone, low-outgassing) bumper; no latch, the spring preload holds the wing on the stop |
| Torsion springs | 17-7PH CH900 wire Ø0.55 mm, mean coil Ø4 mm, ≈ 22 coils, rate 3.2 mN·m/rad; 10.0 mN·m stowed (180° wind-up), 5.0 mN·m at the stop (90°); loaded in the winding direction. Bending stress at 10 mN·m: 674 MPa (Wahl factor 1.1) against ≈ 1,500 MPa (supplier allowable, to confirm): MS +0.48 on yield with the mission-critical factor 1.5, +0.80 on ultimate with 1.65 (5017B Table 4). 34 % of the wire strength at the stowed load keeps stress relaxation low (A.2.5.1) |
| Hold-down and release | One line per wing over the free edge (Vectran or Dyneema, 200 N break, 30 N preload), cut by **two** burn resistors on separate drivers (adopted: four independent 0.8 A drivers, one per resistor, burned one at a time, `hardware/electrical/eps/EPS_BOARD_DESIGN.md` §9 and BASELINE Rev E §4; the first issue found one resistor per wing on a shared line in `deploy.rs`); both line ends captured after the cut (no debris, CDS §2.1.1) |
| Deployed-state sensor | **Non-contact** (reed switch and magnet, or Hall sensor) at the stop: a plunger switch at the stop costs the margin (§4.3). Stowed-state contact switches released by the wing are acceptable (they push the wing out) |
| Inhibits | Three independent physical inhibits between the battery and every burn resistor (BASELINE §4, CDS §2.3.8); deployment no earlier than 30 min after switch release (CDS §2.4.4; `sqso_fsw::deployment`) |
| Antenna system | ISIS AntS 1U–3U, monopole/hybrid configuration: two 2 m and two 70 cm tape-spring elements (VHF ≤ 55 cm, UHF ≤ 17 cm per ISIS) in the plane of the −Z face; vendor release (redundant thermal knives, I²C controller, < 2 W at 3.3 V, < 3 s above +15 °C per ISIS brochures); the elements carry their own strain energy |

## 4. Wing Torque Margin (Version A springs; superseded by §4A)

### 4.1 Method

TM = T_avail / (Σ FS_f·T_f + Σ FS_v·T_v + Σ FS_a·T_a) − 1 ≥ 0 (5017B Eq. 4-1), with the Table 1 "theory or analysis"
factors FS_v = 3.00, FS_f = 1.50, FS_a = 1.25 until test data exist (the "qualification test" row, 2.50 / 1.35 / 1.15,
is shown for when VI-STR-04 has measured the torques). Worst case per DDMR 9: minimum spring torque (−10 % tolerance,
−5 % relaxation over two years stowed, modulus at the temperature) against maximum resistance at the same temperature
and the same point of travel. Temperatures −40 °C and +60 °C.

| Resistive torque (per wing) | Stowed, cold | Stowed, hot | Deployed, cold | Deployed, hot | Kind |
|-----------------------------|--------------|-------------|----------------|---------------|------|
| Hinge friction, 2 pins, μ 0.5 (vacuum, dry), spring leg loads | 1.0 | 0.8 | 0.5 | 0.4 mN·m | T_v |
| Flex-circuit jumper across the hinge (set and cold stiffening; 10 mm wide polyimide, R ≥ 5 mm) | 1.5 | 1.0 | 1.5 | 1.0 mN·m | T_v |
| Bumper adhesion at breakaway | 0.5 | 0.5 | 0 | 0 mN·m | T_v |
| Drag of the cut hold-down line | 0.2 | 0.2 | 0.2 | 0.2 mN·m | T_v |
| Inertial torque from tumbling at 50°/s | 0.1 | 0.1 | 0.1 | 0.1 mN·m | T_f |
| Acceleration to deploy within 5 s from rest | 0.015 mN·m | | | | T_a |

### 4.2 Results

| Point of travel | Temperature | T_avail (mN·m) | ΣT_v | TM, analysis factors | TM, qualification-test factors | One spring out (FS 1.0) |
|-----------------|-------------|----------------|------|----------------------|-------------------------------|-------------------------|
| Start (stowed) | Cold | 17.41 | 3.20 | +0.78 | +1.14 | +1.63 |
| Start (stowed) | Hot | 16.89 | 2.50 | +1.20 | +1.64 | +2.23 |
| **End of travel** | **Cold** | **8.70** | **2.20** | **+0.29** | +0.54 | +0.88 |
| End of travel | Hot | 8.45 | 1.60 | +0.70 | +1.03 | +1.46 |

The margin is positive in every case but the cold end-of-travel case rests on the flex-jumper torque (1.5 mN·m assumed).
**Requirement (STR-13):** the hinge jumper is a polyimide flex circuit, not a wire bundle (a bundle of eight 26 AWG wires
bent around a 5 mm radius would add ≈ 20 mN·m and defeat the springs), with a measured torque ≤ 1.5 mN·m at −40 °C over
the full travel.

### 4.3 Deployed-State Sensor

A plunger microswitch at the stop (0.4 N on a 10 mm arm, 4 mN·m against the spring at the end of travel) gives
TM = −0.54 cold. The deployed-state sensor is therefore a reed switch or Hall sensor; if a contact switch is used, it is
pressed in the stowed state and released by the wing.

### 4.4 Holding Margin on the Stop (DDMR 14)

The spin about +Y (8°/s) produces centrifugal forces along the wing span, through the hinge line: no hinge torque.
Tumbling about X at 50°/s produces ≈ 0.06 mN·m; magnetorquer torques produce < 0.001 mN·m. Holding margin with FS_v 3.0:
8.45 / (3 × 0.06) − 1 = **+46**. The wing never leaves its stop in orbit, so its dynamics are those of the stop
(21–74 Hz, §9).

## 5. Deployment Dynamics (Version A springs; Version B in §5A)

Model: planar, free-floating, two bodies (the spacecraft body with the other wing stowed, MEV 1.63 kg and 3.0 × 10⁻³ kg·m²
about +Z; one wing 50.4 g, 83 mm span) about the hinge axis, angular momentum conserved, the body rotating about +Z at the
start; spring of §3, resistance 2.2 mN·m, stop as a contact spring with 5 % damping.

| Stop | Initial body rate | Time to 90° | Wing rate at the stop | Peak stop torque | Wing-root stress / strain | Peak body rate | Rebound |
|------|-------------------|-------------|-----------------------|------------------|---------------------------|----------------|---------|
| Soft bumper, 2 N·m/rad | 0 | 161 ms | 18.2 rad/s | 0.243 N·m | 5.8 MPa / 313 µε | 65°/s (transient) | 49.5° |
| Soft bumper | ±15°/s | 161 ms | 18.2 rad/s | 0.243 N·m | same | 68–79°/s | 49.5° |
| Soft bumper | 50°/s | 161 ms | 18.3 rad/s | 0.244 N·m | same | 100°/s | 49.6° |
| Metal stop, 25 N·m/rad | 0 | 161 ms | 18.2 rad/s | 0.846 N·m | 20.2 MPa / 1,088 µε | 65°/s | 51.2° |
| Metal stop | 50°/s | 161 ms | 18.3 rad/s | 0.849 N·m | 20.3 MPa / 1,092 µε | 100°/s | 51.3° |

Findings:

1. **Tumbling does not matter to the mechanism.** The wing reaches the stop in 0.16 s whatever the body rate up to
   50°/s; the swing's own rates (18 rad/s) dwarf the tumble.
2. **The body rate jumps to 65–100°/s for ≈ 0.2 s** while the wing swings (internal momentum exchange) and returns to
   its earlier value; afterwards the rate scales with the inertia ratio (15°/s about Z becomes 13.4°/s). The gyro
   (M-G370PDT0 range ±450°/s; BMI088 to ±2,000°/s) does not saturate. B-dot sees a short spike; harmless.
3. **The end-of-travel shock governs the wing.** With a metal stop the root strain is ≈ 1,100 µε. The 3G30A cells are on
   germanium substrates and are brittle; the panel vendor's allowable deployment shock or strain governs. **Requirement
   (STR-14):** a soft or damped stop limiting the peak to ≤ 0.3 N·m (≈ 350 µε), verified by the deployment test with
   high-speed video and electroluminescence inspection of the cells before and after (VI-STR-05). With 5 % damping the
   wing rebounds ≈ 50° and settles over a few cycles; a damper (silicone bumper with ζ ≥ 0.2, or an eddy-current or
   friction damper) also shortens that.
4. The wings release one after the other (the release energies differ by ±1.2 J at 3.2 W, ±0.4 s, longer than the
   0.16 s swing), so the momentum exchange does not cancel between wings; the net angular momentum is unchanged in
   either case.

## 3A. Version B Hinge and Springs (closes MCAD-01; STR-13)

Envelope (SQSO-MECH-CAD-001 §4.2, §6.2): hinge axis at (±54.1, +38.75) mm, parallel to Z. Anything on the axis has a radius
≤ 2.0 mm (6.10 mm from the rail plane nominal, 6.38 mm with the tolerance chain, against 6.5 mm; y ≤ 40.75 mm, 9.25 mm
from the rail edge). Free length on the axis: z −49.0 … −37.3 mm at −Z (below the wing and in its end zone) and
43.3 … 49.6 mm at +Z.

| Element | Design | Location (z, mm) |
|---------|--------|------------------|
| Pin | **A286** (AMS 5732, aged; Version D, was 440C) Ø1.5 mm, sputtered MoS₂ on the pin and a burnished, binder-free MoS₂ film in the bore, in Ti-6Al-4V bushings. A286 is non-ferromagnetic: the W1 hinge is within 20 mm of the magnetometer board A2 (SQSO-MECH-CAD-001 §4.4); W2 uses the same pins | Through the whole hinge line |
| −Z frame ear | Al 6061 bracket, leaf bolted through the body-panel notch to the ±X shear panel; carries the pin and the primary spring's frame leg | −49.0 … −47.6 |
| **Primary spring** | 17-7PH CH900, wire **0.470 mm** (0.0185 in), OD 4.0 mm, mean Ø3.53 mm (index 7.5), **15.0 coils**, 7.5 mm long; legs 5 mm long anchored in the frame ear and in the wing's −Z end zone | −47.6 … −40.1 |
| −Z wing knuckle | Clock-spring flex cup (OD 4.0 mm, inner radius 1.6 mm; hub radius 1.0 mm on the pin) | −39.9 … −37.3 |
| +Z wing knuckle | Ti bushing, Ø3.5 mm | 43.6 … 44.6 |
| **Secondary spring** | 17-7PH CH900, wire **0.343 mm** (0.0135 in), OD 4.0 mm, mean Ø3.66 mm (index 10.7), **9.5 coils**, 3.6 mm long | 44.8 … 48.4 |
| +Z frame ear | As −Z, bolted to the shear panel | 48.6 … 49.6 |
| Preload | Both springs wound **55°** beyond the deployed position (145° at stowage), loaded in the winding direction (A.2.5) | — |
| Stop | Silicone bumper with ζ ≥ 0.3 (damped), 2 N·m/rad, on the frame ear; non-contact (reed or Hall) deployed sensor beside it | −Z end zone |
| Snubber, hold-down | PTFE-faced silicone snubber at the +Z free corner; cup/cone post and line at the −Z free corner | — |

| Spring | Rate (mN·m/rad) | Torque stowed / deployed, nominal (mN·m) | Max stress (MPa), at +10 % torque, cold modulus | MS yield (1.5) | MS ultimate (1.65) | Wound ID (mm) |
|--------|-----------------|------------------------------------------|---------------------------------------------------|----------------|--------------------|---------------|
| Primary | 2.89 | 7.31 / 2.77 | 891 | **+0.12** | +0.36 | 2.97 (pin 1.5) |
| Secondary | 1.25 | 3.15 / 1.20 | 958 | **+0.04** | +0.27 | 3.17 |

Rate k = E d⁴ / (64 D n), stress Ki · 32 M / (π d³) with Wahl's bending factor Ki; E = 200 GPa, allowables 1,500 MPa
(bending) and 2,000 MPa (ultimate), the supplier values Version A used (to be confirmed with the spring vendor). Two
springs make the hinge failure tolerant (DDMR 53); one spring alone cannot reach a positive margin in this envelope
(§4A). Both springs run at 59–64 % of the bending allowable, above the 50 % of the elastic limit that 5017B A.2.5.1
recommends against stress relaxation: the 5 % relaxation allowance in §4A is therefore verified by test on the spring lot
(VI-MECH-01) instead of being taken for granted.

**Harness across the hinge.** A two-layer polyimide flex (2.0 mm wide, 0.10 mm polyimide, 2 × 18 µm copper: two power
pairs and the photodiode, thermistor and sensor lines) wound as a 6-turn clock spring between the hub (radius 1.0 mm) and
the cup (1.6 mm) in the −Z wing knuckle, heat-set at mid-travel. Bend radius 1.0–1.6 mm: above the IPC-2223
flex-to-install minimum of 6 × thickness (0.82 mm), not a dynamic-flex radius, which is acceptable for a one-time
deployment and the ground tests (copper strain 3.4 % when formed, once; ±0.05 % during deployment). Elastic torque
≈ 0.06 mN·m over the 90° travel (EI 3.7 × 10⁻⁶ N·m², 49 mm free length); **allocation 0.50 mN·m cold, 0.35 mN·m hot**
for inter-layer friction and cold stiffening. The R ≥ 5 mm of Version A cannot be packaged at this hinge line.

## 4A. Version B Torque Margin (NASA-STD-5017B Eq. 4-1)

Worst case (DDMR 9): minimum spring torque (−10 % tolerance, −5 % relaxation, modulus +1.8 % at −40 °C and −1.2 % at
+60 °C) against the maximum resistance at the same temperature and point of travel. Pin friction is computed from the
spring-leg reactions: T = μ · 2 (r_pin / a_leg) · M_spring with r_pin 0.75 mm, a_leg 5 mm and the spring torque at
+10 % (conservative), μ = 0.15 cold / 0.12 hot (MoS₂; the vacuum typical is 0.02–0.05).

| Resistive torque per wing (mN·m) | Stowed, cold | Stowed, hot | Deployed, cold | Deployed, hot | Kind |
|----------------------------------|--------------|-------------|----------------|---------------|------|
| Pin friction (both springs' leg loads, +Z bushing 0.05) | 0.58 | 0.47 | 0.25 | 0.21 | T_v |
| Clock-spring flex | 0.50 | 0.35 | 0.50 | 0.35 | T_v |
| PTFE-faced snubber breakaway | 0.20 | 0.20 | 0 | 0 | T_v |
| Cut hold-down line drag | 0.20 | 0.20 | 0.20 | 0.20 | T_v |
| **Σ T_v** | **1.48** | **1.22** | **0.95** | **0.76** | |
| Tumbling inertial torque, 50°/s | 0.10 | 0.10 | 0.10 | 0.10 | T_f |
| Acceleration to deploy within 5 s | 0.01 | 0.01 | 0.01 | 0.01 | T_a |

| Point of travel | Temperature | T_avail (mN·m) | TM, analysis factors (3.00 / 1.50 / 1.25) | TM, qualification-test factors (2.50 / 1.35 / 1.15) | One spring out (FS 1.0) |
|-----------------|-------------|----------------|-------------------------------------------|-----------------------------------------------------|-------------------------|
| Start (stowed) | Cold | 9.11 | +0.98 | +1.37 | +1.24 |
| Start (stowed) | Hot | 8.84 | +1.31 | +1.76 | +1.56 |
| **End of travel** | **Cold** | **3.45** | **+0.14** | **+0.37** | **+0.13** |
| End of travel | Hot | 3.35 | +0.37 | +0.63 | +0.32 |
| End of travel, cold, primary spring only | Cold | 2.41 | −0.15 | — | — |
| End of travel, cold, with a latch pin (0.07 mN·m drag) | Cold | 3.45 | +0.07 | — | +0.05 |

Sensitivities (end of travel, cold, analysis factors): the margin falls to zero at a flex torque of 0.64 mN·m (allocation
0.50, elastic estimate 0.06), at μ = 0.26 on the pin, or at a silicone (non-PTFE) snubber only at the start of travel
(not governing). The governing case is the end of travel, where the springs have unwound: the preload (55°) is the
largest the stress allows.

**Least-bad note, recorded for the chair:** the design closes, but not with the margins of Version A (+0.29). The only
levers left inside the envelope are the resistive torques (lubricant, flex), so the hinge torque test (VI-STR-04) is
the closure of STR-13, and a measured flex torque above 0.64 mN·m at −40 °C would require either a stronger spring
material (Elgiloy or a higher-strength 17-7PH lot, if the vendor's bending allowable exceeds 1,600 MPa) or shorter cells
to free a longer spring.

## 5A. Version B Deployment Dynamics, Stop and Hold

The free-floating two-body model of §5 (`mechanisms.py`) with the Version B springs (nominal torque), the CAD wing
(50.3 g MEV, span 81.6 mm, 1.12 × 10⁻⁴ kg·m² about the hinge) and 1.5 mN·m resistance:

| Stop | Body rate at release | Time to 90° | Wing rate at the stop | Peak stop torque | Wing-root strain | Rebound | Peak body rate |
|------|----------------------|-------------|-----------------------|------------------|------------------|---------|----------------|
| **Damped soft stop, 2 N·m/rad, ζ 0.3 (design)** | 0, ±15, 50°/s | 0.21 s | 13.7 rad/s | **0.157 N·m** | **214 µε** | **13°** | 35–65°/s (transient) |
| Metal stop, 25 N·m/rad, ζ 0.05 (reference) | 0 … 50°/s | 0.21 s | 13.7 rad/s | 0.62 N·m | 853 µε | 50° | 48–86°/s |

The stop requirement of STR-14 (≤ 0.3 N·m, ≈ 350 µε) is met with the damped soft stop (0.157 N·m). **Deployed-position
hold:** no latch. A spring pin dropping into a hole at 90° was evaluated: with the damped stop the rebound returns
e² = 14 % of the impact energy, which a stiff latch (200 N·m/rad) converts into 0.59 N·m on a Ø0.8 mm pin at 1.2 mm
radius: 974 MPa in shear against 662 MPa for A286, and its drag would lower the end-of-travel margin to +0.07. The
springs instead hold the wing on the stop with at least 3.35 mN·m (minimum, hot, both springs) against 0.06 mN·m from a
50°/s tumble: holding margin per DDMR 14 (FS_v 3.0, incidental friction excluded) **+17**; with one spring out
(primary failed) the secondary alone holds 1.0 mN·m: margin +4.6. The rebound (13°) settles within two cycles of the
damped stop.

**Hinge pin strength (Version D, A286; `hinge.py`, `pin_strength`).** The damped stop's peak torque (0.158 N·m) at a
4.0 mm contact radius (CAD: about 4.5 mm) gives 39 N at the stop; with 1.5 × for the wing's momentum change, 59 N on one
pin end. Ø1.5 mm A286 (Ftu 1,103, Fty 758, Fsu 662 MPa) in shear and in bending over a 0.6 mm arm at the ear–knuckle
interface: 33 MPa shear, 107 MPa bending, 122 MPa von Mises: **MS +3.3 yield, +4.6 ultimate** (5001B 1.25 / 1.40 with a
1.15 fitting factor), +11 in shear. At launch the FE hinge load (2.3 N quasi-static plus 3.85 N 3σ, added) gives MS
> +40. Bearing on the Ti-6Al-4V bushing over a 1.0 mm length: 39 MPa. The torque margins of §4A are unchanged: μ is the
MoS₂ film's (0.15 cold, 0.12 hot), not the pin alloy's. What changes is the film's substrate: A286 (HRC 30–35) is softer
than 440C (HRC 58), so the sputtered film wears faster, and unlubricated A286 on Ti-6Al-4V galls (μ about 0.4–0.5,
above the 0.26 at which the end-of-travel margin reaches zero). Hence the film in the bore as well as on the pin, and
VI-MECH-03.

## 6. Burn-Wire Release

Model (`deploy.rs`): release energy N(6.0 J, 1.2 J) at +20 °C, +1 % per °C below +20 °C, 20 % of the heat kept between
attempts; 0.8 A into 5 Ω per resistor (3.2 W); flight timing (`sqso_fsw::deployment`): 3 attempts 60 s apart.

| Line | Temperature | Energy per attempt | 3σ need | Ratio | Released on attempt 1 | Not released after 3 attempts |
|------|-------------|--------------------|---------|-------|-----------------------|-------------------------------|
| Wing, 5 s (flight) | +20 °C | 16.0 J | 9.6 J | 1.67 | 100 % | 0 |
| Wing, 5 s | −20 °C | 16.0 J | 13.4 J | 1.19 | 100 % | 0 |
| Wing, 5 s | **−40 °C** | 16.0 J | 15.4 J | **1.04** | 99.95 % | 0 |
| Wing, 8 s (proposed) | −40 °C | 25.6 J | 15.4 J | 1.67 | 100 % | 0 |
| Antenna line, 3 s (flight; simulation model) | −20 °C | 9.6 J | 13.4 J | 0.71 | 76.5 % | 1.8 % |
| Antenna line, 3 s | −40 °C | 9.6 J | 15.4 J | 0.63 | 50.1 % | **11.4 %** |

The deployment happens 30 min after separation, possibly in eclipse; the mission simulation's smoke run shows the panels
at +10 to +18 °C at that time, but the cold case is not bounded until the thermal analysis gives the minimum stowed-wing
temperature at T + 30 min.

**Requirement (STR-15):** the delivered energy per attempt is at least 1.5 × the 3σ release energy at the minimum predicted
deployment temperature less 10 °C (the thermal uncertainty margin of BASELINE §8), demonstrated by a release
characterisation of ≥ 30 coupons of the flight line, resistor and preload at −40 °C and +20 °C (VI-STR-06). With the
simulation's model this needs **8 s** (`SOLAR_BURN_TIME_MS` = 8000) or 1.0 A per resistor (25 J in 5 s); for the chair,
because the constant is in `sqso_fsw::deployment`. With the EPS board's drivers (0.8 A per resistor from a 7.0 V burn buck, one resistor at a
time for 8 s) the pack carries 0.51 A for the burn, 0.67 A with housekeeping and heater, 0.75 A with the magnetorquers
(`hardware/electrical/eps/EPS_BOARD_DESIGN.md` §9); 25.6 J into a 5 Ω resistor per 8 s burn. 1.0 A per resistor would
need the driver set point changed.

Model fidelity (for the chair, owner of `sqso-sil`): `deploy.rs` (1) drops a resistor's current to zero once its line has
parted, but the resistor stays in circuit unless it burns open, so retries draw more current than modelled; (2) takes
the deployment temperature from the +Z panel, not the wing or the AntS; (3) models the antennas as a 5 Ω line on the
14.4 V bus, which the bought AntS is not (§7).

## 7. Antenna System (ISIS AntS)

### 7.1 Interface

The AntS deploys through its own controller (I²C command, redundant thermal knives per element, < 2 W at 3.3 V, < 3 s
above +15 °C according to the ISIS brochures; the current datasheet, ISIS-ANTS-DSH-0001 v4.0, could not be retrieved:
the vendor's pages returned 404). Its torque margin is the tape springs' own strain energy and is the vendor's
qualification item (VI-STR-13). Consequences for this design:

- `sqso_fsw::deployment` drives an antenna burn line for 3 s (`ANTENNA_BURN_TIME_MS`), and the simulation models it as a
  5 Ω resistor: neither matches the AntS. The antenna step must become the AntS command sequence (arm, deploy each
  element with the vendor's burn time, read back the per-element deployment switches), with the three physical inhibits
  on the AntS deployment supply (for the chair: FSW, EPS and SIL owners).
- The AntS reports each element separately; the flight software's single "both antennas" sense (`deploy.rs`: the two
  switches in series) loses that information.

### 7.2 Clocking and Clearance

- **Clocking:** the 2 m (VHF) elements must lie along ±X, parallel to the deployed wings. Any 2 m element along ±Y makes
  +Y the intermediate axis of inertia and the sun-pointing spin fails: Iyy/Izz = 0.89 with two elements along ±Y, 0.97–0.98
  for options B and C of `hardware/rf/ANTENNA_DESIGN.md` v2.0 §3 (one element along +Y, or a quadrature pair along +X and
  +Y), with 52–60 % of sunlit time within 15° instead of 100 % (MASS_BUDGET §6.4, item 6). The antenna design recommends
  option C for the link; this is a conflict for the chair (STR-01).
- **Clearance:** along ±X the VHF elements run in the −Z face plane (z ≈ −55 mm), 4–6 mm below the lower edge of a stowed
  wing (z = −49 mm). If the antennas deploy first (the present order: antenna burn at T + 30:01, wings at T + 30:06 in the
  mission simulation), the tapes oscillate under the stowed wings and the wings then sweep over them with a few
  millimetres' clearance. Deployed, the wing plane (y = +39.5 mm) is ≈ 40 mm from a tape on the X axis.
  **Requirement (STR-16):** deploy the wings first and the antennas second, or show ≥ 10 mm clearance throughout both
  sweeps in the CAD and in the deployment test (VI-STR-10).

## 8. Partial Deployment

| Case | Mass properties (layout B, rods, CBE) | Consequences | Handling |
|------|---------------------------------------|--------------|----------|
| Both wings and antennas out (nominal) | CG (−0.8, +2.2, −4.7) mm; Iyy/Izz 1.041; major axis 13° from +Y | Nominal | — |
| One wing (W1) out, W2 stowed | CG (+0.6, +0.8, −4.7) mm; I diag (2.87, 3.95, 3.74) × 10⁻³; major axis 12.5° from +Y | +Y face has 4 coplanar cells instead of 6; spin axis tilts 12.5° in the body (the sun sensors and the law assume +Y); aero torque from the CG offset; drag area between the two BASELINE §3 cases | Retries (3 per element, the wing line serves both wings, so a retry burns the released wing's resistor too); afterwards a one-wing ADCS parameter set (spin axis and inertia) by telecommand — **not available today** (`SUN_AXIS` and `INERTIA` are constants in `sqso_fsw::adcs`; for the chair, STR-27) |
| Neither wing out, antennas out | CG (−0.8, −0.7, −4.7) mm; I diag (2.82, 3.59, 3.34) × 10⁻³; major axis 5° from +Y | Wings-failed case of BASELINE §3 (lifetime) and BUILD_VS_BUY §7.3 (power: lowest SOC 66 %, 2.4 % DoD); +Y spin still the major axis (1.075) | Already analysed; no change |
| A wing stuck part-way | Between the cases above; principal axes rotated | Shading and pointing as above; the spring keeps driving it (holding torque 8.4–17 mN·m) | Retry heating does not help a mechanical jam; telemetry of the deployed sensor (non-contact) distinguishes it |
| One AntS element not out | Inertia change ≤ 0.33 × 10⁻³ kg·m² on one axis (a VHF element) | RF: the other element of the band, if the hybrid configuration has two; inertia ratio changes by up to 8 % | AntS retries the element; telemetry per element (§7.1) |

## 9. Flexible Modes, Spin and ADCS Bandwidth

| Mode | Frequency |
|------|-----------|
| Deployed wing on its stop (rigid rotation): soft bumper / metal stop | 21 / 74 Hz |
| Deployed wing bending (cantilever) | 83 Hz |
| Wing rotation if it left the stop (spring only, 6.4 mN·m/rad per wing) | 1.2 Hz; not reached in orbit (holding margin +46, §4.4) |
| AntS VHF tape, 0.5 m (curved / flat direction) | ≈ 4 / ≈ 0.4 Hz |
| Spin (8°/s, BASELINE Rev E §8) | 0.022 Hz |
| Nutation (deployed, layout B) | ≈ 0.004 Hz |
| Control law bandwidth (momentum gain 0.005 s⁻¹) | ≈ 0.0008 Hz |
| Control and sampling rate | 1 Hz (coil on 900 ms, off for the magnetometer sample) |

The appendages are 25 times (tape, flat) to 5,000 times (wing) faster than the spin and the control, so the rigid-body
control model holds. The 1 Hz coil switching sits between the tape's two directions, but the coil torque (≈ 6 µN·m)
moves a 0.5 m tape tip by about 0.1 mm. What the appendages do matter for is energy dissipation: they make the spin
stable only about the major axis of inertia. That is why the inertia ratio of MASS_BUDGET §6 is a requirement.

## 10. Verification Items

| ID | Verification | Pass criterion | Verifies | When |
|----|--------------|----------------|----------|------|
| VI-STR-04 | Hinge torque test of each wing (EM, then flight): spring torque over the travel, pin friction and clock-spring flex torque at −40, +20 and +60 °C in vacuum, each spring removed in turn | TM ≥ 0 with the 5017B Table 1 qualification-test factors (EM) and acceptance factors (flight, DDMR 11); one-spring-out ≥ 0; flex torque ≤ 0.50 mN·m at −40 °C (zero margin at 0.64) | §4A, STR-13 | EM; flight acceptance |
| VI-MECH-01 | Spring-lot test: torque–angle of 5 springs of each type at −40/+20/+60 °C; stress relaxation held at the stowed angle for 30 days at +60 °C; then 50 cycles | Rate and torque within ±10 %; relaxation ≤ 5 %; no set after cycling | §3A (A.2.5.1) | Before EM assembly |
| VI-MECH-02 | Clock-spring flex: 50 deployment cycles at −40 °C with continuity and insulation monitored; torque logged | No open or short; torque ≤ 0.50 mN·m | §3A | EM |
| VI-MECH-03 | A286 pin with MoS₂ in the Ti-6Al-4V bushing: friction torque at −40 / +20 / +60 °C in vacuum before and after 50 deployment cycles and after protoflight random; bore and pin inspected | μ ≤ 0.15 cold throughout (zero margin at 0.26); no galling or film breakthrough | §5A (pin), §4A | EM |
| VI-STR-05 | Deployment test, gravity off-loaded, high-speed video and an accelerometer on the wing, at ambient and at −40 °C and +60 °C; EL images of the cells before and after; repeated after protoflight random (TP-ENV-005) | Deploys within 5 s; stop torque ≤ 0.3 N·m; no cell cracks; rebound settles within 2 s | §5, STR-14 | EM (qual), flight after vibration |
| VI-STR-06 | Burn-wire characterisation: ≥ 30 coupons of the flight line, resistor and preload at −40 °C and +20 °C in vacuum; then the flight energy setting | Flight energy per attempt ≥ 1.5 × (mean + 3σ) of the release energy at the minimum predicted temperature − 10 °C | §6, STR-15 | EM |
| VI-STR-07 | Hold-down line: proof test of each flight line at 1.2 × limit (5001B Table 6) and a 30-day creep test at the preload | No failure; preload loss ≤ 30 % | STRUCTURAL_ANALYSIS §7, STR-17 | EM; each flight line |
| VI-STR-10 | Deployment clearance: CAD sweep of both wings and all four AntS elements in the flight order, then the deployment test with the AntS | ≥ 10 mm clearance at every point of both sweeps | §7.2, STR-16 | CAD release; EM |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR structures review | Initial issue: wing torque margins (NASA-STD-5017B), spring sizing, deployment dynamics and end-of-travel shock, burn-wire energy, AntS interface and clocking, partial deployment, flexible modes, verification items |
| B | 2026-10-27 | Mechanical design (SQSO-MECH-CAD-001, MCAD-01) | Version A springs and R ≥ 5 mm flex shown not to fit the CDS envelope at the hinge line; new hinge (§3A): primary spring on the axis below the wing, secondary at +Z, MoS₂ pins, clock-spring flex at R 1.0–1.6 mm, PTFE-faced snubber, damped soft stop, no latch; torque margins recomputed (§4A: +0.14 cold end of travel, one spring out +0.13, springs +0.12 / +0.04 on yield); dynamics, stop and hold (§5A); VI-STR-04 amended, VI-MECH-01 and -02 added (`hardware/mechanical/hinge.py`) |
| C | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: spin 8°/s (BASELINE Rev E §8) in the hinge-load statements and the frequency table (spin 0.022 Hz, nutation ≈ 0.004 Hz, scaled with the spin); hold-down drivers and the burn pack current from the EPS board (one 0.8 A driver per resistor, 0.67 A at the pack; SQSO-EPS-BRD-001 §9) |
| D | 2026-10-27 | Mechanical design (SQSO-MECH-CAD-001, magnetometer zone) | Hinge pins 440C → A286 (non-ferromagnetic, 20 mm zone round the magnetometer board A2) with sputtered MoS₂ on the pin and burnished binder-free MoS₂ in the bore; pin strength at the end-of-travel impact and at launch (§5A: MS +3.3 yield, +4.6 ultimate); torque margins unchanged; VI-MECH-03 (friction and galling with the A286 pin) |
