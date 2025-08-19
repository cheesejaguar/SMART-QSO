# Unit Tests CMakeLists.txt for SMART-QSO Flight Software
# Uses cmocka testing framework

cmake_minimum_required(VERSION 3.10)
project(smart_qso_tests)

# Set C standard
set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Find cmocka package
find_package(PkgConfig REQUIRED)
pkg_check_modules(CMOCKA REQUIRED cmocka)

# Include directories
include_directories(${CMOCKA_INCLUDE_DIRS})

# Create test executable
add_executable(smart_qso_tests
    test_main.c
)

# Link against cmocka
target_link_libraries(smart_qso_tests ${CMOCKA_LIBRARIES})

# Set compiler flags
target_compile_options(smart_qso_tests PRIVATE ${CMOCKA_CFLAGS_OTHER})

# Add test
add_test(NAME UnitTests COMMAND smart_qso_tests)

# Set test properties
set_tests_properties(UnitTests PROPERTIES
    TIMEOUT 300
    ENVIRONMENT "CMOCKA_MESSAGE_OUTPUT=TAP"
)

# Enable testing
enable_testing()

# Print cmocka information
message(STATUS "CMocka include dirs: ${CMOCKA_INCLUDE_DIRS}")
message(STATUS "CMocka libraries: ${CMOCKA_LIBRARIES}")
message(STATUS "CMocka cflags: ${CMOCKA_CFLAGS_OTHER}")
