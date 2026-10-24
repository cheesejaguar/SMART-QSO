/**
 * @file fault_store.h
 * @brief Storage and console for the fault log (the log itself is Rust:
 *        software/rust/sqso-ffi/src/fault_mgmt.rs)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The fault log is kept as a raw array of FaultLogEntry_t, each with its
 * own CRC-32, so storage is a byte blob. In simulation it is the file
 * FAULT_LOG_FILE (or $SMART_QSO_FAULT_LOG); FLASH_REGION_FAULT_LOG on the
 * target is open (SW-01).
 *
 * @requirement SRS-F041 Maintain the fault log in non-volatile memory
 */

#ifndef SMART_QSO_FAULT_STORE_H
#define SMART_QSO_FAULT_STORE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stddef.h>
#include <stdint.h>

/**
 * @brief Replace the stored log with len bytes
 * @return SMART_QSO_OK, or SMART_QSO_ERROR_IO
 */
SmartQsoResult_t fault_store_save(const uint8_t *data, size_t len);

/**
 * @brief Read the stored log (at most cap bytes)
 * @param[out] len Bytes read
 * @return SMART_QSO_OK, or SMART_QSO_ERROR_IO (nothing stored; data untouched)
 */
SmartQsoResult_t fault_store_load(uint8_t *data, size_t cap, size_t *len);

/** @brief Remove the stored log */
void fault_store_erase(void);

/** @brief Write one line to the console (simulation: stdout) */
void smart_qso_console(const char *line);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_FAULT_STORE_H */
