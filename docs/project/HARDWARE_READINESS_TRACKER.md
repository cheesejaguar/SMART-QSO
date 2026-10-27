# SMART-QSO Hardware Readiness Tracker

**Document ID**: SQSO-HRT-001
**Version**: 2.2
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
| Design documentation | Build-to designs of every custom board: schematic captured as code with selected parts, KiCad netlist, BOM with stresses and ERC (SQSO-EPS-BRD-001, SQSO-OBC-BRD-001, SQSO-RF-BRD-001, SQSO-JET-BRD-001, SQSO-ICX-001); mechanical CAD and FE model (SQSO-MECH-CAD-001, `docs/environment/STRUCTURAL_ANALYSIS.md`); harness design (SQSO-HAR-001) | Complete 2026-10-27 | No PCB layout |
| EM schematic capture | **Complete** (schematic as code, ERC clean; master pin allocation SQSO-ICX-001) | — | — |
| PDR | Not held; board to be named by 2026-12-01 (DA-03) | 2027-01-12 | The PDR board (the CDR 2026 design review's RIDs are all dispositioned, SQSO-REV-CDR-002 §7.1) |
| EM procurement | Not started; no RFQ sent | 2027-01-15 to 2027-04-09 | **EM funding** (assumed 2027-01-15; 2027-03-15 costs 0.2 weeks of the GOLD margin); PDR; procurement data reviews by 2027-01-15 (DA-18) |
| EM layout and boards | Not started; no layout designer engaged | Layout 2027-01-12 to 2027-04-27 (two designers, DA-04); boards 2027-04-27 to 2027-06-01 | Layout designers; VI-RFB-12 (2026-11-13); NVIDIA guides before the carrier layout (DA-19) |
| EM integration and tests | Not started | 2027-06-01 to 2027-09-21 (≥ 500 h on the EM boards, VI-11) | EM hardware; facilities |
| CDR | Not held | 2027-09-28 | EM test results |
| FM procurement and boards | Not started | 2027-09-28 to 2027-12-21 | CDR; FM funding |
| FM integration, burn-in and protoflight tests | Not started | 2027-12-21 to 2028-04-21 (FM burn-in ≥ 200 h, VI-12) | FM hardware; facilities |
| Protoflight campaign end; delivery | — | 2028-04-21; 2028-05-19 (the integrator needs the campaign done by L − 4 months: 71 days of float for 2028-11-01, 80 days late for June 2028) | Pre-ship review; regulatory filings |

## 3. Procurement (buy items, `docs/hardware/BUILD_VS_BUY.md`)

Quantities: EM + FM (+ spare where noted). Lead times are vendor statements where cited, otherwise assumptions [A]
for the schedule. Prices are list prices, not quotes.

| Item | Selected part (alternates) | Qty | Lead time | RFQ | Order | Received | Status |
|------|----------------------------|-----|-----------|-----|-------|----------|--------|
| 1U structure | ISIS, Pumpkin or Spacemanic 1U frame | 2 | 12 wk [A]; 1U catalogue shrinking, order early | Not sent | — | — | Not started |
| Solar panels and deployable wings | Custom panels with AZUR 3G30A cells (DHV, Spacemanic, ISIS or Pumpkin) | 2 sets | **3-7 months, set by cell supply (BVB §2); EM and FM ordered together** | Not sent | — | — | Not started; schedule item PANELS (201 d float for 2028-11) |
| Antennas | ISIS AntS 1U-3U, monopole configuration | 2 | 12 wk [A] | Not sent | — | — | Not started |
| Magnetometer | PNI RM3100 on board A2 (part on the OBC BOM; RID GNC-04, chair decision; a bought module fits J10 as the fallback) | 2 + 1 | 8 wk [A] | — | — | — | Not started |
| Magnetorquers | CubeSpace CR0002 rods (selected; air-core coils the alternate) | 2 sets | 12 wk [A] | Not sent | — | — | Not started; remanence measured on the EM (AV-V4) |
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
| OBC (STM32L4R5ZI), boards A2 and A3 | `hardware/electrical/obc/OBC_BOARD_DESIGN.md` (SQSO-OBC-BRD-001), `hardware/electrical/INTERCONNECT.md` | Captured as code, ERC clean (351 + 15 + 8 placements) | Not started (5 + 2 designer-weeks) | — | — |
| EPS (4S trackers, BMS, inhibits, passivation, PDU) | `hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001) | Captured as code, ERC clean (700 placements) | Not started (9 designer-weeks) | — | — |
| Battery pack and isolated mount | `hardware/eps/BATTERY_DESIGN.md`; sense board on the EPS BOM | Sense board captured with the EPS | — | — | — |
| U/V FM transponder and 2 m transmitter | `hardware/electrical/rf/RF_BOARD_DESIGN.md` (SQSO-RF-BRD-001) | Captured as code, ERC clean (300 fitted) | Not started (6 designer-weeks) | — | — |
| Jetson carrier and power switch | `hardware/electrical/jetson/CARRIER_BOARD_DESIGN.md` (SQSO-JET-BRD-001) | Captured as code, ERC clean (137 placements) | Not started (5 designer-weeks) | — | — |
| Harness | `hardware/harness/HARNESS_DESIGN.md` (SQSO-HAR-001) | Wire list and routing in the CAD | — | — | — |

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
| Requirements (MRD, system) | MRD Rev G, SYS Rev H (proposed: CR-2026-009, CR-2026-010, CR-2026-018, CR-2026-019; PI decision by 2027-01-12); seven wordings await the PI (DA-20) |
| Software requirements, design, test plan | Exist; NPR 7150.2D Appendix C mapping drafted (SQSO-FSW-012, `docs/software/NPR7150_COMPLIANCE_MATRIX.md`) |
| ICDs | Internal ICDs exist; the integrator ICD awaits the contract |
| Test procedures | 42 procedures; environmental levels corrected (TP-ENV-002 Rev E, TP-ENV-003 Rev D, TP-ENV-005 1.1) |
| Safety analysis | Software hazard analysis; system hazard analysis at the level of the CDR design (SQSO-SDP-002, draft, no hazard closed); battery safety assessment (SQSO-BSA-001) |
| FMEA | Software FMEA, system FMEA/CIL (SQSO-FMEA-002) and board-level FMEA of the round-2 boards (SQSO-FMEA-003, draft; RID P3P-04) |
| Drawings and build-to package | Schematics as code, BOMs, netlists; CAD and review drawings; no PCB layouts or manufacturing drawings |

## 7. Procurement and Supply-Chain Risks

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Solar panel lead time 3-7 months, cell supply | Schedule (201 d float for 2028-11; PANELS) | Order EM and FM sets together after the PDR | Open |
| 1U COTS catalogue shrinking (several vendors delisted 1U lines, BVB §2.1) | Part availability | RFQ three vendors per item; buy the FM item with the EM item where storage allows | Open |
| EM funding after 2027-01-15 | Each week late moves the protoflight campaign a week (`mission/SCHEDULE.md` §3) | Funding decision before the PDR | Open |
| Export licence for the GNSS receiver (Czech) and a TAA for a foreign integrator | Schedule | Vendor handles the licence; export counsel at the contract | Open |

## 8. Actions

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| Send RFQs: structure, panels, AntS, magnetometer, gyro, GNSS (ask stock, lead time, export) | PI | 2026-12-31 | Open |
| Send integrator RFQs, including test-pod rental and test services (D7) | PI | 2026-12-31 | Open |
| Obtain an STM32L4R5ZI development board for HIL | Flight software | 2026-11-30 | Open |
| Engage two PCB layout designers for a 2027-01-12 start (DA-04) | PI | 2026-12-15 | Open |
| Name the PDR board (DA-03) | PI | 2026-12-01 | Open |
| Procurement data reviews before the EM order (DA-18) | EPS designer, harness designer, payload lead | 2027-01-15 | Open |
| Request quotes from two test facilities for the ETP §4 flow and the EM tests | Test lead (PI until named) | 2027-03-31 | Open |
| Secure EM funding | PI | 2027-01-15 | Open |

## 9. Schedule Dependencies (critical path)

```
PDR ─► EM layout (two designers, 15 wk) ─► EM boards ─┐
                                EM COTS buys (12 wk) ───┴─► EM integration ─► EM tests ─► CDR
CDR ─► FM COTS buys (12 wk) ─► FM integration ─► FM functional/DITL ─► FM burn-in ─► protoflight tests ─► pre-ship ─► delivery
Solar panels EM+FM (30 wk, from the PDR) ─────────────────────────────► FM integration (201 d float)
```

Float on the critical path to the L − 4 month protoflight milestone: −80 days for the June 2028 opportunity, +71 days
for the 2028-11-01 planning launch (`mission/SCHEDULE.md` Rev E §3).

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
| 2.1 | 2026-10-27 | CDR round 3, systems and programmatics | Status after the round-2 design completion (RID P3P-06): schematic capture complete for every custom board, CAD and FE model exist; layout not started (designer-weeks per board, DA-04); magnetometer the RM3100 on board A2 (was GomSpace M315), rods selected; build items point to the board designs; phases, float and critical path from SQSO-SCH-001 D; actions DA-03, DA-04, DA-18 |
| 2.2 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): document status rows (MRD G, SYS H, NPR 7150.2D mapping drafted, system hazard analysis, board FMEA); PDR blocking items; schedule cited at SQSO-SCH-001 E |
