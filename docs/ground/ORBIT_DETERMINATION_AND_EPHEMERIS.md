# Orbit Determination and Operator Ephemeris
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-GND-002
**Revision:** A
**Date:** 2026-10-27
**Owner:** Ground segment lead (SQSO-GND-001 §8)
**Status:** CDR design. The ground tool is implemented and verified against a simulated orbit, reading the GNSS state
frame as the flight software writes it (§3); the end-to-end demonstration with the integrator and SpaceX (VI-LV-13) is
open.

---

## 1. Purpose

The launch provider requires a forward-predicted operator ephemeris, with covariance, from onboard GNSS, to Space-Track
and SpaceX Space Traffic Coordination **within 24 h of first contact**, then after every pass and at least every 24 h
until the 18th SDS has catalogued the object (SpaceX Rideshare User's Guide v11 §9.5.7.2; LVR-089 in
`docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`; RID LV-04). For the baseline orbit the first pass over the Bay Area comes
0.7–12.2 h after separation, so the first ephemeris is due 24.7–36.2 h after separation
(`docs/operations/MISSION_TIMELINE.md`).

This document covers the ground half of that chain: decoding the GNSS fixes from the downlink, orbit determination,
the products (CCSDS OEM and TLE), their quality flag, the accuracy verified in simulation, and the 24 h procedure.
The onboard half (receiver sessions, the GNSS state frame) is in the flight software (`gnss_schedule.rs`, `gnss.rs`,
`gnss_frame.rs`; SRS-F084, SRS-F158) and the telemetry dictionary (SQSO-TLM-001).

| Item | Where |
|------|-------|
| Ground tool | `software/ground/tools/ephemeris_tool.py` (command line); `gnss_frame.py`, `orbit_frames.py`, `orbit_dynamics.py`, `orbit_determination.py`, `ephemeris_products.py`; simulation `gnss_simulation.py` |
| Tests | `software/ground/tests/test_orbit_determination.py` (32 tests, run in CI by `.github/workflows/program-checks.yml`) |
| Procedure | §7 here; the operator step in SQSO-GSOM-001 §5.3 |
| Owner | Ground segment lead: the tool, its configuration (gravity file, drag area), the procedure and the uploads. The licensee (KK6GDB) is the operator contact on Space-Track and approves the first upload |

## 2. Requirements

| ID | Requirement | Source |
|----|-------------|--------|
| OD-R1 | First operator ephemeris within 24 h of first contact; then after every pass and at least every 24 h until catalogued | UG §9.5.7.2 |
| OD-R2 | Forward-predicted, from GNSS fixes, with covariance | UG §9.5.7.2 |
| OD-R3 | CCSDS OEM (CCSDS 502.0-B-3), KVN, UTC, ITRF or EME2000, a state every 60 s over 3–7 days, a covariance with the states | `docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md` §3a; D6 research §6.2 |
| OD-R4 | A two-line element set for the integrator, the 18th SDS and amateur tracking | RID LV-04 |
| OD-R5 | Residuals and a quality flag with every product; no product uploaded that the flag does not allow | RID LV-04 |
| OD-R6 | Dead-reckoned receiver output never used | D6 research §6.3; `gnss.rs` |

## 3. GNSS Fixes on the Ground

### 3.1 The GNSS state frame

The flight software sends one GNSS state frame after each receiver session that ended with a fix, and repeats the
latest on `GET_HOUSEKEEPING` (0x09) (`software/rust/sqso-fsw/src/gnss_frame.rs`, SRS-F158; SQSO-TLM-001). Plain
ASCII `KEY=value`, integers, `-` for a value not available:

```
GNS N=<session> WK=<GPS week> TOW=<GPS time of week, ms> X=<m> Y=<m> Z=<m> VX=<mm/s> VY=<mm/s> VZ=<mm/s> VDT=<s>
    Q=<GGA fix quality> SV=<satellites used> PD=<PDOP x 10> AGE=<s>
```

