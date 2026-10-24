/**
 * @file deployment.h
 * @brief SMART-QSO Deployment Control Module
 *
 * Implemented in Rust (software/rust/sqso-fsw/src/deployment.rs,
 * software/rust/sqso-ffi/src/deployment.rs).
 *
 * Manages post-separation deployment sequences including antenna deployment,
 * solar panel deployment, and deployment inhibit handling per CubeSat Design
 * Specification requirements.
 *
 * Timing uses elapsed ticks (deployment_tick()), not absolute timestamps, and
 * the sequence is persisted (FLASH_REGION_DEPLOYMENT) so resets neither
 * shorten the 30-minute wait nor repeat burns:
 *  - 30-minute wait (CDS 2.4.4): a power-on, brownout, pin or unknown reset
 *    restarts it (a separation-switch toggle removes power, CDS 2.3.2.4); a
 *    watchdog or software reset resumes it; a stowed switch restarts it.
 *  - Burns: each element has its own burn and sense pin; a burn needs the
 *    master arm (GPIO_PIN_DEPLOY_EN) and the element pin. The attempt is
 *    persisted before the pin is driven, and nothing fires if that fails.
 *    An element whose sense reads deployed is never fired. A reset during a
 *    burn spends the attempt. Attempts are bounded across resets.
 *  - COMPLETE and FAILED ignore the switch (re-burning is not harmless); only
 *    GSE ARM (deployment_arm()) resets them. ARMED does not progress while
 *    powered; the next boot starts the sequence.
 *  - Blank or corrupt record: start over with zero attempts (fault logged);
 *    sense pins skip anything already deployed.
 *
 * Software is not the bench inhibit: with no dispenser the switches read
 * released, so a powered unit fires burn wires 30 minutes after any boot
 * unless the hardware inhibits (RBF, SYS-225) hold deployment power off.
 *
 * @requirement SYS-221 Deployables wait 30 minutes after separation; switch toggle resets the timer
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
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

/** Deployment wait time after separation (s) */
#define DEPLOYMENT_WAIT_TIME_S          (DEPLOYMENT_WAIT_TIME_MS / 1000U)

/** Antenna deployment burn time (ms) */
#define ANTENNA_DEPLOY_BURN_TIME_MS     3000U

/** Solar panel deployment burn time (ms) */
#define SOLAR_DEPLOY_BURN_TIME_MS       5000U

/** Maximum deployment attempts per element (across resets) */
#define DEPLOYMENT_MAX_RETRIES          3U

/** Delay before retrying an unconfirmed deployment (ms) */
#define DEPLOYMENT_RETRY_DELAY_MS       60000U  /* 1 minute */

/** Wait-count persistence interval (s) */
#define DEPLOYMENT_PERSIST_INTERVAL_S   60U

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
    DEPLOY_STATE_ARMED,            /**< GSE-armed for flight; starts at next boot */
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
    uint8_t attempt_count;      /**< Number of attempts (persisted) */
    uint32_t deploy_time_s;     /**< Seconds after separation when confirmed */
    bool switch_state;          /**< Current element sense reading */
} DeploymentElementStatus_t;

/**
 * @brief Complete deployment module state
 */
typedef struct {
    DeploymentState_t state;                    /**< Current state machine state */
    uint32_t wait_elapsed_s;                    /**< Seconds counted toward the 30-minute wait */
    uint32_t state_elapsed_ms;                  /**< Time in the current state */
    uint32_t since_separation_s;                /**< Seconds since separation (this boot) */
    DeploymentElementStatus_t elements[DEPLOY_ELEMENT_COUNT];
    bool separation_detected;                   /**< Deployment switch released */
    bool deployment_enabled;                    /**< Ground command enable */
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
 * @brief Initialize at boot: drive every burn line low, load the persisted
 *        sequence and apply the reset-cause rules. Safe to call again (a
 *        simulated reset).
 *
 * @return true if initialization successful
 */
bool deployment_init(void);

/**
 * @brief Advance the sequence by elapsed time
 *
 * @param[in] elapsed_ms Time since the previous tick
 */
void deployment_tick(uint32_t elapsed_ms);

/**
 * @brief Re-arm for flight (GSE only): attempts and flags cleared; nothing
 *        progresses until the next boot
 */
SmartQsoResult_t deployment_arm(void);

/**
 * @brief Check if separation has been detected
 */
bool deployment_is_separated(void);

/**
 * @brief Get current deployment state
 */
DeploymentState_t deployment_get_state(void);

/**
 * @brief Seconds counted toward the 30-minute wait
 */
uint32_t deployment_wait_elapsed_s(void);

/**
 * @brief Get deployment element status
 *
 * @param[in] element Element to query
 * @param[out] status Status structure to fill
 * @return true if valid element
 */
bool deployment_get_element_status(DeploymentElement_t element,
                                   DeploymentElementStatus_t *status);

/**
 * @brief Get deployment telemetry
 *
 * @param[out] telemetry Telemetry structure to fill
 * @return true if successful
 */
bool deployment_get_telemetry(DeploymentTelemetry_t *telemetry);

/**
 * @brief Enable or inhibit deployment (ground command); persisted
 */
bool deployment_set_enabled(bool enable);

/**
 * @brief Retry an element (ground command)
 *
 * Refused until the 30-minute wait has been served (CDS 2.4.4) and while a
 * burn is in progress.
 *
 * @return true if the retry was accepted
 */
bool deployment_force_retry(DeploymentElement_t element);

/**
 * @brief Get state name string
 */
const char *deployment_state_name(DeploymentState_t state);

/**
 * @brief Check if all deployments complete
 */
bool deployment_all_complete(void);

/**
 * @brief Persist the sequence state
 *
 * @return true if saved
 */
bool deployment_save_state(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_DEPLOYMENT_H */
