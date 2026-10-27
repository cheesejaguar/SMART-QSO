# SMART-QSO System Safety Data Package

**Document ID**: SQSO-SDP-002
**Version**: 1.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: **CDR draft. System hazard analysis complete at the level of the CDR design; no hazard is closed.** Every
control below is designed (or, where marked, missing from the design); none is verified. Each hazard closes only with
the verification evidence its row names.

> **Read this first.** This is the **system** safety data package a launch provider and its range-safety authority
> review: the spacecraft, its ground support equipment (GSE) and the hazardous operations from first battery
> installation to on-orbit deployment, for the baseline path (SpaceX Transporter rideshare through a 1U integrator,
> BASELINE §3, decision D7) and the backup (ISS deployment from the Voyager NRCSD). The
> [software safety data package](SAFETY_DATA_PACKAGE.md) (SQSO-SDP-001) feeds §11; software is never counted as an
> inhibit here (CDS Rev 14.1 §2.3.7.2; SpaceX UG v11 §6.7.7).
>
> The integrator is SpaceX's customer and will tailor these requirements; its user guide and ICD supersede this
> package where they differ. Requirement flow-down to the launch vehicle and dispenser is in
> [LAUNCH_VEHICLE_REQUIREMENTS.md](../mission/LAUNCH_VEHICLE_REQUIREMENTS.md) (SQSO-LVR-001); its verification items
> (VI-LV-nn) are referenced, not repeated.

## 0. CDR Summary

| Result | Value | Basis |
|--------|-------|-------|
| Hazards analysed | 16 (§5); HR-01, HR-02, HR-03, HR-05, HR-06 and HR-16 are catastrophic in at least one phase | §4 method |
| Catastrophic hazard causes meeting their required tolerance **as designed** | External short (two-fault tolerant below the fuse); structural release and materials (design for minimum risk, analysis and test open). **Not met:** RF before 45 min and premature deployment (one inhibit short of three); overcharge (single-fault tolerant until the secondary protector EPS-11 is added); internal short after deep discharge (no recharge lockout) | §6 |
| Gaps that block the safety review | (1) third RF and deployable inhibit not designed (S1 "Inhibit 3 (TBD)"); (2) the shared wing burn line draws ≈ 5.8 A against the 5 A pack fuse and BMS trip, with no hardware on-time limit; (3) no recharge lockout after a deep over-discharge and no dissimilar secondary over-voltage protector (EPS-11, EPS-12); (4) cell lot acceptance below JSC 20793 practice in BATTERY_DESIGN (SQSO-BSA-001 §4 fixes it); (5) UN 38.3 status of a one-off pack for shipping | §6, §12 RIDs |
| RF exposure to personnel | 1 W at 145.9 MHz is exempt from routine evaluation beyond **0.65 m** (47 CFR 1.1307(b)(3)(i)(C)); radiating tests at ≥ 1 m or into a load | HR-09 |
| Accidental-explosion probability on orbit | **1.4 × 10⁻⁴** (< 10⁻³; NASA-STD-8719.14C Req. 4.4-1), 9 × 10⁻⁵ of it from over-discharge then recharge, which a recharge lockout removes | ODAR §4; `software/simulation/cdr/sma/explosion_probability.py` |

## 1. Purpose, Scope and Phases

**Purpose.** Show that SMART-QSO creates no unacceptable hazard to people (developer staff, test-house staff,
integrator and launch-site personnel, and ISS crew on the backup path), the launch vehicle, co-payloads, the dispenser
and facilities.

**Scope.** Flight hardware, flight software as a contributor, GSE (battery charger, EGSE, key-load umbilical, deployment
test fixtures) and these phases:

| Phase | Where | Configuration | Who is exposed |
|-------|-------|---------------|----------------|
| P1 Assembly, integration and test | Developer and test house | Battery installed, RBF in except during powered tests; antennas and wings stowed or deployed in tests | Developer staff, test-house staff |
| P2 Shipping | Road or air | Unpowered, RBF in, battery at 60 % ± 5 % SOC (LVR-051) | Carriers |
| P3 Integrator facility and launch site (PPF) | Integrator; SpaceX PPF, Vandenberg | Dispenser loading: RBF removed, switches actuated by the dispenser rails | Integrator, SpaceX and range personnel |
| P4 Launch and ascent | Falcon 9 fairing | Unpowered inside the closed dispenser | Launch vehicle, co-payloads |
| P5 Separation to 45 min | Free flight | Powered; deployables at ≥ 30 min, RF at ≥ 45 min | Launch vehicle, co-payloads (re-contact, RF) |
| P6 Backup only: ISS cargo, JEM airlock, NRCSD deployment | Pressurized cargo vehicle, ISS cabin, JEM | Unpowered in the NRCSD | **ISS crew** |

## 2. Applicable Documents

