# SMART-QSO Hardware Readiness Tracker

**Document ID**: SQSO-HRT-001
**Version**: 2.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Live tracker. **No hardware has been ordered, received, built or tested.** Version 1.0 was labelled "CDR
Baseline" and listed a CDR as complete on 2026-01-02; no CDR has been held (BASELINE §1, finding PI-04).

## 1. Purpose

This tracker shows, for every hardware item, where it is between selection and flight, and what the critical path
needs from it. Dates come from `mission/SCHEDULE.md` (SQSO-SCH-001). Update it with every order, delivery and test,
and at least every 30 days.

## 2. Summary

| Phase | Status | Planned (early dates) | Blocking items |
|-------|--------|-----------------------|----------------|
| Design documentation | Subsystem design documents exist; block-level schematic drawings only (`hardware/diagrams/schematics/`) | — | No schematic capture or PCB layout in a CAD tool |
| EM schematic capture | Not started | 2026-10-27 to 2027-01-05 | A designer; the OBC pin map is provisional (`software/rust/firmware/src/gpio.rs`) |
| PDR | Not held | 2027-01-12 | This review's RIDs; schematic capture |
| EM procurement | Not started; no RFQ sent | 2027-01-15 to 2027-04-09 | **EM funding** (assumed 2027-01-15; a late-2028 launch tolerates about 2027-03-15); PDR |
| EM boards | Not started | 2027-01-12 to 2027-04-13 | Layout |
| EM integration and tests | Not started | 2027-04-13 to 2027-08-03 | EM hardware; facilities |
| CDR | Not held | 2027-08-03 | EM test results |
| FM procurement and boards | Not started | 2027-08-10 to 2027-11-02 | CDR; FM funding |
| FM integration and protoflight tests | Not started | 2027-11-02 to 2028-02-22 | FM hardware; facilities |
| Protoflight campaign end; delivery | — | 2028-02-22; 2028-03-21 (the integrator needs the campaign done by L − 4 months: missed by 21 days for June 2028) | Pre-ship review; regulatory filings |

## 3. Procurement (buy items, `docs/hardware/BUILD_VS_BUY.md`)

Quantities: EM + FM (+ spare where noted). Lead times are vendor statements where cited, otherwise assumptions [A]
for the schedule. Prices are list prices, not quotes.

| Item | Selected part (alternates) | Qty | Lead time | RFQ | Order | Received | Status |
|------|----------------------------|-----|-----------|-----|-------|----------|--------|
| 1U structure | ISIS, Pumpkin or Spacemanic 1U frame | 2 | 12 wk [A]; 1U catalogue shrinking, order early | Not sent | — | — | Not started |
| Solar panels and deployable wings | Custom panels with AZUR 3G30A cells (DHV, Spacemanic, ISIS or Pumpkin) | 2 sets | **3-7 months, set by cell supply (BVB §2); EM and FM ordered together** | Not sent | — | — | Not started; schedule item PANELS (60 d float for June 2028) |
| Antennas | ISIS AntS 1U-3U, monopole configuration | 2 | 12 wk [A] | Not sent | — | — | Not started |
| Magnetometer | GomSpace M315 (AAC MM200, Spacemanic Lodestone) | 2 | 12 wk [A] | Not sent | — | — | Not started |
| Magnetorquers | CubeSpace CR0002 rods, or the air-core coils (build) | 2 sets | 12 wk [A] | Not sent | — | — | Decision open (rod remanence) |
| Gyro | Epson M-G370PDT0 (Bosch BMI088 backup on the OBC) | 2 + 1 | 8-12 wk [A] | Not sent | — | — | Not started (D9 action: quote) |
| GNSS receiver and patch | SkyFox piNAV-NG/FM with Spacemanic ZENITH V3a (Spacemanic Celeste) | 2 | 12 wk [A]; Czech export licence handled by the vendor | Not sent | — | — | Not started (D6 action: quote both) |
| AI module | NVIDIA Jetson Orin Nano 8 GB, P/N 900-13767-0030-000 | 2 + 1 | 4 wk [A] | — | — | — | Not started |
| OBC processor | STM32L4R5ZI | 3 + dev board | 8 wk [A] | — | — | — | Not started; a development board is needed now for HIL (`software/flight/tests/hil/HIL_TEST_PLAN.md`) |
| Battery cells | Samsung INR18650-35E, one flight lot for screening (D8) | ≥ 16 (screen to 4 flight + spares) + EM set | 4 wk [A] | — | — | — | Not started; LEO cycling at 4.10 V (BATTERY_DESIGN §9) |
| EPS alternate | ISIS ICEPS2 Type B (only if the custom EPS is dropped) | — | — | — | — | — | Alternate only |
| Integrator test pod | Integrator's TestPod (rental) | 1 | With the contract | In the integrator RFQ | — | — | Not started |

Version 1.0 listed an Apollo4 MCU, a Si4463 radio IC, LG INR18650MJ1 cells and Spectrolab XTJ Prime cells; none is in
the baseline.

## 4. Build Items

