/**
 * @file uplink.c
 * @brief Telecommand receive path: receiver -> AX.25 -> command handler
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
 */

#include "uplink.h"
#include "ax25.h"
#include "safe_string.h"
#include "hal/hal_radio.h"
#include <string.h>

static char s_callsign[7];
static uint8_t s_ssid = 0U;
static UplinkStats_t s_stats;
static bool s_initialized = false;

SmartQsoResult_t uplink_init(const char *callsign, uint8_t ssid)
{
    if (callsign == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    size_t len = strlen(callsign);
    if ((len == 0U) || (len >= sizeof(s_callsign)) || (ssid > 15U)) {
        return SMART_QSO_ERROR_INVALID;
    }

    (void)safe_memset(s_callsign, sizeof(s_callsign), 0, sizeof(s_callsign));
    (void)safe_memcpy(s_callsign, sizeof(s_callsign), callsign, len);
    s_ssid = ssid;
    (void)safe_memset(&s_stats, sizeof(s_stats), 0, sizeof(s_stats));
    s_initialized = true;

    return hal_radio_init();
}

bool uplink_poll(CmdResponse_t *response)
{
    uint8_t frame[HAL_RADIO_MAX_FRAME];
    size_t len = 0U;
    Ax25UiView_t view;
    CmdResponse_t local;

    if (!s_initialized || (hal_radio_receive(frame, sizeof(frame), &len) != SMART_QSO_OK)) {
        return false;
    }
    s_stats.frames++;

    if (ax25_parse_ui(frame, len, &view) != AX25_OK) {
        s_stats.rejected++;
        return false;
    }
    if (!ax25_address_matches(view.dest, s_callsign, s_ssid)) {
        s_stats.not_for_us++;
        return false;
    }

    CmdResponse_t *out = (response != NULL) ? response : &local;
    if (cmd_process(view.info, view.info_len, out) != SMART_QSO_OK) {
        s_stats.rejected++;
        return false;
    }
    s_stats.commands++;
    return true;
}

void uplink_get_stats(UplinkStats_t *stats)
{
    if (stats != NULL) {
        (void)safe_memcpy(stats, sizeof(*stats), &s_stats, sizeof(s_stats));
    }
}
