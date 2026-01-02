/**
 * @file adcs_control.h
 * @brief SMART-QSO ADCS Control Module
 *
 * Manages attitude determination and control using magnetorquers.
 *
 * @requirement SRS-F070 Implement B-dot detumble algorithm
 * @requirement SRS-F071 Achieve detumble within 1 orbit
 * @requirement SRS-F072 Implement sun-pointing control
 * @requirement SRS-F073 Provide attitude telemetry
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 * @note Following GeneSat/PharmaSat/O/OREOS/EcAMSat ADCS patterns
 */

#ifndef SMART_QSO_ADCS_CONTROL_H
#define SMART_QSO_ADCS_CONTROL_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* ADCS Configuration Constants                                               */
/*===========================================================================*/

/** Maximum magnetic dipole moment per axis (A*m^2) */
#define ADCS_MAX_DIPOLE         0.2

/** B-dot control gain */
#define ADCS_BDOT_GAIN          5.0e6

/** Sun-pointing proportional gain */
#define ADCS_SUNPOINT_KP        0.001

/** Detumble rate threshold (rad/s) */
#define ADCS_DETUMBLE_RATE_THRESHOLD   0.0087  /* ~0.5 deg/s */

/** Detumble settling time (ms) */
#define ADCS_DETUMBLE_SETTLING_MS      300000  /* 5 minutes */

/** Maximum detumble duration (ms) */
#define ADCS_DETUMBLE_TIMEOUT_MS       5400000 /* 90 minutes (1 orbit) */

/** Control loop period (ms) */
#define ADCS_CONTROL_PERIOD_MS         1000

/** Magnetometer sample period (ms) */
#define ADCS_MAG_SAMPLE_PERIOD_MS      100

/** Number of sun sensors */
#define ADCS_SUN_SENSOR_COUNT          6

/*===========================================================================*/
/* ADCS Types                                                                 */
/*===========================================================================*/

/**
 * @brief 3D vector structure
 */
typedef struct {
    double x;
    double y;
    double z;
} Vec3_t;

/**
 * @brief ADCS operating mode
 */
typedef enum {
    ADCS_MODE_IDLE     = 0,  /**< Sensors on, actuators off */
    ADCS_MODE_DETUMBLE = 1,  /**< B-dot detumble active */
    ADCS_MODE_SUNPOINT = 2,  /**< Sun-pointing active */
    ADCS_MODE_ECLIPSE  = 3,  /**< Eclipse mode (drift) */
    ADCS_MODE_SAFE     = 4   /**< Safe mode (minimal operation) */
} AdcsMode_t;

/**
 * @brief Magnetometer data structure
 */
typedef struct {
    Vec3_t field;            /**< Magnetic field (uT) */
    Vec3_t field_rate;       /**< Field rate of change (uT/s) */
    uint64_t timestamp_ms;   /**< Measurement timestamp */
    bool valid;              /**< Data validity flag */
} MagData_t;

/**
 * @brief Sun sensor data structure
 */
typedef struct {
    uint16_t raw[ADCS_SUN_SENSOR_COUNT];  /**< Raw ADC values */
    Vec3_t sun_vector;       /**< Computed sun unit vector */
    double intensity;        /**< Total intensity */
    bool sun_visible;        /**< Sun visibility flag */
    uint64_t timestamp_ms;   /**< Measurement timestamp */
} SunSensorData_t;

/**
 * @brief Magnetorquer command structure
 */
typedef struct {
    Vec3_t dipole;           /**< Commanded dipole moment (A*m^2) */
    int8_t pwm[3];           /**< PWM duty cycle (-100 to +100) */
    bool enabled;            /**< Actuators enabled flag */
} MtqCommand_t;

/**
 * @brief ADCS state structure
 */
typedef struct {
    AdcsMode_t mode;         /**< Current operating mode */
    MagData_t mag;           /**< Magnetometer data */
    SunSensorData_t sun;     /**< Sun sensor data */
    MtqCommand_t mtq;        /**< Magnetorquer command */
    Vec3_t angular_rate_est; /**< Estimated angular rate (rad/s) */
    double rate_magnitude;   /**< Rate magnitude (rad/s) */
    uint64_t detumble_start_ms; /**< Detumble start time */
    uint64_t settling_start_ms; /**< Rate settling start time */
    bool detumble_complete;  /**< Detumble completion flag */
    uint32_t control_cycles; /**< Control loop iterations */
    uint32_t fault_count;    /**< Fault counter */
} AdcsState_t;

/**
 * @brief ADCS telemetry structure
 */
