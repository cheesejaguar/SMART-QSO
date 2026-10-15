/**
 * @file rf_silence.c
 * @brief 45-minute post-deployment RF silence (CDS Rev 14.1 2.4.5, 2.3.2.4)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-224 No RF signal earlier than 45 minutes after deployment; switch toggle resets the timer
 */

#include "rf_silence.h"
#include "rf_control.h"
#include "separation_switch.h"
#include "nvm_record.h"
#include "fault_mgmt.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_reset.h"
#include <stdio.h>

/*******************************************************************************
 * Private Constants and Data
 ******************************************************************************/

#define RECORD_MAGIC    0x52465349U  /* "RFSI" */

/** Record: state (4) | elapsed seconds (4) */
typedef struct {
    uint32_t state;
    uint32_t elapsed_s;
} SilenceRecord_t;

static RfSilenceState_t s_state = RF_SILENCE_COUNTING;
static uint32_t s_elapsed_s = 0U;
static uint32_t s_unpersisted_s = 0U;
static uint32_t s_remainder_ms = 0U;
static bool s_initialized = false;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

static void save(void)
{
    SilenceRecord_t rec = {(uint32_t)s_state, s_elapsed_s};

    if (nvm_record_save(FLASH_REGION_RF_SILENCE, RECORD_MAGIC, &rec, sizeof(rec)) == SMART_QSO_OK) {
        s_unpersisted_s = 0U;
    }
}

static void apply(void)
{
    rf_control_set_inhibit(RF_INHIBIT_RF_SILENCE, s_state != RF_SILENCE_COMPLETE);
}

static void restart_count(void)
{
    s_state = RF_SILENCE_COUNTING;
    s_elapsed_s = 0U;
    s_remainder_ms = 0U;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t rf_silence_init(void)
{
    SilenceRecord_t rec = {0U, 0U};
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;
    HalResetCause_t cause = hal_reset_cause();
    bool resumable = (cause == HAL_RESET_WATCHDOG) || (cause == HAL_RESET_SOFTWARE);

    separation_switch_init();
    s_unpersisted_s = 0U;
    restart_count();

    SmartQsoResult_t result = nvm_record_load(FLASH_REGION_RF_SILENCE, RECORD_MAGIC,
                                              &rec, sizeof(rec), &status);
    if ((result == SMART_QSO_OK) && (status == NVM_RECORD_VALID)) {
        if (rec.state == (uint32_t)RF_SILENCE_COMPLETE) {
            s_state = RF_SILENCE_COMPLETE;
        } else if ((rec.state == (uint32_t)RF_SILENCE_COUNTING) && resumable &&
                   (rec.elapsed_s < RF_SILENCE_DURATION_S)) {
            s_elapsed_s = rec.elapsed_s;
        } else {
            /* ARMED, power removed, or unrecognized: count from 0 */
        }
    } else if ((result != SMART_QSO_OK) || (status == NVM_RECORD_CORRUPT)) {
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_ERROR,
                            "RF silence record unreadable: counting from 0", 0.0);
    } else {
        /* Blank: first boot, count from 0 */
    }

    save();
    s_initialized = true;
    apply();

    /* Observable at every boot (pre-ship check) */
    printf("[RF_SILENCE] state=%s elapsed=%lus\n",
           (s_state == RF_SILENCE_COMPLETE) ? "COMPLETE" : "COUNTING",
           (unsigned long)s_elapsed_s);
    return SMART_QSO_OK;
}

void rf_silence_tick(uint32_t elapsed_ms)
{
    if (!s_initialized) {
        return;
    }

    if (separation_switch_stowed()) {
        /* CDS 2.3.2.4: a switch toggle restarts the timer, even after completion */
        if ((s_state != RF_SILENCE_COUNTING) || (s_elapsed_s != 0U)) {
            restart_count();
            save();
            apply();
        }
        return;
    }

    if (s_state != RF_SILENCE_COUNTING) {
        return;  /* COMPLETE, or ARMED (waits for the next boot) */
    }

    uint32_t total_ms = s_remainder_ms + elapsed_ms;
    uint32_t seconds = total_ms / 1000U;
    s_remainder_ms = total_ms % 1000U;
    s_elapsed_s += seconds;
    s_unpersisted_s += seconds;

    if (s_elapsed_s >= RF_SILENCE_DURATION_S) {
        s_state = RF_SILENCE_COMPLETE;
        save();
        apply();
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_INFO,
                            "RF silence complete: radio enabled", 0.0);
    } else if (s_unpersisted_s >= RF_SILENCE_PERSIST_INTERVAL_S) {
        save();
    } else {
        /* Persist at the next interval */
    }
}

SmartQsoResult_t rf_silence_arm(void)
{
    SilenceRecord_t rec = {(uint32_t)RF_SILENCE_ARMED, 0U};

    s_state = RF_SILENCE_ARMED;
    s_elapsed_s = 0U;
    s_remainder_ms = 0U;
    apply();

    SmartQsoResult_t result = nvm_record_save(FLASH_REGION_RF_SILENCE, RECORD_MAGIC,
                                              &rec, sizeof(rec));
    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

bool rf_silence_active(void)
{
    return s_state != RF_SILENCE_COMPLETE;
}

RfSilenceState_t rf_silence_get_state(void)
{
    return s_state;
}

uint32_t rf_silence_elapsed_seconds(void)
{
    return s_elapsed_s;
}
