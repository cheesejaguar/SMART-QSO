# SMART-QSO Mechanisms Analysis

**Document ID**: SQSO-MECH-001
**Version**: A
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR issue. Analysis of the two solar-wing hinges and hold-downs and of the bought ISIS AntS antenna system;
no mechanism hardware exists. Script: `software/simulation/cdr/structures/mechanisms.py`. Mass properties:
`docs/environment/MASS_BUDGET.md`; structure: `docs/environment/STRUCTURAL_ANALYSIS.md`.

## 1. Purpose and Result

| Check | Result | Section |
|-------|--------|---------|
| Wing static torque margin, NASA-STD-5017B Eq. 4-1 with the "theory or analysis" factors, worst case (cold, end of travel) | **+0.29** (start of travel +0.78; hot +0.70 to +1.20); one spring out +0.88 | §4 |
| Same with a contact (plunger) deployed-state switch at the stop | **−0.54**: the deployed sensor must be non-contact | §4 |
| Holding margin on the stop (tumbling at 50°/s; spin 6°/s puts no torque on the hinge) | +46 | §4.4 |
| Deployment time | 0.16 s from release to the stop (requirement < 5 s, `hardware/eps/SOLAR_ARRAY_DESIGN.md` §8.1) | §5 |
| End-of-travel shock | 18 rad/s at the stop: 0.85 N·m peak with a metal stop (1,090 µε at the wing root), 0.24 N·m with a soft bumper (310 µε); rebound ≈ 50° | §5 |
| Wing deployment while tumbling (15–50°/s) | No effect on the mechanism (inertial torque ≤ 0.1 mN·m); body rate after deployment scales with the inertia ratio | §5 |
| Burn-wire release energy (simulation model, flight timing) | Wing line, 5 s: 16.0 J against a 3σ need of 15.4 J at −40 °C (ratio **1.04**); 8 s gives 1.67. Antenna line, 3 s: ratio 0.63 at −40 °C, **11 % not released after three attempts** in the model; but the bought AntS has its own release electronics, so the model and the flight timing do not represent it | §6 |
| Partial deployment | One wing out: CG moves 1.4 mm in X, the major principal axis tilts 12° from +Y, generation and drag area drop; survivable; the flight software has no one-wing configuration | §8 |
| Wing and tape flexible modes vs spin and ADCS bandwidth | Wing 21–83 Hz, VHF tape 0.4–4 Hz, against a 0.017 Hz spin and a < 0.001 Hz control bandwidth: no coupling; dissipation makes the major-axis rule apply (MASS_BUDGET §6) | §9 |

Verdict for this discipline: **ready for fabrication with actions**: spring and sensor specification (§4), a soft or
damped stop (§5), a longer wing burn or a higher burn current (§6), the AntS command interface in place of the
simulated antenna burn line (§7), and the deployment order wings-then-antennas or a verified clearance (§7.2). The
torque margins rest on estimated resistive torques; the hinge torque test (VI-STR-04) replaces them.

## 2. Applicable Documents

| Document | Paragraphs used |
|----------|-----------------|
| NASA-STD-5017B (2022-12-06), *Design and Development Requirements for Mechanisms* | §4.3 Eq. 4-1 and Table 1; §4.3.1.1 DDMR 9 (worst case); §4.3.1.3 DDMR 11 (acceptance at the highest level); §4.3.2–§4.3.4 DDMR 12–14 (static, dynamic, holding margins); Appendix A.2.5 Table 4 (springs) and A.2.5.1 (stress relaxation) |
| AIAA S-114A, *Moving Mechanical Assemblies for Space and Launch Vehicles* | Cited by 5017B (its torque-margin section was aligned with S-114A-2020); not obtained for this review, 5017B used |
| NASA-STD-5001B w/Change 3 | Table 6 (softgood lines), Table 1 (metallic parts) |
| CDS Rev 14.1 | §2.2.4 (deployables constrained by the CubeSat), §2.3.8 (three inhibits per deployable), §2.4.4 (30 min wait) |
| SpaceX Rideshare Payload User's Guide v11 | §5.7 (no passive deployment of appendages on ejection), §6.3 item 7 (burn wires may be reset after test if identical and the test shows the original did not fail) |
| Mission simulation | `software/rust/sqso-sil/src/hw/deploy.rs` (release-energy model), `sqso_fsw::deployment` (burn 5 s wing / 3 s antenna, 3 attempts 60 s apart, 30 min wait) |

## 3. Mechanisms

