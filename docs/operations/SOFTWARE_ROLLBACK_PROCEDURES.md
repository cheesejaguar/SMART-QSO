# SMART-QSO Software Rollback and Recovery Procedures

**Document ID**: SQSO-SRP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines procedures for software rollback and recovery operations for the SMART-QSO mission. These procedures ensure safe return to known-good software states following anomalies.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| SQSO-MOP-001 | Mission Operations Plan |
| SQSO-CP-001 | Contingency Procedures |
| SQSO-SVD-001 | Software Version Description |
| SQSO-FSW-DEL-001 | Flight Software Delivery Package |

## 3. Software Architecture for Rollback

### 3.1 Memory Bank Organization

The flight software uses a dual-bank architecture for safe updates:

```
┌─────────────────────────────────────────────────┐
│                   FLASH MEMORY                   │
├─────────────────────────────────────────────────┤
│  0x08000000  ┌─────────────────────────────┐    │
│              │       BOOTLOADER            │    │
│              │         (32 KB)             │    │
│  0x08008000  ├─────────────────────────────┤    │
│              │    BANK A (PRIMARY)         │    │
│              │      Application            │    │
│              │       (240 KB)              │    │
│  0x08040000  ├─────────────────────────────┤    │
│              │    BANK B (BACKUP)          │    │
│              │      Application            │    │
│              │       (240 KB)              │    │
│  0x08078000  ├─────────────────────────────┤    │
│              │    BANK SELECT FLAGS        │    │
│              │         (4 KB)              │    │
│  0x08079000  ├─────────────────────────────┤    │
│              │    CONFIGURATION            │    │
│              │        (28 KB)              │    │
│  0x08080000  └─────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

### 3.2 Bank Select Mechanism

| Flag Address | Value | Boot Behavior |
|--------------|-------|---------------|
| 0x08078000 | 0xA5A5 | Boot from Bank A |
| 0x08078000 | 0x5A5A | Boot from Bank B |
| 0x08078000 | 0xFFFF | Boot from Bank A (default) |

### 3.3 Boot Sequence

```
BOOTLOADER SEQUENCE:

1. Hardware initialization
2. Read bank select flag
3. Validate selected bank:
   a. Check header magic (0xDEADBEEF)
   b. Verify CRC-32
   c. Check version compatibility
4. IF validation passes:
   - Jump to selected bank
5. IF validation fails:
   - Try alternate bank
   - IF alternate valid: Jump to alternate
   - IF alternate invalid: Enter bootloader safe mode
```

## 4. Rollback Triggers

### 4.1 Automatic Rollback Triggers

| Trigger | Detection | Action |
|---------|-----------|--------|
| Boot failure (3x) | Boot counter | Switch to backup bank |
| CRC mismatch | Boot validation | Switch to backup bank |
| Watchdog cascade (5x/hour) | Fault counter | Enter safe mode |

### 4.2 Ground-Commanded Rollback Triggers

| Condition | Decision Authority | Procedure |
|-----------|-------------------|-----------|
| Software anomaly | Mission Director | SRP-001 |
| Performance degradation | Flight Controller + MD | SRP-001 |
| Feature regression | Software Lead + MD | SRP-001 |

## 5. Rollback Procedures

---

### SRP-001: Ground-Commanded Bank Switch

**Purpose**: Switch to backup software bank via ground command

**Prerequisites**:
- [ ] Two certified operators on console
- [ ] Mission Director approval
- [ ] Full pass available (>10 min)
- [ ] Telemetry link confirmed
- [ ] Backup bank contains known-good version

**Procedure**:

| Step | Action | Expected Result | Verify |
|------|--------|-----------------|--------|
| 1 | Verify spacecraft state | STATE = IDLE or SAFE | [ ] |
| 2 | Record current version | VERSION = x.x.x | [ ] |
| 3 | Record current bank | BANK = A or B | [ ] |
| 4 | Command SAFE mode | STATE → SAFE | [ ] |
| 5 | Verify SAFE mode | Telemetry confirms | [ ] |
| 6 | Send BANK_SWITCH command | ACK received | [ ] |
| 7 | Wait for reboot | LOS expected ~5s | [ ] |
| 8 | Wait for beacon | Beacon within 60s | [ ] |
| 9 | Verify new version | VERSION = y.y.y | [ ] |
| 10 | Verify new bank | BANK = alternate | [ ] |
| 11 | Monitor for 3 passes | Stable operation | [ ] |
| 12 | If stable, resume ops | STATE → IDLE | [ ] |

**Contingency**:
```
IF no beacon after 120 seconds:
  - Wait for next pass
  - Attempt contact
  - If still no contact, see CP-001 (LOS)

