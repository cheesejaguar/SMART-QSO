###############################################################################
# SMART-QSO Flight Software CMake Configuration
#
# Compliant with NASA NPR 7150.2 software engineering requirements
# Following JPL Institutional Coding Standard (Power of Ten Rules)
#
# Build Commands:
#   cmake -S . -B build                    # Configure
#   cmake --build build                    # Build
#   ctest --test-dir build                 # Run tests
#   cmake -B build -DENABLE_COVERAGE=ON    # Enable coverage
#   cmake -B build -DFLIGHT_BUILD=ON       # Flight build
###############################################################################

cmake_minimum_required(VERSION 3.16)
project(smart_qso_flight C)

# C11 Standard (required by MISRA C:2012)
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

# Export compile commands for static analysis tools
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

###############################################################################
# Build Options
###############################################################################

option(FLIGHT_BUILD "Build for flight hardware" OFF)
option(ENABLE_COVERAGE "Enable code coverage" OFF)
option(ENABLE_SANITIZERS "Enable address/UB sanitizers" OFF)
option(ENABLE_WERROR "Treat warnings as errors" ON)

###############################################################################
# Compiler Flags (NASA/JPL Standard)
###############################################################################

# Base warning flags (JPL Rule 10)
add_compile_options(
    -Wall
    -Wextra
    -Wpedantic
    -Wconversion
    -Wshadow
    -Wcast-qual
    -Wstrict-prototypes
    -Wmissing-prototypes
    -Wformat=2
    -Wformat-security
    -Wundef
    -Wdouble-promotion
    -Wfloat-equal
    -Wpointer-arith
    -Wwrite-strings
    -Wswitch-enum
    -Wnull-dereference
)

# Treat warnings as errors (configurable)
if(ENABLE_WERROR)
    add_compile_options(-Werror)
endif()

# Security hardening flags
add_compile_options(
    -fstack-protector-strong
    -D_FORTIFY_SOURCE=2
)

# Flight build specific flags
if(FLIGHT_BUILD)
    add_definitions(-DFLIGHT_BUILD)
    # Disable debug output
    add_definitions(-DNDEBUG)
    # Could add: -Wl,--wrap=malloc to catch dynamic allocation
else()
    add_definitions(-DSIMULATION_BUILD)
endif()

# Coverage flags
if(ENABLE_COVERAGE)
    add_compile_options(--coverage -fprofile-arcs -ftest-coverage)
    add_link_options(--coverage)
endif()

# Sanitizer flags
if(ENABLE_SANITIZERS)
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer)
    add_link_options(-fsanitize=address,undefined)
endif()

###############################################################################
# Include Directories
###############################################################################

include_directories(${CMAKE_SOURCE_DIR}/include)

###############################################################################
# Source Files
###############################################################################

set(FLIGHT_SOURCES
    src/main.c
    src/eps_control.c
    src/fault_mgmt.c
    src/sensors.c
    src/uart_comm.c
    src/mission_data.c
    src/crc32.c
    src/time_utils.c
    src/beacon.c
    src/adcs_control.c
    src/input_validation.c
    src/safe_string.c
    src/state_machine.c
    src/system_state.c
    src/cmd_handler.c
    src/telemetry.c
    src/assert_handler.c
    src/watchdog_mgr.c
    src/flight_log.c
    src/deployment.c
    src/hal/hal_sim.c
)

set(FLIGHT_HEADERS
    include/smart_qso.h
    include/eps_control.h
    include/fault_mgmt.h
    include/sensors.h
    include/uart_comm.h
    include/mission_data.h
    include/beacon.h
    include/adcs_control.h
    include/input_validation.h
    include/safe_string.h
    include/state_machine.h
    include/system_state.h
    include/cmd_handler.h
    include/telemetry.h
    include/assert_handler.h
    include/watchdog_mgr.h
    include/flight_log.h
    include/deployment.h
    include/hal/hal.h
    include/hal/hal_gpio.h
    include/hal/hal_i2c.h
    include/hal/hal_uart.h
    include/hal/hal_spi.h
    include/hal/hal_timer.h
    include/hal/hal_adc.h
    include/hal/hal_flash.h
    include/hal/hal_wdt.h
)

###############################################################################
# Main Executable
###############################################################################

add_executable(smart_qso_flight ${FLIGHT_SOURCES})

# Link math library
target_link_libraries(smart_qso_flight m)

###############################################################################
# Post-Build Commands
###############################################################################

# Copy sensor config to build directory
add_custom_command(TARGET smart_qso_flight POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
            ${CMAKE_SOURCE_DIR}/sensors.yaml
            $<TARGET_FILE_DIR:smart_qso_flight>/sensors.yaml
    COMMENT "Copying sensor configuration"
)

###############################################################################
# Testing Configuration
###############################################################################

enable_testing()

# Check for cmocka
find_library(CMOCKA_LIB cmocka)
if(CMOCKA_LIB)
    add_subdirectory(tests)
else()
    message(WARNING "cmocka not found - unit tests disabled")
endif()

###############################################################################
# Installation
###############################################################################

install(TARGETS smart_qso_flight DESTINATION bin)
install(FILES sensors.yaml DESTINATION etc/smart_qso)

###############################################################################
# Documentation Target
###############################################################################

find_package(Doxygen)
if(DOXYGEN_FOUND)
    add_custom_target(docs
        COMMAND ${DOXYGEN_EXECUTABLE} ${CMAKE_SOURCE_DIR}/Doxyfile
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        COMMENT "Generating API documentation"
    )
endif()

###############################################################################
# Static Analysis Targets
###############################################################################

# cppcheck target
find_program(CPPCHECK cppcheck)
if(CPPCHECK)
    add_custom_target(cppcheck
        COMMAND ${CPPCHECK}
            --enable=all
            --inconclusive
            --std=c11
            --suppress=missingIncludeSystem
            --error-exitcode=1
            -I ${CMAKE_SOURCE_DIR}/include
            ${CMAKE_SOURCE_DIR}/src
        COMMENT "Running cppcheck static analysis"
    )
endif()

# clang-tidy target
find_program(CLANG_TIDY clang-tidy)
if(CLANG_TIDY)
    add_custom_target(clang-tidy
        COMMAND ${CLANG_TIDY}
            -p ${CMAKE_BINARY_DIR}
            ${FLIGHT_SOURCES}
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        COMMENT "Running clang-tidy static analysis"
    )
endif()

###############################################################################
# Print Configuration Summary
###############################################################################

message(STATUS "")
message(STATUS "SMART-QSO Flight Software Configuration:")
message(STATUS "  Build type:        ${CMAKE_BUILD_TYPE}")
message(STATUS "  Flight build:      ${FLIGHT_BUILD}")
message(STATUS "  Coverage:          ${ENABLE_COVERAGE}")
message(STATUS "  Sanitizers:        ${ENABLE_SANITIZERS}")
message(STATUS "  Warnings as errors: ${ENABLE_WERROR}")
message(STATUS "")
