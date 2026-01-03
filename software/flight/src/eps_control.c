/**
 * @file eps_control.c
 * @brief Electrical Power Subsystem (EPS) Control Implementation
 *
 * @requirement SRS-EPS-001 System shall manage power distribution to subsystems
 * @requirement SRS-EPS-002 System shall implement power mode state machine
 * @requirement SRS-EPS-003 System shall enforce power budget limits
 */

#include "eps_control.h"
#include "fault_mgmt.h"
#include <stdio.h>
#include <string.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Current EPS control state (module-private) */
static EpsControlState_t s_eps_state = {
    .payload_enabled = false,
    .radio_enabled = true,
    .adcs_enabled = true,
    .beacon_enabled = true,
    .power_mode = POWER_MODE_IDLE,
    .power_limit_w = EPS_POWER_LIMIT_IDLE_W,
    .last_control_ms = 0,
    .crc32 = 0
};

/** Flag indicating if module is initialized */
static bool s_initialized = false;

/*===========================================================================*/
/* Internal Functions                                                         */
/*===========================================================================*/

/**
 * @brief Update CRC for EPS state
 */
static void eps_update_crc(void)
{
    /* Calculate CRC over all fields except the CRC itself */
    size_t crc_offset = offsetof(EpsControlState_t, crc32);
    s_eps_state.crc32 = smart_qso_crc32(&s_eps_state, crc_offset);
}

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

SmartQsoResult_t eps_init(void)
{
    SmartQsoResult_t result = eps_load_config();

    if (result != SMART_QSO_OK) {
        /* Use default configuration */
        s_eps_state.payload_enabled = false;
        s_eps_state.radio_enabled = true;
        s_eps_state.adcs_enabled = true;
        s_eps_state.beacon_enabled = true;
        s_eps_state.power_mode = POWER_MODE_IDLE;
        s_eps_state.power_limit_w = EPS_POWER_LIMIT_IDLE_W;
        s_eps_state.last_control_ms = (uint32_t)smart_qso_now_ms();
        eps_update_crc();

        printf("[EPS] Using default configuration\n");
    }

    s_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t eps_get_state(EpsControlState_t *state)
{
    SMART_QSO_REQUIRE_NOT_NULL(state);

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    *state = s_eps_state;
    return SMART_QSO_OK;
}

SmartQsoResult_t eps_control_payload(bool enable, double soc)
{
    SMART_QSO_REQUIRE(soc >= 0.0 && soc <= 1.0, "SOC out of range");

    if (s_eps_state.payload_enabled == enable) {
        return SMART_QSO_OK;
    }

    /* Check power constraints for enable */
    if (enable && soc < EPS_SOC_PAYLOAD_THRESHOLD) {
        fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_WARNING,
                      "Payload enable blocked: low SOC", soc);
        return SMART_QSO_ERROR;
    }

    s_eps_state.payload_enabled = enable;
    s_eps_state.last_control_ms = (uint32_t)smart_qso_now_ms();
    eps_update_crc();

    if (enable) {
        fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_INFO,
                      "Payload enabled", soc);
        printf("[EPS] Payload enabled\n");
    } else {
        fault_log_add(FAULT_TYPE_POWER, FAULT_SEVERITY_INFO,
                      "Payload disabled", soc);
        printf("[EPS] Payload disabled\n");
    }

    (void)eps_save_config();
    return SMART_QSO_OK;
}

SmartQsoResult_t eps_control_radio(bool enable)
{
    if (s_eps_state.radio_enabled == enable) {
        return SMART_QSO_OK;
    }

    s_eps_state.radio_enabled = enable;
    s_eps_state.last_control_ms = (uint32_t)smart_qso_now_ms();
    eps_update_crc();

    printf("[EPS] Radio %s\n", enable ? "enabled" : "disabled");

    (void)eps_save_config();
    return SMART_QSO_OK;
}

SmartQsoResult_t eps_control_adcs(bool enable)
{
    if (s_eps_state.adcs_enabled == enable) {
        return SMART_QSO_OK;
    }

    s_eps_state.adcs_enabled = enable;
    s_eps_state.last_control_ms = (uint32_t)smart_qso_now_ms();
    eps_update_crc();

    printf("[EPS] ADCS %s\n", enable ? "enabled" : "disabled");

    (void)eps_save_config();
    return SMART_QSO_OK;
}

