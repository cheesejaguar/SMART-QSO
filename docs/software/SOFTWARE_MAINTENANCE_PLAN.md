# SMART-QSO Software Maintenance Plan

**Document ID**: SQSO-SMP-001
**Version**: 1.2
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft (plan; no independent CDR has been held, `docs/BASELINE_PARAMETERS.md` §1)

## 1. Purpose

This document defines the Software Maintenance Plan for SMART-QSO per NPR 7150.2 §4.6, establishing procedures for maintaining flight software throughout the mission lifecycle.

**Version 1.1 (2026-10-24).** The OBC flight software is Rust (`software/rust/README.md`) and runs on the Embassy
executor, with no RTOS. Its version is 0.2.0-dev (`docs/software/SOFTWARE_VERSION_DESCRIPTION.md`); 1.0.0 in §2.1 is the
planned flight version. **On-orbit update of the OBC software is not implemented.** There is no bootloader
(`software/flight/docs/BOOTLOADER_REQUIREMENTS.md` §1.1), and the command dictionary has no upload commands. The OBC
rows of §5 are plans. §11.2 and §12.1 are updated to the toolchain as built.

**Version 1.2 (2026-10-27).** No on-orbit update path exists for the Jetson either: the uplink carries only the
authenticated telecommands of `software/flight/command_dictionary.json` (none transfers files), and at 1200 bps a
100 MB transfer is not credible. Community model updates are not in the flight baseline (`docs/BASELINE_PARAMETERS.md`
§2); any future upload would be data only, adopted solely by authenticated telecommand from a designated station
(47 CFR 97.211). §2.1 and §5.1 are corrected accordingly.

## 2. Scope

### 2.1 Software Covered

| Software | Version | Maintainability |
|----------|---------|-----------------|
| Flight Software (OBC) | 0.2.0-dev (1.0.0 planned) | Ground update before launch only (no bootloader, no upload commands) |
| Payload Software (Jetson) | 1.0.0 (`software/payload/pyproject.toml`) | Ground update before launch only |
| Ground Software | 1.0.0 | Ground update |
| Test Software | 1.0.0 | Ground only |

### 2.2 Maintenance Types

| Type | Description | Frequency |
|------|-------------|-----------|
| Corrective | Bug fixes | As needed |
| Adaptive | Environment changes | Rare |
| Perfective | Enhancements | Planned |
| Preventive | Reliability improvements | Periodic |

## 3. Maintenance Organization

### 3.1 Roles and Responsibilities

| Role | Responsibilities |
|------|------------------|
| Software Maintenance Lead | Overall maintenance coordination |
| Flight Software Engineer | OBC software changes |
| Payload Software Engineer | Jetson software changes |
| Test Engineer | Verification of changes |
| Configuration Manager | Version control, releases |
| Mission Operations | Upload coordination |

### 3.2 Maintenance Team

```
MAINTENANCE ORGANIZATION

Project Manager
       │
Software Maintenance Lead
       │
   ┌───┴───┐
   │       │
Flight SW  Payload SW
Engineer   Engineer
   │       │
   └───┬───┘
       │
Test Engineer
       │
Config Manager
```

## 4. Maintenance Process

### 4.1 Problem Reporting

```
PROBLEM REPORT FLOW

1. Problem Discovery
   └─ Operations, analysis, or user report

2. Problem Report (PR) Creation
   └─ Document symptoms, conditions
   └─ Assign PR number
   └─ Initial severity assessment

3. Triage
   └─ Confirm problem
   └─ Assign priority
   └─ Assign to engineer

4. Analysis
   └─ Root cause determination
   └─ Impact assessment
   └─ Solution options

5. Resolution
   └─ Fix development
   └─ Test
   └─ Review
   └─ Release

6. Closure
   └─ Verify in operations
   └─ Close PR
   └─ Update documentation
```

### 4.2 Problem Severity

| Severity | Definition | Response Time |
|----------|------------|---------------|
| Critical | Mission loss imminent | Immediate |
| High | Mission degraded | 24 hours |
| Medium | Reduced capability | 1 week |
| Low | Minor issue | Next release |

### 4.3 Change Request Process

| Step | Activity | Approval |
|------|----------|----------|
| 1 | Submit change request | Requester |
| 2 | Technical review | SW Lead |
| 3 | Impact assessment | Team |
| 4 | Approval | CCB |
| 5 | Implementation | Engineer |
| 6 | Verification | Test |
| 7 | Release | CM |
| 8 | Deployment | Ops |

## 5. On-Orbit Software Updates

### 5.1 Update Capability

| Component | Update Method | Size Limit |
|-----------|---------------|------------|
| OBC firmware | Bootloader upload (plan; no bootloader exists) | 256 KB |
| OBC parameters | The telecommands that set a value (`SET_BEACON`, `SET_CONTACT_TIMEOUT`, `ADCS_SET_*`) | per command |
| Jetson software | None (no file transfer on the uplink) | — |
| Jetson parameters | None | — |

### 5.2 Update Procedure

```
ON-ORBIT SOFTWARE UPDATE PROCEDURE

Pre-Upload:
[ ] Change approved by CCB
[ ] Software tested on flatsat
[ ] Update package prepared
[ ] Rollback procedure verified
[ ] Operations team briefed

Upload:
[ ] Verify spacecraft health
[ ] Upload software package
[ ] Verify upload integrity (CRC)
[ ] Command software activation
[ ] Verify new version running

Post-Upload:
[ ] Monitor for 24 hours minimum
[ ] Verify all functions nominal
[ ] Document update in ops log
[ ] Update configuration records
[ ] Close change request
```

