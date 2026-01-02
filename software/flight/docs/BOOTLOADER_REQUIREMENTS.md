# Bootloader Requirements
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-BOOT-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NASA-HDBK-4002A

---

## 1. Purpose

This document defines the requirements and design for the SMART-QSO bootloader, ensuring reliable boot and recovery capabilities in the space environment.

## 2. Bootloader Overview

### 2.1 Boot Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    FLASH MEMORY MAP                          │
├─────────────────────────────────────────────────────────────┤
│ 0x0000_0000 │ Boot ROM (Protected)      │  4 KB            │
├─────────────┼───────────────────────────┼──────────────────┤
│ 0x0000_1000 │ Primary Bootloader        │ 16 KB            │
├─────────────┼───────────────────────────┼──────────────────┤
│ 0x0000_5000 │ Backup Bootloader         │ 16 KB            │
├─────────────┼───────────────────────────┼──────────────────┤
│ 0x0000_9000 │ Boot Configuration        │  4 KB            │
├─────────────┼───────────────────────────┼──────────────────┤
│ 0x0000_A000 │ Primary Application       │ 448 KB           │
├─────────────┼───────────────────────────┼──────────────────┤
│ 0x0007_A000 │ Backup Application        │ 448 KB           │
├─────────────┼───────────────────────────┼──────────────────┤
│ 0x000E_A000 │ Persistent Data           │  88 KB           │
└─────────────┴───────────────────────────┴──────────────────┘
```

### 2.2 Boot Sequence

```
┌─────────────┐
│  Power On   │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Boot ROM       │  Hardware initialization
│  (Protected)    │  Basic hardware test
└────────┬────────┘
         │
         ▼
┌─────────────────┐     CRC Fail
│  Verify Primary │─────────────┐
│  Bootloader     │             │
└────────┬────────┘             │
         │ CRC Pass             ▼
         │              ┌───────────────┐
         ▼              │ Load Backup   │
┌─────────────────┐     │ Bootloader    │
│  Primary        │     └───────┬───────┘
│  Bootloader     │             │
└────────┬────────┘◄────────────┘
         │
         ▼
┌─────────────────┐     CRC Fail
│  Verify Primary │─────────────┐
│  Application    │             │
└────────┬────────┘             │
         │ CRC Pass             ▼
         │              ┌───────────────┐
         ▼              │ Load Backup   │
┌─────────────────┐     │ Application   │
│  Primary        │     └───────┬───────┘
│  Application    │             │
└────────┬────────┘◄────────────┘
         │
         ▼
┌─────────────────┐
│  Flight         │
│  Software       │
└─────────────────┘
```

## 3. Functional Requirements

### 3.1 Boot ROM Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| BOOT-001 | Boot ROM shall be hardware write-protected | Inspection |
| BOOT-002 | Boot ROM shall initialize critical hardware within 100ms | Test |
| BOOT-003 | Boot ROM shall verify bootloader CRC before execution | Test |
| BOOT-004 | Boot ROM shall support fallback to backup bootloader | Test |
| BOOT-005 | Boot ROM size shall not exceed 4 KB | Inspection |

### 3.2 Bootloader Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| BOOT-010 | Bootloader shall verify application CRC32 | Test |
| BOOT-011 | Bootloader shall support primary/backup application | Test |
| BOOT-012 | Bootloader shall complete within 5 seconds | Test |
| BOOT-013 | Bootloader shall log boot events to persistent storage | Test |
| BOOT-014 | Bootloader shall detect and recover from boot loops | Test |
| BOOT-015 | Bootloader shall support firmware update via UART | Test |
| BOOT-016 | Bootloader shall validate update images before write | Test |
| BOOT-017 | Bootloader shall preserve persistent data during update | Test |
| BOOT-018 | Bootloader size shall not exceed 16 KB | Inspection |

### 3.3 Safety Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| BOOT-020 | Bootloader shall never erase both application images | Analysis, Test |
| BOOT-021 | Bootloader shall timeout and boot safe image if update fails | Test |
| BOOT-022 | Bootloader shall implement boot attempt counter | Test |
| BOOT-023 | Bootloader shall enter safe mode after 5 consecutive boot failures | Test |

## 4. Boot Configuration

### 4.1 Boot Configuration Structure

```c
/**
 * Boot configuration stored in protected flash
 */
typedef struct __attribute__((packed)) {
    uint32_t magic;                 // 0x424F4F54 ("BOOT")
    uint32_t version;               // Configuration version
    uint8_t  boot_source;           // 0=Primary, 1=Backup
    uint8_t  boot_attempts;         // Consecutive boot attempts
    uint8_t  last_boot_status;      // Success/failure code
    uint8_t  reserved;
    uint32_t primary_app_crc;       // Expected CRC of primary
    uint32_t backup_app_crc;        // Expected CRC of backup
    uint32_t last_boot_time;        // Unix timestamp
    uint32_t total_boots;           // Lifetime boot counter
    uint32_t config_crc;            // CRC of this structure
} BootConfig_t;

