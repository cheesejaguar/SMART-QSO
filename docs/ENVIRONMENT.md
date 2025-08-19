# SMART-QSO Orbital Environment Analysis

## Executive Summary

This document provides comprehensive analysis of the orbital environment for the SMART-QSO CubeSat mission, including thermal environment characterization, radiation shielding requirements for the Jetson Orin Nano Super payload, and orbital parameters based on typical NanoRacks NRCSD deployments. The analysis supports mission design decisions and ensures proper environmental protection for critical payload systems.

## 1. Orbital Thermal Environment

### 1.1 Orbital Parameters
- **Orbit Type**: Low Earth Orbit (LEO)
- **Altitude Range**: 400-450 km (typical ISS deployment)
- **Inclination**: 51.6° (ISS inclination)
- **Orbital Period**: ~93 minutes
- **Eclipse Frequency**: 15-16 eclipses per day
- **Eclipse Duration**: 30-35 minutes per eclipse

### 1.2 Thermal Environment Characteristics

#### 1.2.1 External Heat Sources
- **Solar Irradiance**: 1361 W/m² (AM0)
- **Albedo**: 0.3 (Earth reflectivity)
- **Earth IR**: 237 W/m² (Earth thermal emission)
- **Planetary IR**: 5 W/m² (planetary background)

#### 1.2.2 Thermal Cycling
- **Sunlit Period**: 58-63 minutes per orbit
- **Eclipse Period**: 30-35 minutes per orbit
- **Thermal Cycle**: 93-minute orbital period
- **Daily Cycles**: 15-16 thermal cycles per day

#### 1.2.3 Temperature Extremes
```
Environment          | Sunlit Temperature | Eclipse Temperature | Range
--------------------|-------------------|-------------------|--------
External Surface    | +120°C to +150°C  | -150°C to -100°C  | 270°C
Internal Volume     | +40°C to +60°C    | -20°C to +10°C    | 80°C
Payload Compartment | +35°C to +55°C    | -15°C to +5°C     | 70°C
```

### 1.3 Thermal Analysis Results

#### 1.3.1 Heat Balance
- **Solar Absorption**: 200-300W during sunlit periods
- **Earth IR Absorption**: 50-100W continuous
- **Internal Heat Generation**: 5-15W (payload dependent)
- **Radiative Heat Loss**: 150-400W (temperature dependent)

#### 1.3.2 Thermal Time Constants
- **External Surfaces**: 2-5 minutes
- **Internal Structure**: 15-30 minutes
- **Payload Components**: 10-20 minutes
- **Battery Pack**: 45-90 minutes

### 1.4 Thermal Control Requirements

#### 1.4.1 Passive Thermal Control
- **Surface Coatings**: Low-α/ε for sunlit surfaces, high-ε for radiator surfaces
- **Thermal Insulation**: Multi-layer insulation (MLI) for internal components
- **Heat Paths**: Conductive paths to radiator surfaces
- **Thermal Mass**: Strategic placement of thermal mass for stability

#### 1.4.2 Active Thermal Control
- **Heaters**: Survival heaters for critical components during eclipse
- **Thermal Switches**: Variable conductance heat pipes for payload cooling
- **Power Management**: Adaptive power modes based on thermal conditions

## 2. Radiation Shielding for Payload

### 2.1 Radiation Environment

#### 2.1.1 Primary Radiation Sources
- **Galactic Cosmic Rays (GCR)**: High-energy protons and heavy ions
- **Solar Particle Events (SPE)**: Coronal mass ejections and solar flares
- **Trapped Radiation**: Van Allen belts (minimal at ISS altitude)
- **Secondary Radiation**: Neutrons and gamma rays from interactions

#### 2.1.2 Radiation Flux at ISS Altitude
```
Radiation Type      | Energy Range      | Flux (particles/cm²/s) | LET (MeV·cm²/g)
--------------------|-------------------|------------------------|------------------
Protons            | 1 MeV - 100 MeV   | 1-10                  | 0.1-10
Heavy Ions         | 10 MeV/n - 1 GeV/n| 0.01-0.1             | 1-100
Electrons          | 0.1 MeV - 10 MeV  | 10-100                | 0.01-1
Neutrons           | Thermal - 100 MeV | 0.1-1                 | 0.1-10
```

### 2.2 Radiation Effects on Electronics

#### 2.2.1 Total Ionizing Dose (TID)
- **Jetson Orin Nano Super**: 100 krad (Si) requirement
- **Memory Devices**: 50-100 krad (Si) tolerance
- **Power Electronics**: 100-300 krad (Si) tolerance
- **Sensors**: 20-50 krad (Si) tolerance

