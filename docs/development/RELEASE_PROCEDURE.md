# Software Release Procedure
## SMART-QSO Flight Software

**Document Number:** SQSO-DEV-005
**Revision:** C
**Date:** 2026-10-24
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

```bash
# 1. Ensure clean working directory
git status  # Should show clean

# 2. Checkout release branch
git checkout release/v1.0.0

# 3. Update version in CMakeLists.txt
# project(smart_qso_flight VERSION 1.0.0)

# 4. Build flight configuration
cmake -B build -DFLIGHT_BUILD=ON
cmake --build build

# 5. Run all tests
ctest --test-dir build --output-on-failure

# 6. Generate build artifacts
cmake --build build --target package

# 7. Record build information
./scripts/record_build_info.sh > build_info.txt
```

**As built (Revision C, 2026-10-24).** The flight software is Rust (`software/rust/README.md`), and the steps above
need these changes before the first release:

- **Version.** The version is in three places that must agree: `SMART_QSO_VERSION_STRING` in
  `software/flight/include/smart_qso.h`, `VERSION` in `software/rust/sqso-ffi/src/executive.rs` (both "0.2.0-dev"), and
  `version` in `software/rust/Cargo.toml` and `software/rust/firmware/Cargo.toml` (0.2.0). `CMakeLists.txt` has no
  project version.
- **Flight image.** `-DFLIGHT_BUILD=ON` builds the host simulation's flight configuration, not the image. The image is
  built with `cd software/rust/firmware && cargo build --release --locked` and is the ELF
  `software/rust/firmware/target/thumbv7em-none-eabihf/release/sqso-firmware`. A `.bin` must be made from it, for
  example with `arm-none-eabi-objcopy -O binary`. The build does not produce one.
- **Tests.** `ctest` as in step 5, plus `cargo test --workspace --locked` in `software/rust`. CI runs both, the coverage
  gates, clippy, fuzzing and the image checks (`software/rust/README.md`, section CI).
- **Not yet available.** There is no `package` target (step 6) and no build-info script (step 7). The
  build information to record includes the toolchain (`rustc --version`, pinned by
  `software/rust/rust-toolchain.toml`) and the two `Cargo.lock` files.

### 5.3 Release Verification

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 1 | Compile with FLIGHT_BUILD=ON | No errors/warnings | [ ] |
| 2 | Execute unit tests | 100% pass | [ ] |
| 3 | Execute integration tests | 100% pass | [ ] |
| 4 | Verify version string | Correct version | [ ] |
| 5 | Verify build info | Correct commit hash | [ ] |
| 6 | Binary size check | Within limits | [ ] |
| 7 | Load on target hardware | Boots correctly | [ ] |
| 8 | Execute functional test | All pass | [ ] |

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
| Binary image | .bin/.hex | Git release, backup |
| Source archive | .tar.gz | Git release |
| Documentation | PDF | Git release |
| Test report | PDF | Git release |
| Build info | .txt | Git release |

### 7.2 Storage Locations

| Location | Purpose | Retention |
|----------|---------|-----------|
| GitHub Releases | Primary distribution | Permanent |
| Network drive | Backup | Permanent |
| USB archive | Offline backup | Permanent |

### 7.3 Artifact Verification

```bash
# Generate checksums
sha256sum smart_qso_flight.bin > checksums.txt

# Verify checksums
sha256sum -c checksums.txt
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
