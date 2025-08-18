#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <fcntl.h>
#include <termios.h>
#include <errno.h>
#include <sys/stat.h>
#include <signal.h>

// Object-oriented-ish sensor framework with YAML configuration.
// Enhanced with EPS control, fault recovery, and data persistence capabilities.
// Aligns with CONOPS: always-on OBC monitors EPS & status; Jetson bursts are
// simulated via current draw flags in the status bitmap.

// UART configuration for Jetson communication (configurable via environment variables)
#define UART_DEVICE_DEFAULT "/dev/ttyUSB0"  // Default UART device
#define UART_BAUDRATE_DEFAULT B115200       // Default baud rate
#define TELEMETRY_INTERVAL_MS_DEFAULT 60000 // Default: 60 seconds

// EPS Control and System Configuration
#define EPS_CONFIG_FILE "/tmp/eps_config.dat"
#define MISSION_DATA_FILE "/tmp/mission_data.dat"
#define FAULT_LOG_FILE "/tmp/fault_log.dat"
#define MAX_FAULT_ENTRIES 100
#define WATCHDOG_TIMEOUT_MS 30000  // 30 seconds watchdog timeout

// Runtime configurable values
static char uart_device[64] = UART_DEVICE_DEFAULT;
static speed_t uart_baudrate = UART_BAUDRATE_DEFAULT;
static uint32_t telemetry_interval_ms = TELEMETRY_INTERVAL_MS_DEFAULT;

// EPS Control State
typedef struct {
    bool payload_enabled;      // Jetson payload power state
    bool radio_enabled;        // Radio/transponder power state
    bool adcs_enabled;         // ADCS power state
    bool beacon_enabled;       // Beacon power state
    uint8_t power_mode;        // Power mode (0=SAFE, 1=IDLE, 2=ACTIVE)
    double power_limit_w;      // Current power limit in watts
    uint32_t last_control_ms;  // Last EPS control command time
} EPSControlState;

// Mission Data Persistence
typedef struct {
    uint64_t mission_start_ms;     // Mission start timestamp
    uint64_t total_uptime_ms;      // Total uptime across resets
    uint32_t reset_count;          // Number of resets
    uint64_t last_reset_ms;        // Last reset timestamp
    double total_energy_wh;        // Total energy consumed
    uint32_t fault_count;          // Total fault occurrences
    uint8_t mission_phase;         // Current mission phase
    char last_fault[64];           // Description of last fault
} MissionData;

// Fault Log Entry
typedef struct {
    uint64_t timestamp_ms;         // Fault occurrence time
    uint8_t fault_type;            // Fault type identifier
    uint8_t severity;              // Severity level (1=INFO, 2=WARN, 3=ERROR, 4=CRITICAL)
    char description[64];           // Fault description
    double soc_at_fault;           // Battery SoC at fault time
    bool recovered;                 // Whether fault was recovered from
} FaultLogEntry;

typedef enum {
  SENSOR_NUMERIC = 0,
  SENSOR_HEX2 = 1
} SensorValueType;

struct Sensor;
typedef bool (*SensorReadFn)(struct Sensor *self, double *out_value, char *out_text);

typedef struct Sensor {
  char id[8];
  char name[64];
  char units[16];
  char type[32];
  char channel[32];
  uint32_t period_ms;
  uint64_t next_poll_ms;
  SensorValueType value_type;
  SensorReadFn read;
  double last_value;
  char last_text[8];
} Sensor;

#define MAX_SENSORS 32
static Sensor sensors[MAX_SENSORS];
static size_t num_sensors = 0;

// Global state
static int uart_fd = -1;
static uint64_t last_telemetry_ms = 0;
static bool uart_initialized = false;
static uint64_t program_start_ms;
static bool sunlit = true;
static double soc = 0.75; // 0..1

// EPS Control State
static EPSControlState eps_state = {
    .payload_enabled = false,
    .radio_enabled = true,
    .adcs_enabled = true,
    .beacon_enabled = true,
    .power_mode = 1,  // Start in IDLE mode
    .power_limit_w = 2.0,
    .last_control_ms = 0
};

// Mission Data
static MissionData mission_data = {0};
static FaultLogEntry fault_log[MAX_FAULT_ENTRIES];
static size_t fault_log_count = 0;

// Watchdog timer
static uint64_t last_watchdog_reset_ms = 0;
static bool watchdog_triggered = false;

