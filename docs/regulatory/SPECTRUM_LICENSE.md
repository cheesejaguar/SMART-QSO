# Spectrum Authorization Documentation
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-002
**Revision:** A
**Date:** 2026-01-02

---

## 1. Purpose

This document summarizes the spectrum authorization requirements and status for the SMART-QSO CubeSat mission, covering both amateur and government frequency usage.

## 2. Frequency Summary

### 2.1 Frequency Allocations

| Use | Frequency | Band | Allocation | Authorization |
|-----|-----------|------|------------|---------------|
| Beacon Downlink | 145.825 MHz | VHF | Amateur Satellite | FCC Part 97 |
| Backup Downlink | 145.900 MHz | VHF | Amateur Satellite | FCC Part 97 |
| TT&C Uplink | 449.x MHz | UHF | Government | NTIA |
| TT&C Downlink | 401.x MHz | UHF | Government | NTIA |

### 2.2 Power and Bandwidth

| Frequency | Power (EIRP) | Bandwidth | Modulation |
|-----------|--------------|-----------|------------|
| 145.825 MHz | 27 dBm | 15 kHz | FM/AFSK |
| 449.x MHz | 30 dBm | 25 kHz | FSK |
| 401.x MHz | 30 dBm | 25 kHz | FSK |

## 3. Amateur Radio Authorization

### 3.1 FCC Part 97 Requirements

| Requirement | Compliance |
|-------------|------------|
| Licensed operator | Required for all transmissions |
| Callsign identification | Every 10 minutes and at end |
| No encryption | All transmissions in clear |
| Non-pecuniary interest | NASA research mission |
| Technical standards | Per §97.307 |

### 3.2 Coordination Status

| Organization | Status | Date |
|--------------|--------|------|
| IARU Region 2 | Application submitted | TBD |
| AMSAT-NA | Coordination requested | TBD |
| FCC | License pending | TBD |

### 3.3 Callsign

| Item | Value |
|------|-------|
| Requested Callsign | TBD |
| Backup Callsign | TBD |
| SSID | -1 (satellite) |

## 4. Government Frequency Authorization

### 4.1 NTIA Process

| Step | Description | Status |
|------|-------------|--------|
| 1 | Agency frequency request | In progress |
| 2 | NTIA SPS certification | Pending |
| 3 | Frequency assignment | Pending |
| 4 | ITU filing | Pending |

### 4.2 NASA Frequency Management

| Document | Status |
|----------|--------|
| NASA Form 1722 | Submitted |
| RF Safety Analysis | Complete |
| EMC Analysis | Complete |
| Spectrum Support | Requested |

## 5. ITU Filing Requirements

### 5.1 API/CR Filing

| Item | Value |
|------|-------|
| Administration | USA |
| Network Name | SMART-QSO |
| Filing Type | Non-GSO satellite |
| Service | Amateur-satellite |

### 5.2 Orbital Parameters for Filing

| Parameter | Value |
|-----------|-------|
| Apogee | 500 km |
| Perigee | 500 km |
| Inclination | 51.6° |
| Period | 94.5 min |
| Number of satellites | 1 |

## 6. Interference Analysis

### 6.1 Amateur Band Analysis

| Potential Interference | Mitigation |
|-----------------------|------------|
| Terrestrial repeaters | Low power, high altitude |
| Other amateur satellites | Time/frequency coordination |
| ISS amateur radio | Different frequencies |

### 6.2 Government Band Analysis

| Analysis | Result |
|----------|--------|
| SPS analysis | No interference predicted |
| Coordination with DoD | Complete |
| Coordination with NOAA | Complete |

## 7. RF Safety

### 7.1 Exposure Analysis

| Scenario | Power Density | Limit | Compliant |
|---------|---------------|-------|-----------|
| Ground (nadir) | < 0.001 mW/cm² | 0.2 mW/cm² | Yes |
| ISS proximity | < 0.01 mW/cm² | 0.2 mW/cm² | Yes |
| Aircraft | < 0.0001 mW/cm² | 0.2 mW/cm² | Yes |

### 7.2 Spacecraft RF Safety

| Consideration | Mitigation |
|---------------|------------|
| On-orbit RF hazard | Antenna pattern, power limits |
| Ground testing | Proper shielding, anechoic chamber |
| Integration | TX disabled during handling |

## 8. Export Control

### 8.1 ITAR/EAR Assessment

| Component | Classification | Export License |
|-----------|----------------|----------------|
| RF transmitter | EAR99 | NLR |
| Software | EAR99 | NLR |
| Frequency plan | Not controlled | N/A |

### 8.2 Technology Transfer

| Item | Restriction |
|------|-------------|
| Frequency assignments | Public after launch |
| RF design details | NASA public release |
| Ground station locations | Public information |

## 9. Regulatory Compliance Checklist

### 9.1 Pre-Launch Requirements

- [ ] IARU coordination complete
- [ ] FCC Part 97 license granted
- [ ] NTIA frequency assignment
- [ ] ITU API filed
- [ ] RF safety analysis approved
- [ ] Export control review complete

### 9.2 Post-Launch Requirements

- [ ] ITU notification filed (within 30 days)
- [ ] FCC notification of operation
- [ ] NORAD TLE published
- [ ] Amateur callsign announced

## 10. Points of Contact

### 10.1 Regulatory Contacts

| Role | Organization | Contact |
|------|--------------|---------|
| NASA Spectrum Manager | NASA HQ | spectrum@nasa.gov |
| FCC Liaison | FCC | [contact] |
| IARU Coordinator | AMSAT-NA | [contact] |

### 10.2 Project Contacts

| Role | Name | Email |
|------|------|-------|
| Project Manager | TBD | TBD |
| RF Engineer | TBD | TBD |
| Licensing Lead | TBD | TBD |

## 11. Schedule

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| IARU application | L-18 months | TBD |
| IARU coordination | L-12 months | TBD |
| FCC application | L-12 months | TBD |
| FCC license | L-6 months | TBD |
| NTIA assignment | L-6 months | TBD |
| ITU API | L-3 months | TBD |
| Launch | L-0 | TBD |
| ITU notification | L+30 days | TBD |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
