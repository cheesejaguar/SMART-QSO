/**
 * @file fault_mgmt.h
 * @brief Fault Detection, Isolation, and Recovery (FDIR) Interface
 *
 * This module provides fault management functionality including fault logging,
 * detection, isolation, and recovery procedures.
 *
 * @requirement SRS-FDIR-001 System shall detect and log all fault conditions
 * @requirement SRS-FDIR-002 System shall implement autonomous fault recovery
 * @requirement SRS-FDIR-003 System shall persist fault log across resets
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#ifndef SMART_QSO_FAULT_MGMT_H
#define SMART_QSO_FAULT_MGMT_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* Fault Management Constants                                                 */
/*===========================================================================*/

/** Fault log file path */
#define FAULT_LOG_FILE "/tmp/fault_log.dat"

/*===========================================================================*/
/* Fault Log Entry Structure                                                  */
/*===========================================================================*/

/**
 * @brief Fault Log Entry
 *
 * Records details of a fault occurrence for analysis and debugging.
 */
typedef struct {
    uint64_t timestamp_ms;                               /**< Fault occurrence time */
    uint8_t fault_type;                                  /**< Fault type identifier */
    uint8_t severity;                                    /**< Severity level */
    char description[SMART_QSO_FAULT_DESC_LEN];          /**< Fault description */
    double soc_at_fault;                                 /**< Battery SOC at fault time */
    bool recovered;                                      /**< Whether fault was recovered */
    uint32_t crc32;                                      /**< CRC for entry integrity */
} FaultLogEntry_t;

/*===========================================================================*/
/* Fault Management API                                                       */
/*===========================================================================*/

/**
 * @brief Initialize fault management module
 *
 * Loads fault log from persistent storage if available.
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre None
 * @post Fault log is initialized and ready for use
 */
SmartQsoResult_t fault_mgmt_init(void);

/**
 * @brief Add a fault log entry
 *
 * @param fault_type   Type of fault (FaultType_t)
 * @param severity     Severity level (FaultSeverity_t)
 * @param description  Textual description of fault
 * @param soc          Current state of charge at fault time
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre description != NULL
 * @post New fault entry is added to log
 */
SmartQsoResult_t fault_log_add(FaultType_t fault_type, FaultSeverity_t severity,
                               const char *description, double soc);

/**
 * @brief Get number of fault log entries
 *
 * @return Number of entries in fault log
 */
size_t fault_log_get_count(void);

/**
 * @brief Get a fault log entry
 *
 * @param index   Index of entry to retrieve (0-based)
 * @param[out] entry Pointer to entry structure to fill
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @pre entry != NULL
 * @pre index < fault_log_get_count()
 */
SmartQsoResult_t fault_log_get_entry(size_t index, FaultLogEntry_t *entry);

/**
 * @brief Get the most recent fault log entry
 *
 * @param[out] entry Pointer to entry structure to fill
 * @return SMART_QSO_OK on success, error code if log is empty
 *
 * @pre entry != NULL
 */
SmartQsoResult_t fault_log_get_last(FaultLogEntry_t *entry);

/**
 * @brief Mark a fault as recovered
 *
 * @param index Index of fault entry to mark as recovered
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t fault_log_mark_recovered(size_t index);

/**
 * @brief Save fault log to persistent storage
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t fault_log_save(void);

/**
 * @brief Load fault log from persistent storage
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t fault_log_load(void);

/**
 * @brief Clear all fault log entries
 *
 * @return SMART_QSO_OK on success, error code otherwise
 *
 * @warning This permanently deletes all fault records
 */
SmartQsoResult_t fault_log_clear(void);

/*===========================================================================*/
/* Fault Recovery API                                                         */
/*===========================================================================*/

/**
 * @brief Recover from thermal fault
 *
 * Disables payload and transitions to IDLE mode to reduce thermal load.
 *
 * @param soc Current state of charge
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t fault_recover_thermal(double soc);

/**
 * @brief Recover from power fault
 *
 * Transitions to appropriate power mode based on current SOC.
 *
 * @param soc Current state of charge
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t fault_recover_power(double soc);

/**
 * @brief Handle watchdog timeout
 *
 * Logs the watchdog timeout and sets recovery flag.
 *
 * @param soc Current state of charge
 * @return SMART_QSO_OK on success, error code otherwise
 */
SmartQsoResult_t fault_handle_watchdog(double soc);

/**
 * @brief Check if watchdog was triggered
 *
 * @return true if watchdog was triggered since last reset
 */
bool fault_was_watchdog_triggered(void);

/**
 * @brief Clear watchdog triggered flag
 */
void fault_clear_watchdog_flag(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_FAULT_MGMT_H */
