/**
 * @file rust_contract.h
 * @brief What the Rust modules assume about the C headers, for checking
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * The Rust library (software/rust/sqso-ffi) mirrors C structs with
 * #[repr(C)] types and keeps its own copies of a few constants.
 * tests/test_rust_contract.c compares each with the C definition, so a
 * header change that the Rust side does not follow fails the build's tests.
 */

#ifndef SMART_QSO_RUST_CONTRACT_H
#define SMART_QSO_RUST_CONTRACT_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/** C types the Rust library mirrors */
typedef enum {
    RUST_LAYOUT_AX25_UI_VIEW = 0,
    RUST_LAYOUT_JETSON_MSG,
    RUST_LAYOUT_JETSON_RX,
    RUST_LAYOUT_COMMAND_VALIDATION,
    RUST_LAYOUT_VECTOR3,
    RUST_LAYOUT_CMD_DICT_ENTRY,
    RUST_LAYOUT_CMD_AUTH_VERIFIED,
    RUST_LAYOUT_CMD_AUTH_INFO,
    RUST_LAYOUT_COMMAND,
    RUST_LAYOUT_CMD_RESPONSE,
    RUST_LAYOUT_CMD_STATS,
    RUST_LAYOUT_UPLINK_STATS,
    RUST_LAYOUT_SHA256_CTX,
    RUST_LAYOUT_POWER_STATE,
    RUST_LAYOUT_ADCS_STATE,
    RUST_LAYOUT_SM_CONTEXT,
    RUST_LAYOUT_SM_TRANSITION,
    RUST_LAYOUT_TASK_CONFIG,
    RUST_LAYOUT_TASK_STATS,
    RUST_LAYOUT_SCHED_STATS,
    RUST_LAYOUT_FAULT_LOG_ENTRY,
    RUST_LAYOUT_COUNT,

    /* eps_control.h, sensors.h, mission_data.h, gnss.h (ids 150-199) */
    RUST_LAYOUT_EPS_CONTROL_STATE = 150,
    RUST_LAYOUT_SENSOR,
    RUST_LAYOUT_MISSION_DATA,
    RUST_LAYOUT_GNSS_FIX,
    RUST_LAYOUT_GNSS_RX
} RustLayout_t;

/** Constants the Rust library keeps */
typedef enum {
    RUST_CONST_SOC_PAYLOAD = 0,   /**< EPS_SOC_PAYLOAD_THRESHOLD */
    RUST_CONST_SOC_SAFE,          /**< EPS_SOC_SAFE_THRESHOLD */
    RUST_CONST_SOC_IDLE,          /**< EPS_SOC_IDLE_THRESHOLD */
    RUST_CONST_SOC_EMERGENCY,     /**< EPS_SOC_EMERGENCY_THRESHOLD */
    RUST_CONST_JETSON_LINE_MAX,   /**< JETSON_LINE_MAX */
    RUST_CONST_JETSON_TEXT_MAX,   /**< JETSON_TEXT_MAX */
    RUST_CONST_FILTER_MAX_LEN,    /**< BEACON_FILTER_MAX_LEN */
    RUST_CONST_FILTER_MAX_WORD,   /**< BEACON_FILTER_MAX_WORD */
    RUST_CONST_AUTH_KEY_SIZE,     /**< CMD_AUTH_KEY_SIZE */
    RUST_CONST_AUTH_TAG_SIZE,     /**< CMD_AUTH_TAG_SIZE */
    RUST_CONST_AUTH_MAX_ARGS,     /**< CMD_AUTH_MAX_ARGS */
    RUST_CONST_AUTH_OP_SLOTS,     /**< CMD_AUTH_OP_SLOTS */
    RUST_CONST_NVM_SLOT_SIZE,     /**< NVM_RECORD_SLOT_SIZE */
    RUST_CONST_NVM_FOOTPRINT,     /**< NVM_RECORD_FOOTPRINT */
    RUST_CONST_GSE_FRAME_SIZE,    /**< GSE_FRAME_SIZE */
    RUST_CONST_AX25_MAX_FRAME,    /**< AX25_MAX_FRAME_LEN */
    RUST_CONST_FAULT_ENTRIES,     /**< SMART_QSO_MAX_FAULT_ENTRIES */
    RUST_CONST_FAULT_DESC_LEN,    /**< SMART_QSO_FAULT_DESC_LEN */
    RUST_CONST_SCHED_MAX_TASKS,   /**< SCHED_MAX_TASKS */
    RUST_CONST_SCHED_NAME_LEN,    /**< SCHED_MAX_TASK_NAME */
    RUST_CONST_STATE_COUNT,       /**< STATE_COUNT */
    RUST_CONST_EVENT_COUNT,       /**< EVENT_COUNT */
    RUST_CONST_SSID,              /**< SQSO_SSID */
    RUST_CONST_COUNT,

    /* eps_control.h, sensors.h, mission_data.h, gnss.h (ids 150-199) */
    RUST_CONST_EPS_LIMIT_SAFE = 150,  /**< EPS_POWER_LIMIT_SAFE_W */
    RUST_CONST_EPS_LIMIT_IDLE,        /**< EPS_POWER_LIMIT_IDLE_W */
    RUST_CONST_EPS_LIMIT_ACTIVE,      /**< EPS_POWER_LIMIT_ACTIVE_W */
    RUST_CONST_EPS_BUS_MIN,           /**< EPS_BUS_MIN_V */
    RUST_CONST_EPS_BUS_MAX,           /**< EPS_BUS_MAX_V */
    RUST_CONST_MAX_SENSORS,           /**< SMART_QSO_MAX_SENSORS */
    RUST_CONST_SENSOR_ID_LEN,         /**< SMART_QSO_SENSOR_ID_LEN */
    RUST_CONST_SENSOR_NAME_LEN,       /**< SMART_QSO_SENSOR_NAME_LEN */
    RUST_CONST_SENSOR_UNITS_LEN,      /**< SMART_QSO_SENSOR_UNITS_LEN */
    RUST_CONST_SENSOR_TYPE_LEN,       /**< SMART_QSO_SENSOR_TYPE_LEN */
    RUST_CONST_SENSOR_CHANNEL_LEN,    /**< SMART_QSO_SENSOR_CHANNEL_LEN */
    RUST_CONST_MISSION_FAULT_LEN,     /**< sizeof(MissionData_t.last_fault) */
    RUST_CONST_MISSION_PHASE_EOL,     /**< MISSION_PHASE_EOL */
    RUST_CONST_GNSS_LINE_MAX,         /**< GNSS_LINE_MAX */
    RUST_CONST_GNSS_GGA_MAX_AGE,      /**< GNSS_GGA_MAX_AGE_MS */
    RUST_CONST_GNSS_LEO_MIN_RADIUS,   /**< GNSS_LEO_MIN_RADIUS_M */
    RUST_CONST_GNSS_LEO_MAX_RADIUS,   /**< GNSS_LEO_MAX_RADIUS_M */
    RUST_CONST_GNSS_MIN_SATS          /**< GNSS_MIN_SATS */
} RustConst_t;

/**
 * @brief Size and alignment of the Rust mirror of a C type
 * @return false for an unknown id
 */
bool sqso_rust_layout(RustLayout_t which, size_t *size, size_t *align);

/** @brief A Rust constant (NaN for an unknown id) */
double sqso_rust_constant(RustConst_t which);

/** @brief The station call sign the Rust modules use (SQSO_CALLSIGN) */
const char *sqso_rust_callsign(void);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_RUST_CONTRACT_H */
