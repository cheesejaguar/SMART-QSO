/**
 * @file sensors.c
 * @brief Sensor Management Implementation
 *
 * @requirement SRS-SENS-001 System shall support configurable sensor framework
 * @requirement SRS-SENS-002 System shall poll sensors at configurable intervals
 * @requirement SRS-SENS-003 System shall validate sensor readings
 */

#include "sensors.h"
#include "fault_mgmt.h"
#include "eps_control.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/*===========================================================================*/
/* Module State                                                               */
/*===========================================================================*/

/** Sensor array */
static Sensor_t s_sensors[SMART_QSO_MAX_SENSORS];

/** Number of registered sensors */
static size_t s_num_sensors = 0;

/** Environment state: sunlit flag */
static bool s_sunlit = true;

/** Environment state: state of charge */
static double s_soc = 0.75;

/** Flag indicating if module is initialized */
static bool s_initialized = false;

/*===========================================================================*/
/* Internal: Random Number Generation                                         */
/*===========================================================================*/

/**
 * @brief Generate random value in [0.0, 1.0]
 */
static double rnd_unit(void)
{
    return (double)rand() / (double)RAND_MAX;
}

/*===========================================================================*/
/* Sensor Read Implementations                                                */
/*===========================================================================*/

static bool read_software_timer(Sensor_t *self, double *out_value, char *out_text)
{
    (void)self;
    (void)out_text;
    static uint64_t s_start_ms = 0;

    if (s_start_ms == 0) {
        s_start_ms = smart_qso_now_ms();
    }

    uint64_t elapsed_ms = smart_qso_now_ms() - s_start_ms;
    *out_value = (double)(elapsed_ms / 1000ULL);
    return true;
}

static bool read_eps_voltage(Sensor_t *self, double *out_value, char *out_text)
{
    (void)out_text;
    double base = 0.0;

    if (strcmp(self->channel, "battery") == 0) {
        base = 8.1 + 0.15 * (s_soc - 0.5);
        if (base < 7.0) {
            fault_log_add(FAULT_TYPE_VOLTAGE_LOW, FAULT_SEVERITY_ERROR,
                          "Low battery voltage detected", s_soc);
        }
    } else if (strcmp(self->channel, "bus") == 0) {
        base = 5.0;
        if (base < 4.5 || base > 5.5) {
            fault_log_add(FAULT_TYPE_VOLTAGE_RANGE, FAULT_SEVERITY_ERROR,
                          "Bus voltage out of range", s_soc);
        }
    } else if (strcmp(self->channel, "solar") == 0) {
        base = s_sunlit ? 7.5 : 0.2;
    }

    *out_value = base + 0.02 * (rnd_unit() - 0.5);
    return true;
}

static bool read_eps_current(Sensor_t *self, double *out_value, char *out_text)
{
    (void)out_text;
    double val = 0.0;
    bool payload_enabled = eps_is_payload_enabled();

    if (strcmp(self->channel, "battery_discharge") == 0) {
        val = s_sunlit ? 0.05 : (payload_enabled ? 0.8 : 0.25);
        if (val > 1.0) {
            fault_log_add(FAULT_TYPE_CURRENT_HIGH, FAULT_SEVERITY_ERROR,
                          "Excessive battery discharge current", s_soc);
        }
    } else if (strcmp(self->channel, "jetson") == 0) {
        val = payload_enabled ? 0.7 + 0.05 * (rnd_unit() - 0.5) : 0.0;
    } else if (strcmp(self->channel, "solar") == 0) {
        val = s_sunlit ? (0.6 + 0.1 * (rnd_unit() - 0.5)) : 0.0;
    }

    *out_value = val;
    return true;
}

static bool read_eps_temperature(Sensor_t *self, double *out_value, char *out_text)
{
    (void)self;
    (void)out_text;
    bool payload_enabled = eps_is_payload_enabled();

    double base = s_sunlit ? 22.0 : 14.0;
    if (payload_enabled) {
        base += 3.0;
    }

    if (base > 45.0) {
        fault_log_add(FAULT_TYPE_TEMP_HIGH, FAULT_SEVERITY_ERROR,
                      "High temperature detected", s_soc);
        (void)fault_recover_thermal(s_soc);
    } else if (base < -10.0) {
        fault_log_add(FAULT_TYPE_TEMP_LOW, FAULT_SEVERITY_WARNING,
                      "Low temperature detected", s_soc);
    }

    *out_value = base + 0.5 * (rnd_unit() - 0.5);
    return true;
}

