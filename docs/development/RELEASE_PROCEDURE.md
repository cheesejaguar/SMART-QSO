# Software Release Procedure
## SMART-QSO Flight Software

**Document Number:** SQSO-DEV-005
**Revision:** E
**Date:** 2026-10-27
**Reference:** NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the procedure for releasing flight software versions, ensuring proper verification, documentation, and configuration control per NASA software assurance requirements.

## 2. Scope

This procedure applies to:
- OBC flight software releases
- Payload (Jetson) software releases
- Ground software releases

---

## 3. Version Numbering

### 3.1 Semantic Versioning

Format: `MAJOR.MINOR.PATCH[-QUALIFIER]`

| Component | When to Increment |
|-----------|-------------------|
| MAJOR | Breaking changes, major new features |
| MINOR | New features, backward compatible |
| PATCH | Bug fixes, minor changes |
| QUALIFIER | Pre-release: alpha, beta, rc1, etc. |

### 3.2 Examples

| Version | Description |
|---------|-------------|
| 0.1.0 | Initial development |
| 0.2.0-alpha | Feature addition, pre-release |
| 1.0.0-rc1 | Release candidate for CDR |
| 1.0.0 | CDR baseline release |
| 1.0.1 | Bug fix release |
| 1.1.0 | Feature addition post-CDR |

### 3.3 Build Identification

Each build includes:
- Version number
- Git commit hash (short)
- Build timestamp
- Build configuration (FLIGHT/TEST/DEV)

Example: `v1.0.0-abc1234-20260102-FLIGHT`

---

## 4. Release Types

### 4.1 Development Release

