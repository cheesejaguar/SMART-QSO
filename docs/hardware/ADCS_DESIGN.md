# Attitude Determination and Control Subsystem (ADCS) Design Document

**Document Number**: SMART-QSO-ADCS-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary Design

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- GSFC-STD-7000 (GEVS) - General Environmental Verification Standard
- ECSS-E-ST-10-03C - Testing

**Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This document defines the detailed design of the Attitude Determination and Control Subsystem (ADCS) for the SMART-QSO 1U CubeSat. It provides requirements, architecture, control algorithms, and verification approach.

### 1.2 Scope

The ADCS encompasses:
- Attitude sensors (magnetometer, sun sensors)
- Attitude actuators (magnetorquers)
- Control algorithms (B-dot detumble, sun-pointing)
- Interface to OBC for autonomous operation

### 1.3 Design Philosophy

SMART-QSO employs a passive/semi-active ADCS approach:
- **No precision pointing required**: Power-bias sun pointing adequate
- **Simple, reliable design**: No reaction wheels or thrusters
- **Magnetorquer-only actuation**: Proven CubeSat heritage
- **Autonomous operation**: AI-driven attitude management

---

## 2. ADCS Requirements

### 2.1 Functional Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| ADCS-F001 | Detumble within 1 orbit from tip-off rates ≤10°/s | Analysis, Test |
| ADCS-F002 | Achieve power-bias sun pointing (±15° accuracy) | Analysis, Test |
| ADCS-F003 | Maintain attitude knowledge within ±5° | Analysis, Test |
| ADCS-F004 | Operate autonomously without ground commands | Test |
| ADCS-F005 | Provide attitude telemetry to OBC | Test |
| ADCS-F006 | Support graceful degradation on sensor failure | Test |
| ADCS-F007 | Generate torque in 3 orthogonal axes | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| ADCS-P001 | Detumble time | <90 min | <45 min |
| ADCS-P002 | Sun-pointing accuracy | ±15° | ±10° |
| ADCS-P003 | Attitude knowledge accuracy | ±5° | ±3° |
| ADCS-P004 | Peak power consumption | <2W | <1W |
| ADCS-P005 | Steady-state power | <0.5W | <0.3W |
| ADCS-P006 | Control bandwidth | >0.01 Hz | >0.02 Hz |

### 2.3 Interface Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| ADCS-I001 | I2C interface to OBC | Address 0x1E (mag), 0x40-0x45 (sun) |
| ADCS-I002 | GPIO magnetorquer control | PWM capable, 100mA drive |
| ADCS-I003 | Attitude quaternion output | Via telemetry structure |
| ADCS-I004 | Sensor raw data access | For calibration/debugging |

### 2.4 Environmental Requirements

| Parameter | Operating | Non-Operating |
|-----------|-----------|---------------|
| Temperature | -40°C to +85°C | -55°C to +125°C |
| Magnetic Field | 20-60 μT (Earth orbit) | N/A |
| Vibration | 14.1 Grms (random) | N/A |
| Radiation | 10 krad TID | N/A |

---

## 3. ADCS Architecture

### 3.1 Block Diagram

```
                    +------------------------+
                    |     SUN SENSORS        |
                    |   (6x photodiodes)     |
                    |   +X, -X, +Y, -Y,      |
                    |   +Z, -Z faces         |
                    +-----------+------------+
                                |
        +-----------------------+------------------------+
        |                       |                        |
        v                       v                        v
+-------+-------+     +---------+---------+    +---------+---------+
| 3-AXIS        |     |    ON-BOARD       |    |   3-AXIS         |
| MAGNETOMETER  |     |    COMPUTER       |    |  MAGNETORQUERS   |
| (HMC5883L)    |     |                   |    |  (Air-core coils)|
|               |     | +---------------+ |    |                  |
| - ±8 Gauss    |     | |  B-dot        | |    | - 0.2 A·m²/axis  |
| - 0.5° acc    |---->| |  Detumble     |-+--->| - 100mA max      |
| - 12-bit ADC  |     | +---------------+ |    | - PWM driven     |
+---------------+     | +---------------+ |    +------------------+
                      | |  Sun-Pointing | |
                      | |  Control      | |
                      | +---------------+ |
                      | +---------------+ |
                      | |  Attitude     | |
                      | |  Estimator    | |
                      | +---------------+ |
                      +-------------------+
```

