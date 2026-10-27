# Configuration Management Plan (CMP)

**Document Number**: SMART-QSO-CMP-001
**Version**: 1.1
**Date**: 2026-10-27
**Status**: Draft

**Reference standards** (voluntary for this privately funded mission; `docs/project/CDR_CRITERIA_MATRIX.md` §1):
- NPR 7123.1D §3.2, Configuration Management process (Appendix C)
- NPR 7150.2D, software configuration management requirements (SWE-079 to SWE-085), Class C
- Version 1.0 cited NPR 7120.5 "Rev F" as governing and NPR 7150.2 "Rev B"; neither applies as cited (finding VV-11)

> **Status on 2026-10-27 (CDR review).** No configuration baseline has been established and the repository has no
> Git tags. The working controls are: the document register (`docs/DOCUMENT_REGISTER.md`), the baseline parameters
> (`docs/BASELINE_PARAMETERS.md`, changed only by change request), the change request log
> (`docs/project/CHANGE_REQUEST_LOG.md`, CR-2026-001 to -006), pull requests to `main`, and the automated checks of
> §4.5. The `develop`/`release` branch model of §5.2 in version 1.0 was never used and is withdrawn.

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
| Chair and approver | The PI (licensee KK6GDB); the only approver today |
| Members | The leads of each discipline when named (`docs/project/PROJECT_PLAN.md` §4) |
| Independent check | For a Class I change, at least one reviewer who is not the author reviews the pull request once a second contributor exists; until then the CR record states that the PI approved alone |
| Secretary | The PI; the CR log is the record |

The CCB acts through the change request log and the pull request that implements the change; a Class I change is
approved only when its CR entry (template §4.4) names the decision, the date and the verification. Meetings are held
when a Class I change is pending and at each review gate.

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

| CI ID | Name | Location |
|-------|------|----------|
| SW-001 | OBC flight software (Rust: `sqso-protocol`, `sqso-fsw`, `sqso-ffi`) and the C headers | `software/rust/`, `software/flight/include/` |
| SW-002 | STM32L4R5ZI firmware image (Embassy) | `software/rust/firmware/`; toolchain pinned by `software/rust/rust-toolchain.toml` |
| SW-003 | Jetson payload software | `software/payload/` |
| SW-004 | Ground software (decoder, command tools, dashboards) | `software/ground/` |
| SW-005 | Simulations used for verification: SIL, ADCS Monte Carlo, orbit lifetime, CDR analysis scripts | `software/rust/sqso-sil/`, `software/simulation/` |
| SW-006 | Test software: C test suites, Rust tests, HIL probes | `software/flight/tests/`, crate tests |

#### Firmware Configuration Items

| CI ID | Name | Description |
|-------|------|-------------|
| FW-001 | OBC bootloader | Not yet designed; none in the baseline |
| FW-002 | EPS firmware | Only if the ISIS ICEPS2 alternate is bought (vendor CI, identified by part number and version) |
| FW-003 | TNC / radio firmware | Vendor or custom, identified at selection (CR-2026-005) |

#### Hardware Configuration Items

| CI ID | Name | Description |
|-------|------|-------------|
| HW-001 | Engineering model | Board revisions, part lot list, serial numbers; established at EM build |
| HW-002 | Flight model | As HW-001; frozen at the pre-ship review (SYS-089) |
| HW-003 | Purchased items | Vendor part number, revision, serial, certificate of conformance |

#### Document Configuration Items

| CI ID | Name | Document Number |
|-------|------|-----------------|
| DOC-001 | Software Development Plan | SMART-QSO-SDP-001 |
| DOC-002 | Software Requirements Spec | SMART-QSO-SRS-001 |
| DOC-003 | Software Design Description | SMART-QSO-SDD-001 |
| DOC-004 | Software Test Plan | SMART-QSO-STP-001 |
| DOC-005 | Interface Control Documents | SMART-QSO-ICD-xxx |
| DOC-006 | Systems Engineering Mgmt Plan | SMART-QSO-SEMP-001 |
| DOC-007 | Requirements Traceability Matrix | `docs/verification/RTM.csv` (generated) |
| DOC-008 | Configuration Mgmt Plan | SMART-QSO-CMP-001 |
| DOC-009 | Baseline Parameters | SQSO-BL-001 |
| DOC-010 | Mission and system requirements | SQSO-REQ-001, SQSO-REQ-002 |
| DOC-011 | VCRM | SQSO-VCRM-001 (generated block) |
| DOC-012 | Every other registered document | `docs/DOCUMENT_REGISTER.md` is the configuration index |

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