| Document | Use | Obtained? |
|----------|-----|-----------|
| SpaceX Rideshare Payload User's Guide, **Version 11, August 2026** ("UG v11", <https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf>) | §3.3.4 (appendage deployment ≥ 120 s, hard-body radius), §4.1.7.3 and Table 4-10 (transmitter turn-on delay), §5.2 (fasteners), §5.3 and Table 5-5 (materials, prohibited materials), §5.5.4 (sealed containers, batteries), §5.7 (dispensers, no passive appendage deployment), §6.7.7 (power inhibits verified during random vibration), §7.6 (all systems off during launch), §9.5.2–9.5.3 (no open-air RF checkout; hazardous procedures), §11.2 (hazardous systems), Appendices D, F, G | Yes (read 2026-10-27) |
| Space Systems Command Manual (SSCMAN) 91-710, Range User's Manual, as tailored by SpaceX (UG v11 §11.1) | Range safety design and operations requirements, including the hazard classification and failure tolerance for flight hardware | **No.** Not publicly obtainable here. This package applies the conventional rule: catastrophic hazards two-fault tolerant, critical hazards single-fault tolerant, or design for minimum risk where tolerance is impractical (the rule UG v11 §5.5.7 states for propellant systems, and NSTS 1700.7B §200.1a, the origin of SSP 51700's) |
| CubeSat Design Specification **Rev 14.1**, Cal Poly, 2022-02 | §2.1.7 materials, §2.1.9 venting, §2.3.1–2.3.8 power off, switches, RBF, battery protection, inhibits, §2.4.4–2.4.5 timers | Yes |
| [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) Rev D | Design values | Yes |
| 47 CFR 1.1307(b)(3) and 1.1310 | RF exposure exemption and limits | Yes (eCFR via Cornell LII, 2026-10-27) |
| 49 CFR 172.101, 173.185; UN Manual of Tests and Criteria §38.3 | Hazardous-materials declaration; lithium-battery transport | Cited; test status open (HR-15) |
| IEC 62133-2 / UL 1642 | Cell safety listing of the INR18650-35E (CDS §2.3.6 "extra documentation for non-UL cells") | Cell listing to be confirmed from the cell certificate |
| **ISS backup:** SSP 51700 (Payload Safety Policy and Requirements for the ISS), JSC 20793 Rev D (Crewed Space Vehicle Battery Safety Requirements), SSP 30599 (Safety Review Process), NASA-STD-6001B (flammability, offgassing), Voyager/Nanoracks NRCSD interface document | Appendix B | **No** (documents not obtained; requirements summarised from their published scope; confirm with Voyager before the Phase 0/I review) |
| [SAFETY_DATA_PACKAGE.md](SAFETY_DATA_PACKAGE.md) SQSO-SDP-001 | Software contributions | Yes |
| [SYSTEM_FMEA_CIL.md](SYSTEM_FMEA_CIL.md) SQSO-FMEA-002 | Hardware failure modes behind the causes below | Yes (this review) |
| [BATTERY_SAFETY_ASSESSMENT.md](../environment/BATTERY_SAFETY_ASSESSMENT.md) SQSO-BSA-001 | Battery hazard causes H1–H9, screening, SOC, UN 38.3, V-BSA items | Yes (EPS review, 2026-10-27) |

## 3. Safety-Relevant Description

### 3.1 Energy sources and hazardous materials

