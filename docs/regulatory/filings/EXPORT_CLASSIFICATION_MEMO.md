# Export Control Classification Memorandum
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-012
**Revision:** A
**Date:** 2026-10-14
**Status:** Self-classification draft. **Must be confirmed by export counsel or a BIS commodity classification (SNAP-R) before any export, foreign launch, or foreign-integrator engagement.**
**Supersedes:** the system-level and Jetson classifications in `docs/compliance/EXPORT_CONTROL.md` (SQSO-COMP-004 Rev A)

---

## 1. Why the Prior Assessment Is Withdrawn

SQSO-COMP-004 Rev A classified the spacecraft, the Jetson module and all components as **EAR99**. Two of those
conclusions conflict with the current regulations:

| Item | Prior | Corrected (presumptive) | Basis |
|------|-------|-------------------------|-------|
| SMART-QSO spacecraft (integrated) | EAR99 | **ECCN 9A515.a.5** | 15 CFR 774 Supp. 1, ECCN 9A515.a: "'Spacecraft,' including satellites … whether designated developmental, experimental, research or scientific, not described in USML Category XV or … 9A004 … that … a.5. Are not described in ECCN 9A515.a.1, .a.2, .a.3 or .a.4." SMART-QSO has no remote-sensing capability (a.1–a.3) and does no servicing (a.4), so it falls under a.5. EAR99 is only for items not on the CCL. |
| NVIDIA Jetson Orin Nano module | EAR99 | **ECCN 5A992.c** (mass-market encryption item) | Distributor product data (e.g., Farnell parts 900-13767-0030-000 / -0040-000 list US ECCN 5A992.c). Confirm against NVIDIA's own export statement. |

## 2. Classification Table (presumptive)

| Item | Jurisdiction / ECCN | Reasons for control | Practical effect |
|------|---------------------|---------------------|------------------|
| Integrated spacecraft | EAR, **9A515.a.5** | NS1, RS1, AT1 | License required for export to every destination except Canada. License Exception STA paragraph (c)(1) is available for 9A515.a.5 to Country Group A:5 (the 9A515 STA special conditions exclude only a.1–a.4). LVS ($1,500) does not apply at spacecraft value. |
| Jetson Orin Nano module | EAR, 5A992.c | AT | No license required for most destinations; prohibited for embargoed/sanctioned end users. |
| Other COTS parts (MCU, EPS, radios, solar cells, structure) | Classify per part; most are EAR99 or 5A992/3A991 | — | Collect manufacturer ECCNs in the parts list (`docs/hardware/PARTS_LIST.md`) before CDR re-baseline. |
| Spacecraft "technology" and "software" (9E515 / 9D515) | EAR, *unless published* | NS1, RS1 | Information that is **published** (15 CFR 734.3(b)(3), 734.7), as in this open repository, is not subject to the EAR. **Unpublished** integration or operations data shared with foreign persons is a (deemed) export. |
| **Launch-integration services and launch-failure assistance** | **ITAR** | — | 15 CFR 774 Supp. 1, 9A515 "Related Controls": "all services (including training) directly related to the integration of any satellite or spacecraft to a launch vehicle, including both planning and onsite support, or furnishing any assistance … in the launch failure analysis or investigation for items in ECCN 9A515.a, are 'subject to the ITAR.'" |

## 3. Consequences for Launch Selection

1. **U.S. launch site with a U.S. integrator/dispenser**: no export of the spacecraft. Integration services stay among
   U.S. persons, so the lowest export burden.
2. **U.S. launch site with a foreign-owned dispenser provider or foreign personnel**: the spacecraft is not exported, but
   integration support to foreign persons may be an ITAR defense service. Obtain export-counsel guidance; a Technical
   Assistance Agreement (TAA) may be required. Budget 3–6 months.
3. **Foreign launch site** (e.g., New Zealand, India, Europe): the spacecraft export needs a BIS license or STA (c)(1)
   (A:5 destinations only), **plus** an ITAR authorization for integration services. Budget 6+ months.

These consequences feed the [launch trade](../../funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md).

## 4. Program Controls

| # | Control | Owner |
|---|---------|-------|
| E1 | Keep all design data public in this repository (the published-information exclusion). Do not create private technical annexes without an export review. | PI |
| E2 | Screen every foreign-person team member and contractor against non-public technical data (deemed exports). | PI |
| E3 | Screen all parties against the BIS Entity List, OFAC SDN and the Consolidated Screening List before any transfer. | PI |
| E4 | Obtain a formal classification (counsel opinion or BIS CCATS) before signing a launch agreement. | PI |
| E5 | Record ECCNs for every procured part in the parts list. | Hardware lead |

## 5. Sources (retrieved 2026-10-14)

- 15 CFR Part 774, Supplement No. 1, ECCN 9A515 (eCFR, current as of 2026-10-13)
- 15 CFR 734.3(b)(3), 734.7 (published information), 15 CFR 740.20 (License Exception STA)
- Farnell product pages for NVIDIA 900-13767-0030-000 and 900-13767-0040-000 (ECCN field)

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial memo (funding audit); supersedes SQSO-COMP-004 classifications |