| Attribute | Value |
|-----------|-------|
| Purpose | Internal testing |
| Approval | Developer |
| Testing | Unit tests pass |
| Tag | None |
| Branch | develop or feature/* |

### 4.2 Test Release

| Attribute | Value |
|-----------|-------|
| Purpose | Integration testing |
| Approval | SW Lead |
| Testing | Integration tests pass |
| Tag | vX.Y.Z-beta |
| Branch | release/* |

### 4.3 Flight Release

| Attribute | Value |
|-----------|-------|
| Purpose | Flight hardware |
| Approval | Project Manager |
| Testing | Full verification complete |
| Tag | vX.Y.Z |
| Branch | main |

---

## 5. Release Process

### 5.1 Pre-Release Checklist

| Step | Action | Responsible | Complete |
|------|--------|-------------|----------|
| 1 | All code merged to release branch | Developer | [ ] |
| 2 | All unit tests passing | Developer | [ ] |
| 3 | Static analysis clean | Developer | [ ] |
| 4 | Code coverage ≥80% | Developer | [ ] |
| 5 | Code review complete | SW Lead | [ ] |
| 6 | Integration tests passing | Test | [ ] |
| 7 | Version number updated | Developer | [ ] |
| 8 | Change log updated | Developer | [ ] |
| 9 | Documentation updated | Developer | [ ] |

### 5.2 Release Build

The flight software is Rust (`software/rust/README.md`); the flight image is the Embassy firmware for the STM32L4R5ZI.

```bash
# 1. Clean working tree, on the release branch
git status                      # must show nothing to commit
git checkout release/v1.0.0

# 2. Set the version, the same in all four places (no "-dev" for a release):
#    SMART_QSO_VERSION_STRING  software/flight/include/smart_qso.h
#    VERSION                   software/rust/sqso-ffi/src/executive.rs
#    version                   software/rust/Cargo.toml and software/rust/firmware/Cargo.toml

# 3. Run all tests (CI runs the same, plus coverage, clippy, fuzzing and the image checks)
cmake -S software/flight -B build && cmake --build build
ctest --test-dir build --output-on-failure
(cd software/rust && cargo test --workspace --locked)

# 4. Build and package the image, with its build information
scripts/release_package.sh dist
```

`scripts/release_package.sh` refuses a dirty tree or versions that disagree or end in "-dev". It builds the image with
the pinned toolchain (`software/rust/rust-toolchain.toml`) and the locked dependencies, and writes to `dist/`:

| File | Contents |
|------|----------|
| `sqso-firmware-<version>.elf` | The image with symbols, for the debugger and probe-rs |
| `sqso-firmware-<version>.bin` | The flash contents from 0x08000000 |
| `build_info.txt` | Commit, toolchain (`rustc -vV`, `cargo -V`), the two `Cargo.lock` hashes, section sizes, worst-case stack |
| `stack_usage.txt` | The worst-case stack analysis (`scripts/stack_usage.py`, SRS-C003) |
| `SHA256SUMS` | Checksums of the four files above |

CI runs `scripts/release_package.sh --dev dist` on every change (job STM32L4 Cross-Compile), which allows a dirty tree
and a "-dev" version and keeps the package as an artifact.

### 5.3 Release Verification

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 1 | `scripts/release_package.sh dist` | Completes; no errors or warnings | [ ] |
| 2 | Execute unit tests | 100% pass | [ ] |
| 3 | Execute integration tests | 100% pass | [ ] |
| 4 | Verify version string | Correct version | [ ] |
| 5 | Verify `build_info.txt` | Release commit, "release: yes", pinned toolchain | [ ] |
| 6 | Size and stack check (`build_info.txt`) | Text ≤ 256 KB (SRS-C004); worst-case stack ≤ 8 KB (SRS-C003) | [ ] |
| 7 | Load on target hardware, option bytes first (§5.3.1) | Boots correctly; SRAM2 parity on | [ ] |
| 8 | Execute functional test | All pass | [ ] |

### 5.3.1 Programming the Flight Board: Option Bytes (RID AV-07)

The STM32L4R5's SRAM2 parity is an option bit, not a setting of the image: the flight software never writes option
bytes (a corrupted write could brick the processor in orbit). Every board that carries the flight image is programmed
with SRAM2 parity on, once, before the image is loaded, and checked after:

```bash
# The ST-LINK on the board's SWD connector; STM32CubeProgrammer's command-line tool
STM32_Programmer_CLI -c port=SWD mode=UR -ob SRAM2_PE=0      # 0 = parity check on (FLASH_OPTR bit SRAM2_PE)
STM32_Programmer_CLI -c port=SWD -ob displ                   # must list SRAM2_PE : 0x0
probe-rs download --chip STM32L4R5ZITx sqso-firmware-<version>.elf && probe-rs reset --chip STM32L4R5ZITx
```

The option load resets the processor. After the first boot of the image, the console shows
`[SYSTEM] SRAM2 parity on` and the second housekeeping frame `PE=1` (SRS-F154); `PE=0`, or a fault-log entry
"SRAM2 parity off (option bit SRAM2_PE)", fails the step. Record the `-ob displ` output with the build information.
The image links every static and the 8 KB stack in SRAM2 (`software/rust/firmware/memory.x`), so with the bit set every
live byte of RAM has parity (`software/flight/docs/MEMORY_POLICY.md` §10.3).

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 1 | `-ob SRAM2_PE=0`, then `-ob displ` | `SRAM2_PE : 0x0` | [ ] |
| 2 | Load the image, boot | Console `[SYSTEM] SRAM2 parity on`; HK2 `PE=1`; no "SRAM2 parity off" fault entry | [ ] |

### 5.4 Release Documentation

Required documents for flight release:

| Document | Location | Status |
|----------|----------|--------|
| Software Version Description | docs/software/SOFTWARE_VERSION_DESCRIPTION.md | [ ] |
| Change Log | CHANGELOG.md | [ ] |
| Test Report | docs/verification/release_test_report.md | [ ] |
| Known Issues | docs/software/KNOWN_ISSUES.md | [ ] |
| Build Configuration | build_info.txt | [ ] |

### 5.5 Release Approval

| Role | Signature | Date |
|------|-----------|------|
| Developer | | |
| SW Lead | | |
| QA | | |
| Project Manager | | |

---

## 6. Git Operations

### 6.1 Tagging

```bash
# Create annotated tag
git tag -a v1.0.0 -m "Release v1.0.0 - CDR Baseline"

# Push tag to remote
git push origin v1.0.0
```

### 6.2 Merge to Main

```bash
# Merge release branch to main
git checkout main
git merge --no-ff release/v1.0.0 -m "Merge release v1.0.0"
git push origin main
```

### 6.3 Branch Cleanup

```bash
# Delete release branch (optional)
git branch -d release/v1.0.0
git push origin --delete release/v1.0.0
```

---

## 7. Artifact Management

### 7.1 Release Artifacts

| Artifact | Format | Storage |
|----------|--------|---------|
| Binary image | .bin and .elf (`scripts/release_package.sh`) | Git release, backup |
| Source archive | .tar.gz | Git release |
| Documentation | PDF | Git release |
| Test report | PDF | Git release |
| Build info | `build_info.txt`, `stack_usage.txt`, `SHA256SUMS` | Git release |

### 7.2 Storage Locations

| Location | Purpose | Retention |
|----------|---------|-----------|
| GitHub Releases | Primary distribution | Permanent |
| Network drive | Backup | Permanent |
| USB archive | Offline backup | Permanent |

### 7.3 Artifact Verification

```bash
# Generate checksums
# scripts/release_package.sh writes SHA256SUMS; verify a copy with
sha256sum -c SHA256SUMS
```

---

## 8. Rollback Procedure

### 8.1 When to Rollback

- Critical bug discovered post-release
- Hardware incompatibility
- Safety concern

### 8.2 Rollback Steps

1. Identify last known good version
2. Notify all stakeholders
3. Revert to previous tag
4. Rebuild and verify
5. Document reason for rollback
6. Create anomaly report

```bash
# Checkout previous version
git checkout v0.9.0

# Rebuild
cmake -B build -DFLIGHT_BUILD=ON
cmake --build build
```

---

## 9. Emergency Release

### 9.1 Criteria

- Critical safety issue
- Mission-threatening bug
- Regulatory compliance issue

### 9.2 Expedited Process

| Normal Step | Emergency Action |
|-------------|------------------|
| Full test suite | Critical path tests only |
| Code review by 2 | Code review by 1 + verbal |
| PM approval | PM approval (verbal OK) |
| Documentation | Post-release documentation |

### 9.3 Post-Emergency

- Complete full testing within 48 hours
- Update all documentation
- Conduct lessons learned

---

## 10. Release Schedule

### 10.1 Planned Releases

| Version | Target | Milestone |
|---------|--------|-----------|
| 0.5.0 | PDR | Prototype |
| 1.0.0-rc1 | CDR-30d | Release candidate |
| 1.0.0 | CDR | CDR baseline |
| 1.1.0 | TRR | Test updates |
| 2.0.0 | FRR | Flight release |

### 10.2 Release Freeze Periods

| Period | Freeze Type |
|--------|-------------|
| CDR-7d to CDR | Hard freeze |
| TRR-3d to TRR | Soft freeze (critical only) |
| FRR-7d to Launch | Hard freeze |

---

## 11. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | Document number changed from SQSO-DEV-004, which `COMMIT_MESSAGES.md` also carried, to SQSO-DEV-005 (finding PI-17). No content change | SMART-QSO Team |
| C | 2026-10-24 | §5.2 note for the Rust flight software: where the version is set, how the image is built, the tests, and the steps not yet available (`package` target, build-info script) | SMART-QSO Team |
| D | 2026-10-25 | §5.2 rewritten for the Rust flight software: the release build packages the image with `scripts/release_package.sh` (the `package` target and build-info script it called did not exist); §7 artifacts and checksums to match | SMART-QSO Team |
| E | 2026-10-27 | §5.3.1 added: programming the flight board sets the SRAM2 parity option bit (`SRAM2_PE` = 0) before the image is loaded and checks it in the boot telemetry (RID AV-07, SRS-F154); §5.3 step 7 refers to it | SMART-QSO Team |
