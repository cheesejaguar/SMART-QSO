# Ground Station Design
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-GND-001
**Revision:** A
**Date:** 2026-01-02

---

## 1. Overview

This document defines the ground station architecture for the SMART-QSO CubeSat mission, including primary station design, SatNOGS network integration, and mission control software.

## 2. Ground Segment Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     SMART-QSO Ground Segment                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │   Primary   │    │   SatNOGS   │    │   Mission Control   │ │
│  │   Station   │    │   Network   │    │      Center         │ │
│  └──────┬──────┘    └──────┬──────┘    └──────────┬──────────┘ │
│         │                  │                       │            │
│         └──────────────────┴───────────────────────┘            │
│                            │                                     │
│                     ┌──────┴──────┐                             │
│                     │   Database   │                             │
│                     │   Archive    │                             │
│                     └─────────────┘                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 3. Primary Ground Station

### 3.1 Station Specifications

| Parameter | Specification |
|-----------|---------------|
| Location | TBD (targeting mid-latitude site) |
| Coordinates | TBD |
| Antenna Type | Cross-Yagi or Eggbeater |
| Antenna Gain | > 10 dBi |
| Frequency Range | 144-148 MHz (2m) |
| Polarization | Circular (RHCP) |
| Tracking | Az/El rotator with tracking software |

### 3.2 Receive Chain

```
Antenna → LNA → Bandpass Filter → SDR → Computer → Decoder
```

| Component | Specification |
|-----------|---------------|
| LNA | 0.5 dB NF, 20 dB gain, mast-mounted |
| Bandpass Filter | 144-148 MHz, 3 dB bandwidth |
| Coax Cable | LMR-400 or equivalent, < 2 dB loss |
| SDR | RTL-SDR V3 or better (min 8-bit ADC) |
| Alternative | Airspy R2, SDRPlay RSPdx |

### 3.3 Receive System Performance

| Parameter | Value |
|-----------|-------|
| System Noise Temperature | 290 K (estimated) |
| G/T | -15 dB/K (estimated) |
| Sensitivity | -120 dBm for 1200 bps AFSK |
| Link margin at 500 km | > 10 dB |

### 3.4 Antenna System

| Component | Specification |
|-----------|---------------|
| Primary Antenna | 2x10 element cross-Yagi or M2 Eggbeater |
| Rotator | Yaesu G-5500 or equivalent |
| Controller | AMSAT rotator controller or computer interface |
| Tracking Software | Gpredict, SatPC32, or rotctl |

### 3.5 Station Software

| Function | Software |
|----------|----------|
| SDR Interface | GNU Radio, SDR# |
| Audio Routing | Virtual Audio Cable (VAC) |
| TNC/Decoder | Dire Wolf, UZ7HO |
| Tracking | Gpredict |
| Pass Scheduling | Custom script |
| Data Logging | Custom Python application |

## 4. SatNOGS Integration

### 4.1 Overview

SatNOGS is a global network of open-source ground stations that will provide additional receive coverage for SMART-QSO.

### 4.2 Configuration

| Parameter | Value |
|-----------|-------|
| NORAD ID | TBD (after launch) |
| Transmitter Entry | To be added to SatNOGS DB |
| Mode | AFSK 1200 bps AX.25 |
| Frequency | 145.825 MHz (TBD) |

### 4.3 Data Flow

1. SatNOGS stations automatically schedule observations
2. Raw audio captured and uploaded
3. Demodulated data extracted
4. Data available via SatNOGS API
5. SMART-QSO MCC polls API for new data

### 4.4 Expected Coverage

| Region | Stations | Passes/Day |
|--------|----------|------------|
| North America | 50+ | 6-10 |
| Europe | 100+ | 8-12 |
| Asia | 20+ | 4-6 |
| Australia | 10+ | 4-6 |
| **Total** | **180+** | **22-34** |

## 5. Mission Control Center

### 5.1 Hardware

| Component | Specification |
|-----------|---------------|
| Server | Linux server, 4+ cores, 16GB RAM |
| Storage | 1TB SSD + 4TB HDD archive |
| Network | Reliable internet connection |
| UPS | 30-minute backup power |

### 5.2 Software Architecture

```
┌─────────────────────────────────────────────┐
│              Mission Control Center          │
├─────────────────────────────────────────────┤
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Ingest  │  │ Database │  │   Web    │  │
│  │ Service  │──│ (Postgres)│──│Dashboard │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│       │                            │         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Decoder  │  │ Analysis │  │ Alerting │  │
│  │ Service  │  │  Tools   │  │ Service  │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│                                              │
└─────────────────────────────────────────────┘
```

### 5.3 Ingest Service

Collects data from:
- Primary ground station (direct)
- SatNOGS network (API)
- Community stations (manual upload)

### 5.4 Database Schema