| Item | Design document | Schematic | Layout | EM build | FM build |
|------|-----------------|-----------|--------|----------|----------|
| OBC (STM32L4R5ZI) | `hardware/obc/OBC_DESIGN.md`, `hardware/obc/OBC_ELECTRICAL_DESIGN.md` | Block level only | — | — | — |
| EPS (4S charger, BMS, passivation switch, PDU) | `hardware/eps/EPS_ELECTRICAL_DESIGN.md`, `hardware/eps/PDU_DESIGN.md` | Block level only | — | — | — |
| Battery pack and isolated mount | `hardware/eps/BATTERY_DESIGN.md` | Block level only | — | — | — |
| U/V FM transponder and 2 m transmitter | `hardware/rf/TRANSMITTER_DESIGN.md`, `hardware/rf/RF_ELECTRICAL_DESIGN.md` | Block level only | — | — | — |
| Jetson carrier and power switch | `hardware/payload-jetson/` | Block level only | — | — | — |
| Harness | `hardware/harness/HARNESS_DESIGN.md` | — | — | — | — |

## 5. Test Readiness

| Test | Procedure | Article | Facility | Booked | Hardware ready |
|------|-----------|---------|----------|--------|----------------|
| Flatsat and HIL | TP-FLATSAT-001; HIL plan | Development board, EM boards | Bench | — | No |
| EM development tests (VI-01 to VI-09) | `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §6, §8 | EM | Not selected (ETP §7) | No | No |
| Random vibration | TP-ENV-002 | EM, FM | Not selected | No | No |
| Thermal vacuum and bake-out | TP-ENV-003 | EM, FM | Not selected | No | No |
| EMC | TP-ENV-004 | EM, FM | Not selected | No | No |
| Residual dipole | ETP §3.9 | FM | Not selected (university Helmholtz cage) | No | No |
| End-to-end and DITL | TP-GS-001, TP-SYS-002 | FM | Own ground station | — | No |

Ground station: no station equipment has been installed or tested (TP-GS-001 not run).

## 6. Documentation Readiness

Status of the documents a build needs, as they are, not as version 1.0 claimed ("Complete" for every item):

| Document | Status |
|----------|--------|
| Requirements (MRD, system) | Rebaselined at the CDR review (MRD Rev D, SYS Rev E); seven wordings await the PI |
| Software requirements, design, test plan | Exist; Appendix C mapping of NPR 7150.2D missing |
| ICDs | Internal ICDs exist; the integrator ICD awaits the contract |
| Test procedures | 42 procedures; environmental levels corrected (TP-ENV-002 Rev E, TP-ENV-003 Rev D, TP-ENV-005 1.1) |
| Safety analysis | Software hazard analysis only; system safety data package is a skeleton |
| FMEA | Software FMEA only; no hardware FMEA |
| Drawings and build-to package | None |

## 7. Procurement and Supply-Chain Risks

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Solar panel lead time 3-7 months, cell supply | Schedule (60 d float for June 2028) | Order EM and FM sets together after the PDR | Open |
| 1U COTS catalogue shrinking (several vendors delisted 1U lines, BVB §2.1) | Part availability | RFQ three vendors per item; buy the FM item with the EM item where storage allows | Open |
| EM funding after 2027-01-15 | Each week late moves the protoflight campaign a week (`mission/SCHEDULE.md` §3) | Funding decision before the PDR | Open |
| Export licence for the GNSS receiver (Czech) and a TAA for a foreign integrator | Schedule | Vendor handles the licence; export counsel at the contract | Open |

## 8. Actions

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| Send RFQs: structure, panels, AntS, magnetometer, gyro, GNSS (ask stock, lead time, export) | PI | 2026-12-31 | Open |
| Send integrator RFQs, including test-pod rental and test services (D7) | PI | 2026-12-31 | Open |
| Obtain an STM32L4R5ZI development board for HIL | Flight software | 2026-11-30 | Open |
| Find a schematic and layout designer; start the EM schematic capture | PI | 2026-11-15 | Open |
| Request quotes from two test facilities for the ETP §4 flow and the EM tests | Test lead (PI until named) | 2027-03-31 | Open |
| Secure EM funding | PI | 2027-01-15 | Open |

## 9. Schedule Dependencies (critical path)

```
EM schematic capture ─► PDR ─► EM layout ─► EM boards ─┐
                                EM COTS buys (12 wk) ───┴─► EM integration ─► EM tests ─► CDR
CDR ─► FM COTS buys (12 wk) ─► FM integration ─► FM functional/DITL ─► protoflight tests ─► pre-ship ─► delivery
Solar panels EM+FM (30 wk, from the PDR) ─────────────────────────────► FM integration (60 d float)
```

Float on the critical path to the L − 4 month protoflight milestone: −21 days for a 2028-06-01 launch, +130 days for
2028-11-01 (`mission/SCHEDULE.md` §3).

## 10. Weekly Status Template

```
HARDWARE READINESS STATUS
Date:
1. Procurement: RFQs sent / orders placed / items received / issues
2. Build: EM / FM progress / issues
3. Test: completed / planned / facility bookings / issues
4. Critical-path float this week (schedule_cpm.py):
5. Actions: new / closed / overdue
```

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline (claimed a CDR held on 2026-01-02 and every document complete) |
| 2.0 | 2026-10-27 | CDR review, PM/SE/V&V | Rewritten to the real state: nothing ordered or built; parts per the build-versus-buy decisions (Apollo4, Si4463, LG MJ1 and Spectrolab cells removed); dated phases and actions from the critical-path schedule; build-item design status; facility status; documentation status as it is; supply-chain risks (finding PI-04, RID PM-08) |
