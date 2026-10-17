/**
 * @file fault_mgmt.c
 * @brief Fault Detection, Isolation, and Recovery (FDIR) Implementation
 *
 * @requirement SRS-F040 Detect and log all fault conditions
 * @requirement SRS-F042 Implement autonomous fault recovery
 * @requirement SRS-F041 Maintain the fault log in non-volatile memory
 */

#include "fault_mgmt.h"
#include "eps_control.h"
#include "state_machine.h"
#include "system_state.h"
#include "smart_qso.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <stdint.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Fault log buffer (module-private) */
static FaultLogEntry_t s_fault_log[SMART_QSO_MAX_FAULT_ENTRIES];

/** Current number of entries in fault log */
static size_t s_fault_log_count = 0;

/** Flag indicating watchdog was triggered */
static bool s_watchdog_triggered = false;

/** Flag indicating if module is initialized */
static bool s_initialized = false;

/*===========================================================================*/
/* Internal Functions                                                         */
/*===========================================================================*/

/**
 * @brief Calculate CRC for a fault log entry
 */
static uint32_t fault_entry_crc(const FaultLogEntry_t *entry)
{
    SMART_QSO_REQUIRE_NOT_NULL_OR(entry, 0U);
    size_t crc_offset = offsetof(FaultLogEntry_t, crc32);
    return smart_qso_crc32(entry, crc_offset);
}

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

/** Persistent fault log file: FAULT_LOG_FILE unless overridden */
static const char *fault_log_path(void)
{
    const char *path = getenv(FAULT_LOG_FILE_ENV);
    return ((path != NULL) && (path[0] != '\0')) ? path : FAULT_LOG_FILE;
}

