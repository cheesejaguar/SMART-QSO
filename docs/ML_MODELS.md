# ML Models

- **Power Forecast (LSTM‑style, INT8, <100 kB)**
- **Signal Classifier (tiny CNN, <50 kB)**
- **Anomaly Detector (autoencoder, <50 kB)**
- **TinyLM for Beacons (Transformer ~1 MB, INT8)**

## On‑orbit Learning
- Gradient accumulation with bounded steps; sunlit only; checkpoint to FRAM.
- Federated updates: community submits small, bounded patches; sandbox validated.
