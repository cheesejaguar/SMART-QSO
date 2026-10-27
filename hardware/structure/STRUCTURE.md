# CubeSat Structure Frame Trade Study for SMART-QSO 1U Mission

**Version**: B
**Date**: 2026-10-27
**Status**: Superseded by `docs/hardware/BUILD_VS_BUY.md` §2.2 (the buy decision and the vendor survey) and
`hardware/structure/STRUCTURAL_DESIGN.md` v1.3 §2.1 (the frame requirements for the RFQ). Kept as a record of what the
earlier study got wrong.

## 1. Decision

Buy the 1U frame. Send RFQs to ISIS, Spacemanic and Pumpkin and freeze the choice before the mechanical layout
(BUILD_VS_BUY §2.2). The frame must:

| Requirement | Why | Source |
|-------------|-----|--------|
| Rails 100.0 ± 0.1 mm, 113.5 ± 0.1 mm long, ≥ 8.5 mm to the first protrusion, Type III hard anodize, Ra < 1.6 µm, edges ≥ 1 mm | Dispenser interface | CDS Rev 14.1 §2.2.1–§2.2.13; EXOpod Nova Rev 1.2 §2.3 |
| A −Z face recessed for the 7 mm ISIS AntS below the rail ends | The antenna is bought (BUILD_VS_BUY §2.6) | CDS Appendix B standoff |
| Three deployment-switch positions and two separation-spring plungers in the rail standoffs | Three physical inhibits (BASELINE §4); shared slot (CDS §2.2.14) | STRUCTURAL_DESIGN §5.2–§5.3 |
| Inserts for the battery isolators, the wing hinges and the heat spreader | Isolated battery, wings, Jetson heat path | STRUCTURAL_DESIGN §2.1 |
| Vendor qualification report, mass and CG | Verification by vendor data for a bought frame | STRUCTURAL_ANALYSIS (SQSO-SA-001) §15, VI-STR-11 |

| Candidate | Mass (vendor) | Status (October 2026) |
|-----------|---------------|------------------------|
| ISIS 1U structure | 87 g primary, 108 g with secondary | Page removed from isispace.nl; recessed ±Z faces for the AntS |
| Spacemanic 1U structure | 115 g | Quote |
| Pumpkin CubeSat Kit 1U | Not stated | Store shows "Unavailable"; AntS-compatible plates listed |
| EnduroSat 1U | 120 g | Delisted |
| GomSpace 1U | — | No longer sold |

## 2. Corrections to Version A

Version A (undated, before the funding audit) recommended a Pumpkin structure on the basis of values that do not match
any vendor data:

| Version A statement | Correct value | Source |
|---------------------|---------------|--------|
| Corner rails "12.7 mm × 12.7 mm" | CDS rails are ≥ 8.5 mm wide (EXOpod accepts 6.4 mm) | CDS Rev 14.1 §2.2.5 |
| Frame masses 260–300 g | 87–120 g | BUILD_VS_BUY §2.2 |
| "Load capacity" 8–12 kg, wall thickness, "thermal treatment", "> 100 launches" for each vendor | Not published by the vendors | — |
| GomSpace and EnduroSat 1U frames available | Delisted or no longer sold | BUILD_VS_BUY §2.1 |
| Launch on the NanoRacks NRCSD | Primary path is a Transporter-class SSO rideshare in an EXOpod; NRCSD is the backup | BASELINE §3 (D7) |
| Jetson heat load 5–15 W | ≤ 8 W input in bursts of ≤ 120 s, sunlit only | BASELINE §6 |
| "4-40 threaded inserts on all faces" | Metric (M2, M2.5, M3) per STRUCTURAL_DESIGN §8 | — |
| Black-anodized "optimized thermal design for AI payloads" | Not a vendor claim; the heat path is a project bracket (≥ 1 W/K, BASELINE §6) | BUILD_VS_BUY §2.2 |
| Costs $2,200–3,200 | ISIS ≈ €2,150 [unverified]; Pumpkin ≈ $2.4–3k [inferred]; others by quote | BUILD_VS_BUY §2.2 |

## 3. Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | — | Trade study recommending a Pumpkin 1U structure (values unsourced) |
| B | 2026-10-27 | CDR structures review (STR-23): superseded by the build-versus-buy decision; frame requirements listed; Version A's wrong rail size, masses, vendor status and launch path corrected |