// Signal handler for graceful shutdown
static volatile bool shutdown_requested = false;
static void signal_handler(int sig) {
    (void)sig;
    shutdown_requested = true;
}

static uint64_t now_ms(void) {
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  return (uint64_t)ts.tv_sec * 1000ULL + (ts.tv_nsec / 1000000ULL);
}

static double rnd_unit(void) {
  return (double)rand() / (double)RAND_MAX;
}

// Data Persistence Functions
static bool save_mission_data(void) {
    FILE *f = fopen(MISSION_DATA_FILE, "wb");
    if (!f) return false;
    
    size_t written = fwrite(&mission_data, sizeof(MissionData), 1, f);
    fclose(f);
    return written == 1;
}

static bool load_mission_data(void) {
    FILE *f = fopen(MISSION_DATA_FILE, "rb");
    if (!f) return false;
    
    size_t read = fread(&mission_data, sizeof(MissionData), 1, f);
    fclose(f);
    
    if (read == 1) {
        // Update mission data for this session
        mission_data.reset_count++;
        mission_data.last_reset_ms = now_ms();
        return true;
    }
    return false;
}

static bool save_eps_config(void) {
    FILE *f = fopen(EPS_CONFIG_FILE, "wb");
    if (!f) return false;
    
    size_t written = fwrite(&eps_state, sizeof(EPSControlState), 1, f);
    fclose(f);
    return written == 1;
}

static bool load_eps_config(void) {
    FILE *f = fopen(EPS_CONFIG_FILE, "rb");
    if (!f) return false;
    
    size_t read = fread(&eps_state, sizeof(EPSControlState), 1, f);
    fclose(f);
    return read == 1;
}

static void add_fault_log(uint8_t fault_type, uint8_t severity, const char *description) {
    if (fault_log_count >= MAX_FAULT_ENTRIES) {
        // Shift log entries to make room
        memmove(&fault_log[0], &fault_log[1], sizeof(FaultLogEntry) * (MAX_FAULT_ENTRIES - 1));
        fault_log_count = MAX_FAULT_ENTRIES - 1;
    }
    
    FaultLogEntry *entry = &fault_log[fault_log_count++];
    entry->timestamp_ms = now_ms();
    entry->fault_type = fault_type;
    entry->severity = severity;
    strncpy(entry->description, description, sizeof(entry->description) - 1);
    entry->soc_at_fault = soc;
    entry->recovered = false;
    
    // Update mission data
    mission_data.fault_count++;
    strncpy(mission_data.last_fault, description, sizeof(mission_data.last_fault) - 1);
    
    // Save fault log
    FILE *f = fopen(FAULT_LOG_FILE, "wb");
    if (f) {
        fwrite(fault_log, sizeof(FaultLogEntry), fault_log_count, f);
        fclose(f);
    }
    
    printf("[FAULT] Type=%d Severity=%d: %s\n", fault_type, severity, description);
}

// EPS Control Functions
static bool eps_control_payload(bool enable) {
    if (eps_state.payload_enabled == enable) return true;
    
    // Check power constraints
    if (enable && soc < 0.55) {
        add_fault_log(1, 2, "Payload enable blocked: low SOC");
        return false;
    }
    
    eps_state.payload_enabled = enable;
    eps_state.last_control_ms = now_ms();
    
    if (enable) {
        add_fault_log(1, 1, "Payload enabled");
        printf("[EPS] Payload enabled\n");
    } else {
        add_fault_log(1, 1, "Payload disabled");
        printf("[EPS] Payload disabled\n");
    }
    
    return save_eps_config();
}

static bool eps_control_radio(bool enable) {
    if (eps_state.radio_enabled == enable) return true;
    
    eps_state.radio_enabled = enable;
    eps_state.last_control_ms = now_ms();
    
    if (enable) {
        printf("[EPS] Radio enabled\n");
    } else {
        printf("[EPS] Radio disabled\n");
    }
    
    return save_eps_config();
}

static bool eps_control_adcs(bool enable) {
    if (eps_state.adcs_enabled == enable) return true;
    
    eps_state.adcs_enabled = enable;
    eps_state.last_control_ms = now_ms();
    
    if (enable) {
        printf("[EPS] ADCS enabled\n");
    } else {
        printf("[EPS] ADCS disabled\n");
    }
    
    return save_eps_config();
}

