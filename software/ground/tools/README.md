# SMART-QSO Ground Software Tools

**Document ID**: SMART-QSO-GND-000
**Version**: 1.0
**Date**: 2026-01-02

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
Builds FL-UPDATE v0.2 frames for federated learning model updates.

```bash
# Build frames from model data
python fl_update_builder.py -i model_gradients.bin -o frames/

# Build with manifest
python fl_update_builder.py -i weights.bin --name "qso-priority" --version "1.2.0"

# Validate a frame
python fl_update_builder.py --validate frame_001.bin
```

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
- numpy (optional, for pass predictor)
- sgp4 (optional, for accurate orbit propagation)

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
