# SMART-QSO Executive Summary

**Document ID**: SQSO-ES-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## Mission Overview

**SMART-QSO** (Small Mission for Amateur Radio Technology - Quick Summary Operations) is a 1U CubeSat mission demonstrating AI-assisted amateur radio operations. The spacecraft uses an onboard AI system to generate contextual beacon messages while providing reliable amateur radio services to the global community.

## Mission Objectives

1. **Demonstrate AI-Assisted Amateur Radio**: First CubeSat to use onboard AI for dynamic beacon content generation
2. **Provide Amateur Radio Service**: Transmit beacons receivable by amateur operators worldwide
3. **Autonomous Operations**: Operate for 2+ years with minimal ground intervention
4. **Regulatory Compliance**: Full compliance with amateur radio regulations (no encryption, station ID)

## Key Technical Features

| Feature | Implementation |
|---------|----------------|
| Form Factor | 1U CubeSat (10×10×11.35 cm) |
| Mass | 1.2 kg (33% margin) |
| Power | 7W solar, 7Wh battery |
| AI Compute | NVIDIA Jetson Orin Nano |
| AI Model | TinyLM (~1 MB INT8) |
| Radio | VHF downlink, UHF uplink |
| Design Life | 2 years |

## Mission Architecture

```
┌──────────────────────────────────────────────────┐
│              SMART-QSO Mission                   │
├──────────────────────────────────────────────────┤
│                                                  │
│    Space Segment              Ground Segment     │
│    ┌──────────┐               ┌──────────┐      │
│    │ CubeSat  │───RF Link────►│ Ground   │      │
│    │ + AI     │◄──────────────│ Station  │      │
│    └──────────┘               └──────────┘      │
│         │                          │             │
│         │                          │             │
│         ▼                          ▼             │
│    ┌──────────┐               ┌──────────┐      │
│    │ Amateur  │               │ Mission  │      │
│    │ Community│               │ Ops      │      │
│    └──────────┘               └──────────┘      │
│                                                  │
└──────────────────────────────────────────────────┘
```

## Heritage and Approach

SMART-QSO follows NASA Ames Research Center Small Satellite Division heritage from successful missions:

- **GeneSat-1**: First autonomous biology CubeSat
- **PharmaSat**: Microfluidics in space
- **O/OREOS**: Astrobiology experiments
- **EcAMSat**: E. coli antibiotic resistance study

Key heritage practices applied:
- Rigorous but tailored Class D approach
- Comprehensive documentation
- Thorough verification and validation
- Strong configuration management
- Lessons learned integration

## Program Status

| Milestone | Status |
|-----------|--------|
| System Requirements Review (SRR) | ✓ Complete |
| Preliminary Design Review (PDR) | ✓ Complete |
| Critical Design Review (CDR) | ✓ Current |
| Test Readiness Review (TRR) | Planned |
| Flight Readiness Review (FRR) | Planned |
| Launch | TBD |

## Technical Maturity

| Aspect | Status |
|--------|--------|
| System Design | Complete |
| Software Design | Complete |
| Flight Software | Developed, tested |
| Documentation | 154 documents |
| Requirements | 97.8% addressed |
| Test Procedures | All defined |

## Key Design Features

### Graceful Degradation
```
Full AI Mode → Basic AI → Template Beacons → Safe Mode
     ↓            ↓            ↓             ↓
   Optimal    Reduced      Fallback      Survival
```

### Power-Aware Operations
- AI duty cycle limited to ≤10%
- Automatic power management
- Battery protection at 20% SOC
- Positive power margin in all modes

### Fault Tolerance
- Hardware watchdog (5-second timeout)
- Automatic safe mode on faults
- Boot validation with CRC
- State persistence in FRAM

## Budget Summary

| Budget | Allocation | Margin |
|--------|------------|--------|
| Mass | 1.2 kg / 2.0 kg max | 33% |
| Power (worst) | 0.35W / 0.8W available | 128% |
| Link (30° el) | +1.6 dB / 0 dB required | Positive |
| Data | 1.2 MB/day / 252 KB downlink | 58% |

## Risk Summary

| Risk Level | Count | Status |
|------------|-------|--------|
| High | 0 | No high risks |
| Medium | 2 | Mitigated |
| Low | 3 | Monitored |

Top risks (mitigated):
- AI power consumption → Duty cycle limiting
- Thermal management → Analysis and design margins

## Success Criteria

| Level | Criterion | Duration |
|-------|-----------|----------|
| **Level 1** | Beacon operations | 30 days |
| **Level 2** | AI-assisted beacons | 1 year |
| **Level 3** | Continuous service | 2+ years |

## Standards Compliance

| Standard | Application |
|----------|-------------|
| NPR 7120.5 | Program management |
| NPR 7123.1 | Systems engineering |
| NPR 7150.2 | Software engineering |
| NASA-STD-8719.13 | Software safety |
| NASA-STD-8719.14 | Orbital debris |
| CDS Rev 14.1 | CubeSat standard |

## Forward Plan

### Immediate (Post-CDR)
1. Flatsat integration and testing
2. Flight software verification
3. End-to-end testing

### Near-Term
4. Engineering model build
5. Environmental qualification
6. Flight model fabrication

### Pre-Launch
7. Flight acceptance testing
8. Launch integration
9. Operations readiness

## Conclusion

SMART-QSO is ready to proceed from Critical Design Review to fabrication and test. All design work is complete, documentation is comprehensive, and the team is prepared for the next phase. The mission represents an innovative approach to amateur radio satellites while maintaining the rigorous standards expected of NASA-heritage small satellite programs.

---

**Document Control**

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR release |
