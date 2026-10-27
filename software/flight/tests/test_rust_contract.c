/**
 * @file test_rust_contract.c
 * @brief The Rust modules' view of the C headers matches the headers
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Every C struct the Rust library mirrors has the same size and alignment
 * on both sides, and every constant it keeps has the header's value
 * (include/rust_contract.h). A header change the Rust side has not
 * followed fails here.
 */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <cmocka.h>

#include "rust_contract.h"
#include "ax25.h"
#include "beacon_filter.h"
#include "cmd_auth.h"
#include "cmd_dictionary.h"
#include "cmd_handler.h"
#include "contact_monitor.h"
#include "deployment.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "flight_log.h"
#include "gnss.h"
#include "gse_provision.h"
#include "input_validation.h"
#include "jetson_link.h"
#include "mission_data.h"
#include "nvm_record.h"
#include "payload_guard.h"
#include "rf_control.h"
#include "rf_silence.h"
#include "scheduler.h"
#include "sensors.h"
#include "sha256.h"
#include "state_machine.h"
#include "station_id.h"
#include "system_state.h"
#include "uplink.h"
#include "hal/hal_flash.h"
#include "hal/hal_gpio.h"
#include "hal/hal_reset.h"
#include "assert_handler.h"
#include "flight_log.h"
#include "uart_comm.h"
#include "hal/hal_uart.h"
#include "beacon.h"
#include "telemetry.h"
#include "watchdog_mgr.h"

typedef struct {
    RustLayout_t id;
    const char *name;
    size_t size;
    size_t align;
} Layout_t;

#define LAYOUT(id, type) { id, #type, sizeof(type), _Alignof(type) }

static const Layout_t k_layouts[] = {
    LAYOUT(RUST_LAYOUT_AX25_UI_VIEW, Ax25UiView_t),
    LAYOUT(RUST_LAYOUT_JETSON_MSG, JetsonMsg_t),
    LAYOUT(RUST_LAYOUT_JETSON_RX, JetsonRx_t),
    LAYOUT(RUST_LAYOUT_COMMAND_VALIDATION, CommandValidation_t),
    LAYOUT(RUST_LAYOUT_VECTOR3, Vector3_t),
    LAYOUT(RUST_LAYOUT_CMD_DICT_ENTRY, CmdDictEntry_t),
    LAYOUT(RUST_LAYOUT_CMD_AUTH_VERIFIED, CmdAuthVerified_t),
    LAYOUT(RUST_LAYOUT_CMD_AUTH_INFO, CmdAuthInfo_t),
    LAYOUT(RUST_LAYOUT_COMMAND, Command_t),
    LAYOUT(RUST_LAYOUT_CMD_RESPONSE, CmdResponse_t),
    LAYOUT(RUST_LAYOUT_CMD_STATS, CmdStats_t),
    LAYOUT(RUST_LAYOUT_UPLINK_STATS, UplinkStats_t),
    LAYOUT(RUST_LAYOUT_SHA256_CTX, Sha256Ctx_t),
    LAYOUT(RUST_LAYOUT_POWER_STATE, PowerState_t),
    LAYOUT(RUST_LAYOUT_ADCS_STATE, AdcsState_t),
    LAYOUT(RUST_LAYOUT_SM_CONTEXT, SmContext_t),
    LAYOUT(RUST_LAYOUT_SM_TRANSITION, SmTransition_t),
    LAYOUT(RUST_LAYOUT_TASK_CONFIG, task_config_t),
    LAYOUT(RUST_LAYOUT_TASK_STATS, task_stats_t),
    LAYOUT(RUST_LAYOUT_SCHED_STATS, sched_stats_t),
    LAYOUT(RUST_LAYOUT_FAULT_LOG_ENTRY, FaultLogEntry_t),
};

_Static_assert(sizeof(k_layouts) / sizeof(k_layouts[0]) == (size_t)RUST_LAYOUT_COUNT,
               "every mirrored type is listed");

/* eps_control.h, sensors.h, mission_data.h, gnss.h (ids 150-199) */
static const Layout_t k_layouts_power_sensors_gnss[] = {
    LAYOUT(RUST_LAYOUT_EPS_CONTROL_STATE, EpsControlState_t),
    LAYOUT(RUST_LAYOUT_SENSOR, Sensor_t),
    LAYOUT(RUST_LAYOUT_MISSION_DATA, MissionData_t),
    LAYOUT(RUST_LAYOUT_GNSS_FIX, GnssFix_t),
    LAYOUT(RUST_LAYOUT_GNSS_RX, GnssRx_t),
};

_Static_assert(sizeof(k_layouts_power_sensors_gnss) / sizeof(k_layouts_power_sensors_gnss[0]) ==
                   (size_t)(RUST_LAYOUT_GNSS_RX - RUST_LAYOUT_EPS_CONTROL_STATE + 1),
               "every mirrored type of ids 150-199 is listed");

