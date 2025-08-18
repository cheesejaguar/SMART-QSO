cmake_minimum_required(VERSION 3.16)
project(smart_qso_flight C)
set(CMAKE_C_STANDARD 11)
add_executable(smart_qso_flight
  src/main.c
)
# Placeholder for FreeRTOS/board ports in future commits
