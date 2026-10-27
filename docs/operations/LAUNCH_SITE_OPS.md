# Launch Site Operations Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-OPS-004
**Revision:** D
**Date:** 2026-10-27
**Reference:** SpaceX Rideshare Payload User's Guide v11 (Aug 2026) §4.3, §7.6, §9.5, §10; Exolaunch EXOpod Nova User
Manual Rev 1.2; `docs/ORBITAL_ACCESS.md` §2.3; `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001)

---

## 1. Purpose

This document defines the SMART-QSO team's operations at the integrator's integration site, from arrival of the
flight model through its integration into the EXOpod Nova. On a Transporter-class rideshare the team does not
integrate with the launch vehicle: Exolaunch integrates the CubeSat into the dispenser and delivers the loaded
dispenser to SpaceX (no more than eight weeks before launch, UG §9.5.1); SpaceX mates it to the vehicle at about L-5
days (UG §10.1). After the dispenser door closes there is no access, no charging and no checkout (Nova UM §5.4;
UG §7.6). Where the integrator's mission ICD or integration guide differs, it governs.

## 2. Launch Campaign Overview

### 2.1 Campaign Timeline

| Phase | When (planning launch L = 2028-11-01, BASELINE Rev E §3) | Activities |
|-------|----------------------------------------|------------|
| Arrival and setup | ≈ L-50 d | Unpack, GSE setup, receiving inspection, rail measurement (Exolaunch guide) |
| Pre-integration | L-49 to L-48 d | Functional test on the umbilical (conducted RF into a load only; no open-air RF, UG §9.5.2) |
| Final configuration | L-47 d | Final charge to 60 % ± 5 % (RBF in), key load and GSE provisioning, `ARM_FLIGHT` last, umbilical removed |
| Integration | L-47 d | RBF removed; integration into the Nova slot by Exolaunch; door closed; close-out photos |
| Hand-over | L-47 d | Inputs to the integrator's encapsulation readiness letter (UG Appendix D, scenario A) |
| After hand-over | L-47 d to L | No access. The team moves to the operations centre (`docs/operations/MISSION_TIMELINE.md` §4) |

### 2.2 Team Composition

| Role | Count | Responsibilities |
|------|-------|------------------|
| Mission Manager | 1 | Overall campaign authority |
| Systems Engineer | 1 | Technical lead |
| Integration Engineer | 1 | Mechanical integration |
| Flight Software Engineer | 1 | SW configuration |
| Quality Assurance | 1 | Inspection, documentation |
| Safety Officer | 1 | Safety oversight |

## 3. Shipping and Receiving

### 3.1 Shipping Configuration

| Item | Configuration |
|------|---------------|
| Spacecraft | In protective container |
| Battery | ≤ 30 % SOC if shipped by air as a lithium battery contained in equipment (follow the carrier's current rules); 60 % if shipped by ground |
| RBF pins | Installed |
| Deployment switches | Constrained |
| Antennas | Stowed |
| Documentation | Shipped separately |

### 3.2 Shipping Container

| Requirement | Specification |
|-------------|---------------|
| Type | Pelican 1600 or equivalent |
| Cushioning | Closed-cell foam, custom cut |
| Humidity | Desiccant packs included |
| Shock indicators | ShockWatch or equivalent |
| Tilt indicators | TiltWatch or equivalent |

### 3.3 Receiving Inspection

| Check | Criteria | Result |
|-------|----------|--------|
| Container condition | No visible damage | ______ |
| Shock indicator | Not triggered | ______ |
| Tilt indicator | Not triggered | ______ |
| Seal intact | Tamper-evident seal OK | ______ |
| Documentation | All docs present | ______ |

## 4. Facility Requirements

### 4.1 Cleanroom

| Parameter | Requirement |
|-----------|-------------|
| Class | ISO 8 (Class 100,000) minimum |
| Temperature | 20-25°C |
| Humidity | 30-60% RH |
| ESD | ANSI/ESD S20.20 compliant |
| Access | Controlled, logged |

### 4.2 Equipment

| Equipment | Purpose |
|-----------|---------|
| ESD workstation | Spacecraft handling |
| Power supply | Battery charging, functional test |
| Test equipment | Multimeter, oscilloscope |
| RF receiver | Beacon verification |
| Laptop | Flight SW configuration |
| Camera | Documentation |

### 4.3 Consumables

| Item | Quantity |
|------|----------|
| IPA wipes | 1 container |
| Lint-free wipes | 1 box |
| Gloves (nitrile) | 2 boxes |
| ESD bags | 10 |
| Kapton tape | 1 roll |

## 5. Pre-Integration Activities

### 5.1 Initial Inspection

| Step | Action | Result |
|------|--------|--------|
| 1 | Remove from shipping container | ______ |
| 2 | Visual inspection - all faces | ______ |
| 3 | Photograph all faces | ______ |
| 4 | Check RBF pins installed | ______ |
| 5 | Check deployment switches | ______ |
| 6 | Check antenna stowage | ______ |
| 7 | Document any anomalies | ______ |

### 5.2 Functional Test

| Test | Procedure | Result |
|------|-----------|--------|
| Power on | Apply external power | ______ |
| Boot verification | Check debug console | ______ |
| Telemetry | Verify all channels | ______ |
| Beacon test | Conducted into a dummy load through an attenuator and decoded; no open-air RF (UG §9.5.2). The RF silence must be reset by `ARM_FLIGHT` afterwards | ______ |
| Command test | Send test commands | ______ |
| ADCS test | Verify sensor readings | ______ |
| Power off | Verify clean shutdown | ______ |

### 5.3 Battery Operations

| Step | Action | Criteria | Result |
|------|--------|----------|--------|
| 1 | Measure initial SOC | Record | ______ % |
| 2 | Connect charger | CC-CV mode | ______ |
| 3 | Charge to flight SOC (RBF inserted; the charge path must not power the loads, VI-LV-15) | 60 % ± 5 % | ______ % |
| 4 | Verify cell balance | < 50 mV | ______ mV |
| 5 | Disconnect charger | Verify | ______ |
| 6 | Record final SOC | Document | ______ % |

### 5.4 Flight Configuration

| Item | Configuration | Verified |
|------|---------------|----------|
| Flight software | v1.0.0 loaded | ______ |
| Beacon callsign | KK6GDB-11 set | ______ |
| Keys | Operational and recovery keys provisioned through the umbilical (SQSO-ICD-013) | ______ |
| `ARM_FLIGHT` | Sent as the **last** umbilical action: deployment sequence and RF silence ARMED | ______ |
| Loss-of-contact timeout | 30 days (D3) | ______ |
| Time | None to set: no real-time clock; the onboard time starts at separation | — |

## 6. Dispenser Integration

### 6.1 Pre-Integration Checklist

| Item | Status |
|------|--------|
| Spacecraft functional test complete | ______ |
| Battery at 60 % ± 5 % SOC, charge date and time recorded (for the integrator's Appendix D letter) | ______ |
| `ARM_FLIGHT` acknowledged and the OBC powered off | ______ |
| RBF removed (no access after integration) | ______ |
| Deployment switches verified | ______ |
| Antennas stowed and secured | ______ |
| Mass properties verified | ______ |
| Photos taken | ______ |

### 6.2 Integration Procedure

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Verify dispenser is clean | Visual |
| 2 | Position spacecraft at dispenser | Alignment |
| 3 | Slide spacecraft into rails | Smooth motion |
| 4 | Verify deployment switches | Actuated |
| 5 | Verify door clearance | No interference |
| 6 | Secure access panel | Fasteners |
| 7 | Set screws per the Nova integration guide (Exolaunch) | Integrator's record |

### 6.3 Post-Integration Verification

| Check | Criteria | Result |
|-------|----------|--------|
| Deployment switches actuated (no bus voltage at the test point, if the integrator allows a check through a window) | Actuated | ______ |
| RBF | Removed and tagged in the record | ______ |
| Door clearance | > 1 mm | ______ |
| Rail engagement | Full length | ______ |
| Photographs | All sides | ______ |

## 7. Final Closeout

### 7.1 Closeout Inspection

| Item | Verification | Result |
|------|--------------|--------|
| All fasteners torqued | Witness marks | ______ |
| No FOD present | Visual | ______ |
| RBF removed before integration (CDS §2.3.5.1) | Verify | ______ |
| Access panels secured | Verify | ______ |
| Documentation complete | Review | ______ |

### 7.2 RBF Pin Removal

| Step | Action | Time |
|------|--------|------|
| 1 | Confirm the final charge and `ARM_FLIGHT` are complete | Before integration |
| 2 | Remove the RBF pin; record serial and time | Before the CubeSat enters the slot |
| 3 | Confirm no RF (the radio is unpowered by the switches once the door closes) | At door close |
| 4 | Report complete | Mission manager |

## 8. Launch Readiness

### 8.1 Go/No-Go Criteria

| Criterion | Status Required |
|-----------|-----------------|
| Functional test passed | GO |
| Battery SOC at integration 60 % ± 5 % | GO |
| RBF removed | GO |
| Dispenser integration complete | GO |
| Documentation complete | GO |
| Range safety approval | GO |
| Ground station ready | GO |

### 8.2 Contingency Procedures

| Situation | Response |
|-----------|----------|
| Functional test fail | Troubleshoot, retest |
| Low battery before integration | Charge to 60 %, verify |
| Physical damage | Assess, document, disposition |
| Integration issue | Coordinate with LV provider |
| Launch delay after integration | Nothing to do on the spacecraft: the separation SOC stays ≥ 40 % for 9.4 months at a 50 µA battery-side drain, 18.9 months with none (MISSION_TIMELINE §3) |

## 9. Post-Launch Activities

### 9.1 Deployment Verification

| Event | Expected Time | Verification |
|-------|---------------|--------------|
| Separation | S | Integrator report; SpaceX state vector |
| Wings deploy, then the AntS | Wings S+30 min; the AntS once its temperature reads ≥ −17 °C (decision C-18; S+30.6 min warm, S+80.7 min after a cold eclipse separation, at worst about S+25.8 h) | HK1 `DEP` and `AD` in the first housekeeping frames heard |
| First beacon | S+45 min, or when the AntS deploys if later (decision C-19); first Bay Area pass S+0.7 to S+12.1 h (SQSO-MTL-001 §5) | Ground station or SatNOGS |
| Detumbled | S+0.25 h (simulation; SQSO-MTL-001 §8.4) | HK1 `W` and `AM` at the first pass |

### 9.2 Pack-Out

| Item | Disposition |
|------|-------------|
| GSE | Return ship |
| Documentation | Archive |
| Unused consumables | Dispose/return |
| Shipping containers | Return ship |

## 10. Documentation

### 10.1 Required Records

| Document | Purpose |
|----------|---------|
| Shipping log | Chain of custody |
| Inspection photos | As-shipped condition |
| Functional test data | Pre-integration verification |
| Integration photos | Configuration documentation |
| Closeout checklist | Final configuration |
| RBF removal record | Launch configuration |

### 10.2 Data Retention

| Record | Retention |
|--------|-----------|
| Test data | Mission lifetime + 3 years |
| Photos | Mission lifetime + 3 years |
| Checklists | Mission lifetime + 3 years |
| Anomaly reports | Permanent |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | CDR launch review: integrator-led integration (Exolaunch, EXOpod Nova) replaces team integration at the launch site; final charge to 60 % ± 5 % with the RBF in, key load and `ARM_FLIGHT` last, RBF out before integration, no access afterwards; shipping state of charge; no UTC to set; post-launch times from the simulation | CDR review (LV) |
| C | 2026-10-27 | CDR 2026 consistency pass: planning launch 2028-11-01 (BASELINE Rev E §3) in §2.1 | CDR documentation review |
| D | 2026-10-27 | Documentation audit (post-CDR 2026): §9.1 the AntS gate and transmitter hold of decisions C-18 and C-19, first pass statistics from SQSO-MTL-001 2.7, deployment and attitude from HK1 (were "not observable" and `ADCS_GET_ATTITUDE`) | Documentation audit |
