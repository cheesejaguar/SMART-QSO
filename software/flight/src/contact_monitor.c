/**
 * @file contact_monitor.c
 * @brief Loss-of-contact transmit inhibit
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-205 Cease transmissions after N days without an authenticated telecommand
 */

#include "contact_monitor.h"
#include "rf_control.h"
#include "fault_mgmt.h"
#include "hal/hal_flash.h"
#include "nvm_record.h"
#include "smart_qso.h"
#include <stdint.h>

/*******************************************************************************
 * Private Constants and Data
 ******************************************************************************/

#define MAGIC_COUNTER   0x4C4F4343U  /* "LOCC" */
#define MAGIC_TIMEOUT   0x4C4F4354U  /* "LOCT" */
#define SECONDS_PER_DAY 86400U

static uint32_t s_seconds = 0U;          /**< Seconds without contact */
static uint32_t s_unpersisted_s = 0U;    /**< Seconds since the last save */
static uint32_t s_remainder_ms = 0U;     /**< Sub-second carry */
static uint16_t s_timeout_days = CONTACT_TIMEOUT_DEFAULT_DAYS;
static bool s_inhibited = false;
static bool s_initialized = false;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

static uint32_t timeout_seconds(void)
{
    return (uint32_t)s_timeout_days * SECONDS_PER_DAY;
}

static void save_counter(void)
{
    if (nvm_record_save_at(FLASH_REGION_CONTACT, CONTACT_NVM_COUNTER, MAGIC_COUNTER,
                           &s_seconds, sizeof(s_seconds)) == SMART_QSO_OK) {
        s_unpersisted_s = 0U;
    }
}

/** Apply the inhibit when the counter reaches the timeout */
static void evaluate(void)
{
    bool inhibit = (s_seconds >= timeout_seconds());

    if (inhibit && !s_inhibited) {
        (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_WARNING,
                            "No contact: transmitters inhibited", (double)s_timeout_days);
    }
    s_inhibited = inhibit;
    rf_control_set_inhibit(RF_INHIBIT_CONTACT, inhibit);
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t contact_monitor_init(void)
{
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;
    uint32_t word = 0U;

    s_unpersisted_s = 0U;
    s_remainder_ms = 0U;
    s_inhibited = false;

    /* Timeout: default unless a valid in-range record exists */
    s_timeout_days = CONTACT_TIMEOUT_DEFAULT_DAYS;
    if ((nvm_record_load_at(FLASH_REGION_CONTACT, CONTACT_NVM_TIMEOUT, MAGIC_TIMEOUT,
                            &word, sizeof(word), &status) == SMART_QSO_OK) &&
        (status == NVM_RECORD_VALID) &&
        (word >= CONTACT_TIMEOUT_MIN_DAYS) && (word <= CONTACT_TIMEOUT_MAX_DAYS)) {
        s_timeout_days = (uint16_t)word;
    }

    /* Counter: blank -> 0; valid -> + boot penalty; corrupt -> timed out */
    word = 0U;
    SmartQsoResult_t result = nvm_record_load_at(FLASH_REGION_CONTACT, CONTACT_NVM_COUNTER,
                                                 MAGIC_COUNTER, &word, sizeof(word), &status);
    if ((result == SMART_QSO_OK) && (status == NVM_RECORD_BLANK)) {
        s_seconds = 0U;
    } else if ((result == SMART_QSO_OK) && (status == NVM_RECORD_VALID)) {
        s_seconds = (word > (UINT32_MAX - CONTACT_PERSIST_INTERVAL_S))
                        ? UINT32_MAX : (word + CONTACT_PERSIST_INTERVAL_S);
    } else {
        s_seconds = timeout_seconds();
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_ERROR,
                            "Contact counter unreadable: transmitters inhibited", 0.0);
    }

    /* Write at once so a reset loop accumulates the penalty */
    save_counter();
    s_initialized = true;
    evaluate();

    return SMART_QSO_OK;
}

void contact_monitor_tick(uint32_t elapsed_ms)
{
    if (!s_initialized) {
        return;
    }

    uint32_t total_ms = s_remainder_ms + elapsed_ms;
    uint32_t seconds = total_ms / 1000U;
    s_remainder_ms = total_ms % 1000U;

    s_seconds = (seconds > (UINT32_MAX - s_seconds)) ? UINT32_MAX : (s_seconds + seconds);
    s_unpersisted_s += seconds;
    if (s_unpersisted_s >= CONTACT_PERSIST_INTERVAL_S) {
        save_counter();
    }

    evaluate();
}

void contact_monitor_contact(void)
{
    if (!s_initialized) {
        return;
    }

    s_seconds = 0U;
    s_remainder_ms = 0U;
    save_counter();
    evaluate();
}

SmartQsoResult_t contact_monitor_set_timeout_days(uint16_t days)
{
    if ((days < CONTACT_TIMEOUT_MIN_DAYS) || (days > CONTACT_TIMEOUT_MAX_DAYS)) {
        return SMART_QSO_ERROR_INVALID;
    }

    s_timeout_days = days;
    uint32_t word = days;
    SmartQsoResult_t result = nvm_record_save_at(FLASH_REGION_CONTACT, CONTACT_NVM_TIMEOUT,
                                                 MAGIC_TIMEOUT, &word, sizeof(word));
    if (s_initialized) {
        evaluate();
    }
    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

uint16_t contact_monitor_get_timeout_days(void)
{
    return s_timeout_days;
}

uint32_t contact_monitor_seconds_since_contact(void)
{
    return s_seconds;
}

bool contact_monitor_inhibited(void)
{
    return s_inhibited;
}
