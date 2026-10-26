//! Physical environment and spacecraft hardware models (no flight software).
//!
//! Everything is SI and `f64`; time `t` is seconds since J2000.0
//! (2000-01-01 12:00:00 UTC, TT − UTC ignored). The ECI frame is the mean
//! equator and equinox of date; ECEF is ECI rotated by GMST. Attitude
//! quaternions map body → ECI. The per-step functions do not allocate.
//!
//! Design values come from `docs/BASELINE_PARAMETERS.md` (which governs) and
//! the hardware documents it cites; each module documents its sources and
//! assumptions. The models and what they were checked against (unit tests):
//!
//! | Module | Model | Calibration / check |
//! |--------|-------|---------------------|
//! | [`math`] | `V3`, `Quat`, xoshiro256** `Rng`, constants | identities, moments of the deviates |
//! | [`time`] | civil ↔ J2000 seconds, GMST (IAU 1982) | Vallado example 3-5 |
//! | [`sun`] | *Astronomical Almanac* low-precision Sun (0.01°) | Vallado example 5-1, equinox/solstice |
//! | [`earth`] | ECI/ECEF, WGS84 geodetic, plate-to-Earth view factor, albedo 0.30, Earth IR 237 W/m² | round trips, view-factor continuity |
//! | [`orbit`] | mean elements + J2 secular rates, sun-synchronous design, drag decay da/dt = −ρ(CdA/m)√(μa) | 97.4° and 94.6 min at 500 km, LTAN held over a year |
//! | [`atmosphere`] | Bates–Walker diffusive equilibrium, Jacchia-style T∞, fitted to orbit-averaged NRLMSIS 2.1; solar-cycle F10.7 profile of `orbit_lifetime.py` | lifetime from 2028-06-01 (1.6 kg, Cd 2.2, 0.028 m²): 500 km **5.08 yr** (BASELINE 5.1), 450 km **2.29 yr** (2.3); wings failed 500 km 6.24 yr (6.2) |
//! | [`eclipse`] | conical umbra/penumbra, Earth + 20 km | 500 km, β = 0: **36.0 min** below half illumination of a 94.6 min orbit (SOLAR_ARRAY_DESIGN 35.8 min) |
//! | [`magfield`] | IGRF-14 tilted dipole with secular variation | 23.6–46.4 µT along a 500 km SSO |
//! | [`attitude`] | rigid body, Euler + RK4; gravity gradient, residual dipole, aero torque; stowed/deployed inertia | torque-free invariants; proportional B-dot (0.15 A·m²) from 10°/s → 0.16°/s in 3 orbits |
//! | [`solar`] | 8 strings × 2 AZUR 3G30A, temperature, irradiance, Kelly cosine, fluence + UV degradation, ray-cast self-shadowing, bypass diodes, albedo | EOL 500 km β = 0, before MPPT: tumbling **2.04 W** (×0.95 MPPT: 1.93 W; BASELINE 2.39 W without, SOLAR_ARRAY_DESIGN 2.14 W with shadowing), +Y sun-pointed **3.76 W** (3.58 W after MPPT; BASELINE 3.58 W); shadow factor 0.875 (doc 0.89) |
//! | [`battery`] | 4S1P INR18650-35E equivalent circuit (OCV, R₀, RC), coulombic efficiency, self-discharge, calendar + cycle fade, resistance growth | OCV 3.72 V at 50 %; 2-yr fade at 20 °C: 9.1 % at 10 % DoD, 3.5 % at the nominal 1.3 % DoD |
//! | [`thermal`] | 12-node lumped network (6 panels, 2 wings, battery, OBC stack, RF, Jetson) | energy balance at equilibrium; cold case (β ≈ 0, tumbling, 1 W) battery mean 10.7 °C (−3.5…23.2), +X panel swing 33 K; hot case (no eclipse, sun-pointed, 2.3 W) battery 22.5 °C |
//! | [`ground`] | 9-station network, look angles, range rate | 4–6 passes a day above 5° at mid latitudes (10 at Fairbanks) |
//! | [`link`] | SQSO-LBA-001 budget, dipole-shaped pattern, BER/frame-success curve | beacon 0.5 W at 5°, station A: **+7.7 dB** (−3 dBi) / +4.7 dB (−6 dBi), BASELINE ≈ +6 dB; pattern coverage 63/81/92 % |
//! | [`radiation`] | Poisson SEE (OBC reset, Jetson SEL, FRAM flip) weighted by SAA/polar region, TID | SAA 3.5 %, polar 38 % of the time; 1.5 krad(Si)/yr behind 2 mm Al |
//!
//! Assumptions that move results most: the solar-cycle profile after 2025
//! (lifetime), the global-mean density (no diurnal bulge in drag torque),
//! the Kelly cosine and low-irradiance voltage loss (tumbling power 10–15 %
//! below the document's cos-only method), the aging coefficients (no
//! flight-lot data), the conductances and heat capacities of the thermal
//! network (no EM correlation yet), and the SEE rates (no part-level test
//! data).

pub mod atmosphere;
pub mod attitude;
pub mod battery;
pub mod earth;
pub mod eclipse;
pub mod ground;
pub mod link;
pub mod magfield;
pub mod math;
pub mod orbit;
pub mod radiation;
pub mod solar;
pub mod sun;
pub mod thermal;
pub mod time;
