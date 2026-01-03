/**
 * @file telemetry.c
 * @brief Telemetry formatting implementation for SMART-QSO flight software
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 */

#include "telemetry.h"
#include "system_state.h"
#include "safe_string.h"
#include <stddef.h>

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Telemetry statistics */
static TlmStats_t s_stats;

/** Telemetry rate in milliseconds */
static uint32_t s_rate_ms = TLM_DEFAULT_RATE_MS;

/** Last telemetry generation time */
static uint64_t s_last_tlm_time_ms = 0;

/** Frame version */
static const uint8_t TLM_VERSION = 1;

/** Initialization flag */
static bool s_initialized = false;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Fill telemetry header
 */
static void fill_header(TlmHeader_t *header, TlmType_t type, uint16_t data_len)
{
    header->sync_word = TLM_SYNC_WORD;
    header->version = TLM_VERSION;
    header->type = (uint8_t)type;
    header->sequence = s_stats.sequence_number++;
    header->timestamp_s = sys_get_uptime_s();
    header->data_len = data_len;
}

/**
 * @brief Calculate frame CRC
 */
static uint32_t calculate_frame_crc(const TlmFrame_t *frame, size_t payload_len)
{
    size_t crc_len = sizeof(TlmHeader_t) + payload_len;
    return smart_qso_crc32(frame, crc_len);
}

/*******************************************************************************
 * Public Functions
 ******************************************************************************/

SmartQsoResult_t tlm_init(void)
{
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));
    s_rate_ms = TLM_DEFAULT_RATE_MS;
    s_last_tlm_time_ms = 0;
    s_initialized = true;

    return SMART_QSO_OK;
}

SmartQsoResult_t tlm_set_rate(uint32_t rate_ms)
{
    if (rate_ms < TLM_MIN_RATE_MS) {
        rate_ms = TLM_MIN_RATE_MS;
    } else if (rate_ms > TLM_MAX_RATE_MS) {
        rate_ms = TLM_MAX_RATE_MS;
    }

    s_rate_ms = rate_ms;
    return SMART_QSO_OK;
}

uint32_t tlm_get_rate(void)
{
    return s_rate_ms;
}