### 3.2 Sensor Suite

#### 3.2.1 Magnetometer

**Selected Component**: HMC5883L 3-Axis Magnetometer

| Parameter | Specification |
|-----------|---------------|
| Measurement Range | ±8 Gauss |
| Resolution | 0.73 mGauss/LSB |
| Accuracy | ±0.5° heading |
| Update Rate | Up to 75 Hz |
| Interface | I2C (0x1E) |
| Power | 2.5V-3.3V, 100μA |
| Temperature Range | -40°C to +85°C |

**Calibration**:
- Hard-iron offset: Determined during ground test
- Soft-iron correction: 3x3 calibration matrix
- Temperature compensation: Coefficients stored in EEPROM

#### 3.2.2 Sun Sensors

**Selected Component**: SLCD-61N8 Photodiode Array (6 sensors)

| Parameter | Specification |
|-----------|---------------|
| Type | Silicon photodiode |
| Active Area | 7.5 mm² |
| Spectral Range | 350-1100 nm |
| FOV | ±60° (cosine response) |
| Interface | ADC via OBC |
| Power | <1 mW per sensor |
| Accuracy | ±3° (sun vector) |

**Sun Vector Calculation**:
```c
// Normalized sun vector from 6-face photodiodes
sun_x = (I_px - I_mx) / max(I_px, I_mx);
sun_y = (I_py - I_my) / max(I_py, I_my);
sun_z = (I_pz - I_mz) / max(I_pz, I_mz);
normalize(&sun_x, &sun_y, &sun_z);
```

### 3.3 Actuators

#### 3.3.1 Magnetorquer Coils

**Configuration**: 3-axis air-core coils (PCB trace or wire-wound)

| Parameter | X-Axis | Y-Axis | Z-Axis |
|-----------|--------|--------|--------|
| Dimensions | 80×80 mm | 80×80 mm | 90×90 mm |
| Turns | 200 | 200 | 180 |
| Wire Gauge | 30 AWG | 30 AWG | 30 AWG |
| Resistance | 15 Ω | 15 Ω | 12 Ω |
| Max Current | 100 mA | 100 mA | 100 mA |
| Max Dipole | 0.20 A·m² | 0.20 A·m² | 0.22 A·m² |

**Torque Generation**:
```
τ = m × B
```
Where:
- τ = torque vector (N·m)
- m = magnetic dipole moment (A·m²)
- B = Earth's magnetic field (T)

**Maximum Torque** (at 50 μT field):
```
τ_max = 0.2 A·m² × 50×10⁻⁶ T = 10×10⁻⁶ N·m = 10 μN·m
```

---

## 4. Control Algorithms

### 4.1 B-dot Detumble Algorithm

**Purpose**: Reduce angular rates from deployment to stable rotation

**Control Law**:
```c
// B-dot detumble control
void bdot_control(Vec3 *B_measured, Vec3 *B_prev, float dt, Vec3 *m_cmd) {
    Vec3 B_dot;

    // Estimate B_dot from consecutive measurements
    B_dot.x = (B_measured->x - B_prev->x) / dt;
    B_dot.y = (B_measured->y - B_prev->y) / dt;
    B_dot.z = (B_measured->z - B_prev->z) / dt;

    // Control law: m = -k * B_dot
    float k = 5.0e6;  // Gain (A·m²·s/T)
    m_cmd->x = -k * B_dot.x;
    m_cmd->y = -k * B_dot.y;
    m_cmd->z = -k * B_dot.z;

    // Saturate to maximum dipole
    saturate_dipole(m_cmd, 0.2);  // Max 0.2 A·m²
}
```

