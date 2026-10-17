/**
 * @file main.c
 * @brief SMART-QSO Flight Software Main Entry Point
 *
 * This is the main entry point for the SMART-QSO CubeSat flight software.
 * It initializes all subsystems and runs the main control loop.
 *
 * @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on
 * @requirement SRS-P001 Main control loop executes within 20 ms
 * @note Host simulation only (not a flight requirement): SIGINT/SIGTERM end the loop; state is saved
 *
 * @copyright Copyright (c) 2026 SMART-QSO Team
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
#include "rf_control.h"
#include "contact_monitor.h"
#include "rf_silence.h"
#include "deployment.h"
#include "jetson_link.h"
#include "beacon.h"
#include "watchdog_mgr.h"
#include "payload_guard.h"
#include "system_state.h"
#include "state_machine.h"
#include "cmd_handler.h"
#include "gse_provision.h"
#include "uplink.h"
#include "station_id.h"
#include "hal/hal_gpio.h"
#include "hal/hal_flash.h"
#include "hal/hal_timer.h"

#include <inttypes.h>
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

/** Last loss-of-contact tick */
static uint64_t s_last_contact_tick_ms = 0;

/** Sunlit state (for simulation) */
static bool s_sunlit = true;

/** State of charge (for simulation) */
static double s_soc = 0.75;

/*===========================================================================*/
/* Signal Handlers                                                            */
/*===========================================================================*/

#ifndef HAL_TARGET_STM32L4
/**
 * @brief Signal handler for graceful shutdown (host builds; the target has no signals)
 */
static void signal_handler(int sig)
{
    (void)sig;
    s_shutdown_requested = 1;
}
#endif

/*===========================================================================*/
/* Initialization Functions                                                   */
/*===========================================================================*/

/**
 * @brief Watchdog pre-reset action: persist state before the hardware resets
 */
static void save_state_before_reset(void)
{
    (void)mission_data_save();
    (void)eps_save_config();
}

/**
 * @brief Arm the supervised hardware watchdog (SW-15)
 *
 * The hardware watchdog is refreshed only while every stage of the main loop
 * checks in (wdt_mgr_kick). A stalled stage stops the refresh and the
 * hardware resets the OBC after saving state. On the target each stage
 * becomes an RTOS task (SW-01).
 */
static SmartQsoResult_t init_watchdog(void)
{
    if (wdt_mgr_init() != SMART_QSO_OK) {
        return SMART_QSO_ERROR;
    }
    (void)wdt_mgr_register_task(WDT_TASK_MAIN_LOOP, "main", 1000U);
    (void)wdt_mgr_register_task(WDT_TASK_COMM, "uplink", 1000U);
    (void)wdt_mgr_register_task(WDT_TASK_PAYLOAD, "payload", 1000U);
    (void)wdt_mgr_register_task(WDT_TASK_BEACON, "beacon", 1000U);
    (void)wdt_mgr_register_task(WDT_TASK_HEALTH, "health", 2U * HEALTH_CHECK_INTERVAL_MS);
    (void)wdt_mgr_register_prereset_callback(save_state_before_reset);
    return wdt_mgr_start();
}

/**
 * @brief Initialize system
 *
 * @return SMART_QSO_OK on success, error code otherwise
 */
/**
 * @brief Radio safety and telecommand: before anything can transmit
 *
 * Post-deployment silence (CDS 2.4.5) and the loss-of-contact inhibit come
 * first, so the radio lines are never driven high on a boot that must stay
 * silent; then the latched TX-OFF state (47 CFR 97.207(b)), the key table
 * (with no key only TX-OFF is accepted), the receiver and the GSE port.
 */
static void init_rf_and_telecommand(void)
{
    (void)rf_silence_init();
    (void)contact_monitor_init();
    (void)rf_control_init();

    if (cmd_handler_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Command handler init failed\n");
    }
    if (uplink_init(SQSO_CALLSIGN, SQSO_SSID) != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Telecommand receiver init failed\n");
    }
    if (gse_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] GSE provisioning port init failed\n");
    }
}

/**
 * @brief Mission data, EPS and sensors (sensors.yaml, else the defaults)
 */
