/**
 * @file nvm_record.c
 * @brief Power-fail-safe persistent record (two slots, sequence number, CRC32)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#include "nvm_record.h"
#include "safe_string.h"

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

#define SLOT_COUNT          2U
#define OFFSET_MAGIC        0U
#define OFFSET_SEQUENCE     4U
#define OFFSET_PAYLOAD      8U

/*******************************************************************************
 * Private Types
 ******************************************************************************/

typedef struct {
    bool valid;           /**< Magic and CRC both check */
    bool magic_present;   /**< Magic field matches (even if CRC fails) */
    uint32_t sequence;    /**< Sequence number (meaningful when valid) */
} SlotInfo_t;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

static void put_u32(uint8_t *dst, uint32_t value)
{
    dst[0] = (uint8_t)(value >> 24);
    dst[1] = (uint8_t)(value >> 16);
    dst[2] = (uint8_t)(value >> 8);
    dst[3] = (uint8_t)value;
}

static uint32_t get_u32(const uint8_t *src)
{
    return ((uint32_t)src[0] << 24) | ((uint32_t)src[1] << 16) |
           ((uint32_t)src[2] << 8) | (uint32_t)src[3];
}

/** True if sequence a is newer than b (serial-number arithmetic) */
static bool sequence_newer(uint32_t a, uint32_t b)
{
    return (int32_t)(a - b) > 0;
}

/** Read one slot into buf and classify it */
static SmartQsoResult_t read_slot(HalFlashRegion_t region, uint32_t base, uint32_t slot,
                                  uint32_t magic, size_t len,
                                  uint8_t buf[NVM_RECORD_SLOT_SIZE], SlotInfo_t *info)
{
    size_t record_len = OFFSET_PAYLOAD + len + 4U;

    info->valid = false;
    info->magic_present = false;
    info->sequence = 0U;

    SmartQsoResult_t result = hal_flash_read(region, base + (slot * NVM_RECORD_SLOT_SIZE),
                                             buf, record_len);
    if (result != SMART_QSO_OK) {
        return result;
    }

    info->magic_present = (get_u32(&buf[OFFSET_MAGIC]) == magic);
    if (info->magic_present) {
        uint32_t stored_crc = get_u32(&buf[OFFSET_PAYLOAD + len]);
        if (smart_qso_crc32(buf, OFFSET_PAYLOAD + len) == stored_crc) {
            info->valid = true;
            info->sequence = get_u32(&buf[OFFSET_SEQUENCE]);
        }
    }

    return SMART_QSO_OK;
}

/** Index of the newest valid slot, or SLOT_COUNT if none */
static uint32_t newest_valid_slot(const SlotInfo_t info[SLOT_COUNT])
{
    uint32_t newest = SLOT_COUNT;

    for (uint32_t i = 0U; i < SLOT_COUNT; i++) {
        if (info[i].valid &&
            ((newest == SLOT_COUNT) || sequence_newer(info[i].sequence, info[newest].sequence))) {
            newest = i;
        }
    }

    return newest;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t nvm_record_load(HalFlashRegion_t region, uint32_t magic,
                                 void *payload, size_t len,
                                 NvmRecordStatus_t *status)
{
    return nvm_record_load_at(region, 0U, magic, payload, len, status);
}

SmartQsoResult_t nvm_record_save(HalFlashRegion_t region, uint32_t magic,
                                 const void *payload, size_t len)
{
    return nvm_record_save_at(region, 0U, magic, payload, len);
}

SmartQsoResult_t nvm_record_load_at(HalFlashRegion_t region, uint32_t base, uint32_t magic,
                                    void *payload, size_t len,
                                    NvmRecordStatus_t *status)
{
    uint8_t buf[SLOT_COUNT][NVM_RECORD_SLOT_SIZE];
    SlotInfo_t info[SLOT_COUNT];

    if ((payload == NULL) || (status == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if ((len == 0U) || (len > NVM_RECORD_MAX_PAYLOAD)) {
        return SMART_QSO_ERROR_INVALID;
    }

    for (uint32_t i = 0U; i < SLOT_COUNT; i++) {
        SmartQsoResult_t result = read_slot(region, base, i, magic, len, buf[i], &info[i]);
        if (result != SMART_QSO_OK) {
            return result;
        }
    }

    uint32_t newest = newest_valid_slot(info);
    if (newest < SLOT_COUNT) {
        (void)safe_memcpy(payload, len, &buf[newest][OFFSET_PAYLOAD], len);
        *status = NVM_RECORD_VALID;
    } else if (info[0].magic_present || info[1].magic_present) {
        *status = NVM_RECORD_CORRUPT;
    } else {
        *status = NVM_RECORD_BLANK;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t nvm_record_save_at(HalFlashRegion_t region, uint32_t base, uint32_t magic,
                                    const void *payload, size_t len)
{
    uint8_t buf[SLOT_COUNT][NVM_RECORD_SLOT_SIZE];
    SlotInfo_t info[SLOT_COUNT];
    uint8_t record[NVM_RECORD_SLOT_SIZE];
    uint8_t readback[NVM_RECORD_SLOT_SIZE];

    if (payload == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if ((len == 0U) || (len > NVM_RECORD_MAX_PAYLOAD)) {
        return SMART_QSO_ERROR_INVALID;
    }

    for (uint32_t i = 0U; i < SLOT_COUNT; i++) {
        SmartQsoResult_t result = read_slot(region, base, i, magic, len, buf[i], &info[i]);
        if (result != SMART_QSO_OK) {
            return result;
        }
    }

    /* Write the slot that does not hold the newest valid copy */
    uint32_t newest = newest_valid_slot(info);
    uint32_t target = (newest == 0U) ? 1U : 0U;
    uint32_t sequence = (newest < SLOT_COUNT) ? (info[newest].sequence + 1U) : 1U;

    size_t record_len = OFFSET_PAYLOAD + len + 4U;
    put_u32(&record[OFFSET_MAGIC], magic);
    put_u32(&record[OFFSET_SEQUENCE], sequence);
    (void)safe_memcpy(&record[OFFSET_PAYLOAD], NVM_RECORD_MAX_PAYLOAD, payload, len);
    put_u32(&record[OFFSET_PAYLOAD + len], smart_qso_crc32(record, OFFSET_PAYLOAD + len));

    uint32_t offset = base + (target * NVM_RECORD_SLOT_SIZE);
    SmartQsoResult_t result = hal_flash_write(region, offset, record, record_len);
    if (result != SMART_QSO_OK) {
        return result;
    }

    /* Verify the write landed */
    result = hal_flash_read(region, offset, readback, record_len);
    if (result != SMART_QSO_OK) {
        return result;
    }
    for (size_t i = 0U; i < record_len; i++) {
        if (readback[i] != record[i]) {
            return SMART_QSO_ERROR_IO;
        }
    }

    return SMART_QSO_OK;
}