static bool read_status_hex2(Sensor_t *self, double *out_value, char *out_text)
{
    (void)self;
    (void)out_value;
    bool payload_enabled = eps_is_payload_enabled();
    PowerMode_t power_mode = eps_get_power_mode();
    EpsControlState_t eps_state;
    (void)eps_get_state(&eps_state);

    /* Enhanced bitmap (LSB..):
     * bit0: OBC on
     * bit1: Beacon on
     * bit2: Jetson enabled
     * bit3: Thermal hot
     * bit4: Battery low
     * bit5: Safe mode
     * bit6: Fault detected
     * bit7: Recovery mode
     */
    uint8_t bits = 0;
    bits |= 1u << 0;  /* OBC on */
    bits |= (eps_state.beacon_enabled ? 1u : 0u) << 1;
    bits |= (payload_enabled ? 1u : 0u) << 2;

    /* Occasional hot flag during sunlit periods */
    uint64_t now = smart_qso_now_ms();
    if (s_sunlit && (now / 1000ULL) % 300ULL > 240ULL) {
        bits |= 1u << 3;
    }

    if (s_soc < 0.30) {
        bits |= 1u << 4;  /* Battery low */
    }

    if (power_mode == POWER_MODE_SAFE) {
        bits |= 1u << 5;  /* Safe mode */
    }

    if (fault_log_get_count() > 0) {
        bits |= 1u << 6;  /* Fault detected */
    }

    if (fault_was_watchdog_triggered()) {
        bits |= 1u << 7;  /* Recovery mode */
    }

    int written = snprintf(out_text, 8, "%02X", (unsigned)bits);
    if (written < 0 || written >= 8) {
        return false;
    }

    return true;
}

/*===========================================================================*/
/* Sensor Binding                                                             */
/*===========================================================================*/

/**
 * @brief Bind read function to sensor based on type
 */
static bool bind_sensor_behavior(Sensor_t *s)
{
    SMART_QSO_REQUIRE_NOT_NULL(s);

    if (strcmp(s->type, "software_timer") == 0) {
        s->value_type = SENSOR_VALUE_NUMERIC;
        s->read = read_software_timer;
        return true;
    } else if (strcmp(s->type, "eps_voltage") == 0) {
        s->value_type = SENSOR_VALUE_NUMERIC;
        s->read = read_eps_voltage;
        return true;
    } else if (strcmp(s->type, "eps_current") == 0) {
        s->value_type = SENSOR_VALUE_NUMERIC;
        s->read = read_eps_current;
        return true;
    } else if (strcmp(s->type, "eps_temperature") == 0) {
        s->value_type = SENSOR_VALUE_NUMERIC;
        s->read = read_eps_temperature;
        return true;
    } else if (strcmp(s->type, "status_hex2") == 0) {
        s->value_type = SENSOR_VALUE_HEX2;
        s->read = read_status_hex2;
        return true;
    }

    return false;
}

/*===========================================================================*/
/* YAML Parsing                                                               */
/*===========================================================================*/

/**
 * @brief Trim whitespace from string
 */
static void trim(char *s)
{
    size_t n = strlen(s);
    while (n > 0 && (s[n - 1] == '\n' || s[n - 1] == '\r' ||
                     s[n - 1] == ' ' || s[n - 1] == '\t')) {
        s[--n] = '\0';
    }

    size_t i = 0;
    while (s[i] == ' ' || s[i] == '\t') {
        i++;
    }
    if (i > 0) {
        memmove(s, s + i, strlen(s + i) + 1);
    }
}

/**
 * @brief Parse key:value pair from YAML line
 */
static void parse_keyval(const char *line, char *key, size_t ksz,
                         char *val, size_t vsz)
{
    const char *colon = strchr(line, ':');
    if (colon == NULL) {
        key[0] = '\0';
        val[0] = '\0';
        return;
    }

    /* Extract key */
    size_t klen = 0;
    while (line[klen] != '\0' && line + klen < colon && klen + 1 < ksz) {
        key[klen] = line[klen];
        klen++;
    }
    key[klen] = '\0';

    /* Extract value */
    const char *p = colon + 1;
    while (*p == ' ' || *p == '\t') {
        p++;
    }
    size_t vlen = 0;
    while (p[vlen] != '\0' && vlen + 1 < vsz) {
        val[vlen] = p[vlen];
        vlen++;
    }
    val[vlen] = '\0';

    trim(key);
    trim(val);

    /* Remove quotes if present */
    if ((val[0] == '"' || val[0] == '\'') && strlen(val) >= 2) {
        size_t L = strlen(val);
        if (val[L - 1] == '"' || val[L - 1] == '\'') {
            val[L - 1] = '\0';
            memmove(val, val + 1, L - 1);
        }
    }
}

