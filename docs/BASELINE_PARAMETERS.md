# SMART-QSO Baseline Parameters (Single Source of Truth)

**Document Number:** SQSO-BL-001
**Revision:** B — *decisions D1–D7 taken by the PI (§9); the D2 structure and D3 values still go to the FCC consultation*
**Date:** 2026-10-15

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
| Telecommand stations | ≥ 2 designated amateur stations, geographically separated: **TBD [D1]**. NASA Ames Research Center and Santa Clara University are prospective partners; nothing is agreed |
| Host organization | None yet. Needed only for routes that require an institution (for example NASA CSLI eligibility, §3) **[D1]** |
| Operator compensation | None for operations (Part 97 condition C2) |
| Project phase | **Phase B (pre-PDR).** No independent PDR or CDR has been held; earlier review records are self-assessments (FINDINGS_REGISTER PI-03, PI-04) |
| Payload risk classification | Class D (NPR 8705.4B), proposed |
| Software classification | Formal classification per NPR 7150.2D Appendix D **pending** (not yet determined) |

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
| Launch path | Transporter-class SSO rideshare through a 1U-capable integrator, 2028; contract signed by **2027-06-30** ([trade](funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md)) **[D7]**. Requests for quotes: Exolaunch (lead), ISISPACE (price check), SEOPS (US-owned; 1U support to confirm), D-Orbit and Maverick (one question each) |
| Launch cost (ROM) | **$115k–250k all-in**, central case about $150–170k: integrator 1U slot $85k–150k (no provider publishes a 1U price) plus $30k–100k program-side (tests, export counsel, logistics, ephemeris service). Add 15 % contingency on the slot for rebooking **[D7, U]** |
| Altitude protection | SpaceX offers drop-off altitudes from a menu (500–600 km on the open 2027 flights), so the contract must state a 500 km target and **525 km hard cap**, with no-penalty rebooking if the assigned flight offers nothing at or below the cap **[D7]** |
| Orbit | Circular SSO, inclination ≈ 97.4° |
| Deployment altitude | **Target ≤ 500 km.** 500–525 km acceptable only if NASA DAS shows FCC compliance in the wings-failed case. **Not above 525 km** |
| Backup | ISS deployment (Voyager NRCSD), 400–420 km, 51.6°, ROM $120k–250k all-in; JSC 20793 battery program is the cost driver. Life 0.7–1.9 yr nominal; mission success Level 3 not achievable. Triggered if no integrator commits in writing to ≤ 525 km by 2027-06-30 or the primary exceeds $250k; decide by 2027-09 **[D7]** |
| Contingency | Electron rideshare from New Zealand with a third-party 1U dispenser (altitude is a design point; price unknown; export licence). NASA CSLI only as a no-cost long shot: next solicitation 2028, launch about 2030 or later, eligibility needs an institution or nonprofit (§1) **[D7]** |
| Propulsion / station-keeping | None |
| Disposal | Passive atmospheric drag; solar wings stay deployed (they are the drag augmentation). **No separate drag device** (removed from baseline; it was never in mass, structure, software or test) |

Orbital lifetime from `software/simulation/orbit_lifetime/orbit_lifetime.py` (NRLMSIS 2.1; 1.6 kg; Cd 2.2; deployment
2028-06-01; nominal = approximate solar-cycle profile; bound = F10.7 held at 70). Must be reproduced in NASA DAS before
filing:

| Deployment | Wings deployed (A ≈ 0.028 m²) nominal / bound | Wings failed (A ≈ 0.0164 m²) nominal / bound | Post-mission (2 yr) nominal, wings deployed | FCC ≤ 5 yr post-mission? |
|------------|---------------------------|--------------------------|------------------------|-----------------|
| 450 km | 2.3 / 3.2 yr | 4.0 / 5.5 yr | 0.3 yr | Yes |
| 475 km | 4.0 / 5.5 yr | 5.2 / 9.4 yr | 2.0 yr | Yes |
| 500 km | 5.1 / 9.3 yr | 6.2 / 15.8 yr | 3.1 yr | Yes (nominal, both cases) |
| 525 km | 6.0 / 15.2 yr | 7.4 / 25.9 yr | 4.0 yr | Wings deployed only |

