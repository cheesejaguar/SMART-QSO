# Launch Provider Deliverables Package
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-016
**Revision:** A
**Date:** 2026-10-14
**Status:** Checklist. Every item is **open**; no hardware exists yet.
**Applies to:** whichever launch path is selected in the [launch trade](../../funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md). Provider-specific ICDs (e.g., NRCSD IDD, EXOpod/QuadPack/Maxwell user guides, NASA LSP-REQ-317.01 for CSLI) **supersede** this generic list where stricter.

---

## 1. CubeSat Design Specification Rev 14.1 — items the provider will verify

Source: Cal Poly *CubeSat Design Specification (1U–12U) Rev 14.1*, CP-CDS-R14.1 (Feb 2022). Requirement numbers are
CDS numbers.

| CDS ref | Requirement (abridged) | SMART-QSO evidence to deliver | Status |
|---------|------------------------|-------------------------------|--------|
| 2.1.1 | All parts remain attached during launch, ejection and operation | Design description; post-vibe inspection report | Open |
| 2.1.2 / 2.1.6 | Pyrotechnics and hazardous materials conform to AFSPCMAN 91-710 Vol. 3 | Statement: no pyrotechnics. Hazardous materials list (Li-ion cells) | Open |
| 2.1.5 (note) | Consider FAA airline battery guidance (≤ 100 Wh per battery) | Battery Wh at the BASELINE value (well under 100 Wh) | Open |
| 2.1.7 | Materials TML ≤ 1.0 %, CVCM ≤ 0.1 % (ASTM E595) | Materials list with outgassing data (`docs/compliance/MATERIALS_LIST.md`, to be updated with sources) | Open |
| 2.1.8 | Passive magnets ≤ 0.5 G above Earth's field outside the static envelope | Statement / measurement (magnetorquers are not passive magnets; check any magnetic latches) | Open |
| 2.1.9 | Ascent venting | Vent-path description | Open |
| 2.2.x | Envelope, rails (≥ 75 % rail contact, 2.2.9), mass (1U typical max **2.00 kg**, Table 1; **the dispenser's own limit governs**) | CubeSat Acceptance Checklist (CAC) measurements; mass-properties report (mass, CG, MOI) | Open |
| 2.3.4 / 2.3.5 | RBF pin cuts all power; ≤ 6.5 mm protrusion; within access port if one exists | Electrical schematic; inspection | Open |
| 2.3.6 | Battery charge/discharge protection; extra documentation for modified or non-UL cells | Battery data package (§2) | Open |
| 2.3.7 | **≥ 3 independent RF inhibits** (a timer is not an inhibit) | Inhibit diagram showing three physical inhibits between battery and RF transmitters | Open. The repository currently describes two deployment switches plus a FET under timer logic; confirm three *physical* inhibits |
| 2.3.8 | **≥ 3 independent inhibits** to deployables (antenna, solar panels) | Same diagram for burn-wire circuits | Open |
| 2.4.1 | Documentation of radio licenses; for amateur frequencies, **proof of IARU coordination** (2.4.1.1) | IARU coordination letter + FCC pre-space notification filing record | Open |
| 2.4.3 | Comply with NPR 8715.6; any component re-enters with < 15 J (2.4.3.1) | ODAR with DAS output | Open |
| 2.4.4 | Deployables wait **≥ 30 min** after deployment-switch activation | Flight software parameter + test record | Open. Software and documents must agree (see findings register) |
| 2.4.5 | **No signal generated or transmitted earlier than 45 min** after on-orbit deployment | Flight software parameter + test record | Open |
| 2.4.6 (note) | NOAA remote sensing license if applicable | [NOAA determination](NOAA_REMOTE_SENSING_DETERMINATION.md) | Open |
| 2.4.7 | Fit check(s) with the dispenser or an engineering dispenser | Fit-check report | Open |

## 2. Battery Data Package

Providers, and for ISS deployment NASA JSC-20793, require more than CDS 2.3.6. Prepare:

1. Cell manufacturer, part number, chemistry, capacity and a UL/IEC listing certificate
2. Pack configuration (series/parallel), nominal Wh, protection circuitry schematic (over-charge, over-discharge, over-current, short-circuit, thermal)
3. Cell screening/lot acceptance records (OCV, capacity, internal resistance, visual inspection)
4. Pack acceptance test records (charge/discharge cycles, vibration, thermal, vacuum leak/mass loss where required)
5. Charging procedure and state-of-charge limits for storage, integration and launch
6. For ISS deployment only: JSC-20793 compliance evidence. This is a substantial, schedule-driving activity; see the launch trade

## 3. Regulatory Evidence (provider and FAA payload review)

| Item | Why | Source in repo |
|------|-----|----------------|
| IARU coordination letter | CDS 2.4.1.1 | (after F1 in the roadmap) |
| FCC pre-space notification as filed, plus FCC acknowledgment if issued | FAA favorable payload determination needs "all required licenses, authorizations, and permits" (14 CFR 450.43(a)(1)) | [FCC pre-space draft](FCC_97-207_PRE_SPACE_NOTIFICATION.md) |
| ODAR / debris disclosures | CDS 2.4.3; FCC 97.207(g)(1) | [ODAR](ODAR_SMART-QSO.md) |
| NOAA determination | CDS 2.4.6 | [NOAA determination](NOAA_REMOTE_SENSING_DETERMINATION.md) |
| Export classification statement (and license/TAA if a foreign integrator or site is used) | Provider contract terms; 15 CFR 774 9A515 related controls | [Export memo](EXPORT_CLASSIFICATION_MEMO.md) |
| Operator contact form (24 h), beacon frequency, call sign | 18 SDS pre-launch data; identification | [Space safety plan](SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md) |

## 4. Test and Analysis Reports

| Report | Typical requirement | Repo procedure |
|--------|---------------------|----------------|
| Random vibration (protoflight or acceptance per provider levels) with pre/post functional tests and sine signature | Provider ICD / user guide levels (GEVS GSFC-STD-7000B as fallback) | `docs/test_procedures/TP-ENV-002_Vibration.md` (levels to be re-baselined to the chosen provider) |
| Thermal vacuum / bakeout (if required by provider) | Provider ICD | `docs/test_procedures/TP-ENV-003_Thermal_Vacuum.md` |
| Shock (analysis or test) | Provider ICD | — |
| Deployment-switch, RBF and inhibit verification | CDS 2.3.5–2.3.8 | `docs/test_procedures/TP-MECH-003_Deployment_Switches.md` |
| 30-min deployable / 45-min RF timer verification | CDS 2.4.4–2.4.5 | `docs/test_procedures/TP-ENV-005_Deployment_Post_Env.md` |
| Mass properties | CDS Table 1 / dispenser limit | — (create) |
| Fit check | CDS 2.4.7 | `docs/test_procedures/TP-MECH-001_Fit_Check.md` |
| Structural analysis summary (margins, first mode vs. provider requirement) | Provider ICD | `docs/environment/` analyses (re-baseline to provider loads) |

## 5. Delivery Logistics

- Pack-out procedure with ESD and battery state of charge per provider; shipping container; humidity indicators
- Pre-ship review sign-off (`docs/integration/PRE_SHIP_CHECKLIST.md`)
- Charging and access plan matched to the dispenser's access-port availability (CDS 2.3.4.1)

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial checklist (funding audit) |
