# Orbital Lifetime Estimator

Independent cross-check of SMART-QSO post-mission orbital lifetime for the
orbital debris disclosure (47 CFR 97.207(g)(1)(vii)) and the ODAR
(`docs/regulatory/filings/ODAR_SMART-QSO.md`).

**This is not a substitute for NASA DAS 3.2.x.** The FCC rule names DAS (or a
higher-fidelity tool); every value that goes into a filing must be reproduced
in DAS with the final mass properties.

## Model

- NRLMSIS 2.1 atmosphere (`pymsis`), orbit-averaged over latitude, local time and season
- Circular-orbit decay, `da/dt = -rho (Cd A / m) sqrt(mu a)`
- Solar activity: constant F10.7 bounds (70 = low/bounding, 120 = moderate) and an
  approximate solar-cycle profile whose cycle 25 decline and cycle 26 amplitude are
  **assumptions** (see `CYCLE_POINTS`); replace them with the current NOAA SWPC
  prediction (https://www.swpc.noaa.gov/products/solar-cycle-progression) before formal use

## Usage

```bash
pip install -r requirements.txt
python orbit_lifetime.py --inc 97.4 --alt 475 500 525 --mass 1.33 --area 0.01 0.02
python -m pytest -q
```
