# Storage Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-OBC-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the data storage design for the SMART-QSO OBC subsystem, including primary non-volatile storage, secondary storage, and data integrity mechanisms.

---

## 2. Storage Requirements

### 2.1 Functional Requirements

| Requirement | Value | Notes |
|-------------|-------|-------|
| Boot configuration | 4 KB | Persistent across resets |
| Mission data | 16 KB | Counters, state, logs |
| Telemetry buffer | 64 KB | Circular buffer |
| Science data | 512 KB | If applicable |
| Total minimum | 600 KB | Primary storage |
| Extended storage | 2 MB | For telemetry archive |

### 2.2 Performance Requirements

| Requirement | Value |
|-------------|-------|
| Write endurance | > 10^6 cycles |
| Read speed | > 1 MB/s |
| Write speed | > 100 KB/s |
| Data retention | > 10 years |
| Operating temp | -40°C to +85°C |

### 2.3 Reliability Requirements

| Requirement | Implementation |
|-------------|----------------|
| Bit error rate | < 10^-9 |
| Power failure safe | Yes (atomic writes) |
| Radiation tolerance | SEU mitigation |
| Redundancy | Triple-redundant critical data |

---

## 3. Primary Storage (FRAM/MRAM)

### 3.1 Technology Selection

**Selected: FRAM (Ferroelectric RAM)**

| Parameter | FRAM | MRAM | Flash |
|-----------|------|------|-------|
| Write endurance | 10^14 | 10^12 | 10^5 |
| Write speed | Fast | Fast | Slow |
| Byte-addressable | Yes | Yes | No (page) |
| Power failure safe | Yes | Yes | Risky |
| Radiation tolerance | Good | Good | Poor |
| Cost | Medium | High | Low |

### 3.2 Selected Component

| Parameter | Specification |
|-----------|---------------|
| Part number | FM25V20A (Infineon) or equivalent |
| Capacity | 256 KB (2 Mbit) |
| Interface | SPI (up to 40 MHz) |
| Voltage | 2.0V - 3.6V |
| Temperature | -40°C to +85°C |
| Package | SOIC-8 |

### 3.3 FRAM Memory Map

```
Address Range       | Size    | Content
--------------------|---------|------------------
0x00000 - 0x000FF   | 256 B   | Boot configuration (A)
0x00100 - 0x001FF   | 256 B   | Boot configuration (B)
0x00200 - 0x002FF   | 256 B   | Boot configuration (C)
0x00300 - 0x003FF   | 256 B   | Reserved
0x00400 - 0x043FF   | 16 KB   | Mission data (A)
0x04400 - 0x083FF   | 16 KB   | Mission data (B)
0x08400 - 0x0C3FF   | 16 KB   | Mission data (C)
0x0C400 - 0x0FFFF   | 15 KB   | Event log
0x10000 - 0x1FFFF   | 64 KB   | Telemetry buffer
0x20000 - 0x3FFFF   | 128 KB  | Extended storage
```

### 3.4 Interface Design

```
MCU (STM32L476)                    FRAM (FM25V20A)
     SPI1_SCK  ─────────────────── SCK
     SPI1_MOSI ─────────────────── SI
     SPI1_MISO ─────────────────── SO
     GPIO_CS   ─────────────────── CS#
     GPIO_WP   ─────────────────── WP#
     GPIO_HOLD ─────────────────── HOLD#
```

| Signal | MCU Pin | Notes |
|--------|---------|-------|
| SCK | PA5 | SPI clock (20 MHz max) |
| MOSI | PA7 | Master out |
| MISO | PA6 | Master in |
| CS# | PA4 | Chip select (active low) |
| WP# | PC0 | Write protect (held high) |
| HOLD# | PC1 | Hold (held high) |

---

## 4. Secondary Storage (SD Card)

### 4.1 Use Cases

| Use Case | Description |
|----------|-------------|
| Telemetry archive | Extended telemetry history |
| Science data | Larger data sets |
| Debug logs | Development/testing |
| Firmware backup | Recovery image |

### 4.2 SD Card Specification

| Parameter | Specification |
|-----------|---------------|
| Type | microSD or SD |
| Capacity | 2 GB - 32 GB |
| Interface | SPI mode |
| Speed | Up to 25 MHz |
| File system | FAT32 |

### 4.3 Interface Design

```
MCU (STM32L476)                    SD Card
     SPI2_SCK  ─────────────────── CLK
     SPI2_MOSI ─────────────────── CMD/DI
     SPI2_MISO ─────────────────── DAT0/DO
     GPIO_CS   ─────────────────── DAT3/CS
     GPIO_CD   ─────────────────── Card Detect
```

| Signal | MCU Pin | Notes |
|--------|---------|-------|
| CLK | PB13 | SPI clock |
| MOSI | PB15 | Command/Data in |
| MISO | PB14 | Data out |
| CS | PB12 | Chip select |
| CD | PB11 | Card detect |

### 4.4 SD Card Power Management

| Parameter | Value |
|-----------|-------|
| Operating current | 30-100 mA |
| Idle current | 0.2-1 mA |
| Power control | GPIO-controlled regulator |
| Power-on delay | 10 ms minimum |

---

## 5. Wear Leveling

