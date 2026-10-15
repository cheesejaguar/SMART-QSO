# ITU Appendix 4 Data Sheet (Advance Publication / Notification)
## Non-GSO Amateur-Satellite Network "SMART-QSO"

**Document Number:** SQSO-REG-017
**Revision:** A (draft)
**Date:** 2026-10-14
**Purpose:** Source data for the SpaceCap electronic filing that the FCC submits to the ITU Radiocommunication Bureau
under RR Articles 9 (advance publication) and 11 (notification), as required for the pre-space notification
(47 CFR 97.207(g)(1)). The IARU states that ITU registration in the amateur-satellite service is offered "without any
fee of cost recovery" (IARU coordination instructions).
**Status:** Values marked **(BL)** follow [BASELINE_PARAMETERS.md](../../BASELINE_PARAMETERS.md). Frequencies are placeholders until the IARU coordination letter is received. Data element names below are descriptive; the SpaceCap software assigns the official Appendix 4 item codes, station-class and nature-of-service codes. Do not hand-enter codes from this sheet without checking them in SpaceCap.

---

## A. General Characteristics

| Data | Value |
|------|-------|
| Satellite network name | SMART-QSO |
| Notifying administration | United States of America (FCC) |
| Service | Amateur-satellite (RR 1.57) |
| Type | Non-geostationary, 1 satellite |
| Date of bringing into use | `[planned launch + ~1 day]` |
| Period of validity | Mission life **2 years (BL)**; natural decay after that |
| Responsible operating agency | Space station licensee `[name, call sign]` |
| Resolution 642 earth (telecommand) stations | `[call sign — lat/long]` ×2 minimum |

## B. Orbital Characteristics (single plane)

| Data | Value |
|------|-------|
| Number of orbital planes / satellites per plane | 1 / 1 |
| Reference body | Earth |
| Inclination | `[~97.4°]` SSO **(BL)**; backup ISS deployment 51.6° |
| Apogee / perigee altitude | `[450–525]` km circular **(BL)**; backup 400–420 km |
| Period | `[93.6–95.1]` min (93.6 min at 450 km, 95.1 min at 525 km; 92.6 min at 400 km) |
| Station-keeping | None (no propulsion); altitude decays |
| Right ascension of ascending node / LTAN | `[per launch]` |

## C. Space Station Transmitting Assignments (space-to-Earth)

| Data | Downlink 1: FM transponder | Downlink 2: telemetry beacon |
|------|---------------------------|------------------------------|
| Frequency band | 144–146 MHz amateur-satellite allocation; IARU satellite sub-band 145.800–146.000 MHz | same transmitter, same assigned frequency |
| Assigned frequency | `[IARU-coordinated, e.g. 145.9xx]` MHz | same |
| Necessary bandwidth / emission | **16K0F3E** (FM telephony; peak deviation 5 kHz, max audio 3 kHz: Bn = 2M + 2D = 16 kHz) | **10K4F2D** (1200 bps AFSK on FM; Bell 202 tones 1200/2200 Hz, peak deviation 3 kHz: Bn = 2M + 2D = 2×2.2 + 2×3 = 10.4 kHz) |
| Maximum transmitter output power | `[0.5–1.0]` W **(BL)** = `[−3 to 0]` dBW | same |
| Maximum power density | `[power − 10·log10(Bn)]` dBW/Hz (compute in SpaceCap) | same |
| Antenna | Deployable quarter-wave monopole (tape), ~0 dBi peak, near-omnidirectional, linear polarization **(BL)** | same |
| Service area | Worldwide | Worldwide |

Note: one VHF transmitter carries both functions, time-multiplexed. The beacon is sent at a fixed cadence with the
transponder briefly muted; this is simpler and lower-power than a second transmitter in a 1U.

## D. Space Station Receiving Assignments (Earth-to-space)

| Data | Uplink 1: FM transponder | Uplink 2: telecommand |
|------|-------------------------|-----------------------|
| Frequency band | 435–438 MHz (amateur-satellite) | same receiver, same frequency |
| Assigned frequency | `[IARU-coordinated, e.g. 435.xxx]` MHz | same |
| Emission | 16K0F3E with CTCSS access tone `[ ]` Hz | 10K4F2D (1200 bps AFSK/FM, authenticated per 97.211(b)) |
| Receiver noise temperature | `[~1000]` K (system, Earth-facing) **(BL; confirm by radio datasheet)** | same |
| Antenna | Deployable quarter-wave monopole for 70 cm, or the VHF antenna through a diplexer **(BL decision)**, ~0 dBi | same |

## E. Associated Earth Stations (typical)

| Data | Typical amateur user station | Telecommand station |
|------|------------------------------|---------------------|
| Antenna | Handheld 2-element Yagi to fixed cross-Yagi; gain 5–12 dBi | Cross-Yagi with rotator; gain ~12–14 dBi |
| Transmit power (435 MHz) | 5–50 W | 25–100 W |
| Receive noise temperature (145.9 MHz) | 500–1000 K | ~300 K with LNA |
| Location | Worldwide | `[lat/long]` ×2 |

## F. Items to Confirm Before Filing

1. Coordinated frequencies from the IARU letter.
2. Maximum transmitter power and antenna configuration from the final RF design, measured in TP-RF-002 and TP-RF-004 (occupied bandwidth must not exceed the necessary bandwidth filed).
3. Orbit from the launch agreement (launch-vehicle determination).
4. Whether the FCC files under the ITU's short-duration-mission procedure (WRC-19 Resolution 32) or the standard API/notification path. This is the FCC's choice, and the amateur-satellite status should be confirmed with Space Bureau staff.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial data sheet (funding audit) |
