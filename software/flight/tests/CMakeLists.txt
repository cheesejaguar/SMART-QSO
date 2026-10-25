# Unit Tests CMakeLists.txt for SMART-QSO Flight Software
# NASA NPR 7150.2 Compliant Testing Framework
#
# Uses CMocka testing framework for unit tests
# Target: 80% statement coverage, 75% branch coverage

cmake_minimum_required(VERSION 3.16)
project(smart_qso_tests C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Find cmocka package
find_package(PkgConfig)
if(PkgConfig_FOUND)
    pkg_check_modules(CMOCKA IMPORTED_TARGET cmocka)
endif()

if(NOT CMOCKA_FOUND)
    message(WARNING "CMocka not found - unit tests will not be built")
    message(STATUS "Install CMocka: brew install cmocka (macOS) or apt install libcmocka-dev (Linux)")
    return()
endif()

message(STATUS "CMocka found - building unit tests")
message(STATUS "  Include dirs: ${CMOCKA_INCLUDE_DIRS}")
message(STATUS "  Libraries: ${CMOCKA_LIBRARIES}")

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/../include
    ${CMAKE_CURRENT_SOURCE_DIR}/../src
    ${CMOCKA_INCLUDE_DIRS}
)

# Source files from main project needed for tests
set(FLIGHT_SOURCES
)

# Common compile options for test builds
set(TEST_COMPILE_OPTIONS
    -Wall -Wextra
    -DSIMULATION_BUILD=1
    -DSMART_QSO_DEBUG_ENABLED=1
)

# Enable testing
enable_testing()

#===========================================================================
# Test: CRC32
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_crc32.c")
    add_executable(test_crc32
        test_crc32.c
    )
    # CRC-32 is implemented in Rust (sqso_rust, through flight_test_lib)
    target_link_libraries(test_crc32 flight_test_lib PkgConfig::CMOCKA)
    target_compile_options(test_crc32 PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME CRC32_Tests COMMAND test_crc32)
    set_tests_properties(CRC32_Tests PROPERTIES
        TIMEOUT 60
        LABELS "unit;crc"
    )
endif()

#===========================================================================
# Test: EPS Control
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_eps_control.c")
    add_executable(test_eps_control
        test_eps_control.c
    )
    target_link_libraries(test_eps_control flight_test_lib PkgConfig::CMOCKA)
    target_compile_options(test_eps_control PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME EPS_Control_Tests COMMAND test_eps_control)
    set_tests_properties(EPS_Control_Tests PROPERTIES
        TIMEOUT 120
        LABELS "unit;eps"
    )
endif()

#===========================================================================
# Test: Fault Management
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_fault_mgmt.c")
    add_executable(test_fault_mgmt
        test_fault_mgmt.c
    )
    target_link_libraries(test_fault_mgmt flight_test_lib PkgConfig::CMOCKA)
    target_compile_options(test_fault_mgmt PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME Fault_Mgmt_Tests COMMAND test_fault_mgmt)
    set_tests_properties(Fault_Mgmt_Tests PROPERTIES
        TIMEOUT 120
        LABELS "unit;fdir"
    )
endif()

#===========================================================================
# Test: Sensors
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_sensors.c")
    add_executable(test_sensors
        test_sensors.c
    )
    target_link_libraries(test_sensors flight_test_lib PkgConfig::CMOCKA m)
    target_compile_options(test_sensors PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME Sensor_Tests COMMAND test_sensors)
    set_tests_properties(Sensor_Tests PROPERTIES
        TIMEOUT 120
        LABELS "unit;sensors"
    )
endif()

#===========================================================================
# Test: Mission Data
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_mission_data.c")
    add_executable(test_mission_data
        test_mission_data.c
    )
    target_link_libraries(test_mission_data flight_test_lib PkgConfig::CMOCKA)
    target_compile_options(test_mission_data PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME Mission_Data_Tests COMMAND test_mission_data)
    set_tests_properties(Mission_Data_Tests PROPERTIES
        TIMEOUT 120
        LABELS "unit;data"
    )
endif()

#===========================================================================
# Test: UART Communication
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_uart_comm.c")
    add_executable(test_uart_comm
        test_uart_comm.c
    )
    target_link_libraries(test_uart_comm flight_test_lib PkgConfig::CMOCKA)
    target_compile_options(test_uart_comm PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME UART_Comm_Tests COMMAND test_uart_comm)
    set_tests_properties(UART_Comm_Tests PROPERTIES
        TIMEOUT 120
        LABELS "unit;uart"
    )
