/**
 * @file cmd_auth.h
 * @brief Telecommand authentication: HMAC-SHA-256 tag, replay counters, key table
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
 * cannot be relabelled as another command. Each key has its own counter,
 * which must be strictly greater than the last accepted one; it is persisted
 * before the command executes, so a replay is rejected even after a reset.
 * Authentication (not encryption) of telecommand is permitted by 47 CFR
 * 97.211(b); all other transmissions stay in the clear.
 *
 * Key table (provisioned at integration over the umbilical, never in the
 * repository, no default key):
 *  - CMD_AUTH_OP_SLOTS operational keys. One is active; the others are spares.
 *  - One recovery key, held offline on the ground. It authenticates only
 *    KEY_ROTATE and TX_DISABLE (the command dictionary's key class).
 *
 * KEY_ROTATE activates a spare operational key and revokes (zeroizes) the
 * current one. No key material ever crosses the RF link.
 *
 * If no key is usable at all, every command is rejected except those the
 * dictionary marks without_key (TX_DISABLE): TX-OFF can only reduce
 * emissions, so the 47 CFR 97.207(b) obligation is always met.
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
#include "nvm_record.h"
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

/** Number of operational key slots (one active, the rest spares) */
#define CMD_AUTH_OP_SLOTS       4U

/** Slot number that addresses the recovery key in provisioning */
#define CMD_AUTH_RECOVERY_SLOT  CMD_AUTH_OP_SLOTS

/** Layout of FLASH_REGION_AUTH: one nvm_record per item */
#define CMD_AUTH_NVM_STATE      0U
#define CMD_AUTH_NVM_RECOVERY   NVM_RECORD_FOOTPRINT
#define CMD_AUTH_NVM_OP(slot)   ((2U + (uint32_t)(slot)) * NVM_RECORD_FOOTPRINT)
#define CMD_AUTH_NVM_SIZE       CMD_AUTH_NVM_OP(CMD_AUTH_OP_SLOTS)

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Authentication outcome
 */
typedef enum {
    CMD_AUTH_OK                 = 0, /**< Tag valid and counter fresh */
    CMD_AUTH_ERR_NOT_PROVISIONED = 1, /**< No key for this command: fail closed */
    CMD_AUTH_ERR_FORMAT         = 2, /**< Payload too short or args too long */
    CMD_AUTH_ERR_TAG            = 3, /**< Tag does not match */
    CMD_AUTH_ERR_REPLAY         = 4, /**< Counter not greater than last accepted */
    CMD_AUTH_ERR_EXHAUSTED      = 5  /**< Counter space exhausted: rotate key */
} CmdAuthStatus_t;

/**
 * @brief Which key authenticated a command
 */
typedef enum {
    CMD_AUTH_KEY_NONE        = 0, /**< Accepted without a key (no key usable) */
    CMD_AUTH_KEY_OPERATIONAL = 1, /**< Active operational key */
    CMD_AUTH_KEY_RECOVERY    = 2  /**< Recovery key */
} CmdAuthKey_t;

/**
 * @brief A verified command's fields
 */
typedef struct {
    uint32_t counter;           /**< Command counter */
    const uint8_t *args;        /**< Arguments (points into the payload) */
    size_t args_len;            /**< Argument length */
    CmdAuthKey_t key;           /**< Key that authenticated it */
} CmdAuthVerified_t;

/**
 * @brief Key-table status for telemetry (no key material)
 */
typedef struct {
    bool operational_usable;    /**< Active operational key present */
    bool recovery_usable;       /**< Recovery key present */
    uint8_t active_slot;        /**< Active operational slot (valid if usable) */
    uint8_t active_key_id;      /**< Active operational key ID */
    uint8_t recovery_key_id;    /**< Recovery key ID */
    uint32_t operational_counter; /**< Last accepted operational counter */
    uint32_t recovery_counter;  /**< Last accepted recovery counter */
} CmdAuthInfo_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Load the key table and counters from non-volatile memory
 *
 * Revokes any operational key left marked active by an interrupted rotation.
 * A blank or corrupt record leaves that key unusable (fail closed).
 *
 * @return SMART_QSO_OK once the module is initialized
 */
SmartQsoResult_t cmd_auth_init(void);

/**
 * @brief Write a key into a slot (integration only, through gse_provision)
 *
 * Operational slots (0 .. CMD_AUTH_OP_SLOTS-1) are stored as spares; the
 * recovery slot (CMD_AUTH_RECOVERY_SLOT) resets the recovery counter.
 *
 * @return SMART_QSO_OK if stored and persisted
 */
SmartQsoResult_t cmd_auth_provision_slot(uint8_t slot, const uint8_t key[CMD_AUTH_KEY_SIZE],
                                         uint8_t key_id);

/**
 * @brief Make a spare operational key active and revoke every other active key
 *
 * Used by KEY_ROTATE (recovery key) and by initial activation at
 * integration. The new key starts with counter 0.
 *
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID if the slot is not a spare;
 *         SMART_QSO_ERROR_IO if the change could not be persisted
 */
SmartQsoResult_t cmd_auth_rotate(uint8_t slot);

/**
 * @brief Erase the whole key table (integration only)
 */
SmartQsoResult_t cmd_auth_zeroize(void);

/**
 * @brief Provision operational slot 0 and activate it (integration and tests)
 */
SmartQsoResult_t cmd_auth_provision_key(const uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t key_id);

/**
 * @brief Whether an operational key is active
 */
bool cmd_auth_is_provisioned(void);

/**
 * @brief Whether any key (operational or recovery) is usable
 */
bool cmd_auth_has_key_material(void);

/**
 * @brief Last accepted operational counter
 */
uint32_t cmd_auth_last_counter(void);

/**
 * @brief Key-table status for telemetry
 */
void cmd_auth_get_info(CmdAuthInfo_t *info);

/**
 * @brief Verify a command payload; does not change any state
 *
 * The key is chosen by the command dictionary's key class; IDs outside the
 * dictionary use the operational key.
 *
 * @param[in]  cmd_id      Command ID from the frame header
 * @param[in]  payload     Frame payload: counter | args | tag
 * @param[in]  payload_len Payload length
 * @param[out] verified    Counter, argument view and key (valid only on CMD_AUTH_OK)
 * @return Authentication outcome
 */
CmdAuthStatus_t cmd_auth_verify(uint8_t cmd_id, const uint8_t *payload,
                                size_t payload_len, CmdAuthVerified_t *verified);

/**
 * @brief Accept a verified counter for its key and persist it; call before executing
 *
 * The in-memory counter advances even if persistence fails, so the frame
 * cannot be replayed until the next reset. A command accepted without a key
 * has no counter and always succeeds.
 *
 * @param[in] verified Result of a CMD_AUTH_OK verification
 * @return SMART_QSO_OK if persisted; SMART_QSO_ERROR_IO if not;
 *         SMART_QSO_ERROR_INVALID if the counter is not fresh
 */
SmartQsoResult_t cmd_auth_commit(const CmdAuthVerified_t *verified);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_CMD_AUTH_H */