static void check_layouts(const Layout_t *layouts, size_t count)
{
    for (size_t i = 0U; i < count; i++) {
        size_t size = 0U;
        size_t align = 0U;
        assert_true(sqso_rust_layout(layouts[i].id, &size, &align));
        if ((size != layouts[i].size) || (align != layouts[i].align)) {
            print_error("%s: C %zu/%zu, Rust %zu/%zu (size/align)\n", layouts[i].name,
                        layouts[i].size, layouts[i].align, size, align);
        }
        assert_int_equal(size, layouts[i].size);
        assert_int_equal(align, layouts[i].align);
    }
}

/** The structs of eps_control.h, sensors.h, mission_data.h and gnss.h, also
 *  at their field offsets the persisted records depend on */
static void test_power_sensor_gnss_layouts_match(void **state)
{
    (void)state;
    check_layouts(k_layouts_power_sensors_gnss,
                  sizeof(k_layouts_power_sensors_gnss) / sizeof(k_layouts_power_sensors_gnss[0]));
    assert_false(sqso_rust_layout((RustLayout_t)(RUST_LAYOUT_GNSS_RX + 1), NULL, NULL));

    /* The stored EPS and mission records are these byte layouts */
    assert_int_equal(offsetof(EpsControlState_t, power_mode), 4U);
    assert_int_equal(offsetof(EpsControlState_t, power_limit_w), 8U);
    assert_int_equal(offsetof(EpsControlState_t, last_control_ms), 16U);
    assert_int_equal(offsetof(EpsControlState_t, crc32), 20U);
    assert_int_equal(offsetof(MissionData_t, reset_count), 16U);
    assert_int_equal(offsetof(MissionData_t, last_reset_ms), 24U);
    assert_int_equal(offsetof(MissionData_t, total_energy_wh), 32U);
    assert_int_equal(offsetof(MissionData_t, fault_count), 40U);
    assert_int_equal(offsetof(MissionData_t, mission_phase), 44U);
    assert_int_equal(offsetof(MissionData_t, last_fault), 45U);
    assert_int_equal(offsetof(MissionData_t, crc32), 112U);
}

static void test_struct_layouts_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_layouts) / sizeof(k_layouts[0]); i++) {
        size_t size = 0U;
        size_t align = 0U;
        assert_true(sqso_rust_layout(k_layouts[i].id, &size, &align));
        if ((size != k_layouts[i].size) || (align != k_layouts[i].align)) {
            print_error("%s: C %zu/%zu, Rust %zu/%zu (size/align)\n", k_layouts[i].name,
                        k_layouts[i].size, k_layouts[i].align, size, align);
        }
        assert_int_equal(size, k_layouts[i].size);
        assert_int_equal(align, k_layouts[i].align);
    }
    assert_false(sqso_rust_layout(RUST_LAYOUT_COUNT, NULL, NULL));
}

static void test_enums_are_32_bit(void **state)
{
    (void)state;
    /* The Rust side passes C enums as 32-bit integers (the STM32L4 build
     * uses -fno-short-enums for this) */
    assert_int_equal(sizeof(SmartQsoResult_t), 4U);
    assert_int_equal(sizeof(SmState_t), 4U);
    assert_int_equal(sizeof(CmdResult_t), 4U);
    assert_int_equal(sizeof(HalFlashRegion_t), 4U);
}

typedef struct {
    RustConst_t id;
    const char *name;
    double value;
} Constant_t;

#define CONSTANT(id, value) { id, #value, (double)(value) }

static const Constant_t k_constants[] = {
    CONSTANT(RUST_CONST_SOC_PAYLOAD, EPS_SOC_PAYLOAD_THRESHOLD),
    CONSTANT(RUST_CONST_SOC_SAFE, EPS_SOC_SAFE_THRESHOLD),
    CONSTANT(RUST_CONST_SOC_IDLE, EPS_SOC_IDLE_THRESHOLD),
    CONSTANT(RUST_CONST_SOC_EMERGENCY, EPS_SOC_EMERGENCY_THRESHOLD),
    CONSTANT(RUST_CONST_JETSON_LINE_MAX, JETSON_LINE_MAX),
    CONSTANT(RUST_CONST_JETSON_TEXT_MAX, JETSON_TEXT_MAX),
    CONSTANT(RUST_CONST_FILTER_MAX_LEN, BEACON_FILTER_MAX_LEN),
    CONSTANT(RUST_CONST_FILTER_MAX_WORD, BEACON_FILTER_MAX_WORD),
    CONSTANT(RUST_CONST_AUTH_KEY_SIZE, CMD_AUTH_KEY_SIZE),
    CONSTANT(RUST_CONST_AUTH_TAG_SIZE, CMD_AUTH_TAG_SIZE),
    CONSTANT(RUST_CONST_AUTH_MAX_ARGS, CMD_AUTH_MAX_ARGS),
    CONSTANT(RUST_CONST_AUTH_OP_SLOTS, CMD_AUTH_OP_SLOTS),
    CONSTANT(RUST_CONST_NVM_SLOT_SIZE, NVM_RECORD_SLOT_SIZE),
    CONSTANT(RUST_CONST_NVM_FOOTPRINT, NVM_RECORD_FOOTPRINT),
    CONSTANT(RUST_CONST_GSE_FRAME_SIZE, GSE_FRAME_SIZE),
    CONSTANT(RUST_CONST_AX25_MAX_FRAME, AX25_MAX_FRAME_LEN),
    CONSTANT(RUST_CONST_FAULT_ENTRIES, SMART_QSO_MAX_FAULT_ENTRIES),
    CONSTANT(RUST_CONST_FAULT_DESC_LEN, SMART_QSO_FAULT_DESC_LEN),
    CONSTANT(RUST_CONST_SCHED_MAX_TASKS, SCHED_MAX_TASKS),
    CONSTANT(RUST_CONST_SCHED_NAME_LEN, SCHED_MAX_TASK_NAME),
    CONSTANT(RUST_CONST_STATE_COUNT, STATE_COUNT),
    CONSTANT(RUST_CONST_EVENT_COUNT, EVENT_COUNT),
    CONSTANT(RUST_CONST_SSID, SQSO_SSID),
};