SmartQsoResult_t eps_control_beacon(bool enable)
{
    if (s_eps_state.beacon_enabled == enable) {
        return SMART_QSO_OK;
    }

    s_eps_state.beacon_enabled = enable;
    s_eps_state.last_control_ms = (uint32_t)smart_qso_now_ms();
    eps_update_crc();

    printf("[EPS] Beacon %s\n", enable ? "enabled" : "disabled");

    (void)eps_save_config();
    return SMART_QSO_OK;
}

SmartQsoResult_t eps_set_power_mode(PowerMode_t mode, double soc)
{
    SMART_QSO_REQUIRE(mode <= POWER_MODE_ACTIVE, "Invalid power mode");

    if (s_eps_state.power_mode == mode) {
        return SMART_QSO_OK;
    }

    s_eps_state.power_mode = (uint8_t)mode;
    s_eps_state.last_control_ms = (uint32_t)smart_qso_now_ms();

    switch (mode) {
        case POWER_MODE_SAFE:
            (void)eps_control_payload(false, soc);
            (void)eps_control_radio(false);
            (void)eps_control_adcs(false);
            (void)eps_control_beacon(true);  /* Keep beacon for emergency */
            s_eps_state.power_limit_w = EPS_POWER_LIMIT_SAFE_W;
            fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_WARNING,
                          "Entered SAFE mode", soc);
            break;

        case POWER_MODE_IDLE:
            (void)eps_control_payload(false, soc);
            (void)eps_control_radio(true);
            (void)eps_control_adcs(true);
            (void)eps_control_beacon(true);
            s_eps_state.power_limit_w = EPS_POWER_LIMIT_IDLE_W;
            fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_INFO,
                          "Entered IDLE mode", soc);
            break;

        case POWER_MODE_ACTIVE:
            (void)eps_control_payload(true, soc);
            (void)eps_control_radio(true);
            (void)eps_control_adcs(true);
            (void)eps_control_beacon(true);
            s_eps_state.power_limit_w = EPS_POWER_LIMIT_ACTIVE_W;
            fault_log_add(FAULT_TYPE_MODE_CHANGE, FAULT_SEVERITY_INFO,
                          "Entered ACTIVE mode", soc);
            break;

        default:
            /* Should not reach here due to precondition check */
            return SMART_QSO_ERROR_INVALID;
    }

    eps_update_crc();
    (void)eps_save_config();
    return SMART_QSO_OK;
}

PowerMode_t eps_get_power_mode(void)
{
    return (PowerMode_t)s_eps_state.power_mode;
}

bool eps_is_payload_enabled(void)
{
    return s_eps_state.payload_enabled;
}

double eps_get_power_limit(void)
{
    return s_eps_state.power_limit_w;
}

SmartQsoResult_t eps_save_config(void)
{
    FILE *f = fopen(EPS_CONFIG_FILE, "wb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    eps_update_crc();
    size_t written = fwrite(&s_eps_state, sizeof(EpsControlState_t), 1, f);

    if (fclose(f) != 0) {
        return SMART_QSO_ERROR_IO;
    }

    if (written != 1) {
        return SMART_QSO_ERROR_IO;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t eps_load_config(void)
{
    FILE *f = fopen(EPS_CONFIG_FILE, "rb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    EpsControlState_t loaded_state;
    size_t read_count = fread(&loaded_state, sizeof(EpsControlState_t), 1, f);

    if (fclose(f) != 0) {
        return SMART_QSO_ERROR_IO;
    }

    if (read_count != 1) {
        return SMART_QSO_ERROR_IO;
    }

    /* Verify CRC */
    size_t crc_offset = offsetof(EpsControlState_t, crc32);
    uint32_t calculated_crc = smart_qso_crc32(&loaded_state, crc_offset);

    if (calculated_crc != loaded_state.crc32) {
        printf("[EPS] CRC mismatch in config file, using defaults\n");
        return SMART_QSO_ERROR;
    }

    s_eps_state = loaded_state;
    return SMART_QSO_OK;
}