**Convergence Analysis**:
```
Initial rates: 10°/s (worst case from deployment)
Inertia (1U): ~0.002 kg·m² (per axis)
Max torque: 10 μN·m
Time constant: τ = I*ω / T = 0.002 * 0.17 / 10e-6 = 34 s
Full detumble: ~5τ = 170 s (~3 minutes)
```

### 4.2 Sun-Pointing Control

**Purpose**: Maximize solar array illumination

**Control Law**:
```c
// Sun-pointing control using magnetorquers
void sun_pointing_control(Vec3 *sun_body, Vec3 *B_body, Vec3 *m_cmd) {
    Vec3 desired_torque;

    // Desired sun vector in body frame (maximize +X illumination)
    Vec3 sun_desired = {1.0, 0.0, 0.0};

    // Error quaternion from current to desired sun vector
    Vec3 error;
    cross_product(sun_body, &sun_desired, &error);

    // PD control
    float kp = 0.001;  // Proportional gain
    float kd = 0.0001; // Derivative gain (from gyro or B-dot estimate)

    desired_torque.x = kp * error.x;
    desired_torque.y = kp * error.y;
    desired_torque.z = kp * error.z;

    // Convert desired torque to magnetic dipole: m = (τ × B) / |B|²
    cross_product(&desired_torque, B_body, m_cmd);
    float B_mag_sq = dot_product(B_body, B_body);
    m_cmd->x /= B_mag_sq;
    m_cmd->y /= B_mag_sq;
    m_cmd->z /= B_mag_sq;

    // Saturate
    saturate_dipole(m_cmd, 0.2);
}
```

### 4.3 Attitude Estimator

**Algorithm**: Extended Kalman Filter (EKF)

**State Vector**:
```
x = [q0, q1, q2, q3, ωx, ωy, ωz]ᵀ
```
Where:
- q0, q1, q2, q3 = attitude quaternion
- ωx, ωy, ωz = angular rates (estimated)

**Measurements**:
- Magnetometer: B_body = A(q) × B_ECI
- Sun sensors: S_body = A(q) × S_ECI

**Simplified Implementation** (for 1U CubeSat):
```c
// TRIAD algorithm for attitude determination
void triad_attitude(Vec3 *sun_body, Vec3 *mag_body,
                    Vec3 *sun_eci, Vec3 *mag_eci,
                    Quaternion *q_out) {
    // Reference frame from ECI vectors
    Vec3 r1_eci, r2_eci, r3_eci;
    normalize_copy(sun_eci, &r1_eci);
    cross_product(&r1_eci, mag_eci, &r2_eci);
    normalize(&r2_eci);
    cross_product(&r1_eci, &r2_eci, &r3_eci);

    // Body frame from measured vectors
    Vec3 b1, b2, b3;
    normalize_copy(sun_body, &b1);
    cross_product(&b1, mag_body, &b2);
    normalize(&b2);
    cross_product(&b1, &b2, &b3);

    // Rotation matrix from DCM
    // A = [b1 b2 b3] × [r1 r2 r3]ᵀ
    // Convert to quaternion
    dcm_to_quaternion(&b1, &b2, &b3, &r1_eci, &r2_eci, &r3_eci, q_out);
}
```

---

## 5. Operating Modes

### 5.1 Mode Definitions

| Mode | Description | Entry Condition | Exit Condition |
|------|-------------|-----------------|----------------|
| IDLE | Sensors off, coils off | Power-on, SAFE mode | Command or timer |
| DETUMBLE | B-dot active | ω > 1°/s detected | ω < 0.5°/s for 5 min |
| SUNPOINT | Sun-pointing active | Detumble complete | Eclipse or fault |
| ECLIPSE | Coils off, drift | Sun not detected | Sun detected |
| SAFE | Minimal operation | Fault or low power | Power restored |

### 5.2 Mode State Machine

