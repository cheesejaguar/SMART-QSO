# RF Plan

**Updated:** 2026-10-27 (CDR review). Authoritative values: [BASELINE_PARAMETERS.md](BASELINE_PARAMETERS.md) §7.
Design: [hardware/RF_DESIGN.md](hardware/RF_DESIGN.md) (SMART-QSO-RF-001 v2.4). Link budget:
[environment/LINK_BUDGET_ANALYSIS.md](environment/LINK_BUDGET_ANALYSIS.md) (SQSO-LBA-001 v3.5).

Earlier revisions said "Uplink: None (transmit-only payload)". That contradicted the transponder mission and could not
be authorized: 47 CFR 97.207(b) and ITU RR 22.1 require a telecommand path that can stop all transmissions.

- **Downlink:** one IARU-coordinated frequency in 145.800–146.000 MHz (2 m). Not 145.825 MHz, which carries ARISS/APRS packet
- **Uplink:** one IARU-coordinated frequency, requested in **435.222–435.778 MHz, or else 436.222–436.500 MHz** (70 cm; BASELINE §7). The 3rd harmonic of
  any downlink in the segment falls on 437.400–438.000 MHz, inside 435–438 MHz, so the uplink must stay below
  437.300 MHz, at least 0.9 MHz below 3 × f_down (desense ≤ 0.44 dB with the 3rd harmonic at −70 dBc; BASELINE §7,
  SQSO-RF-BRD-001 §7.5); the receive SAW and the receiver clock limit
  it to 435.050–436.500 MHz (SQSO-RF-BRD-001), and the EPS converters' 1.000 MHz harmonics leave these windows clear by
  ≥ 200 kHz (SQSO-EPS-BRD-001, SQSO-EMC-001)
- **Modes:**
  - Single-channel FM transponder, U/V (uplink 70 cm, downlink 2 m), CTCSS 67.0 Hz access, duty-gated by power
    (`XPDR_EN` heartbeat, hardware key gate and 120 s key-down timer)
  - AX.25 UI telemetry beacon, 1200 bps AFSK on FM (±3.0 kHz), time-shared with the transponder downlink; the beacon
    takes the transmitter first
  - Authenticated telecommand (HMAC-SHA-256 + replay counter, 47 CFR 97.211(b)) on the uplink, from designated stations only
- **Beacon cadence:** 60 s ACTIVE, 120 s IDLE, 180 s SAFE (BASELINE §8), with a uniform ±5 s jitter on each interval as
  built: at the 8°/s spin (45 s period) it raises the frames a reference station decodes from 96.6 % to 98.2 %
  (SQSO-LBA-001 v3.5 §5.3)
- **Power:** 0.5 W nominal, 1 W maximum
- **Cessation of emissions:** TX-OFF telecommand removes PA power through a hardware switch (`RF_TX_EN`), independent of the
  AI payload (IARU form 7.8)
- **RF silence:** no signal generated or transmitted earlier than 45 minutes after deployment (CDS Rev 14.1 §2.4.5)
- **Antennas:** ISIS AntS 1U–3U, separate 2 m and 70 cm tape elements, no diplexer (BASELINE §7). Baseline
  configuration: the 2 m elements along ±X, across the spin axis (keeps +Y the major axis of inertia), the 70 cm
  elements along ±Y (`hardware/rf/ANTENNA_DESIGN.md` v2.3 §3; chair decision C-3)
- **Ground:** typical user station is an FM handheld or mobile rig, best with a handheld Yagi; telecommand stations use
  cross-Yagis with tracking, Doppler pre-correction to ±1 kHz, and ≥ 2 stations
  ([ground/GROUND_STATION_DESIGN.md](ground/GROUND_STATION_DESIGN.md))
- **Emission designators:** 16K0F3E (transponder), 10K4F2D (telemetry and telecommand); see [ITU_APPENDIX4_DATA.md](regulatory/filings/ITU_APPENDIX4_DATA.md)

| Date | Change |
|------|--------|
| 2026-10-14 | Funding audit: uplink and telecommand added; 145.825 MHz excluded |
| 2026-10-27 | CDR review: uplink range 435.000–437.300 MHz; antenna decided (separate elements, no diplexer); beacon cadence to BASELINE §8; CTCSS tone; jitter recommendation |
| 2026-10-27 | CDR 2026 consistency pass: uplink request windows 435.222–435.778 MHz, else 436.222–436.500 MHz (BASELINE Rev E §7) |
| 2026-10-27 | Documentation audit (post-CDR 2026): document versions cited; uplink ≥ 0.9 MHz below 3 × f_down (was ≥ 2 MHz preferred); beacon jitter as built at the 8°/s spin; antenna baseline ±X (C-3) |
