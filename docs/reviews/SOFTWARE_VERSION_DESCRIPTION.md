# SMART-QSO Software Version Description

**Document ID**: SQSO-SVD-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the Software Version Description (SVD) for the SMART-QSO flight software per NPR 7150.2 §4.5.4 requirements. It serves as the authoritative record of delivered software configuration.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| NPR 7150.2 | NASA Software Engineering Requirements |
| SQSO-SRS-001 | Software Requirements Specification |
| SQSO-SDD-001 | Software Design Document |

## 3. Software Identification

### 3.1 Flight Software (OBC)

| Item | Value |
|------|-------|
| **Product Name** | SMART-QSO Flight Software |
| **Version** | 1.0.0 |
| **Release Date** | TBD (Pending flatsat testing) |
| **Git Tag** | v1.0.0-cdr |
| **Git SHA** | TBD |
| **Classification** | Class B (Mission Critical) |

### 3.2 Payload Software (Jetson)

| Item | Value |
|------|-------|
| **Product Name** | SMART-QSO Payload Software |
| **Version** | 1.0.0 |
| **Release Date** | TBD |
| **Git Tag** | v1.0.0-cdr |
| **Git SHA** | TBD |

### 3.3 Ground Software

| Item | Value |
|------|-------|
| **Product Name** | SMART-QSO Ground Software |
| **Version** | 1.0.0 |
| **Release Date** | TBD |
| **Git Tag** | v1.0.0-cdr |
| **Git SHA** | TBD |

## 4. Build Environment

### 4.1 Flight Software Build Environment

| Component | Version | Source |
|-----------|---------|--------|
| Compiler | arm-none-eabi-gcc 12.2.0 | ARM Developer |
| CMake | 3.26+ | cmake.org |
| Host OS | Ubuntu 22.04 LTS | Canonical |
| FreeRTOS | 10.5.1 | freertos.org |
| CMSIS | 5.9.0 | ARM |
| Unity | 2.5.2 | ThrowTheSwitch |

### 4.2 Payload Software Build Environment

| Component | Version | Source |
|-----------|---------|--------|
| Python | 3.10+ | python.org |
| llama-cpp-python | 0.2.0+ | PyPI |
| JetPack | 6.0 | NVIDIA |

### 4.3 Ground Software Build Environment

| Component | Version | Source |
|-----------|---------|--------|
| Python | 3.10+ | python.org |
| pytest | 7.0+ | PyPI |

## 5. Software Components

### 5.1 Flight Software Architecture

```
software/flight/
├── src/
│   ├── main.c                    # Entry point, FreeRTOS init
│   ├── state_machine.c           # SAFE/IDLE/ACTIVE state machine
│   ├── telemetry.c               # Telemetry collection and formatting
│   ├── beacon.c                  # AX.25 beacon generation
│   ├── power_manager.c           # Power mode management
│   ├── watchdog.c                # Watchdog timer service
│   ├── health_monitor.c          # System health monitoring
│   ├── jetson_interface.c        # Payload communication
│   ├── command_handler.c         # Ground command processing
│   ├── fault_handler.c           # Fault detection and response
│   ├── uart_comm.c               # UART driver
│   ├── i2c_driver.c              # I2C driver
│   ├── spi_driver.c              # SPI driver
│   ├── time_utils.c              # Time management
│   ├── adcs_control.c            # ADCS interface
│   ├── crc.c                     # CRC calculations
│   └── assert_handler.c          # Assertion handling
├── include/
│   └── [corresponding headers]
├── tests/
│   └── [22 test suites, 300+ tests]
└── CMakeLists.txt
```

### 5.2 Payload Software Architecture

```
software/payload/
├── agent.py                      # Main agent loop
├── llama_runner.py               # LLM inference wrapper
├── formatter.py                  # Beacon text formatting
├── health.py                     # Health data parsing
└── tests/
    └── [80 tests across 3 suites]
```

### 5.3 Ground Software Architecture

```
software/ground/
├── beacon_decoder.py             # AX.25 beacon decoder
├── telemetry_display.py          # Telemetry visualization
├── kiss_protocol.py              # KISS TNC protocol
└── tests/
    └── [98 tests across 2 suites]
```

## 6. Functional Description

### 6.1 Flight Software Functions

| Function | Description | Priority |
|----------|-------------|----------|
| State Machine | Manages SAFE/IDLE/ACTIVE modes | Critical |
| Watchdog | 5-second hardware watchdog service | Critical |
| Power Management | Battery monitoring, load management | Critical |
| Telemetry | Sensor collection, packet formatting | High |
| Beacon Generation | AX.25 frame creation, scheduling | High |
| Payload Interface | UART communication with Jetson | Medium |
| Command Processing | Ground command parsing, execution | High |
| Fault Handling | Detection, logging, recovery | Critical |
| ADCS Interface | Magnetometer, sun sensor reading | Medium |

### 6.2 Payload Software Functions

| Function | Description |
|----------|-------------|
| Health Parsing | Parse OBC health JSON |
| LLM Inference | Generate beacon text using TinyLM |
| Fallback Mode | Template beacons when AI unavailable |
| Text Formatting | Fit text to 200-byte beacon limit |

