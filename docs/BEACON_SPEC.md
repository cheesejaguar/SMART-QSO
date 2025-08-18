# Stensat Beacon Spec — 1200 bps AFSK, Personified LLM Status

This document defines how SMART‑QSO uses a Stensat radio beacon to transmit AX.25 Unnumbered Information (UI) frames at 1200 bps AFSK that contain concise, personified LLM‑generated status updates, along with a compact key–value telemetry tail for machine parsing.

Reference: Stensat Radio Beacon — frequency bands 2 m/70 cm, UART data interface at 38.4 kbit/s, AX.25 UI at 1200 bps AFSK and 9600 bps FSK supported [link](http://www.stensat.org/products/product1.html).

## Radio and Modem Settings
- Data rate: 1200 bps AFSK (Bell 202 tones: 1200/2200 Hz); NRZI; HDLC flags 0x7E
- Frame type: AX.25 UI (Control 0x03, PID 0xF0)
- Preamble: 200 ms (flags), Tail: 50 ms (flags)
- Beacon interval: adaptive 15–180 s (see Scheduling), default 60 s
- Bands: 2 m or 70 cm per mission config; no digipeater path
- Identification: Callsign embedded in AX.25 address fields and again in the human text body (every frame)

## AX.25 Envelope
- Destination: `BEACON` (SSID `-0`)
- Source: `<CALLSIGN>` (TBD)
- Path: none (space ops; no terrestrial digipeating)
- Control: `0x03` (UI)
- PID: `0xF0` (no L3)
- Info Field: ASCII, ≤ 200 bytes

### Info Field Structure
Human‑readable personified line, followed by a compact machine tail.

```
<TEXT> | T=<ISO8601Z> M=<MODE> SOC=<pct> SUN=<0|1> RF=<0|1> PWR=<GOOD|FAIR|LOW> QSO=<n>
```

- <TEXT>: LLM‑personified status, ASCII, ≤ 120 bytes
- Tail: space‑separated key–values, ≤ 80 bytes
- Total Info length: ≤ 200 bytes (recommended) to ensure robust receive margins at 1200 bps

Examples of Info Field:
```
de SMARTQ-1: I’m sunlit and feeling energetic—SOC 78%, transponder ACTIVE. 73! | T=2025-08-18Z M=ACTIVE SOC=78 SUN=1 RF=1 PWR=GOOD QSO=12
de SMARTQ-1: Coasting in eclipse, conserving power—SOC 34%. Back soon. | T=2025-08-18Z M=IDLE SOC=34 SUN=0 RF=0 PWR=LOW QSO=3
```

## LLM‑Personified Message Generation
Because 1200 bps AFSK has tight payload budgets, the onboard wording generator is constrained and deterministic:

- Inputs: UTC time, `MODE` (ACTIVE/IDLE/SAFE), battery `SOC` (0–100), `SUN` (0/1), `RF` (0/1), short‑horizon power forecast class (`PWR`: GOOD/FAIR/LOW), recent `QSO` rate.
- Output: one sentence, first‑person, ≤ 120 ASCII bytes; friendly but neutral, no emojis, no encoded punctuation beyond ASCII.
- Style rules:
  - Always begin with `de <CALLSIGN>:`
  - Mention sun exposure or forecast and SOC where relevant
  - If `MODE=SAFE` or `SOC<25`, emphasize conservation
  - If `RF=1`, mention “transponder ACTIVE”
  - End with `73!` when not in SAFE
- Safety/fallback: if generator fails or would exceed 120 bytes, fall back to a compact template:
  - `de <CALL>: SOC=<pct> <SUN|ECLIPSE> RF=<ON|OFF>. 73!`

### Template Set (non‑exhaustive)
- Sunlit, healthy: `de {CALL}: I’m sunlit and feeling energetic—SOC {SOC}%, transponder ACTIVE. 73!`
- Sunlit, conserve: `de {CALL}: Sunlit but conserving—SOC {SOC}%. RF {RFSTATE}. 73!`
- Eclipse: `de {CALL}: Coasting in eclipse, conserving power—SOC {SOC}%.`
- Forecast low: `de {CALL}: Clouds ahead in the power budget—SOC {SOC}%, planning lighter ops. 73!`

## Scheduling and Rate Control
- Base period: 60 s
- Adaptive rules:
  - If `MODE=SAFE` or `SOC<20%`: 180 s
  - If `SOC 20–40%`: 120 s
  - If eclipse (`SUN=0`) and `RF=0`: +30 s to the above
  - Immediate beacon on `MODE` transition or first RF activation (debounced ≥ 10 s)
- Quiet hours / regulatory: Ensure callsign present each frame; never encrypt; content strictly mission status

## UART Integration with Stensat Beacon
- Physical link: UART 38.4 kbit/s, 8N1 (per Stensat spec) [link](http://www.stensat.org/products/product1.html)
- Host (OBC) responsibilities:
  - Construct AX.25 UI frame bytes (addresses → Control 0x03 → PID 0xF0 → Info → FCS) and supply them to the beacon module using the module’s UART command/API as documented in the vendor datasheet
  - Manage preamble/tail timing if required by module firmware; otherwise rely on module defaults
  - Rate‑limit messages per Scheduling
- Implementation note: The exact UART command set/framing is vendor‑specific; integrate per the Stensat datasheet while keeping the Info payload and timing defined here.

## Telemetry Mapping (Tail Fields)
- **T**: ISO‑8601 UTC time truncated to seconds, e.g., `2025-08-18Z`
- **M**: `ACTIVE|IDLE|SAFE`
- **SOC**: integer percent (0–100)
- **SUN**: `1` sunlit, `0` eclipse
- **RF**: `1` transponder on, `0` off
- **PWR**: `GOOD|FAIR|LOW` (short‑horizon energy forecast class)
- **QSO**: recent QSO events (rolling minute)

## Example Packet (Field View)
```
DEST=BEACON SRC=SMARTQ-1 CTRL=0x03 PID=0xF0
INFO="de SMARTQ-1: I’m sunlit and feeling energetic—SOC 78%, transponder ACTIVE. 73! | T=2025-08-18Z M=ACTIVE SOC=78 SUN=1 RF=1 PWR=GOOD QSO=12"
```

## Compliance
- Amateur‑Satellite Service: plain‑text status; no encryption
- Callsign is present in AX.25 and again in the text line
- Keep Info ≤ 200 bytes to preserve link margin at 1200 bps