#define BOOT_CONFIG_MAGIC    0x424F4F54
#define BOOT_MAX_ATTEMPTS    5
```

### 4.2 Boot Status Codes

| Code | Name | Description |
|------|------|-------------|
| 0x00 | BOOT_SUCCESS | Normal boot completed |
| 0x01 | BOOT_CRC_FAIL_PRIMARY | Primary app CRC failed |
| 0x02 | BOOT_CRC_FAIL_BACKUP | Backup app CRC failed |
| 0x03 | BOOT_TIMEOUT | Boot sequence timeout |
| 0x04 | BOOT_HARDWARE_FAIL | Hardware initialization failed |
| 0x05 | BOOT_SAFE_MODE | Entered safe mode |
| 0xFF | BOOT_IN_PROGRESS | Boot not yet complete |

## 5. Firmware Update Protocol

### 5.1 Update Sequence

```
Ground Station                    Spacecraft
      │                               │
      │──── UPLOAD_START ────────────►│
      │◄─── UPLOAD_ACK ──────────────│
      │                               │
      │──── DATA_PACKET[0] ──────────►│
      │◄─── DATA_ACK[0] ─────────────│
      │           ...                 │
      │──── DATA_PACKET[N] ──────────►│
      │◄─── DATA_ACK[N] ─────────────│
      │                               │
      │──── UPLOAD_COMPLETE ─────────►│
      │◄─── CRC_VERIFY ──────────────│
      │                               │
      │──── COMMIT_UPDATE ───────────►│
      │◄─── COMMIT_ACK ──────────────│
      │                               │
      │──── REBOOT_CMD ──────────────►│
      │                    (reboot)   │
      │◄─── BOOT_COMPLETE ───────────│
      │                               │
```

### 5.2 Update Commands

| Command | Code | Parameters | Response |
|---------|------|------------|----------|
| UPLOAD_START | 0x01 | image_size, target_slot | ACK/NAK |
| DATA_PACKET | 0x02 | seq_num, data[256] | ACK/NAK |
| UPLOAD_COMPLETE | 0x03 | total_packets | CRC result |
| COMMIT_UPDATE | 0x04 | slot_id | ACK/NAK |
| ABORT_UPDATE | 0x05 | none | ACK |
| QUERY_STATUS | 0x06 | none | Status |

### 5.3 Update Safety

```c
/**
 * Firmware update validation checks
 */
typedef struct {
    bool size_valid;        // Image size within limits
    bool crc_valid;         // CRC32 matches header
    bool signature_valid;   // Optional crypto signature
    bool header_valid;      // Valid image header
    bool target_valid;      // Valid target slot
} UpdateValidation_t;

/**
 * @brief Validate firmware image before commit
 * All checks must pass before writing to flash
 */
SmartQsoResult_t validate_firmware_image(
    const uint8_t *image,
    uint32_t size,
    UpdateValidation_t *result
);
```

## 6. Boot Timing Requirements

### 6.1 Timing Budget

| Phase | Maximum Time | Notes |
|-------|--------------|-------|
| Boot ROM | 100 ms | Hardware init |
| Bootloader load | 50 ms | From flash |
| Bootloader exec | 500 ms | Verify app |
| Application load | 200 ms | From flash |
| Application init | 2000 ms | Full init |
| **Total Boot** | **< 3000 ms** | **Nominal** |

### 6.2 Watchdog During Boot

| Phase | Watchdog Timeout |
|-------|------------------|
| Boot ROM | Disabled |
| Bootloader | 10 seconds |
| Application | 30 seconds (after init) |

## 7. Recovery Mechanisms

### 7.1 Boot Loop Detection

```c
/**
 * @brief Check for boot loop condition
 * @return true if boot loop detected
 */
bool is_boot_loop(void) {
    BootConfig_t config;
    read_boot_config(&config);

    // Boot loop if 5+ consecutive failed boots
    if (config.boot_attempts >= BOOT_MAX_ATTEMPTS) {
        if (config.last_boot_status != BOOT_SUCCESS) {
            return true;
        }
    }
    return false;
}

/**
 * @brief Handle boot loop recovery
 */
void handle_boot_loop(void) {
    // Try backup image if not already
    if (get_boot_source() == BOOT_SOURCE_PRIMARY) {
        set_boot_source(BOOT_SOURCE_BACKUP);
        reset_boot_attempts();
        trigger_reboot();
    } else {
        // Both images failed - enter minimal safe mode
        enter_bootloader_safe_mode();
    }
}
```

### 7.2 Golden Image

| Property | Value |
|----------|-------|
| Location | Backup slot |
| Protection | Write-protected after validation |
| Version | Last known good |
| Updates | Only via ground command with confirmation |

## 8. Testing Requirements

### 8.1 Boot Test Matrix

| Test | Primary OK | Backup OK | Expected Result |
|------|------------|-----------|-----------------|
| TC-001 | Yes | Yes | Boot primary |
| TC-002 | No | Yes | Boot backup |
| TC-003 | Yes | No | Boot primary |
| TC-004 | No | No | Safe mode |
| TC-005 | Corrupt | Yes | Detect, boot backup |
| TC-006 | Yes | Corrupt | Boot primary |

### 8.2 Update Test Matrix

| Test | Condition | Expected Result |
|------|-----------|-----------------|
| TU-001 | Valid image | Successful update |
| TU-002 | CRC mismatch | Reject, keep original |
| TU-003 | Incomplete transfer | Abort, keep original |
| TU-004 | Power loss during write | Recover to backup |
| TU-005 | Invalid header | Reject upload |

## 9. Bootloader Safe Mode

### 9.1 Safe Mode Capabilities

| Feature | Status |
|---------|--------|
| UART communication | Enabled |
| Firmware upload | Enabled |
| Status query | Enabled |
| Application boot | Disabled |
| Peripheral init | Minimal |

### 9.2 Safe Mode Entry

```c
/**
 * Bootloader safe mode - minimal operation for recovery
 */
void enter_bootloader_safe_mode(void) {
    // Initialize minimal hardware
    init_uart_only();
    init_watchdog(SAFE_MODE_WDT_TIMEOUT);

    log_boot_event(BOOT_SAFE_MODE);

    // Wait for ground commands
    while (1) {
        watchdog_kick();
        process_uart_commands();
        sleep_ms(100);
    }
}
```

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
