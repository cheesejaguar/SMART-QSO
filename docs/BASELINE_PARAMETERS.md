# SMART-QSO Baseline Parameters (Single Source of Truth)

**Document Number:** SQSO-BL-001
**Revision:** A — *proposed by the funding audit; pending PI approval of decisions D1–D7*
**Date:** 2026-10-14

---

## 0. Purpose and Rules

Before this document existed, the repository described several incompatible spacecraft: battery capacities from 7 to 50 Wh,
three flight computers, a beacon-only mission and a transponder mission, and ISS versus sun-synchronous (SSO) orbits (see
findings PI-13, PI-14 and TB-01 in [FINDINGS_REGISTER.md](funding_audit/FINDINGS_REGISTER.md)). This document fixes one
self-consistent baseline.

1. Where another document disagrees with this one, **this document governs**. The other document is wrong until it is
   updated through configuration control.
2. Values marked **[A]** are audit estimates to be replaced by analysis or measurement. Values marked **[D#]** depend on
   an open decision (§9).
3. Changes require a configuration-control record (`docs/project/CONFIGURATION_MANAGEMENT_PLAN.md`).

## 1. Identity and Organization

| Parameter | Baseline |
|-----------|----------|
| Name | SMART-QSO (Self-Managing Amateur Radio Transponder — QSO) |
| Licensing regime | FCC Part 97 amateur space station ([pathway determination](regulatory/LICENSING_PATHWAY_DETERMINATION.md)) |
| Space station licensee / call sign | **TBD [D1]** |
| Telecommand stations | ≥ 2 designated amateur stations, geographically separated **[D1]** |
| Operator compensation | None for operations (Part 97 condition C2) |
| Payload risk classification | Class D (NPR 8705.4B), proposed |
| Software classification | Formal classification per NPR 7150.2D Appendix D **pending** (not yet determined) |

## 2. Mission

| Parameter | Baseline |
|-----------|----------|
| Primary mission | Single-channel **U/V FM transponder** for worldwide amateur contacts, **duty-gated** by available power |
| Secondary | AX.25 telemetry beacon, 1200 bps AFSK, plain text and published formats |
| Experiment | Onboard AI (power-gated Jetson) that schedules the transponder within the power budget and composes plain-language beacon text; falls back to a conventional transponder and template beacon |
| Community model updates ("federated learning") | **Not in the flight baseline.** Any future experiment treats public uploads as data only; adoption requires an authenticated telecommand (pathway C5). No onboard implementation exists (SW-11) |
| Design life | **2 years**; minimum mission **12 months** (MRD-001) |
| Success criteria | Unchanged (`docs/mission/MISSION_SUCCESS_CRITERIA.md`): L1 30 days, L2 1 year, L3 2+ years |

## 3. Orbit and Launch

| Parameter | Baseline |
|-----------|----------|
| Launch path | SpaceX Transporter SSO rideshare through a 1U-capable integrator, 2028 ([trade](funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md)) **[D7]** |
| Orbit | Circular SSO, inclination ≈ 97.4° |
| Deployment altitude | **Target ≤ 500 km.** 500–525 km acceptable only if NASA DAS shows FCC compliance in the wings-failed case. **Not above 525 km** |
| Backup | ISS deployment (NRCSD), 400–420 km, 51.6°. Life 0.7–1.9 yr nominal; mission success Level 3 not achievable |
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
| Mass | CBE ≈ 1.44 kg **[A]** (provisional MEL in the [ODAR §2](regulatory/filings/ODAR_SMART-QSO.md)); MEV ≤ 1.7 kg (15 % growth); limit **2.00 kg** (CDS Rev 14.1 Table 1) or the integrator's dispenser limit, whichever is lower. The NRCSD backup limit is 2.40 kg. Lifetime analyses use 1.6 kg, with 1.8 kg as the bound |
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
| Charge temperature window | **0 °C to +45 °C** (SYS-602; no charging below 0 °C) |
| Cell over-voltage protection | ≤ 4.25 V/cell |
| End-of-life passivation | Battery charge path permanently disconnected (non-resettable), transmitters disabled, battery left in a permanent discharge state (47 CFR 97.207(g)(1)(iii)) |

**Power closure (orbit-average allocations; to be replaced by a multi-orbit SOC simulation):**

| Load | Tumbling | Sun-pointed | Gating |
|------|----------|-------------|--------|
| Housekeeping: OBC, EPS, sensors, magnetorquers (≤ 0.3 W avg), heater | 0.70 W | 0.70 W | — |
| 435 MHz receiver, always on | 0.15 W | 0.15 W | Required by 97.207(b) |
| Telemetry beacon (≤ 1 W RF, ~2 s per 60 s) | 0.10 W | 0.10 W | — |
| FM transponder (0.5 W RF ≈ 1.5 W DC), duty ≤ 25 % | 0.40 W | 0.40 W | Sunlit, SOC ≥ 55 %, AI/OBC scheduled |
| Jetson (≤ 8 W input), duty ≤ 5 % tumbling / ≤ 10 % sun-pointed | 0.40 W | 0.80 W | Sunlit, SOC ≥ 55 %, temperature limits |
| **Total load** | **1.75 W** | **2.15 W** | |
| Generation | 2.39 W | 3.58 W | |
| **Margin** | **+37 %** | **+67 %** | Requirement ≥ 20 % |

**Descope trigger (D4):** if the rebuilt power budget or engineering-model thermal test fails to close at these duty
cycles, descope to beacon-only (receiver retained for telecommand). Any beacon-only variant still requires the 435 MHz
receiver and TX-OFF path.

## 6. Avionics and Payload

| Parameter | Baseline |
|-----------|----------|
| OBC | **STM32L4R5ZI**, FreeRTOS. Flight-software memory, timing, build and radiation docs already target this part. Apollo4 and STM32L476 references are superseded |
| AI module | NVIDIA **Jetson Orin Nano 8 GB module** (P/N 900-13767-0030-000) on a custom carrier, **7 W power mode**, ≤ 8 W input including carrier; no fan |
| AI burst | ≤ 120 s including boot; sunlit only; SOC ≥ 55 %; OBC-enforced per-rail current (latch-up) and temperature limits independent of the Jetson |
| AI model | Selected by measured energy ≤ 0.1 Wh per generated beacon **[D5]**. The 3B-parameter model in `software/payload/agent.py` (~2 GB) and the "~1 MB TinyLM" in older docs are both unconfirmed |
| AI text controls | Deterministic post-filter on payload and OBC: allowlisted characters, no encoded/obscured strings, no profanity or commercial content, numeric fields cross-checked against telemetry (97.113(a)(4)); health-derived fallback text |
| Jetson heat path | ≥ 1 W/K to structure, demonstrated on the EM (audit estimate of the documented stack: ≈ 0.44 W/K, TB-07) |
| GNSS | **[D6]** Add a GNSS receiver, or contract third-party ephemeris to meet SpaceX User's Guide v11 §9.5.7 |
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
| Loss of command contact | Transmit inhibit after N days without an authenticated command (implemented in flight software, SQSO-ICD-013 §6.2; N set by telecommand, 1–90 days, placeholder default 14); end-of-mission passivation after M further days (not implemented) **[D3, values with FCC/IARU]** |
| Link closure | 1200 bps beacon closes at 0.5 W with ≈ +6 dB at 5° elevation [A, TB-05]; to be re-issued in `docs/environment/LINK_BUDGET_ANALYSIS.md` with the corrected noise floor |

## 8. ADCS, Thermal, Data, Environments

| Parameter | Baseline |
|-----------|----------|
| ADCS | 3-axis air-core magnetorquers, one coil design delivering ≥ 0.15 A·m² per axis (ACTUATOR_DESIGN 250-turn coil); magnetometer, coarse sun sensors, MEMS gyro; B-dot detumble from 10°/s within 3 orbits; spin-stabilized coarse sun-pointing (statistics by simulation) |
| Thermal cases | Hot = β at maximum (no eclipse) + maximum power; Cold = β = 0 (≈ 36 min eclipse) + safe mode; ≥ 10 °C model-uncertainty margin before EM correlation |
| Data | Beacon every 60 s (~200 B, broadcast); housekeeping stored every 60 s; ≥ 7 days onboard storage |
| Launch environments | Integrator/SpaceX levels. SpaceX v11: random vibe ≥ 5.57 Grms MPE with power-inhibit verification during vibe; first mode > 40 Hz (SpaceX) / per integrator. GSFC-STD-7000B as fallback |
| Materials | TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS 2.1.7); thermal-vacuum bakeout per provider |

## 9. Open Decisions

| ID | Decision | Owner | Needed by |
|----|----------|-------|-----------|
| D1 | Licensee, call sign, control operators, ≥ 2 telecommand stations, legal entity/host organization | PI | Before IARU request |
| D2 | Funding instruments compatible with Part 97 (pathway §7) | PI + funders | Before accepting funds |
| D3 | FCC Space Bureau pre-filing consultation (AI experiment, loss-of-contact timers) | Licensee | Before IARU request |
| D4 | Keep the duty-gated transponder (baseline) or descope to beacon-only (trigger in §5) | PI | After the rebuilt power budget and EM thermal test |
| D5 | AI model selection by measured energy per beacon | Payload lead | Before EM thermal test |
| D6 | GNSS receiver vs. third-party ephemeris | Systems | Before integrator contract |
| D7 | Integrator and flight (≤ 500 km target) | PI | ≤ 2027-06-30 (launch-vehicle determination starts the 30-day FCC clock) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial baseline proposed by the funding audit |
