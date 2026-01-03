# SMART-QSO Rollback and Recovery Procedures

**Document ID**: SQSO-RRP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the rollback and recovery procedures for SMART-QSO flight software per NPR 7150.2 §4.5.5, ensuring safe recovery from software faults or failed updates.

## 2. Rollback Architecture

### 2.1 Software Partition Scheme

```
FLASH MEMORY MAP (32 MB External)

┌─────────────────────────────────────┐ 0x00000000
│  Bootloader (Protected)             │ 64 KB
├─────────────────────────────────────┤ 0x00010000
│  Primary Flight Software            │ 512 KB
│  (Active Image)                     │
├─────────────────────────────────────┤ 0x00090000
│  Backup Flight Software             │ 512 KB
│  (Golden Image)                     │
├─────────────────────────────────────┤ 0x00110000
│  Payload Software Image             │ 4 MB
├─────────────────────────────────────┤ 0x00510000
│  Configuration Data                 │ 64 KB
├─────────────────────────────────────┤ 0x00520000
│  Telemetry Storage                  │ 8 MB
├─────────────────────────────────────┤ 0x00D20000
│  Model Storage                      │ 4 MB
├─────────────────────────────────────┤ 0x01120000
│  Reserved                           │ Remaining
└─────────────────────────────────────┘
```

### 2.2 Boot Selection Logic

```
BOOT SELECTION FLOW

Power On
    │
    ▼
┌─────────────────┐
│  Bootloader     │
│  (Protected)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     Boot Attempts ≥ 3?
│ Check Boot      │────────────────────────►┌─────────────┐
│ Counter         │         Yes             │ Boot Golden │
└────────┬────────┘                         │ Image       │
         │ No                               └──────┬──────┘
         ▼                                         │
┌─────────────────┐                                │
│ Validate Primary│                                │
│ CRC32           │                                │
└────────┬────────┘                                │
         │                                         │
    ┌────┴────┐                                    │
    │Valid?   │                                    │
    └────┬────┘                                    │
    Yes  │  No                                     │
         │  └──────────────────────────────────────┤
         ▼                                         │
┌─────────────────┐                                │
│ Increment Boot  │                                │
│ Counter         │                                │
└────────┬────────┘                                │
         │                                         │
         ▼                                         │
┌─────────────────┐                                │
│ Execute Primary │                                │
│ Image           │                                │
└────────┬────────┘                                │
         │                                         │
         ▼                                         ▼
┌─────────────────┐                    ┌─────────────────┐
│ Clear Boot      │                    │ Set SAFE Mode   │
│ Counter on      │                    │ Flag            │
│ Successful Init │                    └─────────────────┘
└─────────────────┘
```

### 2.3 Image Validation

| Validation | Method | Criteria |
|------------|--------|----------|
| Integrity | CRC32 | Matches stored CRC |
| Version | Header check | Valid version format |
| Size | Bounds check | Within partition |
| Magic | Header check | 0x534D5153 ("SMQS") |

## 3. Automatic Recovery

### 3.1 Watchdog Recovery

| Condition | Timeout | Action |
|-----------|---------|--------|
| Task hang | 5 seconds | System reset |
| Boot failure | 30 seconds | Increment counter, reset |
| Init failure | 60 seconds | Transition to SAFE mode |

### 3.2 Safe Mode Triggers

| Trigger | Detection | Response |
|---------|-----------|----------|
| Repeated resets | Boot counter ≥3 | Golden image boot |
| Memory fault | ECC/parity error | Safe mode, reset |
| Stack overflow | Guard check | Task restart |
| Assertion failure | assert() macro | Log and reset |

### 3.3 Recovery State Machine

```
RECOVERY STATE MACHINE

                      ┌──────────────┐
                      │   NOMINAL    │
                      └──────┬───────┘
                             │ Fault detected
                             ▼
                      ┌──────────────┐
              ┌───────│   RECOVERY   │───────┐
              │       └──────┬───────┘       │
    Recoverable│              │ Not          │ Hardware
              │              │ recoverable  │ fault
              ▼              ▼              ▼
       ┌──────────┐   ┌──────────┐   ┌──────────┐
       │ RESTART  │   │  SAFE    │   │FALLBACK  │
       │ TASK     │   │  MODE    │   │(Golden)  │
       └──────────┘   └──────────┘   └──────────┘
              │              │              │
              └──────────────┴──────────────┘
                             │
                             ▼
                      Ground intervention
                      (if needed)
```

