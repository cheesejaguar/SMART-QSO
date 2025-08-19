# SMART-QSO Flight Software Unit Tests

This directory contains comprehensive unit tests for the SMART-QSO flight software (`main.c`). The tests cover all major functionality including EPS control, fault recovery, data persistence, and sensor operations.

## Test Files

### 1. `test_main.c` - Full CMocka Framework Tests
- **Framework**: Uses CMocka testing framework
- **Dependencies**: Requires CMocka library installation
- **Features**: Full test framework with setup/teardown, mock functions, and comprehensive assertions
- **Build**: Use `CMakeLists.txt`

### 2. `test_main_simple.c` - Standalone Tests
- **Framework**: Custom simple testing framework
- **Dependencies**: No external dependencies
- **Features**: Basic assertions, mock functions, and comprehensive test coverage
- **Build**: Use `CMakeLists_simple.txt`

## Test Coverage

### Core Data Structures
- ✅ **Sensor Framework**: Sensor initialization, validation, and management
- ✅ **EPS Control State**: Power mode management and system control
- ✅ **Mission Data**: Mission statistics and persistence
- ✅ **Fault Log**: Fault tracking and recovery status

### Functionality Tests
- ✅ **Sensor Operations**: Polling timing, value types, data validation
- ✅ **EPS Control**: Power mode transitions, system enable/disable
- ✅ **Fault Management**: Fault detection, severity levels, recovery
- ✅ **Data Persistence**: File operations, data integrity, recovery
- ✅ **Mock Functions**: Time simulation, file operations, error handling

### Validation Tests
- ✅ **Data Integrity**: Structure validation, range checking, consistency
- ✅ **State Consistency**: Power mode validation, system state coherence
- ✅ **Error Handling**: File operation failures, invalid data handling
- ✅ **Boundary Conditions**: Array limits, timing boundaries, value ranges

## Building and Running Tests

### Option 1: Simple Tests (Recommended for Quick Testing)

```bash
cd software/flight/tests
mkdir build_simple
cd build_simple
cmake -f ../CMakeLists_simple.txt ..
make
./smart_qso_simple_tests
```

### Option 2: Full CMocka Tests (Requires CMocka Installation)

```bash
cd software/flight/tests
mkdir build
cd build
cmake ..
make
./smart_qso_tests
```

### Option 3: Direct Compilation (No CMake)

```bash
cd software/flight/tests
gcc -o test_main_simple test_main_simple.c -Wall -Wextra -std=c99
./test_main_simple
```

## Test Output

### Successful Test Run
```
Running SMART-QSO Flight Software Unit Tests
============================================

Testing sensor initialization...
✅ Test passed
Testing EPS state initialization...
✅ Test passed
Testing mission data initialization...
✅ Test passed
Testing fault log initialization...
✅ Test passed
Testing mock file operations...
✅ Test passed
Testing mock time functions...
✅ Test passed
Testing sensor data validation...
✅ Test passed
Testing EPS state validation...
✅ Test passed
Testing mission data validation...
✅ Test passed
Testing fault log validation...
✅ Test passed
Testing sensor polling timing...
✅ Test passed
Testing power mode transitions...
✅ Test passed
Testing fault severity levels...
✅ Test passed
Testing data persistence integrity...
✅ Test passed
Testing fault log management...
✅ Test passed
Testing sensor value types...
✅ Test passed
Testing EPS control consistency...
✅ Test passed

============================================
✅ All tests passed successfully!
============================================
```

### Failed Test Example
```
Testing sensor data validation...
❌ TEST FAILED: strlen(s->id) < sizeof(s->id) at line 123
```

## Test Categories

### 1. Initialization Tests
- **Purpose**: Verify proper data structure initialization
- **Coverage**: All major data structures and their default values
- **Validation**: String lengths, numeric ranges, boolean states

### 2. Mock Function Tests
- **Purpose**: Test mock implementations for testing
- **Coverage**: File operations, time functions, error conditions
- **Validation**: Success/failure scenarios, data integrity

### 3. Data Validation Tests
- **Purpose**: Ensure data consistency and validity
- **Coverage**: Structure fields, value ranges, logical consistency
- **Validation**: Boundary conditions, type checking, format validation

