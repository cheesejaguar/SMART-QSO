/**
 * @file test_telemetry.c
 * @brief Binary telemetry frames: header, payload fields, CRC, serialization
 *
 * telemetry.c builds binary frames (TlmFrame_t: a packed 14-byte header, a
 * payload and a CRC-32 trailer) from the system state: housekeeping, EPS,
 * ADCS and a 4-byte status beacon. The specification is telemetry.h (types,
 * units, constants); no other document describes this frame, and nothing in
 * the flight software calls it yet. The text beacon, with its validity flags
 * and "-" for unmeasured fields (BEACON_SPEC.md sections 3-4), is beacon.c
 * and is tested in test_beacon_format.c and test_beacon_telemetry.c.
 *
 * Two defects found here and fixed in telemetry.c:
 * - Scaled fields were truncated, so a value exact in the field's unit was
 *   sent one unit low: SOC 57 % as 56, 8.123 V as 8122 mV, -1.023 A as
 *   -1022 mA. They are now rounded to the nearest unit.
 * - tlm_serialize() did not bound payload_len by the frame's payload, so it
 *   read past the frame (and, with payload_len near SIZE_MAX, the length
 *   wrapped and it reported success). It now refuses such a length.
 *
 * @requirement SYS-303 The OBC shall collect and store telemetry
 * @requirement SYS-124 The EPS shall report voltage, current, and temperature telemetry
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "eps_control.h"
#include "smart_qso.h"
#include "state_machine.h"
#include "system_state.h"
#include "telemetry.h"

/** Bytes of a frame around its payload: header and CRC-32 trailer */
#define FRAME_OVERHEAD   (sizeof(TlmHeader_t) + sizeof(uint32_t))

/** Status beacon payload (telemetry.c): op state, SOC, power mode, fault */
#define BEACON_PAYLOAD_LEN 4U

typedef SmartQsoResult_t (*TlmGenerate_t)(TlmFrame_t *frame, size_t *frame_len);

static const struct {
    TlmGenerate_t generate;
    TlmType_t type;
    size_t payload_len;
} k_frames[] = {
    {tlm_generate_housekeeping, TLM_TYPE_HOUSEKEEPING, sizeof(TlmHousekeeping_t)},
    {tlm_generate_eps, TLM_TYPE_EPS, sizeof(TlmEps_t)},
    {tlm_generate_adcs, TLM_TYPE_ADCS, sizeof(TlmAdcs_t)},
    {tlm_generate_beacon, TLM_TYPE_BEACON, BEACON_PAYLOAD_LEN},
};
#define FRAME_KINDS (sizeof(k_frames) / sizeof(k_frames[0]))

static int setup(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(eps_init(), SMART_QSO_OK);
    assert_int_equal(sys_state_init(), SMART_QSO_OK);
    assert_int_equal(tlm_init(), SMART_QSO_OK);
    return 0;
}

static TlmStats_t stats(void)
{
    TlmStats_t s;
    assert_int_equal(tlm_get_stats(&s), SMART_QSO_OK);
    return s;
}

static TlmHousekeeping_t housekeeping(void)
{
    TlmFrame_t frame;
    TlmHousekeeping_t hk;
    size_t len = 0U;
    assert_int_equal(tlm_generate_housekeeping(&frame, &len), SMART_QSO_OK);
    (void)memcpy(&hk, frame.payload, sizeof(hk));
    return hk;
}

static TlmEps_t eps_frame(void)
{
    TlmFrame_t frame;
    TlmEps_t eps;
    size_t len = 0U;
    assert_int_equal(tlm_generate_eps(&frame, &len), SMART_QSO_OK);
    (void)memcpy(&eps, frame.payload, sizeof(eps));
    return eps;
}

static TlmAdcs_t adcs_frame(void)
{
    TlmFrame_t frame;
    TlmAdcs_t adcs;
    size_t len = 0U;
    assert_int_equal(tlm_generate_adcs(&frame, &len), SMART_QSO_OK);
    (void)memcpy(&adcs, frame.payload, sizeof(adcs));
    return adcs;
}