_Static_assert(sizeof(k_constants) / sizeof(k_constants[0]) == (size_t)RUST_CONST_COUNT,
               "every kept constant is listed");

/* eps_control.h, sensors.h, mission_data.h, gnss.h (ids 150-199) */
static const Constant_t k_constants_power_sensors_gnss[] = {
    CONSTANT(RUST_CONST_EPS_LIMIT_SAFE, EPS_POWER_LIMIT_SAFE_W),
    CONSTANT(RUST_CONST_EPS_LIMIT_IDLE, EPS_POWER_LIMIT_IDLE_W),
    CONSTANT(RUST_CONST_EPS_LIMIT_ACTIVE, EPS_POWER_LIMIT_ACTIVE_W),
    CONSTANT(RUST_CONST_EPS_BUS_MIN, EPS_BUS_MIN_V),
    CONSTANT(RUST_CONST_EPS_BUS_MAX, EPS_BUS_MAX_V),
    CONSTANT(RUST_CONST_MAX_SENSORS, SMART_QSO_MAX_SENSORS),
    CONSTANT(RUST_CONST_SENSOR_ID_LEN, SMART_QSO_SENSOR_ID_LEN),
    CONSTANT(RUST_CONST_SENSOR_NAME_LEN, SMART_QSO_SENSOR_NAME_LEN),
    CONSTANT(RUST_CONST_SENSOR_UNITS_LEN, SMART_QSO_SENSOR_UNITS_LEN),
    CONSTANT(RUST_CONST_SENSOR_TYPE_LEN, SMART_QSO_SENSOR_TYPE_LEN),
    CONSTANT(RUST_CONST_SENSOR_CHANNEL_LEN, SMART_QSO_SENSOR_CHANNEL_LEN),
    CONSTANT(RUST_CONST_MISSION_FAULT_LEN, SMART_QSO_FAULT_DESC_LEN),
    CONSTANT(RUST_CONST_MISSION_PHASE_EOL, MISSION_PHASE_EOL),
    CONSTANT(RUST_CONST_GNSS_LINE_MAX, GNSS_LINE_MAX),
    CONSTANT(RUST_CONST_GNSS_GGA_MAX_AGE, GNSS_GGA_MAX_AGE_MS),
    CONSTANT(RUST_CONST_GNSS_LEO_MIN_RADIUS, GNSS_LEO_MIN_RADIUS_M),
    CONSTANT(RUST_CONST_GNSS_LEO_MAX_RADIUS, GNSS_LEO_MAX_RADIUS_M),
    CONSTANT(RUST_CONST_GNSS_MIN_SATS, GNSS_MIN_SATS),
};

_Static_assert(sizeof(k_constants_power_sensors_gnss) / sizeof(k_constants_power_sensors_gnss[0]) ==
                   (size_t)(RUST_CONST_GNSS_MIN_SATS - RUST_CONST_EPS_LIMIT_SAFE + 1),
               "every kept constant of ids 150-199 is listed");

static void test_power_sensor_gnss_constants_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < (sizeof(k_constants_power_sensors_gnss) / sizeof(k_constants_power_sensors_gnss[0]));
         i++) {
        double rust = sqso_rust_constant(k_constants_power_sensors_gnss[i].id);
        bool same = memcmp(&rust, &k_constants_power_sensors_gnss[i].value, sizeof(rust)) == 0;
        if (!same) {
            print_error("%s: C %g, Rust %g\n", k_constants_power_sensors_gnss[i].name,
                        k_constants_power_sensors_gnss[i].value, rust);
        }
        assert_true(same);
    }
    assert_true(isnan(sqso_rust_constant((RustConst_t)(RUST_CONST_GNSS_MIN_SATS + 1))));
}