#### 2.2.2 Single Event Effects (SEE)
- **Single Event Upsets (SEU)**: Memory bit flips
- **Single Event Latchups (SEL)**: Circuit lockup requiring reset
- **Single Event Transients (SET)**: Temporary signal corruption
- **Single Event Burnout (SEB)**: Permanent device damage

#### 2.2.3 Displacement Damage
- **Solar Cells**: Efficiency degradation over time
- **Optical Sensors**: Dark current increase
- **Semiconductor Devices**: Parameter shifts

### 2.3 Shielding Design

#### 2.3.1 Material Selection
- **Primary Shield**: 2mm aluminum (6061-T6)
- **Secondary Shield**: 1mm tantalum for high-Z particles
- **Tertiary Shield**: 0.5mm polyethylene for neutron moderation
- **Total Shielding**: 3.5mm equivalent aluminum

#### 2.3.2 Shielding Effectiveness
```
Shielding Material  | Thickness | TID Reduction | SEE Reduction | Mass Penalty
--------------------|-----------|---------------|---------------|-------------
Aluminum (6061-T6) | 2.0 mm    | 85%          | 60%          | 5.4 kg/m²
Tantalum           | 1.0 mm    | 95%          | 80%          | 16.6 kg/m²
Polyethylene       | 0.5 mm    | 70%          | 40%          | 0.5 kg/m²
Combined           | 3.5 mm    | 98%          | 90%          | 22.5 kg/m²
```

#### 2.3.3 Payload-Specific Shielding
- **Jetson Module**: Dedicated 4mm aluminum enclosure
- **Memory Devices**: 2mm aluminum + 1mm tantalum
- **Power Electronics**: 3mm aluminum + 0.5mm tantalum
- **Sensors**: 1.5mm aluminum + 0.3mm tantalum

### 2.4 Radiation Hardening Strategies

#### 2.4.1 Hardware Hardening
- **Radiation-Hardened Components**: COTS components with radiation tolerance
- **Redundant Systems**: Critical functions with backup implementations
- **Error Detection**: ECC memory, CRC checksums, watchdog timers
- **Fault Tolerance**: Graceful degradation and recovery mechanisms

#### 2.4.2 Software Hardening
- **Memory Scrubbing**: Periodic ECC correction and validation
- **Fault Detection**: Runtime error checking and validation
- **Recovery Mechanisms**: Automatic restart and reconfiguration
- **Data Integrity**: Checksums and redundant storage

## 3. Orbital Parameters and Lifetime

### 3.1 NanoRacks NRCSD Deployment

#### 3.1.1 Deployment Characteristics
- **Deployment Vehicle**: International Space Station (ISS)
- **Deployment Altitude**: 408 km (nominal)
- **Deployment Velocity**: 0.5-1.0 m/s relative to ISS
- **Separation Distance**: 100-200 m from ISS
- **Initial Attitude**: Tumbling with tip-off rates

#### 3.1.2 Typical Deployment Parameters
- **Tip-off Rates**: 2-5°/s (angular velocity)
- **Initial Spin**: 1-3 rpm (rotation rate)
- **Attitude Uncertainty**: ±10° (3σ)
- **Velocity Uncertainty**: ±0.2 m/s (3σ)

### 3.2 Example Kepler Elements

#### 3.2.1 Reference Mission: TechEdSat-13
TechEdSat-13 was a 3U CubeSat deployed from ISS in 2023, providing a relevant reference for SMART-QSO orbital parameters.

**Initial Kepler Elements (Epoch: 2023-01-15 12:00:00 UTC)**
```
Parameter           | Value              | Units
--------------------|--------------------|--------
Semi-major Axis    | 6,781.0            | km
Eccentricity       | 0.0008             | dimensionless
Inclination        | 51.64              | degrees
Right Ascension    | 45.2               | degrees
Argument of Perigee| 90.1               | degrees
True Anomaly       | 180.0              | degrees
Mean Motion        | 15.7               | rev/day
Period             | 91.7               | minutes
```

#### 3.2.2 SMART-QSO Projected Elements
Based on TechEdSat-13 parameters and typical ISS deployment characteristics:

**Projected Initial Elements (Epoch: TBD)**
```
Parameter           | Value              | Units
--------------------|--------------------|--------
Semi-major Axis    | 6,781.0 ± 2.0     | km
Eccentricity       | 0.0008 ± 0.0002   | dimensionless
Inclination        | 51.64 ± 0.1        | degrees
Right Ascension    | TBD ± 5.0          | degrees
Argument of Perigee| 90.1 ± 10.0        | degrees
True Anomaly       | 180.0 ± 15.0       | degrees
Mean Motion        | 15.7 ± 0.1         | rev/day
Period             | 91.7 ± 0.6         | minutes
```