static void beacon_frame(uint8_t out[BEACON_PAYLOAD_LEN])
{
    TlmFrame_t frame;
    size_t len = 0U;
    assert_int_equal(tlm_generate_beacon(&frame, &len), SMART_QSO_OK);
    (void)memcpy(out, frame.payload, BEACON_PAYLOAD_LEN);
}

/*===========================================================================*/
/* Rate and schedule                                                          */
/*===========================================================================*/

/** Runs first, before any tlm_init(): nothing is due from an uninitialized module */
static void test_not_due_before_init(void **cm_state)
{
    (void)cm_state;
    assert_false(tlm_is_due());
}

/** Init: default rate, zeroed statistics, first frame due at once */
static void test_init_defaults(void **cm_state)
{
    (void)cm_state;
    TlmStats_t s = stats();

    assert_int_equal(tlm_get_rate(), TLM_DEFAULT_RATE_MS);
    assert_int_equal(s.frames_generated, 0U);
    assert_int_equal(s.frames_transmitted, 0U);
    assert_int_equal(s.frames_failed, 0U);
    assert_int_equal(s.last_tx_time_ms, 0U);
    assert_int_equal(s.sequence_number, 0U);
    assert_true(tlm_is_due());
}

/** The rate is configurable within TLM_MIN_RATE_MS..TLM_MAX_RATE_MS */
static void test_rate_in_range_kept(void **cm_state)
{
    (void)cm_state;
    const uint32_t rates[] = {TLM_MIN_RATE_MS, 30000U, TLM_DEFAULT_RATE_MS, TLM_MAX_RATE_MS};

    for (size_t i = 0U; i < (sizeof(rates) / sizeof(rates[0])); i++) {
        assert_int_equal(tlm_set_rate(rates[i]), SMART_QSO_OK);
        assert_int_equal(tlm_get_rate(), rates[i]);
    }
}

/** A rate outside the limits is held to the nearest limit */
static void test_rate_clamped(void **cm_state)
{
    (void)cm_state;
    const struct { uint32_t asked; uint32_t got; } cases[] = {
        {0U, TLM_MIN_RATE_MS},
        {TLM_MIN_RATE_MS - 1U, TLM_MIN_RATE_MS},
        {TLM_MAX_RATE_MS + 1U, TLM_MAX_RATE_MS},
        {UINT32_MAX, TLM_MAX_RATE_MS},
    };

    for (size_t i = 0U; i < (sizeof(cases) / sizeof(cases[0])); i++) {
        assert_int_equal(tlm_set_rate(cases[i].asked), SMART_QSO_OK);
        assert_int_equal(tlm_get_rate(), cases[i].got);
    }
}

static void test_init_restores_default_rate(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(tlm_set_rate(TLM_MIN_RATE_MS), SMART_QSO_OK);
    assert_int_equal(tlm_init(), SMART_QSO_OK);
    assert_int_equal(tlm_get_rate(), TLM_DEFAULT_RATE_MS);
}

/** After a transmission the next frame is due one rate period later */
static void test_due_follows_rate(void **cm_state)
{
    (void)cm_state;
    assert_int_equal(tlm_mark_transmitted(true), SMART_QSO_OK);
    assert_false(tlm_is_due());
    smart_qso_sim_advance_ms(TLM_DEFAULT_RATE_MS / 2U);
    assert_false(tlm_is_due());
    smart_qso_sim_advance_ms(TLM_DEFAULT_RATE_MS / 2U);
    assert_true(tlm_is_due());

    assert_int_equal(tlm_set_rate(TLM_MIN_RATE_MS), SMART_QSO_OK);
    assert_int_equal(tlm_mark_transmitted(true), SMART_QSO_OK);
    assert_false(tlm_is_due());
    smart_qso_sim_advance_ms(TLM_MIN_RATE_MS);
    assert_true(tlm_is_due());
}