static void test_constants_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_constants) / sizeof(k_constants[0]); i++) {
        double rust = sqso_rust_constant(k_constants[i].id);
        /* Exactly the same value: compare the representations */
        bool same = memcmp(&rust, &k_constants[i].value, sizeof(rust)) == 0;
        if (!same) {
            print_error("%s: C %g, Rust %g\n", k_constants[i].name, k_constants[i].value, rust);
        }
        assert_true(same);
    }
    assert_true(isnan(sqso_rust_constant(RUST_CONST_COUNT)));
}

/* 100-149: RF control, RF silence, contact monitor, deployment, payload guard */

static const Layout_t k_rf_layouts[] = {
    LAYOUT(RUST_LAYOUT_PAYLOAD_SAMPLE, PayloadSample_t),
    LAYOUT(RUST_LAYOUT_DEPLOY_ELEMENT_STATUS, DeploymentElementStatus_t),
    LAYOUT(RUST_LAYOUT_DEPLOY_TELEMETRY, DeploymentTelemetry_t),
};

_Static_assert(sizeof(k_rf_layouts) / sizeof(k_rf_layouts[0]) ==
               (size_t)(RUST_LAYOUT_RF_GROUP_END - RUST_LAYOUT_PAYLOAD_SAMPLE),
               "every mirrored RF-group type is listed");

static const Constant_t k_rf_constants[] = {
    CONSTANT(RUST_CONST_PG_MAX_INPUT_W, PG_MAX_INPUT_W),
    CONSTANT(RUST_CONST_PG_LATCHUP_INPUT_W, PG_LATCHUP_INPUT_W),
    CONSTANT(RUST_CONST_PG_OVERPOWER_SAMPLES, PG_OVERPOWER_SAMPLES),
    CONSTANT(RUST_CONST_PG_SPREADER_MAX_C, PG_SPREADER_MAX_C),
    CONSTANT(RUST_CONST_PG_JUNCTION_MAX_C, PG_JUNCTION_MAX_C),
    CONSTANT(RUST_CONST_PG_RESTART_MARGIN_C, PG_RESTART_MARGIN_C),
    CONSTANT(RUST_CONST_PG_COOLDOWN_MS, PG_COOLDOWN_MS),
    CONSTANT(RUST_CONST_PG_TELEMETRY_TIMEOUT_MS, PG_TELEMETRY_TIMEOUT_MS),
    CONSTANT(RUST_CONST_PG_COOLDOWN, PG_COOLDOWN),
    CONSTANT(RUST_CONST_RF_POWER_LEVEL_COUNT, RF_POWER_LEVEL_COUNT),
    CONSTANT(RUST_CONST_RF_INHIBIT_CONTACT, RF_INHIBIT_CONTACT),
    CONSTANT(RUST_CONST_RF_INHIBIT_RF_SILENCE, RF_INHIBIT_RF_SILENCE),
    CONSTANT(RUST_CONST_RF_SILENCE_DURATION_S, RF_SILENCE_DURATION_S),
    CONSTANT(RUST_CONST_RF_SILENCE_PERSIST_S, RF_SILENCE_PERSIST_INTERVAL_S),
    CONSTANT(RUST_CONST_RF_SILENCE_ARMED, RF_SILENCE_ARMED),
    CONSTANT(RUST_CONST_RF_SILENCE_COMPLETE, RF_SILENCE_COMPLETE),
    CONSTANT(RUST_CONST_CONTACT_DEFAULT_DAYS, CONTACT_TIMEOUT_DEFAULT_DAYS),
    CONSTANT(RUST_CONST_CONTACT_MIN_DAYS, CONTACT_TIMEOUT_MIN_DAYS),
    CONSTANT(RUST_CONST_CONTACT_MAX_DAYS, CONTACT_TIMEOUT_MAX_DAYS),
    CONSTANT(RUST_CONST_CONTACT_PERSIST_S, CONTACT_PERSIST_INTERVAL_S),
    CONSTANT(RUST_CONST_CONTACT_NVM_COUNTER, CONTACT_NVM_COUNTER),
    CONSTANT(RUST_CONST_CONTACT_NVM_TIMEOUT, CONTACT_NVM_TIMEOUT),
    CONSTANT(RUST_CONST_DEPLOY_WAIT_MS, DEPLOYMENT_WAIT_TIME_MS),
    CONSTANT(RUST_CONST_DEPLOY_ANTENNA_BURN_MS, ANTENNA_DEPLOY_BURN_TIME_MS),
    CONSTANT(RUST_CONST_DEPLOY_SOLAR_BURN_MS, SOLAR_DEPLOY_BURN_TIME_MS),
    CONSTANT(RUST_CONST_DEPLOY_MAX_RETRIES, DEPLOYMENT_MAX_RETRIES),
    CONSTANT(RUST_CONST_DEPLOY_RETRY_DELAY_MS, DEPLOYMENT_RETRY_DELAY_MS),
    CONSTANT(RUST_CONST_DEPLOY_PERSIST_S, DEPLOYMENT_PERSIST_INTERVAL_S),
    CONSTANT(RUST_CONST_DEPLOY_STATE_COUNT, DEPLOY_STATE_COUNT),
    CONSTANT(RUST_CONST_DEPLOY_STATE_ARMED, DEPLOY_STATE_ARMED),
    CONSTANT(RUST_CONST_DEPLOY_ELEMENT_COUNT, DEPLOY_ELEMENT_COUNT),
    CONSTANT(RUST_CONST_REGION_RF_CONTROL, FLASH_REGION_RF_CONTROL),
    CONSTANT(RUST_CONST_REGION_CONTACT, FLASH_REGION_CONTACT),
    CONSTANT(RUST_CONST_REGION_RF_SILENCE, FLASH_REGION_RF_SILENCE),
    CONSTANT(RUST_CONST_REGION_DEPLOYMENT, FLASH_REGION_DEPLOYMENT),
    CONSTANT(RUST_CONST_PIN_RADIO_EN, GPIO_PIN_RADIO_EN),
    CONSTANT(RUST_CONST_PIN_DEPLOY_EN, GPIO_PIN_DEPLOY_EN),
    CONSTANT(RUST_CONST_PIN_RF_TX_EN, GPIO_PIN_RF_TX_EN),
    CONSTANT(RUST_CONST_PIN_ANTENNA_BURN, GPIO_PIN_ANTENNA_BURN),
    CONSTANT(RUST_CONST_PIN_SOLAR_BURN, GPIO_PIN_SOLAR_BURN),
    CONSTANT(RUST_CONST_PIN_ANTENNA_SENSE, GPIO_PIN_ANTENNA_SENSE),
    CONSTANT(RUST_CONST_PIN_SOLAR_SENSE, GPIO_PIN_SOLAR_SENSE),
    CONSTANT(RUST_CONST_GPIO_DIR_INPUT, GPIO_DIR_INPUT),
    CONSTANT(RUST_CONST_GPIO_DIR_OUTPUT, GPIO_DIR_OUTPUT),
    CONSTANT(RUST_CONST_GPIO_PULL_DOWN, GPIO_PULL_DOWN),
    CONSTANT(RUST_CONST_RESET_WATCHDOG, HAL_RESET_WATCHDOG),
    CONSTANT(RUST_CONST_RESET_SOFTWARE, HAL_RESET_SOFTWARE),
    CONSTANT(RUST_CONST_FAULT_DEPLOYMENT, FAULT_TYPE_DEPLOYMENT),
    CONSTANT(RUST_CONST_LOG_LEVEL_DEBUG, LOG_LEVEL_DEBUG),
    CONSTANT(RUST_CONST_LOG_LEVEL_INFO, LOG_LEVEL_INFO),
};

