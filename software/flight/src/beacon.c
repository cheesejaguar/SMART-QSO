/**
 * @file beacon.c
 * @brief SMART-QSO Beacon Generation Module Implementation
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 */

#include "beacon.h"
#include "beacon_filter.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "rf_control.h"
#include "ax25.h"
#include "station_id.h"
#include "smart_qso.h"

#include <stdio.h>
#include <string.h>
#include <stdint.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Beacon module state */
static BeaconState_t s_beacon_state;

/** Template message index */
static size_t s_template_index = 0;

/** Module initialized flag */
static bool s_initialized = false;

/** Latest measurements and when they were taken (SW-13) */
static BeaconMeasurements_t s_meas;
static uint64_t s_meas_ms = 0U;
static bool s_meas_set = false;

/*===========================================================================*/
/* Beacon Template Messages                                                   */
/*===========================================================================*/

const char *beacon_templates[] = {
    "SMART-QSO beacon - autonomous AI transponder demo",
    "Hello from space! SMART-QSO CubeSat",
    "Amateur radio in orbit - powered by AI",
    "SMART-QSO 1U - technology demonstration mission",
    "Greetings from LEO! 73 to all",
    "SMART-QSO CubeSat beacon - reports welcome",
    "SMART-QSO mission - thanks for listening!",
    "Amateur satellite beacon - 73 de KK6GDB-11"
};

const size_t beacon_template_count = sizeof(beacon_templates) / sizeof(beacon_templates[0]);

/*===========================================================================*/
/* Helper Functions                                                           */
/*===========================================================================*/

/*===========================================================================*/
/* Beacon Functions                                                           */
/*===========================================================================*/

SmartQsoResult_t beacon_init(void)
{
    (void)memset(&s_beacon_state, 0, sizeof(s_beacon_state));

    s_beacon_state.interval_ms = BEACON_INTERVAL_ACTIVE_MS;
    s_beacon_state.ai_available = false;
    s_template_index = 0;
    (void)memset(&s_meas, 0, sizeof(s_meas));
    s_meas_ms = 0U;
    s_meas_set = false;

    s_initialized = true;

    return SMART_QSO_OK;
}

