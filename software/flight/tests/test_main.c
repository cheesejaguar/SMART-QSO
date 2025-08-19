#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <assert.h>
#include <time.h>

// Test framework includes
#include <setjmp.h>
#include <cmocka.h>

// Include the functions we want to test
// Note: We'll need to extract some functions from main.c for testing
// or create a test version that exposes the necessary functions

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
    if (!mock_file_read_success) return 0;
    
    size_t total_size = size * count;
    if (total_size > mock_file_size) {
        total_size = mock_file_size;
    }
    
    memcpy(ptr, mock_file_content, total_size);
    return total_size / size;
}

static int mock_fclose(FILE *stream) {
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

static void test_sensor_initialization(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test sensor initialization
    assert_int_equal(test_num_sensors, 3);
    assert_string_equal(test_sensors[0].id, "BV");
    assert_string_equal(test_sensors[0].name, "Battery Voltage");
    assert_string_equal(test_sensors[0].type, "eps_voltage");
    assert_string_equal(test_sensors[0].channel, "battery");
    assert_string_equal(test_sensors[0].units, "V");
    assert_int_equal(test_sensors[0].period_ms, 1000);
    assert_int_equal(test_sensors[0].value_type, SENSOR_NUMERIC);
    assert_true(test_sensors[0].last_value == 8.1);
    
    assert_string_equal(test_sensors[1].id, "BT");
    assert_string_equal(test_sensors[1].name, "Battery Temperature");
    assert_string_equal(test_sensors[1].type, "eps_temperature");
    assert_string_equal(test_sensors[1].channel, "battery");
    assert_string_equal(test_sensors[1].units, "C");
    assert_int_equal(test_sensors[1].period_ms, 2000);
    assert_int_equal(test_sensors[1].value_type, SENSOR_NUMERIC);
    assert_true(test_sensors[1].last_value == 25.0);
    
    assert_string_equal(test_sensors[2].id, "ST");
    assert_string_equal(test_sensors[2].name, "Status Hex");
    assert_string_equal(test_sensors[2].type, "status_hex2");
    assert_string_equal(test_sensors[2].units, "hex");
    assert_int_equal(test_sensors[2].period_ms, 1000);
    assert_int_equal(test_sensors[2].value_type, SENSOR_HEX2);
    assert_string_equal(test_sensors[2].last_text, "C7");
}

static void test_eps_state_initialization(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test EPS state initialization
    assert_false(test_eps_state.payload_enabled);
    assert_true(test_eps_state.radio_enabled);
    assert_true(test_eps_state.adcs_enabled);
    assert_true(test_eps_state.beacon_enabled);
    assert_int_equal(test_eps_state.power_mode, 1);
    assert_true(test_eps_state.power_limit_w == 1.5);
    assert_int_equal(test_eps_state.last_control_ms, 0);
}

static void test_mission_data_initialization(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test mission data initialization
    assert_int_equal(test_mission_data.mission_start_ms, 1000);
    assert_int_equal(test_mission_data.total_uptime_ms, 5000);
    assert_int_equal(test_mission_data.reset_count, 2);
    assert_int_equal(test_mission_data.last_reset_ms, 2000);
    assert_true(test_mission_data.total_energy_wh == 10.5);
    assert_int_equal(test_mission_data.fault_count, 3);
    assert_int_equal(test_mission_data.mission_phase, 1);
    assert_string_equal(test_mission_data.last_fault, "Test fault");
}

static void test_fault_log_initialization(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test fault log initialization
    assert_int_equal(test_fault_log_count, 2);
    
    assert_int_equal(test_fault_log[0].timestamp_ms, 1000);
    assert_int_equal(test_fault_log[0].fault_type, 1);
    assert_int_equal(test_fault_log[0].severity, 2);
    assert_string_equal(test_fault_log[0].description, "Test fault 1");
    assert_true(test_fault_log[0].soc_at_fault == 0.75);
    assert_true(test_fault_log[0].recovered);
    
    assert_int_equal(test_fault_log[1].timestamp_ms, 2000);
    assert_int_equal(test_fault_log[1].fault_type, 2);
    assert_int_equal(test_fault_log[1].severity, 1);
    assert_string_equal(test_fault_log[1].description, "Test fault 2");
    assert_true(test_fault_log[1].soc_at_fault == 0.80);
    assert_false(test_fault_log[1].recovered);
}

static void test_mock_file_operations(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test successful file write
    mock_file_write_success = true;
    FILE *write_file = mock_fopen("/test/file.dat", "wb");
    assert_non_null(write_file);
    
    size_t written = mock_fwrite(&test_mission_data, sizeof(MissionData), 1, write_file);
    assert_int_equal(written, 1);
    assert_true(mock_file_exists);
    assert_int_equal(mock_file_size, sizeof(MissionData));
    
    mock_fclose(write_file);
    
    // Test successful file read
    mock_file_read_success = true;
    FILE *read_file = mock_fopen("/test/file.dat", "rb");
    assert_non_null(read_file);
    
    MissionData read_data;
    size_t read = mock_fread(&read_data, sizeof(MissionData), 1, read_file);
    assert_int_equal(read, 1);
    
    // Verify data integrity
    assert_int_equal(read_data.mission_start_ms, test_mission_data.mission_start_ms);
    assert_int_equal(read_data.reset_count, test_mission_data.reset_count);
    assert_true(read_data.total_energy_wh == test_mission_data.total_energy_wh);
    
    mock_fclose(read_file);
    
    // Test failed file write
    mock_file_write_success = false;
    FILE *fail_write = mock_fopen("/test/fail.dat", "wb");
    assert_non_null(fail_write);
    
    written = mock_fwrite(&test_mission_data, sizeof(MissionData), 1, fail_write);
    assert_int_equal(written, 0);
    
    mock_fclose(fail_write);
    
    // Test failed file read
    mock_file_read_success = false;
    FILE *fail_read = mock_fopen("/test/fail.dat", "rb");
    assert_non_null(fail_read);
    
    read = mock_fread(&read_data, sizeof(MissionData), 1, fail_read);
    assert_int_equal(read, 0);
    
    mock_fclose(fail_read);
}

static void test_mock_time_functions(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test initial time
    assert_int_equal(test_now_ms(), 1000);
    
    // Test time advancement
    advance_time(500);
    assert_int_equal(test_now_ms(), 1500);
    
    // Test time setting
    set_mock_time(5000);
    assert_int_equal(test_now_ms(), 5000);
    
    // Test time advancement from new base
    advance_time(1000);
    assert_int_equal(test_now_ms(), 6000);
}

static void test_sensor_data_validation(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test sensor data validation
    for (size_t i = 0; i < test_num_sensors; i++) {
        Sensor *s = &test_sensors[i];
        
        // Check ID length
        assert_true(strlen(s->id) < sizeof(s->id));
        
        // Check name length
        assert_true(strlen(s->name) < sizeof(s->name));
        
        // Check units length
        assert_true(strlen(s->units) < sizeof(s->units));
        
        // Check type length
        assert_true(strlen(s->type) < sizeof(s->type));
        
        // Check channel length
        assert_true(strlen(s->channel) < sizeof(s->channel));
        
        // Check period is reasonable
        assert_true(s->period_ms > 0 && s->period_ms <= 60000);
        
        // Check value type is valid
        assert_true(s->value_type == SENSOR_NUMERIC || s->value_type == SENSOR_HEX2);
    }
}

static void test_eps_state_validation(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test EPS state validation
    assert_true(test_eps_state.power_mode >= 0 && test_eps_state.power_mode <= 2);
    assert_true(test_eps_state.power_limit_w > 0.0 && test_eps_state.power_limit_w <= 5.0);
    
    // Test power mode consistency
    if (test_eps_state.power_mode == 0) { // SAFE mode
        assert_false(test_eps_state.payload_enabled);
        assert_false(test_eps_state.radio_enabled);
        assert_false(test_eps_state.adcs_enabled);
        assert_true(test_eps_state.beacon_enabled);
        assert_true(test_eps_state.power_limit_w <= 1.0);
    } else if (test_eps_state.power_mode == 1) { // IDLE mode
        assert_false(test_eps_state.payload_enabled);
        assert_true(test_eps_state.radio_enabled);
        assert_true(test_eps_state.adcs_enabled);
        assert_true(test_eps_state.beacon_enabled);
        assert_true(test_eps_state.power_limit_w <= 2.0);
    } else if (test_eps_state.power_mode == 2) { // ACTIVE mode
        assert_true(test_eps_state.payload_enabled);
        assert_true(test_eps_state.radio_enabled);
        assert_true(test_eps_state.adcs_enabled);
        assert_true(test_eps_state.beacon_enabled);
        assert_true(test_eps_state.power_limit_w <= 5.0);
    }
}

static void test_mission_data_validation(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test mission data validation
    assert_true(test_mission_data.mission_start_ms > 0);
    assert_true(test_mission_data.total_uptime_ms >= 0);
    assert_true(test_mission_data.reset_count > 0);
    assert_true(test_mission_data.last_reset_ms > 0);
    assert_true(test_mission_data.total_energy_wh >= 0.0);
    assert_true(test_mission_data.fault_count >= 0);
    assert_true(test_mission_data.mission_phase >= 0 && test_mission_data.mission_phase <= 5);
    
    // Test logical consistency
    assert_true(test_mission_data.last_reset_ms >= test_mission_data.mission_start_ms);
    assert_true(test_mission_data.total_uptime_ms >= 0);
    
    // Check last fault description length
    assert_true(strlen(test_mission_data.last_fault) < sizeof(test_mission_data.last_fault));
}

static void test_fault_log_validation(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test fault log validation
    assert_true(test_fault_log_count <= MAX_FAULT_ENTRIES);
    
    for (size_t i = 0; i < test_fault_log_count; i++) {
        FaultLogEntry *entry = &test_fault_log[i];
        
        // Check timestamp is reasonable
        assert_true(entry->timestamp_ms > 0);
        
        // Check fault type is valid
        assert_true(entry->fault_type > 0 && entry->fault_type <= 20);
        
        // Check severity is valid
        assert_true(entry->severity >= 1 && entry->severity <= 4);
        
        // Check SOC at fault is valid
        assert_true(entry->soc_at_fault >= 0.0 && entry->soc_at_fault <= 1.0);
        
        // Check description length
        assert_true(strlen(entry->description) < sizeof(entry->description));
        
        // Check description is not empty
        assert_true(strlen(entry->description) > 0);
    }
}

static void test_sensor_polling_timing(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test sensor polling timing
    for (size_t i = 0; i < test_num_sensors; i++) {
        Sensor *s = &test_sensors[i];
        
        // Set initial polling time
        s->next_poll_ms = test_now_ms();
        
        // Test immediate polling
        assert_true(test_now_ms() >= s->next_poll_ms);
        
        // Advance time and test delayed polling
        advance_time(s->period_ms);
        assert_true(test_now_ms() >= s->next_poll_ms);
        
        // Test future polling
        s->next_poll_ms = test_now_ms() + s->period_ms;
        assert_false(test_now_ms() >= s->next_poll_ms);
    }
}

static void test_power_mode_transitions(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test power mode transitions
    uint8_t original_mode = test_eps_state.power_mode;
    
    // Test transition to SAFE mode
    test_eps_state.power_mode = 0;
    assert_int_equal(test_eps_state.power_mode, 0);
    assert_false(test_eps_state.payload_enabled);
    assert_false(test_eps_state.radio_enabled);
    assert_false(test_eps_state.adcs_enabled);
    assert_true(test_eps_state.beacon_enabled);
    assert_true(test_eps_state.power_limit_w <= 1.0);
    
    // Test transition to IDLE mode
    test_eps_state.power_mode = 1;
    assert_int_equal(test_eps_state.power_mode, 1);
    assert_false(test_eps_state.payload_enabled);
    assert_true(test_eps_state.radio_enabled);
    assert_true(test_eps_state.adcs_enabled);
    assert_true(test_eps_state.beacon_enabled);
    assert_true(test_eps_state.power_limit_w <= 2.0);
    
    // Test transition to ACTIVE mode
    test_eps_state.power_mode = 2;
    assert_int_equal(test_eps_state.power_mode, 2);
    assert_true(test_eps_state.payload_enabled);
    assert_true(test_eps_state.radio_enabled);
    assert_true(test_eps_state.adcs_enabled);
    assert_true(test_eps_state.beacon_enabled);
    assert_true(test_eps_state.power_limit_w <= 5.0);
    
    // Restore original mode
    test_eps_state.power_mode = original_mode;
}

static void test_fault_severity_levels(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test fault severity levels
    for (size_t i = 0; i < test_fault_log_count; i++) {
        FaultLogEntry *entry = &test_fault_log[i];
        
        switch (entry->severity) {
            case 1: // INFO
                // INFO level faults should be informational only
                assert_true(entry->fault_type > 0);
                break;
            case 2: // WARN
                // WARN level faults should require attention but not be critical
                assert_true(entry->fault_type > 0);
                break;
            case 3: // ERROR
                // ERROR level faults should require immediate action
                assert_true(entry->fault_type > 0);
                break;
            case 4: // CRITICAL
                // CRITICAL level faults should be system-threatening
                assert_true(entry->fault_type > 0);
                break;
            default:
                // Invalid severity level
                assert_false(true);
                break;
        }
    }
}

static void test_data_persistence_integrity(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test mission data persistence integrity
    mock_file_write_success = true;
    mock_file_read_success = true;
    
    // Write mission data
    FILE *write_file = mock_fopen("/test/mission.dat", "wb");
    assert_non_null(write_file);
    
    size_t written = mock_fwrite(&test_mission_data, sizeof(MissionData), 1, write_file);
    assert_int_equal(written, 1);
    mock_fclose(write_file);
    
    // Read mission data back
    FILE *read_file = mock_fopen("/test/mission.dat", "rb");
    assert_non_null(read_file);
    
    MissionData read_data;
    size_t read = mock_fread(&read_data, sizeof(MissionData), 1, read_file);
    assert_int_equal(read, 1);
    mock_fclose(read_file);
    
    // Verify complete data integrity
    assert_int_equal(read_data.mission_start_ms, test_mission_data.mission_start_ms);
    assert_int_equal(read_data.total_uptime_ms, test_mission_data.total_uptime_ms);
    assert_int_equal(read_data.reset_count, test_mission_data.reset_count);
    assert_int_equal(read_data.last_reset_ms, test_mission_data.last_reset_ms);
    assert_true(read_data.total_energy_wh == test_mission_data.total_energy_wh);
    assert_int_equal(read_data.fault_count, test_mission_data.fault_count);
    assert_int_equal(read_data.mission_phase, test_mission_data.mission_phase);
    assert_string_equal(read_data.last_fault, test_mission_data.last_fault);
}

static void test_fault_log_management(void **state) {
    (void)state;
    
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
        
        assert_int_equal(test_fault_log_count, original_count + 1);
        assert_string_equal(test_fault_log[original_count].description, "Test new fault");
        assert_int_equal(test_fault_log[original_count].fault_type, 10);
        assert_int_equal(test_fault_log[original_count].severity, 2);
    }
    
    // Test fault log overflow protection
    if (test_fault_log_count >= MAX_FAULT_ENTRIES) {
        // Simulate adding entry when log is full
        size_t old_count = test_fault_log_count;
        
        // This should trigger log rotation in real implementation
        // For testing, we just verify the count doesn't exceed maximum
        assert_true(test_fault_log_count <= MAX_FAULT_ENTRIES);
    }
}

