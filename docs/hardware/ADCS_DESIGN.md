# Attitude Determination and Control Subsystem (ADCS) Design Document

**Document Number**: SMART-QSO-ADCS-001
**Version**: 1.7
**Date**: 2026-10-27
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
| ADCS-F001 | Detumble within 3 orbits from tip-off rates ≤10°/s (BASELINE §8) | Analysis, Test |
| ADCS-F002 | Achieve spin-stabilized coarse sun pointing of +Y, the deployed array's normal (±15° accuracy in sunlight) | Analysis, Test |
| ADCS-F003 | Maintain attitude knowledge within ±5° | Analysis, Test |
| ADCS-F004 | Operate autonomously without ground commands | Test |
| ADCS-F005 | Provide attitude telemetry to OBC | Test |
| ADCS-F006 | Support graceful degradation on sensor failure | Test |
| ADCS-F007 | Generate torque in 3 orthogonal axes | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| ADCS-P001 | Detumble time | < 3 orbits (285 min) | < 2 orbits |
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
| Vibration | ≥ 5.57 Grms random, MPE (interim: SpaceX Rideshare User's Guide v11; `docs/test_procedures/TP-ENV-002_Vibration.md` §4.1) | N/A |
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

**CDR status (1.5): not selected.** The HMC5883L below is a placeholder: a commercial compass part with no radiation
data, and `docs/interfaces/ICD_OBC_ADCS.md` names a different one (LIS3MDL). The candidates are the flight units of
`docs/hardware/BUILD_VS_BUY.md` §3 (GomSpace M315, AAC MM200, Spacemanic Lodestone). No driver exists yet; the flight
software takes the field through `adcs_control` from the simulated HAL. Selection, mounting away from the coils and the
Jetson, and the driver are open (RID GNC-04, closure at the EM). The timing in §11.1 applies to any of them.

**Placeholder Component**: HMC5883L 3-Axis Magnetometer

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

**CDR status (1.5):** the structures review chose CubeSpace CR0002 rods in layout B (STR-01; `docs/environment/MASS_BUDGET.md`
v3.0 §6). The air-core coil data below remain as the alternative. The rods' remanence and hysteresis are not in the
Monte Carlo (§11.5).

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
    float k = 1.0e4;  // Gain (A·m²·s/T): ADCS_BDOT_GAIN, sqso_fsw::adcs::BDOT_GAIN
    m_cmd->x = -k * B_dot.x;
    m_cmd->y = -k * B_dot.y;
    m_cmd->z = -k * B_dot.z;

    // Saturate to maximum dipole
    saturate_dipole(m_cmd, 0.2);  // Max 0.2 A·m²
}
```

**Gain (1.4, 2026-10-26).** B-dot removes only the rate across the field, and only as fast as the field turns in the
body. The gain that detumbles fastest is about k* = 2n(1 + sin ξ) J_min / |B|², with n the orbit rate, ξ the
inclination and J_min the minor moment (Avanzini and Giulietti, *J. Guid. Control Dyn.* 35(4), 2012). For
J_min = 0.0029 kg·m², 500 km and |B| of 20–50 µT that is about 10⁴. The earlier 5·10⁶ is 600 times higher. A gain that
high cancels the rate across the field as soon as it appears, so the momentum follows the turning field and its
magnitude decays only slowly. The Monte Carlo of the flight code (`sqso_adcs_mc`, `docs/hardware/BUILD_VS_BUY.md` §6),
40 trials from the dispenser tip-off distribution, gave:

| Gain (A·m²·s/T) | Not detumbled after 2 days | Median | Worst | Trials with a detumble timeout |
|-----------------|----------------------------|--------|-------|--------------------------------|
| 5·10⁶ | 15 of 40 | 16 h | 35 h | 36 of 40 |
| 1·10⁴ | 0 of 40 | 0.75 h | 1.4 h (from up to 30°/s) | 0 of 40 |

**Convergence** (replacing the earlier single-axis estimate, which assumed the full 10 µN·m about every axis): with
k = 10⁴, the slowest of the trials took 1.4 h, under one orbit, to bring the rate across +Y below 0.5°/s. The
3-orbit requirement (ADCS-F001) holds with a margin of 3.

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

### 4.2.1 Spin-Stabilized Sun Pointing (flight law, 2026-10-26)

The law above points +X and has no rate damping; the deployed wings lie in the +Y plane
(`hardware/eps/SOLAR_ARRAY_DESIGN.md` §8.1), and a proportional law on magnetorquers alone does not converge. The
mission simulation (`docs/test/SIL_MISSION_SIMULATION.md`) found both. SUNPOINT now runs the baseline's
spin-stabilized coarse sun pointing (BASELINE §8, `sqso_fsw::adcs::spin_sunpoint`):

```
h = I ω                         (I: full inertia tensor with the wings out, §11.2)
τ = -k_h (h - h_s ŝ)            (sunlit only: momentum to h_s = ω_s âᵀIâ along the sun vector ŝ)
    - k_n (ω - (ω·â) â)         (nutation damping: no rate across the spin axis â, +Y in flight)
    - k_s (â·h - h_s) â         (spin hold, sunlit and in eclipse)
