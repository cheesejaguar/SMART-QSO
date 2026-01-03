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
    pkg_check_modules(CMOCKA cmocka)
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
    target_link_libraries(test_crc32 ${CMOCKA_LIBRARIES})
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
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/eps_control.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/fault_mgmt.c
    )
    target_link_libraries(test_eps_control ${CMOCKA_LIBRARIES})
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
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/fault_mgmt.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/eps_control.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    )
    target_link_libraries(test_fault_mgmt ${CMOCKA_LIBRARIES})
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
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/sensors.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/eps_control.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/fault_mgmt.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    )
    target_link_libraries(test_sensors ${CMOCKA_LIBRARIES} m)
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
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/mission_data.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    )
    target_link_libraries(test_mission_data ${CMOCKA_LIBRARIES})
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
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/uart_comm.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/crc32.c
        ${CMAKE_CURRENT_SOURCE_DIR}/../src/time_utils.c
    )
    target_link_libraries(test_uart_comm ${CMOCKA_LIBRARIES})
    target_compile_options(test_uart_comm PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME UART_Comm_Tests COMMAND test_uart_comm)
    set_tests_properties(UART_Comm_Tests PROPERTIES
        TIMEOUT 120
        LABELS "unit;uart"
    )
endif()

#===========================================================================
# Test: Legacy Main Tests (if present)
#===========================================================================
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test_main.c")
    add_executable(test_main_legacy
        test_main.c
    )
    target_link_libraries(test_main_legacy ${CMOCKA_LIBRARIES})
    target_compile_options(test_main_legacy PRIVATE ${TEST_COMPILE_OPTIONS})
    add_test(NAME Legacy_Tests COMMAND test_main_legacy)
    set_tests_properties(Legacy_Tests PROPERTIES
        TIMEOUT 300
        LABELS "unit;legacy"
    )
endif()

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
