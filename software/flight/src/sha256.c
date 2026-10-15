/**
 * @file sha256.c
 * @brief SHA-256 (FIPS 180-4) and HMAC-SHA-256 (RFC 2104) implementation
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Static memory only; every loop is bounded by a compile-time constant or by
 * the caller-supplied length.
 */

#include "sha256.h"
#include "safe_string.h"

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Round constants (first 32 bits of the fractional parts of the cube roots
 *  of the first 64 primes), FIPS 180-4 §4.2.2 */
static const uint32_t K[64] = {
    0x428a2f98U, 0x71374491U, 0xb5c0fbcfU, 0xe9b5dba5U, 0x3956c25bU, 0x59f111f1U,
    0x923f82a4U, 0xab1c5ed5U, 0xd807aa98U, 0x12835b01U, 0x243185beU, 0x550c7dc3U,
    0x72be5d74U, 0x80deb1feU, 0x9bdc06a7U, 0xc19bf174U, 0xe49b69c1U, 0xefbe4786U,
    0x0fc19dc6U, 0x240ca1ccU, 0x2de92c6fU, 0x4a7484aaU, 0x5cb0a9dcU, 0x76f988daU,
    0x983e5152U, 0xa831c66dU, 0xb00327c8U, 0xbf597fc7U, 0xc6e00bf3U, 0xd5a79147U,
    0x06ca6351U, 0x14292967U, 0x27b70a85U, 0x2e1b2138U, 0x4d2c6dfcU, 0x53380d13U,
    0x650a7354U, 0x766a0abbU, 0x81c2c92eU, 0x92722c85U, 0xa2bfe8a1U, 0xa81a664bU,
    0xc24b8b70U, 0xc76c51a3U, 0xd192e819U, 0xd6990624U, 0xf40e3585U, 0x106aa070U,
    0x19a4c116U, 0x1e376c08U, 0x2748774cU, 0x34b0bcb5U, 0x391c0cb3U, 0x4ed8aa4aU,
    0x5b9cca4fU, 0x682e6ff3U, 0x748f82eeU, 0x78a5636fU, 0x84c87814U, 0x8cc70208U,
    0x90befffaU, 0xa4506cebU, 0xbef9a3f7U, 0xc67178f2U
};

/** Initial hash value, FIPS 180-4 §5.3.3 */
static const uint32_t H0[8] = {
    0x6a09e667U, 0xbb67ae85U, 0x3c6ef372U, 0xa54ff53aU,
    0x510e527fU, 0x9b05688cU, 0x1f83d9abU, 0x5be0cd19U
};

/** HMAC pad bytes, RFC 2104 */
#define HMAC_IPAD   0x36U
#define HMAC_OPAD   0x5cU

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

static uint32_t rotr(uint32_t x, uint32_t n)
{
    return (x >> n) | (x << (32U - n));
}