IF wrong version reported:
  - DO NOT command further
  - Analyze telemetry
  - Convene anomaly team

IF repeated resets after switch:
  - System should auto-rollback
  - Monitor for stable state
  - Document for analysis
```

**Documentation**:
- [ ] Log all commands and responses
- [ ] Record timeline of events
- [ ] File anomaly report if triggered by issue
- [ ] Update software configuration log

---

### SRP-002: Automatic Rollback Recovery

**Purpose**: Recover from automatic rollback event

**Detection**:
- Boot reason = AUTO_ROLLBACK
- Version changed unexpectedly
- Bank select changed

**Procedure**:

| Step | Action | Expected Result | Verify |
|------|--------|-----------------|--------|
| 1 | Confirm rollback occurred | BOOT_REASON = AUTO_ROLLBACK | [ ] |
| 2 | Record failure bank version | VERSION = x.x.x | [ ] |
| 3 | Record active bank version | VERSION = y.y.y | [ ] |
| 4 | Collect diagnostic telemetry | Fault counters, temps | [ ] |
| 5 | Verify stable operation | No repeated faults | [ ] |
| 6 | Monitor for 3 passes | Nominal behavior | [ ] |
| 7 | File anomaly report | Document event | [ ] |
| 8 | Analyze failure mode | Root cause | [ ] |
| 9 | Determine corrective action | Fix or accept | [ ] |

**Analysis Checklist**:
- [ ] What triggered the rollback?
- [ ] Was the failed software ever functional?
- [ ] Is this a transient or permanent failure?
- [ ] Is the backup version adequate for operations?
- [ ] Is software update needed?

---

### SRP-003: Software Recovery Upload

**Purpose**: Upload corrected software after rollback

**Prerequisites**:
- [ ] Root cause of failure understood
- [ ] Corrected software tested on ground
- [ ] Software review complete
- [ ] Mission Director approval
- [ ] Full contact window (>15 min)

**Procedure**:

| Step | Action | Expected Result | Verify |
|------|--------|-----------------|--------|
| 1 | Prepare upload package | Binary + checksum ready | [ ] |
| 2 | Verify target bank | Failed bank is target | [ ] |
| 3 | Command SAFE mode | STATE → SAFE | [ ] |
| 4 | Initiate upload mode | UPLOAD_MODE active | [ ] |
| 5 | Upload binary (256B blocks) | Progress telemetry | [ ] |
| 6 | Verify upload checksum | CHECKSUM matches | [ ] |
| 7 | Command bank validation | BANK_VALID = OK | [ ] |
| 8 | Decision: Switch or keep backup | Per analysis | [ ] |
| 9 | If switching, see SRP-001 | | [ ] |
| 10 | Document upload | Configuration log | [ ] |

**Upload Protocol**:
```
UPLOAD FRAME FORMAT:
┌────────┬────────┬────────┬────────────────┬────────┐
│ SYNC   │ SEQ#   │ LENGTH │     DATA       │  CRC   │
│ 2 byte │ 2 byte │ 2 byte │   256 bytes    │ 2 byte │
└────────┴────────┴────────┴────────────────┴────────┘

UPLOAD SEQUENCE:
1. UPLOAD_START(bank, total_size)
2. UPLOAD_DATA(seq, data) × N
3. UPLOAD_END(checksum)
4. UPLOAD_VERIFY(bank)

