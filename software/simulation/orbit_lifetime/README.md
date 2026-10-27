# Orbital Lifetime Estimator

Independent cross-check of SMART-QSO post-mission orbital lifetime for the
orbital debris disclosure (47 CFR 97.207(g)(1)(vii)) and the ODAR
(`docs/regulatory/filings/ODAR_SMART-QSO.md`).

The lifetime table of record (`docs/BASELINE_PARAMETERS.md` §3, ODAR Draft D §6.2) comes from
`software/simulation/cdr/sma/debris_lifetime.py`; this estimator is the independent cross-check.

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
# Baseline case (BASELINE §3): 465-485 km, 1.6 kg (1.85 kg bound), wings deployed 0.0247 m^2 / failed 0.0164 m^2,
# deployment 2028-11-01. The script's own defaults (1.33 kg, 0.015 m^2, 2027-06-01) predate the baseline.
python orbit_lifetime.py --inc 97.31 --alt 465 475 485 --mass 1.6 --area 0.0247 0.0164 --start 2028-11-01
python -m pytest -q
```
