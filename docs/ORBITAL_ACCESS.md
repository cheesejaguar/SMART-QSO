# Orbital Access Plan

**Document ID**: SQSO-OAP-001
**Version**: B
**Date**: 2026-10-27
**Status**: Plan for CDR. No launch contract, quote or manifest exists. Prices and dates are planning values.
**Governing values**: `docs/BASELINE_PARAMETERS.md` §3 (decision D7). Requirement flow-down:
`docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001). Research: `docs/funding_audit/research/D7_LAUNCH_INTEGRATORS.md`.

---

## 1. Recommendation

**Fly on a SpaceX Falcon 9 Transporter-class dedicated SSO rideshare from Vandenberg (SLC-4E), in a 1U slot of an
Exolaunch EXOpod Nova bought from Exolaunch, with a written drop-off altitude of 500 km and a 525 km hard cap.**
Sign by 2027-06-30. Keep a US-owned price check (SEOPS) and ISISPACE in the same request for quotes. Hold the ISS
(Voyager NRCSD) route as the backup and Electron from New Zealand as the altitude-controlled contingency; neither is
worked on until a trigger in §5 fires, except for the design choices of §3.3 that cost little now and keep the
backup open.

Why: it is the only route that reaches the baseline orbit (500 km SSO, LTAN about 22:30) in 2028 with flight-proven
1U accommodation, and the one the whole design (power, thermal, lifetime, the simulation) is built for. The
alternatives each lose something the mission needs: the ISS orbit loses mission-success Level 3 and changes the
power and thermal cases; Electron has no published 1U route or price and needs an export licence for a foreign
launch site; CSLI cannot launch before about 2030.

## 2. Primary path

### 2.1 The parts

| Item | Selection | Evidence | Open |
|------|-----------|----------|------|
| Launch vehicle | Falcon 9, Transporter-class **dedicated** rideshare (the UG's Transporter-specific EMI limits, Table 4-9, and transmitter turn-on delays, Table 4-10, apply to these missions) | SpaceX UG v11 (Aug 2026) §1.2, §4.1.7.2–§4.1.7.3 | The flight. SpaceX's portal lists only NET 2027-06-01 and 2027-10-01 (SSO, 500–600 km); a 2028 seat comes from an integrator's allocation (D7 research §1) |
| Launch site | Vandenberg SFB, SLC-4E (SSO). The UG covers CCSFS, KSC and VSFB (§8, §10) | Transporter-17 flew from SLC-4E on 2026-07-07 (D7 research ref. 12) | Confirmed by the flight assignment |
| Orbit | Circular SSO, 97.4°, target 500 km, cap 525 km; LTAN ≈ 22:30 (descending node 10:30) as in the simulation | BASELINE §3; UG Appendix G (orbit chosen at L-5 M from SpaceX's list) | The altitude list of the assigned flight |
| Integrator | **Exolaunch** (Berlin; EQT acquisition signed 2026-06-18, closing unverified). Price check: ISISPACE (QuadPack). US-owned alternative: SEOPS (1U support unverified) | D7 research §3.2–§3.8 | Quotes |
| Dispenser | **EXOpod Nova** (8U/12U/16U deployers; the smallest slot is 3U). A 1U flies in a 3U slot with a non-deployable slot adapter, or sharing the slot with other 1U/2U spacecraft (NOVA §2.2, §3.4; EXO Table 7 note). 1U limits: 2.5 kg, COG ±20 mm, protrusions up to 25 mm from the rail (NOVA Tables 1–2). Door released by two redundant magnetic locks; reed switches report door and wagon to the LV; access windows on three sides; no standard umbilical (NOVA §4.2.4, §4.2.5, §5.4). The Nova hardware is advertised as ITAR-free | EXOpod Nova UM Rev 1.2 (June 2024); EXOpod UM Rev 6.0 (2022) | Which slot type (adapter or shared); the Nova's 1U ejection velocity; the integration guide |
| Price | Integrator 1U slot $85k–150k; all-in $115k–250k; central $150–170k; +15 % contingency on the slot | BASELINE §3; D7 research §6.1 | Written quotes |

### 2.2 Contract chain: who signs what, who delivers what to whom

```
SpaceX  --(Rideshare Launch Services Agreement, SpaceX ICD, SOW; UG v11 applicable)-->  Exolaunch
Exolaunch  --(Exolaunch launch services agreement, mission ICD; flows the UG down)-->  SMART-QSO contracting party
SMART-QSO contracting party  --(FCC Part 97 licence: KK6GDB / KK6GDB-11)-->  FCC, IARU, 18th SDS, NOAA
```

- **SpaceX's customer is Exolaunch**, which holds the plate allocation, signs the SpaceX ICD for the whole Integrated
  CubeSat Dispenser Assembly (UG §1.2, §2.6.3, §9.4.1), and is responsible for enforcing SpaceX's space-safety rules
  on its customers (UG §9.5.7.2). SpaceX does not deal with SMART-QSO directly.
- **SMART-QSO's counterparty is Exolaunch.** The deliverables of `LAUNCH_VEHICLE_REQUIREMENTS.md` §2 and §9 go to
  Exolaunch, which aggregates them into its own SpaceX submissions. Items that are personal to the operator go to the
  agency named: the 18th SDS registration and the ephemeris (Space-Track and SpaceX Space Traffic Coordination), the
  FCC notifications, and the FAA cross-waivers (signed by every customer, collected by the integrator).
- **The contracting party is not yet named.** BASELINE §1 lists no host organization. A launch contract with a
  foreign company, the FAA cross-waivers, the insurance declarations and the export compliance agreement need a legal
  entity; the licensee as an individual would carry the liability personally (RID LV-09).
- **What the contract must say** (D7 research §6.1): the 525 km cap and no-penalty rebooking if the L-5 M orbit list
  has nothing at or below it; written 1U price and payment milestones; slot type and 1U envelope (113.5 mm, stowed
  wings, RBF and switch positions against the access windows); acceptance of unit-level (containerized) testing in a
  TestPod at the UG v11 levels; integration date and where; the state of charge accepted at integration (60 %, not
  "full"); whether a duty-cycled GNSS receiver meets UG §9.5.7.2 or which third party Exolaunch recommends; the ITAR
  defense-services position with US-person integrators; demanifest, storage and rebooking terms; terms that survive
  a change of control.

### 2.3 Flow of the flight model

1. Protoflight environmental campaign on the flight unit in a Nova TestPod (`LAUNCH_VEHICLE_REQUIREMENTS.md` §8), by
   L-4 M.
2. Ship to the integrator's integration site (the mission ICD names it; for a Transporter flight usually the launch
   site); arrive about L-50 d. Ship by ground, or by air at the state of charge the carrier's rules allow, and charge
   at the integrator.
3. At the integrator: inspection and rail measurement, final charge to 60 %, key load and `ARM_FLIGHT` through the
   umbilical, umbilical removed, RBF removed, integration into the Nova slot (the SIL's integration date, L-47 d).
   No access afterwards.
4. Exolaunch delivers the loaded dispenser to SpaceX (≤ 8 weeks before launch, UG §9.5.1); PPF processing
   (≤ 45 days), encapsulation, transport to the hangar (≤ 40 days), mate at about L-5 d, rollout, launch (UG §4.3,
   §10.1).

## 3. Backup: ISS deployment from the Voyager NRCSD

### 3.1 What changes

| Area | Change | Source |
|------|--------|--------|
| Provider | Voyager Technologies (ex-Nanoracks), US-owned: the lowest export burden | D7 research §3.5 |
| Orbit | 400–420 km, 51.6°, deployed 1–3 months after berthing | IDD §3.4.3; BASELINE §3 |
| Lifetime | 0.7–1.9 yr nominal: **mission success Level 3 (2+ years) not achievable**, Level 2 (1 year) at risk | BASELINE §3 |
| Power and thermal | β up to about ±75° instead of 17–24° (`leop_passes.py`): long full-sun seasons and deep eclipse seasons; the SSO power closure and thermal cases (BASELINE §5, §8) must be redone | — |
| Mass | 2.40 kg limit | IDD Table 4.1.2-1 |
| Inhibits | ≥ 3 deployment switches, each an inhibit, plus a ground-leg inhibit; RBF or ABF in addition | IDD §4.1.4, §4.2.1(3) |
| Power-up | No system operating and the main power system not energized for ≥ 30 min after deployment, by a fault-tolerant timer: the OBC cannot boot at separation as it does now | IDD §4.2.1(2) |
| Battery | JSC 20793 programme: cell screening, flight acceptance tests, circuit interrupters, ISS safety review; extra assessment above 80 Wh (the pack is ≈ 50 Wh) | IDD §4.4.7 |
| Safety process | Phase 0/I to III safety data packages to the PSRP between L-10 and L-3.5 M; SSP 51700 hazard controls | IDD §3.4.1, §4.4 |
| Vibration | Soft-stow 5.76 Grms or hard-mount 9.47 Grms, 60 s per axis | IDD §4.3.2.1 |
| Rails | Edge radius 0.5 ± 0.1 mm (CDS says ≥ 1 mm "should"); either face first | IDD §4.1.1(4), §4.1.5(2) |
| Schedule | Contract and ICA at L-12 M; delivery to Voyager L-3.5 to L-1.5 M | IDD Table 3.4.1-1 |
| Cost | ROM $120k–250k all-in; the battery programme drives it | BASELINE §3 |

The full list is `LAUNCH_VEHICLE_REQUIREMENTS.md` §7. The NRCSD IDD found on the web is the 2018 initial release
(NR-NRCSD-S0003 Rev "-"); the current revision must come from Voyager before any compliance claim.

### 3.2 Trigger

No integrator commits in writing to ≤ 525 km by 2027-06-30, or the primary exceeds $250k all-in: start the NRCSD and
Electron requests; decide by 2027-09-30 (D7). An ISS decision on 2027-09-30 cannot meet a mid-2028 launch: the IDD
schedule needs the contract at L-12 M, so the earliest ISS launch is about 2028-09.

### 3.3 What to do now to keep the backup open

Cheap now, expensive later: three deployment switches with a ground-leg inhibit (also closes CDS §2.3.7/§2.3.8 on the
primary path, RID LV-05); a frame whose rails have flown in both the NRCSD and the EXOpod; cell screening records
kept from the start (BATTERY_DESIGN §9). Not done now: the 30-minute hardware power-up delay and the JSC 20793
programme.

## 4. Contingency

| Route | What it is | When to use | Cost and lead time |
|-------|------------|-------------|--------------------|
| Rocket Lab Electron rideshare from LC-1 (Mahia, New Zealand) | SSO at a selectable altitude (Kick Stage); 1U only in a third-party dispenser (the EXOpod has flown on Electron); rideshare CubeSats integrated at Long Beach about 40 days before launch | The primary cannot commit to ≤ 525 km | Price unknown; spacecraft export to New Zealand (EAR licence or STA) plus ITAR integration services: 6+ months (D7 research §3.9, §4.4) |
| NASA CSLI | No launch cost; next solicitation expected 2028; needs a US educational or nonprofit applicant | Only if the funding fails | Launch about 2030 or later (D7 research §4.1) |

## 5. Decision points

| # | Date | Decision | Input | If not met |
|---|------|----------|-------|------------|
| DP-1 | 2026-12-31 | Contracting entity named; RFQs sent to Exolaunch, ISISPACE, SEOPS, one question each to D-Orbit and Maverick | RID LV-09 | The schedule slips day for day |
| DP-2 | 2027-03-31 | Integrator selected | Written quotes: price, slot type, flight, the last 4–6 Transporter drop-off altitudes, test acceptance, export path, ephemeris position | Extend the RFQ; add Electron |
| DP-3 | 2027-06-30 | Contract signed with the 525 km cap | DP-2 | Trigger the backup (§3.2) |
| DP-4 | 2027-09-30 | Backup decision, if triggered | NRCSD and Electron quotes | — |
| DP-5 | L-8 M (2027-10-01) | Design frozen for the mission analysis inputs | CDR closed; mass properties ± 15 % | Rebooking |
| DP-6 | L-5 M (2028-01-01) | Orbit selected from SpaceX's list | List ≤ 525 km | Rebook under the clause |
| DP-7 | L-4 M (2028-02-01) | Environmental campaign passed | VI-LV-03 to 08 | Rebook |
| DP-8 | L-2 M (2028-04-01) | Licences in place; ephemeris capability accepted (VI-LV-13) | FCC, IARU, 18th SDS | Rebooking (UG §9.5.7.2) |
| DP-9 | L-50 d | Flight model shipped | Pre-ship review | Demanifest |

## 6. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| No ≤ 525 km option on the assigned flight | Medium (SpaceX posts 500–600 km) | High: FCC five-year case fails with wings failed | Contract clause; Electron contingency; DAS early |
| 2028 capacity sold out | Medium | High | Sign by 2027-06-30 |
| Ephemeris requirement not demonstrated (no GNSS chain implemented, RID LV-04) | High today | Mandatory rebooking | Implement and demonstrate by L-2 M, or contract a third party before integration |
| Schedule: Phase B with no hardware, mission analysis inputs due L-8 M (2027-10) | High | Rebooking | Dated master schedule from L backwards (RID LV-10) |
| Contracting entity missing | High today | No contract | DP-1 |
| Export review of a foreign integrator | Medium | 3–6 months | Counsel from Q4 2026; SEOPS quote |

## 7. NRCSD compliance summary (backup)

Read against NR-NRCSD-S0003 Rev "-" (2018). Status of each IDD section for the current design; the requirement text
and the design delta are in `LAUNCH_VEHICLE_REQUIREMENTS.md` §7.

| IDD § | Subject | Status |
|-------|---------|--------|
| 4.1.1 | Rails and envelope | Open: edge radius 0.5 ± 0.1 mm conflicts with CDS §2.2.7; check the bought frame |
| 4.1.2 | Mass ≤ 2.40 kg, CG ±2 cm | Met by estimate (1.92–2.00 kg); CG open |
| 4.1.3 | RBF or ABF reachable through the +Y access panels | Open (position) |
| 4.1.4 | Three deployment switches, ≤ 3 N each | Not met (two switches) |
| 4.1.5 | Deployables restrained by the CubeSat; either face first | Met for restraint; face order open |
| 4.1.6 | 0.5–2.0 m/s; ≤ 5°/s tip-off | Met (detumble from 10°/s) |
| 4.2.1 | 30 min with no system operating; three inhibits with a ground leg; charge circuit cannot energize loads; ≤ 6 in of wire to the first inhibit | Not met (OBC boots at separation; no ground leg); charge isolation open (VI-LV-15) |
| 4.2.2 | No electrical interface to the NRCSD | Met |
| 4.3.1–4.3.5 | 7/4/4 g, random 5.76 or 9.47 Grms, 1200 N on the rail ends | Open (no FEM; test at the NRCSD level) |
| 4.3.6–4.3.8 | Thermal, humidity, airlock depressurization 1.0 kPa/s | Open (analysis) |
| 4.4.1–4.4.5 | Frangibles, venting (MEVR ≤ 5080 cm), secondary locking, passivity, no pyrotechnics | Open (venting VI-LV-12); passivity met by inhibits |
| 4.4.6 | No detachable parts; NASA-STD-8719.14A | Met (design) |
| 4.4.7 | Batteries per JSC 20793 | Not started |
| 4.4.10–4.4.11 | Materials, outgassing, bonding (SSP 30245) | Open |
| 4.5 | Jettison; ODAR needed only above 5 kg or if designed to survive | Not required (1.6 kg, demises) |
| 4.6 | Regulatory evidence before handover; documentation | Open |

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-01-02 | Untracked original: an NRCSD IDD compliance plan, with a funding-audit notice (2026-10-14) marking the ISS route as the backup |
| B | 2026-10-27 | Document ID SQSO-OAP-001 assigned. Rewritten as the orbital access plan for the CDR: primary path (Falcon 9 Transporter from VSFB SLC-4E, Exolaunch, EXOpod Nova 1U adapter slot), contract chain, flow of the flight model, ISS backup deltas read against the NRCSD IDD (2018 release) instead of unverified section numbers, Electron contingency, decision points DP-1 to DP-9, risks. The old "Compliant" statements against the IDD are replaced by the status table of §7 |
