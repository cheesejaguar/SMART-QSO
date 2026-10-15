/**
 * @file rf_control.c
 * @brief Transmitter permission and latched TX-OFF implementation
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 */

#include "rf_control.h"
#include "nvm_record.h"
#include "fault_mgmt.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

/** Persisted record identifier ("RFTX") */
#define RF_RECORD_MAGIC         0x52465458U

/** Persisted state words; any other value is treated as inhibited */
#define RF_STATE_PERMITTED      0x5A5A5A5AU
#define RF_STATE_INHIBITED      0xA5A5A5A5U

/*******************************************************************************
 * Private Data
 ******************************************************************************/

static bool s_initialized = false;
static bool s_inhibited = true;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/** Configure and drive the RF transmitter enable line */
static void drive_tx_enable(bool enable)
{
    (void)hal_gpio_config(GPIO_PIN_RF_TX_EN, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN);
    (void)hal_gpio_set(GPIO_PIN_RF_TX_EN, enable);
}

static SmartQsoResult_t persist_state(uint32_t state_word)
{
    SmartQsoResult_t result = nvm_record_save(FLASH_REGION_RF_CONTROL, RF_RECORD_MAGIC,
                                              &state_word, sizeof(state_word));
    return (result == SMART_QSO_OK) ? SMART_QSO_OK : SMART_QSO_ERROR_IO;
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t rf_control_init(void)
{
    uint32_t state_word = RF_STATE_INHIBITED;
    NvmRecordStatus_t status = NVM_RECORD_CORRUPT;

    SmartQsoResult_t result = nvm_record_load(FLASH_REGION_RF_CONTROL, RF_RECORD_MAGIC,
                                              &state_word, sizeof(state_word), &status);

    if (result != SMART_QSO_OK) {
        s_inhibited = true;
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_ERROR,
                            "RF state unreadable: TX inhibited", 0.0);
    } else if (status == NVM_RECORD_BLANK) {
        /* First boot: nothing has ever commanded TX-OFF */
        s_inhibited = false;
    } else if (status == NVM_RECORD_CORRUPT) {
        s_inhibited = true;
        (void)fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_ERROR,
                            "RF state corrupt: TX inhibited", 0.0);
    } else {
        s_inhibited = (state_word != RF_STATE_PERMITTED);
    }

    drive_tx_enable(!s_inhibited);
    s_initialized = true;

    return SMART_QSO_OK;
}

SmartQsoResult_t rf_control_tx_off(void)
{
    /* Silence the hardware first so a flash failure cannot leave TX on */
    s_inhibited = true;
    drive_tx_enable(false);

    SmartQsoResult_t result = persist_state(RF_STATE_INHIBITED);
    (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_WARNING,
                        (result == SMART_QSO_OK) ? "TX-OFF latched" : "TX-OFF applied, not persisted",
                        0.0);

    return result;
}

SmartQsoResult_t rf_control_tx_on(void)
{
    SmartQsoResult_t result = persist_state(RF_STATE_PERMITTED);

    s_inhibited = false;
    drive_tx_enable(true);
    (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_INFO,
                        (result == SMART_QSO_OK) ? "TX enabled" : "TX enabled, not persisted",
                        0.0);

    return result;
}

bool rf_control_tx_permitted(void)
{
    return s_initialized && !s_inhibited;
}
