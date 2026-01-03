/**
 * @file main.c
 * @brief SMART-QSO Flight Software Main Entry Point
 *
 * This is the main entry point for the SMART-QSO CubeSat flight software.
 * It initializes all subsystems and runs the main control loop.
 *
 * @requirement SRS-SYS-001 System shall initialize all subsystems on startup
 * @requirement SRS-SYS-002 System shall run main control loop
 * @requirement SRS-SYS-003 System shall handle graceful shutdown
 *
 * @copyright NASA Ames Research Center Small Satellite Division Heritage
 * @note Following GeneSat/PharmaSat/O/OREOS/EcAMSat design patterns
 */

/* Required for nanosleep and other POSIX functions */
#define _XOPEN_SOURCE 600

#include "smart_qso.h"
#include "eps_control.h"
#include "fault_mgmt.h"
#include "sensors.h"
#include "uart_comm.h"
#include "mission_data.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <time.h>

/*===========================================================================*/
/* Configuration Constants                                                    */
/*===========================================================================*/

/** Main loop iteration count (for simulation) */
#define MAIN_LOOP_ITERATIONS 1000

/** Main loop sleep time in nanoseconds (~20ms) */
#define MAIN_LOOP_SLEEP_NS 20000000L

/** Health check interval in milliseconds */
#define HEALTH_CHECK_INTERVAL_MS 10000

/** Data persistence interval in ticks */
#define PERSISTENCE_INTERVAL_TICKS 100

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Flag for graceful shutdown */
static volatile sig_atomic_t s_shutdown_requested = 0;

/** Program start time */
static uint64_t s_program_start_ms = 0;

/** Last telemetry transmission time */
static uint64_t s_last_telemetry_ms = 0;

/** Last health check time */
static uint64_t s_last_health_check_ms = 0;

/** Last watchdog reset time */
static uint64_t s_last_watchdog_reset_ms = 0;

/** Sunlit state (for simulation) */
static bool s_sunlit = true;

/** State of charge (for simulation) */
static double s_soc = 0.75;

/*===========================================================================*/
/* Signal Handlers                                                            */
/*===========================================================================*/

/**
 * @brief Signal handler for graceful shutdown
 */
static void signal_handler(int sig)
{
    (void)sig;
    s_shutdown_requested = 1;
}

/*===========================================================================*/
/* Initialization Functions                                                   */
/*===========================================================================*/

/**
 * @brief Initialize system
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
static SmartQsoResult_t init_system(void)
{
    SmartQsoResult_t result;

    /* Set up signal handlers */
    if (signal(SIGINT, signal_handler) == SIG_ERR) {
        fprintf(stderr, "[SYSTEM] Failed to set SIGINT handler\n");
    }
    if (signal(SIGTERM, signal_handler) == SIG_ERR) {
        fprintf(stderr, "[SYSTEM] Failed to set SIGTERM handler\n");
    }

    /* Seed random number generator (for simulation) */
    srand((unsigned int)time(NULL));

    /* Record start time */
    s_program_start_ms = smart_qso_now_ms();

    /* Initialize fault management first (for logging during init) */
    result = fault_mgmt_init();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Fault management init failed\n");
        /* Continue anyway - fault logging won't work but system can run */
    }

    /* Initialize mission data */
    result = mission_data_init();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Mission data init failed\n");
        /* Continue with defaults */
    }

    /* Initialize EPS control */
    result = eps_init();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] EPS init failed\n");
        /* Continue with defaults */
    }

    /* Initialize sensors */
    result = sensors_init();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Sensor init failed\n");
        return result;
    }

    /* Load sensor configuration */
    const char *yaml_path = getenv("SENSORS_YAML");
    if (yaml_path == NULL) {
        yaml_path = "software/flight/sensors.yaml";
    }

    result = sensors_load_yaml(yaml_path);
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[WARN] Using default sensors (could not load %s)\n", yaml_path);
        result = sensors_load_defaults();
        if (result != SMART_QSO_OK) {
            fprintf(stderr, "[SYSTEM] Failed to load default sensors\n");
            return result;
        }
    }

    /* Initialize UART for Jetson communication */
    result = uart_init();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[WARN] UART initialization failed, telemetry disabled\n");
        fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_WARNING,
                      "UART initialization failed", s_soc);
    }

    /* Initialize timing */
    s_last_telemetry_ms = s_program_start_ms;
    s_last_health_check_ms = s_program_start_ms;
    s_last_watchdog_reset_ms = s_program_start_ms;

    return SMART_QSO_OK;
}

/*===========================================================================*/
/* Mission State Management                                                   */
/*===========================================================================*/

/**
 * @brief Update mission state (orbital simulation)
 *
 * @param tick Current tick count
 */
