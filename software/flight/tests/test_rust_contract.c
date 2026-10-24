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
#include "eps_control.h"
#include "fault_mgmt.h"
#include "gnss.h"
#include "gse_provision.h"
#include "input_validation.h"
#include "jetson_link.h"
#include "mission_data.h"
#include "nvm_record.h"
#include "scheduler.h"
#include "sensors.h"
#include "sha256.h"
#include "state_machine.h"
#include "station_id.h"
#include "system_state.h"
#include "uplink.h"

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
    assert_int_equal(g_cmd_dictionary[CMD_DICTIONARY_COUNT - 1U].cmd_id, CMD_PLD_DISABLE);
    assert_string_equal(cmd_get_name(CMD_COMM_TX_DISABLE), "TX_DISABLE");
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_struct_layouts_match),
        cmocka_unit_test(test_enums_are_32_bit),
        cmocka_unit_test(test_constants_match),
        cmocka_unit_test(test_station_identity_matches),
        cmocka_unit_test(test_dictionary_table_matches_header),
        cmocka_unit_test(test_power_sensor_gnss_layouts_match),
        cmocka_unit_test(test_power_sensor_gnss_constants_match),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