| Source | Quantity | Notes |
|--------|----------|-------|
| Li-ion battery | 4S1P Samsung INR18650-35E, 3.45 Ah, 14.4 V nominal, **49.7 Wh (179 kJ) at 4.20 V/cell; ≈ 45 Wh at the 4.10 V/cell set point (D8)**; 60 % ± 5 % SOC at integration | Sealed steel cans with a current-interrupt device (CID) and vent; ≈ 12.4 Wh a cell. Below the 80 Wh and 100 Wh thresholds noted in BASELINE §5 |
| Mechanical springs | Two wing hinges (torsion springs), four AntS tape elements (stored strain), dispenser ejection spring (the dispenser's) | Hinge spring not yet selected; AntS is a flown product |
| Burn-wire heaters | Two release resistors (≈ 5 Ω each) per channel on one driver (schematic S5) | At 14.4 V each draws ≈ 2.9 A, ≈ 41 W, unless limited (S5 note F7) |
| RF | One 2 m transmitter, ≤ 1 W into a quarter-wave monopole (≤ 1.6 W EIRP) | The 435 MHz radio and the GNSS are receivers |
| AI payload | Jetson Orin Nano module, ≤ 8 W input, power-gated by the OBC | Powered only on orbit in sunlight; flatsat and TVAC on the ground |
| Heater | ≈ 1 W polyimide film on the pack, hardware cut-out at ≈ +40 °C, fused | BUILD_VS_BUY §4.3 |
| Magnetorquers | Air-core coils or CubeSpace CR0002 rods, ≤ 0.2 A·m² | Unpowered until deployment; rod remanence to measure (LVR-085) |
| Electrolyte | ≈ 5 g a cell, LiPF₆ in carbonate solvents (flammable, corrosive on release) | Hazardous material (UG v11 Appendix F item 2; 49 CFR 172.101) |
| None | Propellant, pressure vessels (other than the sealed cells, UG v11 §5.5.4), ordnance, lasers, radioactive material | Statement to the integrator |

### 3.2 Inhibit architecture as designed at CDR

From `hardware/diagrams/schematics/S1_eps_power_path.py` and `S5_burn_wire.py`:

```
BATT+ ─ F2 5 A fuse ─ RT1 PTC ─ RBF (ground only) ─ DS1 (rail switch, VBAT+) ─ [Inhibit 3: not designed] ─ VBAT bus
BATT− (BMS CHG/DSG FETs) ─ DS2 (rail switch, return) ─ bus return
VBAT bus ─ OBC ─ TX_EN (PA supply switch, after the 45 min software timer) ─ PA ─ VHF antenna
VBAT bus ─ SW6 DEPLOY_EN (high side) ─ 2 × 5 Ω burn resistors in parallel ─ Q low-side NMOS ─ return   (one driver for both wings; one for the antennas)
Charge path: solar strings ─ MPPT ─ BATT+ (upstream of DS1/DS2; F1 end-of-life passivation fuse "non-resettable (TBD)")
```

| Path | Physical inhibits that count (CDS §2.3.7–2.3.8) | Do not count | Required | Gap |
|------|---------------------------------------------------|--------------|----------|-----|
| Battery → RF transmitter | DS1, DS2 | RBF (removed at integration), 45 min timer, TX_EN (commanded by the same OBC that runs the timer) | ≥ 3 | **1 missing** |
| Battery → each burn-wire | DS1, DS2; SW6 + Q count as **one** (both driven by the one OBC, so a single OBC fault can close both) | RBF, 30 min timer | ≥ 3 | **1 missing** unless the third rail switch is added |

**Recommended design (for the chair; EPS and structures owners):** a third independent deployment switch, DS3, on a
different rail end from DS1/DS2, in series with DS1 on VBAT+ (or on the return), as the documents for the NRCSD backup
already suggest (LAUNCH_PROVIDER_DELIVERABLES row 2.3.7). DS1–DS3 then give three independent mechanical inhibits to
every load, RF and deployables alike; SW6/Q and TX_EN remain as additional, uncounted controls. The inhibit-by-inhibit
test VI-LV-06 verifies it.

## 4. Method

- **Hazard identification.** Energy-source checklist (§3.1), the UG v11 §11.2 hazardous-system list, the CDS
  requirements, the ISS payload hazard categories (SSP 51700 heritage: battery, RF, structural, materials, sharp edges,
  touch temperature, EMI), the hardware FMEA (SQSO-FMEA-002) and the funding-audit list of missing CubeSat hazards
  (finding VV-05: inadvertent panel deployment, structural release, sharp edges, magnetic field, hazardous materials,
  ESD, venting, Jetson touch temperature).
- **Severity.** Catastrophic: death, permanently disabling injury, loss of the launch vehicle, a co-payload or the ISS,
  or major facility damage. Critical: severe injury, major damage to the vehicle, a co-payload, the dispenser or a
  facility. Marginal: minor injury or damage. Negligible: below these. Severity is set by the worst credible outcome
  **before** controls and is never lowered by controls (finding VV-05).
- **Fault tolerance required.** Catastrophic: two-fault tolerant (no two failures or operator errors cause the
  hazard), or design for minimum risk (DFMR) where tolerance is impractical (structures, materials, sealed cells).
  Critical: single-fault tolerant. Inhibits must be independent; software timers are not inhibits.
- **Verification.** Each control names its method (T test, A analysis, I inspection, D demonstration), its procedure or
  verification item and a pass criterion. Status is "Open" until the evidence is filed.

## 5. Hazard Reports

### HR-01 Battery fire, venting or thermal runaway

| Field | Entry |
|-------|-------|
| Phases, severity | P1–P4 **Critical** (facility, personnel near the cell, co-payloads in the dispenser); P6 **Catastrophic** (crewed cabin) |
| Causes | (a) Overcharge: charger regulation fails high; BMS over-voltage fails; (b) external short: harness chafe or pinch at the isolated mount, conductive debris, test error at the GSE; (c) internal short: latent cell defect, mechanical damage, lithium plating from charging below 0 °C, copper dissolution after deep over-discharge and recharge; (d) over-temperature: heater stuck on, Jetson or burn-wire heating, test chamber fault; (e) cell-to-cell propagation after one cell vents |
| Controls | The battery hazard causes, controls and their tolerance are analysed in detail in the EPS review's [BATTERY_SAFETY_ASSESSMENT.md](../environment/BATTERY_SAFETY_ASSESSMENT.md) (SQSO-BSA-001, H1–H9); this report carries the system view and agrees with it. **(a) overcharge:** C1 charger CV at 4.10 V/cell (D8); C2 BMS (BQ76920 class) per-cell over-voltage ≤ 4.25 V opening the CHG FET; C3 a **dissimilar secondary protector** (4.35 V/cell, fixed threshold, three-terminal fuse), which is **not yet in the design** (RID EPS-11; BATTERY_DESIGN's "hardware comparator" is the BMS AFE itself, so C1 and C2 both depend on firmware-written settings); cell CID is a further, uncounted barrier. **(b) external short:** C4 non-resettable 5 A pack fuse F2; C5 PTC RT1; C6 BMS short-circuit trip (< 10 µs); C7 cell-tap leads through 1 kΩ (S1) so a tap-harness short is current-limited; insulated, staked harness; insulated tools and the GSE charger fused at ≤ 1 A. **(c) internal short:** C8 cell lot acceptance (§7.1; VI-SAF-03); C9 charge inhibited in hardware outside 0–45 °C (SYS-602); C10 **recharge lockout after any cell < 2.0 V (not in the design: RID EPS-12, which SMA-06 supports)** and the independent hardware low-voltage load disconnect (EPS-09); vibration of the pack at qualification levels. **(d) over-temperature:** C11 heater hardware cut-out at ≈ +40 °C plus a fuse; C12 Jetson bursts only with OBC-enforced current and temperature trips; C13 burn-wire on-time limited in hardware (**not in the design: RID SMA-05**). **(e) propagation:** pack mounted in an aluminium clamp with spacing; vent path out of the pack enclosure (VI-LV-12) |
| Fault tolerance | (a) **single-fault tolerant today; two-fault tolerant with C3** (EPS-11) if C1–C3 are independent (different ICs, sense lines and supply); (b) two-fault tolerant downstream of F2; upstream of F2 only C7 (single) → DFMR by insulation and staking; (c) DFMR (screening, temperature inhibit), with C10 missing; (d) two-fault for the heater (thermostat, cut-out, fuse); single for the burn line until C13 exists; (e) not analysed |
| Verification | V-BSA-1 cell lot screening, V-BSA-2 protection trip tests, V-BSA-3 pack acceptance, V-BSA-4 pack-to-frame isolation (SQSO-BSA-001 §9); VI-SAF-02 independence review of C1–C3 on the EPS schematic; VI-LV-11 (cell screening and pack acceptance, LVR-070, LVR-073); VI-LV-14 (battery-side drain ≤ 20 µA); VI-SAF-04 burn-line current and on-time test |
| Status | **Open.** C3, C10 and C13 missing from the design |

