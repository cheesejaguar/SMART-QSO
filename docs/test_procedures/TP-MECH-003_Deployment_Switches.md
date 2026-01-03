# Test Procedure: Deployment Switch Test
## TP-MECH-003

**Document Number:** SQSO-TP-MECH-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify deployment switch (kill switch) operation and redundancy.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Multimeter | Continuity mode |
| Test-POD or fixture | Switch actuator |
| Force gauge | 0-5 N |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure SW1 (actuated) | Closed (< 1Ω) | |
| 2 | Measure SW2 (actuated) | Closed (< 1Ω) | |
| 3 | Release SW1 | Open (> 1MΩ) | |
| 4 | Verify OBC detects release | Timer starts | |
| 5 | Re-actuate SW1 | Closed | |
| 6 | Measure actuation force | 0.5-2.0 N | |
| 7 | Repeat for SW2 | Same results | |
| 8 | Test series operation | Both must be open | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Contact resistance | < 1Ω closed |
| Open resistance | > 1MΩ |
| Actuation force | 0.5-2.0 N |
| OBC detection | Correct logic |
| Redundancy | Both switches functional |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
