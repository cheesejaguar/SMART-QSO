# Launch Campaign Checklist
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-OPS-005
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 8715.3, Cal Poly CDS Rev 14.1, NASA Ames SmallSat Heritage

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

## 3. Pre-Ship Activities (L-30 to L-14 days)

### 3.1 Hardware Readiness

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] All environmental tests complete | Review test reports | | |
| [ ] Final functional test passed | Test report attached | | |
| [ ] All NCRs closed or waivered | NCR log reviewed | | |
| [ ] Flight configuration verified | As-built matches as-designed | | |
| [ ] Battery charged to storage level | 50% SOC verified | | |
| [ ] Inhibits verified installed | 3 inhibits confirmed | | |
| [ ] RBF pin installed | Visual verification | | |
| [ ] Deployment switches verified | Continuity confirmed | | |
| [ ] Mass properties verified | Final mass: ___ g | | |
| [ ] CG location verified | Within CDS limits | | |

### 3.2 Software Readiness

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Flight software version loaded | SVD: v___.___.___ | | |
| [ ] Software checksum verified | CRC: 0x________ | | |
| [ ] Configuration parameters set | Config dump attached | | |
| [ ] Beacon message verified | Test reception confirmed | | |
| [ ] Safe mode verified | Safe mode test passed | | |
| [ ] Watchdog verified | Reset test passed | | |
| [ ] Time synchronized | UTC within 1 second | | |

### 3.3 Documentation Complete

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Pre-Ship Review completed | Review minutes attached | | |
| [ ] All RIDs closed | RID log reviewed | | |
| [ ] Shipping documentation complete | Manifest prepared | | |
| [ ] Handling procedures distributed | Team briefed | | |
| [ ] Emergency contacts listed | Contact sheet attached | | |
| [ ] Export documentation complete | License/exemption confirmed | | |

---

## 4. Shipping and Transport (L-14 to L-7 days)

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

## 5. Launch Site Arrival (L-7 to L-5 days)

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
| [ ] Beacon transmission verified | RSSI: ___ dBm | | |
| [ ] Command uplink verified | ACK received | | |
| [ ] Battery voltage verified | Voltage: ___ V | | |
| [ ] All sensors responding | Sensor check passed | | |
| [ ] Safe mode entry/exit verified | Test passed | | |

---

## 6. Deployer Integration (L-5 to L-3 days)

### 6.1 Pre-Integration

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Deployer inspection complete | P-POD/deployer verified | | |
| [ ] Rail cleanliness verified | Rails cleaned and inspected | | |
| [ ] Fit check performed | Smooth insertion confirmed | | |
| [ ] Deployment switch alignment | Switches contact rails | | |
| [ ] Access ports aligned | Ports accessible | | |

### 6.2 Integration

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Battery at flight charge level | SOC: ___% | | |
| [ ] RBF pin accessible | Pin location verified | | |
| [ ] Spacecraft inserted into deployer | Insertion complete | | |
| [ ] Deployment switch status verified | All switches open | | |
| [ ] Deployer door closed | Door secure | | |
| [ ] Deployer serial number recorded | S/N: ______ | | |

### 6.3 Post-Integration Test

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Umbilical connection verified | Telemetry received | | |
| [ ] Battery voltage verified | Voltage: ___ V | | |
| [ ] Health status nominal | All parameters green | | |
| [ ] Final state commanded | Awaiting deployment | | |
| [ ] Umbilical disconnected | Power isolated | | |

---

## 7. Launch Vehicle Integration (L-3 to L-1 days)

### 7.1 Deployer to LV

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Deployer mounted to adapter | Torque values recorded | | |
| [ ] Electrical connections verified | Continuity confirmed | | |
| [ ] Separation signal path verified | Test pulse confirmed | | |
| [ ] Thermal blankets installed | MLI in place | | |
| [ ] Final photos taken | Documentation complete | | |

### 7.2 RBF Pin Removal

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] RBF removal authorized | Authorization received | | |
| [ ] RBF pin removed | Pin secured | | |
| [ ] Deployment switch status | All switches open | | |
| [ ] Battery isolated | Inhibits in place | | |
| [ ] Photo of removed RBF | Photo attached | | |

---

## 8. Pre-Launch (L-1 day to L-0)

### 8.1 Final Readiness

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Weather forecast reviewed | Go/No-Go: ______ | | |
| [ ] Ground station ready | Link verified | | |
| [ ] Operations team in position | All stations manned | | |
| [ ] Telemetry system ready | Recording verified | | |
| [ ] TLE prediction received | Initial orbit confirmed | | |
| [ ] Contact schedule prepared | First pass: ______ UTC | | |

### 8.2 Launch Day

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] GO for launch confirmed | Mission Manager approval | | |
| [ ] Ground station tracking | Antenna positioned | | |
| [ ] Telemetry recording started | Recording: ______ | | |
| [ ] Launch occurred | T-0: ______ UTC | | |
| [ ] Launch vehicle nominal | Status: ______ | | |

---

## 9. Post-Separation (L+0 to L+48 hours)

### 9.1 Separation Confirmation

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Separation signal received | Time: ______ UTC | | |
| [ ] TLE updated | NORAD ID: ______ | | |
| [ ] First pass scheduled | Pass time: ______ UTC | | |

### 9.2 First Contact

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Beacon detected | Time: ______ UTC | | |
| [ ] Beacon decoded successfully | Callsign: ______ | | |
| [ ] Telemetry nominal | All parameters green | | |
| [ ] Battery voltage healthy | Voltage: ___ V | | |
| [ ] Solar array deployed (if applicable) | Confirmed | | |
| [ ] Antenna deployed | Confirmed | | |
| [ ] Attitude stable | Angular rate: ___ deg/s | | |

### 9.3 Initial Checkout

| Item | Verification | Signature | Date |
|------|--------------|-----------|------|
| [ ] Full telemetry dump received | Data archived | | |
| [ ] All subsystems responding | Status: ______ | | |
| [ ] Time synchronization | Delta: ___ seconds | | |
| [ ] Detumble initiated | B-dot active | | |
| [ ] Power positive | Net power: ___ mW | | |
| [ ] Commissioning phase started | Phase entry confirmed | | |

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

1. Confirm deployment status via telemetry
2. Command deployment retry (if applicable)
3. Assess mission impact
4. Implement contingency operations plan

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
