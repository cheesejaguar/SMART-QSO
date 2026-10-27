# Attitude Determination and Control Subsystem (ADCS) Design Document

**Document Number**: SMART-QSO-ADCS-001
**Version**: 1.9
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
| ADCS-F002 | Achieve spin-stabilized coarse sun pointing of +Y, the deployed array's normal, judged by the generation it gives (ADCS-P002, decision C-23) | Analysis, Test |
| ADCS-F003 | Determine the Sun's direction in the body frame from the sun sensors and the body rate from the gyro (ADCS-P007), as the sun-pointing law needs; no onboard three-axis attitude estimate (C-22, §4.3) and no accuracy required of the sun vector (C-23) | Analysis, Test |
| ADCS-F004 | Operate autonomously without ground commands | Test |
| ADCS-F005 | Provide attitude telemetry to OBC | Test |
| ADCS-F006 | Support graceful degradation on sensor failure | Test |
| ADCS-F007 | Generate torque in 3 orthogonal axes | Test |

### 2.2 Performance Requirements

| ID | Requirement | Threshold | Goal |
|----|-------------|-----------|------|
| ADCS-P001 | Detumble time | < 3 orbits (285 min) | < 2 orbits |
| ADCS-P002 | Pointing for power (decision C-23), in every Monte Carlo trial with the face-photodiode and albedo model (§11.6): sunlit-averaged generation lost to the pointing (against +Y held on the Sun) | ≤ 5 %, and the Sun within 30° of +Y for ≥ 95 % of sunlit time | — |
| ADCS-P003 | Withdrawn (decision C-23): no sun-vector knowledge requirement; the knowledge error is reported in §11.6 (9.8° mean with albedo) | — | — |
| ADCS-P004 | Peak power consumption | <2W | <1W |
| ADCS-P005 | Steady-state power | <0.5W | <0.3W |
| ADCS-P006 | Control bandwidth | >0.01 Hz | >0.02 Hz |
| ADCS-P007 | Body-rate knowledge per axis, bias in use (C-22; the gyro, BASELINE §8) | ≤ 0.2°/s | ≤ 0.1°/s |

### 2.3 Interface Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| ADCS-I001 | I2C interface to OBC | Address 0x1E (mag), 0x40-0x45 (sun) |
| ADCS-I002 | GPIO magnetorquer control | PWM capable, 100mA drive |
| ADCS-I003 | Sun vector and body rate output (no attitude quaternion: C-22) | Via telemetry structure (housekeeping `W`, ADCS telemetry) |
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
                      | |  Sun vector,  | |
                      | |  rate (C-22)  | |
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

### 4.3 Attitude Estimator (not in the flight baseline: decision C-22)

The spin-stabilised sun-pointing law of §4.2.1 uses the sun vector, the field and the body rate, nothing else. There
is no onboard three-axis attitude estimate, and none is planned: the extended Kalman filter (state q, ω) and the
TRIAD routine that stood here since Version 1.0 were never implemented (`sqso_fsw::adcs` has no estimator; finding
P3F-G-07) and are removed from the flight baseline. What the law needs (ADCS-F003) is the sun vector from the face
photodiodes and the gyro's rate (ADCS-P007, ≤ 0.2°/s: at 0.3°/s the Monte Carlo has detumble timeouts). No accuracy is
required of the sun vector (decision C-23): the pointing is judged by its power (ADCS-P002), which the Monte Carlo
measures with the photodiodes and the Earth's albedo (§11.6).

*Note.* TRIAD remains a ground tool: the downlinked sun vector and field, with the GNSS position, the field model and
the Sun's ephemeris, give the ground a three-axis attitude for the analysis of a pass. It is not flight software
and no requirement rests on it.

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
| Quaternion (q0-q3) | — | — | not produced: no onboard attitude estimate (C-22) |
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
    float quaternion[4];    // Not produced (C-22): no onboard attitude estimate
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
| ADCS-F002 | Analysis (Monte Carlo, photodiodes and albedo: ADCS-P002, §11.6), HILS Test | Pending: met at the 3 mA·m² design dipole; not met at the 10 mA·m² limit (6 of 40 trials, §11.6) |
| ADCS-F003 | Analysis (Monte Carlo, §11.6), Test (TP-ADCS-003 §3.1: photodiode response, shadow, gyro at the spin rate) | Pending |
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
| Both wings (flight, `DEPLOYED`, `INERTIA`) | 3.064 | 4.481 | 4.262 | +0.0112 | −0.0119 | +0.0004 | 1.051 | 0.5° |
| Both wings, CBE (1,520.0 g) | 2.853 | 4.202 | 3.979 | +0.0108 | −0.0080 | −0.0027 | 1.056 | — |
| Both wings, MEV (1,754.9 g) | 3.276 | 4.760 | 4.546 | +0.0115 | −0.0158 | +0.0035 | 1.047 | — |
| Wing W1 only (`WING_1_ONLY`) | 3.027 | 4.167 | 3.911 | −0.1616 | −0.0214 | +0.0095 | 1.071 | 8.4° |
| Wing W2 only (`WING_2_ONLY`) | 3.027 | 4.167 | 3.911 | +0.1836 | −0.0023 | +0.0095 | 1.073 | 9.1° |
| No wings, AntS out (`NO_WINGS`) | 2.986 | 3.858 | 3.560 | +0.0108 | −0.0119 | +0.0186 | 1.084 | 3.6° |
| Stowed, wings and AntS (`sqso_sil::adcs_mc::STOWED`, MASS_BUDGET §6.5) | 2.960 | 3.292 | 2.970 | +0.0115 | −0.0153 | +0.0345 | 1.108 | 6.2° |
| Both wings, AntS stowed (`ants_stowed(DEPLOYED)`, §11.6; derived) | 3.038 | 3.915 | 3.672 | +0.0119 | −0.0153 | +0.0163 | 1.066 | 3.8° |

The flight constants are MASS_BUDGET 3.2 §6.5's rows (flight software round 3, fourth part, after the mass-property
refresh of the harness parts audit): the deployed diagonal moved by at most 0.002 × 10⁻³ (0.05 %) and the presets' major
axes by at most 0.3°, so the Monte Carlo of §11.4 stands. The margin that matters, Iyy over Izz, is 5 %; the products tilt
the deployed major axis 0.5° from +Y. The Monte Carlo plant carries the full tensor (`sqso_sil::adcs_mc::Body`); the
mission simulation's attitude model the diagonal, with the
one-wing diagonals after a single wing deploys.

