/**
 * @file system_state.c
 * @brief System state management implementation
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
 * @license MIT
 *
 * Implements centralized system state management with accessor functions.
 * All global state is encapsulated per NASA NPR 7150.2 requirements.
 */

#include "system_state.h"
#include "safe_string.h"
#include <stddef.h>

/* For simulation builds, use standard file I/O for persistence */
/* For flight builds, use HAL flash interface instead */
#ifdef SIMULATION_BUILD
#include <stdio.h>
/** Persistence file path (simulation only) */
static const char *STATE_FILE = "/tmp/smart_qso_state.dat";
#else
#include "hal/hal_flash.h"
#endif

/*******************************************************************************
 * Private Data
 ******************************************************************************/

/** Single instance of system state - all access through accessors */
static SystemState_t s_state;

/*******************************************************************************
 * Private Functions
 ******************************************************************************/

/**
 * @brief Mark state as dirty (needs saving)
 */
static void mark_dirty(void)
{
    s_state.persistence_dirty = true;
    s_state.last_update_ms = smart_qso_now_ms();
}

/*******************************************************************************
 * Initialization Functions
 ******************************************************************************/

SmartQsoResult_t sys_state_init(void)
{
    /* Clear all state to zero */
    (void)safe_memset(&s_state, sizeof(s_state), 0, sizeof(s_state));

    /* Initialize power state defaults */
    s_state.power.battery_voltage = 3.7;
    s_state.power.state_of_charge = 0.5;
    s_state.power.power_mode = POWER_MODE_SAFE;
    s_state.power.payload_enabled = false;

    /* Initialize thermal state defaults */
    s_state.thermal.obc_temp_c = 25.0f;
    s_state.thermal.eps_temp_c = 25.0f;
    s_state.thermal.battery_temp_c = 25.0f;
    s_state.thermal.payload_temp_c = 25.0f;
    s_state.thermal.external_temp_c = 0.0f;

    /* Initialize communications defaults */
    s_state.comm.beacon_interval_s = 60;

    /* Initialize mission state */
    s_state.mission.boot_count = 0;

    /* Initialize state machine */
    SmartQsoResult_t result = sm_init(&s_state.sm_context, STATE_BOOT);
    if (result != SMART_QSO_OK) {
        return result;
    }

    /* Mark as initialized */
    s_state.initialized = true;
    s_state.watchdog_ok = true;
    s_state.persistence_dirty = false;
    s_state.last_update_ms = smart_qso_now_ms();

    /* Calculate initial CRC */
    (void)sys_state_update_crc();

    return SMART_QSO_OK;
}

SmartQsoResult_t sys_state_load(void)
{
#ifdef SIMULATION_BUILD
    FILE *fp = fopen(STATE_FILE, "rb");
    if (fp == NULL) {
        /* No saved state, use defaults */
        return sys_state_init();
    }

    SystemState_t loaded_state;
    size_t read_count = fread(&loaded_state, sizeof(loaded_state), 1, fp);
    (void)fclose(fp);

    if (read_count != 1) {
        /* Read failed, use defaults */
        return sys_state_init();
    }
#else
    /* Flight build: use HAL flash interface */
    SystemState_t loaded_state;
    SmartQsoResult_t flash_result = hal_flash_read(
        FLASH_REGION_STATE,
        0,
        (uint8_t *)&loaded_state,
        sizeof(loaded_state));

    if (flash_result != SMART_QSO_OK) {
        /* No saved state or read error, use defaults */
        return sys_state_init();
    }
#endif

    /* Verify CRC before accepting loaded state */
    uint32_t saved_crc = loaded_state.crc32;
    loaded_state.crc32 = 0;
    uint32_t calc_crc = smart_qso_crc32(&loaded_state, sizeof(loaded_state) - sizeof(uint32_t));

    if (saved_crc != calc_crc) {
        /* CRC mismatch, use defaults */
        return sys_state_init();
    }

    /* Accept loaded state */
    (void)safe_memcpy(&s_state, sizeof(s_state), &loaded_state, sizeof(loaded_state));

    /* Re-initialize state machine (context needs fresh init) */
    SmState_t last_state = s_state.sm_context.current_state;
    (void)sm_init(&s_state.sm_context, last_state);

    s_state.initialized = true;
    s_state.persistence_dirty = false;

    return SMART_QSO_OK;
}

