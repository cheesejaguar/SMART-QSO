/**
 * @file hk_store.h
 * @brief Stored housekeeping in FRAM and its dump (BASELINE §8, DB-01)
 *
 * Fixed 44-byte records (store id, version, sequence number, housekeeping
 * values, CRC-32; layout in docs/interfaces/ICD_SPACE_GROUND.md §5.6):
 *
 * - five-minute records, HK_STORE_SUMMARY, a ring of 2,048 (7.1 days) in
 *   FLASH_REGION_HK_SUMMARY;
 * - sixty-second records, HK_STORE_DETAIL: a ring of 409 (6.8 h), and three
 *   windows of 360 (6 h) holding the 3 h before and after each event (a mode
 *   change or a fault at ERROR or worse), in FLASH_REGION_HK_DETAIL; the
 *   window table in FLASH_REGION_HK_INDEX.
 *
 * Sequence numbers continue across resets (a boot reads every record).
 * DUMP_TLM (0x0A) starts a dump; frames of up to four records go at most one
 * every 12 s through the downlink and its duty limit.
 *
 * @requirement SRS-F120 Five-minute housekeeping records for at least 7 days in FRAM
 * @requirement SRS-F121 Sixty-second records for the 6 h around each fault and mode change
 * @requirement SRS-F122 DUMP_TLM downlinks a range of stored records within the downlink duty limit
 */

#ifndef SMART_QSO_HK_STORE_H
#define SMART_QSO_HK_STORE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

#define HK_STORE_SUMMARY     0U   /**< Five-minute records */
#define HK_STORE_DETAIL      1U   /**< Sixty-second records */
#define HK_STORE_RECORD_LEN  44U  /**< Bytes a record */
#define HK_STORE_MAX_DUMP    600U /**< Largest DUMP_TLM count */
#define HK_STORE_EVENT_MODE  1U   /**< Event: a mode change */
#define HK_STORE_EVENT_FAULT 2U   /**< Event: a fault at ERROR or worse */

/** Find the newest records and load the window table (boot) */
SmartQsoResult_t hk_store_init(void);

/** Write the records due at now_ms (every pass) */
void hk_store_step(uint64_t now_ms);

/** Note an event: the next sixty-second record carries it */
void hk_store_note_event(uint8_t kind);

/** The next sequence number of each store */
SmartQsoResult_t hk_store_counts(uint32_t *summary, uint32_t *detail);

/**
 * @brief Read stored record seq of store into out (at least 44 bytes)
 * @return SMART_QSO_OK; SMART_QSO_ERROR if not stored;
 *         SMART_QSO_ERROR_INVALID for an unknown store or a short buffer
 */
SmartQsoResult_t hk_store_read(uint8_t store, uint32_t seq, uint8_t *out, size_t len);

/**
 * @brief DUMP_TLM: dump count records of store from seq first (0 stops)
 * @return SMART_QSO_OK; SMART_QSO_ERROR_INVALID for an unknown store or a
 *         count above HK_STORE_MAX_DUMP
 */
SmartQsoResult_t hk_store_dump(uint8_t store, uint32_t first, uint16_t count);

/** Stop the dump in progress */
void hk_store_dump_stop(void);

/** Whether a dump is in progress */
bool hk_store_dump_active(void);

/**
 * @brief The next dump frame's information field ("HKD", store, count,
 *        records), if one is due: its length (0: none)
 *
 * Nothing is consumed until hk_store_dump_commit().
 */
size_t hk_store_dump_peek(uint64_t now_ms, uint8_t *out, size_t max);

/** The frame last peeked went out */
void hk_store_dump_commit(uint64_t now_ms);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_HK_STORE_H */
