# IARU Amateur Satellite Frequency Coordination Request — Draft (form v40 layout)
## SMART-QSO

**Document Number:** SQSO-REG-010
**Revision:** A (draft, not submitted)
**Date:** 2026-10-14
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
- **3.1–3.7:** `[given name, family name, call sign, licensed since, postal address, telephone, email]`
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
- **5.8 Frequencies and functions:** One frequency in 145.800–146.000 MHz: FM transponder downlink, time-shared with the 1200 bps AX.25 telemetry beacon. *We do not request 145.825 MHz, which carries ISS/ARISS packet and APRS digipeater traffic.*
- **5.9 Tuning range / step:** `[per radio datasheet, e.g. 144–146 MHz, 2.5 kHz steps]`
- **5.10 EIRP:** `[+27 to +30]` dBm (0.5–1.0 W, 0 dBi antenna) **(BL)**
- **5.11 ITU emission designators:** 16K0F3E (transponder), 10K4F2D (telemetry)
- **5.12 Common description:** FM voice transponder; 1200 bps AFSK (Bell 202) AX.25 UI frames on FM
- **5.13 Antenna:** Deployable quarter-wave tape monopole, ~0 dBi, near-omnidirectional; attitude magnetically controlled (see 5.25)

**Receiving (Earth-to-space):**
- **5.14 Requested receive frequency and function:** One frequency in 435–438 MHz: FM transponder uplink (CTCSS-gated) and telecommand
- **5.15 Tuning range / step:** `[per radio datasheet]`
- **5.16 / 5.17 Emissions:** 16K0F3E (user uplink); 10K4F2D 1200 bps AFSK/FM (telecommand)
- **5.18 Receiver noise temperature:** `[ ]` K (from radio datasheet)
- **5.19 Antenna:** `[70 cm monopole or diplexed VHF monopole]` **(BL)**, ~0 dBi

**Other information:**
- **5.22 Physical structure:** 1U CubeSat 100 × 100 × 113.5 mm, mass `[ ]` kg **(BL)**; deployable VHF (and UHF) monopoles; `[body-mounted / deployable]` solar panels **(BL)**. Drawings: `[repository URL]/hardware/structure/`
- **5.23 Functional description:** OBC (low-power MCU, FreeRTOS); EPS with Li-ion battery; VHF/UHF transceiver with FM transponder mode; magnetorquer ADCS; AI payload (NVIDIA Jetson Orin Nano, power-gated, burst-only). URL: `[repo]/docs/ARCHITECTURE.md`
- **5.24 Power budget:** `[orbit-average generation / consumption by mode]` **(BL)**. URL: `[repo]/docs/environment/POWER_BUDGET_ANALYSIS.md`
- **5.25 Attitude stabilization:** Magnetorquer B-dot detumble, then low-rate spin/coarse sun-pointing. No precision pointing.
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
- **7.6 Link budget:** `[repo]/docs/environment/LINK_BUDGET_ANALYSIS.md` (uplink section, to be added; see findings register)
- **7.7 Telecommand encryption:** Commands are authenticated, not encrypted: HMAC-SHA-256 with a secret key held only by the licensee, plus a monotonically increasing command counter for replay protection (permitted by 47 CFR 97.211(b)). Command content is otherwise in the clear.
- **7.8 How telecommand stations turn off the transmitter immediately, even with user traffic or computer failure:** A TX-OFF command is decoded by the radio/OBC command path, which removes power from the RF power amplifier through a hardware switch. This path does not depend on the AI payload or the transponder application. A second, independent path is the OBC watchdog-protected command that latches TX inhibit in non-volatile memory, so it persists across resets. Both are verified in `[TP-RF-00x]`.
- **7.9 Mechanism if command contact is lost:** If no valid authenticated command is received for `[N]` days **(BL)**, flight software inhibits the transponder and beacon (receive-only). After a further `[M]` days the end-of-mission passivation sequence runs: battery charge path permanently disconnected, transmitter disabled.

### 8. Telemetry
- **8.1 Telemetry frequency:** same as the 5.8 downlink
- **8.2 Emission:** 10K4F2D
- **8.3 Description:** 1200 bps AFSK AX.25 UI frames, plain ASCII
- **8.4 Link budget:** `[repo]/docs/environment/LINK_BUDGET_ANALYSIS.md`
- **8.5 Format:** AX.25 UI; field definitions and engineering-unit conversions in `docs/BEACON_SPEC.md` and `docs/operations/TELEMETRY_DICTIONARY.md` (RR 25.2A). Decoder: `software/ground/beacon_decoder.py` (open source)

### 9. Launch plans
- **9.1 Launch agency:** `[integrator]` on SpaceX Transporter `[n]` **(BL)**; backup ISS/NRCSD
- **9.2 Launch location:** Vandenberg SFB, California `[confirm]`
- **9.3 Expected launch date:** `[2028]`
- **9.4–9.7 Orbit:** apogee/perigee `[450–525]` km, inclination `[~97.4°]`, period `[93.6–95.1]` min
- **9.8 Other amateur satellites on the same launch:** `[if known]`

### 10. Typical transmitting earth station
- **10.1** FM handheld or mobile transceiver, 5–50 W, on 435 MHz with a handheld 2-element Yagi (≈ 5 dBi) or cross-Yagi
- **10.2 Link budget:** `[repo link]` (uplink closure at 5° elevation for 5 W + 5 dBi)

### 11. Typical receiving earth station
- **11.1** 2 m FM receiver or SDR with a handheld Yagi or omni antenna; telemetry decoded with open-source software (Dire Wolf / SatNOGS)
- **11.2 Link budget:** `[repo link]`

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
| 1 | Licensee named and licensed (pathway D1) | ☐ |
| 2 | AMSAT-NA/ARRL consulted (4.8–4.9) | ☐ |
| 3 | FCC Space Bureau pre-filing consultation done (pathway D3) | ☐ |
| 4 | Baseline RF parameters frozen (power, antenna, receiver) | ☐ |
| 5 | Uplink link budget added to LINK_BUDGET_ANALYSIS.md | ☐ |
| 6 | Telecommand authentication implemented as described in 7.7 (current code does not; see findings) | ☐ |
| 7 | TX-OFF hardware path in RF design (7.8) | ☐ |
| 8 | Signed by licensee | ☐ |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial draft on v40 layout (funding audit) |
