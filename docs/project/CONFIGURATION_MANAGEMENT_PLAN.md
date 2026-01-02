# Configuration Management Plan (CMP)

**Document Number**: SMART-QSO-CMP-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Draft

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- NPR 7150.2 Rev B - NASA Software Engineering Requirements

---

## 1. Introduction

### 1.1 Purpose

This Configuration Management Plan (CMP) establishes the processes, procedures, and tools for managing configuration items throughout the SMART-QSO mission lifecycle.

### 1.2 Scope

This plan covers:
- Configuration identification
- Configuration control
- Configuration status accounting
- Configuration audits
- Software, firmware, hardware, and documentation

---

## 2. Configuration Management Organization

### 2.1 Roles and Responsibilities

| Role | Responsibilities |
|------|------------------|
| Configuration Manager | Overall CM process, baseline management |
| Software Lead | Software CI identification and control |
| Systems Engineer | System-level CI coordination |
| Quality Assurance | Audit participation, verification |
| CCB Chair | Change control board leadership |

### 2.2 Configuration Control Board (CCB)

| Position | Role |
|----------|------|
| Chair | Project Manager or designee |
| Members | Systems Engineer, Software Lead, QA |
| Secretary | Configuration Manager |

CCB meets:
- Weekly during active development
- As-needed after CDR
- Emergency sessions for critical issues

---

## 3. Configuration Identification

### 3.1 Configuration Item Types

| Type | Prefix | Examples |
|------|--------|----------|
| Software | SW | Flight software, payload software |
| Firmware | FW | OBC firmware, EPS firmware |
| Hardware | HW | Schematics, PCB layouts |
| Document | DOC | Requirements, design, procedures |
| Model | MDL | CAD models, simulations |

### 3.2 Configuration Item List

#### Software Configuration Items

| CI ID | Name | Description |
|-------|------|-------------|
| SW-001 | OBC Flight Software | Main flight software for OBC |
| SW-002 | Jetson Payload Software | AI inference software |
| SW-003 | Ground Station Software | Ground operations tools |
| SW-004 | Simulation Software | Mission simulation |
| SW-005 | Test Software | Automated test suites |

#### Firmware Configuration Items

| CI ID | Name | Description |
|-------|------|-------------|
| FW-001 | OBC Bootloader | Secure bootloader |
| FW-002 | EPS Firmware | Power management firmware |
| FW-003 | RF Modem Firmware | Communication firmware |

#### Document Configuration Items

| CI ID | Name | Document Number |
|-------|------|-----------------|
| DOC-001 | Software Development Plan | SMART-QSO-SDP-001 |
| DOC-002 | Software Requirements Spec | SMART-QSO-SRS-001 |
| DOC-003 | Software Design Description | SMART-QSO-SDD-001 |
| DOC-004 | Software Test Plan | SMART-QSO-STP-001 |
| DOC-005 | Interface Control Documents | SMART-QSO-ICD-xxx |
| DOC-006 | Systems Engineering Mgmt Plan | SMART-QSO-SEMP-001 |
| DOC-007 | Requirements Traceability Matrix | SMART-QSO-RTM-001 |
| DOC-008 | Configuration Mgmt Plan | SMART-QSO-CMP-001 |

### 3.3 Naming Conventions

#### Software Files
```
<module>_<function>.<ext>
Example: eps_control.c, fault_mgmt.h
```

#### Document Files
```
<TYPE>_<TITLE>.md
Example: SOFTWARE_REQUIREMENTS.md, ICD_OBC_JETSON.md
```

#### Version Tags
```
v<MAJOR>.<MINOR>.<PATCH>[-<prerelease>]
Example: v1.0.0, v0.2.0-dev, v1.0.1-rc1
```

---

## 4. Configuration Control

### 4.1 Baseline Definitions

| Baseline | Established | Contents | Authority |
|----------|-------------|----------|-----------|
| Functional | PDR | Requirements, ConOps | CCB |
| Allocated | CDR | Design, ICDs | CCB |
| Product | FRR | As-built configuration | CCB |

### 4.2 Change Control Process