### HR-02 RF emission before separation or within 45 minutes

| Field | Entry |
|-------|-------|
| Phases, severity | P3–P5 **Catastrophic** (treated so by CDS §2.3.7: interference with launch-vehicle or range systems, co-payload receivers); P1 marginal (test staff, see HR-09) |
| Causes | Inhibit switch welded or bypassed; switch bounce in vibration powering the bus; software timer fault; OBC fault commanding TX_EN; RBF used as an inhibit |
| Controls | C1 DS1; C2 DS2; C3 **DS3 (proposed, §3.2)**; uncounted: 45 min RF-silence timer with the radio unpowered (`rf_silence.rs`), TX_EN hardware PA switch, RBF on the ground; no open-air RF checkout during integration (UG v11 §9.5.2) |
| Fault tolerance | **Single-fault tolerant as designed** (two counted inhibits). Two-fault tolerant with DS3 |
| Verification | VI-LV-06 inhibit-by-inhibit test; VI-LV-04 power-inhibit monitoring during random vibration with the battery at flight SOC and RBF out (UG v11 §6.7.7); VI-LV-07 timer test; VI-LV-05 switch toggle resets |
| Status | **Open (design gap: DS3)** |

### HR-03 Premature deployment of a wing or antenna

| Field | Entry |
|-------|-------|
| Phases, severity | P3–P4 **Catastrophic** in a shared dispenser (a deployed appendage can jam the door or a neighbouring CubeSat's ejection, or strike a co-payload; CDS §2.3.8; UG v11 §5.7 "passive deployment of secondary appendages … not allowed"); P1 marginal (pinch, eye injury from a tape element) |
| Causes | Burn circuit energized (inhibit failure as HR-02); restraint line creep or cut by a sharp edge in vibration; hinge spring overcomes a slack line; AntS release fault |
| Controls | C1–C3 DS1, DS2, DS3 (proposed) in the burn supply; C4 SW6 + Q (one counted control, see §3.2); C5 restraint line pre-tension and edge radii in the hold-down design; C6 AntS's own redundant release; 30 min timer (CDS §2.4.4; UG v11 §3.3.4 requires ≥ 120 s) not counted |
| Fault tolerance | Electrical: single-fault tolerant as designed, two-fault with DS3. Mechanical release under vibration: DFMR (line strength factor ≥ 2 on the spring load; vibration test) |
| Verification | VI-LV-06 (burn lines); VI-LV-04 (stowed through random vibration, inspected after); VI-SAF-05 restraint-line tension and strength test; VI-LV-17 post-environment deployment |
| Status | **Open** |

### HR-04 Stored mechanical energy during handling

| Field | Entry |
|-------|-------|
| Severity | Marginal (finger pinch at a hinge; tape element striking an eye during a deployment test) |
| Controls | Deployment tests behind a guard with eye protection; stowing tools; procedures list the stowed-energy items; AntS handling per its manual |
| Verification | I: procedure review (TP-MECH, TP-ENV-005) |
| Status | Open |

### HR-05 Structural failure or release of parts during launch

| Field | Entry |
|-------|-------|
| Severity | **Catastrophic** in a shared dispenser (a loose part can jam the dispenser or another CubeSat) |
| Causes | Fastener back-out (notably the low-preload Ti-6Al-4V screws in polymer bushings of the isolated battery mount, where creep and thermal cycling reduce preload: BUILD_VS_BUY §4.3); bracket or wing hold-down failure; Jetson module or heat spreader detaching; staking cracking |
| Controls | DFMR: factors of safety 1.10 yield and 1.25 ultimate for flight (UG v11 Table 5-1); locking features on every fastener (UG v11 §5.2; NASA-STD-5020B practice): staking plus a prevailing-torque or locking insert, never thread-locker alone on a polymer-bushed joint; bought 1U frame with CDS rails; protoflight random vibration at 5.57 Grms with sine sweeps before and after |
| Verification | VI-LV-03, VI-LV-04; VI-SAF-06 fastener list with locking feature and torque; inspection after vibration (no loose parts, staking intact, torque stripe unbroken) |
| Status | Open (no structural analysis yet, TB-15) |

### HR-06 Materials: contamination, prohibited materials, flammability, offgassing

| Field | Entry |
|-------|-------|
| Severity | Marginal for co-payload contamination; **Catastrophic on P6** for flammability and toxic offgassing in the crew cabin |
| Controls | Every non-metallic material TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS §2.1.7; UG v11 RML ≤ 1.0 %), bakeout in TVAC (LVR-084); **UG v11 Table 5-5 prohibited materials apply to a containerized CubeSat** (§5.3 exempts only four other rows): no cadmium or cadmium plating, no zinc plating, no mercury, no pure tin or tin electroplate (see `docs/compliance/MATERIALS_LIST.md` Rev B §16 and `docs/manufacturing/EEE_PARTS_LIST.md` Rev C §2.3); no Sharpie markers (UG v11 Table 5-5) |
| Verification | VI-LV-09 materials list and contamination report; VI-SAF-07 prohibited-materials declaration with each EEE part's terminal finish |
| Status | Open (materials list has unsourced values and one known non-compliant item, Loctite 242: MATERIALS_LIST Rev B) |