ECEF, WGS-84. The receiver's state record carries no velocity: the flight software derives it from the session's first
accepted fix and this one, `VDT` seconds apart (10–120 s), so its noise is about √2 × the position sigma / `VDT`
(0.24 m/s per axis for 5 m over 30 s), and it is `-` when the session had no usable chord. `WK` as the receiver
reports it (the piNAV-NG reports a 10-bit week: the tool resolves the rollover to the week nearest the reception time,
or to the era that began 2019-04-07). `Q` 1 (GPS) or 2 (differential) is a real fix; anything else is refused. `AGE`
is the fix's age when the frame was sent; the repeat on `GET_HOUSEKEEPING` keeps `N`, `WK` and `TOW` and is dropped as
a duplicate.

**One decoder.** `parse_gnss_frame()` in `gnss_frame.py` is the only function that knows the layout; unknown keys are
ignored and decimals accepted. Its test vectors (`TestGnssFrame`) use the flight software's own format.

### 3.2 Input

`--frames` takes the station or mission-database archives: JSON lines (`info` as text, or `frame_hex`: the AX.25 frame
from the address field, with or without FCS, checked with the shared protocol code `sqso_protocol`; `rx_utc`,
`station`) or text lines. Only frames from `KK6GDB-11` are read. The same frame heard by several stations counts once.

### 3.3 Time

GPS time to UTC: UTC = GPS − (TAI − UTC − 19 s) = GPS − 18 s while TAI − UTC is 37 s (`--tai-utc` for a future leap
second, IERS Bulletin C). The fix time tag is the receiver's, not the reception time.

## 4. Orbit Determination

### 4.1 Method and why

**Batch weighted least squares on the GNSS positions with a numerically integrated force model**, rather than fitting
SGP4 elements to the fixes:

1. The requirement is a forward-predicted ephemeris **with covariance** (OD-R2). Batch least squares gives the
   covariance of the estimate directly, and it is mapped forward with the state transition; SGP4 has no covariance.
2. SGP4's theory error (about 1 km over a day for LEO) is a hundred times the receiver's fix error (≈ 5 m per axis).
   Fitting SGP4 directly to the fixes would throw that accuracy away. The TLE is still produced (OD-R4), by fitting
   SGP4 to the predicted trajectory (§5.2), which is what SGP4 is for.
3. The fixes are few and irregular (one session per orbit in launch and early operations, four a day afterwards, and
   what the stations hear). A batch fit over the last 24–48 h uses them all at once and estimates the drag, which an
   arc this short needs; a sequential filter offers nothing here.

### 4.2 Frames

Fixes are ECEF (ITRF to centimetres). The orbit is integrated in TEME (true equator, mean equinox), linked to the
Earth-fixed frame by GMST (IAU-1982). EME2000 uses IAU-1976 precession and the IAU-1980 nutation (18 largest terms).
Checked against Vallado, *Fundamentals of Astrodynamics* (4th ed.), Example 3-15: TEME within 0.1 mm, J2000 within
0.1 m (`TestFrames`). The fit does not depend on DUT1 (the fixes go to TEME and the products back to ITRF with the same
rotation; the force model is unchanged by a rotation about the pole), so the ITRF product needs no Earth-orientation
data. The EME2000 product does (`--dut1`, IERS Bulletin A; refused without it: up to 450 m of error). Polar motion
(< 20 m) is optional.

### 4.3 Force model and integration

| Term | Model |
|------|-------|
| Gravity | Spherical harmonics in the Earth-fixed frame (V/W recursion, Montenbruck and Gill §3.2). Built in: EGM96 to 4 × 4. **Operations: EGM2008 to at least 12 × 12** from an ICGEM `.gfc` file (`--gravity-file`, `--gravity-degree`), kept on the orbit-determination machine |
| Drag | Piecewise-exponential atmosphere (Vallado Table 8-4) at the geodetic height, co-rotating; Cd A/m from BASELINE §3 (2.2 × 0.0247 m² / 1.6 kg = 0.034 m²/kg; `--area 0.0164` with the wings failed) times a **scale factor k, estimated**, a-priori 1.0 ± 1.0 |
| Sun, Moon, diurnal bulge | Available (`orbit_dynamics.py`), not used in the fit: tens of metres over a day (simulation) |
| Integrator | Adams-Bashforth-Moulton order 10 on a 60 s grid, Dormand-Prince start, Lagrange interpolation; 0.07 mm a day against a 1 s Runge-Kutta reference (`TestDynamics`) |

