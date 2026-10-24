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

    /* system_state.h, telemetry.h, beacon.h, watchdog_mgr.h, then
     * adcs_control.h (its AdcsState_t is not system_state.h's, so those are
     * checked in test_rust_contract_adcs.c) */
    RUST_LAYOUT_THERMAL_STATE = 200,
    RUST_LAYOUT_COMM_STATE,
    RUST_LAYOUT_MISSION_STATE,
    RUST_LAYOUT_SYSTEM_STATE,
    RUST_LAYOUT_TLM_FRAME,
    RUST_LAYOUT_TLM_STATS,
    RUST_LAYOUT_BEACON_TELEMETRY,
    RUST_LAYOUT_BEACON_MEASUREMENTS,
    RUST_LAYOUT_BEACON_CONTENT,
    RUST_LAYOUT_AX25_FRAME,
    RUST_LAYOUT_BEACON_STATE,
    RUST_LAYOUT_EPS_CONTROL_STATE,
    RUST_LAYOUT_WDT_TASK_ENTRY,
    RUST_LAYOUT_WDT_STATS,
    RUST_LAYOUT_ADCS_VEC3,
    RUST_LAYOUT_ADCS_MAG_DATA,
    RUST_LAYOUT_ADCS_SUN_SENSOR_DATA,
    RUST_LAYOUT_ADCS_MTQ_COMMAND,
    RUST_LAYOUT_ADCS_CONTROL_STATE,
    RUST_LAYOUT_ADCS_TELEMETRY,
    RUST_LAYOUT_END_200           /**< one past the last of 200-249 */
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

    /* telemetry.h, beacon.h, watchdog_mgr.h, then adcs_control.h
     * (test_rust_contract_adcs.c) */
    RUST_CONST_TLM_MAX_FRAME_SIZE = 200, /**< TLM_MAX_FRAME_SIZE */
    RUST_CONST_TLM_SYNC_WORD,     /**< TLM_SYNC_WORD */
    RUST_CONST_TLM_DEFAULT_RATE,  /**< TLM_DEFAULT_RATE_MS */
    RUST_CONST_TLM_MIN_RATE,      /**< TLM_MIN_RATE_MS */
    RUST_CONST_TLM_MAX_RATE,      /**< TLM_MAX_RATE_MS */
    RUST_CONST_TLM_HEADER_SIZE,   /**< sizeof(TlmHeader_t) */
    RUST_CONST_TLM_HK_SIZE,       /**< sizeof(TlmHousekeeping_t) */
    RUST_CONST_TLM_EPS_SIZE,      /**< sizeof(TlmEps_t) */
    RUST_CONST_TLM_ADCS_SIZE,     /**< sizeof(TlmAdcs_t) */
    RUST_CONST_BEACON_TEXT_LEN,   /**< BEACON_MAX_TEXT_LEN */
    RUST_CONST_BEACON_PAYLOAD_LEN, /**< BEACON_MAX_PAYLOAD_LEN */
    RUST_CONST_BEACON_ACTIVE_MS,  /**< BEACON_INTERVAL_ACTIVE_MS */
    RUST_CONST_BEACON_IDLE_MS,    /**< BEACON_INTERVAL_IDLE_MS */
    RUST_CONST_BEACON_SAFE_MS,    /**< BEACON_INTERVAL_SAFE_MS */
    RUST_CONST_BEACON_MEAS_AGE,   /**< BEACON_MEAS_MAX_AGE_MS */
    RUST_CONST_BEACON_TLM_SOC,    /**< BEACON_TLM_SOC */
    RUST_CONST_BEACON_TLM_BV,     /**< BEACON_TLM_BV */
    RUST_CONST_BEACON_TLM_SUN,    /**< BEACON_TLM_SUN */
    RUST_CONST_BEACON_TLM_RF,     /**< BEACON_TLM_RF */
    RUST_CONST_BEACON_TLM_QSO,    /**< BEACON_TLM_QSO */
    RUST_CONST_WDT_MAX_TASKS,     /**< WDT_MAX_TASKS */
    RUST_CONST_WDT_NAME_LEN,      /**< WDT_TASK_NAME_LEN */
    RUST_CONST_WDT_HW_TIMEOUT,    /**< WDT_HW_TIMEOUT_MS */
    RUST_CONST_WDT_CHECK_INTERVAL, /**< WDT_TASK_CHECK_INTERVAL_MS */
    RUST_CONST_WDT_MISS_THRESHOLD, /**< WDT_TASK_MISS_THRESHOLD */
    RUST_CONST_ADCS_MAX_DIPOLE,   /**< ADCS_MAX_DIPOLE */
    RUST_CONST_ADCS_BDOT_GAIN,    /**< ADCS_BDOT_GAIN */
    RUST_CONST_ADCS_SUNPOINT_KP,  /**< ADCS_SUNPOINT_KP */
    RUST_CONST_ADCS_RATE_THRESHOLD, /**< ADCS_DETUMBLE_RATE_THRESHOLD */
    RUST_CONST_ADCS_SETTLING_MS,  /**< ADCS_DETUMBLE_SETTLING_MS */
    RUST_CONST_ADCS_TIMEOUT_MS,   /**< ADCS_DETUMBLE_TIMEOUT_MS */
    RUST_CONST_ADCS_SUN_SENSORS,  /**< ADCS_SUN_SENSOR_COUNT */
    RUST_CONST_ADCS_MODE_SAFE,    /**< ADCS_MODE_SAFE */
    RUST_CONST_FAULT_TYPE_ADCS,   /**< FAULT_TYPE_ADCS */
    RUST_CONST_END_200            /**< one past the last of 200-249 */
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
