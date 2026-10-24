# Cross-compile for the STM32L4R5ZI (Cortex-M4F) with the GNU Arm Embedded
# toolchain (FINDINGS_REGISTER SW-01).
#
#   cmake -S software/flight -B build-arm \
#         -DCMAKE_TOOLCHAIN_FILE=cmake/arm-none-eabi-stm32l4.cmake \
#         -DCMAKE_BUILD_TYPE=Release
#   cmake --build build-arm
#
# Builds the flight modules and the register-level STM32L4 HAL as a static
# library (smart_qso_target), and links the bare-metal flight image
# smart_qso_flight.elf with cmake/stm32l4r5zi.ld. No RTOS yet.

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR arm)

set(CMAKE_C_COMPILER arm-none-eabi-gcc)
set(CMAKE_AR arm-none-eabi-ar)
set(CMAKE_RANLIB arm-none-eabi-ranlib)

# A static library needs no startup code to link the compiler checks
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

# -fno-short-enums: C enums are 32 bits, as on the host and in the Rust
# modules (software/rust), so struct layouts shared with Rust match the ones
# the host tests check
set(CMAKE_C_FLAGS_INIT
    "-mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 -fno-short-enums -ffunction-sections -fdata-sections")

set(SMART_QSO_TARGET "STM32L4" CACHE STRING "HAL target" FORCE)

# Never pick up host programs' libraries or headers
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