### 4. Functional Tests
- **Purpose**: Test core system functionality
- **Coverage**: Power mode transitions, sensor polling, fault management
- **Validation**: State changes, timing, error handling

### 5. Integration Tests
- **Purpose**: Test component interactions
- **Coverage**: Data flow between components, state consistency
- **Validation**: End-to-end functionality, error propagation

## Mock Functions

### Time Simulation
- `test_now_ms()`: Returns mock current time
- `set_mock_time(time_ms)`: Sets mock time base
- `advance_time(delta_ms)`: Advances mock time

### File Operations
- `mock_fopen()`: Simulates file opening
- `mock_fwrite()`: Simulates file writing
- `mock_fread()`: Simulates file reading
- `mock_fclose()`: Simulates file closing

### Error Simulation
- File write/read failures
- Invalid data conditions
- Boundary condition violations

## Test Data

### Sensor Test Data
- **BV**: Battery Voltage sensor (8.1V, 1000ms period)
- **BT**: Battery Temperature sensor (25.0°C, 2000ms period)
- **ST**: Status Hex sensor (C7, 1000ms period)

### EPS Test State
- **Power Mode**: IDLE (1)
- **Power Limit**: 1.5W
- **Payload**: Disabled
- **Radio/ADCS/Beacon**: Enabled

### Mission Test Data
- **Start Time**: 1000ms
- **Uptime**: 5000ms
- **Reset Count**: 2
- **Energy**: 10.5Wh
- **Fault Count**: 3

### Fault Test Data
- **Fault 1**: Type 1, Severity 2, Recovered
- **Fault 2**: Type 2, Severity 1, Not Recovered

## Adding New Tests

### 1. Create Test Function
```c
static bool test_new_functionality(void) {
    printf("Testing new functionality...\n");
    
    // Setup test data
    setup_test_data();
    
    // Test assertions
    TEST_ASSERT(condition);
    
    // Verify results
    TEST_ASSERT(expected == actual);
    
    TEST_PASS();
}
```

### 2. Add to Test Runner
```c
// In run_all_tests()
if (!test_new_functionality()) all_passed = false;
```

### 3. Update Test Count
Ensure the test count in the summary matches the actual number of tests.

## Test Best Practices

### 1. Test Isolation
- Each test should be independent
- Use `setup_test_data()` for consistent initial state
- Avoid test interdependencies

### 2. Comprehensive Coverage
- Test normal operation
- Test error conditions
- Test boundary conditions
- Test edge cases

### 3. Clear Assertions
- Use descriptive test names
- Provide clear failure messages
- Test one concept per test function

### 4. Mock Functions
- Use mocks for external dependencies
- Simulate error conditions
- Verify mock function calls

## Troubleshooting

### Common Issues

#### 1. Compilation Errors
- Ensure C99 standard is enabled
- Check for missing includes
- Verify function signatures match

#### 2. Test Failures
- Check test data setup
- Verify mock function behavior
- Review assertion conditions

#### 3. Build Issues
- Ensure CMake version compatibility
- Check build directory permissions
- Verify source file paths

### Debug Mode
To enable debug output, add debug prints to test functions:
```c
printf("DEBUG: Testing value %d\n", test_value);
```

## Continuous Integration

### Automated Testing
- Tests can be integrated into CI/CD pipelines
- Use exit codes for pass/fail detection
- Generate test reports for analysis

### Test Metrics
- Test execution time
- Coverage percentage
- Pass/fail ratios
- Performance benchmarks

## Future Enhancements

### Planned Improvements
- **Performance Tests**: Timing and resource usage
- **Memory Tests**: Memory allocation and leaks
- **Stress Tests**: High-load conditions
- **Integration Tests**: Full system testing

### Framework Extensions
- **Parameterized Tests**: Multiple input combinations
- **Test Suites**: Grouped test execution
- **Mock Verification**: Verify mock function calls
- **Test Reporting**: Detailed test results

---

## Quick Start

For immediate testing without external dependencies:

```bash
cd software/flight/tests
gcc -o test_main_simple test_main_simple.c -Wall -Wextra -std=c99
./test_main_simple
```

This will run all tests and provide a comprehensive report of the flight software functionality.