### 11.3 Geometry by Telecommand (STR-27)

`ADCS_SET_GEOMETRY` (0x24, one argument, persisted in the operations record) selects the spin axis and the tensor the
law uses (`Geometry::PRESETS`, SRS-F094); each axis is the major axis of its CAD tensor. The axes below are the code's
(`software/rust/sqso-fsw/src/adcs.rs`, `Geometry::PRESETS`), regenerated in Version 1.9 (finding P3F-G-04); each is
within 0.003° of the major eigenvector of the tensor beside it:

| Preset | Spin axis â | Tensor | Use |
|--------|-------------|--------|-----|
| 0 (default) | +Y | both wings | nominal |
| 1 | major axis of W1-only, (−0.1383, 0.9894, 0.0443), 8.4° from +Y | W1 only | wing W2 failed to deploy, **W1 identified** (§11.7) |
| 2 | major axis of W2-only, (0.1550, 0.9874, 0.0316), 9.1° from +Y | W2 only | wing W1 failed to deploy, **W2 identified** (§11.7) |
| 3 | major axis with no wings, AntS out, (0.0115, 0.9980, 0.0616), 3.6° from +Y | no wings | neither wing deployed (optional: preset 0 is as good, §11.6) |

Any other value is refused (INVALID_PARAM). **Decision C-21:** with one wing deployed the ground keeps preset 0 until the
commissioning identification of §11.7 names the wing. Housekeeping `DEP` carries only the combined wing sense (the
two hinge switches are in series) and `PA` the total array power, so `DEP` alone cannot choose between 1 and 2, and
the wrong one-wing preset is worse than preset 0 (§11.6).

### 11.4 Monte Carlo at CDR

`software/simulation/cdr/avionics/adcs_mc_cdr.sh` (40 trials of 2 days each per case, M-G370PDT0 unless stated, the
window of §11.1 and the full tensor of §11.2 in the plant). The angle is between +Y and the Sun in sunlight.

*Spin rate and nutation gain over Iyy/Izz (decision C-4).* Sweep of 6, 7, 8, 9 and 10°/s against k_n of 1, 3, 5 and
10·10⁻⁴ N·m·s, with Izz = Iyy/ratio and the flight products, re-run on 2026-10-27 for Version 1.9 with the current
tensors (`Inertia::DEPLOYED`, MASS_BUDGET 3.2; the stowed products corrected; finding P3F-G-04; `adcs_mc_cdr.sh OUT
sweep`). Mean angle / time within 15° / worst trial's 95th percentile, a selection of the 100 combinations:

| Iyy/Izz | 6°/s, 1·10⁻⁴ (1.4) | 6°/s, 3·10⁻⁴ | 8°/s, 1·10⁻⁴ | **8°/s, 3·10⁻⁴ (flight)** | 8°/s, 5·10⁻⁴ | 8°/s, 1·10⁻³ | 10°/s, 3·10⁻⁴ |
|---------|---------------------|--------------|--------------|----------------------------|--------------|--------------|----------------|
| 1.000 | 15° / 82.8 % / 152° | 22° / 78.0 % / 162° | 13° / 85.7 % / 156° | **19° / 80.8 % / 162°** | 23° / 77.2 % / 163° | 26° / 72.0 % / 160° | 22° / 78.6 % / 163° |
| 1.005 | 6.5° / 95.1 % / 144° | 9.0° / 92.5 % / 158° | 2.9° / 99.8 % / 11° | **2.0° / 99.9 % / 7.7°** | 2.0° / 99.9 % / 7.1° | 2.2° / 99.9 % / 8.2° | 1.6° / 100 % / 6.5° |
| 1.020 | 2.8° / 100 % / 11° | 2.5° / 100 % / 9.5° | 2.2° / 100 % / 8.4° | **1.8° / 100 % / 7.2°** | 1.8° / 100 % / 6.7° | 1.9° / 100 % / 6.6° | 1.5° / 100 % / 6.3° |
| 1.050 | 2.5° / 100 % / 9.8° | 2.4° / 100 % / 9.4° | 1.8° / 100 % / 7.1° | **1.8° / 100 % / 7.3°** | 1.8° / 100 % / 6.9° | 1.8° / 100 % / 6.6° | 1.4° / 100 % / 5.9° |
| 1.051 (flight tensor) | 2.4° / 100 % / 9.6° | 2.4° / 100 % / 9.4° | 1.8° / 100 % / 7.0° | **1.8° / 100 % / 7.3°** | 1.8° / 100 % / 7.0° | 1.8° / 100 % / 6.8° | 1.4° / 100 % / 5.8° |