## 4. Spacecraft Physical

| Parameter | Baseline |
|-----------|----------|
| Form factor | 1U, 100 × 100 × 113.5 mm (CDS Rev 14.1) |
| Mass | CBE ≈ 1.51 kg **[A]**: 1.44 kg provisional MEL ([ODAR §2](regulatory/filings/ODAR_SMART-QSO.md)) plus ≈ 0.07 kg GNSS receiver, patch antenna and cable **[D6]**; MEV ≤ 1.75 kg (15 % growth); limit **2.00 kg** (CDS Rev 14.1 Table 1) or the integrator's dispenser limit, whichever is lower. The NRCSD backup limit is 2.40 kg. Lifetime analyses use 1.6 kg, with 1.8 kg as the bound |
| Mass basis | The prior 1.2 kg CBE omitted magnetorquers, wings/hinges and the heat spreader (TB-06). A controlled bottom-up MEL is required |
| Deployables | VHF antenna, UHF antenna, two solar wings |
| Inhibits | **≥ 3 independent physical RF inhibits** and **≥ 3 independent physical inhibits per deployable** (CDS 2.3.7, 2.3.8). RBF and timers do not count |
| Timers | Deployables ≥ 30 min after switch release (CDS 2.4.4); **no RF before 45 min** (CDS 2.4.5); timers reset on switch toggle (CDS 2.3.2.4) |

## 5. Power

| Parameter | Baseline |
|-----------|----------|
| Solar array | 16 triple-junction cells (~30 %): 2 on each of 4 long body faces, plus 2 deployable 1U-face wings with 2 cells per side **[A, layout to be drawn]** |
| Generation, EOL orbit average | **2.39 W tumbling; 3.58 W sun-pointed** [A: audit calculation, TB-02] |
| Battery | 4S1P Li-ion 18650, ≈ 3.5 Ah, 14.4 V nominal, **≈ 50 Wh** (below the 80 Wh ISS extra-assessment threshold and the 100 Wh FAA note in CDS 2.1.5) |
| Depth of discharge | ≤ 10 % nominal per eclipse; ≤ 30 % contingency (SYS-112) |
| Mode thresholds (SOC) | ACTIVE allowed at ≥ 55 % (sunlit); ACTIVE → IDLE below 40 %; → SAFE below 25 %; SAFE → EMERGENCY (beacon shed) below 15 %; back to SAFE at 25 %, to IDLE above 40 % (`eps_control.h`, `sm_power_event()`) |
| Charge temperature window | **0 °C to +45 °C** (SYS-602; no charging below 0 °C) |
| Cell over-voltage protection | ≤ 4.25 V/cell |
| End-of-life passivation | Battery charge path permanently disconnected (non-resettable), transmitters disabled, battery left in a permanent discharge state (47 CFR 97.207(g)(1)(iii)) |

**Power closure (orbit-average allocations; to be replaced by a multi-orbit SOC simulation):**

