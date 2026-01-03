/**
 * @file system_state.h
 * @brief System state management interface for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module encapsulates all global system state per NASA NPR 7150.2
 * modularity requirements. All state access is through accessor functions
 * to ensure thread-safety and proper isolation.
 *
 * @requirement SRS-MOD-001 All system state shall be encapsulated
 * @requirement SRS-MOD-002 State access shall be through accessor functions
 * @requirement SRS-MOD-003 State modifications shall be atomic
 *
 * Thread Safety:
 * - All accessors are designed to be interrupt-safe
 * - State modifications use critical sections where needed
 * - Read operations are atomic for single-word values
 */

#ifndef SMART_QSO_SYSTEM_STATE_H
#define SMART_QSO_SYSTEM_STATE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include "state_machine.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** Maximum number of sensors supported */
#define SYS_MAX_SENSORS             32U

/** Maximum fault log entries */
#define SYS_MAX_FAULT_ENTRIES       100U

/** Telemetry history depth */
#define SYS_TELEMETRY_HISTORY       10U

/*******************************************************************************
 * Type Definitions
 ******************************************************************************/

/**
 * @brief Power subsystem state
 */
typedef struct {
    double battery_voltage;         /**< Battery voltage (V) */
    double battery_current;         /**< Battery current (A) */
    double state_of_charge;         /**< State of charge (0.0-1.0) */
    double solar_power;             /**< Solar panel power (W) */
    double total_energy_wh;         /**< Total energy accumulated (Wh) */
    PowerMode_t power_mode;         /**< Current power mode */
    bool payload_enabled;           /**< Jetson payload enabled */
    uint32_t mode_entry_time_ms;    /**< Time entered current mode */
} PowerState_t;

/**
 * @brief Thermal subsystem state
 */
typedef struct {
    float obc_temp_c;               /**< OBC temperature (C) */
    float eps_temp_c;               /**< EPS temperature (C) */
    float battery_temp_c;           /**< Battery temperature (C) */
    float payload_temp_c;           /**< Jetson payload temperature (C) */
    float external_temp_c;          /**< External/structure temperature (C) */
    bool heater_enabled;            /**< Heater enabled flag */
    bool over_temp_flag;            /**< Over-temperature condition */
    bool under_temp_flag;           /**< Under-temperature condition */
} ThermalState_t;

/**
 * @brief ADCS subsystem state
 */
typedef struct {
    float mag_x_ut;                 /**< Magnetometer X (uT) */
    float mag_y_ut;                 /**< Magnetometer Y (uT) */
    float mag_z_ut;                 /**< Magnetometer Z (uT) */
    float gyro_x_dps;               /**< Gyroscope X (deg/s) */
    float gyro_y_dps;               /**< Gyroscope Y (deg/s) */
    float gyro_z_dps;               /**< Gyroscope Z (deg/s) */
    float sun_vector_x;             /**< Sun vector X (normalized) */
    float sun_vector_y;             /**< Sun vector Y (normalized) */
    float sun_vector_z;             /**< Sun vector Z (normalized) */
    bool detumbled;                 /**< Detumble achieved flag */
    bool sun_acquired;              /**< Sun acquisition flag */
    uint32_t last_update_ms;        /**< Last ADCS update timestamp */
} AdcsState_t;

/**
 * @brief Communications subsystem state
 */
typedef struct {
    uint32_t packets_sent;          /**< Packets transmitted */
    uint32_t packets_received;      /**< Packets received */
    uint32_t packets_failed;        /**< Failed transmissions */
    uint32_t last_ground_contact_ms;/**< Last ground contact time */
    uint32_t beacon_count;          /**< Beacons transmitted */
    uint16_t beacon_interval_s;     /**< Beacon interval (seconds) */
    bool comm_active;               /**< Communication link active */
    int8_t rssi_dbm;                /**< Last RSSI (dBm) */
} CommState_t;

/**
 * @brief Mission data state
 */
typedef struct {
    uint32_t boot_count;            /**< System boot count */
    uint32_t uptime_s;              /**< Current uptime (seconds) */
    uint32_t total_uptime_s;        /**< Total mission uptime (seconds) */
    uint64_t mission_time_ms;       /**< Mission elapsed time (ms) */
    uint32_t qso_count;             /**< QSO contacts made */
    uint32_t command_count;         /**< Commands received */
    uint32_t anomaly_count;         /**< Anomalies detected */
    bool safe_mode_entry;           /**< Safe mode was entered */
} MissionState_t;

/**
 * @brief Complete system state structure
 *
 * This structure contains all system state in a single location.
 * Access should only be through the provided accessor functions.
 */
typedef struct {
    /* Subsystem states */
    PowerState_t power;             /**< Power subsystem state */
    ThermalState_t thermal;         /**< Thermal subsystem state */
    AdcsState_t adcs;               /**< ADCS subsystem state */
    CommState_t comm;               /**< Communications state */
    MissionState_t mission;         /**< Mission data state */

    /* State machine */
    SmContext_t sm_context;         /**< Operational state machine */

    /* Flags */
    bool initialized;               /**< State initialized flag */
    bool watchdog_ok;               /**< Watchdog status */
    bool persistence_dirty;         /**< State needs to be saved */

    /* Timestamps */
    uint64_t last_update_ms;        /**< Last state update */
    uint64_t last_persist_ms;       /**< Last persistence save */

    /* Integrity */
    uint32_t crc32;                 /**< State CRC for integrity */
} SystemState_t;

/*******************************************************************************
 * Initialization
 ******************************************************************************/