### HR-07 Electrical hazards to personnel and GSE faults

| Field | Entry |
|-------|-------|
| Severity | Critical (burns from a short across the 16.4 V, 8 A-capable pack; charger fault overcharging the pack) |
| Controls | Insulated tools; jewellery removed; GSE charger with its own over-voltage limit at 16.4 V, a timer and a ≤ 1 A fuse; charging only with the pack temperature monitored and in a fire-safe area (metal tray, no flammables within 1 m); RBF in except for powered tests |
| Verification | VI-SAF-08 GSE acceptance (charger OV limit and fuse), procedure review |
| Status | Open |

### HR-08 Jetson Orin Nano: heat, fire, back-powering

| Field | Entry |
|-------|-------|
| Severity | Marginal on the ground (touch temperature of the heat spreader during flatsat runs; carrier fault overheating) |
| Controls | Ground runs ≤ 120 s per burst as in flight, OBC per-rail current and temperature trips independent of the Jetson; thermocouple on the spreader in every test; 74LVC1T45 Ioff translators against back-powering (S7, finding F13); powered off during launch (UG v11 §7.6) |
| Verification | Flatsat procedure; EM thermal test (TB-07) |
| Status | Open |

### HR-09 RF exposure of personnel

| Field | Entry |
|-------|-------|
| Severity | Negligible to marginal |
| Analysis | 47 CFR 1.1307(b)(3)(i)(C), Table 1: at 30–300 MHz a source is exempt from routine evaluation when ERP ≤ 3.83 R² W, for R ≥ λ/2π (0.33 m at 145.9 MHz). At 1 W into a monopole with up to +2 dBd gain (ERP ≤ 1.6 W) the exemption distance is **R ≥ 0.65 m**. The 435 MHz radio does not transmit |
| Controls | Radiating tests only into a dummy load or a shielded enclosure, or with personnel ≥ 1 m from the deployed VHF element; no open-air RF at the launch site (UG v11 §9.5.2) |
| Verification | Procedure review (TP-RF) |
| Status | Open |

### HR-10 Inadvertent deployment or RF after separation causing re-contact