## 4. Commanded Recovery

### 4.1 Ground Command Rollback

| Command | Action | Confirmation |
|---------|--------|--------------|
| CMD_SW_ROLLBACK | Boot golden image | Beacon version |
| CMD_SW_PRIMARY | Boot primary image | Beacon version |
| CMD_RESET_BOOT_CNT | Clear boot counter | Telemetry |
| CMD_FORCE_SAFE | Enter safe mode | Mode beacon |

### 4.2 Rollback Procedure (Ground)

```
GROUND-COMMANDED ROLLBACK PROCEDURE

Step  Action                           Verification
────────────────────────────────────────────────────
1     Verify current state             TLM review
2     Save critical telemetry          Ground archive
3     Send CMD_SW_ROLLBACK             ACK received
4     Wait for reset                   30-60 seconds
5     Monitor beacon                   Version check
6     Verify SAFE mode entry           Mode = SAFE
7     Perform health check             All TLM nominal
8     Transition to IDLE               Mode = IDLE
9     Resume operations                As appropriate
```

### 4.3 Emergency Recovery

If both primary and golden images are corrupt:

1. **Safe Mode Beacon**: Minimal beacon with degraded state indication
2. **Upload Capability**: Emergency upload path through bootloader
3. **Recovery Window**: 4 hours of battery-only operation for upload

## 5. Software Update Procedure

### 5.1 Pre-Update Checklist

| Check | Criterion | Verification |
|-------|-----------|--------------|
| Battery SOC | ≥60% | Telemetry |
| Thermal | Within limits | Telemetry |
| Link margin | ≥3 dB | Link analysis |
| Pass duration | ≥10 min | Orbit prediction |
| Backup verified | CRC match | Ground command |

### 5.2 Update Flow

```
SOFTWARE UPDATE FLOW

1. Ground Preparation
   └─ Build release image
   └─ Generate CRC32
   └─ Test on flatsat
   └─ Document version

2. Upload Phase
   └─ Segment image (2 KB blocks)
   └─ Upload to staging area
   └─ Verify each block
   └─ Complete image assembly

3. Validation Phase
   └─ Full CRC verification
   └─ Size verification
   └─ Header validation
   └─ Report to ground

4. Commit Phase
   └─ Command commit
   └─ Copy to primary slot
   └─ Update boot pointer
   └─ Reset for new image

5. Verification Phase
   └─ Boot new image
   └─ Run self-test
   └─ Report version
   └─ Confirm nominal ops
```

### 5.3 Update Commands

| Command | Parameters | Action |
|---------|------------|--------|
| CMD_SW_UPLOAD_START | size, crc | Prepare for upload |
| CMD_SW_UPLOAD_BLOCK | offset, data | Store block |
| CMD_SW_UPLOAD_VERIFY | none | Verify staged image |
| CMD_SW_UPLOAD_COMMIT | none | Activate new image |
| CMD_SW_UPLOAD_ABORT | none | Cancel upload |

### 5.4 Rollback After Failed Update

```
FAILED UPDATE RECOVERY

Update fails at any stage
         │
         ▼
┌─────────────────────────┐
│ Automatic rollback to   │
│ previous working image  │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Set update failure flag │
│ in telemetry            │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Increment failed update │
│ counter                 │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│ Resume with known-good  │
│ configuration           │
└─────────────────────────┘
```

## 6. Payload Software Recovery

### 6.1 Jetson Recovery Modes

| Mode | Trigger | Action |
|------|---------|--------|
| Normal | Power enable | Full boot, TensorRT load |
| Safe | 3 boot failures | Minimal boot, no AI |
| Recovery | Ground command | USB recovery mode |
| Disabled | Power failure | OBC fallback beacons |

### 6.2 Payload Rollback

