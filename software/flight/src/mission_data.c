/**
 * @file mission_data.c
 * @brief Mission Data Persistence Implementation
 *
 * @requirement SRS-F060 Persist mission data to non-volatile memory
 * @requirement SRS-F061 Protect persisted data with CRC32
 * @requirement SRS-F063 Maintain mission statistics across resets
 */

#include "mission_data.h"
#include "smart_qso.h"
#include <inttypes.h>
#include <stdlib.h>

#include <stdio.h>
#include <string.h>
#include <stddef.h>
#include <stdint.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Current mission data (module-private) */
static MissionData_t s_mission_data = {0};

/** Flag indicating if module is initialized */
static bool s_initialized = false;

/*===========================================================================*/
/* Internal Functions                                                         */
/*===========================================================================*/

/**
 * @brief Update CRC for mission data
 */
static void mission_data_update_crc(void)
{
    size_t crc_offset = offsetof(MissionData_t, crc32);
    s_mission_data.crc32 = smart_qso_crc32(&s_mission_data, crc_offset);
}

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

SmartQsoResult_t mission_data_init(void)
{
    SmartQsoResult_t result = mission_data_load();

    if (result != SMART_QSO_OK) {
        /* First time running - initialize mission data */
        uint64_t now = smart_qso_now_ms();
        s_mission_data.mission_start_ms = now;
        s_mission_data.total_uptime_ms = 0;
        s_mission_data.reset_count = 1;
        s_mission_data.last_reset_ms = now;
        s_mission_data.total_energy_wh = 0.0;
        s_mission_data.fault_count = 0;
        s_mission_data.mission_phase = MISSION_PHASE_IDLE;
        (void)strncpy(s_mission_data.last_fault, "System startup",
                      sizeof(s_mission_data.last_fault) - 1);
        s_mission_data.last_fault[sizeof(s_mission_data.last_fault) - 1] = '\0';
        mission_data_update_crc();

        (void)printf("[MISSION] Initialized new mission data\n");
    } else {
        /* Update for this session */
        s_mission_data.reset_count++;
        s_mission_data.last_reset_ms = smart_qso_now_ms();
        mission_data_update_crc();

        (void)printf("[MISSION] Loaded mission data (reset #%" PRIu32 ")\n",
                     s_mission_data.reset_count);
    }

    s_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_get(MissionData_t *data)
{
    SMART_QSO_REQUIRE_NOT_NULL(data);

    if (!s_initialized) {
        return SMART_QSO_ERROR;
    }

    *data = s_mission_data;
    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_set_start(uint64_t start_ms)
{
    s_mission_data.mission_start_ms = start_ms;
    mission_data_update_crc();
    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_update_uptime(uint64_t uptime_ms)
{
    s_mission_data.total_uptime_ms = uptime_ms;
    mission_data_update_crc();
    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_increment_reset(void)
{
    s_mission_data.reset_count++;
    s_mission_data.last_reset_ms = smart_qso_now_ms();
    mission_data_update_crc();
    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_add_energy(double energy_wh)
{
    s_mission_data.total_energy_wh += energy_wh;
    mission_data_update_crc();
    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_set_phase(MissionPhase_t phase)
{
    SMART_QSO_REQUIRE(phase <= MISSION_PHASE_EOL, "Invalid mission phase");

    s_mission_data.mission_phase = (uint8_t)phase;
    mission_data_update_crc();
    return SMART_QSO_OK;
}

MissionPhase_t mission_data_get_phase(void)
{
    return (MissionPhase_t)s_mission_data.mission_phase;
}

SmartQsoResult_t mission_data_record_fault(const char *description)
{
    SMART_QSO_REQUIRE_NOT_NULL(description);

    s_mission_data.fault_count++;
    (void)strncpy(s_mission_data.last_fault, description,
                  sizeof(s_mission_data.last_fault) - 1);
    s_mission_data.last_fault[sizeof(s_mission_data.last_fault) - 1] = '\0';
    mission_data_update_crc();

    return SMART_QSO_OK;
}

/** Persistent mission data file: MISSION_DATA_FILE unless overridden */
static const char *mission_data_path(void)
{
    const char *path = getenv(MISSION_DATA_FILE_ENV);
    return ((path != NULL) && (path[0] != '\0')) ? path : MISSION_DATA_FILE;
}

SmartQsoResult_t mission_data_save(void)
{
    FILE *f = fopen(mission_data_path(), "wb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    mission_data_update_crc();
    size_t written = fwrite(&s_mission_data, sizeof(MissionData_t), 1, f);

    if (fclose(f) != 0) {
        return SMART_QSO_ERROR_IO;
    }

    if (written != 1) {
        return SMART_QSO_ERROR_IO;
    }

    return SMART_QSO_OK;
}

SmartQsoResult_t mission_data_load(void)
{
    FILE *f = fopen(mission_data_path(), "rb");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    MissionData_t loaded_data;
    size_t read_count = fread(&loaded_data, sizeof(MissionData_t), 1, f);

    if (fclose(f) != 0) {
        return SMART_QSO_ERROR_IO;
    }

    if (read_count != 1) {
        return SMART_QSO_ERROR_IO;
    }

    /* Verify CRC */
    size_t crc_offset = offsetof(MissionData_t, crc32);
    uint32_t calculated_crc = smart_qso_crc32(&loaded_data, crc_offset);

    if (calculated_crc != loaded_data.crc32) {
        (void)printf("[MISSION] CRC mismatch in mission data, using defaults\n");
        return SMART_QSO_ERROR;
    }

    s_mission_data = loaded_data;
    return SMART_QSO_OK;
}

uint32_t mission_data_get_reset_count(void)
{
    return s_mission_data.reset_count;
}

uint64_t mission_data_get_uptime(void)
{
    return s_mission_data.total_uptime_ms;
}

double mission_data_get_energy(void)
{
    return s_mission_data.total_energy_wh;
}