static SmartQsoResult_t init_power_and_sensors(void)
{
    if (mission_data_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Mission data init failed\n");
    }
    if (eps_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] EPS init failed\n");
    }
    if (sensors_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Sensor init failed\n");
        return SMART_QSO_ERROR;
    }

#ifdef HAL_TARGET_STM32L4
    /* No file system on the target: the built-in sensor table */
    if (sensors_load_defaults() != SMART_QSO_OK) {
        return SMART_QSO_ERROR;
    }
#else
    const char *yaml_path = getenv("SENSORS_YAML");
    if (yaml_path == NULL) {
        yaml_path = "software/flight/sensors.yaml";
    }
    if (sensors_load_yaml(yaml_path) != SMART_QSO_OK) {
        fprintf(stderr, "[WARN] Using default sensors (could not load %s)\n", yaml_path);
        if (sensors_load_defaults() != SMART_QSO_OK) {
            fprintf(stderr, "[SYSTEM] Failed to load default sensors\n");
            return SMART_QSO_ERROR;
        }
    }
#endif
    return SMART_QSO_OK;
}

/**
 * @brief Operational state (one mode manager, SW-14): BOOT -> DETUMBLE -> IDLE
 *
 * The host loop has no ADCS, so detumble completes at once; on the target
 * the ADCS raises EVENT_DETUMBLE_COMPLETE.
 */
static SmartQsoResult_t init_mode_manager(void)
{
    (void)beacon_init();   /* templates until the Jetson's text passes the filter */
    payload_guard_init();
    eps_update_soc(s_soc);
    if (sys_state_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] System state init failed\n");
        return SMART_QSO_ERROR;
    }
    (void)sm_process_event(sys_get_sm_context(), EVENT_BOOT_COMPLETE, NULL);
    (void)sm_process_event(sys_get_sm_context(), EVENT_DETUMBLE_COMPLETE, NULL);
    return SMART_QSO_OK;
}

static SmartQsoResult_t init_system(void)
{
#ifndef HAL_TARGET_STM32L4
    if (signal(SIGINT, signal_handler) == SIG_ERR) {
        fprintf(stderr, "[SYSTEM] Failed to set SIGINT handler\n");
    }
    if (signal(SIGTERM, signal_handler) == SIG_ERR) {
        fprintf(stderr, "[SYSTEM] Failed to set SIGTERM handler\n");
    }
#endif
#ifdef SIMULATION_BUILD
    srand((unsigned int)time(NULL));   /* simulation noise */
#endif

    /* Time base first: on the target every module's timestamps come from
     * it (SysTick), so nothing may read the time before it runs */
    if (hal_timer_init() != SMART_QSO_OK) {
        (void)fprintf(stderr, "[SYSTEM] Timer init failed\n");
        return SMART_QSO_ERROR;
    }
    s_program_start_ms = smart_qso_now_ms();

    /* HAL before any module touches GPIO or non-volatile memory; burn-wire
     * lines low as early as possible; fault logging before the rest */
    if ((hal_gpio_init() != SMART_QSO_OK) || (hal_flash_init() != SMART_QSO_OK)) {
        fprintf(stderr, "[SYSTEM] GPIO/flash init failed\n");
    }
    (void)deployment_init();
    if (fault_mgmt_init() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Fault management init failed\n");
    }

    init_rf_and_telecommand();
    if ((init_power_and_sensors() != SMART_QSO_OK) || (init_mode_manager() != SMART_QSO_OK)) {
        return SMART_QSO_ERROR;
    }
    if (init_watchdog() != SMART_QSO_OK) {
        fprintf(stderr, "[SYSTEM] Watchdog init failed\n");
        return SMART_QSO_ERROR;
    }

    if (uart_init() != SMART_QSO_OK) {
        fprintf(stderr, "[WARN] UART initialization failed, telemetry disabled\n");
        fault_log_add(FAULT_TYPE_INIT, FAULT_SEVERITY_WARNING,
                      "UART initialization failed", s_soc);
    }

    s_last_telemetry_ms = s_program_start_ms;
    s_last_health_check_ms = s_program_start_ms;
    s_last_contact_tick_ms = s_program_start_ms;
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

    /* Mode: the EPS holds the measured SOC; the state machine turns power
     * conditions into transitions, and its entry actions set the EPS power
     * mode (SW-14). In this simulation s_soc stands in for the EPS gauge */
    eps_update_soc(s_soc);
    SmState_t state = STATE_BOOT;
    if (sm_get_state(sys_get_sm_context(), &state) == SMART_QSO_OK) {
        SmEvent_t event = sm_power_event(state, s_soc, s_sunlit);
        if (event != EVENT_NONE) {
            (void)sm_process_event(sys_get_sm_context(), event, NULL);
        }
    }

    uint64_t now = smart_qso_now_ms();

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

static bool fresh_sensor(const char *id, uint64_t now, uint32_t max_age_ms, double *value);

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
                (void)sm_process_event(sys_get_sm_context(), EVENT_THERMAL_FAULT, NULL);
            }
        }
    }

    /* Bus regulation, on the measured value (SW-16) */
    double bus_v = 0.0;
    if (fresh_sensor("BUSV", now, HEALTH_CHECK_INTERVAL_MS, &bus_v) && !eps_bus_voltage_ok(bus_v)) {
        fault_log_add(FAULT_TYPE_VOLTAGE_RANGE, FAULT_SEVERITY_ERROR,
                      "Bus voltage out of range", s_soc);
    }

    /* Check for power issues */
    if (s_soc < 0.30) {
        fault_log_add(FAULT_TYPE_HEALTH_POWER, FAULT_SEVERITY_WARNING,
                      "Low SOC in health check", s_soc);  /* mode change: update_mission_state */
    }

    /* Check for communication issues */
    if (eps_is_payload_enabled() && !uart_is_initialized()) {
        fault_log_add(FAULT_TYPE_HEALTH_COMM, FAULT_SEVERITY_WARNING,
                      "UART communication lost with payload", s_soc);
        (void)uart_init();
    }

    (void)wdt_mgr_checkin(WDT_TASK_HEALTH);
}

