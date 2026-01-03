/**
 * @file adcs_control.c
 * @brief SMART-QSO ADCS Control Module Implementation
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#include "adcs_control.h"
#include "fault_mgmt.h"

#include <stdio.h>
#include <string.h>
#include <math.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** ADCS module state */
static AdcsState_t s_adcs_state;

/** Previous magnetometer reading for rate estimation */
static MagData_t s_mag_prev;

/** Module initialized flag */
static bool s_initialized = false;

/** Simulated sensor values (for testing) */
static Vec3_t s_sim_mag = {30.0, 0.0, 40.0};  /* Typical LEO field */
static Vec3_t s_sim_sun = {1.0, 0.0, 0.0};
static Vec3_t s_sim_rate = {0.05, 0.05, 0.05}; /* ~3 deg/s initial */
static bool s_sim_mode = true;

/*===========================================================================*/
/* Utility Functions                                                          */
/*===========================================================================*/

double vec3_magnitude(const Vec3_t *v)
{
    if (v == NULL) {
        return 0.0;
    }
    return sqrt(v->x * v->x + v->y * v->y + v->z * v->z);
}

void vec3_normalize(Vec3_t *v)
{
    if (v == NULL) {
        return;
    }

    double mag = vec3_magnitude(v);
    if (mag > 1e-10) {
        v->x /= mag;
        v->y /= mag;
        v->z /= mag;
    }
}

void vec3_cross(const Vec3_t *a, const Vec3_t *b, Vec3_t *result)
{
    if (a == NULL || b == NULL || result == NULL) {
        return;
    }

    result->x = a->y * b->z - a->z * b->y;
    result->y = a->z * b->x - a->x * b->z;
    result->z = a->x * b->y - a->y * b->x;
}

double vec3_dot(const Vec3_t *a, const Vec3_t *b)
{
    if (a == NULL || b == NULL) {
        return 0.0;
    }

    return a->x * b->x + a->y * b->y + a->z * b->z;
}

void adcs_saturate_dipole(Vec3_t *dipole, double max_dipole)
{
    if (dipole == NULL) {
        return;
    }

    double mag = vec3_magnitude(dipole);
    if (mag > max_dipole && mag > 1e-10) {
        double scale = max_dipole / mag;
        dipole->x *= scale;
        dipole->y *= scale;
        dipole->z *= scale;
    }
}

/*===========================================================================*/
/* ADCS Functions                                                             */
/*===========================================================================*/

SmartQsoResult_t adcs_init(void)
{
    memset(&s_adcs_state, 0, sizeof(s_adcs_state));
    memset(&s_mag_prev, 0, sizeof(s_mag_prev));

    s_adcs_state.mode = ADCS_MODE_IDLE;
    s_adcs_state.detumble_complete = false;

    s_initialized = true;

    printf("[ADCS] Initialized in IDLE mode\n");

    return SMART_QSO_OK;
}

