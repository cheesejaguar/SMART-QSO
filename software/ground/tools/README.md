# SMART-QSO Ground Software Tools

**Document ID**: SMART-QSO-GND-000
**Version**: 1.1
**Date**: 2026-10-27

Ground segment utilities for SMART-QSO satellite operations.

## Tools Overview

### beacon_decoder.py
Decodes AX.25 amateur radio frames into structured JSON format.

```bash
# Decode beacon from hex string
python beacon_decoder.py "A0A0A0A0A0A060..."

# Decode from file
python beacon_decoder.py -f beacon.hex -o decoded.json

# Read from stdin
cat beacon.hex | python beacon_decoder.py --stdin
```

### fl_update_builder.py
Builds FL-UPDATE v0.2 frames for federated learning model updates. Ground-side research only: community model updates are not in the flight baseline, and the flight software has no receiver for these frames (SW-11).

```bash
# Build frames from model data
python fl_update_builder.py -i model_gradients.bin -o frames/

# Build with manifest
python fl_update_builder.py -i weights.bin --name "qso-priority" --version "1.2.0"

# Validate a frame
python fl_update_builder.py --validate frame_001.bin
```

### telecommand_builder.py
Builds authenticated uplink telecommands (SQSO-ICD-013, `docs/interfaces/TELECOMMAND_AUTHENTICATION.md`).
The key file holds 64 hex characters and must be mode 600. Keys are never committed to any repository.
The counter file is shared by every telecommand station; each counter is reserved on disk before its frame is produced.

```bash
# List the command dictionary
python telecommand_builder.py --list

# Latch the transmitter off (47 CFR 97.207(b))
python telecommand_builder.py TX_DISABLE --key-file /secure/sqso.key --counter-file /secure/sqso.counter

# Set the beacon interval to 60 s, writing the binary frame to a file
python telecommand_builder.py SET_BEACON --args 003C --key-file /secure/sqso.key \
    --counter-file /secure/sqso.counter --out set_beacon.bin

# Wrap in an AX.25 UI frame and KISS-encode it for a TNC (spacecraft KK6GDB-11; use your own station's call sign as the source)
python telecommand_builder.py TX_DISABLE --key-file /secure/sqso.key \
    --counter-file /secure/sqso.counter --ax25 KK6GDB-11 N0CALL --kiss --out tx_off.kiss
```

Keep one counter file per key (operational and recovery), and start a new operational counter file after each KEY_ROTATE.

With two or more telecommand stations, counters come from UTC (SQSO-ICD-013 §5.2.1):
`counter = 4 × (UTC s since 2028-01-01) + station index`. Each station keeps its own counter file and a
`station.json` with its index; the builder refuses unless the kernel reports the clock synchronised with a maximum
error ≤ 0.5 s, and never goes below its own last counter or the newest acknowledged counter of any station.

```bash
echo '{"station": "KK6GDB", "station_index": 0}' > station.json
python telecommand_builder.py NOP --key-file /secure/sqso.key --counter-file /secure/op.ctr \
    --counter-from-utc --station-config station.json --newest-ack 2915678401 --ax25 KK6GDB-11 KK6GDB --kiss
```

### key_provision.py
Generates a telecommand key set and the GSE frames that load it through the umbilical (integration only; SQSO-ICD-013 §9.2).
Key files and frame files contain secrets: they are written mode 600 and are never overwritten. Keep `recovery.key` offline.

```bash
python key_provision.py generate --dir /secure/sqso-keys
python key_provision.py frames --dir /secure/sqso-keys --out /secure/frames.bin   # send over the umbilical, then delete
python key_provision.py zeroize --out /secure/zeroize.bin
```

### ephemeris_tool.py
Operator ephemeris from the onboard GNSS fixes (SQSO-GND-002, `docs/ground/ORBIT_DETERMINATION_AND_EPHEMERIS.md`):
decodes the GNSS state frames from beacon and telemetry archives (`gnss_frame.py`), fits the orbit by batch least
squares with an estimated drag scale (`orbit_dynamics.py`, `orbit_determination.py`), and writes a CCSDS OEM 3.0 with a
covariance per record, a TLE fitted with SGP4, and a report with residuals and the quality flag (GOOD, FAIR, POOR).

```bash
python ephemeris_tool.py --frames gnss_48h.jsonl --out-dir eph/ --gravity-file EGM2008.gfc --gravity-degree 12 \
    --object-id 2028-123A --norad-id 99999                        # ITRF2000 OEM
python ephemeris_tool.py --frames gnss_48h.jsonl --out-dir eph/ --frame EME2000 --dut1 0.071
```

Exit status 0 (GOOD or FAIR: upload), 2 (POOR: files prefixed NOT-FOR-UPLOAD_), 3 (FAIL: no products).
`gnss_simulation.py` makes simulated fixes for rehearsals.

### pass_predictor.py
Predicts satellite passes and calculates QSO fairness metrics.

```bash
# Predict passes for a single station
python pass_predictor.py --lat 37.4 --lon -122.0 --duration 48

# Analyze fairness across multiple stations
python pass_predictor.py --stations stations.json --fairness

# Use TLE for accurate prediction (requires sgp4)
python pass_predictor.py --lat 37.4 --lon -122.0 --tle smart-qso.tle
```

## Installation

```bash
cd software/ground/tools
pip install -r requirements.txt
```

## Requirements

- Python 3.9+
- numpy (the ephemeris tool; optional for the pass predictor)
- sgp4 (the ephemeris tool's TLE; accurate pass prediction)

## Station Configuration Format

For multi-station analysis, create a JSON file:

```json
{
  "stations": [
    {
      "name": "W6YX",
      "latitude": 37.4275,
      "longitude": -122.1697,
      "altitude": 30,
      "grid_square": "CM87wj"
    },
    {
      "name": "K6LRG",
      "latitude": 38.5449,
      "longitude": -121.7405,
      "altitude": 15,
      "grid_square": "CM98qm"
    }
  ]
}
```

## License

MIT License - See repository LICENSE file.