### 4.4 Estimation

| Item | Rule |
|------|------|
| Estimated | TEME position and velocity at the epoch, and k |
| Fix sigma | Per axis: 2.5 m × PDOP (piNAV-NG: 10 m 2σ horizontal), combined with a **force-model error floor**: 150 m (4 × 4 field), 100 m (to 8 × 8), 60 m (to 12 × 12), 30 m (finer) (`--model-sigma`). The gravity left out is correlated along the orbit, not white: without the floor the covariance is too small between and after sparse fixes (§6) |
| Velocity | The frame's chord velocity is fitted, per axis with √(2 × (2.5 m × PDOP)² / VDT² + (0.01 m/s)² + (floor × n)²): its noise, the onboard method error (8 mm/s over 120 s), and the force-model floor times the mean motion. It is derived from the receiver's positions, so the D6 concern about the receiver's own velocity output does not arise. `--no-velocity` leaves it out of the fit |
| Start | A fix's velocity; else two fixes of one session 2–600 s apart; else a prior TLE (`--prior-tle`, e.g. the launch provider's predicted elements: needed when the sessions had no usable chord, `VDT=-`). Otherwise the fit is refused |
| Minimum | 9 measured values from at least 2 fixes: three position-only fixes, or two fixes with velocity |
| Iteration | Levenberg-Marquardt; partials from finite-difference trajectories integrated with the nominal; without a measured velocity the arc is grown from the start outwards |
| Editing | After convergence, a robust stage (Huber weights at 3σ per fix, until they settle) keeps one gross error from dragging a fit of few fixes; then a fix whose residual exceeds 5 × the robust residual scale is rejected and the rest refitted at full weight (three rounds at most). Dead-reckoned (`Q` ≠ 1, 2), < 4 satellites or implausible radius refused before the fit |
| Covariance | Formal, scaled by the reduced chi-square when above 1, mapped forward with the state transition, plus a **drag forecast term**: a 20 % change of k from the last fix on (`--drag-forecast`), the density change after the fit that no fit can know (D6 research §6.2) |
| Window | The last 48 h of fixes (`--window-h`) |

### 4.5 Quality flag

Decided on the 3D position 1σ at the last fix + 24 h (the time to the next update, OD-R1) and the RMS residual:

| Flag | Rule | Use |
|------|------|-----|
| **GOOD** | Converged; 1σ(+24 h) ≤ 5 km; arc ≥ one orbit (90 min); RMS ≤ 50 m per axis; ≤ 20 % of fixes rejected | Upload |
| **FAIR** | Converged; 1σ(+24 h) ≤ 20 km; RMS ≤ 200 m; ≤ 20 % rejected; below GOOD on one count (named in the report) | Upload; next update at the next pass |
| **POOR** | Anything worse, or not converged | Written as `NOT-FOR-UPLOAD_…`; exit 2; not uploaded (§7) |
| **FAIL** | Fewer than the minimum (§4.4), no start, singular normal matrix | No products; exit 3 |

## 5. Products

### 5.1 CCSDS OEM

