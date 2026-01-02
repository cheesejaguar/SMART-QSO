# Export Control Assessment
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-COMP-004
**Revision:** A
**Date:** 2026-01-02
**Reference:** ITAR (22 CFR 120-130), EAR (15 CFR 730-774)

---

## 1. Purpose

This document provides the export control classification assessment for the SMART-QSO CubeSat mission, ensuring compliance with U.S. export control regulations.

## 2. Regulatory Framework

### 2.1 Applicable Regulations

| Regulation | Agency | Scope |
|------------|--------|-------|
| ITAR (22 CFR 120-130) | State Dept. (DDTC) | Defense articles/services |
| EAR (15 CFR 730-774) | Commerce Dept. (BIS) | Dual-use items |
| OFAC Sanctions | Treasury Dept. | Country restrictions |

### 2.2 Classification Approach

All SMART-QSO components are assessed against:
1. USML (United States Munitions List) - ITAR Category XV (Spacecraft)
2. CCL (Commerce Control List) - EAR Category 9 (Spacecraft)

## 3. System-Level Classification

### 3.1 Overall Assessment

| Item | Classification | Rationale |
|------|----------------|-----------|
| SMART-QSO Spacecraft | EAR99 | Educational/research CubeSat |
| Flight Software | EAR99 | No encryption, open source |
| Ground Software | EAR99 | Standard amateur radio |
| Technical Data | EAR99 | Publicly available |

### 3.2 ITAR Category XV Analysis

| USML Criterion | Assessment | Notes |
|----------------|------------|-------|
| XV(a) Spacecraft | Not controlled | Does not meet thresholds |
| XV(e) Parts/components | Not controlled | COTS components |
| XV(f) Technical data | Not controlled | Basic research |

**Determination:** SMART-QSO does not meet USML Category XV thresholds due to:
- No propulsion system
- No precision pointing (> 0.1°)
- No encryption
- Educational/research purpose
- All COTS components

## 4. Component-Level Assessment

### 4.1 Structure and Mechanisms

| Component | Classification | ECCN | License |
|-----------|----------------|------|---------|
| 1U frame (aluminum) | EAR99 | N/A | NLR |
| Fasteners | EAR99 | N/A | NLR |
| Deployment switches | EAR99 | N/A | NLR |
| Antenna hinges | EAR99 | N/A | NLR |

### 4.2 Electrical Power System

| Component | Classification | ECCN | License |
|-----------|----------------|------|---------|
| Solar cells | EAR99 | N/A | NLR |
| Battery (Li-ion) | EAR99 | N/A | NLR |
| Power management IC | EAR99 | N/A | NLR |
| DC-DC converters | EAR99 | N/A | NLR |

### 4.3 On-Board Computer

| Component | Classification | ECCN | License |
|-----------|----------------|------|---------|
| Microcontroller | EAR99 | N/A | NLR |
| EEPROM/Flash | EAR99 | N/A | NLR |
| Oscillators | EAR99 | N/A | NLR |
| Flight software | EAR99 | N/A | NLR |

### 4.4 Attitude Determination and Control

| Component | Classification | ECCN | License |
|-----------|----------------|------|---------|
| Magnetometer | EAR99 | N/A | NLR |
| Sun sensors | EAR99 | N/A | NLR |
| Magnetorquers | EAR99 | N/A | NLR |
| ADCS software | EAR99 | N/A | NLR |

**Note:** ADCS provides < 10° pointing accuracy, well below ITAR threshold.

### 4.5 Communication System

| Component | Classification | ECCN | License |
|-----------|----------------|------|---------|
| VHF transceiver | EAR99 | N/A | NLR |
| Antenna | EAR99 | N/A | NLR |
| AX.25 modem | EAR99 | N/A | NLR |
| RF software | EAR99 | N/A | NLR |

**Note:** No encryption capability. All transmissions in the clear per amateur radio regulations.

### 4.6 AI Payload

| Component | Classification | ECCN | License |
|-----------|----------------|------|---------|
| NVIDIA Jetson | EAR99 | N/A | NLR |
| AI software | EAR99 | N/A | NLR |
| Open-source models | EAR99 | N/A | NLR |

**Note:** AI is used for beacon text generation only. No image recognition, targeting, or military applications.

## 5. Technical Data Assessment

### 5.1 Documentation Classification

| Document Type | Classification | Releasability |
|---------------|----------------|---------------|
| System design | EAR99 | Public release OK |
| Hardware drawings | EAR99 | Public release OK |
| Software source | EAR99 | Open source |
| Test procedures | EAR99 | Public release OK |
| Operations docs | EAR99 | Public release OK |

### 5.2 Fundamental Research Exclusion

SMART-QSO qualifies for the Fundamental Research Exclusion (FRE) under:
- ITAR 22 CFR 120.11(a)(8)
- EAR 15 CFR 734.8

| Criterion | Status |
|-----------|--------|
| Conducted at accredited institution | Yes (NASA/Universities) |
| Results ordinarily published | Yes (public data) |
| No publication restrictions | Yes |
| No access restrictions | Yes (with proper vetting) |

## 6. Personnel and Access

### 6.1 Foreign Person Access

| Activity | Requirement |
|----------|-------------|
| Facility access | Standard visitor screening |
| Technical discussions | No restrictions (FRE applies) |
| Software access | Open source, no restrictions |
| Hardware access | Standard lab protocols |

### 6.2 Deemed Export Considerations

| Item | Deemed Export? | Notes |
|------|----------------|-------|
| Design documents | No | FRE applies |
| Source code | No | Open source |
| Test data | No | Public information |
| Manufacturing | No | COTS assembly |

## 7. International Collaboration

### 7.1 Permitted Activities

| Activity | Status | Notes |
|----------|--------|-------|
| Data sharing | Permitted | Amateur radio, public |
| Technical exchange | Permitted | FRE applies |
| Ground station use | Permitted | SatNOGS network |
| Publication | Permitted | No restrictions |

### 7.2 Restricted Countries

Standard OFAC sanctions apply. No exports to:
- Cuba, Iran, North Korea, Syria
- Other sanctioned entities per OFAC SDN list

## 8. Licensing Requirements

### 8.1 Export License Summary

| Destination | License Required | Notes |
|-------------|------------------|-------|
| NATO allies | No (NLR) | EAR99 items |
| Most countries | No (NLR) | EAR99 items |
| Sanctioned countries | Prohibited | OFAC restrictions |

### 8.2 License Exceptions

| Exception | Applicability |
|-----------|---------------|
| TMP (Temporary) | Available for demos |
| BAG (Baggage) | Available for travel |
| TSR (Technology/Software) | Available |

## 9. Compliance Procedures

### 9.1 Export Control Checklist

- [ ] All team members briefed on export control
- [ ] Visitor access procedures in place
- [ ] Publication review process established
- [ ] Foreign national access documented
- [ ] Technology control plan implemented

### 9.2 Recordkeeping

| Record | Retention | Location |
|--------|-----------|----------|
| Classification determinations | 5 years | Project files |
| Foreign access logs | 5 years | Security office |
| Export documentation | 5 years | Export control office |

## 10. Points of Contact

| Role | Organization | Contact |
|------|--------------|---------|
| Export Control Officer | NASA Ames | [contact] |
| Project Manager | SMART-QSO | [contact] |
| Technology Transfer | NASA Ames | [contact] |

## 11. Certification

This assessment certifies that SMART-QSO components and technical data have been reviewed for export control classification. All items are classified as EAR99 (No License Required) based on the analysis herein.

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Export Control Officer | | | |
| Project Manager | | | |
| Chief Engineer | | | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
