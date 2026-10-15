/**
 * @file cmd_handler.c
 * @brief Command handler implementation for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Implements ground command processing following NASA Ames SmallSat heritage.
 */

#include "cmd_handler.h"
#include "cmd_auth.h"
#include "cmd_dictionary.h"
#include "rf_control.h"
#include "contact_monitor.h"
#include "input_validation.h"
#include "fault_mgmt.h"
#include "system_state.h"
#include "safe_string.h"
#include <stddef.h>

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Maximum number of custom handlers */
#define MAX_CUSTOM_HANDLERS     16U

/** Offset of the payload in an uplink frame: sync (2) | cmd_id | length */
#define FRAME_PAYLOAD_OFFSET    4U

_Static_assert(sizeof(((Command_t *)0)->payload) >= CMD_AUTH_MAX_ARGS,
               "Command_t.payload must hold the largest authenticated argument block");

/*******************************************************************************
 * Private Types
 ******************************************************************************/

/**
 * @brief Custom handler entry
 */
typedef struct {
    uint8_t cmd_id;
    CmdHandlerFunc_t handler;
    bool in_use;
} CustomHandler_t;

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Command statistics */
static CmdStats_t s_stats;

/** A rejection has been logged since the last accepted command; further
 *  rejections are only counted, so forged frames cannot drive fault-log
 *  flash writes */
static bool s_reject_logged = false;

/** Custom handlers */
static CustomHandler_t s_custom_handlers[MAX_CUSTOM_HANDLERS];

/** Initialization flag */
static bool s_initialized = false;

/*******************************************************************************
 * Private Function Declarations
 ******************************************************************************/

static CmdResult_t handle_system_cmd(const Command_t *cmd, CmdResponse_t *response);
static CmdResult_t handle_eps_cmd(const Command_t *cmd, CmdResponse_t *response);
static CmdResult_t handle_adcs_cmd(const Command_t *cmd, CmdResponse_t *response);
static CmdResult_t handle_comm_cmd(const Command_t *cmd, CmdResponse_t *response);
static CmdResult_t handle_payload_cmd(const Command_t *cmd, CmdResponse_t *response);
static CmdHandlerFunc_t find_custom_handler(uint8_t cmd_id);
static bool cmd_args_spec(uint8_t cmd_id, size_t *min_len, size_t *max_len);
static CmdResult_t set_mode(const Command_t *cmd);
static void reject(CmdResponse_t *response, CmdResult_t result, const char *reason);

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t cmd_handler_init(void)
{
    /* Clear statistics */
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));

    /* Clear custom handlers */
    (void)safe_memset(s_custom_handlers, sizeof(s_custom_handlers), 0, sizeof(s_custom_handlers));

    s_reject_logged = false;

    /* Load the telecommand key and replay counter (unprovisioned: fail closed) */
    (void)cmd_auth_init();

    s_initialized = true;

    return SMART_QSO_OK;
}

