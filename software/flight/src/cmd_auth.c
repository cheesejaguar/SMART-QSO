/**
 * @file cmd_auth.c
 * @brief Telecommand authentication: HMAC-SHA-256 tag, replay counters, key table
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include "cmd_auth.h"
#include "cmd_dictionary.h"
#include "sha256.h"
#include "fault_mgmt.h"
#include "safe_string.h"
#include "hal/hal_flash.h"

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Record identifiers */
#define MAGIC_STATE             0x53515453U  /* "SQTS" */
#define MAGIC_RECOVERY          0x53515452U  /* "SQTR" */
#define MAGIC_OP_KEY            0x5351544BU  /* "SQTK" */

/** Domain-separation prefix of the MAC input */
static const uint8_t AUTH_DOMAIN[] = {
    'S', 'Q', 'S', 'O', '-', 'T', 'C', '-', 'v', '1'
};

/** State record: active slot (1) | reserved (3) | operational counter (4, BE) */
#define STATE_LEN               8U
#define STATE_OFFSET_COUNTER    4U

/** Recovery record: key (32) | key ID (1) | reserved (3) | counter (4, BE) */
#define RECOVERY_LEN            40U
#define RECOVERY_OFFSET_ID      32U
#define RECOVERY_OFFSET_COUNTER 36U

/** Operational key record: key (32) | key ID (1) | status (1) | reserved (2) */
#define OP_LEN                  36U
#define OP_OFFSET_ID            32U
#define OP_OFFSET_STATUS        33U

/** Operational key status; revoked keys are also zeroized */
#define KEY_SPARE               0x5AU
#define KEY_ACTIVE              0xA5U
#define KEY_REVOKED             0x00U

/** Highest counter that can be committed; UINT32_MAX is never accepted */
#define AUTH_COUNTER_LIMIT      0xFFFFFFFEU

/** Largest MAC input: domain + cmd_id + len + counter + args */
#define MAC_INPUT_MAX           (sizeof(AUTH_DOMAIN) + 2U + CMD_AUTH_COUNTER_SIZE + CMD_AUTH_MAX_ARGS)

/*******************************************************************************
 * Private Types and Data
 ******************************************************************************/

typedef struct {
    uint8_t key[CMD_AUTH_KEY_SIZE];
    uint8_t key_id;
    uint32_t counter;           /**< Last accepted counter */
    bool usable;
} KeyState_t;

static KeyState_t s_op;         /**< Active operational key */
static uint8_t s_op_slot = 0U;  /**< Slot of the active operational key */
static KeyState_t s_rec;        /**< Recovery key */

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