/**
 * @brief Add sensor from parsed fields
 */
static void add_sensor_from_fields(Sensor_t *cur)
{
    if (s_num_sensors >= SMART_QSO_MAX_SENSORS) {
        return;
    }
    if (!bind_sensor_behavior(cur)) {
        return;
    }
    s_sensors[s_num_sensors++] = *cur;
}

/*===========================================================================*/
/* Public API Implementation                                                  */
/*===========================================================================*/

SmartQsoResult_t sensors_init(void)
{
    memset(s_sensors, 0, sizeof(s_sensors));
    s_num_sensors = 0;
    s_sunlit = true;
    s_soc = 0.75;
    s_initialized = true;
    return SMART_QSO_OK;
}

SmartQsoResult_t sensors_load_yaml(const char *path)
{
    SMART_QSO_REQUIRE_NOT_NULL(path);

    FILE *f = fopen(path, "r");
    if (f == NULL) {
        return SMART_QSO_ERROR_IO;
    }

    char line[256];
    bool in_list = false;
    bool have_item = false;
    Sensor_t cur;
    memset(&cur, 0, sizeof(cur));

    while (fgets(line, sizeof(line), f) != NULL) {
        trim(line);
        if (line[0] == '#' || line[0] == '\0') {
            continue;
        }

        if (!in_list) {
            if (strcmp(line, "sensors:") == 0) {
                in_list = true;
            }
            continue;
        }

        if (strncmp(line, "- ", 2) == 0 || strcmp(line, "-") == 0) {
            if (have_item) {
                add_sensor_from_fields(&cur);
                memset(&cur, 0, sizeof(cur));
            }
            have_item = true;
            continue;
        }

        char key[64], val[128];
        parse_keyval(line, key, sizeof(key), val, sizeof(val));
        if (key[0] == '\0') {
            continue;
        }

        if (strcmp(key, "id") == 0) {
            strncpy(cur.id, val, sizeof(cur.id) - 1);
        } else if (strcmp(key, "name") == 0) {
            strncpy(cur.name, val, sizeof(cur.name) - 1);
        } else if (strcmp(key, "type") == 0) {
            strncpy(cur.type, val, sizeof(cur.type) - 1);
        } else if (strcmp(key, "units") == 0) {
            strncpy(cur.units, val, sizeof(cur.units) - 1);
        } else if (strcmp(key, "channel") == 0) {
            strncpy(cur.channel, val, sizeof(cur.channel) - 1);
        } else if (strcmp(key, "period_ms") == 0) {
            cur.period_ms = (uint32_t)strtoul(val, NULL, 10);
        }
    }

    if (have_item) {
        add_sensor_from_fields(&cur);
    }

    if (fclose(f) != 0) {
        return SMART_QSO_ERROR_IO;
    }

    return (s_num_sensors > 0) ? SMART_QSO_OK : SMART_QSO_ERROR;
}

SmartQsoResult_t sensors_load_defaults(void)
{
    struct {
        const char *id;
        const char *name;
        const char *type;
        const char *units;
        const char *channel;
        uint32_t period;
    } defs[] = {
        {"SET", "Spacecraft Elapsed Timer", "software_timer", "s", "", 1000},
        {"BV", "Battery Voltage", "eps_voltage", "V", "battery", 1000},
        {"BT", "Battery Temperature", "eps_temperature", "C", "battery", 2000},
        {"BUSV", "Bus Voltage", "eps_voltage", "V", "bus", 1000},
        {"BDI", "Battery Discharge Current", "eps_current", "A", "battery_discharge", 1000},
        {"JPC", "Jetson Payload Current", "eps_current", "A", "jetson", 1000},
        {"SPV", "Solar Panel Voltage", "eps_voltage", "V", "solar", 1000},
        {"SPC", "Solar Panel Current", "eps_current", "A", "solar", 1000},
        {"ST", "Status Hex", "status_hex2", "hex", "", 1000},
    };

    size_t num_defs = sizeof(defs) / sizeof(defs[0]);
    for (size_t i = 0; i < num_defs; ++i) {
        Sensor_t s;
        memset(&s, 0, sizeof(s));
        strncpy(s.id, defs[i].id, sizeof(s.id) - 1);
        strncpy(s.name, defs[i].name, sizeof(s.name) - 1);
        strncpy(s.type, defs[i].type, sizeof(s.type) - 1);
        strncpy(s.units, defs[i].units, sizeof(s.units) - 1);
        strncpy(s.channel, defs[i].channel, sizeof(s.channel) - 1);
        s.period_ms = defs[i].period;

        if (bind_sensor_behavior(&s)) {
            s_sensors[s_num_sensors++] = s;
        }
    }

    return SMART_QSO_OK;
}

