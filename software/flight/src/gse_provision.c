/**
 * @file gse_provision.c
 * @brief Telecommand key provisioning over the GSE umbilical (integration only)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include "gse_provision.h"
#include "cmd_auth.h"
#include "rf_silence.h"
#include "deployment.h"
#include "fault_mgmt.h"
#include "safe_string.h"
#include "hal/hal_gpio.h"
#include "hal/hal_uart.h"
#include "smart_qso.h"
#include <stddef.h>
#include <stdint.h>

/*******************************************************************************
 * Private Constants
 ******************************************************************************/

#define OFFSET_OP       4U
#define OFFSET_SLOT     5U
#define OFFSET_KEY_ID   6U
#define OFFSET_KEY      7U
#define OFFSET_CRC      (GSE_FRAME_SIZE - 4U)

static const uint8_t FRAME_MAGIC[4] = {'S', 'Q', 'G', 'P'};
static const uint8_t REPLY_MAGIC[4] = {'S', 'Q', 'G', 'R'};

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t gse_init(void)
{
    SmartQsoResult_t result = hal_gpio_config(GPIO_PIN_UMBILICAL_SENSE, GPIO_DIR_INPUT,
                                              GPIO_PULL_DOWN);
    if (result == SMART_QSO_OK) {
        result = hal_uart_init(UART_PORT_DEBUG, NULL);
    }
    return result;
}

bool gse_umbilical_present(void)
{
    return hal_gpio_get(GPIO_PIN_UMBILICAL_SENSE);
}

GseStatus_t gse_process_frame(const uint8_t *frame, size_t len)
{
    if (!gse_umbilical_present()) {
        return GSE_STATUS_LOCKED;
    }
    if ((frame == NULL) || (len != GSE_FRAME_SIZE)) {
        return GSE_STATUS_BAD_FRAME;
    }
    for (size_t i = 0U; i < sizeof(FRAME_MAGIC); i++) {
        if (frame[i] != FRAME_MAGIC[i]) {
            return GSE_STATUS_BAD_FRAME;
        }
    }
    uint32_t stored = ((uint32_t)frame[OFFSET_CRC] << 24) | ((uint32_t)frame[OFFSET_CRC + 1U] << 16) |
                      ((uint32_t)frame[OFFSET_CRC + 2U] << 8) | (uint32_t)frame[OFFSET_CRC + 3U];
    if (smart_qso_crc32(frame, OFFSET_CRC) != stored) {
        return GSE_STATUS_BAD_FRAME;
    }

    SmartQsoResult_t result;
    switch (frame[OFFSET_OP]) {
        case GSE_OP_PROVISION:
            result = cmd_auth_provision_slot(frame[OFFSET_SLOT], &frame[OFFSET_KEY],
                                             frame[OFFSET_KEY_ID]);
            break;
        case GSE_OP_ACTIVATE:
            result = cmd_auth_rotate(frame[OFFSET_SLOT]);
            break;
        case GSE_OP_ZEROIZE:
            result = cmd_auth_zeroize();
            break;
        case GSE_OP_ARM_FLIGHT:
            /* One action arms both: two steps would be two chances to ship a
             * unit that completed its silence or deployment on the bench */
            result = rf_silence_arm();
            if (deployment_arm() != SMART_QSO_OK) {
                result = SMART_QSO_ERROR_IO;
            }
            break;
        default:
            return GSE_STATUS_BAD_OP;
    }

    (void)fault_log_add(FAULT_TYPE_COMMAND, FAULT_SEVERITY_INFO,
                        (result == SMART_QSO_OK) ? "GSE key operation done" : "GSE key operation refused",
                        (double)frame[OFFSET_OP]);
    return (result == SMART_QSO_OK) ? GSE_STATUS_OK : GSE_STATUS_REJECTED;
}

bool gse_poll(void)
{
    uint8_t frame[GSE_FRAME_SIZE];
    size_t got = 0U;

    if (!gse_umbilical_present()) {
        /* Nothing on the umbilical port is honoured without the umbilical */
        if (hal_uart_available(UART_PORT_DEBUG) > 0U) {
            (void)hal_uart_flush(UART_PORT_DEBUG);
        }
        return false;
    }
    if (hal_uart_available(UART_PORT_DEBUG) < GSE_FRAME_SIZE) {
        return false;
    }
    if ((hal_uart_read(UART_PORT_DEBUG, frame, sizeof(frame), 0U, &got) != SMART_QSO_OK) ||
        (got != sizeof(frame))) {
        return false;
    }

    GseStatus_t status = gse_process_frame(frame, got);
    (void)safe_memset(frame, sizeof(frame), 0, sizeof(frame));

    uint8_t reply[GSE_RESPONSE_SIZE];
    (void)safe_memcpy(reply, sizeof(reply), REPLY_MAGIC, sizeof(REPLY_MAGIC));
    reply[4] = (uint8_t)status;
    (void)hal_uart_write(UART_PORT_DEBUG, reply, sizeof(reply));
    return true;
}