static void scrub(KeyState_t *ks)
{
    (void)safe_memset(ks, sizeof(*ks), 0, sizeof(*ks));
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

static bool tag_matches(const KeyState_t *ks, const uint8_t *mac_input, size_t len,
                        const uint8_t *tag)
{
    uint8_t mac[SHA256_DIGEST_SIZE];

    if (hmac_sha256(ks->key, CMD_AUTH_KEY_SIZE, mac_input, len, mac) != SMART_QSO_OK) {
        return false;
    }
    bool equal = tags_equal(mac, tag, CMD_AUTH_TAG_SIZE);
    (void)safe_memset(mac, sizeof(mac), 0, sizeof(mac));
    return equal;
}

static SmartQsoResult_t load_op(uint8_t slot, uint8_t rec[OP_LEN], NvmRecordStatus_t *status)
{
    return nvm_record_load_at(FLASH_REGION_AUTH, CMD_AUTH_NVM_OP(slot), MAGIC_OP_KEY,
                              rec, OP_LEN, status);
}

static SmartQsoResult_t save_op(uint8_t slot, const uint8_t key[CMD_AUTH_KEY_SIZE],
                                uint8_t key_id, uint8_t key_status)
{
    uint8_t rec[OP_LEN];

    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    (void)safe_memcpy(rec, sizeof(rec), key, CMD_AUTH_KEY_SIZE);
    rec[OP_OFFSET_ID] = key_id;
    rec[OP_OFFSET_STATUS] = key_status;

    SmartQsoResult_t result = nvm_record_save_at(FLASH_REGION_AUTH, CMD_AUTH_NVM_OP(slot),
                                                 MAGIC_OP_KEY, rec, sizeof(rec));
    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

static SmartQsoResult_t revoke_op(uint8_t slot)
{
    static const uint8_t s_zero_key[CMD_AUTH_KEY_SIZE] = {0U};
    return save_op(slot, s_zero_key, 0U, KEY_REVOKED);
}

static SmartQsoResult_t save_state(uint8_t slot, uint32_t counter)
{
    uint8_t rec[STATE_LEN];

    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    rec[0] = slot;
    put_u32(&rec[STATE_OFFSET_COUNTER], counter);

    SmartQsoResult_t result = nvm_record_save_at(FLASH_REGION_AUTH, CMD_AUTH_NVM_STATE,
                                                 MAGIC_STATE, rec, sizeof(rec));
    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

static SmartQsoResult_t save_recovery(void)
{
    uint8_t rec[RECOVERY_LEN];

    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    (void)safe_memcpy(rec, sizeof(rec), s_rec.key, CMD_AUTH_KEY_SIZE);
    rec[RECOVERY_OFFSET_ID] = s_rec.key_id;
    put_u32(&rec[RECOVERY_OFFSET_COUNTER], s_rec.counter);

    SmartQsoResult_t result = nvm_record_save_at(FLASH_REGION_AUTH, CMD_AUTH_NVM_RECOVERY,
                                                 MAGIC_RECOVERY, rec, sizeof(rec));
    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

/** Revoke every slot still marked active except keep_slot */
static void revoke_stale_active(uint8_t keep_slot)
{
    for (uint8_t i = 0U; i < CMD_AUTH_OP_SLOTS; i++) {
        uint8_t rec[OP_LEN];
        NvmRecordStatus_t status = NVM_RECORD_BLANK;

        if ((i != keep_slot) && (load_op(i, rec, &status) == SMART_QSO_OK) &&
            (status == NVM_RECORD_VALID) && (rec[OP_OFFSET_STATUS] == KEY_ACTIVE)) {
            if (revoke_op(i) != SMART_QSO_OK) {
                (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_ERROR,
                                    "Telecommand key revocation not persisted", 0.0);
            }
        }
        (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
    }
}

static const CmdDictEntry_t *dictionary_entry(uint8_t cmd_id)
{
    for (size_t i = 0U; i < g_cmd_dictionary_count; i++) {
        if (g_cmd_dictionary[i].cmd_id == cmd_id) {
            return &g_cmd_dictionary[i];
        }
    }
    return NULL;
}

static bool format_ok(const uint8_t *payload, size_t payload_len)
{
    return (payload != NULL) && (payload_len >= CMD_AUTH_OVERHEAD) &&
           ((payload_len - CMD_AUTH_OVERHEAD) <= CMD_AUTH_MAX_ARGS);
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t cmd_auth_init(void)
{
    uint8_t rec[RECOVERY_LEN];
    uint8_t state[STATE_LEN];
    uint8_t op[OP_LEN];
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;
    bool fault = false;

    scrub(&s_op);
    scrub(&s_rec);
    s_op_slot = 0U;

    /* Recovery key */
    SmartQsoResult_t result = nvm_record_load_at(FLASH_REGION_AUTH, CMD_AUTH_NVM_RECOVERY,
                                                 MAGIC_RECOVERY, rec, sizeof(rec), &status);
    if ((result == SMART_QSO_OK) && (status == NVM_RECORD_VALID)) {
        (void)safe_memcpy(s_rec.key, sizeof(s_rec.key), rec, CMD_AUTH_KEY_SIZE);
        s_rec.key_id = rec[RECOVERY_OFFSET_ID];
        s_rec.counter = get_u32(&rec[RECOVERY_OFFSET_COUNTER]);
        s_rec.usable = true;
    } else if ((result != SMART_QSO_OK) || (status == NVM_RECORD_CORRUPT)) {
        fault = true;
    } else {
        /* Blank: no recovery key provisioned */
    }
    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));

    /* Active operational key */
    result = nvm_record_load_at(FLASH_REGION_AUTH, CMD_AUTH_NVM_STATE, MAGIC_STATE,
                                state, sizeof(state), &status);
    if ((result == SMART_QSO_OK) && (status == NVM_RECORD_VALID)) {
        uint8_t slot = state[0];
        NvmRecordStatus_t op_status = NVM_RECORD_CORRUPT;

        if ((slot < CMD_AUTH_OP_SLOTS) && (load_op(slot, op, &op_status) == SMART_QSO_OK) &&
            (op_status == NVM_RECORD_VALID) && (op[OP_OFFSET_STATUS] == KEY_ACTIVE)) {
            (void)safe_memcpy(s_op.key, sizeof(s_op.key), op, CMD_AUTH_KEY_SIZE);
            s_op.key_id = op[OP_OFFSET_ID];
            s_op.counter = get_u32(&state[STATE_OFFSET_COUNTER]);
            s_op.usable = true;
            s_op_slot = slot;
            /* A rotation interrupted before its state write left a key marked
             * active that never took effect: revoke it */
            revoke_stale_active(slot);
        } else {
            fault = true;
        }
        (void)safe_memset(op, sizeof(op), 0, sizeof(op));
    } else if ((result != SMART_QSO_OK) || (status == NVM_RECORD_CORRUPT)) {
        fault = true;
    } else {
        /* Blank: no operational key activated */
    }

    if (fault) {
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_ERROR,
                            "Telecommand key record unusable", 0.0);
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t cmd_auth_provision_slot(uint8_t slot, const uint8_t key[CMD_AUTH_KEY_SIZE],
                                         uint8_t key_id)
{
    if (key == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if (slot > CMD_AUTH_RECOVERY_SLOT) {
        return SMART_QSO_ERROR_INVALID;
    }

    if (slot == CMD_AUTH_RECOVERY_SLOT) {
        (void)safe_memcpy(s_rec.key, sizeof(s_rec.key), key, CMD_AUTH_KEY_SIZE);
        s_rec.key_id = key_id;
        s_rec.counter = 0U;
        s_rec.usable = true;
        return save_recovery();
    }

    /* Overwriting the active slot deactivates it until it is rotated in again */
    if (s_op.usable && (slot == s_op_slot)) {
        scrub(&s_op);
    }
    return save_op(slot, key, key_id, KEY_SPARE);
}

SmartQsoResult_t cmd_auth_rotate(uint8_t slot)
{
    uint8_t rec[OP_LEN];
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;

    if (slot >= CMD_AUTH_OP_SLOTS) {
        return SMART_QSO_ERROR_INVALID;
    }
    if ((load_op(slot, rec, &status) != SMART_QSO_OK) || (status != NVM_RECORD_VALID) ||
        (rec[OP_OFFSET_STATUS] != KEY_SPARE)) {
        (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
        return SMART_QSO_ERROR_INVALID;
    }

    /* 1. Mark the spare active. 2. Point the state record at it (the commit
     * point). 3. Revoke the previous key. A power loss before step 2 leaves
     * the old key in force; init revokes the half-activated one. */
    SmartQsoResult_t result = save_op(slot, rec, rec[OP_OFFSET_ID], KEY_ACTIVE);
    if (result == SMART_QSO_OK) {
        result = save_state(slot, 0U);
    }
    if (result != SMART_QSO_OK) {
        (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));
        (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_ERROR,
                            "Telecommand key rotation not persisted", 0.0);
        return SMART_QSO_ERROR_IO;
    }

    revoke_stale_active(slot);

    scrub(&s_op);
    (void)safe_memcpy(s_op.key, sizeof(s_op.key), rec, CMD_AUTH_KEY_SIZE);
    s_op.key_id = rec[OP_OFFSET_ID];
    s_op.counter = 0U;
    s_op.usable = true;
    s_op_slot = slot;
    (void)safe_memset(rec, sizeof(rec), 0, sizeof(rec));

    (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_INFO,
                        "Telecommand key rotated", (double)slot);
    return SMART_QSO_OK;
}

SmartQsoResult_t cmd_auth_zeroize(void)
{
    scrub(&s_op);
    scrub(&s_rec);
    s_op_slot = 0U;
    return hal_flash_erase(FLASH_REGION_AUTH);
}

SmartQsoResult_t cmd_auth_provision_key(const uint8_t key[CMD_AUTH_KEY_SIZE], uint8_t key_id)
{
    SmartQsoResult_t result = cmd_auth_provision_slot(0U, key, key_id);
    return (result == SMART_QSO_OK) ? cmd_auth_rotate(0U) : result;
}

bool cmd_auth_is_provisioned(void)
{
    return s_op.usable;
}

bool cmd_auth_has_key_material(void)
{
    return s_op.usable || s_rec.usable;
}

uint32_t cmd_auth_last_counter(void)
{
    return s_op.counter;
}

void cmd_auth_get_info(CmdAuthInfo_t *info)
{
    if (info == NULL) {
        return;
    }
    info->operational_usable = s_op.usable;
    info->recovery_usable = s_rec.usable;
    info->active_slot = s_op_slot;
    info->active_key_id = s_op.key_id;
    info->recovery_key_id = s_rec.key_id;
    info->operational_counter = s_op.counter;
    info->recovery_counter = s_rec.counter;
}

static void fill_verified(CmdAuthVerified_t *verified, uint32_t counter,
                          const uint8_t *payload, size_t args_len, CmdAuthKey_t key)
{
    verified->counter = counter;
    verified->args = &payload[CMD_AUTH_COUNTER_SIZE];
    verified->args_len = args_len;
    verified->key = key;
}

/** MAC input: domain | cmd_id | args_len | counter | args; returns its length */
static size_t build_mac_input(uint8_t cmd_id, const uint8_t *payload, size_t args_len,
                              uint8_t *mac_input)
{
    size_t pos = 0U;
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
    return pos;
}

/** No key at all: only commands marked without_key (TX-OFF) proceed */
static CmdAuthStatus_t verify_without_key(const CmdDictEntry_t *entry, const uint8_t *payload,
                                          size_t payload_len, CmdAuthVerified_t *verified)
{
    if ((entry == NULL) || !entry->without_key) {
        return CMD_AUTH_ERR_NOT_PROVISIONED;
    }
    if (!format_ok(payload, payload_len)) {
        return CMD_AUTH_ERR_FORMAT;
    }
    fill_verified(verified, get_u32(payload), payload, payload_len - CMD_AUTH_OVERHEAD,
                  CMD_AUTH_KEY_NONE);
    return CMD_AUTH_OK;
}

CmdAuthStatus_t cmd_auth_verify(uint8_t cmd_id, const uint8_t *payload,
                                size_t payload_len, CmdAuthVerified_t *verified)
{
    uint8_t mac_input[MAC_INPUT_MAX];

    if (verified == NULL) {
        return CMD_AUTH_ERR_FORMAT;
    }

    const CmdDictEntry_t *entry = dictionary_entry(cmd_id);
    CmdKeyClass_t key_class = (entry != NULL) ? entry->key : CMD_KEY_OPERATIONAL;

    if (!s_op.usable && !s_rec.usable) {
        return verify_without_key(entry, payload, payload_len, verified);
    }

    bool try_op = s_op.usable && (key_class != CMD_KEY_RECOVERY);
    bool try_rec = s_rec.usable && (key_class != CMD_KEY_OPERATIONAL);
    if (!try_op && !try_rec) {
        return CMD_AUTH_ERR_NOT_PROVISIONED;
    }
    if (!format_ok(payload, payload_len)) {
        return CMD_AUTH_ERR_FORMAT;
    }

    size_t args_len = payload_len - CMD_AUTH_OVERHEAD;
    const uint8_t *tag = &payload[CMD_AUTH_COUNTER_SIZE + args_len];
    size_t mac_len = build_mac_input(cmd_id, payload, args_len, mac_input);

    const KeyState_t *matched = NULL;
    CmdAuthKey_t which = CMD_AUTH_KEY_NONE;
    if (try_op && tag_matches(&s_op, mac_input, mac_len, tag)) {
        matched = &s_op;
        which = CMD_AUTH_KEY_OPERATIONAL;
    } else if (try_rec && tag_matches(&s_rec, mac_input, mac_len, tag)) {
        matched = &s_rec;
        which = CMD_AUTH_KEY_RECOVERY;
    } else {
        return CMD_AUTH_ERR_TAG;
    }

    /* Counter checks only after the frame is authentic */
    uint32_t counter = get_u32(payload);
    if ((matched->counter >= AUTH_COUNTER_LIMIT) || (counter > AUTH_COUNTER_LIMIT)) {
        return CMD_AUTH_ERR_EXHAUSTED;
    }
    if (counter <= matched->counter) {
        return CMD_AUTH_ERR_REPLAY;
    }

    fill_verified(verified, counter, payload, args_len, which);
    return CMD_AUTH_OK;
}

SmartQsoResult_t cmd_auth_commit(const CmdAuthVerified_t *verified)
{
    if (verified == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if (verified->key == CMD_AUTH_KEY_NONE) {
        return SMART_QSO_OK;  /* No key, no counter */
    }

    KeyState_t *ks = (verified->key == CMD_AUTH_KEY_RECOVERY) ? &s_rec : &s_op;
    if (!ks->usable) {
        return SMART_QSO_ERROR;
    }
    if ((verified->counter > AUTH_COUNTER_LIMIT) || (verified->counter <= ks->counter)) {
        return SMART_QSO_ERROR_INVALID;
    }

    /* Advance in memory first so the frame cannot be replayed this session */
    ks->counter = verified->counter;

    return (ks == &s_rec) ? save_recovery() : save_state(s_op_slot, s_op.counter);
}