```sql
-- Beacons table
CREATE TABLE beacons (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    station_id VARCHAR(32),
    raw_frame BYTEA,
    decoded_text TEXT,
    callsign VARCHAR(16),
    telemetry JSONB,
    source VARCHAR(16)
);

-- Telemetry table
CREATE TABLE telemetry (
    id SERIAL PRIMARY KEY,
    beacon_id INTEGER REFERENCES beacons(id),
    timestamp TIMESTAMP NOT NULL,
    battery_voltage DECIMAL(4,2),
    solar_current DECIMAL(4,2),
    power_mode INTEGER,
    obc_temp DECIMAL(4,1),
    battery_temp DECIMAL(4,1),
    adcs_mode INTEGER,
    reset_count INTEGER,
    beacon_count INTEGER
);
```

### 5.5 Web Dashboard

| Page | Content |
|------|---------|
| Status | Current spacecraft state, last beacon |
| Telemetry | Real-time and historical plots |
| Beacons | Recent beacon text, AI vs template |
| Passes | Upcoming passes, scheduling |
| Archive | Historical data search |
| Admin | System configuration |

### 5.6 Alerting

| Condition | Alert Type | Recipients |
|-----------|------------|------------|
| No beacon > 2 hours | Email | Operations |
| Low battery | Email + SMS | Operations |
| Temperature limit | Email + SMS | Operations |
| Safe mode entry | Email | All team |
| Anomaly detected | Email + SMS | On-call |

## 6. Link Budget Analysis

### 6.1 Downlink Budget (Beacon)

| Parameter | Value | Notes |
|-----------|-------|-------|
| TX Power | 27 dBm (0.5W) | Low power mode |
| TX Antenna Gain | 0 dBi | Quarter-wave monopole |
| EIRP | 27 dBm | |
| Path Loss (500 km) | 142 dB | At 146 MHz |
| Polarization Loss | 3 dB | Circular-to-linear |
| RX Antenna Gain | 10 dBi | Cross-Yagi |
| System Noise | -174 dBm/Hz | Thermal |
| Noise Figure | 2 dB | LNA |
| Bandwidth | 30 dB-Hz | 1 kHz |
| Received Power | -108 dBm | |
| Required SNR | 10 dB | For 1200 bps AFSK |
| **Link Margin** | **14 dB** | |

### 6.2 Link Availability

| Condition | Margin | Status |
|-----------|--------|--------|
| Clear sky, 30° elevation | 14 dB | Excellent |
| Clear sky, 10° elevation | 8 dB | Good |
| Rain fade (rare at VHF) | 13 dB | Good |
| Interference | Variable | Monitor |

## 7. Pass Operations

### 7.1 Pass Prediction

| Parameter | Source |
|-----------|--------|
| TLE | CelesTrak, Space Track |
| Update frequency | Daily |
| Prediction horizon | 7 days |
| Minimum elevation | 10° |

### 7.2 Pass Execution

1. **T-30 min:** Verify TLE updated
2. **T-10 min:** Start tracking software
3. **T-5 min:** Verify antenna tracking
4. **AOS:** Start recording, enable decoder
5. **Pass:** Monitor beacon reception
6. **LOS:** Stop recording
7. **Post-pass:** Upload data, verify decode

### 7.3 Data Handling

| Step | Action |
|------|--------|
| 1 | Raw audio archived |
| 2 | Beacons decoded |
| 3 | Telemetry extracted |
| 4 | Data uploaded to database |
| 5 | Alerts generated if needed |
| 6 | Dashboard updated |

## 8. Data Products

### 8.1 Real-Time Products

| Product | Format | Update Rate |
|---------|--------|-------------|
| Current status | JSON | Per beacon |
| Telemetry stream | JSON | Per beacon |
| Beacon text | Text | Per beacon |

### 8.2 Archived Products

| Product | Format | Retention |
|---------|--------|-----------|
| Raw audio | WAV | 90 days |
| Decoded frames | JSON | Permanent |
| Telemetry | CSV/JSON | Permanent |
| TLE history | TLE | Permanent |

### 8.3 Public Products

| Product | Access | Update |
|---------|--------|--------|
| Beacon text | Web | Real-time |
| Status page | Web | Real-time |
| Telemetry plots | Web | Real-time |
| TLE | CelesTrak | Daily |

## 9. Network Diagram

```
Internet
    │
    ├─── Primary Station ───┐
    │    (Direct connection) │
    │                        ▼
    ├─── SatNOGS API ──────► MCC Server ──► PostgreSQL
    │                        │              Database
    │                        ▼
    └─── Community ────────► Web Dashboard ──► Users
         Stations            │
                             ▼
                         Alert System
                             │
                             ▼
                      Operations Team
```

## 10. Maintenance

### 10.1 Regular Maintenance

| Task | Frequency |
|------|-----------|
| TLE update | Daily |
| Database backup | Daily |
| Log rotation | Weekly |
| Software updates | Monthly |
| Antenna inspection | Quarterly |
| Full system test | Quarterly |

### 10.2 Troubleshooting

| Symptom | Check |
|---------|-------|
| No beacon received | Antenna, LNA, SDR, tracking |
| Weak signal | LNA power, cable connections |
| Decoder errors | SDR sample rate, audio levels |
| Database issues | Disk space, connections |
| Dashboard down | Web server, network |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
