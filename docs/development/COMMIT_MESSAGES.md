# Commit Message Standard
## SMART-QSO Flight Software

**Document Number:** SQSO-DEV-004
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2 Section 4.5.3, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document establishes commit message standards for SMART-QSO software development to ensure traceability, consistency, and compliance with NASA configuration management requirements.

## 2. Scope

This standard applies to:
- All commits to the SMART-QSO repository
- Flight software, payload software, ground software
- Documentation and test code
- All team members and contributors

---

## 3. Conventional Commits Format

### 3.1 Message Structure

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 3.2 Header Line

| Component | Required | Description |
|-----------|----------|-------------|
| type | Yes | Category of change |
| scope | No | Affected subsystem |
| subject | Yes | Brief description (50 chars max) |

### 3.3 Types

| Type | Description | Example |
|------|-------------|---------|
| feat | New feature | `feat(beacon): add AX.25 frame CRC` |
| fix | Bug fix | `fix(eps): correct voltage threshold` |
| docs | Documentation | `docs(safety): update hazard analysis` |
| refactor | Code restructure | `refactor(hal): simplify I2C interface` |
| test | Test additions | `test(adcs): add detumble unit tests` |
| build | Build system | `build(cmake): add coverage target` |
| ci | CI/CD changes | `ci: add GitHub Actions workflow` |
| chore | Maintenance | `chore: update dependencies` |
| perf | Performance | `perf(sensor): reduce sampling latency` |
| revert | Revert commit | `revert: "feat(rf): add power mode"` |

### 3.4 Scopes

| Scope | Subsystem |
|-------|-----------|
| obc | OBC flight software |
| eps | Power subsystem |
| adcs | Attitude control |
| rf | Communications |
| beacon | Beacon generation |
| jetson | AI payload |
| hal | Hardware abstraction |
| fault | Fault management |
| sensor | Sensor interfaces |
| ground | Ground software |
| test | Test infrastructure |
| safety | Safety documentation |

---

## 4. Issue/Ticket Reference

### 4.1 Requirement

All commits affecting flight software SHALL reference:
- GitHub issue number, OR
- Requirement ID from RTM, OR
- Review action item

### 4.2 Format

```
Refs: #<issue-number>
Refs: <REQ-ID>
Refs: <CDR-AI-xxx>
```

### 4.3 Examples

```
fix(eps): correct undervoltage detection threshold

Battery voltage comparison was using wrong constant,
causing premature safe mode entry.

Refs: #42
Refs: SYS-EPS-005
```

```
feat(beacon): implement dynamic beacon intervals

Add power-aware beacon rate adjustment per ConOps.

Refs: MIS-COM-001, SYS-COM-012
Refs: #78
```

---

## 5. Sign-off Requirement

### 5.1 Developer Certificate of Origin

All commits SHALL include a sign-off line for traceability:

```
Signed-off-by: Developer Name <email@example.com>
```

### 5.2 Git Configuration

```bash
# Configure globally
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Add sign-off to commits
git commit -s -m "feat(scope): message"
```

### 5.3 Sign-off Meaning

By adding a sign-off, the contributor certifies:
1. The contribution is original or properly attributed
2. The contributor has rights to submit the change
3. The change is submitted under the project license
4. The contributor agrees to record keeping for traceability

---

## 6. Body Guidelines

### 6.1 When Required

A body is required for:
- Any change affecting flight software behavior
- Bug fixes (must explain root cause)
- Breaking changes
- Changes requiring review action item closure

### 6.2 Content

| Section | Content |
|---------|---------|
| What | Describe the change |
| Why | Motivation/reason |
| How | Implementation approach (if non-obvious) |
| Impact | Effects on other subsystems |

### 6.3 Formatting

- Wrap at 72 characters
- Use bullet points for multiple items
- Use present tense ("add" not "added")
- Use imperative mood ("change" not "changes")

---

## 7. Footer Guidelines

### 7.1 Breaking Changes

```
BREAKING CHANGE: <description>
```

### 7.2 Issue Closure

```
Closes: #<issue>
Fixes: #<issue>
Resolves: #<issue>
```

### 7.3 Co-authors

