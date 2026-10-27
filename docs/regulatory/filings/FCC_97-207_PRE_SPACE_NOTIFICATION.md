# FCC Part 97 Pre-Space Notification (Draft)
## Amateur Space Station SMART-QSO

**Document Number:** SQSO-REG-013
**Revision:** A.1 (draft, not filed)
**Date:** 2026-10-27
**Rule:** 47 CFR 97.207(g)(1) (eCFR version of 2026-10-01; no amendment since 89 FR 65217, effective 2024-09-09)
**Due:** within 30 days after launch-vehicle determination, and no later than 90 days before integration into the launch vehicle. A replacement is required if any material item changes ((g)(1)(viii)). **Plan** (REGULATORY_ROADMAP Rev B §3): file early on 2027-01-19 with the API data, replace within 30 days of the launch contract (by 2027-06-30), last replacement by 2028-01-18 (integration about 2028-04-17).
**Filing channel:** written notification to the Space Bureau, FCC, Washington, DC 20554. Confirm with Space Bureau staff whether to file via ICFS or by email (DA 13-445 listed amsat.spacecap@fcc.gov in 2013).
**Fill-ins:** `[brackets]`. Values marked **(BL)** come from [BASELINE_PARAMETERS.md](../../BASELINE_PARAMETERS.md) and must be re-checked at filing.

---

