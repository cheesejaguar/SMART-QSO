# Interface Control Document: Beacon Format

**Document Number**: SMART-QSO-ICD-006
**Version**: 1.1
**Date**: 2026-10-27
**Status**: Superseded. The beacon format is defined by `docs/BEACON_SPEC.md`, which describes the frame the flight
software builds (`software/rust/sqso-fsw/src/beacon.rs`). This file is kept as a pointer so that existing references
resolve.

---

## 1. Where the Beacon Format Is Defined

| Subject | Governing source |
|---------|------------------|
| Radio, modem, AX.25 envelope, information field, telemetry fields, text sources, content rules, schedule | `docs/BEACON_SPEC.md` §1–§8 |
| Beacon telemetry tail, housekeeping frames HK1 and HK2, the GNSS state frame, stored-record dumps | `docs/operations/TELEMETRY_DICTIONARY.md` (SQSO-TLM-001) §4; `docs/interfaces/ICD_SPACE_GROUND.md` (SQSO-ICD-005) §5.3–§5.6 |
| Frequencies, power, emissions | `docs/BASELINE_PARAMETERS.md` §7 |
| OBC-to-TNC framing (KISS) | `docs/interfaces/ICD_OBC_RF.md` (SQSO-ICD-003) §5 |
| Jetson text requests | `docs/interfaces/ICD_OBC_JETSON.md` (SMART-QSO-ICD-002) §4 |

## 2. What Version 1.0 Described and Why It Is Withdrawn

Version 1.0 (2026-01-02) described a beacon that the flight software does not send: a placeholder call sign
"KK6XXX" with SSID 0, a 16-byte header with a sequence number and HHMM time, comma-separated telemetry for a 2S battery
(`BV=7.8V`, 6.0–8.4 V) and a 5 V bus, a 175-byte text with templates and an example that put numbers, a 420 km altitude
and an e-mail request in the AI text, and beacon intervals of 30 s (ACTIVE, IDLE) and 60 s (SAFE). The baseline has the
call sign KK6GDB-11, the information field `de KK6GDB-11: <text> | T=… M=… SOC=… BV=… SUN=… RF=… QSO=…`, a 4S
battery (BASELINE §5), no numbers in generated text (BASELINE §6; BEACON_SPEC §6) and intervals of 60 s (ACTIVE),
120 s (IDLE) and 180 s (SAFE) (BASELINE §8). Its content is in the repository history.

## Appendix C: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): superseded by `docs/BEACON_SPEC.md`; the Version 1.0 format (placeholder call sign, 2S telemetry, 30 s intervals, numbers in AI text) withdrawn and replaced by a pointer |
