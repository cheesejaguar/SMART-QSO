/**
 * @file cmd_auth.c
 * @brief Telecommand authentication: HMAC-SHA-256 tag with replay counter
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include "cmd_auth.h"
#include "sha256.h"
#include "nvm_record.h"
#include "fault_mgmt.h"
#include "safe_string.h"
#include "hal/hal_flash.h"

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Persisted record identifier ("SQTA") */
#define AUTH_RECORD_MAGIC       0x53515441U

/** Domain-separation prefix of the MAC input */
static const uint8_t AUTH_DOMAIN[] = {
    'S', 'Q', 'S', 'O', '-', 'T', 'C', '-', 'v', '1'
};

/** Persisted layout: key (32) | key_id (1) | reserved (3) | last_counter (4) */
#define REC_OFFSET_KEY          0U
#define REC_OFFSET_KEY_ID       32U
#define REC_OFFSET_COUNTER      36U
#define REC_SIZE                40U

/** Highest counter that can be committed; UINT32_MAX is never accepted */
#define AUTH_COUNTER_LIMIT      0xFFFFFFFEU

/** Largest MAC input: domain + cmd_id + len + counter + args */
#define MAC_INPUT_MAX           (sizeof(AUTH_DOMAIN) + 2U + CMD_AUTH_COUNTER_SIZE + CMD_AUTH_MAX_ARGS)

/*******************************************************************************
 * Private Data
 ******************************************************************************/

static uint8_t s_key[CMD_AUTH_KEY_SIZE];
static uint8_t s_key_id = 0U;
static uint32_t s_last_counter = 0U;
static bool s_provisioned = false;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

static uint32_t get_u32(const uint8_t *src)
{
    return ((uint32_t)src[0] << 24) | ((uint32_t)src[1] << 16) |
           ((uint32_t)src[2] << 8) | (uint32_t)src[3];
}

static void put_u32(uint8_t *dst, uint32_t value)
{
    dst[0] = (uint8_t)(value >> 24);
    dst[1] = (uint8_t)(value >> 16);
    dst[2] = (uint8_t)(value >> 8);
    dst[3] = (uint8_t)value;
}

/** Constant-time comparison: always examines every byte */
static bool tags_equal(const uint8_t *a, const uint8_t *b, size_t len)
{
    uint8_t diff = 0U;

    for (size_t i = 0U; i < len; i++) {
        diff = (uint8_t)(diff | (uint8_t)(a[i] ^ b[i]));
    }

    return diff == 0U;
}

static SmartQsoResult_t persist_record(void)
{
    uint8_t rec[REC_SIZE];

    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    (void)safe_memcpy(&rec[REC_OFFSET_KEY], CMD_AUTH_KEY_SIZE, s_key, CMD_AUTH_KEY_SIZE);
    rec[REC_OFFSET_KEY_ID] = s_key_id;
    put_u32(&rec[REC_OFFSET_COUNTER], s_last_counter);

    SmartQsoResult_t result = nvm_record_save(FLASH_REGION_AUTH, AUTH_RECORD_MAGIC,
                                              rec, sizeof(rec));
    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));

    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t cmd_auth_init(void)
{
    uint8_t rec[REC_SIZE];
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;

    s_provisioned = false;
    s_last_counter = 0U;
    (void)safe_memset(s_key, sizeof(s_key), 0, sizeof(s_key));

    SmartQsoResult_t result = nvm_record_load(FLASH_REGION_AUTH, AUTH_RECORD_MAGIC,
                                              rec, sizeof(rec), &status);
    if ((result == SMART_QSO_OK) && (status == NVM_RECORD_VALID)) {
        (void)safe_memcpy(s_key, sizeof(s_key), &rec[REC_OFFSET_KEY], CMD_AUTH_KEY_SIZE);
        s_key_id = rec[REC_OFFSET_KEY_ID];
        s_last_counter = get_u32(&rec[REC_OFFSET_COUNTER]);
        s_provisioned = true;
    } else if ((result != SMART_QSO_OK) || (status == NVM_RECORD_CORRUPT)) {
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_ERROR,
                            "Telecommand key record unusable: commands locked", 0.0);
    } else {
        /* Blank: no key provisioned yet; all commands rejected */
    }

    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    return SMART_QSO_OK;
}

SmartQsoResult_t cmd_auth_provision_key(const uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t key_id)
{
    if (key == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(s_key, sizeof(s_key), key, CMD_AUTH_KEY_SIZE);
    s_key_id = key_id;
    s_last_counter = 0U;
    s_provisioned = true;

    return persist_record();
}

bool cmd_auth_is_provisioned(void)
{
    return s_provisioned;
}

uint32_t cmd_auth_last_counter(void)
{
    return s_last_counter;
}

CmdAuthStatus_t cmd_auth_verify(uint8_t cmd_id, const uint8_t *payload,
                                size_t payload_len, CmdAuthVerified_t *verified)
{
    uint8_t mac_input[MAC_INPUT_MAX];
    uint8_t mac[SHA256_DIGEST_SIZE];
    size_t pos = 0U;

    if (!s_provisioned) {
        return CMD_AUTH_ERR_NOT_PROVISIONED;
    }
    if ((payload == NULL) || (verified == NULL) ||
        (payload_len < CMD_AUTH_OVERHEAD) ||
        ((payload_len - CMD_AUTH_OVERHEAD) > CMD_AUTH_MAX_ARGS)) {
        return CMD_AUTH_ERR_FORMAT;
    }

    size_t args_len = payload_len - CMD_AUTH_OVERHEAD;
    const uint8_t *args = &payload[CMD_AUTH_COUNTER_SIZE];
    const uint8_t *tag = &payload[CMD_AUTH_COUNTER_SIZE + args_len];

    /* MAC input: domain | cmd_id | args_len | counter | args */
    for (size_t i = 0U; i < sizeof(AUTH_DOMAIN); i++) {
        mac_input[pos] = AUTH_DOMAIN[i];
        pos++;
    }
    mac_input[pos] = cmd_id;
    pos++;
    mac_input[pos] = (uint8_t)args_len;
    pos++;
    for (size_t i = 0U; i < (CMD_AUTH_COUNTER_SIZE + args_len); i++) {
        mac_input[pos] = payload[i];
        pos++;
    }

    if (hmac_sha256(s_key, sizeof(s_key), mac_input, pos, mac) != SMART_QSO_OK) {
        return CMD_AUTH_ERR_TAG;
    }
    bool tag_ok = tags_equal(mac, tag, CMD_AUTH_TAG_SIZE);
    (void)safe_memset(mac, sizeof(mac), 0, sizeof(mac));
    if (!tag_ok) {
        return CMD_AUTH_ERR_TAG;
    }

    /* Counter checks only after the frame is authentic */
    uint32_t counter = get_u32(payload);
    if ((s_last_counter >= AUTH_COUNTER_LIMIT) || (counter > AUTH_COUNTER_LIMIT)) {
        return CMD_AUTH_ERR_EXHAUSTED;
    }
    if (counter <= s_last_counter) {
        return CMD_AUTH_ERR_REPLAY;
    }

    verified->counter = counter;
    verified->args = args;
    verified->args_len = args_len;

    return CMD_AUTH_OK;
}

SmartQsoResult_t cmd_auth_commit(uint32_t counter)
{
    if (!s_provisioned) {
        return SMART_QSO_ERROR;
    }
    if ((counter > AUTH_COUNTER_LIMIT) || (counter <= s_last_counter)) {
        return SMART_QSO_ERROR_INVALID;
    }

    /* Advance in memory first so the frame cannot be replayed this session */
    s_last_counter = counter;

    return persist_record();
}