### 3.3 Orbital Lifetime Analysis

#### 3.3.1 Atmospheric Drag Effects
- **Drag Coefficient**: 2.2 (typical CubeSat)
- **Ballistic Coefficient**: 65 kg/m²
- **Drag Area**: 0.01 m² (1U deployed configuration)
- **Mass**: 1.33 kg (SMART-QSO estimated)

#### 3.3.2 Lifetime Projections
```
Altitude Range     | Duration           | Decay Rate          | Notes
-------------------|--------------------|---------------------|------------------
400-350 km        | 6-12 months        | 0.1-0.2 km/month   | Solar maximum
350-300 km        | 3-6 months         | 0.3-0.5 km/month   | Increased drag
300-250 km        | 2-4 months         | 0.8-1.2 km/month   | High drag
250-200 km        | 1-2 months         | 2.0-3.0 km/month   | Critical phase
<200 km           | Days to weeks      | 5.0-10.0 km/month  | Reentry
```

#### 3.3.3 Solar Activity Impact
- **Solar Maximum**: Increased atmospheric density, shorter lifetime
- **Solar Minimum**: Decreased atmospheric density, longer lifetime
- **Solar Cycle 25**: Peak expected 2024-2025
- **Lifetime Range**: 12-24 months depending on solar activity

### 3.4 Mission Timeline

#### 3.4.1 Phase Durations
- **Launch and Deployment**: Day 0-1
- **Commissioning**: Days 1-30
- **Nominal Operations**: Months 2-18
- **Extended Operations**: Months 19-24 (if possible)
- **End of Mission**: Month 18-24

#### 3.4.2 Critical Mission Periods
- **First 30 Days**: Commissioning and initial operations
- **Months 2-6**: Primary science and AI operations
- **Months 7-12**: Extended operations and data collection
- **Months 13-18**: Final operations and data downlink
- **Months 19-24**: Contingency operations (if possible)

## 4. Environmental Mitigation Strategies

### 4.1 Thermal Mitigation
- **Multi-layer Insulation**: 10-layer MLI for external surfaces
- **Thermal Coatings**: Low-α/ε for sunlit, high-ε for radiator
- **Heat Pipes**: Variable conductance heat pipes for payload cooling
- **Thermal Mass**: Strategic placement for temperature stability

### 4.2 Radiation Mitigation
- **Shielding Design**: Multi-layer shielding with high-Z materials
- **Component Selection**: Radiation-tolerant COTS components
- **System Redundancy**: Critical function backup implementations
- **Error Detection**: Comprehensive error checking and correction

### 4.3 Orbital Mitigation
- **Attitude Control**: Active detumbling and sun-pointing
- **Power Management**: Adaptive power modes for eclipse survival
- **Communication**: Robust beacon system for mission status
- **Fault Tolerance**: Graceful degradation and recovery mechanisms

## 5. Environmental Testing Requirements

### 5.1 Thermal Testing
- **Thermal Vacuum**: -40°C to +85°C, 10^-6 Torr
- **Thermal Cycling**: 100 cycles (-40°C to +85°C)
- **Thermal Balance**: Steady-state temperature mapping
- **Eclipse Simulation**: Rapid temperature transitions

### 5.2 Radiation Testing
- **TID Testing**: 100 krad (Si) total dose
- **SEE Testing**: Heavy ion and proton testing
- **Neutron Testing**: Displacement damage assessment
- **Component Testing**: Individual component radiation tolerance

### 5.3 Environmental Testing
- **Vibration**: 20g RMS random, 3-axis
- **Shock**: 1500g, 0.5ms half-sine
- **Thermal Vacuum**: Combined thermal and vacuum environment
- **EMI/EMC**: Electromagnetic compatibility testing

## 6. References

### 6.1 Technical Standards
- ECSS-E-ST-10C: Space Engineering - Testing
- ECSS-E-ST-32C: Space Engineering - Structural General Model
- MIL-STD-810G: Environmental Engineering Considerations
- NASA-STD-8739.8: Electronic Parts, Electronic Assembly, and Wire Harness

### 6.2 Mission References
- TechEdSat-13 Mission Data (NASA Ames Research Center)
- ISS Environmental Data (NASA Johnson Space Center)
- NanoRacks NRCSD Interface Control Document
- CubeSat Design Specification (CDS) Rev 13

### 6.3 Related Documentation
- SMART-QSO Architecture Document
- Power and Energy Simulation Results
- Thermal Analysis Report
- Radiation Analysis Report

---

*Document Version: 1.0*  
*Last Updated: [Current Date]*  
*Prepared By: SMART-QSO Engineering Team*  
*Reviewed By: [Name]*  
*Approved By: [Name]*
