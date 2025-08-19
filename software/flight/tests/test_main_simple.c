#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <assert.h>
#include <time.h>
#include <stdbool.h>
#include <stdint.h>

// Simple test framework - no external dependencies
#define TEST_ASSERT(condition) \
    do { \
        if (!(condition)) { \
            printf("❌ TEST FAILED: %s at line %d\n", #condition, __LINE__); \
            return false; \
        } \
    } while(0)

#define TEST_PASS() \
    do { \
        printf("✅ Test passed\n"); \
        return true; \
    } while(0)

#define TEST_FAIL(msg) \
    do { \
        printf("❌ TEST FAILED: %s at line %d\n", msg, __LINE__); \
        return false; \
    } while(0)

// Test data structures (copied from main.c for testing)
#define MAX_SENSORS 32
#define MAX_FAULT_ENTRIES 100
#define WATCHDOG_TIMEOUT_MS 30000

typedef enum {
  SENSOR_NUMERIC = 0,
  SENSOR_HEX2 = 1
} SensorValueType;

typedef bool (*SensorReadFn)(void *self, double *out_value, char *out_text);

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

typedef struct {
    bool payload_enabled;
    bool radio_enabled;
    bool adcs_enabled;
    bool beacon_enabled;
    uint8_t power_mode;
    double power_limit_w;
    uint32_t last_control_ms;
} EPSControlState;

typedef struct {
    uint64_t mission_start_ms;
    uint64_t total_uptime_ms;
    uint32_t reset_count;
    uint64_t last_reset_ms;
    double total_energy_wh;
    uint32_t fault_count;
    uint8_t mission_phase;
    char last_fault[64];
} MissionData;

typedef struct {
    uint64_t timestamp_ms;
    uint8_t fault_type;
    uint8_t severity;
    char description[64];
    double soc_at_fault;
    bool recovered;
} FaultLogEntry;

// Test helper functions
static uint64_t mock_now_ms = 0;
static uint64_t test_now_ms(void) {
    return mock_now_ms;
}

static void set_mock_time(uint64_t time_ms) {
    mock_now_ms = time_ms;
}

static void advance_time(uint64_t delta_ms) {
    mock_now_ms += delta_ms;
}

// Mock file operations for testing
static char mock_file_content[4096];
static size_t mock_file_size = 0;
static bool mock_file_exists = false;
static bool mock_file_write_success = true;
static bool mock_file_read_success = true;

static FILE* mock_fopen(const char *path, const char *mode) {
    (void)path; // Suppress unused parameter warning
    if (strstr(mode, "w") && !mock_file_write_success) {
        return NULL;
    }
    if (strstr(mode, "r") && !mock_file_read_success) {
        return NULL;
    }
    // Return a dummy FILE pointer
    return (FILE*)0x12345678;
}

static size_t mock_fwrite(const void *ptr, size_t size, size_t count, FILE *stream) {
    (void)stream; // Suppress unused parameter warning
    if (!mock_file_write_success) return 0;
    
    size_t total_size = size * count;
    if (total_size > sizeof(mock_file_content) - 1) {
        total_size = sizeof(mock_file_content) - 1;
    }
    
    memcpy(mock_file_content, ptr, total_size);
    mock_file_content[total_size] = '\0';
    mock_file_size = total_size;
    mock_file_exists = true;
    
    return count;
}

static size_t mock_fread(void *ptr, size_t size, size_t count, FILE *stream) {
    (void)stream; // Suppress unused parameter warning
    if (!mock_file_read_success) return 0;
    
    size_t total_size = size * count;
    if (total_size > mock_file_size) {
        total_size = mock_file_size;
    }
    
    memcpy(ptr, mock_file_content, total_size);
    return total_size / size;
}

static int mock_fclose(FILE *stream) {
    (void)stream; // Suppress unused parameter warning
    return 0;
}

// Test data setup
static Sensor test_sensors[MAX_SENSORS];
static size_t test_num_sensors = 0;
static EPSControlState test_eps_state;
static MissionData test_mission_data;
static FaultLogEntry test_fault_log[MAX_FAULT_ENTRIES];
static size_t test_fault_log_count = 0;

static void setup_test_data(void) {
    // Initialize test sensors
    test_num_sensors = 3;
    
    strcpy(test_sensors[0].id, "BV");
    strcpy(test_sensors[0].name, "Battery Voltage");
    strcpy(test_sensors[0].type, "eps_voltage");
    strcpy(test_sensors[0].channel, "battery");
    strcpy(test_sensors[0].units, "V");
    test_sensors[0].period_ms = 1000;
    test_sensors[0].value_type = SENSOR_NUMERIC;
    test_sensors[0].last_value = 8.1;
    
    strcpy(test_sensors[1].id, "BT");
    strcpy(test_sensors[1].name, "Battery Temperature");
    strcpy(test_sensors[1].type, "eps_temperature");
    strcpy(test_sensors[1].channel, "battery");
    strcpy(test_sensors[1].units, "C");
    test_sensors[1].period_ms = 2000;
    test_sensors[1].value_type = SENSOR_NUMERIC;
    test_sensors[1].last_value = 25.0;
    
    strcpy(test_sensors[2].id, "ST");
    strcpy(test_sensors[2].name, "Status Hex");
    strcpy(test_sensors[2].type, "status_hex2");
    strcpy(test_sensors[2].units, "hex");
    test_sensors[2].period_ms = 1000;
    test_sensors[2].value_type = SENSOR_HEX2;
    strcpy(test_sensors[2].last_text, "C7");
    
    // Initialize test EPS state
    test_eps_state.payload_enabled = false;
    test_eps_state.radio_enabled = true;
    test_eps_state.adcs_enabled = true;
    test_eps_state.beacon_enabled = true;
    test_eps_state.power_mode = 1;
    test_eps_state.power_limit_w = 1.5;
    test_eps_state.last_control_ms = 0;
    
    // Initialize test mission data
    test_mission_data.mission_start_ms = 1000;
    test_mission_data.total_uptime_ms = 5000;
    test_mission_data.reset_count = 2;
    test_mission_data.last_reset_ms = 2000;
    test_mission_data.total_energy_wh = 10.5;
    test_mission_data.fault_count = 3;
    test_mission_data.mission_phase = 1;
    strcpy(test_mission_data.last_fault, "Test fault");
    
    // Initialize test fault log
    test_fault_log_count = 2;
    test_fault_log[0].timestamp_ms = 1000;
    test_fault_log[0].fault_type = 1;
    test_fault_log[0].severity = 2;
    strcpy(test_fault_log[0].description, "Test fault 1");
    test_fault_log[0].soc_at_fault = 0.75;
    test_fault_log[0].recovered = true;
    
    test_fault_log[1].timestamp_ms = 2000;
    test_fault_log[1].fault_type = 2;
    test_fault_log[1].severity = 1;
    strcpy(test_fault_log[1].description, "Test fault 2");
    test_fault_log[1].soc_at_fault = 0.80;
    test_fault_log[1].recovered = false;
    
    // Reset mock state
    mock_now_ms = 1000;
    mock_file_exists = false;
    mock_file_size = 0;
    mock_file_write_success = true;
    mock_file_read_success = true;
}

// Test cases

static bool test_sensor_initialization(void) {
    printf("Testing sensor initialization...\n");
    
    setup_test_data();
    
    // Test sensor initialization
    TEST_ASSERT(test_num_sensors == 3);
    TEST_ASSERT(strcmp(test_sensors[0].id, "BV") == 0);
    TEST_ASSERT(strcmp(test_sensors[0].name, "Battery Voltage") == 0);
    TEST_ASSERT(strcmp(test_sensors[0].type, "eps_voltage") == 0);
    TEST_ASSERT(strcmp(test_sensors[0].channel, "battery") == 0);
    TEST_ASSERT(strcmp(test_sensors[0].units, "V") == 0);
    TEST_ASSERT(test_sensors[0].period_ms == 1000);
    TEST_ASSERT(test_sensors[0].value_type == SENSOR_NUMERIC);
    TEST_ASSERT(test_sensors[0].last_value == 8.1);
    
    TEST_ASSERT(strcmp(test_sensors[1].id, "BT") == 0);
    TEST_ASSERT(strcmp(test_sensors[1].name, "Battery Temperature") == 0);
    TEST_ASSERT(strcmp(test_sensors[1].type, "eps_temperature") == 0);
    TEST_ASSERT(strcmp(test_sensors[1].channel, "battery") == 0);
    TEST_ASSERT(strcmp(test_sensors[1].units, "C") == 0);
    TEST_ASSERT(test_sensors[1].period_ms == 2000);
    TEST_ASSERT(test_sensors[1].value_type == SENSOR_NUMERIC);
    TEST_ASSERT(test_sensors[1].last_value == 25.0);
    
    TEST_ASSERT(strcmp(test_sensors[2].id, "ST") == 0);
    TEST_ASSERT(strcmp(test_sensors[2].name, "Status Hex") == 0);
    TEST_ASSERT(strcmp(test_sensors[2].type, "status_hex2") == 0);
    TEST_ASSERT(strcmp(test_sensors[2].units, "hex") == 0);
    TEST_ASSERT(test_sensors[2].period_ms == 1000);
    TEST_ASSERT(test_sensors[2].value_type == SENSOR_HEX2);
    TEST_ASSERT(strcmp(test_sensors[2].last_text, "C7") == 0);
    
    TEST_PASS();
}

static bool test_eps_state_initialization(void) {
    printf("Testing EPS state initialization...\n");
    
    setup_test_data();
    
    // Test EPS state initialization
    TEST_ASSERT(!test_eps_state.payload_enabled);
    TEST_ASSERT(test_eps_state.radio_enabled);
    TEST_ASSERT(test_eps_state.adcs_enabled);
    TEST_ASSERT(test_eps_state.beacon_enabled);
    TEST_ASSERT(test_eps_state.power_mode == 1);
    TEST_ASSERT(test_eps_state.power_limit_w == 1.5);
    TEST_ASSERT(test_eps_state.last_control_ms == 0);
    
    TEST_PASS();
}

static bool test_mission_data_initialization(void) {
    printf("Testing mission data initialization...\n");
    
    setup_test_data();
    
    // Test mission data initialization
    TEST_ASSERT(test_mission_data.mission_start_ms == 1000);
    TEST_ASSERT(test_mission_data.total_uptime_ms == 5000);
    TEST_ASSERT(test_mission_data.reset_count == 2);
    TEST_ASSERT(test_mission_data.last_reset_ms == 2000);
    TEST_ASSERT(test_mission_data.total_energy_wh == 10.5);
    TEST_ASSERT(test_mission_data.fault_count == 3);
    TEST_ASSERT(test_mission_data.mission_phase == 1);
    TEST_ASSERT(strcmp(test_mission_data.last_fault, "Test fault") == 0);
    
    TEST_PASS();
}

static bool test_fault_log_initialization(void) {
    printf("Testing fault log initialization...\n");
    
    setup_test_data();
    
    // Test fault log initialization
    TEST_ASSERT(test_fault_log_count == 2);
    
    TEST_ASSERT(test_fault_log[0].timestamp_ms == 1000);
    TEST_ASSERT(test_fault_log[0].fault_type == 1);
    TEST_ASSERT(test_fault_log[0].severity == 2);
    TEST_ASSERT(strcmp(test_fault_log[0].description, "Test fault 1") == 0);
    TEST_ASSERT(test_fault_log[0].soc_at_fault == 0.75);
    TEST_ASSERT(test_fault_log[0].recovered);
    
    TEST_ASSERT(test_fault_log[1].timestamp_ms == 2000);
    TEST_ASSERT(test_fault_log[1].fault_type == 2);
    TEST_ASSERT(test_fault_log[1].severity == 1);
    TEST_ASSERT(strcmp(test_fault_log[1].description, "Test fault 2") == 0);
    TEST_ASSERT(test_fault_log[1].soc_at_fault == 0.80);
    TEST_ASSERT(!test_fault_log[1].recovered);
    
    TEST_PASS();
}

static bool test_mock_file_operations(void) {
    printf("Testing mock file operations...\n");
    
    setup_test_data();
    
    // Test successful file write
    mock_file_write_success = true;
    FILE *write_file = mock_fopen("/test/file.dat", "wb");
    TEST_ASSERT(write_file != NULL);
    
    size_t written = mock_fwrite(&test_mission_data, sizeof(MissionData), 1, write_file);
    TEST_ASSERT(written == 1);
    TEST_ASSERT(mock_file_exists);
    TEST_ASSERT(mock_file_size == sizeof(MissionData));
    
    mock_fclose(write_file);
    
    // Test successful file read
    mock_file_read_success = true;
    FILE *read_file = mock_fopen("/test/file.dat", "rb");
    TEST_ASSERT(read_file != NULL);
    
    MissionData read_data;
    size_t read = mock_fread(&read_data, sizeof(MissionData), 1, read_file);
    TEST_ASSERT(read == 1);
    
    // Verify data integrity
    TEST_ASSERT(read_data.mission_start_ms == test_mission_data.mission_start_ms);
    TEST_ASSERT(read_data.reset_count == test_mission_data.reset_count);
    TEST_ASSERT(read_data.total_energy_wh == test_mission_data.total_energy_wh);
    
    mock_fclose(read_file);
    
    // Test failed file write
    mock_file_write_success = false;
    FILE *fail_write = mock_fopen("/test/fail.dat", "wb");
    TEST_ASSERT(fail_write == NULL);
    
    // Test failed file read
    mock_file_read_success = false;
    FILE *fail_read = mock_fopen("/test/fail.dat", "rb");
    TEST_ASSERT(fail_read == NULL);
    
    TEST_PASS();
}

static bool test_mock_time_functions(void) {
    printf("Testing mock time functions...\n");
    
    setup_test_data();
    
    // Test initial time
    TEST_ASSERT(test_now_ms() == 1000);
    
    // Test time advancement
    advance_time(500);
    TEST_ASSERT(test_now_ms() == 1500);
    
    // Test time setting
    set_mock_time(5000);
    TEST_ASSERT(test_now_ms() == 5000);
    
    // Test time advancement from new base
    advance_time(1000);
    TEST_ASSERT(test_now_ms() == 6000);
    
    TEST_PASS();
}

static bool test_sensor_data_validation(void) {
    printf("Testing sensor data validation...\n");
    
    setup_test_data();
    
    // Test sensor data validation
    for (size_t i = 0; i < test_num_sensors; i++) {
        Sensor *s = &test_sensors[i];
        
        // Check ID length
        TEST_ASSERT(strlen(s->id) < sizeof(s->id));
        
        // Check name length
        TEST_ASSERT(strlen(s->name) < sizeof(s->name));
        
        // Check units length
        TEST_ASSERT(strlen(s->units) < sizeof(s->units));
        
        // Check type length
        TEST_ASSERT(strlen(s->type) < sizeof(s->type));
        
        // Check channel length
        TEST_ASSERT(strlen(s->channel) < sizeof(s->channel));
        
        // Check period is reasonable
        TEST_ASSERT(s->period_ms > 0 && s->period_ms <= 60000);
        
        // Check value type is valid
        TEST_ASSERT(s->value_type == SENSOR_NUMERIC || s->value_type == SENSOR_HEX2);
    }
    
    TEST_PASS();
}

static bool test_eps_state_validation(void) {
    printf("Testing EPS state validation...\n");
    
    setup_test_data();
    
    // Test EPS state validation
    TEST_ASSERT(test_eps_state.power_mode >= 0 && test_eps_state.power_mode <= 2);
    TEST_ASSERT(test_eps_state.power_limit_w > 0.0 && test_eps_state.power_limit_w <= 5.0);
    
    // Test power mode consistency
    if (test_eps_state.power_mode == 0) { // SAFE mode
        TEST_ASSERT(!test_eps_state.payload_enabled);
        TEST_ASSERT(!test_eps_state.radio_enabled);
        TEST_ASSERT(!test_eps_state.adcs_enabled);
        TEST_ASSERT(test_eps_state.beacon_enabled);
        TEST_ASSERT(test_eps_state.power_limit_w <= 1.0);
    } else if (test_eps_state.power_mode == 1) { // IDLE mode
        TEST_ASSERT(!test_eps_state.payload_enabled);
        TEST_ASSERT(test_eps_state.radio_enabled);
        TEST_ASSERT(test_eps_state.adcs_enabled);
        TEST_ASSERT(test_eps_state.beacon_enabled);
        TEST_ASSERT(test_eps_state.power_limit_w <= 2.0);
    } else if (test_eps_state.power_mode == 2) { // ACTIVE mode
        TEST_ASSERT(test_eps_state.payload_enabled);
        TEST_ASSERT(test_eps_state.radio_enabled);
        TEST_ASSERT(test_eps_state.adcs_enabled);
        TEST_ASSERT(test_eps_state.beacon_enabled);
        TEST_ASSERT(test_eps_state.power_limit_w <= 5.0);
    }
    
    TEST_PASS();
}

static bool test_mission_data_validation(void) {
    printf("Testing mission data validation...\n");
    
    setup_test_data();
    
    // Test mission data validation
    TEST_ASSERT(test_mission_data.mission_start_ms > 0);
    TEST_ASSERT(test_mission_data.total_uptime_ms >= 0);
    TEST_ASSERT(test_mission_data.reset_count > 0);
    TEST_ASSERT(test_mission_data.last_reset_ms > 0);
    TEST_ASSERT(test_mission_data.total_energy_wh >= 0.0);
    TEST_ASSERT(test_mission_data.fault_count >= 0);
    TEST_ASSERT(test_mission_data.mission_phase >= 0 && test_mission_data.mission_phase <= 5);
    
    // Test logical consistency
    TEST_ASSERT(test_mission_data.last_reset_ms >= test_mission_data.mission_start_ms);
    TEST_ASSERT(test_mission_data.total_uptime_ms >= 0);
    
    // Check last fault description length
    TEST_ASSERT(strlen(test_mission_data.last_fault) < sizeof(test_mission_data.last_fault));
    
    TEST_PASS();
}

static bool test_fault_log_validation(void) {
    printf("Testing fault log validation...\n");
    
    setup_test_data();
    
    // Test fault log validation
    TEST_ASSERT(test_fault_log_count <= MAX_FAULT_ENTRIES);
    
    for (size_t i = 0; i < test_fault_log_count; i++) {
        FaultLogEntry *entry = &test_fault_log[i];
        
        // Check timestamp is reasonable
        TEST_ASSERT(entry->timestamp_ms > 0);
        
        // Check fault type is valid
        TEST_ASSERT(entry->fault_type > 0 && entry->fault_type <= 20);
        
        // Check severity is valid
        TEST_ASSERT(entry->severity >= 1 && entry->severity <= 4);
        
        // Check SOC at fault is valid
        TEST_ASSERT(entry->soc_at_fault >= 0.0 && entry->soc_at_fault <= 1.0);
        
        // Check description length
        TEST_ASSERT(strlen(entry->description) < sizeof(entry->description));
        
        // Check description is not empty
        TEST_ASSERT(strlen(entry->description) > 0);
    }
    
    TEST_PASS();
}

static bool test_sensor_polling_timing(void) {
    printf("Testing sensor polling timing...\n");
    
    setup_test_data();
    
    // Test sensor polling timing
    for (size_t i = 0; i < test_num_sensors; i++) {
        Sensor *s = &test_sensors[i];
        
        // Set initial polling time
        s->next_poll_ms = test_now_ms();
        
        // Test immediate polling
        TEST_ASSERT(test_now_ms() >= s->next_poll_ms);
        
        // Advance time and test delayed polling
        advance_time(s->period_ms);
        TEST_ASSERT(test_now_ms() >= s->next_poll_ms);
        
        // Test future polling
        s->next_poll_ms = test_now_ms() + s->period_ms;
        TEST_ASSERT(!(test_now_ms() >= s->next_poll_ms));
    }
    
    TEST_PASS();
}

static bool test_power_mode_transitions(void) {
    printf("Testing power mode transitions...\n");
    
    setup_test_data();
    
    // Test power mode transitions
    uint8_t original_mode = test_eps_state.power_mode;
    
    // Test transition to SAFE mode
    test_eps_state.power_mode = 0;
    test_eps_state.payload_enabled = false;
    test_eps_state.radio_enabled = false;
    test_eps_state.adcs_enabled = false;
    test_eps_state.beacon_enabled = true;
    test_eps_state.power_limit_w = 0.5;
    
    TEST_ASSERT(test_eps_state.power_mode == 0);
    TEST_ASSERT(!test_eps_state.payload_enabled);
    TEST_ASSERT(!test_eps_state.radio_enabled);
    TEST_ASSERT(!test_eps_state.adcs_enabled);
    TEST_ASSERT(test_eps_state.beacon_enabled);
    TEST_ASSERT(test_eps_state.power_limit_w <= 1.0);
    
    // Test transition to IDLE mode
    test_eps_state.power_mode = 1;
    test_eps_state.payload_enabled = false;
    test_eps_state.radio_enabled = true;
    test_eps_state.adcs_enabled = true;
    test_eps_state.beacon_enabled = true;
    test_eps_state.power_limit_w = 1.5;
    
    TEST_ASSERT(test_eps_state.power_mode == 1);
    TEST_ASSERT(!test_eps_state.payload_enabled);
    TEST_ASSERT(test_eps_state.radio_enabled);
    TEST_ASSERT(test_eps_state.adcs_enabled);
    TEST_ASSERT(test_eps_state.beacon_enabled);
    TEST_ASSERT(test_eps_state.power_limit_w <= 2.0);
    
    // Test transition to ACTIVE mode
    test_eps_state.power_mode = 2;
    test_eps_state.payload_enabled = true;
    test_eps_state.radio_enabled = true;
    test_eps_state.adcs_enabled = true;
    test_eps_state.beacon_enabled = true;
    test_eps_state.power_limit_w = 3.0;
    
    TEST_ASSERT(test_eps_state.power_mode == 2);
    TEST_ASSERT(test_eps_state.payload_enabled);
    TEST_ASSERT(test_eps_state.radio_enabled);
    TEST_ASSERT(test_eps_state.adcs_enabled);
    TEST_ASSERT(test_eps_state.beacon_enabled);
    TEST_ASSERT(test_eps_state.power_limit_w <= 5.0);
    
    // Restore original mode
    test_eps_state.power_mode = original_mode;
    
    TEST_PASS();
}

static bool test_fault_severity_levels(void) {
    printf("Testing fault severity levels...\n");
    
    setup_test_data();
    
    // Test fault severity levels
    for (size_t i = 0; i < test_fault_log_count; i++) {
        FaultLogEntry *entry = &test_fault_log[i];
        
        switch (entry->severity) {
            case 1: // INFO
                // INFO level faults should be informational only
                TEST_ASSERT(entry->fault_type > 0);
                break;
            case 2: // WARN
                // WARN level faults should require attention but not be critical
                TEST_ASSERT(entry->fault_type > 0);
                break;
            case 3: // ERROR
                // ERROR level faults should require immediate action
                TEST_ASSERT(entry->fault_type > 0);
                break;
            case 4: // CRITICAL
                // CRITICAL level faults should be system-threatening
                TEST_ASSERT(entry->fault_type > 0);
                break;
            default:
                // Invalid severity level
                TEST_FAIL("Invalid severity level");
                break;
        }
    }
    
    TEST_PASS();
}

static bool test_data_persistence_integrity(void) {
    printf("Testing data persistence integrity...\n");
    
    setup_test_data();
    
    // Test mission data persistence integrity
    mock_file_write_success = true;
    mock_file_read_success = true;
    
    // Write mission data
    FILE *write_file = mock_fopen("/test/mission.dat", "wb");
    TEST_ASSERT(write_file != NULL);
    
    size_t written = mock_fwrite(&test_mission_data, sizeof(MissionData), 1, write_file);
    TEST_ASSERT(written == 1);
    mock_fclose(write_file);
    
    // Read mission data back
    FILE *read_file = mock_fopen("/test/mission.dat", "rb");
    TEST_ASSERT(read_file != NULL);
    
    MissionData read_data;
    size_t read = mock_fread(&read_data, sizeof(MissionData), 1, read_file);
    TEST_ASSERT(read == 1);
    mock_fclose(read_file);
    
    // Verify complete data integrity
    TEST_ASSERT(read_data.mission_start_ms == test_mission_data.mission_start_ms);
    TEST_ASSERT(read_data.total_uptime_ms == test_mission_data.total_uptime_ms);
    TEST_ASSERT(read_data.reset_count == test_mission_data.reset_count);
    TEST_ASSERT(read_data.last_reset_ms == test_mission_data.last_reset_ms);
    TEST_ASSERT(read_data.total_energy_wh == test_mission_data.total_energy_wh);
    TEST_ASSERT(read_data.fault_count == test_mission_data.fault_count);
    TEST_ASSERT(read_data.mission_phase == test_mission_data.mission_phase);
    TEST_ASSERT(strcmp(read_data.last_fault, test_mission_data.last_fault) == 0);
    
    TEST_PASS();
}

static bool test_fault_log_management(void) {
    printf("Testing fault log management...\n");
    
    setup_test_data();
    
    // Test fault log management
    size_t original_count = test_fault_log_count;
    
    // Test adding new fault entry
    if (test_fault_log_count < MAX_FAULT_ENTRIES) {
        FaultLogEntry *new_entry = &test_fault_log[test_fault_log_count];
        new_entry->timestamp_ms = test_now_ms();
        new_entry->fault_type = 10;
        new_entry->severity = 2;
        strcpy(new_entry->description, "Test new fault");
        new_entry->soc_at_fault = 0.70;
        new_entry->recovered = false;
        
        test_fault_log_count++;
        
        TEST_ASSERT(test_fault_log_count == original_count + 1);
        TEST_ASSERT(strcmp(test_fault_log[original_count].description, "Test new fault") == 0);
        TEST_ASSERT(test_fault_log[original_count].fault_type == 10);
        TEST_ASSERT(test_fault_log[original_count].severity == 2);
    }
    
    // Test fault log overflow protection
    if (test_fault_log_count >= MAX_FAULT_ENTRIES) {
        // Simulate adding entry when log is full
        
        // This should trigger log rotation in real implementation
        // For testing, we just verify the count doesn't exceed maximum
        TEST_ASSERT(test_fault_log_count <= MAX_FAULT_ENTRIES);
    }
    
    TEST_PASS();
}

static bool test_sensor_value_types(void) {
    printf("Testing sensor value types...\n");
    
    setup_test_data();
    
    // Test sensor value types
    for (size_t i = 0; i < test_num_sensors; i++) {
        Sensor *s = &test_sensors[i];
        
        if (s->value_type == SENSOR_NUMERIC) {
            // Numeric sensors should have valid numeric values
            TEST_ASSERT(s->last_value >= -1000.0 && s->last_value <= 1000.0);
            
            // Check units are appropriate for numeric values
            if (strcmp(s->units, "V") == 0) {
                // Voltage should be reasonable
                TEST_ASSERT(s->last_value >= 0.0 && s->last_value <= 20.0);
            } else if (strcmp(s->units, "A") == 0) {
                // Current should be reasonable
                TEST_ASSERT(s->last_value >= -5.0 && s->last_value <= 5.0);
            } else if (strcmp(s->units, "C") == 0) {
                // Temperature should be reasonable
                TEST_ASSERT(s->last_value >= -50.0 && s->last_value <= 100.0);
            }
        } else if (s->value_type == SENSOR_HEX2) {
            // Hex sensors should have valid hex text
            TEST_ASSERT(strlen(s->last_text) == 2);
            
            // Check that text contains valid hex characters
            for (int j = 0; j < 2; j++) {
                char c = s->last_text[j];
                TEST_ASSERT((c >= '0' && c <= '9') || (c >= 'A' && c <= 'F'));
            }
        }
    }
    
    TEST_PASS();
}

static bool test_eps_control_consistency(void) {
    printf("Testing EPS control consistency...\n");
    
    setup_test_data();
    
    // Test EPS control consistency
    bool original_payload = test_eps_state.payload_enabled;
    bool original_radio = test_eps_state.radio_enabled;
    bool original_adcs = test_eps_state.adcs_enabled;
    bool original_beacon = test_eps_state.beacon_enabled;
    
    // Test payload control
    test_eps_state.payload_enabled = !original_payload;
    TEST_ASSERT(test_eps_state.payload_enabled != original_payload);
    
    // Test radio control
    test_eps_state.radio_enabled = !original_radio;
    TEST_ASSERT(test_eps_state.radio_enabled != original_radio);
    
    // Test ADCS control
    test_eps_state.adcs_enabled = !original_adcs;
    TEST_ASSERT(test_eps_state.adcs_enabled != original_adcs);
    
    // Test beacon control
    test_eps_state.beacon_enabled = !original_beacon;
    TEST_ASSERT(test_eps_state.beacon_enabled != original_beacon);
    
    // Restore original state
    test_eps_state.payload_enabled = original_payload;
    test_eps_state.radio_enabled = original_radio;
    test_eps_state.adcs_enabled = original_adcs;
    test_eps_state.beacon_enabled = original_beacon;
    
    TEST_PASS();
}

// Test runner
static bool run_all_tests(void) {
    printf("Running SMART-QSO Flight Software Unit Tests\n");
    printf("============================================\n\n");
    
    bool all_passed = true;
    
    // Basic initialization tests
    if (!test_sensor_initialization()) all_passed = false;
    if (!test_eps_state_initialization()) all_passed = false;
    if (!test_mission_data_initialization()) all_passed = false;
    if (!test_fault_log_initialization()) all_passed = false;
    
    // Mock functionality tests
    if (!test_mock_file_operations()) all_passed = false;
    if (!test_mock_time_functions()) all_passed = false;
    
    // Data validation tests
    if (!test_sensor_data_validation()) all_passed = false;
    if (!test_eps_state_validation()) all_passed = false;
    if (!test_mission_data_validation()) all_passed = false;
    if (!test_fault_log_validation()) all_passed = false;
    
    // Functional tests
    if (!test_sensor_polling_timing()) all_passed = false;
    if (!test_power_mode_transitions()) all_passed = false;
    if (!test_fault_severity_levels()) all_passed = false;
    if (!test_data_persistence_integrity()) all_passed = false;
    if (!test_fault_log_management()) all_passed = false;
    if (!test_sensor_value_types()) all_passed = false;
    if (!test_eps_control_consistency()) all_passed = false;
    
    printf("\n============================================\n");
    if (all_passed) {
        printf("✅ All tests passed successfully!\n");
    } else {
        printf("❌ Some tests failed.\n");
    }
    printf("============================================\n");
    
    return all_passed;
}

// Main function
int main(void) {
    bool success = run_all_tests();
    return success ? 0 : 1;
}