_Static_assert(sizeof(k_rf_constants) / sizeof(k_rf_constants[0]) ==
               (size_t)(RUST_CONST_RF_GROUP_END - RUST_CONST_PG_MAX_INPUT_W),
               "every RF-group constant is listed");

static void test_rf_group_contract(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_rf_layouts) / sizeof(k_rf_layouts[0]); i++) {
        size_t size = 0U;
        size_t align = 0U;
        assert_true(sqso_rust_layout(k_rf_layouts[i].id, &size, &align));
        if ((size != k_rf_layouts[i].size) || (align != k_rf_layouts[i].align)) {
            print_error("%s: C %zu/%zu, Rust %zu/%zu (size/align)\n", k_rf_layouts[i].name,
                        k_rf_layouts[i].size, k_rf_layouts[i].align, size, align);
        }
        assert_int_equal(size, k_rf_layouts[i].size);
        assert_int_equal(align, k_rf_layouts[i].align);
    }
    assert_false(sqso_rust_layout(RUST_LAYOUT_RF_GROUP_END, NULL, NULL));

    for (size_t i = 0U; i < sizeof(k_rf_constants) / sizeof(k_rf_constants[0]); i++) {
        double rust = sqso_rust_constant(k_rf_constants[i].id);
        bool same = memcmp(&rust, &k_rf_constants[i].value, sizeof(rust)) == 0;
        if (!same) {
            print_error("%s: C %g, Rust %g\n", k_rf_constants[i].name, k_rf_constants[i].value, rust);
        }
        assert_true(same);
    }
    /* The group fills ids 100-149; 150 starts the next group */
    assert_int_equal(RUST_CONST_RF_GROUP_END, RUST_CONST_EPS_LIMIT_SAFE);

    /* Enums crossing the boundary as 32-bit integers */
    assert_int_equal(sizeof(RfPowerLevel_t), 4U);
    assert_int_equal(sizeof(RfInhibit_t), 4U);
    assert_int_equal(sizeof(RfSilenceState_t), 4U);
    assert_int_equal(sizeof(DeploymentState_t), 4U);
    assert_int_equal(sizeof(DeploymentElement_t), 4U);
    assert_int_equal(sizeof(PayloadGuardResult_t), 4U);
    assert_int_equal(sizeof(HalResetCause_t), 4U);
    assert_int_equal(sizeof(LogLevel_t), 4U);
}

