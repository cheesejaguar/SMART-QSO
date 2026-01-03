/**
 * @file telemetry.h
 * @brief Telemetry formatting interface for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * This module handles telemetry collection, formatting, and transmission.
 * Follows NASA Ames SmallSat heritage for telemetry design.
 *
 * @requirement SRS-TLM-001 System shall generate telemetry at configurable rate
 * @requirement SRS-TLM-002 Telemetry shall include all critical parameters
 * @requirement SRS-TLM-003 Telemetry shall be CRC protected
 */

#ifndef SMART_QSO_TELEMETRY_H
#define SMART_QSO_TELEMETRY_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"
#include <stdint.h>
#include <stdbool.h>

/*******************************************************************************
 * Constants
 ******************************************************************************/

/** Maximum telemetry frame size */
#define TLM_MAX_FRAME_SIZE      256U

/** Telemetry sync pattern */
#define TLM_SYNC_WORD           0x1ACFFC1DU

/** Default telemetry rate (ms) */
#define TLM_DEFAULT_RATE_MS     60000U

/** Minimum telemetry rate (ms) */
#define TLM_MIN_RATE_MS         10000U

/** Maximum telemetry rate (ms) */
#define TLM_MAX_RATE_MS         300000U

/*******************************************************************************
 * Telemetry Types
 ******************************************************************************/

/**
 * @brief Telemetry frame types
 */
typedef enum {
    TLM_TYPE_HOUSEKEEPING   = 0x01,  /**< Housekeeping telemetry */
    TLM_TYPE_SCIENCE        = 0x02,  /**< Science data */
    TLM_TYPE_EVENT          = 0x03,  /**< Event notification */
    TLM_TYPE_BEACON         = 0x04,  /**< Beacon frame */
    TLM_TYPE_ADCS           = 0x05,  /**< ADCS telemetry */
    TLM_TYPE_EPS            = 0x06,  /**< EPS telemetry */
    TLM_TYPE_PAYLOAD        = 0x07,  /**< Payload telemetry */
    TLM_TYPE_FILE           = 0x08   /**< File transfer */
} TlmType_t;

/**
 * @brief Telemetry frame header
 */
typedef struct {
    uint32_t sync_word;             /**< Sync pattern */
    uint8_t version;                /**< Frame version */
    uint8_t type;                   /**< Frame type */
    uint16_t sequence;              /**< Sequence counter */
    uint32_t timestamp_s;           /**< Mission time (seconds) */
    uint16_t data_len;              /**< Payload length */
} __attribute__((packed)) TlmHeader_t;

/**
 * @brief Housekeeping telemetry payload
 */
typedef struct {
    /* Power */
    uint16_t battery_voltage_mv;    /**< Battery voltage (mV) */
    int16_t battery_current_ma;     /**< Battery current (mA) */
    uint8_t state_of_charge;        /**< SOC (%) */
    uint8_t power_mode;             /**< Current power mode */

    /* Thermal */
    int8_t obc_temp_c;              /**< OBC temperature (C) */
    int8_t eps_temp_c;              /**< EPS temperature (C) */
    int8_t battery_temp_c;          /**< Battery temperature (C) */
    int8_t payload_temp_c;          /**< Payload temperature (C) */

    /* Status */
    uint8_t op_state;               /**< Operational state */
    uint8_t fault_flags;            /**< Active fault flags */
    uint16_t boot_count;            /**< Boot counter */
    uint32_t uptime_s;              /**< Current uptime */

    /* Communications */
    uint16_t packets_sent;          /**< Packets transmitted */
    uint16_t packets_received;      /**< Packets received */
    uint16_t beacon_count;          /**< Beacons sent */

    /* ADCS */
    uint8_t adcs_mode;              /**< ADCS mode */
    uint8_t detumbled;              /**< Detumble achieved */
} __attribute__((packed)) TlmHousekeeping_t;

/**
 * @brief EPS telemetry payload
 */
typedef struct {
    uint16_t battery_voltage_mv;    /**< Battery voltage */
    int16_t battery_current_ma;     /**< Battery current */
    uint16_t solar_voltage_mv;      /**< Solar panel voltage */
    int16_t solar_current_ma;       /**< Solar panel current */
    uint8_t state_of_charge;        /**< SOC (%) */
    uint8_t power_mode;             /**< Power mode */
    uint8_t heater_enabled;         /**< Heater state */
    uint8_t payload_enabled;        /**< Payload state */
    int8_t battery_temp_c;          /**< Battery temperature */
    int8_t pcb_temp_c;              /**< EPS PCB temperature */
} __attribute__((packed)) TlmEps_t;