SmartQsoResult_t cmd_process(const uint8_t *raw_data,
                              size_t data_len,
                              CmdResponse_t *response)
{
    if ((raw_data == NULL) || (response == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    (void)safe_memset(response, sizeof(*response), 0, sizeof(*response));

    /* Framing: sync, length and checksum */
    CommandValidation_t validation;
    SmartQsoResult_t result = input_validate_command(raw_data, data_len, &validation);
    if (result != SMART_QSO_OK) {
        return result;
    }

    s_stats.total_received++;
    response->cmd_id = validation.cmd_id;

    if (!validation.is_valid) {
        reject(response, CMD_RESULT_INVALID_CMD, "Invalid command frame");
        return SMART_QSO_OK;
    }

    /* Authentication: tag over cmd_id, args and counter; fresh counter */
    CmdAuthVerified_t verified;
    CmdAuthStatus_t auth = cmd_auth_verify(validation.cmd_id,
                                           &raw_data[FRAME_PAYLOAD_OFFSET],
                                           validation.payload_length, &verified);
    if (auth == CMD_AUTH_ERR_FORMAT) {
        reject(response, CMD_RESULT_INVALID_PARAM, "Telecommand payload malformed");
        return SMART_QSO_OK;
    }
    if (auth != CMD_AUTH_OK) {
        s_stats.auth_failures++;
        reject(response, CMD_RESULT_AUTH_FAIL, "Telecommand authentication failed");
        return SMART_QSO_OK;
    }

    response->sequence_num = (uint8_t)verified.counter;

    /* Argument length must match the command dictionary */
    size_t min_len = 0U;
    size_t max_len = 0U;
    if (!cmd_args_spec(validation.cmd_id, &min_len, &max_len)) {
        reject(response, CMD_RESULT_INVALID_CMD, "Unknown command ID");
        return SMART_QSO_OK;
    }
    if ((verified.args_len < min_len) || (verified.args_len > max_len)) {
        reject(response, CMD_RESULT_INVALID_PARAM, "Command argument length");
        return SMART_QSO_OK;
    }

    /* Persist the counter before executing so a replay fails even after a
     * reset. TX-OFF still executes if persistence fails: reducing emissions
     * is always safe, and a replayed TX-OFF can only silence the station. */
    SmartQsoResult_t committed = cmd_auth_commit(&verified);

    /* An authenticated frame whose counter was just spent is contact with
     * the ground, whatever the command's outcome. Earlier rejections do not
     * count: their counter is unspent, so a replay could otherwise hold off
     * the loss-of-contact inhibit indefinitely. The keyless TX_DISABLE never
     * counts. */
    if (verified.key != CMD_AUTH_KEY_NONE) {
        contact_monitor_contact();
    }

    if ((committed != SMART_QSO_OK) &&
        (validation.cmd_id != (uint8_t)CMD_COMM_TX_DISABLE)) {
        reject(response, CMD_RESULT_EXEC_FAIL, "Command counter not persisted");
        return SMART_QSO_OK;
    }

    s_reject_logged = false;

    Command_t cmd;
    (void)safe_memset(&cmd, sizeof(cmd), 0, sizeof(cmd));
    cmd.cmd_id = validation.cmd_id;
    cmd.sequence_num = (uint8_t)verified.counter;
    cmd.payload_len = (uint8_t)verified.args_len;
    cmd.timestamp_ms = (uint32_t)smart_qso_now_ms();
    cmd.requires_ack = true;
    if (verified.args_len > 0U) {
        (void)safe_memcpy(cmd.payload, sizeof(cmd.payload), verified.args, verified.args_len);
    }

    return cmd_execute(&cmd, response);
}

SmartQsoResult_t cmd_execute(const Command_t *cmd, CmdResponse_t *response)
{
    if ((cmd == NULL) || (response == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    /* Initialize response */
    response->cmd_id = cmd->cmd_id;
    response->sequence_num = cmd->sequence_num;
    response->result = CMD_RESULT_SUCCESS;
    response->data_len = 0;

    /* Check if command is allowed in current state */
    if (!cmd_is_allowed(cmd->cmd_id)) {
        response->result = CMD_RESULT_NOT_ALLOWED;
        s_stats.total_rejected++;
        (void)fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_WARNING,
                            "Command not allowed in state", 0.0);
        return SMART_QSO_OK;
    }

    /* Check for custom handler first */
    CmdHandlerFunc_t custom = find_custom_handler(cmd->cmd_id);
    if (custom != NULL) {
        response->result = custom(cmd, response);
    } else {
        /* Route to category handler */
        uint8_t category = cmd->cmd_id & 0xF0U;

        switch (category) {
            case CMD_CAT_SYSTEM:
                response->result = handle_system_cmd(cmd, response);
                break;

            case CMD_CAT_EPS:
                response->result = handle_eps_cmd(cmd, response);
                break;

            case CMD_CAT_ADCS:
                response->result = handle_adcs_cmd(cmd, response);
                break;

            case CMD_CAT_COMM:
                response->result = handle_comm_cmd(cmd, response);
                break;

            case CMD_CAT_PAYLOAD:
                response->result = handle_payload_cmd(cmd, response);
                break;

            default:
                response->result = CMD_RESULT_INVALID_CMD;
                break;
        }
    }

    /* Update statistics */
    s_stats.last_cmd_time_ms = cmd->timestamp_ms;
    s_stats.last_cmd_id = cmd->cmd_id;
    s_stats.last_result = response->result;

    if (response->result == CMD_RESULT_SUCCESS) {
        s_stats.total_executed++;
        (void)sys_increment_command_count();
    } else {
        s_stats.total_rejected++;
    }

    return SMART_QSO_OK;
}

bool cmd_is_allowed(uint8_t cmd_id)
{
    SmState_t state = sys_get_operational_state();

    /* NOP always allowed */
    if (cmd_id == CMD_SYS_NOP) {
        return true;
    }

    /* Transmitter on/off must work in every state (47 CFR 97.207(b)), and
     * so must key rotation, which restores control after a key compromise */
    if ((cmd_id == CMD_COMM_TX_DISABLE) || (cmd_id == CMD_COMM_TX_ENABLE) ||
        (cmd_id == CMD_SYS_KEY_ROTATE)) {
        return true;
    }

    /* Status queries always allowed */
    if ((cmd_id == CMD_SYS_GET_STATUS) || (cmd_id == CMD_EPS_GET_TELEMETRY) ||
        (cmd_id == CMD_ADCS_GET_ATTITUDE)) {
        return true;
    }

    /* In SAFE mode, only limited commands allowed */
    if (state == STATE_SAFE) {
        switch (cmd_id) {
            case CMD_SYS_GET_STATUS:
            case CMD_SYS_SET_MODE:
            case CMD_EPS_GET_TELEMETRY:
            case CMD_COMM_SET_BEACON:
                return true;
            default:
                return false;
        }
    }

    /* In EMERGENCY mode, very limited commands */
    if (state == STATE_EMERGENCY) {
        return (cmd_id == CMD_SYS_GET_STATUS) || (cmd_id == CMD_SYS_RESET);
    }

    /* Most commands allowed in IDLE and ACTIVE */
    return true;
}

SmartQsoResult_t cmd_get_stats(CmdStats_t *stats)
{
    if (stats == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(stats, sizeof(*stats), &s_stats, sizeof(s_stats));
    return SMART_QSO_OK;
}

SmartQsoResult_t cmd_clear_stats(void)
{
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));
    return SMART_QSO_OK;
}

SmartQsoResult_t cmd_register_handler(uint8_t cmd_id, CmdHandlerFunc_t handler)
{
    if (handler == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    /* Find empty slot */
    for (size_t i = 0; i < MAX_CUSTOM_HANDLERS; i++) {
        if (!s_custom_handlers[i].in_use) {
            s_custom_handlers[i].cmd_id = cmd_id;
            s_custom_handlers[i].handler = handler;
            s_custom_handlers[i].in_use = true;
            return SMART_QSO_OK;
        }
    }

    return SMART_QSO_ERROR_NO_MEM;
}

const char *cmd_get_name(uint8_t cmd_id)
{
    switch (cmd_id) {
        case CMD_SYS_NOP:         return "NOP";
        case CMD_SYS_RESET:       return "RESET";
        case CMD_SYS_SET_MODE:    return "SET_MODE";
        case CMD_SYS_GET_STATUS:  return "GET_STATUS";
        case CMD_SYS_SET_TIME:    return "SET_TIME";
        case CMD_SYS_CLEAR_FAULTS: return "CLEAR_FAULTS";
        case CMD_EPS_SET_MODE:    return "EPS_SET_MODE";
        case CMD_EPS_ENABLE_HEATER: return "EPS_ENABLE_HEATER";
        case CMD_EPS_DISABLE_HEATER: return "EPS_DISABLE_HEATER";
        case CMD_EPS_GET_TELEMETRY: return "EPS_GET_TLM";
        case CMD_ADCS_SET_MODE:   return "ADCS_SET_MODE";
        case CMD_ADCS_DETUMBLE:   return "ADCS_DETUMBLE";
        case CMD_ADCS_POINT_SUN:  return "ADCS_POINT_SUN";
        case CMD_ADCS_GET_ATTITUDE: return "ADCS_GET_ATT";
        case CMD_COMM_SET_BEACON: return "COMM_SET_BEACON";
        case CMD_COMM_TX_ENABLE:  return "COMM_TX_ENABLE";
        case CMD_COMM_TX_DISABLE: return "COMM_TX_DISABLE";
        case CMD_COMM_SET_POWER:  return "COMM_SET_POWER";
        case CMD_PLD_ENABLE:      return "PLD_ENABLE";
        case CMD_PLD_DISABLE:     return "PLD_DISABLE";
        default:                  return "UNKNOWN";
    }
}

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

static CmdHandlerFunc_t find_custom_handler(uint8_t cmd_id)
{
    for (size_t i = 0; i < MAX_CUSTOM_HANDLERS; i++) {
        if (s_custom_handlers[i].in_use && (s_custom_handlers[i].cmd_id == cmd_id)) {
            return s_custom_handlers[i].handler;
        }
    }
    return NULL;
}

static CmdResult_t handle_system_cmd(const Command_t *cmd, CmdResponse_t *response)
{
    switch (cmd->cmd_id) {
        case CMD_SYS_NOP:
            /* No operation - just acknowledge */
            return CMD_RESULT_SUCCESS;

        case CMD_SYS_RESET:
            /* System reset - would trigger watchdog or HW reset */
            (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_INFO,
                                "Reset commanded", 0.0);
            /* In real HW: hal_system_reset(); */
            return CMD_RESULT_SUCCESS;

        case CMD_SYS_SET_MODE:
            return set_mode(cmd);

        case CMD_SYS_GET_STATUS:
            /* Return system status in response */
            response->data[0] = (uint8_t)sys_get_operational_state();
            response->data[1] = (uint8_t)sys_get_power_mode();
            response->data[2] = sys_is_payload_enabled() ? 1U : 0U;
            response->data[3] = sys_has_thermal_fault() ? 1U : 0U;
            response->data_len = 4;
            return CMD_RESULT_SUCCESS;

        case CMD_SYS_CLEAR_FAULTS:
            (void)fault_log_clear();
            return CMD_RESULT_SUCCESS;

        case CMD_SYS_SET_CONTACT_TIMEOUT: {
            uint16_t days = (uint16_t)(((uint16_t)cmd->payload[0] << 8) | (uint16_t)cmd->payload[1]);
            SmartQsoResult_t set = contact_monitor_set_timeout_days(days);
            if (set == SMART_QSO_OK) {
                return CMD_RESULT_SUCCESS;
            }
            return (set == SMART_QSO_ERROR_INVALID) ? CMD_RESULT_INVALID_PARAM : CMD_RESULT_EXEC_FAIL;
        }

        case CMD_SYS_KEY_ROTATE: {
            /* Authenticated by the recovery key (dictionary key class) */
            SmartQsoResult_t rotated = cmd_auth_rotate(cmd->payload[0]);
            if (rotated == SMART_QSO_OK) {
                return CMD_RESULT_SUCCESS;
            }
            return (rotated == SMART_QSO_ERROR_INVALID) ? CMD_RESULT_INVALID_PARAM
                                                        : CMD_RESULT_EXEC_FAIL;
        }

        default:
            return CMD_RESULT_INVALID_CMD;
    }
}

static CmdResult_t handle_eps_cmd(const Command_t *cmd, CmdResponse_t *response)
{
    switch (cmd->cmd_id) {
        case CMD_EPS_SET_MODE:
            if (cmd->payload_len >= 1U) {
                PowerMode_t mode = (PowerMode_t)cmd->payload[0];
                if (mode <= POWER_MODE_ACTIVE) {
                    (void)sys_set_power_mode(mode);
                    return CMD_RESULT_SUCCESS;
                }
            }
            return CMD_RESULT_INVALID_PARAM;

        case CMD_EPS_GET_TELEMETRY:
            {
                PowerState_t power;
                (void)sys_get_power_state(&power);
                /* Pack telemetry into response */
                response->data[0] = (uint8_t)(power.battery_voltage * 10.0);  /* 0.1V resolution */
                response->data[1] = (uint8_t)((power.state_of_charge * 100.0));  /* % */
                response->data[2] = (uint8_t)power.power_mode;
                response->data_len = 3;
            }
            return CMD_RESULT_SUCCESS;

        default:
            return CMD_RESULT_INVALID_CMD;
    }
}

static CmdResult_t handle_adcs_cmd(const Command_t *cmd, CmdResponse_t *response)
{
    switch (cmd->cmd_id) {
        case CMD_ADCS_DETUMBLE:
            /* Start detumble sequence */
            (void)sm_process_event(sys_get_sm_context(), EVENT_BOOT_COMPLETE, NULL);
            return CMD_RESULT_SUCCESS;

        case CMD_ADCS_GET_ATTITUDE:
            {
                AdcsState_t adcs;
                (void)sys_get_adcs_state(&adcs);
                /* Pack attitude data */
                response->data[0] = adcs.detumbled ? 1U : 0U;
                response->data[1] = adcs.sun_acquired ? 1U : 0U;
                response->data_len = 2;
            }
            return CMD_RESULT_SUCCESS;

        default:
            return CMD_RESULT_INVALID_CMD;
    }
}

static CmdResult_t handle_comm_cmd(const Command_t *cmd, CmdResponse_t *response)
{
    (void)response;

    switch (cmd->cmd_id) {
        case CMD_COMM_SET_BEACON:
            if (cmd->payload_len >= 2U) {
                uint16_t interval = (uint16_t)(((uint16_t)cmd->payload[0] << 8) | (uint16_t)cmd->payload[1]);
                SmartQsoResult_t result = sys_set_beacon_interval(interval);
                return (result == SMART_QSO_OK) ? CMD_RESULT_SUCCESS : CMD_RESULT_INVALID_PARAM;
            }
            return CMD_RESULT_INVALID_PARAM;

        case CMD_COMM_TX_DISABLE:
            /* Hardware line is dropped before persistence, so the
             * transmitter is off even if the latch cannot be saved */
            (void)rf_control_tx_off();
            return CMD_RESULT_SUCCESS;

        case CMD_COMM_TX_ENABLE:
            return (rf_control_tx_on() == SMART_QSO_OK) ? CMD_RESULT_SUCCESS : CMD_RESULT_EXEC_FAIL;

        case CMD_COMM_SET_POWER: {
            SmartQsoResult_t set = rf_control_set_power_level((RfPowerLevel_t)cmd->payload[0]);
            if (set == SMART_QSO_OK) {
                return CMD_RESULT_SUCCESS;
            }
            return (set == SMART_QSO_ERROR_INVALID) ? CMD_RESULT_INVALID_PARAM : CMD_RESULT_EXEC_FAIL;
        }

        default:
            return CMD_RESULT_INVALID_CMD;
    }
}

static CmdResult_t handle_payload_cmd(const Command_t *cmd, CmdResponse_t *response)
{
    (void)response;

    switch (cmd->cmd_id) {
        case CMD_PLD_ENABLE:
            (void)sys_set_payload_enabled(true);
            return CMD_RESULT_SUCCESS;

        case CMD_PLD_DISABLE:
            (void)sys_set_payload_enabled(false);
            return CMD_RESULT_SUCCESS;

        default:
            return CMD_RESULT_INVALID_CMD;
    }
}

/**
 * Allowed argument length for a command, from the generated dictionary
 * (software/flight/command_dictionary.json, SQSO-ICD-013 Section 7). IDs that
 * are defined in cmd_handler.h but have no handler are deliberately absent,
 * so they are refused before a counter is consumed.
 */
static bool cmd_args_spec(uint8_t cmd_id, size_t *min_len, size_t *max_len)
{
    for (size_t i = 0U; i < g_cmd_dictionary_count; i++) {
        if (g_cmd_dictionary[i].cmd_id == cmd_id) {
            *min_len = g_cmd_dictionary[i].min_args;
            *max_len = g_cmd_dictionary[i].max_args;
            return true;
        }
    }

    /* Run-time registered handlers define their own arguments */
    if (find_custom_handler(cmd_id) != NULL) {
        *min_len = 0U;
        *max_len = CMD_AUTH_MAX_ARGS;
        return true;
    }

    return false;
}

/** Record a rejected command; only the first of a run is logged */
static void reject(CmdResponse_t *response, CmdResult_t result, const char *reason)
{
    response->result = result;
    s_stats.total_rejected++;
    s_stats.last_result = result;
    if (!s_reject_logged) {
        s_reject_logged = true;
        (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_WARNING, reason, 0.0);
    }
}

/**
 * Ground mode change through the state machine's transition table, so the
 * ground obeys the same rules as onboard events (SAFE -> ACTIVE needs IDLE
 * first). Requesting the current state succeeds without a transition.
 */
static CmdResult_t set_mode(const Command_t *cmd)
{
    SmState_t target;
    SmEvent_t event;

    if (cmd->payload_len != 1U) {
        return CMD_RESULT_INVALID_PARAM;
    }
    switch (cmd->payload[0]) {
        case 0U: target = STATE_SAFE;   event = EVENT_GROUND_CMD_SAFE;   break;
        case 1U: target = STATE_IDLE;   event = EVENT_GROUND_CMD_IDLE;   break;
        case 2U: target = STATE_ACTIVE; event = EVENT_GROUND_CMD_ACTIVE; break;
        default: return CMD_RESULT_INVALID_PARAM;
    }

    SmContext_t *ctx = sys_get_sm_context();
    SmState_t current = STATE_BOOT;
    if (sm_get_state(ctx, &current) != SMART_QSO_OK) {
        return CMD_RESULT_EXEC_FAIL;
    }
    if (current == target) {
        return CMD_RESULT_SUCCESS;
    }

    SmResult_t sm_result = SM_RESULT_NO_TRANSITION;
    if (sm_process_event(ctx, event, &sm_result) != SMART_QSO_OK) {
        return CMD_RESULT_EXEC_FAIL;
    }
    if ((sm_result == SM_RESULT_NO_TRANSITION) || (sm_result == SM_RESULT_GUARD_FAILED)) {
        return CMD_RESULT_NOT_ALLOWED;
    }
    return (sm_result == SM_RESULT_OK) ? CMD_RESULT_SUCCESS : CMD_RESULT_EXEC_FAIL;
}