/*===========================================================================*/
/* Telemetry Functions                                                        */
/*===========================================================================*/

/**
 * @brief Send telemetry to Jetson
 */
/** Latest Jetson STATUS temperature (junction, as the Jetson reports it) */
static int32_t s_jetson_temp_c = 0;
static uint64_t s_jetson_status_ms = 0U;
static bool s_jetson_status_seen = false;

/** Beacon text most recently received from the Jetson (empty: use a template) */
static char s_ai_text[JETSON_TEXT_MAX + 1U];
static uint16_t s_beacon_seq = 0U;
static JetsonRx_t s_jetson_rx;

/** Replace characters that would break the ICD's comma-separated fields */
static void sanitize_field(char *text)
{
    for (char *p = text; *p != '\0'; p++) {
        if ((*p == ',') || (*p < 0x20) || (*p > 0x7E)) {
            *p = ';';
        }
    }
}

static void send_line(const char *line, size_t len, const char *what)
{
    if (len == 0U) {
        fprintf(stderr, "[UART] %s line not built\n", what);
    } else if (uart_send(line, len) != SMART_QSO_OK) {
        fprintf(stderr, "[UART] Failed to send %s to Jetson\n", what);
    } else {
        printf("[UART] Sent %s to Jetson (%zu bytes)\n", what, len);
    }
}

