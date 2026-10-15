/**
 * @file uplink.h
 * @brief Telecommand receive path: receiver -> AX.25 -> command handler
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Each received AX.25 UI frame (PID 0xF0) addressed to this station carries
 * one command frame in its information field (SQSO-ICD-013 Section 3). The
 * FCS is checked here, then cmd_process() authenticates and executes it.
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 */

#ifndef SMART_QSO_UPLINK_H
#define SMART_QSO_UPLINK_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include "cmd_handler.h"
#include <stdbool.h>
#include <stdint.h>

/** Receive statistics (telemetry) */
typedef struct {
    uint32_t frames;        /**< Frames delivered by the receiver */
    uint32_t rejected;      /**< Bad FCS, malformed or not UI/0xF0 */
    uint32_t not_for_us;    /**< Addressed to another station */
    uint32_t commands;      /**< Passed to the command handler */
} UplinkStats_t;

/**
 * @brief Set this station's address and start the receiver
 */
SmartQsoResult_t uplink_init(const char *callsign, uint8_t ssid);

/**
 * @brief Process at most one received frame
 *
 * @param[out] response Command response, valid when true is returned (may be NULL)
 * @return true if a frame reached the command handler
 */
bool uplink_poll(CmdResponse_t *response);

/**
 * @brief Receive statistics
 */
void uplink_get_stats(UplinkStats_t *stats);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_UPLINK_H */
