# Space System Protection and Security Plan

**Document ID**: SQSO-SEC-001
**Version**: B
**Date**: 2026-10-27
**Status**: Draft (design review RID SE-11)

## 1. Purpose and basis

This plan states the threats to SMART-QSO's command, telemetry, ground and supply chain, and the controls that answer
them. NASA-STD-1006A (Space System Protection Standard) is mandatory only for NASA missions; it is used here as the
standard of practice, tailored to an amateur satellite whose emissions must be in the clear (47 CFR 97.113(a)(4)) and
whose telecommand may be authenticated but not encrypted (47 CFR 97.211(b)). The command-authentication design is
`docs/interfaces/TELECOMMAND_AUTHENTICATION.md` (SQSO-ICD-013); the ground stations are `docs/ground/GROUND_STATION_DESIGN.md`
(SQSO-GND-001).

The two protection properties that matter are **command integrity** (only the designated stations can change the
spacecraft's state) and **cessation of emissions** (the licensee can always stop the transmitter: 97.207(b),
ITU RR 22.1). Confidentiality is not a goal: nothing is secret except the keys.

## 2. Threats and controls

| # | Threat (NASA-STD-1006A threat category) | Effect if unmitigated | Controls | Verification |
|---|----------------------------------------|-----------------------|----------|--------------|
| T1 | Command spoofing: anyone with a 70 cm transmitter sends a command | Mode change, transponder on, passivation | HMAC-SHA-256 (truncated tag) over every command, per-key monotonic counter persisted in FRAM; unauthenticated commands rejected except TX-OFF when no key is usable (ICD-013 §4–§6) | Host tests and test vectors (ICD-013 §11); flatsat (TP-SWI) |
| T2 | Replay of a recorded command | Repeats a past action | Counter must exceed the stored value; counters are UTC-derived per station (`4 × (UTC s since 2028-01-01) + station index`, ICD-013 §5.2.1) so two stations never collide | Host tests |
| T3 | Key compromise at a ground station | An attacker commands as that station | Four operational keys (one active, three spares) and an offline recovery key that can rotate to a spare (ICD-013 §9); keys never in the repository, loaded only through the umbilical; station key custody rules (GND-R6) | Key-rotation test on the flatsat |
| T4 | Loss of every key or every station | No command possible | TX-OFF accepted without authentication when no key is usable; loss-of-contact transmit inhibit after N days (BASELINE §7); receiver always on | Host tests; SIL contact-outage scenario |
| T5 | Malicious or erroneous passivation | Mission ends | Two-step command: arm with the operational key, execute with the offline recovery key within 600 s (ICD-013 §6.4); two independent hardware lines to the one-shot disconnect (EPS_DESIGN §7) | Host tests; EPS EM test V-BSA-7 |
| T6 | Uplink jamming or interference | Commands lost | Stations radiating ≥ 64 dBm EIRP above 10° elevation (≥ 100 W PEP into ≥ 15.5 dBic, or ≤ 50 W PEP into ≥ 17.8 dBic in the 47 CFR 2.106 US270 areas; decision C-14) with Doppler pre-correction, ≥ +8.0 dB margin at 10° with a 6 dB interference allowance (BASELINE §7); two stations ≥ 1,000 km apart; loss-of-contact timeout 14–30 days gives time to recover | Link budget (SQSO-LBA-001 3.5 §6.1); SIL outage cases |
| T7 | Downlink interference to other services | Regulatory | TX-OFF by any designated station; duty gates; spurious ≥ 70 dB at the third harmonic | RF tests TP-RF-003 |
| T8 | Malicious content through the AI payload | Prohibited content broadcast | Deterministic content filter on both sides; no generated numbers; templates as fallback (BEACON_SPEC §6) | Shared filter vectors, fuzzing |
| T9 | Flight-software supply chain (dependencies, toolchain) | Hidden behaviour | Pinned Rust toolchain; `Cargo.lock` locked; `unsafe` forbidden in `sqso-protocol` and `sqso-fsw`; CI on every change; release package with hashes (`docs/development/RELEASE_PROCEDURE.md`) | CI |
| T10 | Hardware supply chain (counterfeit parts) | Early failure | Authorised distributors only, lot traceability (`docs/manufacturing/EEE_PARTS_LIST.md` §2A.2) | Receiving inspection |
| T11 | Ground-segment compromise (station PC, key files) | T3 | Keys on an offline host; command generation on a host not connected to the internet during passes; logs kept | Station checklist (SQSO-GSOM-001) |
| T12 | Pre-launch tampering at the integrator | Altered flight unit | Configuration and seal record at delivery; key load and ARM_FLIGHT witnessed; as-run procedures | Launch campaign checklist (SQSO-OPS-005) |

## 3. Residual risk

- Commands and telemetry are in the clear by law: anyone can read the state of the spacecraft and the command
  counters. This is accepted.
- A determined jammer can deny commanding for a while; the loss-of-contact inhibit then silences the transmitter until
  a station gets through, which is the safe outcome.
- There is no on-orbit software upload in the baseline, so the software attack surface after launch is the command
  dictionary only.

## 4. Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-27 | Initial issue (design review, RID SE-11) |
| B | 2026-10-27 | Documentation audit (post-CDR 2026): T6 ground stations at the ≥ 64 dBm EIRP of decision C-14 (was 50 W PEP) and SQSO-LBA-001 3.5 (was 3.0); T10 cites EEE_PARTS_LIST §2A.2 without the superseded revision |