`<object>_<last fix>_<frame>.oem`, KVN, `CCSDS_OEM_VERS = 3.0`, `TIME_SYSTEM = UTC`, `CENTER_NAME = EARTH`,
`REF_FRAME = ITRF2000` (default; the receiver's WGS-84 agrees with it to centimetres) or `EME2000`, `OBJECT_ID` the
international designator once assigned (`--object-id`), `ORIGINATOR = KK6GDB`. States every 60 s from the first step
after the last fix over 72 h (`--span-h`, 3–7 days), km and km/s; `INTERPOLATION = LAGRANGE`, degree 7; a 6 × 6
covariance block per record (`--cov-every` to thin), lower triangle, km², km²/s, km²/s², in the same frame. The
`COMMENT` lines give the quality flag, the fit (fixes, sessions, arc, RMS, k), the force model and the 1σ growth.
The tool reads the file back (`read_oem()`) and checks its keywords and records before reporting it.

### 5.2 TLE

`<object>_<last fix>.tle`: SGP4 (WGS-72) mean elements and B* fitted by least squares to the predicted TEME positions
every 10 minutes over the span (python-sgp4, the reference implementation), epoch at the last fix. The catalogue number
is 99999 (`--norad-id`) until the 18th SDS assigns one; ṅ/2 and n̈/6 are written as zero (SGP4 does not use them). The
report gives the SGP4 fit RMS (≈ 0.7 km over 72 h in simulation). The TLE is for the integrator, the 18th SDS and
amateur tracking; the OEM is the ephemeris of record.

### 5.3 Report

`<object>_<last fix>_report.json`: the flag and its reasons, fixes used and rejected (with reasons), the residual of
each fix, RMS, reduced chi-square, k ± σ, the force model and error floor, the 1σ at the start, at +24 h and at the
end, the TLE fit RMS and the file names.

## 6. Verification and Accuracy

`test_orbit_determination.py` flies a simulated SMART-QSO: 475 km sun-synchronous (97.31°), LTAN 22:30
(BASELINE §3), from 2028-11-02, integrated with **more than the fit models**: gravity to 20 × 20 (EGM96 to 4 × 4,
then Kaula-rule coefficients standing in for the real higher-degree field), Sun and Moon, a ±30 % diurnal density
bulge, and drag 1.4 times the nominal. Each session's fix gets 5 m per axis noise (2.5 m × PDOP 2) and the 30 s chord
velocity its matching noise; it is written as the flight software's GNSS state frame, decoded through the archive
reader, and fitted by the command-line tool; the OEM is read back and compared with the truth.

| Case (one frame per session) | Field in the fit | Flag | 3D error / 1σ at last fix + 24 h | + 48 h | + 72 h | Largest error / 1σ |
|------------------------------|------------------|------|------------------------------|--------|--------|-----------------|
| Launch and early operations: a session every orbit for 24 h (16 frames) | 12 × 12 file | GOOD | 0.60 / 2.8 km | 1.5 / 11.0 km | 3.6 / 24.7 km | 3.1 |
| Same | built-in 4 × 4 | FAIR (RMS 117 m) | 1.8 / 3.1 km | 6.1 / 12.2 km | 11.5 / 27.5 km | 2.9 |
| **First-ephemeris deadline, own station only**: `GNSS_REQUEST` on 4 passes in two pass groups 12 h apart (4 frames) | 4 × 4 | FAIR | 2.0 / 6.7 km | 6.7 / 23.0 km | 15.7 / 48.9 km | 1.3 |
| Same | 12 × 12 file | GOOD | 2.6 / 4.3 km | 8.7 / 15.6 km | 18.5 / 34.3 km | 1.5 |
| Two passes one orbit apart (2 frames) | 4 × 4 | FAIR (drag not observable: k 1.0 ± 1.0) | 5.4 / 11.3 km | 18.7 / 42.6 km | 39 / 95 km | 0.6 |
| Two passes 12 h apart (2 frames) | 4 × 4 | FAIR (same) | 7.9 / 15.2 km | 23 / 51 km | 46 / 107 km | 1.4 |
| Two sessions in one pass, 4 minutes apart (2 frames) | 4 × 4 | POOR | 34 / 60 km | 62 / 126 km | 80 / 202 km | 0.8 |
| One frame | | FAIL (6 values) | | | | |

**Stated accuracy** (asserted by the tests, with margin): launch and early operations with the 12 × 12 field,
**≤ 1.5 km at the last fix + 24 h and ≤ 8 km at + 72 h**, GOOD; the deadline case, **≤ 5 km at + 24 h**, GOOD or FAIR;
in every case the covariance is realistic (error within 3σ at 99 % of the epochs and within 4σ at all); two sessions
in one pass are POOR and not uploaded; one frame, or sessions without a chord and no prior, are FAIL; with a prior
TLE the position-only case fits. The TLE agrees with the truth within 3 km at + 24 h. The tests also check the OEM
keywords, 4,321 records over 72 h at 60 s, positive-definite covariances, the TLE format and checksums against
python-sgp4, the refusal of a dead-reckoned record and the editing of an outlier among four fixes, the same frame
heard by two stations and repeated on `GET_HOUSEKEEPING`, and the EME2000 refusal without DUT1.

**What dominates.** Beyond a day, the drag forecast (20 % of k) dominates the error and the covariance, as the D6
research expected. Within the arc, gravity left out of the field does: the built-in 4 × 4 field leaves 100 m-class
residuals, hence the 12 × 12 file in operations. The receiver's noise is negligible.

**Limits.** The simulation is not the real atmosphere: a geomagnetic storm in the prediction span can exceed the 20 %
term, which is why the ephemeris is refitted after every pass (OD-R1). The drag area changes with attitude (the spin
about +Y with the wings deployed; tumbling before detumble); k absorbs the mean. The real receiver's noise and its PDOP reporting are confirmed on the flatsat
(VI-LV-13).

## 7. First-Ephemeris Procedure (24 h from First Contact)

Clock: **T0 = first contact** (the first beacon decoded by any station after the RF silence; logged in the pass
report). Deadline T0 + 24 h (UG §9.5.7.2). Roles: ground segment lead (orbit determination and upload), licensee
(approval, Space-Track operator contact), station control operators (passes).

| When | Step | Who |
|------|------|-----|
| Before launch | Space-Track operator account and SpaceX Space Traffic Coordination upload access tested with a simulated OEM (VI-LV-13); EGM2008 `.gfc` file and the tool installed on the orbit-determination machine; the launch provider's predicted elements saved as the prior TLE; the international designator, when published, set as `--object-id` | Ground segment lead |
| T0 | Start the clock. Log the pass. Archive every frame (own station, SatNOGS, reports) to the mission database; the frames sent after each fix of the every-orbit sessions are heard by SatNOGS stations as well | Control operator |
| Each own pass from T0 | After the first authenticated command (SQSO-GSOM-001 §7.2), send `GNSS_REQUEST` (0x60) early in the pass; the session (cold start ≈ 80 s, 30 s after the fix) ends inside a 7-minute pass and its frame is downlinked. `GET_HOUSEKEEPING` (0x09) repeats the latest frame if it was missed | Control operator |
| After each pass | Export the GNSS frames of the last 48 h; run `ephemeris_tool.py` (SQSO-GSOM-001 §5.3); read the flag and the report | Ground segment lead |
| First GOOD or FAIR product | Check with `read_oem()` (the tool does it) and by eye: radius ≈ 6,850 km, period ≈ 94 min, epoch after the last fix; licensee approves; upload the OEM to Space-Track and to SpaceX Space Traffic Coordination; send the TLE to the integrator and the 18th SDS; log the upload | Ground segment lead; licensee approves |
| T0 + 12 h with no GOOD or FAIR product | Ask the second station and SatNOGS for priority on the next passes; `GNSS_REQUEST` on every commanding pass; check the receiver telemetry (`GF`, `GS` in HK1) | Ground segment lead |
| T0 + 18 h with no GOOD or FAIR product | **Contingency**: invoke the third-party orbit-determination service contracted before integration (UG §9.5.7.2; LVR-089), and tell the integrator and SpaceX | Licensee |
| Then | A new product after every pass and at least every 24 h until the object is catalogued; afterwards three a day where the passes allow (UG §9.5.7.5), at least daily | Ground segment lead |

A POOR product is never uploaded. With only two fixes in one pass group the fit is FAIR at best and its covariance
large (§6): it is uploaded if nothing better exists before T0 + 24 h, with the next update at the next pass.

## 8. Open Items

| Item | Closure |
|------|---------|
| The GNSS state frame's entry in SQSO-TLM-001; the decoder already follows `gnss_frame.rs` | Flight software round 3 |
| End-to-end demonstration: receiver fix on the flatsat, downlink, this tool, OEM accepted in writing by the integrator and SpaceX | VI-LV-13, ≤ L-2 M |
| Space-Track file naming and upload interface (Spaceflight Safety Handbook for Operators) | Ground segment lead, with VI-LV-13 |
| The third-party orbit-determination contingency contracted | Licensee, before integration |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-10-27 | Initial release (RID LV-04): GNSS state frame decoding (`gnss_frame.rs` layout), batch least-squares orbit determination with an estimated drag scale, CCSDS OEM with covariance, TLE, quality flag, simulated verification at 475 km SSO, the 24 h first-ephemeris procedure | CDR round 3, ground software |
