/**
 * @file state_machine.c
 * @brief State machine implementation for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Implements formal state machine for spacecraft operational modes.
 * Per NPR 7150.2 and NASA Ames SmallSat heritage.
 */

#include "state_machine.h"
#include "fault_mgmt.h"
#include <stddef.h>

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Maximum number of transitions in the table */
#define SM_MAX_TRANSITIONS  64U

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/**
 * @brief State configuration table
 */
static const SmStateConfig_t s_state_configs[STATE_COUNT] = {
    { STATE_BOOT,      "BOOT",      NULL, NULL },
    { STATE_SAFE,      "SAFE",      NULL, NULL },
    { STATE_DETUMBLE,  "DETUMBLE",  NULL, NULL },
    { STATE_IDLE,      "IDLE",      NULL, NULL },
    { STATE_ACTIVE,    "ACTIVE",    NULL, NULL },
    { STATE_EMERGENCY, "EMERGENCY", NULL, NULL }
};

/**
 * @brief State transition table
 *
 * Defines all valid state transitions. Transitions not in this table
 * are considered invalid and will be rejected.
 */
static const SmTransition_t s_transitions[] = {
    /* From BOOT state */
    { STATE_BOOT, EVENT_BOOT_COMPLETE, STATE_DETUMBLE, NULL, NULL },
    { STATE_BOOT, EVENT_POWER_CRITICAL, STATE_SAFE, NULL, NULL },

    /* From DETUMBLE state */
    { STATE_DETUMBLE, EVENT_DETUMBLE_COMPLETE, STATE_IDLE, NULL, NULL },
    { STATE_DETUMBLE, EVENT_POWER_LOW, STATE_SAFE, NULL, NULL },
    { STATE_DETUMBLE, EVENT_POWER_CRITICAL, STATE_SAFE, NULL, NULL },
    { STATE_DETUMBLE, EVENT_WATCHDOG_TIMEOUT, STATE_SAFE, NULL, NULL },
    { STATE_DETUMBLE, EVENT_GROUND_CMD_SAFE, STATE_SAFE, NULL, NULL },

    /* From SAFE state */
    { STATE_SAFE, EVENT_POWER_OK, STATE_IDLE, NULL, NULL },
    { STATE_SAFE, EVENT_RECOVERY_OK, STATE_IDLE, NULL, NULL },
    { STATE_SAFE, EVENT_GROUND_CMD_IDLE, STATE_IDLE, NULL, NULL },

    /* From IDLE state */
    { STATE_IDLE, EVENT_AI_READY, STATE_ACTIVE, NULL, NULL },
    { STATE_IDLE, EVENT_GROUND_CMD_ACTIVE, STATE_ACTIVE, NULL, NULL },
    { STATE_IDLE, EVENT_POWER_LOW, STATE_SAFE, NULL, NULL },
    { STATE_IDLE, EVENT_POWER_CRITICAL, STATE_SAFE, NULL, NULL },
    { STATE_IDLE, EVENT_THERMAL_FAULT, STATE_SAFE, NULL, NULL },
    { STATE_IDLE, EVENT_COMM_TIMEOUT, STATE_SAFE, NULL, NULL },
    { STATE_IDLE, EVENT_WATCHDOG_TIMEOUT, STATE_SAFE, NULL, NULL },
    { STATE_IDLE, EVENT_GROUND_CMD_SAFE, STATE_SAFE, NULL, NULL },

    /* From ACTIVE state */
    { STATE_ACTIVE, EVENT_AI_FAULT, STATE_IDLE, NULL, NULL },
    { STATE_ACTIVE, EVENT_POWER_LOW, STATE_IDLE, NULL, NULL },
    { STATE_ACTIVE, EVENT_POWER_CRITICAL, STATE_SAFE, NULL, NULL },
    { STATE_ACTIVE, EVENT_THERMAL_FAULT, STATE_SAFE, NULL, NULL },
    { STATE_ACTIVE, EVENT_COMM_TIMEOUT, STATE_IDLE, NULL, NULL },
    { STATE_ACTIVE, EVENT_WATCHDOG_TIMEOUT, STATE_SAFE, NULL, NULL },
    { STATE_ACTIVE, EVENT_GROUND_CMD_IDLE, STATE_IDLE, NULL, NULL },
    { STATE_ACTIVE, EVENT_GROUND_CMD_SAFE, STATE_SAFE, NULL, NULL },

    /* From EMERGENCY state */
    { STATE_EMERGENCY, EVENT_RECOVERY_OK, STATE_SAFE, NULL, NULL },
    { STATE_EMERGENCY, EVENT_GROUND_CMD_SAFE, STATE_SAFE, NULL, NULL },

    /* Any state to EMERGENCY on critical events */
    { STATE_IDLE, EVENT_POWER_CRITICAL, STATE_EMERGENCY, NULL, NULL },
    { STATE_ACTIVE, EVENT_POWER_CRITICAL, STATE_EMERGENCY, NULL, NULL }
};