/** Append ",key=value" fields for EPS switches, last fault and health */
static int append_status_fields(char *buffer, size_t room, int offset)
{
    /* EPS state */
    EpsControlState_t eps_state;
    if (eps_get_state(&eps_state) == SMART_QSO_OK) {
        int written = snprintf(buffer + offset, room - (size_t)offset,
                               "EPS_PAYLOAD=%s,EPS_RADIO=%s,EPS_ADCS=%s,EPS_BEACON=%s,",
                               eps_state.payload_enabled ? "ON" : "OFF",
                               eps_state.radio_enabled ? "ON" : "OFF",
                               eps_state.adcs_enabled ? "ON" : "OFF",
                               eps_state.beacon_enabled ? "ON" : "OFF");
        if ((written > 0) && ((size_t)written < (room - (size_t)offset))) {
            offset += written;
        }
    }

    /* Fault information (description sanitized: no commas inside a field) */
    FaultLogEntry_t last_fault;
    if (fault_log_get_last(&last_fault) == SMART_QSO_OK) {
        char description[sizeof(last_fault.description)];
        (void)snprintf(description, sizeof(description), "%s", last_fault.description);
        sanitize_field(description);
        int written = snprintf(buffer + offset, room - (size_t)offset,
                               "LAST_FAULT=%s,FAULT_COUNT=%zu,",
                               description, fault_log_get_count());
        if ((written > 0) && ((size_t)written < (room - (size_t)offset))) {
            offset += written;
        }
    }

    /* Health status */
    int written = snprintf(buffer + offset, room - (size_t)offset,
                           "UART_HEALTH=%s,WATCHDOG=%s",
                           uart_is_initialized() ? "OK" : "FAIL",
                           (wdt_mgr_was_wdt_reset() || wdt_mgr_any_task_stalled()) ? "TRIGGERED" : "OK");
    if ((written > 0) && ((size_t)written < (room - (size_t)offset))) {
        offset += written;
    }
    return offset;
}

static void send_telemetry_to_jetson(void)
{
    if (!eps_is_payload_enabled() || !uart_is_initialized()) {
        return;
    }

    /* ICD_OBC_JETSON section 4: TELEMETRY,<fields...>,<CRC>\n */
    char buffer[JETSON_LINE_MAX];
    size_t room = sizeof(buffer) - 6U;  /* ",XXXX\n" */
    int offset = snprintf(buffer, room,
                          "TELEMETRY,%llu,%s,%.3f,%d,%.1f,",
                          (unsigned long long)smart_qso_now_ms(),
                          s_sunlit ? "SUNLIT" : "ECLIPSE",
                          s_soc,
                          (int)eps_get_power_mode(),
                          eps_get_power_limit());
    if ((offset < 0) || ((size_t)offset >= room)) {
        return;
    }

    /* Sensor values */
    offset += (int)sensors_format_telemetry(buffer + offset, room - (size_t)offset);

    offset = append_status_fields(buffer, room, offset);
    if ((offset > 0) && (buffer[offset - 1] == ',')) {
        offset--;
    }

    send_line(buffer, jetson_link_seal(buffer, (size_t)offset, sizeof(buffer)), "telemetry");

    /* Ask for the next beacon text */
    s_beacon_seq++;
    char request[JETSON_LINE_MAX];
    send_line(request, jetson_link_beacon_req(s_beacon_seq, (uint8_t)JETSON_TEXT_MAX,
                                              request, sizeof(request)), "beacon request");
}

/** Read Jetson lines: keep the answer to our latest beacon request */
static void poll_jetson(void)
{
    char chunk[64];
    size_t received = 0U;

    if (!uart_is_initialized() ||
        (uart_receive(chunk, sizeof(chunk), &received) != SMART_QSO_OK)) {
        return;
    }
    for (size_t i = 0U; i < received; i++) {
        JetsonMsg_t msg;
        JetsonStatus_t status = JETSON_OK;
        if (!jetson_rx_feed(&s_jetson_rx, chunk[i], &msg, &status)) {
            continue;
        }
        if (status != JETSON_OK) {
            fprintf(stderr, "[UART] Rejected Jetson line (%d)\n", (int)status);
        } else if ((msg.type == JETSON_MSG_BEACON_RSP) && (msg.seq == s_beacon_seq)) {
            (void)snprintf(s_ai_text, sizeof(s_ai_text), "%s", msg.text);
            printf("[UART] Beacon text from Jetson (%zu chars)\n", msg.text_len);
        } else if (msg.type == JETSON_MSG_STATUS) {
            s_jetson_temp_c = msg.temp_c;
            s_jetson_status_ms = smart_qso_now_ms();
            s_jetson_status_seen = true;
            printf("[UART] Jetson status: health=%u temp=%dC power=%umW\n",
                   (unsigned)msg.health, (int)msg.temp_c, (unsigned)msg.power_mw);
        } else {
            /* Stale beacon answers, ACK and NACK need no action */
        }
    }
}

