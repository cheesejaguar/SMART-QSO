# Environmental Test Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ETP-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** GSFC-STD-7000 (GEVS), MIL-STD-1540E

---

## 1. Purpose

This Environmental Test Plan defines the environmental testing approach for the SMART-QSO CubeSat to verify survivability and functionality under launch and on-orbit conditions.

## 2. Test Philosophy

### 2.1 Protoflight Approach

Per NASA Ames Small Satellite heritage and CubeSat standard practice:
- Single flight unit tested to protoflight levels
- Protoflight levels: Qualification duration at acceptance levels
- Reduces cost and schedule while providing adequate verification

### 2.2 Test Sequence

```
Pre-Environmental Baseline
         ↓
    Vibration Test
         ↓
  Thermal Vacuum Test
         ↓
    EMC/EMI Test
         ↓
   Deployment Test
         ↓
Post-Environmental Verification
```

## 3. Test Levels

### 3.1 Vibration Levels

| Axis | Frequency (Hz) | Level | Duration |
|------|----------------|-------|----------|
| All | 20-80 | +3 dB/oct | |
| All | 80-500 | 0.04 g²/Hz | |
| All | 500-2000 | -3 dB/oct | |
| | | | |
| **Protoflight** | | Grms = 6.8 | 1 min/axis |
| **Acceptance** | | Grms = 5.1 | 1 min/axis |

*Note: Final levels per launch provider ICD*

### 3.2 Thermal Vacuum Levels

| Parameter | Protoflight | Acceptance |
|-----------|-------------|------------|
| Hot operational | +50°C | +45°C |
| Cold operational | -20°C | -15°C |
| Hot survival | +60°C | N/A |
| Cold survival | -30°C | N/A |
| Cycles | 4 minimum | 2 minimum |
| Dwell time | 2 hours | 1 hour |
| Vacuum level | < 10⁻⁵ Torr | < 10⁻⁵ Torr |

### 3.3 EMC/EMI Levels

Per MIL-STD-461G tailored for CubeSat:

| Test | Requirement |
|------|-------------|
| RE102 | Radiated emissions |
| CE102 | Conducted emissions |
| RS103 | Radiated susceptibility (if required) |
| Self-compatibility | All systems operating |

## 4. Test Procedures

### 4.1 Pre-Environmental Baseline (TP-ENV-001)

| Step | Activity | Pass Criteria |
|------|----------|---------------|
| 1 | Mass measurement | Within budget |
| 2 | CG measurement | Within spec |
| 3 | Comprehensive functional test | All functions pass |
| 4 | RF characterization | Power within spec |
| 5 | Visual inspection | No defects |
| 6 | Photography | Configuration documented |

### 4.2 Vibration Test (TP-ENV-002)

**Pre-Test:**
| Step | Activity |
|------|----------|
| 1 | Abbreviated functional test |
| 2 | Visual inspection |
| 3 | Mount to shaker fixture |
| 4 | Install accelerometers |
| 5 | Verify data acquisition |

**Test Sequence (per axis):**
| Step | Activity | Criteria |
|------|----------|----------|
| 1 | Low-level sine sweep (0.25g) | Baseline signature |
| 2 | Random vibration (protoflight) | Per spectrum |
| 3 | Low-level sine sweep (0.25g) | < 10% shift from baseline |
| 4 | Abbreviated functional test | Critical functions pass |

**Post-Test:**
| Step | Activity |
|------|----------|
| 1 | Remove from fixture |
| 2 | Visual inspection |
| 3 | Full functional test |
| 4 | Compare signatures |
| 5 | Document results |

### 4.3 Thermal Vacuum Test (TP-ENV-003)

**Test Profile:**
```
    +60°C ─────────────  Survival Hot (2h dwell)
           ╱          ╲
    +50°C ─────────────  Operational Hot (2h dwell + functional)
           ╱          ╲
Ambient ──╱            ╲────────────────╱──
                        ╲              ╱
    -20°C               ─────────────  Operational Cold (2h dwell + functional)
                                    ╲
    -30°C                           ──  Survival Cold (2h dwell)
          |    |    |    |    |    |
         0h   6h  12h  18h  24h  30h
```

**Per Cycle:**
| Phase | Duration | Test |
|-------|----------|------|
| Ramp to hot operational | 2-3 hours | - |
| Hot dwell | 2 hours | Full functional |
| Ramp to hot survival | 30 min | - |
| Hot survival dwell | 2 hours | Monitoring only |
| Ramp to cold operational | 2-3 hours | - |
| Cold dwell | 2 hours | Full functional |
| Ramp to cold survival | 30 min | - |
| Cold survival dwell | 2 hours | Monitoring only |
| Return to ambient | 2 hours | - |

**Functional Test at Temperature:**
| Test | Hot | Cold |
|------|-----|------|
| Boot sequence | ✓ | ✓ |
| Beacon transmission | ✓ | ✓ |
| Telemetry | ✓ | ✓ |
| ADCS sensors | ✓ | ✓ |
| Jetson boot (if powered) | ✓ | N/A |