| Field | Entry |
|-------|-------|
| Severity | Critical (re-contact with the stage or a co-payload) |
| Controls | No propulsion; appendages deploy at ≥ 30 min (UG v11 §3.3.4 asks ≥ 120 s); the deployed hard-body radius (≈ 0.6 m with the 0.5 m VHF tape) reported to SpaceX, since deployment is within 24 h (UG v11 §3.3.4); ephemeris per UG v11 §9.5.7.2 (LVR-089) |
| Verification | VI-LV-07; hard-body radius in the RF/deployment webform (VI-LV-10) |
| Status | Open |

### HR-11 Software contribution

Software provides the 30 min deployment and 45 min RF timers, TX-OFF, the Jetson burst limits and the secondary battery
protections. No launch hazard control depends on it (§3.2). Software hazards are in SQSO-SDP-001 and
`docs/safety/SOFTWARE_HAZARD_ANALYSIS.md`, all open.

### HR-12 Magnetic field

Negligible. No permanent magnets; rods (if bought) checked for remanence ≤ 0.5 G outside the envelope (CDS §2.1.8,
VI-LV-18).

### HR-13 Sharp edges and handling

Marginal (ground staff; ISS crew do not handle the CubeSat outside the NRCSD). Edges ≥ 1 mm radius (LVR-023), deburred
panels, inspection.

### HR-14 Venting and sealed volumes

| Field | Entry |
|-------|-------|
| Severity | Critical (a sealed enclosure bursting at ascent depressurization; the cells themselves) |
| Controls | Open 1U frame; every enclosed sub-volume (battery box, Jetson enclosure) vented to ≤ 50.8 m volume/vent-area (CDS §2.1.9, LVR-048); the cells are "other pressurized equipment" under UG v11 §5.5.4: meet Table 5-6 factors or a vacuum test (**LVR-073**), done as part of cell screening (VI-LV-11) |
| Verification | VI-LV-12 venting analysis; VI-LV-11 cell vacuum test (mass loss and OCV before and after, no leak) |
| Status | Open |

### HR-15 Lithium battery in transport and as a hazardous material

| Field | Entry |
|-------|-------|
| Severity | Critical (fire in transport) |
| Requirement | Hazardous material declared against 49 CFR 172.101 for the FAA payload review and the UG v11 Appendix F certification (**LVR-072**); lithium-ion transport per 49 CFR 173.185: the battery design must pass UN 38.3, or ship under the prototype/low-production provision (173.185(e)) with its packaging limits; ≈ 50 Wh installed in equipment |
| Controls | Cells with a UN 38.3 test summary from the manufacturer; the **pack** (a one-off 4S1P) needs either its own UN 38.3 tests or the 173.185(e) route; ship at 60 % ± 5 % SOC, RBF in, in a fire-resistant container; carrier and integrator informed |
| Verification | VI-SAF-09 shipping classification memo with the UN 38.3 test summary or the 173.185(e) approval route |
| Status | **Open** |

### HR-16 ISS backup: crew cabin exposure

Catastrophic (battery and materials in a crewed volume). See Appendix B.

## 6. Fault-Tolerance Summary

| Hazard | Severity (worst phase) | Required | Achieved as designed | With the recommended changes |
|--------|------------------------|----------|----------------------|------------------------------|
| HR-01a Overcharge | Catastrophic (P6) / Critical | 2-FT | **1-FT** (C1, C2) | 2-FT with the secondary protector (EPS-11), if independent (VI-SAF-02) |
| HR-01b External short | Catastrophic (P6) / Critical | 2-FT | 2-FT below F2; DFMR above | Same |
| HR-01c Internal short | Catastrophic (P6) / Critical | DFMR | DFMR without the recharge lockout | DFMR with C10 and JSC-20793-style screening |
| HR-01d Over-temperature | Critical | 1-FT | Heater 2-FT; burn line **0-FT** (no on-time limit, and its current exceeds the fuse) | 1-FT with C13 |
| HR-02 RF before 45 min | Catastrophic | 2-FT (3 inhibits) | **1-FT** | 2-FT with DS3 |
| HR-03 Premature deployment | Catastrophic | 2-FT (3 inhibits) | **1-FT** | 2-FT with DS3 |
| HR-05 Structural release | Catastrophic | DFMR | DFMR (analysis and test open) | DFMR |
| HR-06 Materials (P6) | Catastrophic | DFMR | DFMR (data incomplete) | DFMR |
| HR-14 Venting | Critical | DFMR | DFMR (analysis open) | DFMR |

## 7. Verification Items Defined by This Package

These are in addition to the VI-LV items of SQSO-LVR-001 §10.