### 5.3 Rollback Procedure

| Trigger | Action |
|---------|--------|
| Upload failure | Retry or abort |
| Activation failure | Revert to previous |
| Post-update anomaly | Command rollback |
| Critical failure | Safe mode + rollback |

### 5.4 Update Constraints

| Constraint | Limit |
|------------|-------|
| Battery SOC | >70% |
| Pass duration | >8 minutes |
| Thermal state | Nominal |
| Mode | ACTIVE or IDLE |
| Pending commands | None |

## 6. Version Control

### 6.1 Version Numbering

```
VERSION FORMAT: MAJOR.MINOR.PATCH

MAJOR: Significant functional changes
MINOR: New features, enhancements
PATCH: Bug fixes, minor changes

Examples:
  1.0.0 → Initial flight release
  1.0.1 → Bug fix
  1.1.0 → New feature
  2.0.0 → Major update
```

### 6.2 Branch Strategy

| Branch | Purpose | Lifetime |
|--------|---------|----------|
| main | Production releases | Permanent |
| develop | Integration | Permanent |
| feature/* | New features | Temporary |
| hotfix/* | Critical fixes | Temporary |
| release/* | Release prep | Temporary |

### 6.3 Release Process

| Step | Activity |
|------|----------|
| 1 | Create release branch |
| 2 | Complete testing |
| 3 | Update version numbers |
| 4 | Generate release notes |
| 5 | CCB approval |
| 6 | Tag release |
| 7 | Merge to main |
| 8 | Archive artifacts |

## 7. Testing Requirements

### 7.1 Maintenance Testing

| Change Type | Required Testing |
|-------------|------------------|
| Critical fix | Full regression |
| Bug fix | Affected area + smoke |
| Enhancement | New feature + regression |
| Parameter | Functional verification |

### 7.2 Test Environment

| Environment | Use |
|-------------|-----|
| Unit test | Developer testing |
| Flatsat | Integration testing |
| HITL | System testing |
| Ops sim | Operations verification |

### 7.3 Test Documentation

| Document | Content |
|----------|---------|
| Test plan | Scope, approach |
| Test procedures | Step-by-step |
| Test report | Results, issues |
| Release notes | Changes, known issues |

## 8. Documentation Maintenance

### 8.1 Documents to Maintain

| Document | Update Trigger |
|----------|----------------|
| Software Design | Design changes |
| Requirements | Requirement changes |
| User Manual | Interface changes |
| Test Procedures | Test changes |
| SVD | Each release |
| Known Issues | New issues/fixes |

### 8.2 Documentation Process

| Step | Activity |
|------|----------|
| 1 | Identify documentation impact |
| 2 | Draft updates |
| 3 | Review changes |
| 4 | Approve updates |
| 5 | Release with software |

## 9. Metrics and Reporting

### 9.1 Maintenance Metrics

| Metric | Target | Frequency |
|--------|--------|-----------|
| Open PRs | <10 | Weekly |
| PR closure time | <30 days | Monthly |
| Defect density | <1 per KLOC | Per release |
| Update success rate | >95% | Per update |

### 9.2 Status Reporting

| Report | Frequency | Audience |
|--------|-----------|----------|
| Weekly status | Weekly | Team |
| Monthly summary | Monthly | Management |
| Release report | Per release | All |
| Annual review | Yearly | Stakeholders |

## 10. End of Mission

### 10.1 Software Disposition

| Activity | Timing |
|----------|--------|
| Final software archive | EOL-30 days |
| Documentation archive | EOL-30 days |
| Lessons learned | EOL |
| Knowledge transfer | EOL+30 days |

### 10.2 Archive Contents

| Item | Format |
|------|--------|
| Source code | Git repository |
| Build artifacts | Binary + scripts |
| Documentation | PDF archive |
| Test results | PDF + data |
| Problem reports | Database export |

## 11. Training

### 11.1 Maintenance Training

| Topic | Audience | Frequency |
|-------|----------|-----------|
| Maintenance process | All | Initial |
| Tools and environment | Engineers | Initial |
| On-orbit updates | Ops + SW | Before launch |
| Lessons learned | All | Periodic |

### 11.2 Skills Required

| Skill | Role |
|-------|------|
| Rust programming (`no_std`, embedded; replaces C programming, 2026-10-24) | Flight SW |
| C programming (the C ABI headers, the test suites, the flight-log shim) | Flight SW |
| Python | Payload SW |
| Embassy (embassy-stm32, embassy-executor; replaces FreeRTOS, which was never integrated) | Flight SW |
| Git | All |
| Testing | All |

## 12. Tools

### 12.1 Development Tools

| Tool | Purpose |
|------|---------|
| Git | Version control |
| CMake | Build system (host; drives cargo) |
| Rust toolchain 1.97.0 (`software/rust/rust-toolchain.toml`) | Compiler (replaces GCC ARM, 2026-10-24) |
| probe-rs | Flash and debug over SWD (`software/rust/firmware/.cargo/config.toml`) |
| J-Link | Debug |
| VS Code | IDE |

### 12.2 Management Tools

| Tool | Purpose |
|------|---------|
| GitHub Issues | Problem tracking |
| GitHub Projects | Task management |
| Wiki | Documentation |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-24 | SMART-QSO Team | The flight software is Rust on the Embassy executor: on-orbit OBC update not implemented (no bootloader, no upload commands); skills and tools updated (Rust, Embassy, probe-rs) |
| 1.2 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): status Draft (no CDR held); §1 no Jetson upload path, model updates not in the baseline; §2.1 and §5.1 corrected |