```
Co-Authored-By: Name <email@example.com>
```

### 7.4 Review References

```
Reviewed-by: Name <email@example.com>
Refs: CDR-AI-042
```

---

## 8. Examples

### 8.1 Simple Feature

```
feat(adcs): add B-dot controller gain configuration

Refs: SYS-ADCS-007
Signed-off-by: John Smith <john@example.com>
```

### 8.2 Bug Fix with Details

```
fix(fault): prevent false positive undervoltage faults

Root cause: ADC readings during Jetson boot showed transient
voltage dips below threshold due to inrush current.

Solution: Add 3-sample debounce filter before triggering fault.

Testing: Verified with simulated boot profile showing no
false triggers while maintaining response to actual faults.

Refs: #56
Refs: SYS-FDIR-003
Signed-off-by: Jane Doe <jane@example.com>
```

### 8.3 Breaking Change

```
refactor(hal)!: change I2C API to include bus parameter

The I2C interface now requires explicit bus selection to
support multiple I2C peripherals on the OBC.

BREAKING CHANGE: hal_i2c_read() now takes bus parameter as first arg.

Migration:
- hal_i2c_read(addr, data, len)
+ hal_i2c_read(I2C_BUS_0, addr, data, len)

Refs: SYS-OBC-015
Signed-off-by: Alice Johnson <alice@example.com>
```

### 8.4 Documentation Update

```
docs(safety): update software hazard analysis for CDR

Add hazards identified during PDR review:
- HAZ-SW-008: Watchdog timeout during flash write
- HAZ-SW-009: Invalid command sequence handling

Update mitigations to reflect current design.

Refs: PDR-AI-023, PDR-AI-024
Signed-off-by: Bob Williams <bob@example.com>
```

### 8.5 AI-Assisted Development

```
feat(beacon): add telemetry compression

Implement simple delta encoding for housekeeping telemetry
to reduce beacon size during safe mode.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
Refs: #91
Refs: MIS-COM-003
Signed-off-by: Developer Name <dev@example.com>
```

---

## 9. Commit Review Criteria

### 9.1 Automated Checks

| Check | Tool | Action |
|-------|------|--------|
| Format validation | commitlint | Reject non-conforming |
| Sign-off presence | git hook | Reject unsigned |
| Issue reference | git hook | Warn if missing |

### 9.2 Review Criteria

Commits are reviewed for:
- [ ] Correct type selection
- [ ] Appropriate scope
- [ ] Clear, descriptive subject
- [ ] Body present when required
- [ ] Issue/requirement reference
- [ ] Sign-off present
- [ ] Breaking changes documented

---

## 10. Configuration

### 10.1 Git Hooks

```bash
# .git/hooks/commit-msg
#!/bin/bash
# Validate conventional commit format
if ! grep -qE "^(feat|fix|docs|refactor|test|build|ci|chore|perf|revert)(\(.+\))?!?: .{1,50}" "$1"; then
    echo "ERROR: Commit message does not follow conventional commits format"
    exit 1
fi

# Check for sign-off
if ! grep -q "^Signed-off-by:" "$1"; then
    echo "ERROR: Commit must be signed off (-s flag)"
    exit 1
fi
```

### 10.2 Commitlint Configuration

```javascript
// commitlint.config.js
module.exports = {
    extends: ['@commitlint/config-conventional'],
    rules: {
        'scope-enum': [2, 'always', [
            'obc', 'eps', 'adcs', 'rf', 'beacon',
            'jetson', 'hal', 'fault', 'sensor',
            'ground', 'test', 'safety'
        ]],
        'signed-off-by': [2, 'always', 'Signed-off-by:']
    }
};
```

---

## 11. Traceability

### 11.1 Mapping to NPR 7150.2

| NPR 7150.2 | Commit Standard |
|------------|-----------------|
| 4.5.3.a | Type classification |
| 4.5.3.b | Issue/requirement reference |
| 4.5.3.c | Sign-off for attribution |
| 4.5.3.d | Body for rationale |

### 11.2 CM Database Integration

Commits are automatically linked to:
- GitHub Issues (via #reference)
- Requirements (via REQ-ID in refs)
- Review actions (via AI-xxx in refs)

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