No case of the sweep timed out. **What the sweep now shows (C-4's rationale, revised).** With the flight tensor
(Iyy/Izz 1.051) and anything down to 1.020 every combination holds 100 % of sunlit time within 15° (1.4–2.8° mean),
and the spin rate matters more than the gain (10°/s 1.4°, 8°/s 1.8°, 6°/s 2.4°). At 1.005 the 6°/s pairs fail
(92.5–95.1 %, a worst p95 above 140°) and every pair from 7°/s up holds 99.7 % or more, 3·10⁻⁴ better than 1·10⁻⁴
(2.0° against 2.9° at 8°/s). At the degenerate end, Iyy = Izz, **no pair holds**: the best is 8°/s with 1·10⁻⁴ at 13°
and 85.7 %, the flight pair 19° and 80.8 %, and stronger damping is worse there, not better. Version 1.5's case for
3·10⁻⁴, that it kept 93 % at Iyy = Izz where 6°/s with 1·10⁻⁴ kept 59 %, rested on the earlier products and no longer
holds. 8°/s with 3·10⁻⁴ stands, on two grounds the sweep does support: 6°/s fails at Iyy/Izz 1.005, and from 1.005 up
3·10⁻⁴ is as good as any gain at 8°/s or better. The protection against a degenerate plant is now the mass-properties
requirement (Iyy ≥ 1.03 max(Ixx, Izz), BASELINE §4; CAD 1.056 CBE, 1.047 MEV, MEV 5th percentile 1.036), not the gains.
9–10°/s would gain 0.2–0.4° of mean angle; not worth a change at CDR. Spinning about the plant's major axis instead of
+Y now gives the same (1.8° mean, 4.65 W; the axis is 0.5° from +Y with the current tensor).

*Without a gyro* the 3·10⁻⁴ gain fails (mean 55°, 4 % within 15°): the magnetometer rate misses the rate about the
field and the stronger damping acts on the error. The software therefore uses 1·10⁻⁴ when there is no gyro
(`NUTATION_GAIN_NO_GYRO`): mean 20°, 41.5 % within 15°, 4.33 W (Version 1.9 re-run).

*Robustness at the flight gains (8°/s, 3·10⁻⁴; replaces BUILD_VS_BUY §6.4 at 6°/s).* Re-run in full on 2026-10-27
for Version 1.9 (CDR round 3 closure, P3F-G-04 and -05) with MASS_BUDGET 3.2's tensors in the flight software and the
plant, the stowed tensor's products corrected (§11.6) and the new columns of §11.6. "Slowest capture" is the flight
software's declared end of the detumble, the slowest trial. Against Version 1.8 the nominal rows are unchanged
(1.8°, 100 %, 4.65 W); the degenerate Iyy/Izz 1.000 case moved from 24° and 76.2 % to 20° and 80.0 % with the stowed
products (it is the case most sensitive to the trajectory), and the weak-gyro and large-dipole rows by a similar
amount. The gating and AntS rows are new (§11.6):

| Case | Not detumbled | Slowest capture | Mean | Worst p95 | Within 15° | Worst eclipse exit | Generation mean / worst trial | Coils |
|------|---------------|-----------------|------|-----------|------------|--------------------|-------------------------------|-------|
| M-G370PDT0 | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| BMI088, calibrated (0.1°/s) | 0 | 1.3 h | 1.8° | 7.2° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| BMI088, bias 0.2°/s | 0 | 1.7 h | 2.5° | 9.5° | 100 % | 10° | 4.65 / 4.50 W | 10 mW |
| BMI088, bias 0.3°/s | 1 (3 timeouts) | 2.3 h | 8.9° | 150° | 92.7 % | 166° | 4.50 / 2.48 W | 12 mW |
| BMI088, bias 0.5°/s | 0 (12 timeouts) | 7.2 h | 26° | 162° | 75.4 % | 166° | 4.16 / 2.48 W | 13 mW |
| BMI088 uncalibrated (1°/s) | 1 (32 timeouts) | 2.5 h | 73° | 166° | 21.5 % | 172° | 3.12 / 2.37 W | 3 mW |
| No gyro (magnetometer rate, k_n 1·10⁻⁴) | 0 | 1.2 h | 20° | 56° | 41.5 % | 33° | 4.33 / 3.87 W | 11 mW |
| Tip-off σ 3.33°/s (dispenser) | 0 | 1.0 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Tip-off σ 10°/s, up to 30°/s | 0 | 1.8 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Tip-off σ 30°/s, up to 50°/s | 0 | 1.6 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Residual dipole 10 mA·m² | 0 | 3.0 h | 4.7° | 21° | 98.0 % | 29° | 4.63 / 4.46 W | 7 mW |
| Residual dipole 20 mA·m² | 0 (3 timeouts) | 11.8 h | 9.6° | 45° | 81.0 % | 58° | 4.57 / 4.21 W | 10 mW |
| Residual dipole 50 mA·m² | 0 (23 timeouts) | 31.7 h | 40° | 170° | 34.0 % | 166° | 3.94 / 2.62 W | 19 mW |
| Magnetometer 200 nT, sun sensors 10° | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 9° | 4.65 / 4.50 W | 8 mW |
| Iyy/Izz 1.000 | 0 | 1.3 h | 20° | 162° | 80.0 % | 179° | 4.38 / 3.62 W | 9 mW |
| Iyy/Izz 1.005 | 0 | 1.3 h | 2.0° | 7.7° | 99.9 % | 30° | 4.65 / 4.49 W | 7 mW |
| Iyy/Izz 1.020 | 0 | 1.3 h | 1.8° | 7.2° | 100 % | 9° | 4.65 / 4.50 W | 7 mW |
| Iyy/Izz 1.050 | 0 | 1.3 h | 1.8° | 7.2° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| Spin about the plant's major axis (0.5° from +Y) | 0 | 1.3 h | 1.8° | 6.8° | 100 % | 9° | 4.65 / 4.50 W | 6 mW |
| **CAD tensor, CBE (1,520.0 g; MASS_BUDGET 3.2)** | 0 | 1.4 h | 1.8° | 7.0° | 100 % | 11° | 4.65 / 4.50 W | 6 mW |
| **CAD tensor, MEV (1,754.9 g; MASS_BUDGET 3.2)** | 0 | 1.2 h | 1.7° | 6.8° | 100 % | 8° | 4.65 / 4.50 W | 7 mW |
| Coupling 0 / 3·10⁻⁴ T per A·m² | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Coils on through the sample, software unaware (gyro) | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Same, no gyro | 0 (27 timeouts) | 5.1 h | 68° | 164° | 11.5 % | 176° | 3.20 / 2.50 W | 126 mW |
| **Gating, 1.5 h/day Jetson + 53 min/day keyed (P3F-G-05)** | 0 | 1.3 h | 1.9° | 7.6° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| Gating, residual dipole 10 mA·m² | 0 | 3.0 h | 5.1° | 22° | 96.7 % | 33° | 4.63 / 4.45 W | 7 mW |
| Gating at twice the duty | 0 | 1.3 h | 2.0° | 8.0° | 100 % | 10° | 4.65 / 4.50 W | 5 mW |
| Gating, no gyro | 0 | 1.2 h | 17° | 52° | 51.4 % | 41° | 4.42 / 4.05 W | 10 mW |
| **AntS 3 h after the wings (C-18)** | 0 | 1.3 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |
| AntS 3 h after the wings, gating | 0 | 1.3 h | 1.9° | 7.5° | 100 % | 10° | 4.65 / 4.50 W | 6 mW |

*One wing (STR-27).* Generation is the measure; with one wing the +Y angle and the generation disagree because the
remaining wing is off the centre line:

| Deployed | Preset | Mean (+Y) | Within 15° | Generation mean / worst | Rate across the axis | Coils |
|----------|--------|-----------|------------|-------------------------|----------------------|-------|
| W1 only | 0 (+Y) | 6.0° | 99.9 % | 3.12 / 3.02 W | 1.28°/s | 45 mW |
| W1 only | **1** | 8.4° | 99.9 % | 3.17 / 3.06 W | 0.15°/s | 5 mW |
| W2 only | 0 (+Y) | 7.0° | 99.6 % | 3.11 / 3.01 W | 1.37°/s | 51 mW |
| W2 only | **2** | 9.3° | 99.9 % | 3.20 / 3.07 W | 0.15°/s | 5 mW |
| None | 0 (+Y) | 2.0° | 100 % | 1.62 / 1.57 W | 0.42°/s | 10 mW |
| None | 3 | 3.9° | 100 % | 1.62 / 1.56 W | 0.14°/s | 4 mW |

Re-run on 2026-10-27 (round 3, second part) against the CAD's one-wing and no-wing tensors (STR-27, the table of
§11.2; the plant and the preset both carry the mean tensor). Every case captures (slowest 1.35 h) and stays
spin-stabilized with either preset. The matching preset gives 1.7 % (W1) and 2.9 % (W2) more power and a ninth of the
coil power, because it spins about the principal axis (the Sun then sits 8–9° off +Y, at the major axis); with no wings
the default is as good (1.62 W either way). Re-run in Version 1.9 at the current tensors (the values above; changes
≤ 0.1° and 0.01 W), with the CBE and MEV plants, the wrong preset and gating in §11.6. **Decision C-21:** the ground
keeps preset 0 after a one-wing deployment and sends the matching preset only once the wing is identified (§11.7);
the wrong preset costs 3.5–5.6 % of the power and holds only 35–58 % of sunlit time within 15°.