| Load | Tumbling | Sun-pointed | Gating |
|------|----------|-------------|--------|
| Housekeeping: OBC, EPS, sensors, magnetorquers (≤ 0.3 W avg), heater | 0.70 W | 0.70 W | — |
| GNSS receiver, duty-cycled (3–6 sessions a day; every orbit during launch and early operations) | 0.05 W | 0.05 W | Jetson and PA off; SOC above contingency floor **[D6]** |
| 435 MHz receiver, always on | 0.15 W | 0.15 W | Required by 97.207(b) |
| Telemetry beacon (≤ 1 W RF, ~2 s per 60 s) | 0.10 W | 0.10 W | — |
| FM transponder (0.5 W RF ≈ 1.5 W DC), duty ≤ 25 % | 0.40 W | 0.40 W | Sunlit, SOC ≥ 55 %, AI/OBC scheduled |
| Jetson (≤ 8 W input), duty ≤ 5 % tumbling / ≤ 10 % sun-pointed | 0.40 W | 0.80 W | Sunlit, SOC ≥ 55 %, temperature limits |
| **Total load** | **1.80 W** | **2.20 W** | |
| Generation | 2.39 W | 3.58 W | |
| **Margin** | **+33 %** | **+63 %** | Requirement ≥ 20 % |

The GNSS line is a flat allocation; the expected steady-state draw is 1–7 mW. Continuous operation (up to 0.32 W) would
break the tumbling-case margin, so the receiver is always duty-cycled.

**Transponder (D4):** the PI decided to keep the duty-gated transponder. If the rebuilt power budget or engineering-model
thermal test fails to close at these duty cycles, the PI revisits D4; the fallback is beacon-only (receiver retained for
telecommand). Any beacon-only variant still requires the 435 MHz
receiver and TX-OFF path.

## 6. Avionics and Payload

| Parameter | Baseline |
|-----------|----------|
| OBC | **STM32L4R5ZI**, FreeRTOS. Flight-software memory, timing, build and radiation docs already target this part. Apollo4 and STM32L476 references are superseded |
| AI module | NVIDIA **Jetson Orin Nano 8 GB module** (P/N 900-13767-0030-000) on a custom carrier, **7 W power mode**, ≤ 8 W input including carrier; no fan. NVIDIA lists the 7 W mode only for the original flash configuration, not "Super" (TB-13) |
| AI burst | ≤ 120 s including boot; sunlit only; SOC ≥ 55 %; OBC-enforced per-rail current (latch-up) and temperature limits independent of the Jetson |
| AI model | **Flight baseline:** a calibrated selector on the OBC (gradient-boosted trees or a tiny INT8 MLP, with abstention to fixed rules) that chooses among pre-approved templates and proposes power mode, transponder schedule and QSO priority; hard limits stay outside the model. About 45 µJ to 10 mJ per decision; the Jetson is not woken. **Gated experiment:** SmolLM2-360M-Instruct or Qwen3-0.6B (non-thinking), INT4, Apache 2.0, generating non-numeric lines in batches of ≥ 10. Enabled only when the measured p95 energy per *accepted* beacon at the carrier input, boot to power-off, is ≤ 0.1 Wh (360 J) and the ground filter sweep passes. Jev (TypeSafe AI) is ground-side only, after written terms allow training on its outputs ([selection](payload/AI_MODEL_SELECTION.md)) **[D5]** |
| AI text controls | Deterministic filter on payload and OBC, same rules and shared test vectors: plain characters only, no URLs or commercial words, no profanity, no encoded-looking words, no call sign but the station's own, **no numbers in generated text** (the OBC writes all values from measurements) (97.113(a)(3)–(4); `docs/BEACON_SPEC.md` §6); health-derived fallback text |
| Jetson heat path | ≥ 1 W/K to structure, demonstrated on the EM (audit estimate of the documented stack: ≈ 0.44 W/K, TB-07) |
| GNSS | **Add a GNSS receiver [D6]:** SkyFox piNAV-NG/FM (24 g, 125 mW typical, UART NMEA, LEO cold start 80 s) with a 50 × 50 mm patch (Spacemanic ZENITH V3a); alternate Spacemanic Celeste. Non-critical: OBC load switch with latch-up trip, power-cycled on fault. Drives SpaceX User's Guide v11 §9.5.7.2: forward-predicted GNSS-based ephemeris with covariance within 24 h of first contact, then at least every 24 h until cataloged. Needs a ground orbit-determination and CCSDS OEM upload pipeline. Third-party ephemeris stays as the contingency. Open: patch location on the faces, Czech export licence, L1 noise test with Jetson, PA and magnetorquers |
| Radiation | Design to 10 krad (≥ 2× the 2-year dose behind 2 mm Al); Jetson treated as non-critical and expendable |

