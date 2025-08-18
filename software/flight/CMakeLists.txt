cmake_minimum_required(VERSION 3.16)
project(smart_qso_flight C)
set(CMAKE_C_STANDARD 11)
add_compile_options(-Wall -Wextra -Wpedantic)
add_executable(smart_qso_flight
  src/main.c
)
# Placeholder for FreeRTOS/board ports in future commits

# Copy runtime sensor config next to the built binary
add_custom_command(TARGET smart_qso_flight POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          ${CMAKE_SOURCE_DIR}/sensors.yaml
          $<TARGET_FILE_DIR:smart_qso_flight>/sensors.yaml)