/** A sensor value read successfully within max_age_ms */
static bool fresh_sensor(const char *id, uint64_t now, uint32_t max_age_ms, double *value)
{
    Sensor_t sensor;
    if ((sensors_get_by_id(id, &sensor) != SMART_QSO_OK) || (sensor.last_read_ms == 0U) ||
        (now < sensor.last_read_ms) || ((now - sensor.last_read_ms) > max_age_ms)) {
        return false;
    }
    *value = sensor.last_value;
    return true;
}

/**
 * @brief OBC-enforced payload limits (SW-16)
 *
 * Input power is the bus voltage times the Jetson rail current; the spreader
 * temperature is the OBC's own thermistor. The Jetson-reported junction
 * temperature is a second, not the only, input.
 */
static void guard_payload(uint64_t now)
{
    PayloadSample_t sample;
    double bus_v = 0.0;
    double amps = 0.0;

    (void)memset(&sample, 0, sizeof(sample));
    if (fresh_sensor("BUSV", now, PG_TELEMETRY_TIMEOUT_MS, &bus_v) &&
        fresh_sensor("JPC", now, PG_TELEMETRY_TIMEOUT_MS, &amps)) {
        sample.power_valid = true;
        sample.input_w = bus_v * amps;
    }
    sample.spreader_valid = fresh_sensor("PT", now, PG_TELEMETRY_TIMEOUT_MS, &sample.spreader_c);
    if (s_jetson_status_seen && (now >= s_jetson_status_ms) &&
        ((now - s_jetson_status_ms) <= PG_TELEMETRY_TIMEOUT_MS)) {
        sample.junction_valid = true;
        sample.junction_c = (double)s_jetson_temp_c;
    }

    PayloadGuardResult_t result = payload_guard_step(&sample, eps_is_payload_enabled(), now);
    if (result != PG_OK) {
        FaultType_t type = ((result == PG_TRIP_OVERPOWER) || (result == PG_TRIP_LATCHUP)) ?
                           FAULT_TYPE_CURRENT_HIGH :
                           (((result == PG_TRIP_SPREADER_TEMP) || (result == PG_TRIP_JUNCTION_TEMP)) ?
                            FAULT_TYPE_TEMP_HIGH : FAULT_TYPE_POWER);
        fault_log_add(type, FAULT_SEVERITY_ERROR, payload_guard_reason(result), s_soc);
        (void)eps_control_payload(false, s_soc);
    }
}

/**
 * @brief Measurements for the beacon telemetry (SW-13)
 *
 * In this host simulation s_soc and s_sunlit are the EPS model; on the
 * target the EPS driver supplies them (SW-01). Battery voltage is the BV
 * sensor, only if read recently. There is no QSO detector yet, so the QSO
 * count is not reported.
 */
static void update_beacon_measurements(uint64_t now)
{
    BeaconMeasurements_t m;
    (void)memset(&m, 0, sizeof(m));
    m.soc = (uint8_t)((s_soc * 100.0) + 0.5);
    m.sunlit = s_sunlit ? 1U : 0U;
    m.valid = BEACON_TLM_SOC | BEACON_TLM_SUN;

    Sensor_t bv;
    if ((sensors_get_by_id("BV", &bv) == SMART_QSO_OK) && (bv.last_read_ms != 0U) &&
        ((now - bv.last_read_ms) <= BEACON_MEAS_MAX_AGE_MS) &&
        (bv.last_value > 0.0) && (bv.last_value < 65.0)) {
        m.battery_mv = (uint16_t)((bv.last_value * 1000.0) + 0.5);
        m.valid |= BEACON_TLM_BV;
    }
    (void)beacon_update_measurements(&m, now);
}

/**
 * @brief Send a beacon when one is due
 *
 * Uses the Jetson's latest text if the content filter passes it, otherwise
 * a template. Each Jetson text is sent at most once.
 */
static void run_beacon(uint64_t now)
{
    beacon_update_interval(eps_get_power_mode());
    if (!beacon_should_transmit(now)) {
        return;
    }

    BeaconContent_t content;
    size_t ai_len = strlen(s_ai_text);
    if ((beacon_generate_content(&content, (ai_len > 0U) ? s_ai_text : NULL, ai_len) == SMART_QSO_OK) &&
        (beacon_transmit(&content) == SMART_QSO_OK)) {
        beacon_mark_transmitted(now);
    }
    s_ai_text[0] = '\0';
}

