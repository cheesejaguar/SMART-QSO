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
 *  - Order (STR-16): the solar wings first, then the antenna (the AntS tape
 *    elements pass below the stowed wings). An element whose attempts run
 *    out is fault-logged and the sequence goes on to the next; it ends
 *    COMPLETE only if both deployed, otherwise FAILED.
 *  - Wings: two burn resistors per wing on separate drivers, used on
 *    alternate attempts, one resistor at a time: wing 1 then wing 2 on
 *    GPIO_PIN_BURN_W1A and GPIO_PIN_BURN_W2A (attempts 1 and 3), on
 *    GPIO_PIN_BURN_W1B and GPIO_PIN_BURN_W2B (attempt 2), 10 s each (decision C-17);
 *    sensed by GPIO_PIN_SOLAR_SENSE.
 *  - Antenna: the ISIS AntS, armed and deployed over I2C bus 3 by its own
 *    controller (automatic sequential deployment of its four elements) and
 *    sensed by every element's switch in its status word; three hardware
 *    inhibits sit on its supply. GPIO_PIN_ANTENNA_BURN is wing 2's
 *    resistor A (GPIO_PIN_BURN_W2A).
 *  - A burn needs the master arm (GPIO_PIN_DEPLOY_EN) and the element's line.
 *    The attempt is persisted before the line is driven, and nothing fires
 *    if that fails. Attempts are DEPLOYMENT_RETRY_DELAY_MS apart.
 *    An element whose sense reads deployed is never fired. A reset during a
 *    burn spends the attempt. Attempts are bounded across resets.
 *  - COMPLETE and FAILED ignore the switch (re-burning is not harmless); only
 *    GSE ARM (deployment_arm()) resets them. ARMED does not progress while
 *    powered; the next boot starts the sequence.
 *  - Blank or corrupt record: start over with zero attempts (fault logged);
 *    sense pins skip anything already deployed.
 *  - The transmitters stay silent while the AntS deployment is pending
 *    (decision C-19): deployment_downlink_held(), applied after every call
 *    that can change the sequence as RF_INHIBIT_ANTS_PENDING (rf_control.h;
 *    the receiver stays on). It ends, latched in the record, when the AntS
 *    reads deployed or its sequence ends (attempts spent, or the attempt at
 *    the deferral cap made), and is never in force while deployment is
 *    inhibited by command. It does not shorten the 45-minute RF silence.
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

/** Antenna deployment window (ms): the AntS deploys its four elements in turn, at most 10 s each */
#define ANTENNA_DEPLOY_BURN_TIME_MS     45000U

/** Solar wing burn time (ms): 10 s at 0.8 A per resistor, 32 J (decision C-17), inside the EPS's 12 s one-shot */
#define SOLAR_DEPLOY_BURN_TIME_MS       10000U

/** Maximum deployment attempts per element (across resets) */
#define DEPLOYMENT_MAX_RETRIES          3U

/** Delay before retrying an unconfirmed deployment (ms) */
#define DEPLOYMENT_RETRY_DELAY_MS       3000000U  /* 50 minutes: attempts spread over the orbit, one in sunlight (LV-07) */

/** Wait-count persistence interval (s) */
#define DEPLOYMENT_PERSIST_INTERVAL_S   60U

/** The AntS deploys only at or above this temperature (°C): its −20 °C
 *  qualified minimum plus 3 °C of sensor tolerance (decision C-18) */
#define DEPLOYMENT_ANTS_MIN_TEMP_C      (-17.0)

/** Deferrals of the AntS for its temperature after which it deploys
 *  anyway (30 slots of DEPLOYMENT_RETRY_DELAY_MS, about 25 h) */
#define DEPLOYMENT_ANTS_MAX_DEFERRALS   30U

/** The AntS controller is powered this long before it is read, status or
 *  temperature (ms; it answers only while its supply SW8 is on: RID P3F-A-01) */
#define DEPLOYMENT_ANTS_START_MS        1000U