bool beacon_should_transmit(uint64_t now_ms)
{
    /* No beacon while a transmit inhibit is active or the EPS has shed the
     * beacon (state EMERGENCY) */
    if (!s_initialized || !rf_control_tx_permitted() || !eps_is_beacon_enabled()) {
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

/** Beacon text: the given AI text if it passes the content filter, the
 *  stored AI text if available, else the next template */
static void select_text(BeaconContent_t *content, const char *ai_text, size_t ai_text_len)
{
    bool ai_ok = (ai_text != NULL) && (ai_text_len > 0U) &&
                 beacon_validate_text(ai_text, ai_text_len);
    if ((ai_text != NULL) && (ai_text_len > 0U) && !ai_ok) {
        s_beacon_state.ai_rejected_count++;
    }

    if (ai_ok) {
        size_t copy_len = ai_text_len;
        if (copy_len >= BEACON_MAX_TEXT_LEN) {
            copy_len = BEACON_MAX_TEXT_LEN - 1;
        }
        (void)memcpy(content->text, ai_text, copy_len);
        content->text[copy_len] = '\0';
        content->text_len = copy_len;
        content->source = BEACON_SOURCE_AI;
    } else if (s_beacon_state.ai_available && (s_beacon_state.last_ai_text[0] != '\0')) {
        /* Stored Jetson text is used for one beacon only (BEACON_SPEC section 5) */
        size_t len = strlen(s_beacon_state.last_ai_text);
        (void)memcpy(content->text, s_beacon_state.last_ai_text, len + 1);
        content->text_len = len;
        content->source = BEACON_SOURCE_AI;
        s_beacon_state.last_ai_text[0] = '\0';
    } else {
        const char *tmpl = beacon_get_template();
        size_t len = strlen(tmpl);
        (void)memcpy(content->text, tmpl, len + 1);
        content->text_len = len;
        content->source = BEACON_SOURCE_TEMPLATE;
    }
}

/** Telemetry: measured values only, each with its validity flag (SW-13).
 *  Fields not measured, or measured too long ago, are sent "-" */
static void fill_telemetry(BeaconTelemetry_t *t)
{
    uint64_t now = smart_qso_now_ms();
    t->timestamp = (uint32_t)(now / 1000U);
    t->power_mode = (uint8_t)eps_get_power_mode();

    if (s_meas_set && (now >= s_meas_ms) && ((now - s_meas_ms) <= BEACON_MEAS_MAX_AGE_MS)) {
        uint8_t flags = s_meas.valid;
        if ((flags & BEACON_TLM_SOC) != 0U) {
            t->soc = s_meas.soc;
        }
        if ((flags & BEACON_TLM_BV) != 0U) {
            t->battery_mv = s_meas.battery_mv;
        }
        if ((flags & BEACON_TLM_SUN) != 0U) {
            t->sunlit = s_meas.sunlit;
        }
        if ((flags & BEACON_TLM_QSO) != 0U) {
            t->qso_count = s_meas.qso_count;
        }
        t->valid = (uint8_t)(flags & (BEACON_TLM_SOC | BEACON_TLM_BV |
                                      BEACON_TLM_SUN | BEACON_TLM_QSO));
    }

    /* Radio power switch state */
    EpsControlState_t eps_state;
    if (eps_get_state(&eps_state) == SMART_QSO_OK) {
        t->rf_power = eps_state.radio_enabled ? 1U : 0U;
        t->valid |= BEACON_TLM_RF;
    }

    FaultLogEntry_t last_fault;
    if (fault_log_get_last(&last_fault) == SMART_QSO_OK) {
        t->fault_flags = (uint8_t)(last_fault.severity >= FAULT_SEVERITY_ERROR ? 0x01 : 0x00);
    }
}

SmartQsoResult_t beacon_generate_content(BeaconContent_t *content,
                                          const char *ai_text,
                                          size_t ai_text_len)
{
    if (content == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)memset(content, 0, sizeof(BeaconContent_t));
    select_text(content, ai_text, ai_text_len);
    fill_telemetry(&content->telemetry);

    s_beacon_state.sequence++;
    content->sequence = s_beacon_state.sequence;
    return SMART_QSO_OK;
}

/**
 * Longest the fixed fields can be: "de " CALL ": " then " | T=" (uint32),
 * " M=" (ACTIVE), " SOC=" (uint8), " BV=" (65.5), " SUN=" (uint8),
 * " RF=" (uint8), " QSO=" (uint16). The text gets whatever space is left, so
 * the information field always fits and the telemetry is never cut.
 */
#define BEACON_FIXED_MAX_LEN    (3U + (sizeof(BEACON_CALLSIGN) - 1U) + 2U + \
                                 5U + 10U + 3U + 6U + 5U + 3U + 4U + 4U + \
                                 5U + 3U + 4U + 3U + 5U + 5U)

/** A telemetry value, or "-" when its flag is not set in valid */
static void format_field(char *out, size_t out_len, uint8_t valid, uint8_t flag,
                         uint32_t value)
{
    if ((valid & flag) != 0U) {
        (void)snprintf(out, out_len, "%u", (unsigned int)value);
    } else {
        (void)snprintf(out, out_len, "-");
    }
}

size_t beacon_format_payload(const BeaconContent_t *content,
                             char *buffer,
                             size_t buffer_len)
{
    if ((content == NULL) || (buffer == NULL) || (buffer_len < (BEACON_FIXED_MAX_LEN + 1U))) {
        return 0;
    }

    /* Cut the text, never the telemetry */
    size_t room = buffer_len - BEACON_FIXED_MAX_LEN - 1U;
    const char *nul = (const char *)memchr(content->text, '\0', sizeof(content->text));
    size_t text_len = (nul != NULL) ? (size_t)(nul - content->text) : sizeof(content->text);
    if (text_len > room) {
        text_len = room;
    }

    const BeaconTelemetry_t *t = &content->telemetry;
    char soc[16];
    char bv[16];
    char sun[16];
    char rf[16];
    char qso[16];
    format_field(soc, sizeof(soc), t->valid, BEACON_TLM_SOC, t->soc);
    format_field(sun, sizeof(sun), t->valid, BEACON_TLM_SUN, t->sunlit);
    format_field(rf, sizeof(rf), t->valid, BEACON_TLM_RF, t->rf_power);
    format_field(qso, sizeof(qso), t->valid, BEACON_TLM_QSO, t->qso_count);
    if ((t->valid & BEACON_TLM_BV) != 0U) {
        /* Battery volts to one decimal, rounded, in integer arithmetic */
        uint32_t tenths = ((uint32_t)t->battery_mv + 50U) / 100U;
        (void)snprintf(bv, sizeof(bv), "%u.%u",
                       (unsigned int)(tenths / 10U), (unsigned int)(tenths % 10U));
    } else {
        (void)snprintf(bv, sizeof(bv), "-");
    }

    int written = snprintf(buffer, buffer_len,
        "de %s: %.*s | T=%u M=%s SOC=%s BV=%s SUN=%s RF=%s QSO=%s",
        BEACON_CALLSIGN,
        (int)text_len, content->text,
        (unsigned int)t->timestamp,
        t->power_mode == 0 ? "SAFE" : (t->power_mode == 1 ? "IDLE" : "ACTIVE"),
        soc, bv, sun, rf, qso);

    if ((written < 0) || ((size_t)written >= buffer_len)) {
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

    (void)memset(frame, 0, sizeof(Ax25Frame_t));

    /* Encode destination address (CQ broadcast) */
    ax25_encode_address("CQ", 0U, frame->dest_addr, false);

    /* Encode source address */
    ax25_encode_address(SQSO_CALLSIGN, SQSO_SSID, frame->src_addr, true);

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

    (void)memcpy(fcs_data + fcs_len, frame->dest_addr, AX25_ADDR_LEN);
    fcs_len += AX25_ADDR_LEN;
    (void)memcpy(fcs_data + fcs_len, frame->src_addr, AX25_ADDR_LEN);
    fcs_len += AX25_ADDR_LEN;
    fcs_data[fcs_len++] = frame->ctrl;
    fcs_data[fcs_len++] = frame->pid;
    (void)memcpy(fcs_data + fcs_len, frame->info, frame->info_len);
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
    (void)memcpy(buffer + pos, frame->dest_addr, AX25_ADDR_LEN);
    pos += AX25_ADDR_LEN;

    /* Source address */
    (void)memcpy(buffer + pos, frame->src_addr, AX25_ADDR_LEN);
    pos += AX25_ADDR_LEN;

    /* Control and PID */
    buffer[pos++] = frame->ctrl;
    buffer[pos++] = frame->pid;

    /* Info field */
    (void)memcpy(buffer + pos, frame->info, frame->info_len);
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

    /* Never radiate while TX-OFF is latched (47 CFR 97.207(b)), nor while
     * the EPS has the beacon powered off */
    if (!rf_control_tx_permitted() || !eps_is_beacon_enabled()) {
        return SMART_QSO_ERROR;
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
    /* For simulation, print the frame's info field: exactly what is sent */
    (void)printf("[BEACON] TX: %.*s\n", (int)frame.info_len, (const char *)frame.info);

    return SMART_QSO_OK;
}

void beacon_mark_transmitted(uint64_t now_ms)
{
    if (!s_initialized) {
        return;
    }

    s_beacon_state.last_beacon_ms = now_ms;
}

SmartQsoResult_t beacon_get_state(BeaconState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    (void)memcpy(state, &s_beacon_state, sizeof(BeaconState_t));
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
        s_beacon_state.ai_rejected_count++;
        return SMART_QSO_ERROR;
    }

    size_t copy_len = len;
    if (copy_len >= BEACON_MAX_TEXT_LEN) {
        copy_len = BEACON_MAX_TEXT_LEN - 1;
    }

    (void)memcpy(s_beacon_state.last_ai_text, text, copy_len);
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
    return ax25_fcs(data, len);
}

SmartQsoResult_t beacon_update_measurements(const BeaconMeasurements_t *m, uint64_t sample_ms)
{
    if (m == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if ((((m->valid & BEACON_TLM_SOC) != 0U) && (m->soc > 100U)) ||
        (((m->valid & BEACON_TLM_SUN) != 0U) && (m->sunlit > 1U))) {
        return SMART_QSO_ERROR_INVALID;
    }

    s_meas = *m;
    s_meas_ms = sample_ms;
    s_meas_set = true;
    return SMART_QSO_OK;
}

bool beacon_validate_text(const char *text, size_t len)
{
    return beacon_filter_check(text, len) == BEACON_FILTER_OK;
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