static void test_station_identity_matches(void **state)
{
    (void)state;
    char text[32];
    assert_string_equal(sqso_rust_callsign(), SQSO_CALLSIGN);

    /* The content filter's own-call rule uses the same identity */
    (void)snprintf(text, sizeof(text), "73 de %s-%u", SQSO_CALLSIGN, (unsigned)SQSO_SSID);
    assert_int_equal(beacon_filter_check(text, strlen(text)), BEACON_FILTER_OK);
    (void)snprintf(text, sizeof(text), "73 de %s-%u", SQSO_CALLSIGN, (unsigned)SQSO_SSID - 1U);
    assert_int_equal(beacon_filter_check(text, strlen(text)), BEACON_FILTER_CALLSIGN);
}

static void test_dictionary_table_matches_header(void **state)
{
    (void)state;
    assert_int_equal(g_cmd_dictionary_count, CMD_DICTIONARY_COUNT);
    assert_int_equal(g_cmd_dictionary[0].cmd_id, CMD_SYS_NOP);
    assert_int_equal(g_cmd_dictionary[CMD_DICTIONARY_COUNT - 1U].cmd_id, CMD_GNSS_REQUEST);
    assert_string_equal(cmd_get_name(CMD_COMM_TX_DISABLE), "TX_DISABLE");
}

/* assert_handler.h, flight_log.h, uart_comm.h, hal_uart.h (ids 250-299) */
static const Layout_t k_layouts_250[] = {
    LAYOUT((RustLayout_t)RUST_LAYOUT_ASSERT_RECORD, AssertRecord_t),
    LAYOUT((RustLayout_t)RUST_LAYOUT_ASSERT_STATS, AssertStats_t),
    LAYOUT((RustLayout_t)RUST_LAYOUT_LOG_ENTRY, LogEntry_t),
    LAYOUT((RustLayout_t)RUST_LAYOUT_LOG_STATS, LogStats_t),
    LAYOUT((RustLayout_t)RUST_LAYOUT_UART_CONFIG, UartConfig_t),
    LAYOUT((RustLayout_t)RUST_LAYOUT_HAL_UART_CONFIG, HalUartConfig_t),
};

_Static_assert(sizeof(k_layouts_250) / sizeof(k_layouts_250[0]) ==
                   (size_t)(RUST_LAYOUT_GROUP_250_END - RUST_LAYOUT_ASSERT_RECORD),
               "every mirrored type of ids 250-299 is listed");

static const Constant_t k_constants_250[] = {
    CONSTANT((RustConst_t)RUST_CONST_ASSERT_MSG_LEN, ASSERT_MAX_MSG_LEN),
    CONSTANT((RustConst_t)RUST_CONST_ASSERT_LOG_SIZE, ASSERT_MAX_FAILURE_LOG),
    CONSTANT((RustConst_t)RUST_CONST_ASSERT_FILENAME_LEN, ASSERT_MAX_FILENAME_LEN),
    CONSTANT((RustConst_t)RUST_CONST_LOG_MESSAGE_LEN, LOG_MAX_MESSAGE_LEN),
    CONSTANT((RustConst_t)RUST_CONST_LOG_BUFFER_SIZE, LOG_BUFFER_SIZE),
    CONSTANT((RustConst_t)RUST_CONST_LOG_MODULE_LEN, LOG_MAX_MODULE_LEN),
    CONSTANT((RustConst_t)RUST_CONST_LOG_MIN_LEVEL, LOG_MIN_LEVEL),
    CONSTANT((RustConst_t)RUST_CONST_LOG_LEVEL_OFF, LOG_LEVEL_OFF),
    CONSTANT((RustConst_t)RUST_CONST_UART_DEVICE_LEN, UART_DEVICE_MAX_LEN),
    CONSTANT((RustConst_t)RUST_CONST_UART_BAUD_DEFAULT, UART_BAUDRATE_DEFAULT),
    CONSTANT((RustConst_t)RUST_CONST_TELEMETRY_INTERVAL, SMART_QSO_TELEMETRY_INTERVAL_MS),
    CONSTANT((RustConst_t)RUST_CONST_FAULT_TYPE_ASSERTION, FAULT_TYPE_ASSERTION),
    CONSTANT((RustConst_t)RUST_CONST_UART_DEVICE_DEFAULT, sizeof(UART_DEVICE_DEFAULT) - 1U),
};

_Static_assert(sizeof(k_constants_250) / sizeof(k_constants_250[0]) ==
                   (size_t)(RUST_CONST_GROUP_250_END - RUST_CONST_ASSERT_MSG_LEN),
               "every kept constant of ids 250-299 is listed");