```
        +-------------+
        |   Change    |
        |   Request   |
        +------+------+
               |
               v
        +------+------+
        |   Impact    |
        |  Assessment |
        +------+------+
               |
               v
        +------+------+
        |    CCB      |
        |   Review    |
        +------+------+
               |
        +------+------+
        |             |
   Approved      Rejected
        |             |
        v             v
  +-----+-----+  +----+----+
  |  Implement |  | Archive |
  +-----+-----+  +---------+
        |
        v
  +-----+-----+
  |  Verify   |
  +-----+-----+
        |
        v
  +-----+-----+
  |  Close    |
  +-----------+
```

### 4.3 Change Request (CR) Classification

| Class | Description | Authority | Timeline |
|-------|-------------|-----------|----------|
| Class I | Major change affecting baselines | CCB | Next CCB meeting |
| Class II | Minor change within scope | SW Lead | 48 hours |
| Class III | Editorial/typo | Any team member | Immediate |

### 4.4 Change Request Template

```
Change Request Number: CR-YYYY-NNN
Date Submitted: YYYY-MM-DD
Submitter: Name

1. Configuration Item Affected:
   CI ID:
   Current Version:

2. Description of Change:

3. Reason for Change:

4. Impact Assessment:
   - Technical:
   - Schedule:
   - Cost:
   - Risk:

5. Affected Documents:

6. Verification Required:

7. CCB Decision:
   [ ] Approved
   [ ] Approved with conditions:
   [ ] Rejected
   Reason:

8. Implementation:
   Assigned to:
   Due date:
   Completed:

9. Verification:
   Verified by:
   Date:
```

---

## 5. Version Control

### 5.1 Repository Structure

```
SMART-QSO/
├── .github/
│   └── workflows/          # CI/CD pipelines
├── docs/                   # Documentation
│   ├── interfaces/         # ICDs
│   ├── project/            # Project documents
│   ├── requirements/       # Requirements docs
│   ├── software/           # Software docs
│   └── verification/       # V&V documents
├── hardware/               # Hardware documents
│   ├── electrical/
│   └── mechanical/
├── mission/                # Mission documents
├── software/               # Source code
│   ├── flight/             # OBC flight software
│   ├── payload/            # Jetson software
│   ├── ground/             # Ground software
│   └── simulation/         # Simulation code
└── tests/                  # System tests
```

### 5.2 Branching Strategy

```
main ─────────────────────────────────────────────>
       \                      \            \
        develop ──────────────────────────────>
              \         \         \
               feature/  feature/  release/
               xyz       abc       v1.0
```

