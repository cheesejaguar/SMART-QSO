# Beacon Specification: 1200 bps AX.25 Beacon with Filtered AI Text

This document defines the SMART-QSO telemetry beacon as the flight software builds it. It is an AX.25 UI frame carrying
one short line of plain-language text, followed by measured telemetry. Design values come from
`docs/BASELINE_PARAMETERS.md`, which governs where the two disagree. The information-field layout is also in
`docs/operations/TELEMETRY_DICTIONARY.md` section 4.

Implementation: `software/rust/sqso-fsw/src/beacon.rs` (frame, telemetry, scheduling; C interface `software/rust/sqso-ffi/src/beacon.rs`), `software/rust/sqso-protocol/src/filter.rs`
(content rules), `software/payload/formatter.py` and `software/payload/content_filter.py` (Jetson text).

## 1. Radio and Modem

- 1200 bps AFSK (Bell 202, 1200/2200 Hz), NRZI, HDLC flags `0x7E`, on the 145.8–146.0 MHz downlink
  (`docs/RF_PLAN.md`), 0.5 W nominal, 1 W maximum.
- No beacon is sent while any transmit inhibit is active: TX-OFF (97.207(b)), loss of contact, or the 45-minute
  post-deployment RF silence (`rf_control_tx_permitted()`).

## 2. AX.25 Envelope

| Field | Value |
|---|---|
| Destination | `CQ`, SSID 0 |
| Source | `KK6GDB-11` (licensee call sign KK6GDB, SSID 11; `software/flight/include/station_id.h`) |
| Path | none (no digipeating) |
| Control / PID | `0x03` (UI) / `0xF0` (no layer 3) |
| Information field | ASCII, at most 256 bytes (section 3) |

## 3. Information Field

```
de KK6GDB-11: <text> | T=<s since boot> M=<SAFE|IDLE|ACTIVE> SOC=<%> BV=<V.d> SUN=<0|1> RF=<0|1> QSO=<count>
```

Example (host simulation): `de KK6GDB-11: Hello from orbit! | T=1792115511 M=ACTIVE SOC=83 BV=14.4 SUN=1 RF=1 QSO=-` (the 4S pack is
14.4 V nominal, 16.4 V at the 4.10 V/cell end of charge; BASELINE §5)

- `T=` is the OBC's mission time in seconds **since the last OBC boot** (`hal_timer_get_ms()` / 1000 through
  `smart_qso_now_ms()`; it restarts at 0 at every reset), not UTC. In the host simulation the same clock is the host's
  wall-clock time, which is why the example reads like a Unix time. UTC is the mission clock's UNIX time in housekeeping
  HK2 `RT` (RV-3028-C7), or comes from correlating the beacon's `T=` with the time of reception.
- The call sign appears in the source address and again at the start of every information field (97.119).
- `<text>` is at most 176 characters, the room left after the telemetry fields at their widest. The text is cut if
  needed; the telemetry is never cut.
- Every telemetry value is a measurement (section 4). A field with no measurement, or a measurement older than 60 s, is
  sent as `-`. Decoders treat `-` as "not measured", never as zero.

## 4. Telemetry Fields

| Key | Meaning | Source |
|---|---|---|
| `T` | Seconds since the last OBC boot (not UTC; host simulation: host wall-clock seconds) | OBC mission time, `smart_qso_now_ms()` / 1000 |
| `M` | Power mode | EPS control (`eps_get_power_mode`) |
| `SOC` | Battery state of charge, % | EPS battery measurement |
| `BV` | Battery voltage, V to one decimal | `BV` sensor, if read within the last 60 s |
| `SUN` | 1 sunlit, 0 eclipse | EPS (solar input) |
| `RF` | Radio power switch, 1 on, 0 off | EPS switch state |
| `QSO` | QSOs counted | Not measured yet: always `-` until a QSO detector exists |

The OBC records measurements with `beacon_update_measurements()`, each field with a validity flag. In the host
simulation the EPS model supplies SOC and SUN; on the target the EPS driver does (SW-01).

## 5. Text Sources

1. **Jetson text.** The OBC sends `BEACON_REQ` after each `TELEMETRY` line (`docs/interfaces/ICD_OBC_JETSON.md` §4).
   The Jetson answers with `BEACON_RSP` text. Text whose sequence number matches the latest request is used for the next
   beacon, once, if it passes the content rules (section 6).