static void update_mission_state(uint64_t tick)
{
    /* Simulate ~55% sunlit orbital cycle */
    s_sunlit = (tick % 180) < 100;

    /* Update sensor environment */
    sensors_set_environment(s_sunlit, s_soc);

    /* Simple SOC integration */
    double discharge = eps_is_payload_enabled() ? 0.0040 : 0.0010;
    double charge = 0.0020;

    if (s_sunlit) {
        s_soc += charge;
    } else {
        s_soc -= discharge;
    }

    /* Clamp SOC */
    if (s_soc > 1.0) {
        s_soc = 1.0;
    }
    if (s_soc < 0.0) {
        s_soc = 0.0;
    }

    /* Power management with fault recovery */
    PowerMode_t current_mode = eps_get_power_mode();

    if (s_soc < EPS_SOC_SAFE_THRESHOLD) {
        /* Critical power condition - enter SAFE mode */
        if (current_mode != POWER_MODE_SAFE) {
            (void)eps_set_power_mode(POWER_MODE_SAFE, s_soc);
        }
    } else if (s_soc < EPS_SOC_IDLE_THRESHOLD) {
        /* Low power condition - enter IDLE mode */
        if (current_mode > POWER_MODE_IDLE) {
            (void)eps_set_power_mode(POWER_MODE_IDLE, s_soc);
        }
    } else if (s_sunlit && s_soc > EPS_SOC_PAYLOAD_THRESHOLD && current_mode < POWER_MODE_ACTIVE) {
        /* Good conditions - can enter ACTIVE mode */
        (void)eps_set_power_mode(POWER_MODE_ACTIVE, s_soc);
    }

    /* Watchdog management */
    uint64_t now = smart_qso_now_ms();
    if ((now - s_last_watchdog_reset_ms) > SMART_QSO_WATCHDOG_TIMEOUT_MS) {
        (void)fault_handle_watchdog(s_soc);
        s_last_watchdog_reset_ms = now;
    }

    /* Periodic fault recovery checks */
    if (tick % 50 == 0) {
        if (current_mode == POWER_MODE_SAFE && s_soc > EPS_SOC_IDLE_THRESHOLD) {
            /* Try to recover from SAFE mode */
            (void)eps_set_power_mode(POWER_MODE_IDLE, s_soc);
            fault_log_add(FAULT_TYPE_RECOVERY, FAULT_SEVERITY_INFO,
                          "Recovered from SAFE mode", s_soc);
        }
    }

    /* Update mission data */
    (void)mission_data_update_uptime(now - s_program_start_ms);
    if (s_sunlit && eps_is_payload_enabled()) {
        /* Rough energy calculation */
        (void)mission_data_add_energy(discharge * 3.7 / 3600.0);
    }
}

/*===========================================================================*/
/* Health Monitoring                                                          */
/*===========================================================================*/

/**
 * @brief Monitor system health and trigger fault recovery
 */
static void monitor_system_health(void)
{
    uint64_t now = smart_qso_now_ms();

    if ((now - s_last_health_check_ms) < HEALTH_CHECK_INTERVAL_MS) {
        return;
    }
    s_last_health_check_ms = now;

    /* Check for thermal issues */
    Sensor_t sensor;
    for (size_t i = 0; i < sensors_get_count(); i++) {
        if (sensors_get(i, &sensor) == SMART_QSO_OK) {
            if (strcmp(sensor.type, "eps_temperature") == 0 && sensor.last_value > 40.0) {
                fault_log_add(FAULT_TYPE_HEALTH_THERMAL, FAULT_SEVERITY_WARNING,
                              "High temperature in health check", s_soc);
                (void)fault_recover_thermal(s_soc);
            }
        }
    }

    /* Check for power issues */
    if (s_soc < 0.30) {
        fault_log_add(FAULT_TYPE_HEALTH_POWER, FAULT_SEVERITY_WARNING,
                      "Low SOC in health check", s_soc);
        (void)fault_recover_power(s_soc);
    }

    /* Check for communication issues */
    if (eps_is_payload_enabled() && !uart_is_initialized()) {
        fault_log_add(FAULT_TYPE_HEALTH_COMM, FAULT_SEVERITY_WARNING,
                      "UART communication lost with payload", s_soc);
        (void)uart_init();
    }

    /* Reset watchdog */
    s_last_watchdog_reset_ms = now;
}

/*===========================================================================*/
/* Telemetry Functions                                                        */
/*===========================================================================*/

/**
 * @brief Send telemetry to Jetson
 */
