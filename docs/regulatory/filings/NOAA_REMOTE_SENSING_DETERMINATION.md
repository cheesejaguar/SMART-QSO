# NOAA Commercial Remote Sensing — License Applicability Determination
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-011
**Revision:** A
**Date:** 2026-10-14
**Status:** Internal determination. Written NOAA confirmation is recommended (§5).

---

## 1. Regulatory Basis

- The Land Remote Sensing Policy Act of 1992, as amended (51 U.S.C. 60101 et seq.), implemented by 15 CFR Part 960, requires a license to operate a *private remote sensing space system*.
- 15 CFR 960.4 (eCFR, current as of 2026-10-13) defines:
  - **Private remote sensing space system**: "an instrument that is capable of conducting remote sensing and which is not owned by an agency or instrumentality of the U.S. Government."
  - **Remote sensing**: "the collection of unenhanced data by an instrument in orbit of the Earth which can be processed into imagery of surface features of the Earth."
- 15 CFR 960.5(a) lets a person "consult informally with the Secretary … including whether a license is likely to be required for a system."

## 2. SMART-QSO Sensor Inventory

| Sensor | Purpose | Can its data be processed into imagery of Earth surface features? |
|--------|---------|------------------------------------------------------------------|
| Coarse sun sensors (photodiodes) | Attitude determination | No: single-pixel irradiance, no spatial resolution |
| 3-axis magnetometer | Attitude determination / B-dot | No: magnetic field vector |
| Gyroscope / IMU (if fitted) | Rate sensing | No |
| Thermistors, voltage/current monitors | Housekeeping | No |
| RF receivers (transponder, telecommand) | Communications | No |
| Jetson Orin Nano MIPI-CSI camera interface | **Not populated** (`hardware/payload-jetson/INTEGRATION_DESIGN.md`, connector table) | N/A: no imaging sensor installed |

## 3. Determination

SMART-QSO carries **no instrument capable of remote sensing** as defined in 15 CFR 960.4. A NOAA commercial remote
sensing license is therefore **not required**.

## 4. Configuration Control Condition

The determination holds only while the spacecraft has no imaging sensor. The following are controlled changes that
require this determination to be redone (and probably a NOAA license application under 15 CFR 960 Appendix A) before
the change is approved:

1. Populating the Jetson CSI/USB camera interface, or adding any star tracker or Earth sensor that has a focal-plane array
2. Adding any payload whose data could be processed into Earth-surface imagery

Owner: Systems Engineering (Configuration Control Board).

## 5. Recommended Action

| # | Action | When | Status |
|---|--------|------|--------|
| 1 | Send NOAA CRSRA an informal consultation (15 CFR 960.5(a)), e.g. the CRSRA Initial Contact Form or email to crsra@noaa.gov, enclosing §2 and asking for written confirmation that no license is required | ≥ 12 months before launch | Not started |
| 2 | File NOAA's written response in `docs/regulatory/filings/` and cite it in the launch-provider regulatory package | On receipt | Not started |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial determination (funding audit) |