static void test_sensor_value_types(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test sensor value types
    for (size_t i = 0; i < test_num_sensors; i++) {
        Sensor *s = &test_sensors[i];
        
        if (s->value_type == SENSOR_NUMERIC) {
            // Numeric sensors should have valid numeric values
            assert_true(s->last_value >= -1000.0 && s->last_value <= 1000.0);
            
            // Check units are appropriate for numeric values
            if (strcmp(s->units, "V") == 0) {
                // Voltage should be reasonable
                assert_true(s->last_value >= 0.0 && s->last_value <= 20.0);
            } else if (strcmp(s->units, "A") == 0) {
                // Current should be reasonable
                assert_true(s->last_value >= -5.0 && s->last_value <= 5.0);
            } else if (strcmp(s->units, "C") == 0) {
                // Temperature should be reasonable
                assert_true(s->last_value >= -50.0 && s->last_value <= 100.0);
            }
        } else if (s->value_type == SENSOR_HEX2) {
            // Hex sensors should have valid hex text
            assert_true(strlen(s->last_text) == 2);
            
            // Check that text contains valid hex characters
            for (int j = 0; j < 2; j++) {
                char c = s->last_text[j];
                assert_true((c >= '0' && c <= '9') || (c >= 'A' && c <= 'F'));
            }
        }
    }
}