| Command | Action |
|---------|--------|
| CMD_PL_RESET | Restart Jetson |
| CMD_PL_SAFE | Boot safe mode |
| CMD_PL_DISABLE | Power off, use OBC beacons |
| CMD_PL_UPDATE | Stage new payload SW |

### 6.3 Model Update/Rollback

| Command | Action |
|---------|--------|
| CMD_MODEL_UPLOAD | Upload new TinyLM model |
| CMD_MODEL_ACTIVATE | Switch to new model |
| CMD_MODEL_ROLLBACK | Revert to previous model |
| CMD_MODEL_DEFAULT | Use factory default |

## 7. Configuration Recovery

### 7.1 Configuration Partitions

| Partition | Content | Backup |
|-----------|---------|--------|
| CFG_PRIMARY | Active config | Yes |
| CFG_BACKUP | Previous config | N/A |
| CFG_DEFAULT | Factory defaults | Protected |

### 7.2 Configuration Recovery Commands

| Command | Action |
|---------|--------|
| CMD_CFG_SAVE | Save current to backup |
| CMD_CFG_RESTORE | Restore from backup |
| CMD_CFG_DEFAULT | Load factory defaults |
| CMD_CFG_UPLOAD | Upload new configuration |

## 8. Test Procedures

### 8.1 Rollback Verification Test

```
ROLLBACK VERIFICATION PROCEDURE

Test ID: TP-REC-001
Purpose: Verify automatic rollback functionality

Prerequisites:
- Flatsat or flight hardware
- Ground station connection
- Known-good and test images

Steps:
1. Load known-good image as golden
2. Load intentionally-corrupted image as primary
3. Power cycle spacecraft
4. Verify golden image boots
5. Verify safe mode entry
6. Verify boot counter behavior
7. Command return to primary
8. Verify nominal operations

Pass Criteria:
- Golden image boots within 3 attempts
- Safe mode flag set
- No data loss
- Normal ops resume with valid primary
```

### 8.2 Update Recovery Test

```
UPDATE RECOVERY PROCEDURE

Test ID: TP-REC-002
Purpose: Verify update failure recovery

Prerequisites:
- Flatsat or flight hardware
- Ground station connection
- Valid and invalid update images

Steps:
1. Start with nominal configuration
2. Initiate update with corrupted image
3. Verify upload failure detection
4. Verify automatic rollback
5. Verify failure flag set
6. Retry with valid image
7. Verify successful update
8. Verify rollback capability

Pass Criteria:
- Failed update detected
- No corruption of working image
- Successful retry works
- Rollback verified
```

## 9. Recovery Telemetry

### 9.1 Recovery Status Telemetry

| Parameter | Type | Description |
|-----------|------|-------------|
| BOOT_COUNTER | uint8 | Consecutive boot attempts |
| ACTIVE_IMAGE | enum | PRIMARY, GOLDEN, RECOVERY |
| LAST_RESET_CAUSE | enum | WDT, FAULT, COMMAND, POWER |
| UPDATE_STATUS | enum | NONE, IN_PROGRESS, FAILED, SUCCESS |
| ROLLBACK_COUNT | uint16 | Total rollbacks (mission) |

### 9.2 Recovery Event Log

| Event | Data |
|-------|------|
| BOOT_ATTEMPT | image, counter, crc_status |
| ROLLBACK | from_image, to_image, reason |
| UPDATE_START | size, crc, source |
| UPDATE_COMPLETE | success, duration, new_version |
| UPDATE_FAIL | stage, error_code |

## 10. Ground Support

### 10.1 Recovery Toolkit

| Tool | Purpose |
|------|---------|
| Image Builder | Create update packages |
| CRC Calculator | Verify image integrity |
| Simulator | Test recovery scenarios |
| Log Analyzer | Parse recovery events |

### 10.2 Recovery Checklist (Operations)

```
RECOVERY OPERATIONS CHECKLIST

[ ] Identify failure mode from telemetry
[ ] Review event log for root cause
[ ] Determine recovery action
[ ] Prepare recovery commands
[ ] Verify link budget for pass
[ ] Execute recovery procedure
[ ] Verify recovery success
[ ] Document in anomaly report
[ ] Update lessons learned
```

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