RESPONSE CODES:
- 0x00: OK
- 0x01: Sequence error (resend)
- 0x02: CRC error (resend)
- 0x03: Flash error (abort)
```

---

### SRP-004: Bootloader Safe Mode Recovery

**Purpose**: Recover from bootloader safe mode (both banks invalid)

**Detection**:
- No application beacon (bootloader beacon only)
- Telemetry shows BOOTLOADER_SAFE mode
- Both banks report invalid

**Procedure**:

| Step | Action | Expected Result | Verify |
|------|--------|-----------------|--------|
| 1 | Confirm bootloader mode | BOOT_MODE = SAFE | [ ] |
| 2 | Verify uplink capability | Commands accepted | [ ] |
| 3 | Select target bank | Bank A or B | [ ] |
| 4 | Execute SRP-003 upload | Binary uploaded | [ ] |
| 5 | Verify upload success | CHECKSUM OK | [ ] |
| 6 | Command boot to new bank | BOOT_BANK(x) | [ ] |
| 7 | Wait for application beacon | Within 60s | [ ] |
| 8 | Verify version | VERSION correct | [ ] |
| 9 | Monitor stability | 3 passes nominal | [ ] |

**Recovery Priority**:
1. If one bank recoverable: Upload to that bank
2. If neither recoverable: Upload last known-good to Bank A
3. If upload fails: Maintain bootloader safe mode, retry

---

## 6. Payload Software Recovery

### 6.1 Jetson Recovery

The Jetson payload operates independently and has its own recovery:

| Condition | Recovery |
|-----------|----------|
| Jetson unresponsive | Power cycle via OBC command |
| Python crash | Auto-restart via systemd |
| Model load failure | Fallback to template beacons |

### 6.2 Jetson Power Cycle Procedure

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Command JETSON_POWER_OFF | Power rail disabled |
| 2 | Wait 10 seconds | Discharge complete |
| 3 | Command JETSON_POWER_ON | Power rail enabled |
| 4 | Wait 60 seconds | Boot complete |
| 5 | Verify Jetson status | STATUS = ONLINE |

## 7. Ground Software Recovery

### 7.1 Ground Software Rollback

Ground software uses standard version control:

```bash
# Rollback to previous version
cd SMART-QSO
git checkout v1.0.0-previous

# Or specific commit
git checkout <commit-sha>

# Reinstall dependencies
pip install -r requirements.txt
```

### 7.2 Configuration Recovery

Ground configuration backed up automatically:
- Location: `~/.smart-qso/config.bak`
- Restore: `cp config.bak config.yaml`

## 8. Recovery Test Verification

### 8.1 Pre-Flight Verification

| Test | Procedure | Status |
|------|-----------|--------|
| Bank switch (A→B) | TP-FLATSAT-001 | [ ] |
| Bank switch (B→A) | TP-FLATSAT-001 | [ ] |
| Auto-rollback trigger | TP-FLATSAT-001 | [ ] |
| Upload and verify | TP-FLATSAT-001 | [ ] |
| Bootloader recovery | TP-FLATSAT-001 | [ ] |

### 8.2 On-Orbit Verification

| Test | Timeline | Status |
|------|----------|--------|
| Bank switch demo | L+7 days | [ ] |
| Upload demo (to unused bank) | L+14 days | [ ] |

## 9. Quick Reference

### 9.1 Rollback Decision Tree

```
┌─────────────────────────────┐
│ Software anomaly detected   │
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│ Is spacecraft safe?         │
│ (no cascading failures)     │
└─────────────┬───────────────┘
              │
       ┌──────┴──────┐
       │ YES         │ NO
       ▼             ▼
┌──────────────┐  ┌──────────────┐
│ Analyze and  │  │ Execute      │
│ plan rollback│  │ SRP-001 NOW  │
└──────────────┘  └──────────────┘
```

### 9.2 Command Quick Reference

| Command | Purpose | Parameters |
|---------|---------|------------|
| SAFE_MODE | Enter safe mode | None |
| BANK_SWITCH | Switch active bank | None |
| BANK_QUERY | Report active bank | None |
| UPLOAD_START | Begin upload | bank, size |
| UPLOAD_DATA | Send data block | seq, data |
| UPLOAD_END | Finalize upload | checksum |
| JETSON_POWER | Control Jetson power | ON/OFF |

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