```
              +-------+
              | POWER |
              |  ON   |
              +---+---+
                  |
                  v
              +-------+
    +---------| IDLE  |<--------+
    |         +---+---+         |
    |             |             |
    | cmd/timer   |             | fault
    |             v             |
    |         +-------+         |
    |         |DETUMB-|         |
    |         |  LE   |---------+
    |         +---+---+         |
    |             |             |
    |       ω<0.5°/s            |
    |             |             |
    |             v             |
    |         +-------+  eclipse|
    +-------->|  SUN  |-------->+-------+
              | POINT |         |ECLIPSE|
              +---+---+<--------+-------+
                  |         sun detected
                  |
              fault|
                  v
              +-------+
              | SAFE  |
              +-------+
```

### 5.3 Timing Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Control loop rate | 1 Hz | Magnetorquer update rate |
| Sensor sampling | 10 Hz | Magnetometer/sun sensor |
| Detumble timeout | 90 min | Max detumble duration |
| Settling time | 5 min | Time to declare detumble complete |
| Eclipse threshold | 5% | Sun sensor threshold for eclipse |

---

## 6. Hardware Design

### 6.1 Magnetorquer Driver

```
                 +3.3V
                   |
              +----+----+
              |  L293D  |
              | H-Bridge|
        PWM_X-|1A    1Y |--+
              |         |  |
              |2A    2Y |--+---> Coil_X (15Ω)
              |         |  |
        DIR_X-|1,2EN    |--+
              +---------+
                   |
                  GND
```

**Control Interface**:
- PWM frequency: 20 kHz (inaudible)
- PWM resolution: 8-bit (256 levels)
- Direction: GPIO for polarity control
- Enable: GPIO for on/off control

**Current Limiting**:
- Software: PWM duty cycle limit
- Hardware: Series resistor backup

### 6.2 Magnetometer Interface

```
      +3.3V          OBC
        |        +---------+
    +---+---+    |         |
    |HMC5883|    | I2C     |
    |   L   |====| Master  |
    |       |    |         |
    +---+---+    | GPIO    |====> DRDY (optional)
        |        +---------+
       GND
```

### 6.3 Sun Sensor Interface

```
                +3.3V
                  |
    Photodiode   +-+
        |        | | 10kΩ
        +--------|<|---+---> To ADC
        |        | |   |
       ===       +-+  ===
        |              |
       GND            GND
```

**ADC Requirements**:
- 6 channels (one per face)
- 12-bit resolution
- 1 kHz sample rate per channel
- Simultaneous sampling preferred

---

## 7. Telemetry

### 7.1 ADCS Telemetry Points

| Parameter | Size | Resolution | Update Rate |
|-----------|------|------------|-------------|
| Quaternion (q0-q3) | 8 bytes | 16-bit fixed | 1 Hz |
| Angular rates (ω) | 6 bytes | 0.01°/s | 1 Hz |
| Sun vector | 6 bytes | 0.01 unit | 1 Hz |
| Mag field (body) | 6 bytes | 0.1 μT | 10 Hz |
| Sun sensor raw | 12 bytes | 12-bit ADC | 10 Hz |
| Control mode | 1 byte | Enum | 1 Hz |
| Coil currents | 6 bytes | 1 mA | 1 Hz |
| Eclipse flag | 1 byte | Boolean | 1 Hz |

### 7.2 Telemetry Structure

```c
typedef struct {
    float quaternion[4];    // Attitude quaternion
    float omega[3];         // Angular rates (rad/s)
    float sun_vector[3];    // Sun unit vector (body)
    float mag_field[3];     // Magnetic field (μT)
    uint16_t sun_raw[6];    // Sun sensor ADC values
    uint8_t mode;           // ADCS mode
    int16_t coil_current[3];// Magnetorquer current (mA)
    uint8_t eclipse;        // Eclipse flag
    uint8_t fault_status;   // Fault bitmask
} AdcsTelemetry_t;
```

---

## 8. Verification

### 8.1 Verification Matrix

| Requirement | Method | Status |
|-------------|--------|--------|
| ADCS-F001 | Analysis, HILS Test | Pending |
| ADCS-F002 | Analysis, HILS Test | Pending |
| ADCS-F003 | Analysis, Test | Pending |
| ADCS-F004 | Demonstration | Pending |
| ADCS-F005 | Inspection, Test | Pending |
| ADCS-F006 | Test | Pending |
| ADCS-F007 | Test | Pending |

