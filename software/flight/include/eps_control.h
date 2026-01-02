/**
 * @file eps_control.h
 * @brief Electrical Power Subsystem (EPS) Control Interface
 *
 * This module provides power management functionality including load switching,
 * power mode control, and power budget enforcement.
 *
 * @requirement SRS-EPS-001 System shall manage power distribution to subsystems
 * @requirement SRS-EPS-002 System shall implement power mode state machine
 * @requirement SRS-EPS-003 System shall enforce power budget limits
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#ifndef SMART_QSO_EPS_CONTROL_H
#define SMART_QSO_EPS_CONTROL_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* EPS Configuration Constants                                                */
/*===========================================================================*/

/** EPS configuration file path */
#define EPS_CONFIG_FILE "/tmp/eps_config.dat"

/** Power limit for SAFE mode (Watts) */
#define EPS_POWER_LIMIT_SAFE_W   0.5

/** Power limit for IDLE mode (Watts) */
#define EPS_POWER_LIMIT_IDLE_W   1.5

/** Power limit for ACTIVE mode (Watts) */
#define EPS_POWER_LIMIT_ACTIVE_W 3.0

/** SOC threshold for payload enable */
#define EPS_SOC_PAYLOAD_THRESHOLD 0.55

/** SOC threshold for entering SAFE mode */
#define EPS_SOC_SAFE_THRESHOLD    0.25

/** SOC threshold for entering IDLE mode */
#define EPS_SOC_IDLE_THRESHOLD    0.40

/*===========================================================================*/
/* EPS Control State Structure                                                */
/*===========================================================================*/

/**
 * @brief EPS Control State
 *
 * Holds the current state of the electrical power subsystem including
 * load switch states and power mode.
 */
typedef struct {
    bool payload_enabled;      /**< Jetson payload power state */
    bool radio_enabled;        /**< Radio/transponder power state */
    bool adcs_enabled;         /**< ADCS power state */
    bool beacon_enabled;       /**< Beacon power state */
    uint8_t power_mode;        /**< Current power mode (PowerMode_t) */
    double power_limit_w;      /**< Current power limit in watts */
    uint32_t last_control_ms;  /**< Last EPS control command time (low 32 bits) */
    uint32_t crc32;            /**< CRC32 for data integrity */
} EpsControlState_t;

/*===========================================================================*/
/* EPS Control API                                                            */
/*===========================================================================*/

/**
 * @brief Initialize EPS control module
 *
 * Loads configuration from persistent storage or initializes defaults.
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre None
 * @post EPS state is initialized and ready for use
 */
SmartQsoResult_t eps_init(void);

/**
 * @brief Get current EPS state
 *
 * @param[out] state Pointer to state structure to fill
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre state != NULL
 * @post state contains current EPS state
 */
SmartQsoResult_t eps_get_state(EpsControlState_t *state);

/**
 * @brief Control payload power (Jetson)
 *
 * @param enable true to enable, false to disable
 * @param soc    Current state of charge (0.0-1.0)
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre soc is in range [0.0, 1.0]
 * @post Payload power state is updated
 * @note Payload enable may be blocked if SOC is below threshold
 */
SmartQsoResult_t eps_control_payload(bool enable, double soc);

/**
 * @brief Control radio power
 *
 * @param enable true to enable, false to disable
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @post Radio power state is updated
 */
SmartQsoResult_t eps_control_radio(bool enable);

/**
 * @brief Control ADCS power
 *
 * @param enable true to enable, false to disable
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @post ADCS power state is updated
 */
SmartQsoResult_t eps_control_adcs(bool enable);

/**
 * @brief Control beacon power
 *
 * @param enable true to enable, false to disable
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @post Beacon power state is updated
 */
SmartQsoResult_t eps_control_beacon(bool enable);

/**
 * @brief Set power mode
 *
 * Transitions the system to the specified power mode, automatically
 * configuring load switches and power limits.
 *
 * @param mode  Target power mode
 * @param soc   Current state of charge (0.0-1.0)
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre mode is a valid PowerMode_t value
 * @post System is in specified power mode
 */
SmartQsoResult_t eps_set_power_mode(PowerMode_t mode, double soc);

/**
 * @brief Get current power mode
 *
 * @return Current power mode
 */
PowerMode_t eps_get_power_mode(void);

/**
 * @brief Check if payload is enabled
 *
 * @return true if payload is enabled, false otherwise
 */
bool eps_is_payload_enabled(void);

/**
 * @brief Get current power limit
 *
 * @return Current power limit in watts
 */
double eps_get_power_limit(void);

/**
 * @brief Save EPS configuration to persistent storage
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t eps_save_config(void);

/**
 * @brief Load EPS configuration from persistent storage
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t eps_load_config(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_EPS_CONTROL_H */