*Proposed BASELINE §8 text (for the chair; P3F-G-04).* In the ADCS row, replace "Monte Carlo of the flight code with the
layout-B inertia tensor, mean 4.0°, 99.6 % of sunlit time within 15°, slowest capture 1.2 h (2.1 h from a 50°/s
tip-off); Iyy/Izz 1.00–1.05 gives 3.8–6.9° and ≥ 93 % (`docs/hardware/ADCS_DESIGN.md` 1.5 §11.4)" by "Monte Carlo of
the flight code with the CAD inertia tensor (MASS_BUDGET 3.2), generic sun-sensor error: mean 1.8°, 100 % of sunlit time
within 15°, slowest capture 1.3 h (1.6 h from a 50°/s tip-off); Iyy/Izz 1.005–1.05 gives 1.8–2.0° and ≥ 99.9 %, Iyy = Izz
20° and 80 % (no gain holds it: the mass-properties limit of §4 is the protection); magnetometer gating at the SIL duty
1.9°; with the face photodiodes and Earth albedo (the pointing requirement's model, decision C-23) 12° mean, the Sun
within 30° of +Y for 100 % of sunlit time and 2.0 % of the sunlit generation lost to the pointing (3.2 % in the worst
trial), meeting ADCS-P002 (≤ 5 %, ≥ 95 % within 30°) up to a 7 mA·m² residual dipole but not at the 10 mA·m² limit
(7.7 % and 87 % in the worst trial) (`docs/hardware/ADCS_DESIGN.md` 1.9 §11.4, §11.6)". Add: "One wing deployed: preset 0 until the wing is
identified (decision C-21; ADCS_DESIGN §11.7)".

### 11.5 Limits and Open Items

- Rigid body; the 2 m tape elements are flexible (first mode not known) and the wings' hinge stiffness is not modelled.
  At 8°/s the spin is below any plausible element mode, but the energy dissipation of flexible appendages drives a spin
  about a non-major axis toward the major axis; the nutation damping must out-pull it. Open: a flexible-appendage case
  once the AntS element properties are known (RID GNC-03).
- CR0002 remanence and hysteresis are not modelled; the rod's recoil after the burst may need a settle longer than
  50 ms. Measure on the EM; the window is a constant (`MAG_SETTLE_MS`).
- BASELINE §8 (governing) still quotes Version 1.5's Monte Carlo (mean 4.0°, 99.6 %; Iyy/Izz 1.00–1.05 gives 3.8–6.9° and
  ≥ 93 %). The current values are in §11.4; the replacement text is in the CR noted there (finding P3F-G-04).
- The magnetometer, sun-sensor ADC, gyro and coil drivers are built against the data sheets and tested with
  register-level mocks (flight software round 3, SQSO-ICD-004 1.3 §12.1); none has run on hardware (RID AV-03). The
  Monte Carlo runs the flight law against datasheet sensor models.

### 11.6 Monte Carlo Closure Cases (CDR round 3, P3F-G-05, C-21, C-22)

`software/simulation/cdr/avionics/adcs_mc_cdr.sh` (Version 1.9 of this document; `sqso_sil::adcs_mc`): 40 trials of 2
days per case, M-G370PDT0, the flight gains and window, the plant at `Inertia::DEPLOYED` unless stated. Four changes
to the model, all in the Monte Carlo and none in the flight software:

- *Magnetometer gating (SRS-F143).* While the Jetson carrier is powered or the exciter keyed the flight software does
  not use the sample and leaves the coils off (`Reading::gated`). The Monte Carlo gates at the mission simulation's
  duty (SIL Version F: 9,216 Jetson power-ons and 8,145 Wh in 732 days, 1.5 h a day; the exciter keyed 53 min a
  day), laid out as long blocks, the panel's worst case: Jetson batches of 5–10 min with at least 100 min between,
  the exciter in 4–10 min blocks started only in sunlight. 138 min a day gated (9.6 %), from separation + 3 h.
- *The AntS after the wings.* `--ants-h` deploys the AntS later than the wings at 30 min; meanwhile the plant is the
  deployed tensor less the elements' share (MASS_BUDGET §6.5 "NO_WINGS" less "Stowed", taken as the same with the
  wings out: with both wings (3.038, 3.915, 3.672) × 10⁻³ kg·m², products (+11.9, −15.3, +16.3) × 10⁻⁶, major axis 3.8°
  from +Y; the deployment then raises Iyy by 14 % with the momentum kept, so the spin drops 14 % and the law restores it).
- *The stowed tensor* now carries MASS_BUDGET 3.2 §6.5's products (+11.5, −15.3, +34.5) × 10⁻⁶ kg·m² (it had an
  older CAD's +0.5, −0.5, +31). This changes the detumble phase of every trial slightly; the robustness table of
  §11.4 is the re-run with it.