| Mechanism | Design (proposed here where the documents are silent) |
|-----------|--------------------------------------------------------|
| Wing hinge | Two hinges per wing on the ±X/+Y edge (axis parallel to Z, at x = ±54.0, y = +39.5 mm), 300-series pins Ø1.5 mm in Al knuckles with a dry-film lubricant; a torsion spring at each hinge; a hard stop at 90° with a soft (silicone, low-outgassing) bumper; no latch, the spring preload holds the wing on the stop |
| Torsion springs | 17-7PH CH900 wire Ø0.55 mm, mean coil Ø4 mm, ≈ 22 coils, rate 3.2 mN·m/rad; 10.0 mN·m stowed (180° wind-up), 5.0 mN·m at the stop (90°); loaded in the winding direction. Bending stress at 10 mN·m: 674 MPa (Wahl factor 1.1) against ≈ 1,500 MPa (supplier allowable, to confirm): MS +0.48 on yield with the mission-critical factor 1.5, +0.80 on ultimate with 1.65 (5017B Table 4). 34 % of the wire strength at the stowed load keeps stress relaxation low (A.2.5.1) |
| Hold-down and release | One line per wing over the free edge (Vectran or Dyneema, 200 N break, 30 N preload), cut by **two** burn resistors on separate drivers (the documents have one resistor per wing on a shared line, `deploy.rs`; SOLAR_ARRAY_DESIGN §8.2 calls for two circuits); both line ends captured after the cut (no debris, CDS §2.1.1) |
| Deployed-state sensor | **Non-contact** (reed switch and magnet, or Hall sensor) at the stop: a plunger switch at the stop costs the margin (§4.3). Stowed-state contact switches released by the wing are acceptable (they push the wing out) |
| Inhibits | Three independent physical inhibits between the battery and every burn resistor (BASELINE §4, CDS §2.3.8); deployment no earlier than 30 min after switch release (CDS §2.4.4; `sqso_fsw::deployment`) |
| Antenna system | ISIS AntS 1U–3U, monopole/hybrid configuration: two 2 m and two 70 cm tape-spring elements (VHF ≤ 55 cm, UHF ≤ 17 cm per ISIS) in the plane of the −Z face; vendor release (redundant thermal knives, I²C controller, < 2 W at 3.3 V, < 3 s above +15 °C per ISIS brochures); the elements carry their own strain energy |

## 4. Wing Torque Margin

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

The spin about +Y (6°/s) produces centrifugal forces along the wing span, through the hinge line: no hinge torque.
Tumbling about X at 50°/s produces ≈ 0.06 mN·m; magnetorquer torques produce < 0.001 mN·m. Holding margin with FS_v 3.0:
8.45 / (3 × 0.06) − 1 = **+46**. The wing never leaves its stop in orbit, so its dynamics are those of the stop
(21–74 Hz, §9).

## 5. Deployment Dynamics

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
because the constant is in `sqso_fsw::deployment`. Pack current for the wing pair is 1.6 A for 5–8 s (0.10–0.15 Wh for
three attempts); 1.0 A per resistor is 2.0 A, inside the 5 A pack fuse.

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

- **Clocking:** the VHF elements must lie along ±X, parallel to the deployed wings. Along ±Y they make +Y the
  intermediate axis of inertia (MASS_BUDGET §6.3; Iyy/Izz = 0.89) and the sun-pointing spin fails (§6.4 of that document).
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
| Spin (6°/s) | 0.017 Hz |
| Nutation (deployed, layout B) | ≈ 0.003 Hz |
| Control law bandwidth (momentum gain 0.005 s⁻¹) | ≈ 0.0008 Hz |
| Control and sampling rate | 1 Hz (coil on 900 ms, off for the magnetometer sample) |

The appendages are 25 times (tape, flat) to 5,000 times (wing) faster than the spin and the control, so the rigid-body
control model holds. The 1 Hz coil switching sits between the tape's two directions, but the coil torque (≈ 6 µN·m)
moves a 0.5 m tape tip by about 0.1 mm. What the appendages do matter for is energy dissipation: they make the spin
stable only about the major axis of inertia. That is why the inertia ratio of MASS_BUDGET §6 is a requirement.

## 10. Verification Items

| ID | Verification | Pass criterion | Verifies | When |
|----|--------------|----------------|----------|------|
| VI-STR-04 | Hinge torque test of each wing (EM, then flight): spring torque over the travel, friction and jumper torque at −40, +20 and +60 °C (dry nitrogen or vacuum), one spring removed | TM ≥ 0 with the 5017B Table 1 qualification-test factors (EM) and acceptance factors (flight, DDMR 11); one-spring-out ≥ 0 | §4, STR-13 | EM; flight acceptance |
| VI-STR-05 | Deployment test, gravity off-loaded, high-speed video and an accelerometer on the wing, at ambient and at −40 °C and +60 °C; EL images of the cells before and after; repeated after protoflight random (TP-ENV-005) | Deploys within 5 s; stop torque ≤ 0.3 N·m; no cell cracks; rebound settles within 2 s | §5, STR-14 | EM (qual), flight after vibration |
| VI-STR-06 | Burn-wire characterisation: ≥ 30 coupons of the flight line, resistor and preload at −40 °C and +20 °C in vacuum; then the flight energy setting | Flight energy per attempt ≥ 1.5 × (mean + 3σ) of the release energy at the minimum predicted temperature − 10 °C | §6, STR-15 | EM |
| VI-STR-07 | Hold-down line: proof test of each flight line at 1.2 × limit (5001B Table 6) and a 30-day creep test at the preload | No failure; preload loss ≤ 30 % | STRUCTURAL_ANALYSIS §7, STR-17 | EM; each flight line |
| VI-STR-10 | Deployment clearance: CAD sweep of both wings and all four AntS elements in the flight order, then the deployment test with the AntS | ≥ 10 mm clearance at every point of both sweeps | §7.2, STR-16 | CAD release; EM |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR structures review | Initial issue: wing torque margins (NASA-STD-5017B), spring sizing, deployment dynamics and end-of-travel shock, burn-wire energy, AntS interface and clocking, partial deployment, flexible modes, verification items |