| Baseline | Established | Contents | Authority | Git tag |
|----------|-------------|----------|-----------|---------|
| Functional | PDR (2027-01-12) | BASELINE parameters, MRD, system requirements, ConOps | PI after the PDR | `baseline/functional-pdr` |
| Allocated | CDR (2027-08-03) | Software requirements, design, ICDs, build-to package, ETP | PI after the CDR | `baseline/allocated-cdr` |
| Product | Pre-ship review (2028-03-14) | As-built FM, flight image (hash), test reports, VCRM | PI after the pre-ship review | `baseline/product-psr` |

The baseline parameters document is under change control now (BASELINE §0 rule 3), ahead of the formal functional
baseline.

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

Class I covers any change to a value in `docs/BASELINE_PARAMETERS.md`, to the text, method or parent of a mission or
system requirement, to an ICD, to a safety-critical software component, or to a baselined item after a baseline is
tagged. The CDR review's requirement changes (MRD Rev D, SYS Rev E) are Class I and are logged as CR-2026-007.

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

### 4.5 Automated Configuration Checks

Run in CI on every pull request (`.github/workflows/program-checks.yml`); a failing check blocks the merge:

| Check | Controls |
|-------|----------|
| `scripts/check_parameters.py` | Key parameters agree between code and documents |
| `scripts/check_traceability.py --check` | No undefined requirement IDs in code; no conflicting descriptions; every procedure cites a requirement |
| `scripts/gen_rtm.py --check` | The RTM matches the specifications |
| `scripts/tbd_register.py --check` | The TBD/TBR count only goes down |
| `scripts/check_document_ids.py --check` | Unique document IDs; register rows and revisions current |
| `scripts/gen_command_dictionary.py --check` | Command dictionary matches the flight code |
| `scripts/register_status.py --check`, `scripts/check_doc_references.py` | Findings register counts; no broken document links |
| `software/simulation/cdr/pm/gen_vcrm.py --check` | The VCRM block matches the specifications (to be added to CI) |

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

Trunk-based: `main` is the only long-lived branch; every change arrives by pull request from a short-lived branch
and must pass the CI checks of §4.5. Baselines and flight-image releases are annotated tags on `main` (§5.4). The
`develop`, `release/*` and `hotfix/*` branches of version 1.0 were never created and are withdrawn;
`docs/development/BRANCHING_STRATEGY.md` still describes them and should be brought into line.

| Branch | Purpose | Protection |
|--------|---------|------------|
| main | Integrated, CI-checked state | Protected; pull request and passing checks required |
| short-lived topic branches | One change each | CI on push |

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
| Build System | Cargo (Rust toolchain pinned in `software/rust/rust-toolchain.toml`) and CMake for the C test suites | 3.16+ (CMake) |

### 8.2 GitHub Configuration

Repository settings (intended; the review did not inspect the GitHub settings):
- Branch protection on main
- Require PR reviews (minimum 1)
- Require status checks to pass
- Require signed commits for releases
- Enable GitHub Actions

---

## 9. Release Process

### 9.1 Release Workflow

```
1. Open a pull request on main with the version change and the release notes;
   all §4.5 checks and the flight-software CI (tests, coverage gates, clippy,
   the firmware build) must pass.
2. Merge, then tag the merge commit on main:
   git tag -a v1.0.0 -m "Release v1.0.0"
3. Record the firmware image hash (SW-002) and the toolchain version
   (software/rust/rust-toolchain.toml) in the release notes and the
   software version description.
4. Create the GitHub release with the notes and the image.
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
- Tools: Rust toolchain per software/rust/rust-toolchain.toml; CMake for the C test suites
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
| 1.1 | 2026-10-27 | CDR review, PM/SE/V&V | Reference standards corrected (NPR 7123.1D, NPR 7150.2D; NPR 7120.5 not governing); status notice (no baseline, no tags); CCB as it can work with one approver; real configuration items (Rust crates, firmware, simulations; hardware CIs); Class I scope (baseline parameters, requirements, ICDs; CR-2026-007); dated baselines with tag names; automated configuration checks; trunk-based branching replaces the unused develop/release model |
