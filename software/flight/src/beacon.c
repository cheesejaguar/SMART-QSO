/**
 * @file beacon.c
 * @brief SMART-QSO Beacon Generation Module Implementation
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 */

#include "beacon.h"
#include "eps_control.h"
#include "mission_data.h"
#include "fault_mgmt.h"

#include <stdio.h>
#include <string.h>
#include <ctype.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Beacon module state */
static BeaconState_t s_beacon_state;

/** Template message index */
static size_t s_template_index = 0;

/** Module initialized flag */
static bool s_initialized = false;

/*===========================================================================*/
/* Beacon Template Messages                                                   */
/*===========================================================================*/

const char *beacon_templates[] = {
    "SMART-QSO beacon - autonomous AI transponder demo",
    "Hello from space! CubeSat systems nominal",
    "Amateur radio in orbit - powered by AI",
    "SMART-QSO 1U - technology demonstration mission",
    "Greetings from LEO! All systems go",
    "CubeSat beacon - AI payload standby",
    "SMART-QSO mission - thanks for listening!",
    "Amateur satellite beacon - 73 de SQSO-1"
};

const size_t beacon_template_count = sizeof(beacon_templates) / sizeof(beacon_templates[0]);

/*===========================================================================*/
/* Helper Functions                                                           */
/*===========================================================================*/

/**
 * @brief Encode callsign for AX.25 address field
 */
static void encode_ax25_address(const char *callsign, uint8_t ssid,
                                 uint8_t *output, bool is_last)
{
    size_t len = strlen(callsign);
    size_t i;

    /* Pad with spaces and shift left by 1 */
    for (i = 0; i < 6; i++) {
        if (i < len) {
            output[i] = (uint8_t)(toupper((unsigned char)callsign[i]) << 1);
        } else {
            output[i] = ' ' << 1;
        }
    }

    /* SSID byte: bits 1-4 = SSID, bit 0 = end flag, bits 5-6 = reserved (11) */
    output[6] = (uint8_t)(0x60 | ((ssid & 0x0F) << 1) | (is_last ? 0x01 : 0x00));
}

/*===========================================================================*/
/* Beacon Functions                                                           */
/*===========================================================================*/

SmartQsoResult_t beacon_init(void)
{
    memset(&s_beacon_state, 0, sizeof(s_beacon_state));

    s_beacon_state.interval_ms = BEACON_INTERVAL_ACTIVE_MS;
    s_beacon_state.ai_available = false;
    s_template_index = 0;

    s_initialized = true;

    return SMART_QSO_OK;
}

bool beacon_should_transmit(uint64_t now_ms)
{
    if (!s_initialized) {
        return false;
    }

    uint64_t elapsed = now_ms - s_beacon_state.last_beacon_ms;
    return elapsed >= s_beacon_state.interval_ms;
}

void beacon_update_interval(PowerMode_t power_mode)
{
    if (!s_initialized) {
        return;
    }

    switch (power_mode) {
        case POWER_MODE_SAFE:
            s_beacon_state.interval_ms = BEACON_INTERVAL_SAFE_MS;
            break;
        case POWER_MODE_IDLE:
            s_beacon_state.interval_ms = BEACON_INTERVAL_IDLE_MS;
            break;
        case POWER_MODE_ACTIVE:
        default:
            s_beacon_state.interval_ms = BEACON_INTERVAL_ACTIVE_MS;
            break;
    }
}