/** Number of transitions in the table */
static const size_t s_num_transitions = sizeof(s_transitions) / sizeof(s_transitions[0]);

/**
 * @brief Event name strings
 */
static const char *s_event_names[EVENT_COUNT] = {
    "NONE",
    "BOOT_COMPLETE",
    "DETUMBLE_COMPLETE",
    "COMM_ACQUIRED",
    "POWER_OK",
    "POWER_LOW",
    "POWER_CRITICAL",
    "THERMAL_FAULT",
    "COMM_TIMEOUT",
    "WATCHDOG_TIMEOUT",
    "GROUND_CMD_SAFE",
    "GROUND_CMD_IDLE",
    "GROUND_CMD_ACTIVE",
    "AI_READY",
    "AI_FAULT",
    "RECOVERY_OK"
};

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Find a transition in the table
 */
static const SmTransition_t *find_transition(SmState_t from_state, SmEvent_t event)
{
    for (size_t i = 0U; i < s_num_transitions; i++) {
        if ((s_transitions[i].from_state == from_state) &&
            (s_transitions[i].event == event)) {
            return &s_transitions[i];
        }
    }
    return NULL;
}

/**
 * @brief Execute state exit action
 */
static SmartQsoResult_t execute_exit_action(SmState_t state)
{
    if ((uint32_t)state >= STATE_COUNT) {
        return SMART_QSO_ERROR_INVALID;
    }

    if (s_state_configs[state].on_exit != NULL) {
        return s_state_configs[state].on_exit();
    }

    return SMART_QSO_OK;
}

/**
 * @brief Execute state entry action
 */