m = (B × τ) / |B|²              (the part of τ across B), saturated at 0.2 A·m²
```

with ω_s = 8°/s, k_h = 0.005 s⁻¹, k_n = 3·10⁻⁴ N·m·s with the gyro (1·10⁻⁴ without it) and k_s = 0.005 s⁻¹
(`sqso_fsw::adcs::Gains::FLIGHT`; changed in 1.5 from 6°/s and 1·10⁻⁴, §11.4). The rest
state is a spin about +Y with +Y on the Sun; the spin holds the axis through eclipse with only the nutation damping
and the spin hold acting. The rate is the MEMS gyro's when it reads (BASELINE §8), otherwise the magnetometer estimate
of §4.3.

**Changes in 1.4, from the Monte Carlo of the flight code** (`docs/hardware/BUILD_VS_BUY.md` §6; 40 trials of 2 days
each, B-dot at 10⁴, the BMI088 gyro model):

- *Spin hold.* The momentum law alone has a second rest state: −Y on the Sun with the spin reversed, since h = −h_s(−ŝ)
  satisfies h = h_s ŝ. Without the spin-hold term 23 of 40 trials settled there, with the wings facing away from
  the Sun. The term holds the spin positive about +Y. With it, no trial did.
- *Spin rate.* The gyroscopic stiffness against the residual-dipole, gravity-gradient and aerodynamic torques grows
  with the spin. The time-averaged angle between +Y and the Sun in sunlight was:

  | Spin | Mean | Worst trial's 95th percentile | Time within 15° | Worst at eclipse exit |
  |------|------|-------------------------------|-----------------|------------------------|
  | 2°/s (k_n 4·10⁻⁵, the 1.3 gains) | 12° | 110° | 75 % | 163° |
  | 4°/s | 3.9° | 15° | 99.8 % | 19° |
  | **6°/s** | **2.5°** | **9.8°** | **100 %** | **12°** |

  6°/s also tolerates a 10 mA·m² residual dipole, three times the 3 mA·m² design value (mean 7.1°, 92 % within 15°).
  It needs < 5 mW of coil power once captured. At 6°/s the coarse sun sensors, read at 1 Hz, see the Sun move 6° between
  samples. That is within the law's tolerance, because the law uses only the Sun direction, not its rate.
- *Nutation gain.* With k_h = 0.005, a nutation gain much below 10⁻⁴ lets the nutation grow; 4·10⁻⁵ was marginal.
- *Capture.* With a gyro, the detumble is complete when the rate across +Y is below 0.5°/s and the spin about +Y is
  below twice ω_s, held for 5 min. A spin about +Y is the wanted state, not a tumble. Without a gyro, the earlier
  |ω| < 0.5°/s criterion still applies. After a single-event reset, a spacecraft that is already spinning about +Y
  completes the detumble in 5 min instead of despinning and spinning up again. The proportional law above remains as `adcs_compute_sunpoint()` for its interface.

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
| DETUMBLE | B-dot active | ω > 1°/s detected | Rate across +Y < 0.5°/s and spin about +Y < 2 ω_s, for 5 min (with a gyro; §4.2.1); without one, ω < 0.5°/s for 5 min |
| SUNPOINT | Spin-stabilized sun pointing of +Y (§4.2.1) | Detumble complete | Eclipse or fault |
| ECLIPSE | Coils off, the spin holds the axis | Sun not detected | Sun detected |
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
| Detumble timeout | 285 min | Max detumble duration (3 orbits, BASELINE §8). In the Monte Carlo, the slowest trial took 1.5 h from tip-offs of up to 30°/s (§4.1) |
| Settling time | 5 min | Time to declare detumble complete |
| Eclipse threshold | 5% | Sun sensor threshold for eclipse |

---

## 6. Hardware Design

### 6.1 Magnetorquer Driver

**CDR finding (1.5):** the L293D drawn below needs a logic supply of at least 4.5 V and drops about 1.4 V per bridge;
it cannot run from the 3.3 V rail shown. The electrical design (`hardware/adcs/ADCS_ELECTRICAL_DESIGN.md`) names the
DRV8833 (2.7–10.8 V, low-R_DS(on) MOSFET bridges), which is the part to carry. The driver must also switch the coils
off by its own timer at the end of each 900 ms burst (§11.1); no firmware driver for the coil PWM exists yet (RID
AV-03).

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
| TP-ADCS-002 | Magnetorquer dipole moment | Helmholtz coils |
| TP-ADCS-001 | Magnetometer calibration | Helmholtz coils |
| TP-ADCS-001 | Sun sensor calibration | Solar simulator, rotary table |
| TP-ADCS-003 | B-dot detumble simulation | Air bearing table |
| TP-ADCS-003 | Sun-pointing simulation | Air bearing + lamp |
| TP-ADCS-003 | Algorithm validation | HILS (MATLAB/Simulink) |

IDs are those of the procedure files in `docs/test_procedures/` that cover each test (aligned 2026-10-16, finding VV-18;
this table had numbered them TP-ADCS-001 to -006 in a different order).

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

## 11. CDR Update (2026-10-27)

### 11.1 Magnetometer Measurement Window

The Monte Carlo models the coils' field at the magnetometer as 1·10⁻⁴ T per A·m²: a 0.2 A·m² command gives 20 µT
there, about half the Earth's field. The real coupling depends on the placement and is measured on the EM. The flight
software therefore samples with the coils off (`sqso_fsw::adcs::CoilTiming`, SRS-F090 to SRS-F092):

| Item | Value |
|------|-------|
| Control period | 1000 ms |
| Coil burst | 900 ms from the command, ended by the driver's timer |
| Settling before the sample | 50 ms after the burst ends (air coils: τ = L/R < 1 ms; rods: eddy currents and the rod's recoil, to be measured) |
| Sample | at 950–1000 ms of the period, coils off |
| Impulse | dipole × 1000/900, saturated at 0.2 A·m², so the impulse per period is the law's |
| A sample inside the burst or the settling | not used: no B-dot, no rate estimate that cycle; counted (`mag_rejected`, housekeeping `MR`), the first one after init logged as a fault |
| Detumble without a gyro | not completed on a rate estimate left stale by a rejected sample |

The Monte Carlo (§11.4) shows the window costs nothing with the gyro and is what keeps the gyroless case working: with
the coils on through the sample and the software unaware, the gyroless case loses 27 of 40 trials to the 285-minute
timeout; with the window enforced but the coils left on (a driver fault), every sample is rejected (43,197 in half a day)
and the detumble times out as a reported fault instead of a false completion.

### 11.2 Inertia (layout B, STR-01, STR-27)

`sqso_fsw::adcs::Inertia` holds the mean of the CBE and MEV tensors of the final CAD (`docs/environment/MASS_BUDGET.md`
§6.1 and §6.5, CDR round 3: layout B, CubeSpace rods, all four AntS elements, the C-13 ballast as re-placed, the
harness), kg·m² × 10⁻³ (products × 10⁻³ as well, the tensor's off-diagonal elements):

| Tensor | Ixx | Iyy | Izz | Ixy | Ixz | Iyz | Iyy/Izz | Major axis from +Y |
|--------|-----|-----|-----|-----|-----|-----|---------|--------------------|
| Both wings (flight, `DEPLOYED`, `INERTIA`) | 3.0645 | 4.479 | 4.2615 | +0.0118 | −0.0153 | −0.0011 | 1.051 | 0.6° |
| Both wings, CBE (1,513.9 g) | 2.852 | 4.199 | 3.977 | +0.0114 | −0.0110 | −0.0040 | 1.056 | — |
| Both wings, MEV (1,747.8 g) | 3.274 | 4.756 | 4.543 | +0.0122 | −0.0196 | +0.0019 | 1.047 | — |
| Wing W1 only (`WING_1_ONLY`) | 3.027 | 4.165 | 3.910 | −0.1610 | −0.0248 | +0.0080 | 1.071 | 8.3° |
| Wing W2 only (`WING_2_ONLY`) | 3.027 | 4.166 | 3.911 | +0.1843 | −0.0058 | +0.0080 | 1.073 | 9.1° |
| No wings, AntS out (`NO_WINGS`) | 2.985 | 3.856 | 3.560 | +0.0115 | −0.0153 | +0.0171 | 1.084 | 3.3° |
| Stowed (the mission simulation) | 2.960 | 3.292 | 2.970 | — | — | — | — | — |

The flight constant is the mean of the CAD refresh before the last (2.853, 4.200, 3.978 and 3.276, 4.758, 4.545); the
last refresh moved the rows by at most 0.002 × 10⁻³, which leaves the constant and the ratios as they are. The margin
that matters, Iyy over Izz, is 5 %; the products tilt the deployed major axis 0.6° from +Y. The Monte Carlo plant
carries the full tensor (`sqso_sil::adcs_mc::Body`); the mission simulation's attitude model the diagonal, with the
one-wing diagonals after a single wing deploys.

### 11.3 Geometry by Telecommand (STR-27)

`ADCS_SET_GEOMETRY` (0x24, one argument, persisted in the operations record) selects the spin axis and the tensor the
law uses (`Geometry::PRESETS`, SRS-F094); each axis is the major axis of its CAD tensor (checked against the tensor's
eigenvector to 0.01°):

| Preset | Spin axis â | Tensor | Use |
|--------|-------------|--------|-----|
| 0 (default) | +Y | both wings | nominal |
| 1 | major axis of W1-only, (−0.1381, 0.9896, 0.0410), 8.3° from +Y | W1 only | wing W2 failed to deploy |
| 2 | major axis of W2-only, (0.1557, 0.9875, 0.0247), 9.1° from +Y | W2 only | wing W1 failed to deploy |
| 3 | major axis with no wings, (0.0121, 0.9983, 0.0568), 3.3° from +Y | no wings | neither wing deployed |

Any other value is refused (INVALID_PARAM). The ground chooses from the deployment telemetry (housekeeping `DEP`).

### 11.4 Monte Carlo at CDR

`software/simulation/cdr/avionics/adcs_mc_cdr.sh` (40 trials of 2 days each per case, M-G370PDT0 unless stated, the
window of §11.1 and the full tensor of §11.2 in the plant). The angle is between +Y and the Sun in sunlight.

*Spin rate and nutation gain, robust over Iyy/Izz 1.00–1.05.* Sweep of 6, 7, 8, 9 and 10°/s against k_n of 1, 3, 5
and 10·10⁻⁴ N·m·s, with Izz = Iyy/ratio and the flight products (mean angle / time within 15°):

| Iyy/Izz | 6°/s, 1·10⁻⁴ (1.4) | 6°/s, 3·10⁻⁴ | **8°/s, 3·10⁻⁴ (1.5)** | 8°/s, 5·10⁻⁴ | 10°/s, 3·10⁻⁴ |
|---------|---------------------|--------------|-------------------------|--------------|----------------|
| 1.000 | 14.7° / 59 % | 5.5° / 96 % (worst p95 137°) | **6.9° / 93 %** (worst p95 39–70°) | 4.9° / 95 % (worst p95 84°) | 11.2° / 85 % |
| 1.005 | — | — | **4.4° / 99.9 %** | 2.9° / 99.8 % | — |
| 1.020 | 10.5° / 87 % | 3.9° / 99.7 % | **4.2° / 99.9 %** | — | 5.5° / 98.8 % |
| 1.037 (flight) | 8.7° / 97 % | 3.7° / 99.8 % | **4.0° / 99.6 %** | — | 4.7° / 99.9 % |
| 1.050 | 7.8° / 99.2 % | 3.8° / 99.4 % | **3.8° / 99.9 %** | 2.8° / 99.7 % | 4.4° / 100 % |

No case of the sweep timed out. 8°/s with 3·10⁻⁴ is the best compromise: at the degenerate end (Iyy = Izz) it keeps
93 % of the time within 15° (6°/s with 1·10⁻⁴: 59 %) with a worst-trial 95th percentile of 39–70° (the spread is the
sensitivity to the exact rate, 8.000 against the flight 7.998°/s; 6°/s with 3·10⁻⁴: 137°), and from 1.005 up it keeps
at least 99.6 %. Higher gains do better at 1.005–1.05 but worse at 1.000 (5·10⁻⁴: worst p95 84°). Spinning about the plant's major axis instead of +Y gives 14.6° mean (the axis is 14° from the wing
normal) and 4.51 W instead of 4.64 W: the flight software keeps +Y and lets the nutation damping hold the spin there.

*Without a gyro* the 3·10⁻⁴ gain fails (mean 55°, 4 % within 15°): the magnetometer rate misses the rate about the
field and the stronger damping acts on the error. The software therefore uses 1·10⁻⁴ when there is no gyro
(`NUTATION_GAIN_NO_GYRO`): mean 17°, 47 % within 15°, 4.42 W, as before.

*Robustness at the flight gains (8°/s, 3·10⁻⁴; replaces BUILD_VS_BUY §6.4 at 6°/s).* Re-run on 2026-10-27 (flight
software round 3, Version 1.6) with the final CAD tensors (MECHANICAL_DESIGN B §5, MASS_BUDGET v3.1 §6.1 and §6.5: CBE
1,513.9 g, MEV 1,747.8 g, the C-13 ballast as re-placed), the flight software's `INERTIA` and `Inertia::DEPLOYED`
their mean, the default plant that mean with its products, and the round-3 flight software (the coil drive of
SQSO-OBC-BRD-001 §10 for the coil power). The earlier version of this table (mean 4.0°, coils 29 mW for the M-G370)
predates the CAD tensor; the simulation of 2026-10-27 before round 3 already gave 1.8° and 5 mW, so the change is the
plant, not the round-3 software. The sweep above is not re-run:

| Case | Not detumbled | Slowest capture | Mean | Worst p95 | Within 15° | Worst eclipse exit | Generation mean / worst trial | Coils |
|------|---------------|-----------------|------|-----------|------------|--------------------|-------------------------------|-------|
| M-G370PDT0 | 0 | 1.2 h | 1.8° | 7.3° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| BMI088, calibrated (0.1°/s) | 0 | 1.2 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 7 mW |
| BMI088, bias 0.2°/s | 0 | 1.3 h | 2.5° | 9.8° | 100 % | 10° | 4.65 / 4.50 W | 10 mW |
| BMI088, bias 0.3°/s | 0 (1 timeouts) | 2.9 h | 5.4° | 148° | 97.5 % | 151° | 4.59 / 2.52 W | 14 mW |
| BMI088, bias 0.5°/s | 0 (15 timeouts) | 21.4 h | 29° | 166° | 72.6 % | 169° | 4.12 / 2.52 W | 12 mW |
| BMI088 uncalibrated (1°/s) | 1 (31 timeouts) | 1.2 h | 72° | 166° | 23.9 % | 176° | 3.19 / 2.49 W | 4 mW |
| No gyro (magnetometer rate, k_n 1·10⁻⁴) | 0 | 1.2 h | 20° | 56° | 40.9 % | 32° | 4.33 / 3.86 W | 11 mW |
| Tip-off σ 3.33°/s (dispenser) | 0 | 1.0 h | 1.8° | 7.3° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| Tip-off σ 10°/s, up to 30°/s | 0 | 1.5 h | 1.8° | 7.3° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| Tip-off σ 30°/s, up to 50°/s | 0 | 1.7 h | 1.8° | 7.3° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| Residual dipole 10 mA·m² | 0 | 4.0 h | 4.8° | 21° | 98.0 % | 29° | 4.63 / 4.46 W | 7 mW |
| Residual dipole 20 mA·m² | 0 (3 timeouts) | 10.2 h | 9.6° | 45° | 80.8 % | 58° | 4.57 / 4.21 W | 10 mW |
| Residual dipole 50 mA·m² | 0 (25 timeouts) | 32.5 h | 38° | 170° | 33.0 % | 164° | 3.95 / 2.62 W | 19 mW |
| Magnetometer 200 nT, sun sensors 10° | 0 | 1.2 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 8 mW |
| Iyy/Izz 1.000 | 0 | 1.2 h | 24° | 163° | 76.2 % | 178° | 4.32 / 3.64 W | 9 mW |
| Iyy/Izz 1.005 | 0 | 1.2 h | 2.0° | 7.8° | 99.9 % | 30° | 4.65 / 4.49 W | 7 mW |
| Iyy/Izz 1.020 | 0 | 1.3 h | 1.9° | 7.4° | 100 % | 10° | 4.65 / 4.50 W | 7 mW |
| Iyy/Izz 1.050 | 0 | 1.2 h | 1.8° | 7.2° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| **CAD tensor, CBE (1,513.9 g; final refresh)** | 0 | 1.2 h | 1.8° | 7.1° | 100 % | 11° | 4.65 / 4.50 W | 6 mW |
| **CAD tensor, MEV (1,747.8 g; final refresh)** | 0 | 1.1 h | 1.7° | 6.8° | 100 % | 8° | 4.65 / 4.50 W | 7 mW |
| Coupling 0 / 3·10⁻⁴ T per A·m² | 0 | 1.2 h | 1.8° | 7.3° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| Coils on through the sample, software unaware (gyro) | 0 | 1.2 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Same, no gyro | 0 (27 timeouts) | 5.1 h | 68° | 167° | 11.5 % | 174° | 3.15 / 2.41 W | 126 mW |

*One wing (STR-27).* Generation is the measure; with one wing the +Y angle and the generation disagree because the
remaining wing is off the centre line:

| Deployed | Preset | Mean (+Y) | Within 15° | Generation mean / worst | Rate across the axis | Coils |
|----------|--------|-----------|------------|-------------------------|----------------------|-------|
| W1 only | 0 (+Y) | 6.0° | 99.9 % | 3.12 / 3.02 W | 1.28°/s | 45 mW |
| W1 only | **1** | 8.3° | 99.9 % | 3.17 / 3.06 W | 0.15°/s | 5 mW |
| W2 only | 0 (+Y) | 7.0° | 99.5 % | 3.11 / 3.01 W | 1.38°/s | 51 mW |
| W2 only | **2** | 9.3° | 99.9 % | 3.20 / 3.07 W | 0.15°/s | 5 mW |
| None | 0 (+Y) | 2.0° | 100 % | 1.62 / 1.57 W | 0.40°/s | 9 mW |
| None | 3 | 3.7° | 100 % | 1.62 / 1.57 W | 0.14°/s | 4 mW |

Re-run on 2026-10-27 (round 3, second part) against the CAD's one-wing and no-wing tensors (STR-27, the table of
§11.2; the plant and the preset both carry the mean tensor). Every case captures (slowest 1.35 h) and stays
spin-stabilized with either preset. The matching preset gives 1.7 % (W1) and 2.9 % (W2) more power and a ninth of the
coil power, because it spins about the principal axis (the Sun then sits 8–9° off +Y, at the major axis); with no wings
the default is as good (1.62 W either way). The ground should send the matching preset after a confirmed one-wing
deployment.

### 11.5 Limits and Open Items

- Rigid body; the 2 m tape elements are flexible (first mode not known) and the wings' hinge stiffness is not modelled.
  At 8°/s the spin is below any plausible element mode, but the energy dissipation of flexible appendages drives a spin
  about a non-major axis toward the major axis; the nutation damping must out-pull it. Open: a flexible-appendage case
  once the AntS element properties are known (RID GNC-03).
- CR0002 remanence and hysteresis are not modelled; the rod's recoil after the burst may need a settle longer than
  50 ms. Measure on the EM; the window is a constant (`MAG_SETTLE_MS`).
- The baseline (BASELINE §8) states 6°/s; the flight constant is now 8°/s. The chair updates the baseline.
- The magnetometer, sun-sensor ADC, gyro and coil drivers are built against the data sheets and tested with
  register-level mocks (flight software round 3, SQSO-ICD-004 1.3 §12.1); none has run on hardware (RID AV-03). The
  Monte Carlo runs the flight law against datasheet sensor models.

---

## Appendix A: Coordinate Frames

### A.1 Body Frame

- Origin: Spacecraft center of mass
- +X: Along the wings' hinge-to-tip direction
- +Y: Solar array normal (sun-pointing and spin axis; the deployed wings lie in the +Y face plane)
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
| 1.1 | 2026-10-16 | SMART-QSO Team | §8.2 test IDs aligned with the procedure files (finding VV-18) |
| 1.2 | 2026-10-16 | SMART-QSO Team | §2.4 vibration: interim SpaceX UG v11 level of TP-ENV-002 (≥ 5.57 Grms MPE), replacing 14.1 Grms (finding LA-05) |
| 1.3 | 2026-10-26 | SMART-QSO Team | From the mission simulation: detumble within 3 orbits (F001, P001, §5.3); spin-stabilized sun pointing of +Y (F002, §4.2.1, §5.1, App. A) |
| 1.4 | 2026-10-26 | SMART-QSO Team | From the ADCS Monte Carlo (BUILD_VS_BUY §6): B-dot gain 10⁴ (§4.1); spin hold, 6°/s spin, nutation gain 10⁻⁴ and the capture criterion (§4.2.1, §5) |
| 1.5 | 2026-10-27 | SMART-QSO Team (CDR avionics review) | §11 new: magnetometer window and coil burst, layout B full inertia tensor, geometry presets by telecommand, Monte Carlo re-run (8°/s, k_n 3·10⁻⁴ with gyro, 1·10⁻⁴ without; one-wing cases); §4.2.1 law with the full tensor; §3.2.1 magnetometer not selected; §3.3.1 CR0002 rods; §6.1 L293D unusable at 3.3 V |
| 1.6 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3) | §11.4 robustness and one-wing tables re-run with the final CAD tensors (CBE 1,513.9 g, MEV 1,747.8 g; the flight constant `INERTIA` their mean, (3.0565, 4.4675, 4.2525) × 10⁻³ kg·m² with Ixy +13.2, Ixz −12.9, Iyz +0.05 × 10⁻⁶): the plant rows 1.8° mean, 100 % within 15°, capture within 1.2 h; Iyy/Izz 1.000 now 76 % within 15° (the products of the final CAD); §11.5 the drivers exist (built against the data sheets, not run on hardware) |
| 1.7 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, second part) | §11.2 the final CAD tensors (deployed mean (3.0645, 4.479, 4.2615) × 10⁻³ with Ixy +11.8, Ixz −15.3, Iyz −1.05 × 10⁻⁶; the CBE and MEV rows; the one-wing and no-wing tensors of STR-27; stowed); §11.3 the presets' axes from the CAD tensors (8.3°, 9.1°, 3.3° from +Y); §11.4 the plant rows and the one-wing table re-run against the CAD plant |
