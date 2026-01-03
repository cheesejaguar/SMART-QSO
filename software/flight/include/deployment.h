/**
 * @file deployment.h
 * @brief SMART-QSO Deployment Control Module
 *
 * Manages post-separation deployment sequences including antenna deployment,
 * solar panel deployment, and deployment inhibit handling per CubeSat Design
 * Specification requirements.
 *
 * @requirement SRS-DEP-001 System shall wait 30 minutes after separation before deployment
 * @requirement SRS-DEP-002 System shall monitor deployment switch state
 * @requirement SRS-DEP-003 System shall provide deployment retry capability
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 * @note Following GeneSat/PharmaSat/O/OREOS/EcAMSat deployment patterns
 */

#ifndef SMART_QSO_DEPLOYMENT_H
#define SMART_QSO_DEPLOYMENT_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>

/*===========================================================================*/
/* Constants                                                                  */
/*===========================================================================*/

/** Deployment wait time after separation (ms) - CDS requirement */
#define DEPLOYMENT_WAIT_TIME_MS         (30U * 60U * 1000U)  /* 30 minutes */

/** Antenna deployment burn time (ms) */
#define ANTENNA_DEPLOY_BURN_TIME_MS     3000U

/** Solar panel deployment burn time (ms) */
#define SOLAR_DEPLOY_BURN_TIME_MS       5000U

/** Maximum deployment retry attempts */
#define DEPLOYMENT_MAX_RETRIES          3U

/** Retry delay between attempts (ms) */
#define DEPLOYMENT_RETRY_DELAY_MS       60000U  /* 1 minute */

/** Deployment verification timeout (ms) */
#define DEPLOYMENT_VERIFY_TIMEOUT_MS    10000U

/*===========================================================================*/
/* Types                                                                      */
/*===========================================================================*/

/**
 * @brief Deployment state machine states
 */
typedef enum {
    DEPLOY_STATE_INHIBITED = 0,    /**< Waiting for switch release */
    DEPLOY_STATE_WAITING,          /**< Separation detected, waiting timer */
    DEPLOY_STATE_ANTENNA_PENDING,  /**< Ready to deploy antenna */
    DEPLOY_STATE_ANTENNA_ACTIVE,   /**< Antenna deployment in progress */
    DEPLOY_STATE_ANTENNA_VERIFY,   /**< Verifying antenna deployment */
    DEPLOY_STATE_SOLAR_PENDING,    /**< Ready to deploy solar panels */
    DEPLOY_STATE_SOLAR_ACTIVE,     /**< Solar panel deployment in progress */
    DEPLOY_STATE_SOLAR_VERIFY,     /**< Verifying solar panel deployment */
    DEPLOY_STATE_COMPLETE,         /**< All deployments successful */
    DEPLOY_STATE_FAILED,           /**< Deployment failed after retries */
    DEPLOY_STATE_COUNT
} DeploymentState_t;

/**
 * @brief Deployment element identifiers
 */
typedef enum {
    DEPLOY_ELEMENT_ANTENNA = 0,
    DEPLOY_ELEMENT_SOLAR_PANEL,
    DEPLOY_ELEMENT_COUNT
} DeploymentElement_t;

/**
 * @brief Deployment element status
 */
typedef struct {
    bool deployed;              /**< Deployment confirmed */
    bool deploy_attempted;      /**< At least one attempt made */
    uint8_t attempt_count;      /**< Number of attempts */
    uint32_t deploy_time_ms;    /**< Time when deployed (MET) */
    bool switch_state;          /**< Current deployment switch state */
} DeploymentElementStatus_t;

/**
 * @brief Complete deployment module state
 */
typedef struct {
    DeploymentState_t state;                    /**< Current state machine state */
    uint32_t separation_time_ms;                /**< Time of separation detection */
    uint32_t state_entry_time_ms;               /**< Time of last state entry */
    uint32_t deploy_start_time_ms;              /**< Current deployment action start */
    DeploymentElementStatus_t elements[DEPLOY_ELEMENT_COUNT];
    bool separation_detected;                   /**< Deployment switch released */
    bool deployment_enabled;                    /**< Ground command enable */
    uint32_t crc;                               /**< CRC for data integrity */
} DeploymentStatus_t;

/**
 * @brief Deployment telemetry packet
 */
typedef struct {
    uint8_t state;                  /**< Current deployment state */
    uint8_t antenna_deployed;       /**< Antenna deployment status */
    uint8_t antenna_attempts;       /**< Antenna deployment attempts */
    uint8_t solar_deployed;         /**< Solar panel deployment status */
    uint8_t solar_attempts;         /**< Solar panel deployment attempts */
    uint32_t time_since_sep_s;      /**< Seconds since separation */
    uint8_t inhibit_active;         /**< Deployment inhibited */
} DeploymentTelemetry_t;

/*===========================================================================*/
/* Public Functions                                                           */
/*===========================================================================*/

/**
 * @brief Initialize the deployment control module
 *
 * @return true if initialization successful, false otherwise
 *
 * @pre None
 * @post Module state initialized to INHIBITED
 */
bool deployment_init(void);

/**
 * @brief Process deployment state machine
 *
 * Must be called periodically (recommended: 100ms) to process deployment
 * state machine and manage deployment sequences.
 *
 * @param current_time_ms Current mission elapsed time in milliseconds
 *
 * @pre deployment_init() called
 * @post State machine advanced if conditions met
 */
void deployment_process(uint32_t current_time_ms);

/**
 * @brief Check if deployment switch indicates separation
 *
 * @return true if deployment switch released (separation detected)
 */
bool deployment_is_separated(void);

/**
 * @brief Get current deployment state
 *
 * @return Current deployment state machine state
 */
DeploymentState_t deployment_get_state(void);

/**
 * @brief Get deployment status for specified element
 *
 * @param element Deployment element to query
 * @param status Pointer to status structure to fill
 *
 * @return true if status retrieved, false if invalid element
 */
bool deployment_get_element_status(DeploymentElement_t element,
                                   DeploymentElementStatus_t *status);

/**
 * @brief Get deployment telemetry packet
 *
 * @param telemetry Pointer to telemetry structure to fill
 *
 * @return true if telemetry retrieved
 */
bool deployment_get_telemetry(DeploymentTelemetry_t *telemetry);

/**
 * @brief Enable or disable deployment (ground command)
 *
 * @param enable true to enable deployment, false to inhibit
 *
 * @return true if command accepted
 */
bool deployment_set_enabled(bool enable);

/**
 * @brief Force retry deployment of specified element (ground command)
 *
 * @param element Element to retry
 *
 * @return true if retry initiated
 */
bool deployment_force_retry(DeploymentElement_t element);

/**
 * @brief Get human-readable state name
 *
 * @param state Deployment state
 *
 * @return State name string
 */
const char *deployment_state_name(DeploymentState_t state);

/**
 * @brief Check if all deployments complete
 *
 * @return true if all elements successfully deployed
 */
bool deployment_all_complete(void);

/**
 * @brief Save deployment state to persistent storage
 *
 * @return true if save successful
 */
bool deployment_save_state(void);

/**
 * @brief Load deployment state from persistent storage
 *
 * @return true if load successful and data valid
 */
bool deployment_load_state(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_DEPLOYMENT_H */
