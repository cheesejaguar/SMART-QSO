# SMART-QSO Flight Software Delivery Package

**Document ID**: SQSO-FSW-DEL-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the complete flight software delivery package for the SMART-QSO mission per NPR 7150.2 requirements.

## 2. Software Version Description

### 2.1 Flight Software (OBC)

| Item | Value |
|------|-------|
| Name | SMART-QSO Flight Software |
| Version | 1.0.0 |
| Build Date | TBD |
| Git Tag | v1.0.0 |
| Git SHA | TBD |
| Binary Size | TBD |
| Binary Checksum (SHA-256) | TBD |

### 2.2 Payload Software (Jetson)

| Item | Value |
|------|-------|
| Name | SMART-QSO Payload Software |
| Version | 1.0.0 |
| Build Date | TBD |
| Git Tag | v1.0.0 |
| Git SHA | TBD |

### 2.3 Build Environment

| Component | Version |
|-----------|---------|
| Compiler (OBC) | arm-none-eabi-gcc 12.2.0 |
| Compiler (Jetson) | Python 3.10+ |
| CMake | 3.26+ |
| Host OS | Ubuntu 22.04 LTS |
| FreeRTOS | 10.5.1 |
| CMSIS | 5.9.0 |

## 3. Delivery Contents

### 3.1 Flight Software Package

```
smart-qso-fsw-1.0.0/
├── bin/
│   ├── smart_qso_flight.elf     # Flight binary
│   ├── smart_qso_flight.bin     # Raw binary for flash
│   ├── smart_qso_flight.hex     # Intel HEX format
│   └── checksums.sha256         # Binary checksums
├── symbols/
│   ├── smart_qso_flight.map     # Linker map file
│   └── smart_qso_flight.sym     # Symbol table
├── source/
│   └── smart-qso-fsw-1.0.0.tar.gz  # Source archive
├── docs/
│   ├── RELEASE_NOTES.md
│   ├── KNOWN_ISSUES.md
│   └── BUILD_INSTRUCTIONS.md
└── config/
    └── sensors.yaml             # Default configuration
```

### 3.2 Payload Software Package

```
smart-qso-payload-1.0.0/
├── software/
│   └── payload/                 # Python package
├── models/
│   └── README.md               # Model placement instructions
├── config/
│   └── payload.yaml            # Configuration
└── docs/
    ├── RELEASE_NOTES.md
    └── DEPLOYMENT.md
```

## 4. Release Notes

### 4.1 Version 1.0.0 - Initial Release

**Release Date**: TBD

**New Features**:
- Complete flight software implementation
- Power-aware state machine (SAFE, IDLE, ACTIVE)
- Telemetry collection and formatting
- AX.25 beacon generation with AI-assisted content
- Watchdog and fault management
- Jetson payload interface
- Graceful degradation without AI payload

**Known Issues**: See Section 6

**Test Summary**:
- Unit tests: 22 test suites, 300+ test cases
- Integration tests: 12 test procedures
- Coverage: >80% line coverage
- Static analysis: MISRA C:2012 compliant

## 5. Build Instructions

### 5.1 Prerequisites

```bash
# Install ARM toolchain
sudo apt install gcc-arm-none-eabi

# Install CMake
sudo apt install cmake

# Clone repository
git clone https://github.com/cheesejaguar/SMART-QSO.git
cd SMART-QSO
git checkout v1.0.0
```

### 5.2 Build Procedure

```bash
# Configure
cmake -S software/flight -B build \
    -DCMAKE_BUILD_TYPE=Release \
    -DFLIGHT_BUILD=ON

# Build
cmake --build build

# Verify checksum
sha256sum build/smart_qso_flight.bin
```

### 5.3 Reproducible Build Verification

```bash
# Build twice and compare
cmake --build build --target clean
cmake --build build
sha256sum build/smart_qso_flight.bin > build1.sha

cmake --build build --target clean
cmake --build build
sha256sum build/smart_qso_flight.bin > build2.sha

diff build1.sha build2.sha  # Should show no difference
```

## 6. Known Issues

### 6.1 Open Issues

| ID | Severity | Description | Workaround |
|----|----------|-------------|------------|
| - | - | No known issues at release | - |

### 6.2 Accepted Deviations

| ID | Requirement | Deviation | Rationale | Approval |
|----|-------------|-----------|-----------|----------|
| - | - | No deviations | - | - |

## 7. Memory Map

### 7.1 Flash Layout

| Region | Start | End | Size | Contents |
|--------|-------|-----|------|----------|
| Bootloader | 0x08000000 | 0x08007FFF | 32 KB | Bootloader |
| App Header | 0x08008000 | 0x080083FF | 1 KB | Version info |
| Application | 0x08008400 | 0x0807FFFF | 478 KB | Flight software |
| Config | 0x08080000 | 0x0808FFFF | 64 KB | Configuration |
| Reserved | 0x08090000 | 0x080FFFFF | 448 KB | Future use |

### 7.2 RAM Layout

| Region | Start | End | Size | Contents |
|--------|-------|-----|------|----------|
| Stack | 0x20000000 | 0x20003FFF | 16 KB | Main stack |
| Heap | 0x20004000 | 0x2000BFFF | 32 KB | Dynamic allocation |
| BSS | 0x2000C000 | 0x2001FFFF | 80 KB | Static data |
| Telemetry | 0x20020000 | 0x20023FFF | 16 KB | Telemetry buffers |
| Reserved | 0x20024000 | 0x2003FFFF | 112 KB | Future use |

## 8. Resource Utilization

### 8.1 Memory Usage

| Resource | Allocated | Used | Margin |
|----------|-----------|------|--------|
| Flash (code) | 478 KB | TBD | TBD |
| Flash (config) | 64 KB | TBD | TBD |
| RAM (static) | 128 KB | TBD | TBD |
| RAM (heap) | 32 KB | TBD | TBD |
| RAM (stack) | 16 KB | TBD | TBD |

### 8.2 CPU Utilization

| Task | Priority | Period | WCET | Utilization |
|------|----------|--------|------|-------------|
| Watchdog | Highest | 1s | <1ms | <0.1% |
| Telemetry | High | 100ms | TBD | TBD |
| Beacon | Medium | 60s | TBD | TBD |
| Health | Low | 10s | TBD | TBD |
| Idle | Lowest | - | - | Remainder |

## 9. Upload Procedure

### 9.1 Ground Upload

```
1. PREREQUISITE: Spacecraft in SAFE mode
2. PREREQUISITE: Good command link
3. Verify current software version
4. Upload binary in 256-byte blocks
5. Verify upload checksum
6. Command software bank switch
7. Verify new version in telemetry
8. Monitor for 3 orbits
```

### 9.2 Rollback Procedure

```
1. Command SAFE mode
2. Verify SAFE mode in telemetry
3. Command software bank switch to previous
4. Verify previous version in telemetry
5. If rollback fails:
   a. Retry bank switch (max 3 attempts)
   b. If persistent, maintain SAFE mode
   c. Escalate to anomaly team
```

## 10. Delivery Verification

### 10.1 Delivery Checklist

| Item | Verified | Date | By |
|------|----------|------|-----|
| Source code tagged | [ ] | | |
| Binary built | [ ] | | |
| Checksum recorded | [ ] | | |
| Symbols archived | [ ] | | |
| Documentation complete | [ ] | | |
| Known issues documented | [ ] | | |
| Release notes complete | [ ] | | |
| Build reproducible | [ ] | | |

### 10.2 Acceptance Signatures

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Software Lead | | | |
| Quality Assurance | | | |
| Systems Engineer | | | |
| Project Manager | | | |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
