/**
 * @file mission_data.h
 * @brief Mission Data Persistence Interface
 *
 * This module provides mission data storage and persistence functionality
 * including telemetry storage, state persistence, and data integrity.
 *
 * @requirement SRS-DATA-001 System shall persist mission data across resets
 * @requirement SRS-DATA-002 System shall protect data integrity with CRC
 * @requirement SRS-DATA-003 System shall track cumulative mission statistics
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#ifndef SMART_QSO_MISSION_DATA_H
#define SMART_QSO_MISSION_DATA_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* Mission Data Constants                                                     */
/*===========================================================================*/

/** Mission data file path */
#define MISSION_DATA_FILE "/tmp/mission_data.dat"

/*===========================================================================*/
/* Mission Data Structure                                                     */
/*===========================================================================*/

/**
 * @brief Mission Data
 *
 * Persistent mission statistics and state tracked across resets.
 */
typedef struct {
    uint64_t mission_start_ms;                   /**< Mission start timestamp */
    uint64_t total_uptime_ms;                    /**< Total uptime across resets */
    uint32_t reset_count;                        /**< Number of resets */
    uint64_t last_reset_ms;                      /**< Last reset timestamp */
    double total_energy_wh;                      /**< Total energy consumed */
    uint32_t fault_count;                        /**< Total fault occurrences */
    uint8_t mission_phase;                       /**< Current mission phase */
    char last_fault[SMART_QSO_FAULT_DESC_LEN];   /**< Description of last fault */
    uint32_t crc32;                              /**< CRC32 for data integrity */
} MissionData_t;

/*===========================================================================*/
/* Mission Phase Definitions                                                  */
/*===========================================================================*/

/**
 * @brief Mission phase enumeration
 */
typedef enum {
    MISSION_PHASE_COMMISSIONING = 0,  /**< Initial commissioning phase */
    MISSION_PHASE_IDLE          = 1,  /**< Idle operations */
    MISSION_PHASE_ACTIVE        = 2,  /**< Active operations */
    MISSION_PHASE_EOL           = 3   /**< End of life */
} MissionPhase_t;

/*===========================================================================*/
/* Mission Data API                                                           */
/*===========================================================================*/

/**
 * @brief Initialize mission data module
 *
 * Loads mission data from persistent storage or initializes defaults.
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre None
 * @post Mission data is initialized and ready for use
 */
SmartQsoResult_t mission_data_init(void);

/**
 * @brief Get current mission data
 *
 * @param[out] data Pointer to mission data structure to fill
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre data != NULL
 * @post data contains current mission data
 */
SmartQsoResult_t mission_data_get(MissionData_t *data);

/**
 * @brief Update mission start time
 *
 * @param start_ms Mission start timestamp in milliseconds
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t mission_data_set_start(uint64_t start_ms);

/**
 * @brief Update total uptime
 *
 * @param uptime_ms Total uptime in milliseconds
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t mission_data_update_uptime(uint64_t uptime_ms);

/**
 * @brief Increment reset count
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t mission_data_increment_reset(void);

/**
 * @brief Add energy consumption
 *
 * @param energy_wh Energy consumed in watt-hours
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t mission_data_add_energy(double energy_wh);

/**
 * @brief Set current mission phase
 *
 * @param phase New mission phase
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre phase is a valid MissionPhase_t value
 */
SmartQsoResult_t mission_data_set_phase(MissionPhase_t phase);

/**
 * @brief Get current mission phase
 *
 * @return Current mission phase
 */
MissionPhase_t mission_data_get_phase(void);

/**
 * @brief Record a fault occurrence
 *
 * @param description Fault description
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre description != NULL
 */
SmartQsoResult_t mission_data_record_fault(const char *description);

/**
 * @brief Save mission data to persistent storage
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t mission_data_save(void);

/**
 * @brief Load mission data from persistent storage
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t mission_data_load(void);

/**
 * @brief Get reset count
 *
 * @return Number of resets
 */
uint32_t mission_data_get_reset_count(void);

/**
 * @brief Get total uptime
 *
 * @return Total uptime in milliseconds
 */
uint64_t mission_data_get_uptime(void);

/**
 * @brief Get total energy consumed
 *
 * @return Total energy in watt-hours
 */
double mission_data_get_energy(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_MISSION_DATA_H */
