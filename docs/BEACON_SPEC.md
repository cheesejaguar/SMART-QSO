# Beacon & Federated Update Frames

## Telemetry Beacon (AX.25, 1200/9600 bps)
Fields (subset): time, mode, SOC, power forecast, fairness metrics, QSO stats, model hashes, last update outcome.

## Federated Update Frame (Open, In‑the‑clear)
```
FRAME: FL‑UPDATE v0.2
FIELDS:
  CALL: <callsign>
  GRID: <maidenhead>
  TYPE: {hyperparam|prompt|token}
  PATCH: <bounded list of float/int deltas or token text>
  HASH: <CRC32 of plaintext>
LIMITS:
  Max 256 bytes per frame; rate‑limited; bounded update magnitude.
VALIDATION:
  Schema + bounds; sandbox; KPI gate; rollback on regress.
```