static bool eps_control_beacon(bool enable) {
    if (eps_state.beacon_enabled == enable) return true;
    
    eps_state.beacon_enabled = enable;
    eps_state.last_control_ms = now_ms();
    
    if (enable) {
        printf("[EPS] Beacon enabled\n");
    } else {
        printf("[EPS] Beacon disabled\n");
    }
    
    return save_eps_config();
}

static void eps_set_power_mode(uint8_t mode) {
    if (eps_state.power_mode == mode) return;
    
    eps_state.power_mode = mode;
    eps_state.last_control_ms = now_ms();
    
    switch (mode) {
        case 0: // SAFE mode
            eps_control_payload(false);
            eps_control_radio(false);
            eps_control_adcs(false);
            eps_control_beacon(true);  // Keep beacon for emergency
            eps_state.power_limit_w = 0.5;
            add_fault_log(2, 2, "Entered SAFE mode");
            break;
        case 1: // IDLE mode
            eps_control_payload(false);
            eps_control_radio(true);
            eps_control_adcs(true);
            eps_control_beacon(true);
            eps_state.power_limit_w = 1.5;
            add_fault_log(2, 1, "Entered IDLE mode");
            break;
        case 2: // ACTIVE mode
            eps_control_payload(true);
            eps_control_radio(true);
            eps_control_adcs(true);
            eps_control_beacon(true);
            eps_state.power_limit_w = 3.0;
            add_fault_log(2, 1, "Entered ACTIVE mode");
            break;
    }
    
    save_eps_config();
}

// Fault Recovery Functions
static void recover_from_thermal_fault(void) {
    if (eps_state.payload_enabled) {
        add_fault_log(3, 2, "Thermal fault recovery: disabling payload");
        eps_control_payload(false);
        eps_set_power_mode(1);  // Go to IDLE mode
    }
}

static void recover_from_power_fault(void) {
    if (soc < 0.25) {
        add_fault_log(4, 3, "Critical power fault: entering SAFE mode");
        eps_set_power_mode(0);  // Go to SAFE mode
    } else if (soc < 0.40) {
        add_fault_log(4, 2, "Power fault recovery: entering IDLE mode");
        eps_set_power_mode(1);  // Go to IDLE mode
    }
}

static void recover_from_watchdog_fault(void) {
    add_fault_log(5, 3, "Watchdog timeout: system reset");
    // In real implementation, this would trigger a system reset
    // For simulation, we'll just log it and continue
    watchdog_triggered = true;
}

// UART initialization and communication functions
static bool init_uart(void) {
  if (uart_initialized) return true;
  
  uart_fd = open(uart_device, O_RDWR | O_NOCTTY | O_SYNC);
  if (uart_fd < 0) {
    fprintf(stderr, "[UART] Failed to open %s: %s\n", uart_device, strerror(errno));
    return false;
  }
  
  struct termios tty;
  if (tcgetattr(uart_fd, &tty) != 0) {
    fprintf(stderr, "[UART] Failed to get attributes: %s\n", strerror(errno));
    close(uart_fd);
    uart_fd = -1;
    return false;
  }
  
  // Set baud rate
  cfsetospeed(&tty, uart_baudrate);
  cfsetispeed(&tty, uart_baudrate);
  
  // 8N1 configuration
  tty.c_cflag = (tty.c_cflag & ~CSIZE) | CS8;
  tty.c_cflag &= ~(PARENB | PARODD);
  tty.c_cflag &= ~CSTOPB;
  tty.c_cflag &= ~CRTSCTS;
  tty.c_cflag |= CREAD | CLOCAL;
  
  // Raw input
  tty.c_lflag &= ~(ICANON | ECHO | ECHOE | ISIG);
  
  // Raw output
  tty.c_oflag &= ~OPOST;
  
  // Set attributes
  if (tcsetattr(uart_fd, TCSANOW, &tty) != 0) {
    fprintf(stderr, "[UART] Failed to set attributes: %s\n", strerror(errno));
    close(uart_fd);
    uart_fd = -1;
    return false;
  }
  
  uart_initialized = true;
  fprintf(stderr, "[UART] Initialized %s at %lu baud\n", uart_device, (unsigned long)uart_baudrate);
  return true;
}

static void close_uart(void) {
  if (uart_fd >= 0) {
    close(uart_fd);
    uart_fd = -1;
  }
  uart_initialized = false;
}