> **Federal Communications Commission — Space Bureau**
> **Re: Pre-space station notification under 47 CFR 97.207(g)(1) — amateur space station SMART-QSO**
>
> ### 1. Licensee and control
>
> | Item | Entry |
> |------|-------|
> | Space station licensee | `[full name]`, call sign KK6GDB (space station identifies as KK6GDB-11), license class `[class]`, FRN `[FRN]` |
> | Address / phone / email | `[ ]` |
> | 24-hour interference and space-safety contact | `[name, phone, email]` |
> | Control operators (97.207(a)) | `[call signs]` |
> | Telecommand stations designated under 97.211(a) | `[call sign — location lat/long]` ×2 minimum (RR 25.11) |
> | Organization (if any) | `[club / university / nonprofit]`; licensee's role `[ ]` |
> | Statement of amateur character | SMART-QSO is an amateur space station providing an FM transponder and plain-language telemetry to radio amateurs worldwide, and carries an amateur technical investigation of onboard machine-learning control of the transponder and beacon. No person is compensated for operating the station, no data or services are sold, and all transmission formats are published (97.113; ITU RR 1.56, 1.57, 25.2A). See [Licensing Pathway Determination](../LICENSING_PATHWAY_DETERMINATION.md). |
>
> ### 2. ITU information (97.207(g)(1): Articles 9 and 11, Appendix 4, Resolution 642)
>
> The Appendix 4 data is in [ITU_APPENDIX4_DATA.md](ITU_APPENDIX4_DATA.md) and is provided as a SpaceCap/SpaceCap-compatible
> electronic filing `[file name]` for the Commission to submit to the ITU Radiocommunication Bureau as advance
> publication information. **Resolution 642** (earth stations in the amateur-satellite service): the telecommand earth
> stations listed in §1 are the stations established before launch to terminate harmful interference (RR 22.1, 25.11).
>
> IARU coordination: letter dated `[date]`, coordinated frequencies `[ ]` (attached), request reference `[ ]`.
>
> ### 3. Space station summary
>
> | Item | Entry |
> |------|-------|
> | Spacecraft | 1U CubeSat, 100 × 100 × 113.5 mm (CDS Rev 14.1), mass `[ ]` kg **(BL)**, no propulsion, no pressure vessels, no pyrotechnics |
> | Launch | `[vehicle / mission]`, `[site]`, NET `[date]`; integrator `[ ]`; integration date `[ ]` |
> | Orbit | Circular, `[altitude]` km **(BL: ≤ 500 km target, 525 km maximum)**, inclination `[ ]`° **(BL: SSO ~97.4°)**; not maintained (no propulsion). Orbit decays naturally; see ODAR §6 |
> | Downlink | `[coordinated frequency]` MHz in 145.800–146.000 MHz; FM transponder downlink and AX.25 telemetry; transmitter 1 W maximum (0.5 W nominal), maximum EIRP −1 dBW (1 dB line loss, 0 dBi peak) **(BL)** |
> | Uplink | `[coordinated frequency]` MHz in 435.000–437.300 MHz (clear of the downlink's 3rd harmonic); FM transponder uplink (CTCSS 67.0 Hz) and authenticated telecommand |
> | Emissions | See ITU data sheet: transponder `16K0F3E`; telemetry `10K4F2D` (1200 bps AFSK/FM); telecommand `10K4F2D` (computed per ITU-R necessary-bandwidth formulas; confirm by measurement, TP-RF-004) |
> | Cessation of emissions (97.207(b)) | `TX_DISABLE` telecommand, received by the always-on 435 MHz receiver and executed by the on-board computer, which switches off the RF power amplifier supply in hardware and latches the state in non-volatile memory across resets; independent of the AI payload and of the transponder; accepted in every operating mode, and without authentication if no key is usable. Verified by TP-RF-006 `[date]` |
> | Loss of command contact | All transmitters inhibit after N days without an authenticated telecommand (N = 30 days at launch, 14 days after commissioning), receiver on; lifted by the next authenticated command |
>
> ### 4. Orbital debris mitigation (97.207(g)(1)(i)–(vii))
>
> The detailed analysis is in the attached Orbital Debris Assessment Report, prepared in the NASA-STD-8719.14C format
> using NASA DAS `[version 3.2.x]`: [ODAR_SMART-QSO.md](ODAR_SMART-QSO.md). Summary statements:
>
> **(i) Debris released in normal operations.** The operator has assessed and limited the amount of debris released in
> a planned manner during normal operations. SMART-QSO releases no debris. Deployable antennas and solar panels stay
> captive (CDS Rev 14.1 §2.1.1); burn-wire restraint line remnants stay attached. The CubeSat dispenser belongs to the
> launch provider/integrator and is not a separate deployment device of this space station.
>
> **(ii) Collision with small debris and meteoroids.** The operator has assessed and limited the probability that the
> space station will become a source of debris by collision with small debris or meteoroids that would cause loss of
> control and prevent disposal. Disposal is by passive atmospheric drag and needs no spacecraft function, so loss of
> control cannot prevent disposal. DAS probability: `[DAS 4.5-2 result]`, which is ≤ 0.01.
>
> **(iii) Accidental explosions and stored energy.** The operator has assessed and limited the probability of accidental
> explosions or release of liquids during and after the mission. There are no propellants, pressure vessels or
> pyrotechnics. The only stored-energy source is the Li-ion battery `[chemistry, Wh]` **(BL)**, protected against
> over-charge, over-discharge, over-current and short circuit. At end of mission the battery charge path is
> **permanently disconnected** by telecommand, and loads discharge the
> battery, leaving it in a permanent discharge state. See ODAR §4. `[Before filing: the passivation telecommand is specified in SQSO-ICD-013 §6.4 but not implemented in the flight software, and the autonomous backup (M days after the loss-of-contact inhibit) is proposed, not implemented (decision D3). State only what is implemented at filing.]`
>
> **(iv) Collision with large objects.** Probability of collision with objects ≥ 10 cm over the total orbital lifetime:
> `[DAS 4.5-1 result]` < 0.001.
> (A)(2) Orbit characteristics: SSO at `[altitude]` km is shared with many operational spacecraft; SMART-QSO has no
> propulsion, and the operator will share ephemeris and coordinate as below.
> (A)(3) Inhabitable spacecraft: during decay SMART-QSO will pass through the altitudes of the ISS (~415–420 km) and
> Tiangong (~380–390 km). Strategies: open ephemeris sharing, registration with 18 SDS, prompt response to conjunction
> data messages. No proximity operations. See the [space safety plan](SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md).
> (A)(4) Orbital parameters are not maintained (no propulsion). Expected evolution: circular decay from `[ ]` km to
> re-entry in `[ ]` years (DAS). The spacecraft cannot maneuver.
> (A)(5) Certification: the space station operator certifies that upon receipt of a space situational awareness
> conjunction warning, the operator will review and take all possible steps to assess the collision risk, and will
> mitigate the collision risk if necessary, including contacting the operator of any active spacecraft involved and
> sharing ephemeris data and other appropriate operational information. The space station cannot perform avoidance
> maneuvers.
>
> **(v) Trackability.** The smallest dimension exclusive of deployables is 100 mm, so the station is presumed trackable.
> Tracking is passive. (A) Identification after deployment by beacon Doppler matching against candidate TLEs and
> deployment sequence data from the provider. (B) Registered with the 18th Space Defense Squadron (Space-Track) before
> deployment: `[yes / date]`. (C) Deployment, ephemeris and contact data shared with 18 SDS, `[TraCSS]`, the launch
> provider and other operators on request.
>
> **(vi) Proximity operations.** None planned. No planned release of debris; explosion and collision risks as above.
>
> **(vii) Disposal.** (B) Disposal by uncontrolled atmospheric re-entry; no direct retrieval. Expected time in orbit
> after mission completion: `[DAS: ≤ 5 years]` (mission duration 2 years **(BL)**; deployment `[ ]` km; DAS lifetime
> `[ ]` years). (D)(1) Probability of disposal success ≥ 0.9: disposal is passive and needs no spacecraft function.
> The lifetime holds for solar activity per `[DAS solar flux table]`; sensitivity to low solar activity is shown in
> ODAR §6. (D)(2)(i) uncontrolled re-entry. (D)(2)(ii) DAS shows `[no component reaching the ground with kinetic
> energy > 15 J / list]`; casualty risk `[ ]` < 1:10,000.
>
> **(viii)** The licensee will file a replacement notification at least 90 days before integration if any material
> item changes.
>
> Signed: `[licensee]`  Date: `[ ]`

---

## Filing Checklist (internal)

| # | Item | Ready? |
|---|------|--------|
| 1 | Launch-vehicle determination date recorded (starts the 30-day clock) | ☐ |
| 2 | Integration date confirmed with integrator (90-day backstop) | ☐ |
| 3 | IARU coordination letter attached | ☐ |
| 4 | SpaceCap Appendix 4 file prepared from ITU_APPENDIX4_DATA.md | ☐ |
| 5 | DAS 3.2.x run with final mass/area; ODAR results inserted | ☐ |
| 6 | TX-OFF telecommand path verified by test (TP-RF-006, VI-RF-12) | ☐ |
| 6a | Passivation telecommand implemented and tested (SQSO-ICD-013 §6.4) | ☐ |
| 6b | Telecommand stations designated in writing (F13), with locations | ☐ |
| 7 | Space-Track registration complete | ☐ |
| 8 | Licensee signature | ☐ |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial draft (funding audit) |
| A.1 | 2026-10-27 | CDR review (communications panel): rule currency; filing plan dates; EIRP; uplink range; cessation-of-emissions text made accurate (one command path, through the on-board computer); loss-of-contact line; the "autonomous backup" passivation claim removed (not implemented) and the passivation command flagged; checklist items 6a, 6b |