static void test_layouts_and_constants_250(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_layouts_250) / sizeof(k_layouts_250[0]); i++) {
        size_t size = 0U;
        size_t align = 0U;
        assert_true(sqso_rust_layout(k_layouts_250[i].id, &size, &align));
        if ((size != k_layouts_250[i].size) || (align != k_layouts_250[i].align)) {
            print_error("%s: C %zu/%zu, Rust %zu/%zu (size/align)\n", k_layouts_250[i].name,
                        k_layouts_250[i].size, k_layouts_250[i].align, size, align);
        }
        assert_int_equal(size, k_layouts_250[i].size);
        assert_int_equal(align, k_layouts_250[i].align);
    }
    assert_false(sqso_rust_layout((RustLayout_t)RUST_LAYOUT_GROUP_250_END, NULL, NULL));

    for (size_t i = 0U; i < sizeof(k_constants_250) / sizeof(k_constants_250[0]); i++) {
        double rust = sqso_rust_constant(k_constants_250[i].id);
        bool same = memcmp(&rust, &k_constants_250[i].value, sizeof(rust)) == 0;
        if (!same) {
            print_error("%s: C %g, Rust %g\n", k_constants_250[i].name, k_constants_250[i].value, rust);
        }
        assert_true(same);
    }
    assert_true(isnan(sqso_rust_constant((RustConst_t)RUST_CONST_GROUP_250_END)));
}

/** The UART defaults before any initialization are the header's */
static void test_uart_defaults_match_header(void **state)
{
    (void)state;
    UartConfig_t config;
    assert_int_equal(uart_get_config(&config), SMART_QSO_OK);
    assert_string_equal(config.device, UART_DEVICE_DEFAULT);
    assert_int_equal(config.baudrate, UART_BAUDRATE_DEFAULT);
    assert_int_equal(uart_get_telemetry_interval(), SMART_QSO_TELEMETRY_INTERVAL_MS);
}

/** system_state.h, telemetry.h, beacon.h, watchdog_mgr.h (ids 200-249;
 *  adcs_control.h, whose AdcsState_t is not system_state.h's, is checked in
 *  test_rust_contract_adcs.c) */
static const Layout_t k_layouts_200[] = {
    LAYOUT(RUST_LAYOUT_THERMAL_STATE, ThermalState_t),
    LAYOUT(RUST_LAYOUT_COMM_STATE, CommState_t),
    LAYOUT(RUST_LAYOUT_MISSION_STATE, MissionState_t),
    LAYOUT(RUST_LAYOUT_SYSTEM_STATE, SystemState_t),
    LAYOUT(RUST_LAYOUT_TLM_FRAME, TlmFrame_t),
    LAYOUT(RUST_LAYOUT_TLM_STATS, TlmStats_t),
    LAYOUT(RUST_LAYOUT_BEACON_TELEMETRY, BeaconTelemetry_t),
    LAYOUT(RUST_LAYOUT_BEACON_MEASUREMENTS, BeaconMeasurements_t),
    LAYOUT(RUST_LAYOUT_BEACON_CONTENT, BeaconContent_t),
    LAYOUT(RUST_LAYOUT_AX25_FRAME, Ax25Frame_t),
    LAYOUT(RUST_LAYOUT_BEACON_STATE, BeaconState_t),
    LAYOUT(RUST_LAYOUT_BEACON_EPS_STATE, EpsControlState_t),
    LAYOUT(RUST_LAYOUT_WDT_TASK_ENTRY, WdtTaskEntry_t),
    LAYOUT(RUST_LAYOUT_WDT_STATS, WdtStats_t),
};

_Static_assert(sizeof(k_layouts_200) / sizeof(k_layouts_200[0]) ==
               (size_t)(RUST_LAYOUT_ADCS_VEC3 - RUST_LAYOUT_THERMAL_STATE),
               "every mirrored type of 200-249 before adcs_control.h is listed");

static void test_struct_layouts_200_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_layouts_200) / sizeof(k_layouts_200[0]); i++) {
        size_t size = 0U;
        size_t align = 0U;
        assert_true(sqso_rust_layout(k_layouts_200[i].id, &size, &align));
        if ((size != k_layouts_200[i].size) || (align != k_layouts_200[i].align)) {
            print_error("%s: C %zu/%zu, Rust %zu/%zu (size/align)\n", k_layouts_200[i].name,
                        k_layouts_200[i].size, k_layouts_200[i].align, size, align);
        }
        assert_int_equal(size, k_layouts_200[i].size);
        assert_int_equal(align, k_layouts_200[i].align);
    }
    assert_false(sqso_rust_layout(RUST_LAYOUT_END_200, NULL, NULL));
}