// Parse environment variables for UART configuration
static void parse_uart_config(void) {
  const char *device = getenv("UART_DEVICE");
  if (device && strlen(device) < sizeof(uart_device)) {
    strncpy(uart_device, device, sizeof(uart_device) - 1);
    uart_device[sizeof(uart_device) - 1] = '\0';
  }
  
  const char *baudrate = getenv("UART_BAUDRATE");
  if (baudrate) {
    int baud = atoi(baudrate);
    switch (baud) {
      case 9600: uart_baudrate = B9600; break;
      case 19200: uart_baudrate = B19200; break;
      case 38400: uart_baudrate = B38400; break;
      case 57600: uart_baudrate = B57600; break;
      case 115200: uart_baudrate = B115200; break;
      case 230400: uart_baudrate = B230400; break;
      default: fprintf(stderr, "[UART] Invalid baudrate %s, using default\n", baudrate); break;
    }
  }
  
  const char *interval = getenv("TELEMETRY_INTERVAL_MS");
  if (interval) {
    int ms = atoi(interval);
    if (ms > 0 && ms <= 3600000) { // 1ms to 1 hour
      telemetry_interval_ms = (uint32_t)ms;
    } else {
      fprintf(stderr, "[UART] Invalid interval %s, using default\n", interval);
    }
  }
  
  printf("[UART] Configuration: device=%s, baudrate=%lu, interval=%ums\n", 
         uart_device, (unsigned long)uart_baudrate, telemetry_interval_ms);
}

static bool send_uart_data(const char *data, size_t len) {
  if (!uart_initialized || uart_fd < 0) return false;
  
  ssize_t written = write(uart_fd, data, len);
  if (written != (ssize_t)len) {
    fprintf(stderr, "[UART] Write failed: %s\n", strerror(errno));
    // Try to reinitialize UART on error
    close_uart();
    if (init_uart()) {
      // Retry once
      written = write(uart_fd, data, len);
      if (written != (ssize_t)len) {
        fprintf(stderr, "[UART] Retry failed: %s\n", strerror(errno));
        return false;
      }
    } else {
      return false;
    }
  }
  
  // Ensure data is transmitted
  tcdrain(uart_fd);
  return true;
}

// Enhanced telemetry with comprehensive health data
static void send_telemetry_to_jetson(void) {
  if (!eps_state.payload_enabled || !uart_initialized) return;
  
  char buffer[2048];
  int offset = 0;
  
  // Start telemetry packet with comprehensive mission state
  offset += snprintf(buffer + offset, sizeof(buffer) - offset, 
                    "TELEMETRY,%llu,%s,%.3f,%d,%.1f,", 
                    (unsigned long long)now_ms(),
                    sunlit ? "SUNLIT" : "ECLIPSE",
                    soc,
                    eps_state.power_mode,
                    eps_state.power_limit_w);
  
  // Add all sensor values
  for (size_t i = 0; i < num_sensors; ++i) {
    Sensor *s = &sensors[i];
    if (s->value_type == SENSOR_NUMERIC) {
      offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                        "%s=%.3f%s,", s->id, s->last_value, s->units);
    } else {
      offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                        "%s=%s%s,", s->id, s->last_text, s->units);
    }
  }
  
  // Add EPS control state
  offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                    "EPS_PAYLOAD=%s,EPS_RADIO=%s,EPS_ADCS=%s,EPS_BEACON=%s,",
                    eps_state.payload_enabled ? "ON" : "OFF",
                    eps_state.radio_enabled ? "ON" : "OFF",
                    eps_state.adcs_enabled ? "ON" : "OFF",
                    eps_state.beacon_enabled ? "ON" : "OFF");
  
  // Add fault information
  if (fault_log_count > 0) {
    FaultLogEntry *last_fault = &fault_log[fault_log_count - 1];
    offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                      "LAST_FAULT=%s,FAULT_COUNT=%d,",
                      last_fault->description,
                      (int)fault_log_count);
  }
  
  // Add UART health status
  offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                    "UART_HEALTH=%s,WATCHDOG=%s", 
                    uart_initialized ? "OK" : "FAIL",
                    watchdog_triggered ? "TRIGGERED" : "OK");
  
  // Remove trailing comma and add newline
  if (offset > 0 && buffer[offset-1] == ',') {
    buffer[offset-1] = '\n';
  } else {
    buffer[offset] = '\n';
    offset++;
  }
  
  // Send via UART
  if (send_uart_data(buffer, offset)) {
    printf("[UART] Sent enhanced telemetry to Jetson (%d bytes)\n", offset);
  } else {
    fprintf(stderr, "[UART] Failed to send telemetry to Jetson\n");
  }
}

