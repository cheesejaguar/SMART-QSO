/**
 * @file state_machine.h
 * @brief State machine interface for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module implements a formal state machine for spacecraft operational modes.
 * Per NPR 7150.2 and NASA Ames SmallSat heritage (GeneSat, PharmaSat).
 *
 * @requirement SRS-030 System shall implement formal state machine for mode control
 * @requirement SRS-031 System shall log all state transitions
 */

#ifndef SMART_QSO_STATE_MACHINE_H
#define SMART_QSO_STATE_MACHINE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * State Definitions
 ******************************************************************************/

/**
 * @brief Spacecraft operational states
 *
 * States follow NASA Ames SmallSat heritage conventions:
 * - BOOT: Initial power-on state
 * - SAFE: Minimum power, beacon-only survival mode
 * - DETUMBLE: Post-deployment attitude stabilization
 * - IDLE: Normal operations without AI payload
 * - ACTIVE: Full operations with AI payload enabled
 * - EMERGENCY: Critical fault response mode
 */
typedef enum {
    STATE_BOOT      = 0,  /**< Initial boot state */
    STATE_SAFE      = 1,  /**< Safe mode: minimum power, beacon only */
    STATE_DETUMBLE  = 2,  /**< Detumble mode: attitude stabilization */
    STATE_IDLE      = 3,  /**< Idle mode: normal ops without AI */
    STATE_ACTIVE    = 4,  /**< Active mode: full ops with AI payload */
    STATE_EMERGENCY = 5,  /**< Emergency mode: critical fault response */
    STATE_COUNT     = 6   /**< Number of states (for bounds checking) */
} SmState_t;

/**
 * @brief State machine events (triggers for transitions)
 */
typedef enum {
    EVENT_NONE              = 0,   /**< No event */
    EVENT_BOOT_COMPLETE     = 1,   /**< Boot sequence completed */
    EVENT_DETUMBLE_COMPLETE = 2,   /**< Detumble achieved */
    EVENT_COMM_ACQUIRED     = 3,   /**< Communication link established */
    EVENT_POWER_OK          = 4,   /**< Sufficient power available */
    EVENT_POWER_LOW         = 5,   /**< Low power condition */
    EVENT_POWER_CRITICAL    = 6,   /**< Critical power condition */
    EVENT_THERMAL_FAULT     = 7,   /**< Thermal limit exceeded */
    EVENT_COMM_TIMEOUT      = 8,   /**< Communication timeout */
    EVENT_WATCHDOG_TIMEOUT  = 9,   /**< Watchdog timeout */
    EVENT_GROUND_CMD_SAFE   = 10,  /**< Ground command: enter safe mode */
    EVENT_GROUND_CMD_IDLE   = 11,  /**< Ground command: enter idle mode */
    EVENT_GROUND_CMD_ACTIVE = 12,  /**< Ground command: enter active mode */
    EVENT_AI_READY          = 13,  /**< AI payload ready */
    EVENT_AI_FAULT          = 14,  /**< AI payload fault */
    EVENT_RECOVERY_OK       = 15,  /**< Recovery successful */
    EVENT_COUNT             = 16   /**< Number of events */
} SmEvent_t;

/**
 * @brief Transition result codes
 */
typedef enum {
    SM_RESULT_OK            = 0,  /**< Transition successful */
    SM_RESULT_NO_TRANSITION = 1,  /**< No transition defined for event */
    SM_RESULT_GUARD_FAILED  = 2,  /**< Transition guard condition failed */
    SM_RESULT_INVALID_STATE = 3,  /**< Invalid current state */
    SM_RESULT_INVALID_EVENT = 4,  /**< Invalid event */
    SM_RESULT_ACTION_FAILED = 5   /**< Transition action failed */
} SmResult_t;

/*******************************************************************************
 * Type Definitions
 ******************************************************************************/

/**
 * @brief Guard function type
 * @return true if transition is allowed, false otherwise
 */
typedef bool (*SmGuardFunc_t)(void);

/**
 * @brief Action function type (entry, exit, or transition action)
 * @return SMART_QSO_OK on success, error code otherwise
 */
typedef SmartQsoResult_t (*SmActionFunc_t)(void);

/**
 * @brief State transition definition
 */
typedef struct {
    SmState_t from_state;       /**< Source state */
    SmEvent_t event;            /**< Trigger event */
    SmState_t to_state;         /**< Target state */
    SmGuardFunc_t guard;        /**< Guard function (may be NULL) */
    SmActionFunc_t action;      /**< Transition action (may be NULL) */
} SmTransition_t;

/**
 * @brief State definition with entry/exit actions
 */
typedef struct {
    SmState_t state;            /**< State identifier */
    const char *name;           /**< State name string */
    SmActionFunc_t on_entry;    /**< Entry action (may be NULL) */
    SmActionFunc_t on_exit;     /**< Exit action (may be NULL) */
} SmStateConfig_t;

/**
 * @brief State machine context
 */
typedef struct {
    SmState_t current_state;    /**< Current state */
    SmState_t previous_state;   /**< Previous state (for diagnostics) */
    uint32_t transition_count;  /**< Number of transitions executed */
    uint64_t last_transition_ms;/**< Timestamp of last transition */
    uint64_t state_entry_ms;    /**< Timestamp of current state entry */
    bool is_initialized;        /**< Initialization flag */
} SmContext_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize the state machine
 *
 * @param[out] ctx State machine context to initialize
 * @param[in] initial_state Initial state
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sm_init(SmContext_t *ctx, SmState_t initial_state);

/**
 * @brief Process an event and execute any resulting transition
 *
 * @param[in,out] ctx State machine context
 * @param[in] event Event to process
 * @param[out] result Transition result (may be NULL)
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sm_process_event(SmContext_t *ctx,
                                   SmEvent_t event,
                                   SmResult_t *result);

/**
 * @brief Get current state
 *
 * @param[in] ctx State machine context
 * @param[out] state Current state
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sm_get_state(const SmContext_t *ctx, SmState_t *state);

/**
 * @brief Get state name string
 *
 * @param[in] state State to get name for
 * @return State name string, or "UNKNOWN" if invalid
 */
const char *sm_get_state_name(SmState_t state);

/**
 * @brief Get event name string
 *
 * @param[in] event Event to get name for
 * @return Event name string, or "UNKNOWN" if invalid
 */
const char *sm_get_event_name(SmEvent_t event);

/**
 * @brief Get time in current state
 *
 * @param[in] ctx State machine context
 * @param[out] duration_ms Time in current state in milliseconds
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sm_get_state_duration(const SmContext_t *ctx,
                                        uint64_t *duration_ms);

/**
 * @brief Check if a specific transition is valid
 *
 * @param[in] ctx State machine context
 * @param[in] event Event to check
 * @param[out] is_valid True if transition is valid
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sm_is_transition_valid(const SmContext_t *ctx,
                                         SmEvent_t event,
                                         bool *is_valid);

/**
 * @brief Force state change (for recovery scenarios only)
 *
 * @param[in,out] ctx State machine context
 * @param[in] new_state State to force
 * @return SMART_QSO_OK on success
 *
 * @warning Use only for fault recovery. Bypasses guards and normal transitions.
 */
SmartQsoResult_t sm_force_state(SmContext_t *ctx, SmState_t new_state);

/**
 * @brief Get transition statistics
 *
 * @param[in] ctx State machine context
 * @param[out] transition_count Number of transitions executed
 * @param[out] previous_state Previous state
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sm_get_statistics(const SmContext_t *ctx,
                                    uint32_t *transition_count,
                                    SmState_t *previous_state);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_STATE_MACHINE_H */
