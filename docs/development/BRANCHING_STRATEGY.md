# Branching Strategy
## SMART-QSO Flight Software

**Document Number:** SQSO-BS-001
**Revision:** A
**Date:** 2026-01-02

---

## 1. Overview

This document defines the Git branching strategy for SMART-QSO flight software development, ensuring traceability and configuration control per NPR 7150.2.

## 2. Branch Structure

```
main (protected)
├── develop
│   ├── feature/EPS-control
│   ├── feature/ADCS-algorithm
│   └── feature/beacon-generation
├── release/v1.0.0
└── hotfix/critical-fix
```

## 3. Branch Types

### 3.1 Main Branch

| Property | Value |
|----------|-------|
| Name | `main` |
| Purpose | Production-ready code |
| Protection | Required reviews, no direct push |
| Tags | Release versions (v1.0.0, v1.1.0) |

### 3.2 Develop Branch

| Property | Value |
|----------|-------|
| Name | `develop` |
| Purpose | Integration branch for features |
| Source | Feature branches |
| Target | Release branches |

### 3.3 Feature Branches

| Property | Value |
|----------|-------|
| Naming | `feature/<description>` |
| Purpose | New functionality development |
| Source | `develop` |
| Target | `develop` |
| Lifetime | Until feature complete |

Examples:
- `feature/eps-power-modes`
- `feature/adcs-bdot-control`
- `feature/beacon-ax25`

### 3.4 Release Branches

| Property | Value |
|----------|-------|
| Naming | `release/v<version>` |
| Purpose | Release preparation |
| Source | `develop` |
| Target | `main` and `develop` |
| Lifetime | Until release complete |

### 3.5 Hotfix Branches

| Property | Value |
|----------|-------|
| Naming | `hotfix/<description>` |
| Purpose | Critical production fixes |
| Source | `main` |
| Target | `main` and `develop` |
| Lifetime | Until fix deployed |

## 4. Workflow

### 4.1 Feature Development

```
1. Create feature branch from develop
   git checkout develop
   git checkout -b feature/new-feature

2. Develop and commit
   git add .
   git commit -m "feat: implement new feature"

3. Push and create PR
   git push origin feature/new-feature
   # Create PR to develop

4. After review, merge to develop
   # Squash merge preferred

5. Delete feature branch
   git branch -d feature/new-feature
```

### 4.2 Release Process

```
1. Create release branch
   git checkout develop
   git checkout -b release/v1.0.0

2. Final testing and fixes
   git commit -m "fix: pre-release bug fix"

3. Update version numbers
   git commit -m "chore: bump version to 1.0.0"

4. Merge to main
   git checkout main
   git merge release/v1.0.0
   git tag -a v1.0.0 -m "Release v1.0.0"

5. Merge back to develop
   git checkout develop
   git merge release/v1.0.0

6. Delete release branch
   git branch -d release/v1.0.0
```

### 4.3 Hotfix Process

```
1. Create hotfix branch from main
   git checkout main
   git checkout -b hotfix/critical-fix

2. Fix and commit
   git commit -m "fix: critical issue"

3. Merge to main
   git checkout main
   git merge hotfix/critical-fix
   git tag -a v1.0.1 -m "Hotfix v1.0.1"

4. Merge to develop
   git checkout develop
   git merge hotfix/critical-fix

5. Delete hotfix branch
   git branch -d hotfix/critical-fix
```

## 5. Commit Messages

### 5.1 Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 5.2 Types

| Type | Description |
|------|-------------|
| feat | New feature |
| fix | Bug fix |
| docs | Documentation |
| style | Formatting |
| refactor | Code restructuring |
| test | Adding tests |
| chore | Maintenance |

### 5.3 Examples

```
feat(eps): implement power mode state machine

Add SAFE, IDLE, and ACTIVE power modes with automatic
transitions based on battery state of charge.

Requirement: SRS-EPS-001
```

```
fix(beacon): correct CRC-16 calculation

The CRC polynomial was incorrectly initialized.
This caused beacon frames to fail FCS validation.

Fixes: #42
```

## 6. Code Review Requirements

### 6.1 Review Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Static analysis clean
- [ ] Coding standards followed
- [ ] Documentation updated
- [ ] Commit messages follow format

### 6.2 Approval Requirements

| Branch Target | Required Approvals |
|---------------|-------------------|
| develop | 1 reviewer |
| main | 2 reviewers |
| release/* | 2 reviewers |

## 7. Protected Branch Rules

### 7.1 Main Branch

- Require pull request reviews (2)
- Require status checks to pass
- Require signed commits
- No force push
- No deletion

### 7.2 Develop Branch

- Require pull request reviews (1)
- Require status checks to pass
- No force push

## 8. Version Numbering

### 8.1 Semantic Versioning

```
MAJOR.MINOR.PATCH

MAJOR: Breaking changes
MINOR: New features (backward compatible)
PATCH: Bug fixes (backward compatible)
```

### 8.2 Pre-release Versions

```
1.0.0-alpha.1  # Alpha release
1.0.0-beta.1   # Beta release
1.0.0-rc.1     # Release candidate
1.0.0          # Production release
```

## 9. Flight Software Baselines

| Milestone | Tag Format | Example |
|-----------|------------|---------|
| PDR | vX.Y.Z-pdr | v0.2.0-pdr |
| CDR | vX.Y.Z-cdr | v0.3.0-cdr |
| Flight | vX.Y.Z | v1.0.0 |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
