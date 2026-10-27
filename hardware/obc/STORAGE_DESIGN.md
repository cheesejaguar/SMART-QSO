# Storage Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-OBC-002
**Version:** 1.7
**Date:** 2026-10-27
**Status:** Draft (pins of §3.4 updated to the build-to design SQSO-OBC-BRD-001)

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
0x00300 - 0x003FF   | 256 B   | RF control: TX-OFF latch, TX power level (§6.5)
0x00400 - 0x043FF   | 16 KB   | Mission data (A)
0x04400 - 0x083FF   | 16 KB   | Mission data (B)
0x08400 - 0x0C3FF   | 16 KB   | Mission data (C)
0x0C400 - 0x0C6FF   | 768 B   | Telecommand key table (§6.5)
0x0C700 - 0x0C7FF   | 256 B   | Loss-of-contact counter and timeout (§6.5)
0x0C800 - 0x0C87F   | 128 B   | Post-deployment RF-silence state (§6.5)
0x0C880 - 0x0C8FF   | 128 B   | Deployment sequence state (§6.5)
0x0C900 - 0x0FFFF   | 13.75 KB| Event log (reduced from 15 KB for the key table, contact, silence and deployment records)
0x10000 - 0x1FFFF   | 64 KB   | Telemetry buffer: sixty-second housekeeping records, ring and three event windows (HK_DETAIL, §3.3.1)
0x20000 - 0x201FF   | 512 B   | Sensor configuration
0x20200 - 0x205FF   | 1 KB    | Backup
0x20600 - 0x2067F   | 128 B   | Operations record: commissioning hold, transponder enable (OPS)
0x20700 - 0x366FF   | 88 KB   | Five-minute housekeeping records, 2,048 × 44 B (HK_SUMMARY, §3.3.1)
0x36700 - 0x367FF   | 256 B   | Housekeeping event-window table (HK_INDEX)
0x36800 - 0x3687F   | 128 B   | End-of-life passivation record (EOL)
0x36880 - 0x3FFFF   | 37.9 KB | Unallocated
```

#### 3.3.1 Stored housekeeping (SRS-F120 to F122)

The regions are those of `software/rust/firmware/src/fram.rs` (its compile-time check rejects overlaps). Each record is 44
bytes with a CRC. HK_SUMMARY holds one record every 5 minutes, 2,048 kept (7.1 days); HK_DETAIL one every 60 s, 409
kept (6.8 h), plus three event windows of 3 h either side of a fault or mode change. DUMP_TLM (0x0A) downlinks at most
600 records, 4 per frame, one frame per 12 s.

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
| SCK | PG2 (SPI1 AF5) | SPI clock, 20 MHz (33 Ω series) |
| MOSI | PG4 (SPI1 AF5) | Master out |
| MISO | PG3 (SPI1 AF5) | Master in |
| CS# | PG5 | Chip select (active low), 10 kΩ pull-up |
| WP# | PE0 | Write protect (held high), 10 kΩ pull-up |
| HOLD# | PE1 | Hold (held high), 10 kΩ pull-up |

Pins as allocated by SQSO-ICX-001 §4 and the board design SQSO-OBC-BRD-001 §7 (1.6). Until 1.5 this table gave PA4–PA7
and PC0/PC1; the firmware uses PA4–PA7 with WP#/HOLD# on PE0/PE1 and moves to PG2–PG5 (SQSO-ICX-001 §6 row 3), which
frees four ADC1 inputs. The MCU is the STM32L4R5ZIT6 (the diagram above names the superseded STM32L476).

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

### 6.5 Fail-Safe Records

The telecommand key table (`FLASH_REGION_AUTH`, 768 B) and RF control (`FLASH_REGION_RF_CONTROL`, 256 B) use the two-slot form of §6.4, implemented in `software/rust/sqso-fsw/src/nvm.rs`. Each record occupies 128 B (two 64 B slots); each slot is magic (4) | sequence (4) | payload (≤ 52) | CRC-32 (4). A save writes the slot that does not hold the newest valid copy and reads it back.

| Region | Offset | Record |
|--------|--------|--------|
| RF control | 0 | TX-OFF latch |
| RF control | 128 | TX power level |
| Contact | 0 | Seconds without contact (written every 60 s) |
| Contact | 128 | Loss-of-contact timeout (days) |
| RF silence | 0 | State (armed, counting, complete) and seconds counted (written every 60 s while counting) |
| Deployment | 0 | State, attempts per element, deployed/enabled/separated flags, wait seconds (written on every transition, before every burn, and every 60 s while waiting) |
| Key table | 0 | State: active operational slot and its counter |
| Key table | 128 | Recovery key, ID and counter |
| Key table | 256 + 128 × n | Operational key n (0–3): key, ID, spare/active/revoked |

Unlike the data in §6.3, these records do **not** fall back to defaults when corrupt:

| Record | Blank (never written) | Corrupt |
|--------|-----------------------|---------|
| TX-OFF latch | Transmit permitted | Transmit inhibited |
| TX power level | 0.5 W | 0.5 W |
| Seconds without contact | 0 (first boot) | Timed out: transmitters inhibited until contact |
| Contact timeout | Default (placeholder 14 days) | Default |
| RF silence | Count from 0 (silent 45 min) | Count from 0 (silent 45 min) |
| Deployment | Start over (30-min wait) | Start over with zero attempts, fault logged; sense lines skip deployed elements |
| Key-table state or active key | No operational key: operational commands rejected | Same; fault logged; recover with KEY_ROTATE (recovery key) |
| Recovery key | No recovery key | Same; fault logged |
| No usable key at all | Only TX_DISABLE accepted (unauthenticated) | Same |

The interface is defined in SQSO-ICD-013 (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`). Only the simulation HAL implements these regions today. The FRAM driver must map them to the addresses in §3.3.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-15 | Allocated the telecommand key and TX-OFF latch records (§3.3, §6.5) |
| 1.2 | 2026-10-15 | Key table (768 B, carved from the event log) and TX power-level record (§3.3, §6.5) |
| 1.3 | 2026-10-15 | Loss-of-contact records (256 B, carved from the event log) |
| 1.4 | 2026-10-15 | RF-silence record (128 B, carved from the event log) |
| 1.5 | 2026-10-15 | Deployment record (128 B, carved from the event log) |
| 1.6 | 2026-10-27 | Header version aligned with the history (was 1.0 after 1.5). §3.4 FRAM pins from the master pin allocation SQSO-ICX-001 (SPI1 on PG2–PG5, WP# PE0, HOLD# PE1) |
| 1.7 | 2026-10-27 | §3.3 extended storage allocated as implemented (`software/rust/firmware/src/fram.rs`): sensor configuration, backup, operations record, five-minute and sixty-second housekeeping, event-window table, passivation record; §3.3.1 stored housekeeping |
