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

/** Command authorization code */
#define CMD_AUTH_CODE           0xDEADBEEFU

/** Commands requiring authorization */
#define CMD_AUTH_REQUIRED_MASK  0xF0  /* Reset, deploy, etc. */

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

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t cmd_handler_init(void)
{
    /* Clear statistics */
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));

    /* Clear custom handlers */
    (void)safe_memset(s_custom_handlers, sizeof(s_custom_handlers), 0, sizeof(s_custom_handlers));

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

    /* Validate command structure */
    CommandValidation_t validation;
    SmartQsoResult_t result = input_validate_command(raw_data, data_len, &validation);
    if (result != SMART_QSO_OK) {
        return result;
    }

    s_stats.total_received++;

    if (!validation.is_valid) {
        s_stats.total_rejected++;
        response->result = CMD_RESULT_INVALID_CMD;
        (void)fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_WARNING,
                            "Invalid command received", 0.0);
        return SMART_QSO_OK;
    }

    /* Build command structure */
    Command_t cmd;
    cmd.cmd_id = validation.cmd_id;
    cmd.payload_len = validation.payload_length;
    cmd.timestamp_ms = (uint32_t)smart_qso_now_ms();
    cmd.requires_ack = true;
    cmd.sequence_num = raw_data[2];  /* Assuming sequence in header */

    /* Copy payload */
    if (cmd.payload_len > 0U) {
        size_t copy_len = (cmd.payload_len > 64U) ? 64U : cmd.payload_len;
        (void)safe_memcpy(cmd.payload, sizeof(cmd.payload), &raw_data[4], copy_len);
    }

    /* Validate payload for this command type */
    bool payload_valid = false;
    result = input_validate_command_payload(cmd.cmd_id, cmd.payload, cmd.payload_len, &payload_valid);
    if ((result != SMART_QSO_OK) || !payload_valid) {
        s_stats.total_rejected++;
        response->result = CMD_RESULT_INVALID_PARAM;
        return SMART_QSO_OK;
    }

    /* Execute command */
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

    /* Check authorization if required */
    if (cmd_requires_auth(cmd->cmd_id)) {
        if (!cmd_verify_auth(cmd)) {
            response->result = CMD_RESULT_AUTH_FAIL;
            s_stats.auth_failures++;
            (void)fault_log_add(FAULT_TYPE_UART, FAULT_SEVERITY_WARNING,
                                "Command auth failed", 0.0);
            return SMART_QSO_OK;
        }
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

bool cmd_requires_auth(uint8_t cmd_id)
{
    /* Critical commands require authorization */
    switch (cmd_id) {
        case CMD_SYS_RESET:
        case CMD_SYS_CLEAR_FAULTS:
        case CMD_PLD_ENABLE:
            return true;
        default:
            return false;
    }
}

bool cmd_verify_auth(const Command_t *cmd)
{
    if (cmd == NULL) {
        return false;
    }

    /* Auth code must be in first 4 bytes of payload */
    if (cmd->payload_len < 4U) {
        return false;
    }

    uint32_t auth_code = ((uint32_t)cmd->payload[0] << 24) |
                         ((uint32_t)cmd->payload[1] << 16) |
                         ((uint32_t)cmd->payload[2] << 8) |
                         (uint32_t)cmd->payload[3];

    return (auth_code == CMD_AUTH_CODE);
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
            if (cmd->payload_len >= 1U) {
                SmState_t new_state;
                switch (cmd->payload[0]) {
                    case 0: new_state = STATE_SAFE; break;
                    case 1: new_state = STATE_IDLE; break;
                    case 2: new_state = STATE_ACTIVE; break;
                    default: return CMD_RESULT_INVALID_PARAM;
                }
                (void)sm_force_state(sys_get_sm_context(), new_state);
                return CMD_RESULT_SUCCESS;
            }
            return CMD_RESULT_INVALID_PARAM;

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