size_t sensors_get_count(void)
{
    return s_num_sensors;
}

SmartQsoResult_t sensors_get(size_t index, Sensor_t *sensor)
{
    SMART_QSO_REQUIRE_NOT_NULL(sensor);
    SMART_QSO_REQUIRE(index < s_num_sensors, "Index out of range");

    *sensor = s_sensors[index];
    return SMART_QSO_OK;
}

SmartQsoResult_t sensors_get_by_id(const char *id, Sensor_t *sensor)
{
    SMART_QSO_REQUIRE_NOT_NULL(id);
    SMART_QSO_REQUIRE_NOT_NULL(sensor);

    for (size_t i = 0; i < s_num_sensors; i++) {
        if (strcmp(s_sensors[i].id, id) == 0) {
            *sensor = s_sensors[i];
            return SMART_QSO_OK;
        }
    }

    return SMART_QSO_ERROR;
}

size_t sensors_poll(uint64_t current_ms)
{
    size_t count = 0;

    for (size_t i = 0; i < s_num_sensors; ++i) {
        Sensor_t *s = &s_sensors[i];
        if (current_ms >= s->next_poll_ms) {
            double val = 0.0;
            char text[8] = {0};

            if (s->read != NULL && s->read(s, &val, text)) {
                if (s->value_type == SENSOR_VALUE_NUMERIC) {
                    s->last_value = val;
                    printf("[READ] id=%s name=\"%s\" value=%.3f units=%s\n",
                           s->id, s->name, s->last_value, s->units);
                } else {
                    strncpy(s->last_text, text, sizeof(s->last_text) - 1);
                    printf("[READ] id=%s name=\"%s\" value=%s units=%s\n",
                           s->id, s->name, s->last_text, s->units);
                }
                count++;
            }

            s->next_poll_ms = current_ms + (s->period_ms > 0 ? s->period_ms : 1000);
        }
    }

    return count;
}

SmartQsoResult_t sensors_poll_one(size_t index)
{
    SMART_QSO_REQUIRE(index < s_num_sensors, "Index out of range");

    Sensor_t *s = &s_sensors[index];
    double val = 0.0;
    char text[8] = {0};

    if (s->read == NULL || !s->read(s, &val, text)) {
        return SMART_QSO_ERROR;
    }

    if (s->value_type == SENSOR_VALUE_NUMERIC) {
        s->last_value = val;
    } else {
        strncpy(s->last_text, text, sizeof(s->last_text) - 1);
    }

    return SMART_QSO_OK;
}

void sensors_set_environment(bool sunlit, double soc)
{
    s_sunlit = sunlit;
    s_soc = soc;
}

size_t sensors_format_telemetry(char *buffer, size_t buffer_len)
{
    SMART_QSO_REQUIRE_NOT_NULL(buffer);
    SMART_QSO_REQUIRE(buffer_len > 0, "Invalid buffer length");

    size_t offset = 0;

    for (size_t i = 0; i < s_num_sensors; ++i) {
        Sensor_t *s = &s_sensors[i];
        int written;

        if (s->value_type == SENSOR_VALUE_NUMERIC) {
            written = snprintf(buffer + offset, buffer_len - offset,
                               "%s=%.3f%s,", s->id, s->last_value, s->units);
        } else {
            written = snprintf(buffer + offset, buffer_len - offset,
                               "%s=%s%s,", s->id, s->last_text, s->units);
        }

        if (written < 0 || (size_t)written >= buffer_len - offset) {
            break;  /* Truncation detected */
        }
        offset += (size_t)written;
    }

    return offset;
}
