# SMART-QSO Hardware Readiness Tracker

**Document ID**: SQSO-HRT-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document tracks hardware procurement, assembly, and test readiness for the SMART-QSO mission, providing visibility into the critical path to flight readiness.

## 2. Summary Status

### 2.1 Overall Readiness

| Phase | Status | Blocking Items |
|-------|--------|----------------|
| CDR Documentation | Complete | None |
| Hardware Procurement | Pending | Awaiting funding |
| Flatsat Build | Pending | Parts procurement |
| EM Build | Planned | Flatsat completion |
| Flight Build | Planned | EM qualification |
| Flight Readiness | Planned | All above |

### 2.2 Milestone Status

| Milestone | Target | Actual | Status |
|-----------|--------|--------|--------|
| CDR | 2026-01-02 | 2026-01-02 | Complete |
| Parts Ordered | TBD | - | Pending |
| Parts Received | TBD | - | Pending |
| Flatsat Complete | TBD | - | Pending |
| EM Complete | TBD | - | Pending |
| Qualification Complete | TBD | - | Pending |
| Flight Complete | TBD | - | Pending |
| FRR | TBD | - | Pending |

## 3. Procurement Status

### 3.1 Critical Components

| Component | Qty | Vendor | P/N | Lead Time | Order Date | Received | Status |
|-----------|-----|--------|-----|-----------|------------|----------|--------|
| Jetson Orin Nano | 2 | NVIDIA | 900-13767-0030-000 | 4 wk | | | Pending |
| Apollo4 MCU | 3 | Ambiq | AMA4B2KK-KBR | 8 wk | | | Pending |
| Si4463 RF IC | 5 | SiLabs | SI4463-C2A-GMR | 6 wk | | | Pending |
| 18650 Cells | 6 | LG | INR18650MJ1 | 2 wk | | | Pending |
| Solar Cells | 8 | Spectrolab | XTJ Prime | 12 wk | | | Pending |

### 3.2 Structure Components

| Component | Qty | Vendor | Material | Lead Time | Status |
|-----------|-----|--------|----------|-----------|--------|
| Rails (4) | 1 set | TBD | Al 7075-T6 | 3 wk | Pending |
| Side Panels (4) | 1 set | TBD | Al 6061-T6 | 3 wk | Pending |
| Top Panel | 1 | TBD | Al 6061-T6 | 3 wk | Pending |
| Bottom Panel | 1 | TBD | Al 6061-T6 | 3 wk | Pending |
| Antenna Mount | 2 | TBD | Al 6061-T6 | 3 wk | Pending |

### 3.3 PCB Status

| Board | Qty | Status | Fab Date | Received |
|-------|-----|--------|----------|----------|
| EPS Board | 3 | Design Complete | | |
| OBC Board | 3 | Design Complete | | |
| RF Board | 3 | Design Complete | | |
| Jetson Carrier | 2 | Design Complete | | |
| ADCS Board | 3 | Design Complete | | |

### 3.4 Long-Lead Items

| Item | Lead Time | Critical Path | Notes |
|------|-----------|---------------|-------|
| Solar Cells | 12 weeks | Yes | Order ASAP |
| FRAM | 10 weeks | Yes | Radiation-tolerant |
| Apollo4 MCU | 8 weeks | Yes | Automotive grade |
| Connectors | 6 weeks | No | Hirose DF13/DF12 |

## 4. Build Status

### 4.1 Flatsat Build

**Purpose**: Software development and integration testing

| Subsystem | Status | Notes |
|-----------|--------|-------|
| EPS | Pending | Need boards + cells |
| OBC | Pending | Need board + components |
| RF | Pending | Need board + transceiver |
| Jetson | Pending | COTS + carrier |
| GSE | Pending | Power supply, cables |

**Flatsat Checklist**:
- [ ] All boards fabricated
- [ ] All boards assembled
- [ ] All boards tested (standalone)
- [ ] GSE ready
- [ ] Integration bench ready
- [ ] Software loaded
- [ ] Initial power-on
- [ ] Interface verification
- [ ] Flatsat operational

### 4.2 Engineering Model Build

**Purpose**: Environmental qualification

| Subsystem | Status | Notes |
|-----------|--------|-------|
| Structure | Pending | Flight-like |
| EPS | Pending | Flight-like |
| OBC | Pending | Flight-like |
| RF | Pending | Flight-like |
| Jetson | Pending | Flight-like |
| Harness | Pending | Flight-like |
| Antenna | Pending | Flight mechanism |

**EM Checklist**:
- [ ] Structure machined
- [ ] Structure inspected
- [ ] Boards assembled
- [ ] Boards tested
- [ ] Harness built
- [ ] Integration complete
- [ ] Functional test pass
- [ ] Mass properties measured
- [ ] Environmental test ready