/**
 * @brief Initialize system state to defaults
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_state_init(void);

/**
 * @brief Load system state from persistent storage
 *
 * @return SMART_QSO_OK on success, error if load failed
 */
SmartQsoResult_t sys_state_load(void);

/**
 * @brief Save system state to persistent storage
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_state_save(void);

/**
 * @brief Check if state has been modified and needs saving
 *
 * @return true if state is dirty
 */
bool sys_state_is_dirty(void);

/**
 * @brief Clear dirty flag after successful save
 */
void sys_state_clear_dirty(void);

/*******************************************************************************
 * Power State Accessors
 ******************************************************************************/

/**
 * @brief Get current power state
 *
 * @param[out] state Power state copy
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_get_power_state(PowerState_t *state);

/**
 * @brief Update battery voltage
 *
 * @param voltage_v Battery voltage in volts
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_battery_voltage(double voltage_v);

/**
 * @brief Update battery current
 *
 * @param current_a Battery current in amps
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_battery_current(double current_a);

/**
 * @brief Update state of charge
 *
 * @param soc State of charge (0.0 to 1.0)
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_state_of_charge(double soc);

/**
 * @brief Get current state of charge
 *
 * @return State of charge (0.0 to 1.0)
 */
double sys_get_state_of_charge(void);

/**
 * @brief Set power mode
 *
 * @param mode New power mode
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_power_mode(PowerMode_t mode);

/**
 * @brief Get current power mode
 *
 * @return Current power mode
 */
PowerMode_t sys_get_power_mode(void);

/**
 * @brief Enable/disable payload
 *
 * @param enabled true to enable payload
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_payload_enabled(bool enabled);

/**
 * @brief Check if payload is enabled
 *
 * @return true if payload is enabled
 */
bool sys_is_payload_enabled(void);

/*******************************************************************************
 * Thermal State Accessors
 ******************************************************************************/

/**
 * @brief Get current thermal state
 *
 * @param[out] state Thermal state copy
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_get_thermal_state(ThermalState_t *state);

/**
 * @brief Update temperature reading
 *
 * @param sensor Sensor index (0=OBC, 1=EPS, 2=Battery, 3=Payload, 4=External)
 * @param temp_c Temperature in Celsius
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_temperature(uint8_t sensor, float temp_c);

/**
 * @brief Check for thermal fault condition
 *
 * @return true if thermal fault exists
 */
bool sys_has_thermal_fault(void);

/*******************************************************************************
 * ADCS State Accessors
 ******************************************************************************/

/**
 * @brief Get current ADCS state
 *
 * @param[out] state ADCS state copy
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_get_adcs_state(AdcsState_t *state);

/**
 * @brief Update magnetometer reading
 *
 * @param x X component (uT)
 * @param y Y component (uT)
 * @param z Z component (uT)
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_magnetometer(float x, float y, float z);

/**
 * @brief Update gyroscope reading
 *
 * @param x X component (deg/s)
 * @param y Y component (deg/s)
 * @param z Z component (deg/s)
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_gyroscope(float x, float y, float z);

/**
 * @brief Set detumble achieved flag
 *
 * @param achieved true if detumble achieved
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_detumbled(bool achieved);

/**
 * @brief Check if spacecraft is detumbled
 *
 * @return true if detumbled
 */
bool sys_is_detumbled(void);

/*******************************************************************************
 * Communications State Accessors
 ******************************************************************************/

/**
 * @brief Get current communications state
 *
 * @param[out] state Communications state copy
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_get_comm_state(CommState_t *state);

/**
 * @brief Increment packet sent counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_packets_sent(void);

/**
 * @brief Increment packet received counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_packets_received(void);

/**
 * @brief Increment beacon counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_beacon_count(void);

/**
 * @brief Update last ground contact time
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_update_ground_contact(void);

/**
 * @brief Set beacon interval
 *
 * @param interval_s Interval in seconds
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_set_beacon_interval(uint16_t interval_s);

/*******************************************************************************
 * Mission State Accessors
 ******************************************************************************/

/**
 * @brief Get current mission state
 *
 * @param[out] state Mission state copy
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_get_mission_state(MissionState_t *state);

/**
 * @brief Increment boot counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_boot_count(void);

/**
 * @brief Get boot count
 *
 * @return Current boot count
 */
uint32_t sys_get_boot_count(void);

/**
 * @brief Update uptime
 *
 * Called periodically to update uptime counters.
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_update_uptime(void);

/**
 * @brief Get current uptime in seconds
 *
 * @return Current uptime
 */
uint32_t sys_get_uptime_s(void);

/**
 * @brief Increment QSO counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_qso_count(void);

/**
 * @brief Increment command counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_command_count(void);

/**
 * @brief Increment anomaly counter
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_increment_anomaly_count(void);

/*******************************************************************************
 * State Machine Accessors
 ******************************************************************************/

/**
 * @brief Get state machine context pointer
 *
 * @return Pointer to state machine context
 */
SmContext_t *sys_get_sm_context(void);

/**
 * @brief Get current operational state
 *
 * @return Current state
 */
SmState_t sys_get_operational_state(void);

/*******************************************************************************
 * Integrity Functions
 ******************************************************************************/

/**
 * @brief Calculate and update state CRC
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_state_update_crc(void);

/**
 * @brief Verify state CRC integrity
 *
 * @return true if CRC is valid
 */
bool sys_state_verify_crc(void);

/**
 * @brief Get complete state snapshot
 *
 * @param[out] state Complete state copy
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sys_get_full_state(SystemState_t *state);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SYSTEM_STATE_H */