### 6.3 Ground Software Functions

| Function | Description |
|----------|-------------|
| Beacon Decoding | Parse AX.25 frames, extract telemetry |
| KISS Protocol | TNC interface |
| Telemetry Display | Real-time visualization, limits |
| Data Export | CSV/JSON export |

## 7. Memory Map

### 7.1 Flash Memory (1 MB)

| Region | Address Range | Size | Contents |
|--------|---------------|------|----------|
| Bootloader | 0x08000000 - 0x08007FFF | 32 KB | System bootloader |
| App Header | 0x08008000 - 0x080083FF | 1 KB | Version, checksum |
| Application | 0x08008400 - 0x0807FFFF | 478 KB | Flight software |
| Config | 0x08080000 - 0x0808FFFF | 64 KB | Configuration data |
| Reserved | 0x08090000 - 0x080FFFFF | 448 KB | Future use |

### 7.2 RAM (256 KB)

| Region | Address Range | Size | Contents |
|--------|---------------|------|----------|
| Stack | 0x20000000 - 0x20003FFF | 16 KB | Main stack |
| Heap | 0x20004000 - 0x2000BFFF | 32 KB | Dynamic allocation |
| BSS | 0x2000C000 - 0x2001FFFF | 80 KB | Static data |
| Telemetry | 0x20020000 - 0x20023FFF | 16 KB | Telemetry buffers |
| Reserved | 0x20024000 - 0x2003FFFF | 112 KB | Future use |

## 8. Resource Utilization

### 8.1 Memory Usage (CDR Estimate)

| Resource | Allocated | Used | Margin |
|----------|-----------|------|--------|
| Flash (code) | 478 KB | ~150 KB | 69% |
| Flash (config) | 64 KB | ~4 KB | 94% |
| RAM (static) | 80 KB | ~30 KB | 63% |
| RAM (heap) | 32 KB | ~8 KB | 75% |
| RAM (stack) | 16 KB | ~4 KB | 75% |

### 8.2 CPU Utilization (CDR Estimate)

| Task | Priority | Period | WCET | Utilization |
|------|----------|--------|------|-------------|
| Watchdog | Highest | 1s | <1 ms | <0.1% |
| Telemetry | High | 100 ms | ~5 ms | ~5% |
| Beacon | Medium | 60s | ~50 ms | <0.1% |
| Health | Low | 10s | ~10 ms | <0.1% |
| Idle | Lowest | - | - | ~94% |

## 9. Requirements Traceability

### 9.1 Verification Summary

| Category | Total | Verified | Pending |
|----------|-------|----------|---------|
| Functional | 45 | 40 | 5 |
| Performance | 15 | 10 | 5 |
| Interface | 12 | 10 | 2 |
| Safety | 8 | 8 | 0 |
| Quality | 10 | 10 | 0 |
| **Total** | **90** | **78** | **12** |

*Pending items require flatsat hardware verification*

### 9.2 Test Summary

| Test Type | Suites | Tests | Pass | Fail |
|-----------|--------|-------|------|------|
| Flight Unit Tests | 22 | 300+ | All | 0 |
| Payload Unit Tests | 3 | 80 | All | 0 |
| Ground Unit Tests | 2 | 98 | All | 0 |
| Integration Tests | - | - | Pending | - |
| System Tests | - | - | Pending | - |

## 10. Known Issues

See SQSO-KI-001 (Known Issues Document) for complete list.

**Summary**:
- No critical issues at CDR
- No high-severity issues at CDR
- HAL implementations require target hardware headers

## 11. Deviations and Waivers

| ID | Requirement | Deviation | Rationale | Approval |
|----|-------------|-----------|-----------|----------|
| None | - | No deviations at this time | - | - |

## 12. Change History from Previous Version

This is the initial release (v1.0.0-cdr).

## 13. Limitations and Constraints

### 13.1 Known Limitations

1. **AI Inference**: Limited to 10% duty cycle due to power constraints
2. **Beacon Rate**: Fixed at 60-second intervals
3. **Command Rate**: Maximum 1 command per second
4. **Storage**: No persistent logging beyond boot cycle

### 13.2 Environmental Constraints

1. **Temperature**: -40°C to +85°C (OBC operational)
2. **Radiation**: LEO environment, no specific hardening
3. **Power**: Minimum 3.0V battery voltage for operation

## 14. Build Instructions

### 14.1 Flight Software

```bash
# Prerequisites
sudo apt install gcc-arm-none-eabi cmake

# Clone and checkout
git clone https://github.com/cheesejaguar/SMART-QSO.git
cd SMART-QSO
git checkout v1.0.0-cdr

# Build
cmake -S software/flight -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build

# Run tests
ctest --test-dir build --output-on-failure
```

### 14.2 Payload Software

```bash
# Prerequisites
pip install llama-cpp-python

# Run tests
cd software/payload
python -m pytest tests/ -v
```

### 14.3 Ground Software

```bash
# Prerequisites
pip install pytest

# Run tests
cd software/ground
python -m pytest tests/ -v
```

## 15. Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Software Lead | | | |
| Systems Engineer | | | |
| Quality Assurance | | | |
| Project Manager | | | |

## 16. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR baseline |