/** Success counts a transmitted frame and a sent packet; failure counts neither */
static void test_mark_transmitted_counts(void **cm_state)
{
    (void)cm_state;
    CommState_t comm;

    assert_int_equal(tlm_mark_transmitted(true), SMART_QSO_OK);
    assert_int_equal(tlm_mark_transmitted(true), SMART_QSO_OK);
    assert_int_equal(tlm_mark_transmitted(false), SMART_QSO_OK);

    TlmStats_t s = stats();
    assert_int_equal(s.frames_transmitted, 2U);
    assert_int_equal(s.frames_failed, 1U);
    assert_int_not_equal(s.last_tx_time_ms, 0U);
    assert_int_equal(sys_get_comm_state(&comm), SMART_QSO_OK);
    assert_int_equal(comm.packets_sent, 2U);
}

/*===========================================================================*/
/* Arguments                                                                  */
/*===========================================================================*/

/** NULL outputs are refused and nothing is generated */
static void test_null_arguments(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;
    uint8_t buffer[TLM_MAX_FRAME_SIZE];

    for (size_t i = 0U; i < FRAME_KINDS; i++) {
        assert_int_equal(k_frames[i].generate(NULL, &len), SMART_QSO_ERROR_NULL_PTR);
        assert_int_equal(k_frames[i].generate(&frame, NULL), SMART_QSO_ERROR_NULL_PTR);
    }
    assert_int_equal(stats().frames_generated, 0U);
    assert_int_equal(stats().sequence_number, 0U);

    assert_int_equal(tlm_get_stats(NULL), SMART_QSO_ERROR_NULL_PTR);

    assert_int_equal(tlm_generate_eps(&frame, &len), SMART_QSO_OK);
    assert_int_equal(tlm_serialize(NULL, 0U, buffer, sizeof(buffer), &len), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(tlm_serialize(&frame, 0U, NULL, sizeof(buffer), &len), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(tlm_serialize(&frame, 0U, buffer, sizeof(buffer), NULL), SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Header, sequence, CRC                                                      */
/*===========================================================================*/

/** Every frame: sync word, version 1, its type, payload length, uptime, total length */
static void test_header_fields(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;

    assert_int_equal(sizeof(TlmFrame_t), TLM_MAX_FRAME_SIZE);
    for (size_t i = 0U; i < FRAME_KINDS; i++) {
        assert_int_equal(k_frames[i].generate(&frame, &len), SMART_QSO_OK);
        assert_int_equal(frame.header.sync_word, TLM_SYNC_WORD);
        assert_int_equal(frame.header.version, 1U);
        assert_int_equal(frame.header.type, k_frames[i].type);
        assert_int_equal(frame.header.data_len, k_frames[i].payload_len);
        assert_int_equal(frame.header.timestamp_s, sys_get_uptime_s());
        assert_int_equal(len, FRAME_OVERHEAD + k_frames[i].payload_len);
        assert_true(len <= TLM_MAX_FRAME_SIZE);
    }
}

/** The header time is the mission uptime in seconds */
static void test_timestamp_is_uptime(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;

    (void)sys_update_uptime();
    smart_qso_sim_advance_ms(5000U);
    (void)sys_update_uptime();
    assert_true(sys_get_uptime_s() >= 5U);

    assert_int_equal(tlm_generate_housekeeping(&frame, &len), SMART_QSO_OK);
    assert_int_equal(frame.header.timestamp_s, sys_get_uptime_s());
}

/** One sequence counter across all frame types, one count per frame */
static void test_sequence_counts_frames(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;

    for (size_t i = 0U; i < FRAME_KINDS; i++) {
        assert_int_equal(k_frames[i].generate(&frame, &len), SMART_QSO_OK);
        assert_int_equal(frame.header.sequence, i);
    }
    assert_int_equal(stats().frames_generated, FRAME_KINDS);
    assert_int_equal(stats().sequence_number, FRAME_KINDS);
}

/** The 16-bit sequence wraps to 0 after 65535 */
static void test_sequence_wraps(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;

    for (uint32_t i = 0U; i < UINT16_MAX; i++) {
        assert_int_equal(tlm_generate_adcs(&frame, &len), SMART_QSO_OK);
    }
    assert_int_equal(frame.header.sequence, UINT16_MAX - 1U);
    assert_int_equal(tlm_generate_adcs(&frame, &len), SMART_QSO_OK);
    assert_int_equal(frame.header.sequence, UINT16_MAX);
    assert_int_equal(tlm_generate_adcs(&frame, &len), SMART_QSO_OK);
    assert_int_equal(frame.header.sequence, 0U);
    assert_int_equal(stats().frames_generated, (uint32_t)UINT16_MAX + 2U);
}

/** The CRC-32 covers header and payload: a change to either is detected */
static void test_crc_covers_header_and_payload(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;

    for (size_t i = 0U; i < FRAME_KINDS; i++) {
        size_t covered = sizeof(TlmHeader_t) + k_frames[i].payload_len;
        size_t last = k_frames[i].payload_len - 1U;
        assert_int_equal(k_frames[i].generate(&frame, &len), SMART_QSO_OK);
        uint32_t crc = frame.crc32;
        assert_int_equal(smart_qso_crc32(&frame, covered), crc);

        frame.payload[last] = (uint8_t)(frame.payload[last] ^ 0x01U);
        assert_int_not_equal(smart_qso_crc32(&frame, covered), crc);
        frame.payload[last] = (uint8_t)(frame.payload[last] ^ 0x01U);

        frame.header.sequence = (uint16_t)(frame.header.sequence + 1U);
        assert_int_not_equal(smart_qso_crc32(&frame, covered), crc);
    }
}

/*===========================================================================*/
/* Payload fields                                                             */
/*===========================================================================*/

/** Housekeeping carries power, thermal, status, comms and ADCS state */
static void test_housekeeping_fields(void **cm_state)
{
    (void)cm_state;

    /* Leave BOOT first: state entry actions set the EPS power mode */
    (void)sm_process_event(sys_get_sm_context(), EVENT_BOOT_COMPLETE, NULL);
    assert_int_not_equal(sys_get_operational_state(), STATE_BOOT);

    assert_int_equal(sys_set_battery_voltage(8.1), SMART_QSO_OK);
    assert_int_equal(sys_set_battery_current(-0.25), SMART_QSO_OK);
    eps_update_soc(0.83);
    assert_int_equal(eps_set_power_mode(POWER_MODE_IDLE, 0.83), SMART_QSO_OK);
    assert_int_equal(sys_set_temperature(1U, 22.0f), SMART_QSO_OK);
    assert_int_equal(sys_set_temperature(2U, -10.0f), SMART_QSO_OK);
    assert_int_equal(sys_set_temperature(3U, 45.0f), SMART_QSO_OK);
    assert_int_equal(sys_set_temperature(0U, 21.0f), SMART_QSO_OK);
    for (uint32_t i = 0U; i < 3U; i++) {
        assert_int_equal(sys_increment_boot_count(), SMART_QSO_OK);
    }
    assert_int_equal(sys_increment_packets_sent(), SMART_QSO_OK);
    assert_int_equal(sys_increment_packets_sent(), SMART_QSO_OK);
    assert_int_equal(sys_increment_packets_received(), SMART_QSO_OK);
    assert_int_equal(sys_increment_beacon_count(), SMART_QSO_OK);
    assert_int_equal(sys_set_detumbled(true), SMART_QSO_OK);

    TlmHousekeeping_t hk = housekeeping();
    assert_int_equal(hk.battery_voltage_mv, 8100U);
    assert_int_equal(hk.battery_current_ma, -250);
    assert_int_equal(hk.state_of_charge, 83U);
    assert_int_equal(hk.power_mode, POWER_MODE_IDLE);
    assert_int_equal(hk.obc_temp_c, 21);
    assert_int_equal(hk.eps_temp_c, 22);
    assert_int_equal(hk.battery_temp_c, -10);
    assert_int_equal(hk.payload_temp_c, 45);
    assert_int_equal(hk.op_state, sys_get_operational_state());
    assert_int_equal(hk.fault_flags, 0U);
    assert_int_equal(hk.boot_count, 3U);
    assert_int_equal(hk.uptime_s, sys_get_uptime_s());
    assert_int_equal(hk.packets_sent, 2U);
    assert_int_equal(hk.packets_received, 1U);
    assert_int_equal(hk.beacon_count, 1U);
    assert_int_equal(hk.adcs_mode, 0U);
    assert_int_equal(hk.detumbled, 1U);
}

/** EPS frame: battery, SOC, mode, load switches, battery and PCB temperatures */
static void test_eps_fields(void **cm_state)
{
    (void)cm_state;

    assert_int_equal(sys_set_battery_voltage(14.4), SMART_QSO_OK);
    assert_int_equal(sys_set_battery_current(0.5), SMART_QSO_OK);
    eps_update_soc(0.20);
    assert_int_equal(eps_set_power_mode(POWER_MODE_SAFE, 0.20), SMART_QSO_OK);
    assert_int_equal(sys_set_payload_enabled(true), SMART_QSO_OK);
    assert_int_equal(sys_set_temperature(1U, 31.0f), SMART_QSO_OK);
    assert_int_equal(sys_set_temperature(2U, 5.0f), SMART_QSO_OK);

    TlmEps_t eps = eps_frame();
    assert_int_equal(eps.battery_voltage_mv, 14400U);
    assert_int_equal(eps.battery_current_ma, 500);
    assert_int_equal(eps.state_of_charge, 20U);
    assert_int_equal(eps.power_mode, POWER_MODE_SAFE);
    assert_int_equal(eps.payload_enabled, 1U);
    assert_int_equal(eps.heater_enabled, 0U);   /* no heater control yet */
    assert_int_equal(eps.solar_current_ma, 0);  /* not measured: telemetry.c sends 0 */
    assert_int_equal(eps.battery_temp_c, 5);
    assert_int_equal(eps.pcb_temp_c, 31);

    assert_int_equal(sys_set_payload_enabled(false), SMART_QSO_OK);
    assert_int_equal(eps_frame().payload_enabled, 0U);
}

/** ADCS frame: magnetometer in 0.1 uT, gyro in 0.1 deg/s, status bits */
static void test_adcs_fields(void **cm_state)
{
    (void)cm_state;

    assert_int_equal(sys_set_magnetometer(12.3f, -45.6f, 0.0f), SMART_QSO_OK);
    assert_int_equal(sys_set_gyroscope(1.5f, -0.2f, 300.0f), SMART_QSO_OK);

    TlmAdcs_t a = adcs_frame();
    assert_int_equal(a.mag_x_ut_x10, 123);
    assert_int_equal(a.mag_y_ut_x10, -456);
    assert_int_equal(a.mag_z_ut_x10, 0);
    assert_int_equal(a.gyro_x_dps_x10, 15);
    assert_int_equal(a.gyro_y_dps_x10, -2);
    assert_int_equal(a.gyro_z_dps_x10, 3000);
    /* No sun sensor or ADCS mode source feeds the system state yet */
    assert_int_equal(a.sun_x_x100, 0);
    assert_int_equal(a.sun_y_x100, 0);
    assert_int_equal(a.sun_z_x100, 0);
    assert_int_equal(a.mode, 0U);
    assert_int_equal(a.status, 0U);

    assert_int_equal(sys_set_detumbled(true), SMART_QSO_OK);
    assert_int_equal(adcs_frame().status, 0x01U);   /* bit 0 detumbled, bit 1 sun acquired */
}

/** Status beacon: operational state, SOC, power mode, thermal fault */
static void test_beacon_fields(void **cm_state)
{
    (void)cm_state;
    uint8_t b[BEACON_PAYLOAD_LEN];
    CommState_t comm;

    eps_update_soc(0.64);
    assert_int_equal(eps_set_power_mode(POWER_MODE_ACTIVE, 0.64), SMART_QSO_OK);
    beacon_frame(b);
    assert_int_equal(b[0], sys_get_operational_state());
    assert_int_equal(b[1], 64U);
    assert_int_equal(b[2], POWER_MODE_ACTIVE);
    assert_int_equal(b[3], 0U);

    /* Each beacon frame counts in the communications state */
    assert_int_equal(sys_get_comm_state(&comm), SMART_QSO_OK);
    assert_int_equal(comm.beacon_count, 1U);
}

/** An over- or under-temperature reading sets the fault flag in housekeeping and beacon */
static void test_thermal_fault_flag(void **cm_state)
{
    (void)cm_state;
    uint8_t b[BEACON_PAYLOAD_LEN];
    const float faults[] = {70.0f, -25.0f};

    for (size_t i = 0U; i < (sizeof(faults) / sizeof(faults[0])); i++) {
        assert_int_equal(sys_set_temperature(0U, faults[i]), SMART_QSO_OK);
        assert_true(sys_has_thermal_fault());
        assert_int_equal(housekeeping().fault_flags, 0x01U);
        beacon_frame(b);
        assert_int_equal(b[3], 0x01U);
    }

    assert_int_equal(sys_set_temperature(0U, 20.0f), SMART_QSO_OK);
    assert_int_equal(housekeeping().fault_flags, 0x00U);
    beacon_frame(b);
    assert_int_equal(b[3], 0x00U);
}

/** BUG (fixed): a value exact in a field's unit is sent exactly (telemetry.h
 *  units; TELEMETRY_DICTIONARY.md 3.1: BATT_V (V) = raw / 1000). Truncation
 *  sent SOC 57 % as 56, 8.123 V as 8122 mV and -1.023 A as -1022 mA */
static void test_scaled_fields_round_to_unit(void **cm_state)
{
    (void)cm_state;
    uint8_t b[BEACON_PAYLOAD_LEN];

    for (uint32_t pct = 0U; pct <= 100U; pct++) {
        eps_update_soc((double)pct / 100.0);
        assert_int_equal(housekeeping().state_of_charge, pct);
        assert_int_equal(eps_frame().state_of_charge, pct);
        beacon_frame(b);
        assert_int_equal(b[1], pct);
    }

    for (uint32_t mv = 0U; mv <= 20000U; mv++) {
        assert_int_equal(sys_set_battery_voltage((double)mv / 1000.0), SMART_QSO_OK);
        assert_int_equal(housekeeping().battery_voltage_mv, mv);
        assert_int_equal(eps_frame().battery_voltage_mv, mv);
    }

    for (int32_t ma = -2000; ma <= 2000; ma++) {
        assert_int_equal(sys_set_battery_current((double)ma / 1000.0), SMART_QSO_OK);
        assert_int_equal(housekeeping().battery_current_ma, ma);
        assert_int_equal(eps_frame().battery_current_ma, ma);
    }
}

/*===========================================================================*/
/* Serialization                                                              */
/*===========================================================================*/

/** Header and payload as generated, then the CRC-32 big-endian; a receiver
 *  can check the CRC over the bytes before it */
static void test_serialize_layout(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    TlmHeader_t header;
    size_t len = 0U;
    size_t written = 0U;
    uint8_t buffer[TLM_MAX_FRAME_SIZE];

    for (size_t i = 0U; i < FRAME_KINDS; i++) {
        size_t payload_len = k_frames[i].payload_len;
        assert_int_equal(k_frames[i].generate(&frame, &len), SMART_QSO_OK);
        assert_int_equal(tlm_serialize(&frame, payload_len, buffer, sizeof(buffer), &written),
                         SMART_QSO_OK);
        assert_int_equal(written, len);

        (void)memcpy(&header, &frame.header, sizeof(header));
        assert_memory_equal(buffer, &header, sizeof(header));
        assert_memory_equal(&buffer[sizeof(header)], frame.payload, payload_len);

        const uint8_t *crc = &buffer[sizeof(header) + payload_len];
        uint32_t sent = ((uint32_t)crc[0] << 24) | ((uint32_t)crc[1] << 16) |
                        ((uint32_t)crc[2] << 8) | (uint32_t)crc[3];
        assert_int_equal(sent, frame.crc32);
        assert_int_equal(smart_qso_crc32(buffer, sizeof(header) + payload_len), sent);
    }
}

/** The buffer must hold header, payload and CRC: one byte short is refused,
 *  untouched */
static void test_serialize_buffer_limits(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;
    size_t written = 0U;
    uint8_t buffer[TLM_MAX_FRAME_SIZE];
    const size_t payload_len = sizeof(TlmHousekeeping_t);

    assert_int_equal(tlm_generate_housekeeping(&frame, &len), SMART_QSO_OK);

    (void)memset(buffer, 0xA5, sizeof(buffer));
    assert_int_equal(tlm_serialize(&frame, payload_len, buffer, len - 1U, &written),
                     SMART_QSO_ERROR_NO_MEM);
    assert_int_equal(written, 0U);
    for (size_t i = 0U; i < sizeof(buffer); i++) {
        assert_int_equal(buffer[i], 0xA5U);
    }
    assert_int_equal(tlm_serialize(&frame, payload_len, buffer, 0U, &written),
                     SMART_QSO_ERROR_NO_MEM);

    assert_int_equal(tlm_serialize(&frame, payload_len, buffer, len, &written), SMART_QSO_OK);
    assert_int_equal(written, len);

    /* An empty payload is header and CRC only */
    assert_int_equal(tlm_serialize(&frame, 0U, buffer, sizeof(buffer), &written), SMART_QSO_OK);
    assert_int_equal(written, FRAME_OVERHEAD);
}

/** BUG (fixed): a frame is at most TLM_MAX_FRAME_SIZE bytes (telemetry.h).
 *  A payload_len beyond the frame's payload made tlm_serialize() read past
 *  the frame, and one near SIZE_MAX wrapped the length check and succeeded */
static void test_serialize_rejects_payload_beyond_frame(void **cm_state)
{
    (void)cm_state;
    TlmFrame_t frame;
    size_t len = 0U;
    size_t written = 0U;
    uint8_t buffer[2U * TLM_MAX_FRAME_SIZE];
    const size_t full = sizeof(frame.payload);
    const size_t too_long[] = {full + 1U, 300U, SIZE_MAX - 10U, SIZE_MAX};

    assert_int_equal(tlm_generate_housekeeping(&frame, &len), SMART_QSO_OK);

    /* The whole payload area fills the frame exactly */
    assert_int_equal(tlm_serialize(&frame, full, buffer, sizeof(buffer), &written), SMART_QSO_OK);
    assert_int_equal(written, TLM_MAX_FRAME_SIZE);

    for (size_t i = 0U; i < (sizeof(too_long) / sizeof(too_long[0])); i++) {
        written = 0U;
        assert_int_not_equal(tlm_serialize(&frame, too_long[i], buffer, sizeof(buffer), &written),
                             SMART_QSO_OK);
        assert_int_equal(written, 0U);
    }
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        /* First, with no setup: the module has never been initialized */
        cmocka_unit_test(test_not_due_before_init),
        cmocka_unit_test_setup(test_init_defaults, setup),
        cmocka_unit_test_setup(test_rate_in_range_kept, setup),
        cmocka_unit_test_setup(test_rate_clamped, setup),
        cmocka_unit_test_setup(test_init_restores_default_rate, setup),
        cmocka_unit_test_setup(test_due_follows_rate, setup),
        cmocka_unit_test_setup(test_mark_transmitted_counts, setup),
        cmocka_unit_test_setup(test_null_arguments, setup),
        cmocka_unit_test_setup(test_header_fields, setup),
        cmocka_unit_test_setup(test_timestamp_is_uptime, setup),
        cmocka_unit_test_setup(test_sequence_counts_frames, setup),
        cmocka_unit_test_setup(test_sequence_wraps, setup),
        cmocka_unit_test_setup(test_crc_covers_header_and_payload, setup),
        cmocka_unit_test_setup(test_housekeeping_fields, setup),
        cmocka_unit_test_setup(test_eps_fields, setup),
        cmocka_unit_test_setup(test_adcs_fields, setup),
        cmocka_unit_test_setup(test_beacon_fields, setup),
        cmocka_unit_test_setup(test_thermal_fault_flag, setup),
        cmocka_unit_test_setup(test_scaled_fields_round_to_unit, setup),
        cmocka_unit_test_setup(test_serialize_layout, setup),
        cmocka_unit_test_setup(test_serialize_buffer_limits, setup),
        cmocka_unit_test_setup(test_serialize_rejects_payload_beyond_frame, setup),
    };

    return cmocka_run_group_tests_name("Telemetry Tests", tests, NULL, NULL);
}