### 8.2 Test Procedures

| Test | Description | Equipment |
|------|-------------|-----------|
| TP-ADCS-001 | Magnetorquer dipole moment | Helmholtz coils |
| TP-ADCS-002 | Magnetometer calibration | Helmholtz coils |
| TP-ADCS-003 | Sun sensor calibration | Solar simulator, rotary table |
| TP-ADCS-004 | B-dot detumble simulation | Air bearing table |
| TP-ADCS-005 | Sun-pointing simulation | Air bearing + lamp |
| TP-ADCS-006 | Algorithm validation | HILS (MATLAB/Simulink) |

### 8.3 Hardware-in-Loop Simulation

**HILS Configuration**:
```
+-------------+     +-------------+     +-------------+
|   MATLAB    |     |    OBC      |     |  ADCS HW    |
|  Simulink   |<--->|   Flight    |<--->| Magnetorquer|
| Environment |     |   Software  |     | Mag, Sun    |
+-------------+     +-------------+     +-------------+
       |
       v
+-------------+
| Orbit/Att   |
| Dynamics    |
| Magnetic    |
| Field Model |
+-------------+
```

---

## 9. Risk Assessment

### 9.1 ADCS-Specific Risks

| Risk ID | Description | L | C | Mitigation |
|---------|-------------|---|---|------------|
| ADCS-R001 | Magnetometer saturation near poles | L | L | Software limits |
| ADCS-R002 | Sun sensor degradation | M | L | Redundant sensors |
| ADCS-R003 | Magnetorquer coil failure | L | M | Conservative design |
| ADCS-R004 | Algorithm divergence | L | M | Robust algorithms |
| ADCS-R005 | Magnetic interference from payload | M | M | Careful placement |

---

## 10. COTS Alternative

### 10.1 GomSpace NanoMind A3200

If schedule or risk warrants, a COTS integrated solution is available:

| Parameter | GomSpace NanoMind A3200 |
|-----------|------------------------|
| Mass | 85g |
| Power | 0.8W peak, 0.2W idle |
| Magnetorquer | 0.2 A·m² per axis |
| Magnetometer | 3-axis, ±8 Gauss |
| Sun Sensors | 6-face coverage |
| Interface | I2C, UART, SPI |
| Flight Heritage | >100 missions |
| Cost | $15-25K |

**Trade-off**: COTS solution provides proven heritage but at higher cost and reduced customization.

---

## Appendix A: Coordinate Frames

### A.1 Body Frame

- Origin: Spacecraft center of mass
- +X: Solar array normal (sun-pointing axis)
- +Y: Along spacecraft transverse axis
- +Z: Completes right-hand system

### A.2 ECI Frame (J2000)

- Origin: Earth center
- +X: Vernal equinox
- +Z: North pole
- +Y: Completes right-hand system

### A.3 Orbital Frame (LVLH)

- Origin: Spacecraft center of mass
- +X: Velocity direction
- +Z: Nadir
- +Y: Completes right-hand system (orbit normal)

---

## Appendix B: Magnetic Field Model

**IGRF-13 Implementation**:
```c
// Simplified dipole model for LEO
void mag_field_eci(double jd, Vec3 *r_eci, Vec3 *B_eci) {
    // Earth's magnetic moment: 7.94×10²² A·m²
    const double M = 7.94e22;
    const double mu0_4pi = 1.0e-7;

    double r = vec3_magnitude(r_eci);
    double r3 = r * r * r;
    double r5 = r3 * r * r;

    // Dipole field (simplified, tilted 11° from rotation axis)
    // Full IGRF-13 coefficients for flight code
    B_eci->x = 3 * mu0_4pi * M * r_eci->x * r_eci->z / r5;
    B_eci->y = 3 * mu0_4pi * M * r_eci->y * r_eci->z / r5;
    B_eci->z = mu0_4pi * M * (3 * r_eci->z * r_eci->z - r * r) / r5;
}
```

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