// Check UART health and attempt recovery if needed
static void check_uart_health(void) {
  if (!uart_initialized) return;
  
  // Simple health check - try to get UART attributes
  struct termios tty;
  if (tcgetattr(uart_fd, &tty) != 0) {
    fprintf(stderr, "[UART] Health check failed, attempting recovery\n");
    add_fault_log(6, 2, "UART health check failed");
    close_uart();
    if (eps_state.payload_enabled) {
      init_uart(); // Try to reinitialize
    }
  }
}

// Enhanced sensor read implementations with fault detection
static bool read_software_timer(Sensor *self, double *out_value, char *out_text) {
  (void)self; (void)out_text;
  uint64_t ms = now_ms() - program_start_ms;
  *out_value = (double)(ms / 1000ULL);
  return true;
}

static bool read_eps_voltage(Sensor *self, double *out_value, char *out_text) {
  (void)out_text;
  double base = 0.0;
  if (strcmp(self->channel, "battery") == 0) {
    base = 8.1 + 0.15 * (soc - 0.5);
    // Check for low battery voltage
    if (base < 7.0) {
      add_fault_log(7, 3, "Low battery voltage detected");
    }
  } else if (strcmp(self->channel, "bus") == 0) {
    base = 5.0;
    // Check for bus voltage anomaly
    if (base < 4.5 || base > 5.5) {
      add_fault_log(8, 3, "Bus voltage out of range");
    }
  } else if (strcmp(self->channel, "solar") == 0) {
    base = sunlit ? 7.5 : 0.2;
  }
  *out_value = base + 0.02 * (rnd_unit() - 0.5);
  return true;
}

static bool read_eps_current(Sensor *self, double *out_value, char *out_text) {
  (void)out_text;
  double val = 0.0;
  if (strcmp(self->channel, "battery_discharge") == 0) {
    val = sunlit ? 0.05 : (eps_state.payload_enabled ? 0.8 : 0.25);
    // Check for excessive discharge current
    if (val > 1.0) {
      add_fault_log(9, 3, "Excessive battery discharge current");
    }
  } else if (strcmp(self->channel, "jetson") == 0) {
    val = eps_state.payload_enabled ? 0.7 + 0.05 * (rnd_unit() - 0.5) : 0.0;
  } else if (strcmp(self->channel, "solar") == 0) {
    val = sunlit ? (0.6 + 0.1 * (rnd_unit() - 0.5)) : 0.0;
  }
  *out_value = val;
  return true;
}

static bool read_eps_temperature(Sensor *self, double *out_value, char *out_text) {
  (void)self; (void)out_text;
  double base = sunlit ? 22.0 : 14.0;
  if (eps_state.payload_enabled) base += 3.0;
  
  // Check for thermal anomalies
  if (base > 45.0) {
    add_fault_log(10, 3, "High temperature detected");
    recover_from_thermal_fault();
  } else if (base < -10.0) {
    add_fault_log(11, 2, "Low temperature detected");
  }
  
  *out_value = base + 0.5 * (rnd_unit() - 0.5);
  return true;
}

static bool read_status_hex2(Sensor *self, double *out_value, char *out_text) {
  (void)self; (void)out_value;
  // Enhanced bitmap (LSB..):
  // bit0: OBC on; bit1: Beacon on; bit2: Jetson enabled; bit3: Thermal hot;
  // bit4: Battery low; bit5: Safe mode; bit6: Fault detected; bit7: Recovery mode
  uint8_t bits = 0;
  bits |= 1u << 0; // OBC on
  bits |= (eps_state.beacon_enabled ? 1u : 0u) << 1; // Beacon on
  bits |= (eps_state.payload_enabled ? 1u : 0u) << 2; // Jetson enabled
  if (sunlit && (now_ms() / 1000ULL) % 300ULL > 240ULL) bits |= 1u << 3; // occasional hot
  if (soc < 0.30) bits |= 1u << 4; // Battery low
  if (eps_state.power_mode == 0) bits |= 1u << 5; // Safe mode
  if (fault_log_count > 0) bits |= 1u << 6; // Fault detected
  if (watchdog_triggered) bits |= 1u << 7; // Recovery mode
  
  snprintf(out_text, 8, "%02X", (unsigned)bits);
  return true;
}