typedef struct {
    uint8_t mode;            /**< Current mode */
    int16_t mag_x;           /**< Magnetic field X (0.1 uT) */
    int16_t mag_y;           /**< Magnetic field Y (0.1 uT) */
    int16_t mag_z;           /**< Magnetic field Z (0.1 uT) */
    int16_t sun_x;           /**< Sun vector X (0.01 unit) */
    int16_t sun_y;           /**< Sun vector Y (0.01 unit) */
    int16_t sun_z;           /**< Sun vector Z (0.01 unit) */
    int16_t rate_x;          /**< Angular rate X (0.01 deg/s) */
    int16_t rate_y;          /**< Angular rate Y (0.01 deg/s) */
    int16_t rate_z;          /**< Angular rate Z (0.01 deg/s) */
    int8_t mtq_x;            /**< MTQ X duty cycle */
    int8_t mtq_y;            /**< MTQ Y duty cycle */
    int8_t mtq_z;            /**< MTQ Z duty cycle */
    uint8_t sun_visible;     /**< Sun visibility */
    uint8_t detumble_done;   /**< Detumble status */
} AdcsTelemetry_t;

/*===========================================================================*/
/* ADCS Functions                                                             */
/*===========================================================================*/

/**
 * @brief Initialize ADCS module
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_init(void);

/**
 * @brief Run ADCS control loop iteration
 *
 * @param now_ms Current time in milliseconds
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_update(uint64_t now_ms);

/**
 * @brief Set ADCS operating mode
 *
 * @param mode Target mode
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_set_mode(AdcsMode_t mode);

/**
 * @brief Get current ADCS mode
 *
 * @return Current ADCS mode
 */
AdcsMode_t adcs_get_mode(void);

/**
 * @brief Get ADCS state
 *
 * @param state Pointer to state structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_get_state(AdcsState_t *state);

/**
 * @brief Get ADCS telemetry
 *
 * @param telemetry Pointer to telemetry structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_get_telemetry(AdcsTelemetry_t *telemetry);

/**
 * @brief Check if detumble is complete
 *
 * @return true if detumble is complete
 */
bool adcs_is_detumbled(void);

/**
 * @brief Check if sun is visible
 *
 * @return true if sun is detected
 */
bool adcs_sun_visible(void);

/**
 * @brief Get estimated angular rate magnitude
 *
 * @return Rate magnitude in rad/s
 */
double adcs_get_rate_magnitude(void);

/*===========================================================================*/
/* Sensor Functions                                                           */
/*===========================================================================*/

/**
 * @brief Read magnetometer
 *
 * @param data Pointer to magnetometer data structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_read_magnetometer(MagData_t *data);

/**
 * @brief Read sun sensors
 *
 * @param data Pointer to sun sensor data structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_read_sun_sensors(SunSensorData_t *data);

/**
 * @brief Set simulated sensor values (for testing)
 *
 * @param mag Magnetic field vector (uT)
 * @param sun Sun vector (unit vector)
 * @param rate Angular rate (rad/s)
 */
void adcs_set_sim_sensors(const Vec3_t *mag, const Vec3_t *sun, const Vec3_t *rate);

/*===========================================================================*/
/* Control Functions                                                          */
/*===========================================================================*/

/**
 * @brief Compute B-dot detumble control
 *
 * @param mag Current magnetometer data
 * @param dipole Output dipole command
 */
void adcs_compute_bdot(const MagData_t *mag, Vec3_t *dipole);

/**
 * @brief Compute sun-pointing control
 *
 * @param sun Sun sensor data
 * @param mag Magnetometer data
 * @param dipole Output dipole command
 */
void adcs_compute_sunpoint(const SunSensorData_t *sun,
                           const MagData_t *mag,
                           Vec3_t *dipole);

/**
 * @brief Apply dipole command to magnetorquers
 *
 * @param dipole Dipole command
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_apply_dipole(const Vec3_t *dipole);

/**
 * @brief Disable all magnetorquers
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t adcs_disable_mtq(void);

/*===========================================================================*/
/* Utility Functions                                                          */
/*===========================================================================*/

/**
 * @brief Compute vector magnitude
 *
 * @param v Input vector
 * @return Magnitude
 */
double vec3_magnitude(const Vec3_t *v);

/**
 * @brief Normalize vector in place
 *
 * @param v Vector to normalize
 */
void vec3_normalize(Vec3_t *v);

/**
 * @brief Compute cross product: result = a x b
 *
 * @param a First vector
 * @param b Second vector
 * @param result Output vector
 */
void vec3_cross(const Vec3_t *a, const Vec3_t *b, Vec3_t *result);

/**
 * @brief Compute dot product
 *
 * @param a First vector
 * @param b Second vector
 * @return Dot product
 */
double vec3_dot(const Vec3_t *a, const Vec3_t *b);

/**
 * @brief Saturate dipole to maximum value
 *
 * @param dipole Dipole vector
 * @param max_dipole Maximum dipole magnitude
 */
void adcs_saturate_dipole(Vec3_t *dipole, double max_dipole);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_ADCS_CONTROL_H */
