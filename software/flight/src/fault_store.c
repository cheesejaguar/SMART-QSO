/**
 * @file fault_store.c
 * @brief Fault log storage (simulation: a file) and console output
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SRS-F041 Maintain the fault log in non-volatile memory
 */

#include "fault_store.h"
#include "fault_mgmt.h"
#include "smart_qso.h"
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
#include <stdint.h>

/** Persistent fault log file: FAULT_LOG_FILE unless overridden */
static const char *fault_log_path(void)
{
    const char *path = getenv(FAULT_LOG_FILE_ENV);
    return ((path != NULL) && (path[0] != '\0')) ? path : FAULT_LOG_FILE;
}

SmartQsoResult_t fault_store_save(const uint8_t *data, size_t len)
{
    SmartQsoResult_t result = SMART_QSO_OK;
    FILE *f = fopen(fault_log_path(), "wb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }
    if ((len > 0U) && ((data == NULL) || (fwrite(data, 1U, len, f) != len))) {
        result = SMART_QSO_ERROR_IO;
    }
    if (fclose(f) != 0) {
        result = SMART_QSO_ERROR_IO;
    }
    return result;
}

SmartQsoResult_t fault_store_load(uint8_t *data, size_t cap, size_t *len)
{
    if ((data == NULL) || (len == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    FILE *f = fopen(fault_log_path(), "rb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }
    *len = fread(data, 1U, cap, f);
    if (fclose(f) != 0) {
        *len = 0U;
        return SMART_QSO_ERROR_IO;
    }
    return SMART_QSO_OK;
}

void fault_store_erase(void)
{
    (void)remove(fault_log_path());
}

void smart_qso_console(const char *line)
{
    if (line != NULL) {
        (void)printf("%s\n", line);
    }
}