// Assign function pointers by sensor type
static bool bind_sensor_behavior(Sensor *s) {
  if (strcmp(s->type, "software_timer") == 0) {
    s->value_type = SENSOR_NUMERIC; s->read = read_software_timer; return true;
  } else if (strcmp(s->type, "eps_voltage") == 0) {
    s->value_type = SENSOR_NUMERIC; s->read = read_eps_voltage; return true;
  } else if (strcmp(s->type, "eps_current") == 0) {
    s->value_type = SENSOR_NUMERIC; s->read = read_eps_current; return true;
  } else if (strcmp(s->type, "eps_temperature") == 0) {
    s->value_type = SENSOR_NUMERIC; s->read = read_eps_temperature; return true;
  } else if (strcmp(s->type, "status_hex2") == 0) {
    s->value_type = SENSOR_HEX2; s->read = read_status_hex2; return true;
  }
  return false;
}

// Minimal YAML reader for a constrained schema:
// sensors:
//  - id: BV
//    name: Battery Voltage
//    type: eps_voltage
//    channel: battery
//    units: V
//    period_ms: 1000
static void trim(char *s) {
  size_t n = strlen(s);
  while (n && (s[n-1] == '\n' || s[n-1] == '\r' || s[n-1] == ' ' || s[n-1] == '\t')) { s[--n] = 0; }
  size_t i = 0; while (s[i] == ' ' || s[i] == '\t') i++; if (i) memmove(s, s + i, strlen(s + i) + 1);
}

static void parse_keyval(const char *line, char *key, size_t ksz, char *val, size_t vsz) {
  const char *colon = strchr(line, ':');
  if (!colon) { key[0] = 0; val[0] = 0; return; }
  size_t klen = 0; while (line[klen] && line + klen < colon && klen + 1 < ksz) { key[klen] = line[klen]; klen++; }
  key[klen] = 0; while (klen && key[klen-1] == ' ') { key[--klen] = 0; }
  const char *p = colon + 1; while (*p == ' ' || *p == '\t') p++;
  size_t vlen = 0; while (p[vlen] && vlen + 1 < vsz) { val[vlen] = p[vlen]; vlen++; }
  val[vlen] = 0;
  trim(key); trim(val);
  if (val[0] == '"' || val[0] == '\'') {
    size_t L = strlen(val);
    if (L >= 2 && (val[L-1] == '"' || val[L-1] == '\'')) { val[L-1] = 0; memmove(val, val + 1, L - 1); }
  }
}

static void add_sensor_from_fields(Sensor *cur) {
  if (num_sensors >= MAX_SENSORS) return;
  if (!bind_sensor_behavior(cur)) return;
  sensors[num_sensors++] = *cur;
}

static bool load_yaml_config(const char *path) {
  FILE *f = fopen(path, "r");
  if (!f) return false;
  char line[256];
  bool in_list = false; bool have_item = false; Sensor cur; memset(&cur, 0, sizeof(cur));
  while (fgets(line, sizeof(line), f)) {
    trim(line);
    if (line[0] == '#' || line[0] == 0) continue;
    if (!in_list) {
      if (strcmp(line, "sensors:") == 0) { in_list = true; }
      continue;
    }
    if (strncmp(line, "- ", 2) == 0 || strcmp(line, "-") == 0) {
      if (have_item) { add_sensor_from_fields(&cur); memset(&cur, 0, sizeof(cur)); }
      have_item = true; continue;
    }
    char key[64], val[128];
    parse_keyval(line, key, sizeof(key), val, sizeof(val));
    if (!key[0]) continue;
    if (strcmp(key, "id") == 0) { strncpy(cur.id, val, sizeof(cur.id)-1); }
    else if (strcmp(key, "name") == 0) { strncpy(cur.name, val, sizeof(cur.name)-1); }
    else if (strcmp(key, "type") == 0) { strncpy(cur.type, val, sizeof(cur.type)-1); }
    else if (strcmp(key, "units") == 0) { strncpy(cur.units, val, sizeof(cur.units)-1); }
    else if (strcmp(key, "channel") == 0) { strncpy(cur.channel, val, sizeof(cur.channel)-1); }
    else if (strcmp(key, "period_ms") == 0) { cur.period_ms = (uint32_t)strtoul(val, NULL, 10); }
  }
  if (have_item) { add_sensor_from_fields(&cur); }
  fclose(f);
  return num_sensors > 0;
}

