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

// Object-oriented-ish sensor framework with YAML configuration.
// Aligns with CONOPS: always-on OBC monitors EPS & status; Jetson bursts are
// simulated via current draw flags in the status bitmap.

// UART configuration for Jetson communication (configurable via environment variables)
#define UART_DEVICE_DEFAULT "/dev/ttyUSB0"  // Default UART device
#define UART_BAUDRATE_DEFAULT B115200       // Default baud rate
#define TELEMETRY_INTERVAL_MS_DEFAULT 60000 // Default: 60 seconds

// Runtime configurable values
static char uart_device[64] = UART_DEVICE_DEFAULT;
static speed_t uart_baudrate = UART_BAUDRATE_DEFAULT;
static uint32_t telemetry_interval_ms = TELEMETRY_INTERVAL_MS_DEFAULT;

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

// UART communication state
static int uart_fd = -1;
static uint64_t last_telemetry_ms = 0;
static bool uart_initialized = false;

// Simple mission state simulation
static uint64_t program_start_ms;
static bool sunlit = true;
static bool jetson_enabled = false;
static double soc = 0.75; // 0..1

static uint64_t now_ms(void) {
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  return (uint64_t)ts.tv_sec * 1000ULL + (uint64_t)(ts.tv_nsec / 1000000ULL);
}

static double rnd_unit(void) {
  return (double)rand() / (double)RAND_MAX;
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

// Send telemetry data to Jetson
static void send_telemetry_to_jetson(void) {
  if (!jetson_enabled || !uart_initialized) return;
  
  char buffer[1024];
  int offset = 0;
  
  // Start telemetry packet with timestamp and mission state
  offset += snprintf(buffer + offset, sizeof(buffer) - offset, 
                    "TELEMETRY,%llu,%s,%.3f,", 
                    (unsigned long long)now_ms(),
                    sunlit ? "SUNLIT" : "ECLIPSE",
                    soc);
  
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
  
  // Add UART health status
  offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                    "UART_HEALTH=%s,", uart_initialized ? "OK" : "FAIL");
  
  // Remove trailing comma and add newline
  if (offset > 0 && buffer[offset-1] == ',') {
    buffer[offset-1] = '\n';
  } else {
    buffer[offset] = '\n';
    offset++;
  }
  
  // Send via UART
  if (send_uart_data(buffer, offset)) {
    printf("[UART] Sent telemetry to Jetson (%d bytes): %s", offset, buffer);
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
    close_uart();
    if (jetson_enabled) {
      init_uart(); // Try to reinitialize
    }
  }
}

// Sensor read implementations
static bool read_software_timer(Sensor *self, double *out_value, char *out_text) {
  (void)self; (void)out_text;
  uint64_t ms = now_ms() - program_start_ms;
  *out_value = (double)(ms / 1000ULL);
  return true;
}

static bool read_eps_voltage(Sensor *self, double *out_value, char *out_text) {
  (void)out_text;
  double base = 0.0;
  if (strcmp(self->channel, "battery") == 0) base = 8.1 + 0.15 * (soc - 0.5);
  else if (strcmp(self->channel, "bus") == 0) base = 5.0;
  else if (strcmp(self->channel, "solar") == 0) base = sunlit ? 7.5 : 0.2;
  *out_value = base + 0.02 * (rnd_unit() - 0.5);
  return true;
}

static bool read_eps_current(Sensor *self, double *out_value, char *out_text) {
  (void)out_text;
  double val = 0.0;
  if (strcmp(self->channel, "battery_discharge") == 0) {
    val = sunlit ? 0.05 : (jetson_enabled ? 0.8 : 0.25);
  } else if (strcmp(self->channel, "jetson") == 0) {
    val = jetson_enabled ? 0.7 + 0.05 * (rnd_unit() - 0.5) : 0.0;
  } else if (strcmp(self->channel, "solar") == 0) {
    val = sunlit ? (0.6 + 0.1 * (rnd_unit() - 0.5)) : 0.0;
  }
  *out_value = val;
  return true;
}

static bool read_eps_temperature(Sensor *self, double *out_value, char *out_text) {
  (void)self; (void)out_text;
  double base = sunlit ? 22.0 : 14.0;
  if (jetson_enabled) base += 3.0;
  *out_value = base + 0.5 * (rnd_unit() - 0.5);
  return true;
}

static bool read_status_hex2(Sensor *self, double *out_value, char *out_text) {
  (void)self; (void)out_value;
  // Bitmap (LSB..):
  // bit0: OBC on; bit1: Beacon on; bit2: Jetson enabled; bit3: Thermal hot;
  // bit4: Battery low; bit5: Safe mode; bits6-7: reserved
  uint8_t bits = 0;
  bits |= 1u << 0; // OBC on
  bits |= 1u << 1; // Beacon on
  if (jetson_enabled) bits |= 1u << 2;
  if (sunlit && (now_ms() / 1000ULL) % 300ULL > 240ULL) bits |= 1u << 3; // occasional hot
  if (soc < 0.30) bits |= 1u << 4;
  if (soc < 0.25) bits |= 1u << 5;
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

static void update_mission_state(uint64_t tick) {
  sunlit = (tick % 180) < 100; // ~55% sunlit cycle
  // Simple SOC integration
  double discharge = jetson_enabled ? 0.0040 : 0.0010;
  double charge = 0.0020;
  if (sunlit) soc += charge; else soc -= discharge;
  if (soc > 1.0) soc = 1.0; if (soc < 0.0) soc = 0.0;
  // Enable Jetson only when sunlit and SOC healthy
  jetson_enabled = sunlit && soc > 0.55;
}

int main(void) {
  srand((unsigned)time(NULL));
  program_start_ms = now_ms();

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
  }

  // Main loop: simulate 1,000 ticks (~1 s cadence)
  for (uint64_t tick = 0; tick < 1000; ++tick) {
    update_mission_state(tick);
    uint64_t now = now_ms();
    
    // Check if it's time to send telemetry to Jetson
    if (jetson_enabled && uart_initialized && 
        (now - last_telemetry_ms) >= telemetry_interval_ms) {
      send_telemetry_to_jetson();
      last_telemetry_ms = now;
    }
    
    // Periodic UART health check (every 10 seconds)
    if (uart_initialized && (now % 10000) < 100) {
      check_uart_health();
    }
    
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
    struct timespec ts = {.tv_sec=0, .tv_nsec=20000000L}; // ~20 ms
    nanosleep(&ts, NULL);
  }
  
  // Cleanup
  close_uart();
  return 0;
}