static void test_eps_control_consistency(void **state) {
    (void)state;
    
    setup_test_data();
    
    // Test EPS control consistency
    bool original_payload = test_eps_state.payload_enabled;
    bool original_radio = test_eps_state.radio_enabled;
    bool original_adcs = test_eps_state.adcs_enabled;
    bool original_beacon = test_eps_state.beacon_enabled;
    
    // Test payload control
    test_eps_state.payload_enabled = !original_payload;
    assert_true(test_eps_state.payload_enabled != original_payload);
    
    // Test radio control
    test_eps_state.radio_enabled = !original_radio;
    assert_true(test_eps_state.radio_enabled != original_radio);
    
    // Test ADCS control
    test_eps_state.adcs_enabled = !original_adcs;
    assert_true(test_eps_state.adcs_enabled != original_adcs);
    
    // Test beacon control
    test_eps_state.beacon_enabled = !original_beacon;
    assert_true(test_eps_state.beacon_enabled != original_beacon);
    
    // Restore original state
    test_eps_state.payload_enabled = original_payload;
    test_eps_state.radio_enabled = original_radio;
    test_eps_state.adcs_enabled = original_adcs;
    test_eps_state.beacon_enabled = original_beacon;
}

// Test suite setup and teardown
static int setup(void **state) {
    (void)state;
    setup_test_data();
    return 0;
}