/** Process one 64-byte block (FIPS 180-4 §6.2.2) */
static void sha256_compress(uint32_t state[8], const uint8_t block[SHA256_BLOCK_SIZE])
{
    uint32_t w[64];

    for (size_t t = 0U; t < 16U; t++) {
        w[t] = ((uint32_t)block[4U * t] << 24) |
               ((uint32_t)block[(4U * t) + 1U] << 16) |
               ((uint32_t)block[(4U * t) + 2U] << 8) |
               (uint32_t)block[(4U * t) + 3U];
    }
    for (size_t t = 16U; t < 64U; t++) {
        uint32_t s0 = rotr(w[t - 15U], 7U) ^ rotr(w[t - 15U], 18U) ^ (w[t - 15U] >> 3);
        uint32_t s1 = rotr(w[t - 2U], 17U) ^ rotr(w[t - 2U], 19U) ^ (w[t - 2U] >> 10);
        w[t] = w[t - 16U] + s0 + w[t - 7U] + s1;
    }

    uint32_t a = state[0];
    uint32_t b = state[1];
    uint32_t c = state[2];
    uint32_t d = state[3];
    uint32_t e = state[4];
    uint32_t f = state[5];
    uint32_t g = state[6];
    uint32_t h = state[7];

    for (size_t t = 0U; t < 64U; t++) {
        uint32_t big_s1 = rotr(e, 6U) ^ rotr(e, 11U) ^ rotr(e, 25U);
        uint32_t ch = (e & f) ^ ((~e) & g);
        uint32_t t1 = h + big_s1 + ch + K[t] + w[t];
        uint32_t big_s0 = rotr(a, 2U) ^ rotr(a, 13U) ^ rotr(a, 22U);
        uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
        uint32_t t2 = big_s0 + maj;

        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
    state[5] += f;
    state[6] += g;
    state[7] += h;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t sha256_init(Sha256Ctx_t *ctx)
{
    if (ctx == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    for (size_t i = 0U; i < 8U; i++) {
        ctx->state[i] = H0[i];
    }
    ctx->total_len = 0U;
    ctx->buffer_len = 0U;
    (void)safe_memset(ctx->buffer, sizeof(ctx->buffer), 0, sizeof(ctx->buffer));

    return SMART_QSO_OK;
}

SmartQsoResult_t sha256_update(Sha256Ctx_t *ctx, const uint8_t *data, size_t len)
{
    if (ctx == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if ((data == NULL) && (len > 0U)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    for (size_t i = 0U; i < len; i++) {
        ctx->buffer[ctx->buffer_len] = data[i];
        ctx->buffer_len++;
        if (ctx->buffer_len == SHA256_BLOCK_SIZE) {
            sha256_compress(ctx->state, ctx->buffer);
            ctx->buffer_len = 0U;
        }
    }
    ctx->total_len += (uint64_t)len;

    return SMART_QSO_OK;
}

SmartQsoResult_t sha256_final(Sha256Ctx_t *ctx, uint8_t digest[SHA256_DIGEST_SIZE])
{
    if ((ctx == NULL) || (digest == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    uint64_t bit_len = ctx->total_len * 8U;

    /* Append 0x80, then zeros until 56 bytes into the block */
    ctx->buffer[ctx->buffer_len] = 0x80U;
    ctx->buffer_len++;
    if (ctx->buffer_len > 56U) {
        while (ctx->buffer_len < SHA256_BLOCK_SIZE) {
            ctx->buffer[ctx->buffer_len] = 0U;
            ctx->buffer_len++;
        }
        sha256_compress(ctx->state, ctx->buffer);
        ctx->buffer_len = 0U;
    }
    while (ctx->buffer_len < 56U) {
        ctx->buffer[ctx->buffer_len] = 0U;
        ctx->buffer_len++;
    }

    /* Append message length in bits, big-endian */
    for (size_t i = 0U; i < 8U; i++) {
        ctx->buffer[56U + i] = (uint8_t)(bit_len >> (56U - (8U * i)));
    }
    sha256_compress(ctx->state, ctx->buffer);

    for (size_t i = 0U; i < 8U; i++) {
        digest[4U * i] = (uint8_t)(ctx->state[i] >> 24);
        digest[(4U * i) + 1U] = (uint8_t)(ctx->state[i] >> 16);
        digest[(4U * i) + 2U] = (uint8_t)(ctx->state[i] >> 8);
        digest[(4U * i) + 3U] = (uint8_t)ctx->state[i];
    }

    /* Do not leave message material in the context */
    (void)safe_memset(ctx, sizeof(*ctx), 0, sizeof(*ctx));

    return SMART_QSO_OK;
}

SmartQsoResult_t sha256(const uint8_t *data, size_t len,
                        uint8_t digest[SHA256_DIGEST_SIZE])
{
    Sha256Ctx_t ctx;

    if (digest == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    SmartQsoResult_t result = sha256_init(&ctx);
    if (result == SMART_QSO_OK) {
        result = sha256_update(&ctx, data, len);
    }
    if (result == SMART_QSO_OK) {
        result = sha256_final(&ctx, digest);
    }

    return result;
}

SmartQsoResult_t hmac_sha256(const uint8_t *key, size_t key_len,
                             const uint8_t *msg, size_t msg_len,
                             uint8_t mac[SHA256_DIGEST_SIZE])
{
    uint8_t key_block[SHA256_BLOCK_SIZE];
    uint8_t pad[SHA256_BLOCK_SIZE];
    uint8_t inner[SHA256_DIGEST_SIZE];
    Sha256Ctx_t ctx;

    if ((key == NULL) || (mac == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if ((msg == NULL) && (msg_len > 0U)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    /* K0: keys longer than a block are hashed, then zero-padded */
    (void)safe_memset(key_block, sizeof(key_block), 0, sizeof(key_block));
    if (key_len > SHA256_BLOCK_SIZE) {
        SmartQsoResult_t result = sha256(key, key_len, key_block);
        if (result != SMART_QSO_OK) {
            return result;
        }
    } else {
        for (size_t i = 0U; i < key_len; i++) {
            key_block[i] = key[i];
        }
    }

    /* inner = H((K0 ^ ipad) || msg) */
    for (size_t i = 0U; i < SHA256_BLOCK_SIZE; i++) {
        pad[i] = key_block[i] ^ HMAC_IPAD;
    }
    (void)sha256_init(&ctx);
    (void)sha256_update(&ctx, pad, sizeof(pad));
    (void)sha256_update(&ctx, msg, msg_len);
    (void)sha256_final(&ctx, inner);

    /* mac = H((K0 ^ opad) || inner) */
    for (size_t i = 0U; i < SHA256_BLOCK_SIZE; i++) {
        pad[i] = key_block[i] ^ HMAC_OPAD;
    }
    (void)sha256_init(&ctx);
    (void)sha256_update(&ctx, pad, sizeof(pad));
    (void)sha256_update(&ctx, inner, sizeof(inner));
    (void)sha256_final(&ctx, mac);

    /* Scrub key material from the stack */
    (void)safe_memset(key_block, sizeof(key_block), 0, sizeof(key_block));
    (void)safe_memset(pad, sizeof(pad), 0, sizeof(pad));

    return SMART_QSO_OK;
}
