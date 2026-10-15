/**
 * @file gse_provision.h
 * @brief Key provisioning and flight arming over the GSE umbilical (integration only)
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Keys are loaded through the umbilical UART (UART_PORT_DEBUG) and only while
 * GPIO_PIN_UMBILICAL_SENSE is high, which the GSE connector provides and the
 * flight harness pulls low. Nothing here is reachable over RF.
 *
 * Frame (GSE_FRAME_SIZE bytes):
 *
 *     "SQGP" | op (1) | slot (1) | key_id (1) | key (32) | CRC-32 (4, BE)
 *
 * CRC-32 is IEEE 802.3 (zlib) over the first 39 bytes. Operations: PROVISION
 * writes a key into a slot (0-3 operational spares, 4 recovery); ACTIVATE makes
 * a spare the active operational key; ZEROIZE erases the key table;
 * ARM_RF_SILENCE re-arms the 45-minute post-deployment silence. The reply
 * is "SQGR" | status (1).
 *
 * Ground tool: software/ground/tools/key_provision.py
 * Interface definition: docs/interfaces/TELECOMMAND_AUTHENTICATION.md section 9
 */

#ifndef SMART_QSO_GSE_PROVISION_H
#define SMART_QSO_GSE_PROVISION_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/** Provisioning frame length */
#define GSE_FRAME_SIZE      43U

/** Reply length: "SQGR" | status */
#define GSE_RESPONSE_SIZE   5U

/** Operations */
typedef enum {
    GSE_OP_PROVISION = 0x01, /**< Write a key into a slot */
    GSE_OP_ACTIVATE  = 0x02, /**< Activate a spare operational key */
    GSE_OP_ZEROIZE   = 0x03, /**< Erase the key table */
    GSE_OP_ARM_RF_SILENCE = 0x04 /**< Re-arm the post-deployment RF silence (last action before shipping) */
} GseOp_t;

/** Outcome, also sent as the reply status byte */
typedef enum {
    GSE_STATUS_OK        = 0, /**< Done */
    GSE_STATUS_LOCKED    = 1, /**< Umbilical not present */
    GSE_STATUS_BAD_FRAME = 2, /**< Length, magic or CRC wrong */
    GSE_STATUS_BAD_OP    = 3, /**< Unknown operation */
    GSE_STATUS_REJECTED  = 4  /**< Key table refused (bad slot, not a spare, I/O) */
} GseStatus_t;

/**
 * @brief Configure the umbilical sense input (pull-down) and the GSE UART
 */
SmartQsoResult_t gse_init(void);

/**
 * @brief Whether the GSE umbilical is connected
 */
bool gse_umbilical_present(void);

/**
 * @brief Validate and execute one provisioning frame
 */
GseStatus_t gse_process_frame(const uint8_t *frame, size_t len);

/**
 * @brief Read and execute one frame from the umbilical UART, if connected
 *
 * Without the umbilical, received bytes are discarded.
 *
 * @return true if a frame was processed
 */
bool gse_poll(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_GSE_PROVISION_H */
