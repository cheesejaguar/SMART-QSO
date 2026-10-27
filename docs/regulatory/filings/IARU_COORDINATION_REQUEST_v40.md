# IARU Amateur Satellite Frequency Coordination Request — Draft (form v40 layout)
## SMART-QSO

**Document Number:** SQSO-REG-010
**Revision:** A.5 (draft, not submitted)
**Date:** 2026-10-27
**Form:** IARU *Amateur Satellite Frequency Coordination Request* **v40** (effective 15 May 2020; "earlier versions will no longer be accepted" — https://www.iaru.org/reference/satellites/, page updated 26 Jul 2026)
**Submit to:** satcoord@iaru.org, by the **prospective space station licensee**, as a .doc/.pdf named `smartqso_req YYMMDD` (form checklist item 1). Large files go by URL (checklist item 2).
**Supersedes:** `docs/regulatory/IARU_COORDINATION.md` Rev A, which mis-stated the operator, mission scope and uplink (see that file's Rev B notice).

Transfer each answer below into the corresponding numbered box of the official v40 form. `[brackets]` = fill-in.
**(BL)** = value from [BASELINE_PARAMETERS.md](../../BASELINE_PARAMETERS.md).

---

### 1. Administrative information
- **1.1 Date of this submission:** `[DD-MMM-YYYY]`
- **1.2 Submission revision number:** 0

### 2. Spacecraft identification
- **2.1 Name:** SMART-QSO (must match the API name in [ITU_APPENDIX4_DATA.md](ITU_APPENDIX4_DATA.md))
- **2.2 Notifying administration:** United States of America (FCC)
- **2.3 API/A number:** `[API/A special section]`, or if not yet published: "API data sent to the FCC with the pre-space notification on `[date]`"

### 3. Licensee of the space station
- **3.1–3.7:** `[given name, family name]`, call sign **KK6GDB** (decision D1), `[licensed since, postal address, telephone, email]`
- **3.8:** Licensee acknowledges publication of the request (signature/date)

### 4. Organizations
- **4.1 Organization:** `[club / university / nonprofit, or "none — individual amateur"]`
- **4.2 Licensee's position:** `[ ]`
- **4.3–4.6:** `[address, phone, email, project URL — public repository: https://github.com/cheesejaguar/SMART-QSO]`
- **4.7 National Amateur Radio Society:** ARRL, The National Association for Amateur Radio (IARU member society for the USA)
- **4.8 Endorsement:** AMSAT-NA `[yes/no]`; ARRL `[yes/no]`. **Seek written support before submitting.**
- **4.9 Contacts:** `[names and emails at AMSAT-NA / ARRL]`
- **4.10 Will anyone be financially compensated for operating the satellite or ground stations?** **No**
- **4.11:** — (Funding for construction and launch is by grants/donations with nothing required in return. No operations compensation; no data rights. See [pathway determination](../LICENSING_PATHWAY_DETERMINATION.md) §5, §7.)

### 5. Space station mission and frequencies
- **5.1 Type of mission:** ☒ Amateur `[or ☒ Amateur combined with Educational, if a school/university leads]`
- **5.2 Do amateur-band transmitters/receivers control or retrieve data from non-amateur subsystems?** No. There are no non-amateur subsystems or frequencies.
- **5.4 Mission description:** SMART-QSO is a 1U CubeSat carrying (1) a **single-channel U/V FM transponder** for worldwide amateur voice contacts (QSOs); (2) a **1200 bps AX.25 telemetry beacon** in plain text; and (3) an **amateur technical investigation** of onboard machine learning. A low-power AI computer schedules the transponder within the power available, balances fair access across regions, and composes plain-language beacon text. If the AI is off or faulty, the spacecraft falls back to a conventional transponder and template beacon.
- **5.5 Compliance with RR 1.56, 1.57 and Article 25; contribution; global participation:** All transmissions are amateur communications or telemetry, in the clear, with formats published before launch. The project is carried out by licensed amateurs for self-training and technical investigation, without pecuniary interest. It contributes an open, documented example of onboard autonomy for amateur transponders, and open-source flight and ground software. Amateurs participate by **using the FM transponder for QSOs**, decoding telemetry (SatNOGS-compatible), and reporting reception to the public dashboard. Any future community model-update experiment will be published openly and enabled only by designated telecommand stations; public uplinks never alter spacecraft operation.
- **5.6 Amateur satellite bands used:** ☒ 144–146 MHz ☒ 435–438 MHz
- **5.7 Planned duration:** Commissioning ~2 weeks (beacon only); transponder operations 2 years **(BL)**; then end-of-mission passivation and natural decay (see ODAR)

**Transmitting (space-to-Earth):**
- **5.8 Frequencies and functions:** One frequency in 145.800–146.000 MHz: FM transponder downlink, time-shared with the 1200 bps AX.25 telemetry beacon. *We do not request 145.825 MHz, which carries ISS/ARISS packet and APRS digipeater traffic.* The 3rd harmonic of the downlink falls in 437.4–438.0 MHz; we ask that the pair keep the uplink at least 2 MHz below 3 × the downlink (see 5.14).
- **5.9 Tuning range / step:** `[per radio datasheet, e.g. 144–146 MHz, 2.5 kHz steps]`
- **5.10 EIRP:** +26 dBm at 0.5 W and +29 dBm at 1 W maximum (1 dB line loss, 0 dBi peak antenna) **(BL)**
- **5.11 ITU emission designators:** 16K0F3E (transponder), 10K4F2D (telemetry)
- **5.12 Common description:** FM voice transponder; 1200 bps AFSK (Bell 202) AX.25 UI frames on FM
- **5.13 Antenna:** ISIS AntS deployable tape elements, ~0 dBi peak; quarter-wave 2 m elements along ±X, across the spin axis **(BL)** (`hardware/rf/ANTENNA_DESIGN.md` §3). Spacecraft spins at 8°/s about the Sun-pointing axis (5.25), so the pattern null moves slowly across the sky

**Receiving (Earth-to-space):**
- **5.14 Requested receive frequency and function:** One frequency in **435.222–435.778 MHz**, or if none is available there, in **436.222–436.500 MHz** **(BL)**: FM transponder uplink (CTCSS 67.0 Hz) and telecommand. These windows are where the spacecraft's own receiver works best: the receive SAW filter's temperature drift and the receiver clock's harmonic at 435.000 MHz limit the band to 435.050–436.500 MHz, and the power-system converters, synchronised to 1.000 MHz, leave the uplink clear by at least 200 kHz only inside these windows. Both lie below the downlink's 3rd harmonic (437.4–438.0 MHz); within them we ask for a frequency at least 2 MHz below 3 × the coordinated downlink where one is available (5.8)
- **5.15 Tuning range / step:** `[per radio datasheet]`
- **5.16 / 5.17 Emissions:** 16K0F3E (user uplink); 10K4F2D 1200 bps AFSK/FM (telecommand)
- **5.18 Receiver noise temperature:** 1,000 K system at the antenna port, budgeted (requirement ≤ 650 K for the receiver with a 290 K source; `docs/hardware/RF_DESIGN.md` RF-P006)
- **5.19 Antenna:** separate 70 cm deployable tape element (no diplexer) **(BL)**, ~0 dBi

**Other information:**
- **5.22 Physical structure:** 1U CubeSat 100 × 100 × 113.5 mm, mass `[ ]` kg **(BL)**; deployable VHF (and UHF) monopoles; `[body-mounted / deployable]` solar panels **(BL)**. Drawings: `[repository URL]/hardware/structure/`
- **5.23 Functional description:** OBC (STM32L4R5ZI MCU, flight software in Rust, no RTOS); EPS with a 4S Li-ion battery (≈ 50 Wh); U/v FM transponder built from the 435 MHz telecommand receiver and the 2 m transmitter, gated in hardware (CTCSS, OBC heartbeat, 120 s key-down timer); KISS TNC for the beacon and telecommand; magnetorquer ADCS; GNSS receiver; AI payload (NVIDIA Jetson Orin Nano, power-gated, burst-only). URL: `[repo]/docs/hardware/RF_DESIGN.md`, `[repo]/docs/ARCHITECTURE.md`
- **5.24 Power budget:** `[orbit-average generation / consumption by mode]` **(BL)**. URL: `[repo]/docs/environment/POWER_BUDGET_ANALYSIS.md`
- **5.25 Attitude stabilization:** Magnetorquer B-dot detumble, then spin-stabilized coarse Sun-pointing: +Y (the solar-wing normal) on the Sun, spinning at 8°/s. No precision pointing.
- **5.26 Service area:** Worldwide

### 6. Encryption of downlink
- **6.1 Will all transmissions have published, publicly available details?** ☒ Yes
- **6.3 Responsible person:** `[name, call, email]`
- **6.4 Signature:** `[ ]`

### 7. Telecommand information (not published)
- **7.1 Telecommand station call signs:** `[call 1]`, `[call 2]` (minimum two, RR 25.11)
- **7.2 Locations:** `[lat/long]`
- **7.3 Telecommand frequency:** the coordinated 435–438 MHz uplink frequency
- **7.4 / 7.5 Emission:** 10K4F2D, 1200 bps AFSK on FM, AX.25 framing
- **7.6 Link budget:** `[repo]/docs/environment/LINK_BUDGET_ANALYSIS.md` v3.4 §6.1 (475 km; telecommand stations radiate ≥ 64 dBm EIRP (2.5 kW) toward the spacecraft above 10° elevation (decision C-14): variant A, outside the 47 CFR 2.106 US270 areas, ≥ 100 W PEP at the feed into ≥ 15.5 dBic; variant B, inside them (the Bay Area site near Beale AFB, Fairbanks near Clear SFS), ≤ 50 W PEP at the feed into ≥ 17.5 dBic (a stacked pair of cross-Yagis, ≤ 0.5 dB feed); telecommand margin with the 10th-percentile spacecraft gain and a 6 dB interference allowance +6.5 dB at 5° and +8.3 dB at 10°, less a 0.3 dB pointing allowance; commands only above 10°; analysis). Telecommand stations pre-correct Doppler to ±1 kHz
- **7.7 Telecommand encryption:** Commands are authenticated, not encrypted: HMAC-SHA-256 with a secret key held only by the licensee, plus a monotonically increasing command counter for replay protection (permitted by 47 CFR 97.211(b)). Command content is otherwise in the clear. Keys are loaded before launch through the umbilical; a compromised key is replaced on orbit by switching to a pre-loaded spare, so no key material is ever transmitted.
- **7.8 How telecommand stations turn off the transmitter immediately, even with user traffic or computer failure:** The receiver is always on and is not switched by any inhibit. A `TX_DISABLE` telecommand (authenticated; accepted without authentication only if no key is usable) is decoded by the on-board computer, which drives the power-amplifier supply switch off and latches the state in non-volatile memory, so it persists across resets. User traffic does not block it: the receiver hears the command channel whatever the transponder is doing. The path does not depend on the AI payload or the transponder logic, and nothing in the transponder can power the amplifier. If the computer fails, its independent watchdog resets it within about 32 s, and during a reset or with the computer unpowered the switch is held off by a pull-down. There is one command path, through the on-board computer; there is no separate decoder in the radio. Verified by TP-RF-006 (`docs/hardware/RF_DESIGN.md` VI-RF-12).
- **7.9 Mechanism if command contact is lost:** If no valid authenticated command is received for N days, flight software inhibits the transponder and beacon (receive-only); the next authenticated command lifts the inhibit, and a reset does not. N is 30 days at launch and is set to 14 days by telecommand once commissioning is complete (settable 1–90 days). *(Proposed, subject to the FCC consultation: after a further 180 days, never within the first year after launch, the end-of-mission passivation sequence runs: battery charge path permanently disconnected, transmitter disabled. The autonomous passivation is not yet implemented.)*

### 8. Telemetry
- **8.1 Telemetry frequency:** same as the 5.8 downlink
- **8.2 Emission:** 10K4F2D
- **8.3 Description:** 1200 bps AFSK AX.25 UI frames, plain ASCII
- **8.4 Link budget:** `[repo]/docs/environment/LINK_BUDGET_ANALYSIS.md` v3.2 §4 (475 km; 1200 bps beacon at 0.5 W: +7.9 dB at 5° elevation for a 12 dBic station at the median spacecraft gain, +0.9 dB at the 10th percentile; analysis)
- **8.5 Format:** AX.25 UI; field definitions and engineering-unit conversions in `docs/BEACON_SPEC.md` and `docs/operations/TELEMETRY_DICTIONARY.md` (RR 25.2A). Decoder: `software/ground/beacon_decoder.py` (open source)

### 9. Launch plans
- **9.1 Launch agency:** `[integrator]` on SpaceX Transporter `[n]` **(BL)**; backup ISS/NRCSD
- **9.2 Launch location:** Vandenberg SFB, California `[confirm]`
- **9.3 Expected launch date:** `[about 2028-11]` (planning date: the last Transporter flight of 2028) **(BL)**
- **9.4–9.7 Orbit:** apogee/perigee `[475]` km target, 465–485 km (not above 485 km) **(BL)**, inclination `[~97.4°]`, period `[93.9–94.3]` min; descending node about 10:30 local time
- **9.8 Other amateur satellites on the same launch:** `[if known]`

### 10. Typical transmitting earth station
- **10.1** FM handheld or mobile transceiver, 5–50 W, on 435 MHz with a handheld 2-element Yagi (≈ 5 dBi) or cross-Yagi
- **10.2 Link budget:** `[repo]/docs/environment/LINK_BUDGET_ANALYSIS.md` v3.0 §6.2–6.3: 5 W with a handheld Yagi reaches 12 dB SINAD at the spacecraft from 5°; 5 W with a whip from about 12°

### 11. Typical receiving earth station
- **11.1** 2 m FM receiver or SDR with a handheld Yagi or omni antenna; telemetry decoded with open-source software (Dire Wolf / SatNOGS)
- **11.2 Link budget:** same, §4.2–4.3: a handheld Yagi hears the transponder at 20 dB SINAD from 5°; a handheld with a whip at 12 dB SINAD from about 18°

### 12. Additional information
- Public repository with all designs and documentation: `[URL]`
- The spacecraft follows the CDS Rev 14.1 RF silence of 45 minutes after deployment

### 13. Certification
☒ The licensee has reviewed all relevant laws, rules and regulations, certifies that the request complies as understood by the IARU, and confirms operation without pecuniary interest per RR 1.56/1.57. **Commercial interests: none.**

### 14. Signature
`[licensee signature and date]`

---

## Pre-Submission Checklist

| # | Item | Status |
|---|------|--------|
| 1 | Licensee named and licensed (pathway D1) | ☒ KK6GDB; ☐ two telecommand stations agreed (F13) |
| 2 | AMSAT-NA/ARRL consulted (4.8–4.9) | ☐ |
| 3 | FCC Space Bureau pre-filing consultation done (pathway D3) | ☐ |
| 4 | Baseline RF parameters frozen (power, antenna, receiver) | ☐ |
| 5 | Uplink link budget added to LINK_BUDGET_ANALYSIS.md | ☒ (v2.0 §4, analysis; replace assumptions with EM measurements, §7 of the budget) |
| 6 | Telecommand authentication implemented as described in 7.7 (implemented and tested in flight-software simulation per `docs/interfaces/TELECOMMAND_AUTHENTICATION.md`; target receiver driver and hardware test TP-RF-006 open, findings SW-02/SW-03) | ☐ |
| 7 | TX-OFF hardware path in RF design (7.8) | ☒ designed (`docs/hardware/RF_DESIGN.md` v2.0 §3.5, SQSO-ICD-003 v2.0); ☐ built and tested (VI-RF-12) |
| 8 | Signed by licensee | ☐ |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial draft on v40 layout (funding audit) |
| A.1 | 2026-10-16 | Items 7.6, 8.4 and checklist item 5 cite the re-issued link budget (TB-05) |
| A.1, note | 2026-10-24 | Item 5.23: dated note that the OBC software has no RTOS (correct before submission) |
| A.1, note | 2026-10-25 | Item 5.23: the note gives the rebaselined text (no RTOS; BASELINE §6, CR-2026-002). Not submitted |
| A.2 | 2026-10-27 | CDR review (communications panel): licensee call sign; uplink range 435.000–437.300 MHz and the harmonic request (5.8, 5.14); EIRP; antenna and attitude (5.13, 5.25); receiver noise temperature and antenna (5.18, 5.19); 5.23 functional description rebaselined; link-budget items to SQSO-LBA-001 v3.0 (7.6, 8.4, 10.2, 11.2); 7.8 rewritten to describe the single command path accurately (the "second, independent path" did not exist). Not submitted |
| A.3 | 2026-10-27 | CDR 2026 consistency pass: uplink request 435.222–435.778 MHz, else 436.222–436.500 MHz, with the reason (5.14; BASELINE Rev E §7); 2 m elements along ±X and spin 8°/s (5.13, 5.25); launch about 2028-11 and orbit 475 km target, 465–485 km, period 93.9–94.3 min (9.3–9.7). Not submitted |
| A.4 | 2026-10-27 | Items 7.6 and 8.4: link budget values from SQSO-LBA-001 v3.2 at 475 km (RID SE-16) |
| A.5 | 2026-10-27 | Item 7.6: telecommand EIRP requirement and the two station variants of decision C-14 (LBA v3.4) |