SmartQsoResult_t sys_state_save(void)
{
    if (!s_state.initialized) {
        return SMART_QSO_ERROR_INVALID;
    }

    /* Update CRC before saving */
    (void)sys_state_update_crc();

#ifdef SIMULATION_BUILD
    FILE *fp = fopen(STATE_FILE, "wb");
    if (fp == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    size_t written = fwrite(&s_state, sizeof(s_state), 1, fp);
    (void)fclose(fp);

    if (written != 1) {
        return SMART_QSO_ERROR_IO;
    }
#else
    /* Flight build: use HAL flash interface */
    SmartQsoResult_t flash_result = hal_flash_erase(FLASH_REGION_STATE);

    if (flash_result != SMART_QSO_OK) {
        return SMART_QSO_ERROR_IO;
    }

    flash_result = hal_flash_write(
        FLASH_REGION_STATE,
        0,
        (const uint8_t *)&s_state,
        sizeof(s_state));

    if (flash_result != SMART_QSO_OK) {
        return SMART_QSO_ERROR_IO;
    }
#endif

    s_state.persistence_dirty = false;
    s_state.last_persist_ms = smart_qso_now_ms();

    return SMART_QSO_OK;
}

bool sys_state_is_dirty(void)
{
    return s_state.persistence_dirty;
}

void sys_state_clear_dirty(void)
{
    s_state.persistence_dirty = false;
}

/*******************************************************************************
 * Power State Accessors
 ******************************************************************************/

SmartQsoResult_t sys_get_power_state(PowerState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(state, sizeof(*state), &s_state.power, sizeof(s_state.power));
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_battery_voltage(double voltage_v)
{
    s_state.power.battery_voltage = voltage_v;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_battery_current(double current_a)
{
    s_state.power.battery_current = current_a;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_state_of_charge(double soc)
{
    /* Clamp to valid range */
    if (soc < 0.0) {
        soc = 0.0;
    } else if (soc > 1.0) {
        soc = 1.0;
    }

    s_state.power.state_of_charge = soc;
    mark_dirty();
    return SMART_QSO_OK;
}

double sys_get_state_of_charge(void)
{
    return s_state.power.state_of_charge;
}

SmartQsoResult_t sys_set_power_mode(PowerMode_t mode)
{
    s_state.power.power_mode = mode;
    s_state.power.mode_entry_time_ms = (uint32_t)smart_qso_now_ms();
    mark_dirty();
    return SMART_QSO_OK;
}

PowerMode_t sys_get_power_mode(void)
{
    return s_state.power.power_mode;
}

SmartQsoResult_t sys_set_payload_enabled(bool enabled)
{
    s_state.power.payload_enabled = enabled;
    mark_dirty();
    return SMART_QSO_OK;
}

bool sys_is_payload_enabled(void)
{
    return s_state.power.payload_enabled;
}

/*******************************************************************************
 * Thermal State Accessors
 ******************************************************************************/

SmartQsoResult_t sys_get_thermal_state(ThermalState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(state, sizeof(*state), &s_state.thermal, sizeof(s_state.thermal));
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_temperature(uint8_t sensor, float temp_c)
{
    switch (sensor) {
        case 0:
            s_state.thermal.obc_temp_c = temp_c;
            break;
        case 1:
            s_state.thermal.eps_temp_c = temp_c;
            break;
        case 2:
            s_state.thermal.battery_temp_c = temp_c;
            break;
        case 3:
            s_state.thermal.payload_temp_c = temp_c;
            break;
        case 4:
            s_state.thermal.external_temp_c = temp_c;
            break;
        default:
            return SMART_QSO_ERROR_INVALID;
    }

    /* Check thermal limits */
    s_state.thermal.over_temp_flag = (temp_c > 60.0f);
    s_state.thermal.under_temp_flag = (temp_c < -20.0f);

    mark_dirty();
    return SMART_QSO_OK;
}

bool sys_has_thermal_fault(void)
{
    return s_state.thermal.over_temp_flag || s_state.thermal.under_temp_flag;
}

/*******************************************************************************
 * ADCS State Accessors
 ******************************************************************************/

SmartQsoResult_t sys_get_adcs_state(AdcsState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(state, sizeof(*state), &s_state.adcs, sizeof(s_state.adcs));
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_magnetometer(float x, float y, float z)
{
    s_state.adcs.mag_x_ut = x;
    s_state.adcs.mag_y_ut = y;
    s_state.adcs.mag_z_ut = z;
    s_state.adcs.last_update_ms = (uint32_t)smart_qso_now_ms();
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_gyroscope(float x, float y, float z)
{
    s_state.adcs.gyro_x_dps = x;
    s_state.adcs.gyro_y_dps = y;
    s_state.adcs.gyro_z_dps = z;
    s_state.adcs.last_update_ms = (uint32_t)smart_qso_now_ms();
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_detumbled(bool achieved)
{
    s_state.adcs.detumbled = achieved;
    mark_dirty();
    return SMART_QSO_OK;
}

bool sys_is_detumbled(void)
{
    return s_state.adcs.detumbled;
}

/*******************************************************************************
 * Communications State Accessors
 ******************************************************************************/

SmartQsoResult_t sys_get_comm_state(CommState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(state, sizeof(*state), &s_state.comm, sizeof(s_state.comm));
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_increment_packets_sent(void)
{
    s_state.comm.packets_sent++;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_increment_packets_received(void)
{
    s_state.comm.packets_received++;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_increment_beacon_count(void)
{
    s_state.comm.beacon_count++;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_update_ground_contact(void)
{
    s_state.comm.last_ground_contact_ms = (uint32_t)smart_qso_now_ms();
    s_state.comm.comm_active = true;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_set_beacon_interval(uint16_t interval_s)
{
    /* Validate range: 10-120 seconds */
    if ((interval_s < 10U) || (interval_s > 120U)) {
        return SMART_QSO_ERROR_INVALID;
    }

    s_state.comm.beacon_interval_s = interval_s;
    mark_dirty();
    return SMART_QSO_OK;
}

/*******************************************************************************
 * Mission State Accessors
 ******************************************************************************/

SmartQsoResult_t sys_get_mission_state(MissionState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(state, sizeof(*state), &s_state.mission, sizeof(s_state.mission));
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_increment_boot_count(void)
{
    s_state.mission.boot_count++;
    mark_dirty();
    return SMART_QSO_OK;
}

uint32_t sys_get_boot_count(void)
{
    return s_state.mission.boot_count;
}

SmartQsoResult_t sys_update_uptime(void)
{
    static uint64_t s_last_uptime_update = 0;

    uint64_t now = smart_qso_now_ms();
    if (s_last_uptime_update == 0) {
        s_last_uptime_update = now;
    }

    uint64_t elapsed_ms = now - s_last_uptime_update;
    uint32_t elapsed_s = (uint32_t)(elapsed_ms / 1000U);

    if (elapsed_s > 0) {
        s_state.mission.uptime_s += elapsed_s;
        s_state.mission.total_uptime_s += elapsed_s;
        s_state.mission.mission_time_ms += elapsed_ms;
        s_last_uptime_update = now;
        mark_dirty();
    }

    return SMART_QSO_OK;
}

uint32_t sys_get_uptime_s(void)
{
    return s_state.mission.uptime_s;
}

SmartQsoResult_t sys_increment_qso_count(void)
{
    s_state.mission.qso_count++;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_increment_command_count(void)
{
    s_state.mission.command_count++;
    mark_dirty();
    return SMART_QSO_OK;
}

SmartQsoResult_t sys_increment_anomaly_count(void)
{
    s_state.mission.anomaly_count++;
    mark_dirty();
    return SMART_QSO_OK;
}

/*******************************************************************************
 * State Machine Accessors
 ******************************************************************************/

SmContext_t *sys_get_sm_context(void)
{
    return &s_state.sm_context;
}

SmState_t sys_get_operational_state(void)
{
    return s_state.sm_context.current_state;
}

/*******************************************************************************
 * Integrity Functions
 ******************************************************************************/

SmartQsoResult_t sys_state_update_crc(void)
{
    /* Calculate CRC over state excluding the CRC field itself */
    s_state.crc32 = 0;
    s_state.crc32 = smart_qso_crc32(&s_state, sizeof(s_state) - sizeof(uint32_t));
    return SMART_QSO_OK;
}

bool sys_state_verify_crc(void)
{
    uint32_t saved_crc = s_state.crc32;
    s_state.crc32 = 0;
    uint32_t calc_crc = smart_qso_crc32(&s_state, sizeof(s_state) - sizeof(uint32_t));
    s_state.crc32 = saved_crc;

    return (saved_crc == calc_crc);
}

SmartQsoResult_t sys_get_full_state(SystemState_t *state)
{
    if (state == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }

    (void)safe_memcpy(state, sizeof(*state), &s_state, sizeof(s_state));
    return SMART_QSO_OK;
}