/** telemetry.h, beacon.h, watchdog_mgr.h (adcs_control.h: test_rust_contract_adcs.c) */
static const Constant_t k_constants_200[] = {
    CONSTANT(RUST_CONST_TLM_MAX_FRAME_SIZE, TLM_MAX_FRAME_SIZE),
    CONSTANT(RUST_CONST_TLM_SYNC_WORD, TLM_SYNC_WORD),
    CONSTANT(RUST_CONST_TLM_DEFAULT_RATE, TLM_DEFAULT_RATE_MS),
    CONSTANT(RUST_CONST_TLM_MIN_RATE, TLM_MIN_RATE_MS),
    CONSTANT(RUST_CONST_TLM_MAX_RATE, TLM_MAX_RATE_MS),
    CONSTANT(RUST_CONST_TLM_HEADER_SIZE, sizeof(TlmHeader_t)),
    CONSTANT(RUST_CONST_TLM_HK_SIZE, sizeof(TlmHousekeeping_t)),
    CONSTANT(RUST_CONST_TLM_EPS_SIZE, sizeof(TlmEps_t)),
    CONSTANT(RUST_CONST_TLM_ADCS_SIZE, sizeof(TlmAdcs_t)),
    CONSTANT(RUST_CONST_BEACON_TEXT_LEN, BEACON_MAX_TEXT_LEN),
    CONSTANT(RUST_CONST_BEACON_PAYLOAD_LEN, BEACON_MAX_PAYLOAD_LEN),
    CONSTANT(RUST_CONST_BEACON_ACTIVE_MS, BEACON_INTERVAL_ACTIVE_MS),
    CONSTANT(RUST_CONST_BEACON_IDLE_MS, BEACON_INTERVAL_IDLE_MS),
    CONSTANT(RUST_CONST_BEACON_SAFE_MS, BEACON_INTERVAL_SAFE_MS),
    CONSTANT(RUST_CONST_BEACON_MEAS_AGE, BEACON_MEAS_MAX_AGE_MS),
    CONSTANT(RUST_CONST_BEACON_TLM_SOC, BEACON_TLM_SOC),
    CONSTANT(RUST_CONST_BEACON_TLM_BV, BEACON_TLM_BV),
    CONSTANT(RUST_CONST_BEACON_TLM_SUN, BEACON_TLM_SUN),
    CONSTANT(RUST_CONST_BEACON_TLM_RF, BEACON_TLM_RF),
    CONSTANT(RUST_CONST_BEACON_TLM_QSO, BEACON_TLM_QSO),
    CONSTANT(RUST_CONST_WDT_MAX_TASKS, WDT_MAX_TASKS),
    CONSTANT(RUST_CONST_WDT_NAME_LEN, WDT_TASK_NAME_LEN),
    CONSTANT(RUST_CONST_WDT_HW_TIMEOUT, WDT_HW_TIMEOUT_MS),
    CONSTANT(RUST_CONST_WDT_CHECK_INTERVAL, WDT_TASK_CHECK_INTERVAL_MS),
    CONSTANT(RUST_CONST_WDT_MISS_THRESHOLD, WDT_TASK_MISS_THRESHOLD),
};

_Static_assert(sizeof(k_constants_200) / sizeof(k_constants_200[0]) ==
               (size_t)(RUST_CONST_ADCS_MAX_DIPOLE - RUST_CONST_TLM_MAX_FRAME_SIZE),
               "every kept constant of 200-249 before adcs_control.h is listed");

static void test_constants_200_match(void **state)
{
    (void)state;
    for (size_t i = 0U; i < sizeof(k_constants_200) / sizeof(k_constants_200[0]); i++) {
        double rust = sqso_rust_constant(k_constants_200[i].id);
        /* Exactly the same value: compare the representations */
        bool same = memcmp(&rust, &k_constants_200[i].value, sizeof(rust)) == 0;
        if (!same) {
            print_error("%s: C %g, Rust %g\n", k_constants_200[i].name, k_constants_200[i].value, rust);
        }
        assert_true(same);
    }
    assert_true(isnan(sqso_rust_constant(RUST_CONST_END_200)));
}

/** The template table the Rust beacon exports is the one it sends */
static void test_beacon_templates_exported(void **state)
{
    (void)state;
    assert_int_equal(beacon_template_count, 8U);
    for (size_t i = 0U; i < beacon_template_count; i++) {
        assert_non_null(beacon_templates[i]);
        assert_true(strlen(beacon_templates[i]) > 0U);
    }
    assert_string_equal(beacon_templates[beacon_template_count - 1U],
                        "Amateur satellite beacon - 73 de " BEACON_CALLSIGN);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_struct_layouts_match),
        cmocka_unit_test(test_enums_are_32_bit),
        cmocka_unit_test(test_constants_match),
        cmocka_unit_test(test_rf_group_contract),
        cmocka_unit_test(test_station_identity_matches),
        cmocka_unit_test(test_dictionary_table_matches_header),
        cmocka_unit_test(test_layouts_and_constants_250),
        cmocka_unit_test(test_uart_defaults_match_header),
        cmocka_unit_test(test_power_sensor_gnss_layouts_match),
        cmocka_unit_test(test_power_sensor_gnss_constants_match),
        cmocka_unit_test(test_struct_layouts_200_match),
        cmocka_unit_test(test_constants_200_match),
        cmocka_unit_test(test_beacon_templates_exported),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