SmartQsoResult_t fault_mgmt_init(void)
{
    /* Clear log on init - will be populated from persistent storage if available */
    memset(s_fault_log, 0, sizeof(s_fault_log));
    s_fault_log_count = 0;
    s_watchdog_triggered = false;

    /* Try to load from persistent storage */
    SmartQsoResult_t result = fault_log_load();
    if (result == SMART_QSO_OK) {
        printf("[FAULT] Loaded %zu fault log entries\n", s_fault_log_count);
    }

    s_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t fault_log_add(FaultType_t fault_type, FaultSeverity_t severity,
                               const char *description, double soc)
{
    SMART_QSO_REQUIRE_NOT_NULL(description);
    SMART_QSO_REQUIRE(fault_type > 0, "Invalid fault type");
    SMART_QSO_REQUIRE(severity >= FAULT_SEVERITY_INFO &&
                      severity <= FAULT_SEVERITY_CRITICAL, "Invalid severity");

    /* Make room if log is full */
    if (s_fault_log_count >= SMART_QSO_MAX_FAULT_ENTRIES) {
        /* Shift entries to make room (lose oldest entry) */
        memmove(&s_fault_log[0], &s_fault_log[1],
                sizeof(FaultLogEntry_t) * (SMART_QSO_MAX_FAULT_ENTRIES - 1));
        s_fault_log_count = SMART_QSO_MAX_FAULT_ENTRIES - 1;
    }

    /* Add new entry */
    FaultLogEntry_t *entry = &s_fault_log[s_fault_log_count];
    entry->timestamp_ms = smart_qso_now_ms();
    entry->fault_type = (uint8_t)fault_type;
    entry->severity = (uint8_t)severity;
    entry->soc_at_fault = soc;
    entry->recovered = false;

    /* Safe string copy with guaranteed null termination */
    strncpy(entry->description, description, SMART_QSO_FAULT_DESC_LEN - 1);
    entry->description[SMART_QSO_FAULT_DESC_LEN - 1] = '\0';

    entry->crc32 = fault_entry_crc(entry);
    s_fault_log_count++;

    /* Print fault message */
    printf("[FAULT] Type=%d Severity=%d: %s\n",
           fault_type, severity, description);

    /* Save to persistent storage */
    (void)fault_log_save();

    return SMART_QSO_OK;
}

size_t fault_log_get_count(void)
{
    return s_fault_log_count;
}

SmartQsoResult_t fault_log_get_entry(size_t index, FaultLogEntry_t *entry)
{
    SMART_QSO_REQUIRE_NOT_NULL(entry);
    SMART_QSO_REQUIRE(index < s_fault_log_count, "Index out of range");

    *entry = s_fault_log[index];
    return SMART_QSO_OK;
}

SmartQsoResult_t fault_log_get_last(FaultLogEntry_t *entry)
{
    SMART_QSO_REQUIRE_NOT_NULL(entry);

    if (s_fault_log_count == 0) {
        return SMART_QSO_ERROR;
    }

    *entry = s_fault_log[s_fault_log_count - 1];
    return SMART_QSO_OK;
}

SmartQsoResult_t fault_log_mark_recovered(size_t index)
{
    SMART_QSO_REQUIRE(index < s_fault_log_count, "Index out of range");

    s_fault_log[index].recovered = true;
    s_fault_log[index].crc32 = fault_entry_crc(&s_fault_log[index]);

    return fault_log_save();
}

SmartQsoResult_t fault_log_save(void)
{
    FILE *f = fopen(fault_log_path(), "wb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    size_t written = fwrite(s_fault_log, sizeof(FaultLogEntry_t),
                            s_fault_log_count, f);

    if (fclose(f) != 0) {
        return SMART_QSO_ERROR_IO;
    }

    if (written != s_fault_log_count) {
        return SMART_QSO_ERROR_IO;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t fault_log_load(void)
{
    FILE *f = fopen(fault_log_path(), "rb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    s_fault_log_count = fread(s_fault_log, sizeof(FaultLogEntry_t),
                               SMART_QSO_MAX_FAULT_ENTRIES, f);

    if (fclose(f) != 0) {
        s_fault_log_count = 0;
        return SMART_QSO_ERROR_IO;
    }

    /* Verify CRC of each entry */
    size_t valid_count = 0;
    for (size_t i = 0; i < s_fault_log_count; i++) {
        uint32_t expected_crc = fault_entry_crc(&s_fault_log[i]);
        if (expected_crc == s_fault_log[i].crc32) {
            if (valid_count != i) {
                s_fault_log[valid_count] = s_fault_log[i];
            }
            valid_count++;
        } else {
            printf("[FAULT] Discarded corrupt entry %zu\n", i);
        }
    }
    s_fault_log_count = valid_count;

    return SMART_QSO_OK;
}

SmartQsoResult_t fault_log_clear(void)
{
    memset(s_fault_log, 0, sizeof(s_fault_log));
    s_fault_log_count = 0;

    /* Remove persistent file */
    (void)remove(fault_log_path());

    return SMART_QSO_OK;
}

/*
 * Mode changes go through the state machine, never eps_set_power_mode()
 * (SW-14): its table and guards decide, and the entry action of the new
 * state sets the EPS power mode.
 */
SmartQsoResult_t fault_recover_thermal(double soc)
{
    SmartQsoResult_t result = SMART_QSO_OK;

    if (eps_is_payload_enabled()) {
        (void)fault_log_add(FAULT_TYPE_THERMAL, FAULT_SEVERITY_WARNING,
                            "Thermal fault recovery: disabling payload", soc);
        (void)eps_control_payload(false, soc);
        result = sm_process_event(sys_get_sm_context(), EVENT_THERMAL_FAULT, NULL);
    }

    return result;
}

SmartQsoResult_t fault_recover_power(double soc)
{
    SmState_t state = STATE_BOOT;
    SmartQsoResult_t result = sm_get_state(sys_get_sm_context(), &state);

    /* Not sunlit: recovery sheds load or restores IDLE, never enters ACTIVE */
    SmEvent_t event = (result == SMART_QSO_OK) ? sm_power_event(state, soc, false) : EVENT_NONE;
    if (event == EVENT_POWER_CRITICAL) {
        (void)fault_log_add(FAULT_TYPE_POWER_CRITICAL, FAULT_SEVERITY_ERROR,
                            "Critical power fault: shedding load", soc);
    } else if (event == EVENT_POWER_LOW) {
        (void)fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING,
                            "Power fault recovery: entering IDLE mode", soc);
    } else {
        /* EVENT_POWER_OK (SOC recovered) or nothing to do */
    }
    if (event != EVENT_NONE) {
        result = sm_process_event(sys_get_sm_context(), event, NULL);
    }

    return result;
}

SmartQsoResult_t fault_handle_watchdog(double soc)
{
    fault_log_add(FAULT_TYPE_WATCHDOG, FAULT_SEVERITY_ERROR,
                  "Watchdog timeout: system reset", soc);
    s_watchdog_triggered = true;
    /* In real implementation, this would trigger a system reset */
    /* For simulation, we just log it and continue */

    return SMART_QSO_OK;
}

bool fault_was_watchdog_triggered(void)
{
    return s_watchdog_triggered;
}

void fault_clear_watchdog_flag(void)
{
    s_watchdog_triggered = false;
}
