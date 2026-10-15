/**
 * @file nvm_record.h
 * @brief Power-fail-safe persistent record (two slots, sequence number, CRC32)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * A record lives in one flash region as two slots (A at offset 0, B at
 * offset NVM_RECORD_SLOT_SIZE). Each save writes the slot that does NOT hold
 * the newest valid copy, so an interrupted write always leaves the previous
 * copy intact. Load returns the valid copy with the newest sequence number.
 *
 * Slot layout: magic (4) | sequence (4) | payload (len) | CRC32 (4),
 * CRC32 computed over magic, sequence and payload.
 */

#ifndef SMART_QSO_NVM_RECORD_H
#define SMART_QSO_NVM_RECORD_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include "hal/hal_flash.h"
#include <stdint.h>
#include <stddef.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** Size reserved for one slot in the region */
#define NVM_RECORD_SLOT_SIZE        64U

/** Slot overhead: magic + sequence + CRC32 */
#define NVM_RECORD_OVERHEAD         12U

/** Largest payload a slot can hold */
#define NVM_RECORD_MAX_PAYLOAD      (NVM_RECORD_SLOT_SIZE - NVM_RECORD_OVERHEAD)

/*******************************************************************************
 * Types
 ******************************************************************************/

/**
 * @brief Outcome of loading a record
 */
typedef enum {
    NVM_RECORD_VALID   = 0,  /**< A valid copy was found and returned */
    NVM_RECORD_BLANK   = 1,  /**< Neither slot carries the magic (never written) */
    NVM_RECORD_CORRUPT = 2   /**< Magic present but no slot passes its CRC */
} NvmRecordStatus_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Load the newest valid copy of a record
 *
 * @param[in]  region  Flash region holding the record
 * @param[in]  magic   Record type identifier
 * @param[out] payload Destination for the payload (written only when VALID)
 * @param[in]  len     Payload length (≤ NVM_RECORD_MAX_PAYLOAD)
 * @param[out] status  VALID, BLANK or CORRUPT
 * @return SMART_QSO_OK if the flash could be read; an error otherwise
 */
SmartQsoResult_t nvm_record_load(HalFlashRegion_t region, uint32_t magic,
                                 void *payload, size_t len,
                                 NvmRecordStatus_t *status);

/**
 * @brief Save a record into the slot not holding the newest valid copy
 *
 * @param[in] region  Flash region holding the record
 * @param[in] magic   Record type identifier
 * @param[in] payload Payload bytes
 * @param[in] len     Payload length (≤ NVM_RECORD_MAX_PAYLOAD)
 * @return SMART_QSO_OK on success; an error if the write or read-back fails
 */
SmartQsoResult_t nvm_record_save(HalFlashRegion_t region, uint32_t magic,
                                 const void *payload, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_NVM_RECORD_H */
