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
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/eps_control.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/fault_mgmt.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/sensors.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/mission_data.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/uart_comm.c
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
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    )
    target_link_libraries(test_crc32 PkgConfig::CMOCKA)
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
# Tests below link against all flight sources except the executive
# (main.c) and the two units that do not yet compile (system_init.c,
# yaml_parser.c - see docs/funding_audit/FINDINGS_REGISTER.md). The
# static library only pulls in the objects each test needs.
#===========================================================================
set(FLIGHT_LIB_SOURCES
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/adcs_control.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/assert_handler.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/beacon.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/beacon_filter.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/cmd_auth.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/cmd_handler.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/cmd_dictionary.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/gse_provision.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/ax25.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/uplink.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/contact_monitor.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/rf_silence.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/separation_switch.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/jetson_link.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/gnss.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/payload_guard.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/deployment.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/eps_control.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/fault_mgmt.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/flight_log.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/input_validation.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/mission_data.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/nvm_record.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/rf_control.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/safe_string.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/scheduler.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/sensors.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/sha256.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/state_machine.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/system_state.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/telemetry.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/uart_comm.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/watchdog_mgr.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../src/hal/hal_sim.c
)
add_library(flight_test_lib STATIC ${FLIGHT_LIB_SOURCES})
target_compile_options(flight_test_lib PRIVATE ${TEST_COMPILE_OPTIONS})

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
# JPL rule 3: no flight object references malloc/calloc/realloc/free
add_test(NAME no_dynamic_memory
         COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/../scripts/check_no_dynamic_memory.sh
                 $<TARGET_OBJECTS:flight_test_lib>)
set_tests_properties(no_dynamic_memory PROPERTIES LABELS "static;jpl")

# Accept/reject vectors shared with the Jetson-side filter
target_compile_definitions(test_beacon_filter PRIVATE
    BEACON_FILTER_VECTORS="${CMAKE_CURRENT_SOURCE_DIR}/vectors/beacon_filter_vectors.txt")

#===========================================================================
# Not registered (tracked in docs/funding_audit/FINDINGS_REGISTER.md):
#   - test_adcs, test_beacon, test_scheduler, test_system_init,
#     test_telemetry, test_telemetry_flow: written for Unity, which is not
#     vendored; port to CMocka
#   - test_main_simple.c: tests local copies of structs, not flight code
#===========================================================================

#===========================================================================
# Test: Legacy Main Tests (DISABLED - superseded by proper module tests)
#===========================================================================
# NOTE: test_main.c is obsolete legacy code with many issues.
# Proper module-specific tests are now in test_*.c files.
# if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_main.c")
#     add_executable(test_main_legacy
#         test_main.c
#     )
#     target_link_libraries(test_main_legacy PkgConfig::CMOCKA)
#     target_compile_options(test_main_legacy PRIVATE ${TEST_COMPILE_OPTIONS})
#     add_test(NAME Legacy_Tests COMMAND test_main_legacy)
#     set_tests_properties(Legacy_Tests PROPERTIES
#         TIMEOUT 300
#         LABELS "unit;legacy"
#     )
# endif()

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