/** At the deferral cap, or on the schedule fallback, the AntS deploys only
 *  after this much continuous sunlight (ms; RID P3F-G-02) */
#define DEPLOYMENT_ANTS_SUN_MS          (20U * 60U * 1000U)

/** ... waiting at most this long for it (ms: two slots, more than an orbit) */
#define DEPLOYMENT_ANTS_SUN_WAIT_MS     (2U * DEPLOYMENT_RETRY_DELAY_MS)

/** Unknown AntS temperatures in a row after which the analysis schedule
 *  replaces the gate */
#define DEPLOYMENT_ANTS_UNKNOWN_LIMIT   3U

/** The schedule fallback deploys no earlier than this after separation (s) */
#define DEPLOYMENT_ANTS_SCHEDULE_S      (69U * 60U)

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
 * @brief DEPLOY_RETRY with its force flag: as deployment_force_retry(),
 *        and with force set (the antenna only) the AntS temperature gate is
 *        passed by, so the antenna deploys at once
 *
 * The gate (decision C-18): before each antenna attempt the AntS is
 * powered for DEPLOYMENT_ANTS_START_MS, its status read (deployed: never
 * fired) and its own temperature read; below DEPLOYMENT_ANTS_MIN_TEMP_C or
 * unknown the attempt waits the next DEPLOYMENT_RETRY_DELAY_MS slot without
 * being spent (the slot, the deferrals and the force flag are persisted, so
 * a reset spends none), at most DEPLOYMENT_ANTS_MAX_DEFERRALS times, then
 * deploys after DEPLOYMENT_ANTS_SUN_MS of continuous sun (or
 * DEPLOYMENT_ANTS_SUN_WAIT_MS without it). After
 * DEPLOYMENT_ANTS_UNKNOWN_LIMIT unknown readings in a row it deploys on the
 * analysis schedule: the first DEPLOYMENT_ANTS_SUN_MS of sun from
 * separation + DEPLOYMENT_ANTS_SCHEDULE_S (RID P3F-G-02).
 *
 * @return true if accepted (false for force with the wings)
 */
bool deployment_retry(DeploymentElement_t element, bool force);

/**
 * @brief The AntS temperature gate's deferrals and last reading
 * @param[out] deferrals deferrals so far (persisted); may be NULL
 * @param[out] temp_c the last AntS temperature (°C), NaN if none valid; may be NULL
 * @return true unless the state is in use (a nested call)
 */
bool deployment_ants_gate(uint8_t *deferrals, double *temp_c);

/**
 * @brief Get state name string
 */
const char *deployment_state_name(DeploymentState_t state);

/**
 * @brief Check if all deployments complete
 */
bool deployment_all_complete(void);

/**
 * @brief Whether the wings did not deploy: their attempts spent, not
 *        confirmed, and the wing sense line reading stowed
 *
 * The mode manager then holds IDLE (EPS-10). DEPLOY_RETRY of the wings
 * gives fresh attempts and clears it.
 *
 * @requirement SRS-F128 The software shall hold the mode at IDLE while the wings read not deployed after their attempts
 */
bool deployment_wings_failed(void);

/**
 * @brief Whether the transmitters are held off for the AntS (decision C-19)
 *
 * True after deployment_init() while deployment is enabled and the AntS
 * sequence has not ended: the antenna not read deployed, its attempts not
 * spent, and no attempt made at the deferral cap
 * (DEPLOYMENT_ANTS_MAX_DEFERRALS). The end is persisted and latched: a
 * later DEPLOY_RETRY does not bring the hold back; GSE ARM or a corrupt
 * record does (the sequence starts over). A nested call reads true.
 *
 * @requirement SRS-F167 Transmitters silent while the AntS deployment is pending (deferred or not yet attempted after the wings); the downlink starts when the AntS reads deployed or its sequence ends (attempts spent, or the attempt at the deferral cap made), latched, and never before the 45-minute RF silence
 */
bool deployment_downlink_held(void);

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