### 4.3 Flight Unit Build

**Purpose**: Flight mission

| Subsystem | Status | Notes |
|-----------|--------|-------|
| Structure | Planned | Flight |
| EPS | Planned | Flight |
| OBC | Planned | Flight |
| RF | Planned | Flight |
| Jetson | Planned | Flight |
| Harness | Planned | Flight |
| Antenna | Planned | Flight |

## 5. Test Readiness

### 5.1 Flatsat Test Readiness

| Test | Procedure | Hardware Ready | SW Ready | Status |
|------|-----------|----------------|----------|--------|
| Boot/Init | TP-FLATSAT-001 | No | Yes | Pending |
| Mode Transition | TP-FLATSAT-002 | No | Yes | Pending |
| Telemetry | TP-FLATSAT-003 | No | Yes | Pending |
| Beacon | TP-FLATSAT-004 | No | Yes | Pending |
| Fault Response | TP-FLATSAT-005 | No | Yes | Pending |
| Power Mode | TP-FLATSAT-006 | No | Yes | Pending |
| Extended | TP-FLATSAT-007 | No | Yes | Pending |

### 5.2 Environmental Test Readiness

| Test | Facility | Scheduled | Ready |
|------|----------|-----------|-------|
| Vibration | TBD | No | No |
| Thermal Vacuum | TBD | No | No |
| EMC | TBD | No | No |

### 5.3 Ground Station Readiness

| Item | Status | Notes |
|------|--------|-------|
| Antenna installed | Pending | |
| Transceiver operational | Pending | |
| TNC configured | Pending | |
| MCS software deployed | Pending | |
| Database operational | Pending | |
| Tested with spacecraft sim | Pending | |

## 6. Documentation Readiness

### 6.1 CDR Documentation

| Document | Status | Version |
|----------|--------|---------|
| System Requirements | Complete | 1.0 |
| Software Requirements | Complete | 1.0 |
| Software Design | Complete | 1.0 |
| Interface Control Docs | Complete | 1.0 |
| Test Procedures | Complete | 1.0 |
| Safety Analysis | Complete | 1.0 |
| FMEA | Complete | 1.0 |

**Documentation Count**: 159 documents

### 6.2 Post-CDR Documentation Needs

| Document | When Needed | Status |
|----------|-------------|--------|
| As-built drawings | Post-fabrication | Pending |
| Test reports | Post-test | Pending |
| Calibration records | Post-calibration | Pending |
| NCR closures | Post-resolution | Pending |

## 7. Risk Items

### 7.1 Procurement Risks

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Solar cell lead time | Schedule | Order early | Open |
| Component obsolescence | Cost/Schedule | Second source | Open |
| Vendor delays | Schedule | Margin | Open |

### 7.2 Build Risks

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| PCB yield | Schedule | Order extras | Open |
| Assembly defects | Schedule | Workmanship stds | Open |
| Test failures | Schedule | Repair capability | Open |

## 8. Action Items

### 8.1 Immediate Actions (Post-CDR)

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| Secure funding for parts | PM | TBD | Open |
| Order long-lead items | Procurement | TBD | Open |
| Finalize vendor selection | SE | TBD | Open |
| Reserve test facilities | Test Lead | TBD | Open |

### 8.2 Near-Term Actions

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| Complete Gerber files | HW Lead | TBD | Open |
| Order PCB fabrication | Procurement | TBD | Open |
| Procure assembly supplies | Integration | TBD | Open |
| Set up flatsat bench | Integration | TBD | Open |

## 9. Schedule Dependencies

```
CRITICAL PATH

Solar Cells ──────────────────────────┐
(12 week lead)                        │
                                      ├──► EM Build ──► Qual ──► FRR
Structure ────────────────────────────┤
(3 week lead)                         │
                                      │
PCBs ─────┬──► Assembly ──► Flatsat ──┘
(4 week)  │     (2 week)    (4 week)
          │
Components┘
(8 week)
```

## 10. Weekly Status Meeting Template

```
HARDWARE READINESS STATUS MEETING

Date: ___________
Attendees: ___________

1. PROCUREMENT STATUS
   - Orders placed this week:
   - Items received:
   - Outstanding POs:
   - Issues:

2. BUILD STATUS
   - Flatsat progress:
   - EM progress:
   - Issues:

3. TEST STATUS
   - Tests completed:
   - Tests planned:
   - Issues:

4. ACTION ITEMS
   - New actions:
   - Closed actions:
   - Overdue actions:

5. SCHEDULE
   - On track: [ ] Yes [ ] No
   - If no, explain:

6. NEXT STEPS

Next meeting: ___________
```

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
