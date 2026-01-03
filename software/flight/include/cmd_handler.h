/**
 * @file cmd_handler.h
 * @brief Command handler interface for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module handles all ground commands and internal command dispatching.
 * Follows NASA Ames SmallSat heritage (GeneSat, PharmaSat, EcAMSat).
 *
 * @requirement SRS-CMD-001 System shall process ground commands
 * @requirement SRS-CMD-002 Commands shall be validated before execution
 * @requirement SRS-CMD-003 Command execution shall be logged
 */

#ifndef SMART_QSO_CMD_HANDLER_H
#define SMART_QSO_CMD_HANDLER_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include "input_validation.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Command Definitions
 ******************************************************************************/

/**
 * @brief Command categories
 */
typedef enum {
    CMD_CAT_SYSTEM      = 0x00,  /**< System commands (reset, mode) */
    CMD_CAT_EPS         = 0x10,  /**< EPS commands */
    CMD_CAT_ADCS        = 0x20,  /**< ADCS commands */
    CMD_CAT_COMM        = 0x30,  /**< Communications commands */
    CMD_CAT_PAYLOAD     = 0x40,  /**< Payload commands */
    CMD_CAT_FILE        = 0x50,  /**< File transfer commands */
    CMD_CAT_DEBUG       = 0xF0   /**< Debug commands (disabled in flight) */
} CmdCategory_t;

/**
 * @brief System commands
 */
typedef enum {
    CMD_SYS_NOP         = 0x00,  /**< No operation */
    CMD_SYS_RESET       = 0x01,  /**< System reset */
    CMD_SYS_SET_MODE    = 0x02,  /**< Set operational mode */
    CMD_SYS_GET_STATUS  = 0x03,  /**< Get system status */
    CMD_SYS_SET_TIME    = 0x04,  /**< Set mission time */
    CMD_SYS_CLEAR_FAULTS = 0x05  /**< Clear fault log */
} CmdSystem_t;

/**
 * @brief EPS commands
 */
typedef enum {
    CMD_EPS_SET_MODE    = 0x10,  /**< Set power mode */
    CMD_EPS_ENABLE_HEATER = 0x11, /**< Enable heater */
    CMD_EPS_DISABLE_HEATER = 0x12, /**< Disable heater */
    CMD_EPS_GET_TELEMETRY = 0x13  /**< Get EPS telemetry */
} CmdEps_t;

/**
 * @brief ADCS commands
 */
typedef enum {
    CMD_ADCS_SET_MODE   = 0x20,  /**< Set ADCS mode */
    CMD_ADCS_DETUMBLE   = 0x21,  /**< Start detumble */
    CMD_ADCS_POINT_SUN  = 0x22,  /**< Point to sun */
    CMD_ADCS_GET_ATTITUDE = 0x23 /**< Get attitude data */
} CmdAdcs_t;

/**
 * @brief Communications commands
 */
typedef enum {
    CMD_COMM_SET_BEACON = 0x30,  /**< Set beacon parameters */
    CMD_COMM_TX_ENABLE  = 0x31,  /**< Enable transmitter */
    CMD_COMM_TX_DISABLE = 0x32,  /**< Disable transmitter */
    CMD_COMM_SET_POWER  = 0x33   /**< Set TX power */
} CmdComm_t;

/**
 * @brief Payload commands
 */
typedef enum {
    CMD_PLD_ENABLE      = 0x40,  /**< Enable Jetson payload */
    CMD_PLD_DISABLE     = 0x41,  /**< Disable Jetson payload */
    CMD_PLD_SEND_DATA   = 0x42,  /**< Send data to payload */
    CMD_PLD_REQUEST_DATA = 0x43  /**< Request data from payload */
} CmdPayload_t;

/**
 * @brief Command execution result
 */
typedef enum {
    CMD_RESULT_SUCCESS      = 0,  /**< Command executed successfully */
    CMD_RESULT_INVALID_CMD  = 1,  /**< Invalid command ID */
    CMD_RESULT_INVALID_PARAM = 2, /**< Invalid parameter */
    CMD_RESULT_AUTH_FAIL    = 3,  /**< Authorization failed */
    CMD_RESULT_NOT_ALLOWED  = 4,  /**< Command not allowed in current state */
    CMD_RESULT_EXEC_FAIL    = 5,  /**< Execution failed */
    CMD_RESULT_BUSY         = 6   /**< System busy */
} CmdResult_t;

/*******************************************************************************
 * Type Definitions
 ******************************************************************************/

/**
 * @brief Command structure
 */
typedef struct {
    uint8_t cmd_id;             /**< Command ID */
    uint8_t sequence_num;       /**< Sequence number */
    uint8_t payload_len;        /**< Payload length */
    uint8_t payload[64];        /**< Command payload */
    uint32_t timestamp_ms;      /**< Reception timestamp */
    bool requires_ack;          /**< Acknowledgment required */
} Command_t;

/**
 * @brief Command response structure
 */
typedef struct {
    uint8_t cmd_id;             /**< Original command ID */
    uint8_t sequence_num;       /**< Original sequence number */
    CmdResult_t result;         /**< Execution result */
    uint8_t data_len;           /**< Response data length */
    uint8_t data[64];           /**< Response data */
} CmdResponse_t;

/**
 * @brief Command handler function type
 */
typedef CmdResult_t (*CmdHandlerFunc_t)(const Command_t *cmd, CmdResponse_t *response);

/**
 * @brief Command statistics
 */
typedef struct {
    uint32_t total_received;    /**< Total commands received */
    uint32_t total_executed;    /**< Successfully executed */
    uint32_t total_rejected;    /**< Rejected commands */
    uint32_t auth_failures;     /**< Authorization failures */
    uint32_t last_cmd_time_ms;  /**< Last command timestamp */
    uint8_t last_cmd_id;        /**< Last command ID */
    CmdResult_t last_result;    /**< Last execution result */
} CmdStats_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize command handler
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t cmd_handler_init(void);

/**
 * @brief Process a received command
 *
 * @param[in] raw_data Raw command data
 * @param[in] data_len Length of raw data
 * @param[out] response Command response
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t cmd_process(const uint8_t *raw_data,
                              size_t data_len,
                              CmdResponse_t *response);

/**
 * @brief Execute a validated command
 *
 * @param[in] cmd Command to execute
 * @param[out] response Execution response
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t cmd_execute(const Command_t *cmd, CmdResponse_t *response);

/**
 * @brief Check if command is allowed in current state
 *
 * @param[in] cmd_id Command ID
 * @return true if command is allowed
 */
bool cmd_is_allowed(uint8_t cmd_id);

/**
 * @brief Check if command requires authorization
 *
 * @param[in] cmd_id Command ID
 * @return true if authorization required
 */
bool cmd_requires_auth(uint8_t cmd_id);

/**
 * @brief Verify command authorization
 *
 * @param[in] cmd Command with auth data
 * @return true if authorized
 */
bool cmd_verify_auth(const Command_t *cmd);

/**
 * @brief Get command statistics
 *
 * @param[out] stats Command statistics
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t cmd_get_stats(CmdStats_t *stats);

/**
 * @brief Clear command statistics
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t cmd_clear_stats(void);

/**
 * @brief Register custom command handler
 *
 * @param[in] cmd_id Command ID
 * @param[in] handler Handler function
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t cmd_register_handler(uint8_t cmd_id, CmdHandlerFunc_t handler);

/**
 * @brief Get command name string
 *
 * @param[in] cmd_id Command ID
 * @return Command name or "UNKNOWN"
 */
const char *cmd_get_name(uint8_t cmd_id);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_CMD_HANDLER_H */
