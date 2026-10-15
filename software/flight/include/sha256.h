/**
 * @file sha256.h
 * @brief SHA-256 (FIPS 180-4) and HMAC-SHA-256 (RFC 2104) for telecommand
 *        authentication
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Static memory only, bounded loops, no dynamic allocation.
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#ifndef SMART_QSO_SHA256_H
#define SMART_QSO_SHA256_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stddef.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** SHA-256 digest size in bytes */
#define SHA256_DIGEST_SIZE      32U

/** SHA-256 block size in bytes */
#define SHA256_BLOCK_SIZE       64U

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Incremental SHA-256 context
 */
typedef struct {
    uint32_t state[8];                  /**< Chaining state H0..H7 */
    uint64_t total_len;                 /**< Message length processed (bytes) */
    uint8_t  buffer[SHA256_BLOCK_SIZE]; /**< Partial block */
    size_t   buffer_len;                /**< Bytes held in buffer */
} Sha256Ctx_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize an incremental SHA-256 computation
 *
 * @param[out] ctx Context to initialize
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sha256_init(Sha256Ctx_t *ctx);

/**
 * @brief Absorb message bytes
 *
 * @param[in,out] ctx  Context
 * @param[in]     data Message bytes (may be NULL only when len is 0)
 * @param[in]     len  Number of bytes
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sha256_update(Sha256Ctx_t *ctx, const uint8_t *data, size_t len);

/**
 * @brief Finish the computation and write the digest
 *
 * @param[in,out] ctx    Context (unusable afterwards until re-initialized)
 * @param[out]    digest SHA256_DIGEST_SIZE bytes
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sha256_final(Sha256Ctx_t *ctx, uint8_t digest[SHA256_DIGEST_SIZE]);

/**
 * @brief One-shot SHA-256
 *
 * @param[in]  data   Message bytes (may be NULL only when len is 0)
 * @param[in]  len    Number of bytes
 * @param[out] digest SHA256_DIGEST_SIZE bytes
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t sha256(const uint8_t *data, size_t len,
                        uint8_t digest[SHA256_DIGEST_SIZE]);

/**
 * @brief HMAC-SHA-256 (RFC 2104); keys longer than one block are hashed first
 *
 * @param[in]  key     Key bytes
 * @param[in]  key_len Key length
 * @param[in]  msg     Message bytes (may be NULL only when msg_len is 0)
 * @param[in]  msg_len Message length
 * @param[out] mac     SHA256_DIGEST_SIZE bytes
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t hmac_sha256(const uint8_t *key, size_t key_len,
                             const uint8_t *msg, size_t msg_len,
                             uint8_t mac[SHA256_DIGEST_SIZE]);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_SHA256_H */