| Branch | Purpose | Protection |
|--------|---------|------------|
| main | Flight-ready code | Protected, requires review |
| develop | Integration branch | Protected |
| feature/* | Feature development | None |
| release/* | Release preparation | Protected |
| hotfix/* | Critical fixes | None |

### 5.3 Commit Guidelines

Commit messages follow conventional commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Test additions
- `chore`: Build/tooling

Example:
```
feat(eps): add power mode state machine

Implements SRS-F012 through SRS-F015 for power mode
management with automatic transitions based on SOC.

Refs: SRS-F012, SRS-F013, SRS-F014, SRS-F015
```

### 5.4 Tagging Releases

```bash
# Create annotated tag for release
git tag -a v1.0.0 -m "Release v1.0.0 - Flight software baseline"

# Tag format for milestones
git tag -a PDR-baseline -m "PDR baseline configuration"
git tag -a CDR-baseline -m "CDR baseline configuration"
git tag -a FRR-baseline -m "FRR baseline configuration"
```

---

## 6. Configuration Status Accounting

### 6.1 Status Reports

| Report | Frequency | Content |
|--------|-----------|---------|
| Weekly Status | Weekly | Open CRs, recent changes |
| Baseline Report | Per baseline | All CIs at baseline |
| Audit Report | Per audit | Audit findings |
| Release Notes | Per release | Changes since last release |

### 6.2 CI Status Tracking

| Status | Description |
|--------|-------------|
| Draft | Under development |
| Review | Under peer review |
| Approved | Approved but not baselined |
| Baselined | Part of formal baseline |
| Superseded | Replaced by newer version |

### 6.3 Traceability

All changes tracked via:
1. Git commit history
2. GitHub Issues/PRs
3. Change Request log
4. Requirements Traceability Matrix

---

## 7. Configuration Audits

### 7.1 Audit Types

| Audit | Timing | Purpose |
|-------|--------|---------|
| Functional Configuration Audit (FCA) | Pre-TRR | Verify SW meets requirements |
| Physical Configuration Audit (PCA) | Pre-FRR | Verify build matches documentation |

### 7.2 FCA Checklist

- [ ] All requirements traced to test cases
- [ ] All test cases executed and passed
- [ ] All waivers documented and approved
- [ ] Requirements coverage complete
- [ ] No open critical defects

### 7.3 PCA Checklist

- [ ] Build from tagged source succeeds
- [ ] Binary matches flight binary
- [ ] Documentation matches as-built
- [ ] All CIs at correct versions
- [ ] Configuration index complete

---

## 8. Tools

### 8.1 Tool Summary

| Function | Tool | Version |
|----------|------|---------|
| Version Control | Git | 2.x |
| Repository Hosting | GitHub | - |
| CI/CD | GitHub Actions | - |
| Issue Tracking | GitHub Issues | - |
| Documentation | Markdown | - |
| Build System | CMake | 3.16+ |

### 8.2 GitHub Configuration

Repository settings:
- Branch protection on main and develop
- Require PR reviews (minimum 1)
- Require status checks to pass
- Require signed commits for releases
- Enable GitHub Actions

---

## 9. Release Process

### 9.1 Release Workflow

```
1. Create release branch from develop
   git checkout -b release/v1.0.0 develop

2. Update version numbers
   - CMakeLists.txt
   - smart_qso.h

3. Final testing on release branch

4. Create release tag
   git tag -a v1.0.0 -m "Release v1.0.0"

5. Merge to main
   git checkout main
   git merge release/v1.0.0

6. Merge back to develop
   git checkout develop
   git merge release/v1.0.0

7. Create GitHub release with notes
```

### 9.2 Release Notes Template

```
# SMART-QSO Flight Software v1.0.0

## Summary
Brief description of this release.

## New Features
- Feature 1 (SRS-Fxxx)
- Feature 2 (SRS-Fxxx)

## Bug Fixes
- Fix description (#issue-number)

## Known Issues
- Issue description

## Configuration
- OBC Firmware: FW-001 v1.0.0
- Payload Software: SW-002 v1.0.0

## Verification
- Unit test coverage: XX%
- All system tests: PASS
- Baseline tag: v1.0.0

## Compatibility
- Hardware: OBC Rev A
- Tools: GCC ARM 12.x, CMake 3.16+
```

---

## 10. Archive and Backup

### 10.1 Backup Schedule

| Item | Frequency | Retention |
|------|-----------|-----------|
| Git repository | Continuous (GitHub) | Permanent |
| CI artifacts | Per build | 90 days |
| Release binaries | Per release | Permanent |
| Test data | Per test campaign | 1 year |

### 10.2 Archive Requirements

All flight baselines archived with:
- Complete source code
- Build tools and dependencies
- Build instructions
- Binary images
- Test results
- Documentation

---

## Appendix A: Configuration Item Index Template

```
SMART-QSO Configuration Item Index
Baseline: [FRR/CDR/PDR]
Date: YYYY-MM-DD

| CI ID | Name | Version | Location | Status |
|-------|------|---------|----------|--------|
| SW-001 | OBC Flight Software | v1.0.0 | software/flight | Baselined |
| ... | ... | ... | ... | ... |
```

---

## Appendix B: Acronyms

| Acronym | Definition |
|---------|------------|
| CCB | Configuration Control Board |
| CI | Configuration Item |
| CM | Configuration Management |
| CMP | Configuration Management Plan |
| CR | Change Request |
| FCA | Functional Configuration Audit |
| PCA | Physical Configuration Audit |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
