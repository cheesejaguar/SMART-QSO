# SMART-QSO Formal Inspection Report Template

**Document ID**: SQSO-FIR-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: Template

## 1. Purpose

This template provides the format for Formal Inspection Reports per NPR 7150.2 §4.4.2.2 for software inspections, design reviews, and code reviews.

---

## FORMAL INSPECTION REPORT

### Inspection Information

| Field | Value |
|-------|-------|
| Inspection ID | FIR-XXXX |
| Date | YYYY-MM-DD |
| Type | [ ] Code [ ] Design [ ] Requirements [ ] Test |
| Item Inspected | |
| Version/Revision | |
| Author | |
| Moderator | |
| Duration | |

### Inspection Team

| Role | Name | Signature |
|------|------|-----------|
| Moderator | | |
| Author | | |
| Reader | | |
| Inspector 1 | | |
| Inspector 2 | | |
| Recorder | | |

### Preparation

| Inspector | Prep Time (hrs) | Issues Found |
|-----------|-----------------|--------------|
| | | |
| | | |
| | | |
| | | |
| **Total** | | |

### Inspection Metrics

| Metric | Value |
|--------|-------|
| Lines of code (or pages) | |
| Preparation time (total hrs) | |
| Meeting time (hrs) | |
| Total effort (hrs) | |
| Inspection rate (LOC/hr) | |
| Defect density (defects/KLOC) | |

### Issues Found

| ID | Line/Section | Severity | Type | Description | Status |
|----|--------------|----------|------|-------------|--------|
| 1 | | [ ] M [ ] m | | | [ ] Open |
| 2 | | [ ] M [ ] m | | | [ ] Open |
| 3 | | [ ] M [ ] m | | | [ ] Open |
| 4 | | [ ] M [ ] m | | | [ ] Open |
| 5 | | [ ] M [ ] m | | | [ ] Open |

**Severity**: M = Major, m = minor

### Issue Types

| Code | Type | Count |
|------|------|-------|
| LO | Logic | |
| IF | Interface | |
| DA | Data | |
| DC | Documentation | |
| ST | Standards | |
| PF | Performance | |
| OT | Other | |
| **Total** | | |

### Inspection Summary

| Category | Count |
|----------|-------|
| Major issues | |
| Minor issues | |
| Total issues | |
| Questions/suggestions | |

### Disposition

| Disposition | Selection |
|-------------|-----------|
| [ ] Accepted | No major issues, author resolves minor |
| [ ] Conditional | Re-verify corrections |
| [ ] Re-inspect | Major issues require re-inspection |

### Follow-Up Required

| Action | Owner | Due Date |
|--------|-------|----------|
| | | |
| | | |

### Verification of Corrections

| Issue ID | Verified By | Date | Status |
|----------|-------------|------|--------|
| | | | [ ] Closed |
| | | | [ ] Closed |
| | | | [ ] Closed |

### Signatures

```
INSPECTION CLOSURE

Moderator: _______________ Date: _______

All issues verified closed: [ ] Yes [ ] N/A

Final Status: [ ] CLOSED  [ ] OPEN
```

---

## 2. Instructions for Use

### 2.1 Inspection Types

| Type | When Used |
|------|-----------|
| Code Inspection | New or modified code |
| Design Inspection | Design documents |
| Requirements Inspection | Requirements changes |
| Test Inspection | Test procedures |

### 2.2 Severity Definitions

| Severity | Definition |
|----------|------------|
| Major | Will cause incorrect operation or maintenance problems |
| Minor | Does not affect operation but violates standards |

### 2.3 Process

```
INSPECTION PROCESS

1. Planning (Moderator)
   └─ Schedule inspection
   └─ Distribute materials

2. Preparation (Inspectors)
   └─ Review materials
   └─ Note issues
   └─ Log prep time

3. Meeting (Team)
   └─ Reader presents
   └─ Record issues
   └─ Determine disposition

4. Rework (Author)
   └─ Correct issues
   └─ Document changes

5. Follow-up (Moderator)
   └─ Verify corrections
   └─ Close inspection
   └─ Archive report
```

### 2.4 Exit Criteria

| Disposition | Exit Criteria |
|-------------|---------------|
| Accepted | No majors, minors logged |
| Conditional | Majors fixed, verify without meeting |
| Re-inspect | Majors require new inspection |

## 3. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