## 7. RF and Telecommand

| Parameter | Baseline |
|-----------|----------|
| Downlink | One IARU-coordinated frequency in 145.800–146.000 MHz; **not 145.825 MHz** |
| Uplink | One IARU-coordinated frequency in 435–438 MHz, chosen **away from 3 × f_downlink** |
| Transmitter power | **Maximum 1 W**, nominal 0.5 W (file the hardware maximum) |
| Emissions | 16K0F3E (transponder), 10K4F2D (telemetry and telecommand); confirm by measurement |
| Spurious / harmonics | Meet 47 CFR 97.307(e) (≤ 25 µW and ≥ 40 dB below carrier; at 1 W that means ≥ 46 dBc). **Target ≥ 60 dB 3rd-harmonic suppression** to avoid desensing the uplink receiver |
| Antennas | Separate deployable quarter-wave monopoles for 2 m (≈ 0.5 m) and 70 cm (≈ 0.17 m), with an RX filter; no shared diplexed element |
| Telecommand authentication | HMAC-SHA-256 (truncated tag) with a persisted monotonic counter per key; required for every command except TX-OFF when no key is usable (97.211(b)). Key table: 4 operational keys (1 active, 3 spares) and an offline recovery key that can rotate to a spare; keys loaded only through the umbilical and never in the repository (SQSO-ICD-013) |
| Cessation of emissions | TX-OFF removes PA power through a hardware switch; latched in NVM; overrides SAFE-mode beacon; independent of the AI payload (97.207(b), RR 22.1) |
| Loss of command contact | Transmit inhibit after N days without an authenticated command, receiver on (SQSO-ICD-013 §6.2). **N = 30 days at launch**, set to **14 days** by `SET_CONTACT_TIMEOUT` once commissioning is complete (range 1–90). Proposed, not implemented: autonomous passivation **M = 180 days** after the inhibit, never in the first year. No FCC or IARU rule sets either value; both go to the FCC consultation **[D3]** |
| Link closure | 1200 bps beacon closes at 0.5 W with ≈ +6 dB at 5° elevation [A, TB-05]; to be re-issued in `docs/environment/LINK_BUDGET_ANALYSIS.md` with the corrected noise floor |

## 8. ADCS, Thermal, Data, Environments

| Parameter | Baseline |
|-----------|----------|
| ADCS | 3-axis air-core magnetorquers, one coil design delivering ≥ 0.15 A·m² per axis (ACTUATOR_DESIGN 250-turn coil); magnetometer, coarse sun sensors, MEMS gyro; B-dot detumble from 10°/s within 3 orbits; spin-stabilized coarse sun-pointing (statistics by simulation) |
| Thermal cases | Hot = β at maximum (no eclipse) + maximum power; Cold = β = 0 (≈ 36 min eclipse) + safe mode; ≥ 10 °C model-uncertainty margin before EM correlation |
| Data | Beacon every 60 s in ACTIVE, 120 s in IDLE, 180 s in SAFE (~200 B, broadcast; MRD-020); housekeeping stored every 60 s; ≥ 7 days onboard storage |
| Launch environments | Integrator/SpaceX levels. SpaceX v11: random vibe ≥ 5.57 Grms MPE with power-inhibit verification during vibe; first mode > 40 Hz (SpaceX) / per integrator. GSFC-STD-7000B as fallback |
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

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial baseline proposed by the funding audit |
| B | 2026-10-15 | Decisions D1–D7 recorded (§9): licensee, paid-contract structure, contact timeout, transponder kept, AI model selection, GNSS receiver (mass and power lines), integrator plan and cost |