SmartQsoResult_t adcs_update(uint64_t now_ms)
{
    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    /* Read sensors */
    adcs_read_magnetometer(&s_adcs_state.mag);
    adcs_read_sun_sensors(&s_adcs_state.sun);

    /* Estimate angular rate from B-dot (simplified) */
    if (s_mag_prev.valid && s_adcs_state.mag.valid) {
        double dt = (double)(now_ms - s_mag_prev.timestamp_ms) / 1000.0;
        if (dt > 0.01) {
            /* Rate estimation from magnetic field rotation */
            double B_mag = vec3_magnitude(&s_adcs_state.mag.field);
            if (B_mag > 1.0) {
                Vec3_t B_dot;
                B_dot.x = (s_adcs_state.mag.field.x - s_mag_prev.field.x) / dt;
                B_dot.y = (s_adcs_state.mag.field.y - s_mag_prev.field.y) / dt;
                B_dot.z = (s_adcs_state.mag.field.z - s_mag_prev.field.z) / dt;

                /* Store rate for telemetry */
                s_adcs_state.mag.field_rate = B_dot;

                /* Estimate angular rate (simplified: omega ~ B_dot / |B|) */
                s_adcs_state.angular_rate_est.x = B_dot.x / B_mag;
                s_adcs_state.angular_rate_est.y = B_dot.y / B_mag;
                s_adcs_state.angular_rate_est.z = B_dot.z / B_mag;
            }
        }
    }

    /* Save current reading for next iteration */
    s_mag_prev = s_adcs_state.mag;

    /* Calculate rate magnitude */
    s_adcs_state.rate_magnitude = vec3_magnitude(&s_adcs_state.angular_rate_est);

    /* Mode-specific control */
    Vec3_t dipole = {0.0, 0.0, 0.0};

    switch (s_adcs_state.mode) {
        case ADCS_MODE_DETUMBLE:
            /* Check for detumble timeout */
            if (s_adcs_state.detumble_start_ms > 0 &&
                (now_ms - s_adcs_state.detumble_start_ms) > ADCS_DETUMBLE_TIMEOUT_MS) {
                fault_log_add(FAULT_TYPE_ADCS, FAULT_SEVERITY_WARNING,
                              "Detumble timeout", 0.0);
                s_adcs_state.detumble_complete = true;
                s_adcs_state.mode = ADCS_MODE_IDLE;
                break;
            }

            /* B-dot control */
            adcs_compute_bdot(&s_adcs_state.mag, &dipole);

            /* Check for rate convergence */
            if (s_adcs_state.rate_magnitude < ADCS_DETUMBLE_RATE_THRESHOLD) {
                if (s_adcs_state.settling_start_ms == 0) {
                    s_adcs_state.settling_start_ms = now_ms;
                } else if ((now_ms - s_adcs_state.settling_start_ms) > ADCS_DETUMBLE_SETTLING_MS) {
                    s_adcs_state.detumble_complete = true;
                    s_adcs_state.mode = ADCS_MODE_IDLE;
                    printf("[ADCS] Detumble complete!\n");
                }
            } else {
                s_adcs_state.settling_start_ms = 0;
            }
            break;

        case ADCS_MODE_SUNPOINT:
            if (s_adcs_state.sun.sun_visible) {
                adcs_compute_sunpoint(&s_adcs_state.sun, &s_adcs_state.mag, &dipole);
            } else {
                /* No sun - enter eclipse mode */
                s_adcs_state.mode = ADCS_MODE_ECLIPSE;
            }
            break;

        case ADCS_MODE_ECLIPSE:
            /* Drift mode - no control */
            if (s_adcs_state.sun.sun_visible) {
                s_adcs_state.mode = ADCS_MODE_SUNPOINT;
            }
            break;

        case ADCS_MODE_SAFE:
        case ADCS_MODE_IDLE:
        default:
            /* No active control */
            break;
    }

    /* Apply control if in active mode */
    if (s_adcs_state.mode == ADCS_MODE_DETUMBLE ||
        s_adcs_state.mode == ADCS_MODE_SUNPOINT) {
        adcs_apply_dipole(&dipole);
    } else {
        adcs_disable_mtq();
    }

    s_adcs_state.control_cycles++;

    return SMART_QSO_OK;
}

SmartQsoResult_t adcs_set_mode(AdcsMode_t mode)
{
    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    AdcsMode_t prev_mode = s_adcs_state.mode;
    s_adcs_state.mode = mode;

    if (mode == ADCS_MODE_DETUMBLE && prev_mode != ADCS_MODE_DETUMBLE) {
        s_adcs_state.detumble_start_ms = smart_qso_now_ms();
        s_adcs_state.settling_start_ms = 0;
        s_adcs_state.detumble_complete = false;
        printf("[ADCS] Starting detumble sequence\n");
    }

    printf("[ADCS] Mode changed: %d -> %d\n", prev_mode, mode);

    return SMART_QSO_OK;
}

AdcsMode_t adcs_get_mode(void)
{
    if (!s_initialized) {
        return ADCS_MODE_SAFE;
    }

    return s_adcs_state.mode;
}

SmartQsoResult_t adcs_get_state(AdcsState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    memcpy(state, &s_adcs_state, sizeof(AdcsState_t));
    return SMART_QSO_OK;
}

