# IARU Satellite Frequency Coordination
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** IARU Satellite Frequency Coordination Guidelines

---

## 1. Purpose

This document provides the IARU (International Amateur Radio Union) frequency coordination application for the SMART-QSO CubeSat amateur radio experiment.

## 2. Mission Summary

### 2.1 Spacecraft Identification

| Parameter | Value |
|-----------|-------|
| Spacecraft Name | SMART-QSO |
| NORAD ID | TBD (post-launch) |
| International Designator | TBD (post-launch) |
| Operator | NASA Ames Research Center |
| Country | United States |

### 2.2 Mission Description

SMART-QSO is a 1U CubeSat technology demonstration mission experimenting with onboard AI for amateur radio beacon generation. The spacecraft will transmit telemetry beacons with AI-generated content on amateur satellite frequencies, demonstrating adaptive communication capabilities while providing an educational amateur radio resource.

### 2.3 Amateur Radio Justification

| Criterion | Compliance |
|-----------|------------|
| Open to all amateurs | Yes - all transmissions in the clear |
| No encryption | Yes - AX.25 standard protocol |
| Educational purpose | Yes - AI/amateur radio technology demonstration |
| Non-commercial | Yes - NASA research mission |
| Amateur operator control | Yes - licensed operators at ground station |

## 3. Frequency Request

### 3.1 Downlink Frequencies

| Parameter | Primary | Backup |
|-----------|---------|--------|
| Frequency | 145.825 MHz | 145.900 MHz |
| Band | 2m (VHF) | 2m (VHF) |
| Mode | FM/AFSK | FM/AFSK |
| Data Rate | 1200 bps | 1200 bps |
| Protocol | AX.25 | AX.25 |
| Power (EIRP) | 27 dBm (0.5W) | 27 dBm (0.5W) |
| Bandwidth | 15 kHz | 15 kHz |
| Duty Cycle | < 10% | < 10% |

### 3.2 Frequency Justification

145.825 MHz is requested as the primary downlink frequency because:
- Standard amateur satellite beacon frequency
- Compatible with existing ground station infrastructure
- Within ITU Region 1/2/3 amateur satellite allocation
- Wide amateur community reception capability
- SatNOGS network compatibility

### 3.3 No Uplink Required

SMART-QSO operates as a beacon-only mission with no command uplink on amateur frequencies. All commanding is performed via separate non-amateur frequencies.

## 4. Technical Specifications

### 4.1 Transmitter Characteristics

| Parameter | Specification |
|-----------|---------------|
| Transmitter Type | Commercial module (modified) |
| RF Output Power | 0.5W to 2W (selectable) |
| Nominal Power | 0.5W |
| Frequency Stability | ± 1 kHz |
| Spurious Emissions | < -43 dBc |
| Harmonics | < -50 dBc |
| Modulation | AFSK (1200 baud) |
| Deviation | ± 3 kHz |

### 4.2 Antenna System

| Parameter | Specification |
|-----------|---------------|
| Type | Deployable dipole |
| Polarization | Linear |
| Gain | 0 dBi (isotropic) |
| Pattern | Omnidirectional |
| VSWR | < 2:1 at 145.825 MHz |

### 4.3 Link Budget

| Parameter | Value |
|-----------|-------|
| TX Power | 27 dBm |
| TX Antenna Gain | 0 dBi |
| EIRP | 27 dBm |
| Path Loss (500 km) | -145 dB |
| RX Antenna Gain | 12 dBi (typical ground) |
| Received Power | -106 dBm |
| Receiver Sensitivity | -120 dBm |
| Link Margin | 14 dB |

## 5. Operational Plan

### 5.1 Beacon Schedule

| Phase | Interval | Content |
|-------|----------|---------|
| LEOP (first 2 weeks) | 60 seconds | Basic telemetry |
| Normal Operations | 60 seconds | AI-generated + telemetry |
| Low Power Mode | 120 seconds | Basic telemetry only |
| Ground Pass | 30 seconds | Enhanced telemetry |

### 5.2 Beacon Content

```
SMART-QSO Beacon Frame (AX.25 UI Frame)
├── Destination: CQ
├── Source: [CALLSIGN]-1
├── Control: UI (0x03)
├── PID: No L3 (0xF0)
└── Information:
    ├── Timestamp (UTC)
    ├── Telemetry (voltage, current, temperature)
    ├── ADCS state
    ├── AI-generated text (when available)
    └── CRC
```

### 5.3 Sample Beacon

```
SMART-QSO de [CALLSIGN]
UTC: 2026-01-15T14:32:00Z
V:7.4 I:0.3 T:25C SOC:85%
ADCS:SUN_POINT ERR:2.5deg
AI: Greetings from orbit! Solar panels
catching rays over the Pacific.
73 de SMART-QSO
```

## 6. Interference Mitigation

### 6.1 Measures Implemented

| Measure | Implementation |
|---------|----------------|
| Low duty cycle | < 10% transmission time |
| Low power | 0.5W nominal |
| Accurate frequency | ± 1 kHz stability |
| Standard protocol | AX.25 UI frames |
| Ground override | Transmitter can be disabled |

### 6.2 Coordination with Other Satellites

SMART-QSO will coordinate with:
- ISS amateur radio (ARISS)
- Other CubeSats on 145.825 MHz
- Terrestrial repeater coordination

## 7. Ground Station

### 7.1 Primary Ground Station

| Parameter | Value |
|-----------|-------|
| Location | NASA Ames Research Center |
| Coordinates | 37.4°N, 122.1°W |
| Callsign | [TBD - Licensed amateur] |
| Antenna | Yagi, 12 dBi |
| Receiver | SDR-based |

### 7.2 Network Stations

- SatNOGS global network
- AMSAT member stations
- Educational institution partners

## 8. Licensing

### 8.1 US Licensing

| License | Status |
|---------|--------|
| FCC Part 97 | Application pending |
| NTIA (federal) | Coordination in progress |
| ITU Filing | Via FCC |

### 8.2 Responsible Operators

| Role | Callsign | License Class |
|------|----------|---------------|
| Primary Operator | TBD | Extra |
| Backup Operator | TBD | Extra |
| Mission Director | TBD | Extra |

## 9. Contact Information

### 9.1 Technical Contact

| Field | Information |
|-------|-------------|
| Name | [Project Engineer] |
| Organization | NASA Ames Research Center |
| Email | [email]@nasa.gov |
| Phone | [phone] |

### 9.2 IARU Coordinator

| Region | Coordinator |
|--------|-------------|
| IARU Region 2 | AMSAT-NA Frequency Coordination |

## 10. Timeline

| Milestone | Date |
|-----------|------|
| IARU Application Submitted | TBD |
| Coordination Response | TBD + 60 days |
| FCC License Application | TBD |
| FCC License Grant | TBD |
| Launch | TBD |
| First Beacon | Launch + 30 minutes |

## 11. Attachments

- A: Detailed RF specifications
- B: Antenna radiation pattern
- C: Orbital parameters
- D: Ground station details

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
