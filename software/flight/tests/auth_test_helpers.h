/**
 * @file auth_test_helpers.h
 * @brief Test-only helpers that build authenticated telecommand payloads
 *
 * Mirrors software/ground/tools/telecommand_builder.py. Keys built here are
 * TEST-ONLY patterns (first, first+1, ...); flight keys are provisioned at
 * integration and never appear in the repository.
 */

#ifndef SMART_QSO_AUTH_TEST_HELPERS_H
#define SMART_QSO_AUTH_TEST_HELPERS_H

#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include "cmd_auth.h"
#include "sha256.h"

/** Fill a TEST-ONLY key with first, first+1, ... */
static inline void test_key_fill(uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t first)
{
    for (size_t i = 0U; i < CMD_AUTH_KEY_SIZE; i++) {
        key[i] = (uint8_t)(first + i);
    }
}

/** Build counter | args | tag; returns the payload length (0 on failure) */
static inline size_t test_auth_payload(const uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t cmd_id,
                                       uint32_t counter, const uint8_t *args, size_t args_len,
                                       uint8_t *out)
{
    static const uint8_t domain[] = {'S', 'Q', 'S', 'O', '-', 'T', 'C', '-', 'v', '1'};
    uint8_t mac_input[sizeof(domain) + 2U + CMD_AUTH_COUNTER_SIZE + CMD_AUTH_MAX_ARGS + 1U];
    uint8_t mac[SHA256_DIGEST_SIZE];
    size_t pos;

    if (args_len > (CMD_AUTH_MAX_ARGS + 1U)) {
        return 0U;
    }
    out[0] = (uint8_t)(counter >> 24);
    out[1] = (uint8_t)(counter >> 16);
    out[2] = (uint8_t)(counter >> 8);
    out[3] = (uint8_t)counter;
    if (args_len > 0U) {
        memcpy(&out[CMD_AUTH_COUNTER_SIZE], args, args_len);
    }

    memcpy(mac_input, domain, sizeof(domain));
    pos = sizeof(domain);
    mac_input[pos++] = cmd_id;
    mac_input[pos++] = (uint8_t)args_len;
    memcpy(&mac_input[pos], out, CMD_AUTH_COUNTER_SIZE + args_len);
    pos += CMD_AUTH_COUNTER_SIZE + args_len;

    if (hmac_sha256(key, CMD_AUTH_KEY_SIZE, mac_input, pos, mac) != SMART_QSO_OK) {
        return 0U;
    }
    memcpy(&out[CMD_AUTH_COUNTER_SIZE + args_len], mac, CMD_AUTH_TAG_SIZE);
    return CMD_AUTH_OVERHEAD + args_len;
}

#endif /* SMART_QSO_AUTH_TEST_HELPERS_H */