### 5.1 Strategy

For FRAM (high endurance): No wear leveling required (10^14 cycles)

For SD Card (FAT32):
- Rely on SD card controller wear leveling
- Use sequential write patterns
- Avoid frequent small writes

### 5.2 Write Patterns

| Data Type | Pattern | Frequency |
|-----------|---------|-----------|
| Mission data | Redundant, scheduled | Every 60 s |
| Telemetry | Circular buffer | Every 10 s |
| Event log | Append-only | On event |
| SD archive | Sequential | Every 10 min |

---

## 6. Data Integrity Features

### 6.1 CRC Protection

| Data Area | CRC Type | Coverage |
|-----------|----------|----------|
| Boot config | CRC-32 | Entire block |
| Mission data | CRC-32 | Entire block |
| Telemetry packets | CRC-16 | Each packet |
| Event log entries | CRC-16 | Each entry |

### 6.2 CRC-32 Implementation

```c
// CRC-32 polynomial (IEEE 802.3)
#define CRC32_POLYNOMIAL 0xEDB88320

uint32_t crc32_calculate(const uint8_t *data, size_t length) {
    uint32_t crc = 0xFFFFFFFF;
    for (size_t i = 0; i < length; i++) {
        crc ^= data[i];
        for (int j = 0; j < 8; j++) {
            crc = (crc >> 1) ^ ((crc & 1) ? CRC32_POLYNOMIAL : 0);
        }
    }
    return ~crc;
}
```

### 6.3 Triple Redundancy

Critical data (boot config, mission data) stored in triplicate:

```
Write sequence:
1. Calculate CRC for new data
2. Write to location A
3. Write to location B
4. Write to location C

Read sequence:
1. Read all three copies
2. Verify CRC of each
3. Vote on valid copies
4. If 2+ valid and agree: use data
5. If 1 valid: use and repair others
6. If 0 valid: use defaults, set fault flag
```

### 6.4 Atomic Writes

For power-fail safety:

```c
typedef struct {
    uint32_t magic;      // 0xDEADBEEF when valid
    uint32_t sequence;   // Incrementing counter
    uint8_t  data[248];  // Actual data
    uint32_t crc;        // CRC of magic+sequence+data
} atomic_block_t;

// Write uses two alternating blocks
// Read selects block with higher valid sequence
```

---

## 7. File System Selection

### 7.1 Primary Storage (FRAM)

**No file system** - direct memory-mapped access

Rationale:
- Fixed data structures
- Known sizes and locations
- Maximum efficiency
- Deterministic access time

### 7.2 Secondary Storage (SD Card)

**FAT32 file system**

| Feature | Implementation |
|---------|----------------|
| Library | FatFs (ChaN) or similar |
| Long filenames | Disabled (8.3 only) |
| Subdirectories | Limited use |
| File size limit | 4 GB |

Directory structure:
```
/
├── TELEM/          # Telemetry archives
│   ├── TM_0001.CSV
│   ├── TM_0002.CSV
│   └── ...
├── LOG/            # Event logs
│   └── EVENTS.LOG
├── CONFIG/         # Configuration backups
│   └── BACKUP.BIN
└── FW/             # Firmware images
    └── OBC_FW.BIN
```

---

## 8. Error Handling

### 8.1 FRAM Error Handling

| Error | Detection | Response |
|-------|-----------|----------|
| SPI timeout | Watchdog | Retry, then fault |
| CRC mismatch | Read verify | Use redundant copy |
| All copies bad | Triple vote | Use defaults + fault |
| Write failure | Read-back verify | Retry, then fault |

### 8.2 SD Card Error Handling

| Error | Detection | Response |
|-------|-----------|----------|
| Card not present | CD pin | Disable SD features |
| Initialization fail | Timeout | Retry 3x, then disable |
| Write error | Return code | Retry, then disable |
| File system error | FatFs return | Re-mount, then disable |
| Card full | Free space check | Delete oldest files |

### 8.3 Fallback Behavior

| Condition | Fallback |
|-----------|----------|
| FRAM failure | Safe mode, minimal operation |
| SD card failure | Continue without archiving |
| Both failed | Safe mode, beacon only |

---

## 9. Power Considerations

### 9.1 Power Budget

| Component | Active (mW) | Standby (mW) |
|-----------|-------------|--------------|
| FRAM | 15 | 0.01 |
| SD Card | 150 | 1 |
| Total | 165 | 1 |

### 9.2 Power Optimization

| Strategy | Implementation |
|----------|----------------|
| SD card power gating | GPIO-controlled regulator |
| SD card duty cycling | Power on only when writing |
| FRAM always on | Low standby power |

---

## 10. Testing Requirements

### 10.1 Unit Tests

| Test | Description |
|------|-------------|
| FRAM read/write | All addresses |
| CRC calculation | Known vectors |
| Triple redundancy | Inject single/double errors |
| Atomic write | Power interrupt simulation |
| SD card init | Card present/absent |
| File operations | Create, write, read, delete |

### 10.2 Environmental Tests

| Test | Condition |
|------|-----------|
| Temperature | -40°C to +85°C operation |
| Radiation (SEU) | If applicable |
| Endurance | 10^6 write cycles |
| Power cycling | 10,000 cycles |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
