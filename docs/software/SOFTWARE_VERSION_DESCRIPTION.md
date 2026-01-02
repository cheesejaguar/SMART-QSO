# Software Version Description
## SMART-QSO Flight Software

**Document Number:** SQSO-SVD-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2 §3.6

---

## 1. Purpose

This Software Version Description (SVD) identifies and describes a specific version of the SMART-QSO flight software, documenting its contents, build configuration, and verification status.

## 2. Version Identification

### 2.1 Release Information

| Field | Value |
|-------|-------|
| Software Name | SMART-QSO Flight Software |
| Version Number | 0.2.0 |
| Version Type | Development |
| Release Date | 2026-01-02 |
| Git Tag | v0.2.0-dev |
| Git Commit | [commit hash] |

### 2.2 Version History

| Version | Date | Type | Description |
|---------|------|------|-------------|
| 0.1.0 | 2025-xx-xx | Development | Initial skeleton |
| 0.2.0 | 2026-01-02 | Development | CDR candidate |
| 1.0.0 | TBD | Flight | Flight release |

## 3. Software Components

### 3.1 Source Files

| Module | Files | Lines | Description |
|--------|-------|-------|-------------|
| main | main.c | ~500 | Main application |
| sensors | sensors.c/h | ~600 | Sensor management |
| eps_control | eps_control.c/h | ~400 | Power system control |
| adcs_control | adcs_control.c/h | ~700 | Attitude control |
| beacon | beacon.c/h | ~600 | Beacon generation |
| fault_mgmt | fault_mgmt.c/h | ~400 | Fault management |
| uart_comm | uart_comm.c/h | ~400 | UART communication |
| mission_data | mission_data.c/h | ~300 | Data persistence |
| crc32 | crc32.c/h | ~150 | CRC calculation |
| time_utils | time_utils.c/h | ~100 | Time utilities |
| hal | hal/*.c/h | ~500 | Hardware abstraction |
| **Total** | | **~4650** | |

### 3.2 Test Files

| Test Module | Files | Tests | Coverage |
|-------------|-------|-------|----------|
| test_main | test_main.c | 15 | Core functions |
| test_sensors | test_sensors.c | 10 | Sensor logic |
| test_eps | test_eps.c | 12 | EPS control |
| test_crc32 | test_crc32.c | 5 | CRC validation |

### 3.3 Configuration Files

| File | Purpose |
|------|---------|
| CMakeLists.txt | Build configuration |
| .clang-tidy | Static analysis rules |
| .clang-format | Code formatting |
| .cppcheck | Cppcheck configuration |

## 4. Build Configuration

### 4.1 Compiler and Tools

| Tool | Version | Purpose |
|------|---------|---------|
| GCC | 12.x | C compiler |
| CMake | 3.20+ | Build system |
| clang-tidy | 17.x | Static analysis |
| cppcheck | 2.x | Static analysis |

### 4.2 Compiler Flags

```
-std=c11
-Wall -Wextra -Wpedantic -Werror
-Wconversion -Wshadow -Wcast-qual
-ffunction-sections -fdata-sections
-O2 (Release) / -O0 -g (Debug)
```

### 4.3 Build Variants

| Variant | Purpose | Debug | Assertions |
|---------|---------|-------|------------|
| Debug | Development | Yes | Yes |
| Release | Test builds | No | Yes |
| Flight | Flight software | No | Limited |

## 5. Dependencies

### 5.1 External Libraries

| Library | Version | License | Purpose |
|---------|---------|---------|---------|
| cmocka | 1.1.5 | Apache 2.0 | Unit testing |
| (none runtime) | - | - | No runtime deps |

### 5.2 Hardware Dependencies

| Component | Driver | Version |
|-----------|--------|---------|
| STM32L4xx | HAL | Mfr provided |
| SPI Flash | Custom | 1.0 |
| I2C Sensors | Custom | 1.0 |

## 6. Verification Status

### 6.1 Test Results

| Test Suite | Tests | Pass | Fail | Skip |
|------------|-------|------|------|------|
| Unit tests | 42 | 42 | 0 | 0 |
| Integration | TBD | - | - | - |
| System | TBD | - | - | - |

### 6.2 Static Analysis

| Tool | Findings | Resolved | Open |
|------|----------|----------|------|
| clang-tidy | TBD | TBD | 0 |
| cppcheck | TBD | TBD | 0 |

### 6.3 Code Coverage

| Metric | Target | Achieved |
|--------|--------|----------|
| Statement | 80% | TBD |
| Branch | 70% | TBD |
| Function | 100% | TBD |

## 7. Known Issues

### 7.1 Open Issues

| ID | Severity | Description | Workaround |
|----|----------|-------------|------------|
| None | - | No open issues | - |

### 7.2 Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| Simulation only | Not flight-ready | Hardware integration TBD |
| No FreeRTOS | Single-threaded | Future integration |

## 8. Change Summary

### 8.1 Changes from Previous Version

| Category | Changes |
|----------|---------|
| New Features | ADCS B-dot control, CRC32 protection |
| Bug Fixes | None (new code) |
| Enhancements | Modular architecture |
| Documentation | Complete design docs |

### 8.2 Requirements Implemented

| Requirement | Status | Test |
|-------------|--------|------|
| SRS-001 | Complete | test_eps_control |
| SRS-002 | Complete | test_fault_mgmt |
| SRS-003 | Complete | test_sensors |
| SRS-004 | Complete | test_beacon |
| SRS-005 | Complete | test_adcs |

## 9. Installation

### 9.1 Build Instructions

```bash
# Clone repository
git clone [repo-url]
cd SMART-QSO

# Build flight software
cmake -S software/flight -B build
cmake --build build

# Run tests
ctest --test-dir build
```

### 9.2 Deployment

| Target | Method |
|--------|--------|
| Simulation | Direct execution |
| Hardware | JTAG/SWD flash |

## 10. Configuration Items

### 10.1 Files in This Release

```
software/flight/
├── CMakeLists.txt
├── include/
│   ├── smart_qso.h
│   ├── sensors.h
│   ├── eps_control.h
│   ├── adcs_control.h
│   ├── beacon.h
│   ├── fault_mgmt.h
│   ├── uart_comm.h
│   ├── mission_data.h
│   ├── crc32.h
│   ├── time_utils.h
│   └── hal/
├── src/
│   ├── main.c
│   ├── sensors.c
│   ├── eps_control.c
│   ├── adcs_control.c
│   ├── beacon.c
│   ├── fault_mgmt.c
│   ├── uart_comm.c
│   ├── mission_data.c
│   ├── crc32.c
│   ├── time_utils.c
│   └── hal/
├── tests/
│   └── test_main.c
└── docs/
    ├── RADIATION_MITIGATION.md
    ├── BOOTLOADER_REQUIREMENTS.md
    ├── AUTONOMY_DESIGN.md
    ├── FDIR_DESIGN.md
    └── TIME_MANAGEMENT.md
```

## 11. Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Software Lead | | | |
| Systems Engineer | | | |
| Quality Assurance | | | |
| Project Manager | | | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