2. **OBC templates.** Otherwise the OBC sends the next of its fixed templates (`beacon_templates[]`, from `TEMPLATES` in `software/rust/sqso-fsw/src/beacon.rs`). None of
   them states a health condition; the telemetry fields carry the state.

On the Jetson, text comes from the language model or, when the model is absent or its output fails the same content
rules, from health in words: "Safe mode, conserving power.", "Battery low, conserving power.", or
"<Sunlit | In eclipse>, transponder <on | off>. 73!". The model is prompted with health words only (no numbers),
sampled at temperature 0.3, and held to a grammar that admits letters and plain punctuation only. Without a model the
runner returns no text, never a canned status.

Model choice is decision D5 (`docs/payload/AI_MODEL_SELECTION.md`).

## 6. Content Rules

Every beacon text not taken from the OBC's own templates must pass these rules. The OBC applies them to everything it
receives (`beacon_filter_check()`); the Jetson applies the same rules to its own output first
(`content_filter.check()`). A text is rejected for the first rule it breaks:

| Order | Result | Rule | Basis |
|---|---|---|---|
| 1 | LENGTH | 1 to 176 characters | Frame size |
| 2 | CHARSET | Letters, digits, space and `. , ! ? ' " : ; ( ) / - &` only. No `\|` (it separates the telemetry, so the text cannot spoof it), `$`, `@`, `%`, `=` or control characters | 97.113(a)(4) |
| 3 | COMMERCIAL | No URL fragments: `http`, `www.`, `.com`, `.net`, `.org` | 97.113(a)(3) |

Then, word by word (a word is a run of letters and digits):

| Order | Result | Rule | Basis |
|---|---|---|---|
| 4 | ENCODED | More than 20 characters | 97.113(a)(4), obscured meaning |
| 5 | CALLSIGN | Shaped like a call sign but not KK6GDB, or an SSID after `KK6GDB-` other than 11 | 97.113(a)(4), false identification; 97.119 |
| 6 | NUMERIC | Digits anywhere except `73`, `1U`, the call sign and its SSID. Only the OBC writes numbers, from measurements, so the text cannot state a false value | 97.113(a)(4), false or deceptive messages |
| 7 | ENCODED | Five or more letters with no vowel (a e i o u y), eight or more hex digits only, or more than two lower-to-upper case changes | 97.113(a)(4) |
| 8 | PROFANITY | A word on the profanity list | 97.113(a)(4) |
| 9 | COMMERCIAL | A word on the commercial list (buy, sell, price, sponsor and similar) | 97.113(a)(3) |

- Word lists are whole-word and case-insensitive: `PROFANITY` and `COMMERCIAL` in
  `software/rust/sqso-protocol/src/filter.rs`, the one filter the OBC and the Jetson (`content_filter.py`) both run.
- Shared accept/reject vectors are in `software/flight/tests/vectors/beacon_filter_vectors.txt`, run by both suites, with
  fuzz properties on each side.
- The OBC counts rejected texts (`ai_rejected_count` in the beacon state) and sends a template instead.
- Before flight, the licensee reviews the word lists, and the reject rate is measured on the selected model's output
  (D5 gate).

## 7. Scheduling

| Power mode | Interval |
|---|---|
| ACTIVE | 60 s (the baseline rate; the beacon power line assumes ~2 s of transmission per 60 s) |
| IDLE | 120 s |
| SAFE | 180 s |

`BEACON_INTERVAL_*_MS` in `beacon.h`; no mode beacons faster than ACTIVE. Each interval carries a uniform ±5 s random
jitter, so that successive beacons do not fall at the same phase of the 8°/s spin (RID RF-04; SQSO-LBA-001 §5.3).

## 8. Compliance Summary

- Plain text in the clear; nothing encrypted or encoded (97.113(a)(4); `docs/regulatory/`).
- Station identification in every frame (97.119).
- Content rules in section 6 cover obscene, commercial, encoded and false content mechanically. The licensee remains
  responsible for content (97.103).
- Transmitter can be stopped by telecommand, independent of the AI payload (97.207(b), 97.211(b)).

*Revision note (2026-10-27, documentation audit (post-CDR 2026)): the example's battery voltage for the 4S pack (was
8.1 V, a 2S value); the ±5 s beacon-interval jitter (§7).*

*Revision note (2026-10-27, documentation audit, follow-up): §3–§4 the `T=` field is seconds since the last OBC boot
(`smart_qso_now_ms()`, the HAL time base; it restarts at every reset), not seconds since the Unix epoch; the example is
from the host simulation, whose clock is the host wall clock; UTC is HK2 `RT`.*
