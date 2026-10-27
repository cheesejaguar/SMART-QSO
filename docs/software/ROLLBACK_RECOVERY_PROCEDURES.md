# SMART-QSO Rollback and Recovery Procedures

**Document ID**: SQSO-RRP-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft (plan; no independent CDR has been held, `docs/BASELINE_PARAMETERS.md` §1)

## 1. Purpose

This document defines the rollback and recovery procedures for SMART-QSO flight software per NPR 7150.2 §4.5.5, ensuring safe recovery from software faults or failed updates.

**Version 1.2 (2026-10-24): what exists.** The OBC flight software is Rust (`software/rust/README.md`). The image
(`software/rust/firmware`) starts from reset at 0x08000000. There is no bootloader, golden image, staging area or
boot pointer, and the `CMD_SW_UPLOAD_*` commands of §5.3 are not in the command dictionary
(`software/flight/command_dictionary.json`). The partition scheme, update flow and image rollback in §2 and §5 are
therefore plans. The recovery that the software provides today:

- the hardware watchdog (IWDG, 25.6 s: `WDT_HW_TIMEOUT_MS`, reload 3200, 24.1–27.8 s), started at reset and refreshed
  only at the TPS3850 supervisor's WDI falling edges, every 10 s while every main-loop stage checks in (SRS-F050,
  SRS-F146; `software/flight/docs/RTOS_ARCHITECTURE.md` §8), with state saved before a stall reset; the supervisor
  (minimum 29.3 s, `hardware/obc/OBC_DESIGN.md`) resets the OBC if the IWDG fails;
- safe mode and the mode manager's recovery events (`software/rust/sqso-fsw/src/mode.rs`);
- records that survive resets and power loss: two-slot NVM records with CRC-32 for the TX-OFF latch, keys and replay
  counters, contact, RF-silence and deployment state, and CRC-checked blobs for the fault log, mission data, EPS state
  and system state (`software/flight/docs/MEMORY_POLICY.md` §8);
- the RESET telecommand, which is only logged until a hardware reset path exists (command dictionary);
- `PLD_DISABLE` (Jetson off; the OBC sends its template beacons) and `PLD_ENABLE`, the only payload commands.

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

Revision 1.0 values (plan). As built, the only watchdog is the 25.6 s IWDG with the 10 s supervisor kick (§1).

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
| Battery SOC | ≥60% (above the 55 % ACTIVE threshold, BASELINE §5) | Telemetry |
| Thermal | Within limits | Telemetry |
| Link margin | ≥3 dB predicted for the pass, uplink and downlink | `docs/environment/LINK_BUDGET_ANALYSIS.md` v3.5 (BASELINE §7): telecommand at ≥ 64 dBm EIRP ≥ +8.0 dB at 10° and ≥ +6.2 dB at 5°; beacon +7.9 dB at 5° (median spacecraft gain), +0.9 dB at the 10th percentile; analysis |
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

Plan. Of the commands below only `PLD_DISABLE` (and `PLD_ENABLE`) exist (`software/flight/command_dictionary.json`).

| Command | Action |
|---------|--------|
| CMD_PL_RESET | Restart Jetson |
| CMD_PL_SAFE | Boot safe mode |
| CMD_PL_DISABLE | Power off, use OBC beacons |
| CMD_PL_UPDATE | Stage new payload SW |

### 6.3 Model Update/Rollback

Not in the flight baseline: community model updates are ground-side research only, no upload path exists on board,
and any future upload would be data only, adopted solely by authenticated telecommand from a designated station
(`docs/BASELINE_PARAMETERS.md` §2; 47 CFR 97.211). The table is the Revision 1.0 idea, kept for that decision.

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
| 1.1 | 2026-10-16 | SMART-QSO Team | §5.1 link-margin check cites LINK_BUDGET_ANALYSIS v2.0 for the predicted margins (finding TB-05) |
| 1.2 | 2026-10-24 | SMART-QSO Team | §1 note: what recovery the Rust flight software provides; no bootloader, golden image or upload commands exist, so §2 and §5 are plans |
| 1.3 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): status Draft (no CDR held); §1 watchdog 25.6 s with the supervisor kick, RESET only logged, the payload commands; §3.1 and §6.2 marked as plans; §6.3 model updates not in the baseline; §5.1 link margins from LBA v3.5 |