static void add_default_sensors(void) {
  struct {
    const char *id, *name, *type, *units, *channel; uint32_t period;
  } defs[] = {
    {"SET","Spacecraft Elapsed Timer","software_timer","s","",1000},
    {"BV","Battery Voltage","eps_voltage","V","battery",1000},
    {"BT","Battery Temperature","eps_temperature","C","battery",2000},
    {"BUSV","Bus Voltage","eps_voltage","V","bus",1000},
    {"BDI","Battery Discharge Current","eps_current","A","battery_discharge",1000},
    {"JPC","Jetson Payload Current","eps_current","A","jetson",1000},
    {"SPV","Solar Panel Voltage","eps_voltage","V","solar",1000},
    {"SPC","Solar Panel Current","eps_current","A","solar",1000},
    {"ST","Status Hex","status_hex2","hex","",1000},
  };
  for (size_t i = 0; i < sizeof(defs)/sizeof(defs[0]); ++i) {
    Sensor s; memset(&s, 0, sizeof(s));
    strncpy(s.id, defs[i].id, sizeof(s.id)-1);
    strncpy(s.name, defs[i].name, sizeof(s.name)-1);
    strncpy(s.type, defs[i].type, sizeof(s.type)-1);
    strncpy(s.units, defs[i].units, sizeof(s.units)-1);
    strncpy(s.channel, defs[i].channel, sizeof(s.channel)-1);
    s.period_ms = defs[i].period;
    if (bind_sensor_behavior(&s)) { sensors[num_sensors++] = s; }
  }
}

// Enhanced mission state management with fault recovery
static void update_mission_state(uint64_t tick) {
  sunlit = (tick % 180) < 100; // ~55% sunlit cycle
  
  // Simple SOC integration with fault detection
  double discharge = eps_state.payload_enabled ? 0.0040 : 0.0010;
  double charge = 0.0020;
  if (sunlit) soc += charge; else soc -= discharge;
  if (soc > 1.0) soc = 1.0; if (soc < 0.0) soc = 0.0;
  
  // Power management with fault recovery
  if (soc < 0.25) {
    // Critical power condition - enter SAFE mode
    if (eps_state.power_mode != 0) {
      eps_set_power_mode(0);
    }
  } else if (soc < 0.40) {
    // Low power condition - enter IDLE mode
    if (eps_state.power_mode > 1) {
      eps_set_power_mode(1);
    }
  } else if (sunlit && soc > 0.55 && eps_state.power_mode < 2) {
    // Good conditions - can enter ACTIVE mode
    eps_set_power_mode(2);
  }
  
  // Watchdog management
  if ((now_ms() - last_watchdog_reset_ms) > WATCHDOG_TIMEOUT_MS) {
    recover_from_watchdog_fault();
    last_watchdog_reset_ms = now_ms();
  }
  
  // Periodic fault recovery checks
  if (tick % 50 == 0) { // Every 50 ticks
    // Check if we can recover from faults
    if (eps_state.power_mode == 0 && soc > 0.40) {
      // Try to recover from SAFE mode
      eps_set_power_mode(1);
      add_fault_log(12, 1, "Recovered from SAFE mode");
    }
  }
  
  // Update mission data
  mission_data.total_uptime_ms = now_ms() - mission_data.mission_start_ms;
  if (sunlit && eps_state.payload_enabled) {
    mission_data.total_energy_wh += (discharge * 3.7 / 3600.0); // Rough energy calculation
  }
}

// System health monitoring and fault detection
static void monitor_system_health(void) {
  static uint64_t last_health_check_ms = 0;
  uint64_t now = now_ms();
  
  if ((now - last_health_check_ms) < 10000) return; // Check every 10 seconds
  last_health_check_ms = now;
  
  // Check for thermal issues
  for (size_t i = 0; i < num_sensors; ++i) {
    Sensor *s = &sensors[i];
    if (strcmp(s->type, "eps_temperature") == 0 && s->last_value > 40.0) {
      add_fault_log(13, 2, "High temperature detected in health check");
      recover_from_thermal_fault();
    }
  }
  
  // Check for power issues
  if (soc < 0.30) {
    add_fault_log(14, 2, "Low SOC detected in health check");
    recover_from_power_fault();
  }
  
  // Check for communication issues
  if (eps_state.payload_enabled && !uart_initialized) {
    add_fault_log(15, 2, "UART communication lost with payload");
    // Try to recover UART
    init_uart();
  }
  
  // Reset watchdog
  last_watchdog_reset_ms = now;
}