| ID | Test or analysis | Pass criterion | Verifies | When |
|----|------------------|----------------|----------|------|
| VI-SAF-01 | (Covered by V-BSA-2 of SQSO-BSA-001: each protection tripped alone, including the secondary protector on the EM) | Trip within ±25 mV and ±2 °C of the setting | HR-01 | EM, then flight pack |
| VI-SAF-02 | Independence review of the overcharge controls on the EPS schematic | C1, C2, C3 use different ICs, different sense paths and no common supply or software | HR-01a | EPS CDR, before PCB release |
| VI-SAF-03 | (Covered by V-BSA-1 and V-BSA-3 of SQSO-BSA-001: JSC 20793 Rev D §4.2.3 and §6.4.3.3 screening of one lot, ±3σ, pack vibration and thermal vacuum with mass and OCV change < 0.5 %, which is also the UG v11 §5.5.4 vacuum test of the sealed cells, LVR-073) | SQSO-BSA-001 §4 | HR-01c, HR-14 | Before the EM pack; flight pack |
| VI-SAF-04 | Burn-line test on the EM: current, on-time, behaviour with the driver forced on | Current per resistor within the design limit (0.5–1 A, S5 F7); total burn current below the BMS and fuse trips with ≥ 30 % margin; hardware cut-off ≤ the design on-time (proposed 10 s) with the OBC driver held on | HR-01d, HR-03 | EM |
| VI-SAF-05 | Restraint-line tension and break test | Line strength ≥ 2× the hinge spring's maximum load after thermal cycling; tension within ± 10 % of the drawing | HR-03 C5 | EM, then flight unit |
| VI-SAF-06 | Fastener and locking-feature list, torque table, inspection after vibration | Every fastener has a locking feature (UG v11 §5.2); none loosened; staking uncracked | HR-05 | With VI-LV-04 |
| VI-SAF-07 | Prohibited-materials and terminal-finish declaration | No cadmium, zinc plating, mercury; every pure-tin-finished part re-tinned with Sn-Pb or coated per MATERIALS_LIST Rev B §16 | HR-06 | L-4 M, with VI-LV-09 |
| VI-SAF-08 | GSE charger acceptance | OV limit 16.40 V ± 0.05 V independent of the controller; fuse ≤ 1 A; timer | HR-07 | Before the first flight-pack charge |
| VI-SAF-09 | Battery shipping classification and the hazardous-materials declaration: UN 38.3 pack tests (V-BSA-5, SQSO-BSA-001 §8) or the 49 CFR 173.185(e) prototype route; 49 CFR 172.101 entry for the UG v11 Appendix F certification | Test summary issued or the route agreed with the carrier and the integrator; declaration accepted | HR-15, LVR-072 | Before the first shipment of a pack; L-6 M for the declaration |

## 8. Battery Safety Program Summary

- **Cells.** Samsung INR18650-35E (named in `hardware/eps/BATTERY_DESIGN.md` v2.0; the repository names three other cells
  elsewhere, finding TB-08: freeze the part number). Buy from an authorized distributor, one lot, with the manufacturer's
  UN 38.3 summary and safety listing; screening and acceptance per SQSO-BSA-001 §4.
- **Pack.** 4S1P with BMS, fuse, PTC, cell-tap resistors (secondary protector to add); charged to 4.10 V/cell;
  integration SOC 60 % ± 5 % (LVR-051) or 50 % ± 10 % (SQSO-BSA-001 §6), to be made one value (SMA-10); charged only
  inside 0–45 °C.
- **Launch-site charging.** UG v11 Appendix D scenario A (no charging after stand-alone operations): the pack must stay
  flight-ready for ≥ 45 days encapsulated (LVR-071).
- **Missing design elements** (RIDs SMA-05, SMA-06; EPS-09, EPS-11, EPS-12 of SQSO-BSA-001): burn-line current and
  on-time limit; dissimilar secondary over-voltage protector; recharge lockout after deep over-discharge; hardware
  low-voltage load disconnect; and the end-of-life charge-path disconnect (F1, "non-resettable (TBD)" in S1), which is a debris
  requirement (ODAR §4) rather than a launch-safety one.

## 9. Ground Operations and Hazardous Procedures

| Operation | Hazards | Procedure (to be written or revised) |
|-----------|---------|-------------------------------------|
| Battery installation and first charge | HR-01, HR-07 | Battery handling and charging procedure with VI-SAF-08 GSE |
| Powered functional and RF tests | HR-02, HR-09 | Dummy load or shielded enclosure; ≥ 1 m rule |
| Deployment tests | HR-03, HR-04 | Guarded fixture, eye protection |
| TVAC and vibration | HR-01, HR-05, HR-14 | Pack temperature limits in the chamber; fire plan |
| Shipping | HR-15 | VI-SAF-09 |
| Dispenser loading (RBF removal) | HR-02, HR-03 | Integrator's procedure; 72 h notice for hazardous operations (UG v11 §9.5.2); Ground Operations Plan (UG v11 §9.5.3, Appendix G) |

## 10. Certifications and Letters

| Item | Source | Status |
|------|--------|--------|
| FAA payload review information template | UG v11 Appendix G | Not started |
| Payload licensing certification, including hazardous materials (49 CFR 172.101) | UG v11 Appendix F | Not started (LVR-072) |
| Battery charge-state certification at encapsulation | UG v11 Appendix D | Not started |
| Environmental test approach worksheet and verification report | UG v11 §6.4, Appendix G | Not started |
| Contamination report and materials list | UG v11 §6.7.12 | Not started (VI-LV-09) |