/**
 * @brief ADCS telemetry payload
 */
typedef struct {
    int16_t mag_x_ut_x10;           /**< Magnetometer X (0.1 uT) */
    int16_t mag_y_ut_x10;           /**< Magnetometer Y (0.1 uT) */
    int16_t mag_z_ut_x10;           /**< Magnetometer Z (0.1 uT) */
    int16_t gyro_x_dps_x10;         /**< Gyroscope X (0.1 deg/s) */
    int16_t gyro_y_dps_x10;         /**< Gyroscope Y (0.1 deg/s) */
    int16_t gyro_z_dps_x10;         /**< Gyroscope Z (0.1 deg/s) */
    int16_t sun_x_x100;             /**< Sun vector X (0.01) */
    int16_t sun_y_x100;             /**< Sun vector Y (0.01) */
    int16_t sun_z_x100;             /**< Sun vector Z (0.01) */
    uint8_t mode;                   /**< ADCS mode */
    uint8_t status;                 /**< Status flags */
} __attribute__((packed)) TlmAdcs_t;

/**
 * @brief Complete telemetry frame
 */
typedef struct {
    TlmHeader_t header;             /**< Frame header */
    uint8_t payload[TLM_MAX_FRAME_SIZE - sizeof(TlmHeader_t) - 4];
    uint32_t crc32;                 /**< Frame CRC */
} __attribute__((packed)) TlmFrame_t;

/**
 * @brief Telemetry statistics
 */
typedef struct {
    uint32_t frames_generated;      /**< Total frames generated */
    uint32_t frames_transmitted;    /**< Frames transmitted */
    uint32_t frames_failed;         /**< Transmission failures */
    uint32_t last_tx_time_ms;       /**< Last transmission time */
    uint16_t sequence_number;       /**< Current sequence number */
} TlmStats_t;

/*******************************************************************************
 * Public Function Declarations
 ******************************************************************************/

/**
 * @brief Initialize telemetry module
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_init(void);

/**
 * @brief Set telemetry rate
 *
 * @param[in] rate_ms Rate in milliseconds
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_set_rate(uint32_t rate_ms);

/**
 * @brief Get current telemetry rate
 *
 * @return Current rate in milliseconds
 */
uint32_t tlm_get_rate(void);

/**
 * @brief Generate housekeeping telemetry frame
 *
 * @param[out] frame Output frame buffer
 * @param[out] frame_len Actual frame length
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_generate_housekeeping(TlmFrame_t *frame, size_t *frame_len);

/**
 * @brief Generate EPS telemetry frame
 *
 * @param[out] frame Output frame buffer
 * @param[out] frame_len Actual frame length
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_generate_eps(TlmFrame_t *frame, size_t *frame_len);

/**
 * @brief Generate ADCS telemetry frame
 *
 * @param[out] frame Output frame buffer
 * @param[out] frame_len Actual frame length
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_generate_adcs(TlmFrame_t *frame, size_t *frame_len);

/**
 * @brief Generate beacon frame
 *
 * @param[out] frame Output frame buffer
 * @param[out] frame_len Actual frame length
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_generate_beacon(TlmFrame_t *frame, size_t *frame_len);

/**
 * @brief Check if telemetry is due
 *
 * @return true if telemetry should be generated
 */
bool tlm_is_due(void);

/**
 * @brief Mark telemetry as transmitted
 *
 * @param[in] success true if transmission succeeded
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_mark_transmitted(bool success);

/**
 * @brief Get telemetry statistics
 *
 * @param[out] stats Statistics structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_get_stats(TlmStats_t *stats);

/**
 * @brief Serialize telemetry frame to bytes
 *
 * @param[in] frame Frame to serialize
 * @param[in] payload_len Payload length
 * @param[out] buffer Output buffer
 * @param[in] buffer_size Buffer size
 * @param[out] bytes_written Actual bytes written
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t tlm_serialize(const TlmFrame_t *frame,
                                size_t payload_len,
                                uint8_t *buffer,
                                size_t buffer_size,
                                size_t *bytes_written);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_TELEMETRY_H */