SmartQsoResult_t beacon_generate_content(BeaconContent_t *content,
                                          const char *ai_text,
                                          size_t ai_text_len)
{
    if (content == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    memset(content, 0, sizeof(BeaconContent_t));

    /* Determine text source */
    if (ai_text != NULL && ai_text_len > 0 &&
        beacon_validate_text(ai_text, ai_text_len)) {
        /* Use AI-generated text */
        size_t copy_len = ai_text_len;
        if (copy_len >= BEACON_MAX_TEXT_LEN) {
            copy_len = BEACON_MAX_TEXT_LEN - 1;
        }
        memcpy(content->text, ai_text, copy_len);
        content->text[copy_len] = '\0';
        content->text_len = copy_len;
        content->source = BEACON_SOURCE_AI;
    } else if (s_beacon_state.ai_available &&
               s_beacon_state.last_ai_text[0] != '\0') {
        /* Use stored AI text */
        size_t len = strlen(s_beacon_state.last_ai_text);
        memcpy(content->text, s_beacon_state.last_ai_text, len + 1);
        content->text_len = len;
        content->source = BEACON_SOURCE_AI;
    } else {
        /* Fallback to template */
        const char *tmpl = beacon_get_template();
        size_t len = strlen(tmpl);
        memcpy(content->text, tmpl, len + 1);
        content->text_len = len;
        content->source = BEACON_SOURCE_TEMPLATE;
    }

    /* Populate telemetry */
    content->telemetry.timestamp = (uint32_t)(smart_qso_now_ms() / 1000);
    content->telemetry.power_mode = (uint8_t)eps_get_power_mode();

    /* Get EPS state for telemetry */
    EpsControlState_t eps_state;
    if (eps_get_state(&eps_state) == SMART_QSO_OK) {
        /* Note: EpsControlState_t doesn't have soc/sunlit - use power_mode */
        content->telemetry.soc = 75;  /* Default - would come from actual EPS */
        content->telemetry.battery_mv = 7400;  /* Default */
        content->telemetry.sunlit = 1;  /* Default */
        content->telemetry.rf_power = eps_state.radio_enabled ? 1 : 0;
    }

    /* Get mission data */
    MissionData_t mission;
    if (mission_data_get(&mission) == SMART_QSO_OK) {
        content->telemetry.qso_count = (uint16_t)(mission.reset_count);
        content->telemetry.orbit_number = (uint16_t)(mission.total_uptime_ms / 5400000UL);
    }

    /* Get fault status */
    FaultLogEntry_t last_fault;
    if (fault_log_get_last(&last_fault) == SMART_QSO_OK) {
        content->telemetry.fault_flags = (uint8_t)(last_fault.severity >= FAULT_SEVERITY_ERROR ? 0x01 : 0x00);
    }

    /* Increment sequence */
    s_beacon_state.sequence++;
    content->sequence = s_beacon_state.sequence;

    return SMART_QSO_OK;
}

size_t beacon_format_payload(const BeaconContent_t *content,
                             char *buffer,
                             size_t buffer_len)
{
    if (content == NULL || buffer == NULL || buffer_len == 0) {
        return 0;
    }

    int written = snprintf(buffer, buffer_len,
        "de %s: %s | T=%u M=%s SOC=%u BV=%.1f SUN=%u RF=%u QSO=%u",
        BEACON_CALLSIGN,
        content->text,
        content->telemetry.timestamp,
        content->telemetry.power_mode == 0 ? "SAFE" :
            (content->telemetry.power_mode == 1 ? "IDLE" : "ACTIVE"),
        content->telemetry.soc,
        content->telemetry.battery_mv / 1000.0,
        content->telemetry.sunlit,
        content->telemetry.rf_power,
        content->telemetry.qso_count);

    if (written < 0 || (size_t)written >= buffer_len) {
        return 0;
    }

    return (size_t)written;
}

SmartQsoResult_t beacon_build_ax25_frame(const BeaconContent_t *content,
                                          Ax25Frame_t *frame)
{
    if (content == NULL || frame == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    memset(frame, 0, sizeof(Ax25Frame_t));

    /* Encode destination address (CQ broadcast) */
    encode_ax25_address("CQ", 0, frame->dest_addr, false);

    /* Encode source address */
    encode_ax25_address(BEACON_CALLSIGN, 1, frame->src_addr, true);

    /* Control and PID */
    frame->ctrl = AX25_CTRL_UI;
    frame->pid = AX25_PID_NO_L3;

    /* Format info field */
    frame->info_len = beacon_format_payload(content,
                                             (char *)frame->info,
                                             sizeof(frame->info));

    if (frame->info_len == 0) {
        return SMART_QSO_ERROR;
    }

    /* Calculate FCS (CRC-16 CCITT) */
    uint8_t fcs_data[AX25_ADDR_LEN * 2 + 2 + BEACON_MAX_PAYLOAD_LEN];
    size_t fcs_len = 0;

    memcpy(fcs_data + fcs_len, frame->dest_addr, AX25_ADDR_LEN);
    fcs_len += AX25_ADDR_LEN;
    memcpy(fcs_data + fcs_len, frame->src_addr, AX25_ADDR_LEN);
    fcs_len += AX25_ADDR_LEN;
    fcs_data[fcs_len++] = frame->ctrl;
    fcs_data[fcs_len++] = frame->pid;
    memcpy(fcs_data + fcs_len, frame->info, frame->info_len);
    fcs_len += frame->info_len;

    frame->fcs = beacon_crc16(fcs_data, fcs_len);

    return SMART_QSO_OK;
}

size_t beacon_serialize_ax25(const Ax25Frame_t *frame,
                             uint8_t *buffer,
                             size_t buffer_len)
{
    if (frame == NULL || buffer == NULL) {
        return 0;
    }

    /* Calculate total frame size */
    size_t frame_size = 1 +                    /* Opening flag */
                        AX25_ADDR_LEN * 2 +    /* Addresses */
                        1 + 1 +                /* Control + PID */
                        frame->info_len +      /* Info */
                        2 +                    /* FCS */
                        1;                     /* Closing flag */

    if (frame_size > buffer_len) {
        return 0;
    }

    size_t pos = 0;

    /* Opening flag */
    buffer[pos++] = AX25_FLAG;

    /* Destination address */
    memcpy(buffer + pos, frame->dest_addr, AX25_ADDR_LEN);
    pos += AX25_ADDR_LEN;

    /* Source address */
    memcpy(buffer + pos, frame->src_addr, AX25_ADDR_LEN);
    pos += AX25_ADDR_LEN;

    /* Control and PID */
    buffer[pos++] = frame->ctrl;
    buffer[pos++] = frame->pid;

    /* Info field */
    memcpy(buffer + pos, frame->info, frame->info_len);
    pos += frame->info_len;

    /* FCS (little endian) */
    buffer[pos++] = (uint8_t)(frame->fcs & 0xFF);
    buffer[pos++] = (uint8_t)((frame->fcs >> 8) & 0xFF);

    /* Closing flag */
    buffer[pos++] = AX25_FLAG;

    return pos;
}

SmartQsoResult_t beacon_transmit(const BeaconContent_t *content)
{
    if (content == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    /* Build AX.25 frame */
    Ax25Frame_t frame;
    SmartQsoResult_t result = beacon_build_ax25_frame(content, &frame);
    if (result != SMART_QSO_OK) {
        return result;
    }

    /* Serialize frame */
    uint8_t tx_buffer[AX25_MAX_FRAME_LEN];
    size_t tx_len = beacon_serialize_ax25(&frame, tx_buffer, sizeof(tx_buffer));
    if (tx_len == 0) {
        return SMART_QSO_ERROR;
    }

    /* Update statistics */
    s_beacon_state.total_bytes_tx += (uint32_t)tx_len;

    if (content->source == BEACON_SOURCE_AI) {
        s_beacon_state.ai_beacon_count++;
    } else {
        s_beacon_state.template_count++;
    }

    /* Note: Actual RF transmission would be done via HAL/RF driver */
    /* For simulation, just print the beacon content */
    char payload[BEACON_MAX_PAYLOAD_LEN];
    beacon_format_payload(content, payload, sizeof(payload));
    printf("[BEACON] TX: %s\n", payload);

    return SMART_QSO_OK;
}

void beacon_mark_transmitted(uint64_t now_ms)
{
    if (!s_initialized) {
        return;
    }

    s_beacon_state.last_beacon_ms = (uint32_t)now_ms;
}

SmartQsoResult_t beacon_get_state(BeaconState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    memcpy(state, &s_beacon_state, sizeof(BeaconState_t));
    return SMART_QSO_OK;
}

void beacon_set_ai_available(bool available)
{
    if (!s_initialized) {
        return;
    }

    s_beacon_state.ai_available = available;
}

SmartQsoResult_t beacon_store_ai_text(const char *text, size_t len)
{
    if (text == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    if (!beacon_validate_text(text, len)) {
        return SMART_QSO_ERROR;
    }

    size_t copy_len = len;
    if (copy_len >= BEACON_MAX_TEXT_LEN) {
        copy_len = BEACON_MAX_TEXT_LEN - 1;
    }

    memcpy(s_beacon_state.last_ai_text, text, copy_len);
    s_beacon_state.last_ai_text[copy_len] = '\0';

    return SMART_QSO_OK;
}

const char *beacon_get_template(void)
{
    const char *tmpl = beacon_templates[s_template_index];

    s_template_index++;
    if (s_template_index >= beacon_template_count) {
        s_template_index = 0;
    }

    return tmpl;
}

uint16_t beacon_crc16(const uint8_t *data, size_t len)
{
    uint16_t crc = CRC16_INIT;

    for (size_t i = 0; i < len; i++) {
        crc ^= data[i];
        for (int j = 0; j < 8; j++) {
            if (crc & 0x0001) {
                crc = (crc >> 1) ^ CRC16_POLY;
            } else {
                crc >>= 1;
            }
        }
    }

    return crc ^ 0xFFFF;
}

bool beacon_validate_text(const char *text, size_t len)
{
    if (text == NULL) {
        return false;
    }

    if (len == 0 || len > BEACON_MAX_TEXT_LEN) {
        return false;
    }

    /* Check for ASCII only */
    for (size_t i = 0; i < len; i++) {
        unsigned char c = (unsigned char)text[i];
        /* Allow printable ASCII and common whitespace */
        if (c < 0x20 || c > 0x7E) {
            if (c != '\t' && c != '\n' && c != '\r') {
                return false;
            }
        }
    }

    return true;
}

uint32_t beacon_get_total_count(void)
{
    if (!s_initialized) {
        return 0;
    }

    return s_beacon_state.ai_beacon_count + s_beacon_state.template_count;
}

uint8_t beacon_get_ai_percentage(void)
{
    if (!s_initialized) {
        return 0;
    }

    uint32_t total = beacon_get_total_count();
    if (total == 0) {
        return 0;
    }

    return (uint8_t)((s_beacon_state.ai_beacon_count * 100) / total);
}

void beacon_reset_stats(void)
{
    if (!s_initialized) {
        return;
    }

    s_beacon_state.ai_beacon_count = 0;
    s_beacon_state.template_count = 0;
    s_beacon_state.total_bytes_tx = 0;
    s_beacon_state.sequence = 0;
}
