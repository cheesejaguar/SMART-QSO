/**
 * @file cmd_auth.h
 * @brief Telecommand authentication: HMAC-SHA-256 tag with replay counter
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Every telecommand payload carries
 *
 *     counter (4, big-endian) | args (N) | tag (16)
 *
 * where tag = first 16 bytes of
 *
 *     HMAC-SHA-256(key, "SQSO-TC-v1" | cmd_id | N | counter | args)
 *
 * The command ID and argument length are inside the MAC, so a captured frame
 * cannot be relabelled as another command. The counter must be strictly
 * greater than the last accepted counter; it is persisted before the command
 * executes, so a replay is rejected even after a reset. Authentication (not
 * encryption) of telecommand is permitted by 47 CFR 97.211(b); all other
 * transmissions stay in the clear.
 *
 * The key is provisioned at integration over the umbilical; it is never
 * stored in the repository and there is no default key. Until a key is
 * provisioned every command is rejected (fail closed).
 *
 * Interface definition: docs/interfaces/TELECOMMAND_AUTHENTICATION.md
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#ifndef SMART_QSO_CMD_AUTH_H
#define SMART_QSO_CMD_AUTH_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** HMAC key length in bytes */
#define CMD_AUTH_KEY_SIZE       32U

/** Truncated tag length in bytes */
#define CMD_AUTH_TAG_SIZE       16U

/** Counter length in bytes */
#define CMD_AUTH_COUNTER_SIZE   4U

/** Bytes added to every command payload by authentication */
#define CMD_AUTH_OVERHEAD       (CMD_AUTH_COUNTER_SIZE + CMD_AUTH_TAG_SIZE)

/** Largest argument block that can be authenticated */
#define CMD_AUTH_MAX_ARGS       64U

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Authentication outcome
 */
typedef enum {
    CMD_AUTH_OK                 = 0, /**< Tag valid and counter fresh */
    CMD_AUTH_ERR_NOT_PROVISIONED = 1, /**< No key provisioned: fail closed */
    CMD_AUTH_ERR_FORMAT         = 2, /**< Payload too short or args too long */
    CMD_AUTH_ERR_TAG            = 3, /**< Tag does not match */
    CMD_AUTH_ERR_REPLAY         = 4, /**< Counter not greater than last accepted */
    CMD_AUTH_ERR_EXHAUSTED      = 5  /**< Counter space exhausted: rotate key */
} CmdAuthStatus_t;

/**
 * @brief A verified command's fields
 */
typedef struct {
    uint32_t counter;           /**< Command counter */
    const uint8_t *args;        /**< Arguments (points into the payload) */
    size_t args_len;            /**< Argument length */
} CmdAuthVerified_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Load the key and last accepted counter from non-volatile memory
 *
 * A blank or corrupt record leaves the module unprovisioned (fail closed).
 *
 * @return SMART_QSO_OK once the module is initialized
 */
SmartQsoResult_t cmd_auth_init(void);

/**
 * @brief Provision a key and reset the counter (integration only, not via RF)
 *
 * @param[in] key    CMD_AUTH_KEY_SIZE bytes
 * @param[in] key_id Key identifier reported in telemetry
 * @return SMART_QSO_OK if stored and persisted
 */
SmartQsoResult_t cmd_auth_provision_key(const uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t key_id);

/**
 * @brief Whether a key is provisioned
 */
bool cmd_auth_is_provisioned(void);

/**
 * @brief Last accepted (committed) counter
 */
uint32_t cmd_auth_last_counter(void);

/**
 * @brief Verify a command payload; does not change any state
 *
 * @param[in]  cmd_id      Command ID from the frame header
 * @param[in]  payload     Frame payload: counter | args | tag
 * @param[in]  payload_len Payload length
 * @param[out] verified    Counter and argument view (valid only on CMD_AUTH_OK)
 * @return Authentication outcome
 */
CmdAuthStatus_t cmd_auth_verify(uint8_t cmd_id, const uint8_t *payload,
                                size_t payload_len, CmdAuthVerified_t *verified);

/**
 * @brief Accept a verified counter and persist it; call before executing
 *
 * The in-memory counter advances even if persistence fails, so the frame
 * cannot be replayed until the next reset.
 *
 * @param[in] counter Counter from a CMD_AUTH_OK verification
 * @return SMART_QSO_OK if persisted; SMART_QSO_ERROR_IO otherwise
 */
SmartQsoResult_t cmd_auth_commit(uint32_t counter);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_CMD_AUTH_H */