## 11. Software Safety

Summary of SQSO-SDP-001: the software hazards are open; software controls are credited for mission safety (TX-OFF,
battery and Jetson limits) and never for launch safety.

## 12. RIDs Raised by This Package

| RID | Severity | Finding | Required action | Disposition |
|-----|----------|---------|-----------------|-------------|
| SMA-01 | 1 | Third RF and deployable inhibit not designed (S1 "Inhibit 3 (TBD)"); SW6 and Q are one OBC-dependent control | Add DS3 (§3.2) | For the chair (EPS/structures) |
| SMA-05 | 1 | The two parallel 5 Ω wing burn resistors on one driver draw ≈ 5.8 A at 14.4 V, above the 5 A pack fuse F2 and the 5 A BMS trip; no hardware on-time limit | Current limit 0.5–1 A per resistor (S5 F7) and a hardware on-time cut-off; VI-SAF-04 | For the chair (EPS) |
| SMA-06 | 2 | No recharge lockout after a cell falls below ≈ 2.0 V (the precharge mode charges any cell under 3.0 V); it is the largest term of the explosion probability (ODAR §4) | Latched charge lockout (same as EPS-12), cleared only by ground command | For the chair (EPS) |
| SMA-07 | 2 | Cell screening in BATTERY_DESIGN §9.1 has no vibration, vacuum, self-discharge or lot statistics | Adopt SQSO-BSA-001 §4 (V-BSA-1, -3) | Verification item |
| SMA-08 | 2 | Pack UN 38.3 status and the 49 CFR 172.101 declaration | VI-SAF-09 (V-BSA-5) | Verification item |
| SMA-10 | 3 | Integration SOC differs between documents: 60 % ± 5 % (LVR-051, LVR-071) and 50 % ± 10 % (SQSO-BSA-001 §6) | One value in the battery program and the launch requirements | For the chair |
| SMA-09 | 3 | SSCMAN 91-710 and the ISS documents not obtained | Obtain through the integrator / Voyager | Open |

## Appendix A. Launch-Provider Deliverable Mapping

The SpaceX Statement-of-Work deliverables (UG v11 Appendix G) and their status are in
[LAUNCH_PROVIDER_DELIVERABLES.md](../regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md) Rev B §0. The safety content
of this package maps to them: hazard reports (§5) → range safety submission; §7 → environmental verification report
inputs; §9 → Ground Operations Plan; §10 → certifications.

## Appendix B. ISS Backup (Voyager NRCSD): Additional Safety Content

Triggered only if decision D7 falls back to the ISS (BASELINE §3). The CubeSat then travels in a pressurized cargo
vehicle and is handled inside the ISS, so the crew is exposed and the safety process is the ISS payload process
(SSP 51700 requirements, SSP 30599 phased safety reviews through the deployer provider).

| Topic | Additional content | SMART-QSO status |
|-------|--------------------|------------------|
| Failure tolerance | Catastrophic hazards two-fault tolerant, critical single-fault (SSP 51700, from NSTS 1700.7B §200.1) | As §6; DS3 needed |
| Battery | JSC 20793 Rev D: cell and battery qualification and lot acceptance, two-fault tolerance to overcharge, over-discharge, external short and over-temperature, venting design, and an assessment of cell-to-cell propagation; the 4S1P pack is ≈ 50 Wh, below the 80 Wh level the baseline notes for added assessment | VI-SAF-01–03; propagation not analysed; this is the cost driver BASELINE §3 records (6–9 months, R17) |
| RF and deployables | Inhibits as for the rideshare; deployer switches; no RF or deployment inside the ISS or before the deployer's minimum time | §3.2 |
| Materials | Flammability (NASA-STD-6001B Test 1 or configuration analysis), offgassing toxicity (Test 7), and the materials usage agreement for any exceedance | Not started; MATERIALS_LIST Rev B §2.3 |
| Structures | Loads and fracture control for the cargo-vehicle environment (soft-stowed); pack restraint | Not started |
| Sharp edges, touch temperature | Crew handling of the NRCSD only; CubeSat unpowered | Low risk |
| Mass limit | 2.40 kg (BASELINE §4) | Met by the MEL prediction |

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 0.1 | 2026-10-16 | SMART-QSO Team | Skeleton created (funding audit VV-20), structured on SpaceX Rideshare Payload User's Guide v11 as the interim provider |
| 1.0 | 2026-10-27 | CDR S&MA review | System hazard analysis: energy-source inventory, inhibit architecture against CDS 2.3.7–2.3.8 from schematics S1 and S5, 16 hazard reports with causes, controls, fault tolerance and verification, fault-tolerance summary, VI-SAF-01–09, battery program, ground operations, ISS backup appendix; UG v11 sections cited from the guide; LVR-070–073 reflected (HR-01, HR-14, HR-15) |
