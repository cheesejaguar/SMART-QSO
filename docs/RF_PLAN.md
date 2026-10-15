# RF Plan (Baseline after funding audit, 2026-10-14)

Earlier revisions said "Uplink: None (transmit-only payload)". That contradicted the transponder mission and could not
be authorized: 47 CFR 97.207(b) and ITU RR 22.1 require a telecommand path that can stop all transmissions. The plan
below supersedes it. Authoritative values: [BASELINE_PARAMETERS.md](BASELINE_PARAMETERS.md).

- **Downlink:** one IARU-coordinated frequency in 145.800–146.000 MHz (2 m). Not 145.825 MHz, which carries ARISS/APRS packet
- **Uplink:** one IARU-coordinated frequency in 435–438 MHz (70 cm)
- **Modes:**
  - Single-channel FM transponder, U/V (uplink 70 cm, downlink 2 m), CTCSS-gated
  - AX.25 UI telemetry beacon, 1200 bps AFSK on FM, time-shared with the transponder downlink
  - Authenticated telecommand (HMAC + replay counter, 47 CFR 97.211(b)) on the uplink, from designated stations only
- **Beacon cadence:** 60–120 s, adaptive
- **Cessation of emissions:** TX-OFF telecommand removes PA power through a hardware switch, independent of the AI payload (IARU form 7.8)
- **RF silence:** no transmission earlier than 45 minutes after deployment (CDS Rev 14.1 §2.4.5)
- **Antennas:** deployable tape quarter-wave monopole for 2 m (≈ 0.5 m). The 70 cm receive antenna is a separate monopole or the 2 m element through a diplexer (decision in BASELINE_PARAMETERS)
- **Ground:** typical user station is an FM handheld or mobile rig with a handheld Yagi; telecommand stations use cross-Yagis with rotators (≥ 2 stations)
- **Emission designators:** 16K0F3E (transponder), 10K4F2D (telemetry and telecommand); see [ITU_APPENDIX4_DATA.md](regulatory/filings/ITU_APPENDIX4_DATA.md)