static SmartQsoResult_t execute_entry_action(SmState_t state)
{
    if ((uint32_t)state >= STATE_COUNT) {
        return SMART_QSO_ERROR_INVALID;
    }

    if (s_state_configs[state].on_entry != NULL) {
        return s_state_configs[state].on_entry();
    }

    return SMART_QSO_OK;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t sm_init(SmContext_t *ctx, SmState_t initial_state)
{
    if (ctx == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if ((uint32_t)initial_state >= STATE_COUNT) {
        return SMART_QSO_ERROR_INVALID;
    }

    ctx->current_state = initial_state;
    ctx->previous_state = initial_state;
    ctx->transition_count = 0U;
    ctx->last_transition_ms = smart_qso_now_ms();
    ctx->state_entry_ms = ctx->last_transition_ms;
    ctx->is_initialized = true;

    /* Execute entry action for initial state */
    (void)execute_entry_action(initial_state);

    return SMART_QSO_OK;
}

SmartQsoResult_t sm_process_event(SmContext_t *ctx,
                                   SmEvent_t event,
                                   SmResult_t *result)
{
    SmResult_t local_result = SM_RESULT_OK;

    if (ctx == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!ctx->is_initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    if ((uint32_t)event >= EVENT_COUNT) {
        local_result = SM_RESULT_INVALID_EVENT;
        if (result != NULL) {
            *result = local_result;
        }
        return SMART_QSO_OK;
    }

    /* Find applicable transition */
    const SmTransition_t *trans = find_transition(ctx->current_state, event);

    if (trans == NULL) {
        local_result = SM_RESULT_NO_TRANSITION;
        if (result != NULL) {
            *result = local_result;
        }
        return SMART_QSO_OK;
    }

    /* Check guard condition if present */
    if (trans->guard != NULL) {
        if (!trans->guard()) {
            local_result = SM_RESULT_GUARD_FAILED;
            if (result != NULL) {
                *result = local_result;
            }
            return SMART_QSO_OK;
        }
    }

    /* Execute exit action for current state */
    SmartQsoResult_t action_result = execute_exit_action(ctx->current_state);
    if (action_result != SMART_QSO_OK) {
        /* Log but continue with transition */
        (void)fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_WARNING,
                            "Exit action failed", 0.0);
    }

    /* Execute transition action if present */
    if (trans->action != NULL) {
        action_result = trans->action();
        if (action_result != SMART_QSO_OK) {
            local_result = SM_RESULT_ACTION_FAILED;
            (void)fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_WARNING,
                                "Transition action failed", 0.0);
        }
    }

    /* Update state */
    ctx->previous_state = ctx->current_state;
    ctx->current_state = trans->to_state;
    ctx->transition_count++;
    ctx->last_transition_ms = smart_qso_now_ms();
    ctx->state_entry_ms = ctx->last_transition_ms;

    /* Log the transition */
    (void)fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_INFO,
                        sm_get_state_name(ctx->current_state), 0.0);

    /* Execute entry action for new state */
    action_result = execute_entry_action(ctx->current_state);
    if (action_result != SMART_QSO_OK) {
        (void)fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_WARNING,
                            "Entry action failed", 0.0);
    }

    if (result != NULL) {
        *result = local_result;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t sm_get_state(const SmContext_t *ctx, SmState_t *state)
{
    if ((ctx == NULL) || (state == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!ctx->is_initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    *state = ctx->current_state;
    return SMART_QSO_OK;
}

const char *sm_get_state_name(SmState_t state)
{
    if ((uint32_t)state >= STATE_COUNT) {
        return "UNKNOWN";
    }

    return s_state_configs[state].name;
}

const char *sm_get_event_name(SmEvent_t event)
{
    if ((uint32_t)event >= EVENT_COUNT) {
        return "UNKNOWN";
    }

    return s_event_names[event];
}

SmartQsoResult_t sm_get_state_duration(const SmContext_t *ctx,
                                        uint64_t *duration_ms)
{
    if ((ctx == NULL) || (duration_ms == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!ctx->is_initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    uint64_t now = smart_qso_now_ms();
    *duration_ms = now - ctx->state_entry_ms;

    return SMART_QSO_OK;
}

SmartQsoResult_t sm_is_transition_valid(const SmContext_t *ctx,
                                         SmEvent_t event,
                                         bool *is_valid)
{
    if ((ctx == NULL) || (is_valid == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!ctx->is_initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    const SmTransition_t *trans = find_transition(ctx->current_state, event);
    *is_valid = (trans != NULL);

    return SMART_QSO_OK;
}

SmartQsoResult_t sm_force_state(SmContext_t *ctx, SmState_t new_state)
{
    if (ctx == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if ((uint32_t)new_state >= STATE_COUNT) {
        return SMART_QSO_ERROR_INVALID;
    }

    /* Log forced transition */
    (void)fault_log_add(FAULT_TYPE_RECOVERY, FAULT_SEVERITY_WARNING,
                        "Forced state change", 0.0);

    /* Execute exit action */
    (void)execute_exit_action(ctx->current_state);

    /* Update state */
    ctx->previous_state = ctx->current_state;
    ctx->current_state = new_state;
    ctx->transition_count++;
    ctx->last_transition_ms = smart_qso_now_ms();
    ctx->state_entry_ms = ctx->last_transition_ms;

    /* Execute entry action */
    (void)execute_entry_action(new_state);

    return SMART_QSO_OK;
}

SmartQsoResult_t sm_get_statistics(const SmContext_t *ctx,
                                    uint32_t *transition_count,
                                    SmState_t *previous_state)
{
    if (ctx == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!ctx->is_initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    if (transition_count != NULL) {
        *transition_count = ctx->transition_count;
    }

    if (previous_state != NULL) {
        *previous_state = ctx->previous_state;
    }

    return SMART_QSO_OK;
}
