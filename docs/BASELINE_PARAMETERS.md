# SMART-QSO Baseline Parameters (Single Source of Truth)

**Document Number:** SQSO-BL-001
**Revision:** E — *CDR 2026 chair dispositions (CR-2026-007): launch path and date, 475 km target / 485 km cap, mass properties, inhibits and deployment, 8 °/s spin, uplink windows, telecommand stations*
**Date:** 2026-10-27

---

## 0. Purpose and Rules

Before this document existed, the repository described several incompatible spacecraft: battery capacities from 7 to 50 Wh,
three flight computers, a beacon-only mission and a transponder mission, and ISS versus sun-synchronous (SSO) orbits (see
findings PI-13, PI-14 and TB-01 in [FINDINGS_REGISTER.md](funding_audit/FINDINGS_REGISTER.md)). This document fixes one
self-consistent baseline.

1. Where another document disagrees with this one, **this document governs**. The other document is wrong until it is
   updated through configuration control.
2. Values marked **[A]** are audit estimates to be replaced by analysis or measurement. Values marked **[D#]** follow a
   decision recorded in §9, with its research in `funding_audit/research/`.
3. Changes require a configuration-control record (`docs/project/CONFIGURATION_MANAGEMENT_PLAN.md`).

## 1. Identity and Organization

| Parameter | Baseline |
|-----------|----------|
| Name | SMART-QSO (Self-Managing Amateur Radio Transponder — QSO) |
| Licensing regime | FCC Part 97 amateur space station ([pathway determination](regulatory/LICENSING_PATHWAY_DETERMINATION.md)). A paid contract, if one is signed, is carried only by a separate Part 5 station with its own licensee and radio **[D2]** |
| Space station licensee / call sign | **KK6GDB** (individual amateur licensee); space station identifies as **KK6GDB-11** (`software/flight/include/station_id.h`) **[D1]** |
| Telecommand stations | **≥ 2 designated amateur stations at least 1,000 km apart, each able to command alone**, with written agreements by **2027-01-05** **[D1]**. One station alone trips the 14-day loss-of-contact inhibit in 56–67 % of simulated missions; two stations with independent outages only in 0.6 %, but **25–26 %** once common-mode ground outages are included (shared counter file, a single key holder, one licensee; SQSO-GND-001 Rev G §5). Mitigations: UTC-derived counters so that each station commands independently (SQSO-ICD-013 §5.2.1), a copy of the operational key with each station's custodian, a backup control operator for the licensee, and a spare station kit. NASA Ames Research Center and Santa Clara University are prospective partners, but they are 13 km apart and count as one site. Nothing is agreed |
| Host organization | None yet. Needed only for routes that require an institution (for example NASA CSLI eligibility, §3) **[D1]** |
| Operator compensation | None for operations (Part 97 condition C2) |
| Project phase | **Phase B (pre-PDR).** No independent PDR or CDR has been held; earlier review records are self-assessments (FINDINGS_REGISTER PI-03, PI-04). The 2026-10-27 design review against the NPR 7123.1D CDR criteria (`docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`) is an engineering assessment, not an independent board; independent PDR and CDR dates are in the integrated master schedule (`mission/SCHEDULE.md`) |
| Contracting party | A legal entity to sign the launch services agreement, the FAA cross-waivers, the insurance declarations and the export agreements, named by **2026-12-31** (SQSO-OAP-001 DP-1) |
| Payload risk classification | Class D (NPR 8705.4B), proposed |
| Software classification | **Class C** per NPR 7150.2D Appendix D (CubeSat mission software; software on a Class D payload); research and analysis tools Class D (`software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1). Safety-critical components: SDP §3.2 |

## 2. Mission

| Parameter | Baseline |
|-----------|----------|
| Primary mission | Single-channel **U/V FM transponder** for worldwide amateur contacts, **duty-gated** by available power |
| Secondary | AX.25 telemetry beacon, 1200 bps AFSK, plain text and published formats |
| Experiment | Onboard AI that schedules the transponder within the power budget and selects plain-language beacon text. Flight baseline: a small calibrated selector on the OBC choosing among pre-approved templates. Gated experiment: generative beacon text on the power-gated Jetson (§6). Falls back to a conventional transponder and template beacon **[D5]** |
| Community model updates ("federated learning") | **Not in the flight baseline.** Any future experiment treats public uploads as data only; adoption requires an authenticated telecommand (pathway C5). No onboard implementation exists (SW-11) |
| Design life | **2 years**; minimum mission **12 months** (MRD-001) |
| Success criteria | Unchanged (`docs/mission/MISSION_SUCCESS_CRITERIA.md`): L1 30 days, L2 1 year, L3 2+ years |

## 3. Orbit and Launch

| Parameter | Baseline |
|-----------|----------|
| Launch path | SpaceX Falcon 9 Transporter-class dedicated SSO rideshare from Vandenberg SLC-4E, in a 1U slot of an **Exolaunch EXOpod Nova** (a 3U slot with a non-deployable adapter, or shared with other 1U/2U spacecraft). SpaceX's customer is the integrator; SMART-QSO contracts with the integrator ([orbital access plan](ORBITAL_ACCESS.md); [requirements flow-down](mission/LAUNCH_VEHICLE_REQUIREMENTS.md)). Requests for quotes: Exolaunch (lead), ISISPACE (price check), SEOPS (US-owned; 1U support to confirm), D-Orbit and Maverick (one question each). Contract signed by **2027-06-30** **[D7]** |
| Launch date | **Planning date: the last Transporter flight of 2028 (about 2028-11)**; June 2028 only as an opportunity. A June launch misses the integrator's protoflight milestone (4 months before launch) by 80 days; November has 71 days of float against 67 wanted (`mission/SCHEDULE.md` Rev D §3, with the layout re-estimate and the reliability programme) |
| Launch cost (ROM) | **$142k–345k all-in**: integrator 1U slot $85k–150k (no provider publishes a 1U price) plus $57k–195k program-side (tests $42k–130k, export counsel $10k–40k, logistics $5k–15k, ephemeris up to $10k; SQSO-AUD-005 Rev F; CR-2026-015). Add 15 % contingency on the slot for rebooking **[D7, U]** |
| Altitude protection | SpaceX offers drop-off altitudes from a menu (500–600 km on the open 2027 flights), so the contract must state a **475 km target and 485 km hard cap**, with no-penalty rebooking if the assigned flight offers nothing at or below the cap. Ask each integrator about an orbital-transfer-vehicle drop-off in the band **[D7]** |
| Orbit | Circular SSO, inclination ≈ 97.4° |
| Deployment altitude | **Target 475 km; acceptable 465–485 km; not above 485 km** (the cap applies to the 3σ-high mean altitude of the drop-off orbit, or apogee, injection dispersion included; RFQs ask for a 460 km quote too) (FCC 5-year rule with P ≥ 0.9 under the conservative solar projection, mission counted to its planned 2 years: P ≥ 0.997; ODAR Draft D §6.4–§6.5). The rule does not say whether an early failure ends the mission; if the FCC counts it so, P at 475 km falls to 0.67–0.75, and the band becomes **460–465 km, target 460 km** (P ≥ 0.99), with the 2-year life at risk in a strong solar cycle (shortest life 2.5 yr at 460 km at 1.6 kg, 2.3 yr at the 1.49 kg CBE). Question in the FCC consultation before the first pre-space filing (2027-01-19) |
| Backup | ISS deployment (Voyager NRCSD), 400–420 km, 51.6°, ROM $120k–250k all-in; JSC 20793 battery program is the cost driver. Life 0.7–1.9 yr nominal; mission success Level 3 not achievable. Triggered if no integrator commits in writing to ≤ 485 km by 2027-06-30 or the integrator's slot price exceeds $250k (CR-2026-015); decide by 2027-09. The NRCSD needs its contract 12 months before launch, so a 2027-09 decision gives an ISS launch no earlier than about 2028-09. The backup changes the design (SQSO-OAP-001 §3; SQSO-LVR-001 §7): nothing energized for 30 min after deployment, a ground-leg inhibit, rail edge radius 0.5 ± 0.1 mm, random vibration 9.47 Grms hard-mounted, JSC 20793 battery programme **[D7]** |
| Contingency | Electron rideshare from New Zealand with a third-party 1U dispenser (altitude is a design point; price unknown; export licence). NASA CSLI only as a no-cost long shot: next solicitation 2028, launch about 2030 or later, eligibility needs an institution or nonprofit (§1) **[D7]** |
| Propulsion / station-keeping | None |
| Disposal | Passive atmospheric drag; solar wings stay deployed (they are the drag augmentation). **No separate drag device** (removed from baseline; it was never in mass, structure, software or test) |

Orbital lifetime (ODAR Draft D §6.2, deployment 2028-11-01, `software/simulation/cdr/sma/debris_lifetime.py`; NRLMSIS 2.1; NASA-STD-8719.14C
cross-section: 0.0247 m² wings deployed, 0.0164 m² wings failed; Cd 2.2; nominal, conservative and bound solar
projections). Total life in years (life after the 2-year mission in brackets); **bold** exceeds the FCC 5-year rule.
Must be reproduced in NASA DAS before the early pre-space filing of 2027-01-19 (ODA-02: agreement requested 2026-11-06, DAS run by 2027-01-08):

| Deployment | Mass | Wings deployed: nominal / conservative / bound | Wings failed: nominal / conservative / bound |
|------------|------|-----------------------------------------------|---------------------------------------------|
| 450 km | 1.6 kg | 3.2 (1.2) / 3.7 (1.7) / 3.7 (1.7) | 4.5 (2.5) / 5.0 (3.0) / 5.5 (3.5) |
| 465 km | 1.85 kg | 4.7 (2.7) / 5.2 (3.2) / 5.9 (3.9) | 5.6 (3.6) / 6.4 (4.4) / **8.9 (6.9)** |
| 475 km | 1.6 kg | 4.8 (2.8) / 5.3 (3.3) / 6.3 (4.3) | 5.7 (3.7) / 6.6 (4.6) / **9.5 (7.5)** |
| 475 km | 1.85 kg | 5.1 (3.1) / 5.7 (3.7) / **7.3 (5.3)** | 6.1 (4.1) / **7.1 (5.1)** / **10.9 (8.9)** |
| 485 km | 1.85 kg | 5.6 (3.6) / 6.3 (4.3) / **9.0 (7.0)** | 6.5 (4.5) / **7.9 (5.9)** / **13.5 (11.5)** |
| 500 km | 1.85 kg | 6.2 (4.2) / **7.3 (5.3)** / **12.2 (10.2)** | **7.3 (5.3)** / **9.6 (7.6)** / **18.3 (16.3)** |
| 525 km | 1.85 kg | **7.3 (5.3)** / **10.1 (8.1)** / **20.0 (18.0)** | **9.3 (7.3)** / **16.4 (14.4)** / **30.1 (28.1)** |

The wings-failed probability is 0.29 % with four independent burn drivers and their shared supply (1.6 % with one driver for both; ODAR Draft D §6.4, SQSO-FMEA-002
HF-13), so the wings-deployed columns govern the P ≥ 0.9 test.

## 4. Spacecraft Physical

| Parameter | Baseline |
|-----------|----------|
| Form factor | 1U, 100 × 100 × 113.5 mm (CDS Rev 14.1) |
| Mass | CBE **1.52 kg** (1,519 g) with CubeSpace CR0002 rods (selected; 1.60 kg with air-core coils); MEV **1.75 kg** (1,754 g), maturity-based growth 15.5 % by the AIAA S-120A-2015 method (`docs/environment/MASS_BUDGET.md` v3.1, with the CAD heat-spreader pedestal, the 20 g inertia ballast (decision C-13), the harness distribution and the battery cell retention). Design limit **2.00 kg** (SYS-002; CDS Rev 14.1): margin **+14.0 %**. Dispenser limits: EXOpod Nova 1U 2.5 kg; NRCSD backup 2.40 kg. Lifetime analyses use 1.6 kg, with **1.85 kg** as the bound |
| Mass properties | Stowed CG within 5 mm of the geometric centre (limit ±20 mm; CAD (+0.1, −0.4, −1.4) mm CBE). Deployed inertia (wings and all four AntS elements out): **Iyy ≥ 1.03 × max(Ixx, Izz)**, design target 1.05, major principal axis within 5° of +Y; CAD 1.056 CBE / 1.047 MEV, axis 1.2° / 0.7° (MEV 5th percentile 1.037 at 2 mm placement scatter), measured to ±0.5 % MOI and ±1×10⁻⁵ kg·m² products (VI-MCAD-03), with **2 × 10 g W-Ni-Cu ballast** on the +Z end frame (decision C-13), trimmed at integration from the measured tensor (VI-MCAD-03). Layout: battery at the −Z end of the stack, RF at +Z, AntS 2 m elements along ±X and 70 cm along ±Y (`hardware/mechanical/MECHANICAL_DESIGN.md` §5; `docs/environment/MASS_BUDGET.md` §6; `docs/environment/STRUCTURAL_ANALYSIS.md`) |
| Mass basis | Bottom-up MEL with the build-versus-buy selections (`docs/hardware/BUILD_VS_BUY.md`); the prior 1.2 kg and 1.69 kg figures are superseded (TB-06) |
| Deployables | VHF antenna, UHF antenna, two solar wings |
| Inhibits | **≥ 3 independent physical RF inhibits** and **≥ 3 independent physical inhibits per deployable** (CDS 2.3.7, 2.3.8). RBF and timers do not count. Implemented with **three deployment switches on three rails**: DS2 → low-side FET in the pack return, DS1 → high-side FET in VBAT+, DS3 → a second high-side FET in VBAT+ that also holds the trackers' enables low; each switch drives a FET gate through a ≥ 10 ms RC filter. The same three give the NRCSD backup its ground-leg inhibit (`docs/hardware/EPS_DESIGN.md` 2.0 §8) |
| Timers | Deployables ≥ 30 min after switch release (CDS 2.4.4); **no RF before 45 min** (CDS 2.4.5); timers reset on switch toggle (CDS 2.3.2.4). **No magnetorquer actuation or deployment before 120 s** after separation (SpaceX UG v11 §3.3.4). Once the RF silence has completed it is latched: a stowed sense reading no longer restarts it (NRCSD IDD §4.1.4(5) limits the reset to the first 30 min) |
| Deployment | Wings first: two burn resistors per wing (redundant), each on its own current-limited driver (0.8 A), burned one at a time for 10 s (decision C-17: release-energy ratio 1.90 at −55 °C; was 8 s) (the second only if the wing has not released), with a 12 s hardware on-time limit per driver; peak pack current 1.0 A. Then the ISIS AntS by its I²C arm/deploy sequence; retries spread into sunlight, and a ground-commanded retry (`docs/hardware/EPS_DESIGN.md` §5.1; `docs/environment/MECHANISMS_ANALYSIS.md`) |
| Integration and storage | Final charge to **60 % ± 5 % SOC** at the integrator, about 47 days before launch; no access or charging afterwards; separation SOC ≥ 40 %; battery-side drain with all inhibits open ≤ 20 µA (SQSO-LVR-001 LVR-071) |
| Launch-vehicle rules | SpaceX UG v11 (August 2026): no attitude manoeuvre or appendage deployment before 120 s after separation (§3.3.4); hard-body radius 0.6 m reported; GNSS-based ephemeris with covariance within 24 h of first contact (§9.5.7.2) |

## 5. Power

| Parameter | Baseline |
|-----------|----------|
| Solar array | 16 triple-junction cells (~30 %): 2 on each of 4 long body faces, plus 2 deployable 1U-face wings with 2 cells per side **[A, layout to be drawn]** |
| Generation, EOL orbit average delivered to the bus | **2.48 W tumbling; 3.92 W sun-pointed**; 1.47 W with the wings stowed (mission simulation at 475 km from 2028-11-01, worst 30 days of the second year, LTAN 22:30 ± 1 h; `docs/environment/POWER_BUDGET_ANALYSIS.md` v2.2 §4). The isotropic-tumble analysis (2.14 W, SOLAR_ARRAY_DESIGN §9.3, cells at 60 °C, no albedo) is kept as a bound |
| Battery | 4S1P Li-ion 18650, ≈ 3.5 Ah, 14.4 V nominal, **≈ 50 Wh** (below the 80 Wh ISS extra-assessment threshold and the 100 Wh FAA note in CDS 2.1.5). Size kept; **charged to 4.10 V/cell (16.4 V), not 4.20 V**, because the pack sits at the end of charge for most of the mission **[D8]** |
| Depth of discharge | ≤ 10 % nominal per eclipse; ≤ 30 % contingency (SYS-112) |
| Mode thresholds (SOC) | ACTIVE allowed at ≥ 55 % (sunlit); ACTIVE → IDLE below 40 %; → SAFE below 25 %; SAFE → EMERGENCY (beacon shed) below 15 %; back to SAFE at 25 %, to IDLE above 40 % (`eps_control.h`, `sm_power_event()`) |
| Charge temperature window | **0 °C to +45 °C** (SYS-602; no charging below 0 °C) |
| Cell over-voltage protection | ≤ 4.25 V/cell |
| End-of-life passivation | Battery charge path permanently disconnected (non-resettable), transmitters disabled, battery left in a permanent discharge state (47 CFR 97.207(g)(1)(iii)) |

**Power closure (SQSO-PBA-001 v2.2 §6):** ACTIVE at the duty limits 2.50 W sun-pointed (**+54 %**), 1.82 W tumbling
(**+34 %**); IDLE 1.18 W; SAFE 0.79 W; EMERGENCY 0.72 W, heater included. Margin ≥ 20 % in every mode at end of life
and at LTAN 22:30 ± 1 h; SYS-100 (2.23 W: the 1.86 W allocation × 1.2, CR-2026-009) met with 11 % to spare. With the RF board's designed transponder draw (1.84 W DC at 0.5 W RF, Version E) the bottom-up ACTIVE loads are 2.60 W sun-pointed and 1.91 W tumbling: margins +48 % and +27 % by the PBA formula (+51 % / +30 % by the BASELINE method; PBA 2.4). As flown by the flight software in the mission
simulation: depth of discharge ≤ 2.3 % (≤ 3.7 % in the full SIL run), deepest in any contingency 2.8 %. Wings stowed: SAFE and IDLE only (ACTIVE
would be −42 %). A Jetson rail stuck on is shed in hardware below 13.6 V (EPS_DESIGN §6); the PA is not shed (CR-2026-016). The allocations behind the
original closure:

| Load | Tumbling | Sun-pointed | Gating |
|------|----------|-------------|--------|
| Housekeeping: OBC, EPS, sensors, magnetorquers (≤ 0.3 W avg), heater | 0.70 W | 0.70 W | — |
| GNSS receiver, duty-cycled (3–6 sessions a day; every orbit during launch and early operations) | 0.05 W | 0.05 W | Jetson and PA off; SOC above contingency floor **[D6]** |
| 435 MHz receiver, always on | 0.15 W | 0.15 W | Required by 97.207(b) |
| Telemetry beacon (≤ 1 W RF, ~2 s per 60 s) | 0.10 W | 0.10 W | — |
| FM transponder (0.5 W RF: 1.84 W DC designed with the synthesizer's own regulator, SQSO-RF-BRD-001 E; allocation 1.85 W DC), duty ≤ 25 % | 0.46 W | 0.46 W | Sunlit, SOC ≥ 55 %, AI/OBC scheduled |
| Jetson (≤ 8 W input), duty ≤ 5 % tumbling / ≤ 10 % sun-pointed | 0.40 W | 0.80 W | Sunlit, SOC ≥ 55 %, temperature limits |
| **Total load** | **1.86 W** | **2.26 W** | |
| Generation (simulated, EOL) | 2.48 W | 3.92 W | |
| **Margin** | **+33 %** | **+73 %** | Requirement ≥ 20 % |

The GNSS line is a flat allocation; the expected steady-state draw is 1–7 mW. Continuous operation (up to 0.32 W) would
break the tumbling-case margin, so the receiver is always duty-cycled.

**Transponder (D4):** the PI decided to keep the duty-gated transponder. If the rebuilt power budget or engineering-model
thermal test fails to close at these duty cycles, the PI revisits D4; the fallback is beacon-only (receiver retained for
telecommand). Any beacon-only variant still requires the 435 MHz
receiver and TX-OFF path.

## 6. Avionics and Payload

| Parameter | Baseline |
|-----------|----------|
| OBC | **STM32L4R5ZI**; flight software in Rust on the **Embassy executor, no RTOS** (one task; `software/flight/docs/RTOS_ARCHITECTURE.md`). Flight-software memory, timing, build and radiation docs already target this part. Apollo4 and STM32L476 references are superseded |
| AI module | NVIDIA **Jetson Orin Nano 8 GB module** (P/N 900-13767-0030-000) on a custom carrier, **7 W power mode**, ≤ 8 W input including carrier; no fan. NVIDIA lists the 7 W mode only for the original flash configuration, not "Super" (TB-13) |
| AI burst | Each Jetson power-on ≤ 10 minutes including boot, producing a batch of ≥ 10 lines, then ≥ 100 minutes off (`sqso_fsw::payload_schedule`, SIL-01; the energy gate of the AI model row governs); sunlit only; SOC ≥ 55 %; Jetson temperature ≥ −15 °C at power-on; OBC-enforced per-rail current (latch-up) and temperature limits independent of the Jetson; hardware load shed below 13.6 V |
| AI model | **Flight baseline:** a calibrated selector on the OBC (gradient-boosted trees or a tiny INT8 MLP, with abstention to fixed rules) that chooses among pre-approved templates and proposes power mode and transponder schedule (the FM repeater has no per-station control, so it sets no QSO priority; `docs/payload/OBC_SELECTOR_DESIGN.md` §3.4); hard limits stay outside the model. Designed, not yet implemented. About 45 µJ to 10 mJ per decision; the Jetson is not woken. **Gated experiment:** SmolLM2-360M-Instruct or Qwen3-0.6B (non-thinking), INT4, Apache 2.0, generating non-numeric lines in batches of ≥ 10. Enabled only when the measured p95 energy per *accepted* beacon at the carrier input, boot to power-off, is ≤ 0.1 Wh (360 J) and the ground filter sweep passes. Jev (TypeSafe AI) is ground-side only, after written terms allow training on its outputs ([selection](payload/AI_MODEL_SELECTION.md)) **[D5]** |
| AI text controls | Deterministic filter on payload and OBC, same rules and shared test vectors: plain characters only, no URLs or commercial words, no profanity, no encoded-looking words, no call sign but the station's own, **no numbers in generated text** (the OBC writes all values from measurements) (97.113(a)(3)–(4); `docs/BEACON_SPEC.md` §6); health-derived fallback text |
| Jetson heat path | SYS-603 Rev G temperature limits (CR-2026-010): spreader ≤ +40 °C in a 600 s batch, junction ≤ 95 °C, spreader ≤ +70 °C if stuck on; design 1.10 W/K spreader to frame, 1.09–1.16 K/W junction to spreader, measured on the EM (VI-01, V-THM-3) |
| GNSS | **Add a GNSS receiver [D6]:** SkyFox piNAV-NG/FM (24 g, 125 mW typical, UART NMEA, LEO cold start 80 s) with a 50 × 50 mm patch (Spacemanic ZENITH V3a); alternate Spacemanic Celeste. Non-critical: OBC load switch with latch-up trip, power-cycled on fault. Drives SpaceX User's Guide v11 §9.5.7.2: forward-predicted GNSS-based ephemeris with covariance within 24 h of first contact, then at least every 24 h until cataloged. Needs a ground orbit-determination and CCSDS OEM upload pipeline. Third-party ephemeris stays as the contingency. Open: patch location on the faces, Czech export licence, L1 noise test with Jetson, PA and magnetorquers |
| Radiation | Design to 10 krad(Si). The 2-year dose behind 2 mm Al is 5.25 krad(Si) at 500 km (5.92 at 525 km; `docs/environment/RADIATION_ANALYSIS.md` C), a margin of 1.90 on the design level, so **sensitive parts need ≥ 2.1 mm Al-equivalent shielding by sector analysis, or a demonstrated ≥ 2× margin on their sector dose** (RID RAD-01). Single-event latch-up: current-limited, tripping supplies on every rail (Jetson, OBC, radio, sensors). Jetson treated as non-critical and expendable |

**Note (2026-10-25).** The OBC row was "FreeRTOS" until Revision C. FreeRTOS was never integrated; the flight software
is Rust (`software/rust/README.md`) on the Embassy executor, and the PI rebaselined the row to match (change request
CR-2026-002, `docs/project/CHANGE_REQUEST_LOG.md`).

## 7. RF and Telecommand

| Parameter | Baseline |
|-----------|----------|
| Downlink | One IARU-coordinated frequency in 145.800–146.000 MHz; **not 145.825 MHz** |
| Uplink | One IARU-coordinated frequency, requested in **435.222–435.778 MHz, or else 436.222–436.500 MHz**: the RF board's receive SAW drift and its clock harmonics at 435.000 MHz limit the band to 435.050–436.500 MHz (SQSO-RF-BRD-001), and the EPS converters' synchronised 1.000 MHz harmonics leave windows clear only at 435.222–435.778, 436.222–436.778 and 437.222–437.300 MHz (SQSO-EPS-BRD-001, SQSO-EMC-001), at least 0.9 MHz below 3 × f_downlink (437.4–438.0 MHz): with the third harmonic at −70 dBc and the LMX2571 exciter, desense is ≤ 0.44 dB even 100 kHz from 3f and ≤ 0.01 dB at 2.2 MHz, inside RF-P009's 0.5 dB (SQSO-RF-BRD-001 §7.5). Receiver: CC1200 in analog-FM receive with a BFP640F LNA, a 7th-order trapped high-pass and a SAW; TNC on its own STM32L431 (a Class C firmware item) |
| Transmitter power | **Maximum 1 W**, nominal 0.5 W (file the hardware maximum) |
| Emissions | 16K0F3E (transponder), 10K4F2D (telemetry and telecommand); confirm by measurement |
| Spurious / harmonics | Meet 47 CFR 97.307(e) (≤ 25 µW and ≥ 40 dB below carrier; at 1 W that means ≥ 46 dBc). **≥ 70 dB 3rd-harmonic suppression** measured at the antenna port with the antennas deployed (≤ 0.5 dB desense of the uplink at 1 W; 60 dB costs 3.2 dB; SQSO-LBA-001 §7.3). Receive filter ≥ 60 dB rejection at 144–146 MHz |
| Antennas | **ISIS AntS 1U–3U** (bought), separate deployable quarter-wave tape monopoles for 2 m (≈ 0.5 m) and 70 cm (≈ 0.17 m), with an RX filter; no shared diplexed element. **2 m elements along ±X**, across the spin axis (keeps +Y the major axis of inertia, §4); the resulting spin fading costs about 4 % of frames (SQSO-LBA-001 §5), partly recovered by beacon-interval jitter. Antennas deploy after the wings (the stowed tapes pass 4–6 mm under the wings) |
| Telecommand authentication | HMAC-SHA-256 (truncated tag) with a persisted monotonic counter per key; required for every command except TX-OFF when no key is usable (97.211(b)). Key table: 4 operational keys (1 active, 3 spares) and an offline recovery key that can rotate to a spare; keys loaded only through the umbilical and never in the repository (SQSO-ICD-013) |
| Cessation of emissions | TX-OFF removes PA power through a hardware switch; latched in NVM; overrides SAFE-mode beacon; independent of the AI payload (97.207(b), RR 22.1) |
| Loss of command contact | Transmit inhibit after N days without an authenticated command, receiver on (SQSO-ICD-013 §6.2). **N = 30 days at launch**, set to **14 days** by `SET_CONTACT_TIMEOUT` once commissioning is complete (range 1–90). Proposed, not implemented: autonomous passivation **M = 180 days** after the inhibit, never in the first year. No FCC or IARU rule sets either value; both go to the FCC consultation **[D3]** |
| Link closure | 1200 bps beacon at 0.5 W to a 12 dBic / 500 K station: **+7.9 dB at 5°** (median spacecraft gain), +0.9 dB (10th percentile). Telecommand at **≥ 64 dBm EIRP** above 10° (≥ 100 W PEP into ≥ 15.5 dBic, or ≤ 50 W PEP at the transmitter output, ≤ 0.5 dB feed, into a ≥ 17.8 dBic stacked pair in the 47 CFR 2.106 US270 areas; decision C-14): ≥ +8.0 dB at 10° and ≥ +6.2 dB at 5° with the receiver as designed, the 10th-percentile gain and a 6 dB interference allowance (variant A with its amplifier out, 50 W into 15.5 dBic, meets 6 dB from 12°: degraded mode, commands above 12°). Transponder downlink to a handheld Yagi: 20 / 28 / 32 dB SINAD at 5° / 20° / zenith. Ground stations pre-correct uplink Doppler (±10.4 kHz at 435 MHz) to ±1 kHz (`docs/environment/LINK_BUDGET_ANALYSIS.md` v3.5, analysis) |

## 8. ADCS, Thermal, Data, Environments

| Parameter | Baseline |
|-----------|----------|
| ADCS | Three-axis magnetorquers: **CubeSpace CR0002 rods** (> 0.2 A·m², selected; the 250-turn air-core coil delivering ≥ 0.15 A·m² is the alternate), H-bridge drive with a hardware off-timer; magnetometer, coarse sun sensors (the panel photodiodes), MEMS gyro (**Epson M-G370PDT0**, Bosch BMI088 backup; bias ≤ 0.1°/s 1σ in use) **[D9]**. B-dot detumble from 10°/s within 3 orbits (gain 10⁴ A·m²·s/T; the magnetometer is sampled 50 ms after a 900 ms coil burst). Spin-stabilized coarse sun-pointing of +Y at **8°/s**, nutation damping 3·10⁻⁴ N·m·s (1·10⁻⁴ without the gyro): Monte Carlo of the flight code with the layout-B inertia tensor, mean 4.0°, 99.6 % of sunlit time within 15°, slowest capture 1.2 h (2.1 h from a 50°/s tip-off); Iyy/Izz 1.00–1.05 gives 3.8–6.9° and ≥ 93 % (`docs/hardware/ADCS_DESIGN.md` 1.5 §11.4) **[D9]**. Spin axis and inertia selectable by telecommand for a one-wing deployment. Residual dipole ≤ 3 mA·m² design, 10 mA·m² limit (Helmholtz-cage measurement of the flight unit, wings deployed and illuminated) |
| Thermal cases | Hot = β at maximum + maximum power (the orbit reaches β ≤ 41°, so the no-eclipse hot case is kept as a bound only); Cold = β = 0 (≈ 36 min eclipse) + safe mode; dispenser soaks hot and cold; LEOP from the cold soak with separation at eclipse entry and the wings stowed until the burn; ≥ 10 °C model-uncertainty margin before EM correlation. Allowable flight temperatures per unit (`hardware/structure/THERMAL_DESIGN.md` 1.5 §1.1): boards −30 °C cold operating and non-operating, −40 °C for the first orbit after separation (tested at its AFT without the 10 °C margin, ETP §9 deviation 8; CR-2026-017). Battery heater ≈ 207 Ω (0.70–1.30 W across the 12.0–16.4 V bus), on below +5 °C, off above +9 °C (`docs/environment/THERMAL_ANALYSIS.md` v2.2) |
| Data | Beacon every 60 s in ACTIVE, 120 s in IDLE, 180 s in SAFE (~200 B, broadcast; MRD-020). Housekeeping **summarized every 5 minutes, ≥ 7 days onboard (≈ 88 kB)**, with 60 s records for the 6 h around each fault and mode change, downlinked by telecommand (`DUMP_TLM`); 7 days of 60 s records (444 kB) would not fit the 192 KB FRAM nor be dumped by one station (SQSO-DB-001 v2.0) |
| Launch environments | SpaceX Rideshare Payload User's Guide v11 (August 2026, current) and the EXOpod Nova ICD. **Protoflight random vibration** at the envelope of the provider's MPE + 3 dB and the GSFC-STD-7000B workmanship spectrum (**8.18 Grms** on the interim levels), 1 minute per axis, power-inhibit verification during the test; first mode > 40 Hz (computed ≈ 140 Hz stowed). **Thermal vacuum:** allowable flight temperatures ±10 °C, hot plateaus capped at +60 °C by the battery, 8 cycles; **bake-out** +60 °C, 6 h, ≤ 1·10⁻⁴ Torr. Structural factors of safety NASA-STD-5001B protoflight (1.25 yield / 1.40 ultimate) at MPE + 3 dB. Levels become final with the integrator's ICD (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C) |
| Materials | TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS 2.1.7); thermal-vacuum bakeout per provider |

## 9. Decisions

Taken by the PI on 2026-10-15 from the research in `funding_audit/research/`. Values that the FCC consultation (D3) may
change are marked there.

| ID | Decision | Research | Remaining actions |
|----|----------|----------|-------------------|
| D1 | Licensee **KK6GDB** (individual); station KK6GDB-11. Telecommand stations TBD; NASA Ames and Santa Clara University are prospective partners only | — | Name ≥ 2 telecommand stations before the IARU request; licensee name and address in the filings |
| D2 | **Paid contracts allowed**, on these terms. The amateur station (transponder, beacon, telecommand) stays Part 97 with no compensation and no paid content. A paid deliverable travels only on a separate Part 5 radio, held by a separate licensee, on non-amateur frequencies (HuskySat-1 pattern). Part 5 is non-commercial too: the payer must be the U.S. Government (47 CFR 5.3(c)) or the research the licensee's own; selling data needs Part 25. Until a contract is signed, funding is grants and donations with nothing required in return | [D2_LICENSING.md](funding_audit/research/D2_LICENSING.md) | A signed contract triggers a second radio (UHF ≈ $6k; 400.15–401 MHz down, 449.75–450.25 MHz up), a power and volume trade against D4, a Part 5 filing and IARU combined-mission coordination. Questions for the FCC consultation in the research §5 R6 |
| D3 | N = 30 days at launch, 14 days after commissioning; M = 180 days proposed with a first-year lockout, not implemented | [D3_LOSS_OF_CONTACT.md](funding_audit/research/D3_LOSS_OF_CONTACT.md) | FCC Space Bureau consultation before the IARU request: N in the pre-space notification; whether M is wanted; whether a reversible inhibit counts as termination for 97.207(g)(3) |
| D4 | **Keep the duty-gated transponder** | — | Revisit only if the power budget or EM thermal test fails (§5) |
| D5 | OBC-resident selector as flight baseline; Jetson generative text as a gated experiment; Jev ground-side only | [D5_AI_MODEL_SELECTION.md](funding_audit/research/D5_AI_MODEL_SELECTION.md), [selection](payload/AI_MODEL_SELECTION.md) | Measure energy per accepted beacon on the EM carrier (selection §5); resolve the 7 W mode (TB-13) |
| D6 | **Add a GNSS receiver** (piNAV-NG/FM, Celeste alternate); third-party ephemeris as contingency | [D6_GNSS_RECEIVER.md](funding_audit/research/D6_GNSS_RECEIVER.md) | Quote both EM units; confirm with SpaceX and the integrator that a duty-cycled receiver meets §9.5.7.2; patch location; export licence; ground orbit-determination pipeline |
| D7 | Transporter-class SSO via a 1U integrator, contract by 2027-06-30, altitude clause; NRCSD backup | [D7_LAUNCH_INTEGRATORS.md](funding_audit/research/D7_LAUNCH_INTEGRATORS.md) | Requests for quotes by 2026-12-31 (§3) |
| D8 | **Keep the 4S1P ≈ 50 Wh pack; charge to 4.10 V/cell.** It is 2–3 times larger than the energy needs (the worst case simulated, wings and ADCS both failed, uses 3.2 % per eclipse), but the only lighter packs are 2S and change the 14.4 V bus for ≈ 125 g. Revisit with 2S1P if the MEL re-run after the COTS buys still has < 10 % mass margin. Taken 2026-10-26 under the PI's delegation | [BUILD_VS_BUY.md §7](hardware/BUILD_VS_BUY.md) | Charger set point in the EPS design; LEO-profile cycling of flight-lot cells at 4.10 V (BATTERY_DESIGN §9) |
| D9 | **Gyro: Epson M-G370PDT0, BMI088 backup. ADCS: B-dot gain 10⁴, spin 8°/s (6°/s until the design review re-tuned it for the computed inertia tensor) with a spin-hold term.** Taken 2026-10-26 under the PI's delegation; spin rate changed by the 2026-10-27 design review | [BUILD_VS_BUY.md §5–§6](hardware/BUILD_VS_BUY.md) | Quote the M-G370PDT0; BMI088 thermal calibration rig; magnetic cleanliness test of the residual dipole; re-run the Monte Carlo with the measured inertia and, if rods are bought, their remanence |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial baseline proposed by the funding audit |
| B | 2026-10-15 | Decisions D1–D7 recorded (§9): licensee, paid-contract structure, contact timeout, transponder kept, AI model selection, GNSS receiver (mass and power lines), integrator plan and cost |
| B, note | 2026-10-16 | Notes on mass (§4) and the tumbling margin (§5) from the re-issued mass budget and solar-array analysis; values unchanged pending the PI |
| B, note | 2026-10-24 | Note beneath §6: the flight software is Rust on the Embassy executor, not FreeRTOS; the OBC row is unchanged pending the PI |
| C | 2026-10-25 | §6 OBC: the flight software runs on the Embassy executor with no RTOS (was FreeRTOS), by PI decision (CR-2026-002). §1 software classification: Class C (NPR 7150.2D Appendix D) |
| D | 2026-10-26 | Decisions D8 (battery size kept, charge voltage 4.10 V/cell) and D9 (gyro; B-dot gain; 6°/s spin), delegated by the PI, from the build-versus-buy study and the ADCS Monte Carlo (`docs/hardware/BUILD_VS_BUY.md`) |
| E | 2026-10-27 | CDR 2026 chair dispositions (CR-2026-007; `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md` §4): §1 stations and contracting party; §3 launch path (Falcon 9 / Exolaunch), planning date 2028-11, 475 km target and 485 km cap, lifetime per ODAR Draft B; §4 mass and mass properties, inhibits DS1–DS3, timers, deployment; §5 generation and allocation; §6 radiation and AI rows; §7 uplink windows, spurious, antennas, link; §8 data, environments, thermal, ADCS 8 °/s; §9 D9 note |
| E, amended | 2026-10-27 | CR-2026-008: §4 inertia ballast (C-13), mass 1.51 / 1.75 kg, +14.4 %, inertia ratio 1.055 / 1.046 with four AntS elements; §7 telecommand ≥ 64 dBm EIRP in two station variants (C-14), uplink ≥ 0.9 MHz below 3 × f_down; §1, §3, §5, §7 values from the Rev E re-runs (ODAR Draft D, PBA 2.2, LBA 3.4) |