SmartQsoResult_t tlm_generate_housekeeping(TlmFrame_t *frame, size_t *frame_len)
{
    if ((frame == NULL) || (frame_len == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    /* Get system state */
    PowerState_t power;
    ThermalState_t thermal;
    CommState_t comm;
    AdcsState_t adcs;

    (void)sys_get_power_state(&power);
    (void)sys_get_thermal_state(&thermal);
    (void)sys_get_comm_state(&comm);
    (void)sys_get_adcs_state(&adcs);

    /* Fill housekeeping payload */
    TlmHousekeeping_t *hk = (TlmHousekeeping_t *)frame->payload;

    /* Power */
    hk->battery_voltage_mv = (uint16_t)(power.battery_voltage * 1000.0);
    hk->battery_current_ma = (int16_t)(power.battery_current * 1000.0);
    hk->state_of_charge = (uint8_t)(power.state_of_charge * 100.0);
    hk->power_mode = (uint8_t)power.power_mode;

    /* Thermal */
    hk->obc_temp_c = (int8_t)thermal.obc_temp_c;
    hk->eps_temp_c = (int8_t)thermal.eps_temp_c;
    hk->battery_temp_c = (int8_t)thermal.battery_temp_c;
    hk->payload_temp_c = (int8_t)thermal.payload_temp_c;

    /* Status */
    hk->op_state = (uint8_t)sys_get_operational_state();
    hk->fault_flags = sys_has_thermal_fault() ? 0x01U : 0x00U;
    hk->boot_count = (uint16_t)sys_get_boot_count();
    hk->uptime_s = sys_get_uptime_s();

    /* Communications */
    hk->packets_sent = (uint16_t)comm.packets_sent;
    hk->packets_received = (uint16_t)comm.packets_received;
    hk->beacon_count = (uint16_t)comm.beacon_count;

    /* ADCS */
    hk->adcs_mode = 0;  /* Would come from ADCS module */
    hk->detumbled = adcs.detumbled ? 1U : 0U;

    /* Fill header */
    fill_header(&frame->header, TLM_TYPE_HOUSEKEEPING, sizeof(TlmHousekeeping_t));

    /* Calculate CRC */
    frame->crc32 = calculate_frame_crc(frame, sizeof(TlmHousekeeping_t));

    *frame_len = sizeof(TlmHeader_t) + sizeof(TlmHousekeeping_t) + sizeof(uint32_t);
    s_stats.frames_generated++;

    return SMART_QSO_OK;
}

SmartQsoResult_t tlm_generate_eps(TlmFrame_t *frame, size_t *frame_len)
{
    if ((frame == NULL) || (frame_len == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    PowerState_t power;
    ThermalState_t thermal;

    (void)sys_get_power_state(&power);
    (void)sys_get_thermal_state(&thermal);

    TlmEps_t *eps = (TlmEps_t *)frame->payload;

    eps->battery_voltage_mv = (uint16_t)(power.battery_voltage * 1000.0);
    eps->battery_current_ma = (int16_t)(power.battery_current * 1000.0);
    eps->solar_voltage_mv = (uint16_t)(power.solar_power * 100.0);  /* Simplified */
    eps->solar_current_ma = 0;
    eps->state_of_charge = (uint8_t)(power.state_of_charge * 100.0);
    eps->power_mode = (uint8_t)power.power_mode;
    eps->heater_enabled = thermal.heater_enabled ? 1U : 0U;
    eps->payload_enabled = power.payload_enabled ? 1U : 0U;
    eps->battery_temp_c = (int8_t)thermal.battery_temp_c;
    eps->pcb_temp_c = (int8_t)thermal.eps_temp_c;

    fill_header(&frame->header, TLM_TYPE_EPS, sizeof(TlmEps_t));
    frame->crc32 = calculate_frame_crc(frame, sizeof(TlmEps_t));

    *frame_len = sizeof(TlmHeader_t) + sizeof(TlmEps_t) + sizeof(uint32_t);
    s_stats.frames_generated++;

    return SMART_QSO_OK;
}

SmartQsoResult_t tlm_generate_adcs(TlmFrame_t *frame, size_t *frame_len)
{
    if ((frame == NULL) || (frame_len == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    AdcsState_t adcs;
    (void)sys_get_adcs_state(&adcs);

    TlmAdcs_t *tlm_adcs = (TlmAdcs_t *)frame->payload;

    tlm_adcs->mag_x_ut_x10 = (int16_t)(adcs.mag_x_ut * 10.0f);
    tlm_adcs->mag_y_ut_x10 = (int16_t)(adcs.mag_y_ut * 10.0f);
    tlm_adcs->mag_z_ut_x10 = (int16_t)(adcs.mag_z_ut * 10.0f);
    tlm_adcs->gyro_x_dps_x10 = (int16_t)(adcs.gyro_x_dps * 10.0f);
    tlm_adcs->gyro_y_dps_x10 = (int16_t)(adcs.gyro_y_dps * 10.0f);
    tlm_adcs->gyro_z_dps_x10 = (int16_t)(adcs.gyro_z_dps * 10.0f);
    tlm_adcs->sun_x_x100 = (int16_t)(adcs.sun_vector_x * 100.0f);
    tlm_adcs->sun_y_x100 = (int16_t)(adcs.sun_vector_y * 100.0f);
    tlm_adcs->sun_z_x100 = (int16_t)(adcs.sun_vector_z * 100.0f);
    tlm_adcs->mode = 0;
    tlm_adcs->status = (adcs.detumbled ? 0x01U : 0x00U) |
                       (adcs.sun_acquired ? 0x02U : 0x00U);

    fill_header(&frame->header, TLM_TYPE_ADCS, sizeof(TlmAdcs_t));
    frame->crc32 = calculate_frame_crc(frame, sizeof(TlmAdcs_t));

    *frame_len = sizeof(TlmHeader_t) + sizeof(TlmAdcs_t) + sizeof(uint32_t);
    s_stats.frames_generated++;

    return SMART_QSO_OK;
}

SmartQsoResult_t tlm_generate_beacon(TlmFrame_t *frame, size_t *frame_len)
{
    if ((frame == NULL) || (frame_len == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    /* Beacon is a condensed housekeeping */
    PowerState_t power;
    (void)sys_get_power_state(&power);

    /* Minimal beacon payload */
    uint8_t *beacon = frame->payload;
    beacon[0] = (uint8_t)sys_get_operational_state();
    beacon[1] = (uint8_t)(power.state_of_charge * 100.0);
    beacon[2] = (uint8_t)power.power_mode;
    beacon[3] = sys_has_thermal_fault() ? 0x01U : 0x00U;

    fill_header(&frame->header, TLM_TYPE_BEACON, 4);
    frame->crc32 = calculate_frame_crc(frame, 4);

    *frame_len = sizeof(TlmHeader_t) + 4 + sizeof(uint32_t);
    s_stats.frames_generated++;

    (void)sys_increment_beacon_count();

    return SMART_QSO_OK;
}

bool tlm_is_due(void)
{
    if (!s_initialized) {
        return false;
    }

    uint64_t now = smart_qso_now_ms();
    return ((now - s_last_tlm_time_ms) >= s_rate_ms);
}

SmartQsoResult_t tlm_mark_transmitted(bool success)
{
    s_last_tlm_time_ms = smart_qso_now_ms();
    s_stats.last_tx_time_ms = (uint32_t)s_last_tlm_time_ms;

    if (success) {
        s_stats.frames_transmitted++;
        (void)sys_increment_packets_sent();
    } else {
        s_stats.frames_failed++;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t tlm_get_stats(TlmStats_t *stats)
{
    if (stats == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(stats, sizeof(*stats), &s_stats, sizeof(s_stats));
    return SMART_QSO_OK;
}

SmartQsoResult_t tlm_serialize(const TlmFrame_t *frame,
                                size_t payload_len,
                                uint8_t *buffer,
                                size_t buffer_size,
                                size_t *bytes_written)
{
    if ((frame == NULL) || (buffer == NULL) || (bytes_written == NULL)) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    size_t total_len = sizeof(TlmHeader_t) + payload_len + sizeof(uint32_t);
    if (buffer_size < total_len) {
        return SMART_QSO_ERROR_NO_MEM;
    }

    /* Copy header */
    (void)safe_memcpy(buffer, buffer_size, &frame->header, sizeof(TlmHeader_t));

    /* Copy payload */
    (void)safe_memcpy(&buffer[sizeof(TlmHeader_t)],
                      buffer_size - sizeof(TlmHeader_t),
                      frame->payload, payload_len);

    /* Copy CRC at end */
    size_t crc_offset = sizeof(TlmHeader_t) + payload_len;
    buffer[crc_offset] = (uint8_t)(frame->crc32 >> 24);
    buffer[crc_offset + 1] = (uint8_t)(frame->crc32 >> 16);
    buffer[crc_offset + 2] = (uint8_t)(frame->crc32 >> 8);
    buffer[crc_offset + 3] = (uint8_t)(frame->crc32);

    *bytes_written = total_len;
    return SMART_QSO_OK;
}
