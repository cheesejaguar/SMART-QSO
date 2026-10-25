/**
 * @file downlink.h
 * @brief Downlink transmitter: the last gate before a frame goes to the TNC
 *
 * Implemented in Rust (software/rust/sqso-fsw/src/downlink.rs,
 * software/rust/sqso-ffi/src/downlink.rs).
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The OBC hands each AX.25 frame (addresses through information, no FCS)
 * to a KISS TNC on the radio UART (hal_radio_transmit()). The TNC adds the
 * flags and FCS, modulates AFSK 1200 and keys the 145.8 MHz transmitter
 * (SQSO-RF-TX-001 §10; SQSO-CRL-001 CR-2026-005). downlink_transmit():
 *  - refuses the frame unless rf_control_tx_permitted() at that moment
 *    (TX-OFF, RF silence, loss of contact); nothing is queued;
 *  - sends one frame at a time: not while the last may still be on the air;
 *  - holds keyed time to DOWNLINK_DUTY_PERCENT, with a credit of at most
 *    DOWNLINK_DUTY_WINDOW_MS's worth; every frame that fits is on the air for
 *    less than DOWNLINK_MAX_KEYDOWN_MS (SRS-P002);
 *  - selects the commanded power level on GPIO_PIN_TX_PWR_SEL (high: 1 W),
 *    capped by the VSWR protection.
 * downlink_poll(), each main-loop pass, reads the transmitter monitors
 * (ADC_CHANNEL_TX_TEMP, _TX_FWD, _TX_REV) while a frame may be on the air:
 *  - reflected power above a quarter of forward (VSWR 3:1) on
 *    DOWNLINK_VSWR_STRIKES samples in a row with output present: power capped
 *    at 0.5 W until reset; at 0.5 W the downlink stops for DOWNLINK_BACKOFF_MS;
 *  - a keyed window with samples and no forward power is reported;
 *  - above DOWNLINK_TEMP_LIMIT_C nothing is sent until the temperature is
 *    below DOWNLINK_TEMP_RESUME_C.
 * Each of these is logged (FAULT_TYPE_RF_TX). The thresholds are TBR with the
 * transmitter; the window a frame is on the air is estimated from its
 * length, the UART rate and the TNC parameters (TBR with the TNC).
 *
 * @requirement SRS-I023 Downlink frames to the TNC in KISS framing, with the TNC parameters
 * @requirement SRS-I024 No downlink frame unless transmission is permitted when it is handed over
 * @requirement SRS-I025 Key-down time per frame and duty limited
 * @requirement SRS-I026 Downlink power at the commanded level, never above 1 W
 * @requirement SRS-I027 Transmitter temperature and VSWR protection
 */

#ifndef SMART_QSO_DOWNLINK_H
#define SMART_QSO_DOWNLINK_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/** Largest frame: AX25_MAX_FRAME_LEN less the FCS, which the TNC adds */
#define DOWNLINK_MAX_FRAME_LEN      328U
/** Longest a frame may keep the transmitter on (SRS-P002) */
#define DOWNLINK_MAX_KEYDOWN_MS     5000U
/** Keyed time as a share of all time, percent (TBR) */
#define DOWNLINK_DUTY_PERCENT       25U
/** The keyed-time credit holds this window's worth at most */
#define DOWNLINK_DUTY_WINDOW_MS     60000U
/** Reflected over forward power above which the match is bad (VSWR 3:1) */
#define DOWNLINK_VSWR_LIMIT_RATIO   0.25
/** Forward power that counts as output present, W */
#define DOWNLINK_RF_PRESENT_W       0.1
/** Bad-match samples in a row before acting */
#define DOWNLINK_VSWR_STRIKES       3U
/** Downlink stopped after a bad match at 0.5 W, ms */
#define DOWNLINK_BACKOFF_MS         600000U
/** Transmitter temperature above which nothing is sent, °C (TBR) */
#define DOWNLINK_TEMP_LIMIT_C       70.0
/** Temperature below which sending resumes, °C */
#define DOWNLINK_TEMP_RESUME_C      60.0
/** The TNC's keying delay (KISS TXDELAY), ms; monitors are read after it */
#define DOWNLINK_TXDELAY_MS         300U

/** Downlink counters and protection state */
typedef struct {
    uint32_t sent;              /**< Frames handed to the TNC */
    uint32_t inhibited;         /**< Refused: not permitted or not initialized */
    uint32_t invalid;           /**< Refused: empty or too long */
    uint32_t over_temperature;  /**< Refused: transmitter too hot */
    uint32_t backed_off;        /**< Refused: backed off after a bad match */
    uint32_t busy;              /**< Refused: the last frame still on the air */
    uint32_t duty;              /**< Refused: no keyed-time credit */
    uint32_t hal_errors;        /**< The HAL did not take a frame */
    uint32_t vswr_events;       /**< Bad-match detections */
    uint32_t no_output;         /**< Keyed windows without output */
    uint32_t level_cap;         /**< Highest power level allowed (RfPowerLevel_t) */
    bool over_temp;             /**< Held for temperature */
    bool backoff;               /**< Backed off now */
    bool keyed;                 /**< A frame may be on the air now */
} DownlinkStatus_t;

/**
 * @brief Start: nothing on the air, full credit, power cap 1 W
 *
 * Call after rf_control_init(). Until then every frame is refused.
 */
SmartQsoResult_t downlink_init(void);

/**
 * @brief Hand one AX.25 frame to the TNC, if every gate allows it now
 *
 * @param frame Addresses through information field (no flags, no FCS)
 * @param len   1 to DOWNLINK_MAX_FRAME_LEN bytes
 * @return SMART_QSO_OK handed over;
 *         SMART_QSO_ERROR not permitted (TX-OFF, an inhibit) or not initialized;
 *         SMART_QSO_ERROR_INVALID empty or too long;
 *         SMART_QSO_ERROR_BUSY on the air, out of duty credit, too hot or backed off;
 *         SMART_QSO_ERROR_NULL_PTR frame is NULL;
 *         otherwise the HAL's error (nothing on the air)
 */
SmartQsoResult_t downlink_transmit(const uint8_t *frame, size_t len);

/**
 * @brief Each main-loop pass: watch the transmitter while it may be keyed
 */
void downlink_poll(void);

/**
 * @brief Counters and protection state
 */
SmartQsoResult_t downlink_get_status(DownlinkStatus_t *status);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_DOWNLINK_H */