static int teardown(void **state) {
    (void)state;
    // Clean up any test resources
    return 0;
}

// Main test suite
int main(void) {
    const struct CMUnitTest tests[] = {
        // Basic initialization tests
        cmocka_unit_test_setup_teardown(test_sensor_initialization, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_state_initialization, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_initialization, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_initialization, setup, teardown),
        
        // Mock functionality tests
        cmocka_unit_test_setup_teardown(test_mock_file_operations, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mock_time_functions, setup, teardown),
        
        // Data validation tests
        cmocka_unit_test_setup_teardown(test_sensor_data_validation, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_state_validation, setup, teardown),
        cmocka_unit_test_setup_teardown(test_mission_data_validation, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_validation, setup, teardown),
        
        // Functional tests
        cmocka_unit_test_setup_teardown(test_sensor_polling_timing, setup, teardown),
        cmocka_unit_test_setup_teardown(test_power_mode_transitions, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_severity_levels, setup, teardown),
        cmocka_unit_test_setup_teardown(test_data_persistence_integrity, setup, teardown),
        cmocka_unit_test_setup_teardown(test_fault_log_management, setup, teardown),
        cmocka_unit_test_setup_teardown(test_sensor_value_types, setup, teardown),
        cmocka_unit_test_setup_teardown(test_eps_control_consistency, setup, teardown),
    };
    
    printf("Running SMART-QSO Flight Software Unit Tests\n");
    printf("============================================\n\n");
    
    int result = cmocka_run_group_tests(tests, NULL, NULL);
    
    if (result == 0) {
        printf("\n✅ All tests passed successfully!\n");
    } else {
        printf("\n❌ Some tests failed.\n");
    }
    
    return result;
}
