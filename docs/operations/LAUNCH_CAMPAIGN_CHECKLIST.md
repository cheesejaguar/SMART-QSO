# Launch Campaign Checklist
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-OPS-005
**Revision:** B
**Date:** 2026-10-27
**Reference:** SpaceX Rideshare Payload User's Guide v11 (Aug 2026); Cal Poly CDS Rev 14.1; Exolaunch EXOpod Nova User
Manual Rev 1.2; `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001); `docs/operations/LAUNCH_SITE_OPS.md`

> Revision B (2026-10-27): the campaign is integrator-led. Exolaunch integrates SMART-QSO into the EXOpod Nova about
> 47 days before launch (planning dates, SQSO-LVR-001 §9) and delivers the dispenser to SpaceX; the team has no
> access after the door closes, and no electrical connection to the dispenser or the launch vehicle exists. Day
> numbers below are relative to the integration day I (≈ L-47 d) where the work is the team's, and to launch L
> where it is SpaceX's.

---

## 1. Purpose

This checklist provides a comprehensive list of activities and verifications required for the SMART-QSO launch campaign, from pre-ship through separation confirmation.

## 2. Scope

This checklist covers:
- Pre-ship activities
- Transport and handling
- Launch site operations
- Integration into deployer
- Pre-launch verification
- Launch day activities
- Post-separation operations

---

## 3. Pre-Ship Activities (I-30 to I-7 days)

### 3.1 Hardware Readiness

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] All environmental tests complete | Review test reports | | |
| [ ] Final functional test passed | Test report attached | | |
| [ ] All NCRs closed or waivered | NCR log reviewed | | |
| [ ] Flight configuration verified | As-built matches as-designed | | |
| [ ] Battery at the shipping SOC | ≤ 30 % if flown as air cargo (carrier's lithium rules), else 60 % | | |
| [ ] Inhibits verified | Inhibit-by-inhibit test report (VI-LV-06): 3 RF and 3 deployable inhibits | | |
| [ ] Battery-side drain measured | ≤ 20 µA stowed (VI-LV-14) | | |
| [ ] RBF pin installed | Visual verification | | |
| [ ] Deployment switches verified | Continuity confirmed | | |
| [ ] Mass properties verified | Final mass: ___ g (≤ 2.5 kg Nova; within the L-3.5 M bound) | | |
| [ ] CG location verified | Within ±20 mm | | |

### 3.2 Software Readiness

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Flight software version loaded | SVD: v___.___.___ | | |
| [ ] Software checksum verified | CRC: 0x________ | | |
| [ ] Configuration parameters set | Config dump attached | | |
| [ ] Beacon message verified | Test reception confirmed | | |
| [ ] Safe mode verified | Safe mode test passed | | |
| [ ] Watchdog verified | Reset test passed | | |
| [ ] Deployment and RF-silence parameters | 30 min and 45 min in the release build | | |
| [ ] Ephemeris capability accepted | Integrator's written acceptance (VI-LV-13) or third-party contract | | |

### 3.3 Documentation Complete

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Pre-Ship Review completed | Review minutes attached | | |
| [ ] All RIDs closed | RID log reviewed | | |
| [ ] Shipping documentation complete | Manifest prepared | | |
| [ ] Handling procedures distributed | Team briefed | | |
| [ ] Emergency contacts listed | Contact sheet attached | | |
| [ ] Export documentation complete | License/exemption confirmed | | |
| [ ] Licensing certification letter (UG Appendix F) | Signed before shipment | | |
| [ ] UN 38.3 summary and hazardous materials list | Attached to the shipment | | |

---

## 4. Shipping and Transport (I-7 to I-3 days)

### 4.1 Packaging

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Spacecraft in ESD-safe container | Container verified | | |
| [ ] Desiccant packs included | Humidity indicator green | | |
| [ ] Shock indicators attached | Baseline recorded | | |
| [ ] Temperature monitors attached | Baseline recorded | | |
| [ ] "Fragile" and "This End Up" labels | Labels verified | | |
| [ ] Outer container sealed | Seal number: ______ | | |
| [ ] Chain of custody initiated | Log started | | |

### 4.2 Transport Verification

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Transport company briefed | Handling requirements confirmed | | |
| [ ] Insurance confirmed | Policy number: ______ | | |
| [ ] Tracking number recorded | Tracking: ______ | | |
| [ ] Estimated arrival confirmed | ETA: ______ | | |
| [ ] Receiving personnel notified | Contact confirmed | | |

---

## 5. Arrival at the Integrator (I-3 to I-1 days)

### 5.1 Receiving Inspection

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Container intact | No visible damage | | |
| [ ] Shock indicators checked | Within limits: Yes/No | | |
| [ ] Temperature monitors checked | Within limits: Yes/No | | |
| [ ] Humidity indicator checked | Green: Yes/No | | |
| [ ] Chain of custody verified | Log complete | | |
| [ ] Spacecraft removed from container | ESD precautions taken | | |
| [ ] Visual inspection performed | No damage observed | | |
| [ ] Photo documentation | Photos attached | | |

### 5.2 Functional Verification

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] External power applied | Voltage: ___ V | | |
| [ ] Telemetry received | Signal confirmed | | |
| [ ] Beacon verified conducted into a load (no open-air RF, UG §9.5.2) | Decoded: Yes/No | | |
| [ ] Command uplink verified over the umbilical or conducted RF | ACK received | | |
| [ ] Battery voltage verified | Voltage: ___ V | | |
| [ ] All sensors responding | Sensor check passed | | |
| [ ] Safe mode entry/exit verified | Test passed | | |

---

## 6. Final Configuration and Dispenser Integration (day I)

### 6.1 Pre-Integration

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Rail measurement (Exolaunch guide) | In tolerance | | |
| [ ] Final charge with the RBF in | SOC 60 % ± 5 %; charge end ______ UTC | | |
| [ ] Keys provisioned; `ARM_FLIGHT` sent last | GSE log: 7 of 7 operations | | |
| [ ] OBC off, umbilical removed | Bus voltage 0 with the switches actuated | | |
| [ ] RBF removed | Serial ______ | | |

### 6.2 Integration

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Spacecraft inserted into the Nova slot (with the 1U adapter, or with the slot neighbours) | Insertion complete (Exolaunch) | | |
| [ ] Deployment switches actuated | Door closed, set screws set (Exolaunch) | | |
| [ ] Dispenser serial and slot recorded | S/N: ______ slot: ______ | | |
| [ ] Close-out photos | All faces visible through the windows | | |

### 6.3 Post-Integration Test

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] No post-integration electrical test | None possible (no umbilical, Nova UM §5.4) | | |
| [ ] Inputs to the encapsulation readiness letter | Charge date and SOC; flight-ready ≥ 45 days after encapsulation | | |

---

## 7. Dispenser to Launch Vehicle (SpaceX and Exolaunch; ≤ L-8 weeks to L-5 days)

### 7.1 Deployer to LV

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Dispenser delivered to the launch site | ≤ 8 weeks before launch (UG §9.5.1) | | |
| [ ] Dispenser mated and encapsulated | Integrator report | | |
| [ ] Encapsulated assembly mated to the LV | ≈ L-5 days (UG §10.1) | | |
| [ ] Launch readiness letter inputs | Integrator report | | |

### 7.2 RBF

The RBF is removed on day I, before the CubeSat enters the slot (§6.1); there is no later removal step.

---

## 8. Pre-Launch (L-1 day to L-0)

### 8.1 Final Readiness

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Weather forecast reviewed | Go/No-Go: ______ | | |
| [ ] Ground station ready | Link verified | | |
| [ ] Operations team in position | All stations manned | | |
| [ ] Telemetry system ready | Recording verified | | |
| [ ] Pre-launch OPM (UG Appendix E) loaded | Initial orbit confirmed | | |
| [ ] Contact schedule prepared for all separation times | First Bay Area pass S+0.7 to S+12.2 h | | |
| [ ] Space-Track and SpaceX ephemeris accounts tested | Test upload acknowledged | | |

### 8.2 Launch Day

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] GO for launch confirmed | Mission Manager approval | | |
| [ ] Ground station tracking | Antenna positioned | | |
| [ ] Telemetry recording started | Recording: ______ | | |
| [ ] Launch occurred | T-0: ______ UTC | | |
| [ ] Launch vehicle nominal | Status: ______ | | |

---

## 9. Post-Separation (S to S+48 hours)

### 9.1 Separation Confirmation

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Separation report and state vector received | Time: ______ UTC | | |
| [ ] Candidate objects from the 18th SDS | NORAD ID: ______ (claim within 30 days) | | |
| [ ] First pass scheduled | Pass time: ______ UTC | | |

### 9.2 First Contact

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Beacon detected | Time: ______ UTC | | |
| [ ] Beacon decoded successfully | Callsign: ______ | | |
| [ ] Telemetry nominal | All parameters green | | |
| [ ] Battery voltage healthy | Voltage: ___ V | | |
| [ ] Wings deployed (proxy: SOC recovery rate in sun; no deployment telemetry, LV-27) | Consistent: Yes/No | | |
| [ ] Antennas deployed (proxy: link margin against range) | Consistent: Yes/No | | |
| [ ] Detumbled and sun acquired (`ADCS_GET_ATTITUDE`) | Flags: __ / __ | | |

### 9.3 Initial Checkout

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] First authenticated command executed (C-2) | `GET_STATUS` response | | |
| [ ] TX-OFF proven (C-3) | Silence observed, then `TX_ENABLE` | | |
| [ ] Onboard time correlated (beacon `T=` against UTC of reception) | Offset: ______ s | | |
| [ ] Power positive | SOC rising in sun; ≥ 80 % at passes | | |
| [ ] First ephemeris published | ≤ 24 h after first contact | | |
| [ ] Commissioning started (COMMISSIONING_PLAN C-4) | Entry confirmed | | |

---

## 10. Contingency Procedures

### 10.1 No Beacon Received

1. Verify ground station configuration
2. Check TLE accuracy and antenna pointing
3. Wait for next pass
4. Attempt blind commanding if no beacon after 24 hours
5. Escalate to Mission Manager after 48 hours

### 10.2 Anomalous Telemetry

1. Record all telemetry
2. Compare to pre-launch baseline
3. Assess safe mode entry criteria
4. Command safe mode if necessary
5. Schedule anomaly review

### 10.3 Deployment Failure

1. Assess from the proxies (no deployment telemetry exists, LV-27)
2. No telecommand can retry a deployment (LV-07); the flight software makes 3 attempts 60 s apart and stops
3. Assess mission impact (MISSION_TIMELINE §8.2: a failed wing is power-positive)
4. Implement the contingency operations plan

---

## 11. Approvals

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Mission Manager | | | |
| Systems Engineer | | | |
| Quality Assurance | | | |
| Safety Officer | | | |

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | CDR launch review: integrator-led campaign (Exolaunch, EXOpod Nova); integration day I ≈ L-47 d; final charge 60 % ± 5 % with the RBF in, `ARM_FLIGHT` last, RBF out before integration; no post-integration test, umbilical or RBF step; ephemeris, licensing letter, UN 38.3 and drain checks; post-separation items match the flight dictionary and the telemetry that exists | CDR review (LV) |