SmartQsoResult_t adcs_get_telemetry(AdcsTelemetry_t *telemetry)
{
    if (telemetry == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    telemetry->mode = (uint8_t)s_adcs_state.mode;

    /* Magnetic field (0.1 uT resolution) */
    telemetry->mag_x = (int16_t)(s_adcs_state.mag.field.x * 10.0);
    telemetry->mag_y = (int16_t)(s_adcs_state.mag.field.y * 10.0);
    telemetry->mag_z = (int16_t)(s_adcs_state.mag.field.z * 10.0);

    /* Sun vector (0.01 unit resolution) */
    telemetry->sun_x = (int16_t)(s_adcs_state.sun.sun_vector.x * 100.0);
    telemetry->sun_y = (int16_t)(s_adcs_state.sun.sun_vector.y * 100.0);
    telemetry->sun_z = (int16_t)(s_adcs_state.sun.sun_vector.z * 100.0);

    /* Angular rate (0.01 deg/s resolution) */
    double rad_to_deg = 180.0 / 3.14159265359;
    telemetry->rate_x = (int16_t)(s_adcs_state.angular_rate_est.x * rad_to_deg * 100.0);
    telemetry->rate_y = (int16_t)(s_adcs_state.angular_rate_est.y * rad_to_deg * 100.0);
    telemetry->rate_z = (int16_t)(s_adcs_state.angular_rate_est.z * rad_to_deg * 100.0);

    /* Magnetorquer commands */
    telemetry->mtq_x = s_adcs_state.mtq.pwm[0];
    telemetry->mtq_y = s_adcs_state.mtq.pwm[1];
    telemetry->mtq_z = s_adcs_state.mtq.pwm[2];

    /* Status flags */
    telemetry->sun_visible = s_adcs_state.sun.sun_visible ? 1 : 0;
    telemetry->detumble_done = s_adcs_state.detumble_complete ? 1 : 0;

    return SMART_QSO_OK;
}

bool adcs_is_detumbled(void)
{
    if (!s_initialized) {
        return false;
    }

    return s_adcs_state.detumble_complete;
}

bool adcs_sun_visible(void)
{
    if (!s_initialized) {
        return false;
    }

    return s_adcs_state.sun.sun_visible;
}

double adcs_get_rate_magnitude(void)
{
    if (!s_initialized) {
        return 0.0;
    }

    return s_adcs_state.rate_magnitude;
}

/*===========================================================================*/
/* Sensor Functions                                                           */
/*===========================================================================*/

SmartQsoResult_t adcs_read_magnetometer(MagData_t *data)
{
    if (data == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (s_sim_mode) {
        /* Use simulated values */
        data->field = s_sim_mag;
        data->timestamp_ms = smart_qso_now_ms();
        data->valid = true;

        /* Simulate slow field rotation (spacecraft tumbling) */
        /* This would come from actual hardware in flight */
    } else {
        /* TODO: Read from actual I2C magnetometer */
        return SMART_QSO_ERROR;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t adcs_read_sun_sensors(SunSensorData_t *data)
{
    if (data == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (s_sim_mode) {
        /* Use simulated values */
        data->sun_vector = s_sim_sun;
        vec3_normalize(&data->sun_vector);
        data->intensity = vec3_magnitude(&s_sim_sun);
        data->sun_visible = (data->intensity > 0.1);
        data->timestamp_ms = smart_qso_now_ms();

        /* Simulate raw ADC values */
        for (int i = 0; i < ADCS_SUN_SENSOR_COUNT; i++) {
            data->raw[i] = (uint16_t)(1000.0 * data->intensity);
        }
    } else {
        /* TODO: Read from actual ADC sun sensors */
        return SMART_QSO_ERROR;
    }

    return SMART_QSO_OK;
}

void adcs_set_sim_sensors(const Vec3_t *mag, const Vec3_t *sun, const Vec3_t *rate)
{
    if (mag != NULL) {
        s_sim_mag = *mag;
    }
    if (sun != NULL) {
        s_sim_sun = *sun;
    }
    if (rate != NULL) {
        s_sim_rate = *rate;
    }
}

/*===========================================================================*/
/* Control Functions                                                          */
/*===========================================================================*/

void adcs_compute_bdot(const MagData_t *mag, Vec3_t *dipole)
{
    if (mag == NULL || dipole == NULL) {
        return;
    }

    if (!mag->valid) {
        dipole->x = 0.0;
        dipole->y = 0.0;
        dipole->z = 0.0;
        return;
    }

    /* B-dot control law: m = -k * B_dot */
    double k = ADCS_BDOT_GAIN;

    dipole->x = -k * mag->field_rate.x * 1e-6;  /* Convert uT to T */
    dipole->y = -k * mag->field_rate.y * 1e-6;
    dipole->z = -k * mag->field_rate.z * 1e-6;

    /* Saturate to maximum dipole */
    adcs_saturate_dipole(dipole, ADCS_MAX_DIPOLE);
}

void adcs_compute_sunpoint(const SunSensorData_t *sun,
                           const MagData_t *mag,
                           Vec3_t *dipole)
{
    if (sun == NULL || mag == NULL || dipole == NULL) {
        return;
    }

    if (!sun->sun_visible || !mag->valid) {
        dipole->x = 0.0;
        dipole->y = 0.0;
        dipole->z = 0.0;
        return;
    }

    /* Desired sun vector (pointing +X at sun for max solar power) */
    Vec3_t sun_desired = {1.0, 0.0, 0.0};

    /* Error vector (cross product gives rotation axis) */
    Vec3_t error;
    vec3_cross(&sun->sun_vector, &sun_desired, &error);

    /* Desired torque (proportional to error) */
    Vec3_t torque;
    torque.x = ADCS_SUNPOINT_KP * error.x;
    torque.y = ADCS_SUNPOINT_KP * error.y;
    torque.z = ADCS_SUNPOINT_KP * error.z;

    /* Convert torque to dipole using: m = (tau x B) / |B|^2 */
    Vec3_t B = mag->field;
    double B_mag_sq = vec3_dot(&B, &B);

    if (B_mag_sq > 1.0) {  /* Prevent divide by zero */
        Vec3_t tau_cross_B;
        vec3_cross(&torque, &B, &tau_cross_B);

        dipole->x = tau_cross_B.x / B_mag_sq * 1e6;  /* Convert from T to uT */
        dipole->y = tau_cross_B.y / B_mag_sq * 1e6;
        dipole->z = tau_cross_B.z / B_mag_sq * 1e6;
    } else {
        dipole->x = 0.0;
        dipole->y = 0.0;
        dipole->z = 0.0;
    }

    /* Saturate to maximum dipole */
    adcs_saturate_dipole(dipole, ADCS_MAX_DIPOLE);
}

SmartQsoResult_t adcs_apply_dipole(const Vec3_t *dipole)
{
    if (dipole == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    /* Store commanded dipole */
    s_adcs_state.mtq.dipole = *dipole;
    s_adcs_state.mtq.enabled = true;

    /* Convert dipole to PWM (-100 to +100) */
    s_adcs_state.mtq.pwm[0] = (int8_t)(dipole->x / ADCS_MAX_DIPOLE * 100.0);
    s_adcs_state.mtq.pwm[1] = (int8_t)(dipole->y / ADCS_MAX_DIPOLE * 100.0);
    s_adcs_state.mtq.pwm[2] = (int8_t)(dipole->z / ADCS_MAX_DIPOLE * 100.0);

    /* Clamp PWM values */
    for (int i = 0; i < 3; i++) {
        if (s_adcs_state.mtq.pwm[i] > 100) {
            s_adcs_state.mtq.pwm[i] = 100;
        }
        if (s_adcs_state.mtq.pwm[i] < -100) {
            s_adcs_state.mtq.pwm[i] = -100;
        }
    }

    /* TODO: Apply via HAL GPIO/PWM */

    return SMART_QSO_OK;
}

SmartQsoResult_t adcs_disable_mtq(void)
{
    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    s_adcs_state.mtq.dipole.x = 0.0;
    s_adcs_state.mtq.dipole.y = 0.0;
    s_adcs_state.mtq.dipole.z = 0.0;
    s_adcs_state.mtq.pwm[0] = 0;
    s_adcs_state.mtq.pwm[1] = 0;
    s_adcs_state.mtq.pwm[2] = 0;
    s_adcs_state.mtq.enabled = false;

    /* TODO: Disable via HAL GPIO */

    return SMART_QSO_OK;
}