### 4.4 EMC/EMI Test (TP-ENV-004)

| Test | Setup | Pass Criteria |
|------|-------|---------------|
| RE102 | All systems on, beacon TX | Emissions below limits |
| CE102 | Power leads measured | Emissions below limits |
| Self-compatibility | All systems operating | No interference |
| RF spurious | Spectrum analyzer | Harmonics < -43 dBc |

### 4.5 Deployment Test (TP-ENV-005)

| Test | Condition | Pass Criteria |
|------|-----------|---------------|
| Antenna deployment | Flight-like actuation | Full deployment |
| Deployment time | Measured | Within spec |
| Deployed impedance | VNA measurement | VSWR < 2:1 |
| Deployment reliability | Multiple cycles | 100% success |

### 4.6 Post-Environmental Verification (TP-ENV-006)

| Step | Activity | Pass Criteria |
|------|----------|---------------|
| 1 | Full functional test | All functions pass |
| 2 | RF characterization | Within 1 dB of baseline |
| 3 | Mass measurement | No change |
| 4 | Visual inspection | No damage |
| 5 | Final photography | Configuration documented |

## 5. Test Facilities

### 5.1 Required Facilities

| Facility | Specification |
|----------|---------------|
| Vibration shaker | 50 kN force, 100g capability |
| Thermal vacuum chamber | -60°C to +100°C, 10⁻⁶ Torr |
| EMC chamber | Shielded room, antennas |
| Cleanroom | Class 100,000 |

### 5.2 Candidate Facilities

| Facility | Location | Capabilities |
|----------|----------|-------------|
| NASA Ames | Moffett Field, CA | Full environmental suite |
| Cal Poly | San Luis Obispo, CA | Vibration, thermal |
| NTS | Various | Commercial testing |
| Element | Various | Commercial testing |

## 6. Test Equipment

### 6.1 Vibration Test

| Equipment | Purpose |
|-----------|---------|
| Electrodynamic shaker | Vibration input |
| Accelerometers | Response measurement |
| Data acquisition | Recording |
| Control system | Closed-loop control |
| Fixture | Spacecraft mounting |

### 6.2 Thermal Vacuum Test

| Equipment | Purpose |
|-----------|---------|
| Vacuum chamber | Environment |
| Thermal shroud | Temperature control |
| Vacuum pump | Pressure |
| Thermocouples | Temperature measurement |
| Feedthroughs | Electrical interface |
| GSE | Spacecraft operation |

### 6.3 EMC Test

| Equipment | Purpose |
|-----------|---------|
| Spectrum analyzer | Emission measurement |
| Receive antennas | Field measurement |
| Signal generator | Susceptibility source |
| Current probes | CE measurement |
| LISN | Power line measurement |

## 7. Data Requirements

### 7.1 Vibration Data

| Data | Format | Retention |
|------|--------|-----------|
| Input spectrum | PSD plot | Permanent |
| Response spectra | PSD plot | Permanent |
| Sine sweep overlays | Bode plot | Permanent |
| Functional test results | Report | Permanent |
| Photos | JPEG | Permanent |

### 7.2 Thermal Vacuum Data

| Data | Format | Retention |
|------|--------|-----------|
| Temperature profiles | Time series | Permanent |
| Pressure profile | Time series | Permanent |
| Telemetry logs | CSV/JSON | Permanent |
| Functional test results | Report | Permanent |

### 7.3 EMC Data

| Data | Format | Retention |
|------|--------|-----------|
| Emission spectra | dBuV/m vs freq | Permanent |
| Limit comparison | Plot | Permanent |
| Susceptibility results | Report | Permanent |

## 8. Success Criteria

### 8.1 Vibration

- [ ] No resonance frequency shift > 10%
- [ ] No structural damage
- [ ] All functional tests pass
- [ ] No workmanship failures

### 8.2 Thermal Vacuum

- [ ] Operation at all temperature plateaus
- [ ] All functional tests pass at temperature
- [ ] No thermal cycling damage
- [ ] No outgassing issues

### 8.3 EMC

- [ ] Emissions within limits
- [ ] No self-interference
- [ ] Susceptibility tests pass (if required)

### 8.4 Overall

- [ ] All environmental tests complete
- [ ] No critical anomalies
- [ ] All anomalies resolved or waived
- [ ] Post-environmental functional test passes

## 9. Schedule

| Test | Duration | Dependencies |
|------|----------|--------------|
| Pre-env baseline | 2 days | Integration complete |
| Vibration | 3 days | Baseline complete |
| Thermal vacuum | 7 days | Vibration complete |
| EMC | 2 days | TVAC complete |
| Deployment | 1 day | EMC complete |
| Post-env verification | 2 days | All tests complete |
| **Total** | **17 days** | |

## 10. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Facility availability | Book early, identify alternates |
| Test failure | Plan for retest time |
| Anomaly investigation | Budget time for root cause |
| Schedule slip | Identify critical path, parallel activities |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
