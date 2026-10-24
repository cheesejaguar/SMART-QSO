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
#include "beacon.h"
#include "beacon_filter.h"
#include "cmd_auth.h"
#include "cmd_dictionary.h"
#include "cmd_handler.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "gse_provision.h"
#include "input_validation.h"
#include "jetson_link.h"
#include "nvm_record.h"
#include "scheduler.h"
#include "sha256.h"
#include "state_machine.h"
#include "station_id.h"
#include "system_state.h"
#include "telemetry.h"
#include "uplink.h"
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
    LAYOUT(RUST_LAYOUT_EPS_CONTROL_STATE, EpsControlState_t),
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
        cmocka_unit_test(test_struct_layouts_200_match),
        cmocka_unit_test(test_constants_200_match),
        cmocka_unit_test(test_beacon_templates_exported),
    };
    return cmocka_run_group_tests(tests, NULL, NULL);
}
