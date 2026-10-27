# ITU Appendix 4 Data Sheet (Advance Publication / Notification)
## Non-GSO Amateur-Satellite Network "SMART-QSO"

**Document Number:** SQSO-REG-017
**Revision:** A.4 (draft)
**Date:** 2026-10-27
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
| Apogee / perigee altitude | `[475]` km circular target, 465–485 km, 485 km maximum **(BL)**; backup 400–420 km |
| Period | `[93.9–94.3]` min (94.1 min at 475 km; 92.6 min at 400 km) |
| Station-keeping | None (no propulsion); altitude decays |
| Right ascension of ascending node / LTAN | `[per launch]` |

## C. Space Station Transmitting Assignments (space-to-Earth)

| Data | Downlink 1: FM transponder | Downlink 2: telemetry beacon |
|------|---------------------------|------------------------------|
| Frequency band | 144–146 MHz amateur-satellite allocation; IARU satellite sub-band 145.800–146.000 MHz | same transmitter, same assigned frequency |
| Assigned frequency | `[IARU-coordinated, e.g. 145.9xx]` MHz | same |
| Necessary bandwidth / emission | **16K0F3E** (FM telephony; peak deviation 5 kHz, max audio 3 kHz: Bn = 2M + 2D = 16 kHz) | **10K4F2D** (1200 bps AFSK on FM; Bell 202 tones 1200/2200 Hz, peak deviation 3 kHz: Bn = 2M + 2D = 2×2.2 + 2×3 = 10.4 kHz) |
| Maximum transmitter output power | 1.0 W (0 dBW) maximum, 0.5 W nominal **(BL)** | same |
| Maximum power density | `[power − 10·log10(Bn)]` dBW/Hz (compute in SpaceCap) | same |
| Antenna | ISIS AntS deployable quarter-wave tape element(s), 0 dBi peak, linear polarization (elliptical if two elements in quadrature are chosen), spinning spacecraft **(BL)**; maximum EIRP −1 dBW | same |
| Service area | Worldwide | Worldwide |

Note: one VHF transmitter carries both functions, time-multiplexed. The beacon is sent at a fixed cadence with the
transponder briefly muted; this is simpler and lower-power than a second transmitter in a 1U.

## D. Space Station Receiving Assignments (Earth-to-space)

| Data | Uplink 1: FM transponder | Uplink 2: telecommand |
|------|-------------------------|-----------------------|
| Frequency band | 435–438 MHz (amateur-satellite); assignment requested in 435.222–435.778 MHz, or else 436.222–436.500 MHz (BASELINE §7), clear of the downlink's 3rd harmonic | same receiver, same frequency |
| Assigned frequency | `[IARU-coordinated, e.g. 435.xxx]` MHz | same |
| Emission | 16K0F3E with CTCSS access tone 67.0 Hz | 10K4F2D (1200 bps AFSK/FM, authenticated per 97.211(b)) |
| Receiver noise temperature | 1,000 K (system, Earth-facing, at the antenna port; receiver requirement ≤ 650 K, `docs/hardware/RF_DESIGN.md` RF-P006) | same |
| Antenna | Separate deployable quarter-wave tape element for 70 cm, no diplexer **(BL §7)**, 0 dBi peak | same |

## E. Associated Earth Stations (typical)

| Data | Typical amateur user station | Telecommand station |
|------|------------------------------|---------------------|
| Antenna | Handheld 2-element Yagi to fixed cross-Yagi; gain 5–12 dBi | Cross-Yagi with rotator, ≥ 15.5 dBic (variant A), or a stacked pair of cross-Yagis, ≥ 17.8 dBic (variant B) |
| Transmit power (435 MHz) | 5–50 W | ≥ 64 dBm EIRP (2.5 kW) toward the spacecraft above 10° elevation (decision C-14): variant A, outside the 47 CFR 2.106 US270 areas, ≥ 100 W PEP at the feed into ≥ 15.5 dBic; variant B, inside them (the Bay Area site near Beale AFB, Fairbanks near Clear SFS), **≤ 50 W PEP** at the feed into ≥ 17.8 dBic (a stacked pair of cross-Yagis, ≤ 0.5 dB feed); EIRP ≈ 2.5–3.5 kW at boresight; transmitting only above 10° elevation (SQSO-GND-001 Rev H §3.1; BASELINE Rev E §7) |
| Receive noise temperature (145.9 MHz) | 500–1000 K | 500 K with LNA (galactic noise alone is ≈ 480 K at 146 MHz, ITU-R P.372) |
| Location | Worldwide | `[lat/long]` ×2 |

## F. Items to Confirm Before Filing

1. Coordinated frequencies from the IARU letter.
2. Maximum transmitter power and antenna configuration from the final RF design, measured in TP-RF-002 and TP-RF-004 (occupied bandwidth must not exceed the necessary bandwidth filed).
3. Orbit from the launch agreement (launch-vehicle determination).
4. Whether the FCC files under the ITU's short-duration-mission procedure (Resolution 32 (WRC-19): validity ≤ 3 years, no extension; notification may accompany the API and counts as received 4 months after the API is published) or the standard API/notification path (RR 9.1: API preferably ≥ 2 years before bringing into use, which a late-2028 launch already misses; that is allowed). Res. 32 fits the 2-year design life; ask for the standard path if an extended mission is wanted. This is the FCC's choice; confirm with Space Bureau staff in the D3 consultation.
5. API publication takes about 2–3 months after the Bureau receives the data; the FCC's own forwarding time is to be asked. Planned submission of this data to the FCC: 2027-01-19 (REGULATORY_ROADMAP Rev D §3).

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial data sheet (funding audit) |
| A.1 | 2026-10-27 | CDR review (communications panel): power and EIRP, antenna, uplink range and CTCSS tone, receiver noise temperature, telecommand-station power limits, Res. 32 and API timing |
| A.2 | 2026-10-27 | CDR 2026 consistency pass: altitude 475 km target, 465–485 km, period 93.9–94.3 min, uplink request windows (BASELINE Rev E §3, §7) |
| A.3 | 2026-10-27 | §E telecommand station: the EIRP requirement of decision C-14 and the two station variants (≥ 100 W into ≥ 15.5 dBic outside US270; ≤ 50 W PEP into ≥ 17.5 dBic inside US270) |
| A.4 | 2026-10-27 | Documentation audit (post-CDR 2026): §E variant B ≥ 17.8 dBic (BASELINE Rev E §7; was 17.5), GND-001 Rev H; §F the late-2028 planning launch and REGULATORY_ROADMAP Rev D |