int main(void) {
  // Set up signal handlers for graceful shutdown
  signal(SIGINT, signal_handler);
  signal(SIGTERM, signal_handler);
  
  srand((unsigned)time(NULL));
  program_start_ms = now_ms();
  
  // Initialize mission data
  if (!load_mission_data()) {
    // First time running - initialize mission data
    mission_data.mission_start_ms = program_start_ms;
    mission_data.total_uptime_ms = 0;
    mission_data.reset_count = 1;
    mission_data.last_reset_ms = program_start_ms;
    mission_data.total_energy_wh = 0.0;
    mission_data.fault_count = 0;
    mission_data.mission_phase = 1; // IDLE phase
    strcpy(mission_data.last_fault, "System startup");
  }
  
  // Load EPS configuration
  if (!load_eps_config()) {
    // Use default EPS configuration
    printf("[EPS] Using default configuration\n");
  }
  
  // Load fault log if available
  FILE *f = fopen(FAULT_LOG_FILE, "rb");
  if (f) {
    fault_log_count = fread(fault_log, sizeof(FaultLogEntry), MAX_FAULT_ENTRIES, f);
    fclose(f);
    printf("[FAULT] Loaded %zu fault log entries\n", fault_log_count);
  }

  // Parse UART configuration from environment variables
  parse_uart_config();

  const char *yaml = getenv("SENSORS_YAML");
  if (!yaml) yaml = "software/flight/sensors.yaml";
  if (!load_yaml_config(yaml)) {
    fprintf(stderr, "[WARN] Using default sensors (could not load %s)\n", yaml);
    add_default_sensors();
  }
  for (size_t i = 0; i < num_sensors; ++i) {
    sensors[i].next_poll_ms = program_start_ms; // poll immediately
  }

  // Initialize UART for Jetson communication
  if (!init_uart()) {
    fprintf(stderr, "[WARN] UART initialization failed, telemetry disabled\n");
    add_fault_log(16, 2, "UART initialization failed");
  }

  printf("[SYSTEM] SMART-QSO Flight Software Started\n");
  printf("[SYSTEM] Mission Phase: %d, Power Mode: %d, SOC: %.1f%%\n", 
         mission_data.mission_phase, eps_state.power_mode, soc * 100.0);

  // Main loop: simulate 1,000 ticks (~1 s cadence)
  for (uint64_t tick = 0; tick < 1000 && !shutdown_requested; ++tick) {
    update_mission_state(tick);
    uint64_t now = now_ms();
    
    // System health monitoring
    monitor_system_health();
    
    // Check if it's time to send telemetry to Jetson
    if (eps_state.payload_enabled && uart_initialized && 
        (now - last_telemetry_ms) >= telemetry_interval_ms) {
      send_telemetry_to_jetson();
      last_telemetry_ms = now;
    }
    
    // Periodic UART health check (every 10 seconds)
    if (uart_initialized && (now % 10000) < 100) {
      check_uart_health();
    }
    
    // Sensor polling
    for (size_t i = 0; i < num_sensors; ++i) {
      Sensor *s = &sensors[i];
      if (now >= s->next_poll_ms) {
        double val = 0.0; char text[8] = {0};
        if (s->read(s, &val, text)) {
          if (s->value_type == SENSOR_NUMERIC) {
            s->last_value = val;
            printf("[READ] id=%s name=\"%s\" value=%.3f units=%s\n",
                   s->id, s->name, s->last_value, s->units);
          } else {
            strncpy(s->last_text, text, sizeof(s->last_text)-1);
            printf("[READ] id=%s name=\"%s\" value=%s units=%s\n",
                   s->id, s->name, s->last_text, s->units);
          }
        }
        s->next_poll_ms = now + (s->period_ms ? s->period_ms : 1000);
      }
    }
    
    // Periodic data persistence (every 100 ticks)
    if (tick % 100 == 0) {
      save_mission_data();
      save_eps_config();
    }
    
    struct timespec ts = {.tv_sec=0, .tv_nsec=20000000L}; // ~20 ms
    nanosleep(&ts, NULL);
  }
  
  // Graceful shutdown
  printf("[SYSTEM] Shutting down gracefully...\n");
  
  // Save final state
  save_mission_data();
  save_eps_config();
  
  // Cleanup
  close_uart();
  
  printf("[SYSTEM] Shutdown complete. Total uptime: %llu ms, Faults: %d\n",
         (unsigned long long)mission_data.total_uptime_ms, mission_data.fault_count);
  
  return 0;
}
