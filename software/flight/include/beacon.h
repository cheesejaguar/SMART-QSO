/**
 * @file beacon.h
 * @brief SMART-QSO Beacon Generation Module
 *
 * Manages beacon content generation, scheduling, and AX.25 frame formatting.
 *
 * @requirement SRS-F020 Generate beacon at configurable intervals
 * @requirement SRS-F021 Include telemetry in beacon
 * @requirement SRS-F022 Support AI-generated beacon text
 * @requirement SRS-F023 Fallback to template text if AI unavailable
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 * @note Following GeneSat/PharmaSat/O/OREOS/EcAMSat beacon patterns
 */

#ifndef SMART_QSO_BEACON_H
#define SMART_QSO_BEACON_H

#ifdef __cplusplus
extern "C" {
#endif

#include "smart_qso.h"

/*===========================================================================*/
/* Beacon Configuration Constants                                             */
/*===========================================================================*/

/** Maximum beacon text length (AI-generated portion) */
#define BEACON_MAX_TEXT_LEN      200

/** Maximum total beacon payload length */
#define BEACON_MAX_PAYLOAD_LEN   256

/** Callsign for beacon identification */
#define BEACON_CALLSIGN          "SQSO-1"

/** Default beacon interval in ACTIVE mode (ms) */
#define BEACON_INTERVAL_ACTIVE_MS   30000

/** Default beacon interval in IDLE mode (ms) */
#define BEACON_INTERVAL_IDLE_MS     60000

/** Default beacon interval in SAFE mode (ms) */
#define BEACON_INTERVAL_SAFE_MS     180000

/** AX.25 frame constants */
#define AX25_FLAG               0x7E
#define AX25_CTRL_UI            0x03
#define AX25_PID_NO_L3          0xF0
#define AX25_ADDR_LEN           7
#define AX25_MAX_FRAME_LEN      330

/** CRC-16 CCITT constants */
#define CRC16_INIT              0xFFFF
#define CRC16_POLY              0x8408

/*===========================================================================*/
/* Beacon Types                                                               */
/*===========================================================================*/

/**
 * @brief Beacon source enumeration
 */
typedef enum {
    BEACON_SOURCE_TEMPLATE = 0,  /**< Fallback template text */
    BEACON_SOURCE_AI       = 1,  /**< AI-generated text from Jetson */
    BEACON_SOURCE_CUSTOM   = 2   /**< Custom text (for testing) */
} BeaconSource_t;

/**
 * @brief Beacon telemetry structure
 */
typedef struct {
    uint32_t timestamp;           /**< Unix timestamp */
    uint8_t  power_mode;          /**< Current power mode */
    uint8_t  soc;                 /**< State of charge (%) */
    uint16_t battery_mv;          /**< Battery voltage (mV) */
    int8_t   temp_obc;            /**< OBC temperature (C) */
    int8_t   temp_payload;        /**< Payload temperature (C) */
    uint8_t  sunlit;              /**< Sunlit flag */
    uint8_t  rf_power;            /**< RF power level (0-2) */
    uint16_t qso_count;           /**< Total QSO count */
    uint16_t orbit_number;        /**< Current orbit number */
    uint8_t  fault_flags;         /**< Fault status bitmask */
} BeaconTelemetry_t;

/**
 * @brief Beacon content structure
 */
typedef struct {
    char          text[BEACON_MAX_TEXT_LEN];     /**< AI/template text */
    size_t        text_len;                       /**< Text length */
    BeaconSource_t source;                        /**< Text source */
    BeaconTelemetry_t telemetry;                 /**< Telemetry data */
    uint32_t      sequence;                       /**< Beacon sequence number */
} BeaconContent_t;

/**
 * @brief AX.25 frame structure
 */
typedef struct {
    uint8_t  dest_addr[AX25_ADDR_LEN];   /**< Destination address */
    uint8_t  src_addr[AX25_ADDR_LEN];    /**< Source address */
    uint8_t  ctrl;                        /**< Control field */
    uint8_t  pid;                         /**< Protocol ID */
    uint8_t  info[BEACON_MAX_PAYLOAD_LEN];/**< Information field */
    size_t   info_len;                    /**< Information length */
    uint16_t fcs;                         /**< Frame check sequence */
} Ax25Frame_t;

/**
 * @brief Beacon state structure
 */
typedef struct {
    uint32_t last_beacon_ms;      /**< Last beacon transmission time */
    uint32_t interval_ms;         /**< Current beacon interval */
    uint32_t sequence;            /**< Beacon sequence counter */
    uint32_t ai_beacon_count;     /**< Count of AI-generated beacons */
    uint32_t template_count;      /**< Count of template beacons */
    uint32_t total_bytes_tx;      /**< Total bytes transmitted */
    bool     ai_available;        /**< AI text generation available */
    char     last_ai_text[BEACON_MAX_TEXT_LEN]; /**< Last AI text */
} BeaconState_t;

/*===========================================================================*/
/* Beacon Template Messages                                                   */
/*===========================================================================*/

/**
 * @brief Fallback template messages for when AI is unavailable
 */
extern const char *beacon_templates[];
extern const size_t beacon_template_count;

/*===========================================================================*/
/* Beacon Functions                                                           */
/*===========================================================================*/

/**
 * @brief Initialize beacon module
 *
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_init(void);

/**
 * @brief Check if beacon should be transmitted
 *
 * @param now_ms Current time in milliseconds
 * @return true if beacon should be transmitted
 */
bool beacon_should_transmit(uint64_t now_ms);

/**
 * @brief Update beacon interval based on power mode
 *
 * @param power_mode Current power mode
 */
void beacon_update_interval(PowerMode_t power_mode);

/**
 * @brief Generate beacon content
 *
 * @param content Pointer to beacon content structure
 * @param ai_text AI-generated text (NULL for template)
 * @param ai_text_len Length of AI text
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_generate_content(BeaconContent_t *content,
                                          const char *ai_text,
                                          size_t ai_text_len);

/**
 * @brief Format beacon payload string
 *
 * @param content Beacon content
 * @param buffer Output buffer
 * @param buffer_len Buffer length
 * @return Number of bytes written, 0 on error
 */
size_t beacon_format_payload(const BeaconContent_t *content,
                             char *buffer,
                             size_t buffer_len);

/**
 * @brief Build AX.25 frame from beacon content
 *
 * @param content Beacon content
 * @param frame Output AX.25 frame
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_build_ax25_frame(const BeaconContent_t *content,
                                          Ax25Frame_t *frame);

/**
 * @brief Serialize AX.25 frame to byte array
 *
 * @param frame AX.25 frame structure
 * @param buffer Output buffer
 * @param buffer_len Buffer length
 * @return Number of bytes written, 0 on error
 */
size_t beacon_serialize_ax25(const Ax25Frame_t *frame,
                             uint8_t *buffer,
                             size_t buffer_len);

/**
 * @brief Transmit beacon
 *
 * @param content Beacon content to transmit
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_transmit(const BeaconContent_t *content);

/**
 * @brief Mark beacon as transmitted
 *
 * @param now_ms Current time in milliseconds
 */
void beacon_mark_transmitted(uint64_t now_ms);

/**
 * @brief Get beacon state
 *
 * @param state Pointer to state structure
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_get_state(BeaconState_t *state);

/**
 * @brief Set AI availability flag
 *
 * @param available true if AI text generation is available
 */
void beacon_set_ai_available(bool available);

/**
 * @brief Store AI-generated text for next beacon
 *
 * @param text AI-generated text
 * @param len Text length
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_store_ai_text(const char *text, size_t len);

/**
 * @brief Get next template message (cycles through templates)
 *
 * @return Pointer to template message
 */
const char *beacon_get_template(void);

/**
 * @brief Calculate CRC-16 CCITT
 *
 * @param data Data buffer
 * @param len Data length
 * @return CRC-16 value
 */
uint16_t beacon_crc16(const uint8_t *data, size_t len);

/**
 * @brief Validate beacon text (ASCII only, length check)
 *
 * @param text Text to validate
 * @param len Text length
 * @return true if text is valid
 */
bool beacon_validate_text(const char *text, size_t len);

/*===========================================================================*/
/* Beacon Statistics                                                          */
/*===========================================================================*/

/**
 * @brief Get total beacon count
 *
 * @return Total beacons transmitted
 */
uint32_t beacon_get_total_count(void);

/**
 * @brief Get AI beacon percentage
 *
 * @return Percentage of beacons using AI text (0-100)
 */
uint8_t beacon_get_ai_percentage(void);

/**
 * @brief Reset beacon statistics
 */
void beacon_reset_stats(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_BEACON_H */