/*===========================================================================*/
/* Main Loop                                                                  */
/*===========================================================================*/

/**
 * @brief Run main control loop
 *
 * @return SMART_QSO_OK on normal exit
 */
/**
 * @brief One pass of the main loop; each stage checks in with the watchdog
 */
static void run_tick(uint64_t tick, uint32_t telemetry_interval)
{
    update_mission_state(tick);
    uint64_t now = smart_qso_now_ms();

    monitor_system_health();
    (void)wdt_mgr_checkin(WDT_TASK_MAIN_LOOP);

    /* Telemetry and beacon request to the Jetson, and a periodic UART check */
    if (eps_is_payload_enabled() && uart_is_initialized() &&
        (now - s_last_telemetry_ms) >= telemetry_interval) {
        send_telemetry_to_jetson();
        s_last_telemetry_ms = now;
    }
    if (uart_is_initialized() && (now % 10000) < 100) {
        (void)uart_check_health();
    }
    (void)sensors_poll(now);

    /* Jetson replies (beacon text, status), then the OBC's payload limits */
    poll_jetson();
    guard_payload(now);
    (void)wdt_mgr_checkin(WDT_TASK_PAYLOAD);

    /* Beacon (stays silent while TX-OFF, contact or RF-silence inhibits) */
    update_beacon_measurements(now);
    run_beacon(now);
    (void)wdt_mgr_checkin(WDT_TASK_BEACON);

    /* Telecommands from the 435 MHz receiver, then key provisioning
     * (umbilical only) */
    (void)uplink_poll(NULL);
    (void)gse_poll();
    (void)wdt_mgr_checkin(WDT_TASK_COMM);

    /* Loss-of-contact, RF-silence and deployment timers. On target these
     * need a monotonic time base (smart_qso_now_ms() is not implemented
     * there yet; SW-01) */
    uint32_t elapsed = (uint32_t)(now - s_last_contact_tick_ms);
    contact_monitor_tick(elapsed);
    rf_silence_tick(elapsed);
    deployment_tick(elapsed);
    s_last_contact_tick_ms = now;

    if (tick % PERSISTENCE_INTERVAL_TICKS == 0) {
        (void)mission_data_save();
        (void)eps_save_config();
    }

    /* Refresh the hardware watchdog if every stage checked in */
    wdt_mgr_kick();
}

/** Pause between main-loop ticks (~20 ms) */
static void loop_sleep(void)
{
#ifdef HAL_TARGET_STM32L4
    hal_timer_delay_ms((uint32_t)(MAIN_LOOP_SLEEP_NS / 1000000L));
#else
    struct timespec ts = {.tv_sec = 0, .tv_nsec = MAIN_LOOP_SLEEP_NS};
    nanosleep(&ts, NULL);
#endif
}

static SmartQsoResult_t run_main_loop(void)
{
    MissionData_t mission;
    (void)mission_data_get(&mission);

    printf("[SYSTEM] SMART-QSO Flight Software Started\n");
    printf("[SYSTEM] Version: %s\n", SMART_QSO_VERSION_STRING);
    printf("[SYSTEM] Mission Phase: %d, Power Mode: %d, SOC: %.1f%%\n",
           mission.mission_phase, (int)eps_get_power_mode(), s_soc * 100.0);

    uint32_t telemetry_interval = uart_get_telemetry_interval();
#ifdef SIMULATION_BUILD
    for (uint64_t tick = 0; tick < MAIN_LOOP_ITERATIONS && !s_shutdown_requested; ++tick) {
#else
    /* Flight: runs until power-off; the watchdog resets a stalled loop
     * (docs/LOOP_BOUNDS.md) */
    for (uint64_t tick = 0; !s_shutdown_requested; ++tick) {
#endif
        run_tick(tick, telemetry_interval);
        loop_sleep();
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
    printf("[SYSTEM] Shutdown complete. Total uptime: %llu ms, Faults: %" PRIu32 "\n",
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