static void send_telemetry_to_jetson(void)
{
    if (!eps_is_payload_enabled() || !uart_is_initialized()) {
        return;
    }

    char buffer[UART_TX_BUFFER_SIZE];
    int offset = 0;

    /* Format header with mission state */
    offset = snprintf(buffer, sizeof(buffer),
                      "TELEMETRY,%llu,%s,%.3f,%d,%.1f,",
                      (unsigned long long)smart_qso_now_ms(),
                      s_sunlit ? "SUNLIT" : "ECLIPSE",
                      s_soc,
                      (int)eps_get_power_mode(),
                      eps_get_power_limit());

    if (offset < 0 || (size_t)offset >= sizeof(buffer)) {
        return;  /* Truncation */
    }

    /* Add sensor values */
    size_t sensor_len = sensors_format_telemetry(buffer + offset,
                                                  sizeof(buffer) - (size_t)offset);
    offset += (int)sensor_len;

    /* Add EPS state */
    EpsControlState_t eps_state;
    if (eps_get_state(&eps_state) == SMART_QSO_OK) {
        int written = snprintf(buffer + offset, sizeof(buffer) - (size_t)offset,
                               "EPS_PAYLOAD=%s,EPS_RADIO=%s,EPS_ADCS=%s,EPS_BEACON=%s,",
                               eps_state.payload_enabled ? "ON" : "OFF",
                               eps_state.radio_enabled ? "ON" : "OFF",
                               eps_state.adcs_enabled ? "ON" : "OFF",
                               eps_state.beacon_enabled ? "ON" : "OFF");
        if (written > 0 && (size_t)written < sizeof(buffer) - (size_t)offset) {
            offset += written;
        }
    }

    /* Add fault information */
    FaultLogEntry_t last_fault;
    if (fault_log_get_last(&last_fault) == SMART_QSO_OK) {
        int written = snprintf(buffer + offset, sizeof(buffer) - (size_t)offset,
                               "LAST_FAULT=%s,FAULT_COUNT=%zu,",
                               last_fault.description,
                               fault_log_get_count());
        if (written > 0 && (size_t)written < sizeof(buffer) - (size_t)offset) {
            offset += written;
        }
    }

    /* Add health status */
    int written = snprintf(buffer + offset, sizeof(buffer) - (size_t)offset,
                           "UART_HEALTH=%s,WATCHDOG=%s",
                           uart_is_initialized() ? "OK" : "FAIL",
                           fault_was_watchdog_triggered() ? "TRIGGERED" : "OK");
    if (written > 0 && (size_t)written < sizeof(buffer) - (size_t)offset) {
        offset += written;
    }

    /* Finalize message */
    if (offset > 0 && buffer[offset - 1] == ',') {
        buffer[offset - 1] = '\n';
    } else if ((size_t)offset < sizeof(buffer) - 1) {
        buffer[offset++] = '\n';
    }

    /* Send via UART */
    if (uart_send(buffer, (size_t)offset) == SMART_QSO_OK) {
        printf("[UART] Sent telemetry to Jetson (%d bytes)\n", offset);
    } else {
        fprintf(stderr, "[UART] Failed to send telemetry to Jetson\n");
    }
}

/*===========================================================================*/
/* Main Loop                                                                  */
/*===========================================================================*/

/**
 * @brief Run main control loop
 *
 * @return SMART_QSO_OK on normal exit
 */
static SmartQsoResult_t run_main_loop(void)
{
    MissionData_t mission;
    (void)mission_data_get(&mission);

    printf("[SYSTEM] SMART-QSO Flight Software Started\n");
    printf("[SYSTEM] Version: %s\n", SMART_QSO_VERSION_STRING);
    printf("[SYSTEM] Mission Phase: %d, Power Mode: %d, SOC: %.1f%%\n",
           mission.mission_phase, (int)eps_get_power_mode(), s_soc * 100.0);

    uint32_t telemetry_interval = uart_get_telemetry_interval();

    /* Main loop */
    for (uint64_t tick = 0; tick < MAIN_LOOP_ITERATIONS && !s_shutdown_requested; ++tick) {
        update_mission_state(tick);
        uint64_t now = smart_qso_now_ms();

        /* System health monitoring */
        monitor_system_health();

        /* Telemetry transmission */
        if (eps_is_payload_enabled() && uart_is_initialized() &&
            (now - s_last_telemetry_ms) >= telemetry_interval) {
            send_telemetry_to_jetson();
            s_last_telemetry_ms = now;
        }

        /* Periodic UART health check */
        if (uart_is_initialized() && (now % 10000) < 100) {
            (void)uart_check_health();
        }

        /* Sensor polling */
        (void)sensors_poll(now);

        /* Periodic data persistence */
        if (tick % PERSISTENCE_INTERVAL_TICKS == 0) {
            (void)mission_data_save();
            (void)eps_save_config();
        }

        /* Sleep */
        struct timespec ts = {.tv_sec = 0, .tv_nsec = MAIN_LOOP_SLEEP_NS};
        nanosleep(&ts, NULL);
    }

    return SMART_QSO_OK;
}

/*===========================================================================*/
/* Shutdown Functions                                                         */
/*===========================================================================*/

/**
 * @brief Shutdown system gracefully
 */
static void shutdown_system(void)
{
    printf("[SYSTEM] Shutting down gracefully...\n");

    /* Save final state */
    (void)mission_data_save();
    (void)eps_save_config();
    (void)fault_log_save();

    /* Close UART */
    (void)uart_close();

    MissionData_t mission;
    (void)mission_data_get(&mission);
    printf("[SYSTEM] Shutdown complete. Total uptime: %llu ms, Faults: %u\n",
           (unsigned long long)mission.total_uptime_ms, mission.fault_count);
}

/*===========================================================================*/
/* Main Entry Point                                                           */
/*===========================================================================*/

/**
 * @brief Main entry point
 *
 * @return 0 on success, non-zero on error
 */
int main(void)
{
    SmartQsoResult_t result;

    result = init_system();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Initialization failed\n");
        return 1;
    }

    result = run_main_loop();
    if (result != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Main loop error\n");
    }

    shutdown_system();

    return 0;
}