endif()

#===========================================================================
# Shared flight library for module tests
#
# Tests below link this library and the flight software (Rust, sqso_rust).
# Its only C source is the flight-log shim (stable Rust cannot define a C
# variadic function).
#===========================================================================
set(FLIGHT_LIB_SOURCES
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/flight_log_fmt.c
)
add_library(flight_test_lib STATIC ${FLIGHT_LIB_SOURCES})
target_compile_options(flight_test_lib PRIVATE ${TEST_COMPILE_OPTIONS})
# The Rust modules (cmake/rust.cmake); every test linking flight_test_lib gets them
link_rust(flight_test_lib)

# Module tests: name, label
set(MODULE_TESTS
    "test_assert_handler\;assert"
    "test_deployment\;deployment"
    "test_flight_log\;log"
    "test_input_validation\;validation"
    "test_safe_string\;string"
    "test_state_machine\;state"
    "test_system_state\;state"
    "test_watchdog_mgr\;watchdog"
    "test_sha256\;security"
    "test_nvm_record\;nvm"
    "test_rf_control\;rf"
    "test_cmd_auth\;security"
    "test_cmd_handler\;command"
    "test_gse_provision\;security"
    "test_ax25\;rf"
    "test_uplink\;rf"
    "test_contact_monitor\;rf"
    "test_rf_silence\;rf"
    "test_beacon_format\;rf"
    "test_jetson_link\;payload"
    "test_beacon_filter\;rf"
    "test_beacon_telemetry\;rf"
    "test_gnss\;gnss"
    "test_payload_guard\;payload"
    "test_assert_release\;assert"
    "test_scheduler\;scheduler"
    "test_beacon\;rf"
    "test_telemetry\;telemetry"
    "test_telemetry_flow\;integration"
    "test_adcs\;adcs"
    "test_rust_contract\;rust"
    "test_rust_contract_adcs\;rust"
)
foreach(entry ${MODULE_TESTS})
    list(GET entry 0 test_name)
    list(GET entry 1 test_label)
    add_executable(${test_name} ${test_name}.c)
    target_link_libraries(${test_name} flight_test_lib PkgConfig::CMOCKA m)
    target_compile_options(${test_name} PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME ${test_name} COMMAND ${test_name})
    set_tests_properties(${test_name} PROPERTIES
        TIMEOUT 120
        LABELS "unit;${test_label}"
    )
endforeach()

# Integration and regression suites: <directory>/<name>, labelled by directory
set(SUITE_TESTS
    "integration/test_mode_transitions"
    "integration/test_fault_scenarios"
    "integration/test_system_startup"
    "regression/test_regression_suite"
)
foreach(entry ${SUITE_TESTS})
    get_filename_component(test_name ${entry} NAME)
    get_filename_component(test_label ${entry} DIRECTORY)
    add_executable(${test_name} ${entry}.c)
    target_link_libraries(${test_name} flight_test_lib PkgConfig::CMOCKA m)
    target_compile_options(${test_name} PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME ${test_name} COMMAND ${test_name})
    set_tests_properties(${test_name} PROPERTIES
        TIMEOUT 120
        LABELS "${test_label}"
    )
endforeach()
# JPL rule 3: no flight object references malloc/calloc/realloc/free (the
# Rust library included: it is no_std without an allocator)
add_test(NAME no_dynamic_memory
         COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/../scripts/check_no_dynamic_memory.sh
                 $<TARGET_OBJECTS:flight_test_lib> ${SQSO_RUST_LIB})
set_tests_properties(no_dynamic_memory PROPERTIES LABELS "static;jpl")

# Accept/reject vectors shared with the Jetson-side filter
target_compile_definitions(test_beacon_filter PRIVATE
    BEACON_FILTER_VECTORS="${CMAKE_CURRENT_SOURCE_DIR}/vectors/beacon_filter_vectors.txt")

#===========================================================================
# Test isolation
#===========================================================================
# The simulated fault log, EPS configuration and mission data persist to
# files. Give every test its own files so
# parallel ctest runs (ci.yml uses --parallel) cannot interfere.
get_property(_sqso_tests DIRECTORY PROPERTY TESTS)
foreach(_sqso_test IN LISTS _sqso_tests)
    set_property(TEST ${_sqso_test} APPEND PROPERTY ENVIRONMENT
        "SMART_QSO_FAULT_LOG=${CMAKE_CURRENT_BINARY_DIR}/fault_log_${_sqso_test}.dat"
        "SMART_QSO_EPS_CONFIG=${CMAKE_CURRENT_BINARY_DIR}/eps_config_${_sqso_test}.dat"
        "SMART_QSO_MISSION_DATA=${CMAKE_CURRENT_BINARY_DIR}/mission_data_${_sqso_test}.dat")