- *Sun-vector knowledge* (C-22): the angle between the flight software's sun vector (`state.sun.sun_vector`) and
  the true direction at the sample, in sunlight, steady state. The sun-sensor model is selectable: the generic
  5° 1σ white error the earlier rows used, or the face photodiodes (`adcs_mc::Photodiodes::FLIGHT`): the six
  ADS7138 channels with opposite faces subtracted as `sqso_fsw::ads7138` does, noise 10⁻³ of full sun, 3 % gain
  error per channel, a cosine response to 60° falling to half of cos θ at 90° (grazing loss), Earth albedo on each
  face by its view factor of the sunlit Earth (`env::earth`), the ±X photodiode under a stowed wing dark and behind a
  deployed one shadowed by the geometry of `env::solar` (for the Sun between 10° and 61° from +Y toward that side).

*Gating and the AntS (P3F-G-05 (a), (d)).* Rows of the §11.4 table: gating costs 0.1° of mean angle (1.9°, 100 % within
15°, 4.65 W); with the 10 mA·m² residual dipole 5.1° and 96.7 % (98.0 % ungated); at twice the duty 2.0°; without the
gyro 17° and 51 % (the ungated gyroless case is 20° and 42 %: the gated cycles hold the coils off, which the weak
gyroless damping does not miss). The AntS 3 h after the wings: the steady state is unchanged (1.8°) and the worst
sunlit angle in the 3 h after the deployment is 11° (15° with gating).

*Wings stowed, the AntS held 25 h (P3F-G-05 (b); C-18's cap).* The stowed state (wings and AntS stowed, the
`STOWED` tensor, which has no preset) between 6 and 24 h, then the state after the AntS deploys at 25 h:

| Case | Slowest capture | Mean | Worst p95 | Within 15° | Worst eclipse exit | Generation mean / worst | Coils | Worst in the 3 h after the AntS |
|------|-----------------|------|-----------|------------|--------------------|-------------------------|-------|---------------------------------|
| Wings stowed, AntS stowed (6–24 h), preset 0 | 1.2 h | 2.5° | 8.7° | 100 % | 18° | 1.57 / 1.53 W | 13 mW | — |
| Same, preset 3 | 1.2 h | 4.3° | 11° | 100 % | 17° | 1.57 / 1.53 W | 6 mW | — |
| Wings stowed, AntS out at 25 h (from 42 h), preset 0 | 1.2 h | 2.0° | 7.9° | 100 % | 16° | 1.62 / 1.57 W | 10 mW | 16° |
| Same, preset 3 | 1.2 h | 3.9° | 10° | 100 % | 12° | 1.62 / 1.56 W | 4 mW | 13° |
| No wings, AntS with the wings (the earlier case), preset 0 | 1.1 h | 2.0° | 8.1° | 100 % | 16° | 1.62 / 1.57 W | 10 mW | — |
| Same, preset 3 | 1.1 h | 3.9° | 10° | 100 % | 12° | 1.62 / 1.56 W | 4 mW | — |
| Both wings, AntS 3 h after them | 1.3 h | 1.8° | 7.3° | 100 % | 10° | 4.65 / 4.50 W | 6 mW | 11° |
| Same, gating | 1.3 h | 1.9° | 7.5° | 100 % | 10° | 4.65 / 4.50 W | 6 mW | 15° |

Preset 0 holds the stowed spacecraft as well as preset 3 or better (2.5° against 4.3°, 1.57 W either way), and the
AntS deployment after 25 h is a transient of at most 16°. No stowed preset is needed.

*One wing, plant ≠ flight model (P3F-G-05 (e), C-21).* The plant at the CAD's one-wing CBE and MEV tensors
(MASS_BUDGET 3.2 §6.1) as well as their mean, with preset 0, the right preset and the wrong one; the last column is the
mean body-rate estimate's tilt from +Y toward +X, atan2(ω̄x, ω̄y), over the steady state (mean over trials, and the
range), which §11.7 uses:

| Deployed | Plant | Preset | Mean (+Y) | Worst p95 | Within 15° | Generation mean / worst | Rate across the axis | Coils | Rate tilt toward +X |
|----------|-------|--------|-----------|-----------|------------|-------------------------|----------------------|-------|---------------------|
| W1 only | mean | 0 (C-21 default) | 6.0° | 12° | 99.9 % | 3.12 / 3.02 W | 1.28°/s | 45 mW | −3.6° (−4.4 to −3.0) |
| W1 only | mean | **1** (right) | 8.4° | 14° | 99.9 % | 3.17 / 3.06 W | 0.15°/s | 5 mW | −8.0° (−8.8 to −7.4) |
| W1 only | mean | 2 (wrong) | 14° | 29° | 56.3 % | 3.06 / 2.94 W | 2.19°/s | 67 mW | +2.8° (+2.3 to +3.3) |
| W1 only | CBE | 0 (C-21 default) | 5.9° | 11° | 99.8 % | 3.12 / 3.02 W | 1.26°/s | 43 mW | −3.5° (−4.2 to −3.0) |
| W1 only | CBE | **1** (right) | 8.3° | 14° | 99.9 % | 3.17 / 3.06 W | 0.17°/s | 5 mW | −7.9° (−8.7 to −7.3) |
| W1 only | CBE | 2 (wrong) | 14° | 28° | 58.1 % | 3.06 / 2.95 W | 2.17°/s | 66 mW | +2.9° (+2.4 to +3.4) |
| W1 only | MEV | 0 (C-21 default) | 6.2° | 12° | 99.8 % | 3.12 / 3.02 W | 1.32°/s | 46 mW | −3.8° (−4.5 to −3.1) |
| W1 only | MEV | **1** (right) | 8.5° | 14° | 99.9 % | 3.17 / 3.06 W | 0.16°/s | 5 mW | −8.1° (−8.9 to −7.5) |
| W1 only | MEV | 2 (wrong) | 14° | 29° | 54.4 % | 3.06 / 2.94 W | 2.21°/s | 68 mW | +2.8° (+2.2 to +3.3) |
| W2 only | mean | 0 (C-21 default) | 7.0° | 14° | 99.6 % | 3.11 / 3.01 W | 1.37°/s | 51 mW | +3.4° (+2.7 to +4.1) |
| W2 only | mean | **2** (right) | 9.3° | 14° | 99.9 % | 3.20 / 3.07 W | 0.15°/s | 5 mW | +8.9° (+8.1 to +9.5) |
| W2 only | mean | 1 (wrong) | 16° | 30° | 38.3 % | 3.03 / 2.92 W | 2.17°/s | 67 mW | −1.9° (−2.5 to −1.3) |
| W2 only | CBE | 0 (C-21 default) | 6.7° | 14° | 99.5 % | 3.12 / 3.01 W | 1.35°/s | 49 mW | +3.4° (+2.8 to +4.1) |
| W2 only | CBE | **2** (right) | 9.4° | 14° | 99.9 % | 3.20 / 3.07 W | 0.16°/s | 5 mW | +8.9° (+8.1 to +9.5) |
| W2 only | CBE | 1 (wrong) | 16° | 30° | 41.6 % | 3.03 / 2.92 W | 2.15°/s | 66 mW | −1.8° (−2.5 to −1.1) |
| W2 only | MEV | 0 (C-21 default) | 7.3° | 14° | 99.5 % | 3.11 / 3.01 W | 1.40°/s | 52 mW | +3.4° (+2.7 to +4.0) |
| W2 only | MEV | **2** (right) | 9.3° | 13° | 100 % | 3.20 / 3.07 W | 0.15°/s | 5 mW | +8.8° (+8.0 to +9.5) |
| W2 only | MEV | 1 (wrong) | 17° | 31° | 35.4 % | 3.02 / 2.91 W | 2.17°/s | 68 mW | −2.0° (−2.5 to −1.4) |
| W1 only, gating | mean | 0 | 7.0° | 18° | 95.2 % | 3.12 / 3.02 W | 1.37°/s | 39 mW | −4.2° (−4.9 to −3.5) |
| Both (reference) | mean | 0 | 1.8° | 7.3° | 100 % | 4.65 / 4.50 W | 0.18°/s | 6 mW | +0.1° (−0.6 to +0.7) |