endforeach()

#===========================================================================
# Coverage Target
#===========================================================================
if(ENABLE_COVERAGE)
    find_program(LCOV lcov)
    find_program(GENHTML genhtml)

    if(LCOV AND GENHTML)
        add_custom_target(coverage
            COMMAND ${LCOV} --capture --directory . --output-file coverage.info
            COMMAND ${LCOV} --remove coverage.info '/usr/*' '*/tests/*' --output-file coverage.info
            COMMAND ${GENHTML} coverage.info --output-directory coverage_report
            COMMAND ${LCOV} --summary coverage.info
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
            COMMENT "Generating code coverage report"
        )
    else()
        message(STATUS "lcov/genhtml not found - coverage target disabled")
    endif()
endif()

#===========================================================================
# Test Summary
#===========================================================================
message(STATUS "")
message(STATUS "SMART-QSO Unit Test Configuration")
message(STATUS "==================================")
message(STATUS "Test framework: CMocka ${CMOCKA_VERSION}")
message(STATUS "Coverage target: ${ENABLE_COVERAGE}")
message(STATUS "")

#===========================================================================
# The host simulation end to end: about 20 s of operation (1000 passes of
# the executive) from the repository root, so sensors.yaml is loaded. It
# must shut down cleanly. Its persistent files are its own, so it can run
# beside the other suites. Not in the flight configuration, which runs
# until stopped.
#===========================================================================
if(NOT FLIGHT_BUILD)
    add_test(NAME Simulation_Run COMMAND ${CMAKE_BINARY_DIR}/smart_qso_flight)
    set_tests_properties(Simulation_Run PROPERTIES
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/../.."
        TIMEOUT 120
        PASS_REGULAR_EXPRESSION "\\[SYSTEM\\] Shutdown complete"
        FAIL_REGULAR_EXPRESSION "Initialization failed;Rust panic"
        ENVIRONMENT "SMART_QSO_FAULT_LOG=${CMAKE_BINARY_DIR}/sim_fault_log.dat;SMART_QSO_EPS_CONFIG=${CMAKE_BINARY_DIR}/sim_eps_config.dat;SMART_QSO_MISSION_DATA=${CMAKE_BINARY_DIR}/sim_mission_data.dat;SMART_QSO_STATE_FILE=${CMAKE_BINARY_DIR}/sim_state.dat;SENSORS_YAML=${CMAKE_SOURCE_DIR}/sensors.yaml"
        LABELS "integration")

    # The executive's Jetson link: the same simulation with a pseudo-terminal
    # as the Jetson port, the driver playing the Jetson (about 21 s each).
    # The driver links no flight code: it checks the lines with its own CRC.
    add_executable(test_executive_jetson integration/test_executive_jetson.c)
    target_link_libraries(test_executive_jetson PkgConfig::CMOCKA)
    target_compile_options(test_executive_jetson PRIVATE ${TEST_COMPILE_OPTIONS})
    add_dependencies(test_executive_jetson smart_qso_flight)
    foreach(_sqso_scenario link guard)
        set(_sqso_test Executive_Jetson_${_sqso_scenario})
        add_test(NAME ${_sqso_test}
                 COMMAND test_executive_jetson ${CMAKE_BINARY_DIR}/smart_qso_flight ${_sqso_scenario}
                         ${CMAKE_SOURCE_DIR}/sensors.yaml)
        set_tests_properties(${_sqso_test} PROPERTIES
            TIMEOUT 150
            ENVIRONMENT "SMART_QSO_FAULT_LOG=${CMAKE_CURRENT_BINARY_DIR}/fault_log_${_sqso_test}.dat;SMART_QSO_EPS_CONFIG=${CMAKE_CURRENT_BINARY_DIR}/eps_config_${_sqso_test}.dat;SMART_QSO_MISSION_DATA=${CMAKE_CURRENT_BINARY_DIR}/mission_data_${_sqso_test}.dat;SMART_QSO_STATE_FILE=${CMAKE_CURRENT_BINARY_DIR}/state_${_sqso_test}.dat"
            LABELS "integration")
    endforeach()
endif()