The preset-0 one-wing rows re-run at the current tensors agree with §11.4 (6.0° and 7.0°, 3.12 and 3.11 W); the CBE and
MEV plants move every row by at most 0.4° and 0.01 W. **The wrong one-wing preset is worse than preset 0**: 14–17° mean,
35–58 % of sunlit time within 15° and 3.02–3.06 W, against 3.11–3.12 W with preset 0 and 3.17–3.20 W with the right
preset (preset 0 costs 1.6–2.9 %, the wrong preset 3.5–5.6 %). This supports C-21: preset 0 until the wing is
identified.

*Sun-vector knowledge (reported, not a requirement: C-23).* Knowledge columns (mean over trials, the worst trial's 95th percentile, and
the worst trial's share of sunlit samples within 5°), with the pointing they give:

| Sun-sensor model | Knowledge mean | Worst p95 | Within 5°, worst trial | Pointing mean | Worst p95 | Within 15° | Generation mean / worst |
|------------------|----------------|-----------|------------------------|---------------|-----------|------------|-------------------------|
| Generic 5° 1σ white error (the other rows) | 3.1° | 8.4° | 79 % | 1.8° | 7.3° | 100 % | 4.65 / 4.50 W |
| Generic, magnetometer 200 nT, sun sensors 10° | 6.3° | 16.8° | 50 % | 1.8° | 7.3° | 100 % | 4.65 / 4.50 W |
| **Face photodiodes (flight model)** | 9.8° | 27.2° | 21 % | 12° | 21° | 73.6 % | 4.56 / 4.41 W |
| Photodiodes, no response beyond 85° | 10.2° | 27.2° | 18 % | 12° | 21° | 72.4 % | 4.56 / 4.41 W |
| Photodiodes without albedo | 1.0° | 3.2° | 100 % | 2.1° | 8.4° | 100 % | 4.65 / 4.50 W |
| Photodiodes, gating | 9.3° | 26.4° | 24 % | 11° | 21° | 77.4 % | 4.57 / 4.42 W |
| Photodiodes, residual dipole 10 mA·m² | 11.8° | 32.4° | 8 % | 14° | 36° | 56.2 % | 4.50 / 4.27 W |
| Photodiodes, no gyro | 22.9° | 63.0° | 9 % | 34° | 58° | 8.2 % | 3.85 / 3.61 W |
| Photodiodes, W1 only, preset 0 | 8.9° | 24.3° | 21 % | 11° | 23° | 77.4 % | 3.11 / 3.01 W |
| Photodiodes, W1 only, preset 1 | 12.3° | 29.0° | 15 % | 11° | 23° | 71.7 % | 3.18 / 3.07 W |
| Photodiodes, W2 only, preset 0 | 8.7° | 25.1° | 26 % | 11° | 24° | 76.2 % | 3.10 / 3.00 W |
| Photodiodes, no wings (AntS out), preset 0 | 7.0° | 14.0° | 14 % | 7.6° | 16° | 99.5 % | 1.63 / 1.57 W |
| Photodiodes, wings and AntS stowed, preset 0 | 7.5° | 15.9° | 13 % | 8.4° | 18° | 98.4 % | 1.59 / 1.54 W |

**With the face photodiodes the albedo dominates the sun vector.** The knowledge error is 9.8° on average and 27° at
the worst trial's 95th percentile, and the pointing settles at 12° mean with 74 % of sunlit time within 15°. Without
albedo the same sensors give 1.0° mean and 3.2° worst p95, and the pointing is the generic model's. The cause is
geometric: in sun-pointing the off-axis component of the sun vector is read by the ±X and ±Z photodiodes at 85–90°
incidence, a few hundredths of full sun, while the face that looks at the sunlit Earth receives up to a quarter of full
sun from it; differencing opposite faces cannot remove it, and the spin carries the albedo direction round with the
Sun's, so no averaging separates them. The wings' shadow on the ±X photodiodes and the grazing loss matter little (the
"no response beyond 85°" row). The generic 5° white error of the earlier rows is a test of the control law's
tolerance, not a sensor model.

**Decision C-23 (chair, 2026-10-27).** The pointing requirement exists for power, so ADCS-P002 is judged by power, in
every trial with the photodiode-and-albedo model: the sunlit-averaged generation lost to the pointing at most 5 %
(against the same instants with +Y held on the Sun, the Earth turned with it, `Trial::pointing_loss`) and the Sun
within 30° of +Y for at least 95 % of sunlit time (`Trial::within_30`). Sun-vector knowledge is not a requirement
(ADCS-P003 withdrawn; ADCS-P007, the rate, stays: the controller needs it). If TP-ADCS-003 §3.1 on the EM shows the
photodiodes worse than this model, the recorded option is an albedo-immune sun sensor on +Y (a slit or digital type,
`hardware/adcs/SENSOR_DESIGN.md` §3.1). Mean / worst trial, and the trials of 40 that fail (loss, within 30°):

| Case (photodiodes and albedo unless stated) | Loss of sunlit generation | Sun within 30° of +Y | Trials failing | Orbit-average generation (mean / worst trial) |
|---------------------------------------------|---------------------------|----------------------|----------------|-----------------------------------------------|
| Both wings, flight | 2.0 % / 3.2 % | 100 % / 100 % | 0 / 0 | 4.56 / 4.41 W (+Y held on the Sun: 4.65 W) |
| CAD tensor CBE / MEV | 2.3 % / 3.6 % · 1.9 % / 2.9 % | 100 % / 100 % | 0 / 0 | 4.55 / 4.57 W |
| No response beyond 85° | 2.1 % / 3.2 % | 100 % / 100 % | 0 / 0 | 4.56 W |
| BMI088, bias 0.2°/s | 2.1 % / 3.5 % | 100 % / 100 % | 0 / 0 | 4.56 W |
| Gating | 1.8 % / 2.7 % | 100 % / 100 % | 0 / 0 | 4.57 W |
| AntS 3 h after the wings, gating | 1.8 % / 2.7 % | 100 % / 100 % | 0 / 0 | 4.57 W |
| Gating, residual dipole 5 mA·m² | 2.1 % / 3.7 % | 100 % / 99.98 % | 0 / 0 | 4.56 / 4.40 W |
| Gating, residual dipole 7 mA·m² | 2.5 % / 5.0 % | 99.7 % / 96.3 % | 0 / 0 | 4.54 / 4.35 W |
| **Residual dipole 10 mA·m² (the limit)** | **3.2 % / 7.7 %** | **98.5 % / 86.9 %** | **6 / 6** | 4.50 / 4.27 W |
| **Gating, residual dipole 10 mA·m²** | **3.3 % / 7.6 %** | **97.9 % / 84.9 %** | **6 / 6** | 4.50 / 4.26 W |
| W1 only, preset 0 (mean / CBE / MEV) | 0.9 % / 1.6 % | 100 % / 99.97 % | 0 / 0 | 3.11 / 3.01 W (+Y on the Sun: 3.14 W) |
| W1 only, preset 0, gating | 0.8 % / 1.4 % | 99.6 % / 99.1 % | 0 / 0 | 3.11 W |
| W2 only, preset 0 (mean / CBE / MEV) | 1.2 % / 2.0 % | 100 % / 99.93 % | 0 / 0 | 3.10 / 2.99 W |
| W1 only, preset 1 | below +Y on the Sun (−1.3 %) | 100 % / 100 % | 0 / 0 | 3.18 W |
| Wings and AntS stowed, preset 0; after the AntS at 25 h | below +Y on the Sun (−1.3 %, −0.7 %) | 100 % / 100 % | 0 / 0 | 1.60 / 1.63 W |
| No gyro (a contingency, not the requirement's case) | 17 % / 23 % | 39 % / 19 % | 40 / 40 | 3.85 W |
| Generic sun-sensor error, gating, 10 mA·m² (for comparison) | 0.5 % / 1.8 % | 99.9 % / 99.4 % | 0 / 0 | 4.63 W |

**ADCS-P002 is met in every trial at the 3 mA·m² design residual dipole, with gating, with one wing at preset 0 and
with the wings stowed, and up to 7 mA·m² (worst trial 5.0 % and 96.3 %). It is not met at the 10 mA·m² limit of
BASELINE §8: 6 of 40 trials fail both parts, the worst by 2.7 points of loss (7.7 % against 5 %) and 10.1 points of
time within 30° (84.9 % against 95 %, with gating).** The residual-dipole limit consistent with ADCS-P002 is 7 mA·m²
with the photodiodes (the flight unit's Helmholtz-cage measurement, BASELINE §8, decides it), or an albedo-immune +Y
sensor restores 10 mA·m² (the generic-error row: 1.8 % and 99.4 %). A negative loss means the attitude the law holds
gives more than +Y on the Sun would (the stowed wings' outward cells, or the one-wing preset that tilts the Sun
toward the remaining wing). Without a gyro the photodiodes are unusable for pointing (17 % lost, 39 % within 30°).

### 11.7 Identifying the Deployed Wing (decision C-21)

With one wing out, `DEP` says one wing and `PA` gives the total power; the ground needs which. Two signatures separate
W1 from W2 under preset 0, both from telemetry the spacecraft already has. The flight software engineer writes the
procedure into the commissioning plan from these numbers.

**A. Array power against the sun-sensor phase over a spin (the chair's method).** With one wing out the asymmetry is on
the ±X faces: the stowed wing's front string faces outward on its side (W2 stowed: string W2F faces −X), while the
deployed wing shadows the body string on its own side (W1 out: PX dark for the Sun 10–61° from +Y). Over one turn of
the spin about +Y the array power therefore peaks when the Sun is toward the stowed wing's face. The ±X photodiodes
are dark or shadowed in exactly this case, so the phase reference is the +Z photodiode: with the spin positive about
+Y the Sun's azimuth about +Y advances at the spin rate. The ground fits, over three turns at 8°/s (135 s of 1 Hz
samples of `PA`, the sun vector and `W`), y = c + a cos ωt + b sin ωt to the +Z channel and to `PA`, with ω from the
gyro's y rate, and takes the lag of the power's harmonic behind the +Z channel's (atan2(b, a) of `PA` less that of +Z,
wrapped to ±180°). Expected, from `adcs_mc::wing_id` (500 draws per row: random spin phase and nadir direction, the
photodiode model above with its own gains each draw, the M-G370 bias, `PA` noise 20 mW + 1 % at 1σ, the array at
commissioning behind the MPPT's 0.95; `wing_id.csv` of the script). Lag mean / σ / worst departure from the mean,
for albedo on a nadir plate of 0, 100 and 300 W/m²:

| Sun from +Y | W1 only, 0 / 100 / 300 W/m² | W2 only, 0 / 100 / 300 W/m² | Amplitude, least draw (one wing) | Amplitude, both wings or neither |
|-------------|------------------------------|------------------------------|----------------------------------|----------------------------------|
| 15° | +90° / 3° / 10° · +90° / 13° / 27° · +89° / 50° / 175° | −90° / 3° / 13° · −90° / 13° / 27° · −91° / 50° / 178° | 0.13 W | ≤ 0.015 W |
| 20° | +90° / 2° / 6° · +90° / 9° / 18° · +90° / 29° / 59° | −90° / 2° / 9° · −90° / 9° / 18° · −90° / 29° / 56° | 0.22 W | ≤ 0.015 W |
| 30° | +90° / 1° / 3° · +90° / 5° / 11° · +90° / 16° / 30° | −90° / 1° / 5° · −90° / 5° / 11° · −90° / 16° / 29° | 0.43 W | ≤ 0.016 W |
| 45° | +90° / 1° / 3° · +90° / 3° / 7° · +90° / 11° / 19° | −90° / 1° / 4° · −90° / 3° / 7° · −90° / 11° / 19° | 0.64 W | ≤ 0.015 W |
| 60° | +90° / 1° / 3° · +90° / 3° / 7° · +90° / 9° / 16° | −90° / 1° / 4° · −90° / 3° / 8° · −90° / 9° / 16° | 0.56 W | ≤ 0.020 W |
| 70° | +90° / 1° / 4° · +90° / 3° / 8° · +90° / 8° / 16° | −90° / 1° / 5° · −90° / 3° / 8° · −90° / 8° / 16° | 0.41 W | ≤ 0.018 W |

*Pass criterion:* with the Sun 30–70° from +Y (from the sun vector), a harmonic amplitude of at least 0.2 W and the lag
within ±45° of +90° names **W1**; within ±45° of −90° names **W2**; an amplitude below 0.2 W means no one-wing
asymmetry (both wings or neither give at most 0.02 W); anything else, repeat. Of 3,000 draws per wing at 30–70° every
one was named correctly: the worst lag was 30° from its signature at 300 W/m² (15° inside the boundary, about 2.7σ) and
11° at 100 W/m² (34° inside, about 6σ); the least amplitude, 0.41 W, is twice the threshold and twenty times what both
wings or neither give. Taking the record with the sub-satellite point near the terminator (the minutes before eclipse
entry) keeps the albedo near 100 W/m² or below. Below 30° the margin goes (at 15° and 300 W/m² one draw was 175° off).
*The condition is the hard part:* steady sun-pointing under preset 0 holds the Sun 6–7° from +Y, so the 30–70° window
exists only during a capture (after the detumble, or after a commanded DETUMBLE and the automatic return to
SUNPOINT). The Monte Carlo gives a median of 3–8 min per capture with the Sun 20–55° from +Y and the spin at 6°/s or
more, and none in some trials. The procedure therefore needs a 1 Hz record of `PA`, the sun vector and `W` through a
capture, and may need more than one capture.

**B. The body-rate tilt in steady state (no special attitude; housekeeping `W`).** Under preset 0 the law holds the spin
near +Y, but the one-wing product of inertia (Ixy −162 or +184 × 10⁻⁶ kg·m²) tilts the momentum toward the major axis,
and the mean body rate leans with it: atan2(W̄x, W̄y) over at least one sunlit orbit of housekeeping is **−3.5 to −3.8°
for W1 only** (every trial −4.5 to −3.0° over the mean, CBE and MEV plants; with gating −4.9 to −3.5°) and **+3.4° for W2
only** (+2.7 to +4.1°), against +0.1° (−0.6 to +0.7°) with both wings. The M-G370's 0.1°/s 1σ bias is in the trials
(about 0.7° of the spread). *Pass criterion:* a tilt of −1.5° or less names W1, +1.5° or more names W2, between them no
identification; the nearest trial is 1.2° from its threshold. It needs only housekeeping and works at any time, so it
is the primary method and A the confirmation; both should agree before preset 1 or 2 is sent. *Check after the
preset:* with the right preset the tilt moves to −8.0° (W1, preset 1; trials −8.9 to −7.3°) or +8.9° (W2, preset 2;
+8.0 to +9.5°); the wrong one gives +2.8° or −1.9° (and 14–17° pointing). A tilt more than 2° from the preset's axis
(−8.4° or +9.1°) after one day means the wrong preset: send 0.

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
| 1.8 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, fourth part) | §11.2 the tensors of MASS_BUDGET 3.2 §6.5 (deployed (3.064, 4.481, 4.262) × 10⁻³, the presets' major axes 8.4°, 9.1°, 3.6°); §11.4 the M-G370, CAD-plant and preset rows re-run with them (RID GNC-07) |
| 1.9 | 2026-10-27 | SMART-QSO Team (CDR round 3 closure, GN&C) | Findings P3F-G-04 to -07 and decisions C-21, C-22, C-23: §2.1, §2.2 ADCS-F002 and ADCS-P002 restated for power (≤ 5 % of the sunlit generation lost to the pointing and the Sun within 30° of +Y ≥ 95 % of sunlit time, every trial, photodiodes and albedo), ADCS-F003 the sun vector and rate the law needs, ADCS-P003 withdrawn, ADCS-P007 rate knowledge (≤ 0.2°/s); ADCS-I003, §3.1, §7 no quaternion; §4.3 the EKF/TRIAD removed from the flight baseline (TRIAD kept as a ground tool); §8.1 F003 method and status; §11.2 stowed products and the AntS-stowed tensor; §11.3 preset axes regenerated from the code (8.4°, 9.1°, 3.6°) and the C-21 rule; §11.4 the gain sweep and the robustness table re-run with the current tensors (C-4 rationale revised: no gain holds Iyy = Izz; 8°/s and 3·10⁻⁴ stand on the 1.005–1.05 range), the proposed BASELINE §8 text; §11.6 new: gating, wings stowed with the AntS held 25 h, the AntS 3 h after the wings, one wing at CBE/MEV with the wrong preset, the face-photodiode sun sensors with albedo, the sun-vector knowledge (reported) and ADCS-P002 per trial (met at 3–7 mA·m², not at the 10 mA·m² limit: 6 of 40 trials); §11.7 new: the wing-identification signatures and pass criteria |
